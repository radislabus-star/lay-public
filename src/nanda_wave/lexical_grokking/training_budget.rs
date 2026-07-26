use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

const POLL_INTERVAL: Duration = Duration::from_millis(250);
static ACTIVE: Mutex<Option<Arc<BudgetState>>> = Mutex::new(None);

struct BudgetState {
    maximum_rss_bytes: u64,
    peak_rss_bytes: AtomicU64,
    stage: Mutex<&'static str>,
    started_at: Instant,
    last_checkpoint_at: Mutex<Instant>,
    marker_path: PathBuf,
    stop: AtomicBool,
}

pub(super) struct TrainingBudgetGuard {
    state: Arc<BudgetState>,
    watcher: Option<thread::JoinHandle<()>>,
}

impl TrainingBudgetGuard {
    pub(super) fn install(maximum_rss_mib: usize, marker_path: &Path) -> Result<Self, String> {
        if maximum_rss_mib == 0 {
            return Err("L1 training RSS budget must be positive".to_string());
        }
        let maximum_rss_bytes = u64::try_from(maximum_rss_mib)
            .unwrap_or(u64::MAX)
            .saturating_mul(1024 * 1024);
        let started_at = Instant::now();
        let state = Arc::new(BudgetState {
            maximum_rss_bytes,
            peak_rss_bytes: AtomicU64::new(current_rss_bytes().unwrap_or_default()),
            stage: Mutex::new("startup"),
            started_at,
            last_checkpoint_at: Mutex::new(started_at),
            marker_path: marker_path.to_path_buf(),
            stop: AtomicBool::new(false),
        });
        {
            let mut active = ACTIVE
                .lock()
                .map_err(|_| "L1 training budget registry is poisoned".to_string())?;
            if active.is_some() {
                return Err("an L1 training RSS budget is already active".to_string());
            }
            *active = Some(Arc::clone(&state));
        }
        let watcher_state = Arc::clone(&state);
        let watcher = thread::spawn(move || watch(watcher_state));
        checkpoint("startup")?;
        Ok(Self {
            state,
            watcher: Some(watcher),
        })
    }

    pub(super) fn maximum_rss_bytes(&self) -> u64 {
        self.state.maximum_rss_bytes
    }

    pub(super) fn peak_rss_bytes(&self) -> u64 {
        self.state.peak_rss_bytes.load(Ordering::Relaxed)
    }
}

impl Drop for TrainingBudgetGuard {
    fn drop(&mut self) {
        self.state.stop.store(true, Ordering::Relaxed);
        if let Some(watcher) = self.watcher.take() {
            let _ = watcher.join();
        }
        if let Ok(mut active) = ACTIVE.lock() {
            if active
                .as_ref()
                .is_some_and(|state| Arc::ptr_eq(state, &self.state))
            {
                *active = None;
            }
        }
    }
}

pub(super) fn checkpoint(stage: &'static str) -> Result<(), String> {
    let state = ACTIVE
        .lock()
        .map_err(|_| "L1 training budget registry is poisoned".to_string())?
        .clone();
    let Some(state) = state else {
        return Ok(());
    };
    *state
        .stage
        .lock()
        .map_err(|_| "L1 training budget stage is poisoned".to_string())? = stage;
    let now = Instant::now();
    let stage_elapsed_ms = {
        let mut last_checkpoint_at = state
            .last_checkpoint_at
            .lock()
            .map_err(|_| "L1 training checkpoint clock is poisoned".to_string())?;
        let elapsed = now.duration_since(*last_checkpoint_at).as_millis();
        *last_checkpoint_at = now;
        elapsed
    };
    if let Some(rss_bytes) = current_rss_bytes() {
        update_peak(&state, rss_bytes);
        eprintln!(
            concat!(
                "l11_training stage={} elapsed_ms={} stage_elapsed_ms={} ",
                "rss_bytes={} peak_rss_bytes={} max_rss_bytes={}"
            ),
            stage,
            now.duration_since(state.started_at).as_millis(),
            stage_elapsed_ms,
            rss_bytes,
            state.peak_rss_bytes.load(Ordering::Relaxed),
            state.maximum_rss_bytes
        );
        if rss_bytes > state.maximum_rss_bytes {
            veto_and_exit(&state, stage, rss_bytes);
        }
    }
    Ok(())
}

fn watch(state: Arc<BudgetState>) {
    while !state.stop.load(Ordering::Relaxed) {
        thread::sleep(POLL_INTERVAL);
        let Some(rss_bytes) = current_rss_bytes() else {
            continue;
        };
        update_peak(&state, rss_bytes);
        if rss_bytes <= state.maximum_rss_bytes {
            continue;
        }
        let stage = state.stage.lock().map(|stage| *stage).unwrap_or("unknown");
        veto_and_exit(&state, stage, rss_bytes);
    }
}

fn veto_and_exit(state: &BudgetState, stage: &str, rss_bytes: u64) -> ! {
    let marker = format!(
        concat!(
            "{{\n",
            "  \"verdict\": \"VETO_RSS_BUDGET\",\n",
            "  \"stage\": \"{}\",\n",
            "  \"rss_bytes\": {},\n",
            "  \"peak_rss_bytes\": {},\n",
            "  \"max_rss_bytes\": {}\n",
            "}}\n"
        ),
        stage,
        rss_bytes,
        state.peak_rss_bytes.load(Ordering::Relaxed),
        state.maximum_rss_bytes
    );
    if let Some(parent) = state.marker_path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let _ = std::fs::write(&state.marker_path, marker);
    eprintln!(
        "l11_training verdict=VETO_RSS_BUDGET stage={stage} rss_bytes={rss_bytes} max_rss_bytes={}",
        state.maximum_rss_bytes
    );
    // A cooperative error cannot protect sshd while an allocator-heavy
    // phase is between checkpoints. Exit immediately and leave the final
    // package untouched; publication uses a separate atomic rename.
    unsafe {
        libc::_exit(86);
    }
}

fn update_peak(state: &BudgetState, rss_bytes: u64) {
    state.peak_rss_bytes.fetch_max(rss_bytes, Ordering::Relaxed);
}

fn current_rss_bytes() -> Option<u64> {
    let status = std::fs::read_to_string("/proc/self/status").ok()?;
    let rss_kib = status
        .lines()
        .find_map(|line| line.strip_prefix("VmRSS:"))?
        .split_whitespace()
        .next()?
        .parse::<u64>()
        .ok()?;
    Some(rss_kib.saturating_mul(1024))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn high_budget_checkpoint_is_non_destructive_and_reusable() {
        let marker = std::env::temp_dir().join(format!(
            "lay-l11-budget-{}-{}.json",
            std::process::id(),
            current_rss_bytes().unwrap_or_default()
        ));
        let _ = std::fs::remove_file(&marker);
        {
            let guard =
                TrainingBudgetGuard::install(1024 * 1024, &marker).expect("install high budget");
            checkpoint("unit_test").expect("checkpoint below high budget");
            assert!(guard.peak_rss_bytes() > 0);
        }
        {
            let _guard =
                TrainingBudgetGuard::install(1024 * 1024, &marker).expect("reinstall budget");
        }
        assert!(!marker.exists());
    }
}

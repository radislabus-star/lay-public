use std::io;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, OnceLock, RwLock};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use super::{
    composite, default_manifest_path, default_memory_path, ContextPhasePackage, L3CompositeMemory,
};

const REFRESH_CHECK_INTERVAL_MS: u64 = 1_000;
const REFRESH_WATCH_INTERVAL: Duration = Duration::from_millis(250);

static DEFAULT_RUNTIME: OnceLock<DefaultMemoryRuntime> = OnceLock::new();

struct DefaultMemoryRuntime {
    memory: RwLock<Arc<L3CompositeMemory>>,
    watcher_started: AtomicBool,
    refresh_check_ms: AtomicU64,
    refresh_in_flight: AtomicBool,
    load_generation: AtomicU64,
    refresh_successes: AtomicU64,
    refresh_failures: AtomicU64,
    last_refresh_ms: AtomicU64,
}

impl DefaultMemoryRuntime {
    fn new(memory: L3CompositeMemory) -> Self {
        Self {
            memory: RwLock::new(Arc::new(memory)),
            watcher_started: AtomicBool::new(false),
            refresh_check_ms: AtomicU64::new(0),
            refresh_in_flight: AtomicBool::new(false),
            load_generation: AtomicU64::new(1),
            refresh_successes: AtomicU64::new(0),
            refresh_failures: AtomicU64::new(0),
            last_refresh_ms: AtomicU64::new(unix_time_ms()),
        }
    }

    fn ensure_watcher(&'static self) {
        if self
            .watcher_started
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Relaxed)
            .is_err()
        {
            return;
        }
        if std::thread::Builder::new()
            .name("lay-l3-memory-watch".to_string())
            .spawn(move || loop {
                std::thread::sleep(REFRESH_WATCH_INTERVAL);
                self.maybe_reload();
            })
            .is_err()
        {
            self.watcher_started.store(false, Ordering::Release);
            self.record_failure("watcher_spawn_failed", None);
        } else {
            self.publish("watcher_started", None);
        }
    }

    fn maybe_reload(&'static self) {
        let now = unix_time_ms();
        let previous = self.refresh_check_ms.load(Ordering::Relaxed);
        if now.saturating_sub(previous) < REFRESH_CHECK_INTERVAL_MS
            || self
                .refresh_check_ms
                .compare_exchange(previous, now, Ordering::AcqRel, Ordering::Relaxed)
                .is_err()
        {
            return;
        }
        let manifest_path = default_manifest_path();
        let Ok(stamp) = composite::file_stamp(&manifest_path) else {
            return;
        };
        {
            let current = self
                .memory
                .read()
                .unwrap_or_else(|error| error.into_inner());
            if current.manifest_stamp == stamp {
                return;
            }
        }
        if self
            .refresh_in_flight
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Relaxed)
            .is_err()
        {
            return;
        }
        if std::thread::Builder::new()
            .name("lay-l3-memory-refresh".to_string())
            .spawn(move || {
                struct RefreshGuard(&'static DefaultMemoryRuntime);
                impl Drop for RefreshGuard {
                    fn drop(&mut self) {
                        self.0.refresh_in_flight.store(false, Ordering::Release);
                    }
                }
                let _guard = RefreshGuard(self);
                match L3CompositeMemory::load_manifest(&manifest_path) {
                    Ok(memory) => {
                        self.refresh_in_flight.store(false, Ordering::Release);
                        self.install(memory, "manifest_refresh", false);
                    }
                    Err(error) => {
                        self.refresh_in_flight.store(false, Ordering::Release);
                        self.record_failure("manifest_refresh_failed", Some(&error));
                    }
                }
            })
            .is_err()
        {
            self.refresh_in_flight.store(false, Ordering::Release);
            self.record_failure("refresh_spawn_failed", None);
        }
    }

    fn install(&self, memory: L3CompositeMemory, event: &str, force: bool) -> bool {
        {
            let mut current = self
                .memory
                .write()
                .unwrap_or_else(|error| error.into_inner());
            if !force && current.manifest_stamp == memory.manifest_stamp {
                return false;
            }
            *current = Arc::new(memory);
        }
        self.load_generation.fetch_add(1, Ordering::AcqRel);
        self.refresh_successes.fetch_add(1, Ordering::AcqRel);
        self.last_refresh_ms
            .store(unix_time_ms(), Ordering::Release);
        self.publish(event, None);
        true
    }

    fn record_failure(&self, event: &str, error: Option<&io::Error>) {
        self.refresh_failures.fetch_add(1, Ordering::AcqRel);
        self.last_refresh_ms
            .store(unix_time_ms(), Ordering::Release);
        self.publish(event, error.map(ToString::to_string).as_deref());
    }

    fn status_json(&self, event: &str, error: Option<&str>) -> serde_json::Value {
        let memory = self
            .memory
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        serde_json::json!({
            "kind": "l3_process_memory_runtime",
            "pid": std::process::id(),
            "process": process_label(),
            "event": event,
            "error": error,
            "memory_warm": true,
            "watcher_started": self.watcher_started.load(Ordering::Acquire),
            "refresh_in_flight": self.refresh_in_flight.load(Ordering::Acquire),
            "load_generation": self.load_generation.load(Ordering::Acquire),
            "refresh_successes": self.refresh_successes.load(Ordering::Acquire),
            "refresh_failures": self.refresh_failures.load(Ordering::Acquire),
            "last_check_unix_ms": self.refresh_check_ms.load(Ordering::Acquire),
            "last_refresh_unix_ms": self.last_refresh_ms.load(Ordering::Acquire),
            "memory": memory.report(),
        })
    }

    fn publish(&self, event: &str, error: Option<&str>) {
        let Some(path) = runtime_status_path() else {
            return;
        };
        let Ok(mut bytes) = serde_json::to_vec_pretty(&self.status_json(event, error)) else {
            return;
        };
        bytes.push(b'\n');
        let _ = crate::private_file::write_private_bytes_atomic(&path, &bytes);
    }
}

pub(crate) fn warm_default_memory() {
    let runtime = default_runtime();
    runtime.ensure_watcher();
}

pub(crate) fn default_memory_is_warm() -> bool {
    DEFAULT_RUNTIME.get().is_some()
}

pub(crate) fn with_default_memory<T>(read: impl FnOnce(&ContextPhasePackage) -> T) -> T {
    let runtime = default_runtime();
    runtime.ensure_watcher();
    runtime.maybe_reload();
    let memory = runtime
        .memory
        .read()
        .unwrap_or_else(|error| error.into_inner());
    read(memory.package())
}

pub(crate) fn reload_default_memory() -> io::Result<serde_json::Value> {
    let manifest_path = default_manifest_path();
    let memory = if manifest_path.is_file() {
        L3CompositeMemory::load_manifest(&manifest_path)?
    } else {
        L3CompositeMemory::from_package(&default_memory_path())?
    };
    let report = memory.report();
    default_runtime().install(memory, "manual_reload", true);
    Ok(report)
}

pub(crate) fn default_memory_runtime_status_json() -> serde_json::Value {
    default_runtime().status_json("snapshot", None)
}

fn default_runtime() -> &'static DefaultMemoryRuntime {
    DEFAULT_RUNTIME.get_or_init(|| {
        let runtime = DefaultMemoryRuntime::new(load_default_composite());
        runtime.publish("initial_load", None);
        runtime
    })
}

fn load_default_composite() -> L3CompositeMemory {
    let manifest_path = default_manifest_path();
    if manifest_path.is_file() {
        if let Ok(memory) = L3CompositeMemory::load_manifest(&manifest_path) {
            return memory;
        }
    }
    L3CompositeMemory::from_package(&default_memory_path())
        .unwrap_or_else(|_| L3CompositeMemory::empty(default_memory_path()))
}

fn runtime_status_path() -> Option<PathBuf> {
    let root = std::env::var_os("LAY_NANDA_L3_RUNTIME_STATUS_DIR")
        .map(PathBuf::from)
        .or_else(|| {
            std::env::var_os("XDG_RUNTIME_DIR")
                .map(PathBuf::from)
                .map(|path| path.join("lay/l3-context"))
        })?;
    Some(root.join(format!("{}-{}.json", process_label(), std::process::id())))
}

fn process_label() -> String {
    let label = std::env::current_exe()
        .ok()
        .and_then(|path| {
            path.file_name()
                .map(|name| name.to_string_lossy().into_owned())
        })
        .unwrap_or_else(|| "lay-process".to_string());
    label
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() || matches!(character, '.' | '_' | '-') {
                character
            } else {
                '_'
            }
        })
        .collect()
}

fn unix_time_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
        .min(u128::from(u64::MAX)) as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn arc_load_generation_changes_only_after_install() {
        let first = L3CompositeMemory::empty(PathBuf::from("first.nwpc"));
        let runtime = DefaultMemoryRuntime::new(first);
        assert_eq!(runtime.load_generation.load(Ordering::Acquire), 1);

        let same = L3CompositeMemory::empty(PathBuf::from("same.nwpc"));
        assert!(!runtime.install(same, "test", false));
        assert_eq!(runtime.load_generation.load(Ordering::Acquire), 1);

        let mut changed = L3CompositeMemory::empty(PathBuf::from("changed.nwpc"));
        changed.manifest_stamp = 42;
        assert!(runtime.install(changed, "test", false));
        assert_eq!(runtime.load_generation.load(Ordering::Acquire), 2);
        assert_eq!(runtime.refresh_successes.load(Ordering::Acquire), 1);
    }

    #[test]
    fn process_label_is_safe_for_runtime_file_name() {
        let label = process_label();
        assert!(!label.is_empty());
        assert!(label
            .chars()
            .all(|character| character.is_ascii_alphanumeric()
                || matches!(character, '.' | '_' | '-')));
    }
}

use std::sync::{Arc, Condvar, Mutex, OnceLock};
use std::time::{Duration, Instant};

use lay::config::LayConfig;
use lay::ime_correction::{
    ActiveCompositionAutocorrectDecision, ActiveCompositionAutocorrectRequest,
};

const SPACE_PREFETCH_WAIT_BUDGET: Duration = Duration::from_millis(8);

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct SpaceAutocorrectKey {
    path: String,
    tail_epoch: u64,
    tail: String,
    active_layout_is_ru: bool,
}

impl SpaceAutocorrectKey {
    pub(crate) fn new(
        path: String,
        tail_epoch: u64,
        tail: String,
        active_layout_is_ru: bool,
    ) -> Self {
        Self {
            path,
            tail_epoch,
            tail,
            active_layout_is_ru,
        }
    }
}

pub(crate) struct SpaceAutocorrectWork {
    pub(crate) key: SpaceAutocorrectKey,
    pub(crate) boundary_text: String,
    pub(crate) committed_tail: String,
    pub(crate) config: LayConfig,
}

pub(crate) enum SpaceAutocorrectLookup {
    Ready {
        decision: Option<ActiveCompositionAutocorrectDecision>,
        decision_us: u128,
    },
    NotReady,
}

struct CompletedWork {
    key: SpaceAutocorrectKey,
    decision: Option<ActiveCompositionAutocorrectDecision>,
    decision_us: u128,
}

#[derive(Default)]
struct WorkerState {
    generation: u64,
    desired: Option<SpaceAutocorrectWork>,
    completed: Option<CompletedWork>,
}

struct Worker {
    state: Arc<(Mutex<WorkerState>, Condvar)>,
}

impl Worker {
    fn start() -> Self {
        let state = Arc::new((Mutex::new(WorkerState::default()), Condvar::new()));
        let worker_state = Arc::clone(&state);
        std::thread::Builder::new()
            .name("lay-space-prefetch".to_string())
            .spawn(move || run_worker(worker_state))
            .expect("failed to start Space autocorrect prefetch worker");
        Self { state }
    }

    fn schedule(&self, work: SpaceAutocorrectWork) {
        let (lock, wake) = &*self.state;
        let Ok(mut state) = lock.lock() else {
            return;
        };
        state.generation = state.generation.wrapping_add(1);
        state.desired = Some(work);
        state.completed = None;
        wake.notify_one();
    }

    fn take(&self, key: &SpaceAutocorrectKey) -> SpaceAutocorrectLookup {
        let started = Instant::now();
        let (lock, wake) = &*self.state;
        let Ok(mut state) = lock.lock() else {
            return SpaceAutocorrectLookup::NotReady;
        };
        loop {
            if let Some(completed) = state.completed.take() {
                if completed.key == *key {
                    return SpaceAutocorrectLookup::Ready {
                        decision: completed.decision,
                        decision_us: completed.decision_us,
                    };
                }
            }
            let remaining = SPACE_PREFETCH_WAIT_BUDGET.saturating_sub(started.elapsed());
            if remaining.is_zero() {
                return SpaceAutocorrectLookup::NotReady;
            }
            let Ok((next, timeout)) = wake.wait_timeout(state, remaining) else {
                return SpaceAutocorrectLookup::NotReady;
            };
            state = next;
            if timeout.timed_out() && state.completed.is_none() {
                return SpaceAutocorrectLookup::NotReady;
            }
        }
    }
}

fn run_worker(shared: Arc<(Mutex<WorkerState>, Condvar)>) {
    loop {
        let (generation, work) = {
            let (lock, wake) = &*shared;
            let Ok(mut state) = lock.lock() else {
                return;
            };
            while state.desired.is_none() {
                let Ok(next) = wake.wait(state) else {
                    return;
                };
                state = next;
            }
            let generation = state.generation;
            let work = state.desired.take().expect("desired work checked above");
            (generation, work)
        };

        let started = Instant::now();
        let decision = lay::ime_correction::decide_active_composition_autocorrect(
            ActiveCompositionAutocorrectRequest {
                text: &work.boundary_text,
                committed_tail: &work.committed_tail,
                config: &work.config,
                active_layout_is_ru: Some(work.key.active_layout_is_ru),
            },
        );
        let completed = CompletedWork {
            key: work.key,
            decision,
            decision_us: started.elapsed().as_micros(),
        };

        let (lock, wake) = &*shared;
        let Ok(mut state) = lock.lock() else {
            return;
        };
        if state.generation == generation {
            state.completed = Some(completed);
            wake.notify_all();
        }
    }
}

fn worker() -> &'static Worker {
    static WORKER: OnceLock<Worker> = OnceLock::new();
    WORKER.get_or_init(Worker::start)
}

pub(crate) fn schedule(work: SpaceAutocorrectWork) {
    worker().schedule(work);
}

pub(crate) fn take(key: &SpaceAutocorrectKey) -> SpaceAutocorrectLookup {
    worker().take(key)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn take_waits_for_matching_inflight_generation_within_budget() {
        let state = Arc::new((Mutex::new(WorkerState::default()), Condvar::new()));
        let worker = Worker {
            state: Arc::clone(&state),
        };
        let key = SpaceAutocorrectKey::new("/test".to_string(), 7, "yt".to_string(), false);
        let completed_key = key.clone();
        let producer = std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(2));
            let (lock, wake) = &*state;
            let mut state = lock.lock().expect("worker state");
            state.completed = Some(CompletedWork {
                key: completed_key,
                decision: None,
                decision_us: 2_000,
            });
            wake.notify_all();
        });

        let lookup = worker.take(&key);
        producer.join().expect("producer");

        assert!(matches!(
            lookup,
            SpaceAutocorrectLookup::Ready {
                decision: None,
                decision_us: 2_000
            }
        ));
    }
}

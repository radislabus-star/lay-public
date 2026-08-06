use std::sync::{Arc, Condvar, Mutex, OnceLock};
use std::time::Instant;

use lay::config::LayConfig;
use lay::ime_correction::{
    ActiveCompositionAutocorrectDecision, ActiveCompositionAutocorrectRequest,
};

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
        let (lock, _) = &*self.state;
        let Ok(mut state) = lock.lock() else {
            return SpaceAutocorrectLookup::NotReady;
        };
        let Some(completed) = state.completed.take() else {
            return SpaceAutocorrectLookup::NotReady;
        };
        if completed.key != *key {
            return SpaceAutocorrectLookup::NotReady;
        }
        SpaceAutocorrectLookup::Ready {
            decision: completed.decision,
            decision_us: completed.decision_us,
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

        let (lock, _) = &*shared;
        let Ok(mut state) = lock.lock() else {
            return;
        };
        if state.generation == generation {
            state.completed = Some(completed);
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

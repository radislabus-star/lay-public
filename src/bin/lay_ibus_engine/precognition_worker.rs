use std::sync::{Arc, Condvar, Mutex, OnceLock};
use std::time::{Duration, Instant};

use zbus::Connection;

use super::engine::{InputFrameIdentity, LayIbusEngine};
use super::preedit::{
    materialize_precognition_candidates_observed, PrecognitionInput,
    PrecognitionMaterializationTiming,
};
use super::trace;
use lay::typing_cpu::TypingCpu;

const PRECOGNITION_DISPLAY_DEADLINE: Duration = Duration::from_millis(150);

pub(crate) struct PrecognitionWork {
    pub(crate) identity: InputFrameIdentity,
    pub(crate) input: PrecognitionInput,
    pub(crate) connection: Connection,
    pub(crate) scheduled_at: Instant,
}

#[derive(Default)]
struct WorkerState {
    generation: u64,
    desired: Option<PrecognitionWork>,
}

impl WorkerState {
    fn replace(&mut self, work: PrecognitionWork) {
        self.generation = self.generation.wrapping_add(1);
        self.desired = Some(work);
    }

    fn cancel(&mut self) {
        self.generation = self.generation.wrapping_add(1);
        self.desired = None;
    }
}

struct Worker {
    state: Arc<(Mutex<WorkerState>, Condvar)>,
}

static WORKER: OnceLock<Worker> = OnceLock::new();

struct ApplyCompletion {
    stage: &'static str,
    display_age: Duration,
}

impl Worker {
    fn start() -> Self {
        let state = Arc::new((Mutex::new(WorkerState::default()), Condvar::new()));
        let worker_state = Arc::clone(&state);
        std::thread::Builder::new()
            .name("lay-precognition".to_string())
            .spawn(move || run_worker(worker_state))
            .expect("failed to start IME precognition worker");
        Self { state }
    }

    fn schedule(&self, work: PrecognitionWork) {
        let (lock, wake) = &*self.state;
        let Ok(mut state) = lock.lock() else {
            return;
        };
        state.replace(work);
        wake.notify_one();
    }

    fn cancel(&self) {
        let (lock, _) = &*self.state;
        if let Ok(mut state) = lock.lock() {
            state.cancel();
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

        let trace_enabled = trace::enabled();
        let candidate_memory_warm_before =
            trace_enabled && TypingCpu::ime_candidate_memory_is_warm();
        let started = Instant::now();
        let materialized = materialize_precognition_candidates_observed(&work.input);
        let candidate_memory_warm_after =
            trace_enabled && TypingCpu::ime_candidate_memory_is_warm();
        let candidates = materialized.candidates;
        let material_us = started.elapsed().as_micros();
        let display_age = work.scheduled_at.elapsed();
        let identity = work.identity.clone();
        let token = identity.observed_token.clone();
        let top = candidates
            .first()
            .map(|candidate| candidate.display_text().to_string());
        if !generation_is_current(&shared, generation) {
            record_completion(
                "superseded",
                generation,
                &identity,
                &materialized.timing,
                material_us,
                display_age.as_micros(),
                candidates.len(),
                trace_enabled,
                candidate_memory_warm_before,
                candidate_memory_warm_after,
                &token,
                top.as_deref(),
            );
            continue;
        }
        if !display_age_is_fresh(display_age) {
            zbus::block_on(retire_late(Arc::clone(&shared), generation, &work));
            record_completion(
                "late",
                generation,
                &identity,
                &materialized.timing,
                material_us,
                display_age.as_micros(),
                candidates.len(),
                trace_enabled,
                candidate_memory_warm_before,
                candidate_memory_warm_after,
                &token,
                top.as_deref(),
            );
            continue;
        }

        let candidate_count = candidates.len();
        let completion = zbus::block_on(apply_completed(
            Arc::clone(&shared),
            generation,
            work,
            candidates,
        ));
        record_completion(
            completion.stage,
            generation,
            &identity,
            &materialized.timing,
            material_us,
            completion.display_age.as_micros(),
            candidate_count,
            trace_enabled,
            candidate_memory_warm_before,
            candidate_memory_warm_after,
            &token,
            top.as_deref(),
        );
    }
}

async fn retire_late(
    shared: Arc<(Mutex<WorkerState>, Condvar)>,
    generation: u64,
    work: &PrecognitionWork,
) {
    if !generation_is_current(&shared, generation) {
        return;
    }
    let Ok(iface_ref) = work
        .connection
        .object_server()
        .interface::<_, LayIbusEngine>(work.identity.path.as_str())
        .await
    else {
        return;
    };
    let emitter = iface_ref.signal_emitter();
    let mut engine = iface_ref.get_mut().await;
    if !generation_is_current(&shared, generation)
        || !engine.precognition_identity_matches(&work.identity)
    {
        return;
    }
    let mut output = super::output::EngineOutput::legacy(emitter);
    let _ = engine.retire_late_precognition(&mut output).await;
}

async fn apply_completed(
    shared: Arc<(Mutex<WorkerState>, Condvar)>,
    generation: u64,
    work: PrecognitionWork,
    candidates: Vec<lay::typing_cpu::ImeCandidateProposal>,
) -> ApplyCompletion {
    if !generation_is_current(&shared, generation) {
        return ApplyCompletion {
            stage: "discarded",
            display_age: work.scheduled_at.elapsed(),
        };
    }
    let Ok(iface_ref) = work
        .connection
        .object_server()
        .interface::<_, LayIbusEngine>(work.identity.path.as_str())
        .await
    else {
        return ApplyCompletion {
            stage: "discarded",
            display_age: work.scheduled_at.elapsed(),
        };
    };
    let emitter = iface_ref.signal_emitter();
    let mut engine = iface_ref.get_mut().await;
    if !generation_is_current(&shared, generation)
        || !engine.precognition_identity_matches(&work.identity)
    {
        return ApplyCompletion {
            stage: "discarded",
            display_age: work.scheduled_at.elapsed(),
        };
    }
    let display_age = work.scheduled_at.elapsed();
    if !display_age_is_fresh(display_age) {
        let mut output = super::output::EngineOutput::legacy(emitter);
        let _ = engine.retire_late_precognition(&mut output).await;
        return ApplyCompletion {
            stage: "late",
            display_age,
        };
    }
    let mut output = super::output::EngineOutput::legacy(emitter);
    let stage = if engine
        .apply_background_precognition(&mut output, candidates)
        .await
        .is_ok()
    {
        "applied"
    } else {
        "discarded"
    };
    ApplyCompletion { stage, display_age }
}

fn generation_is_current(shared: &Arc<(Mutex<WorkerState>, Condvar)>, generation: u64) -> bool {
    shared
        .0
        .lock()
        .is_ok_and(|state| state.generation == generation)
}

fn display_age_is_fresh(age: Duration) -> bool {
    age <= PRECOGNITION_DISPLAY_DEADLINE
}

#[expect(
    clippy::too_many_arguments,
    reason = "existing explicit boundary contract"
)]
fn record_completion(
    stage: &'static str,
    generation: u64,
    identity: &InputFrameIdentity,
    timing: &PrecognitionMaterializationTiming,
    material_us: u128,
    display_age_us: u128,
    candidates: usize,
    trace_enabled: bool,
    candidate_memory_warm_before: bool,
    candidate_memory_warm_after: bool,
    token: &str,
    top: Option<&str>,
) {
    #[cfg(test)]
    completion_observation::notify(identity, stage, candidates, timing.word.cache_hit);
    if !trace_enabled {
        return;
    }
    trace::record(format!(
        r#"{{"kind":"ibus_precognition_worker","stage":{},"generation":{generation},"tail_epoch":{},"material_us":{material_us},"display_age_us":{display_age_us},"candidates":{candidates},"candidate_memory_warm_before":{candidate_memory_warm_before},"candidate_memory_warm_after":{candidate_memory_warm_after}}}"#,
        serde_json::to_string(stage).unwrap_or_else(|_| "\"unknown\"".to_string()),
        identity.tail_epoch,
    ));
    trace::record_precognition_timing(
        stage,
        generation,
        identity,
        timing,
        candidates,
        Some(token),
        top,
    );
}

fn worker() -> &'static Worker {
    WORKER.get_or_init(Worker::start)
}

pub(crate) fn schedule(work: PrecognitionWork) {
    worker().schedule(work);
}

pub(crate) fn cancel() {
    if let Some(worker) = WORKER.get() {
        worker.cancel();
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Condvar, Mutex};

    use lay::config::LayConfig;

    use super::{display_age_is_fresh, generation_is_current, InputFrameIdentity, WorkerState};
    use std::time::Duration;

    fn identity() -> InputFrameIdentity {
        InputFrameIdentity::new(
            "/engine/a".to_string(),
            Some("field-a".to_string()),
            7,
            "context token".to_string(),
            "context ".to_string(),
            "token".to_string(),
            true,
            true,
            &LayConfig::default(),
        )
    }

    #[test]
    fn result_identity_binds_every_visible_input_dimension() {
        let expected = identity();
        let mut changed = expected.clone();
        changed.path = "/engine/b".to_string();
        assert_ne!(expected, changed);

        let mut changed = expected.clone();
        changed.focus_receipt = Some("field-b".to_string());
        assert_ne!(expected, changed);

        let mut changed = expected.clone();
        changed.tail_epoch += 1;
        assert_ne!(expected, changed);

        let mut changed = expected.clone();
        changed.context_prefix.push_str("other ");
        assert_ne!(expected, changed);

        let mut changed = expected.clone();
        changed.observed_token.push('x');
        assert_ne!(expected, changed);

        let mut changed = expected.clone();
        changed.active_composition = false;
        assert_ne!(expected, changed);

        let mut changed = expected.clone();
        changed.committed_tail.push('x');
        assert_ne!(expected, changed);

        let mut changed = expected.clone();
        changed.active_layout_is_ru = false;
        assert_ne!(expected, changed);
    }

    #[test]
    fn superseded_generation_cannot_be_applied() {
        let shared = Arc::new((
            Mutex::new(WorkerState {
                generation: 41,
                desired: None,
            }),
            Condvar::new(),
        ));
        assert!(generation_is_current(&shared, 41));

        shared.0.lock().expect("worker state").cancel();

        assert!(!generation_is_current(&shared, 41));
        assert!(generation_is_current(&shared, 42));
    }

    #[test]
    fn late_display_results_are_not_publishable() {
        assert!(display_age_is_fresh(Duration::from_millis(150)));
        assert!(!display_age_is_fresh(Duration::from_millis(151)));
    }
}

#[cfg(test)]
pub(crate) mod completion_observation {
    use super::*;
    use std::sync::mpsc::{self, Receiver, Sender};

    struct Observation {
        identity: InputFrameIdentity,
        sender: Sender<(&'static str, usize, bool)>,
    }

    static OBSERVATION: Mutex<Option<Observation>> = Mutex::new(None);

    pub(crate) fn observe(identity: InputFrameIdentity) -> Receiver<(&'static str, usize, bool)> {
        let (sender, receiver) = mpsc::channel();
        *OBSERVATION.lock().unwrap() = Some(Observation { identity, sender });
        receiver
    }

    pub(super) fn notify(
        identity: &InputFrameIdentity,
        stage: &'static str,
        candidates: usize,
        cache_hit: bool,
    ) {
        let mut observer = OBSERVATION.lock().unwrap();
        if observer
            .as_ref()
            .is_some_and(|pending| &pending.identity == identity)
        {
            let pending = observer.take().unwrap();
            let _ = pending.sender.send((stage, candidates, cache_hit));
        }
    }
}

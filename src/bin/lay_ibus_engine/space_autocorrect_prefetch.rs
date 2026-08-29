use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Condvar, Mutex, OnceLock, TryLockError};
use std::time::{Duration, Instant};

use lay::config::LayConfig;
use lay::ime_correction::{
    decide_active_composition_autocorrect_observed,
    decide_active_composition_autocorrect_observed_with_exact,
    prepare_exact_layout_active_composition_autocorrect_observed,
    ActiveCompositionAutocorrectDecision, ActiveCompositionAutocorrectRequest,
    ActiveCompositionAutocorrectTelemetry, AutocorrectNoApplyStage,
};

use super::engine::InputFrameIdentity;
use super::trace;

// Leave room inside the 4 ms product deadline for lock and wake-up overhead.
const SPACE_FULL_WAIT_BUDGET: Duration = Duration::from_micros(3_500);
const MAX_PREFETCH_PATH_LANES: usize = 8;

pub(crate) struct SpaceAutocorrectWork {
    pub(crate) identity: InputFrameIdentity,
    pub(crate) config: LayConfig,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PreparedLeaseKind {
    Full,
    ExactLayout,
}

pub(crate) struct PreparedCorrectionLease {
    pub(crate) identity: InputFrameIdentity,
    pub(crate) decision: ActiveCompositionAutocorrectDecision,
    pub(crate) decision_us: u128,
    pub(crate) worker_generation: u64,
    pub(crate) material_generation: u64,
    pub(crate) kind: PreparedLeaseKind,
    pub(crate) exact_certificate:
        Option<lay::exact_layout_authority::ExactLayoutContourCertificate>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PreparedNoApplyStage {
    Rank,
    Verifier,
    Infrastructure,
}

#[expect(
    clippy::large_enum_variant,
    reason = "boxing would change the bounded prefetch handoff"
)]
pub(crate) enum SpaceAutocorrectLookup {
    Ready(PreparedCorrectionLease),
    NoApply(PreparedNoApplyStage),
    NotReady,
    Stale,
}

pub(crate) struct SpaceAutocorrectLookupReceipt {
    pub(crate) lookup: SpaceAutocorrectLookup,
    pub(crate) wait_us: u128,
    pub(crate) worker_generation: u64,
}

#[expect(
    clippy::large_enum_variant,
    reason = "boxing would change the bounded prefetch slot"
)]
enum PreparedFullOutcome {
    Apply(PreparedCorrectionLease),
    NoApply {
        stage: PreparedNoApplyStage,
        decision_us: u128,
    },
}

#[expect(
    clippy::large_enum_variant,
    reason = "boxing would change the bounded prefetch slot"
)]
enum FullSlotState {
    Pending,
    Terminal(PreparedFullOutcome),
}

#[expect(
    clippy::large_enum_variant,
    reason = "boxing would change the bounded prefetch slot"
)]
enum ExactSlotState {
    Absent,
    Prepared(PreparedCorrectionLease),
}

struct PreparedDecisionSlot {
    identity: InputFrameIdentity,
    request_generation: u64,
    material_generation: u64,
    full: FullSlotState,
    exact: ExactSlotState,
}

struct DesiredWork {
    worker_generation: u64,
    material_generation: u64,
    work: SpaceAutocorrectWork,
    exact_certificate: Option<lay::exact_layout_authority::ExactLayoutContourCertificate>,
}

#[derive(Default)]
struct WorkerState {
    generation: u64,
    slot: Option<PreparedDecisionSlot>,
    desired: Option<DesiredWork>,
}

struct Worker {
    state: Arc<(Mutex<WorkerState>, Condvar)>,
    latest_request_generation: Arc<AtomicU64>,
    stop: Arc<AtomicBool>,
}

struct WorkerLane {
    worker: Arc<Worker>,
    last_used: u64,
}

#[derive(Default)]
struct WorkerPool {
    lanes: HashMap<String, WorkerLane>,
    clock: u64,
}

static WORKERS: OnceLock<Mutex<WorkerPool>> = OnceLock::new();

impl Worker {
    fn start() -> Self {
        let state = Arc::new((Mutex::new(WorkerState::default()), Condvar::new()));
        let worker_state = Arc::clone(&state);
        let latest_request_generation = Arc::new(AtomicU64::new(0));
        let worker_latest_request_generation = Arc::clone(&latest_request_generation);
        let stop = Arc::new(AtomicBool::new(false));
        let worker_stop = Arc::clone(&stop);
        std::thread::Builder::new()
            .name("lay-space-prefetch".to_string())
            .spawn(move || run_worker(worker_state, worker_latest_request_generation, worker_stop))
            .expect("failed to start Space autocorrect prefetch worker");
        Self {
            state,
            latest_request_generation,
            stop,
        }
    }

    fn schedule(&self, work: SpaceAutocorrectWork) {
        let material_generation = lay::nanda_wave::candidate_material_generation();
        let Some(worker_generation) = self.begin_request(&work.identity, material_generation)
        else {
            trace::record(
                r#"{"kind":"ibus_space_exact_layout_lease","status":"register_unavailable"}"#,
            );
            return;
        };
        let exact_started = Instant::now();
        let exact = prepare_inline_exact(&work);
        let exact_us = exact_started.elapsed().as_micros();
        let (exact_decision, exact_certificate) = match exact {
            Some(prepared) => (prepared.decision, Some(prepared.certificate)),
            None => (None, None),
        };

        if !self.finish_request(
            worker_generation,
            material_generation,
            work,
            exact_decision,
            exact_certificate,
            exact_us,
        ) {
            trace::record(
                r#"{"kind":"ibus_space_exact_layout_lease","status":"publish_unavailable_or_superseded"}"#,
            );
        }
    }

    fn begin_request(
        &self,
        identity: &InputFrameIdentity,
        material_generation: u64,
    ) -> Option<u64> {
        let worker_generation = reserve_generation(&self.latest_request_generation);
        let (lock, wake) = &*self.state;
        let mut state = lock.try_lock().ok()?;
        if self.latest_request_generation.load(Ordering::Acquire) != worker_generation {
            return None;
        }
        state.generation = worker_generation;
        state.slot = Some(PreparedDecisionSlot {
            identity: identity.clone(),
            request_generation: worker_generation,
            material_generation,
            full: FullSlotState::Pending,
            exact: ExactSlotState::Absent,
        });
        state.desired = None;
        wake.notify_all();
        Some(worker_generation)
    }

    fn finish_request(
        &self,
        worker_generation: u64,
        material_generation: u64,
        work: SpaceAutocorrectWork,
        exact_decision: Option<ActiveCompositionAutocorrectDecision>,
        exact_certificate: Option<lay::exact_layout_authority::ExactLayoutContourCertificate>,
        exact_us: u128,
    ) -> bool {
        if self.latest_request_generation.load(Ordering::Acquire) != worker_generation {
            return false;
        }
        let (lock, wake) = &*self.state;
        let mut state = match lock.try_lock() {
            Ok(state) => state,
            Err(TryLockError::WouldBlock) | Err(TryLockError::Poisoned(_)) => return false,
        };
        if self.latest_request_generation.load(Ordering::Acquire) != worker_generation
            || state.generation != worker_generation
            || !state.slot.as_ref().is_some_and(|slot| {
                slot.identity == work.identity
                    && slot.request_generation == worker_generation
                    && slot.material_generation == material_generation
                    && matches!(slot.full, FullSlotState::Pending)
            })
        {
            return false;
        }
        let exact_state = exact_decision.map_or(ExactSlotState::Absent, |decision| {
            ExactSlotState::Prepared(PreparedCorrectionLease {
                identity: work.identity.clone(),
                decision,
                decision_us: exact_us,
                worker_generation,
                material_generation,
                kind: PreparedLeaseKind::ExactLayout,
                exact_certificate: exact_certificate.clone(),
            })
        });
        state
            .slot
            .as_mut()
            .expect("current request slot checked above")
            .exact = exact_state;
        state.desired = Some(DesiredWork {
            worker_generation,
            material_generation,
            work,
            exact_certificate,
        });
        wake.notify_one();
        true
    }

    fn take(&self, identity: &InputFrameIdentity) -> SpaceAutocorrectLookupReceipt {
        let started = Instant::now();
        let (lock, wake) = &*self.state;
        let Ok(mut state) = lock.lock() else {
            return lookup_receipt(SpaceAutocorrectLookup::NotReady, started, 0);
        };
        loop {
            let Some(slot) = state.slot.as_ref() else {
                return lookup_receipt(SpaceAutocorrectLookup::NotReady, started, state.generation);
            };
            if slot.identity != *identity {
                return lookup_receipt(
                    SpaceAutocorrectLookup::Stale,
                    started,
                    slot.request_generation,
                );
            }
            if slot.request_generation != state.generation
                || slot.request_generation != self.latest_request_generation.load(Ordering::Acquire)
                || slot.material_generation != lay::nanda_wave::candidate_material_generation()
            {
                let generation = slot.request_generation;
                retire_slot(&mut state, &self.latest_request_generation, generation);
                wake.notify_all();
                return lookup_receipt(SpaceAutocorrectLookup::Stale, started, generation);
            }

            if matches!(slot.full, FullSlotState::Terminal(_)) {
                let generation = state.generation;
                let slot = state.slot.take().expect("terminal slot checked above");
                retire_generation(&mut state, &self.latest_request_generation, generation);
                wake.notify_all();
                let FullSlotState::Terminal(outcome) = slot.full else {
                    unreachable!("terminal slot checked above")
                };
                return match outcome {
                    PreparedFullOutcome::Apply(lease) => {
                        lookup_receipt(SpaceAutocorrectLookup::Ready(lease), started, generation)
                    }
                    PreparedFullOutcome::NoApply { stage, decision_us } => {
                        trace::record(format!(
                            r#"{{"kind":"ibus_space_full_terminal","status":"no_apply","stage":"{}","decision_us":{decision_us}}}"#,
                            no_apply_stage_name(stage),
                        ));
                        lookup_receipt(SpaceAutocorrectLookup::NoApply(stage), started, generation)
                    }
                };
            }

            if matches!(slot.exact, ExactSlotState::Prepared(_)) {
                let generation = state.generation;
                let slot = state.slot.take().expect("exact slot checked above");
                retire_generation(&mut state, &self.latest_request_generation, generation);
                wake.notify_all();
                let ExactSlotState::Prepared(lease) = slot.exact else {
                    unreachable!("exact slot checked above")
                };
                return lookup_receipt(SpaceAutocorrectLookup::Ready(lease), started, generation);
            }

            let remaining = SPACE_FULL_WAIT_BUDGET.saturating_sub(started.elapsed());
            if remaining.is_zero() {
                let generation = state.generation;
                retire_slot(&mut state, &self.latest_request_generation, generation);
                wake.notify_all();
                return lookup_receipt(SpaceAutocorrectLookup::NotReady, started, generation);
            }
            let Ok((next, timeout)) = wake.wait_timeout(state, remaining) else {
                return lookup_receipt(SpaceAutocorrectLookup::NotReady, started, 0);
            };
            state = next;
            if timeout.timed_out()
                && state
                    .slot
                    .as_ref()
                    .is_some_and(|slot| matches!(slot.full, FullSlotState::Pending))
            {
                let generation = state.generation;
                retire_slot(&mut state, &self.latest_request_generation, generation);
                wake.notify_all();
                return lookup_receipt(SpaceAutocorrectLookup::NotReady, started, generation);
            }
        }
    }

    fn invalidate(&self, identity: &InputFrameIdentity) {
        let (lock, wake) = &*self.state;
        let Ok(mut state) = lock.lock() else {
            return;
        };
        if state.slot.as_ref().map(|slot| &slot.identity) != Some(identity) {
            return;
        }
        let generation = state.generation;
        retire_slot(&mut state, &self.latest_request_generation, generation);
        wake.notify_all();
    }

    fn invalidate_path(&self, path: &str) {
        let (lock, wake) = &*self.state;
        let Ok(mut state) = lock.lock() else {
            return;
        };
        if !state
            .slot
            .as_ref()
            .is_some_and(|slot| slot.identity.path == path)
        {
            return;
        }
        let generation = state.generation;
        retire_slot(&mut state, &self.latest_request_generation, generation);
        wake.notify_all();
    }
}

impl Drop for Worker {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Release);
        self.state.1.notify_all();
    }
}

impl WorkerPool {
    fn lane(&mut self, path: &str) -> Arc<Worker> {
        self.clock = next_generation(self.clock);
        if let Some(lane) = self.lanes.get_mut(path) {
            lane.last_used = self.clock;
            return Arc::clone(&lane.worker);
        }
        if self.lanes.len() >= MAX_PREFETCH_PATH_LANES {
            if let Some(evicted) = self
                .lanes
                .iter()
                .min_by_key(|(_, lane)| lane.last_used)
                .map(|(path, _)| path.clone())
            {
                self.lanes.remove(&evicted);
            }
        }
        let worker = Arc::new(Worker::start());
        self.lanes.insert(
            path.to_string(),
            WorkerLane {
                worker: Arc::clone(&worker),
                last_used: self.clock,
            },
        );
        worker
    }

    fn existing_lane(&mut self, path: &str) -> Option<Arc<Worker>> {
        self.clock = next_generation(self.clock);
        let lane = self.lanes.get_mut(path)?;
        lane.last_used = self.clock;
        Some(Arc::clone(&lane.worker))
    }

    fn remove_lane(&mut self, path: &str) -> Option<Arc<Worker>> {
        self.lanes.remove(path).map(|lane| lane.worker)
    }
}

fn worker_pool() -> &'static Mutex<WorkerPool> {
    WORKERS.get_or_init(|| Mutex::new(WorkerPool::default()))
}

fn worker_for_schedule(path: &str) -> Option<Arc<Worker>> {
    worker_pool().lock().ok().map(|mut pool| pool.lane(path))
}

fn existing_worker(path: &str) -> Option<Arc<Worker>> {
    worker_pool()
        .lock()
        .ok()
        .and_then(|mut pool| pool.existing_lane(path))
}

fn prepare_inline_exact(
    work: &SpaceAutocorrectWork,
) -> Option<lay::ime_correction::PreparedExactLayoutAutocorrect> {
    if !work.identity.config_matches(&work.config) {
        return None;
    }
    let boundary_text = work.identity.boundary_text()?;
    let frame = work.identity.exact_layout_frame();
    let lexical_authority_frame = work.identity.lexical_authority_frame();
    prepare_exact_layout_active_composition_autocorrect_observed(
        ActiveCompositionAutocorrectRequest {
            text: &boundary_text,
            committed_tail: &work.identity.committed_tail,
            config: &work.config,
            lexical_authority_frame: Some(&lexical_authority_frame),
            active_layout_is_ru: Some(work.identity.active_layout_is_ru),
        },
        &frame,
    )
    .prepared
}

fn run_worker(
    shared: Arc<(Mutex<WorkerState>, Condvar)>,
    latest_request_generation: Arc<AtomicU64>,
    stop: Arc<AtomicBool>,
) {
    loop {
        let desired = {
            let (lock, wake) = &*shared;
            let Ok(mut state) = lock.lock() else {
                return;
            };
            while state.desired.is_none() && !stop.load(Ordering::Acquire) {
                let Ok(next) = wake.wait(state) else {
                    return;
                };
                state = next;
            }
            if stop.load(Ordering::Acquire) {
                return;
            }
            state.desired.take().expect("desired work checked above")
        };

        let started = Instant::now();
        let (outcome, telemetry) = evaluate_full(&desired, started);
        let trace_identity = desired.work.identity.clone();
        let current_material_generation = lay::nanda_wave::candidate_material_generation();

        let (lock, wake) = &*shared;
        let Ok(mut state) = lock.lock() else {
            return;
        };
        let published = full_outcome_may_publish(
            &state,
            &desired,
            current_material_generation,
            latest_request_generation.load(Ordering::Acquire),
        );
        if published {
            if let Some(slot) = state.slot.as_mut() {
                slot.full = FullSlotState::Terminal(outcome);
            }
            wake.notify_all();
        }
        drop(state);
        trace::record_correction_projection_timing(
            if published { "prepared" } else { "superseded" },
            desired.worker_generation,
            &trace_identity,
            telemetry,
        );
    }
}

fn evaluate_full(
    desired: &DesiredWork,
    started: Instant,
) -> (PreparedFullOutcome, ActiveCompositionAutocorrectTelemetry) {
    if !desired.work.identity.config_matches(&desired.work.config) {
        return (
            PreparedFullOutcome::NoApply {
                stage: PreparedNoApplyStage::Infrastructure,
                decision_us: started.elapsed().as_micros(),
            },
            ActiveCompositionAutocorrectTelemetry::default(),
        );
    }
    let Some(boundary_text) = desired.work.identity.boundary_text() else {
        return (
            PreparedFullOutcome::NoApply {
                stage: PreparedNoApplyStage::Infrastructure,
                decision_us: started.elapsed().as_micros(),
            },
            ActiveCompositionAutocorrectTelemetry::default(),
        );
    };
    let lexical_authority_frame = desired.work.identity.lexical_authority_frame();
    let request = ActiveCompositionAutocorrectRequest {
        text: &boundary_text,
        committed_tail: &desired.work.identity.committed_tail,
        config: &desired.work.config,
        lexical_authority_frame: Some(&lexical_authority_frame),
        active_layout_is_ru: Some(desired.work.identity.active_layout_is_ru),
    };
    let observed = match desired.exact_certificate.as_ref() {
        Some(certificate) => {
            decide_active_composition_autocorrect_observed_with_exact(request, certificate)
        }
        None => decide_active_composition_autocorrect_observed(request),
    };
    let decision_us = started.elapsed().as_micros();
    let telemetry = observed.telemetry;
    let outcome = match observed.decision {
        Some(decision) => PreparedFullOutcome::Apply(PreparedCorrectionLease {
            identity: desired.work.identity.clone(),
            decision,
            decision_us,
            worker_generation: desired.worker_generation,
            material_generation: desired.material_generation,
            kind: PreparedLeaseKind::Full,
            exact_certificate: desired.exact_certificate.clone(),
        }),
        None => PreparedFullOutcome::NoApply {
            stage: match observed.no_apply_stage {
                Some(AutocorrectNoApplyStage::Rank) => PreparedNoApplyStage::Rank,
                Some(AutocorrectNoApplyStage::Verifier) => PreparedNoApplyStage::Verifier,
                None => PreparedNoApplyStage::Infrastructure,
            },
            decision_us,
        },
    };
    (outcome, telemetry)
}

fn full_outcome_may_publish(
    state: &WorkerState,
    desired: &DesiredWork,
    current_material_generation: u64,
    latest_request_generation: u64,
) -> bool {
    latest_request_generation == desired.worker_generation
        && state.generation == desired.worker_generation
        && desired.material_generation == current_material_generation
        && state.slot.as_ref().is_some_and(|slot| {
            slot.identity == desired.work.identity
                && slot.request_generation == desired.worker_generation
                && slot.material_generation == desired.material_generation
                && matches!(slot.full, FullSlotState::Pending)
        })
}

fn next_generation(generation: u64) -> u64 {
    generation.wrapping_add(1).max(1)
}

fn reserve_generation(latest_request_generation: &AtomicU64) -> u64 {
    let previous = latest_request_generation
        .fetch_update(Ordering::AcqRel, Ordering::Acquire, |generation| {
            Some(next_generation(generation))
        })
        .expect("generation update cannot fail");
    next_generation(previous)
}

fn retire_slot(state: &mut WorkerState, latest_request_generation: &AtomicU64, generation: u64) {
    state.slot = None;
    retire_generation(state, latest_request_generation, generation);
}

fn retire_generation(
    state: &mut WorkerState,
    latest_request_generation: &AtomicU64,
    generation: u64,
) {
    let _ = latest_request_generation.compare_exchange(
        generation,
        next_generation(generation),
        Ordering::AcqRel,
        Ordering::Acquire,
    );
    if state
        .desired
        .as_ref()
        .is_some_and(|desired| desired.worker_generation == generation)
    {
        state.desired = None;
    }
}

fn no_apply_stage_name(stage: PreparedNoApplyStage) -> &'static str {
    match stage {
        PreparedNoApplyStage::Rank => "rank",
        PreparedNoApplyStage::Verifier => "verifier",
        PreparedNoApplyStage::Infrastructure => "infrastructure",
    }
}

pub(crate) fn initialize() {
    let _ = worker_pool();
}

pub(crate) fn schedule(work: SpaceAutocorrectWork) {
    let Some(worker) = worker_for_schedule(&work.identity.path) else {
        return;
    };
    worker.schedule(work);
}

pub(crate) fn take(identity: &InputFrameIdentity) -> SpaceAutocorrectLookupReceipt {
    let Some(worker) = existing_worker(&identity.path) else {
        return SpaceAutocorrectLookupReceipt {
            lookup: SpaceAutocorrectLookup::NotReady,
            wait_us: 0,
            worker_generation: 0,
        };
    };
    worker.take(identity)
}

pub(crate) fn invalidate(identity: &InputFrameIdentity) {
    if let Some(worker) = existing_worker(&identity.path) {
        worker.invalidate(identity);
    }
}

pub(crate) fn invalidate_path(path: &str) {
    if let Ok(mut pool) = worker_pool().lock() {
        if let Some(worker) = pool.remove_lane(path) {
            worker.invalidate_path(path);
        }
    }
}

fn lookup_receipt(
    lookup: SpaceAutocorrectLookup,
    started: Instant,
    worker_generation: u64,
) -> SpaceAutocorrectLookupReceipt {
    SpaceAutocorrectLookupReceipt {
        lookup,
        wait_us: started.elapsed().as_micros(),
        worker_generation,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn identity(path: &str, epoch: u64, tail: &str) -> InputFrameIdentity {
        identity_with_focus(path, "focus-a", epoch, tail)
    }

    fn identity_with_focus(
        path: &str,
        focus_receipt: &str,
        epoch: u64,
        tail: &str,
    ) -> InputFrameIdentity {
        let config = LayConfig::default();
        identity_with_config(path, focus_receipt, epoch, tail, &config)
    }

    fn identity_with_config(
        path: &str,
        focus_receipt: &str,
        epoch: u64,
        tail: &str,
        config: &LayConfig,
    ) -> InputFrameIdentity {
        let token = tail.split_whitespace().last().unwrap_or_default();
        let context_prefix = tail.strip_suffix(token).unwrap_or_default();
        InputFrameIdentity::new(
            path.to_string(),
            Some(focus_receipt.to_string()),
            epoch,
            tail.to_string(),
            context_prefix.to_string(),
            token.to_string(),
            true,
            false,
            config,
        )
    }

    fn exact_config() -> LayConfig {
        LayConfig {
            auto_replace: true,
            ..LayConfig::default()
        }
    }

    fn worker_with_state(state: WorkerState) -> Worker {
        let generation = state.generation;
        Worker {
            state: Arc::new((Mutex::new(state), Condvar::new())),
            latest_request_generation: Arc::new(AtomicU64::new(generation)),
            stop: Arc::new(AtomicBool::new(false)),
        }
    }

    fn exact_lease(
        identity: &InputFrameIdentity,
        config: &LayConfig,
        worker_generation: u64,
        material_generation: u64,
    ) -> PreparedCorrectionLease {
        let prepared = prepare_inline_exact(&SpaceAutocorrectWork {
            identity: identity.clone(),
            config: config.clone(),
        })
        .expect("closed exact preparation");
        PreparedCorrectionLease {
            identity: identity.clone(),
            decision: prepared.decision.expect("closed exact decision"),
            decision_us: 1,
            worker_generation,
            material_generation,
            kind: PreparedLeaseKind::ExactLayout,
            exact_certificate: Some(prepared.certificate),
        }
    }

    #[test]
    fn current_full_no_apply_is_terminal_and_consumed_once() {
        let expected = identity("/engine/a", 7, "token");
        let state = WorkerState {
            generation: 11,
            slot: Some(PreparedDecisionSlot {
                identity: expected.clone(),
                request_generation: 11,
                material_generation: lay::nanda_wave::candidate_material_generation(),
                full: FullSlotState::Terminal(PreparedFullOutcome::NoApply {
                    stage: PreparedNoApplyStage::Rank,
                    decision_us: 9,
                }),
                exact: ExactSlotState::Absent,
            }),
            desired: None,
        };
        let worker = worker_with_state(state);

        assert!(matches!(
            worker.take(&expected).lookup,
            SpaceAutocorrectLookup::NoApply(PreparedNoApplyStage::Rank)
        ));
        assert!(matches!(
            worker.take(&expected).lookup,
            SpaceAutocorrectLookup::NotReady
        ));
    }

    #[test]
    fn current_full_no_apply_suppresses_prepared_exact_lease() {
        lay::exact_layout_authority::warm_up_exact_layout_authority_for_ibus()
            .expect("warm exact-layout authority");
        let config = exact_config();
        let expected = identity_with_config("/engine/a", "focus-a", 7, "ghbdtn", &config);
        let material_generation = lay::nanda_wave::candidate_material_generation();
        let state = WorkerState {
            generation: 11,
            slot: Some(PreparedDecisionSlot {
                identity: expected.clone(),
                request_generation: 11,
                material_generation,
                full: FullSlotState::Terminal(PreparedFullOutcome::NoApply {
                    stage: PreparedNoApplyStage::Verifier,
                    decision_us: 9,
                }),
                exact: ExactSlotState::Prepared(exact_lease(
                    &expected,
                    &config,
                    11,
                    material_generation,
                )),
            }),
            desired: None,
        };
        let worker = worker_with_state(state);

        assert!(matches!(
            worker.take(&expected).lookup,
            SpaceAutocorrectLookup::NoApply(PreparedNoApplyStage::Verifier)
        ));
        assert!(matches!(
            worker.take(&expected).lookup,
            SpaceAutocorrectLookup::NotReady
        ));
    }

    #[test]
    fn exact_lease_is_consumed_at_most_once() {
        lay::exact_layout_authority::warm_up_exact_layout_authority_for_ibus()
            .expect("warm exact-layout authority");
        let config = exact_config();
        let expected = identity_with_config("/engine/a", "focus-a", 7, "ghbdtn", &config);
        let material_generation = lay::nanda_wave::candidate_material_generation();
        let state = WorkerState {
            generation: 11,
            slot: Some(PreparedDecisionSlot {
                identity: expected.clone(),
                request_generation: 11,
                material_generation,
                full: FullSlotState::Pending,
                exact: ExactSlotState::Prepared(exact_lease(
                    &expected,
                    &config,
                    11,
                    material_generation,
                )),
            }),
            desired: None,
        };
        let worker = worker_with_state(state);

        let first = worker.take(&expected);
        let SpaceAutocorrectLookup::Ready(lease) = first.lookup else {
            panic!("expected exact lease")
        };
        assert_eq!(lease.kind, PreparedLeaseKind::ExactLayout);
        assert!(matches!(
            worker.take(&expected).lookup,
            SpaceAutocorrectLookup::NotReady
        ));
    }

    #[test]
    fn stale_focus_take_never_consumes_newer_focus_slot() {
        let stale = identity_with_focus("/engine/a", "focus-a", 7, "stale");
        let current = identity_with_focus("/engine/a", "focus-b", 8, "current");
        let state = WorkerState {
            generation: 12,
            slot: Some(PreparedDecisionSlot {
                identity: current.clone(),
                request_generation: 12,
                material_generation: lay::nanda_wave::candidate_material_generation(),
                full: FullSlotState::Terminal(PreparedFullOutcome::NoApply {
                    stage: PreparedNoApplyStage::Rank,
                    decision_us: 9,
                }),
                exact: ExactSlotState::Absent,
            }),
            desired: None,
        };
        let worker = worker_with_state(state);

        assert!(matches!(
            worker.take(&stale).lookup,
            SpaceAutocorrectLookup::Stale
        ));
        assert!(matches!(
            worker.take(&current).lookup,
            SpaceAutocorrectLookup::NoApply(PreparedNoApplyStage::Rank)
        ));
    }

    #[test]
    fn full_publication_requires_current_generation_and_material() {
        let expected = identity("/engine/a", 9, "token");
        let config = LayConfig::default();
        let material_generation = lay::nanda_wave::candidate_material_generation();
        let desired = DesiredWork {
            worker_generation: 14,
            material_generation,
            work: SpaceAutocorrectWork {
                identity: expected.clone(),
                config,
            },
            exact_certificate: None,
        };
        let state = WorkerState {
            generation: 14,
            slot: Some(PreparedDecisionSlot {
                identity: expected,
                request_generation: 14,
                material_generation,
                full: FullSlotState::Pending,
                exact: ExactSlotState::Absent,
            }),
            desired: None,
        };

        assert!(full_outcome_may_publish(
            &state,
            &desired,
            material_generation,
            14,
        ));
        assert!(!full_outcome_may_publish(
            &state,
            &desired,
            next_generation(material_generation),
            14,
        ));
        assert!(!full_outcome_may_publish(
            &state,
            &desired,
            material_generation,
            15,
        ));
    }

    #[test]
    fn material_generation_change_retires_only_the_matching_slot() {
        let expected = identity("/engine/a", 9, "token");
        let current_material_generation = lay::nanda_wave::candidate_material_generation();
        let state = WorkerState {
            generation: 14,
            slot: Some(PreparedDecisionSlot {
                identity: expected.clone(),
                request_generation: 14,
                material_generation: next_generation(current_material_generation),
                full: FullSlotState::Pending,
                exact: ExactSlotState::Absent,
            }),
            desired: None,
        };
        let worker = worker_with_state(state);

        assert!(matches!(
            worker.take(&expected).lookup,
            SpaceAutocorrectLookup::Stale
        ));
        assert!(matches!(
            worker.take(&expected).lookup,
            SpaceAutocorrectLookup::NotReady
        ));
    }

    #[test]
    fn late_older_preparation_cannot_replace_newer_slot() {
        let older = identity_with_focus("/engine/a", "focus-a", 10, "older");
        let newer = identity_with_focus("/engine/b", "focus-b", 11, "newer");
        let material_generation = lay::nanda_wave::candidate_material_generation();
        let config = LayConfig::default();
        let worker = worker_with_state(WorkerState::default());

        let older_generation = worker
            .begin_request(&older, material_generation)
            .expect("older request must register");
        let newer_generation = worker
            .begin_request(&newer, material_generation)
            .expect("newer request must register");
        assert!(newer_generation > older_generation);
        assert!(worker.finish_request(
            newer_generation,
            material_generation,
            SpaceAutocorrectWork {
                identity: newer.clone(),
                config: config.clone(),
            },
            None,
            None,
            1,
        ));
        assert!(!worker.finish_request(
            older_generation,
            material_generation,
            SpaceAutocorrectWork {
                identity: older,
                config,
            },
            None,
            None,
            2,
        ));

        let state = worker.state.0.lock().expect("worker state poisoned");
        let slot = state.slot.as_ref().expect("newer slot must survive");
        assert_eq!(slot.identity, newer);
        assert_eq!(slot.request_generation, newer_generation);
        assert_eq!(
            state
                .desired
                .as_ref()
                .map(|desired| desired.worker_generation),
            Some(newer_generation)
        );
    }

    #[test]
    fn space_take_path_never_computes_exact_or_full_decisions() {
        let source = include_str!("space_autocorrect_prefetch.rs");
        let take_body = source
            .split("fn take(&self")
            .nth(1)
            .expect("take method")
            .split("fn invalidate(&self")
            .next()
            .expect("bounded take method");

        assert!(!take_body.contains("prepare_inline_exact"));
        assert!(!take_body.contains("prepare_exact_layout"));
        assert!(!take_body.contains("evaluate_full"));
        assert!(!take_body.contains("decide_active_composition"));
    }

    #[test]
    fn worker_pool_isolates_paths_and_evicts_to_the_fixed_bound() {
        let mut pool = WorkerPool::default();
        let first = pool.lane("/engine/0");
        let second = pool.lane("/engine/1");
        assert!(!Arc::ptr_eq(&first, &second));
        drop(first);
        let evicted = Arc::downgrade(&pool.lanes["/engine/0"].worker);

        for index in 2..=MAX_PREFETCH_PATH_LANES {
            pool.lane(&format!("/engine/{index}"));
        }

        assert_eq!(pool.lanes.len(), MAX_PREFETCH_PATH_LANES);
        assert!(!pool.lanes.contains_key("/engine/0"));
        assert!(evicted.upgrade().is_none());
    }
}

#[cfg(test)]
#[path = "space_autocorrect_prefetch/proof.rs"]
pub(crate) mod proof;

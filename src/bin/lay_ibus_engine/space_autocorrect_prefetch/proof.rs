use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Barrier, Mutex};
use std::time::{Duration, Instant};

use super::*;
use crate::engine::{InputFrameIdentity, LayIbusEngine};
use crate::output::{AtomicEffectBuilder, AtomicProposal, EngineOutput, PROPOSAL_FRAME_READY};
use lay::config::LayConfig;

fn exact_config() -> LayConfig {
    LayConfig {
        auto_replace: true,
        auto_switch_layout: true,
        text_backend: "ime".to_string(),
        ..LayConfig::default()
    }
}

fn identity(
    path: &str,
    focus: &str,
    epoch: u64,
    tail: &str,
    config: &LayConfig,
) -> InputFrameIdentity {
    let token = tail.split_whitespace().last().unwrap_or_default();
    let context = tail.strip_suffix(token).unwrap_or_default();
    InputFrameIdentity::new(
        path.to_string(),
        Some(focus.to_string()),
        epoch,
        tail.to_string(),
        context.to_string(),
        token.to_string(),
        true,
        false,
        config,
    )
}

fn worker_with_terminal(identity: InputFrameIdentity, generation: u64) -> Worker {
    let material_generation = lay::nanda_wave::candidate_material_generation();
    Worker {
        state: Arc::new((
            Mutex::new(WorkerState {
                generation,
                slot: Some(PreparedDecisionSlot {
                    identity,
                    request_generation: generation,
                    material_generation,
                    full: FullSlotState::Terminal(PreparedFullOutcome::NoApply {
                        stage: PreparedNoApplyStage::Rank,
                        decision_us: 1,
                    }),
                    exact: ExactSlotState::Absent,
                }),
                desired: None,
            }),
            Condvar::new(),
        )),
        latest_request_generation: Arc::new(AtomicU64::new(generation)),
    }
}

fn managed_engine(path: &str, token: &str) -> LayIbusEngine {
    let mut engine = LayIbusEngine::new(
        path.to_string(),
        Arc::new(Mutex::new(Default::default())),
        false,
        true,
        exact_config(),
    );
    assert!(engine.bind_focus_path());
    engine.surrounding_text_supported = true;
    for character in token.chars() {
        engine.push_tail_char(character);
    }
    engine
}

fn exact_lease(
    identity: &InputFrameIdentity,
    config: &LayConfig,
    generation: u64,
) -> PreparedCorrectionLease {
    let material_generation = lay::nanda_wave::candidate_material_generation();
    let prepared = prepare_inline_exact(&SpaceAutocorrectWork {
        identity: identity.clone(),
        config: config.clone(),
    })
    .expect("closed exact preparation");
    PreparedCorrectionLease {
        identity: identity.clone(),
        decision: prepared.decision.expect("closed exact decision"),
        decision_us: 1,
        worker_generation: generation,
        material_generation,
        kind: PreparedLeaseKind::ExactLayout,
        exact_certificate: Some(prepared.certificate),
    }
}

fn committed_texts(proposal: &AtomicProposal) -> Vec<String> {
    proposal
        .1
        .iter()
        .filter(|(tag, _)| *tag == 1)
        .map(|(_, value)| String::try_from(value.clone()).expect("CommitText string"))
        .collect()
}

fn render_lookup(
    engine: &mut LayIbusEngine,
    identity: &InputFrameIdentity,
    lookup: SpaceAutocorrectLookup,
) -> AtomicProposal {
    let mut builder = AtomicEffectBuilder::default();
    let handled = {
        let mut output = EngineOutput::atomic(&mut builder);
        let corrected = zbus::block_on(engine.autocorrect_committed_token_on_space(
            &mut output,
            identity,
            SpaceAutocorrectLookupReceipt {
                lookup,
                wait_us: 1,
                worker_generation: 23,
            },
        ))
        .expect("Space correction route");
        if !corrected {
            zbus::block_on(engine.commit_managed_passthrough_char(&mut output, ' '))
                .expect("native managed Space");
        }
        true
    };
    builder.finish(handled)
}

#[test]
fn v27_race_fault_and_space_effect_matrix() {
    lay::exact_layout_authority::warm_up_exact_layout_authority_for_ibus()
        .expect("warm exact-layout authority");
    let config = exact_config();
    let current = identity("/engine/a", "focus-a", 17, "context ghbdtn", &config);

    let mut faults = Vec::new();
    macro_rules! fault {
        ($field:ident, $value:expr) => {{
            let mut changed = current.clone();
            changed.$field = $value;
            faults.push(changed);
        }};
    }
    fault!(path, "/engine/b".to_string());
    fault!(focus_receipt, Some("focus-b".to_string()));
    fault!(tail_epoch, 18);
    fault!(committed_tail, "context ghbdto".to_string());
    fault!(context_prefix, "other ".to_string());
    fault!(observed_token, "ghbdto".to_string());
    fault!(active_composition, false);
    fault!(active_layout_is_ru, true);
    fault!(
        factory_engine_profile,
        lay::exact_layout_authority::FactoryEngineProfile::Unknown
    );
    fault!(exact_authority_snapshot, None);
    fault!(output_capability_fingerprint, 1);
    fault!(frame_fingerprint, current.frame_fingerprint ^ 1);
    let mut changed_config = config.clone();
    changed_config.auto_replace = false;
    faults.push(identity(
        "/engine/a",
        "focus-a",
        17,
        "context ghbdtn",
        &changed_config,
    ));

    for stale in &faults {
        let worker = worker_with_terminal(current.clone(), 31);
        assert!(matches!(
            worker.take(stale).lookup,
            SpaceAutocorrectLookup::Stale
        ));
        assert!(matches!(
            worker.take(&current).lookup,
            SpaceAutocorrectLookup::NoApply(PreparedNoApplyStage::Rank)
        ));
    }

    let worker = Arc::new(worker_with_terminal(current.clone(), 41));
    let lock = worker.state.0.lock().expect("test lock");
    let blocked = Arc::clone(&worker);
    let next = current.clone();
    let join = std::thread::spawn(move || {
        blocked.begin_request(&next, lay::nanda_wave::candidate_material_generation())
    });
    assert!(join.join().expect("nonblocking registration").is_none());
    drop(lock);
    assert!(matches!(
        worker.take(&current).lookup,
        SpaceAutocorrectLookup::Stale
    ));

    let poisoned = Arc::new(worker_with_terminal(current.clone(), 51));
    let poison_state = Arc::clone(&poisoned.state);
    assert!(std::thread::spawn(move || {
        let _guard = poison_state.0.lock().expect("poison lock");
        panic!("intentional V27 lock poison");
    })
    .join()
    .is_err());
    let poison_started = Instant::now();
    assert!(matches!(
        poisoned.take(&current).lookup,
        SpaceAutocorrectLookup::NotReady
    ));
    assert!(poison_started.elapsed() < Duration::from_millis(10));

    let publication = worker_with_terminal(current.clone(), 61);
    publication
        .latest_request_generation
        .store(0, Ordering::Release);
    let generation = publication
        .begin_request(&current, lay::nanda_wave::candidate_material_generation())
        .expect("new request");
    {
        let state = publication.state.0.lock().expect("publication state");
        let slot = state.slot.as_ref().expect("registered slot");
        assert!(matches!(slot.full, FullSlotState::Pending));
        assert!(matches!(slot.exact, ExactSlotState::Absent));
        assert!(state.desired.is_none());
    }
    let prepared = prepare_inline_exact(&SpaceAutocorrectWork {
        identity: current.clone(),
        config: config.clone(),
    })
    .expect("exact publication");
    assert!(publication.finish_request(
        generation,
        lay::nanda_wave::candidate_material_generation(),
        SpaceAutocorrectWork {
            identity: current.clone(),
            config: config.clone(),
        },
        prepared.decision,
        Some(prepared.certificate),
        1,
    ));
    {
        let state = publication.state.0.lock().expect("published state");
        let slot = state.slot.as_ref().expect("complete slot");
        assert!(matches!(slot.exact, ExactSlotState::Prepared(_)));
        assert!(state.desired.is_some());
    }

    for lookup in [
        SpaceAutocorrectLookup::NoApply(PreparedNoApplyStage::Rank),
        SpaceAutocorrectLookup::NoApply(PreparedNoApplyStage::Verifier),
        SpaceAutocorrectLookup::NoApply(PreparedNoApplyStage::Infrastructure),
        SpaceAutocorrectLookup::NotReady,
        SpaceAutocorrectLookup::Stale,
    ] {
        let mut engine = managed_engine("/engine/fallback", "ghbdtn");
        let frame = engine
            .capture_input_frame_identity()
            .expect("fallback frame");
        let proposal = render_lookup(&mut engine, &frame, lookup);
        assert_eq!(proposal.0, PROPOSAL_FRAME_READY);
        assert_eq!(committed_texts(&proposal), [" "]);
        assert!(engine.tail_buffer.ends_with(' '));
        assert!(!engine.tail_buffer.ends_with("  "));
    }

    let mut engine = managed_engine("/engine/exact", "ghbdtn");
    let frame = engine.capture_input_frame_identity().expect("exact frame");
    let lease = exact_lease(&frame, &engine.config, 71);
    let proposal = render_lookup(&mut engine, &frame, SpaceAutocorrectLookup::Ready(lease));
    assert_eq!(proposal.0, PROPOSAL_FRAME_READY);
    assert_eq!(
        committed_texts(&proposal),
        ["\u{43f}\u{440}\u{438}\u{432}\u{435}\u{442} "]
    );
    assert_eq!(
        engine.tail_buffer,
        "\u{43f}\u{440}\u{438}\u{432}\u{435}\u{442} "
    );

    println!(
        "LAY_V27_SYSTEM_MATRIX identity_faults={} lock_contention=PASS lock_poison=PASS complete_publication=PASS fallback_space_cases=5 exact_space=PASS",
        faults.len()
    );
}

fn percentile(samples: &mut [u128], numerator: usize, denominator: usize) -> u128 {
    samples.sort_unstable();
    let index = samples
        .len()
        .saturating_mul(numerator)
        .div_ceil(denominator)
        .saturating_sub(1)
        .min(samples.len().saturating_sub(1));
    samples[index]
}

fn pending_worker(identity: InputFrameIdentity, generation: u64) -> Worker {
    let material_generation = lay::nanda_wave::candidate_material_generation();
    Worker {
        state: Arc::new((
            Mutex::new(WorkerState {
                generation,
                slot: Some(PreparedDecisionSlot {
                    identity,
                    request_generation: generation,
                    material_generation,
                    full: FullSlotState::Pending,
                    exact: ExactSlotState::Absent,
                }),
                desired: None,
            }),
            Condvar::new(),
        )),
        latest_request_generation: Arc::new(AtomicU64::new(generation)),
    }
}

#[test]
fn v27_component_latency_denominators() {
    lay::exact_layout_authority::warm_up_exact_layout_authority_for_ibus()
        .expect("warm exact-layout authority");
    let config = exact_config();
    let miss = identity("/latency/miss", "focus", 1, "hello", &config);
    let hit = identity("/latency/hit", "focus", 1, "ghbdtn", &config);
    let stale = DesiredWork {
        worker_generation: 1,
        material_generation: lay::nanda_wave::candidate_material_generation(),
        work: SpaceAutocorrectWork {
            identity: identity("/latency/stale", "focus", 1, "raobae", &config),
            config: config.clone(),
        },
        exact_certificate: None,
    };
    let _ = evaluate_full(&stale, Instant::now());

    let running = Arc::new(AtomicBool::new(true));
    let ready = Arc::new(Barrier::new(2));
    let worker_running = Arc::clone(&running);
    let worker_ready = Arc::clone(&ready);
    let busy = std::thread::spawn(move || {
        worker_ready.wait();
        while worker_running.load(Ordering::Acquire) {
            std::hint::black_box(evaluate_full(&stale, Instant::now()));
        }
    });
    ready.wait();

    let samples = std::env::var("LAY_V27_LATENCY_SAMPLES")
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(2_048)
        .max(256);
    let schedule_worker = Worker {
        state: Arc::new((Mutex::new(WorkerState::default()), Condvar::new())),
        latest_request_generation: Arc::new(AtomicU64::new(0)),
    };
    for frame in [&miss, &hit] {
        for _ in 0..64 {
            schedule_worker.schedule(SpaceAutocorrectWork {
                identity: frame.clone(),
                config: config.clone(),
            });
        }
    }

    let mut miss_us = Vec::with_capacity(samples);
    let mut hit_us = Vec::with_capacity(samples);
    for _ in 0..samples {
        let started = Instant::now();
        schedule_worker.schedule(SpaceAutocorrectWork {
            identity: miss.clone(),
            config: config.clone(),
        });
        miss_us.push(started.elapsed().as_micros());

        let started = Instant::now();
        schedule_worker.schedule(SpaceAutocorrectWork {
            identity: hit.clone(),
            config: config.clone(),
        });
        hit_us.push(started.elapsed().as_micros());
    }

    let mut lookup_us = Vec::with_capacity(samples);
    let lookup_worker = Worker {
        state: Arc::new((Mutex::new(WorkerState::default()), Condvar::new())),
        latest_request_generation: Arc::new(AtomicU64::new(0)),
    };
    for _ in 0..samples {
        lookup_worker.schedule(SpaceAutocorrectWork {
            identity: hit.clone(),
            config: config.clone(),
        });
        let receipt = lookup_worker.take(&hit);
        assert!(matches!(receipt.lookup, SpaceAutocorrectLookup::Ready(_)));
        lookup_us.push(receipt.wait_us);
    }

    let wait_samples = (samples / 16).max(64);
    let mut timeout_us = Vec::with_capacity(wait_samples);
    let mut completion_us = Vec::with_capacity(wait_samples);
    for generation in 1..=wait_samples as u64 {
        let worker = pending_worker(miss.clone(), generation);
        let receipt = worker.take(&miss);
        assert!(matches!(receipt.lookup, SpaceAutocorrectLookup::NotReady));
        timeout_us.push(receipt.wait_us);

        let worker = pending_worker(miss.clone(), generation + wait_samples as u64);
        let state = Arc::clone(&worker.state);
        let publisher = std::thread::spawn(move || {
            std::thread::sleep(Duration::from_micros(250));
            let (lock, wake) = &*state;
            let mut state = lock.lock().expect("completion state");
            let slot = state.slot.as_mut().expect("pending slot");
            slot.full = FullSlotState::Terminal(PreparedFullOutcome::NoApply {
                stage: PreparedNoApplyStage::Rank,
                decision_us: 1,
            });
            wake.notify_all();
        });
        let receipt = worker.take(&miss);
        publisher.join().expect("completion publisher");
        assert!(matches!(
            receipt.lookup,
            SpaceAutocorrectLookup::NoApply(PreparedNoApplyStage::Rank)
        ));
        completion_us.push(receipt.wait_us);
    }

    running.store(false, Ordering::Release);
    busy.join().expect("busy stale full worker");

    let miss_p99 = percentile(&mut miss_us, 99, 100);
    let hit_p99 = percentile(&mut hit_us, 99, 100);
    let lookup_p99 = percentile(&mut lookup_us, 99, 100);
    let timeout_p99 = percentile(&mut timeout_us, 99, 100);
    let completion_p99 = percentile(&mut completion_us, 99, 100);
    let full_wait_p99 = timeout_p99.max(completion_p99);
    println!(
        "LAY_V27_COMPONENT_LATENCY printable_miss_n={} printable_miss_p99_us={} printable_hit_n={} printable_hit_p99_us={} space_exact_lookup_n={} space_exact_lookup_p99_us={} space_full_timeout_n={} space_full_timeout_p99_us={} space_full_completion_n={} space_full_completion_p99_us={} space_full_wait_p99_us={} busy_stale_full_worker=true",
        miss_us.len(),
        miss_p99,
        hit_us.len(),
        hit_p99,
        lookup_us.len(),
        lookup_p99,
        timeout_us.len(),
        timeout_p99,
        completion_us.len(),
        completion_p99,
        full_wait_p99,
    );

    assert!(miss_p99 <= 250, "printable exact miss p99 {miss_p99}us");
    assert!(hit_p99 <= 2_000, "printable exact hit p99 {hit_p99}us");
    assert!(lookup_p99 <= 1_000, "Space exact lookup p99 {lookup_p99}us");
    assert!(
        full_wait_p99 <= 4_000,
        "Space full wait p99 {full_wait_p99}us"
    );
}

pub(crate) fn install_exact_lease(identity: &InputFrameIdentity, config: &LayConfig) {
    initialize();
    let worker = WORKER.get().expect("global prefetch worker");
    let generation = reserve_generation(&worker.latest_request_generation);
    let material_generation = lay::nanda_wave::candidate_material_generation();
    let lease = exact_lease(identity, config, generation);
    let (lock, wake) = &*worker.state;
    let mut state = lock.lock().expect("global proof slot");
    state.generation = generation;
    state.slot = Some(PreparedDecisionSlot {
        identity: identity.clone(),
        request_generation: generation,
        material_generation,
        full: FullSlotState::Pending,
        exact: ExactSlotState::Prepared(lease),
    });
    state.desired = None;
    wake.notify_all();
}

use std::sync::{Arc, Mutex};

use super::*;
use crate::engine::{LayIbusEngine, SurroundingTextSnapshot};
use crate::output::{AtomicProposal, PROPOSAL_FRAME_READY, PROPOSAL_NATIVE_UNHANDLED};
use crate::protocol::{KEY_LEFT_SHIFT, KEY_SPACE, RELEASE_MASK};
use crate::space_autocorrect_prefetch;
use lay::config::LayConfig;
use std::time::{Duration, Instant};

fn engine(path: &str) -> LayIbusEngine {
    engine_with_shared(
        path,
        Arc::new(Mutex::new(Default::default())),
        false,
        "ghbdtn",
    )
}

fn engine_with_shared(
    path: &str,
    shared: crate::protocol::Shared,
    layout_is_ru: bool,
    initial_tail: &str,
) -> LayIbusEngine {
    let mut engine = LayIbusEngine::new(
        path.to_string(),
        shared,
        layout_is_ru,
        true,
        LayConfig {
            auto_replace: true,
            auto_switch_layout: true,
            text_backend: "ime".to_string(),
            ..LayConfig::default()
        },
    );
    assert!(engine.bind_focus_path());
    engine.surrounding_text_supported = true;
    engine.atomic_route_active = true;
    for character in initial_tail.chars() {
        engine.push_tail_char(character);
    }
    engine
}

#[test]
fn v27_cross_engine_shift_gesture_restores_exact_source() {
    lay::exact_layout_authority::warm_up_exact_layout_authority_for_ibus()
        .expect("warm exact-layout authority");

    let shared = Arc::new(Mutex::new(Default::default()));
    let mut source = engine_with_shared("/atomic/handoff-us", Arc::clone(&shared), false, "ghbdtn");
    prepare(&source);
    let applied = zbus::block_on(source.process_atomic_key_event(
        KEY_SPACE,
        65,
        0,
        envelope(501),
        capability(3, true),
        (RECEIPT_NONE, 0, Vec::new()),
    ))
    .expect("exact Space proposal");
    assert_eq!(applied.0, PROPOSAL_FRAME_READY);
    source.observe_external_surrounding_text(Some(SurroundingTextSnapshot::new(
        "привет ".to_string(),
        7,
        7,
    )));

    let first_press = zbus::block_on(source.process_atomic_key_event(
        KEY_LEFT_SHIFT,
        42,
        0,
        envelope(502),
        capability(3, true),
        (RECEIPT_SUBMITTED_ATOMIC, 501, vec![9; DIGEST_BYTES]),
    ))
    .expect("settle Space and observe first Shift press");
    assert_eq!(first_press.0, PROPOSAL_NATIVE_UNHANDLED);
    assert!(source.shift_active);
    assert!(shared
        .lock()
        .expect("shared state")
        .shift_gesture_handoff
        .as_ref()
        .is_some_and(|gesture| gesture.source_path == source.path));

    let mut target = engine_with_shared("/atomic/handoff-ru", shared, true, "");
    target.observe_external_surrounding_text(Some(SurroundingTextSnapshot::new(
        "привет ".to_string(),
        7,
        7,
    )));
    assert_eq!(target.tail_buffer, "привет ");

    for (transaction, state) in [(503, RELEASE_MASK), (504, 0)] {
        let proposal = zbus::block_on(target.process_atomic_key_event(
            KEY_LEFT_SHIFT,
            42,
            state,
            envelope(transaction),
            capability(3, true),
            (RECEIPT_NONE, 0, Vec::new()),
        ))
        .expect("cross-engine double Shift prefix");
        assert_eq!(proposal.0, PROPOSAL_NATIVE_UNHANDLED);
    }

    let undo = zbus::block_on(target.process_atomic_key_event(
        KEY_LEFT_SHIFT,
        42,
        RELEASE_MASK,
        envelope(505),
        capability(3, true),
        (RECEIPT_NONE, 0, Vec::new()),
    ))
    .expect("cross-engine exact undo");
    assert_eq!(undo.0, PROPOSAL_FRAME_READY);
    assert_eq!(committed_texts(&undo), ["ghbdtn "]);
    assert!(
        target.settle_atomic_pending(41, &(RECEIPT_SUBMITTED_ATOMIC, 505, vec![10; DIGEST_BYTES]),)
    );
    assert_eq!(target.tail_buffer, "ghbdtn ");
    assert!(target
        .shared
        .lock()
        .expect("shared state")
        .shift_gesture_handoff
        .is_none());
}

#[test]
fn v27_cross_engine_modifier_use_cannot_become_a_tap() {
    let shared = Arc::new(Mutex::new(Default::default()));
    let mut source = engine_with_shared("/atomic/modifier-us", Arc::clone(&shared), false, "");
    source.tail_buffer = "собака ".to_string();
    source.publish_tail_handoff();
    source.remember_pending_ime_auto_undo(
        "cj,frf ".to_string(),
        "собака ".to_string(),
        lay::typing_cpu::ObservedSystemTransition::LayoutProjection,
    );
    source.publish_active_path_preserve_handoff(Instant::now() + Duration::from_millis(700));
    source.shift_active = true;
    source.shift_pressed_at = Some(Instant::now());
    source.shift_used_as_modifier = true;
    source.last_shift_release_at = Some(Instant::now() - Duration::from_millis(100));
    source.publish_shift_gesture_handoff();

    let mut target = engine_with_shared("/atomic/modifier-ru", shared, true, "");
    let release = zbus::block_on(target.process_atomic_key_event(
        KEY_LEFT_SHIFT,
        42,
        RELEASE_MASK,
        envelope(601),
        capability(3, true),
        (RECEIPT_NONE, 0, Vec::new()),
    ))
    .expect("cross-engine modifier release");

    assert_eq!(release.0, PROPOSAL_NATIVE_UNHANDLED);
    assert!(!target.shift_active);
    assert!(target.shift_pressed_at.is_none());
    assert!(target.last_shift_release_at.is_none());
    assert!(target
        .shared
        .lock()
        .expect("shared state")
        .pending_auto_undo
        .is_some());
}

fn capability(maximum_effects: u8, delete_allowed: bool) -> AtomicCapability {
    (
        (
            PROTOCOL_VERSION,
            ADAPTER_KIND,
            PRODUCTION_DIGEST.to_vec(),
            TRANSACTION_KIND,
            EFFECT_MASK,
            maximum_effects,
            GUARANTEE_FLAGS,
        ),
        (
            11,
            12,
            13,
            14,
            15,
            delete_allowed,
            delete_allowed
                .then(|| vec![7; DIGEST_BYTES])
                .unwrap_or_default(),
        ),
    )
}

fn envelope(transaction: u64) -> AtomicEnvelope {
    (transaction, 2, 12, 41, 5, 13, vec![8; DIGEST_BYTES])
}

fn committed_texts(proposal: &AtomicProposal) -> Vec<String> {
    proposal
        .1
        .iter()
        .filter(|(tag, _)| *tag == 1)
        .map(|(_, value)| String::try_from(value.clone()).expect("CommitText string"))
        .collect()
}

fn prepare(engine: &LayIbusEngine) {
    let frame = engine
        .capture_input_frame_identity()
        .expect("exact printable frame");
    space_autocorrect_prefetch::proof::install_exact_lease(&frame, &engine.config);
}

#[test]
fn v27_atomic_space_refusal_and_double_shift_round_trip() {
    lay::exact_layout_authority::warm_up_exact_layout_authority_for_ibus()
        .expect("warm exact-layout authority");

    let mut refused = engine("/atomic/refused");
    prepare(&refused);
    let proposal = zbus::block_on(refused.process_atomic_key_event(
        KEY_SPACE,
        65,
        0,
        envelope(101),
        capability(1, true),
        (RECEIPT_NONE, 0, Vec::new()),
    ))
    .expect("refused exact Space");
    assert_eq!(proposal.0, PROPOSAL_NATIVE_UNHANDLED);
    assert!(proposal.1.is_empty());
    assert_eq!(refused.tail_buffer, "ghbdtn");
    assert!(!pending_transitions()
        .lock()
        .expect("pending transitions")
        .contains_key(&refused.path));

    let mut live = engine("/atomic/undo");
    prepare(&live);
    let applied = zbus::block_on(live.process_atomic_key_event(
        KEY_SPACE,
        65,
        0,
        envelope(201),
        capability(3, true),
        (RECEIPT_NONE, 0, Vec::new()),
    ))
    .expect("exact Space proposal");
    assert_eq!(applied.0, PROPOSAL_FRAME_READY);
    assert_eq!(
        committed_texts(&applied),
        ["\u{43f}\u{440}\u{438}\u{432}\u{435}\u{442} "]
    );
    assert_eq!(live.tail_buffer, "ghbdtn");
    live.observe_external_surrounding_text(Some(SurroundingTextSnapshot::new(
        "\u{43f}\u{440}\u{438}\u{432}\u{435}\u{442} ".to_string(),
        7,
        7,
    )));
    assert_eq!(live.surrounding_observation_revision, 1);

    let first_press = zbus::block_on(live.process_atomic_key_event(
        KEY_LEFT_SHIFT,
        42,
        0,
        envelope(202),
        capability(3, true),
        (RECEIPT_SUBMITTED_ATOMIC, 201, vec![9; DIGEST_BYTES]),
    ))
    .expect("settle exact Space and press Shift");
    assert_eq!(first_press.0, PROPOSAL_NATIVE_UNHANDLED);
    assert_eq!(
        live.tail_buffer,
        "\u{43f}\u{440}\u{438}\u{432}\u{435}\u{442} "
    );
    assert_eq!(live.surrounding_observation_revision, 1);
    assert_eq!(
        live.surrounding_text_snapshot
            .as_ref()
            .map(|snapshot| snapshot.text.as_str()),
        Some("\u{43f}\u{440}\u{438}\u{432}\u{435}\u{442} ")
    );
    assert!(live.pending_visible_postcondition.is_none());

    for (transaction, state) in [(203, RELEASE_MASK), (204, 0)] {
        let proposal = zbus::block_on(live.process_atomic_key_event(
            KEY_LEFT_SHIFT,
            42,
            state,
            envelope(transaction),
            capability(3, true),
            (RECEIPT_NONE, 0, Vec::new()),
        ))
        .expect("double Shift prefix");
        assert_eq!(proposal.0, PROPOSAL_NATIVE_UNHANDLED);
    }

    let undo = zbus::block_on(live.process_atomic_key_event(
        KEY_LEFT_SHIFT,
        42,
        RELEASE_MASK,
        envelope(205),
        capability(3, true),
        (RECEIPT_NONE, 0, Vec::new()),
    ))
    .expect("double Shift exact undo");
    assert_eq!(undo.0, PROPOSAL_FRAME_READY);
    assert_eq!(committed_texts(&undo), ["ghbdtn "]);
    assert_eq!(
        live.tail_buffer,
        "\u{43f}\u{440}\u{438}\u{432}\u{435}\u{442} "
    );
    assert!(
        live.settle_atomic_pending(41, &(RECEIPT_SUBMITTED_ATOMIC, 205, vec![10; DIGEST_BYTES]),)
    );
    assert_eq!(live.tail_buffer, "ghbdtn ");

    println!(
        "LAY_V27_ATOMIC_COMPOSITE refusal_zero_effect=PASS exact_space=PASS double_shift_exact_restore=PASS legacy_retry=0"
    );
}

#[test]
fn same_revision_preproposal_snapshot_is_not_resurrected() {
    lay::exact_layout_authority::warm_up_exact_layout_authority_for_ibus()
        .expect("warm exact-layout authority");

    let mut live = engine("/atomic/stale-surrounding");
    live.observe_external_surrounding_text(Some(SurroundingTextSnapshot::new(
        "ghbdtn".to_string(),
        6,
        6,
    )));
    assert_eq!(live.surrounding_observation_revision, 1);
    prepare(&live);

    let applied = zbus::block_on(live.process_atomic_key_event(
        KEY_SPACE,
        65,
        0,
        envelope(301),
        capability(3, true),
        (RECEIPT_NONE, 0, Vec::new()),
    ))
    .expect("exact Space proposal");
    assert_eq!(applied.0, PROPOSAL_FRAME_READY);
    assert!(live.surrounding_text_snapshot.is_some());

    let first_press = zbus::block_on(live.process_atomic_key_event(
        KEY_LEFT_SHIFT,
        42,
        0,
        envelope(302),
        capability(3, true),
        (RECEIPT_SUBMITTED_ATOMIC, 301, vec![11; DIGEST_BYTES]),
    ))
    .expect("settle exact Space");

    assert_eq!(first_press.0, PROPOSAL_NATIVE_UNHANDLED);
    assert_eq!(live.surrounding_observation_revision, 1);
    assert!(live.surrounding_text_snapshot.is_none());
    assert!(live.pending_visible_postcondition.is_some());
}

#[test]
fn newer_capability_loss_is_not_overwritten_by_speculation() {
    lay::exact_layout_authority::warm_up_exact_layout_authority_for_ibus()
        .expect("warm exact-layout authority");

    let mut live = engine("/atomic/capability-loss");
    live.observe_external_surrounding_text(Some(SurroundingTextSnapshot::new(
        "ghbdtn".to_string(),
        6,
        6,
    )));
    prepare(&live);

    let applied = zbus::block_on(live.process_atomic_key_event(
        KEY_SPACE,
        65,
        0,
        envelope(401),
        capability(3, true),
        (RECEIPT_NONE, 0, Vec::new()),
    ))
    .expect("exact Space proposal");
    assert_eq!(applied.0, PROPOSAL_FRAME_READY);

    live.set_client_capabilities(0);
    assert_eq!(live.surrounding_observation_revision, 2);
    assert!(!live.surrounding_text_supported);

    let first_press = zbus::block_on(live.process_atomic_key_event(
        KEY_LEFT_SHIFT,
        42,
        0,
        envelope(402),
        capability(3, true),
        (RECEIPT_SUBMITTED_ATOMIC, 401, vec![12; DIGEST_BYTES]),
    ))
    .expect("settle exact Space after capability loss");

    assert_eq!(first_press.0, PROPOSAL_NATIVE_UNHANDLED);
    assert_eq!(live.surrounding_observation_revision, 2);
    assert!(!live.surrounding_text_supported);
    assert!(live.surrounding_text_snapshot.is_none());
}

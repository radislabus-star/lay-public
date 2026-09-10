use std::sync::{Arc, Mutex};

use crate::engine::{InputFrameIdentity, LayIbusEngine};
use crate::output::{
    AtomicEffectBuilder, AtomicProposal, EngineOutput, PROPOSAL_FRAME_READY,
    PROPOSAL_NATIVE_UNHANDLED,
};
use crate::protocol::{Shared, KEY_BACKSPACE, KEY_SPACE};
use lay::config::LayConfig;

const LEFT_CONTEXT: &str = "left ";
const WRONG_LAYOUT_TOKEN: &str = "gjxbnfq";
const EXPECTED_CORRECTION: &str = "почитай ";
const IBUS_CAP_SURROUNDING_TEXT: u32 = 1 << 5;

fn exact_config(profile: &str) -> LayConfig {
    LayConfig {
        auto_replace: true,
        auto_switch_layout: true,
        correction_safety: profile.to_string(),
        text_backend: "ime".to_string(),
        ..LayConfig::default()
    }
}

fn bound_engine(
    path: String,
    shared: Shared,
    layout_is_ru: bool,
    config: LayConfig,
) -> LayIbusEngine {
    let mut engine = LayIbusEngine::new(path, shared, layout_is_ru, true, config);
    engine.set_client_capabilities(IBUS_CAP_SURROUNDING_TEXT);
    assert!(
        engine.bind_focus_path(),
        "fixture must bind a real focus owner"
    );
    engine
}

fn seed_tail(engine: &mut LayIbusEngine, text: &str) {
    for ch in text.chars() {
        engine.push_tail_char(ch);
    }
}

fn press(engine: &mut LayIbusEngine, keyval: u32) -> (bool, AtomicProposal) {
    let mut builder = AtomicEffectBuilder::default();
    let handled = {
        let mut output = EngineOutput::atomic(&mut builder);
        zbus::block_on(engine.process_pressed_key(&mut output, keyval, 0, 0))
            .expect("managed key route")
    };
    let proposal = builder.finish(handled);
    (handled, proposal)
}

fn unicode_keyval(ch: char) -> u32 {
    if ch.is_ascii() {
        ch as u32
    } else {
        0x0100_0000 | ch as u32
    }
}

fn type_through_managed_key_route(engine: &mut LayIbusEngine, text: &str) {
    for ch in text.chars() {
        let (handled, proposal) = press(engine, unicode_keyval(ch));
        assert!(handled, "printable {ch:?} must use managed CommitText");
        assert_eq!(proposal.0, PROPOSAL_FRAME_READY);
        assert_eq!(text_effect_tags(&proposal), [1]);
        assert_eq!(committed_texts(&proposal), [ch.to_string()]);
        assert!(deleted_ranges(&proposal).is_empty());
    }
}

fn press_native_backspace_without_text_mutation(engine: &mut LayIbusEngine) {
    let (handled, proposal) = press(engine, KEY_BACKSPACE);
    assert!(!handled, "committed-tail Backspace remains client-owned");
    assert!(text_effect_tags(&proposal).is_empty());
    assert!(committed_texts(&proposal).is_empty());
    assert!(deleted_ranges(&proposal).is_empty());
    if proposal.1.is_empty() {
        assert_eq!(proposal.0, PROPOSAL_NATIVE_UNHANDLED);
    } else {
        assert_eq!(proposal.0, PROPOSAL_FRAME_READY);
        assert!(
            proposal.1.iter().all(|(tag, _)| matches!(*tag, 3 | 4)),
            "native Backspace may publish only legitimate preedit effects"
        );
    }
}

fn erase_open_token_through_native_backspace(engine: &mut LayIbusEngine, token: &str) {
    for _ in token.chars() {
        press_native_backspace_without_text_mutation(engine);
    }
    assert_eq!(engine.committed_tail.buffer, LEFT_CONTEXT);
    assert!(engine.composition.preedit_fast.token().is_empty());
}

fn actual_manual_toggle(
    engine: &mut LayIbusEngine,
    source_token: &str,
    expected_replacement: &str,
    expected_target_layout_is_ru: bool,
) {
    let mut builder = AtomicEffectBuilder::default();
    let target_layout_is_ru = {
        let mut output = EngineOutput::atomic(&mut builder);
        zbus::block_on(engine.toggle_committed_tail_target(&mut output))
            .expect("committed-tail manual toggle")
    };
    assert_eq!(target_layout_is_ru, Some(expected_target_layout_is_ru));

    let proposal = builder.finish(true);
    assert_eq!(proposal.0, PROPOSAL_FRAME_READY);
    assert_eq!(text_effect_tags(&proposal), [2, 1]);
    assert_eq!(
        deleted_ranges(&proposal),
        [(
            -(source_token.chars().count() as i32),
            source_token.chars().count() as u32
        )]
    );
    assert_eq!(committed_texts(&proposal), [expected_replacement]);
    assert_eq!(
        engine.committed_tail.buffer,
        format!("{LEFT_CONTEXT}{expected_replacement}")
    );
}

fn former_word_manual_edit_then_us_handoff(profile: &str, case: &str) -> LayIbusEngine {
    let shared = Arc::new(Mutex::new(Default::default()));
    let config = exact_config(profile);
    let mut source = bound_engine(
        format!("/td120/o6/{case}/{profile}/ru"),
        Arc::clone(&shared),
        true,
        config.clone(),
    );
    seed_tail(&mut source, &format!("{LEFT_CONTEXT}вот"));
    actual_manual_toggle(&mut source, "вот", "djn", false);

    let target = bound_engine(
        format!("/td120/o6/{case}/{profile}/us"),
        shared,
        false,
        config,
    );
    assert_eq!(target.committed_tail.buffer, format!("{LEFT_CONTEXT}djn"));
    target
}

fn current_word_protected_us_handoff(profile: &str) -> LayIbusEngine {
    let shared = Arc::new(Mutex::new(Default::default()));
    let config = exact_config(profile);
    let mut source = bound_engine(
        format!("/td120/o6/protected/{profile}/ru"),
        Arc::clone(&shared),
        true,
        config.clone(),
    );
    seed_tail(&mut source, &format!("{LEFT_CONTEXT}почитай"));
    actual_manual_toggle(&mut source, "почитай", WRONG_LAYOUT_TOKEN, false);

    let mut target = bound_engine(
        format!("/td120/o6/protected/{profile}/us"),
        shared,
        false,
        config,
    );
    assert_eq!(
        target.committed_tail.buffer,
        format!("{LEFT_CONTEXT}{WRONG_LAYOUT_TOKEN}")
    );
    // A path handoff transfers the open word and its ordinary guard, but not
    // `last_input_at`. Continue the same word through real key routes and
    // restore its exact surface so closed exact-layout admission observes a
    // genuinely live frame without consuming or fabricating the guard.
    type_through_managed_key_route(&mut target, "x");
    press_native_backspace_without_text_mutation(&mut target);
    assert_eq!(
        target.committed_tail.buffer,
        format!("{LEFT_CONTEXT}{WRONG_LAYOUT_TOKEN}")
    );
    target
}

fn clean_us_frame(profile: &str) -> LayIbusEngine {
    let mut engine = bound_engine(
        format!("/td120/o6/clean/{profile}/us"),
        Arc::new(Mutex::new(Default::default())),
        false,
        exact_config(profile),
    );
    seed_tail(&mut engine, LEFT_CONTEXT);
    type_through_managed_key_route(&mut engine, WRONG_LAYOUT_TOKEN);
    engine
}

fn assert_complete_current_frame(engine: &LayIbusEngine) -> InputFrameIdentity {
    let frame = engine
        .capture_input_frame_identity()
        .expect("complete gjxbnfq input frame");
    assert_eq!(
        frame.committed_tail,
        format!("{LEFT_CONTEXT}{WRONG_LAYOUT_TOKEN}")
    );
    assert_eq!(frame.context_prefix, LEFT_CONTEXT);
    assert_eq!(frame.observed_token, WRONG_LAYOUT_TOKEN);
    assert!(
        frame.active_composition,
        "closed exact-layout admission requires a live printable frame"
    );
    assert!(!frame.active_layout_is_ru);
    assert!(frame.exact_authority_snapshot.is_some());
    assert!(frame.lexical_coordinates.is_some());
    assert!(frame.config_matches(&engine.config));
    assert!(engine.input_frame_identity_matches(&frame));
    frame
}

fn install_current_exact_lease(engine: &LayIbusEngine, frame: &InputFrameIdentity) {
    crate::space_autocorrect_prefetch::proof::install_exact_lease(frame, &engine.config);
}

fn assert_space_applies_exact_correction(engine: &mut LayIbusEngine) {
    let frame = assert_complete_current_frame(engine);
    install_current_exact_lease(engine, &frame);

    let (handled, proposal) = press(engine, KEY_SPACE);
    assert!(handled);
    assert_eq!(proposal.0, PROPOSAL_FRAME_READY);
    assert_eq!(text_effect_tags(&proposal), [2, 1]);
    assert_eq!(
        deleted_ranges(&proposal),
        [(
            -(WRONG_LAYOUT_TOKEN.chars().count() as i32),
            WRONG_LAYOUT_TOKEN.chars().count() as u32
        )]
    );
    assert_eq!(committed_texts(&proposal), [EXPECTED_CORRECTION]);
    assert_eq!(
        engine.committed_tail.buffer,
        format!("{LEFT_CONTEXT}{EXPECTED_CORRECTION}")
    );
}

fn text_effect_tags(proposal: &AtomicProposal) -> Vec<u8> {
    proposal
        .1
        .iter()
        .filter_map(|(tag, _)| matches!(*tag, 1 | 2).then_some(*tag))
        .collect()
}

fn committed_texts(proposal: &AtomicProposal) -> Vec<String> {
    proposal
        .1
        .iter()
        .filter(|(tag, _)| *tag == 1)
        .map(|(_, value)| String::try_from(value.clone()).expect("CommitText string"))
        .collect()
}

fn deleted_ranges(proposal: &AtomicProposal) -> Vec<(i32, u32)> {
    proposal
        .1
        .iter()
        .filter(|(tag, _)| *tag == 2)
        .map(|(_, value)| {
            let zbus::zvariant::Value::Structure(structure) = &**value else {
                panic!("DeleteSurroundingText structure");
            };
            let fields = structure.fields();
            match (fields.first(), fields.get(1), fields.get(2)) {
                (
                    Some(zbus::zvariant::Value::I32(offset)),
                    Some(zbus::zvariant::Value::U32(chars)),
                    None,
                ) => (*offset, *chars),
                _ => panic!("DeleteSurroundingText offset/count fields"),
            }
        })
        .collect()
}

#[test]
fn td120_o6_a_former_word_guard_is_absent_before_new_frame_all_safety_profiles() {
    lay::exact_layout_authority::warm_up_exact_layout_authority_for_ibus()
        .expect("warm exact-layout authority");

    for profile in ["strict", "normal", "experimental"] {
        let mut engine = former_word_manual_edit_then_us_handoff(profile, "guard-absence");
        erase_open_token_through_native_backspace(&mut engine, "djn");
        type_through_managed_key_route(&mut engine, WRONG_LAYOUT_TOKEN);

        assert_eq!(engine.last_tail_token_text(), WRONG_LAYOUT_TOKEN);
        assert!(
            !engine.take_manual_toggle_autocorrect_suppression(),
            "profile={profile}: a fresh word must not inherit the former ordinary guard"
        );
    }
}

#[test]
fn td120_o6_b_exact_space_frame_matrix_all_safety_profiles() {
    lay::exact_layout_authority::warm_up_exact_layout_authority_for_ibus()
        .expect("warm exact-layout authority");

    for profile in ["strict", "normal", "experimental"] {
        let mut after_former_word =
            former_word_manual_edit_then_us_handoff(profile, "former-word-apply");
        erase_open_token_through_native_backspace(&mut after_former_word, "djn");
        type_through_managed_key_route(&mut after_former_word, WRONG_LAYOUT_TOKEN);
        assert_space_applies_exact_correction(&mut after_former_word);

        let mut clean = clean_us_frame(profile);
        assert_space_applies_exact_correction(&mut clean);

        let mut protected = current_word_protected_us_handoff(profile);
        let protected_frame = assert_complete_current_frame(&protected);
        install_current_exact_lease(&protected, &protected_frame);
        let (handled, proposal) = press(&mut protected, KEY_SPACE);
        assert!(handled);
        assert_eq!(proposal.0, PROPOSAL_FRAME_READY);
        assert_eq!(text_effect_tags(&proposal), [1]);
        assert!(deleted_ranges(&proposal).is_empty());
        assert_eq!(committed_texts(&proposal), [" "]);
        assert_eq!(
            protected.committed_tail.buffer,
            format!("{LEFT_CONTEXT}{WRONG_LAYOUT_TOKEN} ")
        );
        assert!(!protected.take_manual_toggle_autocorrect_suppression());
    }
}

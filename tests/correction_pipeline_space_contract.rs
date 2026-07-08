use lay::config::{default_typing_assist_pipeline, CorrectionSafety};
use lay::correction_core::{CorrectionDecisionSource, CorrectionMode};
use lay::decoder::{decode_enter_autocorrect_tail, decode_typing_assist_tail, CorrectionSource};
use lay::input_gate::{decide_input_gate, InputGateAction, InputGateRequest, InputGateTrigger};
use lay::keyboard::text_to_key_events;

fn decide_space(text_tail: &str) -> lay::input_gate::InputGateDecision {
    decide_space_deterministic(text_tail)
}

fn decide_space_deterministic(text_tail: &str) -> lay::input_gate::InputGateDecision {
    let pipeline = default_typing_assist_pipeline();
    decide_input_gate(InputGateRequest {
        trigger: InputGateTrigger::Space,
        text_tail,
        auto_replace: true,
        typing_assist: true,
        auto_switch_layout: true,
        correction_safety: CorrectionSafety::Normal,
        typing_assist_pipeline: &pipeline,
        nanda_autocorrect: false,
        correction_mode: CorrectionMode::DeterministicOnly,
    })
}

#[test]
fn space_autocorrect_keeps_existing_public_gate_contract() {
    let cases = [
        ("читай логии ", "читай логи "),
        ("звгрузи ", "загрузи "),
        ("ghbdtn ", "привет "),
        ("file ljgecnbv ", "file допустим "),
    ];

    for (input, expected) in cases {
        let decision = decide_space(input);
        let InputGateAction::ApplyReplacement {
            replacement,
            source,
        } = decision.action
        else {
            panic!(
                "expected replacement for {input:?}, got {:?}, trace={:?}",
                decision.action, decision.trace
            );
        };

        assert_eq!(replacement, expected, "{input:?}");
        assert!(
            matches!(
                source,
                CorrectionDecisionSource::Deterministic | CorrectionDecisionSource::Nanda
            ),
            "{input:?}: unexpected source {source:?}"
        );
        assert!(
            decision.correction.is_some(),
            "{input:?}: correction resolution must be preserved"
        );
    }
}

#[test]
fn daemon_space_and_enter_decoders_share_input_gate_replacement_contract() {
    let pipeline = default_typing_assist_pipeline();
    let cases = [
        ("читай логии ", true, "читай логи "),
        ("звгрузи ", true, "загрузи "),
        ("ghbdtn ", false, "привет "),
        ("file ljgecnbv ", false, "file допустим "),
    ];

    for (input, layout_is_ru, expected) in cases {
        let gate = decide_space_deterministic(input);
        let InputGateAction::ApplyReplacement {
            replacement: gate_replacement,
            ..
        } = gate.action
        else {
            panic!(
                "expected gate replacement for {input:?}, got {:?}",
                gate.action
            );
        };
        assert_eq!(gate_replacement, expected, "{input:?}");

        let events = text_to_key_events(input, layout_is_ru)
            .unwrap_or_else(|| panic!("fixture must map to key events: {input:?}"));
        let space_plan =
            decode_typing_assist_tail(&events, true, &pipeline, CorrectionSource::TypingAssist)
                .unwrap_or_else(|| panic!("expected daemon space decoder plan for {input:?}"));
        assert_eq!(space_plan.replacement, gate_replacement, "{input:?}");
        assert!(space_plan.plan_matches_replacement(), "{input:?}");
        assert!(space_plan.preserves_committed_separator(), "{input:?}");

        if layout_is_ru {
            let enter_plan = decode_enter_autocorrect_tail(&events, true, true, &pipeline)
                .unwrap_or_else(|| panic!("expected daemon enter decoder plan for {input:?}"));
            assert_eq!(enter_plan.replacement, gate_replacement, "{input:?}");
            assert!(enter_plan.plan_matches_replacement(), "{input:?}");
            assert!(enter_plan.preserves_committed_separator(), "{input:?}");
        }
    }
}

#[test]
fn weak_known_word_drift_is_suggest_only_without_latent_state_proof() {
    let decision = decide_space_deterministic("мы можем ");

    assert!(
        !matches!(decision.action, InputGateAction::ApplyReplacement { .. }),
        "known-word drift must not auto-apply without latent transition proof: {:?}",
        decision.action
    );
    let trace = decision.trace.as_ref().expect("trace");
    assert_ne!(trace.reason, "apply_selected_candidate");
}

use lay::config::{default_typing_assist_pipeline, CorrectionSafety};
use lay::correction_core::{CorrectionDecisionSource, CorrectionMode};
use lay::input_gate::{decide_input_gate, InputGateAction, InputGateRequest, InputGateTrigger};

fn decide_space(text_tail: &str) -> lay::input_gate::InputGateDecision {
    let pipeline = default_typing_assist_pipeline();
    decide_input_gate(InputGateRequest {
        trigger: InputGateTrigger::Space,
        text_tail,
        auto_replace: true,
        typing_assist: true,
        auto_switch_layout: true,
        correction_safety: CorrectionSafety::Experimental,
        typing_assist_pipeline: &pipeline,
        nanda_autocorrect: true,
        correction_mode: CorrectionMode::DeterministicThenNanda,
    })
}

#[test]
fn space_autocorrect_keeps_existing_public_gate_contract() {
    let cases = [
        ("читай логии ", "читай логи "),
        ("звгрузи ", "загрузи "),
        ("ghbdtn ", "привет "),
    ];

    for (input, expected) in cases {
        let decision = decide_space(input);
        let InputGateAction::ApplyReplacement {
            replacement,
            source,
        } = decision.action
        else {
            panic!(
                "expected replacement for {input:?}, got {:?}",
                decision.action
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

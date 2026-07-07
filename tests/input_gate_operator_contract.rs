use lay::action_log::RecentActionGateTrace;
use lay::config::{CorrectionSafety, TypingAssistRuleConfig};
use lay::correction_core::CorrectionMode;
use lay::input_gate::{decide_input_gate, InputGateRequest, InputGateStage, InputGateTrigger};

fn request<'a>(trigger: InputGateTrigger, text_tail: &'a str) -> InputGateRequest<'a> {
    let empty_pipeline: &'a [TypingAssistRuleConfig] = &[];
    InputGateRequest {
        trigger,
        text_tail,
        auto_replace: true,
        typing_assist: true,
        auto_switch_layout: true,
        correction_safety: CorrectionSafety::Normal,
        typing_assist_pipeline: empty_pipeline,
        nanda_autocorrect: false,
        correction_mode: CorrectionMode::DeterministicOnly,
    }
}

#[test]
fn double_shift_trace_exposes_manual_toggle_operator_contract() {
    let decision = decide_input_gate(request(InputGateTrigger::DoubleShift, "ghbdtn"));
    assert_eq!(decision.stage, InputGateStage::ManualToggle);
    let trace = decision.trace.as_ref().expect("manual toggle trace");
    assert_eq!(trace.reason, "manual_toggle_route_observed");

    let recent = RecentActionGateTrace::from_input_gate(trace);
    assert_eq!(recent.stage, "manual_toggle");
    assert_eq!(recent.reason, "manual_toggle_route_observed");
}

#[test]
fn tab_trace_exposes_completion_operator_contract() {
    let decision = decide_input_gate(request(InputGateTrigger::TabAccept, "пров"));
    assert_eq!(decision.stage, InputGateStage::CompletionAccept);
    let trace = decision.trace.as_ref().expect("completion trace");
    assert_eq!(trace.reason, "completion_accept_route_observed");
}

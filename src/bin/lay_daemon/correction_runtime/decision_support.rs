use lay::action_log::RecentActionGateTrace;
use lay::config::{CorrectionSafety, TypingAssistRuleConfig};
use lay::correction_core::CorrectionMode;
use lay::input_gate::{decide_input_gate, InputGateRequest, InputGateTrigger};
use lay::manual_toggle::recovered_initial_double_shift_replacement;

use super::super::active_auto_switch_layout;

pub(super) fn manual_toggle_gate_trace(
    text_tail: &str,
    auto_replace: bool,
) -> Option<RecentActionGateTrace> {
    let empty_pipeline: &[TypingAssistRuleConfig] = &[];
    let decision = decide_input_gate(InputGateRequest {
        trigger: InputGateTrigger::DoubleShift,
        text_tail,
        auto_replace,
        typing_assist: false,
        auto_switch_layout: active_auto_switch_layout(),
        correction_safety: CorrectionSafety::Normal,
        typing_assist_pipeline: empty_pipeline,
        nanda_autocorrect: false,
        nanda_candidate_route: lay::correction_core::CandidateReadoutRoute::CompactL2,
        nanda_wave_options: lay::nanda_wave::WaveOptions::default(),
        correction_mode: CorrectionMode::DeterministicOnly,
    });
    decision
        .trace
        .as_ref()
        .map(RecentActionGateTrace::from_input_gate)
}

pub(super) fn recovered_initial_manual_toggle_target(
    mapped_orig: &str,
    mapped_target: &str,
    words_orig: usize,
    n_backspaces: u32,
) -> String {
    if words_orig != 1 || n_backspaces != mapped_orig.chars().count() as u32 {
        return mapped_target.to_string();
    }
    recovered_initial_double_shift_replacement(mapped_orig)
        .filter(|replacement| replacement != mapped_target)
        .unwrap_or_else(|| mapped_target.to_string())
}

use lay::action_log::RecentActionGateTrace;
use lay::config::{CorrectionSafety, TypingAssistRuleConfig};
use lay::correction_core::CorrectionMode;
use lay::input_gate::{decide_input_gate, InputGateRequest, InputGateTrigger};
use lay::word_buffer::WordBuffer;

use super::super::append_learning_log;

pub(crate) struct LayoutReplayMemory<'a> {
    pub(crate) replace_words: usize,
    pub(crate) target_is_ru: bool,
    pub(crate) force_replay_toggle: bool,
    pub(crate) original: &'a str,
    pub(crate) replacement: &'a str,
    pub(crate) words: usize,
    pub(crate) elapsed_ms: u128,
}

#[rustfmt::skip]
pub(crate) fn remember_layout_replay_success(buf: &mut WordBuffer, replay: LayoutReplayMemory<'_>) {
    buf.mark_replayed_layout(replay.replace_words, replay.target_is_ru);
    if !replay.force_replay_toggle && replay.original != replay.replacement {
        append_learning_log(
            "layout-replay",
            replay.original,
            replay.replacement,
            replay.replace_words,
            replay.words,
        );
    }
    lay::action_log::record_action_with_stages_and_gate(
        "layout-replay",
        replay.original,
        replay.replacement,
        replay.replace_words,
        replay.words,
        replay.elapsed_ms,
        None,
        None,
        manual_toggle_gate_trace(replay.original),
        true,
    );
}

fn manual_toggle_gate_trace(text_tail: &str) -> Option<RecentActionGateTrace> {
    let empty_pipeline: &[TypingAssistRuleConfig] = &[];
    let decision = decide_input_gate(InputGateRequest {
        trigger: InputGateTrigger::DoubleShift,
        text_tail,
        auto_replace: true,
        typing_assist: false,
        auto_switch_layout: false,
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

use super::super::memory::TypingAssistTiming;

pub(super) fn record_ime_timing(
    timing: TypingAssistTiming,
    replace_tail_ms: u128,
    layout_ms: u128,
    remember_ms: u128,
    forward_ms: u128,
) {
    lay::action_log::record_timing_profile(
        "typing-assist",
        "daemon-ime",
        &[
            ("decision", timing.decision_ms),
            ("ime_replace_tail_call", replace_tail_ms),
            ("layout", layout_ms),
            ("remember", remember_ms),
            ("forward", forward_ms),
            ("total", timing.started_at.elapsed().as_millis()),
        ],
    );
}

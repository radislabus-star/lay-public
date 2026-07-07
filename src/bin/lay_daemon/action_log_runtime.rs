use lay::action_log::RecentActionGateTrace;
use std::time::Instant;

pub(super) fn record_recent_action(
    kind: &str,
    from: &str,
    to: &str,
    replace_words: usize,
    words: usize,
    started_at: Instant,
    input_gate: Option<RecentActionGateTrace>,
    undo_available: bool,
) {
    lay::action_log::record_action_with_stages_and_gate(
        kind,
        from,
        to,
        replace_words,
        words,
        started_at.elapsed().as_millis(),
        None,
        None,
        input_gate,
        undo_available,
    );
}

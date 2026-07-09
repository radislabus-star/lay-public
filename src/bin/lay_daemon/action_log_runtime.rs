use lay::action_log::RecentActionGateTrace;
use std::time::Instant;

pub(super) struct RecentActionRecord<'a> {
    pub(super) kind: &'a str,
    pub(super) from: &'a str,
    pub(super) to: &'a str,
    pub(super) replace_words: usize,
    pub(super) words: usize,
    pub(super) started_at: Instant,
    pub(super) input_gate: Option<RecentActionGateTrace>,
    pub(super) undo_available: bool,
}

pub(super) fn record_recent_action(record: RecentActionRecord<'_>) {
    lay::action_log::record_action_with_stages_and_gate(
        record.kind,
        record.from,
        record.to,
        record.replace_words,
        record.words,
        record.started_at.elapsed().as_millis(),
        None,
        None,
        record.input_gate,
        record.undo_available,
    );
}

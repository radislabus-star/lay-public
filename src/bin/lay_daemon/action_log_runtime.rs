use std::time::Instant;

pub(super) fn record_recent_action(
    kind: &str,
    from: &str,
    to: &str,
    replace_words: usize,
    words: usize,
    started_at: Instant,
    undo_available: bool,
) {
    lay::action_log::record_action(
        kind,
        from,
        to,
        replace_words,
        words,
        started_at.elapsed().as_millis(),
        undo_available,
    );
}

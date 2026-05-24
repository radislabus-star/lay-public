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
    lay::action_log::record_action(
        "layout-replay",
        replay.original,
        replay.replacement,
        replay.replace_words,
        replay.words,
        replay.elapsed_ms,
        true,
    );
}

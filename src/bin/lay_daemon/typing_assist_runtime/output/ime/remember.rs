use lay::keyboard::KeyEvent;
use lay::text_edit::TextReplacement;
use lay::word_buffer::WordBuffer;

use super::super::memory::{
    remember_typing_assist_correction, TypingAssistMemoryContext, TypingAssistTiming,
};

pub(super) fn remember_ime_typing_correction(
    buf: &mut WordBuffer,
    events: &[KeyEvent],
    original: &str,
    replacement: &str,
    rule_id: Option<&str>,
    input_gate: Option<lay::action_log::RecentActionGateTrace>,
    timing: TypingAssistTiming,
) {
    let plan = TextReplacement {
        move_left: 0,
        backspaces: original.chars().count() as u32,
        insert: replacement.to_string(),
        move_right: 0,
    };
    remember_typing_assist_correction(TypingAssistMemoryContext {
        buf,
        events,
        plan: &plan,
        original,
        replacement,
        rule_id,
        input_gate,
        cursor_offset: 0,
        timing,
    });
}

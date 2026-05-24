use lay::decoder::DecoderEditPlan;

use super::super::super::log;
use super::super::TypingAssistOutcome;

pub(crate) fn should_defer_immediate_typing_edit(edit: &DecoderEditPlan) -> bool {
    edit.plan.move_right > 0
        && edit.plan.backspaces > 0
        && edit.plan.insert.chars().any(char::is_whitespace)
}

pub(crate) fn defer_complex_edit() -> TypingAssistOutcome {
    log("· typing-assist deferred: complex live edit needs safe boundary");
    TypingAssistOutcome::Deferred
}

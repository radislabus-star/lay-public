use lay::word_buffer::WordBuffer;

use super::super::super::{active_auto_switch_layout, active_typing_assist_words};
use super::super::{find_typing_assist_correction, TypingAssistCorrection};

pub(crate) fn next_correction_after_forwarded_spaces(
    buf: &WordBuffer,
    spaces: usize,
) -> Option<TypingAssistCorrection> {
    if spaces == 0 {
        return None;
    }
    find_typing_assist_correction(
        buf,
        active_auto_switch_layout(),
        active_typing_assist_words(),
    )
}

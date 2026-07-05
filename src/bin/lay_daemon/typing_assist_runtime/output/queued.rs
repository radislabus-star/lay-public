use lay::word_buffer::WordBuffer;

use super::super::TypingAssistCorrection;

pub(crate) fn next_correction_after_forwarded_spaces(
    buf: &WordBuffer,
    spaces: usize,
) -> Option<TypingAssistCorrection> {
    let _ = (buf, spaces);
    None
}

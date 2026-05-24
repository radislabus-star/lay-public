use super::TypingAssistCorrection;

pub(super) struct PendingTypingAssist {
    correction: TypingAssistCorrection,
    cursor_offset: u32,
    separator_released: bool,
}

impl PendingTypingAssist {
    pub(super) fn new(correction: TypingAssistCorrection) -> Self {
        Self {
            correction,
            cursor_offset: 0,
            separator_released: false,
        }
    }

    pub(super) fn with_cursor_offset(
        correction: TypingAssistCorrection,
        cursor_offset: u32,
    ) -> Self {
        Self {
            correction,
            cursor_offset,
            separator_released: true,
        }
    }

    pub(super) fn note_visible_char(&mut self) {
        self.cursor_offset = self.cursor_offset.saturating_add(1);
    }

    pub(super) fn note_separator_released(&mut self) {
        self.separator_released = true;
    }

    pub(super) fn ready_to_apply(&self) -> bool {
        self.separator_released
    }

    pub(super) fn into_parts(self) -> (TypingAssistCorrection, u32) {
        (self.correction, self.cursor_offset)
    }
}

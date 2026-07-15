use super::DaemonTextContext;
use super::TypingAssistCorrection;

pub(super) struct PendingTypingAssist {
    request_id: Option<u64>,
    correction: Option<TypingAssistCorrection>,
    cursor_offset: u32,
    separator_released: bool,
    text_context: DaemonTextContext,
}

impl PendingTypingAssist {
    #[cfg(test)]
    pub(super) fn new(correction: TypingAssistCorrection, text_context: DaemonTextContext) -> Self {
        Self {
            request_id: None,
            correction: Some(correction),
            cursor_offset: 0,
            separator_released: false,
            text_context,
        }
    }

    pub(super) fn with_cursor_offset(
        correction: TypingAssistCorrection,
        cursor_offset: u32,
        text_context: DaemonTextContext,
    ) -> Self {
        Self {
            request_id: None,
            correction: Some(correction),
            cursor_offset,
            separator_released: true,
            text_context,
        }
    }

    pub(super) fn waiting(request_id: u64, text_context: DaemonTextContext) -> Self {
        Self {
            request_id: Some(request_id),
            correction: None,
            cursor_offset: 0,
            separator_released: false,
            text_context,
        }
    }

    pub(super) fn request_id(&self) -> Option<u64> {
        self.request_id
    }

    pub(super) fn resolve(&mut self, correction: TypingAssistCorrection) {
        self.request_id = None;
        self.correction = Some(correction);
    }

    pub(super) fn note_visible_char(&mut self) {
        self.cursor_offset = self.cursor_offset.saturating_add(1);
    }

    pub(super) fn note_separator_released(&mut self) {
        self.separator_released = true;
    }

    pub(super) fn ready_to_apply(&self) -> bool {
        self.separator_released && self.correction.is_some()
    }

    pub(super) fn into_parts(self) -> Option<(TypingAssistCorrection, u32, DaemonTextContext)> {
        Some((self.correction?, self.cursor_offset, self.text_context))
    }
}

pub(super) fn drop_pending_after_following_word_started(
    pending: &mut Option<PendingTypingAssist>,
) -> bool {
    pending.take().is_some()
}

use super::engine::LayIbusEngine;
use super::trace;

impl LayIbusEngine {
    pub(super) fn trace_key(
        &self,
        stage: &str,
        keyval: u32,
        keycode: u32,
        handled: bool,
        decoded: Option<char>,
    ) {
        let sensitive = !self.content_allows_personalization();
        trace::record_key(
            stage,
            if sensitive { 0 } else { keyval },
            if sensitive { 0 } else { keycode },
            handled,
            (!sensitive).then_some(decoded).flatten(),
            if sensitive {
                0
            } else {
                self.committed_tail.buffer.chars().count()
            },
            if sensitive {
                0
            } else {
                self.composition.preedit_suffix.chars().count()
            },
        );
    }

    pub(super) fn trace_committed_tail_replace(
        &self,
        source: lay::text_edit::VisibleTailSource,
        output_route: &str,
        backspaces: u32,
        text: &str,
    ) {
        if self.content_allows_personalization() {
            trace::record_committed_tail_replace(source, output_route, backspaces, text);
        }
    }

    pub(super) fn trace_committed_tail_replace_guard(
        &self,
        source: lay::text_edit::VisibleTailSource,
        reason: &str,
        backspaces: u32,
        expected: &str,
        actual: &str,
    ) {
        if self.content_allows_personalization() {
            trace::record_committed_tail_replace_guard(
                source, reason, backspaces, expected, actual,
            );
        }
    }
}

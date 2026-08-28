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
        let sensitive = self.content_is_sensitive();
        trace::record_key(
            stage,
            keyval,
            keycode,
            handled,
            (!sensitive).then_some(decoded).flatten(),
            if sensitive {
                0
            } else {
                self.tail_buffer.chars().count()
            },
            if sensitive {
                0
            } else {
                self.preedit_suffix.chars().count()
            },
        );
    }
}

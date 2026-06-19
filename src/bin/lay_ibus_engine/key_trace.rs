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
        trace::record_key(
            stage,
            keyval,
            keycode,
            handled,
            decoded,
            self.tail_buffer.chars().count(),
            self.preedit_suffix.chars().count(),
        );
    }
}

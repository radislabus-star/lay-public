use std::process::Command;

use lay::keyboard::preferred_layout_for_text;

use super::engine::LayIbusEngine;
use super::trace;

const RU_ENGINE: &str = "lay-ime-ru";
const US_ENGINE: &str = "lay-ime-us";

impl LayIbusEngine {
    pub(super) fn sync_layout_after_committed_text(&mut self, text: &str) {
        if !self.config.auto_switch_layout {
            return;
        }
        let target_is_ru = preferred_layout_for_text(text, self.layout_is_ru);
        if target_is_ru == self.layout_is_ru {
            return;
        }
        let target_engine = ime_engine_for_layout(target_is_ru);
        let result = Command::new("ibus")
            .args(["engine", target_engine])
            .status();
        let ok = result.as_ref().is_ok_and(|status| status.success());
        if ok {
            self.layout_is_ru = target_is_ru;
        }
        trace::record_layout_sync(target_is_ru, target_engine, ok);
    }
}

fn ime_engine_for_layout(target_is_ru: bool) -> &'static str {
    if target_is_ru {
        RU_ENGINE
    } else {
        US_ENGINE
    }
}

#[cfg(test)]
mod tests {
    use super::ime_engine_for_layout;

    #[test]
    fn chooses_managed_ime_engine_for_target_layout() {
        assert_eq!(ime_engine_for_layout(true), "lay-ime-ru");
        assert_eq!(ime_engine_for_layout(false), "lay-ime-us");
    }
}

use lay::keyboard::preferred_layout_for_text;
#[cfg(not(test))]
use std::process::Command;

use super::engine::LayIbusEngine;
use super::trace;

const RU_ENGINE: &str = "lay-ime-ru";
const US_ENGINE: &str = "lay-ime-us";
#[cfg(not(test))]
const DBUS_DEST: &str = "org.gnome.Shell";
#[cfg(not(test))]
const DBUS_PATH: &str = "/io/github/radislabus_star/LayDaemon";
#[cfg(not(test))]
const DBUS_ACTIVATE_LAYOUT: &str = "io.github.radislabus_star.LayDaemon.ActivateLayout";

impl LayIbusEngine {
    pub(super) fn sync_layout_after_committed_text(&mut self, text: &str) {
        if !self.config.auto_switch_layout {
            return;
        }
        let target_is_ru = preferred_layout_for_text(text, self.layout_is_ru);
        if target_is_ru != self.layout_is_ru {
            self.layout_is_ru = target_is_ru;
        }
        self.publish_tail_handoff();
        let target_engine = ime_engine_for_layout(target_is_ru);
        let target_layout = shell_layout_for_layout(target_is_ru);
        let ok = ensure_ime_engine(target_engine) && ensure_shell_layout(target_layout);
        trace::record_layout_sync(target_is_ru, target_engine, ok);
    }

    pub(super) fn sync_layout_after_manual_toggle(&mut self, text: &str) {
        if !self.config.auto_switch_layout {
            return;
        }
        let target_is_ru = preferred_layout_for_text(text, self.layout_is_ru);
        if target_is_ru != self.layout_is_ru {
            self.layout_is_ru = target_is_ru;
        }
        self.publish_tail_handoff();
        let target_engine = ime_engine_for_layout(target_is_ru);
        let target_layout = shell_layout_for_layout(target_is_ru);
        let ok = ensure_ime_engine(target_engine) && ensure_shell_layout(target_layout);
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

fn shell_layout_for_layout(target_is_ru: bool) -> &'static str {
    if target_is_ru {
        "ru"
    } else {
        "us"
    }
}

#[cfg(not(test))]
fn ensure_ime_engine(engine: &str) -> bool {
    Command::new("ibus")
        .args(["engine", engine])
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .is_ok()
}

#[cfg(not(test))]
fn ensure_shell_layout(layout: &str) -> bool {
    Command::new("gdbus")
        .args([
            "call",
            "--session",
            "--dest",
            DBUS_DEST,
            "--object-path",
            DBUS_PATH,
            "--method",
            DBUS_ACTIVATE_LAYOUT,
            layout,
        ])
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .is_ok()
}

#[cfg(test)]
fn ensure_ime_engine(_engine: &str) -> bool {
    true
}

#[cfg(test)]
fn ensure_shell_layout(_layout: &str) -> bool {
    true
}

#[cfg(test)]
mod tests {
    use super::{ime_engine_for_layout, shell_layout_for_layout};

    #[test]
    fn chooses_managed_ime_engine_for_target_layout() {
        assert_eq!(ime_engine_for_layout(true), "lay-ime-ru");
        assert_eq!(ime_engine_for_layout(false), "lay-ime-us");
        assert_eq!(shell_layout_for_layout(true), "ru");
        assert_eq!(shell_layout_for_layout(false), "us");
    }

    #[test]
    fn manual_toggle_syncs_internal_layout_both_directions() {
        let mut engine = super::LayIbusEngine::new(
            "/test".to_string(),
            std::sync::Arc::new(std::sync::Mutex::new(Default::default())),
            false,
            true,
            lay::config::LayConfig {
                auto_switch_layout: true,
                ..lay::config::LayConfig::default()
            },
        );

        engine.sync_layout_after_manual_toggle("привет ");
        assert!(engine.layout_is_ru);

        engine.sync_layout_after_manual_toggle("hello ");
        assert!(!engine.layout_is_ru);
    }
}

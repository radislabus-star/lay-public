#[cfg(not(test))]
use std::process::Command;
use std::sync::{Arc, Condvar, Mutex, OnceLock};

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
        let target_engine = ime_engine_for_layout(target_is_ru);
        if target_is_ru == self.layout_is_ru {
            self.publish_tail_handoff();
            trace::record_layout_sync(target_is_ru, target_engine, true);
            return;
        }

        // CommitText already contains the corrected surface. Keep the physical
        // key path responsive by changing this engine's decoder immediately;
        // the process-level IBus engine switch is a postcondition owned by one
        // latest-only background worker.
        self.layout_is_ru = target_is_ru;
        self.publish_tail_handoff();
        request_active_ime_engine_switch(target_is_ru, target_engine);
        trace::record_layout_sync_requested(target_is_ru, target_engine);
    }

    pub(super) fn sync_layout_after_manual_toggle(&mut self, text: &str) {
        self.sync_layout_for_text_blocking(text);
    }

    fn sync_layout_for_text_blocking(&mut self, text: &str) {
        if !self.config.auto_switch_layout {
            return;
        }
        let target_is_ru = preferred_layout_for_text(text, self.layout_is_ru);
        let target_engine = ime_engine_for_layout(target_is_ru);
        self.publish_tail_handoff();
        if target_is_ru == self.layout_is_ru {
            trace::record_layout_sync(target_is_ru, target_engine, true);
            return;
        }
        let ok = switch_active_ime_engine(target_engine).is_ok();
        if ok {
            self.layout_is_ru = target_is_ru;
        }
        trace::record_layout_sync(target_is_ru, target_engine, ok);
    }

    pub(super) fn toggle_layout_from_modifier_hotkey(&mut self) -> bool {
        let target_is_ru = current_active_ime_layout_is_ru()
            .map(|current_is_ru| !current_is_ru)
            .unwrap_or(!self.layout_is_ru);
        let target_engine = ime_engine_for_layout(target_is_ru);
        let ok = activate_gnome_layout_for_ime(target_is_ru)
            .or_else(|_| switch_active_ime_engine(target_engine))
            .is_ok();
        if ok {
            self.layout_is_ru = target_is_ru;
        }
        trace::record_layout_sync(target_is_ru, target_engine, ok);
        ok
    }
}

#[derive(Clone, Copy)]
struct LayoutSwitchRequest {
    target_is_ru: bool,
    engine: &'static str,
}

#[derive(Default)]
struct LayoutSwitchState {
    desired: Option<LayoutSwitchRequest>,
}

struct LayoutSwitchWorker {
    state: Arc<(Mutex<LayoutSwitchState>, Condvar)>,
}

impl LayoutSwitchWorker {
    fn start() -> Self {
        let state = Arc::new((Mutex::new(LayoutSwitchState::default()), Condvar::new()));
        let worker_state = Arc::clone(&state);
        std::thread::Builder::new()
            .name("lay-layout-switch".to_string())
            .spawn(move || run_layout_switch_worker(worker_state))
            .expect("failed to start layout switch worker");
        Self { state }
    }

    fn schedule(&self, request: LayoutSwitchRequest) {
        let (lock, wake) = &*self.state;
        let Ok(mut state) = lock.lock() else {
            return;
        };
        state.desired = Some(request);
        wake.notify_one();
    }
}

fn run_layout_switch_worker(shared: Arc<(Mutex<LayoutSwitchState>, Condvar)>) {
    loop {
        let request = {
            let (lock, wake) = &*shared;
            let Ok(mut state) = lock.lock() else {
                return;
            };
            while state.desired.is_none() {
                let Ok(next) = wake.wait(state) else {
                    return;
                };
                state = next;
            }
            state.desired.take().expect("desired switch checked above")
        };
        let ok = switch_active_ime_engine(request.engine).is_ok();
        trace::record_layout_sync(request.target_is_ru, request.engine, ok);
    }
}

fn request_active_ime_engine_switch(target_is_ru: bool, engine: &'static str) {
    static WORKER: OnceLock<LayoutSwitchWorker> = OnceLock::new();
    WORKER
        .get_or_init(LayoutSwitchWorker::start)
        .schedule(LayoutSwitchRequest {
            target_is_ru,
            engine,
        });
}

fn ime_engine_for_layout(target_is_ru: bool) -> &'static str {
    if target_is_ru {
        RU_ENGINE
    } else {
        US_ENGINE
    }
}

fn switch_active_ime_engine(engine: &str) -> Result<(), String> {
    #[cfg(test)]
    {
        let _ = engine;
        Ok(())
    }

    #[cfg(not(test))]
    {
        let out = Command::new("timeout")
            .args(["0.12s", "ibus", "engine", engine])
            .output()
            .map_err(|error| error.to_string())?;
        if out.status.success() {
            Ok(())
        } else {
            let stderr = String::from_utf8_lossy(&out.stderr).trim().to_string();
            Err(if stderr.is_empty() {
                format!("ibus engine {engine} exited with {}", out.status)
            } else {
                stderr
            })
        }
    }
}

fn current_active_ime_layout_is_ru() -> Option<bool> {
    let engine = read_active_ime_engine()?;
    if engine == RU_ENGINE {
        Some(true)
    } else if engine == US_ENGINE {
        Some(false)
    } else {
        None
    }
}

#[cfg(not(test))]
fn read_active_ime_engine() -> Option<String> {
    let out = Command::new("timeout")
        .args(["0.08s", "ibus", "engine"])
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    Some(String::from_utf8_lossy(&out.stdout).trim().to_string())
}

#[cfg(test)]
fn read_active_ime_engine() -> Option<String> {
    None
}

fn activate_gnome_layout_for_ime(target_is_ru: bool) -> Result<(), String> {
    #[cfg(test)]
    {
        let _ = target_is_ru;
        Ok(())
    }

    #[cfg(not(test))]
    {
        let layout_id = if target_is_ru { "ru" } else { "us" };
        let out = Command::new("timeout")
            .args([
                "0.18s",
                "gdbus",
                "call",
                "--session",
                "--dest",
                "org.gnome.Shell",
                "--object-path",
                "/io/github/radislabus_star/LayDaemon",
                "--method",
                "io.github.radislabus_star.LayDaemon.ActivateLayout",
                layout_id,
            ])
            .output()
            .map_err(|error| error.to_string())?;
        if out.status.success() && String::from_utf8_lossy(&out.stdout).contains("true") {
            Ok(())
        } else {
            let stderr = String::from_utf8_lossy(&out.stderr).trim().to_string();
            Err(if stderr.is_empty() {
                format!("ActivateLayout {layout_id} exited with {}", out.status)
            } else {
                stderr
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::ime_engine_for_layout;

    #[test]
    fn chooses_managed_ime_engine_label_for_target_layout() {
        assert_eq!(ime_engine_for_layout(true), "lay-ime-ru");
        assert_eq!(ime_engine_for_layout(false), "lay-ime-us");
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

    #[test]
    fn modifier_hotkey_toggles_internal_layout_state() {
        let mut engine = super::LayIbusEngine::new(
            "/test".to_string(),
            std::sync::Arc::new(std::sync::Mutex::new(Default::default())),
            false,
            true,
            lay::config::LayConfig::default(),
        );

        assert!(engine.toggle_layout_from_modifier_hotkey());
        assert!(engine.layout_is_ru);
        assert!(engine.toggle_layout_from_modifier_hotkey());
        assert!(!engine.layout_is_ru);
    }
}

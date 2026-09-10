#[cfg(not(test))]
use std::process::Command;
use std::sync::{Arc, Condvar, Mutex, OnceLock};

use lay::keyboard::preferred_layout_for_text;

use super::context_admission::{ContextAdmissionAdapter, LayoutIntentToken};
use super::engine::{DeferredLayoutAction, LayIbusEngine};
use super::trace;

const RU_ENGINE: &str = "lay-ime-ru";
const US_ENGINE: &str = "lay-ime-us";
impl LayIbusEngine {
    pub(super) fn sync_layout_after_committed_text(&mut self, text: &str, owner: &'static str) {
        self.invalidate_input_frame_background_work();
        if !self.config.auto_switch_layout {
            return;
        }
        let target_is_ru = preferred_layout_for_text(text, self.layout_gesture.layout_is_ru);
        let target_engine = ime_engine_for_layout(target_is_ru);
        if self.atomic.speculation {
            if target_is_ru != self.layout_gesture.layout_is_ru {
                let previous_is_ru = self.layout_gesture.layout_is_ru;
                self.set_layout_is_ru(target_is_ru);
                self.publish_tail_handoff();
                self.atomic
                    .deferred_layout_actions
                    .push(DeferredLayoutAction::Background {
                        previous_is_ru,
                        target_is_ru,
                        engine: target_engine,
                    });
            } else {
                self.atomic
                    .deferred_layout_actions
                    .push(DeferredLayoutAction::CancelBackground);
            }
            return;
        }
        supersede_active_ime_engine_switch();
        if target_is_ru == self.layout_gesture.layout_is_ru {
            self.publish_tail_handoff();
            trace::record_layout_sync(target_is_ru, target_engine, true);
            return;
        }

        // CommitText already contains the corrected surface. Keep the physical
        // key path responsive by changing this engine's decoder immediately;
        // the process-level IBus engine switch is a postcondition owned by one
        // latest-only background worker.
        if !self.publish_tail_handoff() {
            return;
        }
        self.set_layout_is_ru(target_is_ru);
        trace::record(format!(
            r#"{{"kind":"ibus_layout_sync_owner","owner":"{owner}","target_is_ru":{target_is_ru}}}"#
        ));
        request_active_ime_engine_switch(
            target_is_ru,
            target_engine,
            self.context_admission.clone(),
            self.live_layout_intent_token(),
            self.context_admission_required,
        );
        trace::record_layout_sync_requested(target_is_ru, target_engine);
    }

    pub(super) fn sync_layout_after_manual_toggle(&mut self, text: &str) {
        self.sync_layout_for_text_blocking(text);
    }

    fn sync_layout_for_text_blocking(&mut self, text: &str) {
        self.invalidate_input_frame_background_work();
        if !self.config.auto_switch_layout {
            return;
        }
        let target_is_ru = preferred_layout_for_text(text, self.layout_gesture.layout_is_ru);
        let target_engine = ime_engine_for_layout(target_is_ru);
        if self.atomic.speculation {
            if target_is_ru != self.layout_gesture.layout_is_ru {
                let previous_is_ru = self.layout_gesture.layout_is_ru;
                self.set_layout_is_ru(target_is_ru);
                self.publish_tail_handoff();
                self.atomic
                    .deferred_layout_actions
                    .push(DeferredLayoutAction::Blocking {
                        previous_is_ru,
                        target_is_ru,
                        engine: target_engine,
                        activate_gnome: true,
                    });
            } else {
                self.atomic
                    .deferred_layout_actions
                    .push(DeferredLayoutAction::CancelBackground);
            }
            return;
        }
        if target_is_ru == self.layout_gesture.layout_is_ru {
            // A settled manual no-op is still newer than queued automatic
            // work, but it must not activate IBus/GNOME.
            supersede_active_ime_engine_switch();
            self.publish_tail_handoff();
            trace::record_layout_sync(target_is_ru, target_engine, true);
            return;
        }
        if !self.publish_tail_handoff() {
            return;
        }
        let dispatch = dispatch_direct_layout_switch(
            target_is_ru,
            target_engine,
            self.context_admission.clone(),
            self.live_layout_intent_token(),
            || switch_complete_layout_stack(target_is_ru, target_engine),
        );
        let ok = dispatch.promotes_decoder();
        if ok {
            self.set_layout_is_ru(target_is_ru);
        }
        trace::record_layout_sync(target_is_ru, target_engine, ok);
    }

    pub(super) fn toggle_layout_from_modifier_hotkey(&mut self) -> bool {
        self.invalidate_input_frame_background_work();
        let target_is_ru = current_active_ime_layout_is_ru()
            .map(|current_is_ru| !current_is_ru)
            .unwrap_or(!self.layout_gesture.layout_is_ru);
        let target_engine = ime_engine_for_layout(target_is_ru);
        if self.atomic.speculation {
            let previous_is_ru = self.layout_gesture.layout_is_ru;
            self.set_layout_is_ru(target_is_ru);
            self.publish_tail_handoff();
            self.atomic
                .deferred_layout_actions
                .push(DeferredLayoutAction::Blocking {
                    previous_is_ru,
                    target_is_ru,
                    engine: target_engine,
                    activate_gnome: true,
                });
            return true;
        }
        let dispatch = dispatch_direct_layout_switch(
            target_is_ru,
            target_engine,
            self.context_admission.clone(),
            self.live_layout_intent_token(),
            || switch_complete_layout_stack(target_is_ru, target_engine),
        );
        let ok = dispatch.promotes_decoder();
        if ok {
            self.set_layout_is_ru(target_is_ru);
        }
        trace::record_layout_sync(target_is_ru, target_engine, ok);
        ok
    }

    pub(super) fn apply_deferred_layout_actions(&mut self) {
        self.atomic.speculation = false;
        for action in std::mem::take(&mut self.atomic.deferred_layout_actions) {
            match action {
                DeferredLayoutAction::Background {
                    previous_is_ru,
                    target_is_ru,
                    engine,
                } => {
                    let _ = previous_is_ru;
                    request_active_ime_engine_switch(
                        target_is_ru,
                        engine,
                        self.context_admission.clone(),
                        self.live_layout_intent_token(),
                        self.context_admission_required,
                    );
                    trace::record_layout_sync_requested(target_is_ru, engine);
                }
                DeferredLayoutAction::CancelBackground => {
                    supersede_active_ime_engine_switch();
                }
                DeferredLayoutAction::Blocking {
                    previous_is_ru,
                    target_is_ru,
                    engine,
                    activate_gnome,
                } => {
                    let dispatch = if activate_gnome {
                        dispatch_direct_layout_switch(
                            target_is_ru,
                            engine,
                            self.context_admission.clone(),
                            self.live_layout_intent_token(),
                            || switch_complete_layout_stack(target_is_ru, engine),
                        )
                    } else {
                        dispatch_direct_layout_switch(
                            target_is_ru,
                            engine,
                            self.context_admission.clone(),
                            self.live_layout_intent_token(),
                            || switch_active_ime_engine(engine),
                        )
                    };
                    let ok = dispatch.promotes_decoder();
                    self.set_layout_is_ru(if ok { target_is_ru } else { previous_is_ru });
                    self.publish_tail_handoff();
                    trace::record_layout_sync(target_is_ru, engine, ok);
                }
            }
        }
    }
}

#[derive(Clone)]
struct LayoutSwitchRequest {
    request_generation: u64,
    target_is_ru: bool,
    engine: &'static str,
    authority: LayoutSwitchAuthority,
}

impl LayoutSwitchRequest {
    fn context_is_live(&self) -> bool {
        self.authority.context_is_live()
    }

    fn revoke_text_authority(&self) {
        self.authority.revoke_text_authority();
    }
}

#[derive(Clone)]
enum LayoutSwitchAuthority {
    Background {
        admission_required: bool,
        admission: Option<ContextAdmissionAdapter>,
        token: Option<LayoutIntentToken>,
    },
    Direct {
        admission: Option<ContextAdmissionAdapter>,
        token: Option<LayoutIntentToken>,
    },
}

impl LayoutSwitchAuthority {
    fn context_is_live(&self) -> bool {
        match self {
            Self::Background {
                admission_required,
                admission,
                token,
            } => match (admission, token) {
                (Some(admission), Some(token)) => admission.revalidate_layout_intent(token),
                (None, None) => !admission_required,
                _ => false,
            },
            Self::Direct { admission, token } => match (admission, token) {
                (Some(admission), Some(token)) => admission.revalidate_layout_intent(token),
                (None, None) | (Some(_), None) => true,
                (None, Some(_)) => false,
            },
        }
    }

    fn revoke_text_authority(&self) {
        let (admission, token) = match self {
            Self::Background {
                admission, token, ..
            }
            | Self::Direct { admission, token } => (admission, token),
        };
        if let (Some(admission), Some(token)) = (admission, token) {
            let _ = admission.revoke_layout_intent(token);
        }
    }
}

#[derive(Default)]
struct LayoutSwitchState {
    desired: Option<LayoutSwitchRequest>,
    latest_request_generation: u64,
}

impl LayoutSwitchState {
    fn schedule(&mut self, mut request: LayoutSwitchRequest) {
        self.latest_request_generation =
            next_layout_request_generation(self.latest_request_generation);
        request.request_generation = self.latest_request_generation;
        self.desired = Some(request);
    }

    fn request_is_current(&self, request: &LayoutSwitchRequest) -> bool {
        request.request_generation == self.latest_request_generation
    }

    fn supersede_pending(&mut self) {
        self.latest_request_generation =
            next_layout_request_generation(self.latest_request_generation);
        self.desired = None;
    }
}

fn next_layout_request_generation(generation: u64) -> u64 {
    generation.wrapping_add(1).max(1)
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
        state.schedule(request);
        wake.notify_one();
    }

    fn supersede_pending(&self) {
        let (lock, wake) = &*self.state;
        let Ok(mut state) = lock.lock() else {
            return;
        };
        state.supersede_pending();
        wake.notify_one();
    }

    fn reserve_generation(&self) -> u64 {
        let (lock, wake) = &*self.state;
        let Ok(mut state) = lock.lock() else {
            return 0;
        };
        state.supersede_pending();
        wake.notify_one();
        state.latest_request_generation
    }

    fn request_is_current(&self, request: &LayoutSwitchRequest) -> bool {
        let (lock, _) = &*self.state;
        lock.lock()
            .is_ok_and(|state| state.request_is_current(request))
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
        match dispatch_layout_switch(
            &request,
            || layout_request_is_current(&shared, &request),
            || switch_complete_layout_stack(request.target_is_ru, request.engine),
        ) {
            LayoutSwitchDispatch::Superseded => {
                trace::record(
                    r#"{"kind":"ibus_layout_sync","status":"superseded_request_censored"}"#,
                );
            }
            LayoutSwitchDispatch::StaleContext => {
                trace::record(r#"{"kind":"ibus_layout_sync","status":"stale_context_censored"}"#);
            }
            LayoutSwitchDispatch::Completed { switched } => {
                trace::record_layout_sync(request.target_is_ru, request.engine, switched);
            }
            LayoutSwitchDispatch::ContextConflict { switched } => {
                trace::record(
                    r#"{"kind":"ibus_layout_sync","status":"in_flight_context_conflict"}"#,
                );
                trace::record_layout_sync(request.target_is_ru, request.engine, switched);
            }
            LayoutSwitchDispatch::SupersededCompletion { switched } => {
                trace::record(
                    r#"{"kind":"ibus_layout_sync","status":"stale_completion_authority_revoked"}"#,
                );
                trace::record_layout_sync(request.target_is_ru, request.engine, switched);
            }
        }
    }
}

fn layout_request_is_current(
    shared: &Arc<(Mutex<LayoutSwitchState>, Condvar)>,
    request: &LayoutSwitchRequest,
) -> bool {
    shared
        .0
        .lock()
        .is_ok_and(|state| state.request_is_current(request))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LayoutSwitchDispatch {
    Superseded,
    StaleContext,
    Completed { switched: bool },
    ContextConflict { switched: bool },
    SupersededCompletion { switched: bool },
}

impl LayoutSwitchDispatch {
    fn promotes_decoder(self) -> bool {
        matches!(self, Self::Completed { switched: true })
    }
}

fn dispatch_layout_switch(
    request: &LayoutSwitchRequest,
    request_is_current: impl Fn() -> bool,
    switch: impl FnOnce() -> Result<(), String>,
) -> LayoutSwitchDispatch {
    if !request_is_current() {
        return LayoutSwitchDispatch::Superseded;
    }
    if !request.context_is_live() {
        return LayoutSwitchDispatch::StaleContext;
    }
    let switched = switch().is_ok();
    if !request.context_is_live() {
        request.revoke_text_authority();
        return LayoutSwitchDispatch::ContextConflict { switched };
    }
    if !request_is_current() {
        request.revoke_text_authority();
        return LayoutSwitchDispatch::SupersededCompletion { switched };
    }
    LayoutSwitchDispatch::Completed { switched }
}

fn request_active_ime_engine_switch(
    target_is_ru: bool,
    engine: &'static str,
    admission: Option<ContextAdmissionAdapter>,
    token: Option<LayoutIntentToken>,
    admission_required: bool,
) {
    layout_switch_worker().schedule(LayoutSwitchRequest {
        request_generation: 0,
        target_is_ru,
        engine,
        authority: LayoutSwitchAuthority::Background {
            admission_required,
            admission,
            token,
        },
    });
}

fn supersede_active_ime_engine_switch() {
    layout_switch_worker().supersede_pending();
}

fn dispatch_direct_layout_switch(
    target_is_ru: bool,
    engine: &'static str,
    admission: Option<ContextAdmissionAdapter>,
    token: Option<LayoutIntentToken>,
    switch: impl FnOnce() -> Result<(), String>,
) -> LayoutSwitchDispatch {
    let request = LayoutSwitchRequest {
        request_generation: layout_switch_worker().reserve_generation(),
        target_is_ru,
        engine,
        authority: LayoutSwitchAuthority::Direct { admission, token },
    };
    dispatch_layout_switch(
        &request,
        || layout_switch_worker().request_is_current(&request),
        switch,
    )
}

fn layout_switch_worker() -> &'static LayoutSwitchWorker {
    static WORKER: OnceLock<LayoutSwitchWorker> = OnceLock::new();
    WORKER.get_or_init(LayoutSwitchWorker::start)
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

fn switch_complete_layout_stack(target_is_ru: bool, engine: &str) -> Result<(), String> {
    let _ = engine;
    activate_gnome_layout_for_ime(target_is_ru)
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
#[path = "layout_sync/tests.rs"]
mod tests;

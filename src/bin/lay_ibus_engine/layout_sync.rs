#[cfg(not(test))]
use std::process::Command;
use std::sync::{Arc, Condvar, Mutex, OnceLock};

#[cfg(test)]
use std::cell::RefCell;

use lay::desktop::LayoutBackend;
use lay::keyboard::preferred_layout_for_text;

#[path = "gnome/layout_sync.rs"]
mod gnome;
#[path = "kde/layout_sync.rs"]
mod kde;

use gnome::activate_gnome_layout_for_ime;

use super::context_admission::{ContextAdmissionAdapter, LayoutIntentToken};
use super::engine::{DeferredLayoutAction, LayIbusEngine};
use super::trace;

const RU_ENGINE: &str = "lay-ime-ru";
const US_ENGINE: &str = "lay-ime-us";

#[cfg(test)]
#[derive(Default)]
pub(super) struct LayoutStackProbe {
    pub(super) active_engine: Option<String>,
    pub(super) after_engine_read: Option<Box<dyn FnOnce()>>,
    pub(super) activations: Vec<bool>,
    pub(super) kde_activations: Vec<bool>,
    pub(super) ibus_switches: Vec<String>,
    pub(super) fail_kde_activation: bool,
    pub(super) after_kde_activation: Option<Box<dyn FnOnce()>>,
}

#[cfg(test)]
thread_local! {
    pub(super) static LAYOUT_STACK_PROBE: RefCell<Option<LayoutStackProbe>> = const { RefCell::new(None) };
}

impl LayIbusEngine {
    pub(super) fn sync_layout_after_committed_text(&mut self, text: &str, owner: &'static str) {
        self.invalidate_input_frame_background_work();
        if !self.config.auto_switch_layout {
            return;
        }
        let target_is_ru = preferred_layout_for_text(text, self.layout_gesture.layout_is_ru);
        let target_engine = ime_engine_for_layout(target_is_ru);
        // An IME-owned word may already have changed its physical decoder
        // while its selected source stayed fixed to preserve the preedit.
        let source_is_ru = self
            .client_context
            .factory_engine_profile
            .initial_layout_is_ru();
        if self.atomic.speculation {
            if target_is_ru != source_is_ru {
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
        if target_is_ru == source_is_ru {
            self.set_layout_is_ru(target_is_ru);
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
            self.config.active_layout_backend(),
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

    pub(super) fn sync_layout_for_owned_preedit(&self, target_is_ru: bool) -> Result<(), String> {
        let backend = self.config.active_layout_backend();
        if backend != LayoutBackend::Kde {
            // The existing native InputMode publication owns this boundary on
            // GNOME. Replacing the IBus engine would discard its open preedit.
            return Ok(());
        }
        if !self.context_owns_active_preedit_manual_toggle() {
            return Err("owned preedit lost authority before KDE layout completion".into());
        }
        let (Some(admission), Some(token)) = (
            self.context_admission.clone(),
            self.live_layout_intent_token(),
        ) else {
            return Err("owned preedit has no live KDE layout intent".into());
        };
        let dispatch = dispatch_direct_layout_switch(
            backend,
            target_is_ru,
            ime_engine_for_layout(target_is_ru),
            Some(admission),
            Some(token),
            |request| {
                request.switch_owned_preedit_group(|| {
                    layout_switch_worker().request_is_current(request)
                })
            },
        );
        if dispatch.promotes_decoder() {
            Ok(())
        } else {
            Err(format!("owned preedit KDE layout refused: {dispatch:?}"))
        }
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
            self.config.active_layout_backend(),
            target_is_ru,
            target_engine,
            self.context_admission.clone(),
            self.live_layout_intent_token(),
            |request| request.switch_stack(|| layout_switch_worker().request_is_current(request)),
        );
        let ok = dispatch.promotes_decoder();
        if !ok && trace::enabled() {
            trace::record(format!(
                r#"{{"kind":"ibus_manual_layout_dispatch_refusal","target_is_ru":{target_is_ru},"dispatch":"{dispatch:?}"}}"#,
            ));
        }
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
            self.config.active_layout_backend(),
            target_is_ru,
            target_engine,
            self.context_admission.clone(),
            self.live_layout_intent_token(),
            |request| request.switch_stack(|| layout_switch_worker().request_is_current(request)),
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
                        self.config.active_layout_backend(),
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
                            self.config.active_layout_backend(),
                            target_is_ru,
                            engine,
                            self.context_admission.clone(),
                            self.live_layout_intent_token(),
                            |request| {
                                request.switch_stack(|| {
                                    layout_switch_worker().request_is_current(request)
                                })
                            },
                        )
                    } else {
                        dispatch_direct_layout_switch(
                            self.config.active_layout_backend(),
                            target_is_ru,
                            engine,
                            self.context_admission.clone(),
                            self.live_layout_intent_token(),
                            |_| switch_active_ime_engine(engine),
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
    backend: LayoutBackend,
    target_is_ru: bool,
    engine: &'static str,
    authority: LayoutSwitchAuthority,
}

impl LayoutSwitchRequest {
    fn switch_owned_preedit_group(
        &self,
        request_is_current: impl Fn() -> bool,
    ) -> Result<(), String> {
        if self.backend != LayoutBackend::Kde {
            return Err("owned preedit group completion requires KDE".into());
        }
        complete_kde_owned_preedit_group(self.target_is_ru, || {
            request_is_current() && self.context_is_live()
        })
    }

    fn switch_stack(&self, request_is_current: impl Fn() -> bool) -> Result<(), String> {
        switch_complete_layout_stack(self.backend, self.target_is_ru, self.engine, || {
            request_is_current() && self.context_is_live()
        })
    }

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
                (Some(admission), Some(token)) => {
                    // This request completes forward input mode for already
                    // dispatched text. Reset still retires all word authority.
                    admission.revalidate_committed_layout_intent(token)
                }
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
            || request.switch_stack(|| layout_request_is_current(&shared, &request)),
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
    backend: LayoutBackend,
    target_is_ru: bool,
    engine: &'static str,
    admission: Option<ContextAdmissionAdapter>,
    token: Option<LayoutIntentToken>,
    admission_required: bool,
) {
    layout_switch_worker().schedule(LayoutSwitchRequest {
        request_generation: 0,
        backend,
        target_is_ru,
        engine,
        authority: LayoutSwitchAuthority::Background {
            admission_required,
            admission,
            token,
        },
    });
}

pub(super) fn supersede_active_ime_engine_switch() {
    layout_switch_worker().supersede_pending();
}

/// Capture a Space-generation request for a controlled late dispatch using
/// the real worker's generation and existing authority/dispatch contract.
#[cfg(test)]
pub(crate) fn delayed_automatic_layout_dispatch_for_test(
    admission: ContextAdmissionAdapter,
    token: LayoutIntentToken,
    target_is_ru: bool,
    during_switch: impl FnOnce(),
) -> impl FnOnce() -> (bool, usize, bool) {
    let worker = layout_switch_worker();
    let request_generation = worker.state.0.lock().unwrap().latest_request_generation;
    let request = LayoutSwitchRequest {
        request_generation,
        backend: LayoutBackend::Gnome,
        target_is_ru,
        engine: ime_engine_for_layout(target_is_ru),
        authority: LayoutSwitchAuthority::Background {
            admission_required: true,
            admission: Some(admission),
            token: Some(token),
        },
    };
    move || {
        let mut switches = 0;
        let result = dispatch_layout_switch(
            &request,
            || worker.request_is_current(&request),
            || {
                switches += 1;
                during_switch();
                Ok(())
            },
        );
        (
            matches!(result, LayoutSwitchDispatch::Superseded),
            switches,
            result.promotes_decoder(),
        )
    }
}

fn dispatch_direct_layout_switch(
    backend: LayoutBackend,
    target_is_ru: bool,
    engine: &'static str,
    admission: Option<ContextAdmissionAdapter>,
    token: Option<LayoutIntentToken>,
    switch: impl FnOnce(&LayoutSwitchRequest) -> Result<(), String>,
) -> LayoutSwitchDispatch {
    let request = LayoutSwitchRequest {
        request_generation: layout_switch_worker().reserve_generation(),
        backend,
        target_is_ru,
        engine,
        authority: LayoutSwitchAuthority::Direct { admission, token },
    };
    dispatch_layout_switch(
        &request,
        || layout_switch_worker().request_is_current(&request),
        || switch(&request),
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
        LAYOUT_STACK_PROBE.with(|probe| {
            if let Some(probe) = probe.borrow_mut().as_mut() {
                probe.ibus_switches.push(engine.to_string());
                probe.active_engine = Some(engine.to_string());
            }
        });
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

fn switch_complete_layout_stack(
    backend: LayoutBackend,
    target_is_ru: bool,
    engine: &str,
    is_live: impl Fn() -> bool,
) -> Result<(), String> {
    if !is_live() {
        return Err("layout intent is no longer current".to_string());
    }
    if backend == LayoutBackend::Kde {
        return complete_kde_layout_stack(target_is_ru, engine, is_live);
    }
    activate_gnome_layout_for_ime(target_is_ru)?;
    // GNOME may select the Lay source while IBus still reports its fallback
    // XKB engine, especially when Firefox resets the context during the same
    // Space callback. Complete this one layout operation at the IBus boundary
    // only if the selected engine has not followed GNOME yet.
    complete_ibus_layout(target_is_ru, engine, || true)?;
    Ok(())
}

fn complete_ibus_layout(
    target_is_ru: bool,
    engine: &str,
    is_live: impl Fn() -> bool,
) -> Result<(), String> {
    let observed = current_active_ime_layout_is_ru();
    if !is_live() {
        return Err("layout intent changed during IBus observation".to_string());
    }
    if observed != Some(target_is_ru) {
        switch_active_ime_engine(engine)?;
    }
    Ok(())
}

#[cfg(not(test))]
fn complete_kde_owned_preedit_group(
    target_is_ru: bool,
    is_live: impl Fn() -> bool,
) -> Result<(), String> {
    // Reuse the same bounded unique-owner/mapping/readback transaction. Its
    // engine step is deliberately empty while this engine owns open preedit.
    kde::switch(target_is_ru, is_live, || Ok(()))
}

#[cfg(test)]
fn complete_kde_owned_preedit_group(
    target_is_ru: bool,
    is_live: impl Fn() -> bool,
) -> Result<(), String> {
    if !is_live() {
        return Err("owned preedit layout intent is no longer current".into());
    }
    let (failed, after_activation) = LAYOUT_STACK_PROBE.with(|slot| {
        let mut slot = slot.borrow_mut();
        let Some(probe) = slot.as_mut() else {
            return (false, None);
        };
        probe.kde_activations.push(target_is_ru);
        (probe.fail_kde_activation, probe.after_kde_activation.take())
    });
    if failed {
        return Err("injected KDE group rejection".into());
    }
    if let Some(after_activation) = after_activation {
        after_activation();
    }
    if !is_live() {
        return Err("owned preedit layout intent changed during KDE activation".into());
    }
    Ok(())
}

#[cfg(not(test))]
fn complete_kde_layout_stack(
    target_is_ru: bool,
    engine: &str,
    is_live: impl Fn() -> bool,
) -> Result<(), String> {
    kde::switch(target_is_ru, &is_live, || {
        complete_kde_ibus_layout(target_is_ru, engine, &is_live)
    })
}

#[cfg(test)]
fn complete_kde_layout_stack(
    target_is_ru: bool,
    engine: &str,
    is_live: impl Fn() -> bool,
) -> Result<(), String> {
    LAYOUT_STACK_PROBE.with(|probe| {
        if let Some(probe) = probe.borrow_mut().as_mut() {
            probe.kde_activations.push(target_is_ru);
        }
    });
    if !is_live() {
        return Err("layout intent changed during activation".to_string());
    }
    complete_kde_ibus_layout(target_is_ru, engine, is_live)
}

fn complete_kde_ibus_layout(
    target_is_ru: bool,
    engine: &str,
    is_live: impl Fn() -> bool,
) -> Result<(), String> {
    complete_ibus_layout(target_is_ru, engine, is_live)?;
    if current_active_ime_layout_is_ru() != Some(target_is_ru) {
        return Err("IBus engine did not follow the KDE layout intent".to_string());
    }
    Ok(())
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
    let (engine, after_read) = LAYOUT_STACK_PROBE.with(|probe| {
        probe
            .borrow_mut()
            .as_mut()
            .map(|probe| (probe.active_engine.clone(), probe.after_engine_read.take()))
            .unwrap_or_default()
    });
    if let Some(after_read) = after_read {
        after_read();
    }
    engine
}

#[cfg(test)]
#[path = "layout_sync/tests.rs"]
mod tests;

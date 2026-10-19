use lay::desktop::LayoutBackend;
use std::cell::{Cell, RefCell};

use super::super::context_admission::{ContextAdmissionAdapter, WordCompleteness, WordScope};
use super::{
    dispatch_layout_switch, ime_engine_for_layout, LayoutSwitchAuthority, LayoutSwitchDispatch,
    LayoutSwitchRequest, LayoutSwitchState,
};

fn background_request(
    admission: Option<ContextAdmissionAdapter>,
    token: Option<super::super::context_admission::LayoutIntentToken>,
    admission_required: bool,
) -> LayoutSwitchRequest {
    LayoutSwitchRequest {
        request_generation: 0,
        backend: LayoutBackend::Gnome,
        target_is_ru: true,
        engine: "lay-ime-ru",
        authority: LayoutSwitchAuthority::Background {
            admission_required,
            admission,
            token,
        },
    }
}

fn request(
    admission: Option<ContextAdmissionAdapter>,
    token: Option<super::super::context_admission::LayoutIntentToken>,
) -> LayoutSwitchRequest {
    let admission_required = admission.is_some();
    background_request(admission, token, admission_required)
}

fn direct_request(
    admission: Option<ContextAdmissionAdapter>,
    token: Option<super::super::context_admission::LayoutIntentToken>,
) -> LayoutSwitchRequest {
    LayoutSwitchRequest {
        request_generation: 0,
        backend: LayoutBackend::Gnome,
        target_is_ru: true,
        engine: "lay-ime-ru",
        authority: LayoutSwitchAuthority::Direct { admission, token },
    }
}

#[test]
fn chooses_managed_ime_engine_label_for_target_layout() {
    assert_eq!(ime_engine_for_layout(true), "lay-ime-ru");
    assert_eq!(ime_engine_for_layout(false), "lay-ime-us");
}

#[test]
fn space_layout_stack_completes_stale_ibus_once_in_both_directions() {
    use super::{switch_complete_layout_stack, LayoutStackProbe, LAYOUT_STACK_PROBE};

    for (target_is_ru, observed_engine, expected_switches) in [
        (false, "lay-ime-ru", vec!["lay-ime-us"]),
        (true, "lay-ime-us", vec!["lay-ime-ru"]),
        (false, "xkb:us::eng", vec!["lay-ime-us"]),
        (false, "lay-ime-us", vec![]),
        (true, "lay-ime-ru", vec![]),
    ] {
        LAYOUT_STACK_PROBE.with(|probe| {
            *probe.borrow_mut() = Some(LayoutStackProbe {
                active_engine: Some(observed_engine.to_string()),
                ..Default::default()
            });
        });
        let target_engine = ime_engine_for_layout(target_is_ru);
        let result =
            switch_complete_layout_stack(LayoutBackend::Gnome, target_is_ru, target_engine, || {
                true
            });
        let observed = LAYOUT_STACK_PROBE.with(|probe| probe.borrow_mut().take().unwrap());

        assert!(
            result.is_ok(),
            "target={target_engine} observed={observed_engine}"
        );
        assert_eq!(observed.activations, [target_is_ru]);
        assert_eq!(observed.ibus_switches, expected_switches);
        assert_eq!(observed.active_engine.as_deref(), Some(target_engine));
    }
}

#[test]
fn kde_space_layout_stack_uses_its_captured_backend_in_both_directions() {
    use super::{LayoutStackProbe, LAYOUT_STACK_PROBE};
    for is_ru in [false, true] {
        LAYOUT_STACK_PROBE.with(|probe| {
            *probe.borrow_mut() = Some(LayoutStackProbe {
                active_engine: Some(ime_engine_for_layout(!is_ru).to_string()),
                ..Default::default()
            });
        });
        let mut request = request(None, None);
        request.backend = LayoutBackend::Kde;
        request.target_is_ru = is_ru;
        request.engine = ime_engine_for_layout(is_ru);
        let result = dispatch_layout_switch(&request, || true, || request.switch_stack(|| true));
        let observed = LAYOUT_STACK_PROBE.with(|probe| probe.borrow_mut().take().unwrap());
        assert_eq!(result, LayoutSwitchDispatch::Completed { switched: true });
        assert_eq!(observed.kde_activations, [is_ru]);
        assert!(observed.activations.is_empty(), "KDE must not invoke GNOME");
        assert_eq!(observed.ibus_switches, [request.engine]);
        assert_eq!(observed.active_engine.as_deref(), Some(request.engine));
    }
}

#[test]
fn kde_ibus_observation_cannot_overwrite_a_newer_completed_direct_request() {
    use super::{LayoutStackProbe, LAYOUT_STACK_PROBE};

    for old_is_ru in [false, true] {
        let worker = super::layout_switch_worker();
        let mut old = direct_request(None, None);
        old.request_generation = worker.reserve_generation();
        old.backend = LayoutBackend::Kde;
        old.target_is_ru = old_is_ru;
        old.engine = ime_engine_for_layout(old_is_ru);
        let new_engine = ime_engine_for_layout(!old_is_ru);
        LAYOUT_STACK_PROBE.with(|probe| {
            *probe.borrow_mut() = Some(LayoutStackProbe {
                active_engine: Some("xkb:us::eng".to_string()),
                after_engine_read: Some(Box::new(move || {
                    let newer = super::dispatch_direct_layout_switch(
                        LayoutBackend::Kde,
                        !old_is_ru,
                        new_engine,
                        None,
                        None,
                        |request| request.switch_stack(|| worker.request_is_current(request)),
                    );
                    assert_eq!(newer, LayoutSwitchDispatch::Completed { switched: true });
                })),
                ..Default::default()
            });
        });
        let result = dispatch_layout_switch(
            &old,
            || worker.request_is_current(&old),
            || old.switch_stack(|| worker.request_is_current(&old)),
        );
        let observed = LAYOUT_STACK_PROBE.with(|probe| probe.borrow_mut().take().unwrap());
        assert_eq!(
            result,
            LayoutSwitchDispatch::SupersededCompletion { switched: false }
        );
        assert_eq!(observed.active_engine.as_deref(), Some(new_engine));
        assert_eq!(observed.ibus_switches, [new_engine]);
        assert_eq!(observed.kde_activations, [old_is_ru, !old_is_ru]);
        assert!(observed.activations.is_empty());
    }
}

#[test]
fn superseded_layout_stack_does_not_activate_a_backend_or_complete_ibus() {
    use super::{LayoutStackProbe, LAYOUT_STACK_PROBE};
    for backend in [LayoutBackend::Gnome, LayoutBackend::Kde] {
        LAYOUT_STACK_PROBE.with(|probe| *probe.borrow_mut() = Some(LayoutStackProbe::default()));
        let mut request = request(None, None);
        request.backend = backend;
        assert!(request.switch_stack(|| false).is_err());
        let observed = LAYOUT_STACK_PROBE.with(|probe| probe.borrow_mut().take().unwrap());
        assert!(observed.activations.is_empty());
        assert!(observed.kde_activations.is_empty());
        assert!(observed.ibus_switches.is_empty());
    }
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
    assert!(engine.layout_gesture.layout_is_ru);
    engine.atomic.speculation = true;
    engine.sync_layout_after_committed_text("привет ", "owned_preedit_boundary");
    assert!(matches!(
        engine.atomic.deferred_layout_actions.last(),
        Some(super::super::engine::DeferredLayoutAction::Background {
            target_is_ru: true,
            ..
        })
    ));
    engine.atomic.speculation = false;

    engine.sync_layout_after_manual_toggle("hello ");
    assert!(!engine.layout_gesture.layout_is_ru);
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
    assert!(engine.layout_gesture.layout_is_ru);
    assert!(engine.toggle_layout_from_modifier_hotkey());
    assert!(!engine.layout_gesture.layout_is_ru);
}

#[test]
fn td121_space_boundary_after_enqueue_keeps_the_layout_request_live() {
    let (admission, grant, _peer) = ContextAdmissionAdapter::test_established(
        "/org/freedesktop/IBus/Engine/Lay/layout-space",
        "/org/freedesktop/IBus/InputContext/layout-space",
        WordCompleteness::KnownStart,
        4,
    );
    let layout_token = admission
        .current_layout_intent_token_for(&grant.target_owner)
        .expect("admitted layout token");
    let ordinary_token = admission.current_token().expect("ordinary token");
    let mut state = LayoutSwitchState::default();
    state.schedule(request(Some(admission.clone()), Some(layout_token)));
    let queued = state
        .desired
        .as_ref()
        .expect("queued layout request")
        .clone();

    let mut next_word = WordScope::new(grant.lineage);
    next_word.close_at_observed_boundary(5, None);
    assert!(admission.test_set_word_scope(&grant.target_owner, 5, &next_word));
    assert!(!admission.revalidate(&ordinary_token));

    assert_eq!(
        dispatch_layout_switch(&queued, || state.request_is_current(&queued), || Ok(())),
        LayoutSwitchDispatch::Completed { switched: true }
    );
}

#[test]
fn td121_superseded_layout_request_between_pop_and_dispatch_is_refused() {
    let mut state = LayoutSwitchState::default();
    state.schedule(request(None, None));
    let superseded = state.desired.take().expect("first request");
    state.schedule(request(None, None));
    let switches = Cell::new(0);

    assert_eq!(
        dispatch_layout_switch(
            &superseded,
            || state.request_is_current(&superseded),
            || {
                switches.set(switches.get() + 1);
                Ok(())
            }
        ),
        LayoutSwitchDispatch::Superseded
    );
    assert_eq!(switches.get(), 0);
}

#[test]
fn td121_explicit_layout_intent_supersedes_a_popped_auto_request_before_dispatch() {
    let mut state = LayoutSwitchState::default();
    state.schedule(request(None, None));
    let superseded = state.desired.take().expect("auto request");
    state.supersede_pending();
    let switches = Cell::new(0);

    assert_eq!(
        dispatch_layout_switch(
            &superseded,
            || state.request_is_current(&superseded),
            || {
                switches.set(switches.get() + 1);
                Ok(())
            }
        ),
        LayoutSwitchDispatch::Superseded
    );
    assert_eq!(switches.get(), 0);
}

#[test]
fn td121_owner_transfer_refuses_the_old_layout_request_before_dispatch() {
    let (admission, grant, _peer) = ContextAdmissionAdapter::test_established(
        "/org/freedesktop/IBus/Engine/Lay/layout-owner",
        "/org/freedesktop/IBus/InputContext/layout-owner",
        WordCompleteness::KnownStart,
        4,
    );
    let layout_token = admission
        .current_layout_intent_token_for(&grant.target_owner)
        .expect("admitted layout token");
    let request = request(Some(admission.clone()), Some(layout_token));
    assert!(admission.revoke_current_owner(&grant.target_owner));
    let switches = Cell::new(0);

    assert_eq!(
        dispatch_layout_switch(
            &request,
            || true,
            || {
                switches.set(switches.get() + 1);
                Ok(())
            }
        ),
        LayoutSwitchDispatch::StaleContext
    );
    assert_eq!(switches.get(), 0);
}

#[test]
fn td121_required_background_adapter_without_token_is_censored() {
    let (admission, _grant, _peer) = ContextAdmissionAdapter::test_established(
        "/org/freedesktop/IBus/Engine/Lay/layout-required-token",
        "/org/freedesktop/IBus/InputContext/layout-required-token",
        WordCompleteness::KnownStart,
        4,
    );
    let request = background_request(Some(admission), None, true);
    let switches = Cell::new(0);

    assert_eq!(
        dispatch_layout_switch(
            &request,
            || true,
            || {
                switches.set(switches.get() + 1);
                Ok(())
            }
        ),
        LayoutSwitchDispatch::StaleContext
    );
    assert_eq!(switches.get(), 0);
}

#[test]
fn td121_direct_modifier_route_allows_unknown_start_without_a_layout_token() {
    let (admission, _grant, _peer) = ContextAdmissionAdapter::test_established(
        "/org/freedesktop/IBus/Engine/Lay/layout-direct-unknown",
        "/org/freedesktop/IBus/InputContext/layout-direct-unknown",
        WordCompleteness::UnknownStart,
        4,
    );
    let request = direct_request(Some(admission), None);
    let switches = Cell::new(0);

    assert_eq!(
        dispatch_layout_switch(
            &request,
            || true,
            || {
                switches.set(switches.get() + 1);
                Ok(())
            }
        ),
        LayoutSwitchDispatch::Completed { switched: true }
    );
    assert_eq!(switches.get(), 1);
}

#[test]
fn td121_in_flight_layout_conflict_switches_once_and_leaves_no_text_authority() {
    let (admission, grant, _peer) = ContextAdmissionAdapter::test_established(
        "/org/freedesktop/IBus/Engine/Lay/layout-conflict",
        "/org/freedesktop/IBus/InputContext/layout-conflict",
        WordCompleteness::KnownStart,
        4,
    );
    let layout_token = admission
        .current_layout_intent_token_for(&grant.target_owner)
        .expect("admitted layout token");
    let request = request(Some(admission.clone()), Some(layout_token.clone()));
    let switches = Cell::new(0);

    assert_eq!(
        dispatch_layout_switch(
            &request,
            || true,
            || {
                switches.set(switches.get() + 1);
                assert!(admission.revoke_layout_intent(&layout_token));
                Ok(())
            }
        ),
        LayoutSwitchDispatch::ContextConflict { switched: true }
    );
    assert_eq!(switches.get(), 1);
    assert!(!admission.revalidate_layout_intent(&layout_token));
}

#[test]
fn td121_in_flight_superseded_completion_revokes_text_authority_without_switching_again() {
    let (admission, grant, _peer) = ContextAdmissionAdapter::test_established(
        "/org/freedesktop/IBus/Engine/Lay/layout-completion",
        "/org/freedesktop/IBus/InputContext/layout-completion",
        WordCompleteness::KnownStart,
        4,
    );
    let layout_token = admission
        .current_layout_intent_token_for(&grant.target_owner)
        .expect("admitted layout token");
    let state = RefCell::new(LayoutSwitchState::default());
    state
        .borrow_mut()
        .schedule(request(Some(admission.clone()), Some(layout_token.clone())));
    let in_flight = state
        .borrow()
        .desired
        .as_ref()
        .expect("queued layout request")
        .clone();
    let switches = Cell::new(0);

    let dispatch = dispatch_layout_switch(
        &in_flight,
        || state.borrow().request_is_current(&in_flight),
        || {
            switches.set(switches.get() + 1);
            state.borrow_mut().schedule(request(None, None));
            Ok(())
        },
    );
    assert_eq!(
        dispatch,
        LayoutSwitchDispatch::SupersededCompletion { switched: true }
    );
    assert!(!dispatch.promotes_decoder());
    assert_eq!(switches.get(), 1);
    assert!(!admission.revalidate_layout_intent(&layout_token));
}

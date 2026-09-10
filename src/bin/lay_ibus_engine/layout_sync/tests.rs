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

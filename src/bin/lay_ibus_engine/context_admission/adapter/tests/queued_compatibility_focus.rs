//! Delayed KDE compatibility callbacks must leave the successor field intact.
use super::*;
use crate::window_interaction::{LifecycleReceipt, WindowInteraction, WindowLifecycleEvent};

#[derive(Clone, Copy, Debug)]
enum Metadata {
    Retained,
    Revoked,
    Evicted,
}

#[derive(Clone, Copy, Debug)]
enum Successor {
    Pending,
    Ready,
    Installed,
}

async fn delayed_focus_preserves_successor(metadata: Metadata, successor: Successor) {
    let mut harness = bootstrap_harness_with_budget(CALLBACK_BUDGET).await;
    let mut engine = new_engine(&harness);
    engine.config.layout_backend = "kde".to_owned();
    let mut focuses = Vec::new();
    for (serial, member) in [
        (166_000, "FocusIn"),
        (166_001, "FocusOut"),
        (166_002, "FocusIn"),
    ] {
        let message = method_message(
            DISPATCH_SENDER,
            serial,
            TARGET_PATH,
            ENGINE_INTERFACE,
            member,
        );
        send_manually_dispatched_callback(
            &mut harness.peer,
            &message,
            TARGET_PATH,
            ENGINE_INTERFACE,
        )
        .await;
        assert!(harness.observer.process_next().await.unwrap());
        if member == "FocusIn" {
            focuses.push(message);
        }
    }
    let current = harness
        .adapter
        .observe_callback(&focuses[1].header(), Instant::now())
        .await
        .unwrap();
    harness
        .adapter
        .start_compatibility_activation_with_deadline(
            engine_path(TARGET_PATH),
            current.position,
            std::future::ready(Some(Instant::now() + CALLBACK_BUDGET)),
        )
        .unwrap();
    if !matches!(successor, Successor::Pending) {
        serve_current_context_and_marker(&mut harness.peer).await;
        assert!(harness.observer.process_next().await.unwrap());
        assert!(harness.adapter.current_token().is_some());
        if matches!(successor, Successor::Installed) {
            engine.try_install_pending_context_activation();
            assert!(engine.live_context_token().is_some());
        }
    }
    match metadata {
        Metadata::Retained => {}
        Metadata::Revoked => harness.adapter.shared.stamps.revoke(),
        Metadata::Evicted => {
            // Exercise the actual bounded store through inert observer packets,
            // without revoking the current field or manufacturing an owner.
            for serial in 166_100..166_229 {
                let message = method_message(
                    DISPATCH_SENDER,
                    serial,
                    SOURCE_PATH,
                    ENGINE_INTERFACE,
                    "Enable",
                );
                send_manually_dispatched_callback(
                    &mut harness.peer,
                    &message,
                    SOURCE_PATH,
                    ENGINE_INTERFACE,
                )
                .await;
                assert!(harness.observer.process_next().await.unwrap());
            }
        }
    }
    let retained = harness
        .adapter
        .observe_callback(&focuses[0].header(), Instant::now())
        .await
        .is_ok();
    assert_eq!(retained, matches!(metadata, Metadata::Retained));

    // Non-authoritative local display/work witnesses expose the outer callback's
    // destructive cleanup. All ownership above comes from real Get + marker.
    engine.atomic.active = true;
    engine.composition.preedit_dirty = true;
    let snapshot = SurroundingTextSnapshot::new("current".to_owned(), 7, 7);
    engine.client_context.surrounding_text_snapshot = Some(snapshot.clone());
    let engine_owner = engine.context_owner.clone();
    let engine_token = engine.context_token.clone();
    let owner = harness.adapter.current_owner();
    let token = harness.adapter.current_token();
    let request = harness
        .adapter
        .shared
        .reducer
        .lock()
        .unwrap()
        .request
        .as_ref()
        .map(|request| request.generation);
    let pending = || {
        harness
            .adapter
            .shared
            .pending
            .lock()
            .unwrap()
            .as_ref()
            .map(|fence| {
                (
                    fence.nonce,
                    fence.deadline,
                    fence.ready_token.clone(),
                    fence.marker_observed,
                    fence.ready,
                )
            })
    };
    let ready = || {
        harness
            .adapter
            .shared
            .ready_activation
            .lock()
            .unwrap()
            .as_ref()
            .map(|ready| {
                (
                    ready.request,
                    ready.fence,
                    ready.target_path.clone(),
                    ready.owner.clone(),
                    ready.outcome.clone(),
                )
            })
    };
    let pending_before = pending();
    let ready_before = ready();

    assert_eq!(
        WindowInteraction::observe_lifecycle(
            &mut engine,
            WindowLifecycleEvent::FocusIn {
                header: &focuses[0].header()
            },
            None,
        )
        .await
        .unwrap(),
        LifecycleReceipt::Focused { changed: false }
    );
    assert_eq!(
        harness.adapter.current_owner(),
        owner,
        "{metadata:?}/{successor:?}: stale callback revoked the successor"
    );
    assert_eq!(harness.adapter.current_token(), token);
    assert_eq!(engine.context_owner, engine_owner);
    assert_eq!(engine.context_token, engine_token);
    assert_eq!(
        harness
            .adapter
            .shared
            .reducer
            .lock()
            .unwrap()
            .request
            .as_ref()
            .map(|request| request.generation),
        request
    );
    assert_eq!(pending(), pending_before);
    assert_eq!(ready(), ready_before);
    assert!(
        engine.atomic.active,
        "stale callback must not cancel successor work"
    );
    assert!(engine.composition.preedit_dirty);
    assert_eq!(
        engine.client_context.surrounding_text_snapshot,
        Some(snapshot)
    );
    assert_eq!(engine.config.layout_backend, "kde");
    if let Some(token) = token {
        assert!(harness.adapter.revalidate(&token));
    }
}

fn all_successors(metadata: Metadata) {
    for successor in [Successor::Pending, Successor::Ready, Successor::Installed] {
        zbus::block_on(bounded(delayed_focus_preserves_successor(
            metadata, successor,
        )));
    }
}

#[test]
fn retained_queued_compatibility_lifecycle_preserves_successor() {
    all_successors(Metadata::Retained);
}

#[test]
fn revoked_queued_compatibility_lifecycle_preserves_successor() {
    all_successors(Metadata::Revoked);
}

#[test]
fn evicted_queued_compatibility_lifecycle_preserves_successor() {
    all_successors(Metadata::Evicted);
}

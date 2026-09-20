use std::future::Future;
use std::pin::Pin;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll, Waker};

use ordered_stream::{OrderedStream, PollResult};

use super::rendezvous::RendezvousBegin;
use super::*;

fn engine_path(value: &str) -> EnginePath {
    EnginePath::new(value).expect("bounded engine path")
}

fn profile(value: &str) -> EngineProfile {
    EngineProfile::new(value).expect("bounded profile")
}

fn context(epoch: ConnectionGeneration, value: &str) -> ContextKey {
    ContextKey::new(epoch, value).expect("bounded context")
}

fn header(epoch: ConnectionGeneration, serial: u32) -> HeaderKey {
    HeaderKey::new(
        epoch,
        ":1.42",
        serial,
        "/org/freedesktop/IBus/Engine/Lay/0",
        "ProcessKeyEvent",
    )
    .expect("valid header")
}

#[test]
fn bridge_output_settlement_preserves_lineage_and_rejects_interposed_work() {
    let mut reducer: ContextAdmissionReducer<u64> = ContextAdmissionReducer::new(
        ConnectionGeneration(11),
        GlobalEngineMode::Verified,
        GlobalProfile::Lay(profile("lay-us")),
    );
    let owner = reducer
        .establish_source(
            engine_path("/engine/source"),
            context(ConnectionGeneration(11), "/context"),
            WordCompleteness::KnownStart,
            7,
        )
        .unwrap();
    let token = reducer.bridge_admission_token().unwrap();
    let settled = SettledWordState {
        tail_epoch: 8,
        lineage: token.lineage,
        content_type: None,
    };
    assert!(reducer.settle_bridge_output(&token, settled));
    assert!(
        !reducer.settle_bridge_output(&token, settled),
        "duplicate epoch is not another output"
    );
    let key = header(ConnectionGeneration(11), 8_100);
    assert!(reducer.observe_key_start(&owner, key.clone(), KeyWordEffect::PossibleMutation));
    assert!(!reducer.settle_bridge_output(
        &token,
        SettledWordState {
            tail_epoch: 9,
            ..settled
        }
    ));
    assert_eq!(reducer.latest_tail_epoch, 8);
    assert!(reducer.settle_key(&owner, &key, settled));
    let mut forged = settled;
    forged.tail_epoch = 9;
    forged.lineage.generation = LineageGeneration(999);
    assert!(!reducer.settle_bridge_output(&token, forged));
    open_bound_factory(
        &mut reducer,
        engine_path("/engine/target"),
        profile("lay-us"),
        10,
    );
    assert!(!reducer.settle_bridge_output(
        &token,
        SettledWordState {
            tail_epoch: 9,
            ..settled
        }
    ));
    reducer.malformed_or_unobserved_lifecycle();
    assert!(!reducer.settle_bridge_output(
        &token,
        SettledWordState {
            tail_epoch: 9,
            ..settled
        }
    ));
    assert_eq!(reducer.latest_tail_epoch, 8);
    assert_eq!(
        reducer.lineage().completeness,
        WordCompleteness::UnknownStart
    );
}

#[test]
fn stale_foreign_snapshot_publish_cannot_clear_the_current_owner_receipt() {
    let mut reducer: ContextAdmissionReducer<u64> = ContextAdmissionReducer::new(
        ConnectionGeneration(11),
        GlobalEngineMode::Verified,
        GlobalProfile::Lay(profile("lay-us")),
    );
    reducer
        .establish_source(
            engine_path("/engine/source"),
            context(ConnectionGeneration(11), "/context"),
            WordCompleteness::UnknownStart,
            7,
        )
        .unwrap();
    let token = reducer.bridge_admission_token().unwrap();
    assert!(reducer.publish_exact_manual_snapshot(
        &token,
        7,
        "abc".into(),
        3,
        Instant::now() + Duration::from_secs(1),
    ));
    let mut stale_foreign = token.clone();
    stale_foreign.owner.path = engine_path("/engine/stale-foreign");
    assert!(!reducer.publish_exact_manual_snapshot(
        &stale_foreign,
        7,
        "foreign".into(),
        4,
        Instant::now() + Duration::from_secs(1),
    ));
    assert_eq!(
        reducer
            .exact_manual_snapshot
            .as_ref()
            .map(|receipt| receipt.tail.as_str()),
        Some("abc")
    );
}

#[test]
fn repeated_content_type_requires_settled_provenance_and_exact_target() {
    let mut reducer: ContextAdmissionReducer<u64> = ContextAdmissionReducer::new(
        ConnectionGeneration(11),
        GlobalEngineMode::Verified,
        GlobalProfile::Lay(profile("lay-us")),
    );
    let source = engine_path("/engine/source");
    let owner = reducer
        .establish_source(
            source.clone(),
            context(ConnectionGeneration(11), "/context"),
            WordCompleteness::KnownStart,
            7,
        )
        .unwrap();
    assert!(reducer
        .unchanged_content_type_token(&source, (10, 0))
        .is_none());
    let key = header(ConnectionGeneration(11), 8_110);
    assert!(reducer.observe_key_start(&owner, key.clone(), KeyWordEffect::PossibleMutation));
    assert!(reducer.settle_key(
        &owner,
        &key,
        SettledWordState {
            tail_epoch: 8,
            lineage: reducer.lineage(),
            content_type: Some((10, 0)),
        }
    ));
    let token = reducer
        .unchanged_content_type_token(&source, (10, 0))
        .unwrap();
    for value in [(0, 0), (10, 1)] {
        assert!(reducer
            .unchanged_content_type_token(&source, value)
            .is_none());
    }
    let target = engine_path("/engine/target");
    assert!(reducer
        .unchanged_content_type_token(&target, (10, 0))
        .is_none());
    open_bound_factory(&mut reducer, target.clone(), profile("lay-us"), 10);
    assert!(reducer
        .unchanged_content_type_token(&target, (10, 0))
        .is_some());
    assert!(
        reducer.consume().is_none(),
        "matching metadata grants no transfer"
    );
    assert!(reducer
        .unchanged_content_type_token(&engine_path("/engine/foreign"), (10, 0))
        .is_none());
    reducer.malformed_or_unobserved_lifecycle();
    assert!(!reducer.revalidate_content_type(&token, &target, (10, 0)));
    assert!(reducer
        .unchanged_content_type_token(&source, (10, 0))
        .is_none());
}

fn open_bound_factory<P>(
    reducer: &mut ContextAdmissionReducer<P>,
    target: EnginePath,
    expected_profile: EngineProfile,
    position: P,
) -> TicketId
where
    P: Clone + Ord,
{
    let ticket = reducer
        .open_factory_request(expected_profile, position)
        .expect("factory request");
    assert!(reducer.bind_factory_target(ticket, target));
    ticket
}

struct ReducerFixture<P>
where
    P: Clone + Ord,
{
    reducer: ContextAdmissionReducer<P>,
    source_owner: EngineOwner,
    canonical_context: ContextKey,
    nonce: BarrierNonce,
    request: RequestGeneration,
    source_tail: String,
    target_tail: String,
    output_edits: usize,
    ordinary_guard_transfers: usize,
    feedback_events: usize,
}

impl<P> ReducerFixture<P>
where
    P: Clone + Ord,
{
    fn apply_grant(&mut self, grant: TransferGrant) {
        assert_eq!(grant.source_tail_epoch, 7);
        assert_eq!(grant.lineage.completeness, WordCompleteness::KnownStart);
        assert!(grant.invalidate_prior_authority);
        assert!(self.target_tail.is_empty(), "grant applied more than once");
        self.target_tail.clone_from(&self.source_tail);
        // Metadata transfer is not a client edit, feedback event or TD-120 guard.
        assert_eq!(self.output_edits, 0);
        assert_eq!(self.ordinary_guard_transfers, 0);
        assert_eq!(self.feedback_events, 0);
    }
}

fn pending_fixture() -> ReducerFixture<u64> {
    let epoch = ConnectionGeneration(11);
    let canonical_context = context(epoch, "/org/freedesktop/IBus/InputContext_7");
    let lay_profile = profile("lay-us");
    let mut reducer = ContextAdmissionReducer::new(
        epoch,
        GlobalEngineMode::Verified,
        GlobalProfile::Lay(lay_profile.clone()),
    );
    let source_owner = reducer
        .establish_source(
            engine_path("/org/freedesktop/IBus/Engine/Lay/0"),
            canonical_context.clone(),
            WordCompleteness::KnownStart,
            7,
        )
        .expect("source established");
    open_bound_factory(
        &mut reducer,
        engine_path("/org/freedesktop/IBus/Engine/Lay/1"),
        lay_profile,
        10,
    );
    reducer.focus_out(&source_owner, 20).expect("focus out");
    assert!(reducer.disable(&source_owner, 25));
    assert!(reducer.seal_source(&source_owner, 7, 15));
    let nonce = BarrierNonce(91);
    let request = reducer
        .focus_in(
            &engine_path("/org/freedesktop/IBus/Engine/Lay/1"),
            nonce,
            ReceiptOrigin::CompatibilityProperty,
            40,
        )
        .expect("target focus");
    assert!(reducer.context_reply(request, nonce, canonical_context.clone(), 50,));
    assert_eq!(reducer.status(), AdmissionStatus::Pending);
    assert!(reducer.consume().is_none(), "Get reply alone cannot admit");

    ReducerFixture {
        reducer,
        source_owner,
        canonical_context,
        nonce,
        request,
        source_tail: "l".to_string(),
        target_tail: String::new(),
        output_edits: 0,
        ordinary_guard_transfers: 0,
        feedback_events: 0,
    }
}

#[test]
fn p121_1_d1_held_fifo_rejects_foreign_aba_and_get_alone() {
    let mut fixture = pending_fixture();

    // The test FIFO is deliberately held: Get(A) has already arrived, while
    // these two signals have not yet been presented to the actual reducer.
    let mut held_fifo = vec![
        GlobalProfile::Foreign(profile("foreign-ime")),
        GlobalProfile::Lay(profile("lay-us")),
    ];
    assert_eq!(fixture.reducer.status(), AdmissionStatus::Pending);
    assert!(fixture.reducer.consume().is_none());

    let before_aba = fixture.reducer.revocation_generation();
    for event in held_fifo.drain(..) {
        fixture.reducer.global_engine_changed(event);
    }
    assert!(fixture.reducer.revocation_generation() > before_aba);
    assert!(!fixture.reducer.marker(fixture.request, fixture.nonce, 60));
    assert_eq!(fixture.reducer.status(), AdmissionStatus::Revoked);
    assert!(fixture.reducer.consume().is_none());
    assert!(fixture.target_tail.is_empty());
    assert_eq!(fixture.output_edits, 0);
    assert_eq!(fixture.ordinary_guard_transfers, 0);
    assert_eq!(fixture.feedback_events, 0);
    assert_eq!(
        fixture.reducer.lineage().completeness,
        WordCompleteness::UnknownStart
    );
}

#[test]
fn abandoned_key_requires_exact_owner_and_header_and_invalidates_sealed_transfer() {
    let mut fixture = pending_fixture();
    let key = header(fixture.reducer.connection, 901);
    let owner = fixture.source_owner.clone();
    let token = fixture.reducer.admission_token().unwrap();
    assert!(fixture.reducer.observe_key_start(
        &owner,
        key.clone(),
        KeyWordEffect::PossibleMutation
    ));
    let other_owner = EngineOwner {
        generation: OwnerGeneration(owner.generation.0 + 1),
        ..owner.clone()
    };
    assert!(!fixture.reducer.abandon_key(&other_owner, &key));
    assert!(!fixture
        .reducer
        .abandon_key(&owner, &header(key.connection, 902)));
    assert!(fixture.reducer.revalidate(&token));
    assert_eq!(fixture.reducer.unsettled.len(), 1);
    assert!(fixture.reducer.abandon_key(&owner, &key));
    assert!(fixture.reducer.unsettled.is_empty());
    assert!(!fixture.reducer.revalidate(&token));
    assert_eq!(
        fixture.reducer.lineage.completeness,
        WordCompleteness::UnknownStart
    );
    assert!(
        fixture.reducer.consume().is_none(),
        "abandoned input cannot transfer a sealed old tail"
    );
    assert!(
        !fixture.reducer.abandon_key(&owner, &key),
        "completion is exact and single-use"
    );
}

#[test]
fn genuinely_unsettled_key_overflow_remains_fail_closed() {
    let mut fixture = pending_fixture();
    let owner = fixture.source_owner.clone();
    let token = fixture.reducer.admission_token().unwrap();
    for serial in 1..=MAX_UNSETTLED_KEYS {
        assert!(fixture.reducer.observe_key_start(
            &owner,
            header(token.connection, serial as u32),
            KeyWordEffect::PossibleMutation
        ));
    }
    assert!(!fixture.reducer.observe_key_start(
        &owner,
        header(token.connection, MAX_UNSETTLED_KEYS as u32 + 1),
        KeyWordEffect::PossibleMutation
    ));
    assert!(!fixture.reducer.revalidate(&token));
    assert!(fixture.reducer.consume().is_none());
}

#[test]
fn pending_transfer_native_receipt_enriches_without_rearm_or_losing_ordered_marker() {
    for marker_before_native in [false, true] {
        let mut fixture = pending_fixture();
        let target = engine_path("/org/freedesktop/IBus/Engine/Lay/1");
        let revision = fixture.reducer.revocation_generation();
        if marker_before_native {
            fixture.reducer.ticket.as_mut().unwrap().source_seal = None;
            assert!(!fixture.reducer.marker(fixture.request, fixture.nonce, 60));
        }
        assert!(fixture.reducer.enrich_pending_native_activation(
            &target,
            fixture.canonical_context.clone(),
            65,
        ));
        let request = fixture.reducer.request.as_ref().unwrap();
        assert_eq!(request.generation, fixture.request);
        assert_eq!(request.nonce, fixture.nonce);
        assert_eq!(request.reply.as_ref().unwrap().1, 50);
        assert_eq!(request.origin, ReceiptOrigin::Native);
        assert_eq!(fixture.reducer.revocation_generation(), revision);
        if marker_before_native {
            assert!(fixture.reducer.seal_source(&fixture.source_owner, 7, 15));
        } else {
            assert!(fixture.reducer.marker(fixture.request, fixture.nonce, 70));
        }
        let grant = fixture
            .reducer
            .consume()
            .expect("same request retains its transfer");
        fixture.apply_grant(grant);
        assert_eq!(fixture.target_tail, "l");
        assert!(fixture.reducer.consume().is_none());
    }
}

#[test]
fn pending_native_enrichment_keeps_target_and_context_refusal_boundaries() {
    let mut fixture = pending_fixture();
    let target = engine_path("/org/freedesktop/IBus/Engine/Lay/1");
    let revision = fixture.reducer.revocation_generation();
    let other_context = context(
        ConnectionGeneration(11),
        "/org/freedesktop/IBus/InputContext_other",
    );
    assert!(!fixture.reducer.enrich_pending_native_activation(
        &engine_path("/org/freedesktop/IBus/Engine/Lay/unrelated"),
        other_context.clone(),
        55,
    ));
    assert_eq!(fixture.reducer.revocation_generation(), revision);
    assert_eq!(
        fixture.reducer.request.as_ref().unwrap().generation,
        fixture.request
    );
    assert!(!fixture
        .reducer
        .enrich_pending_native_activation(&target, other_context, 55));
    assert!(fixture.reducer.revocation_generation() > revision);
    assert!(fixture.reducer.consume().is_none());
    assert_eq!(
        fixture.reducer.lineage().completeness,
        WordCompleteness::UnknownStart
    );
}

#[test]
fn native_receipt_before_get_preserves_same_field_or_revokes_cross_field_transfer() {
    for same_field in [false, true] {
        let mut fixture = pending_fixture();
        // Hold the compatibility Get response while its exact request remains current.
        fixture.reducer.request.as_mut().unwrap().reply = None;
        let target = engine_path("/org/freedesktop/IBus/Engine/Lay/1");
        let native_context = if same_field {
            fixture.canonical_context.clone()
        } else {
            context(
                ConnectionGeneration(11),
                "/org/freedesktop/IBus/InputContext_other",
            )
        };
        assert!(fixture
            .reducer
            .enrich_pending_native_activation(&target, native_context, 55));
        assert_eq!(
            fixture.reducer.request.as_ref().unwrap().generation,
            fixture.request
        );
        assert!(fixture.reducer.marker(fixture.request, fixture.nonce, 60));
        if same_field {
            let grant = fixture.reducer.consume().unwrap();
            fixture.apply_grant(grant);
            assert_eq!(fixture.target_tail, "l");
        } else {
            assert!(fixture.reducer.consume().is_none());
            let grant = fixture.reducer.consume_source_free_activation().unwrap();
            assert_eq!(grant.lineage.completeness, WordCompleteness::UnknownStart);
            assert!(fixture.target_tail.is_empty());
            assert!(fixture.reducer.consume_source_free_activation().is_none());
        }
        assert_eq!(fixture.output_edits, 0);
        assert_eq!(fixture.feedback_events, 0);
    }
}

#[test]
fn next_factory_supersedes_unadmitted_source_free_without_borrowing_authority() {
    let epoch = ConnectionGeneration(111);
    let lay_profile = profile("lay-us");
    let mut reducer = ContextAdmissionReducer::new(
        epoch,
        GlobalEngineMode::Verified,
        GlobalProfile::Lay(lay_profile.clone()),
    );
    let old_target = engine_path("/org/freedesktop/IBus/Engine/Lay/old");
    let target = engine_path("/org/freedesktop/IBus/Engine/Lay/new");
    let old_nonce = BarrierNonce(10);
    let old_request = reducer
        .begin_source_free_activation(
            old_target,
            old_nonce,
            ReceiptOrigin::CompatibilityProperty,
            10,
        )
        .unwrap();
    let factory = reducer
        .open_factory_request(lay_profile, 20)
        .expect("later factory replaces unadmitted empty acquisition");
    assert!(reducer.bind_factory_target(factory, target.clone()));
    let nonce = BarrierNonce(11);
    let request = reducer
        .begin_source_free_activation(target, nonce, ReceiptOrigin::Native, 30)
        .unwrap();
    let revision = reducer.revocation_generation();
    assert!(!reducer.context_reply(
        old_request,
        old_nonce,
        context(epoch, "/org/freedesktop/IBus/InputContext_old"),
        31
    ));
    assert!(!reducer.context_acquisition_failed(old_request));
    assert_eq!(reducer.revocation_generation(), revision);
    assert_eq!(reducer.request.as_ref().unwrap().generation, request);
    assert!(reducer.context_reply(
        request,
        nonce,
        context(epoch, "/org/freedesktop/IBus/InputContext_new"),
        32
    ));
    assert!(reducer.marker(request, nonce, 40));
    assert!(reducer.consume().is_none());
    let grant = reducer.consume_source_free_activation().unwrap();
    assert_eq!(grant.lineage.completeness, WordCompleteness::UnknownStart);
    assert!(reducer.consume_source_free_activation().is_none());
}

#[test]
fn revoked_transfer_next_factory_reacquires_unknown_without_old_authority() {
    for was_ready in [false, true] {
        let mut fixture = pending_fixture();
        let old_token = fixture.reducer.admission_token().unwrap();
        let old_ticket = fixture.reducer.ticket.as_ref().unwrap().id;
        if was_ready {
            assert!(fixture.reducer.marker(fixture.request, fixture.nonce, 60));
        }
        assert!(fixture.reducer.context_acquisition_failed(fixture.request));
        assert_eq!(fixture.reducer.status(), AdmissionStatus::Revoked);
        let target = engine_path("/org/freedesktop/IBus/Engine/Lay/recovered");
        let factory = fixture
            .reducer
            .open_factory_request(profile("lay-us"), 80)
            .expect("later factory retires a revoked transfer, not the observer");
        assert_ne!(factory, old_ticket);
        assert!(fixture.reducer.owner().is_none());
        assert!(fixture.reducer.activation().is_none());
        assert!(fixture.reducer.ticket.is_none());
        assert!(fixture.reducer.consume().is_none());
        assert!(!fixture.reducer.revalidate(&old_token));
        let revision = fixture.reducer.revocation_generation();
        assert!(!fixture
            .reducer
            .bind_factory_target(old_ticket, target.clone()));
        assert_eq!(fixture.reducer.revocation_generation(), revision);
        assert!(fixture.reducer.bind_factory_target(factory, target.clone()));
        let nonce = BarrierNonce(92);
        let request = fixture
            .reducer
            .begin_source_free_activation(target, nonce, ReceiptOrigin::Native, 90)
            .unwrap();
        let revision = fixture.reducer.revocation_generation();
        assert!(!fixture.reducer.context_reply(
            fixture.request,
            fixture.nonce,
            fixture.canonical_context.clone(),
            91,
        ));
        assert!(!fixture.reducer.marker(fixture.request, fixture.nonce, 92));
        assert!(!fixture.reducer.context_acquisition_failed(fixture.request));
        assert_eq!(fixture.reducer.revocation_generation(), revision);
        assert!(fixture.reducer.context_reply(
            request,
            nonce,
            fixture.canonical_context.clone(),
            93,
        ));
        assert!(fixture.reducer.marker(request, nonce, 94));
        assert!(fixture.reducer.consume().is_none());
        let grant = fixture.reducer.consume_source_free_activation().unwrap();
        assert_eq!(grant.lineage.completeness, WordCompleteness::UnknownStart);
        assert_ne!(grant.target_owner.generation, old_token.owner.generation);
        assert_ne!(
            grant.target_activation.generation,
            old_token.activation.generation
        );
        assert!(!fixture.reducer.revalidate(&old_token));
        assert!(fixture.target_tail.is_empty());
        assert_eq!(fixture.output_edits, 0);
        assert_eq!(fixture.feedback_events, 0);
    }
}

#[test]
fn next_factory_preserves_live_overlap_rejection_and_consumed_transfer() {
    for status in [
        AdmissionStatus::Pending,
        AdmissionStatus::Ready,
        AdmissionStatus::Consumed,
    ] {
        let mut fixture = pending_fixture();
        if status != AdmissionStatus::Pending {
            assert!(fixture.reducer.marker(fixture.request, fixture.nonce, 60));
        }
        if status == AdmissionStatus::Consumed {
            assert!(fixture.reducer.consume().is_some());
        }
        assert_eq!(fixture.reducer.status(), status);
        let owner = fixture.reducer.owner().cloned().unwrap();
        let next = fixture.reducer.open_factory_request(profile("lay-us"), 80);
        if status == AdmissionStatus::Consumed {
            assert!(next.is_some());
            assert_eq!(fixture.reducer.ticket.as_ref().unwrap().source_owner, owner);
            assert!(fixture.reducer.source_free_factory.is_none());
        } else {
            assert!(next.is_none());
            assert_eq!(fixture.reducer.status(), AdmissionStatus::Revoked);
        }
    }
}

#[test]
fn revoked_transfer_rejects_factory_before_retained_lifecycle_positions() {
    for position in [0, 10, 20, 25, 40] {
        let mut fixture = pending_fixture();
        assert!(fixture.reducer.context_acquisition_failed(fixture.request));
        assert!(fixture
            .reducer
            .open_factory_request(profile("lay-us"), position)
            .is_none());
        assert_eq!(fixture.reducer.status(), AdmissionStatus::Revoked);
        assert!(fixture.reducer.source_free_factory.is_none());
        assert!(fixture.reducer.consume().is_none());
    }
}

#[test]
fn p121_1_d1_no_foreign_positive_consumes_l_once_without_output() {
    let mut fixture = pending_fixture();
    assert!(fixture.reducer.marker(fixture.request, fixture.nonce, 60));
    assert_eq!(fixture.reducer.status(), AdmissionStatus::Ready);
    let grant = fixture.reducer.consume().expect("single transfer grant");
    assert_eq!(grant.source_owner, fixture.source_owner);
    assert_ne!(grant.source_owner.generation, grant.target_owner.generation);
    assert_ne!(grant.target_activation.generation, ActivationGeneration(1));
    assert_eq!(grant.target_activation.context, fixture.canonical_context);
    assert_eq!(grant.receipt_origin, ReceiptOrigin::CompatibilityProperty);
    fixture.apply_grant(grant);
    assert_eq!(fixture.target_tail, "l");
    assert!(fixture.reducer.consume().is_none());
    assert_eq!(fixture.reducer.status(), AdmissionStatus::Consumed);
    assert_eq!(fixture.output_edits, 0);
}

#[test]
fn p121_1_consumption_preserves_the_sealed_word_lineage() {
    let mut fixture = pending_fixture();
    let sealed_lineage = fixture.reducer.lineage();
    assert!(fixture.reducer.marker(fixture.request, fixture.nonce, 60));
    let grant = fixture.reducer.consume().expect("single transfer grant");
    assert_eq!(grant.lineage, sealed_lineage);
    assert_eq!(fixture.reducer.lineage(), sealed_lineage);
}

#[test]
fn p121_1_native_focus_payload_may_share_sequence_but_compatibility_reply_must_follow() {
    fn reducer_at_equal_focus_and_reply(
        origin: ReceiptOrigin,
    ) -> (
        ContextAdmissionReducer<u64>,
        RequestGeneration,
        BarrierNonce,
    ) {
        let epoch = ConnectionGeneration(111);
        let canonical_context = context(epoch, "/org/freedesktop/IBus/InputContext_equal");
        let lay_profile = profile("lay-us");
        let target = engine_path("/org/freedesktop/IBus/Engine/Lay/1");
        let mut reducer = ContextAdmissionReducer::new(
            epoch,
            GlobalEngineMode::Verified,
            GlobalProfile::Lay(lay_profile.clone()),
        );
        let source = reducer
            .establish_source(
                engine_path("/org/freedesktop/IBus/Engine/Lay/0"),
                canonical_context.clone(),
                WordCompleteness::KnownStart,
                7,
            )
            .unwrap();
        open_bound_factory(&mut reducer, target.clone(), lay_profile, 10);
        reducer.focus_out(&source, 20).unwrap();
        assert!(reducer.disable(&source, 25));
        assert!(reducer.seal_source(&source, 7, 15));
        let nonce = BarrierNonce(191);
        let request = reducer.focus_in(&target, nonce, origin, 40).unwrap();
        assert!(reducer.context_reply(request, nonce, canonical_context, 40));
        (reducer, request, nonce)
    }

    let (mut native, native_request, native_nonce) =
        reducer_at_equal_focus_and_reply(ReceiptOrigin::Native);
    assert!(native.marker(native_request, native_nonce, 50));
    assert_eq!(native.status(), AdmissionStatus::Ready);
    assert!(native.consume().is_some());

    let (mut compatibility, compatibility_request, compatibility_nonce) =
        reducer_at_equal_focus_and_reply(ReceiptOrigin::CompatibilityProperty);
    assert!(!compatibility.marker(compatibility_request, compatibility_nonce, 50));
    assert_eq!(compatibility.status(), AdmissionStatus::Pending);
    assert!(compatibility.consume().is_none());
}

#[test]
fn p121_1_wrong_nonce_marker_does_not_settle_current_request() {
    let mut fixture = pending_fixture();
    assert!(!fixture
        .reducer
        .marker(fixture.request, BarrierNonce(92), 60));
    assert_eq!(fixture.reducer.status(), AdmissionStatus::Pending);
    assert!(fixture.reducer.consume().is_none());
    assert!(fixture.reducer.marker(fixture.request, fixture.nonce, 61));
}

#[test]
fn source_key_must_settle_before_source_seal() {
    let epoch = ConnectionGeneration(12);
    let canonical_context = context(epoch, "/org/freedesktop/IBus/InputContext_8");
    let lay_profile = profile("lay-us");
    let mut reducer = ContextAdmissionReducer::new(
        epoch,
        GlobalEngineMode::Verified,
        GlobalProfile::Lay(lay_profile.clone()),
    );
    let owner = reducer
        .establish_source(
            engine_path("/org/freedesktop/IBus/Engine/Lay/0"),
            canonical_context.clone(),
            WordCompleteness::KnownStart,
            6,
        )
        .unwrap();
    let key = header(epoch, 31);
    assert!(reducer.observe_key_start(&owner, key.clone(), KeyWordEffect::PossibleMutation));
    open_bound_factory(
        &mut reducer,
        engine_path("/org/freedesktop/IBus/Engine/Lay/1"),
        lay_profile,
        10,
    );
    reducer.focus_out(&owner, 20).unwrap();
    assert!(reducer.disable(&owner, 25));
    assert!(!reducer.seal_source(&owner, 6, 15));
    let nonce = BarrierNonce(1);
    let request = reducer
        .focus_in(
            &engine_path("/org/freedesktop/IBus/Engine/Lay/1"),
            nonce,
            ReceiptOrigin::Native,
            40,
        )
        .unwrap();
    assert!(reducer.context_reply(request, nonce, canonical_context, 50));
    assert!(!reducer.marker(request, nonce, 60));
    assert_eq!(reducer.status(), AdmissionStatus::Pending);
    let settled_scope = WordScope::new(reducer.lineage());
    assert!(reducer.settle_key(
        &owner,
        &key,
        SettledWordState::from_scope(7, &settled_scope),
    ));
    assert!(reducer.seal_source(&owner, 7, 15));
    assert_eq!(reducer.status(), AdmissionStatus::Ready);
    assert_eq!(reducer.consume().unwrap().source_tail_epoch, 7);
}

#[test]
fn unknown_start_is_sticky_until_next_observed_boundary() {
    let mut scope = WordScope::new(WordLineage {
        generation: LineageGeneration(7),
        completeness: WordCompleteness::UnknownStart,
        observed_suffix_chars: 0,
        observed_boundary_floor: None,
    });
    scope.keep_or_revoke_unknown();
    assert_eq!(scope.lineage().completeness, WordCompleteness::UnknownStart);
    assert_eq!(
        scope.close_at_observed_boundary(44, None),
        WordCompleteness::UnknownStart,
        "the just-closed partial word must remain ineligible"
    );
    assert_eq!(scope.lineage().completeness, WordCompleteness::KnownStart);
    scope.backspace_crossed_boundary(44);
    assert_eq!(scope.lineage().completeness, WordCompleteness::UnknownStart);
}

#[test]
fn p121_3_sender_roles_and_space_budget_are_exact() {
    let epoch = ConnectionGeneration(13);
    let bindings = SenderBindings::new(":1.0", ":1.1", "org.freedesktop.DBus").unwrap();
    let ibus = HeaderKey::new(
        epoch,
        ":1.0",
        1,
        "/org/freedesktop/IBus",
        "GlobalEngineChanged",
    )
    .unwrap();
    let marker = HeaderKey::new(epoch, ":1.1", 2, "/org/lay/Admission", "Barrier").unwrap();
    let callback = header(epoch, 3);
    assert!(bindings.accepts(HeaderRole::IbusLifecycle, &ibus));
    assert!(!bindings.accepts(HeaderRole::EngineCallback, &ibus));
    assert!(!bindings.accepts(HeaderRole::PrivateMarker, &ibus));
    assert!(bindings.accepts(HeaderRole::PrivateMarker, &marker));
    assert!(!bindings.accepts(HeaderRole::IbusLifecycle, &marker));
    let rewritten_foreign_marker =
        HeaderKey::new(epoch, ":1.2", 2, "/org/lay/Admission", "Barrier").unwrap();
    assert!(!bindings.accepts(HeaderRole::PrivateMarker, &rewritten_foreign_marker));
    assert!(!bindings.accepts(HeaderRole::EngineCallback, &callback));
    let dispatched = HeaderKey::new(
        epoch,
        "org.freedesktop.DBus",
        4,
        "/org/freedesktop/IBus/Engine/Lay/0",
        "ProcessKeyEvent",
    )
    .unwrap();
    assert!(bindings.accepts(HeaderRole::EngineCallback, &dispatched));

    assert_eq!(remaining_space_wait_budget_us(0), 3_500);
    assert_eq!(remaining_space_wait_budget_us(999), 2_501);
    assert_eq!(remaining_space_wait_budget_us(3_500), 0);
    assert_eq!(remaining_space_wait_budget_us(9_000), 0);
    assert_eq!(
        remaining_space_wait_budget(std::time::Duration::from_micros(999)),
        std::time::Duration::from_micros(2_501)
    );
}

#[test]
fn acquisition_timeout_revokes_and_cannot_be_rearmed_by_late_reply() {
    let mut fixture = pending_fixture();
    assert!(fixture.reducer.context_acquisition_failed(fixture.request));
    assert_eq!(fixture.reducer.status(), AdmissionStatus::Revoked);
    assert!(!fixture.reducer.context_reply(
        fixture.request,
        fixture.nonce,
        fixture.canonical_context,
        70,
    ));
    assert!(fixture.reducer.consume().is_none());
    assert_eq!(
        fixture.reducer.lineage().completeness,
        WordCompleteness::UnknownStart
    );
}

#[test]
fn target_key_before_admission_stays_literal_and_poisons_transferred_completeness() {
    let mut fixture = pending_fixture();
    let target = engine_path("/org/freedesktop/IBus/Engine/Lay/1");
    let mut literal_deliveries = 0;
    assert!(fixture.reducer.target_input_before_admission(&target));
    literal_deliveries += 1;
    assert!(fixture.reducer.marker(fixture.request, fixture.nonce, 60));
    let grant = fixture.reducer.consume().unwrap();
    assert_eq!(grant.lineage.completeness, WordCompleteness::UnknownStart);
    assert_eq!(literal_deliveries, 1);
    assert_eq!(fixture.output_edits, 0);
}

#[test]
fn reflexive_ticket_is_single_use_and_rotates_owner_generation() {
    let epoch = ConnectionGeneration(14);
    let canonical_context = context(epoch, "/org/freedesktop/IBus/InputContext_10");
    let mut reducer = ContextAdmissionReducer::new(
        epoch,
        GlobalEngineMode::Verified,
        GlobalProfile::Lay(profile("lay-us")),
    );
    let source = reducer
        .establish_source(
            engine_path("/org/freedesktop/IBus/Engine/Lay/0"),
            canonical_context.clone(),
            WordCompleteness::KnownStart,
            3,
        )
        .unwrap();
    reducer.focus_out(&source, 20).unwrap();
    assert!(reducer.disable(&source, 21));
    assert!(reducer.seal_source(&source, 3, 19));
    let nonce = BarrierNonce(8);
    let request = reducer
        .focus_in(&source.path, nonce, ReceiptOrigin::Native, 22)
        .unwrap();
    assert!(reducer.context_reply(request, nonce, canonical_context, 23));
    assert!(reducer.marker(request, nonce, 24));
    let grant = reducer.consume().unwrap();
    assert_eq!(grant.kind, TicketKind::Reflexive);
    assert_eq!(grant.source_owner.path, grant.target_owner.path);
    assert_ne!(grant.source_owner.generation, grant.target_owner.generation);
    assert!(reducer.is_current_owner(&grant.target_owner));
    assert!(reducer.consume().is_none());
}

#[test]
fn lifecycle_reflexive_refocus_does_not_require_engine_disable() {
    for origin in [ReceiptOrigin::Native, ReceiptOrigin::CompatibilityProperty] {
        let epoch = ConnectionGeneration(141);
        let canonical_context = context(epoch, "/org/freedesktop/IBus/InputContext_refocus");
        let mut reducer = ContextAdmissionReducer::new(
            epoch,
            GlobalEngineMode::Verified,
            GlobalProfile::Lay(profile("lay-us")),
        );
        let owner = reducer
            .establish_source(
                engine_path("/org/freedesktop/IBus/Engine/Lay/refocus"),
                canonical_context.clone(),
                WordCompleteness::KnownStart,
                3,
            )
            .unwrap();
        reducer.focus_out(&owner, 20).unwrap();
        assert!(reducer.seal_source(&owner, 3, 20));
        let nonce = BarrierNonce(81);
        let request = reducer.focus_in(&owner.path, nonce, origin, 21).unwrap();
        assert!(reducer.context_reply(request, nonce, canonical_context, 22));
        assert!(
            reducer.marker(request, nonce, 23),
            "ordinary focus-out never promises Disable: {origin:?}"
        );
        let grant = reducer.consume().expect("reflexive focus is ready");
        assert_eq!(grant.kind, TicketKind::Reflexive);
        assert_eq!(grant.lineage.completeness, WordCompleteness::KnownStart);
        assert_ne!(grant.source_owner.generation, grant.target_owner.generation);
        assert!(reducer.bridge_admission_token().is_some());
        assert!(reducer.consume().is_none());
    }
}

#[test]
fn lifecycle_later_empty_factory_supersedes_before_or_after_target_binding() {
    for bind_first in [false, true] {
        let epoch = ConnectionGeneration(142);
        let mut reducer = ContextAdmissionReducer::<u64>::new(
            epoch,
            GlobalEngineMode::Verified,
            GlobalProfile::Lay(profile("lay-us")),
        );
        let stale_path = engine_path("/org/freedesktop/IBus/Engine/Lay/stale_factory");
        let current_path = engine_path("/org/freedesktop/IBus/Engine/Lay/current_factory");
        let stale = reducer.open_factory_request(profile("lay-us"), 10).unwrap();
        if bind_first {
            assert!(reducer.bind_factory_target(stale, stale_path.clone()));
        }
        let current = reducer
            .open_factory_request(profile("lay-us"), 11)
            .expect("a later empty factory must remain recoverable");
        assert_ne!(current, stale);
        assert!(!reducer.bind_factory_target(stale, stale_path));
        assert!(reducer.bind_factory_target(current, current_path.clone()));
        let nonce = BarrierNonce(82);
        let request = reducer
            .begin_source_free_activation(current_path, nonce, ReceiptOrigin::Native, 12)
            .unwrap();
        assert!(reducer.context_reply(
            request,
            nonce,
            context(epoch, "/org/freedesktop/IBus/InputContext_factory"),
            12,
        ));
        assert!(reducer.marker(request, nonce, 13));
        let grant = reducer.consume_source_free_activation().unwrap();
        assert_eq!(grant.lineage.completeness, WordCompleteness::UnknownStart);
        assert_eq!(grant.tail_epoch, 0);
        assert!(
            reducer.consume().is_none(),
            "there was no transferable source"
        );
    }
}

#[test]
fn lifecycle_empty_factory_supersession_requires_strictly_later_position() {
    for position in [9, 10] {
        let mut reducer = ContextAdmissionReducer::<u64>::new(
            ConnectionGeneration(143),
            GlobalEngineMode::Verified,
            GlobalProfile::Lay(profile("lay-us")),
        );
        reducer.open_factory_request(profile("lay-us"), 10).unwrap();
        assert!(reducer
            .open_factory_request(profile("lay-us"), position)
            .is_none());
        assert!(reducer.owner().is_none());
        assert!(reducer.admission_token().is_none());
    }
}

#[test]
fn source_free_initial_activation_stays_unknown_until_a_real_boundary() {
    let epoch = ConnectionGeneration(40);
    let target = engine_path("/org/freedesktop/IBus/Engine/Lay/initial");
    let target_context = context(epoch, "/org/freedesktop/IBus/InputContext_initial");
    let mut reducer = ContextAdmissionReducer::new(
        epoch,
        GlobalEngineMode::Verified,
        GlobalProfile::Lay(profile("lay-us")),
    );
    let nonce = BarrierNonce(401);
    let request = reducer
        .begin_source_free_activation(target.clone(), nonce, ReceiptOrigin::Native, 10)
        .expect("source-free request");
    assert!(reducer.context_reply(request, nonce, target_context.clone(), 10));
    assert!(reducer.consume_source_free_activation().is_none());
    assert!(reducer.marker(request, nonce, 11));
    let grant = reducer
        .consume_source_free_activation()
        .expect("source-free activation grant");
    assert_eq!(grant.target_owner.path, target);
    assert_eq!(grant.target_activation.context, target_context);
    assert_eq!(grant.lineage.completeness, WordCompleteness::UnknownStart);
    assert_eq!(reducer.lineage(), grant.lineage);
    assert_eq!(reducer.latest_tail_epoch, 0);
    assert!(reducer
        .admission_token()
        .is_some_and(|token| reducer.revalidate(&token)));
}

#[test]
fn source_free_epoch_binding_survives_next_factory_before_source_handler() {
    let epoch = ConnectionGeneration(42);
    let source = engine_path("/org/freedesktop/IBus/Engine/Lay/source");
    let target = engine_path("/org/freedesktop/IBus/Engine/Lay/target");
    let canonical = context(epoch, "/org/freedesktop/IBus/InputContext_current");
    let lay_profile = profile("lay-us");
    let mut reducer = ContextAdmissionReducer::new(
        epoch,
        GlobalEngineMode::Verified,
        GlobalProfile::Lay(lay_profile.clone()),
    );
    let nonce = BarrierNonce(421);
    let request = reducer
        .begin_source_free_activation(source, nonce, ReceiptOrigin::Native, 10)
        .unwrap();
    assert!(reducer.context_reply(request, nonce, canonical, 10));
    assert!(reducer.marker(request, nonce, 11));
    let grant = reducer.consume_source_free_activation().unwrap();
    // The observer publishes the owner before the engine installs its empty
    // tail epoch. A later factory callback must not destroy that binding.
    open_bound_factory(&mut reducer, target, lay_profile, 12);
    assert!(reducer.is_current_owner(&grant.target_owner));
    assert!(reducer.bind_source_free_tail_epoch(&grant, 43));
    assert_eq!(reducer.latest_tail_epoch, 43);
    assert!(!reducer.bind_source_free_tail_epoch(&grant, 44));
    assert_eq!(reducer.lineage.completeness, WordCompleteness::UnknownStart);
}

#[test]
fn different_field_transfer_falls_back_to_the_same_receipt_and_marker_contract() {
    fn prove_fallback(origin: ReceiptOrigin, focus_position: u64, reply_position: u64) {
        let epoch = ConnectionGeneration(41);
        let source_context = context(epoch, "/org/freedesktop/IBus/InputContext_A");
        let target_context = context(epoch, "/org/freedesktop/IBus/InputContext_B");
        let target = engine_path("/org/freedesktop/IBus/Engine/Lay/target");
        let lay_profile = profile("lay-us");
        let mut reducer = ContextAdmissionReducer::new(
            epoch,
            GlobalEngineMode::Verified,
            GlobalProfile::Lay(lay_profile.clone()),
        );
        let source = reducer
            .establish_source(
                engine_path("/org/freedesktop/IBus/Engine/Lay/source"),
                source_context,
                WordCompleteness::KnownStart,
                12,
            )
            .unwrap();
        open_bound_factory(&mut reducer, target.clone(), lay_profile, 10);
        reducer.focus_out(&source, 20).unwrap();
        assert!(reducer.disable(&source, 21));
        assert!(reducer.seal_source(&source, 12, 19));
        let nonce = BarrierNonce(411);
        let request = reducer
            .focus_in(&target, nonce, origin, focus_position)
            .unwrap();
        assert!(reducer.context_reply(request, nonce, target_context.clone(), reply_position,));
        assert!(
            reducer.consume().is_none(),
            "mismatched field cannot transfer"
        );
        assert!(reducer.consume_source_free_activation().is_none());
        assert!(reducer.marker(request, nonce, reply_position + 1));
        let grant = reducer.consume_source_free_activation().unwrap();
        assert_eq!(grant.target_activation.context, target_context);
        assert_eq!(grant.lineage.completeness, WordCompleteness::UnknownStart);
        assert_eq!(reducer.latest_tail_epoch, 0);
    }

    prove_fallback(ReceiptOrigin::CompatibilityProperty, 40, 50);
    prove_fallback(ReceiptOrigin::Native, 40, 40);
}

#[test]
fn settled_word_scope_updates_lineage_and_the_only_sealable_tail_revision() {
    let epoch = ConnectionGeneration(42);
    let canonical_context = context(epoch, "/org/freedesktop/IBus/InputContext_settle");
    let lay_profile = profile("lay-us");
    let target = engine_path("/org/freedesktop/IBus/Engine/Lay/settled-target");
    let mut reducer = ContextAdmissionReducer::new(
        epoch,
        GlobalEngineMode::Verified,
        GlobalProfile::Lay(lay_profile.clone()),
    );
    let owner = reducer
        .establish_source(
            engine_path("/org/freedesktop/IBus/Engine/Lay/settled-source"),
            canonical_context.clone(),
            WordCompleteness::UnknownStart,
            6,
        )
        .unwrap();
    let key = header(epoch, 420);
    assert!(reducer.observe_key_start(&owner, key.clone(), KeyWordEffect::PossibleMutation));
    let mut scope = WordScope::new(reducer.lineage());
    assert_eq!(
        scope.close_at_observed_boundary(7, None),
        WordCompleteness::UnknownStart
    );
    let settled = SettledWordState::from_scope(7, &scope);
    assert!(reducer.settle_key(&owner, &key, settled));
    assert_eq!(reducer.lineage(), settled.lineage);
    assert_eq!(reducer.latest_tail_epoch, 7);

    open_bound_factory(&mut reducer, target.clone(), lay_profile, 10);
    reducer.focus_out(&owner, 20).unwrap();
    assert!(reducer.disable(&owner, 21));
    assert!(reducer.seal_source(&owner, 7, 19));
    let nonce = BarrierNonce(421);
    let request = reducer
        .focus_in(&target, nonce, ReceiptOrigin::Native, 22)
        .unwrap();
    assert!(reducer.context_reply(request, nonce, canonical_context, 22));
    assert!(reducer.marker(request, nonce, 23));
    let grant = reducer.consume().unwrap();
    assert_eq!(grant.source_tail_epoch, 7);
    assert_eq!(grant.lineage, settled.lineage);
}

#[test]
fn stale_settlement_revokes_but_old_owner_callbacks_cannot_modify_the_new_owner() {
    let epoch = ConnectionGeneration(43);
    let canonical_context = context(epoch, "/org/freedesktop/IBus/InputContext_stale");
    let mut stale: ContextAdmissionReducer<u64> = ContextAdmissionReducer::new(
        epoch,
        GlobalEngineMode::Verified,
        GlobalProfile::Lay(profile("lay-us")),
    );
    let stale_owner = stale
        .establish_source(
            engine_path("/org/freedesktop/IBus/Engine/Lay/stale"),
            canonical_context,
            WordCompleteness::KnownStart,
            9,
        )
        .unwrap();
    let stale_key = header(epoch, 430);
    assert!(stale.observe_key_start(
        &stale_owner,
        stale_key.clone(),
        KeyWordEffect::PossibleMutation
    ));
    let stale_scope = WordScope::new(stale.lineage());
    let before_stale = stale.revocation_generation();
    assert!(!stale.settle_key(
        &stale_owner,
        &stale_key,
        SettledWordState::from_scope(8, &stale_scope),
    ));
    assert!(stale.revocation_generation() > before_stale);
    assert_eq!(stale.lineage().completeness, WordCompleteness::UnknownStart);

    let mut fixture = pending_fixture();
    assert!(fixture.reducer.marker(fixture.request, fixture.nonce, 60));
    let old_owner = fixture.source_owner.clone();
    let new_owner = fixture.reducer.consume().unwrap().target_owner;
    let new_lineage = fixture.reducer.lineage();
    let new_revocation = fixture.reducer.revocation_generation();
    let new_token = fixture.reducer.admission_token().unwrap();
    let old_key = header(ConnectionGeneration(11), 431);
    assert!(!fixture.reducer.observe_key_start(
        &old_owner,
        old_key.clone(),
        KeyWordEffect::PossibleMutation
    ));
    assert!(!fixture.reducer.settle_key(
        &old_owner,
        &old_key,
        SettledWordState {
            tail_epoch: 99,
            content_type: None,
            lineage: WordLineage {
                generation: LineageGeneration(99),
                completeness: WordCompleteness::KnownStart,
                observed_suffix_chars: 0,
                observed_boundary_floor: None,
            },
        },
    ));
    assert!(fixture.reducer.focus_out(&old_owner, 70).is_none());
    assert!(!fixture.reducer.disable(&old_owner, 71));
    assert!(!fixture.reducer.seal_source(&old_owner, 99, 69));
    assert_eq!(fixture.reducer.owner(), Some(&new_owner));
    assert_eq!(fixture.reducer.lineage(), new_lineage);
    assert_eq!(fixture.reducer.revocation_generation(), new_revocation);
    assert!(fixture.reducer.revalidate(&new_token));
}

#[test]
fn unbound_factory_ticket_cannot_focus_before_its_engine_path_is_bound() {
    let epoch = ConnectionGeneration(44);
    let canonical_context = context(epoch, "/org/freedesktop/IBus/InputContext_factory");
    let lay_profile = profile("lay-us");
    let target = engine_path("/org/freedesktop/IBus/Engine/Lay/unbound");
    let mut reducer = ContextAdmissionReducer::new(
        epoch,
        GlobalEngineMode::Verified,
        GlobalProfile::Lay(lay_profile.clone()),
    );
    reducer
        .establish_source(
            engine_path("/org/freedesktop/IBus/Engine/Lay/source"),
            canonical_context,
            WordCompleteness::KnownStart,
            1,
        )
        .unwrap();
    let ticket = reducer.open_factory_request(lay_profile, 10).unwrap();
    let before = reducer.revocation_generation();
    assert!(reducer
        .focus_in(&target, BarrierNonce(441), ReceiptOrigin::Native, 11)
        .is_none());
    assert!(reducer.revocation_generation() > before);
    assert_eq!(reducer.status(), AdmissionStatus::Revoked);
    assert!(!reducer.bind_factory_target(ticket, target));
}

#[test]
fn admission_token_invalidates_on_lineage_owner_and_revocation_changes() {
    let epoch = ConnectionGeneration(45);
    let canonical_context = context(epoch, "/org/freedesktop/IBus/InputContext_token");
    let lay_profile = profile("lay-us");
    let target = engine_path("/org/freedesktop/IBus/Engine/Lay/token-target");
    let mut reducer = ContextAdmissionReducer::new(
        epoch,
        GlobalEngineMode::Verified,
        GlobalProfile::Lay(lay_profile.clone()),
    );
    let owner = reducer
        .establish_source(
            engine_path("/org/freedesktop/IBus/Engine/Lay/token-source"),
            canonical_context.clone(),
            WordCompleteness::KnownStart,
            1,
        )
        .unwrap();
    let before_lineage = reducer.admission_token().unwrap();
    let key = header(epoch, 450);
    assert!(reducer.observe_key_start(&owner, key.clone(), KeyWordEffect::PossibleMutation));
    let mut scope = WordScope::new(reducer.lineage());
    scope.close_at_observed_boundary(2, None);
    assert!(reducer.settle_key(&owner, &key, SettledWordState::from_scope(2, &scope),));
    assert!(!reducer.revalidate(&before_lineage));
    let before_owner = reducer.admission_token().unwrap();

    open_bound_factory(&mut reducer, target.clone(), lay_profile, 10);
    reducer.focus_out(&owner, 20).unwrap();
    assert!(reducer.disable(&owner, 21));
    assert!(reducer.seal_source(&owner, 2, 19));
    let nonce = BarrierNonce(451);
    let request = reducer
        .focus_in(&target, nonce, ReceiptOrigin::Native, 22)
        .unwrap();
    assert!(reducer.context_reply(request, nonce, canonical_context, 22));
    assert!(reducer.marker(request, nonce, 23));
    reducer.consume().unwrap();
    assert!(!reducer.revalidate(&before_owner));
    let before_revocation = reducer.admission_token().unwrap();
    reducer.malformed_or_unobserved_lifecycle();
    assert!(!reducer.revalidate(&before_revocation));
}

#[derive(Clone)]
struct ManualDeadline(Arc<ManualDeadlineState>);

struct ManualDeadlineState {
    fired: AtomicBool,
    waker: Mutex<Option<Waker>>,
}

impl ManualDeadline {
    fn new() -> Self {
        Self(Arc::new(ManualDeadlineState {
            fired: AtomicBool::new(false),
            waker: Mutex::new(None),
        }))
    }

    fn fire(&self) {
        self.0.fired.store(true, Ordering::Release);
        if let Some(waker) = self.0.waker.lock().unwrap().take() {
            waker.wake();
        }
    }
}

impl Future for ManualDeadline {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.0.fired.load(Ordering::Acquire) {
            Poll::Ready(())
        } else {
            *self.0.waker.lock().unwrap() = Some(cx.waker().clone());
            if self.0.fired.load(Ordering::Acquire) {
                Poll::Ready(())
            } else {
                Poll::Pending
            }
        }
    }
}

fn rendezvous_fixture(
    serial: u32,
) -> (
    CallbackStampStore<u64>,
    HeaderKey,
    RendezvousGuard,
    IngressStamp<u64>,
) {
    let epoch = ConnectionGeneration(20);
    let owner = OwnerGeneration(8);
    let store = CallbackStampStore::with_capacity(epoch, owner, 8);
    let key = header(epoch, serial);
    let guard = store.guard().unwrap();
    let stamp = IngressStamp {
        header: key.clone(),
        position: serial as u64,
        disposition: IngressDisposition::Passive,
    };
    (store, key, guard, stamp)
}

fn expect_stamp(outcome: RendezvousOutcome<u64>, stamp: &IngressStamp<u64>) {
    let mut delivered = 0;
    let scope = WordScope::new(WordLineage {
        generation: LineageGeneration(80),
        completeness: WordCompleteness::KnownStart,
        observed_suffix_chars: 0,
        observed_boundary_floor: None,
    });
    if outcome == RendezvousOutcome::Stamp(stamp.clone()) {
        delivered += 1;
    }
    assert_eq!(delivered, 1, "the already-dispatched key is delivered once");
    assert_eq!(scope.lineage().completeness, WordCompleteness::KnownStart);
}

#[test]
fn p121_2_rendezvous_publish_before_listen() {
    let (store, key, guard, stamp) = rendezvous_fixture(1);
    let RendezvousBegin::Pending(mut pending) = store.begin(key, guard) else {
        panic!("initial miss expected")
    };
    assert!(store.publish(stamp.clone()));
    pending.listen();
    expect_stamp(pending.recheck().unwrap(), &stamp);
    drop(pending);
    assert_eq!(store.waiter_count(), 0);
}

#[test]
fn p121_2_rendezvous_publish_between_listen_and_recheck() {
    let (store, key, guard, stamp) = rendezvous_fixture(2);
    let RendezvousBegin::Pending(mut pending) = store.begin(key, guard) else {
        panic!("initial miss expected")
    };
    pending.listen();
    assert!(store.publish(stamp.clone()));
    expect_stamp(pending.recheck().unwrap(), &stamp);
    drop(pending);
    assert_eq!(store.waiter_count(), 0);
}

#[test]
fn p121_2_rendezvous_publish_between_recheck_and_await() {
    let (store, key, guard, stamp) = rendezvous_fixture(3);
    let RendezvousBegin::Pending(mut pending) = store.begin(key, guard) else {
        panic!("initial miss expected")
    };
    pending.listen();
    assert!(pending.recheck().is_none());
    assert!(store.publish(stamp.clone()));
    let deadline = ManualDeadline::new();
    let outcome = zbus::block_on(pending.wait_until(deadline));
    expect_stamp(outcome, &stamp);
    drop(pending);
    assert_eq!(store.waiter_count(), 0);
}

#[test]
fn p121_2_rendezvous_publish_after_await_is_polled() {
    let (store, key, guard, stamp) = rendezvous_fixture(4);
    let RendezvousBegin::Pending(mut pending) = store.begin(key, guard) else {
        panic!("initial miss expected")
    };
    let deadline = ManualDeadline::new();
    let mut wait = Box::pin(pending.wait_until(deadline));
    let mut cx = Context::from_waker(Waker::noop());
    assert!(wait.as_mut().poll(&mut cx).is_pending());
    assert_eq!(store.waiter_count(), 1);
    assert!(store.publish(stamp.clone()));
    let Poll::Ready(outcome) = wait.as_mut().poll(&mut cx) else {
        panic!("publication must wake the listener")
    };
    expect_stamp(outcome, &stamp);
    drop(wait);
    drop(pending);
    assert_eq!(store.waiter_count(), 0);
}

#[test]
fn p121_2_rendezvous_absolute_timeout_and_revocation_do_not_leak_waiters() {
    let (store, key, guard, _) = rendezvous_fixture(5);
    let RendezvousBegin::Pending(mut pending) = store.begin(key, guard) else {
        panic!("initial miss expected")
    };
    let deadline = ManualDeadline::new();
    let mut wait = Box::pin(pending.wait_until(deadline.clone()));
    let mut cx = Context::from_waker(Waker::noop());
    assert!(wait.as_mut().poll(&mut cx).is_pending());
    deadline.fire();
    assert_eq!(
        wait.as_mut().poll(&mut cx),
        Poll::Ready(RendezvousOutcome::Failed(RendezvousFailure::Timeout))
    );
    drop(wait);
    drop(pending);
    assert_eq!(store.waiter_count(), 0);

    let (store, key, guard, _) = rendezvous_fixture(6);
    let RendezvousBegin::Pending(mut pending) = store.begin(key, guard) else {
        panic!("initial miss expected")
    };
    let deadline = ManualDeadline::new();
    let mut wait = Box::pin(pending.wait_until(deadline));
    assert!(wait.as_mut().poll(&mut cx).is_pending());
    store.revoke();
    assert_eq!(
        wait.as_mut().poll(&mut cx),
        Poll::Ready(RendezvousOutcome::Failed(RendezvousFailure::Revoked))
    );
    drop(wait);
    drop(pending);
    assert_eq!(store.waiter_count(), 0);
}

#[test]
fn rendezvous_ring_eviction_fails_closed() {
    // A full ring may discard an unrelated old stamp while publishing the
    // exact awaited header. Its retained identity must remain deliverable.
    for capacity in [1, 2, 8, 128] {
        let epoch = ConnectionGeneration(21);
        let owner = OwnerGeneration(9);
        let store = CallbackStampStore::with_capacity(epoch, owner, capacity);
        for serial in 1..=capacity as u32 {
            assert!(store.publish(IngressStamp {
                header: header(epoch, serial),
                position: serial as u64,
                disposition: IngressDisposition::Passive,
            }));
        }
        let serial = capacity as u32 + 1;
        let wanted = header(epoch, serial);
        let guard = store.guard().unwrap();
        let RendezvousBegin::Pending(pending) = store.begin(wanted.clone(), guard) else {
            panic!("initial miss expected")
        };
        let stamp = IngressStamp {
            header: wanted,
            position: serial as u64,
            disposition: IngressDisposition::Passive,
        };
        assert!(store.publish(stamp.clone()));
        expect_stamp(pending.recheck().unwrap(), &stamp);
        // Revocation still dominates even an exact retained stamp.
        store.replace_owner(OwnerGeneration(10));
        assert_eq!(
            pending.recheck(),
            Some(RendezvousOutcome::Failed(RendezvousFailure::Revoked))
        );
    }
    let epoch = ConnectionGeneration(21);
    let owner = OwnerGeneration(9);
    let store = CallbackStampStore::with_capacity(epoch, owner, 1);
    let wanted = header(epoch, 70);
    let guard = store.guard().unwrap();
    let RendezvousBegin::Pending(pending) = store.begin(wanted, guard) else {
        panic!("initial miss expected")
    };
    assert!(store.publish(IngressStamp {
        header: header(epoch, 71),
        position: 71,
        disposition: IngressDisposition::Passive,
    }));
    assert!(pending.recheck().is_none());
    assert!(store.publish(IngressStamp {
        header: header(epoch, 72),
        position: 72,
        disposition: IngressDisposition::Passive,
    }));
    assert_eq!(
        pending.recheck(),
        Some(RendezvousOutcome::Failed(RendezvousFailure::Evicted))
    );
}

#[test]
fn duplicate_complete_header_key_revokes_rendezvous_epoch() {
    let (store, key, guard, stamp) = rendezvous_fixture(73);
    assert!(store.publish(stamp.clone()));
    assert!(!store.publish(stamp));
    let RendezvousBegin::Ready(outcome) = store.begin(key, guard) else {
        panic!("duplicate must revoke, not wait")
    };
    assert_eq!(
        outcome,
        RendezvousOutcome::Failed(RendezvousFailure::Revoked)
    );
}

#[test]
fn owner_replacement_revokes_waiters_but_retains_typed_stamp_for_stale_handler() {
    let epoch = ConnectionGeneration(22);
    let old_generation = OwnerGeneration(10);
    let new_generation = OwnerGeneration(11);
    let store = CallbackStampStore::with_capacity(epoch, old_generation, 8);
    let key = header(epoch, 74);
    let old_owner = EngineOwner {
        path: engine_path("/org/freedesktop/IBus/Engine/Lay/old"),
        generation: old_generation,
    };
    let old_guard = store.guard().unwrap();
    assert!(store.publish(IngressStamp {
        header: key.clone(),
        position: 74,
        disposition: IngressDisposition::Revocation {
            owner: old_owner.clone(),
        },
    }));

    store.replace_owner(new_generation);

    let RendezvousBegin::Ready(old_outcome) = store.begin(key.clone(), old_guard) else {
        panic!("old waiter guard must fail immediately")
    };
    assert_eq!(
        old_outcome,
        RendezvousOutcome::Failed(RendezvousFailure::Revoked)
    );
    let new_guard = store.guard().unwrap();
    let RendezvousBegin::Ready(RendezvousOutcome::Stamp(stamp)) = store.begin(key, new_guard)
    else {
        panic!("new handler must recover the typed old-owner disposition")
    };
    assert_eq!(
        stamp.disposition,
        IngressDisposition::Revocation { owner: old_owner }
    );
}

struct HeldOrderedStream<S> {
    inner: S,
    released: Arc<AtomicBool>,
    waiter: Arc<Mutex<Option<Waker>>>,
}

impl<S> HeldOrderedStream<S> {
    fn new(inner: S) -> (Self, HeldRelease) {
        let released = Arc::new(AtomicBool::new(false));
        let waiter = Arc::new(Mutex::new(None));
        (
            Self {
                inner,
                released: Arc::clone(&released),
                waiter: Arc::clone(&waiter),
            },
            HeldRelease { released, waiter },
        )
    }
}

impl<S> OrderedStream for HeldOrderedStream<S>
where
    S: OrderedStream + Unpin,
{
    type Ordering = S::Ordering;
    type Data = S::Data;

    fn poll_next_before(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        before: Option<&Self::Ordering>,
    ) -> Poll<PollResult<Self::Ordering, Self::Data>> {
        if !self.released.load(Ordering::Acquire) {
            *self.waiter.lock().unwrap() = Some(cx.waker().clone());
            // Crucial: an intentionally hidden earlier item is not NoneBefore.
            return Poll::Pending;
        }
        Pin::new(&mut self.inner).poll_next_before(cx, before)
    }
}

struct HeldRelease {
    released: Arc<AtomicBool>,
    waiter: Arc<Mutex<Option<Waker>>>,
}

impl HeldRelease {
    fn release(&self) {
        self.released.store(true, Ordering::Release);
        if let Some(waker) = self.waiter.lock().unwrap().take() {
            waker.wake();
        }
    }
}

#[cfg(unix)]
#[test]
fn p121_2_d2_actual_zbus_join_holds_ready_marker_for_earlier_stream() {
    use std::os::unix::net::UnixStream;
    use std::thread;

    use zbus::message::Type;
    use zbus::{connection, MatchRule, MessageStream};

    const SETUP_IFACE: &str = "org.lay.ContextAdmission.Setup";
    const LIFECYCLE_IFACE: &str = "org.freedesktop.IBus";
    const MARKER_IFACE: &str = "org.lay.ContextAdmission";
    const PATH: &str = "/org/lay/ContextAdmission";

    let (server_socket, client_socket) = UnixStream::pair().unwrap();
    let guid = zbus::Guid::generate();
    let server_guid = guid.clone();
    let server_thread = thread::spawn(move || {
        zbus::block_on(
            connection::Builder::unix_stream(server_socket)
                .server(server_guid)
                .unwrap()
                .p2p()
                .build(),
        )
        .unwrap()
    });
    let receiver = zbus::block_on(
        connection::Builder::unix_stream(client_socket)
            .p2p()
            .build(),
    )
    .unwrap();
    let sender = server_thread.join().unwrap();

    let signal_rule = |interface: &'static str, member: Option<&'static str>| {
        let builder = MatchRule::builder()
            .msg_type(Type::Signal)
            .interface(interface)
            .unwrap();
        match member {
            Some(member) => builder.member(member).unwrap().build(),
            None => builder.build(),
        }
    };

    let mut setup = zbus::block_on(MessageStream::for_match_rule(
        signal_rule(SETUP_IFACE, None),
        &receiver,
        Some(8),
    ))
    .unwrap();
    let lifecycle = zbus::block_on(MessageStream::for_match_rule(
        signal_rule(LIFECYCLE_IFACE, Some("GlobalEngineChanged")),
        &receiver,
        Some(8),
    ))
    .unwrap();
    let marker = zbus::block_on(MessageStream::for_match_rule(
        signal_rule(MARKER_IFACE, Some("ContextAdmissionBarrier")),
        &receiver,
        Some(8),
    ))
    .unwrap();
    let mut marker_probe = marker.clone();
    let idle = zbus::block_on(MessageStream::for_match_rule(
        signal_rule(MARKER_IFACE, Some("NeverEmitted")),
        &receiver,
        Some(1),
    ))
    .unwrap();

    let setup_members = [
        "SourceSeal",
        "Factory",
        "FocusOut",
        "Disable",
        "FocusIn",
        "GetReply",
    ];
    for member in setup_members {
        zbus::block_on(sender.emit_signal(None::<()>, PATH, SETUP_IFACE, member, &())).unwrap();
    }
    let mut setup_positions = Vec::new();
    for expected in setup_members {
        let message = zbus::block_on(next_ordered_message(Pin::new(&mut setup)))
            .expect("setup stream open")
            .expect("valid setup message");
        assert_eq!(message.header().member().unwrap().as_str(), expected);
        setup_positions.push(message.recv_position());
    }

    let epoch = ConnectionGeneration(30);
    let canonical_context = context(epoch, "/org/freedesktop/IBus/InputContext_9");
    let lay_profile = profile("lay-us");
    let mut reducer = ContextAdmissionReducer::new(
        epoch,
        GlobalEngineMode::Verified,
        GlobalProfile::Lay(lay_profile.clone()),
    );
    let source_owner = reducer
        .establish_source(
            engine_path("/org/freedesktop/IBus/Engine/Lay/0"),
            canonical_context.clone(),
            WordCompleteness::KnownStart,
            7,
        )
        .unwrap();
    open_bound_factory(
        &mut reducer,
        engine_path("/org/freedesktop/IBus/Engine/Lay/1"),
        lay_profile.clone(),
        setup_positions[1],
    );
    reducer
        .focus_out(&source_owner, setup_positions[2])
        .unwrap();
    assert!(reducer.disable(&source_owner, setup_positions[3]));
    assert!(reducer.seal_source(&source_owner, 7, setup_positions[0]));
    let nonce = BarrierNonce(991);
    let request = reducer
        .focus_in(
            &engine_path("/org/freedesktop/IBus/Engine/Lay/1"),
            nonce,
            ReceiptOrigin::CompatibilityProperty,
            setup_positions[4],
        )
        .unwrap();
    assert!(reducer.context_reply(request, nonce, canonical_context, setup_positions[5],));

    zbus::block_on(sender.emit_signal(
        None::<()>,
        PATH,
        LIFECYCLE_IFACE,
        "GlobalEngineChanged",
        &"foreign-ime",
    ))
    .unwrap();
    zbus::block_on(sender.emit_signal(
        None::<()>,
        PATH,
        LIFECYCLE_IFACE,
        "GlobalEngineChanged",
        &"lay-us",
    ))
    .unwrap();
    zbus::block_on(sender.emit_signal(
        None::<()>,
        PATH,
        MARKER_IFACE,
        "ContextAdmissionBarrier",
        &nonce.0,
    ))
    .unwrap();

    // A duplicate narrow subscription proves the socket reader has queued the
    // marker while the earlier lifecycle branch below remains test-gated.
    let probe = zbus::block_on(next_ordered_message(Pin::new(&mut marker_probe)))
        .expect("probe stream open")
        .expect("valid marker probe");
    assert_eq!(
        probe.header().member().unwrap().as_str(),
        "ContextAdmissionBarrier"
    );

    let (held_lifecycle, release) = HeldOrderedStream::new(lifecycle);
    let lifecycle_and_marker = join_ordered_streams(held_lifecycle, marker);
    let mut merged = Box::pin(join_ordered_streams(lifecycle_and_marker, idle));
    let mut cx = Context::from_waker(Waker::noop());
    assert!(merged.as_mut().poll_next_before(&mut cx, None).is_pending());
    {
        let mut inner = merged.as_mut().stream_a();
        let (lifecycle_buffer, marker_buffer) = inner.as_mut().peek_buffered();
        assert!(lifecycle_buffer.is_none());
        assert!(marker_buffer.is_some(), "marker must be ready but held");
    }

    release.release();
    let mut observed = Vec::new();
    for _ in 0..3 {
        let message = zbus::block_on(next_ordered_message(merged.as_mut()))
            .expect("joined stream open")
            .expect("valid joined message");
        observed.push(message);
    }
    assert!(observed[0].recv_position() < observed[1].recv_position());
    assert!(observed[1].recv_position() < observed[2].recv_position());
    let foreign_name = observed[0].body().deserialize::<String>().unwrap();
    let returned_name = observed[1].body().deserialize::<String>().unwrap();
    let observed_nonce = BarrierNonce(observed[2].body().deserialize::<u64>().unwrap());
    assert_eq!(foreign_name, "foreign-ime");
    assert_eq!(returned_name, "lay-us");
    assert_eq!(observed_nonce, nonce);
    assert_eq!(
        observed[2].header().member().unwrap().as_str(),
        "ContextAdmissionBarrier"
    );

    reducer.global_engine_changed(GlobalProfile::Foreign(profile(&foreign_name)));
    reducer.global_engine_changed(GlobalProfile::Lay(profile(&returned_name)));
    assert!(!reducer.marker(request, observed_nonce, observed[2].recv_position()));
    assert_eq!(reducer.status(), AdmissionStatus::Revoked);
    assert!(reducer.consume().is_none());
}

#[test]
fn observed_boundary_floor_reopens_only_with_retained_provenance() {
    let mut scope = WordScope::new(WordLineage {
        generation: LineageGeneration(7),
        completeness: WordCompleteness::UnknownStart,
        observed_suffix_chars: 0,
        observed_boundary_floor: None,
    });
    assert_eq!(
        scope.close_at_observed_boundary(10, Some(3)),
        WordCompleteness::UnknownStart
    );
    scope.close_at_observed_boundary(11, Some(8));
    let closed = scope.lineage();
    assert_eq!(closed.observed_boundary_floor, Some(3));
    scope.reopen_at_retained_boundary(Some(3));
    assert_eq!(scope.lineage().completeness, WordCompleteness::KnownStart);
    assert_eq!(scope.lineage().observed_boundary_floor, Some(3));
    assert_ne!(scope.lineage().generation, closed.generation);
    assert_eq!(scope.lineage().observed_suffix_chars, 0);
    assert!(std::mem::size_of::<WordLineage>() <= 24);
    assert!(std::mem::size_of::<RendezvousOutcome<zbus::message::Sequence>>() <= 224);
    println!(
        "word_lineage_bytes={} callback_outcome_bytes={}",
        std::mem::size_of::<WordLineage>(),
        std::mem::size_of::<RendezvousOutcome<zbus::message::Sequence>>()
    );
}

#[test]
fn observed_boundary_floor_rejects_deleted_or_unobserved_separators() {
    for retained in [None, Some(0), Some(2)] {
        let mut scope = WordScope::new(WordLineage {
            generation: LineageGeneration(7),
            completeness: WordCompleteness::KnownStart,
            observed_suffix_chars: 0,
            observed_boundary_floor: Some(3),
        });
        scope.reopen_at_retained_boundary(retained);
        assert_eq!(scope.lineage().completeness, WordCompleteness::UnknownStart);
        assert_eq!(scope.lineage().observed_boundary_floor, None);
        assert_eq!(scope.lineage().observed_suffix_chars, 0);
    }
}

#[test]
fn observed_boundary_floor_does_not_infer_leftward_prefix_rebasing() {
    let mut scope = WordScope::new(WordLineage {
        generation: LineageGeneration(7),
        completeness: WordCompleteness::KnownStart,
        observed_suffix_chars: 0,
        observed_boundary_floor: Some(100),
    });
    scope.close_at_observed_boundary(11, Some(159));
    assert_eq!(scope.lineage().observed_boundary_floor, Some(100));
    scope.reopen_at_retained_boundary(Some(99));
    assert_eq!(scope.lineage().completeness, WordCompleteness::UnknownStart);
    assert_eq!(scope.lineage().observed_boundary_floor, None);
    scope.close_at_observed_boundary(12, Some(19));
    assert_eq!(scope.lineage().completeness, WordCompleteness::KnownStart);
    assert_eq!(scope.lineage().observed_boundary_floor, Some(19));
    scope.revoke_for_input_gap();
    scope.reopen_at_retained_boundary(Some(19));
    assert_eq!(scope.lineage().completeness, WordCompleteness::UnknownStart);
    assert_eq!(scope.lineage().observed_boundary_floor, None);
}

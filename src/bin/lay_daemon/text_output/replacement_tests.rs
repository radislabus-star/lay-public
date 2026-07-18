use super::super::observable_state::{
    expected_buffered_suffix, DaemonInputObservation, DaemonMutationLease, DaemonMutationPolicy,
    DaemonTextContext, DaemonTextContextObserver, DaemonTextObservation,
};
use super::*;
use lay::keyboard::text_to_key_events;
use lay::word_buffer::WordBuffer;
use std::sync::atomic::{AtomicU64, Ordering};

fn text_replacement(
    move_left: u32,
    backspaces: u32,
    insert: impl Into<String>,
    move_right: u32,
) -> TextReplacement {
    TextReplacement {
        move_left,
        backspaces,
        insert: insert.into(),
        move_right,
    }
}

#[test]
fn minimal_current_tail_insert_keeps_layout_for_inserted_symbol() {
    let current_tail_plan = text_replacement(0, 1, "$", 0);
    assert!(!layout_after_replacement_plan(
        &current_tail_plan,
        "только $",
        false
    ));

    let middle_plan = text_replacement(7, 3, "ТУТ", 7);
    assert!(!layout_after_replacement_plan(
        &middle_plan,
        "ТУТ DOUBLE",
        true
    ));
}

#[test]
fn completed_mixed_tail_continues_in_previous_context_layout() {
    let completed_tail_plan = text_replacement(1, 6, "Wechat", 1);
    let mixed_context_layout =
        layout_after_replacement_plan(&completed_tail_plan, "текст в Wechat ", false);
    let english_context_layout =
        layout_after_replacement_plan(&completed_tail_plan, "file on off ", false);

    assert!(mixed_context_layout);
    assert!(!english_context_layout);
}

#[test]
fn middle_insert_does_not_claim_insert_layout_is_cursor_layout() {
    let middle_plan = text_replacement(7, 3, "ТУТ", 7);

    let cursor_layout = layout_after_replacement_plan(&middle_plan, "ТУТ DOUBLE", true);
    assert!(!cursor_layout);
}

fn lease_context(epoch: u64) -> DaemonTextContext {
    DaemonTextContext::new(Some("test-window:field".to_string()), epoch)
}

fn automatic_lease(epoch: u64, suffix: &str) -> DaemonMutationLease {
    DaemonMutationLease::new(
        expected_buffered_suffix(lease_context(epoch), suffix),
        DaemonMutationPolicy::AutomaticDestructive,
    )
}

fn buffered_text(text: &str) -> WordBuffer {
    let mut buffer = WordBuffer::new();
    for event in text_to_key_events(text, false).expect("test text must map to key events") {
        buffer.push(event);
    }
    buffer
}

fn mutation_preflight<'a>(
    lease: DaemonMutationLease,
    epoch: &'a AtomicU64,
    buffer: &'a WordBuffer,
    input_observation: DaemonInputObservation,
) -> DaemonMutationPreflight<'a, 'a> {
    DaemonMutationPreflight::new(
        lease,
        DaemonTextContextObserver::new(Some("test-window:field"), epoch),
        buffer,
        input_observation,
    )
}

#[test]
fn mutation_lease_rejects_stale_epoch() {
    let epoch = AtomicU64::new(7);
    let buffer = buffered_text("ghbdtn");
    let mut preflight = mutation_preflight(
        automatic_lease(7, "ghbdtn"),
        &epoch,
        &buffer,
        DaemonInputObservation::ExclusiveInputObserved,
    );
    epoch.store(8, Ordering::Release);

    let error = preflight.consume().expect_err("stale epoch must block");

    assert!(error.contains("stale field epoch"));
}

#[test]
fn mutation_lease_rejects_stale_suffix() {
    let epoch = AtomicU64::new(7);
    let buffer = buffered_text("ghbdty");
    let mut preflight = mutation_preflight(
        automatic_lease(7, "ghbdtn"),
        &epoch,
        &buffer,
        DaemonInputObservation::ExclusiveInputObserved,
    );

    let error = preflight.consume().expect_err("stale suffix must block");

    assert!(error.contains("stale daemon buffered suffix"));
}

#[test]
fn automatic_mutation_lease_rejects_non_isolated_edit() {
    let epoch = AtomicU64::new(7);
    let buffer = buffered_text("ghbdtn");
    let mut preflight = mutation_preflight(
        automatic_lease(7, "ghbdtn"),
        &epoch,
        &buffer,
        DaemonInputObservation::for_input_isolation(false),
    );

    let error = preflight
        .consume()
        .expect_err("automatic uinput edit must require isolated input");

    assert!(error.contains("exclusive input observation"));
}

#[test]
fn mutation_lease_is_consumed_once() {
    let epoch = AtomicU64::new(7);
    let buffer = buffered_text("ghbdtn");
    let mut preflight = mutation_preflight(
        automatic_lease(7, "ghbdtn"),
        &epoch,
        &buffer,
        DaemonInputObservation::ExclusiveInputObserved,
    );
    preflight
        .validate_current()
        .expect("non-consuming validation");
    preflight.consume().expect("single final consume");

    let error = preflight.consume().expect_err("lease reuse must block");

    assert!(error.contains("already consumed"));
}

#[test]
fn mutation_lease_revalidates_when_consumed() {
    let epoch = AtomicU64::new(7);
    let buffer = buffered_text("ghbdtn");
    let mut preflight = mutation_preflight(
        automatic_lease(7, "ghbdtn"),
        &epoch,
        &buffer,
        DaemonInputObservation::ExclusiveInputObserved,
    );
    preflight.validate_current().expect("initial validation");
    epoch.store(8, Ordering::Release);

    let error = preflight
        .consume()
        .expect_err("final consume must revalidate current state");

    assert!(error.contains("stale field epoch"));
    assert!(preflight
        .consume()
        .unwrap_err()
        .contains("already consumed"));
}

#[test]
fn valid_isolated_mutation_lease_allows_edit() {
    let epoch = AtomicU64::new(7);
    let buffer = buffered_text("ghbdtn");
    let mut preflight = mutation_preflight(
        automatic_lease(7, "ghbdtn"),
        &epoch,
        &buffer,
        DaemonInputObservation::ExclusiveInputObserved,
    );

    preflight.consume().expect("fresh isolated edit");
}

#[test]
fn delayed_mutation_preflight_verifies_original_behind_observed_cursor_tail() {
    let epoch = AtomicU64::new(7);
    let buffer = buffered_text("rfr x");
    let observation = DaemonTextObservation::new(
        lease_context(7),
        DaemonTextContextObserver::new(Some("test-window:field"), &epoch),
    );
    let mut preflight = observation
        .automatic_destructive_preflight_behind_cursor(&buffer, "rfr ", 1, true)
        .expect("observed following key should form a verifiable suffix");

    preflight
        .consume()
        .expect("the original token remains immediately behind the observed tail");
}

#[test]
fn delayed_mutation_preflight_rejects_changed_original_behind_cursor() {
    let epoch = AtomicU64::new(7);
    let buffer = buffered_text("rfx x");
    let observation = DaemonTextObservation::new(
        lease_context(7),
        DaemonTextContextObserver::new(Some("test-window:field"), &epoch),
    );
    let mut preflight = observation
        .automatic_destructive_preflight_behind_cursor(&buffer, "rfr ", 1, true)
        .expect("cursor tail is observable");

    let error = preflight
        .consume()
        .expect_err("changed original token must block delayed output");
    assert!(error.contains("stale daemon buffered suffix"));
}

#[test]
fn manual_mutation_policy_accepts_fresh_observable_without_claiming_exclusive_input() {
    let epoch = AtomicU64::new(7);
    let buffer = buffered_text("ghbdtn");
    let lease = DaemonMutationLease::new(
        expected_buffered_suffix(lease_context(7), "ghbdtn"),
        DaemonMutationPolicy::ExplicitManualUserIntent,
    );
    let mut preflight = mutation_preflight(
        lease,
        &epoch,
        &buffer,
        DaemonInputObservation::for_input_isolation(false),
    );

    preflight
        .consume()
        .expect("manual policy can use a fresh daemon buffer without claiming exclusivity");
}

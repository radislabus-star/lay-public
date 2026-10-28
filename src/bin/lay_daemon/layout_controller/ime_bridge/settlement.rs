//! Settlement of the existing exact-tail replay; no new input or edit route.
use super::{ImeDelegatedTailLease, VisibleTailV3Reply};
use lay::text_edit::VisibleTailSource;
use std::time::{Duration, Instant};

const QUEUED_MANUAL_TOGGLE_SETTLEMENT_MAX: Duration = Duration::from_millis(80);

pub(in crate::layout_controller) fn wait_for_committed_tail_settlement(
    expected_tail: &str,
    expected_layout_is_ru: bool,
    lease: &ImeDelegatedTailLease,
    target_path: Option<&str>,
    mut verify_focus: impl FnMut() -> Result<(), String>,
) -> Result<(), String> {
    if expected_tail.is_empty() {
        return Err("queued manual toggle has no expected visible tail".to_string());
    }
    let deadline = Instant::now() + QUEUED_MANUAL_TOGGLE_SETTLEMENT_MAX;
    let mut target_path = target_path.map(str::to_owned);
    loop {
        verify_focus()?;
        let reply = lease.current_v3_reply()?;
        // The controlled layout handoff may select a different engine path.
        // Once observed, that path and the original exact field stay bound
        // through settlement. A matching suffix in another field is no receipt.
        let path = target_path.get_or_insert_with(|| reply.4.clone());
        if reply.4.is_empty()
            || reply.4 != *path
            || lease.exact_focus_receipt.as_deref() != Some(reply.5.as_str())
            || reply.3 < lease.expected_epoch()
        {
            return Err("exact IME tail settlement identity changed".to_string());
        }
        if committed_tail_is_settled(&reply, expected_tail, expected_layout_is_ru, lease, path) {
            verify_focus()?;
            return Ok(());
        }
        if Instant::now() >= deadline {
            return Err(format!(
                "queued manual toggle timed out waiting for exact committed tail: expected_chars={} expected_layout_is_ru={expected_layout_is_ru} last_source={} last_epoch={}",
                expected_tail.chars().count(), reply.0, reply.3
            ));
        }
        std::thread::yield_now();
    }
}

pub(super) fn committed_tail_is_settled(
    reply: &VisibleTailV3Reply,
    expected_tail: &str,
    expected_layout_is_ru: bool,
    lease: &ImeDelegatedTailLease,
    target_path: &str,
) -> bool {
    VisibleTailSource::from_bridge_state(&reply.0) == Some(VisibleTailSource::ImeCommittedTail)
        && !target_path.is_empty()
        && reply.4 == target_path
        && lease.exact_focus_receipt.as_deref() == Some(reply.5.as_str())
        && reply.3 > lease.expected_epoch()
        && reply.2 == expected_layout_is_ru
        && reply.1.ends_with(expected_tail)
}

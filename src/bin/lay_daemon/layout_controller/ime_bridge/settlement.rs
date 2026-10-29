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
    mut input_processed: impl FnMut() -> Result<bool, String>,
    verify_focus: impl FnMut() -> Result<(), String>,
    readback_matches: impl Fn(&str, &str) -> bool,
) -> Result<(), String> {
    wait_for_tail_observation(
        expected_tail,
        expected_layout_is_ru,
        lease,
        target_path,
        || {
            if input_processed()? {
                lease.current_v3_reply().map(Some)
            } else {
                Ok(None)
            }
        },
        verify_focus,
        readback_matches,
    )
}

fn wait_for_tail_observation(
    expected_tail: &str,
    expected_layout_is_ru: bool,
    lease: &ImeDelegatedTailLease,
    target_path: Option<&str>,
    mut observe: impl FnMut() -> Result<Option<VisibleTailV3Reply>, String>,
    mut verify_focus: impl FnMut() -> Result<(), String>,
    readback_matches: impl Fn(&str, &str) -> bool,
) -> Result<(), String> {
    if expected_tail.is_empty() {
        return Err("queued manual toggle has no expected visible tail".to_string());
    }
    let deadline = Instant::now() + QUEUED_MANUAL_TOGGLE_SETTLEMENT_MAX;
    let mut target_path = target_path.map(str::to_owned);
    loop {
        verify_focus()?;
        let Some(reply) = observe()? else {
            if Instant::now() >= deadline {
                return Err(
                    "queued manual toggle timed out waiting for processed native input".into(),
                );
            }
            std::thread::yield_now();
            continue;
        };
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
        if committed_tail_is_settled(
            &reply,
            expected_tail,
            expected_layout_is_ru,
            lease,
            path,
            &readback_matches,
        ) {
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
    readback_matches: impl Fn(&str, &str) -> bool,
) -> bool {
    VisibleTailSource::from_bridge_state(&reply.0) == Some(VisibleTailSource::ImeCommittedTail)
        && !target_path.is_empty()
        && reply.4 == target_path
        && lease.exact_focus_receipt.as_deref() == Some(reply.5.as_str())
        && reply.3 > lease.expected_epoch()
        && reply.2 == expected_layout_is_ru
        && readback_matches(&reply.1, expected_tail)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;

    fn lease() -> ImeDelegatedTailLease {
        super::super::tail_lease_from_v3_reply(
            VisibleTailSource::ImeCommittedTail,
            "before",
            6,
            (
                "passive:committed-tail".into(),
                "before".into(),
                true,
                7,
                "/engine/source".into(),
                "field-a".into(),
            ),
        )
        .unwrap()
    }

    fn actual_target() -> VisibleTailV3Reply {
        (
            "passive:committed-tail".into(),
            "after".into(),
            false,
            9,
            "/engine/target".into(),
            "field-a".into(),
        )
    }

    #[test]
    fn native_processing_client_readback_policy_cannot_override_receipt_scope() {
        let lease = lease();
        let mut target = actual_target();
        target.1 = "after\u{a0}".into();
        assert!(!committed_tail_is_settled(
            &target,
            "after ",
            false,
            &lease,
            "/engine/target",
            |actual, expected| actual.ends_with(expected),
        ));
        assert!(committed_tail_is_settled(
            &target,
            "after ",
            false,
            &lease,
            "/engine/target",
            |actual, expected| actual == "after\u{a0}" && expected == "after ",
        ));
        for invalid in [
            "source",
            "field",
            "path",
            "epoch",
            "unchanged_epoch",
            "mode",
        ] {
            let mut reply = target.clone();
            match invalid {
                "source" => reply.0 = "passive:unknown-context".into(),
                "field" => reply.5 = "field-b".into(),
                "path" => reply.4 = "/engine/other".into(),
                "epoch" => reply.3 = 6,
                "unchanged_epoch" => reply.3 = 7,
                "mode" => reply.2 = true,
                _ => unreachable!(),
            }
            let comparisons = Cell::new(0);
            assert!(
                !committed_tail_is_settled(
                    &reply,
                    "after ",
                    false,
                    &lease,
                    "/engine/target",
                    |_, _| {
                        comparisons.set(comparisons.get() + 1);
                        true
                    },
                ),
                "{invalid}"
            );
            assert_eq!(comparisons.get(), 0, "{invalid}");
        }
    }

    #[test]
    fn native_processing_pending_precedes_the_actual_field_read() {
        let polls = Cell::new(0);
        let actual_reads = Cell::new(0);
        let checks = Cell::new(0);
        let result = wait_for_tail_observation(
            "after",
            false,
            &lease(),
            Some("/engine/target"),
            || {
                polls.set(polls.get() + 1);
                if polls.get() <= 3 {
                    return Ok(None);
                }
                actual_reads.set(actual_reads.get() + 1);
                Ok(Some(actual_target()))
            },
            || {
                checks.set(checks.get() + 1);
                Ok(())
            },
            |actual, expected| actual.ends_with(expected),
        );
        assert!(
            result.is_ok(),
            "pending native processing must stay within the existing settlement wait: {result:?}"
        );
        assert_eq!(polls.get(), 4);
        assert_eq!(actual_reads.get(), 1);
        assert_eq!(checks.get(), 5);
    }

    #[test]
    fn native_processing_pending_preserves_focus_identity_and_errors() {
        for invalid in ["focus", "field", "path", "epoch", "read_error"] {
            let polls = Cell::new(0);
            let checks = Cell::new(0);
            let result = wait_for_tail_observation(
                "after",
                false,
                &lease(),
                Some("/engine/target"),
                || {
                    polls.set(polls.get() + 1);
                    if polls.get() == 1 {
                        return Ok(None);
                    }
                    if invalid == "read_error" {
                        return Err("observation denied".into());
                    }
                    let mut reply = actual_target();
                    match invalid {
                        "field" => reply.5 = "field-b".into(),
                        "path" => reply.4 = "/engine/other".into(),
                        "epoch" => reply.3 = 6,
                        _ => (),
                    }
                    Ok(Some(reply))
                },
                || {
                    checks.set(checks.get() + 1);
                    if invalid == "focus" && checks.get() > 1 {
                        Err("focus changed".into())
                    } else {
                        Ok(())
                    }
                },
                |actual, expected| actual.ends_with(expected),
            );
            let expected = match invalid {
                "focus" => "focus changed",
                "read_error" => "observation denied",
                _ => "exact IME tail settlement identity changed",
            };
            assert_eq!(
                result.as_ref().map_err(String::as_str),
                Err(expected),
                "{invalid}"
            );
            assert_eq!(polls.get(), if invalid == "focus" { 1 } else { 2 });
        }
    }

    #[test]
    fn native_processing_pending_uses_the_existing_settlement_deadline() {
        let started = Instant::now();
        let result = wait_for_tail_observation(
            "after",
            false,
            &lease(),
            Some("/engine/target"),
            || Ok(None),
            || Ok(()),
            |actual, expected| actual.ends_with(expected),
        );
        assert!(result.unwrap_err().contains("timed out"));
        assert!(started.elapsed() >= QUEUED_MANUAL_TOGGLE_SETTLEMENT_MAX);
    }
}

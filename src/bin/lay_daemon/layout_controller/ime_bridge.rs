use lay::manual_toggle::{
    plan_manual_toggle, ImeManualToggleOutcome, ManualTogglePlan, ManualToggleRequest,
    ManualToggleRoute, VisibleTail,
};
use lay::text_backend::ImeReplaceRequest;
use lay::text_edit::{tail_chars, VisibleTailSnapshot, VisibleTailSource};
use serde::de::DeserializeOwned;
use std::time::{Duration, Instant};
use zbus::zvariant::Type;

use super::super::{active_text_backend, log};
use super::gnome_dbus::dbus_connection;

const IME_DBUS_DEST: &str = "io.github.radislabus_star.LayIme";
const IME_DBUS_PATH: &str = "/io/github/radislabus_star/LayIme";
const IME_DBUS_INTERFACE: &str = "io.github.radislabus_star.LayIme";
const QUEUED_MANUAL_TOGGLE_SETTLEMENT_MAX: Duration = Duration::from_millis(80);

type VisibleTailV2Reply = (String, String, bool, u64, String);
type VisibleTailV3Reply = (String, String, bool, u64, String, String);

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ImeDelegatedTailLease {
    snapshot: VisibleTailSnapshot,
    initial_layout_is_ru: bool,
    source_path: String,
    exact_focus_receipt: Option<String>,
}

impl ImeDelegatedTailLease {
    pub(crate) fn expected_suffix(&self) -> &str {
        &self.snapshot.expected_suffix
    }

    pub(crate) fn initial_layout_is_ru(&self) -> bool {
        self.initial_layout_is_ru
    }

    pub(crate) fn expected_epoch(&self) -> u64 {
        self.snapshot.epoch
    }

    pub(crate) fn expected_source_path(&self) -> &str {
        &self.source_path
    }

    pub(crate) fn validate_current(&self, backspaces: u32) -> Result<(), String> {
        if let Some(expected_focus_receipt) = self.exact_focus_receipt.as_deref() {
            return validate_delegated_tail_v3_reply(
                &self.snapshot,
                &self.source_path,
                expected_focus_receipt,
                backspaces,
                visible_tail_v3()?,
                false,
                self.initial_layout_is_ru,
            );
        }
        validate_delegated_tail_reply(
            &self.snapshot,
            backspaces,
            visible_tail_v2()?,
            false,
            self.initial_layout_is_ru,
        )
    }

    pub(crate) fn validate_after_controlled_layout_handoff(
        &self,
        backspaces: u32,
        target_layout_is_ru: bool,
    ) -> Result<String, String> {
        if let Some(expected_focus_receipt) = self.exact_focus_receipt.as_deref() {
            let reply = visible_tail_v3()?;
            let target_path = reply.4.clone();
            validate_delegated_tail_v3_reply(
                &self.snapshot,
                &self.source_path,
                expected_focus_receipt,
                backspaces,
                reply,
                true,
                target_layout_is_ru,
            )?;
            return Ok(target_path);
        }
        let reply = visible_tail_v2()?;
        let target_path = reply.4.clone();
        validate_delegated_tail_reply(
            &self.snapshot,
            backspaces,
            reply,
            true,
            target_layout_is_ru,
        )?;
        Ok(target_path)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ImeCommittedTailReplay {
    plan: ManualTogglePlan,
    lease: ImeDelegatedTailLease,
}

impl ImeCommittedTailReplay {
    pub(crate) fn into_parts(self) -> (ManualTogglePlan, ImeDelegatedTailLease) {
        (self.plan, self.lease)
    }
}

pub(super) fn should_try_text_backend() -> bool {
    active_text_backend().should_try_ime()
}

pub(super) fn try_replace_tail(
    original: &str,
    replacement: &str,
    kind: &str,
) -> Result<bool, String> {
    if !should_try_text_backend() {
        return Ok(false);
    }
    let request = ImeReplaceRequest::committed_tail(original, replacement);
    if request.is_noop() {
        return Ok(false);
    }
    let expected_tail = tail_chars(original, request.backspaces as usize);
    let (_, visible_tail, _, epoch, focus) = visible_tail_v2()?;
    if !visible_tail.is_empty() && !visible_tail.ends_with(&expected_tail) {
        return Ok(false);
    }
    match replace_tail_checked(
        request.backspaces,
        &request.text,
        kind,
        &expected_tail,
        epoch,
        &focus,
    ) {
        Ok(true) => {
            log(&format!(
                "  IME replace-tail ({kind}): bs={} insert={:?}",
                request.backspaces, request.text
            ));
            Ok(true)
        }
        Ok(false) => {
            log(
                "⚠ IME replace-tail rejected/no proven surrounding text; secondary backend blocked",
            );
            Ok(false)
        }
        Err(e) => {
            log(&format!(
                "⚠ IME replace-tail failed: {e}; secondary backend blocked"
            ));
            Err(e)
        }
    }
}

fn visible_tail_v2() -> Result<VisibleTailV2Reply, String> {
    call_ime_noarg("VisibleTailV2")
}

fn visible_tail_v3() -> Result<VisibleTailV3Reply, String> {
    call_ime_noarg("VisibleTailV3")
}

pub(super) fn capture_delegated_tail_lease(
    expected_suffix: &str,
    backspaces: u32,
) -> Result<ImeDelegatedTailLease, String> {
    delegated_tail_lease_from_reply(expected_suffix, backspaces, visible_tail_v2()?)
}

pub(super) fn capture_committed_tail_replay() -> Result<ImeCommittedTailReplay, String> {
    committed_tail_replay_from_v3_reply(visible_tail_v3()?)
}

pub(super) fn wait_for_committed_tail_settlement(
    expected_tail: &str,
    expected_layout_is_ru: bool,
) -> Result<(), String> {
    if expected_tail.is_empty() {
        return Err("queued manual toggle has no expected visible tail".to_string());
    }
    let deadline = Instant::now() + QUEUED_MANUAL_TOGGLE_SETTLEMENT_MAX;
    loop {
        let reply = visible_tail_v3()?;
        if committed_tail_is_settled(&reply, expected_tail, expected_layout_is_ru) {
            return Ok(());
        }
        if Instant::now() >= deadline {
            return Err(format!(
                "queued manual toggle timed out waiting for exact committed tail: expected_tail={expected_tail:?} expected_layout_is_ru={expected_layout_is_ru} last={:?}",
                (reply.0, reply.1, reply.2, reply.3, reply.4)
            ));
        }
        std::thread::yield_now();
    }
}

fn committed_tail_is_settled(
    reply: &VisibleTailV3Reply,
    expected_tail: &str,
    expected_layout_is_ru: bool,
) -> bool {
    VisibleTailSource::from_bridge_state(&reply.0) == Some(VisibleTailSource::ImeCommittedTail)
        && reply.2 == expected_layout_is_ru
        && reply.1.ends_with(expected_tail)
}

fn committed_tail_replay_from_v3_reply(
    reply: VisibleTailV3Reply,
) -> Result<ImeCommittedTailReplay, String> {
    let source = VisibleTailSource::from_bridge_state(&reply.0)
        .ok_or_else(|| format!("unknown IME tail source: {}", reply.0))?;
    if source != VisibleTailSource::ImeCommittedTail {
        return Err(format!(
            "exact committed-tail replay source changed to {}",
            source.source_id()
        ));
    }
    let plan = plan_manual_toggle(ManualToggleRequest {
        visible_tail: VisibleTail::ime_committed_tail(&reply.1),
        current_layout_is_ru: reply.2,
        preserve_trailing_whitespace: true,
    })
    .ok_or_else(|| "exact committed-tail snapshot has no reversible token".to_string())?;
    if plan.route != ManualToggleRoute::ImeCommittedTail {
        return Err("exact committed-tail planner selected the wrong route".to_string());
    }
    let expected_suffix = tail_chars(&reply.1, plan.backspaces as usize);
    let lease = tail_lease_from_v3_reply(
        VisibleTailSource::ImeCommittedTail,
        &expected_suffix,
        plan.backspaces,
        reply,
    )?;
    Ok(ImeCommittedTailReplay { plan, lease })
}

fn delegated_tail_lease_from_reply(
    expected_suffix: &str,
    backspaces: u32,
    reply: VisibleTailV2Reply,
) -> Result<ImeDelegatedTailLease, String> {
    tail_lease_from_reply(
        VisibleTailSource::DaemonWordBuffer,
        expected_suffix,
        backspaces,
        reply,
    )
}

fn tail_lease_from_reply(
    expected_source: VisibleTailSource,
    expected_suffix: &str,
    backspaces: u32,
    reply: VisibleTailV2Reply,
) -> Result<ImeDelegatedTailLease, String> {
    let source = VisibleTailSource::from_bridge_state(&reply.0)
        .ok_or_else(|| format!("unknown IME tail source: {}", reply.0))?;
    if source != expected_source {
        return Err(format!(
            "tail lease source changed to {}",
            source.source_id()
        ));
    }
    if reply.4.is_empty() {
        return Err("delegated tail has no focus identity".to_string());
    }
    let snapshot =
        VisibleTailSnapshot::new(source, expected_suffix, Some(reply.4.clone()), reply.3);
    let initial_layout_is_ru = reply.2;
    let source_path = reply.4.clone();
    validate_delegated_tail_reply(&snapshot, backspaces, reply, false, initial_layout_is_ru)?;
    Ok(ImeDelegatedTailLease {
        snapshot,
        initial_layout_is_ru,
        source_path,
        exact_focus_receipt: None,
    })
}

fn tail_lease_from_v3_reply(
    expected_source: VisibleTailSource,
    expected_suffix: &str,
    backspaces: u32,
    reply: VisibleTailV3Reply,
) -> Result<ImeDelegatedTailLease, String> {
    if reply.4.is_empty() || reply.5.is_empty() {
        return Err("exact committed tail has no engine path or field focus receipt".to_string());
    }
    let source_path = reply.4.clone();
    let focus_receipt = reply.5.clone();
    let projected = (reply.0, reply.1, reply.2, reply.3, source_path.clone());
    let mut lease = tail_lease_from_reply(expected_source, expected_suffix, backspaces, projected)?;
    lease.source_path = source_path;
    lease.exact_focus_receipt = Some(focus_receipt);
    Ok(lease)
}

fn validate_delegated_tail_v3_reply(
    snapshot: &VisibleTailSnapshot,
    source_path: &str,
    expected_focus_receipt: &str,
    backspaces: u32,
    reply: VisibleTailV3Reply,
    allow_controlled_path_handoff: bool,
    expected_layout_is_ru: bool,
) -> Result<(), String> {
    if reply.5 != expected_focus_receipt {
        return Err(format!(
            "delegated field focus changed: expected={expected_focus_receipt:?} actual={:?}",
            reply.5
        ));
    }
    if !allow_controlled_path_handoff && reply.4 != source_path {
        return Err(format!(
            "delegated engine path changed: expected={source_path:?} actual={:?}",
            reply.4
        ));
    }
    let projected = (reply.0, reply.1, reply.2, reply.3, reply.4);
    validate_delegated_tail_reply(
        snapshot,
        backspaces,
        projected,
        allow_controlled_path_handoff,
        expected_layout_is_ru,
    )
}

fn validate_delegated_tail_reply(
    snapshot: &VisibleTailSnapshot,
    backspaces: u32,
    reply: VisibleTailV2Reply,
    allow_controlled_focus_handoff: bool,
    expected_layout_is_ru: bool,
) -> Result<(), String> {
    let source = VisibleTailSource::from_bridge_state(&reply.0)
        .ok_or_else(|| format!("unknown IME tail source: {}", reply.0))?;
    let delete_chars = usize::try_from(backspaces)
        .map_err(|_| "delegated Backspace count does not fit usize".to_string())?;
    if delete_chars == 0 || snapshot.expected_suffix.chars().count() != delete_chars {
        return Err(format!(
            "delegated Backspace count mismatch: backspaces={backspaces} suffix_chars={}",
            snapshot.expected_suffix.chars().count()
        ));
    }
    let identity_matches = if allow_controlled_focus_handoff {
        !reply.4.is_empty() && snapshot.source == source && snapshot.epoch == reply.3
    } else {
        snapshot.matches_source_focus_and_epoch(source, Some(reply.4.as_str()), reply.3)
    };
    if !identity_matches {
        return Err(format!(
            "delegated tail identity changed: expected_source={} actual_source={} expected_focus={:?} actual_focus={:?} expected_epoch={} actual_epoch={}",
            snapshot.source.source_id(),
            source.source_id(),
            snapshot.focus_id,
            reply.4,
            snapshot.epoch,
            reply.3,
        ));
    }
    if reply.2 != expected_layout_is_ru {
        return Err(format!(
            "delegated tail layout changed: expected_ru={expected_layout_is_ru} actual_ru={}",
            reply.2
        ));
    }
    if !snapshot.matches_current_suffix(&reply.1, delete_chars) {
        return Err(format!(
            "delegated IME tail does not end with exact suffix {:?}",
            snapshot.expected_suffix
        ));
    }
    Ok(())
}

pub(super) fn call_ping() -> Result<String, String> {
    call_ime_noarg("Ping")
}

pub(super) fn owns_active_text() -> Result<bool, String> {
    call_ime_noarg("OwnsActiveText")
}

pub(super) fn input_state() -> Result<String, String> {
    call_ime_noarg("InputState")
}

pub(super) fn manual_toggle() -> Result<ImeManualToggleOutcome, String> {
    let (status, target_layout_is_ru): (u8, bool) = call_ime_noarg("ManualToggleV3")?;
    ImeManualToggleOutcome::from_v3(status, target_layout_is_ru).map_err(ToString::to_string)
}

pub(super) fn can_replace_committed_tail(backspaces: u32) -> Result<bool, String> {
    let reply = dbus_connection()?
        .call_method(
            Some(IME_DBUS_DEST),
            IME_DBUS_PATH,
            Some(IME_DBUS_INTERFACE),
            "CanReplaceCommittedTail",
            &(backspaces,),
        )
        .map_err(|e| e.to_string())?;
    reply
        .body()
        .deserialize::<bool>()
        .map_err(|e| e.to_string())
}

pub(super) fn suppress_next_autocorrect() -> Result<bool, String> {
    call_ime_noarg("SuppressNextAutocorrect")
}

pub(super) fn suppress_next_autocorrect_v2(
    expected_suffix: &str,
    expected_epoch: u64,
    expected_path: &str,
    expected_layout_is_ru: bool,
) -> Result<bool, String> {
    let reply = dbus_connection()?
        .call_method(
            Some(IME_DBUS_DEST),
            IME_DBUS_PATH,
            Some(IME_DBUS_INTERFACE),
            "SuppressNextAutocorrectV2",
            &(
                expected_suffix,
                expected_epoch,
                expected_path,
                expected_layout_is_ru,
            ),
        )
        .map_err(|error| error.to_string())?;
    reply
        .body()
        .deserialize::<bool>()
        .map_err(|error| error.to_string())
}

pub(super) fn cancel_exact_manual_toggle_handoff_v2(
    expected_epoch: u64,
    expected_path: &str,
) -> Result<bool, String> {
    let reply = dbus_connection()?
        .call_method(
            Some(IME_DBUS_DEST),
            IME_DBUS_PATH,
            Some(IME_DBUS_INTERFACE),
            "CancelExactManualToggleHandoffV2",
            &(expected_epoch, expected_path),
        )
        .map_err(|error| error.to_string())?;
    reply
        .body()
        .deserialize::<bool>()
        .map_err(|error| error.to_string())
}

pub(super) fn cancel_exact_manual_toggle_suppression_v2(
    expected_epoch: u64,
    expected_path: &str,
) -> Result<bool, String> {
    let reply = dbus_connection()?
        .call_method(
            Some(IME_DBUS_DEST),
            IME_DBUS_PATH,
            Some(IME_DBUS_INTERFACE),
            "CancelExactManualToggleSuppressionV2",
            &(expected_epoch, expected_path),
        )
        .map_err(|error| error.to_string())?;
    reply
        .body()
        .deserialize::<bool>()
        .map_err(|error| error.to_string())
}

fn call_ime_noarg<T>(method: &str) -> Result<T, String>
where
    T: DeserializeOwned + Type,
{
    let reply = dbus_connection()?
        .call_method(
            Some(IME_DBUS_DEST),
            IME_DBUS_PATH,
            Some(IME_DBUS_INTERFACE),
            method,
            &(),
        )
        .map_err(|e| e.to_string())?;
    reply.body().deserialize::<T>().map_err(|e| e.to_string())
}

fn replace_tail_checked(
    backspaces: u32,
    text: &str,
    kind: &str,
    expected_tail: &str,
    expected_epoch: u64,
    expected_focus: &str,
) -> Result<bool, String> {
    let started = std::time::Instant::now();
    let reply = dbus_connection()?
        .call_method(
            Some(IME_DBUS_DEST),
            IME_DBUS_PATH,
            Some(IME_DBUS_INTERFACE),
            "ReplaceTailV4",
            &(
                backspaces,
                text,
                kind,
                expected_tail,
                expected_epoch,
                expected_focus,
            ),
        )
        .map_err(|e| e.to_string())?;
    let call_ms = started.elapsed().as_millis();
    lay::action_log::record_timing_profile(
        "ime-bridge",
        kind,
        &[("replace_tail_v4_dbus_call", call_ms)],
    );
    reply
        .body()
        .deserialize::<bool>()
        .map_err(|e| e.to_string())
}

#[cfg(test)]
mod delegated_tail_tests {
    use super::*;

    fn reply(state: &str, text: &str, epoch: u64, focus: &str) -> VisibleTailV2Reply {
        (
            state.to_string(),
            text.to_string(),
            false,
            epoch,
            focus.to_string(),
        )
    }

    fn reply_v3(
        state: &str,
        text: &str,
        epoch: u64,
        path: &str,
        focus_receipt: &str,
    ) -> VisibleTailV3Reply {
        (
            state.to_string(),
            text.to_string(),
            false,
            epoch,
            path.to_string(),
            focus_receipt.to_string(),
        )
    }

    #[test]
    fn exact_daemon_tail_creates_and_revalidates_one_lease() {
        let lease = delegated_tail_lease_from_reply(
            "ytn",
            3,
            reply("passive:daemon-word-buffer", "prefix ytn", 7, "/focus/1"),
        )
        .expect("exact lease");

        assert!(validate_delegated_tail_reply(
            &lease.snapshot,
            3,
            reply("passive:daemon-word-buffer", "prefix ytn", 7, "/focus/1"),
            false,
            false,
        )
        .is_ok());
    }

    #[test]
    fn exact_committed_tail_plan_keeps_the_observed_autocomplete_suffix() {
        let replay = committed_tail_replay_from_v3_reply(reply_v3(
            "passive:committed-tail",
            "prefix ghjdthrf ",
            19,
            "/engine/us",
            "/field/7\u{1f}gtk-entry",
        ))
        .expect("exact committed-tail replay");
        let (plan, lease) = replay.into_parts();

        assert_eq!(plan.edit.original_tail, "prefix ghjdthrf ");
        assert_eq!(plan.edit.original_token, "ghjdthrf");
        assert_eq!(plan.replacement, "проверка ");
        assert_eq!(plan.backspaces, 9);
        assert!(plan.target_layout_is_ru);
        assert_eq!(lease.expected_suffix(), "ghjdthrf ");
    }

    #[test]
    fn exact_committed_tail_plan_rejects_daemon_or_active_sources() {
        for state in ["passive:daemon-word-buffer", "active:composition"] {
            assert!(committed_tail_replay_from_v3_reply(reply_v3(
                state,
                "ghbdtn",
                7,
                "/engine/us",
                "/field/1\u{1f}gtk-entry",
            ))
            .is_err());
        }
    }

    #[test]
    fn exact_committed_tail_rejects_field_change_across_layout_handoff() {
        let lease = tail_lease_from_v3_reply(
            VisibleTailSource::ImeCommittedTail,
            "ghbdtn",
            6,
            reply_v3(
                "passive:committed-tail",
                "ghbdtn",
                7,
                "/engine/us",
                "/field/1\u{1f}gtk-entry",
            ),
        )
        .expect("exact field lease");

        assert!(validate_delegated_tail_v3_reply(
            &lease.snapshot,
            lease.expected_source_path(),
            lease.exact_focus_receipt.as_deref().expect("focus receipt"),
            6,
            reply_v3(
                "passive:committed-tail",
                "ghbdtn",
                7,
                "/engine/ru",
                "/field/2\u{1f}gtk-entry",
            ),
            true,
            false,
        )
        .is_err());
    }

    #[test]
    fn stale_wrong_source_or_wrong_length_tail_is_fail_closed() {
        let lease = delegated_tail_lease_from_reply(
            "ytn",
            3,
            reply("passive:daemon-word-buffer", "prefix ytn", 7, "/focus/1"),
        )
        .expect("exact lease");

        for invalid in [
            reply("passive:daemon-word-buffer", "prefix yt", 7, "/focus/1"),
            reply("passive:daemon-word-buffer", "prefix ytn", 8, "/focus/1"),
            reply("passive:daemon-word-buffer", "prefix ytn", 7, "/focus/2"),
            reply("active:composition", "prefix ytn", 7, "/focus/1"),
        ] {
            assert!(
                validate_delegated_tail_reply(&lease.snapshot, 3, invalid, false, false).is_err()
            );
        }
        assert!(validate_delegated_tail_reply(
            &lease.snapshot,
            4,
            reply("passive:daemon-word-buffer", "prefix ytn", 7, "/focus/1"),
            false,
            false,
        )
        .is_err());
        assert!(validate_delegated_tail_reply(
            &lease.snapshot,
            3,
            reply("passive:daemon-word-buffer", "prefix ytn", 7, "/engine/ru"),
            true,
            true,
        )
        .is_err());
    }

    #[test]
    fn controlled_layout_handoff_may_change_only_engine_path() {
        let lease = delegated_tail_lease_from_reply(
            "ytn",
            3,
            reply("passive:daemon-word-buffer", "prefix ytn", 7, "/engine/us"),
        )
        .expect("exact lease");

        assert!(validate_delegated_tail_reply(
            &lease.snapshot,
            3,
            reply("passive:daemon-word-buffer", "prefix ytn", 7, "/engine/ru"),
            true,
            false,
        )
        .is_ok());
        assert!(validate_delegated_tail_reply(
            &lease.snapshot,
            3,
            reply("passive:daemon-word-buffer", "prefix ytn", 8, "/engine/ru"),
            true,
            false,
        )
        .is_err());
    }

    #[test]
    fn empty_receipt_cannot_authorize_deletion() {
        assert!(delegated_tail_lease_from_reply(
            "ytn",
            3,
            reply("passive:daemon-word-buffer", "", 7, "/focus/1"),
        )
        .is_err());
        assert!(delegated_tail_lease_from_reply(
            "ytn",
            3,
            reply("passive:daemon-word-buffer", "ytn", 7, ""),
        )
        .is_err());
    }

    #[test]
    fn queued_manual_toggle_waits_for_the_complete_previous_replay() {
        assert!(!committed_tail_is_settled(
            &reply_v3("passive:committed-tail", "при", 7, "/engine/ru", "field-a"),
            "привет",
            false,
        ));
        assert!(committed_tail_is_settled(
            &reply_v3(
                "passive:committed-tail",
                "привет",
                10,
                "/engine/ru",
                "field-a"
            ),
            "привет",
            false,
        ));
    }
}

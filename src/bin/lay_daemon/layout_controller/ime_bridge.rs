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

pub(crate) fn await_same_gesture_exact_receipt(
    mut request: impl FnMut() -> Result<ImeManualToggleOutcome, String>,
) -> Result<ImeManualToggleOutcome, String> {
    let started = Instant::now();
    loop {
        let outcome = request()?;
        if outcome != ImeManualToggleOutcome::AwaitingExactSnapshot {
            return Ok(outcome);
        }
        // Give the IME's 700-ms owner lease one final poll to revoke itself.
        if started.elapsed() > Duration::from_millis(750) {
            return Ok(ImeManualToggleOutcome::NotHandled);
        }
        std::thread::sleep(Duration::from_millis(10));
    }
}

type VisibleTailV2Reply = (String, String, bool, u64, String);
type VisibleTailV3Reply = (String, String, bool, u64, String, String);
type VisibleTailV4Reply = (String, String, bool, u64, String, String, bool);

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ImeDelegatedTailLease {
    snapshot: VisibleTailSnapshot,
    initial_layout_is_ru: bool,
    source_path: String,
    exact_focus_receipt: Option<String>,
    personalization: Option<bool>,
}

impl ImeDelegatedTailLease {
    pub(crate) fn allows_personalization(&self) -> bool {
        self.personalization == Some(true)
    }

    fn privacy_safe_error(&self, error: String) -> String {
        if self.allows_personalization() {
            error
        } else {
            "exact IME tail lease rejected; payload retention disabled".to_string()
        }
    }

    fn current_v3_reply(&self) -> Result<VisibleTailV3Reply, String> {
        let Some(expected) = self.personalization else {
            return visible_tail_v3();
        };
        let reply: VisibleTailV4Reply = call_ime_noarg("VisibleTailV4")?;
        self.bound_v4_reply(reply, expected)
    }

    fn bound_v4_reply(
        &self,
        reply: VisibleTailV4Reply,
        expected: bool,
    ) -> Result<VisibleTailV3Reply, String> {
        if reply.6 != expected || self.personalization != Some(expected) {
            return Err("exact IME tail retention policy changed".to_string());
        }
        Ok(project_visible_tail_v4(reply).0)
    }

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
            return self
                .current_v3_reply()
                .and_then(|reply| {
                    validate_delegated_tail_v3_reply(
                        &self.snapshot,
                        &self.source_path,
                        expected_focus_receipt,
                        backspaces,
                        reply,
                        false,
                        self.initial_layout_is_ru,
                    )
                })
                .map_err(|error| self.privacy_safe_error(error));
        }
        validate_delegated_tail_reply(
            &self.snapshot,
            backspaces,
            visible_tail_v2()?,
            false,
            self.initial_layout_is_ru,
        )
        .map_err(|error| self.privacy_safe_error(error))
    }

    pub(crate) fn validate_after_controlled_layout_handoff(
        &self,
        backspaces: u32,
        target_layout_is_ru: bool,
    ) -> Result<String, String> {
        if let Some(expected_focus_receipt) = self.exact_focus_receipt.as_deref() {
            let reply = self
                .current_v3_reply()
                .map_err(|error| self.privacy_safe_error(error))?;
            let target_path = reply.4.clone();
            validate_delegated_tail_v3_reply(
                &self.snapshot,
                &self.source_path,
                expected_focus_receipt,
                backspaces,
                reply,
                true,
                target_layout_is_ru,
            )
            .map_err(|error| self.privacy_safe_error(error))?;
            return Ok(target_path);
        }
        let reply = visible_tail_v2()?;
        let target_path = reply.4.clone();
        validate_delegated_tail_reply(&self.snapshot, backspaces, reply, true, target_layout_is_ru)
            .map_err(|error| self.privacy_safe_error(error))?;
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

pub(super) fn current_input_mode_is_ru() -> Result<bool, String> {
    input_mode_from_v3_reply(visible_tail_v3()?)
}

fn input_mode_from_v3_reply(reply: VisibleTailV3Reply) -> Result<bool, String> {
    // No focus/admission returns a sentinel false without an engine. A real
    // private decoder still has its path even when its text/field is redacted.
    if reply.4 == "/" || zbus::zvariant::ObjectPath::try_from(reply.4.as_str()).is_err() {
        return Err("actual IME decoder observation is unavailable".to_string());
    }
    Ok(reply.2)
}

fn project_visible_tail_v4(reply: VisibleTailV4Reply) -> (VisibleTailV3Reply, bool) {
    (
        (reply.0, reply.1, reply.2, reply.3, reply.4, reply.5),
        reply.6,
    )
}

fn visible_tail_for_manual_replay() -> Result<(VisibleTailV3Reply, Option<bool>), String> {
    match call_ime_noarg::<VisibleTailV4Reply>("VisibleTailV4") {
        Ok(reply) => {
            let (reply, permission) = project_visible_tail_v4(reply);
            Ok((reply, Some(permission)))
        }
        // Older public IME versions still support conversion. Their unknown
        // privacy cannot authorize text retention. Do not hide other errors.
        Err(error) if error.contains("UnknownMethod") => {
            visible_tail_v3().map(|reply| (reply, None))
        }
        Err(error) => Err(error),
    }
}

pub(super) fn capture_delegated_tail_lease(
    expected_suffix: &str,
    backspaces: u32,
) -> Result<ImeDelegatedTailLease, String> {
    delegated_tail_lease_from_reply(expected_suffix, backspaces, visible_tail_v2()?)
}

pub(super) fn capture_committed_tail_replay() -> Result<ImeCommittedTailReplay, String> {
    let (reply, permission) = visible_tail_for_manual_replay()?;
    committed_tail_replay_from_readout(reply, permission)
}

fn committed_tail_replay_from_readout(
    reply: VisibleTailV3Reply,
    permission: Option<bool>,
) -> Result<ImeCommittedTailReplay, String> {
    let source = VisibleTailSource::from_bridge_state(&reply.0);
    let engine_present = !reply.4.is_empty();
    let field_receipt_present = !reply.5.is_empty();
    let mut replay = committed_tail_replay_from_v3_reply(reply).map_err(|error| {
        log(&error.diagnostic(source, permission, engine_present, field_receipt_present));
        if permission == Some(true) {
            error.detail
        } else {
            "exact IME tail capture rejected; payload retention disabled".to_string()
        }
    })?;
    replay.lease.personalization = permission;
    Ok(replay)
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
        let (reply, _) = visible_tail_for_manual_replay()?;
        if committed_tail_is_settled(&reply, expected_tail, expected_layout_is_ru) {
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

fn committed_tail_is_settled(
    reply: &VisibleTailV3Reply,
    expected_tail: &str,
    expected_layout_is_ru: bool,
) -> bool {
    VisibleTailSource::from_bridge_state(&reply.0) == Some(VisibleTailSource::ImeCommittedTail)
        && reply.2 == expected_layout_is_ru
        && reply.1.ends_with(expected_tail)
}

#[derive(Debug)]
struct CommittedTailCaptureRejection {
    reason: &'static str,
    detail: String,
}

impl CommittedTailCaptureRejection {
    fn diagnostic(
        &self,
        source: Option<VisibleTailSource>,
        permission: Option<bool>,
        engine_present: bool,
        field_receipt_present: bool,
    ) -> String {
        let source_kind = source.map_or("unknown", VisibleTailSource::source_id);
        let permission = match permission {
            None => "unknown",
            Some(false) => "denied",
            Some(true) => "allowed",
        };
        format!(
            "· exact IME tail capture rejection: reason={} source_kind={source_kind} permission={permission} engine_present={engine_present} field_receipt_present={field_receipt_present}",
            self.reason
        )
    }
}

fn committed_tail_replay_from_v3_reply(
    reply: VisibleTailV3Reply,
) -> Result<ImeCommittedTailReplay, CommittedTailCaptureRejection> {
    let source = VisibleTailSource::from_bridge_state(&reply.0).ok_or_else(|| {
        CommittedTailCaptureRejection {
            reason: "source_unknown",
            detail: format!("unknown IME tail source: {}", reply.0),
        }
    })?;
    if source != VisibleTailSource::ImeCommittedTail {
        return Err(CommittedTailCaptureRejection {
            reason: "source_changed",
            detail: format!(
                "exact committed-tail replay source changed to {}",
                source.source_id()
            ),
        });
    }
    let plan = plan_manual_toggle(ManualToggleRequest {
        visible_tail: VisibleTail::ime_committed_tail(&reply.1),
        current_layout_is_ru: reply.2,
        preserve_trailing_whitespace: true,
    })
    .ok_or_else(|| CommittedTailCaptureRejection {
        reason: "no_reversible_token",
        detail: "exact committed-tail snapshot has no reversible token".to_string(),
    })?;
    if plan.route != ManualToggleRoute::ImeCommittedTail {
        return Err(CommittedTailCaptureRejection {
            reason: "wrong_route",
            detail: "exact committed-tail planner selected the wrong route".to_string(),
        });
    }
    let expected_suffix = tail_chars(&reply.1, plan.backspaces as usize);
    let lease = tail_lease_from_v3_reply(
        VisibleTailSource::ImeCommittedTail,
        &expected_suffix,
        plan.backspaces,
        reply,
    )
    .map_err(|detail| CommittedTailCaptureRejection {
        reason: "lease_validation",
        detail,
    })?;
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
        personalization: None,
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

pub(super) fn observed_ibus_engine() -> Result<String, String> {
    observed_ibus_engine_on(&dbus_connection()?)
}

fn observed_ibus_engine_on(connection: &zbus::blocking::Connection) -> Result<String, String> {
    let reply = connection
        .call_method(
            Some(IME_DBUS_DEST),
            IME_DBUS_PATH,
            Some(IME_DBUS_INTERFACE),
            "ObservedIbusEngine",
            &(),
        )
        .map_err(|error| error.to_string())?;
    reply
        .body()
        .deserialize::<String>()
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

    #[test]
    fn mode_readback_rejects_unavailable_sentinels_and_accepts_real_private_decoders() {
        for source in ["passive:no-focus", "passive:unknown-context"] {
            for path in ["", "/", "invalid-path"] {
                assert!(input_mode_from_v3_reply(reply_v3(source, "", 0, path, "")).is_err());
            }
            for mode in [false, true] {
                let mut reply = reply_v3(
                    source,
                    "",
                    7,
                    "/io/github/radislabus_star/LayIme/engine/1",
                    "",
                );
                reply.2 = mode;
                assert_eq!(input_mode_from_v3_reply(reply), Ok(mode));
            }
        }
    }

    #[test]
    fn late_exact_receipt_stays_within_one_physical_gesture() {
        let mut calls = 0;
        let result = await_same_gesture_exact_receipt(|| {
            calls += 1;
            Ok(if calls < 3 {
                ImeManualToggleOutcome::AwaitingExactSnapshot
            } else {
                ImeManualToggleOutcome::DelegateExactImeTail
            })
        });

        assert_eq!(result, Ok(ImeManualToggleOutcome::DelegateExactImeTail));
        assert_eq!(calls, 3);
    }

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
    fn exact_committed_tail_capture_reports_first_validation_failure() {
        for (reply, reason, detail, source_kind) in [
            (
                reply_v3("passive:unknown-context", "", 0, "", ""),
                "source_unknown",
                "unknown IME tail source: passive:unknown-context",
                "unknown",
            ),
            (
                reply_v3("passive:daemon-word-buffer", "", 0, "", ""),
                "source_changed",
                "exact committed-tail replay source changed to daemon_word_buffer",
                "daemon_word_buffer",
            ),
            (
                reply_v3("active:composition", "", 0, "", ""),
                "source_changed",
                "exact committed-tail replay source changed to ime_active_composition",
                "ime_active_composition",
            ),
            (
                reply_v3("passive:committed-tail", "", 7, "", ""),
                "no_reversible_token",
                "exact committed-tail snapshot has no reversible token",
                "ime_committed_tail",
            ),
            (
                reply_v3("passive:committed-tail", "ghbdtn", 7, "", "field-a"),
                "lease_validation",
                "exact committed tail has no engine path or field focus receipt",
                "ime_committed_tail",
            ),
            (
                reply_v3("passive:committed-tail", "ghbdtn", 7, "/engine/us", ""),
                "lease_validation",
                "exact committed tail has no engine path or field focus receipt",
                "ime_committed_tail",
            ),
        ] {
            let rejection = committed_tail_replay_from_v3_reply(reply.clone()).unwrap_err();
            assert_eq!(rejection.reason, reason);
            assert_eq!(rejection.detail, detail);
            for (permission, permission_kind) in [
                (None, "unknown"),
                (Some(false), "denied"),
                (Some(true), "allowed"),
            ] {
                let diagnostic = rejection.diagnostic(
                    VisibleTailSource::from_bridge_state(&reply.0),
                    permission,
                    !reply.4.is_empty(),
                    !reply.5.is_empty(),
                );
                assert!(diagnostic.contains(&format!("reason={reason} ")));
                assert!(diagnostic.contains(&format!("source_kind={source_kind} ")));
                assert!(diagnostic.contains(&format!("permission={permission_kind} ")));
                let error =
                    committed_tail_replay_from_readout(reply.clone(), permission).unwrap_err();
                assert_eq!(
                    error,
                    if permission == Some(true) {
                        detail
                    } else {
                        "exact IME tail capture rejected; payload retention disabled"
                    }
                );
            }
        }
    }

    #[test]
    fn exact_committed_tail_capture_rejection_redacts_untrusted_reply_fields() {
        let reply = reply_v3(
            "SOURCE_PRIVATE_SENTINEL",
            "TAIL_PRIVATE_SENTINEL",
            7,
            "/ENGINE_PRIVATE_SENTINEL",
            "FIELD_PRIVATE_SENTINEL",
        );
        let rejection = committed_tail_replay_from_v3_reply(reply.clone()).unwrap_err();
        assert_eq!(rejection.reason, "source_unknown");
        for (permission, permission_kind) in [
            (None, "unknown"),
            (Some(false), "denied"),
            (Some(true), "allowed"),
        ] {
            let diagnostic = rejection.diagnostic(
                VisibleTailSource::from_bridge_state(&reply.0),
                permission,
                !reply.4.is_empty(),
                !reply.5.is_empty(),
            );
            assert_eq!(
                diagnostic,
                format!(
                    "· exact IME tail capture rejection: reason=source_unknown source_kind=unknown permission={permission_kind} engine_present=true field_receipt_present=true"
                )
            );
            let error = committed_tail_replay_from_readout(reply.clone(), permission).unwrap_err();
            assert_eq!(
                error,
                if permission == Some(true) {
                    "unknown IME tail source: SOURCE_PRIVATE_SENTINEL"
                } else {
                    "exact IME tail capture rejected; payload retention disabled"
                }
            );
            for sentinel in [
                "SOURCE_PRIVATE_SENTINEL",
                "TAIL_PRIVATE_SENTINEL",
                "ENGINE_PRIVATE_SENTINEL",
                "FIELD_PRIVATE_SENTINEL",
            ] {
                assert!(!diagnostic.contains(sentinel));
                if permission != Some(true) || sentinel != "SOURCE_PRIVATE_SENTINEL" {
                    assert!(!error.contains(sentinel));
                }
            }
        }
    }

    #[test]
    fn exact_private_tail_keeps_conversion_but_never_upgrades_retention() {
        for permission in [None, Some(false), Some(true)] {
            let replay = committed_tail_replay_from_readout(
                reply_v3(
                    "passive:committed-tail",
                    "prefix ghjdthrf ",
                    19,
                    "/engine/us",
                    "field-a",
                ),
                permission,
            )
            .expect("valid exact manual source");
            assert_eq!(replay.plan.replacement, "проверка ");
            assert_eq!(
                replay.lease.allows_personalization(),
                permission == Some(true)
            );
            if let Some(expected) = permission {
                for changed in [false, true] {
                    let base = reply_v3(
                        "passive:committed-tail",
                        "prefix ghjdthrf ",
                        19,
                        "/engine/us",
                        "field-a",
                    );
                    let current = (
                        base.0,
                        base.1,
                        base.2,
                        base.3,
                        base.4,
                        base.5,
                        if changed { !expected } else { expected },
                    );
                    let bound = replay.lease.bound_v4_reply(current, expected);
                    assert_eq!(bound.is_ok(), !changed);
                    if let Ok(reply) = bound {
                        assert!(validate_delegated_tail_v3_reply(
                            &replay.lease.snapshot,
                            replay.lease.expected_source_path(),
                            "field-a",
                            replay.plan.backspaces,
                            reply,
                            false,
                            false,
                        )
                        .is_ok());
                    }
                }
                let other_field = reply_v3(
                    "passive:committed-tail",
                    "prefix ghjdthrf ",
                    19,
                    "/engine/ru",
                    "field-b",
                );
                let current = (
                    other_field.0,
                    other_field.1,
                    true,
                    other_field.3,
                    other_field.4,
                    other_field.5,
                    expected,
                );
                let bound = replay.lease.bound_v4_reply(current, expected).unwrap();
                assert!(
                    validate_delegated_tail_v3_reply(
                        &replay.lease.snapshot,
                        replay.lease.expected_source_path(),
                        "field-a",
                        replay.plan.backspaces,
                        bound,
                        true,
                        true,
                    )
                    .is_err(),
                    "matching privacy cannot admit a different field"
                );
            }
            let safe = replay
                .lease
                .privacy_safe_error("private sentinel".to_string());
            assert_eq!(safe.contains("private sentinel"), permission == Some(true));
        }
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

#[cfg(test)]
mod observed_engine_rpc_tests {
    use super::*;
    struct Peer(std::sync::Arc<std::sync::Mutex<String>>);
    #[zbus::interface(name = "io.github.radislabus_star.LayIme")]
    impl Peer {
        #[zbus(name = "ObservedIbusEngine")]
        fn observed(&self) -> zbus::fdo::Result<String> {
            let value = self.0.lock().unwrap().clone();
            if value.is_empty() {
                Err(zbus::fdo::Error::Failed("backend unavailable".into()))
            } else {
                Ok(value)
            }
        }
    }
    struct LegacyPeer;
    #[zbus::interface(name = "io.github.radislabus_star.LayIme")]
    impl LegacyPeer {
        fn ping(&self) -> String {
            "old-IME".into()
        }
    }
    fn peer<T: zbus::object_server::Interface + Send + Sync + 'static>(
        service: T,
    ) -> (zbus::blocking::Connection, zbus::blocking::Connection) {
        let (server_stream, client_stream) = std::os::unix::net::UnixStream::pair().unwrap();
        let guid = zbus::Guid::generate();
        let server = std::thread::spawn(move || {
            zbus::blocking::connection::Builder::unix_stream(server_stream)
                .server(guid)
                .unwrap()
                .p2p()
                .serve_at(IME_DBUS_PATH, service)
                .unwrap()
                .build()
                .unwrap()
        });
        let client = zbus::blocking::connection::Builder::unix_stream(client_stream)
            .p2p()
            .method_timeout(Duration::from_millis(250))
            .build()
            .unwrap();
        (client, server.join().unwrap())
    }
    #[test]
    fn client_reads_fresh_rpc_and_propagates_backend_error_and_disconnect() {
        let value = std::sync::Arc::new(std::sync::Mutex::new("lay-ime-ru".to_string()));
        let (client, _server) = peer(Peer(value.clone()));
        assert_eq!(observed_ibus_engine_on(&client).unwrap(), "lay-ime-ru");
        *value.lock().unwrap() = "lay-ime-us".into();
        assert_eq!(observed_ibus_engine_on(&client).unwrap(), "lay-ime-us");
        value.lock().unwrap().clear();
        assert!(observed_ibus_engine_on(&client).is_err());
        zbus::block_on(client.inner().clone().close()).unwrap();
        assert!(observed_ibus_engine_on(&client).is_err());
    }
    #[test]
    fn client_reports_old_ime_unknown_method() {
        let (client, _server) = peer(LegacyPeer);
        let error = observed_ibus_engine_on(&client).unwrap_err();
        assert!(error.contains("UnknownMethod"), "{error}");
    }
}

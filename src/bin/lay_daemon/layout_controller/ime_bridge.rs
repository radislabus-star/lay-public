use lay::manual_toggle::ImeManualToggleOutcome;
use lay::text_backend::ImeReplaceRequest;
use lay::text_edit::{tail_chars, VisibleTailSnapshot, VisibleTailSource};
use serde::de::DeserializeOwned;
use zbus::zvariant::Type;

use super::super::{active_text_backend, log};
use super::gnome_dbus::dbus_connection;

const IME_DBUS_DEST: &str = "io.github.radislabus_star.LayIme";
const IME_DBUS_PATH: &str = "/io/github/radislabus_star/LayIme";
const IME_DBUS_INTERFACE: &str = "io.github.radislabus_star.LayIme";

type VisibleTailV2Reply = (String, String, bool, u64, String);

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ImeDelegatedTailLease {
    snapshot: VisibleTailSnapshot,
}

impl ImeDelegatedTailLease {
    pub(crate) fn validate_current(&self, backspaces: u32) -> Result<(), String> {
        validate_delegated_tail_reply(&self.snapshot, backspaces, visible_tail_v2()?, false)
    }

    pub(crate) fn validate_after_controlled_layout_handoff(
        &self,
        backspaces: u32,
    ) -> Result<(), String> {
        validate_delegated_tail_reply(&self.snapshot, backspaces, visible_tail_v2()?, true)
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

pub(super) fn capture_delegated_tail_lease(
    expected_suffix: &str,
    backspaces: u32,
) -> Result<ImeDelegatedTailLease, String> {
    delegated_tail_lease_from_reply(expected_suffix, backspaces, visible_tail_v2()?)
}

fn delegated_tail_lease_from_reply(
    expected_suffix: &str,
    backspaces: u32,
    reply: VisibleTailV2Reply,
) -> Result<ImeDelegatedTailLease, String> {
    let source = VisibleTailSource::from_bridge_state(&reply.0)
        .ok_or_else(|| format!("unknown IME tail source: {}", reply.0))?;
    if source != VisibleTailSource::DaemonWordBuffer {
        return Err(format!(
            "delegated tail source changed to {}",
            source.source_id()
        ));
    }
    if reply.4.is_empty() {
        return Err("delegated tail has no focus identity".to_string());
    }
    let snapshot =
        VisibleTailSnapshot::new(source, expected_suffix, Some(reply.4.clone()), reply.3);
    validate_delegated_tail_reply(&snapshot, backspaces, reply, false)?;
    Ok(ImeDelegatedTailLease { snapshot })
}

fn validate_delegated_tail_reply(
    snapshot: &VisibleTailSnapshot,
    backspaces: u32,
    reply: VisibleTailV2Reply,
    allow_controlled_focus_handoff: bool,
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
        return Err("delegated tail source/focus/epoch changed".to_string());
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
        )
        .is_ok());
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
            assert!(validate_delegated_tail_reply(&lease.snapshot, 3, invalid, false).is_err());
        }
        assert!(validate_delegated_tail_reply(
            &lease.snapshot,
            4,
            reply("passive:daemon-word-buffer", "prefix ytn", 7, "/focus/1"),
            false,
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
        )
        .is_ok());
        assert!(validate_delegated_tail_reply(
            &lease.snapshot,
            3,
            reply("passive:daemon-word-buffer", "prefix ytn", 8, "/engine/ru"),
            true,
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
}

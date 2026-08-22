use lay::text_backend::ImeReplaceRequest;
use lay::text_edit::tail_chars;
use serde::de::DeserializeOwned;
use zbus::zvariant::Type;

use super::super::{active_text_backend, log};
use super::gnome_dbus::dbus_connection;

const IME_DBUS_DEST: &str = "io.github.radislabus_star.LayIme";
const IME_DBUS_PATH: &str = "/io/github/radislabus_star/LayIme";
const IME_DBUS_INTERFACE: &str = "io.github.radislabus_star.LayIme";

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

fn visible_tail_v2() -> Result<(String, String, bool, u64, String), String> {
    call_ime_noarg("VisibleTailV2")
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

pub(super) fn manual_toggle() -> Result<(bool, bool), String> {
    call_ime_noarg("ManualToggleV2")
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

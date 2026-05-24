use lay::text_backend::ImeReplaceRequest;

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
    match replace_tail(request.backspaces, &request.text) {
        Ok(true) => {
            log(&format!(
                "  IME replace-tail ({kind}): bs={} insert={:?}",
                request.backspaces, request.text
            ));
            Ok(true)
        }
        Ok(false) => {
            log("⚠ IME replace-tail returned false; fallback to uinput");
            Ok(false)
        }
        Err(e) => {
            log(&format!(
                "⚠ IME replace-tail failed: {e}; fallback to uinput"
            ));
            Err(e)
        }
    }
}

pub(super) fn call_ping() -> Result<String, String> {
    let reply = dbus_connection()?
        .call_method(
            Some(IME_DBUS_DEST),
            IME_DBUS_PATH,
            Some(IME_DBUS_INTERFACE),
            "Ping",
            &(),
        )
        .map_err(|e| e.to_string())?;
    reply
        .body()
        .deserialize::<String>()
        .map_err(|e| e.to_string())
}

fn replace_tail(backspaces: u32, text: &str) -> Result<bool, String> {
    let reply = dbus_connection()?
        .call_method(
            Some(IME_DBUS_DEST),
            IME_DBUS_PATH,
            Some(IME_DBUS_INTERFACE),
            "ReplaceTail",
            &(backspaces, text),
        )
        .map_err(|e| e.to_string())?;
    reply
        .body()
        .deserialize::<bool>()
        .map_err(|e| e.to_string())
}

//! Native X11 XKB layout backend.
//!
//! X11 can switch keyboard groups directly through the XKB extension. This is
//! faster and less invasive than spawning `setxkbmap` for every correction.

use std::sync::Mutex;

use x11rb::connection::Connection;
use x11rb::protocol::xkb::ConnectionExt as _;
use x11rb::protocol::xproto::ModMask;
use x11rb::rust_connection::RustConnection;

const XKB_USE_CORE_KBD: u16 = 0x0100;
const X11_GROUP_US: u8 = 0;
const X11_GROUP_RU: u8 = 1;

struct Backend {
    conn: RustConnection,
}

impl Backend {
    fn connect() -> Result<Self, String> {
        let (conn, _) = RustConnection::connect(None).map_err(|e| format!("X11 connect: {e}"))?;
        conn.xkb_use_extension(1, 0)
            .map_err(|e| format!("xkb_use_extension send: {e}"))?
            .reply()
            .map_err(|e| format!("xkb_use_extension reply: {e}"))?;
        Ok(Self { conn })
    }

    fn current_group(&self) -> Result<u8, String> {
        let state = self
            .conn
            .xkb_get_state(XKB_USE_CORE_KBD)
            .map_err(|e| format!("xkb_get_state send: {e}"))?
            .reply()
            .map_err(|e| format!("xkb_get_state reply: {e}"))?;
        Ok(state.group.into())
    }

    fn lock_group(&self, group: u8) -> Result<(), String> {
        let no_mods = ModMask::from(0u16);
        self.conn
            .xkb_latch_lock_state(
                XKB_USE_CORE_KBD,
                no_mods,
                no_mods,
                true,
                group.into(),
                no_mods,
                false,
                0,
            )
            .map_err(|e| format!("xkb_latch_lock_state send: {e}"))?
            .check()
            .map_err(|e| format!("xkb_latch_lock_state check: {e}"))?;
        self.conn.flush().map_err(|e| format!("X11 flush: {e}"))?;
        Ok(())
    }
}

static BACKEND: Mutex<Option<Backend>> = Mutex::new(None);

fn with_backend<F, R>(f: F) -> Result<R, String>
where
    F: FnOnce(&Backend) -> Result<R, String>,
{
    let mut guard = BACKEND.lock().map_err(|e| e.to_string())?;
    if guard.is_none() {
        *guard = Some(Backend::connect()?);
    }

    let Some(backend) = guard.as_ref() else {
        return Err("X11 backend initialization failed".to_string());
    };
    let result = f(backend);
    if result.is_err() {
        *guard = None;
    }
    result
}

pub fn current_group() -> Result<u8, String> {
    with_backend(|backend| backend.current_group())
}

pub fn lock_group(group: u8) -> Result<(), String> {
    with_backend(|backend| backend.lock_group(group))
}

pub fn layout_for_group(group: u8) -> Option<&'static str> {
    match group {
        X11_GROUP_US => Some("us"),
        X11_GROUP_RU => Some("ru"),
        _ => None,
    }
}

pub fn group_for_layout(layout: &str) -> Option<u8> {
    match crate::desktop::normalize_layout_id(layout).as_str() {
        "us" | "en" => Some(X11_GROUP_US),
        "ru" => Some(X11_GROUP_RU),
        _ => None,
    }
}

pub fn current_layout_id() -> Result<String, String> {
    let group = current_group()?;
    layout_for_group(group)
        .map(str::to_string)
        .ok_or_else(|| format!("unexpected XKB group {group}"))
}

pub fn lock_layout_id(layout: &str) -> Result<(), String> {
    let group =
        group_for_layout(layout).ok_or_else(|| format!("unknown X11 layout id: {layout}"))?;
    lock_group(group)
}

pub fn ping() -> Result<String, String> {
    let group = current_group()?;
    let layout = layout_for_group(group).unwrap_or("unknown");
    Ok(format!(
        "X11 XKB ok, current_group={group}, layout={layout}"
    ))
}

#[cfg(test)]
#[path = "x11_layout_tests.rs"]
mod tests;

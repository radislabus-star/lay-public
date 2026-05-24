use std::process::Command;
use std::sync::{Mutex, OnceLock};

use super::super::log;

const DBUS_PATH: &str = "/io/github/radislabus_star/LayDaemon";
const DBUS_INTERFACE: &str = "io.github.radislabus_star.LayDaemon";
const DBUS_DEST: &str = "org.gnome.Shell";
static DBUS_CONNECTION: OnceLock<Mutex<Option<zbus::blocking::Connection>>> = OnceLock::new();

pub(crate) fn call_list_layouts() -> Result<String, String> {
    call_dbus_list_layouts().or_else(|fast_error| {
        reset_dbus_connection();
        log(&format!(
            "⚠ DBus fast ListLayouts failed: {fast_error}; fallback gdbus"
        ));
        run_gdbus(&format!("{DBUS_INTERFACE}.ListLayouts"), &[])
    })
}

pub(crate) fn parse_gdbus_string(reply: &str) -> Option<String> {
    let trimmed = reply.trim();
    let without_tuple = trimmed.strip_prefix("('")?.strip_suffix("',)")?;
    Some(without_tuple.replace("\\'", "'"))
}

pub(crate) fn parse_gdbus_bool(reply: &str) -> Option<bool> {
    let trimmed = reply.trim();
    match trimmed {
        "(true,)" => Some(true),
        "(false,)" => Some(false),
        _ => None,
    }
}

pub(crate) fn parse_current_layout_from_list(layouts: &str) -> Option<String> {
    let list = parse_gdbus_string(layouts).unwrap_or_else(|| layouts.to_string());
    list.split(',').find_map(|entry| {
        let current = entry.strip_suffix('*')?;
        current.rsplit(':').next().map(str::to_string)
    })
}

pub(crate) fn call_ping() -> Result<String, String> {
    call_dbus_ping().or_else(|fast_error| {
        reset_dbus_connection();
        log(&format!(
            "⚠ DBus fast Ping failed: {fast_error}; fallback gdbus"
        ));
        let reply = run_gdbus(&format!("{DBUS_INTERFACE}.Ping"), &[])?;
        parse_gdbus_string(&reply).ok_or_else(|| format!("не распарсил Ping: {reply}"))
    })
}

pub(crate) fn call_activate_layout(id: &str) -> Result<bool, String> {
    call_dbus_activate_layout(id).or_else(|fast_error| {
        reset_dbus_connection();
        log(&format!(
            "⚠ DBus fast ActivateLayout failed: {fast_error}; fallback gdbus"
        ));
        let reply = run_gdbus(
            &format!("{DBUS_INTERFACE}.ActivateLayout"),
            &[&format!("\"{id}\"")],
        )?;
        parse_gdbus_bool(&reply).ok_or_else(|| format!("не распарсил ActivateLayout: {reply}"))
    })
}

pub(crate) fn call_focused_window_info() -> Result<String, String> {
    call_dbus_focused_window_info().or_else(|fast_error| {
        reset_dbus_connection();
        log(&format!(
            "⚠ DBus fast FocusedWindowInfo failed: {fast_error}; fallback gdbus"
        ));
        let reply = run_gdbus(&format!("{DBUS_INTERFACE}.FocusedWindowInfo"), &[])?;
        parse_gdbus_string(&reply).ok_or_else(|| format!("не распарсил FocusedWindowInfo: {reply}"))
    })
}

pub(crate) fn call_replace_text(
    move_left: u32,
    backspaces: u32,
    text: &str,
    move_right: u32,
    layout_id: &str,
) -> Result<bool, String> {
    call_dbus_replace_text(move_left, backspaces, text, move_right, layout_id).or_else(
        |fast_error| {
            reset_dbus_connection();
            log(&format!(
                "⚠ DBus fast ReplaceText failed: {fast_error}; fallback gdbus"
            ));
            call_replace_text_gdbus(move_left, backspaces, text, move_right, layout_id)
        },
    )
}

pub(crate) fn call_current_layout() -> Result<String, String> {
    call_dbus_current_layout().or_else(|fast_error| {
        reset_dbus_connection();
        log(&format!(
            "⚠ DBus fast CurrentLayout failed: {fast_error}; fallback gdbus"
        ));
        let current = run_gdbus(&format!("{DBUS_INTERFACE}.CurrentLayout"), &[]);
        match current {
            Ok(reply) => parse_gdbus_string(&reply).ok_or_else(|| format!("не распарсил: {reply}")),
            Err(current_error) => {
                let layouts = call_list_layouts()
                    .map_err(|list_error| format!("{current_error}; ListLayouts: {list_error}"))?;
                parse_current_layout_from_list(&layouts)
                    .ok_or_else(|| format!("не нашёл текущую раскладку: {layouts}"))
            }
        }
    })
}

pub(crate) fn dbus_connection() -> Result<zbus::blocking::Connection, String> {
    let cell = DBUS_CONNECTION.get_or_init(|| Mutex::new(None));
    let mut guard = cell.lock().map_err(|e| e.to_string())?;
    if let Some(conn) = guard.as_ref() {
        return Ok(conn.clone());
    }

    let conn = zbus::blocking::Connection::session().map_err(|e| e.to_string())?;
    *guard = Some(conn.clone());
    Ok(conn)
}

fn reset_dbus_connection() {
    if let Some(cell) = DBUS_CONNECTION.get() {
        if let Ok(mut guard) = cell.lock() {
            *guard = None;
        }
    }
}

fn call_dbus_ping() -> Result<String, String> {
    let reply = dbus_connection()?
        .call_method(
            Some(DBUS_DEST),
            DBUS_PATH,
            Some(DBUS_INTERFACE),
            "Ping",
            &(),
        )
        .map_err(|e| e.to_string())?;
    reply
        .body()
        .deserialize::<String>()
        .map_err(|e| e.to_string())
}

fn call_dbus_replace_text(
    move_left: u32,
    backspaces: u32,
    text: &str,
    move_right: u32,
    layout_id: &str,
) -> Result<bool, String> {
    let reply = dbus_connection()?
        .call_method(
            Some(DBUS_DEST),
            DBUS_PATH,
            Some(DBUS_INTERFACE),
            "ReplaceText",
            &(move_left, backspaces, text, move_right, layout_id),
        )
        .map_err(|e| e.to_string())?;
    reply
        .body()
        .deserialize::<bool>()
        .map_err(|e| e.to_string())
}

fn call_dbus_activate_layout(id: &str) -> Result<bool, String> {
    let reply = dbus_connection()?
        .call_method(
            Some(DBUS_DEST),
            DBUS_PATH,
            Some(DBUS_INTERFACE),
            "ActivateLayout",
            &id,
        )
        .map_err(|e| e.to_string())?;
    reply
        .body()
        .deserialize::<bool>()
        .map_err(|e| e.to_string())
}

fn call_dbus_current_layout() -> Result<String, String> {
    let reply = dbus_connection()?
        .call_method(
            Some(DBUS_DEST),
            DBUS_PATH,
            Some(DBUS_INTERFACE),
            "CurrentLayout",
            &(),
        )
        .map_err(|e| e.to_string())?;
    reply
        .body()
        .deserialize::<String>()
        .map_err(|e| e.to_string())
}

fn call_dbus_list_layouts() -> Result<String, String> {
    let reply = dbus_connection()?
        .call_method(
            Some(DBUS_DEST),
            DBUS_PATH,
            Some(DBUS_INTERFACE),
            "ListLayouts",
            &(),
        )
        .map_err(|e| e.to_string())?;
    reply
        .body()
        .deserialize::<String>()
        .map_err(|e| e.to_string())
}

fn call_dbus_focused_window_info() -> Result<String, String> {
    let reply = dbus_connection()?
        .call_method(
            Some(DBUS_DEST),
            DBUS_PATH,
            Some(DBUS_INTERFACE),
            "FocusedWindowInfo",
            &(),
        )
        .map_err(|e| e.to_string())?;
    reply
        .body()
        .deserialize::<String>()
        .map_err(|e| e.to_string())
}

fn call_replace_text_gdbus(
    move_left: u32,
    backspaces: u32,
    text: &str,
    move_right: u32,
    layout_id: &str,
) -> Result<bool, String> {
    let text_arg = gvariant_string(text);
    let layout_arg = gvariant_string(layout_id);
    let reply = run_gdbus(
        &format!("{DBUS_INTERFACE}.ReplaceText"),
        &[
            &move_left.to_string(),
            &backspaces.to_string(),
            &text_arg,
            &move_right.to_string(),
            &layout_arg,
        ],
    )?;
    parse_gdbus_bool(&reply).ok_or_else(|| format!("не распарсил ReplaceText: {reply}"))
}

fn gvariant_string(text: &str) -> String {
    format!("{text:?}")
}

fn run_gdbus(method: &str, args: &[&str]) -> Result<String, String> {
    let mut cmd_args = vec![
        "call",
        "--session",
        "--dest",
        DBUS_DEST,
        "--object-path",
        DBUS_PATH,
        "--method",
        method,
    ];
    cmd_args.extend(args);
    let out = Command::new("gdbus")
        .args(&cmd_args)
        .output()
        .map_err(|e| e.to_string())?;
    if !out.status.success() {
        return Err(String::from_utf8_lossy(&out.stderr).trim().to_string());
    }
    Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
}

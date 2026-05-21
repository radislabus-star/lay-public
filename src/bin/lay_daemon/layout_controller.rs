use lay::desktop::{is_ru_layout_id, normalize_layout_id, parse_setxkbmap_layout, LayoutBackend};
use lay::text_backend::ImeReplaceRequest;
use std::process::Command;
use std::sync::{Mutex, OnceLock};
use std::time::Duration;

use super::{active_layout_backend, active_text_backend, log};

const DBUS_PATH: &str = "/io/github/radislabus_star/LayDaemon";
const DBUS_INTERFACE: &str = "io.github.radislabus_star.LayDaemon";
const DBUS_DEST: &str = "org.gnome.Shell";
const IME_DBUS_DEST: &str = "io.github.radislabus_star.LayIme";
const IME_DBUS_PATH: &str = "/io/github/radislabus_star/LayIme";
const IME_DBUS_INTERFACE: &str = "io.github.radislabus_star.LayIme";
static DBUS_CONNECTION: OnceLock<Mutex<Option<zbus::blocking::Connection>>> = OnceLock::new();
const LAYOUT_SWITCH_SETTLE_MS: u64 = 12;
const TRIGGER_RELEASE_SETTLE_MS: u64 = 80;

fn switch_ibus_engine(engine: &str) -> Result<(), String> {
    let out = Command::new("ibus")
        .args(["engine", engine])
        .output()
        .map_err(|e| e.to_string())?;
    if !out.status.success() {
        return Err(String::from_utf8_lossy(&out.stderr).trim().to_string());
    }
    Ok(())
}

fn read_ibus_engine() -> Result<String, String> {
    let out = Command::new("ibus")
        .arg("engine")
        .output()
        .map_err(|e| e.to_string())?;
    if !out.status.success() {
        return Err(String::from_utf8_lossy(&out.stderr).trim().to_string());
    }
    Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
}

pub(super) fn read_current_layout_is_ru() -> Result<bool, String> {
    match active_layout_backend() {
        LayoutBackend::Gnome => read_current_layout_gnome_is_ru(),
        LayoutBackend::Kde => read_current_layout_kde_is_ru(),
        LayoutBackend::X11 => read_current_layout_x11_is_ru(),
    }
}

fn read_current_layout_gnome_is_ru() -> Result<bool, String> {
    read_current_gnome_shell_layout_is_ru().or_else(|_| read_current_ibus_layout_is_ru())
}

fn read_current_gnome_shell_layout_is_ru() -> Result<bool, String> {
    call_current_layout().map(|id| is_ru_layout_id(&id))
}

fn read_current_ibus_layout_is_ru() -> Result<bool, String> {
    read_ibus_engine().map(|engine| is_ru_layout_id(&engine))
}

fn read_current_layout_kde_is_ru() -> Result<bool, String> {
    let qdbus = find_qdbus_command().ok_or_else(|| "qdbus/qdbus6 not found".to_string())?;
    let layout = read_current_kde_layout(qdbus)?;
    Ok(is_ru_layout_id(&layout))
}

fn read_current_layout_x11_is_ru() -> Result<bool, String> {
    let layout = read_x11_layout()?;
    Ok(is_ru_layout_id(&layout))
}

pub(super) fn call_list_layouts() -> Result<String, String> {
    call_dbus_list_layouts().or_else(|fast_error| {
        reset_dbus_connection();
        log(&format!(
            "⚠ DBus fast ListLayouts failed: {fast_error}; fallback gdbus"
        ));
        run_gdbus(&format!("{DBUS_INTERFACE}.ListLayouts"), &[])
    })
}

pub(super) fn parse_gdbus_string(reply: &str) -> Option<String> {
    let trimmed = reply.trim();
    let without_tuple = trimmed.strip_prefix("('")?.strip_suffix("',)")?;
    Some(without_tuple.replace("\\'", "'"))
}

pub(super) fn parse_gdbus_bool(reply: &str) -> Option<bool> {
    let trimmed = reply.trim();
    match trimmed {
        "(true,)" => Some(true),
        "(false,)" => Some(false),
        _ => None,
    }
}

pub(super) fn parse_current_layout_from_list(layouts: &str) -> Option<String> {
    let list = parse_gdbus_string(layouts).unwrap_or_else(|| layouts.to_string());
    list.split(',').find_map(|entry| {
        let current = entry.strip_suffix('*')?;
        current.rsplit(':').next().map(str::to_string)
    })
}

// ─── uinput re-typing ──────────────────────────────────────

// ─── DBus и ibus ────────────────────────────────────────────

pub(super) fn call_ping() -> Result<String, String> {
    call_dbus_ping().or_else(|fast_error| {
        reset_dbus_connection();
        log(&format!(
            "⚠ DBus fast Ping failed: {fast_error}; fallback gdbus"
        ));
        let reply = run_gdbus(&format!("{DBUS_INTERFACE}.Ping"), &[])?;
        parse_gdbus_string(&reply).ok_or_else(|| format!("не распарсил Ping: {reply}"))
    })
}

fn call_activate_layout(id: &str) -> Result<bool, String> {
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

pub(super) fn call_focused_window_info() -> Result<String, String> {
    if active_layout_backend() != LayoutBackend::Gnome {
        return Err("FocusedWindowInfo is available only through the GNOME backend".to_string());
    }
    call_dbus_focused_window_info().or_else(|fast_error| {
        reset_dbus_connection();
        log(&format!(
            "⚠ DBus fast FocusedWindowInfo failed: {fast_error}; fallback gdbus"
        ));
        let reply = run_gdbus(&format!("{DBUS_INTERFACE}.FocusedWindowInfo"), &[])?;
        parse_gdbus_string(&reply).ok_or_else(|| format!("не распарсил FocusedWindowInfo: {reply}"))
    })
}

fn switch_to_layout(layout_id: &str, ibus_engine: &str, target_is_ru: bool) -> Result<(), String> {
    match active_layout_backend() {
        LayoutBackend::Gnome => switch_to_gnome_layout(layout_id, ibus_engine, target_is_ru),
        LayoutBackend::Kde => switch_to_kde_layout(layout_id, target_is_ru),
        LayoutBackend::X11 => switch_to_x11_layout(layout_id, target_is_ru),
    }
}

fn switch_to_gnome_layout(
    layout_id: &str,
    ibus_engine: &str,
    target_is_ru: bool,
) -> Result<(), String> {
    let activate_error = match call_activate_layout(layout_id) {
        Ok(true) => {
            if verify_gnome_shell_layout(target_is_ru) {
                None
            } else {
                Some("ActivateLayout returned true but layout verify failed".to_string())
            }
        }
        Ok(false) => Some("ActivateLayout returned false".to_string()),
        Err(error) => Some(error),
    };

    let ibus_error = ensure_ibus_engine(ibus_engine, target_is_ru).err();
    if verify_gnome_shell_layout(target_is_ru) {
        if let Some(error) = ibus_error {
            log(&format!(
                "⚠ SetGlobalEngine refresh failed, GNOME Shell layout verified: {error}"
            ));
        }
        return Ok(());
    }

    if verify_gnome_layout_stack(target_is_ru) {
        if let Some(error) = activate_error {
            log(&format!(
                "⚠ ActivateLayout failed, ibus layout verified: {error}"
            ));
        }
        return Ok(());
    }

    Err(match (activate_error, ibus_error) {
        (Some(activate), Some(ibus)) => {
            format!("ActivateLayout failed: {activate}; SetGlobalEngine failed: {ibus}; layout verify failed")
        }
        (Some(activate), None) => {
            format!("ActivateLayout failed: {activate}; layout verify failed")
        }
        (None, Some(ibus)) => format!("SetGlobalEngine failed: {ibus}; layout verify failed"),
        (None, None) => "layout verify failed".to_string(),
    })
}

fn switch_to_kde_layout(layout_id: &str, target_is_ru: bool) -> Result<(), String> {
    let qdbus = find_qdbus_command().ok_or_else(|| "qdbus/qdbus6 not found".to_string())?;
    match kde_layout_index(qdbus, layout_id) {
        Ok(index) => {
            let index = index.to_string();
            run_command_capture(
                qdbus,
                &["org.kde.keyboard", "/Layouts", "setLayout", &index],
            )?;
        }
        Err(index_error) => {
            log(&format!(
                "⚠ KDE indexed layout lookup failed ({index_error}); trying legacy setLayout"
            ));
            run_command_capture(
                qdbus,
                &["org.kde.keyboard", "/Layouts", "setLayout", layout_id],
            )?;
        }
    }
    if verify_current_layout(target_is_ru) {
        Ok(())
    } else {
        Err("KDE layout verify failed".to_string())
    }
}

fn read_current_kde_layout(qdbus: &str) -> Result<String, String> {
    if let Ok(index) = run_command_capture(qdbus, &["org.kde.keyboard", "/Layouts", "getLayout"]) {
        let index = index
            .trim()
            .parse::<usize>()
            .map_err(|e| format!("cannot parse KDE layout index {index:?}: {e}"))?;
        let layouts = kde_layout_ids(qdbus)?;
        return layouts
            .get(index)
            .cloned()
            .ok_or_else(|| format!("KDE layout index {index} out of range: {layouts:?}"));
    }

    run_command_capture(qdbus, &["org.kde.keyboard", "/Layouts", "getCurrentLayout"])
        .map(|layout| normalize_layout_id(&layout))
}

fn kde_layout_index(qdbus: &str, layout_id: &str) -> Result<usize, String> {
    let target = normalize_layout_id(layout_id);
    let layouts = kde_layout_ids(qdbus)?;
    layouts
        .iter()
        .position(|layout| normalize_layout_id(layout) == target)
        .ok_or_else(|| format!("KDE layout {target:?} not found in {layouts:?}"))
}

fn kde_layout_ids(qdbus: &str) -> Result<Vec<String>, String> {
    let output = run_command_capture(
        qdbus,
        &[
            "--literal",
            "org.kde.keyboard",
            "/Layouts",
            "getLayoutsList",
        ],
    )?;
    let layouts = parse_kde_layouts_list(&output);
    if layouts.is_empty() {
        Err(format!("cannot parse KDE layouts: {output}"))
    } else {
        Ok(layouts)
    }
}

pub(super) fn parse_kde_layouts_list(output: &str) -> Vec<String> {
    output
        .split("(sss)")
        .skip(1)
        .filter_map(|entry| first_quoted_string(entry).map(|layout| normalize_layout_id(&layout)))
        .collect()
}

pub(super) fn first_quoted_string(input: &str) -> Option<String> {
    let mut chars = input.chars();
    for ch in chars.by_ref() {
        if ch == '"' {
            break;
        }
    }

    let mut out = String::new();
    let mut escaped = false;
    for ch in chars {
        if escaped {
            out.push(ch);
            escaped = false;
            continue;
        }
        match ch {
            '\\' => escaped = true,
            '"' => return Some(out),
            _ => out.push(ch),
        }
    }
    None
}

fn switch_to_x11_layout(layout_id: &str, target_is_ru: bool) -> Result<(), String> {
    if let Err(native_error) = lay::x11_layout::lock_layout_id(layout_id) {
        log(&format!(
            "⚠ native X11 XKB layout switch failed: {native_error}; fallback shell tools"
        ));
    } else if verify_current_layout(target_is_ru) {
        return Ok(());
    } else {
        log("⚠ native X11 XKB layout verify failed; fallback shell tools");
    }

    if command_exists("xkb-switch") {
        run_command_capture("xkb-switch", &["-s", layout_id])?;
    } else {
        run_command_capture("setxkbmap", &[layout_id])?;
    }

    if verify_current_layout(target_is_ru) {
        Ok(())
    } else {
        Err("X11 layout verify failed".to_string())
    }
}

pub(super) fn switch_to_target_layout(target_is_ru: bool) -> Result<&'static str, String> {
    let (layout_id, ibus_engine) = target_layout(target_is_ru);
    if active_layout_backend() != LayoutBackend::Gnome
        && read_current_layout_is_ru().is_ok_and(|current| current == target_is_ru)
    {
        return Ok(layout_id);
    }
    switch_to_layout(layout_id, ibus_engine, target_is_ru).map(|()| {
        settle_after_layout_switch();
        layout_id
    })
}

pub(super) fn target_layout(target_is_ru: bool) -> (&'static str, &'static str) {
    if target_is_ru {
        (
            "ru",
            if active_text_backend().should_try_ime() {
                "lay-ime-ru"
            } else {
                "xkb:ru::rus"
            },
        )
    } else {
        (
            "us",
            if active_text_backend().should_try_ime() {
                "lay-ime-us"
            } else {
                "xkb:us::eng"
            },
        )
    }
}

fn verify_current_layout(target_is_ru: bool) -> bool {
    for _ in 0..5 {
        if read_current_layout_is_ru().is_ok_and(|current| current == target_is_ru) {
            return true;
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    false
}

fn verify_gnome_shell_layout(target_is_ru: bool) -> bool {
    for _ in 0..5 {
        if read_current_gnome_shell_layout_is_ru().is_ok_and(|current| current == target_is_ru) {
            return true;
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    false
}

fn verify_gnome_layout_stack(target_is_ru: bool) -> bool {
    for _ in 0..5 {
        if verify_gnome_layout_stack_once(target_is_ru) {
            return true;
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    false
}

fn verify_gnome_layout_stack_once(target_is_ru: bool) -> bool {
    read_current_gnome_shell_layout_is_ru().is_ok_and(|current| current == target_is_ru)
        && read_current_ibus_layout_is_ru().is_ok_and(|current| current == target_is_ru)
}

fn ensure_ibus_engine(ibus_engine: &str, target_is_ru: bool) -> Result<(), String> {
    let already_target =
        read_current_ibus_layout_is_ru().is_ok_and(|current| current == target_is_ru);
    if let Err(error) = switch_ibus_engine(ibus_engine) {
        if already_target {
            log(&format!(
                "⚠ IBus refresh failed but engine is already target: {error}"
            ));
            return Ok(());
        }
        return Err(error);
    }
    if read_current_ibus_layout_is_ru().is_ok_and(|current| current == target_is_ru) {
        Ok(())
    } else {
        Err("IBus engine verify failed".to_string())
    }
}

fn settle_after_layout_switch() {
    std::thread::sleep(Duration::from_millis(LAYOUT_SWITCH_SETTLE_MS));
}

pub(super) fn settle_after_physical_trigger_release() {
    std::thread::sleep(Duration::from_millis(TRIGGER_RELEASE_SETTLE_MS));
}

pub(super) fn call_replace_text(
    move_left: u32,
    backspaces: u32,
    text: &str,
    move_right: u32,
    layout_id: &str,
) -> Result<bool, String> {
    if active_layout_backend() != LayoutBackend::Gnome {
        return Err("ReplaceText is available only through the GNOME backend".to_string());
    }
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

pub(super) fn should_try_ime_text_backend() -> bool {
    active_text_backend().should_try_ime()
}

pub(super) fn try_ime_replace_tail(
    original: &str,
    replacement: &str,
    kind: &str,
) -> Result<bool, String> {
    if !should_try_ime_text_backend() {
        return Ok(false);
    }
    let request = ImeReplaceRequest::committed_tail(original, replacement);
    if request.is_noop() {
        return Ok(false);
    }
    match call_ime_replace_tail(request.backspaces, &request.text) {
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

pub(super) fn call_ime_ping() -> Result<String, String> {
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

fn call_ime_replace_tail(backspaces: u32, text: &str) -> Result<bool, String> {
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

fn dbus_connection() -> Result<zbus::blocking::Connection, String> {
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

fn call_current_layout() -> Result<String, String> {
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

fn run_command_capture(command: &str, args: &[&str]) -> Result<String, String> {
    let out = Command::new(command)
        .args(args)
        .output()
        .map_err(|e| format!("{command}: {e}"))?;
    if !out.status.success() {
        return Err(format!(
            "{command}: {}",
            String::from_utf8_lossy(&out.stderr).trim()
        ));
    }
    Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
}

fn command_exists(command: &str) -> bool {
    let Some(paths) = std::env::var_os("PATH") else {
        return false;
    };
    std::env::split_paths(&paths).any(|dir| dir.join(command).is_file())
}

fn find_qdbus_command() -> Option<&'static str> {
    ["qdbus6", "qdbus-qt6", "qdbus"]
        .into_iter()
        .find(|cmd| command_exists(cmd))
}

pub(super) fn detect_auto_layout_backend_hint() -> Option<LayoutBackend> {
    let qdbus = find_qdbus_command()?;
    if run_command_capture(qdbus, &["org.kde.keyboard", "/Layouts", "getLayout"]).is_ok()
        || run_command_capture(qdbus, &["org.kde.keyboard", "/Layouts", "getCurrentLayout"]).is_ok()
    {
        return Some(LayoutBackend::Kde);
    }
    None
}

fn read_x11_layout() -> Result<String, String> {
    if let Ok(layout) = lay::x11_layout::current_layout_id() {
        return Ok(layout);
    }

    if command_exists("xkb-switch") {
        return run_command_capture("xkb-switch", &[]).map(|layout| normalize_layout_id(&layout));
    }
    if command_exists("xkblayout-state") {
        return run_command_capture("xkblayout-state", &["print", "%s"])
            .map(|layout| normalize_layout_id(&layout));
    }

    let query = run_command_capture("setxkbmap", &["-query"])?;
    parse_setxkbmap_layout(&query).ok_or_else(|| format!("cannot parse setxkbmap output: {query}"))
}

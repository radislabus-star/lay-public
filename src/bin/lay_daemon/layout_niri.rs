use niri_ipc::{socket::Socket, Action, LayoutSwitchTarget, Request, Response};

/// Connect to the niri IPC socket and send a request.
fn niri_send(request: Request) -> Result<Response, String> {
    let mut socket = Socket::connect().map_err(|e| format!("niri IPC connect failed: {e}"))?;
    socket
        .send(request)
        .map_err(|e| format!("niri IPC send failed: {e}"))
        .and_then(|reply| reply.map_err(|msg| format!("niri replied with error: {msg}")))
}

/// Get current keyboard layouts from niri via direct IPC.
fn fetch_layouts() -> Result<niri_ipc::KeyboardLayouts, String> {
    match niri_send(Request::KeyboardLayouts)? {
        Response::KeyboardLayouts(kl) => Ok(kl),
        other => Err(format!("unexpected response: {other:?}")),
    }
}

/// Check if a layout name represents Russian layout.
fn is_ru_layout_name(name: &str) -> bool {
    name == "Russian"
}

/// Read current keyboard layout from niri via direct IPC socket.
pub(super) fn read_current_layout_is_ru() -> Result<bool, String> {
    let layouts = fetch_layouts()?;
    let current_name = layouts.names.get(layouts.current_idx as usize).map(|s| s.as_str()).unwrap_or("");
    Ok(is_ru_layout_name(current_name))
}

/// Switch to target layout via direct IPC socket.
pub(super) fn switch_to_layout(_layout_id: &str, target_is_ru: bool) -> Result<(), String> {
    let layouts = fetch_layouts()?;

    let target_idx = layouts
        .names
        .iter()
        .position(|name| is_ru_layout_name(name) == target_is_ru)
        .ok_or_else(|| {
            let found = layouts.names.join(", ");
            format!("Russian/English layout not found in niri config (available: {found})")
        })?;

    if layouts.current_idx as usize == target_idx {
        return Ok(());
    }

    let action_response = niri_send(Request::Action(Action::SwitchLayout {
        layout: LayoutSwitchTarget::Index(target_idx as u8),
    }))?;
    match action_response {
        Response::Handled => {}
        other => return Err(format!("switch-layout unexpected response: {other:?}")),
    }

    Ok(())
}

/// Ping niri IPC to check if it's available.
pub(super) fn ping() -> Result<String, String> {
    let layouts = fetch_layouts()?;
    let current_name = layouts.names.get(layouts.current_idx as usize).map(|s| s.as_str()).unwrap_or("unknown");
    Ok(format!("niri layouts: {:?}, current: {}", layouts.names, current_name))
}

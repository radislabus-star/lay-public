use niri_ipc::{socket::Socket, Action, KeyboardLayouts, LayoutSwitchTarget, Request, Response};

use super::layout_controller::verify_current_layout;

pub(super) fn read_current_layout_is_ru() -> Result<bool, String> {
    let layouts = fetch_layouts()?;
    let current_name = current_layout_name(&layouts).ok_or_else(|| {
        format!(
            "Niri current layout index {} out of range: {:?}",
            layouts.current_idx, layouts.names
        )
    })?;
    Ok(is_russian_layout_name(current_name))
}

pub(super) fn switch_to_layout(_layout_id: &str, target_is_ru: bool) -> Result<(), String> {
    let layouts = fetch_layouts()?;
    let target_idx = choose_layout_index(&layouts.names, target_is_ru).ok_or_else(|| {
        format!(
            "Niri RU/EN layout not found; available layouts: {:?}",
            layouts.names
        )
    })?;

    if layouts.current_idx as usize == target_idx {
        return Ok(());
    }

    match niri_send(Request::Action(Action::SwitchLayout {
        layout: LayoutSwitchTarget::Index(target_idx as u8),
    }))? {
        Response::Handled => {}
        other => return Err(format!("Niri switch-layout unexpected response: {other:?}")),
    }

    if verify_current_layout(target_is_ru) {
        Ok(())
    } else {
        Err("Niri layout verify failed".to_string())
    }
}

pub(super) fn ping() -> Result<String, String> {
    let layouts = fetch_layouts()?;
    let current = current_layout_name(&layouts).unwrap_or("unknown");
    Ok(format!("layouts={:?}, current={current}", layouts.names))
}

fn fetch_layouts() -> Result<KeyboardLayouts, String> {
    match niri_send(Request::KeyboardLayouts)? {
        Response::KeyboardLayouts(layouts) => Ok(layouts),
        other => Err(format!("Niri keyboard-layouts unexpected response: {other:?}")),
    }
}

fn niri_send(request: Request) -> Result<Response, String> {
    let mut socket = Socket::connect().map_err(|e| format!("Niri IPC connect failed: {e}"))?;
    socket
        .send(request)
        .map_err(|e| format!("Niri IPC send failed: {e}"))?
        .map_err(|msg| format!("Niri IPC error: {msg}"))
}

fn current_layout_name(layouts: &KeyboardLayouts) -> Option<&str> {
    layouts
        .names
        .get(layouts.current_idx as usize)
        .map(String::as_str)
}

fn choose_layout_index(names: &[String], target_is_ru: bool) -> Option<usize> {
    if target_is_ru {
        return names
            .iter()
            .position(|name| is_russian_layout_name(name));
    }

    names
        .iter()
        .position(|name| is_english_layout_name(name))
        .or_else(|| {
            names
                .iter()
                .position(|name| !is_russian_layout_name(name))
        })
}

fn is_russian_layout_name(name: &str) -> bool {
    let normalized = normalize_layout_name(name);
    normalized == "ru"
        || normalized.contains("russian")
        || normalized.contains("рус")
        || normalized.contains("russia")
}

fn is_english_layout_name(name: &str) -> bool {
    let normalized = normalize_layout_name(name);
    normalized == "us"
        || normalized == "en"
        || normalized.contains("english")
        || normalized.contains("англ")
}

fn normalize_layout_name(name: &str) -> String {
    name.trim().to_lowercase()
}

#[cfg(test)]
mod tests {
    use super::choose_layout_index;

    fn names(values: &[&str]) -> Vec<String> {
        values.iter().map(|value| value.to_string()).collect()
    }

    #[test]
    fn chooses_russian_and_english_layouts_by_name() {
        let layouts = names(&["English (US)", "Russian"]);
        assert_eq!(choose_layout_index(&layouts, true), Some(1));
        assert_eq!(choose_layout_index(&layouts, false), Some(0));
    }

    #[test]
    fn chooses_ru_and_us_short_layout_names() {
        let layouts = names(&["us", "ru"]);
        assert_eq!(choose_layout_index(&layouts, true), Some(1));
        assert_eq!(choose_layout_index(&layouts, false), Some(0));
    }

    #[test]
    fn english_target_falls_back_to_first_non_russian_layout() {
        let layouts = names(&["Deutsch", "Русская"]);
        assert_eq!(choose_layout_index(&layouts, true), Some(1));
        assert_eq!(choose_layout_index(&layouts, false), Some(0));
    }
}

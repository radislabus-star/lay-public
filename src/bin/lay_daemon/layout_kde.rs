use lay::desktop::{normalize_layout_id, LayoutBackend};

use super::command_runtime::{command_exists, run_command_capture};
use super::layout_controller::verify_current_layout;
use super::log;

pub(super) fn read_current_layout_is_ru() -> Result<bool, String> {
    let qdbus = find_qdbus_command().ok_or_else(|| "qdbus/qdbus6 not found".to_string())?;
    let layout = read_current_layout(qdbus)?;
    Ok(lay::desktop::is_ru_layout_id(&layout))
}

pub(super) fn switch_to_layout(layout_id: &str, target_is_ru: bool) -> Result<(), String> {
    let qdbus = find_qdbus_command().ok_or_else(|| "qdbus/qdbus6 not found".to_string())?;
    match layout_index(qdbus, layout_id) {
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

pub(super) fn detect_auto_backend_hint() -> Option<LayoutBackend> {
    let qdbus = find_qdbus_command()?;
    if run_command_capture(qdbus, &["org.kde.keyboard", "/Layouts", "getLayout"]).is_ok()
        || run_command_capture(qdbus, &["org.kde.keyboard", "/Layouts", "getCurrentLayout"]).is_ok()
    {
        return Some(LayoutBackend::Kde);
    }
    None
}

fn read_current_layout(qdbus: &str) -> Result<String, String> {
    if let Ok(index) = run_command_capture(qdbus, &["org.kde.keyboard", "/Layouts", "getLayout"]) {
        let index = index
            .trim()
            .parse::<usize>()
            .map_err(|e| format!("cannot parse KDE layout index {index:?}: {e}"))?;
        let layouts = layout_ids(qdbus)?;
        return layouts
            .get(index)
            .cloned()
            .ok_or_else(|| format!("KDE layout index {index} out of range: {layouts:?}"));
    }

    run_command_capture(qdbus, &["org.kde.keyboard", "/Layouts", "getCurrentLayout"])
        .map(|layout| normalize_layout_id(&layout))
}

fn layout_index(qdbus: &str, layout_id: &str) -> Result<usize, String> {
    let target = normalize_layout_id(layout_id);
    let layouts = layout_ids(qdbus)?;
    layouts
        .iter()
        .position(|layout| normalize_layout_id(layout) == target)
        .ok_or_else(|| format!("KDE layout {target:?} not found in {layouts:?}"))
}

fn layout_ids(qdbus: &str) -> Result<Vec<String>, String> {
    let output = run_command_capture(
        qdbus,
        &[
            "--literal",
            "org.kde.keyboard",
            "/Layouts",
            "getLayoutsList",
        ],
    )?;
    let layouts = parse_layouts_list(&output);
    if layouts.is_empty() {
        Err(format!("cannot parse KDE layouts: {output}"))
    } else {
        Ok(layouts)
    }
}

pub(super) fn parse_layouts_list(output: &str) -> Vec<String> {
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

fn find_qdbus_command() -> Option<&'static str> {
    ["qdbus6", "qdbus-qt6", "qdbus"]
        .into_iter()
        .find(|cmd| command_exists(cmd))
}

use super::{ime_bridge, switch_to_target_layout};

pub(super) fn try_manual_toggle(ime_enabled: bool) -> Result<Option<bool>, String> {
    if !ime_enabled {
        return Ok(None);
    }
    let (handled, target_layout_is_ru) = ime_bridge::manual_toggle()?;
    if !handled {
        return Ok(None);
    }
    switch_to_target_layout(target_layout_is_ru)?;
    Ok(Some(target_layout_is_ru))
}

#[cfg(test)]
mod tests {
    #[test]
    fn daemon_delegates_the_physical_trigger_to_the_focused_ime_owner() {
        let source = include_str!("ime_manual_toggle.rs");
        assert!(source.contains("ime_bridge::manual_toggle()"));
        assert!(source.contains("switch_to_target_layout(target_layout_is_ru)"));
    }
}

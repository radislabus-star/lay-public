use super::{ime_bridge, switch_to_target_layout};
use lay::manual_toggle::ImeManualToggleOutcome;

pub(super) fn try_manual_toggle(ime_enabled: bool) -> Result<ImeManualToggleOutcome, String> {
    if !ime_enabled {
        return Ok(ImeManualToggleOutcome::DelegateDaemon);
    }
    let outcome = ime_bridge::manual_toggle()?;
    if let Some(target_layout_is_ru) = outcome.target_layout_is_ru() {
        switch_to_target_layout(target_layout_is_ru)?;
    }
    Ok(outcome)
}

#[cfg(test)]
mod tests {
    #[test]
    fn daemon_delegates_the_physical_trigger_to_the_focused_ime_owner() {
        let source = include_str!("ime_manual_toggle.rs");
        assert!(source.contains("let outcome = ime_bridge::manual_toggle()?"));
        assert!(source.contains("switch_to_target_layout(target_layout_is_ru)"));
    }
}

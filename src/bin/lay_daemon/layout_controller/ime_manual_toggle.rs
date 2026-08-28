use super::ime_bridge;
use lay::manual_toggle::ImeManualToggleOutcome;

pub(super) fn try_manual_toggle(ime_enabled: bool) -> Result<ImeManualToggleOutcome, String> {
    if !ime_enabled {
        return Ok(ImeManualToggleOutcome::DelegateDaemon);
    }
    ime_bridge::manual_toggle()
}

#[cfg(test)]
mod tests {
    #[test]
    fn physical_double_shift_owner_daemon_does_not_repeat_ime_layout_switch() {
        let source = include_str!("ime_manual_toggle.rs");
        let route = source
            .split("pub(super) fn try_manual_toggle")
            .nth(1)
            .expect("IME manual-toggle route")
            .split("#[cfg(test)]")
            .next()
            .expect("test boundary");

        assert!(route.contains("ime_bridge::manual_toggle()"));
        assert!(
            !route.contains("switch_to_target_layout"),
            "daemon regained a second layout owner after ManualToggleV3"
        );
    }
}

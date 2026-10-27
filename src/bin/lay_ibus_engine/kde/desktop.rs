use crate::trace;

#[path = "desktop/window_identity.rs"]
mod window_identity;
pub(in crate::window_interaction) use window_identity::{
    native_window_continues, same_native_window,
};

pub(super) fn focused_window_until(deadline: std::time::Instant) -> Option<super::FocusedWindow> {
    observe_focused_window_until(deadline).ok()
}

pub(in crate::window_interaction) fn observe_focused_window_until(
    deadline: std::time::Instant,
) -> Result<super::FocusedWindow, &'static str> {
    let started = std::time::Instant::now();
    // IME cleanup stays inside its key callback deadline, as before.
    let observation = match crate::kde_focus::read_with_deadline(deadline, false) {
        Ok(observation) => observation,
        Err(error) => {
            trace::record(
                serde_json::json!({
                    "kind": "ibus_native_desktop_probe",
                    "status": "refused",
                    "reason": match &error {
                        crate::kde_focus::ReadError::Busy => "provider_busy",
                        crate::kde_focus::ReadError::Unavailable(_) => "provider_unavailable",
                    },
                    "detail": error.to_string(),
                    "elapsed_us": started.elapsed().as_micros(),
                })
                .to_string(),
            );
            // This is an unavailable observation, not a successfully observed
            // different window. It cannot supply native text/edit evidence.
            return Err("native_desktop_unobserved");
        }
    };
    super::FocusedWindow::from_info(observation.identity, &observation.info)
        .ok_or("native_no_desktop")
}

pub(in crate::window_interaction) fn exact_kde_terminal_window(
    observation: &crate::kde_focus::Observation,
) -> bool {
    // The canonical provider already authenticates KWin's owner, nonce,
    // lifetime and UUID. Its identity is not a title or a client hint.
    if observation.identity.is_empty() {
        return false;
    }
    let Ok(window) = serde_json::from_str::<serde_json::Value>(&observation.info) else {
        return false;
    };
    matches!(
        (
            window.get("appId").and_then(|v| v.as_str()),
            window.get("wmClass").and_then(|v| v.as_str()),
        ),
        // KWin can leave desktopFileName empty. A conflicting ID fails closed.
        (Some("" | "kitty" | "kitty.desktop"), Some("kitty"))
            | (
                Some("" | "konsole" | "org.kde.konsole" | "org.kde.konsole.desktop"),
                Some("konsole")
            )
    )
}

pub(super) fn focused_window_is_native_terminal() -> bool {
    match crate::kde_focus::read() {
        Ok(observation) => {
            let matches = exact_kde_terminal_window(&observation);
            trace::record(format!(
                r#"{{"kind":"ibus_terminal_focus_probe","source":"kwin","matches":{matches}}}"#,
            ));
            matches
        }
        Err(error) => {
            trace::record(
                serde_json::json!({
                    "kind": "ibus_terminal_focus_probe",
                    "source": "kwin",
                    "error": error.to_string(),
                })
                .to_string(),
            );
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unavailable_observation_keeps_the_option_contract() {
        crate::kde_focus::with_occupied_provider_for_test(|| {
            let deadline = std::time::Instant::now() + std::time::Duration::from_millis(20);
            assert_eq!(
                observe_focused_window_until(deadline).err(),
                Some("native_desktop_unobserved")
            );
            assert!(focused_window_until(deadline).is_none());
        });
    }
}

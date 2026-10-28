//! KDE terminal qualification; actual input/edit authority stays common.
use crate::trace;

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

pub(in crate::window_interaction::desktop) fn focused_window_is_native_terminal() -> bool {
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

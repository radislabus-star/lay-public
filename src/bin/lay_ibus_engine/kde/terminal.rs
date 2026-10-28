//! KDE terminal qualification; actual input/edit authority stays common.
use crate::trace;

pub(in crate::window_interaction) fn native_terminal_focus_purpose_supported(purpose: u32) -> bool {
    // GTK/VTE reports TERMINAL even when it advertises surrounding text.
    // This only permits the existing exact current-field focus probe.
    purpose == 0 || purpose == crate::window_interaction::IBUS_INPUT_PURPOSE_TERMINAL
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
            | (
                Some("" | "org.gnome.Terminal" | "org.gnome.Terminal.desktop"),
                Some("gnome-terminal")
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

#[cfg(test)]
mod tests {
    use super::exact_kde_terminal_window;

    fn observed(app: &str, class: &str) -> crate::kde_focus::Observation {
        crate::kde_focus::Observation {
            identity: "current-kwin-window".into(),
            info: serde_json::json!({"appId": app, "wmClass": class, "pid": 42}).to_string(),
        }
    }

    #[test]
    fn gtk_vte_qualification_accepts_coherent_terminal_metadata() {
        for app in ["", "org.gnome.Terminal", "org.gnome.Terminal.desktop"] {
            assert!(exact_kde_terminal_window(&observed(app, "gnome-terminal")));
        }
    }

    #[test]
    fn gtk_vte_qualification_keeps_other_clients_and_empty_receipts_closed() {
        for app in [
            "kitty",
            "org.kde.konsole",
            "firefox",
            "google-chrome.desktop",
        ] {
            assert!(!exact_kde_terminal_window(&observed(app, "gnome-terminal")));
        }
        for class in [
            "",
            "gnome-terminal-server",
            "Gnome-terminal",
            "terminal",
            "firefox",
            "kitty",
        ] {
            assert!(!exact_kde_terminal_window(&observed(
                "org.gnome.Terminal",
                class
            )));
        }
        let mut missing = observed("", "gnome-terminal");
        missing.identity.clear();
        assert!(!exact_kde_terminal_window(&missing));
        for info in [
            "{}",
            "{",
            r#"{"wmClass":"gnome-terminal"}"#,
            r#"{"appId":"","wmClass":null}"#,
        ] {
            assert!(!exact_kde_terminal_window(&crate::kde_focus::Observation {
                identity: "current-kwin-window".into(),
                info: info.into(),
            }));
        }
    }
}

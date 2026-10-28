#[path = "../gnome/desktop.rs"]
pub(super) mod gnome;
#[path = "../kde/desktop.rs"]
pub(super) mod kde;

/// A fresh observation from the selected desktop adapter. Process identity is
/// metadata for authenticating a native accessibility peer, not edit authority.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct FocusedWindow {
    pub(super) identity: String,
    pub(super) pid: u32,
}

impl FocusedWindow {
    pub(super) fn from_info(identity: String, info: &str) -> Option<Self> {
        let info: serde_json::Value = serde_json::from_str(info).ok()?;
        let value = info.get("pid")?;
        let pid = value
            .as_u64()
            .and_then(|pid| u32::try_from(pid).ok())
            .or_else(|| value.as_str()?.parse::<u32>().ok())?;
        (!identity.is_empty() && pid != 0).then_some(Self { identity, pid })
    }
}

pub(super) async fn focused_window_until(
    backend: lay::desktop::LayoutBackend,
    deadline: std::time::Instant,
) -> Option<FocusedWindow> {
    use lay::desktop::LayoutBackend;
    match backend {
        LayoutBackend::Kde => kde::focused_window_until(deadline),
        LayoutBackend::Gnome | LayoutBackend::Niri | LayoutBackend::X11 => {
            // Same explicit shell-protocol compatibility as terminal probing.
            gnome::focused_window_until(deadline).await
        }
    }
}

pub(super) async fn focused_window_is_native_terminal(
    backend: lay::desktop::LayoutBackend,
) -> bool {
    use lay::desktop::LayoutBackend;

    match backend {
        LayoutBackend::Kde => kde::focused_window_is_native_terminal(),
        LayoutBackend::Gnome => gnome::focused_window_is_native_terminal().await,
        // Preserve the existing shell-protocol compatibility for these backends.
        // This is an explicit dispatch, never a retry after a KDE probe fails.
        LayoutBackend::Niri | LayoutBackend::X11 => {
            gnome::focused_window_is_native_terminal().await
        }
    }
}

pub(super) fn native_terminal_focus_purpose_supported(
    backend: lay::desktop::LayoutBackend,
    purpose: u32,
) -> bool {
    use lay::desktop::LayoutBackend;
    match backend {
        LayoutBackend::Kde => kde::native_terminal_focus_purpose_supported(purpose),
        LayoutBackend::Gnome | LayoutBackend::Niri | LayoutBackend::X11 => purpose == 0,
    }
}

#[cfg(test)]
mod tests {
    use super::FocusedWindow;

    #[test]
    fn native_peer_binding_requires_explicit_nonzero_desktop_process() {
        for info in [
            "{}",
            r#"{"pid":0}"#,
            r#"{"pid":-1}"#,
            r#"{"pid":4294967296}"#,
            r#"{"pid":"app-title"}"#,
        ] {
            assert!(FocusedWindow::from_info("window".into(), info).is_none());
        }
        assert!(FocusedWindow::from_info(String::new(), r#"{"pid":12}"#).is_none());
        for info in [r#"{"pid":12}"#, r#"{"pid":"12"}"#] {
            assert_eq!(
                FocusedWindow::from_info("window".into(), info),
                Some(FocusedWindow {
                    identity: "window".into(),
                    pid: 12
                })
            );
        }
    }
}

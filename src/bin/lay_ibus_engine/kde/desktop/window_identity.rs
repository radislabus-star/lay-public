use super::super::FocusedWindow;

struct WindowReceipt<'a> {
    authority: &'a str,
    provider_lifetime: u64,
    window: &'a str,
}

fn receipt(identity: &str) -> Option<WindowReceipt<'_>> {
    // Canonical kde_focus::observation format. Its nonce, sender, deadline and
    // UUID are authenticated by that provider. This only projects field
    // identity; it never authorizes input or an edit. Unknown formats fail
    // closed when a changed receipt is compared.
    let body = identity.strip_prefix("kde-window:")?;
    let (prefix, window) = body.rsplit_once(':')?;
    let (authority, lifetime) = prefix.rsplit_once(':')?;
    let (bus, owner) = authority.split_once(':')?;
    if bus.is_empty() || window.is_empty() {
        return None;
    }
    zbus::names::UniqueName::try_from(owner).ok()?;
    Some(WindowReceipt {
        authority,
        provider_lifetime: lifetime.parse().ok()?,
        window,
    })
}

pub(in crate::window_interaction) fn same_native_window(
    first: &FocusedWindow,
    second: &FocusedWindow,
) -> bool {
    if first.pid != second.pid {
        return false;
    }
    if first.identity == second.identity {
        return true;
    }
    match (receipt(&first.identity), receipt(&second.identity)) {
        (Some(first), Some(second)) => {
            first.authority == second.authority && first.window == second.window
        }
        _ => false,
    }
}

pub(in crate::window_interaction) fn native_window_continues(
    previous: &FocusedWindow,
    current: &FocusedWindow,
) -> bool {
    if previous.pid == 0 || previous.identity.is_empty() || !same_native_window(previous, current) {
        return false;
    }
    if previous.identity == current.identity {
        return true;
    }
    match (receipt(&previous.identity), receipt(&current.identity)) {
        (Some(previous), Some(current)) => current.provider_lifetime >= previous.provider_lifetime,
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn window(lifetime: u64) -> FocusedWindow {
        FocusedWindow {
            identity: format!(
                "kde-window:bus1::1.5:{lifetime}:{{00000000-0000-0000-0000-000000000001}}"
            ),
            pid: 17,
        }
    }

    #[test]
    fn physical_window_equality_is_symmetric_but_receipt_continuity_is_forward() {
        let old = window(7);
        let fresh = window(9);
        assert_ne!(old.identity, fresh.identity);
        assert!(same_native_window(&old, &fresh));
        assert!(same_native_window(&fresh, &old));
        assert!(native_window_continues(&old, &fresh));
        assert!(!native_window_continues(&fresh, &old));
        assert!(native_window_continues(&old, &old));
    }

    #[test]
    fn continuity_keeps_window_authority_process_and_wire_format_exact() {
        let old = window(7);
        for identity in [
            window(9).identity.replace("bus1", "bus2"),
            window(9).identity.replace(":1.5", ":1.6"),
            window(9).identity.replace("000000000001", "000000000002"),
            window(9).identity.replace(":9:", ":invalid:"),
            "kde-window:bus1:org.kde.KWin:9:window".into(),
            "kde-window:bus1::1.5:9:".into(),
            "different-format".into(),
            String::new(),
        ] {
            let changed = FocusedWindow {
                identity,
                pid: old.pid,
            };
            assert!(!native_window_continues(&old, &changed));
        }
        let mut changed = window(9);
        changed.pid += 1;
        assert!(!same_native_window(&old, &changed));
        let invalid = FocusedWindow {
            identity: String::new(),
            pid: 0,
        };
        assert!(same_native_window(&invalid, &invalid)); // Equality grants no authority.
        assert!(!native_window_continues(&invalid, &invalid));
    }
}

use super::*;

#[test]
fn force_layout_hotkeys_use_single_key_ids_only() {
    assert_eq!(
        single_hotkey_keycode("single-rctrl"),
        Some(KeyCode::KEY_RIGHTCTRL)
    );
    assert_eq!(
        single_hotkey_keycode("single-ralt"),
        Some(KeyCode::KEY_RIGHTALT)
    );
    assert_eq!(
        single_hotkey_keycode("caps-lock"),
        Some(KeyCode::KEY_CAPSLOCK)
    );
    assert_eq!(single_hotkey_keycode("double-lshift"), None);
    assert_eq!(single_hotkey_keycode(""), None);
}

#[test]
fn host_focus_ignore_detects_vm_windows() {
    assert!(focused_window_json_is_ignored(
        r#"{"appId":"org.virt-manager.virt-manager","wmClass":"virt-manager","title":"KDE VM"}"#
    ));
    assert!(focused_window_json_is_ignored(
        r#"{"appId":"remote-viewer.desktop","wmClass":"remote-viewer","title":"SPICE display"}"#
    ));
    assert!(focused_window_json_is_ignored(
        r#"{"appId":"python3","wmClass":"python3","title":"lay-kde-test SPICE clipboard ON"}"#
    ));
    assert!(!focused_window_json_is_ignored(
        r#"{"appId":"org.gnome.Terminal.desktop","wmClass":"org.gnome.Terminal","title":"Terminal"}"#
    ));
}

#[test]
fn focused_window_identity_prefers_gnome_stable_sequence() {
    assert_eq!(
        focused_window_identity_from_json(
            r#"{"stableSequence":"4365","windowId":"3214725164","appId":"google-chrome.desktop","title":"about:blank"}"#
        ),
        Some("gnome-window:stableSequence:4365".to_string())
    );
    assert_eq!(
        focused_window_identity_from_json(
            r#"{"windowId":"3214725164","appId":"google-chrome.desktop","title":"about:blank"}"#
        ),
        Some("gnome-window:windowId:3214725164".to_string())
    );
}

#[test]
fn keyboard_discovery_ignores_service_virtual_devices() {
    assert!(should_ignore_keyboard_device_name("lay-virtual-keyboard"));
    assert!(should_ignore_keyboard_device_name(
        "ydotoold virtual device"
    ));
    assert!(!should_ignore_keyboard_device_name(
        "AT Translated Set 2 keyboard"
    ));
}

use super::*;

#[test]
fn parses_gdbus_string_tuple() {
    assert_eq!(parse_gdbus_string("('us',)"), Some("us".to_string()));
}

#[test]
fn parses_current_layout_from_list_layouts_reply() {
    assert_eq!(
        parse_current_layout_from_list("('0:xkb:us,1:xkb:ru*',)"),
        Some("ru".to_string())
    );
}

#[test]
fn parses_kde6_layout_list_reply() {
    let reply = r#"[Argument: a(sss) {[Argument: (sss) "us", "", "English (US)"], [Argument: (sss) "ru", "", "Russian"]}]"#;
    assert_eq!(parse_kde_layouts_list(reply), vec!["us", "ru"]);
}

#[test]
fn parses_first_quoted_string_with_escapes() {
    assert_eq!(
        first_quoted_string(r#" "us\"intl", "", "English" "#),
        Some(r#"us"intl"#.to_string())
    );
}

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
fn layout_backend_can_be_explicit_or_auto_detected() {
    assert_eq!(
        resolve_layout_backend("gnome", Some("KDE"), None, Some("wayland")),
        LayoutBackend::Gnome
    );
    assert_eq!(
        resolve_layout_backend("kde", Some("GNOME"), None, Some("wayland")),
        LayoutBackend::Kde
    );
    assert_eq!(
        resolve_layout_backend("x11", Some("GNOME"), None, Some("wayland")),
        LayoutBackend::X11
    );
    assert_eq!(
        resolve_layout_backend("auto", Some("KDE"), Some("plasma"), Some("wayland")),
        LayoutBackend::Kde
    );
    assert_eq!(
        resolve_layout_backend("auto", Some("GNOME"), None, Some("wayland")),
        LayoutBackend::Gnome
    );
    assert_eq!(
        resolve_layout_backend("auto", None, None, Some("x11")),
        LayoutBackend::X11
    );
}

#[test]
fn parses_x11_layout_tool_output() {
    assert_eq!(
        parse_setxkbmap_layout("rules: evdev\nmodel: pc105\nlayout: us,ru\n"),
        Some("us".to_string())
    );
    assert_eq!(normalize_layout_id(" ru\n"), "ru");
    assert_eq!(normalize_layout_id("xkb:ru::rus"), "ru");
    assert!(is_ru_layout_id("xkb:ru"));
    assert!(!is_ru_layout_id("xkb:us"));
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
fn keyboard_discovery_ignores_service_virtual_devices() {
    assert!(should_ignore_keyboard_device_name("lay-virtual-keyboard"));
    assert!(should_ignore_keyboard_device_name(
        "ydotoold virtual device"
    ));
    assert!(!should_ignore_keyboard_device_name(
        "AT Translated Set 2 keyboard"
    ));
}

#[test]
fn parses_gdbus_bool_tuple() {
    assert_eq!(parse_gdbus_bool("(true,)"), Some(true));
    assert_eq!(parse_gdbus_bool("(false,)"), Some(false));
    assert_eq!(parse_gdbus_bool("true"), None);
}

use super::*;

#[test]
fn backend_can_be_explicit_or_auto_detected() {
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
        resolve_layout_backend("niri", Some("GNOME"), None, Some("wayland")),
        LayoutBackend::Niri
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
        resolve_layout_backend("auto", Some("niri"), None, Some("wayland")),
        LayoutBackend::Niri
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
    assert!(is_ru_layout_id("lay-ime-ru"));
    assert!(!is_ru_layout_id("xkb:us"));
    assert!(!is_ru_layout_id("lay-ime-us"));
}

#[test]
fn parses_kde6_layout_list_reply() {
    let reply = r#"[Argument: a(sss) {[Argument: (sss) "us", "", "English (US)"], [Argument: (sss) "ru", "", "Russian"]}]"#;
    assert_eq!(parse_kde_layouts_list(reply), vec!["us", "ru"]);
}

#[test]
fn parses_kde_layout_list_with_escaped_id() {
    assert_eq!(
        first_quoted_string(r#" "us\"intl", "", "English" "#),
        Some(r#"us"intl"#.to_string())
    );
    let reply = r#"[Argument: a(sss) {[Argument: (sss) "xkb:ru::rus", "", "Russian"]}]"#;
    assert_eq!(parse_kde_layouts_list(reply), vec!["ru"]);
}

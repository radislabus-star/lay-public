//! Desktop/backend helpers shared by daemon frontends.
//!
//! GNOME, KDE and X11 use different integration layers, but they still need the
//! same backend selection and layout-id normalization rules.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LayoutBackend {
    Gnome,
    Kde,
    X11,
}

impl LayoutBackend {
    pub fn label(self) -> &'static str {
        match self {
            Self::Gnome => "gnome",
            Self::Kde => "kde",
            Self::X11 => "x11",
        }
    }
}

pub fn resolve_layout_backend(
    configured: &str,
    desktop: Option<&str>,
    session: Option<&str>,
    session_type: Option<&str>,
) -> LayoutBackend {
    match configured.trim().to_ascii_lowercase().as_str() {
        "gnome" => return LayoutBackend::Gnome,
        "kde" | "plasma" => return LayoutBackend::Kde,
        "x11" | "xorg" => return LayoutBackend::X11,
        _ => {}
    }

    let desktop = format!(
        "{}:{}",
        desktop.unwrap_or_default(),
        session.unwrap_or_default()
    )
    .to_ascii_lowercase();
    if desktop.contains("kde") || desktop.contains("plasma") {
        return LayoutBackend::Kde;
    }
    if desktop.contains("gnome") {
        return LayoutBackend::Gnome;
    }
    if session_type.is_some_and(|value| value.eq_ignore_ascii_case("x11")) {
        return LayoutBackend::X11;
    }
    LayoutBackend::Gnome
}

pub fn parse_setxkbmap_layout(output: &str) -> Option<String> {
    output.lines().find_map(|line| {
        let (key, value) = line.split_once(':')?;
        (key.trim() == "layout")
            .then(|| normalize_layout_id(value.split(',').next().unwrap_or(value)))
    })
}

pub fn normalize_layout_id(layout: &str) -> String {
    let trimmed = layout.trim();
    if let Some(rest) = trimmed.strip_prefix("xkb:") {
        return rest.split(':').next().unwrap_or("").to_ascii_lowercase();
    }
    trimmed
        .split([':', ' ', '\t', '\n'])
        .next()
        .unwrap_or("")
        .to_ascii_lowercase()
}

pub fn is_ru_layout_id(layout: &str) -> bool {
    normalize_layout_id(layout) == "ru"
}

#[cfg(test)]
mod tests {
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
}

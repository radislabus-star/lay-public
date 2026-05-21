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
    matches!(normalize_layout_id(layout).as_str(), "ru" | "lay-ime-ru")
}

#[cfg(test)]
#[path = "desktop_tests.rs"]
mod tests;

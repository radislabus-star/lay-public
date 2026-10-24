//! Select presentation from the same resolved desktop as layout/focus adapters.
//! This module formats proposals; it never grants text-edit authority.
use lay::desktop::LayoutBackend;
use zbus::zvariant::Value;

use crate::text::HintStyle;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum CompletionPresentation {
    Inline,
    Auxiliary,
}

// Preserve the actual pre-refactor serializer for unmodified desktop backends.
// Keep this independent of future KDE and GNOME presentation changes.
const OTHER_BACKEND_HINT_STYLE: HintStyle = HintStyle {
    foreground: 0x000000,
    background: Some(0xffffff),
    auxiliary_underline: Some(0),
    preedit_underline: 0,
};

pub(crate) fn completion_presentation(backend: LayoutBackend) -> CompletionPresentation {
    match backend {
        LayoutBackend::Gnome => crate::gnome::display::COMPLETION_PRESENTATION,
        LayoutBackend::Kde => crate::kde::display::COMPLETION_PRESENTATION,
        LayoutBackend::Niri | LayoutBackend::X11 => CompletionPresentation::Auxiliary,
    }
}

fn hint_style(backend: LayoutBackend) -> HintStyle {
    match backend {
        LayoutBackend::Kde => crate::kde::display::HINT_STYLE,
        LayoutBackend::Gnome => crate::gnome::display::HINT_STYLE,
        LayoutBackend::Niri | LayoutBackend::X11 => OTHER_BACKEND_HINT_STYLE,
    }
}

pub(crate) fn make_hint_ibus_text(backend: LayoutBackend, text: String) -> Value<'static> {
    crate::text::make_hint_ibus_text(text, hint_style(backend))
}

pub(crate) fn make_preedit_ibus_text(
    backend: LayoutBackend,
    text: String,
    suggestion_start: u32,
) -> Value<'static> {
    crate::text::make_preedit_ibus_text(text, suggestion_start, hint_style(backend))
}

#[cfg(test)]
#[path = "display/tests.rs"]
mod tests;

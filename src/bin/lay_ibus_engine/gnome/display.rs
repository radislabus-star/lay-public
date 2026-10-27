//! User-accepted GNOME inline presentation, independent of KDE styling.
use crate::display::CompletionPresentation;
use crate::text::HintStyle;

pub(crate) const COMPLETION_PRESENTATION: CompletionPresentation = CompletionPresentation::Inline;
pub(crate) const HINT_STYLE: HintStyle = HintStyle {
    foreground: 0x888888,
    background: None,
    auxiliary_underline: None,
    preedit_underline: 1,
};

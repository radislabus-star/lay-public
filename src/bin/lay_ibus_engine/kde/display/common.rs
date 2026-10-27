//! KDE presentation policy; shared text code owns scalar ranges/serialization.
//! Client adaptations stay here without gaining text-edit authority.
use crate::display::CompletionPresentation;
use crate::text::HintStyle;

pub(crate) const COMPLETION_PRESENTATION: CompletionPresentation =
    CompletionPresentation::Auxiliary;
pub(crate) const HINT_STYLE: HintStyle = HintStyle {
    foreground: 0x000000,
    background: Some(0xffffff),
    auxiliary_underline: Some(0),
    preedit_underline: 0,
};

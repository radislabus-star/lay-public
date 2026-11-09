//! A letter produces a request, never a client edit or a boundary candidate.

use crate::text_edit::EditAction;
use crate::typing_transition::decision::TransitionDecisionCore;

/// A model proposal admitted by the common decision owner, bound to exact text.
/// The existing input owner must independently verify its current word/frame.
#[derive(Debug, Clone)]
pub struct FragmentLayoutRequest {
    pub(crate) action: EditAction,
    pub(crate) target_layout_is_ru: bool,
}

impl FragmentLayoutRequest {
    pub fn action(&self) -> &EditAction {
        &self.action
    }

    pub fn target_layout_is_ru(&self) -> bool {
        self.target_layout_is_ru
    }
}

/// Only the just-inserted alphabetic scalar can trigger open-prefix scoring.
/// No global flag, Space rule, field observation, timer, learning or output.
pub fn on_letter(
    token: &str,
    inserted: char,
    current_layout_is_ru: bool,
) -> Option<FragmentLayoutRequest> {
    if !inserted.is_alphabetic()
        || !token.ends_with(inserted)
        || !(3..=32).contains(&token.chars().count())
        || !token.chars().all(char::is_alphabetic)
    {
        return None;
    }
    let analysis = super::analyze(token, super::Boundary::Open)?;
    TransitionDecisionCore::decide_fragment_layout(token, current_layout_is_ru, &analysis)
}

//! Minimal text replacement planning shared by desktop frontends.
//!
//! The planner decides which already-typed prefix/suffix can stay on screen and
//! returns the smallest edit needed for replacing the bad middle range.

#[path = "text_edit/action.rs"]
mod action;
#[path = "text_edit/committed_tail.rs"]
mod committed_tail;
#[path = "text_edit/cursor.rs"]
mod cursor;
#[path = "text_edit/diff_plan.rs"]
mod diff_plan;
#[path = "text_edit/safety.rs"]
mod safety;
#[path = "text_edit/types.rs"]
mod types;

pub use action::{EditAction, EditActionKind};
pub use committed_tail::{
    committed_separator_is_preserved, ensure_committed_tail_spacing,
    plan_committed_tail_full_token_replacement, plan_committed_tail_replacement,
    plan_committed_whitespace_insertions,
};
pub use cursor::offset_replacement_plan_for_cursor;
pub use diff_plan::{
    apply_replacement_plan_to_text, plan_text_replacement, replacement_plan_matches, tail_chars,
};
pub use safety::{autocorrect_edit_safety, EditPlanSafetyReport};
pub use types::TextReplacement;

#[cfg(test)]
#[path = "text_edit_tests.rs"]
mod tests;

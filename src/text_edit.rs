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
#[path = "text_edit/executor.rs"]
mod executor;
#[path = "text_edit/gate.rs"]
mod gate;
#[path = "text_edit/mutation.rs"]
mod mutation;
#[path = "text_edit/safety.rs"]
pub(crate) mod safety;
#[path = "text_edit/transition.rs"]
mod transition;
#[path = "text_edit/types.rs"]
mod types;
#[path = "text_edit/visible_tail.rs"]
mod visible_tail;

pub(crate) use action::PlannedReplacementInput;
pub use action::{EditAction, EditActionKind};
pub(crate) use committed_tail::plan_committed_tail_last_token_replacement;
pub use committed_tail::{
    committed_separator_is_preserved, ensure_committed_tail_spacing,
    plan_committed_tail_full_token_replacement, plan_committed_tail_replacement,
    plan_committed_whitespace_insertions,
};
pub use cursor::offset_replacement_plan_for_cursor;
pub use diff_plan::{
    apply_replacement_plan_to_text, plan_text_replacement, replacement_plan_matches, tail_chars,
};
pub use executor::{
    authorize_backend_edit, AuthorizedEdit, BackendEditAuthorization, TextEditBackend,
};
pub(crate) use gate::plan_verified_transition_edit;
pub use gate::{
    plan_ime_completion_edit, plan_input_gate_edit, plan_manual_edit, plan_native_edit,
    plan_recorded_undo_edit,
};
pub use mutation::{TransitionAudit, TransitionOperator, TransitionProof};
pub use safety::{autocorrect_edit_safety, EditPlanSafetyReport};
pub use transition::{
    decide_text_transition, LatentTextTransitionCandidate, TextTransitionDecision,
    TextTransitionIntent, TextTransitionRejection, VisibleFieldState,
};
pub use types::TextReplacement;
pub use visible_tail::{VisibleTail, VisibleTailSnapshot, VisibleTailSource};

#[cfg(test)]
#[path = "text_edit_tests.rs"]
mod tests;

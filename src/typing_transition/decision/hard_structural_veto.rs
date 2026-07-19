//! Non-negotiable transition-shape vetoes.
//!
//! This owner checks the selected typed transition only. It does not rank the
//! lattice, calibrate confidence, or choose a replacement.

use super::{CandidateDecisionEvaluation, CandidateDecisionSignals};
use crate::correction_core::{TypingErrorEvent, UnifiedCorrectionCandidate};
use crate::typing_transition::verifier::EditTransitionOperator;

pub(super) fn hidden_state_rejection(signals: &CandidateDecisionSignals) -> Option<&'static str> {
    if signals.l4_hidden_plan_commitment != 0 && !signals.l4_hidden_certificate_valid {
        return Some("l4_invalid_resolution_certificate");
    }
    let disposition = signals.l4_hidden_disposition;
    let hidden_rejected = disposition
        == crate::nanda_wave::l4_hidden_state::L4HiddenDisposition::Rejected
        || (disposition == crate::nanda_wave::l4_hidden_state::L4HiddenDisposition::Ambiguous
            && signals.l4_hidden_ambiguity_authoritative)
        || (signals.l4_hidden_selected_witnessed
            && disposition != crate::nanda_wave::l4_hidden_state::L4HiddenDisposition::Witnessed);
    hidden_rejected.then_some(disposition.as_str())
}

pub(super) fn boundary_shift_rejection(
    event: &TypingErrorEvent,
    candidate: &UnifiedCorrectionCandidate,
    evaluation: &CandidateDecisionEvaluation,
    learned_short_boundary_authority: bool,
) -> Option<&'static str> {
    let action = evaluation.action;
    if action.edit_operator != EditTransitionOperator::BoundaryShift {
        return None;
    }
    let high_precision = high_precision_boundary_shift(event, candidate, evaluation);
    (!high_precision && !learned_short_boundary_authority)
        .then_some("ambiguous_short_boundary_shift")
}

pub(super) fn high_precision_boundary_shift(
    event: &TypingErrorEvent,
    candidate: &UnifiedCorrectionCandidate,
    evaluation: &CandidateDecisionEvaluation,
) -> bool {
    evaluation.action.edit_operator == EditTransitionOperator::BoundaryShift
        && evaluation.action.verifier_passed
        && boundary_shift_has_stable_token_mass(&candidate.replacement)
        && boundary_shift_field_readout(&event.original, &candidate.replacement)
            .is_some_and(|readout| readout.has_direct_apply_mass())
}

pub(super) fn verifier_rejection(evaluation: &CandidateDecisionEvaluation) -> Option<&'static str> {
    (!evaluation.action.verifier_passed).then_some("unverified_transition")
}

fn boundary_shift_has_stable_token_mass(replacement: &str) -> bool {
    let words = crate::correction_core::normalized_correction_words(replacement);
    let Some(pair) = words.get(words.len().saturating_sub(2)..) else {
        return false;
    };
    pair.len() == 2 && pair.iter().all(|word| word.chars().count() >= 4)
}

fn boundary_shift_field_readout(
    original: &str,
    replacement: &str,
) -> Option<crate::hot_field::HotBoundaryShiftReadout> {
    let original_words = crate::correction_core::normalized_correction_words(original);
    let replacement_words = crate::correction_core::normalized_correction_words(replacement);
    if original_words.len() != replacement_words.len() || original_words.len() < 2 {
        return None;
    }
    let original_pair = &original_words[original_words.len() - 2..];
    let replacement_pair = &replacement_words[replacement_words.len() - 2..];
    Some(
        crate::hot_field::HotFieldSnapshot::current().boundary_shift_readout(
            &original_pair[0],
            &original_pair[1],
            &replacement_pair[0],
            &replacement_pair[1],
        ),
    )
}

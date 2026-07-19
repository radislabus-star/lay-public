//! Final policy primitives after field and structural evidence is available.

use super::proposal_admission::CandidateGateAction;
use crate::typing_transition::L4SignedTransitionSignal;

pub(super) fn producer_allows_authority_evaluation(
    action: CandidateGateAction,
    l4_signal: L4SignedTransitionSignal,
) -> bool {
    action == CandidateGateAction::Eligible
        || (action == CandidateGateAction::SuggestOnly && l4_signal.exact_positive())
}

pub(super) fn unresolved_competitor_blocks(
    exact_positive_transition: bool,
    stronger_unresolved_exists: bool,
) -> bool {
    !exact_positive_transition && stronger_unresolved_exists
}

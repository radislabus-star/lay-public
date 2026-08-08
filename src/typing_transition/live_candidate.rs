//! Canonical live-candidate arbitration owned by the Typing Transition CPU.
//!
//! Producers may calculate evidence, but only `TransitionDecisionCore` may
//! admit, merge, and order candidates for a live IME readout.

use super::decision::TransitionDecisionCore;
use crate::typing_cpu::{ImeCandidateProposal, ImeCandidateSource};
use std::collections::HashSet;

#[derive(Debug, Clone)]
pub(crate) struct LiveCompletionProposal {
    pub(crate) state_before: u64,
    pub(crate) surface: String,
    pub(crate) suffix: String,
    pub(crate) score: f32,
    pub(crate) rank_score: f32,
    /// Unclamped phase-field strength. Ranking keeps this as a tie-break when
    /// later probabilistic signals saturate to the same score.
    pub(crate) field_strength: u32,
    pub(crate) source: &'static str,
    pub(crate) partial_len: usize,
    pub(crate) suffix_len: usize,
    /// The typed prefix is already an exact lexical state. Extending it needs
    /// independent context evidence instead of lexical geometry alone.
    pub(crate) partial_state_known: bool,
    pub(crate) allow_short_lexical: bool,
    pub(crate) structural: f32,
    pub(crate) usage: f32,
    pub(crate) context_usage: f32,
    pub(crate) accepted: u32,
    pub(crate) common: bool,
    pub(crate) hot: bool,
    pub(crate) l2_center_grounded: bool,
    pub(crate) l3_memory_supported: bool,
    pub(crate) context_birth: bool,
    pub(crate) completed_state_known: bool,
    pub(crate) corrected_prefix_completion: bool,
    pub(crate) l3_relation_class: u64,
    pub(crate) l4_transition_state_specific: bool,
    pub(crate) l4_transition_attract_count: u32,
    pub(crate) l4_transition_repel_count: u32,
}

#[derive(Debug, Clone)]
pub(crate) struct SelectedLiveCompletion {
    pub(crate) surface: String,
    pub(crate) suffix: String,
    pub(crate) score: f32,
    pub(crate) rank_score: f32,
    pub(crate) source: &'static str,
}

impl TransitionDecisionCore {
    pub(crate) fn select_live_completions(
        proposals: Vec<LiveCompletionProposal>,
        limit: usize,
    ) -> Vec<SelectedLiveCompletion> {
        let proposals = proposals
            .into_iter()
            .filter(|proposal| Self::admit_live_completion(proposal).visible())
            .collect::<Vec<_>>();
        let inputs = proposals
            .iter()
            .map(
                |proposal| crate::nanda_wave::l4_hidden_state::L4HiddenCandidateInput {
                    predicted_state: crate::nanda_wave::l4_hidden_state::predicted_state_id(
                        proposal.state_before,
                        "accept_completion",
                        &proposal.surface,
                    ),
                    relation_class: proposal.l3_relation_class,
                    operator_class: crate::nanda_wave::phase_field::hash_text("accept_completion"),
                    verifier_passed: true,
                    rank_milli: crate::text_metrics::score_to_milli(proposal.rank_score),
                    context_support: proposal.l3_memory_supported || proposal.completed_state_known,
                    pairwise_context_witness: false,
                    eligible: true,
                    witness_attract: proposal.l4_transition_attract_count,
                    witness_repel: proposal.l4_transition_repel_count,
                    witness_state_specific: proposal.l4_transition_state_specific,
                    phase_witness_milli: 0,
                    phase_witness_supported: false,
                    operator_consensus_witness: false,
                },
            )
            .collect::<Vec<_>>();
        let hidden = crate::nanda_wave::l4_hidden_state::estimate_hidden_typing_state(&inputs);
        let mut selected = proposals
            .into_iter()
            .zip(hidden)
            .filter(|(_, state)| {
                state.disposition
                    != crate::nanda_wave::l4_hidden_state::L4HiddenDisposition::Rejected
            })
            .map(|(proposal, _)| proposal)
            .collect::<Vec<_>>();

        selected.sort_by(|left, right| {
            right
                .rank_score
                .total_cmp(&left.rank_score)
                .then_with(|| right.field_strength.cmp(&left.field_strength))
                .then_with(|| left.suffix_len.cmp(&right.suffix_len))
                .then_with(|| left.surface.cmp(&right.surface))
        });
        let corrected_prefix_reserve = selected
            .iter()
            .find(|candidate| candidate.corrected_prefix_completion)
            .cloned();
        let mut seen_surfaces = HashSet::new();
        let mut seen_suffixes = HashSet::new();
        selected.retain(|candidate| {
            seen_surfaces.insert(candidate.surface.clone())
                && (candidate.suffix.is_empty() || seen_suffixes.insert(candidate.suffix.clone()))
        });
        selected.truncate(limit);
        if let Some(candidate) = corrected_prefix_reserve {
            if !selected
                .iter()
                .any(|current| current.surface == candidate.surface)
            {
                if selected.len() == limit {
                    selected.pop();
                }
                selected.push(candidate);
            }
        }
        selected
            .into_iter()
            .map(|candidate| SelectedLiveCompletion {
                surface: candidate.surface,
                suffix: candidate.suffix,
                score: candidate.score,
                rank_score: candidate.rank_score,
                source: candidate.source,
            })
            .collect()
    }

    pub(crate) fn select_ime_readout(
        proposals: &[ImeCandidateProposal],
        limit: usize,
    ) -> Vec<ImeCandidateProposal> {
        if limit == 0 {
            return Vec::new();
        }
        let first_l2_order = proposals.iter().position(|proposal| {
            matches!(
                proposal.source,
                ImeCandidateSource::L2Completion | ImeCandidateSource::L2Replacement
            )
        });
        let mut selected = Vec::<(ImeCandidateProposal, usize)>::with_capacity(proposals.len());
        for (order, proposal) in proposals.iter().enumerate() {
            let source_already_admitted = matches!(
                proposal.source,
                ImeCandidateSource::L2Completion | ImeCandidateSource::L2Replacement
            ) && first_l2_order == Some(order);
            if !proposal.is_replacement()
                && !source_already_admitted
                && !crate::typing_cpu::is_allowed_visible_completion_suffix(&proposal.suffix)
            {
                continue;
            }
            if let Some(existing) = selected
                .iter_mut()
                .find(|(candidate, _)| candidate.display_text() == proposal.display_text())
            {
                let proposal_owns_order =
                    match (proposal.authority_order, existing.0.authority_order) {
                        (Some(proposal_order), Some(existing_order)) => {
                            proposal_order < existing_order
                        }
                        (Some(_), None) => true,
                        (None, Some(_)) => false,
                        (None, None) => proposal.confidence > existing.0.confidence,
                    };
                if proposal_owns_order {
                    existing.0 = proposal.clone();
                    existing.1 = order;
                }
                continue;
            }
            selected.push((proposal.clone(), order));
        }
        selected.sort_by(|left, right| {
            match (left.0.authority_order, right.0.authority_order) {
                (Some(left_order), Some(right_order)) => left_order.cmp(&right_order),
                (Some(_), None) => std::cmp::Ordering::Less,
                (None, Some(_)) => std::cmp::Ordering::Greater,
                (None, None) => right.0.confidence.total_cmp(&left.0.confidence),
            }
            .then_with(|| left.1.cmp(&right.1))
            .then_with(|| left.0.display_text().cmp(right.0.display_text()))
        });
        selected.truncate(limit);
        selected.into_iter().map(|(proposal, _)| proposal).collect()
    }
}

#[cfg(test)]
pub(crate) fn live_completion_has_authority(candidate: &LiveCompletionProposal) -> bool {
    TransitionDecisionCore::admit_live_completion(candidate).candidate_visible
}

#[cfg(test)]
pub(crate) fn live_suffix_has_display_authority(candidate: &LiveCompletionProposal) -> bool {
    TransitionDecisionCore::admit_live_completion(candidate).suffix_visible
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::typing_cpu::{ImeCandidateProposal, ImeCandidateSource};

    fn completion(surface: &str, suffix: &str, rank_score: f32) -> LiveCompletionProposal {
        LiveCompletionProposal {
            state_before: crate::nanda_wave::phase_field::hash_text("test-state"),
            surface: surface.to_string(),
            suffix: suffix.to_string(),
            score: rank_score.clamp(0.0, 1.0),
            rank_score,
            field_strength: 0,
            source: "test",
            partial_len: 4,
            suffix_len: suffix.chars().count(),
            partial_state_known: false,
            allow_short_lexical: true,
            structural: 0.5,
            usage: 0.0,
            context_usage: 0.0,
            accepted: 0,
            common: true,
            hot: false,
            l2_center_grounded: true,
            l3_memory_supported: false,
            context_birth: false,
            completed_state_known: true,
            corrected_prefix_completion: false,
            l3_relation_class: 0,
            l4_transition_state_specific: false,
            l4_transition_attract_count: 0,
            l4_transition_repel_count: 0,
        }
    }

    #[test]
    fn decision_core_is_the_live_completion_sort_owner() {
        let selected = TransitionDecisionCore::select_live_completions(
            vec![
                completion("провод", "од", 0.4),
                completion("проверка", "ерка", 0.8),
            ],
            8,
        );
        assert_eq!(selected[0].surface, "проверка");
    }

    #[test]
    fn decision_core_does_not_emit_a_candidate_rejected_by_live_admission() {
        let mut weak_extension = completion("осьмых", "мых", 0.9);
        weak_extension.partial_len = 3;
        weak_extension.partial_state_known = true;
        assert!(
            TransitionDecisionCore::select_live_completions(vec![weak_extension.clone()], 8)
                .is_empty()
        );

        weak_extension.context_birth = true;
        let selected = TransitionDecisionCore::select_live_completions(vec![weak_extension], 8);
        assert_eq!(selected.len(), 1);
    }

    #[test]
    fn ime_projection_preserves_single_core_order_and_deduplicates() {
        let proposals = vec![
            ImeCandidateProposal::new("ождь", 0.7, ImeCandidateSource::L3Context),
            ImeCandidateProposal::new("ождь", 0.9, ImeCandidateSource::L2Completion),
            ImeCandidateProposal::new("ень", 0.8, ImeCandidateSource::L2Completion),
        ];
        let selected = TransitionDecisionCore::select_ime_readout(&proposals, 8);
        assert_eq!(
            selected
                .iter()
                .map(|proposal| proposal.display_text())
                .collect::<Vec<_>>(),
            vec!["ождь", "ень"]
        );
    }

    #[test]
    fn authorized_l2_single_letter_suffix_reaches_ime_readout() {
        let proposals = vec![ImeCandidateProposal::new(
            "ь",
            0.9,
            ImeCandidateSource::L2Completion,
        )];

        let selected = TransitionDecisionCore::select_ime_readout(&proposals, 8);
        assert_eq!(selected[0].display_text(), "ь");
    }

    #[test]
    fn typed_replacement_survives_ime_readout_without_becoming_a_suffix() {
        let proposals = vec![ImeCandidateProposal::replacement(
            "работает",
            0.9,
            ImeCandidateSource::L2Replacement,
        )];

        let selected = TransitionDecisionCore::select_ime_readout(&proposals, 8);
        assert_eq!(selected[0].display_text(), "работает");
        assert!(selected[0].is_replacement());
    }
}

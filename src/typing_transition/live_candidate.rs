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
    /// True only while the user is still editing this token in preedit. A
    /// committed clean token is a settled state, not an open suffix lane.
    pub(crate) active_composition: bool,
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
        if limit == 0 {
            return Vec::new();
        }
        let proposals = proposals
            .into_iter()
            .filter(|proposal| Self::admit_live_completion(proposal).visible())
            .collect::<Vec<_>>();
        // This route only exposes display candidates. L4 attraction/repulsion
        // has already contributed to `rank_score`; mutation-oriented hidden
        // state must not erase a grounded L1.1/L2 candidate. The separate
        // Space/Tab apply route retains verifier and authority ownership.
        let mut selected = proposals;

        selected.sort_by(|left, right| {
            right
                .rank_score
                .total_cmp(&left.rank_score)
                .then_with(|| right.field_strength.cmp(&left.field_strength))
                .then_with(|| left.suffix_len.cmp(&right.suffix_len))
                .then_with(|| left.surface.cmp(&right.surface))
        });
        let mut seen_surfaces = HashSet::new();
        let mut seen_suffixes = HashSet::new();
        selected.retain(|candidate| {
            seen_surfaces.insert(candidate.surface.clone())
                && (candidate.suffix.is_empty() || seen_suffixes.insert(candidate.suffix.clone()))
        });

        // Keep the bounded display field diverse: corrected-prefix basins must
        // not evict every exact L1.1/L2 continuation, while an exact-only lane
        // must not hide the best typo-tolerant candidate. This is candidate
        // topology, not a word- or suffix-specific rule.
        let exact_reserve_limit = limit.saturating_div(3).max(1);
        let exact_reserve = selected
            .iter()
            .filter(|candidate| {
                !candidate.corrected_prefix_completion && !candidate.suffix.is_empty()
            })
            .take(exact_reserve_limit)
            .cloned()
            .collect::<Vec<_>>();
        let corrected_prefix_reserve = selected
            .iter()
            .find(|candidate| candidate.corrected_prefix_completion)
            .cloned();
        let mut bounded = Vec::with_capacity(limit);
        for candidate in exact_reserve
            .into_iter()
            .chain(corrected_prefix_reserve)
            .chain(selected)
        {
            if bounded
                .iter()
                .any(|current: &LiveCompletionProposal| current.surface == candidate.surface)
            {
                continue;
            }
            bounded.push(candidate);
            if bounded.len() == limit {
                break;
            }
        }
        bounded.sort_by(|left, right| {
            right
                .rank_score
                .total_cmp(&left.rank_score)
                .then_with(|| right.field_strength.cmp(&left.field_strength))
                .then_with(|| left.suffix_len.cmp(&right.suffix_len))
                .then_with(|| left.surface.cmp(&right.surface))
        });
        bounded
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
            active_composition: true,
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
    fn decision_core_requires_grounding_for_known_prefix_extension() {
        let mut weak_extension = completion("known-extension", "extension", 0.9);
        weak_extension.partial_len = 3;
        weak_extension.partial_state_known = true;
        weak_extension.l2_center_grounded = false;
        weak_extension.completed_state_known = false;
        assert!(
            TransitionDecisionCore::select_live_completions(vec![weak_extension.clone()], 8)
                .is_empty()
        );

        weak_extension.l2_center_grounded = true;
        weak_extension.completed_state_known = true;
        let grounded =
            TransitionDecisionCore::select_live_completions(vec![weak_extension.clone()], 8);
        assert_eq!(grounded.len(), 1);

        weak_extension.l2_center_grounded = false;
        weak_extension.completed_state_known = false;
        weak_extension.context_birth = true;
        weak_extension.l3_memory_supported = true;
        let selected = TransitionDecisionCore::select_live_completions(vec![weak_extension], 8);
        assert_eq!(selected.len(), 1);
    }

    #[test]
    fn bounded_readout_preserves_exact_and_corrected_prefix_lanes() {
        let mut proposals = Vec::new();
        for index in 0..8 {
            let mut candidate = completion(
                &format!("corrected-{index}"),
                "",
                0.90 - index as f32 * 0.01,
            );
            candidate.corrected_prefix_completion = true;
            proposals.push(candidate);
        }
        proposals.push(completion("exact-continuation", "continuation", 0.40));

        let selected = TransitionDecisionCore::select_live_completions(proposals, 4);

        assert!(selected
            .iter()
            .any(|candidate| candidate.surface == "exact-continuation"));
        assert!(selected.iter().any(|candidate| candidate.suffix.is_empty()));
    }

    #[test]
    fn l4_negative_witness_cannot_erase_a_grounded_display_candidate() {
        let mut candidate = completion("проверка", "ерка", 0.42);
        candidate.l2_center_grounded = true;
        candidate.completed_state_known = true;
        candidate.l4_transition_state_specific = true;
        candidate.l4_transition_attract_count = 0;
        candidate.l4_transition_repel_count = 4;

        let selected = TransitionDecisionCore::select_live_completions(vec![candidate], 8);

        assert_eq!(selected.len(), 1);
        assert_eq!(selected[0].surface, "проверка");
        assert_eq!(selected[0].suffix, "ерка");
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

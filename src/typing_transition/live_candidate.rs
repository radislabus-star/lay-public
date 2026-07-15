//! Canonical live-candidate arbitration owned by the Typing Transition CPU.
//!
//! Producers may calculate evidence, but only `TransitionDecisionCore` may
//! admit, merge, and order candidates for a live IME readout.

use super::decision::TransitionDecisionCore;
use crate::ime_candidate_readout::ImeCandidateProposal;

#[derive(Debug, Clone)]
pub(crate) struct LiveCompletionProposal {
    pub(crate) surface: String,
    pub(crate) suffix: String,
    pub(crate) score: f32,
    pub(crate) rank_score: f32,
    pub(crate) source: &'static str,
    pub(crate) partial_len: usize,
    pub(crate) suffix_len: usize,
    pub(crate) allow_short_lexical: bool,
    pub(crate) structural: f32,
    pub(crate) usage: f32,
    pub(crate) context_usage: f32,
    pub(crate) accepted: u32,
    pub(crate) common: bool,
    pub(crate) hot: bool,
    pub(crate) l2_center_grounded: bool,
    pub(crate) l3_memory_supported: bool,
    pub(crate) completed_state_known: bool,
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
        let mut selected = proposals
            .into_iter()
            .filter(live_completion_has_authority)
            .filter(live_suffix_has_display_authority)
            .collect::<Vec<_>>();

        selected.sort_by(|left, right| {
            right
                .rank_score
                .total_cmp(&left.rank_score)
                .then_with(|| left.suffix_len.cmp(&right.suffix_len))
                .then_with(|| left.surface.cmp(&right.surface))
        });
        selected
            .dedup_by(|left, right| left.surface == right.surface || left.suffix == right.suffix);
        selected.truncate(limit);
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
    ) -> Vec<String> {
        if limit == 0 {
            return Vec::new();
        }
        let mut selected = Vec::<(String, f32, usize)>::with_capacity(proposals.len());
        for (order, proposal) in proposals.iter().enumerate() {
            if !crate::ime_candidate_readout::is_allowed_visible_completion_suffix(&proposal.suffix)
            {
                continue;
            }
            if let Some(existing) = selected
                .iter_mut()
                .find(|(suffix, _, _)| suffix == &proposal.suffix)
            {
                if proposal.confidence > existing.1 {
                    existing.1 = proposal.confidence;
                    existing.2 = order;
                }
                continue;
            }
            selected.push((proposal.suffix.clone(), proposal.confidence, order));
        }
        selected.sort_by(|left, right| {
            right
                .1
                .total_cmp(&left.1)
                .then_with(|| left.2.cmp(&right.2))
                .then_with(|| left.0.cmp(&right.0))
        });
        selected.truncate(limit);
        selected.into_iter().map(|(suffix, _, _)| suffix).collect()
    }
}

pub(crate) fn live_completion_has_authority(candidate: &LiveCompletionProposal) -> bool {
    let usage_signal =
        candidate.usage >= 0.025 || candidate.context_usage >= 0.018 || candidate.accepted >= 1;
    let lexical_signal = candidate.common || candidate.hot || candidate.l2_center_grounded;
    let structural_signal = candidate.structural >= 0.34;
    let bound_structural_signal = candidate.l2_center_grounded && structural_signal;

    match candidate.partial_len {
        0 | 1 => false,
        2 => {
            candidate.allow_short_lexical
                && (usage_signal
                    || bound_structural_signal
                    || candidate.context_usage >= 0.018
                    || candidate.hot
                    || candidate.common)
                && candidate.suffix_len <= 8
        }
        3 => {
            if !candidate.allow_short_lexical {
                usage_signal
            } else {
                usage_signal
                    || bound_structural_signal
                    || (lexical_signal && candidate.suffix_len <= 7)
            }
        }
        4 => usage_signal || bound_structural_signal || lexical_signal,
        _ => usage_signal || lexical_signal || (structural_signal && candidate.l3_memory_supported),
    }
}

pub(crate) fn live_suffix_has_display_authority(candidate: &LiveCompletionProposal) -> bool {
    if candidate.suffix_len != 1 || matches!(candidate.suffix.as_str(), "и" | "я") {
        return true;
    }
    candidate.completed_state_known
        || candidate.accepted >= 2
        || candidate.context_usage >= 0.060
        || candidate.usage >= 0.095
        || (candidate.score >= 0.90 && candidate.structural >= 0.46)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ime_candidate_readout::{ImeCandidateProposal, ImeCandidateSource};

    fn completion(surface: &str, suffix: &str, rank_score: f32) -> LiveCompletionProposal {
        LiveCompletionProposal {
            surface: surface.to_string(),
            suffix: suffix.to_string(),
            score: rank_score.clamp(0.0, 1.0),
            rank_score,
            source: "test",
            partial_len: 4,
            suffix_len: suffix.chars().count(),
            allow_short_lexical: true,
            structural: 0.5,
            usage: 0.0,
            context_usage: 0.0,
            accepted: 0,
            common: true,
            hot: false,
            l2_center_grounded: true,
            l3_memory_supported: false,
            completed_state_known: true,
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
    fn ime_projection_preserves_single_core_order_and_deduplicates() {
        let proposals = vec![
            ImeCandidateProposal::new("ождь", 0.7, ImeCandidateSource::L3Context),
            ImeCandidateProposal::new("ождь", 0.9, ImeCandidateSource::L2Completion),
            ImeCandidateProposal::new("ень", 0.8, ImeCandidateSource::L2Completion),
        ];
        assert_eq!(
            TransitionDecisionCore::select_ime_readout(&proposals, 8),
            vec!["ождь", "ень"]
        );
    }
}

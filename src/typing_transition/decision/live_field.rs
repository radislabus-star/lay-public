//! Live IME admission through the same L2/L3/L4 field owner.
//!
//! IME producers build evidence.  They do not decide that a suffix is visible:
//! the decision belongs to `TransitionDecisionCore`, just like Space
//! autocorrect admission.

use super::TransitionDecisionCore;
use crate::typing_transition::live_candidate::LiveCompletionProposal;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct LiveFieldAdmission {
    pub(crate) candidate_visible: bool,
    pub(crate) suffix_visible: bool,
    pub(crate) reason: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct LiveFieldScoreInput {
    pub(crate) structural: f32,
    pub(crate) boundary_center_grounded: bool,
    pub(crate) wave_peak_rank_bonus: f32,
    pub(crate) usage: f32,
    pub(crate) context_usage: f32,
    pub(crate) accepted: u32,
    pub(crate) common: bool,
    pub(crate) hot: bool,
    pub(crate) partial_len: usize,
    pub(crate) l4_signed_weight: f32,
    pub(crate) l3_rank_delta: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct LiveFieldScore {
    pub(crate) score: f32,
    pub(crate) rank_score: f32,
}

impl LiveFieldAdmission {
    pub(crate) const fn visible(self) -> bool {
        self.candidate_visible && self.suffix_visible
    }
}

impl TransitionDecisionCore {
    pub(crate) fn score_live_completion_field(input: LiveFieldScoreInput) -> LiveFieldScore {
        let base_score = 0.22
            + input.structural
            + if input.boundary_center_grounded {
                0.28
            } else {
                0.0
            }
            + input.wave_peak_rank_bonus
            + input.usage * 2.30
            + input.context_usage * 3.20
            + (input.accepted.min(20) as f32 * 0.030)
            + if input.common { 0.055 } else { 0.0 }
            + if input.hot { 0.045 } else { 0.0 }
            + (input.partial_len.min(8) as f32 * 0.018)
            + live_l4_signed_bias(input.l4_signed_weight);
        let rank_score = base_score + input.l3_rank_delta;
        LiveFieldScore {
            score: rank_score.clamp(0.0, 1.0),
            rank_score,
        }
    }

    pub(crate) fn admit_live_completion(candidate: &LiveCompletionProposal) -> LiveFieldAdmission {
        let candidate_visible = live_candidate_field_has_authority(candidate);
        let suffix_visible = live_suffix_field_has_display_authority(candidate);
        LiveFieldAdmission {
            candidate_visible,
            suffix_visible,
            reason: live_admission_reason(candidate_visible, suffix_visible),
        }
    }
}

fn live_l4_signed_bias(signed_weight: f32) -> f32 {
    (signed_weight * 0.085).clamp(-0.080, 0.080)
}

fn live_admission_reason(candidate_visible: bool, suffix_visible: bool) -> &'static str {
    match (candidate_visible, suffix_visible) {
        (true, true) => "field_visible",
        (false, _) => "field_candidate_not_grounded",
        (true, false) => "field_suffix_not_grounded",
    }
}

fn live_candidate_field_has_authority(candidate: &LiveCompletionProposal) -> bool {
    let grounded_active_extension = candidate.active_composition
        && candidate.allow_short_lexical
        && !candidate.suffix.is_empty()
        && candidate.l2_center_grounded
        && candidate.completed_state_known;
    if candidate.partial_state_known
        && !candidate.context_birth
        && candidate.context_usage < 0.018
        && !grounded_active_extension
    {
        return false;
    }
    let usage_signal =
        candidate.usage >= 0.025 || candidate.context_usage >= 0.018 || candidate.accepted >= 1;
    let lexical_signal = candidate.common || candidate.hot || candidate.l2_center_grounded;
    let structural_signal = candidate.structural >= 0.34;
    let bound_structural_signal = candidate.l2_center_grounded && structural_signal;
    let short_prefix_suffix_supported =
        candidate.suffix_len <= 12 || (candidate.context_birth && candidate.l3_memory_supported);

    match candidate.partial_len {
        0 => false,
        1 => {
            candidate.allow_short_lexical
                && (candidate.l3_memory_supported
                    || candidate.context_usage >= 0.040
                    || candidate.usage >= 0.080
                    || candidate.accepted >= 2
                    || (candidate.l2_center_grounded && (candidate.common || candidate.hot)))
                && short_prefix_suffix_supported
        }
        2 => {
            candidate.allow_short_lexical
                && (usage_signal
                    || bound_structural_signal
                    || candidate.context_usage >= 0.018
                    || candidate.hot
                    || candidate.common)
                && short_prefix_suffix_supported
        }
        3 => {
            if !candidate.allow_short_lexical {
                usage_signal
            } else {
                usage_signal
                    || bound_structural_signal
                    || (lexical_signal && candidate.suffix_len <= 7)
                    || (candidate.context_birth && candidate.l3_memory_supported)
            }
        }
        4 => usage_signal || bound_structural_signal || lexical_signal,
        _ => usage_signal || lexical_signal || (structural_signal && candidate.l3_memory_supported),
    }
}

fn live_suffix_field_has_display_authority(candidate: &LiveCompletionProposal) -> bool {
    if candidate.suffix_len != 1 || matches!(candidate.suffix.as_str(), "и" | "я") {
        return true;
    }
    candidate.completed_state_known
        || candidate.accepted >= 2
        || candidate.context_usage >= 0.060
        || candidate.usage >= 0.095
}

#[cfg(test)]
mod tests {
    use super::*;

    fn proposal(partial_len: usize, suffix: &str) -> LiveCompletionProposal {
        LiveCompletionProposal {
            state_before: crate::nanda_wave::phase_field::hash_text("live-field-test"),
            surface: "проверка".to_string(),
            suffix: suffix.to_string(),
            score: 0.72,
            rank_score: 0.72,
            field_strength: 100,
            source: "test",
            partial_len,
            suffix_len: suffix.chars().count(),
            partial_state_known: false,
            active_composition: true,
            allow_short_lexical: true,
            structural: 0.50,
            usage: 0.0,
            context_usage: 0.0,
            accepted: 0,
            common: false,
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
    fn live_field_admission_is_owned_by_decision_core() {
        let admission = TransitionDecisionCore::admit_live_completion(&proposal(4, "ерка"));

        assert!(admission.visible(), "{admission:?}");
        assert_eq!(admission.reason, "field_visible");
    }

    #[test]
    fn learned_context_birth_can_show_long_suffix_after_short_prefix() {
        let mut candidate = proposal(1, "родолжительный");
        candidate.context_birth = true;
        candidate.l3_memory_supported = true;

        let admission = TransitionDecisionCore::admit_live_completion(&candidate);

        assert!(admission.visible(), "admission={admission:?}");
    }

    #[test]
    fn grounded_hot_candidate_is_visible_after_first_letter() {
        let mut candidate = proposal(1, "bounded-tail");
        candidate.hot = true;

        let admission = TransitionDecisionCore::admit_live_completion(&candidate);

        assert!(admission.visible(), "admission={admission:?}");
    }

    #[test]
    fn lexical_only_long_suffix_after_short_prefix_stays_hidden() {
        let candidate = proposal(1, "родолжительный");

        let admission = TransitionDecisionCore::admit_live_completion(&candidate);

        assert!(!admission.candidate_visible, "admission={admission:?}");
    }

    #[test]
    fn ungrounded_single_letter_suffix_stays_hidden() {
        let mut candidate = proposal(4, "е");
        candidate.completed_state_known = false;

        let admission = TransitionDecisionCore::admit_live_completion(&candidate);

        assert!(admission.candidate_visible);
        assert!(!admission.suffix_visible);
        assert_eq!(admission.reason, "field_suffix_not_grounded");
    }

    #[test]
    fn geometry_alone_cannot_authorize_an_unbound_single_letter_suffix() {
        let mut candidate = proposal(4, "ю");
        candidate.completed_state_known = false;
        candidate.score = 0.99;
        candidate.rank_score = 0.99;
        candidate.structural = 0.99;

        let admission = TransitionDecisionCore::admit_live_completion(&candidate);

        assert!(admission.candidate_visible);
        assert!(!admission.suffix_visible);
        assert_eq!(admission.reason, "field_suffix_not_grounded");
    }

    #[test]
    fn known_complete_word_allows_grounded_active_ime_extension_only() {
        let mut candidate = proposal(3, "extension");
        candidate.partial_state_known = true;

        let admission = TransitionDecisionCore::admit_live_completion(&candidate);
        assert!(admission.visible(), "{admission:?}");

        candidate.active_composition = false;
        let inactive = TransitionDecisionCore::admit_live_completion(&candidate);
        assert!(!inactive.candidate_visible, "{inactive:?}");

        candidate.context_birth = true;
        let supported = TransitionDecisionCore::admit_live_completion(&candidate);
        assert!(supported.visible(), "{supported:?}");
    }

    #[test]
    fn live_field_score_combines_l2_l3_l4_inside_decision_core() {
        let neutral = TransitionDecisionCore::score_live_completion_field(LiveFieldScoreInput {
            structural: 0.30,
            boundary_center_grounded: false,
            wave_peak_rank_bonus: 0.10,
            usage: 0.02,
            context_usage: 0.01,
            accepted: 1,
            common: true,
            hot: false,
            partial_len: 4,
            l4_signed_weight: 0.0,
            l3_rank_delta: 0.0,
        });
        let supported = TransitionDecisionCore::score_live_completion_field(LiveFieldScoreInput {
            l4_signed_weight: 1.0,
            l3_rank_delta: 0.12,
            ..LiveFieldScoreInput {
                structural: 0.30,
                boundary_center_grounded: false,
                wave_peak_rank_bonus: 0.10,
                usage: 0.02,
                context_usage: 0.01,
                accepted: 1,
                common: true,
                hot: false,
                partial_len: 4,
                l4_signed_weight: 0.0,
                l3_rank_delta: 0.0,
            }
        });
        let repelled = TransitionDecisionCore::score_live_completion_field(LiveFieldScoreInput {
            l4_signed_weight: -1.0,
            l3_rank_delta: -0.12,
            ..LiveFieldScoreInput {
                structural: 0.30,
                boundary_center_grounded: false,
                wave_peak_rank_bonus: 0.10,
                usage: 0.02,
                context_usage: 0.01,
                accepted: 1,
                common: true,
                hot: false,
                partial_len: 4,
                l4_signed_weight: 0.0,
                l3_rank_delta: 0.0,
            }
        });

        assert!(supported.rank_score > neutral.rank_score);
        assert!(repelled.rank_score < neutral.rank_score);
    }
}

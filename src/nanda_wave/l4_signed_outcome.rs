use super::l4_goal_state::{L4AllowedAction, L4EditIntent, L4LanguageScene, L4SceneState};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum L4OutcomePolarity {
    Attract,
    Neutral,
    Repel,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct L4SignedOutcome {
    pub(crate) attraction: f32,
    pub(crate) repulsion: f32,
    pub(crate) neutral: f32,
    pub(crate) signed_weight: f32,
    pub(crate) polarity: L4OutcomePolarity,
    pub(crate) reason: &'static str,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct L4SignedOutcomeInput<'a> {
    pub(crate) scene: &'a L4SceneState,
    pub(crate) candidate: &'a str,
    pub(crate) suffix: &'a str,
    pub(crate) partial_len: usize,
    pub(crate) structural: f32,
    pub(crate) usage: f32,
    pub(crate) context_usage: f32,
    pub(crate) accepted: u32,
    pub(crate) learned_attraction: f32,
    pub(crate) learned_repulsion: f32,
}

pub(crate) fn l4_signed_outcome(input: L4SignedOutcomeInput<'_>) -> L4SignedOutcome {
    let mut attraction = 0.0_f32;
    let mut repulsion = 0.0_f32;
    let mut neutral = 0.0_f32;
    let mut reason = match input.scene.allowed_action {
        L4AllowedAction::Suggest => {
            attraction += 0.090 * input.scene.confidence;
            "scene_attracts"
        }
        L4AllowedAction::Wait => {
            repulsion += 0.120 * input.scene.confidence;
            neutral += 0.080;
            "scene_waits"
        }
        L4AllowedAction::Block => {
            repulsion += 0.280 * input.scene.confidence;
            "scene_blocks"
        }
    };

    attraction += input.structural * 0.38;
    attraction += input.usage * 1.15;
    attraction += input.context_usage * 1.85;
    attraction += input.accepted.min(10) as f32 * 0.035;
    attraction += input.learned_attraction * 0.72;
    repulsion += input.learned_repulsion * 0.86;

    if input.accepted > 0 || input.context_usage >= 0.030 {
        reason = "usage_context_attracts";
    }
    if input.learned_attraction > input.learned_repulsion && input.learned_attraction >= 0.060 {
        reason = "learned_state_attracts";
    }
    if input.learned_repulsion > input.learned_attraction && input.learned_repulsion >= 0.060 {
        reason = "learned_state_repels";
    }

    if input.structural < 0.28 && input.usage < 0.020 && input.context_usage < 0.015 {
        repulsion += 0.090;
        neutral += 0.060;
        reason = "low_evidence_repels";
    }

    if single_letter_suffix_without_memory(input) {
        repulsion += 0.180;
        reason = "single_letter_without_memory";
    }

    if matches!(input.scene.language_scene, L4LanguageScene::Technical)
        && input.partial_len < 4
        && input.accepted == 0
    {
        repulsion += 0.140;
        reason = "technical_short_repels";
    }

    if matches!(input.scene.language_scene, L4LanguageScene::Mixed)
        && input.partial_len < 3
        && input.accepted == 0
    {
        repulsion += 0.100;
        reason = "mixed_short_repels";
    }

    if matches!(
        input.scene.edit_intent,
        L4EditIntent::Command | L4EditIntent::Code
    ) && input.context_usage == 0.0
        && input.accepted == 0
    {
        repulsion += 0.080;
        reason = "non_typing_no_context";
    }

    let signed_weight = (attraction - repulsion).clamp(-1.0, 1.0);
    let polarity = if signed_weight >= 0.140 {
        L4OutcomePolarity::Attract
    } else if signed_weight <= -0.140 {
        L4OutcomePolarity::Repel
    } else {
        neutral += 0.120;
        L4OutcomePolarity::Neutral
    };

    L4SignedOutcome {
        attraction: attraction.clamp(0.0, 1.0),
        repulsion: repulsion.clamp(0.0, 1.0),
        neutral: neutral.clamp(0.0, 1.0),
        signed_weight,
        polarity,
        reason,
    }
}

fn single_letter_suffix_without_memory(input: L4SignedOutcomeInput<'_>) -> bool {
    input.suffix.chars().count() == 1
        && !matches!(input.suffix, "и" | "я")
        && input.accepted < 2
        && input.context_usage < 0.055
        && input.usage < 0.085
        && input.structural < 0.48
        && input
            .candidate
            .chars()
            .count()
            .saturating_sub(input.partial_len)
            == 1
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scene(action: L4AllowedAction) -> L4SceneState {
        L4SceneState {
            language_scene: L4LanguageScene::Russian,
            edit_intent: L4EditIntent::Typing,
            allowed_action: action,
            confidence: 0.80,
            context_tokens: 3,
            reason: "test",
        }
    }

    #[test]
    fn signed_outcome_attracts_context_supported_candidate() {
        let outcome = l4_signed_outcome(L4SignedOutcomeInput {
            scene: &scene(L4AllowedAction::Suggest),
            candidate: "проверка",
            suffix: "ерка",
            partial_len: 4,
            structural: 0.42,
            usage: 0.04,
            context_usage: 0.05,
            accepted: 2,
            learned_attraction: 0.10,
            learned_repulsion: 0.0,
        });

        assert_eq!(outcome.polarity, L4OutcomePolarity::Attract);
        assert!(outcome.signed_weight > 0.0);
    }

    #[test]
    fn signed_outcome_repels_weak_single_letter_suffix() {
        let outcome = l4_signed_outcome(L4SignedOutcomeInput {
            scene: &scene(L4AllowedAction::Suggest),
            candidate: "будет",
            suffix: "т",
            partial_len: 4,
            structural: 0.10,
            usage: 0.0,
            context_usage: 0.0,
            accepted: 0,
            learned_attraction: 0.0,
            learned_repulsion: 0.0,
        });

        assert_eq!(outcome.polarity, L4OutcomePolarity::Repel);
        assert!(outcome.repulsion > outcome.attraction);
    }

    #[test]
    fn signed_outcome_keeps_watch_scene_neutral_when_evidence_is_thin() {
        let outcome = l4_signed_outcome(L4SignedOutcomeInput {
            scene: &scene(L4AllowedAction::Wait),
            candidate: "кандидат",
            suffix: "дат",
            partial_len: 5,
            structural: 0.24,
            usage: 0.0,
            context_usage: 0.0,
            accepted: 0,
            learned_attraction: 0.0,
            learned_repulsion: 0.0,
        });

        assert_eq!(outcome.polarity, L4OutcomePolarity::Neutral);
    }

    #[test]
    fn learned_repulsion_can_turn_thin_candidate_negative() {
        let outcome = l4_signed_outcome(L4SignedOutcomeInput {
            scene: &scene(L4AllowedAction::Suggest),
            candidate: "отвравим",
            suffix: "им",
            partial_len: 5,
            structural: 0.30,
            usage: 0.0,
            context_usage: 0.0,
            accepted: 0,
            learned_attraction: 0.0,
            learned_repulsion: 0.46,
        });

        assert_eq!(outcome.polarity, L4OutcomePolarity::Repel);
        assert_eq!(outcome.reason, "learned_state_repels");
    }
}

//! Shared IME correction decision layer.
//!
//! IME frontends own composition display and commit mechanics. They must ask
//! this layer for correction decisions instead of building an InputGate request
//! inside the frontend state machine.

use crate::action_log::RecentActionGateTrace;
use crate::config::{CorrectionSafety, LayConfig};
use crate::correction_core::CorrectionMode;
use crate::input_gate::{decide_input_gate, InputGateAction, InputGateRequest, InputGateTrigger};
use crate::text_edit::{
    plan_committed_tail_last_token_replacement, plan_input_gate_edit, plan_text_replacement,
    EditAction,
};

pub struct ActiveCompositionAutocorrectRequest<'a> {
    pub text: &'a str,
    pub committed_tail: &'a str,
    pub config: &'a LayConfig,
}

pub struct ActiveCompositionAutocorrectDecision {
    pub replacement: String,
    pub action: EditAction,
    pub input_gate: Option<RecentActionGateTrace>,
}

pub fn decide_active_composition_autocorrect(
    request: ActiveCompositionAutocorrectRequest<'_>,
) -> Option<ActiveCompositionAutocorrectDecision> {
    let (gate_text, active_prefix) =
        active_composition_gate_text(request.text, request.committed_tail);
    let gate_config = ActiveCompositionGateConfig::from_config(request.config);
    let decision = decide_input_gate(InputGateRequest {
        trigger: InputGateTrigger::Space,
        text_tail: &gate_text,
        auto_replace: gate_config.auto_replace,
        typing_assist: gate_config.typing_assist,
        auto_switch_layout: gate_config.auto_switch_layout,
        correction_safety: gate_config.correction_safety,
        typing_assist_pipeline: &request.config.typing_assist_pipeline,
        nanda_autocorrect: gate_config.nanda_autocorrect,
        nanda_candidate_route: crate::correction_core::CandidateReadoutRoute::CompactL2,
        nanda_wave_options: request.config.active_nanda_wave_options(),
        correction_mode: gate_config.correction_mode(),
    });
    let InputGateAction::ApplyReplacement {
        ref replacement, ..
    } = decision.action
    else {
        return None;
    };
    if replacement.as_str() == gate_text {
        return None;
    }
    let replacement = if active_prefix.is_empty() {
        replacement.clone()
    } else {
        replacement.strip_prefix(&active_prefix)?.to_string()
    };
    let plan = plan_committed_tail_last_token_replacement(request.text, &replacement)
        .or_else(|| plan_text_replacement(request.text, &replacement))?;
    let action = plan_input_gate_edit(
        "ibus-active-composition",
        request.text,
        &replacement,
        plan,
        &decision,
    );
    let input_gate = decision
        .trace
        .as_ref()
        .map(RecentActionGateTrace::from_input_gate)?;
    Some(ActiveCompositionAutocorrectDecision {
        replacement,
        action,
        input_gate: Some(input_gate),
    })
}

fn active_composition_gate_text(text: &str, committed_tail: &str) -> (String, String) {
    let active_word = text.trim_end_matches(char::is_whitespace);
    let visible_tail = committed_tail.trim_end_matches(char::is_whitespace);
    if active_word.is_empty() {
        return (text.to_string(), String::new());
    }
    let Some(prefix) = visible_tail.strip_suffix(active_word) else {
        return (text.to_string(), String::new());
    };
    if prefix.is_empty() {
        return (text.to_string(), String::new());
    }
    (format!("{prefix}{text}"), prefix.to_string())
}

#[derive(Debug, Clone, Copy)]
struct ActiveCompositionGateConfig {
    auto_replace: bool,
    typing_assist: bool,
    auto_switch_layout: bool,
    nanda_autocorrect: bool,
    correction_safety: CorrectionSafety,
}

impl ActiveCompositionGateConfig {
    fn from_config(config: &LayConfig) -> Self {
        Self {
            auto_replace: config.auto_replace,
            typing_assist: config.typing_assist,
            auto_switch_layout: config.auto_switch_layout,
            nanda_autocorrect: config.nanda_autocorrect,
            correction_safety: config.active_correction_safety(),
        }
    }

    fn correction_mode(self) -> CorrectionMode {
        if self.nanda_autocorrect {
            CorrectionMode::DeterministicThenNanda
        } else {
            CorrectionMode::DeterministicOnly
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        active_composition_gate_text, decide_active_composition_autocorrect,
        ActiveCompositionAutocorrectRequest,
    };
    use crate::config::LayConfig;

    fn config() -> LayConfig {
        LayConfig {
            text_backend: "ime".to_string(),
            auto_replace: true,
            typing_assist: true,
            auto_switch_layout: true,
            correction_safety: "experimental".to_string(),
            nanda_autocorrect: true,
            nanda_precognition: true,
            nanda_l2_phase_apply: false,
            ..LayConfig::default()
        }
    }

    #[test]
    fn active_composition_gate_text_preserves_committed_prefix_for_decision_only() {
        let (gate_text, prefix) = active_composition_gate_text("прохоил ", "я прохоил");

        assert_eq!(gate_text, "я прохоил ");
        assert_eq!(prefix, "я ");
    }

    #[test]
    fn active_composition_decision_returns_only_live_text_replacement() {
        let cfg = config();
        let decision =
            decide_active_composition_autocorrect(super::ActiveCompositionAutocorrectRequest {
                text: "прохоил ",
                committed_tail: "я прохоил",
                config: &cfg,
            })
            .expect("decision");

        assert_eq!(decision.replacement, "проходил ");
        assert_eq!(decision.action.from_text(), "прохоил ");
        assert!(decision.action.allow_apply());
    }

    #[test]
    fn active_composition_autocorrect_can_use_nanda_fallback() {
        assert_replacement("тфтвф ", "", "nanda ");
    }

    #[test]
    fn active_composition_autocorrect_uses_unified_input_gate() {
        assert_replacement("прохоил ", "я прохоил", "проходил ");
    }

    #[test]
    fn active_composition_context_replacement_keeps_previous_words_out_of_commit() {
        assert_replacement("ффективная ", "на сколько ффективная", "эффективная ");
    }

    #[test]
    fn committed_tail_autocorrect_can_use_tail_context_for_nanda() {
        assert_replacement("ghjdthrf ", "file ghjdthrf", "проверка ");
    }

    #[test]
    fn committed_tail_autocorrect_handles_ascii_tail_after_russian_context() {
        assert_replacement("ghjdthrf ", "проверка ghjdthrf", "проверка ");
    }

    #[test]
    fn committed_tail_autocorrect_handles_autozamena_layout_word() {
        assert_replacement("fdnjpfvtyf ", "fdnjpfvtyf", "автозамена ");
    }

    #[test]
    fn committed_tail_autocorrect_repairs_layout_word_with_missing_initial_letter() {
        assert_replacement("dnjpfvtyf ", "dnjpfvtyf", "автозамена ");
    }

    #[test]
    fn committed_tail_autocorrect_repairs_autozamena_mixed_prefix() {
        assert_replacement("fвтозамена ", "fвтозамена", "автозамена ");
    }

    #[test]
    fn committed_tail_autocorrect_repairs_duplicate_latin_prefix_before_russian_word() {
        assert_replacement("fавтозамена ", "fавтозамена", "автозамена ");
    }

    #[test]
    fn committed_tail_autocorrect_handles_plain_en_to_ru_layout_words() {
        assert_replacement("ghbdtn ", "ghbdtn", "привет ");
    }

    #[test]
    fn sequential_layout_words_keep_the_same_boundary_authority() {
        let cfg = config();
        let mut committed_tail = String::new();
        for (typed, expected) in [
            ("lfkmit ", "дальше "),
            ("yt ", "не "),
            ("gthtdjhfxbdftncz ", "переворачивается "),
        ] {
            let decision =
                decide_active_composition_autocorrect(ActiveCompositionAutocorrectRequest {
                    text: typed,
                    committed_tail: &committed_tail,
                    config: &cfg,
                })
                .unwrap_or_else(|| panic!("missing layout transition for {typed:?}"));
            assert_eq!(decision.replacement, expected);
            committed_tail.push_str(expected);
        }
    }

    #[test]
    fn committed_tail_boundary_uses_same_dual_layout_decision_as_cli() {
        let cfg = config();
        for (tail, token, expected) in [
            ("смотрим цусрфе", "цусрфе", "wechat "),
            ("проверяем вщцутдщфв", "вщцутдщфв", "download "),
            ("check ghbdtn", "ghbdtn", "привет "),
        ] {
            let text = format!("{token} ");
            let decision =
                decide_active_composition_autocorrect(ActiveCompositionAutocorrectRequest {
                    text: &text,
                    committed_tail: tail,
                    config: &cfg,
                })
                .unwrap_or_else(|| panic!("dual-layout decision for tail={tail:?}"));

            assert_eq!(decision.replacement, expected, "tail={tail:?}");
        }
    }

    #[test]
    fn committed_tail_boundary_keeps_valid_current_layout_word() {
        let cfg = config();
        let decision = decide_active_composition_autocorrect(ActiveCompositionAutocorrectRequest {
            text: "привет ",
            committed_tail: "смотрим привет",
            config: &cfg,
        });

        assert!(decision.is_none());
    }

    #[test]
    fn committed_tail_boundary_split_uses_the_shared_decision_core() {
        let cfg = config();
        let decision = decide_active_composition_autocorrect(ActiveCompositionAutocorrectRequest {
            text: "тоесть ",
            committed_tail: "тоесть",
            config: &cfg,
        })
        .expect("boundary decision");

        assert_eq!(decision.replacement, "то есть ");
        assert!(
            decision.action.allow_apply(),
            "source={:?} class={:?} confidence={} safety={} transition={:?}",
            decision.action.selected_source_id(),
            decision.action.selected_error_class(),
            decision.action.confidence_milli(),
            decision.action.safety_reason(),
            decision.action.transition(),
        );
    }

    #[test]
    fn committed_tail_autocorrect_keeps_ascii_layout_punctuation_in_token() {
        assert_replacement("ghj,ktvf ", "ghj,ktvf", "проблема ");
    }

    fn assert_replacement(text: &str, committed_tail: &str, expected: &str) {
        let cfg = config();
        let decision = decide_active_composition_autocorrect(ActiveCompositionAutocorrectRequest {
            text,
            committed_tail,
            config: &cfg,
        })
        .expect("decision");

        assert_eq!(decision.replacement, expected);
        assert!(decision.action.allow_apply());
    }
}

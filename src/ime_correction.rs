//! Shared IME correction decision layer.
//!
//! IME frontends own composition display and commit mechanics. They must ask
//! this layer for correction decisions instead of building an InputGate request
//! inside the frontend state machine.
//!
//! Route contract:
//! - IME/preedit is a display and completion route for an unfinished token.
//!   Tab may accept that visible completion.
//! - Space autocorrect is a committed-token route. It asks InputGate/DecisionCore
//!   for a verified edit plan and may apply only the resulting AuthorizedEdit.
//! - L2/L3/L4/Bayes are shared signal layers. They may rank, boost, suppress, or
//!   veto candidates in both routes, but they do not turn an IME completion into
//!   an autocorrect edit or bypass the Space-route verifier.
//!
//! Do not show full-token boundary/typo autocorrections as IME completions.

use crate::action_log::RecentActionGateTrace;
use crate::config::{CorrectionSafety, LayConfig};
use crate::correction_core::CorrectionMode;
use crate::input_gate::{decide_input_gate, InputGateAction, InputGateRequest, InputGateTrigger};
use crate::text_edit::TransitionProof;
use crate::text_edit::{
    plan_committed_tail_last_token_replacement, plan_input_gate_edit, plan_text_replacement,
    EditAction, TextReplacement,
};

pub struct ActiveCompositionAutocorrectRequest<'a> {
    pub text: &'a str,
    pub committed_tail: &'a str,
    pub config: &'a LayConfig,
    /// Layout that produced the live token. `None` is reserved for callers
    /// that do not own physical/IME layout evidence.
    pub active_layout_is_ru: Option<bool>,
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
        nanda_candidate_route: crate::correction_core::CandidateReadoutRoute::live_default(),
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
        let stripped = replacement.strip_prefix(&active_prefix)?;
        stripped.to_string()
    };
    let (action_from_text, plan) = if let Some(projection) =
        physical_committed_tail_projection_plan(request.text, request.committed_tail, &replacement)
    {
        projection
    } else {
        (
            request.text,
            plan_committed_tail_last_token_replacement(request.text, &replacement)
                .or_else(|| plan_text_replacement(request.text, &replacement))?,
        )
    };
    let action = plan_input_gate_edit(
        "ibus-active-composition",
        action_from_text,
        &replacement,
        plan,
        &decision,
    );
    if active_layout_preserves_known_token(
        action.from_text(),
        action.transition().proof(),
        request.active_layout_is_ru,
    ) {
        return None;
    }
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

/// Prevents automatic layout evidence from overturning an independently known
/// token that was typed in the currently active layout. Manual layout toggles
/// do not call this Space-route guard.
pub fn active_layout_preserves_known_token(
    token: &str,
    transition_proof: Option<TransitionProof>,
    active_layout_is_ru: Option<bool>,
) -> bool {
    if transition_proof != Some(TransitionProof::Layout) {
        return false;
    }
    let Some(active_layout_is_ru) = active_layout_is_ru else {
        return false;
    };
    // When the complete physical-key projection is a known Russian surface,
    // do not let generic ASCII protection suppress the exact layout
    // transition. This includes short words such as `yt` -> `не` as well as
    // internal punctuation keys (`;` -> `ж`). Technical and real English
    // tokens remain protected by the layout autoswitch classifier itself.
    if !active_layout_is_ru
        && crate::layout_autoswitch::correct_wrong_layout_ascii_word(token).is_some()
    {
        return false;
    }
    let identity = crate::word_recognizer::recognize_token(token.trim());
    if active_layout_is_ru {
        identity.is_known_russian_plain_word()
    } else {
        identity.is_known_ascii_or_protected_token()
    }
}

fn physical_committed_tail_projection_plan<'a>(
    text: &'a str,
    committed_tail: &str,
    replacement: &str,
) -> Option<(&'a str, TextReplacement)> {
    // Space has not been committed yet; the executor deletes the visible token
    // and inserts the replacement together with that pending separator.
    let action_from_text = text.trim_end_matches(char::is_whitespace);
    if action_from_text.is_empty() || action_from_text == text {
        return None;
    }
    let visible_tail = committed_tail.trim_end_matches(char::is_whitespace);
    if !visible_tail.ends_with(action_from_text) {
        return None;
    }
    Some((
        action_from_text,
        TextReplacement {
            move_left: 0,
            backspaces: action_from_text.chars().count() as u32,
            insert: replacement.to_string(),
            move_right: 0,
        },
    ))
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

    fn live_l2_phase_config() -> LayConfig {
        LayConfig {
            nanda_l2_phase_apply: true,
            ..config()
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
                active_layout_is_ru: None,
            })
            .expect("decision");

        assert_eq!(decision.replacement, "проходил ");
        assert_eq!(decision.action.from_text(), "прохоил");
        assert!(
            decision.action.allow_apply(),
            "replacement={:?} action={:?}",
            decision.replacement,
            decision.action
        );
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
                    active_layout_is_ru: None,
                })
                .unwrap_or_else(|| panic!("missing layout transition for {typed:?}"));
            assert_eq!(decision.replacement, expected);
            committed_tail.push_str(expected);
        }
    }

    #[test]
    fn live_l2_space_route_keeps_short_wrong_layout_function_word_authority() {
        let cfg = live_l2_phase_config();
        let decision = decide_active_composition_autocorrect(ActiveCompositionAutocorrectRequest {
            text: "yt ",
            committed_tail: "yt",
            config: &cfg,
            active_layout_is_ru: Some(false),
        })
        .expect("short wrong-layout function word");

        assert_eq!(decision.replacement, "не ");
        assert_eq!(decision.action.selected_error_class(), Some("wrong_layout"));
        assert!(decision.action.allow_apply());
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
                    active_layout_is_ru: None,
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
            active_layout_is_ru: None,
        });

        assert!(decision.is_none());
    }

    #[test]
    fn active_english_layout_preserves_known_ascii_token_from_layout_projection() {
        let cfg = config();
        let decision = decide_active_composition_autocorrect(ActiveCompositionAutocorrectRequest {
            text: "pdf ",
            committed_tail: "pdf",
            config: &cfg,
            active_layout_is_ru: Some(false),
        });

        assert!(
            decision.is_none(),
            "known active-layout token must be preserved"
        );
    }

    #[test]
    fn active_english_layout_does_not_protect_internal_layout_letter_symbol() {
        let cfg = live_l2_phase_config();
        let decision = decide_active_composition_autocorrect(ActiveCompositionAutocorrectRequest {
            text: "ye;ty ",
            committed_tail: "ye;ty",
            config: &cfg,
            active_layout_is_ru: Some(false),
        })
        .expect("internal layout-letter symbol must remain eligible for projection");

        assert_eq!(decision.replacement, "нужен ");
        assert_eq!(decision.action.selected_error_class(), Some("wrong_layout"));
        assert!(decision.action.allow_apply());
    }

    #[test]
    fn active_russian_layout_allows_unknown_cyrillic_projection_to_known_ascii() {
        let cfg = config();
        let decision = decide_active_composition_autocorrect(ActiveCompositionAutocorrectRequest {
            text: "зва ",
            committed_tail: "зва",
            config: &cfg,
            active_layout_is_ru: Some(true),
        })
        .expect("unknown active-layout surface may project to a known opposite-layout token");

        assert_eq!(decision.replacement, "pdf ");
    }

    #[test]
    fn active_layout_guard_does_not_block_non_layout_typo_repair() {
        let cfg = config();
        let decision = decide_active_composition_autocorrect(ActiveCompositionAutocorrectRequest {
            text: "прохоил ",
            committed_tail: "прохоил",
            config: &cfg,
            active_layout_is_ru: Some(true),
        })
        .expect("ordinary typo correction");

        assert_eq!(decision.replacement, "проходил ");
    }

    #[test]
    fn committed_tail_repairs_accidental_final_consonant_after_imperative() {
        let cfg = config();
        let decision = decide_active_composition_autocorrect(ActiveCompositionAutocorrectRequest {
            text: "читайл ",
            committed_tail: "читайл",
            config: &cfg,
            active_layout_is_ru: Some(true),
        })
        .expect("final-consonant extra-letter correction");

        assert_eq!(decision.replacement, "читай ");
        assert_eq!(decision.action.selected_error_class(), Some("extra-letter"));
        assert!(decision.action.allow_apply());
    }

    #[test]
    fn committed_tail_boundary_split_is_authorized_for_space_autocorrect() {
        let cfg = config();
        let decision = decide_active_composition_autocorrect(ActiveCompositionAutocorrectRequest {
            text: "тоесть ",
            committed_tail: "тоесть",
            config: &cfg,
            active_layout_is_ru: None,
        })
        .expect("boundary decision");

        assert_eq!(decision.replacement, "то есть ");
        assert_eq!(decision.action.selected_source_id(), Some("glued_phrase"));
        assert!(
            decision.action.allow_apply(),
            "action={:?}",
            decision.action
        );
    }

    #[test]
    fn repeated_boundary_token_remains_authorized_at_the_next_space() {
        assert_replacement("тоесть ", "тоесть тоесть", "то есть ");
    }

    #[test]
    fn boundary_split_survives_live_l2_phase_apply() {
        let cfg = live_l2_phase_config();
        let decision = decide_active_composition_autocorrect(ActiveCompositionAutocorrectRequest {
            text: "тоесть ",
            committed_tail: "тоесть",
            config: &cfg,
            active_layout_is_ru: None,
        })
        .expect("boundary decision");

        assert_eq!(decision.replacement, "то есть ");
        assert_eq!(decision.action.selected_source_id(), Some("glued_phrase"));
        assert!(
            decision.action.allow_apply(),
            "action={:?}",
            decision.action
        );
    }

    #[test]
    fn repeated_boundary_token_live_route_changes_only_current_token() {
        let cfg = live_l2_phase_config();
        let decision = decide_active_composition_autocorrect(ActiveCompositionAutocorrectRequest {
            text: "тоесть ",
            committed_tail: "тоесть тоесть",
            config: &cfg,
            active_layout_is_ru: None,
        })
        .expect("boundary decision");

        assert_eq!(decision.replacement, "то есть ");
        assert_eq!(decision.action.from_text(), "тоесть");
        assert_eq!(decision.action.to_text(), "то есть ");
        assert_eq!(decision.action.selected_source_id(), Some("glued_phrase"));
        assert!(
            decision.action.allow_apply(),
            "action={:?}",
            decision.action
        );
    }

    #[test]
    fn committed_tail_space_action_matches_physical_token_without_pending_space() {
        let cfg = live_l2_phase_config();
        let decision = decide_active_composition_autocorrect(ActiveCompositionAutocorrectRequest {
            text: "автозаменет ",
            committed_tail: "блять зайди в лог посмотреть как он автозаменет",
            config: &cfg,
            active_layout_is_ru: None,
        })
        .expect("autozamena decision");

        assert_eq!(decision.replacement, "автозамена ");
        assert_eq!(decision.action.from_text(), "автозаменет");
        assert_eq!(decision.action.to_text(), "автозамена ");
        let plan = decision.action.plan().expect("edit plan");
        assert_eq!(plan.move_left, 0);
        assert_eq!(plan.backspaces, "автозаменет".chars().count() as u32);
        assert_eq!(plan.insert, "автозамена ");
        assert_eq!(plan.move_right, 0);
        assert!(
            decision.action.allow_apply(),
            "action={:?}",
            decision.action
        );
    }

    #[test]
    fn committed_tail_space_route_uses_shared_decision_core_for_dirty_tokens() {
        let cfg = live_l2_phase_config();
        for (committed_tail, token, expected) in [
            ("ятут", "ятут", "я тут "),
            ("видешь", "видешь", "видишь "),
            ("за окном весь вечер идёт дожь", "дожь", "дождь "),
        ] {
            let text = format!("{token} ");
            let decision =
                decide_active_composition_autocorrect(ActiveCompositionAutocorrectRequest {
                    text: &text,
                    committed_tail,
                    config: &cfg,
                    active_layout_is_ru: None,
                })
                .unwrap_or_else(|| panic!("missing shared decision for {committed_tail:?}"));

            assert_eq!(decision.replacement, expected, "tail={committed_tail:?}");
            assert!(
                decision.action.allow_apply(),
                "tail={committed_tail:?} action={:?}",
                decision.action
            );
        }
    }

    #[test]
    fn committed_tail_context_recurrence_restores_unique_one_edit_word() {
        assert_replacement(
            "мло ",
            "сделать ошибку в слове мало и написать мло",
            "мало ",
        );
    }

    #[test]
    fn isolated_short_one_edit_word_remains_ambiguous() {
        let cfg = config();
        let decision = decide_active_composition_autocorrect(ActiveCompositionAutocorrectRequest {
            text: "мло ",
            committed_tail: "мло",
            config: &cfg,
            active_layout_is_ru: Some(true),
        });

        assert!(
            decision.is_none(),
            "decision={:?}",
            decision.map(|item| item.replacement)
        );
    }

    #[test]
    fn committed_tail_space_route_restores_trailing_function_boundaries() {
        for (tail, token, expected) in [
            ("Готовь документыдля", "документыдля ", "документы для "),
            ("Какие документыим", "документыим ", "документы им "),
        ] {
            assert_replacement(token, tail, expected);
        }
    }

    #[test]
    fn committed_tail_space_route_keeps_verified_inverse_length_member_of_l2_tie() {
        assert_replacement("перхвачу ", "клавиатурой не перхвачу", "перехвачу ");
    }

    #[test]
    fn committed_tail_space_route_defaults_to_canonical_l2_owner_for_l11_seeded_restore() {
        let cfg = config();
        let decision = decide_active_composition_autocorrect(ActiveCompositionAutocorrectRequest {
            text: "врмея ",
            committed_tail: "врмея",
            config: &cfg,
            active_layout_is_ru: None,
        })
        .expect("canonical live-owner decision");

        assert_eq!(decision.replacement, "время ");
        assert!(
            decision
                .action
                .selected_source_id()
                .is_some_and(|source_id| source_id.starts_with("CanonicalL2Field")),
            "selected_source_id={:?} action={:?}",
            decision.action.selected_source_id(),
            decision.action
        );
    }

    #[test]
    fn committed_tail_space_route_keeps_short_ambiguous_signal_unapplied() {
        let cfg = config();
        let decision = decide_active_composition_autocorrect(ActiveCompositionAutocorrectRequest {
            text: "пку ",
            committed_tail: "пку",
            config: &cfg,
            active_layout_is_ru: None,
        });

        assert!(
            decision.is_none(),
            "short ambiguous token must stay abstained on live route: {:?}",
            decision
                .as_ref()
                .map(|value| (&value.replacement, value.action.selected_source_id()))
        );
    }

    #[test]
    fn committed_tail_space_route_does_not_extend_known_imperative_to_infinitive() {
        let previous_policy = crate::hot_field::process_policy();
        crate::hot_field::set_process_policy(
            crate::hot_field::HotFieldPolicy::daemon_for_text_backend(
                crate::text_backend::TextBackendPreference::Ime,
            ),
        );

        for correction_safety in ["normal", "experimental"] {
            let mut cfg = live_l2_phase_config();
            cfg.correction_safety = correction_safety.to_string();
            let decision =
                decide_active_composition_autocorrect(ActiveCompositionAutocorrectRequest {
                    text: "посмотри ",
                    committed_tail: "давай там посмотри",
                    config: &cfg,
                    active_layout_is_ru: None,
                });

            assert!(
                decision.is_none(),
                "known imperative must not auto-grow into infinitive on Space: safety={correction_safety} replacement={:?}",
                decision.as_ref().map(|value| value.replacement.as_str())
            );
        }

        crate::hot_field::set_process_policy(previous_policy);
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
            active_layout_is_ru: None,
        })
        .expect("decision");

        assert_eq!(decision.replacement, expected);
        assert!(
            decision.action.allow_apply(),
            "replacement={:?} input_gate={:?} action={:?}",
            decision.replacement,
            decision.input_gate,
            decision.action
        );
    }
}

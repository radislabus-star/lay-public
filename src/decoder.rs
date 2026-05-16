//! Correction decoder.
//!
//! The decoder is the single place where lay chooses *what* should happen to a
//! buffered text tail. Runtime backends still decide *how* to execute the edit:
//! uinput replay, text insert, an IME bridge, or a future compositor-native
//! replace operation.

use crate::config::{CorrectionEngine, TypingAssistRuleConfig};
use crate::correction::Correction;
use crate::keyboard::{map_original_events, split_event_words, KeyEvent};
use crate::lem::ScoredCandidate;
use crate::text_edit::{
    ensure_committed_tail_spacing, plan_committed_tail_replacement, plan_text_replacement,
    TextReplacement,
};
use crate::typing_assist::{
    apply_auto_replace, apply_typing_assist_with_pipeline,
    decide_scoped_tail_correction_with_options, scoped_tail_lem_candidates, ScopedTailOptions,
};

const MANUAL_SCOPED_TAIL_MIN_MARGIN: f64 = 0.20;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CorrectionTrigger {
    Manual,
    AfterSpace,
    Enter,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CorrectionSource {
    Replay,
    SmartText,
    AutoReplace,
    TypingAssist,
    EnterAutocorrect,
}

impl CorrectionSource {
    pub fn log_kind(self) -> &'static str {
        match self {
            Self::Replay => "layout-replay",
            Self::SmartText => "smart-text",
            Self::AutoReplace => "auto-replace",
            Self::TypingAssist => "typing-assist",
            Self::EnterAutocorrect => "enter-autocorrect",
        }
    }

    pub fn needs_undo_checkpoint(self) -> bool {
        !matches!(self, Self::Replay)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DecoderAction {
    KeepOriginal,
    ReplayAll,
    ReplaceText {
        replacement: String,
        source: CorrectionSource,
    },
}

impl DecoderAction {
    pub fn replacement_text(&self) -> Option<&str> {
        match self {
            Self::ReplaceText { replacement, .. } => Some(replacement),
            Self::KeepOriginal | Self::ReplayAll => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecoderEditPlan {
    pub trigger: CorrectionTrigger,
    pub original: String,
    pub replacement: String,
    pub plan: TextReplacement,
    pub source: CorrectionSource,
}

impl DecoderEditPlan {
    pub fn committed_tail(
        trigger: CorrectionTrigger,
        original: &str,
        replacement: &str,
        source: CorrectionSource,
    ) -> Option<Self> {
        let replacement = ensure_committed_tail_spacing(original, replacement.to_string());
        let plan = match trigger {
            CorrectionTrigger::AfterSpace | CorrectionTrigger::Enter => {
                plan_committed_tail_replacement(original, &replacement)
            }
            CorrectionTrigger::Manual => plan_text_replacement(original, &replacement),
        }?;

        Some(Self {
            trigger,
            original: original.to_string(),
            replacement,
            plan,
            source,
        })
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ManualDecodeRequest<'a> {
    pub events: &'a [KeyEvent],
    pub original: &'a str,
    pub converted: &'a str,
    pub engine: CorrectionEngine,
    pub force_replay: bool,
    pub auto_replace: bool,
    pub scoped_options: ScopedTailOptions,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ManualDecodeResult {
    pub action: DecoderAction,
    pub edit: Option<DecoderEditPlan>,
    pub ranked: Option<RankedDecoderDecision>,
}

pub fn decode_manual_tail(request: ManualDecodeRequest<'_>) -> ManualDecodeResult {
    if request.force_replay || request.engine == CorrectionEngine::Replay {
        return maybe_apply_auto_replace(request, DecoderAction::ReplayAll, None);
    }

    let mut ranked = None;
    let action = if request.engine == CorrectionEngine::Smart {
        if let Some(decision) = choose_ranked_scoped_tail(request.events, request.scoped_options) {
            ranked = Some(decision.clone());
            DecoderAction::ReplaceText {
                replacement: decision.best.text,
                source: CorrectionSource::SmartText,
            }
        } else {
            decide_scoped_tail_correction_with_options(request.events, request.scoped_options)
                .filter(|text| !text.trim().is_empty())
                .map(|replacement| DecoderAction::ReplaceText {
                    replacement,
                    source: CorrectionSource::SmartText,
                })
                .unwrap_or_else(|| {
                    correction_to_action(crate::typing_assist::decide_correction(
                        request.original,
                        request.converted,
                        request.engine,
                    ))
                })
        }
    } else {
        correction_to_action(crate::typing_assist::decide_correction(
            request.original,
            request.converted,
            request.engine,
        ))
    };

    maybe_apply_auto_replace(request, action, ranked)
}

fn maybe_apply_auto_replace(
    request: ManualDecodeRequest<'_>,
    action: DecoderAction,
    ranked: Option<RankedDecoderDecision>,
) -> ManualDecodeResult {
    if !matches!(action, DecoderAction::ReplayAll) || !request.auto_replace {
        return manual_decode_result(request.original, action, ranked);
    }

    let Some(replacement) = apply_auto_replace(request.original, request.converted) else {
        return manual_decode_result(request.original, action, ranked);
    };

    if replacement == request.original
        || replacement == request.converted
        || replacement.trim().is_empty()
    {
        return manual_decode_result(request.original, action, ranked);
    }

    manual_decode_result(
        request.original,
        DecoderAction::ReplaceText {
            replacement,
            source: CorrectionSource::AutoReplace,
        },
        ranked,
    )
}

fn manual_decode_result(
    original: &str,
    action: DecoderAction,
    ranked: Option<RankedDecoderDecision>,
) -> ManualDecodeResult {
    let edit = match &action {
        DecoderAction::ReplaceText {
            replacement,
            source,
        } if !replacement.trim().is_empty() => DecoderEditPlan::committed_tail(
            CorrectionTrigger::Manual,
            original,
            replacement,
            *source,
        ),
        DecoderAction::KeepOriginal
        | DecoderAction::ReplayAll
        | DecoderAction::ReplaceText { .. } => None,
    };

    ManualDecodeResult {
        action,
        edit,
        ranked,
    }
}

fn correction_to_action(correction: Correction) -> DecoderAction {
    match correction {
        Correction::ReplayAll => DecoderAction::ReplayAll,
        Correction::InsertText(replacement) if replacement.trim().is_empty() => {
            DecoderAction::KeepOriginal
        }
        Correction::InsertText(replacement) => DecoderAction::ReplaceText {
            replacement,
            source: CorrectionSource::SmartText,
        },
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RankedDecoderCandidate {
    pub text: String,
    pub total: f64,
    pub language: f64,
    pub noise: f64,
    pub edit: f64,
    pub intervention: f64,
}

impl From<ScoredCandidate> for RankedDecoderCandidate {
    fn from(value: ScoredCandidate) -> Self {
        Self {
            text: value.text,
            total: value.total,
            language: value.language,
            noise: value.noise,
            edit: value.edit,
            intervention: value.intervention,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RankedDecoderDecision {
    pub original: String,
    pub best: RankedDecoderCandidate,
    pub second: Option<RankedDecoderCandidate>,
    pub margin: f64,
}

impl Eq for RankedDecoderDecision {}

impl Eq for RankedDecoderCandidate {}

pub fn rank_scoped_tail_candidates(
    events: &[KeyEvent],
    options: ScopedTailOptions,
) -> Option<RankedDecoderDecision> {
    if !options.lem_enabled {
        return None;
    }

    let words = split_event_words(events)?;
    if words.len() < 2 {
        return None;
    }

    let original = map_original_events(events);
    let has_trailing_space = events
        .last()
        .is_some_and(|event| event.keycode == evdev::KeyCode::KEY_SPACE.code());
    let candidates =
        scoped_tail_lem_candidates(&words, !has_trailing_space, options.allow_layout_auto)
            .into_iter()
            .map(|candidate| {
                if has_trailing_space {
                    format!("{candidate} ")
                } else {
                    candidate
                }
            });
    let ranked = crate::lem::rank_candidates(&original, candidates);
    let mut ranked = ranked.into_iter();
    let best: RankedDecoderCandidate = ranked.next()?.into();
    let second = ranked.next().map(RankedDecoderCandidate::from);
    let margin = second
        .as_ref()
        .map(|candidate| best.total - candidate.total)
        .unwrap_or(f64::INFINITY);

    Some(RankedDecoderDecision {
        original,
        best,
        second,
        margin,
    })
}

pub fn choose_ranked_scoped_tail(
    events: &[KeyEvent],
    options: ScopedTailOptions,
) -> Option<RankedDecoderDecision> {
    let decision = rank_scoped_tail_candidates(events, options)?;
    if decision.best.text == decision.original || decision.best.text.trim().is_empty() {
        return None;
    }
    if decision.margin < MANUAL_SCOPED_TAIL_MIN_MARGIN {
        return None;
    }
    Some(decision)
}

pub fn decode_typing_assist_tail(
    events: &[KeyEvent],
    allow_layout_auto: bool,
    pipeline: &[TypingAssistRuleConfig],
    source: CorrectionSource,
) -> Option<DecoderEditPlan> {
    let original = map_original_events(events);
    let replacement = apply_typing_assist_with_pipeline(&original, allow_layout_auto, pipeline)?;
    DecoderEditPlan::committed_tail(
        CorrectionTrigger::AfterSpace,
        &original,
        &replacement,
        source,
    )
}

pub fn decode_enter_autocorrect_tail(
    events: &[KeyEvent],
    original_has_trailing_space: bool,
    allow_layout_auto: bool,
    pipeline: &[TypingAssistRuleConfig],
) -> Option<DecoderEditPlan> {
    let original = map_original_events(events);
    if original.trim().is_empty() {
        return None;
    }

    let assist_input = if original_has_trailing_space {
        original.clone()
    } else {
        format!("{original} ")
    };
    let mut replacement =
        apply_typing_assist_with_pipeline(&assist_input, allow_layout_auto, pipeline)?;
    if original_has_trailing_space {
        replacement = ensure_committed_tail_spacing(&original, replacement);
    } else {
        replacement = replacement.trim_end().to_string();
    }

    if replacement == original || replacement.trim().is_empty() {
        None
    } else {
        DecoderEditPlan::committed_tail(
            CorrectionTrigger::Enter,
            &original,
            &replacement,
            CorrectionSource::EnterAutocorrect,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::default_typing_assist_pipeline;
    use evdev::KeyCode;

    fn ev(keycode: KeyCode, layout_is_ru: bool) -> KeyEvent {
        KeyEvent {
            keycode: keycode.code(),
            shift: false,
            layout_is_ru,
        }
    }

    fn events_for_ascii(text: &str) -> Vec<KeyEvent> {
        text.chars()
            .filter_map(|ch| {
                let key = match ch {
                    'a' => KeyCode::KEY_A,
                    'b' => KeyCode::KEY_B,
                    'c' => KeyCode::KEY_C,
                    'd' => KeyCode::KEY_D,
                    'e' => KeyCode::KEY_E,
                    'f' => KeyCode::KEY_F,
                    'g' => KeyCode::KEY_G,
                    'h' => KeyCode::KEY_H,
                    'i' => KeyCode::KEY_I,
                    'j' => KeyCode::KEY_J,
                    'k' => KeyCode::KEY_K,
                    'l' => KeyCode::KEY_L,
                    'm' => KeyCode::KEY_M,
                    'n' => KeyCode::KEY_N,
                    'o' => KeyCode::KEY_O,
                    'p' => KeyCode::KEY_P,
                    'q' => KeyCode::KEY_Q,
                    'r' => KeyCode::KEY_R,
                    's' => KeyCode::KEY_S,
                    't' => KeyCode::KEY_T,
                    'u' => KeyCode::KEY_U,
                    'v' => KeyCode::KEY_V,
                    'w' => KeyCode::KEY_W,
                    'x' => KeyCode::KEY_X,
                    'y' => KeyCode::KEY_Y,
                    'z' => KeyCode::KEY_Z,
                    ' ' => KeyCode::KEY_SPACE,
                    _ => return None,
                };
                Some(ev(key, false))
            })
            .collect()
    }

    #[test]
    fn manual_decoder_keeps_replay_as_explicit_user_command() {
        let events = events_for_ascii("good");
        let result = decode_manual_tail(ManualDecodeRequest {
            events: &events,
            original: "good",
            converted: "пщщв",
            engine: CorrectionEngine::Smart,
            force_replay: true,
            auto_replace: true,
            scoped_options: ScopedTailOptions::default(),
        });

        assert_eq!(result.action, DecoderAction::ReplayAll);
    }

    #[test]
    fn manual_decoder_uses_smart_tail_for_mixed_two_words() {
        let events = events_for_ascii("good ntrcn");
        let result = decode_manual_tail(ManualDecodeRequest {
            events: &events,
            original: "good ntrcn",
            converted: "пщщв текст",
            engine: CorrectionEngine::Smart,
            force_replay: false,
            auto_replace: false,
            scoped_options: ScopedTailOptions {
                lem_enabled: true,
                allow_layout_auto: true,
            },
        });

        assert_eq!(
            result.action,
            DecoderAction::ReplaceText {
                replacement: "good текст".to_string(),
                source: CorrectionSource::SmartText,
            }
        );
        assert_eq!(
            result.edit.expect("manual edit").plan,
            TextReplacement {
                move_left: 0,
                backspaces: 5,
                insert: "текст".to_string(),
                move_right: 0,
            }
        );
    }

    #[test]
    fn typing_assist_decoder_preserves_committed_space_boundary() {
        let events = events_for_ascii("double b ");
        let plan = decode_typing_assist_tail(
            &events,
            true,
            &default_typing_assist_pipeline(),
            CorrectionSource::TypingAssist,
        )
        .expect("assist plan");

        assert_eq!(plan.replacement, "double и ");
        assert_eq!(
            plan.plan,
            TextReplacement {
                move_left: 1,
                backspaces: 1,
                insert: "и".to_string(),
                move_right: 1,
            }
        );
        assert!(plan.source.needs_undo_checkpoint());
    }
}

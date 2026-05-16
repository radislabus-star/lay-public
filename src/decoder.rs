//! Correction decoder.
//!
//! The decoder is the single place where lay chooses *what* should happen to a
//! buffered text tail. Runtime backends still decide *how* to execute the edit:
//! uinput replay, text insert, an IME bridge, or a future compositor-native
//! replace operation.

use crate::config::{CorrectionEngine, TypingAssistRuleConfig};
use crate::correction::Correction;
use crate::keyboard::{map_original_events, KeyEvent};
use crate::text_edit::{
    ensure_committed_tail_spacing, plan_committed_tail_replacement, plan_text_replacement,
    TextReplacement,
};
use crate::typing_assist::{
    apply_auto_replace, apply_typing_assist_with_pipeline,
    decide_scoped_tail_correction_with_options, ScopedTailOptions,
};

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
}

pub fn decode_manual_tail(request: ManualDecodeRequest<'_>) -> ManualDecodeResult {
    if request.force_replay || request.engine == CorrectionEngine::Replay {
        return maybe_apply_auto_replace(request, DecoderAction::ReplayAll);
    }

    let action = if request.engine == CorrectionEngine::Smart {
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
    } else {
        correction_to_action(crate::typing_assist::decide_correction(
            request.original,
            request.converted,
            request.engine,
        ))
    };

    maybe_apply_auto_replace(request, action)
}

fn maybe_apply_auto_replace(
    request: ManualDecodeRequest<'_>,
    action: DecoderAction,
) -> ManualDecodeResult {
    if !matches!(action, DecoderAction::ReplayAll) || !request.auto_replace {
        return ManualDecodeResult { action };
    }

    let Some(replacement) = apply_auto_replace(request.original, request.converted) else {
        return ManualDecodeResult { action };
    };

    if replacement == request.original
        || replacement == request.converted
        || replacement.trim().is_empty()
    {
        return ManualDecodeResult { action };
    }

    ManualDecodeResult {
        action: DecoderAction::ReplaceText {
            replacement,
            source: CorrectionSource::AutoReplace,
        },
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

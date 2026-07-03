use lay::config::TypingAssistRuleConfig;
use lay::decoder::{CorrectionSource, DecoderEditPlan};
use lay::keyboard::KeyEvent;
use lay::typing_context::completed_tail_context;
use lay::word_buffer::WordBuffer;

#[cfg(not(test))]
use super::super::super::{
    active_auto_replace, active_auto_switch_layout, active_correction_safety,
    active_nanda_autocorrect, active_typing_assist, active_typing_assist_pipeline_for_auto_replace,
};

pub(super) fn decode_completed_tail(
    buf: &WordBuffer,
    word_count: usize,
    events: &[KeyEvent],
    allow_layout_auto: bool,
) -> Option<DecodedCompletedTail> {
    let context = completed_tail_context(buf, word_count, events);
    let pipeline = active_pipeline(&context);
    if let Some(decoded) = decode_input_gate_tail(events, &context, allow_layout_auto, &pipeline) {
        return Some(decoded);
    }
    None
}

#[derive(Debug, Clone)]
pub(super) struct DecodedCompletedTail {
    pub(super) edit: DecoderEditPlan,
    pub(super) rule_id: Option<String>,
    pub(super) input_gate: Option<lay::action_log::RecentActionGateTrace>,
}

impl DecodedCompletedTail {
    fn with_input_gate(
        edit: DecoderEditPlan,
        rule_id: Option<String>,
        input_gate: Option<lay::action_log::RecentActionGateTrace>,
    ) -> Self {
        Self {
            edit,
            rule_id,
            input_gate,
        }
    }
}

fn decode_input_gate_tail(
    events: &[KeyEvent],
    context: &str,
    allow_layout_auto: bool,
    pipeline: &[TypingAssistRuleConfig],
) -> Option<DecodedCompletedTail> {
    let original = lay::keyboard::map_original_events(events);
    let text_tail = if context.ends_with(&original) {
        context
    } else {
        &original
    };
    let gate_config = GateRuntimeConfig::active(allow_layout_auto);
    let decision = lay::input_gate::decide_input_gate(lay::input_gate::InputGateRequest {
        trigger: lay::input_gate::InputGateTrigger::Space,
        text_tail,
        auto_replace: gate_config.auto_replace,
        typing_assist: gate_config.typing_assist,
        auto_switch_layout: gate_config.auto_switch_layout,
        correction_safety: gate_config.correction_safety,
        typing_assist_pipeline: pipeline,
        nanda_autocorrect: gate_config.nanda_autocorrect,
        correction_mode: lay::correction_core::CorrectionMode::DeterministicThenNanda,
    });
    let lay::input_gate::InputGateAction::ApplyReplacement { replacement, .. } = &decision.action
    else {
        return None;
    };
    let replacement = replacement.clone();

    let (original_tail, replacement_tail) = if text_tail == context {
        let prefix = context.strip_suffix(&original)?;
        if prefix.is_empty() {
            (original.as_str(), replacement.as_str())
        } else {
            let replacement_tail = replacement.strip_prefix(prefix)?;
            if prefix.ends_with(char::is_whitespace)
                && replacement_tail.chars().count() > original.chars().count()
            {
                let separator = prefix
                    .chars()
                    .next_back()
                    .expect("prefix ends with whitespace");
                let anchored_original = format!("{separator}{original}");
                let anchored_replacement = format!("{separator}{replacement_tail}");
                return build_input_gate_decoded_tail(
                    decision,
                    &anchored_original,
                    &anchored_replacement,
                );
            }
            (original.as_str(), replacement_tail)
        }
    } else {
        (original.as_str(), replacement.as_str())
    };
    build_input_gate_decoded_tail(decision, original_tail, replacement_tail)
}

fn build_input_gate_decoded_tail(
    decision: lay::input_gate::InputGateDecision,
    original: &str,
    replacement_tail: &str,
) -> Option<DecodedCompletedTail> {
    let edit = DecoderEditPlan::committed_tail(
        lay::decoder::CorrectionTrigger::AfterSpace,
        original,
        replacement_tail,
        CorrectionSource::TypingAssist,
    )?;
    let rule_id = decision
        .correction
        .as_ref()
        .and_then(|resolution| resolution.selected.as_ref())
        .map(|candidate| {
            if candidate.source == lay::correction_core::CorrectionDecisionSource::Nanda {
                lay::typing_assist::NANDA_WAVE_RULE_ID.to_string()
            } else {
                candidate.source_id.clone()
            }
        });
    let input_gate = decision
        .trace
        .as_ref()
        .map(lay::action_log::RecentActionGateTrace::from_input_gate);
    Some(DecodedCompletedTail::with_input_gate(
        edit, rule_id, input_gate,
    ))
}

#[derive(Debug, Clone, Copy)]
struct GateRuntimeConfig {
    auto_replace: bool,
    typing_assist: bool,
    auto_switch_layout: bool,
    nanda_autocorrect: bool,
    correction_safety: lay::config::CorrectionSafety,
}

impl GateRuntimeConfig {
    fn active(allow_layout_auto: bool) -> Self {
        Self {
            auto_replace: active_auto_replace_for_gate(),
            typing_assist: active_typing_assist_for_gate(),
            auto_switch_layout: allow_layout_auto && active_auto_switch_layout_for_gate(),
            nanda_autocorrect: active_nanda_autocorrect_for_gate(),
            correction_safety: active_correction_safety_for_gate(),
        }
    }
}

#[cfg(test)]
fn active_pipeline(context: &str) -> Vec<TypingAssistRuleConfig> {
    lay::typing_context::typing_assist_pipeline_for_context(
        true,
        lay::config::CorrectionSafety::Normal,
        &lay::config::default_typing_assist_pipeline(),
        context,
    )
}

#[cfg(not(test))]
fn active_pipeline(context: &str) -> Vec<TypingAssistRuleConfig> {
    active_typing_assist_pipeline_for_auto_replace(context)
}

#[cfg(test)]
fn active_auto_replace_for_gate() -> bool {
    true
}

#[cfg(not(test))]
fn active_auto_replace_for_gate() -> bool {
    active_auto_replace()
}

#[cfg(test)]
fn active_typing_assist_for_gate() -> bool {
    true
}

#[cfg(not(test))]
fn active_typing_assist_for_gate() -> bool {
    active_typing_assist()
}

#[cfg(test)]
fn active_auto_switch_layout_for_gate() -> bool {
    true
}

#[cfg(not(test))]
fn active_auto_switch_layout_for_gate() -> bool {
    active_auto_switch_layout()
}

#[cfg(test)]
fn active_nanda_autocorrect_for_gate() -> bool {
    false
}

#[cfg(not(test))]
fn active_nanda_autocorrect_for_gate() -> bool {
    active_nanda_autocorrect()
}

#[cfg(test)]
fn active_correction_safety_for_gate() -> lay::config::CorrectionSafety {
    lay::config::CorrectionSafety::Normal
}

#[cfg(not(test))]
fn active_correction_safety_for_gate() -> lay::config::CorrectionSafety {
    active_correction_safety()
}

#[cfg(test)]
mod tests {
    use super::decode_completed_tail;
    use lay::keyboard::{text_to_key_events, KeyEvent};
    use lay::text_edit::apply_replacement_plan_to_text;
    use lay::word_buffer::WordBuffer;

    fn push_text_as_layout(buffer: &mut WordBuffer, text: &str, layout_is_ru: bool) {
        for event in text_events(text, layout_is_ru) {
            if lay::keyboard::original_event_char(&event) == Some(' ') {
                buffer.handle_space();
            } else {
                buffer.push(event);
            }
        }
    }

    fn text_events(text: &str, layout_is_ru: bool) -> Vec<KeyEvent> {
        text_to_key_events(text, layout_is_ru).expect("text must map to key events")
    }

    #[test]
    fn input_gate_context_tail_keeps_left_space_anchor_for_longer_word_fix() {
        let mut buffer = WordBuffer::new();
        push_text_as_layout(&mut buffer, "я прохоил ", true);
        let events = buffer
            .last_completed_words_events(1)
            .expect("last completed word");

        let decoded = decode_completed_tail(&buffer, 1, &events, true).expect("decoded");

        assert_eq!(decoded.edit.original, " прохоил ");
        assert_eq!(decoded.edit.replacement, " проходил ");
        assert_eq!(
            apply_replacement_plan_to_text(&decoded.edit.original, &decoded.edit.plan),
            decoded.edit.replacement
        );
    }

    #[test]
    fn input_gate_prefers_effective_for_missing_initial_vowel_tail() {
        let mut buffer = WordBuffer::new();
        push_text_as_layout(&mut buffer, "на сколько ффективная ", true);
        let events = buffer
            .last_completed_words_events(1)
            .expect("last completed word");

        let decoded = decode_completed_tail(&buffer, 1, &events, true).expect("decoded");

        assert_eq!(decoded.edit.original, " ффективная ");
        assert_eq!(decoded.edit.replacement, " эффективная ");
        assert_eq!(
            apply_replacement_plan_to_text(&decoded.edit.original, &decoded.edit.plan),
            decoded.edit.replacement
        );
    }
}

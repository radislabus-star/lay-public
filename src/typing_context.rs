//! Dynamic typing-assist policy.
//!
//! Static config says which rule families are generally allowed. This module
//! makes narrow per-context adjustments when the surrounding text gives a strong
//! signal that a normally risky rule is safe enough for live auto-replace.

use crate::config::{
    normalize_typing_assist_pipeline, typing_assist_pipeline_for_policy, CorrectionSafety,
    TypingAssistRuleConfig,
};
use crate::layout_autoswitch::{
    ascii_layout_prefix_can_be_letter, correct_wrong_layout_ascii_word,
    is_ascii_layout_letter_symbol, is_confident_wrong_layout_ascii_pair,
};
use crate::lexicon::is_common_ru_word;
use crate::word_reader::split_word_punctuation;
use crate::word_reader::{split_edge_whitespace, split_ws_segments};
use crate::word_recognizer::{recognize_token, WordKind, WordScript};

const ASCII_TO_RU_RULE: &str = "layout_en_to_ru";
const CONTEXTUAL_ASCII_TO_RU_RULE: &str = "contextual_layout_en_to_ru";

pub fn typing_assist_pipeline_for_context(
    auto_replace: bool,
    safety: CorrectionSafety,
    configured: &[TypingAssistRuleConfig],
    context: &str,
) -> Vec<TypingAssistRuleConfig> {
    let mut pipeline = typing_assist_pipeline_for_policy(auto_replace, safety, configured);
    if auto_replace
        && safety == CorrectionSafety::Normal
        && should_enable_ascii_to_ru_layout(context)
        && user_config_allows_rule(configured, ASCII_TO_RU_RULE)
    {
        pipeline.push(TypingAssistRuleConfig {
            id: CONTEXTUAL_ASCII_TO_RU_RULE.to_string(),
            enabled: true,
            priority: contextual_ascii_to_ru_priority(&pipeline),
        });
        pipeline.sort_by(|a, b| a.priority.cmp(&b.priority).then_with(|| a.id.cmp(&b.id)));
    }
    pipeline
}

pub fn should_enable_ascii_to_ru_layout(context: &str) -> bool {
    let (_, core, _) = split_edge_whitespace(context);
    let tokens: Vec<&str> = split_ws_segments(core)
        .into_iter()
        .filter_map(|(segment, is_ws)| (!is_ws).then_some(segment))
        .collect();
    let Some((last, rest)) = tokens.split_last() else {
        return false;
    };

    let last_strong_layout = strong_ascii_to_ru_layout_candidate(last);
    let last_clean_layout = clean_ascii_to_ru_layout_candidate(last);
    let previous_clean_layout = rest
        .last()
        .is_some_and(|previous| clean_ascii_to_ru_layout_candidate(previous));
    let phrase_layout = rest
        .last()
        .is_some_and(|previous| is_confident_wrong_layout_ascii_pair(previous, last));
    if !(last_strong_layout || last_clean_layout && previous_clean_layout || phrase_layout) {
        return false;
    }

    let layout_punctuation = has_layout_punctuation_signal(last);
    let previous_allows_contextual_layout = rest.last().is_some_and(|previous| {
        is_russian_context_token(previous)
            || (last_clean_layout
                && clean_ascii_to_ru_layout_candidate(previous)
                && !layout_punctuation)
            || phrase_layout
            || (layout_punctuation && is_natural_english_context_token(previous))
    });
    let standalone_layout_punctuation = rest.is_empty() && layout_punctuation;

    previous_allows_contextual_layout || standalone_layout_punctuation
}

fn user_config_allows_rule(configured: &[TypingAssistRuleConfig], id: &str) -> bool {
    normalize_typing_assist_pipeline(configured)
        .iter()
        .find(|rule| rule.id == id)
        .is_some_and(|rule| rule.enabled)
}

fn contextual_ascii_to_ru_priority(pipeline: &[TypingAssistRuleConfig]) -> i32 {
    pipeline
        .iter()
        .find(|rule| rule.id == ASCII_TO_RU_RULE)
        .map(|rule| rule.priority.saturating_sub(1).max(1))
        .unwrap_or(99)
}

fn strong_ascii_to_ru_layout_candidate(token: &str) -> bool {
    let identity = recognize_token(token);
    if identity.kind != WordKind::PlainWord
        || identity.script != WordScript::Ascii
        || identity.technical
        || identity.protected
    {
        return false;
    }

    let Some(converted) = correct_wrong_layout_ascii_word(token) else {
        return false;
    };
    if !identity.known_en {
        return true;
    }

    let (leading, _, _) = split_word_punctuation(token);
    let (_, converted_word, _) = split_word_punctuation(&converted);
    ascii_layout_prefix_can_be_letter(leading) && is_common_ru_word(&converted_word.to_lowercase())
}

fn clean_ascii_to_ru_layout_candidate(token: &str) -> bool {
    if !token.chars().all(|ch| ch.is_ascii_alphabetic()) {
        return false;
    }
    let identity = recognize_token(token);
    identity.kind == WordKind::PlainWord
        && identity.script == WordScript::Ascii
        && !identity.technical
        && !identity.protected
        && correct_wrong_layout_ascii_word(token).is_some()
}

fn has_layout_punctuation_signal(token: &str) -> bool {
    let (leading, _, trailing) = split_word_punctuation(token);
    ascii_layout_prefix_can_be_letter(leading)
        || ascii_layout_prefix_can_be_letter(trailing)
        || token.chars().any(is_ascii_layout_letter_symbol)
}

fn is_russian_context_token(token: &str) -> bool {
    let identity = recognize_token(token);
    matches!(
        identity.kind,
        WordKind::PlainWord | WordKind::TechnicalToken
    ) && identity.script == WordScript::Cyrillic
        && !identity.technical
}

fn is_natural_english_context_token(token: &str) -> bool {
    if token.chars().any(|ch| {
        matches!(
            ch,
            '\'' | ';'
                | '['
                | ']'
                | '`'
                | ','
                | '.'
                | '?'
                | '!'
                | ':'
                | '$'
                | '%'
                | '^'
                | '&'
                | '|'
                | '#'
                | '@'
                | '/'
                | '\\'
        )
    }) {
        return false;
    }
    let identity = recognize_token(token);
    identity.kind == WordKind::PlainWord
        && identity.script == WordScript::Ascii
        && identity.known_en
        && !identity.technical
        && !identity.protected
}

#[cfg(test)]
#[path = "typing_context_tests.rs"]
mod tests;

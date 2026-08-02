use crate::ru_typo::repair_extra_letters_after_layout;
use crate::word_reader::{is_cyrillic_word, split_word_punctuation, split_ws_segments};

use super::super::types::TypingRuleContext;

pub(super) type TextRule = fn(&str) -> Option<String>;

pub(super) fn layout_auto_allowed(ctx: &TypingRuleContext<'_>) -> bool {
    ctx.allow_layout_auto
}

pub(super) fn apply_core_then_word_rule(
    ctx: &TypingRuleContext<'_>,
    rule: TextRule,
) -> Option<String> {
    rule(ctx.core).or_else(|| apply_token_word_rule(ctx, rule))
}

pub(super) fn apply_short_left_word_rule(
    ctx: &TypingRuleContext<'_>,
    rule: TextRule,
) -> Option<String> {
    let segments = split_ws_segments(ctx.core);
    let word_indices = segments
        .iter()
        .enumerate()
        .filter_map(|(idx, (_, is_ws))| (!*is_ws).then_some(idx))
        .collect::<Vec<_>>();
    if word_indices.len() < 2 {
        return None;
    }
    let left_idx = word_indices[word_indices.len() - 2];
    let right_idx = word_indices[word_indices.len() - 1];
    let (left_leading, left_word, left_trailing) = split_word_punctuation(segments[left_idx].0);
    let (right_leading, right_word, right_trailing) = split_word_punctuation(segments[right_idx].0);
    if left_word.is_empty()
        || right_word.is_empty()
        || !left_trailing.is_empty()
        || !right_leading.is_empty()
        || !crate::phrase_lexicon::is_short_russian_function_word(&left_word.to_lowercase())
    {
        return None;
    }
    let replacement = rule(right_word)?;
    if replacement == right_word {
        return None;
    }

    let mut output = String::with_capacity(ctx.core.len() + replacement.len());
    for (idx, (segment, _is_ws)) in segments.iter().enumerate() {
        if idx == left_idx {
            output.push_str(left_leading);
            output.push_str(left_word);
        } else if idx == right_idx {
            output.push_str(right_leading);
            output.push_str(&replacement);
            output.push_str(right_trailing);
        } else {
            output.push_str(segment);
        }
    }
    Some(output)
}

pub(super) fn apply_last_two_word_rule(
    ctx: &TypingRuleContext<'_>,
    rule: TextRule,
) -> Option<String> {
    let segments = split_ws_segments(ctx.core);
    let word_indices = segments
        .iter()
        .enumerate()
        .filter_map(|(idx, (_, is_ws))| (!*is_ws).then_some(idx))
        .collect::<Vec<_>>();
    if word_indices.len() < 3 {
        return None;
    }
    let start = word_indices[word_indices.len() - 2];
    let suffix = segments[start..]
        .iter()
        .map(|(segment, _)| *segment)
        .collect::<String>();
    let replacement = rule(&suffix)?;
    let mut output = String::with_capacity(ctx.core.len().max(replacement.len()));
    for (segment, _) in &segments[..start] {
        output.push_str(segment);
    }
    output.push_str(&replacement);
    Some(output)
}

pub(super) fn apply_token_word_rule(ctx: &TypingRuleContext<'_>, rule: TextRule) -> Option<String> {
    if ctx.word.is_empty() {
        return None;
    }
    rule(ctx.word)
        .map(|replacement| format!("{}{}{}", ctx.token_leading, replacement, ctx.token_trailing))
}

pub(super) fn apply_last_word_rule(ctx: &TypingRuleContext<'_>, rule: TextRule) -> Option<String> {
    let segments = split_ws_segments(ctx.core);
    let idx = segments.iter().rposition(|(_, is_ws)| !*is_ws)?;
    let (leading, word, trailing) = split_word_punctuation(segments[idx].0);
    if word.is_empty() {
        return None;
    }
    let replacement = rule(word)?;
    if replacement == word {
        return None;
    }

    let mut output = String::with_capacity(ctx.core.len() + replacement.len());
    for (segment_idx, (segment, _)) in segments.iter().enumerate() {
        if segment_idx == idx {
            output.push_str(leading);
            output.push_str(&replacement);
            output.push_str(trailing);
        } else {
            output.push_str(segment);
        }
    }
    Some(output)
}

pub(super) fn cleanup_extra_letters_after_ru_layout(text: &str) -> String {
    let mut changed = false;
    let repaired = text
        .split_whitespace()
        .map(|part| {
            let (leading, word, trailing) = split_word_punctuation(part);
            if word.is_empty() || !is_cyrillic_word(word) {
                return part.to_string();
            }
            let Some(replacement) = repair_extra_letters_after_layout(word) else {
                return part.to_string();
            };
            changed = true;
            format!("{leading}{replacement}{trailing}")
        })
        .collect::<Vec<_>>()
        .join(" ");
    if changed {
        repaired
    } else {
        text.to_string()
    }
}

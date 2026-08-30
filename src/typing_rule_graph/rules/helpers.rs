use crate::ru_typo::repair_extra_letters_after_layout;
use crate::word_reader::{
    is_cyrillic_word, previous_non_whitespace_segment, split_word_punctuation, split_ws_segments,
};

use super::super::types::TypingRuleContext;

pub(super) type TextRule = fn(&str) -> Option<String>;

pub(super) fn layout_auto_allowed(ctx: &TypingRuleContext<'_>) -> bool {
    ctx.allow_layout_auto
}

pub(super) fn apply_core_then_word_rule(
    ctx: &TypingRuleContext<'_>,
    rule: TextRule,
) -> Option<String> {
    rule(ctx.core).or_else(|| {
        (!last_word_has_protected_ascii_context(ctx))
            .then(|| apply_token_word_rule(ctx, rule))
            .flatten()
    })
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

pub(super) fn apply_last_physical_layout_token_rule(
    ctx: &TypingRuleContext<'_>,
    rule: TextRule,
) -> Option<String> {
    let segments = split_ws_segments(ctx.core);
    let idx = segments.iter().rposition(|(_, is_ws)| !*is_ws)?;
    let token = segments[idx].0;
    if !token_has_physical_layout_prefix(token)
        || previous_non_whitespace_segment(&segments, idx)
            .is_some_and(crate::word_recognizer::is_protected_ascii_token)
    {
        return None;
    }
    let replacement = rule(token)?;
    if replacement == token {
        return None;
    }

    let mut output = String::with_capacity(ctx.core.len().max(replacement.len()));
    for (segment_idx, (segment, _)) in segments.iter().enumerate() {
        if segment_idx == idx {
            output.push_str(&replacement);
        } else {
            output.push_str(segment);
        }
    }
    Some(output)
}

pub(super) fn apply_last_trailing_layout_token_rule(
    ctx: &TypingRuleContext<'_>,
    rule: TextRule,
) -> Option<String> {
    let segments = split_ws_segments(ctx.core);
    let idx = segments.iter().rposition(|(_, is_ws)| !*is_ws)?;
    let token = segments[idx].0;
    let (leading, word, trailing) = split_word_punctuation(token);
    if !leading.is_empty()
        || word.is_empty()
        || trailing.is_empty()
        || !trailing
            .chars()
            .all(crate::layout_autoswitch::is_ascii_layout_token_symbol)
    {
        return None;
    }
    let replacement = rule(token)?;
    if replacement == token {
        return None;
    }

    let mut output = String::with_capacity(ctx.core.len().max(replacement.len()));
    for (segment_idx, (segment, _)) in segments.iter().enumerate() {
        if segment_idx == idx {
            output.push_str(&replacement);
        } else {
            output.push_str(segment);
        }
    }
    Some(output)
}

pub(super) fn last_token_has_physical_layout_prefix(ctx: &TypingRuleContext<'_>) -> bool {
    split_ws_segments(ctx.core)
        .into_iter()
        .rev()
        .find_map(|(segment, is_ws)| (!is_ws).then_some(segment))
        .is_some_and(token_has_physical_layout_prefix)
}

pub(super) fn last_word_has_protected_ascii_context(ctx: &TypingRuleContext<'_>) -> bool {
    let segments = split_ws_segments(ctx.core);
    let Some(idx) = segments.iter().rposition(|(_, is_ws)| !*is_ws) else {
        return false;
    };
    previous_non_whitespace_segment(&segments, idx)
        .is_some_and(crate::word_recognizer::is_protected_ascii_token)
}

fn token_has_physical_layout_prefix(token: &str) -> bool {
    let (leading, word, _) = split_word_punctuation(token);
    !leading.is_empty()
        && !word.is_empty()
        && leading
            .chars()
            .all(crate::layout_autoswitch::is_ascii_layout_letter_symbol)
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

use crate::ru_typo::repair_extra_letters_after_layout;
use crate::word_reader::{is_cyrillic_word, split_word_punctuation};

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
    let parts: Vec<&str> = ctx.core.split_whitespace().collect();
    if parts.len() != 2 {
        return None;
    }
    if !crate::phrase_lexicon::is_short_russian_function_word(&parts[0].to_lowercase()) {
        return None;
    }
    let replacement = rule(parts[1])?;
    (replacement != parts[1]).then(|| format!("{} {}", parts[0], replacement))
}

pub(super) fn apply_token_word_rule(ctx: &TypingRuleContext<'_>, rule: TextRule) -> Option<String> {
    if ctx.word.is_empty() {
        return None;
    }
    rule(ctx.word)
        .map(|replacement| format!("{}{}{}", ctx.token_leading, replacement, ctx.token_trailing))
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

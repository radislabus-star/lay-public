use crate::layout_autoswitch::{
    ascii_layout_prefix_can_be_letter, correct_confident_wrong_layout_ascii_word,
    correct_contextual_ascii_conjunction_i, correct_contextual_ascii_preposition_v,
    correct_duplicate_layout_prefix_on_ascii_token, correct_wrong_layout_ascii_phrase,
    correct_wrong_layout_ascii_technical_token, correct_wrong_layout_ascii_word,
    correct_wrong_layout_ascii_word_experimental, correct_wrong_layout_cyrillic_word,
    correct_wrong_layout_cyrillic_word_experimental,
};
use crate::phrase_reader::{correct_moved_prefix_letter_pair, correct_split_word_pair};
use crate::typing_replacements::replace_visual_b_words;

use super::types::TypingRuleContext;

#[path = "rules/helpers.rs"]
mod helpers;
use helpers::TextRule;
use helpers::{
    apply_core_then_word_rule, apply_last_physical_layout_token_rule,
    apply_last_trailing_layout_token_rule, apply_last_two_word_rule, apply_last_word_rule,
    apply_short_left_word_rule, apply_token_word_rule, cleanup_extra_letters_after_ru_layout,
    last_token_has_physical_layout_prefix, last_word_has_protected_ascii_context,
    layout_auto_allowed,
};

pub(super) fn apply_moved_prefix_pair(ctx: &TypingRuleContext<'_>) -> Option<String> {
    correct_moved_prefix_letter_pair(ctx.core)
        .or_else(|| apply_last_two_word_rule(ctx, correct_moved_prefix_letter_pair))
}

pub(super) fn apply_split_word_pair(ctx: &TypingRuleContext<'_>) -> Option<String> {
    correct_split_word_pair(ctx.core)
}

pub(super) fn apply_visual_b(ctx: &TypingRuleContext<'_>) -> Option<String> {
    if ctx.core.split_whitespace().any(|token| token == "b") {
        return None;
    }
    replace_visual_b_words(ctx.core, ctx.core)
}

pub(super) fn apply_duplicate_layout_prefix(ctx: &TypingRuleContext<'_>) -> Option<String> {
    apply_word_rule(ctx, correct_duplicate_layout_prefix_on_ascii_token)
}

pub(super) fn apply_mixed_script_layout(ctx: &TypingRuleContext<'_>) -> Option<String> {
    if !layout_auto_allowed(ctx) {
        return None;
    }
    crate::llm::repair_mixed_script(ctx.core)
        .or_else(|| apply_word_rule(ctx, crate::llm::repair_mixed_script))
}

pub(super) fn apply_layout_technical(ctx: &TypingRuleContext<'_>) -> Option<String> {
    apply_word_rule(ctx, correct_wrong_layout_ascii_technical_token)
}

pub(crate) fn apply_fast_layout_en_to_ru(ctx: &TypingRuleContext<'_>) -> Option<String> {
    if !layout_auto_allowed(ctx) {
        return None;
    }
    apply_core_then_word_rule(ctx, correct_confident_wrong_layout_ascii_word)
        .or_else(|| {
            apply_last_trailing_layout_token_rule(ctx, correct_confident_wrong_layout_ascii_word)
        })
        .or_else(|| {
            apply_last_physical_layout_token_rule(ctx, correct_confident_wrong_layout_ascii_word)
        })
        .map(|replacement| cleanup_extra_letters_after_ru_layout(&replacement))
}

pub(super) fn apply_contextual_ru_conjunction_i(ctx: &TypingRuleContext<'_>) -> Option<String> {
    if !layout_auto_allowed(ctx) {
        return None;
    }
    correct_contextual_ascii_conjunction_i(ctx.core)
}

pub(super) fn apply_contextual_ru_preposition_v(ctx: &TypingRuleContext<'_>) -> Option<String> {
    if !layout_auto_allowed(ctx) {
        return None;
    }
    correct_contextual_ascii_preposition_v(ctx.core)
}

pub(super) fn apply_layout_ru_to_en(ctx: &TypingRuleContext<'_>) -> Option<String> {
    if !layout_auto_allowed(ctx) {
        return None;
    }
    apply_core_then_word_rule(ctx, correct_wrong_layout_cyrillic_word)
}

pub(super) fn apply_layout_ru_to_en_experimental(ctx: &TypingRuleContext<'_>) -> Option<String> {
    if !layout_auto_allowed(ctx) {
        return None;
    }
    apply_core_then_word_rule(ctx, correct_wrong_layout_cyrillic_word_experimental)
}

pub(super) fn apply_layout_en_to_ru(ctx: &TypingRuleContext<'_>) -> Option<String> {
    if !layout_auto_allowed(ctx) {
        return None;
    }
    if let Some(replacement) = correct_wrong_layout_ascii_phrase(ctx.core)
        .or_else(|| correct_wrong_layout_ascii_word(ctx.core))
        .or_else(|| apply_last_two_word_rule(ctx, correct_wrong_layout_ascii_phrase))
        .or_else(|| apply_last_trailing_layout_token_rule(ctx, correct_wrong_layout_ascii_word))
    {
        Some(cleanup_extra_letters_after_ru_layout(&replacement))
    } else if let Some(replacement) =
        apply_last_physical_layout_token_rule(ctx, correct_wrong_layout_ascii_word)
    {
        Some(cleanup_extra_letters_after_ru_layout(&replacement))
    } else if last_token_has_physical_layout_prefix(ctx)
        || ascii_layout_prefix_can_be_letter(ctx.token_leading)
        || last_word_has_protected_ascii_context(ctx)
    {
        None
    } else {
        apply_word_rule(ctx, correct_wrong_layout_ascii_word)
            .map(|replacement| cleanup_extra_letters_after_ru_layout(&replacement))
    }
}

pub(super) fn apply_layout_en_to_ru_experimental(ctx: &TypingRuleContext<'_>) -> Option<String> {
    if !layout_auto_allowed(ctx) {
        return None;
    }
    if let Some(replacement) = correct_wrong_layout_ascii_phrase(ctx.core)
        .or_else(|| correct_wrong_layout_ascii_word_experimental(ctx.core))
        .or_else(|| apply_last_two_word_rule(ctx, correct_wrong_layout_ascii_phrase))
        .or_else(|| {
            apply_last_trailing_layout_token_rule(ctx, correct_wrong_layout_ascii_word_experimental)
        })
    {
        Some(cleanup_extra_letters_after_ru_layout(&replacement))
    } else if let Some(replacement) =
        apply_last_physical_layout_token_rule(ctx, correct_wrong_layout_ascii_word_experimental)
    {
        Some(cleanup_extra_letters_after_ru_layout(&replacement))
    } else if last_token_has_physical_layout_prefix(ctx)
        || ascii_layout_prefix_can_be_letter(ctx.token_leading)
        || last_word_has_protected_ascii_context(ctx)
    {
        None
    } else {
        apply_word_rule(ctx, correct_wrong_layout_ascii_word_experimental)
            .map(|replacement| cleanup_extra_letters_after_ru_layout(&replacement))
    }
}

include!("rules/typo.rs");

fn apply_word_rule(ctx: &TypingRuleContext<'_>, rule: TextRule) -> Option<String> {
    apply_token_word_rule(ctx, rule).or_else(|| apply_last_word_rule(ctx, rule))
}

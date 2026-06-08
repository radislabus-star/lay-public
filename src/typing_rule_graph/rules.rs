use crate::layout_autoswitch::{
    ascii_layout_prefix_can_be_letter, correct_confident_wrong_layout_ascii_word,
    correct_duplicate_layout_prefix_on_ascii_token, correct_wrong_layout_ascii_phrase,
    correct_wrong_layout_ascii_technical_token, correct_wrong_layout_ascii_word,
    correct_wrong_layout_ascii_word_experimental, correct_wrong_layout_cyrillic_word,
    correct_wrong_layout_cyrillic_word_experimental,
};
use crate::phrase_reader::{
    correct_contextual_glued_tail, correct_contextual_known_word_missing_letter,
    correct_glued_russian_phrase, correct_moved_prefix_letter_pair, correct_split_word_pair,
};
use crate::ru_typo::{
    correct_adjacent_transposition, correct_cyrillic_word_case, correct_extra_letters,
    correct_hard_sign_typo, correct_missing_letter, correct_repeated_letter,
    correct_single_letter_substitution, correct_verb_ending_confusion, correct_vowel_confusion,
};
use crate::typing_replacements::{replace_visual_b_words, replacement_for_token};

use super::types::TypingRuleContext;

type TextRule = fn(&str) -> Option<String>;

pub(super) fn apply_moved_prefix_pair(ctx: &TypingRuleContext<'_>) -> Option<String> {
    correct_moved_prefix_letter_pair(ctx.core)
}

pub(super) fn apply_split_word_pair(ctx: &TypingRuleContext<'_>) -> Option<String> {
    correct_split_word_pair(ctx.core)
}

pub(super) fn apply_visual_b(ctx: &TypingRuleContext<'_>) -> Option<String> {
    replace_visual_b_words(ctx.core, ctx.core)
}

pub(super) fn apply_personal_phrase(ctx: &TypingRuleContext<'_>) -> Option<String> {
    replacement_for_token(ctx.core)
}

pub(super) fn apply_personal_token(ctx: &TypingRuleContext<'_>) -> Option<String> {
    apply_word_rule(ctx, replacement_for_token)
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
    {
        Some(replacement)
    } else if ascii_layout_prefix_can_be_letter(ctx.token_leading) {
        None
    } else {
        apply_word_rule(ctx, correct_wrong_layout_ascii_word)
    }
}

pub(super) fn apply_layout_en_to_ru_experimental(ctx: &TypingRuleContext<'_>) -> Option<String> {
    if !layout_auto_allowed(ctx) {
        return None;
    }
    if let Some(replacement) = correct_wrong_layout_ascii_phrase(ctx.core)
        .or_else(|| correct_wrong_layout_ascii_word_experimental(ctx.core))
    {
        Some(replacement)
    } else if ascii_layout_prefix_can_be_letter(ctx.token_leading) {
        None
    } else {
        apply_word_rule(ctx, correct_wrong_layout_ascii_word_experimental)
    }
}

pub(super) fn apply_cyrillic_case(ctx: &TypingRuleContext<'_>) -> Option<String> {
    apply_word_rule(ctx, correct_cyrillic_word_case)
}

pub(super) fn apply_hard_sign(ctx: &TypingRuleContext<'_>) -> Option<String> {
    apply_word_rule(ctx, correct_hard_sign_typo)
}

pub(super) fn apply_adjacent_transposition(ctx: &TypingRuleContext<'_>) -> Option<String> {
    apply_word_rule(ctx, correct_adjacent_transposition)
}

pub(super) fn apply_repeated_letter(ctx: &TypingRuleContext<'_>) -> Option<String> {
    apply_word_rule(ctx, correct_repeated_letter)
}

pub(super) fn apply_single_letter_substitution(ctx: &TypingRuleContext<'_>) -> Option<String> {
    apply_word_rule(ctx, correct_single_letter_substitution)
}

pub(super) fn apply_verb_ending(ctx: &TypingRuleContext<'_>) -> Option<String> {
    apply_word_rule(ctx, correct_verb_ending_confusion)
}

pub(super) fn apply_vowel_confusion(ctx: &TypingRuleContext<'_>) -> Option<String> {
    apply_word_rule(ctx, correct_vowel_confusion)
}

pub(super) fn apply_extra_letters(ctx: &TypingRuleContext<'_>) -> Option<String> {
    apply_word_rule(ctx, correct_extra_letters)
}

pub(super) fn apply_missing_letter(ctx: &TypingRuleContext<'_>) -> Option<String> {
    correct_contextual_known_word_missing_letter(ctx.core)
        .or_else(|| apply_word_rule(ctx, correct_missing_letter))
}

pub(super) fn apply_glued_phrase(ctx: &TypingRuleContext<'_>) -> Option<String> {
    correct_contextual_glued_tail(ctx.core)
        .or_else(|| apply_word_rule(ctx, correct_glued_russian_phrase))
}

fn layout_auto_allowed(ctx: &TypingRuleContext<'_>) -> bool {
    ctx.allow_layout_auto
}

fn apply_core_then_word_rule(ctx: &TypingRuleContext<'_>, rule: TextRule) -> Option<String> {
    rule(ctx.core).or_else(|| apply_word_rule(ctx, rule))
}

fn apply_word_rule(ctx: &TypingRuleContext<'_>, rule: TextRule) -> Option<String> {
    if ctx.word.is_empty() {
        return None;
    }
    rule(ctx.word)
        .map(|replacement| format!("{}{}{}", ctx.token_leading, replacement, ctx.token_trailing))
}

use crate::phrase_reader::{
    correct_contextual_fuzzy_pair, correct_contextual_glued_tail,
    correct_contextual_known_word_missing_letter, correct_glued_russian_phrase,
};
use crate::ru_typo::{
    correct_adjacent_transposition, correct_contextual_past_tense_vowel_confusion,
    correct_cyrillic_word_case, correct_extra_letters, correct_hard_sign_typo,
    correct_missing_letter, correct_repeated_letter, correct_single_letter_substitution,
    correct_verb_ending_confusion, correct_vowel_confusion,
};

pub(super) fn apply_cyrillic_case(ctx: &TypingRuleContext<'_>) -> Option<String> {
    apply_word_rule(ctx, correct_cyrillic_word_case)
}

pub(super) fn apply_hard_sign(ctx: &TypingRuleContext<'_>) -> Option<String> {
    apply_word_rule(ctx, correct_hard_sign_typo)
}

pub(super) fn apply_adjacent_transposition(ctx: &TypingRuleContext<'_>) -> Option<String> {
    apply_short_left_word_rule(ctx, correct_adjacent_transposition)
        .or_else(|| apply_word_rule(ctx, correct_adjacent_transposition))
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
    apply_short_left_word_rule(ctx, correct_contextual_past_tense_vowel_confusion)
        .or_else(|| apply_word_rule(ctx, correct_vowel_confusion))
}

pub(super) fn apply_extra_letters(ctx: &TypingRuleContext<'_>) -> Option<String> {
    apply_word_rule(ctx, correct_extra_letters)
}

pub(super) fn apply_missing_letter(ctx: &TypingRuleContext<'_>) -> Option<String> {
    correct_contextual_known_word_missing_letter(ctx.core)
        .or_else(|| correct_contextual_fuzzy_pair(ctx.core))
        .or_else(|| apply_short_left_word_rule(ctx, correct_missing_letter))
        .or_else(|| apply_word_rule(ctx, correct_missing_letter))
}

pub(super) fn apply_glued_phrase(ctx: &TypingRuleContext<'_>) -> Option<String> {
    correct_contextual_glued_tail(ctx.core)
        .or_else(|| apply_word_rule(ctx, correct_glued_russian_phrase))
}

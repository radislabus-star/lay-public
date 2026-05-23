//! Registry of typing-assist rules.
//!
//! The pipeline owns ordering and arbitration. Individual rule modules own
//! language logic. This registry is the only place that maps a stable rule id to
//! its family and execution function.

use crate::layout_autoswitch::{
    ascii_layout_prefix_can_be_letter, correct_duplicate_layout_prefix_on_ascii_token,
    correct_wrong_layout_ascii_phrase, correct_wrong_layout_ascii_technical_token,
    correct_wrong_layout_ascii_word, correct_wrong_layout_cyrillic_word,
};
use crate::phrase_reader::{
    correct_contextual_glued_tail, correct_glued_russian_phrase, correct_moved_prefix_letter_pair,
    correct_split_word_pair,
};
use crate::ru_typo::{
    correct_adjacent_transposition, correct_cyrillic_word_case, correct_extra_letters,
    correct_hard_sign_typo, correct_missing_letter, correct_repeated_letter,
    correct_single_letter_substitution, correct_verb_ending_confusion, correct_vowel_confusion,
};
use crate::typing_candidate::TypingCandidateFamily;
use crate::typing_replacements::{replace_visual_b_words, replacement_for_token};

#[derive(Debug, Clone, Copy)]
pub(crate) struct TypingRuleContext<'a> {
    pub core: &'a str,
    pub word: &'a str,
    pub token_leading: &'a str,
    pub token_trailing: &'a str,
    pub allow_layout_auto: bool,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct TypingRuleDefinition {
    pub id: &'static str,
    pub family: TypingCandidateFamily,
    pub apply: for<'a> fn(&TypingRuleContext<'a>) -> Option<String>,
}

pub(crate) fn typing_rule_definitions() -> &'static [TypingRuleDefinition] {
    RULES
}

pub(crate) fn find_typing_rule(id: &str) -> Option<&'static TypingRuleDefinition> {
    typing_rule_definitions().iter().find(|rule| rule.id == id)
}

static RULES: &[TypingRuleDefinition] = &[
    TypingRuleDefinition {
        id: "moved_prefix_pair",
        family: TypingCandidateFamily::Structural,
        apply: apply_moved_prefix_pair,
    },
    TypingRuleDefinition {
        id: "split_word_pair",
        family: TypingCandidateFamily::Structural,
        apply: apply_split_word_pair,
    },
    TypingRuleDefinition {
        id: "visual_b",
        family: TypingCandidateFamily::Visual,
        apply: apply_visual_b,
    },
    TypingRuleDefinition {
        id: "personal_phrase",
        family: TypingCandidateFamily::Exact,
        apply: apply_personal_phrase,
    },
    TypingRuleDefinition {
        id: "personal_token",
        family: TypingCandidateFamily::Exact,
        apply: apply_personal_token,
    },
    TypingRuleDefinition {
        id: "duplicate_layout_prefix",
        family: TypingCandidateFamily::Layout,
        apply: apply_duplicate_layout_prefix,
    },
    TypingRuleDefinition {
        id: "mixed_script_layout",
        family: TypingCandidateFamily::Layout,
        apply: apply_mixed_script_layout,
    },
    TypingRuleDefinition {
        id: "layout_technical",
        family: TypingCandidateFamily::Layout,
        apply: apply_layout_technical,
    },
    TypingRuleDefinition {
        id: "layout_ru_to_en",
        family: TypingCandidateFamily::Layout,
        apply: apply_layout_ru_to_en,
    },
    TypingRuleDefinition {
        id: "layout_en_to_ru",
        family: TypingCandidateFamily::Layout,
        apply: apply_layout_en_to_ru,
    },
    TypingRuleDefinition {
        id: "contextual_layout_en_to_ru",
        family: TypingCandidateFamily::Layout,
        apply: apply_layout_en_to_ru,
    },
    TypingRuleDefinition {
        id: "cyrillic_case",
        family: TypingCandidateFamily::Typo,
        apply: apply_cyrillic_case,
    },
    TypingRuleDefinition {
        id: "hard_sign",
        family: TypingCandidateFamily::Typo,
        apply: apply_hard_sign,
    },
    TypingRuleDefinition {
        id: "adjacent_transposition",
        family: TypingCandidateFamily::Typo,
        apply: apply_adjacent_transposition,
    },
    TypingRuleDefinition {
        id: "repeated_letter",
        family: TypingCandidateFamily::Typo,
        apply: apply_repeated_letter,
    },
    TypingRuleDefinition {
        id: "single_letter_substitution",
        family: TypingCandidateFamily::Typo,
        apply: apply_single_letter_substitution,
    },
    TypingRuleDefinition {
        id: "verb_ending",
        family: TypingCandidateFamily::Typo,
        apply: apply_verb_ending,
    },
    TypingRuleDefinition {
        id: "vowel_confusion",
        family: TypingCandidateFamily::Typo,
        apply: apply_vowel_confusion,
    },
    TypingRuleDefinition {
        id: "extra_letters",
        family: TypingCandidateFamily::Typo,
        apply: apply_extra_letters,
    },
    TypingRuleDefinition {
        id: "missing_letter",
        family: TypingCandidateFamily::Typo,
        apply: apply_missing_letter,
    },
    TypingRuleDefinition {
        id: "glued_phrase",
        family: TypingCandidateFamily::Structural,
        apply: apply_glued_phrase,
    },
];

fn apply_moved_prefix_pair(ctx: &TypingRuleContext<'_>) -> Option<String> {
    correct_moved_prefix_letter_pair(ctx.core)
}

fn apply_split_word_pair(ctx: &TypingRuleContext<'_>) -> Option<String> {
    correct_split_word_pair(ctx.core)
}

fn apply_visual_b(ctx: &TypingRuleContext<'_>) -> Option<String> {
    replace_visual_b_words(ctx.core, ctx.core)
}

fn apply_personal_phrase(ctx: &TypingRuleContext<'_>) -> Option<String> {
    replacement_for_token(ctx.core)
}

fn apply_personal_token(ctx: &TypingRuleContext<'_>) -> Option<String> {
    word_rule(ctx, replacement_for_token)
}

fn apply_duplicate_layout_prefix(ctx: &TypingRuleContext<'_>) -> Option<String> {
    word_rule(ctx, correct_duplicate_layout_prefix_on_ascii_token)
}

fn apply_mixed_script_layout(ctx: &TypingRuleContext<'_>) -> Option<String> {
    if !ctx.allow_layout_auto {
        return None;
    }
    crate::llm::repair_mixed_script(ctx.core)
        .or_else(|| word_rule(ctx, crate::llm::repair_mixed_script))
}

fn apply_layout_technical(ctx: &TypingRuleContext<'_>) -> Option<String> {
    word_rule(ctx, correct_wrong_layout_ascii_technical_token)
}

fn apply_layout_ru_to_en(ctx: &TypingRuleContext<'_>) -> Option<String> {
    if !ctx.allow_layout_auto {
        return None;
    }
    correct_wrong_layout_cyrillic_word(ctx.core)
        .or_else(|| word_rule(ctx, correct_wrong_layout_cyrillic_word))
}

fn apply_layout_en_to_ru(ctx: &TypingRuleContext<'_>) -> Option<String> {
    if !ctx.allow_layout_auto {
        return None;
    }
    if let Some(replacement) = correct_wrong_layout_ascii_phrase(ctx.core)
        .or_else(|| correct_wrong_layout_ascii_word(ctx.core))
    {
        Some(replacement)
    } else if ascii_layout_prefix_can_be_letter(ctx.token_leading) {
        None
    } else {
        word_rule(ctx, correct_wrong_layout_ascii_word)
    }
}

fn apply_cyrillic_case(ctx: &TypingRuleContext<'_>) -> Option<String> {
    word_rule(ctx, correct_cyrillic_word_case)
}

fn apply_hard_sign(ctx: &TypingRuleContext<'_>) -> Option<String> {
    word_rule(ctx, correct_hard_sign_typo)
}

fn apply_adjacent_transposition(ctx: &TypingRuleContext<'_>) -> Option<String> {
    word_rule(ctx, correct_adjacent_transposition)
}

fn apply_repeated_letter(ctx: &TypingRuleContext<'_>) -> Option<String> {
    word_rule(ctx, correct_repeated_letter)
}

fn apply_single_letter_substitution(ctx: &TypingRuleContext<'_>) -> Option<String> {
    word_rule(ctx, correct_single_letter_substitution)
}

fn apply_verb_ending(ctx: &TypingRuleContext<'_>) -> Option<String> {
    word_rule(ctx, correct_verb_ending_confusion)
}

fn apply_vowel_confusion(ctx: &TypingRuleContext<'_>) -> Option<String> {
    word_rule(ctx, correct_vowel_confusion)
}

fn apply_extra_letters(ctx: &TypingRuleContext<'_>) -> Option<String> {
    word_rule(ctx, correct_extra_letters)
}

fn apply_missing_letter(ctx: &TypingRuleContext<'_>) -> Option<String> {
    word_rule(ctx, correct_missing_letter)
}

fn apply_glued_phrase(ctx: &TypingRuleContext<'_>) -> Option<String> {
    correct_contextual_glued_tail(ctx.core).or_else(|| word_rule(ctx, correct_glued_russian_phrase))
}

fn word_rule(ctx: &TypingRuleContext<'_>, f: fn(&str) -> Option<String>) -> Option<String> {
    if ctx.word.is_empty() {
        return None;
    }
    f(ctx.word)
        .map(|replacement| format!("{}{}{}", ctx.token_leading, replacement, ctx.token_trailing))
}

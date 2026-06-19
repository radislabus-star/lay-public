//! Layout autoswitch correction rules.
//!
//! This module decides whether a finished token is clearly a wrong-layout
//! token and returns deterministic replacements. It does not own daemon I/O,
//! typing-assist pipeline order, or smart-tail range planning.

mod ascii;
mod cyrillic;
mod english;
mod hyphen;
mod score;
mod technical;

use crate::keyboard::is_cyrillic_letter;
use crate::lexicon::is_common_ru_word;
use crate::ru_typo::{
    correct_extra_letters, correct_hard_sign_typo, correct_missing_letter, correct_repeated_letter,
};
use crate::russian_lexicon::{
    is_known_russian_adverb_o_form, is_known_russian_ka_oblique_form,
    is_known_russian_word_or_form, russian_dictionary, russian_short_dictionary,
    russian_tiny_dictionary,
};
use crate::russian_typo_candidates::generate_extra_letter_candidates;
use crate::text_case::apply_word_case;
use crate::word_reader::{is_cyrillic_word, split_word_punctuation};

pub(crate) use ascii::{
    ascii_layout_prefix_can_be_letter, correct_confident_wrong_layout_ascii_word,
    correct_wrong_layout_ascii_phrase, correct_wrong_layout_ascii_word,
    correct_wrong_layout_ascii_word_experimental, is_ascii_layout_letter_symbol,
    is_confident_wrong_layout_ascii_pair, is_protected_ascii_layout_token,
};
pub(crate) use cyrillic::{
    correct_wrong_layout_cyrillic_word, correct_wrong_layout_cyrillic_word_experimental,
};
pub(crate) use english::is_known_english_layout_autoswitch_word;
pub(crate) use hyphen::is_cyrillic_hyphenated_word_for_layout;
pub use technical::{
    correct_duplicate_layout_prefix_on_ascii_token, correct_wrong_layout_ascii_technical_token,
    should_keep_plain_cyrillic_before_ascii_technical,
};

pub(crate) fn warm_up() {
    crate::word_recognizer::warm_up();
}

fn polish_converted_russian_layout_token(token: &str) -> Option<String> {
    let (leading, word, trailing) = split_word_punctuation(token);
    if word.chars().count() < 5 || !is_cyrillic_word(word) {
        return None;
    }
    let lower = word.to_lowercase();

    if is_known_russian_layout_autoswitch_word(&lower) {
        return None;
    }
    if let Some(corrected) = correct_common_layout_extra_letter(word) {
        return Some(format!("{leading}{corrected}{trailing}"));
    }

    let corrected = correct_hard_sign_typo(word)
        .or_else(|| correct_repeated_letter(word))
        .or_else(|| correct_missing_letter(word))
        .or_else(|| correct_extra_letters(word))?;
    if !is_strong_layout_polish_word(&corrected.to_lowercase()) {
        return None;
    }
    Some(format!("{leading}{corrected}{trailing}"))
}

fn correct_common_layout_extra_letter(word: &str) -> Option<String> {
    if word.chars().count() < 5 || !is_cyrillic_word(word) {
        return None;
    }

    let lower = word.to_lowercase();

    let (candidate, _) = crate::candidate_ranker::choose_best_with_gap(
        generate_extra_letter_candidates(&lower),
        0.50,
        |candidate| {
            if candidate == &lower || !is_common_ru_word(candidate) {
                return None;
            }
            let mut score = crate::ngram::ru_candidate_margin(candidate, &lower);
            score += 3.0;
            Some(score)
        },
    )?;
    Some(apply_word_case(word, &candidate))
}

fn is_strong_layout_polish_word(word: &str) -> bool {
    is_common_ru_word(word)
        || russian_dictionary().contains(word)
        || russian_short_dictionary().contains(word)
        || russian_tiny_dictionary().contains(word)
}

pub(crate) fn is_known_russian_layout_autoswitch_word(word: &str) -> bool {
    let len = word.chars().filter(|ch| is_cyrillic_letter(*ch)).count();
    if len <= 3 {
        return russian_tiny_dictionary().contains(word);
    }

    is_known_russian_word_or_form(word)
        || is_known_russian_adverb_o_form(word)
        || is_known_russian_ka_oblique_form(word)
        || russian_short_dictionary().contains(word)
}

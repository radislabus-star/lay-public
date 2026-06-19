use crate::dict::{convert, Direction};
use crate::layout_autoswitch::{
    ascii_layout_prefix_can_be_letter, correct_wrong_layout_ascii_word,
    is_ascii_layout_letter_symbol,
};
use crate::lexicon::{is_common_ru_word, is_ru_short_function_word};
use crate::word_reader::split_word_punctuation;
use crate::word_recognizer::recognize_token;

pub(super) fn strong_ascii_to_ru_layout_candidate(token: &str) -> bool {
    if strong_shifted_ascii_to_ru_layout_candidate(token) {
        return true;
    }

    let identity = recognize_token(token);
    if !identity.is_unprotected_plain_ascii_word() {
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

pub(super) fn contextual_ascii_to_ru_layout_candidate(token: &str) -> bool {
    let identity = recognize_token(token);
    if !identity.is_unprotected_plain_ascii_word() {
        return false;
    }

    let Some(converted_lower) = converted_layout_word_lower(token) else {
        return false;
    };
    if converted_lower.is_empty() {
        return false;
    }

    !identity.known_en
        || has_leading_layout_punctuation_signal(token)
        || is_ru_short_function_word(&converted_lower)
        || is_common_ru_word(&converted_lower)
}

pub(super) fn ascii_letter_count(token: &str) -> usize {
    token.chars().filter(|ch| ch.is_ascii_alphabetic()).count()
}

pub(super) fn clean_ascii_to_ru_layout_candidate(token: &str) -> bool {
    if !token.chars().all(|ch| ch.is_ascii_alphabetic()) {
        return false;
    }
    let identity = recognize_token(token);
    identity.is_unprotected_plain_ascii_word() && correct_wrong_layout_ascii_word(token).is_some()
}

pub(super) fn has_layout_punctuation_signal(token: &str) -> bool {
    let (leading, _, trailing) = split_word_punctuation(token);
    ascii_layout_prefix_can_be_letter(leading)
        || ascii_layout_prefix_can_be_letter(trailing)
        || token.chars().any(is_ascii_layout_letter_symbol)
}

pub(super) fn has_leading_layout_punctuation_signal(token: &str) -> bool {
    let (leading, _, _) = split_word_punctuation(token);
    ascii_layout_prefix_can_be_letter(leading)
}

pub(super) fn has_trailing_layout_punctuation_signal(token: &str) -> bool {
    let (_, _, trailing) = split_word_punctuation(token);
    !trailing.is_empty()
}

fn strong_shifted_ascii_to_ru_layout_candidate(token: &str) -> bool {
    if !token.is_ascii()
        || token.chars().any(|ch| ch.is_ascii_digit())
        || !token.chars().any(is_ascii_upper_shift_layout_symbol)
    {
        return false;
    }

    let letters = token.bytes().filter(|byte| byte.is_ascii_alphabetic());
    letters.clone().count() >= 4
        && letters.into_iter().all(|byte| byte.is_ascii_uppercase())
        && correct_wrong_layout_ascii_word(token).is_some()
}

fn is_ascii_upper_shift_layout_symbol(ch: char) -> bool {
    matches!(ch, '{' | '}' | ':' | '"' | '<' | '>' | '~')
}

fn converted_layout_word_lower(token: &str) -> Option<String> {
    let (_, word, _) = split_word_punctuation(token);
    if word.is_empty() || !word.chars().all(|ch| ch.is_ascii_alphabetic()) {
        return None;
    }
    Some(convert(word, Direction::Us2Ru).to_lowercase())
}

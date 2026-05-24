use crate::layout_autoswitch::{
    ascii_layout_prefix_can_be_letter, correct_wrong_layout_ascii_word,
    is_ascii_layout_letter_symbol, is_confident_wrong_layout_ascii_pair,
};
use crate::lexicon::is_common_ru_word;
use crate::word_reader::{split_edge_whitespace, split_word_punctuation, split_ws_segments};
use crate::word_recognizer::{recognize_token, WordKind, WordScript};

use super::tokens::{
    has_recent_russian_context_before_last, is_embedded_ascii_term_context_token,
    is_natural_english_context_token, is_russian_context_token,
};

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
    let phrase_layout = rest.last().is_some_and(|previous| {
        is_confident_wrong_layout_ascii_pair(previous, last)
            && !has_trailing_layout_punctuation_signal(previous)
            && !has_trailing_layout_punctuation_signal(last)
    });
    if !(last_strong_layout || last_clean_layout && previous_clean_layout || phrase_layout) {
        return false;
    }

    let layout_punctuation = has_layout_punctuation_signal(last);
    let standalone_strong_layout =
        rest.is_empty() && last_strong_layout && !has_trailing_layout_punctuation_signal(last);
    let previous_allows_contextual_layout = rest.last().is_some_and(|previous| {
        is_russian_context_token(previous)
            || (last_clean_layout
                && clean_ascii_to_ru_layout_candidate(previous)
                && !layout_punctuation)
            || phrase_layout
            || (last_strong_layout
                && is_embedded_ascii_term_context_token(previous)
                && has_recent_russian_context_before_last(rest))
            || (layout_punctuation && is_natural_english_context_token(previous))
    });
    let standalone_layout_punctuation =
        rest.is_empty() && has_leading_layout_punctuation_signal(last);

    previous_allows_contextual_layout || standalone_layout_punctuation || standalone_strong_layout
}

fn strong_ascii_to_ru_layout_candidate(token: &str) -> bool {
    if strong_shifted_ascii_to_ru_layout_candidate(token) {
        return true;
    }

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

fn has_leading_layout_punctuation_signal(token: &str) -> bool {
    let (leading, _, _) = split_word_punctuation(token);
    ascii_layout_prefix_can_be_letter(leading)
}

fn has_trailing_layout_punctuation_signal(token: &str) -> bool {
    let (_, _, trailing) = split_word_punctuation(token);
    !trailing.is_empty()
}

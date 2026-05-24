#[path = "layout_signal/candidate.rs"]
mod candidate;

use crate::layout_autoswitch::is_confident_wrong_layout_ascii_pair;
use crate::word_reader::{split_edge_whitespace, split_ws_segments};

use candidate::{
    ascii_letter_count, clean_ascii_to_ru_layout_candidate,
    contextual_ascii_to_ru_layout_candidate, has_layout_punctuation_signal,
    has_leading_layout_punctuation_signal, has_trailing_layout_punctuation_signal,
    strong_ascii_to_ru_layout_candidate,
};

use super::tokens::{
    has_recent_russian_context_before_last, is_ascii_technical_context_token,
    is_embedded_ascii_term_context_token, is_natural_english_context_token,
    is_russian_context_token,
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
    let last_contextual_layout = contextual_ascii_to_ru_layout_candidate(last);
    let previous_clean_layout = rest
        .last()
        .is_some_and(|previous| clean_ascii_to_ru_layout_candidate(previous));
    let phrase_layout = rest.last().is_some_and(|previous| {
        is_confident_wrong_layout_ascii_pair(previous, last)
            && !has_trailing_layout_punctuation_signal(previous)
            && !has_trailing_layout_punctuation_signal(last)
    });
    if !(last_strong_layout
        || last_clean_layout && previous_clean_layout
        || phrase_layout
        || last_contextual_layout)
    {
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
            || (last_strong_layout
                && !layout_punctuation
                && rest.len() == 1
                && is_ascii_technical_context_token(previous))
            || (layout_punctuation && is_natural_english_context_token(previous))
            || (last_contextual_layout
                && !has_trailing_layout_punctuation_signal(last)
                && ascii_context_can_host_layout_tail(previous)
                && (has_recent_russian_context_before_last(rest)
                    || is_natural_english_context_token(previous)
                        && last_strong_layout
                        && ascii_letter_count(last) >= 4
                        && rest.len() == 1
                    || is_natural_english_context_token(previous)
                        && last_contextual_layout
                        && ascii_letter_count(last) <= 3
                        && rest.len() == 1
                    || rest.len() == 1
                        && is_ascii_technical_context_token(previous)
                        && previous.chars().all(|ch| ch.is_ascii_alphanumeric())
                        && ascii_letter_count(previous) >= 4))
    });
    let standalone_layout_punctuation =
        rest.is_empty() && has_leading_layout_punctuation_signal(last);

    previous_allows_contextual_layout || standalone_layout_punctuation || standalone_strong_layout
}

fn ascii_context_can_host_layout_tail(token: &str) -> bool {
    is_natural_english_context_token(token)
        || is_ascii_technical_context_token(token)
        || is_embedded_ascii_term_context_token(token)
}

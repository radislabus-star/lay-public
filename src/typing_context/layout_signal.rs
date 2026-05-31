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

    let signal = LayoutSignal::from_tokens(rest, last);
    if !signal.has_candidate_signal() {
        return false;
    }

    let previous_allows_contextual_layout = rest
        .last()
        .is_some_and(|previous| previous_token_allows_layout_tail(rest, previous, last, &signal));
    let standalone_layout_punctuation =
        rest.is_empty() && has_leading_layout_punctuation_signal(last);

    previous_allows_contextual_layout
        || standalone_layout_punctuation
        || signal.standalone_strong_layout(rest)
}

fn ascii_context_can_host_layout_tail(token: &str) -> bool {
    is_natural_english_context_token(token)
        || is_ascii_technical_context_token(token)
        || is_embedded_ascii_term_context_token(token)
}

#[derive(Debug, Clone, Copy)]
struct LayoutSignal {
    last_strong_layout: bool,
    last_clean_layout: bool,
    last_contextual_layout: bool,
    previous_clean_layout: bool,
    phrase_layout: bool,
    layout_punctuation: bool,
    trailing_layout_punctuation: bool,
}

impl LayoutSignal {
    fn from_tokens(rest: &[&str], last: &str) -> Self {
        let last_strong_layout = strong_ascii_to_ru_layout_candidate(last);
        let last_clean_layout = clean_ascii_to_ru_layout_candidate(last);
        let last_contextual_layout = contextual_ascii_to_ru_layout_candidate(last);
        let previous_clean_layout = rest
            .last()
            .is_some_and(|previous| clean_ascii_to_ru_layout_candidate(previous));
        let trailing_layout_punctuation = has_trailing_layout_punctuation_signal(last);
        let phrase_layout = rest.last().is_some_and(|previous| {
            is_confident_wrong_layout_ascii_pair(previous, last)
                && !has_trailing_layout_punctuation_signal(previous)
                && !trailing_layout_punctuation
        });

        Self {
            last_strong_layout,
            last_clean_layout,
            last_contextual_layout,
            previous_clean_layout,
            phrase_layout,
            layout_punctuation: has_layout_punctuation_signal(last),
            trailing_layout_punctuation,
        }
    }

    fn has_candidate_signal(self) -> bool {
        self.last_strong_layout
            || self.last_clean_layout && self.previous_clean_layout
            || self.phrase_layout
            || self.last_contextual_layout
    }

    fn standalone_strong_layout(self, rest: &[&str]) -> bool {
        rest.is_empty() && self.last_strong_layout && !self.trailing_layout_punctuation
    }
}

fn previous_token_allows_layout_tail(
    rest: &[&str],
    previous: &str,
    last: &str,
    signal: &LayoutSignal,
) -> bool {
    is_russian_context_token(previous)
        || (signal.last_clean_layout
            && clean_ascii_to_ru_layout_candidate(previous)
            && !signal.layout_punctuation)
        || signal.phrase_layout
        || (signal.last_strong_layout
            && is_embedded_ascii_term_context_token(previous)
            && has_recent_russian_context_before_last(rest))
        || (signal.last_strong_layout
            && !signal.layout_punctuation
            && rest.len() == 1
            && is_ascii_technical_context_token(previous))
        || (signal.layout_punctuation && is_natural_english_context_token(previous))
        || contextual_tail_allowed_by_previous_token(rest, previous, last, signal)
}

fn contextual_tail_allowed_by_previous_token(
    rest: &[&str],
    previous: &str,
    last: &str,
    signal: &LayoutSignal,
) -> bool {
    signal.last_contextual_layout
        && !signal.trailing_layout_punctuation
        && ascii_context_can_host_layout_tail(previous)
        && (has_recent_russian_context_before_last(rest)
            || is_natural_english_context_token(previous)
                && signal.last_strong_layout
                && ascii_letter_count(last) >= 4
                && rest.len() == 1
            || is_natural_english_context_token(previous)
                && signal.last_contextual_layout
                && ascii_letter_count(last) <= 3
                && rest.len() == 1
            || rest.len() == 1
                && is_ascii_technical_context_token(previous)
                && previous.chars().all(|ch| ch.is_ascii_alphanumeric())
                && ascii_letter_count(previous) >= 4)
}

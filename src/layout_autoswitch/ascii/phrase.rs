use crate::word_reader::{split_word_punctuation, split_ws_segments};

use super::super::english::is_known_english_layout_autoswitch_word;
use super::candidate::ascii_to_russian_layout_candidate;

pub(crate) fn correct_wrong_layout_ascii_phrase(text: &str) -> Option<String> {
    let segments = split_ws_segments(text);
    let word_count = segments.iter().filter(|(_, is_ws)| !*is_ws).count();
    if word_count < 2 {
        return None;
    }

    let mut converted_words = 0usize;
    let mut known_converted_words = 0usize;
    let mut clean_alpha_words = 0usize;
    let mut single_letter_words = 0usize;
    let mut multi_letter_words = 0usize;
    let mut known_english_context_words = 0usize;
    let mut has_shift_letter_signal = false;
    let mut out = String::with_capacity(text.len());
    for (segment, is_ws) in segments {
        if is_ws {
            out.push_str(segment);
            continue;
        }
        match ascii_alpha_count(segment) {
            1 => single_letter_words += 1,
            len if len > 1 => multi_letter_words += 1,
            _ => {}
        }
        let candidate = ascii_to_russian_layout_candidate(segment, true)?;
        converted_words += 1;
        if candidate.known {
            known_converted_words += 1;
        }
        if candidate.clean_alpha {
            clean_alpha_words += 1;
        }
        if candidate.clean_alpha
            && !candidate.shift_letter_signal
            && ascii_segment_is_known_english_word(segment)
        {
            known_english_context_words += 1;
        }
        has_shift_letter_signal |= candidate.shift_letter_signal;
        out.push_str(&candidate.replacement);
    }

    if converted_words < 2 || out == text || (single_letter_words > 0 && multi_letter_words == 0) {
        return None;
    }
    if known_english_context_words > 0 && !has_shift_letter_signal {
        return None;
    }

    confident_wrong_layout_ascii_phrase(
        converted_words,
        known_converted_words,
        clean_alpha_words,
        has_shift_letter_signal,
    )
    .then_some(out)
}

fn ascii_alpha_count(token: &str) -> usize {
    let (_, word, _) = split_word_punctuation(token);
    word.chars().filter(|ch| ch.is_ascii_alphabetic()).count()
}

fn ascii_segment_is_known_english_word(token: &str) -> bool {
    let (_, word, _) = split_word_punctuation(token);
    !word.is_empty() && is_known_english_layout_autoswitch_word(&word.to_ascii_lowercase())
}

pub(crate) fn is_confident_wrong_layout_ascii_pair(first: &str, second: &str) -> bool {
    let Some(first_candidate) = ascii_to_russian_layout_candidate(first, true) else {
        return false;
    };
    let Some(second_candidate) = ascii_to_russian_layout_candidate(second, true) else {
        return false;
    };

    let known_converted_words =
        usize::from(first_candidate.known) + usize::from(second_candidate.known);
    let clean_alpha_words =
        usize::from(first_candidate.clean_alpha) + usize::from(second_candidate.clean_alpha);
    let has_shift_letter_signal =
        first_candidate.shift_letter_signal || second_candidate.shift_letter_signal;

    confident_wrong_layout_ascii_phrase(
        2,
        known_converted_words,
        clean_alpha_words,
        has_shift_letter_signal,
    )
}

fn confident_wrong_layout_ascii_phrase(
    converted_words: usize,
    known_converted_words: usize,
    clean_alpha_words: usize,
    has_shift_letter_signal: bool,
) -> bool {
    let all_clean_known =
        clean_alpha_words == converted_words && known_converted_words == converted_words;
    let shifted_physical_run = has_shift_letter_signal && known_converted_words > 0;
    all_clean_known || shifted_physical_run
}

#[cfg(test)]
mod tests {
    use super::correct_wrong_layout_ascii_phrase;

    #[test]
    fn known_english_context_word_blocks_whole_phrase_layout_flip() {
        assert_eq!(correct_wrong_layout_ascii_phrase("file ljgecnbv"), None);
    }
}

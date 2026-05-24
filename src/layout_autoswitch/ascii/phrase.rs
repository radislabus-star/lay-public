use crate::word_reader::split_ws_segments;

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
    let mut has_shift_letter_signal = false;
    let mut out = String::with_capacity(text.len());
    for (segment, is_ws) in segments {
        if is_ws {
            out.push_str(segment);
            continue;
        }
        let candidate = ascii_to_russian_layout_candidate(segment, true)?;
        converted_words += 1;
        if candidate.known {
            known_converted_words += 1;
        }
        if candidate.clean_alpha {
            clean_alpha_words += 1;
        }
        has_shift_letter_signal |= candidate.shift_letter_signal;
        out.push_str(&candidate.replacement);
    }

    if converted_words < 2 || out == text {
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

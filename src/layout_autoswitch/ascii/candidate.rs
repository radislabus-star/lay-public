use crate::text_case::apply_word_case;
use crate::word_reader::{is_cyrillic_word, split_word_punctuation};

use super::super::{
    is_known_russian_layout_autoswitch_word, polish_converted_russian_layout_token,
};
use super::symbols::{has_ascii_shift_letter_signal, is_blocked_ascii_layout_token};

#[derive(Debug, Clone)]
pub(super) struct AsciiToRussianLayoutCandidate {
    pub replacement: String,
    pub word: String,
    pub known: bool,
    pub clean_alpha: bool,
    pub shift_letter_signal: bool,
}

pub(super) fn ascii_to_russian_layout_candidate(
    token: &str,
    allow_shift_fallback: bool,
) -> Option<AsciiToRussianLayoutCandidate> {
    if is_blocked_ascii_layout_token(token) {
        return None;
    }

    let shift_letter_signal = has_ascii_shift_letter_signal(token);
    let shifted_layout_word = shift_letter_signal
        && token
            .chars()
            .all(|ch| ch.is_ascii_alphabetic() || matches!(ch, '<' | '>' | '{' | '}' | ':' | '"'));
    let (_, split_word, _) = split_word_punctuation(token);
    let original_word = if shifted_layout_word {
        token
    } else {
        split_word
    };
    if original_word.is_empty() {
        return None;
    }

    let converted = crate::dict::convert(token, crate::dict::Direction::Us2Ru);
    if converted == token {
        return None;
    }

    let (_, converted_word, _) = split_word_punctuation(&converted);
    if converted_word.is_empty() || !is_cyrillic_word(converted_word) {
        return None;
    }

    let converted_lower = converted_word.to_lowercase();
    let raw_projection_stable = is_known_russian_layout_autoswitch_word(&converted_lower)
        || crate::russian_lexicon::is_known_russian_word_or_form(&converted_lower)
        || crate::russian_lexicon::is_reference_backed_russian_form(&converted_lower)
        || crate::nanda_wave::l2::l2_surface_foundation_contains(&converted_lower)
        || l2_phase_covers_raw_projection(&converted_lower)
        || crate::hot_field::HotFieldSnapshot::current()
            .layout_projection_has_phase_authority(&converted_lower);
    if !(raw_projection_stable || allow_shift_fallback && shift_letter_signal) {
        if let Some(replacement) = polish_converted_russian_layout_token(&converted) {
            let (_, replacement_word, _) = split_word_punctuation(&replacement);
            let word = replacement_word.to_string();
            return Some(AsciiToRussianLayoutCandidate {
                replacement,
                word,
                known: true,
                clean_alpha: token.chars().all(|ch| ch.is_ascii_alphabetic()),
                shift_letter_signal,
            });
        }
    }
    if !(raw_projection_stable || allow_shift_fallback && shift_letter_signal) {
        return None;
    }

    let replacement = if raw_projection_stable {
        let normalized_word = apply_word_case(original_word, &converted_lower);
        let (converted_leading, _, converted_trailing) = split_word_punctuation(&converted);
        format!("{converted_leading}{normalized_word}{converted_trailing}")
    } else {
        converted
    };

    let replacement = polish_converted_russian_layout_token(&replacement).unwrap_or(replacement);
    let (_, replacement_word, _) = split_word_punctuation(&replacement);
    let word = replacement_word.to_string();
    let replacement_known =
        raw_projection_stable || is_known_russian_layout_autoswitch_word(&word.to_lowercase());

    Some(AsciiToRussianLayoutCandidate {
        replacement,
        word,
        known: replacement_known,
        clean_alpha: token.chars().all(|ch| ch.is_ascii_alphabetic()),
        shift_letter_signal,
    })
}

fn l2_phase_covers_raw_projection(word: &str) -> bool {
    let readout = crate::nanda_wave::l2::l2_surface_phase_readout(word);
    readout.l1_refs >= 12 && readout.residual_l1_refs == 0 && readout.coherence_milli() >= 920
}

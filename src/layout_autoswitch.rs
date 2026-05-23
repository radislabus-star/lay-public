//! Layout autoswitch and technical-token correction rules.
//!
//! This module decides whether a finished token is clearly a wrong-layout
//! token and returns deterministic replacements. It does not own daemon I/O,
//! typing-assist pipeline order, or smart-tail range planning.

use std::collections::HashSet;
use std::sync::OnceLock;

use crate::keyboard::is_cyrillic_letter;
use crate::lexicon::{
    extend_user_protected_ascii_words, is_common_en_technical_word, is_common_ru_word,
    is_ru_hyphen_particle, EN_HUNSPELL, EN_WORDS,
};
use crate::phrase_lexicon::is_common_short_russian_preposition;
use crate::ru_typo::{
    correct_extra_letters, correct_hard_sign_typo, correct_missing_letter, correct_repeated_letter,
    has_plausible_russian_typo_candidate,
};
use crate::russian_chars::is_russian_vowel;
use crate::russian_lexicon::{
    is_known_cyrillic_hyphen_part, is_known_russian_adverb_o_form,
    is_known_russian_ka_oblique_form, is_known_russian_word_or_form, russian_dictionary,
    russian_short_dictionary, russian_tiny_dictionary,
};
use crate::russian_typo_candidates::generate_extra_letter_candidates;
use crate::text_case::apply_word_case;
use crate::word_reader::{is_cyrillic_word, split_word_punctuation, split_ws_segments};
use crate::word_recognizer::{
    is_ascii_technical_token, is_cli_option_token, is_protected_ascii_token,
};

const LEM_LAYOUT_AUTOSWITCH_MARGIN: f64 = 0.25;

pub(crate) fn warm_up() {
    let _ = english_dictionary().len();
}

pub(crate) fn correct_wrong_layout_ascii_word(token: &str) -> Option<String> {
    if is_cli_option_token(token) {
        return None;
    }
    if !is_plain_ascii_layout_token(token) {
        return None;
    }

    let (_, original_word, _) = split_word_punctuation(token);
    if original_word.is_empty() {
        return None;
    }

    let converted = crate::dict::convert(token, crate::dict::Direction::Us2Ru);
    if converted == token {
        return None;
    }

    let polished = polish_converted_russian_layout_token(&converted);
    let converted_token = polished.as_deref().unwrap_or(&converted);
    let (converted_leading, converted_word, converted_trailing) =
        split_word_punctuation(converted_token);
    if converted_word.is_empty() || !is_cyrillic_word(converted_word) {
        return None;
    }

    let converted_lower = converted_word.to_lowercase();
    if !is_known_russian_layout_autoswitch_word(&converted_lower) {
        return None;
    }
    if is_protected_ascii_layout_token(token)
        && is_known_english_layout_autoswitch_word(&original_word.to_ascii_lowercase())
    {
        return None;
    }

    let normalized_word = apply_word_case(original_word, &converted_lower);
    let normalized = format!("{converted_leading}{normalized_word}{converted_trailing}");
    if is_protected_ascii_layout_token(token) {
        return lem_prefers_layout_candidate(original_word, &normalized_word).then_some(normalized);
    }
    match crate::llm::choose_token_hybrid(original_word, &normalized_word) {
        Ok(Some(choice)) if choice == normalized_word => Some(normalized),
        Ok(Some(choice)) if choice == original_word => {
            allow_short_layout_word(original_word, &converted_lower).then_some(normalized)
        }
        _ => Some(normalized),
    }
}

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
        let candidate = ascii_phrase_segment_candidate(segment)?;
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
    let Some(first_candidate) = ascii_phrase_segment_candidate(first) else {
        return false;
    };
    let Some(second_candidate) = ascii_phrase_segment_candidate(second) else {
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

#[derive(Debug, Clone)]
struct AsciiPhraseSegmentCandidate {
    replacement: String,
    known: bool,
    clean_alpha: bool,
    shift_letter_signal: bool,
}

fn ascii_phrase_segment_candidate(token: &str) -> Option<AsciiPhraseSegmentCandidate> {
    if is_cli_option_token(token) || !is_plain_ascii_layout_token(token) {
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
    let known = is_known_russian_layout_autoswitch_word(&converted_lower);
    let clean_alpha = token.chars().all(|ch| ch.is_ascii_alphabetic());
    let shift_letter_signal = has_ascii_shift_letter_signal(token);

    let replacement = if known {
        let (_, original_word, _) = split_word_punctuation(token);
        let normalized_word = apply_word_case(original_word, &converted_lower);
        let (converted_leading, _, converted_trailing) = split_word_punctuation(&converted);
        format!("{converted_leading}{normalized_word}{converted_trailing}")
    } else if shift_letter_signal {
        converted
    } else {
        return None;
    };
    let replacement = polish_converted_russian_layout_token(&replacement).unwrap_or(replacement);
    let (_, replacement_word, _) = split_word_punctuation(&replacement);
    let known = known || is_known_russian_layout_autoswitch_word(&replacement_word.to_lowercase());

    Some(AsciiPhraseSegmentCandidate {
        replacement,
        known,
        clean_alpha,
        shift_letter_signal,
    })
}

fn polish_converted_russian_layout_token(token: &str) -> Option<String> {
    let (leading, word, trailing) = split_word_punctuation(token);
    if word.chars().count() < 5 || !is_cyrillic_word(word) {
        return None;
    }
    let lower = word.to_lowercase();

    if let Some(corrected) = correct_common_layout_extra_letter(word) {
        return Some(format!("{leading}{corrected}{trailing}"));
    }
    if is_known_russian_layout_autoswitch_word(&lower) {
        return None;
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

pub(crate) fn ascii_layout_prefix_can_be_letter(prefix: &str) -> bool {
    prefix.chars().any(is_ascii_layout_letter_symbol)
}

pub(crate) fn is_ascii_layout_letter_symbol(ch: char) -> bool {
    matches!(
        ch,
        '\'' | ';' | '[' | ']' | '`' | ',' | '.' | '-' | '{' | '}' | ':' | '"' | '<' | '>' | '~'
    )
}

pub(crate) fn correct_wrong_layout_cyrillic_word(token: &str) -> Option<String> {
    if !is_plain_cyrillic_layout_token(token) {
        return None;
    }

    let (_, original_word, _) = split_word_punctuation(token);
    if original_word.is_empty() {
        return None;
    }

    let original_lower = original_word.to_lowercase();
    if is_known_russian_layout_autoswitch_word(&original_lower) {
        return None;
    }
    if has_plausible_russian_typo_candidate(&original_lower) {
        return None;
    }

    let converted = crate::dict::convert(token, crate::dict::Direction::Ru2Us);
    if converted == token {
        return None;
    }

    let (converted_leading, converted_word, converted_trailing) =
        split_word_punctuation(&converted);
    if converted_word.is_empty() || !is_plain_ascii_word_candidate(converted_word) {
        return None;
    }

    english_layout_autoswitch_candidates(converted_word)
        .into_iter()
        .find_map(|candidate_lower| {
            let candidate_word = apply_word_case(original_word, &candidate_lower);
            let candidate = format!("{converted_leading}{candidate_word}{converted_trailing}");
            lem_prefers_layout_candidate(original_word, &candidate_word).then_some(candidate)
        })
}

fn english_layout_autoswitch_candidates(converted: &str) -> Vec<String> {
    let lower = converted.to_ascii_lowercase();
    let mut out = Vec::new();
    if is_known_english_layout_autoswitch_word(&lower) {
        out.push(lower.clone());
    }

    let chars: Vec<char> = lower.chars().collect();
    if chars.len() >= 6 {
        let without_prefix: String = chars[1..].iter().collect();
        if !is_known_english_layout_autoswitch_word(&lower)
            && is_known_english_layout_autoswitch_word(&without_prefix)
        {
            out.push(without_prefix);
        }
    }

    out
}

fn lem_prefers_layout_candidate(typed: &str, candidate: &str) -> bool {
    let ranked = crate::lem::rank_candidates(typed, [typed.to_string(), candidate.to_string()]);
    let Some(best) = ranked.first() else {
        return false;
    };
    if best.text != candidate {
        return false;
    }

    let margin = ranked
        .get(1)
        .map(|second| best.total - second.total)
        .unwrap_or(f64::INFINITY);
    margin >= LEM_LAYOUT_AUTOSWITCH_MARGIN
}

fn is_plain_cyrillic_layout_token(token: &str) -> bool {
    token.chars().any(is_cyrillic_letter)
        && token.chars().all(|ch| {
            is_cyrillic_letter(ch)
                || matches!(
                    ch,
                    ',' | '.' | '!' | '?' | ':' | ';' | '$' | '%' | '&' | '#' | '@' | '-' | '_'
                )
        })
}

fn is_plain_ascii_layout_token(token: &str) -> bool {
    token.is_ascii()
        && token.chars().any(|ch| ch.is_ascii_alphabetic())
        && !token.chars().any(|ch| ch.is_ascii_digit())
        && token
            .chars()
            .all(|ch| ch.is_ascii_alphabetic() || is_ascii_layout_token_symbol(ch))
}

fn has_ascii_shift_letter_signal(token: &str) -> bool {
    token.chars().any(is_ascii_shift_letter_symbol)
}

pub(crate) fn is_ascii_shift_letter_symbol(ch: char) -> bool {
    matches!(ch, '{' | '}' | ':' | '"' | '<' | '>' | '~')
}

fn is_ascii_layout_token_symbol(ch: char) -> bool {
    is_ascii_layout_letter_symbol(ch)
        || matches!(
            ch,
            '/' | '?' | '!' | '$' | '%' | '^' | '&' | '#' | '@' | '_'
        )
}

pub(crate) fn is_protected_ascii_layout_token(token: &str) -> bool {
    is_protected_ascii_token(token)
}

fn allow_short_layout_word(original: &str, converted_lower: &str) -> bool {
    original
        .chars()
        .filter(|ch| ch.is_ascii_alphabetic())
        .count()
        <= 3
        && russian_tiny_dictionary().contains(converted_lower)
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

pub(crate) fn is_known_english_layout_autoswitch_word(word: &str) -> bool {
    let len = word.chars().filter(|ch| ch.is_ascii_alphabetic()).count();
    if len < 4 {
        return is_common_en_technical_word(word);
    }
    english_dictionary().contains(word)
}

fn is_plain_ascii_word_candidate(token: &str) -> bool {
    token.is_ascii()
        && token.chars().any(|ch| ch.is_ascii_alphabetic())
        && token
            .chars()
            .all(|ch| ch.is_ascii_alphabetic() || ch == '-')
}

pub fn correct_duplicate_layout_prefix_on_ascii_token(token: &str) -> Option<String> {
    let mut chars = token.chars();
    let first = chars.next()?;
    if !is_cyrillic_letter(first) {
        return None;
    }

    let rest: String = chars.collect();
    if !is_ascii_technical_token(&rest) {
        return None;
    }

    let mapped = crate::dict::convert(&first.to_string(), crate::dict::Direction::Ru2Us);
    let mut mapped_chars = mapped.chars();
    let mapped = mapped_chars.next()?;
    if mapped_chars.next().is_some() {
        return None;
    }

    let rest_first = rest.chars().next()?;
    if rest_first.is_ascii_alphabetic() && mapped.eq_ignore_ascii_case(&rest_first) {
        Some(rest)
    } else {
        None
    }
}

pub fn correct_wrong_layout_ascii_technical_token(token: &str) -> Option<String> {
    if !token.contains('-') {
        return None;
    }
    if !token.chars().any(is_cyrillic_letter) || token.chars().any(|ch| ch.is_ascii_alphabetic()) {
        return None;
    }
    if !token
        .chars()
        .all(|ch| is_cyrillic_letter(ch) || ch.is_ascii_digit() || matches!(ch, '-' | '_' | '.'))
    {
        return None;
    }

    let converted = crate::dict::convert(token, crate::dict::Direction::Ru2Us);
    if converted == token || !is_ascii_technical_token(&converted) {
        return None;
    }
    if !has_clear_ascii_technical_layout_signal(&converted) {
        return None;
    }

    let has_clear_separator = converted.contains('-');
    let has_short_ascii_segment = converted
        .split(['-', '_', '.'])
        .any(|part| (2..=4).contains(&part.chars().count()));
    let original_known_hyphen_word = token.contains('-')
        && (is_cyrillic_hyphenated_word_for_layout(token)
            || has_known_cyrillic_hyphen_fragment(token));

    if has_clear_separator && has_short_ascii_segment && !original_known_hyphen_word {
        Some(converted)
    } else {
        None
    }
}

fn has_clear_ascii_technical_layout_signal(token: &str) -> bool {
    let alpha_total = token.chars().filter(|ch| ch.is_ascii_alphabetic()).count();
    let alpha_segment = token
        .split(['-', '_', '.', '@', '/', '\\', ':', '+', '#'])
        .any(|part| part.chars().filter(|ch| ch.is_ascii_alphabetic()).count() >= 2);

    alpha_total >= 4 && alpha_segment
}

pub fn should_keep_plain_cyrillic_before_ascii_technical(original: &str, converted: &str) -> bool {
    original.chars().count() >= 4
        && original.chars().all(is_cyrillic_letter)
        && converted != original
        && is_ascii_technical_token(converted)
}

fn is_known_cyrillic_hyphenated_word(word: &str) -> bool {
    if !is_cyrillic_word(word) {
        return false;
    }
    let dict = russian_short_dictionary();
    word.split('-')
        .all(|part| part.chars().count() >= 3 && is_known_cyrillic_hyphen_part(part, dict))
}

fn has_known_cyrillic_hyphen_fragment(word: &str) -> bool {
    if !word.contains('-') || !is_cyrillic_word(word) {
        return false;
    }

    word.split('-').any(|part| {
        let lower = part.to_lowercase();
        lower.chars().count() >= 3
            && is_known_cyrillic_hyphen_part(&lower, russian_short_dictionary())
    })
}

pub(crate) fn is_cyrillic_hyphenated_word_for_layout(word: &str) -> bool {
    is_known_cyrillic_hyphenated_word(word) || is_plausible_cyrillic_hyphenated_word(word)
}

fn is_plausible_cyrillic_hyphenated_word(word: &str) -> bool {
    if !word.contains('-') || !is_cyrillic_word(word) {
        return false;
    }
    let parts: Vec<&str> = word.split('-').collect();
    if parts.len() < 2 || parts.iter().any(|part| part.is_empty()) {
        return false;
    }

    let mut strong_parts = 0usize;
    for (idx, part) in parts.iter().enumerate() {
        let lower = part.to_lowercase();
        let len = lower.chars().count();
        if len < 2 || !lower.chars().any(is_russian_vowel) {
            return false;
        }
        if len >= 3
            || is_known_cyrillic_hyphen_part(&lower, russian_short_dictionary())
            || (idx == 0 && is_common_short_russian_preposition(&lower))
            || (idx > 0 && is_russian_hyphen_particle(&lower))
        {
            strong_parts += 1;
        }
    }
    strong_parts >= 2
}

fn is_russian_hyphen_particle(part: &str) -> bool {
    is_ru_hyphen_particle(part)
}

fn english_dictionary() -> &'static HashSet<String> {
    static WORDS: OnceLock<HashSet<String>> = OnceLock::new();
    WORDS.get_or_init(|| {
        let mut words = load_ascii_hunspell_words_min_len(EN_HUNSPELL, 4).unwrap_or_default();
        if let Ok(extra) = load_ascii_word_list_min_len(EN_WORDS, 4) {
            words.extend(extra);
        }
        extend_user_protected_ascii_words(&mut words, 1);
        words
    })
}

fn load_ascii_hunspell_words_min_len(
    path: &str,
    min_chars: usize,
) -> std::io::Result<HashSet<String>> {
    let text = std::fs::read_to_string(path)?;
    let mut words = HashSet::new();
    for line in text.lines().skip(1) {
        let word = line.split('/').next().unwrap_or("").trim().to_lowercase();
        if word.chars().count() >= min_chars && is_plain_ascii_word_candidate(&word) {
            words.insert(word);
        }
    }
    Ok(words)
}

fn load_ascii_word_list_min_len(path: &str, min_chars: usize) -> std::io::Result<HashSet<String>> {
    let text = std::fs::read_to_string(path)?;
    let mut words = HashSet::new();
    for line in text.lines() {
        let word = line.trim().to_lowercase();
        if word.chars().count() >= min_chars && is_plain_ascii_word_candidate(&word) {
            words.insert(word);
        }
    }
    Ok(words)
}

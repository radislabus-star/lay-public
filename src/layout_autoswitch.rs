//! Layout autoswitch and technical-token correction rules.
//!
//! This module decides whether a finished token is clearly a wrong-layout
//! token and returns deterministic replacements. It does not own daemon I/O,
//! typing-assist pipeline order, or smart-tail range planning.

use std::collections::HashSet;
use std::sync::OnceLock;

use crate::keyboard::is_cyrillic_letter;
use crate::lexicon::{is_common_en_technical_word, is_ru_hyphen_particle, EN_HUNSPELL, EN_WORDS};
use crate::phrase_lexicon::is_common_short_russian_preposition;
use crate::ru_typo::has_plausible_russian_typo_candidate;
use crate::russian_chars::is_russian_vowel;
use crate::russian_lexicon::{
    is_known_cyrillic_hyphen_part, is_known_russian_adverb_o_form,
    is_known_russian_ka_oblique_form, is_known_russian_word_or_form, russian_short_dictionary,
    russian_tiny_dictionary,
};
use crate::text_case::apply_word_case;
use crate::word_reader::{is_cyrillic_word, split_word_punctuation};
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

    let (converted_leading, converted_word, converted_trailing) =
        split_word_punctuation(&converted);
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

pub(crate) fn ascii_layout_prefix_can_be_letter(prefix: &str) -> bool {
    prefix
        .chars()
        .any(|ch| matches!(ch, '\'' | ';' | '[' | ']' | '`' | ',' | '.' | '-'))
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
        && token.chars().all(|ch| {
            ch.is_ascii_alphabetic()
                || matches!(
                    ch,
                    ',' | ';'
                        | '\''
                        | '['
                        | ']'
                        | '`'
                        | '.'
                        | '/'
                        | '?'
                        | '!'
                        | ':'
                        | '$'
                        | '%'
                        | '^'
                        | '&'
                        | '#'
                        | '@'
                        | '-'
                        | '_'
                )
        })
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

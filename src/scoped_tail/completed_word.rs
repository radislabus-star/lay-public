use crate::keyboard::{is_cyrillic_letter, map_original_events, KeyEvent};
use crate::layout_autoswitch::{
    correct_duplicate_layout_prefix_on_ascii_token, correct_wrong_layout_ascii_technical_token,
    correct_wrong_layout_ascii_word, is_known_english_layout_autoswitch_word,
    is_known_russian_layout_autoswitch_word, is_protected_ascii_layout_token,
    should_keep_plain_cyrillic_before_ascii_technical,
};
use crate::token_language::{is_known_en_token, is_known_ru_token};
use crate::word_reader::{is_cyrillic_word, split_word_punctuation};
use crate::word_recognizer::{
    is_ascii_technical_or_brand_token, is_ascii_technical_token, is_ascii_titlecase_token,
};

use super::word_flip::flip_word_events;

pub fn decide_completed_scope_word(word: &[KeyEvent]) -> String {
    let original = map_original_events(word);
    if is_single_cyrillic_completed_scope_word(&original) {
        return original;
    }
    if is_short_repeated_completed_scope_word(&original) {
        return original;
    }
    if let Some(repaired) = deterministic_completed_scope_repair(&original) {
        return repaired;
    }
    let converted = flip_word_events(word);
    if should_keep_plain_cyrillic_before_ascii_technical(&original, &converted) {
        return original;
    }
    let (_, converted_word, _) = split_word_punctuation(&converted);
    if converted != original
        && !stable_completed_scope_original(&original)
        && completed_scope_flip_target_is_known(&converted, converted_word)
    {
        return converted;
    }
    let decision = if crate::llm::model_backend_enabled() {
        crate::llm::choose_token_consensus(&original, &converted)
    } else {
        crate::llm::choose_token_hybrid(&original, &converted)
    };
    match decision {
        Ok(Some(text)) if !text.trim().is_empty() => text,
        Ok(_) | Err(_) => original,
    }
}

pub(super) fn short_completed_tail_layout_flip(word: &[KeyEvent]) -> Option<String> {
    let original = map_original_events(word);
    let (_, original_word, _) = split_word_punctuation(&original);
    let original_len = original_word.chars().count();
    if !is_cyrillic_word(original_word)
        || is_known_russian_layout_autoswitch_word(&original_word.to_lowercase())
    {
        return None;
    }

    let flipped = flip_word_events(word);
    let (_, flipped_word, _) = split_word_punctuation(&flipped);
    let flipped_len = flipped_word.chars().count();
    if !flipped_word.is_ascii() {
        return None;
    }

    if (2..=4).contains(&original_len)
        && (2..=4).contains(&flipped_len)
        && flipped_word.chars().all(|ch| ch.is_ascii_alphabetic())
    {
        return Some(flipped);
    }

    completed_scope_flip_target_is_known(flipped_word, flipped_word).then_some(flipped)
}

pub(super) fn stable_completed_scope_original(original: &str) -> bool {
    let (_, word, _) = split_word_punctuation(original);
    if word.is_empty() {
        return false;
    }

    let lower = word.to_lowercase();
    if is_cyrillic_word(word) {
        if strong_russian_completed_scope_original(&lower) {
            return true;
        }
        if cyrillic_to_known_ascii_layout_target(word) {
            return false;
        }
        return is_known_ru_token(&lower);
    }

    if word.is_ascii() {
        let ascii_lower = word.to_ascii_lowercase();
        return is_protected_ascii_layout_token(word)
            || is_ascii_technical_token(original)
            || is_known_en_token(&ascii_lower)
            || is_known_english_layout_autoswitch_word(&ascii_lower);
    }

    false
}

fn strong_russian_completed_scope_original(lower: &str) -> bool {
    crate::lexicon::is_common_ru_word(lower)
        || crate::lexicon::is_user_protected_word(lower)
        || crate::russian_lexicon::russian_tiny_dictionary().contains(lower)
        || crate::russian_lexicon::russian_short_dictionary().contains(lower)
}

fn cyrillic_to_known_ascii_layout_target(word: &str) -> bool {
    let converted = crate::dict::convert(word, crate::dict::Direction::Ru2Us);
    if converted == word {
        return false;
    }
    let (_, converted_word, _) = split_word_punctuation(&converted);
    completed_scope_flip_target_is_known(&converted, converted_word)
}

pub(super) fn is_short_repeated_completed_scope_word(original: &str) -> bool {
    let (_, word, _) = split_word_punctuation(original);
    let mut chars = word.chars();
    matches!(
        (chars.next(), chars.next(), chars.next()),
        (Some(first), Some(second), None)
            if first == second && (is_cyrillic_letter(first) || first.is_ascii_alphabetic())
    )
}

pub(super) fn deterministic_completed_scope_repair(original: &str) -> Option<String> {
    correct_duplicate_layout_prefix_on_ascii_token(original)
        .or_else(|| correct_wrong_layout_ascii_technical_token(original))
        .or_else(|| correct_wrong_layout_ascii_word(original))
}

pub(super) fn completed_scope_flip_target_is_known(flipped: &str, flipped_word: &str) -> bool {
    if flipped_word.is_empty() {
        return false;
    }

    let flipped_lower = flipped_word.to_lowercase();
    if is_cyrillic_word(flipped_word) {
        return is_known_russian_layout_autoswitch_word(&flipped_lower);
    }

    if flipped_word.is_ascii() {
        return is_known_english_layout_autoswitch_word(&flipped_word.to_ascii_lowercase())
            || is_known_en_token(&flipped_word.to_ascii_lowercase())
            || is_ascii_technical_token(flipped)
            || is_ascii_technical_or_brand_token(flipped_word)
            || is_ascii_titlecase_token(flipped_word);
    }

    false
}

fn is_single_cyrillic_completed_scope_word(word: &str) -> bool {
    let (_, core, _) = split_word_punctuation(word);
    let mut chars = core.chars();
    matches!(
        (chars.next(), chars.next()),
        (Some(ch), None) if is_cyrillic_letter(ch)
    )
}

#[cfg(test)]
mod tests {
    use super::{decide_completed_scope_word, stable_completed_scope_original};
    use crate::keyboard::{split_event_words, text_to_key_events};

    #[test]
    fn completed_scope_recovers_known_ascii_layout_target() {
        let events = text_to_key_events("цщкв", false).expect("events");
        let words = split_event_words(&events).expect("words");

        assert_eq!(decide_completed_scope_word(words[0]), "word");
    }

    #[test]
    fn layout_garbage_is_not_stable_completed_scope_original() {
        assert!(!stable_completed_scope_original("цщкв"));
        assert!(!stable_completed_scope_original("ашду"));
        assert!(!stable_completed_scope_original("пше"));
    }
}

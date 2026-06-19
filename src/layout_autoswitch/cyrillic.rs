//! Cyrillic-to-ASCII layout autoswitch.

use crate::keyboard::is_cyrillic_letter;
use crate::lexicon::is_common_en_technical_word;
use crate::ru_typo::has_plausible_russian_typo_candidate;
use crate::text_case::apply_word_case;
use crate::word_reader::split_word_punctuation;
use crate::word_recognizer::{recognize_token, WordScript};

use super::english::{is_known_english_layout_autoswitch_word, is_plain_ascii_word_candidate};
use super::is_known_russian_layout_autoswitch_word;
use super::score::lem_prefers_layout_candidate;

pub(crate) fn correct_wrong_layout_cyrillic_word(token: &str) -> Option<String> {
    correct_wrong_layout_cyrillic_word_with_policy(token, EnglishLayoutPolicy::Strict)
}

pub(crate) fn correct_wrong_layout_cyrillic_word_experimental(token: &str) -> Option<String> {
    if correct_wrong_layout_cyrillic_word(token).is_some() {
        return None;
    }
    correct_wrong_layout_cyrillic_word_with_policy(token, EnglishLayoutPolicy::Experimental)
}

#[derive(Clone, Copy)]
enum EnglishLayoutPolicy {
    Strict,
    Experimental,
}

fn correct_wrong_layout_cyrillic_word_with_policy(
    token: &str,
    policy: EnglishLayoutPolicy,
) -> Option<String> {
    if !is_plain_cyrillic_layout_token(token) {
        return None;
    }

    let (_, original_word, _) = split_word_punctuation(token);
    if original_word.is_empty() {
        return None;
    }

    let converted = crate::dict::convert(token, crate::dict::Direction::Ru2Us);
    if converted == token {
        return None;
    }

    let (converted_leading, converted_word, converted_trailing) =
        split_word_punctuation(&converted);
    if !converted_leading.is_empty() {
        return None;
    }
    if converted_word.is_empty() || !is_plain_ascii_word_candidate(converted_word) {
        return None;
    }

    let converted_lower = converted_word.to_ascii_lowercase();
    let converted_is_technical = is_common_en_technical_word(&converted_lower);
    let original_lower = original_word.to_lowercase();
    if !converted_is_technical && is_known_russian_layout_autoswitch_word(&original_lower) {
        return None;
    }
    if !converted_is_technical && has_plausible_russian_typo_candidate(&original_lower) {
        return None;
    }

    english_layout_autoswitch_candidates(converted_word, policy)
        .into_iter()
        .find_map(|candidate_lower| {
            let candidate_word = apply_word_case(original_word, &candidate_lower);
            let candidate = format!("{converted_leading}{candidate_word}{converted_trailing}");
            (is_common_en_technical_word(&candidate_lower)
                || lem_prefers_layout_candidate(original_word, &candidate_word))
            .then_some(candidate)
        })
}

fn english_layout_autoswitch_candidates(
    converted: &str,
    policy: EnglishLayoutPolicy,
) -> Vec<String> {
    let lower = converted.to_ascii_lowercase();
    let mut out = Vec::new();
    if is_short_ascii_layout_token(&lower) && !is_common_en_technical_word(&lower) {
        return out;
    }
    if is_short_plain_dictionary_fragment(&lower) && !is_common_en_technical_word(&lower) {
        return out;
    }
    if is_known_english_layout_autoswitch_word(&lower)
        || matches!(policy, EnglishLayoutPolicy::Experimental)
            && is_known_english_word_for_experimental_layout(&lower)
    {
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

fn is_short_ascii_layout_token(word: &str) -> bool {
    word.chars().filter(|ch| ch.is_ascii_alphabetic()).count() <= 2
}

fn is_short_plain_dictionary_fragment(word: &str) -> bool {
    word.chars().filter(|ch| ch.is_ascii_alphabetic()).count() == 3
}

fn is_known_english_word_for_experimental_layout(word: &str) -> bool {
    let identity = recognize_token(word);
    identity.script == WordScript::Ascii && identity.known_en && identity.is_plain_word()
}

fn is_plain_cyrillic_layout_token(token: &str) -> bool {
    token.chars().any(is_cyrillic_letter)
        && token.chars().all(|ch| {
            is_cyrillic_letter(ch)
                || ch.is_ascii_digit()
                || matches!(
                    ch,
                    ',' | '.' | '!' | '?' | ':' | ';' | '$' | '%' | '&' | '#' | '@' | '-' | '_'
                )
        })
}

#[cfg(test)]
mod tests {
    use super::correct_wrong_layout_cyrillic_word;

    #[test]
    fn short_plain_dictionary_fragments_do_not_autoswitch_to_english() {
        assert_eq!(correct_wrong_layout_cyrillic_word("щдф"), None);
        assert_eq!(correct_wrong_layout_cyrillic_word("сщла"), None);
        assert_eq!(
            correct_wrong_layout_cyrillic_word("сщдф"),
            Some("cola".to_string())
        );
        assert_eq!(
            correct_wrong_layout_cyrillic_word("дфн"),
            Some("lay".to_string())
        );
    }
}

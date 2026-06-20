use crate::lexicon::is_user_protected_ascii_word;
use crate::word_reader::split_word_punctuation;

use super::candidate::ascii_to_russian_layout_candidate;
use super::symbols::is_ascii_layout_token_symbol;

pub(super) fn correct_word_preserving_trailing_punctuation(token: &str) -> Option<String> {
    let trailing_start = token
        .char_indices()
        .rev()
        .find(|(_, ch)| ch.is_ascii_alphanumeric())
        .map(|(idx, ch)| idx + ch.len_utf8())
        .unwrap_or(0);
    if trailing_start == 0 || trailing_start == token.len() {
        return None;
    }

    let core = &token[..trailing_start];
    let trailing = &token[trailing_start..];
    if !core_is_layout_token(core) || !trailing.chars().all(is_ascii_layout_token_symbol) {
        return None;
    }

    let (_, original_word, _) = split_word_punctuation(core);
    if original_word.is_empty() || is_user_protected_ascii_word(original_word) {
        return None;
    }
    let normalized = ascii_to_russian_layout_candidate(core, false)?.replacement;
    Some(format!("{normalized}{}", converted_trailing(trailing)))
}

fn core_is_layout_token(core: &str) -> bool {
    core.chars().any(|ch| ch.is_ascii_alphabetic())
        && core
            .chars()
            .all(|ch| ch.is_ascii_alphabetic() || is_ascii_layout_token_symbol(ch))
}

fn converted_trailing(trailing: &str) -> String {
    let converted = crate::dict::convert(trailing, crate::dict::Direction::Us2Ru);
    if converted == trailing
        || converted
            .chars()
            .any(|ch| matches!(ch, 'А'..='я' | 'ё' | 'Ё'))
    {
        trailing.to_string()
    } else {
        converted
    }
}

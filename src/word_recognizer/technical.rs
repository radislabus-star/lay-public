use crate::keyboard::is_cyrillic_letter;
use crate::lexicon::{is_common_en_technical_word, is_user_protected_ascii_word};

pub(super) fn is_technical_token(core: &str) -> bool {
    let lower = core.to_ascii_lowercase();
    if is_common_en_technical_word(&lower) {
        return true;
    }
    if core.contains("://") || core.contains('@') {
        return true;
    }
    if core.chars().any(|ch| ch.is_ascii_digit()) {
        return true;
    }
    if is_ascii_technical_or_brand_token(core) {
        return true;
    }
    false
}

pub fn is_cli_option_token(token: &str) -> bool {
    let rest = if let Some(rest) = token.strip_prefix("--") {
        rest
    } else if let Some(rest) = token.strip_prefix('-') {
        rest
    } else {
        return false;
    };

    !rest.is_empty()
        && rest.chars().any(|ch| ch.is_ascii_alphabetic())
        && rest
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '=' | ':' | '.' | '/'))
}

pub fn is_protected_ascii_token(core: &str) -> bool {
    if !core.chars().any(|ch| ch.is_ascii_alphabetic()) {
        return false;
    }
    core.is_ascii()
        && (is_user_protected_ascii_word(core)
            || has_domain_like_dot(core)
            || core.contains('@')
            || core.contains("://")
            || core.contains('/')
            || core.contains('\\')
            || is_upper_ascii_acronym(core)
            || is_mixed_case_ascii_brand(core))
}

pub fn is_ascii_technical_token(core: &str) -> bool {
    core.is_ascii()
        && core.chars().any(|ch| ch.is_ascii_alphabetic())
        && core.chars().all(|ch| {
            ch.is_ascii_alphanumeric()
                || matches!(ch, '-' | '_' | '.' | '@' | '/' | '\\' | ':' | '+' | '#')
        })
        && core
            .chars()
            .any(|ch| matches!(ch, '-' | '_' | '.' | '@' | '/' | '\\' | ':' | '+' | '#'))
}

pub fn is_ascii_technical_or_brand_token(core: &str) -> bool {
    core.is_ascii()
        && core.chars().any(|ch| ch.is_ascii_alphabetic())
        && (has_domain_like_dot(core)
            || has_ascii_hyphen_or_underscore_segments(core)
            || core
                .chars()
                .any(|ch| matches!(ch, '@' | '/' | '\\' | ':' | '+' | '#'))
            || is_upper_ascii_acronym(core)
            || is_mixed_case_ascii_brand(core))
}

pub fn is_ascii_titlecase_token(core: &str) -> bool {
    if !core.is_ascii() || !core.chars().all(|ch| ch.is_ascii_alphabetic()) {
        return false;
    }

    let mut letters = core.chars();
    let Some(first) = letters.next() else {
        return false;
    };
    first.is_ascii_uppercase()
        && core.chars().count() >= 4
        && letters.all(|ch| ch.is_ascii_lowercase())
}

pub fn is_upper_ascii_acronym(core: &str) -> bool {
    let letters: Vec<char> = core.chars().filter(|ch| ch.is_ascii_alphabetic()).collect();
    (2..=4).contains(&letters.len()) && letters.iter().all(|ch| ch.is_ascii_uppercase())
}

pub fn is_mixed_case_ascii_brand(core: &str) -> bool {
    let letters: Vec<char> = core.chars().filter(|ch| ch.is_ascii_alphabetic()).collect();
    letters.len() >= 4
        && letters.iter().any(|ch| ch.is_ascii_lowercase())
        && letters.iter().skip(1).any(|ch| ch.is_ascii_uppercase())
}

pub fn is_mixed_cyrillic_ascii_alpha_token(core: &str) -> bool {
    let mut has_cyrillic = false;
    let mut has_ascii = false;
    for ch in core.chars() {
        if is_cyrillic_letter(ch) {
            has_cyrillic = true;
        } else if ch.is_ascii_alphabetic() {
            has_ascii = true;
        } else if !matches!(ch, '-' | '\'') {
            return false;
        }
    }
    has_cyrillic && has_ascii
}

fn has_domain_like_dot(core: &str) -> bool {
    core.split('.').count() >= 2
        && core.rsplit_once('.').is_some_and(|(name, tld)| {
            name.chars().filter(|ch| ch.is_ascii_alphabetic()).count() >= 2
                && (2..=4).contains(&tld.chars().count())
                && tld.chars().all(|ch| ch.is_ascii_alphabetic())
        })
}

fn has_ascii_hyphen_or_underscore_segments(core: &str) -> bool {
    core.split(['-', '_']).count() >= 2
        && core
            .split(['-', '_'])
            .all(|part| part.chars().filter(|ch| ch.is_ascii_alphabetic()).count() >= 2)
}

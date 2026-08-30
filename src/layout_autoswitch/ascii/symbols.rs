use crate::word_recognizer::{is_cli_option_token, is_protected_ascii_token};

pub(crate) fn ascii_layout_prefix_can_be_letter(prefix: &str) -> bool {
    prefix.chars().any(is_ascii_layout_letter_symbol)
}

pub(crate) fn is_ascii_layout_letter_symbol(ch: char) -> bool {
    matches!(
        ch,
        '\'' | ';' | '[' | ']' | '`' | ',' | '.' | '-' | '{' | '}' | ':' | '"' | '<' | '>' | '~'
    )
}

/// True when every byte is produced by a physical key that represents a
/// Russian letter in the US layout. Punctuation-shaped keys stay lexical only
/// when the token also contains an ASCII letter; punctuation by itself remains
/// a boundary.
pub(crate) fn is_ascii_layout_letter_surface(surface: &str) -> bool {
    surface.is_ascii()
        && surface.chars().any(|ch| ch.is_ascii_alphabetic())
        && surface
            .chars()
            .all(|ch| ch.is_ascii_alphabetic() || is_ascii_layout_letter_symbol(ch))
}

pub(super) fn is_plain_ascii_layout_token(token: &str) -> bool {
    token.is_ascii()
        && token.chars().any(|ch| ch.is_ascii_alphabetic())
        && !token.chars().any(|ch| ch.is_ascii_digit())
        && token
            .chars()
            .all(|ch| ch.is_ascii_alphabetic() || is_ascii_layout_token_symbol(ch))
}

pub(super) fn has_ascii_shift_letter_signal(token: &str) -> bool {
    token.chars().any(is_ascii_shift_letter_symbol)
}

pub(super) fn is_ascii_shift_letter_symbol(ch: char) -> bool {
    matches!(ch, '{' | '}' | ':' | '"' | '<' | '>' | '~')
}

pub(crate) fn is_ascii_layout_token_symbol(ch: char) -> bool {
    is_ascii_layout_letter_symbol(ch)
        || matches!(
            ch,
            '/' | '?' | '!' | '$' | '%' | '^' | '&' | '#' | '@' | '_'
        )
}

pub(crate) fn is_protected_ascii_layout_token(token: &str) -> bool {
    is_protected_ascii_token(token)
}

pub(super) fn is_blocked_ascii_layout_token(token: &str) -> bool {
    is_cli_option_token(token) || !is_plain_ascii_layout_token(token)
}

use crate::keyboard::is_cyrillic_letter;

use super::identity::WordScript;

pub(super) fn detect_script(core: &str) -> WordScript {
    let mut has_cyrillic = false;
    let mut has_ascii = false;
    let mut has_digit = false;
    let mut has_other = false;

    for ch in core.chars() {
        if is_cyrillic_letter(ch) {
            has_cyrillic = true;
        } else if ch.is_ascii_alphabetic() {
            has_ascii = true;
        } else if ch.is_ascii_digit() {
            has_digit = true;
        } else if is_token_separator(ch) {
        } else {
            has_other = true;
        }
    }

    match (has_cyrillic, has_ascii, has_digit, has_other) {
        (false, false, false, _) => WordScript::Other,
        (true, false, false, false) => WordScript::Cyrillic,
        (false, true, false, false) => WordScript::Ascii,
        (false, false, true, false) => WordScript::Numeric,
        (true, true, _, false) => WordScript::Mixed,
        (true, false, true, false) => WordScript::Mixed,
        (false, true, true, false) => WordScript::Mixed,
        _ => WordScript::Other,
    }
}

fn is_token_separator(ch: char) -> bool {
    matches!(
        ch,
        '-' | '_'
            | '.'
            | '/'
            | '+'
            | ','
            | ';'
            | '\''
            | '['
            | ']'
            | '`'
            | '?'
            | '!'
            | ':'
            | '$'
            | '%'
            | '^'
            | '&'
            | '#'
            | '@'
    )
}

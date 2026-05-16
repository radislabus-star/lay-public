//! Physical keycode mapping shared by desktop adapters.
//!
//! The daemon still owns evdev listening and uinput replay. This module only
//! describes how stored physical key events map to US/RU text.

use evdev::KeyCode;

#[derive(Clone, Copy, Debug)]
pub struct KeyEvent {
    pub keycode: u16,
    pub shift: bool,
    pub layout_is_ru: bool,
}

#[derive(Debug, Clone)]
pub struct TextInputRun {
    pub target_is_ru: bool,
    pub events: Vec<KeyEvent>,
}

#[inline]
pub fn is_typing_key(key: KeyCode) -> bool {
    use KeyCode as K;
    matches!(
        key,
        K::KEY_A
            | K::KEY_B
            | K::KEY_C
            | K::KEY_D
            | K::KEY_E
            | K::KEY_F
            | K::KEY_G
            | K::KEY_H
            | K::KEY_I
            | K::KEY_J
            | K::KEY_K
            | K::KEY_L
            | K::KEY_M
            | K::KEY_N
            | K::KEY_O
            | K::KEY_P
            | K::KEY_Q
            | K::KEY_R
            | K::KEY_S
            | K::KEY_T
            | K::KEY_U
            | K::KEY_V
            | K::KEY_W
            | K::KEY_X
            | K::KEY_Y
            | K::KEY_Z
            | K::KEY_1
            | K::KEY_2
            | K::KEY_3
            | K::KEY_4
            | K::KEY_5
            | K::KEY_6
            | K::KEY_7
            | K::KEY_8
            | K::KEY_9
            | K::KEY_0
            | K::KEY_SEMICOLON
            | K::KEY_APOSTROPHE
            | K::KEY_COMMA
            | K::KEY_DOT
            | K::KEY_LEFTBRACE
            | K::KEY_RIGHTBRACE
            | K::KEY_GRAVE
            | K::KEY_SLASH
            | K::KEY_BACKSLASH
            | K::KEY_MINUS
            | K::KEY_EQUAL
            | K::KEY_SPACE
    )
}

#[inline]
pub fn keycode_to_ru_char(keycode: u16, shift: bool) -> Option<char> {
    use KeyCode as K;
    let key = KeyCode::new(keycode);
    if shift {
        return match key {
            K::KEY_Q => Some('Й'),
            K::KEY_W => Some('Ц'),
            K::KEY_E => Some('У'),
            K::KEY_R => Some('К'),
            K::KEY_T => Some('Е'),
            K::KEY_Y => Some('Н'),
            K::KEY_U => Some('Г'),
            K::KEY_I => Some('Ш'),
            K::KEY_O => Some('Щ'),
            K::KEY_P => Some('З'),
            K::KEY_LEFTBRACE => Some('Х'),
            K::KEY_RIGHTBRACE => Some('Ъ'),
            K::KEY_A => Some('Ф'),
            K::KEY_S => Some('Ы'),
            K::KEY_D => Some('В'),
            K::KEY_F => Some('А'),
            K::KEY_G => Some('П'),
            K::KEY_H => Some('Р'),
            K::KEY_J => Some('О'),
            K::KEY_K => Some('Л'),
            K::KEY_L => Some('Д'),
            K::KEY_SEMICOLON => Some('Ж'),
            K::KEY_APOSTROPHE => Some('Э'),
            K::KEY_Z => Some('Я'),
            K::KEY_X => Some('Ч'),
            K::KEY_C => Some('С'),
            K::KEY_V => Some('М'),
            K::KEY_B => Some('И'),
            K::KEY_N => Some('Т'),
            K::KEY_M => Some('Ь'),
            K::KEY_COMMA => Some('Б'),
            K::KEY_DOT => Some('Ю'),
            K::KEY_GRAVE => Some('Ё'),
            K::KEY_1 => Some('!'),
            K::KEY_2 => Some('"'),
            K::KEY_3 => Some('№'),
            K::KEY_4 => Some(';'),
            K::KEY_5 => Some('%'),
            K::KEY_6 => Some(':'),
            K::KEY_7 => Some('?'),
            K::KEY_8 => Some('*'),
            K::KEY_9 => Some('('),
            K::KEY_0 => Some(')'),
            K::KEY_MINUS => Some('_'),
            K::KEY_EQUAL => Some('+'),
            K::KEY_SLASH => Some(','),
            K::KEY_SPACE => Some(' '),
            _ => None,
        };
    }

    match key {
        K::KEY_Q => Some('й'),
        K::KEY_W => Some('ц'),
        K::KEY_E => Some('у'),
        K::KEY_R => Some('к'),
        K::KEY_T => Some('е'),
        K::KEY_Y => Some('н'),
        K::KEY_U => Some('г'),
        K::KEY_I => Some('ш'),
        K::KEY_O => Some('щ'),
        K::KEY_P => Some('з'),
        K::KEY_LEFTBRACE => Some('х'),
        K::KEY_RIGHTBRACE => Some('ъ'),
        K::KEY_A => Some('ф'),
        K::KEY_S => Some('ы'),
        K::KEY_D => Some('в'),
        K::KEY_F => Some('а'),
        K::KEY_G => Some('п'),
        K::KEY_H => Some('р'),
        K::KEY_J => Some('о'),
        K::KEY_K => Some('л'),
        K::KEY_L => Some('д'),
        K::KEY_SEMICOLON => Some('ж'),
        K::KEY_APOSTROPHE => Some('э'),
        K::KEY_Z => Some('я'),
        K::KEY_X => Some('ч'),
        K::KEY_C => Some('с'),
        K::KEY_V => Some('м'),
        K::KEY_B => Some('и'),
        K::KEY_N => Some('т'),
        K::KEY_M => Some('ь'),
        K::KEY_COMMA => Some('б'),
        K::KEY_DOT => Some('ю'),
        K::KEY_GRAVE => Some('ё'),
        K::KEY_SLASH => Some('.'),
        K::KEY_1 => Some('1'),
        K::KEY_2 => Some('2'),
        K::KEY_3 => Some('3'),
        K::KEY_4 => Some('4'),
        K::KEY_5 => Some('5'),
        K::KEY_6 => Some('6'),
        K::KEY_7 => Some('7'),
        K::KEY_8 => Some('8'),
        K::KEY_9 => Some('9'),
        K::KEY_0 => Some('0'),
        K::KEY_MINUS => Some('-'),
        K::KEY_EQUAL => Some('='),
        K::KEY_SPACE => Some(' '),
        _ => None,
    }
}

#[inline]
pub fn keycode_to_us_char(keycode: u16, shift: bool) -> Option<char> {
    use KeyCode as K;
    let key = KeyCode::new(keycode);
    if shift {
        return match key {
            K::KEY_A => Some('A'),
            K::KEY_B => Some('B'),
            K::KEY_C => Some('C'),
            K::KEY_D => Some('D'),
            K::KEY_E => Some('E'),
            K::KEY_F => Some('F'),
            K::KEY_G => Some('G'),
            K::KEY_H => Some('H'),
            K::KEY_I => Some('I'),
            K::KEY_J => Some('J'),
            K::KEY_K => Some('K'),
            K::KEY_L => Some('L'),
            K::KEY_M => Some('M'),
            K::KEY_N => Some('N'),
            K::KEY_O => Some('O'),
            K::KEY_P => Some('P'),
            K::KEY_Q => Some('Q'),
            K::KEY_R => Some('R'),
            K::KEY_S => Some('S'),
            K::KEY_T => Some('T'),
            K::KEY_U => Some('U'),
            K::KEY_V => Some('V'),
            K::KEY_W => Some('W'),
            K::KEY_X => Some('X'),
            K::KEY_Y => Some('Y'),
            K::KEY_Z => Some('Z'),
            K::KEY_1 => Some('!'),
            K::KEY_2 => Some('@'),
            K::KEY_3 => Some('#'),
            K::KEY_4 => Some('$'),
            K::KEY_5 => Some('%'),
            K::KEY_6 => Some('^'),
            K::KEY_7 => Some('&'),
            K::KEY_8 => Some('*'),
            K::KEY_9 => Some('('),
            K::KEY_0 => Some(')'),
            K::KEY_SEMICOLON => Some(':'),
            K::KEY_APOSTROPHE => Some('"'),
            K::KEY_COMMA => Some('<'),
            K::KEY_DOT => Some('>'),
            K::KEY_LEFTBRACE => Some('{'),
            K::KEY_RIGHTBRACE => Some('}'),
            K::KEY_GRAVE => Some('~'),
            K::KEY_SLASH => Some('?'),
            K::KEY_BACKSLASH => Some('|'),
            K::KEY_MINUS => Some('_'),
            K::KEY_EQUAL => Some('+'),
            K::KEY_SPACE => Some(' '),
            _ => None,
        };
    }

    match key {
        K::KEY_A => Some('a'),
        K::KEY_B => Some('b'),
        K::KEY_C => Some('c'),
        K::KEY_D => Some('d'),
        K::KEY_E => Some('e'),
        K::KEY_F => Some('f'),
        K::KEY_G => Some('g'),
        K::KEY_H => Some('h'),
        K::KEY_I => Some('i'),
        K::KEY_J => Some('j'),
        K::KEY_K => Some('k'),
        K::KEY_L => Some('l'),
        K::KEY_M => Some('m'),
        K::KEY_N => Some('n'),
        K::KEY_O => Some('o'),
        K::KEY_P => Some('p'),
        K::KEY_Q => Some('q'),
        K::KEY_R => Some('r'),
        K::KEY_S => Some('s'),
        K::KEY_T => Some('t'),
        K::KEY_U => Some('u'),
        K::KEY_V => Some('v'),
        K::KEY_W => Some('w'),
        K::KEY_X => Some('x'),
        K::KEY_Y => Some('y'),
        K::KEY_Z => Some('z'),
        K::KEY_1 => Some('1'),
        K::KEY_2 => Some('2'),
        K::KEY_3 => Some('3'),
        K::KEY_4 => Some('4'),
        K::KEY_5 => Some('5'),
        K::KEY_6 => Some('6'),
        K::KEY_7 => Some('7'),
        K::KEY_8 => Some('8'),
        K::KEY_9 => Some('9'),
        K::KEY_0 => Some('0'),
        K::KEY_SEMICOLON => Some(';'),
        K::KEY_APOSTROPHE => Some('\''),
        K::KEY_COMMA => Some(','),
        K::KEY_DOT => Some('.'),
        K::KEY_LEFTBRACE => Some('['),
        K::KEY_RIGHTBRACE => Some(']'),
        K::KEY_GRAVE => Some('`'),
        K::KEY_SLASH => Some('/'),
        K::KEY_BACKSLASH => Some('\\'),
        K::KEY_MINUS => Some('-'),
        K::KEY_EQUAL => Some('='),
        K::KEY_SPACE => Some(' '),
        _ => None,
    }
}

#[inline]
pub fn map_original_events(events: &[KeyEvent]) -> String {
    events
        .iter()
        .filter_map(|ev| {
            if ev.layout_is_ru {
                keycode_to_ru_char(ev.keycode, ev.shift)
            } else {
                keycode_to_us_char(ev.keycode, ev.shift)
            }
        })
        .collect()
}

#[inline]
pub fn map_opposite_events(events: &[KeyEvent]) -> String {
    events
        .iter()
        .filter_map(|ev| {
            if ev.layout_is_ru {
                keycode_to_us_char(ev.keycode, ev.shift)
            } else {
                keycode_to_ru_char(ev.keycode, ev.shift)
            }
        })
        .collect()
}

#[inline]
pub fn map_events_to_layout(events: &[KeyEvent], target_is_ru: bool) -> String {
    events
        .iter()
        .filter_map(|ev| {
            if target_is_ru {
                keycode_to_ru_char(ev.keycode, ev.shift)
            } else {
                keycode_to_us_char(ev.keycode, ev.shift)
            }
        })
        .collect()
}

#[inline]
pub fn preferred_layout_for_text(text: &str, fallback_is_ru: bool) -> bool {
    text.chars()
        .rev()
        .find_map(|ch| {
            if is_cyrillic_letter(ch) {
                Some(true)
            } else if ch.is_ascii_alphabetic() {
                Some(false)
            } else {
                None
            }
        })
        .unwrap_or(fallback_is_ru)
}

pub fn text_to_uinput_runs(text: &str, fallback_is_ru: bool) -> Option<Vec<TextInputRun>> {
    let mut runs: Vec<TextInputRun> = Vec::new();
    let mut current_is_ru = preferred_layout_for_text(text, fallback_is_ru);

    for ch in text.chars() {
        let (target_is_ru, event) = char_to_layout_key_event(ch, current_is_ru)?;
        current_is_ru = target_is_ru;
        if let Some(run) = runs
            .last_mut()
            .filter(|run| run.target_is_ru == target_is_ru)
        {
            run.events.push(event);
        } else {
            runs.push(TextInputRun {
                target_is_ru,
                events: vec![event],
            });
        }
    }

    Some(runs)
}

pub fn is_cyrillic_letter(ch: char) -> bool {
    matches!(ch, 'А'..='я' | 'ё' | 'Ё')
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReplayLayoutDecision {
    pub target_is_ru: bool,
    pub mixed_layouts: bool,
}

pub fn replay_layout_decision(events: &[KeyEvent]) -> ReplayLayoutDecision {
    let typed_layouts: Vec<bool> = events
        .iter()
        .filter(|ev| is_layout_decision_key(KeyCode::new(ev.keycode)))
        .map(|ev| ev.layout_is_ru)
        .collect();
    let first_layout = typed_layouts.first().copied().unwrap_or(false);
    let last_layout = typed_layouts.last().copied().unwrap_or(first_layout);
    let mixed_layouts = typed_layouts.iter().any(|layout| *layout != first_layout);
    let target_is_ru = if mixed_layouts {
        mixed_visual_latin_word_target_layout(events).unwrap_or(last_layout)
    } else {
        !first_layout
    };
    ReplayLayoutDecision {
        target_is_ru,
        mixed_layouts,
    }
}

pub fn is_layout_decision_key(key: KeyCode) -> bool {
    is_typing_key(key) && key != KeyCode::KEY_SPACE
}

pub fn mark_word_layout(word: &mut [KeyEvent], layout_is_ru: bool) {
    for event in word {
        if is_typing_key(KeyCode::new(event.keycode)) {
            event.layout_is_ru = layout_is_ru;
        }
    }
}

pub fn split_event_words(events: &[KeyEvent]) -> Option<Vec<&[KeyEvent]>> {
    if events.is_empty() {
        return None;
    }

    let end = if events
        .last()
        .is_some_and(|event| event.keycode == KeyCode::KEY_SPACE.code())
    {
        events.len().saturating_sub(1)
    } else {
        events.len()
    };
    if end == 0 {
        return None;
    }

    let mut words = Vec::new();
    let mut start = 0;
    for (idx, event) in events.iter().take(end).enumerate() {
        if event.keycode == KeyCode::KEY_SPACE.code() {
            if start < idx {
                words.push(&events[start..idx]);
            }
            start = idx + 1;
        }
    }
    if start < end {
        words.push(&events[start..end]);
    }

    (!words.is_empty()).then_some(words)
}

pub fn mixed_visual_latin_word_target_layout(word: &[KeyEvent]) -> Option<bool> {
    if word.is_empty()
        || word
            .iter()
            .any(|event| event.keycode == KeyCode::KEY_SPACE.code())
    {
        return None;
    }

    let first_layout = word.first()?.layout_is_ru;
    if word.iter().all(|event| event.layout_is_ru == first_layout) {
        return None;
    }

    let mut latin_count = 0usize;
    let mut same_key_homoglyph_count = 0usize;
    let mut other_cyrillic_count = 0usize;

    for event in word {
        let ch = original_event_char(event)?;
        if ch.is_ascii_alphabetic() {
            latin_count += 1;
        } else if is_cyrillic_letter(ch) {
            if same_key_latin_cyrillic_homoglyph(event) {
                same_key_homoglyph_count += 1;
            } else {
                other_cyrillic_count += 1;
            }
        }
    }

    if latin_count >= 2 && same_key_homoglyph_count > 0 && other_cyrillic_count == 0 {
        Some(true)
    } else {
        None
    }
}

pub fn original_event_char(event: &KeyEvent) -> Option<char> {
    if event.layout_is_ru {
        keycode_to_ru_char(event.keycode, event.shift)
    } else {
        keycode_to_us_char(event.keycode, event.shift)
    }
}

fn same_key_latin_cyrillic_homoglyph(event: &KeyEvent) -> bool {
    matches!(
        (
            keycode_to_us_char(event.keycode, event.shift),
            keycode_to_ru_char(event.keycode, event.shift),
        ),
        (Some('c' | 'C'), Some('с' | 'С'))
    )
}

fn char_to_layout_key_event(ch: char, current_is_ru: bool) -> Option<(bool, KeyEvent)> {
    if is_cyrillic_letter(ch) {
        return char_to_ru_key_event(ch).map(|event| (true, event));
    }
    if ch.is_ascii_alphabetic() {
        return char_to_us_key_event(ch).map(|event| (false, event));
    }
    if current_is_ru {
        if let Some(event) = char_to_ru_key_event(ch) {
            return Some((true, event));
        }
    }
    if let Some(event) = char_to_us_key_event(ch) {
        return Some((false, event));
    }
    char_to_ru_key_event(ch).map(|event| (true, event))
}

fn char_to_ru_key_event(ch: char) -> Option<KeyEvent> {
    use KeyCode as K;
    let mut chars = ch.to_lowercase();
    let lower = chars.next()?;
    if chars.next().is_some() {
        return None;
    }
    let shift = ch.is_uppercase();
    let (key, force_shift) = match lower {
        'й' => (K::KEY_Q, false),
        'ц' => (K::KEY_W, false),
        'у' => (K::KEY_E, false),
        'к' => (K::KEY_R, false),
        'е' => (K::KEY_T, false),
        'н' => (K::KEY_Y, false),
        'г' => (K::KEY_U, false),
        'ш' => (K::KEY_I, false),
        'щ' => (K::KEY_O, false),
        'з' => (K::KEY_P, false),
        'х' => (K::KEY_LEFTBRACE, false),
        'ъ' => (K::KEY_RIGHTBRACE, false),
        'ф' => (K::KEY_A, false),
        'ы' => (K::KEY_S, false),
        'в' => (K::KEY_D, false),
        'а' => (K::KEY_F, false),
        'п' => (K::KEY_G, false),
        'р' => (K::KEY_H, false),
        'о' => (K::KEY_J, false),
        'л' => (K::KEY_K, false),
        'д' => (K::KEY_L, false),
        'ж' => (K::KEY_SEMICOLON, false),
        'э' => (K::KEY_APOSTROPHE, false),
        'я' => (K::KEY_Z, false),
        'ч' => (K::KEY_X, false),
        'с' => (K::KEY_C, false),
        'м' => (K::KEY_V, false),
        'и' => (K::KEY_B, false),
        'т' => (K::KEY_N, false),
        'ь' => (K::KEY_M, false),
        'б' => (K::KEY_COMMA, false),
        'ю' => (K::KEY_DOT, false),
        'ё' => (K::KEY_GRAVE, false),
        '1' => (K::KEY_1, false),
        '2' => (K::KEY_2, false),
        '3' => (K::KEY_3, false),
        '4' => (K::KEY_4, false),
        '5' => (K::KEY_5, false),
        '6' => (K::KEY_6, false),
        '7' => (K::KEY_7, false),
        '8' => (K::KEY_8, false),
        '9' => (K::KEY_9, false),
        '0' => (K::KEY_0, false),
        '!' => (K::KEY_1, true),
        '"' => (K::KEY_2, true),
        '№' => (K::KEY_3, true),
        ';' => (K::KEY_4, true),
        '%' => (K::KEY_5, true),
        ':' => (K::KEY_6, true),
        '?' => (K::KEY_7, true),
        '*' => (K::KEY_8, true),
        '(' => (K::KEY_9, true),
        ')' => (K::KEY_0, true),
        '-' => (K::KEY_MINUS, false),
        '_' => (K::KEY_MINUS, true),
        '=' => (K::KEY_EQUAL, false),
        '+' => (K::KEY_EQUAL, true),
        '.' => (K::KEY_SLASH, false),
        ',' => (K::KEY_SLASH, true),
        ' ' => (K::KEY_SPACE, false),
        _ => return None,
    };

    Some(KeyEvent {
        keycode: key.code(),
        shift: shift || force_shift,
        layout_is_ru: true,
    })
}

fn char_to_us_key_event(ch: char) -> Option<KeyEvent> {
    use KeyCode as K;
    let (key, shift) = match ch {
        'a' | 'A' => (K::KEY_A, ch.is_uppercase()),
        'b' | 'B' => (K::KEY_B, ch.is_uppercase()),
        'c' | 'C' => (K::KEY_C, ch.is_uppercase()),
        'd' | 'D' => (K::KEY_D, ch.is_uppercase()),
        'e' | 'E' => (K::KEY_E, ch.is_uppercase()),
        'f' | 'F' => (K::KEY_F, ch.is_uppercase()),
        'g' | 'G' => (K::KEY_G, ch.is_uppercase()),
        'h' | 'H' => (K::KEY_H, ch.is_uppercase()),
        'i' | 'I' => (K::KEY_I, ch.is_uppercase()),
        'j' | 'J' => (K::KEY_J, ch.is_uppercase()),
        'k' | 'K' => (K::KEY_K, ch.is_uppercase()),
        'l' | 'L' => (K::KEY_L, ch.is_uppercase()),
        'm' | 'M' => (K::KEY_M, ch.is_uppercase()),
        'n' | 'N' => (K::KEY_N, ch.is_uppercase()),
        'o' | 'O' => (K::KEY_O, ch.is_uppercase()),
        'p' | 'P' => (K::KEY_P, ch.is_uppercase()),
        'q' | 'Q' => (K::KEY_Q, ch.is_uppercase()),
        'r' | 'R' => (K::KEY_R, ch.is_uppercase()),
        's' | 'S' => (K::KEY_S, ch.is_uppercase()),
        't' | 'T' => (K::KEY_T, ch.is_uppercase()),
        'u' | 'U' => (K::KEY_U, ch.is_uppercase()),
        'v' | 'V' => (K::KEY_V, ch.is_uppercase()),
        'w' | 'W' => (K::KEY_W, ch.is_uppercase()),
        'x' | 'X' => (K::KEY_X, ch.is_uppercase()),
        'y' | 'Y' => (K::KEY_Y, ch.is_uppercase()),
        'z' | 'Z' => (K::KEY_Z, ch.is_uppercase()),
        '1' => (K::KEY_1, false),
        '2' => (K::KEY_2, false),
        '3' => (K::KEY_3, false),
        '4' => (K::KEY_4, false),
        '5' => (K::KEY_5, false),
        '6' => (K::KEY_6, false),
        '7' => (K::KEY_7, false),
        '8' => (K::KEY_8, false),
        '9' => (K::KEY_9, false),
        '0' => (K::KEY_0, false),
        '!' => (K::KEY_1, true),
        '@' => (K::KEY_2, true),
        '#' => (K::KEY_3, true),
        '$' => (K::KEY_4, true),
        '%' => (K::KEY_5, true),
        '^' => (K::KEY_6, true),
        '&' => (K::KEY_7, true),
        '*' => (K::KEY_8, true),
        '(' => (K::KEY_9, true),
        ')' => (K::KEY_0, true),
        ';' => (K::KEY_SEMICOLON, false),
        ':' => (K::KEY_SEMICOLON, true),
        '\'' => (K::KEY_APOSTROPHE, false),
        '"' => (K::KEY_APOSTROPHE, true),
        ',' => (K::KEY_COMMA, false),
        '<' => (K::KEY_COMMA, true),
        '.' => (K::KEY_DOT, false),
        '>' => (K::KEY_DOT, true),
        '[' => (K::KEY_LEFTBRACE, false),
        '{' => (K::KEY_LEFTBRACE, true),
        ']' => (K::KEY_RIGHTBRACE, false),
        '}' => (K::KEY_RIGHTBRACE, true),
        '`' => (K::KEY_GRAVE, false),
        '~' => (K::KEY_GRAVE, true),
        '/' => (K::KEY_SLASH, false),
        '?' => (K::KEY_SLASH, true),
        '\\' => (K::KEY_BACKSLASH, false),
        '|' => (K::KEY_BACKSLASH, true),
        '-' => (K::KEY_MINUS, false),
        '_' => (K::KEY_MINUS, true),
        '=' => (K::KEY_EQUAL, false),
        '+' => (K::KEY_EQUAL, true),
        ' ' => (K::KEY_SPACE, false),
        _ => return None,
    };

    Some(KeyEvent {
        keycode: key.code(),
        shift,
        layout_is_ru: false,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn us_event(key: KeyCode) -> KeyEvent {
        KeyEvent {
            keycode: key.code(),
            shift: false,
            layout_is_ru: false,
        }
    }

    fn ru_event(key: KeyCode, shift: bool) -> KeyEvent {
        KeyEvent {
            keycode: key.code(),
            shift,
            layout_is_ru: true,
        }
    }

    #[test]
    fn maps_wrong_layout_word_to_russian_target() {
        let events = [
            us_event(KeyCode::KEY_L),
            us_event(KeyCode::KEY_T),
            us_event(KeyCode::KEY_K),
            us_event(KeyCode::KEY_F),
            us_event(KeyCode::KEY_Q),
        ];

        assert_eq!(map_original_events(&events), "ltkfq");
        assert_eq!(map_events_to_layout(&events, true), "делай");
        assert_eq!(map_opposite_events(&events), "делай");
    }

    #[test]
    fn maps_shifted_ru_currency_key_to_us_dollar_on_replay() {
        let events = [
            ru_event(KeyCode::KEY_4, false),
            ru_event(KeyCode::KEY_0, false),
            ru_event(KeyCode::KEY_0, false),
            ru_event(KeyCode::KEY_0, false),
            ru_event(KeyCode::KEY_4, true),
        ];

        assert_eq!(map_original_events(&events), "4000;");
        assert_eq!(map_events_to_layout(&events, false), "4000$");
        assert_eq!(map_opposite_events(&events), "4000$");
        assert_eq!(
            replay_layout_decision(&events),
            ReplayLayoutDecision {
                target_is_ru: false,
                mixed_layouts: false,
            }
        );
    }

    #[test]
    fn text_insert_can_type_russian_shifted_punctuation_on_ru_layout() {
        let runs = text_to_uinput_runs("4000; 50%", true).expect("typable text");

        assert_eq!(runs.len(), 1);
        assert!(runs[0].target_is_ru);
        assert_eq!(map_events_to_layout(&runs[0].events, true), "4000; 50%");
    }

    #[test]
    fn typing_key_excludes_shift_and_includes_space() {
        assert!(is_typing_key(KeyCode::KEY_A));
        assert!(is_typing_key(KeyCode::KEY_SPACE));
        assert!(!is_typing_key(KeyCode::KEY_LEFTSHIFT));
    }

    #[test]
    fn splits_text_insert_into_layout_runs() {
        let runs = text_to_uinput_runs("Привет Double", true).expect("typable text");

        assert_eq!(runs.len(), 2);
        assert!(runs[0].target_is_ru);
        assert!(!runs[1].target_is_ru);
        assert_eq!(map_events_to_layout(&runs[0].events, true), "Привет ");
        assert_eq!(map_events_to_layout(&runs[1].events, false), "Double");
    }

    #[test]
    fn replay_layout_decision_ignores_space() {
        let events = [
            KeyEvent {
                keycode: KeyCode::KEY_A.code(),
                shift: false,
                layout_is_ru: false,
            },
            KeyEvent {
                keycode: KeyCode::KEY_SPACE.code(),
                shift: false,
                layout_is_ru: true,
            },
        ];

        assert!(!is_layout_decision_key(KeyCode::KEY_SPACE));
        assert_eq!(
            replay_layout_decision(&events),
            ReplayLayoutDecision {
                target_is_ru: true,
                mixed_layouts: false,
            }
        );
    }

    #[test]
    fn splits_event_words_without_trailing_space_word() {
        let events = [
            us_event(KeyCode::KEY_A),
            KeyEvent {
                keycode: KeyCode::KEY_SPACE.code(),
                shift: false,
                layout_is_ru: false,
            },
            us_event(KeyCode::KEY_B),
            KeyEvent {
                keycode: KeyCode::KEY_SPACE.code(),
                shift: false,
                layout_is_ru: false,
            },
        ];
        let words = split_event_words(&events).expect("words");

        assert_eq!(words.len(), 2);
        assert_eq!(map_original_events(words[0]), "a");
        assert_eq!(map_original_events(words[1]), "b");
    }

    #[test]
    fn marks_only_typing_keys_layout() {
        let mut events = [
            us_event(KeyCode::KEY_A),
            KeyEvent {
                keycode: KeyCode::KEY_LEFTSHIFT.code(),
                shift: false,
                layout_is_ru: false,
            },
        ];

        mark_word_layout(&mut events, true);

        assert!(events[0].layout_is_ru);
        assert!(!events[1].layout_is_ru);
    }
}

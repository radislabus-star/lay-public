//! Словарная конвертация US (qwerty) ↔ RU (йцукен).
//! Чистая функция, никаких аллокаций кроме результата.

use std::collections::HashMap;
use std::sync::OnceLock;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Us2Ru,
    Ru2Us,
}

/// Пары соответствия US → RU. Регистр генерируется автоматически.
const PAIRS: &[(char, char)] = &[
    // row 2 qwerty
    ('q', 'й'),
    ('w', 'ц'),
    ('e', 'у'),
    ('r', 'к'),
    ('t', 'е'),
    ('y', 'н'),
    ('u', 'г'),
    ('i', 'ш'),
    ('o', 'щ'),
    ('p', 'з'),
    ('[', 'х'),
    (']', 'ъ'),
    // row 3 asdfg
    ('a', 'ф'),
    ('s', 'ы'),
    ('d', 'в'),
    ('f', 'а'),
    ('g', 'п'),
    ('h', 'р'),
    ('j', 'о'),
    ('k', 'л'),
    ('l', 'д'),
    (';', 'ж'),
    ('\'', 'э'),
    // row 4 zxcvb
    ('z', 'я'),
    ('x', 'ч'),
    ('c', 'с'),
    ('v', 'м'),
    ('b', 'и'),
    ('n', 'т'),
    ('m', 'ь'),
    (',', 'б'),
    ('.', 'ю'),
    // знаки на разных кнопках
    ('/', '.'),
    ('?', ','),
    ('@', '"'),
    ('#', '№'),
    ('$', ';'),
    ('^', ':'),
    ('&', '?'),
    ('`', 'ё'),
];

const SHIFT_PAIRS: &[(char, char)] = &[
    ('{', 'Х'),
    ('}', 'Ъ'),
    (':', 'Ж'),
    ('"', 'Э'),
    ('<', 'Б'),
    ('>', 'Ю'),
    ('~', 'Ё'),
];

fn build_us_to_ru() -> HashMap<char, char> {
    let mut m = HashMap::with_capacity(PAIRS.len() * 2);
    for &(u, r) in PAIRS {
        m.insert(u, r);
        // Регистр: для букв генерируем uppercase. Для знаков — нет смысла.
        if u.is_alphabetic() {
            // upper char of ascii is single char
            for upper_u in u.to_uppercase() {
                for upper_r in r.to_uppercase() {
                    m.insert(upper_u, upper_r);
                }
            }
        }
    }
    for &(u, r) in SHIFT_PAIRS {
        m.insert(u, r);
    }
    m
}

fn build_ru_to_us() -> HashMap<char, char> {
    let mut m = HashMap::with_capacity(PAIRS.len() * 2);
    for &(u, r) in PAIRS {
        m.insert(r, u);
        if u.is_alphabetic() {
            for upper_r in r.to_uppercase() {
                for upper_u in u.to_uppercase() {
                    m.insert(upper_r, upper_u);
                }
            }
        }
    }
    for &(u, r) in SHIFT_PAIRS {
        m.insert(r, u);
    }
    m
}

fn us_to_ru() -> &'static HashMap<char, char> {
    static T: OnceLock<HashMap<char, char>> = OnceLock::new();
    T.get_or_init(build_us_to_ru)
}

fn ru_to_us() -> &'static HashMap<char, char> {
    static T: OnceLock<HashMap<char, char>> = OnceLock::new();
    T.get_or_init(build_ru_to_us)
}

fn is_cyrillic(c: char) -> bool {
    matches!(c, 'А'..='я' | 'ё' | 'Ё')
}

fn is_latin(c: char) -> bool {
    c.is_ascii_alphabetic()
}

pub fn detect_direction(text: &str) -> Direction {
    let cyr = text.chars().filter(|&c| is_cyrillic(c)).count();
    let lat = text.chars().filter(|&c| is_latin(c)).count();
    if cyr > lat {
        Direction::Ru2Us
    } else {
        Direction::Us2Ru
    }
}

pub fn convert(text: &str, direction: Direction) -> String {
    let table = match direction {
        Direction::Us2Ru => us_to_ru(),
        Direction::Ru2Us => ru_to_us(),
    };
    text.chars().map(|c| *table.get(&c).unwrap_or(&c)).collect()
}

#[cfg(test)]
#[path = "dict_tests.rs"]
mod tests;

const CYRILLIC: &str = "абвгдеёжзийклмнопрстуфхцчшщъыьэюя";
const ASCII: &str = "abcdefghijklmnopqrstuvwxyz";

pub(crate) fn alphabet_successor(ch: char) -> Option<char> {
    rotate_in_alphabet(ch, CYRILLIC, 1).or_else(|| rotate_in_alphabet(ch, ASCII, 1))
}

pub(crate) fn alphabet_predecessor(ch: char) -> Option<char> {
    rotate_in_alphabet(ch, CYRILLIC, -1).or_else(|| rotate_in_alphabet(ch, ASCII, -1))
}

fn rotate_in_alphabet(ch: char, alphabet: &str, offset: isize) -> Option<char> {
    let lower = ch.to_lowercase().next()?;
    let letters = alphabet.chars().collect::<Vec<_>>();
    let index = letters.iter().position(|candidate| *candidate == lower)?;
    let replacement =
        letters[(index as isize + offset).rem_euclid(letters.len() as isize) as usize];
    if ch.is_uppercase() {
        replacement.to_uppercase().next()
    } else {
        Some(replacement)
    }
}

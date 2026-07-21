const CYRILLIC: &str = "абвгдеёжзийклмнопрстуфхцчшщъыьэюя";
const ASCII: &str = "abcdefghijklmnopqrstuvwxyz";

pub(crate) fn alphabet_successor(ch: char) -> Option<char> {
    rotate_in_alphabet(ch, CYRILLIC).or_else(|| rotate_in_alphabet(ch, ASCII))
}

fn rotate_in_alphabet(ch: char, alphabet: &str) -> Option<char> {
    let lower = ch.to_lowercase().next()?;
    let letters = alphabet.chars().collect::<Vec<_>>();
    let index = letters.iter().position(|candidate| *candidate == lower)?;
    let replacement = letters[(index + 1) % letters.len()];
    if ch.is_uppercase() {
        replacement.to_uppercase().next()
    } else {
        Some(replacement)
    }
}

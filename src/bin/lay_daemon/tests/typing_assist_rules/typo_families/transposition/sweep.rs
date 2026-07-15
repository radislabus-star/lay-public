use super::*;

#[test]
fn typing_assist_transposition_sweep_over_generated_forms() {
    let mut checked = 0usize;
    let mut repaired = 0usize;
    let mut misses = Vec::new();

    for word in russian_generated_form_dictionary()
        .iter()
        .filter(|word| {
            let len = word.chars().count();
            (6..=10).contains(&len) && word.chars().all(is_cyrillic_letter)
        })
        .take(200)
    {
        let Some(typo) = first_internal_transposition_typo(word) else {
            continue;
        };
        if is_known_russian_word_or_form(&typo) {
            continue;
        }
        checked += 1;
        let input = format!("{typo} ");
        let expected = format!("{word} ");
        if select_typing_assist_exact(&input) == Some(expected) {
            repaired += 1;
        } else if misses.len() < 12 {
            misses.push((typo, word.clone()));
        }
    }

    assert!(checked >= 80, "transposition sweep too small: {checked}");
    assert!(
        repaired * 100 / checked >= 80,
        "transposition coverage too low: {repaired}/{checked}, misses={misses:?}"
    );
}

fn first_internal_transposition_typo(word: &str) -> Option<String> {
    let chars: Vec<char> = word.chars().collect();
    for idx in 1..chars.len().saturating_sub(2) {
        if chars[idx] == chars[idx + 1] {
            continue;
        }
        let mut typo = chars.clone();
        typo.swap(idx, idx + 1);
        return Some(typo.into_iter().collect());
    }
    None
}

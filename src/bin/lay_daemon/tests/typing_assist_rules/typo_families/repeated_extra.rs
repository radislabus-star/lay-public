use super::*;

#[test]
fn typing_assist_fixes_extra_repeated_letter() {
    for row in fixture_rows("daemon_typing_assist_repeated_letter.tsv") {
        assert_eq!(row.len(), 2, "repeated-letter fixture must be TSV");
        assert_eq!(apply_typing_assist_exact(&row[0]), Some(row[1].clone()));
    }
    for input in fixture_lines("daemon_typing_assist_repeated_letter_keep.txt") {
        assert_eq!(apply_typing_assist_exact(&input), None, "input={input:?}");
    }
}

#[test]
fn extra_letter_rule_defers_to_missing_letter_candidates() {
    let mut words: Vec<String> = russian_generated_form_dictionary()
        .iter()
        .filter(|word| (7..=12).contains(&word.chars().count()))
        .cloned()
        .collect();
    words.sort();

    let mut checked = 0usize;
    'outer: for word in words {
        let chars: Vec<char> = word.chars().collect();
        for idx in 1..chars.len().saturating_sub(1) {
            let mut typo_chars = chars.clone();
            typo_chars.remove(idx);
            let typo: String = typo_chars.into_iter().collect();
            if typo.chars().count() < 6 || is_known_russian_word_or_form(&typo) {
                continue;
            }
            if correct_missing_letter(&typo).as_deref() != Some(word.as_str()) {
                continue;
            }

            assert_eq!(correct_extra_letters(&typo), None, "typo={typo:?}");
            checked += 1;
            if checked >= 12 {
                break 'outer;
            }
            break;
        }
    }

    assert!(checked >= 12, "checked={checked}");
}

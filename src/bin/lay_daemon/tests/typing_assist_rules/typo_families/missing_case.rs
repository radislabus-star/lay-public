use super::*;

#[test]
fn typing_assist_fixes_common_missing_letter_typos() {
    for row in fixture_rows("daemon_typing_assist_missing_letter_core.tsv") {
        assert_eq!(row.len(), 2, "missing-letter fixture must be TSV");
        assert_eq!(
            apply_typing_assist_exact(&row[0]),
            Some(row[1].clone()),
            "input={:?}",
            row[0]
        );
    }
    for input in fixture_lines("daemon_typing_assist_missing_letter_keep.txt") {
        assert_eq!(apply_typing_assist_exact(&input), None, "input={input:?}");
    }
}

#[test]
fn typing_assist_fixes_live_user_stream_typos() {
    for word in fixture_lines("daemon_typing_assist_known_forms.txt") {
        assert!(is_known_russian_word_or_form(&word), "word={word:?}");
    }
    for row in fixture_rows("daemon_typing_assist_missing_letter_live.tsv") {
        assert_eq!(row.len(), 2, "live missing-letter fixture must be TSV");
        assert_eq!(
            apply_typing_assist_exact(&row[0]),
            Some(row[1].clone()),
            "input={:?}",
            row[0]
        );
    }
}

#[test]
fn typing_assist_normalizes_accidental_inner_uppercase() {
    for row in fixture_rows("daemon_typing_assist_inner_uppercase.tsv") {
        assert_eq!(row.len(), 2, "inner-uppercase fixture must be TSV");
        assert_eq!(
            apply_typing_assist_exact(&row[0]),
            Some(row[1].clone()),
            "input={:?}",
            row[0]
        );
    }
}

#[test]
fn typing_assist_single_letter_typos_only_use_neighbor_keys() {
    assert!(are_ru_keyboard_neighbors('з', 'х'));
    assert!(are_ru_keyboard_neighbors('р', 'п'));
    assert!(!are_ru_keyboard_neighbors('о', 'ь'));
    assert!(is_known_russian_word_or_form("кнопку"));
    for row in fixture_rows("daemon_typing_assist_neighbor_key_fix.tsv") {
        assert_eq!(row.len(), 2, "neighbor-key fixture must be TSV");
        assert_eq!(
            apply_typing_assist_exact(&row[0]),
            Some(row[1].clone()),
            "input={:?}",
            row[0]
        );
    }
}

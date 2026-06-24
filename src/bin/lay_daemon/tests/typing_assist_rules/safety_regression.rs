use super::*;

#[test]
fn typing_assist_keeps_valid_russian_words() {
    for input in fixture_lines("daemon_typing_assist_valid_word_keep.txt") {
        assert_eq!(apply_typing_assist_exact(&input), None, "input={input:?}");
    }
    for input in fixture_lines("daemon_typing_assist_valid_phrase_keep.txt") {
        assert_eq!(apply_typing_assist_exact(&input), None, "input={input:?}");
    }
}

#[test]
fn typing_assist_ignores_words_with_digits() {
    for input in fixture_lines("daemon_typing_assist_digit_word_keep.txt") {
        assert_eq!(apply_typing_assist_exact(&input), None, "input={input:?}");
    }
    for input in fixture_lines("daemon_typing_assist_digit_phrase_keep.txt") {
        assert_eq!(apply_typing_assist_exact(&input), None, "input={input:?}");
    }
}

#[test]
fn typing_assist_regression_suite_100_cases() {
    let should_fix = fixture_rows("daemon_typing_assist_regression_fix.tsv");
    for row in &should_fix {
        assert_eq!(row.len(), 2, "fix fixture must be TSV");
        let input = &row[0];
        let expected = &row[1];
        assert_eq!(
            apply_typing_assist_to_text_tail(input),
            Some(expected.clone()),
            "input={input:?}"
        );
    }

    let should_keep = fixture_lines("daemon_typing_assist_regression_keep.txt");
    for input in &should_keep {
        assert_eq!(
            apply_typing_assist_to_text_tail(input),
            None,
            "input={input:?}"
        );
    }

    let total = should_fix.len() + should_keep.len();
    assert!(
        total >= 100,
        "regression suite should keep at least 100 cases, got {total}"
    );
}

#[test]
fn auto_replace_regression_suite() {
    for row in fixture_rows("daemon_auto_replace_regression.tsv") {
        assert_eq!(row.len(), 3, "auto-replace fixture must be TSV");
        let original = &row[0];
        let target = &row[1];
        let expected = &row[2];
        assert_eq!(
            apply_auto_replace(original, target),
            Some(expected.clone()),
            "original={original:?} target={target:?}"
        );
    }
}

#[test]
fn typing_assist_keeps_live_log_false_positive_words() {
    for input in [
        "бейсовских ",
        "свойств ",
        "окончанием слов ",
        "переиспользуется ",
        "спикок ",
        "лучшить ",
    ] {
        assert_eq!(apply_typing_assist_exact(input), None, "input={input:?}");
    }
}

#[test]
fn typing_assist_keeps_live_log_good_repairs() {
    for (input, expected) in [
        ("смтори ", "смотри "),
        ("спсика ", "списка "),
        ("дургие ", "другие "),
        ("нилинейная ", "нелинейная "),
        ("провблему ", "проблему "),
        ("спаибо ", "спасибо "),
        ("свойсва ", "свойства "),
    ] {
        assert_eq!(
            apply_typing_assist_exact(input),
            Some(expected.to_string()),
            "input={input:?}"
        );
    }
}

#[test]
fn replaces_visual_b_inside_russian_context() {
    for row in fixture_rows("daemon_auto_replace_visual_b.tsv") {
        assert_eq!(row.len(), 3, "visual-b fixture must be TSV");
        assert_eq!(
            apply_auto_replace(&row[0], &row[1]),
            Some(row[2].clone()),
            "original={:?} target={:?}",
            row[0],
            row[1]
        );
    }
    assert_eq!(apply_typing_assist_exact("b "), None);
}

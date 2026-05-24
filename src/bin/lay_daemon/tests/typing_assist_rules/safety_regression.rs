use super::*;

#[test]
fn typing_assist_keeps_valid_russian_words() {
    assert_eq!(apply_typing_assist_exact("проверка "), None);
    assert_eq!(apply_typing_assist_exact("работает "), None);
    assert_eq!(apply_typing_assist_exact("привет "), None);
    assert_eq!(apply_typing_assist_exact("можем "), None);
    assert_eq!(apply_typing_assist_exact("можешь "), None);
    assert_eq!(apply_typing_assist_exact("может "), None);
    assert_eq!(apply_typing_assist_exact("ладно "), None);
    assert_eq!(apply_typing_assist_exact("можно "), None);
    assert_eq!(apply_typing_assist_exact("дальше "), None);
    assert_eq!(apply_typing_assist_exact("плохо "), None);
    assert_eq!(apply_typing_assist_exact("правильно "), None);
    assert_eq!(apply_typing_assist_exact("исправляет "), None);
    assert_eq!(apply_typing_assist_exact("начинаю "), None);
    assert_eq!(apply_typing_assist_exact("удаляется "), None);
    assert_eq!(apply_typing_assist_exact("удателятеся "), None);
    assert_eq!(apply_typing_assist_exact("еще "), None);
    assert_eq!(apply_typing_assist_exact("елка "), None);
    assert_eq!(apply_typing_assist_exact("все "), None);
    assert_eq!(apply_typing_assist_exact("раскладок "), None);
    assert_eq!(apply_typing_assist_exact("кнопок "), None);
    assert_eq!(apply_typing_assist_exact("тестами "), None);
    assert_eq!(apply_typing_assist_exact("словами "), None);
    assert_eq!(apply_typing_assist_exact("вариантами "), None);
    assert_eq!(apply_typing_assist_exact("страдает "), None);
    assert_eq!(apply_typing_assist_exact("установки "), None);
    assert_eq!(apply_typing_assist_exact("изменю "), None);
    for input in fixture_lines("daemon_typing_assist_valid_phrase_keep.txt") {
        assert_eq!(apply_typing_assist_exact(&input), None, "input={input:?}");
    }
    assert_eq!(apply_typing_assist_exact("нужна "), None);
    assert_eq!(apply_typing_assist_exact("важна "), None);
    assert_eq!(apply_typing_assist_exact("важно "), None);
    assert_eq!(apply_typing_assist_exact("банный "), None);
    assert_eq!(apply_typing_assist_exact("бешанный "), None);
    assert_eq!(apply_typing_assist_exact("БЕШАННЫЙ "), None);
    assert_eq!(apply_typing_assist_exact("поения "), None);
    assert_eq!(apply_typing_assist_exact("автозамена "), None);
    assert_eq!(apply_typing_assist_exact("агрессивная "), None);
}

#[test]
fn typing_assist_ignores_words_with_digits() {
    assert_eq!(apply_typing_assist_exact("товара7 "), None);
    assert_eq!(apply_typing_assist_exact("привемр7 "), None);
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
fn replaces_visual_b_inside_russian_context() {
    let cases = fixture_rows("daemon_auto_replace_visual_b.tsv");
    let row = &cases[0];
    assert_eq!(apply_auto_replace(&row[0], &row[1]), Some(row[2].clone()));
    assert_eq!(
        apply_auto_replace("b ghjcnj", "и просто"),
        Some("в просто".to_string())
    );
    let row = &cases[1];
    assert_eq!(apply_auto_replace(&row[0], &row[1]), Some(row[2].clone()));
}

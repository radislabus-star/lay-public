use super::*;

#[test]
fn good_russian_text() {
    // Реальные русские слова
    let s = score("Ну вот пример хорошего текста", "ru");
    assert!(s > 0.8, "score = {s}");
}

#[test]
fn bad_russian_text() {
    // Случайный набор кириллицы (как если бы английский набрали в RU)
    // Эвристика без словаря даёт ~0.5 — этого достаточно для трешхолда 0.7
    let s = score("руддщ цщкдв", "ru");
    assert!(s < 0.7, "score = {s}");
}

#[test]
fn good_english_text() {
    let s = score("hello world this is fine", "en");
    assert!(s > 0.8, "score = {s}");
}

#[test]
fn bad_english_text() {
    // Русский в английской раскладке
    let s = score("Ye djn ghbvth", "en");
    assert!(s < 0.5, "score = {s}");
}

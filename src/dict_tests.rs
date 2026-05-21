use super::*;

#[test]
fn us_to_ru_basic() {
    assert_eq!(convert("Ye djn ghbvth", Direction::Us2Ru), "Ну вот пример");
}

#[test]
fn ru_to_us_basic() {
    assert_eq!(convert("руддщ цщкдв", Direction::Ru2Us), "hello world");
}

#[test]
fn detect() {
    assert_eq!(detect_direction("hello"), Direction::Us2Ru);
    assert_eq!(detect_direction("привет"), Direction::Ru2Us);
    assert_eq!(detect_direction("Ye djn ghbvth"), Direction::Us2Ru);
}

#[test]
fn preserves_unknown_chars() {
    // Цифры, пробелы, спецсимволы остаются
    assert_eq!(convert("hello 123!", Direction::Us2Ru), "руддщ 123!");
}

use super::*;

#[test]
fn recognizes_known_ru_and_en_tokens() {
    assert!(is_known_ru_token("котовые"));
    assert!(is_known_en_token("file"));
    assert!(all_tokens_known("как котовые", Lang::Ru));
    assert!(all_tokens_known("good file", Lang::En));
}

use super::*;

#[test]
fn exposes_insert_text_check() {
    assert!(Correction::InsertText("Double".to_string()).is_insert_text());
    assert!(!Correction::ReplayAll.is_insert_text());
}

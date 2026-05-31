use super::*;

#[test]
fn typing_assist_converts_wrong_layout_ascii_hyphen_token() {
    let technical_events = [
        key_event(KeyCode::KEY_W, true),
        key_event(KeyCode::KEY_I, true),
        key_event(KeyCode::KEY_MINUS, true),
        key_event(KeyCode::KEY_F, true),
        key_event(KeyCode::KEY_I, true),
    ];
    let typed_technical = map_events_to_layout(&technical_events, true);
    let target_technical = map_events_to_layout(&technical_events, false);
    assert_eq!(
        apply_typing_assist_exact(&format!("{typed_technical} ")),
        Some(format!("{target_technical} "))
    );
}

#[test]
fn typing_assist_keeps_natural_cyrillic_hyphen_words() {
    for input in fixture_lines("daemon_typing_assist_natural_hyphen_keep.txt") {
        assert_eq!(apply_typing_assist(&input, true), None, "input={input:?}");
    }
    for row in fixture_rows("daemon_typing_assist_technical_hyphen_token.tsv") {
        assert_eq!(row.len(), 2, "technical hyphen token fixture must be TSV");
        let expected = if row[1] == "None" {
            None
        } else {
            Some(row[1].clone())
        };
        assert_eq!(
            correct_wrong_layout_ascii_technical_token(&row[0]),
            expected,
            "token={:?}",
            row[0]
        );
    }
}

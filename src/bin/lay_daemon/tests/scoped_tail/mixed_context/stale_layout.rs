use super::*;

#[test]
fn scoped_tail_keeps_completed_russian_y_word_and_flips_current_latin_brand() {
    let mut buffer = WordBuffer::new();
    push_text_as_layout(&mut buffer, "протокол", true);
    buffer.handle_space();
    push_text_as_layout(&mut buffer, "испытаний", true);
    buffer.handle_space();
    push_text_as_layout(&mut buffer, "Сщсф", true);
    let (events, _) = buffer.what_to_replay(3).expect("three-word tail");
    let original = map_original_events(&events);
    let replacement = decide_scoped_tail_correction(&events).expect("smart replacement");

    assert_eq!(original, "протокол испытаний Сщсф");
    assert_eq!(replacement, "протокол испытаний Coca");
    assert_eq!(
        plan_text_replacement(&original, &replacement),
        Some(TextReplacement {
            move_left: 0,
            backspaces: 4,
            insert: "Coca".to_string(),
            move_right: 0,
        })
    );
}

#[test]
fn scoped_tail_repairs_stale_layout_flag_inside_completed_russian_word() {
    let mut buffer = WordBuffer::new();
    push_text_as_layout(&mut buffer, "протокол", true);
    buffer.handle_space();
    push_text_as_layout(&mut buffer, "испытани", true);
    buffer.push(KeyEvent {
        keycode: KeyCode::KEY_Q.code(),
        shift: false,
        layout_is_ru: false,
    });
    buffer.handle_space();
    push_text_as_layout(&mut buffer, "Сщсф", true);
    let (events, _) = buffer.what_to_replay(3).expect("three-word tail");
    let original = map_original_events(&events);
    let replacement = decide_scoped_tail_correction(&events).expect("smart replacement");

    assert_eq!(original, "протокол испытаниq Сщсф");
    assert_eq!(replacement, "протокол испытаний Coca");
    assert_eq!(
        plan_text_replacement(&original, &replacement),
        Some(TextReplacement {
            move_left: 0,
            backspaces: 6,
            insert: "й Coca".to_string(),
            move_right: 0,
        })
    );
}

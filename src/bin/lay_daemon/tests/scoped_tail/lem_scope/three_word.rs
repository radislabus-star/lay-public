use super::*;

#[test]
fn scoped_tail_generalizes_to_more_than_two_words() {
    let mut buffer = WordBuffer::new();
    push_keys(
        &mut buffer,
        &[
            KeyCode::KEY_G,
            KeyCode::KEY_H,
            KeyCode::KEY_J,
            KeyCode::KEY_D,
            KeyCode::KEY_T,
            KeyCode::KEY_H,
            KeyCode::KEY_R,
            KeyCode::KEY_F,
        ],
        true,
    );
    buffer.handle_space();
    push_keys(&mut buffer, &[KeyCode::KEY_D], true);
    buffer.handle_space();
    push_key_events(
        &mut buffer,
        &[
            (KeyCode::KEY_D, true),
            (KeyCode::KEY_O, false),
            (KeyCode::KEY_U, false),
            (KeyCode::KEY_B, false),
            (KeyCode::KEY_L, false),
            (KeyCode::KEY_E, false),
        ],
        true,
    );
    let (events, _) = buffer.what_to_replay(3).expect("three-word tail");

    assert_eq!(map_original_events(&events), "проверка в Вщгиду");
    assert_eq!(
        decide_scoped_tail_correction(&events),
        Some("проверка в Double".to_string())
    );
}

#[test]
fn scoped_tail_uses_lem_for_three_word_mixed_tail() {
    let mut buffer = WordBuffer::new();
    push_text_as_layout(&mut buffer, "good", false);
    buffer.handle_space();
    for ch in "ghbdtn".chars() {
        buffer.push(text_key_event(ch, false));
    }
    buffer.handle_space();
    for ch in "ntrcn".chars() {
        buffer.push(text_key_event(ch, false));
    }

    let (events, _) = buffer.what_to_replay(3).expect("three-word tail");
    assert_eq!(map_original_events(&events), "good ghbdtn ntrcn");
    assert_eq!(
        decide_scoped_tail_correction(&events),
        Some("good привет текст".to_string())
    );
}

#[test]
fn scoped_tail_keeps_two_russian_words_and_flips_current_english_layout_tail() {
    let mut buffer = WordBuffer::new();
    push_text_as_layout(&mut buffer, "как", true);
    buffer.handle_space();
    push_text_as_layout(&mut buffer, "котовые", true);
    buffer.handle_space();
    push_text_as_layout(&mut buffer, "ашду", true);

    let (events, _) = buffer.what_to_replay(3).expect("three-word tail");
    let original = map_original_events(&events);
    let words = split_event_words(&events).expect("split words");
    let candidates = scoped_tail_lem_candidates(&words, true, true);
    let replacement =
        decide_scoped_tail_correction_with_lem(&events, true).expect("smart replacement");

    assert_eq!(original, "как котовые ашду");
    assert!(candidates
        .iter()
        .any(|candidate| candidate == "как котовые file"));
    assert!(
        !candidates
            .iter()
            .any(|candidate| candidate.contains("кротовые")),
        "stable completed Russian context must not be typo-corrected: {candidates:?}"
    );
    assert_eq!(replacement, "как котовые file");
    assert_eq!(
        plan_text_replacement(&original, &replacement),
        Some(TextReplacement {
            move_left: 0,
            backspaces: 4,
            insert: "file".to_string(),
            move_right: 0,
        })
    );
}

#[test]
fn scoped_tail_keeps_short_repeated_completed_word_and_flips_current_tail() {
    let mut buffer = WordBuffer::new();
    push_text_as_layout(&mut buffer, "аа", true);
    buffer.handle_space();
    push_text_as_layout(&mut buffer, "слово", true);
    buffer.handle_space();
    push_text_as_layout(&mut buffer, "вот", true);

    let (events, _) = buffer.what_to_replay(3).expect("three-word tail");
    let original = map_original_events(&events);
    let replacement =
        decide_scoped_tail_correction_with_lem(&events, true).expect("smart replacement");

    assert_eq!(original, "аа слово вот");
    assert_eq!(replacement, "аа слово djn");
    assert_eq!(
        plan_text_replacement(&original, &replacement),
        Some(TextReplacement {
            move_left: 0,
            backspaces: 3,
            insert: "djn".to_string(),
            move_right: 0,
        })
    );
}

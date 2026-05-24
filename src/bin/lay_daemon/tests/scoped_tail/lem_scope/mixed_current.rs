use super::*;

#[test]
fn scoped_tail_handles_three_completed_words_with_typo() {
    let mut buffer = WordBuffer::new();
    push_text_as_layout(&mut buffer, "ljgecntv", false);
    buffer.handle_space();
    push_text_as_layout(&mut buffer, ",ele", false);
    buffer.handle_space();
    push_text_as_layout(&mut buffer, "ошибатся", true);
    buffer.handle_space();

    let scope = effective_replace_words(&buffer, 3, CorrectionEngine::Smart, true);
    let (events, _) = buffer.what_to_replay(scope).expect("three-word tail");

    assert_eq!(scope, 3);
    assert_eq!(map_original_events(&events), "ljgecntv ,ele ошибатся ");
    assert_eq!(
        decide_scoped_tail_correction_with_lem(&events, true),
        Some("допустем буду ошибаться ".to_string())
    );
}

#[test]
fn scoped_tail_keeps_live_and_flips_russian_current_tail() {
    let mut buffer = WordBuffer::new();
    push_key_events(
        &mut buffer,
        &[
            (KeyCode::KEY_L, true),
            (KeyCode::KEY_I, false),
            (KeyCode::KEY_V, false),
            (KeyCode::KEY_E, false),
        ],
        false,
    );
    buffer.handle_space();
    push_keys(
        &mut buffer,
        &[
            KeyCode::KEY_L,
            KeyCode::KEY_B,
            KeyCode::KEY_C,
            KeyCode::KEY_N,
            KeyCode::KEY_H,
            KeyCode::KEY_B,
        ],
        false,
    );
    let (events, _) = buffer.what_to_replay(2).expect("two-word tail");

    assert_eq!(map_original_events(&events), "Live lbcnhb");
    assert_eq!(
        decide_scoped_tail_correction(&events),
        Some("Live дистри".to_string())
    );
}

#[test]
fn scoped_tail_normalizes_mixed_current_word_to_last_layout() {
    let mut buffer = WordBuffer::new();
    push_key_events(
        &mut buffer,
        &[
            (KeyCode::KEY_L, true),
            (KeyCode::KEY_I, false),
            (KeyCode::KEY_V, false),
            (KeyCode::KEY_E, false),
        ],
        false,
    );
    buffer.handle_space();
    push_keys(&mut buffer, &[KeyCode::KEY_L], false);
    push_keys(
        &mut buffer,
        &[KeyCode::KEY_L, KeyCode::KEY_B, KeyCode::KEY_C],
        true,
    );
    let (events, _) = buffer.what_to_replay(2).expect("two-word tail");

    assert_eq!(map_original_events(&events), "Live lдис");
    assert_eq!(
        decide_scoped_tail_correction(&events),
        Some("Live дис".to_string())
    );
}

#[test]
fn scoped_tail_repairs_mixed_previous_ru_word_and_flips_current_tail() {
    let mut buffer = WordBuffer::new();
    push_key_events(
        &mut buffer,
        &[
            (KeyCode::KEY_G, true),
            (KeyCode::KEY_H, true),
            (KeyCode::KEY_J, true),
            (KeyCode::KEY_D, true),
        ],
        true,
    );
    push_key_events(
        &mut buffer,
        &[
            (KeyCode::KEY_T, true),
            (KeyCode::KEY_H, true),
            (KeyCode::KEY_M, true),
        ],
        false,
    );
    buffer.handle_space();
    push_key_events(
        &mut buffer,
        &[
            (KeyCode::KEY_W, true),
            (KeyCode::KEY_O, true),
            (KeyCode::KEY_R, true),
            (KeyCode::KEY_D, true),
        ],
        true,
    );
    let (events, _) = buffer.what_to_replay(2).expect("two-word tail");

    assert_eq!(map_original_events(&events), "ПРОВTHM ЦЩКВ");
    assert_eq!(
        decide_scoped_tail_correction(&events),
        Some("ПРОВЕРЬ WORD".to_string())
    );
}

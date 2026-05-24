use super::*;

#[test]
fn plain_cyrillic_scope_word_does_not_become_ascii_technical_noise() {
    let events = [
        key_event(KeyCode::KEY_A, true),
        key_event(KeyCode::KEY_Q, true),
        key_event(KeyCode::KEY_DOT, true),
        key_event(KeyCode::KEY_Z, true),
    ];
    let original = map_events_to_layout(&events, true);
    let converted = map_events_to_layout(&events, false);

    assert!(original.chars().all(is_cyrillic_letter));
    assert!(is_ascii_technical_token(&converted));
    assert!(should_keep_plain_cyrillic_before_ascii_technical(
        &original, &converted
    ));
    assert_eq!(decide_completed_scope_word(&events), original);
}

#[test]
fn smart_scoped_tail_handles_large_mixed_language_pair_matrix() {
    let english_left = [
        "good", "test", "word", "live", "double", "text", "mode", "file", "code", "data",
    ];
    let russian_left = [
        "привет",
        "текст",
        "слово",
        "тест",
        "проверка",
        "можно",
        "нужно",
        "дальше",
        "хорошо",
        "пример",
    ];
    let russian_targets = [
        "привет",
        "текст",
        "слово",
        "тест",
        "проверка",
        "можно",
        "нужно",
        "дальше",
        "хорошо",
        "пример",
    ];
    let english_targets = [
        "good", "test", "word", "live", "double", "text", "mode", "file", "code", "data",
    ];

    let mut cases = 0;
    for left in english_left {
        for target in russian_targets {
            let typed = lay::dict::convert(target, lay::dict::Direction::Ru2Us);
            assert_smart_pair(left, false, &typed, false, &format!("{left} {target}"));
            cases += 1;
        }
    }

    for left in russian_left {
        for target in english_targets {
            let typed = lay::dict::convert(target, lay::dict::Direction::Us2Ru);
            assert_smart_pair(left, true, &typed, true, &format!("{left} {target}"));
            cases += 1;
        }
    }

    assert!(cases >= 100, "expected at least 100 mixed pair cases");
}

#[test]
fn scoped_tail_flips_current_visual_latin_word_with_cyrillic_c_homoglyph() {
    let mut buffer = WordBuffer::new();
    push_key_events(
        &mut buffer,
        &[
            (KeyCode::KEY_C, false),
            (KeyCode::KEY_H, false),
            (KeyCode::KEY_E, false),
            (KeyCode::KEY_C, false),
            (KeyCode::KEY_K, false),
        ],
        false,
    );
    buffer.handle_space();
    buffer.push(key_event(KeyCode::KEY_C, true));
    buffer.push(key_event(KeyCode::KEY_H, false));
    buffer.push(key_event(KeyCode::KEY_E, false));
    buffer.push(key_event(KeyCode::KEY_C, false));
    let (events, _) = buffer.what_to_replay(2).expect("two-word tail");

    assert_eq!(map_original_events(&events), "check сhec");
    assert_eq!(
        decide_scoped_tail_correction(&events),
        Some("check срус".to_string())
    );
}

#[test]
fn scoped_tail_removes_duplicate_layout_prefix_from_completed_ascii_technical_token() {
    let mut buffer = WordBuffer::new();
    let mut completed_events = vec![key_event(KeyCode::KEY_W, true)];
    completed_events.extend(key_events(&ascii_hyphen_token_keycodes(), false));
    for event in &completed_events {
        buffer.push(*event);
    }
    buffer.handle_space();
    let current_events = key_events(&[KeyCode::KEY_G, KeyCode::KEY_H, KeyCode::KEY_J], false);
    for event in &current_events {
        buffer.push(*event);
    }
    let (events, _) = buffer.what_to_replay(2).expect("two-word tail");
    let completed_original = map_original_events(&completed_events);
    let current_original = map_original_events(&current_events);
    let completed_repaired = correct_duplicate_layout_prefix_on_ascii_token(&completed_original)
        .expect("duplicate prefix repair");
    let current_target = map_events_to_layout(&current_events, true);

    assert_eq!(
        map_original_events(&events),
        format!("{completed_original} {current_original}")
    );
    assert_eq!(
        decide_scoped_tail_correction(&events),
        Some(format!("{completed_repaired} {current_target}"))
    );
}

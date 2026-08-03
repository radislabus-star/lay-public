use super::decode_completed_tail;
use lay::keyboard::{text_to_key_events, KeyEvent};
use lay::text_edit::apply_replacement_plan_to_text;
use lay::word_buffer::WordBuffer;
use std::time::{Duration, Instant};

fn push_text_as_layout(buffer: &mut WordBuffer, text: &str, layout_is_ru: bool) {
    for event in text_events(text, layout_is_ru) {
        if lay::keyboard::original_event_char(&event) == Some(' ') {
            buffer.handle_space();
        } else {
            buffer.push(event);
        }
    }
}

fn text_events(text: &str, layout_is_ru: bool) -> Vec<KeyEvent> {
    text_to_key_events(text, layout_is_ru).expect("text must map to key events")
}

#[test]
fn input_gate_context_tail_keeps_left_space_anchor_for_longer_word_fix() {
    let mut buffer = WordBuffer::new();
    push_text_as_layout(&mut buffer, "я прохоил ", true);
    let events = buffer
        .last_completed_words_events(1)
        .expect("last completed word");

    let decoded = decode_completed_tail(&buffer, 1, &events, true).expect("decoded");

    assert_eq!(decoded.edit.original, " прохоил ");
    assert_eq!(decoded.edit.replacement, " проходил ");
    assert_eq!(
        apply_replacement_plan_to_text(&decoded.edit.original, &decoded.edit.plan),
        decoded.edit.replacement
    );
}

#[test]
fn input_gate_prefers_effective_for_missing_initial_vowel_tail() {
    let mut buffer = WordBuffer::new();
    push_text_as_layout(&mut buffer, "на сколько ффективная ", true);
    let events = buffer
        .last_completed_words_events(1)
        .expect("last completed word");

    let decoded = decode_completed_tail(&buffer, 1, &events, true).expect("decoded");

    assert_eq!(decoded.edit.original, " ффективная ");
    assert_eq!(decoded.edit.replacement, " эффективная ");
    assert_eq!(
        apply_replacement_plan_to_text(&decoded.edit.original, &decoded.edit.plan),
        decoded.edit.replacement
    );
}

#[test]
fn input_gate_projects_unknown_cyrillic_surface_to_known_english_center() {
    let mut buffer = WordBuffer::new();
    push_text_as_layout(&mut buffer, "дфн ", true);
    let events = buffer
        .last_completed_words_events(1)
        .expect("last completed word");

    let decoded = decode_completed_tail(&buffer, 1, &events, true).expect("decoded");

    assert_eq!(decoded.edit.original, "дфн ");
    assert_eq!(decoded.edit.replacement, "lay ");
}

#[test]
fn input_gate_preserves_known_ascii_token_typed_on_active_english_layout() {
    let mut buffer = WordBuffer::new();
    push_text_as_layout(&mut buffer, "pdf ", false);
    let events = buffer
        .last_completed_words_events(1)
        .expect("last completed word");

    assert!(decode_completed_tail(&buffer, 1, &events, true).is_none());
}

#[test]
fn input_gate_allows_unknown_cyrillic_token_to_project_from_active_russian_layout() {
    let mut buffer = WordBuffer::new();
    push_text_as_layout(&mut buffer, "зва ", true);
    let events = buffer
        .last_completed_words_events(1)
        .expect("last completed word");

    let decoded = decode_completed_tail(&buffer, 1, &events, true).expect("decoded");
    assert_eq!(decoded.edit.replacement, "pdf ");
}

#[test]
fn startup_warmup_removes_first_boundary_decision_stall() {
    lay::typing_assist::warm_up();
    let mut buffer = WordBuffer::new();
    push_text_as_layout(&mut buffer, "дфн ", true);
    let events = buffer
        .last_completed_words_events(1)
        .expect("last completed word");
    let started = Instant::now();

    let decoded = decode_completed_tail(&buffer, 1, &events, true).expect("decoded");
    let elapsed = started.elapsed();

    assert_eq!(decoded.edit.replacement, "lay ");
    if !cfg!(debug_assertions) {
        assert!(
            elapsed < Duration::from_millis(250),
            "warmed boundary decision took {elapsed:?}"
        );
    }
}

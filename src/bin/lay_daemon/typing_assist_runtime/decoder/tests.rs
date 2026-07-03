use super::decode_completed_tail;
use lay::keyboard::{text_to_key_events, KeyEvent};
use lay::text_edit::apply_replacement_plan_to_text;
use lay::word_buffer::WordBuffer;

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

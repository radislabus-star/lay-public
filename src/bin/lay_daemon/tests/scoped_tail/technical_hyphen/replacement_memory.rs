use super::*;

#[test]
fn replacement_last_word_memory_ignores_middle_insert_plan() {
    let mut buffer = WordBuffer::new();
    push_text_as_layout(&mut buffer, "AmoCRM", false);
    buffer.handle_space();
    push_text_as_layout(&mut buffer, "Z", false);
    buffer.handle_space();
    push_text_as_layout(&mut buffer, "тут", true);
    buffer.handle_space();
    push_text_as_layout(&mut buffer, "задача", true);

    let (events, _) = buffer.what_to_replay(4).expect("four-word tail");
    let plan = plan_text_replacement("AmoCRM Z тут задача", "AmoCRM Я тут задача")
        .expect("middle replacement plan");

    assert_eq!(plan.move_right, 11);
    assert!(!buffer.remember_replacement_last_word_for_replay(
        &events,
        &plan,
        "AmoCRM Я тут задача"
    ));
}

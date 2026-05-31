use super::*;

#[test]
fn replacement_last_word_memory_ignores_middle_insert_plan() {
    let (mut buffer, events, _) = typed_tail(
        &[("AmoCRM Z ", false), ("тут задача", true)],
        4,
        "four-word tail",
    );
    let plan = plan_text_replacement("AmoCRM Z тут задача", "AmoCRM Я тут задача")
        .expect("middle replacement plan");

    assert_eq!(plan.move_right, 11);
    assert!(!buffer.remember_replacement_last_word_for_replay(
        &events,
        &plan,
        "AmoCRM Я тут задача"
    ));
}

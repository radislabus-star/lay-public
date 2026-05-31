use super::*;

#[test]
fn enter_autocorrect_candidate_is_off_contract_until_enabled_by_config() {
    let cfg = LayConfig::default();
    assert!(!cfg.enter_autocorrect);
    assert!(!active_enter_autocorrect_from_env(false, None));
    assert!(active_enter_autocorrect_from_env(true, None));
    assert!(!active_enter_autocorrect_from_env(true, Some("0")));
    assert!(active_enter_autocorrect_from_env(true, Some("1")));
    assert!(active_enter_autocorrect_from_env(true, Some("true")));
}

#[test]
fn enter_autocorrect_candidate_rejects_plain_layout_word_guess() {
    let pipeline = typing_pipeline_with_only("layout_en_to_ru");

    for input in ["ghbdtn", "lfkmit"] {
        let buffer = typed_buffer(&[(input, false)]);

        assert!(
            enter_autocorrect_candidate(&buffer, 1, true, &pipeline).is_none(),
            "plain layout words are not safe enough for Enter autocorrect: {input}"
        );
    }
}

#[test]
fn enter_autocorrect_candidate_keeps_normal_english_word() {
    let buffer = typed_buffer(&[("good", false)]);
    let pipeline = typing_pipeline_with_only("layout_en_to_ru");

    assert!(enter_autocorrect_candidate(&buffer, 1, true, &pipeline).is_none());
}

#[test]
fn enter_autocorrect_candidate_can_use_completed_tail_scope() {
    let buffer = typed_buffer(&[("double b", false)]);
    let pipeline = typing_pipeline_with_only("visual_b");

    let (_events, edit) =
        enter_autocorrect_candidate(&buffer, 2, true, &pipeline).expect("correction");

    assert_eq!(edit.original, "double b");
    assert_eq!(edit.replacement, "double и");
}

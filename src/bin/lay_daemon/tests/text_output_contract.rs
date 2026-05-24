use super::*;

#[test]
fn text_insert_runs_use_uinput_layout_channels() {
    for row in fixture_rows("daemon_text_insert_runs.tsv") {
        assert_eq!(row.len(), 4, "text insert fixture must be TSV");
        let default_layout_is_ru = row[1] == "ru";
        if row[2] == "none" {
            assert!(text_to_uinput_runs(&row[0], default_layout_is_ru).is_none());
            continue;
        }

        let expected_targets: Vec<bool> = row[2].split(',').map(|part| part == "ru").collect();
        let expected_outputs: Vec<&str> = row[3].split('|').collect();
        let runs = text_to_uinput_runs(&row[0], default_layout_is_ru).expect("typable text");
        assert_eq!(runs.len(), expected_targets.len());
        assert_eq!(runs.len(), expected_outputs.len());
        for (idx, run) in runs.iter().enumerate() {
            assert_eq!(run.target_is_ru, expected_targets[idx], "row={row:?}");
            assert_eq!(
                map_events_to_layout(&run.events, run.target_is_ru),
                expected_outputs[idx],
                "row={row:?}"
            );
        }
    }
}

#[test]
fn typing_assist_minimal_plan_keeps_inter_word_space() {
    let row = fixture_rows("daemon_typing_assist_minimal_plan.tsv")
        .into_iter()
        .next()
        .expect("minimal plan fixture");
    let plan = plan_text_replacement(&row[0], &row[1]).expect("replacement");

    assert_eq!(plan.move_left, 1);
    assert_eq!(plan.backspaces, 1);
    assert_eq!(plan.insert, "о");
    assert_eq!(plan.move_right, 1);
}

#[test]
fn replacement_memory_keeps_space_boundary_after_i_autofix() {
    let mut buffer = WordBuffer::new();
    push_text_as_layout(&mut buffer, "double b ", false);
    let events = buffer
        .last_completed_words_events(2)
        .expect("completed two-word tail");
    let original = map_original_events(&events);
    let replacement = "double и ";
    let plan = plan_committed_tail_replacement(&original, replacement).expect("replacement");

    assert!(buffer.remember_replacement_last_word_for_replay(&events, &plan, replacement));
    assert!(buffer.current_is_empty());
    assert!(buffer.prev_had_trailing_space());
    assert_eq!(buffer.prev_words_len(), 1);
    assert_eq!(
        map_original_events(buffer.prev_word_events(0).expect("prev word")),
        "и"
    );

    push_text_as_layout(&mut buffer, "слово", true);
    let (tail, _) = buffer.what_to_replay(2).expect("two-word tail");

    assert_eq!(map_original_events(&tail), "и слово");
}

#[test]
fn replacement_memory_synthesizes_last_word_after_glued_phrase_split() {
    let mut buffer = WordBuffer::new();
    let row = fixture_rows("daemon_replacement_memory_glued.tsv")
        .into_iter()
        .next()
        .expect("replacement memory fixture");
    push_text_as_layout(&mut buffer, &row[0], true);
    let events = buffer
        .last_completed_words_events(1)
        .expect("completed one-word tail");
    let original = map_original_events(&events);
    let replacement = &row[1];
    let plan = plan_committed_tail_replacement(&original, replacement).expect("replacement");

    assert_eq!(original, row[0]);
    assert_eq!(
        plan,
        TextReplacement {
            move_left: 6,
            backspaces: 0,
            insert: " ".to_string(),
            move_right: 6,
        }
    );
    assert!(buffer.remember_replacement_last_word_for_replay(&events, &plan, replacement));
    assert!(buffer.current_is_empty());
    assert!(buffer.prev_had_trailing_space());
    assert_eq!(buffer.prev_words_len(), 2);
    assert_eq!(
        map_original_events(buffer.prev_word_events(0).expect("first prev word")),
        row[2]
    );
    assert_eq!(
        map_original_events(buffer.prev_word_events(1).expect("second prev word")),
        row[3]
    );

    push_text_as_layout(&mut buffer, &row[4], true);
    let (tail, _) = buffer.what_to_replay(2).expect("two-word tail");
    assert_eq!(map_original_events(&tail), row[5]);
}

#[test]
fn replacement_memory_can_update_completed_words_without_dropping_current_word() {
    let mut buffer = WordBuffer::new();
    let row = fixture_rows("daemon_replacement_memory_completed.tsv")
        .into_iter()
        .next()
        .expect("replacement completed fixture");
    push_text_as_layout(&mut buffer, &row[0], true);
    push_text_as_layout(&mut buffer, &row[4], true);

    assert!(buffer.remember_completed_replacement_words_for_replay(&row[1]));
    assert_eq!(buffer.prev_words_len(), 2);
    assert!(buffer.prev_had_trailing_space());
    assert_eq!(
        map_original_events(buffer.prev_word_events(0).expect("first prev")),
        row[2]
    );
    assert_eq!(
        map_original_events(buffer.prev_word_events(1).expect("second prev")),
        row[3]
    );
    assert_eq!(buffer.current_len(), 6);

    let (tail, _) = buffer.what_to_replay(1).expect("current word tail");
    assert_eq!(map_original_events(&tail), row[4]);
}

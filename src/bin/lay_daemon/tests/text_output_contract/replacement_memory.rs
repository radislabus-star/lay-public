use super::super::*;

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
    assert_eq!(buffer.prev_words_len(), 2);
    assert_eq!(
        map_original_events(buffer.prev_word_events(0).expect("left context")),
        "double"
    );
    assert_eq!(
        map_original_events(buffer.prev_word_events(1).expect("prev word")),
        "и"
    );

    push_text_as_layout(&mut buffer, "слово", true);
    let (tail, _) = buffer.what_to_replay(2).expect("two-word tail");

    assert_eq!(map_original_events(&tail), "и слово");
}

#[test]
fn replacement_memory_stays_synced_after_html_autofix_and_next_word() {
    let mut buffer = WordBuffer::new();
    push_text_as_layout(&mut buffer, "реьд ", true);

    let html_events = buffer
        .last_completed_words_events(1)
        .expect("completed html source");
    let html_original = map_original_events(&html_events);
    let html_replacement = "html ";
    let html_plan =
        plan_committed_tail_replacement(&html_original, html_replacement).expect("html correction");

    assert_eq!(html_original, "реьд ");
    assert_eq!(
        html_plan,
        TextReplacement {
            move_left: 1,
            backspaces: 4,
            insert: "html".to_string(),
            move_right: 1,
        }
    );
    assert!(buffer.remember_replacement_last_word_for_replay(
        &html_events,
        &html_plan,
        html_replacement
    ));
    assert!(buffer.current_is_empty());
    assert!(buffer.prev_had_trailing_space());
    assert_eq!(
        map_original_events(buffer.prev_word_events(0).expect("html prev")),
        "html"
    );

    push_text_as_layout(&mut buffer, "djn", false);
    let (tail, _) = buffer.what_to_replay(2).expect("html + current");
    assert_eq!(map_original_events(&tail), "html djn");

    buffer.handle_space();
    let djn_events = buffer
        .last_completed_words_events(1)
        .expect("completed next word");
    let djn_original = map_original_events(&djn_events);
    let djn_replacement = "вот ";
    let djn_plan =
        plan_committed_tail_replacement(&djn_original, djn_replacement).expect("вот correction");

    assert_eq!(djn_original, "djn ");
    assert!(buffer.remember_replacement_last_word_for_replay(
        &djn_events,
        &djn_plan,
        djn_replacement
    ));
    assert_eq!(buffer.prev_words_len(), 2);
    assert!(buffer.prev_had_trailing_space());
    assert_eq!(
        map_original_events(buffer.prev_word_events(0).expect("html remains")),
        "html"
    );
    assert_eq!(
        map_original_events(buffer.prev_word_events(1).expect("вот appended")),
        "вот"
    );
    let completed_tail = buffer
        .last_completed_words_events(2)
        .expect("completed html + вот tail");
    assert_eq!(map_original_events(&completed_tail), "html вот ");
}

#[test]
fn replacement_memory_preserves_current_after_deferred_completed_tail() {
    let mut buffer = WordBuffer::new();
    push_text_as_layout(&mut buffer, "html", false);
    buffer.handle_space();
    push_text_as_layout(&mut buffer, "djn", false);
    buffer.handle_space();
    push_text_as_layout(&mut buffer, "x", false);

    let djn_events = buffer
        .last_completed_words_events(1)
        .expect("completed word behind current");
    let djn_original = map_original_events(&djn_events);
    let djn_replacement = "вот ";
    let djn_plan =
        plan_committed_tail_replacement(&djn_original, djn_replacement).expect("вот correction");

    assert_eq!(djn_original, "djn ");
    assert!(buffer.remember_visible_replacement_tail_for_replay(&djn_events, djn_replacement));
    assert_eq!(buffer.prev_words_len(), 2);
    assert!(!buffer.prev_had_trailing_space());
    assert_eq!(
        map_original_events(buffer.prev_word_events(0).expect("html remains")),
        "html"
    );
    assert_eq!(
        map_original_events(buffer.prev_word_events(1).expect("вот appended")),
        "вот"
    );
    assert_eq!(buffer.current_len(), 1);

    let completed_tail = buffer
        .last_completed_words_events(1)
        .expect("completed corrected word remains readable");
    assert_eq!(map_original_events(&completed_tail), "вот ");
    let (tail, _) = buffer.what_to_replay(1).expect("current stays active");
    assert_eq!(map_original_events(&tail), "x");
    assert_eq!(
        djn_plan,
        TextReplacement {
            move_left: 1,
            backspaces: 3,
            insert: "вот".to_string(),
            move_right: 1,
        }
    );
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
        lay::text_edit::apply_replacement_plan_to_text(&original, &plan),
        *replacement
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

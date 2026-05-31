use super::super::*;

struct ReplacementMemorySequence {
    initial: String,
    scope: usize,
    original: String,
    replacement: String,
    plan: TextReplacement,
    prev_len: usize,
    prev_space: bool,
    prev0: String,
    prev1: String,
    next_input: String,
    tail: String,
}

impl ReplacementMemorySequence {
    fn load(id: &str) -> Self {
        let row = fixture_row_by_id("daemon_replacement_memory_sequences.tsv", id);
        Self::from_row(id, row)
    }

    fn from_row(id: &str, row: Vec<String>) -> Self {
        assert_eq!(
            row.len(),
            15,
            "bad replacement-memory fixture row {id:?}: {row:?}"
        );
        Self {
            initial: row[1].clone(),
            scope: row[2].parse().expect("scope"),
            original: row[3].clone(),
            replacement: row[4].clone(),
            plan: text_replacement_from_fixture(&row, 5, 6, 7, 8),
            prev_len: row[9].parse().expect("prev_len"),
            prev_space: row[10].parse().expect("prev_space"),
            prev0: row[11].clone(),
            prev1: row[12].clone(),
            next_input: row[13].clone(),
            tail: row[14].clone(),
        }
    }

    fn typed_buffer(&self, layout_is_ru: bool) -> WordBuffer {
        typed_buffer(&[(&self.initial, layout_is_ru)])
    }

    fn original_without_trailing_space(&self) -> &str {
        self.original.trim_end()
    }
}

fn replacement_memory_sequence(id: &str) -> ReplacementMemorySequence {
    ReplacementMemorySequence::load(id)
}

#[test]
fn replacement_memory_keeps_space_boundary_after_i_autofix() {
    let case = replacement_memory_sequence("visual_b_space");
    let mut buffer = case.typed_buffer(false);
    let events = buffer
        .last_completed_words_events(case.scope)
        .expect("completed two-word tail");
    let original = map_original_events(&events);
    let plan = plan_committed_tail_replacement(&original, &case.replacement).expect("replacement");

    assert_eq!(original, case.original);
    assert_eq!(plan, case.plan);
    assert!(buffer.remember_replacement_last_word_for_replay(&events, &plan, &case.replacement));
    assert!(buffer.current_is_empty());
    assert_eq!(buffer.prev_had_trailing_space(), case.prev_space);
    assert_eq!(buffer.prev_words_len(), case.prev_len);
    assert_eq!(
        map_original_events(buffer.prev_word_events(0).expect("left context")),
        case.prev0
    );
    assert_eq!(
        map_original_events(buffer.prev_word_events(1).expect("prev word")),
        case.prev1
    );

    push_text_as_layout(&mut buffer, &case.next_input, true);
    let (tail, _) = buffer.what_to_replay(2).expect("two-word tail");

    assert_eq!(map_original_events(&tail), case.tail);
}

#[test]
fn replacement_memory_stays_synced_after_html_autofix_and_next_word() {
    let first = replacement_memory_sequence("html_first");
    let second = replacement_memory_sequence("html_second");
    let mut buffer = first.typed_buffer(true);

    let html_events = buffer
        .last_completed_words_events(first.scope)
        .expect("completed html source");
    let html_original = map_original_events(&html_events);
    let html_plan = plan_committed_tail_replacement(&html_original, &first.replacement)
        .expect("first correction");

    assert_eq!(html_original, first.original);
    assert_eq!(html_plan, first.plan);
    assert!(buffer.remember_replacement_last_word_for_replay(
        &html_events,
        &html_plan,
        &first.replacement
    ));
    assert!(buffer.current_is_empty());
    assert_eq!(buffer.prev_had_trailing_space(), first.prev_space);
    assert_eq!(
        map_original_events(buffer.prev_word_events(0).expect("html prev")),
        first.prev0
    );

    let second_input = second.original_without_trailing_space();
    push_text_as_layout(&mut buffer, second_input, false);
    let (tail, _) = buffer.what_to_replay(2).expect("html + current");
    assert_eq!(
        map_original_events(&tail),
        format!("{} {}", first.prev0, second_input)
    );

    buffer.handle_space();
    let djn_events = buffer
        .last_completed_words_events(second.scope)
        .expect("completed next word");
    let djn_original = map_original_events(&djn_events);
    let djn_plan = plan_committed_tail_replacement(&djn_original, &second.replacement)
        .expect("second correction");

    assert_eq!(djn_original, second.original);
    assert_eq!(djn_plan, second.plan);
    assert!(buffer.remember_replacement_last_word_for_replay(
        &djn_events,
        &djn_plan,
        &second.replacement
    ));
    assert_eq!(buffer.prev_words_len(), second.prev_len);
    assert_eq!(buffer.prev_had_trailing_space(), second.prev_space);
    assert_eq!(
        map_original_events(buffer.prev_word_events(0).expect("html remains")),
        second.prev0
    );
    assert_eq!(
        map_original_events(buffer.prev_word_events(1).expect("second prev word")),
        second.prev1
    );
    let completed_tail = buffer
        .last_completed_words_events(2)
        .expect("completed html + second tail");
    assert_eq!(map_original_events(&completed_tail), second.tail);
}

#[test]
fn replacement_memory_preserves_current_after_deferred_completed_tail() {
    let case = replacement_memory_sequence("deferred_djn");
    let mut buffer = case.typed_buffer(false);

    let djn_events = buffer
        .last_completed_words_events(case.scope)
        .expect("completed word behind current");
    let djn_original = map_original_events(&djn_events);
    let djn_plan =
        plan_committed_tail_replacement(&djn_original, &case.replacement).expect("correction");

    assert_eq!(djn_original, case.original);
    assert_eq!(djn_plan, case.plan);
    assert!(buffer.remember_visible_replacement_tail_for_replay(&djn_events, &case.replacement));
    assert_eq!(buffer.prev_words_len(), case.prev_len);
    assert_eq!(buffer.prev_had_trailing_space(), case.prev_space);
    assert_eq!(
        map_original_events(buffer.prev_word_events(0).expect("html remains")),
        case.prev0
    );
    assert_eq!(
        map_original_events(buffer.prev_word_events(1).expect("second prev word")),
        case.prev1
    );
    assert_eq!(buffer.current_len(), case.next_input.chars().count());

    let completed_tail = buffer
        .last_completed_words_events(1)
        .expect("completed corrected word remains readable");
    assert_eq!(map_original_events(&completed_tail), case.tail);
    let (tail, _) = buffer.what_to_replay(1).expect("current stays active");
    assert_eq!(map_original_events(&tail), case.next_input);
}

#[test]
fn replacement_memory_synthesizes_last_word_after_glued_phrase_split() {
    let row = first_fixture_row("daemon_replacement_memory_glued.tsv");
    let mut buffer = typed_buffer(&[(&row[0], true)]);
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
    let row = first_fixture_row("daemon_replacement_memory_completed.tsv");
    let mut buffer = typed_buffer(&[(&row[0], true), (&row[4], true)]);

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

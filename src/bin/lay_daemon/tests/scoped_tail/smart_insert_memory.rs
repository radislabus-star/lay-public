use super::*;
use crate::correction_memory_runtime::{
    remember_manual_text_correction, ManualTextCorrectionMemory,
};

fn smart_insert_decision_case(id: &str) -> Vec<String> {
    fixture_row_by_id("daemon_scoped_tail_smart_insert_decision.tsv", id)
}

fn smart_insert_memory_case(id: &str) -> Vec<String> {
    fixture_row_by_id("daemon_scoped_tail_smart_insert_memory.tsv", id)
}

struct SmartInsertMemoryCase {
    parts: String,
    scope: usize,
    original: String,
    replacement: String,
    plan: TextReplacement,
    memory_kind: String,
    remember_tail: bool,
    remember_last: bool,
    undo_original: String,
    undo_backspaces: Option<u32>,
    undo_target_is_ru: Option<bool>,
    undo_target: String,
}

impl SmartInsertMemoryCase {
    fn load(id: &str) -> Self {
        let row = smart_insert_memory_case(id);
        assert_eq!(row.len(), 16, "bad smart-insert memory row {id:?}: {row:?}");
        Self {
            parts: row[1].clone(),
            scope: row[2].parse().expect("scope"),
            original: row[3].clone(),
            replacement: row[4].clone(),
            plan: text_replacement_from_fixture(&row, 5, 6, 7, 8),
            memory_kind: row[9].clone(),
            remember_tail: row[10].parse().expect("remember_tail"),
            remember_last: row[11].parse().expect("remember_last"),
            undo_original: row[12].clone(),
            undo_backspaces: (!row[13].is_empty()).then(|| {
                row[13]
                    .parse()
                    .expect("undo_backspaces must be u32 or empty")
            }),
            undo_target_is_ru: (!row[14].is_empty()).then(|| {
                row[14]
                    .parse()
                    .expect("undo_target_is_ru must be bool or empty")
            }),
            undo_target: row[15].clone(),
        }
    }

    fn buffer(&self) -> WordBuffer {
        typed_buffer_from_fixture_parts(&self.parts)
    }
}

#[test]
fn smart_decision_keeps_good_word_and_converts_bad_neighbor() {
    let row = smart_insert_decision_case("mixed_neighbor");
    assert_eq!(row.len(), 4, "bad smart-insert decision row: {row:?}");
    assert_eq!(
        decide_correction(&row[1], &row[2], CorrectionEngine::Smart),
        Correction::InsertText(row[3].clone())
    );
}

#[test]
fn scoped_tail_keeps_good_previous_word_and_flips_current_fragment() {
    let case = SmartInsertMemoryCase::load("current_fragment");
    let buffer = case.buffer();
    let (events, _) = buffer.what_to_replay(case.scope).expect("two-word tail");

    assert_eq!(map_original_events(&events), case.original);
    assert_eq!(
        decide_scoped_tail_correction(&events),
        Some(case.replacement.clone())
    );
    assert_eq!(
        plan_text_replacement(&case.original, &case.replacement),
        Some(case.plan)
    );
}

fn assert_smart_insert_memory_case(id: &str) {
    let case = SmartInsertMemoryCase::load(id);
    let mut buffer = case.buffer();
    let (events, _) = buffer.what_to_replay(case.scope).expect("two-word tail");
    let original = map_original_events(&events);
    let replacement = decide_scoped_tail_correction(&events).expect("smart replacement");
    let plan = plan_text_replacement(&original, &replacement).expect("minimal plan");

    assert_eq!(original, case.original);
    assert_eq!(replacement, case.replacement);
    assert_eq!(plan, case.plan);

    if case.remember_tail {
        assert!(buffer.remember_inserted_tail_for_replay(&events, &plan, false));
    } else {
        assert!(!buffer.remember_inserted_tail_for_replay(&events, &plan, false));
    }
    if case.remember_last {
        assert!(buffer.remember_inserted_last_word_for_replay(&events, &plan));
    }

    assert_undo_tail(&buffer, &case);
}

#[test]
fn smart_insert_remembers_only_inserted_tail_for_immediate_undo() {
    assert_smart_insert_memory_case("inserted_tail");
}

#[test]
fn smart_insert_remembers_last_word_after_full_tail_replace() {
    assert_smart_insert_memory_case("last_word_after_full_tail");
}

fn assert_undo_tail(buffer: &WordBuffer, case: &SmartInsertMemoryCase) {
    let (undo_events, undo_backspaces) = buffer.what_to_replay(2).expect("undo tail");
    let undo_decision = replay_layout_decision(&undo_events);
    assert_eq!(map_original_events(&undo_events), case.undo_original);
    assert_eq!(Some(undo_backspaces), case.undo_backspaces);
    assert_eq!(Some(undo_decision.target_is_ru), case.undo_target_is_ru);
    assert_eq!(
        map_events_to_layout(&undo_events, undo_decision.target_is_ru),
        case.undo_target
    );
    assert!(buffer.replay_toggle_ready());
}

#[test]
fn manual_text_correction_keeps_pending_full_undo() {
    let case = SmartInsertMemoryCase::load("manual_full_undo");
    let mut buffer = case.buffer();
    let (events, _) = buffer.what_to_replay(case.scope).expect("two-word tail");
    let original = map_original_events(&events);
    let replacement = case.replacement.clone();
    let plan = plan_text_replacement(&original, &replacement).expect("minimal plan");

    assert_eq!(original, case.original);
    assert_eq!(plan, case.plan);

    remember_manual_text_correction(
        &mut buffer,
        ManualTextCorrectionMemory {
            events: &events,
            plan: &plan,
            original: &original,
            replacement: &replacement,
            kind: &case.memory_kind,
            replace_words: case.scope,
            words: case.scope,
            inserted_layout_is_ru: Some(true),
        },
    );

    let undo = buffer.take_pending_auto_undo().expect("pending undo");
    assert_eq!(undo.original, case.original);
    assert_eq!(undo.replacement, case.replacement);
    assert_eq!(
        undo.replacement_plan(),
        text_replacement(
            0,
            case.undo_backspaces
                .expect("manual full undo fixture must include undo_backspaces"),
            case.undo_original,
            0,
        )
    );
}

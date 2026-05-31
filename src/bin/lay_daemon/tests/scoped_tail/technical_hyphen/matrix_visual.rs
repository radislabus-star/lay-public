use super::*;

fn matrix_visual_case(id: &str) -> Vec<String> {
    fixture_row_by_id("daemon_scoped_tail_matrix_visual.tsv", id)
}

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
    let english_words = fixture_lines("daemon_scoped_tail_matrix_english_words.txt");
    let russian_words = fixture_lines("daemon_scoped_tail_matrix_russian_words.txt");

    let mut cases = 0;
    for left in &english_words {
        for target in &russian_words {
            let typed = lay::dict::convert(target, lay::dict::Direction::Ru2Us);
            assert_smart_pair(left, false, &typed, false, &format!("{left} {target}"));
            cases += 1;
        }
    }

    for left in &russian_words {
        for target in &english_words {
            let typed = lay::dict::convert(target, lay::dict::Direction::Us2Ru);
            assert_smart_pair(left, true, &typed, true, &format!("{left} {target}"));
            cases += 1;
        }
    }

    assert!(cases >= 100, "expected at least 100 mixed pair cases");
}

#[test]
fn scoped_tail_flips_current_visual_latin_word_with_cyrillic_c_homoglyph() {
    let row = matrix_visual_case("cyrillic_c_homoglyph");
    assert_eq!(row.len(), 5, "bad fixture row: {row:?}");
    let buffer = typed_buffer_from_fixture_parts(&row[1]);
    let scope: usize = row[2].parse().expect("scope");
    let (events, _) = buffer.what_to_replay(scope).expect("two-word tail");

    assert_eq!(map_original_events(&events), row[3]);
    assert_eq!(decide_scoped_tail_correction(&events), Some(row[4].clone()));
}

#[test]
fn scoped_tail_removes_duplicate_layout_prefix_from_completed_ascii_technical_token() {
    let row = matrix_visual_case("duplicate_ascii_prefix");
    assert_eq!(row.len(), 5, "bad fixture row: {row:?}");
    let buffer = typed_buffer_from_fixture_parts(&row[1]);
    let scope: usize = row[2].parse().expect("scope");
    let (events, _) = buffer.what_to_replay(scope).expect("two-word tail");

    assert_eq!(map_original_events(&events), row[3]);
    assert_eq!(decide_scoped_tail_correction(&events), Some(row[4].clone()));
}

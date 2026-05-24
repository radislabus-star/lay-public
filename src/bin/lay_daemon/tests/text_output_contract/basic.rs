use super::super::*;

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

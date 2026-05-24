use super::*;

#[test]
fn typing_assist_pipeline_can_disable_rules() {
    let no_en_to_ru = typing_pipeline_with_disabled(&["layout_en_to_ru"]);
    assert_eq!(
        apply_typing_assist_with_pipeline("njkmrj ", true, &no_en_to_ru),
        None
    );

    let no_ru_to_en = typing_pipeline_with_disabled(&["layout_ru_to_en"]);
    assert_eq!(
        apply_typing_assist_with_pipeline("ашдуы ", true, &no_ru_to_en),
        None
    );

    let no_hard_sign = typing_pipeline_with_disabled(&["hard_sign"]);
    assert_eq!(
        apply_typing_assist_with_pipeline("Обьясни ", false, &no_hard_sign),
        None
    );
}

#[test]
fn typing_assist_pipeline_priority_changes_first_match() {
    let personal_first = typing_pipeline_with_first("personal_phrase");
    let normalized = normalize_typing_assist_pipeline(&personal_first);
    assert_eq!(normalized[0].id, "personal_phrase");
    assert_eq!(normalized[0].priority, 1);
}

#[test]
fn typing_assist_each_default_rule_has_isolated_positive_case() {
    struct Case {
        id: String,
        input: String,
        expected: Option<String>,
        allow_layout_auto: bool,
    }

    let technical_ascii =
        map_events_to_layout(&key_events(&ascii_hyphen_token_keycodes(), false), false);
    let technical_cyrillic = lay::dict::convert(&technical_ascii, lay::dict::Direction::Us2Ru);
    let prefix_cyrillic = map_events_to_layout(&[key_event(KeyCode::KEY_W, true)], true);

    let mut cases: Vec<Case> = fixture_rows("daemon_typing_assist_default_rule_cases.tsv")
        .into_iter()
        .map(|row| {
            assert_eq!(row.len(), 4, "default rule fixture must be TSV");
            Case {
                id: row[0].clone(),
                input: row[1].clone(),
                expected: (row[2] != "None").then(|| row[2].clone()),
                allow_layout_auto: row[3] == "true",
            }
        })
        .collect();
    cases.extend([
        Case {
            id: "duplicate_layout_prefix".to_string(),
            input: format!("{prefix_cyrillic}{technical_ascii} "),
            expected: Some(format!("{technical_ascii} ")),
            allow_layout_auto: false,
        },
        Case {
            id: "layout_technical".to_string(),
            input: format!("{technical_cyrillic} "),
            expected: Some(format!("{technical_ascii} ")),
            allow_layout_auto: false,
        },
    ]);

    let mut covered: HashSet<String> = HashSet::new();
    for case in cases {
        let pipeline = typing_pipeline_with_only(&case.id);
        assert_eq!(
            apply_typing_assist_with_pipeline(&case.input, case.allow_layout_auto, &pipeline),
            case.expected,
            "rule={} input={:?}",
            case.id,
            case.input
        );
        covered.insert(case.id);
    }

    let expected: HashSet<String> = default_typing_assist_rules()
        .into_iter()
        .map(|(id, _)| id.to_string())
        .collect();
    assert_eq!(covered, expected);
}

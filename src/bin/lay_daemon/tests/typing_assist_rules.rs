use super::*;

#[test]
fn applies_builtin_auto_replace_with_trailing_space() {
    assert_eq!(
        apply_auto_replace("gjlk.xbcm ", "подлючись "),
        Some("подключись ".to_string())
    );
    assert_eq!(apply_auto_replace("Tcnm ", "Есть "), None);
}

#[test]
fn typing_assist_uses_exact_rules_only() {
    assert_eq!(
        apply_typing_assist_exact("подлючись "),
        Some("подключись ".to_string())
    );
    assert_eq!(
        apply_typing_assist_exact("Надйи "),
        Some("Найди ".to_string())
    );
    assert_eq!(apply_typing_assist_exact("нормально "), None);
    assert_eq!(apply_typing_assist_exact("Есть "), None);
}

#[test]
fn russian_suffix_forms_are_known_candidates() {
    assert!(is_known_russian_word_or_form("препаратов"));
    assert!(is_known_russian_word_or_form("кнопками"));
    assert!(is_known_russian_word_or_form("могу"));
    assert!(is_known_russian_word_or_form("помогу"));
    assert!(is_known_russian_word_or_form("видишь"));
    assert!(is_known_russian_word_or_form("значит"));
    assert!(is_known_russian_word_or_form("страдает"));
    assert!(is_known_russian_word_or_form("установки"));
}

#[test]
fn typing_assist_auto_switch_blocks_plain_layout_words_and_keeps_explicit_cases() {
    for input in [
        "njkmrj ",
        "vjue ",
        "yt ",
        "hf,jnftn ",
        "'nj ",
        "Lfdfq ",
        "lfkmit ",
    ] {
        assert_eq!(
            apply_typing_assist(input, true),
            None,
            "plain layout word must not be auto-switched: {input:?}"
        );
    }

    assert_eq!(
        apply_typing_assist("double b ", true),
        Some("double и ".to_string())
    );
    for row in fixture_rows("daemon_typing_assist_tail_cases.tsv") {
        assert_eq!(row.len(), 2, "tail cases fixture must be TSV");
        assert_eq!(
            apply_typing_assist_to_text_tail(&row[0]),
            Some(row[1].clone())
        );
    }
    assert_eq!(
        apply_typing_assist("ашду ", true),
        Some("file ".to_string())
    );
    assert_eq!(
        apply_typing_assist("ашдуы ", true),
        Some("files ".to_string())
    );
    assert_eq!(
        apply_typing_assist("еукьштфд ", true),
        Some("terminal ".to_string())
    );
    assert_eq!(
        apply_typing_assist("сфкпщ ", true),
        Some("cargo ".to_string())
    );
    assert_eq!(
        apply_typing_assist("ОБYJDB ", true),
        Some("ОБНОВИ ".to_string())
    );
    assert_eq!(
        apply_typing_assist("CRBK ", true),
        Some("СКИЛ ".to_string())
    );
    for row in fixture_rows("daemon_typing_assist_layout_explicit.tsv") {
        assert_eq!(row.len(), 2, "layout explicit fixture must be TSV");
        assert_eq!(apply_typing_assist(&row[0], true), Some(row[1].clone()));
    }
}

#[test]
fn typing_assist_auto_replace_off_keeps_layout_only_rules() {
    let pipeline =
        typing_assist_pipeline_for_auto_replace(false, &default_typing_assist_pipeline());

    assert_eq!(
        apply_typing_assist_with_pipeline("кгы ", true, &pipeline),
        Some("rus ".to_string())
    );
    assert_eq!(
        apply_typing_assist_with_pipeline("утп ", true, &pipeline),
        Some("eng ".to_string())
    );
    assert_eq!(
        apply_typing_assist_with_pipeline("njkmrj ", true, &pipeline),
        None
    );
    assert_eq!(
        apply_typing_assist_with_pipeline("прорватся ", false, &pipeline),
        None
    );
    assert_eq!(
        apply_typing_assist_with_pipeline("фактческим ", false, &pipeline),
        None
    );
}

#[test]
fn typing_assist_auto_replace_pipeline_avoids_risky_deletions() {
    let pipeline = typing_assist_pipeline_for_auto_replace(true, &default_typing_assist_pipeline());

    assert_eq!(
        apply_typing_assist_with_pipeline("исправленнно ", false, &pipeline),
        Some("исправлено ".to_string())
    );
    assert_eq!(
        apply_typing_assist_with_pipeline("кнокопками ", false, &pipeline),
        None
    );
    assert_eq!(
        apply_typing_assist_with_pipeline("бешанный ", false, &pipeline),
        None
    );
}

#[test]
fn typing_assist_prefers_reflexive_verb_fix_over_extra_letter_guess() {
    assert_eq!(correct_extra_letters("прорватся"), None);
    assert_eq!(
        apply_typing_assist("прорватся ", false),
        Some("прорваться ".to_string())
    );
    assert_eq!(
        apply_typing_assist("ошибатся ", false),
        Some("ошибаться ".to_string())
    );
}

#[test]
fn typing_assist_auto_switch_keeps_english_and_protected_ascii() {
    assert_eq!(apply_typing_assist("hello ", true), None);
    assert_eq!(apply_typing_assist("test ", true), None);
    assert_eq!(apply_typing_assist("good ", true), None);
    assert_eq!(apply_typing_assist("три ", true), None);
    assert_eq!(apply_typing_assist("раскладок ", true), None);
    assert_eq!(apply_typing_assist("API ", true), None);
    assert_eq!(apply_typing_assist("BTC ", true), None);
    assert_eq!(apply_typing_assist("ETH ", true), None);
    assert_eq!(apply_typing_assist("TRX ", true), None);
    assert_eq!(apply_typing_assist("AmoCRM ", true), None);
    assert_eq!(apply_typing_assist("wi-fi ", true), None);
    assert_eq!(apply_typing_assist("command -f ", true), None);
    assert_eq!(apply_typing_assist("command -r ", true), None);
    assert_eq!(apply_typing_assist("command -c ", true), None);
    assert_eq!(apply_typing_assist("grep --color=auto ", true), None);
}

#[test]
fn typing_assist_keeps_user_protected_ascii_words_when_configured() {
    if std::env::var_os("LAY_TEST_USER_PROTECTED_ASCII").is_none() {
        return;
    }

    assert_eq!(apply_typing_assist("vs ", true), None);
}

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

    let expected: HashSet<String> = DEFAULT_TYPING_ASSIST_RULES
        .iter()
        .map(|(id, _)| (*id).to_string())
        .collect();
    assert_eq!(covered, expected);
}

#[test]
fn typing_assist_fixes_adjacent_transposition() {
    for row in fixture_rows("daemon_typing_assist_transposition.tsv") {
        assert_eq!(row.len(), 3, "transposition fixture must be TSV");
        let got = if row[2] == "tail" {
            apply_typing_assist_to_text_tail(&row[0])
        } else {
            apply_typing_assist_exact(&row[0])
        };
        assert_eq!(got, Some(row[1].clone()), "input={:?}", row[0]);
    }
}

#[test]
fn typing_assist_fixes_small_glued_words() {
    for row in fixture_rows("daemon_typing_assist_small_glued.tsv") {
        assert_eq!(row.len(), 2, "small glued fixture must be TSV");
        assert_eq!(apply_typing_assist_exact(&row[0]), Some(row[1].clone()));
    }
}

#[test]
fn typing_assist_fixes_common_missing_letter_typos() {
    assert_eq!(
        apply_typing_assist_exact("првильно "),
        Some("правильно ".to_string())
    );
    assert_eq!(
        apply_typing_assist_exact("Првильно "),
        Some("Правильно ".to_string())
    );
    assert_eq!(
        apply_typing_assist_exact("можн "),
        Some("можно ".to_string())
    );
    assert_eq!(
        apply_typing_assist_exact("Можн "),
        Some("Можно ".to_string())
    );
    assert_eq!(
        apply_typing_assist_exact("дльше "),
        Some("дальше ".to_string())
    );
    assert_eq!(
        apply_typing_assist_exact("дальг "),
        Some("дальше ".to_string())
    );
    assert_eq!(
        apply_typing_assist_exact("плозо "),
        Some("плохо ".to_string())
    );
    assert_eq!(
        apply_typing_assist_exact("фактческим "),
        Some("фактическим ".to_string())
    );
    assert_eq!(
        apply_typing_assist_exact("иблиотеку "),
        Some("библиотеку ".to_string())
    );
    assert_eq!(apply_typing_assist_exact("крипта "), None);
    assert_eq!(apply_typing_assist_exact("Крипта "), None);
}

#[test]
fn typing_assist_fixes_live_user_stream_typos() {
    assert!(is_known_russian_word_or_form("ориентироваться"));
    assert!(is_known_russian_word_or_form("переиспользоваться"));
    assert_eq!(
        apply_typing_assist_exact("занчит "),
        Some("значит ".to_string())
    );
    assert_eq!(
        apply_typing_assist_exact("орентироваться "),
        Some("ориентироваться ".to_string())
    );
    assert_eq!(
        apply_typing_assist_exact("переиспользоватся "),
        Some("переиспользоваться ".to_string())
    );
    assert_eq!(
        apply_typing_assist_exact("работатет "),
        Some("работает ".to_string())
    );
    assert_eq!(
        apply_typing_assist_exact("котром "),
        Some("котором ".to_string())
    );
    assert_eq!(
        apply_typing_assist_exact("рабоТТА "),
        Some("работа ".to_string())
    );
    assert_eq!(
        apply_typing_assist_exact("помагу "),
        Some("помогу ".to_string())
    );
    assert_eq!(
        apply_typing_assist_exact("видешь "),
        Some("видишь ".to_string())
    );
    assert_eq!(
        apply_typing_assist_exact("кнокопками "),
        Some("кнопками ".to_string())
    );
}

#[test]
fn typing_assist_normalizes_accidental_inner_uppercase() {
    assert_eq!(
        apply_typing_assist_exact("МОжно "),
        Some("Можно ".to_string())
    );
    assert_eq!(
        apply_typing_assist_exact("моЖно "),
        Some("можно ".to_string())
    );
    assert_eq!(
        apply_typing_assist_exact("рабоТА "),
        Some("работа ".to_string())
    );
    assert_eq!(
        apply_typing_assist_exact("улУЧШИТЬ "),
        Some("улучшить ".to_string())
    );
    assert_eq!(
        apply_typing_assist_exact("улУЧШИТЬ? "),
        Some("улучшить? ".to_string())
    );
    assert_eq!(apply_typing_assist_exact("МОЖНО "), None);
}

#[test]
fn typing_assist_single_letter_typos_only_use_neighbor_keys() {
    assert!(are_ru_keyboard_neighbors('з', 'х'));
    assert!(!are_ru_keyboard_neighbors('о', 'ь'));
    assert_eq!(apply_typing_assist_exact("покрыто "), None);
    assert_eq!(apply_typing_assist_exact("робило "), None);
    assert_eq!(
        apply_typing_assist_exact("плозо "),
        Some("плохо ".to_string())
    );
}

#[test]
fn typing_assist_merges_accidental_space_inside_word() {
    for row in fixture_rows("daemon_typing_assist_split_word_fix.tsv") {
        assert_eq!(row.len(), 2, "split-word fixture must be TSV");
        assert_eq!(
            apply_typing_assist_exact(&row[0]),
            Some(row[1].clone()),
            "input={:?}",
            row[0]
        );
    }
    for input in fixture_lines("daemon_typing_assist_split_word_keep.txt") {
        assert_eq!(apply_typing_assist_exact(&input), None, "input={input:?}");
    }
    for input in fixture_lines("daemon_typing_assist_split_word_keep_layout.txt") {
        assert_eq!(apply_typing_assist(&input, true), None, "input={input:?}");
    }
}

#[test]
fn typing_assist_splits_accidentally_glued_words() {
    for row in fixture_rows("daemon_typing_assist_glued_split_fix.tsv") {
        assert_eq!(row.len(), 2, "glued-split fixture must be TSV");
        assert_eq!(
            apply_typing_assist_exact(&row[0]),
            Some(row[1].clone()),
            "input={:?}",
            row[0]
        );
    }
    for input in fixture_lines("daemon_typing_assist_glued_split_keep.txt") {
        assert_eq!(apply_typing_assist_exact(&input), None, "input={input:?}");
    }
    for input in fixture_lines("daemon_typing_assist_glued_split_keep_tail.txt") {
        assert_eq!(
            apply_typing_assist_to_text_tail(&input),
            None,
            "input={input:?}"
        );
    }
}

#[test]
fn typing_assist_fixes_hard_sign_typos() {
    assert_eq!(
        apply_typing_assist_exact("Обьясни "),
        Some("Объясни ".to_string())
    );
    assert_eq!(apply_typing_assist_exact("ОБЬЯСНИШСНИШЬ "), None);
}

#[test]
fn typing_assist_moves_letter_from_next_word_back() {
    for row in fixture_rows("daemon_typing_assist_moved_prefix.tsv") {
        assert_eq!(row.len(), 3, "moved-prefix fixture must be TSV");
        let input = &row[0];
        let expected = &row[1];
        let use_tail = row[2] == "tail";
        let got = if use_tail {
            apply_typing_assist_to_text_tail(input)
        } else {
            apply_typing_assist_exact(input)
        };
        assert_eq!(got, Some(expected.clone()), "input={input:?}");
    }
}

#[test]
fn typing_assist_removes_duplicate_layout_prefix_from_ascii_technical_token() {
    let prefix_lower = map_events_to_layout(&[key_event(KeyCode::KEY_W, true)], true);
    let prefix_upper = map_events_to_layout(
        &[KeyEvent {
            keycode: KeyCode::KEY_W.code(),
            shift: true,
            layout_is_ru: true,
        }],
        true,
    );
    let technical_lower =
        map_events_to_layout(&key_events(&ascii_hyphen_token_keycodes(), false), false);
    let technical_upper = map_events_to_layout(
        &[
            KeyEvent {
                keycode: KeyCode::KEY_W.code(),
                shift: true,
                layout_is_ru: false,
            },
            key_event(KeyCode::KEY_I, false),
            key_event(KeyCode::KEY_MINUS, false),
            KeyEvent {
                keycode: KeyCode::KEY_F.code(),
                shift: true,
                layout_is_ru: false,
            },
            key_event(KeyCode::KEY_I, false),
        ],
        false,
    );
    let no_separator = map_events_to_layout(
        &key_events(
            &[
                KeyCode::KEY_W,
                KeyCode::KEY_I,
                KeyCode::KEY_F,
                KeyCode::KEY_I,
            ],
            false,
        ),
        false,
    );

    assert_eq!(
        apply_typing_assist_exact(&format!("{prefix_lower}{technical_lower} ")),
        Some(format!("{technical_lower} "))
    );
    assert_eq!(
        apply_typing_assist_exact(&format!("{prefix_upper}{technical_upper}, ")),
        Some(format!("{technical_upper}, "))
    );
    assert_eq!(
        apply_typing_assist_exact(&format!("{prefix_lower}{no_separator} ")),
        None
    );
}

#[test]
fn typing_assist_does_not_move_normal_word_prefixes() {
    for input in fixture_lines("daemon_typing_assist_prefix_keep.txt") {
        assert_eq!(apply_typing_assist_exact(&input), None, "input={input:?}");
    }
}

#[test]
fn typing_assist_fixes_extra_repeated_letter() {
    assert_eq!(
        apply_typing_assist_exact("исправленно "),
        Some("исправлено ".to_string())
    );
    assert_eq!(
        apply_typing_assist_exact("исправленнно "),
        Some("исправлено ".to_string())
    );
    for row in fixture_rows("daemon_typing_assist_repeated_letter.tsv") {
        assert_eq!(row.len(), 2, "repeated-letter fixture must be TSV");
        assert_eq!(apply_typing_assist_exact(&row[0]), Some(row[1].clone()));
    }
    assert_eq!(apply_typing_assist_exact("про "), None);
    assert_eq!(apply_typing_assist_exact("ии "), None);
    assert_eq!(apply_typing_assist_exact("яя "), None);
    assert_eq!(apply_typing_assist_exact("вв "), None);
}

#[test]
fn extra_letter_rule_defers_to_missing_letter_candidates() {
    let mut words: Vec<String> = russian_generated_form_dictionary()
        .iter()
        .filter(|word| (7..=12).contains(&word.chars().count()))
        .cloned()
        .collect();
    words.sort();

    let mut checked = 0usize;
    'outer: for word in words {
        let chars: Vec<char> = word.chars().collect();
        for idx in 1..chars.len().saturating_sub(1) {
            let mut typo_chars = chars.clone();
            typo_chars.remove(idx);
            let typo: String = typo_chars.into_iter().collect();
            if typo.chars().count() < 6 || is_known_russian_word_or_form(&typo) {
                continue;
            }
            if correct_missing_letter(&typo).as_deref() != Some(word.as_str()) {
                continue;
            }

            assert_eq!(correct_extra_letters(&typo), None, "typo={typo:?}");
            checked += 1;
            if checked >= 12 {
                break 'outer;
            }
            break;
        }
    }

    assert!(checked >= 12, "checked={checked}");
}

#[test]
fn typing_assist_keeps_valid_russian_words() {
    assert_eq!(apply_typing_assist_exact("проверка "), None);
    assert_eq!(apply_typing_assist_exact("работает "), None);
    assert_eq!(apply_typing_assist_exact("привет "), None);
    assert_eq!(apply_typing_assist_exact("можем "), None);
    assert_eq!(apply_typing_assist_exact("можешь "), None);
    assert_eq!(apply_typing_assist_exact("может "), None);
    assert_eq!(apply_typing_assist_exact("ладно "), None);
    assert_eq!(apply_typing_assist_exact("можно "), None);
    assert_eq!(apply_typing_assist_exact("дальше "), None);
    assert_eq!(apply_typing_assist_exact("плохо "), None);
    assert_eq!(apply_typing_assist_exact("правильно "), None);
    assert_eq!(apply_typing_assist_exact("исправляет "), None);
    assert_eq!(apply_typing_assist_exact("начинаю "), None);
    assert_eq!(apply_typing_assist_exact("удаляется "), None);
    assert_eq!(apply_typing_assist_exact("удателятеся "), None);
    assert_eq!(apply_typing_assist_exact("еще "), None);
    assert_eq!(apply_typing_assist_exact("елка "), None);
    assert_eq!(apply_typing_assist_exact("все "), None);
    assert_eq!(apply_typing_assist_exact("раскладок "), None);
    assert_eq!(apply_typing_assist_exact("кнопок "), None);
    assert_eq!(apply_typing_assist_exact("тестами "), None);
    assert_eq!(apply_typing_assist_exact("словами "), None);
    assert_eq!(apply_typing_assist_exact("вариантами "), None);
    assert_eq!(apply_typing_assist_exact("страдает "), None);
    assert_eq!(apply_typing_assist_exact("установки "), None);
    assert_eq!(apply_typing_assist_exact("изменю "), None);
    for input in fixture_lines("daemon_typing_assist_valid_phrase_keep.txt") {
        assert_eq!(apply_typing_assist_exact(&input), None, "input={input:?}");
    }
    assert_eq!(apply_typing_assist_exact("нужна "), None);
    assert_eq!(apply_typing_assist_exact("важна "), None);
    assert_eq!(apply_typing_assist_exact("важно "), None);
    assert_eq!(apply_typing_assist_exact("банный "), None);
    assert_eq!(apply_typing_assist_exact("бешанный "), None);
    assert_eq!(apply_typing_assist_exact("БЕШАННЫЙ "), None);
    assert_eq!(apply_typing_assist_exact("поения "), None);
    assert_eq!(apply_typing_assist_exact("автозамена "), None);
    assert_eq!(apply_typing_assist_exact("агрессивная "), None);
}

#[test]
fn typing_assist_ignores_words_with_digits() {
    assert_eq!(apply_typing_assist_exact("товара7 "), None);
    assert_eq!(apply_typing_assist_exact("привемр7 "), None);
    for input in fixture_lines("daemon_typing_assist_digit_phrase_keep.txt") {
        assert_eq!(apply_typing_assist_exact(&input), None, "input={input:?}");
    }
}

#[test]
fn typing_assist_regression_suite_100_cases() {
    let should_fix = fixture_rows("daemon_typing_assist_regression_fix.tsv");
    for row in &should_fix {
        assert_eq!(row.len(), 2, "fix fixture must be TSV");
        let input = &row[0];
        let expected = &row[1];
        assert_eq!(
            apply_typing_assist_to_text_tail(input),
            Some(expected.clone()),
            "input={input:?}"
        );
    }

    let should_keep = fixture_lines("daemon_typing_assist_regression_keep.txt");
    for input in &should_keep {
        assert_eq!(
            apply_typing_assist_to_text_tail(input),
            None,
            "input={input:?}"
        );
    }

    let total = should_fix.len() + should_keep.len();
    assert!(
        total >= 100,
        "regression suite should keep at least 100 cases, got {total}"
    );
}

#[test]
fn auto_replace_regression_suite() {
    for row in fixture_rows("daemon_auto_replace_regression.tsv") {
        assert_eq!(row.len(), 3, "auto-replace fixture must be TSV");
        let original = &row[0];
        let target = &row[1];
        let expected = &row[2];
        assert_eq!(
            apply_auto_replace(original, target),
            Some(expected.clone()),
            "original={original:?} target={target:?}"
        );
    }
}

#[test]
fn replaces_visual_b_inside_russian_context() {
    let cases = fixture_rows("daemon_auto_replace_visual_b.tsv");
    let row = &cases[0];
    assert_eq!(apply_auto_replace(&row[0], &row[1]), Some(row[2].clone()));
    assert_eq!(
        apply_auto_replace("b ghjcnj", "и просто"),
        Some("в просто".to_string())
    );
    let row = &cases[1];
    assert_eq!(apply_auto_replace(&row[0], &row[1]), Some(row[2].clone()));
}

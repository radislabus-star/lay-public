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
    assert_eq!(
        apply_typing_assist_to_text_tail("посмотри я double b "),
        Some("посмотри я double и ".to_string())
    );
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
    assert_eq!(apply_typing_assist("кгы ", true), Some("rus ".to_string()));
    assert_eq!(apply_typing_assist("утп ", true), Some("eng ".to_string()));
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
        id: &'static str,
        input: String,
        expected: Option<String>,
        allow_layout_auto: bool,
    }

    let technical_ascii =
        map_events_to_layout(&key_events(&ascii_hyphen_token_keycodes(), false), false);
    let technical_cyrillic = lay::dict::convert(&technical_ascii, lay::dict::Direction::Us2Ru);
    let prefix_cyrillic = map_events_to_layout(&[key_event(KeyCode::KEY_W, true)], true);

    let cases = [
        Case {
            id: "moved_prefix_pair",
            input: "расчет ыприблизительные ".to_string(),
            expected: Some("расчеты приблизительные ".to_string()),
            allow_layout_auto: false,
        },
        Case {
            id: "split_word_pair",
            input: "я вно ".to_string(),
            expected: Some("явно ".to_string()),
            allow_layout_auto: false,
        },
        Case {
            id: "visual_b",
            input: "слово b ".to_string(),
            expected: Some("слово в ".to_string()),
            allow_layout_auto: false,
        },
        Case {
            id: "personal_phrase",
            input: "нуда ".to_string(),
            expected: Some("ну да ".to_string()),
            allow_layout_auto: false,
        },
        Case {
            id: "personal_token",
            input: "подлючись. ".to_string(),
            expected: Some("подключись. ".to_string()),
            allow_layout_auto: false,
        },
        Case {
            id: "duplicate_layout_prefix",
            input: format!("{prefix_cyrillic}{technical_ascii} "),
            expected: Some(format!("{technical_ascii} ")),
            allow_layout_auto: false,
        },
        Case {
            id: "mixed_script_layout",
            input: "ОБYJDB ".to_string(),
            expected: Some("ОБНОВИ ".to_string()),
            allow_layout_auto: true,
        },
        Case {
            id: "layout_technical",
            input: format!("{technical_cyrillic} "),
            expected: Some(format!("{technical_ascii} ")),
            allow_layout_auto: false,
        },
        Case {
            id: "layout_ru_to_en",
            input: "ашду ".to_string(),
            expected: Some("file ".to_string()),
            allow_layout_auto: true,
        },
        Case {
            id: "layout_en_to_ru",
            input: "njkmrj ".to_string(),
            expected: None,
            allow_layout_auto: true,
        },
        Case {
            id: "cyrillic_case",
            input: "МОжно ".to_string(),
            expected: Some("Можно ".to_string()),
            allow_layout_auto: false,
        },
        Case {
            id: "hard_sign",
            input: "Обьясни ".to_string(),
            expected: Some("Объясни ".to_string()),
            allow_layout_auto: false,
        },
        Case {
            id: "adjacent_transposition",
            input: "рабоатет ".to_string(),
            expected: Some("работает ".to_string()),
            allow_layout_auto: false,
        },
        Case {
            id: "repeated_letter",
            input: "исправленно ".to_string(),
            expected: Some("исправлено ".to_string()),
            allow_layout_auto: false,
        },
        Case {
            id: "single_letter_substitution",
            input: "плозо ".to_string(),
            expected: Some("плохо ".to_string()),
            allow_layout_auto: false,
        },
        Case {
            id: "verb_ending",
            input: "прорватся ".to_string(),
            expected: Some("прорваться ".to_string()),
            allow_layout_auto: false,
        },
        Case {
            id: "vowel_confusion",
            input: "помагу ".to_string(),
            expected: Some("помогу ".to_string()),
            allow_layout_auto: false,
        },
        Case {
            id: "extra_letters",
            input: "кнокопками ".to_string(),
            expected: Some("кнопками ".to_string()),
            allow_layout_auto: false,
        },
        Case {
            id: "missing_letter",
            input: "фактческим ".to_string(),
            expected: Some("фактическим ".to_string()),
            allow_layout_auto: false,
        },
        Case {
            id: "glued_phrase",
            input: "когдая ".to_string(),
            expected: Some("когда я ".to_string()),
            allow_layout_auto: false,
        },
    ];

    let mut covered = HashSet::new();
    for case in cases {
        let pipeline = typing_pipeline_with_only(case.id);
        assert_eq!(
            apply_typing_assist_with_pipeline(&case.input, case.allow_layout_auto, &pipeline),
            case.expected,
            "rule={} input={:?}",
            case.id,
            case.input
        );
        covered.insert(case.id);
    }

    let expected: HashSet<&str> = DEFAULT_TYPING_ASSIST_RULES
        .iter()
        .map(|(id, _)| *id)
        .collect();
    assert_eq!(covered, expected);
}

#[test]
fn typing_assist_fixes_adjacent_transposition() {
    assert_eq!(
        apply_typing_assist_exact("рабоатет "),
        Some("работает ".to_string())
    );
    assert_eq!(
        apply_typing_assist_exact("Проверак "),
        Some("Проверка ".to_string())
    );
    assert_eq!(
        apply_typing_assist_exact("перпаратов "),
        Some("препаратов ".to_string())
    );
    assert_eq!(
        apply_typing_assist_to_text_tail("сделай понятную таблицу конкретных перпаратов "),
        Some("сделай понятную таблицу конкретных препаратов ".to_string())
    );
}

#[test]
fn typing_assist_fixes_small_glued_words() {
    assert_eq!(
        apply_typing_assist_exact("нуда "),
        Some("ну да ".to_string())
    );
    assert_eq!(
        apply_typing_assist_exact("вчем "),
        Some("в чем ".to_string())
    );
    assert_eq!(
        apply_typing_assist_exact("Вчем, "),
        Some("В чем, ".to_string())
    );
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
    assert_eq!(
        apply_typing_assist_exact("я вно "),
        Some("явно ".to_string())
    );
    assert_eq!(
        apply_typing_assist_exact("тако й "),
        Some("такой ".to_string())
    );
    assert_eq!(
        apply_typing_assist_exact("Я вно, "),
        Some("Явно, ".to_string())
    );
    assert_eq!(apply_typing_assist_exact("я тут "), None);
    assert_eq!(apply_typing_assist_exact("мы сами "), None);
    assert_eq!(apply_typing_assist_exact("чтобы точно "), None);
    assert_eq!(apply_typing_assist_exact("хо хо "), None);
    assert_eq!(apply_typing_assist_exact("про сою "), None);
    assert_eq!(apply_typing_assist_exact("по делу "), None);
    assert_eq!(apply_typing_assist_exact("по любому "), None);
    assert_eq!(apply_typing_assist_exact("ПО ЛЮБОМУ "), None);
    assert_eq!(apply_typing_assist_exact("уже по любому "), None);
    assert_eq!(apply_typing_assist_exact("я ГОДАМИ! "), None);
    assert_eq!(apply_typing_assist_exact("проблем "), None);
    assert_eq!(apply_typing_assist_exact("валют "), None);
    assert_eq!(apply_typing_assist_exact("систем "), None);
    assert_eq!(apply_typing_assist_exact("ноавый "), None);
    assert_eq!(apply_typing_assist("ноавый ", true), None);
    assert_eq!(apply_typing_assist_exact("раработает "), None);
    assert_eq!(apply_typing_assist_exact("зработает "), None);
    assert_eq!(apply_typing_assist_exact("новавый "), None);
    assert_eq!(
        apply_typing_assist_exact("новыйы "),
        Some("новый ".to_string())
    );
    assert_eq!(apply_typing_assist_exact("за дело "), None);
}

#[test]
fn typing_assist_splits_accidentally_glued_words() {
    assert_eq!(
        apply_typing_assist_exact("ятут "),
        Some("я тут ".to_string())
    );
    assert_eq!(
        apply_typing_assist_exact("чтобыточно "),
        Some("чтобы точно ".to_string())
    );
    assert_eq!(
        apply_typing_assist_exact("когдая "),
        Some("когда я ".to_string())
    );
    assert_eq!(
        apply_typing_assist_exact("еслия "),
        Some("если я ".to_string())
    );
    assert_eq!(
        apply_typing_assist_exact("тогдая "),
        Some("тогда я ".to_string())
    );
    assert_eq!(
        apply_typing_assist_exact("можноя "),
        Some("можно я ".to_string())
    );
    assert_eq!(
        apply_typing_assist_exact("неработает "),
        Some("не работает ".to_string())
    );
    assert_eq!(
        apply_typing_assist_exact("Неработает, "),
        Some("Не работает, ".to_string())
    );
    assert_eq!(
        apply_typing_assist_exact("будуя "),
        Some("буду я ".to_string())
    );
    assert_eq!(
        apply_typing_assist_exact("у насесть "),
        Some("у нас есть ".to_string())
    );
    assert_eq!(
        apply_typing_assist_exact("какпроверка "),
        Some("как проверка ".to_string())
    );
    assert_eq!(
        apply_typing_assist_exact("тожесамое "),
        Some("тоже самое ".to_string())
    );
    assert_eq!(
        apply_typing_assist_exact("тоесамое "),
        Some("тоже самое ".to_string())
    );
    assert_eq!(
        apply_typing_assist_exact("самоетоже "),
        Some("самое тоже ".to_string())
    );
    assert_eq!(
        apply_typing_assist_exact("вотэто "),
        Some("вот это ".to_string())
    );
    assert_eq!(
        apply_typing_assist_exact("янебудузавас "),
        Some("я не буду за вас ".to_string())
    );
    assert_eq!(
        apply_typing_assist_exact("янебуду "),
        Some("я не буду ".to_string())
    );
    assert_eq!(
        apply_typing_assist_exact("янебудузавастожесамое "),
        Some("я не буду за вас тоже самое ".to_string())
    );
    assert_eq!(
        apply_typing_assist_exact("ненаучишьсярезатьслова "),
        Some("не научишься резать слова ".to_string())
    );
    assert_eq!(apply_typing_assist_exact("но не "), None);
    assert_eq!(apply_typing_assist_exact("не ты "), None);
    assert_eq!(apply_typing_assist_exact("ноне ты "), None);
    assert_eq!(apply_typing_assist_exact("у насест "), None);
    assert_eq!(apply_typing_assist_exact("у насилие "), None);
    assert_eq!(apply_typing_assist_exact("машина "), None);
    assert_eq!(apply_typing_assist_exact("земля "), None);
    assert_eq!(apply_typing_assist_exact("какая "), None);
    assert_eq!(apply_typing_assist_exact("статья "), None);
    assert_eq!(apply_typing_assist_exact("семья "), None);
    assert_eq!(apply_typing_assist_exact("идея "), None);
    assert_eq!(apply_typing_assist_exact("синяя "), None);
    assert_eq!(apply_typing_assist_exact("пошли "), None);
    assert_eq!(apply_typing_assist_exact("язык "), None);
    assert_eq!(apply_typing_assist_exact("изводитель?! "), None);
    assert_eq!(apply_typing_assist_exact("отточеная "), None);
    assert_eq!(apply_typing_assist_to_text_tail("я язык "), None);
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
    assert_eq!(
        apply_typing_assist_exact("расчет ыприблизительные "),
        Some("расчеты приблизительные ".to_string())
    );
    assert_eq!(
        apply_typing_assist_exact("дл япроверки "),
        Some("для проверки ".to_string())
    );
    assert_eq!(
        apply_typing_assist_to_text_tail("все расчет ыприблизительные "),
        Some("все расчеты приблизительные ".to_string())
    );
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
    assert_eq!(apply_typing_assist_exact("схеме таможенник "), None);
    assert_eq!(apply_typing_assist_exact("схема таможженик "), None);
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
    assert_eq!(apply_typing_assist_exact("поо "), Some("по ".to_string()));
    assert_eq!(apply_typing_assist_exact("ПОО "), Some("ПО ".to_string()));
    assert_eq!(apply_typing_assist_exact("заа "), Some("за ".to_string()));
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
    assert_eq!(apply_typing_assist_exact("изменю параметры "), None);
    assert_eq!(apply_typing_assist_exact("нужна "), None);
    assert_eq!(apply_typing_assist_exact("она нужна "), None);
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
    assert_eq!(apply_typing_assist_exact("пример? привемр7 "), None);
}

#[test]
fn typing_assist_regression_suite_100_cases() {
    let should_fix = [
        ("подлючись ", "подключись "),
        ("надйи ", "найди "),
        ("Надйи ", "Найди "),
        ("нуда ", "ну да "),
        ("Нуда ", "Ну да "),
        ("вчем ", "в чем "),
        ("Вчем, ", "В чем, "),
        ("можн ", "можно "),
        ("Можн ", "Можно "),
        ("МОжно ", "Можно "),
        ("моЖно ", "можно "),
        ("дльше ", "дальше "),
        ("Дльше ", "Дальше "),
        ("дальг ", "дальше "),
        ("првильно ", "правильно "),
        ("Првильно ", "Правильно "),
        ("рабоатет ", "работает "),
        ("Рабоатет ", "Работает "),
        ("Проверак ", "Проверка "),
        ("ошисбя ", "ошибся "),
        ("Ошисбя ", "Ошибся "),
        ("сиправить ", "исправить "),
        ("Сиправить ", "Исправить "),
        ("плозо ", "плохо "),
        ("Плозо ", "Плохо "),
        ("фактческим ", "фактическим "),
        ("иблиотеку ", "библиотеку "),
        ("занчит ", "значит "),
        ("работатет ", "работает "),
        ("помагу ", "помогу "),
        ("видешь ", "видишь "),
        ("кнокопками ", "кнопками "),
        ("Обьясни ", "Объясни "),
        ("исправленно ", "исправлено "),
        ("Исправленно ", "Исправлено "),
        ("исправленнно ", "исправлено "),
        ("я вно ", "явно "),
        ("Я вно, ", "Явно, "),
        (
            "все расчет ыприблизительные ",
            "все расчеты приблизительные ",
        ),
        ("тут я вно ", "тут явно "),
        ("Но я вно ", "Но явно "),
        ("подлючись. ", "подключись. "),
        ("надйи! ", "найди! "),
        ("можн? ", "можно? "),
        ("дльше, ", "дальше, "),
        ("првильно. ", "правильно. "),
        ("плозо! ", "плохо! "),
        ("ошисбя, ", "ошибся, "),
    ];

    for (input, expected) in should_fix {
        assert_eq!(
            apply_typing_assist_to_text_tail(input),
            Some(expected.to_string()),
            "input={input:?}"
        );
    }

    let should_keep = [
        "привет ",
        "проверка ",
        "работает ",
        "ошибка ",
        "ошибся ",
        "явно ",
        "ладно ",
        "можно ",
        "дальше ",
        "плохо ",
        "правильно ",
        "исправлено ",
        "исправляет ",
        "покрыто ",
        "покрыть ",
        "слово ",
        "текст ",
        "модель ",
        "режим ",
        "файл ",
        "проект ",
        "тест ",
        "код ",
        "корпус ",
        "кеш ",
        "лог ",
        "демон ",
        "помощник ",
        "клавиатура ",
        "раскладка ",
        "раскладок ",
        "буфер ",
        "пробел ",
        "сейчас ",
        "потом ",
        "очень ",
        "нужно ",
        "хорошо ",
        "плохо ",
        "сделал ",
        "проверил ",
        "пишу ",
        "печатаю ",
        "быстро ",
        "медленно ",
        "нормально ",
        "отлично ",
        "давай ",
        "нет ",
        "вот ",
        "это ",
        "как ",
        "что ",
        "если ",
        "тогда ",
        "тут ",
        "там ",
        "уже ",
        "ещё ",
        "не ",
        "ни ",
        "хо хо ",
        "ха ха ",
        "CPU ",
        "LLM ",
        "API ",
        "МГУ ",
        "README ",
        "GitHub ",
        "WeChat ",
        "hello ",
        "world ",
        "cargo ",
        "Rust ",
        "GNOME ",
        "Wayland ",
        "Ollama ",
        "Qwen ",
        "BitNet ",
        "smollm ",
        "conecargo.ru ",
        "test@example.com ",
        "https://example.com ",
        "123 ",
        "7 ",
        "b/ ",
        "и. ",
        "в магазин ",
        "в вот ",
        "машина ",
        "магазин ",
        "тестами ",
        "словами ",
        "вариантами ",
        "схеме таможенник ",
        "схема таможженик ",
        "пошли ",
        "пошли в ",
        "ни фига ",
        "не фига ",
        "как говорится ",
        "ну что же ",
    ];

    for input in should_keep {
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
    let cases = [
        ("перейти b", "gthtqnb b", "перейти в"),
        ("b ghjcnj", "и просто", "в просто"),
        ("слово b ", "слово и ", "слово в "),
        ("b vfufpby ", "и магазин ", "в магазин "),
        ("b djn", "и вот", "в вот"),
        (
            "посмотри я double b",
            "gjcvjnhb z вщгиду и",
            "посмотри я double и",
        ),
    ];

    for (original, target, expected) in cases {
        assert_eq!(
            apply_auto_replace(original, target),
            Some(expected.to_string()),
            "original={original:?} target={target:?}"
        );
    }
}

#[test]
fn replaces_visual_b_inside_russian_context() {
    assert_eq!(
        apply_auto_replace("перейти b", "gthtqnb b"),
        Some("перейти в".to_string())
    );
    assert_eq!(
        apply_auto_replace("b ghjcnj", "и просто"),
        Some("в просто".to_string())
    );
    assert_eq!(
        apply_auto_replace("слово b ", "слово и "),
        Some("слово в ".to_string())
    );
}

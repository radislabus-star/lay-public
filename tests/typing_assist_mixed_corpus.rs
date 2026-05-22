use lay::config::{
    default_typing_assist_pipeline, typing_assist_pipeline_for_policy, CorrectionSafety,
    TypingAssistRuleConfig,
};
use lay::dict::{convert, Direction};
use lay::typing_assist::{
    apply_typing_assist_with_pipeline, split_edge_whitespace, split_ws_segments,
};
use lay::typing_context::{should_enable_ascii_to_ru_layout, typing_assist_pipeline_for_context};

fn apply_typing_assist_to_tail(
    text: &str,
    allow_layout_auto: bool,
    pipeline: &[TypingAssistRuleConfig],
) -> Option<String> {
    let context_pipeline =
        typing_assist_pipeline_for_context(true, CorrectionSafety::Normal, pipeline, text);
    apply_typing_assist_with_pipeline(text, allow_layout_auto, &context_pipeline).or_else(|| {
        let (leading, core, trailing) = split_edge_whitespace(text);
        let segments = split_ws_segments(core);
        if segments.len() < 3 {
            return None;
        }

        for word_count in [2, 1] {
            let mut suffix_start = core.len();
            let mut non_ws_seen = 0;
            for (segment, is_ws) in segments.iter().rev() {
                suffix_start -= segment.len();
                if !is_ws {
                    non_ws_seen += 1;
                    if non_ws_seen == word_count {
                        break;
                    }
                }
            }
            if non_ws_seen != word_count {
                continue;
            }

            let suffix = &core[suffix_start..];
            if let Some(replacement) =
                apply_typing_assist_with_pipeline(suffix, allow_layout_auto, &context_pipeline)
            {
                let mut out = String::with_capacity(text.len().max(replacement.len()));
                out.push_str(leading);
                out.push_str(&core[..suffix_start]);
                out.push_str(&replacement);
                out.push_str(trailing);
                if out != text {
                    return Some(out);
                }
            }
        }

        None
    })
}

fn simulate_space_triggered_typing_assist(input: &str, allow_layout_auto: bool) -> String {
    simulate_space_triggered_typing_assist_with_pipeline(
        input,
        allow_layout_auto,
        &default_typing_assist_pipeline(),
    )
}

fn simulate_space_triggered_typing_assist_with_pipeline(
    input: &str,
    allow_layout_auto: bool,
    pipeline: &[TypingAssistRuleConfig],
) -> String {
    let mut text = String::new();
    for ch in input.chars() {
        text.push(ch);
        if ch.is_whitespace() {
            if let Some(next) = apply_typing_assist_to_tail(&text, allow_layout_auto, pipeline) {
                text = next;
            }
        }
    }
    text
}

fn ru_text_typed_in_us_layout(text: &str) -> String {
    convert(text, Direction::Ru2Us)
}

fn en_text_typed_in_ru_layout(text: &str) -> String {
    convert(text, Direction::Us2Ru)
}

#[test]
fn forum_like_mixed_sentences_preserve_spaces_and_terms() {
    let cases = [
        (
            "сегодня проверяю git status и потом njkmrj тест ",
            "сегодня проверяю git status и потом только тест ",
        ),
        (
            "можно открыть Windows на NTFS и написать Lfdfq дальше ",
            "можно открыть Windows на NTFS и написать Давай дальше ",
        ),
        (
            "в терминале еукьштфд работает рядом с API JSON ",
            "в терминале terminal работает рядом с API JSON ",
        ),
        (
            "я смотрю wi-fi и double b прямо в тексте ",
            "я смотрю wi-fi и double и прямо в тексте ",
        ),
        (
            "это очнеь простой тест для Chrome и GNOME ",
            "это очень простой тест для Chrome и GNOME ",
        ),
        (
            "тут я вно вижу что good test должен остаться ",
            "тут явно вижу что good test должен остаться ",
        ),
        ("ОБYJDB CRBK lay ", "ОБНОВИ CRBK lay "),
        ("я ghbdtn и потом ckjdf ", "я привет и потом слова "),
    ];

    for (input, expected) in cases {
        assert_eq!(
            simulate_space_triggered_typing_assist(input, true),
            expected,
            "input={input:?}"
        );
    }
}

#[test]
fn dynamic_context_allows_ascii_to_ru_in_russian_sentence() {
    let pipeline = default_typing_assist_pipeline();
    assert!(should_enable_ascii_to_ru_layout("проверяю Lfdfq "));
    let context_pipeline = typing_assist_pipeline_for_context(
        true,
        CorrectionSafety::Normal,
        &pipeline,
        "проверяю Lfdfq ",
    );
    assert!(context_pipeline
        .iter()
        .find(|rule| rule.id == "contextual_layout_en_to_ru")
        .is_some_and(|rule| rule.enabled));
    assert_eq!(
        apply_typing_assist_with_pipeline("Lfdfq", true, &context_pipeline),
        Some("Давай".to_string())
    );
    assert_eq!(
        apply_typing_assist_to_tail("'nj ", true, &pipeline),
        Some("это ".to_string())
    );
    assert_eq!(
        apply_typing_assist_to_tail("проверяю Lfdfq ", true, &pipeline),
        Some("проверяю Давай ".to_string())
    );
    assert_eq!(
        apply_typing_assist_to_tail("я ghbdtn ", true, &pipeline),
        Some("я привет ".to_string())
    );
    assert_eq!(
        apply_typing_assist_to_tail("пишу 'nj ", true, &pipeline),
        Some("пишу это ".to_string())
    );
    assert_eq!(
        apply_typing_assist_to_tail("worked 'nj ", true, &pipeline),
        Some("worked это ".to_string())
    );
    assert_eq!(
        apply_typing_assist_to_tail("status; 'nj ", true, &pipeline),
        None
    );
    assert_eq!(
        apply_typing_assist_to_tail("good ghbdtn ", true, &pipeline),
        None
    );
}

#[test]
fn alternating_layout_sentences_fix_every_second_word() {
    let cases = [
        (
            "сегодня ghbdtn потом ашду дальше ckjdf снова еукьштфд здесь 'nj ",
            "сегодня привет потом file дальше слова снова terminal здесь это ",
        ),
        (
            "проверяю Lfdfq и цщкдв затем hf,jnftn рядом ашдуы будет vjue ",
            "проверяю Давай и world затем работает рядом files будет могу ",
        ),
        (
            "можно njkmrj открыть пщщв потом ytn рядом еукьштфд теперь 'nj ",
            "можно только открыть good потом нет рядом terminal теперь это ",
        ),
        (
            "тут ckjdf и ашду потом ltkf рядом кгы дальше dctulf ",
            "тут слова и file потом дела рядом rus дальше всегда ",
        ),
    ];

    for (input, expected) in cases {
        assert_eq!(
            simulate_space_triggered_typing_assist(input, true),
            expected,
            "input={input:?}"
        );
    }
}

#[test]
fn glued_pronoun_phrase_is_split_without_daemon_rules() {
    let pipeline = default_typing_assist_pipeline();

    assert_eq!(
        apply_typing_assist_with_pipeline("онаубыточная ", false, &pipeline),
        Some("она убыточная ".to_string())
    );
    assert_eq!(
        apply_typing_assist_with_pipeline("ониготовы ", false, &pipeline),
        Some("они готовы ".to_string())
    );
}

#[test]
fn glued_phrase_split_can_repair_one_safe_internal_part() {
    let pipeline = typing_assist_pipeline_for_policy(
        true,
        CorrectionSafety::Normal,
        &default_typing_assist_pipeline(),
    );
    let expected_left = "проблема";
    let expected_right = "тут";
    let input_left = expected_left.replacen('о', "", 1);
    let input = format!("{input_left}{expected_right} ");

    assert_eq!(
        apply_typing_assist_with_pipeline(&input, false, &pipeline),
        Some(format!("{expected_left} {expected_right} "))
    );
    assert_eq!(
        apply_typing_assist_with_pipeline("тоесамое ", false, &pipeline),
        Some("тоже самое ".to_string())
    );
}

#[test]
fn live_user_sentences_keep_spaces_after_typing_assist() {
    let cases = [
        (
            "я пишу мои слова мои предложения чтобыточно проверить дальше ",
            "я пишу мои слова мои предложения чтобы точно проверить дальше ",
        ),
        (
            "нужно проверить когдая пишу быстро ",
            "нужно проверить когда я пишу быстро ",
        ),
        (
            "сейчас думаю тако й пример работает ",
            "сейчас думаю такой пример работает ",
        ),
        (
            "я тут вижу что пробел не должен липнуть ",
            "я тут вижу что пробел не должен липнуть ",
        ),
        (
            "ошибка в наборе но не ты должен остаться ",
            "ошибка в наборе но не ты должен остаться ",
        ),
        (
            "изменю параметры и проверю что слова не склеиваются ",
            "изменю параметры и проверю что слова не склеиваются ",
        ),
        (
            "пишу про сою и проверяю что предлог не липнет ",
            "пишу про сою и проверяю что предлог не липнет ",
        ),
        (
            "за нас уже по любому и дальше пишем ",
            "за нас уже по любому и дальше пишем ",
        ),
        (
            "за нас уже поо любому и дальше пишем ",
            "за нас уже по любому и дальше пишем ",
        ),
        (
            "мы должны помнить что у насесть право на информацию ",
            "мы должны помнить что у нас есть право на информацию ",
        ),
        (
            "вот какпроверка автозамены выглядит сейчас ",
            "вот как проверка автозамены выглядит сейчас ",
        ),
        (
            "сейчас тожесамое проверяю быстро ",
            "сейчас тоже самое проверяю быстро ",
        ),
        (
            "проверяю вотэто и самоетоже быстро ",
            "проверяю вот это и самое тоже быстро ",
        ),
        (
            "я не буду за вас янебудузавас дальше ",
            "я не буду за вас я не буду за вас дальше ",
        ),
        (
            "проверяю тоже самое янебуду дальше ",
            "проверяю тоже самое я не буду дальше ",
        ),
        (
            "проверяю тоже самое самое тоже янебудузавастожесамое дальше ",
            "проверяю тоже самое самое тоже я не буду за вас тоже самое дальше ",
        ),
        (
            "проверяю пока ненаучишьсярезатьслова дальше ",
            "проверяю пока не научишься резать слова дальше ",
        ),
        (
            "пишем тест я язык НАПИШИ дальше ",
            "пишем тест я язык НАПИШИ дальше ",
        ),
    ];

    for (input, expected) in cases {
        let got = simulate_space_triggered_typing_assist(input, true);
        assert_eq!(got, expected, "input={input:?}");
        assert!(!got.contains("чтобыточно"));
        assert!(!got.contains("когдая"));
        assert!(!got.contains("тако й"));
        assert!(!got.contains("нонеты"));
        assert!(!got.contains("ноне ты"));
        assert!(!got.contains("изменюпараметры"));
        assert!(!got.contains("просою"));
        assert!(!got.contains("какпроверка"));
        assert!(!got.contains("тожесамое"));
        assert!(!got.contains("янебудузавас"));
    }
}

#[test]
fn live_journal_false_positive_candidates_are_rejected() {
    let normal_auto_pipeline = typing_assist_pipeline_for_policy(
        true,
        CorrectionSafety::Normal,
        &default_typing_assist_pipeline(),
    );

    assert_eq!(
        apply_typing_assist_with_pipeline("компаниия ", false, &normal_auto_pipeline),
        Some("компания ".to_string()),
        "duplicated-letter typo must not be transposed into a word plus orphan function suffix"
    );
    assert_eq!(
        apply_typing_assist_with_pipeline("елать ", false, &normal_auto_pipeline),
        None,
        "single-letter substitution must not guess a different dictionary word"
    );
    assert_eq!(
        apply_typing_assist_with_pipeline("таможе ", false, &normal_auto_pipeline),
        None,
        "missing-letter rule must not append a final consonant to an unfinished-looking word"
    );
    assert_eq!(
        apply_typing_assist_with_pipeline("перекупа ", false, &normal_auto_pipeline),
        None,
        "missing-letter rule must not append a final consonant to a plausible standalone form"
    );
    assert_eq!(
        apply_typing_assist_with_pipeline("никуму ", true, &normal_auto_pipeline),
        None,
        "normal live autocorrect should not use vowel-confusion guesses"
    );

    let experimental_auto_pipeline = typing_assist_pipeline_for_policy(
        true,
        CorrectionSafety::Experimental,
        &default_typing_assist_pipeline(),
    );
    assert_eq!(
        apply_typing_assist_with_pipeline("никуму ", true, &experimental_auto_pipeline),
        Some("никому ".to_string()),
        "experimental mode may use vowel-confusion guesses"
    );
}

#[test]
fn normal_autocorrect_keeps_safe_rules_and_rejects_aggressive_guesses() {
    let normal_auto_pipeline = typing_assist_pipeline_for_policy(
        true,
        CorrectionSafety::Normal,
        &default_typing_assist_pipeline(),
    );

    let safe_cases = [
        ("кторое ", Some("которое ")),
        ("очнеь ", Some("очень ")),
        ("рабоатет ", Some("работает ")),
        ("перпаратов ", Some("препаратов ")),
        ("ОФФИЦИАЛЬНОМ ", Some("ОФИЦИАЛЬНОМ ")),
    ];

    for (input, expected) in safe_cases {
        assert_eq!(
            apply_typing_assist_with_pipeline(input, false, &normal_auto_pipeline),
            expected.map(str::to_string),
            "safe normal autocorrect case failed: {input:?}"
        );
    }

    for input in ["робило ", "банный ", "поения ", "страдает ", "никуму "]
    {
        assert_eq!(
            apply_typing_assist_with_pipeline(input, true, &normal_auto_pipeline),
            None,
            "normal autocorrect was too aggressive for {input:?}"
        );
    }
}

#[test]
fn one_letter_function_words_do_not_steal_next_word_prefix() {
    let cases = [
        "я язык",
        "я явно",
        "в версии",
        "в воде",
        "и идея",
        "и инструкция",
        "к команде",
        "с системой",
        "у утилиты",
        "о окне",
    ];

    for phrase in cases {
        let input = format!("проверяю {phrase} дальше ");
        assert_eq!(
            simulate_space_triggered_typing_assist(&input, true),
            input,
            "one-letter function word stole next prefix: {input:?}"
        );
    }
}

#[test]
fn forum_like_mixed_matrix_autofixes_contextual_layout_words_and_keeps_boundaries() {
    let prefixes = [
        "проверяю",
        "открываю",
        "смотрю",
        "чиню",
        "пишу",
        "сравниваю",
        "тестирую",
        "запускаю",
        "обновляю",
        "собираю",
        "настраиваю",
        "публикую",
    ];
    let english_terms = [
        "git", "status", "Windows", "NTFS", "wi-fi", "API", "JSON", "Linux", "Chrome", "GNOME",
    ];
    let layout_words = [
        ("njkmrj", Some("только")),
        ("vjue", Some("могу")),
        ("yt", None),
        ("hf,jnftn", Some("работает")),
        ("'nj", Some("это")),
        ("ашдуы", Some("files")),
        ("еукьштфд", Some("terminal")),
        ("кгы", Some("rus")),
        ("утп", Some("eng")),
    ];

    let mut checked = 0usize;
    for (idx, prefix) in prefixes.iter().enumerate() {
        for (typed, expected) in layout_words {
            let term = english_terms[(idx + checked) % english_terms.len()];
            let input = format!("я {prefix} {term} и пишу {typed} дальше ");
            let got = simulate_space_triggered_typing_assist(&input, true);

            assert!(
                got.contains(&format!(" {term} ")),
                "english term boundary lost: input={input:?} got={got:?}"
            );
            if let Some(expected) = expected {
                assert!(
                    got.contains(&format!(" {expected} ")),
                    "safe RU->EN layout word was not auto-fixed: input={input:?} got={got:?}"
                );
                assert!(
                    !got.contains(&format!("{term}{expected}")),
                    "words were glued after replacement: input={input:?} got={got:?}"
                );
                assert!(
                    !got.contains(&format!("{expected}дальше")),
                    "tail was glued to next word: input={input:?} got={got:?}"
                );
            } else {
                assert!(got.contains(&format!(" {typed} ")));
            }
            checked += 1;
        }
    }

    assert!(checked >= 100, "checked={checked}");
}

#[test]
fn forum_like_clean_mixed_sentences_do_not_get_rewritten() {
    let cases = [
        "я проверяю git status и Windows NTFS ",
        "тут good test рядом с русским текстом ",
        "wi-fi работает и API JSON остаются как есть ",
        "Chrome GNOME Linux file mode code data ",
        "это нормальная русская фраза без правки ",
    ];

    for input in cases {
        assert_eq!(
            simulate_space_triggered_typing_assist(input, true),
            input,
            "clean sentence was changed: {input:?}"
        );
    }
}

#[test]
fn normal_mode_does_not_autocorrect_full_opposite_layout_russian_sentence() {
    let cases = [
        "только, могу? не; работает 100% это нормально! ",
        "Давай: это тест? работает; можно 50% дальше. ",
    ];

    for expected in cases {
        let input = ru_text_typed_in_us_layout(expected);
        assert_eq!(
            simulate_space_triggered_typing_assist(&input, true),
            input,
            "input={input:?}"
        );
    }
}

#[test]
fn normal_mode_autocorrects_confident_english_typed_in_ru_layout() {
    let cases = [
        "git status; echo files 100% ",
        "terminal files? rus; eng 50% ",
    ];

    for expected in cases {
        let input = en_text_typed_in_ru_layout(expected);
        assert_eq!(
            simulate_space_triggered_typing_assist(&input, true),
            expected,
            "input={input:?}"
        );
    }
}

#[test]
fn mixed_sentence_autofixes_ru_to_en_but_leaves_en_to_ru_for_manual_double_shift() {
    let input = format!(
        "{} {} {} {} {} ",
        ru_text_typed_in_us_layout("только,"),
        en_text_typed_in_ru_layout("git"),
        ru_text_typed_in_us_layout("работает?"),
        en_text_typed_in_ru_layout("status;"),
        ru_text_typed_in_us_layout("это")
    );

    assert_eq!(
        simulate_space_triggered_typing_assist(&input, true),
        "njkmrj? git hf,jnftn& status; 'nj "
    );
}

#[test]
fn clean_shell_like_commands_and_symbols_are_not_rewritten() {
    let cases = [
        "git status; echo hello? files 100% ",
        "ls -la | grep test && echo 100% ",
        "curl -I https://example.com/path?x=1; echo ok ",
        "command -f file.txt && command -r repo && command -c config ",
        "grep --color=auto -n test file.txt ",
        "git checkout -b feature && git status ",
        "git branch -D old && git checkout -b new ",
        "git checkout -b # create branch ",
    ];

    for input in cases {
        assert_eq!(
            simulate_space_triggered_typing_assist(input, true),
            input,
            "command-like text was changed: {input:?}"
        );
    }
}

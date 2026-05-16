use lay::config::{default_typing_assist_pipeline, TypingAssistRuleConfig};
use lay::dict::{convert, Direction};
use lay::typing_assist::{
    apply_typing_assist_with_pipeline, split_edge_whitespace, split_ws_segments,
};

fn apply_typing_assist_to_tail(
    text: &str,
    allow_layout_auto: bool,
    pipeline: &[TypingAssistRuleConfig],
) -> Option<String> {
    apply_typing_assist_with_pipeline(text, allow_layout_auto, pipeline).or_else(|| {
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
                apply_typing_assist_with_pipeline(suffix, allow_layout_auto, pipeline)
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
    let pipeline = default_typing_assist_pipeline();
    let mut text = String::new();
    for ch in input.chars() {
        text.push(ch);
        if ch.is_whitespace() {
            if let Some(next) = apply_typing_assist_to_tail(&text, allow_layout_auto, &pipeline) {
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
        ("ОБYJDB CRBK lay ", "ОБНОВИ СКИЛ lay "),
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
fn forum_like_mixed_matrix_keeps_boundaries_after_layout_autofix() {
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
        ("njkmrj", "только"),
        ("vjue", "могу"),
        ("yt", "не"),
        ("hf,jnftn", "работает"),
        ("'nj", "это"),
        ("ашдуы", "files"),
        ("еукьштфд", "terminal"),
        ("кгы", "rus"),
        ("утп", "eng"),
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
            assert!(
                got.contains(&format!(" {expected} ")),
                "layout word was not fixed: input={input:?} got={got:?}"
            );
            assert!(
                !got.contains(&format!("{term}{expected}")),
                "words were glued after replacement: input={input:?} got={got:?}"
            );
            assert!(
                !got.contains(&format!("{expected}дальше")),
                "tail was glued to next word: input={input:?} got={got:?}"
            );
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
fn opposite_layout_russian_sentence_with_symbols_is_corrected_word_by_word() {
    let cases = [
        "только, могу? не; работает 100% это нормально! ",
        "Давай: это тест? работает; можно 50% дальше. ",
    ];

    for expected in cases {
        let input = ru_text_typed_in_us_layout(expected);
        assert_eq!(
            simulate_space_triggered_typing_assist(&input, true),
            expected,
            "input={input:?}"
        );
    }
}

#[test]
fn opposite_layout_english_commands_with_symbols_are_corrected_word_by_word() {
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
fn every_word_can_be_opposite_layout_inside_one_mixed_sentence() {
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
        "только, git работает? status; это "
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

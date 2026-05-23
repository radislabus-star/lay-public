use lay::config::{
    default_typing_assist_pipeline, typing_assist_pipeline_for_policy, CorrectionSafety,
    TypingAssistRuleConfig,
};
use lay::dict::{convert, Direction};
use lay::typing_assist::{
    apply_typing_assist_with_pipeline, split_edge_whitespace, split_ws_segments,
};
use lay::typing_context::{should_enable_ascii_to_ru_layout, typing_assist_pipeline_for_context};

const FORUM_MIXED_CASES: &str = include_str!("fixtures/typing_assist_forum_mixed.tsv");
const DYNAMIC_TAIL_CASES: &str = include_str!("fixtures/typing_assist_dynamic_tail.tsv");
const DYNAMIC_TAIL_NONE_CASES: &str = include_str!("fixtures/typing_assist_dynamic_tail_none.txt");
const ALTERNATING_CASES: &str = include_str!("fixtures/typing_assist_alternating.tsv");
const LIVE_SPACING_CASES: &str = include_str!("fixtures/typing_assist_live_spacing.tsv");
const CLEAN_MIXED_CASES: &str = include_str!("fixtures/typing_assist_clean_mixed.txt");
const FULL_OPPOSITE_RU_CASES: &str = include_str!("fixtures/typing_assist_full_opposite_ru.txt");
const CONFIDENT_EN_CASES: &str = include_str!("fixtures/typing_assist_confident_en.txt");
const FUNCTION_BOUNDARY_CASES: &str =
    include_str!("fixtures/typing_assist_function_boundaries.txt");
const FORBIDDEN_FRAGMENTS: &str = include_str!("fixtures/typing_assist_forbidden_fragments.txt");
const CONTEXT_ENABLED_CASES: &str = include_str!("fixtures/typing_context_enabled.txt");

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

fn assert_preserves_committed_space_boundary(input: &str, expected: &str) {
    let got = simulate_space_triggered_typing_assist(input, true);
    assert_eq!(got, expected, "input={input:?}");
    assert_eq!(
        input.ends_with(char::is_whitespace),
        got.ends_with(char::is_whitespace),
        "space trigger boundary changed: input={input:?} got={got:?}"
    );
}

fn ru_text_typed_in_us_layout(text: &str) -> String {
    convert(text, Direction::Ru2Us)
}

fn en_text_typed_in_ru_layout(text: &str) -> String {
    convert(text, Direction::Us2Ru)
}

fn fixture_cases(data: &'static str) -> impl Iterator<Item = (String, String)> {
    fixture_rows(data).map(|line| {
        let (input, expected) = line.split_once('\t').expect("fixture row must be TSV");
        (decode_fixture_field(input), decode_fixture_field(expected))
    })
}

fn fixture_lines(data: &'static str) -> impl Iterator<Item = String> {
    fixture_rows(data).map(decode_fixture_field)
}

fn fixture_rows(data: &'static str) -> impl Iterator<Item = &'static str> {
    data.lines()
        .filter(|line| !line.trim().is_empty() && !line.starts_with('#'))
}

fn decode_fixture_field(value: &str) -> String {
    value.replace("\\s", " ")
}

#[test]
fn forum_like_mixed_sentences_preserve_spaces_and_terms() {
    for (input, expected) in fixture_cases(FORUM_MIXED_CASES) {
        assert_preserves_committed_space_boundary(&input, &expected);
    }
}

#[test]
fn dynamic_context_allows_ascii_to_ru_in_russian_sentence() {
    let pipeline = default_typing_assist_pipeline();
    let context = fixture_lines(CONTEXT_ENABLED_CASES)
        .find(|line| line.contains("Lfdfq"))
        .expect("context fixture");
    assert!(should_enable_ascii_to_ru_layout(&context));
    let context_pipeline =
        typing_assist_pipeline_for_context(true, CorrectionSafety::Normal, &pipeline, &context);
    assert!(context_pipeline
        .iter()
        .find(|rule| rule.id == "contextual_layout_en_to_ru")
        .is_some_and(|rule| rule.enabled));
    assert_eq!(
        apply_typing_assist_with_pipeline("Lfdfq", true, &context_pipeline),
        Some("Давай".to_string())
    );
    for (input, expected) in fixture_cases(DYNAMIC_TAIL_CASES) {
        assert_eq!(
            apply_typing_assist_to_tail(&input, true, &pipeline),
            Some(expected)
        );
    }
    for input in fixture_lines(DYNAMIC_TAIL_NONE_CASES) {
        assert_eq!(apply_typing_assist_to_tail(&input, true, &pipeline), None);
    }
}

#[test]
fn alternating_layout_sentences_fix_every_second_word() {
    for (input, expected) in fixture_cases(ALTERNATING_CASES) {
        assert_eq!(
            simulate_space_triggered_typing_assist(&input, true),
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
    for (input, expected) in fixture_cases(LIVE_SPACING_CASES) {
        let got = simulate_space_triggered_typing_assist(&input, true);
        assert_eq!(got, expected, "input={input:?}");
        assert_eq!(
            input.ends_with(char::is_whitespace),
            got.ends_with(char::is_whitespace),
            "space trigger boundary changed: input={input:?} got={got:?}"
        );
        for fragment in fixture_lines(FORBIDDEN_FRAGMENTS) {
            assert!(
                !got.contains(&fragment),
                "fragment={fragment:?} got={got:?}"
            );
        }
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
    assert_eq!(
        apply_typing_assist_with_pipeline("котовые ", false, &normal_auto_pipeline),
        None,
        "known Russian adjective form must not be changed into a different known word"
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
    for phrase in fixture_lines(FUNCTION_BOUNDARY_CASES) {
        let input = format!("проверяю {phrase} дальше ");
        assert_preserves_committed_space_boundary(&input, &input);
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
    for input in fixture_lines(CLEAN_MIXED_CASES) {
        assert_preserves_committed_space_boundary(&input, &input);
    }
}

#[test]
fn normal_mode_does_not_autocorrect_full_opposite_layout_russian_sentence() {
    for expected in fixture_lines(FULL_OPPOSITE_RU_CASES) {
        let input = ru_text_typed_in_us_layout(&expected);
        assert_eq!(
            simulate_space_triggered_typing_assist(&input, true),
            input,
            "input={input:?}"
        );
    }
}

#[test]
fn normal_mode_autocorrects_confident_english_typed_in_ru_layout() {
    for expected in fixture_lines(CONFIDENT_EN_CASES) {
        let input = en_text_typed_in_ru_layout(&expected);
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

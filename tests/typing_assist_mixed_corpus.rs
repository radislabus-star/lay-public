mod common;

use lay::config::{
    default_typing_assist_pipeline, typing_assist_pipeline_for_policy, CorrectionSafety,
};
use lay::dict::{convert, Direction};
use lay::typing_assist::{apply_typing_assist_with_pipeline, explain_typing_assist_with_pipeline};
use lay::typing_context::{should_enable_ascii_to_ru_layout, typing_assist_pipeline_for_context};

use common::{
    apply_typing_assist_to_tail, fixture_cases, fixture_cols, fixture_lines,
    simulate_space_triggered_typing_assist,
};

const FORUM_MIXED_CASES: &str = include_str!("fixtures/typing_assist_forum_mixed.tsv");
const DYNAMIC_TAIL_CASES: &str = include_str!("fixtures/typing_assist_dynamic_tail.tsv");
const DYNAMIC_TAIL_NONE_CASES: &str = include_str!("fixtures/typing_assist_dynamic_tail_none.txt");
const ALTERNATING_CASES: &str = include_str!("fixtures/typing_assist_alternating.tsv");
const BETA_ALTERNATING_CASES: &str = include_str!("fixtures/typing_assist_beta_alternating.tsv");
const LIVE_SPACING_CASES: &str = include_str!("fixtures/typing_assist_live_spacing.tsv");
const CLEAN_MIXED_CASES: &str = include_str!("fixtures/typing_assist_clean_mixed.txt");
const FULL_OPPOSITE_RU_CASES: &str = include_str!("fixtures/typing_assist_full_opposite_ru.txt");
const CONFIDENT_EN_CASES: &str = include_str!("fixtures/typing_assist_confident_en.txt");
const FUNCTION_BOUNDARY_CASES: &str =
    include_str!("fixtures/typing_assist_function_boundaries.txt");
const FORBIDDEN_FRAGMENTS: &str = include_str!("fixtures/typing_assist_forbidden_fragments.txt");
const CONTEXT_ENABLED_CASES: &str = include_str!("fixtures/typing_context_enabled.txt");
const NORMAL_SAFE_CASES: &str = include_str!("fixtures/typing_assist_normal_safe.tsv");
const NORMAL_REJECT_CASES: &str = include_str!("fixtures/typing_assist_normal_reject.txt");
const MIXED_MATRIX_PREFIXES: &str =
    include_str!("fixtures/typing_assist_mixed_matrix_prefixes.txt");
const MIXED_MATRIX_TERMS: &str = include_str!("fixtures/typing_assist_mixed_matrix_terms.txt");
const MIXED_MATRIX_LAYOUT_WORDS: &str =
    include_str!("fixtures/typing_assist_mixed_matrix_layout_words.tsv");
const SHELL_KEEP_CASES: &str = include_str!("fixtures/typing_assist_shell_keep.txt");
const CLI_COMMAND_CASES: &str = include_str!("fixtures/typing_assist_cli_commands.txt");
const POLICY_CASES: &str = include_str!("fixtures/typing_assist_policy_cases.tsv");

#[derive(Clone, Copy)]
enum ExpectedAssist<'a> {
    Some(&'a str),
    None,
}

fn policy_pipeline(safety: CorrectionSafety) -> Vec<lay::config::TypingAssistRuleConfig> {
    typing_assist_pipeline_for_policy(true, safety, &default_typing_assist_pipeline())
}

fn assert_policy_case(
    pipeline: &[lay::config::TypingAssistRuleConfig],
    input: &str,
    allow_layout_auto: bool,
    expected: ExpectedAssist<'_>,
    message: &str,
) {
    let got = apply_typing_assist_with_pipeline(input, allow_layout_auto, pipeline);
    match expected {
        ExpectedAssist::Some(expected) => {
            assert_eq!(got, Some(expected.to_string()), "{message}");
        }
        ExpectedAssist::None => {
            assert_eq!(got, None, "{message}");
        }
    }
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
fn experimental_context_accepts_plain_ascii_to_ru_layout_words() {
    let pipeline = typing_assist_pipeline_for_context(
        true,
        CorrectionSafety::Experimental,
        &default_typing_assist_pipeline(),
        "",
    );
    assert!(pipeline
        .iter()
        .find(|rule| rule.id == "experimental_layout_en_to_ru")
        .is_some_and(|rule| rule.enabled));
    assert_eq!(
        apply_typing_assist_with_pipeline("djn ", true, &pipeline),
        Some("вот ".to_string())
    );
    assert_eq!(
        apply_typing_assist_with_pipeline("z ", true, &pipeline),
        None
    );
    assert_eq!(
        apply_typing_assist_with_pipeline("b ", true, &pipeline),
        None
    );

    let normal_pipeline = typing_assist_pipeline_for_context(
        true,
        CorrectionSafety::Normal,
        &default_typing_assist_pipeline(),
        "",
    );
    assert_eq!(
        apply_typing_assist_with_pipeline("z ", true, &normal_pipeline),
        None
    );
    assert_eq!(
        apply_typing_assist_with_pipeline("b ", true, &normal_pipeline),
        None
    );
}

#[test]
fn contextual_ru_conjunction_i_requires_phrase_support_on_both_sides() {
    for (input, expected) in [
        ("чай b кофе ", "чай и кофе "),
        ("пишу b проверяю ", "пишу и проверяю "),
        ("быстро b удобно ", "быстро и удобно "),
        ("проверил file b папку ", "проверил file и папку "),
        ("lay b справится ", "lay и справится "),
    ] {
        assert_eq!(
            simulate_space_triggered_typing_assist(input, true),
            expected,
            "input={input:?}"
        );
    }

    for input in [
        "b ",
        "b b ",
        "c ",
        "wave c ",
        "wave b ",
        "wave b вот ",
        "api b json ",
        "git checkout -b test ",
        "double b прямо ",
    ] {
        assert_eq!(
            simulate_space_triggered_typing_assist(input, true),
            input,
            "input={input:?}"
        );
    }
}

#[test]
fn contextual_ru_preposition_v_requires_russian_context_and_technical_anchor() {
    for (input, expected) in [
        ("читай cola d wechat ", "читай cola в wechat "),
        ("читай d wechat ", "читай в wechat "),
        ("проверил file d папку ", "проверил file в папку "),
        ("пиши d html ", "пиши в html "),
    ] {
        assert_eq!(
            simulate_space_triggered_typing_assist(input, true),
            expected,
            "input={input:?}"
        );
    }

    for input in [
        "d ",
        "читай d ",
        "wave d wechat ",
        "api d json ",
        "git checkout -d test ",
        "vitamin d дефицит ",
        "он gpu d html ",
        "читай d wave ",
        "double d прямо ",
    ] {
        assert_eq!(
            simulate_space_triggered_typing_assist(input, true),
            input,
            "input={input:?}"
        );
    }
}

#[test]
fn confident_en_to_ru_layout_words_use_fast_path_without_rewriting_english() {
    let pipeline = typing_assist_pipeline_for_context(
        true,
        CorrectionSafety::Experimental,
        &default_typing_assist_pipeline(),
        "",
    );

    for (input, expected) in [("vj;tn ", "может "), ("djn ", "вот "), ("cegth ", "супер ")]
    {
        let explanation = explain_typing_assist_with_pipeline(input, true, &pipeline);
        assert_eq!(
            explanation.output.as_deref(),
            Some(expected),
            "input={input:?}"
        );
        assert_eq!(
            explanation
                .chosen
                .as_ref()
                .map(|candidate| candidate.rule_id.as_str()),
            Some("fast_layout_en_to_ru"),
            "input={input:?}"
        );
    }

    for input in ["word ", "file ", "api ", "git "] {
        let explanation = explain_typing_assist_with_pipeline(input, true, &pipeline);
        assert_ne!(
            explanation
                .chosen
                .as_ref()
                .map(|candidate| candidate.rule_id.as_str()),
            Some("fast_layout_en_to_ru"),
            "known English token must not use fast EN->RU path: {input:?}"
        );
        assert_eq!(explanation.output, None, "input={input:?}");
    }

    let one_letter = explain_typing_assist_with_pipeline("d ", true, &pipeline);
    assert_ne!(
        one_letter
            .chosen
            .as_ref()
            .map(|candidate| candidate.rule_id.as_str()),
        Some("fast_layout_en_to_ru")
    );
}

#[test]
fn experimental_context_accepts_plain_cyrillic_to_ascii_layout_words() {
    let pipeline = typing_assist_pipeline_for_context(
        true,
        CorrectionSafety::Experimental,
        &default_typing_assist_pipeline(),
        "",
    );
    assert!(pipeline
        .iter()
        .find(|rule| rule.id == "experimental_layout_ru_to_en")
        .is_some_and(|rule| rule.enabled));
    assert_eq!(
        apply_typing_assist_with_pipeline("щт ", true, &pipeline),
        Some("on ".to_string())
    );
    assert_eq!(
        apply_typing_assist_with_pipeline("щаа ", true, &pipeline),
        Some("off ".to_string())
    );
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
fn beta_alternating_layout_sentences_keep_language_boundaries() {
    for (input, expected) in fixture_cases(BETA_ALTERNATING_CASES) {
        let got = simulate_space_triggered_typing_assist(&input, true);
        assert_eq!(got, expected, "input={input:?}");
        assert_eq!(
            input.ends_with(char::is_whitespace),
            got.ends_with(char::is_whitespace),
            "space trigger boundary changed: input={input:?} got={got:?}"
        );
        assert_eq!(
            expected.split_whitespace().count(),
            got.split_whitespace().count(),
            "word boundary count changed: input={input:?} got={got:?}"
        );
    }
}

#[test]
fn glued_pronoun_phrase_is_split_without_daemon_rules() {
    let pipeline = default_typing_assist_pipeline();

    for row in fixture_cols(POLICY_CASES) {
        if row[0] != "default_glued_pronoun" {
            continue;
        }
        assert_policy_case(
            &pipeline,
            &row[2],
            row[3] == "true",
            ExpectedAssist::Some(&row[4]),
            &row[5],
        );
    }
}

#[test]
fn glued_phrase_split_can_repair_one_safe_internal_part() {
    let pipeline = policy_pipeline(CorrectionSafety::Normal);

    for row in fixture_cols(POLICY_CASES) {
        if row[0] != "normal_glued_phrase" {
            continue;
        }
        assert_policy_case(
            &pipeline,
            &row[2],
            row[3] == "true",
            ExpectedAssist::Some(&row[4]),
            &row[5],
        );
    }
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
    for row in fixture_cols(POLICY_CASES) {
        if row[0] != "normal_false_positive" && row[0] != "experimental_allowed" {
            continue;
        }
        let safety = match row[1].as_str() {
            "normal" => CorrectionSafety::Normal,
            "experimental" => CorrectionSafety::Experimental,
            other => panic!("unknown safety fixture value {other:?}"),
        };
        let pipeline = policy_pipeline(safety);
        let expected = if row[4] == "None" {
            ExpectedAssist::None
        } else {
            ExpectedAssist::Some(&row[4])
        };
        assert_policy_case(&pipeline, &row[2], row[3] == "true", expected, &row[5]);
    }
}

#[test]
fn normal_autocorrect_keeps_safe_rules_and_rejects_aggressive_guesses() {
    let normal_auto_pipeline = typing_assist_pipeline_for_policy(
        true,
        CorrectionSafety::Normal,
        &default_typing_assist_pipeline(),
    );

    for (input, expected) in fixture_cases(NORMAL_SAFE_CASES) {
        assert_eq!(
            apply_typing_assist_with_pipeline(&input, false, &normal_auto_pipeline),
            Some(expected),
            "safe normal autocorrect case failed: {input:?}"
        );
    }

    for input in fixture_lines(NORMAL_REJECT_CASES) {
        assert_eq!(
            apply_typing_assist_with_pipeline(&input, true, &normal_auto_pipeline),
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
    let prefixes = fixture_lines(MIXED_MATRIX_PREFIXES).collect::<Vec<_>>();
    let english_terms = fixture_lines(MIXED_MATRIX_TERMS).collect::<Vec<_>>();
    let layout_words = fixture_cases(MIXED_MATRIX_LAYOUT_WORDS).collect::<Vec<_>>();

    let mut checked = 0usize;
    for (idx, prefix) in prefixes.iter().enumerate() {
        for (typed, expected) in &layout_words {
            let term = &english_terms[(idx + checked) % english_terms.len()];
            let input = format!("я {prefix} {term} и пишу {typed} дальше ");
            let got = simulate_space_triggered_typing_assist(&input, true);

            assert!(
                got.contains(&format!(" {term} ")),
                "english term boundary lost: input={input:?} got={got:?}"
            );
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
    for input in fixture_lines(SHELL_KEEP_CASES) {
        assert_eq!(
            simulate_space_triggered_typing_assist(&input, true),
            input,
            "command-like text was changed: {input:?}"
        );
    }
}

#[test]
fn cli_commands_stay_ascii_and_recover_from_ru_layout() {
    for command in fixture_lines(CLI_COMMAND_CASES) {
        let ascii = format!("{command} ");
        assert_eq!(
            simulate_space_triggered_typing_assist(&ascii, true),
            ascii,
            "ASCII CLI command was changed: {command:?}"
        );

        let typed_ru = convert(&command, Direction::Us2Ru);
        if typed_ru == command {
            continue;
        }
        assert_eq!(
            simulate_space_triggered_typing_assist(&format!("{typed_ru} "), true),
            ascii,
            "RU-layout CLI command was not restored: {command:?} typed={typed_ru:?}"
        );
    }
}

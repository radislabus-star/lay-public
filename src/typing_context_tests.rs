use crate::config::{
    default_typing_assist_pipeline, typing_assist_pipeline_for_policy, CorrectionSafety,
};
use crate::typing_context::{should_enable_ascii_to_ru_layout, typing_assist_pipeline_for_context};

const CONTEXT_ENABLED_CASES: &str = include_str!("../tests/fixtures/typing_context_enabled.txt");
const CONTEXT_DISABLED_CASES: &str = include_str!("../tests/fixtures/typing_context_disabled.txt");

fn fixture_lines(data: &'static str) -> impl Iterator<Item = String> {
    data.lines()
        .filter(|line| !line.trim().is_empty() && !line.starts_with('#'))
        .map(|line| line.replace("\\s", " "))
}

#[test]
fn russian_context_enables_ascii_to_ru_layout_rule() {
    let mut enabled_cases = fixture_lines(CONTEXT_ENABLED_CASES);
    let first_context = enabled_cases.next().expect("enabled context fixture");
    let pipeline = typing_assist_pipeline_for_context(
        true,
        CorrectionSafety::Normal,
        &default_typing_assist_pipeline(),
        &first_context,
    );

    assert!(pipeline
        .iter()
        .find(|rule| rule.id == "layout_en_to_ru")
        .is_some_and(|rule| !rule.enabled));
    assert!(pipeline
        .iter()
        .find(|rule| rule.id == "contextual_layout_en_to_ru")
        .is_some_and(|rule| rule.enabled));
    assert!(should_enable_ascii_to_ru_layout(&first_context));
    for context in enabled_cases {
        assert!(should_enable_ascii_to_ru_layout(&context), "{context:?}");
    }
}

#[test]
fn no_context_or_english_context_keeps_ascii_to_ru_disabled() {
    let base = typing_assist_pipeline_for_policy(
        true,
        CorrectionSafety::Normal,
        &default_typing_assist_pipeline(),
    );
    assert!(base
        .iter()
        .find(|rule| rule.id == "layout_en_to_ru")
        .is_some_and(|rule| !rule.enabled));

    for context in fixture_lines(CONTEXT_DISABLED_CASES) {
        assert!(
            !should_enable_ascii_to_ru_layout(&context),
            "context={context:?}"
        );
        let pipeline = typing_assist_pipeline_for_context(
            true,
            CorrectionSafety::Normal,
            &default_typing_assist_pipeline(),
            &context,
        );
        assert!(
            pipeline
                .iter()
                .all(|rule| rule.id != "contextual_layout_en_to_ru"),
            "context={context:?}"
        );
    }
}

#[test]
fn explicit_user_disabled_rule_stays_disabled() {
    let mut configured = default_typing_assist_pipeline();
    configured
        .iter_mut()
        .find(|rule| rule.id == "layout_en_to_ru")
        .expect("layout_en_to_ru rule")
        .enabled = false;

    let pipeline = typing_assist_pipeline_for_context(
        true,
        CorrectionSafety::Normal,
        &configured,
        "я ghbdtn ",
    );

    assert!(pipeline
        .iter()
        .all(|rule| rule.id != "contextual_layout_en_to_ru"));
}

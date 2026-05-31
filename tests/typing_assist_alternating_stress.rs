mod common;

use common::{fixture_tagged_cases, simulate_space_triggered_typing_assist};

const ALTERNATING_STRESS_CASES: &str =
    include_str!("fixtures/typing_assist_alternating_stress.tsv");

#[test]
fn alternating_language_stress_cases_hold_boundaries() {
    for (class, input, expected) in fixture_tagged_cases(ALTERNATING_STRESS_CASES) {
        let got = simulate_space_triggered_typing_assist(&input, true);
        assert_eq!(got, expected, "class={class} input={input:?}");
        assert_eq!(
            expected.ends_with(char::is_whitespace),
            got.ends_with(char::is_whitespace),
            "class={class} trailing space boundary changed: got={got:?}"
        );
        assert_eq!(
            expected.split_whitespace().count(),
            got.split_whitespace().count(),
            "class={class} word count changed: got={got:?}"
        );
    }
}

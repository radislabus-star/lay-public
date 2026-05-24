use lay::config::{default_typing_assist_pipeline, CorrectionSafety, TypingAssistRuleConfig};
use lay::typing_assist::{
    apply_typing_assist_with_pipeline, split_edge_whitespace, split_ws_segments,
};
use lay::typing_context::typing_assist_pipeline_for_context;

const ALTERNATING_STRESS_CASES: &str =
    include_str!("fixtures/typing_assist_alternating_stress.tsv");

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

fn simulate_space_triggered_typing_assist(input: &str) -> String {
    let pipeline = default_typing_assist_pipeline();
    let mut text = String::new();
    for ch in input.chars() {
        text.push(ch);
        if ch.is_whitespace() {
            if let Some(next) = apply_typing_assist_to_tail(&text, true, &pipeline) {
                text = next;
            }
        }
    }
    text
}

fn fixture_rows(data: &'static str) -> impl Iterator<Item = (&'static str, String, String)> {
    data.lines()
        .filter(|line| !line.trim().is_empty() && !line.starts_with('#'))
        .map(|line| {
            let mut fields = line.split('\t');
            let class = fields.next().expect("fixture class");
            let input = fields.next().expect("fixture input");
            let expected = fields.next().expect("fixture expected");
            assert!(fields.next().is_none(), "fixture row must have 3 columns");
            (
                class,
                decode_fixture_field(input),
                decode_fixture_field(expected),
            )
        })
}

fn decode_fixture_field(value: &str) -> String {
    value.replace("\\s", " ")
}

#[test]
fn alternating_language_stress_cases_hold_boundaries() {
    for (class, input, expected) in fixture_rows(ALTERNATING_STRESS_CASES) {
        let got = simulate_space_triggered_typing_assist(&input);
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

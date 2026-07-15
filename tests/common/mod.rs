#![allow(dead_code)]

use lay::config::{default_typing_assist_pipeline, CorrectionSafety, TypingAssistRuleConfig};
use lay::text_edit::TextReplacement;
use lay::typing_assist::{
    select_typing_assist_with_pipeline, split_edge_whitespace, split_ws_segments,
};
use lay::typing_context::typing_assist_pipeline_for_context;

pub fn decode_fixture_field(value: &str) -> String {
    value.replace("\\s", " ")
}

pub fn fixture_rows(data: &'static str) -> impl Iterator<Item = &'static str> {
    data.lines()
        .filter(|line| !line.trim().is_empty() && !line.starts_with('#'))
}

pub fn fixture_lines(data: &'static str) -> impl Iterator<Item = String> {
    fixture_rows(data).map(decode_fixture_field)
}

pub fn fixture_cases(data: &'static str) -> impl Iterator<Item = (String, String)> {
    fixture_rows(data).map(|line| {
        let [input, expected] = split_fixture_columns(line);
        (decode_fixture_field(input), decode_fixture_field(expected))
    })
}

pub fn fixture_cols(data: &'static str) -> Vec<Vec<String>> {
    fixture_rows(data)
        .map(|line| line.split('\t').map(decode_fixture_field).collect())
        .collect()
}

pub fn fixture_row_by_id(data: &'static str, id: &str) -> Vec<String> {
    fixture_cols(data)
        .into_iter()
        .find(|row| row.first().is_some_and(|value| value == id))
        .unwrap_or_else(|| panic!("missing fixture row {id:?}"))
}

pub fn first_fixture_row(data: &'static str) -> Vec<String> {
    fixture_cols(data)
        .into_iter()
        .next()
        .expect("missing fixture row")
}

pub fn text_replacement(
    move_left: u32,
    backspaces: u32,
    insert: impl Into<String>,
    move_right: u32,
) -> TextReplacement {
    TextReplacement {
        move_left,
        backspaces,
        insert: insert.into(),
        move_right,
    }
}

pub fn zero_edge_text_replacement(
    row: &[String],
    backspaces: usize,
    insert: usize,
) -> TextReplacement {
    text_replacement(
        0,
        row[backspaces].parse().expect("backspaces"),
        &row[insert],
        0,
    )
}

pub fn fixture_tagged_cases(
    data: &'static str,
) -> impl Iterator<Item = (&'static str, String, String)> {
    fixture_rows(data).map(|line| {
        let [class, input, expected] = split_tagged_fixture_columns(line);
        (
            class,
            decode_fixture_field(input),
            decode_fixture_field(expected),
        )
    })
}

fn split_fixture_columns(line: &'static str) -> [&'static str; 2] {
    let mut fields = line.split('\t');
    let input = fields.next().expect("fixture input");
    let expected = fields.next().expect("fixture expected");
    assert!(fields.next().is_none(), "fixture row must have 2 columns");
    [input, expected]
}

fn split_tagged_fixture_columns(line: &'static str) -> [&'static str; 3] {
    let mut fields = line.split('\t');
    let class = fields.next().expect("fixture class");
    let input = fields.next().expect("fixture input");
    let expected = fields.next().expect("fixture expected");
    assert!(fields.next().is_none(), "fixture row must have 3 columns");
    [class, input, expected]
}

pub fn apply_typing_assist_to_tail(
    text: &str,
    allow_layout_auto: bool,
    pipeline: &[TypingAssistRuleConfig],
) -> Option<String> {
    let context_pipeline =
        typing_assist_pipeline_for_context(true, CorrectionSafety::Normal, pipeline, text);
    select_typing_assist_with_pipeline(text, allow_layout_auto, &context_pipeline).or_else(|| {
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
                select_typing_assist_with_pipeline(suffix, allow_layout_auto, &context_pipeline)
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

pub fn simulate_space_triggered_typing_assist(input: &str, allow_layout_auto: bool) -> String {
    simulate_space_triggered_typing_assist_with_pipeline(
        input,
        allow_layout_auto,
        &default_typing_assist_pipeline(),
    )
}

pub fn simulate_space_triggered_typing_assist_with_pipeline(
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

pub fn assert_same_boundaries(got: &str, expected: &str) {
    assert_eq!(
        expected.ends_with(char::is_whitespace),
        got.ends_with(char::is_whitespace),
        "trailing space boundary changed: got={got:?}"
    );
    assert_eq!(
        expected.split_whitespace().count(),
        got.split_whitespace().count(),
        "word count changed: got={got:?}"
    );
}

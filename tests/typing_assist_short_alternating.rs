mod common;

use common::{assert_same_boundaries, fixture_rows};
use lay::config::{default_typing_assist_pipeline, CorrectionSafety};
use lay::dict::{convert, Direction};
use lay::microbrain::MicrobrainOptions;
use lay::typing_assist::{
    explain_typing_assist_with_microbrain_options, split_edge_whitespace, split_ws_segments,
};
use lay::typing_context::typing_assist_pipeline_for_context;

fn join_with_trailing_space(words: &[String]) -> String {
    let mut out = words.join(" ");
    out.push(' ');
    out
}

fn simulate_experimental_typing_assist(input: &str) -> String {
    let configured = default_typing_assist_pipeline();
    let mut text = String::new();
    for ch in input.chars() {
        text.push(ch);
        if ch.is_whitespace() {
            let pipeline = typing_assist_pipeline_for_context(
                true,
                CorrectionSafety::Experimental,
                &configured,
                &text,
            );
            if let Some(next) = apply_nanda_typing_assist_to_tail(&text, true, &pipeline) {
                text = next;
            }
        }
    }
    text
}

fn apply_nanda_typing_assist_to_tail(
    text: &str,
    allow_layout_auto: bool,
    pipeline: &[lay::config::TypingAssistRuleConfig],
) -> Option<String> {
    explain_typing_assist_with_microbrain_options(
        text,
        allow_layout_auto,
        pipeline,
        &MicrobrainOptions::default(),
    )
    .output
    .or_else(|| {
        let (leading, core, trailing) = split_edge_whitespace(text);
        let segments = split_ws_segments(core);
        if segments.len() < 3 {
            return None;
        }

        for word_count in [1, 2] {
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
            if let Some(replacement) = explain_typing_assist_with_microbrain_options(
                suffix,
                allow_layout_auto,
                pipeline,
                &MicrobrainOptions::default(),
            )
            .output
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

fn short_alternating_words_50() -> Vec<String> {
    let words = fixture_rows(include_str!(
        "fixtures/typing_assist_short_alternating_pairs.tsv"
    ))
    .map(|row| {
        let mut fields = row.split('\t');
        let ru = fields.next().expect("ru word");
        let en = fields.next().expect("en word");
        assert!(fields.next().is_none(), "short alternating row must be TSV");
        (ru, en)
    })
    .flat_map(|(ru, en)| [ru.to_string(), en.to_string()])
    .collect::<Vec<_>>();

    assert_eq!(words.len(), 50);
    assert!(words.iter().all(|word| word.chars().count() <= 4));
    words
}

#[test]
fn clean_short_ru_en_alternation_stays_clean() {
    let expected = join_with_trailing_space(&short_alternating_words_50());
    let got = simulate_experimental_typing_assist(&expected);

    assert_eq!(got, expected);
    assert_same_boundaries(&got, &expected);
}

#[test]
fn short_english_words_typed_in_ru_layout_are_recovered_between_russian_words() {
    let expected_words = short_alternating_words_50();
    let input_words = expected_words
        .iter()
        .enumerate()
        .map(|(idx, word)| {
            if idx % 2 == 1 {
                convert(word, Direction::Us2Ru)
            } else {
                word.clone()
            }
        })
        .collect::<Vec<_>>();
    let input = join_with_trailing_space(&input_words);
    let expected = join_with_trailing_space(&expected_words);
    assert!(input_words.iter().all(|word| word.chars().count() <= 4));

    let got = simulate_experimental_typing_assist(&input);

    assert_eq!(got, expected, "input={input:?}");
    assert_same_boundaries(&got, &expected);
}

#[test]
fn short_russian_words_typed_in_us_layout_are_recovered_between_english_words() {
    let expected_words = short_alternating_words_50();
    let input_words = expected_words
        .iter()
        .enumerate()
        .map(|(idx, word)| {
            if idx % 2 == 0 {
                convert(word, Direction::Ru2Us)
            } else {
                word.clone()
            }
        })
        .collect::<Vec<_>>();
    let input = join_with_trailing_space(&input_words);
    let expected = join_with_trailing_space(&expected_words);
    assert!(input_words.iter().all(|word| word.chars().count() <= 4));

    let got = simulate_experimental_typing_assist(&input);

    assert_eq!(got, expected, "input={input:?}");
    assert_same_boundaries(&got, &expected);
}

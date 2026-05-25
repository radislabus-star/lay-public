use lay::config::{default_typing_assist_pipeline, CorrectionSafety, TypingAssistRuleConfig};
use lay::dict::{convert, Direction};
use lay::typing_assist::{
    apply_typing_assist_with_pipeline, split_edge_whitespace, split_ws_segments,
};
use lay::typing_context::typing_assist_pipeline_for_context;

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

fn join_with_trailing_space(words: &[String]) -> String {
    let mut out = words.join(" ");
    out.push(' ');
    out
}

fn assert_same_boundaries(got: &str, expected: &str) {
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

fn short_alternating_words_30() -> Vec<String> {
    let ru_words = [
        "я", "и", "мы", "ты", "он", "в", "к", "с", "не", "на", "ну", "по", "за", "вот", "это",
    ];
    let en_words = [
        "git", "api", "css", "cpu", "gpu", "html", "json", "llm", "log", "md", "pdf", "ram", "sql",
        "ssh", "vpn",
    ];

    let words = ru_words
        .into_iter()
        .zip(en_words)
        .flat_map(|(ru, en)| [ru.to_string(), en.to_string()])
        .collect::<Vec<_>>();

    assert_eq!(words.len(), 30);
    assert!(words.iter().all(|word| word.chars().count() <= 4));
    words
}

#[test]
fn clean_short_ru_en_alternation_stays_clean() {
    let expected = join_with_trailing_space(&short_alternating_words_30());
    let got = simulate_space_triggered_typing_assist(&expected);

    assert_eq!(got, expected);
    assert_same_boundaries(&got, &expected);
}

#[test]
fn short_english_words_typed_in_ru_layout_are_recovered_between_russian_words() {
    let expected_words = short_alternating_words_30();
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

    let got = simulate_space_triggered_typing_assist(&input);

    assert_eq!(got, expected, "input={input:?}");
    assert_same_boundaries(&got, &expected);
}

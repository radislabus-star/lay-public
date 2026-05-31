mod common;

use common::{assert_same_boundaries, fixture_rows, simulate_space_triggered_typing_assist};
use lay::dict::{convert, Direction};

fn join_with_trailing_space(words: &[String]) -> String {
    let mut out = words.join(" ");
    out.push(' ');
    out
}

fn short_alternating_words_30() -> Vec<String> {
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

    assert_eq!(words.len(), 30);
    assert!(words.iter().all(|word| word.chars().count() <= 4));
    words
}

#[test]
fn clean_short_ru_en_alternation_stays_clean() {
    let expected = join_with_trailing_space(&short_alternating_words_30());
    let got = simulate_space_triggered_typing_assist(&expected, true);

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

    let got = simulate_space_triggered_typing_assist(&input, true);

    assert_eq!(got, expected, "input={input:?}");
    assert_same_boundaries(&got, &expected);
}

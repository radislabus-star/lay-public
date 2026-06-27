use crate::word_reader::{split_word_punctuation, split_ws_segments};

pub(crate) fn syntax_allows_candidate(original: &str, replacement: &str) -> bool {
    future_auxiliary_allows_candidate(original, replacement)
}

fn future_auxiliary_allows_candidate(original: &str, replacement: &str) -> bool {
    let Some((left, original_word)) = two_word_tail(original) else {
        return true;
    };
    if !is_future_auxiliary(left) || !looks_like_infinitive_like_tail(original_word) {
        return true;
    }

    let Some((_, replacement_word)) = two_word_tail(replacement) else {
        return true;
    };
    looks_like_infinitive_like_tail(replacement_word)
}

fn two_word_tail(text: &str) -> Option<(&str, &str)> {
    let words = split_ws_segments(text)
        .into_iter()
        .filter_map(|(segment, is_ws)| {
            if is_ws {
                None
            } else {
                let (_, word, _) = split_word_punctuation(segment);
                (!word.is_empty()).then_some(word)
            }
        })
        .collect::<Vec<_>>();
    if words.len() == 2 {
        Some((words[0], words[1]))
    } else {
        None
    }
}

fn is_future_auxiliary(word: &str) -> bool {
    matches!(
        word.to_lowercase().as_str(),
        "буду" | "будешь" | "будет" | "будем" | "будете" | "будут"
    )
}

fn looks_like_infinitive_like_tail(word: &str) -> bool {
    let lower = word.to_lowercase();
    lower.ends_with("ть")
        || lower.ends_with("ти")
        || lower.ends_with("ться")
        || lower.ends_with("тись")
}

use crate::config::CorrectionEngine;
use crate::keyboard::map_original_events;
use crate::typing_replacements::contains_visual_b_word;
use crate::word_buffer::{WordBuffer, MAX_REPLACE_WORDS};

fn should_expand_auto_replace_context(buf: &WordBuffer) -> bool {
    let Some((events, _)) = buf.what_to_replay(2) else {
        return false;
    };
    contains_visual_b_word(&map_original_events(&events))
}

pub fn should_force_replay_for_short_fragment(text: &str) -> bool {
    let mut words = text.split_whitespace();
    let Some(word) = words.next() else {
        return false;
    };
    words.next().is_none() && (1..=2).contains(&word.chars().count())
}

pub fn effective_replace_words(
    buf: &WordBuffer,
    replace_words: usize,
    engine: CorrectionEngine,
    auto_replace: bool,
) -> usize {
    let replace_words = replace_words.clamp(1, MAX_REPLACE_WORDS);
    if engine == CorrectionEngine::Replay && auto_replace && should_expand_auto_replace_context(buf)
    {
        return replace_words.max(2);
    }
    replace_words
}

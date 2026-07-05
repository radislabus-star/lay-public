//! Lightweight word reading primitives.
//!
//! This module does not decide autocorrections. It only turns raw text into
//! word-shaped facts that higher-level scorers can reason about.

use crate::keyboard::is_cyrillic_letter;

pub const MAX_RU_FUNCTION_GLUE_LEFT_LEN: usize = 4;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WordSplit<'a> {
    pub left: &'a str,
    pub right: &'a str,
    pub left_len: usize,
    pub right_len: usize,
}

pub fn split_edge_whitespace(text: &str) -> (&str, &str, &str) {
    let start = text
        .char_indices()
        .find(|(_, ch)| !ch.is_whitespace())
        .map(|(idx, _)| idx)
        .unwrap_or(text.len());
    let end = text
        .char_indices()
        .rev()
        .find(|(_, ch)| !ch.is_whitespace())
        .map(|(idx, ch)| idx + ch.len_utf8())
        .unwrap_or(start);

    (&text[..start], &text[start..end], &text[end..])
}

pub fn split_word_punctuation(token: &str) -> (&str, &str, &str) {
    let start = token
        .char_indices()
        .find(|(_, ch)| ch.is_alphanumeric())
        .map(|(idx, _)| idx)
        .unwrap_or(token.len());
    let end = token
        .char_indices()
        .rev()
        .find(|(_, ch)| ch.is_alphanumeric())
        .map(|(idx, ch)| idx + ch.len_utf8())
        .unwrap_or(start);

    (&token[..start], &token[start..end], &token[end..])
}

pub fn split_ws_segments(text: &str) -> Vec<(&str, bool)> {
    let mut segments = Vec::new();
    let mut start = 0;
    let mut current_ws: Option<bool> = None;

    for (idx, ch) in text.char_indices() {
        let ws = ch.is_whitespace();
        match current_ws {
            Some(prev) if prev != ws => {
                segments.push((&text[start..idx], prev));
                start = idx;
                current_ws = Some(ws);
            }
            None => current_ws = Some(ws),
            _ => {}
        }
    }

    if let Some(ws) = current_ws {
        segments.push((&text[start..], ws));
    }
    segments
}

pub fn split_last_ws_token(text: &str) -> Option<(&str, &str)> {
    if text.is_empty() {
        return None;
    }
    let start = text
        .char_indices()
        .rev()
        .find(|(_idx, ch)| ch.is_whitespace())
        .map(|(idx, ch)| idx + ch.len_utf8())
        .unwrap_or(0);
    let (prefix, token) = text.split_at(start);
    (!token.is_empty()).then_some((prefix, token))
}

pub fn split_last_trimmed_ws_token(text: &str) -> Option<(&str, &str)> {
    split_last_ws_token(text.trim_end())
}

pub fn split_last_alphabetic_token(text: &str) -> Option<(&str, &str)> {
    let trimmed = text.trim_end();
    if trimmed.is_empty() {
        return None;
    }
    let end = trimmed
        .char_indices()
        .rev()
        .find_map(|(idx, ch)| ch.is_alphabetic().then_some(idx + ch.len_utf8()))?;
    let start = trimmed[..end]
        .char_indices()
        .rev()
        .find_map(|(idx, ch)| (!ch.is_alphabetic()).then_some(idx + ch.len_utf8()))
        .unwrap_or(0);
    let (prefix, rest) = trimmed.split_at(start);
    let token = &rest[..end - start];
    (!token.is_empty()).then_some((prefix, token))
}

pub fn is_cyrillic_word(word: &str) -> bool {
    word.chars()
        .all(|ch| matches!(ch, 'А'..='я' | 'ё' | 'Ё' | '-'))
}

pub fn is_cyrillic_letters_only(word: &str) -> bool {
    !word.is_empty() && word.chars().all(is_cyrillic_letter)
}

pub fn cyrillic_word_splits(word: &str) -> Vec<WordSplit<'_>> {
    if !is_cyrillic_letters_only(word) {
        return Vec::new();
    }

    let mut boundaries: Vec<usize> = word.char_indices().map(|(idx, _)| idx).collect();
    boundaries.push(word.len());
    let char_len = boundaries.len().saturating_sub(1);
    if char_len < 2 {
        return Vec::new();
    }

    (1..char_len)
        .map(|split_idx| {
            let byte_idx = boundaries[split_idx];
            WordSplit {
                left: &word[..byte_idx],
                right: &word[byte_idx..],
                left_len: split_idx,
                right_len: char_len - split_idx,
            }
        })
        .collect()
}

pub fn cyrillic_word_segmentations(word: &str, max_parts: usize) -> Vec<Vec<&str>> {
    if max_parts < 2 || !is_cyrillic_letters_only(word) {
        return Vec::new();
    }

    let mut boundaries: Vec<usize> = word.char_indices().map(|(idx, _)| idx).collect();
    boundaries.push(word.len());
    let char_len = boundaries.len().saturating_sub(1);
    if char_len < 2 {
        return Vec::new();
    }

    let mut out = Vec::new();
    let mut current = Vec::new();
    collect_segmentations(word, &boundaries, 0, max_parts, &mut current, &mut out);
    out
}

fn collect_segmentations<'a>(
    word: &'a str,
    boundaries: &[usize],
    start_idx: usize,
    max_parts: usize,
    current: &mut Vec<&'a str>,
    out: &mut Vec<Vec<&'a str>>,
) {
    let char_len = boundaries.len().saturating_sub(1);
    if start_idx == char_len {
        if current.len() >= 2 {
            out.push(current.clone());
        }
        return;
    }
    if current.len() >= max_parts {
        return;
    }

    let remaining_parts = max_parts - current.len();
    for end_idx in start_idx + 1..=char_len {
        let remaining_chars = char_len - end_idx;
        if remaining_chars > 0 && remaining_parts <= 1 {
            continue;
        }
        let segment = &word[boundaries[start_idx]..boundaries[end_idx]];
        current.push(segment);
        collect_segmentations(word, boundaries, end_idx, max_parts, current, out);
        current.pop();
    }
}

#[cfg(test)]
#[path = "word_reader_tests.rs"]
mod tests;

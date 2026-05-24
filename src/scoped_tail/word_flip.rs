use crate::keyboard::{
    is_cyrillic_letter, map_events_to_layout, map_original_events,
    mixed_visual_latin_word_target_layout, original_event_char, replay_layout_decision, KeyEvent,
};
use crate::layout_autoswitch::{
    correct_duplicate_layout_prefix_on_ascii_token, is_cyrillic_hyphenated_word_for_layout,
};
use crate::russian_chars::same_letter_ignore_case;

pub(super) fn flip_word_events(word: &[KeyEvent]) -> String {
    if let Some(repaired) = repair_cyrillic_prefix_before_ascii_tail(word) {
        return repaired;
    }
    let original = map_original_events(word);
    if let Some(repaired) = correct_duplicate_layout_prefix_on_ascii_token(&original) {
        return repaired;
    }
    if let Some(target_is_ru) = mixed_visual_latin_word_target_layout(word) {
        return map_events_to_layout(word, target_is_ru);
    }
    if let Some(normalized) = normalize_mixed_word_to_last_layout(word) {
        return normalized;
    }
    let decision = replay_layout_decision(word);
    map_events_to_layout(word, decision.target_is_ru)
}

pub fn repair_cyrillic_prefix_before_ascii_tail(word: &[KeyEvent]) -> Option<String> {
    let first_event = word.first()?;
    let first = original_event_char(first_event)?;
    if !is_cyrillic_letter(first) || word.len() < 3 {
        return None;
    }

    let rest = &word[1..];
    let rest_original: String = rest.iter().filter_map(original_event_char).collect();
    if rest_original.chars().count() != rest.len()
        || !rest_original.is_ascii()
        || !rest_original.chars().any(|ch| ch.is_ascii_alphabetic())
        || !rest_original
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || ch == '-')
    {
        return None;
    }

    let all_ru = map_events_to_layout(word, true);
    if all_ru != map_original_events(word) && is_cyrillic_hyphenated_word_for_layout(&all_ru) {
        return Some(all_ru);
    }

    let mut chars = all_ru.chars();
    let first_ru = chars.next()?;
    let second_ru = chars.next()?;
    if !same_letter_ignore_case(first_ru, second_ru) {
        return None;
    }

    let mut candidate = String::new();
    candidate.push(first_ru);
    candidate.extend(chars);
    if candidate == all_ru || candidate == map_original_events(word) {
        return None;
    }
    is_cyrillic_hyphenated_word_for_layout(&candidate).then_some(candidate)
}

fn normalize_mixed_word_to_last_layout(word: &[KeyEvent]) -> Option<String> {
    let target_is_ru = word.last()?.layout_is_ru;
    if word.iter().all(|event| event.layout_is_ru == target_is_ru) {
        return None;
    }

    let mut out = String::new();
    let mut run_start = 0;
    let mut current_layout = word.first()?.layout_is_ru;
    for (idx, event) in word.iter().enumerate() {
        if event.layout_is_ru != current_layout {
            let run = map_events_to_layout(&word[run_start..idx], target_is_ru);
            push_with_overlap(&mut out, &run);
            run_start = idx;
            current_layout = event.layout_is_ru;
        }
    }
    let run = map_events_to_layout(&word[run_start..], target_is_ru);
    push_with_overlap(&mut out, &run);

    (!out.is_empty()).then_some(out)
}

fn push_with_overlap(out: &mut String, next: &str) {
    if out.is_empty() || next.is_empty() {
        out.push_str(next);
        return;
    }

    let out_chars: Vec<char> = out.chars().collect();
    let next_chars: Vec<char> = next.chars().collect();
    let max_overlap = out_chars.len().min(next_chars.len());
    let overlap = (1..=max_overlap)
        .rev()
        .find(|len| {
            out_chars[out_chars.len() - len..]
                .iter()
                .zip(&next_chars[..*len])
                .all(|(left, right)| left == right)
        })
        .unwrap_or(0);
    out.push_str(&next_chars[overlap..].iter().collect::<String>());
}

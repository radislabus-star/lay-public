use super::ru_emit::char_to_ru_key_event;
use super::script::{is_cyrillic_letter, preferred_layout_for_text};
use super::us_emit::char_to_us_key_event;
use crate::keyboard::{KeyEvent, TextInputRun};

pub fn text_to_key_events(text: &str, fallback_is_ru: bool) -> Option<Vec<KeyEvent>> {
    let mut events = Vec::with_capacity(text.chars().count());
    text_to_key_events_into(text, fallback_is_ru, &mut events)?;
    Some(events)
}

pub(crate) fn text_to_key_events_into(
    text: &str,
    fallback_is_ru: bool,
    events: &mut Vec<KeyEvent>,
) -> Option<()> {
    events.clear();
    let mut current_is_ru = preferred_layout_for_text(text, fallback_is_ru);
    for ch in text.chars() {
        let Some((target_is_ru, event)) = char_to_layout_key_event(ch, current_is_ru) else {
            events.clear();
            return None;
        };
        current_is_ru = target_is_ru;
        events.push(event);
    }
    Some(())
}

pub fn text_to_uinput_runs(text: &str, fallback_is_ru: bool) -> Option<Vec<TextInputRun>> {
    let mut runs: Vec<TextInputRun> = Vec::new();
    let mut current_is_ru = preferred_layout_for_text(text, fallback_is_ru);

    for ch in text.chars() {
        let (target_is_ru, event) = char_to_layout_key_event(ch, current_is_ru)?;
        current_is_ru = target_is_ru;
        if let Some(run) = runs
            .last_mut()
            .filter(|run| run.target_is_ru == target_is_ru)
        {
            run.events.push(event);
        } else {
            runs.push(TextInputRun {
                target_is_ru,
                events: vec![event],
            });
        }
    }

    Some(runs)
}

fn char_to_layout_key_event(ch: char, current_is_ru: bool) -> Option<(bool, KeyEvent)> {
    if is_cyrillic_letter(ch) {
        return char_to_ru_key_event(ch).map(|event| (true, event));
    }
    if ch.is_ascii_alphabetic() {
        return char_to_us_key_event(ch).map(|event| (false, event));
    }
    if current_is_ru {
        if let Some(event) = char_to_ru_key_event(ch) {
            return Some((true, event));
        }
    }
    if let Some(event) = char_to_us_key_event(ch) {
        return Some((false, event));
    }
    char_to_ru_key_event(ch).map(|event| (true, event))
}

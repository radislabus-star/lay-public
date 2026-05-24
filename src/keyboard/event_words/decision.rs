use evdev::KeyCode;

use super::super::keymap::is_typing_key;
use super::super::KeyEvent;
use super::visual_latin::mixed_visual_latin_word_target_layout;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReplayLayoutDecision {
    pub target_is_ru: bool,
    pub mixed_layouts: bool,
}

pub fn replay_layout_decision(events: &[KeyEvent]) -> ReplayLayoutDecision {
    let typed_layouts: Vec<bool> = events
        .iter()
        .filter(|ev| is_layout_decision_key(KeyCode::new(ev.keycode)))
        .map(|ev| ev.layout_is_ru)
        .collect();
    let first_layout = typed_layouts.first().copied().unwrap_or(false);
    let last_layout = typed_layouts.last().copied().unwrap_or(first_layout);
    let mixed_layouts = typed_layouts.iter().any(|layout| *layout != first_layout);
    let target_is_ru = if mixed_layouts {
        mixed_visual_latin_word_target_layout(events).unwrap_or(last_layout)
    } else {
        !first_layout
    };
    ReplayLayoutDecision {
        target_is_ru,
        mixed_layouts,
    }
}

pub fn is_layout_decision_key(key: KeyCode) -> bool {
    is_typing_key(key) && key != KeyCode::KEY_SPACE
}

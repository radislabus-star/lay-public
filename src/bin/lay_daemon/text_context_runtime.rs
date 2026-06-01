use evdev::KeyCode;

use super::ShiftState;

pub(super) fn should_advance_text_context(
    key: KeyCode,
    value: i32,
    modifiers: &ShiftState,
) -> bool {
    value == 1 && (is_pointer_context_key(key) || is_keyboard_context_shortcut(key, modifiers))
}

fn is_pointer_context_key(key: KeyCode) -> bool {
    matches!(
        key,
        KeyCode::BTN_LEFT
            | KeyCode::BTN_RIGHT
            | KeyCode::BTN_MIDDLE
            | KeyCode::BTN_SIDE
            | KeyCode::BTN_EXTRA
            | KeyCode::BTN_FORWARD
            | KeyCode::BTN_BACK
            | KeyCode::BTN_TASK
    )
}

fn is_keyboard_context_shortcut(key: KeyCode, modifiers: &ShiftState) -> bool {
    let ctrl = modifiers.ctrl_active();
    let alt = modifiers.alt_active();
    let meta = modifiers.meta_active();

    (alt && key == KeyCode::KEY_TAB)
        || (ctrl
            && matches!(
                key,
                KeyCode::KEY_TAB
                    | KeyCode::KEY_PAGEUP
                    | KeyCode::KEY_PAGEDOWN
                    | KeyCode::KEY_L
                    | KeyCode::KEY_T
                    | KeyCode::KEY_W
                    | KeyCode::KEY_N
            ))
        || (meta && key == KeyCode::KEY_TAB)
}

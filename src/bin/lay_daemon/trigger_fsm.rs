use evdev::KeyCode;
use lay::keyboard::is_typing_key;
use std::time::Instant;

#[derive(Default)]
pub(super) struct ShiftState {
    left: bool,
    right: bool,
    left_ctrl: bool,
    right_ctrl: bool,
    left_alt: bool,
    right_alt: bool,
    left_meta: bool,
    right_meta: bool,
}
impl ShiftState {
    pub(super) fn update(&mut self, key: KeyCode, value: i32) {
        let pressed = value != 0;
        match key {
            KeyCode::KEY_LEFTSHIFT => self.left = pressed,
            KeyCode::KEY_RIGHTSHIFT => self.right = pressed,
            KeyCode::KEY_LEFTCTRL => self.left_ctrl = pressed,
            KeyCode::KEY_RIGHTCTRL => self.right_ctrl = pressed,
            KeyCode::KEY_LEFTALT => self.left_alt = pressed,
            KeyCode::KEY_RIGHTALT => self.right_alt = pressed,
            KeyCode::KEY_LEFTMETA => self.left_meta = pressed,
            KeyCode::KEY_RIGHTMETA => self.right_meta = pressed,
            _ => {}
        }
    }

    pub(super) fn clear_shifts(&mut self) {
        self.left = false;
        self.right = false;
    }

    pub(super) fn any(&self) -> bool {
        self.left || self.right
    }

    pub(super) fn shortcut_active(&self) -> bool {
        self.left_ctrl
            || self.right_ctrl
            || self.left_alt
            || self.right_alt
            || self.left_meta
            || self.right_meta
    }
}

pub(super) fn single_hotkey_keycode(id: &str) -> Option<KeyCode> {
    match id {
        "single-lshift" => Some(KeyCode::KEY_LEFTSHIFT),
        "single-rshift" => Some(KeyCode::KEY_RIGHTSHIFT),
        "single-lctrl" => Some(KeyCode::KEY_LEFTCTRL),
        "single-rctrl" => Some(KeyCode::KEY_RIGHTCTRL),
        "single-lalt" => Some(KeyCode::KEY_LEFTALT),
        "single-ralt" => Some(KeyCode::KEY_RIGHTALT),
        "single-pause" => Some(KeyCode::KEY_PAUSE),
        "caps-lock" => Some(KeyCode::KEY_CAPSLOCK),
        _ => None,
    }
}

/// FSM для детекции двойного левого Shift по паттерну press→release→press→release.
///
/// Каждый Shift должен быть именно тапом (≤ tap_max мс).
/// Если держать дольше — это заглавная буква, не двойной Shift.
/// Любая другая клавиша в любом состоянии → Idle (отмена).
#[derive(Debug, Clone, Copy)]
pub(super) enum DShiftState {
    Idle,
    /// Первый Shift нажат, ждём release
    FirstPress {
        pressed_at: Instant,
    },
    /// Первый тап завершён, ждём второй press
    WaitingSecond {
        first_release: Instant,
    },
    /// Второй Shift нажат, ждём release → DOUBLE
    SecondPress {
        second_press: Instant,
    },
    /// Третий/четвёртый тап в optional multi-tap mode.
    AdditionalPress {
        pressed_at: Instant,
    },
}

#[derive(Debug, Clone, Copy)]
pub(super) struct MultiTapPending {
    pub(super) tap_count: u8,
    pub(super) last_release: Instant,
}

pub(super) fn multi_tap_scope_for_taps(taps: u8) -> Option<usize> {
    match taps {
        0 | 1 => None,
        2 => Some(1),
        3 => Some(2),
        _ => Some(3),
    }
}

// ─── Word boundary детекция ─────────────────────────────────

pub(super) fn is_hard_boundary(key: KeyCode) -> bool {
    use KeyCode as K;
    matches!(
        key,
        K::KEY_ENTER
            | K::KEY_TAB
            | K::KEY_ESC
            | K::KEY_LEFT
            | K::KEY_RIGHT
            | K::KEY_UP
            | K::KEY_DOWN
            | K::KEY_HOME
            | K::KEY_END
            | K::KEY_PAGEUP
            | K::KEY_PAGEDOWN
            | K::KEY_BACKSPACE
            | K::KEY_DELETE
    )
}

pub(super) fn should_ignore_buffer_key(
    key: KeyCode,
    modifiers: &ShiftState,
    current_empty: bool,
) -> bool {
    if modifiers.shortcut_active()
        && (key == KeyCode::KEY_SPACE
            || is_typing_key(key)
            || is_leading_non_word_symbol_key(key, modifiers.any()))
    {
        return true;
    }

    should_start_ignored_buffer_token(key, modifiers, current_empty)
}

pub(super) fn should_start_ignored_buffer_token(
    key: KeyCode,
    modifiers: &ShiftState,
    current_empty: bool,
) -> bool {
    if modifiers.shortcut_active() {
        return false;
    }
    current_empty && is_leading_non_word_symbol_key(key, modifiers.any())
}

pub(super) fn should_schedule_typing_assist_after_space(
    active: bool,
    suppress_once: &mut bool,
) -> bool {
    if !active {
        return false;
    }
    if *suppress_once {
        *suppress_once = false;
        return false;
    }
    true
}

pub(super) fn should_run_typing_assist_on_space_release(
    pending: bool,
    active: bool,
    shift_active: bool,
    buffer_empty: bool,
) -> bool {
    pending && active && !shift_active && !buffer_empty
}

pub(super) fn should_run_deferred_typing_assist_after_space(
    pending: bool,
    active: bool,
    shift_active: bool,
) -> bool {
    pending && active && !shift_active
}

pub(super) fn typing_assist_cursor_offset_after_space(current_len: usize) -> u32 {
    current_len.min(u32::MAX as usize) as u32
}

fn is_leading_non_word_symbol_key(key: KeyCode, _shift: bool) -> bool {
    matches!(key, KeyCode::KEY_EQUAL | KeyCode::KEY_MINUS)
}

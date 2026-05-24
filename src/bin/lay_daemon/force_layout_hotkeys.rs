use evdev::{uinput::VirtualDevice, KeyCode};
use lay::config::LayConfig;
use lay::word_buffer::WordBuffer;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use super::{
    handle_force_layout_hotkey, lock_virtual_keyboard, log, single_hotkey_keycode, DShiftState,
    MultiTapPending, ShiftState,
};

pub(super) struct ForceLayoutHotkeys {
    enabled: bool,
    ru_key: Option<KeyCode>,
    en_key: Option<KeyCode>,
    ru_pressed_at: Option<Instant>,
    en_pressed_at: Option<Instant>,
    other_key: bool,
}

pub(super) struct ForceLayoutHotkeyContext<'a> {
    pub(super) buffer: &'a mut WordBuffer,
    pub(super) virtual_kbd: &'a Arc<Mutex<Option<VirtualDevice>>>,
    pub(super) executing: &'a mut bool,
    pub(super) current_layout_is_ru: &'a mut bool,
    pub(super) last_layout_poll: &'a mut Instant,
    pub(super) shift_state: &'a mut ShiftState,
    pub(super) dshift_state: &'a mut DShiftState,
    pub(super) pending_multi_tap: &'a mut Option<MultiTapPending>,
    pub(super) single_pressed_at: &'a mut Option<Instant>,
    pub(super) last_double_at: &'a mut Option<Instant>,
    pub(super) clear_on_next_typing: &'a mut bool,
    pub(super) shift_tap_max: Duration,
    pub(super) debounce_window: Duration,
}

impl ForceLayoutHotkeys {
    pub(super) fn from_config(cfg: &LayConfig) -> Self {
        let ru_key = single_hotkey_keycode(&cfg.force_ru_key);
        let en_key = single_hotkey_keycode(&cfg.force_en_key);
        let enabled =
            cfg.force_layout_hotkeys && ru_key.is_some() && en_key.is_some() && ru_key != en_key;

        Self {
            enabled,
            ru_key,
            en_key,
            ru_pressed_at: None,
            en_pressed_at: None,
            other_key: false,
        }
    }

    pub(super) fn handle_event(
        &mut self,
        key: KeyCode,
        value: i32,
        ctx: ForceLayoutHotkeyContext<'_>,
    ) -> bool {
        if !self.enabled {
            return false;
        }

        let force_target = if Some(key) == self.ru_key {
            Some(true)
        } else if Some(key) == self.en_key {
            Some(false)
        } else {
            None
        };

        if let Some(target_is_ru) = force_target {
            self.handle_force_key(target_is_ru, value, ctx);
            return true;
        }

        if value == 1 && (self.ru_pressed_at.is_some() || self.en_pressed_at.is_some()) {
            self.other_key = true;
        }
        false
    }

    fn handle_force_key(
        &mut self,
        target_is_ru: bool,
        value: i32,
        ctx: ForceLayoutHotkeyContext<'_>,
    ) {
        let pressed_at = if target_is_ru {
            &mut self.ru_pressed_at
        } else {
            &mut self.en_pressed_at
        };

        match value {
            1 => {
                *pressed_at = Some(Instant::now());
                self.other_key = false;
            }
            0 => {
                let Some(t) = pressed_at.take() else {
                    return;
                };
                let held = t.elapsed();
                if self.other_key
                    || held > ctx.shift_tap_max
                    || !ctx
                        .last_double_at
                        .map_or(true, |d| d.elapsed() >= ctx.debounce_window)
                {
                    return;
                }

                let mut g = lock_virtual_keyboard(ctx.virtual_kbd);
                let result =
                    handle_force_layout_hotkey(target_is_ru, ctx.buffer, g.as_mut(), ctx.executing);
                if let Some(is_ru) = result {
                    *ctx.current_layout_is_ru = is_ru;
                    *ctx.last_layout_poll = Instant::now();
                }
                drop(g);
                ctx.shift_state.clear_shifts();
                *ctx.dshift_state = DShiftState::Idle;
                *ctx.pending_multi_tap = None;
                *ctx.single_pressed_at = None;
                *ctx.last_double_at = Some(Instant::now());
                *ctx.clear_on_next_typing = true;
                log(&format!(
                    "· force-layout {} fired (held {}ms)",
                    if target_is_ru { "RU" } else { "EN" },
                    held.as_millis()
                ));
            }
            _ => {}
        }
    }
}

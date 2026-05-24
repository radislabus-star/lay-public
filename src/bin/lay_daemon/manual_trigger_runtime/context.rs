use evdev::{uinput::VirtualDevice, Device, KeyCode};
use lay::word_buffer::WordBuffer;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use super::super::{DShiftState, MultiTapPending, ShiftState};

pub(crate) struct PendingMultiTapTimeoutContext<'a> {
    pub(crate) buffer: &'a mut WordBuffer,
    pub(crate) device: &'a mut Device,
    pub(crate) virtual_kbd: &'a Arc<Mutex<Option<VirtualDevice>>>,
    pub(crate) executing: &'a mut bool,
    pub(crate) current_layout_is_ru: &'a mut bool,
    pub(crate) last_layout_poll: &'a mut Instant,
    pub(crate) suppress_next_typing_assist_after_manual_replay: &'a mut bool,
    pub(crate) shift_state: &'a mut ShiftState,
    pub(crate) dshift_state: &'a mut DShiftState,
    pub(crate) pending_multi_tap: &'a mut Option<MultiTapPending>,
    pub(crate) last_double_at: &'a mut Option<Instant>,
    pub(crate) clear_on_next_typing: &'a mut bool,
    pub(crate) shift_window: Duration,
    pub(crate) events_since_word_start: u32,
}

pub(crate) struct ManualTriggerEventContext<'a> {
    pub(crate) key: KeyCode,
    pub(crate) code: u16,
    pub(crate) value: i32,
    pub(crate) verbose: bool,
    pub(crate) trigger_key: KeyCode,
    pub(crate) is_caps_trigger: bool,
    pub(crate) is_single_trigger: bool,
    pub(crate) shift_tap_max: Duration,
    pub(crate) shift_window: Duration,
    pub(crate) debounce_window: Duration,
    pub(crate) multi_tap_scope: bool,
    pub(crate) multi_tap_max_taps: u8,
    pub(crate) events_since_word_start: u32,
    pub(crate) buffer: &'a mut WordBuffer,
    pub(crate) device: &'a mut Device,
    pub(crate) virtual_kbd: &'a Arc<Mutex<Option<VirtualDevice>>>,
    pub(crate) executing: &'a mut bool,
    pub(crate) current_layout_is_ru: &'a mut bool,
    pub(crate) last_layout_poll: &'a mut Instant,
    pub(crate) suppress_next_typing_assist_after_manual_replay: &'a mut bool,
    pub(crate) shift_state: &'a mut ShiftState,
    pub(crate) dshift_state: &'a mut DShiftState,
    pub(crate) pending_multi_tap: &'a mut Option<MultiTapPending>,
    pub(crate) last_double_at: &'a mut Option<Instant>,
    pub(crate) clear_on_next_typing: &'a mut bool,
    pub(crate) single_pressed_at: &'a mut Option<Instant>,
    pub(crate) single_other_key: &'a mut bool,
}

pub(crate) struct ManualTriggerFireContext<'a> {
    pub(crate) buffer: &'a mut WordBuffer,
    pub(crate) device: &'a mut Device,
    pub(crate) virtual_kbd: &'a Arc<Mutex<Option<VirtualDevice>>>,
    pub(crate) executing: &'a mut bool,
    pub(crate) current_layout_is_ru: &'a mut bool,
    pub(crate) last_layout_poll: &'a mut Instant,
    pub(crate) suppress_next_typing_assist_after_manual_replay: &'a mut bool,
    pub(crate) shift_state: &'a mut ShiftState,
    pub(crate) dshift_state: &'a mut DShiftState,
    pub(crate) pending_multi_tap: &'a mut Option<MultiTapPending>,
    pub(crate) last_double_at: &'a mut Option<Instant>,
    pub(crate) clear_on_next_typing: &'a mut bool,
}

impl<'a> ManualTriggerEventContext<'a> {
    pub(crate) fn fire_context(&mut self) -> ManualTriggerFireContext<'_> {
        ManualTriggerFireContext {
            buffer: self.buffer,
            device: self.device,
            virtual_kbd: self.virtual_kbd,
            executing: self.executing,
            current_layout_is_ru: self.current_layout_is_ru,
            last_layout_poll: self.last_layout_poll,
            suppress_next_typing_assist_after_manual_replay: self
                .suppress_next_typing_assist_after_manual_replay,
            shift_state: self.shift_state,
            dshift_state: self.dshift_state,
            pending_multi_tap: self.pending_multi_tap,
            last_double_at: self.last_double_at,
            clear_on_next_typing: self.clear_on_next_typing,
        }
    }
}

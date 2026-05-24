use lay::config::LayConfig;
use lay::word_buffer::WordBuffer;
use std::time::{Duration, Instant};

use super::pending_typing_assist::PendingTypingAssist;
use super::{
    read_current_layout_is_ru, DShiftState, ForceLayoutHotkeys, MultiTapPending, ShiftState,
    FOCUS_IGNORE_POLL_INTERVAL_MS,
};

pub(super) struct DaemonLoopState {
    pub(super) single_pressed_at: Option<Instant>,
    pub(super) single_other_key: bool,
    pub(super) force_layout_hotkeys: ForceLayoutHotkeys,
    pub(super) multi_tap_scope: bool,
    pub(super) multi_tap_max_taps: u8,
    pub(super) pending_multi_tap: Option<MultiTapPending>,
    pub(super) buffer: WordBuffer,
    pub(super) shift_state: ShiftState,
    pub(super) dshift_state: DShiftState,
    pub(super) executing: bool,
    pub(super) shift_tap_max: Duration,
    pub(super) shift_window: Duration,
    pub(super) debounce_window: Duration,
    pub(super) current_layout_is_ru: bool,
    pub(super) last_layout_poll: Instant,
    pub(super) last_double_at: Option<Instant>,
    pub(super) clear_on_next_typing: bool,
    pub(super) suppress_next_typing_assist_after_manual_replay: bool,
    pub(super) events_since_word_start: u32,
    pub(super) pending_typing_assist_after_space: Option<PendingTypingAssist>,
    pub(super) focus_ignored: bool,
    pub(super) ignore_current_token_until_space: bool,
    pub(super) last_focus_ignore_poll: Instant,
}

impl DaemonLoopState {
    pub(super) fn new(cfg: &LayConfig, is_caps_trigger: bool, is_single_trigger: bool) -> Self {
        let now = Instant::now();
        Self {
            single_pressed_at: None,
            single_other_key: false,
            force_layout_hotkeys: ForceLayoutHotkeys::from_config(cfg),
            multi_tap_scope: cfg.multi_tap_scope && !is_caps_trigger && !is_single_trigger,
            multi_tap_max_taps: cfg.active_multi_tap_max_taps(),
            pending_multi_tap: None,
            buffer: WordBuffer::new(),
            shift_state: ShiftState::default(),
            dshift_state: DShiftState::Idle,
            executing: false,
            shift_tap_max: Duration::from_millis(cfg.tap_max_ms),
            shift_window: Duration::from_millis(cfg.shift_window_ms),
            debounce_window: Duration::from_millis(cfg.debounce_ms),
            current_layout_is_ru: read_current_layout_is_ru().unwrap_or(false),
            last_layout_poll: now,
            last_double_at: None,
            clear_on_next_typing: false,
            suppress_next_typing_assist_after_manual_replay: false,
            events_since_word_start: 0,
            pending_typing_assist_after_space: None,
            focus_ignored: false,
            ignore_current_token_until_space: false,
            last_focus_ignore_poll: now - Duration::from_millis(FOCUS_IGNORE_POLL_INTERVAL_MS),
        }
    }
}

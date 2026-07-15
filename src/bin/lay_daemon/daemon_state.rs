use lay::config::LayConfig;
use lay::word_buffer::WordBuffer;
use std::collections::HashMap;
use std::time::{Duration, Instant};

use super::pending_typing_assist::PendingTypingAssist;
use super::typing_assist_worker::TypingAssistWorker;
use super::{
    read_current_layout_is_ru, DShiftState, DaemonTextContext, ForceLayoutHotkeys, MultiTapPending,
    ShiftState, FOCUS_IGNORE_POLL_INTERVAL_MS,
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
    pub(super) alt_shift_layout_before: Option<bool>,
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
    pub(super) typing_assist_worker: TypingAssistWorker,
    pub(super) focus_ignored: bool,
    pub(super) focused_window_identity: Option<String>,
    pub(super) field_context_epoch: u64,
    pub(super) active_window_identity: Option<String>,
    pub(super) window_states: HashMap<String, WindowInputState>,
    pub(super) ignore_current_token_until_space: bool,
    pub(super) last_focus_ignore_poll: Instant,
}

pub(super) struct WindowInputState {
    buffer: WordBuffer,
    events_since_word_start: u32,
    pending_typing_assist_after_space: Option<PendingTypingAssist>,
    ignore_current_token_until_space: bool,
    clear_on_next_typing: bool,
    suppress_next_typing_assist_after_manual_replay: bool,
    saved_at: Instant,
}

impl WindowInputState {
    fn take_from(state: &mut DaemonLoopState) -> Self {
        Self {
            buffer: std::mem::take(&mut state.buffer),
            events_since_word_start: state.events_since_word_start,
            pending_typing_assist_after_space: state.pending_typing_assist_after_space.take(),
            ignore_current_token_until_space: state.ignore_current_token_until_space,
            clear_on_next_typing: state.clear_on_next_typing,
            suppress_next_typing_assist_after_manual_replay: state
                .suppress_next_typing_assist_after_manual_replay,
            saved_at: Instant::now(),
        }
    }

    fn restore_into(self, state: &mut DaemonLoopState) {
        state.buffer = self.buffer;
        state.events_since_word_start = self.events_since_word_start;
        state.pending_typing_assist_after_space = self.pending_typing_assist_after_space;
        state.ignore_current_token_until_space = self.ignore_current_token_until_space;
        state.clear_on_next_typing = self.clear_on_next_typing;
        state.suppress_next_typing_assist_after_manual_replay =
            self.suppress_next_typing_assist_after_manual_replay;
    }
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
            alt_shift_layout_before: None,
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
            typing_assist_worker: TypingAssistWorker::new(),
            focus_ignored: false,
            focused_window_identity: None,
            field_context_epoch: 0,
            active_window_identity: None,
            window_states: HashMap::new(),
            ignore_current_token_until_space: false,
            last_focus_ignore_poll: now - Duration::from_millis(FOCUS_IGNORE_POLL_INTERVAL_MS),
        }
    }

    pub(super) fn switch_window_input_state(&mut self, identity: Option<String>) -> bool {
        self.focused_window_identity = identity;
        self.switch_text_context_state()
    }

    pub(super) fn switch_field_context_epoch(&mut self, epoch: u64) -> bool {
        if self.field_context_epoch == epoch {
            return false;
        }
        self.field_context_epoch = epoch;
        self.switch_text_context_state()
    }

    fn switch_text_context_state(&mut self) -> bool {
        let identity = self.active_text_context_identity();
        if self.active_window_identity == identity {
            return false;
        }
        if let Some(previous) = self.active_window_identity.take() {
            if self.should_save_current_text_context() {
                let previous_state = WindowInputState::take_from(self);
                self.window_states.insert(previous, previous_state);
            }
        }

        if let Some(current) = identity.clone() {
            if let Some(current_state) = self.window_states.remove(&current) {
                current_state.restore_into(self);
            } else {
                self.buffer = WordBuffer::new();
                self.events_since_word_start = 0;
                self.pending_typing_assist_after_space = None;
                self.ignore_current_token_until_space = false;
                self.clear_on_next_typing = false;
                self.suppress_next_typing_assist_after_manual_replay = false;
            }
        }
        self.active_window_identity = identity;
        self.prune_window_states();
        true
    }

    fn active_text_context_identity(&self) -> Option<String> {
        Some(match self.focused_window_identity.as_ref() {
            Some(window) => format!("{window}:field:{}", self.field_context_epoch),
            None => format!("field:{}", self.field_context_epoch),
        })
    }

    pub(super) fn daemon_text_context(&self) -> DaemonTextContext {
        DaemonTextContext::new(
            self.focused_window_identity.clone(),
            self.field_context_epoch,
        )
    }

    fn should_save_current_text_context(&self) -> bool {
        !self.buffer.current_is_empty()
            || self.events_since_word_start > 0
            || self.pending_typing_assist_after_space.is_some()
            || self.ignore_current_token_until_space
            || self.clear_on_next_typing
            || self.suppress_next_typing_assist_after_manual_replay
    }

    fn prune_window_states(&mut self) {
        const MAX_SAVED_TEXT_CONTEXTS: usize = 50;

        while self.window_states.len() > MAX_SAVED_TEXT_CONTEXTS {
            let Some(oldest_key) = self
                .window_states
                .iter()
                .min_by_key(|(_, state)| state.saved_at)
                .map(|(key, _)| key.clone())
            else {
                break;
            };
            self.window_states.remove(&oldest_key);
        }
    }
}

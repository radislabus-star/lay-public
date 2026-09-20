use evdev::{uinput::VirtualDevice, Device, EventType, KeyCode};
use lay::keyboard::{is_typing_key, KeyEvent};
use lay::word_buffer::WordBuffer;
use std::os::fd::{AsRawFd, RawFd};
use std::time::{Duration, Instant};

use super::{
    emit_closed_key_chord, emit_left_shift_pair_fast, log, virtual_keyboard_supports_key,
    wait_for_keyboard_event_or_timeout, DShiftRelease, DShiftState,
};

const MODIFIER_SETTLE_BUDGET: Duration = Duration::from_millis(12);
const QUEUED_DRAIN_BUDGET: Duration = Duration::from_millis(20);

type ManualToggleReplay<'a> =
    dyn FnMut(&mut VirtualDevice, &mut WordBuffer, bool) -> Option<bool> + 'a;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
struct CapturedModifiers {
    left_shift: bool,
    right_shift: bool,
    left_ctrl: bool,
    right_ctrl: bool,
    left_alt: bool,
    right_alt: bool,
    left_meta: bool,
    right_meta: bool,
}

fn should_wait_for_modifier_settle(modifiers: CapturedModifiers, elapsed: Duration) -> bool {
    modifiers.any_active() && elapsed < MODIFIER_SETTLE_BUDGET
}

fn queued_drain_budget_expired(elapsed: Duration) -> bool {
    elapsed >= QUEUED_DRAIN_BUDGET
}

fn wait_for_unresolved_modifier(
    device_fd: RawFd,
    modifiers: CapturedModifiers,
    elapsed: Duration,
) -> std::io::Result<bool> {
    if !should_wait_for_modifier_settle(modifiers, elapsed) {
        return Ok(false);
    }
    wait_for_keyboard_event_or_timeout(device_fd, MODIFIER_SETTLE_BUDGET.saturating_sub(elapsed))?;
    Ok(true)
}

impl CapturedModifiers {
    fn observe(&mut self, key: KeyCode, value: i32) -> bool {
        let down = value != 0;
        let slot = match key {
            KeyCode::KEY_LEFTSHIFT => &mut self.left_shift,
            KeyCode::KEY_RIGHTSHIFT => &mut self.right_shift,
            KeyCode::KEY_LEFTCTRL => &mut self.left_ctrl,
            KeyCode::KEY_RIGHTCTRL => &mut self.right_ctrl,
            KeyCode::KEY_LEFTALT => &mut self.left_alt,
            KeyCode::KEY_RIGHTALT => &mut self.right_alt,
            KeyCode::KEY_LEFTMETA => &mut self.left_meta,
            KeyCode::KEY_RIGHTMETA => &mut self.right_meta,
            _ => return false,
        };
        *slot = down;
        true
    }

    const fn shift_active(self) -> bool {
        self.left_shift || self.right_shift
    }

    const fn command_modifier_active(self) -> bool {
        self.left_ctrl
            || self.right_ctrl
            || self.left_alt
            || self.right_alt
            || self.left_meta
            || self.right_meta
    }

    const fn any_active(self) -> bool {
        self.shift_active() || self.command_modifier_active()
    }

    fn active_keys(self) -> Vec<KeyCode> {
        let mut keys = Vec::with_capacity(8);
        for (active, key) in [
            (self.left_shift, KeyCode::KEY_LEFTSHIFT),
            (self.right_shift, KeyCode::KEY_RIGHTSHIFT),
            (self.left_ctrl, KeyCode::KEY_LEFTCTRL),
            (self.right_ctrl, KeyCode::KEY_RIGHTCTRL),
            (self.left_alt, KeyCode::KEY_LEFTALT),
            (self.right_alt, KeyCode::KEY_RIGHTALT),
            (self.left_meta, KeyCode::KEY_LEFTMETA),
            (self.right_meta, KeyCode::KEY_RIGHTMETA),
        ] {
            if active {
                keys.push(key);
            }
        }
        keys
    }
}

fn physical_modifier_is_held(device: &Device) -> std::io::Result<bool> {
    let keys = device.get_key_state()?;
    Ok([
        KeyCode::KEY_LEFTSHIFT,
        KeyCode::KEY_RIGHTSHIFT,
        KeyCode::KEY_LEFTCTRL,
        KeyCode::KEY_RIGHTCTRL,
        KeyCode::KEY_LEFTALT,
        KeyCode::KEY_RIGHTALT,
        KeyCode::KEY_LEFTMETA,
        KeyCode::KEY_RIGHTMETA,
    ]
    .into_iter()
    .any(|key| keys.contains(key)))
}

pub(super) struct PhysicalInputGrab<'a> {
    device: Option<&'a mut Device>,
    active: bool,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(super) struct ForwardedTyping {
    pub(super) keys: usize,
    pub(super) spaces: usize,
    pub(super) boundaries: usize,
    pub(super) manual_toggles: usize,
    pub(super) last_manual_toggle_layout_is_ru: Option<bool>,
}

fn observe_queued_double_shift(
    state: &mut DShiftState,
    key: KeyCode,
    value: i32,
    now: Instant,
    shift_window: Duration,
) -> bool {
    if key != KeyCode::KEY_LEFTSHIFT {
        if value == 1 {
            state.cancel();
        }
        return false;
    }
    match value {
        1 => state.trigger_press(now, shift_window),
        0 => return state.trigger_release(now) == DShiftRelease::Double,
        _ => {}
    }
    false
}

fn dispatch_completed_queued_left_shift_pair<R, T, E, Emit, Replay>(
    resource: &mut R,
    current_layout_is_ru: bool,
    mut emit: Emit,
    mut replay: Replay,
) -> Result<T, E>
where
    Emit: FnMut(&mut R) -> Result<(), E>,
    Replay: FnMut(&mut R, bool) -> T,
{
    emit(resource)?;
    Ok(replay(resource, current_layout_is_ru))
}

#[cfg(test)]
fn forward_queued_typing_key<E, Emit>(
    buf: &mut WordBuffer,
    key: KeyCode,
    shift: bool,
    layout_is_ru: bool,
    emit: Emit,
) -> Result<(), E>
where
    Emit: FnOnce() -> Result<(), E>,
{
    emit()?;
    buf.push(KeyEvent {
        keycode: key.code(),
        shift,
        layout_is_ru,
    });
    Ok(())
}

#[cfg(test)]
fn forward_queued_boundary<E, Emit>(buf: &mut WordBuffer, emit: Emit) -> Result<(), E>
where
    Emit: FnOnce() -> Result<(), E>,
{
    emit()?;
    buf.reset_all();
    Ok(())
}

impl<'a> PhysicalInputGrab<'a> {
    pub(super) fn new(device: Option<&'a mut Device>) -> Self {
        let Some(device) = device else {
            return Self {
                device: None,
                active: false,
            };
        };

        match physical_modifier_is_held(device) {
            Ok(true) => {
                log("· physical input grab skipped: modifier is already held");
                return Self {
                    device: Some(device),
                    active: false,
                };
            }
            Ok(false) => {}
            Err(error) => log(&format!(
                "⚠ physical modifier state unavailable before grab: {error}"
            )),
        }

        match device.grab() {
            Ok(()) => Self {
                device: Some(device),
                active: true,
            },
            Err(e) => {
                log(&format!(
                    "⚠ physical device grab failed: {e}; continuing without input isolation"
                ));
                Self {
                    device: Some(device),
                    active: false,
                }
            }
        }
    }

    pub(super) fn is_active(&self) -> bool {
        self.active
    }

    pub(super) fn forward_queued_typing(
        &mut self,
        virtual_kbd: &mut VirtualDevice,
        buf: &mut WordBuffer,
        layout_is_ru: bool,
        label: &str,
        skip_spaces: usize,
        forward_boundaries: bool,
    ) -> ForwardedTyping {
        let mut ignored_trigger_state = DShiftState::Idle;
        self.forward_queued_input(
            virtual_kbd,
            buf,
            layout_is_ru,
            label,
            skip_spaces,
            forward_boundaries,
            Duration::ZERO,
            &mut ignored_trigger_state,
            None,
        )
    }

    #[expect(
        clippy::too_many_arguments,
        reason = "input replay state remains explicit"
    )]
    pub(super) fn forward_queued_typing_with_manual_toggles(
        &mut self,
        virtual_kbd: &mut VirtualDevice,
        buf: &mut WordBuffer,
        layout_is_ru: bool,
        label: &str,
        skip_spaces: usize,
        forward_boundaries: bool,
        shift_window: Duration,
        dshift_state: &mut DShiftState,
        replay_manual_toggle: &mut ManualToggleReplay<'_>,
    ) -> ForwardedTyping {
        self.forward_queued_input(
            virtual_kbd,
            buf,
            layout_is_ru,
            label,
            skip_spaces,
            forward_boundaries,
            shift_window,
            dshift_state,
            Some(replay_manual_toggle),
        )
    }

    #[expect(
        clippy::too_many_arguments,
        reason = "input replay state remains explicit"
    )]
    fn forward_queued_input(
        &mut self,
        virtual_kbd: &mut VirtualDevice,
        buf: &mut WordBuffer,
        mut layout_is_ru: bool,
        label: &str,
        mut skip_spaces: usize,
        _forward_boundaries: bool,
        shift_window: Duration,
        dshift_state: &mut DShiftState,
        mut replay_manual_toggle: Option<&mut ManualToggleReplay<'_>>,
    ) -> ForwardedTyping {
        if !self.active {
            return ForwardedTyping::default();
        }

        let Some(device) = self.device.as_deref_mut() else {
            return ForwardedTyping::default();
        };

        let device_fd = device.as_raw_fd();
        let drain_started = Instant::now();
        let mut modifiers = CapturedModifiers::default();
        let mut forwarded = ForwardedTyping::default();

        loop {
            let events = match device.fetch_events() {
                Ok(events) => events.collect::<Vec<_>>(),
                Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                    match wait_for_unresolved_modifier(
                        device_fd,
                        modifiers,
                        drain_started.elapsed(),
                    ) {
                        Ok(true) => continue,
                        Ok(false) => break,
                        Err(error) => {
                            log(&format!(
                                "warning: {label} modifier-settle wait failed: {error}"
                            ));
                            break;
                        }
                    }
                }
                Err(error) => {
                    log(&format!("⚠ {label} passthrough read failed: {error}"));
                    break;
                }
            };

            if events.is_empty() {
                match wait_for_unresolved_modifier(device_fd, modifiers, drain_started.elapsed()) {
                    Ok(true) => continue,
                    Ok(false) => break,
                    Err(error) => {
                        log(&format!(
                            "warning: {label} modifier-settle wait failed: {error}"
                        ));
                        break;
                    }
                }
            }

            for event in events {
                if event.event_type() != EventType::KEY {
                    continue;
                }
                let key = KeyCode::new(event.code());
                let value = event.value();

                if modifiers.observe(key, value) {
                    match key {
                        KeyCode::KEY_LEFTSHIFT => {
                            if observe_queued_double_shift(
                                dshift_state,
                                key,
                                value,
                                Instant::now(),
                                shift_window,
                            ) {
                                if let Some(replay) = replay_manual_toggle.as_deref_mut() {
                                    let mut resources = (&mut *virtual_kbd, &mut *buf);
                                    let dispatched = dispatch_completed_queued_left_shift_pair(
                                        &mut resources,
                                        layout_is_ru,
                                        |resources| emit_left_shift_pair_fast(resources.0),
                                        |resources, current_layout| {
                                            replay(resources.0, resources.1, current_layout)
                                        },
                                    );
                                    match dispatched {
                                        Ok(Some(target_layout_is_ru)) => {
                                            layout_is_ru = target_layout_is_ru;
                                            forwarded.manual_toggles += 1;
                                            forwarded.last_manual_toggle_layout_is_ru =
                                                Some(target_layout_is_ru);
                                        }
                                        Ok(None) => {}
                                        Err(error) => log(&format!(
                                            "warning: {label} queued Double Shift emit failed: {error}"
                                        )),
                                    }
                                }
                            }
                        }
                        _ => {
                            let _ = observe_queued_double_shift(
                                dshift_state,
                                key,
                                value,
                                Instant::now(),
                                shift_window,
                            );
                        }
                    }
                    continue;
                }

                if value != 1 && value != 2 {
                    continue;
                }

                let _ = observe_queued_double_shift(
                    dshift_state,
                    key,
                    value,
                    Instant::now(),
                    shift_window,
                );

                let command_modifier = modifiers.command_modifier_active();

                if key == KeyCode::KEY_SPACE {
                    if skip_spaces > 0 && !command_modifier {
                        skip_spaces -= 1;
                        continue;
                    }
                    let active_modifiers = modifiers.active_keys();
                    if let Err(error) = emit_closed_key_chord(virtual_kbd, &active_modifiers, key) {
                        log(&format!("⚠ {label} passthrough space failed: {error}"));
                        continue;
                    }
                    if command_modifier {
                        buf.reset_all();
                        forwarded.boundaries += 1;
                    } else {
                        buf.handle_space();
                        forwarded.spaces += 1;
                    }
                    continue;
                }

                if virtual_keyboard_supports_key(key) && !is_typing_key(key) {
                    let active_modifiers = modifiers.active_keys();
                    if let Err(error) = emit_closed_key_chord(virtual_kbd, &active_modifiers, key) {
                        log(&format!(
                            "warning: {label} passthrough boundary failed: {error}"
                        ));
                        continue;
                    }
                    buf.reset_all();
                    forwarded.boundaries += 1;
                    continue;
                }

                if !is_typing_key(key) {
                    continue;
                }

                let active_modifiers = modifiers.active_keys();
                if let Err(error) = emit_closed_key_chord(virtual_kbd, &active_modifiers, key) {
                    log(&format!("⚠ {label} passthrough key failed: {error}"));
                    continue;
                }

                if command_modifier {
                    // Ctrl/Alt/Meta chords are commands, not inserted text. The
                    // visible cursor/content may have changed, so retire the
                    // daemon word mirror instead of inventing typed characters.
                    buf.reset_all();
                    forwarded.boundaries += 1;
                } else {
                    buf.push(KeyEvent {
                        keycode: key.code(),
                        shift: modifiers.shift_active(),
                        layout_is_ru,
                    });
                    forwarded.keys += 1;
                }
            }

            if queued_drain_budget_expired(drain_started.elapsed()) {
                log(&format!(
                    "⚠ {label} passthrough drain reached {}ms budget",
                    QUEUED_DRAIN_BUDGET.as_millis()
                ));
                break;
            }
        }

        if modifiers.any_active() {
            log(&format!(
                "⚠ {label} passthrough released grab with an unresolved physical modifier after bounded drain"
            ));
        }

        if forwarded.keys + forwarded.spaces + forwarded.boundaries + forwarded.manual_toggles > 0 {
            log(&format!(
                "· {label} passthrough forwarded {} queued keys, {} spaces, {} boundaries, {} manual toggles",
                forwarded.keys,
                forwarded.spaces,
                forwarded.boundaries,
                forwarded.manual_toggles
            ));
        }
        forwarded
    }
}

impl Drop for PhysicalInputGrab<'_> {
    fn drop(&mut self) {
        if self.active {
            if let Some(device) = self.device.as_deref_mut() {
                if let Err(e) = device.ungrab() {
                    log(&format!("⚠ physical device ungrab failed: {e}"));
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        dispatch_completed_queued_left_shift_pair, forward_queued_boundary,
        forward_queued_typing_key, observe_queued_double_shift, queued_drain_budget_expired,
        should_wait_for_modifier_settle, virtual_keyboard_supports_key, CapturedModifiers,
        MODIFIER_SETTLE_BUDGET, QUEUED_DRAIN_BUDGET,
    };
    use crate::DShiftState;
    use evdev::KeyCode;
    use lay::word_buffer::WordBuffer;
    use std::time::{Duration, Instant};

    fn feed(state: &mut DShiftState, events: &[(KeyCode, i32)], start: Instant) -> usize {
        events
            .iter()
            .enumerate()
            .filter(|(index, (key, value))| {
                observe_queued_double_shift(
                    state,
                    *key,
                    *value,
                    start + Duration::from_millis(*index as u64 * 10),
                    Duration::from_millis(250),
                )
            })
            .count()
    }

    #[test]
    fn shared_fsm_preserves_every_internal_partition_without_virtual_partial_output() {
        let pair = [
            (KeyCode::KEY_LEFTSHIFT, 1),
            (KeyCode::KEY_LEFTSHIFT, 0),
            (KeyCode::KEY_LEFTSHIFT, 1),
            (KeyCode::KEY_LEFTSHIFT, 0),
        ];
        for split in 1..4 {
            let mut state = DShiftState::Idle;
            let start = Instant::now();
            assert_eq!(feed(&mut state, &pair[..split], start), 0);
            assert_eq!(
                feed(
                    &mut state,
                    &pair[split..],
                    start + Duration::from_millis(40)
                ),
                1
            );
            assert!(state.is_idle());
        }
    }

    #[test]
    fn repeats_do_not_create_or_cancel_taps_and_other_press_cancels() {
        let mut state = DShiftState::Idle;
        let repeated_pair = [
            (KeyCode::KEY_LEFTSHIFT, 1),
            (KeyCode::KEY_LEFTSHIFT, 2),
            (KeyCode::KEY_LEFTSHIFT, 0),
            (KeyCode::KEY_LEFTSHIFT, 1),
            (KeyCode::KEY_LEFTSHIFT, 2),
            (KeyCode::KEY_LEFTSHIFT, 0),
        ];
        assert_eq!(feed(&mut state, &repeated_pair, Instant::now()), 1);
        let interrupted = [
            (KeyCode::KEY_LEFTSHIFT, 1),
            (KeyCode::KEY_LEFTSHIFT, 0),
            (KeyCode::KEY_A, 1),
            (KeyCode::KEY_LEFTSHIFT, 1),
            (KeyCode::KEY_LEFTSHIFT, 0),
        ];
        assert_eq!(feed(&mut state, &interrupted, Instant::now()), 0);
    }

    #[test]
    fn balanced_pair_emits_once_before_callback_and_failure_blocks_callback() {
        let mut observed = Vec::new();
        let ok = dispatch_completed_queued_left_shift_pair(
            &mut observed,
            false,
            |seen| {
                seen.extend([1, 0, 1, 0]);
                Ok::<(), &'static str>(())
            },
            |seen, layout| {
                seen.push(9);
                layout
            },
        );
        assert_eq!(ok, Ok(false));
        assert_eq!(observed, [1, 0, 1, 0, 9]);

        let mut callbacks = 0;
        let failed = dispatch_completed_queued_left_shift_pair(
            &mut callbacks,
            true,
            |_| Err::<(), _>("emit failed"),
            |count, _| {
                *count += 1;
                true
            },
        );
        assert_eq!(failed, Err("emit failed"));
        assert_eq!(callbacks, 0);
    }

    #[test]
    fn modifier_replay_is_bounded_and_tracks_command_chords() {
        assert!(MODIFIER_SETTLE_BUDGET <= Duration::from_millis(20));
        assert!(QUEUED_DRAIN_BUDGET <= Duration::from_millis(25));
        assert!(MODIFIER_SETTLE_BUDGET < QUEUED_DRAIN_BUDGET);

        let mut modifiers = CapturedModifiers::default();
        assert!(modifiers.observe(KeyCode::KEY_LEFTCTRL, 1));
        assert!(modifiers.command_modifier_active());
        assert!(modifiers.any_active());
        assert_eq!(modifiers.active_keys(), [KeyCode::KEY_LEFTCTRL]);

        assert!(modifiers.observe(KeyCode::KEY_LEFTSHIFT, 1));
        assert!(modifiers.shift_active());
        assert_eq!(
            modifiers.active_keys(),
            [KeyCode::KEY_LEFTSHIFT, KeyCode::KEY_LEFTCTRL]
        );

        assert!(modifiers.observe(KeyCode::KEY_LEFTCTRL, 0));
        assert!(!modifiers.command_modifier_active());
        assert!(modifiers.shift_active());
        assert!(modifiers.observe(KeyCode::KEY_LEFTSHIFT, 0));
        assert!(!modifiers.any_active());

        assert!(!modifiers.observe(KeyCode::KEY_ENTER, 1));

        let settle_edge = MODIFIER_SETTLE_BUDGET;
        assert!(should_wait_for_modifier_settle(
            CapturedModifiers {
                left_shift: true,
                ..CapturedModifiers::default()
            },
            settle_edge.saturating_sub(Duration::from_nanos(1))
        ));
        assert!(!should_wait_for_modifier_settle(
            CapturedModifiers {
                left_shift: true,
                ..CapturedModifiers::default()
            },
            settle_edge
        ));
        assert!(!should_wait_for_modifier_settle(
            CapturedModifiers::default(),
            Duration::ZERO
        ));

        assert!(!queued_drain_budget_expired(
            QUEUED_DRAIN_BUDGET.saturating_sub(Duration::from_nanos(1))
        ));
        assert!(queued_drain_budget_expired(QUEUED_DRAIN_BUDGET));
        assert!(queued_drain_budget_expired(
            QUEUED_DRAIN_BUDGET + Duration::from_millis(1)
        ));

        for key in [
            KeyCode::KEY_PAGEUP,
            KeyCode::KEY_PAGEDOWN,
            KeyCode::KEY_ENTER,
            KeyCode::KEY_LEFT,
        ] {
            assert!(
                virtual_keyboard_supports_key(key),
                "captured command must be representable by the virtual keyboard: {key:?}"
            );
        }
    }

    #[test]
    fn multiple_pairs_receive_current_layout_and_word_buffer_remains_fifo() {
        let mut layouts = Vec::new();
        let mut current = false;
        for expected in [false, true, false] {
            assert_eq!(current, expected);
            let result = dispatch_completed_queued_left_shift_pair(
                &mut layouts,
                current,
                |_| Ok::<(), &'static str>(()),
                |seen, layout| {
                    seen.push(layout);
                    Some(!layout)
                },
            );
            assert_eq!(result, Ok(Some(!current)));
            current = result.unwrap().unwrap();
        }
        assert_eq!(layouts, [false, true, false]);

        let mut buffer = WordBuffer::new();
        let mut emitted = Vec::new();
        forward_queued_typing_key(&mut buffer, KeyCode::KEY_A, false, false, || {
            emitted.push(KeyCode::KEY_A);
            Ok::<(), &'static str>(())
        })
        .unwrap();
        forward_queued_typing_key(&mut buffer, KeyCode::KEY_B, true, true, || {
            emitted.push(KeyCode::KEY_B);
            Ok::<(), &'static str>(())
        })
        .unwrap();
        assert_eq!(emitted, [KeyCode::KEY_A, KeyCode::KEY_B]);
        assert_eq!(buffer.current_len(), 2);
        assert_eq!(buffer.current_last_keycode(), Some(KeyCode::KEY_B.code()));
        forward_queued_boundary(&mut buffer, || {
            emitted.push(KeyCode::KEY_ENTER);
            Ok::<(), &'static str>(())
        })
        .unwrap();
        assert_eq!(
            emitted,
            [KeyCode::KEY_A, KeyCode::KEY_B, KeyCode::KEY_ENTER]
        );
        assert_eq!(buffer.current_len(), 0);
    }
}

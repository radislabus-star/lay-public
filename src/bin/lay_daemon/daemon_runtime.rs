use evdev::{uinput::VirtualDevice, Device, EventType, InputEvent, KeyCode};
use lay::config::LayConfig;
use lay::keyboard::is_typing_key;
use std::os::fd::AsRawFd;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use super::boundary_runtime::{
    handle_hard_boundary_if_needed, handle_space_press, note_learning_backspace_if_needed,
    try_handle_deferred_typing_assist, try_handle_enter_autocorrect, try_handle_space_release,
    DeferredTypingAssistContext, EnterAutocorrectContext, HardBoundaryContext, SpacePressContext,
    SpaceReleaseContext,
};
use super::buffer_filter_runtime::BufferFilterContext;
use super::daemon_state::DaemonLoopState;
use super::manual_trigger_runtime::{
    fire_expired_pending_multi_tap, handle_manual_trigger_event, ManualTriggerEventContext,
    PendingMultiTapTimeoutContext,
};
use super::pending_typing_assist::drop_pending_after_following_word_started;
use super::text_context_runtime::should_advance_text_context;
use super::trigger_dispatch::{is_single_trigger_id, trigger_key_from_config};
use super::typing_key_runtime::{handle_typing_key_press, TypingKeyContext};
use super::{
    active_enter_autocorrect_from_env, active_layout_backend, idle_wait_timeout, log,
    should_skip_buffer_input, switch_to_target_layout, wait_for_keyboard_event_or_timeout,
    DShiftState, ForceLayoutHotkeyContext, ShiftState, ENTER_AUTOCORRECT_EXPERIMENT_ENV,
};

#[path = "daemon_runtime/focus.rs"]
mod focus;
pub(super) use focus::listen_pointer;
use focus::{sync_field_context_epoch, update_focus_state, update_focus_state_for_key_batch};

pub(super) fn listen_keyboard(
    device_path: std::path::PathBuf,
    virtual_kbd: Arc<Mutex<Option<VirtualDevice>>>,
    field_context_epoch: Arc<AtomicU64>,
    verbose: bool,
    cfg: LayConfig,
) -> std::io::Result<()> {
    let mut device = Device::open(&device_path)?;
    device.set_nonblocking(true)?;
    let device_fd = device.as_raw_fd();
    log(&format!(
        "► слушаю: {device_path:?} имя={:?}",
        device.name().unwrap_or("?")
    ));
    let enter_autocorrect_active = active_enter_autocorrect_from_env(
        cfg.enter_autocorrect,
        std::env::var(ENTER_AUTOCORRECT_EXPERIMENT_ENV)
            .ok()
            .as_deref(),
    );
    log(&format!(
        "► config: mode={} backend={} replace_words={} auto_replace={} typing_assist={} enter_autocorrect={} auto_switch_layout={} lem2={} lem3={} trigger={} force_layout={} ru_key={} en_key={} multi_tap={} max_taps={} tap={}ms window={}ms debounce={}ms",
        cfg.mode,
        active_layout_backend().label(),
        cfg.replace_words,
        cfg.auto_replace,
        cfg.typing_assist,
        enter_autocorrect_active,
        cfg.auto_switch_layout,
        cfg.lem_2_words,
        cfg.lem_3_words,
        cfg.trigger,
        cfg.force_layout_hotkeys,
        cfg.force_ru_key,
        cfg.force_en_key,
        cfg.multi_tap_scope,
        cfg.active_multi_tap_max_taps(),
        cfg.tap_max_ms,
        cfg.shift_window_ms,
        cfg.debounce_ms
    ));

    let trigger_key = trigger_key_from_config(&cfg.trigger);
    let is_caps_trigger = cfg.trigger == "caps-lock";
    let is_single_trigger = is_single_trigger_id(&cfg.trigger);
    let mut state = DaemonLoopState::new(&cfg, is_caps_trigger, is_single_trigger);

    loop {
        let fetched_events = {
            device
                .fetch_events()
                .map(|events| events.collect::<Vec<_>>())
        };
        let events: Vec<InputEvent> = match fetched_events {
            Ok(events) => events,
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                update_focus_state(&mut state);
                if state.focus_ignored {
                    wait_for_keyboard_event_or_timeout(
                        device_fd,
                        idle_wait_timeout(
                            state.pending_multi_tap.as_ref(),
                            state.last_focus_ignore_poll,
                            state.shift_window,
                        ),
                    )?;
                    continue;
                }
                if try_handle_deferred_typing_assist(DeferredTypingAssistContext {
                    buffer: &mut state.buffer,
                    device: &mut device,
                    virtual_kbd: &virtual_kbd,
                    executing: &mut state.executing,
                    pending_typing_assist_after_space: &mut state.pending_typing_assist_after_space,
                    typing_assist_worker: &state.typing_assist_worker,
                    current_layout_is_ru: &mut state.current_layout_is_ru,
                    last_layout_poll: &mut state.last_layout_poll,
                    shift_state: &state.shift_state,
                }) {
                    continue;
                }
                if state.multi_tap_scope
                    && state
                        .pending_multi_tap
                        .is_some_and(|pending| pending.last_release.elapsed() >= state.shift_window)
                {
                    fire_expired_pending_multi_tap(PendingMultiTapTimeoutContext {
                        buffer: &mut state.buffer,
                        device: &mut device,
                        virtual_kbd: &virtual_kbd,
                        executing: &mut state.executing,
                        current_layout_is_ru: &mut state.current_layout_is_ru,
                        last_layout_poll: &mut state.last_layout_poll,
                        suppress_next_typing_assist_after_manual_replay: &mut state
                            .suppress_next_typing_assist_after_manual_replay,
                        pending_typing_assist_after_space: &mut state
                            .pending_typing_assist_after_space,
                        shift_state: &mut state.shift_state,
                        dshift_state: &mut state.dshift_state,
                        pending_multi_tap: &mut state.pending_multi_tap,
                        last_double_at: &mut state.last_double_at,
                        clear_on_next_typing: &mut state.clear_on_next_typing,
                        shift_window: state.shift_window,
                        events_since_word_start: state.events_since_word_start,
                    });
                }
                wait_for_keyboard_event_or_timeout(
                    device_fd,
                    idle_wait_timeout(
                        state.pending_multi_tap.as_ref(),
                        state.last_focus_ignore_poll,
                        state.shift_window,
                    )
                    .min(
                        if state.pending_typing_assist_after_space.is_some() {
                            std::time::Duration::from_millis(2)
                        } else {
                            std::time::Duration::MAX
                        },
                    ),
                )?;
                continue;
            }
            Err(e) => return Err(e),
        };

        update_focus_state_for_key_batch(&events, &mut state);
        sync_field_context_epoch(&field_context_epoch, &mut state);
        if state.focus_ignored {
            state.shift_state = ShiftState::default();
            state.dshift_state = DShiftState::Idle;
            state.pending_multi_tap = None;
            state.pending_typing_assist_after_space.take();
            state.ignore_current_token_until_space = false;
            continue;
        }

        for (event_idx, event) in events.iter().enumerate() {
            if event.event_type() != EventType::KEY {
                continue;
            }
            let code = event.code();
            let value = event.value();
            let key = KeyCode::new(code);

            // ─── флаг выполнения: пока идёт замена — все события в игнор ───
            // Clutter virtual device (TypeText fallback) создаёт evdev-устройство
            // которое мы тоже слушаем → feedback loop: TypeText-события попадают
            // обратно в буфер. Блокируем ВСЕ ключи пока state.executing=true.
            // modifier state обновляем всё равно — чтобы не рассинхронизироваться.
            if state.executing {
                state.shift_state.update(key, value);
                continue;
            }

            // ─── modifier tracking ────────────────────────────
            state.shift_state.update(key, value);
            if handle_alt_shift_layout_switch(key, value, &mut state) {
                continue;
            }
            if should_advance_text_context(key, value, &state.shift_state) {
                let epoch = field_context_epoch.fetch_add(1, Ordering::Relaxed) + 1;
                if state.switch_field_context_epoch(epoch) && verbose {
                    log("► text context changed: switched field buffer");
                }
            }
            if state.force_layout_hotkeys.handle_event(
                key,
                value,
                ForceLayoutHotkeyContext {
                    buffer: &mut state.buffer,
                    virtual_kbd: &virtual_kbd,
                    executing: &mut state.executing,
                    current_layout_is_ru: &mut state.current_layout_is_ru,
                    last_layout_poll: &mut state.last_layout_poll,
                    shift_state: &mut state.shift_state,
                    dshift_state: &mut state.dshift_state,
                    pending_multi_tap: &mut state.pending_multi_tap,
                    single_pressed_at: &mut state.single_pressed_at,
                    last_double_at: &mut state.last_double_at,
                    clear_on_next_typing: &mut state.clear_on_next_typing,
                    shift_tap_max: state.shift_tap_max,
                    debounce_window: state.debounce_window,
                },
            ) {
                continue;
            }

            if handle_manual_trigger_event(ManualTriggerEventContext {
                key,
                code,
                value,
                verbose,
                trigger_key,
                is_caps_trigger,
                is_single_trigger,
                shift_tap_max: state.shift_tap_max,
                shift_window: state.shift_window,
                debounce_window: state.debounce_window,
                multi_tap_scope: state.multi_tap_scope,
                multi_tap_max_taps: state.multi_tap_max_taps,
                events_since_word_start: state.events_since_word_start,
                buffer: &mut state.buffer,
                device: &mut device,
                virtual_kbd: &virtual_kbd,
                executing: &mut state.executing,
                current_layout_is_ru: &mut state.current_layout_is_ru,
                last_layout_poll: &mut state.last_layout_poll,
                suppress_next_typing_assist_after_manual_replay: &mut state
                    .suppress_next_typing_assist_after_manual_replay,
                pending_typing_assist_after_space: &mut state.pending_typing_assist_after_space,
                shift_state: &mut state.shift_state,
                dshift_state: &mut state.dshift_state,
                pending_multi_tap: &mut state.pending_multi_tap,
                last_double_at: &mut state.last_double_at,
                clear_on_next_typing: &mut state.clear_on_next_typing,
                single_pressed_at: &mut state.single_pressed_at,
                single_other_key: &mut state.single_other_key,
            }) {
                continue;
            }

            if try_handle_space_release(
                key,
                value,
                SpaceReleaseContext {
                    events: &events,
                    event_idx,
                    buffer: &mut state.buffer,
                    pending_typing_assist_after_space: &mut state.pending_typing_assist_after_space,
                    shift_state: &state.shift_state,
                    verbose,
                },
            ) {
                continue;
            }

            // release не интересен — пропускаем
            if value == 0 {
                continue;
            }

            if should_skip_buffer_input(BufferFilterContext {
                key,
                code,
                shift_state: &state.shift_state,
                current_empty: state.buffer.current_is_empty(),
                ignore_current_token_until_space: &mut state.ignore_current_token_until_space,
                events_since_word_start: &mut state.events_since_word_start,
                pending_typing_assist_after_space: &mut state.pending_typing_assist_after_space,
                verbose,
            }) {
                continue;
            }

            // ─── пробел: переносим current → prev (только на press) ──
            if key == KeyCode::KEY_SPACE {
                if value == 1 {
                    handle_space_press(SpacePressContext {
                        buffer: &mut state.buffer,
                        pending_typing_assist_after_space: &mut state
                            .pending_typing_assist_after_space,
                        typing_assist_worker: &mut state.typing_assist_worker,
                        events_since_word_start: &mut state.events_since_word_start,
                        suppress_next_typing_assist_after_manual_replay: &mut state
                            .suppress_next_typing_assist_after_manual_replay,
                        verbose,
                    });
                }
                continue;
            }

            // ─── граница (Enter/Tab/Esc/стрелки/BS/Del) — сброс на press ──
            note_learning_backspace_if_needed(key, value, &mut state.buffer);
            if try_handle_enter_autocorrect(
                key,
                value,
                EnterAutocorrectContext {
                    buffer: &mut state.buffer,
                    device: &mut device,
                    virtual_kbd: &virtual_kbd,
                    executing: &mut state.executing,
                    current_layout_is_ru: &mut state.current_layout_is_ru,
                    last_layout_poll: &mut state.last_layout_poll,
                    pending_typing_assist_after_space: &mut state.pending_typing_assist_after_space,
                    ignore_current_token_until_space: &mut state.ignore_current_token_until_space,
                    events_since_word_start: &mut state.events_since_word_start,
                    clear_on_next_typing: &mut state.clear_on_next_typing,
                },
            ) {
                continue;
            }
            if handle_hard_boundary_if_needed(
                key,
                value,
                HardBoundaryContext {
                    buffer: &mut state.buffer,
                    pending_typing_assist_after_space: &mut state.pending_typing_assist_after_space,
                    ignore_current_token_until_space: &mut state.ignore_current_token_until_space,
                    events_since_word_start: &mut state.events_since_word_start,
                    verbose,
                },
            ) {
                continue;
            }

            // ─── обычный символ ─────
            if is_typing_key(key) {
                if drop_pending_after_following_word_started(
                    &mut state.pending_typing_assist_after_space,
                ) {
                    log("· typing-assist pending dropped: following word started");
                }
                handle_typing_key_press(
                    code,
                    value,
                    TypingKeyContext {
                        buffer: &mut state.buffer,
                        shift_state: &state.shift_state,
                        current_layout_is_ru: &mut state.current_layout_is_ru,
                        last_layout_poll: &mut state.last_layout_poll,
                        events_since_word_start: &mut state.events_since_word_start,
                        clear_on_next_typing: &mut state.clear_on_next_typing,
                        ignore_current_token_until_space: &mut state
                            .ignore_current_token_until_space,
                        suppress_next_typing_assist_after_manual_replay: &mut state
                            .suppress_next_typing_assist_after_manual_replay,
                        verbose,
                    },
                );
            }
        }
    }
}

fn handle_alt_shift_layout_switch(key: KeyCode, value: i32, state: &mut DaemonLoopState) -> bool {
    if !matches!(
        key,
        KeyCode::KEY_LEFTSHIFT
            | KeyCode::KEY_RIGHTSHIFT
            | KeyCode::KEY_LEFTALT
            | KeyCode::KEY_RIGHTALT
    ) {
        return false;
    }

    if state.shift_state.any() && state.shift_state.alt_active() {
        if state.alt_shift_layout_before.is_none() {
            let before = state.current_layout_is_ru;
            state.alt_shift_layout_before = Some(before);
            log(&format!(
                "· alt-shift layout switch pending from {}",
                if before { "ru" } else { "us" }
            ));
        }
        return true;
    }

    if value == 0 {
        let Some(before) = state.alt_shift_layout_before.take() else {
            return false;
        };
        let current = super::read_current_layout_is_ru().unwrap_or(before);
        let target = alt_shift_target_layout(before, current);
        match switch_to_target_layout(target) {
            Ok(layout_id) => {
                state.current_layout_is_ru = target;
                state.last_layout_poll = std::time::Instant::now();
                state.buffer.reset_all();
                state.pending_typing_assist_after_space.take();
                state.clear_on_next_typing = true;
                log(&format!("✓ alt-shift layout → {layout_id}"));
            }
            Err(error) => log(&format!("⚠ alt-shift layout switch failed: {error}")),
        }
        return true;
    }

    false
}

fn alt_shift_target_layout(before: bool, current: bool) -> bool {
    if current == before {
        !before
    } else {
        current
    }
}

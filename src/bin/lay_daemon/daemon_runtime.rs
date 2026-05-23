use evdev::{uinput::VirtualDevice, Device, EventType, InputEvent, KeyCode};
use lay::config::LayConfig;
use lay::keyboard::is_typing_key;
use lay::word_buffer::WordBuffer;
use std::os::fd::AsRawFd;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use super::boundary_runtime::{
    handle_hard_boundary_if_needed, handle_space_press, note_learning_backspace_if_needed,
    try_handle_enter_autocorrect, try_handle_space_release, EnterAutocorrectContext,
    HardBoundaryContext, SpacePressContext, SpaceReleaseContext,
};
use super::trigger_dispatch::{
    apply_manual_correction_result, is_single_trigger_id, run_configured_manual_correction,
    run_scoped_manual_correction, trigger_key_from_config,
};
use super::typing_key_runtime::{handle_typing_key_press, TypingKeyContext};
use super::{
    active_enter_autocorrect_from_env, active_layout_backend, handle_force_layout_hotkey,
    idle_wait_timeout, is_hard_boundary, lock_virtual_keyboard, log, multi_tap_scope_for_taps,
    read_current_layout_is_ru, should_ignore_buffer_key, should_start_ignored_buffer_token,
    single_hotkey_keycode, update_focus_ignore_state, wait_for_keyboard_event_or_timeout,
    DShiftState, MultiTapPending, ShiftState, ENTER_AUTOCORRECT_EXPERIMENT_ENV,
    FOCUS_IGNORE_POLL_INTERVAL_MS,
};

pub(super) fn listen_keyboard(
    device_path: std::path::PathBuf,
    virtual_kbd: Arc<Mutex<Option<VirtualDevice>>>,
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
    // Одиночный триггер: нажал и отпустил без других клавиш — конвертация
    let is_single_trigger = is_single_trigger_id(&cfg.trigger);
    let mut single_pressed_at: Option<Instant> = None; // когда нажата single-клавиша
    let mut single_other_key = false; // была ли другая клавиша пока держали
    let force_ru_key = single_hotkey_keycode(&cfg.force_ru_key);
    let force_en_key = single_hotkey_keycode(&cfg.force_en_key);
    let force_layout_hotkeys = cfg.force_layout_hotkeys
        && force_ru_key.is_some()
        && force_en_key.is_some()
        && force_ru_key != force_en_key;
    let mut force_ru_pressed_at: Option<Instant> = None;
    let mut force_en_pressed_at: Option<Instant> = None;
    let mut force_other_key = false;
    let multi_tap_scope = cfg.multi_tap_scope && !is_caps_trigger && !is_single_trigger;
    let multi_tap_max_taps = cfg.active_multi_tap_max_taps();
    let mut pending_multi_tap: Option<MultiTapPending> = None;

    let mut buffer = WordBuffer::new();
    let mut shift_state = ShiftState::default();
    let mut dshift_state = DShiftState::Idle;
    let mut executing = false;
    let shift_tap_max = Duration::from_millis(cfg.tap_max_ms);
    let shift_window = Duration::from_millis(cfg.shift_window_ms);
    let debounce_window = Duration::from_millis(cfg.debounce_ms);
    let mut current_layout_is_ru = read_current_layout_is_ru().unwrap_or(false);
    let mut last_layout_poll = Instant::now();
    let mut last_double_at: Option<Instant> = None;
    // После DOUBLE буфер сохраняется (для toggle). Но как только пользователь
    // начнёт печатать НОВОЕ слово — нужно сбросить буфер чтобы новое слово
    // не приклеилось к предыдущему.
    let mut clear_on_next_typing: bool = false;
    let mut suppress_next_typing_assist_after_manual_replay: bool = false;
    // Перекрёстный счёт: считаем ВСЕ typing-events (press+repeat) с момента
    // последнего пробела/границы, независимо от accept-фильтра. На DOUBLE
    // сравниваем с buffer.current_len() — должны совпасть. Если нет —
    // видно где autorepeat терялся.
    let mut events_since_word_start: u32 = 0;
    let mut pending_typing_assist_after_space = false;
    let mut focus_ignored = false;
    let mut ignore_current_token_until_space = false;
    let mut last_focus_ignore_poll =
        Instant::now() - Duration::from_millis(FOCUS_IGNORE_POLL_INTERVAL_MS);

    loop {
        let fetched_events = {
            device
                .fetch_events()
                .map(|events| events.collect::<Vec<_>>())
        };
        let events: Vec<InputEvent> = match fetched_events {
            Ok(events) => events,
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                update_focus_ignore_state(
                    &mut focus_ignored,
                    &mut last_focus_ignore_poll,
                    &mut buffer,
                    &mut events_since_word_start,
                );
                if focus_ignored {
                    wait_for_keyboard_event_or_timeout(
                        device_fd,
                        idle_wait_timeout(
                            pending_multi_tap.as_ref(),
                            last_focus_ignore_poll,
                            shift_window,
                        ),
                    )?;
                    continue;
                }
                if multi_tap_scope
                    && pending_multi_tap
                        .is_some_and(|pending| pending.last_release.elapsed() >= shift_window)
                {
                    if let Some(pending) = pending_multi_tap.take() {
                        let replace_words =
                            multi_tap_scope_for_taps(pending.tap_count).unwrap_or(1);
                        let correction_result = run_scoped_manual_correction(
                            &mut buffer,
                            replace_words,
                            &mut device,
                            &virtual_kbd,
                            &mut executing,
                            events_since_word_start,
                            "multi-tap timeout",
                        );
                        apply_manual_correction_result(
                            correction_result,
                            &mut current_layout_is_ru,
                            &mut last_layout_poll,
                            &mut suppress_next_typing_assist_after_manual_replay,
                        );
                        shift_state.clear_shifts();
                        dshift_state = DShiftState::Idle;
                        last_double_at = Some(Instant::now());
                        clear_on_next_typing = true;
                    }
                }
                wait_for_keyboard_event_or_timeout(
                    device_fd,
                    idle_wait_timeout(
                        pending_multi_tap.as_ref(),
                        last_focus_ignore_poll,
                        shift_window,
                    ),
                )?;
                continue;
            }
            Err(e) => return Err(e),
        };

        update_focus_ignore_state(
            &mut focus_ignored,
            &mut last_focus_ignore_poll,
            &mut buffer,
            &mut events_since_word_start,
        );
        if focus_ignored {
            shift_state = ShiftState::default();
            dshift_state = DShiftState::Idle;
            pending_multi_tap = None;
            pending_typing_assist_after_space = false;
            ignore_current_token_until_space = false;
            continue;
        }

        for (event_idx, event) in events.iter().enumerate() {
            if event.event_type() != EventType::KEY {
                continue;
            }
            let code = event.code();
            let value = event.value();
            let key = KeyCode::new(code);

            // (тайм-аут очистки буфера убран — буфер чистится только на
            // явных границах: Enter/Tab/Esc/стрелки/Backspace/Delete)

            // ─── флаг выполнения: пока идёт замена — все события в игнор ───
            // Clutter virtual device (TypeText fallback) создаёт evdev-устройство
            // которое мы тоже слушаем → feedback loop: TypeText-события попадают
            // обратно в буфер. Блокируем ВСЕ ключи пока executing=true.
            // modifier state обновляем всё равно — чтобы не рассинхронизироваться.
            if executing {
                shift_state.update(key, value);
                continue;
            }

            // ─── modifier tracking ────────────────────────────
            shift_state.update(key, value);
            if force_layout_hotkeys {
                let force_target = if Some(key) == force_ru_key {
                    Some(true)
                } else if Some(key) == force_en_key {
                    Some(false)
                } else {
                    None
                };

                if let Some(target_is_ru) = force_target {
                    let pressed_at = if target_is_ru {
                        &mut force_ru_pressed_at
                    } else {
                        &mut force_en_pressed_at
                    };
                    match value {
                        1 => {
                            *pressed_at = Some(Instant::now());
                            force_other_key = false;
                        }
                        0 => {
                            if let Some(t) = pressed_at.take() {
                                let held = t.elapsed();
                                if !force_other_key
                                    && held <= shift_tap_max
                                    && last_double_at
                                        .map_or(true, |d| d.elapsed() >= debounce_window)
                                {
                                    let mut g = lock_virtual_keyboard(&virtual_kbd);
                                    let result = handle_force_layout_hotkey(
                                        target_is_ru,
                                        &mut buffer,
                                        g.as_mut(),
                                        &mut executing,
                                    );
                                    if let Some(is_ru) = result {
                                        current_layout_is_ru = is_ru;
                                        last_layout_poll = Instant::now();
                                    }
                                    drop(g);
                                    shift_state.clear_shifts();
                                    dshift_state = DShiftState::Idle;
                                    pending_multi_tap = None;
                                    single_pressed_at = None;
                                    last_double_at = Some(Instant::now());
                                    clear_on_next_typing = true;
                                    log(&format!(
                                        "· force-layout {} fired (held {}ms)",
                                        if target_is_ru { "RU" } else { "EN" },
                                        held.as_millis()
                                    ));
                                }
                            }
                        }
                        _ => {}
                    }
                    continue;
                } else if value == 1
                    && (force_ru_pressed_at.is_some() || force_en_pressed_at.is_some())
                {
                    force_other_key = true;
                }
            }

            // ═══ FSM: press→release→press→release = DOUBLE TRIGGER ════
            // RShift и RAlt не участвуют в FSM
            // Single-trigger клавиши не фильтруем (они сами и есть триггер)
            if !is_single_trigger
                && (key == KeyCode::KEY_RIGHTSHIFT || key == KeyCode::KEY_RIGHTALT)
            {
                continue;
            }

            // ─── Single trigger (правый Shift/Ctrl/Alt/Pause) ───────────
            // Нажал без других клавиш → отпустил ≤ tap_max → конвертация
            if is_single_trigger {
                if key == trigger_key {
                    match value {
                        1 => {
                            // press
                            single_pressed_at = Some(Instant::now());
                            single_other_key = false;
                        }
                        0 => {
                            // release
                            if let Some(t) = single_pressed_at.take() {
                                let held = t.elapsed();
                                if !single_other_key
                                    && held <= shift_tap_max
                                    && last_double_at
                                        .map_or(true, |d| d.elapsed() >= debounce_window)
                                {
                                    let buf_count = buffer.current_len() as u32;
                                    log(&format!(
                                        "═ CROSS-CHECK: buffer={} events={}{}",
                                        buf_count,
                                        events_since_word_start,
                                        if buf_count != events_since_word_start {
                                            " ⚠"
                                        } else {
                                            " ✓"
                                        }
                                    ));
                                    let correction_result = run_configured_manual_correction(
                                        &mut buffer,
                                        &mut device,
                                        &virtual_kbd,
                                        &mut executing,
                                    );
                                    apply_manual_correction_result(
                                        correction_result,
                                        &mut current_layout_is_ru,
                                        &mut last_layout_poll,
                                        &mut suppress_next_typing_assist_after_manual_replay,
                                    );
                                    shift_state.clear_shifts();
                                    last_double_at = Some(Instant::now());
                                    clear_on_next_typing = true;
                                    log(&format!(
                                        "· single-trigger fired (held {}ms)",
                                        held.as_millis()
                                    ));
                                }
                            }
                        }
                        _ => {}
                    }
                    continue;
                } else if value == 1 {
                    // Другая клавиша нажата пока держим триггер → отмена
                    single_other_key = true;
                }
                // Для single-trigger продолжаем обычную обработку buffer
            }

            // CapsLock — специальный режим: одно нажатие = триггер
            if is_caps_trigger && key == KeyCode::KEY_CAPSLOCK && value == 1 {
                if let Some(t) = last_double_at {
                    if t.elapsed() < debounce_window {
                        continue;
                    }
                }
                let buf_count = buffer.current_len() as u32;
                log(&format!(
                    "═ CROSS-CHECK: buffer.current={} events={}{}",
                    buf_count,
                    events_since_word_start,
                    if buf_count != events_since_word_start {
                        " ⚠ MISMATCH"
                    } else {
                        " ✓"
                    }
                ));
                let correction_result = run_configured_manual_correction(
                    &mut buffer,
                    &mut device,
                    &virtual_kbd,
                    &mut executing,
                );
                apply_manual_correction_result(
                    correction_result,
                    &mut current_layout_is_ru,
                    &mut last_layout_poll,
                    &mut suppress_next_typing_assist_after_manual_replay,
                );
                shift_state.clear_shifts();
                dshift_state = DShiftState::Idle;
                last_double_at = Some(Instant::now());
                clear_on_next_typing = true;
                log("· CAPS LOCK triggered");
                continue;
            }
            if key == trigger_key && !is_caps_trigger {
                // Дебаунс после DOUBLE
                if let Some(t) = last_double_at {
                    if t.elapsed() < debounce_window {
                        continue;
                    }
                }

                let now = Instant::now();
                if multi_tap_scope
                    && pending_multi_tap.is_some()
                    && value == 1
                    && pending_multi_tap.is_some_and(|pending| {
                        now.duration_since(pending.last_release) <= shift_window
                    })
                {
                    dshift_state = DShiftState::AdditionalPress { pressed_at: now };
                    if verbose {
                        log("· FSM: multi-tap waiting → AdditionalPress");
                    }
                    continue;
                }
                match (value, dshift_state) {
                    // ── press ──
                    (1, DShiftState::Idle) => {
                        dshift_state = DShiftState::FirstPress { pressed_at: now };
                        if verbose {
                            log("· FSM: Idle → FirstPress");
                        }
                    }
                    (1, DShiftState::WaitingSecond { first_release }) => {
                        if now.duration_since(first_release) <= shift_window {
                            dshift_state = DShiftState::SecondPress { second_press: now };
                            if verbose {
                                log("· FSM: WaitingSecond → SecondPress");
                            }
                        } else {
                            // слишком долго ждали — начинаем сначала
                            dshift_state = DShiftState::FirstPress { pressed_at: now };
                            if verbose {
                                log("· FSM: timeout → FirstPress");
                            }
                        }
                    }
                    (1, _) => {
                        // повторный press без release — игнор (autorepeat Shift)
                    }

                    // ── release ──
                    (0, DShiftState::FirstPress { pressed_at }) => {
                        let held = now.duration_since(pressed_at);
                        if held <= shift_tap_max {
                            dshift_state = DShiftState::WaitingSecond { first_release: now };
                            if verbose {
                                log(&format!(
                                    "· FSM: FirstPress → WaitingSecond (held {}ms)",
                                    held.as_millis()
                                ));
                            }
                        } else {
                            // держали долго — заглавная буква, не двойной Shift
                            dshift_state = DShiftState::Idle;
                            if verbose {
                                log(&format!(
                                    "· FSM: FirstPress → Idle (held {}ms, заглавная)",
                                    held.as_millis()
                                ));
                            }
                        }
                    }
                    (0, DShiftState::SecondPress { second_press, .. }) => {
                        let held = now.duration_since(second_press);
                        if held <= shift_tap_max {
                            // DOUBLE SHIFT! press→release→press→release ✓
                            if multi_tap_scope {
                                pending_multi_tap = Some(MultiTapPending {
                                    tap_count: 2,
                                    last_release: now,
                                });
                                dshift_state = DShiftState::Idle;
                                if verbose {
                                    log("· FSM: DOUBLE captured, wait for optional 3rd tap");
                                }
                                continue;
                            }
                            let buf_count = buffer.current_len() as u32;
                            log(&format!(
                                "═ CROSS-CHECK: buffer.current={} events_since_word_start={}{}",
                                buf_count,
                                events_since_word_start,
                                if buf_count != events_since_word_start {
                                    " ⚠ MISMATCH"
                                } else {
                                    " ✓"
                                }
                            ));
                            let correction_result = run_configured_manual_correction(
                                &mut buffer,
                                &mut device,
                                &virtual_kbd,
                                &mut executing,
                            );
                            apply_manual_correction_result(
                                correction_result,
                                &mut current_layout_is_ru,
                                &mut last_layout_poll,
                                &mut suppress_next_typing_assist_after_manual_replay,
                            );
                            shift_state.clear_shifts();
                            dshift_state = DShiftState::Idle;
                            last_double_at = Some(Instant::now());
                            clear_on_next_typing = true;
                            log("· FSM: DOUBLE! (p→r→p→r)");
                        } else {
                            // второй Shift держали долго — не двойной
                            dshift_state = DShiftState::Idle;
                            if verbose {
                                log(&format!(
                                    "· FSM: SecondPress → Idle (held {}ms, не тап)",
                                    held.as_millis()
                                ));
                            }
                        }
                    }
                    (0, DShiftState::AdditionalPress { pressed_at }) => {
                        let held = now.duration_since(pressed_at);
                        if held <= shift_tap_max {
                            if let Some(mut pending) = pending_multi_tap.take() {
                                pending.tap_count = pending.tap_count.saturating_add(1);
                                if pending.tap_count >= multi_tap_max_taps {
                                    let replace_words =
                                        multi_tap_scope_for_taps(pending.tap_count).unwrap_or(3);
                                    let correction_result = run_scoped_manual_correction(
                                        &mut buffer,
                                        replace_words,
                                        &mut device,
                                        &virtual_kbd,
                                        &mut executing,
                                        events_since_word_start,
                                        "multi-tap max",
                                    );
                                    apply_manual_correction_result(
                                        correction_result,
                                        &mut current_layout_is_ru,
                                        &mut last_layout_poll,
                                        &mut suppress_next_typing_assist_after_manual_replay,
                                    );
                                    shift_state.clear_shifts();
                                    dshift_state = DShiftState::Idle;
                                    last_double_at = Some(Instant::now());
                                    clear_on_next_typing = true;
                                } else {
                                    pending.last_release = now;
                                    pending_multi_tap = Some(pending);
                                    dshift_state = DShiftState::Idle;
                                    if verbose {
                                        log("· FSM: multi-tap captured, wait for next tap");
                                    }
                                }
                            } else {
                                dshift_state = DShiftState::Idle;
                            }
                        } else {
                            pending_multi_tap = None;
                            dshift_state = DShiftState::Idle;
                            if verbose {
                                log(&format!(
                                    "· FSM: AdditionalPress → Idle (held {}ms, не тап)",
                                    held.as_millis()
                                ));
                            }
                        }
                    }
                    (0, _) => {
                        // release в Idle или WaitingSecond — сброс
                        dshift_state = DShiftState::Idle;
                    }
                    _ => {}
                }
                continue;
            }

            // Любая ДРУГАЯ клавиша (press) сбрасывает FSM,
            // НО только до SecondPress — если второй Shift уже нажат,
            // ждём только его release (другие клавиши не мешают).
            if !matches!(
                dshift_state,
                DShiftState::Idle | DShiftState::SecondPress { .. }
            ) && value == 1
            {
                if verbose {
                    log(&format!("· FSM: cancel → Idle (key {code})"));
                }
                dshift_state = DShiftState::Idle;
            }
            if pending_multi_tap.is_some() && value == 1 {
                pending_multi_tap = None;
                if verbose {
                    log(&format!("· FSM: multi-tap cancel (key {code})"));
                }
            }

            if try_handle_space_release(
                key,
                value,
                SpaceReleaseContext {
                    events: &events,
                    event_idx,
                    buffer: &mut buffer,
                    device: &mut device,
                    virtual_kbd: &virtual_kbd,
                    executing: &mut executing,
                    pending_typing_assist_after_space: &mut pending_typing_assist_after_space,
                    shift_state: &shift_state,
                    verbose,
                },
            ) {
                continue;
            }

            // release не интересен — пропускаем
            if value == 0 {
                continue;
            }

            if shift_state.shortcut_active()
                && should_ignore_buffer_key(key, &shift_state, buffer.current_is_empty())
            {
                if verbose {
                    log(&format!("· key {code} ignored for buffer (shortcut/noise)"));
                }
                continue;
            }

            if ignore_current_token_until_space {
                if key == KeyCode::KEY_SPACE {
                    ignore_current_token_until_space = false;
                    events_since_word_start = 0;
                    pending_typing_assist_after_space = false;
                    continue;
                }
                if is_hard_boundary(key) {
                    ignore_current_token_until_space = false;
                } else if is_typing_key(key) {
                    if verbose {
                        log(&format!("· key {code} ignored inside non-word token"));
                    }
                    continue;
                }
            }

            if should_start_ignored_buffer_token(key, &shift_state, buffer.current_is_empty()) {
                ignore_current_token_until_space = true;
                if verbose {
                    log(&format!("· key {code} starts ignored non-word token"));
                }
                continue;
            }

            if should_ignore_buffer_key(key, &shift_state, buffer.current_is_empty()) {
                if verbose {
                    log(&format!("· key {code} ignored for buffer (shortcut/noise)"));
                }
                continue;
            }

            // ─── пробел: переносим current → prev (только на press) ──
            if key == KeyCode::KEY_SPACE {
                if value == 1 {
                    handle_space_press(SpacePressContext {
                        buffer: &mut buffer,
                        pending_typing_assist_after_space: &mut pending_typing_assist_after_space,
                        events_since_word_start: &mut events_since_word_start,
                        suppress_next_typing_assist_after_manual_replay:
                            &mut suppress_next_typing_assist_after_manual_replay,
                        verbose,
                    });
                }
                continue;
            }

            // ─── граница (Enter/Tab/Esc/стрелки/BS/Del) — сброс на press ──
            note_learning_backspace_if_needed(key, value, &mut buffer);
            if try_handle_enter_autocorrect(
                key,
                value,
                EnterAutocorrectContext {
                    buffer: &mut buffer,
                    device: &mut device,
                    virtual_kbd: &virtual_kbd,
                    executing: &mut executing,
                    current_layout_is_ru: &mut current_layout_is_ru,
                    last_layout_poll: &mut last_layout_poll,
                    pending_typing_assist_after_space: &mut pending_typing_assist_after_space,
                    ignore_current_token_until_space: &mut ignore_current_token_until_space,
                    events_since_word_start: &mut events_since_word_start,
                    clear_on_next_typing: &mut clear_on_next_typing,
                },
            ) {
                continue;
            }
            if handle_hard_boundary_if_needed(
                key,
                value,
                HardBoundaryContext {
                    buffer: &mut buffer,
                    virtual_kbd: &virtual_kbd,
                    executing: &mut executing,
                    pending_typing_assist_after_space: &mut pending_typing_assist_after_space,
                    ignore_current_token_until_space: &mut ignore_current_token_until_space,
                    events_since_word_start: &mut events_since_word_start,
                    shift_state: &shift_state,
                    verbose,
                },
            ) {
                continue;
            }

            // ─── обычный символ ─────
            if is_typing_key(key) {
                handle_typing_key_press(
                    code,
                    value,
                    TypingKeyContext {
                        buffer: &mut buffer,
                        shift_state: &shift_state,
                        current_layout_is_ru: &mut current_layout_is_ru,
                        last_layout_poll: &mut last_layout_poll,
                        events_since_word_start: &mut events_since_word_start,
                        clear_on_next_typing: &mut clear_on_next_typing,
                        ignore_current_token_until_space: &mut ignore_current_token_until_space,
                        suppress_next_typing_assist_after_manual_replay:
                            &mut suppress_next_typing_assist_after_manual_replay,
                        verbose,
                    },
                );
            }
        }
    }
}

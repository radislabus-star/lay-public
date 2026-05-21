use evdev::{uinput::VirtualDevice, Device, EventType, InputEvent, KeyCode};
use lay::config::LayConfig;
use lay::keyboard::{is_typing_key, KeyEvent};
use lay::word_buffer::WordBuffer;
use std::os::fd::AsRawFd;
use std::time::{Duration, Instant};

use super::{
    active_auto_replace, active_correction_engine, active_enter_autocorrect,
    active_enter_autocorrect_from_env, active_layout_backend, active_replace_words,
    active_typing_assist, append_user_correction_learning_log, grab_physical_device_for_correction,
    handle_double_shift, handle_enter_autocorrect, handle_force_layout_hotkey,
    handle_typing_assist_after_space, idle_wait_timeout, is_hard_boundary, log,
    multi_tap_scope_for_taps, read_current_layout_is_ru, run_manual_correction_with_scope,
    should_drop_stale_typing_assist_after_space, should_ignore_buffer_key,
    should_run_typing_assist_on_space_release, should_schedule_typing_assist_after_space,
    should_start_ignored_buffer_token, single_hotkey_keycode, update_focus_ignore_state,
    wait_for_keyboard_event_or_timeout, DShiftState, MultiTapPending, ShiftState,
    TypingAssistOutcome, ENTER_AUTOCORRECT_EXPERIMENT_ENV, FOCUS_IGNORE_POLL_INTERVAL_MS,
    LAYOUT_POLL_INTERVAL_MS,
};

pub(super) fn listen_keyboard(
    device_path: std::path::PathBuf,
    virtual_kbd: std::sync::Arc<std::sync::Mutex<Option<VirtualDevice>>>,
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

    // Клавиша-триггер из config
    let trigger_key = match cfg.trigger.as_str() {
        "double-ctrl" => KeyCode::KEY_LEFTCTRL,
        "double-alt" => KeyCode::KEY_LEFTALT,
        "caps-lock" => KeyCode::KEY_CAPSLOCK,
        "single-rshift" => KeyCode::KEY_RIGHTSHIFT,
        "single-rctrl" => KeyCode::KEY_RIGHTCTRL,
        "single-ralt" => KeyCode::KEY_RIGHTALT,
        "single-pause" => KeyCode::KEY_PAUSE,
        _ => KeyCode::KEY_LEFTSHIFT, // default: double-lshift
    };
    let is_caps_trigger = cfg.trigger == "caps-lock";
    // Одиночный триггер: нажал и отпустил без других клавиш — конвертация
    let is_single_trigger = cfg.trigger.starts_with("single-");
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
                        let _physical_grab = grab_physical_device_for_correction(&mut device);
                        let mut g = virtual_kbd.lock().unwrap();
                        let correction_result = run_manual_correction_with_scope(
                            &mut buffer,
                            replace_words,
                            g.as_mut(),
                            &mut executing,
                            events_since_word_start,
                            "multi-tap timeout",
                        );
                        if let Some(is_ru) = correction_result {
                            current_layout_is_ru = is_ru;
                            last_layout_poll = Instant::now();
                        }
                        if correction_result.is_some() {
                            suppress_next_typing_assist_after_manual_replay = true;
                        }
                        drop(g);
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
                                    let mut g = virtual_kbd.lock().unwrap();
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
                                    let mut g = virtual_kbd.lock().unwrap();
                                    let _physical_grab =
                                        grab_physical_device_for_correction(&mut device);
                                    let replace_words = active_replace_words();
                                    let engine = active_correction_engine();
                                    let auto_replace = active_auto_replace();
                                    let correction_result = handle_double_shift(
                                        &mut buffer,
                                        replace_words,
                                        engine,
                                        auto_replace,
                                        g.as_mut(),
                                        &mut executing,
                                    );
                                    if let Some(is_ru) = correction_result {
                                        current_layout_is_ru = is_ru;
                                        last_layout_poll = Instant::now();
                                    }
                                    if correction_result.is_some() {
                                        suppress_next_typing_assist_after_manual_replay = true;
                                    }
                                    drop(g);
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
                let _physical_grab = grab_physical_device_for_correction(&mut device);
                let mut g = virtual_kbd.lock().unwrap();
                let replace_words = active_replace_words();
                let engine = active_correction_engine();
                let auto_replace = active_auto_replace();
                let correction_result = handle_double_shift(
                    &mut buffer,
                    replace_words,
                    engine,
                    auto_replace,
                    g.as_mut(),
                    &mut executing,
                );
                if let Some(is_ru) = correction_result {
                    current_layout_is_ru = is_ru;
                    last_layout_poll = Instant::now();
                }
                if correction_result.is_some() {
                    suppress_next_typing_assist_after_manual_replay = true;
                }
                drop(g);
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
                            let _physical_grab = grab_physical_device_for_correction(&mut device);
                            let mut g = virtual_kbd.lock().unwrap();
                            let replace_words = active_replace_words();
                            let engine = active_correction_engine();
                            let auto_replace = active_auto_replace();
                            let correction_result = handle_double_shift(
                                &mut buffer,
                                replace_words,
                                engine,
                                auto_replace,
                                g.as_mut(),
                                &mut executing,
                            );
                            if let Some(is_ru) = correction_result {
                                current_layout_is_ru = is_ru;
                                last_layout_poll = Instant::now();
                            }
                            if correction_result.is_some() {
                                suppress_next_typing_assist_after_manual_replay = true;
                            }
                            drop(g);
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
                                    let _physical_grab =
                                        grab_physical_device_for_correction(&mut device);
                                    let mut g = virtual_kbd.lock().unwrap();
                                    let correction_result = run_manual_correction_with_scope(
                                        &mut buffer,
                                        replace_words,
                                        g.as_mut(),
                                        &mut executing,
                                        events_since_word_start,
                                        "multi-tap max",
                                    );
                                    if let Some(is_ru) = correction_result {
                                        current_layout_is_ru = is_ru;
                                        last_layout_poll = Instant::now();
                                    }
                                    if correction_result.is_some() {
                                        suppress_next_typing_assist_after_manual_replay = true;
                                    }
                                    drop(g);
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

            if key == KeyCode::KEY_SPACE
                && value == 0
                && should_run_typing_assist_on_space_release(
                    pending_typing_assist_after_space,
                    active_typing_assist(),
                    shift_state.any(),
                    buffer.is_empty(),
                )
            {
                if has_later_typing_press(&events, event_idx) {
                    if verbose {
                        log("· typing-assist deferred: next key already queued");
                    }
                    continue;
                }
                let mut g = virtual_kbd.lock().unwrap();
                let outcome = handle_typing_assist_after_space(
                    &mut buffer,
                    g.as_mut(),
                    Some(&mut device),
                    &mut executing,
                    0,
                );
                pending_typing_assist_after_space =
                    matches!(outcome, TypingAssistOutcome::Deferred);
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
                    if should_drop_stale_typing_assist_after_space(
                        pending_typing_assist_after_space,
                        buffer.current_len(),
                    ) {
                        pending_typing_assist_after_space = false;
                        if verbose {
                            log("· typing-assist stale previous word skipped behind current word");
                        }
                    }
                    if let Some(correction) = buffer.take_user_learning_correction(true) {
                        append_user_correction_learning_log(&correction);
                    }
                    buffer.handle_space();
                    events_since_word_start = 0;
                    if should_schedule_typing_assist_after_space(
                        active_typing_assist(),
                        &mut suppress_next_typing_assist_after_manual_replay,
                    ) {
                        pending_typing_assist_after_space = true;
                        if verbose {
                            log("· typing-assist scheduled after space");
                        }
                    }
                    if verbose {
                        log(&format!(
                            "· space, history={:?}, current={:?}",
                            buffer.prev_words_len(),
                            buffer.current_len()
                        ));
                    }
                }
                continue;
            }

            // ─── граница (Enter/Tab/Esc/стрелки/BS/Del) — сброс на press ──
            if matches!(key, KeyCode::KEY_BACKSPACE | KeyCode::KEY_DELETE) && value == 1 {
                buffer.note_learning_backspace();
            }
            if key == KeyCode::KEY_ENTER
                && value == 1
                && active_enter_autocorrect()
                && !buffer.is_empty()
            {
                let _physical_grab = grab_physical_device_for_correction(&mut device);
                let mut g = virtual_kbd.lock().unwrap();
                let correction_result = handle_enter_autocorrect(
                    &mut buffer,
                    active_replace_words(),
                    g.as_mut(),
                    &mut executing,
                );
                if let Some(is_ru) = correction_result {
                    current_layout_is_ru = is_ru;
                    last_layout_poll = Instant::now();
                    buffer.reset_all();
                    pending_typing_assist_after_space = false;
                    ignore_current_token_until_space = false;
                    events_since_word_start = 0;
                    clear_on_next_typing = true;
                    log("· Enter autocorrect consumed boundary");
                    continue;
                }
            }
            if is_hard_boundary(key) {
                if value == 1 && !buffer.is_empty() {
                    if pending_typing_assist_after_space
                        && active_typing_assist()
                        && !shift_state.any()
                    {
                        let cursor_offset = buffer.current_len() as u32;
                        let mut g = virtual_kbd.lock().unwrap();
                        let _ = handle_typing_assist_after_space(
                            &mut buffer,
                            g.as_mut(),
                            None,
                            &mut executing,
                            cursor_offset,
                        );
                    }
                    if !matches!(key, KeyCode::KEY_BACKSPACE | KeyCode::KEY_DELETE) {
                        if let Some(correction) = buffer.take_user_learning_correction(false) {
                            append_user_correction_learning_log(&correction);
                        }
                    }
                    buffer.reset_all();
                    pending_typing_assist_after_space = false;
                    ignore_current_token_until_space = false;
                    events_since_word_start = 0;
                    if verbose {
                        log(&format!("· reset (граница: {key:?})"));
                    }
                }
                continue;
            }

            // ─── обычный символ ─────
            if is_typing_key(key) {
                if clear_on_next_typing {
                    buffer.reset_all();
                    events_since_word_start = 0;
                    clear_on_next_typing = false;
                    ignore_current_token_until_space = false;
                    suppress_next_typing_assist_after_manual_replay = false;
                }
                let starts_new_word = buffer.current_is_empty();
                // Перекрёстный счёт — увеличиваем НА КАЖДОЕ press/repeat
                // независимо от accept-фильтра.
                events_since_word_start += 1;
                // v=2 (autorepeat) — добавляем ТОЛЬКО если это repeat той же
                // клавиши что была последней. Иначе чужой repeat ломал бы счёт.
                let accept = if value == 2 {
                    buffer.current_last_keycode() == Some(code)
                } else {
                    true
                };
                if !accept {
                    if verbose {
                        log(&format!("· key {code} v=2 SKIP (autorepeat другой) events={events_since_word_start}"));
                    }
                    continue;
                }
                if starts_new_word
                    || last_layout_poll.elapsed() >= Duration::from_millis(LAYOUT_POLL_INTERVAL_MS)
                {
                    if let Ok(is_ru) = read_current_layout_is_ru() {
                        current_layout_is_ru = is_ru;
                    }
                    last_layout_poll = Instant::now();
                }
                let typed_event = KeyEvent {
                    keycode: code,
                    shift: shift_state.any(),
                    layout_is_ru: current_layout_is_ru,
                };
                buffer.push(typed_event);
                buffer.note_learning_typed(typed_event);
                if verbose {
                    log(&format!(
                        "· key {code} v={value} shift={} → current={} events={events_since_word_start}",
                        shift_state.any(),
                        buffer.current_len()
                    ));
                }
            }
        }
    }
}

fn has_later_typing_press(events: &[InputEvent], current_index: usize) -> bool {
    events.iter().skip(current_index + 1).any(|event| {
        event.event_type() == EventType::KEY
            && event.value() == 1
            && is_typing_key(KeyCode::new(event.code()))
    })
}

pub(super) fn find_all_keyboards() -> std::io::Result<Vec<std::path::PathBuf>> {
    let mut found = Vec::new();
    for entry in std::fs::read_dir("/dev/input")? {
        let entry = entry?;
        let path = entry.path();
        if !path
            .file_name()
            .and_then(|n| n.to_str())
            .is_some_and(|s| s.starts_with("event"))
        {
            continue;
        }
        if let Ok(dev) = Device::open(&path) {
            if let Some(keys) = dev.supported_keys() {
                if keys.contains(KeyCode::KEY_LEFTSHIFT) && keys.contains(KeyCode::KEY_A) {
                    // НЕ слушаем наши/служебные uinput-устройства: это не железная
                    // клавиатура, а источник фантомных повторов в VM/desktop-тестах.
                    let name = dev.name().unwrap_or("").to_string();
                    if should_ignore_keyboard_device_name(&name) {
                        continue;
                    }
                    found.push(path);
                }
            }
        }
    }
    if found.is_empty() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "клавиатура не найдена. Возможно нет группы input — проверь `id`",
        ));
    }
    Ok(found)
}

pub(super) fn should_ignore_keyboard_device_name(name: &str) -> bool {
    matches!(name, "lay-virtual-keyboard" | "ydotoold virtual device")
}

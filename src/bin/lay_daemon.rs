//! lay-daemon — Caramba/Punto-style keyboard daemon for Linux desktops.
//!
//! Базовый replay-принцип: запоминаем физические нажатия клавиш и при двойном
//! Shift:
//!   1) стираем последнее слово через uinput Backspace × N,
//!   2) переключаем раскладку через выбранный desktop backend,
//!   3) повторяем те же физические клавиши через uinput — рабочее окружение
//!      интерпретирует их в новой раскладке.
//!
//! Этот replay core не требует словарной конвертации. Smart/typing-assist
//! ветки дополнительно используют RU/EN-таблицы, словари и n-gram scorer; они
//! сейчас оптимизированы и протестированы именно для RU/EN.

use clap::Parser;
use evdev::{uinput::VirtualDevice, AttributeSet, Device, EventType, InputEvent, KeyCode};
#[cfg(test)]
use lay::config::{
    default_typing_assist_pipeline, normalize_typing_assist_pipeline, DEFAULT_TYPING_ASSIST_RULES,
};
use lay::config::{
    typing_assist_pipeline_for_auto_replace, CorrectionEngine, LayConfig, TypingAssistRuleConfig,
};
#[cfg(test)]
use lay::correction::Correction;
use lay::decoder::{
    decode_enter_autocorrect_tail, decode_manual_tail, decode_typing_assist_tail, CorrectionSource,
    DecoderAction, DecoderEditPlan, ManualDecodeRequest,
};
#[cfg(test)]
use lay::desktop::resolve_layout_backend;
use lay::desktop::{is_ru_layout_id, normalize_layout_id, parse_setxkbmap_layout, LayoutBackend};
#[cfg(test)]
use lay::keyboard::map_opposite_events;
use lay::keyboard::{
    is_cyrillic_letter, is_typing_key, keycode_to_ru_char, keycode_to_us_char, map_original_events,
    preferred_layout_for_text, replay_layout_decision, text_to_uinput_runs, KeyEvent,
};
#[cfg(test)]
use lay::keyboard::{
    is_layout_decision_key, map_events_to_layout, split_event_words, ReplayLayoutDecision,
};
use lay::text_backend::{ImeReplaceRequest, TextBackendPreference};
#[cfg(test)]
use lay::text_edit::plan_text_replacement;
use lay::text_edit::{plan_committed_tail_replacement, TextReplacement};
#[cfg(test)]
use lay::typing_assist::{
    apply_typing_assist_with_pipeline, are_ru_keyboard_neighbors,
    correct_duplicate_layout_prefix_on_ascii_token, correct_extra_letters, correct_missing_letter,
    correct_wrong_layout_ascii_technical_token, decide_completed_scope_word, decide_correction,
    decide_scoped_tail_correction, decide_scoped_tail_correction_with_lem,
    is_ascii_technical_token, promoted_replacement_for_token,
    repair_cyrillic_prefix_before_ascii_tail, russian_generated_form_dictionary,
    scoped_tail_lem_candidates, should_keep_plain_cyrillic_before_ascii_technical,
    split_edge_whitespace, split_ws_segments,
};
use lay::typing_assist::{
    effective_replace_words, is_cyrillic_word, is_known_russian_word_or_form,
    remember_promoted_replacement, should_force_replay_for_short_fragment, ScopedTailOptions,
    REPLACEMENTS_PATH,
};
use lay::word_buffer::{PendingAutoUndo, UserLearningCorrection, WordBuffer};
use std::collections::BTreeMap;
#[cfg(test)]
use std::collections::HashSet;
use std::io::Write;
use std::os::fd::{AsRawFd, RawFd};
use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

const LEARN_LOG_PATH: &str = ".local/share/lay/corrections.jsonl"; // относительно $HOME
const LEARN_CANDIDATES_PATH: &str = ".local/share/lay/learning_candidates.json"; // относительно $HOME
const LEARN_LOG_MAX_BYTES: u64 = 1024 * 1024;
const LEARN_LOG_KEEP_LINES: usize = 3000;
const LEARN_PROMOTION_THRESHOLD: u32 = 2;
const KEY_PACE_MS: u64 = 1;
const BACKSPACE_DOWN_MS: u64 = 1;
const BACKSPACE_PACE_MS: u64 = 2;
const BACKSPACE_SETTLE_MS: u64 = 16;
const TEXT_REPLACE_KEY_PACE_MS: u64 = 1;
const TEXT_REPLACE_BACKSPACE_DOWN_MS: u64 = 1;
const TEXT_REPLACE_BACKSPACE_PACE_MS: u64 = 2;
const TEXT_REPLACE_BACKSPACE_SETTLE_MS: u64 = 16;
const TYPING_ASSIST_IDLE_DELAY_MS: u64 = 55;
const TYPING_ASSIST_SPACE_COMMIT_SETTLE_MS: u64 = 8;
const TEXT_INSERT_KEY_PACE_MS: u64 = 1;
const TEXT_INSERT_SPACE_SETTLE_MS: u64 = 8;
const LAYOUT_SWITCH_SETTLE_MS: u64 = 12;
const MODIFIER_RELEASE_ROUNDS: usize = 2;
const MODIFIER_RELEASE_PACE_MS: u64 = 3;
const MODIFIER_RELEASE_SETTLE_MS: u64 = 4;
const TRIGGER_RELEASE_SETTLE_MS: u64 = 80;
const GNOME_NATIVE_REPLACE_EXPERIMENTAL: bool = false;
const LAYOUT_POLL_INTERVAL_MS: u64 = 250;
const FOCUS_IGNORE_POLL_INTERVAL_MS: u64 = 500;
const IDLE_EVENT_WAIT_MAX_MS: u64 = 500;
const ENTER_AUTOCORRECT_EXPERIMENT_ENV: &str = "LAY_EXPERIMENTAL_ENTER_AUTOCORRECT";
const HOST_FOCUS_IGNORE_HINTS: &[&str] = &[
    "org.virt-manager.virt-manager",
    "virt-manager",
    "remote-viewer",
    "virt-viewer",
    "spicy",
    "org.gnome.boxes",
    "gnome-boxes",
    "virtualbox machine",
    "virtualboxvm",
    "qemu-system",
    "lay-kde-test spice",
    "lay-kde-spice-viewer-clipboard",
    "spice clipboard",
];
static DBUS_CONNECTION: OnceLock<Mutex<Option<zbus::blocking::Connection>>> = OnceLock::new();
static AUTO_LAYOUT_BACKEND_HINT: OnceLock<Option<LayoutBackend>> = OnceLock::new();
static TYPING_ASSIST_RUNTIME_READY: AtomicBool = AtomicBool::new(false);
static FOCUS_INFO_UNAVAILABLE: AtomicBool = AtomicBool::new(false);

// ─── Config ─────────────────────────────────────────────────

fn active_replace_words() -> usize {
    LayConfig::load().active_replace_words()
}

fn active_correction_engine() -> CorrectionEngine {
    LayConfig::load().active_correction_engine()
}

fn active_layout_backend() -> LayoutBackend {
    let config = LayConfig::load();
    let backend = config.active_layout_backend();
    let configured = config.layout_backend.trim().to_ascii_lowercase();
    if configured != "auto" || backend != LayoutBackend::Gnome {
        return backend;
    }

    if let Some(hint) = *AUTO_LAYOUT_BACKEND_HINT.get_or_init(detect_auto_layout_backend_hint) {
        return hint;
    }
    backend
}

fn active_text_backend() -> TextBackendPreference {
    LayConfig::load().active_text_backend()
}

fn active_auto_replace() -> bool {
    LayConfig::load().auto_replace
}

fn active_typing_assist() -> bool {
    LayConfig::load().typing_assist
}

fn active_enter_autocorrect() -> bool {
    let cfg = LayConfig::load();
    active_enter_autocorrect_from_env(
        cfg.enter_autocorrect,
        std::env::var(ENTER_AUTOCORRECT_EXPERIMENT_ENV)
            .ok()
            .as_deref(),
    )
}

fn active_enter_autocorrect_from_env(config_enabled: bool, env_value: Option<&str>) -> bool {
    config_enabled
        && env_value
            .map(|value| {
                matches!(
                    value.trim().to_ascii_lowercase().as_str(),
                    "1" | "true" | "yes" | "on"
                )
            })
            .unwrap_or(false)
}

fn active_auto_switch_layout() -> bool {
    LayConfig::load().auto_switch_layout
}

fn active_learning_log() -> bool {
    LayConfig::load().learning_log
}

fn active_lem_enabled_for_scope(word_count: usize) -> bool {
    LayConfig::load().lem_enabled_for_scope(word_count)
}

#[cfg(not(test))]
fn active_typing_assist_pipeline_for_auto_replace() -> Vec<TypingAssistRuleConfig> {
    let cfg = LayConfig::load();
    typing_assist_pipeline_for_auto_replace(cfg.auto_replace, &cfg.typing_assist_pipeline)
}

const DBUS_PATH: &str = "/io/github/radislabus_star/LayDaemon";
const DBUS_INTERFACE: &str = "io.github.radislabus_star.LayDaemon";
const DBUS_DEST: &str = "org.gnome.Shell";
const IME_DBUS_DEST: &str = "io.github.radislabus_star.LayIme";
const IME_DBUS_PATH: &str = "/io/github/radislabus_star/LayIme";
const IME_DBUS_INTERFACE: &str = "io.github.radislabus_star.LayIme";

#[derive(Parser, Debug)]
#[command(
    name = "lay-daemon",
    version,
    about = "Caramba-style daemon for Linux Wayland"
)]
struct Args {
    /// Не вызывать DBus extension и не эмулировать — только лог.
    #[arg(long)]
    detect_only: bool,
    /// Принудительно использовать конкретное устройство клавиатуры.
    #[arg(long)]
    device: Option<String>,
    /// Verbose: лог каждого нажатия в stderr/journal. Может содержать набранный текст.
    #[arg(short, long)]
    verbose: bool,
    /// Писать диагностический вывод в stderr/journal. Может содержать набранный текст.
    #[arg(long)]
    debug_log: bool,
}

struct ExecutingGuard<'a>(&'a mut bool);

impl Drop for ExecutingGuard<'_> {
    fn drop(&mut self) {
        *self.0 = false;
    }
}

struct DeviceGrabGuard<'a> {
    device: &'a mut Device,
    active: bool,
}

impl Drop for DeviceGrabGuard<'_> {
    fn drop(&mut self) {
        if self.active {
            if let Err(e) = self.device.ungrab() {
                log(&format!("⚠ physical device ungrab failed: {e}"));
            }
        }
    }
}

fn grab_physical_device_for_correction(device: &mut Device) -> DeviceGrabGuard<'_> {
    match device.grab() {
        Ok(()) => DeviceGrabGuard {
            device,
            active: true,
        },
        Err(e) => {
            log(&format!(
                "⚠ physical device grab failed: {e}; continuing without input isolation"
            ));
            DeviceGrabGuard {
                device,
                active: false,
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TextInsertMethod {
    UinputReplay,
    TypeTextFallback,
}

fn main() -> std::io::Result<()> {
    let args = Args::parse();
    set_log_enabled(args.debug_log || args.verbose || args.detect_only);

    let device_paths: Vec<std::path::PathBuf> = match args.device.clone() {
        Some(p) => vec![std::path::PathBuf::from(p)],
        None => find_all_keyboards()?,
    };
    log(&format!("► старт, устройства: {device_paths:?}"));
    log(&format!(
        "► режим: {}",
        if args.detect_only {
            "DETECT-ONLY"
        } else {
            "LIVE (DBus + uinput)"
        }
    ));
    let startup_cfg = LayConfig::load();
    let startup_backend = active_layout_backend();
    log(&format!(
        "► layout backend: {} (config={})",
        startup_backend.label(),
        startup_cfg.layout_backend
    ));
    log(&format!(
        "► text backend: {}",
        startup_cfg.active_text_backend().as_str()
    ));
    let warm_smart = startup_cfg.active_correction_engine() == CorrectionEngine::Smart;
    let enter_autocorrect_active = active_enter_autocorrect_from_env(
        startup_cfg.enter_autocorrect,
        std::env::var(ENTER_AUTOCORRECT_EXPERIMENT_ENV)
            .ok()
            .as_deref(),
    );
    let warm_typing_assist = startup_cfg.typing_assist || enter_autocorrect_active;
    if !args.detect_only && (warm_smart || warm_typing_assist) {
        std::thread::spawn(move || {
            let started_at = Instant::now();
            lay::ngram::warm_up();
            lay::lem::warm_up();
            lay::typing_assist::warm_up();
            TYPING_ASSIST_RUNTIME_READY.store(true, Ordering::Relaxed);
            if warm_smart {
                match lay::llm::warm_up() {
                    Ok(()) => log("► smart engine: модель прогрета заранее"),
                    Err(e) => log(&format!("⚠ smart engine warmup failed: {e}")),
                }
            }
            log(&format!(
                "► dictionaries/ngram/LEM warmed in {}ms",
                started_at.elapsed().as_millis()
            ));
        });
    } else {
        TYPING_ASSIST_RUNTIME_READY.store(true, Ordering::Relaxed);
    }

    // GNOME backend uses the Shell extension for layout activation and TypeText fallback.
    if !args.detect_only && startup_backend == LayoutBackend::Gnome {
        match call_ping() {
            Ok(reply) => {
                log(&format!("► extension: {reply}"));
            }
            Err(e) => {
                log(&format!("⚠ extension не отвечает ({e})"));
                log("⚠ работаю в detect-only");
            }
        }
    } else if !args.detect_only && startup_backend == LayoutBackend::X11 {
        match lay::x11_layout::ping() {
            Ok(reply) => log(&format!("► native X11 backend: {reply}")),
            Err(e) => log(&format!(
                "⚠ native X11 backend unavailable ({e}); shell fallback remains enabled"
            )),
        }
    } else if !args.detect_only {
        log("► GNOME extension ping skipped for non-GNOME layout backend");
    }
    if !args.detect_only && startup_cfg.active_text_backend().should_try_ime() {
        match call_ime_ping() {
            Ok(reply) => log(&format!("► IME bridge: {reply}")),
            Err(e) => log(&format!(
                "⚠ IME bridge unavailable ({e}); uinput fallback remains enabled"
            )),
        }
    }

    // Virtual keyboard через uinput для re-typing физических кнопок
    let virtual_kbd = if args.detect_only {
        None
    } else {
        match make_virtual_keyboard() {
            Ok(d) => {
                log("► uinput virtual keyboard создан");
                Some(d)
            }
            Err(e) => {
                log(&format!(
                    "⚠ uinput недоступен ({e}). Re-typing работать не будет"
                ));
                None
            }
        }
    };

    // Spawn один тред на каждую клавиатуру. Каждый тред держит свой
    // буфер и shift_state — клавиатуры независимы, что корректно
    // (если у пользователя 2 клавиатуры — он печатает на одной).
    use std::sync::{Arc, Mutex};
    let virtual_kbd = Arc::new(Mutex::new(virtual_kbd));

    let mut handles = Vec::new();
    for path in device_paths {
        let virtual_kbd = Arc::clone(&virtual_kbd);
        let v = args.verbose;
        let cfg = LayConfig::load();
        handles.push(std::thread::spawn(move || {
            if let Err(e) = listen_keyboard(path, virtual_kbd, v, cfg) {
                log(&format!("⚠ thread keyboard: {e}"));
            }
        }));
    }
    for h in handles {
        let _ = h.join();
    }
    Ok(())
}

fn listen_keyboard(
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
    let mut pending_typing_assist_after_space: Option<Instant> = None;
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
                    &mut pending_typing_assist_after_space,
                    &mut events_since_word_start,
                );
                if focus_ignored {
                    wait_for_keyboard_event_or_timeout(
                        device_fd,
                        idle_wait_timeout(
                            pending_multi_tap.as_ref(),
                            pending_typing_assist_after_space,
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
                if pending_typing_assist_after_space.is_some_and(|scheduled_at| {
                    scheduled_at.elapsed() >= Duration::from_millis(TYPING_ASSIST_IDLE_DELAY_MS)
                }) {
                    pending_typing_assist_after_space = None;
                    if active_typing_assist() {
                        let mut g = virtual_kbd.lock().unwrap();
                        handle_typing_assist_after_space(&mut buffer, g.as_mut(), &mut executing);
                    }
                }
                wait_for_keyboard_event_or_timeout(
                    device_fd,
                    idle_wait_timeout(
                        pending_multi_tap.as_ref(),
                        pending_typing_assist_after_space,
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
            &mut pending_typing_assist_after_space,
            &mut events_since_word_start,
        );
        if focus_ignored {
            shift_state = ShiftState::default();
            dshift_state = DShiftState::Idle;
            pending_multi_tap = None;
            ignore_current_token_until_space = false;
            continue;
        }

        for event in events {
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
            if value == 1 && key != KeyCode::KEY_SPACE {
                pending_typing_assist_after_space = None;
            }

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

            // release не интересен — пропускаем
            if value == 0 {
                continue;
            }

            if ignore_current_token_until_space {
                if key == KeyCode::KEY_SPACE {
                    ignore_current_token_until_space = false;
                    events_since_word_start = 0;
                    pending_typing_assist_after_space = None;
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
                    if let Some(correction) = buffer.take_user_learning_correction(true) {
                        append_user_correction_learning_log(&correction);
                    }
                    buffer.handle_space();
                    events_since_word_start = 0;
                    if should_schedule_typing_assist_after_space(
                        active_typing_assist(),
                        &mut suppress_next_typing_assist_after_manual_replay,
                    ) {
                        pending_typing_assist_after_space = Some(Instant::now());
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
                    pending_typing_assist_after_space = None;
                    ignore_current_token_until_space = false;
                    events_since_word_start = 0;
                    clear_on_next_typing = true;
                    log("· Enter autocorrect consumed boundary");
                    continue;
                }
            }
            if is_hard_boundary(key) {
                if value == 1 && !buffer.is_empty() {
                    if !matches!(key, KeyCode::KEY_BACKSPACE | KeyCode::KEY_DELETE) {
                        if let Some(correction) = buffer.take_user_learning_correction(false) {
                            append_user_correction_learning_log(&correction);
                        }
                    }
                    buffer.reset_all();
                    pending_typing_assist_after_space = None;
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
                buffer.push(KeyEvent {
                    keycode: code,
                    shift: shift_state.any(),
                    layout_is_ru: current_layout_is_ru,
                });
                buffer.note_learning_typed(KeyEvent {
                    keycode: code,
                    shift: shift_state.any(),
                    layout_is_ru: current_layout_is_ru,
                });
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

fn update_focus_ignore_state(
    focus_ignored: &mut bool,
    last_poll: &mut Instant,
    buffer: &mut WordBuffer,
    pending_typing_assist_after_space: &mut Option<Instant>,
    events_since_word_start: &mut u32,
) {
    if active_layout_backend() != LayoutBackend::Gnome
        || last_poll.elapsed() < Duration::from_millis(FOCUS_IGNORE_POLL_INTERVAL_MS)
    {
        return;
    }
    *last_poll = Instant::now();

    let ignored = focused_window_should_be_ignored();
    if ignored != *focus_ignored {
        if ignored {
            log("► focused window ignored: VM/remote viewer, host lay paused");
        } else {
            log("► focused window accepted: host lay resumed");
        }
    }
    if ignored {
        buffer.reset_all();
        *pending_typing_assist_after_space = None;
        *events_since_word_start = 0;
    }
    *focus_ignored = ignored;
}

fn wait_for_keyboard_event_or_timeout(fd: RawFd, timeout: Duration) -> std::io::Result<()> {
    let mut poll_fd = libc::pollfd {
        fd,
        events: libc::POLLIN,
        revents: 0,
    };
    let timeout_ms = timeout
        .max(Duration::from_millis(1))
        .as_millis()
        .min(i32::MAX as u128) as i32;

    let rc = unsafe { libc::poll(&mut poll_fd, 1, timeout_ms) };
    if rc < 0 {
        let err = std::io::Error::last_os_error();
        if err.kind() == std::io::ErrorKind::Interrupted {
            return Ok(());
        }
        return Err(err);
    }
    Ok(())
}

fn idle_wait_timeout(
    pending_multi_tap: Option<&MultiTapPending>,
    pending_typing_assist_after_space: Option<Instant>,
    last_focus_ignore_poll: Instant,
    shift_window: Duration,
) -> Duration {
    idle_wait_timeout_at(
        Instant::now(),
        pending_multi_tap,
        pending_typing_assist_after_space,
        last_focus_ignore_poll,
        shift_window,
    )
}

fn idle_wait_timeout_at(
    now: Instant,
    pending_multi_tap: Option<&MultiTapPending>,
    pending_typing_assist_after_space: Option<Instant>,
    last_focus_ignore_poll: Instant,
    shift_window: Duration,
) -> Duration {
    let mut wait = Duration::from_millis(IDLE_EVENT_WAIT_MAX_MS);

    if let Some(pending) = pending_multi_tap {
        wait = wait.min(deadline_remaining(now, pending.last_release, shift_window));
    }
    if let Some(scheduled_at) = pending_typing_assist_after_space {
        wait = wait.min(deadline_remaining(
            now,
            scheduled_at,
            Duration::from_millis(TYPING_ASSIST_IDLE_DELAY_MS),
        ));
    }
    wait.min(deadline_remaining(
        now,
        last_focus_ignore_poll,
        Duration::from_millis(FOCUS_IGNORE_POLL_INTERVAL_MS),
    ))
}

fn deadline_remaining(now: Instant, started_at: Instant, delay: Duration) -> Duration {
    started_at
        .checked_add(delay)
        .and_then(|deadline| deadline.checked_duration_since(now))
        .unwrap_or(Duration::ZERO)
}

fn focused_window_should_be_ignored() -> bool {
    if FOCUS_INFO_UNAVAILABLE.load(Ordering::Relaxed) {
        return false;
    }
    match call_focused_window_info() {
        Ok(json) => focused_window_json_is_ignored(&json),
        Err(error) => {
            FOCUS_INFO_UNAVAILABLE.store(true, Ordering::Relaxed);
            log(&format!(
                "⚠ FocusedWindowInfo unavailable, host focus guard disabled: {error}"
            ));
            false
        }
    }
}

fn focused_window_json_is_ignored(json: &str) -> bool {
    let haystack = focused_window_haystack(json);
    HOST_FOCUS_IGNORE_HINTS
        .iter()
        .any(|hint| haystack.contains(&hint.to_ascii_lowercase()))
}

fn focused_window_haystack(json: &str) -> String {
    let value = match serde_json::from_str::<serde_json::Value>(json) {
        Ok(value) => value,
        Err(_) => return json.to_ascii_lowercase(),
    };
    let mut parts = Vec::new();
    if let Some(object) = value.as_object() {
        for key in [
            "appId",
            "wmClass",
            "wmClassInstance",
            "label",
            "title",
            "value",
        ] {
            if let Some(text) = object.get(key).and_then(|item| item.as_str()) {
                parts.push(text);
            }
        }
    }
    parts.join(" ").to_ascii_lowercase()
}

#[derive(Default)]
struct ShiftState {
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
    fn update(&mut self, key: KeyCode, value: i32) {
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

    fn clear_shifts(&mut self) {
        self.left = false;
        self.right = false;
    }

    fn any(&self) -> bool {
        self.left || self.right
    }

    fn shortcut_active(&self) -> bool {
        self.left_ctrl
            || self.right_ctrl
            || self.left_alt
            || self.right_alt
            || self.left_meta
            || self.right_meta
    }
}

fn single_hotkey_keycode(id: &str) -> Option<KeyCode> {
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
enum DShiftState {
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
struct MultiTapPending {
    tap_count: u8,
    last_release: Instant,
}

fn multi_tap_scope_for_taps(taps: u8) -> Option<usize> {
    match taps {
        0 | 1 => None,
        2 => Some(1),
        3 => Some(2),
        _ => Some(3),
    }
}

// ─── Word boundary детекция ─────────────────────────────────

fn is_hard_boundary(key: KeyCode) -> bool {
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

fn should_ignore_buffer_key(key: KeyCode, modifiers: &ShiftState, current_empty: bool) -> bool {
    if modifiers.shortcut_active() && (key == KeyCode::KEY_SPACE || is_typing_key(key)) {
        return true;
    }

    should_start_ignored_buffer_token(key, modifiers, current_empty)
}

fn should_start_ignored_buffer_token(
    key: KeyCode,
    modifiers: &ShiftState,
    current_empty: bool,
) -> bool {
    current_empty && is_leading_non_word_symbol_key(key, modifiers.any())
}

fn should_schedule_typing_assist_after_space(active: bool, suppress_once: &mut bool) -> bool {
    if !active {
        return false;
    }
    if *suppress_once {
        *suppress_once = false;
        return false;
    }
    true
}

fn is_leading_non_word_symbol_key(key: KeyCode, _shift: bool) -> bool {
    matches!(key, KeyCode::KEY_EQUAL | KeyCode::KEY_MINUS)
}

// ─── Двойной Shift handler ──────────────────────────────────

fn handle_force_layout_hotkey(
    target_is_ru: bool,
    buf: &mut WordBuffer,
    virtual_kbd: Option<&mut VirtualDevice>,
    executing: &mut bool,
) -> Option<bool> {
    let started_at = Instant::now();
    settle_after_physical_trigger_release();
    *executing = true;
    let _executing_guard = ExecutingGuard(executing);

    if let Some(kbd) = virtual_kbd {
        if let Err(e) = release_possible_modifiers(kbd) {
            log(&format!("⚠ force-layout modifier cleanup failed: {e}"));
        }
    }

    match switch_to_target_layout(target_is_ru) {
        Ok(layout_id) => {
            buf.reset_all();
            log(&format!(
                "✓ force-layout → {layout_id} за {}ms",
                started_at.elapsed().as_millis()
            ));
            Some(target_is_ru)
        }
        Err(e) => {
            log(&format!("⚠ force-layout switch failed: {e}"));
            None
        }
    }
}

fn run_manual_correction_with_scope(
    buf: &mut WordBuffer,
    replace_words: usize,
    virtual_kbd: Option<&mut VirtualDevice>,
    executing: &mut bool,
    events_since_word_start: u32,
    label: &str,
) -> Option<bool> {
    let buf_count = buf.current_len() as u32;
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
    let engine = active_correction_engine();
    let auto_replace = active_auto_replace();
    let result = handle_double_shift(
        buf,
        replace_words,
        engine,
        auto_replace,
        virtual_kbd,
        executing,
    );
    log(&format!("· {label} fired with scope={replace_words}"));
    result
}

fn handle_typing_assist_after_space(
    buf: &mut WordBuffer,
    virtual_kbd: Option<&mut VirtualDevice>,
    executing: &mut bool,
) {
    if !TYPING_ASSIST_RUNTIME_READY.load(Ordering::Relaxed) {
        log("· typing-assist skipped: warmup pending");
        return;
    }

    let started_at = Instant::now();
    let allow_layout_auto = active_auto_switch_layout();
    #[cfg(test)]
    let pipeline = default_typing_assist_pipeline();
    #[cfg(not(test))]
    let pipeline = active_typing_assist_pipeline_for_auto_replace();
    let correction = [2, 1].into_iter().find_map(|word_count| {
        let events = buf.last_completed_words_events(word_count)?;
        let edit = decode_typing_assist_tail(
            &events,
            allow_layout_auto,
            &pipeline,
            CorrectionSource::TypingAssist,
        )?;
        Some((events, edit))
    });
    let Some((events, edit)) = correction else {
        return;
    };
    let original = edit.original.clone();
    let replacement = edit.replacement.clone();

    if should_try_ime_text_backend() {
        let original_layout = read_current_layout_is_ru().ok();
        if try_ime_replace_tail(&original, &replacement, "typing-assist").unwrap_or(false) {
            let target_layout = preferred_layout_for_text(&replacement, true);
            if active_auto_switch_layout() {
                match switch_to_target_layout(target_layout) {
                    Ok(layout_id) => log(&format!("  typing-assist layout → {layout_id}")),
                    Err(e) => log(&format!("⚠ typing-assist layout switch failed: {e}")),
                }
            } else if let Some(layout_is_ru) = original_layout {
                match switch_to_target_layout(layout_is_ru) {
                    Ok(layout_id) => log(&format!("  typing-assist layout restored → {layout_id}")),
                    Err(e) => log(&format!("⚠ typing-assist layout restore failed: {e}")),
                }
            }
            let words = original.split_whitespace().count();
            remember_assisted_text_correction(
                buf,
                AssistedCorrectionMemory {
                    events: &events,
                    plan: &TextReplacement {
                        move_left: 0,
                        backspaces: original.chars().count() as u32,
                        insert: replacement.clone(),
                        move_right: 0,
                    },
                    original: &original,
                    replacement: &replacement,
                    kind: "typing-assist",
                    replace_words: words,
                    words,
                },
            );
            log(&format!(
                "✓ done: помощь при наборе {:?} → {:?} через IME за {}ms",
                original,
                replacement,
                started_at.elapsed().as_millis()
            ));
            return;
        }
    }

    let Some(kbd) = virtual_kbd else {
        log("⚠ typing-assist: нет uinput device");
        return;
    };

    *executing = true;
    let _executing_guard = ExecutingGuard(executing);

    if let Err(e) = release_possible_modifiers(kbd) {
        log(&format!("⚠ typing-assist modifier cleanup failed: {e}"));
    }

    let original_layout = read_current_layout_is_ru().ok();
    if original
        .chars()
        .next_back()
        .is_some_and(char::is_whitespace)
    {
        std::thread::sleep(Duration::from_millis(TYPING_ASSIST_SPACE_COMMIT_SETTLE_MS));
    }
    let plan = edit.plan.clone();

    if let Err(e) = apply_text_replacement(kbd, &plan) {
        log(&format!("⚠ typing-assist minimal replace failed: {e}"));
        return;
    }

    let target_layout =
        match insert_text_for_replacement_plan(kbd, &plan, &replacement, true, "typing-assist") {
            Ok(layout) => layout,
            Err(e) => {
                log(&format!("⚠ typing-assist {e}"));
                return;
            }
        };
    if active_auto_switch_layout() {
        match switch_to_target_layout(target_layout) {
            Ok(layout_id) => log(&format!("  typing-assist layout → {layout_id}")),
            Err(e) => log(&format!("⚠ typing-assist layout switch failed: {e}")),
        }
    } else if let Some(layout_is_ru) = original_layout {
        match switch_to_target_layout(layout_is_ru) {
            Ok(layout_id) => log(&format!("  typing-assist layout restored → {layout_id}")),
            Err(e) => log(&format!("⚠ typing-assist layout restore failed: {e}")),
        }
    }

    let words = original.split_whitespace().count();
    remember_assisted_text_correction(
        buf,
        AssistedCorrectionMemory {
            events: &events,
            plan: &plan,
            original: &original,
            replacement: &replacement,
            kind: "typing-assist",
            replace_words: words,
            words,
        },
    );
    log(&format!(
        "✓ done: помощь при наборе {:?} → {:?} за {}ms",
        original,
        replacement,
        started_at.elapsed().as_millis()
    ));
}

fn enter_autocorrect_candidate(
    buf: &WordBuffer,
    replace_words: usize,
    allow_layout_auto: bool,
    pipeline: &[TypingAssistRuleConfig],
) -> Option<(Vec<KeyEvent>, DecoderEditPlan)> {
    let (events, _) = buf.what_to_replay(replace_words)?;
    let original = map_original_events(&events);
    if original.trim().is_empty() {
        return None;
    }

    let original_has_trailing_space = original
        .chars()
        .next_back()
        .is_some_and(char::is_whitespace);
    let edit = decode_enter_autocorrect_tail(
        &events,
        original_has_trailing_space,
        allow_layout_auto,
        pipeline,
    )?;
    Some((events, edit))
}

struct AssistedCorrectionMemory<'a> {
    events: &'a [KeyEvent],
    plan: &'a TextReplacement,
    original: &'a str,
    replacement: &'a str,
    kind: &'a str,
    replace_words: usize,
    words: usize,
}

fn remember_assisted_text_correction(
    buf: &mut WordBuffer,
    correction: AssistedCorrectionMemory<'_>,
) {
    buf.remember_pending_learning_correction(
        correction.kind,
        correction.original,
        correction.replacement,
        correction.replace_words,
        correction.words,
    );
    if !buf.remember_replacement_last_word_for_replay(
        correction.events,
        correction.plan,
        correction.replacement,
    ) {
        buf.reset_all();
    }
    buf.remember_pending_auto_undo(
        correction.kind,
        correction.original,
        correction.replacement,
        correction.replace_words,
        correction.words,
    );
}

fn handle_enter_autocorrect(
    buf: &mut WordBuffer,
    replace_words: usize,
    virtual_kbd: Option<&mut VirtualDevice>,
    executing: &mut bool,
) -> Option<bool> {
    if !TYPING_ASSIST_RUNTIME_READY.load(Ordering::Relaxed) {
        log("· enter-autocorrect skipped: warmup pending");
        return None;
    }

    let started_at = Instant::now();
    let allow_layout_auto = active_auto_switch_layout();
    #[cfg(test)]
    let pipeline = default_typing_assist_pipeline();
    #[cfg(not(test))]
    let pipeline = active_typing_assist_pipeline_for_auto_replace();
    let (events, edit) =
        enter_autocorrect_candidate(buf, replace_words, allow_layout_auto, &pipeline)?;
    let original = edit.original.clone();
    let replacement = edit.replacement.clone();

    if should_try_ime_text_backend() {
        let original_layout = read_current_layout_is_ru().ok();
        if try_ime_replace_tail(&original, &replacement, "enter-autocorrect").unwrap_or(false) {
            let target_layout = preferred_layout_for_text(&replacement, true);
            if let Some(kbd) = virtual_kbd {
                if let Err(e) = emit_key_taps_fast(kbd, KeyCode::KEY_ENTER, 1) {
                    log(&format!("⚠ enter-autocorrect Enter send failed: {e}"));
                }
            }
            if active_auto_switch_layout() {
                match switch_to_target_layout(target_layout) {
                    Ok(layout_id) => log(&format!("  enter-autocorrect layout → {layout_id}")),
                    Err(e) => log(&format!("⚠ enter-autocorrect layout switch failed: {e}")),
                }
            } else if let Some(layout_is_ru) = original_layout {
                match switch_to_target_layout(layout_is_ru) {
                    Ok(layout_id) => log(&format!(
                        "  enter-autocorrect layout restored → {layout_id}"
                    )),
                    Err(e) => log(&format!("⚠ enter-autocorrect layout restore failed: {e}")),
                }
            }
            log(&format!(
                "✓ done: Enter autocorrect {:?} → {:?} через IME за {}ms",
                original,
                replacement,
                started_at.elapsed().as_millis()
            ));
            return Some(target_layout);
        }
    }

    let Some(kbd) = virtual_kbd else {
        log("⚠ enter-autocorrect: нет uinput device");
        return None;
    };

    *executing = true;
    let _executing_guard = ExecutingGuard(executing);

    if let Err(e) = release_possible_modifiers(kbd) {
        log(&format!("⚠ enter-autocorrect modifier cleanup failed: {e}"));
    }

    let original_layout = read_current_layout_is_ru().ok();
    let plan = edit.plan.clone();

    if let Err(e) = apply_text_replacement(kbd, &plan) {
        log(&format!("⚠ enter-autocorrect minimal replace failed: {e}"));
        return None;
    }

    let target_layout =
        match insert_text_for_replacement_plan(kbd, &plan, &replacement, true, "enter-autocorrect")
        {
            Ok(layout) => layout,
            Err(e) => {
                log(&format!("⚠ enter-autocorrect {e}"));
                return None;
            }
        };
    if active_auto_switch_layout() {
        match switch_to_target_layout(target_layout) {
            Ok(layout_id) => log(&format!("  enter-autocorrect layout → {layout_id}")),
            Err(e) => log(&format!("⚠ enter-autocorrect layout switch failed: {e}")),
        }
    } else if let Some(layout_is_ru) = original_layout {
        match switch_to_target_layout(layout_is_ru) {
            Ok(layout_id) => log(&format!(
                "  enter-autocorrect layout restored → {layout_id}"
            )),
            Err(e) => log(&format!("⚠ enter-autocorrect layout restore failed: {e}")),
        }
    }

    if let Err(e) = emit_key_taps_fast(kbd, KeyCode::KEY_ENTER, 1) {
        log(&format!("⚠ enter-autocorrect Enter send failed: {e}"));
        return None;
    }

    remember_assisted_text_correction(
        buf,
        AssistedCorrectionMemory {
            events: &events,
            plan: &plan,
            original: &original,
            replacement: &replacement,
            kind: "enter-autocorrect",
            replace_words,
            words: original.split_whitespace().count(),
        },
    );
    log(&format!(
        "✓ done: Enter autocorrect {:?} → {:?} за {}ms",
        original,
        replacement,
        started_at.elapsed().as_millis()
    ));
    Some(target_layout)
}

fn handle_double_shift(
    buf: &mut WordBuffer,
    replace_words: usize,
    engine: CorrectionEngine,
    auto_replace: bool,
    virtual_kbd: Option<&mut VirtualDevice>,
    executing: &mut bool,
) -> Option<bool> {
    let started_at = Instant::now();
    if let Some(undo) = buf.take_pending_auto_undo() {
        return handle_pending_auto_undo(buf, undo, virtual_kbd, executing, started_at);
    }

    let replace_words = effective_replace_words(buf, replace_words, engine, auto_replace);
    let Some((events, n_backspaces)) = buf.what_to_replay(replace_words) else {
        log("👆 двойной Shift, но буфер пуст");
        return None;
    };
    *executing = true; // блокируем Shift events на время выполнения
    let _executing_guard = ExecutingGuard(executing);

    let layout_decision = replay_layout_decision(&events);
    let target_is_ru = layout_decision.target_is_ru;
    let mixed_layouts = layout_decision.mixed_layouts;

    // 3-й счёт: попытаться смаппить каждый keycode → char в ОБЕ раскладки.
    // Если char_count != events.len() — какой-то keycode вне таблиц
    // keycode_to_*_char (значит backspace×N сотрёт лишнее ИЛИ замаппится не всё).
    let mapped_orig: String = events
        .iter()
        .filter_map(|ev| {
            if ev.layout_is_ru {
                keycode_to_ru_char(ev.keycode, ev.shift)
            } else {
                keycode_to_us_char(ev.keycode, ev.shift)
            }
        })
        .collect();
    let mapped_target: String = events
        .iter()
        .filter_map(|ev| {
            if target_is_ru {
                keycode_to_ru_char(ev.keycode, ev.shift)
            } else {
                keycode_to_us_char(ev.keycode, ev.shift)
            }
        })
        .collect();
    let chars_orig = mapped_orig.chars().count();
    let chars_target = mapped_target.chars().count();
    let words_orig = mapped_orig.split_whitespace().count();
    let mismatch = chars_orig != events.len() || chars_target != events.len();
    log(&format!(
        "👆 events={} n_bs={n_backspaces} | chars_orig={chars_orig} chars_target={chars_target} words={words_orig} {} mixed={} | orig={mapped_orig:?} → target={mapped_target:?}",
        events.len(),
        if mismatch { "⚠ MAP-MISMATCH" } else { "✓" },
        mixed_layouts,
    ));

    if mapped_target.is_empty() {
        log("⚠ mapped_target пуст — не вставляем");
        return None;
    }
    // ═══ АЛГОРИТМ: decision layer → backspace → replay/text insert ═══

    let force_short_replay = should_force_replay_for_short_fragment(&mapped_orig);
    let force_replay_toggle =
        engine == CorrectionEngine::Smart && (buf.replay_toggle_ready() || force_short_replay);
    if force_replay_toggle {
        log("  smart: replay без модели");
    }
    let scoped_options = ScopedTailOptions {
        lem_enabled: active_lem_enabled_for_scope(words_orig),
        allow_layout_auto: active_auto_switch_layout(),
    };
    let correction_result = decode_manual_tail(ManualDecodeRequest {
        events: &events,
        original: &mapped_orig,
        converted: &mapped_target,
        engine,
        force_replay: force_replay_toggle,
        auto_replace,
        scoped_options,
    });
    let correction_action = correction_result.action.clone();
    let correction_edit = correction_result.edit.clone();

    if should_try_ime_text_backend() {
        let (replace_text, replace_kind, is_replay) = match &correction_action {
            DecoderAction::ReplaceText {
                replacement,
                source,
            } if !replacement.trim().is_empty() => (replacement.clone(), source.log_kind(), false),
            _ => (mapped_target.clone(), "ime-replay", true),
        };
        let replace_target_is_ru = preferred_layout_for_text(&replace_text, target_is_ru);
        if try_ime_replace_tail(&mapped_orig, &replace_text, replace_kind).unwrap_or(false) {
            if is_replay {
                buf.mark_replayed_layout(replace_words, replace_target_is_ru);
                if !force_replay_toggle && mapped_orig != replace_text {
                    append_learning_log(
                        "layout-replay",
                        &mapped_orig,
                        &replace_text,
                        replace_words,
                        words_orig,
                    );
                }
            } else {
                let plan = TextReplacement {
                    move_left: 0,
                    backspaces: mapped_orig.chars().count() as u32,
                    insert: replace_text.clone(),
                    move_right: 0,
                };
                buf.remember_pending_learning_correction(
                    replace_kind,
                    &mapped_orig,
                    &replace_text,
                    replace_words,
                    words_orig,
                );
                if !buf.remember_replacement_last_word_for_replay(&events, &plan, &replace_text) {
                    buf.reset_all();
                }
            }
            return match switch_to_target_layout(replace_target_is_ru) {
                Ok(layout_id) => {
                    log(&format!("  layout → {layout_id}"));
                    log(&format!(
                        "✓ done: {replace_kind}, IME replace-tail за {}ms",
                        started_at.elapsed().as_millis()
                    ));
                    Some(replace_target_is_ru)
                }
                Err(e) => {
                    log(&format!(
                        "⚠ {replace_kind} IME text committed, layout switch failed: {e}"
                    ));
                    None
                }
            };
        }
    }

    if GNOME_NATIVE_REPLACE_EXPERIMENTAL && active_layout_backend() == LayoutBackend::Gnome {
        let (replace_text, replace_kind, is_replay) = match &correction_action {
            DecoderAction::ReplaceText {
                replacement,
                source,
            } if !replacement.trim().is_empty() => (replacement.clone(), source.log_kind(), false),
            _ => (mapped_target.clone(), "gnome-replace", true),
        };
        let replace_target_is_ru = preferred_layout_for_text(&replace_text, target_is_ru);
        let (layout_id, _) = target_layout(replace_target_is_ru);
        match call_replace_text(0, n_backspaces, &replace_text, 0, layout_id) {
            Ok(true) => {
                if is_replay {
                    buf.mark_replayed_layout(replace_words, replace_target_is_ru);
                    if !force_replay_toggle && mapped_orig != replace_text {
                        append_learning_log(
                            "layout-replay",
                            &mapped_orig,
                            &replace_text,
                            replace_words,
                            words_orig,
                        );
                    }
                } else {
                    let plan = TextReplacement {
                        move_left: 0,
                        backspaces: n_backspaces,
                        insert: replace_text.clone(),
                        move_right: 0,
                    };
                    buf.remember_pending_learning_correction(
                        replace_kind,
                        &mapped_orig,
                        &replace_text,
                        replace_words,
                        words_orig,
                    );
                    if !buf.remember_replacement_last_word_for_replay(&events, &plan, &replace_text)
                    {
                        buf.reset_all();
                    }
                }
                log(&format!(
                    "  1. GNOME ReplaceText: bs={} insert={:?}",
                    n_backspaces, replace_text
                ));
                log(&format!("  2. layout → {layout_id}"));
                log(&format!(
                    "✓ done: {replace_kind}, GNOME-native replace за {}ms",
                    started_at.elapsed().as_millis()
                ));
                return Some(replace_target_is_ru);
            }
            Ok(false) => log("⚠ GNOME ReplaceText returned false; fallback to uinput replay"),
            Err(e) => log(&format!(
                "⚠ GNOME ReplaceText failed: {e}; fallback to uinput replay"
            )),
        }
    }

    let kbd = match virtual_kbd {
        Some(k) => k,
        None => {
            log("⚠ нет uinput device");
            return None;
        }
    };
    settle_after_physical_trigger_release();
    if let Err(e) = release_possible_modifiers(kbd) {
        log(&format!("⚠ modifier cleanup before backspace failed: {e}"));
    }

    if let DecoderAction::ReplaceText {
        replacement: text,
        source,
    } = correction_action
    {
        let kind = source.log_kind();
        if text.trim().is_empty() || text == mapped_target {
            log("  2. text decision совпал с replay — replay для сохранения toggle");
        } else {
            let plan = correction_edit
                .as_ref()
                .map(|edit| edit.plan.clone())
                .or_else(|| plan_committed_tail_replacement(&mapped_orig, &text))
                .unwrap_or_else(|| TextReplacement {
                    move_left: 0,
                    backspaces: n_backspaces,
                    insert: text.clone(),
                    move_right: 0,
                });
            if let Err(e) = apply_text_replacement(kbd, &plan) {
                log(&format!("⚠ {kind} minimal replace failed: {e}"));
                return None;
            } else {
                let insert_target_is_ru =
                    match insert_text_for_replacement_plan(kbd, &plan, &text, target_is_ru, kind) {
                        Ok(layout) => layout,
                        Err(e) => {
                            log(&format!("⚠ {kind} {e}"));
                            return None;
                        }
                    };
                let layout_result = switch_to_target_layout(insert_target_is_ru);
                buf.remember_pending_learning_correction(
                    kind,
                    &mapped_orig,
                    &text,
                    replace_words,
                    words_orig,
                );
                if !buf.remember_replacement_last_word_for_replay(&events, &plan, &text)
                    && !buf.remember_inserted_tail_for_replay(
                        &events,
                        &plan,
                        preferred_layout_for_text(&plan.insert, insert_target_is_ru),
                    )
                    && !buf.remember_inserted_last_word_for_replay(&events, &plan)
                {
                    buf.reset_all();
                }
                log(&format!(
                    "  1. minimal replace: left={} bs={} insert={:?} right={}",
                    plan.move_left, plan.backspaces, plan.insert, plan.move_right
                ));
                return match layout_result {
                    Ok(layout_id) => {
                        log(&format!("  2. layout → {layout_id}"));
                        log(&format!(
                            "✓ done: {kind}, исправлен BAD-диапазон за {}ms",
                            started_at.elapsed().as_millis()
                        ));
                        Some(insert_target_is_ru)
                    }
                    Err(e) => {
                        log(&format!(
                            "⚠ {kind} layout switch after text insert failed: {e}"
                        ));
                        log(&format!(
                            "✓ done: {kind}, текст исправлен, layout не подтверждён за {}ms",
                            started_at.elapsed().as_millis()
                        ));
                        None
                    }
                };
            }
        }
    }

    // ЭТАП 1: backspace через uinput (надёжно)
    if let Err(e) = emit_backspaces(kbd, n_backspaces) {
        log(&format!("⚠ Этап 1 backspaces failed: {e}"));
        return None;
    }
    log(&format!("  1. uinput Backspace × {n_backspaces}"));

    // ЭТАП 2: переключить раскладку через extension (синхронно через DBus).
    // ActivateLayout — прямой inputSources[i].activate() в JS, мгновенно.
    if let Err(e) = switch_to_target_layout(target_is_ru) {
        log(&format!("⚠ Этап 2 layout switch failed: {e}"));
        if let Err(type_error) = call_type_text(&mapped_target) {
            log(&format!(
                "⚠ fallback TypeText failed after layout switch failure: {type_error}"
            ));
        } else {
            append_learning_log(
                "layout-text-fallback",
                &mapped_orig,
                &mapped_target,
                replace_words,
                words_orig,
            );
            buf.reset_all();
            log(&format!(
                "✓ done: layout fallback text insert за {}ms",
                started_at.elapsed().as_millis()
            ));
        }
        return None;
    }
    let (layout_id, ibus_engine) = target_layout(target_is_ru);
    log(&format!("  2. layout → {layout_id}"));

    // ЭТАП 3: replay тех же keycodes — в новой раскладке дают другие символы.
    if let Err(e) = replay_keycodes(kbd, &events) {
        log(&format!("⚠ Этап 3 replay failed: {e}"));
        return Some(target_is_ru);
    }
    buf.mark_replayed_layout(replace_words, target_is_ru);
    if !force_replay_toggle && mapped_orig != mapped_target {
        append_learning_log(
            "layout-replay",
            &mapped_orig,
            &mapped_target,
            replace_words,
            words_orig,
        );
    }
    log(&format!("  3. uinput replay × {}", events.len()));

    log(&format!(
        "✓ done: раскладка {ibus_engine}, перенабрано {} клавиш за {}ms",
        events.len(),
        started_at.elapsed().as_millis()
    ));
    Some(target_is_ru)
}

fn handle_pending_auto_undo(
    buf: &mut WordBuffer,
    undo: PendingAutoUndo,
    virtual_kbd: Option<&mut VirtualDevice>,
    executing: &mut bool,
    started_at: Instant,
) -> Option<bool> {
    let Some(kbd) = virtual_kbd else {
        log("⚠ auto-undo: нет uinput device");
        return None;
    };

    *executing = true;
    let _executing_guard = ExecutingGuard(executing);

    if let Err(e) = release_possible_modifiers(kbd) {
        log(&format!("⚠ auto-undo modifier cleanup failed: {e}"));
    }

    let plan = pending_auto_undo_plan(&undo);
    if let Err(e) = apply_text_replacement(kbd, &plan) {
        log(&format!("⚠ auto-undo delete failed: {e}"));
        return None;
    }

    let target_is_ru = preferred_layout_for_text(&undo.original, true);
    if let Err(e) = insert_text_via_uinput_or_type_text(kbd, &plan.insert, target_is_ru) {
        log(&format!("⚠ auto-undo insert failed: {e}"));
        return None;
    }
    match switch_to_target_layout(target_is_ru) {
        Ok(layout_id) => log(&format!("  auto-undo layout → {layout_id}")),
        Err(e) => log(&format!("⚠ auto-undo layout switch failed: {e}")),
    }

    append_user_correction_learning_log(&UserLearningCorrection {
        lay_kind: undo.lay_kind.clone(),
        lay_from: undo.original.clone(),
        lay_to: undo.replacement.clone(),
        from: undo.replacement.clone(),
        to: undo.original.clone(),
        replace_words: undo.replace_words,
        words: undo.words,
    });
    buf.clear_pending_learning();
    buf.reset_all();
    log(&format!(
        "✓ done: auto-undo {:?} → {:?} за {}ms",
        undo.replacement,
        undo.original,
        started_at.elapsed().as_millis()
    ));
    Some(target_is_ru)
}

fn pending_auto_undo_plan(undo: &PendingAutoUndo) -> TextReplacement {
    TextReplacement {
        move_left: 0,
        backspaces: undo.replacement.chars().count() as u32,
        insert: undo.original.clone(),
        move_right: 0,
    }
}

fn replay_keycodes(dev: &mut VirtualDevice, events: &[KeyEvent]) -> std::io::Result<()> {
    replay_keycodes_with_pace(dev, events, KEY_PACE_MS, 0)
}

fn replay_text_insert_keycodes(
    dev: &mut VirtualDevice,
    events: &[KeyEvent],
) -> std::io::Result<()> {
    replay_keycodes_with_pace(
        dev,
        events,
        TEXT_INSERT_KEY_PACE_MS,
        TEXT_INSERT_SPACE_SETTLE_MS,
    )
}

fn replay_keycodes_with_pace(
    dev: &mut VirtualDevice,
    events: &[KeyEvent],
    key_pace_ms: u64,
    space_settle_ms: u64,
) -> std::io::Result<()> {
    let shift_l = KeyCode::KEY_LEFTSHIFT.code();

    // CRITICAL: при быстром двойном Shift physical Shift_L юзера может ещё
    // быть «зажат» в kernel/mutter modifier state (FSM сработал по release
    // event, но modifier применяется async). Если не сбросить — все
    // последующие keys получат CAPS от висящего modifier, дают «GHBDTn».
    // Принудительно emit Shift_L/Shift_R release ДО replay — overrides
    // любой stuck physical state.
    release_possible_modifiers(dev)?;

    for ev in events {
        if ev.shift {
            dev.emit(&[
                InputEvent::new(EventType::KEY.0, shift_l, 1),
                InputEvent::new(EventType::KEY.0, ev.keycode, 1),
                InputEvent::new(EventType::KEY.0, ev.keycode, 0),
                InputEvent::new(EventType::KEY.0, shift_l, 0),
            ])?;
        } else {
            dev.emit(&[
                InputEvent::new(EventType::KEY.0, ev.keycode, 1),
                InputEvent::new(EventType::KEY.0, ev.keycode, 0),
            ])?;
        }
        let settle_ms = if ev.keycode == KeyCode::KEY_SPACE.code() && space_settle_ms > 0 {
            space_settle_ms
        } else {
            key_pace_ms
        };
        std::thread::sleep(Duration::from_millis(settle_ms));
    }
    Ok(())
}

fn insert_text_via_uinput_or_type_text(
    dev: &mut VirtualDevice,
    text: &str,
    fallback_is_ru: bool,
) -> Result<TextInsertMethod, String> {
    if let Some(runs) = text_to_uinput_runs(text, fallback_is_ru) {
        for run in runs {
            switch_to_target_layout(run.target_is_ru)?;
            replay_text_insert_keycodes(dev, &run.events).map_err(|e| e.to_string())?;
        }
        return Ok(TextInsertMethod::UinputReplay);
    }

    call_type_text(text).map(|_| TextInsertMethod::TypeTextFallback)
}

fn apply_text_replacement(dev: &mut VirtualDevice, plan: &TextReplacement) -> std::io::Result<()> {
    emit_key_taps(
        dev,
        KeyCode::KEY_LEFT,
        plan.move_left,
        TEXT_REPLACE_KEY_PACE_MS,
    )?;
    emit_backspaces_for_text_replace(dev, plan.backspaces)?;
    Ok(())
}

fn insert_text_for_replacement_plan(
    dev: &mut VirtualDevice,
    plan: &TextReplacement,
    replacement: &str,
    fallback_layout_is_ru: bool,
    label: &str,
) -> Result<bool, String> {
    let insert_layout_is_ru = preferred_layout_for_text(&plan.insert, fallback_layout_is_ru);
    if let Err(e) = insert_text_via_uinput_or_type_text(dev, &plan.insert, insert_layout_is_ru) {
        if let Err(restore_error) = emit_key_taps_fast(dev, KeyCode::KEY_RIGHT, plan.move_right) {
            log(&format!(
                "⚠ {label} cursor restore failed after insert error: {restore_error}"
            ));
        }
        return Err(format!("text insert failed: {e}"));
    }
    if let Err(e) = emit_key_taps_fast(dev, KeyCode::KEY_RIGHT, plan.move_right) {
        return Err(format!("cursor restore failed: {e}"));
    }
    Ok(preferred_layout_for_text(replacement, insert_layout_is_ru))
}

fn emit_key_taps_fast(dev: &mut VirtualDevice, key: KeyCode, n: u32) -> std::io::Result<()> {
    emit_key_taps(dev, key, n, 0)
}

fn emit_key_taps(
    dev: &mut VirtualDevice,
    key: KeyCode,
    n: u32,
    pace_ms: u64,
) -> std::io::Result<()> {
    let code = key.code();
    for _ in 0..n {
        dev.emit(&[
            InputEvent::new(EventType::KEY.0, code, 1),
            InputEvent::new(EventType::KEY.0, code, 0),
        ])?;
        if pace_ms > 0 {
            std::thread::sleep(Duration::from_millis(pace_ms));
        }
    }
    Ok(())
}

fn release_possible_modifiers(dev: &mut VirtualDevice) -> std::io::Result<()> {
    let modifiers = [
        KeyCode::KEY_LEFTSHIFT.code(),
        KeyCode::KEY_RIGHTSHIFT.code(),
        KeyCode::KEY_LEFTCTRL.code(),
        KeyCode::KEY_RIGHTCTRL.code(),
        KeyCode::KEY_LEFTALT.code(),
        KeyCode::KEY_RIGHTALT.code(),
    ];
    let events: Vec<_> = modifiers
        .iter()
        .map(|code| InputEvent::new(EventType::KEY.0, *code, 0))
        .collect();

    for _ in 0..MODIFIER_RELEASE_ROUNDS {
        dev.emit(&events)?;
        std::thread::sleep(Duration::from_millis(MODIFIER_RELEASE_PACE_MS));
    }
    std::thread::sleep(Duration::from_millis(MODIFIER_RELEASE_SETTLE_MS));
    Ok(())
}

fn emit_backspaces(dev: &mut VirtualDevice, n: u32) -> std::io::Result<()> {
    let bs = KeyCode::KEY_BACKSPACE.code();

    // Длинный batch может частично теряться в Mutter/GTK при сотнях клавиш.
    // Пейсинг делает удаление детерминированным для длинных слов.
    for _ in 0..n {
        dev.emit(&[InputEvent::new(EventType::KEY.0, bs, 1)])?;
        std::thread::sleep(Duration::from_millis(BACKSPACE_DOWN_MS));
        dev.emit(&[InputEvent::new(EventType::KEY.0, bs, 0)])?;
        std::thread::sleep(Duration::from_millis(BACKSPACE_PACE_MS));
    }
    std::thread::sleep(Duration::from_millis(BACKSPACE_SETTLE_MS));
    Ok(())
}

fn emit_backspaces_for_text_replace(dev: &mut VirtualDevice, n: u32) -> std::io::Result<()> {
    let bs = KeyCode::KEY_BACKSPACE.code();
    for _ in 0..n {
        dev.emit(&[InputEvent::new(EventType::KEY.0, bs, 1)])?;
        std::thread::sleep(Duration::from_millis(TEXT_REPLACE_BACKSPACE_DOWN_MS));
        dev.emit(&[InputEvent::new(EventType::KEY.0, bs, 0)])?;
        std::thread::sleep(Duration::from_millis(TEXT_REPLACE_BACKSPACE_PACE_MS));
    }
    std::thread::sleep(Duration::from_millis(TEXT_REPLACE_BACKSPACE_SETTLE_MS));
    Ok(())
}

fn switch_ibus_engine(engine: &str) -> Result<(), String> {
    let out = Command::new("ibus")
        .args(["engine", engine])
        .output()
        .map_err(|e| e.to_string())?;
    if !out.status.success() {
        return Err(String::from_utf8_lossy(&out.stderr).trim().to_string());
    }
    Ok(())
}

fn read_ibus_engine() -> Result<String, String> {
    let out = Command::new("ibus")
        .arg("engine")
        .output()
        .map_err(|e| e.to_string())?;
    if !out.status.success() {
        return Err(String::from_utf8_lossy(&out.stderr).trim().to_string());
    }
    Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
}

fn read_current_layout_is_ru() -> Result<bool, String> {
    match active_layout_backend() {
        LayoutBackend::Gnome => read_current_layout_gnome_is_ru(),
        LayoutBackend::Kde => read_current_layout_kde_is_ru(),
        LayoutBackend::X11 => read_current_layout_x11_is_ru(),
    }
}

fn read_current_layout_gnome_is_ru() -> Result<bool, String> {
    call_current_layout()
        .map(|id| is_ru_layout_id(&id))
        .or_else(|_| read_ibus_engine().map(|engine| is_ru_layout_id(&engine)))
}

fn read_current_layout_kde_is_ru() -> Result<bool, String> {
    let qdbus = find_qdbus_command().ok_or_else(|| "qdbus/qdbus6 not found".to_string())?;
    let layout = read_current_kde_layout(qdbus)?;
    Ok(is_ru_layout_id(&layout))
}

fn read_current_layout_x11_is_ru() -> Result<bool, String> {
    let layout = read_x11_layout()?;
    Ok(is_ru_layout_id(&layout))
}

fn call_list_layouts() -> Result<String, String> {
    call_dbus_list_layouts().or_else(|fast_error| {
        reset_dbus_connection();
        log(&format!(
            "⚠ DBus fast ListLayouts failed: {fast_error}; fallback gdbus"
        ));
        run_gdbus(&format!("{DBUS_INTERFACE}.ListLayouts"), &[])
    })
}

fn parse_gdbus_string(reply: &str) -> Option<String> {
    let trimmed = reply.trim();
    let without_tuple = trimmed.strip_prefix("('")?.strip_suffix("',)")?;
    Some(without_tuple.replace("\\'", "'"))
}

fn parse_gdbus_bool(reply: &str) -> Option<bool> {
    let trimmed = reply.trim();
    match trimmed {
        "(true,)" => Some(true),
        "(false,)" => Some(false),
        _ => None,
    }
}

fn parse_current_layout_from_list(layouts: &str) -> Option<String> {
    let list = parse_gdbus_string(layouts).unwrap_or_else(|| layouts.to_string());
    list.split(',').find_map(|entry| {
        let current = entry.strip_suffix('*')?;
        current.rsplit(':').next().map(str::to_string)
    })
}

// ─── uinput re-typing ──────────────────────────────────────

fn make_virtual_keyboard() -> std::io::Result<VirtualDevice> {
    use KeyCode as K;
    // Перечисляем все клавиши которые виртуальное устройство сможет генерировать.
    let mut keys = AttributeSet::new();
    let typing = [
        K::KEY_A,
        K::KEY_B,
        K::KEY_C,
        K::KEY_D,
        K::KEY_E,
        K::KEY_F,
        K::KEY_G,
        K::KEY_H,
        K::KEY_I,
        K::KEY_J,
        K::KEY_K,
        K::KEY_L,
        K::KEY_M,
        K::KEY_N,
        K::KEY_O,
        K::KEY_P,
        K::KEY_Q,
        K::KEY_R,
        K::KEY_S,
        K::KEY_T,
        K::KEY_U,
        K::KEY_V,
        K::KEY_W,
        K::KEY_X,
        K::KEY_Y,
        K::KEY_Z,
        K::KEY_1,
        K::KEY_2,
        K::KEY_3,
        K::KEY_4,
        K::KEY_5,
        K::KEY_6,
        K::KEY_7,
        K::KEY_8,
        K::KEY_9,
        K::KEY_0,
        K::KEY_SPACE,
        K::KEY_SEMICOLON,
        K::KEY_APOSTROPHE,
        K::KEY_COMMA,
        K::KEY_DOT,
        K::KEY_LEFTBRACE,
        K::KEY_RIGHTBRACE,
        K::KEY_GRAVE,
        K::KEY_SLASH,
        K::KEY_BACKSLASH,
        K::KEY_MINUS,
        K::KEY_EQUAL,
        K::KEY_LEFTSHIFT,
        K::KEY_RIGHTSHIFT,
        K::KEY_LEFTALT,
        K::KEY_RIGHTALT,
        K::KEY_LEFTCTRL,
        K::KEY_RIGHTCTRL,
        K::KEY_INSERT,
        K::KEY_LEFT,
        K::KEY_RIGHT,
        K::KEY_BACKSPACE, // для удаления слова с экрана (cut этап)
    ];
    for k in typing.iter() {
        keys.insert(*k);
    }

    VirtualDevice::builder()?
        .name("lay-virtual-keyboard")
        .with_keys(&keys)?
        .build()
}

// ─── DBus и ibus ────────────────────────────────────────────

fn call_ping() -> Result<String, String> {
    call_dbus_ping().or_else(|fast_error| {
        reset_dbus_connection();
        log(&format!(
            "⚠ DBus fast Ping failed: {fast_error}; fallback gdbus"
        ));
        let reply = run_gdbus(&format!("{DBUS_INTERFACE}.Ping"), &[])?;
        parse_gdbus_string(&reply).ok_or_else(|| format!("не распарсил Ping: {reply}"))
    })
}

fn call_activate_layout(id: &str) -> Result<bool, String> {
    call_dbus_activate_layout(id).or_else(|fast_error| {
        reset_dbus_connection();
        log(&format!(
            "⚠ DBus fast ActivateLayout failed: {fast_error}; fallback gdbus"
        ));
        let reply = run_gdbus(
            &format!("{DBUS_INTERFACE}.ActivateLayout"),
            &[&format!("\"{id}\"")],
        )?;
        parse_gdbus_bool(&reply).ok_or_else(|| format!("не распарсил ActivateLayout: {reply}"))
    })
}

fn call_focused_window_info() -> Result<String, String> {
    if active_layout_backend() != LayoutBackend::Gnome {
        return Err("FocusedWindowInfo is available only through the GNOME backend".to_string());
    }
    call_dbus_focused_window_info().or_else(|fast_error| {
        reset_dbus_connection();
        log(&format!(
            "⚠ DBus fast FocusedWindowInfo failed: {fast_error}; fallback gdbus"
        ));
        let reply = run_gdbus(&format!("{DBUS_INTERFACE}.FocusedWindowInfo"), &[])?;
        parse_gdbus_string(&reply).ok_or_else(|| format!("не распарсил FocusedWindowInfo: {reply}"))
    })
}

fn switch_to_layout(layout_id: &str, ibus_engine: &str, target_is_ru: bool) -> Result<(), String> {
    match active_layout_backend() {
        LayoutBackend::Gnome => switch_to_gnome_layout(layout_id, ibus_engine, target_is_ru),
        LayoutBackend::Kde => switch_to_kde_layout(layout_id, target_is_ru),
        LayoutBackend::X11 => switch_to_x11_layout(layout_id, target_is_ru),
    }
}

fn switch_to_gnome_layout(
    layout_id: &str,
    ibus_engine: &str,
    target_is_ru: bool,
) -> Result<(), String> {
    let needs_ime_engine = active_text_backend().should_try_ime();
    let activate_error = match call_activate_layout(layout_id) {
        Ok(true) => {
            if verify_current_layout(target_is_ru) {
                if !needs_ime_engine {
                    return Ok(());
                }
                None
            } else {
                Some("ActivateLayout returned true but layout verify failed".to_string())
            }
        }
        Ok(false) => Some("ActivateLayout returned false".to_string()),
        Err(error) => Some(error),
    };

    let ibus_error = switch_ibus_engine(ibus_engine).err();
    if verify_current_layout(target_is_ru) {
        if let Some(error) = activate_error {
            log(&format!(
                "⚠ ActivateLayout failed, ibus layout verified: {error}"
            ));
        }
        if let Some(error) = ibus_error {
            log(&format!(
                "⚠ SetGlobalEngine failed, GNOME layout verified: {error}"
            ));
        }
        return Ok(());
    }

    Err(match (activate_error, ibus_error) {
        (Some(activate), Some(ibus)) => {
            format!("ActivateLayout failed: {activate}; SetGlobalEngine failed: {ibus}; layout verify failed")
        }
        (Some(activate), None) => {
            format!("ActivateLayout failed: {activate}; layout verify failed")
        }
        (None, Some(ibus)) => format!("SetGlobalEngine failed: {ibus}; layout verify failed"),
        (None, None) => "layout verify failed".to_string(),
    })
}

fn switch_to_kde_layout(layout_id: &str, target_is_ru: bool) -> Result<(), String> {
    let qdbus = find_qdbus_command().ok_or_else(|| "qdbus/qdbus6 not found".to_string())?;
    match kde_layout_index(qdbus, layout_id) {
        Ok(index) => {
            let index = index.to_string();
            run_command_capture(
                qdbus,
                &["org.kde.keyboard", "/Layouts", "setLayout", &index],
            )?;
        }
        Err(index_error) => {
            log(&format!(
                "⚠ KDE indexed layout lookup failed ({index_error}); trying legacy setLayout"
            ));
            run_command_capture(
                qdbus,
                &["org.kde.keyboard", "/Layouts", "setLayout", layout_id],
            )?;
        }
    }
    if verify_current_layout(target_is_ru) {
        Ok(())
    } else {
        Err("KDE layout verify failed".to_string())
    }
}

fn read_current_kde_layout(qdbus: &str) -> Result<String, String> {
    if let Ok(index) = run_command_capture(qdbus, &["org.kde.keyboard", "/Layouts", "getLayout"]) {
        let index = index
            .trim()
            .parse::<usize>()
            .map_err(|e| format!("cannot parse KDE layout index {index:?}: {e}"))?;
        let layouts = kde_layout_ids(qdbus)?;
        return layouts
            .get(index)
            .cloned()
            .ok_or_else(|| format!("KDE layout index {index} out of range: {layouts:?}"));
    }

    run_command_capture(qdbus, &["org.kde.keyboard", "/Layouts", "getCurrentLayout"])
        .map(|layout| normalize_layout_id(&layout))
}

fn kde_layout_index(qdbus: &str, layout_id: &str) -> Result<usize, String> {
    let target = normalize_layout_id(layout_id);
    let layouts = kde_layout_ids(qdbus)?;
    layouts
        .iter()
        .position(|layout| normalize_layout_id(layout) == target)
        .ok_or_else(|| format!("KDE layout {target:?} not found in {layouts:?}"))
}

fn kde_layout_ids(qdbus: &str) -> Result<Vec<String>, String> {
    let output = run_command_capture(
        qdbus,
        &[
            "--literal",
            "org.kde.keyboard",
            "/Layouts",
            "getLayoutsList",
        ],
    )?;
    let layouts = parse_kde_layouts_list(&output);
    if layouts.is_empty() {
        Err(format!("cannot parse KDE layouts: {output}"))
    } else {
        Ok(layouts)
    }
}

fn parse_kde_layouts_list(output: &str) -> Vec<String> {
    output
        .split("(sss)")
        .skip(1)
        .filter_map(|entry| first_quoted_string(entry).map(|layout| normalize_layout_id(&layout)))
        .collect()
}

fn first_quoted_string(input: &str) -> Option<String> {
    let mut chars = input.chars();
    for ch in chars.by_ref() {
        if ch == '"' {
            break;
        }
    }

    let mut out = String::new();
    let mut escaped = false;
    for ch in chars {
        if escaped {
            out.push(ch);
            escaped = false;
            continue;
        }
        match ch {
            '\\' => escaped = true,
            '"' => return Some(out),
            _ => out.push(ch),
        }
    }
    None
}

fn switch_to_x11_layout(layout_id: &str, target_is_ru: bool) -> Result<(), String> {
    if let Err(native_error) = lay::x11_layout::lock_layout_id(layout_id) {
        log(&format!(
            "⚠ native X11 XKB layout switch failed: {native_error}; fallback shell tools"
        ));
    } else if verify_current_layout(target_is_ru) {
        return Ok(());
    } else {
        log("⚠ native X11 XKB layout verify failed; fallback shell tools");
    }

    if command_exists("xkb-switch") {
        run_command_capture("xkb-switch", &["-s", layout_id])?;
    } else {
        run_command_capture("setxkbmap", &[layout_id])?;
    }

    if verify_current_layout(target_is_ru) {
        Ok(())
    } else {
        Err("X11 layout verify failed".to_string())
    }
}

fn switch_to_target_layout(target_is_ru: bool) -> Result<&'static str, String> {
    let (layout_id, ibus_engine) = target_layout(target_is_ru);
    if read_current_layout_is_ru().is_ok_and(|current| current == target_is_ru) {
        return Ok(layout_id);
    }
    switch_to_layout(layout_id, ibus_engine, target_is_ru).map(|()| {
        settle_after_layout_switch();
        layout_id
    })
}

fn target_layout(target_is_ru: bool) -> (&'static str, &'static str) {
    if target_is_ru {
        (
            "ru",
            if active_text_backend().should_try_ime() {
                "lay-ime-ru"
            } else {
                "xkb:ru::rus"
            },
        )
    } else {
        (
            "us",
            if active_text_backend().should_try_ime() {
                "lay-ime-us"
            } else {
                "xkb:us::eng"
            },
        )
    }
}

fn verify_current_layout(target_is_ru: bool) -> bool {
    for _ in 0..5 {
        if read_current_layout_is_ru().is_ok_and(|current| current == target_is_ru) {
            return true;
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    false
}

fn settle_after_layout_switch() {
    std::thread::sleep(Duration::from_millis(LAYOUT_SWITCH_SETTLE_MS));
}

fn settle_after_physical_trigger_release() {
    std::thread::sleep(Duration::from_millis(TRIGGER_RELEASE_SETTLE_MS));
}

fn call_type_text(text: &str) -> Result<String, String> {
    if active_layout_backend() != LayoutBackend::Gnome {
        return Err("TypeText fallback is available only through the GNOME backend".to_string());
    }
    call_dbus_type_text(text).or_else(|fast_error| {
        reset_dbus_connection();
        log(&format!(
            "⚠ DBus fast TypeText failed: {fast_error}; fallback gdbus"
        ));
        call_type_text_gdbus(text)
    })
}

fn call_replace_text(
    move_left: u32,
    backspaces: u32,
    text: &str,
    move_right: u32,
    layout_id: &str,
) -> Result<bool, String> {
    if active_layout_backend() != LayoutBackend::Gnome {
        return Err("ReplaceText is available only through the GNOME backend".to_string());
    }
    call_dbus_replace_text(move_left, backspaces, text, move_right, layout_id).or_else(
        |fast_error| {
            reset_dbus_connection();
            log(&format!(
                "⚠ DBus fast ReplaceText failed: {fast_error}; fallback gdbus"
            ));
            call_replace_text_gdbus(move_left, backspaces, text, move_right, layout_id)
        },
    )
}

fn should_try_ime_text_backend() -> bool {
    active_text_backend().should_try_ime()
}

fn try_ime_replace_tail(original: &str, replacement: &str, kind: &str) -> Result<bool, String> {
    if !should_try_ime_text_backend() {
        return Ok(false);
    }
    let request = ImeReplaceRequest::committed_tail(original, replacement);
    if request.is_noop() {
        return Ok(false);
    }
    match call_ime_replace_tail(request.backspaces, &request.text) {
        Ok(true) => {
            log(&format!(
                "  IME replace-tail ({kind}): bs={} insert={:?}",
                request.backspaces, request.text
            ));
            Ok(true)
        }
        Ok(false) => {
            log("⚠ IME replace-tail returned false; fallback to uinput");
            Ok(false)
        }
        Err(e) => {
            log(&format!(
                "⚠ IME replace-tail failed: {e}; fallback to uinput"
            ));
            Err(e)
        }
    }
}

fn call_ime_ping() -> Result<String, String> {
    let reply = dbus_connection()?
        .call_method(
            Some(IME_DBUS_DEST),
            IME_DBUS_PATH,
            Some(IME_DBUS_INTERFACE),
            "Ping",
            &(),
        )
        .map_err(|e| e.to_string())?;
    reply
        .body()
        .deserialize::<String>()
        .map_err(|e| e.to_string())
}

fn call_ime_replace_tail(backspaces: u32, text: &str) -> Result<bool, String> {
    let reply = dbus_connection()?
        .call_method(
            Some(IME_DBUS_DEST),
            IME_DBUS_PATH,
            Some(IME_DBUS_INTERFACE),
            "ReplaceTail",
            &(backspaces, text),
        )
        .map_err(|e| e.to_string())?;
    reply
        .body()
        .deserialize::<bool>()
        .map_err(|e| e.to_string())
}

fn call_type_text_gdbus(text: &str) -> Result<String, String> {
    let arg = gvariant_string(text);
    run_gdbus(&format!("{DBUS_INTERFACE}.TypeText"), &[&arg])
}

fn call_replace_text_gdbus(
    move_left: u32,
    backspaces: u32,
    text: &str,
    move_right: u32,
    layout_id: &str,
) -> Result<bool, String> {
    let text_arg = gvariant_string(text);
    let layout_arg = gvariant_string(layout_id);
    let reply = run_gdbus(
        &format!("{DBUS_INTERFACE}.ReplaceText"),
        &[
            &move_left.to_string(),
            &backspaces.to_string(),
            &text_arg,
            &move_right.to_string(),
            &layout_arg,
        ],
    )?;
    parse_gdbus_bool(&reply).ok_or_else(|| format!("не распарсил ReplaceText: {reply}"))
}

fn dbus_connection() -> Result<zbus::blocking::Connection, String> {
    let cell = DBUS_CONNECTION.get_or_init(|| Mutex::new(None));
    let mut guard = cell.lock().map_err(|e| e.to_string())?;
    if let Some(conn) = guard.as_ref() {
        return Ok(conn.clone());
    }

    let conn = zbus::blocking::Connection::session().map_err(|e| e.to_string())?;
    *guard = Some(conn.clone());
    Ok(conn)
}

fn reset_dbus_connection() {
    if let Some(cell) = DBUS_CONNECTION.get() {
        if let Ok(mut guard) = cell.lock() {
            *guard = None;
        }
    }
}

fn call_dbus_ping() -> Result<String, String> {
    let reply = dbus_connection()?
        .call_method(
            Some(DBUS_DEST),
            DBUS_PATH,
            Some(DBUS_INTERFACE),
            "Ping",
            &(),
        )
        .map_err(|e| e.to_string())?;
    reply
        .body()
        .deserialize::<String>()
        .map_err(|e| e.to_string())
}

fn call_dbus_type_text(text: &str) -> Result<String, String> {
    dbus_connection()?
        .call_method(
            Some(DBUS_DEST),
            DBUS_PATH,
            Some(DBUS_INTERFACE),
            "TypeText",
            &text,
        )
        .map_err(|e| e.to_string())?;
    Ok(String::new())
}

fn call_dbus_replace_text(
    move_left: u32,
    backspaces: u32,
    text: &str,
    move_right: u32,
    layout_id: &str,
) -> Result<bool, String> {
    let reply = dbus_connection()?
        .call_method(
            Some(DBUS_DEST),
            DBUS_PATH,
            Some(DBUS_INTERFACE),
            "ReplaceText",
            &(move_left, backspaces, text, move_right, layout_id),
        )
        .map_err(|e| e.to_string())?;
    reply
        .body()
        .deserialize::<bool>()
        .map_err(|e| e.to_string())
}

fn call_dbus_activate_layout(id: &str) -> Result<bool, String> {
    let reply = dbus_connection()?
        .call_method(
            Some(DBUS_DEST),
            DBUS_PATH,
            Some(DBUS_INTERFACE),
            "ActivateLayout",
            &id,
        )
        .map_err(|e| e.to_string())?;
    reply
        .body()
        .deserialize::<bool>()
        .map_err(|e| e.to_string())
}

fn call_current_layout() -> Result<String, String> {
    call_dbus_current_layout().or_else(|fast_error| {
        reset_dbus_connection();
        log(&format!(
            "⚠ DBus fast CurrentLayout failed: {fast_error}; fallback gdbus"
        ));
        let current = run_gdbus(&format!("{DBUS_INTERFACE}.CurrentLayout"), &[]);
        match current {
            Ok(reply) => parse_gdbus_string(&reply).ok_or_else(|| format!("не распарсил: {reply}")),
            Err(current_error) => {
                let layouts = call_list_layouts()
                    .map_err(|list_error| format!("{current_error}; ListLayouts: {list_error}"))?;
                parse_current_layout_from_list(&layouts)
                    .ok_or_else(|| format!("не нашёл текущую раскладку: {layouts}"))
            }
        }
    })
}

fn call_dbus_current_layout() -> Result<String, String> {
    let reply = dbus_connection()?
        .call_method(
            Some(DBUS_DEST),
            DBUS_PATH,
            Some(DBUS_INTERFACE),
            "CurrentLayout",
            &(),
        )
        .map_err(|e| e.to_string())?;
    reply
        .body()
        .deserialize::<String>()
        .map_err(|e| e.to_string())
}

fn call_dbus_list_layouts() -> Result<String, String> {
    let reply = dbus_connection()?
        .call_method(
            Some(DBUS_DEST),
            DBUS_PATH,
            Some(DBUS_INTERFACE),
            "ListLayouts",
            &(),
        )
        .map_err(|e| e.to_string())?;
    reply
        .body()
        .deserialize::<String>()
        .map_err(|e| e.to_string())
}

fn call_dbus_focused_window_info() -> Result<String, String> {
    let reply = dbus_connection()?
        .call_method(
            Some(DBUS_DEST),
            DBUS_PATH,
            Some(DBUS_INTERFACE),
            "FocusedWindowInfo",
            &(),
        )
        .map_err(|e| e.to_string())?;
    reply
        .body()
        .deserialize::<String>()
        .map_err(|e| e.to_string())
}

fn gvariant_string(text: &str) -> String {
    format!("{text:?}")
}

fn run_gdbus(method: &str, args: &[&str]) -> Result<String, String> {
    let mut cmd_args = vec![
        "call",
        "--session",
        "--dest",
        DBUS_DEST,
        "--object-path",
        DBUS_PATH,
        "--method",
        method,
    ];
    cmd_args.extend(args);
    let out = Command::new("gdbus")
        .args(&cmd_args)
        .output()
        .map_err(|e| e.to_string())?;
    if !out.status.success() {
        return Err(String::from_utf8_lossy(&out.stderr).trim().to_string());
    }
    Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
}

fn run_command_capture(command: &str, args: &[&str]) -> Result<String, String> {
    let out = Command::new(command)
        .args(args)
        .output()
        .map_err(|e| format!("{command}: {e}"))?;
    if !out.status.success() {
        return Err(format!(
            "{command}: {}",
            String::from_utf8_lossy(&out.stderr).trim()
        ));
    }
    Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
}

fn command_exists(command: &str) -> bool {
    let Some(paths) = std::env::var_os("PATH") else {
        return false;
    };
    std::env::split_paths(&paths).any(|dir| dir.join(command).is_file())
}

fn find_qdbus_command() -> Option<&'static str> {
    ["qdbus6", "qdbus-qt6", "qdbus"]
        .into_iter()
        .find(|cmd| command_exists(cmd))
}

fn detect_auto_layout_backend_hint() -> Option<LayoutBackend> {
    let qdbus = find_qdbus_command()?;
    if run_command_capture(qdbus, &["org.kde.keyboard", "/Layouts", "getLayout"]).is_ok()
        || run_command_capture(qdbus, &["org.kde.keyboard", "/Layouts", "getCurrentLayout"]).is_ok()
    {
        return Some(LayoutBackend::Kde);
    }
    None
}

fn read_x11_layout() -> Result<String, String> {
    if let Ok(layout) = lay::x11_layout::current_layout_id() {
        return Ok(layout);
    }

    if command_exists("xkb-switch") {
        return run_command_capture("xkb-switch", &[]).map(|layout| normalize_layout_id(&layout));
    }
    if command_exists("xkblayout-state") {
        return run_command_capture("xkblayout-state", &["print", "%s"])
            .map(|layout| normalize_layout_id(&layout));
    }

    let query = run_command_capture("setxkbmap", &["-query"])?;
    parse_setxkbmap_layout(&query).ok_or_else(|| format!("cannot parse setxkbmap output: {query}"))
}

// ─── Поиск устройства клавиатуры ────────────────────────────

fn find_all_keyboards() -> std::io::Result<Vec<std::path::PathBuf>> {
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

fn should_ignore_keyboard_device_name(name: &str) -> bool {
    matches!(name, "lay-virtual-keyboard" | "ydotoold virtual device")
}

// ─── Лог ────────────────────────────────────────────────────

static LOG_ENABLED: OnceLock<bool> = OnceLock::new();

fn set_log_enabled(enabled: bool) {
    let env_enabled = std::env::var("LAY_DEBUG_LOG")
        .is_ok_and(|v| matches!(v.as_str(), "1" | "true" | "yes" | "on"));
    let _ = LOG_ENABLED.set(enabled || env_enabled);
}

fn log(msg: &str) {
    if !*LOG_ENABLED.get_or_init(|| {
        std::env::var("LAY_DEBUG_LOG")
            .is_ok_and(|v| matches!(v.as_str(), "1" | "true" | "yes" | "on"))
    }) {
        return;
    }

    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let line = format!("[{ts}] {msg}\n");
    eprint!("{line}");
}

fn unix_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

#[derive(serde::Serialize)]
struct LearningEntry<'a> {
    ts: u64,
    kind: &'a str,
    from: &'a str,
    to: &'a str,
    replace_words: usize,
    words: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    lay_kind: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    lay_from: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    lay_to: Option<&'a str>,
}

fn append_learning_log(kind: &str, from: &str, to: &str, replace_words: usize, words: usize) {
    if !active_learning_log() {
        return;
    }
    let Some(home) = std::env::var_os("HOME") else {
        return;
    };
    let path = std::path::PathBuf::from(home).join(LEARN_LOG_PATH);
    append_learning_log_to_path(&path, kind, from, to, replace_words, words);
}

fn append_user_correction_learning_log(correction: &UserLearningCorrection) {
    if !active_learning_log() {
        return;
    }
    let Some(home) = std::env::var_os("HOME") else {
        return;
    };
    let home = std::path::PathBuf::from(home);
    let path = home.join(LEARN_LOG_PATH);
    append_user_correction_learning_log_to_path(&path, correction);
    match promote_user_correction_if_repeated(
        &home.join(LEARN_CANDIDATES_PATH),
        &home.join(REPLACEMENTS_PATH),
        correction,
    ) {
        LearningPromotion::Promoted { from, to } => {
            log(&format!("  learn: promoted exact rule {from:?} → {to:?}"));
        }
        LearningPromotion::Recorded { count, from, to } => {
            log(&format!(
                "  learn: candidate {from:?} → {to:?}, count={count}/{LEARN_PROMOTION_THRESHOLD}"
            ));
        }
        LearningPromotion::Skipped => {}
    }
}

fn append_learning_log_to_path(
    path: &std::path::Path,
    kind: &str,
    from: &str,
    to: &str,
    replace_words: usize,
    words: usize,
) {
    let entry = LearningEntry {
        ts: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs(),
        kind,
        from,
        to,
        replace_words,
        words,
        lay_kind: None,
        lay_from: None,
        lay_to: None,
    };
    append_learning_entry_to_path(path, &entry);
}

fn append_user_correction_learning_log_to_path(
    path: &std::path::Path,
    correction: &UserLearningCorrection,
) {
    let entry = LearningEntry {
        ts: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs(),
        kind: "user-correction",
        from: &correction.from,
        to: &correction.to,
        replace_words: correction.replace_words,
        words: correction.words,
        lay_kind: Some(&correction.lay_kind),
        lay_from: Some(&correction.lay_from),
        lay_to: Some(&correction.lay_to),
    };
    append_learning_entry_to_path(path, &entry);
}

fn append_learning_entry_to_path(path: &std::path::Path, entry: &LearningEntry<'_>) {
    if entry.from == entry.to || entry.from.trim().is_empty() || entry.to.trim().is_empty() {
        return;
    }

    if let Some(parent) = path.parent() {
        if let Err(e) = std::fs::create_dir_all(parent) {
            log(&format!("⚠ learn-log mkdir failed: {e}"));
            return;
        }
    }

    let Ok(mut line) = serde_json::to_string(&entry) else {
        return;
    };
    line.push('\n');

    match std::fs::OpenOptions::new()
        .append(true)
        .create(true)
        .open(path)
    {
        Ok(mut f) => {
            if f.write_all(line.as_bytes()).is_ok() {
                compact_learning_log_if_needed(path);
                #[cfg(not(test))]
                lay::stats::record_learning_log_entry(entry.kind);
                log("  learn-log: correction saved");
            }
        }
        Err(e) => log(&format!("⚠ learn-log open failed: {e}")),
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum LearningPromotion {
    Skipped,
    Recorded {
        from: String,
        to: String,
        count: u32,
    },
    Promoted {
        from: String,
        to: String,
    },
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
struct LearningCandidate {
    from: String,
    to: String,
    count: u32,
    first_ts: u64,
    last_ts: u64,
    promoted: bool,
}

fn promote_user_correction_if_repeated(
    candidates_path: &std::path::Path,
    replacements_path: &std::path::Path,
    correction: &UserLearningCorrection,
) -> LearningPromotion {
    let Some((from, to)) = normalizable_learning_rule(correction) else {
        return LearningPromotion::Skipped;
    };

    let now = unix_timestamp();
    let key = format!("{from}\u{1f}{to}");
    let mut candidates = load_learning_candidates(candidates_path);
    let candidate = candidates.entry(key).or_insert_with(|| LearningCandidate {
        from: from.clone(),
        to: to.clone(),
        count: 0,
        first_ts: now,
        last_ts: now,
        promoted: false,
    });
    candidate.count = candidate.count.saturating_add(1);
    candidate.last_ts = now;

    if candidate.promoted {
        remember_promoted_replacement(&from, &to);
        let _ = save_learning_candidates(candidates_path, &candidates);
        return LearningPromotion::Promoted { from, to };
    }

    if candidate.count < LEARN_PROMOTION_THRESHOLD {
        let count = candidate.count;
        let _ = save_learning_candidates(candidates_path, &candidates);
        return LearningPromotion::Recorded { from, to, count };
    }

    match add_replacement_rule_to_path(replacements_path, &from, &to) {
        Ok(true) | Ok(false) => {
            candidate.promoted = true;
            remember_promoted_replacement(&from, &to);
            #[cfg(not(test))]
            lay::stats::record_learning_promotion();
            let _ = save_learning_candidates(candidates_path, &candidates);
            LearningPromotion::Promoted { from, to }
        }
        Err(e) => {
            log(&format!("⚠ learn promotion failed: {e}"));
            let _ = save_learning_candidates(candidates_path, &candidates);
            LearningPromotion::Skipped
        }
    }
}

fn normalizable_learning_rule(correction: &UserLearningCorrection) -> Option<(String, String)> {
    if correction.lay_kind == "layout-replay" {
        return None;
    }

    let from = correction.from.trim();
    let to = correction.to.trim();
    if from.is_empty() || to.is_empty() || from == to {
        return None;
    }
    if from.split_whitespace().count() != 1 || to.split_whitespace().count() > 3 {
        return None;
    }

    let from_lower = from.to_lowercase();
    let to_lower = to.to_lowercase();
    let from_letters = from_lower.chars().filter(|ch| ch.is_alphabetic()).count();
    let to_letters = to_lower.chars().filter(|ch| ch.is_alphabetic()).count();
    if from_letters < 4 || to_letters < 2 {
        return None;
    }
    if !is_cyrillic_word(&from_lower) {
        return None;
    }
    if !to_lower
        .chars()
        .all(|ch| is_cyrillic_letter(ch) || ch.is_whitespace() || ch == '-')
    {
        return None;
    }
    if is_known_russian_word_or_form(&from_lower) {
        return None;
    }

    Some((from_lower, to_lower))
}

fn load_learning_candidates(path: &std::path::Path) -> BTreeMap<String, LearningCandidate> {
    std::fs::read_to_string(path)
        .ok()
        .and_then(|text| serde_json::from_str(&text).ok())
        .unwrap_or_default()
}

fn save_learning_candidates(
    path: &std::path::Path,
    candidates: &BTreeMap<String, LearningCandidate>,
) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let text = serde_json::to_string_pretty(candidates).unwrap_or_else(|_| "{}".to_string());
    std::fs::write(path, format!("{text}\n"))
}

fn add_replacement_rule_to_path(
    path: &std::path::Path,
    from: &str,
    to: &str,
) -> Result<bool, String> {
    let mut rules: BTreeMap<String, String> = std::fs::read_to_string(path)
        .ok()
        .and_then(|text| serde_json::from_str(&text).ok())
        .unwrap_or_default();

    if let Some(existing) = rules.get(from) {
        if existing == to {
            return Ok(false);
        }
        return Err(format!(
            "replacement conflict for {from:?}: existing {existing:?}, learned {to:?}"
        ));
    }

    rules.insert(from.to_string(), to.to_string());
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let text = serde_json::to_string_pretty(&rules).map_err(|e| e.to_string())?;
    std::fs::write(path, format!("{text}\n")).map_err(|e| e.to_string())?;
    Ok(true)
}

fn compact_learning_log_if_needed(path: &std::path::Path) {
    let Ok(meta) = std::fs::metadata(path) else {
        return;
    };
    if meta.len() <= LEARN_LOG_MAX_BYTES {
        return;
    }

    let Ok(content) = std::fs::read_to_string(path) else {
        return;
    };
    let compacted = keep_last_jsonl_lines(&content, LEARN_LOG_KEEP_LINES);
    if std::fs::write(path, compacted).is_ok() {
        log("  learn-log: compacted");
    }
}

fn keep_last_jsonl_lines(content: &str, max_lines: usize) -> String {
    let lines: Vec<&str> = content.lines().collect();
    let start = lines.len().saturating_sub(max_lines);
    let mut out = lines[start..].join("\n");
    if !out.is_empty() {
        out.push('\n');
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Once;

    fn seed_test_replacements() {
        static ONCE: Once = Once::new();
        ONCE.call_once(|| {
            for (from, to) in [
                ("подлючись", "подключись"),
                ("надйи", "найди"),
                ("нуда", "ну да"),
                ("вчем", "в чем"),
                ("можн", "можно"),
                ("дльше", "дальше"),
                ("дальг", "дальше"),
                ("првильно", "правильно"),
            ] {
                remember_promoted_replacement(from, to);
            }
        });
    }

    fn apply_typing_assist_exact(text: &str) -> Option<String> {
        seed_test_replacements();
        lay::typing_assist::apply_typing_assist_exact(text)
    }

    fn apply_typing_assist(text: &str, allow_layout_auto: bool) -> Option<String> {
        seed_test_replacements();
        lay::typing_assist::apply_typing_assist(text, allow_layout_auto)
    }

    fn apply_auto_replace(original: &str, target: &str) -> Option<String> {
        seed_test_replacements();
        lay::typing_assist::apply_auto_replace(original, target)
    }

    fn key_event(key: KeyCode, layout_is_ru: bool) -> KeyEvent {
        KeyEvent {
            keycode: key.code(),
            shift: false,
            layout_is_ru,
        }
    }

    fn push_keys(buffer: &mut WordBuffer, keys: &[KeyCode], layout_is_ru: bool) {
        for key in keys {
            buffer.push(key_event(*key, layout_is_ru));
        }
    }

    fn key_events(keys: &[KeyCode], layout_is_ru: bool) -> Vec<KeyEvent> {
        keys.iter()
            .map(|key| key_event(*key, layout_is_ru))
            .collect()
    }

    #[test]
    fn idle_wait_uses_long_sleep_when_no_internal_deadlines() {
        let now = Instant::now();

        assert_eq!(
            idle_wait_timeout_at(now, None, None, now, Duration::from_millis(120)),
            Duration::from_millis(IDLE_EVENT_WAIT_MAX_MS)
        );
    }

    #[test]
    fn idle_wait_keeps_multi_tap_and_typing_assist_deadlines_precise() {
        let now = Instant::now();
        let pending = MultiTapPending {
            tap_count: 2,
            last_release: now - Duration::from_millis(80),
        };

        assert_eq!(
            idle_wait_timeout_at(
                now,
                Some(&pending),
                Some(now - Duration::from_millis(40)),
                now,
                Duration::from_millis(120)
            ),
            Duration::from_millis(15)
        );
    }

    #[test]
    fn idle_wait_returns_zero_when_a_deadline_is_due() {
        let now = Instant::now();

        assert_eq!(
            idle_wait_timeout_at(
                now,
                None,
                Some(now - Duration::from_millis(TYPING_ASSIST_IDLE_DELAY_MS)),
                now,
                Duration::from_millis(120)
            ),
            Duration::ZERO
        );
    }

    #[test]
    fn shift_state_cleanup_after_trigger_keeps_shortcuts_but_drops_caps() {
        let mut state = ShiftState::default();
        state.update(KeyCode::KEY_LEFTSHIFT, 1);
        state.update(KeyCode::KEY_RIGHTSHIFT, 1);
        state.update(KeyCode::KEY_LEFTCTRL, 1);

        assert!(state.any());
        assert!(state.shortcut_active());

        state.clear_shifts();

        assert!(!state.any());
        assert!(state.shortcut_active());
    }

    fn ascii_hyphen_token_keycodes() -> [KeyCode; 5] {
        [
            KeyCode::KEY_W,
            KeyCode::KEY_I,
            KeyCode::KEY_MINUS,
            KeyCode::KEY_F,
            KeyCode::KEY_I,
        ]
    }

    fn typing_pipeline_with_disabled(disabled: &[&str]) -> Vec<TypingAssistRuleConfig> {
        default_typing_assist_pipeline()
            .into_iter()
            .map(|mut rule| {
                if disabled.iter().any(|id| *id == rule.id) {
                    rule.enabled = false;
                }
                rule
            })
            .collect()
    }

    fn typing_pipeline_with_only(enabled: &str) -> Vec<TypingAssistRuleConfig> {
        default_typing_assist_pipeline()
            .into_iter()
            .map(|mut rule| {
                rule.enabled = rule.id == enabled;
                rule
            })
            .collect()
    }

    fn typing_pipeline_with_first(first: &str) -> Vec<TypingAssistRuleConfig> {
        let mut rules = default_typing_assist_pipeline();
        for rule in &mut rules {
            rule.priority += 10;
            if rule.id == first {
                rule.priority = 1;
            }
        }
        rules
    }

    #[test]
    fn text_insert_runs_use_uinput_layout_channels() {
        let runs = text_to_uinput_runs("Привет Double", true).expect("typable text");
        assert_eq!(runs.len(), 2);
        assert!(runs[0].target_is_ru);
        assert!(!runs[1].target_is_ru);
        assert_eq!(map_events_to_layout(&runs[0].events, true), "Привет ");
        assert_eq!(map_events_to_layout(&runs[1].events, false), "Double");

        let runs = text_to_uinput_runs("ну да ", true).expect("typable text");
        assert_eq!(runs.len(), 1);
        assert!(runs[0].target_is_ru);
        assert_eq!(map_events_to_layout(&runs[0].events, true), "ну да ");

        let runs = text_to_uinput_runs("hello world", false).expect("typable text");
        assert_eq!(runs.len(), 1);
        assert!(!runs[0].target_is_ru);
        assert_eq!(map_events_to_layout(&runs[0].events, false), "hello world");

        assert!(text_to_uinput_runs("привет 🙂", true).is_none());
    }

    #[test]
    fn typing_assist_minimal_plan_keeps_inter_word_space() {
        let plan = plan_text_replacement("чтобы точнр ", "чтобы точно ").expect("replacement");

        assert_eq!(plan.move_left, 1);
        assert_eq!(plan.backspaces, 1);
        assert_eq!(plan.insert, "о");
        assert_eq!(plan.move_right, 1);
    }

    #[test]
    fn committed_tail_plan_preserves_typed_trailing_space_after_short_replacement() {
        let plan = plan_committed_tail_replacement("double b ", "double и ").expect("replacement");

        assert_eq!(plan.move_left, 1);
        assert_eq!(plan.backspaces, 1);
        assert_eq!(plan.insert, "и");
        assert_eq!(plan.move_right, 1);
    }

    #[test]
    fn committed_tail_plan_does_not_leave_space_behind_single_letter_fix() {
        let plan =
            plan_committed_tail_replacement("чтобы точнр ", "чтобы точно ").expect("replacement");

        assert_eq!(plan.move_left, 1);
        assert_eq!(plan.backspaces, 1);
        assert_eq!(plan.insert, "о");
        assert_eq!(plan.move_right, 1);
    }

    #[test]
    fn replacement_memory_keeps_space_boundary_after_i_autofix() {
        let mut buffer = WordBuffer::new();
        push_text_as_layout(&mut buffer, "double b ", false);
        let events = buffer
            .last_completed_words_events(2)
            .expect("completed two-word tail");
        let original = map_original_events(&events);
        let replacement = "double и ";
        let plan = plan_committed_tail_replacement(&original, replacement).expect("replacement");

        assert!(buffer.remember_replacement_last_word_for_replay(&events, &plan, replacement));
        assert!(buffer.current_is_empty());
        assert!(buffer.prev_had_trailing_space());
        assert_eq!(buffer.prev_words_len(), 1);
        assert_eq!(
            map_original_events(buffer.prev_word_events(0).expect("prev word")),
            "и"
        );

        push_text_as_layout(&mut buffer, "слово", true);
        let (tail, _) = buffer.what_to_replay(2).expect("two-word tail");

        assert_eq!(map_original_events(&tail), "и слово");
    }

    #[test]
    fn enter_autocorrect_candidate_is_off_contract_until_enabled_by_config() {
        let cfg = LayConfig::default();
        assert!(!cfg.enter_autocorrect);
        assert!(!active_enter_autocorrect_from_env(true, None));
        assert!(!active_enter_autocorrect_from_env(true, Some("0")));
        assert!(active_enter_autocorrect_from_env(true, Some("1")));
        assert!(active_enter_autocorrect_from_env(true, Some("true")));
    }

    #[test]
    fn enter_autocorrect_candidate_fixes_current_wrong_layout_word() {
        let mut buffer = WordBuffer::new();
        push_text_as_layout(&mut buffer, "ghbdtn", false);
        let pipeline = typing_pipeline_with_only("layout_en_to_ru");

        let (_events, edit) =
            enter_autocorrect_candidate(&buffer, 1, true, &pipeline).expect("correction");

        assert_eq!(edit.original, "ghbdtn");
        assert_eq!(edit.replacement, "привет");
    }

    #[test]
    fn enter_autocorrect_candidate_keeps_normal_english_word() {
        let mut buffer = WordBuffer::new();
        push_text_as_layout(&mut buffer, "good", false);
        let pipeline = typing_pipeline_with_only("layout_en_to_ru");

        assert!(enter_autocorrect_candidate(&buffer, 1, true, &pipeline).is_none());
    }

    #[test]
    fn enter_autocorrect_candidate_can_use_completed_tail_scope() {
        let mut buffer = WordBuffer::new();
        push_text_as_layout(&mut buffer, "double", false);
        buffer.handle_space();
        push_text_as_layout(&mut buffer, "b", false);
        let pipeline = typing_pipeline_with_only("visual_b");

        let (_events, edit) =
            enter_autocorrect_candidate(&buffer, 2, true, &pipeline).expect("correction");

        assert_eq!(edit.original, "double b");
        assert_eq!(edit.replacement, "double и");
    }

    fn push_key_events(buffer: &mut WordBuffer, keys: &[(KeyCode, bool)], layout_is_ru: bool) {
        for (key, shift) in keys {
            buffer.push(KeyEvent {
                keycode: key.code(),
                shift: *shift,
                layout_is_ru,
            });
        }
    }

    fn text_key_event(ch: char, layout_is_ru: bool) -> KeyEvent {
        const KEYS: &[KeyCode] = &[
            KeyCode::KEY_A,
            KeyCode::KEY_B,
            KeyCode::KEY_C,
            KeyCode::KEY_D,
            KeyCode::KEY_E,
            KeyCode::KEY_F,
            KeyCode::KEY_G,
            KeyCode::KEY_H,
            KeyCode::KEY_I,
            KeyCode::KEY_J,
            KeyCode::KEY_K,
            KeyCode::KEY_L,
            KeyCode::KEY_M,
            KeyCode::KEY_N,
            KeyCode::KEY_O,
            KeyCode::KEY_P,
            KeyCode::KEY_Q,
            KeyCode::KEY_R,
            KeyCode::KEY_S,
            KeyCode::KEY_T,
            KeyCode::KEY_U,
            KeyCode::KEY_V,
            KeyCode::KEY_W,
            KeyCode::KEY_X,
            KeyCode::KEY_Y,
            KeyCode::KEY_Z,
            KeyCode::KEY_1,
            KeyCode::KEY_2,
            KeyCode::KEY_3,
            KeyCode::KEY_4,
            KeyCode::KEY_5,
            KeyCode::KEY_6,
            KeyCode::KEY_7,
            KeyCode::KEY_8,
            KeyCode::KEY_9,
            KeyCode::KEY_0,
            KeyCode::KEY_SEMICOLON,
            KeyCode::KEY_APOSTROPHE,
            KeyCode::KEY_COMMA,
            KeyCode::KEY_DOT,
            KeyCode::KEY_LEFTBRACE,
            KeyCode::KEY_RIGHTBRACE,
            KeyCode::KEY_GRAVE,
            KeyCode::KEY_SLASH,
            KeyCode::KEY_BACKSLASH,
            KeyCode::KEY_MINUS,
            KeyCode::KEY_EQUAL,
        ];

        for key in KEYS {
            for shift in [false, true] {
                let mapped = if layout_is_ru {
                    keycode_to_ru_char(key.code(), shift)
                } else {
                    keycode_to_us_char(key.code(), shift)
                };
                if mapped == Some(ch) {
                    return KeyEvent {
                        keycode: key.code(),
                        shift,
                        layout_is_ru,
                    };
                }
            }
        }

        panic!("no key event for {ch:?} in layout_is_ru={layout_is_ru}");
    }

    fn push_text_as_layout(buffer: &mut WordBuffer, text: &str, layout_is_ru: bool) {
        for ch in text.chars() {
            if ch == ' ' {
                buffer.handle_space();
            } else {
                buffer.push(text_key_event(ch, layout_is_ru));
            }
        }
    }

    fn assert_smart_pair(
        left: &str,
        left_layout_is_ru: bool,
        current_typed: &str,
        current_layout_is_ru: bool,
        expected: &str,
    ) {
        let mut buffer = WordBuffer::new();
        push_text_as_layout(&mut buffer, left, left_layout_is_ru);
        buffer.handle_space();
        push_text_as_layout(&mut buffer, current_typed, current_layout_is_ru);
        let (events, _) = buffer.what_to_replay(2).expect("two-word tail");
        let original = map_original_events(&events);
        let got = decide_scoped_tail_correction(&events).unwrap_or(original.clone());

        assert_eq!(got, expected, "original tail: {original:?}");
    }

    fn map_target_events(events: &[KeyEvent], target_is_ru: bool) -> String {
        events
            .iter()
            .filter_map(|ev| {
                if target_is_ru {
                    keycode_to_ru_char(ev.keycode, ev.shift)
                } else {
                    keycode_to_us_char(ev.keycode, ev.shift)
                }
            })
            .collect()
    }

    fn apply_typing_assist_to_text_tail(text: &str) -> Option<String> {
        apply_typing_assist_exact(text).or_else(|| {
            let (leading, core, trailing) = split_edge_whitespace(text);
            let segments = split_ws_segments(core);
            if segments.len() < 3 {
                return None;
            }

            for word_count in [2, 1] {
                let mut suffix_start = core.len();
                let mut non_ws_seen = 0;
                for (segment, is_ws) in segments.iter().rev() {
                    suffix_start -= segment.len();
                    if !is_ws {
                        non_ws_seen += 1;
                        if non_ws_seen == word_count {
                            break;
                        }
                    }
                }

                let prefix = &core[..suffix_start];
                let suffix = &core[suffix_start..];
                if let Some(replacement) = apply_typing_assist_exact(&format!("{suffix}{trailing}"))
                {
                    return Some(format!("{leading}{prefix}{replacement}"));
                }
            }

            None
        })
    }

    #[test]
    fn parses_gdbus_string_tuple() {
        assert_eq!(parse_gdbus_string("('us',)"), Some("us".to_string()));
    }

    #[test]
    fn parses_current_layout_from_list_layouts_reply() {
        assert_eq!(
            parse_current_layout_from_list("('0:xkb:us,1:xkb:ru*',)"),
            Some("ru".to_string())
        );
    }

    #[test]
    fn parses_kde6_layout_list_reply() {
        let reply = r#"[Argument: a(sss) {[Argument: (sss) "us", "", "English (US)"], [Argument: (sss) "ru", "", "Russian"]}]"#;
        assert_eq!(parse_kde_layouts_list(reply), vec!["us", "ru"]);
    }

    #[test]
    fn parses_first_quoted_string_with_escapes() {
        assert_eq!(
            first_quoted_string(r#" "us\"intl", "", "English" "#),
            Some(r#"us"intl"#.to_string())
        );
    }

    #[test]
    fn marks_current_word_after_replay_for_next_toggle() {
        let mut buffer = WordBuffer::new();
        for key in [
            KeyCode::KEY_D,
            KeyCode::KEY_H,
            KeyCode::KEY_T,
            KeyCode::KEY_V,
            KeyCode::KEY_Z,
        ] {
            buffer.push(KeyEvent {
                keycode: key.code(),
                shift: false,
                layout_is_ru: false,
            });
        }

        buffer.mark_replayed_layout(1, true);
        let (events, _) = buffer.what_to_replay(1).expect("word is buffered");

        assert!(events.iter().all(|event| event.layout_is_ru));
        assert!(buffer.replay_toggle_ready());
    }

    #[test]
    fn short_fragments_force_replay_without_llm() {
        assert!(should_force_replay_for_short_fragment("N"));
        assert!(should_force_replay_for_short_fragment("gh"));
        assert!(should_force_replay_for_short_fragment("т"));
        assert!(!should_force_replay_for_short_fragment("ghb"));
        assert!(!should_force_replay_for_short_fragment("a b"));
        assert!(!should_force_replay_for_short_fragment(""));
    }

    #[test]
    fn typing_assist_after_space_is_suppressed_once_after_manual_replay() {
        let mut suppress_once = true;

        assert!(!should_schedule_typing_assist_after_space(
            true,
            &mut suppress_once
        ));
        assert!(!suppress_once);
        assert!(should_schedule_typing_assist_after_space(
            true,
            &mut suppress_once
        ));
        assert!(!should_schedule_typing_assist_after_space(
            false,
            &mut suppress_once
        ));
    }

    #[test]
    fn leading_cli_option_token_is_ignored_until_space() {
        for (leader, leader_shift, token_key, next_word) in [
            (KeyCode::KEY_MINUS, false, KeyCode::KEY_B, "feature"),
            (KeyCode::KEY_EQUAL, true, KeyCode::KEY_X, "script"),
        ] {
            let mut modifiers = ShiftState::default();
            modifiers.update(KeyCode::KEY_LEFTSHIFT, i32::from(leader_shift));
            let mut buffer = WordBuffer::new();
            let mut ignore_token =
                should_start_ignored_buffer_token(leader, &modifiers, buffer.current_is_empty());
            assert!(ignore_token);

            if !ignore_token {
                buffer.push(key_event(token_key, false));
            }
            assert!(buffer.current_is_empty());

            if ignore_token {
                ignore_token = false;
            } else {
                buffer.handle_space();
            }
            assert!(!ignore_token);
            assert!(!buffer.prev_had_trailing_space());

            push_text_as_layout(&mut buffer, next_word, false);
            let (events, _) = buffer.what_to_replay(1).expect("word");
            assert_eq!(map_original_events(&events), next_word);
        }
    }

    #[test]
    fn config_replace_words_is_independent_from_engine_mode() {
        let simple = LayConfig {
            mode: "simple".to_string(),
            correction_engine: Some("replay".to_string()),
            replace_words: 2,
            ..LayConfig::default()
        };
        let smart = LayConfig {
            mode: "simple".to_string(),
            correction_engine: Some("smart".to_string()),
            replace_words: 2,
            ..LayConfig::default()
        };

        assert_eq!(simple.active_replace_words(), 2);
        assert_eq!(smart.active_replace_words(), 2);
        assert_eq!(simple.active_correction_engine(), CorrectionEngine::Replay);
        assert_eq!(smart.active_correction_engine(), CorrectionEngine::Smart);
    }

    #[test]
    fn force_layout_hotkeys_use_single_key_ids_only() {
        assert_eq!(
            single_hotkey_keycode("single-rctrl"),
            Some(KeyCode::KEY_RIGHTCTRL)
        );
        assert_eq!(
            single_hotkey_keycode("single-ralt"),
            Some(KeyCode::KEY_RIGHTALT)
        );
        assert_eq!(
            single_hotkey_keycode("caps-lock"),
            Some(KeyCode::KEY_CAPSLOCK)
        );
        assert_eq!(single_hotkey_keycode("double-lshift"), None);
        assert_eq!(single_hotkey_keycode(""), None);
    }

    #[test]
    fn multi_tap_scope_design_contract_maps_taps_to_scope() {
        assert_eq!(multi_tap_scope_for_taps(0), None);
        assert_eq!(multi_tap_scope_for_taps(1), None);
        assert_eq!(multi_tap_scope_for_taps(2), Some(1));
        assert_eq!(multi_tap_scope_for_taps(3), Some(2));
        assert_eq!(multi_tap_scope_for_taps(4), Some(3));
        assert_eq!(multi_tap_scope_for_taps(5), Some(3));
    }

    #[test]
    fn layout_backend_can_be_explicit_or_auto_detected() {
        assert_eq!(
            resolve_layout_backend("gnome", Some("KDE"), None, Some("wayland")),
            LayoutBackend::Gnome
        );
        assert_eq!(
            resolve_layout_backend("kde", Some("GNOME"), None, Some("wayland")),
            LayoutBackend::Kde
        );
        assert_eq!(
            resolve_layout_backend("x11", Some("GNOME"), None, Some("wayland")),
            LayoutBackend::X11
        );
        assert_eq!(
            resolve_layout_backend("auto", Some("KDE"), Some("plasma"), Some("wayland")),
            LayoutBackend::Kde
        );
        assert_eq!(
            resolve_layout_backend("auto", Some("GNOME"), None, Some("wayland")),
            LayoutBackend::Gnome
        );
        assert_eq!(
            resolve_layout_backend("auto", None, None, Some("x11")),
            LayoutBackend::X11
        );
    }

    #[test]
    fn parses_x11_layout_tool_output() {
        assert_eq!(
            parse_setxkbmap_layout("rules: evdev\nmodel: pc105\nlayout: us,ru\n"),
            Some("us".to_string())
        );
        assert_eq!(normalize_layout_id(" ru\n"), "ru");
        assert_eq!(normalize_layout_id("xkb:ru::rus"), "ru");
        assert!(is_ru_layout_id("xkb:ru"));
        assert!(!is_ru_layout_id("xkb:us"));
    }

    #[test]
    fn host_focus_ignore_detects_vm_windows() {
        assert!(focused_window_json_is_ignored(
            r#"{"appId":"org.virt-manager.virt-manager","wmClass":"virt-manager","title":"KDE VM"}"#
        ));
        assert!(focused_window_json_is_ignored(
            r#"{"appId":"remote-viewer.desktop","wmClass":"remote-viewer","title":"SPICE display"}"#
        ));
        assert!(focused_window_json_is_ignored(
            r#"{"appId":"python3","wmClass":"python3","title":"lay-kde-test SPICE clipboard ON"}"#
        ));
        assert!(!focused_window_json_is_ignored(
            r#"{"appId":"org.gnome.Terminal.desktop","wmClass":"org.gnome.Terminal","title":"Terminal"}"#
        ));
    }

    #[test]
    fn keyboard_discovery_ignores_service_virtual_devices() {
        assert!(should_ignore_keyboard_device_name("lay-virtual-keyboard"));
        assert!(should_ignore_keyboard_device_name(
            "ydotoold virtual device"
        ));
        assert!(!should_ignore_keyboard_device_name(
            "AT Translated Set 2 keyboard"
        ));
    }

    #[test]
    fn config_allows_three_word_scope() {
        let cfg = LayConfig {
            replace_words: 3,
            ..LayConfig::default()
        };
        assert_eq!(cfg.active_replace_words(), 3);

        let too_large = LayConfig {
            replace_words: 8,
            ..LayConfig::default()
        };
        assert_eq!(too_large.active_replace_words(), 3);
    }

    #[test]
    fn auto_switch_layout_is_enabled_by_default() {
        assert!(LayConfig::default().auto_switch_layout);
    }

    #[test]
    fn lem_scope_flags_are_enabled_by_default() {
        let cfg = LayConfig::default();
        assert!(!cfg.lem_enabled_for_scope(1));
        assert!(cfg.lem_enabled_for_scope(2));
        assert!(cfg.lem_enabled_for_scope(3));
        assert!(cfg.lem_enabled_for_scope(8));
        assert_eq!(
            cfg.active_typing_assist_pipeline().len(),
            DEFAULT_TYPING_ASSIST_RULES.len()
        );
    }

    #[test]
    fn legacy_llm_mode_maps_to_smart_only_without_explicit_engine() {
        let legacy = LayConfig {
            mode: "llm".to_string(),
            correction_engine: None,
            ..LayConfig::default()
        };
        let explicit_replay = LayConfig {
            mode: "llm".to_string(),
            correction_engine: Some("replay".to_string()),
            ..LayConfig::default()
        };

        assert_eq!(legacy.active_correction_engine(), CorrectionEngine::Smart);
        assert_eq!(
            explicit_replay.active_correction_engine(),
            CorrectionEngine::Replay
        );
    }

    #[test]
    fn two_word_replay_keeps_space_and_backspace_count() {
        let mut buffer = WordBuffer::new();
        push_keys(
            &mut buffer,
            &[
                KeyCode::KEY_G,
                KeyCode::KEY_H,
                KeyCode::KEY_B,
                KeyCode::KEY_D,
                KeyCode::KEY_T,
                KeyCode::KEY_N,
            ],
            false,
        );
        buffer.handle_space();
        push_keys(
            &mut buffer,
            &[KeyCode::KEY_V, KeyCode::KEY_B, KeyCode::KEY_H],
            false,
        );

        let (events, backspaces) = buffer.what_to_replay(2).expect("two words are buffered");

        assert_eq!(map_original_events(&events), "ghbdtn vbh");
        assert_eq!(backspaces, 10);
        assert_eq!(events[6].keycode, KeyCode::KEY_SPACE.code());
        let decision = replay_layout_decision(&events);
        assert_eq!(
            map_target_events(&events, decision.target_is_ru),
            "привет мир"
        );
    }

    #[test]
    fn two_word_trailing_space_replay_deletes_expected_tail() {
        let mut buffer = WordBuffer::new();
        push_keys(&mut buffer, &[KeyCode::KEY_G, KeyCode::KEY_H], false);
        buffer.handle_space();
        push_keys(&mut buffer, &[KeyCode::KEY_V, KeyCode::KEY_B], false);
        buffer.handle_space();

        let (events, backspaces) = buffer.what_to_replay(2).expect("two completed words");

        assert_eq!(map_original_events(&events), "gh vb ");
        assert_eq!(backspaces, 6);
    }

    #[test]
    fn smart_scope_after_trailing_space_keeps_configured_scope() {
        let mut buffer = WordBuffer::new();
        push_keys(
            &mut buffer,
            &[
                KeyCode::KEY_R,
                KeyCode::KEY_J,
                KeyCode::KEY_H,
                KeyCode::KEY_J,
                KeyCode::KEY_X,
                KeyCode::KEY_T,
            ],
            true,
        );
        buffer.handle_space();
        push_keys(
            &mut buffer,
            &[KeyCode::KEY_N, KeyCode::KEY_F, KeyCode::KEY_V],
            true,
        );
        buffer.handle_space();

        let scope = effective_replace_words(&buffer, 2, CorrectionEngine::Smart, true);
        let (events, backspaces) = buffer.what_to_replay(scope).expect("last word is buffered");

        assert_eq!(scope, 2);
        assert_eq!(map_original_events(&events), "короче там ");
        assert_eq!(backspaces, 11);
    }

    #[test]
    fn replay_layout_decision_ignores_inserted_space() {
        let events = [
            key_event(KeyCode::KEY_G, true),
            key_event(KeyCode::KEY_H, true),
            key_event(KeyCode::KEY_SPACE, false),
            key_event(KeyCode::KEY_V, true),
            key_event(KeyCode::KEY_B, true),
        ];

        assert!(!is_layout_decision_key(KeyCode::KEY_SPACE));
        assert_eq!(
            replay_layout_decision(&events),
            ReplayLayoutDecision {
                target_is_ru: false,
                mixed_layouts: false,
            }
        );
    }

    #[test]
    fn shortcut_modified_text_keys_do_not_enter_word_buffer() {
        let mut modifiers = ShiftState::default();

        modifiers.update(KeyCode::KEY_LEFTCTRL, 1);
        assert!(should_ignore_buffer_key(
            KeyCode::KEY_EQUAL,
            &modifiers,
            true
        ));
        assert!(should_ignore_buffer_key(
            KeyCode::KEY_MINUS,
            &modifiers,
            true
        ));
        assert!(should_ignore_buffer_key(
            KeyCode::KEY_SPACE,
            &modifiers,
            true
        ));
        assert!(should_ignore_buffer_key(KeyCode::KEY_A, &modifiers, true));

        modifiers.update(KeyCode::KEY_LEFTCTRL, 0);
        assert!(!should_ignore_buffer_key(KeyCode::KEY_A, &modifiers, true));
    }

    #[test]
    fn leading_plus_minus_symbols_do_not_attach_to_next_word() {
        let mut buffer = WordBuffer::new();

        for (key, shift) in [
            (KeyCode::KEY_EQUAL, true),
            (KeyCode::KEY_EQUAL, false),
            (KeyCode::KEY_MINUS, true),
            (KeyCode::KEY_MINUS, false),
        ] {
            let mut modifiers = ShiftState::default();
            modifiers.update(KeyCode::KEY_LEFTSHIFT, i32::from(shift));
            if !should_ignore_buffer_key(key, &modifiers, buffer.current_is_empty()) {
                buffer.push(key_event(key, shift));
            }
        }
        push_text_as_layout(&mut buffer, "есть", true);

        let (events, backspaces) = buffer.what_to_replay(1).expect("word tail");

        assert_eq!(map_original_events(&events), "есть");
        assert_eq!(backspaces, 4);
    }

    #[test]
    fn visual_latin_word_with_cyrillic_c_homoglyph_replays_to_ru() {
        let events = [
            key_event(KeyCode::KEY_C, true),
            key_event(KeyCode::KEY_H, false),
            key_event(KeyCode::KEY_E, false),
            key_event(KeyCode::KEY_C, false),
        ];

        assert_eq!(map_original_events(&events), "сhec");
        assert_eq!(
            replay_layout_decision(&events),
            ReplayLayoutDecision {
                target_is_ru: true,
                mixed_layouts: true,
            }
        );
        assert_eq!(map_target_events(&events, true), "срус");
    }

    #[test]
    fn smart_decision_keeps_good_word_and_converts_bad_neighbor() {
        assert_eq!(
            decide_correction("Главное Вщгиду", "Ukfdyjt Double", CorrectionEngine::Smart),
            Correction::InsertText("Главное Double".to_string())
        );
    }

    #[test]
    fn scoped_tail_keeps_good_previous_word_and_flips_current_fragment() {
        let mut buffer = WordBuffer::new();
        push_keys(&mut buffer, &[KeyCode::KEY_D], true);
        buffer.handle_space();
        push_key_events(
            &mut buffer,
            &[
                (KeyCode::KEY_D, true),
                (KeyCode::KEY_O, false),
                (KeyCode::KEY_U, false),
                (KeyCode::KEY_B, false),
                (KeyCode::KEY_L, false),
                (KeyCode::KEY_E, false),
            ],
            true,
        );
        let (events, _) = buffer.what_to_replay(2).expect("two-word tail");

        assert_eq!(map_original_events(&events), "в Вщгиду");
        assert_eq!(
            decide_scoped_tail_correction(&events),
            Some("в Double".to_string())
        );
        assert_eq!(
            plan_text_replacement("в Вщгиду", "в Double"),
            Some(TextReplacement {
                move_left: 0,
                backspaces: 6,
                insert: "Double".to_string(),
                move_right: 0,
            })
        );
    }

    #[test]
    fn smart_insert_remembers_only_inserted_tail_for_immediate_undo() {
        let mut buffer = WordBuffer::new();
        push_key_events(
            &mut buffer,
            &[
                (KeyCode::KEY_G, true),
                (KeyCode::KEY_H, false),
                (KeyCode::KEY_J, false),
                (KeyCode::KEY_D, false),
                (KeyCode::KEY_T, false),
                (KeyCode::KEY_H, false),
                (KeyCode::KEY_R, false),
                (KeyCode::KEY_F, false),
            ],
            true,
        );
        buffer.handle_space();
        push_keys(
            &mut buffer,
            &[
                KeyCode::KEY_C,
                KeyCode::KEY_K,
                KeyCode::KEY_J,
                KeyCode::KEY_D,
                KeyCode::KEY_F,
            ],
            true,
        );
        let (events, _) = buffer.what_to_replay(2).expect("two-word tail");
        let original = map_original_events(&events);
        let replacement = decide_scoped_tail_correction(&events).expect("smart replacement");
        let plan = plan_text_replacement(&original, &replacement).expect("minimal plan");

        assert_eq!(original, "Проверка слова");
        assert_eq!(replacement, "Проверка ckjdf");
        assert_eq!(
            plan,
            TextReplacement {
                move_left: 0,
                backspaces: 5,
                insert: "ckjdf".to_string(),
                move_right: 0,
            }
        );
        assert!(buffer.remember_inserted_tail_for_replay(&events, &plan, false));

        let (undo_events, undo_backspaces) = buffer.what_to_replay(2).expect("undo tail");
        let undo_decision = replay_layout_decision(&undo_events);
        assert_eq!(map_original_events(&undo_events), "ckjdf");
        assert_eq!(undo_backspaces, 5);
        assert!(undo_decision.target_is_ru);
        assert_eq!(map_events_to_layout(&undo_events, true), "слова");
        assert!(buffer.replay_toggle_ready());
    }

    #[test]
    fn smart_insert_remembers_last_word_after_full_tail_replace() {
        let mut buffer = WordBuffer::new();
        push_keys(
            &mut buffer,
            &[
                KeyCode::KEY_G,
                KeyCode::KEY_O,
                KeyCode::KEY_O,
                KeyCode::KEY_D,
            ],
            true,
        );
        buffer.handle_space();
        push_keys(
            &mut buffer,
            &[
                KeyCode::KEY_N,
                KeyCode::KEY_T,
                KeyCode::KEY_R,
                KeyCode::KEY_C,
                KeyCode::KEY_N,
            ],
            true,
        );
        let (events, _) = buffer.what_to_replay(2).expect("two-word tail");
        let original = map_original_events(&events);
        let replacement = decide_scoped_tail_correction(&events).expect("smart replacement");
        let plan = plan_text_replacement(&original, &replacement).expect("minimal plan");

        assert_eq!(original, "пщщв текст");
        assert_eq!(replacement, "good ntrcn");
        assert_eq!(
            plan,
            TextReplacement {
                move_left: 0,
                backspaces: 10,
                insert: "good ntrcn".to_string(),
                move_right: 0,
            }
        );
        assert!(!buffer.remember_inserted_tail_for_replay(&events, &plan, false));
        assert!(buffer.remember_inserted_last_word_for_replay(&events, &plan));

        let (undo_events, undo_backspaces) = buffer.what_to_replay(2).expect("undo tail");
        let undo_decision = replay_layout_decision(&undo_events);
        assert_eq!(map_original_events(&undo_events), "ntrcn");
        assert_eq!(undo_backspaces, 5);
        assert!(undo_decision.target_is_ru);
        assert_eq!(map_events_to_layout(&undo_events, true), "текст");
        assert!(buffer.replay_toggle_ready());
    }

    #[test]
    fn scoped_tail_keeps_good_english_previous_word_and_flips_current_layout_word() {
        let mut buffer = WordBuffer::new();
        push_keys(
            &mut buffer,
            &[
                KeyCode::KEY_G,
                KeyCode::KEY_O,
                KeyCode::KEY_O,
                KeyCode::KEY_D,
            ],
            false,
        );
        buffer.handle_space();
        push_keys(
            &mut buffer,
            &[
                KeyCode::KEY_N,
                KeyCode::KEY_T,
                KeyCode::KEY_R,
                KeyCode::KEY_C,
                KeyCode::KEY_N,
            ],
            false,
        );
        let (events, _) = buffer.what_to_replay(2).expect("two-word tail");

        assert_eq!(map_original_events(&events), "good ntrcn");
        assert_eq!(
            decide_scoped_tail_correction(&events),
            Some("good текст".to_string())
        );
    }

    #[test]
    fn scoped_tail_keeps_completed_ascii_title_word_and_flips_current_latin_keys() {
        let mut buffer = WordBuffer::new();
        let left_events = [
            KeyEvent {
                keycode: KeyCode::KEY_D.code(),
                shift: true,
                layout_is_ru: false,
            },
            key_event(KeyCode::KEY_O, false),
            key_event(KeyCode::KEY_U, false),
            key_event(KeyCode::KEY_B, false),
            key_event(KeyCode::KEY_L, false),
            key_event(KeyCode::KEY_E, false),
        ];
        for event in left_events {
            buffer.push(event);
        }
        buffer.handle_space();
        let current_events = [
            key_event(KeyCode::KEY_N, false),
            key_event(KeyCode::KEY_J, false),
            key_event(KeyCode::KEY_SEMICOLON, false),
            key_event(KeyCode::KEY_T, false),
        ];
        for event in current_events {
            buffer.push(event);
        }
        let (events, _) = buffer.what_to_replay(2).expect("two-word tail");
        let left = map_original_events(&left_events);
        let current_original = map_original_events(&current_events);
        let current_target = map_events_to_layout(&current_events, true);

        assert_eq!(
            map_original_events(&events),
            format!("{left} {current_original}")
        );
        assert_eq!(
            decide_scoped_tail_correction(&events),
            Some(format!("{left} {current_target}"))
        );
        assert_eq!(
            plan_text_replacement(
                &format!("{left} {current_original}"),
                &format!("{left} {current_target}")
            ),
            Some(TextReplacement {
                move_left: 0,
                backspaces: current_original.chars().count() as u32,
                insert: current_target,
                move_right: 0,
            })
        );
    }

    #[test]
    fn scoped_tail_keeps_completed_russian_y_word_and_flips_current_latin_brand() {
        let mut buffer = WordBuffer::new();
        push_text_as_layout(&mut buffer, "протокол", true);
        buffer.handle_space();
        push_text_as_layout(&mut buffer, "испытаний", true);
        buffer.handle_space();
        push_text_as_layout(&mut buffer, "Сщсф", true);
        let (events, _) = buffer.what_to_replay(3).expect("three-word tail");
        let original = map_original_events(&events);
        let replacement = decide_scoped_tail_correction(&events).expect("smart replacement");

        assert_eq!(original, "протокол испытаний Сщсф");
        assert_eq!(replacement, "протокол испытаний Coca");
        assert_eq!(
            plan_text_replacement(&original, &replacement),
            Some(TextReplacement {
                move_left: 0,
                backspaces: 4,
                insert: "Coca".to_string(),
                move_right: 0,
            })
        );
    }

    #[test]
    fn scoped_tail_repairs_stale_layout_flag_inside_completed_russian_word() {
        let mut buffer = WordBuffer::new();
        push_text_as_layout(&mut buffer, "протокол", true);
        buffer.handle_space();
        push_text_as_layout(&mut buffer, "испытани", true);
        buffer.push(KeyEvent {
            keycode: KeyCode::KEY_Q.code(),
            shift: false,
            layout_is_ru: false,
        });
        buffer.handle_space();
        push_text_as_layout(&mut buffer, "Сщсф", true);
        let (events, _) = buffer.what_to_replay(3).expect("three-word tail");
        let original = map_original_events(&events);
        let replacement = decide_scoped_tail_correction(&events).expect("smart replacement");

        assert_eq!(original, "протокол испытаниq Сщсф");
        assert_eq!(replacement, "протокол испытаний Coca");
        assert_eq!(
            plan_text_replacement(&original, &replacement),
            Some(TextReplacement {
                move_left: 0,
                backspaces: 6,
                insert: "й Coca".to_string(),
                move_right: 0,
            })
        );
    }

    #[test]
    fn scoped_tail_keeps_single_completed_cyrillic_fragment_before_current_word() {
        let mut buffer = WordBuffer::new();
        push_text_as_layout(&mut buffer, "й", true);
        buffer.handle_space();
        push_text_as_layout(&mut buffer, "Сщсф", true);
        let (events, _) = buffer.what_to_replay(3).expect("two-word tail");
        let original = map_original_events(&events);
        let replacement = decide_scoped_tail_correction_with_lem(&events, true)
            .or_else(|| decide_scoped_tail_correction(&events))
            .expect("smart replacement");

        assert_eq!(original, "й Сщсф");
        assert_eq!(replacement, "й Coca");
        assert_eq!(
            plan_text_replacement(&original, &replacement),
            Some(TextReplacement {
                move_left: 0,
                backspaces: 4,
                insert: "Coca".to_string(),
                move_right: 0,
            })
        );
    }

    #[test]
    fn scoped_tail_trailing_space_keeps_previous_good_word_and_flips_current_completed_latin_keys()
    {
        let mut buffer = WordBuffer::new();
        let left_events = [
            KeyEvent {
                keycode: KeyCode::KEY_D.code(),
                shift: true,
                layout_is_ru: false,
            },
            key_event(KeyCode::KEY_O, false),
            key_event(KeyCode::KEY_U, false),
            key_event(KeyCode::KEY_B, false),
            key_event(KeyCode::KEY_L, false),
            key_event(KeyCode::KEY_E, false),
        ];
        for event in left_events {
            buffer.push(event);
        }
        buffer.handle_space();
        let current_events = [
            key_event(KeyCode::KEY_N, false),
            key_event(KeyCode::KEY_J, false),
            key_event(KeyCode::KEY_SEMICOLON, false),
            key_event(KeyCode::KEY_T, false),
        ];
        for event in current_events {
            buffer.push(event);
        }
        buffer.handle_space();

        let scope = effective_replace_words(&buffer, 2, CorrectionEngine::Smart, true);
        let (events, backspaces) = buffer.what_to_replay(scope).expect("last word tail");
        let left = map_original_events(&left_events);
        let current_original = map_original_events(&current_events);
        let current_target = map_events_to_layout(&current_events, true);

        assert_eq!(scope, 2);
        assert_eq!(
            map_original_events(&events),
            format!("{left} {current_original} ")
        );
        assert_eq!(
            decide_scoped_tail_correction(&events),
            Some(format!("{left} {current_target} "))
        );
        assert_eq!(
            backspaces,
            (left.chars().count() + 1 + current_original.chars().count() + 1) as u32
        );
    }

    #[test]
    fn scoped_tail_trailing_space_keeps_previous_russian_word_and_flips_completed_tail() {
        let mut buffer = WordBuffer::new();
        push_text_as_layout(&mut buffer, "открывал", true);
        buffer.handle_space();
        push_text_as_layout(&mut buffer, "цзы", true);
        buffer.handle_space();

        let (events, _) = buffer.what_to_replay(2).expect("two-word tail");
        let original = map_original_events(&events);
        let replacement =
            decide_scoped_tail_correction_with_lem(&events, true).expect("smart replacement");

        assert_eq!(original, "открывал цзы ");
        assert_eq!(replacement, "открывал wps ");
        assert_eq!(
            plan_text_replacement(&original, &replacement),
            Some(TextReplacement {
                move_left: 1,
                backspaces: 3,
                insert: "wps".to_string(),
                move_right: 1,
            })
        );
    }

    #[test]
    fn scoped_tail_flips_cyrillic_hyphen_technical_token_to_ascii() {
        let mut buffer = WordBuffer::new();
        let left_events = [
            key_event(KeyCode::KEY_C, true),
            key_event(KeyCode::KEY_K, true),
            key_event(KeyCode::KEY_J, true),
            key_event(KeyCode::KEY_D, true),
            key_event(KeyCode::KEY_J, true),
        ];
        for event in left_events {
            buffer.push(event);
        }
        buffer.handle_space();
        let technical_events = [
            key_event(KeyCode::KEY_W, true),
            key_event(KeyCode::KEY_I, true),
            key_event(KeyCode::KEY_MINUS, true),
            key_event(KeyCode::KEY_F, true),
            key_event(KeyCode::KEY_I, true),
        ];
        for event in technical_events {
            buffer.push(event);
        }
        let (events, _) = buffer.what_to_replay(2).expect("two-word tail");
        let left = map_events_to_layout(&left_events, true);
        let typed_technical = map_events_to_layout(&technical_events, true);
        let target_technical = map_events_to_layout(&technical_events, false);

        assert_eq!(
            map_original_events(&events),
            format!("{left} {typed_technical}")
        );
        assert_eq!(
            decide_scoped_tail_correction(&events),
            Some(format!("{left} {target_technical}"))
        );
    }

    #[test]
    fn scoped_tail_keeps_unknown_previous_word_and_flips_cyrillic_hyphen_technical_token() {
        let mut buffer = WordBuffer::new();
        let left_events = [
            KeyEvent {
                keycode: KeyCode::KEY_SEMICOLON.code(),
                shift: true,
                layout_is_ru: true,
            },
            key_event(KeyCode::KEY_SEMICOLON, true),
            key_event(KeyCode::KEY_SEMICOLON, true),
            key_event(KeyCode::KEY_SEMICOLON, true),
        ];
        for event in left_events {
            buffer.push(event);
        }
        buffer.handle_space();
        let technical_events = [
            key_event(KeyCode::KEY_W, true),
            key_event(KeyCode::KEY_I, true),
            key_event(KeyCode::KEY_MINUS, true),
            key_event(KeyCode::KEY_F, true),
            key_event(KeyCode::KEY_I, true),
        ];
        for event in technical_events {
            buffer.push(event);
        }
        let (events, _) = buffer.what_to_replay(2).expect("two-word tail");
        let left = map_events_to_layout(&left_events, true);
        let typed_technical = map_events_to_layout(&technical_events, true);
        let target_technical = map_events_to_layout(&technical_events, false);

        assert_eq!(
            map_original_events(&events),
            format!("{left} {typed_technical}")
        );
        assert_eq!(
            decide_scoped_tail_correction(&events),
            Some(format!("{left} {target_technical}"))
        );
    }

    #[test]
    fn typing_assist_converts_wrong_layout_ascii_hyphen_token() {
        let technical_events = [
            key_event(KeyCode::KEY_W, true),
            key_event(KeyCode::KEY_I, true),
            key_event(KeyCode::KEY_MINUS, true),
            key_event(KeyCode::KEY_F, true),
            key_event(KeyCode::KEY_I, true),
        ];
        let typed_technical = map_events_to_layout(&technical_events, true);
        let target_technical = map_events_to_layout(&technical_events, false);
        assert_eq!(
            apply_typing_assist_exact(&format!("{typed_technical} ")),
            Some(format!("{target_technical} "))
        );
    }

    #[test]
    fn typing_assist_keeps_natural_cyrillic_hyphen_words() {
        assert_eq!(apply_typing_assist("что-то ", true), None);
        assert_eq!(apply_typing_assist("кто-то ", true), None);
        assert_eq!(apply_typing_assist("где-то ", true), None);
        assert_eq!(apply_typing_assist("как-то ", true), None);
        assert_eq!(apply_typing_assist("из-за ", true), None);
        assert_eq!(apply_typing_assist("кока-коле ", true), None);
        assert_eq!(apply_typing_assist("код-дэ-вуар ", true), None);
        assert_eq!(correct_wrong_layout_ascii_technical_token("из-за"), None);
        assert_eq!(
            correct_wrong_layout_ascii_technical_token("цш-аш"),
            Some("wi-fi".to_string())
        );
        assert_eq!(correct_wrong_layout_ascii_technical_token("15р-16р"), None);
        assert_eq!(apply_typing_assist("15р-16р ", true), None);
    }

    #[test]
    fn plain_cyrillic_scope_word_does_not_become_ascii_technical_noise() {
        let events = [
            key_event(KeyCode::KEY_A, true),
            key_event(KeyCode::KEY_Q, true),
            key_event(KeyCode::KEY_DOT, true),
            key_event(KeyCode::KEY_Z, true),
        ];
        let original = map_events_to_layout(&events, true);
        let converted = map_events_to_layout(&events, false);

        assert!(original.chars().all(is_cyrillic_letter));
        assert!(is_ascii_technical_token(&converted));
        assert!(should_keep_plain_cyrillic_before_ascii_technical(
            &original, &converted
        ));
        assert_eq!(decide_completed_scope_word(&events), original);
    }

    #[test]
    fn smart_scoped_tail_handles_large_mixed_language_pair_matrix() {
        let english_left = [
            "good", "test", "word", "live", "double", "text", "mode", "file", "code", "data",
        ];
        let russian_left = [
            "привет",
            "текст",
            "слово",
            "тест",
            "проверка",
            "можно",
            "нужно",
            "дальше",
            "хорошо",
            "пример",
        ];
        let russian_targets = [
            "привет",
            "текст",
            "слово",
            "тест",
            "проверка",
            "можно",
            "нужно",
            "дальше",
            "хорошо",
            "пример",
        ];
        let english_targets = [
            "good", "test", "word", "live", "double", "text", "mode", "file", "code", "data",
        ];

        let mut cases = 0;
        for left in english_left {
            for target in russian_targets {
                let typed = lay::dict::convert(target, lay::dict::Direction::Ru2Us);
                assert_smart_pair(left, false, &typed, false, &format!("{left} {target}"));
                cases += 1;
            }
        }

        for left in russian_left {
            for target in english_targets {
                let typed = lay::dict::convert(target, lay::dict::Direction::Us2Ru);
                assert_smart_pair(left, true, &typed, true, &format!("{left} {target}"));
                cases += 1;
            }
        }

        assert!(cases >= 100, "expected at least 100 mixed pair cases");
    }

    #[test]
    fn scoped_tail_flips_current_visual_latin_word_with_cyrillic_c_homoglyph() {
        let mut buffer = WordBuffer::new();
        push_key_events(
            &mut buffer,
            &[
                (KeyCode::KEY_C, false),
                (KeyCode::KEY_H, false),
                (KeyCode::KEY_E, false),
                (KeyCode::KEY_C, false),
                (KeyCode::KEY_K, false),
            ],
            false,
        );
        buffer.handle_space();
        buffer.push(key_event(KeyCode::KEY_C, true));
        buffer.push(key_event(KeyCode::KEY_H, false));
        buffer.push(key_event(KeyCode::KEY_E, false));
        buffer.push(key_event(KeyCode::KEY_C, false));
        let (events, _) = buffer.what_to_replay(2).expect("two-word tail");

        assert_eq!(map_original_events(&events), "check сhec");
        assert_eq!(
            decide_scoped_tail_correction(&events),
            Some("check срус".to_string())
        );
    }

    #[test]
    fn scoped_tail_removes_duplicate_layout_prefix_from_completed_ascii_technical_token() {
        let mut buffer = WordBuffer::new();
        let mut completed_events = vec![key_event(KeyCode::KEY_W, true)];
        completed_events.extend(key_events(&ascii_hyphen_token_keycodes(), false));
        for event in &completed_events {
            buffer.push(*event);
        }
        buffer.handle_space();
        let current_events = key_events(&[KeyCode::KEY_G, KeyCode::KEY_H, KeyCode::KEY_J], false);
        for event in &current_events {
            buffer.push(*event);
        }
        let (events, _) = buffer.what_to_replay(2).expect("two-word tail");
        let completed_original = map_original_events(&completed_events);
        let current_original = map_original_events(&current_events);
        let completed_repaired =
            correct_duplicate_layout_prefix_on_ascii_token(&completed_original)
                .expect("duplicate prefix repair");
        let current_target = map_events_to_layout(&current_events, true);

        assert_eq!(
            map_original_events(&events),
            format!("{completed_original} {current_original}")
        );
        assert_eq!(
            decide_scoped_tail_correction(&events),
            Some(format!("{completed_repaired} {current_target}"))
        );
    }

    #[test]
    fn scoped_tail_keeps_ascii_hyphen_word_and_flips_current_short_tail() {
        let mut buffer = WordBuffer::new();
        let completed_events = key_events(&ascii_hyphen_token_keycodes(), false);
        for event in &completed_events {
            buffer.push(*event);
        }
        buffer.handle_space();
        let current_events = key_events(&[KeyCode::KEY_Y, KeyCode::KEY_E], false);
        for event in &current_events {
            buffer.push(*event);
        }
        let (events, _) = buffer.what_to_replay(2).expect("two-word tail");
        let completed_original = map_original_events(&completed_events);
        let current_original = map_original_events(&current_events);
        let current_target = map_events_to_layout(&current_events, true);

        assert_eq!(
            map_original_events(&events),
            format!("{completed_original} {current_original}")
        );
        assert_eq!(
            decide_scoped_tail_correction(&events),
            Some(format!("{completed_original} {current_target}"))
        );
        assert_eq!(
            plan_text_replacement(
                &format!("{completed_original} {current_original}"),
                &format!("{completed_original} {current_target}")
            ),
            Some(TextReplacement {
                move_left: 0,
                backspaces: current_original.chars().count() as u32,
                insert: current_target,
                move_right: 0,
            })
        );
    }

    #[test]
    fn trailing_space_scope_keeps_ascii_hyphen_word_and_flips_last_short_word() {
        let mut buffer = WordBuffer::new();
        let completed_events = key_events(&ascii_hyphen_token_keycodes(), false);
        for event in &completed_events {
            buffer.push(*event);
        }
        buffer.handle_space();
        let current_events = key_events(&[KeyCode::KEY_Y, KeyCode::KEY_E], false);
        for event in &current_events {
            buffer.push(*event);
        }
        buffer.handle_space();

        let scope = effective_replace_words(&buffer, 2, CorrectionEngine::Smart, true);
        let (events, backspaces) = buffer.what_to_replay(scope).expect("last word tail");
        let left = map_original_events(&completed_events);
        let current_original = map_original_events(&current_events);
        let current_target = map_events_to_layout(&current_events, true);

        assert_eq!(scope, 2);
        assert_eq!(
            map_original_events(&events),
            format!("{left} {current_original} ")
        );
        assert_eq!(
            decide_scoped_tail_correction(&events),
            Some(format!("{left} {current_target} "))
        );
        assert_eq!(
            backspaces,
            (left.chars().count() + 1 + current_original.chars().count() + 1) as u32
        );
    }

    #[test]
    fn scoped_tail_collapses_cyrillic_prefix_before_ascii_hyphen_tail() {
        let mut buffer = WordBuffer::new();
        push_keys(
            &mut buffer,
            &[
                KeyCode::KEY_C,
                KeyCode::KEY_K,
                KeyCode::KEY_J,
                KeyCode::KEY_D,
                KeyCode::KEY_J,
            ],
            true,
        );
        buffer.handle_space();
        let mut current_events = vec![key_event(KeyCode::KEY_G, true)];
        current_events.extend(key_events(
            &[
                KeyCode::KEY_G,
                KeyCode::KEY_F,
                KeyCode::KEY_H,
                KeyCode::KEY_F,
                KeyCode::KEY_MINUS,
                KeyCode::KEY_G,
                KeyCode::KEY_F,
                KeyCode::KEY_H,
                KeyCode::KEY_F,
            ],
            false,
        ));
        for event in &current_events {
            buffer.push(*event);
        }
        let (events, _) = buffer.what_to_replay(2).expect("two-word tail");
        let left = map_events_to_layout(
            &[
                key_event(KeyCode::KEY_C, true),
                key_event(KeyCode::KEY_K, true),
                key_event(KeyCode::KEY_J, true),
                key_event(KeyCode::KEY_D, true),
                key_event(KeyCode::KEY_J, true),
            ],
            true,
        );
        let current_original = map_original_events(&current_events);
        let current_target = repair_cyrillic_prefix_before_ascii_tail(&current_events)
            .expect("prefix collapse repair");

        assert_eq!(
            map_original_events(&events),
            format!("{left} {current_original}")
        );
        assert_eq!(
            decide_scoped_tail_correction(&events),
            Some(format!("{left} {current_target}"))
        );
    }

    #[test]
    fn scoped_tail_repairs_mixed_cyrillic_prefix_ascii_hyphen_word_and_keeps_undo() {
        let mut buffer = WordBuffer::new();
        push_text_as_layout(&mut buffer, "Иракскую", true);
        buffer.handle_space();
        buffer.push(text_key_event('к', true));
        for ch in "jrf-rjke".chars() {
            buffer.push(text_key_event(ch, false));
        }

        let (events, _) = buffer.what_to_replay(2).expect("two-word tail");
        let original = map_original_events(&events);
        let replacement = decide_scoped_tail_correction(&events).expect("smart replacement");
        let plan = plan_text_replacement(&original, &replacement).expect("minimal plan");

        assert_eq!(original, "Иракскую кjrf-rjke");
        assert_eq!(replacement, "Иракскую кока-колу");
        assert_eq!(
            plan,
            TextReplacement {
                move_left: 0,
                backspaces: 8,
                insert: "ока-колу".to_string(),
                move_right: 0,
            }
        );
        assert!(buffer.remember_replacement_last_word_for_replay(&events, &plan, &replacement));

        let (undo_events, undo_backspaces) = buffer.what_to_replay(2).expect("undo tail");
        let undo_decision = replay_layout_decision(&undo_events);
        assert_eq!(map_original_events(&undo_events), "кока-колу");
        assert_eq!(undo_backspaces, 9);
        assert!(!undo_decision.target_is_ru);
        assert_eq!(map_events_to_layout(&undo_events, false), "rjrf-rjke");
        assert!(buffer.replay_toggle_ready());
    }

    #[test]
    fn scoped_tail_repairs_mixed_cyrillic_prefix_ascii_hyphen_dative_word() {
        let mut buffer = WordBuffer::new();
        push_text_as_layout(&mut buffer, "Иракскую", true);
        buffer.handle_space();
        buffer.push(text_key_event('к', true));
        for ch in "jrf-rjkt".chars() {
            buffer.push(text_key_event(ch, false));
        }

        let (events, _) = buffer.what_to_replay(2).expect("two-word tail");
        let original = map_original_events(&events);
        let replacement = decide_scoped_tail_correction(&events).expect("smart replacement");
        let plan = plan_text_replacement(&original, &replacement).expect("minimal plan");

        assert_eq!(original, "Иракскую кjrf-rjkt");
        assert_eq!(replacement, "Иракскую кока-коле");
        assert_eq!(
            plan,
            TextReplacement {
                move_left: 0,
                backspaces: 8,
                insert: "ока-коле".to_string(),
                move_right: 0,
            }
        );
        assert!(buffer.remember_replacement_last_word_for_replay(&events, &plan, &replacement));

        let (undo_events, undo_backspaces) = buffer.what_to_replay(2).expect("undo tail");
        let undo_decision = replay_layout_decision(&undo_events);
        assert_eq!(map_original_events(&undo_events), "кока-коле");
        assert_eq!(undo_backspaces, 9);
        assert!(!undo_decision.target_is_ru);
        assert_eq!(map_events_to_layout(&undo_events, false), "rjrf-rjkt");
        assert!(buffer.replay_toggle_ready());
    }

    #[test]
    fn replacement_last_word_memory_ignores_middle_insert_plan() {
        let mut buffer = WordBuffer::new();
        push_text_as_layout(&mut buffer, "AmoCRM", false);
        buffer.handle_space();
        push_text_as_layout(&mut buffer, "Z", false);
        buffer.handle_space();
        push_text_as_layout(&mut buffer, "тут", true);
        buffer.handle_space();
        push_text_as_layout(&mut buffer, "задача", true);

        let (events, _) = buffer.what_to_replay(4).expect("four-word tail");
        let plan = plan_text_replacement("AmoCRM Z тут задача", "AmoCRM Я тут задача")
            .expect("middle replacement plan");

        assert_eq!(plan.move_right, 11);
        assert!(!buffer.remember_replacement_last_word_for_replay(
            &events,
            &plan,
            "AmoCRM Я тут задача"
        ));
    }

    #[test]
    fn scoped_tail_does_not_turn_valid_ascii_hyphen_tail_into_bad_russian() {
        let mut buffer = WordBuffer::new();
        push_keys(&mut buffer, &[KeyCode::KEY_D], true);
        buffer.handle_space();
        let mut current_events = vec![key_event(KeyCode::KEY_W, true)];
        current_events.extend([
            KeyEvent {
                keycode: KeyCode::KEY_W.code(),
                shift: true,
                layout_is_ru: false,
            },
            key_event(KeyCode::KEY_I, false),
            key_event(KeyCode::KEY_MINUS, false),
            key_event(KeyCode::KEY_F, false),
            key_event(KeyCode::KEY_I, false),
        ]);
        for event in &current_events {
            buffer.push(*event);
        }
        let (events, _) = buffer.what_to_replay(2).expect("two-word tail");
        let left = map_events_to_layout(&[key_event(KeyCode::KEY_D, true)], true);
        let current_original = map_original_events(&current_events);
        let current_wrong_layout = map_events_to_layout(&current_events, true);

        assert_eq!(
            map_original_events(&events),
            format!("{left} {current_original}")
        );
        assert_ne!(
            decide_scoped_tail_correction(&events),
            Some(format!("{left} {current_wrong_layout}"))
        );
    }

    #[test]
    fn scoped_tail_converts_confident_bad_previous_word() {
        let mut buffer = WordBuffer::new();
        push_keys(
            &mut buffer,
            &[
                KeyCode::KEY_G,
                KeyCode::KEY_H,
                KeyCode::KEY_B,
                KeyCode::KEY_D,
                KeyCode::KEY_T,
                KeyCode::KEY_N,
            ],
            false,
        );
        buffer.handle_space();
        push_keys(
            &mut buffer,
            &[KeyCode::KEY_V, KeyCode::KEY_B, KeyCode::KEY_H],
            false,
        );
        let (events, _) = buffer.what_to_replay(2).expect("two-word tail");

        assert_eq!(
            decide_scoped_tail_correction(&events),
            Some("привет мир".to_string())
        );
    }

    #[test]
    fn scoped_tail_keeps_unknown_previous_word() {
        let mut buffer = WordBuffer::new();
        push_keys(
            &mut buffer,
            &[
                KeyCode::KEY_F,
                KeyCode::KEY_O,
                KeyCode::KEY_O,
                KeyCode::KEY_B,
                KeyCode::KEY_A,
                KeyCode::KEY_R,
            ],
            false,
        );
        buffer.handle_space();
        push_keys(
            &mut buffer,
            &[KeyCode::KEY_G, KeyCode::KEY_H, KeyCode::KEY_J],
            false,
        );
        let (events, _) = buffer.what_to_replay(2).expect("two-word tail");

        assert_eq!(
            decide_scoped_tail_correction(&events),
            Some("foobar про".to_string())
        );
    }

    #[test]
    fn scoped_tail_generalizes_to_more_than_two_words() {
        let mut buffer = WordBuffer::new();
        push_keys(
            &mut buffer,
            &[
                KeyCode::KEY_G,
                KeyCode::KEY_H,
                KeyCode::KEY_J,
                KeyCode::KEY_D,
                KeyCode::KEY_T,
                KeyCode::KEY_H,
                KeyCode::KEY_R,
                KeyCode::KEY_F,
            ],
            true,
        );
        buffer.handle_space();
        push_keys(&mut buffer, &[KeyCode::KEY_D], true);
        buffer.handle_space();
        push_key_events(
            &mut buffer,
            &[
                (KeyCode::KEY_D, true),
                (KeyCode::KEY_O, false),
                (KeyCode::KEY_U, false),
                (KeyCode::KEY_B, false),
                (KeyCode::KEY_L, false),
                (KeyCode::KEY_E, false),
            ],
            true,
        );
        let (events, _) = buffer.what_to_replay(3).expect("three-word tail");

        assert_eq!(map_original_events(&events), "проверка в Вщгиду");
        assert_eq!(
            decide_scoped_tail_correction(&events),
            Some("проверка в Double".to_string())
        );
    }

    #[test]
    fn scoped_tail_uses_lem_for_three_word_mixed_tail() {
        let mut buffer = WordBuffer::new();
        push_text_as_layout(&mut buffer, "good", false);
        buffer.handle_space();
        for ch in "ghbdtn".chars() {
            buffer.push(text_key_event(ch, false));
        }
        buffer.handle_space();
        for ch in "ntrcn".chars() {
            buffer.push(text_key_event(ch, false));
        }

        let (events, _) = buffer.what_to_replay(3).expect("three-word tail");
        assert_eq!(map_original_events(&events), "good ghbdtn ntrcn");
        assert_eq!(
            decide_scoped_tail_correction(&events),
            Some("good привет текст".to_string())
        );
    }

    #[test]
    fn scoped_tail_keeps_short_repeated_completed_word_and_flips_current_tail() {
        let mut buffer = WordBuffer::new();
        push_text_as_layout(&mut buffer, "аа", true);
        buffer.handle_space();
        push_text_as_layout(&mut buffer, "слово", true);
        buffer.handle_space();
        push_text_as_layout(&mut buffer, "вот", true);

        let (events, _) = buffer.what_to_replay(3).expect("three-word tail");
        let original = map_original_events(&events);
        let replacement =
            decide_scoped_tail_correction_with_lem(&events, true).expect("smart replacement");

        assert_eq!(original, "аа слово вот");
        assert_eq!(replacement, "аа слово djn");
        assert_eq!(
            plan_text_replacement(&original, &replacement),
            Some(TextReplacement {
                move_left: 0,
                backspaces: 3,
                insert: "djn".to_string(),
                move_right: 0,
            })
        );
    }

    #[test]
    fn scoped_tail_uses_lem_for_two_word_mixed_tail() {
        let mut buffer = WordBuffer::new();
        push_text_as_layout(&mut buffer, "good", false);
        buffer.handle_space();
        for ch in "ntrcn".chars() {
            buffer.push(text_key_event(ch, false));
        }

        let (events, _) = buffer.what_to_replay(2).expect("two-word tail");
        let words = split_event_words(&events).expect("split words");
        let ranked = lay::lem::rank_candidates(
            &map_original_events(&events),
            scoped_tail_lem_candidates(&words, true, true),
        );

        assert_eq!(map_original_events(&events), "good ntrcn");
        assert_eq!(ranked[0].text, "good текст");
        assert_eq!(
            decide_scoped_tail_correction_with_lem(&events, true),
            Some("good текст".to_string())
        );
    }

    #[test]
    fn scoped_tail_keeps_good_russian_context_and_flips_current_acronym() {
        let cases = [
            ("ВСЁ", "ДЕЛАЙ", "ЛВУ", "KDE"),
            ("НУЖНО", "ДЕЛАТЬ", "ТЕАЫ", "NTFS"),
            ("ПРОСТО", "ДЕЛАЙ", "СЗГ", "CPU"),
        ];

        for (left1, left2, typed_tail, expected_tail) in cases {
            let mut buffer = WordBuffer::new();
            push_text_as_layout(&mut buffer, left1, true);
            buffer.handle_space();
            push_text_as_layout(&mut buffer, left2, true);
            buffer.handle_space();
            push_text_as_layout(&mut buffer, typed_tail, true);

            let (events, _) = buffer.what_to_replay(3).expect("three-word tail");
            let original = map_original_events(&events);
            let expected = format!("{left1} {left2} {expected_tail}");

            assert_eq!(
                decide_scoped_tail_correction_with_lem(&events, true),
                Some(expected.clone()),
                "original={original:?}"
            );
            assert_eq!(
                decide_scoped_tail_correction(&events),
                Some(expected.clone()),
                "original={original:?}"
            );
            assert_eq!(
                plan_text_replacement(&original, &expected),
                Some(TextReplacement {
                    move_left: 0,
                    backspaces: typed_tail.chars().count() as u32,
                    insert: expected_tail.to_string(),
                    move_right: 0,
                })
            );
        }
    }

    #[test]
    fn scoped_tail_converts_apostrophe_layout_word_as_letter() {
        let mut buffer = WordBuffer::new();
        push_text_as_layout(&mut buffer, "'nj", false);
        buffer.handle_space();
        push_text_as_layout(&mut buffer, "ckjdj", false);

        let (events, _) = buffer.what_to_replay(2).expect("two-word tail");
        let original = map_original_events(&events);

        assert_eq!(original, "'nj ckjdj");
        assert_eq!(
            decide_scoped_tail_correction_with_lem(&events, true),
            Some("это слово".to_string())
        );
    }

    #[test]
    fn scoped_tail_handles_three_completed_words_with_typo() {
        let mut buffer = WordBuffer::new();
        push_text_as_layout(&mut buffer, "ljgecntv", false);
        buffer.handle_space();
        push_text_as_layout(&mut buffer, ",ele", false);
        buffer.handle_space();
        push_text_as_layout(&mut buffer, "ошибатся", true);
        buffer.handle_space();

        let scope = effective_replace_words(&buffer, 3, CorrectionEngine::Smart, true);
        let (events, _) = buffer.what_to_replay(scope).expect("three-word tail");

        assert_eq!(scope, 3);
        assert_eq!(map_original_events(&events), "ljgecntv ,ele ошибатся ");
        assert_eq!(
            decide_scoped_tail_correction_with_lem(&events, true),
            Some("допустем буду ошибаться ".to_string())
        );
    }

    #[test]
    fn scoped_tail_keeps_live_and_flips_russian_current_tail() {
        let mut buffer = WordBuffer::new();
        push_key_events(
            &mut buffer,
            &[
                (KeyCode::KEY_L, true),
                (KeyCode::KEY_I, false),
                (KeyCode::KEY_V, false),
                (KeyCode::KEY_E, false),
            ],
            false,
        );
        buffer.handle_space();
        push_keys(
            &mut buffer,
            &[
                KeyCode::KEY_L,
                KeyCode::KEY_B,
                KeyCode::KEY_C,
                KeyCode::KEY_N,
                KeyCode::KEY_H,
                KeyCode::KEY_B,
            ],
            false,
        );
        let (events, _) = buffer.what_to_replay(2).expect("two-word tail");

        assert_eq!(map_original_events(&events), "Live lbcnhb");
        assert_eq!(
            decide_scoped_tail_correction(&events),
            Some("Live дистри".to_string())
        );
    }

    #[test]
    fn scoped_tail_normalizes_mixed_current_word_to_last_layout() {
        let mut buffer = WordBuffer::new();
        push_key_events(
            &mut buffer,
            &[
                (KeyCode::KEY_L, true),
                (KeyCode::KEY_I, false),
                (KeyCode::KEY_V, false),
                (KeyCode::KEY_E, false),
            ],
            false,
        );
        buffer.handle_space();
        push_keys(&mut buffer, &[KeyCode::KEY_L], false);
        push_keys(
            &mut buffer,
            &[KeyCode::KEY_L, KeyCode::KEY_B, KeyCode::KEY_C],
            true,
        );
        let (events, _) = buffer.what_to_replay(2).expect("two-word tail");

        assert_eq!(map_original_events(&events), "Live lдис");
        assert_eq!(
            decide_scoped_tail_correction(&events),
            Some("Live дис".to_string())
        );
    }

    #[test]
    fn scoped_tail_repairs_mixed_previous_ru_word_and_flips_current_tail() {
        let mut buffer = WordBuffer::new();
        push_key_events(
            &mut buffer,
            &[
                (KeyCode::KEY_G, true),
                (KeyCode::KEY_H, true),
                (KeyCode::KEY_J, true),
                (KeyCode::KEY_D, true),
            ],
            true,
        );
        push_key_events(
            &mut buffer,
            &[
                (KeyCode::KEY_T, true),
                (KeyCode::KEY_H, true),
                (KeyCode::KEY_M, true),
            ],
            false,
        );
        buffer.handle_space();
        push_key_events(
            &mut buffer,
            &[
                (KeyCode::KEY_W, true),
                (KeyCode::KEY_O, true),
                (KeyCode::KEY_R, true),
                (KeyCode::KEY_D, true),
            ],
            true,
        );
        let (events, _) = buffer.what_to_replay(2).expect("two-word tail");

        assert_eq!(map_original_events(&events), "ПРОВTHM ЦЩКВ");
        assert_eq!(
            decide_scoped_tail_correction(&events),
            Some("ПРОВЕРЬ WORD".to_string())
        );
    }

    #[test]
    fn smart_decision_replays_single_valid_word_as_manual_toggle() {
        assert_eq!(
            decide_correction("DOUBLE", "ВЩГИДУ", CorrectionEngine::Smart),
            Correction::ReplayAll
        );
    }

    #[test]
    fn single_word_wrong_layout_replay_target_is_opposite_layout() {
        let mut buffer = WordBuffer::new();
        push_text_as_layout(&mut buffer, "ltkfq", false);
        let (events, backspaces) = buffer.what_to_replay(1).expect("single word");
        let decision = replay_layout_decision(&events);

        assert_eq!(backspaces, 5);
        assert_eq!(map_original_events(&events), "ltkfq");
        assert!(decision.target_is_ru);
        assert_eq!(map_target_events(&events, decision.target_is_ru), "делай");
        assert_eq!(
            decide_correction("ltkfq", "делай", CorrectionEngine::Smart),
            Correction::ReplayAll
        );
    }

    #[test]
    fn single_currency_tail_replays_ru_semicolon_as_us_dollar() {
        let mut buffer = WordBuffer::new();
        push_key_events(
            &mut buffer,
            &[
                (KeyCode::KEY_4, false),
                (KeyCode::KEY_0, false),
                (KeyCode::KEY_0, false),
                (KeyCode::KEY_0, false),
                (KeyCode::KEY_4, true),
            ],
            true,
        );
        let (events, backspaces) = buffer.what_to_replay(1).expect("single word");
        let decision = replay_layout_decision(&events);
        let original = map_original_events(&events);
        let target = map_target_events(&events, decision.target_is_ru);

        assert_eq!(backspaces, 5);
        assert_eq!(original, "4000;");
        assert!(!decision.target_is_ru);
        assert_eq!(target, "4000$");
        assert_eq!(
            decide_correction(&original, &target, CorrectionEngine::Smart),
            Correction::ReplayAll
        );
    }

    #[test]
    fn smart_decision_replays_single_cyrillic_acronym_as_manual_toggle() {
        let events = [
            KeyEvent {
                keycode: KeyCode::KEY_L.code(),
                shift: true,
                layout_is_ru: true,
            },
            KeyEvent {
                keycode: KeyCode::KEY_L.code(),
                shift: true,
                layout_is_ru: true,
            },
            KeyEvent {
                keycode: KeyCode::KEY_M.code(),
                shift: true,
                layout_is_ru: true,
            },
        ];
        let decision = replay_layout_decision(&events);
        let original = map_original_events(&events);
        let target = map_events_to_layout(&events, decision.target_is_ru);

        assert_eq!(original, "ДДЬ");
        assert_eq!(target, "LLM");
        assert!(!decision.target_is_ru);
        assert_eq!(
            decide_correction(&original, &target, CorrectionEngine::Smart),
            Correction::ReplayAll
        );
    }

    #[test]
    fn smart_decision_replays_two_valid_words_as_manual_toggle() {
        assert_eq!(
            decide_correction("выводим два", "dsdjlbv ldf", CorrectionEngine::Smart),
            Correction::ReplayAll
        );
    }

    #[test]
    fn smart_decision_replays_valid_russian_preposition_phrase_as_manual_toggle() {
        assert_eq!(
            decide_correction("в доме", "d ljvt", CorrectionEngine::Smart),
            Correction::ReplayAll
        );
    }

    #[test]
    fn smart_decision_converts_mixed_layout_neighbor_only() {
        assert_eq!(
            decide_correction("рка ghj", "hrf про", CorrectionEngine::Smart),
            Correction::InsertText("рка про".to_string())
        );
        assert_eq!(
            decide_correction("проверка ghj", "ghjdthrf про", CorrectionEngine::Smart),
            Correction::InsertText("проверка про".to_string())
        );
    }

    #[test]
    fn smart_decision_replays_protected_ascii_span_as_manual_toggle() {
        assert_eq!(
            decide_correction("AmoCRM Я", "ФьщСКЬ Z", CorrectionEngine::Smart),
            Correction::ReplayAll
        );
    }

    #[test]
    fn smart_decision_repairs_brand_plus_letter_inside_larger_tail() {
        assert_eq!(
            decide_correction(
                "AmoCRM Z тут задача",
                "ФьщСКЬ Я nen pflfxf",
                CorrectionEngine::Smart
            ),
            Correction::InsertText("AmoCRM Я тут задача".to_string())
        );
    }

    #[test]
    fn replacement_plan_keeps_good_suffix_in_place() {
        assert_eq!(
            plan_text_replacement("NEN DOUBLE", "ТУТ DOUBLE"),
            Some(TextReplacement {
                move_left: 7,
                backspaces: 3,
                insert: "ТУТ".to_string(),
                move_right: 7,
            })
        );
    }

    #[test]
    fn replacement_plan_keeps_good_prefix_in_place() {
        assert_eq!(
            plan_text_replacement("Главное Вщгиду", "Главное Double"),
            Some(TextReplacement {
                move_left: 0,
                backspaces: 6,
                insert: "Double".to_string(),
                move_right: 0,
            })
        );
    }

    #[test]
    fn replacement_plan_replaces_single_bad_middle_token() {
        assert_eq!(
            plan_text_replacement("AmoCRM Z тут задача", "AmoCRM Я тут задача"),
            Some(TextReplacement {
                move_left: 11,
                backspaces: 1,
                insert: "Я".to_string(),
                move_right: 11,
            })
        );
    }

    #[test]
    fn replacement_plan_deletes_duplicate_prefix_before_kept_suffix() {
        assert_eq!(
            plan_text_replacement("на ппредмет", "на предмет"),
            Some(TextReplacement {
                move_left: 6,
                backspaces: 1,
                insert: String::new(),
                move_right: 6,
            })
        );
    }

    #[test]
    fn pending_auto_undo_restores_full_original_text() {
        let mut buffer = WordBuffer::new();
        buffer.remember_pending_auto_undo("typing-assist", "15р-16р ", "15h-16h ", 1, 1);

        let undo = buffer.take_pending_auto_undo().expect("pending undo");
        assert_eq!(
            pending_auto_undo_plan(&undo),
            TextReplacement {
                move_left: 0,
                backspaces: 8,
                insert: "15р-16р ".to_string(),
                move_right: 0,
            }
        );
    }

    #[test]
    fn opposite_events_flip_each_key_own_layout_for_smart_mixed_tail() {
        let events = [
            key_event(KeyCode::KEY_H, true),
            key_event(KeyCode::KEY_R, true),
            key_event(KeyCode::KEY_F, true),
            key_event(KeyCode::KEY_SPACE, false),
            key_event(KeyCode::KEY_G, false),
            key_event(KeyCode::KEY_H, false),
            key_event(KeyCode::KEY_J, false),
        ];

        assert_eq!(map_original_events(&events), "рка ghj");
        assert_eq!(map_opposite_events(&events), "hrf про");
    }

    #[test]
    fn smart_insert_layout_follows_result_text_tail() {
        assert!(preferred_layout_for_text("рка про", false));
        assert!(!preferred_layout_for_text("Главное Double", true));
        assert!(preferred_layout_for_text("AmoCRM Я тут задача", false));
    }

    #[test]
    fn target_layout_matches_cache_contract() {
        assert_eq!(target_layout(true), ("ru", "xkb:ru::rus"));
        assert_eq!(target_layout(false), ("us", "xkb:us::eng"));
    }

    #[test]
    fn typing_after_replay_clears_toggle_shortcut() {
        let mut buffer = WordBuffer::new();
        buffer.push(KeyEvent {
            keycode: KeyCode::KEY_D.code(),
            shift: false,
            layout_is_ru: false,
        });
        buffer.mark_replayed_layout(1, true);

        buffer.push(KeyEvent {
            keycode: KeyCode::KEY_H.code(),
            shift: false,
            layout_is_ru: true,
        });

        assert!(!buffer.replay_toggle_ready());
    }

    #[test]
    fn writes_learning_log_as_jsonl() {
        let tmp = std::env::temp_dir().join(format!(
            "lay-learn-log-test-{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()
        ));
        std::fs::create_dir_all(&tmp).unwrap();

        let path = tmp.join("corrections.jsonl");
        append_learning_log_to_path(&path, "layout-replay", "ghbdtn", "привет", 1, 1);
        let line = std::fs::read_to_string(&path).unwrap();
        let value: serde_json::Value = serde_json::from_str(line.trim()).unwrap();
        assert_eq!(value["kind"], "layout-replay");
        assert_eq!(value["from"], "ghbdtn");
        assert_eq!(value["to"], "привет");
        assert!(value.get("lay_kind").is_none());

        let _ = std::fs::remove_dir_all(tmp);
    }

    #[test]
    fn learning_feedback_records_user_fix_after_lay_correction() {
        let mut buffer = WordBuffer::new();
        buffer.remember_pending_learning_correction("typing-assist", "смотри ", "смотрин ", 1, 1);
        for _ in 0.."смотрин ".chars().count() {
            buffer.note_learning_backspace();
        }
        for key in [
            KeyCode::KEY_C,
            KeyCode::KEY_V,
            KeyCode::KEY_J,
            KeyCode::KEY_N,
            KeyCode::KEY_H,
            KeyCode::KEY_B,
        ] {
            buffer.note_learning_typed(key_event(key, true));
        }

        let correction = buffer
            .take_user_learning_correction(true)
            .expect("user correction should be captured");

        assert_eq!(
            correction,
            UserLearningCorrection {
                lay_kind: "typing-assist".to_string(),
                lay_from: "смотри ".to_string(),
                lay_to: "смотрин ".to_string(),
                from: "смотрин ".to_string(),
                to: "смотри ".to_string(),
                replace_words: 1,
                words: 1,
            }
        );
    }

    #[test]
    fn learning_feedback_ignores_lay_output_without_user_edit() {
        let mut buffer = WordBuffer::new();
        buffer.remember_pending_learning_correction("typing-assist", "смотри ", "смотрин ", 1, 1);
        buffer.note_learning_typed(key_event(KeyCode::KEY_G, true));

        assert!(buffer.take_user_learning_correction(true).is_none());
    }

    #[test]
    fn learning_feedback_does_not_attach_space_to_non_space_correction() {
        let mut buffer = WordBuffer::new();
        buffer.remember_pending_learning_correction("smart-text", "abc", "abd", 1, 1);
        buffer.note_learning_backspace();
        buffer.note_learning_typed(key_event(KeyCode::KEY_C, false));

        let correction = buffer
            .take_user_learning_correction(true)
            .expect("user correction should be captured");

        assert_eq!(correction.from, "d");
        assert_eq!(correction.to, "c");
    }

    #[test]
    fn writes_user_correction_learning_log_with_lay_context() {
        let tmp = std::env::temp_dir().join(format!(
            "lay-user-learn-log-test-{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()
        ));
        std::fs::create_dir_all(&tmp).unwrap();

        let path = tmp.join("corrections.jsonl");
        append_user_correction_learning_log_to_path(
            &path,
            &UserLearningCorrection {
                lay_kind: "typing-assist".to_string(),
                lay_from: "смотри ".to_string(),
                lay_to: "смотрин ".to_string(),
                from: "смотрин ".to_string(),
                to: "смотри ".to_string(),
                replace_words: 1,
                words: 1,
            },
        );

        let line = std::fs::read_to_string(&path).unwrap();
        let value: serde_json::Value = serde_json::from_str(line.trim()).unwrap();
        assert_eq!(value["kind"], "user-correction");
        assert_eq!(value["from"], "смотрин ");
        assert_eq!(value["to"], "смотри ");
        assert_eq!(value["lay_kind"], "typing-assist");
        assert_eq!(value["lay_from"], "смотри ");
        assert_eq!(value["lay_to"], "смотрин ");

        let _ = std::fs::remove_dir_all(tmp);
    }

    #[test]
    fn repeated_user_correction_promotes_exact_rule() {
        let tmp = std::env::temp_dir().join(format!(
            "lay-learn-promote-test-{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()
        ));
        std::fs::create_dir_all(&tmp).unwrap();

        let candidates = tmp.join("learning_candidates.json");
        let replacements = tmp.join("replacements.json");
        let correction = UserLearningCorrection {
            lay_kind: "typing-assist".to_string(),
            lay_from: "смотри ".to_string(),
            lay_to: "смотриии ".to_string(),
            from: "смотриии ".to_string(),
            to: "смотри ".to_string(),
            replace_words: 1,
            words: 1,
        };

        assert_eq!(
            promote_user_correction_if_repeated(&candidates, &replacements, &correction),
            LearningPromotion::Recorded {
                from: "смотриии".to_string(),
                to: "смотри".to_string(),
                count: 1,
            }
        );
        assert!(!replacements.exists());

        assert_eq!(
            promote_user_correction_if_repeated(&candidates, &replacements, &correction),
            LearningPromotion::Promoted {
                from: "смотриии".to_string(),
                to: "смотри".to_string(),
            }
        );

        let rules: BTreeMap<String, String> =
            serde_json::from_str(&std::fs::read_to_string(&replacements).unwrap()).unwrap();
        assert_eq!(rules.get("смотриии"), Some(&"смотри".to_string()));
        assert_eq!(
            promoted_replacement_for_token("Смотриии"),
            Some("Смотри".to_string())
        );

        let _ = std::fs::remove_dir_all(tmp);
    }

    #[test]
    fn learning_promotion_skips_unsafe_short_edits() {
        let tmp = std::env::temp_dir().join(format!(
            "lay-learn-skip-test-{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()
        ));
        std::fs::create_dir_all(&tmp).unwrap();

        let correction = UserLearningCorrection {
            lay_kind: "auto-replace".to_string(),
            lay_from: "b ".to_string(),
            lay_to: "в ".to_string(),
            from: "в ".to_string(),
            to: "и ".to_string(),
            replace_words: 1,
            words: 1,
        };

        assert_eq!(
            promote_user_correction_if_repeated(
                &tmp.join("learning_candidates.json"),
                &tmp.join("replacements.json"),
                &correction,
            ),
            LearningPromotion::Skipped
        );
        assert!(!tmp.join("replacements.json").exists());

        let _ = std::fs::remove_dir_all(tmp);
    }

    #[test]
    fn parses_gdbus_bool_tuple() {
        assert_eq!(parse_gdbus_bool("(true,)"), Some(true));
        assert_eq!(parse_gdbus_bool("(false,)"), Some(false));
        assert_eq!(parse_gdbus_bool("true"), None);
    }

    #[test]
    fn keeps_only_last_jsonl_lines() {
        let compacted = keep_last_jsonl_lines("a\nb\nc\nd\n", 2);
        assert_eq!(compacted, "c\nd\n");
    }

    #[test]
    fn applies_builtin_auto_replace_with_trailing_space() {
        assert_eq!(
            apply_auto_replace("gjlk.xbcm ", "подлючись "),
            Some("подключись ".to_string())
        );
        assert_eq!(apply_auto_replace("Tcnm ", "Есть "), None);
    }

    #[test]
    fn typing_assist_uses_exact_rules_only() {
        assert_eq!(
            apply_typing_assist_exact("подлючись "),
            Some("подключись ".to_string())
        );
        assert_eq!(
            apply_typing_assist_exact("Надйи "),
            Some("Найди ".to_string())
        );
        assert_eq!(apply_typing_assist_exact("нормально "), None);
        assert_eq!(apply_typing_assist_exact("Есть "), None);
    }

    #[test]
    fn russian_suffix_forms_are_known_candidates() {
        assert!(is_known_russian_word_or_form("препаратов"));
        assert!(is_known_russian_word_or_form("кнопками"));
        assert!(is_known_russian_word_or_form("могу"));
        assert!(is_known_russian_word_or_form("помогу"));
        assert!(is_known_russian_word_or_form("видишь"));
        assert!(is_known_russian_word_or_form("значит"));
        assert!(is_known_russian_word_or_form("страдает"));
        assert!(is_known_russian_word_or_form("установки"));
    }

    #[test]
    fn typing_assist_auto_switch_converts_confident_wrong_layout_words() {
        assert_eq!(
            apply_typing_assist("njkmrj ", true),
            Some("только ".to_string())
        );
        assert_eq!(
            apply_typing_assist("vjue ", true),
            Some("могу ".to_string())
        );
        assert_eq!(apply_typing_assist("yt ", true), Some("не ".to_string()));
        assert_eq!(
            apply_typing_assist("double b ", true),
            Some("double и ".to_string())
        );
        assert_eq!(
            apply_typing_assist_to_text_tail("посмотри я double b "),
            Some("посмотри я double и ".to_string())
        );
        assert_eq!(
            apply_typing_assist("hf,jnftn ", true),
            Some("работает ".to_string())
        );
        assert_eq!(apply_typing_assist("'nj ", true), Some("это ".to_string()));
        assert_eq!(
            apply_typing_assist("ашдуы ", true),
            Some("files ".to_string())
        );
        assert_eq!(
            apply_typing_assist("еукьштфд ", true),
            Some("terminal ".to_string())
        );
        assert_eq!(
            apply_typing_assist("неукьштфд ", true),
            Some("terminal ".to_string())
        );
        assert_eq!(
            apply_typing_assist("Lfdfq ", true),
            Some("Давай ".to_string())
        );
        assert_eq!(
            apply_typing_assist("ОБYJDB ", true),
            Some("ОБНОВИ ".to_string())
        );
        assert_eq!(
            apply_typing_assist("CRBK ", true),
            Some("СКИЛ ".to_string())
        );
        assert_eq!(apply_typing_assist("кгы ", true), Some("rus ".to_string()));
        assert_eq!(apply_typing_assist("утп ", true), Some("eng ".to_string()));
        assert_eq!(
            apply_typing_assist("njkmrj ", false),
            None,
            "auto layout word repair must stay behind the tray checkbox"
        );
    }

    #[test]
    fn typing_assist_auto_replace_off_keeps_layout_only_rules() {
        let pipeline =
            typing_assist_pipeline_for_auto_replace(false, &default_typing_assist_pipeline());

        assert_eq!(
            apply_typing_assist_with_pipeline("кгы ", true, &pipeline),
            Some("rus ".to_string())
        );
        assert_eq!(
            apply_typing_assist_with_pipeline("утп ", true, &pipeline),
            Some("eng ".to_string())
        );
        assert_eq!(
            apply_typing_assist_with_pipeline("njkmrj ", true, &pipeline),
            Some("только ".to_string())
        );
        assert_eq!(
            apply_typing_assist_with_pipeline("прорватся ", false, &pipeline),
            None
        );
        assert_eq!(
            apply_typing_assist_with_pipeline("фактческим ", false, &pipeline),
            None
        );
    }

    #[test]
    fn typing_assist_auto_replace_pipeline_avoids_risky_deletions() {
        let pipeline =
            typing_assist_pipeline_for_auto_replace(true, &default_typing_assist_pipeline());

        assert_eq!(
            apply_typing_assist_with_pipeline("исправленнно ", false, &pipeline),
            Some("исправлено ".to_string())
        );
        assert_eq!(
            apply_typing_assist_with_pipeline("кнокопками ", false, &pipeline),
            None
        );
        assert_eq!(
            apply_typing_assist_with_pipeline("бешанный ", false, &pipeline),
            None
        );
    }

    #[test]
    fn typing_assist_prefers_reflexive_verb_fix_over_extra_letter_guess() {
        assert_eq!(correct_extra_letters("прорватся"), None);
        assert_eq!(
            apply_typing_assist("прорватся ", false),
            Some("прорваться ".to_string())
        );
        assert_eq!(
            apply_typing_assist("ошибатся ", false),
            Some("ошибаться ".to_string())
        );
    }

    #[test]
    fn typing_assist_auto_switch_keeps_english_and_protected_ascii() {
        assert_eq!(apply_typing_assist("hello ", true), None);
        assert_eq!(apply_typing_assist("test ", true), None);
        assert_eq!(apply_typing_assist("good ", true), None);
        assert_eq!(apply_typing_assist("три ", true), None);
        assert_eq!(apply_typing_assist("раскладок ", true), None);
        assert_eq!(apply_typing_assist("API ", true), None);
        assert_eq!(apply_typing_assist("BTC ", true), None);
        assert_eq!(apply_typing_assist("ETH ", true), None);
        assert_eq!(apply_typing_assist("TRX ", true), None);
        assert_eq!(apply_typing_assist("AmoCRM ", true), None);
        assert_eq!(apply_typing_assist("wi-fi ", true), None);
        assert_eq!(apply_typing_assist("command -f ", true), None);
        assert_eq!(apply_typing_assist("command -r ", true), None);
        assert_eq!(apply_typing_assist("command -c ", true), None);
        assert_eq!(apply_typing_assist("grep --color=auto ", true), None);
    }

    #[test]
    fn typing_assist_pipeline_can_disable_rules() {
        let no_en_to_ru = typing_pipeline_with_disabled(&["layout_en_to_ru"]);
        assert_eq!(
            apply_typing_assist_with_pipeline("njkmrj ", true, &no_en_to_ru),
            None
        );

        let no_ru_to_en = typing_pipeline_with_disabled(&["layout_ru_to_en"]);
        assert_eq!(
            apply_typing_assist_with_pipeline("ашдуы ", true, &no_ru_to_en),
            None
        );

        let no_hard_sign = typing_pipeline_with_disabled(&["hard_sign"]);
        assert_eq!(
            apply_typing_assist_with_pipeline("Обьясни ", false, &no_hard_sign),
            None
        );
    }

    #[test]
    fn typing_assist_pipeline_priority_changes_first_match() {
        let personal_first = typing_pipeline_with_first("personal_phrase");
        let normalized = normalize_typing_assist_pipeline(&personal_first);
        assert_eq!(normalized[0].id, "personal_phrase");
        assert_eq!(normalized[0].priority, 1);
    }

    #[test]
    fn typing_assist_each_default_rule_has_isolated_positive_case() {
        struct Case {
            id: &'static str,
            input: String,
            expected: String,
            allow_layout_auto: bool,
        }

        let technical_ascii =
            map_events_to_layout(&key_events(&ascii_hyphen_token_keycodes(), false), false);
        let technical_cyrillic = lay::dict::convert(&technical_ascii, lay::dict::Direction::Us2Ru);
        let prefix_cyrillic = map_events_to_layout(&[key_event(KeyCode::KEY_W, true)], true);

        let cases = [
            Case {
                id: "moved_prefix_pair",
                input: "расчет ыприблизительные ".to_string(),
                expected: "расчеты приблизительные ".to_string(),
                allow_layout_auto: false,
            },
            Case {
                id: "split_word_pair",
                input: "я вно ".to_string(),
                expected: "явно ".to_string(),
                allow_layout_auto: false,
            },
            Case {
                id: "visual_b",
                input: "слово b ".to_string(),
                expected: "слово в ".to_string(),
                allow_layout_auto: false,
            },
            Case {
                id: "personal_phrase",
                input: "нуда ".to_string(),
                expected: "ну да ".to_string(),
                allow_layout_auto: false,
            },
            Case {
                id: "personal_token",
                input: "подлючись. ".to_string(),
                expected: "подключись. ".to_string(),
                allow_layout_auto: false,
            },
            Case {
                id: "duplicate_layout_prefix",
                input: format!("{prefix_cyrillic}{technical_ascii} "),
                expected: format!("{technical_ascii} "),
                allow_layout_auto: false,
            },
            Case {
                id: "mixed_script_layout",
                input: "ОБYJDB ".to_string(),
                expected: "ОБНОВИ ".to_string(),
                allow_layout_auto: true,
            },
            Case {
                id: "layout_technical",
                input: format!("{technical_cyrillic} "),
                expected: format!("{technical_ascii} "),
                allow_layout_auto: false,
            },
            Case {
                id: "layout_ru_to_en",
                input: "ашдуы ".to_string(),
                expected: "files ".to_string(),
                allow_layout_auto: true,
            },
            Case {
                id: "layout_en_to_ru",
                input: "njkmrj ".to_string(),
                expected: "только ".to_string(),
                allow_layout_auto: true,
            },
            Case {
                id: "cyrillic_case",
                input: "МОжно ".to_string(),
                expected: "Можно ".to_string(),
                allow_layout_auto: false,
            },
            Case {
                id: "hard_sign",
                input: "Обьясни ".to_string(),
                expected: "Объясни ".to_string(),
                allow_layout_auto: false,
            },
            Case {
                id: "adjacent_transposition",
                input: "рабоатет ".to_string(),
                expected: "работает ".to_string(),
                allow_layout_auto: false,
            },
            Case {
                id: "repeated_letter",
                input: "исправленно ".to_string(),
                expected: "исправлено ".to_string(),
                allow_layout_auto: false,
            },
            Case {
                id: "single_letter_substitution",
                input: "плозо ".to_string(),
                expected: "плохо ".to_string(),
                allow_layout_auto: false,
            },
            Case {
                id: "verb_ending",
                input: "прорватся ".to_string(),
                expected: "прорваться ".to_string(),
                allow_layout_auto: false,
            },
            Case {
                id: "vowel_confusion",
                input: "помагу ".to_string(),
                expected: "помогу ".to_string(),
                allow_layout_auto: false,
            },
            Case {
                id: "extra_letters",
                input: "кнокопками ".to_string(),
                expected: "кнопками ".to_string(),
                allow_layout_auto: false,
            },
            Case {
                id: "missing_letter",
                input: "фактческим ".to_string(),
                expected: "фактическим ".to_string(),
                allow_layout_auto: false,
            },
            Case {
                id: "glued_phrase",
                input: "когдая ".to_string(),
                expected: "когда я ".to_string(),
                allow_layout_auto: false,
            },
        ];

        let mut covered = HashSet::new();
        for case in cases {
            let pipeline = typing_pipeline_with_only(case.id);
            assert_eq!(
                apply_typing_assist_with_pipeline(&case.input, case.allow_layout_auto, &pipeline),
                Some(case.expected),
                "rule={} input={:?}",
                case.id,
                case.input
            );
            covered.insert(case.id);
        }

        let expected: HashSet<&str> = DEFAULT_TYPING_ASSIST_RULES
            .iter()
            .map(|(id, _)| *id)
            .collect();
        assert_eq!(covered, expected);
    }

    #[test]
    fn typing_assist_fixes_adjacent_transposition() {
        assert_eq!(
            apply_typing_assist_exact("рабоатет "),
            Some("работает ".to_string())
        );
        assert_eq!(
            apply_typing_assist_exact("Проверак "),
            Some("Проверка ".to_string())
        );
        assert_eq!(
            apply_typing_assist_exact("перпаратов "),
            Some("препаратов ".to_string())
        );
        assert_eq!(
            apply_typing_assist_to_text_tail("сделай понятную таблицу конкретных перпаратов "),
            Some("сделай понятную таблицу конкретных препаратов ".to_string())
        );
    }

    #[test]
    fn typing_assist_fixes_small_glued_words() {
        assert_eq!(
            apply_typing_assist_exact("нуда "),
            Some("ну да ".to_string())
        );
        assert_eq!(
            apply_typing_assist_exact("вчем "),
            Some("в чем ".to_string())
        );
        assert_eq!(
            apply_typing_assist_exact("Вчем, "),
            Some("В чем, ".to_string())
        );
    }

    #[test]
    fn typing_assist_fixes_common_missing_letter_typos() {
        assert_eq!(
            apply_typing_assist_exact("првильно "),
            Some("правильно ".to_string())
        );
        assert_eq!(
            apply_typing_assist_exact("Првильно "),
            Some("Правильно ".to_string())
        );
        assert_eq!(
            apply_typing_assist_exact("можн "),
            Some("можно ".to_string())
        );
        assert_eq!(
            apply_typing_assist_exact("Можн "),
            Some("Можно ".to_string())
        );
        assert_eq!(
            apply_typing_assist_exact("дльше "),
            Some("дальше ".to_string())
        );
        assert_eq!(
            apply_typing_assist_exact("дальг "),
            Some("дальше ".to_string())
        );
        assert_eq!(
            apply_typing_assist_exact("плозо "),
            Some("плохо ".to_string())
        );
        assert_eq!(
            apply_typing_assist_exact("фактческим "),
            Some("фактическим ".to_string())
        );
        assert_eq!(
            apply_typing_assist_exact("иблиотеку "),
            Some("библиотеку ".to_string())
        );
        assert_eq!(apply_typing_assist_exact("крипта "), None);
        assert_eq!(apply_typing_assist_exact("Крипта "), None);
    }

    #[test]
    fn typing_assist_fixes_live_user_stream_typos() {
        assert_eq!(
            apply_typing_assist_exact("занчит "),
            Some("значит ".to_string())
        );
        assert_eq!(
            apply_typing_assist_exact("работатет "),
            Some("работает ".to_string())
        );
        assert_eq!(
            apply_typing_assist_exact("котром "),
            Some("котором ".to_string())
        );
        assert_eq!(
            apply_typing_assist_exact("рабоТТА "),
            Some("работа ".to_string())
        );
        assert_eq!(
            apply_typing_assist_exact("помагу "),
            Some("помогу ".to_string())
        );
        assert_eq!(
            apply_typing_assist_exact("видешь "),
            Some("видишь ".to_string())
        );
        assert_eq!(
            apply_typing_assist_exact("кнокопками "),
            Some("кнопками ".to_string())
        );
    }

    #[test]
    fn typing_assist_normalizes_accidental_inner_uppercase() {
        assert_eq!(
            apply_typing_assist_exact("МОжно "),
            Some("Можно ".to_string())
        );
        assert_eq!(
            apply_typing_assist_exact("моЖно "),
            Some("можно ".to_string())
        );
        assert_eq!(
            apply_typing_assist_exact("рабоТА "),
            Some("работа ".to_string())
        );
        assert_eq!(apply_typing_assist_exact("МОЖНО "), None);
    }

    #[test]
    fn typing_assist_single_letter_typos_only_use_neighbor_keys() {
        assert!(are_ru_keyboard_neighbors('з', 'х'));
        assert!(!are_ru_keyboard_neighbors('о', 'ь'));
        assert_eq!(apply_typing_assist_exact("покрыто "), None);
        assert_eq!(apply_typing_assist_exact("робило "), None);
        assert_eq!(
            apply_typing_assist_exact("плозо "),
            Some("плохо ".to_string())
        );
    }

    #[test]
    fn typing_assist_merges_accidental_space_inside_word() {
        assert_eq!(
            apply_typing_assist_exact("я вно "),
            Some("явно ".to_string())
        );
        assert_eq!(
            apply_typing_assist_exact("тако й "),
            Some("такой ".to_string())
        );
        assert_eq!(
            apply_typing_assist_exact("Я вно, "),
            Some("Явно, ".to_string())
        );
        assert_eq!(apply_typing_assist_exact("я тут "), None);
        assert_eq!(apply_typing_assist_exact("мы сами "), None);
        assert_eq!(apply_typing_assist_exact("чтобы точно "), None);
        assert_eq!(apply_typing_assist_exact("хо хо "), None);
        assert_eq!(apply_typing_assist_exact("про сою "), None);
        assert_eq!(apply_typing_assist_exact("по делу "), None);
        assert_eq!(apply_typing_assist_exact("по любому "), None);
        assert_eq!(apply_typing_assist_exact("ПО ЛЮБОМУ "), None);
        assert_eq!(apply_typing_assist_exact("уже по любому "), None);
        assert_eq!(apply_typing_assist_exact("проблем "), None);
        assert_eq!(apply_typing_assist_exact("валют "), None);
        assert_eq!(apply_typing_assist_exact("систем "), None);
        assert_eq!(apply_typing_assist_exact("ноавый "), None);
        assert_eq!(apply_typing_assist("ноавый ", true), None);
        assert_eq!(apply_typing_assist_exact("раработает "), None);
        assert_eq!(apply_typing_assist_exact("зработает "), None);
        assert_eq!(apply_typing_assist_exact("новавый "), None);
        assert_eq!(
            apply_typing_assist_exact("новыйы "),
            Some("новый ".to_string())
        );
        assert_eq!(apply_typing_assist_exact("за дело "), None);
    }

    #[test]
    fn typing_assist_splits_accidentally_glued_words() {
        assert_eq!(
            apply_typing_assist_exact("ятут "),
            Some("я тут ".to_string())
        );
        assert_eq!(
            apply_typing_assist_exact("чтобыточно "),
            Some("чтобы точно ".to_string())
        );
        assert_eq!(
            apply_typing_assist_exact("когдая "),
            Some("когда я ".to_string())
        );
        assert_eq!(
            apply_typing_assist_exact("еслия "),
            Some("если я ".to_string())
        );
        assert_eq!(
            apply_typing_assist_exact("тогдая "),
            Some("тогда я ".to_string())
        );
        assert_eq!(
            apply_typing_assist_exact("можноя "),
            Some("можно я ".to_string())
        );
        assert_eq!(
            apply_typing_assist_exact("неработает "),
            Some("не работает ".to_string())
        );
        assert_eq!(
            apply_typing_assist_exact("Неработает, "),
            Some("Не работает, ".to_string())
        );
        assert_eq!(
            apply_typing_assist_exact("будуя "),
            Some("буду я ".to_string())
        );
        assert_eq!(
            apply_typing_assist_exact("у насесть "),
            Some("у нас есть ".to_string())
        );
        assert_eq!(apply_typing_assist_exact("но не "), None);
        assert_eq!(apply_typing_assist_exact("не ты "), None);
        assert_eq!(apply_typing_assist_exact("ноне ты "), None);
        assert_eq!(apply_typing_assist_exact("у насест "), None);
        assert_eq!(apply_typing_assist_exact("у насилие "), None);
        assert_eq!(apply_typing_assist_exact("машина "), None);
        assert_eq!(apply_typing_assist_exact("земля "), None);
        assert_eq!(apply_typing_assist_exact("какая "), None);
        assert_eq!(apply_typing_assist_exact("статья "), None);
        assert_eq!(apply_typing_assist_exact("семья "), None);
        assert_eq!(apply_typing_assist_exact("идея "), None);
        assert_eq!(apply_typing_assist_exact("синяя "), None);
        assert_eq!(apply_typing_assist_exact("пошли "), None);
        assert_eq!(apply_typing_assist_exact("язык "), None);
        assert_eq!(apply_typing_assist_to_text_tail("я язык "), None);
    }

    #[test]
    fn typing_assist_fixes_hard_sign_typos() {
        assert_eq!(
            apply_typing_assist_exact("Обьясни "),
            Some("Объясни ".to_string())
        );
    }

    #[test]
    fn typing_assist_moves_letter_from_next_word_back() {
        assert_eq!(
            apply_typing_assist_exact("расчет ыприблизительные "),
            Some("расчеты приблизительные ".to_string())
        );
        assert_eq!(
            apply_typing_assist_exact("дл япроверки "),
            Some("для проверки ".to_string())
        );
        assert_eq!(
            apply_typing_assist_to_text_tail("все расчет ыприблизительные "),
            Some("все расчеты приблизительные ".to_string())
        );
    }

    #[test]
    fn typing_assist_removes_duplicate_layout_prefix_from_ascii_technical_token() {
        let prefix_lower = map_events_to_layout(&[key_event(KeyCode::KEY_W, true)], true);
        let prefix_upper = map_events_to_layout(
            &[KeyEvent {
                keycode: KeyCode::KEY_W.code(),
                shift: true,
                layout_is_ru: true,
            }],
            true,
        );
        let technical_lower =
            map_events_to_layout(&key_events(&ascii_hyphen_token_keycodes(), false), false);
        let technical_upper = map_events_to_layout(
            &[
                KeyEvent {
                    keycode: KeyCode::KEY_W.code(),
                    shift: true,
                    layout_is_ru: false,
                },
                key_event(KeyCode::KEY_I, false),
                key_event(KeyCode::KEY_MINUS, false),
                KeyEvent {
                    keycode: KeyCode::KEY_F.code(),
                    shift: true,
                    layout_is_ru: false,
                },
                key_event(KeyCode::KEY_I, false),
            ],
            false,
        );
        let no_separator = map_events_to_layout(
            &key_events(
                &[
                    KeyCode::KEY_W,
                    KeyCode::KEY_I,
                    KeyCode::KEY_F,
                    KeyCode::KEY_I,
                ],
                false,
            ),
            false,
        );

        assert_eq!(
            apply_typing_assist_exact(&format!("{prefix_lower}{technical_lower} ")),
            Some(format!("{technical_lower} "))
        );
        assert_eq!(
            apply_typing_assist_exact(&format!("{prefix_upper}{technical_upper}, ")),
            Some(format!("{technical_upper}, "))
        );
        assert_eq!(
            apply_typing_assist_exact(&format!("{prefix_lower}{no_separator} ")),
            None
        );
    }

    #[test]
    fn typing_assist_does_not_move_normal_word_prefixes() {
        assert_eq!(apply_typing_assist_exact("схеме таможенник "), None);
        assert_eq!(apply_typing_assist_exact("схема таможженик "), None);
    }

    #[test]
    fn typing_assist_fixes_extra_repeated_letter() {
        assert_eq!(
            apply_typing_assist_exact("исправленно "),
            Some("исправлено ".to_string())
        );
        assert_eq!(
            apply_typing_assist_exact("исправленнно "),
            Some("исправлено ".to_string())
        );
        assert_eq!(apply_typing_assist_exact("поо "), Some("по ".to_string()));
        assert_eq!(apply_typing_assist_exact("ПОО "), Some("ПО ".to_string()));
        assert_eq!(apply_typing_assist_exact("заа "), Some("за ".to_string()));
        assert_eq!(apply_typing_assist_exact("про "), None);
        assert_eq!(apply_typing_assist_exact("ии "), None);
        assert_eq!(apply_typing_assist_exact("яя "), None);
        assert_eq!(apply_typing_assist_exact("вв "), None);
    }

    #[test]
    fn extra_letter_rule_defers_to_missing_letter_candidates() {
        let mut words: Vec<String> = russian_generated_form_dictionary()
            .iter()
            .filter(|word| (7..=12).contains(&word.chars().count()))
            .cloned()
            .collect();
        words.sort();

        let mut checked = 0usize;
        'outer: for word in words {
            let chars: Vec<char> = word.chars().collect();
            for idx in 1..chars.len().saturating_sub(1) {
                let mut typo_chars = chars.clone();
                typo_chars.remove(idx);
                let typo: String = typo_chars.into_iter().collect();
                if typo.chars().count() < 6 || is_known_russian_word_or_form(&typo) {
                    continue;
                }
                if correct_missing_letter(&typo).as_deref() != Some(word.as_str()) {
                    continue;
                }

                assert_eq!(correct_extra_letters(&typo), None, "typo={typo:?}");
                checked += 1;
                if checked >= 12 {
                    break 'outer;
                }
                break;
            }
        }

        assert!(checked >= 12, "checked={checked}");
    }

    #[test]
    fn typing_assist_keeps_valid_russian_words() {
        assert_eq!(apply_typing_assist_exact("проверка "), None);
        assert_eq!(apply_typing_assist_exact("работает "), None);
        assert_eq!(apply_typing_assist_exact("привет "), None);
        assert_eq!(apply_typing_assist_exact("можем "), None);
        assert_eq!(apply_typing_assist_exact("можешь "), None);
        assert_eq!(apply_typing_assist_exact("может "), None);
        assert_eq!(apply_typing_assist_exact("ладно "), None);
        assert_eq!(apply_typing_assist_exact("можно "), None);
        assert_eq!(apply_typing_assist_exact("дальше "), None);
        assert_eq!(apply_typing_assist_exact("плохо "), None);
        assert_eq!(apply_typing_assist_exact("правильно "), None);
        assert_eq!(apply_typing_assist_exact("исправляет "), None);
        assert_eq!(apply_typing_assist_exact("начинаю "), None);
        assert_eq!(apply_typing_assist_exact("удаляется "), None);
        assert_eq!(apply_typing_assist_exact("удателятеся "), None);
        assert_eq!(apply_typing_assist_exact("еще "), None);
        assert_eq!(apply_typing_assist_exact("елка "), None);
        assert_eq!(apply_typing_assist_exact("все "), None);
        assert_eq!(apply_typing_assist_exact("раскладок "), None);
        assert_eq!(apply_typing_assist_exact("кнопок "), None);
        assert_eq!(apply_typing_assist_exact("тестами "), None);
        assert_eq!(apply_typing_assist_exact("словами "), None);
        assert_eq!(apply_typing_assist_exact("вариантами "), None);
        assert_eq!(apply_typing_assist_exact("страдает "), None);
        assert_eq!(apply_typing_assist_exact("установки "), None);
        assert_eq!(apply_typing_assist_exact("изменю "), None);
        assert_eq!(apply_typing_assist_exact("изменю параметры "), None);
        assert_eq!(apply_typing_assist_exact("нужна "), None);
        assert_eq!(apply_typing_assist_exact("она нужна "), None);
        assert_eq!(apply_typing_assist_exact("важна "), None);
        assert_eq!(apply_typing_assist_exact("важно "), None);
        assert_eq!(apply_typing_assist_exact("банный "), None);
        assert_eq!(apply_typing_assist_exact("бешанный "), None);
        assert_eq!(apply_typing_assist_exact("БЕШАННЫЙ "), None);
        assert_eq!(apply_typing_assist_exact("поения "), None);
        assert_eq!(apply_typing_assist_exact("автозамена "), None);
        assert_eq!(apply_typing_assist_exact("агрессивная "), None);
    }

    #[test]
    fn typing_assist_ignores_words_with_digits() {
        assert_eq!(apply_typing_assist_exact("товара7 "), None);
        assert_eq!(apply_typing_assist_exact("привемр7 "), None);
        assert_eq!(apply_typing_assist_exact("пример? привемр7 "), None);
    }

    #[test]
    fn typing_assist_regression_suite_100_cases() {
        let should_fix = [
            ("подлючись ", "подключись "),
            ("надйи ", "найди "),
            ("Надйи ", "Найди "),
            ("нуда ", "ну да "),
            ("Нуда ", "Ну да "),
            ("вчем ", "в чем "),
            ("Вчем, ", "В чем, "),
            ("можн ", "можно "),
            ("Можн ", "Можно "),
            ("МОжно ", "Можно "),
            ("моЖно ", "можно "),
            ("дльше ", "дальше "),
            ("Дльше ", "Дальше "),
            ("дальг ", "дальше "),
            ("првильно ", "правильно "),
            ("Првильно ", "Правильно "),
            ("рабоатет ", "работает "),
            ("Рабоатет ", "Работает "),
            ("Проверак ", "Проверка "),
            ("ошисбя ", "ошибся "),
            ("Ошисбя ", "Ошибся "),
            ("сиправить ", "исправить "),
            ("Сиправить ", "Исправить "),
            ("плозо ", "плохо "),
            ("Плозо ", "Плохо "),
            ("фактческим ", "фактическим "),
            ("иблиотеку ", "библиотеку "),
            ("занчит ", "значит "),
            ("работатет ", "работает "),
            ("помагу ", "помогу "),
            ("видешь ", "видишь "),
            ("кнокопками ", "кнопками "),
            ("Обьясни ", "Объясни "),
            ("исправленно ", "исправлено "),
            ("Исправленно ", "Исправлено "),
            ("исправленнно ", "исправлено "),
            ("я вно ", "явно "),
            ("Я вно, ", "Явно, "),
            (
                "все расчет ыприблизительные ",
                "все расчеты приблизительные ",
            ),
            ("тут я вно ", "тут явно "),
            ("Но я вно ", "Но явно "),
            ("подлючись. ", "подключись. "),
            ("надйи! ", "найди! "),
            ("можн? ", "можно? "),
            ("дльше, ", "дальше, "),
            ("првильно. ", "правильно. "),
            ("плозо! ", "плохо! "),
            ("ошисбя, ", "ошибся, "),
        ];

        for (input, expected) in should_fix {
            assert_eq!(
                apply_typing_assist_to_text_tail(input),
                Some(expected.to_string()),
                "input={input:?}"
            );
        }

        let should_keep = [
            "привет ",
            "проверка ",
            "работает ",
            "ошибка ",
            "ошибся ",
            "явно ",
            "ладно ",
            "можно ",
            "дальше ",
            "плохо ",
            "правильно ",
            "исправлено ",
            "исправляет ",
            "покрыто ",
            "покрыть ",
            "слово ",
            "текст ",
            "модель ",
            "режим ",
            "файл ",
            "проект ",
            "тест ",
            "код ",
            "корпус ",
            "кеш ",
            "лог ",
            "демон ",
            "помощник ",
            "клавиатура ",
            "раскладка ",
            "раскладок ",
            "буфер ",
            "пробел ",
            "сейчас ",
            "потом ",
            "очень ",
            "нужно ",
            "хорошо ",
            "плохо ",
            "сделал ",
            "проверил ",
            "пишу ",
            "печатаю ",
            "быстро ",
            "медленно ",
            "нормально ",
            "отлично ",
            "давай ",
            "нет ",
            "вот ",
            "это ",
            "как ",
            "что ",
            "если ",
            "тогда ",
            "тут ",
            "там ",
            "уже ",
            "ещё ",
            "не ",
            "ни ",
            "хо хо ",
            "ха ха ",
            "CPU ",
            "LLM ",
            "API ",
            "МГУ ",
            "README ",
            "GitHub ",
            "WeChat ",
            "hello ",
            "world ",
            "cargo ",
            "Rust ",
            "GNOME ",
            "Wayland ",
            "Ollama ",
            "Qwen ",
            "BitNet ",
            "smollm ",
            "conecargo.ru ",
            "test@example.com ",
            "https://example.com ",
            "123 ",
            "7 ",
            "b/ ",
            "и. ",
            "в магазин ",
            "в вот ",
            "машина ",
            "магазин ",
            "тестами ",
            "словами ",
            "вариантами ",
            "схеме таможенник ",
            "схема таможженик ",
            "пошли ",
            "пошли в ",
            "ни фига ",
            "не фига ",
            "как говорится ",
            "ну что же ",
        ];

        for input in should_keep {
            assert_eq!(
                apply_typing_assist_to_text_tail(input),
                None,
                "input={input:?}"
            );
        }

        let total = should_fix.len() + should_keep.len();
        assert!(
            total >= 100,
            "regression suite should keep at least 100 cases, got {total}"
        );
    }

    #[test]
    fn auto_replace_regression_suite() {
        let cases = [
            ("перейти b", "gthtqnb b", "перейти в"),
            ("b ghjcnj", "и просто", "в просто"),
            ("слово b ", "слово и ", "слово в "),
            ("b vfufpby ", "и магазин ", "в магазин "),
            ("b djn", "и вот", "в вот"),
            (
                "посмотри я double b",
                "gjcvjnhb z вщгиду и",
                "посмотри я double и",
            ),
        ];

        for (original, target, expected) in cases {
            assert_eq!(
                apply_auto_replace(original, target),
                Some(expected.to_string()),
                "original={original:?} target={target:?}"
            );
        }
    }

    #[test]
    fn replaces_visual_b_inside_russian_context() {
        assert_eq!(
            apply_auto_replace("перейти b", "gthtqnb b"),
            Some("перейти в".to_string())
        );
        assert_eq!(
            apply_auto_replace("b ghjcnj", "и просто"),
            Some("в просто".to_string())
        );
        assert_eq!(
            apply_auto_replace("слово b ", "слово и "),
            Some("слово в ".to_string())
        );
    }
}

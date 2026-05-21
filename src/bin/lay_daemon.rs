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
use evdev::Device;
use lay::config::{CorrectionEngine, LayConfig, TypingAssistRuleConfig};
use lay::desktop::LayoutBackend;
use lay::text_backend::TextBackendPreference;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::OnceLock;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

const GNOME_NATIVE_REPLACE_EXPERIMENTAL: bool = false;
const LAYOUT_POLL_INTERVAL_MS: u64 = 250;
const ENTER_AUTOCORRECT_EXPERIMENT_ENV: &str = "LAY_EXPERIMENTAL_ENTER_AUTOCORRECT";
static AUTO_LAYOUT_BACKEND_HINT: OnceLock<Option<LayoutBackend>> = OnceLock::new();
static TYPING_ASSIST_RUNTIME_READY: AtomicBool = AtomicBool::new(false);

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
    if !config_enabled {
        return false;
    }
    env_value
        .map(|value| {
            matches!(
                value.trim().to_ascii_lowercase().as_str(),
                "1" | "true" | "yes" | "on"
            )
        })
        .unwrap_or(true)
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
    lay::config::typing_assist_pipeline_for_policy(
        cfg.auto_replace,
        cfg.active_correction_safety(),
        &cfg.typing_assist_pipeline,
    )
}

fn record_recent_action(
    kind: &str,
    from: &str,
    to: &str,
    replace_words: usize,
    words: usize,
    started_at: Instant,
    undo_available: bool,
) {
    lay::action_log::record_action(
        kind,
        from,
        to,
        replace_words,
        words,
        started_at.elapsed().as_millis(),
        undo_available,
    );
}

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

// ─── Text output / uinput helpers ─────────────────────────

#[path = "lay_daemon/text_output.rs"]
mod text_output;
use text_output::*;

// ─── Daemon runtime orchestration ──────────────────────

#[path = "lay_daemon/daemon_runtime.rs"]
mod daemon_runtime;
use daemon_runtime::*;

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

// keyboard event loop lives in lay_daemon/daemon_runtime.rs

// ─── Focus guard / event wait timing ─────────────────────

#[path = "lay_daemon/focus_guard.rs"]
mod focus_guard;
use focus_guard::*;

// ─── Trigger FSM and word-boundary helpers ─────────────────

#[path = "lay_daemon/trigger_fsm.rs"]
mod trigger_fsm;
use trigger_fsm::*;

// ─── Двойной Shift handler ──────────────────────────────────

// ─── Correction runtime orchestration ────────────────────

#[path = "lay_daemon/correction_runtime.rs"]
mod correction_runtime;
use correction_runtime::*;

#[path = "lay_daemon/typing_assist_runtime.rs"]
mod typing_assist_runtime;
use typing_assist_runtime::*;

// ─── Layout, DBus, IME controller ────────────────────────

#[path = "lay_daemon/layout_controller.rs"]
mod layout_controller;
use layout_controller::*;

// keyboard discovery lives in lay_daemon/daemon_runtime.rs

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

// ─── Learning log / promotion runtime ──────────────────────

#[path = "lay_daemon/learning_runtime.rs"]
mod learning_runtime;
use learning_runtime::*;

#[cfg(test)]
#[path = "lay_daemon/tests.rs"]
mod tests;

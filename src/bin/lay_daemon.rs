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
use std::sync::atomic::AtomicBool;

const GNOME_NATIVE_REPLACE_EXPERIMENTAL: bool = false;
const LAYOUT_POLL_INTERVAL_MS: u64 = 250;
const ENTER_AUTOCORRECT_EXPERIMENT_ENV: &str = "LAY_EXPERIMENTAL_ENTER_AUTOCORRECT";
static TYPING_ASSIST_RUNTIME_READY: AtomicBool = AtomicBool::new(false);

// ─── Config ─────────────────────────────────────────────────

#[path = "lay_daemon/config_runtime.rs"]
mod config_runtime;
use config_runtime::*;

#[path = "lay_daemon/action_log_runtime.rs"]
mod action_log_runtime;
use action_log_runtime::*;

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

impl DeviceGrabGuard<'_> {
    fn is_active(&self) -> bool {
        self.active
    }
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

// ─── Keyboard device I/O helpers ──────────────────────────

#[path = "lay_daemon/keyboard_io.rs"]
mod keyboard_io;
use keyboard_io::*;

// ─── Daemon runtime orchestration ──────────────────────

#[path = "lay_daemon/trigger_dispatch.rs"]
mod trigger_dispatch;

#[path = "lay_daemon/manual_trigger_runtime.rs"]
mod manual_trigger_runtime;

#[path = "lay_daemon/boundary_runtime.rs"]
mod boundary_runtime;

#[path = "lay_daemon/typing_key_runtime.rs"]
mod typing_key_runtime;

#[path = "lay_daemon/text_context_runtime.rs"]
mod text_context_runtime;

#[path = "lay_daemon/buffer_filter_runtime.rs"]
mod buffer_filter_runtime;
use buffer_filter_runtime::*;

#[path = "lay_daemon/daemon_runtime.rs"]
mod daemon_runtime;
use daemon_runtime::*;

#[path = "lay_daemon/daemon_state.rs"]
mod daemon_state;

#[path = "lay_daemon/pending_typing_assist.rs"]
mod pending_typing_assist;

#[path = "lay_daemon/startup_runtime.rs"]
mod startup_runtime;
use startup_runtime::*;

fn main() -> std::io::Result<()> {
    let args = Args::parse();
    set_log_enabled(args.debug_log || args.verbose || args.detect_only);
    run_daemon(args.detect_only, args.device, args.verbose)
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

#[path = "lay_daemon/force_layout_hotkeys.rs"]
mod force_layout_hotkeys;
use force_layout_hotkeys::*;

#[path = "lay_daemon/manual_trigger_diagnostics.rs"]
mod manual_trigger_diagnostics;
use manual_trigger_diagnostics::*;

// ─── Двойной Shift handler ──────────────────────────────────

// ─── Correction runtime orchestration ────────────────────

#[path = "lay_daemon/correction_memory_runtime.rs"]
mod correction_memory_runtime;

#[path = "lay_daemon/auto_undo_runtime.rs"]
mod auto_undo_runtime;

#[path = "lay_daemon/correction_runtime.rs"]
mod correction_runtime;
use correction_runtime::*;

#[path = "lay_daemon/physical_input_grab.rs"]
mod physical_input_grab;

#[path = "lay_daemon/typing_assist_runtime.rs"]
mod typing_assist_runtime;
use typing_assist_runtime::*;

#[path = "lay_daemon/enter_autocorrect_runtime.rs"]
mod enter_autocorrect_runtime;
use enter_autocorrect_runtime::*;

// ─── Layout, DBus, IME controller ────────────────────────

#[path = "lay_daemon/layout_kde.rs"]
mod layout_kde;

#[path = "lay_daemon/layout_x11.rs"]
mod layout_x11;

#[path = "lay_daemon/command_runtime.rs"]
mod command_runtime;

#[path = "lay_daemon/layout_controller.rs"]
mod layout_controller;
use layout_controller::*;

// keyboard discovery lives in lay_daemon/keyboard_io.rs

// ─── Лог ────────────────────────────────────────────────────

#[path = "lay_daemon/log_runtime.rs"]
mod log_runtime;
use log_runtime::*;

// ─── Learning log / promotion runtime ──────────────────────

#[path = "lay_daemon/learning_runtime.rs"]
mod learning_runtime;
use learning_runtime::*;

#[cfg(test)]
#[path = "lay_daemon/tests.rs"]
mod tests;

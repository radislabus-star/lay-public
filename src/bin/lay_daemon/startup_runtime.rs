use evdev::uinput::VirtualDevice;
use lay::config::{CorrectionEngine, LayConfig};
use lay::desktop::LayoutBackend;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Instant;

use super::{
    active_enter_autocorrect_from_env, active_layout_backend, call_ime_ping, call_ping,
    find_all_keyboards, find_all_pointers, layout_niri, listen_keyboard, listen_pointer, log,
    make_virtual_keyboard, startup_sanitize, ENTER_AUTOCORRECT_EXPERIMENT_ENV,
    TYPING_ASSIST_RUNTIME_READY,
};

pub(super) fn run_daemon(
    detect_only: bool,
    device: Option<String>,
    verbose: bool,
) -> std::io::Result<()> {
    let device_paths = keyboard_device_paths(device)?;
    log_startup_mode(&device_paths, detect_only);

    let startup_cfg = LayConfig::load();
    let startup_backend = active_layout_backend();
    log_startup_backends(&startup_cfg, startup_backend);
    startup_sanitize::sanitize_user_replacements();
    warm_runtime_if_needed(detect_only, &startup_cfg);
    probe_backends(detect_only, startup_backend, &startup_cfg);

    let virtual_kbd = make_virtual_keyboard_for_runtime(detect_only);
    spawn_keyboard_threads(device_paths, virtual_kbd, verbose)
}

fn keyboard_device_paths(device: Option<String>) -> std::io::Result<Vec<PathBuf>> {
    match device {
        Some(path) => Ok(vec![PathBuf::from(path)]),
        None => find_all_keyboards(),
    }
}

fn log_startup_mode(device_paths: &[PathBuf], detect_only: bool) {
    log(&format!("► старт, устройства: {device_paths:?}"));
    let mode = if detect_only {
        "DETECT-ONLY"
    } else {
        "LIVE (DBus + uinput)"
    };
    log(&format!("► режим: {mode}"));
}

fn log_startup_backends(cfg: &LayConfig, backend: LayoutBackend) {
    log(&format!(
        "► layout backend: {} (config={})",
        backend.label(),
        cfg.layout_backend
    ));
    log(&format!(
        "► text backend: {}",
        cfg.active_text_backend().as_str()
    ));
}

fn warm_runtime_if_needed(detect_only: bool, cfg: &LayConfig) {
    let plan = runtime_warmup_plan(
        detect_only,
        cfg,
        std::env::var(ENTER_AUTOCORRECT_EXPERIMENT_ENV)
            .ok()
            .as_deref(),
    );
    if plan.spawn_background {
        std::thread::spawn(move || {
            let started_at = Instant::now();
            lay::ngram::warm_up();
            lay::lem::warm_up();
            lay::typing_assist::warm_up();
            if plan.warm_nanda {
                lay::nanda_wave::warm_up();
            }
            TYPING_ASSIST_RUNTIME_READY.store(true, Ordering::Relaxed);
            if plan.warm_smart {
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
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct RuntimeWarmupPlan {
    spawn_background: bool,
    warm_smart: bool,
    warm_nanda: bool,
}

fn runtime_warmup_plan(
    detect_only: bool,
    cfg: &LayConfig,
    enter_autocorrect_env: Option<&str>,
) -> RuntimeWarmupPlan {
    let warm_smart = cfg.active_correction_engine() == CorrectionEngine::Smart;
    let enter_autocorrect_active =
        active_enter_autocorrect_from_env(cfg.enter_autocorrect, enter_autocorrect_env);
    let warm_typing_assist = cfg.typing_assist || enter_autocorrect_active;
    let warm_nanda = cfg.nanda_autocorrect || cfg.nanda_precognition || cfg.nanda_trace;
    RuntimeWarmupPlan {
        spawn_background: !detect_only && (warm_smart || warm_typing_assist),
        warm_smart,
        warm_nanda,
    }
}

fn probe_backends(detect_only: bool, backend: LayoutBackend, cfg: &LayConfig) {
    if detect_only {
        return;
    }

    match backend {
        LayoutBackend::Gnome => match call_ping() {
            Ok(reply) => log(&format!("► extension: {reply}")),
            Err(e) => {
                log(&format!("⚠ extension не отвечает ({e})"));
                log("⚠ работаю в detect-only");
            }
        },
        LayoutBackend::X11 => match lay::x11_layout::ping() {
            Ok(reply) => log(&format!("► native X11 backend: {reply}")),
            Err(e) => log(&format!(
                "⚠ native X11 backend unavailable ({e}); shell fallback remains enabled"
            )),
        },
        LayoutBackend::Niri => match layout_niri::ping() {
            Ok(reply) => log(&format!("► niri IPC backend: {reply}")),
            Err(e) => {
                log(&format!("⚠ niri IPC не отвечает ({e})"));
                log("⚠ работаю в detect-only");
            }
        },
        LayoutBackend::Kde => log("► GNOME extension ping skipped for non-GNOME layout backend"),
    }

    if cfg.active_text_backend().should_try_ime() {
        match call_ime_ping() {
            Ok(reply) => log(&format!("► IME bridge: {reply}")),
            Err(e) => log(&format!(
                "⚠ IME bridge unavailable ({e}); uinput fallback remains enabled"
            )),
        }
    }
}

fn make_virtual_keyboard_for_runtime(detect_only: bool) -> Option<VirtualDevice> {
    if detect_only {
        return None;
    }
    match make_virtual_keyboard() {
        Ok(device) => {
            log("► uinput virtual keyboard создан");
            Some(device)
        }
        Err(e) => {
            log(&format!(
                "⚠ uinput недоступен ({e}). Re-typing работать не будет"
            ));
            None
        }
    }
}

fn spawn_keyboard_threads(
    device_paths: Vec<PathBuf>,
    virtual_kbd: Option<VirtualDevice>,
    verbose: bool,
) -> std::io::Result<()> {
    let virtual_kbd = Arc::new(Mutex::new(virtual_kbd));
    let field_context_epoch = Arc::new(AtomicU64::new(0));
    let mut handles = Vec::new();
    for path in find_all_pointers().unwrap_or_default() {
        let field_context_epoch = Arc::clone(&field_context_epoch);
        handles.push(std::thread::spawn(move || {
            if let Err(e) = listen_pointer(path, field_context_epoch, verbose) {
                log(&format!("⚠ thread pointer: {e}"));
            }
        }));
    }
    for path in device_paths {
        let virtual_kbd = Arc::clone(&virtual_kbd);
        let field_context_epoch = Arc::clone(&field_context_epoch);
        let cfg = LayConfig::load();
        handles.push(std::thread::spawn(move || {
            if let Err(e) = listen_keyboard(path, virtual_kbd, field_context_epoch, verbose, cfg) {
                log(&format!("⚠ thread keyboard: {e}"));
            }
        }));
    }
    for handle in handles {
        let _ = handle.join();
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn warmup_plan_waits_for_nanda_when_autocorrect_uses_it() {
        let cfg = LayConfig {
            typing_assist: true,
            nanda_autocorrect: true,
            ..LayConfig::default()
        };

        let plan = runtime_warmup_plan(false, &cfg, None);

        assert!(plan.spawn_background);
        assert!(plan.warm_nanda);
    }

    #[test]
    fn warmup_plan_does_not_wait_for_nanda_when_nanda_is_disabled() {
        let cfg = LayConfig {
            typing_assist: true,
            nanda_autocorrect: false,
            nanda_precognition: false,
            nanda_trace: false,
            ..LayConfig::default()
        };

        let plan = runtime_warmup_plan(false, &cfg, None);

        assert!(plan.spawn_background);
        assert!(!plan.warm_nanda);
    }

    #[test]
    fn warmup_plan_keeps_detect_only_ready_without_background_thread() {
        let cfg = LayConfig {
            typing_assist: true,
            nanda_autocorrect: true,
            ..LayConfig::default()
        };

        let plan = runtime_warmup_plan(true, &cfg, None);

        assert!(!plan.spawn_background);
        assert!(plan.warm_nanda);
    }
}

use evdev::{uinput::VirtualDevice, Device};
use lay::config::{CorrectionEngine, LayConfig};
use lay::desktop::LayoutBackend;
use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Instant;

use super::{
    active_enter_autocorrect_from_env, active_layout_backend, call_ime_ping, call_ping,
    find_all_keyboards, find_all_pointers, layout_niri, listen_keyboard, listen_pointer, log,
    make_virtual_keyboard, release_all_virtual_keys, startup_sanitize,
    wait_for_keyboard_event_or_timeout, ENTER_AUTOCORRECT_EXPERIMENT_ENV, SHUTDOWN_REQUESTED,
    TYPING_ASSIST_RUNTIME_READY,
};

#[path = "startup_runtime/device_access.rs"]
mod device_access;
use device_access::{
    discover_paths, recover_output, refresh_listeners, DeviceKind, Listeners, OutputState,
};

include!("startup_runtime/warmup.rs");
#[cfg(test)]
#[path = "startup_runtime/tests/warmup.rs"]
mod tests;

pub(super) fn run_daemon(
    detect_only: bool,
    device: Option<String>,
    verbose: bool,
) -> std::io::Result<()> {
    let explicit_device = device.map(PathBuf::from);
    log_startup_mode(
        &explicit_device.iter().cloned().collect::<Vec<_>>(),
        detect_only,
    );

    let startup_cfg = LayConfig::load();
    let startup_backend = active_layout_backend();
    log_startup_backends(&startup_cfg, startup_backend);
    startup_sanitize::sanitize_user_replacements();
    warm_runtime_if_needed(detect_only, &startup_cfg);
    probe_backends(detect_only, startup_backend, &startup_cfg);

    maintain_input_devices(explicit_device, detect_only, verbose)
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

fn make_ready_virtual_keyboard() -> std::io::Result<VirtualDevice> {
    let mut device = make_virtual_keyboard()?;
    release_all_virtual_keys(&mut device)?;
    Ok(device)
}

/// A listener's lifetime invalidates existing field/gesture context, including panic exit.
struct ListenerEpoch(Arc<AtomicU64>);

impl ListenerEpoch {
    fn enter(epoch: Arc<AtomicU64>) -> Self {
        epoch.fetch_add(1, Ordering::AcqRel);
        Self(epoch)
    }
}

impl Drop for ListenerEpoch {
    fn drop(&mut self) {
        self.0.fetch_add(1, Ordering::AcqRel);
    }
}

fn report_device_access(label: &str, error: Option<&std::io::Error>, warned: &mut bool) {
    let failed = error.is_some();
    if failed != *warned {
        if failed {
            eprintln!(
                "lay: {label} unavailable ({}); retrying device access once per second",
                error.unwrap()
            );
        } else {
            eprintln!("lay: {label} access recovered");
        }
        *warned = failed;
    }
}

fn maintain_input_devices(
    explicit_device: Option<PathBuf>,
    detect_only: bool,
    verbose: bool,
) -> std::io::Result<()> {
    let virtual_kbd = Arc::new(Mutex::new(None));
    let field_context_epoch = Arc::new(AtomicU64::new(0));
    let mut listeners = Listeners::new();
    let mut listener_warnings = HashSet::new();
    let (mut output_warning, mut keyboard_warning, mut pointer_warning) = (false, false, false);

    while !SHUTDOWN_REQUESTED.load(Ordering::Acquire) {
        match recover_output(
            &virtual_kbd,
            detect_only,
            &SHUTDOWN_REQUESTED,
            make_ready_virtual_keyboard,
        ) {
            Ok(OutputState::Ready | OutputState::Recovered) => {
                report_device_access("uinput", None, &mut output_warning);
            }
            Ok(OutputState::Skipped) => {}
            Err(error) => report_device_access("uinput", Some(&error), &mut output_warning),
        }
        if SHUTDOWN_REQUESTED.load(Ordering::Acquire) {
            break;
        }
        let (keyboards, pointers) = discover_paths(
            explicit_device.as_deref(),
            find_all_keyboards,
            find_all_pointers,
        );
        report_device_access("keyboards", keyboards.as_ref().err(), &mut keyboard_warning);
        report_device_access("pointers", pointers.as_ref().err(), &mut pointer_warning);
        let desired: Vec<_> = keyboards
            .unwrap_or_default()
            .into_iter()
            .map(|path| (DeviceKind::Keyboard, path))
            .chain(
                pointers
                    .unwrap_or_default()
                    .into_iter()
                    .map(|path| (DeviceKind::Pointer, path)),
            )
            .collect();
        listener_warnings.retain(|key| desired.contains(key) || listeners.contains_key(key));
        let attempts = refresh_listeners(
            &mut listeners,
            &desired,
            &SHUTDOWN_REQUESTED,
            |(kind, path)| {
                // Pass the actual successfully opened fd to the listener, not a probe followed by a second open.
                let device = Device::open(path)?;
                if SHUTDOWN_REQUESTED.load(Ordering::Acquire) {
                    return Err(std::io::Error::from(std::io::ErrorKind::Interrupted));
                }
                let field_context_epoch = Arc::clone(&field_context_epoch);
                let path = path.clone();
                let virtual_kbd = Arc::clone(&virtual_kbd);
                let kind = kind.clone();
                let cfg = LayConfig::load();
                if SHUTDOWN_REQUESTED.load(Ordering::Acquire) {
                    return Err(std::io::Error::from(std::io::ErrorKind::Interrupted));
                }
                Ok(std::thread::spawn(move || {
                    if SHUTDOWN_REQUESTED.load(Ordering::Acquire) {
                        return;
                    }
                    let _epoch = ListenerEpoch::enter(Arc::clone(&field_context_epoch));
                    let result = match kind {
                        DeviceKind::Keyboard => listen_keyboard(
                            path,
                            device,
                            virtual_kbd,
                            field_context_epoch,
                            verbose,
                            cfg,
                        ),
                        DeviceKind::Pointer => {
                            listen_pointer(path, device, field_context_epoch, verbose)
                        }
                    };
                    if let Err(error) = result {
                        log(&format!(
                            "⚠ input listener exited; device will be rediscovered: {error}"
                        ));
                    }
                }))
            },
        );
        for (key, result) in attempts {
            match result {
                Ok(()) => {
                    if listener_warnings.remove(&key) {
                        eprintln!("lay: input listener access recovered: {key:?}");
                    }
                }
                Err(error) => {
                    if !SHUTDOWN_REQUESTED.load(Ordering::Acquire)
                        && listener_warnings.insert(key.clone())
                    {
                        eprintln!("lay: input listener unavailable ({error}); retrying: {key:?}");
                    }
                }
            }
        }
        if !SHUTDOWN_REQUESTED.load(Ordering::Acquire) {
            // poll ignores a negative fd: use the existing input-loop wait owner
            // for a timeout even while no keyboard is available.
            wait_for_keyboard_event_or_timeout(-1, std::time::Duration::from_secs(1))?;
        }
    }
    for (_, handle) in listeners {
        let _ = handle.join();
    }
    let mut virtual_kbd = match virtual_kbd.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };
    if let Some(device) = virtual_kbd.as_mut() {
        if let Err(error) = release_all_virtual_keys(device) {
            log(&format!(
                "⚠ virtual keyboard shutdown release failed: {error}"
            ));
        }
    }
    Ok(())
}

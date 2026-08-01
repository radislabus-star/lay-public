//! lay-test-input — test harness для автоматической проверки lay-daemon.
//!
//! Создаёт виртуальную клавиатуру через uinput, печатает её путь в stdout
//! (для запуска `lay-daemon --device <path>`), затем по сигналу или таймеру
//! эмулирует тестовые сценарии.
//!
//! Использование:
//!   lay-test-input x11-diagnostics — печатает X11/backend diagnostics report
//!   lay-test-input x11-report — печатает GitHub-ready X11 validation report
//!   lay-test-input script:<path> — запускает TSV-сценарий
//!   lay-test-input <name> — запускает ручной сценарий или builtin TSV из
//!       data/test_input/builtin_scripts.tsv
//!   lay-test-input list        — только создаёт kbd и держит, печатает путь
//!
//! Input scenarios require `LAY_TEST_INPUT_ARMED=1`. The runtime smoke
//! harness sets it only after opening its isolated capture field.

#[path = "lay_test_input/desktop_probe.rs"]
mod desktop_probe;
#[path = "lay_test_input/input_device.rs"]
mod input_device;
#[path = "lay_test_input/scenarios.rs"]
mod scenarios;

use desktop_probe::{print_x11_diagnostics, print_x11_report};
use input_device::build_virtual_keyboard;
use scenarios::run_scenario;
use std::env;
use std::io::Write;
use std::thread::sleep;
use std::time::Duration;

fn main() -> std::io::Result<()> {
    let scenario = env::args().nth(1).unwrap_or_else(|| "list".to_string());
    if matches!(scenario.as_str(), "x11-diagnostics" | "diagnose-x11") {
        print_x11_diagnostics();
        return Ok(());
    }
    if matches!(scenario.as_str(), "x11-report" | "report-x11") {
        print_x11_report();
        return Ok(());
    }
    if scenario != "list" && env::var("LAY_TEST_INPUT_ARMED").ok().as_deref() != Some("1") {
        return Err(std::io::Error::new(
            std::io::ErrorKind::PermissionDenied,
            "refusing live synthetic input without LAY_TEST_INPUT_ARMED=1",
        ));
    }

    let mut dev = build_virtual_keyboard()?;

    if let Some(path) = dev.enumerate_dev_nodes_blocking()?.next().transpose()? {
        println!("{}", path.display());
        std::io::stdout().flush()?;
    }

    eprintln!("[test] virtual keyboard создана");
    let start_delay_ms = env::var("LAY_TEST_START_DELAY_MS")
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .unwrap_or(3000);
    sleep(Duration::from_millis(start_delay_ms)); // дать daemon открыть устройство

    run_scenario(&mut dev, &scenario)?;

    Ok(())
}

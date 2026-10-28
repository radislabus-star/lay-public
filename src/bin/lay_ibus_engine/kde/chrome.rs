//! Chrome process qualification and activation budget; no presentation or edit authority.
use std::path::Path;
use std::time::{Duration, Instant};

const ACTIVATION_BUDGET: Duration = Duration::from_millis(100);

pub(crate) fn activation_deadline(pid: u32, now: Instant) -> Option<Instant> {
    let exe = std::fs::read_link(format!("/proc/{pid}/exe")).ok()?;
    activation_budget_for_process(&exe).and_then(|budget| now.checked_add(budget))
}

fn activation_budget_for_process(exe: &Path) -> Option<Duration> {
    is_chrome_process(exe).then_some(ACTIVATION_BUDGET)
}

pub(crate) fn supports_native_prefix(pid: u32) -> bool {
    std::fs::read_link(format!("/proc/{pid}/exe"))
        .ok()
        .is_some_and(|exe| is_chrome_process(&exe))
}

fn is_chrome_process(exe: &Path) -> bool {
    // The launcher execs this binary. Use the actual canonical-window process,
    // not a window title, browser hint, wrapper or accessibility description.
    exe.file_name().is_some_and(|name| name == "chrome")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn native_prefix_scope_preserves_other_kde_clients() {
        assert!(is_chrome_process(Path::new("/opt/google/chrome/chrome")));
        assert_eq!(
            activation_budget_for_process(Path::new("/opt/google/chrome/chrome")),
            Some(Duration::from_millis(100)),
        );
        for exe in [
            "/usr/bin/kitty",
            "/usr/bin/konsole",
            "/usr/bin/firefox",
            "/usr/bin/soffice.bin",
            "/opt/google/chrome/chrome_crashpad_handler",
            "/usr/bin/google-chrome-stable",
        ] {
            assert!(!is_chrome_process(Path::new(exe)), "{exe}");
            assert_eq!(activation_budget_for_process(Path::new(exe)), None);
        }
        assert!(!supports_native_prefix(0));
        assert_eq!(activation_deadline(0, Instant::now()), None);
    }
}

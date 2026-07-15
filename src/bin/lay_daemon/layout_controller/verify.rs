use std::time::Duration;

const ATTEMPTS: usize = 5;
const POLL_MS: u64 = 10;

pub(crate) fn verify_current_layout(target_is_ru: bool) -> bool {
    verify_with_retry(|| {
        super::read_current_layout_is_ru().is_ok_and(|current| current == target_is_ru)
    })
}

pub(super) fn verify_gnome_shell_layout(target_is_ru: bool) -> bool {
    verify_with_retry(|| {
        super::read_current_gnome_shell_layout_is_ru().is_ok_and(|current| current == target_is_ru)
    })
}

pub(super) fn verify_gnome_layout_stack(target_is_ru: bool) -> bool {
    verify_with_retry(|| {
        super::read_current_gnome_shell_layout_is_ru().is_ok_and(|current| current == target_is_ru)
            && super::read_current_ibus_layout_is_ru().is_ok_and(|current| current == target_is_ru)
    })
}

fn verify_with_retry(check: impl FnMut() -> bool) -> bool {
    verify_layout_with_retry_config(ATTEMPTS, POLL_MS, check)
}

pub(super) fn verify_layout_with_retry_config(
    attempts: usize,
    poll_ms: u64,
    mut check: impl FnMut() -> bool,
) -> bool {
    for _ in 0..attempts {
        if check() {
            return true;
        }
        if poll_ms > 0 {
            std::thread::sleep(Duration::from_millis(poll_ms));
        }
    }
    false
}

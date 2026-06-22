use super::ime_state::has_text_authority as ime_input_state_has_text_authority;
use super::verify_layout_with_retry_config;

#[test]
fn verify_layout_retry_stops_after_success() {
    let mut calls = 0;

    assert!(verify_layout_with_retry_config(5, 0, || {
        calls += 1;
        calls == 3
    }));
    assert_eq!(calls, 3);
}

#[test]
fn verify_layout_retry_uses_all_attempts_on_failure() {
    let mut calls = 0;

    assert!(!verify_layout_with_retry_config(5, 0, || {
        calls += 1;
        false
    }));
    assert_eq!(calls, 5);
}

#[test]
fn ime_committed_tail_blocks_daemon_boundary_autocorrect() {
    assert!(ime_input_state_has_text_authority("active:composition"));
    assert!(ime_input_state_has_text_authority("passive:committed-tail"));
    assert!(!ime_input_state_has_text_authority(
        "passive:daemon-word-buffer"
    ));
    assert!(!ime_input_state_has_text_authority("passive:no-focus"));
}

use super::verify::verify_layout_with_retry_config;

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

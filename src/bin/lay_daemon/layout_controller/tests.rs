use super::{exact_gnome_layout_id_matches, verify::verify_layout_with_retry_config};

#[test]
fn exact_replay_layout_readback_accepts_only_the_frozen_layout_and_ime_aliases() {
    assert!(exact_gnome_layout_id_matches("us", "us", "lay-ime-us"));
    assert!(exact_gnome_layout_id_matches(
        "lay-ime-us",
        "us",
        "lay-ime-us"
    ));
    assert!(exact_gnome_layout_id_matches("ru", "ru", "lay-ime-ru"));
    assert!(exact_gnome_layout_id_matches(
        "lay-ime-ru",
        "ru",
        "lay-ime-ru"
    ));
    assert!(!exact_gnome_layout_id_matches(
        "lay-ime-ru",
        "us",
        "lay-ime-us"
    ));
    assert!(!exact_gnome_layout_id_matches(
        "xkb:us::eng",
        "us",
        "lay-ime-us"
    ));
}

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
fn exact_replay_layout_handoff_is_one_shot_and_has_no_generic_wait_path() {
    let source = include_str!("../layout_controller.rs");
    let exact_route = source
        .split("pub(super) fn activate_target_layout_once_for_exact_replay")
        .nth(1)
        .expect("exact replay route")
        .split("pub(super) fn call_ping")
        .next()
        .expect("next public function");

    assert!(exact_route.contains("call_activate_layout_once(layout_id)"));
    assert!(exact_route.contains("call_current_layout_once()"));
    assert!(exact_route.contains("verify_engine_once(ibus_engine)"));
    assert!(exact_route.contains("activate_gnome_layout_once(initial_layout_is_ru)"));
    assert!(!exact_route.contains("call_activate_layout(layout_id)"));
    assert!(!exact_route.contains("call_current_layout()"));
    assert!(!exact_route.contains("verify_with_retry"));
    assert!(!exact_route.contains("verify_gnome_layout_stack("));
    assert!(!exact_route.contains("switch_to_target_layout"));
    assert!(!exact_route.contains("reconcile"));
    assert!(!exact_route.contains("thread::sleep"));
}

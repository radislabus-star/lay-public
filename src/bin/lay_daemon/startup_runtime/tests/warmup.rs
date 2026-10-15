use super::*;

#[test]
fn late_permissions_listener_lifetime_advances_existing_epoch() {
    let epoch = Arc::new(AtomicU64::new(0));
    {
        let _listener = ListenerEpoch::enter(Arc::clone(&epoch));
        assert_eq!(epoch.load(Ordering::Acquire), 1);
    }
    assert_eq!(epoch.load(Ordering::Acquire), 2);
}

#[test]
fn warmup_plan_keeps_direct_ime_owner_out_of_daemon_memory() {
    let cfg = LayConfig {
        typing_assist: true,
        nanda_autocorrect: true,
        text_backend: "auto".to_string(),
        ..LayConfig::default()
    };

    let plan = runtime_warmup_plan(false, &cfg, None);

    assert!(!plan.spawn_background);
    assert!(!plan.warm_l11_service);
    assert!(!plan.warm_typing_assist);
    assert!(!plan.warm_l2_candidates);
    assert!(!plan.warm_l3_phrase);
}

#[test]
fn warmup_plan_can_warm_full_typing_heap_for_uinput_daemon() {
    let cfg = LayConfig {
        typing_assist: true,
        nanda_autocorrect: true,
        text_backend: "uinput".to_string(),
        ..LayConfig::default()
    };

    let plan = runtime_warmup_plan(false, &cfg, None);

    assert!(plan.spawn_background);
    assert!(plan.warm_l11_service);
    assert!(plan.warm_typing_assist);
    assert!(plan.warm_l2_candidates);
    assert!(plan.warm_l3_phrase);
}

#[test]
fn warmup_plan_does_not_wait_for_nanda_when_nanda_is_disabled() {
    let cfg = LayConfig {
        typing_assist: true,
        nanda_autocorrect: false,
        nanda_precognition: false,
        nanda_trace: false,
        text_backend: "uinput".to_string(),
        ..LayConfig::default()
    };

    let plan = runtime_warmup_plan(false, &cfg, None);

    assert!(plan.spawn_background);
    assert!(!plan.warm_l11_service);
    assert!(plan.warm_typing_assist);
    assert!(!plan.warm_l2_candidates);
    assert!(!plan.warm_l3_phrase);
}

#[test]
fn warmup_plan_keeps_detect_only_ready_without_background_thread() {
    let cfg = LayConfig {
        typing_assist: true,
        nanda_autocorrect: true,
        text_backend: "uinput".to_string(),
        ..LayConfig::default()
    };

    let plan = runtime_warmup_plan(true, &cfg, None);

    assert!(!plan.spawn_background);
    assert!(plan.warm_l11_service);
    assert!(plan.warm_typing_assist);
    assert!(plan.warm_l2_candidates);
    assert!(plan.warm_l3_phrase);
}

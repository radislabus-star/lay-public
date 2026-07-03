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

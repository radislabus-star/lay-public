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
            if plan.warm_typing_assist {
                lay::typing_assist::warm_up_hot();
            }
            TYPING_ASSIST_RUNTIME_READY.store(true, Ordering::Relaxed);
            if plan.warm_smart {
                match lay::llm::warm_up() {
                    Ok(()) => log("► smart engine: модель прогрета заранее"),
                    Err(e) => log(&format!("⚠ smart engine warmup failed: {e}")),
                }
            }
            log(&format!(
                "► hot typing runtime warmed in {}ms; cold lexicon/NANDA memory stays lazy",
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
    warm_typing_assist: bool,
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
    let daemon_can_own_full_hot_memory =
        lay::hot_field::HotFieldPolicy::daemon_for_text_backend(cfg.active_text_backend())
            .allows_full_reference_authority();
    let warm_typing_assist =
        daemon_can_own_full_hot_memory && (cfg.typing_assist || enter_autocorrect_active);
    let warm_nanda = daemon_can_own_full_hot_memory
        && (cfg.nanda_autocorrect || cfg.nanda_precognition || cfg.nanda_trace);
    RuntimeWarmupPlan {
        spawn_background: !detect_only && (warm_smart || warm_typing_assist),
        warm_typing_assist,
        warm_smart,
        warm_nanda,
    }
}

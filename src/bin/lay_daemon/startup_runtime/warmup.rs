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
            if plan.warm_l11_service {
                match lay::typing_cpu::TypingCpu::ensure_l11_service_started() {
                    Ok(Some(lay::typing_cpu::L11ServiceEnsureReport::Ready {
                        status,
                        package_path,
                        ..
                    })) => log(&format!(
                        "► L1.1 sidecar {status}: {}",
                        package_path.display()
                    )),
                    Ok(Some(lay::typing_cpu::L11ServiceEnsureReport::Reloaded {
                        package_path,
                        ..
                    })) => log(&format!(
                        "► L1.1 sidecar reloaded: {}",
                        package_path.display()
                    )),
                    Ok(Some(lay::typing_cpu::L11ServiceEnsureReport::Spawned {
                        package_path,
                        ..
                    })) => log(&format!(
                        "► L1.1 sidecar spawn started: {}",
                        package_path.display()
                    )),
                    Ok(None) => {}
                    Err(error) => log(&format!("⚠ L1.1 sidecar warmup failed: {error}")),
                }
            }
            if plan.warm_typing_assist {
                lay::typing_assist::warm_up();
            }
            if plan.warm_l2_candidates {
                lay::typing_cpu::TypingCpu::warm_l2_for_ime();
            }
            if plan.warm_l3_phrase {
                lay::typing_cpu::TypingCpu::warm_l3_phrase_memory();
            }
            TYPING_ASSIST_RUNTIME_READY.store(true, Ordering::Relaxed);
            if plan.warm_smart {
                match lay::llm::warm_up() {
                    Ok(()) => log("► smart engine: модель прогрета заранее"),
                    Err(e) => log(&format!("⚠ smart engine warmup failed: {e}")),
                }
            }
            log(&format!(
                "► boundary decision memory warmed in {}ms; input runtime ready",
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
    warm_l11_service: bool,
    warm_typing_assist: bool,
    warm_l2_candidates: bool,
    warm_smart: bool,
    warm_l3_phrase: bool,
}

fn runtime_warmup_plan(
    detect_only: bool,
    cfg: &LayConfig,
    enter_autocorrect_env: Option<&str>,
) -> RuntimeWarmupPlan {
    let warm_smart = cfg.active_correction_engine() == CorrectionEngine::Smart;
    let enter_autocorrect_active =
        active_enter_autocorrect_from_env(cfg.enter_autocorrect, enter_autocorrect_env);
    // The daemon owns the after-Space boundary decision even when IME owns
    // rendering. Its compact lookup state must be ready before the first word.
    let warm_typing_assist = cfg.typing_assist || enter_autocorrect_active;
    let warm_l11_service = cfg.nanda_autocorrect;
    let warm_l2_candidates = cfg.nanda_autocorrect;
    let warm_l3_phrase = cfg.nanda_autocorrect || cfg.nanda_trace;
    RuntimeWarmupPlan {
        spawn_background: !detect_only
            && (warm_smart
                || warm_l11_service
                || warm_typing_assist
                || warm_l2_candidates
                || warm_l3_phrase),
        warm_l11_service,
        warm_typing_assist,
        warm_l2_candidates,
        warm_smart,
        warm_l3_phrase,
    }
}

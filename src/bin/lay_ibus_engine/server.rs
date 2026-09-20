use std::sync::{mpsc, Arc, Mutex};
use std::time::{Duration, Instant};

use zbus::connection;

use super::args::Args;
use super::bridge::LayImeBridge;
use super::context_admission::{
    AdapterConfig, ConnectionGeneration, EngineProfile, PendingContextAdapter,
};
use super::factory::LayIbusFactory;
use super::protocol::{SharedState, BUS_NAME, BUS_PATH, IBUS_ENGINE_NAME, IBUS_FACTORY_PATH};

const STARTUP_WARMUP_BUDGET: Duration = Duration::from_secs(5);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum StartupWarmupCompletion {
    Exact {
        available: bool,
    },
    L2 {
        available: bool,
        candidate_ready: bool,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct StartupWarmupReceipt {
    exact_available: bool,
    // These two fields describe the lexical candidate memory consumed by the
    // IME. They do not certify every canonical/productive L2 package.
    l2_available: bool,
    l2_candidate_ready: bool,
}

fn missing_startup_warmups(exact_available: Option<bool>, l2_complete: bool) -> String {
    let mut missing = Vec::new();
    if exact_available.is_none() {
        missing.push("exact");
    }
    if !l2_complete {
        missing.push("l2");
    }
    missing.join(",")
}

fn startup_warmup_timeout(exact_available: Option<bool>, l2_complete: bool) -> String {
    let missing = missing_startup_warmups(exact_available, l2_complete);
    if missing.is_empty() {
        "startup warmup exceeded 5000 ms before publication".to_string()
    } else {
        format!("startup warmup exceeded 5000 ms; incomplete: {missing}")
    }
}

fn collect_startup_warmups(
    receiver: mpsc::Receiver<StartupWarmupCompletion>,
    deadline: Instant,
) -> Result<StartupWarmupReceipt, String> {
    collect_startup_warmups_with_now(receiver, deadline, Instant::now)
}

fn collect_startup_warmups_with_now<Now>(
    receiver: mpsc::Receiver<StartupWarmupCompletion>,
    deadline: Instant,
    mut now: Now,
) -> Result<StartupWarmupReceipt, String>
where
    Now: FnMut() -> Instant,
{
    let mut exact_available = None;
    let mut l2_completion = None;
    while exact_available.is_none() || l2_completion.is_none() {
        let remaining = deadline
            .checked_duration_since(now())
            .ok_or_else(|| startup_warmup_timeout(exact_available, l2_completion.is_some()))?;
        match receiver.recv_timeout(remaining) {
            Ok(StartupWarmupCompletion::Exact { available }) => {
                if exact_available.replace(available).is_some() {
                    return Err("duplicate exact startup completion".to_string());
                }
            }
            Ok(StartupWarmupCompletion::L2 {
                available,
                candidate_ready,
            }) => {
                if l2_completion
                    .replace((available, candidate_ready))
                    .is_some()
                {
                    return Err("duplicate l2 startup completion".to_string());
                }
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {
                return Err(startup_warmup_timeout(
                    exact_available,
                    l2_completion.is_some(),
                ));
            }
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                return Err(format!(
                    "startup warmup worker stopped; incomplete: {}",
                    missing_startup_warmups(exact_available, l2_completion.is_some()),
                ));
            }
        }
        if now() > deadline {
            return Err(startup_warmup_timeout(
                exact_available,
                l2_completion.is_some(),
            ));
        }
    }
    let (l2_available, l2_candidate_ready) = l2_completion.expect("L2 completion checked");
    Ok(StartupWarmupReceipt {
        exact_available: exact_available.expect("exact completion checked"),
        l2_available,
        l2_candidate_ready,
    })
}

fn complete_startup_warmup(nanda_autocorrect: bool) -> zbus::Result<()> {
    let started = Instant::now();
    let deadline = started + STARTUP_WARMUP_BUDGET;
    let (sender, receiver) = mpsc::channel();

    let exact_sender = sender.clone();
    std::thread::Builder::new()
        .name("lay-exact-layout-warmup".to_string())
        .spawn(move || {
            let exact_started = Instant::now();
            super::trace::record(r#"{"kind":"ibus_exact_authority_warmup","stage":"started"}"#);
            let receipt = lay::exact_layout_authority::warm_up_exact_layout_authority_for_ibus();
            if super::trace::enabled() {
                super::trace::record(format!(
                    r#"{{"kind":"ibus_exact_authority_warmup","stage":"completed","available":{},"elapsed_us":{}}}"#,
                    receipt.is_some(),
                    exact_started.elapsed().as_micros(),
                ));
            }
            let _ = exact_sender.send(StartupWarmupCompletion::Exact {
                available: receipt.is_some(),
            });
        })
        .map_err(|error| zbus::Error::Failure(format!("exact startup worker: {error}")))?;

    let l2_sender = sender.clone();
    std::thread::Builder::new()
        .name("lay-l2-ime-startup".to_string())
        .spawn(move || {
            // Keep the lifecycle's synchronous L2 fallback inside this bounded
            // worker rather than blocking the async server executor.
            lay::typing_cpu::TypingCpu::ensure_ime_runtime_warmup_started(nanda_autocorrect);
            let available = lay::typing_cpu::TypingCpu::warm_l2_for_ime();
            let candidate_ready = lay::typing_cpu::TypingCpu::ime_candidate_memory_is_warm();
            let _ = l2_sender.send(StartupWarmupCompletion::L2 {
                available,
                candidate_ready,
            });
        })
        .map_err(|error| zbus::Error::Failure(format!("l2 startup worker: {error}")))?;
    drop(sender);

    // This bounded pre-publication wait blocks the server task only. zbus runs
    // its internal connection executor on a separate thread, so admission
    // observation remains live while the factory is intentionally absent.
    let receipt = collect_startup_warmups(receiver, deadline).map_err(zbus::Error::Failure)?;
    if super::trace::enabled() {
        super::trace::record(format!(
            r#"{{"kind":"ibus_startup_warmup","stage":"completed","exact_available":{},"l2_complete":true,"l2_available":{},"l2_candidate_ready":{},"elapsed_us":{}}}"#,
            receipt.exact_available,
            receipt.l2_available,
            receipt.l2_candidate_ready,
            started.elapsed().as_micros(),
        ));
    }
    Ok(())
}

pub(crate) async fn run(args: &Args) -> zbus::Result<()> {
    let shared = Arc::new(Mutex::new(SharedState::default()));
    let managed_input = managed_input_enabled(args);
    let startup_config = lay::config::LayConfig::load();
    let ibus_connection = connection::Builder::ibus()?.build().await?;
    let admission_config = AdapterConfig::new(
        ConnectionGeneration(1),
        ["lay-ime-us", "lay-ime-ru"]
            .into_iter()
            .filter_map(EngineProfile::new)
            .collect(),
    )
    .map_err(|error| zbus::Error::Failure(error.to_string()))?;
    let admission = match PendingContextAdapter::subscribe(
        ibus_connection.clone(),
        admission_config,
    )
    .await
    {
        Ok(pending) => match pending.bootstrap().await {
            Ok((admission, observer, _bootstrap_identity)) => {
                ibus_connection
                    .executor()
                    .spawn(
                        async move {
                            if let Err(error) = observer.run().await {
                                super::trace::record(format!(
                                    r#"{{"kind":"ibus_context_admission_observer","status":"stopped","error":{:?}}}"#,
                                    error.to_string()
                                ));
                            }
                        },
                        "lay-context-admission-observer",
                    )
                    .detach();
                Some(admission)
            }
            Err(error) => {
                super::trace::record(format!(
                    r#"{{"kind":"ibus_context_admission","status":"bootstrap_unavailable","error":{:?}}}"#,
                    error.to_string()
                ));
                None
            }
        },
        Err(error) => {
            super::trace::record(format!(
                r#"{{"kind":"ibus_context_admission","status":"subscribe_unavailable","error":{:?}}}"#,
                error.to_string()
            ));
            None
        }
    };
    super::space_autocorrect_prefetch::initialize();
    complete_startup_warmup(startup_config.nanda_autocorrect)?;
    ibus_connection
        .object_server()
        .at(
            IBUS_FACTORY_PATH,
            LayIbusFactory {
                ibus_connection: ibus_connection.clone(),
                shared: Arc::clone(&shared),
                admission: admission.clone(),
                managed_input,
            },
        )
        .await?;
    ibus_connection.request_name(IBUS_ENGINE_NAME).await?;

    let _session_connection = connection::Builder::session()?
        .serve_at(
            BUS_PATH,
            LayImeBridge {
                ibus_connection,
                shared,
                context_admission_required: true,
                admission,
            },
        )?
        .name(BUS_NAME)?
        .allow_name_replacements(true)
        .replace_existing_names(true)
        .build()
        .await?;

    std::future::pending::<()>().await;
    Ok(())
}

fn managed_input_enabled(args: &Args) -> bool {
    if args.managed {
        return true;
    }
    std::env::var("LAY_IME_MANAGED")
        .ok()
        .is_some_and(|value| matches!(value.as_str(), "1" | "true" | "yes" | "on"))
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;
    use std::sync::mpsc;
    use std::time::{Duration, Instant};

    use super::{
        collect_startup_warmups, collect_startup_warmups_with_now, StartupWarmupCompletion,
    };

    #[test]
    fn startup_gate_accepts_completed_unavailable_exact_capability() {
        let (sender, receiver) = mpsc::channel();
        sender
            .send(StartupWarmupCompletion::L2 {
                available: false,
                candidate_ready: false,
            })
            .unwrap();
        sender
            .send(StartupWarmupCompletion::Exact { available: false })
            .unwrap();
        drop(sender);

        let receipt = collect_startup_warmups(receiver, Instant::now() + Duration::from_secs(1))
            .expect("completed unavailable capability still permits publication");

        assert!(!receipt.exact_available);
        assert!(!receipt.l2_available);
        assert!(!receipt.l2_candidate_ready);
    }

    #[test]
    fn startup_gate_reports_worker_loss_before_publication() {
        let (sender, receiver) = mpsc::channel();
        sender
            .send(StartupWarmupCompletion::Exact { available: true })
            .unwrap();
        drop(sender);

        let error = collect_startup_warmups(receiver, Instant::now() + Duration::from_secs(1))
            .expect_err("missing L2 completion must fail closed");

        assert_eq!(error, "startup warmup worker stopped; incomplete: l2");
    }

    #[test]
    fn startup_gate_uses_one_absolute_deadline_after_partial_completion() {
        let (sender, receiver) = mpsc::channel();
        sender
            .send(StartupWarmupCompletion::Exact { available: true })
            .unwrap();
        let base = Instant::now();
        let calls = Cell::new(0_u8);

        let error =
            collect_startup_warmups_with_now(receiver, base + Duration::from_secs(1), || {
                let call = calls.get();
                calls.set(call + 1);
                if call == 0 {
                    base
                } else {
                    base + Duration::from_secs(2)
                }
            })
            .expect_err("the deadline must not reset after exact completion");

        assert_eq!(error, "startup warmup exceeded 5000 ms; incomplete: l2");
    }

    #[test]
    fn startup_gate_rejects_conflicting_worker_completion() {
        let (sender, receiver) = mpsc::channel();
        sender
            .send(StartupWarmupCompletion::Exact { available: true })
            .unwrap();
        sender
            .send(StartupWarmupCompletion::Exact { available: false })
            .unwrap();
        sender
            .send(StartupWarmupCompletion::L2 {
                available: true,
                candidate_ready: true,
            })
            .unwrap();

        let error = collect_startup_warmups(receiver, Instant::now() + Duration::from_secs(1))
            .expect_err("duplicate worker completion must fail closed");

        assert_eq!(error, "duplicate exact startup completion");
    }

    #[test]
    fn startup_gate_rejects_completion_observed_after_deadline() {
        let (sender, receiver) = mpsc::channel();
        sender
            .send(StartupWarmupCompletion::Exact { available: true })
            .unwrap();
        sender
            .send(StartupWarmupCompletion::L2 {
                available: true,
                candidate_ready: true,
            })
            .unwrap();
        let base = Instant::now();
        let calls = Cell::new(0_u8);

        let error =
            collect_startup_warmups_with_now(receiver, base + Duration::from_secs(1), || {
                let call = calls.get();
                calls.set(call + 1);
                if call < 3 {
                    base
                } else {
                    base + Duration::from_secs(2)
                }
            })
            .expect_err("late final completion must not publish readiness");

        assert_eq!(error, "startup warmup exceeded 5000 ms before publication");
    }
}

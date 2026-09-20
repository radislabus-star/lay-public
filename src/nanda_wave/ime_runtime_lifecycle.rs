use std::{
    io::{self, Write},
    sync::atomic::{AtomicBool, Ordering},
    thread,
};

use super::L11ServiceEnsureReport;

const LIFECYCLE_EVENT: &str = "ime_l11_lifecycle_startup";

static IME_L11_LIFECYCLE_STARTED: AtomicBool = AtomicBool::new(false);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ImeL11LifecycleClaim {
    Disabled,
    Start,
    AlreadyStarted,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ImeL11LifecycleStatus {
    Disabled,
    Absent,
    Error,
    Spawned,
    Ready,
    Warming,
    Reloaded,
}

impl ImeL11LifecycleStatus {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Disabled => "disabled",
            Self::Absent => "absent",
            Self::Error => "error",
            Self::Spawned => "spawned",
            Self::Ready => "ready",
            Self::Warming => "warming",
            Self::Reloaded => "reloaded",
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
struct ImeL11LifecycleObservation {
    status: ImeL11LifecycleStatus,
    detail: Option<String>,
}

impl ImeL11LifecycleObservation {
    const fn status(status: ImeL11LifecycleStatus) -> Self {
        Self {
            status,
            detail: None,
        }
    }

    fn error(detail: impl Into<String>) -> Self {
        Self {
            status: ImeL11LifecycleStatus::Error,
            detail: Some(detail.into()),
        }
    }
}

fn claim_lifecycle(enabled: bool, gate: &AtomicBool) -> ImeL11LifecycleClaim {
    if !enabled {
        return ImeL11LifecycleClaim::Disabled;
    }
    if gate
        .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
        .is_ok()
    {
        ImeL11LifecycleClaim::Start
    } else {
        ImeL11LifecycleClaim::AlreadyStarted
    }
}

fn classify_ensure_result(
    result: io::Result<Option<L11ServiceEnsureReport>>,
) -> ImeL11LifecycleObservation {
    match result {
        Ok(None) => ImeL11LifecycleObservation::status(ImeL11LifecycleStatus::Absent),
        Err(error) => ImeL11LifecycleObservation::error(error.to_string()),
        Ok(Some(L11ServiceEnsureReport::Spawned { .. })) => {
            ImeL11LifecycleObservation::status(ImeL11LifecycleStatus::Spawned)
        }
        Ok(Some(L11ServiceEnsureReport::Reloaded { .. })) => {
            ImeL11LifecycleObservation::status(ImeL11LifecycleStatus::Reloaded)
        }
        Ok(Some(L11ServiceEnsureReport::Ready { status, .. })) => match status.as_str() {
            "ready" => ImeL11LifecycleObservation::status(ImeL11LifecycleStatus::Ready),
            "warming" => ImeL11LifecycleObservation::status(ImeL11LifecycleStatus::Warming),
            other => ImeL11LifecycleObservation::error(format!(
                "unexpected L1.1 ensure status {other:?}"
            )),
        },
    }
}

fn format_observation(observation: &ImeL11LifecycleObservation) -> String {
    let status = observation.status.as_str();
    match observation.detail.as_deref() {
        Some(detail) => format!("event={LIFECYCLE_EVENT} status={status} detail={detail:?}"),
        None => format!("event={LIFECYCLE_EVENT} status={status}"),
    }
}

fn log_observation(observation: ImeL11LifecycleObservation) {
    let line = format_observation(&observation);
    let _ = writeln!(io::stderr().lock(), "[lay] {line}");
}

fn run_enabled_lifecycle_with<Ensure, Observe, WarmL2>(
    ensure_l11: Ensure,
    observe_status: Observe,
    warm_l2: WarmL2,
) where
    Ensure: FnOnce() -> io::Result<Option<L11ServiceEnsureReport>>,
    Observe: FnOnce(ImeL11LifecycleObservation),
    WarmL2: FnOnce(),
{
    let observation = classify_ensure_result(ensure_l11());
    observe_status(observation);
    warm_l2();
}

fn dispatch_lifecycle_with<Start, Observe, RequestL2, WarmL2OnThreadFailure>(
    enabled: bool,
    gate: &AtomicBool,
    start_background: Start,
    observe_status: Observe,
    request_l2_warmup: RequestL2,
    warm_l2_on_thread_failure: WarmL2OnThreadFailure,
) where
    Start: FnOnce() -> io::Result<()>,
    Observe: FnOnce(ImeL11LifecycleObservation),
    RequestL2: FnOnce(),
    WarmL2OnThreadFailure: FnOnce(),
{
    match claim_lifecycle(enabled, gate) {
        ImeL11LifecycleClaim::Disabled => {
            observe_status(ImeL11LifecycleObservation::status(
                ImeL11LifecycleStatus::Disabled,
            ));
            request_l2_warmup();
        }
        ImeL11LifecycleClaim::Start => {
            if let Err(error) = start_background() {
                observe_status(ImeL11LifecycleObservation::error(format!(
                    "background thread start failed: {error}"
                )));
                warm_l2_on_thread_failure();
            }
        }
        ImeL11LifecycleClaim::AlreadyStarted => {}
    }
}

pub fn ensure_ime_runtime_warmup_started(nanda_autocorrect: bool) {
    dispatch_lifecycle_with(
        nanda_autocorrect,
        &IME_L11_LIFECYCLE_STARTED,
        || {
            thread::Builder::new()
                .name("lay-ime-l11-lifecycle".to_string())
                .spawn(|| {
                    run_enabled_lifecycle_with(
                        super::ensure_l11_service_started,
                        log_observation,
                        || {
                            super::warm_up_l2_for_ime();
                        },
                    );
                })
                .map(|_| ())
        },
        log_observation,
        super::ensure_l2_ime_warmup_started,
        // Thread creation already failed, so do not delegate the fallback to
        // another thread. This warmup is process-free and runs only on this
        // exceptional startup branch.
        || {
            super::warm_up_l2_for_ime();
        },
    );
}

#[cfg(test)]
mod tests {
    use std::{cell::RefCell, io, path::PathBuf, sync::atomic::AtomicBool};

    use super::{
        claim_lifecycle, classify_ensure_result, dispatch_lifecycle_with, format_observation,
        run_enabled_lifecycle_with, ImeL11LifecycleClaim, ImeL11LifecycleObservation,
        ImeL11LifecycleStatus, L11ServiceEnsureReport,
    };

    #[test]
    fn enabled_lifecycle_orders_ensure_observation_before_l2_warmup() {
        let trace = RefCell::new(Vec::new());

        run_enabled_lifecycle_with(
            || {
                trace.borrow_mut().push("ensure_l11");
                Ok(None)
            },
            |_| trace.borrow_mut().push("observe_status"),
            || trace.borrow_mut().push("warm_l2"),
        );

        assert_eq!(
            trace.into_inner(),
            ["ensure_l11", "observe_status", "warm_l2"]
        );
    }

    #[test]
    fn disabled_lifecycle_skips_start_warms_l2_and_does_not_claim_gate() {
        let gate = AtomicBool::new(false);
        let trace = RefCell::new(Vec::new());

        dispatch_lifecycle_with(
            false,
            &gate,
            || {
                trace.borrow_mut().push("start_background");
                Ok(())
            },
            |observation| trace.borrow_mut().push(observation.status.as_str()),
            || trace.borrow_mut().push("request_l2_warmup"),
            || panic!("disabled startup must not take the thread-failure fallback"),
        );

        assert_eq!(trace.into_inner(), ["disabled", "request_l2_warmup"]);
        assert_eq!(claim_lifecycle(true, &gate), ImeL11LifecycleClaim::Start);
    }

    #[test]
    fn repeated_enabled_signal_starts_background_once() {
        let gate = AtomicBool::new(false);
        let starts = RefCell::new(0_u8);

        for _ in 0..2 {
            dispatch_lifecycle_with(
                true,
                &gate,
                || {
                    *starts.borrow_mut() += 1;
                    Ok(())
                },
                |_| panic!("successful dispatch must observe in the background"),
                || panic!("enabled dispatch must not request disabled L2 warmup"),
                || panic!("successful dispatch must warm in the background"),
            );
        }

        assert_eq!(*starts.borrow(), 1);
        assert_eq!(
            claim_lifecycle(true, &gate),
            ImeL11LifecycleClaim::AlreadyStarted
        );
    }

    #[test]
    fn ensure_error_is_observed_before_l2_still_warms() {
        let trace = RefCell::new(Vec::new());

        run_enabled_lifecycle_with(
            || {
                trace.borrow_mut().push("ensure_l11");
                Err(io::Error::other("fixture ensure failure"))
            },
            |observation| {
                assert_eq!(observation.status, ImeL11LifecycleStatus::Error);
                trace.borrow_mut().push("observe_error");
            },
            || trace.borrow_mut().push("warm_l2"),
        );

        assert_eq!(
            trace.into_inner(),
            ["ensure_l11", "observe_error", "warm_l2"]
        );
    }

    #[test]
    fn background_thread_start_error_is_observed_and_l2_still_warms() {
        let gate = AtomicBool::new(false);
        let trace = RefCell::new(Vec::new());

        dispatch_lifecycle_with(
            true,
            &gate,
            || Err(io::Error::other("fixture thread failure")),
            |observation| {
                assert_eq!(observation.status, ImeL11LifecycleStatus::Error);
                trace.borrow_mut().push("observe_error");
            },
            || panic!("thread failure must not take the disabled L2 route"),
            || trace.borrow_mut().push("warm_l2"),
        );

        assert_eq!(trace.into_inner(), ["observe_error", "warm_l2"]);
        assert_eq!(
            claim_lifecycle(true, &gate),
            ImeL11LifecycleClaim::AlreadyStarted
        );
    }

    #[test]
    fn ensure_results_have_stable_distinct_statuses() {
        let fixture_path = || PathBuf::from("/fixture");
        let cases = [
            (Ok(None), ImeL11LifecycleStatus::Absent),
            (
                Ok(Some(L11ServiceEnsureReport::Spawned {
                    binary: fixture_path(),
                    socket: fixture_path(),
                    package_path: fixture_path(),
                })),
                ImeL11LifecycleStatus::Spawned,
            ),
            (
                Ok(Some(L11ServiceEnsureReport::Ready {
                    socket: fixture_path(),
                    package_path: fixture_path(),
                    status: "ready".to_string(),
                })),
                ImeL11LifecycleStatus::Ready,
            ),
            (
                Ok(Some(L11ServiceEnsureReport::Ready {
                    socket: fixture_path(),
                    package_path: fixture_path(),
                    status: "warming".to_string(),
                })),
                ImeL11LifecycleStatus::Warming,
            ),
            (
                Ok(Some(L11ServiceEnsureReport::Reloaded {
                    socket: fixture_path(),
                    package_path: fixture_path(),
                })),
                ImeL11LifecycleStatus::Reloaded,
            ),
        ];

        for (result, expected) in cases {
            assert_eq!(classify_ensure_result(result).status, expected);
        }

        assert_eq!(
            classify_ensure_result(Err(io::Error::other("fixture"))).status,
            ImeL11LifecycleStatus::Error
        );
        assert_eq!(
            classify_ensure_result(Ok(Some(L11ServiceEnsureReport::Ready {
                socket: fixture_path(),
                package_path: fixture_path(),
                status: "unknown".to_string(),
            })))
            .status,
            ImeL11LifecycleStatus::Error
        );
    }

    #[test]
    fn lifecycle_observation_has_one_stable_event_name() {
        assert_eq!(
            format_observation(&ImeL11LifecycleObservation::status(
                ImeL11LifecycleStatus::Warming
            )),
            "event=ime_l11_lifecycle_startup status=warming"
        );
        assert_eq!(
            format_observation(&ImeL11LifecycleObservation::error("fixture")),
            "event=ime_l11_lifecycle_startup status=error detail=\"fixture\""
        );
    }

    #[test]
    fn direct_l2_warmup_cannot_consume_the_distinct_lifecycle_gate() {
        super::super::warm_up_l2_for_ime();
        let lifecycle_gate = AtomicBool::new(false);

        assert_eq!(
            claim_lifecycle(true, &lifecycle_gate),
            ImeL11LifecycleClaim::Start
        );
    }
}

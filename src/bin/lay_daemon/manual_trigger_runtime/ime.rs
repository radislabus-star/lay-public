use super::super::{
    active_text_backend, capture_ime_committed_tail_replay, log, try_ime_manual_toggle,
    ImeCommittedTailReplay, ManualCorrectionDispatchPlan, ManualCorrectionInputIsolation,
    ManualCorrectionOutputRoute,
};
use lay::manual_toggle::ImeManualToggleOutcome;
use lay::word_buffer::WordBuffer;

#[derive(Debug, Clone, PartialEq, Eq)]
#[expect(
    clippy::large_enum_variant,
    reason = "boxing would change the bounded manual-toggle dispatch"
)]
pub(crate) enum ImeManualToggleDispatch {
    DelegateDaemon(ManualCorrectionDispatchPlan),
    ReplayExactImeTail(ImeCommittedTailReplay),
    RejectExactImeTailCapture,
    Complete(Option<bool>),
}

fn daemon_manual_toggle_plan(
    pending_auto_undo_ready: bool,
    should_try_ime: bool,
) -> Option<ManualCorrectionDispatchPlan> {
    if pending_auto_undo_ready {
        return Some(ManualCorrectionDispatchPlan {
            output_route: ManualCorrectionOutputRoute::DaemonUinput,
            input_isolation: ManualCorrectionInputIsolation::PhysicalGrab,
        });
    }
    if !should_try_ime {
        return Some(ManualCorrectionDispatchPlan {
            output_route: ManualCorrectionOutputRoute::ConfiguredBackend,
            input_isolation: ManualCorrectionInputIsolation::PhysicalGrab,
        });
    }
    None
}

pub(crate) fn dispatch_ime_manual_toggle(buffer: &mut WordBuffer) -> ImeManualToggleDispatch {
    if let Some(plan) = daemon_manual_toggle_plan(
        buffer.pending_auto_undo_ready(),
        active_text_backend().should_try_ime(),
    ) {
        return ImeManualToggleDispatch::DelegateDaemon(plan);
    }
    run_ime_manual_toggle()
}

fn run_ime_manual_toggle() -> ImeManualToggleDispatch {
    match try_ime_manual_toggle() {
        Ok(ImeManualToggleOutcome::Handled {
            target_layout_is_ru,
        }) => {
            log("· physical manual trigger handled by focused IME engine");
            ImeManualToggleDispatch::Complete(Some(target_layout_is_ru))
        }
        Ok(ImeManualToggleOutcome::DelegateDaemon) => {
            log("· physical manual trigger delegated to daemon WordBuffer");
            ImeManualToggleDispatch::DelegateDaemon(ManualCorrectionDispatchPlan {
                output_route: ManualCorrectionOutputRoute::ConfiguredBackend,
                input_isolation: ManualCorrectionInputIsolation::Shared,
            })
        }
        Ok(ImeManualToggleOutcome::DelegateExactImeTail) => {
            match capture_ime_committed_tail_replay() {
                Ok(replay) => {
                    log("· physical manual trigger delegated with exact IME committed tail");
                    ImeManualToggleDispatch::ReplayExactImeTail(replay)
                }
                Err(error) => {
                    log(&format!(
                        "⚠ exact IME committed-tail capture failed before replay: {error}; identity-free cancellation forbidden, bounded handoff will expire"
                    ));
                    ImeManualToggleDispatch::RejectExactImeTailCapture
                }
            }
        }
        Ok(ImeManualToggleOutcome::NotHandled) => {
            log("· physical manual trigger blocked by focused IME owner");
            ImeManualToggleDispatch::Complete(None)
        }
        Err(error) => {
            log(&format!(
                "⚠ IME physical manual trigger failed: {error}; identity-free cancellation forbidden"
            ));
            ImeManualToggleDispatch::Complete(None)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn daemon_pending_auto_undo_keeps_physical_grab_ownership_without_consuming_it() {
        assert_eq!(
            daemon_manual_toggle_plan(false, false),
            Some(ManualCorrectionDispatchPlan {
                output_route: ManualCorrectionOutputRoute::ConfiguredBackend,
                input_isolation: ManualCorrectionInputIsolation::PhysicalGrab,
            })
        );
        assert_eq!(daemon_manual_toggle_plan(false, true), None);
        assert_eq!(
            daemon_manual_toggle_plan(true, false),
            Some(ManualCorrectionDispatchPlan {
                output_route: ManualCorrectionOutputRoute::DaemonUinput,
                input_isolation: ManualCorrectionInputIsolation::PhysicalGrab,
            })
        );
        assert_eq!(
            daemon_manual_toggle_plan(true, true),
            Some(ManualCorrectionDispatchPlan {
                output_route: ManualCorrectionOutputRoute::DaemonUinput,
                input_isolation: ManualCorrectionInputIsolation::PhysicalGrab,
            })
        );
        let mut buffer = WordBuffer::new();
        buffer.remember_pending_auto_undo("typing-assist", "посмотри", "посмотреть", 1, 1);

        assert!(buffer.pending_auto_undo_ready());
        assert_eq!(
            dispatch_ime_manual_toggle(&mut buffer),
            ImeManualToggleDispatch::DelegateDaemon(ManualCorrectionDispatchPlan {
                output_route: ManualCorrectionOutputRoute::DaemonUinput,
                input_isolation: ManualCorrectionInputIsolation::PhysicalGrab,
            })
        );
        assert!(buffer.take_pending_auto_undo().is_some());
    }

    #[test]
    fn exact_tail_disposition_cannot_be_downgraded_to_a_completed_noop() {
        let source = include_str!("ime.rs");
        let production = source.split("#[cfg(test)]").next().expect("production");
        let exact_arm = production
            .split("Ok(ImeManualToggleOutcome::DelegateExactImeTail)")
            .nth(1)
            .expect("exact-tail arm")
            .split("Ok(ImeManualToggleOutcome::NotHandled)")
            .next()
            .expect("next arm");

        assert!(exact_arm.contains("capture_ime_committed_tail_replay()"));
        assert!(exact_arm.contains("ReplayExactImeTail(replay)"));
        assert!(exact_arm.contains("RejectExactImeTailCapture"));
        assert!(!exact_arm.contains("DelegateDaemon"));
        assert!(!exact_arm.contains("Complete(None)"));
    }
}

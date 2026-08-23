use super::super::{active_text_backend, log, try_ime_manual_toggle, ManualCorrectionOutputRoute};
use lay::manual_toggle::ImeManualToggleOutcome;
use lay::word_buffer::WordBuffer;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ImeManualToggleDispatch {
    DelegateDaemon(ManualCorrectionOutputRoute),
    Complete(Option<bool>),
}

pub(crate) fn dispatch_ime_manual_toggle(buffer: &mut WordBuffer) -> ImeManualToggleDispatch {
    if buffer.pending_auto_undo_ready() || !active_text_backend().should_try_ime() {
        return ImeManualToggleDispatch::DelegateDaemon(
            ManualCorrectionOutputRoute::ConfiguredBackend,
        );
    }
    run_ime_manual_toggle()
}

fn run_ime_manual_toggle() -> ImeManualToggleDispatch {
    match try_ime_manual_toggle() {
        Ok(outcome @ ImeManualToggleOutcome::Handled { .. }) => {
            log("· physical manual trigger handled by focused IME engine");
            dispatch_from_result(Ok(outcome))
        }
        Ok(ImeManualToggleOutcome::DelegateDaemon) => {
            log("· physical manual trigger delegated to daemon WordBuffer");
            dispatch_from_result(Ok(ImeManualToggleOutcome::DelegateDaemon))
        }
        Ok(ImeManualToggleOutcome::NotHandled) => {
            log("· physical manual trigger blocked by focused IME owner");
            dispatch_from_result(Ok(ImeManualToggleOutcome::NotHandled))
        }
        Err(error) => {
            log(&format!("⚠ IME physical manual trigger failed: {error}"));
            dispatch_from_result(Err(error))
        }
    }
}

fn dispatch_from_result(result: Result<ImeManualToggleOutcome, String>) -> ImeManualToggleDispatch {
    match result {
        Ok(outcome) => dispatch_from_outcome(outcome),
        Err(_) => ImeManualToggleDispatch::Complete(None),
    }
}

fn dispatch_from_outcome(outcome: ImeManualToggleOutcome) -> ImeManualToggleDispatch {
    match outcome {
        ImeManualToggleOutcome::DelegateDaemon => {
            ImeManualToggleDispatch::DelegateDaemon(ManualCorrectionOutputRoute::DaemonUinput)
        }
        ImeManualToggleOutcome::NotHandled => ImeManualToggleDispatch::Complete(None),
        ImeManualToggleOutcome::Handled {
            target_layout_is_ru,
        } => ImeManualToggleDispatch::Complete(Some(target_layout_is_ru)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn daemon_pending_auto_undo_keeps_precedence_without_consuming_it() {
        let mut buffer = WordBuffer::new();
        buffer.remember_pending_auto_undo("typing-assist", "посмотри", "посмотреть", 1, 1);

        assert!(buffer.pending_auto_undo_ready());
        assert_eq!(
            dispatch_ime_manual_toggle(&mut buffer),
            ImeManualToggleDispatch::DelegateDaemon(ManualCorrectionOutputRoute::ConfiguredBackend)
        );
        assert!(buffer.take_pending_auto_undo().is_some());
    }

    #[test]
    fn ime_dispatch_shape_delegates_only_the_explicit_daemon_outcome() {
        assert_eq!(
            dispatch_from_outcome(ImeManualToggleOutcome::DelegateDaemon),
            ImeManualToggleDispatch::DelegateDaemon(ManualCorrectionOutputRoute::DaemonUinput)
        );
        assert_eq!(
            dispatch_from_outcome(ImeManualToggleOutcome::NotHandled),
            ImeManualToggleDispatch::Complete(None)
        );
        assert_eq!(
            dispatch_from_outcome(ImeManualToggleOutcome::handled(true)),
            ImeManualToggleDispatch::Complete(Some(true))
        );
        assert_eq!(
            dispatch_from_result(Err("malformed ManualToggleV3 reply".to_string())),
            ImeManualToggleDispatch::Complete(None)
        );
    }
}

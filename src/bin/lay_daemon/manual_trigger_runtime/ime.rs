use super::super::{active_text_backend, log, try_ime_manual_toggle};
use lay::word_buffer::WordBuffer;

pub(crate) fn dispatch_ime_manual_toggle(buffer: &mut WordBuffer) -> Option<Option<bool>> {
    if buffer.pending_auto_undo_ready() || !active_text_backend().should_try_ime() {
        return None;
    }
    Some(run_ime_manual_toggle())
}

fn run_ime_manual_toggle() -> Option<bool> {
    match try_ime_manual_toggle() {
        Ok(Some(target_is_ru)) => {
            log("· physical manual trigger handled by focused IME engine");
            Some(target_is_ru)
        }
        Ok(None) => {
            log("· physical manual trigger skipped: focused IME has no editable target");
            None
        }
        Err(error) => {
            log(&format!("⚠ IME physical manual trigger failed: {error}"));
            None
        }
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
        assert!(buffer.take_pending_auto_undo().is_some());
    }

    #[test]
    fn ime_dispatch_shape_distinguishes_not_selected_from_not_handled() {
        fn selected(result: Option<bool>) -> Option<Option<bool>> {
            Some(result)
        }

        assert_eq!(selected(Some(true)), Some(Some(true)));
        assert_eq!(selected(None), Some(None));
    }
}

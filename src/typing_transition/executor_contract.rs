//! Contract between transition authority and output backends.
//!
//! Daemon, IME, clipboard, and tray code may execute a verified edit plan, but
//! they must not decide that a text replacement is true.

use crate::text_edit::EditAction;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ExecutionBackend {
    Daemon,
    Ime,
    Clipboard,
    TrayStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ExecutorContract {
    pub(crate) backend: ExecutionBackend,
    pub(crate) may_decide_apply: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ExecutorAuthorization {
    pub(crate) backend: ExecutionBackend,
    pub(crate) allow_execute: bool,
    pub(crate) reason: &'static str,
}

impl ExecutorContract {
    pub(crate) const fn backend_only(backend: ExecutionBackend) -> Self {
        Self {
            backend,
            may_decide_apply: false,
        }
    }

    pub(crate) fn authorize_edit(self, action: &EditAction) -> ExecutorAuthorization {
        if self.may_decide_apply {
            return ExecutorAuthorization::blocked(self.backend, "backend_may_not_decide_apply");
        }
        if let Some(reason) = action.execution_rejection_reason() {
            return ExecutorAuthorization::blocked(self.backend, reason);
        }
        ExecutorAuthorization {
            backend: self.backend,
            allow_execute: true,
            reason: "verified_transition_authority",
        }
    }
}

impl ExecutorAuthorization {
    const fn blocked(backend: ExecutionBackend, reason: &'static str) -> Self {
        Self {
            backend,
            allow_execute: false,
            reason,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{ExecutionBackend, ExecutorContract};
    use crate::text_edit::{EditAction, TextReplacement};

    #[test]
    fn ime_is_backend_not_authority() {
        let contract = ExecutorContract::backend_only(ExecutionBackend::Ime);
        assert!(!contract.may_decide_apply);
    }

    #[test]
    fn backend_can_execute_verified_action() {
        let action = EditAction::planned_replacement(
            "test",
            900,
            "ghbdtn".to_string(),
            "привет".to_string(),
            TextReplacement {
                move_left: 0,
                backspaces: 6,
                insert: "привет".to_string(),
                move_right: 0,
            },
            Some("layout"),
            Some("layout-flip"),
        );
        let auth = ExecutorContract::backend_only(ExecutionBackend::Daemon).authorize_edit(&action);
        assert!(auth.allow_execute);
        assert_eq!(auth.backend, ExecutionBackend::Daemon);
        assert_eq!(auth.reason, "verified_transition_authority");
    }

    #[test]
    fn backend_cannot_override_blocked_action() {
        let action = EditAction::planned_replacement(
            "test",
            100,
            "а б".to_string(),
            "аб".to_string(),
            TextReplacement {
                move_left: 0,
                backspaces: 3,
                insert: "аб".to_string(),
                move_right: 0,
            },
            Some("test"),
            Some("boundary-unsafe"),
        );
        let auth = ExecutorContract::backend_only(ExecutionBackend::Ime).authorize_edit(&action);
        assert!(!auth.allow_execute);
        assert_eq!(auth.backend, ExecutionBackend::Ime);
        assert_ne!(auth.reason, "verified_transition_authority");
    }
}

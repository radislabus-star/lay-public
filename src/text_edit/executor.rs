use super::action::EditAction;
use crate::typing_transition::executor_contract::{ExecutionBackend, ExecutorContract};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextEditBackend {
    Daemon,
    Ime,
    Clipboard,
    TrayStatus,
}

/// Receipt for one physical-backend dispatch attempt.
///
/// A caller may select another backend only when no mutation call was sent.
/// `Dispatched` deliberately does not claim that the client has observed the
/// expected text. Once a mutation call was sent, fail closed and let the
/// backend's postcondition observer reconcile the visible state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BackendDispatchReceipt {
    NotDispatched {
        backend: TextEditBackend,
        reason: &'static str,
    },
    Dispatched {
        backend: TextEditBackend,
    },
    Rejected {
        backend: TextEditBackend,
        reason: &'static str,
    },
    Indeterminate {
        backend: TextEditBackend,
        error: String,
    },
}

impl BackendDispatchReceipt {
    pub const fn not_dispatched(backend: TextEditBackend, reason: &'static str) -> Self {
        Self::NotDispatched { backend, reason }
    }

    pub const fn dispatched(backend: TextEditBackend) -> Self {
        Self::Dispatched { backend }
    }

    pub const fn rejected(backend: TextEditBackend, reason: &'static str) -> Self {
        Self::Rejected { backend, reason }
    }

    pub fn indeterminate(backend: TextEditBackend, error: impl Into<String>) -> Self {
        Self::Indeterminate {
            backend,
            error: error.into(),
        }
    }

    pub const fn backend(&self) -> TextEditBackend {
        match self {
            Self::NotDispatched { backend, .. }
            | Self::Dispatched { backend }
            | Self::Rejected { backend, .. }
            | Self::Indeterminate { backend, .. } => *backend,
        }
    }

    pub const fn was_dispatched(&self) -> bool {
        matches!(self, Self::Dispatched { .. })
    }

    pub const fn permits_backend_reselection(&self) -> bool {
        matches!(self, Self::NotDispatched { .. })
    }

    /// True only when the backend proved that no visible mutation happened.
    ///
    /// Explicit rollback may use this stronger receipt to select a fallback
    /// backend. Indeterminate and dispatched outcomes must remain single-shot.
    pub const fn confirmed_no_mutation(&self) -> bool {
        matches!(self, Self::NotDispatched { .. } | Self::Rejected { .. })
    }

    pub fn reason(&self) -> &str {
        match self {
            Self::NotDispatched { reason, .. } | Self::Rejected { reason, .. } => reason,
            Self::Dispatched { .. } => "dispatched",
            Self::Indeterminate { error, .. } => error,
        }
    }
}

impl TextEditBackend {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Daemon => "daemon",
            Self::Ime => "ime",
            Self::Clipboard => "clipboard",
            Self::TrayStatus => "tray-status",
        }
    }
}

/// Capability issued only after the transition verifier admits the edit.
///
/// Output adapters may inspect this capability, but cannot manufacture one.
/// This is intentionally the only value that represents permission to mutate
/// user-visible text.
#[derive(Debug, PartialEq, Eq)]
pub struct AuthorizedEdit {
    backend: TextEditBackend,
    action: EditAction,
}

impl AuthorizedEdit {
    pub const fn backend(&self) -> TextEditBackend {
        self.backend
    }

    pub fn action(&self) -> &EditAction {
        &self.action
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct BackendEditAuthorization {
    pub backend: TextEditBackend,
    pub allow_execute: bool,
    pub reason: &'static str,
    authorized: Option<AuthorizedEdit>,
}

impl BackendEditAuthorization {
    /// Consumes the authorization receipt and yields the one-shot mutation
    /// capability. The capability is deliberately not cloneable.
    pub fn into_authorized(self) -> Option<AuthorizedEdit> {
        self.authorized
    }
}

pub fn authorize_backend_edit(
    backend: TextEditBackend,
    action: EditAction,
) -> BackendEditAuthorization {
    let execution_backend: ExecutionBackend = backend.into();
    let auth = ExecutorContract::backend_only(execution_backend).authorize_edit(&action);
    debug_assert_eq!(auth.backend, execution_backend);
    if auth.allow_execute {
        BackendEditAuthorization {
            backend,
            allow_execute: true,
            reason: auth.reason,
            authorized: Some(AuthorizedEdit { backend, action }),
        }
    } else {
        BackendEditAuthorization {
            backend,
            allow_execute: false,
            reason: auth.reason,
            authorized: None,
        }
    }
}

impl From<TextEditBackend> for ExecutionBackend {
    fn from(value: TextEditBackend) -> Self {
        match value {
            TextEditBackend::Daemon => Self::Daemon,
            TextEditBackend::Ime => Self::Ime,
            TextEditBackend::Clipboard => Self::Clipboard,
            TextEditBackend::TrayStatus => Self::TrayStatus,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{authorize_backend_edit, BackendDispatchReceipt, TextEditBackend};
    use crate::text_edit::{
        plan_committed_tail_full_token_replacement, plan_manual_edit, EditAction, TextReplacement,
    };

    #[test]
    fn text_edit_backend_is_execution_only() {
        let action = plan_manual_edit(
            "test",
            100,
            "a b",
            "ab",
            TextReplacement {
                move_left: 0,
                backspaces: 3,
                insert: "ab".to_string(),
                move_right: 0,
            },
            2,
        );
        let auth = authorize_backend_edit(TextEditBackend::Ime, action);
        assert!(!auth.allow_execute);
        assert!(auth.into_authorized().is_none());
    }

    #[test]
    fn safe_plan_without_transition_proof_cannot_create_capability() {
        let action = EditAction::keep("test", "провека ");
        assert!(!action.allow_apply());

        let auth = authorize_backend_edit(TextEditBackend::Daemon, action);

        assert!(!auth.allow_execute);
        assert_eq!(auth.reason, "non_executable_edit_action");
        assert!(auth.into_authorized().is_none());
    }

    #[test]
    fn verified_transition_creates_one_shot_capability() {
        let action = plan_manual_edit(
            "test",
            900,
            "провека ",
            "проверка ",
            plan_committed_tail_full_token_replacement("провека ", "проверка ").expect("plan"),
            1,
        );

        let auth = authorize_backend_edit(TextEditBackend::Daemon, action);

        assert!(auth.allow_execute);
        assert!(auth.into_authorized().is_some());
    }

    #[test]
    fn only_an_undispatched_receipt_allows_backend_reselection() {
        let not_dispatched =
            BackendDispatchReceipt::not_dispatched(TextEditBackend::Ime, "no_focused_ime");
        let rejected =
            BackendDispatchReceipt::rejected(TextEditBackend::Ime, "visible_state_rejected");
        let indeterminate =
            BackendDispatchReceipt::indeterminate(TextEditBackend::Ime, "transport closed");
        let dispatched = BackendDispatchReceipt::dispatched(TextEditBackend::Ime);

        assert!(not_dispatched.permits_backend_reselection());
        assert!(!rejected.permits_backend_reselection());
        assert!(!indeterminate.permits_backend_reselection());
        assert!(!dispatched.permits_backend_reselection());
        assert!(not_dispatched.confirmed_no_mutation());
        assert!(rejected.confirmed_no_mutation());
        assert!(!indeterminate.confirmed_no_mutation());
        assert!(!dispatched.confirmed_no_mutation());
        assert!(dispatched.was_dispatched());
        assert_eq!(dispatched.reason(), "dispatched");
    }
}

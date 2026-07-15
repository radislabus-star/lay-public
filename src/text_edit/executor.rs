use super::action::EditAction;
use crate::typing_transition::executor_contract::{ExecutionBackend, ExecutorContract};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextEditBackend {
    Daemon,
    Ime,
    Clipboard,
    TrayStatus,
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
    use super::{authorize_backend_edit, TextEditBackend};
    use crate::text_edit::{
        plan_committed_tail_full_token_replacement, EditAction, TextReplacement, TransitionAudit,
    };

    #[test]
    fn text_edit_backend_is_execution_only() {
        let action = EditAction::planned_replacement(
            "test",
            100,
            "a b".to_string(),
            "ab".to_string(),
            TextReplacement {
                move_left: 0,
                backspaces: 3,
                insert: "ab".to_string(),
                move_right: 0,
            },
            Some("test"),
            Some("boundary-unsafe"),
        );
        let auth = authorize_backend_edit(TextEditBackend::Ime, action);
        assert!(!auth.allow_execute);
        assert!(auth.into_authorized().is_none());
    }

    #[test]
    fn safe_plan_without_transition_proof_cannot_create_capability() {
        let action = EditAction::planned_replacement(
            "test",
            900,
            "провека ",
            "проверка ",
            plan_committed_tail_full_token_replacement("провека ", "проверка ").expect("plan"),
            Some("test"),
            Some("missing-letter"),
        );
        assert!(action.allow_apply());

        let auth = authorize_backend_edit(TextEditBackend::Daemon, action);

        assert!(!auth.allow_execute);
        assert_eq!(auth.reason, "verified_transition_proof_missing");
        assert!(auth.into_authorized().is_none());
    }

    #[test]
    fn verified_transition_creates_one_shot_capability() {
        let action = EditAction::planned_replacement(
            "test",
            900,
            "провека ",
            "проверка ",
            plan_committed_tail_full_token_replacement("провека ", "проверка ").expect("plan"),
            Some("test"),
            Some("missing-letter"),
        )
        .with_transition(TransitionAudit::proven(
            "replace_current_word",
            "candidate_and_plan_verified",
            true,
            false,
            1,
        ));

        let auth = authorize_backend_edit(TextEditBackend::Daemon, action);

        assert!(auth.allow_execute);
        assert!(auth.into_authorized().is_some());
    }
}

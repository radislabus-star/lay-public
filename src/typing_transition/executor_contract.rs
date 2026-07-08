//! Contract between transition authority and output backends.
//!
//! Daemon, IME, clipboard, and tray code may execute a verified edit plan, but
//! they must not decide that a text replacement is true.

#![allow(dead_code)]

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

impl ExecutorContract {
    pub(crate) const fn backend_only(backend: ExecutionBackend) -> Self {
        Self {
            backend,
            may_decide_apply: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{ExecutionBackend, ExecutorContract};

    #[test]
    fn ime_is_backend_not_authority() {
        let contract = ExecutorContract::backend_only(ExecutionBackend::Ime);
        assert!(!contract.may_decide_apply);
    }
}

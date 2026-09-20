use super::{TextTargetAuthority, WindowInteraction};
use crate::context_admission::{AdmissionToken, SettledWordState};
use crate::engine::LayIbusEngine;
use crate::output::EngineOutput;
use crate::state::CommittedTailReplaceRequest;
use zbus::fdo;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum LocalEffectProgress {
    None,
    SurroundingTextRequested,
    CursorOrPreedit,
    DeleteDispatched,
    CommitDispatched,
}

impl LocalEffectProgress {
    pub(crate) fn may_have_mutated_client(self) -> bool {
        matches!(
            self,
            Self::CursorOrPreedit | Self::DeleteDispatched | Self::CommitDispatched
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ExecutionReceipt {
    LocalComplete,
    LocalPending,
    LocalCancelled,
    LocalIndeterminatePartial,
    LocalErrorBeforeMutation,
    DelegatedExactImeTail,
    DelegatedDaemonBuffer,
    Rejected,
}

#[derive(Debug)]
pub(crate) struct LocalExecutionFailure {
    progress: LocalEffectProgress,
    source: fdo::Error,
}

impl LocalExecutionFailure {
    pub(crate) fn new(progress: LocalEffectProgress, source: fdo::Error) -> Self {
        Self { progress, source }
    }

    #[cfg(test)]
    pub(crate) fn progress(&self) -> LocalEffectProgress {
        self.progress
    }

    pub(crate) fn receipt(&self) -> ExecutionReceipt {
        if self.progress.may_have_mutated_client() {
            ExecutionReceipt::LocalIndeterminatePartial
        } else {
            ExecutionReceipt::LocalErrorBeforeMutation
        }
    }
}

impl From<fdo::Error> for LocalExecutionFailure {
    fn from(source: fdo::Error) -> Self {
        Self::new(LocalEffectProgress::None, source)
    }
}

impl From<LocalExecutionFailure> for fdo::Error {
    fn from(failure: LocalExecutionFailure) -> Self {
        failure.source
    }
}

impl std::fmt::Display for LocalExecutionFailure {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.source.fmt(formatter)
    }
}

impl std::error::Error for LocalExecutionFailure {}

impl WindowInteraction {
    pub(crate) async fn execute_local(
        engine: &mut LayIbusEngine,
        authority: TextTargetAuthority,
        request: CommittedTailReplaceRequest,
        output: &mut EngineOutput<'_, '_>,
    ) -> Result<ExecutionReceipt, LocalExecutionFailure> {
        let live = Self::admit_replace(engine, request.backspaces);
        if live != authority
            || !matches!(
                live,
                TextTargetAuthority::LocalTerminalErase
                    | TextTargetAuthority::LocalSurroundingDeleteCommit
            )
        {
            return Ok(ExecutionReceipt::Rejected);
        }
        engine
            .replace_committed_tail_with_effect_progress(output, request)
            .await
            .map(|handled| {
                if handled {
                    ExecutionReceipt::LocalComplete
                } else {
                    ExecutionReceipt::Rejected
                }
            })
    }

    pub(crate) async fn execute_manual_toggle(
        engine: &mut LayIbusEngine,
        authority: TextTargetAuthority,
        output: &mut EngineOutput<'_, '_>,
    ) -> Result<(Option<bool>, ExecutionReceipt), LocalExecutionFailure> {
        if Self::admit_manual_toggle(engine) != authority {
            return Ok((None, ExecutionReceipt::Rejected));
        }
        if matches!(
            authority,
            TextTargetAuthority::AtomicOwned | TextTargetAuthority::Reject(_)
        ) {
            return Ok((None, ExecutionReceipt::Rejected));
        }
        engine
            .manual_toggle_active_text_target_with_disposition(output)
            .await
    }
}

/// Scoped to the bridge's existing exclusive engine guard. A cancelled or
/// failed output cannot leave its admission witness available to later work.
pub(crate) struct ContextBridgeOutput<'a> {
    engine: &'a mut LayIbusEngine,
    completed: bool,
}

impl std::ops::Deref for ContextBridgeOutput<'_> {
    type Target = LayIbusEngine;
    fn deref(&self) -> &Self::Target {
        self.engine
    }
}

impl std::ops::DerefMut for ContextBridgeOutput<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.engine
    }
}

impl ContextBridgeOutput<'_> {
    pub(crate) fn complete(&mut self) {
        self.completed = true;
    }
}

impl Drop for ContextBridgeOutput<'_> {
    fn drop(&mut self) {
        self.engine.context_bridge_token = None;
        if !self.completed {
            self.engine.revoke_context_word();
        }
    }
}

impl LayIbusEngine {
    pub(crate) fn begin_context_bridge_output(
        &mut self,
        token: Option<&AdmissionToken>,
    ) -> ContextBridgeOutput<'_> {
        self.context_bridge_token = token.cloned();
        ContextBridgeOutput {
            engine: self,
            completed: false,
        }
    }

    pub(crate) fn settle_context_bridge_output(&mut self) -> bool {
        let Some(token) = self.context_bridge_token.as_ref() else {
            return true;
        };
        let accepted = !self.atomic.speculation
            && self.context_word_scope.as_ref().is_some_and(|scope| {
                self.context_admission.as_ref().is_some_and(|admission| {
                    admission.settle_bridge_output(
                        token,
                        SettledWordState::from_scope(self.committed_tail.epoch, scope),
                    )
                })
            });
        if !accepted {
            self.revoke_context_word();
        }
        accepted
    }
}

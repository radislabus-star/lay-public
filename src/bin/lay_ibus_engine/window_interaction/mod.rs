mod authority;
mod execution;
mod observation;
#[cfg(test)]
mod tests;

pub(crate) use authority::TextTargetEditRoute;
#[cfg(test)]
pub(crate) use authority::{
    TextTargetCapabilityFacts, TextTargetDecision, TextTargetDecisionReason,
};
pub(crate) use execution::{ExecutionReceipt, LocalEffectProgress, LocalExecutionFailure};
pub(crate) use observation::{
    ClientContextState, PendingContextResetRereceipt, SurroundingTextSnapshot, WindowFactEvent,
    WindowInteraction, WindowLifecycleEvent,
};
#[cfg(test)]
pub(crate) use observation::{LifecycleReceipt, ObservationReceipt};

pub(crate) const IBUS_CAP_SURROUNDING_TEXT: u32 = 1 << 5;
pub(crate) const IBUS_INPUT_PURPOSE_PASSWORD: u32 = 8;
pub(crate) const IBUS_INPUT_PURPOSE_PIN: u32 = 9;
pub(crate) const IBUS_INPUT_PURPOSE_TERMINAL: u32 = 10;
pub(crate) const IBUS_INPUT_HINT_PRIVATE: u32 = 1 << 11;
pub(crate) const IBUS_INPUT_HINT_HIDDEN_TEXT: u32 = 1 << 12;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum WindowRejectReason {
    SensitiveContent,
    SelectionPresent,
    MissingProvenDeleteCapability,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TextTargetAuthority {
    DelegateExactImeTail,
    LocalTerminalErase,
    LocalSurroundingDeleteCommit,
    DelegateDaemonBuffer,
    ActiveCompositionOwned,
    AtomicOwned,
    Reject(WindowRejectReason),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum OutcomeProof {
    ExistingPostconditionConfirmed,
    ExistingPostconditionPending,
    ExistingPostconditionMismatch,
    ExistingPostconditionCensored,
    Rejected,
}

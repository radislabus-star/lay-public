use super::action::EditAction;
use super::mutation::{TransitionAudit, TransitionOperator, TransitionProof};
use super::types::TextReplacement;
use super::visible_tail::{SnapshotIdentity, VisibleTailSnapshot, VisibleTailSource};
use crate::text_metrics::{transition_changed_token_count, transition_left_context_changed};
use crate::typing_transition::decision::{DecisionTransitionReceipt, TransitionDecisionCore};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum TransitionAuthorityKind {
    AutomaticDecision,
    ExplicitUserIntent,
    RecordedUndo,
    CompletionAcceptance,
    NativeIntent,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct TransitionAuthority {
    kind: TransitionAuthorityKind,
    transition: TransitionAudit,
}

impl TransitionAuthority {
    pub(super) fn automatic_decision(
        receipt: &DecisionTransitionReceipt,
        from_text: &str,
        to_text: &str,
    ) -> Option<Self> {
        let transition = receipt.projected_transition(from_text, to_text)?;
        Self::from_verified_transition(TransitionAuthorityKind::AutomaticDecision, transition)
    }

    pub(super) fn explicit_user_intent(from_text: &str, to_text: &str) -> Self {
        Self::verified(
            TransitionAuthorityKind::ExplicitUserIntent,
            TransitionOperator::ManualReplace,
            TransitionProof::ManualIntent,
            from_text,
            to_text,
        )
    }

    pub(super) fn recorded_undo(from_text: &str, to_text: &str) -> Self {
        Self::verified(
            TransitionAuthorityKind::RecordedUndo,
            TransitionOperator::Undo,
            TransitionProof::UndoRecord,
            from_text,
            to_text,
        )
    }

    pub(super) fn completion_acceptance(from_text: &str, to_text: &str) -> Option<Self> {
        if !completion_projection_is_valid(from_text, to_text) {
            return None;
        }
        Some(Self::from_transition(
            TransitionAuthorityKind::CompletionAcceptance,
            TransitionAudit::proven(
                TransitionOperator::Completion,
                TransitionProof::Completion,
                true,
                false,
                1,
            ),
        ))
    }

    pub(super) fn native_intent(from_text: &str, to_text: &str) -> Self {
        Self::verified(
            TransitionAuthorityKind::NativeIntent,
            TransitionOperator::NativeReplace,
            TransitionProof::NativeIntent,
            from_text,
            to_text,
        )
    }

    pub(super) const fn transition(&self) -> &TransitionAudit {
        &self.transition
    }

    pub(super) fn matches_transition(&self, transition: &TransitionAudit) -> bool {
        if &self.transition != transition {
            return false;
        }
        let Some(operator) = transition.operator() else {
            return false;
        };
        let Some(proof) = transition.proof() else {
            return false;
        };
        transition.is_verified() && self.kind.accepts(operator, proof)
    }

    fn verified(
        kind: TransitionAuthorityKind,
        operator: TransitionOperator,
        proof: TransitionProof,
        from_text: &str,
        to_text: &str,
    ) -> Self {
        Self::from_transition(
            kind,
            TransitionAudit::proven(
                operator,
                proof,
                true,
                transition_left_context_changed(from_text, to_text),
                transition_changed_token_count(from_text, to_text),
            ),
        )
    }

    fn from_verified_transition(
        kind: TransitionAuthorityKind,
        transition: TransitionAudit,
    ) -> Option<Self> {
        let operator = transition.operator()?;
        let proof = transition.proof()?;
        if !transition.is_verified() || !kind.accepts(operator, proof) {
            return None;
        }
        Some(Self::from_transition(kind, transition))
    }

    fn from_transition(kind: TransitionAuthorityKind, transition: TransitionAudit) -> Self {
        Self { kind, transition }
    }
}

impl TransitionAuthorityKind {
    fn accepts(self, operator: TransitionOperator, proof: TransitionProof) -> bool {
        match self {
            Self::AutomaticDecision => automatic_decision_pair_is_valid(operator, proof),
            Self::ExplicitUserIntent => {
                operator == TransitionOperator::ManualReplace
                    && proof == TransitionProof::ManualIntent
            }
            Self::RecordedUndo => {
                operator == TransitionOperator::Undo && proof == TransitionProof::UndoRecord
            }
            Self::CompletionAcceptance => {
                operator == TransitionOperator::Completion && proof == TransitionProof::Completion
            }
            Self::NativeIntent => {
                operator == TransitionOperator::NativeReplace
                    && proof == TransitionProof::NativeIntent
            }
        }
    }
}

fn automatic_decision_pair_is_valid(operator: TransitionOperator, proof: TransitionProof) -> bool {
    matches!(
        (operator, proof),
        (
            TransitionOperator::ReplaceCurrentWord,
            TransitionProof::Typo
                | TransitionProof::Layout
                | TransitionProof::Context
                | TransitionProof::Grammar
        ) | (
            TransitionOperator::LayoutProjection,
            TransitionProof::Layout
        ) | (
            TransitionOperator::BoundaryShift
                | TransitionOperator::BoundaryMergeSplit
                | TransitionOperator::SplitPreviousGluedAndRepairTail,
            TransitionProof::Boundary
        ) | (
            TransitionOperator::PhraseTokenRepair,
            TransitionProof::Context
        ) | (TransitionOperator::Completion, TransitionProof::Completion)
            | (
                TransitionOperator::VisibleTail,
                TransitionProof::VisibleState
            )
            | (
                TransitionOperator::DecoderTail,
                TransitionProof::DecoderPlan
            )
            | (
                TransitionOperator::EnterAutocorrect,
                TransitionProof::Context | TransitionProof::Typo | TransitionProof::Layout
            )
    )
}

fn completion_projection_is_valid(from_text: &str, to_text: &str) -> bool {
    let from = from_text.trim_end_matches(char::is_whitespace);
    let to = to_text.trim_end_matches(char::is_whitespace);
    !from.is_empty() && to.len() > from.len() && to.starts_with(from)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VisibleFieldState {
    pub(crate) visible_tail: String,
    pub(crate) focus_id: Option<String>,
    pub(crate) epoch: u64,
    pub(crate) external_state_present: bool,
    pub(crate) external_tail_before_cursor: Option<String>,
    pub(crate) external_selection_active: bool,
}

impl VisibleFieldState {
    pub fn committed_tail(visible_tail: impl Into<String>, focus_id: Option<String>) -> Self {
        Self {
            visible_tail: visible_tail.into(),
            focus_id,
            epoch: 0,
            external_state_present: false,
            external_tail_before_cursor: None,
            external_selection_active: false,
        }
    }

    pub fn with_epoch(mut self, epoch: u64) -> Self {
        self.epoch = epoch;
        self
    }

    pub fn with_external_tail_before_cursor(
        mut self,
        external_tail_before_cursor: Option<String>,
        external_selection_active: bool,
    ) -> Self {
        self.external_state_present = true;
        self.external_tail_before_cursor = external_tail_before_cursor;
        self.external_selection_active = external_selection_active;
        self
    }

    pub fn snapshot_identity(&self, source: VisibleTailSource) -> SnapshotIdentity {
        VisibleTailSnapshot::new(
            source,
            self.visible_tail.clone(),
            self.focus_id.clone(),
            self.epoch,
        )
        .identity()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextTransitionIntent {
    ImeAutocorrect,
    ImeManualToggle,
    DaemonBridge,
}

impl TextTransitionIntent {
    pub(crate) const fn operator(self) -> TransitionOperator {
        TransitionOperator::VisibleTail
    }

    pub(crate) const fn proof(self) -> TransitionProof {
        TransitionProof::VisibleState
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LatentTextTransitionCandidate {
    pub(crate) source: VisibleTailSource,
    pub(crate) delete_chars: u32,
    pub(crate) insert_text: String,
    pub(crate) intent: TextTransitionIntent,
    pub(crate) expected_tail: Option<VisibleTailSnapshot>,
}

impl LatentTextTransitionCandidate {
    pub fn new(
        source: VisibleTailSource,
        delete_chars: u32,
        insert_text: impl Into<String>,
        intent: TextTransitionIntent,
        expected_tail: Option<VisibleTailSnapshot>,
    ) -> Self {
        Self {
            source,
            delete_chars,
            insert_text: insert_text.into(),
            intent,
            expected_tail,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TextTransitionDecision {
    AlreadyApplied,
    Apply {
        plan: TextReplacement,
        action: EditAction,
    },
    Reject {
        rejection: TextTransitionRejection,
        action: Option<EditAction>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TextTransitionRejection {
    Noop,
    StaleVisibleTail { expected: String, actual: String },
    StaleVisibleRevision { expected: u64, actual: u64 },
    StaleSurroundingText { expected: String, actual: String },
    UnsafeEdit { reason: &'static str },
}

impl TextTransitionRejection {
    pub fn reason(&self) -> &'static str {
        match self {
            Self::Noop => "noop_transition",
            Self::StaleVisibleTail { .. } => "stale_visible_tail",
            Self::StaleVisibleRevision { .. } => "stale_visible_revision",
            Self::StaleSurroundingText { .. } => "stale_surrounding_text",
            Self::UnsafeEdit { reason } => reason,
        }
    }

    pub fn expected(&self) -> &str {
        match self {
            Self::Noop | Self::UnsafeEdit { .. } | Self::StaleVisibleRevision { .. } => "",
            Self::StaleVisibleTail { expected, .. }
            | Self::StaleSurroundingText { expected, .. } => expected,
        }
    }

    pub fn actual(&self) -> &str {
        match self {
            Self::Noop | Self::UnsafeEdit { .. } | Self::StaleVisibleRevision { .. } => "",
            Self::StaleVisibleTail { actual, .. } | Self::StaleSurroundingText { actual, .. } => {
                actual
            }
        }
    }
}

pub fn decide_text_transition(
    state: &VisibleFieldState,
    candidate: LatentTextTransitionCandidate,
) -> TextTransitionDecision {
    TransitionDecisionCore::decide_visible_text_transition(state, candidate)
}

#[cfg(test)]
mod tests {
    use super::{
        decide_text_transition, LatentTextTransitionCandidate, TextTransitionDecision,
        TextTransitionIntent, TextTransitionRejection, VisibleFieldState,
    };
    use crate::text_edit::{
        EditActionKind, TransitionOperator, VisibleTailSnapshot, VisibleTailSource,
    };

    fn candidate(delete_chars: u32, insert_text: &str) -> LatentTextTransitionCandidate {
        LatentTextTransitionCandidate::new(
            VisibleTailSource::ImeCommittedTail,
            delete_chars,
            insert_text,
            TextTransitionIntent::ImeAutocorrect,
            None,
        )
    }

    #[test]
    fn text_transition_applies_matching_visible_and_surrounding_tail() {
        let state = VisibleFieldState::committed_tail("abc ghbdtn", Some("/test".to_string()))
            .with_external_tail_before_cursor(Some("bdtn".to_string()), false);

        let decision = decide_text_transition(&state, candidate(4, "вет"));

        match decision {
            TextTransitionDecision::Apply { plan, action } => {
                assert_eq!(plan.backspaces, 4);
                assert_eq!(plan.insert, "вет");
                assert_eq!(action.kind(), EditActionKind::ReplaceLastToken);
                assert_eq!(
                    action.transition().operator(),
                    Some(TransitionOperator::VisibleTail)
                );
            }
            other => panic!("unexpected decision: {other:?}"),
        }
    }

    #[test]
    fn text_transition_rejects_stale_visible_tail_snapshot() {
        let state = VisibleFieldState::committed_tail("abc ghjdt", Some("/test".to_string()));
        let mut request = candidate(4, "вет");
        request.expected_tail = Some(VisibleTailSnapshot::new(
            VisibleTailSource::ImeCommittedTail,
            "bdtn",
            Some("/test".to_string()),
            0,
        ));

        let decision = decide_text_transition(&state, request);

        match decision {
            TextTransitionDecision::Reject { rejection, action } => {
                assert_eq!(action, None);
                assert_eq!(rejection.reason(), "stale_visible_tail");
                assert_eq!(rejection.expected(), "bdtn");
                assert_eq!(rejection.actual(), "hjdt");
            }
            other => panic!("unexpected decision: {other:?}"),
        }
    }

    #[test]
    fn text_transition_rejects_stale_visible_revision() {
        let state =
            VisibleFieldState::committed_tail("abc ghjdt", Some("/test".to_string())).with_epoch(8);
        let mut request = candidate(4, "вет");
        request.expected_tail = Some(VisibleTailSnapshot::new(
            VisibleTailSource::ImeCommittedTail,
            "hjdt",
            Some("/test".to_string()),
            7,
        ));

        let decision = decide_text_transition(&state, request);

        assert!(matches!(
            decision,
            TextTransitionDecision::Reject {
                rejection: TextTransitionRejection::StaleVisibleRevision {
                    expected: 7,
                    actual: 8
                },
                action: None
            }
        ));
    }

    #[test]
    fn visible_field_state_exports_immutable_snapshot_identity() {
        let state = VisibleFieldState::committed_tail("abc ghbdtn", Some("/test".to_string()))
            .with_epoch(8);
        let identity = state.snapshot_identity(VisibleTailSource::ImeCommittedTail);

        assert_eq!(identity.focus_id.as_deref(), Some("/test"));
        assert_eq!(identity.revision, 8);
        assert_ne!(identity.visible_tail_hash, 0);
    }

    #[test]
    fn text_transition_is_idempotent_when_target_state_is_already_visible() {
        let state = VisibleFieldState::committed_tail("abc Ты ", Some("/test".to_string()));
        let mut request = candidate(3, "Ты ");
        request.expected_tail = Some(VisibleTailSnapshot::new(
            VisibleTailSource::DaemonWordBuffer,
            "Ns ",
            Some("/test".to_string()),
            0,
        ));
        request.source = VisibleTailSource::DaemonWordBuffer;

        assert_eq!(
            decide_text_transition(&state, request),
            TextTransitionDecision::AlreadyApplied
        );
    }

    #[test]
    fn text_transition_rejects_stale_surrounding_text() {
        let state = VisibleFieldState::committed_tail("abc ghbdtn", Some("/test".to_string()))
            .with_external_tail_before_cursor(Some("hjdt".to_string()), false);

        let decision = decide_text_transition(&state, candidate(4, "вет"));

        match decision {
            TextTransitionDecision::Reject { rejection, action } => {
                assert_eq!(action, None);
                assert_eq!(rejection.reason(), "stale_surrounding_text");
                assert_eq!(rejection.expected(), "bdtn");
                assert_eq!(rejection.actual(), "hjdt");
            }
            other => panic!("unexpected decision: {other:?}"),
        }
    }

    #[test]
    fn text_transition_rejects_active_external_selection() {
        let state = VisibleFieldState::committed_tail("abc ghbdtn", Some("/test".to_string()))
            .with_external_tail_before_cursor(Some("bdtn".to_string()), true);

        let decision = decide_text_transition(&state, candidate(4, "вет"));

        match decision {
            TextTransitionDecision::Reject { rejection, action } => {
                assert_eq!(action, None);
                assert_eq!(rejection.reason(), "stale_surrounding_text");
            }
            other => panic!("unexpected decision: {other:?}"),
        }
    }

    #[test]
    fn text_transition_rejects_present_but_unreadable_surrounding_text() {
        let state = VisibleFieldState::committed_tail("abc ghbdtn", Some("/test".to_string()))
            .with_external_tail_before_cursor(None, false);

        let decision = decide_text_transition(&state, candidate(4, "вет"));

        match decision {
            TextTransitionDecision::Reject { rejection, action } => {
                assert_eq!(action, None);
                assert_eq!(rejection.reason(), "stale_surrounding_text");
                assert_eq!(rejection.expected(), "bdtn");
                assert_eq!(rejection.actual(), "");
            }
            other => panic!("unexpected decision: {other:?}"),
        }
    }

    #[test]
    fn text_transition_blocks_unsafe_multiword_edit() {
        let state = VisibleFieldState::committed_tail("одно два", Some("/test".to_string()));

        let decision = decide_text_transition(&state, candidate(8, "однотри"));

        match decision {
            TextTransitionDecision::Reject { rejection, action } => {
                assert_eq!(
                    rejection,
                    TextTransitionRejection::UnsafeEdit {
                        reason: "unsafe_multiword_autocorrect_scope"
                    }
                );
                assert_eq!(
                    action.expect("edit action").kind(),
                    EditActionKind::BlockUnsafe
                );
            }
            other => panic!("unexpected decision: {other:?}"),
        }
    }

    #[test]
    fn text_transition_blocks_two_word_cursor_teleport_edit() {
        let original = "так можно проверить скры";
        let state = VisibleFieldState::committed_tail(original, Some("/test".to_string()));
        let request = LatentTextTransitionCandidate::new(
            VisibleTailSource::ImeCommittedTail,
            original.chars().count() as u32,
            "так можно проверять нкрытое сос",
            TextTransitionIntent::ImeAutocorrect,
            None,
        );

        let decision = decide_text_transition(&state, request);

        match decision {
            TextTransitionDecision::Reject { rejection, action } => {
                assert_eq!(
                    rejection,
                    TextTransitionRejection::UnsafeEdit {
                        reason: "unsafe_multiword_autocorrect_scope"
                    }
                );
                let action = action.expect("edit action");
                assert_eq!(action.kind(), EditActionKind::BlockUnsafe);
                assert_eq!(
                    action.transition().operator(),
                    Some(TransitionOperator::VisibleTail)
                );
            }
            other => panic!("unexpected decision: {other:?}"),
        }
    }
}

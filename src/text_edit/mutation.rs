use crate::language_action::LanguageActionProof;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TransitionOperator {
    ReplaceCurrentWord,
    LayoutProjection,
    BoundaryShift,
    BoundaryMergeSplit,
    PhraseTokenRepair,
    SplitPreviousGluedAndRepairTail,
    Completion,
    VisibleTail,
    DecoderTail,
    ManualReplace,
    Undo,
    EnterAutocorrect,
    NativeReplace,
    Protected,
    Unknown,
}

impl TransitionOperator {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ReplaceCurrentWord => "replace_current_word",
            Self::LayoutProjection => "layout_projection",
            Self::BoundaryShift => "boundary_shift",
            Self::BoundaryMergeSplit => "boundary_merge_split",
            Self::PhraseTokenRepair => "phrase_token_repair",
            Self::SplitPreviousGluedAndRepairTail => "split_previous_glued_and_repair_tail",
            Self::Completion => "completion",
            Self::VisibleTail => "visible_tail",
            Self::DecoderTail => "decoder_tail",
            Self::ManualReplace => "manual_replace",
            Self::Undo => "undo",
            Self::EnterAutocorrect => "enter_autocorrect",
            Self::NativeReplace => "native_replace",
            Self::Protected => "protected",
            Self::Unknown => "unknown",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TransitionProof {
    Typo,
    Layout,
    Boundary,
    Completion,
    Context,
    Grammar,
    VisibleState,
    DecoderPlan,
    ManualIntent,
    UndoRecord,
    NativeIntent,
    Invariant,
}

impl TransitionProof {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Typo => "typo",
            Self::Layout => "layout",
            Self::Boundary => "boundary",
            Self::Completion => "completion",
            Self::Context => "context",
            Self::Grammar => "grammar",
            Self::VisibleState => "visible_state",
            Self::DecoderPlan => "decoder_plan",
            Self::ManualIntent => "manual_intent",
            Self::UndoRecord => "undo_record",
            Self::NativeIntent => "native_intent",
            Self::Invariant => "invariant",
        }
    }
}

impl From<LanguageActionProof> for TransitionProof {
    fn from(proof: LanguageActionProof) -> Self {
        match proof {
            LanguageActionProof::Layout => Self::Layout,
            LanguageActionProof::Typo => Self::Typo,
            LanguageActionProof::Boundary => Self::Boundary,
            LanguageActionProof::Completion => Self::Completion,
            LanguageActionProof::Context => Self::Context,
            LanguageActionProof::Grammar => Self::Grammar,
            LanguageActionProof::None | LanguageActionProof::SafetyVeto => Self::Invariant,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TransitionAudit {
    operator: Option<TransitionOperator>,
    proof: Option<TransitionProof>,
    verified: Option<bool>,
    left_context_changed: Option<bool>,
    changed_tokens: Option<usize>,
}

impl TransitionAudit {
    pub fn none() -> Self {
        Self::default()
    }

    /// Creates diagnostic evidence. This value is not an execution capability;
    /// only the sealed verifier receipt in `text_edit::gate` can authorize an edit.
    pub(crate) fn proven(
        operator: TransitionOperator,
        proof: TransitionProof,
        verified: bool,
        left_context_changed: bool,
        changed_tokens: usize,
    ) -> Self {
        Self {
            operator: Some(operator),
            proof: Some(proof),
            verified: Some(verified),
            left_context_changed: Some(left_context_changed),
            changed_tokens: Some(changed_tokens),
        }
    }

    pub fn blocks_apply(&self) -> bool {
        self.verified == Some(false)
            || (self.left_context_changed.unwrap_or(false) && !self.is_verified())
    }

    pub fn is_verified(&self) -> bool {
        self.verified == Some(true)
            && self.operator.is_some_and(|operator| {
                !matches!(
                    operator,
                    TransitionOperator::Unknown | TransitionOperator::Protected
                )
            })
            && self
                .proof
                .is_some_and(|proof| !matches!(proof, TransitionProof::Invariant))
    }

    pub const fn operator(&self) -> Option<TransitionOperator> {
        self.operator
    }

    pub const fn proof(&self) -> Option<TransitionProof> {
        self.proof
    }

    pub const fn verified(&self) -> Option<bool> {
        self.verified
    }

    pub const fn left_context_changed(&self) -> Option<bool> {
        self.left_context_changed
    }

    pub const fn changed_tokens(&self) -> Option<usize> {
        self.changed_tokens
    }

    pub fn block_reason(&self) -> Option<&'static str> {
        self.blocks_apply()
            .then_some("edit_transition_not_verified")
    }
}

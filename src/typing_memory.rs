//! Unified typing memory event contract.
//!
//! Runtime code should describe learning as typed memory events. Storage layers
//! can then route the same event into L1/L2 usage, L3 phrase context, L4 signed
//! state, and Bayes priors without each caller inventing its own schema.

use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

const CONTEXT_WORDS: usize = 5;
static NEXT_CAUSAL_EPISODE: AtomicU64 = AtomicU64::new(1);

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Deserialize, serde::Serialize,
)]
#[serde(rename_all = "snake_case")]
#[repr(u8)]
pub(crate) enum LayoutProjectionDirection {
    EnToRu = 1,
    RuToEn = 2,
    MixedToRu = 3,
    MixedToEn = 4,
    Unknown = 255,
}

impl LayoutProjectionDirection {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::EnToRu => "en_to_ru",
            Self::RuToEn => "ru_to_en",
            Self::MixedToRu => "mixed_to_ru",
            Self::MixedToEn => "mixed_to_en",
            Self::Unknown => "unknown",
        }
    }

    pub(crate) const fn code(self) -> u8 {
        self as u8
    }

    pub(crate) const fn from_code(code: u8) -> Option<Self> {
        Some(match code {
            1 => Self::EnToRu,
            2 => Self::RuToEn,
            3 => Self::MixedToRu,
            4 => Self::MixedToEn,
            255 => Self::Unknown,
            _ => return None,
        })
    }

    pub(crate) fn from_str(value: &str) -> Option<Self> {
        Some(match value {
            "en_to_ru" => Self::EnToRu,
            "ru_to_en" => Self::RuToEn,
            "mixed_to_ru" => Self::MixedToRu,
            "mixed_to_en" => Self::MixedToEn,
            "unknown" => Self::Unknown,
            _ => return None,
        })
    }
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Deserialize, serde::Serialize,
)]
#[serde(rename_all = "snake_case")]
#[repr(u8)]
pub(crate) enum LayoutProjectionScope {
    Grapheme = 1,
    CurrentToken = 2,
    Phrase = 3,
}

impl LayoutProjectionScope {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::Grapheme => "grapheme",
            Self::CurrentToken => "current_token",
            Self::Phrase => "phrase",
        }
    }

    pub(crate) const fn code(self) -> u8 {
        self as u8
    }

    pub(crate) const fn from_code(code: u8) -> Option<Self> {
        Some(match code {
            1 => Self::Grapheme,
            2 => Self::CurrentToken,
            3 => Self::Phrase,
            _ => return None,
        })
    }

    pub(crate) fn from_str(value: &str) -> Option<Self> {
        Some(match value {
            "grapheme" => Self::Grapheme,
            "current_token" => Self::CurrentToken,
            "phrase" => Self::Phrase,
            _ => return None,
        })
    }
}

/// Typed provenance for causal evidence. `Diagnostic` preserves old adapter
/// labels for replay, but its payload is never semantic authority.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum TypingMemoryEvidenceSource {
    User,
    UserCorrection,
    Layout,
    Ime,
    Autocorrect,
    Candidate,
    Diagnostic(String),
}

impl TypingMemoryEvidenceSource {
    pub(crate) fn from_legacy(value: &str) -> Self {
        match value {
            "user" => Self::User,
            "user_correction" => Self::UserCorrection,
            "layout" => Self::Layout,
            "ime" => Self::Ime,
            "autocorrect" => Self::Autocorrect,
            "candidate" => Self::Candidate,
            other => Self::Diagnostic(other.to_string()),
        }
    }

    pub(crate) fn as_str(&self) -> &str {
        match self {
            Self::User => "user",
            Self::UserCorrection => "user_correction",
            Self::Layout => "layout",
            Self::Ime => "ime",
            Self::Autocorrect => "autocorrect",
            Self::Candidate => "candidate",
            Self::Diagnostic(value) => value,
        }
    }

    pub(crate) const fn code(&self) -> u8 {
        match self {
            Self::User => 1,
            Self::UserCorrection => 2,
            Self::Layout => 3,
            Self::Ime => 4,
            Self::Autocorrect => 5,
            Self::Candidate => 6,
            Self::Diagnostic(_) => 255,
        }
    }
}

/// Typed event operation. The canonical transferable identity remains
/// `TransitionOperatorKind`; this enum preserves interaction semantics and a
/// lossless compatibility label without letting callers branch on raw text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum TypingMemoryOperation {
    Typed,
    Replacement,
    Completion,
    CompletionEdit,
    PredictionMatch,
    ImeAutoUndo,
    LayoutProjection,
    Diagnostic(String),
}

impl TypingMemoryOperation {
    pub(crate) fn from_legacy(value: &str) -> Self {
        match value {
            "typed" => Self::Typed,
            "replacement" => Self::Replacement,
            "completion" => Self::Completion,
            "completion_edit" => Self::CompletionEdit,
            "prediction_match" => Self::PredictionMatch,
            "ime_auto_undo" => Self::ImeAutoUndo,
            "layout" | "layout_projection" => Self::LayoutProjection,
            other => Self::Diagnostic(other.to_string()),
        }
    }

    pub(crate) fn as_str(&self) -> &str {
        match self {
            Self::Typed => "typed",
            Self::Replacement => "replacement",
            Self::Completion => "completion",
            Self::CompletionEdit => "completion_edit",
            Self::PredictionMatch => "prediction_match",
            Self::ImeAutoUndo => "ime_auto_undo",
            Self::LayoutProjection => "layout",
            Self::Diagnostic(value) => value,
        }
    }

    pub(crate) const fn code(&self) -> u8 {
        match self {
            Self::Typed => 1,
            Self::Replacement => 2,
            Self::Completion => 3,
            Self::CompletionEdit => 4,
            Self::PredictionMatch => 5,
            Self::ImeAutoUndo => 6,
            Self::LayoutProjection => 7,
            Self::Diagnostic(_) => 255,
        }
    }
}

/// Compact identity of a learned text transition.
///
/// `source` remains diagnostic provenance only. The operator identity is what
/// L4 can later transfer between adapters and applications.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct TypingTransitionIdentity {
    pub(crate) operator: crate::transition_relation::TransitionOperatorKind,
    pub(crate) layout_direction: Option<LayoutProjectionDirection>,
    pub(crate) layout_scope: Option<LayoutProjectionScope>,
}

impl TypingTransitionIdentity {
    pub(crate) fn observed(from: &str, to: &str, operation: &str) -> Self {
        use crate::transition_relation::TransitionOperatorKind;

        let operator = TransitionOperatorKind::infer(from, to, operation);
        let (layout_direction, layout_scope) =
            if operator == TransitionOperatorKind::LayoutProjection {
                (
                    Some(layout_projection_direction(from, to)),
                    Some(layout_projection_scope(from, to)),
                )
            } else {
                (None, None)
            };
        Self {
            operator,
            layout_direction,
            layout_scope,
        }
    }

    fn typed() -> Self {
        Self {
            operator: crate::transition_relation::TransitionOperatorKind::Other,
            layout_direction: None,
            layout_scope: None,
        }
    }

    fn no_mutation() -> Self {
        Self {
            operator: crate::transition_relation::TransitionOperatorKind::Other,
            layout_direction: None,
            layout_scope: None,
        }
    }

    pub(crate) fn learning_key(self) -> String {
        let mut key = self.operator.as_str().to_string();
        if let (Some(direction), Some(scope)) = (self.layout_direction, self.layout_scope) {
            key.push(':');
            key.push_str(direction.as_str());
            key.push(':');
            key.push_str(scope.as_str());
        }
        key
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TypingMemoryEventKind {
    Typed,
    AcceptedFix,
    AcceptedIme,
    EditedIme,
    ConfirmedImePrediction,
    RejectedIme,
    RejectedCandidate,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub(crate) struct CompletionEditTrace {
    pub(crate) prefix: String,
    pub(crate) accepted_suffix_chars: u32,
    pub(crate) preserved_suffix_chars: u32,
    pub(crate) deleted_chars: u32,
    pub(crate) inserted_chars: u32,
}

/// Canonical learning key shared by feedback recording and live L4 readout.
/// It keeps keyboard direction/scope distinct without making source adapters
/// part of the learned authority.
pub(crate) fn transition_learning_key(from: &str, to: &str, operation: &str) -> String {
    TypingTransitionIdentity::observed(from, to, operation).learning_key()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TypingMemoryFeedback {
    Observed,
    Accepted,
    Rejected,
}

/// Causal status of the observed transition.
///
/// Only confirmed/reverted outcomes may later teach L4. Transport loss,
/// stale snapshots and raw typing remain censored observations rather than
/// negative semantic evidence.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "snake_case")]
#[repr(u8)]
pub(crate) enum TypingMemoryOutcome {
    ConfirmedPositive = 1,
    ConfirmedNegative = 2,
    Reverted = 3,
    Ambiguous = 4,
    Censored = 5,
}

impl TypingMemoryOutcome {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::ConfirmedPositive => "confirmed_positive",
            Self::ConfirmedNegative => "confirmed_negative",
            Self::Reverted => "reverted",
            Self::Ambiguous => "ambiguous",
            Self::Censored => "censored",
        }
    }

    pub(crate) const fn code(self) -> u8 {
        self as u8
    }

    pub(crate) const fn from_code(code: u8) -> Option<Self> {
        Some(match code {
            1 => Self::ConfirmedPositive,
            2 => Self::ConfirmedNegative,
            3 => Self::Reverted,
            4 => Self::Ambiguous,
            5 => Self::Censored,
            _ => return None,
        })
    }

    pub(crate) fn from_str(value: &str) -> Option<Self> {
        Some(match value {
            "confirmed_positive" => Self::ConfirmedPositive,
            "confirmed_negative" => Self::ConfirmedNegative,
            "reverted" => Self::Reverted,
            "ambiguous" => Self::Ambiguous,
            "censored" => Self::Censored,
            _ => return None,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TypingMemoryEvent {
    pub(crate) kind: TypingMemoryEventKind,
    pub(crate) feedback: TypingMemoryFeedback,
    pub(crate) outcome: TypingMemoryOutcome,
    pub(crate) word: String,
    pub(crate) context: Vec<String>,
    pub(crate) from: Option<String>,
    pub(crate) to: Option<String>,
    pub(crate) evidence_source: TypingMemoryEvidenceSource,
    pub(crate) operation: TypingMemoryOperation,
    pub(crate) identity: TypingTransitionIdentity,
    /// Stable relation shape, intentionally independent from concrete words.
    pub(crate) surface: Option<String>,
    pub(crate) completion_edit: Option<CompletionEditTrace>,
    /// Shared by every record produced by one confirmed user action.
    /// Raw typing and automatic model application remain non-authoritative and
    /// therefore carry no causal episode identity.
    pub(crate) episode_id: Option<String>,
    /// The candidate presented before the observed user outcome. Keeping it
    /// separate from `from` prevents a rollback from teaching the typo as the
    /// rejected L3 candidate.
    pub(crate) proposal: Option<String>,
}

impl TypingMemoryEvent {
    pub(crate) fn typed_tail(tail: &str) -> Option<Self> {
        let (context, word) = context_and_last_word(tail)?;
        Some(Self {
            kind: TypingMemoryEventKind::Typed,
            feedback: TypingMemoryFeedback::Observed,
            outcome: TypingMemoryOutcome::Censored,
            word,
            context,
            from: None,
            to: None,
            evidence_source: TypingMemoryEvidenceSource::User,
            operation: TypingMemoryOperation::Typed,
            identity: TypingTransitionIdentity::typed(),
            surface: None,
            completion_edit: None,
            episode_id: None,
            proposal: None,
        })
    }

    pub(crate) fn accepted_fix(from: &str, to: &str) -> Vec<Self> {
        let episode_id = next_causal_episode_id();
        accepted_events(
            TypingMemoryEventKind::AcceptedFix,
            from,
            to,
            TypingMemoryEvidenceSource::UserCorrection,
            TypingMemoryOperation::Replacement,
            TypingMemoryFeedback::Accepted,
            TypingMemoryOutcome::ConfirmedPositive,
            Some(&episode_id),
            None,
        )
    }

    pub(crate) fn accepted_layout_projection(from: &str, to: &str) -> Vec<Self> {
        let episode_id = next_causal_episode_id();
        accepted_events(
            TypingMemoryEventKind::AcceptedFix,
            from,
            to,
            TypingMemoryEvidenceSource::Layout,
            TypingMemoryOperation::Replacement,
            TypingMemoryFeedback::Accepted,
            TypingMemoryOutcome::ConfirmedPositive,
            Some(&episode_id),
            None,
        )
    }

    pub(crate) fn confirmed_user_correction(
        original: &str,
        proposal: &str,
        accepted: &str,
        operation: &str,
    ) -> Vec<Self> {
        let episode_id = next_causal_episode_id();
        let proposal = proposal.trim();
        accepted_events(
            TypingMemoryEventKind::AcceptedFix,
            original,
            accepted,
            TypingMemoryEvidenceSource::UserCorrection,
            TypingMemoryOperation::from_legacy(operation),
            TypingMemoryFeedback::Accepted,
            TypingMemoryOutcome::ConfirmedPositive,
            Some(&episode_id),
            (!proposal.is_empty()).then_some(proposal),
        )
    }

    pub(crate) fn observed_system_apply(
        from: &str,
        to: &str,
        evidence_source: TypingMemoryEvidenceSource,
        operation: TypingMemoryOperation,
    ) -> Vec<Self> {
        accepted_events(
            TypingMemoryEventKind::AcceptedFix,
            from,
            to,
            evidence_source,
            operation,
            TypingMemoryFeedback::Observed,
            TypingMemoryOutcome::Censored,
            None,
            Some(to.trim()),
        )
    }

    pub(crate) fn reverted_system_apply(
        original: &str,
        rejected: &str,
        evidence_source: TypingMemoryEvidenceSource,
        operation: TypingMemoryOperation,
    ) -> Vec<Self> {
        rejected_events(original, rejected, evidence_source, operation)
    }

    pub(crate) fn accepted_ime(context_tail: &str, accepted_text: &str) -> Vec<Self> {
        ime_events(
            TypingMemoryEventKind::AcceptedIme,
            TypingMemoryFeedback::Accepted,
            context_tail,
            accepted_text,
            TypingMemoryOperation::Completion,
        )
    }

    pub(crate) fn edited_ime(
        context_tail: &str,
        typed_prefix: &str,
        suggested_text: &str,
        final_text: &str,
    ) -> Option<Self> {
        Self::edited_ime_with_shared_identity(
            context_tail,
            typed_prefix,
            suggested_text,
            final_text,
            false,
        )
    }

    pub(crate) fn edited_ime_with_shared_identity(
        context_tail: &str,
        typed_prefix: &str,
        suggested_text: &str,
        final_text: &str,
        shared_morphology_identity: bool,
    ) -> Option<Self> {
        let suggested = single_normalized_word(suggested_text)?;
        let final_word = single_normalized_word(final_text)?;
        if suggested == final_word || !learning_target_is_attested(&final_word) {
            return None;
        }

        let prefix = normalize_memory_word(typed_prefix);
        if !completion_edit_geometry_is_linked_with_identity(
            &prefix,
            &suggested,
            &final_word,
            shared_morphology_identity,
        ) {
            return None;
        }
        let common_chars = common_prefix_chars(&suggested, &final_word);
        let reusable_prefix_chars = prefix.chars().count();
        let suggested_chars = suggested.chars().count();
        let final_chars = final_word.chars().count();
        let operation = TypingMemoryOperation::CompletionEdit;

        Some(Self {
            kind: TypingMemoryEventKind::EditedIme,
            feedback: TypingMemoryFeedback::Accepted,
            outcome: TypingMemoryOutcome::ConfirmedPositive,
            word: final_word.clone(),
            context: recent_context_words(context_tail),
            from: Some(suggested.clone()),
            to: Some(final_word.clone()),
            evidence_source: TypingMemoryEvidenceSource::Ime,
            operation: operation.clone(),
            identity: TypingTransitionIdentity::observed(
                &suggested,
                &final_word,
                operation.as_str(),
            ),
            surface: Some(transition_surface_key(
                &suggested,
                &final_word,
                "ime",
                operation.as_str(),
            )),
            completion_edit: Some(CompletionEditTrace {
                prefix,
                accepted_suffix_chars: suggested_chars.saturating_sub(reusable_prefix_chars) as u32,
                preserved_suffix_chars: common_chars.saturating_sub(reusable_prefix_chars) as u32,
                deleted_chars: suggested_chars.saturating_sub(common_chars) as u32,
                inserted_chars: final_chars.saturating_sub(common_chars) as u32,
            }),
            episode_id: Some(next_causal_episode_id()),
            proposal: Some(suggested),
        })
    }

    /// The visible IME candidate matched the word the user finished manually.
    /// This is weaker than an explicit Tab accept, but is still supervised
    /// evidence that the prediction was correct at the word boundary.
    pub(crate) fn confirmed_ime_prediction(context_tail: &str, predicted_text: &str) -> Vec<Self> {
        if !learning_target_is_attested(predicted_text) {
            return Vec::new();
        }
        ime_events(
            TypingMemoryEventKind::ConfirmedImePrediction,
            TypingMemoryFeedback::Accepted,
            context_tail,
            predicted_text,
            TypingMemoryOperation::PredictionMatch,
        )
    }

    pub(crate) fn rejected_ime(context_tail: &str, rejected_text: &str) -> Vec<Self> {
        ime_events(
            TypingMemoryEventKind::RejectedIme,
            TypingMemoryFeedback::Rejected,
            context_tail,
            rejected_text,
            TypingMemoryOperation::Completion,
        )
    }

    pub(crate) fn rejected_candidate(
        context_tail: &str,
        rejected_text: &str,
        source: &str,
        operation: &str,
    ) -> Vec<Self> {
        rejected_events(
            context_tail,
            rejected_text,
            TypingMemoryEvidenceSource::from_legacy(source),
            TypingMemoryOperation::from_legacy(operation),
        )
    }
}

pub(crate) fn completion_edit_geometry_is_linked(
    typed_prefix: &str,
    suggested_text: &str,
    final_text: &str,
) -> bool {
    completion_edit_geometry_is_linked_with_identity(
        typed_prefix,
        suggested_text,
        final_text,
        false,
    )
}

pub(crate) fn completion_edit_geometry_is_linked_with_identity(
    typed_prefix: &str,
    suggested_text: &str,
    final_text: &str,
    shared_morphology_identity: bool,
) -> bool {
    const MAX_ENDING_EDIT_CHARS: usize = 5;
    const MIN_STEM_ADVANTAGE_CHARS: usize = 3;

    let prefix = normalize_memory_word(typed_prefix);
    let suggested = normalize_memory_word(suggested_text);
    let final_word = normalize_memory_word(final_text);
    if prefix.is_empty()
        || suggested == final_word
        || !suggested.starts_with(&prefix)
        || !final_word.starts_with(&prefix)
    {
        return false;
    }
    let common_chars = common_prefix_chars(&suggested, &final_word);
    let prefix_chars = prefix.chars().count();
    let changed_tail_chars = suggested
        .chars()
        .count()
        .saturating_sub(common_chars)
        .max(final_word.chars().count().saturating_sub(common_chars));
    common_chars > prefix_chars
        && (shared_morphology_identity
            || (changed_tail_chars <= MAX_ENDING_EDIT_CHARS
                && common_chars >= changed_tail_chars.saturating_add(MIN_STEM_ADVANTAGE_CHARS)))
}

fn rejected_events(
    original: &str,
    rejected: &str,
    evidence_source: TypingMemoryEvidenceSource,
    operation: TypingMemoryOperation,
) -> Vec<TypingMemoryEvent> {
    let original_words = normalized_words(original);
    let rejected_words = normalized_words(rejected);
    let episode_id = next_causal_episode_id();
    changed_target_indexes(&original_words, &rejected_words)
        .into_iter()
        .filter_map(|index| {
            let word = rejected_words.get(index)?.clone();
            let context = words_before_last(&rejected_words[..index]);
            Some(TypingMemoryEvent {
                kind: TypingMemoryEventKind::RejectedCandidate,
                feedback: TypingMemoryFeedback::Rejected,
                outcome: TypingMemoryOutcome::Reverted,
                word,
                context,
                from: Some(original.trim().to_string()),
                to: Some(rejected.trim().to_string()),
                evidence_source: evidence_source.clone(),
                operation: operation.clone(),
                identity: TypingTransitionIdentity::observed(
                    original,
                    rejected,
                    operation.as_str(),
                ),
                surface: Some(transition_surface_key(
                    original,
                    rejected,
                    evidence_source.as_str(),
                    operation.as_str(),
                )),
                completion_edit: None,
                episode_id: Some(episode_id.clone()),
                proposal: Some(rejected.trim().to_string()),
            })
        })
        .collect()
}

fn accepted_events(
    kind: TypingMemoryEventKind,
    from: &str,
    to: &str,
    evidence_source: TypingMemoryEvidenceSource,
    operation: TypingMemoryOperation,
    feedback: TypingMemoryFeedback,
    outcome: TypingMemoryOutcome,
    episode_id: Option<&str>,
    proposal: Option<&str>,
) -> Vec<TypingMemoryEvent> {
    let to_words = normalized_words(to);
    if to_words.is_empty() {
        return Vec::new();
    }
    let from_words = normalized_words(from);
    let target_indexes = changed_target_indexes(&from_words, &to_words);
    target_indexes
        .into_iter()
        .map(|index| {
            let word = to_words[index].clone();
            let context = words_before_last(&to_words[..index]);
            TypingMemoryEvent {
                kind,
                feedback,
                outcome,
                word,
                context,
                from: Some(from.trim().to_string()),
                to: Some(to.trim().to_string()),
                evidence_source: evidence_source.clone(),
                operation: operation.clone(),
                identity: TypingTransitionIdentity::observed(from, to, operation.as_str()),
                surface: Some(transition_surface_key(
                    from,
                    to,
                    evidence_source.as_str(),
                    operation.as_str(),
                )),
                completion_edit: None,
                episode_id: episode_id.map(str::to_owned),
                proposal: proposal.map(str::to_owned),
            }
        })
        .collect()
}

pub(crate) fn changed_target_indexes(from_words: &[String], to_words: &[String]) -> Vec<usize> {
    let indexes = to_words
        .iter()
        .enumerate()
        .filter_map(|(index, word)| (from_words.get(index) != Some(word)).then_some(index))
        .collect::<Vec<_>>();
    if indexes.is_empty() {
        to_words.len().checked_sub(1).into_iter().collect()
    } else {
        indexes
    }
}

/// Normalized target region beginning at the first changed token.
///
/// The unchanged phrase prefix belongs to L3 context, not to the L4 operator
/// identity. Boundary transitions keep every changed target token, so a split
/// cannot collapse back to a last-word lookup.
pub(crate) fn transition_target_text(from: &str, to: &str) -> String {
    let from_words = normalized_words(from);
    let to_words = normalized_words(to);
    let start = changed_target_indexes(&from_words, &to_words)
        .into_iter()
        .next()
        .unwrap_or_else(|| to_words.len().saturating_sub(1));
    to_words.get(start..).unwrap_or_default().join(" ")
}

pub(crate) fn transition_context_words(from: &str, to: &str) -> Vec<String> {
    let from_words = normalized_words(from);
    let to_words = normalized_words(to);
    let start = changed_target_indexes(&from_words, &to_words)
        .into_iter()
        .next()
        .unwrap_or_else(|| from_words.len().saturating_sub(1));
    words_before_last(&from_words[..start.min(from_words.len())])
}

fn ime_events(
    kind: TypingMemoryEventKind,
    feedback: TypingMemoryFeedback,
    context_tail: &str,
    text: &str,
    operation: TypingMemoryOperation,
) -> Vec<TypingMemoryEvent> {
    let context = recent_context_words(context_tail);
    let episode_id = next_causal_episode_id();
    let identity = if operation == TypingMemoryOperation::PredictionMatch {
        TypingTransitionIdentity::no_mutation()
    } else {
        TypingTransitionIdentity::observed(context_tail, text, operation.as_str())
    };
    normalized_words(text)
        .into_iter()
        .map(|word| TypingMemoryEvent {
            kind,
            feedback,
            outcome: match feedback {
                TypingMemoryFeedback::Accepted => TypingMemoryOutcome::ConfirmedPositive,
                TypingMemoryFeedback::Rejected => TypingMemoryOutcome::Reverted,
                TypingMemoryFeedback::Observed => TypingMemoryOutcome::Censored,
            },
            word,
            context: context.clone(),
            from: None,
            to: Some(text.trim().to_string()),
            evidence_source: TypingMemoryEvidenceSource::Ime,
            operation: operation.clone(),
            identity,
            surface: None,
            completion_edit: None,
            episode_id: Some(episode_id.clone()),
            proposal: None,
        })
        .collect()
}

fn next_causal_episode_id() -> String {
    let time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_micros();
    let ordinal = NEXT_CAUSAL_EPISODE.fetch_add(1, Ordering::Relaxed);
    format!("{}-{time}-{ordinal}", std::process::id())
}

fn single_normalized_word(text: &str) -> Option<String> {
    let mut words = normalized_words(text).into_iter();
    let word = words.next()?;
    words.next().is_none().then_some(word)
}

fn common_prefix_chars(left: &str, right: &str) -> usize {
    left.chars()
        .zip(right.chars())
        .take_while(|(left, right)| left == right)
        .count()
}

fn layout_projection_direction(from: &str, to: &str) -> LayoutProjectionDirection {
    let source = crate::transition_relation::script_family(from);
    let target = crate::transition_relation::script_family(to);
    match (source, target) {
        ("en", "ru") => LayoutProjectionDirection::EnToRu,
        ("ru", "en") => LayoutProjectionDirection::RuToEn,
        ("mixed", "ru") => LayoutProjectionDirection::MixedToRu,
        ("mixed", "en") => LayoutProjectionDirection::MixedToEn,
        _ => LayoutProjectionDirection::Unknown,
    }
}

fn layout_projection_scope(from: &str, to: &str) -> LayoutProjectionScope {
    let from_words = normalized_words(from);
    let to_words = normalized_words(to);
    let changed = changed_target_indexes(&from_words, &to_words);
    if changed.len() > 1 {
        return LayoutProjectionScope::Phrase;
    }
    let from_word = from_words.last().map(String::as_str).unwrap_or_default();
    let to_word = to_words.last().map(String::as_str).unwrap_or_default();
    if from_word.chars().count() == 1 && to_word.chars().count() == 1 {
        LayoutProjectionScope::Grapheme
    } else {
        LayoutProjectionScope::CurrentToken
    }
}

pub(crate) fn transition_surface_key(
    from: &str,
    to: &str,
    _source: &str,
    operation: &str,
) -> String {
    let (_, relation) =
        crate::transition_relation::TransitionRelationAtoms::inferred(from, to, operation);
    relation.surface_key().to_string()
}

pub(crate) fn context_and_last_word(text: &str) -> Option<(Vec<String>, String)> {
    let words = normalized_words(text);
    let (word, context) = words.split_last()?;
    Some((words_before_last(context), word.clone()))
}

pub(crate) fn recent_context_words(text: &str) -> Vec<String> {
    normalized_words(text)
        .into_iter()
        .rev()
        .take(CONTEXT_WORDS)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect()
}

pub(crate) fn normalized_words(text: &str) -> Vec<String> {
    text.split_whitespace()
        .filter_map(|token| {
            let word = normalize_memory_word(token);
            (!word.is_empty()).then_some(word)
        })
        .collect()
}

/// Raw phrase observations may train L3 only from attested lexical surfaces.
/// Accepted IME/correction outcomes have their own explicit training path.
pub(crate) fn phrase_is_attested_for_learning(text: &str) -> bool {
    let words = normalized_words(text);
    !words.is_empty()
        && words.into_iter().all(|word| {
            if word.chars().all(|ch| ch.is_ascii_alphabetic() || ch == '-') {
                crate::layout_autoswitch::is_known_english_layout_autoswitch_word(&word)
            } else {
                crate::hot_field::HotFieldSnapshot::current().learning_surface_is_attested(&word)
                    || crate::russian_lexicon::is_known_russian_word_or_form(&word)
                    || crate::russian_lexicon::is_reference_backed_russian_form(&word)
            }
        })
}

/// Strong admission gate for explicit IME feedback. Generated morphology may
/// help readout, but it is not enough to turn a user's typo into positive
/// evidence. The final surface must already exist in an exact lexical bank.
pub(crate) fn learning_target_is_attested(text: &str) -> bool {
    let words = normalized_words(text);
    !words.is_empty()
        && words.into_iter().all(|word| {
            if word.chars().all(|ch| ch.is_ascii_alphabetic() || ch == '-') {
                crate::layout_autoswitch::is_known_english_layout_autoswitch_word(&word)
            } else {
                crate::hot_field::HotFieldSnapshot::current().learning_surface_is_attested(&word)
                    || crate::nanda_wave::l2::l2_decoder_contains_surface(&word)
            }
        })
}

pub(crate) fn normalize_memory_word(word: &str) -> String {
    let trimmed = word
        .trim()
        .trim_matches(|ch: char| !ch.is_alphabetic() && ch != '-');
    if !trimmed.chars().any(|ch| ch.is_alphabetic()) {
        return String::new();
    }
    trimmed.to_lowercase()
}

fn words_before_last(words: &[String]) -> Vec<String> {
    words
        .iter()
        .rev()
        .take(CONTEXT_WORDS)
        .cloned()
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn typed_tail_event_keeps_recent_context_and_word() {
        let event = TypingMemoryEvent::typed_tail("на улице опять идёт дождь ").unwrap();

        assert_eq!(event.kind, TypingMemoryEventKind::Typed);
        assert_eq!(event.feedback, TypingMemoryFeedback::Observed);
        assert_eq!(event.outcome, TypingMemoryOutcome::Censored);
        assert_eq!(event.word, "дождь");
        assert_eq!(event.context, ["на", "улице", "опять", "идёт"]);
    }

    #[test]
    fn raw_phrase_learning_requires_attested_surfaces() {
        assert!(phrase_is_attested_for_learning("на улице идёт дождь"));
        assert!(phrase_is_attested_for_learning(
            "Вообще делай проект рефакторинга"
        ));
        assert!(!phrase_is_attested_for_learning("на улице идёт дожть"));
        assert!(!phrase_is_attested_for_learning("звгрузи пакет"));
    }

    #[test]
    fn production_log_typos_are_not_attested_learning_targets() {
        let valid = [
            "прекрасно",
            "хостинге",
            "зарегистрированы",
            "режиме",
            "видишь",
            "переписки",
            "посмотреть",
        ];
        let missed = valid
            .into_iter()
            .filter(|word| !learning_target_is_attested(word))
            .collect::<Vec<_>>();
        assert!(
            missed.is_empty(),
            "valid forms are not attested: {missed:?}"
        );
        let typos = [
            "зарегестрированы",
            "такм",
            "режимем",
            "ивдешь",
            "перписки",
            "апосмотреть",
        ];
        let leaked = typos
            .into_iter()
            .filter(|typo| learning_target_is_attested(typo))
            .collect::<Vec<_>>();
        assert!(
            leaked.is_empty(),
            "production typos entered learning authority: {leaked:?}"
        );
    }

    #[test]
    fn accepted_fix_events_mark_positive_result_and_source() {
        let events = TypingMemoryEvent::accepted_fix("мы отвравим", "мы отравим");

        assert!(events.iter().any(|event| event.word == "отравим"));
        assert!(events
            .iter()
            .all(|event| event.feedback == TypingMemoryFeedback::Accepted));
        assert!(events
            .iter()
            .all(|event| event.outcome == TypingMemoryOutcome::ConfirmedPositive));
        assert!(events
            .iter()
            .all(|event| { event.evidence_source == TypingMemoryEvidenceSource::UserCorrection }));
        assert!(events
            .iter()
            .all(|event| event.operation == TypingMemoryOperation::Replacement));
        assert!(events.iter().all(|event| event.surface.is_some()));
        assert!(events[0].episode_id.is_some());
        assert!(events
            .iter()
            .all(|event| event.episode_id == events[0].episode_id));
    }

    #[test]
    fn confirmed_user_correction_keeps_proposal_and_one_episode() {
        let events = TypingMemoryEvent::confirmed_user_correction(
            "читай новсти",
            "читай новость",
            "читай новости",
            "ime_auto_undo",
        );

        assert_eq!(events.len(), 1);
        assert_eq!(events[0].proposal.as_deref(), Some("читай новость"));
        assert_eq!(events[0].operation, TypingMemoryOperation::ImeAutoUndo);
        assert!(events[0].episode_id.is_some());
        assert_eq!(events[0].outcome, TypingMemoryOutcome::ConfirmedPositive);
    }

    #[test]
    fn automatic_apply_is_censored_and_has_no_causal_episode() {
        let events = TypingMemoryEvent::observed_system_apply(
            "читай новсти",
            "читай новости",
            TypingMemoryEvidenceSource::Autocorrect,
            TypingMemoryOperation::Diagnostic("typing-assist".to_string()),
        );

        assert_eq!(events.len(), 1);
        assert_eq!(events[0].feedback, TypingMemoryFeedback::Observed);
        assert_eq!(events[0].outcome, TypingMemoryOutcome::Censored);
        assert_eq!(
            events[0].evidence_source,
            TypingMemoryEvidenceSource::Autocorrect
        );
        assert_eq!(events[0].episode_id, None);
    }

    #[test]
    fn confirmed_ime_prediction_is_positive_but_not_an_explicit_completion() {
        let events = TypingMemoryEvent::confirmed_ime_prediction("ну", "да");

        assert_eq!(events.len(), 1);
        assert_eq!(
            events[0].kind,
            TypingMemoryEventKind::ConfirmedImePrediction
        );
        assert_eq!(events[0].feedback, TypingMemoryFeedback::Accepted);
        assert_eq!(events[0].evidence_source, TypingMemoryEvidenceSource::Ime);
        assert_eq!(events[0].operation, TypingMemoryOperation::PredictionMatch);
        assert_eq!(events[0].word, "да");
        assert_eq!(
            events[0].identity.operator,
            crate::transition_relation::TransitionOperatorKind::Other
        );
    }

    #[test]
    fn edited_ime_preserves_partial_completion_geometry() {
        let event = TypingMemoryEvent::edited_ime("это было", "прек", "прекрасный", "прекрасно")
            .expect("one edited completion event");

        assert_eq!(event.kind, TypingMemoryEventKind::EditedIme);
        assert_eq!(event.proposal.as_deref(), Some("прекрасный"));
        assert!(event.episode_id.is_some());
        assert_eq!(event.word, "прекрасно");
        assert_eq!(event.context, ["это", "было"]);
        assert_eq!(event.from.as_deref(), Some("прекрасный"));
        assert_eq!(event.to.as_deref(), Some("прекрасно"));
        assert_eq!(event.evidence_source, TypingMemoryEvidenceSource::Ime);
        assert_eq!(event.operation, TypingMemoryOperation::CompletionEdit);
        let trace = event
            .completion_edit
            .expect("typed completion edit geometry");
        assert_eq!(trace.prefix, "прек");
        assert_eq!(trace.accepted_suffix_chars, 6);
        assert_eq!(trace.preserved_suffix_chars, 4);
        assert_eq!(trace.deleted_chars, 2);
        assert_eq!(trace.inserted_chars, 1);
    }

    #[test]
    fn completion_edit_requires_a_shared_stem_not_only_a_shared_typed_prefix() {
        assert!(completion_edit_geometry_is_linked(
            "прек",
            "прекрасный",
            "прекрасно"
        ));
        assert!(completion_edit_geometry_is_linked(
            "пос",
            "посмотреть",
            "посмотри"
        ));
        assert!(!completion_edit_geometry_is_linked(
            "прек",
            "прекрасный",
            "прекратить"
        ));
        assert!(completion_edit_geometry_is_linked_with_identity(
            "к",
            "котами",
            "коты",
            true
        ));
    }

    #[test]
    fn ime_feedback_rejects_unattested_targets_and_forged_prefixes() {
        assert!(TypingMemoryEvent::edited_ime(
            "в логах косяков не",
            "ив",
            "использовать",
            "ивдешь"
        )
        .is_none());
        assert!(
            TypingMemoryEvent::edited_ime("это было", "чужой", "прекрасный", "прекрасно").is_none()
        );
        assert!(TypingMemoryEvent::confirmed_ime_prediction("ты проверь", "такм").is_empty());
    }

    #[test]
    fn rejected_candidate_events_are_negative_transition_material() {
        let events = TypingMemoryEvent::rejected_candidate(
            "ну исходник",
            "ну даша",
            "L2LiveCandidateGate32",
            "completion",
        );

        assert_eq!(events.len(), 1);
        assert_eq!(events[0].kind, TypingMemoryEventKind::RejectedCandidate);
        assert_eq!(events[0].feedback, TypingMemoryFeedback::Rejected);
        assert_eq!(events[0].outcome, TypingMemoryOutcome::Reverted);
        assert_eq!(events[0].context, ["ну"]);
        assert_eq!(events[0].word, "даша");
        assert!(events[0].surface.is_some());
    }

    #[test]
    fn reverted_system_apply_keeps_the_rejected_proposal_and_typed_source() {
        let events = TypingMemoryEvent::reverted_system_apply(
            "обновлять модель по ходу ",
            "обновлять модель по ход ",
            TypingMemoryEvidenceSource::Autocorrect,
            TypingMemoryOperation::Replacement,
        );

        assert_eq!(events.len(), 1);
        assert_eq!(events[0].kind, TypingMemoryEventKind::RejectedCandidate);
        assert_eq!(events[0].feedback, TypingMemoryFeedback::Rejected);
        assert_eq!(events[0].outcome, TypingMemoryOutcome::Reverted);
        assert_eq!(events[0].word, "ход");
        assert_eq!(events[0].context, ["обновлять", "модель", "по"]);
        assert_eq!(events[0].from.as_deref(), Some("обновлять модель по ходу"));
        assert_eq!(events[0].to.as_deref(), Some("обновлять модель по ход"));
        assert_eq!(
            events[0].proposal.as_deref(),
            Some("обновлять модель по ход")
        );
        assert_eq!(
            events[0].evidence_source,
            TypingMemoryEvidenceSource::Autocorrect
        );
        assert_eq!(events[0].operation, TypingMemoryOperation::Replacement);
        assert!(events[0].episode_id.is_some());
    }

    #[test]
    fn rejected_candidate_excludes_unchanged_left_context() {
        let events = TypingMemoryEvent::rejected_candidate(
            "как попусы",
            "как опусы",
            "typing-assist",
            "replacement",
        );

        assert_eq!(events.len(), 1);
        assert_eq!(events[0].word, "опусы");
        assert_eq!(events[0].context, ["как"]);
        assert_eq!(events[0].from.as_deref(), Some("как попусы"));
        assert_eq!(events[0].to.as_deref(), Some("как опусы"));
    }

    #[test]
    fn accepted_layout_projection_uses_the_runtime_operator_identity() {
        let events = TypingMemoryEvent::accepted_layout_projection("ltkfq", "делай");

        assert_eq!(events.len(), 1);
        assert_eq!(events[0].kind, TypingMemoryEventKind::AcceptedFix);
        assert_eq!(
            events[0].evidence_source,
            TypingMemoryEvidenceSource::Layout
        );
        assert_eq!(events[0].operation, TypingMemoryOperation::Replacement);
        assert_eq!(events[0].from.as_deref(), Some("ltkfq"));
        assert_eq!(events[0].to.as_deref(), Some("делай"));
        assert_eq!(
            events[0].identity.operator,
            crate::transition_relation::TransitionOperatorKind::LayoutProjection
        );
        assert_eq!(
            events[0].identity.layout_direction,
            Some(LayoutProjectionDirection::EnToRu)
        );
        assert_eq!(
            events[0].identity.layout_scope,
            Some(LayoutProjectionScope::CurrentToken)
        );
        assert_eq!(
            events[0].identity.learning_key(),
            "layout_projection:en_to_ru:current_token"
        );
    }

    #[test]
    fn layout_identity_is_bidirectional_and_marks_single_grapheme_scope() {
        let ru_to_en = TypingMemoryEvent::accepted_layout_projection("делай", "ltkfq");
        let grapheme = TypingMemoryEvent::accepted_layout_projection("b", "и");

        assert_eq!(
            ru_to_en[0].identity.layout_direction,
            Some(LayoutProjectionDirection::RuToEn)
        );
        assert_eq!(
            grapheme[0].identity.layout_direction,
            Some(LayoutProjectionDirection::EnToRu)
        );
        assert_eq!(
            grapheme[0].identity.layout_scope,
            Some(LayoutProjectionScope::Grapheme)
        );
    }

    #[test]
    fn transition_target_keeps_all_changed_boundary_tokens() {
        assert_eq!(transition_target_text("мыслов", "мы слов"), "мы слов");
        assert_eq!(
            transition_target_text("мы отвравим", "мы отравим"),
            "отравим"
        );
        assert_eq!(
            transition_context_words("мы отвравим", "мы отравим"),
            ["мы"]
        );
    }

    #[test]
    fn transition_surface_key_generalizes_across_concrete_words() {
        let first = transition_surface_key("мы cat", "мы car", "autocorrect", "replacement");
        let second = transition_surface_key("они dog", "они dot", "autocorrect", "replacement");

        assert_eq!(first, second);
    }
}

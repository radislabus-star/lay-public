use std::collections::{HashMap, HashSet};
#[cfg(test)]
use std::mem;
use std::path::{Path, PathBuf};
#[cfg(not(test))]
use std::sync::mpsc::{self, RecvTimeoutError, SyncSender, TrySendError};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::{Duration, Instant};

use crate::time::unix_timestamp;
#[cfg(test)]
use crate::typing_memory;
use crate::typing_memory::{
    normalize_memory_word, normalized_words, CompletionEditTrace, LayoutProjectionDirection,
    LayoutProjectionScope, TypingMemoryEvent, TypingMemoryEventKind, TypingMemoryEvidenceSource,
    TypingMemoryOperation, TypingMemoryOutcome,
};
use crate::typing_scene::{
    KeyboardGeometryId, LanguageId, LayoutId, SceneIdentityEvidence, ScriptFamily,
};

mod hot;
mod projection;

pub(crate) use hot::{
    UsageCandidatePrior, UsageContextCandidate, UsageHotContext, UsageHotReadout,
    UsageSurfaceCoverage, UsageTransitionSignal,
};
use hot::{UsageHotState, CONTEXT_WORDS, MIN_CONTEXT_NGRAM};
use projection::{UsageEventProjection, TRANSITION_ANY};

#[cfg(not(test))]
const USAGE_EVENTS_PATH: &str = ".local/share/lay/nanda_wave/word_usage_events.jsonl";
#[cfg(not(test))]
const USAGE_COUNTS_PATH: &str = ".local/share/lay/nanda_wave/word_usage_counts.json";
#[cfg(not(test))]
const USAGE_FEEDBACK_COUNTS_PATH: &str =
    ".local/share/lay/nanda_wave/word_usage_feedback_counts.json";
#[cfg(not(test))]
const LEGACY_USAGE_PRIOR_PATH: &str = ".local/share/lay/learning_candidates.json";
const USAGE_EVENTS_MAX_BYTES: u64 = 500 * 1024;
const USAGE_EVENTS_FULL_REBUILD_MAX_BYTES: u64 = 8 * 1024 * 1024;
const USAGE_COUNTS_SCHEMA_VERSION: u32 = 15;
const USAGE_COUNTS_MAX_WORDS: usize = 10_000;
const USAGE_COUNTS_MAX_ACCEPTED_WORDS: usize = 5_000;
const USAGE_COUNTS_MAX_CONTEXT_WORDS: usize = 12_000;
const USAGE_COUNTS_MAX_REJECTED_WORDS: usize = 5_000;
const USAGE_COUNTS_MAX_REJECTED_CONTEXT_WORDS: usize = 12_000;
const USAGE_COUNTS_MAX_TRANSITION_STATES: usize = 24_000;
const USAGE_REFRESH_INTERVAL: Duration = Duration::from_millis(1000);
#[cfg(not(test))]
const USAGE_PERSIST_INTERVAL: Duration = Duration::from_millis(1000);
#[cfg(not(test))]
const USAGE_PERSIST_CHANNEL_CAPACITY: usize = 8192;
#[cfg(not(test))]
const USAGE_PERSIST_PENDING_MAX_BYTES: usize = 64 * 1024;
const TYPED_EVENT_SCHEMA_V2: u8 = 2;
const TYPED_EVENT_SCHEMA_V3: u8 = 3;

#[derive(Debug, serde::Deserialize)]
struct LearningCandidate {
    to: String,
    count: u32,
    #[serde(default)]
    promoted: bool,
}

#[derive(Debug, Clone, Default, serde::Deserialize, serde::Serialize)]
struct UsageEvent {
    ts: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    schema: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    episode_id: Option<String>,
    kind: UsageEventKind,
    #[serde(skip_serializing_if = "Option::is_none")]
    word: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    context: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    from: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    to: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    source: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    operation: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    surface: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    operator: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    layout_direction: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    layout_scope: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    source_language: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    target_language: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    source_layout: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    target_layout: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    source_script: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    target_script: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    keyboard_geometry: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    identity_evidence: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    sentence_language: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    outcome: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    evidence_source_code: Option<u8>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    operation_code: Option<u8>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    operator_code: Option<u8>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    layout_direction_code: Option<u8>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    layout_scope_code: Option<u8>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    source_language_id: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    target_language_id: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    source_layout_id: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    target_layout_id: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    source_script_code: Option<u8>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    target_script_code: Option<u8>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    keyboard_geometry_id: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    identity_evidence_code: Option<u8>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    sentence_language_id: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    sentence_language_support_milli: Option<u16>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    sentence_language_alternative_milli: Option<u16>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    sentence_language_observed_tokens: Option<u8>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    outcome_code: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    completion_edit: Option<CompletionEditTrace>,
    #[serde(skip_serializing_if = "Option::is_none")]
    proposal: Option<String>,
}

#[derive(Debug, Default, serde::Deserialize)]
struct CorrectionFeedbackReceipt {
    #[serde(default)]
    kind: String,
    #[serde(default)]
    lay_kind: String,
    #[serde(default)]
    lay_from: String,
    #[serde(default)]
    lay_to: String,
    #[serde(default)]
    from: String,
    #[serde(default)]
    to: String,
    #[serde(default)]
    user_target: Option<String>,
}

impl UsageEvent {
    fn from_typing_memory_event(event: &TypingMemoryEvent) -> Self {
        let scene = event.identity.scene;
        let sentence = event.sentence_language;
        Self {
            ts: unix_timestamp(),
            schema: Some(TYPED_EVENT_SCHEMA_V3),
            episode_id: event.episode_id.clone(),
            kind: match event.kind {
                TypingMemoryEventKind::Typed => UsageEventKind::Typed,
                TypingMemoryEventKind::AcceptedFix => UsageEventKind::AcceptedFix,
                TypingMemoryEventKind::AcceptedIme => UsageEventKind::AcceptedIme,
                TypingMemoryEventKind::EditedIme => UsageEventKind::EditedIme,
                TypingMemoryEventKind::ConfirmedImePrediction => {
                    UsageEventKind::ConfirmedImePrediction
                }
                TypingMemoryEventKind::RejectedIme => UsageEventKind::RejectedIme,
                TypingMemoryEventKind::RejectedCandidate => UsageEventKind::RejectedCandidate,
            },
            word: Some(event.word.clone()),
            context: event.context.clone(),
            from: event.from.clone(),
            to: event.to.clone(),
            source: Some(event.evidence_source.as_str().to_string()),
            operation: Some(event.operation.as_str().to_string()),
            surface: event.surface.clone(),
            operator: Some(event.identity.learning_key()),
            layout_direction: event
                .identity
                .layout_direction
                .map(|direction| direction.as_str().to_string()),
            layout_scope: event
                .identity
                .layout_scope
                .map(|scope| scope.as_str().to_string()),
            source_language: scene.source_language.known_label().map(str::to_string),
            target_language: scene.target_language.known_label().map(str::to_string),
            source_layout: scene.source_layout.known_label().map(str::to_string),
            target_layout: scene.target_layout.known_label().map(str::to_string),
            source_script: Some(scene.source_script.as_str().to_string()),
            target_script: Some(scene.target_script.as_str().to_string()),
            keyboard_geometry: scene.keyboard_geometry.known_label().map(str::to_string),
            identity_evidence: Some(scene.evidence.as_str().to_string()),
            sentence_language: sentence.language.known_label().map(str::to_string),
            outcome: Some(event.outcome.as_str().to_string()),
            evidence_source_code: Some(event.evidence_source.code()),
            operation_code: Some(event.operation.code()),
            operator_code: Some(event.identity.operator as u8),
            layout_direction_code: event
                .identity
                .layout_direction
                .map(LayoutProjectionDirection::code),
            layout_scope_code: event.identity.layout_scope.map(LayoutProjectionScope::code),
            source_language_id: (!scene.source_language.is_unknown())
                .then_some(scene.source_language.code()),
            target_language_id: (!scene.target_language.is_unknown())
                .then_some(scene.target_language.code()),
            source_layout_id: (!scene.source_layout.is_unknown())
                .then_some(scene.source_layout.code()),
            target_layout_id: (!scene.target_layout.is_unknown())
                .then_some(scene.target_layout.code()),
            source_script_code: Some(scene.source_script.code()),
            target_script_code: Some(scene.target_script.code()),
            keyboard_geometry_id: (!scene.keyboard_geometry.is_unknown())
                .then_some(scene.keyboard_geometry.code()),
            identity_evidence_code: Some(scene.evidence.code()),
            sentence_language_id: (!sentence.language.is_unknown())
                .then_some(sentence.language.code()),
            sentence_language_support_milli: Some(sentence.support_milli),
            sentence_language_alternative_milli: Some(sentence.alternative_milli),
            sentence_language_observed_tokens: Some(sentence.observed_tokens),
            outcome_code: Some(event.outcome.code()),
            completion_edit: event.completion_edit.clone(),
            proposal: event.proposal.clone(),
        }
    }

    fn enrich_typed_v2(&mut self) {
        self.schema = Some(TYPED_EVENT_SCHEMA_V2);
        self.evidence_source_code = self
            .source
            .as_deref()
            .map(TypingMemoryEvidenceSource::from_legacy)
            .map(|source| source.code());
        self.operation_code = self
            .operation
            .as_deref()
            .map(TypingMemoryOperation::from_legacy)
            .map(|operation| operation.code());
        self.operator_code = self
            .operator
            .as_deref()
            .and_then(crate::transition_relation::TransitionOperatorKind::from_str)
            .map(|operator| operator as u8);
        self.layout_direction_code = self
            .layout_direction
            .as_deref()
            .and_then(LayoutProjectionDirection::from_str)
            .map(LayoutProjectionDirection::code);
        self.layout_scope_code = self
            .layout_scope
            .as_deref()
            .and_then(LayoutProjectionScope::from_str)
            .map(LayoutProjectionScope::code);
        self.outcome_code = self
            .outcome
            .as_deref()
            .and_then(TypingMemoryOutcome::from_str)
            .map(TypingMemoryOutcome::code);
    }

    fn typed_v2_is_consistent(&self) -> bool {
        if self.schema != Some(TYPED_EVENT_SCHEMA_V2) {
            return false;
        }
        self.typed_base_is_consistent()
    }

    fn typed_v3_is_consistent(&self) -> bool {
        if self.schema != Some(TYPED_EVENT_SCHEMA_V3) || !self.typed_base_is_consistent() {
            return false;
        }
        let scene_ids_ok = typed_optional_symbol_matches(
            self.source_language_id,
            self.source_language.as_deref(),
            LanguageId::from_label,
            LanguageId::code,
        ) && typed_optional_symbol_matches(
            self.target_language_id,
            self.target_language.as_deref(),
            LanguageId::from_label,
            LanguageId::code,
        ) && typed_optional_symbol_matches(
            self.source_layout_id,
            self.source_layout.as_deref(),
            LayoutId::from_label,
            LayoutId::code,
        ) && typed_optional_symbol_matches(
            self.target_layout_id,
            self.target_layout.as_deref(),
            LayoutId::from_label,
            LayoutId::code,
        ) && typed_optional_symbol_matches(
            self.keyboard_geometry_id,
            self.keyboard_geometry.as_deref(),
            KeyboardGeometryId::from_label,
            KeyboardGeometryId::code,
        ) && typed_optional_symbol_matches(
            self.sentence_language_id,
            self.sentence_language.as_deref(),
            LanguageId::from_label,
            LanguageId::code,
        );
        let typed_scene_ok = typed_optional_code_matches(
            self.source_script_code,
            self.source_script.as_deref(),
            ScriptFamily::from_code,
            ScriptFamily::from_str,
        ) && typed_optional_code_matches(
            self.target_script_code,
            self.target_script.as_deref(),
            ScriptFamily::from_code,
            ScriptFamily::from_str,
        ) && typed_optional_code_matches(
            self.identity_evidence_code,
            self.identity_evidence.as_deref(),
            SceneIdentityEvidence::from_code,
            SceneIdentityEvidence::from_str,
        );
        let sentence_evidence_ok = matches!(
            (
                self.sentence_language_support_milli,
                self.sentence_language_alternative_milli,
                self.sentence_language_observed_tokens,
            ),
            (Some(support), Some(alternative), Some(_))
                if support <= 1_000 && alternative <= 1_000
        );
        scene_ids_ok && typed_scene_ok && sentence_evidence_ok
    }

    fn typed_base_is_consistent(&self) -> bool {
        let source_ok = match (self.evidence_source_code, self.source.as_deref()) {
            (Some(code), Some(source)) => {
                TypingMemoryEvidenceSource::from_legacy(source).code() == code
            }
            (None, None) => true,
            _ => false,
        };
        let operation_ok = match (self.operation_code, self.operation.as_deref()) {
            (Some(code), Some(operation)) => {
                TypingMemoryOperation::from_legacy(operation).code() == code
            }
            (None, None) => true,
            _ => false,
        };
        let operator_ok = match (self.operator_code, self.operator.as_deref()) {
            (Some(code), Some(operator)) => {
                crate::transition_relation::TransitionOperatorKind::from_code(code)
                    == crate::transition_relation::TransitionOperatorKind::from_str(operator)
            }
            (None, None) => true,
            _ => false,
        };
        let direction_ok = typed_optional_code_matches(
            self.layout_direction_code,
            self.layout_direction.as_deref(),
            LayoutProjectionDirection::from_code,
            LayoutProjectionDirection::from_str,
        );
        let scope_ok = typed_optional_code_matches(
            self.layout_scope_code,
            self.layout_scope.as_deref(),
            LayoutProjectionScope::from_code,
            LayoutProjectionScope::from_str,
        );
        let outcome_ok = typed_optional_code_matches(
            self.outcome_code,
            self.outcome.as_deref(),
            TypingMemoryOutcome::from_code,
            TypingMemoryOutcome::from_str,
        );
        source_ok && operation_ok && operator_ok && direction_ok && scope_ok && outcome_ok
    }
}

fn typed_optional_code_matches<T: PartialEq>(
    code: Option<u8>,
    label: Option<&str>,
    from_code: impl Fn(u8) -> Option<T>,
    from_str: impl Fn(&str) -> Option<T>,
) -> bool {
    match (code, label) {
        (Some(code), Some(label)) => from_code(code) == from_str(label),
        (None, None) => true,
        _ => false,
    }
}

fn typed_optional_symbol_matches<T: Copy>(
    code: Option<u64>,
    label: Option<&str>,
    from_label: impl Fn(&str) -> Option<T>,
    to_code: impl Fn(T) -> u64,
) -> bool {
    match (code, label) {
        (Some(code), Some(label)) => from_label(label).is_some_and(|value| to_code(value) == code),
        (None, None) => true,
        _ => false,
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "snake_case")]
enum UsageEventKind {
    #[default]
    Typed,
    AcceptedFix,
    AcceptedIme,
    EditedIme,
    ConfirmedImePrediction,
    RejectedIme,
    RejectedCandidate,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
struct UsageCounts {
    words: HashMap<String, u32>,
    #[serde(default)]
    accepted_words: HashMap<String, u32>,
    context_words: HashMap<String, u32>,
    #[serde(default)]
    rejected_words: HashMap<String, u32>,
    #[serde(default)]
    rejected_context_words: HashMap<String, u32>,
    #[serde(default)]
    transition_observed: HashMap<String, u32>,
    #[serde(default)]
    transition_attract: HashMap<String, u32>,
    #[serde(default)]
    transition_repel: HashMap<String, u32>,
    #[serde(default)]
    surface_observed: HashMap<String, u32>,
    #[serde(default)]
    surface_attract: HashMap<String, u32>,
    #[serde(default)]
    surface_repel: HashMap<String, u32>,
}

#[derive(Debug, Clone, Default)]
pub struct UsagePriorSnapshot {
    hot: Arc<UsageHotState>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct UsageStateMapSummary {
    pub(crate) source_bytes: u64,
    pub(crate) parsed_events: usize,
    pub(crate) hot_logical_payload_bytes: usize,
    pub(crate) cold_dictionary_logical_bytes: usize,
    pub(crate) word_states: usize,
    pub(crate) accepted_word_states: usize,
    pub(crate) context_word_states: usize,
    pub(crate) rejected_word_states: usize,
    pub(crate) rejected_context_word_states: usize,
    pub(crate) signed_word_states: usize,
    pub(crate) transition_states: usize,
    pub(crate) transition_observed_states: usize,
    pub(crate) transition_attract_states: usize,
    pub(crate) transition_repel_states: usize,
    pub(crate) transition_signed_states: usize,
    pub(crate) transition_conflict_states: usize,
    pub(crate) surface_states: usize,
    pub(crate) surface_observed_states: usize,
    pub(crate) surface_covered_states: usize,
    pub(crate) surface_repelled_states: usize,
    pub(crate) surface_conflict_states: usize,
}

impl UsagePriorSnapshot {
    pub(crate) fn surface_coverage(&self, surface: &str) -> UsageSurfaceCoverage {
        self.hot.surface_coverage(surface)
    }

    pub(crate) fn phase_witness(
        &self,
        surface: &str,
    ) -> super::l4_phase_witness::L4PhaseWitnessReadout {
        self.hot.phase_witness(surface)
    }

    #[cfg(test)]
    pub(crate) fn hot_logical_payload_bytes(&self) -> usize {
        self.hot.logical_payload_bytes()
    }

    pub fn word_prior(&self, word: &str) -> f32 {
        self.hot.word_prior(word)
    }

    pub fn context_word_prior(&self, context: &[String], word: &str) -> f32 {
        self.hot.context_word_prior(context, word)
    }

    pub fn accepted_word_count(&self, word: &str) -> u32 {
        self.hot.accepted_word_count(word)
    }

    pub(crate) fn rejected_word_prior(&self, word: &str) -> f32 {
        self.hot.rejected_word_prior(word)
    }

    pub(crate) fn context_rejected_word_prior(&self, context: &[String], word: &str) -> f32 {
        self.hot.context_rejected_word_prior(context, word)
    }

    pub(crate) fn hot_readout(
        &self,
        context: &[String],
        source: &str,
        operation: &str,
        state_word: &str,
        candidate_text: &str,
    ) -> UsageHotReadout {
        let prepared = self.prepare_hot_context(context);
        self.hot_readout_prepared(&prepared, source, operation, state_word, candidate_text)
    }

    pub(crate) fn prepare_hot_context(&self, context: &[String]) -> UsageHotContext {
        UsageHotContext::from_words(context)
    }

    pub(crate) fn candidate_prior_prepared(
        &self,
        context: &UsageHotContext,
        normalized_word: &str,
    ) -> UsageCandidatePrior {
        if normalized_word.is_empty() {
            return UsageCandidatePrior::default();
        }
        self.hot.candidate_prior_prepared(context, normalized_word)
    }

    pub(crate) fn context_prefix_candidates(
        &self,
        context: &UsageHotContext,
        partial: &str,
        limit: usize,
    ) -> Vec<UsageContextCandidate> {
        self.hot.context_prefix_candidates(context, partial, limit)
    }

    pub(crate) fn context_candidates(
        &self,
        context: &UsageHotContext,
        limit: usize,
    ) -> Vec<UsageContextCandidate> {
        self.hot.context_candidates(context, limit)
    }

    pub(crate) fn hot_readout_prepared(
        &self,
        context: &UsageHotContext,
        source: &str,
        operation: &str,
        state_word: &str,
        candidate_text: &str,
    ) -> UsageHotReadout {
        self.hot
            .hot_readout_prepared(context, source, operation, state_word, candidate_text)
    }
}

#[derive(Debug, Clone, Default, serde::Deserialize, serde::Serialize)]
struct PersistedUsageCounts {
    #[serde(default)]
    schema_version: u32,
    source_len: u64,
    counts: UsageCounts,
}

#[derive(Debug, Default)]
struct UsageCache {
    loaded_at: Option<Instant>,
    hot: Arc<UsageHotState>,
}

#[cfg(not(test))]
struct UsagePersistLine {
    path: PathBuf,
    line: String,
}

#[cfg(not(test))]
static USAGE_PERSIST_SENDER: OnceLock<SyncSender<UsagePersistLine>> = OnceLock::new();
static LAST_USAGE_EVENT: OnceLock<Mutex<Option<UsageEvent>>> = OnceLock::new();

pub(crate) fn record_typed_tail_if_enabled(tail: &str) {
    if !usage_learning_enabled() {
        return;
    }
    let Some(event) = TypingMemoryEvent::typed_tail(tail) else {
        return;
    };
    record_typing_memory_episode_if_enabled(std::slice::from_ref(&event));
}

pub(crate) fn record_accepted_fix_if_enabled(from: &str, to: &str) {
    if !usage_learning_enabled() || from == to {
        return;
    }
    let events = TypingMemoryEvent::accepted_fix(from, to);
    record_typing_memory_episode_if_enabled(&events);
    super::llmwave::record_phrase_experience("space", to);
}

pub(crate) fn record_confirmed_user_correction_if_enabled(
    original: &str,
    proposal: &str,
    accepted: &str,
    operation: &str,
) {
    if !usage_learning_enabled() || (original == accepted && proposal == accepted) {
        return;
    }
    let events =
        TypingMemoryEvent::confirmed_user_correction(original, proposal, accepted, operation);
    record_typing_memory_episode_if_enabled(&events);
    super::llmwave::record_phrase_experience("space", accepted);
}

pub(crate) fn record_observed_system_apply_if_enabled(
    from: &str,
    to: &str,
    source: TypingMemoryEvidenceSource,
    operation: TypingMemoryOperation,
) {
    if !usage_learning_enabled() || from == to {
        return;
    }
    let events = TypingMemoryEvent::observed_system_apply(from, to, source, operation);
    record_typing_memory_episode_if_enabled(&events);
}

pub(crate) fn record_reverted_system_apply_if_enabled(
    original: &str,
    rejected: &str,
    source: TypingMemoryEvidenceSource,
    operation: TypingMemoryOperation,
) {
    if !usage_learning_enabled() || original == rejected {
        return;
    }
    let events = TypingMemoryEvent::reverted_system_apply(original, rejected, source, operation);
    record_typing_memory_episode_if_enabled(&events);
}

pub(crate) fn record_accepted_layout_projection_if_enabled(from: &str, to: &str) {
    if !usage_learning_enabled() || from == to {
        return;
    }
    let events = TypingMemoryEvent::accepted_layout_projection(from, to);
    record_typing_memory_episode_if_enabled(&events);
}

pub(crate) fn record_accepted_ime_if_enabled(context_tail: &str, accepted_text: &str) {
    if !usage_learning_enabled() {
        return;
    }
    let events = TypingMemoryEvent::accepted_ime(context_tail, accepted_text);
    record_typing_memory_episode_if_enabled(&events);
    let phrase = [context_tail.trim(), accepted_text.trim()]
        .into_iter()
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join(" ");
    super::llmwave::record_phrase_experience("space", &phrase);
}

pub(crate) fn record_edited_ime_if_enabled(
    context_tail: &str,
    typed_prefix: &str,
    suggested_text: &str,
    final_text: &str,
    shared_morphology_identity: bool,
) {
    if !usage_learning_enabled() {
        return;
    }
    let Some(event) = TypingMemoryEvent::edited_ime_with_shared_identity(
        context_tail,
        typed_prefix,
        suggested_text,
        final_text,
        shared_morphology_identity,
    ) else {
        return;
    };
    record_typing_memory_episode_if_enabled(std::slice::from_ref(&event));
}

pub(crate) fn record_confirmed_ime_prediction_if_enabled(context_tail: &str, predicted_text: &str) {
    if !usage_learning_enabled() {
        return;
    }
    let events = TypingMemoryEvent::confirmed_ime_prediction(context_tail, predicted_text);
    record_typing_memory_episode_if_enabled(&events);
}

pub(crate) fn record_rejected_ime_if_enabled(context_tail: &str, rejected_text: &str) {
    if !usage_learning_enabled() {
        return;
    }
    let events = TypingMemoryEvent::rejected_ime(context_tail, rejected_text);
    record_typing_memory_episode_if_enabled(&events);
}

pub(crate) fn record_rejected_candidate_if_enabled(
    context_tail: &str,
    rejected_text: &str,
    source: &str,
    operation: &str,
) {
    if !usage_learning_enabled() {
        return;
    }
    let events =
        TypingMemoryEvent::rejected_candidate(context_tail, rejected_text, source, operation);
    record_typing_memory_episode_if_enabled(&events);
}

fn record_typing_memory_episode_if_enabled(events: &[TypingMemoryEvent]) {
    if !usage_learning_enabled() || events.is_empty() {
        return;
    }
    let usage_events = events
        .iter()
        .map(UsageEvent::from_typing_memory_event)
        .collect::<Vec<_>>();
    for event in &usage_events {
        append_usage_event(event.clone());
    }
    let Some(episode_id) = usage_events
        .first()
        .and_then(|event| event.episode_id.as_deref())
    else {
        return;
    };
    if usage_events.iter().any(|event| {
        event.schema != Some(TYPED_EVENT_SCHEMA_V3)
            || event.episode_id.as_deref() != Some(episode_id)
    }) {
        return;
    }
    let rows = usage_events
        .iter()
        .map(serde_json::to_value)
        .collect::<Result<Vec<_>, _>>();
    if let Ok(rows) = rows {
        let _ = super::l4_cross_scene::enqueue_episode(rows);
    }
}

pub(crate) fn word_usage_prior(word: &str) -> f32 {
    ingest_usage_hot_state_if_stale().word_prior(word)
}

pub(crate) fn context_word_usage_prior(context: &[String], word: &str) -> f32 {
    ingest_usage_hot_state_if_stale().context_word_prior(context, word)
}

fn usage_learning_enabled() -> bool {
    crate::config::runtime_usage_learning_enabled()
}

fn ingest_usage_hot_state_if_stale() -> Arc<UsageHotState> {
    let Ok(mut cache) = usage_cache().lock() else {
        return Arc::new(UsageHotState::default());
    };
    if cache
        .loaded_at
        .is_some_and(|loaded_at| loaded_at.elapsed() < USAGE_REFRESH_INTERVAL)
    {
        return Arc::clone(&cache.hot);
    }
    set_usage_cache_hot_from_counts(&mut cache, load_usage_counts());
    cache.loaded_at = Some(Instant::now());
    Arc::clone(&cache.hot)
}

pub(crate) fn word_usage_prior_cached(word: &str) -> f32 {
    cached_usage_hot_state().word_prior(word)
}

pub(crate) fn accepted_word_usage_count_cached(word: &str) -> u32 {
    cached_usage_hot_state().accepted_word_count(word)
}

pub(crate) fn context_word_usage_prior_cached(context: &[String], word: &str) -> f32 {
    cached_usage_hot_state().context_word_prior(context, word)
}

pub(crate) fn cached_usage_prior_snapshot() -> UsagePriorSnapshot {
    UsagePriorSnapshot {
        hot: cached_usage_hot_state(),
    }
}

#[cfg(test)]
pub(crate) fn snapshot_from_usage_events_for_tests(text: &str) -> UsagePriorSnapshot {
    let mut counts = UsageCounts::default();
    add_usage_event_counts(&mut counts, text);
    usage_snapshot_from_counts(counts)
}

pub(crate) fn l2_surface_words_by_usage(limit: usize) -> Vec<String> {
    if limit == 0 {
        return Vec::new();
    }
    let counts = refresh_usage_counts_from_disk();
    let mut words = usage_surface_words_from_counts(counts);
    words.truncate(limit);
    words
}

pub(crate) fn usage_debug_summary() -> (u64, usize, usize) {
    let summary = usage_state_map_summary();
    (
        summary.source_bytes,
        summary.parsed_events,
        summary.word_states,
    )
}

pub(crate) fn usage_state_map_summary() -> UsageStateMapSummary {
    let text = usage_events_path()
        .and_then(|path| read_usage_events_text(&path))
        .unwrap_or_default();
    let counts = load_usage_counts();
    let hot = UsageHotState::from_counts(&counts);
    UsageStateMapSummary {
        source_bytes: text.len() as u64,
        parsed_events: usage_events_from_jsonl(&text).count(),
        hot_logical_payload_bytes: hot.logical_payload_bytes(),
        cold_dictionary_logical_bytes: usage_counts_cold_dictionary_logical_bytes(&counts),
        word_states: counts.words.len(),
        accepted_word_states: counts.accepted_words.len(),
        context_word_states: counts.context_words.len(),
        rejected_word_states: counts.rejected_words.len(),
        rejected_context_word_states: counts.rejected_context_words.len(),
        signed_word_states: counts
            .accepted_words
            .keys()
            .chain(counts.rejected_words.keys())
            .collect::<HashSet<_>>()
            .len(),
        transition_states: counts
            .transition_observed
            .keys()
            .chain(counts.transition_attract.keys())
            .chain(counts.transition_repel.keys())
            .collect::<HashSet<_>>()
            .len(),
        transition_observed_states: counts.transition_observed.len(),
        transition_attract_states: counts.transition_attract.len(),
        transition_repel_states: counts.transition_repel.len(),
        transition_signed_states: counts
            .transition_attract
            .keys()
            .chain(counts.transition_repel.keys())
            .collect::<HashSet<_>>()
            .len(),
        transition_conflict_states: counts
            .transition_attract
            .keys()
            .filter(|key| counts.transition_repel.contains_key(*key))
            .count(),
        surface_states: counts
            .surface_observed
            .keys()
            .chain(counts.surface_attract.keys())
            .chain(counts.surface_repel.keys())
            .collect::<HashSet<_>>()
            .len(),
        surface_observed_states: counts.surface_observed.len(),
        surface_covered_states: counts.surface_attract.len(),
        surface_repelled_states: counts.surface_repel.len(),
        surface_conflict_states: counts
            .surface_attract
            .keys()
            .filter(|key| counts.surface_repel.contains_key(*key))
            .count(),
    }
}

pub fn usage_memory_learned_report_json() -> serde_json::Value {
    let text = usage_events_path()
        .and_then(|path| read_usage_events_text(&path))
        .unwrap_or_default();
    let counts = load_usage_counts();
    let summary = usage_state_map_summary();
    let causal = causal_feedback_summary(&text);
    serde_json::json!({
        "kind": "typing_memory_learned_report",
        "status": "ok",
        "source": "word_usage_events.jsonl + word_usage_counts.json",
        "summary": {
            "source_bytes": summary.source_bytes,
            "parsed_events": summary.parsed_events,
            "hot_logical_payload_bytes": summary.hot_logical_payload_bytes,
            "cold_dictionary_logical_bytes": summary.cold_dictionary_logical_bytes,
            "word_states": summary.word_states,
            "accepted_word_states": summary.accepted_word_states,
            "context_word_states": summary.context_word_states,
            "rejected_word_states": summary.rejected_word_states,
            "rejected_context_word_states": summary.rejected_context_word_states,
            "signed_word_states": summary.signed_word_states,
            "transition_states": summary.transition_states,
            "transition_observed_states": summary.transition_observed_states,
            "transition_attract_states": summary.transition_attract_states,
            "transition_repel_states": summary.transition_repel_states,
            "transition_signed_states": summary.transition_signed_states,
            "transition_conflict_states": summary.transition_conflict_states,
            "surface_states": summary.surface_states,
            "surface_observed_states": summary.surface_observed_states,
            "surface_covered_states": summary.surface_covered_states,
            "surface_repelled_states": summary.surface_repelled_states,
            "surface_conflict_states": summary.surface_conflict_states
        },
        "learned_top": {
            "accepted_words": top_count_json(&counts.accepted_words, 12),
            "rejected_words": top_count_json(&counts.rejected_words, 12),
            "context_words": top_count_json(&counts.context_words, 12),
            "transition_attract": top_count_json(&counts.transition_attract, 12),
            "transition_repel": top_count_json(&counts.transition_repel, 12),
            "surface_covered": top_count_json(&counts.surface_attract, 12),
            "surface_repelled": top_count_json(&counts.surface_repel, 12)
        },
        "hot_readout": {
            "mode": "UsagePriorSnapshot::hot_readout",
            "single_pass": true,
            "uses": ["word_prior", "context_prior", "rejected_prior", "context_rejected", "accepted_count", "rejected_count", "transition_signal", "surface_frontier"]
        },
        "causal_feedback": causal,
        "events_tail_bytes": text.len(),
        "authority": "ranking signal only; edit safety gate remains final"
    })
}

/// Read-only migration proof for the typed L4 event envelope. It enriches an
/// in-memory copy of the journal and compares the complete signed usage state;
/// the source file is never rewritten.
pub fn usage_memory_typed_replay_report_json(path: Option<&Path>) -> serde_json::Value {
    let path = path
        .map(Path::to_path_buf)
        .or_else(usage_events_path)
        .unwrap_or_default();
    let text = read_usage_events_text(&path).unwrap_or_default();
    let events = usage_events_from_jsonl(&text).collect::<Vec<_>>();
    let mut baseline = UsageCounts::default();
    for event in &events {
        add_usage_event_count(&mut baseline, event);
    }

    let mut migrated = events.clone();
    for event in &mut migrated {
        event.enrich_typed_v2();
    }
    let invalid_typed_rows = migrated
        .iter()
        .filter(|event| !event.typed_v2_is_consistent())
        .count();
    let mut replay = UsageCounts::default();
    for event in &migrated {
        add_usage_event_count(&mut replay, event);
    }
    let replay_parity = baseline == replay;
    serde_json::json!({
        "kind": "l4_typed_event_replay",
        "source": path,
        "source_bytes": text.len(),
        "rows": events.len(),
        "typed_schema": TYPED_EVENT_SCHEMA_V2,
        "typed_rows": migrated.len(),
        "invalid_typed_rows": invalid_typed_rows,
        "word_states": baseline.words.len(),
        "transition_states": baseline.transition_observed.len(),
        "signed_transition_states": baseline
            .transition_attract
            .keys()
            .chain(baseline.transition_repel.keys())
            .collect::<HashSet<_>>()
            .len(),
        "replay_parity": replay_parity,
        "false_apply_behavior_changed": false,
        "source_rewritten": false,
        "verdict": if replay_parity && invalid_typed_rows == 0 { "PASS" } else { "FAIL" },
    })
}

/// Diagnostics only: this reads journal metadata and never changes a score.
/// It tells the tray/CLI which outcomes L4 has actually observed, including
/// censored receipts that must not become negative semantic evidence.
fn causal_feedback_summary(text: &str) -> serde_json::Value {
    let mut outcomes = HashMap::new();
    let mut operators = HashMap::new();
    let mut layout_directions = HashMap::new();
    let mut layout_scopes = HashMap::new();
    for event in usage_events_from_jsonl(text) {
        increment_optional_count(&mut outcomes, event.outcome.as_deref());
        increment_optional_count(&mut operators, event.operator.as_deref());
        increment_optional_count(&mut layout_directions, event.layout_direction.as_deref());
        increment_optional_count(&mut layout_scopes, event.layout_scope.as_deref());
    }
    serde_json::json!({
        "outcomes": top_count_json(&outcomes, 12),
        "operators": top_count_json(&operators, 12),
        "layout_directions": top_count_json(&layout_directions, 8),
        "layout_scopes": top_count_json(&layout_scopes, 8),
    })
}

fn increment_optional_count(target: &mut HashMap<String, u32>, value: Option<&str>) {
    let Some(value) = value.filter(|value| !value.is_empty()) else {
        return;
    };
    *target.entry(value.to_string()).or_default() += 1;
}

fn refresh_usage_counts_from_disk() -> UsageCounts {
    let counts = load_usage_counts();
    if let Ok(mut cache) = usage_cache().lock() {
        cache.hot = Arc::new(UsageHotState::from_counts(&counts));
        cache.loaded_at = Some(Instant::now());
    }
    counts
}

fn usage_surface_words_from_counts(counts: UsageCounts) -> Vec<String> {
    let word_counts = counts.words;
    let mut words = counts
        .accepted_words
        .into_iter()
        .filter(|(word, count)| {
            *count >= 1
                && (2..=32).contains(&word.chars().count())
                && word.chars().all(|ch| ch.is_alphabetic() || ch == '-')
        })
        .map(|(word, accepted_count)| {
            let typed_count = word_counts.get(&word).copied().unwrap_or_default();
            let score = accepted_count
                .saturating_mul(4)
                .saturating_add(typed_count.min(50));
            (word, score)
        })
        .collect::<Vec<_>>();
    words.sort_by(|(left_word, left_count), (right_word, right_count)| {
        right_count
            .cmp(left_count)
            .then_with(|| left_word.chars().count().cmp(&right_word.chars().count()))
            .then_with(|| left_word.cmp(right_word))
    });
    words.into_iter().map(|(word, _)| word).collect()
}

#[cfg(test)]
fn context_ngram_prior_from_counts(counts: &UsageCounts, context: &[String], word: &str) -> f32 {
    let context_keys = context_ngram_keys(context);
    context_ngram_prior_from_keys(&counts.context_words, &context_keys, word, 0.020)
}

#[cfg(test)]
fn context_ngram_prior_from_keys(
    source: &HashMap<String, u32>,
    context_keys: &[String],
    word: &str,
    base_weight: f32,
) -> f32 {
    context_keys
        .iter()
        .filter_map(|context_key| {
            let ngram_len = context_key.split_whitespace().count();
            let key = context_word_key(context_key, word);
            source.get(&key).copied().map(|count| (count, ngram_len))
        })
        .map(|(count, ngram_len)| {
            let ngram_weight = base_weight + ngram_len as f32 * 0.010;
            ((count as f32 + 1.0).ln() * ngram_weight).min(0.18)
        })
        .sum::<f32>()
        .clamp(0.0, 0.34)
}

fn cached_usage_hot_state() -> Arc<UsageHotState> {
    let Ok(mut cache) = usage_cache().lock() else {
        return Arc::new(UsageHotState::default());
    };
    // The hot readout is allowed to be the first usage-memory consumer.
    // Returning the default Arc here made L4 depend on an unrelated warmup
    // route and left first-word decisions blind until another action loaded
    // the persisted state.
    ensure_usage_cache_initialized(&mut cache, load_usage_counts);
    Arc::clone(&cache.hot)
}

fn ensure_usage_cache_initialized(cache: &mut UsageCache, load: impl FnOnce() -> UsageCounts) {
    if cache.loaded_at.is_some() {
        return;
    }
    set_usage_cache_hot_from_counts(cache, load());
    cache.loaded_at = Some(Instant::now());
}

fn set_usage_cache_hot_from_counts(cache: &mut UsageCache, counts: UsageCounts) {
    cache.hot = Arc::new(UsageHotState::from_counts(&counts));
}

#[cfg(test)]
fn usage_snapshot_from_counts(counts: UsageCounts) -> UsagePriorSnapshot {
    UsagePriorSnapshot {
        hot: Arc::new(UsageHotState::from_counts(&counts)),
    }
}

fn usage_cache() -> &'static Mutex<UsageCache> {
    static CACHE: OnceLock<Mutex<UsageCache>> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(UsageCache::default()))
}

fn load_usage_counts() -> UsageCounts {
    let mut counts = load_usage_feedback_counts();
    if let Some(text) =
        legacy_usage_prior_path().and_then(|path| std::fs::read_to_string(path).ok())
    {
        add_legacy_usage_counts(&mut counts, &text);
    }
    merge_usage_counts(&mut counts, load_usage_event_counts());
    counts
}

fn load_usage_feedback_counts() -> UsageCounts {
    let Some(path) = usage_feedback_counts_path() else {
        return UsageCounts::default();
    };
    load_persisted_usage_counts(&path, None).unwrap_or_default()
}

fn load_usage_event_counts() -> UsageCounts {
    let Some(path) = usage_events_path() else {
        return UsageCounts::default();
    };
    let source_len = std::fs::metadata(&path)
        .map(|meta| meta.len())
        .unwrap_or_default();
    if let Some(snapshot) = load_usage_counts_snapshot(source_len) {
        return snapshot;
    }

    let text = if source_len <= USAGE_EVENTS_FULL_REBUILD_MAX_BYTES {
        read_full_text_lossy(&path)
    } else {
        read_usage_events_text(&path)
    };
    let mut counts = UsageCounts::default();
    if let Some(text) = text {
        add_usage_event_counts(&mut counts, &text);
    }
    persist_usage_counts_snapshot(&counts, source_len);
    counts
}

fn merge_usage_counts(target: &mut UsageCounts, source: UsageCounts) {
    for (word, count) in source.words {
        *target.words.entry(word).or_default() = target
            .words
            .get(&word)
            .copied()
            .unwrap_or_default()
            .saturating_add(count);
    }
    for (word, count) in source.accepted_words {
        *target.accepted_words.entry(word).or_default() = target
            .accepted_words
            .get(&word)
            .copied()
            .unwrap_or_default()
            .saturating_add(count);
    }
    for (key, count) in source.context_words {
        *target.context_words.entry(key).or_default() = target
            .context_words
            .get(&key)
            .copied()
            .unwrap_or_default()
            .saturating_add(count);
    }
    for (word, count) in source.rejected_words {
        *target.rejected_words.entry(word.clone()).or_default() = target
            .rejected_words
            .get(&word)
            .copied()
            .unwrap_or_default()
            .saturating_add(count);
    }
    for (key, count) in source.rejected_context_words {
        *target
            .rejected_context_words
            .entry(key.clone())
            .or_default() = target
            .rejected_context_words
            .get(&key)
            .copied()
            .unwrap_or_default()
            .saturating_add(count);
    }
    merge_count_map(&mut target.transition_observed, source.transition_observed);
    merge_count_map(&mut target.transition_attract, source.transition_attract);
    merge_count_map(&mut target.transition_repel, source.transition_repel);
    merge_count_map(&mut target.surface_observed, source.surface_observed);
    merge_count_map(&mut target.surface_attract, source.surface_attract);
    merge_count_map(&mut target.surface_repel, source.surface_repel);
}

fn merge_count_map(target: &mut HashMap<String, u32>, source: HashMap<String, u32>) {
    for (key, count) in source {
        *target.entry(key.clone()).or_default() = target
            .get(&key)
            .copied()
            .unwrap_or_default()
            .saturating_add(count);
    }
}

fn add_legacy_usage_counts(counts: &mut UsageCounts, text: &str) {
    for (word, count) in legacy_usage_counts_from_json(text) {
        *counts.words.entry(word).or_default() = counts
            .words
            .get(&word)
            .copied()
            .unwrap_or_default()
            .saturating_add(count);
    }
}

fn add_usage_event_counts(counts: &mut UsageCounts, text: &str) {
    let mut seen = HashSet::new();
    for event in usage_events_from_jsonl(text) {
        let Ok(key) = serde_json::to_string(&event) else {
            continue;
        };
        if !seen.insert(key) {
            continue;
        }
        add_usage_event_count(counts, &event);
    }
}

fn add_usage_event_count(counts: &mut UsageCounts, event: &UsageEvent) {
    let Some(projected) = UsageEventProjection::from_event(event) else {
        return;
    };
    if projected.is_rejected() {
        if let Some(surface) = projected.surface {
            *counts
                .surface_observed
                .entry(surface.to_string())
                .or_default() += projected.weight;
            *counts.surface_repel.entry(surface.to_string()).or_default() += projected.weight;
        }
        add_rejected_word_state(
            counts,
            RejectedStateEvidence {
                context: projected.context,
                source: projected.source,
                operation: projected.operation,
                state_word: &projected.state_word,
                rejected: &projected.word,
                transition_context: &projected.transition_context,
                transition_target: &projected.transition_target,
                weight: projected.weight,
                transition_weight: projected.transition_weight,
                record_transition: true,
            },
        );
        if matches!(event.kind, UsageEventKind::RejectedCandidate)
            && projected.state_word != TRANSITION_ANY
        {
            add_state_authority_repel(
                &mut counts.transition_repel,
                &projected.transition_context,
                projected.source,
                projected.operation,
                &projected.state_word,
                projected.transition_weight,
            );
        }
        return;
    }
    if let Some(surface) = projected.surface {
        *counts
            .surface_observed
            .entry(surface.to_string())
            .or_default() += projected.weight;
        if projected.is_accepted() {
            *counts
                .surface_attract
                .entry(surface.to_string())
                .or_default() += projected.weight;
        }
    }
    *counts.words.entry(projected.word.clone()).or_default() = counts
        .words
        .get(&projected.word)
        .copied()
        .unwrap_or_default()
        .saturating_add(projected.weight);
    if projected.is_accepted() {
        *counts
            .accepted_words
            .entry(projected.word.clone())
            .or_default() = counts
            .accepted_words
            .get(&projected.word)
            .copied()
            .unwrap_or_default()
            .saturating_add(projected.weight);
    }

    for context_key in context_ngram_keys(projected.context) {
        let key = context_word_key(&context_key, &projected.word);
        *counts.context_words.entry(key.clone()).or_default() = counts
            .context_words
            .get(&key)
            .copied()
            .unwrap_or_default()
            .saturating_add(projected.weight);
    }
    add_transition_counts(
        &mut counts.transition_observed,
        &projected.transition_context,
        projected.source,
        projected.operation,
        &projected.state_word,
        &projected.transition_target,
        projected.transition_weight,
    );
    if projected.is_accepted() {
        add_transition_counts(
            &mut counts.transition_attract,
            &projected.transition_context,
            projected.source,
            projected.operation,
            &projected.state_word,
            &projected.transition_target,
            projected.transition_weight,
        );
    }

    if projected.records_rejected_fix_sources() {
        add_rejected_fix_sources(
            counts,
            event,
            projected.weight,
            projected.source,
            projected.operation,
        );
    }
}

fn usage_events_from_jsonl(text: &str) -> impl Iterator<Item = UsageEvent> + '_ {
    text.lines()
        .filter(|line| !line.trim().is_empty())
        .filter_map(|line| serde_json::from_str::<UsageEvent>(line).ok())
}

fn legacy_usage_counts_from_json(text: &str) -> HashMap<String, u32> {
    let Ok(records) = serde_json::from_str::<HashMap<String, LearningCandidate>>(text) else {
        return HashMap::new();
    };
    let mut counts = HashMap::<String, u32>::new();
    for record in records.into_values() {
        let token = normalized_words(&record.to)
            .into_iter()
            .next_back()
            .unwrap_or_default();
        if token.is_empty() {
            continue;
        }
        let weight = record
            .count
            .saturating_add(if record.promoted { 3 } else { 0 });
        *counts.entry(token.clone()).or_default() = counts
            .get(&token)
            .copied()
            .unwrap_or_default()
            .saturating_add(weight);
    }
    counts
}

fn add_rejected_fix_sources(
    counts: &mut UsageCounts,
    event: &UsageEvent,
    weight: u32,
    source: &str,
    operation: &str,
) {
    let Some(from) = event.proposal.as_deref().or(event.from.as_deref()) else {
        return;
    };
    let accepted = event
        .to
        .as_deref()
        .map(normalized_words)
        .unwrap_or_default()
        .into_iter()
        .collect::<HashSet<_>>();
    for rejected in normalized_words(from)
        .into_iter()
        .filter(|word| !accepted.contains(word))
    {
        add_rejected_word_state(
            counts,
            RejectedStateEvidence {
                context: &event.context,
                source,
                operation,
                state_word: &rejected,
                rejected: &rejected,
                transition_context: &event.context,
                transition_target: &rejected,
                weight,
                transition_weight: weight,
                record_transition: false,
            },
        );
    }
}

struct RejectedStateEvidence<'a> {
    context: &'a [String],
    source: &'a str,
    operation: &'a str,
    state_word: &'a str,
    rejected: &'a str,
    transition_context: &'a [String],
    transition_target: &'a str,
    weight: u32,
    transition_weight: u32,
    record_transition: bool,
}

fn add_rejected_word_state(counts: &mut UsageCounts, evidence: RejectedStateEvidence<'_>) {
    let RejectedStateEvidence {
        context,
        source,
        operation,
        state_word,
        rejected,
        transition_context,
        transition_target,
        weight,
        transition_weight,
        record_transition,
    } = evidence;
    *counts
        .rejected_words
        .entry(rejected.to_string())
        .or_default() = counts
        .rejected_words
        .get(rejected)
        .copied()
        .unwrap_or_default()
        .saturating_add(weight);
    for context_key in context_ngram_keys(context) {
        let key = context_word_key(&context_key, rejected);
        *counts
            .rejected_context_words
            .entry(key.clone())
            .or_default() = counts
            .rejected_context_words
            .get(&key)
            .copied()
            .unwrap_or_default()
            .saturating_add(weight);
    }
    if record_transition {
        add_transition_counts(
            &mut counts.transition_repel,
            transition_context,
            source,
            operation,
            state_word,
            transition_target,
            transition_weight,
        );
    }
}

fn append_usage_event(event: UsageEvent) {
    let Some(path) = usage_events_path() else {
        return;
    };
    let _ = cached_usage_hot_state();
    if adjacent_usage_event_is_duplicate(&path, &event) {
        return;
    }
    let Ok(mut line) = serde_json::to_string(&event) else {
        return;
    };
    line.push('\n');
    refresh_usage_cache_after_write(&event);
    enqueue_usage_persist(path, line);
}

fn refresh_usage_cache_after_write(event: &UsageEvent) {
    let Ok(mut cache) = usage_cache().lock() else {
        return;
    };
    apply_usage_event_to_cache(&mut cache, event, load_usage_counts);
}

fn apply_usage_event_to_cache(
    cache: &mut UsageCache,
    event: &UsageEvent,
    load: impl FnOnce() -> UsageCounts,
) {
    ensure_usage_cache_initialized(cache, load);
    Arc::make_mut(&mut cache.hot).apply_event(event);
    cache.loaded_at = Some(Instant::now());
}

fn adjacent_usage_event_is_duplicate(path: &Path, event: &UsageEvent) -> bool {
    let last = LAST_USAGE_EVENT.get_or_init(|| Mutex::new(read_last_usage_event(path)));
    let Ok(mut last) = last.lock() else {
        return false;
    };
    if last
        .as_ref()
        .is_some_and(|previous| usage_event_payload_eq(previous, event))
    {
        return true;
    }
    *last = Some(event.clone());
    false
}

fn read_last_usage_event(path: &Path) -> Option<UsageEvent> {
    let text = read_usage_events_text(path)?;
    let line = text.lines().rev().find(|line| !line.trim().is_empty())?;
    serde_json::from_str(line).ok()
}

fn usage_event_payload_eq(left: &UsageEvent, right: &UsageEvent) -> bool {
    left.schema == right.schema
        && left.episode_id == right.episode_id
        && left.kind == right.kind
        && left.word == right.word
        && left.context == right.context
        && left.from == right.from
        && left.to == right.to
        && left.source == right.source
        && left.operation == right.operation
        && left.surface == right.surface
        && left.operator == right.operator
        && left.layout_direction == right.layout_direction
        && left.layout_scope == right.layout_scope
        && left.source_language == right.source_language
        && left.target_language == right.target_language
        && left.source_layout == right.source_layout
        && left.target_layout == right.target_layout
        && left.source_script == right.source_script
        && left.target_script == right.target_script
        && left.keyboard_geometry == right.keyboard_geometry
        && left.identity_evidence == right.identity_evidence
        && left.sentence_language == right.sentence_language
        && left.outcome == right.outcome
        && left.evidence_source_code == right.evidence_source_code
        && left.operation_code == right.operation_code
        && left.operator_code == right.operator_code
        && left.layout_direction_code == right.layout_direction_code
        && left.layout_scope_code == right.layout_scope_code
        && left.source_language_id == right.source_language_id
        && left.target_language_id == right.target_language_id
        && left.source_layout_id == right.source_layout_id
        && left.target_layout_id == right.target_layout_id
        && left.source_script_code == right.source_script_code
        && left.target_script_code == right.target_script_code
        && left.keyboard_geometry_id == right.keyboard_geometry_id
        && left.identity_evidence_code == right.identity_evidence_code
        && left.sentence_language_id == right.sentence_language_id
        && left.sentence_language_support_milli == right.sentence_language_support_milli
        && left.sentence_language_alternative_milli == right.sentence_language_alternative_milli
        && left.sentence_language_observed_tokens == right.sentence_language_observed_tokens
        && left.outcome_code == right.outcome_code
        && left.completion_edit == right.completion_edit
        && left.proposal == right.proposal
}

#[cfg(not(test))]
fn enqueue_usage_persist(path: PathBuf, line: String) {
    let sender = USAGE_PERSIST_SENDER.get_or_init(spawn_usage_persist_writer);
    match sender.try_send(UsagePersistLine { path, line }) {
        Ok(()) | Err(TrySendError::Full(_)) | Err(TrySendError::Disconnected(_)) => {}
    }
}

#[cfg(test)]
fn enqueue_usage_persist(path: PathBuf, line: String) {
    if crate::private_file::append_private_text(&path, &line).is_ok() {
        compact_usage_events_if_needed(&path);
    }
}

#[cfg(not(test))]
fn spawn_usage_persist_writer() -> SyncSender<UsagePersistLine> {
    let (sender, receiver) = mpsc::sync_channel::<UsagePersistLine>(USAGE_PERSIST_CHANNEL_CAPACITY);
    std::thread::Builder::new()
        .name("lay-usage-persist".to_string())
        .spawn(move || {
            let mut pending = HashMap::<PathBuf, String>::new();
            let mut pending_bytes = 0usize;
            let mut next_flush = Instant::now() + USAGE_PERSIST_INTERVAL;
            loop {
                let timeout = next_flush.saturating_duration_since(Instant::now());
                match receiver.recv_timeout(timeout) {
                    Ok(record) => {
                        pending_bytes = pending_bytes.saturating_add(record.line.len());
                        pending
                            .entry(record.path)
                            .or_default()
                            .push_str(&record.line);
                        if pending_bytes >= USAGE_PERSIST_PENDING_MAX_BYTES {
                            flush_usage_persist(&mut pending);
                            pending_bytes = 0;
                            next_flush = Instant::now() + USAGE_PERSIST_INTERVAL;
                        }
                    }
                    Err(RecvTimeoutError::Timeout) => {
                        flush_usage_persist(&mut pending);
                        pending_bytes = 0;
                        next_flush = Instant::now() + USAGE_PERSIST_INTERVAL;
                    }
                    Err(RecvTimeoutError::Disconnected) => {
                        flush_usage_persist(&mut pending);
                        break;
                    }
                }
            }
        })
        .expect("spawn lay usage persistence writer");
    sender
}

#[cfg(not(test))]
fn flush_usage_persist(pending: &mut HashMap<PathBuf, String>) {
    for (path, text) in std::mem::take(pending) {
        if crate::private_file::append_private_text(&path, &text).is_err() {
            continue;
        }
        compact_usage_events_if_needed(&path);
        let _ = load_usage_event_counts();
    }
}

fn compact_usage_events_if_needed(path: &Path) {
    let Ok(meta) = std::fs::metadata(path) else {
        return;
    };
    if meta.len() <= USAGE_EVENTS_MAX_BYTES {
        return;
    }
    let Some(text) = read_usage_events_text(path) else {
        return;
    };
    let compacted = keep_jsonl_tail_bytes(&text, USAGE_EVENTS_MAX_BYTES as usize);
    let _ = crate::private_file::write_private_bytes_atomic(path, compacted.as_bytes());
}

fn read_usage_events_text(path: &Path) -> Option<String> {
    read_tail_text_lossy(path, USAGE_EVENTS_MAX_BYTES as usize)
}

fn read_full_text_lossy(path: &Path) -> Option<String> {
    let bytes = std::fs::read(path).ok()?;
    Some(String::from_utf8_lossy(&bytes).into_owned())
}

fn read_tail_text_lossy(path: &Path, max_bytes: usize) -> Option<String> {
    let bytes = std::fs::read(path).ok()?;
    let start = bytes.len().saturating_sub(max_bytes);
    let text = String::from_utf8_lossy(&bytes[start..]).into_owned();
    if start == 0 {
        return Some(text);
    }
    Some(
        text.find('\n')
            .map(|index| text[index + 1..].to_string())
            .unwrap_or(text),
    )
}

fn load_usage_counts_snapshot(source_len: u64) -> Option<UsageCounts> {
    let path = usage_counts_path()?;
    load_persisted_usage_counts(&path, Some(source_len))
}

fn persist_usage_counts_snapshot(counts: &UsageCounts, source_len: u64) {
    let Some(path) = usage_counts_path() else {
        return;
    };
    let _ = persist_usage_counts_snapshot_to_path(&path, counts, source_len);
}

fn load_persisted_usage_counts(path: &Path, source_len: Option<u64>) -> Option<UsageCounts> {
    let text = std::fs::read_to_string(path).ok()?;
    let snapshot = serde_json::from_str::<PersistedUsageCounts>(&text).ok()?;
    (snapshot.schema_version == USAGE_COUNTS_SCHEMA_VERSION
        && source_len.is_none_or(|expected| snapshot.source_len == expected))
    .then_some(snapshot.counts)
}

fn persist_usage_counts_snapshot_to_path(
    path: &Path,
    counts: &UsageCounts,
    source_len: u64,
) -> std::io::Result<()> {
    let snapshot = PersistedUsageCounts {
        schema_version: USAGE_COUNTS_SCHEMA_VERSION,
        source_len,
        counts: compact_usage_counts_for_persist(counts),
    };
    let mut text = serde_json::to_string(&snapshot)?;
    text.push('\n');
    crate::private_file::write_private_text(path, &text)
}

pub fn compile_usage_feedback_snapshot(
    input: &Path,
    output: &Path,
) -> std::io::Result<serde_json::Value> {
    let text = std::fs::read_to_string(input)?;
    let mut counts = UsageCounts::default();
    add_usage_event_counts(&mut counts, &text);
    let correction_events = correction_feedback_events_from_jsonl(&text);
    for event in &correction_events {
        add_usage_event_count(&mut counts, event);
    }
    let usage_event_count = usage_events_from_jsonl(&text).count();
    let correction_receipt_count = correction_feedback_receipts_from_jsonl(&text).count();
    persist_usage_counts_snapshot_to_path(output, &counts, text.len() as u64)?;
    let hot = UsageHotState::from_counts(&counts);
    Ok(serde_json::json!({
        "kind": "typing_feedback_snapshot_compile",
        "status": "ok",
        "input": input.display().to_string(),
        "output": output.display().to_string(),
        "source_bytes": text.len(),
        "parsed_events": usage_event_count.saturating_add(correction_events.len()),
        "usage_events": usage_event_count,
        "correction_receipts": correction_receipt_count,
        "correction_events": correction_events.len(),
        "accepted_transitions": counts.transition_attract.len(),
        "rejected_transitions": counts.transition_repel.len(),
        "surface_anti_states": counts.surface_repel.len(),
        "hot_logical_payload_bytes": hot.logical_payload_bytes(),
        "authority": "signed-memory evidence only; TransitionDecisionCore and verifier retain edit authority"
    }))
}

fn correction_feedback_events_from_jsonl(text: &str) -> Vec<UsageEvent> {
    let mut events = Vec::new();
    for receipt in correction_feedback_receipts_from_jsonl(text) {
        let user_target = correction_receipt_user_target(&receipt);
        let Some(user_target) = user_target else {
            continue;
        };
        if receipt.lay_from.trim().is_empty()
            || receipt.lay_to.trim().is_empty()
            || user_target.trim().is_empty()
        {
            continue;
        }
        if correction_receipt_is_exact_system_revert(&receipt, &user_target) {
            events.extend(reverted_system_apply_usage_events(&receipt));
            continue;
        }
        let operation = if receipt.lay_kind.trim().is_empty() {
            "replacement"
        } else {
            receipt.lay_kind.as_str()
        };
        if user_target != receipt.lay_to {
            events.extend(
                TypingMemoryEvent::rejected_candidate(
                    &receipt.lay_from,
                    &receipt.lay_to,
                    "user_correction",
                    operation,
                )
                .iter()
                .map(UsageEvent::from_typing_memory_event),
            );
        }
        events.extend(
            TypingMemoryEvent::accepted_fix(&receipt.lay_from, &user_target)
                .iter()
                .map(UsageEvent::from_typing_memory_event),
        );
    }
    events
}

pub(crate) fn exact_reverted_system_apply_usage_jsonl(text: &str) -> (String, u32) {
    let mut output = String::new();
    let mut receipts = 0_u32;
    for receipt in correction_feedback_receipts_from_jsonl(text) {
        let Some(user_target) = correction_receipt_user_target(&receipt) else {
            continue;
        };
        if !correction_receipt_is_exact_system_revert(&receipt, &user_target) {
            continue;
        }
        receipts = receipts.saturating_add(1);
        for event in reverted_system_apply_usage_events(&receipt) {
            if let Ok(line) = serde_json::to_string(&event) {
                output.push_str(&line);
                output.push('\n');
            }
        }
    }
    (output, receipts)
}

fn correction_receipt_user_target(receipt: &CorrectionFeedbackReceipt) -> Option<String> {
    receipt.user_target.clone().or_else(|| {
        crate::word_buffer::reconstruct_user_correction_target(
            &receipt.lay_to,
            &receipt.from,
            &receipt.to,
        )
    })
}

fn reverted_system_apply_usage_events(receipt: &CorrectionFeedbackReceipt) -> Vec<UsageEvent> {
    TypingMemoryEvent::reverted_system_apply(
        &receipt.lay_from,
        &receipt.lay_to,
        TypingMemoryEvidenceSource::Autocorrect,
        TypingMemoryOperation::Replacement,
    )
    .iter()
    .map(UsageEvent::from_typing_memory_event)
    .collect()
}

fn correction_receipt_is_exact_system_revert(
    receipt: &CorrectionFeedbackReceipt,
    user_target: &str,
) -> bool {
    user_target == receipt.lay_from
        && receipt.from == receipt.lay_to
        && receipt.to == receipt.lay_from
}

fn correction_feedback_receipts_from_jsonl(
    text: &str,
) -> impl Iterator<Item = CorrectionFeedbackReceipt> + '_ {
    text.lines()
        .filter(|line| !line.trim().is_empty())
        .filter_map(|line| serde_json::from_str::<CorrectionFeedbackReceipt>(line).ok())
        .filter(|receipt| {
            matches!(
                receipt.kind.as_str(),
                "user-correction" | "system-apply-reverted"
            )
        })
}

fn compact_usage_counts_for_persist(counts: &UsageCounts) -> UsageCounts {
    UsageCounts {
        words: top_count_entries(&counts.words, USAGE_COUNTS_MAX_WORDS),
        accepted_words: top_count_entries(&counts.accepted_words, USAGE_COUNTS_MAX_ACCEPTED_WORDS),
        context_words: top_count_entries(&counts.context_words, USAGE_COUNTS_MAX_CONTEXT_WORDS),
        rejected_words: top_count_entries(&counts.rejected_words, USAGE_COUNTS_MAX_REJECTED_WORDS),
        rejected_context_words: top_count_entries(
            &counts.rejected_context_words,
            USAGE_COUNTS_MAX_REJECTED_CONTEXT_WORDS,
        ),
        transition_observed: top_count_entries(
            &counts.transition_observed,
            USAGE_COUNTS_MAX_TRANSITION_STATES,
        ),
        transition_attract: top_count_entries(
            &counts.transition_attract,
            USAGE_COUNTS_MAX_TRANSITION_STATES,
        ),
        transition_repel: top_count_entries(
            &counts.transition_repel,
            USAGE_COUNTS_MAX_TRANSITION_STATES,
        ),
        surface_observed: top_count_entries(
            &counts.surface_observed,
            USAGE_COUNTS_MAX_TRANSITION_STATES,
        ),
        surface_attract: top_count_entries(
            &counts.surface_attract,
            USAGE_COUNTS_MAX_TRANSITION_STATES,
        ),
        surface_repel: top_count_entries(&counts.surface_repel, USAGE_COUNTS_MAX_TRANSITION_STATES),
    }
}

fn usage_counts_cold_dictionary_logical_bytes(counts: &UsageCounts) -> usize {
    [
        &counts.words,
        &counts.accepted_words,
        &counts.context_words,
        &counts.rejected_words,
        &counts.rejected_context_words,
        &counts.transition_observed,
        &counts.transition_attract,
        &counts.transition_repel,
        &counts.surface_observed,
        &counts.surface_attract,
        &counts.surface_repel,
    ]
    .into_iter()
    .map(|map| map.keys().map(String::len).sum::<usize>())
    .sum()
}

fn top_count_entries(source: &HashMap<String, u32>, limit: usize) -> HashMap<String, u32> {
    if source.len() <= limit {
        return source.clone();
    }
    let mut entries = source
        .iter()
        .map(|(key, count)| (key.clone(), *count))
        .collect::<Vec<_>>();
    entries.sort_by(|(left_key, left_count), (right_key, right_count)| {
        right_count
            .cmp(left_count)
            .then_with(|| left_key.cmp(right_key))
    });
    entries.truncate(limit);
    entries.into_iter().collect()
}

fn keep_jsonl_tail_bytes(content: &str, max_bytes: usize) -> String {
    if content.len() <= max_bytes {
        return content.to_string();
    }
    #[derive(Debug)]
    struct EpisodeGroup {
        start: usize,
        end: usize,
        episode_id: Option<String>,
    }

    let mut groups = Vec::<EpisodeGroup>::new();
    let mut offset = 0usize;
    for line in content.split_inclusive('\n') {
        let start = offset;
        offset = offset.saturating_add(line.len());
        let episode_id = serde_json::from_str::<serde_json::Value>(line.trim())
            .ok()
            .and_then(|value| {
                value
                    .get("episode_id")
                    .and_then(serde_json::Value::as_str)
                    .map(str::to_string)
            });
        let continues_episode = episode_id.is_some()
            && groups
                .last()
                .is_some_and(|group| group.episode_id == episode_id);
        if continues_episode {
            groups.last_mut().expect("checked group").end = offset;
        } else {
            groups.push(EpisodeGroup {
                start,
                end: offset,
                episode_id,
            });
        }
    }
    if offset < content.len() {
        groups.push(EpisodeGroup {
            start: offset,
            end: content.len(),
            episode_id: None,
        });
    }
    let mut selected_start = content.len();
    let mut selected_bytes = 0usize;
    for group in groups.iter().rev() {
        let group_bytes = group.end.saturating_sub(group.start);
        if selected_bytes > 0 && selected_bytes.saturating_add(group_bytes) > max_bytes {
            break;
        }
        selected_start = group.start;
        selected_bytes = selected_bytes.saturating_add(group_bytes);
        if selected_bytes >= max_bytes {
            break;
        }
    }
    content[selected_start..].to_string()
}

fn context_ngram_keys(context: &[String]) -> Vec<String> {
    let normalized = context
        .iter()
        .filter_map(|word| {
            let word = normalize_memory_word(word);
            (!word.is_empty()).then_some(word)
        })
        .collect::<Vec<_>>();
    let max_len = normalized.len().min(CONTEXT_WORDS);
    (MIN_CONTEXT_NGRAM..=max_len)
        .filter_map(|len| {
            let start = normalized.len().saturating_sub(len);
            let key = normalized[start..].join(" ");
            (!key.is_empty()).then_some(key)
        })
        .collect()
}

fn context_word_key(context: &str, word: &str) -> String {
    format!("{context}\u{1f}{word}")
}

fn add_transition_counts(
    target: &mut HashMap<String, u32>,
    context: &[String],
    source: &str,
    operation: &str,
    state_word: &str,
    word: &str,
    weight: u32,
) {
    for key in transition_record_keys(context, source, operation, state_word, word) {
        *target.entry(key.clone()).or_default() = target
            .get(&key)
            .copied()
            .unwrap_or_default()
            .saturating_add(weight);
    }
}

fn add_state_authority_repel(
    target: &mut HashMap<String, u32>,
    _context: &[String],
    source: &str,
    operation: &str,
    state_word: &str,
    weight: u32,
) {
    for key in
        transition_lookup_keys_from_context_keys(&[], source, operation, state_word, TRANSITION_ANY)
    {
        *target.entry(key.clone()).or_default() = target
            .get(&key)
            .copied()
            .unwrap_or_default()
            .saturating_add(weight);
    }
}

fn transition_record_keys(
    context: &[String],
    source: &str,
    operation: &str,
    state_word: &str,
    word: &str,
) -> Vec<String> {
    let context_keys = context_ngram_keys(context);
    let mut keys = transition_lookup_keys_from_context_keys(
        &context_keys,
        source,
        operation,
        state_word,
        word,
    );
    if state_word != TRANSITION_ANY {
        keys.extend(transition_lookup_keys_from_context_keys(
            &context_keys,
            source,
            operation,
            TRANSITION_ANY,
            word,
        ));
    }
    keys.sort();
    keys.dedup();
    keys
}

fn transition_lookup_keys_from_context_keys(
    context_keys: &[String],
    source: &str,
    operation: &str,
    state_word: &str,
    word: &str,
) -> Vec<String> {
    let mut keys = Vec::new();
    let contexts = if context_keys.is_empty() {
        vec![String::new()]
    } else {
        context_keys.to_vec()
    };
    for context_key in contexts {
        keys.push(transition_key(
            &context_key,
            source,
            operation,
            state_word,
            word,
        ));
        keys.push(transition_key(
            &context_key,
            TRANSITION_ANY,
            operation,
            state_word,
            word,
        ));
        keys.push(transition_key(
            &context_key,
            TRANSITION_ANY,
            TRANSITION_ANY,
            state_word,
            word,
        ));
    }
    keys.sort();
    keys.dedup();
    keys
}

fn transition_key(
    context: &str,
    source: &str,
    operation: &str,
    state_word: &str,
    word: &str,
) -> String {
    format!("{context}\u{1e}{source}\u{1e}{operation}\u{1f}{state_word}\u{1d}{word}")
}

fn top_count_json(source: &HashMap<String, u32>, limit: usize) -> Vec<serde_json::Value> {
    let mut entries = source
        .iter()
        .map(|(key, count)| (key.as_str(), *count))
        .collect::<Vec<_>>();
    entries.sort_by(|(left_key, left_count), (right_key, right_count)| {
        right_count
            .cmp(left_count)
            .then_with(|| left_key.cmp(right_key))
    });
    entries
        .into_iter()
        .take(limit)
        .map(|(key, count)| serde_json::json!({ "key": key, "count": count }))
        .collect()
}

fn usage_events_path() -> Option<PathBuf> {
    if let Some(path) = std::env::var_os("LAY_NANDA_WORD_USAGE_EVENTS").map(PathBuf::from) {
        return Some(path);
    }
    #[cfg(test)]
    {
        None
    }
    #[cfg(not(test))]
    {
        std::env::var_os("HOME")
            .map(PathBuf::from)
            .map(|home| home.join(USAGE_EVENTS_PATH))
    }
}

fn usage_counts_path() -> Option<PathBuf> {
    if let Some(path) = std::env::var_os("LAY_NANDA_WORD_USAGE_COUNTS").map(PathBuf::from) {
        return Some(path);
    }
    #[cfg(test)]
    {
        None
    }
    #[cfg(not(test))]
    {
        std::env::var_os("HOME")
            .map(PathBuf::from)
            .map(|home| home.join(USAGE_COUNTS_PATH))
    }
}

fn usage_feedback_counts_path() -> Option<PathBuf> {
    if let Some(path) = std::env::var_os("LAY_NANDA_WORD_USAGE_FEEDBACK_COUNTS").map(PathBuf::from)
    {
        return Some(path);
    }
    #[cfg(test)]
    {
        None
    }
    #[cfg(not(test))]
    {
        std::env::var_os("HOME")
            .map(PathBuf::from)
            .map(|home| home.join(USAGE_FEEDBACK_COUNTS_PATH))
    }
}

fn legacy_usage_prior_path() -> Option<PathBuf> {
    if let Some(path) = std::env::var_os("LAY_NANDA_USAGE_PRIOR").map(PathBuf::from) {
        return Some(path);
    }
    #[cfg(test)]
    {
        None
    }
    #[cfg(not(test))]
    {
        std::env::var_os("HOME")
            .map(PathBuf::from)
            .map(|home| home.join(LEGACY_USAGE_PRIOR_PATH))
    }
}

#[cfg(test)]
#[path = "usage_prior_tests.rs"]
mod tests;

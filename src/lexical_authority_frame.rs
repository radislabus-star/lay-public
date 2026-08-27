//! Immutable caller-owned identity for candidate-specific lexical authority.
//!
//! This type carries evidence; it does not grant authority by itself. Callers
//! that cannot bind every field to the current input frame must pass `None`.

use crate::config::{CorrectionSafety, LayConfig};
use crate::exact_layout_authority::{ExactAuthoritySnapshot, FactoryEngineProfile};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LexicalAuthorityCoordinatesV1 {
    runtime_owner_lease_identity: u64,
    monotonic_epoch_identity: [u64; 2],
    focus_serial: u64,
    source_window: String,
    left_context: String,
    caret_scalar: u32,
    selection: (u32, u32),
    preedit: String,
    preedit_cursor_scalar: u32,
    layout_generation: u64,
    config_generation: u64,
}

impl LexicalAuthorityCoordinatesV1 {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        runtime_owner_lease_identity: u64,
        monotonic_epoch_identity: [u64; 2],
        focus_serial: u64,
        source_window: String,
        left_context: String,
        caret_scalar: u32,
        selection: (u32, u32),
        preedit: String,
        preedit_cursor_scalar: u32,
        layout_generation: u64,
        config_generation: u64,
    ) -> Option<Self> {
        let source_scalars = u32::try_from(source_window.chars().count()).ok()?;
        let preedit_scalars = u32::try_from(preedit.chars().count()).ok()?;
        (runtime_owner_lease_identity != 0
            && monotonic_epoch_identity != [0; 2]
            && focus_serial != 0
            && caret_scalar <= source_scalars
            && selection.0 <= selection.1
            && selection.1 <= source_scalars
            && preedit_cursor_scalar <= preedit_scalars
            && layout_generation != 0
            && config_generation != 0)
            .then_some(Self {
                runtime_owner_lease_identity,
                monotonic_epoch_identity,
                focus_serial,
                source_window,
                left_context,
                caret_scalar,
                selection,
                preedit,
                preedit_cursor_scalar,
                layout_generation,
                config_generation,
            })
    }

    pub fn runtime_owner_lease_identity(&self) -> u64 {
        self.runtime_owner_lease_identity
    }
    pub(crate) fn monotonic_epoch_identity(&self) -> [u64; 2] {
        self.monotonic_epoch_identity
    }
    pub fn focus_serial(&self) -> u64 {
        self.focus_serial
    }
    pub(crate) fn source_window(&self) -> &str {
        &self.source_window
    }
    pub(crate) fn left_context(&self) -> &str {
        &self.left_context
    }
    pub(crate) fn caret_scalar(&self) -> u32 {
        self.caret_scalar
    }
    pub(crate) fn selection(&self) -> (u32, u32) {
        self.selection
    }
    pub(crate) fn preedit(&self) -> &str {
        &self.preedit
    }
    pub(crate) fn preedit_cursor_scalar(&self) -> u32 {
        self.preedit_cursor_scalar
    }
    pub fn layout_generation(&self) -> u64 {
        self.layout_generation
    }
    pub(crate) fn config_generation(&self) -> u64 {
        self.config_generation
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct TypingAssistRuleIdentityV1 {
    id: String,
    enabled: bool,
    priority: i32,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LexicalAuthorityConfigIdentityV1 {
    auto_replace: bool,
    typing_assist: bool,
    auto_switch_layout: bool,
    nanda_autocorrect: bool,
    correction_safety: CorrectionSafety,
    typing_assist_pipeline: Vec<TypingAssistRuleIdentityV1>,
    nanda_l2_weight_percent: u8,
    nanda_l3_weight_percent: u8,
    llmwave_shadow: bool,
    llmwave_apply: bool,
    nanda_l2_phase_shadow: bool,
    nanda_l2_phase_apply: bool,
    nanda_l3_phase_shadow: bool,
    nanda_precognition: bool,
    ime_bracket_candidates: bool,
    text_backend: String,
}

impl LexicalAuthorityConfigIdentityV1 {
    pub fn from_config(config: &LayConfig) -> Self {
        Self {
            auto_replace: config.auto_replace,
            typing_assist: config.typing_assist,
            auto_switch_layout: config.auto_switch_layout,
            nanda_autocorrect: config.nanda_autocorrect,
            correction_safety: config.active_correction_safety(),
            typing_assist_pipeline: config
                .typing_assist_pipeline
                .iter()
                .map(|rule| TypingAssistRuleIdentityV1 {
                    id: rule.id.clone(),
                    enabled: rule.enabled,
                    priority: rule.priority,
                })
                .collect(),
            nanda_l2_weight_percent: config.nanda_l2_weight_percent.min(200),
            nanda_l3_weight_percent: config.nanda_l3_weight_percent.min(200),
            llmwave_shadow: config.llmwave_shadow,
            llmwave_apply: config.llmwave_shadow && config.llmwave_apply,
            nanda_l2_phase_shadow: config.nanda_l2_phase_shadow,
            nanda_l2_phase_apply: config.nanda_l2_phase_shadow && config.nanda_l2_phase_apply,
            nanda_l3_phase_shadow: config.nanda_l3_phase_shadow,
            nanda_precognition: config.nanda_precognition,
            ime_bracket_candidates: config.ime_bracket_candidates,
            text_backend: config.text_backend.trim().to_ascii_lowercase(),
        }
    }

    pub fn matches_config(&self, config: &LayConfig) -> bool {
        self == &Self::from_config(config)
    }

    pub fn identity_fingerprint(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        hasher.finish().max(1)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LexicalAuthorityFrameV1 {
    path: String,
    focus_receipt: Option<String>,
    tail_epoch: u64,
    committed_tail: String,
    context_prefix: String,
    observed_token: String,
    active_composition: bool,
    active_layout_is_ru: bool,
    factory_engine_profile: FactoryEngineProfile,
    exact_authority_snapshot: Option<ExactAuthoritySnapshot>,
    output_capability_fingerprint: u64,
    frame_fingerprint: u64,
    config: LexicalAuthorityConfigIdentityV1,
    coordinates: Option<LexicalAuthorityCoordinatesV1>,
}

impl LexicalAuthorityFrameV1 {
    #[allow(clippy::too_many_arguments)]
    pub fn from_exact_parts(
        path: String,
        focus_receipt: Option<String>,
        tail_epoch: u64,
        committed_tail: String,
        context_prefix: String,
        observed_token: String,
        active_composition: bool,
        active_layout_is_ru: bool,
        factory_engine_profile: FactoryEngineProfile,
        exact_authority_snapshot: Option<ExactAuthoritySnapshot>,
        output_capability_fingerprint: u64,
        frame_fingerprint: u64,
        config: LexicalAuthorityConfigIdentityV1,
    ) -> Self {
        Self {
            path,
            focus_receipt,
            tail_epoch,
            committed_tail,
            context_prefix,
            observed_token,
            active_composition,
            active_layout_is_ru,
            factory_engine_profile,
            exact_authority_snapshot,
            output_capability_fingerprint,
            frame_fingerprint,
            config,
            coordinates: None,
        }
    }

    pub fn with_coordinates(mut self, coordinates: Option<LexicalAuthorityCoordinatesV1>) -> Self {
        self.coordinates = coordinates;
        self
    }

    pub fn path(&self) -> &str {
        &self.path
    }
    pub fn focus_receipt(&self) -> Option<&str> {
        self.focus_receipt.as_deref()
    }
    pub fn tail_epoch(&self) -> u64 {
        self.tail_epoch
    }
    pub fn committed_tail(&self) -> &str {
        &self.committed_tail
    }
    pub fn context_prefix(&self) -> &str {
        &self.context_prefix
    }
    pub fn observed_token(&self) -> &str {
        &self.observed_token
    }
    pub fn active_composition(&self) -> bool {
        self.active_composition
    }
    pub fn active_layout_is_ru(&self) -> bool {
        self.active_layout_is_ru
    }
    pub fn factory_engine_profile(&self) -> FactoryEngineProfile {
        self.factory_engine_profile
    }
    pub fn exact_authority_snapshot(&self) -> Option<ExactAuthoritySnapshot> {
        self.exact_authority_snapshot
    }
    pub fn output_capability_fingerprint(&self) -> u64 {
        self.output_capability_fingerprint
    }
    pub fn frame_fingerprint(&self) -> u64 {
        self.frame_fingerprint
    }
    pub fn config(&self) -> &LexicalAuthorityConfigIdentityV1 {
        &self.config
    }
    pub(crate) fn coordinates(&self) -> Option<&LexicalAuthorityCoordinatesV1> {
        self.coordinates.as_ref()
    }
}

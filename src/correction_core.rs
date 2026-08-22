//! Shared text-correction decision facade.
//!
//! Runtime backends still own output and state. This module only answers one
//! question: should this completed text be replaced, and by which engine?

pub use crate::candidate_contract::CandidateReadoutRoute;
use crate::candidate_contract::{CandidateOrigin, CorrectionSourceRole};
use crate::candidate_explanation::{explain_candidate, CandidateExplanation};
use crate::config::{CorrectionSafety, TypingAssistRuleConfig};
use crate::nanda_wave::{run_wave_trace_with_options, WaveOptions, WordCandidate};
use crate::russian_typo_candidates::{
    inserted_char_position_for_missing_letter, repeated_run_deletion_candidates,
};
use crate::text_case::apply_word_case;
use crate::text_metrics::{damerau_levenshtein, has_cyrillic};
use crate::typing_assist::split_ws_segments;
use crate::typing_candidate::TypingCandidateFamily;
use crate::typing_context::{syntax_allows_candidate, typing_assist_pipeline_for_context};
use crate::typing_pipeline::{
    collect_typing_assist_candidates_with_pipeline, explain_typing_assist_with_pipeline,
};
use crate::typing_rule_graph::ids;
pub use crate::typing_transition::proposal_admission::{
    CandidateGateAction, CandidateGateDecision,
};
use crate::typing_transition::{
    action as action_operator,
    candidate::L2CandidateLattice,
    decision::{CandidateDecisionBatch, TransitionDecisionCore},
    state::L1SurfaceSignal,
};
use crate::word_reader::{
    cyrillic_word_splits, is_cyrillic_letters_only, last_text_word, replace_last_text_word,
    split_edge_whitespace, split_last_trimmed_ws_token, split_word_punctuation,
};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::OnceLock;
use std::time::Instant;

const COMPOSITE_TRANSPOSE_MIN_MARGIN: f64 = -8.0;
#[cfg(test)]
use crate::typing_transition::proposal_admission::gate_candidate;
#[cfg(test)]
use crate::typing_transition::proposal_admission::gate_candidate_with_origin;
#[cfg(test)]
use crate::typing_transition::proposal_admission::gate_candidate_with_source;
pub(crate) use crate::typing_transition::proposal_admission::normalized_correction_words;
use crate::typing_transition::proposal_admission::{
    repeated_deletion_has_surface_support, should_prefer_composite_after_repeated_repair,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CorrectionMode {
    DeterministicOnly,
    NandaOnly,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CorrectionDecisionSource {
    Deterministic,
    Nanda,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TypingErrorClass {
    WrongLayout,
    PartialLayout,
    MixedScript,
    MissingLetter,
    SparseInternalMultiOmission,
    ExtraLetter,
    RepeatedLetter,
    AdjacentTransposition,
    LetterSubstitution,
    CompositeTypo,
    BoundaryShift,
    SplitWord,
    GluedWords,
    CaseNoise,
    GrammarAgreement,
    CompletionOnly,
    TechnicalToken,
    ProtectedToken,
    Unknown,
}

impl TypingErrorClass {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::WrongLayout => "wrong_layout",
            Self::PartialLayout => "partial-layout",
            Self::MixedScript => "mixed-script",
            Self::MissingLetter => "missing-letter",
            Self::SparseInternalMultiOmission => "sparse-internal-multi-omission",
            Self::ExtraLetter => "extra-letter",
            Self::RepeatedLetter => "repeated-letter",
            Self::AdjacentTransposition => "adjacent-transposition",
            Self::LetterSubstitution => "letter-substitution",
            Self::CompositeTypo => "composite-typo",
            Self::BoundaryShift => "boundary-shift",
            Self::SplitWord => "split-word",
            Self::GluedWords => "glued-words",
            Self::CaseNoise => "case-noise",
            Self::GrammarAgreement => "grammar-agreement",
            Self::CompletionOnly => "completion-only",
            Self::TechnicalToken => "technical-token",
            Self::ProtectedToken => "protected-token",
            Self::Unknown => "unknown",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypingErrorEvent {
    pub original: String,
    pub core: String,
    pub current_word: String,
    pub input_class: TypingErrorClass,
}

#[derive(Debug, Clone, PartialEq)]
pub struct UnifiedCorrectionCandidate {
    pub replacement: String,
    pub source: CorrectionDecisionSource,
    pub(crate) origin: CandidateOrigin,
    pub source_id: String,
    pub error_class: TypingErrorClass,
    pub gate: CandidateGateDecision,
    pub(crate) evidence: Vec<CandidateEvidence>,
    pub(crate) morphology_slot_evidence: Vec<MorphologySlotEvidence>,
    pub(crate) authority_evidence: CandidateAuthorityEvidence,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum CandidateAuthorityEvidence {
    None,
    ClosedExactLayout(crate::exact_layout_authority::ExactLayoutContourCertificate),
    Conflict,
}

impl CandidateAuthorityEvidence {
    fn merge(self, incoming: Self) -> Self {
        match (self, incoming) {
            (Self::Conflict, _) | (_, Self::Conflict) => Self::Conflict,
            (Self::None, evidence) | (evidence, Self::None) => evidence,
            (Self::ClosedExactLayout(left), Self::ClosedExactLayout(right)) => {
                if left == right {
                    Self::ClosedExactLayout(left)
                } else {
                    Self::Conflict
                }
            }
        }
    }
}

/// Stable identity used only to preserve morphology diversity in bounded
/// candidate lattices. The domain is explicit because canonical morphology
/// feature masks and Productive V90 slot IDs are different number spaces.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct MorphologySlotIdentity {
    pub(crate) domain: MorphologySlotIdentityDomain,
    pub(crate) lemma_id: u32,
    pub(crate) slot_id: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) enum MorphologySlotIdentityDomain {
    CanonicalFeature,
    ProductiveV1,
}

/// Typed L2 evidence for choosing a morphology slot inside one lemma basin.
///
/// This metadata may affect candidate order, but it never grants mutation
/// authority. Generated surfaces remain subject to their producer gate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct MorphologySlotEvidence {
    pub(crate) lemma_id: u32,
    pub(crate) source_feature_mask: u32,
    pub(crate) target_feature_mask: u32,
    pub(crate) context_positive_support: u32,
    pub(crate) context_alternative_support: u32,
    pub(crate) context_posterior_milli: u16,
    pub(crate) slot_evidence_milli: i32,
    pub(crate) joint_evidence_milli: u16,
    pub(crate) generated: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CandidateEvidence {
    pub(crate) source: CorrectionDecisionSource,
    pub(crate) origin: CandidateOrigin,
    pub(crate) source_id: String,
    pub(crate) error_class: TypingErrorClass,
    pub(crate) gate: CandidateGateDecision,
}

fn common_correction_target_evidence(
    replacement: &str,
    origin: CandidateOrigin,
    error_class: TypingErrorClass,
) -> crate::typing_transition::target_evidence::TargetEvidenceSetV1 {
    use crate::typing_transition::target_evidence::{
        stable_bytes_ref, target_relation_from_error_class_tag, GroundingNamespaceV1,
        TargetEvidenceSetV1, TargetWitnessV1, VerdictMembershipV1,
    };

    let relation = target_relation_from_error_class_tag(error_class.as_str());
    let target_ref = stable_bytes_ref(replacement.as_bytes());
    let operator_ref = stable_bytes_ref(error_class.as_str().as_bytes());
    let provenance_annotations = match origin {
        CandidateOrigin::Layout => 1 << 0,
        CandidateOrigin::LayoutThenTypo => 1 << 1,
        CandidateOrigin::Boundary => 1 << 2,
        CandidateOrigin::Completion => 1 << 3,
        CandidateOrigin::L2Surface => 1 << 4,
        CandidateOrigin::L3Context => 1 << 5,
        CandidateOrigin::DeterministicTypo => 1 << 6,
        CandidateOrigin::Technical => 1 << 7,
    };
    TargetEvidenceSetV1::from_one(TargetWitnessV1::new(
        relation,
        GroundingNamespaceV1::LegacyCorrectionCandidate,
        VerdictMembershipV1::Born,
        u8::from(origin == CandidateOrigin::Boundary),
        operator_ref,
        target_ref,
        target_ref,
        0,
        provenance_annotations,
    ))
}

impl UnifiedCorrectionCandidate {
    pub(crate) fn new(
        replacement: impl Into<String>,
        source: CorrectionDecisionSource,
        origin: CandidateOrigin,
        source_id: impl Into<String>,
        error_class: TypingErrorClass,
        gate: CandidateGateDecision,
    ) -> Self {
        let replacement = replacement.into();
        let source_id = source_id.into();
        Self {
            replacement,
            source,
            origin,
            source_id: source_id.clone(),
            error_class,
            gate: gate.clone(),
            evidence: vec![CandidateEvidence {
                source,
                origin,
                source_id,
                error_class,
                gate,
            }],
            morphology_slot_evidence: Vec::new(),
            authority_evidence: CandidateAuthorityEvidence::None,
        }
    }

    pub(crate) fn with_closed_exact_layout_authority(
        mut self,
        certificate: crate::exact_layout_authority::ExactLayoutContourCertificate,
    ) -> Self {
        self.authority_evidence = CandidateAuthorityEvidence::ClosedExactLayout(certificate);
        self
    }

    pub(crate) fn merge_evidence(&mut self, candidate: Self) {
        let retains_closed_exact_identity = self.closed_exact_layout_certificate().is_some();
        self.authority_evidence = std::mem::replace(
            &mut self.authority_evidence,
            CandidateAuthorityEvidence::None,
        )
        .merge(candidate.authority_evidence.clone());
        let promote_verified_same_replacement = candidate.gate.action
            == CandidateGateAction::Eligible
            && self.gate.action == CandidateGateAction::SuggestOnly;
        let promote_verified_layout = candidate.origin == CandidateOrigin::Layout
            && candidate.gate.action == CandidateGateAction::Eligible
            && (self.gate.action == CandidateGateAction::SuggestOnly
                || self.origin == CandidateOrigin::LayoutThenTypo);
        let promote_wave_layout_owner = candidate.source == CorrectionDecisionSource::Nanda
            && candidate.origin == CandidateOrigin::Layout
            && candidate.gate.action == CandidateGateAction::Eligible
            && self.source == CorrectionDecisionSource::Deterministic
            && self.origin.source_role() == CorrectionSourceRole::Layout
            && !matches!(
                self.gate.action,
                CandidateGateAction::KeepOriginal | CandidateGateAction::Veto
            );
        let promote_wave_owner = candidate.origin.source_role() == CorrectionSourceRole::L2Surface
            && self.origin.source_role() == CorrectionSourceRole::DeterministicTypo
            && matches!(
                (self.gate.action, candidate.gate.action),
                (CandidateGateAction::Eligible, CandidateGateAction::Eligible)
                    | (
                        CandidateGateAction::SuggestOnly,
                        CandidateGateAction::Eligible
                    )
                    | (
                        CandidateGateAction::SuggestOnly,
                        CandidateGateAction::SuggestOnly
                    )
            );
        let promote_wave_boundary_owner = candidate.source == CorrectionDecisionSource::Nanda
            && candidate.origin == CandidateOrigin::Boundary
            && candidate.gate.action == CandidateGateAction::Eligible
            && self.source == CorrectionDecisionSource::Deterministic
            && self.origin == CandidateOrigin::Boundary
            && !matches!(
                self.gate.action,
                CandidateGateAction::KeepOriginal | CandidateGateAction::Veto
            );
        if !retains_closed_exact_identity
            && (promote_verified_same_replacement
                || promote_verified_layout
                || promote_wave_layout_owner
                || promote_wave_owner
                || promote_wave_boundary_owner)
        {
            self.source = candidate.source;
            self.origin = candidate.origin;
            self.source_id.clone_from(&candidate.source_id);
            self.error_class = candidate.error_class;
            self.gate = candidate.gate.clone();
        }
        for evidence in candidate.evidence {
            let already_present = self.evidence.iter().any(|existing| {
                existing.source == evidence.source
                    && existing.origin == evidence.origin
                    && existing.source_id == evidence.source_id
                    && existing.error_class == evidence.error_class
                    && existing.gate == evidence.gate
            });
            if !already_present {
                self.evidence.push(evidence);
            }
        }
        self.extend_morphology_slot_evidence(candidate.morphology_slot_evidence);
    }

    pub(crate) fn closed_exact_layout_certificate(
        &self,
    ) -> Option<&crate::exact_layout_authority::ExactLayoutContourCertificate> {
        match &self.authority_evidence {
            CandidateAuthorityEvidence::ClosedExactLayout(certificate) => Some(certificate),
            CandidateAuthorityEvidence::None | CandidateAuthorityEvidence::Conflict => None,
        }
    }

    pub(crate) fn has_authority_conflict(&self) -> bool {
        self.authority_evidence == CandidateAuthorityEvidence::Conflict
    }

    pub(crate) fn common_target_evidence(
        &self,
    ) -> crate::typing_transition::target_evidence::TargetEvidenceSetV1 {
        self.evidence.iter().fold(
            crate::typing_transition::target_evidence::TargetEvidenceSetV1::complete_empty(),
            |merged, evidence| {
                merged.merge(common_correction_target_evidence(
                    &self.replacement,
                    evidence.origin,
                    evidence.error_class,
                ))
            },
        )
    }

    pub(crate) fn has_origin(&self, origin: CandidateOrigin) -> bool {
        self.evidence
            .iter()
            .any(|evidence| evidence.origin == origin)
    }

    pub(crate) fn has_source_id(&self, source_id: &str) -> bool {
        self.evidence
            .iter()
            .any(|evidence| evidence.source_id == source_id)
    }

    pub(crate) fn evidence_count(&self) -> usize {
        self.evidence.len()
    }

    pub(crate) fn extend_morphology_slot_evidence(
        &mut self,
        evidence: impl IntoIterator<Item = MorphologySlotEvidence>,
    ) {
        for evidence in evidence {
            if !self.morphology_slot_evidence.contains(&evidence) {
                self.morphology_slot_evidence.push(evidence);
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CorrectionResolution {
    pub event: TypingErrorEvent,
    pub candidates: Vec<UnifiedCorrectionCandidate>,
    pub selected: Option<UnifiedCorrectionCandidate>,
    pub decision: Option<CorrectionDecision>,
    pub scoreboard: CorrectionScoreboard,
    pub(crate) candidate_scores: Vec<CorrectionCandidateScoreTrace>,
    pub(crate) selected_transition:
        Option<crate::typing_transition::decision::DecisionTransitionReceipt>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct CorrectionRouteTelemetry {
    pub(crate) canonical_field: crate::nanda_wave::l2_field::CanonicalFieldTelemetry,
    pub(crate) correction_l3_us: u64,
    pub(crate) decision_total_us: u64,
    pub(crate) total_us: u64,
}

pub(crate) struct ObservedCorrectionResolution {
    pub(crate) resolution: CorrectionResolution,
    pub(crate) telemetry: CorrectionRouteTelemetry,
}

#[derive(Debug, Clone)]
pub struct CorrectionRequest<'a> {
    pub text: &'a str,
    pub auto_replace: bool,
    pub typing_assist: bool,
    pub auto_switch_layout: bool,
    pub correction_safety: CorrectionSafety,
    pub typing_assist_pipeline: &'a [TypingAssistRuleConfig],
    pub nanda_autocorrect: bool,
    pub nanda_candidate_route: CandidateReadoutRoute,
    pub nanda_wave_options: WaveOptions,
    pub mode: CorrectionMode,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CorrectionDecision {
    pub replacement: String,
    pub source: CorrectionDecisionSource,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct CorrectionScoreboard {
    pub total_candidates: usize,
    pub apply_candidates: usize,
    pub suggest_only_candidates: usize,
    pub keep_original_candidates: usize,
    pub veto_candidates: usize,
    pub deterministic_candidates: usize,
    pub nanda_candidates: usize,
    pub selected_bayes_posterior_milli: Option<i16>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
struct CorrectionGateRuntimeStats {
    requests: u64,
    total_candidates: u64,
    apply_candidates: u64,
    suggest_only_candidates: u64,
    keep_original_candidates: u64,
    veto_candidates: u64,
    deterministic_candidates: u64,
    nanda_candidates: u64,
    selected_apply: u64,
    total_us: u64,
    max_us: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CorrectionCandidateScoreTrace {
    pub(crate) replacement: String,
    pub(crate) source: CorrectionDecisionSource,
    pub(crate) source_id: String,
    pub(crate) error_class: TypingErrorClass,
    pub(crate) action_operator: &'static str,
    pub(crate) action_proof: &'static str,
    pub(crate) edit_transition_operator: &'static str,
    pub(crate) edit_transition_proof: &'static str,
    pub(crate) edit_transition_operator_kind: crate::text_edit::TransitionOperator,
    pub(crate) edit_transition_proof_kind: crate::text_edit::TransitionProof,
    pub(crate) edit_transition_verified: bool,
    pub(crate) edit_transition_left_context_changed: bool,
    pub(crate) edit_transition_changed_tokens: usize,
    pub(crate) edit_shape: &'static str,
    pub(crate) preservation_milli: i16,
    pub(crate) lost_mass_milli: i16,
    pub(crate) added_mass_milli: i16,
    pub(crate) operator_fit_milli: i16,
    pub(crate) shortcut_risk_milli: i16,
    pub(crate) anti_wave_milli: i16,
    pub(crate) explanation_score_milli: i16,
    pub(crate) gate_action: CandidateGateAction,
    pub(crate) gate_reason: &'static str,
    pub(crate) likelihood_milli: i16,
    pub(crate) usage_prior_milli: i16,
    pub(crate) context_prior_milli: i16,
    pub(crate) transition_field_milli: i16,
    pub(crate) transition_field_attraction_milli: i16,
    pub(crate) transition_field_repulsion_milli: i16,
    pub(crate) transition_field_uncertainty_milli: i16,
    pub(crate) transition_field_phase_competition_milli: i16,
    pub(crate) l2_wave_peak_milli: i16,
    pub(crate) l2_wave_peak_positive_milli: i16,
    pub(crate) l2_wave_peak_negative_milli: i16,
    pub(crate) l2_wave_peak_uncertainty_milli: i16,
    pub(crate) l2_wave_peak_reason: &'static str,
    pub(crate) l2_transition_phase_milli: i16,
    pub(crate) l2_transition_phase_threshold_milli: i16,
    pub(crate) l2_transition_phase_verdict: &'static str,
    pub(crate) l2_transition_phase_package_loaded: bool,
    pub(crate) l2_transition_phase_operator_present: bool,
    pub(crate) l2_transition_phase_operator_promoted: bool,
    pub(crate) l2_transition_phase_positive_centers: u8,
    pub(crate) l2_transition_phase_anti_centers: u8,
    pub(crate) l2_transition_phase_surfaces: u32,
    pub(crate) l2_morphology_milli: i16,
    pub(crate) l2_morphology_disposition: &'static str,
    pub(crate) l2_morphology_lemma_id: u32,
    pub(crate) l2_morphology_source_feature_mask: u32,
    pub(crate) l2_morphology_target_feature_mask: u32,
    pub(crate) l2_morphology_context_posterior_milli: u16,
    pub(crate) l2_morphology_slot_evidence_milli: i32,
    pub(crate) l2_morphology_joint_evidence_milli: u16,
    pub(crate) l2_morphology_competitors: u16,
    pub(crate) l2_morphology_generated: bool,
    pub(crate) l3_phrase_milli: i16,
    pub(crate) l3_phrase_decision: &'static str,
    pub(crate) l4_hidden_disposition: &'static str,
    pub(crate) l4_hidden_semantic_classes: u16,
    pub(crate) l4_hidden_unresolved_classes: u16,
    pub(crate) l4_hidden_plan_commitment: u64,
    pub(crate) l4_hidden_receipts: u8,
    pub(crate) l4_hidden_probe: &'static str,
    pub(crate) l4_hidden_certificate_valid: bool,
    pub(crate) l4_scene_milli: i16,
    pub(crate) l4_scene_action: &'static str,
    pub(crate) l4_scene_reason: &'static str,
    pub(crate) l4_signed_milli: i16,
    pub(crate) l4_signed_reason: &'static str,
    pub(crate) l4_surface_status: &'static str,
    pub(crate) l4_transition_state_specific: bool,
    pub(crate) l4_transition_attract_count: u32,
    pub(crate) l4_transition_repel_count: u32,
    pub(crate) l4_phase_witness_milli: i16,
    pub(crate) l4_phase_witness_supported: bool,
    pub(crate) l4_phase_positive_centers: u8,
    pub(crate) l4_phase_negative_centers: u8,
    pub(crate) l4_cross_scene_package_loaded: bool,
    pub(crate) l4_cross_scene_profile_present: bool,
    pub(crate) l4_cross_scene_disposition: &'static str,
    pub(crate) l4_cross_scene_recommendation: &'static str,
    pub(crate) l4_cross_scene_margin_milli: i16,
    pub(crate) l4_cross_scene_threshold_milli: i16,
    pub(crate) l4_cross_scene_pair_margin_milli: i16,
    pub(crate) l4_cross_scene_automatic_apply: bool,
    pub(crate) risk_milli: i16,
    pub(crate) posterior_milli: i16,
    pub(crate) decision_rank_milli: i16,
    pub(crate) selected: bool,
}

pub fn decide_text_correction(req: CorrectionRequest<'_>) -> Option<CorrectionDecision> {
    resolve_text_correction(req).decision
}

pub fn resolve_text_correction(req: CorrectionRequest<'_>) -> CorrectionResolution {
    resolve_text_correction_observed(req).resolution
}

pub(crate) fn resolve_text_correction_observed(
    req: CorrectionRequest<'_>,
) -> ObservedCorrectionResolution {
    resolve_text_correction_observed_internal(req, CorrectionEvidenceScope::FullField(None))
}

pub(crate) fn resolve_text_correction_observed_with_exact(
    req: CorrectionRequest<'_>,
    certificate: &crate::exact_layout_authority::ExactLayoutContourCertificate,
) -> ObservedCorrectionResolution {
    resolve_text_correction_observed_internal(
        req,
        CorrectionEvidenceScope::FullField(Some(certificate)),
    )
}

pub(crate) fn resolve_closed_exact_text_correction_observed(
    req: CorrectionRequest<'_>,
    certificate: Option<&crate::exact_layout_authority::ExactLayoutContourCertificate>,
) -> ObservedCorrectionResolution {
    resolve_text_correction_observed_internal(
        req,
        CorrectionEvidenceScope::ClosedExact(certificate),
    )
}

#[derive(Clone, Copy)]
enum CorrectionEvidenceScope<'a> {
    FullField(Option<&'a crate::exact_layout_authority::ExactLayoutContourCertificate>),
    ClosedExact(Option<&'a crate::exact_layout_authority::ExactLayoutContourCertificate>),
}

fn resolve_text_correction_observed_internal(
    req: CorrectionRequest<'_>,
    scope: CorrectionEvidenceScope<'_>,
) -> ObservedCorrectionResolution {
    let started = Instant::now();
    let timing_enabled = std::env::var_os("LAY_CORRECTION_CORE_TIMING").is_some();
    let mut l2_peak_context = None;
    let peak_ready = Instant::now();
    let mut lattice = L2CandidateLattice::with_options(
        TypingErrorEvent::from_text(req.text),
        &req.nanda_wave_options,
    );
    let mut canonical_telemetry = crate::nanda_wave::l2_field::CanonicalFieldTelemetry::default();

    let exact_certificate = match scope {
        CorrectionEvidenceScope::FullField(certificate)
        | CorrectionEvidenceScope::ClosedExact(certificate) => certificate,
    };
    if let Some(certificate) = exact_certificate {
        match retained_exact_layout_candidate(&req, certificate) {
            Some(candidate) => lattice.retain_exact(candidate),
            None => lattice.reject_exact_authority(),
        }
    }

    if matches!(scope, CorrectionEvidenceScope::FullField(_)) {
        for source in L2CandidateSource::for_mode(req.mode) {
            source.push_candidates(
                &req,
                &mut lattice,
                l2_peak_context.as_ref(),
                &mut canonical_telemetry,
            );
        }
        if L2CandidateSource::for_mode(req.mode).contains(&L2CandidateSource::Deterministic) {
            lattice.push_source(short_cyrillic_layout_suggestion_candidate(&req));
        }
    }
    let candidates_ready = Instant::now();

    if matches!(scope, CorrectionEvidenceScope::FullField(_))
        && !lattice.is_empty()
        && l2_peak_context.is_none()
    {
        l2_peak_context =
            Some(crate::nanda_wave::l2_wave_peak::prepare_correction_peak_context(req.text));
    }
    let (resolution, decision_timing) = match scope {
        CorrectionEvidenceScope::FullField(_) => {
            lattice.into_observed_resolution_with_peak_context(l2_peak_context.as_ref())
        }
        CorrectionEvidenceScope::ClosedExact(certificate) => {
            lattice.into_closed_exact_resolution(certificate)
        }
    };
    if timing_enabled {
        let decision_ready = Instant::now();
        eprintln!(
            "lay_correction_core_timing peak_us={} candidates_us={} decision_us={} total_us={} candidates={}",
            peak_ready.duration_since(started).as_micros(),
            candidates_ready.duration_since(peak_ready).as_micros(),
            decision_ready.duration_since(candidates_ready).as_micros(),
            decision_ready.duration_since(started).as_micros(),
            resolution.candidates.len(),
        );
    }
    record_correction_gate_stats(started, &resolution);
    ObservedCorrectionResolution {
        resolution,
        telemetry: CorrectionRouteTelemetry {
            canonical_field: canonical_telemetry,
            correction_l3_us: decision_timing.l3_us,
            decision_total_us: decision_timing.total_us,
            total_us: started.elapsed().as_micros().min(u128::from(u64::MAX)) as u64,
        },
    }
}

pub fn correction_gate_stats_json() -> serde_json::Value {
    let stats = correction_gate_runtime_stats();
    let avg_us = stats.total_us.checked_div(stats.requests).unwrap_or(0);
    serde_json::json!({
        "requests": stats.requests,
        "total_candidates": stats.total_candidates,
        "apply_candidates": stats.apply_candidates,
        "suggest_only_candidates": stats.suggest_only_candidates,
        "keep_original_candidates": stats.keep_original_candidates,
        "veto_candidates": stats.veto_candidates,
        "deterministic_candidates": stats.deterministic_candidates,
        "nanda_candidates": stats.nanda_candidates,
        "selected_apply": stats.selected_apply,
        "avg_us": avg_us,
        "max_us": stats.max_us,
    })
}

pub fn query_canonical_l2_context(text: &str) -> serde_json::Value {
    let pipeline = crate::config::default_typing_assist_pipeline();
    let started = Instant::now();
    let resolution = resolve_text_correction(CorrectionRequest {
        text,
        auto_replace: true,
        typing_assist: true,
        auto_switch_layout: true,
        correction_safety: CorrectionSafety::Experimental,
        typing_assist_pipeline: &pipeline,
        nanda_autocorrect: true,
        nanda_candidate_route: CandidateReadoutRoute::CanonicalL2Field,
        nanda_wave_options: WaveOptions::default(),
        mode: CorrectionMode::NandaOnly,
    });
    let mut candidates = resolution
        .candidates
        .iter()
        .zip(&resolution.candidate_scores)
        .map(|(candidate, score)| {
            serde_json::json!({
                "replacement": candidate.replacement,
                "source_id": candidate.source_id,
                "error_class": candidate.error_class.as_str(),
                "gate_action": format!("{:?}", candidate.gate.action),
                "gate_reason": candidate.gate.reason,
                "productive": candidate.source_id == "CanonicalL2ProductiveSurface",
                "morphology_evidence": candidate.morphology_slot_evidence.iter().map(|evidence| serde_json::json!({
                    "lemma_id": evidence.lemma_id,
                    "source_feature_mask": evidence.source_feature_mask,
                    "target_feature_mask": evidence.target_feature_mask,
                    "context_positive_support": evidence.context_positive_support,
                    "context_alternative_support": evidence.context_alternative_support,
                    "context_posterior_milli": evidence.context_posterior_milli,
                    "slot_evidence_milli": evidence.slot_evidence_milli,
                    "joint_evidence_milli": evidence.joint_evidence_milli,
                    "generated": evidence.generated,
                })).collect::<Vec<_>>(),
                "l2_morphology_milli": score.l2_morphology_milli,
                "l2_morphology_disposition": score.l2_morphology_disposition,
                "l2_morphology_lemma_id": score.l2_morphology_lemma_id,
                "l2_morphology_target_feature_mask": score.l2_morphology_target_feature_mask,
                "l2_morphology_competitors": score.l2_morphology_competitors,
                "l3_phrase_milli": score.l3_phrase_milli,
                "l3_phrase_decision": score.l3_phrase_decision,
                "l4_cross_scene_disposition": score.l4_cross_scene_disposition,
                "posterior_milli": score.posterior_milli,
                "risk_milli": score.risk_milli,
                "decision_rank_milli": score.decision_rank_milli,
                "selected": score.selected,
            })
        })
        .collect::<Vec<_>>();
    candidates.sort_by(|left, right| {
        right["decision_rank_milli"]
            .as_i64()
            .cmp(&left["decision_rank_milli"].as_i64())
            .then_with(|| {
                left["replacement"]
                    .as_str()
                    .cmp(&right["replacement"].as_str())
            })
    });
    serde_json::json!({
        "kind": "canonical_l2_l3_context_live_query",
        "text": text,
        "route": "L1.1 bounded lattice -> canonical L2 local field -> L3 sentence field -> DecisionCore",
        "elapsed_us": started.elapsed().as_micros() as u64,
        "selected": resolution.selected.as_ref().map(|candidate| candidate.replacement.as_str()),
        "decision": resolution.decision.as_ref().map(|decision| decision.replacement.as_str()),
        "scoreboard": {
            "total_candidates": resolution.scoreboard.total_candidates,
            "apply_candidates": resolution.scoreboard.apply_candidates,
            "suggest_only_candidates": resolution.scoreboard.suggest_only_candidates,
            "keep_original_candidates": resolution.scoreboard.keep_original_candidates,
            "veto_candidates": resolution.scoreboard.veto_candidates,
        },
        "candidates": candidates,
        "runtime_authority_changed": false,
    })
}

impl CorrectionScoreboard {
    pub(crate) fn from_candidates(
        candidates: &[UnifiedCorrectionCandidate],
        decision_batch: &CandidateDecisionBatch,
    ) -> Self {
        let selected = decision_batch
            .selected_index
            .and_then(|index| candidates.get(index));
        let mut scoreboard = Self {
            total_candidates: candidates.len(),
            ..Self::default()
        };

        for candidate in candidates {
            match candidate.gate.action {
                CandidateGateAction::Eligible if selected == Some(candidate) => {
                    scoreboard.apply_candidates += 1;
                }
                CandidateGateAction::Eligible => scoreboard.suggest_only_candidates += 1,
                CandidateGateAction::SuggestOnly => scoreboard.suggest_only_candidates += 1,
                CandidateGateAction::KeepOriginal => scoreboard.keep_original_candidates += 1,
                CandidateGateAction::Veto => scoreboard.veto_candidates += 1,
            }
            for evidence in &candidate.evidence {
                match evidence.source {
                    CorrectionDecisionSource::Deterministic => {
                        scoreboard.deterministic_candidates += 1;
                    }
                    CorrectionDecisionSource::Nanda => {
                        scoreboard.nanda_candidates += 1;
                    }
                }
            }
        }

        scoreboard.selected_bayes_posterior_milli = decision_batch
            .selected_index
            .and_then(|index| decision_batch.evaluations.get(index))
            .map(|evaluation| (evaluation.bayes.posterior * 1000.0).round() as i16);
        scoreboard
    }
}

fn record_correction_gate_stats(started: Instant, resolution: &CorrectionResolution) {
    let elapsed_us = started.elapsed().as_micros().min(u128::from(u64::MAX)) as u64;
    let stats = correction_gate_stats();
    stats.requests.fetch_add(1, Ordering::Relaxed);
    stats.total_candidates.fetch_add(
        resolution.scoreboard.total_candidates as u64,
        Ordering::Relaxed,
    );
    stats.apply_candidates.fetch_add(
        resolution.scoreboard.apply_candidates as u64,
        Ordering::Relaxed,
    );
    stats.suggest_only_candidates.fetch_add(
        resolution.scoreboard.suggest_only_candidates as u64,
        Ordering::Relaxed,
    );
    stats.keep_original_candidates.fetch_add(
        resolution.scoreboard.keep_original_candidates as u64,
        Ordering::Relaxed,
    );
    stats.veto_candidates.fetch_add(
        resolution.scoreboard.veto_candidates as u64,
        Ordering::Relaxed,
    );
    stats.deterministic_candidates.fetch_add(
        resolution.scoreboard.deterministic_candidates as u64,
        Ordering::Relaxed,
    );
    stats.nanda_candidates.fetch_add(
        resolution.scoreboard.nanda_candidates as u64,
        Ordering::Relaxed,
    );
    if resolution.decision.is_some() {
        stats.selected_apply.fetch_add(1, Ordering::Relaxed);
    }
    stats.total_us.fetch_add(elapsed_us, Ordering::Relaxed);
    update_max_atomic(&stats.max_us, elapsed_us);
}

fn correction_gate_runtime_stats() -> CorrectionGateRuntimeStats {
    let stats = correction_gate_stats();
    CorrectionGateRuntimeStats {
        requests: stats.requests.load(Ordering::Relaxed),
        total_candidates: stats.total_candidates.load(Ordering::Relaxed),
        apply_candidates: stats.apply_candidates.load(Ordering::Relaxed),
        suggest_only_candidates: stats.suggest_only_candidates.load(Ordering::Relaxed),
        keep_original_candidates: stats.keep_original_candidates.load(Ordering::Relaxed),
        veto_candidates: stats.veto_candidates.load(Ordering::Relaxed),
        deterministic_candidates: stats.deterministic_candidates.load(Ordering::Relaxed),
        nanda_candidates: stats.nanda_candidates.load(Ordering::Relaxed),
        selected_apply: stats.selected_apply.load(Ordering::Relaxed),
        total_us: stats.total_us.load(Ordering::Relaxed),
        max_us: stats.max_us.load(Ordering::Relaxed),
    }
}

fn correction_gate_stats() -> &'static CorrectionGateAtomicStats {
    static STATS: OnceLock<CorrectionGateAtomicStats> = OnceLock::new();
    STATS.get_or_init(CorrectionGateAtomicStats::default)
}

#[derive(Default)]
struct CorrectionGateAtomicStats {
    requests: AtomicU64,
    total_candidates: AtomicU64,
    apply_candidates: AtomicU64,
    suggest_only_candidates: AtomicU64,
    keep_original_candidates: AtomicU64,
    veto_candidates: AtomicU64,
    deterministic_candidates: AtomicU64,
    nanda_candidates: AtomicU64,
    selected_apply: AtomicU64,
    total_us: AtomicU64,
    max_us: AtomicU64,
}

fn update_max_atomic(target: &AtomicU64, value: u64) {
    let mut current = target.load(Ordering::Relaxed);
    while value > current {
        match target.compare_exchange_weak(current, value, Ordering::Relaxed, Ordering::Relaxed) {
            Ok(_) => break,
            Err(next) => current = next,
        }
    }
}

impl CorrectionCandidateScoreTrace {
    pub(crate) fn from_decision_batch(
        candidates: &[UnifiedCorrectionCandidate],
        batch: &CandidateDecisionBatch,
    ) -> Vec<Self> {
        candidates
            .iter()
            .zip(&batch.evaluations)
            .enumerate()
            .map(|(index, (candidate, evaluation))| {
                let score = &evaluation.bayes;
                let explanation = evaluation.explanation;
                let action = evaluation.action;
                let decision_signals = &evaluation.signals;
                Self {
                    replacement: candidate.replacement.clone(),
                    source: candidate.source,
                    source_id: candidate.source_id.clone(),
                    error_class: candidate.error_class,
                    action_operator: action.operator.as_str(),
                    action_proof: action.proof.as_str(),
                    edit_transition_operator: action.edit_operator.as_str(),
                    edit_transition_proof: action.edit_proof.as_str(),
                    edit_transition_operator_kind: action.edit_operator,
                    edit_transition_proof_kind: action.edit_proof.into(),
                    edit_transition_verified: action.verifier_passed,
                    edit_transition_left_context_changed: action.left_context_changed,
                    edit_transition_changed_tokens: action.changed_tokens,
                    edit_shape: explanation.edit_shape,
                    preservation_milli: explanation.preservation_milli,
                    lost_mass_milli: explanation.lost_mass_milli,
                    added_mass_milli: explanation.added_mass_milli,
                    operator_fit_milli: explanation.operator_fit_milli,
                    shortcut_risk_milli: explanation.shortcut_risk_milli,
                    anti_wave_milli: explanation.anti_wave_milli,
                    explanation_score_milli: explanation.explanation_score_milli,
                    gate_action: candidate.gate.action,
                    gate_reason: candidate.gate.reason,
                    likelihood_milli: crate::text_metrics::score_to_milli(score.likelihood),
                    usage_prior_milli: crate::text_metrics::score_to_milli(score.usage_prior),
                    context_prior_milli: crate::text_metrics::score_to_milli(score.context_prior),
                    transition_field_milli: decision_signals.transition_field_milli,
                    transition_field_attraction_milli: decision_signals
                        .transition_field_attraction_milli,
                    transition_field_repulsion_milli: decision_signals
                        .transition_field_repulsion_milli,
                    transition_field_uncertainty_milli: decision_signals
                        .transition_field_uncertainty_milli,
                    transition_field_phase_competition_milli: decision_signals
                        .transition_field_phase_competition_milli,
                    l2_wave_peak_milli: decision_signals.l2_wave_peak_milli,
                    l2_wave_peak_positive_milli: decision_signals.l2_wave_peak_positive_milli,
                    l2_wave_peak_negative_milli: decision_signals.l2_wave_peak_negative_milli,
                    l2_wave_peak_uncertainty_milli: decision_signals.l2_wave_peak_uncertainty_milli,
                    l2_wave_peak_reason: decision_signals.l2_wave_peak_reason,
                    l2_transition_phase_milli: decision_signals.l2_transition_phase_milli,
                    l2_transition_phase_threshold_milli: decision_signals
                        .l2_transition_phase_threshold_milli,
                    l2_transition_phase_verdict: decision_signals
                        .l2_transition_phase_verdict
                        .as_str(),
                    l2_transition_phase_package_loaded: decision_signals
                        .l2_transition_phase_package_loaded,
                    l2_transition_phase_operator_present: decision_signals
                        .l2_transition_phase_operator_present,
                    l2_transition_phase_operator_promoted: decision_signals
                        .l2_transition_phase_operator_promoted,
                    l2_transition_phase_positive_centers: decision_signals
                        .l2_transition_phase_positive_centers,
                    l2_transition_phase_anti_centers: decision_signals
                        .l2_transition_phase_anti_centers,
                    l2_transition_phase_surfaces: decision_signals.l2_transition_phase_surfaces,
                    l2_morphology_milli: decision_signals.l2_morphology_milli,
                    l2_morphology_disposition: decision_signals.l2_morphology_disposition,
                    l2_morphology_lemma_id: decision_signals.l2_morphology_lemma_id,
                    l2_morphology_source_feature_mask: decision_signals
                        .l2_morphology_source_feature_mask,
                    l2_morphology_target_feature_mask: decision_signals
                        .l2_morphology_target_feature_mask,
                    l2_morphology_context_posterior_milli: decision_signals
                        .l2_morphology_context_posterior_milli,
                    l2_morphology_slot_evidence_milli: decision_signals
                        .l2_morphology_slot_evidence_milli,
                    l2_morphology_joint_evidence_milli: decision_signals
                        .l2_morphology_joint_evidence_milli,
                    l2_morphology_competitors: decision_signals.l2_morphology_competitors,
                    l2_morphology_generated: decision_signals.l2_morphology_generated,
                    l3_phrase_milli: decision_signals.l3_phrase_milli,
                    l3_phrase_decision: decision_signals.l3_phrase_decision.as_str(),
                    l4_hidden_disposition: decision_signals.l4_hidden_disposition.as_str(),
                    l4_hidden_semantic_classes: decision_signals.l4_hidden_semantic_classes,
                    l4_hidden_unresolved_classes: decision_signals.l4_hidden_unresolved_classes,
                    l4_hidden_plan_commitment: decision_signals.l4_hidden_plan_commitment,
                    l4_hidden_receipts: decision_signals.l4_hidden_receipts,
                    l4_hidden_probe: decision_signals.l4_hidden_probe,
                    l4_hidden_certificate_valid: decision_signals.l4_hidden_certificate_valid,
                    l4_scene_milli: decision_signals.l4_scene_milli,
                    l4_scene_action: decision_signals.l4_scene_action.as_str(),
                    l4_scene_reason: decision_signals.l4_scene_reason,
                    l4_signed_milli: decision_signals.l4_signed_milli,
                    l4_signed_reason: decision_signals.l4_signed_reason,
                    l4_surface_status: decision_signals.l4_surface_status.as_str(),
                    l4_transition_state_specific: decision_signals.l4_transition_state_specific,
                    l4_transition_attract_count: decision_signals.l4_transition_attract_count,
                    l4_transition_repel_count: decision_signals.l4_transition_repel_count,
                    l4_phase_witness_milli: decision_signals.l4_phase_witness_milli,
                    l4_phase_witness_supported: decision_signals.l4_phase_witness_supported,
                    l4_phase_positive_centers: decision_signals.l4_phase_positive_centers,
                    l4_phase_negative_centers: decision_signals.l4_phase_negative_centers,
                    l4_cross_scene_package_loaded: decision_signals.l4_cross_scene_package_loaded,
                    l4_cross_scene_profile_present: decision_signals.l4_cross_scene_profile_present,
                    l4_cross_scene_disposition: decision_signals
                        .l4_cross_scene_disposition
                        .as_str(),
                    l4_cross_scene_recommendation: decision_signals
                        .l4_cross_scene_recommendation
                        .as_str(),
                    l4_cross_scene_margin_milli: decision_signals.l4_cross_scene_margin_milli,
                    l4_cross_scene_threshold_milli: decision_signals.l4_cross_scene_threshold_milli,
                    l4_cross_scene_pair_margin_milli: decision_signals
                        .l4_cross_scene_pair_margin_milli,
                    l4_cross_scene_automatic_apply: decision_signals.l4_cross_scene_automatic_apply,
                    risk_milli: crate::text_metrics::score_to_milli(score.risk),
                    posterior_milli: crate::text_metrics::score_to_milli(score.posterior),
                    decision_rank_milli: decision_signals.rank_milli,
                    selected: batch.selected_index == Some(index),
                }
            })
            .collect()
    }
}

pub(crate) fn explanation_for_candidate(
    original: &str,
    candidate: &UnifiedCorrectionCandidate,
) -> CandidateExplanation {
    explain_candidate(
        original,
        &candidate.replacement,
        candidate.error_class,
        candidate.origin,
    )
}

impl TypingErrorEvent {
    fn from_text(text: &str) -> Self {
        L1SurfaceSignal::from_text(text).into_event()
    }
}

#[cfg(test)]
mod target_evidence_adapter_tests {
    use super::*;
    use crate::typing_transition::target_evidence::{GroundingNamespaceV1, TargetRelationV1};

    #[test]
    fn correction_candidate_merges_producer_aliases_without_extra_votes() {
        let gate = CandidateGateDecision {
            action: CandidateGateAction::SuggestOnly,
            reason: "adapter-test",
        };
        let mut candidate = UnifiedCorrectionCandidate::new(
            "candidate",
            CorrectionDecisionSource::Nanda,
            CandidateOrigin::L2Surface,
            "l2",
            TypingErrorClass::MissingLetter,
            gate.clone(),
        );
        candidate.merge_evidence(UnifiedCorrectionCandidate::new(
            "candidate",
            CorrectionDecisionSource::Nanda,
            CandidateOrigin::L3Context,
            "l3",
            TypingErrorClass::MissingLetter,
            gate,
        ));

        let common = candidate.common_target_evidence();
        assert_eq!(common.logical_count(), 1);
        let [witness] = common.witnesses() else {
            panic!("producer aliases must retain one semantic witness");
        };
        assert_eq!(witness.relation, TargetRelationV1::MissingLetter);
        assert_eq!(
            witness.grounding_namespace,
            GroundingNamespaceV1::LegacyCorrectionCandidate
        );
        assert_eq!(witness.provenance_annotations, (1 << 4) | (1 << 5));
    }
}

include!("correction_core/candidate_sources.rs");
include!("correction_core/tests.rs");

//! Candidate proposal admission owned by the Typing Transition CPU.
//!
//! Producers supply a typed origin and a proposed surface transition. This
//! module classifies its maximum authority before the learned decision field
//! performs final selection.

use super::{action as action_operator, verifier as edit_transition};
use crate::candidate_contract::{CandidateOrigin, CorrectionSourceRole};
use crate::candidate_explanation::explain_candidate;
use crate::correction_core::TypingErrorClass;
use crate::russian_typo_candidates::{
    inserted_char_position_for_missing_letter, repeated_run_deletion_candidates,
};
use crate::text_metrics::{damerau_levenshtein, has_cyrillic};
use crate::word_reader::{
    is_cyrillic_letters_only, last_text_word, split_edge_whitespace, split_word_punctuation,
};
const REPEATED_DELETE_SURFACE_MARGIN: f64 = 0.25;

#[cfg(test)]
const ADMISSION_TRACE_ENV: &str = "LAY_PROPOSAL_ADMISSION_TRACE";
#[cfg(test)]
const ADMISSION_TRACE_SCHEMA: &str = "v10-admission-substage-v1";
#[cfg(test)]
const ADMISSION_TRACE_STAGE_COUNT: usize = 36;
#[cfg(test)]
const ADMISSION_TRACE_STAGE_NAMES: [&str; ADMISSION_TRACE_STAGE_COUNT] = [
    "unchanged",
    "explain_candidate",
    "replacement_glues_separate_words",
    "boundary_glues_short_function_tail",
    "boundary_eats_known_current_word",
    "boundary_changes_non_whitespace_surface",
    "multiword_last_vowel_completion",
    "adjacent_transposition_boundary_competition",
    "boundary_splits_known_word",
    "boundary_splits_weak_tail",
    "reflexive_suffix_requires_grammar",
    "known_current_surface_drift",
    "verify_action_operator",
    "surface_changes_left_context",
    "l2_surface_stem_truncation",
    "structural_over_compress",
    "structural_function_prefix_drop",
    "structural_phrase_part_growth",
    "structural_short_initial_growth",
    "structural_short_case_vowel",
    "structural_soft_sign_vowel",
    "structural_short_internal_consonant",
    "structural_short_same_length_multi_edit",
    "structural_same_tail_consonant",
    "structural_infinitive_overreach",
    "structural_protected_context_authority",
    "structural_known_word_different_known",
    "structural_short_layout_context",
    "structural_short_cyrillic_ascii",
    "structural_short_nanda_shrink",
    "structural_short_nanda_internal_vowel",
    "structural_nanda_unknown_word",
    "unproven_stable_surface_shape",
    "semantic_surface_authority",
    "completion_only",
    "final_class_dispatch",
];

#[cfg(test)]
const ADMISSION_TRACE_REASON_NAMES: [&str; 43] = [
    "unchanged",
    "unexplained_signal_loss",
    "word_count_shrink_requires_boundary_class",
    "unsafe_boundary_glue_short_function_tail",
    "moved_prefix_eats_known_current_word",
    "boundary_operator_changes_surface",
    "unsafe_multi_word_vowel_completion",
    "single_letter_boundary_beats_transposition",
    "known_single_word_boundary_split",
    "weak_boundary_split_tail",
    "reflexive_suffix_requires_grammar_proof",
    "known_current_word_surface_drift",
    "edit_transition_not_verified",
    "surface_left_context_apply_blocked",
    "l2_surface_stem_truncation_low",
    "candidate_over_compresses_word",
    "function_prefix_letter_drop",
    "known_phrase_part_one_letter_growth",
    "short_initial_letter_growth",
    "short_case_vowel_drift",
    "soft_sign_vowel_drift",
    "short_internal_consonant_drift",
    "short_same_length_multi_edit_drift",
    "same_tail_single_consonant_drift",
    "known_form_to_infinitive_overreach",
    "protected_current_surface_rewrite_requires_context_authority",
    "known_word_to_different_known_word",
    "short_layout_without_phrase_context",
    "short_cyrillic_to_ascii_layout",
    "short_nanda_word_shrink",
    "short_nanda_internal_vowel_growth",
    "nanda_surface_unknown_word",
    "unproven_stable_surface_shape_drift",
    "semantic_wave_surface_authority_low",
    "completion_is_not_autocorrect",
    "protected_or_technical",
    "single_step_typo_still_unknown",
    "unknown_error_class",
    "class_allows_apply",
    "productive_v90_lattice_requires_common_l3",
    "productive_v90_lattice_abstained",
    "productive_v90_lattice_unavailable",
    "productive_v90_non_winner_requires_common_l3",
];

#[cfg(test)]
#[repr(usize)]
#[derive(Clone, Copy)]
enum AdmissionTraceStage {
    Unchanged,
    ExplainCandidate,
    ReplacementGluesSeparateWords,
    BoundaryGluesShortFunctionTail,
    BoundaryEatsKnownCurrentWord,
    BoundaryChangesNonWhitespaceSurface,
    MultiwordLastVowelCompletion,
    AdjacentTranspositionBoundaryCompetition,
    BoundarySplitsKnownWord,
    BoundarySplitsWeakTail,
    ReflexiveSuffixRequiresGrammar,
    KnownCurrentSurfaceDrift,
    VerifyActionOperator,
    SurfaceChangesLeftContext,
    L2SurfaceStemTruncation,
    StructuralOverCompress,
    StructuralFunctionPrefixDrop,
    StructuralPhrasePartGrowth,
    StructuralShortInitialGrowth,
    StructuralShortCaseVowel,
    StructuralSoftSignVowel,
    StructuralShortInternalConsonant,
    StructuralShortSameLengthMultiEdit,
    StructuralSameTailConsonant,
    StructuralInfinitiveOverreach,
    StructuralProtectedContextAuthority,
    StructuralKnownWordDifferentKnown,
    StructuralShortLayoutContext,
    StructuralShortCyrillicAscii,
    StructuralShortNandaShrink,
    StructuralShortNandaInternalVowel,
    StructuralNandaUnknownWord,
    UnprovenStableSurfaceShape,
    SemanticSurfaceAuthority,
    CompletionOnly,
    FinalClassDispatch,
}

#[cfg(test)]
struct AdmissionTraceCounters {
    stage_calls: [u64; ADMISSION_TRACE_STAGE_COUNT],
    stage_hits: [u64; ADMISSION_TRACE_STAGE_COUNT],
    stage_elapsed_ns: [u64; ADMISSION_TRACE_STAGE_COUNT],
    admission_calls: u64,
    admission_elapsed_ns: u64,
    post_override_calls: u64,
    post_override_hits: u64,
    post_override_elapsed_ns: u64,
    action_counts: [u64; 4],
    reason_counts: [u64; ADMISSION_TRACE_REASON_NAMES.len()],
    unknown_reasons: u64,
}

#[cfg(test)]
impl Default for AdmissionTraceCounters {
    fn default() -> Self {
        Self {
            stage_calls: [0; ADMISSION_TRACE_STAGE_COUNT],
            stage_hits: [0; ADMISSION_TRACE_STAGE_COUNT],
            stage_elapsed_ns: [0; ADMISSION_TRACE_STAGE_COUNT],
            admission_calls: 0,
            admission_elapsed_ns: 0,
            post_override_calls: 0,
            post_override_hits: 0,
            post_override_elapsed_ns: 0,
            action_counts: [0; 4],
            reason_counts: [0; ADMISSION_TRACE_REASON_NAMES.len()],
            unknown_reasons: 0,
        }
    }
}

#[cfg(test)]
std::thread_local! {
    static ADMISSION_TRACE: std::cell::RefCell<Option<AdmissionTraceCounters>> =
        const { std::cell::RefCell::new(None) };
}

#[cfg(test)]
fn duration_ns(duration: std::time::Duration) -> u64 {
    u64::try_from(duration.as_nanos()).unwrap_or(u64::MAX)
}

#[cfg(test)]
fn admission_trace_active() -> bool {
    ADMISSION_TRACE.with(|trace| trace.borrow().is_some())
}

#[cfg(test)]
fn trace_stage_value<T>(
    stage: AdmissionTraceStage,
    observe: impl FnOnce() -> T,
    hit: impl FnOnce(&T) -> bool,
) -> T {
    if !admission_trace_active() {
        return observe();
    }
    let started = std::time::Instant::now();
    let value = observe();
    let elapsed_ns = duration_ns(started.elapsed());
    let hit = hit(&value);
    ADMISSION_TRACE.with(|trace| {
        if let Some(trace) = trace.borrow_mut().as_mut() {
            let index = stage as usize;
            trace.stage_calls[index] = trace.stage_calls[index].saturating_add(1);
            trace.stage_hits[index] = trace.stage_hits[index].saturating_add(u64::from(hit));
            trace.stage_elapsed_ns[index] =
                trace.stage_elapsed_ns[index].saturating_add(elapsed_ns);
        }
    });
    value
}

#[cfg(test)]
fn trace_stage_bool(stage: AdmissionTraceStage, observe: impl FnOnce() -> bool) -> bool {
    trace_stage_value(stage, observe, |value| *value)
}

#[cfg(test)]
fn record_admission(elapsed: std::time::Duration) {
    ADMISSION_TRACE.with(|trace| {
        if let Some(trace) = trace.borrow_mut().as_mut() {
            trace.admission_calls = trace.admission_calls.saturating_add(1);
            trace.admission_elapsed_ns = trace
                .admission_elapsed_ns
                .saturating_add(duration_ns(elapsed));
        }
    });
}

#[cfg(test)]
pub(crate) fn record_live_authority_override(
    elapsed: std::time::Duration,
    override_hit: bool,
    decision: &CandidateGateDecision,
) {
    ADMISSION_TRACE.with(|trace| {
        let mut trace = trace.borrow_mut();
        let Some(trace) = trace.as_mut() else {
            return;
        };
        trace.post_override_calls = trace.post_override_calls.saturating_add(1);
        trace.post_override_hits = trace
            .post_override_hits
            .saturating_add(u64::from(override_hit));
        trace.post_override_elapsed_ns = trace
            .post_override_elapsed_ns
            .saturating_add(duration_ns(elapsed));
        let action_index = match decision.action {
            CandidateGateAction::Eligible => 0,
            CandidateGateAction::SuggestOnly => 1,
            CandidateGateAction::KeepOriginal => 2,
            CandidateGateAction::Veto => 3,
        };
        trace.action_counts[action_index] = trace.action_counts[action_index].saturating_add(1);
        if let Some(reason_index) = ADMISSION_TRACE_REASON_NAMES
            .iter()
            .position(|reason| *reason == decision.reason)
        {
            trace.reason_counts[reason_index] = trace.reason_counts[reason_index].saturating_add(1);
        } else {
            trace.unknown_reasons = trace.unknown_reasons.saturating_add(1);
        }
    });
}

#[cfg(test)]
pub(crate) struct AdmissionTraceSession {
    active: bool,
    finished: bool,
}

#[cfg(test)]
impl AdmissionTraceSession {
    pub(crate) fn post_override_started(&self) -> Option<std::time::Instant> {
        self.active.then(std::time::Instant::now)
    }

    pub(crate) fn finish_line(
        mut self,
        surfaces: usize,
        emitted: usize,
    ) -> Result<Option<String>, String> {
        if !self.active {
            self.finished = true;
            return Ok(None);
        }
        let trace = ADMISSION_TRACE.with(|trace| trace.borrow_mut().take());
        self.finished = true;
        let trace = trace.ok_or_else(|| "V10 admission trace session disappeared".to_string())?;
        if trace.unknown_reasons != 0 {
            return Err(format!(
                "V10 admission trace observed {} unknown final reasons",
                trace.unknown_reasons
            ));
        }
        let emitted = u64::try_from(emitted)
            .map_err(|_| "V10 emitted candidate count does not fit u64".to_string())?;
        let action_total = trace.action_counts.iter().copied().sum::<u64>();
        let reason_total = trace.reason_counts.iter().copied().sum::<u64>();
        if trace.admission_calls != emitted
            || trace.post_override_calls != emitted
            || action_total != emitted
            || reason_total != emitted
        {
            return Err(format!(
                "V10 admission trace cardinality mismatch: emitted={emitted} admission={} post={} actions={action_total} reasons={reason_total}",
                trace.admission_calls, trace.post_override_calls
            ));
        }
        let leaf_elapsed_ns = trace.stage_elapsed_ns.iter().copied().sum::<u64>();
        let residual_ns = trace.admission_elapsed_ns.saturating_sub(leaf_elapsed_ns);
        let mut line = format!(
            "proposal_admission_substage_trace schema={ADMISSION_TRACE_SCHEMA} surfaces={surfaces} emitted={emitted} admission_calls={} admission_ns={} leaf_ns={leaf_elapsed_ns} residual_ns={residual_ns} post_calls={} post_hits={} post_ns={} stages=",
            trace.admission_calls,
            trace.admission_elapsed_ns,
            trace.post_override_calls,
            trace.post_override_hits,
            trace.post_override_elapsed_ns,
        );
        use std::fmt::Write as _;
        for (index, name) in ADMISSION_TRACE_STAGE_NAMES.iter().enumerate() {
            if index != 0 {
                line.push(',');
            }
            write!(
                line,
                "{name}:{}:{}:{}",
                trace.stage_calls[index], trace.stage_hits[index], trace.stage_elapsed_ns[index]
            )
            .expect("writing to String cannot fail");
        }
        line.push_str(" actions=");
        for (index, name) in ["eligible", "suggest_only", "keep_original", "veto"]
            .iter()
            .enumerate()
        {
            if index != 0 {
                line.push(',');
            }
            write!(line, "{name}:{}", trace.action_counts[index])
                .expect("writing to String cannot fail");
        }
        line.push_str(" reasons=");
        for (index, name) in ADMISSION_TRACE_REASON_NAMES.iter().enumerate() {
            if index != 0 {
                line.push(',');
            }
            write!(line, "{name}:{}", trace.reason_counts[index])
                .expect("writing to String cannot fail");
        }
        write!(line, " unknown_reasons={}", trace.unknown_reasons)
            .expect("writing to String cannot fail");
        Ok(Some(line))
    }
}

#[cfg(test)]
impl Drop for AdmissionTraceSession {
    fn drop(&mut self) {
        if self.active && !self.finished {
            ADMISSION_TRACE.with(|trace| {
                trace.borrow_mut().take();
            });
        }
    }
}

#[cfg(test)]
pub(crate) fn begin_admission_trace_session() -> Result<AdmissionTraceSession, String> {
    let active = std::env::var(ADMISSION_TRACE_ENV).as_deref() == Ok("1");
    if active {
        ADMISSION_TRACE.with(|trace| {
            let mut trace = trace.borrow_mut();
            if trace.is_some() {
                return Err("V10 admission trace session is already active".to_string());
            }
            *trace = Some(AdmissionTraceCounters::default());
            Ok(())
        })?;
    }
    Ok(AdmissionTraceSession {
        active,
        finished: false,
    })
}

#[cfg(test)]
macro_rules! admission_trace_bool {
    ($stage:ident, $expression:expr) => {
        trace_stage_bool(AdmissionTraceStage::$stage, || $expression)
    };
}

#[cfg(not(test))]
macro_rules! admission_trace_bool {
    ($stage:ident, $expression:expr) => {
        $expression
    };
}

#[cfg(test)]
macro_rules! admission_trace_value {
    ($stage:ident, $expression:expr, $hit:expr) => {
        trace_stage_value(AdmissionTraceStage::$stage, || $expression, $hit)
    };
}

#[cfg(not(test))]
macro_rules! admission_trace_value {
    ($stage:ident, $expression:expr, $hit:expr) => {
        $expression
    };
}

#[cfg(test)]
const ADMISSION_FACT_REUSE_ENV: &str = "LAY_PROPOSAL_ADMISSION_FACT_REUSE";

#[cfg(test)]
static ADMISSION_FACT_REUSE_FROM_ENV: std::sync::OnceLock<bool> = std::sync::OnceLock::new();

#[cfg(test)]
std::thread_local! {
    static ADMISSION_FACT_REUSE_OVERRIDE: std::cell::Cell<Option<bool>> =
        const { std::cell::Cell::new(None) };
}

#[cfg(test)]
fn configured_admission_fact_reuse() -> bool {
    ADMISSION_FACT_REUSE_OVERRIDE
        .with(std::cell::Cell::get)
        .unwrap_or_else(|| {
            *ADMISSION_FACT_REUSE_FROM_ENV.get_or_init(|| {
                match std::env::var(ADMISSION_FACT_REUSE_ENV) {
                    Ok(value) if value == "REUSE" => true,
                    Ok(value) if value == "UNCACHED" => false,
                    Err(std::env::VarError::NotPresent) => false,
                    Ok(value) => panic!(
                        "{ADMISSION_FACT_REUSE_ENV} must be UNCACHED or REUSE, got {value:?}"
                    ),
                    Err(std::env::VarError::NotUnicode(_)) => {
                        panic!("{ADMISSION_FACT_REUSE_ENV} must be valid UTF-8")
                    }
                }
            })
        })
}

#[cfg(test)]
struct AdmissionFactReuseOverride {
    previous: Option<bool>,
}

#[cfg(test)]
impl Drop for AdmissionFactReuseOverride {
    fn drop(&mut self) {
        ADMISSION_FACT_REUSE_OVERRIDE.with(|slot| slot.set(self.previous));
    }
}

#[cfg(test)]
fn with_admission_fact_reuse<T>(reuse: bool, run: impl FnOnce() -> T) -> T {
    let previous = ADMISSION_FACT_REUSE_OVERRIDE.with(|slot| {
        let previous = slot.get();
        slot.set(Some(reuse));
        previous
    });
    let _guard = AdmissionFactReuseOverride { previous };
    run()
}

#[derive(Debug)]
struct AdmissionWordFacts {
    word: String,
    lower: std::cell::OnceCell<String>,
    cyrillic_letters_only: std::cell::OnceCell<bool>,
    char_len: std::cell::OnceCell<usize>,
    known_russian: std::cell::OnceCell<bool>,
    protected_current: std::cell::OnceCell<bool>,
}

impl AdmissionWordFacts {
    fn new(word: String) -> Self {
        Self {
            word,
            lower: std::cell::OnceCell::new(),
            cyrillic_letters_only: std::cell::OnceCell::new(),
            char_len: std::cell::OnceCell::new(),
            known_russian: std::cell::OnceCell::new(),
            protected_current: std::cell::OnceCell::new(),
        }
    }

    fn word(&self) -> &str {
        &self.word
    }

    fn lower(&self) -> &str {
        self.lower.get_or_init(|| self.word.to_lowercase())
    }

    fn is_cyrillic_letters_only(&self) -> bool {
        *self
            .cyrillic_letters_only
            .get_or_init(|| is_cyrillic_letters_only(&self.word))
    }

    fn char_len(&self) -> usize {
        *self.char_len.get_or_init(|| self.lower().chars().count())
    }

    fn is_known_russian(&self) -> bool {
        *self
            .known_russian
            .get_or_init(|| known_russian_autocorrect_token(self.lower()))
    }

    fn is_protected_current(&self) -> bool {
        *self.protected_current.get_or_init(|| {
            let field = crate::hot_field::HotFieldSnapshot::current();
            self.is_known_russian()
                || crate::phrase_lexicon::is_known_russian_phrase_part(self.lower())
                || field
                    .input_surface_readout(self.lower())
                    .has_phase_authority()
                || field.word_readout(self.lower()).is_known()
                || crate::nanda_wave::l2::l2_surface_foundation_contains(self.lower())
                || crate::nanda_wave::l2::l2_surface_foundation_has_authority(self.lower())
                || crate::russian_lexicon::is_reference_backed_russian_form(self.lower())
                || crate::russian_lexicon::is_center_backed_russian_form(self.lower())
                || crate::russian_lexicon::is_reference_known_russian_word_or_form(self.lower())
                || crate::russian_lexicon::russian_dictionary().contains(self.lower())
                || crate::russian_lexicon::russian_short_dictionary().contains(self.lower())
        })
    }
}

#[cfg(test)]
#[derive(Debug, Default, PartialEq, Eq)]
struct AdmissionLexicalFactSnapshot {
    original_word: bool,
    replacement_word: bool,
    original_lower: bool,
    replacement_lower: bool,
    original_known: bool,
    replacement_known: bool,
    original_protected: bool,
    replacement_protected: bool,
}

struct AdmissionLexicalFacts<'a> {
    original: &'a str,
    replacement: &'a str,
    reuse: bool,
    original_word: std::cell::OnceCell<Option<AdmissionWordFacts>>,
    replacement_word: std::cell::OnceCell<Option<AdmissionWordFacts>>,
}

impl<'a> AdmissionLexicalFacts<'a> {
    fn new(original: &'a str, replacement: &'a str) -> Self {
        #[cfg(test)]
        let reuse = configured_admission_fact_reuse();
        #[cfg(not(test))]
        let reuse = true;

        Self::with_mode(original, replacement, reuse)
    }

    fn with_mode(original: &'a str, replacement: &'a str, reuse: bool) -> Self {
        Self {
            original,
            replacement,
            reuse,
            original_word: std::cell::OnceCell::new(),
            replacement_word: std::cell::OnceCell::new(),
        }
    }

    fn reuses_facts(&self) -> bool {
        self.reuse
    }

    fn assert_pair(&self, original: &str, replacement: &str) {
        debug_assert_eq!(
            self.original, original,
            "lexical fact original identity drift"
        );
        debug_assert_eq!(
            self.replacement, replacement,
            "lexical fact replacement identity drift"
        );
    }

    fn original_word(&self) -> Option<&AdmissionWordFacts> {
        self.original_word
            .get_or_init(|| last_text_word(self.original).map(AdmissionWordFacts::new))
            .as_ref()
    }

    fn replacement_word(&self) -> Option<&AdmissionWordFacts> {
        self.replacement_word
            .get_or_init(|| last_text_word(self.replacement).map(AdmissionWordFacts::new))
            .as_ref()
    }

    #[cfg(test)]
    fn snapshot(&self) -> AdmissionLexicalFactSnapshot {
        let original = self.original_word.get().and_then(Option::as_ref);
        let replacement = self.replacement_word.get().and_then(Option::as_ref);
        AdmissionLexicalFactSnapshot {
            original_word: self.original_word.get().is_some(),
            replacement_word: self.replacement_word.get().is_some(),
            original_lower: original.is_some_and(|word| word.lower.get().is_some()),
            replacement_lower: replacement.is_some_and(|word| word.lower.get().is_some()),
            original_known: original.is_some_and(|word| word.known_russian.get().is_some()),
            replacement_known: replacement.is_some_and(|word| word.known_russian.get().is_some()),
            original_protected: original.is_some_and(|word| word.protected_current.get().is_some()),
            replacement_protected: replacement
                .is_some_and(|word| word.protected_current.get().is_some()),
        }
    }
}

macro_rules! admission_fact_call {
    ($facts:expr, $cached:ident, $uncached:ident $(, $arg:expr)* $(,)?) => {
        $cached($($arg,)* $facts)
    };
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CandidateGateAction {
    /// Producer supplied evidence and no structural constraint blocked it. Only
    /// TransitionDecisionCore may turn this into a physical Apply.
    Eligible,
    SuggestOnly,
    KeepOriginal,
    Veto,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CandidateGateDecision {
    pub action: CandidateGateAction,
    pub reason: &'static str,
}

#[cfg(test)]
pub(crate) fn gate_candidate(
    original: &str,
    replacement: &str,
    error_class: TypingErrorClass,
) -> CandidateGateDecision {
    gate_candidate_with_origin(
        original,
        replacement,
        error_class,
        CandidateOrigin::DeterministicTypo,
    )
}

/// Compatibility adapter for legacy fixtures. Runtime producers must supply a
/// typed origin and cannot route authority through a diagnostic source name.
#[cfg(test)]
pub(crate) fn gate_candidate_with_source(
    original: &str,
    replacement: &str,
    error_class: TypingErrorClass,
    source_id: &str,
) -> CandidateGateDecision {
    gate_candidate_with_origin(
        original,
        replacement,
        error_class,
        fixture_origin(source_id),
    )
}

#[cfg(test)]
fn fixture_origin(source_id: &str) -> CandidateOrigin {
    if source_id.starts_with("layout_then_") {
        CandidateOrigin::LayoutThenTypo
    } else if source_id.contains("layout")
        || matches!(source_id, "LayoutWordCell32" | "ShortTokenCell32")
    {
        CandidateOrigin::Layout
    } else if matches!(
        source_id,
        "BoundaryCell32" | "BoundaryShiftCell32" | "layout_phrase"
    ) {
        CandidateOrigin::Boundary
    } else if matches!(
        source_id,
        "PhraseForecastCell32" | "L2SurfaceCompletionCell32"
    ) {
        CandidateOrigin::Completion
    } else if matches!(source_id, "L2SurfaceMotifCell32" | "L2WordAttractorCell32") {
        CandidateOrigin::L2Surface
    } else if matches!(
        source_id,
        "PhraseCell32" | "PhraseMemoryCell32" | "SemanticWordCell32"
    ) {
        CandidateOrigin::L3Context
    } else if source_id == "TechTokenCell32" {
        CandidateOrigin::Technical
    } else {
        CandidateOrigin::DeterministicTypo
    }
}

pub(crate) fn gate_candidate_with_origin(
    original: &str,
    replacement: &str,
    error_class: TypingErrorClass,
    origin: CandidateOrigin,
) -> CandidateGateDecision {
    #[cfg(test)]
    if admission_trace_active() {
        let started = std::time::Instant::now();
        let decision = candidate_admission(original, replacement, error_class, origin);
        record_admission(started.elapsed());
        return decision;
    }
    candidate_admission(original, replacement, error_class, origin)
}

fn candidate_admission(
    original: &str,
    replacement: &str,
    error_class: TypingErrorClass,
    origin: CandidateOrigin,
) -> CandidateGateDecision {
    let lexical_facts = AdmissionLexicalFacts::new(original, replacement);
    candidate_admission_with_facts(original, replacement, error_class, origin, &lexical_facts)
}

fn candidate_admission_with_facts(
    original: &str,
    replacement: &str,
    error_class: TypingErrorClass,
    origin: CandidateOrigin,
    lexical_facts: &AdmissionLexicalFacts<'_>,
) -> CandidateGateDecision {
    if admission_trace_bool!(Unchanged, original == replacement) {
        return CandidateGateDecision {
            action: CandidateGateAction::KeepOriginal,
            reason: "unchanged",
        };
    }
    let (explanation, explanation_blocks_apply) = admission_trace_value!(
        ExplainCandidate,
        {
            let explanation = explain_candidate(original, replacement, error_class, origin);
            let blocks_apply = explanation.blocks_apply();
            (explanation, blocks_apply)
        },
        |value: &(crate::candidate_explanation::CandidateExplanation, bool)| value.1
    );
    if explanation_blocks_apply {
        return CandidateGateDecision {
            action: CandidateGateAction::SuggestOnly,
            reason: "unexplained_signal_loss",
        };
    }
    drop(explanation);
    if admission_trace_bool!(
        ReplacementGluesSeparateWords,
        replacement_glues_separate_words_without_boundary_class(original, replacement, error_class)
    ) {
        return CandidateGateDecision {
            action: CandidateGateAction::SuggestOnly,
            reason: "word_count_shrink_requires_boundary_class",
        };
    }
    if admission_trace_bool!(
        BoundaryGluesShortFunctionTail,
        boundary_candidate_glues_short_function_tail(original, replacement, error_class)
    ) {
        return CandidateGateDecision {
            action: CandidateGateAction::SuggestOnly,
            reason: "unsafe_boundary_glue_short_function_tail",
        };
    }
    if admission_trace_bool!(
        BoundaryEatsKnownCurrentWord,
        boundary_candidate_eats_known_current_word(original, replacement, error_class, origin)
    ) {
        return CandidateGateDecision {
            action: CandidateGateAction::SuggestOnly,
            reason: "moved_prefix_eats_known_current_word",
        };
    }
    if admission_trace_bool!(
        BoundaryChangesNonWhitespaceSurface,
        boundary_operator_changes_non_whitespace_surface(
            original,
            replacement,
            error_class,
            origin
        )
    ) {
        return CandidateGateDecision {
            action: CandidateGateAction::SuggestOnly,
            reason: "boundary_operator_changes_surface",
        };
    }
    if admission_trace_bool!(
        MultiwordLastVowelCompletion,
        multi_word_candidate_only_completes_last_vowel(original, replacement, error_class)
    ) {
        return CandidateGateDecision {
            action: CandidateGateAction::SuggestOnly,
            reason: "unsafe_multi_word_vowel_completion",
        };
    }
    if admission_trace_bool!(
        AdjacentTranspositionBoundaryCompetition,
        adjacent_transposition_competes_with_single_letter_boundary(
            original,
            replacement,
            error_class,
        )
    ) {
        return CandidateGateDecision {
            action: CandidateGateAction::SuggestOnly,
            reason: "single_letter_boundary_beats_transposition",
        };
    }
    if admission_trace_bool!(
        BoundarySplitsKnownWord,
        boundary_candidate_splits_known_russian_word(original, replacement, error_class)
    ) {
        return CandidateGateDecision {
            action: CandidateGateAction::SuggestOnly,
            reason: "known_single_word_boundary_split",
        };
    }
    if admission_trace_bool!(
        BoundarySplitsWeakTail,
        boundary_candidate_splits_to_short_function_and_weak_tail(
            original,
            replacement,
            error_class,
        )
    ) {
        return CandidateGateDecision {
            action: CandidateGateAction::SuggestOnly,
            reason: "weak_boundary_split_tail",
        };
    }
    if admission_trace_bool!(
        ReflexiveSuffixRequiresGrammar,
        reflexive_suffix_candidate_requires_grammar_proof(original, replacement, error_class)
    ) {
        return CandidateGateDecision {
            action: CandidateGateAction::SuggestOnly,
            reason: "reflexive_suffix_requires_grammar_proof",
        };
    }
    if admission_trace_bool!(
        KnownCurrentSurfaceDrift,
        admission_fact_call!(
            lexical_facts,
            known_current_word_gets_unproven_surface_drift_with_facts,
            known_current_word_gets_unproven_surface_drift,
            original,
            replacement,
            error_class,
            origin,
        )
    ) {
        return CandidateGateDecision {
            action: CandidateGateAction::SuggestOnly,
            reason: "known_current_word_surface_drift",
        };
    }
    let action_blocker = admission_trace_value!(
        VerifyActionOperator,
        action_operator::verify_action_operator(original, replacement, error_class, origin)
            .apply_blocker(),
        |value: &Option<&'static str>| value.is_some()
    );
    if let Some(reason) = action_blocker {
        return CandidateGateDecision {
            action: CandidateGateAction::SuggestOnly,
            reason,
        };
    }
    if admission_trace_bool!(
        SurfaceChangesLeftContext,
        surface_candidate_changes_left_context(original, replacement, origin)
    ) {
        return CandidateGateDecision {
            action: CandidateGateAction::SuggestOnly,
            reason: "surface_left_context_apply_blocked",
        };
    }
    if admission_trace_bool!(
        L2SurfaceStemTruncation,
        l2_surface_candidate_truncates_to_stem_without_deletion_proof(
            original,
            replacement,
            origin,
        )
    ) {
        return CandidateGateDecision {
            action: CandidateGateAction::SuggestOnly,
            reason: "l2_surface_stem_truncation_low",
        };
    }
    if let Some(decision) =
        structural_context_gate(original, replacement, error_class, origin, lexical_facts)
    {
        return decision;
    }
    if admission_trace_bool!(
        UnprovenStableSurfaceShape,
        admission_fact_call!(
            lexical_facts,
            unproven_stable_surface_shape_drift_with_facts,
            unproven_stable_surface_shape_drift,
            original,
            replacement,
            error_class,
            origin,
        )
    ) {
        return CandidateGateDecision {
            action: CandidateGateAction::SuggestOnly,
            reason: "unproven_stable_surface_shape_drift",
        };
    }
    if admission_trace_bool!(
        SemanticSurfaceAuthority,
        semantic_candidate_lacks_surface_authority(original, replacement, origin)
    ) {
        return CandidateGateDecision {
            action: CandidateGateAction::SuggestOnly,
            reason: "semantic_wave_surface_authority_low",
        };
    }
    if admission_trace_bool!(
        CompletionOnly,
        error_class == TypingErrorClass::CompletionOnly
    ) {
        return CandidateGateDecision {
            action: CandidateGateAction::SuggestOnly,
            reason: "completion_is_not_autocorrect",
        };
    }
    admission_trace_value!(
        FinalClassDispatch,
        match error_class {
            TypingErrorClass::TechnicalToken | TypingErrorClass::ProtectedToken => {
                CandidateGateDecision {
                    action: CandidateGateAction::Veto,
                    reason: "protected_or_technical",
                }
            }
            TypingErrorClass::RepeatedLetter | TypingErrorClass::ExtraLetter
                if replacement_last_word_is_unknown_cyrillic(original, replacement) =>
            {
                CandidateGateDecision {
                    action: CandidateGateAction::SuggestOnly,
                    reason: "single_step_typo_still_unknown",
                }
            }
            TypingErrorClass::Unknown => CandidateGateDecision {
                action: CandidateGateAction::SuggestOnly,
                reason: "unknown_error_class",
            },
            _ => CandidateGateDecision {
                action: CandidateGateAction::Eligible,
                reason: "class_allows_apply",
            },
        },
        |_value: &CandidateGateDecision| true
    )
}

fn surface_candidate_changes_left_context(
    original: &str,
    replacement: &str,
    origin: CandidateOrigin,
) -> bool {
    let source_may_only_fix_current_word = origin == CandidateOrigin::L2Surface;
    source_may_only_fix_current_word && candidate_changes_non_last_word(original, replacement)
}

fn candidate_changes_non_last_word(original: &str, replacement: &str) -> bool {
    let original_words = normalized_correction_words(original);
    let replacement_words = normalized_correction_words(replacement);
    if original_words.len() != replacement_words.len() {
        return original_words.len() > 1 || replacement_words.len() > 1;
    }
    if original_words.len() <= 1 {
        return false;
    }
    original_words[..original_words.len() - 1] != replacement_words[..replacement_words.len() - 1]
}

pub(crate) fn normalized_correction_words(text: &str) -> Vec<String> {
    crate::word_reader::normalized_text_words(text)
}

fn structural_context_gate(
    original: &str,
    replacement: &str,
    error_class: TypingErrorClass,
    origin: CandidateOrigin,
    lexical_facts: &AdmissionLexicalFacts<'_>,
) -> Option<CandidateGateDecision> {
    if admission_trace_bool!(
        StructuralOverCompress,
        candidate_over_compresses_word(original, replacement, error_class)
    ) {
        return Some(CandidateGateDecision {
            action: CandidateGateAction::KeepOriginal,
            reason: "candidate_over_compresses_word",
        });
    }
    if admission_trace_bool!(
        StructuralFunctionPrefixDrop,
        candidate_drops_letter_after_one_letter_function_prefix(original, replacement, error_class,)
    ) {
        return Some(CandidateGateDecision {
            action: CandidateGateAction::KeepOriginal,
            reason: "function_prefix_letter_drop",
        });
    }
    if admission_trace_bool!(
        StructuralPhrasePartGrowth,
        known_phrase_part_only_grows_by_one_letter(original, replacement, error_class)
    ) {
        return Some(CandidateGateDecision {
            action: CandidateGateAction::SuggestOnly,
            reason: "known_phrase_part_one_letter_growth",
        });
    }
    if admission_trace_bool!(
        StructuralShortInitialGrowth,
        short_word_only_grows_initial_letter(original, replacement, error_class)
    ) {
        return Some(CandidateGateDecision {
            action: CandidateGateAction::SuggestOnly,
            reason: "short_initial_letter_growth",
        });
    }
    if admission_trace_bool!(
        StructuralShortCaseVowel,
        short_word_gets_case_vowel_drift(original, replacement, error_class)
    ) {
        return Some(CandidateGateDecision {
            action: CandidateGateAction::SuggestOnly,
            reason: "short_case_vowel_drift",
        });
    }
    if admission_trace_bool!(
        StructuralSoftSignVowel,
        soft_sign_word_gets_vowel_drift(original, replacement, error_class)
    ) {
        return Some(CandidateGateDecision {
            action: CandidateGateAction::SuggestOnly,
            reason: "soft_sign_vowel_drift",
        });
    }
    if admission_trace_bool!(
        StructuralShortInternalConsonant,
        short_word_gets_internal_consonant_drift(original, replacement, error_class)
    ) {
        return Some(CandidateGateDecision {
            action: CandidateGateAction::SuggestOnly,
            reason: "short_internal_consonant_drift",
        });
    }
    if admission_trace_bool!(
        StructuralShortSameLengthMultiEdit,
        short_word_same_length_multi_edit_drift(original, replacement, error_class)
    ) {
        return Some(CandidateGateDecision {
            action: CandidateGateAction::SuggestOnly,
            reason: "short_same_length_multi_edit_drift",
        });
    }
    if admission_trace_bool!(
        StructuralSameTailConsonant,
        same_tail_single_consonant_drift(original, replacement, error_class)
    ) {
        return Some(CandidateGateDecision {
            action: CandidateGateAction::SuggestOnly,
            reason: "same_tail_single_consonant_drift",
        });
    }
    if admission_trace_bool!(
        StructuralInfinitiveOverreach,
        admission_fact_call!(
            lexical_facts,
            stable_known_form_grows_into_infinitive_overreach_with_facts,
            stable_known_form_grows_into_infinitive_overreach,
            original,
            replacement,
            error_class,
            origin,
        )
    ) {
        return Some(CandidateGateDecision {
            action: CandidateGateAction::SuggestOnly,
            reason: "known_form_to_infinitive_overreach",
        });
    }
    if admission_trace_bool!(
        StructuralProtectedContextAuthority,
        admission_fact_call!(
            lexical_facts,
            protected_current_surface_rewrite_requires_context_authority_with_facts,
            protected_current_surface_rewrite_requires_context_authority,
            original,
            replacement,
            error_class,
            origin,
        )
    ) {
        return Some(CandidateGateDecision {
            action: CandidateGateAction::SuggestOnly,
            reason: "protected_current_surface_rewrite_requires_context_authority",
        });
    }
    if admission_trace_bool!(
        StructuralKnownWordDifferentKnown,
        origin != CandidateOrigin::L3Context
            && admission_fact_call!(
                lexical_facts,
                known_russian_word_rewritten_to_different_known_word_with_facts,
                known_russian_word_rewritten_to_different_known_word,
                original,
                replacement,
                error_class,
            )
    ) {
        return Some(CandidateGateDecision {
            action: CandidateGateAction::SuggestOnly,
            reason: "known_word_to_different_known_word",
        });
    }
    if admission_trace_bool!(
        StructuralShortLayoutContext,
        short_layout_candidate_lacks_phrase_context(original, replacement, error_class, origin)
    ) {
        return Some(CandidateGateDecision {
            action: CandidateGateAction::KeepOriginal,
            reason: "short_layout_without_phrase_context",
        });
    }
    if admission_trace_bool!(
        StructuralShortCyrillicAscii,
        short_cyrillic_word_switches_to_ascii_layout(original, replacement, error_class, origin,)
    ) {
        return Some(CandidateGateDecision {
            action: CandidateGateAction::KeepOriginal,
            reason: "short_cyrillic_to_ascii_layout",
        });
    }
    if admission_trace_bool!(
        StructuralShortNandaShrink,
        short_nanda_composite_candidate_shrinks_word(original, replacement, error_class, origin,)
    ) {
        return Some(CandidateGateDecision {
            action: CandidateGateAction::SuggestOnly,
            reason: "short_nanda_word_shrink",
        });
    }
    if admission_trace_bool!(
        StructuralShortNandaInternalVowel,
        short_nanda_candidate_inserts_internal_vowel(original, replacement, error_class, origin,)
    ) {
        return Some(CandidateGateDecision {
            action: CandidateGateAction::SuggestOnly,
            reason: "short_nanda_internal_vowel_growth",
        });
    }
    if admission_trace_bool!(
        StructuralNandaUnknownWord,
        admission_fact_call!(
            lexical_facts,
            nanda_surface_candidate_outputs_unknown_word_with_facts,
            nanda_surface_candidate_outputs_unknown_word,
            original,
            replacement,
            error_class,
            origin,
        )
    ) {
        return Some(CandidateGateDecision {
            action: CandidateGateAction::SuggestOnly,
            reason: "nanda_surface_unknown_word",
        });
    }
    None
}

fn reflexive_suffix_candidate_requires_grammar_proof(
    original: &str,
    replacement: &str,
    error_class: TypingErrorClass,
) -> bool {
    if error_class == TypingErrorClass::GrammarAgreement {
        return false;
    }
    if !matches!(
        error_class,
        TypingErrorClass::MissingLetter
            | TypingErrorClass::CompositeTypo
            | TypingErrorClass::LetterSubstitution
    ) {
        return false;
    }
    let Some(original_word) = last_text_word(original) else {
        return false;
    };
    let Some(replacement_word) = last_text_word(replacement) else {
        return false;
    };
    if !is_cyrillic_letters_only(&original_word) || !is_cyrillic_letters_only(&replacement_word) {
        return false;
    }

    toggles_reflexive_soft_sign(
        &original_word.to_lowercase(),
        &replacement_word.to_lowercase(),
    )
}

fn toggles_reflexive_soft_sign(original: &str, replacement: &str) -> bool {
    let Some(stem) = original.strip_suffix("тся") else {
        return replacement
            .strip_suffix("тся")
            .is_some_and(|stem| original == format!("{stem}ться"));
    };
    replacement == format!("{stem}ться")
}

fn known_current_word_gets_unproven_surface_drift(
    original: &str,
    replacement: &str,
    error_class: TypingErrorClass,
    origin: CandidateOrigin,
) -> bool {
    if matches!(
        error_class,
        TypingErrorClass::WrongLayout
            | TypingErrorClass::PartialLayout
            | TypingErrorClass::SplitWord
            | TypingErrorClass::GluedWords
            | TypingErrorClass::CaseNoise
            | TypingErrorClass::TechnicalToken
            | TypingErrorClass::ProtectedToken
            | TypingErrorClass::CompletionOnly
            | TypingErrorClass::Unknown
    ) {
        return false;
    }
    if matches!(
        origin.source_role(),
        CorrectionSourceRole::Layout
            | CorrectionSourceRole::Boundary
            | CorrectionSourceRole::Technical
    ) {
        return false;
    }
    let Some(original_word) = last_text_word(original) else {
        return false;
    };
    let Some(replacement_word) = last_text_word(replacement) else {
        return false;
    };
    if !is_cyrillic_letters_only(&original_word) || !is_cyrillic_letters_only(&replacement_word) {
        return false;
    }

    let original_lower = original_word.to_lowercase();
    let replacement_lower = replacement_word.to_lowercase();
    if verified_surface_to_lexical_center_repair(&original_lower, &replacement_lower, error_class) {
        return false;
    }
    if original_lower == replacement_lower || !protected_current_surface_token(&original_lower) {
        return false;
    }

    let original_len = original_lower.chars().count();
    let replacement_len = replacement_lower.chars().count();
    if original_len < 4 || replacement_len > original_len + 1 {
        return false;
    }

    damerau_levenshtein(&original_lower, &replacement_lower) <= 1
        || inserted_char_position_for_missing_letter(&original_lower, &replacement_lower).is_some()
}

fn known_current_word_gets_unproven_surface_drift_with_facts(
    original: &str,
    replacement: &str,
    error_class: TypingErrorClass,
    origin: CandidateOrigin,
    facts: &AdmissionLexicalFacts<'_>,
) -> bool {
    if !facts.reuses_facts() {
        return known_current_word_gets_unproven_surface_drift(
            original,
            replacement,
            error_class,
            origin,
        );
    }
    facts.assert_pair(original, replacement);
    if matches!(
        error_class,
        TypingErrorClass::WrongLayout
            | TypingErrorClass::PartialLayout
            | TypingErrorClass::SplitWord
            | TypingErrorClass::GluedWords
            | TypingErrorClass::CaseNoise
            | TypingErrorClass::TechnicalToken
            | TypingErrorClass::ProtectedToken
            | TypingErrorClass::CompletionOnly
            | TypingErrorClass::Unknown
    ) {
        return false;
    }
    if matches!(
        origin.source_role(),
        CorrectionSourceRole::Layout
            | CorrectionSourceRole::Boundary
            | CorrectionSourceRole::Technical
    ) {
        return false;
    }
    let Some(original_word) = facts.original_word() else {
        return false;
    };
    let Some(replacement_word) = facts.replacement_word() else {
        return false;
    };
    if !original_word.is_cyrillic_letters_only() || !replacement_word.is_cyrillic_letters_only() {
        return false;
    }
    if verified_surface_to_lexical_center_repair_with_facts(
        original_word,
        replacement_word,
        error_class,
    ) {
        return false;
    }
    if original_word.lower() == replacement_word.lower() || !original_word.is_protected_current() {
        return false;
    }
    if original_word.char_len() < 4 || replacement_word.char_len() > original_word.char_len() + 1 {
        return false;
    }
    damerau_levenshtein(original_word.lower(), replacement_word.lower()) <= 1
        || inserted_char_position_for_missing_letter(
            original_word.lower(),
            replacement_word.lower(),
        )
        .is_some()
}

fn stable_known_form_grows_into_infinitive_overreach(
    original: &str,
    replacement: &str,
    error_class: TypingErrorClass,
    origin: CandidateOrigin,
) -> bool {
    if matches!(
        error_class,
        TypingErrorClass::WrongLayout
            | TypingErrorClass::PartialLayout
            | TypingErrorClass::SplitWord
            | TypingErrorClass::GluedWords
            | TypingErrorClass::CaseNoise
            | TypingErrorClass::RepeatedLetter
            | TypingErrorClass::AdjacentTransposition
            | TypingErrorClass::GrammarAgreement
            | TypingErrorClass::TechnicalToken
            | TypingErrorClass::ProtectedToken
            | TypingErrorClass::CompletionOnly
            | TypingErrorClass::Unknown
    ) {
        return false;
    }
    if matches!(
        origin.source_role(),
        CorrectionSourceRole::Layout
            | CorrectionSourceRole::Boundary
            | CorrectionSourceRole::Technical
    ) {
        return false;
    }
    if candidate_changes_non_last_word(original, replacement) {
        return false;
    }
    let Some(original_word) = last_text_word(original) else {
        return false;
    };
    let Some(replacement_word) = last_text_word(replacement) else {
        return false;
    };
    if !is_cyrillic_letters_only(&original_word) || !is_cyrillic_letters_only(&replacement_word) {
        return false;
    }

    let original_lower = original_word.to_lowercase();
    let replacement_lower = replacement_word.to_lowercase();
    if original_lower == replacement_lower
        || !protected_current_surface_token(&original_lower)
        || !known_russian_autocorrect_token(&replacement_lower)
    {
        return false;
    }
    let original_len = original_lower.chars().count();
    let replacement_len = replacement_lower.chars().count();
    if replacement_len <= original_len
        || replacement_len > original_len + 4
        || russian_infinitive_like_tail(&original_lower)
        || !russian_infinitive_like_tail(&replacement_lower)
    {
        return false;
    }

    common_prefix_chars(&original_lower, &replacement_lower) >= original_len.saturating_sub(2)
}

fn stable_known_form_grows_into_infinitive_overreach_with_facts(
    original: &str,
    replacement: &str,
    error_class: TypingErrorClass,
    origin: CandidateOrigin,
    facts: &AdmissionLexicalFacts<'_>,
) -> bool {
    if !facts.reuses_facts() {
        return stable_known_form_grows_into_infinitive_overreach(
            original,
            replacement,
            error_class,
            origin,
        );
    }
    facts.assert_pair(original, replacement);
    if matches!(
        error_class,
        TypingErrorClass::WrongLayout
            | TypingErrorClass::PartialLayout
            | TypingErrorClass::SplitWord
            | TypingErrorClass::GluedWords
            | TypingErrorClass::CaseNoise
            | TypingErrorClass::RepeatedLetter
            | TypingErrorClass::AdjacentTransposition
            | TypingErrorClass::GrammarAgreement
            | TypingErrorClass::TechnicalToken
            | TypingErrorClass::ProtectedToken
            | TypingErrorClass::CompletionOnly
            | TypingErrorClass::Unknown
    ) {
        return false;
    }
    if matches!(
        origin.source_role(),
        CorrectionSourceRole::Layout
            | CorrectionSourceRole::Boundary
            | CorrectionSourceRole::Technical
    ) {
        return false;
    }
    if candidate_changes_non_last_word(original, replacement) {
        return false;
    }
    let Some(original_word) = facts.original_word() else {
        return false;
    };
    let Some(replacement_word) = facts.replacement_word() else {
        return false;
    };
    if !original_word.is_cyrillic_letters_only() || !replacement_word.is_cyrillic_letters_only() {
        return false;
    }
    if original_word.lower() == replacement_word.lower()
        || !original_word.is_protected_current()
        || !replacement_word.is_known_russian()
    {
        return false;
    }
    if replacement_word.char_len() <= original_word.char_len()
        || replacement_word.char_len() > original_word.char_len() + 4
        || russian_infinitive_like_tail(original_word.lower())
        || !russian_infinitive_like_tail(replacement_word.lower())
    {
        return false;
    }
    common_prefix_chars(original_word.lower(), replacement_word.lower())
        >= original_word.char_len().saturating_sub(2)
}

fn protected_current_surface_rewrite_requires_context_authority(
    original: &str,
    replacement: &str,
    error_class: TypingErrorClass,
    origin: CandidateOrigin,
) -> bool {
    if origin != CandidateOrigin::L2Surface {
        return false;
    }
    if matches!(
        error_class,
        TypingErrorClass::WrongLayout
            | TypingErrorClass::PartialLayout
            | TypingErrorClass::SplitWord
            | TypingErrorClass::GluedWords
            | TypingErrorClass::BoundaryShift
            | TypingErrorClass::CaseNoise
            | TypingErrorClass::TechnicalToken
            | TypingErrorClass::ProtectedToken
            | TypingErrorClass::CompletionOnly
            | TypingErrorClass::Unknown
    ) {
        return false;
    }
    if candidate_changes_non_last_word(original, replacement) {
        return false;
    }
    let Some(original_word) = last_text_word(original) else {
        return false;
    };
    let Some(replacement_word) = last_text_word(replacement) else {
        return false;
    };
    if !is_cyrillic_letters_only(&original_word) || !is_cyrillic_letters_only(&replacement_word) {
        return false;
    }

    let original_lower = original_word.to_lowercase();
    let replacement_lower = replacement_word.to_lowercase();
    if original_lower == replacement_lower
        || !protected_current_surface_token(&original_lower)
        || original_lower.chars().count() < 4
        || replacement_lower.chars().count() < 4
    {
        return false;
    }

    true
}

fn protected_current_surface_rewrite_requires_context_authority_with_facts(
    original: &str,
    replacement: &str,
    error_class: TypingErrorClass,
    origin: CandidateOrigin,
    facts: &AdmissionLexicalFacts<'_>,
) -> bool {
    if !facts.reuses_facts() {
        return protected_current_surface_rewrite_requires_context_authority(
            original,
            replacement,
            error_class,
            origin,
        );
    }
    facts.assert_pair(original, replacement);
    if origin != CandidateOrigin::L2Surface {
        return false;
    }
    if matches!(
        error_class,
        TypingErrorClass::WrongLayout
            | TypingErrorClass::PartialLayout
            | TypingErrorClass::SplitWord
            | TypingErrorClass::GluedWords
            | TypingErrorClass::BoundaryShift
            | TypingErrorClass::CaseNoise
            | TypingErrorClass::TechnicalToken
            | TypingErrorClass::ProtectedToken
            | TypingErrorClass::CompletionOnly
            | TypingErrorClass::Unknown
    ) {
        return false;
    }
    if candidate_changes_non_last_word(original, replacement) {
        return false;
    }
    let Some(original_word) = facts.original_word() else {
        return false;
    };
    let Some(replacement_word) = facts.replacement_word() else {
        return false;
    };
    if !original_word.is_cyrillic_letters_only() || !replacement_word.is_cyrillic_letters_only() {
        return false;
    }
    if original_word.lower() == replacement_word.lower()
        || !original_word.is_protected_current()
        || original_word.char_len() < 4
        || replacement_word.char_len() < 4
    {
        return false;
    }
    true
}

fn verified_surface_to_lexical_center_repair(
    original_lower: &str,
    replacement_lower: &str,
    error_class: TypingErrorClass,
) -> bool {
    matches!(
        error_class,
        TypingErrorClass::LetterSubstitution
            | TypingErrorClass::MissingLetter
            | TypingErrorClass::SparseInternalMultiOmission
            | TypingErrorClass::ExtraLetter
            | TypingErrorClass::RepeatedLetter
            | TypingErrorClass::AdjacentTransposition
    ) && !known_russian_autocorrect_token(original_lower)
        && known_russian_autocorrect_token(replacement_lower)
        && (damerau_levenshtein(original_lower, replacement_lower) <= 1
            || crate::text_metrics::sparse_internal_omission_count(
                original_lower,
                replacement_lower,
            )
            .is_some())
}

fn verified_surface_to_lexical_center_repair_with_facts(
    original: &AdmissionWordFacts,
    replacement: &AdmissionWordFacts,
    error_class: TypingErrorClass,
) -> bool {
    matches!(
        error_class,
        TypingErrorClass::LetterSubstitution
            | TypingErrorClass::MissingLetter
            | TypingErrorClass::SparseInternalMultiOmission
            | TypingErrorClass::ExtraLetter
            | TypingErrorClass::RepeatedLetter
            | TypingErrorClass::AdjacentTransposition
    ) && !original.is_known_russian()
        && replacement.is_known_russian()
        && (damerau_levenshtein(original.lower(), replacement.lower()) <= 1
            || crate::text_metrics::sparse_internal_omission_count(
                original.lower(),
                replacement.lower(),
            )
            .is_some())
}

fn unproven_stable_surface_shape_drift(
    original: &str,
    replacement: &str,
    error_class: TypingErrorClass,
    origin: CandidateOrigin,
) -> bool {
    if matches!(
        error_class,
        TypingErrorClass::WrongLayout
            | TypingErrorClass::PartialLayout
            | TypingErrorClass::SplitWord
            | TypingErrorClass::GluedWords
            | TypingErrorClass::CaseNoise
            | TypingErrorClass::RepeatedLetter
            | TypingErrorClass::AdjacentTransposition
            | TypingErrorClass::TechnicalToken
            | TypingErrorClass::ProtectedToken
            | TypingErrorClass::CompletionOnly
            | TypingErrorClass::Unknown
    ) {
        return false;
    }
    if matches!(
        origin.source_role(),
        CorrectionSourceRole::Layout
            | CorrectionSourceRole::Boundary
            | CorrectionSourceRole::Technical
    ) {
        return false;
    }
    if candidate_changes_non_last_word(original, replacement) {
        return false;
    }
    let Some(original_word) = last_text_word(original) else {
        return false;
    };
    let Some(replacement_word) = last_text_word(replacement) else {
        return false;
    };
    if !is_cyrillic_letters_only(&original_word) || !is_cyrillic_letters_only(&replacement_word) {
        return false;
    }

    let original_lower = original_word.to_lowercase();
    let replacement_lower = replacement_word.to_lowercase();
    if verified_surface_to_lexical_center_repair(&original_lower, &replacement_lower, error_class) {
        return false;
    }
    if original_lower == replacement_lower {
        return false;
    }

    unproven_internal_vowel_insertion(&original_lower, &replacement_lower)
        || unproven_soft_sign_tail_insertion(&original_lower, &replacement_lower)
        || unproven_short_vowel_substitution(&original_lower, &replacement_lower)
        || unproven_tail_vowel_substitution(&original_lower, &replacement_lower)
        || unproven_inflection_tail_vowel_to_consonant(&original_lower, &replacement_lower)
}

fn unproven_stable_surface_shape_drift_with_facts(
    original: &str,
    replacement: &str,
    error_class: TypingErrorClass,
    origin: CandidateOrigin,
    facts: &AdmissionLexicalFacts<'_>,
) -> bool {
    if !facts.reuses_facts() {
        return unproven_stable_surface_shape_drift(original, replacement, error_class, origin);
    }
    facts.assert_pair(original, replacement);
    if matches!(
        error_class,
        TypingErrorClass::WrongLayout
            | TypingErrorClass::PartialLayout
            | TypingErrorClass::SplitWord
            | TypingErrorClass::GluedWords
            | TypingErrorClass::CaseNoise
            | TypingErrorClass::RepeatedLetter
            | TypingErrorClass::AdjacentTransposition
            | TypingErrorClass::TechnicalToken
            | TypingErrorClass::ProtectedToken
            | TypingErrorClass::CompletionOnly
            | TypingErrorClass::Unknown
    ) {
        return false;
    }
    if matches!(
        origin.source_role(),
        CorrectionSourceRole::Layout
            | CorrectionSourceRole::Boundary
            | CorrectionSourceRole::Technical
    ) {
        return false;
    }
    if candidate_changes_non_last_word(original, replacement) {
        return false;
    }
    let Some(original_word) = facts.original_word() else {
        return false;
    };
    let Some(replacement_word) = facts.replacement_word() else {
        return false;
    };
    if !original_word.is_cyrillic_letters_only() || !replacement_word.is_cyrillic_letters_only() {
        return false;
    }
    if verified_surface_to_lexical_center_repair_with_facts(
        original_word,
        replacement_word,
        error_class,
    ) {
        return false;
    }
    if original_word.lower() == replacement_word.lower() {
        return false;
    }
    unproven_internal_vowel_insertion(original_word.lower(), replacement_word.lower())
        || unproven_soft_sign_tail_insertion(original_word.lower(), replacement_word.lower())
        || unproven_short_vowel_substitution(original_word.lower(), replacement_word.lower())
        || unproven_tail_vowel_substitution(original_word.lower(), replacement_word.lower())
        || unproven_inflection_tail_vowel_to_consonant(
            original_word.lower(),
            replacement_word.lower(),
        )
}

fn unproven_internal_vowel_insertion(original: &str, replacement: &str) -> bool {
    let Some((idx, inserted)) = inserted_char_position_for_missing_letter(original, replacement)
    else {
        return false;
    };
    if idx == 0 || !crate::russian_chars::is_russian_vowel(inserted) {
        return false;
    }
    !inserted_vowel_repairs_consonant_cluster(original, idx)
}

fn inserted_vowel_repairs_consonant_cluster(original: &str, idx: usize) -> bool {
    let chars = original.chars().collect::<Vec<_>>();
    if idx == 0 || idx >= chars.len() {
        return false;
    }
    is_russian_consonant(chars[idx - 1]) && is_russian_consonant(chars[idx])
}

fn unproven_soft_sign_tail_insertion(original: &str, replacement: &str) -> bool {
    let Some((idx, inserted)) = inserted_char_position_for_missing_letter(original, replacement)
    else {
        return false;
    };
    inserted == 'ь' && idx + 2 >= original.chars().count()
}

fn unproven_short_vowel_substitution(original: &str, replacement: &str) -> bool {
    let original_chars = original.chars().collect::<Vec<_>>();
    let replacement_chars = replacement.chars().collect::<Vec<_>>();
    if original_chars.len() > 6
        || original_chars.len() != replacement_chars.len()
        || damerau_levenshtein(original, replacement) != 1
    {
        return false;
    }
    let diffs = original_chars
        .iter()
        .zip(&replacement_chars)
        .enumerate()
        .filter(|(_, (left, right))| left != right)
        .collect::<Vec<_>>();
    let [(idx, (left, right))] = diffs.as_slice() else {
        return false;
    };
    *idx > 0
        && crate::russian_chars::is_russian_vowel(**left)
        && crate::russian_chars::is_russian_vowel(**right)
}

fn unproven_tail_vowel_substitution(original: &str, replacement: &str) -> bool {
    let original_chars = original.chars().collect::<Vec<_>>();
    let replacement_chars = replacement.chars().collect::<Vec<_>>();
    if original_chars.len() < 7
        || original_chars.len() != replacement_chars.len()
        || damerau_levenshtein(original, replacement) != 1
    {
        return false;
    }
    let diffs = original_chars
        .iter()
        .zip(&replacement_chars)
        .enumerate()
        .filter(|(_, (left, right))| left != right)
        .collect::<Vec<_>>();
    let [(idx, (left, right))] = diffs.as_slice() else {
        return false;
    };
    *idx + 2 >= original_chars.len()
        && crate::russian_chars::is_russian_vowel(**left)
        && crate::russian_chars::is_russian_vowel(**right)
}

fn unproven_inflection_tail_vowel_to_consonant(original: &str, replacement: &str) -> bool {
    let original_chars = original.chars().collect::<Vec<_>>();
    let replacement_chars = replacement.chars().collect::<Vec<_>>();
    if original_chars.len() < 5
        || original_chars.len() != replacement_chars.len()
        || damerau_levenshtein(original, replacement) != 1
    {
        return false;
    }
    let diffs = original_chars
        .iter()
        .zip(&replacement_chars)
        .enumerate()
        .filter(|(_, (left, right))| left != right)
        .collect::<Vec<_>>();
    let [(idx, (left, right))] = diffs.as_slice() else {
        return false;
    };
    *idx + 3 >= original_chars.len()
        && crate::russian_chars::is_russian_vowel(**left)
        && is_russian_consonant(**right)
        && original_chars
            .last()
            .is_some_and(|ch| crate::russian_chars::is_russian_vowel(*ch))
}

fn protected_current_surface_token(lower: &str) -> bool {
    let field = crate::hot_field::HotFieldSnapshot::current();
    known_russian_autocorrect_token(lower)
        || crate::phrase_lexicon::is_known_russian_phrase_part(lower)
        || field.input_surface_readout(lower).has_phase_authority()
        || field.word_readout(lower).is_known()
        || crate::nanda_wave::l2::l2_surface_foundation_contains(lower)
        || crate::nanda_wave::l2::l2_surface_foundation_has_authority(lower)
        || crate::russian_lexicon::is_reference_backed_russian_form(lower)
        || crate::russian_lexicon::is_center_backed_russian_form(lower)
        || crate::russian_lexicon::is_reference_known_russian_word_or_form(lower)
        || crate::russian_lexicon::russian_dictionary().contains(lower)
        || crate::russian_lexicon::russian_short_dictionary().contains(lower)
}

fn russian_infinitive_like_tail(lower: &str) -> bool {
    matches!(
        lower,
        word if word.ends_with("ться")
            || word.ends_with("тись")
            || word.ends_with("чься")
            || word.ends_with("ть")
            || word.ends_with("ти")
            || word.ends_with("чь")
    )
}

fn common_prefix_chars(left: &str, right: &str) -> usize {
    left.chars()
        .zip(right.chars())
        .take_while(|(left, right)| left == right)
        .count()
}

fn boundary_operator_changes_non_whitespace_surface(
    original: &str,
    replacement: &str,
    error_class: TypingErrorClass,
    origin: CandidateOrigin,
) -> bool {
    if !matches!(
        error_class,
        TypingErrorClass::BoundaryShift
            | TypingErrorClass::SplitWord
            | TypingErrorClass::GluedWords
    ) && origin != CandidateOrigin::Boundary
    {
        return false;
    }
    if crate::text_metrics::current_token_repaired_boundary_split(original, replacement) {
        return false;
    }
    original
        .chars()
        .filter(|ch| !ch.is_whitespace())
        .ne(replacement.chars().filter(|ch| !ch.is_whitespace()))
}

fn replacement_glues_separate_words_without_boundary_class(
    original: &str,
    replacement: &str,
    error_class: TypingErrorClass,
) -> bool {
    if matches!(
        error_class,
        TypingErrorClass::BoundaryShift
            | TypingErrorClass::SplitWord
            | TypingErrorClass::GluedWords
    ) {
        return false;
    }
    let original_words = core_word_count(original);
    let replacement_words = core_word_count(replacement);
    original_words >= 2 && replacement_words < original_words
}

fn boundary_candidate_glues_short_function_tail(
    original: &str,
    replacement: &str,
    error_class: TypingErrorClass,
) -> bool {
    if !matches!(
        error_class,
        TypingErrorClass::BoundaryShift
            | TypingErrorClass::SplitWord
            | TypingErrorClass::GluedWords
    ) {
        return false;
    }
    let (_, original_core, _) = split_edge_whitespace(original);
    let (_, replacement_core, _) = split_edge_whitespace(replacement);
    let original_words = original_core.split_whitespace().collect::<Vec<_>>();
    let replacement_words = replacement_core.split_whitespace().collect::<Vec<_>>();
    if original_words.len() != 2 || replacement_words.len() != 1 {
        return false;
    }
    let left = original_words[0];
    let right = original_words[1];
    let merged = replacement_words[0];
    if !edit_transition::same_cyrillic_token(&format!("{left}{right}"), merged) {
        return false;
    }
    let (_, right_word, _) = split_word_punctuation(right);
    let right_lower = right_word.to_lowercase();
    if matches!(right_lower.as_str(), "ся" | "сь") {
        return false;
    }
    right_word.chars().count() <= 3
        && (crate::phrase_lexicon::is_known_russian_phrase_part(&right_lower)
            || crate::russian_lexicon::is_known_russian_word_or_form(&right_lower)
            || crate::lexicon::is_common_ru_word(&right_lower))
}

fn boundary_candidate_eats_known_current_word(
    original: &str,
    replacement: &str,
    error_class: TypingErrorClass,
    origin: CandidateOrigin,
) -> bool {
    if error_class != TypingErrorClass::BoundaryShift || origin != CandidateOrigin::Boundary {
        return false;
    }
    let (_, original_core, _) = split_edge_whitespace(original);
    let (_, replacement_core, _) = split_edge_whitespace(replacement);
    let original_words = original_core.split_whitespace().collect::<Vec<_>>();
    let replacement_words = replacement_core.split_whitespace().collect::<Vec<_>>();
    if original_words.len() < 2 || original_words.len() != replacement_words.len() {
        return false;
    }
    let Some((original_last, original_prefix)) = original_words.split_last() else {
        return false;
    };
    let Some((replacement_last, replacement_prefix)) = replacement_words.split_last() else {
        return false;
    };
    if original_prefix != replacement_prefix {
        return false;
    }
    let (_, original_word, _) = split_word_punctuation(original_last);
    let (_, replacement_word, _) = split_word_punctuation(replacement_last);
    if !is_cyrillic_letters_only(original_word)
        || !is_cyrillic_letters_only(replacement_word)
        || original_word.chars().count() < 4
    {
        return false;
    }
    let original_lower = original_word.to_lowercase();
    let replacement_lower = replacement_word.to_lowercase();
    let stripped = original_lower.chars().skip(1).collect::<String>();
    stripped == replacement_lower
}

fn multi_word_candidate_only_completes_last_vowel(
    original: &str,
    replacement: &str,
    error_class: TypingErrorClass,
) -> bool {
    if !matches!(
        error_class,
        TypingErrorClass::MissingLetter | TypingErrorClass::CompositeTypo
    ) {
        return false;
    }
    let (_, original_core, _) = split_edge_whitespace(original);
    let (_, replacement_core, _) = split_edge_whitespace(replacement);
    let original_words = original_core.split_whitespace().collect::<Vec<_>>();
    let replacement_words = replacement_core.split_whitespace().collect::<Vec<_>>();
    if original_words.len() < 2 || original_words.len() != replacement_words.len() {
        return false;
    }
    let Some((original_last, original_prefix)) = original_words.split_last() else {
        return false;
    };
    let Some((replacement_last, replacement_prefix)) = replacement_words.split_last() else {
        return false;
    };
    if original_prefix != replacement_prefix {
        return false;
    }
    let (_, original_word, _) = split_word_punctuation(original_last);
    let (_, replacement_word, _) = split_word_punctuation(replacement_last);
    if !is_cyrillic_letters_only(original_word)
        || !is_cyrillic_letters_only(replacement_word)
        || original_word.chars().count() < 5
    {
        return false;
    }
    let original_lower = original_word.to_lowercase();
    let replacement_lower = replacement_word.to_lowercase();
    let Some(suffix) = replacement_lower.strip_prefix(&original_lower) else {
        return false;
    };
    suffix.chars().count() == 1
        && suffix.chars().next().is_some_and(|ch| {
            matches!(
                ch,
                'а' | 'е' | 'ё' | 'и' | 'о' | 'у' | 'ы' | 'э' | 'ю' | 'я'
            )
        })
}

fn adjacent_transposition_competes_with_single_letter_boundary(
    original: &str,
    replacement: &str,
    error_class: TypingErrorClass,
) -> bool {
    if error_class != TypingErrorClass::AdjacentTransposition {
        return false;
    }
    let (_, original_core, _) = split_edge_whitespace(original);
    let (_, replacement_core, _) = split_edge_whitespace(replacement);
    let original_words = original_core.split_whitespace().collect::<Vec<_>>();
    let replacement_words = replacement_core.split_whitespace().collect::<Vec<_>>();
    if original_words.len() < 2 || original_words.len() != replacement_words.len() {
        return false;
    }
    let Some((original_last, original_prefix)) = original_words.split_last() else {
        return false;
    };
    let Some((replacement_last, replacement_prefix)) = replacement_words.split_last() else {
        return false;
    };
    if original_prefix != replacement_prefix {
        return false;
    }
    let (_, original_word, _) = split_word_punctuation(original_last);
    let (_, replacement_word, _) = split_word_punctuation(replacement_last);
    if !is_cyrillic_letters_only(original_word)
        || !is_cyrillic_letters_only(replacement_word)
        || original_word.chars().count() < 4
    {
        return false;
    }
    let original_lower = original_word.to_lowercase();
    let replacement_lower = replacement_word.to_lowercase();
    if original_lower == replacement_lower {
        return false;
    }
    let mut chars = original_lower.chars();
    let Some(preposition) = chars.next() else {
        return false;
    };
    if !matches!(preposition, 'в' | 'к' | 'с') {
        return false;
    }
    let tail = chars.collect::<String>();
    tail.chars().count() >= 3
        && (crate::phrase_lexicon::is_known_russian_phrase_part(&tail)
            || crate::russian_lexicon::is_known_russian_word_or_form(&tail)
            || crate::lexicon::is_common_ru_word(&tail))
}

fn boundary_candidate_splits_known_russian_word(
    original: &str,
    replacement: &str,
    _error_class: TypingErrorClass,
) -> bool {
    let (_, original_core, _) = split_edge_whitespace(original);
    let (_, replacement_core, _) = split_edge_whitespace(replacement);
    let original_words = original_core.split_whitespace().collect::<Vec<_>>();
    let replacement_words = replacement_core.split_whitespace().collect::<Vec<_>>();
    if original_words.is_empty() || replacement_words.len() != original_words.len() + 1 {
        return false;
    }

    for original_idx in 0..original_words.len() {
        let first = replacement_words
            .get(original_idx)
            .copied()
            .unwrap_or_default();
        let second = replacement_words
            .get(original_idx + 1)
            .copied()
            .unwrap_or_default();
        let merged = format!("{first}{second}");
        if !same_known_russian_token(original_words[original_idx], &merged) {
            continue;
        }

        let before_matches = original_words[..original_idx] == replacement_words[..original_idx];
        let after_matches =
            original_words[original_idx + 1..] == replacement_words[original_idx + 2..];
        if before_matches && after_matches {
            return true;
        }
    }
    false
}

fn boundary_candidate_splits_to_short_function_and_weak_tail(
    original: &str,
    replacement: &str,
    error_class: TypingErrorClass,
) -> bool {
    if !matches!(
        error_class,
        TypingErrorClass::SplitWord | TypingErrorClass::GluedWords
    ) {
        return false;
    }
    let (_, original_core, _) = split_edge_whitespace(original);
    let (_, replacement_core, _) = split_edge_whitespace(replacement);
    let original_words = original_core.split_whitespace().collect::<Vec<_>>();
    let replacement_words = replacement_core.split_whitespace().collect::<Vec<_>>();
    if original_words.is_empty() || replacement_words.len() != original_words.len() + 1 {
        return false;
    }

    for (original_idx, original_word) in original_words.iter().enumerate() {
        let first = replacement_words
            .get(original_idx)
            .copied()
            .unwrap_or_default();
        let second = replacement_words
            .get(original_idx + 1)
            .copied()
            .unwrap_or_default();
        let merged = format!("{first}{second}");
        if !edit_transition::same_cyrillic_token(original_word, &merged) {
            continue;
        }

        let (_, first_word, _) = split_word_punctuation(first);
        let (_, second_word, _) = split_word_punctuation(second);
        let first_lower = first_word.to_lowercase();
        let second_lower = second_word.to_lowercase();
        if first_word.chars().count() == 1
            && crate::phrase_lexicon::is_one_letter_russian_function_word(&first_lower)
            && !strong_standalone_split_tail(&second_lower)
        {
            return true;
        }
    }
    false
}

pub(crate) fn semantic_candidate_lacks_surface_authority(
    original: &str,
    replacement: &str,
    origin: CandidateOrigin,
) -> bool {
    if origin != CandidateOrigin::L3Context {
        return false;
    }
    let Some(original_word) = last_text_word(original) else {
        return true;
    };
    let Some(replacement_word) = last_text_word(replacement) else {
        return true;
    };
    if !is_cyrillic_letters_only(&original_word) || !is_cyrillic_letters_only(&replacement_word) {
        return false;
    }

    let original_lower = original_word.to_lowercase();
    let replacement_lower = replacement_word.to_lowercase();
    let distance = damerau_levenshtein(&original_lower, &replacement_lower);
    if distance <= 1 {
        return false;
    }
    !crate::nanda_wave::l2::l2_center_near_surfaces(&original_lower, 24)
        .iter()
        .any(|candidate| candidate == &replacement_lower)
}

fn l2_surface_candidate_truncates_to_stem_without_deletion_proof(
    original: &str,
    replacement: &str,
    origin: CandidateOrigin,
) -> bool {
    if origin != CandidateOrigin::L2Surface {
        return false;
    }
    let Some(original_word) = last_text_word(original) else {
        return false;
    };
    let Some(replacement_word) = last_text_word(replacement) else {
        return false;
    };
    if !is_cyrillic_letters_only(&original_word) || !is_cyrillic_letters_only(&replacement_word) {
        return false;
    }
    let original_lower = original_word.to_lowercase();
    let replacement_lower = replacement_word.to_lowercase();
    let original_len = original_lower.chars().count();
    let replacement_len = replacement_lower.chars().count();
    if replacement_len >= original_len || replacement_len < 4 {
        return false;
    }
    if one_deletion_reduces_to(&original_lower, &replacement_lower) {
        return false;
    }
    let prefix = crate::text_metrics::common_prefix_char_len(&original_lower, &replacement_lower);
    prefix + 1 >= replacement_len
}

fn one_deletion_reduces_to(original: &str, replacement: &str) -> bool {
    let original_chars = original.chars().collect::<Vec<_>>();
    let replacement_chars = replacement.chars().collect::<Vec<_>>();
    if original_chars.len() != replacement_chars.len() + 1 {
        return false;
    }
    for skip in 0..original_chars.len() {
        if original_chars
            .iter()
            .enumerate()
            .filter_map(|(idx, ch)| (idx != skip).then_some(*ch))
            .eq(replacement_chars.iter().copied())
        {
            return true;
        }
    }
    false
}

fn candidate_over_compresses_word(
    original: &str,
    replacement: &str,
    error_class: TypingErrorClass,
) -> bool {
    if matches!(
        error_class,
        TypingErrorClass::WrongLayout
            | TypingErrorClass::PartialLayout
            | TypingErrorClass::SplitWord
            | TypingErrorClass::GluedWords
            | TypingErrorClass::RepeatedLetter
    ) {
        return false;
    }
    let Some(original_word) = last_text_word(original) else {
        return false;
    };
    let Some(replacement_word) = last_text_word(replacement) else {
        return false;
    };
    if !is_cyrillic_letters_only(&original_word) || !is_cyrillic_letters_only(&replacement_word) {
        return false;
    }
    let original_len = original_word.chars().count();
    let replacement_len = replacement_word.chars().count();
    original_len >= 6 && replacement_len + 3 <= original_len
}

fn candidate_drops_letter_after_one_letter_function_prefix(
    original: &str,
    replacement: &str,
    error_class: TypingErrorClass,
) -> bool {
    if matches!(
        error_class,
        TypingErrorClass::WrongLayout
            | TypingErrorClass::PartialLayout
            | TypingErrorClass::SplitWord
            | TypingErrorClass::GluedWords
    ) {
        return false;
    }

    let (_, original_core, _) = split_edge_whitespace(original);
    let (_, replacement_core, _) = split_edge_whitespace(replacement);
    let original_words = original_core.split_whitespace().collect::<Vec<_>>();
    let replacement_words = replacement_core.split_whitespace().collect::<Vec<_>>();
    if original_words.len() < 2 || original_words.len() != replacement_words.len() {
        return false;
    }
    let Some((original_last, original_prefix)) = original_words.split_last() else {
        return false;
    };
    let Some((replacement_last, replacement_prefix)) = replacement_words.split_last() else {
        return false;
    };
    if original_prefix != replacement_prefix {
        return false;
    }

    let (_, original_word, _) = split_word_punctuation(original_last);
    let (_, replacement_word, _) = split_word_punctuation(replacement_last);
    if !is_cyrillic_letters_only(original_word) || !is_cyrillic_letters_only(replacement_word) {
        return false;
    }
    let original_lower = original_word.to_lowercase();
    let replacement_lower = replacement_word.to_lowercase();
    let original_chars = original_lower.chars().collect::<Vec<_>>();
    if original_chars.len() < 4 || replacement_lower.chars().count() + 1 != original_chars.len() {
        return false;
    }
    let prefix = original_chars[0].to_string();
    if !crate::phrase_lexicon::is_one_letter_russian_function_word(&prefix) {
        return false;
    }

    let compressed = std::iter::once(original_chars[0])
        .chain(original_chars.iter().skip(2).copied())
        .collect::<String>();
    compressed == replacement_lower
}

fn known_russian_word_rewritten_to_different_known_word(
    original: &str,
    replacement: &str,
    error_class: TypingErrorClass,
) -> bool {
    if !matches!(
        error_class,
        TypingErrorClass::CompositeTypo
            | TypingErrorClass::BoundaryShift
            | TypingErrorClass::MissingLetter
            | TypingErrorClass::SparseInternalMultiOmission
            | TypingErrorClass::ExtraLetter
            | TypingErrorClass::RepeatedLetter
            | TypingErrorClass::AdjacentTransposition
            | TypingErrorClass::LetterSubstitution
            | TypingErrorClass::GrammarAgreement
    ) {
        return false;
    }
    let Some(original_word) = last_text_word(original) else {
        return false;
    };
    let Some(replacement_word) = last_text_word(replacement) else {
        return false;
    };
    if !is_cyrillic_letters_only(&original_word) || !is_cyrillic_letters_only(&replacement_word) {
        return false;
    }

    let original_lower = original_word.to_lowercase();
    let replacement_lower = replacement_word.to_lowercase();
    if original_lower == replacement_lower {
        return false;
    }

    known_russian_autocorrect_token(&original_lower)
        && known_russian_autocorrect_token(&replacement_lower)
}

fn known_russian_word_rewritten_to_different_known_word_with_facts(
    original: &str,
    replacement: &str,
    error_class: TypingErrorClass,
    facts: &AdmissionLexicalFacts<'_>,
) -> bool {
    if !facts.reuses_facts() {
        return known_russian_word_rewritten_to_different_known_word(
            original,
            replacement,
            error_class,
        );
    }
    facts.assert_pair(original, replacement);
    if !matches!(
        error_class,
        TypingErrorClass::CompositeTypo
            | TypingErrorClass::BoundaryShift
            | TypingErrorClass::MissingLetter
            | TypingErrorClass::SparseInternalMultiOmission
            | TypingErrorClass::ExtraLetter
            | TypingErrorClass::RepeatedLetter
            | TypingErrorClass::AdjacentTransposition
            | TypingErrorClass::LetterSubstitution
            | TypingErrorClass::GrammarAgreement
    ) {
        return false;
    }
    let Some(original_word) = facts.original_word() else {
        return false;
    };
    let Some(replacement_word) = facts.replacement_word() else {
        return false;
    };
    if !original_word.is_cyrillic_letters_only() || !replacement_word.is_cyrillic_letters_only() {
        return false;
    }
    if original_word.lower() == replacement_word.lower() {
        return false;
    }
    original_word.is_known_russian() && replacement_word.is_known_russian()
}

fn known_russian_autocorrect_token(lower: &str) -> bool {
    crate::lexicon::is_common_ru_word(lower)
        || crate::lexicon::is_ru_live_protected_word(lower)
        || crate::lexicon::is_user_protected_word(lower)
        || crate::nanda_wave::l2::l2_surface_foundation_contains(lower)
        || crate::russian_lexicon::is_reference_backed_russian_form(lower)
        || crate::russian_lexicon::is_reference_known_russian_word_or_form(lower)
        || crate::russian_lexicon::is_known_russian_word_or_form(lower)
        || crate::russian_lexicon::is_known_russian_adverb_o_form(lower)
        || crate::russian_lexicon::is_known_russian_ka_oblique_form(lower)
        || protected_pattern_term_stem(lower)
}

fn protected_pattern_term_stem(lower: &str) -> bool {
    lower.starts_with("патерн") || lower.starts_with("паттерн")
}

fn known_phrase_part_only_grows_by_one_letter(
    original: &str,
    replacement: &str,
    error_class: TypingErrorClass,
) -> bool {
    if !matches!(
        error_class,
        TypingErrorClass::MissingLetter
            | TypingErrorClass::CompositeTypo
            | TypingErrorClass::LetterSubstitution
            | TypingErrorClass::GrammarAgreement
            | TypingErrorClass::AdjacentTransposition
    ) {
        return false;
    }
    let Some(original_word) = last_text_word(original) else {
        return false;
    };
    let Some(replacement_word) = last_text_word(replacement) else {
        return false;
    };
    let original_lower = original_word.to_lowercase();
    let replacement_lower = replacement_word.to_lowercase();
    if !is_cyrillic_letters_only(&original_word)
        || !is_cyrillic_letters_only(&replacement_word)
        || original_lower == replacement_lower
        || !crate::phrase_lexicon::is_known_russian_phrase_part(&original_lower)
    {
        return false;
    }

    inserted_char_position_for_missing_letter(&original_lower, &replacement_lower).is_some()
}

fn short_word_only_grows_initial_letter(
    original: &str,
    replacement: &str,
    error_class: TypingErrorClass,
) -> bool {
    if !matches!(
        error_class,
        TypingErrorClass::MissingLetter
            | TypingErrorClass::CompositeTypo
            | TypingErrorClass::LetterSubstitution
            | TypingErrorClass::GrammarAgreement
            | TypingErrorClass::AdjacentTransposition
    ) {
        return false;
    }
    let Some(original_word) = last_text_word(original) else {
        return false;
    };
    let Some(replacement_word) = last_text_word(replacement) else {
        return false;
    };
    let original_lower = original_word.to_lowercase();
    let replacement_lower = replacement_word.to_lowercase();
    if !is_cyrillic_letters_only(&original_word)
        || !is_cyrillic_letters_only(&replacement_word)
        || original_lower == replacement_lower
    {
        return false;
    }
    let Some((idx, _inserted)) =
        inserted_char_position_for_missing_letter(&original_lower, &replacement_lower)
    else {
        return false;
    };
    idx == 0 && original_lower.chars().count() <= 6
}

fn short_word_gets_case_vowel_drift(
    original: &str,
    replacement: &str,
    error_class: TypingErrorClass,
) -> bool {
    if !matches!(
        error_class,
        TypingErrorClass::MissingLetter
            | TypingErrorClass::CompositeTypo
            | TypingErrorClass::LetterSubstitution
            | TypingErrorClass::GrammarAgreement
    ) {
        return false;
    }
    let Some(original_word) = last_text_word(original) else {
        return false;
    };
    let Some(replacement_word) = last_text_word(replacement) else {
        return false;
    };
    if !is_cyrillic_letters_only(&original_word) || !is_cyrillic_letters_only(&replacement_word) {
        return false;
    }
    let original_lower = original_word.to_lowercase();
    let replacement_lower = replacement_word.to_lowercase();
    if original_lower.chars().count() > 5 || !replacement_lower.starts_with(&original_lower) {
        return false;
    }
    let suffix = replacement_lower
        .strip_prefix(&original_lower)
        .unwrap_or_default();
    suffix.chars().count() == 1 && matches!(suffix, "а" | "я" | "у" | "ю" | "ы" | "и")
}

fn soft_sign_word_gets_vowel_drift(
    original: &str,
    replacement: &str,
    error_class: TypingErrorClass,
) -> bool {
    if !matches!(
        error_class,
        TypingErrorClass::MissingLetter
            | TypingErrorClass::CompositeTypo
            | TypingErrorClass::LetterSubstitution
            | TypingErrorClass::GrammarAgreement
    ) {
        return false;
    }
    let Some(original_word) = last_text_word(original) else {
        return false;
    };
    let Some(replacement_word) = last_text_word(replacement) else {
        return false;
    };
    if !is_cyrillic_letters_only(&original_word) || !is_cyrillic_letters_only(&replacement_word) {
        return false;
    }
    let original_lower = original_word.to_lowercase();
    let replacement_lower = replacement_word.to_lowercase();
    if !original_lower.ends_with('ь') || original_lower.chars().count() > 6 {
        return false;
    }
    let original_stem = original_lower.trim_end_matches('ь');
    replacement_lower.starts_with(original_stem)
        && replacement_lower
            .chars()
            .last()
            .is_some_and(crate::russian_chars::is_russian_vowel)
}

fn short_word_gets_internal_consonant_drift(
    original: &str,
    replacement: &str,
    error_class: TypingErrorClass,
) -> bool {
    if !matches!(
        error_class,
        TypingErrorClass::MissingLetter
            | TypingErrorClass::CompositeTypo
            | TypingErrorClass::LetterSubstitution
            | TypingErrorClass::GrammarAgreement
    ) {
        return false;
    }
    let Some(original_word) = last_text_word(original) else {
        return false;
    };
    let Some(replacement_word) = last_text_word(replacement) else {
        return false;
    };
    if !is_cyrillic_letters_only(&original_word) || !is_cyrillic_letters_only(&replacement_word) {
        return false;
    }
    let original_lower = original_word.to_lowercase();
    let replacement_lower = replacement_word.to_lowercase();
    if verified_surface_to_lexical_center_repair(&original_lower, &replacement_lower, error_class) {
        return false;
    }
    if original_lower.chars().count() > 6 {
        return false;
    }
    let Some((idx, inserted)) =
        inserted_char_position_for_missing_letter(&original_lower, &replacement_lower)
    else {
        return false;
    };
    if crate::russian_chars::is_russian_vowel(inserted) {
        return false;
    }
    let previous_original = idx
        .checked_sub(1)
        .and_then(|previous_idx| original_lower.chars().nth(previous_idx));
    let next_original = original_lower.chars().nth(idx);
    if Some(inserted) == previous_original || Some(inserted) == next_original {
        return false;
    }
    !(inserted == 'ч' && matches!(next_original, Some('ш' | 'щ')))
}

fn short_word_same_length_multi_edit_drift(
    original: &str,
    replacement: &str,
    error_class: TypingErrorClass,
) -> bool {
    if !matches!(
        error_class,
        TypingErrorClass::CompositeTypo
            | TypingErrorClass::LetterSubstitution
            | TypingErrorClass::GrammarAgreement
    ) {
        return false;
    }
    let Some(original_word) = last_text_word(original) else {
        return false;
    };
    let Some(replacement_word) = last_text_word(replacement) else {
        return false;
    };
    if !is_cyrillic_letters_only(&original_word) || !is_cyrillic_letters_only(&replacement_word) {
        return false;
    }
    let original_lower = original_word.to_lowercase();
    let replacement_lower = replacement_word.to_lowercase();
    let original_len = original_lower.chars().count();
    let replacement_len = replacement_lower.chars().count();
    original_len <= 6
        && original_len == replacement_len
        && damerau_levenshtein(&original_lower, &replacement_lower) >= 2
}

fn same_tail_single_consonant_drift(
    original: &str,
    replacement: &str,
    error_class: TypingErrorClass,
) -> bool {
    if !matches!(
        error_class,
        TypingErrorClass::CompositeTypo | TypingErrorClass::GrammarAgreement
    ) {
        return false;
    }
    let Some(original_word) = last_text_word(original) else {
        return false;
    };
    let Some(replacement_word) = last_text_word(replacement) else {
        return false;
    };
    if !is_cyrillic_letters_only(&original_word) || !is_cyrillic_letters_only(&replacement_word) {
        return false;
    }
    let original_lower = original_word.to_lowercase();
    let replacement_lower = replacement_word.to_lowercase();
    let original_chars = original_lower.chars().collect::<Vec<_>>();
    let replacement_chars = replacement_lower.chars().collect::<Vec<_>>();
    if original_chars.len() < 6
        || original_chars.len() != replacement_chars.len()
        || damerau_levenshtein(&original_lower, &replacement_lower) != 1
    {
        return false;
    }
    let diffs = original_chars
        .iter()
        .zip(&replacement_chars)
        .enumerate()
        .filter(|(_, (left, right))| left != right)
        .collect::<Vec<_>>();
    let [(idx, (left, right))] = diffs.as_slice() else {
        return false;
    };
    *idx > 1
        && *idx + 2 < original_chars.len()
        && is_russian_consonant(**left)
        && is_russian_consonant(**right)
        && original_chars[original_chars.len() - 2..]
            == replacement_chars[replacement_chars.len() - 2..]
}

fn is_russian_consonant(ch: char) -> bool {
    matches!(ch, 'а'..='я' | 'ё')
        && !crate::russian_chars::is_russian_vowel(ch)
        && !matches!(ch, 'ь' | 'ъ')
}

fn short_layout_candidate_lacks_phrase_context(
    original: &str,
    replacement: &str,
    error_class: TypingErrorClass,
    origin: CandidateOrigin,
) -> bool {
    if !matches!(
        origin,
        CandidateOrigin::Layout | CandidateOrigin::LayoutThenTypo
    ) || !matches!(
        error_class,
        TypingErrorClass::WrongLayout
            | TypingErrorClass::PartialLayout
            | TypingErrorClass::MixedScript
    ) {
        return false;
    }
    let Some(original_word) = last_text_word(original) else {
        return false;
    };
    let Some(replacement_word) = last_text_word(replacement) else {
        return false;
    };
    if original_word.chars().count() != 1 || replacement_word.chars().count() != 1 {
        return false;
    }
    let (_, original_core, _) = split_edge_whitespace(original);
    let previous_words = original_core
        .split_whitespace()
        .take(original_core.split_whitespace().count().saturating_sub(1))
        .collect::<Vec<_>>();
    let has_cyrillic_context = previous_words.iter().any(|word| has_cyrillic(word));
    let has_ascii_context = previous_words
        .iter()
        .any(|word| word.chars().any(|ch| ch.is_ascii_alphabetic()));
    let immediate_entity_context = previous_words.last().is_some_and(|word| {
        crate::word_recognizer::is_ascii_titlecase_token(word)
            || crate::word_recognizer::is_ascii_technical_or_brand_token(word)
    });

    has_ascii_context && !has_cyrillic_context && !immediate_entity_context
}

fn short_cyrillic_word_switches_to_ascii_layout(
    original: &str,
    replacement: &str,
    error_class: TypingErrorClass,
    origin: CandidateOrigin,
) -> bool {
    if error_class != TypingErrorClass::WrongLayout {
        return false;
    }
    let Some(original_word) = last_text_word(original) else {
        return false;
    };
    let Some(replacement_word) = last_text_word(replacement) else {
        return false;
    };
    let exact_known_layout_center = origin == CandidateOrigin::Layout
        && crate::dict::convert(&original_word, crate::dict::Direction::Ru2Us)
            .eq_ignore_ascii_case(&replacement_word)
        && crate::layout_autoswitch::is_known_english_layout_autoswitch_word(
            &replacement_word.to_ascii_lowercase(),
        );
    original_word.chars().count() <= 3
        && original_word
            .chars()
            .any(|ch| matches!(ch, 'а'..='я' | 'ё' | 'А'..='Я' | 'Ё'))
        && replacement_word
            .chars()
            .all(|ch| ch.is_ascii_alphabetic() || matches!(ch, '`'))
        && !exact_known_layout_center
}

fn short_nanda_composite_candidate_shrinks_word(
    original: &str,
    replacement: &str,
    error_class: TypingErrorClass,
    origin: CandidateOrigin,
) -> bool {
    if !matches!(error_class, TypingErrorClass::CompositeTypo) || !origin.is_surface_or_context() {
        return false;
    }
    let Some(original_word) = last_text_word(original) else {
        return false;
    };
    let Some(replacement_word) = last_text_word(replacement) else {
        return false;
    };
    if !is_cyrillic_letters_only(&original_word) || !is_cyrillic_letters_only(&replacement_word) {
        return false;
    }
    let original_len = original_word.chars().count();
    let replacement_len = replacement_word.chars().count();
    original_len <= 4 && replacement_len < original_len
}

fn nanda_surface_candidate_outputs_unknown_word(
    original: &str,
    replacement: &str,
    error_class: TypingErrorClass,
    origin: CandidateOrigin,
) -> bool {
    if !matches!(error_class, TypingErrorClass::CompositeTypo) || !origin.is_surface_or_context() {
        return false;
    }
    let Some(original_word) = last_text_word(original) else {
        return false;
    };
    let Some(replacement_word) = last_text_word(replacement) else {
        return false;
    };
    if original_word == replacement_word || !is_cyrillic_letters_only(&replacement_word) {
        return false;
    }
    let replacement_lower = replacement_word.to_lowercase();
    !crate::russian_lexicon::is_known_russian_word_or_form(&replacement_lower)
        && !crate::lexicon::is_common_ru_word(&replacement_lower)
        && !crate::phrase_lexicon::is_known_russian_phrase_part(&replacement_lower)
}

fn nanda_surface_candidate_outputs_unknown_word_with_facts(
    original: &str,
    replacement: &str,
    error_class: TypingErrorClass,
    origin: CandidateOrigin,
    facts: &AdmissionLexicalFacts<'_>,
) -> bool {
    if !facts.reuses_facts() {
        return nanda_surface_candidate_outputs_unknown_word(
            original,
            replacement,
            error_class,
            origin,
        );
    }
    facts.assert_pair(original, replacement);
    if !matches!(error_class, TypingErrorClass::CompositeTypo) || !origin.is_surface_or_context() {
        return false;
    }
    let Some(original_word) = facts.original_word() else {
        return false;
    };
    let Some(replacement_word) = facts.replacement_word() else {
        return false;
    };
    if original_word.word() == replacement_word.word()
        || !replacement_word.is_cyrillic_letters_only()
    {
        return false;
    }
    !crate::russian_lexicon::is_known_russian_word_or_form(replacement_word.lower())
        && !crate::lexicon::is_common_ru_word(replacement_word.lower())
        && !crate::phrase_lexicon::is_known_russian_phrase_part(replacement_word.lower())
}

fn short_nanda_candidate_inserts_internal_vowel(
    original: &str,
    replacement: &str,
    error_class: TypingErrorClass,
    origin: CandidateOrigin,
) -> bool {
    if !matches!(error_class, TypingErrorClass::CompositeTypo) || !origin.is_surface_or_context() {
        return false;
    }
    let Some(original_word) = last_text_word(original) else {
        return false;
    };
    let Some(replacement_word) = last_text_word(replacement) else {
        return false;
    };
    if !is_cyrillic_letters_only(&original_word) || !is_cyrillic_letters_only(&replacement_word) {
        return false;
    }
    let original_lower = original_word.to_lowercase();
    let replacement_lower = replacement_word.to_lowercase();
    if original_lower.chars().count() > 6
        || damerau_levenshtein(&original_lower, &replacement_lower) != 1
    {
        return false;
    }
    let Some((idx, inserted)) =
        inserted_char_position_for_missing_letter(&original_lower, &replacement_lower)
    else {
        return false;
    };
    idx > 0 && crate::russian_chars::is_russian_vowel(inserted)
}

fn same_known_russian_token(original: &str, candidate: &str) -> bool {
    let (_, original_word, _) = split_word_punctuation(original);
    let (_, candidate_word, _) = split_word_punctuation(candidate);
    if original_word.is_empty()
        || candidate_word.is_empty()
        || !is_cyrillic_letters_only(original_word)
        || !is_cyrillic_letters_only(candidate_word)
    {
        return false;
    }
    let original_lower = original_word.to_lowercase();
    original_lower == candidate_word.to_lowercase()
        && (crate::nanda_wave::l2::l2_surface_foundation_contains(&original_lower)
            || crate::russian_lexicon::is_reference_backed_russian_form(&original_lower)
            || crate::lexicon::is_common_ru_word(&original_lower)
            || crate::lexicon::is_ru_technical_loanword(&original_lower))
}

fn strong_standalone_split_tail(lower: &str) -> bool {
    let len = lower.chars().count();
    (len >= 3 && crate::lexicon::is_common_ru_word(lower))
        || (len >= 4
            && (crate::russian_lexicon::russian_dictionary().contains(lower)
                || crate::russian_lexicon::is_known_russian_adverb_o_form(lower)
                || crate::russian_lexicon::is_known_russian_ka_oblique_form(lower)))
}

fn core_word_count(text: &str) -> usize {
    let (_, core, _) = split_edge_whitespace(text);
    core.split_whitespace().count()
}

fn replacement_last_word_is_unknown_cyrillic(original: &str, replacement: &str) -> bool {
    let Some(original_word) = last_text_word(original) else {
        return false;
    };
    let Some(replacement_word) = last_text_word(replacement) else {
        return false;
    };
    if original_word == replacement_word || !is_cyrillic_letters_only(&replacement_word) {
        return false;
    }
    let original_lower = original_word.to_lowercase();
    let replacement_lower = replacement_word.to_lowercase();
    if repeated_deletion_has_surface_support(&original_lower, &replacement_lower) {
        return false;
    }
    !crate::russian_lexicon::is_known_russian_word_or_form(&replacement_lower)
        && !crate::lexicon::is_common_ru_word(&replacement_lower)
}

pub(crate) fn repeated_deletion_has_surface_support(
    original_lower: &str,
    replacement_lower: &str,
) -> bool {
    if !repeated_run_deletion_candidates(original_lower)
        .into_iter()
        .any(|candidate| candidate == replacement_lower)
    {
        return false;
    }
    crate::russian_lexicon::is_known_russian_word_or_form(replacement_lower)
        || crate::lexicon::is_common_ru_word(replacement_lower)
        || short_final_repeated_vowel_delete_has_surface_support(original_lower, replacement_lower)
}

fn short_final_repeated_vowel_delete_has_surface_support(
    original_lower: &str,
    replacement_lower: &str,
) -> bool {
    let original_chars = original_lower.chars().collect::<Vec<_>>();
    let replacement_chars = replacement_lower.chars().collect::<Vec<_>>();
    if original_chars.len() > 5 || original_chars.len() != replacement_chars.len() + 1 {
        return false;
    }
    let Some(&last) = original_chars.last() else {
        return false;
    };
    if last != 'и'
        || !crate::russian_chars::is_russian_vowel(last)
        || original_chars
            .get(original_chars.len().saturating_sub(2))
            .copied()
            != Some(last)
    {
        return false;
    }
    replacement_chars.as_slice() == &original_chars[..original_chars.len() - 1]
        && crate::russian_typo_scoring::ngram_allows_ru_candidate(
            replacement_lower,
            original_lower,
            REPEATED_DELETE_SURFACE_MARGIN,
        )
}

pub(crate) fn should_prefer_composite_after_repeated_repair(
    original: &str,
    single_step: &str,
    composite: &str,
) -> bool {
    let Some(original_word) = last_text_word(original) else {
        return false;
    };
    let Some(single_word) = last_text_word(single_step) else {
        return false;
    };
    let Some(composite_word) = last_text_word(composite) else {
        return false;
    };
    let original_lower = original_word.to_lowercase();
    let single_lower = single_word.to_lowercase();
    let composite_lower = composite_word.to_lowercase();
    if single_lower == composite_lower || !is_cyrillic_letters_only(&composite_word) {
        return false;
    }
    if single_word.chars().count() < original_word.chars().count()
        && composite_word.chars().count() > original_word.chars().count()
        && damerau_levenshtein(&original_lower, &composite_lower) <= 1
        && crate::russian_lexicon::is_known_russian_word_or_form(&composite_lower)
    {
        return true;
    }
    repeated_run_deletion_candidates(&original_lower)
        .into_iter()
        .any(|candidate| candidate == single_lower)
        && composite_word.chars().count() > original_word.chars().count()
        && damerau_levenshtein(&single_lower, &composite_lower) <= 1
        && crate::russian_lexicon::is_known_russian_word_or_form(&composite_lower)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Copy)]
    struct AdmissionFixture {
        original: &'static str,
        replacement: &'static str,
        error_class: TypingErrorClass,
        origin: CandidateOrigin,
    }

    const EXISTING_ADMISSION_FIXTURES: &[AdmissionFixture] = &[
        AdmissionFixture {
            original: "40 000 р ",
            replacement: "40 000 h ",
            error_class: TypingErrorClass::WrongLayout,
            origin: CandidateOrigin::DeterministicTypo,
        },
        AdmissionFixture {
            original: "Екб ",
            replacement: "Tr, ",
            error_class: TypingErrorClass::WrongLayout,
            origin: CandidateOrigin::DeterministicTypo,
        },
        AdmissionFixture {
            original: "дфн ",
            replacement: "lay ",
            error_class: TypingErrorClass::WrongLayout,
            origin: CandidateOrigin::Layout,
        },
        AdmissionFixture {
            original: "в коде ",
            replacement: "в код ",
            error_class: TypingErrorClass::ExtraLetter,
            origin: CandidateOrigin::L2Surface,
        },
        AdmissionFixture {
            original: "закинем ",
            replacement: "закон ",
            error_class: TypingErrorClass::CompositeTypo,
            origin: CandidateOrigin::L2Surface,
        },
        AdmissionFixture {
            original: "китайцы ",
            replacement: "китайы ",
            error_class: TypingErrorClass::ExtraLetter,
            origin: CandidateOrigin::L2Surface,
        },
        AdmissionFixture {
            original: "ходу ",
            replacement: "ход ",
            error_class: TypingErrorClass::ExtraLetter,
            origin: CandidateOrigin::L2Surface,
        },
        AdmissionFixture {
            original: "делаем ",
            replacement: "деваем ",
            error_class: TypingErrorClass::LetterSubstitution,
            origin: CandidateOrigin::L2Surface,
        },
        AdmissionFixture {
            original: "допусти мнабираю ",
            replacement: "допустим набираю ",
            error_class: TypingErrorClass::BoundaryShift,
            origin: CandidateOrigin::Boundary,
        },
        AdmissionFixture {
            original: "я думаю допусти мнабираю ",
            replacement: "я думаю допустим набираю ",
            error_class: TypingErrorClass::BoundaryShift,
            origin: CandidateOrigin::Boundary,
        },
        AdmissionFixture {
            original: "тоесть ",
            replacement: "есть ",
            error_class: TypingErrorClass::CompositeTypo,
            origin: CandidateOrigin::L2Surface,
        },
        AdmissionFixture {
            original: "что получилось содержкой ",
            replacement: "что получилось содержать ",
            error_class: TypingErrorClass::CompositeTypo,
            origin: CandidateOrigin::L2Surface,
        },
        AdmissionFixture {
            original: "патерна ",
            replacement: "пара ",
            error_class: TypingErrorClass::CompositeTypo,
            origin: CandidateOrigin::L3Context,
        },
        AdmissionFixture {
            original: "я прохоил ",
            replacement: "я проход ",
            error_class: TypingErrorClass::CompositeTypo,
            origin: CandidateOrigin::L2Surface,
        },
        AdmissionFixture {
            original: "ответили вчате ",
            replacement: "ответили вате ",
            error_class: TypingErrorClass::CompositeTypo,
            origin: CandidateOrigin::L3Context,
        },
        AdmissionFixture {
            original: "будет примать ",
            replacement: "будет придать ",
            error_class: TypingErrorClass::CompositeTypo,
            origin: CandidateOrigin::L3Context,
        },
        AdmissionFixture {
            original: "видешь ",
            replacement: "видишь ",
            error_class: TypingErrorClass::LetterSubstitution,
            origin: CandidateOrigin::L2Surface,
        },
        AdmissionFixture {
            original: "дожь ",
            replacement: "дождь ",
            error_class: TypingErrorClass::MissingLetter,
            origin: CandidateOrigin::L2Surface,
        },
        AdmissionFixture {
            original: "твой ",
            replacement: "тывой ",
            error_class: TypingErrorClass::CompositeTypo,
            origin: CandidateOrigin::DeterministicTypo,
        },
        AdmissionFixture {
            original: "что нравится? ",
            replacement: "что нравиться? ",
            error_class: TypingErrorClass::MissingLetter,
            origin: CandidateOrigin::L2Surface,
        },
        AdmissionFixture {
            original: "Читал логи ",
            replacement: "Читал логик ",
            error_class: TypingErrorClass::CompositeTypo,
            origin: CandidateOrigin::L2Surface,
        },
        AdmissionFixture {
            original: "смотри, ",
            replacement: "смотори, ",
            error_class: TypingErrorClass::CompositeTypo,
            origin: CandidateOrigin::DeterministicTypo,
        },
        AdmissionFixture {
            original: "давай там посмотри ",
            replacement: "давай там просмотри ",
            error_class: TypingErrorClass::MissingLetter,
            origin: CandidateOrigin::L2Surface,
        },
        AdmissionFixture {
            original: "посмотри ",
            replacement: "посмотреть ",
            error_class: TypingErrorClass::CompositeTypo,
            origin: CandidateOrigin::L2Surface,
        },
        AdmissionFixture {
            original: "искать хрень! ",
            replacement: "искать хрену ",
            error_class: TypingErrorClass::CompositeTypo,
            origin: CandidateOrigin::DeterministicTypo,
        },
        AdmissionFixture {
            original: "тели ",
            replacement: "тел ",
            error_class: TypingErrorClass::CompositeTypo,
            origin: CandidateOrigin::L2Surface,
        },
        AdmissionFixture {
            original: "нас моного ",
            replacement: "нас мюоного ",
            error_class: TypingErrorClass::CompositeTypo,
            origin: CandidateOrigin::L2Surface,
        },
    ];

    #[test]
    fn lexical_fact_reuse_preserves_existing_fixture_decisions_under_both_authorities() {
        let previous_policy = crate::hot_field::process_policy();
        for policy in [
            crate::hot_field::HotFieldPolicy::ime(),
            crate::hot_field::HotFieldPolicy::daemon_for_text_backend(
                crate::text_backend::TextBackendPreference::Uinput,
            ),
        ] {
            crate::hot_field::set_process_policy(policy);
            for fixture in EXISTING_ADMISSION_FIXTURES {
                let uncached = with_admission_fact_reuse(false, || {
                    candidate_admission(
                        fixture.original,
                        fixture.replacement,
                        fixture.error_class,
                        fixture.origin,
                    )
                });
                let reused = with_admission_fact_reuse(true, || {
                    candidate_admission(
                        fixture.original,
                        fixture.replacement,
                        fixture.error_class,
                        fixture.origin,
                    )
                });
                assert_eq!(
                    reused, uncached,
                    "policy={policy:?} original={:?} replacement={:?} class={:?} origin={:?}",
                    fixture.original, fixture.replacement, fixture.error_class, fixture.origin
                );
            }
        }
        crate::hot_field::set_process_policy(previous_policy);
    }

    #[test]
    fn lexical_fact_owner_is_lazy_call_local_and_uncached_mode_retains_nothing() {
        let unchanged = AdmissionLexicalFacts::with_mode("слово ", "слово ", true);
        let decision = candidate_admission_with_facts(
            "слово ",
            "слово ",
            TypingErrorClass::LetterSubstitution,
            CandidateOrigin::DeterministicTypo,
            &unchanged,
        );
        assert_eq!(decision.reason, "unchanged");
        assert_eq!(
            unchanged.snapshot(),
            AdmissionLexicalFactSnapshot::default()
        );

        let uncached = AdmissionLexicalFacts::with_mode("посмотри ", "посмотреть ", false);
        let _ = candidate_admission_with_facts(
            "посмотри ",
            "посмотреть ",
            TypingErrorClass::CompositeTypo,
            CandidateOrigin::L2Surface,
            &uncached,
        );
        assert_eq!(uncached.snapshot(), AdmissionLexicalFactSnapshot::default());

        let reused = AdmissionLexicalFacts::with_mode("посмотри ", "посмотреть ", true);
        let _ = candidate_admission_with_facts(
            "посмотри ",
            "посмотреть ",
            TypingErrorClass::CompositeTypo,
            CandidateOrigin::L2Surface,
            &reused,
        );
        let snapshot = reused.snapshot();
        assert!(snapshot.original_word && snapshot.replacement_word);
        assert!(snapshot.original_lower && snapshot.replacement_lower);
        assert!(snapshot.original_known && snapshot.replacement_known);
        assert!(snapshot.original_protected);
        assert!(!snapshot.replacement_protected);

        let next_call = AdmissionLexicalFacts::with_mode("дожь ", "дождь ", true);
        assert_eq!(
            next_call.snapshot(),
            AdmissionLexicalFactSnapshot::default()
        );
    }

    #[test]
    fn short_cyrillic_to_ascii_layout_is_never_applyable_from_logs() {
        for (original, replacement) in [("40 000 р ", "40 000 h "), ("Екб ", "Tr, ")] {
            let gate = gate_candidate(original, replacement, TypingErrorClass::WrongLayout);

            assert_eq!(gate.action, CandidateGateAction::KeepOriginal);
            assert_eq!(gate.reason, "short_cyrillic_to_ascii_layout");
        }
    }

    #[test]
    fn exact_short_layout_projection_to_known_english_center_is_eligible() {
        let gate = gate_candidate_with_origin(
            "дфн ",
            "lay ",
            TypingErrorClass::WrongLayout,
            CandidateOrigin::Layout,
        );

        assert_eq!(gate.action, CandidateGateAction::Eligible, "{gate:?}");
    }

    #[test]
    fn l2_cannot_delete_a_known_inflection_without_context_authority() {
        let gate = gate_candidate_with_origin(
            "в коде ",
            "в код ",
            TypingErrorClass::ExtraLetter,
            CandidateOrigin::L2Surface,
        );

        assert_eq!(gate.action, CandidateGateAction::SuggestOnly);
        assert_eq!(gate.reason, "known_current_word_surface_drift");
    }

    #[test]
    fn l2_cannot_rewrite_known_russian_surfaces_from_live_log() {
        for (original, replacement, error_class) in [
            ("закинем ", "закон ", TypingErrorClass::CompositeTypo),
            ("китайцы ", "китайы ", TypingErrorClass::ExtraLetter),
            ("ходу ", "ход ", TypingErrorClass::ExtraLetter),
            ("делаем ", "деваем ", TypingErrorClass::LetterSubstitution),
        ] {
            let gate = gate_candidate_with_origin(
                original,
                replacement,
                error_class,
                CandidateOrigin::L2Surface,
            );

            assert_eq!(
                gate.action,
                CandidateGateAction::SuggestOnly,
                "{original:?} -> {replacement:?}: {gate:?}"
            );
            assert_ne!(
                gate.reason, "class_allows_apply",
                "{original:?} -> {replacement:?}: {gate:?}"
            );
        }
    }

    #[test]
    fn boundary_shift_tail_pair_full_text_is_eligible() {
        for (original, replacement) in [
            ("допусти мнабираю ", "допустим набираю "),
            ("я думаю допусти мнабираю ", "я думаю допустим набираю "),
        ] {
            let gate = gate_candidate_with_origin(
                original,
                replacement,
                TypingErrorClass::BoundaryShift,
                CandidateOrigin::Boundary,
            );

            assert_eq!(gate.action, CandidateGateAction::Eligible, "{gate:?}");
        }
    }
}

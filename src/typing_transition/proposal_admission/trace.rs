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

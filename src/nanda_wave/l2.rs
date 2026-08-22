use super::context::{TailContext, TokenKind};
use super::feedback::{apply_l3_feedback, L3Feedback};
use super::lexical_attractor::{lexical_attractor_candidates, LEXICAL_ATTRACTOR_CELL};
use super::lexical_phase::{default_memory, LexicalPhaseCandidate, LexicalPhaseMemory};
use super::options::WaveOptions;
use super::signal::{WavePacket, WordCandidate};
use super::PHRASE_FORECAST_CELL;
use crate::candidate_contract::CandidateOrigin;
use crate::keyboard::is_cyrillic_letter;
use crate::lexicon::{
    is_common_en_technical_word, is_common_ru_word, is_ru_live_protected_word,
    is_user_protected_word,
};
use crate::russian_lexicon::is_known_russian_word_or_form;
use crate::text_case::apply_word_case;
use crate::text_metrics::damerau_levenshtein;
use crate::word_reader::{split_last_ws_token, split_word_punctuation};

#[path = "l2/grammar_adapter.rs"]
mod grammar_adapter;
#[path = "l2/hot_memory.rs"]
mod hot_memory;
#[path = "l2/ime_readout.rs"]
mod ime_readout;
#[path = "l2/layout_adapter.rs"]
mod layout_adapter;
#[path = "l2/phase.rs"]
mod phase;
#[path = "l2/tail_scan_adapter.rs"]
mod tail_scan_adapter;
#[path = "l2/taught_adapter.rs"]
mod taught_adapter;
use grammar_adapter::grammar_agreement_candidates;
pub use hot_memory::{
    ime_word_candidate_memory_is_warm, l2_surface_memory_status, L2SurfaceMemoryStatus,
};
pub(crate) use hot_memory::{warm_up_ime_word_candidate_memory, warm_up_surface_motif_memory};
pub(crate) use ime_readout::{
    l2_center_contains_surface, l2_center_near_surfaces, l2_decoder_contains_surface,
    l2_surface_phase_readout,
};
use layout_adapter::{
    layout_candidate, layout_scan_candidates, layout_sequence_candidate, short_token_candidates,
};
#[cfg(test)]
use layout_adapter::{layout_candidate_allowed, LAYOUT_THEN_L2_WORD_CENTER};
use phase::apply_l2_phase_shadow;
use tail_scan_adapter::{
    boundary_scan_candidates, boundary_split_candidates, surface_motif_scan_candidates,
};
use taught_adapter::{should_run_taught_candidates, taught_candidates};

#[path = "l2/surface.rs"]
mod surface;
use surface::*;
pub(crate) use surface::{
    l2_surface_foundation_contains, l2_surface_foundation_has_authority, l2_surface_foundation_rank,
};

const L2_ACTIVE_SOURCE_TARGET: usize = 1_000_000;
pub(super) const L2_SURFACE_MOTIF_CELL: &str = "L2SurfaceMotifCell32";
pub(super) const L2_SURFACE_COMPLETION_CELL: &str = "L2SurfaceCompletionCell32";
const L2_FORM_ATTRACTOR_LIMIT: usize = 20;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum L2ImeWordCandidateKind {
    AdjacentTransposition,
    Completion,
    Replacement,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum L2ImeWordCandidateSource {
    LexicalPhase,
    /// A grounded surface projected from the canonical L1.1 -> Productive V90
    /// field. This is candidate evidence, not a second IME ranking owner.
    CanonicalField,
    /// Exact keyboard-layout projection. It must compete before a projection
    /// that also requires lexical typo repair.
    ExactLayoutPhase,
    /// Keyboard-layout projection followed by lexical settling.
    LayoutThenTypoPhase,
    /// A longer lexical center reached through one Damerau edit of the active
    /// prefix. It is display-only and requires explicit IME acceptance.
    CorrectedPrefixPhase,
    /// A bounded next-word center born by online L3 context memory and
    /// independently verified against the L2 decoder.
    ContextPhase,
    /// A two-center boundary candidate born by the same L1/L2 field as the
    /// full correction route.  It stays a display-only replacement in IME.
    BoundaryPhase,
}

pub(crate) use crate::typing_transition::target_evidence::L2ImeTargetEvidence;

impl L2ImeWordCandidateSource {
    pub(crate) const fn is_lexically_grounded(self) -> bool {
        matches!(
            self,
            Self::LexicalPhase
                | Self::CanonicalField
                | Self::ExactLayoutPhase
                | Self::LayoutThenTypoPhase
        )
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct L2ImeWordCandidate {
    pub surface: String,
    pub kind: L2ImeWordCandidateKind,
    pub source: L2ImeWordCandidateSource,
    pub score: u32,
    pub l1_overlap: usize,
    pub l2_overlap: usize,
    pub motif_overlap: usize,
    pub usage_prior: f32,
    pub context_prior: f32,
    pub accepted_count: u32,
    pub(crate) target_evidence: L2ImeTargetEvidence,
    pub(crate) morphology_slots: Vec<crate::correction_core::MorphologySlotIdentity>,
}

impl L2ImeWordCandidate {
    pub(crate) fn common_target_evidence(
        &self,
    ) -> crate::typing_transition::target_evidence::TargetEvidenceSetV1 {
        self.target_evidence.to_common()
    }
}

pub fn ime_l2_word_candidates(
    context_prefix: &str,
    token: &str,
    limit: usize,
) -> Vec<L2ImeWordCandidate> {
    ime_readout::ime_l2_word_candidates_impl(context_prefix, token, limit)
}

/// Supplies compact BoundaryCell32 proposals to the live IME lattice.
///
/// The full L2 route already births split/glue candidates through
/// `boundary_split_candidates`.  Keeping this projection here prevents the
/// IME-only lexical readout from silently losing a valid L2 operator.
pub fn ime_l2_boundary_candidates(
    context_prefix: &str,
    token: &str,
    limit: usize,
) -> Vec<L2ImeWordCandidate> {
    let started = std::time::Instant::now();
    if limit == 0 {
        return Vec::new();
    }
    let normalized = token.to_lowercase();
    if !(4..=18).contains(&normalized.chars().count())
        || !normalized.chars().all(is_cyrillic_letter)
    {
        return Vec::new();
    }
    let mut tail = context_prefix.trim_end().to_string();
    if !tail.is_empty() {
        tail.push(' ');
    }
    tail.push_str(token);
    let context = TailContext::from_text(&tail);
    let context_ready = std::time::Instant::now();
    let l1 = super::l1::run_l1(token);
    let mut candidates = boundary_split_candidates("", token, &l1, &context)
        .into_iter()
        .map(|candidate| L2ImeWordCandidate {
            surface: candidate.text,
            kind: L2ImeWordCandidateKind::Replacement,
            source: L2ImeWordCandidateSource::BoundaryPhase,
            // BoundaryCell32 has already proved two lexical centers.  Keep
            // its field strength explicit for common live arbitration.
            score: (candidate.energy * 1_600.0).round() as u32,
            l1_overlap: normalized.chars().count().min(10),
            l2_overlap: 4,
            motif_overlap: 2,
            usage_prior: 0.0,
            context_prior: 0.0,
            accepted_count: 0,
            target_evidence: L2ImeTargetEvidence::Boundary,
            morphology_slots: Vec::new(),
        })
        .collect::<Vec<_>>();
    let split_ready = std::time::Instant::now();
    candidates.sort_by(|left, right| {
        right
            .score
            .cmp(&left.score)
            .then_with(|| left.surface.cmp(&right.surface))
    });
    candidates.truncate(limit);
    if std::env::var_os("LAY_L2_FIELD_TRACE").is_some() {
        eprintln!(
            "l2_boundary_projection_trace context_us={} split_us={} settle_us={} candidates={}",
            context_ready.duration_since(started).as_micros(),
            split_ready.duration_since(context_ready).as_micros(),
            std::time::Instant::now()
                .duration_since(split_ready)
                .as_micros(),
            candidates.len(),
        );
    }
    candidates
}

pub(crate) fn ime_l2_boundary_evidence(token: &str) -> bool {
    tail_scan_adapter::boundary_split_has_structural_evidence(token)
}

pub(crate) fn ime_l2_boundary_target_evidence(token: &str, surface: &str) -> bool {
    tail_scan_adapter::boundary_split_target_has_structural_evidence(token, surface)
}

pub fn correction_l2_word_candidates(
    context_prefix: &str,
    token: &str,
    limit: usize,
) -> Vec<L2ImeWordCandidate> {
    ime_readout::correction_l2_word_candidates_impl(context_prefix, token, limit)
}

/// Settles one projected Cyrillic surface into the strongest admitted L2 form center.
///
/// This is intentionally narrower than `run_l2`: layout projection can reuse the
/// lexical field without recursively invoking layout candidate generation.
pub(crate) fn l2_settle_russian_surface(surface: &str) -> Option<String> {
    surface::settle_russian_surface(surface)
}

pub fn run_l2(original: &str, l1: &[WavePacket]) -> Vec<WordCandidate> {
    run_l2_with_options(original, l1, &WaveOptions::default())
}

pub fn run_l2_with_options(
    original: &str,
    l1: &[WavePacket],
    options: &WaveOptions,
) -> Vec<WordCandidate> {
    run_l2_refined_with_feedback(original, l1, options, &L3Feedback::default())
}

pub(crate) fn hot_layout_candidate(original: &str) -> Option<WordCandidate> {
    hot_layout_candidate_with_noisy_projection(original, true)
}

/// Resolve a projected ASCII surface through the compact English L2 center.
/// The verifier uses this to prove a typed layout transition rather than
/// treating layout-plus-typo recovery as an untyped word replacement.
pub(crate) fn l2_settle_english_surface(surface: &str) -> Option<String> {
    layout_adapter::settle_english_word_center(surface)
}

pub(crate) fn hot_layout_candidate_with_noisy_projection(
    original: &str,
    allow_noisy_projection: bool,
) -> Option<WordCandidate> {
    let tail = original.trim_end();
    if tail.is_empty() {
        return None;
    }
    let (prefix, token) = split_last_ws_token(tail).unwrap_or(("", tail));
    if token.trim().is_empty() {
        return None;
    }
    let context = TailContext::from_text(tail);
    layout_adapter::layout_candidate_with_projection_policy(
        prefix,
        token,
        &context,
        &[],
        allow_noisy_projection,
    )
}

pub fn run_l2_refined_with_feedback(
    original: &str,
    l1: &[WavePacket],
    options: &WaveOptions,
    feedback: &L3Feedback,
) -> Vec<WordCandidate> {
    let tail = original.trim_end();
    if tail.is_empty() {
        return Vec::new();
    }
    // The first word is a complete L2 scene too. Requiring a preceding space
    // made the entire candidate lattice disappear until the user typed a
    // second token or pressed Backspace, which is why IME seemed to wake up
    // only after deletion.
    let (prefix, token) = split_last_ws_token(tail).unwrap_or(("", tail));
    if token.trim().is_empty() {
        return Vec::new();
    }
    let context = TailContext::from_text(tail);
    let mut candidates = Vec::new();
    let timing_enabled = std::env::var_os("LAY_NANDA_L2_TIMING").is_some();
    let mut timing_last = std::time::Instant::now();
    macro_rules! mark_timing {
        ($stage:literal) => {
            if timing_enabled {
                let now = std::time::Instant::now();
                eprintln!(
                    "lay_nanda_l2_timing stage={} elapsed_us={} candidates={}",
                    $stage,
                    now.duration_since(timing_last).as_micros(),
                    candidates.len()
                );
                timing_last = now;
            }
        };
    }
    if options.is_enabled("LayoutWordCell32") {
        if let Some(candidate) = layout_sequence_candidate(tail, &context, l1) {
            push_unique_candidate(&mut candidates, candidate);
        }
        if let Some(candidate) = layout_candidate(prefix, token, &context, l1) {
            push_unique_candidate(&mut candidates, candidate);
        }
        for candidate in layout_scan_candidates(tail, &context, l1) {
            push_unique_candidate(&mut candidates, candidate);
        }
    }
    mark_timing!("layout");
    if options.is_enabled("ShortTokenCell32") {
        for candidate in short_token_candidates(prefix, token, &context, l1) {
            push_unique_candidate(&mut candidates, candidate);
        }
    }
    mark_timing!("short-token");
    if options.is_enabled("TechTokenCell32") {
        if let Some(candidate) = technical_keep_candidate(token, l1) {
            push_unique_candidate(&mut candidates, candidate);
        }
        if let Some(candidate) = technical_context_keep_candidate(tail, l1) {
            push_unique_candidate(&mut candidates, candidate);
        }
    }
    mark_timing!("tech-token");
    let boundary_scan = if options.is_enabled("BoundaryCell32") {
        boundary_scan_candidates(tail, l1, &context)
    } else {
        Vec::new()
    };
    if options.is_enabled(L2_SURFACE_MOTIF_CELL) || options.is_enabled(L2_SURFACE_COMPLETION_CELL) {
        for candidate in surface_motif_word_candidates(prefix, token, &context, l1, options) {
            push_unique_candidate(&mut candidates, candidate);
        }
        if options.is_enabled(LEXICAL_ATTRACTOR_CELL) {
            for candidate in form_attractor_word_candidates(prefix, token, &context, l1, options) {
                push_unique_candidate(&mut candidates, candidate);
            }
        }
        if options.is_enabled(L2_SURFACE_MOTIF_CELL) {
            for candidate in surface_motif_scan_candidates(tail, l1, &context) {
                push_unique_candidate(&mut candidates, candidate);
            }
        }
    }
    mark_timing!("surface-motif");
    if options.is_enabled("BoundaryCell32") {
        for candidate in boundary_split_candidates(prefix, token, l1, &context) {
            push_unique_candidate(&mut candidates, candidate);
        }
        for candidate in boundary_scan {
            push_unique_candidate(&mut candidates, candidate);
        }
    }
    mark_timing!("boundary");
    if options.is_enabled(LEXICAL_ATTRACTOR_CELL) {
        for candidate in lexical_attractor_candidates(tail, &context) {
            push_unique_candidate(&mut candidates, candidate);
        }
    }
    mark_timing!("lexical-attractor");
    mark_timing!("phrase");
    if options.is_enabled(PHRASE_FORECAST_CELL) && options.llmwave_shadow() {
        let memory = phrase_forecast_memory();
        for candidate in super::llmwave::phrase_forecast_candidates(tail, &memory) {
            push_unique_candidate(&mut candidates, candidate);
        }
    }
    mark_timing!("phrase-forecast");
    if options.is_enabled("GrammarCell32") {
        for candidate in grammar_agreement_candidates(tail, &context, l1) {
            push_unique_candidate(&mut candidates, candidate);
        }
    }
    mark_timing!("grammar");
    if should_run_taught_candidates(token, options) {
        for candidate in taught_candidates(tail, &context, l1, options) {
            push_unique_candidate(&mut candidates, candidate);
        }
    }
    mark_timing!("taught");
    apply_l2_phase_shadow(tail, &mut candidates, options);
    mark_timing!("l2-phase-shadow");
    apply_l2_weight(&mut candidates, options);
    candidates.sort_by(|left, right| {
        right
            .energy
            .total_cmp(&left.energy)
            .then_with(|| left.risk.total_cmp(&right.risk))
    });
    apply_l3_feedback(&mut candidates, feedback);
    candidates.sort_by(|left, right| {
        right
            .energy
            .total_cmp(&left.energy)
            .then_with(|| left.risk.total_cmp(&right.risk))
    });
    mark_timing!("feedback-sort");
    if timing_enabled {
        let _ = timing_last.elapsed();
    }
    candidates
}

fn apply_l2_weight(candidates: &mut [WordCandidate], options: &WaveOptions) {
    if (options.l2_weight() - 1.0).abs() < f32::EPSILON {
        return;
    }
    for candidate in candidates {
        candidate.energy = options.scale_l2_energy(candidate.energy);
        candidate
            .support
            .push(format!("l2-weight:{:.2}", options.l2_weight()));
    }
}

#[cfg(not(test))]
fn phrase_forecast_memory() -> super::llmwave::LlmWaveMemory {
    super::llmwave::load_default_memory()
}

#[cfg(test)]
fn phrase_forecast_memory() -> super::llmwave::LlmWaveMemory {
    super::llmwave::load_default_memory_uncached()
}

fn push_unique_candidate(candidates: &mut Vec<WordCandidate>, candidate: WordCandidate) {
    if candidates
        .iter()
        .any(|item| item.text == candidate.text && item.source == candidate.source)
    {
        return;
    }
    candidates.push(candidate);
}

fn technical_keep_candidate(token: &str, l1: &[WavePacket]) -> Option<WordCandidate> {
    if !is_common_en_technical_word(&token.to_ascii_lowercase()) {
        return None;
    }
    Some(WordCandidate {
        text: token.to_string(),
        origin: CandidateOrigin::Technical,
        source: "TechTokenCell32",
        energy: l1_energy(l1, "ScriptCell32").max(0.8),
        risk: 0.05,
        support: top_support(l1),
    })
}

fn technical_context_keep_candidate(text: &str, l1: &[WavePacket]) -> Option<WordCandidate> {
    if !looks_like_shell_or_technical_phrase(text) {
        return None;
    }
    Some(WordCandidate {
        text: text.to_string(),
        origin: CandidateOrigin::Technical,
        source: "TechTokenCell32",
        energy: l1_energy(l1, "ScriptCell32").max(0.92),
        risk: 0.02,
        support: top_support(l1),
    })
}

fn looks_like_shell_or_technical_phrase(text: &str) -> bool {
    let mut tokens = text.split_whitespace().peekable();
    let Some(first) = tokens.peek().copied() else {
        return false;
    };
    if !is_common_en_technical_word(&first.to_ascii_lowercase()) {
        return false;
    }
    text.contains(" -")
        || text.contains(" --")
        || text.contains("&&")
        || text.contains("://")
        || text.contains('/')
        || text.contains('=')
}

fn l1_energy(l1: &[WavePacket], cell: &str) -> f32 {
    l1.iter()
        .filter(|packet| packet.cell == cell)
        .map(WavePacket::top_energy)
        .fold(0.0, f32::max)
}

fn top_support(l1: &[WavePacket]) -> Vec<String> {
    l1.iter()
        .filter_map(|packet| packet.modes.first())
        .take(8)
        .map(|mode| mode.label())
        .collect()
}

fn candidate_support(l1: &[WavePacket], context: &TailContext) -> Vec<String> {
    let mut support = top_support(l1);
    support.push(format!("ctx:{}", context.phrase_signature()));
    support
}

#[cfg(test)]
mod target_evidence_adapter_tests {
    use super::*;

    #[test]
    fn l2_candidate_projects_the_exact_legacy_evidence_value() {
        for target_evidence in [
            L2ImeTargetEvidence::None,
            L2ImeTargetEvidence::LexicalReconstruction,
            L2ImeTargetEvidence::ContextBoundEdit,
            L2ImeTargetEvidence::CanonicalWinner,
            L2ImeTargetEvidence::LayoutRepair,
            L2ImeTargetEvidence::ExactLayout,
            L2ImeTargetEvidence::Boundary,
        ] {
            let candidate = L2ImeWordCandidate {
                surface: "candidate".to_string(),
                kind: L2ImeWordCandidateKind::Replacement,
                source: L2ImeWordCandidateSource::CanonicalField,
                score: 1,
                l1_overlap: 1,
                l2_overlap: 1,
                motif_overlap: 0,
                usage_prior: 0.0,
                context_prior: 0.0,
                accepted_count: 0,
                target_evidence,
                morphology_slots: Vec::new(),
            };

            assert_eq!(
                candidate.common_target_evidence(),
                target_evidence.to_common()
            );
        }
    }
}

#[cfg(test)]
mod tests;

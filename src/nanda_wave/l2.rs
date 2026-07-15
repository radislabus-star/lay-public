use super::context::{TailContext, TokenKind};
use super::feedback::{apply_l3_feedback, L3Feedback};
use super::lexical_attractor::{lexical_attractor_candidates, LEXICAL_ATTRACTOR_CELL};
use super::lexical_phase::{default_memory, LexicalPhaseCandidate, LexicalPhaseMemory};
use super::llmwave;
use super::options::WaveOptions;
use super::signal::{WavePacket, WordCandidate};
use crate::config::CorrectionSafety;
use crate::dict::{convert, detect_direction};
use crate::keyboard::is_cyrillic_letter;
use crate::lexicon::{
    is_common_en_guard_prefix, is_common_en_technical_word, is_common_ru_word,
    is_ru_live_protected_word, is_ru_one_letter_function_word, is_ru_short_function_word,
    is_user_protected_word, visual_b_after_ascii_replacement, visual_b_default_replacement,
};
use crate::russian_lexicon::is_known_russian_word_or_form;
use crate::text_case::apply_word_case;
use crate::text_metrics::damerau_levenshtein;
use crate::typing_candidate::TypingCandidateFamily;
use crate::typing_context;
use crate::typing_pipeline::explain_typing_assist_with_pipeline;
use crate::word_reader::{split_last_ws_token, split_word_punctuation, split_ws_segments};

#[path = "l2/hot_memory.rs"]
mod hot_memory;
#[path = "l2/phase.rs"]
mod phase;
pub use hot_memory::{
    ime_word_candidate_memory_is_warm, l2_surface_memory_status, L2SurfaceMemoryStatus,
};
pub(crate) use hot_memory::{warm_up_ime_word_candidate_memory, warm_up_surface_motif_memory};
use phase::apply_l2_phase_shadow;

#[path = "l2/surface.rs"]
mod surface;
use surface::*;
pub(crate) use surface::{
    l2_surface_foundation_contains, l2_surface_foundation_has_authority, l2_surface_foundation_rank,
};

const MAX_LAYOUT_SCAN_CANDIDATES: usize = 4;
const LAYOUT_THEN_L2_WORD_CENTER: &str = "layout_then_l2_word_center";
const MAX_TAUGHT_CANDIDATES: usize = 6;
const L2_ACTIVE_SOURCE_TARGET: usize = 1_000_000;
pub(super) const L2_SURFACE_MOTIF_CELL: &str = "L2SurfaceMotifCell32";
pub(super) const L2_SURFACE_COMPLETION_CELL: &str = "L2SurfaceCompletionCell32";
const L2_FORM_ATTRACTOR_LIMIT: usize = 6;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum L2ImeWordCandidateKind {
    Completion,
    Replacement,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum L2ImeWordCandidateSource {
    LexicalPhase,
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
}

struct TaughtCandidateInput<'a> {
    original: &'a str,
    context: &'a TailContext,
    l1: &'a [WavePacket],
    options: &'a WaveOptions,
    replacement: &'a str,
    family: TypingCandidateFamily,
    score: f64,
    chosen: bool,
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

pub fn ime_l2_word_candidates(
    context_prefix: &str,
    token: &str,
    limit: usize,
) -> Vec<L2ImeWordCandidate> {
    if limit == 0 {
        return Vec::new();
    }
    let normalized = token.to_lowercase();
    let token_len = normalized.chars().count();
    if !(2..=18).contains(&token_len) || !normalized.chars().all(is_cyrillic_letter) {
        return Vec::new();
    }
    let context_tokens = super::llmwave::tokenize(context_prefix);
    let usage = super::usage_prior::cached_usage_prior_snapshot();
    let memory = surface_motif_memory();
    let material_limit = limit.saturating_mul(8).max(limit);
    let mut lexical = memory.surface_candidates(&normalized, material_limit);
    lexical.extend(memory.completion_candidates(&normalized, material_limit));
    lexical.sort_by(|left, right| {
        right
            .score
            .cmp(&left.score)
            .then_with(|| left.rank.cmp(&right.rank))
            .then_with(|| left.word.cmp(&right.word))
    });
    lexical.dedup_by(|left, right| left.word == right.word);
    let mut candidates = lexical
        .into_iter()
        .map(|candidate| {
            let candidate_len = candidate.word.chars().count();
            let kind = if candidate.word.starts_with(&normalized) && candidate_len > token_len {
                L2ImeWordCandidateKind::Completion
            } else {
                L2ImeWordCandidateKind::Replacement
            };
            let usage_prior = usage.word_prior(&candidate.word);
            let context_prior = usage.context_word_prior(&context_tokens, &candidate.word);
            L2ImeWordCandidate {
                surface: candidate.word,
                kind,
                source: L2ImeWordCandidateSource::LexicalPhase,
                score: candidate.score,
                l1_overlap: candidate.l1_overlap,
                l2_overlap: candidate.l2_overlap,
                motif_overlap: candidate.motif_overlap,
                usage_prior,
                context_prior,
            }
        })
        .collect::<Vec<_>>();
    sort_and_truncate_ime_l2_candidates(&mut candidates, &usage, limit);
    candidates
}

pub(crate) fn l2_center_near_surfaces(text: &str, limit: usize) -> Vec<String> {
    if limit == 0 {
        return Vec::new();
    }
    let normalized = text.to_lowercase();
    let len = normalized.chars().count();
    if !(3..=18).contains(&len) || !normalized.chars().all(is_cyrillic_letter) {
        return Vec::new();
    }
    surface_motif_memory()
        .surface_candidates(&normalized, limit.saturating_mul(8))
        .into_iter()
        .filter(|candidate| {
            let distance = damerau_levenshtein(&normalized, &candidate.word);
            (1..=3).contains(&distance)
                && len.abs_diff(candidate.word.chars().count()) <= 3
                && form_attractor_has_authority(
                    &normalized,
                    &candidate.word,
                    len,
                    distance,
                    candidate.score,
                )
        })
        .take(limit)
        .map(|candidate| candidate.word)
        .collect()
}

pub(crate) fn l2_center_contains_surface(word: &str) -> bool {
    surface_motif_memory().contains_surface(word)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct L2SurfacePhaseReadout {
    pub(crate) exact_center: bool,
    pub(crate) l1_refs: usize,
    pub(crate) motif_refs: usize,
    pub(crate) covered_l1_refs: usize,
    pub(crate) residual_l1_refs: usize,
}

impl L2SurfacePhaseReadout {
    pub(crate) fn coherence_milli(self) -> u32 {
        if self.l1_refs == 0 {
            return 0;
        }
        ((self.covered_l1_refs.saturating_mul(1_000)) / self.l1_refs).min(u32::MAX as usize) as u32
    }
}

pub(crate) fn l2_surface_phase_readout(word: &str) -> L2SurfacePhaseReadout {
    let readout = surface_motif_memory().phase_readout(word);
    L2SurfacePhaseReadout {
        exact_center: readout.exact_center,
        l1_refs: readout.atom_count,
        motif_refs: readout.center_hits,
        covered_l1_refs: readout.center_hits,
        residual_l1_refs: readout.atom_count.saturating_sub(readout.center_hits),
    }
}

fn sort_and_truncate_ime_l2_candidates(
    candidates: &mut Vec<L2ImeWordCandidate>,
    usage: &super::usage_prior::UsagePriorSnapshot,
    limit: usize,
) {
    candidates.sort_by(|left, right| {
        l2_ime_word_candidate_score(right, usage)
            .cmp(&l2_ime_word_candidate_score(left, usage))
            .then_with(|| right.motif_overlap.cmp(&left.motif_overlap))
            .then_with(|| right.l2_overlap.cmp(&left.l2_overlap))
            .then_with(|| right.l1_overlap.cmp(&left.l1_overlap))
            .then_with(|| {
                left.surface
                    .chars()
                    .count()
                    .cmp(&right.surface.chars().count())
            })
            .then_with(|| left.surface.cmp(&right.surface))
    });
    candidates.dedup_by(|left, right| left.surface == right.surface);
    candidates.truncate(limit);
}

fn l2_ime_word_candidate_score(
    candidate: &L2ImeWordCandidate,
    usage: &super::usage_prior::UsagePriorSnapshot,
) -> u32 {
    let prior = ((candidate.usage_prior * 1600.0 + candidate.context_prior * 2600.0)
        .round()
        .clamp(0.0, 820.0) as u32)
        .saturating_add(usage.accepted_word_count(&candidate.surface).min(40) * 18);
    let kind_bonus = match candidate.kind {
        L2ImeWordCandidateKind::Completion => 80,
        L2ImeWordCandidateKind::Replacement => 0,
    };
    candidate
        .score
        .saturating_add(prior)
        .saturating_add(kind_bonus)
}

pub fn run_l2_refined_with_feedback(
    original: &str,
    l1: &[WavePacket],
    options: &WaveOptions,
    feedback: &L3Feedback,
) -> Vec<WordCandidate> {
    let tail = original.trim_end();
    let Some((prefix, token)) = split_last_ws_token(tail) else {
        return Vec::new();
    };
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
    let mut has_l2_surface_motif_candidate = false;
    if boundary_scan.is_empty()
        && (options.is_enabled(L2_SURFACE_MOTIF_CELL)
            || options.is_enabled(L2_SURFACE_COMPLETION_CELL))
    {
        for candidate in surface_motif_word_candidates(prefix, token, &context, l1, options) {
            has_l2_surface_motif_candidate |= candidate.source == L2_SURFACE_MOTIF_CELL;
            push_unique_candidate(&mut candidates, candidate);
        }
        if options.is_enabled(LEXICAL_ATTRACTOR_CELL) {
            for candidate in form_attractor_word_candidates(prefix, token, &context, l1) {
                push_unique_candidate(&mut candidates, candidate);
            }
        }
        if options.is_enabled(L2_SURFACE_MOTIF_CELL) {
            for candidate in surface_motif_scan_candidates(tail, l1, &context) {
                has_l2_surface_motif_candidate |= candidate.source == L2_SURFACE_MOTIF_CELL;
                push_unique_candidate(&mut candidates, candidate);
            }
        }
    }
    mark_timing!("surface-motif");
    let has_explicit_boundary_split = token.chars().all(is_cyrillic_letter)
        && boundary_replacement_for_word(&token.to_lowercase()).is_some();
    if options.is_enabled("BoundaryCell32") {
        if !has_l2_surface_motif_candidate || has_explicit_boundary_split {
            for candidate in boundary_split_candidates(prefix, token, l1, &context) {
                push_unique_candidate(&mut candidates, candidate);
            }
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
    if options.is_enabled(super::context_wave::SEMANTIC_WORD_SOURCE) {
        for candidate in super::context_wave::semantic_word_candidates(tail) {
            push_unique_candidate(&mut candidates, candidate);
        }
    }
    mark_timing!("semantic-word");
    if options.is_enabled("PhraseCell32") {
        for candidate in customs_actor_phrase_candidates(tail, &context, l1) {
            push_unique_candidate(&mut candidates, candidate);
        }
    }
    mark_timing!("phrase");
    if options.is_enabled(super::context_wave::PHRASE_FORECAST_CELL) && options.llmwave_shadow() {
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

fn taught_candidates(
    original: &str,
    context: &TailContext,
    l1: &[WavePacket],
    options: &WaveOptions,
) -> Vec<WordCandidate> {
    let configured = crate::config::default_typing_assist_pipeline();
    let pipeline = typing_context::typing_assist_pipeline_for_context(
        true,
        CorrectionSafety::Experimental,
        &configured,
        original,
    );
    let explanation = explain_typing_assist_with_pipeline(original, true, &pipeline);
    if explanation.output.is_none() {
        return Vec::new();
    }
    let mut candidates = Vec::new();
    if let Some(chosen) = explanation.chosen.as_ref() {
        if let Some(candidate) = taught_word_candidate(TaughtCandidateInput {
            original,
            context,
            l1,
            options,
            replacement: &chosen.replacement,
            family: chosen.score.family,
            score: chosen.score.total,
            chosen: true,
        }) {
            push_unique_candidate(&mut candidates, candidate);
        }
    }
    for evaluation in &explanation.evaluations {
        if evaluation.rejected.is_some() {
            continue;
        }
        let Some(candidate) = evaluation.candidate.as_ref() else {
            continue;
        };
        if let Some(candidate) = taught_word_candidate(TaughtCandidateInput {
            original,
            context,
            l1,
            options,
            replacement: &candidate.replacement,
            family: candidate.score.family,
            score: candidate.score.total,
            chosen: false,
        }) {
            push_unique_candidate(&mut candidates, candidate);
        }
        if candidates.len() >= MAX_TAUGHT_CANDIDATES {
            break;
        }
    }
    candidates
}

fn should_run_taught_candidates(token: &str, options: &WaveOptions) -> bool {
    if options.is_enabled(L2_SURFACE_MOTIF_CELL)
        && token.chars().count() >= 4
        && token.chars().all(is_cyrillic_letter)
    {
        return false;
    }
    true
}

fn taught_word_candidate(input: TaughtCandidateInput<'_>) -> Option<WordCandidate> {
    let replacement = input.replacement.trim_end();
    if replacement == input.original.trim_end() {
        return None;
    }
    if matches!(input.family, TypingCandidateFamily::Layout)
        && known_short_russian_token_blocks_layout(input.original.trim_end())
        && !short_cyrillic_layout_technical_allowed(replacement)
    {
        return None;
    }
    let source = match input.family {
        TypingCandidateFamily::Layout if input.options.is_enabled("LayoutWordCell32") => {
            "LayoutWordCell32"
        }
        TypingCandidateFamily::Structural if input.options.is_enabled("BoundaryCell32") => {
            "BoundaryCell32"
        }
        TypingCandidateFamily::Typo | TypingCandidateFamily::Exact
            if is_phrase_grammar_candidate(input.context, input.original, replacement)
                && input.options.is_enabled("GrammarCell32") =>
        {
            "GrammarCell32"
        }
        TypingCandidateFamily::Typo
        | TypingCandidateFamily::Visual
        | TypingCandidateFamily::Exact
        | TypingCandidateFamily::Cleanup
            if input.options.is_enabled("PhraseCell32") =>
        {
            "PhraseCell32"
        }
        _ => return None,
    };
    if source == "PhraseCell32" && unsafe_single_token_phrase_typo(input.original, replacement) {
        return None;
    }
    Some(WordCandidate {
        text: replacement.to_string(),
        source,
        energy: taught_energy(input.score, source, input.l1, input.chosen),
        risk: taught_risk(
            input.family,
            source,
            input.original,
            replacement,
            input.chosen,
        ),
        support: candidate_support(input.l1, input.context),
    })
}

fn layout_candidate(
    prefix: &str,
    token: &str,
    context: &TailContext,
    l1: &[WavePacket],
) -> Option<WordCandidate> {
    if token.chars().count() < 2 {
        return None;
    }
    if is_common_en_technical_word(&token.to_ascii_lowercase()) {
        return None;
    }
    if technical_context_blocks_layout(prefix, token) {
        return None;
    }
    let (converted, strong_autoswitch, word_center_settled) = layout_converted_token(token)?;
    if converted == token {
        return None;
    }
    let learned_transition = learned_layout_transition_accepts(prefix, token, &converted);
    if context.token_count() < 2
        && token.chars().count() > 3
        && !is_common_en_technical_word(&converted.to_ascii_lowercase())
        && !strong_autoswitch
        && !learned_transition
        && !word_center_settled
    {
        return None;
    }
    if known_short_russian_token_blocks_layout(token)
        && !short_cyrillic_layout_technical_allowed(&converted)
    {
        return None;
    }
    if !layout_candidate_allowed(
        token,
        &converted,
        strong_autoswitch,
        learned_transition || word_center_settled,
    ) {
        return None;
    }
    if !language_allows_layout(token, &converted, learned_transition || word_center_settled) {
        return None;
    }
    let energy = l1_energy(l1, "KeyboardCell32").max(0.35);
    let risk = if strong_autoswitch {
        layout_risk(token, &converted, context).min(0.05)
    } else if word_center_settled {
        (layout_risk(token, &converted, context) + 0.06).min(0.24)
    } else {
        layout_risk(token, &converted, context)
    };
    if energy <= risk {
        return None;
    }
    Some(WordCandidate {
        text: format!("{prefix}{converted}"),
        source: if word_center_settled {
            LAYOUT_THEN_L2_WORD_CENTER
        } else {
            "LayoutWordCell32"
        },
        energy,
        risk,
        support: candidate_support(l1, context),
    })
}

fn layout_scan_candidates(
    tail: &str,
    context: &TailContext,
    l1: &[WavePacket],
) -> Vec<WordCandidate> {
    let tokens = tail.split_whitespace().collect::<Vec<_>>();
    if tokens.len() < 2 || tokens.len() > 15 {
        return Vec::new();
    }
    let mut candidates = Vec::new();
    for idx in (0..tokens.len()).rev() {
        let token = tokens[idx];
        if token.chars().count() < 2 {
            continue;
        }
        if is_common_en_technical_word(&token.to_ascii_lowercase()) {
            continue;
        }
        let prefix = if idx == 0 { "" } else { tokens[idx - 1] };
        if technical_context_blocks_layout(prefix, token) {
            continue;
        }
        let Some((converted, strong_autoswitch, word_center_settled)) =
            layout_converted_token(token)
        else {
            continue;
        };
        let learned_transition = learned_layout_transition_accepts(prefix, token, &converted);
        if converted == token
            || !layout_candidate_allowed(
                token,
                &converted,
                strong_autoswitch,
                learned_transition || word_center_settled,
            )
            || !language_allows_layout(token, &converted, learned_transition || word_center_settled)
        {
            continue;
        }
        if known_short_russian_token_blocks_layout(token)
            && !short_cyrillic_layout_technical_allowed(&converted)
        {
            continue;
        }
        let mut replaced = tokens
            .iter()
            .map(|item| (*item).to_string())
            .collect::<Vec<_>>();
        replaced[idx] = converted;
        let text = replaced.join(" ");
        let energy = l1_energy(l1, "KeyboardCell32").max(0.35);
        let base_risk = layout_risk(token, &replaced[idx], context);
        let risk = if strong_autoswitch {
            base_risk.min(0.08)
        } else if word_center_settled {
            (base_risk + 0.08).min(0.28)
        } else {
            (base_risk + 0.08).min(0.90)
        };
        if energy <= risk {
            continue;
        }
        candidates.push(WordCandidate {
            text,
            source: if word_center_settled {
                LAYOUT_THEN_L2_WORD_CENTER
            } else {
                "LayoutWordCell32"
            },
            energy,
            risk,
            support: candidate_support(l1, context),
        });
        if candidates.len() >= MAX_LAYOUT_SCAN_CANDIDATES {
            break;
        }
    }
    candidates
}

fn layout_converted_token(token: &str) -> Option<(String, bool, bool)> {
    if token.chars().any(is_cyrillic_letter) {
        let raw_converted = convert(token, detect_direction(token));
        if token.chars().all(is_cyrillic_letter)
            && !cyrillic_layout_word_center_blocked(token)
            && raw_converted != token
        {
            let (leading, word, trailing) = split_word_punctuation(&raw_converted);
            if leading.is_empty() && !word.is_empty() {
                if let Some(center) = settle_english_word_center(word) {
                    let center = apply_word_case(token, &center);
                    return Some((format!("{center}{trailing}"), false, true));
                }
            }
        }
        if let Some(converted) = crate::layout_autoswitch::correct_wrong_layout_cyrillic_word(token)
        {
            return Some((converted, true, false));
        }
    }
    let converted = convert(token, detect_direction(token));
    if converted == token {
        return None;
    }
    Some((converted, false, false))
}

fn settle_english_word_center(token: &str) -> Option<String> {
    let normalized = token.to_ascii_lowercase();
    if !(4..=18).contains(&normalized.chars().count())
        || !normalized.chars().all(|ch| ch.is_ascii_alphabetic())
    {
        return None;
    }
    let candidates = super::context_wave::semantic_word_candidates(token)
        .into_iter()
        .filter(|candidate| {
            candidate.source == LEXICAL_ATTRACTOR_CELL
                && candidate.text.chars().all(|ch| ch.is_ascii_alphabetic())
        })
        .collect::<Vec<_>>();
    let best = candidates.first()?;
    let best_distance = damerau_levenshtein(&normalized, &best.text);
    if let Some(second) = candidates.get(1) {
        let second_distance = damerau_levenshtein(&normalized, &second.text);
        let best_net = best.energy - best.risk;
        let second_net = second.energy - second.risk;
        if second_distance == best_distance && best_net < second_net + 0.02 {
            return None;
        }
    }
    Some(best.text.clone())
}

fn cyrillic_layout_word_center_blocked(token: &str) -> bool {
    let lower = token.to_lowercase();
    is_user_protected_word(&lower)
        || is_common_ru_word(&lower)
        || is_known_russian_word_or_form(&lower)
        || surface_motif_strict_known_surface(&lower)
}

fn language_allows_layout(token: &str, converted: &str, learned_transition: bool) -> bool {
    if learned_transition {
        return true;
    }
    let token_ascii = token.chars().all(|ch| ch.is_ascii_alphabetic());
    let converted_cyrillic = converted.chars().all(is_cyrillic_letter);
    if token_ascii && converted_cyrillic {
        return is_common_ru_word(&converted.to_lowercase());
    }
    true
}

fn short_token_candidates(
    prefix: &str,
    token: &str,
    context: &TailContext,
    l1: &[WavePacket],
) -> Vec<WordCandidate> {
    let clean = token.trim_matches(|ch: char| ch.is_ascii_punctuation());
    if clean.chars().count() != 1 || !clean.chars().all(|ch| ch.is_ascii_alphabetic()) {
        return Vec::new();
    }
    if context.token_count() < 2 || technical_context_blocks_layout(prefix, token) {
        return Vec::new();
    }

    let mut candidates = Vec::new();
    let converted = convert(clean, detect_direction(clean));
    let converted_lower = converted.to_lowercase();
    if converted != clean
        && converted.chars().all(is_cyrillic_letter)
        && (is_ru_short_function_word(&converted_lower) || is_common_ru_word(&converted_lower))
    {
        candidates.push(short_token_candidate(ShortTokenCandidateInput {
            prefix,
            token,
            replacement: &converted,
            reason: "keyboard-short-token",
            energy_floor: 0.90,
            risk: short_token_risk(context, token, "keyboard"),
            l1,
            context,
        }));
    }

    if clean.eq_ignore_ascii_case("b") {
        for (replacement, reason) in [
            (visual_b_default_replacement(), "visual-b-default"),
            (visual_b_after_ascii_replacement(), "visual-b-after-ascii"),
        ] {
            if replacement != converted
                && !candidates.iter().any(|item| {
                    item.text
                        .split_whitespace()
                        .last()
                        .is_some_and(|last| last == replacement)
                })
            {
                candidates.push(short_token_candidate(ShortTokenCandidateInput {
                    prefix,
                    token,
                    replacement,
                    reason,
                    energy_floor: 0.76,
                    risk: short_token_risk(context, token, "visual"),
                    l1,
                    context,
                }));
            }
        }
    }
    candidates
}

struct ShortTokenCandidateInput<'a> {
    prefix: &'a str,
    token: &'a str,
    replacement: &'a str,
    reason: &'a str,
    energy_floor: f32,
    risk: f32,
    l1: &'a [WavePacket],
    context: &'a TailContext,
}

fn short_token_candidate(input: ShortTokenCandidateInput<'_>) -> WordCandidate {
    let replacement = if input.token.chars().next().is_some_and(char::is_uppercase) {
        input.replacement.to_uppercase()
    } else {
        input.replacement.to_string()
    };
    WordCandidate {
        text: format!("{}{}", input.prefix, replacement),
        source: "ShortTokenCell32",
        energy: l1_energy(input.l1, "KeyboardCell32").max(input.energy_floor),
        risk: input.risk,
        support: {
            let mut support = candidate_support(input.l1, input.context);
            support.push(input.reason.to_string());
            support
        },
    }
}

fn short_token_risk(context: &TailContext, token: &str, mode: &str) -> f32 {
    let technical_context = context.has_technical_context();
    let ascii_context = context.tokens.iter().any(|item| {
        matches!(item.kind, TokenKind::AsciiWord | TokenKind::TechnicalAscii)
            && !item.text.eq_ignore_ascii_case(token)
    });
    let cyrillic_context = context
        .tokens
        .iter()
        .any(|item| item.kind == TokenKind::CyrillicWord);
    let mut risk: f32 = match mode {
        "visual" => 0.30,
        _ => 0.18,
    };
    if technical_context {
        risk += 0.35;
    }
    if ascii_context && !cyrillic_context {
        risk += 0.28;
    }
    if cyrillic_context {
        risk -= 0.08;
    }
    risk.clamp(0.05, 0.85)
}

fn technical_context_blocks_layout(prefix: &str, token: &str) -> bool {
    if !token.chars().all(|ch| ch.is_ascii_alphabetic()) {
        return false;
    }
    let Some(previous) = previous_token(prefix) else {
        return false;
    };
    is_common_en_guard_prefix(&previous.to_ascii_lowercase()) && token.chars().count() >= 3
}

fn known_short_russian_token_blocks_layout(token: &str) -> bool {
    let lower = token.to_lowercase();
    token.chars().count() <= 3
        && lower.chars().all(is_cyrillic_letter)
        && (is_common_ru_word(&lower)
            || surface_motif_known_surface(&lower)
            || token.chars().all(is_cyrillic_letter))
}

fn short_cyrillic_layout_technical_allowed(converted: &str) -> bool {
    matches!(
        converted.to_ascii_lowercase().as_str(),
        "api" | "css" | "eng" | "git" | "go" | "lay" | "log" | "md" | "ms" | "rus" | "ssh" | "vpn"
    )
}

fn customs_actor_phrase_candidates(
    tail: &str,
    context: &TailContext,
    l1: &[WavePacket],
) -> Vec<WordCandidate> {
    if context.has_technical_context() || context.tokens.len() < 4 {
        return Vec::new();
    }
    let Some(previous) = context.previous() else {
        return Vec::new();
    };
    let Some(last) = context.last() else {
        return Vec::new();
    };
    if clean_ru_token(&previous.text) != "таможен" || clean_ru_token(&last.text) != "мы" {
        return Vec::new();
    }
    if !has_customs_actor_context(context) {
        return Vec::new();
    }
    let mut tokens = tail
        .split_whitespace()
        .map(str::to_string)
        .collect::<Vec<_>>();
    if tokens.len() < 2 {
        return Vec::new();
    }
    let previous_idx = tokens.len() - 2;
    let Some(replacement) = replace_cyrillic_core(&tokens[previous_idx], "таможим") else {
        return Vec::new();
    };
    tokens[previous_idx] = replacement;
    vec![WordCandidate {
        text: tokens.join(" "),
        source: "PhraseCell32",
        energy: l1_energy(l1, "ScriptCell32")
            .max(l1_energy(l1, "BoundaryCell32"))
            .max(0.88),
        risk: 0.13,
        support: {
            let mut support = candidate_support(l1, context);
            support.push("customs-actor-phrase".to_string());
            support.push("previous=таможен last=мы replacement=таможим".to_string());
            support
        },
    }]
}

fn has_customs_actor_context(context: &TailContext) -> bool {
    context.tokens.iter().any(|token| {
        let token = clean_ru_token(&token.text);
        token.contains("поставщик")
            || token.contains("цен")
            || token.contains("склад")
            || token.contains("покупател")
            || token.contains("накладн")
            || token.contains("меркур")
            || token.contains("логист")
            || token.contains("достав")
    })
}

fn replace_cyrillic_core(token: &str, replacement: &str) -> Option<String> {
    let start = token.find(is_cyrillic_letter)?;
    let end = token
        .char_indices()
        .rev()
        .find(|(_idx, ch)| is_cyrillic_letter(*ch))
        .map(|(idx, ch)| idx + ch.len_utf8())?;
    if start >= end {
        return None;
    }
    let replacement = if token[start..end]
        .chars()
        .next()
        .is_some_and(char::is_uppercase)
    {
        capitalize_first(replacement)
    } else {
        replacement.to_string()
    };
    Some(format!(
        "{}{}{}",
        &token[..start],
        replacement,
        &token[end..]
    ))
}

fn capitalize_first(word: &str) -> String {
    let mut chars = word.chars();
    let Some(first) = chars.next() else {
        return String::new();
    };
    first.to_uppercase().chain(chars).collect()
}

fn grammar_agreement_candidates(
    tail: &str,
    context: &TailContext,
    l1: &[WavePacket],
) -> Vec<WordCandidate> {
    if context.has_technical_context() {
        return Vec::new();
    }
    let Some(previous) = context.previous() else {
        return Vec::new();
    };
    let Some(last) = context.last() else {
        return Vec::new();
    };
    if previous.kind != TokenKind::CyrillicWord || last.kind != TokenKind::CyrillicWord {
        return Vec::new();
    }
    let previous = clean_ru_token(&previous.text);
    let last_clean = clean_ru_token(&last.text);
    let Some(replacement) = preposition_case_completion(&previous, &last_clean)
        .or_else(|| agree_adjective_like_tail(&previous, &last_clean))
    else {
        return Vec::new();
    };
    if replacement == last_clean {
        return Vec::new();
    }
    let Some((prefix, _token)) = split_last_ws_token(tail.trim_end()) else {
        return Vec::new();
    };
    vec![WordCandidate {
        text: format!("{prefix}{replacement}"),
        source: "GrammarCell32",
        energy: l1_energy(l1, "ScriptCell32")
            .max(l1_energy(l1, "BoundaryCell32"))
            .max(0.84),
        risk: 0.13,
        support: vec![
            "grammar-agreement".to_string(),
            format!("previous={previous:?} last={last_clean:?} replacement={replacement:?}"),
        ],
    }]
}

fn preposition_case_completion(previous: &str, word: &str) -> Option<String> {
    if !crate::lexicon::is_ru_short_preposition(previous)
        && !is_ru_one_letter_function_word(previous)
    {
        return None;
    }
    if word.chars().count() < 5 || is_common_ru_word(word) || is_known_russian_word_or_form(word) {
        return None;
    }
    let replacement = format!("{word}и");
    if is_known_russian_word_or_form(&replacement)
        || is_common_ru_word(&replacement)
        || word.ends_with("ани")
        || word.ends_with("ени")
    {
        return Some(replacement);
    }
    None
}

fn agree_adjective_like_tail(previous: &str, word: &str) -> Option<String> {
    if previous.chars().count() < 4 || word.chars().count() < 6 {
        return None;
    }
    if is_common_ru_word(word) {
        return None;
    }
    if has_russian_verb_tail(previous) {
        return None;
    }
    let stem = word
        .strip_suffix("ительные")
        .map(|stem| (stem, "ительный"))
        .or_else(|| word.strip_suffix("альные").map(|stem| (stem, "альный")))
        .or_else(|| word.strip_suffix("ные").map(|stem| (stem, "ный")))
        .or_else(|| word.strip_suffix("ые").map(|stem| (stem, "ый")))?;
    if !looks_like_singular_anchor(previous) {
        return None;
    }
    if looks_like_plural_anchor(previous) {
        return None;
    }
    Some(format!("{}{}", stem.0, stem.1))
}

fn has_russian_verb_tail(word: &str) -> bool {
    const VERB_TAILS: &[&str] = &[
        "ет", "ит", "ют", "ут", "ат", "ят", "ем", "им", "ешь", "ишь", "ете", "ите", "ал", "ала",
        "ило", "или", "ил", "ено", "ена", "ены", "ает", "яет", "ует",
    ];
    VERB_TAILS.iter().any(|tail| word.ends_with(tail))
}

fn looks_like_plural_anchor(word: &str) -> bool {
    word.ends_with("ые")
        || word.ends_with("ие")
        || word.ends_with("ых")
        || word.ends_with("их")
        || word.ends_with("ыми")
        || word.ends_with("ими")
}

fn looks_like_singular_anchor(word: &str) -> bool {
    let Some(last) = word.chars().last() else {
        return false;
    };
    matches!(
        last,
        'б' | 'в'
            | 'г'
            | 'д'
            | 'ж'
            | 'з'
            | 'й'
            | 'к'
            | 'л'
            | 'м'
            | 'н'
            | 'п'
            | 'р'
            | 'с'
            | 'т'
            | 'ф'
            | 'х'
            | 'ц'
            | 'ч'
            | 'ш'
            | 'щ'
            | 'о'
            | 'е'
    )
}

fn clean_ru_token(token: &str) -> String {
    token
        .trim_matches(|ch: char| ch.is_ascii_punctuation() || matches!(ch, '«' | '»' | '“' | '”'))
        .to_lowercase()
}

fn layout_candidate_allowed(
    token: &str,
    converted: &str,
    strong_autoswitch: bool,
    learned_transition: bool,
) -> bool {
    if strong_autoswitch || learned_transition {
        return true;
    }
    let token_ascii = token.chars().all(|ch| ch.is_ascii_alphabetic());
    let token_cyrillic = token.chars().all(is_cyrillic_letter);
    let converted_ascii = converted.chars().all(|ch| ch.is_ascii_alphabetic());
    let converted_cyrillic = converted.chars().all(is_cyrillic_letter);

    if token_ascii && converted_cyrillic {
        return true;
    }
    if token_cyrillic && converted_ascii {
        let token_lower = token.to_lowercase();
        if is_user_protected_word(&token_lower) || surface_motif_known_surface(&token_lower) {
            return false;
        }
        return is_common_en_technical_word(&converted.to_ascii_lowercase());
    }
    false
}

fn learned_layout_transition_accepts(prefix: &str, token: &str, converted: &str) -> bool {
    let context = llmwave::tokenize(prefix);
    let state = crate::transition_relation::transition_state_id(&format!("{prefix}{token}"));
    let usage = super::usage_prior::cached_usage_prior_snapshot();
    let transition = usage
        .hot_readout(&context, "LayoutWordCell32", "layout", &state, converted)
        .transition;
    transition.state_specific && transition.attract_count > transition.repel_count
}

fn technical_keep_candidate(token: &str, l1: &[WavePacket]) -> Option<WordCandidate> {
    if !is_common_en_technical_word(&token.to_ascii_lowercase()) {
        return None;
    }
    Some(WordCandidate {
        text: token.to_string(),
        source: "TechTokenCell32",
        energy: l1_energy(l1, "ScriptCell32").max(0.8),
        risk: 0.05,
        support: top_support(l1),
    })
}

fn boundary_split_candidates(
    prefix: &str,
    token: &str,
    l1: &[WavePacket],
    context: &TailContext,
) -> Vec<WordCandidate> {
    if context.has_technical_context() || !token.chars().all(is_cyrillic_letter) {
        return Vec::new();
    }
    let normalized = token.to_lowercase();
    if normalized.chars().count() < 6
        || is_common_ru_word(&normalized)
        || is_known_russian_word_or_form(&normalized)
        || is_ru_live_protected_word(&normalized)
    {
        return Vec::new();
    }
    if surface_motif_known_surface(&normalized) {
        return Vec::new();
    }
    if let Some(replacement) = light_boundary_replacement(&normalized) {
        return vec![WordCandidate {
            text: format!("{prefix}{}", apply_word_case(token, &replacement)),
            source: "BoundaryCell32",
            energy: l1_energy(l1, "BoundaryCell32").max(0.99),
            risk: 0.04,
            support: {
                let mut support = candidate_support(l1, context);
                support.push("light-boundary-split".to_string());
                support.push(format!("word={normalized:?} replacement={replacement:?}"));
                support
            },
        }];
    }
    if let Some(replacement) = crate::phrase_reader::correct_glued_russian_phrase(&normalized) {
        if replacement != normalized {
            return vec![WordCandidate {
                text: format!("{prefix}{}", apply_word_case(token, &replacement)),
                source: "BoundaryCell32",
                energy: l1_energy(l1, "BoundaryCell32").max(0.99),
                risk: 0.04,
                support: {
                    let mut support = candidate_support(l1, context);
                    support.push("direct-glued-phrase-boundary".to_string());
                    support.push(format!("word={normalized:?} replacement={replacement:?}"));
                    support
                },
            }];
        }
    }
    let chars = normalized.chars().collect::<Vec<_>>();
    let mut fuzzy_typo_candidates: Option<Vec<String>> = None;
    let mut candidates = Vec::new();
    for split in 1..chars.len() {
        let left = chars[..split].iter().collect::<String>();
        let right = chars[split..].iter().collect::<String>();
        if left.chars().count() > 2 && right.chars().count() < 3 {
            continue;
        }
        let short_function_boundary =
            left.chars().count() == 1 && is_ru_one_letter_function_word(&left);
        if short_function_boundary && fuzzy_typo_candidates.is_none() {
            fuzzy_typo_candidates = Some(crate::ru_typo::fuzzy_known_word_candidates(&normalized));
        }
        if short_function_boundary
            && fuzzy_typo_candidates
                .as_ref()
                .is_some_and(|candidates| !candidates.is_empty())
            && !strong_boundary_right_anchor(&right)
        {
            continue;
        }
        let known_left = short_function_boundary;
        let known_right = surface_motif_known_surface(&right);
        if !known_left || !known_right {
            continue;
        }
        let (energy, risk, reason) = if short_function_boundary {
            (
                l1_energy(l1, "BoundaryCell32").max(0.99),
                0.04,
                "hidden-short-function-boundary",
            )
        } else {
            (
                l1_energy(l1, "BoundaryCell32").max(0.78),
                if left.chars().count() <= 2 {
                    0.18
                } else {
                    0.12
                },
                "dictionary-split",
            )
        };
        candidates.push(WordCandidate {
            text: format!("{prefix}{left} {right}"),
            source: "BoundaryCell32",
            energy,
            risk,
            support: vec![reason.to_string(), format!("left={left:?} right={right:?}")],
        });
        if candidates.len() >= 3 {
            break;
        }
    }
    candidates
}

fn light_boundary_replacement(word: &str) -> Option<String> {
    let chars = word.chars().collect::<Vec<_>>();
    let mut best = None::<(usize, String)>;
    for split in 1..chars.len() {
        let left = chars[..split].iter().collect::<String>();
        let right = chars[split..].iter().collect::<String>();
        if left.chars().count() > 3 || right.chars().count() < 3 {
            continue;
        }
        if left.chars().count() > 1 && crate::lexicon::is_ru_short_preposition(&left) {
            continue;
        }
        let known_left_function = is_ru_one_letter_function_word(&left);
        let known_left_pronoun = crate::lexicon::is_ru_short_pronoun(&left);
        let known_left_common = is_common_ru_word(&left);
        let known_left = known_left_function || known_left_pronoun || known_left_common;
        let known_right = surface_motif_known_surface(&right);
        if known_left && known_right {
            let score = boundary_split_score(
                left.chars().count(),
                right.chars().count(),
                known_left_function,
                known_left_pronoun || known_left_common,
                is_common_ru_word(&right),
            );
            let replacement = format!("{left} {right}");
            let replace_best = match best.as_ref() {
                Some((best_score, _)) => score > *best_score,
                None => true,
            };
            if replace_best {
                best = Some((score, replacement));
            }
        }
    }
    best.map(|(_, replacement)| replacement)
}

fn boundary_split_score(
    left_len: usize,
    right_len: usize,
    left_function: bool,
    left_common: bool,
    right_common: bool,
) -> usize {
    let mut score = right_len.min(12);
    if left_common {
        score += 20;
    }
    if right_common {
        score += 10;
    }
    if left_function {
        score += 4;
    }
    score + left_len.min(4) * 3
}

fn boundary_scan_candidates(
    tail: &str,
    l1: &[WavePacket],
    context: &TailContext,
) -> Vec<WordCandidate> {
    if context.has_technical_context() {
        return Vec::new();
    }
    let segments = split_ws_segments(tail);
    if segments.len() < 3 || context.token_count() > 15 {
        return Vec::new();
    }

    let mut candidates = Vec::new();
    for (idx, (segment, is_ws)) in segments.iter().enumerate().rev() {
        if *is_ws {
            continue;
        }
        let (leading, word, trailing) = split_word_punctuation(segment);
        if word.is_empty() || !word.chars().all(is_cyrillic_letter) {
            continue;
        }
        let previous = previous_word_segment(&segments, idx);
        let Some(replacement) = contextual_boundary_replacement_for_word(word, previous)
            .or_else(|| boundary_replacement_for_word(word))
        else {
            continue;
        };
        if replacement == word {
            continue;
        }
        let text = replace_segment_word(&segments, idx, leading, &replacement, trailing);
        candidates.push(WordCandidate {
            text,
            source: "BoundaryCell32",
            energy: l1_energy(l1, "BoundaryCell32").max(0.82),
            risk: 0.10,
            support: {
                let mut support = candidate_support(l1, context);
                support.push("tail-boundary-scan".to_string());
                support.push(format!("word={word:?} replacement={replacement:?}"));
                support
            },
        });
        if candidates.len() >= 4 {
            return candidates;
        }
    }

    for window in word_segment_windows(&segments).into_iter().rev() {
        let pair_text = format!(
            "{}{}{}",
            segments[window.left_idx].0, segments[window.ws_idx].0, segments[window.right_idx].0
        );
        let Some((replacement, repair_kind, energy, risk)) =
            crate::phrase_reader::propose_moved_prefix_letter_pair(&pair_text)
                .map(|replacement| {
                    (
                        replacement,
                        "tail-moved-prefix-pair-scan",
                        l1_energy(l1, "BoundaryShiftCell32").max(0.92),
                        0.06,
                    )
                })
                .or_else(|| {
                    crate::phrase_reader::correct_split_word_pair(&pair_text).map(|replacement| {
                        (
                            replacement,
                            "tail-split-pair-scan",
                            l1_energy(l1, "BoundaryCell32").max(0.80),
                            0.12,
                        )
                    })
                })
        else {
            continue;
        };
        if replacement == pair_text {
            continue;
        }
        candidates.push(WordCandidate {
            text: replace_segment_window(
                &segments,
                window.left_idx,
                window.right_idx,
                &replacement,
            ),
            source: if repair_kind == "tail-moved-prefix-pair-scan" {
                "BoundaryShiftCell32"
            } else {
                "BoundaryCell32"
            },
            energy,
            risk,
            support: {
                let mut support = candidate_support(l1, context);
                support.push(repair_kind.to_string());
                support.push(format!("pair={pair_text:?} replacement={replacement:?}"));
                support
            },
        });
        if candidates.len() >= 4 {
            break;
        }
    }

    candidates
}

fn boundary_replacement_for_word(word: &str) -> Option<String> {
    crate::phrase_reader::correct_glued_russian_phrase(word).or_else(|| {
        let lower = word.to_lowercase();
        if lower.chars().count() < 6
            || is_common_ru_word(&lower)
            || is_known_russian_word_or_form(&lower)
        {
            return None;
        }
        let chars = lower.chars().collect::<Vec<_>>();
        for split in 1..chars.len() {
            let left = chars[..split].iter().collect::<String>();
            let right = chars[split..].iter().collect::<String>();
            if left.chars().count() > 2 && right.chars().count() < 3 {
                continue;
            }
            let known_left = left.chars().count() == 1 && is_ru_one_letter_function_word(&left);
            let known_right = is_common_ru_word(&right) || is_known_russian_word_or_form(&right);
            if known_left && known_right {
                let replacement = format!("{left} {right}");
                return Some(apply_word_case(word, &replacement));
            }
        }
        None
    })
}

fn contextual_boundary_replacement_for_word(word: &str, previous: Option<&str>) -> Option<String> {
    let previous = previous?.to_lowercase();
    if !crate::phrase_lexicon::is_short_russian_function_word(&previous) {
        return None;
    }

    let lower = word.to_lowercase();
    let chars = lower.chars().collect::<Vec<_>>();
    for split in 1..chars.len() {
        let left = chars[..split].iter().collect::<String>();
        let right = chars[split..].iter().collect::<String>();
        if !crate::lexicon::is_ru_short_pronoun(&left) {
            continue;
        }
        if !(right == "есть" || is_common_ru_word(&right) || is_known_russian_word_or_form(&right))
        {
            continue;
        }
        let replacement = format!("{left} {right}");
        return Some(apply_word_case(word, &replacement));
    }
    None
}

fn surface_motif_scan_candidates(
    tail: &str,
    l1: &[WavePacket],
    context: &TailContext,
) -> Vec<WordCandidate> {
    if context.has_technical_context() || context.token_count() > 15 {
        return Vec::new();
    }
    let segments = split_ws_segments(tail);
    if segments.len() < 3 {
        return Vec::new();
    }
    let mut candidates = Vec::new();
    for (idx, (segment, is_ws)) in segments.iter().enumerate().rev() {
        if *is_ws {
            continue;
        }
        let (leading, word, trailing) = split_word_punctuation(segment);
        if word.is_empty() || !word.chars().all(is_cyrillic_letter) {
            continue;
        }
        let Some(replacement) = surface_replacement_for_word(word) else {
            continue;
        };
        if replacement == word {
            continue;
        }
        let lower = word.to_lowercase();
        let replacement_lower = replacement.to_lowercase();
        let distance = damerau_levenshtein(&lower, &replacement_lower);
        if distance == 0 || distance > 3 {
            continue;
        }
        candidates.push(WordCandidate {
            text: replace_segment_word(&segments, idx, leading, &replacement, trailing),
            source: L2_SURFACE_MOTIF_CELL,
            energy: l1_energy(l1, "ScriptCell32").max(0.78),
            risk: surface_motif_typo_risk(context, distance),
            support: {
                let mut support = candidate_support(l1, context);
                support.push("tail-surface-scan".to_string());
                support.push(format!(
                    "word={word:?} replacement={replacement:?} distance={distance}"
                ));
                support
            },
        });
        if candidates.len() >= 4 {
            break;
        }
    }
    candidates
}

fn surface_replacement_for_word(word: &str) -> Option<String> {
    crate::ru_typo::correct_repeated_letter(word)
        .or_else(|| crate::ru_typo::correct_adjacent_transposition(word))
        .or_else(|| crate::ru_typo::correct_missing_letter(word))
}

struct SegmentWindow {
    left_idx: usize,
    ws_idx: usize,
    right_idx: usize,
}

fn word_segment_windows(segments: &[(&str, bool)]) -> Vec<SegmentWindow> {
    segments
        .windows(3)
        .enumerate()
        .filter_map(|(idx, window)| {
            let [left, ws, right] = window else {
                return None;
            };
            (!left.1 && ws.1 && !right.1).then_some(SegmentWindow {
                left_idx: idx,
                ws_idx: idx + 1,
                right_idx: idx + 2,
            })
        })
        .collect()
}

fn replace_segment_word(
    segments: &[(&str, bool)],
    target_idx: usize,
    leading: &str,
    replacement: &str,
    trailing: &str,
) -> String {
    let mut out = String::new();
    for (idx, (segment, _)) in segments.iter().enumerate() {
        if idx == target_idx {
            out.push_str(leading);
            out.push_str(replacement);
            out.push_str(trailing);
        } else {
            out.push_str(segment);
        }
    }
    out
}

fn replace_segment_window(
    segments: &[(&str, bool)],
    left_idx: usize,
    right_idx: usize,
    replacement: &str,
) -> String {
    let mut out = String::new();
    let mut idx = 0;
    while idx < segments.len() {
        if idx == left_idx {
            out.push_str(replacement);
            idx = right_idx + 1;
        } else {
            out.push_str(segments[idx].0);
            idx += 1;
        }
    }
    out
}

fn previous_word_segment<'a>(
    segments: &'a [(&'a str, bool)],
    before_idx: usize,
) -> Option<&'a str> {
    segments[..before_idx]
        .iter()
        .rev()
        .find_map(|(segment, is_ws)| {
            if *is_ws {
                return None;
            }
            let (_, word, _) = split_word_punctuation(segment);
            (!word.is_empty()).then_some(word)
        })
}

fn strong_boundary_right_anchor(lower: &str) -> bool {
    lower.chars().count() >= 5
        && (lower.ends_with("ах") || lower.ends_with("ях"))
        && (is_common_ru_word(lower) || is_known_russian_word_or_form(lower))
}

fn technical_context_keep_candidate(text: &str, l1: &[WavePacket]) -> Option<WordCandidate> {
    if !looks_like_shell_or_technical_phrase(text) {
        return None;
    }
    Some(WordCandidate {
        text: text.to_string(),
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

fn previous_token(prefix: &str) -> Option<&str> {
    prefix.split_whitespace().last()
}

fn layout_risk(token: &str, converted: &str, context: &TailContext) -> f32 {
    let short: f32 = if token.chars().count() <= 2 {
        0.35
    } else {
        0.10
    };
    let technical: f32 = if is_common_en_technical_word(&token.to_ascii_lowercase())
        || is_common_en_technical_word(&converted.to_ascii_lowercase())
    {
        0.20
    } else {
        0.0
    };
    let context_bonus = context.mixed_language_score();
    (short + technical - context_bonus).clamp(0.0, 0.85)
}

fn taught_energy(score: f64, source: &str, l1: &[WavePacket], chosen: bool) -> f32 {
    let base = match source {
        "LayoutWordCell32" => l1_energy(l1, "KeyboardCell32"),
        "BoundaryCell32" => l1_energy(l1, "BoundaryCell32"),
        "GrammarCell32" => l1_energy(l1, "ScriptCell32").max(l1_energy(l1, "BoundaryCell32")),
        "PhraseCell32" => l1_energy(l1, "ScriptCell32"),
        _ => 0.5,
    };
    let score = (score / 14.0).clamp(0.25, 0.95) as f32;
    let chosen_bonus = if chosen { 0.04 } else { 0.0 };
    (base.max(score) + chosen_bonus).min(0.99)
}

fn taught_risk(
    family: TypingCandidateFamily,
    source: &str,
    original: &str,
    replacement: &str,
    chosen: bool,
) -> f32 {
    let edit_ratio = normalized_edit_ratio(original, replacement);
    let base = match family {
        TypingCandidateFamily::Layout => 0.10,
        TypingCandidateFamily::Structural => 0.08,
        TypingCandidateFamily::Typo => 0.14,
        TypingCandidateFamily::Visual | TypingCandidateFamily::Exact => 0.10,
        TypingCandidateFamily::Cleanup | TypingCandidateFamily::Unknown => 0.22,
    };
    let bad_split =
        if source == "BoundaryCell32" && compact_text(original) != compact_text(replacement) {
            0.50
        } else {
            0.0
        };
    let chosen_bonus = if chosen { -0.03 } else { 0.0 };
    (base + edit_ratio * 0.20 + bad_split + chosen_bonus).clamp(0.02, 0.85)
}

fn is_phrase_grammar_candidate(context: &TailContext, original: &str, replacement: &str) -> bool {
    if context.token_count() < 2 {
        return false;
    }
    if original.split_whitespace().count() != replacement.split_whitespace().count() {
        return false;
    }
    if context.has_technical_context() {
        return false;
    }
    let Some(previous) = context.previous() else {
        return false;
    };
    let Some(last) = context.last() else {
        return false;
    };
    if previous.kind != TokenKind::CyrillicWord || last.kind != TokenKind::CyrillicWord {
        return false;
    }
    let previous = clean_ru_token(&previous.text);
    let last = clean_ru_token(&last.text);
    let Some(replacement_last) = replacement
        .split_whitespace()
        .next_back()
        .map(clean_ru_token)
    else {
        return false;
    };
    agree_adjective_like_tail(&previous, &last).as_deref() == Some(replacement_last.as_str())
}

fn normalized_edit_ratio(original: &str, replacement: &str) -> f32 {
    let original_len = original.chars().count().max(1);
    let replacement_len = replacement.chars().count();
    original_len.abs_diff(replacement_len) as f32 / original_len as f32
}

fn compact_text(text: &str) -> String {
    text.chars().filter(|ch| !ch.is_whitespace()).collect()
}

fn unsafe_single_token_phrase_typo(original: &str, replacement: &str) -> bool {
    let original = original.trim();
    let replacement = replacement.trim();
    if original.split_whitespace().count() != 1 || replacement.split_whitespace().count() != 1 {
        return false;
    }
    if original == replacement {
        return false;
    }
    let original_lower = original.to_lowercase();
    let replacement_lower = replacement.to_lowercase();
    if crate::ru_typo::rewrites_protected_pattern_term_stem(&original_lower, &replacement_lower) {
        return true;
    }
    original.chars().count() >= 4
        && original.chars().all(is_cyrillic_letter)
        && original.chars().any(char::is_uppercase)
        && original.chars().all(|ch| !ch.is_lowercase())
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
mod tests {
    use super::*;
    use crate::nanda_wave::l1::run_l1;

    #[test]
    fn accepted_transition_can_admit_unknown_layout_surface() {
        assert!(!layout_candidate_allowed("полняй", "gjkyzq", false, false));
        assert!(layout_candidate_allowed("полняй", "gjkyzq", false, true));
    }

    #[test]
    fn layout_candidate_for_last_token() {
        let original = "html djn ";
        let l1 = run_l1(original);
        let candidates = run_l2(original, &l1);
        assert!(candidates
            .iter()
            .any(|candidate| candidate.text == "html вот"));
    }

    #[test]
    fn l2_weight_scales_candidate_energy() {
        let original = "html djn ";
        let l1 = run_l1(original);
        let normal = run_l2_with_options(original, &l1, &WaveOptions::default());
        let muted = run_l2_with_options(
            original,
            &l1,
            &WaveOptions::default().with_layer_weights(0.5, 1.0),
        );
        let normal_layout = normal
            .iter()
            .find(|candidate| candidate.text == "html вот")
            .expect("normal layout candidate");
        let muted_layout = muted
            .iter()
            .find(|candidate| candidate.text == "html вот")
            .expect("muted layout candidate");

        assert!(muted_layout.energy < normal_layout.energy);
        assert!(muted_layout
            .support
            .iter()
            .any(|item| item == "l2-weight:0.50"));
    }

    #[test]
    fn keeps_known_technical_ascii_token() {
        let original = "git status ";
        let l1 = run_l1(original);
        let candidates = run_l2(original, &l1);
        assert!(candidates
            .iter()
            .all(|candidate| candidate.source != "LayoutWordCell32"));
    }

    #[test]
    fn technical_context_does_not_flip_argument_like_ascii() {
        let original = "vpn port ";
        let l1 = run_l1(original);
        let candidates = run_l2(original, &l1);
        assert!(candidates
            .iter()
            .all(|candidate| candidate.source != "LayoutWordCell32"));
    }

    #[test]
    fn scans_previous_layout_token_before_technical_tail() {
        let original = "html djn api ";
        let l1 = run_l1(original);
        let candidates = run_l2(original, &l1);
        assert!(candidates
            .iter()
            .any(|candidate| candidate.text == "html вот api"));
    }

    #[test]
    fn exposes_current_and_previous_layout_candidates_to_mesh() {
        let original = "html djn api ашду ";
        let l1 = run_l1(original);
        let candidates = run_l2(original, &l1);
        assert!(candidates
            .iter()
            .any(|candidate| candidate.text == "html djn api file"));
        assert!(candidates
            .iter()
            .any(|candidate| candidate.text == "html вот api ашду"));
    }

    #[test]
    fn mixed_ru_en_context_does_not_emit_raw_malformed_layout_candidate() {
        let original = "тест Ghjljkbv file ";
        let l1 = run_l1(original);
        let candidates = run_l2(original, &l1);

        assert!(candidates
            .iter()
            .all(|candidate| candidate.text != "тест Продолим file"));
    }

    #[test]
    fn guard_prefix_blocks_short_layout_argument() {
        let original = "api djn ";
        let l1 = run_l1(original);
        let candidates = run_l2(original, &l1);
        assert!(candidates
            .iter()
            .all(|candidate| candidate.text != "api вот"));
    }

    #[test]
    fn does_not_flip_normal_cyrillic_word_to_ascii_noise() {
        let original = "у нас есть ";
        let l1 = run_l1(original);
        let candidates = run_l2(original, &l1);
        assert!(candidates.is_empty());
    }

    #[test]
    fn layout_word_cell_respects_known_short_russian_words() {
        let original = "ой ";
        let l1 = run_l1(original);
        let candidates = run_l2(original, &l1);
        assert!(candidates
            .iter()
            .all(|candidate| candidate.text != "jq" && candidate.source != "LayoutWordCell32"));
    }

    #[test]
    fn grammar_cell_keeps_known_plural_forms_after_verbs() {
        let original = "имеет волнистые ";
        let l1 = run_l1(original);
        let candidates = run_l2(original, &l1);
        assert!(candidates
            .iter()
            .all(|candidate| candidate.text != "имеет волнистый"));
    }

    #[test]
    fn boundary_cell_gets_structural_candidate() {
        let original = "у насесть ";
        let l1 = run_l1(original);
        let candidates = run_l2(original, &l1);
        assert!(candidates
            .iter()
            .any(|candidate| candidate.source == "BoundaryCell32"));
    }

    #[test]
    fn boundary_cell_splits_dictionary_glue() {
        let original = "она есть ";
        let glued = original.replace(' ', "");
        let l1 = run_l1(&glued);
        let candidates = run_l2(&glued, &l1);
        assert!(candidates
            .iter()
            .any(|candidate| candidate.text == "она есть"));
    }

    #[test]
    fn boundary_cell_does_not_split_known_russian_word_forms() {
        for original in [
            "упоминай ",
            "поехал ",
            "поплыл ",
            "указать ",
            "сторона ",
            "улетели ",
            "кодировании ",
        ] {
            let l1 = run_l1(original);
            let candidates = run_l2(original, &l1);
            assert!(
                candidates
                    .iter()
                    .all(|candidate| candidate.source != "BoundaryCell32"),
                "known word must not become boundary split: {original:?} -> {candidates:?}"
            );
        }
    }

    #[test]
    fn boundary_cell_recovers_one_letter_function_boundary() {
        let original = "влогах ";
        let l1 = run_l1(original);
        let candidates = run_l2(original, &l1);
        let split = candidates
            .iter()
            .find(|candidate| candidate.text == "в логах")
            .expect("hidden short function boundary candidate");
        assert_eq!(split.source, "BoundaryCell32");
        assert!(
            split.energy - split.risk > 0.90,
            "split candidate must outrank single-word typo: {split:?}"
        );
    }

    #[test]
    fn boundary_cell_does_not_split_multi_letter_preposition_guesses() {
        for original in ["заполни поспорта ", "в задани "] {
            let l1 = run_l1(original);
            let candidates = run_l2(original, &l1);
            assert!(
                candidates
                    .iter()
                    .all(|candidate| candidate.source != "BoundaryCell32"),
                "multi-letter preposition guesses must not split automatically: {original:?} -> {candidates:?}"
            );
        }
    }

    #[test]
    fn boundary_cell_scans_glued_word_inside_tail() {
        let original = "я пишу мои слова мои предложения чтобыточно проверить дальше ";
        let l1 = run_l1(original);
        let candidates = run_l2(original, &l1);
        assert!(
            candidates.iter().any(|candidate| {
                candidate.source == "BoundaryCell32"
                    && candidate.text
                        == "я пишу мои слова мои предложения чтобы точно проверить дальше"
            }),
            "candidates={candidates:?}"
        );
    }

    #[test]
    fn boundary_cell_scans_split_pair_inside_tail() {
        let original = "сейчас думаю тако й пример работает ";
        let l1 = run_l1(original);
        let candidates = run_l2(original, &l1);
        assert!(
            candidates.iter().any(|candidate| {
                candidate.source == "BoundaryCell32"
                    && candidate.text == "сейчас думаю такой пример работает"
            }),
            "candidates={candidates:?}"
        );
    }

    #[test]
    fn boundary_cell_scans_moved_prefix_pair_inside_tail() {
        let original = "сервер работает н апостоянку ";
        let l1 = run_l1(original);
        let candidates = run_l2(original, &l1);
        assert!(
            candidates.iter().any(|candidate| {
                candidate.source == "BoundaryShiftCell32"
                    && candidate.text == "сервер работает на постоянку"
            }),
            "candidates={candidates:?}"
        );
    }

    #[test]
    fn boundary_cell_uses_context_to_split_known_glued_form() {
        let original = "мы должны помнить что у насесть право на информацию ";
        let l1 = run_l1(original);
        let candidates = run_l2(original, &l1);
        assert!(
            candidates.iter().any(|candidate| {
                candidate.source == "BoundaryCell32"
                    && candidate.text == "мы должны помнить что у нас есть право на информацию"
            }),
            "candidates={candidates:?}"
        );
    }

    #[test]
    fn phrase_cell_does_not_rewrite_single_all_caps_russian_terms() {
        for original in ["БЕЙСОВ ", "БЕЙСОВК ", "БЕЙСОВКИ ", "БЕЙСОВСКИ "]
        {
            let l1 = run_l1(original);
            let candidates = run_l2(original, &l1);
            assert!(
                candidates
                    .iter()
                    .all(|candidate| candidate.source != "PhraseCell32"),
                "all-caps term should not get PhraseCell typo candidate: {original:?} -> {candidates:?}"
            );
        }
    }

    #[test]
    fn phrase_cell_does_not_delete_n_from_pattern_terms() {
        for (original, rejected) in [
            ("патерн ", "патер"),
            ("патерна ", "патера"),
            ("патернов ", "патеров"),
        ] {
            let l1 = run_l1(original);
            let candidates = run_l2(original, &l1);
            assert!(
                candidates
                    .iter()
                    .all(|candidate| candidate.text.trim() != rejected),
                "pattern-like term should not get n-deletion candidate: {original:?} -> {candidates:?}"
            );
            assert!(
                candidates
                    .iter()
                    .all(|candidate| candidate.source != "PhraseCell32"),
                "pattern-like term should not get PhraseCell typo candidate: {original:?} -> {candidates:?}"
            );
        }
    }

    #[test]
    fn phrase_cell_gets_typo_candidate() {
        let original = "рабоатет ";
        let l1 = run_l1(original);
        let candidates = run_l2(original, &l1);
        assert!(candidates
            .iter()
            .any(|candidate| candidate.source == L2_SURFACE_MOTIF_CELL));
    }

    #[test]
    fn l2_surface_motif_cell_generates_word_candidate() {
        let original = "делай проверк ";
        let l1 = run_l1(original);
        let candidates = run_l2(original, &l1);
        assert!(
            candidates.iter().any(|candidate| {
                candidate.source == L2_SURFACE_MOTIF_CELL && candidate.text == "делай проверка"
            }),
            "candidates={candidates:?}"
        );
    }

    #[test]
    fn l2_surface_motif_cell_recovers_known_word_from_fuzzy_dictionary() {
        let original = "звгрузи ";
        let l1 = run_l1(original);
        let candidates = run_l2(original, &l1);
        assert!(
            candidates.iter().any(|candidate| {
                candidate.source == L2_SURFACE_MOTIF_CELL && candidate.text == "загрузи"
            }),
            "candidates={candidates:?}"
        );
    }

    #[test]
    fn l2_surface_layer_recovers_adjacent_transposition() {
        let original = "пукнт ";
        let l1 = run_l1(original);
        let candidates = run_l2(original, &l1);
        assert!(
            candidates.iter().any(|candidate| {
                matches!(
                    candidate.source,
                    L2_SURFACE_MOTIF_CELL | LEXICAL_ATTRACTOR_CELL
                ) && candidate.text == "пункт"
            }),
            "candidates={candidates:?}"
        );
    }

    #[test]
    fn l2_form_attractor_prefers_clean_corpus_center() {
        let original = "пукнт ";
        let l1 = run_l1(original);
        let candidates = run_l2(original, &l1);
        let first = candidates
            .first()
            .expect("dirty transposition should produce L2 attractor candidates");
        assert!(matches!(
            first.source,
            L2_SURFACE_MOTIF_CELL | LEXICAL_ATTRACTOR_CELL
        ));
        assert_eq!(first.text, "пункт");
        assert!(
            candidates.iter().any(|candidate| {
                candidate.text == "пуант" && candidate.source == LEXICAL_ATTRACTOR_CELL
            }),
            "near clean centers should remain visible but lower-ranked: {candidates:?}"
        );
    }

    #[test]
    fn l2_form_attractor_does_not_rewrite_stable_word() {
        let original = "писать ";
        let l1 = run_l1(original);
        let candidates = run_l2(original, &l1);
        assert!(
            candidates
                .iter()
                .all(|candidate| candidate.source != LEXICAL_ATTRACTOR_CELL),
            "stable word should not become an attractor rewrite: {candidates:?}"
        );
    }

    #[test]
    fn l2_form_attractor_does_not_rewrite_known_verb_form() {
        assert!(surface_motif_stable_existing_word("можем"));
        assert!(!surface_motif_stable_existing_word("пукнт"));
        assert!(!surface_motif_stable_existing_word("звгрузи"));

        for original in ["можем ", "проверка можем "] {
            let l1 = run_l1(original);
            let candidates = run_l2(original, &l1);
            assert!(
                candidates.iter().all(|candidate| {
                    !matches!(
                        candidate.source,
                        L2_SURFACE_MOTIF_CELL | LEXICAL_ATTRACTOR_CELL
                    ) || candidate.text == original.trim_end()
                }),
                "known verb form should not drift to a neighboring word: {candidates:?}"
            );
            assert!(
                candidates
                    .iter()
                    .all(|candidate| !matches!(candidate.text.as_str(), "модем" | "может")),
                "known verb form leaked standalone drift candidates: {candidates:?}"
            );
            assert!(
                candidates.iter().all(|candidate| {
                    !matches!(candidate.text.as_str(), "проверка модем" | "проверка может")
                }),
                "known verb form leaked phrase drift candidates: {candidates:?}"
            );
        }
    }

    #[test]
    fn l2_surface_motif_does_not_treat_usage_typo_as_stable_word() {
        assert!(!surface_motif_stable_existing_word("пукнт"));
        assert!(!fuzzy_surface_candidate_blocked("пукнт", "пукнт", "пункт"));
        let fuzzy = crate::ru_typo::fuzzy_known_word_candidates("пукнт");
        assert!(fuzzy.iter().any(|candidate| candidate == "пункт"));
        assert!(surface_motif_typo_has_authority(
            "пукнт",
            "пункт",
            900,
            &[],
            &fuzzy
        ));
        assert!(surface_motif_typo_allowed("пукнт", "пункт", 5, 1, 900));
        let l1 = run_l1("пукнт");
        let context = TailContext::from_text("пукнт");
        let cell_candidates =
            surface_motif_word_candidates("", "пукнт", &context, &l1, &WaveOptions::default());
        assert!(
            cell_candidates
                .iter()
                .any(|candidate| candidate.text == "пункт"),
            "cell_candidates={cell_candidates:?}"
        );
    }

    #[test]
    fn l2_surface_motif_memory_recovers_missing_letter_without_fuzzy_route() {
        let candidates = surface_motif_memory().surface_candidates("звгрузи", 8);
        assert!(
            candidates
                .iter()
                .any(|candidate| candidate.word == "загрузи"),
            "candidates={candidates:?}"
        );
    }

    #[test]
    fn lexical_phase_field_recovers_inflected_forms_from_compiled_transition_mass() {
        let cases = [
            ("рабоатет", "работает"),
            ("кнокопками", "кнопками"),
            ("фактческим", "фактическим"),
            ("подлючись", "подключись"),
            ("исправленно", "исправлено"),
        ];
        for (input, expected) in cases {
            let candidates = surface_motif_memory().surface_candidates(input, 32);
            assert!(
                candidates
                    .iter()
                    .any(|candidate| candidate.word == expected),
                "{input} -> {expected}, candidates={candidates:?}"
            );
        }
    }

    #[test]
    fn ime_l2_word_candidates_return_whole_words_not_suffixes() {
        let candidates = ime_l2_word_candidates("я хочу ", "пров", 8);
        assert!(
            candidates.iter().any(|candidate| {
                candidate.kind == L2ImeWordCandidateKind::Completion
                    && candidate.surface.starts_with("провер")
            }),
            "L2 IME candidates must expose complete word surfaces, got {candidates:?}"
        );
        assert!(
            candidates
                .iter()
                .all(|candidate| !candidate.surface.starts_with("ер")),
            "L2 must not return display suffixes as word candidates: {candidates:?}"
        );
    }

    #[test]
    fn lexical_phase_field_feeds_ime_completion_candidates() {
        let candidates = ime_l2_word_candidates("я хочу ", "пров", 8);
        assert!(
            candidates.iter().any(|candidate| {
                candidate.kind == L2ImeWordCandidateKind::Completion
                    && candidate.surface.starts_with("пров")
                    && candidate.surface.chars().count() > 4
            }),
            "lexical phase field must feed complete generated surfaces, got {candidates:?}"
        );
    }

    #[test]
    fn ime_l2_word_candidates_keep_replacements_distinct_from_completions() {
        let candidates = ime_l2_word_candidates("", "звгрузи", 8);
        assert!(
            candidates.iter().any(|candidate| {
                candidate.kind == L2ImeWordCandidateKind::Replacement
                    && candidate.surface == "загрузи"
            }),
            "noisy input should produce a whole-word replacement candidate, got {candidates:?}"
        );
        assert!(
            candidates
                .iter()
                .all(|candidate| candidate.surface != "агрузи"),
            "replacement candidates must not be converted into suffix fragments: {candidates:?}"
        );
    }

    #[test]
    fn l2_surface_motif_memory_recovers_common_shadow_words() {
        for (input, expected) in [("эсперемнт", "эксперимент"), ("ффективная", "эффективная")]
        {
            let candidates = surface_motif_memory().surface_candidates(input, 32);
            assert!(
                candidates
                    .iter()
                    .any(|candidate| candidate.word == expected),
                "input={input} expected={expected} candidates={candidates:?}"
            );
        }
    }

    #[test]
    fn l2_surface_motif_cell_promotes_common_shadow_words() {
        for (input, expected) in [
            ("эсперемнт ", "эксперимент"),
            ("ффективная ", "эффективная"),
        ] {
            let l1 = run_l1(input);
            let candidates = run_l2(input, &l1);
            let surface_candidates = surface_motif_memory().surface_candidates(input.trim(), 24);
            assert!(
                candidates.iter().any(|candidate| {
                    candidate.source == L2_SURFACE_MOTIF_CELL && candidate.text == expected
                }),
                "input={input} expected={expected} candidates={candidates:?} surface_candidates={surface_candidates:?}"
            );
        }
    }

    #[test]
    fn l2_surface_motif_cell_repairs_repeated_letter_all_caps_word() {
        let original = "ТРУССС ";
        let l1 = run_l1(original);
        let candidates = run_l2(original, &l1);
        assert!(
            candidates.iter().any(|candidate| {
                candidate.source == L2_SURFACE_MOTIF_CELL && candidate.text == "ТРУС"
            }),
            "candidates={candidates:?}"
        );
    }

    #[test]
    fn l2_surface_motif_cell_does_not_rewrite_known_word_without_context() {
        let original = "пукнут ";
        let l1 = run_l1(original);
        let candidates = run_l2(original, &l1);
        assert!(
            candidates
                .iter()
                .all(|candidate| candidate.source != L2_SURFACE_MOTIF_CELL),
            "candidates={candidates:?}"
        );
    }

    #[test]
    fn l2_surface_completion_cell_is_separate_from_typo_candidate() {
        let original = "делай пров ";
        let l1 = run_l1(original);
        let candidates = run_l2(original, &l1);
        assert!(
            candidates
                .iter()
                .any(|candidate| candidate.source == L2_SURFACE_COMPLETION_CELL),
            "candidates={candidates:?}"
        );
    }

    #[test]
    fn grammar_cell_does_not_fake_unknown_phrase_candidate() {
        let original = "фразы связанности ";
        let l1 = run_l1(original);
        let candidates = run_l2(original, &l1);
        assert!(candidates
            .iter()
            .all(|candidate| candidate.source != "GrammarCell32"));
    }

    #[test]
    fn grammar_cell_generates_agreement_candidate() {
        let original = "расчёт приблизительные ";
        let l1 = run_l1(original);
        let candidates = run_l2(original, &l1);
        assert!(candidates.iter().any(|candidate| {
            candidate.source == "GrammarCell32" && candidate.text == "расчёт приблизительный"
        }));
    }

    #[test]
    fn grammar_cell_completes_preposition_case_tail() {
        let original = "в задани ";
        let l1 = run_l1(original);
        let candidates = run_l2(original, &l1);
        assert!(candidates.iter().any(|candidate| {
            candidate.source == "GrammarCell32" && candidate.text == "в задании"
        }));
    }

    #[test]
    fn phrase_cell_generates_customs_actor_candidate() {
        let original = "Поставщик говорит что цена до склада нашего покупателя но таможен мы! ";
        let l1 = run_l1(original);
        let candidates = run_l2(original, &l1);
        assert!(candidates.iter().any(|candidate| {
            candidate.source == "PhraseCell32"
                && candidate.text
                    == "Поставщик говорит что цена до склада нашего покупателя но таможим мы!"
        }));
    }

    #[test]
    fn phrase_cell_does_not_rewrite_customs_actor_without_right_anchor() {
        let original = "Поставщик говорит что цена до склада нашего покупателя но таможен ";
        let l1 = run_l1(original);
        let candidates = run_l2(original, &l1);
        assert!(candidates
            .iter()
            .all(|candidate| !candidate.text.contains("таможим")));
    }

    #[test]
    fn phrase_cell_does_not_rewrite_customs_actor_without_domain_context() {
        let original = "я сказал что странно но таможен мы! ";
        let l1 = run_l1(original);
        let candidates = run_l2(original, &l1);
        assert!(candidates
            .iter()
            .all(|candidate| !candidate.text.contains("таможим")));
    }

    #[test]
    fn l2_exposes_l3_phrase_forecast_candidate_when_llmwave_is_enabled() {
        let memory = super::super::llmwave::LlmWaveMemory::from_text("на улице опять идёт дождь");
        let path =
            std::env::temp_dir().join(format!("lay-l2-llmwave-{}.llmw.bin", std::process::id()));
        super::super::llmwave::write_memory_packet(&path, &memory).unwrap();
        std::env::set_var("LAY_LLMWAVE_MEMORY", &path);

        let original = "на улице опять идёт д";
        let l1 = run_l1(original);
        let options = crate::nanda_wave::WaveOptions::default().with_llmwave_shadow(true);
        let candidates = run_l2_with_options(original, &l1, &options);
        std::env::remove_var("LAY_LLMWAVE_MEMORY");
        let _ = std::fs::remove_file(path);

        assert!(candidates.iter().any(|candidate| {
            candidate.source == crate::nanda_wave::context_wave::PHRASE_FORECAST_CELL
                && candidate.text == "на улице опять идёт дождь"
        }));
    }

    #[test]
    fn grammar_cell_keeps_plural_anchor_phrases() {
        for original in ["первые которые ", "такие условие ", "другие перемнные "]
        {
            let l1 = run_l1(original);
            let candidates = run_l2(original, &l1);
            assert!(
                candidates
                    .iter()
                    .all(|candidate| candidate.source != "GrammarCell32"),
                "plural anchor phrase should not get grammar candidate: {original:?} -> {candidates:?}"
            );
        }
    }

    #[test]
    fn grammar_cell_keeps_neuter_nouns_ending_with_ie() {
        for original in ["обратил внимание ", "срабатывает переварачивание "]
        {
            let l1 = run_l1(original);
            let candidates = run_l2(original, &l1);
            assert!(
                candidates
                    .iter()
                    .all(|candidate| candidate.source != "GrammarCell32"),
                "neuter noun should not get adjective agreement candidate: {original:?} -> {candidates:?}"
            );
        }
    }

    #[test]
    fn grammar_cell_keeps_neutral_clause_context() {
        for original in ["там недоказно ", "что там недоказно "] {
            let l1 = run_l1(original);
            let candidates = run_l2(original, &l1);
            assert!(
                candidates
                    .iter()
                    .all(|candidate| candidate.source != "GrammarCell32"),
                "neutral clause should not get grammar agreement candidate: {original:?} -> {candidates:?}"
            );
        }
    }

    #[test]
    fn technical_cell_protects_shell_phrase() {
        let original = "git checkout -b new ";
        let l1 = run_l1(original);
        let candidates = run_l2(original, &l1);
        assert!(candidates
            .iter()
            .any(|candidate| candidate.source == "TechTokenCell32"));
    }

    #[test]
    fn layout_cell_does_not_overrule_teacher_for_plain_ascii() {
        let original = "ola ";
        let l1 = run_l1(original);
        let candidates = run_l2(original, &l1);
        assert!(candidates.is_empty());
    }

    #[test]
    fn layout_cell_exposes_known_english_target_even_with_russian_typo_shadow() {
        let original = "вудуеу ";
        let l1 = run_l1(original);
        let candidates = run_l2(original, &l1);
        assert!(
            candidates
                .iter()
                .any(|candidate| candidate.source == "LayoutWordCell32"
                    && candidate.text == "delete"),
            "known English layout target must survive Russian typo shadow: {candidates:?}"
        );
    }

    #[test]
    fn short_token_cell_exposes_keyboard_and_visual_hypotheses() {
        let original = "пер b ";
        let l1 = run_l1(original);
        let candidates = run_l2(original, &l1);
        assert!(candidates.iter().any(|candidate| {
            candidate.source == "ShortTokenCell32" && candidate.text == "пер и"
        }));
        assert!(candidates.iter().any(|candidate| {
            candidate.source == "ShortTokenCell32" && candidate.text == "пер в"
        }));
    }

    #[test]
    fn short_token_cell_marks_ascii_context_as_risky() {
        let original = "vitamin B ";
        let l1 = run_l1(original);
        let candidates = run_l2(original, &l1);
        let short = candidates
            .iter()
            .find(|candidate| {
                candidate.source == "ShortTokenCell32" && candidate.text == "vitamin И"
            })
            .expect("short token candidate");
        assert!(short.risk >= 0.40);
    }
}

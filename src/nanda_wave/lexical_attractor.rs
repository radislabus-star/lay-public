use crate::dict::{convert, detect_direction};
use crate::keyboard::is_cyrillic_letter;
use crate::text_metrics::damerau_levenshtein;
use crate::word_reader::split_last_ws_token;

use super::context::TailContext;
use super::signal::WordCandidate;

pub const LEXICAL_ATTRACTOR_CELL: &str = "L2WordAttractorCell32";

#[repr(C)]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct LexicalBirthCandidate32 {
    pub form_wave_id: u32,
    pub lemma_wave_id: u32,
    pub context_centroid_id: u32,
    pub segmentation_score: u16,
    pub fast_mapping_score: u16,
    pub cross_situational_score: u16,
    pub usage_score: u16,
    pub grammar_score: u16,
    pub attractor_margin: i16,
    pub anti_confusion_penalty: i16,
    pub route: u8,
    pub flags: u8,
    pub reserved: u32,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct LexicalBindingRecord32 {
    pub form_wave_id: u32,
    pub lemma_wave_id: u32,
    pub concept_centroid_id: u32,
    pub context_centroid_id: u32,
    pub cleanup_basin_id: u32,
    pub morpheme_route_id: u32,
    pub usage_energy: u16,
    pub evidence_refs: u16,
    pub attractor_margin: i16,
    pub anti_confusion_penalty: i16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SurfaceProductionRoute {
    GraphemeWave,
    MorphemeRoute,
    LayoutTransducer,
    CopySpan,
    ByteFallback,
}

impl SurfaceProductionRoute {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::GraphemeWave => "grapheme_wave",
            Self::MorphemeRoute => "morpheme_route",
            Self::LayoutTransducer => "layout_transducer",
            Self::CopySpan => "copy_span",
            Self::ByteFallback => "byte_fallback",
        }
    }

    fn id(self) -> u8 {
        match self {
            Self::GraphemeWave => 1,
            Self::MorphemeRoute => 2,
            Self::LayoutTransducer => 3,
            Self::CopySpan => 4,
            Self::ByteFallback => 5,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct L2WordAttractorTrace {
    pub route: SurfaceProductionRoute,
    pub produced_surface: String,
    pub candidate: LexicalBirthCandidate32,
    pub accepted_binding: Option<LexicalBindingRecord32>,
    pub gate: LexicalBirthGate,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LexicalBirthGate {
    pub segmentation_ready: bool,
    pub fast_mapping_ready: bool,
    pub cross_situational_ready: bool,
    pub usage_ready: bool,
    pub grammar_ready: bool,
    pub attractor_ready: bool,
    pub anti_confusion_clear: bool,
    pub total_score: i16,
    pub accepted: bool,
}

pub fn lexical_attractor_candidates(tail: &str, context: &TailContext) -> Vec<WordCandidate> {
    lexical_attractor_traces(tail, context)
        .into_iter()
        .filter(|trace| trace.gate.accepted)
        .map(trace_to_candidate)
        .collect()
}

pub fn lexical_attractor_traces(tail: &str, context: &TailContext) -> Vec<L2WordAttractorTrace> {
    let Some((prefix, token)) = split_last_ws_token(tail.trim_end()) else {
        return Vec::new();
    };
    let mut traces = Vec::new();
    if let Some(surface) = layout_surface(token, context) {
        traces.push(build_trace(
            prefix,
            token,
            &surface,
            SurfaceProductionRoute::LayoutTransducer,
            context,
        ));
    }
    if let Some(surface) = typo_surface(token, context) {
        traces.push(build_trace(
            prefix,
            token,
            &surface,
            SurfaceProductionRoute::GraphemeWave,
            context,
        ));
    }
    traces
}

fn trace_to_candidate(trace: L2WordAttractorTrace) -> WordCandidate {
    WordCandidate {
        text: trace.produced_surface,
        source: LEXICAL_ATTRACTOR_CELL,
        energy: lexical_energy(&trace.gate),
        risk: lexical_risk(&trace.gate),
        support: vec![
            format!("surface-route={}", trace.route.as_str()),
            format!("form-wave={}", trace.candidate.form_wave_id),
            format!("lemma-wave={}", trace.candidate.lemma_wave_id),
            format!("context-centroid={}", trace.candidate.context_centroid_id),
            format!("attractor-margin={}", trace.candidate.attractor_margin),
            format!("anti-confusion={}", trace.candidate.anti_confusion_penalty),
        ],
    }
}

fn build_trace(
    prefix: &str,
    token: &str,
    surface: &str,
    route: SurfaceProductionRoute,
    context: &TailContext,
) -> L2WordAttractorTrace {
    let produced_surface = format!("{prefix}{surface}");
    let candidate = birth_candidate(token, surface, route, context);
    let gate = evaluate_birth(candidate);
    let accepted_binding = gate
        .accepted
        .then(|| binding_record(candidate, surface, context));
    L2WordAttractorTrace {
        route,
        produced_surface,
        candidate,
        accepted_binding,
        gate,
    }
}

fn birth_candidate(
    token: &str,
    surface: &str,
    route: SurfaceProductionRoute,
    context: &TailContext,
) -> LexicalBirthCandidate32 {
    let segmentation_score = if token.chars().count() >= 2 { 18 } else { 8 };
    let fast_mapping_score = match route {
        SurfaceProductionRoute::LayoutTransducer => 16,
        SurfaceProductionRoute::GraphemeWave => 12,
        _ => 8,
    };
    let cross_situational_score = if context.token_count() >= 2 { 20 } else { 10 };
    let usage_score = if known_surface(surface) { 18 } else { 8 };
    let grammar_score = grammar_score(surface, context);
    let attractor_margin = attractor_margin(token, surface, route);
    let anti_confusion_penalty = anti_confusion_penalty(token, surface, context);
    LexicalBirthCandidate32 {
        form_wave_id: stable_hash(surface),
        lemma_wave_id: stable_hash(&surface.to_lowercase()),
        context_centroid_id: stable_hash(&context.phrase_signature()),
        segmentation_score,
        fast_mapping_score,
        cross_situational_score,
        usage_score,
        grammar_score,
        attractor_margin,
        anti_confusion_penalty,
        route: route.id(),
        flags: 0,
        reserved: 0,
    }
}

fn evaluate_birth(candidate: LexicalBirthCandidate32) -> LexicalBirthGate {
    let segmentation_ready = candidate.segmentation_score >= 14;
    let fast_mapping_ready = candidate.fast_mapping_score >= 10;
    let cross_situational_ready = candidate.cross_situational_score >= 12;
    let usage_ready = candidate.usage_score >= 12;
    let grammar_ready = candidate.grammar_score >= 10;
    let attractor_ready = candidate.attractor_margin >= 10;
    let anti_confusion_clear = candidate.anti_confusion_penalty <= 12;
    let total_score = candidate
        .segmentation_score
        .saturating_add(candidate.fast_mapping_score)
        .saturating_add(candidate.cross_situational_score)
        .saturating_add(candidate.usage_score)
        .saturating_add(candidate.grammar_score) as i16
        + candidate.attractor_margin
        - candidate.anti_confusion_penalty;
    let accepted = total_score >= 74
        && segmentation_ready
        && fast_mapping_ready
        && cross_situational_ready
        && usage_ready
        && grammar_ready
        && attractor_ready
        && anti_confusion_clear;
    LexicalBirthGate {
        segmentation_ready,
        fast_mapping_ready,
        cross_situational_ready,
        usage_ready,
        grammar_ready,
        attractor_ready,
        anti_confusion_clear,
        total_score,
        accepted,
    }
}

fn binding_record(
    candidate: LexicalBirthCandidate32,
    surface: &str,
    context: &TailContext,
) -> LexicalBindingRecord32 {
    LexicalBindingRecord32 {
        form_wave_id: candidate.form_wave_id,
        lemma_wave_id: candidate.lemma_wave_id,
        concept_centroid_id: stable_hash(surface),
        context_centroid_id: candidate.context_centroid_id,
        cleanup_basin_id: candidate.form_wave_id,
        morpheme_route_id: stable_hash(surface_suffix(surface)),
        usage_energy: candidate.usage_score,
        evidence_refs: context.token_count().clamp(1, u16::MAX as usize) as u16,
        attractor_margin: candidate.attractor_margin,
        anti_confusion_penalty: candidate.anti_confusion_penalty,
    }
}

fn layout_surface(token: &str, context: &TailContext) -> Option<String> {
    if token.chars().count() < 2 {
        return None;
    }
    if context.has_technical_context() {
        return None;
    }
    let converted = convert(token, detect_direction(token));
    (converted != token && known_surface(&converted)).then_some(converted)
}

fn typo_surface(token: &str, context: &TailContext) -> Option<String> {
    if context.token_count() < 2 || !token.chars().all(is_cyrillic_letter) {
        return None;
    }
    let lower = token.to_lowercase();
    if token
        .chars()
        .all(|ch| !ch.is_alphabetic() || ch.is_uppercase())
        || crate::lexicon::is_user_protected_word(&lower)
        || crate::lexicon::is_ru_live_protected_word(&lower)
        || known_surface(&lower)
        || crate::ru_typo::rewrites_protected_pattern_term_stem(&lower, &lower)
    {
        return None;
    }
    for word in crate::lexicon::common_ru_words_iter() {
        if word.chars().count().abs_diff(lower.chars().count()) > 1 {
            continue;
        }
        if word.chars().next() != lower.chars().next() {
            continue;
        }
        if damerau_levenshtein(&lower, word) == 1 {
            return Some(word.to_string());
        }
    }
    None
}

fn known_surface(surface: &str) -> bool {
    let lower = surface.to_lowercase();
    crate::lexicon::is_common_ru_word(&lower)
        || crate::lexicon::is_common_en_technical_word(&lower)
        || crate::lexicon::is_ru_technical_loanword(&lower)
        || crate::lexicon::is_ru_live_protected_word(&lower)
        || crate::russian_lexicon::is_known_russian_word_or_form(&lower)
}

fn grammar_score(surface: &str, context: &TailContext) -> u16 {
    if surface.chars().all(is_cyrillic_letter) && context.token_count() >= 2 {
        14
    } else if known_surface(surface) {
        12
    } else {
        6
    }
}

fn attractor_margin(token: &str, surface: &str, route: SurfaceProductionRoute) -> i16 {
    match route {
        SurfaceProductionRoute::LayoutTransducer => 18,
        SurfaceProductionRoute::GraphemeWave => {
            let distance = damerau_levenshtein(&token.to_lowercase(), &surface.to_lowercase());
            (18 - (distance as i16 * 4)).max(4)
        }
        _ => 4,
    }
}

fn anti_confusion_penalty(token: &str, surface: &str, context: &TailContext) -> i16 {
    let mut penalty = 2;
    if token.chars().count() <= 2 {
        penalty += 8;
    }
    if context.has_technical_context() {
        penalty += 24;
    }
    if surface.eq_ignore_ascii_case(token) {
        penalty += 20;
    }
    penalty
}

fn lexical_energy(gate: &LexicalBirthGate) -> f32 {
    (gate.total_score as f32 / 100.0).clamp(0.0, 0.96)
}

fn lexical_risk(gate: &LexicalBirthGate) -> f32 {
    (0.28 - gate.total_score as f32 / 500.0).clamp(0.04, 0.40)
}

fn surface_suffix(surface: &str) -> &str {
    surface
        .char_indices()
        .rev()
        .nth(2)
        .map(|(idx, _)| &surface[idx..])
        .unwrap_or(surface)
}

fn stable_hash(text: &str) -> u32 {
    let mut hash = 0x811c_9dc5_u32;
    for byte in text.as_bytes() {
        hash ^= u32::from(*byte);
        hash = hash.wrapping_mul(0x0100_0193);
    }
    hash
}

#[cfg(test)]
mod tests {
    use super::*;

    fn context(text: &str) -> TailContext {
        TailContext::from_text(text.trim_end())
    }

    #[test]
    fn records_are_exactly_32_bytes() {
        assert_eq!(core::mem::size_of::<LexicalBirthCandidate32>(), 32);
        assert_eq!(core::mem::size_of::<LexicalBindingRecord32>(), 32);
    }

    #[test]
    fn layout_route_produces_surface_without_token_id_lookup() {
        let context = context("мы djn");
        let traces = lexical_attractor_traces("мы djn", &context);
        let trace = traces
            .iter()
            .find(|trace| trace.route == SurfaceProductionRoute::LayoutTransducer)
            .expect("layout trace");
        assert_eq!(trace.produced_surface, "мы вот");
        assert!(trace.gate.accepted);
        assert!(trace.accepted_binding.is_some());
    }

    #[test]
    fn accepted_candidate_exposes_wave_route_without_surface_lookup() {
        let context = context("мы djn");
        let candidates = lexical_attractor_candidates("мы djn ", &context);
        let candidate = candidates
            .iter()
            .find(|candidate| candidate.source == LEXICAL_ATTRACTOR_CELL)
            .expect("lexical attractor candidate");

        assert_eq!(candidate.text, "мы вот");
        assert!(candidate
            .support
            .iter()
            .any(|item| item == "surface-route=layout_transducer"));
        assert!(candidate
            .support
            .iter()
            .any(|item| item.starts_with("form-wave=")));
        assert!(candidate
            .support
            .iter()
            .all(|item| !item.contains("token_id")));
    }

    #[test]
    fn l2_pipeline_exposes_attractor_candidate_but_keeps_existing_layout_owner() {
        let l1 = crate::nanda_wave::l1::run_l1("мы djn ");
        let candidates = crate::nanda_wave::l2::run_l2_with_options(
            "мы djn ",
            &l1,
            &crate::nanda_wave::WaveOptions::default(),
        );

        assert!(candidates
            .iter()
            .any(|candidate| candidate.source == LEXICAL_ATTRACTOR_CELL
                && candidate.text == "мы вот"));
        assert!(candidates
            .iter()
            .any(|candidate| candidate.source == "LayoutWordCell32" && candidate.text == "мы вот"));
    }

    #[test]
    fn l3_does_not_apply_attractor_candidate_as_standalone_source() {
        let candidate = WordCandidate {
            text: "мы вот".to_string(),
            source: LEXICAL_ATTRACTOR_CELL,
            energy: 0.96,
            risk: 0.04,
            support: Vec::new(),
        };

        let (_trace, decision) = crate::nanda_wave::l3::run_l3("мы djn ", &[candidate]);

        assert!(matches!(
            decision,
            crate::nanda_wave::WaveDecision::Keep { .. }
        ));
    }

    #[test]
    fn typo_route_repairs_known_russian_surface_with_phrase_context() {
        let context = context("мы превет");
        let traces = lexical_attractor_traces("мы превет", &context);
        let trace = traces
            .iter()
            .find(|trace| trace.route == SurfaceProductionRoute::GraphemeWave)
            .expect("typo trace");

        assert_eq!(trace.produced_surface, "мы привет");
        assert!(trace.gate.accepted);
        assert!(trace.accepted_binding.is_some());
    }

    #[test]
    fn typo_route_does_not_rewrite_known_verb_form() {
        let context = context("проверка можем");
        let traces = lexical_attractor_traces("проверка можем", &context);

        assert!(
            traces
                .iter()
                .all(|trace| trace.route != SurfaceProductionRoute::GraphemeWave),
            "known Russian verb form must not become a neighboring attractor: {traces:?}"
        );
    }

    #[test]
    fn typo_route_requires_context_before_birth() {
        let context = context("превет");
        let traces = lexical_attractor_traces("превет", &context);

        assert!(traces
            .iter()
            .all(|trace| trace.route != SurfaceProductionRoute::GraphemeWave));
    }

    #[test]
    fn one_char_fragment_does_not_become_accepted_word() {
        let context = context("я b");
        let traces = lexical_attractor_traces("я b", &context);
        assert!(traces.iter().all(|trace| !trace.gate.accepted));
    }

    #[test]
    fn technical_context_blocks_unsafe_birth() {
        let context = context("git djn");
        let traces = lexical_attractor_traces("git djn", &context);
        assert!(traces.iter().all(|trace| !trace.gate.accepted));
        assert!(traces
            .iter()
            .all(|trace| trace.route != SurfaceProductionRoute::LayoutTransducer));
    }
}

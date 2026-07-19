use super::grammar_adapter::{agree_adjective_like_tail, clean_ru_token};
use super::{
    candidate_support, l1_energy, push_unique_candidate, TailContext, TokenKind, WaveOptions,
    WavePacket, WordCandidate, L2_SURFACE_MOTIF_CELL,
};
use crate::candidate_contract::CandidateOrigin;
use crate::config::CorrectionSafety;
use crate::keyboard::is_cyrillic_letter;
use crate::typing_candidate::TypingCandidateFamily;

const MAX_TAUGHT_CANDIDATES: usize = 6;

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

pub(super) fn taught_candidates(
    original: &str,
    context: &TailContext,
    l1: &[WavePacket],
    options: &WaveOptions,
) -> Vec<WordCandidate> {
    let configured = crate::config::default_typing_assist_pipeline();
    let pipeline = crate::typing_context::typing_assist_pipeline_for_context(
        true,
        CorrectionSafety::Experimental,
        &configured,
        original,
    );
    let explanation =
        crate::typing_pipeline::explain_typing_assist_with_pipeline(original, true, &pipeline);
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

pub(super) fn should_run_taught_candidates(token: &str, options: &WaveOptions) -> bool {
    !(options.is_enabled(L2_SURFACE_MOTIF_CELL)
        && token.chars().count() >= 4
        && token.chars().all(is_cyrillic_letter))
}

fn taught_word_candidate(input: TaughtCandidateInput<'_>) -> Option<WordCandidate> {
    let replacement = input.replacement.trim_end();
    if replacement == input.original.trim_end() {
        return None;
    }
    let (source, origin) = match input.family {
        TypingCandidateFamily::Layout if input.options.is_enabled("LayoutWordCell32") => {
            ("LayoutWordCell32", CandidateOrigin::Layout)
        }
        TypingCandidateFamily::Structural if input.options.is_enabled("BoundaryCell32") => {
            ("BoundaryCell32", CandidateOrigin::Boundary)
        }
        TypingCandidateFamily::Typo | TypingCandidateFamily::Exact
            if is_phrase_grammar_candidate(input.context, input.original, replacement)
                && input.options.is_enabled("GrammarCell32") =>
        {
            ("GrammarCell32", CandidateOrigin::L3Context)
        }
        TypingCandidateFamily::Typo
        | TypingCandidateFamily::Visual
        | TypingCandidateFamily::Exact
        | TypingCandidateFamily::Cleanup
            if input.options.is_enabled("PhraseCell32") =>
        {
            ("PhraseCell32", CandidateOrigin::L3Context)
        }
        _ => return None,
    };
    if origin == CandidateOrigin::L3Context
        && unsafe_single_token_phrase_typo(input.original, replacement)
    {
        return None;
    }
    Some(WordCandidate {
        text: replacement.to_string(),
        origin,
        source,
        energy: taught_energy(input.score, origin, input.l1, input.chosen),
        risk: taught_risk(
            input.family,
            origin,
            input.original,
            replacement,
            input.chosen,
        ),
        support: candidate_support(input.l1, input.context),
    })
}

fn taught_energy(score: f64, origin: CandidateOrigin, l1: &[WavePacket], chosen: bool) -> f32 {
    let base = match origin {
        CandidateOrigin::Layout | CandidateOrigin::LayoutThenTypo => {
            l1_energy(l1, "KeyboardCell32")
        }
        CandidateOrigin::Boundary => l1_energy(l1, "BoundaryCell32"),
        CandidateOrigin::L3Context => {
            l1_energy(l1, "ScriptCell32").max(l1_energy(l1, "BoundaryCell32") * 0.85)
        }
        CandidateOrigin::Completion
        | CandidateOrigin::L2Surface
        | CandidateOrigin::DeterministicTypo
        | CandidateOrigin::Technical => 0.5,
    };
    let score = (score / 14.0).clamp(0.25, 0.95) as f32;
    let chosen_bonus = if chosen { 0.04 } else { 0.0 };
    (base.max(score) + chosen_bonus).min(0.99)
}

fn taught_risk(
    family: TypingCandidateFamily,
    origin: CandidateOrigin,
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
    let bad_split = if origin == CandidateOrigin::Boundary
        && compact_text(original) != compact_text(replacement)
    {
        0.50
    } else {
        0.0
    };
    let chosen_bonus = if chosen { -0.03 } else { 0.0 };
    (base + edit_ratio * 0.20 + bad_split + chosen_bonus).clamp(0.02, 0.85)
}

fn is_phrase_grammar_candidate(context: &TailContext, original: &str, replacement: &str) -> bool {
    if context.token_count() < 2
        || original.split_whitespace().count() != replacement.split_whitespace().count()
        || context.has_technical_context()
    {
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

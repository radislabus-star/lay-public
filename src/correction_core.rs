//! Shared text-correction decision facade.
//!
//! Runtime backends still own output and state. This module only answers one
//! question: should this completed text be replaced, and by which engine?

use crate::config::{CorrectionSafety, TypingAssistRuleConfig};
use crate::nanda_wave::l3_phrase_gate::{evaluate_default_candidate, L3PhraseGateDecision};
use crate::nanda_wave::{run_wave_trace, WaveDecision};
use crate::russian_typo_candidates::{
    inserted_char_position_for_missing_letter, repeated_run_deletion_candidates,
};
use crate::text_case::apply_word_case;
use crate::text_metrics::{damerau_levenshtein, has_cyrillic, has_latin};
use crate::typing_assist::{explain_typing_assist_with_pipeline, split_ws_segments};
use crate::typing_context::{syntax_allows_candidate, typing_assist_pipeline_for_context};
use crate::typing_rule_graph::ids;
use crate::word_reader::{
    cyrillic_word_splits, is_cyrillic_letters_only, split_edge_whitespace, split_word_punctuation,
};
use crate::word_recognizer::is_ascii_technical_token;

const COMPOSITE_TRANSPOSE_MIN_MARGIN: f64 = -8.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CorrectionMode {
    DeterministicOnly,
    NandaOnly,
    DeterministicThenNanda,
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
    ExtraLetter,
    RepeatedLetter,
    AdjacentTransposition,
    LetterSubstitution,
    CompositeTypo,
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
            Self::ExtraLetter => "extra-letter",
            Self::RepeatedLetter => "repeated-letter",
            Self::AdjacentTransposition => "adjacent-transposition",
            Self::LetterSubstitution => "letter-substitution",
            Self::CompositeTypo => "composite-typo",
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CandidateGateAction {
    Apply,
    SuggestOnly,
    KeepOriginal,
    Veto,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CandidateGateDecision {
    pub action: CandidateGateAction,
    pub reason: &'static str,
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
    pub source_id: String,
    pub error_class: TypingErrorClass,
    pub gate: CandidateGateDecision,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CorrectionResolution {
    pub event: TypingErrorEvent,
    pub candidates: Vec<UnifiedCorrectionCandidate>,
    pub selected: Option<UnifiedCorrectionCandidate>,
    pub decision: Option<CorrectionDecision>,
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
    pub mode: CorrectionMode,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CorrectionDecision {
    pub replacement: String,
    pub source: CorrectionDecisionSource,
}

pub fn decide_text_correction(req: CorrectionRequest<'_>) -> Option<CorrectionDecision> {
    resolve_text_correction(req).decision
}

pub fn resolve_text_correction(req: CorrectionRequest<'_>) -> CorrectionResolution {
    let mut board = CandidateBoard::new(TypingErrorEvent::from_text(req.text));

    match req.mode {
        CorrectionMode::DeterministicOnly => {
            board.push(deterministic_text_correction(&req));
        }
        CorrectionMode::NandaOnly => {
            board.push(nanda_text_correction(&req));
        }
        CorrectionMode::DeterministicThenNanda => {
            board.push(deterministic_text_correction(&req));
            board.push(nanda_text_correction(&req));
        }
    }

    board.into_resolution()
}

struct CandidateBoard {
    event: TypingErrorEvent,
    candidates: Vec<UnifiedCorrectionCandidate>,
}

impl CandidateBoard {
    fn new(event: TypingErrorEvent) -> Self {
        Self {
            event,
            candidates: Vec::new(),
        }
    }

    fn push(&mut self, candidate: Option<UnifiedCorrectionCandidate>) {
        if let Some(candidate) = candidate {
            self.candidates.push(candidate);
        }
    }

    fn selected_apply_candidate(&self) -> Option<UnifiedCorrectionCandidate> {
        self.candidates
            .iter()
            .filter(|candidate| candidate.gate.action == CandidateGateAction::Apply)
            .cloned()
            .max_by(|left, right| {
                bayes_score_for_candidate(&self.event.original, left)
                    .posterior
                    .total_cmp(&bayes_score_for_candidate(&self.event.original, right).posterior)
            })
    }

    fn into_resolution(self) -> CorrectionResolution {
        let selected = self.selected_apply_candidate();
        let decision = selected.as_ref().map(|candidate| CorrectionDecision {
            replacement: candidate.replacement.clone(),
            source: candidate.source,
        });

        CorrectionResolution {
            event: self.event,
            candidates: self.candidates,
            selected,
            decision,
        }
    }
}

impl TypingErrorEvent {
    fn from_text(text: &str) -> Self {
        let (_, core, _) = split_edge_whitespace(text);
        let current_word = last_text_word(core).unwrap_or_default();
        let input_class = classify_input_word(&current_word);

        Self {
            original: text.to_string(),
            core: core.to_string(),
            current_word,
            input_class,
        }
    }
}

fn deterministic_text_correction(
    req: &CorrectionRequest<'_>,
) -> Option<UnifiedCorrectionCandidate> {
    if !(req.auto_replace || req.typing_assist || req.auto_switch_layout) {
        return None;
    }

    let pipeline = typing_assist_pipeline_for_context(
        req.auto_replace,
        req.correction_safety,
        req.typing_assist_pipeline,
        req.text,
    );
    let explanation =
        explain_typing_assist_with_pipeline(req.text, req.auto_switch_layout, &pipeline);
    let Some(replacement) = explanation.output else {
        return deterministic_composite_text_correction(req, &pipeline);
    };
    let rule_id = explanation
        .chosen
        .as_ref()
        .map(|candidate| candidate.rule_id.as_str())
        .unwrap_or("deterministic");
    let error_class = rule_error_class(rule_id);
    let gate = gate_candidate_with_source(req.text, &replacement, error_class, rule_id);
    if matches!(
        error_class,
        TypingErrorClass::RepeatedLetter | TypingErrorClass::ExtraLetter
    ) {
        if let Some(composite) = deterministic_composite_text_correction(req, &pipeline) {
            if should_prefer_composite_after_repeated_repair(
                req.text,
                &replacement,
                &composite.replacement,
            ) {
                return Some(composite);
            }
        }
    }
    if gate.action != CandidateGateAction::Apply {
        return deterministic_composite_text_correction(req, &pipeline).or(Some(
            UnifiedCorrectionCandidate {
                replacement,
                source: CorrectionDecisionSource::Deterministic,
                source_id: rule_id.to_string(),
                error_class,
                gate,
            },
        ));
    }

    Some(UnifiedCorrectionCandidate {
        replacement,
        source: CorrectionDecisionSource::Deterministic,
        source_id: rule_id.to_string(),
        error_class,
        gate,
    })
}

fn deterministic_composite_text_correction(
    req: &CorrectionRequest<'_>,
    pipeline: &[TypingAssistRuleConfig],
) -> Option<UnifiedCorrectionCandidate> {
    layout_then_typo_candidate(req, pipeline)
        .or_else(|| repeated_letter_fallback_candidate(req))
        .or_else(|| composite_russian_typo_candidate(req))
}

fn repeated_letter_fallback_candidate(
    req: &CorrectionRequest<'_>,
) -> Option<UnifiedCorrectionCandidate> {
    if !req.typing_assist && !req.auto_replace {
        return None;
    }
    let (_, core, _) = split_edge_whitespace(req.text);
    let current_word = last_text_word(core)?;
    let replacement_word = crate::ru_typo::correct_repeated_letter(&current_word)
        .or_else(|| unique_known_repeated_deletion_word(&current_word))?;
    let replacement = replace_last_text_word(req.text, &replacement_word)?;
    if replacement == req.text || !syntax_allows_candidate(req.text, &replacement) {
        return None;
    }

    let source_id = ids::REPEATED_LETTER;
    let gate = gate_candidate_with_source(
        req.text,
        &replacement,
        TypingErrorClass::RepeatedLetter,
        source_id,
    );
    Some(UnifiedCorrectionCandidate {
        replacement,
        source: CorrectionDecisionSource::Deterministic,
        source_id: source_id.to_string(),
        error_class: TypingErrorClass::RepeatedLetter,
        gate,
    })
}

fn unique_known_repeated_deletion_word(word: &str) -> Option<String> {
    let lower = word.to_lowercase();
    let mut candidates = repeated_run_deletion_candidates(&lower)
        .into_iter()
        .filter(|candidate| {
            crate::russian_lexicon::is_known_russian_word_or_form(candidate)
                || crate::lexicon::is_common_ru_word(candidate)
        })
        .collect::<Vec<_>>();
    candidates.sort();
    candidates.dedup();
    let [candidate] = candidates.as_slice() else {
        return None;
    };
    Some(apply_word_case(word, candidate))
}

fn layout_then_typo_candidate(
    req: &CorrectionRequest<'_>,
    pipeline: &[TypingAssistRuleConfig],
) -> Option<UnifiedCorrectionCandidate> {
    if !req.auto_switch_layout {
        return None;
    }

    let (_, core, _) = split_edge_whitespace(req.text);
    let current_word = last_text_word(core)?;
    if !looks_like_ascii_layout_word(&current_word) {
        return None;
    }

    let converted_word = crate::dict::convert(&current_word, crate::dict::Direction::Us2Ru);
    if converted_word == current_word || !is_cyrillic_letters_only(&converted_word) {
        return None;
    }

    let converted_text = replace_last_text_word(req.text, &converted_word)?;
    let explanation = explain_typing_assist_with_pipeline(&converted_text, false, pipeline);
    let final_replacement = explanation.output.unwrap_or_else(|| {
        if crate::russian_lexicon::is_known_russian_word_or_form(&converted_word) {
            converted_text.clone()
        } else {
            String::new()
        }
    });
    if final_replacement.is_empty() || final_replacement == req.text {
        return None;
    }
    let source_id = explanation
        .chosen
        .as_ref()
        .map(|candidate| format!("layout_then_{}", candidate.rule_id))
        .unwrap_or_else(|| "layout_then_known_word".to_string());
    let gate = gate_candidate_with_source(
        req.text,
        &final_replacement,
        TypingErrorClass::CompositeTypo,
        &source_id,
    );
    Some(UnifiedCorrectionCandidate {
        replacement: final_replacement,
        source: CorrectionDecisionSource::Deterministic,
        source_id,
        error_class: TypingErrorClass::CompositeTypo,
        gate,
    })
}

fn composite_russian_typo_candidate(
    req: &CorrectionRequest<'_>,
) -> Option<UnifiedCorrectionCandidate> {
    if !req.typing_assist && !req.auto_replace {
        return None;
    }

    let (_, core, _) = split_edge_whitespace(req.text);
    let current_word = last_text_word(core)?;
    let lower = current_word.to_lowercase();
    if !is_cyrillic_letters_only(&current_word)
        || crate::russian_lexicon::is_known_russian_word_or_form(&lower)
    {
        return None;
    }

    if let Some(replacement_word) = unique_adjacent_transposition_word(&current_word) {
        let replacement = replace_last_word_and_split_previous_glued(req.text, &replacement_word)
            .or_else(|| replace_last_text_word(req.text, &replacement_word))?;
        if replacement != req.text && syntax_allows_candidate(req.text, &replacement) {
            let source_id = ids::ADJACENT_TRANSPOSITION;
            let gate = gate_candidate_with_source(
                req.text,
                &replacement,
                TypingErrorClass::AdjacentTransposition,
                source_id,
            );
            return Some(UnifiedCorrectionCandidate {
                replacement,
                source: CorrectionDecisionSource::Deterministic,
                source_id: source_id.to_string(),
                error_class: TypingErrorClass::AdjacentTransposition,
                gate,
            });
        }
    }

    if lower.chars().count() < 4 {
        return None;
    }

    if let Some(candidate) = repeated_prefix_composite_word(&lower) {
        let replacement_word = apply_word_case(&current_word, &candidate);
        let replacement = replace_last_word_and_split_previous_glued(req.text, &replacement_word)
            .or_else(|| replace_last_text_word(req.text, &replacement_word))?;
        if replacement != req.text && syntax_allows_candidate(req.text, &replacement) {
            let source_id = "composite_ru_typo";
            let gate = gate_candidate_with_source(
                req.text,
                &replacement,
                TypingErrorClass::CompositeTypo,
                source_id,
            );
            return Some(UnifiedCorrectionCandidate {
                replacement,
                source: CorrectionDecisionSource::Deterministic,
                source_id: source_id.to_string(),
                error_class: TypingErrorClass::CompositeTypo,
                gate,
            });
        }
    }

    let (candidate, _) = crate::candidate_ranker::choose_best_with_gap(
        crate::ru_typo::fuzzy_known_word_candidates(&lower),
        0.85,
        |candidate| {
            if candidate == &lower
                || repeated_run_deletion_candidates(&lower)
                    .iter()
                    .any(|repaired| repaired == candidate)
                || !crate::russian_lexicon::is_known_russian_word_or_form(candidate)
            {
                return None;
            }
            let distance = damerau_levenshtein(&lower, candidate);
            if distance == 0 || distance > 3 {
                return None;
            }
            let inserted = candidate
                .chars()
                .count()
                .saturating_sub(lower.chars().count());
            if risky_short_initial_insertion(&lower, candidate) {
                return None;
            }
            let common_word_recovery = lower.chars().count() >= 7
                && inserted <= 2
                && distance <= 3
                && repeated_run_deletion_candidates(&lower).is_empty()
                && crate::lexicon::is_common_ru_word(candidate);
            if !common_word_recovery
                && !compatible_composite_typo_shape(&lower, candidate, distance)
            {
                return None;
            }
            let margin = crate::ngram::ru_candidate_margin(candidate, &lower);
            if inserted > 1 && !common_word_recovery {
                return None;
            }
            let shape_bonus = inserted as f64 * 8.0;
            let close_insert_bonus = if distance == 1 && inserted == 1 {
                12.0
            } else {
                0.0
            };
            let common_word_bonus = if common_word_recovery { 12.0 } else { 0.0 };
            let repeated_repair_bonus =
                if repeated_prefix_typo_shape_is_preserved(&lower, candidate) {
                    8.0
                } else {
                    0.0
                };
            let initial_vowel_bonus =
                missing_initial_vowel_before_double_consonant_bonus(&lower, candidate);
            let score = margin
                + shape_bonus
                + close_insert_bonus
                + common_word_bonus
                + repeated_repair_bonus
                + initial_vowel_bonus
                - distance as f64 * 0.35;
            (score >= 0.0).then_some(score)
        },
    )?;
    let replacement_word = apply_word_case(&current_word, &candidate);
    let replacement = replace_last_word_and_split_previous_glued(req.text, &replacement_word)
        .or_else(|| replace_last_text_word(req.text, &replacement_word))?;
    if replacement == req.text {
        return None;
    }
    if !syntax_allows_candidate(req.text, &replacement) {
        return None;
    }

    let source_id = "composite_ru_typo";
    let gate = gate_candidate_with_source(
        req.text,
        &replacement,
        TypingErrorClass::CompositeTypo,
        source_id,
    );
    Some(UnifiedCorrectionCandidate {
        replacement,
        source: CorrectionDecisionSource::Deterministic,
        source_id: source_id.to_string(),
        error_class: TypingErrorClass::CompositeTypo,
        gate,
    })
}

fn repeated_prefix_composite_word(lower: &str) -> Option<String> {
    let repaired_forms = repeated_run_deletion_candidates(lower);
    if repaired_forms.is_empty() {
        return None;
    }
    crate::candidate_ranker::choose_best_with_gap(
        crate::ru_typo::fuzzy_known_word_candidates(lower)
            .into_iter()
            .filter(|candidate| {
                candidate != lower
                    && !repaired_forms.iter().any(|repaired| repaired == candidate)
                    && crate::russian_lexicon::is_known_russian_word_or_form(candidate)
                    && repaired_forms
                        .iter()
                        .any(|repaired| damerau_levenshtein(repaired, candidate) <= 1)
            }),
        0.25,
        |candidate| {
            let best_repaired_distance = repaired_forms
                .iter()
                .map(|repaired| damerau_levenshtein(repaired, candidate))
                .min()?;
            if best_repaired_distance > 1 {
                return None;
            }
            let margin = crate::ngram::ru_candidate_margin(candidate, lower);
            Some(margin + 8.0 - best_repaired_distance as f64 * 0.35)
        },
    )
    .map(|(candidate, _)| candidate)
}

fn nanda_text_correction(req: &CorrectionRequest<'_>) -> Option<UnifiedCorrectionCandidate> {
    if !req.nanda_autocorrect {
        return None;
    }

    let trace = run_wave_trace(req.text);
    match &trace.decision {
        WaveDecision::Apply { text, .. } if text != req.text => {
            let source_id = accepted_wave_source(&trace, text).unwrap_or("NANDA");
            let error_class = nanda_source_error_class(source_id);
            let gate = gate_candidate_with_source(req.text, text, error_class, source_id);
            Some(UnifiedCorrectionCandidate {
                replacement: text.clone(),
                source: CorrectionDecisionSource::Nanda,
                source_id: source_id.to_string(),
                error_class,
                gate,
            })
        }
        WaveDecision::Apply { .. } | WaveDecision::Keep { .. } | WaveDecision::Veto { .. } => None,
    }
}

fn last_text_word(core: &str) -> Option<String> {
    split_ws_segments(core)
        .into_iter()
        .rev()
        .find_map(|(segment, is_ws)| {
            if is_ws {
                return None;
            }
            let (_, word, _) = split_word_punctuation(segment);
            (!word.is_empty()).then(|| word.to_string())
        })
}

fn unique_adjacent_transposition_word(word: &str) -> Option<String> {
    if word.chars().count() < 5 || !is_cyrillic_letters_only(word) {
        return None;
    }

    let lower = word.to_lowercase();
    if crate::russian_lexicon::is_known_russian_word_or_form(&lower) {
        return None;
    }
    let chars: Vec<char> = lower.chars().collect();
    let mut found: Option<String> = None;

    for idx in 0..chars.len().saturating_sub(1) {
        if chars[idx] == chars[idx + 1] {
            continue;
        }

        let mut candidate = chars.clone();
        candidate.swap(idx, idx + 1);
        let candidate: String = candidate.into_iter().collect();
        if candidate == lower || !crate::russian_lexicon::is_known_russian_word_or_form(&candidate)
        {
            continue;
        }
        if crate::ngram::ru_candidate_margin(&candidate, &lower) < COMPOSITE_TRANSPOSE_MIN_MARGIN {
            continue;
        }

        if found.is_some() {
            return None;
        }
        found = Some(candidate);
    }

    found.map(|candidate| apply_word_case(word, &candidate))
}

fn replace_last_text_word(text: &str, replacement_word: &str) -> Option<String> {
    let (leading_ws, core, trailing_ws) = split_edge_whitespace(text);
    let segments = split_ws_segments(core);
    let replace_idx = segments
        .iter()
        .enumerate()
        .rev()
        .find_map(|(idx, (_, is_ws))| (!*is_ws).then_some(idx))?;

    let mut output = String::with_capacity(text.len() + replacement_word.len());
    output.push_str(leading_ws);
    for (idx, (segment, _is_ws)) in segments.iter().enumerate() {
        if idx == replace_idx {
            let (token_leading, word, token_trailing) = split_word_punctuation(segment);
            if word.is_empty() {
                return None;
            }
            output.push_str(token_leading);
            output.push_str(replacement_word);
            output.push_str(token_trailing);
        } else {
            output.push_str(segment);
        }
    }
    output.push_str(trailing_ws);
    Some(output)
}

fn replace_last_word_and_split_previous_glued(
    text: &str,
    replacement_word: &str,
) -> Option<String> {
    let (leading_ws, core, trailing_ws) = split_edge_whitespace(text);
    let segments = split_ws_segments(core);
    let word_indices: Vec<usize> = segments
        .iter()
        .enumerate()
        .filter_map(|(idx, (_, is_ws))| (!*is_ws).then_some(idx))
        .collect();
    let [.., prev_idx, last_idx] = word_indices.as_slice() else {
        return None;
    };

    let (prev_leading, prev_word, prev_trailing) = split_word_punctuation(segments[*prev_idx].0);
    let (last_leading, last_word, last_trailing) = split_word_punctuation(segments[*last_idx].0);
    if !prev_leading.is_empty()
        || !prev_trailing.is_empty()
        || !last_leading.is_empty()
        || prev_word.is_empty()
        || last_word.is_empty()
        || !is_cyrillic_letters_only(prev_word)
        || !is_cyrillic_letters_only(last_word)
    {
        return None;
    }

    let split_previous = split_previous_glued_before_typo(prev_word)?;
    let mut output = String::with_capacity(text.len() + replacement_word.len() + 1);
    output.push_str(leading_ws);
    for (idx, (segment, _is_ws)) in segments.iter().enumerate() {
        if idx == *prev_idx {
            output.push_str(&split_previous);
        } else if idx == *last_idx {
            output.push_str(replacement_word);
            output.push_str(last_trailing);
        } else {
            output.push_str(segment);
        }
    }
    output.push_str(trailing_ws);

    (output != text).then_some(output)
}

fn split_previous_glued_before_typo(word: &str) -> Option<String> {
    let lower = word.to_lowercase();
    if lower.chars().count() < 8 || crate::russian_lexicon::is_known_russian_word_or_form(&lower) {
        return None;
    }

    let mut candidates = Vec::new();
    for split in cyrillic_word_splits(&lower) {
        if split.left_len < 4 || split.left_len > 8 || split.right_len < 5 {
            continue;
        }
        if !crate::phrase_lexicon::is_known_russian_phrase_part(split.left) {
            continue;
        }
        if !crate::phrase_lexicon::is_known_russian_phrase_part(split.right)
            && !looks_like_contextual_russian_verb_form(split.right)
        {
            continue;
        }

        let candidate = format!("{} {}", split.left, split.right);
        let margin = crate::ngram::ru_candidate_margin(&candidate, &lower);
        if margin < -30.0 {
            continue;
        }
        let verb_bonus = if looks_like_contextual_russian_verb_form(split.right) {
            2.0
        } else {
            0.0
        };
        candidates.push((candidate, margin + verb_bonus));
    }

    let ((candidate, _), _) =
        crate::candidate_ranker::choose_best_with_gap(candidates, 0.75, |(_, score)| Some(*score))?;
    Some(crate::text_case::apply_phrase_case(word, &candidate))
}

fn looks_like_contextual_russian_verb_form(word: &str) -> bool {
    word.chars().count() >= 5
        && [
            "ается",
            "яется",
            "уется",
            "ется",
            "етесь",
            "итесь",
            "аешь",
            "яешь",
            "уешь",
            "ешь",
            "ишь",
            "аете",
            "яете",
            "уете",
            "аете",
            "яете",
            "ует",
            "ает",
            "яет",
            "ете",
            "ите",
            "ают",
            "яют",
            "уют",
            "ет",
            "ит",
            "ут",
            "ют",
            "ат",
            "ят",
        ]
        .iter()
        .any(|ending| word.ends_with(ending))
}

fn looks_like_ascii_layout_word(word: &str) -> bool {
    word.is_ascii()
        && word.chars().filter(|ch| ch.is_ascii_alphabetic()).count() >= 3
        && !word.chars().any(|ch| ch.is_ascii_digit())
        && word.chars().all(|ch| {
            ch.is_ascii_alphabetic()
                || matches!(
                    ch,
                    '\'' | ';'
                        | '['
                        | ']'
                        | '`'
                        | ','
                        | '.'
                        | '-'
                        | '{'
                        | '}'
                        | ':'
                        | '"'
                        | '<'
                        | '>'
                        | '~'
                )
        })
}

fn loose_original_shape_is_preserved(original: &str, candidate: &str) -> bool {
    if candidate.chars().count() < original.chars().count() {
        return false;
    }

    let mut original_chars = original.chars().map(loose_shape_char);
    let mut needed = original_chars.next();
    for ch in candidate.chars().map(loose_shape_char) {
        if needed == Some(ch) {
            needed = original_chars.next();
            if needed.is_none() {
                return true;
            }
        }
    }
    needed.is_none()
}

fn compatible_composite_typo_shape(original: &str, candidate: &str, distance: usize) -> bool {
    if loose_original_shape_is_preserved(original, candidate) {
        return true;
    }
    if repeated_prefix_typo_shape_is_preserved(original, candidate) {
        return true;
    }

    distance == 1 && original.chars().count() == candidate.chars().count()
}

fn missing_initial_vowel_before_double_consonant_bonus(original: &str, candidate: &str) -> f64 {
    let Some((idx, inserted)) = inserted_char_position_for_missing_letter(original, candidate)
    else {
        return 0.0;
    };
    if idx != 0 {
        return 0.0;
    }
    let mut chars = original.chars();
    let Some(first) = chars.next() else {
        return 0.0;
    };
    if chars.next() != Some(first) {
        return 0.0;
    }
    match inserted {
        'э' => 10.0,
        'а' | 'о' | 'и' | 'у' | 'е' | 'ё' | 'ю' | 'я' => 2.0,
        _ => 0.0,
    }
}

fn risky_short_initial_insertion(original: &str, candidate: &str) -> bool {
    if original.chars().count() > 6 {
        return false;
    }
    let Some((idx, inserted)) = inserted_char_position_for_missing_letter(original, candidate)
    else {
        return false;
    };
    idx == 0 && (!crate::russian_chars::is_russian_vowel(inserted) || original.chars().count() <= 6)
}

fn repeated_prefix_typo_shape_is_preserved(original: &str, candidate: &str) -> bool {
    repeated_run_deletion_candidates(original)
        .into_iter()
        .any(|repaired| {
            if loose_original_shape_is_preserved(&repaired, candidate) {
                return true;
            }
            repaired.chars().count() == candidate.chars().count()
                && damerau_levenshtein(&repaired, candidate) <= 1
        })
}

fn loose_shape_char(ch: char) -> char {
    match ch {
        'ё' => 'е',
        'Ё' => 'Е',
        'щ' => 'ш',
        'Щ' => 'Ш',
        other => other,
    }
}

fn classify_input_word(word: &str) -> TypingErrorClass {
    if word.is_empty() {
        return TypingErrorClass::Unknown;
    }
    if is_ascii_technical_token(word) {
        return TypingErrorClass::TechnicalToken;
    }
    if has_cyrillic(word) && has_latin(word) {
        return TypingErrorClass::MixedScript;
    }
    if is_cyrillic_letters_only(word)
        && !crate::russian_lexicon::is_known_russian_word_or_form(word)
    {
        return TypingErrorClass::CompositeTypo;
    }
    if word.chars().all(|ch| ch.is_ascii_alphabetic()) && word.chars().count() >= 3 {
        return TypingErrorClass::WrongLayout;
    }
    TypingErrorClass::Unknown
}

fn rule_error_class(rule_id: &str) -> TypingErrorClass {
    match rule_id {
        ids::MIXED_SCRIPT_LAYOUT | ids::DUPLICATE_LAYOUT_PREFIX => TypingErrorClass::MixedScript,
        ids::LAYOUT_TECHNICAL => TypingErrorClass::TechnicalToken,
        ids::FAST_LAYOUT_EN_TO_RU
        | ids::CONTEXTUAL_RU_CONJUNCTION_I
        | ids::CONTEXTUAL_RU_PREPOSITION_V
        | ids::LAYOUT_RU_TO_EN
        | ids::LAYOUT_EN_TO_RU
        | ids::CONTEXTUAL_LAYOUT_EN_TO_RU
        | ids::EXPERIMENTAL_LAYOUT_EN_TO_RU
        | ids::EXPERIMENTAL_LAYOUT_RU_TO_EN
        | ids::VISUAL_B => TypingErrorClass::WrongLayout,
        ids::MOVED_PREFIX_PAIR => TypingErrorClass::PartialLayout,
        ids::SPLIT_WORD_PAIR => TypingErrorClass::SplitWord,
        ids::CYRILLIC_CASE => TypingErrorClass::CaseNoise,
        ids::HARD_SIGN | ids::SINGLE_LETTER_SUBSTITUTION | ids::VOWEL_CONFUSION => {
            TypingErrorClass::LetterSubstitution
        }
        ids::ADJACENT_TRANSPOSITION => TypingErrorClass::AdjacentTransposition,
        ids::REPEATED_LETTER => TypingErrorClass::RepeatedLetter,
        ids::EXTRA_LETTERS => TypingErrorClass::ExtraLetter,
        ids::MISSING_LETTER => TypingErrorClass::MissingLetter,
        ids::VERB_ENDING => TypingErrorClass::GrammarAgreement,
        ids::GLUED_PHRASE => TypingErrorClass::GluedWords,
        ids::PERSONAL_PHRASE | ids::PERSONAL_TOKEN => TypingErrorClass::CompositeTypo,
        _ => TypingErrorClass::Unknown,
    }
}

fn nanda_source_error_class(source: &str) -> TypingErrorClass {
    match source {
        "LayoutWordCell32" => TypingErrorClass::WrongLayout,
        "ShortTokenCell32" => TypingErrorClass::PartialLayout,
        "TechTokenCell32" | "TechnicalContextCell32" => TypingErrorClass::TechnicalToken,
        "BoundaryCell32" => TypingErrorClass::GluedWords,
        "GrammarCell32" => TypingErrorClass::GrammarAgreement,
        "PhraseForecastCell32" | "L2WordAttractorCell32" | "L2SurfaceCompletionCell32" => {
            TypingErrorClass::CompletionOnly
        }
        "CommonRuFixCell32"
        | "LearnedMemoryCell32"
        | "PhraseMemoryCell32"
        | "PhraseCell32"
        | "L2SurfaceMotifCell32"
        | "SemanticWordCell32" => TypingErrorClass::CompositeTypo,
        _ => TypingErrorClass::Unknown,
    }
}

#[cfg(test)]
fn gate_candidate(
    original: &str,
    replacement: &str,
    error_class: TypingErrorClass,
) -> CandidateGateDecision {
    gate_candidate_with_source(original, replacement, error_class, "candidate_gate")
}

fn gate_candidate_with_source(
    original: &str,
    replacement: &str,
    error_class: TypingErrorClass,
    source_id: &str,
) -> CandidateGateDecision {
    if original == replacement {
        return CandidateGateDecision {
            action: CandidateGateAction::KeepOriginal,
            reason: "unchanged",
        };
    }
    if replacement_glues_separate_words_without_boundary_class(original, replacement, error_class) {
        return CandidateGateDecision {
            action: CandidateGateAction::SuggestOnly,
            reason: "word_count_shrink_requires_boundary_class",
        };
    }
    if boundary_candidate_splits_known_russian_word(original, replacement, error_class) {
        return CandidateGateDecision {
            action: CandidateGateAction::SuggestOnly,
            reason: "known_single_word_boundary_split",
        };
    }
    if boundary_candidate_splits_to_short_function_and_weak_tail(original, replacement, error_class)
    {
        return CandidateGateDecision {
            action: CandidateGateAction::SuggestOnly,
            reason: "weak_boundary_split_tail",
        };
    }
    if semantic_wave_candidate_lacks_surface_authority(original, replacement, source_id) {
        return CandidateGateDecision {
            action: CandidateGateAction::SuggestOnly,
            reason: "semantic_wave_surface_authority_low",
        };
    }
    if error_class == TypingErrorClass::CompletionOnly {
        return CandidateGateDecision {
            action: CandidateGateAction::SuggestOnly,
            reason: "completion_is_not_autocorrect",
        };
    }
    if let Some(decision) = l3_context_gate(original, replacement, error_class) {
        return decision;
    }
    if let Some(reason) = crate::correction_bayes::bayes_suggest_only_reason(
        original,
        replacement,
        error_class.as_str(),
        source_id,
    ) {
        return CandidateGateDecision {
            action: CandidateGateAction::SuggestOnly,
            reason,
        };
    }

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
        TypingErrorClass::RepeatedLetter | TypingErrorClass::ExtraLetter
            if repeated_single_step_has_competing_composite(original, replacement) =>
        {
            CandidateGateDecision {
                action: CandidateGateAction::SuggestOnly,
                reason: "single_step_typo_has_competing_composite",
            }
        }
        TypingErrorClass::Unknown => CandidateGateDecision {
            action: CandidateGateAction::SuggestOnly,
            reason: "unknown_error_class",
        },
        _ => CandidateGateDecision {
            action: CandidateGateAction::Apply,
            reason: "class_allows_apply",
        },
    }
}

fn bayes_score_for_candidate(
    original: &str,
    candidate: &UnifiedCorrectionCandidate,
) -> crate::correction_bayes::BayesCandidateScore {
    crate::correction_bayes::bayes_score_candidate(
        original,
        &candidate.replacement,
        candidate.error_class.as_str(),
        &candidate.source_id,
    )
}

fn l3_context_gate(
    original: &str,
    replacement: &str,
    error_class: TypingErrorClass,
) -> Option<CandidateGateDecision> {
    if candidate_over_compresses_word(original, replacement, error_class) {
        return Some(CandidateGateDecision {
            action: CandidateGateAction::KeepOriginal,
            reason: "candidate_over_compresses_word",
        });
    }
    if known_phrase_part_only_grows_by_one_letter(original, replacement, error_class) {
        return Some(CandidateGateDecision {
            action: CandidateGateAction::SuggestOnly,
            reason: "known_phrase_part_one_letter_growth",
        });
    }
    if short_word_only_grows_initial_letter(original, replacement, error_class) {
        return Some(CandidateGateDecision {
            action: CandidateGateAction::SuggestOnly,
            reason: "short_initial_letter_growth",
        });
    }
    if short_word_gets_case_vowel_drift(original, replacement, error_class) {
        return Some(CandidateGateDecision {
            action: CandidateGateAction::SuggestOnly,
            reason: "short_case_vowel_drift",
        });
    }
    if soft_sign_word_gets_vowel_drift(original, replacement, error_class) {
        return Some(CandidateGateDecision {
            action: CandidateGateAction::SuggestOnly,
            reason: "soft_sign_vowel_drift",
        });
    }
    if short_word_gets_internal_consonant_drift(original, replacement, error_class) {
        return Some(CandidateGateDecision {
            action: CandidateGateAction::SuggestOnly,
            reason: "short_internal_consonant_drift",
        });
    }
    if short_word_same_length_multi_edit_drift(original, replacement, error_class) {
        return Some(CandidateGateDecision {
            action: CandidateGateAction::SuggestOnly,
            reason: "short_same_length_multi_edit_drift",
        });
    }
    if known_russian_word_rewritten_to_different_known_word(original, replacement, error_class) {
        return Some(CandidateGateDecision {
            action: CandidateGateAction::SuggestOnly,
            reason: "known_word_to_different_known_word",
        });
    }
    if short_layout_candidate_lacks_phrase_context(original, replacement, error_class) {
        return Some(CandidateGateDecision {
            action: CandidateGateAction::KeepOriginal,
            reason: "short_layout_without_phrase_context",
        });
    }
    if let Some(decision) = l3_phrase_memory_gate(original, replacement, error_class) {
        return Some(decision);
    }
    None
}

fn l3_phrase_memory_gate(
    original: &str,
    replacement: &str,
    error_class: TypingErrorClass,
) -> Option<CandidateGateDecision> {
    if !l3_phrase_memory_applies_to(error_class) {
        return None;
    }
    let report = evaluate_default_candidate(original, replacement)?;
    match report.decision {
        L3PhraseGateDecision::Support => Some(CandidateGateDecision {
            action: CandidateGateAction::Apply,
            reason: "l3_phrase_memory_support",
        }),
        L3PhraseGateDecision::Suppress => Some(CandidateGateDecision {
            action: CandidateGateAction::KeepOriginal,
            reason: "l3_phrase_memory_conflict",
        }),
        L3PhraseGateDecision::Neutral => None,
    }
}

fn l3_phrase_memory_applies_to(error_class: TypingErrorClass) -> bool {
    matches!(
        error_class,
        TypingErrorClass::CompositeTypo
            | TypingErrorClass::MissingLetter
            | TypingErrorClass::ExtraLetter
            | TypingErrorClass::AdjacentTransposition
            | TypingErrorClass::LetterSubstitution
            | TypingErrorClass::GrammarAgreement
    )
}

fn replacement_glues_separate_words_without_boundary_class(
    original: &str,
    replacement: &str,
    error_class: TypingErrorClass,
) -> bool {
    if matches!(
        error_class,
        TypingErrorClass::SplitWord | TypingErrorClass::GluedWords
    ) {
        return false;
    }
    let original_words = core_word_count(original);
    let replacement_words = core_word_count(replacement);
    original_words >= 2 && replacement_words < original_words
}

fn boundary_candidate_splits_known_russian_word(
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
        if !same_cyrillic_token(original_word, &merged) {
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

fn semantic_wave_candidate_lacks_surface_authority(
    original: &str,
    replacement: &str,
    source_id: &str,
) -> bool {
    if source_id != crate::nanda_wave::context_wave::SEMANTIC_WORD_SOURCE {
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

    let max_len = original_lower
        .chars()
        .count()
        .max(replacement_lower.chars().count());
    let original_len = original_lower.chars().count();
    let replacement_len = replacement_lower.chars().count();
    if distance >= 2 && original_len == replacement_len {
        return true;
    }
    let prefix = common_prefix_len(&original_lower, &replacement_lower);
    let known_replacement =
        crate::russian_lexicon::is_known_russian_word_or_form(&replacement_lower)
            || crate::lexicon::is_common_ru_word(&replacement_lower);
    let known_original = crate::russian_lexicon::is_known_russian_word_or_form(&original_lower);

    if distance == 2 && original_len <= 8 && prefix >= 4 && replacement_len <= original_len + 1 {
        return true;
    }
    if distance == 2 && max_len >= 7 && prefix >= 2 && known_replacement {
        return false;
    }
    if distance == 3
        && original_len >= 9
        && max_len >= 10
        && prefix >= 3
        && known_replacement
        && !known_original
    {
        return false;
    }
    true
}

fn common_prefix_len(left: &str, right: &str) -> usize {
    left.chars()
        .zip(right.chars())
        .take_while(|(left, right)| left == right)
        .count()
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

fn known_russian_word_rewritten_to_different_known_word(
    original: &str,
    replacement: &str,
    error_class: TypingErrorClass,
) -> bool {
    if !matches!(
        error_class,
        TypingErrorClass::CompositeTypo
            | TypingErrorClass::MissingLetter
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

fn known_russian_autocorrect_token(lower: &str) -> bool {
    crate::lexicon::is_common_ru_word(lower)
        || crate::lexicon::is_ru_live_protected_word(lower)
        || crate::lexicon::is_user_protected_word(lower)
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

fn short_layout_candidate_lacks_phrase_context(
    original: &str,
    replacement: &str,
    error_class: TypingErrorClass,
) -> bool {
    if !matches!(error_class, TypingErrorClass::PartialLayout) {
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

    has_ascii_context && !has_cyrillic_context
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
        && crate::russian_lexicon::is_known_russian_word_or_form(&original_lower)
}

fn same_cyrillic_token(original: &str, candidate: &str) -> bool {
    let (_, original_word, _) = split_word_punctuation(original);
    let (_, candidate_word, _) = split_word_punctuation(candidate);
    !original_word.is_empty()
        && !candidate_word.is_empty()
        && is_cyrillic_letters_only(original_word)
        && is_cyrillic_letters_only(candidate_word)
        && original_word.to_lowercase() == candidate_word.to_lowercase()
}

fn strong_standalone_split_tail(lower: &str) -> bool {
    lower.chars().count() >= 4
        && (crate::lexicon::is_common_ru_word(lower)
            || crate::russian_lexicon::russian_dictionary().contains(lower)
            || crate::russian_lexicon::is_known_russian_adverb_o_form(lower)
            || crate::russian_lexicon::is_known_russian_ka_oblique_form(lower))
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
    let replacement_lower = replacement_word.to_lowercase();
    !crate::russian_lexicon::is_known_russian_word_or_form(&replacement_lower)
        && !crate::lexicon::is_common_ru_word(&replacement_lower)
}

fn repeated_single_step_has_competing_composite(original: &str, replacement: &str) -> bool {
    let Some(original_word) = last_text_word(original) else {
        return false;
    };
    let Some(replacement_word) = last_text_word(replacement) else {
        return false;
    };
    let original_lower = original_word.to_lowercase();
    let replacement_lower = replacement_word.to_lowercase();
    if !repeated_run_deletion_candidates(&original_lower)
        .into_iter()
        .any(|candidate| candidate == replacement_lower)
    {
        return false;
    }
    crate::ru_typo::fuzzy_known_word_candidates(&original_lower)
        .into_iter()
        .any(|candidate| {
            candidate != replacement_lower
                && crate::russian_lexicon::is_known_russian_word_or_form(&candidate)
                && damerau_levenshtein(&replacement_lower, &candidate) <= 1
        })
}

fn should_prefer_composite_after_repeated_repair(
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
        && composite_word.chars().count() >= original_word.chars().count()
        && damerau_levenshtein(&original_lower, &composite_lower) <= 1
        && crate::russian_lexicon::is_known_russian_word_or_form(&composite_lower)
    {
        return true;
    }
    repeated_run_deletion_candidates(&original_lower)
        .into_iter()
        .any(|candidate| candidate == single_lower)
        && damerau_levenshtein(&single_lower, &composite_lower) <= 1
        && crate::russian_lexicon::is_known_russian_word_or_form(&composite_lower)
}

fn accepted_wave_source<'a>(
    trace: &'a crate::nanda_wave::WaveTrace,
    replacement: &str,
) -> Option<&'a str> {
    let trimmed = replacement.trim();
    trace
        .l2_candidates
        .iter()
        .find(|candidate| candidate.text.trim() == trimmed)
        .map(|candidate| candidate.source)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::default_typing_assist_pipeline;

    fn request<'a>(
        text: &'a str,
        pipeline: &'a [TypingAssistRuleConfig],
        mode: CorrectionMode,
    ) -> CorrectionRequest<'a> {
        CorrectionRequest {
            text,
            auto_replace: true,
            typing_assist: true,
            auto_switch_layout: true,
            correction_safety: CorrectionSafety::Experimental,
            typing_assist_pipeline: pipeline,
            nanda_autocorrect: true,
            mode,
        }
    }

    #[test]
    fn deterministic_mode_corrects_wrong_layout_text() {
        let pipeline = default_typing_assist_pipeline();
        let decision = decide_text_correction(request(
            "lfdfq ",
            &pipeline,
            CorrectionMode::DeterministicOnly,
        ))
        .unwrap();
        assert_eq!(decision.replacement, "давай ");
        assert_eq!(decision.source, CorrectionDecisionSource::Deterministic);
    }

    #[test]
    fn deterministic_mode_corrects_multiword_wrong_layout_tail() {
        let pipeline = default_typing_assist_pipeline();
        let resolution = resolve_text_correction(request(
            "HF<JNF NTCN CFV ",
            &pipeline,
            CorrectionMode::DeterministicOnly,
        ));

        assert_eq!(
            resolution
                .decision
                .as_ref()
                .map(|decision| decision.replacement.as_str()),
            Some("РАБОТА ТЕСТ САМ ")
        );
        assert_eq!(
            resolution
                .selected
                .as_ref()
                .map(|candidate| candidate.gate.action),
            Some(CandidateGateAction::Apply)
        );
    }

    #[test]
    fn deterministic_mode_corrects_multiword_wrong_layout_tail_with_context_pipeline() {
        let default_pipeline = default_typing_assist_pipeline();
        let pipeline = crate::typing_context::typing_assist_pipeline_for_context(
            true,
            CorrectionSafety::Normal,
            &default_pipeline,
            "HF<JNF NTCN CFV ",
        );
        let resolution = resolve_text_correction(CorrectionRequest {
            text: "HF<JNF NTCN CFV ",
            auto_replace: true,
            typing_assist: true,
            auto_switch_layout: true,
            correction_safety: CorrectionSafety::Normal,
            typing_assist_pipeline: &pipeline,
            nanda_autocorrect: false,
            mode: CorrectionMode::DeterministicOnly,
        });

        assert_eq!(
            resolution
                .decision
                .as_ref()
                .map(|decision| decision.replacement.as_str()),
            Some("РАБОТА ТЕСТ САМ ")
        );
        assert_eq!(
            resolution
                .selected
                .as_ref()
                .map(|candidate| candidate.gate.action),
            Some(CandidateGateAction::Apply)
        );
    }

    #[test]
    fn resolution_routes_missing_letter_through_unified_gate() {
        let pipeline = default_typing_assist_pipeline();
        let resolution = resolve_text_correction(request(
            "автозаена ",
            &pipeline,
            CorrectionMode::DeterministicOnly,
        ));

        let selected = resolution
            .selected
            .clone()
            .unwrap_or_else(|| panic!("selected candidate: {resolution:?}"));
        assert_eq!(selected.replacement, "автозамена ");
        assert_eq!(selected.error_class, TypingErrorClass::MissingLetter);
        assert_eq!(selected.gate.action, CandidateGateAction::Apply);
    }

    #[test]
    fn unknown_russian_shape_is_classified_before_candidate_generation() {
        let pipeline = default_typing_assist_pipeline();
        let resolution = resolve_text_correction(request(
            "приудишна ",
            &pipeline,
            CorrectionMode::DeterministicThenNanda,
        ));

        assert_eq!(resolution.event.current_word, "приудишна");
        assert_eq!(
            resolution.event.input_class,
            TypingErrorClass::CompositeTypo
        );
        assert_eq!(resolution.decision, None);
    }

    #[test]
    fn l3_anti_shortcut_blocks_overcompressed_word_candidate() {
        let pipeline = default_typing_assist_pipeline();
        let resolution = resolve_text_correction(request(
            "патерна ",
            &pipeline,
            CorrectionMode::DeterministicOnly,
        ));

        assert_eq!(resolution.decision, None);
        assert!(resolution.candidates.iter().any(|candidate| {
            candidate.replacement == "пара "
                && candidate.gate.action == CandidateGateAction::KeepOriginal
                && candidate.gate.reason == "candidate_over_compresses_word"
        }));
    }

    #[test]
    fn l3_anti_shortcut_blocks_short_layout_without_phrase_context() {
        let pipeline = default_typing_assist_pipeline();
        let resolution =
            resolve_text_correction(request("wave b ", &pipeline, CorrectionMode::NandaOnly));

        assert_eq!(resolution.decision, None);
        assert!(
            resolution.candidates.is_empty(),
            "short layout candidate must be stopped inside NANDA L3 before correction_core: {resolution:?}"
        );
    }

    #[test]
    fn russian_phrase_context_still_allows_short_preposition_repair() {
        let pipeline = default_typing_assist_pipeline();
        let resolution = resolve_text_correction(request(
            "читай cola d wechat ",
            &pipeline,
            CorrectionMode::DeterministicOnly,
        ));

        let selected = resolution
            .selected
            .clone()
            .unwrap_or_else(|| panic!("selected candidate: {resolution:?}"));
        assert_eq!(selected.replacement, "читай cola в wechat ");
        assert_eq!(selected.gate.action, CandidateGateAction::Apply);
    }

    #[test]
    fn layout_then_typo_repairs_dirty_wrong_layout_word() {
        let pipeline = default_typing_assist_pipeline();
        let resolution = resolve_text_correction(request(
            "hf,jfntn ",
            &pipeline,
            CorrectionMode::DeterministicOnly,
        ));

        let selected = resolution.selected.expect("selected candidate");
        assert_eq!(selected.replacement, "работает ");
        assert_eq!(selected.source_id, "layout_then_adjacent_transposition");
        assert_eq!(selected.error_class, TypingErrorClass::CompositeTypo);
        assert_eq!(selected.gate.action, CandidateGateAction::Apply);
    }

    #[test]
    fn composite_typo_repairs_known_russian_word() {
        let pipeline = default_typing_assist_pipeline();
        let resolution = resolve_text_correction(request(
            "помшник ",
            &pipeline,
            CorrectionMode::DeterministicOnly,
        ));

        let selected = resolution
            .selected
            .clone()
            .unwrap_or_else(|| panic!("selected candidate: {resolution:?}"));
        assert_eq!(selected.replacement, "помощник ");
        assert_eq!(selected.source_id, "composite_ru_typo");
        assert_eq!(selected.error_class, TypingErrorClass::CompositeTypo);
        assert_eq!(selected.gate.action, CandidateGateAction::Apply);
    }

    #[test]
    fn composite_typo_rejects_short_initial_consonant_growth() {
        let pipeline = default_typing_assist_pipeline();
        let resolution = resolve_text_correction(request(
            "давай лушее ",
            &pipeline,
            CorrectionMode::DeterministicOnly,
        ));

        let selected = resolution.selected.expect("safe candidate should remain");
        assert_eq!(selected.replacement, "давай лучшее ");
        assert!(!resolution
            .candidates
            .iter()
            .any(|candidate| candidate.replacement == "давай глушее "
                && candidate.gate.action == CandidateGateAction::Apply));
    }

    #[test]
    fn composite_typo_rejects_short_initial_vowel_growth_from_logs() {
        let pipeline = default_typing_assist_pipeline();
        for (input, forbidden) in [("рина ", "арина "), ("решение задачь ", "решение озадачь ")]
        {
            let resolution = resolve_text_correction(request(
                input,
                &pipeline,
                CorrectionMode::DeterministicOnly,
            ));

            assert!(
                resolution
                    .decision
                    .as_ref()
                    .map(|decision| &decision.replacement)
                    != Some(&forbidden.to_string()),
                "forbidden candidate auto-applied: {resolution:?}"
            );
        }
    }

    #[test]
    fn known_russian_words_do_not_autorewrite_to_other_known_words() {
        let pipeline = default_typing_assist_pipeline();
        for (input, forbidden) in [
            ("искать хрень! ", "искать хрену "),
            ("будет плох ", "будет плоха "),
            ("Блин ", "Блина "),
            ("не мение ", "не мерние "),
            ("не мение ", "не менте "),
            ("теорию бейса ", "теорию бейсяа "),
        ] {
            let resolution = resolve_text_correction(request(
                input,
                &pipeline,
                CorrectionMode::DeterministicOnly,
            ));

            assert!(
                resolution
                    .decision
                    .as_ref()
                    .map(|decision| &decision.replacement)
                    != Some(&forbidden.to_string()),
                "forbidden known-word rewrite auto-applied: {resolution:?}"
            );
        }
    }

    #[test]
    fn composite_typo_recovers_common_word_with_broken_prefix() {
        let pipeline = default_typing_assist_pipeline();
        let resolution = resolve_text_correction(request(
            "где эсперемнт ",
            &pipeline,
            CorrectionMode::DeterministicOnly,
        ));

        let selected = resolution.selected.expect("selected candidate");
        assert_eq!(selected.replacement, "где эксперимент ");
        assert_eq!(selected.source_id, "composite_ru_typo");
        assert_eq!(selected.error_class, TypingErrorClass::CompositeTypo);
        assert_eq!(selected.gate.action, CandidateGateAction::Apply);
    }

    #[test]
    fn composite_typo_prefers_effective_over_affective_for_missing_initial_vowel() {
        let pipeline = default_typing_assist_pipeline();
        let resolution = resolve_text_correction(request(
            "на сколько ффективная ",
            &pipeline,
            CorrectionMode::DeterministicOnly,
        ));

        let selected = resolution.selected.expect("selected candidate");
        assert_eq!(selected.replacement, "на сколько эффективная ");
        assert_eq!(selected.source_id, "composite_ru_typo");
        assert_eq!(selected.error_class, TypingErrorClass::CompositeTypo);
        assert_eq!(selected.gate.action, CandidateGateAction::Apply);
    }

    #[test]
    fn boundary_gate_does_not_split_known_single_word() {
        let gate = gate_candidate("уровне ", "у ровне ", TypingErrorClass::GluedWords);

        assert_eq!(gate.action, CandidateGateAction::SuggestOnly);
        assert_eq!(gate.reason, "known_single_word_boundary_split");
    }

    #[test]
    fn boundary_gate_does_not_split_known_word_inside_phrase() {
        let gate = gate_candidate("на уровне ", "на у ровне ", TypingErrorClass::GluedWords);

        assert_eq!(gate.action, CandidateGateAction::SuggestOnly);
        assert_eq!(gate.reason, "known_single_word_boundary_split");
    }

    #[test]
    fn boundary_gate_rejects_short_function_split_with_unknown_tail() {
        let gate = gate_candidate("со скрина ", "со с крина ", TypingErrorClass::GluedWords);

        assert_eq!(gate.action, CandidateGateAction::SuggestOnly);
        assert_eq!(gate.reason, "weak_boundary_split_tail");
    }

    #[test]
    fn composite_typo_repairs_generated_russian_forms() {
        let pipeline = default_typing_assist_pipeline();
        for (input, expected) in [("руских ", "русских "), ("звгрузи ", "загрузи ")]
        {
            let resolution = resolve_text_correction(request(
                input,
                &pipeline,
                CorrectionMode::DeterministicOnly,
            ));

            let selected = resolution.selected.expect("selected candidate");
            assert_eq!(selected.replacement, expected, "input={input:?}");
            assert_eq!(selected.error_class, TypingErrorClass::CompositeTypo);
            assert_eq!(selected.gate.action, CandidateGateAction::Apply);
        }
    }

    #[test]
    fn known_phrase_parts_do_not_autogrow_by_one_letter() {
        let pipeline = default_typing_assist_pipeline();
        for (input, forbidden) in [
            ("у меня ", "у меняю "),
            ("твой ", "тывой "),
            ("к тебе ", "к требе "),
            ("Тебе ", "Требе "),
            ("в план! ", "в плана! "),
            ("но пока ", "но прока "),
        ] {
            let resolution = resolve_text_correction(request(
                input,
                &pipeline,
                CorrectionMode::DeterministicOnly,
            ));

            assert_eq!(resolution.decision, None, "input={input:?}");
            assert!(
                resolution.candidates.iter().all(|candidate| {
                    candidate.replacement != forbidden
                        || candidate.gate.action != CandidateGateAction::Apply
                }),
                "forbidden candidate auto-applied: {resolution:?}"
            );
        }
    }

    #[test]
    fn nanda_candidate_cannot_autogrow_known_phrase_part_either() {
        let gate = gate_candidate("твой ", "тывой ", TypingErrorClass::CompositeTypo);

        assert_eq!(gate.action, CandidateGateAction::SuggestOnly);
        assert_eq!(gate.reason, "known_phrase_part_one_letter_growth");
    }

    #[test]
    fn nanda_semantic_candidate_cannot_rewrite_known_word_to_neighbor_word() {
        let gate = gate_candidate(
            "искать хрень! ",
            "искать хрену ",
            TypingErrorClass::CompositeTypo,
        );

        assert_eq!(gate.action, CandidateGateAction::SuggestOnly);
        assert_eq!(gate.reason, "soft_sign_vowel_drift");
    }

    #[test]
    fn semantic_word_cell_far_surface_jumps_are_suggest_only() {
        for (input, replacement) in [
            ("реально помагаешь ", "реально понимаешь "),
            ("она спраивтя ", "она спрашивая "),
        ] {
            assert!(
                semantic_wave_candidate_lacks_surface_authority(
                    input,
                    replacement,
                    crate::nanda_wave::context_wave::SEMANTIC_WORD_SOURCE,
                ),
                "semantic helper must reject {input:?} -> {replacement:?}"
            );
            let gate = gate_candidate_with_source(
                input,
                replacement,
                TypingErrorClass::CompositeTypo,
                crate::nanda_wave::context_wave::SEMANTIC_WORD_SOURCE,
            );

            assert_eq!(gate.action, CandidateGateAction::SuggestOnly);
            assert_eq!(gate.reason, "semantic_wave_surface_authority_low");
        }
    }

    #[test]
    fn nanda_l3_support_cannot_override_live_protected_terms() {
        let pipeline = default_typing_assist_pipeline();
        for input in [
            "это патерн ",
            "в гугле ",
            "блять ",
            "слово грокать ",
            "тоже грокнулся. ",
        ] {
            let resolution = resolve_text_correction(request(
                input,
                &pipeline,
                CorrectionMode::DeterministicThenNanda,
            ));

            assert_eq!(resolution.decision, None, "input={input:?}: {resolution:?}");
            assert!(
                resolution.candidates.iter().all(|candidate| {
                    candidate.source != CorrectionDecisionSource::Nanda
                        || candidate.gate.action != CandidateGateAction::Apply
                }),
                "NANDA candidate bypassed hard safety: {resolution:?}"
            );
        }
    }

    #[test]
    fn composite_typo_splits_previous_glued_word_when_fixing_current_typo() {
        let pipeline = default_typing_assist_pipeline();
        let resolution = resolve_text_correction(request(
            "ее простозальет свтеом ",
            &pipeline,
            CorrectionMode::DeterministicOnly,
        ));

        let selected = resolution.selected.expect("selected candidate");
        assert_eq!(selected.replacement, "ее просто зальет светом ");
        assert_eq!(selected.source_id, ids::ADJACENT_TRANSPOSITION);
        assert_eq!(
            selected.error_class,
            TypingErrorClass::AdjacentTransposition
        );
        assert_eq!(selected.gate.action, CandidateGateAction::Apply);
    }

    #[test]
    fn composite_typo_does_not_glue_two_committed_words() {
        let pipeline = default_typing_assist_pipeline();
        let resolution = resolve_text_correction(request(
            "реально ое ",
            &pipeline,
            CorrectionMode::DeterministicOnly,
        ));

        assert_eq!(resolution.decision, None);
        assert!(!resolution
            .candidates
            .iter()
            .any(|candidate| candidate.replacement == "реальное "));
    }

    #[test]
    fn repeated_letter_repairs_short_all_caps_word() {
        let pipeline = default_typing_assist_pipeline();
        let resolution = resolve_text_correction(request(
            "ТРУССС ",
            &pipeline,
            CorrectionMode::DeterministicOnly,
        ));

        let selected = resolution
            .selected
            .clone()
            .unwrap_or_else(|| panic!("selected candidate, resolution={resolution:?}"));
        assert_eq!(selected.replacement, "ТРУС ");
        assert_eq!(selected.source_id, ids::REPEATED_LETTER);
        assert_eq!(selected.error_class, TypingErrorClass::RepeatedLetter);
        assert_eq!(selected.gate.action, CandidateGateAction::Apply);
    }

    #[test]
    fn repeated_prefix_plus_letter_substitution_does_not_apply_intermediate_word() {
        let pipeline = default_typing_assist_pipeline();
        let resolution = resolve_text_correction(request(
            "ППОНИКАЕШЬ? ",
            &pipeline,
            CorrectionMode::DeterministicOnly,
        ));

        assert_eq!(resolution.decision, None);
        assert!(resolution.candidates.iter().any(|candidate| {
            candidate.replacement == "ПОНИКАЕШЬ? "
                && candidate.gate.action == CandidateGateAction::SuggestOnly
                && candidate.gate.reason == "single_step_typo_has_competing_composite"
        }));
    }

    #[test]
    fn composite_typo_repairs_short_adjacent_transposition_in_phrase() {
        let pipeline = default_typing_assist_pipeline();
        let resolution = resolve_text_correction(request(
            "имеет смылс ",
            &pipeline,
            CorrectionMode::DeterministicOnly,
        ));

        let selected = resolution.selected.expect("selected candidate");
        assert_eq!(selected.replacement, "имеет смысл ");
        assert_eq!(selected.source_id, ids::ADJACENT_TRANSPOSITION);
        assert_eq!(
            selected.error_class,
            TypingErrorClass::AdjacentTransposition
        );
        assert_eq!(selected.gate.action, CandidateGateAction::Apply);
    }

    #[test]
    fn adjacent_transposition_keeps_already_known_word() {
        let pipeline = default_typing_assist_pipeline();
        let resolution = resolve_text_correction(request(
            "Ладно ",
            &pipeline,
            CorrectionMode::DeterministicOnly,
        ));

        assert!(resolution.selected.is_none());
        assert!(resolution.decision.is_none());
    }

    #[test]
    fn future_auxiliary_blocks_non_infinitive_typo_candidate() {
        let pipeline = default_typing_assist_pipeline();
        let resolution = resolve_text_correction(request(
            "будет несити ",
            &pipeline,
            CorrectionMode::DeterministicOnly,
        ));

        assert!(resolution.selected.is_none());
        assert!(resolution.decision.is_none());
    }

    #[test]
    fn nanda_mode_corrects_wave_writer_text() {
        let pipeline = default_typing_assist_pipeline();
        let decision =
            decide_text_correction(request("тфтвф ", &pipeline, CorrectionMode::NandaOnly))
                .expect("nanda should produce a layout candidate");
        assert_eq!(decision.replacement, "nanda ");
        assert_eq!(decision.source, CorrectionDecisionSource::Nanda);
    }

    #[test]
    fn nanda_candidate_also_passes_unified_gate() {
        let pipeline = default_typing_assist_pipeline();
        let resolution =
            resolve_text_correction(request("тфтвф ", &pipeline, CorrectionMode::NandaOnly));

        let selected = resolution.selected.expect("selected candidate");
        assert_eq!(selected.replacement, "nanda ");
        assert_eq!(selected.error_class, TypingErrorClass::WrongLayout);
        assert_eq!(selected.gate.action, CandidateGateAction::Apply);
    }

    #[test]
    fn nanda_surface_motif_can_apply_known_typo() {
        let pipeline = default_typing_assist_pipeline();
        let resolution =
            resolve_text_correction(request("звгрузи ", &pipeline, CorrectionMode::NandaOnly));

        let selected = resolution.selected.expect("selected candidate");
        assert_eq!(selected.replacement, "загрузи ");
        assert_eq!(selected.source, CorrectionDecisionSource::Nanda);
        assert_eq!(selected.source_id, "L2SurfaceMotifCell32");
        assert_eq!(selected.error_class, TypingErrorClass::CompositeTypo);
        assert_eq!(selected.gate.action, CandidateGateAction::Apply);
    }

    #[test]
    fn nanda_surface_completion_is_suggest_only_not_autocorrect() {
        let pipeline = default_typing_assist_pipeline();
        let resolution =
            resolve_text_correction(request("делай пров ", &pipeline, CorrectionMode::NandaOnly));

        assert!(resolution.decision.is_none());
        let completion = resolution
            .candidates
            .iter()
            .find(|candidate| candidate.source_id == "L2SurfaceCompletionCell32")
            .expect("completion candidate");
        assert_eq!(completion.error_class, TypingErrorClass::CompletionOnly);
        assert_eq!(completion.gate.action, CandidateGateAction::SuggestOnly);
    }

    #[test]
    fn nanda_corrects_customs_actor_phrase_with_right_anchor() {
        let pipeline = default_typing_assist_pipeline();
        let resolution = resolve_text_correction(request(
            "Поставщик говорит что цена до склада нашего покупателя но таможен мы! ",
            &pipeline,
            CorrectionMode::NandaOnly,
        ));

        let selected = resolution.selected.expect("selected candidate");
        assert_eq!(
            selected.replacement,
            "Поставщик говорит что цена до склада нашего покупателя но таможим мы! "
        );
        assert_eq!(selected.source, CorrectionDecisionSource::Nanda);
        assert_eq!(selected.source_id, "PhraseCell32");
        assert_eq!(selected.error_class, TypingErrorClass::CompositeTypo);
        assert_eq!(selected.gate.action, CandidateGateAction::Apply);
    }

    #[test]
    fn nanda_does_not_correct_customs_actor_phrase_without_right_anchor() {
        let pipeline = default_typing_assist_pipeline();
        let resolution = resolve_text_correction(request(
            "Поставщик говорит что цена до склада нашего покупателя но таможен ",
            &pipeline,
            CorrectionMode::NandaOnly,
        ));

        assert!(resolution.selected.is_none());
        assert!(resolution.decision.is_none());
    }

    #[test]
    fn disabled_runtime_flags_keep_original() {
        let pipeline = default_typing_assist_pipeline();
        let decision = decide_text_correction(CorrectionRequest {
            text: "lfdfq ",
            auto_replace: false,
            typing_assist: false,
            auto_switch_layout: false,
            correction_safety: CorrectionSafety::Experimental,
            typing_assist_pipeline: &pipeline,
            nanda_autocorrect: false,
            mode: CorrectionMode::DeterministicThenNanda,
        });
        assert_eq!(decision, None);
    }
}

//! Shared text-correction decision facade.
//!
//! Runtime backends still own output and state. This module only answers one
//! question: should this completed text be replaced, and by which engine?

use crate::config::{CorrectionSafety, TypingAssistRuleConfig};
use crate::nanda_wave::{run_wave_trace, WaveDecision};
use crate::text_case::apply_word_case;
use crate::text_metrics::{damerau_levenshtein, has_cyrillic, has_latin};
use crate::typing_assist::{explain_typing_assist_with_pipeline, split_ws_segments};
use crate::typing_context::typing_assist_pipeline_for_context;
use crate::typing_rule_graph::ids;
use crate::word_reader::{is_cyrillic_letters_only, split_edge_whitespace, split_word_punctuation};
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
            if board.selected_apply_candidate().is_none() {
                board.push(nanda_text_correction(&req));
            }
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
            .find(|candidate| candidate.gate.action == CandidateGateAction::Apply)
            .cloned()
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
    let gate = gate_candidate(req.text, &replacement, error_class);

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
    layout_then_typo_candidate(req, pipeline).or_else(|| composite_russian_typo_candidate(req))
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
    let gate = gate_candidate(
        req.text,
        &final_replacement,
        TypingErrorClass::CompositeTypo,
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
    if !is_cyrillic_letters_only(&current_word)
        || crate::russian_lexicon::is_known_russian_word_or_form(&current_word)
    {
        return None;
    }

    if let Some(replacement_word) = unique_adjacent_transposition_word(&current_word) {
        let replacement = replace_last_text_word(req.text, &replacement_word)?;
        if replacement != req.text {
            let gate = gate_candidate(
                req.text,
                &replacement,
                TypingErrorClass::AdjacentTransposition,
            );
            return Some(UnifiedCorrectionCandidate {
                replacement,
                source: CorrectionDecisionSource::Deterministic,
                source_id: ids::ADJACENT_TRANSPOSITION.to_string(),
                error_class: TypingErrorClass::AdjacentTransposition,
                gate,
            });
        }
    }

    let lower = current_word.to_lowercase();
    let (candidate, _) = crate::candidate_ranker::choose_best_with_gap(
        crate::ru_typo::fuzzy_known_word_candidates(&lower),
        0.85,
        |candidate| {
            if candidate == &lower
                || !crate::russian_lexicon::is_known_russian_word_or_form(candidate)
            {
                return None;
            }
            let distance = damerau_levenshtein(&lower, candidate);
            if distance == 0 || distance > 3 {
                return None;
            }
            if !compatible_composite_typo_shape(&lower, candidate, distance) {
                return None;
            }
            let margin = crate::ngram::ru_candidate_margin(candidate, &lower);
            let inserted = candidate
                .chars()
                .count()
                .saturating_sub(lower.chars().count());
            if inserted > 1 {
                return None;
            }
            let shape_bonus = inserted as f64 * 8.0;
            let score = margin + shape_bonus - distance as f64 * 0.35;
            (score >= 0.0).then_some(score)
        },
    )?;
    let replacement_word = apply_word_case(&current_word, &candidate);
    let replacement = replace_last_text_word(req.text, &replacement_word)?;
    if replacement == req.text {
        return None;
    }

    let gate = gate_candidate(req.text, &replacement, TypingErrorClass::CompositeTypo);
    Some(UnifiedCorrectionCandidate {
        replacement,
        source: CorrectionDecisionSource::Deterministic,
        source_id: "composite_ru_typo".to_string(),
        error_class: TypingErrorClass::CompositeTypo,
        gate,
    })
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
            let gate = gate_candidate(req.text, text, error_class);
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

    distance == 1 && original.chars().count() == candidate.chars().count()
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
        "PhraseForecastCell32" | "L2WordAttractorCell32" => TypingErrorClass::CompletionOnly,
        "CommonRuFixCell32"
        | "LearnedMemoryCell32"
        | "PhraseMemoryCell32"
        | "PhraseCell32"
        | "SemanticWordCell32" => TypingErrorClass::CompositeTypo,
        _ => TypingErrorClass::Unknown,
    }
}

fn gate_candidate(
    original: &str,
    replacement: &str,
    error_class: TypingErrorClass,
) -> CandidateGateDecision {
    if original == replacement {
        return CandidateGateDecision {
            action: CandidateGateAction::KeepOriginal,
            reason: "unchanged",
        };
    }

    match error_class {
        TypingErrorClass::TechnicalToken | TypingErrorClass::ProtectedToken => {
            CandidateGateDecision {
                action: CandidateGateAction::Veto,
                reason: "protected_or_technical",
            }
        }
        TypingErrorClass::CompletionOnly => CandidateGateDecision {
            action: CandidateGateAction::SuggestOnly,
            reason: "completion_is_not_autocorrect",
        },
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
    fn resolution_routes_missing_letter_through_unified_gate() {
        let pipeline = default_typing_assist_pipeline();
        let resolution = resolve_text_correction(request(
            "автозаена ",
            &pipeline,
            CorrectionMode::DeterministicOnly,
        ));

        let selected = resolution.selected.expect("selected candidate");
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

        let selected = resolution.selected.expect("selected candidate");
        assert_eq!(selected.replacement, "помощник ");
        assert_eq!(selected.source_id, "composite_ru_typo");
        assert_eq!(selected.error_class, TypingErrorClass::CompositeTypo);
        assert_eq!(selected.gate.action, CandidateGateAction::Apply);
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

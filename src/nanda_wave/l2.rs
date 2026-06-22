use crate::config::CorrectionSafety;
use crate::dict::{convert, detect_direction};
use crate::lexicon::{
    is_common_en_guard_prefix, is_common_en_technical_word, is_common_ru_word,
    is_ru_short_function_word, visual_b_after_ascii_replacement, visual_b_default_replacement,
};
use crate::typing_candidate::TypingCandidateFamily;
use crate::typing_context;
use crate::typing_pipeline::explain_typing_assist_with_pipeline;

use super::context::{TailContext, TokenKind};
use super::feedback::{apply_l3_feedback, L3Feedback};
use super::lexical_attractor::{lexical_attractor_candidates, LEXICAL_ATTRACTOR_CELL};
use super::options::WaveOptions;
use super::pattern_memory::{apply_pattern_memory, PATTERN_MEMORY_CELL};
use super::signal::{WavePacket, WordCandidate};

const MAX_LAYOUT_SCAN_CANDIDATES: usize = 4;
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

pub fn run_l2_refined_with_feedback(
    original: &str,
    l1: &[WavePacket],
    options: &WaveOptions,
    feedback: &L3Feedback,
) -> Vec<WordCandidate> {
    let tail = original.trim_end();
    let Some((prefix, token)) = split_last_token(tail) else {
        return Vec::new();
    };
    let context = TailContext::from_text(tail);
    let mut candidates = Vec::new();
    if options.is_enabled("LayoutWordCell32") {
        if let Some(candidate) = layout_candidate(prefix, token, &context, l1) {
            push_unique_candidate(&mut candidates, candidate);
        }
        for candidate in layout_scan_candidates(tail, &context, l1) {
            push_unique_candidate(&mut candidates, candidate);
        }
    }
    if options.is_enabled("ShortTokenCell32") {
        for candidate in short_token_candidates(prefix, token, &context, l1) {
            push_unique_candidate(&mut candidates, candidate);
        }
    }
    if options.is_enabled("TechTokenCell32") {
        if let Some(candidate) = technical_keep_candidate(token, l1) {
            push_unique_candidate(&mut candidates, candidate);
        }
        if let Some(candidate) = technical_context_keep_candidate(tail, l1) {
            push_unique_candidate(&mut candidates, candidate);
        }
    }
    if options.is_enabled("BoundaryCell32") {
        for candidate in boundary_split_candidates(prefix, token, l1, &context) {
            push_unique_candidate(&mut candidates, candidate);
        }
    }
    if options.is_enabled(LEXICAL_ATTRACTOR_CELL) {
        for candidate in lexical_attractor_candidates(tail, &context) {
            push_unique_candidate(&mut candidates, candidate);
        }
    }
    if memory_cells_enabled(options) {
        for candidate in super::learned::learned_candidates(tail)
            .into_iter()
            .filter(|candidate| input_source_enabled(candidate.source, options))
        {
            push_unique_candidate(&mut candidates, candidate);
        }
    }
    if options.is_enabled(super::context_wave::SEMANTIC_WORD_SOURCE) {
        for candidate in super::context_wave::semantic_word_candidates(tail) {
            push_unique_candidate(&mut candidates, candidate);
        }
    }
    if options.is_enabled(super::context_wave::PHRASE_FORECAST_CELL) && options.llmwave_shadow() {
        let memory = phrase_forecast_memory();
        for candidate in super::llmwave::phrase_forecast_candidates(tail, &memory) {
            push_unique_candidate(&mut candidates, candidate);
        }
    }
    if options.is_enabled("GrammarCell32") {
        for candidate in grammar_agreement_candidates(tail, &context, l1) {
            push_unique_candidate(&mut candidates, candidate);
        }
    }
    for candidate in taught_candidates(tail, &context, l1, options) {
        push_unique_candidate(&mut candidates, candidate);
    }
    if options.is_enabled(PATTERN_MEMORY_CELL) {
        let report = apply_pattern_memory(tail, &mut candidates);
        if report.applied > 0 {
            candidates.iter_mut().for_each(|candidate| {
                candidate.support.push(format!(
                    "pattern-memory-report:records={} applied={}",
                    report.records, report.applied
                ));
            });
        }
    }
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

fn memory_cells_enabled(options: &WaveOptions) -> bool {
    options.is_enabled("LearnedMemoryCell32")
        || options.is_enabled("CommonRuFixCell32")
        || options.is_enabled("PhraseMemoryCell32")
        || options.is_enabled("UserMemoryCell32")
}

fn input_source_enabled(source: &str, options: &WaveOptions) -> bool {
    match source {
        "LearnedMemoryCell32" => options.is_enabled("LearnedMemoryCell32"),
        "CommonRuFixCell32" => options.is_enabled("CommonRuFixCell32"),
        "PhraseMemoryCell32" => options.is_enabled("PhraseMemoryCell32"),
        "UserMemoryCell32" => options.is_enabled("UserMemoryCell32"),
        source if source == PATTERN_MEMORY_CELL => false,
        _ => true,
    }
}

fn taught_word_candidate(input: TaughtCandidateInput<'_>) -> Option<WordCandidate> {
    let replacement = input.replacement.trim_end();
    if replacement == input.original.trim_end() {
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
    if context.token_count() < 2 && token.chars().count() > 3 {
        return None;
    }
    if is_common_en_technical_word(&token.to_ascii_lowercase()) {
        return None;
    }
    if technical_context_blocks_layout(prefix, token) {
        return None;
    }
    let converted = convert(token, detect_direction(token));
    if converted == token {
        return None;
    }
    if !layout_candidate_allowed(token, &converted) {
        return None;
    }
    if !language_allows_layout(token, &converted) {
        return None;
    }
    let energy = l1_energy(l1, "KeyboardCell32").max(0.35);
    let risk = layout_risk(token, &converted, context);
    if energy <= risk {
        return None;
    }
    Some(WordCandidate {
        text: format!("{prefix}{converted}"),
        source: "LayoutWordCell32",
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
        let converted = convert(token, detect_direction(token));
        if converted == token
            || !layout_candidate_allowed(token, &converted)
            || !language_allows_layout(token, &converted)
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
        let risk = (layout_risk(token, &replaced[idx], context) + 0.08).min(0.90);
        if energy <= risk {
            continue;
        }
        candidates.push(WordCandidate {
            text,
            source: "LayoutWordCell32",
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

fn language_allows_layout(token: &str, converted: &str) -> bool {
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
    let Some(replacement) = agree_adjective_like_tail(&previous, &last_clean) else {
        return Vec::new();
    };
    if replacement == last_clean {
        return Vec::new();
    }
    let Some((prefix, _token)) = split_last_token(tail.trim_end()) else {
        return Vec::new();
    };
    vec![WordCandidate {
        text: format!("{prefix}{replacement}"),
        source: "GrammarCell32",
        energy: l1_energy(l1, "ScriptCell32")
            .max(l1_energy(l1, "BoundaryCell32"))
            .max(0.72),
        risk: 0.16,
        support: vec![
            "grammar-agreement".to_string(),
            format!("previous={previous:?} last={last_clean:?} replacement={replacement:?}"),
        ],
    }]
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

fn layout_candidate_allowed(token: &str, converted: &str) -> bool {
    let token_ascii = token.chars().all(|ch| ch.is_ascii_alphabetic());
    let token_cyrillic = token.chars().all(is_cyrillic_letter);
    let converted_ascii = converted.chars().all(|ch| ch.is_ascii_alphabetic());
    let converted_cyrillic = converted.chars().all(is_cyrillic_letter);

    if token_ascii && converted_cyrillic {
        return true;
    }
    if token_cyrillic && converted_ascii {
        return is_common_en_technical_word(&converted.to_ascii_lowercase());
    }
    false
}

fn is_cyrillic_letter(ch: char) -> bool {
    ('а'..='я').contains(&ch) || ('А'..='Я').contains(&ch) || ch == 'ё' || ch == 'Ё'
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
    if normalized.chars().count() < 6 || is_common_ru_word(&normalized) {
        return Vec::new();
    }
    let chars = normalized.chars().collect::<Vec<_>>();
    let mut candidates = Vec::new();
    for split in 1..chars.len() {
        let left = chars[..split].iter().collect::<String>();
        let right = chars[split..].iter().collect::<String>();
        if left.chars().count() > 2 && right.chars().count() < 3 {
            continue;
        }
        if !is_common_ru_word(&left) || !is_common_ru_word(&right) {
            continue;
        }
        candidates.push(WordCandidate {
            text: format!("{prefix}{left} {right}"),
            source: "BoundaryCell32",
            energy: l1_energy(l1, "BoundaryCell32").max(0.78),
            risk: if left.chars().count() <= 2 {
                0.18
            } else {
                0.12
            },
            support: vec![
                "dictionary-split".to_string(),
                format!("left={left:?} right={right:?}"),
            ],
        });
        if candidates.len() >= 3 {
            break;
        }
    }
    candidates
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

fn split_last_token(text: &str) -> Option<(&str, &str)> {
    if text.is_empty() {
        return None;
    }
    let start = text
        .char_indices()
        .rev()
        .find(|(_idx, ch)| ch.is_whitespace())
        .map(|(idx, ch)| idx + ch.len_utf8())
        .unwrap_or(0);
    let (prefix, token) = text.split_at(start);
    (!token.is_empty()).then_some((prefix, token))
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
    previous.kind == TokenKind::CyrillicWord && last.kind == TokenKind::CyrillicWord
}

fn normalized_edit_ratio(original: &str, replacement: &str) -> f32 {
    let original_len = original.chars().count().max(1);
    let replacement_len = replacement.chars().count();
    original_len.abs_diff(replacement_len) as f32 / original_len as f32
}

fn compact_text(text: &str) -> String {
    text.chars().filter(|ch| !ch.is_whitespace()).collect()
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
    fn phrase_cell_gets_typo_candidate() {
        let original = "рабоатет ";
        let l1 = run_l1(original);
        let candidates = run_l2(original, &l1);
        assert!(candidates
            .iter()
            .any(|candidate| candidate.source == "PhraseCell32"));
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

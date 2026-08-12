//! Shared IME candidate readout helpers.
//!
//! IBus supplies typed candidate material here and receives the one ordered
//! readout selected by `TransitionDecisionCore`.

use crate::word_reader::split_last_alphabetic_token;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImeCandidateSource {
    L2Completion,
    L2Replacement,
    L3Context,
}

pub fn is_ascii_layout_letter_symbol(ch: char) -> bool {
    crate::layout_autoswitch::is_ascii_layout_letter_symbol(ch)
}

#[derive(Debug, Clone, PartialEq)]
pub struct ImeCandidateProposal {
    /// Text appended after the active token for a completion proposal.
    pub suffix: String,
    /// A full current-token replacement. This is display-only until Tab builds
    /// an AuthorizedEdit for the active composition.
    pub replacement: Option<String>,
    pub confidence: f32,
    pub source: ImeCandidateSource,
    /// A candidate already admitted by the shared L2/L3/L4 field carries its
    /// final lattice order into the IBus adapter. Display must not recreate a
    /// second ranking from the lossy UI confidence value.
    pub authority_order: Option<usize>,
}

impl ImeCandidateProposal {
    pub fn new(suffix: impl Into<String>, confidence: f32, source: ImeCandidateSource) -> Self {
        Self {
            suffix: suffix.into(),
            replacement: None,
            confidence: confidence.clamp(0.0, 1.0),
            source,
            authority_order: None,
        }
    }

    pub fn replacement(
        surface: impl Into<String>,
        confidence: f32,
        source: ImeCandidateSource,
    ) -> Self {
        Self {
            suffix: String::new(),
            replacement: Some(surface.into()),
            confidence: confidence.clamp(0.0, 1.0),
            source,
            authority_order: None,
        }
    }

    pub fn display_text(&self) -> &str {
        self.replacement.as_deref().unwrap_or(&self.suffix)
    }

    pub fn is_replacement(&self) -> bool {
        self.replacement.is_some()
    }

    pub fn with_authority_order(mut self, order: usize) -> Self {
        self.authority_order = Some(order);
        self
    }
}

pub struct ImeCandidateReadoutRequest<'a> {
    pub proposals: &'a [ImeCandidateProposal],
    pub limit: usize,
}

pub fn select_ime_candidate_suffixes(request: ImeCandidateReadoutRequest<'_>) -> Vec<String> {
    select_ime_candidate_proposals(request)
        .into_iter()
        .filter(|proposal| !proposal.is_replacement())
        .map(|proposal| proposal.suffix)
        .collect()
}

pub fn select_ime_candidate_proposals(
    request: ImeCandidateReadoutRequest<'_>,
) -> Vec<ImeCandidateProposal> {
    crate::typing_transition::decision::TransitionDecisionCore::select_ime_readout(
        request.proposals,
        request.limit,
    )
}

pub fn push_unique_suffix(candidates: &mut Vec<String>, suffix: Option<String>) {
    let Some(suffix) = suffix else {
        return;
    };
    if suffix.is_empty()
        || !is_allowed_visible_completion_suffix(&suffix)
        || candidates.iter().any(|candidate| candidate == &suffix)
    {
        return;
    }
    candidates.push(suffix);
}

pub fn push_unique_ascii_known_suffix(candidates: &mut Vec<String>, token: &str, suffix: String) {
    if suffix.is_empty() || candidates.iter().any(|candidate| candidate == &suffix) {
        return;
    }
    let completed = format!("{token}{suffix}").to_ascii_lowercase();
    let one_ascii_char =
        suffix.chars().count() == 1 && suffix.chars().all(|ch| ch.is_ascii_alphabetic());
    if one_ascii_char && !crate::lexicon::is_common_en_technical_word(&completed) {
        return;
    }
    if one_ascii_char || is_allowed_visible_completion_suffix(&suffix) {
        candidates.push(suffix);
    }
}

pub fn is_allowed_visible_completion_suffix(suffix: &str) -> bool {
    let trimmed = suffix.trim();
    let mut chars = trimmed.chars();
    let Some(ch) = chars.next() else {
        return false;
    };
    if chars.next().is_some() {
        return true;
    }
    matches!(ch, 'и' | 'я' | 'I' | 'a')
}

pub fn preedit_suffix_context_and_word(tail: &str, suffix: &str) -> Option<(Vec<String>, String)> {
    let tail = tail.trim_end();
    let suffix_starts_new_word = suffix.chars().next().is_some_and(char::is_whitespace);
    if suffix_starts_new_word || tail.is_empty() {
        let word = suffix.split_whitespace().next()?.to_lowercase();
        let context = crate::nanda_wave::llmwave::tokenize(tail);
        return Some((context, word));
    }
    let (prefix, partial) = split_last_alphabetic_token(tail)?;
    let suffix_word_part = suffix.split_whitespace().next().unwrap_or(suffix);
    let word = format!(
        "{}{}",
        partial.to_lowercase(),
        suffix_word_part.to_lowercase()
    );
    let context = crate::nanda_wave::llmwave::tokenize(prefix);
    Some((context, word))
}

pub fn is_command_like_long_tail(tail: &str) -> bool {
    let mut word_count = 0usize;
    let mut uppercase = 0usize;
    let mut lowercase = 0usize;
    for token in tail.split_whitespace() {
        let mut has_alpha = false;
        for ch in token.chars().filter(|ch| ch.is_alphabetic()) {
            has_alpha = true;
            if ch.is_uppercase() {
                uppercase += 1;
            }
            if ch.is_lowercase() {
                lowercase += 1;
            }
        }
        if has_alpha {
            word_count += 1;
        }
    }
    word_count >= 4 && uppercase >= 12 && uppercase >= lowercase.saturating_mul(2).max(1)
}

pub fn phrase_candidate_suffix(
    tail: &str,
    candidate: &str,
    max_suffix_chars: usize,
) -> Option<String> {
    let suffix = strip_prefix_case_insensitive(candidate, tail)?;
    let suffix = if tail.ends_with(char::is_whitespace) {
        suffix.trim_start_matches(char::is_whitespace)
    } else {
        suffix
    };
    let suffix = next_word_suffix(suffix)?;
    (!suffix.is_empty() && suffix.chars().count() <= max_suffix_chars).then_some(suffix)
}

fn strip_prefix_case_insensitive<'a>(text: &'a str, prefix: &str) -> Option<&'a str> {
    let mut text_chars = text.char_indices();
    for prefix_ch in prefix.chars() {
        let (_, text_ch) = text_chars.next()?;
        if !text_ch.to_lowercase().eq(prefix_ch.to_lowercase()) {
            return None;
        }
    }
    let suffix_start = text_chars
        .next()
        .map(|(index, _)| index)
        .unwrap_or(text.len());
    Some(&text[suffix_start..])
}

pub fn should_query_llmwave_phrase_suffix(tail: &str) -> bool {
    if tail.ends_with(char::is_whitespace) {
        return true;
    }
    let trimmed = tail.trim_end();
    let Some((left, token)) = trimmed.rsplit_once(char::is_whitespace) else {
        return false;
    };
    let token_chars = token.chars().count();
    (1..=6).contains(&token_chars)
        && !left.split_whitespace().collect::<Vec<_>>().is_empty()
        && token.chars().all(|ch| ch.is_alphabetic())
}

fn next_word_suffix(suffix: &str) -> Option<String> {
    let leading_space = suffix.chars().next().is_some_and(char::is_whitespace);
    let word = suffix.split_whitespace().next()?;
    if leading_space {
        Some(format!(" {word}"))
    } else {
        Some(word.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::{
        phrase_candidate_suffix, select_ime_candidate_suffixes, ImeCandidateProposal,
        ImeCandidateReadoutRequest, ImeCandidateSource,
    };

    #[test]
    fn shared_readout_merges_sources_before_ranking() {
        let proposals = vec![
            ImeCandidateProposal::new("вет", 0.8, ImeCandidateSource::L3Context),
            ImeCandidateProposal::new("вет", 0.7, ImeCandidateSource::L2Completion),
            ImeCandidateProposal::new("чер", 0.6, ImeCandidateSource::L2Completion),
        ];
        let ranked = select_ime_candidate_suffixes(ImeCandidateReadoutRequest {
            proposals: &proposals,
            limit: 8,
        });

        assert_eq!(
            ranked
                .iter()
                .filter(|suffix| suffix.as_str() == "вет")
                .count(),
            1
        );
        assert!(ranked.iter().any(|suffix| suffix == "чер"));
    }

    #[test]
    fn shared_lattice_order_survives_ime_projection() {
        let proposals = vec![
            ImeCandidateProposal::new("смысл", 0.99, ImeCandidateSource::L3Context),
            ImeCandidateProposal::new("ивет", 0.40, ImeCandidateSource::L2Completion)
                .with_authority_order(0),
            ImeCandidateProposal::new("оверь", 0.30, ImeCandidateSource::L2Completion)
                .with_authority_order(1),
        ];
        let ranked = select_ime_candidate_suffixes(ImeCandidateReadoutRequest {
            proposals: &proposals,
            limit: 8,
        });

        assert_eq!(ranked, vec!["ивет", "оверь", "смысл"]);
    }

    #[test]
    fn phrase_suffix_matches_normalized_memory_after_sentence_capitalization() {
        assert_eq!(
            phrase_candidate_suffix("На улице опять идёт д", "на улице опять идёт дождь", 16,)
                .as_deref(),
            Some("ождь")
        );
    }
}

//! Unified typing memory event contract.
//!
//! Runtime code should describe learning as typed memory events. Storage layers
//! can then route the same event into L1/L2 usage, L3 phrase context, L4 signed
//! state, and Bayes priors without each caller inventing its own schema.

const CONTEXT_WORDS: usize = 5;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TypingMemoryEventKind {
    Typed,
    AcceptedFix,
    AcceptedIme,
    RejectedIme,
    RejectedCandidate,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TypingMemoryFeedback {
    Observed,
    Accepted,
    Rejected,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TypingMemoryEvent {
    pub(crate) kind: TypingMemoryEventKind,
    pub(crate) feedback: TypingMemoryFeedback,
    pub(crate) word: String,
    pub(crate) context: Vec<String>,
    pub(crate) from: Option<String>,
    pub(crate) to: Option<String>,
    pub(crate) source: String,
    pub(crate) operation: String,
    /// Stable relation shape, intentionally independent from concrete words.
    pub(crate) surface: Option<String>,
}

impl TypingMemoryEvent {
    pub(crate) fn typed_tail(tail: &str) -> Option<Self> {
        let (context, word) = context_and_last_word(tail)?;
        Some(Self {
            kind: TypingMemoryEventKind::Typed,
            feedback: TypingMemoryFeedback::Observed,
            word,
            context,
            from: None,
            to: None,
            source: "user".to_string(),
            operation: "typed".to_string(),
            surface: None,
        })
    }

    pub(crate) fn accepted_fix(from: &str, to: &str) -> Vec<Self> {
        accepted_events(
            TypingMemoryEventKind::AcceptedFix,
            from,
            to,
            "autocorrect",
            "replacement",
        )
    }

    pub(crate) fn accepted_ime(context_tail: &str, accepted_text: &str) -> Vec<Self> {
        ime_events(
            TypingMemoryEventKind::AcceptedIme,
            TypingMemoryFeedback::Accepted,
            context_tail,
            accepted_text,
        )
    }

    pub(crate) fn rejected_ime(context_tail: &str, rejected_text: &str) -> Vec<Self> {
        ime_events(
            TypingMemoryEventKind::RejectedIme,
            TypingMemoryFeedback::Rejected,
            context_tail,
            rejected_text,
        )
    }

    pub(crate) fn rejected_candidate(
        context_tail: &str,
        rejected_text: &str,
        source: &str,
        operation: &str,
    ) -> Vec<Self> {
        let from_words = normalized_words(context_tail);
        let rejected_words = normalized_words(rejected_text);
        changed_target_indexes(&from_words, &rejected_words)
            .into_iter()
            .filter_map(|index| {
                let word = rejected_words.get(index)?.clone();
                let context = words_before_last(&rejected_words[..index]);
                Some(Self {
                    kind: TypingMemoryEventKind::RejectedCandidate,
                    feedback: TypingMemoryFeedback::Rejected,
                    word,
                    context,
                    from: Some(context_tail.trim().to_string()),
                    to: Some(rejected_text.trim().to_string()),
                    source: source.to_string(),
                    operation: operation.to_string(),
                    surface: Some(transition_surface_key(
                        context_tail,
                        rejected_text,
                        source,
                        operation,
                    )),
                })
            })
            .collect()
    }
}

fn accepted_events(
    kind: TypingMemoryEventKind,
    from: &str,
    to: &str,
    source: &str,
    operation: &str,
) -> Vec<TypingMemoryEvent> {
    let to_words = normalized_words(to);
    if to_words.is_empty() {
        return Vec::new();
    }
    let from_words = normalized_words(from);
    let target_indexes = changed_target_indexes(&from_words, &to_words);
    target_indexes
        .into_iter()
        .map(|index| {
            let word = to_words[index].clone();
            let context = words_before_last(&to_words[..index]);
            TypingMemoryEvent {
                kind,
                feedback: TypingMemoryFeedback::Accepted,
                word,
                context,
                from: Some(from.trim().to_string()),
                to: Some(to.trim().to_string()),
                source: source.to_string(),
                operation: operation.to_string(),
                surface: Some(transition_surface_key(from, to, source, operation)),
            }
        })
        .collect()
}

pub(crate) fn changed_target_indexes(from_words: &[String], to_words: &[String]) -> Vec<usize> {
    let indexes = to_words
        .iter()
        .enumerate()
        .filter_map(|(index, word)| (from_words.get(index) != Some(word)).then_some(index))
        .collect::<Vec<_>>();
    if indexes.is_empty() {
        to_words.len().checked_sub(1).into_iter().collect()
    } else {
        indexes
    }
}

/// Normalized target region beginning at the first changed token.
///
/// The unchanged phrase prefix belongs to L3 context, not to the L4 operator
/// identity. Boundary transitions keep every changed target token, so a split
/// cannot collapse back to a last-word lookup.
pub(crate) fn transition_target_text(from: &str, to: &str) -> String {
    let from_words = normalized_words(from);
    let to_words = normalized_words(to);
    let start = changed_target_indexes(&from_words, &to_words)
        .into_iter()
        .next()
        .unwrap_or_else(|| to_words.len().saturating_sub(1));
    to_words.get(start..).unwrap_or_default().join(" ")
}

pub(crate) fn transition_context_words(from: &str, to: &str) -> Vec<String> {
    let from_words = normalized_words(from);
    let to_words = normalized_words(to);
    let start = changed_target_indexes(&from_words, &to_words)
        .into_iter()
        .next()
        .unwrap_or_else(|| from_words.len().saturating_sub(1));
    words_before_last(&from_words[..start.min(from_words.len())])
}

fn ime_events(
    kind: TypingMemoryEventKind,
    feedback: TypingMemoryFeedback,
    context_tail: &str,
    text: &str,
) -> Vec<TypingMemoryEvent> {
    let context = recent_context_words(context_tail);
    normalized_words(text)
        .into_iter()
        .map(|word| TypingMemoryEvent {
            kind,
            feedback,
            word,
            context: context.clone(),
            from: None,
            to: Some(text.trim().to_string()),
            source: "ime".to_string(),
            operation: "completion".to_string(),
            surface: None,
        })
        .collect()
}

pub(crate) fn transition_surface_key(
    from: &str,
    to: &str,
    _source: &str,
    operation: &str,
) -> String {
    let (_, relation) =
        crate::transition_relation::TransitionRelationAtoms::inferred(from, to, operation);
    relation.surface_key().to_string()
}

pub(crate) fn context_and_last_word(text: &str) -> Option<(Vec<String>, String)> {
    let words = normalized_words(text);
    let (word, context) = words.split_last()?;
    Some((words_before_last(context), word.clone()))
}

pub(crate) fn recent_context_words(text: &str) -> Vec<String> {
    normalized_words(text)
        .into_iter()
        .rev()
        .take(CONTEXT_WORDS)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect()
}

pub(crate) fn normalized_words(text: &str) -> Vec<String> {
    text.split_whitespace()
        .filter_map(|token| {
            let word = normalize_memory_word(token);
            (!word.is_empty()).then_some(word)
        })
        .collect()
}

pub(crate) fn normalize_memory_word(word: &str) -> String {
    let trimmed = word
        .trim()
        .trim_matches(|ch: char| !ch.is_alphabetic() && ch != '-');
    if !trimmed.chars().any(|ch| ch.is_alphabetic()) {
        return String::new();
    }
    trimmed.to_lowercase()
}

fn words_before_last(words: &[String]) -> Vec<String> {
    words
        .iter()
        .rev()
        .take(CONTEXT_WORDS)
        .cloned()
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn typed_tail_event_keeps_recent_context_and_word() {
        let event = TypingMemoryEvent::typed_tail("на улице опять идёт дождь ").unwrap();

        assert_eq!(event.kind, TypingMemoryEventKind::Typed);
        assert_eq!(event.feedback, TypingMemoryFeedback::Observed);
        assert_eq!(event.word, "дождь");
        assert_eq!(event.context, ["на", "улице", "опять", "идёт"]);
    }

    #[test]
    fn accepted_fix_events_mark_positive_result_and_source() {
        let events = TypingMemoryEvent::accepted_fix("мы отвравим", "мы отравим");

        assert!(events.iter().any(|event| event.word == "отравим"));
        assert!(events
            .iter()
            .all(|event| event.feedback == TypingMemoryFeedback::Accepted));
        assert!(events.iter().all(|event| event.source == "autocorrect"));
        assert!(events.iter().all(|event| event.operation == "replacement"));
        assert!(events.iter().all(|event| event.surface.is_some()));
    }

    #[test]
    fn rejected_candidate_events_are_negative_transition_material() {
        let events = TypingMemoryEvent::rejected_candidate(
            "ну исходник",
            "ну даша",
            "L2LiveCandidateGate32",
            "completion",
        );

        assert_eq!(events.len(), 1);
        assert_eq!(events[0].kind, TypingMemoryEventKind::RejectedCandidate);
        assert_eq!(events[0].feedback, TypingMemoryFeedback::Rejected);
        assert_eq!(events[0].context, ["ну"]);
        assert_eq!(events[0].word, "даша");
        assert!(events[0].surface.is_some());
    }

    #[test]
    fn rejected_candidate_excludes_unchanged_left_context() {
        let events = TypingMemoryEvent::rejected_candidate(
            "как попусы",
            "как опусы",
            "typing-assist",
            "replacement",
        );

        assert_eq!(events.len(), 1);
        assert_eq!(events[0].word, "опусы");
        assert_eq!(events[0].context, ["как"]);
        assert_eq!(events[0].from.as_deref(), Some("как попусы"));
        assert_eq!(events[0].to.as_deref(), Some("как опусы"));
    }

    #[test]
    fn transition_target_keeps_all_changed_boundary_tokens() {
        assert_eq!(transition_target_text("мыслов", "мы слов"), "мы слов");
        assert_eq!(
            transition_target_text("мы отвравим", "мы отравим"),
            "отравим"
        );
        assert_eq!(
            transition_context_words("мы отвравим", "мы отравим"),
            ["мы"]
        );
    }

    #[test]
    fn transition_surface_key_generalizes_across_concrete_words() {
        let first = transition_surface_key("мы cat", "мы car", "autocorrect", "replacement");
        let second = transition_surface_key("они dog", "они dot", "autocorrect", "replacement");

        assert_eq!(first, second);
    }
}

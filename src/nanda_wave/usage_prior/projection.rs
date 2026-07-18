use crate::typing_memory::{changed_target_indexes, normalize_memory_word, normalized_words};

use super::{UsageEvent, UsageEventKind};

pub(super) const TRANSITION_ANY: &str = "*";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum UsageEventProjectionKind {
    Typed,
    AcceptedFix,
    AcceptedIme,
    ConfirmedImePrediction,
    Rejected,
}

pub(super) struct UsageEventProjection<'a> {
    pub(super) context: &'a [String],
    pub(super) surface: Option<&'a str>,
    pub(super) word: String,
    pub(super) state_word: String,
    pub(super) transition_target: String,
    pub(super) transition_context: Vec<String>,
    pub(super) source: &'a str,
    pub(super) operation: &'a str,
    pub(super) weight: u32,
    pub(super) transition_weight: u32,
    kind: UsageEventProjectionKind,
}

impl<'a> UsageEventProjection<'a> {
    pub(super) fn from_event(event: &'a UsageEvent) -> Option<Self> {
        // An automatic apply is an observation, not user acceptance. Older
        // runtimes wrote it as positive feedback; ignore that poisoned lane.
        if matches!(event.kind, UsageEventKind::AcceptedFix)
            && event.source.as_deref() == Some("autocorrect")
        {
            return None;
        }
        let word = event.word.as_deref().map(normalize_memory_word)?;
        if word.is_empty()
            || (matches!(event.kind, UsageEventKind::RejectedCandidate)
                && !event_word_is_changed_target(event, &word))
        {
            return None;
        }
        let kind = match event.kind {
            UsageEventKind::Typed => UsageEventProjectionKind::Typed,
            UsageEventKind::AcceptedFix => UsageEventProjectionKind::AcceptedFix,
            UsageEventKind::AcceptedIme => UsageEventProjectionKind::AcceptedIme,
            UsageEventKind::ConfirmedImePrediction => {
                UsageEventProjectionKind::ConfirmedImePrediction
            }
            UsageEventKind::RejectedIme | UsageEventKind::RejectedCandidate => {
                UsageEventProjectionKind::Rejected
            }
        };
        let weight = match kind {
            UsageEventProjectionKind::Typed => 1,
            UsageEventProjectionKind::AcceptedFix => 6,
            UsageEventProjectionKind::AcceptedIme => 5,
            UsageEventProjectionKind::ConfirmedImePrediction => 3,
            UsageEventProjectionKind::Rejected => rejected_usage_weight(event.kind),
        };
        Some(Self {
            context: &event.context,
            surface: event.surface.as_deref(),
            state_word: event_state_word(event),
            transition_target: event_transition_target(event, &word),
            transition_context: event_transition_context(event),
            source: event_source(event),
            operation: event_operation(event),
            transition_weight: event_transition_weight(event, weight),
            weight,
            word,
            kind,
        })
    }

    pub(super) fn is_rejected(&self) -> bool {
        self.kind == UsageEventProjectionKind::Rejected
    }

    pub(super) fn is_accepted(&self) -> bool {
        matches!(
            self.kind,
            UsageEventProjectionKind::AcceptedFix
                | UsageEventProjectionKind::AcceptedIme
                | UsageEventProjectionKind::ConfirmedImePrediction
        )
    }

    pub(super) fn records_rejected_fix_sources(&self) -> bool {
        self.kind == UsageEventProjectionKind::AcceptedFix
    }
}

fn event_word_is_changed_target(event: &UsageEvent, word: &str) -> bool {
    let (Some(from), Some(to)) = (event.from.as_deref(), event.to.as_deref()) else {
        return true;
    };
    let from_words = normalized_words(from);
    let to_words = normalized_words(to);
    changed_target_indexes(&from_words, &to_words)
        .into_iter()
        .any(|index| to_words.get(index).is_some_and(|target| target == word))
}

fn event_transition_target(event: &UsageEvent, fallback_word: &str) -> String {
    let target = match (event.from.as_deref(), event.to.as_deref()) {
        (Some(from), Some(to)) => crate::typing_memory::transition_target_text(from, to),
        (_, Some(to)) => to.to_string(),
        _ => fallback_word.to_string(),
    };
    crate::transition_relation::transition_target_id(&target)
}

fn event_transition_context(event: &UsageEvent) -> Vec<String> {
    match (event.from.as_deref(), event.to.as_deref()) {
        (Some(from), Some(to)) => crate::typing_memory::transition_context_words(from, to),
        _ => event.context.clone(),
    }
}

fn event_transition_weight(event: &UsageEvent, weight: u32) -> u32 {
    let event_count = match (event.from.as_deref(), event.to.as_deref()) {
        (Some(from), Some(to)) => {
            let from_words = normalized_words(from);
            let to_words = normalized_words(to);
            changed_target_indexes(&from_words, &to_words).len()
        }
        (_, Some(to)) => normalized_words(to).len(),
        _ => 1,
    }
    .max(1) as u32;
    weight.saturating_add(event_count - 1) / event_count
}

fn event_state_word(event: &UsageEvent) -> String {
    event
        .from
        .as_deref()
        .map(crate::transition_relation::transition_state_id)
        .unwrap_or_else(|| TRANSITION_ANY.to_string())
}

fn event_source(event: &UsageEvent) -> &str {
    if matches!(event.kind, UsageEventKind::AcceptedFix)
        && event.source.as_deref() == Some("user_correction")
    {
        return "autocorrect";
    }
    event.source.as_deref().unwrap_or(match event.kind {
        UsageEventKind::Typed => "user",
        UsageEventKind::AcceptedFix => "autocorrect",
        UsageEventKind::AcceptedIme
        | UsageEventKind::ConfirmedImePrediction
        | UsageEventKind::RejectedIme => "ime",
        UsageEventKind::RejectedCandidate => "candidate",
    })
}

fn event_operation(event: &UsageEvent) -> &str {
    event.operation.as_deref().unwrap_or(match event.kind {
        UsageEventKind::Typed => "typed",
        UsageEventKind::AcceptedFix => "replacement",
        UsageEventKind::AcceptedIme
        | UsageEventKind::ConfirmedImePrediction
        | UsageEventKind::RejectedIme => "completion",
        UsageEventKind::RejectedCandidate => "candidate",
    })
}

fn rejected_usage_weight(kind: UsageEventKind) -> u32 {
    match kind {
        UsageEventKind::RejectedCandidate => 8,
        UsageEventKind::RejectedIme => 8,
        _ => 0,
    }
}

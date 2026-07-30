use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, VecDeque};

pub(super) const STATE_FORMAT: &str = "lay-l3-online-v1";
pub(super) const MIN_SCENES: usize = 2;
const MAX_SCENES: usize = 8;
const MAX_RELATIONS: usize = 128;
const MAX_RECENT_IME_REJECTIONS: usize = 32;
const MAX_IME_PAIR_EVENT_GAP: u64 = 16;

#[derive(Clone, Debug, Deserialize)]
pub(super) struct UsageEvent {
    #[serde(default)]
    kind: String,
    #[serde(default)]
    outcome: String,
    #[serde(default)]
    source: String,
    #[serde(default)]
    word: Option<String>,
    #[serde(default)]
    context: Vec<String>,
    #[serde(default)]
    from: Option<String>,
    #[serde(default)]
    to: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub(super) struct PendingRelation {
    pub(super) rejected: String,
    pub(super) expected: String,
    pub(super) scenes: Vec<String>,
    pub(super) last_attempted_scenes: usize,
    #[serde(default)]
    pub(super) last_observed_ordinal: u64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub(super) struct PendingImeRejection {
    rejected: String,
    context: Vec<String>,
    event_ordinal: u64,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub(super) struct OnlineFeedbackStats {
    parsed_events: u64,
    direct_correction_observations: u64,
    causal_ime_choice_observations: u64,
    stored_ime_rejections: u64,
    expired_ime_rejections: u64,
    evicted_relations: u64,
    pub(super) replayed_events: u64,
    pub(super) journal_compactions: u64,
    pub(super) journal_reanchors_without_overlap: u64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub(super) struct OnlineState {
    pub(super) format: String,
    pub(super) source_offset: u64,
    #[serde(default)]
    pub(super) source_device: u64,
    #[serde(default)]
    pub(super) source_inode: u64,
    #[serde(default)]
    pub(super) source_tail_hashes: Vec<u64>,
    pub(super) generation: u64,
    #[serde(default)]
    pub(super) pending: BTreeMap<String, PendingRelation>,
    #[serde(default)]
    pub(super) event_ordinal: u64,
    #[serde(default)]
    pub(super) recent_ime_rejections: VecDeque<PendingImeRejection>,
    #[serde(default)]
    pub(super) admitted_deltas: u64,
    #[serde(default)]
    pub(super) replayed_source_bytes: u64,
    #[serde(default)]
    pub(super) feedback: OnlineFeedbackStats,
}

impl Default for OnlineState {
    fn default() -> Self {
        Self {
            format: STATE_FORMAT.to_string(),
            source_offset: 0,
            source_device: 0,
            source_inode: 0,
            source_tail_hashes: Vec::new(),
            generation: 0,
            pending: BTreeMap::new(),
            event_ordinal: 0,
            recent_ime_rejections: VecDeque::new(),
            admitted_deltas: 0,
            replayed_source_bytes: 0,
            feedback: OnlineFeedbackStats::default(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ObservationSource {
    DirectCorrection,
    CausalImeChoice,
}

#[derive(Clone, Debug)]
pub(super) struct RelationObservation {
    rejected: String,
    expected: String,
    scene: String,
    pub(super) source: ObservationSource,
}

pub(super) fn relation_observation(
    state: &mut OnlineState,
    event: &UsageEvent,
) -> Option<RelationObservation> {
    state.event_ordinal = state.event_ordinal.saturating_add(1);
    state.feedback.parsed_events = state.feedback.parsed_events.saturating_add(1);
    expire_ime_rejections(state);

    if is_ime_rejection(event) {
        remember_ime_rejection(state, event);
        return None;
    }
    if let Some(observation) = direct_correction_observation(event) {
        state.feedback.direct_correction_observations = state
            .feedback
            .direct_correction_observations
            .saturating_add(1);
        return Some(observation);
    }
    let observation = causal_ime_choice_observation(state, event)?;
    state.feedback.causal_ime_choice_observations = state
        .feedback
        .causal_ime_choice_observations
        .saturating_add(1);
    Some(observation)
}

fn direct_correction_observation(event: &UsageEvent) -> Option<RelationObservation> {
    if event.kind != "accepted_fix"
        || event.outcome != "confirmed_positive"
        || event.source != "user_correction"
        || event.context.len() < 2
    {
        return None;
    }
    let rejected = rejected_word(event)?;
    let expected = normalize_word(event.word.as_deref()?)?;
    if rejected == expected {
        return None;
    }
    let context = event
        .context
        .iter()
        .filter_map(|token| normalize_word(token))
        .collect::<Vec<_>>();
    if context.len() < 2 {
        return None;
    }
    let scene = context
        .iter()
        .cloned()
        .chain(std::iter::once(expected.clone()))
        .collect::<Vec<_>>()
        .join(" ");
    Some(RelationObservation {
        rejected,
        expected,
        scene,
        source: ObservationSource::DirectCorrection,
    })
}

fn is_ime_rejection(event: &UsageEvent) -> bool {
    event.kind == "rejected_ime"
        && event.outcome == "reverted"
        && event.source == "ime"
        && event.word.is_some()
}

fn remember_ime_rejection(state: &mut OnlineState, event: &UsageEvent) {
    let Some(rejected) = event.word.as_deref().and_then(normalize_word) else {
        return;
    };
    let context = normalized_context(&event.context);
    if context.len() < 2 {
        return;
    }
    if let Some(existing) = state
        .recent_ime_rejections
        .iter_mut()
        .find(|entry| entry.rejected == rejected && entry.context == context)
    {
        existing.event_ordinal = state.event_ordinal;
        return;
    }
    if state.recent_ime_rejections.len() >= MAX_RECENT_IME_REJECTIONS {
        state.recent_ime_rejections.pop_front();
        state.feedback.expired_ime_rejections =
            state.feedback.expired_ime_rejections.saturating_add(1);
    }
    state.recent_ime_rejections.push_back(PendingImeRejection {
        rejected,
        context,
        event_ordinal: state.event_ordinal,
    });
    state.feedback.stored_ime_rejections = state.feedback.stored_ime_rejections.saturating_add(1);
}

fn causal_ime_choice_observation(
    state: &mut OnlineState,
    event: &UsageEvent,
) -> Option<RelationObservation> {
    if !matches!(
        event.kind.as_str(),
        "accepted_ime" | "confirmed_ime_prediction"
    ) || event.outcome != "confirmed_positive"
        || event.source != "ime"
    {
        return None;
    }
    let expected = event.word.as_deref().and_then(normalize_word)?;
    let mut context = normalized_context(&event.context);
    if event.kind == "accepted_ime"
        && context
            .last()
            .is_some_and(|prefix| prefix != &expected && expected.starts_with(prefix))
    {
        context.pop();
    }
    if context.len() < 2 {
        return None;
    }
    let index = state
        .recent_ime_rejections
        .iter()
        .rposition(|entry| entry.context == context && entry.rejected != expected)?;
    let rejected = state.recent_ime_rejections.remove(index)?;
    let scene = context
        .iter()
        .cloned()
        .chain(std::iter::once(expected.clone()))
        .collect::<Vec<_>>()
        .join(" ");
    Some(RelationObservation {
        rejected: rejected.rejected,
        expected,
        scene,
        source: ObservationSource::CausalImeChoice,
    })
}

fn expire_ime_rejections(state: &mut OnlineState) {
    while state.recent_ime_rejections.front().is_some_and(|entry| {
        state.event_ordinal.saturating_sub(entry.event_ordinal) > MAX_IME_PAIR_EVENT_GAP
    }) {
        state.recent_ime_rejections.pop_front();
        state.feedback.expired_ime_rejections =
            state.feedback.expired_ime_rejections.saturating_add(1);
    }
}

fn normalized_context(tokens: &[String]) -> Vec<String> {
    tokens
        .iter()
        .filter_map(|token| normalize_word(token))
        .collect()
}

pub(super) fn insert_relation_observation(
    state: &mut OnlineState,
    observation: RelationObservation,
) {
    let key = format!("{}\u{1f}{}", observation.rejected, observation.expected);
    let relation = state.pending.entry(key).or_insert_with(|| PendingRelation {
        rejected: observation.rejected,
        expected: observation.expected,
        scenes: Vec::new(),
        last_attempted_scenes: 0,
        last_observed_ordinal: state.event_ordinal,
    });
    relation.last_observed_ordinal = state.event_ordinal;
    if !relation.scenes.contains(&observation.scene) && relation.scenes.len() < MAX_SCENES {
        relation.scenes.push(observation.scene);
    }
}

pub(super) fn enforce_relation_bound(state: &mut OnlineState) {
    while state.pending.len() > MAX_RELATIONS {
        let Some(key) = state
            .pending
            .iter()
            .min_by_key(|(key, relation)| (relation.last_observed_ordinal, *key))
            .map(|(key, _)| key.clone())
        else {
            break;
        };
        state.pending.remove(&key);
        state.feedback.evicted_relations = state.feedback.evicted_relations.saturating_add(1);
    }
}

fn rejected_word(event: &UsageEvent) -> Option<String> {
    let from_words = normalized_words(event.from.as_deref()?);
    let to_words = normalized_words(event.to.as_deref()?);
    if from_words.len() != to_words.len() {
        return None;
    }
    let changed = from_words
        .iter()
        .zip(&to_words)
        .enumerate()
        .filter_map(|(index, (left, right))| (left != right).then_some(index))
        .collect::<Vec<_>>();
    let index = *changed.first()?;
    (changed.len() == 1 && index + 1 == to_words.len()).then(|| from_words[index].clone())
}

fn normalized_words(text: &str) -> Vec<String> {
    text.split_whitespace().filter_map(normalize_word).collect()
}

fn normalize_word(raw: &str) -> Option<String> {
    let word = raw
        .trim_matches(|ch: char| !ch.is_alphanumeric() && ch != '-')
        .to_lowercase();
    (!word.is_empty()
        && word.chars().count() <= 48
        && word.chars().all(|ch| ch.is_alphabetic() || ch == '-'))
    .then_some(word)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepted_fix_requires_context_and_one_tail_change() {
        let event: UsageEvent = serde_json::from_str(
            r#"{"kind":"accepted_fix","outcome":"confirmed_positive","source":"user_correction","word":"ходу","context":["обновлять","модель","по"],"from":"обновлять модель по ход","to":"обновлять модель по ходу"}"#,
        )
        .unwrap();
        let mut state = OnlineState::default();
        let observation = relation_observation(&mut state, &event).unwrap();
        assert_eq!(observation.rejected, "ход");
        assert_eq!(observation.expected, "ходу");
        assert_eq!(observation.scene, "обновлять модель по ходу");
        assert_eq!(observation.source, ObservationSource::DirectCorrection);
    }

    #[test]
    fn rejected_ime_and_confirmed_choice_form_one_causal_relation() {
        let rejected: UsageEvent = serde_json::from_str(
            r#"{"kind":"rejected_ime","outcome":"reverted","source":"ime","word":"все","context":["ну","давай","запросим","ты"]}"#,
        )
        .unwrap();
        let accepted: UsageEvent = serde_json::from_str(
            r#"{"kind":"confirmed_ime_prediction","outcome":"confirmed_positive","source":"ime","word":"вроде","context":["ну","давай","запросим","ты"]}"#,
        )
        .unwrap();
        let mut state = OnlineState::default();

        assert!(relation_observation(&mut state, &rejected).is_none());
        let observation = relation_observation(&mut state, &accepted).unwrap();

        assert_eq!(observation.rejected, "все");
        assert_eq!(observation.expected, "вроде");
        assert_eq!(observation.scene, "ну давай запросим ты вроде");
        assert_eq!(observation.source, ObservationSource::CausalImeChoice);
        assert!(state.recent_ime_rejections.is_empty());
    }

    #[test]
    fn accepted_ime_prefix_is_not_mistaken_for_sentence_context() {
        let rejected: UsageEvent = serde_json::from_str(
            r#"{"kind":"rejected_ime","outcome":"reverted","source":"ime","word":"все","context":["ну","давай","запросим","ты"]}"#,
        )
        .unwrap();
        let accepted: UsageEvent = serde_json::from_str(
            r#"{"kind":"accepted_ime","outcome":"confirmed_positive","source":"ime","word":"вроде","context":["ну","давай","запросим","ты","вро"]}"#,
        )
        .unwrap();
        let mut state = OnlineState::default();

        assert!(relation_observation(&mut state, &rejected).is_none());
        let observation = relation_observation(&mut state, &accepted).unwrap();

        assert_eq!(observation.scene, "ну давай запросим ты вроде");
    }

    #[test]
    fn unrelated_context_cannot_bind_an_ime_rejection() {
        let rejected: UsageEvent = serde_json::from_str(
            r#"{"kind":"rejected_ime","outcome":"reverted","source":"ime","word":"все","context":["ну","давай"]}"#,
        )
        .unwrap();
        let accepted: UsageEvent = serde_json::from_str(
            r#"{"kind":"confirmed_ime_prediction","outcome":"confirmed_positive","source":"ime","word":"вроде","context":["совсем","другой"]}"#,
        )
        .unwrap();
        let mut state = OnlineState::default();

        assert!(relation_observation(&mut state, &rejected).is_none());
        assert!(relation_observation(&mut state, &accepted).is_none());
        assert_eq!(state.recent_ime_rejections.len(), 1);
    }
}

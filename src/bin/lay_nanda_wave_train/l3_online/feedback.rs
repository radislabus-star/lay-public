use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub(super) const STATE_FORMAT: &str = "lay-l3-online-v3-causal-episodes";
pub(super) const DIRECT_STATE_FORMAT: &str = "lay-l3-online-v2-direct-relations";
pub(super) const LEGACY_STATE_FORMAT: &str = "lay-l3-online-v1";
/// Bump only when general compiler/readout/proof semantics can change the
/// verdict of an already attempted relation without adding new evidence.
pub(super) const PROOF_PIPELINE_REVISION: u32 = 1;
pub(super) const MIN_EPISODES: usize = 2;
pub(super) const MIN_SCENES: usize = 2;
const MAX_SCENES: usize = 8;
const MAX_EPISODES: usize = 8;
const MAX_RELATIONS: usize = 128;

#[derive(Clone, Debug, Deserialize, Serialize)]
struct CompletionEditTrace {
    prefix: String,
    accepted_suffix_chars: u32,
    preserved_suffix_chars: u32,
    deleted_chars: u32,
    inserted_chars: u32,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub(super) struct UsageEvent {
    #[serde(default)]
    ts: u64,
    #[serde(default)]
    episode_id: Option<String>,
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
    #[serde(default)]
    completion_edit: Option<CompletionEditTrace>,
    #[serde(default)]
    proposal: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub(super) struct PendingRelation {
    pub(super) rejected: String,
    pub(super) expected: String,
    pub(super) scenes: Vec<String>,
    #[serde(default)]
    pub(super) episode_ids: Vec<String>,
    #[serde(default, alias = "last_attempted_scenes")]
    pub(super) last_attempted_episodes: usize,
    #[serde(default)]
    pub(super) last_observed_ordinal: u64,
}

impl PendingRelation {
    pub(super) fn independent_episodes(&self) -> usize {
        self.episode_ids.len()
    }

    pub(super) fn distinct_scenes(&self) -> usize {
        self.scenes.len()
    }

    pub(super) fn ready_for_impact_probe(&self) -> bool {
        let episodes = self.independent_episodes();
        episodes >= MIN_EPISODES
            && self.distinct_scenes() >= MIN_SCENES
            && episodes > self.last_attempted_episodes
            && episodes.is_power_of_two()
    }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub(super) struct OnlineFeedbackStats {
    parsed_events: u64,
    direct_correction_observations: u64,
    #[serde(default)]
    partial_ime_edit_observations: u64,
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
    #[serde(default)]
    pub(super) proof_pipeline_revision: u32,
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
            proof_pipeline_revision: PROOF_PIPELINE_REVISION,
            source_offset: 0,
            source_device: 0,
            source_inode: 0,
            source_tail_hashes: Vec::new(),
            generation: 0,
            pending: BTreeMap::new(),
            event_ordinal: 0,
            admitted_deltas: 0,
            replayed_source_bytes: 0,
            feedback: OnlineFeedbackStats::default(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ObservationSource {
    DirectCorrection,
    PartialImeEdit,
}

#[derive(Clone, Debug)]
pub(super) struct RelationObservation {
    rejected: String,
    expected: String,
    scene: String,
    episode_id: String,
    pub(super) source: ObservationSource,
}

pub(super) fn relation_observation(
    state: &mut OnlineState,
    event: &UsageEvent,
) -> Option<RelationObservation> {
    state.event_ordinal = state.event_ordinal.saturating_add(1);
    state.feedback.parsed_events = state.feedback.parsed_events.saturating_add(1);
    let observation = direct_relation_observation(event)?;
    match observation.source {
        ObservationSource::DirectCorrection => {
            state.feedback.direct_correction_observations = state
                .feedback
                .direct_correction_observations
                .saturating_add(1);
        }
        ObservationSource::PartialImeEdit => {
            state.feedback.partial_ime_edit_observations = state
                .feedback
                .partial_ime_edit_observations
                .saturating_add(1);
        }
    }
    Some(observation)
}

fn direct_relation_observation(event: &UsageEvent) -> Option<RelationObservation> {
    let source = match (event.kind.as_str(), event.source.as_str()) {
        ("accepted_fix", "user_correction") => ObservationSource::DirectCorrection,
        ("edited_ime", "ime") => ObservationSource::PartialImeEdit,
        _ => return None,
    };
    if event.outcome != "confirmed_positive" || event.context.len() < 2 {
        return None;
    }
    let rejected = proposed_word(event).or_else(|| rejected_word(event))?;
    let expected = normalize_word(event.word.as_deref()?)?;
    if rejected == expected
        || !lay::typing_cpu::TypingCpu::learning_target_is_attested(&expected)
        || (source == ObservationSource::PartialImeEdit
            && !completion_edit_geometry_is_valid(event, &rejected, &expected))
    {
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
        episode_id: causal_episode_id(event),
        source,
    })
}

fn completion_edit_geometry_is_valid(event: &UsageEvent, suggested: &str, expected: &str) -> bool {
    let Some(edit) = event.completion_edit.as_ref() else {
        return false;
    };
    let Some(prefix) = normalize_word(&edit.prefix) else {
        return false;
    };
    if !suggested.starts_with(&prefix) || !expected.starts_with(&prefix) {
        return false;
    }
    let prefix_chars = prefix.chars().count();
    let suggested_chars = suggested.chars().count();
    let expected_chars = expected.chars().count();
    let common_chars = suggested
        .chars()
        .zip(expected.chars())
        .take_while(|(left, right)| left == right)
        .count();
    edit.accepted_suffix_chars == suggested_chars.saturating_sub(prefix_chars) as u32
        && edit.preserved_suffix_chars == common_chars.saturating_sub(prefix_chars) as u32
        && edit.deleted_chars == suggested_chars.saturating_sub(common_chars) as u32
        && edit.inserted_chars == expected_chars.saturating_sub(common_chars) as u32
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
        episode_ids: Vec::new(),
        last_attempted_episodes: 0,
        last_observed_ordinal: state.event_ordinal,
    });
    relation.last_observed_ordinal = state.event_ordinal;
    if relation.episode_ids.contains(&observation.episode_id) {
        return;
    }
    if relation.episode_ids.len() < MAX_EPISODES {
        relation.episode_ids.push(observation.episode_id);
    }
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

fn proposed_word(event: &UsageEvent) -> Option<String> {
    let proposal = event.proposal.as_deref()?;
    let proposal_words = normalized_words(proposal);
    let expected_words = normalized_words(event.to.as_deref()?);
    if proposal_words.len() == 1 && expected_words.len() == 1 {
        return proposal_words.into_iter().next();
    }
    if proposal_words.len() != expected_words.len() {
        return None;
    }
    let changed = proposal_words
        .iter()
        .zip(&expected_words)
        .enumerate()
        .filter_map(|(index, (left, right))| (left != right).then_some(index))
        .collect::<Vec<_>>();
    let index = *changed.first()?;
    (changed.len() == 1 && index + 1 == expected_words.len()).then(|| proposal_words[index].clone())
}

fn causal_episode_id(event: &UsageEvent) -> String {
    if let Some(id) = event.episode_id.as_deref().filter(|id| !id.is_empty()) {
        return id.to_string();
    }
    let mut hash = 0xcbf2_9ce4_8422_2325_u64;
    for part in [
        event.ts.to_string(),
        event.kind.clone(),
        event.source.clone(),
        event.outcome.clone(),
        event.from.clone().unwrap_or_default(),
        event.to.clone().unwrap_or_default(),
        event.proposal.clone().unwrap_or_default(),
        event.context.join("\u{1f}"),
    ] {
        for byte in part.as_bytes() {
            hash = (hash ^ u64::from(*byte)).wrapping_mul(0x0000_0100_0000_01b3);
        }
        hash = (hash ^ 0xff).wrapping_mul(0x0000_0100_0000_01b3);
    }
    format!("legacy-{hash:016x}")
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
    fn prior_online_state_defaults_partial_ime_edit_counter() {
        let mut value = serde_json::to_value(OnlineState::default()).unwrap();
        value
            .get_mut("feedback")
            .and_then(serde_json::Value::as_object_mut)
            .expect("feedback object")
            .remove("partial_ime_edit_observations");

        let state: OnlineState = serde_json::from_value(value).unwrap();

        assert_eq!(state.feedback.partial_ime_edit_observations, 0);
    }

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
    fn accepted_fix_prefers_the_causal_proposal_over_the_typed_source() {
        let event: UsageEvent = serde_json::from_str(
            r#"{"ts":9,"episode_id":"episode-9","kind":"accepted_fix","outcome":"confirmed_positive","source":"user_correction","word":"новости","context":["читай","свежие"],"from":"читай новсти","proposal":"читай новость","to":"читай новости"}"#,
        )
        .unwrap();
        let mut state = OnlineState::default();

        let observation = relation_observation(&mut state, &event).unwrap();

        assert_eq!(observation.rejected, "новость");
        assert_eq!(observation.expected, "новости");
        assert_eq!(observation.episode_id, "episode-9");
    }

    #[test]
    fn automatic_apply_is_not_online_learning_authority() {
        let event: UsageEvent = serde_json::from_str(
            r#"{"ts":9,"kind":"accepted_fix","outcome":"censored","source":"autocorrect","word":"новости","context":["читай","свежие"],"from":"читай новсти","proposal":"читай новости","to":"читай новости"}"#,
        )
        .unwrap();
        let mut state = OnlineState::default();

        assert!(relation_observation(&mut state, &event).is_none());
        assert!(state.pending.is_empty());
    }

    #[test]
    fn repeated_journal_record_does_not_duplicate_a_causal_episode() {
        let first: UsageEvent = serde_json::from_str(
            r#"{"ts":9,"episode_id":"episode-9","kind":"accepted_fix","outcome":"confirmed_positive","source":"user_correction","word":"новости","context":["читай","свежие"],"from":"читай новсти","proposal":"читай новость","to":"читай новости"}"#,
        )
        .unwrap();
        let mut second = first.clone();
        second.ts = 10;
        let mut state = OnlineState::default();

        for event in [&first, &second] {
            let observation = relation_observation(&mut state, event).unwrap();
            insert_relation_observation(&mut state, observation);
        }

        let relation = state.pending.values().next().unwrap();
        assert_eq!(relation.independent_episodes(), 1);
        assert_eq!(relation.distinct_scenes(), 1);
    }

    #[test]
    fn edited_ime_is_one_direct_contextual_relation() {
        let event: UsageEvent = serde_json::from_str(
            r#"{"kind":"edited_ime","outcome":"confirmed_positive","source":"ime","word":"прекрасно","context":["это","было"],"from":"прекрасный","to":"прекрасно","completion_edit":{"prefix":"прек","accepted_suffix_chars":6,"preserved_suffix_chars":4,"deleted_chars":2,"inserted_chars":1}}"#,
        )
        .unwrap();
        let mut state = OnlineState::default();
        let observation = relation_observation(&mut state, &event).unwrap();
        assert_eq!(observation.rejected, "прекрасный");
        assert_eq!(observation.expected, "прекрасно");
        assert_eq!(observation.scene, "это было прекрасно");
        assert_eq!(observation.source, ObservationSource::PartialImeEdit);
        assert_eq!(state.feedback.partial_ime_edit_observations, 1);
    }

    #[test]
    fn separate_rejection_and_prediction_cannot_form_a_relation() {
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
        assert!(relation_observation(&mut state, &accepted).is_none());
        assert_eq!(state.feedback.causal_ime_choice_observations, 0);
    }

    #[test]
    fn accepted_ime_never_binds_a_prior_rejection() {
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
        assert!(relation_observation(&mut state, &accepted).is_none());
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
    }

    #[test]
    fn production_log_typo_cannot_enter_partial_ime_learning() {
        let event: UsageEvent = serde_json::from_str(
            r#"{"kind":"edited_ime","outcome":"confirmed_positive","source":"ime","word":"зарегестрированы","context":["доступ","на","хостинге"],"from":"зарегестрировать","to":"зарегестрированы","completion_edit":{"prefix":"зарегест","accepted_suffix_chars":8,"preserved_suffix_chars":6,"deleted_chars":2,"inserted_chars":2}}"#,
        )
        .unwrap();
        let mut state = OnlineState::default();

        assert!(relation_observation(&mut state, &event).is_none());
        assert_eq!(state.feedback.partial_ime_edit_observations, 0);
    }

    #[test]
    fn forged_partial_ime_geometry_cannot_enter_learning() {
        let event: UsageEvent = serde_json::from_str(
            r#"{"kind":"edited_ime","outcome":"confirmed_positive","source":"ime","word":"прекрасно","context":["это","было"],"from":"прекрасный","to":"прекрасно","completion_edit":{"prefix":"другое","accepted_suffix_chars":6,"preserved_suffix_chars":4,"deleted_chars":2,"inserted_chars":1}}"#,
        )
        .unwrap();
        let mut state = OnlineState::default();

        assert!(relation_observation(&mut state, &event).is_none());
        assert_eq!(state.feedback.partial_ime_edit_observations, 0);
    }
}

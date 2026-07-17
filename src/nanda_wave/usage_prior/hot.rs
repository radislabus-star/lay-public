use std::collections::{HashMap, HashSet};
use std::hash::Hash;
use std::mem;

use crate::typing_memory::{normalize_memory_word, normalized_words};

use super::projection::{UsageEventProjection, TRANSITION_ANY};
use super::{UsageCounts, UsageEvent};

pub(super) const CONTEXT_WORDS: usize = 5;
pub(super) const MIN_CONTEXT_NGRAM: usize = 1;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub(super) struct UsageTextId(u64);

#[derive(Debug, Clone, Copy)]
struct UsageTextIdBuilder {
    state: u64,
    len: u64,
}

impl Default for UsageTextIdBuilder {
    fn default() -> Self {
        Self {
            state: crate::stable_hash::mix64_golden(0x5553_4147_455f_5445),
            len: 0,
        }
    }
}

impl UsageTextIdBuilder {
    fn push_bytes(&mut self, bytes: &[u8]) {
        for byte in bytes {
            self.state = crate::stable_hash::mix64_golden(
                self.state ^ u64::from(*byte) ^ self.len.rotate_left(17),
            );
            self.len = self.len.saturating_add(1);
        }
    }

    fn finish(self) -> UsageTextId {
        UsageTextId(crate::stable_hash::mix64_golden(self.state ^ self.len))
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(super) struct UsageContextIds {
    ids: [UsageTextId; CONTEXT_WORDS],
    len: u8,
}

impl UsageContextIds {
    pub(super) fn as_slice(&self) -> &[UsageTextId] {
        &self.ids[..usize::from(self.len)]
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
struct UsageContextWordKey {
    context: UsageTextId,
    word: UsageTextId,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
struct UsageTransitionKey {
    context: UsageTextId,
    source: UsageTextId,
    operation: UsageTextId,
    state_word: UsageTextId,
    word: UsageTextId,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct UsageHotCountMap<K>
where
    K: Eq + Hash + Copy,
{
    counts: HashMap<K, u32>,
}

impl<K> UsageHotCountMap<K>
where
    K: Eq + Hash + Copy,
{
    fn from_text_map(source: &HashMap<String, u32>, key: impl Fn(&str) -> Option<K>) -> Self {
        let mut hot = Self {
            counts: HashMap::<K, u32>::with_capacity(source.len()),
        };
        for (text, count) in source {
            let Some(key) = key(text) else {
                continue;
            };
            hot.increment(key, *count);
        }
        hot
    }

    fn get_text(&self, text: &str) -> u32
    where
        K: From<UsageTextId>,
    {
        self.get(K::from(usage_text_id(text)))
    }

    fn get(&self, key: K) -> u32 {
        self.counts.get(&key).copied().unwrap_or_default()
    }

    fn increment(&mut self, key: K, count: u32) {
        let value = self.counts.entry(key).or_default();
        *value = value.saturating_add(count);
    }

    fn logical_payload_bytes(&self) -> usize {
        self.counts
            .len()
            .saturating_mul(mem::size_of::<K>() + mem::size_of::<u32>())
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(super) struct UsageHotState {
    words: UsageHotCountMap<UsageTextId>,
    accepted_words: UsageHotCountMap<UsageTextId>,
    context_words: UsageHotCountMap<UsageContextWordKey>,
    rejected_words: UsageHotCountMap<UsageTextId>,
    rejected_context_words: UsageHotCountMap<UsageContextWordKey>,
    transition_attract: UsageHotCountMap<UsageTransitionKey>,
    transition_repel: UsageHotCountMap<UsageTransitionKey>,
    surface_observed: UsageHotCountMap<UsageTextId>,
    surface_attract: UsageHotCountMap<UsageTextId>,
    surface_repel: UsageHotCountMap<UsageTextId>,
    phase_witness: super::super::l4_phase_witness::L4PhaseWitnessBank,
}

impl UsageHotState {
    pub(super) fn from_counts(counts: &UsageCounts) -> Self {
        Self {
            words: UsageHotCountMap::from_text_map(&counts.words, |text| Some(usage_text_id(text))),
            accepted_words: UsageHotCountMap::from_text_map(&counts.accepted_words, |text| {
                Some(usage_text_id(text))
            }),
            context_words: UsageHotCountMap::from_text_map(
                &counts.context_words,
                parse_context_word_key,
            ),
            rejected_words: UsageHotCountMap::from_text_map(&counts.rejected_words, |text| {
                Some(usage_text_id(text))
            }),
            rejected_context_words: UsageHotCountMap::from_text_map(
                &counts.rejected_context_words,
                parse_context_word_key,
            ),
            transition_attract: UsageHotCountMap::from_text_map(
                &counts.transition_attract,
                parse_transition_key,
            ),
            transition_repel: UsageHotCountMap::from_text_map(
                &counts.transition_repel,
                parse_transition_key,
            ),
            surface_observed: UsageHotCountMap::from_text_map(&counts.surface_observed, |text| {
                Some(usage_text_id(text))
            }),
            surface_attract: UsageHotCountMap::from_text_map(&counts.surface_attract, |text| {
                Some(usage_text_id(text))
            }),
            surface_repel: UsageHotCountMap::from_text_map(&counts.surface_repel, |text| {
                Some(usage_text_id(text))
            }),
            phase_witness: super::super::l4_phase_witness::L4PhaseWitnessBank::compile(
                &counts.surface_attract,
                &counts.surface_repel,
            ),
        }
    }

    pub(super) fn logical_payload_bytes(&self) -> usize {
        self.words
            .logical_payload_bytes()
            .saturating_add(self.accepted_words.logical_payload_bytes())
            .saturating_add(self.context_words.logical_payload_bytes())
            .saturating_add(self.rejected_words.logical_payload_bytes())
            .saturating_add(self.rejected_context_words.logical_payload_bytes())
            .saturating_add(self.transition_attract.logical_payload_bytes())
            .saturating_add(self.transition_repel.logical_payload_bytes())
            .saturating_add(self.surface_observed.logical_payload_bytes())
            .saturating_add(self.surface_attract.logical_payload_bytes())
            .saturating_add(self.surface_repel.logical_payload_bytes())
            .saturating_add(self.phase_witness.logical_payload_bytes())
    }

    pub(super) fn apply_event(&mut self, event: &UsageEvent) {
        let Some(projected) = UsageEventProjection::from_event(event) else {
            return;
        };
        if projected.is_rejected() {
            if let Some(surface) = projected.surface {
                let surface_id = usage_text_id(surface);
                self.surface_observed
                    .increment(surface_id, projected.weight);
                self.surface_repel.increment(surface_id, projected.weight);
                self.phase_witness
                    .observe_negative(surface, projected.weight);
            }
            self.add_rejected_word_state(RejectedStateEvidence {
                context: projected.context,
                source: projected.source,
                operation: projected.operation,
                state_word: &projected.state_word,
                rejected: &projected.word,
                transition_context: &projected.transition_context,
                transition_target: &projected.transition_target,
                weight: projected.weight,
                transition_weight: projected.transition_weight,
                record_transition: true,
            });
            return;
        }

        if let Some(surface) = projected.surface {
            let surface_id = usage_text_id(surface);
            self.surface_observed
                .increment(surface_id, projected.weight);
            if projected.is_accepted() {
                self.surface_attract.increment(surface_id, projected.weight);
                self.phase_witness
                    .observe_positive(surface, projected.weight);
            }
        }

        let word_id = usage_text_id(&projected.word);
        self.words.increment(word_id, projected.weight);
        if projected.is_accepted() {
            self.accepted_words.increment(word_id, projected.weight);
        }
        add_hot_context_counts(
            &mut self.context_words,
            projected.context,
            &projected.word,
            projected.weight,
        );
        if projected.is_accepted() {
            add_hot_transition_counts(
                &mut self.transition_attract,
                &projected.transition_context,
                projected.source,
                projected.operation,
                &projected.state_word,
                &projected.transition_target,
                projected.transition_weight,
            );
        }
        if projected.records_rejected_fix_sources() {
            self.add_rejected_fix_sources(
                event,
                projected.weight,
                projected.source,
                projected.operation,
            );
        }
    }

    pub(super) fn surface_coverage(&self, surface: &str) -> UsageSurfaceCoverage {
        UsageSurfaceCoverage {
            observed: self.surface_observed.get_text(surface),
            accepted: self.surface_attract.get_text(surface),
            rejected: self.surface_repel.get_text(surface),
        }
    }

    pub(super) fn phase_witness(
        &self,
        surface: &str,
    ) -> super::super::l4_phase_witness::L4PhaseWitnessReadout {
        self.phase_witness.readout(surface)
    }

    pub(super) fn word_prior(&self, word: &str) -> f32 {
        let lower = normalize_memory_word(word);
        if lower.is_empty() {
            return 0.0;
        }
        word_prior_from_hot_count(self.words.get_text(&lower))
    }

    pub(super) fn context_word_prior(&self, context: &[String], word: &str) -> f32 {
        let lower = normalize_memory_word(word);
        if lower.is_empty() || context.is_empty() {
            return 0.0;
        }
        let context = UsageHotContext::from_words(context);
        context_ngram_prior_from_hot(
            &self.context_words,
            context.context_ids.as_slice(),
            &lower,
            0.020,
        )
    }

    pub(super) fn accepted_word_count(&self, word: &str) -> u32 {
        let lower = normalize_memory_word(word);
        if lower.is_empty() {
            return 0;
        }
        self.accepted_words.get_text(&lower)
    }

    pub(super) fn rejected_word_prior(&self, word: &str) -> f32 {
        let lower = normalize_memory_word(word);
        if lower.is_empty() {
            return 0.0;
        }
        rejected_prior_from_hot_count(self.rejected_words.get_text(&lower))
    }

    pub(super) fn context_rejected_word_prior(&self, context: &[String], word: &str) -> f32 {
        let lower = normalize_memory_word(word);
        if lower.is_empty() || context.is_empty() {
            return 0.0;
        }
        let context = UsageHotContext::from_words(context);
        context_ngram_prior_from_hot(
            &self.rejected_context_words,
            context.context_ids.as_slice(),
            &lower,
            0.012,
        )
    }

    pub(super) fn candidate_prior_prepared(
        &self,
        context: &UsageHotContext,
        normalized_word: &str,
    ) -> UsageCandidatePrior {
        if normalized_word.is_empty() {
            return UsageCandidatePrior::default();
        }
        UsageCandidatePrior {
            word_prior: word_prior_from_hot_count(self.words.get_text(normalized_word)),
            context_prior: context_ngram_prior_from_hot(
                &self.context_words,
                context.context_ids.as_slice(),
                normalized_word,
                0.020,
            ),
            accepted_count: self.accepted_words.get_text(normalized_word),
        }
    }

    pub(super) fn hot_readout_prepared(
        &self,
        context: &UsageHotContext,
        source: &str,
        operation: &str,
        state_word: &str,
        candidate_text: &str,
    ) -> UsageHotReadout {
        let lower = normalized_words(candidate_text)
            .into_iter()
            .next_back()
            .unwrap_or_default();
        if lower.is_empty() {
            return UsageHotReadout::default();
        }
        let transition_target = crate::transition_relation::transition_target_id(candidate_text);
        let context_ids = context.context_ids.as_slice();
        UsageHotReadout {
            word_prior: word_prior_from_hot_count(self.words.get_text(&lower)),
            context_prior: context_ngram_prior_from_hot(
                &self.context_words,
                context_ids,
                &lower,
                0.020,
            ),
            rejected_prior: rejected_prior_from_hot_count(self.rejected_words.get_text(&lower)),
            context_rejected: context_ngram_prior_from_hot(
                &self.rejected_context_words,
                context_ids,
                &lower,
                0.012,
            ),
            accepted_count: self.accepted_words.get_text(&lower),
            rejected_count: self.rejected_words.get_text(&lower),
            transition: transition_signal_from_hot_for_word(
                self,
                context_ids,
                source,
                operation,
                state_word,
                &transition_target,
            ),
        }
    }

    fn add_rejected_word_state(&mut self, evidence: RejectedStateEvidence<'_>) {
        let RejectedStateEvidence {
            context,
            source,
            operation,
            state_word,
            rejected,
            transition_context,
            transition_target,
            weight,
            transition_weight,
            record_transition,
        } = evidence;
        self.rejected_words
            .increment(usage_text_id(rejected), weight);
        add_hot_context_counts(&mut self.rejected_context_words, context, rejected, weight);
        if record_transition {
            add_hot_transition_counts(
                &mut self.transition_repel,
                transition_context,
                source,
                operation,
                state_word,
                transition_target,
                transition_weight,
            );
        }
    }

    fn add_rejected_fix_sources(
        &mut self,
        event: &UsageEvent,
        weight: u32,
        source: &str,
        operation: &str,
    ) {
        let Some(from) = event.from.as_deref() else {
            return;
        };
        let accepted = event
            .to
            .as_deref()
            .map(normalized_words)
            .unwrap_or_default()
            .into_iter()
            .collect::<HashSet<_>>();
        for rejected in normalized_words(from)
            .into_iter()
            .filter(|word| !accepted.contains(word))
        {
            self.add_rejected_word_state(RejectedStateEvidence {
                context: &event.context,
                source,
                operation,
                state_word: &rejected,
                rejected: &rejected,
                transition_context: &event.context,
                transition_target: &rejected,
                weight,
                transition_weight: weight,
                record_transition: false,
            });
        }
    }

    #[cfg(test)]
    pub(super) fn rejected_word_count_for_tests(&self, word: &str) -> u32 {
        self.rejected_words.get_text(word)
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub(crate) struct UsageTransitionSignal {
    pub(crate) attraction: f32,
    pub(crate) repulsion: f32,
    pub(crate) signed_weight: f32,
    pub(crate) attract_count: u32,
    pub(crate) repel_count: u32,
    pub(crate) state_specific: bool,
    pub(crate) reason: &'static str,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct UsageSurfaceCoverage {
    pub(crate) observed: u32,
    pub(crate) accepted: u32,
    pub(crate) rejected: u32,
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub(crate) struct UsageHotReadout {
    pub(crate) word_prior: f32,
    pub(crate) context_prior: f32,
    pub(crate) rejected_prior: f32,
    pub(crate) context_rejected: f32,
    pub(crate) accepted_count: u32,
    pub(crate) rejected_count: u32,
    pub(crate) transition: UsageTransitionSignal,
}

#[derive(Debug, Clone, Default)]
pub(crate) struct UsageHotContext {
    context_ids: UsageContextIds,
}

impl UsageHotContext {
    pub(super) fn from_words(context: &[String]) -> Self {
        Self {
            context_ids: context_ngram_ids(context),
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub(crate) struct UsageCandidatePrior {
    pub(crate) word_prior: f32,
    pub(crate) context_prior: f32,
    pub(crate) accepted_count: u32,
}

struct RejectedStateEvidence<'a> {
    context: &'a [String],
    source: &'a str,
    operation: &'a str,
    state_word: &'a str,
    rejected: &'a str,
    transition_context: &'a [String],
    transition_target: &'a str,
    weight: u32,
    transition_weight: u32,
    record_transition: bool,
}

fn add_hot_context_counts(
    target: &mut UsageHotCountMap<UsageContextWordKey>,
    context: &[String],
    word: &str,
    weight: u32,
) {
    let word = usage_text_id(word);
    for context in context_ngram_ids(context).as_slice() {
        target.increment(
            UsageContextWordKey {
                context: *context,
                word,
            },
            weight,
        );
    }
}

fn add_hot_transition_counts(
    target: &mut UsageHotCountMap<UsageTransitionKey>,
    context: &[String],
    source: &str,
    operation: &str,
    state_word: &str,
    word: &str,
    weight: u32,
) {
    let context_ids = context_ngram_ids(context);
    for key in transition_record_keys_from_context_ids(
        context_ids.as_slice(),
        source,
        operation,
        state_word,
        word,
    ) {
        target.increment(key, weight);
    }
}

fn word_prior_from_count(count: u32) -> f32 {
    ((count as f32 + 1.0).ln() * 0.036).clamp(0.0, 0.22)
}

fn word_prior_from_hot_count(count: u32) -> f32 {
    if count > 0 {
        word_prior_from_count(count)
    } else {
        0.0
    }
}

fn rejected_prior_from_count(count: u32) -> f32 {
    ((count as f32 + 1.0).ln() * 0.040).clamp(0.0, 0.26)
}

fn rejected_prior_from_hot_count(count: u32) -> f32 {
    if count > 0 {
        rejected_prior_from_count(count)
    } else {
        0.0
    }
}

fn context_ngram_prior_from_hot(
    source: &UsageHotCountMap<UsageContextWordKey>,
    context_ids: &[UsageTextId],
    word: &str,
    base_weight: f32,
) -> f32 {
    let word = usage_text_id(word);
    context_ids
        .iter()
        .enumerate()
        .filter_map(|(index, context)| {
            let count = source.get(UsageContextWordKey {
                context: *context,
                word,
            });
            (count > 0).then_some((count, index + 1))
        })
        .map(|(count, ngram_len)| {
            let ngram_weight = base_weight + ngram_len as f32 * 0.010;
            ((count as f32 + 1.0).ln() * ngram_weight).min(0.18)
        })
        .sum::<f32>()
        .clamp(0.0, 0.34)
}

fn transition_signal_from_hot_for_word(
    hot: &UsageHotState,
    context_ids: &[UsageTextId],
    source: &str,
    operation: &str,
    state_word: &str,
    word: &str,
) -> UsageTransitionSignal {
    let exact_keys =
        transition_lookup_keys_from_context_ids(context_ids, source, operation, state_word, word);
    let (mut attract_count, mut repel_count) = transition_counts_for_hot_keys(hot, &exact_keys);
    let state_specific = state_word != TRANSITION_ANY && (attract_count > 0 || repel_count > 0);
    if attract_count == 0 && repel_count == 0 && state_word != TRANSITION_ANY {
        let fallback_keys = transition_lookup_keys_from_context_ids(
            context_ids,
            source,
            operation,
            TRANSITION_ANY,
            word,
        );
        (attract_count, repel_count) = transition_counts_for_hot_keys(hot, &fallback_keys);
    }
    let attraction = transition_attraction_from_count(attract_count);
    let repulsion = transition_repulsion_from_count(repel_count);
    let signed_weight = (attraction - repulsion).clamp(-1.0, 1.0);
    let reason = if repel_count > 0 && repulsion > attraction {
        "transition_repels"
    } else if attract_count > 0 && attraction > repulsion {
        "transition_attracts"
    } else if attract_count > 0 || repel_count > 0 {
        "transition_conflict"
    } else {
        "transition_empty"
    };
    UsageTransitionSignal {
        attraction,
        repulsion,
        signed_weight,
        attract_count,
        repel_count,
        state_specific,
        reason,
    }
}

fn transition_counts_for_hot_keys(hot: &UsageHotState, keys: &[UsageTransitionKey]) -> (u32, u32) {
    let attract = keys
        .iter()
        .map(|key| hot.transition_attract.get(*key))
        .max()
        .unwrap_or_default();
    let repel = keys
        .iter()
        .map(|key| hot.transition_repel.get(*key))
        .max()
        .unwrap_or_default();
    (attract, repel)
}

fn transition_attraction_from_count(count: u32) -> f32 {
    if count == 0 {
        return 0.0;
    }
    ((count as f32 + 1.0).ln() * 0.050).clamp(0.0, 0.32)
}

fn transition_repulsion_from_count(count: u32) -> f32 {
    if count == 0 {
        return 0.0;
    }
    ((count as f32 + 1.0).ln() * 0.060).clamp(0.0, 0.38)
}

pub(super) fn context_ngram_ids(context: &[String]) -> UsageContextIds {
    let mut normalized: [Option<String>; CONTEXT_WORDS] = std::array::from_fn(|_| None);
    let mut normalized_len = 0usize;
    for word in context.iter().rev() {
        let word = normalize_memory_word(word);
        if word.is_empty() {
            continue;
        }
        normalized[CONTEXT_WORDS - normalized_len - 1] = Some(word);
        normalized_len += 1;
        if normalized_len == CONTEXT_WORDS {
            break;
        }
    }

    let mut context_ids = UsageContextIds::default();
    for suffix_len in MIN_CONTEXT_NGRAM..=normalized_len {
        let mut id = UsageTextIdBuilder::default();
        for (index, word) in normalized[CONTEXT_WORDS - suffix_len..].iter().enumerate() {
            if index > 0 {
                id.push_bytes(b" ");
            }
            id.push_bytes(
                word.as_deref()
                    .expect("normalized context suffix must be contiguous")
                    .as_bytes(),
            );
        }
        context_ids.ids[suffix_len - 1] = id.finish();
    }
    context_ids.len = normalized_len as u8;
    context_ids
}

fn parse_context_word_key(key: &str) -> Option<UsageContextWordKey> {
    let (context, word) = key.split_once('\u{1f}')?;
    Some(UsageContextWordKey {
        context: usage_text_id(context),
        word: usage_text_id(word),
    })
}

fn transition_lookup_keys_from_context_ids(
    context_ids: &[UsageTextId],
    source: &str,
    operation: &str,
    state_word: &str,
    word: &str,
) -> Vec<UsageTransitionKey> {
    let source = usage_text_id(source);
    let operation = usage_text_id(operation);
    let state_word = usage_text_id(state_word);
    let word = usage_text_id(word);
    let any = usage_text_id(TRANSITION_ANY);
    let mut keys = Vec::with_capacity(context_ids.len().max(1) * 3);
    let mut push_context = |context| {
        keys.push(UsageTransitionKey {
            context,
            source,
            operation,
            state_word,
            word,
        });
        keys.push(UsageTransitionKey {
            context,
            source: any,
            operation,
            state_word,
            word,
        });
        keys.push(UsageTransitionKey {
            context,
            source: any,
            operation: any,
            state_word,
            word,
        });
    };
    if context_ids.is_empty() {
        push_context(usage_text_id(""));
    } else {
        for context in context_ids {
            push_context(*context);
        }
    }
    keys.sort_by_key(|key| {
        (
            key.context.0,
            key.source.0,
            key.operation.0,
            key.state_word.0,
            key.word.0,
        )
    });
    keys.dedup();
    keys
}

fn transition_record_keys_from_context_ids(
    context_ids: &[UsageTextId],
    source: &str,
    operation: &str,
    state_word: &str,
    word: &str,
) -> Vec<UsageTransitionKey> {
    let mut keys =
        transition_lookup_keys_from_context_ids(context_ids, source, operation, state_word, word);
    if state_word != TRANSITION_ANY {
        keys.extend(transition_lookup_keys_from_context_ids(
            context_ids,
            source,
            operation,
            TRANSITION_ANY,
            word,
        ));
    }
    keys.sort_by_key(|key| {
        (
            key.context.0,
            key.source.0,
            key.operation.0,
            key.state_word.0,
            key.word.0,
        )
    });
    keys.dedup();
    keys
}

fn parse_transition_key(key: &str) -> Option<UsageTransitionKey> {
    let (prefix, word) = key.rsplit_once('\u{1d}')?;
    let (left, state_word) = prefix.rsplit_once('\u{1f}')?;
    let mut parts = left.split('\u{1e}');
    let context = parts.next()?;
    let source = parts.next()?;
    let operation = parts.next()?;
    if parts.next().is_some() {
        return None;
    }
    Some(UsageTransitionKey {
        context: usage_text_id(context),
        source: usage_text_id(source),
        operation: usage_text_id(operation),
        state_word: usage_text_id(state_word),
        word: usage_text_id(word),
    })
}

pub(super) fn usage_text_id(text: &str) -> UsageTextId {
    let mut id = UsageTextIdBuilder::default();
    id.push_bytes(text.as_bytes());
    id.finish()
}

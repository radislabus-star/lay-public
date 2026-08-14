//! Proof-only sound bounds over the complete V8 forward posting field.

use std::cmp::Reverse;
use std::collections::{BTreeMap, BinaryHeap};
use std::io;
use std::path::Path;
use std::sync::Arc;
use std::time::Instant;

use rayon::prelude::*;
use serde::Serialize;
use sha2::{Digest, Sha256};

use super::atoms::AtomChannel;
use super::forward_decoder_index::ForwardDecoderIndex;
use super::model::WaveCoupling;
use super::proof::{corpus_words_from_lines, prepare_fixed_heldout_cases};
use super::runtime::{ForwardActivation, LexicalGrokkingMemory, ObservedAtom};
use super::typed_edit_traversal::phase7d_terminal_evidence;

const POSITION_BUCKETS: usize = 16;
const DEFAULT_REQUESTED_K: usize = 128;
const WAND_TERMINAL_SHARDS: usize = 16;
const HOT_LATENCY_LIMIT_US: u64 = 5_000;
const PACKAGE_LIMIT_BYTES: u64 = 195 * 1024 * 1024;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct PostingGroupDescriptor {
    relation_start: u32,
    relation_count: u16,
    first_terminal: u32,
    last_terminal: u32,
    max_strength_by_position: [u8; POSITION_BUCKETS],
}

impl PostingGroupDescriptor {
    fn contribution_upper(self, atom: ObservedAtom) -> u64 {
        self.max_strength_by_position
            .iter()
            .copied()
            .enumerate()
            .map(|(bucket, strength)| {
                u64::from(strength)
                    .saturating_mul(u64::from(atom.weight))
                    .saturating_mul(u64::from(bucket_position_coherence_upper(
                        atom.position,
                        bucket,
                    )))
            })
            .max()
            .unwrap_or_default()
    }
}

#[derive(Debug)]
struct PostingBoundIndex {
    group_relations: usize,
    atom_offsets: Vec<u32>,
    atom_global_maxima: Vec<[u8; POSITION_BUCKETS]>,
    groups: Vec<PostingGroupDescriptor>,
    relation_count: usize,
}

impl PostingBoundIndex {
    fn build_global(memory: &LexicalGrokkingMemory) -> Result<Self, String> {
        let terminal_count = memory.package.terminal_count();
        let per_atom = (0..memory.package.atoms.len() as u32)
            .into_par_iter()
            .map(|atom_id| {
                let relations = memory.complete_forward_couplings(atom_id)?;
                validate_forward_posting(&relations, terminal_count)?;
                let mut maxima = [0_u8; POSITION_BUCKETS];
                for relation in relations.iter() {
                    let bucket = expected_position_bucket(relation.position_mode);
                    maxima[bucket] = maxima[bucket].max(relation.strength);
                }
                Ok((maxima, relations.len()))
            })
            .collect::<Result<Vec<_>, String>>()?;
        let relation_count = per_atom
            .iter()
            .map(|(_, relation_count)| *relation_count)
            .sum::<usize>();
        if relation_count != memory.forward_relation_count() {
            return Err(format!(
                "posting envelope relation count differs: {relation_count} != {}",
                memory.forward_relation_count()
            ));
        }
        Ok(Self {
            group_relations: 0,
            atom_offsets: Vec::new(),
            atom_global_maxima: per_atom.into_iter().map(|(maxima, _)| maxima).collect(),
            groups: Vec::new(),
            relation_count,
        })
    }

    fn build_interval(
        memory: &LexicalGrokkingMemory,
        group_relations: usize,
    ) -> Result<Self, String> {
        if group_relations == 0 || group_relations > usize::from(u16::MAX) {
            return Err("posting group size must be in 1..=65535".to_string());
        }
        let terminal_count = memory.package.terminal_count();
        let per_atom = (0..memory.package.atoms.len() as u32)
            .into_par_iter()
            .map(|atom_id| {
                let relations = memory.complete_forward_couplings(atom_id)?;
                build_atom_descriptors(&relations, terminal_count, group_relations)
            })
            .collect::<Result<Vec<_>, String>>()?;

        let mut atom_offsets = Vec::with_capacity(per_atom.len().saturating_add(1));
        let mut atom_global_maxima = Vec::with_capacity(per_atom.len());
        let mut groups = Vec::new();
        let mut relation_count = 0_usize;
        atom_offsets.push(0);
        for mut atom_groups in per_atom {
            let mut global = [0_u8; POSITION_BUCKETS];
            for group in &atom_groups {
                for (target, source) in global
                    .iter_mut()
                    .zip(group.max_strength_by_position.iter().copied())
                {
                    *target = (*target).max(source);
                }
            }
            atom_global_maxima.push(global);
            relation_count = relation_count.saturating_add(
                atom_groups
                    .iter()
                    .map(|group| usize::from(group.relation_count))
                    .sum::<usize>(),
            );
            groups.append(&mut atom_groups);
            atom_offsets.push(
                u32::try_from(groups.len())
                    .map_err(|_| "posting descriptor count exceeds u32".to_string())?,
            );
        }
        if relation_count != memory.forward_relation_count() {
            return Err(format!(
                "posting descriptor relation count differs: {relation_count} != {}",
                memory.forward_relation_count()
            ));
        }
        Ok(Self {
            group_relations,
            atom_offsets,
            atom_global_maxima,
            groups,
            relation_count,
        })
    }

    fn atom_groups(&self, atom_id: u32) -> &[PostingGroupDescriptor] {
        let index = atom_id as usize;
        let Some(&start) = self.atom_offsets.get(index) else {
            return &[];
        };
        let Some(&end) = self.atom_offsets.get(index.saturating_add(1)) else {
            return &[];
        };
        self.groups
            .get(start as usize..end as usize)
            .unwrap_or_default()
    }

    fn atom_global_maxima(&self, atom_id: u32) -> [u8; POSITION_BUCKETS] {
        self.atom_global_maxima
            .get(atom_id as usize)
            .copied()
            .unwrap_or_default()
    }

    fn resident_bytes(&self) -> usize {
        self.atom_offsets
            .len()
            .saturating_mul(std::mem::size_of::<u32>())
            .saturating_add(
                self.groups
                    .len()
                    .saturating_mul(std::mem::size_of::<PostingGroupDescriptor>()),
            )
            .saturating_add(
                self.atom_global_maxima
                    .len()
                    .saturating_mul(std::mem::size_of::<[u8; POSITION_BUCKETS]>()),
            )
    }

    fn packed_projection_bytes(&self) -> usize {
        self.atom_offsets
            .len()
            .saturating_mul(std::mem::size_of::<u32>())
            .saturating_add(self.groups.len().saturating_mul(22))
    }

    fn global_envelope_projection_bytes(&self) -> usize {
        self.atom_global_maxima
            .len()
            .saturating_mul(std::mem::size_of::<[u8; POSITION_BUCKETS]>())
    }
}

fn build_atom_descriptors(
    relations: &[WaveCoupling],
    terminal_count: u32,
    group_relations: usize,
) -> Result<Vec<PostingGroupDescriptor>, String> {
    validate_forward_posting(relations, terminal_count)?;

    relations
        .chunks(group_relations)
        .enumerate()
        .map(|(group_index, group)| {
            let first = group
                .first()
                .ok_or_else(|| "empty posting group".to_string())?;
            let last = group
                .last()
                .ok_or_else(|| "empty posting group".to_string())?;
            let mut maxima = [0_u8; POSITION_BUCKETS];
            for relation in group {
                let bucket = expected_position_bucket(relation.position_mode);
                maxima[bucket] = maxima[bucket].max(relation.strength);
            }
            Ok(PostingGroupDescriptor {
                relation_start: u32::try_from(group_index.saturating_mul(group_relations))
                    .map_err(|_| "posting relation offset exceeds u32".to_string())?,
                relation_count: u16::try_from(group.len())
                    .map_err(|_| "posting group relation count exceeds u16".to_string())?,
                first_terminal: first.peer_id,
                last_terminal: last.peer_id,
                max_strength_by_position: maxima,
            })
        })
        .collect()
}

fn validate_forward_posting(relations: &[WaveCoupling], terminal_count: u32) -> Result<(), String> {
    for (index, relation) in relations.iter().copied().enumerate() {
        if relation.peer_id >= terminal_count {
            return Err(format!(
                "posting terminal {} exceeds center count {terminal_count}",
                relation.peer_id
            ));
        }
        if index > 0 && relations[index - 1].peer_id >= relation.peer_id {
            return Err("posting terminal IDs must be strictly increasing".to_string());
        }
        if relation.flags != 0 {
            return Err("forward posting relation carries non-forward flags".to_string());
        }
    }
    Ok(())
}

fn expected_position_bucket(position: u8) -> usize {
    usize::from(position).saturating_mul(POSITION_BUCKETS) / 256
}

fn bucket_position_coherence_upper(observed: u8, bucket: usize) -> u16 {
    let width = 256 / POSITION_BUCKETS;
    let low = bucket.saturating_mul(width).min(255) as u8;
    let high = (bucket
        .saturating_add(1)
        .saturating_mul(width)
        .saturating_sub(1))
    .min(255) as u8;
    let distance = if observed < low {
        low - observed
    } else if observed > high {
        observed - high
    } else {
        0
    };
    256_u16.saturating_sub(u16::from(distance))
}

fn exact_contribution(atom: ObservedAtom, relation: WaveCoupling) -> u64 {
    u64::from(relation.strength)
        .saturating_mul(u64::from(atom.weight))
        .saturating_mul(u64::from(256_u16.saturating_sub(u16::from(
            atom.position.abs_diff(relation.position_mode),
        ))))
}

struct QueryPosting<'a> {
    atom: ObservedAtom,
    relations: Arc<[WaveCoupling]>,
    groups: &'a [PostingGroupDescriptor],
    global_maxima: [u8; POSITION_BUCKETS],
}

fn query_postings<'a>(
    memory: &LexicalGrokkingMemory,
    index: &'a PostingBoundIndex,
    surface: &str,
) -> Result<Vec<QueryPosting<'a>>, String> {
    memory
        .resolve_surface(surface)
        .into_iter()
        .filter(|(_, atom)| atom.channel != AtomChannel::CharacterAnchor)
        .collect::<BTreeMap<_, _>>()
        .into_iter()
        .map(|(atom_id, atom)| {
            Ok(QueryPosting {
                atom,
                relations: memory.complete_forward_couplings(atom_id)?,
                groups: index.atom_groups(atom_id),
                global_maxima: index.atom_global_maxima(atom_id),
            })
        })
        .collect()
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize)]
struct SearchMetrics {
    posting_relations_total: usize,
    posting_relations_decoded: usize,
    posting_groups_total: usize,
    centers_evaluated: usize,
    upper_bound_violations: usize,
    posting_relations_skipped: usize,
    posting_iterators: usize,
    candidates_scored: usize,
    scheduler_iterations: usize,
    posting_seeks: usize,
}

#[derive(Debug)]
struct PostingClosure {
    beta_k: u64,
    retained: Vec<(u32, ForwardActivation)>,
    metrics: SearchMetrics,
}

fn exact_posting_closure(
    postings: &[QueryPosting<'_>],
    terminal_count: u32,
    requested_k: usize,
) -> PostingClosure {
    let mut activations = vec![ForwardActivation::default(); terminal_count as usize];
    for posting in postings {
        let keyboard = is_keyboard_channel(posting.atom.channel);
        for relation in posting.relations.iter().copied() {
            add_relation(
                &mut activations[relation.peer_id as usize],
                posting.atom,
                relation,
                keyboard,
            );
        }
    }
    let effective_k = requested_k.max(1).min(terminal_count as usize);
    let mut masses = activations
        .iter()
        .map(|activation| activation.mass)
        .collect::<Vec<_>>();
    masses.select_nth_unstable_by(effective_k - 1, |left, right| right.cmp(left));
    let beta_k = masses[effective_k - 1];
    let retained = activations
        .into_iter()
        .enumerate()
        .filter_map(|(terminal_id, activation)| {
            (activation.mass >= beta_k).then_some((terminal_id as u32, activation))
        })
        .collect();
    PostingClosure {
        beta_k,
        retained,
        metrics: SearchMetrics {
            posting_relations_total: postings.iter().map(|posting| posting.relations.len()).sum(),
            posting_relations_decoded: postings.iter().map(|posting| posting.relations.len()).sum(),
            posting_groups_total: postings.iter().map(|posting| posting.groups.len()).sum(),
            centers_evaluated: terminal_count as usize,
            ..SearchMetrics::default()
        },
    }
}

fn retained_equal(left: &[(u32, ForwardActivation)], right: &[(u32, ForwardActivation)]) -> bool {
    left.len() == right.len()
        && left
            .iter()
            .zip(right)
            .all(|((left_id, left), (right_id, right))| {
                left_id == right_id
                    && left.mass == right.mass
                    && left.hits == right.hits
                    && left.surface_hits == right.surface_hits
                    && left.keyboard_hits == right.keyboard_hits
            })
}

const POSTING_TERMINAL_SOURCE: u8 = 1;
const TYPED_TERMINAL_SOURCE: u8 = 2;
const DUAL_TERMINAL_SOURCE: u8 = POSTING_TERMINAL_SOURCE | TYPED_TERMINAL_SOURCE;

fn merge_terminal_evidence(
    posting_terminal_ids: &[u32],
    typed_terminal_ids: &[u32],
    typed_first: bool,
) -> BTreeMap<u32, u8> {
    let posting = (posting_terminal_ids, POSTING_TERMINAL_SOURCE);
    let typed = (typed_terminal_ids, TYPED_TERMINAL_SOURCE);
    let sources = if typed_first {
        [typed, posting]
    } else {
        [posting, typed]
    };
    let mut evidence = BTreeMap::new();
    for (terminal_ids, source) in sources {
        for terminal_id in terminal_ids {
            evidence
                .entry(*terminal_id)
                .and_modify(|mask| *mask |= source)
                .or_insert(source);
        }
    }
    evidence
}

#[derive(Clone, Copy, Debug)]
struct WandCursor {
    posting_index: usize,
    offset: usize,
    end: usize,
    upper: u64,
    terminal: u32,
}

fn wand_posting_closure(
    postings: &[QueryPosting<'_>],
    terminal_count: u32,
    requested_k: usize,
    terminal_shards: usize,
    initial_beta: Option<u64>,
) -> PostingClosure {
    if terminal_count == 0 {
        return PostingClosure {
            beta_k: 0,
            retained: Vec::new(),
            metrics: SearchMetrics::default(),
        };
    }
    let requested_k = requested_k.max(1).min(terminal_count as usize);
    let shard_count = terminal_shards.max(1).min(terminal_count as usize);
    let mut closures = (0..shard_count)
        .into_par_iter()
        .map(|shard| {
            let low = (u64::from(terminal_count) * shard as u64 / shard_count as u64) as u32;
            let high = (u64::from(terminal_count) * (shard + 1) as u64 / shard_count as u64) as u32;
            wand_posting_range(postings, low, high, requested_k, initial_beta)
        })
        .collect::<Vec<_>>();
    let mut metrics = SearchMetrics::default();
    let mut retained = Vec::new();
    for closure in &mut closures {
        merge_search_metrics(&mut metrics, closure.metrics);
        retained.append(&mut closure.retained);
    }
    metrics.posting_groups_total = postings.iter().map(|posting| posting.groups.len()).sum();
    let mut masses = retained
        .iter()
        .map(|(_, activation)| activation.mass)
        .collect::<Vec<_>>();
    masses.select_nth_unstable_by(requested_k - 1, |left, right| right.cmp(left));
    let beta_k = masses[requested_k - 1];
    retained.retain(|(_, activation)| activation.mass >= beta_k);
    retained.sort_unstable_by_key(|(terminal_id, _)| *terminal_id);
    PostingClosure {
        beta_k,
        retained,
        metrics,
    }
}

fn wand_posting_range(
    postings: &[QueryPosting<'_>],
    low: u32,
    high: u32,
    requested_k: usize,
    initial_beta: Option<u64>,
) -> PostingClosure {
    let requested_k = requested_k.max(1).min((high - low) as usize);
    let mut metrics = SearchMetrics::default();
    let mut cursors = postings
        .iter()
        .enumerate()
        .filter_map(|(posting_index, posting)| {
            let start = posting
                .relations
                .partition_point(|relation| relation.peer_id < low);
            let end = start
                + posting.relations[start..].partition_point(|relation| relation.peer_id < high);
            metrics.posting_relations_total = metrics
                .posting_relations_total
                .saturating_add(end.saturating_sub(start));
            if start == end {
                return None;
            }
            metrics.posting_iterators += 1;
            let upper = range_contribution_upper(posting, low, high);
            Some(WandCursor {
                posting_index,
                offset: start,
                end,
                upper,
                terminal: posting.relations[start].peer_id,
            })
        })
        .collect::<Vec<_>>();
    cursors.sort_unstable_by_key(|cursor| (cursor.terminal, cursor.posting_index));
    let mut active_upper = cursors
        .iter()
        .map(|cursor| cursor.upper)
        .fold(0_u64, u64::saturating_add);
    let mut top_k = BinaryHeap::<Reverse<(u64, u32)>>::new();
    let mut scored = Vec::<(u32, ForwardActivation)>::new();
    let mut cursor_scratch = Vec::with_capacity(cursors.len());

    while !cursors.is_empty() {
        metrics.scheduler_iterations += 1;
        let learned_threshold = (top_k.len() == requested_k)
            .then(|| top_k.peek().map(|minimum| minimum.0 .0))
            .flatten();
        let threshold = match (initial_beta, learned_threshold) {
            (Some(initial), Some(learned)) => Some(initial.max(learned)),
            (initial, learned) => initial.or(learned),
        };
        if threshold.is_some_and(|threshold| active_upper < threshold) {
            break;
        }

        let mut cumulative = 0_u64;
        let pivot_index = cursors
            .iter()
            .position(|cursor| {
                cumulative = cumulative.saturating_add(cursor.upper);
                threshold.is_none_or(|threshold| cumulative >= threshold)
            })
            .unwrap_or(cursors.len() - 1);
        let pivot_terminal = cursors[pivot_index].terminal;

        let changed_end = if cursors[0].terminal == pivot_terminal {
            let equal_end = cursors.partition_point(|cursor| cursor.terminal == pivot_terminal);
            let mut activation = ForwardActivation::default();
            for cursor in &mut cursors[..equal_end] {
                let posting = &postings[cursor.posting_index];
                let relation = posting.relations[cursor.offset];
                metrics.posting_relations_decoded += 1;
                add_relation(
                    &mut activation,
                    posting.atom,
                    relation,
                    is_keyboard_channel(posting.atom.channel),
                );
                cursor.offset += 1;
                if cursor.offset < cursor.end {
                    cursor.terminal = posting.relations[cursor.offset].peer_id;
                } else {
                    cursor.terminal = u32::MAX;
                    active_upper = active_upper.saturating_sub(cursor.upper);
                }
            }
            metrics.candidates_scored += 1;
            metrics.centers_evaluated += 1;
            if top_k.len() < requested_k {
                top_k.push(Reverse((activation.mass, pivot_terminal)));
            } else if top_k
                .peek()
                .is_some_and(|minimum| activation.mass > minimum.0 .0)
            {
                top_k.pop();
                top_k.push(Reverse((activation.mass, pivot_terminal)));
            }
            scored.push((pivot_terminal, activation));
            equal_end
        } else {
            for cursor in &mut cursors[..pivot_index] {
                let relations = &postings[cursor.posting_index].relations;
                cursor.offset =
                    gallop_posting_to(&relations[..cursor.end], cursor.offset, pivot_terminal);
                metrics.posting_seeks += 1;
                if cursor.offset < cursor.end {
                    cursor.terminal = relations[cursor.offset].peer_id;
                } else {
                    cursor.terminal = u32::MAX;
                    active_upper = active_upper.saturating_sub(cursor.upper);
                }
            }
            pivot_index
        };
        restore_wand_prefix_order(&mut cursors, changed_end, &mut cursor_scratch);
    }

    metrics.posting_relations_skipped = metrics
        .posting_relations_total
        .saturating_sub(metrics.posting_relations_decoded);

    let learned_beta = if top_k.len() == requested_k {
        top_k.peek().map(|minimum| minimum.0 .0).unwrap_or_default()
    } else {
        0
    };
    let beta_k = initial_beta.unwrap_or_default().max(learned_beta);
    if beta_k == 0 {
        let by_terminal = scored.into_iter().collect::<BTreeMap<_, _>>();
        scored = (low..high)
            .map(|terminal_id| {
                (
                    terminal_id,
                    by_terminal.get(&terminal_id).copied().unwrap_or_default(),
                )
            })
            .collect();
    } else {
        scored.retain(|(_, activation)| activation.mass >= beta_k);
        scored.sort_unstable_by_key(|(terminal_id, _)| *terminal_id);
    }
    PostingClosure {
        beta_k,
        retained: scored,
        metrics,
    }
}

fn merge_search_metrics(total: &mut SearchMetrics, item: SearchMetrics) {
    total.posting_relations_total += item.posting_relations_total;
    total.posting_relations_decoded += item.posting_relations_decoded;
    total.centers_evaluated += item.centers_evaluated;
    total.upper_bound_violations += item.upper_bound_violations;
    total.posting_relations_skipped += item.posting_relations_skipped;
    total.posting_iterators += item.posting_iterators;
    total.candidates_scored += item.candidates_scored;
    total.scheduler_iterations += item.scheduler_iterations;
    total.posting_seeks += item.posting_seeks;
}

fn gallop_posting_to(relations: &[WaveCoupling], offset: usize, target: u32) -> usize {
    if offset >= relations.len() || relations[offset].peer_id >= target {
        return offset;
    }
    let mut low = offset + 1;
    let mut step = 1_usize;
    loop {
        let probe = offset.saturating_add(step);
        if probe >= relations.len() {
            return low + relations[low..].partition_point(|relation| relation.peer_id < target);
        }
        if relations[probe].peer_id >= target {
            return low
                + relations[low..=probe].partition_point(|relation| relation.peer_id < target);
        }
        low = probe + 1;
        let Some(next) = step.checked_mul(2) else {
            return low + relations[low..].partition_point(|relation| relation.peer_id < target);
        };
        step = next;
    }
}

fn restore_wand_prefix_order(
    cursors: &mut Vec<WandCursor>,
    changed_end: usize,
    scratch: &mut Vec<WandCursor>,
) {
    cursors[..changed_end].sort_unstable_by_key(|cursor| (cursor.terminal, cursor.posting_index));
    scratch.clear();
    let mut left = 0;
    let mut right = changed_end;
    while left < changed_end || right < cursors.len() {
        let take_left = right == cursors.len()
            || (left < changed_end
                && (cursors[left].terminal, cursors[left].posting_index)
                    <= (cursors[right].terminal, cursors[right].posting_index));
        let cursor = if take_left {
            let cursor = cursors[left];
            left += 1;
            cursor
        } else {
            let cursor = cursors[right];
            right += 1;
            cursor
        };
        if cursor.terminal != u32::MAX {
            scratch.push(cursor);
        }
    }
    std::mem::swap(cursors, scratch);
    scratch.clear();
}

fn global_contribution_upper(atom: ObservedAtom, maxima: [u8; POSITION_BUCKETS]) -> u64 {
    maxima
        .iter()
        .copied()
        .enumerate()
        .map(|(bucket, strength)| {
            u64::from(strength)
                .saturating_mul(u64::from(atom.weight))
                .saturating_mul(u64::from(bucket_position_coherence_upper(
                    atom.position,
                    bucket,
                )))
        })
        .max()
        .unwrap_or_default()
}

fn range_contribution_upper(posting: &QueryPosting<'_>, low: u32, high: u32) -> u64 {
    let first = posting
        .groups
        .partition_point(|group| group.last_terminal < low);
    posting.groups[first..]
        .iter()
        .take_while(|group| group.first_terminal < high)
        .map(|group| group.contribution_upper(posting.atom))
        .max()
        .unwrap_or_else(|| global_contribution_upper(posting.atom, posting.global_maxima))
}

fn add_relation(
    activation: &mut ForwardActivation,
    atom: ObservedAtom,
    relation: WaveCoupling,
    keyboard: bool,
) {
    activation.mass = activation
        .mass
        .saturating_add(exact_contribution(atom, relation));
    activation.hits = activation.hits.saturating_add(1);
    if keyboard {
        activation.keyboard_hits = activation.keyboard_hits.saturating_add(1);
    } else {
        activation.surface_hits = activation.surface_hits.saturating_add(1);
    }
}

fn is_keyboard_channel(channel: AtomChannel) -> bool {
    matches!(
        channel,
        AtomChannel::KeyboardGram
            | AtomChannel::KeyboardBigram
            | AtomChannel::KeyboardBagGram
            | AtomChannel::KeyboardSkipGram
    )
}

#[derive(Default)]
struct ClassMetrics {
    cases: usize,
    closure_parity: usize,
    target_in_posting_closure: usize,
    tie_boundary_losses: usize,
    upper_bound_violations: usize,
    latencies_us: Vec<u64>,
    decoded_relations: u64,
    skipped_relations: u64,
    total_relations: u64,
    centers_evaluated: u64,
    posting_iterators: u64,
    candidates_scored: u64,
    scheduler_iterations: u64,
    posting_seeks: u64,
    typed_target_retained: usize,
    typed_terminal_ids: u64,
    typed_states_expanded: u64,
    typed_queue_peak: u64,
    typed_terminal_events: u64,
    typed_union_schedule_parity: usize,
    merged_target_retained: usize,
    merged_terminal_ids: u64,
    typed_only_terminal_ids: u64,
    posting_only_terminal_ids: u64,
    dual_source_terminal_ids: u64,
    typed_traversal_us: Vec<u64>,
    wand_us: Vec<u64>,
}

impl ClassMetrics {
    fn finish(mut self) -> serde_json::Value {
        self.latencies_us.sort_unstable();
        self.typed_traversal_us.sort_unstable();
        self.wand_us.sort_unstable();
        serde_json::json!({
            "cases": self.cases,
            "closure_parity": self.closure_parity,
            "target_in_posting_closure": self.target_in_posting_closure,
            "tie_boundary_losses": self.tie_boundary_losses,
            "upper_bound_violations": self.upper_bound_violations,
            "decoded_relations": self.decoded_relations,
            "skipped_relations": self.skipped_relations,
            "total_relations": self.total_relations,
            "centers_evaluated": self.centers_evaluated,
            "posting_iterators": self.posting_iterators,
            "candidates_scored": self.candidates_scored,
            "scheduler_iterations": self.scheduler_iterations,
            "posting_seeks": self.posting_seeks,
            "typed_target_retained": self.typed_target_retained,
            "typed_terminal_ids": self.typed_terminal_ids,
            "typed_states_expanded": self.typed_states_expanded,
            "typed_queue_peak": self.typed_queue_peak,
            "typed_terminal_events": self.typed_terminal_events,
            "typed_union_schedule_parity": self.typed_union_schedule_parity,
            "merged_target_retained": self.merged_target_retained,
            "merged_terminal_ids": self.merged_terminal_ids,
            "typed_only_terminal_ids": self.typed_only_terminal_ids,
            "posting_only_terminal_ids": self.posting_only_terminal_ids,
            "dual_source_terminal_ids": self.dual_source_terminal_ids,
            "typed_traversal_us_p50": percentile(&self.typed_traversal_us, 50),
            "typed_traversal_us_p95": percentile(&self.typed_traversal_us, 95),
            "typed_traversal_us_p99": percentile(&self.typed_traversal_us, 99),
            "wand_us_p50": percentile(&self.wand_us, 50),
            "wand_us_p95": percentile(&self.wand_us, 95),
            "wand_us_p99": percentile(&self.wand_us, 99),
            "latency_us_p50": percentile(&self.latencies_us, 50),
            "latency_us_p95": percentile(&self.latencies_us, 95),
            "latency_us_p99": percentile(&self.latencies_us, 99),
        })
    }
}

pub fn prove_l1_posting_bounds(
    corpus_path: &Path,
    package_path: &Path,
    max_words: usize,
    heldout_per_class: usize,
    group_relations: usize,
    requested_k: usize,
    terminal_shards: usize,
) -> io::Result<serde_json::Value> {
    let started = Instant::now();
    let package_sha256_before = file_sha256(package_path)?;
    let words = corpus_words_from_lines(&std::fs::read_to_string(corpus_path)?, max_words);
    let memory = LexicalGrokkingMemory::load(package_path).map_err(io::Error::other)?;
    if words.len() != memory.package.centers.len() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "corpus/package terminal count differs: {} != {}",
                words.len(),
                memory.package.centers.len()
            ),
        ));
    }
    let requested_k = if requested_k == 0 {
        DEFAULT_REQUESTED_K
    } else {
        requested_k
    };
    let terminal_shards = if terminal_shards == 0 {
        WAND_TERMINAL_SHARDS
    } else {
        terminal_shards
    };
    let index_started = Instant::now();
    let posting_index = if group_relations == 0 {
        PostingBoundIndex::build_global(&memory)
    } else {
        PostingBoundIndex::build_interval(&memory, group_relations)
    }
    .map_err(io::Error::other)?;
    let index_ms = index_started.elapsed().as_millis();
    let decoder_index_started = Instant::now();
    let decoder_index = ForwardDecoderIndex::build(&memory.package).map_err(io::Error::other)?;
    let decoder_index_ms = decoder_index_started.elapsed().as_millis();
    let cases = prepare_fixed_heldout_cases(&words, heldout_per_class, 0)?;
    let mut classes = BTreeMap::<&'static str, ClassMetrics>::new();

    for case in &cases {
        let postings =
            query_postings(&memory, &posting_index, &case.surface).map_err(io::Error::other)?;
        let dense = exact_posting_closure(&postings, memory.package.terminal_count(), requested_k);
        let case_started = Instant::now();
        let typed_started = Instant::now();
        let typed =
            phase7d_terminal_evidence(&decoder_index, &memory.package.decoder_nodes, &case.surface)
                .map_err(io::Error::other)?;
        let typed_traversal_us = typed_started
            .elapsed()
            .as_micros()
            .min(u128::from(u64::MAX)) as u64;
        let wand_started = Instant::now();
        let bounded = wand_posting_closure(
            &postings,
            memory.package.terminal_count(),
            requested_k,
            terminal_shards,
            None,
        );
        let wand_us = wand_started.elapsed().as_micros().min(u128::from(u64::MAX)) as u64;
        let bounded_ids = bounded
            .retained
            .iter()
            .map(|(terminal_id, _)| *terminal_id)
            .collect::<Vec<_>>();
        let posting_first = merge_terminal_evidence(&bounded_ids, &typed.terminal_ids, false);
        let typed_first = merge_terminal_evidence(&bounded_ids, &typed.terminal_ids, true);
        let union_schedule_parity = posting_first == typed_first;
        let latency_us = case_started.elapsed().as_micros().min(u128::from(u64::MAX)) as u64;
        let dense_ids = dense
            .retained
            .iter()
            .map(|(terminal_id, _)| *terminal_id)
            .collect::<Vec<_>>();
        let tie_losses = dense_ids
            .iter()
            .filter(|terminal_id| !bounded_ids.binary_search(terminal_id).is_ok())
            .count();
        let parity = dense.beta_k == bounded.beta_k
            && retained_equal(&dense.retained, &bounded.retained)
            && bounded.metrics.upper_bound_violations == 0;
        let class = classes.entry(case.class).or_default();
        class.cases += 1;
        class.closure_parity += usize::from(parity);
        class.target_in_posting_closure +=
            usize::from(bounded_ids.binary_search(&case.terminal_id).is_ok());
        class.typed_target_retained +=
            usize::from(typed.terminal_ids.binary_search(&case.terminal_id).is_ok());
        class.typed_terminal_ids = class
            .typed_terminal_ids
            .saturating_add(typed.terminal_ids.len() as u64);
        class.typed_states_expanded = class
            .typed_states_expanded
            .saturating_add(typed.states_expanded);
        class.typed_queue_peak = class.typed_queue_peak.max(typed.queue_peak as u64);
        class.typed_terminal_events = class
            .typed_terminal_events
            .saturating_add(typed.terminal_events);
        class.typed_union_schedule_parity += usize::from(union_schedule_parity);
        class.merged_target_retained += usize::from(posting_first.contains_key(&case.terminal_id));
        class.merged_terminal_ids = class
            .merged_terminal_ids
            .saturating_add(posting_first.len() as u64);
        for source_mask in posting_first.values() {
            match *source_mask {
                TYPED_TERMINAL_SOURCE => class.typed_only_terminal_ids += 1,
                POSTING_TERMINAL_SOURCE => class.posting_only_terminal_ids += 1,
                DUAL_TERMINAL_SOURCE => {
                    class.dual_source_terminal_ids += 1;
                }
                _ => unreachable!("terminal evidence source mask is closed"),
            }
        }
        class.typed_traversal_us.push(typed_traversal_us);
        class.wand_us.push(wand_us);
        class.tie_boundary_losses += tie_losses;
        class.upper_bound_violations += bounded.metrics.upper_bound_violations;
        class.latencies_us.push(latency_us);
        class.decoded_relations = class
            .decoded_relations
            .saturating_add(bounded.metrics.posting_relations_decoded as u64);
        class.skipped_relations = class
            .skipped_relations
            .saturating_add(bounded.metrics.posting_relations_skipped as u64);
        class.total_relations = class
            .total_relations
            .saturating_add(bounded.metrics.posting_relations_total as u64);
        class.centers_evaluated = class
            .centers_evaluated
            .saturating_add(bounded.metrics.centers_evaluated as u64);
        class.posting_iterators = class
            .posting_iterators
            .saturating_add(bounded.metrics.posting_iterators as u64);
        class.candidates_scored = class
            .candidates_scored
            .saturating_add(bounded.metrics.candidates_scored as u64);
        class.scheduler_iterations = class
            .scheduler_iterations
            .saturating_add(bounded.metrics.scheduler_iterations as u64);
        class.posting_seeks = class
            .posting_seeks
            .saturating_add(bounded.metrics.posting_seeks as u64);
    }

    let classes = classes
        .into_iter()
        .map(|(class, metrics)| (class, metrics.finish()))
        .collect::<BTreeMap<_, _>>();
    let package_sha256_after = file_sha256(package_path)?;
    let package_unchanged = package_sha256_before == package_sha256_after;
    let closure_parity = classes.values().all(|metrics| {
        metrics.get("cases").and_then(serde_json::Value::as_u64)
            == metrics
                .get("closure_parity")
                .and_then(serde_json::Value::as_u64)
    });
    let upper_bound_violations = classes
        .values()
        .filter_map(|metrics| metrics.get("upper_bound_violations")?.as_u64())
        .sum::<u64>();
    let tie_boundary_losses = classes
        .values()
        .filter_map(|metrics| metrics.get("tie_boundary_losses")?.as_u64())
        .sum::<u64>();
    let typed_terminal_losses = classes
        .values()
        .filter_map(|metrics| {
            let cases = metrics.get("cases")?.as_u64()?;
            let retained = metrics.get("typed_target_retained")?.as_u64()?;
            Some(cases.saturating_sub(retained))
        })
        .sum::<u64>();
    let typed_union_schedule_parity = classes.values().all(|metrics| {
        metrics.get("cases").and_then(serde_json::Value::as_u64)
            == metrics
                .get("typed_union_schedule_parity")
                .and_then(serde_json::Value::as_u64)
    });
    let merged_target_losses = classes
        .values()
        .filter_map(|metrics| {
            let cases = metrics.get("cases")?.as_u64()?;
            let retained = metrics.get("merged_target_retained")?.as_u64()?;
            Some(cases.saturating_sub(retained))
        })
        .sum::<u64>();
    let correctness_passed = package_unchanged
        && closure_parity
        && upper_bound_violations == 0
        && tie_boundary_losses == 0
        && typed_terminal_losses == 0
        && typed_union_schedule_parity
        && merged_target_losses == 0;
    let maximum_complete_latency_us = classes
        .values()
        .filter_map(|metrics| metrics.get("latency_us_p99")?.as_u64())
        .max()
        .unwrap_or_default();
    let package_bytes = std::fs::metadata(package_path)?.len();
    let latency_feasible = maximum_complete_latency_us <= HOT_LATENCY_LIMIT_US;
    let package_feasible = package_bytes <= PACKAGE_LIMIT_BYTES;
    let feasibility_passed = latency_feasible && package_feasible;
    let passed = correctness_passed && feasibility_passed;
    let gates = serde_json::json!({
        "correctness_verdict": if correctness_passed { "PASS" } else { "FAIL" },
        "feasibility_verdict": if feasibility_passed { "PASS" } else { "FAIL" },
        "package_limit_bytes": PACKAGE_LIMIT_BYTES,
        "package_feasible": package_feasible,
        "maximum_complete_latency_us": maximum_complete_latency_us,
        "hot_latency_limit_us": HOT_LATENCY_LIMIT_US,
        "latency_feasible": latency_feasible,
    });

    Ok(serde_json::json!({
        "schema": "lay.l11.posting-bound-phase8-proof.v3",
        "verdict": if passed { "PASS" } else { "FAIL" },
        "gates": gates,
        "corpus": corpus_path,
        "package": package_path,
        "package_sha256_before": package_sha256_before,
        "package_sha256_after": package_sha256_after,
        "package_bytes_unchanged": package_unchanged,
        "package_bytes": package_bytes,
        "centers": memory.package.terminal_count(),
        "atoms": memory.package.atoms.len(),
        "forward_relations": posting_index.relation_count,
        "envelope_relations_verified": posting_index.relation_count,
        "envelope_verification_scope": "all_forward_relations_once_during_index_build",
        "group_relations": posting_index.group_relations,
        "posting_groups": posting_index.groups.len(),
        "directory_resident_bytes": posting_index.resident_bytes(),
        "directory_packed_projection_bytes": posting_index.packed_projection_bytes(),
        "global_envelope_projection_bytes": posting_index.global_envelope_projection_bytes(),
        "search": "parallel_terminal_shard_wand_global_max",
        "terminal_shards": terminal_shards,
        "requested_k": requested_k,
        "heldout_per_class": heldout_per_class,
        "heldout_cases": cases.len(),
        "classes": classes,
        "closure_parity_complete": closure_parity,
        "upper_bound_violations": upper_bound_violations,
        "tie_boundary_losses": tie_boundary_losses,
        "typed_terminal_losses": typed_terminal_losses,
        "typed_union_schedule_parity": typed_union_schedule_parity,
        "merged_target_losses": merged_target_losses,
        "generated_target_strings": 0,
        "queue_truncations": 0,
        "runtime_authority_changed": false,
        "package_format_changed": false,
        "index_build_ms": index_ms,
        "decoder_index_build_ms": decoder_index_ms,
        "wall_ms": started.elapsed().as_millis(),
    }))
}

fn file_sha256(path: &Path) -> io::Result<String> {
    Ok(format!("{:x}", Sha256::digest(std::fs::read(path)?)))
}

fn percentile(values: &[u64], percentile: usize) -> u64 {
    if values.is_empty() {
        return 0;
    }
    values[values.len().saturating_sub(1).saturating_mul(percentile) / 100]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::nanda_wave::lexical_grokking::compiler::compile;
    use crate::nanda_wave::lexical_grokking::training_corpus::TrainingWord;

    fn memory() -> LexicalGrokkingMemory {
        let words = [
            "form",
            "farm",
            "foam",
            "from",
            "frame",
            "формат",
            "ферма",
            "форма",
        ]
        .into_iter()
        .enumerate()
        .map(|(terminal_id, surface)| TrainingWord {
            terminal_id: terminal_id as u32,
            surface: surface.to_string(),
            training_surfaces: Vec::new(),
        })
        .collect::<Vec<_>>();
        LexicalGrokkingMemory::from_package(compile(&words).expect("compile posting-bound package"))
    }

    #[test]
    fn position_descriptor_never_understates_exact_relation_maximum() {
        let relations = (0..32_u32)
            .map(|terminal_id| WaveCoupling {
                peer_id: terminal_id,
                strength: (terminal_id.saturating_mul(7) % 255) as u8,
                position_mode: (terminal_id.saturating_mul(29) % 256) as u8,
                ..WaveCoupling::default()
            })
            .collect::<Vec<_>>();
        let descriptor = build_atom_descriptors(&relations, 32, 32).unwrap()[0];
        for position in 0..=u8::MAX {
            for weight in 1..=3 {
                let atom = ObservedAtom {
                    position,
                    weight,
                    channel: AtomChannel::CharacterGram,
                };
                let exact = relations
                    .iter()
                    .copied()
                    .map(|relation| exact_contribution(atom, relation))
                    .max()
                    .unwrap();
                assert!(descriptor.contribution_upper(atom) >= exact);
            }
        }
    }

    #[test]
    fn posting_descriptor_rejects_duplicate_terminal_ids() {
        let relations = [
            WaveCoupling {
                peer_id: 2,
                strength: 100,
                ..WaveCoupling::default()
            },
            WaveCoupling {
                peer_id: 2,
                strength: 90,
                ..WaveCoupling::default()
            },
        ];
        assert!(build_atom_descriptors(&relations, 3, 32).is_err());
    }

    #[test]
    fn galloping_seek_matches_exact_partition_for_every_suffix() {
        let relations = [1_u32, 3, 7, 8, 20, 21, 40, 90]
            .into_iter()
            .map(|peer_id| WaveCoupling {
                peer_id,
                ..WaveCoupling::default()
            })
            .collect::<Vec<_>>();
        for offset in 0..=relations.len() {
            for target in 0..=100 {
                let exact = offset
                    + relations[offset..].partition_point(|relation| relation.peer_id < target);
                assert_eq!(gallop_posting_to(&relations, offset, target), exact);
            }
        }
    }

    #[test]
    fn query_posting_order_cannot_change_closure() {
        let memory = memory();
        let index = PostingBoundIndex::build_interval(&memory, 2).unwrap();
        let mut postings = query_postings(&memory, &index, "frmo").unwrap();
        let forward = wand_posting_closure(
            &postings,
            memory.package.terminal_count(),
            3,
            WAND_TERMINAL_SHARDS,
            None,
        );
        postings.reverse();
        let reverse = wand_posting_closure(
            &postings,
            memory.package.terminal_count(),
            3,
            WAND_TERMINAL_SHARDS,
            None,
        );
        assert_eq!(forward.beta_k, reverse.beta_k);
        assert!(retained_equal(&forward.retained, &reverse.retained));
    }

    #[test]
    fn wand_posting_closure_matches_dense_and_preserves_ties() {
        let memory = memory();
        let index = PostingBoundIndex::build_interval(&memory, 2).unwrap();
        for surface in ["form", "frmo", "far", "фрма", "форма"] {
            let postings = query_postings(&memory, &index, surface).unwrap();
            let dense = exact_posting_closure(&postings, memory.package.terminal_count(), 3);
            let wand = wand_posting_closure(
                &postings,
                memory.package.terminal_count(),
                3,
                WAND_TERMINAL_SHARDS,
                None,
            );
            assert_eq!(wand.beta_k, dense.beta_k, "{surface}");
            assert!(retained_equal(&wand.retained, &dense.retained), "{surface}");
            assert_eq!(wand.metrics.upper_bound_violations, 0, "{surface}");
        }
    }

    #[test]
    fn terminal_evidence_union_is_schedule_independent_and_preserves_sources() {
        let posting = [1, 2, 5];
        let typed = [2, 3, 5];
        let posting_first = merge_terminal_evidence(&posting, &typed, false);
        let typed_first = merge_terminal_evidence(&posting, &typed, true);
        assert_eq!(posting_first, typed_first);
        assert_eq!(posting_first.get(&1), Some(&POSTING_TERMINAL_SOURCE));
        assert_eq!(posting_first.get(&3), Some(&TYPED_TERMINAL_SOURCE));
        assert_eq!(posting_first.get(&2), Some(&DUAL_TERMINAL_SOURCE));
    }
}

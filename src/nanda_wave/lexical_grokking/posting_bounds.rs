//! Proof-only sound bounds over the complete V8 forward posting field.

mod decoder_subtree_cover;
mod epoch;
mod impact;
mod modal_residual;

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
use super::proof::{corpus_words_from_lines, prepare_fixed_heldout_cases, FixedHeldoutCase};
use super::runtime::{ForwardActivation, LexicalGrokkingMemory, ObservedAtom};
use super::typed_edit_traversal::phase7d_terminal_evidence;
use decoder_subtree_cover::{
    summarize_queries as summarize_subtree_queries, DecoderSubtreeCoverProjection,
    DecoderSubtreeQueryMetrics, SUBTREE_PROJECTION_WORK_LIMIT,
};
use epoch::{EpochSearchMetrics, ShardedEpochAccumulator};
use impact::{
    residual_upper_bound_violations, ImpactPreparationMetrics, ImpactSearchMetrics,
    ImpactThresholdSearch, PreparedImpactQuery,
};
use modal_residual::{
    summarize_queries as summarize_modal_queries, ModalQueryMetrics, ModalResidualProjection,
    PROJECTION_EVENT_LIMIT,
};

const POSITION_BUCKETS: usize = 16;
const DEFAULT_REQUESTED_K: usize = 128;
const WAND_TERMINAL_SHARDS: usize = 16;
const HOT_LATENCY_LIMIT_US: u64 = 5_000;
const IMPACT_INTRINSIC_LIMIT_US: u64 = 2_500;
const PACKAGE_LIMIT_BYTES: u64 = 195 * 1024 * 1024;
const EPOCH_ACCUMULATOR_LIMIT_BYTES: usize = 32 * 1024 * 1024;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PostingSearchMode {
    Wand,
    Epoch,
    Impact,
    Modal,
    Subtree,
}

impl PostingSearchMode {
    fn parse(value: &str) -> io::Result<Self> {
        match value {
            "wand" => Ok(Self::Wand),
            "epoch" => Ok(Self::Epoch),
            "impact" => Ok(Self::Impact),
            "modal" => Ok(Self::Modal),
            "subtree" => Ok(Self::Subtree),
            _ => Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!(
                    "unknown --posting-search {value:?}; expected wand, epoch, impact, modal or subtree"
                ),
            )),
        }
    }

    fn report_name(self) -> &'static str {
        match self {
            Self::Wand => "parallel_terminal_shard_wand_global_max",
            Self::Epoch => "exact_sharded_epoch_accumulator",
            Self::Impact => "ideal_impact_order_residual_threshold",
            Self::Modal => "proof_only_modal_residual_projection",
            Self::Subtree => "proof_only_exact_decoder_subtree_cover_projection",
        }
    }
}

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

fn verify_complete_forward_field(memory: &LexicalGrokkingMemory) -> Result<usize, String> {
    let terminal_count = memory.package.terminal_count();
    let relation_count = (0..memory.package.atoms.len() as u32)
        .into_par_iter()
        .map(|atom_id| {
            let relations = memory.complete_forward_couplings(atom_id)?;
            validate_forward_posting(&relations, terminal_count)?;
            Ok(relations.len())
        })
        .collect::<Result<Vec<_>, String>>()?
        .into_iter()
        .sum::<usize>();
    if relation_count != memory.forward_relation_count() {
        return Err(format!(
            "complete posting relation count differs: {relation_count} != {}",
            memory.forward_relation_count()
        ));
    }
    Ok(relation_count)
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

#[derive(Clone)]
struct QueryPosting<'a> {
    atom_id: u32,
    atom: ObservedAtom,
    relations: Arc<[WaveCoupling]>,
    groups: &'a [PostingGroupDescriptor],
    global_maxima: [u8; POSITION_BUCKETS],
}

fn query_postings<'a>(
    memory: &LexicalGrokkingMemory,
    index: Option<&'a PostingBoundIndex>,
    surface: &str,
) -> Result<Vec<QueryPosting<'a>>, String> {
    memory
        .resolve_surface(surface)
        .into_iter()
        .filter(|(_, atom)| atom.channel != AtomChannel::CharacterAnchor)
        .collect::<BTreeMap<_, _>>()
        .into_iter()
        .map(|(atom_id, atom)| {
            let (groups, global_maxima) = index
                .map(|index| {
                    (
                        index.atom_groups(atom_id),
                        index.atom_global_maxima(atom_id),
                    )
                })
                .unwrap_or((&[], [0; POSITION_BUCKETS]));
            Ok(QueryPosting {
                atom_id,
                atom,
                relations: memory.complete_forward_couplings(atom_id)?,
                groups,
                global_maxima,
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

struct ExactPostingResult {
    closure: PostingClosure,
    touched: Vec<(u32, ForwardActivation)>,
    all: Vec<ForwardActivation>,
}

fn exact_posting_closure(
    postings: &[QueryPosting<'_>],
    terminal_count: u32,
    requested_k: usize,
) -> ExactPostingResult {
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
    if terminal_count == 0 {
        return ExactPostingResult {
            closure: PostingClosure {
                beta_k: 0,
                retained: Vec::new(),
                metrics: SearchMetrics::default(),
            },
            touched: Vec::new(),
            all: Vec::new(),
        };
    }
    let touched = activations
        .iter()
        .copied()
        .enumerate()
        .filter_map(|(terminal_id, activation)| {
            (activation.hits != 0).then_some((terminal_id as u32, activation))
        })
        .collect::<Vec<_>>();
    let effective_k = requested_k.max(1).min(terminal_count as usize);
    let mut masses = activations
        .iter()
        .map(|activation| activation.mass)
        .collect::<Vec<_>>();
    masses.select_nth_unstable_by(effective_k - 1, |left, right| right.cmp(left));
    let beta_k = masses[effective_k - 1];
    let retained = activations
        .iter()
        .copied()
        .enumerate()
        .filter_map(|(terminal_id, activation)| {
            (activation.mass >= beta_k).then_some((terminal_id as u32, activation))
        })
        .collect();
    ExactPostingResult {
        closure: PostingClosure {
            beta_k,
            retained,
            metrics: SearchMetrics {
                posting_relations_total: postings
                    .iter()
                    .map(|posting| posting.relations.len())
                    .sum(),
                posting_relations_decoded: postings
                    .iter()
                    .map(|posting| posting.relations.len())
                    .sum(),
                posting_groups_total: postings.iter().map(|posting| posting.groups.len()).sum(),
                centers_evaluated: terminal_count as usize,
                ..SearchMetrics::default()
            },
        },
        touched,
        all: activations,
    }
}

fn activation_equal(left: ForwardActivation, right: ForwardActivation) -> bool {
    left.mass == right.mass
        && left.hits == right.hits
        && left.surface_hits == right.surface_hits
        && left.keyboard_hits == right.keyboard_hits
}

fn retained_equal(left: &[(u32, ForwardActivation)], right: &[(u32, ForwardActivation)]) -> bool {
    left.len() == right.len()
        && left
            .iter()
            .zip(right)
            .all(|((left_id, left), (right_id, right))| {
                left_id == right_id && activation_equal(*left, *right)
            })
}

fn activation_field_mismatches(
    left: &[(u32, ForwardActivation)],
    right: &[(u32, ForwardActivation)],
) -> usize {
    let mut left_index = 0;
    let mut right_index = 0;
    let mut mismatches = 0;
    while left_index < left.len() || right_index < right.len() {
        match (left.get(left_index), right.get(right_index)) {
            (Some((left_id, left_activation)), Some((right_id, right_activation)))
                if left_id == right_id =>
            {
                mismatches += usize::from(!activation_equal(*left_activation, *right_activation));
                left_index += 1;
                right_index += 1;
            }
            (Some((left_id, _)), Some((right_id, _))) if left_id < right_id => {
                mismatches += 1;
                left_index += 1;
            }
            (Some(_), Some(_)) => {
                mismatches += 1;
                right_index += 1;
            }
            (Some(_), None) => {
                mismatches += left.len() - left_index;
                break;
            }
            (None, Some(_)) => {
                mismatches += right.len() - right_index;
                break;
            }
            (None, None) => break,
        }
    }
    mismatches
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
    dense_activation_field_mismatches: usize,
    zero_mass_semantic_losses: usize,
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
    evidence_union_us: Vec<u64>,
    wand_us: Vec<u64>,
    epoch_us: Vec<u64>,
    epoch_accumulation_us: Vec<u64>,
    epoch_readout_us: Vec<u64>,
    epoch_partition_operations: u64,
    epoch_touched_centers: u64,
    epoch_positive_centers: u64,
    epoch_zero_mass_full_scans: u64,
    epoch_wraps: u64,
    epoch_resident_bytes_max: u64,
    impact_preparation_us: Vec<u64>,
    impact_search_us: Vec<u64>,
    impact_accumulation_us: Vec<u64>,
    impact_closure_scan_us: Vec<u64>,
    impact_exact_replay_us: Vec<u64>,
    impact_exact_readout_us: Vec<u64>,
    impact_certified_unseen_thresholds: Vec<u64>,
    impact_certified_partial_betas: Vec<u64>,
    impact_query_postings: u64,
    impact_cells_total: u64,
    impact_cells_consumed: u64,
    impact_equality_layers_consumed: u64,
    impact_relation_events_total: u64,
    impact_relation_events_consumed: u64,
    impact_unique_centers_touched: u64,
    impact_threshold_checks: u64,
    impact_uncertain_closure_size: u64,
    impact_exact_replay_posting_probes: u64,
    impact_exact_replay_relation_hits: u64,
    impact_largest_cell_relations: u64,
    impact_largest_equality_layer_cells: u64,
    impact_largest_equality_layer_relations: u64,
    impact_delta_varint_bytes: u64,
    impact_packed_projection_bytes: u64,
    impact_prepared_resident_payload_bytes: u64,
    impact_scratch_resident_bytes_max: u64,
    impact_relation_accounting_losses: u64,
    impact_ordering_violations: u64,
    impact_full_exhaustions: u64,
    impact_zero_mass_full_scans: u64,
    impact_epoch_wraps: u64,
    modal_queries: Vec<ModalQueryMetrics>,
    subtree_queries: Vec<DecoderSubtreeQueryMetrics>,
}

impl ClassMetrics {
    fn finish(mut self) -> serde_json::Value {
        self.latencies_us.sort_unstable();
        self.typed_traversal_us.sort_unstable();
        self.evidence_union_us.sort_unstable();
        self.wand_us.sort_unstable();
        self.epoch_us.sort_unstable();
        self.epoch_accumulation_us.sort_unstable();
        self.epoch_readout_us.sort_unstable();
        self.impact_preparation_us.sort_unstable();
        self.impact_search_us.sort_unstable();
        self.impact_accumulation_us.sort_unstable();
        self.impact_closure_scan_us.sort_unstable();
        self.impact_exact_replay_us.sort_unstable();
        self.impact_exact_readout_us.sort_unstable();
        self.impact_certified_unseen_thresholds.sort_unstable();
        self.impact_certified_partial_betas.sort_unstable();
        let posting_work = serde_json::json!({
            "skipped_relations": self.skipped_relations,
            "centers_evaluated": self.centers_evaluated,
            "posting_iterators": self.posting_iterators,
            "candidates_scored": self.candidates_scored,
            "scheduler_iterations": self.scheduler_iterations,
            "posting_seeks": self.posting_seeks,
        });
        let typed_work = serde_json::json!({
            "typed_terminal_ids": self.typed_terminal_ids,
            "typed_states_expanded": self.typed_states_expanded,
            "typed_queue_peak": self.typed_queue_peak,
            "typed_terminal_events": self.typed_terminal_events,
            "merged_terminal_ids": self.merged_terminal_ids,
            "typed_only_terminal_ids": self.typed_only_terminal_ids,
            "posting_only_terminal_ids": self.posting_only_terminal_ids,
            "dual_source_terminal_ids": self.dual_source_terminal_ids,
        });
        let epoch_work = serde_json::json!({
            "partition_operations": self.epoch_partition_operations,
            "touched_centers": self.epoch_touched_centers,
            "positive_centers": self.epoch_positive_centers,
            "epoch_wraps": self.epoch_wraps,
        });
        let impact_work = serde_json::json!({
            "query_postings": self.impact_query_postings,
            "cells_total": self.impact_cells_total,
            "cells_consumed": self.impact_cells_consumed,
            "equality_layers_consumed": self.impact_equality_layers_consumed,
            "relation_events_total": self.impact_relation_events_total,
            "relation_events_consumed": self.impact_relation_events_consumed,
            "unique_centers_touched": self.impact_unique_centers_touched,
            "threshold_checks": self.impact_threshold_checks,
            "certified_unseen_threshold_max": self.impact_certified_unseen_thresholds.last().copied().unwrap_or_default(),
            "certified_partial_beta_min": self.impact_certified_partial_betas.first().copied().unwrap_or_default(),
            "uncertain_closure_size": self.impact_uncertain_closure_size,
            "exact_replay_posting_probes": self.impact_exact_replay_posting_probes,
            "exact_replay_relation_hits": self.impact_exact_replay_relation_hits,
            "largest_cell_relations": self.impact_largest_cell_relations,
            "largest_equality_layer_cells": self.impact_largest_equality_layer_cells,
            "largest_equality_layer_relations": self.impact_largest_equality_layer_relations,
            "full_exhaustions": self.impact_full_exhaustions,
            "zero_mass_full_scans": self.impact_zero_mass_full_scans,
            "epoch_wraps": self.impact_epoch_wraps,
        });
        let impact_representation_screen = serde_json::json!({
            "query_local_delta_varint_bytes": self.impact_delta_varint_bytes,
            "query_local_packed_projection_bytes": self.impact_packed_projection_bytes,
            "query_local_resident_payload_bytes": self.impact_prepared_resident_payload_bytes,
            "scratch_resident_bytes_max": self.impact_scratch_resident_bytes_max,
            "relation_accounting_losses": self.impact_relation_accounting_losses,
            "ordering_violations": self.impact_ordering_violations,
            "scope": "selected_query_postings_only_not_full_package_projection",
        });
        let modal_projection = summarize_modal_queries(&self.modal_queries);
        let subtree_projection = summarize_subtree_queries(&self.subtree_queries);
        let timing = serde_json::json!({
            "typed_traversal_us_p50": percentile(&self.typed_traversal_us, 50),
            "typed_traversal_us_p95": percentile(&self.typed_traversal_us, 95),
            "typed_traversal_us_p99": percentile(&self.typed_traversal_us, 99),
            "evidence_union_us_p50": percentile(&self.evidence_union_us, 50),
            "evidence_union_us_p95": percentile(&self.evidence_union_us, 95),
            "evidence_union_us_p99": percentile(&self.evidence_union_us, 99),
            "wand_us_p50": percentile(&self.wand_us, 50),
            "wand_us_p95": percentile(&self.wand_us, 95),
            "wand_us_p99": percentile(&self.wand_us, 99),
            "epoch_us_p50": percentile(&self.epoch_us, 50),
            "epoch_us_p95": percentile(&self.epoch_us, 95),
            "epoch_us_p99": percentile(&self.epoch_us, 99),
            "epoch_accumulation_us_p50": percentile(&self.epoch_accumulation_us, 50),
            "epoch_accumulation_us_p95": percentile(&self.epoch_accumulation_us, 95),
            "epoch_accumulation_us_p99": percentile(&self.epoch_accumulation_us, 99),
            "epoch_readout_us_p50": percentile(&self.epoch_readout_us, 50),
            "epoch_readout_us_p95": percentile(&self.epoch_readout_us, 95),
            "epoch_readout_us_p99": percentile(&self.epoch_readout_us, 99),
            "impact_preparation_us_p50": percentile(&self.impact_preparation_us, 50),
            "impact_preparation_us_p95": percentile(&self.impact_preparation_us, 95),
            "impact_preparation_us_p99": percentile(&self.impact_preparation_us, 99),
            "impact_search_us_p50": percentile(&self.impact_search_us, 50),
            "impact_search_us_p95": percentile(&self.impact_search_us, 95),
            "impact_search_us_p99": percentile(&self.impact_search_us, 99),
            "impact_accumulation_us_p99": percentile(&self.impact_accumulation_us, 99),
            "impact_closure_scan_us_p99": percentile(&self.impact_closure_scan_us, 99),
            "impact_exact_replay_us_p99": percentile(&self.impact_exact_replay_us, 99),
            "impact_exact_readout_us_p99": percentile(&self.impact_exact_readout_us, 99),
            "complete_us_p50": percentile(&self.latencies_us, 50),
            "complete_us_p95": percentile(&self.latencies_us, 95),
            "complete_us_p99": percentile(&self.latencies_us, 99),
        });
        serde_json::json!({
            "cases": self.cases,
            "closure_parity": self.closure_parity,
            "dense_activation_field_mismatches": self.dense_activation_field_mismatches,
            "zero_mass_semantic_losses": self.zero_mass_semantic_losses,
            "target_in_posting_closure": self.target_in_posting_closure,
            "tie_boundary_losses": self.tie_boundary_losses,
            "upper_bound_violations": self.upper_bound_violations,
            "decoded_relations": self.decoded_relations,
            "total_relations": self.total_relations,
            "typed_target_retained": self.typed_target_retained,
            "typed_union_schedule_parity": self.typed_union_schedule_parity,
            "merged_target_retained": self.merged_target_retained,
            "epoch_zero_mass_full_scans": self.epoch_zero_mass_full_scans,
            "epoch_resident_bytes_max": self.epoch_resident_bytes_max,
            "latency_us_p99": percentile(&self.latencies_us, 99),
            "posting_work": posting_work,
            "typed_work": typed_work,
            "epoch_work": epoch_work,
            "impact_work": impact_work,
            "impact_representation_screen": impact_representation_screen,
            "modal_projection": modal_projection,
            "subtree_projection": subtree_projection,
            "timing": timing,
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
    posting_search: &str,
) -> io::Result<serde_json::Value> {
    let started = Instant::now();
    let search_mode = PostingSearchMode::parse(posting_search)?;
    if search_mode != PostingSearchMode::Wand && group_relations != 0 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "--posting-group-relations is valid only with --posting-search wand",
        ));
    }
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
    let (posting_index, forward_relation_count, modal_projection, subtree_projection) =
        match search_mode {
            PostingSearchMode::Wand => {
                let index = if group_relations == 0 {
                    PostingBoundIndex::build_global(&memory)
                } else {
                    PostingBoundIndex::build_interval(&memory, group_relations)
                }
                .map_err(io::Error::other)?;
                let relation_count = index.relation_count;
                (Some(index), relation_count, None, None)
            }
            PostingSearchMode::Epoch | PostingSearchMode::Impact => (
                None,
                verify_complete_forward_field(&memory).map_err(io::Error::other)?,
                None,
                None,
            ),
            PostingSearchMode::Modal => {
                let projection = ModalResidualProjection::build(&memory, package_path)
                    .map_err(io::Error::other)?;
                let relation_count = projection.package.original_relation_events;
                (None, relation_count, Some(projection), None)
            }
            PostingSearchMode::Subtree => {
                let projection = DecoderSubtreeCoverProjection::build(&memory, package_path)
                    .map_err(io::Error::other)?;
                let relation_count = projection.package.original_relation_events;
                (None, relation_count, None, Some(projection))
            }
        };
    let index_ms = index_started.elapsed().as_millis();
    let decoder_index_started = Instant::now();
    let decoder_index = ForwardDecoderIndex::build(&memory.package).map_err(io::Error::other)?;
    let decoder_index_ms = decoder_index_started.elapsed().as_millis();
    let cases = prepare_fixed_heldout_cases(&words, heldout_per_class, 0)?;
    let mut classes = BTreeMap::<&'static str, ClassMetrics>::new();
    let mut epoch_accumulator = (search_mode == PostingSearchMode::Epoch)
        .then(|| ShardedEpochAccumulator::new(memory.package.terminal_count(), terminal_shards));
    let mut impact_search = (search_mode == PostingSearchMode::Impact)
        .then(|| ImpactThresholdSearch::new(memory.package.terminal_count()));

    for case in &cases {
        let postings = query_postings(&memory, posting_index.as_ref(), &case.surface)
            .map_err(io::Error::other)?;
        let impact_prepared = (search_mode == PostingSearchMode::Impact)
            .then(|| PreparedImpactQuery::build(&postings))
            .transpose()
            .map_err(io::Error::other)?;
        let impact_preparation_metrics = impact_prepared.as_ref().map(|prepared| prepared.metrics);
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
        let search_started = Instant::now();
        let (
            mut bounded,
            epoch_metrics,
            impact_metrics,
            impact_bounds,
            modal_metrics,
            modal_touched,
            subtree_metrics,
            subtree_touched,
        ) = match search_mode {
            PostingSearchMode::Wand => (
                wand_posting_closure(
                    &postings,
                    memory.package.terminal_count(),
                    requested_k,
                    terminal_shards,
                    None,
                ),
                None,
                None,
                None,
                None,
                None,
                None,
                None,
            ),
            PostingSearchMode::Epoch => {
                let result = epoch_accumulator
                    .as_mut()
                    .expect("epoch search mode owns one persistent accumulator")
                    .search(&postings, requested_k);
                (
                    result.closure,
                    Some(result.metrics),
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                )
            }
            PostingSearchMode::Impact => {
                let result = impact_search
                    .as_mut()
                    .expect("impact mode owns one persistent search scratch")
                    .search(
                        impact_prepared
                            .as_ref()
                            .expect("impact mode owns prepared query cells"),
                        requested_k,
                    )
                    .map_err(io::Error::other)?;
                (
                    result.closure,
                    None,
                    Some(result.metrics),
                    Some((result.touched_upper_bounds, result.unseen_mass_upper)),
                    None,
                    None,
                    None,
                    None,
                )
            }
            PostingSearchMode::Modal => {
                let result = modal_projection
                    .as_ref()
                    .expect("modal mode owns one immutable full-field profile")
                    .project_query(&postings, requested_k, dense.closure.beta_k, &dense.all)
                    .map_err(io::Error::other)?;
                let mut closure = result.exact.closure;
                closure.metrics.posting_relations_total = result.metrics.original_relation_events;
                closure.metrics.posting_relations_decoded = result.metrics.residual_events;
                closure.metrics.posting_iterators = result.metrics.query_postings;
                closure.metrics.centers_evaluated = memory.package.terminal_count() as usize;
                (
                    closure,
                    None,
                    None,
                    None,
                    Some(result.metrics),
                    Some(result.exact.touched),
                    None,
                    None,
                )
            }
            PostingSearchMode::Subtree => {
                let result = subtree_projection
                    .as_ref()
                    .expect("subtree mode owns one immutable exact cover")
                    .project_query(&postings, requested_k, dense.closure.beta_k, &dense.all)
                    .map_err(io::Error::other)?;
                let mut closure = result.exact.closure;
                closure.metrics.posting_relations_total = result.metrics.original_relation_events;
                closure.metrics.posting_relations_decoded = result.metrics.cover_token_events;
                closure.metrics.posting_iterators = result.metrics.query_postings;
                closure.metrics.centers_evaluated = result.metrics.symbolic_cohort_records;
                (
                    closure,
                    None,
                    None,
                    None,
                    None,
                    None,
                    Some(result.metrics),
                    Some(result.exact.touched),
                )
            }
        };
        let search_us = search_started
            .elapsed()
            .as_micros()
            .min(u128::from(u64::MAX)) as u64;
        let bounded_ids = bounded
            .retained
            .iter()
            .map(|(terminal_id, _)| *terminal_id)
            .collect::<Vec<_>>();
        let union_started = Instant::now();
        let posting_first = merge_terminal_evidence(&bounded_ids, &typed.terminal_ids, false);
        let typed_first = merge_terminal_evidence(&bounded_ids, &typed.terminal_ids, true);
        let evidence_union_us = union_started
            .elapsed()
            .as_micros()
            .min(u128::from(u64::MAX)) as u64;
        let union_schedule_parity = posting_first == typed_first;
        let latency_us = case_started.elapsed().as_micros().min(u128::from(u64::MAX)) as u64;
        let dense_ids = dense
            .closure
            .retained
            .iter()
            .map(|(terminal_id, _)| *terminal_id)
            .collect::<Vec<_>>();
        if let Some((touched_upper_bounds, unseen_mass_upper)) = impact_bounds.as_ref() {
            bounded.metrics.upper_bound_violations = residual_upper_bound_violations(
                &dense.touched,
                touched_upper_bounds,
                *unseen_mass_upper,
            );
        }
        let tie_losses = dense_ids
            .iter()
            .filter(|terminal_id| !bounded_ids.binary_search(terminal_id).is_ok())
            .count();
        let field_mismatches = subtree_touched
            .as_ref()
            .map(|touched| activation_field_mismatches(&dense.touched, touched))
            .or_else(|| {
                modal_touched
                    .as_ref()
                    .map(|touched| activation_field_mismatches(&dense.touched, touched))
            })
            .or_else(|| {
                epoch_accumulator.as_ref().map(|accumulator| {
                    activation_field_mismatches(&dense.touched, &accumulator.touched_activations())
                })
            })
            .unwrap_or_default();
        let parity = dense.closure.beta_k == bounded.beta_k
            && retained_equal(&dense.closure.retained, &bounded.retained)
            && field_mismatches == 0
            && bounded.metrics.upper_bound_violations == 0;
        let zero_mass_semantic_loss = usize::from(dense.closure.beta_k == 0 && !parity);
        let class = classes.entry(case.class).or_default();
        class.cases += 1;
        class.closure_parity += usize::from(parity);
        class.dense_activation_field_mismatches += field_mismatches;
        class.zero_mass_semantic_losses += zero_mass_semantic_loss;
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
        class.evidence_union_us.push(evidence_union_us);
        match search_mode {
            PostingSearchMode::Wand => class.wand_us.push(search_us),
            PostingSearchMode::Epoch => class.epoch_us.push(search_us),
            PostingSearchMode::Impact => class.impact_search_us.push(search_us),
            PostingSearchMode::Modal | PostingSearchMode::Subtree => {}
        }
        if let Some(metrics) = modal_metrics {
            class.modal_queries.push(metrics);
        }
        if let Some(metrics) = subtree_metrics {
            class.subtree_queries.push(metrics);
        }
        if let Some(EpochSearchMetrics {
            partition_operations,
            touched_centers,
            positive_centers,
            zero_mass_full_scans,
            epoch_wraps,
            resident_bytes,
            accumulation_us,
            readout_us,
            ..
        }) = epoch_metrics
        {
            class.epoch_partition_operations = class
                .epoch_partition_operations
                .saturating_add(partition_operations as u64);
            class.epoch_touched_centers = class
                .epoch_touched_centers
                .saturating_add(touched_centers as u64);
            class.epoch_positive_centers = class
                .epoch_positive_centers
                .saturating_add(positive_centers as u64);
            class.epoch_zero_mass_full_scans = class
                .epoch_zero_mass_full_scans
                .saturating_add(zero_mass_full_scans as u64);
            class.epoch_wraps = class.epoch_wraps.saturating_add(epoch_wraps as u64);
            class.epoch_resident_bytes_max =
                class.epoch_resident_bytes_max.max(resident_bytes as u64);
            class.epoch_accumulation_us.push(accumulation_us);
            class.epoch_readout_us.push(readout_us);
        }
        if let Some(ImpactPreparationMetrics {
            preparation_us,
            postings,
            largest_cell_relations,
            delta_varint_bytes,
            packed_projection_bytes,
            resident_payload_bytes,
            relation_accounting_losses,
            ordering_violations,
            ..
        }) = impact_preparation_metrics
        {
            class.impact_preparation_us.push(preparation_us);
            class.impact_query_postings =
                class.impact_query_postings.saturating_add(postings as u64);
            class.impact_largest_cell_relations = class
                .impact_largest_cell_relations
                .max(largest_cell_relations as u64);
            class.impact_delta_varint_bytes = class
                .impact_delta_varint_bytes
                .saturating_add(delta_varint_bytes as u64);
            class.impact_packed_projection_bytes = class
                .impact_packed_projection_bytes
                .saturating_add(packed_projection_bytes as u64);
            class.impact_prepared_resident_payload_bytes = class
                .impact_prepared_resident_payload_bytes
                .max(resident_payload_bytes as u64);
            class.impact_relation_accounting_losses = class
                .impact_relation_accounting_losses
                .saturating_add(relation_accounting_losses as u64);
            class.impact_ordering_violations = class
                .impact_ordering_violations
                .saturating_add(ordering_violations as u64);
        }
        if let Some(ImpactSearchMetrics {
            cells_total,
            cells_consumed,
            equality_layers_consumed,
            relation_events_total,
            relation_events_consumed,
            unique_centers_touched,
            threshold_checks,
            certified_unseen_threshold,
            certified_partial_beta,
            uncertain_closure_size,
            exact_replay_posting_probes,
            exact_replay_relation_hits,
            largest_equality_layer_cells,
            largest_equality_layer_relations,
            full_exhaustions,
            zero_mass_full_scans,
            epoch_wraps,
            scratch_resident_bytes,
            accumulation_us,
            closure_scan_us,
            exact_replay_us,
            exact_readout_us,
            ..
        }) = impact_metrics
        {
            class.impact_cells_total = class.impact_cells_total.saturating_add(cells_total as u64);
            class.impact_cells_consumed = class
                .impact_cells_consumed
                .saturating_add(cells_consumed as u64);
            class.impact_equality_layers_consumed = class
                .impact_equality_layers_consumed
                .saturating_add(equality_layers_consumed as u64);
            class.impact_relation_events_total = class
                .impact_relation_events_total
                .saturating_add(relation_events_total as u64);
            class.impact_relation_events_consumed = class
                .impact_relation_events_consumed
                .saturating_add(relation_events_consumed as u64);
            class.impact_unique_centers_touched = class
                .impact_unique_centers_touched
                .saturating_add(unique_centers_touched as u64);
            class.impact_threshold_checks = class
                .impact_threshold_checks
                .saturating_add(threshold_checks as u64);
            class
                .impact_certified_unseen_thresholds
                .push(certified_unseen_threshold);
            class
                .impact_certified_partial_betas
                .push(certified_partial_beta);
            class.impact_uncertain_closure_size = class
                .impact_uncertain_closure_size
                .saturating_add(uncertain_closure_size as u64);
            class.impact_exact_replay_posting_probes = class
                .impact_exact_replay_posting_probes
                .saturating_add(exact_replay_posting_probes as u64);
            class.impact_exact_replay_relation_hits = class
                .impact_exact_replay_relation_hits
                .saturating_add(exact_replay_relation_hits as u64);
            class.impact_largest_equality_layer_cells = class
                .impact_largest_equality_layer_cells
                .max(largest_equality_layer_cells as u64);
            class.impact_largest_equality_layer_relations = class
                .impact_largest_equality_layer_relations
                .max(largest_equality_layer_relations as u64);
            class.impact_full_exhaustions = class
                .impact_full_exhaustions
                .saturating_add(full_exhaustions as u64);
            class.impact_zero_mass_full_scans = class
                .impact_zero_mass_full_scans
                .saturating_add(zero_mass_full_scans as u64);
            class.impact_epoch_wraps = class.impact_epoch_wraps.saturating_add(epoch_wraps as u64);
            class.impact_scratch_resident_bytes_max = class
                .impact_scratch_resident_bytes_max
                .max(scratch_resident_bytes as u64);
            class.impact_accumulation_us.push(accumulation_us);
            class.impact_closure_scan_us.push(closure_scan_us);
            class.impact_exact_replay_us.push(exact_replay_us);
            class.impact_exact_readout_us.push(exact_readout_us);
        }
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
    let dense_activation_field_mismatches = classes
        .values()
        .filter_map(|metrics| metrics.get("dense_activation_field_mismatches")?.as_u64())
        .sum::<u64>();
    let zero_mass_semantic_losses = classes
        .values()
        .filter_map(|metrics| metrics.get("zero_mass_semantic_losses")?.as_u64())
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
    let impact_relation_accounting_losses = classes
        .values()
        .filter_map(|metrics| {
            metrics
                .pointer("/impact_representation_screen/relation_accounting_losses")?
                .as_u64()
        })
        .sum::<u64>();
    let impact_ordering_violations = classes
        .values()
        .filter_map(|metrics| {
            metrics
                .pointer("/impact_representation_screen/ordering_violations")?
                .as_u64()
        })
        .sum::<u64>();
    let modal_cases = classes
        .values()
        .filter_map(|metrics| metrics.pointer("/modal_projection/cases")?.as_u64())
        .sum::<u64>();
    let modal_reconstruction_mismatches = classes
        .values()
        .filter_map(|metrics| {
            metrics
                .pointer("/modal_projection/reconstruction_field_mismatches")?
                .as_u64()
        })
        .sum::<u64>();
    let modal_kth_mismatches = classes
        .values()
        .filter_map(|metrics| {
            metrics
                .pointer("/modal_projection/kth_or_equality_mismatches")?
                .as_u64()
        })
        .sum::<u64>();
    let modal_upper_bound_violations = classes
        .values()
        .filter_map(|metrics| {
            metrics
                .pointer("/modal_projection/untouched_upper_bound_violations")?
                .as_u64()
        })
        .sum::<u64>();
    let modal_default_cohort_resolved = classes
        .values()
        .filter_map(|metrics| {
            metrics
                .pointer("/modal_projection/default_cohort_resolved")?
                .as_u64()
        })
        .sum::<u64>();
    let modal_oracle_threshold_certified = classes
        .values()
        .filter_map(|metrics| {
            metrics
                .pointer("/modal_projection/oracle_threshold_certified")?
                .as_u64()
        })
        .sum::<u64>();
    let maximum_modal_greedy_events = classes
        .values()
        .filter_map(|metrics| {
            metrics
                .pointer("/modal_projection/oracle_greedy_events_max")?
                .as_u64()
        })
        .max()
        .unwrap_or_default();
    let maximum_modal_equality_layer = classes
        .values()
        .filter_map(|metrics| {
            metrics
                .pointer("/modal_projection/largest_consumed_equality_layer")?
                .as_u64()
        })
        .max()
        .unwrap_or_default();
    let maximum_modal_fractional_lower_bound = classes
        .values()
        .filter_map(|metrics| {
            metrics
                .pointer("/modal_projection/fractional_event_lower_bound_max")?
                .as_u64()
        })
        .max()
        .unwrap_or_default();
    let maximum_modal_signed_residual_events = classes
        .values()
        .filter_map(|metrics| {
            metrics
                .pointer("/modal_projection/residual_events_max")?
                .as_u64()
        })
        .max()
        .unwrap_or_default();
    let subtree_cases = classes
        .values()
        .filter_map(|metrics| metrics.pointer("/subtree_projection/cases")?.as_u64())
        .sum::<u64>();
    let subtree_reconstruction_mismatches = classes
        .values()
        .filter_map(|metrics| {
            metrics
                .pointer("/subtree_projection/reconstruction_field_mismatches")?
                .as_u64()
        })
        .sum::<u64>();
    let subtree_histogram_mismatches = classes
        .values()
        .filter_map(|metrics| {
            metrics
                .pointer("/subtree_projection/activation_histogram_mismatches")?
                .as_u64()
        })
        .sum::<u64>();
    let subtree_kth_mismatches = classes
        .values()
        .filter_map(|metrics| {
            metrics
                .pointer("/subtree_projection/kth_or_equality_mismatches")?
                .as_u64()
        })
        .sum::<u64>();
    let subtree_retained_id_differences = classes
        .values()
        .filter_map(|metrics| {
            metrics
                .pointer("/subtree_projection/retained_id_symmetric_difference")?
                .as_u64()
        })
        .sum::<u64>();
    let maximum_subtree_cover_tokens = classes
        .values()
        .filter_map(|metrics| {
            metrics
                .pointer("/subtree_projection/cover_token_events_max")?
                .as_u64()
        })
        .max()
        .unwrap_or_default();
    let maximum_subtree_closure_nodes = classes
        .values()
        .filter_map(|metrics| {
            metrics
                .pointer("/subtree_projection/activated_ancestor_closure_nodes_max")?
                .as_u64()
        })
        .max()
        .unwrap_or_default();
    let maximum_subtree_symbolic_cohorts = classes
        .values()
        .filter_map(|metrics| {
            metrics
                .pointer("/subtree_projection/symbolic_cohort_records_max")?
                .as_u64()
        })
        .max()
        .unwrap_or_default();
    let maximum_subtree_retained_id_expansions = classes
        .values()
        .filter_map(|metrics| {
            metrics
                .pointer("/subtree_projection/retained_terminal_id_expansions_max")?
                .as_u64()
        })
        .max()
        .unwrap_or_default();
    let maximum_subtree_combined_work = classes
        .values()
        .filter_map(|metrics| {
            metrics
                .pointer("/subtree_projection/projected_sparse_work_units_max")?
                .as_u64()
        })
        .max()
        .unwrap_or_default();
    let modal_package = modal_projection
        .as_ref()
        .map(|projection| &projection.package);
    let modal_package_correct = modal_package.is_none_or(|package| {
        package.state_partition_omissions == 0
            && package.state_partition_duplicates == 0
            && package.residual_bound_violations == 0
            && package.residual_events <= package.original_relation_events
    });
    let modal_correctness = search_mode != PostingSearchMode::Modal
        || (modal_cases == cases.len() as u64
            && modal_reconstruction_mismatches == 0
            && modal_kth_mismatches == 0
            && modal_upper_bound_violations == 0
            && modal_package_correct);
    let subtree_package = subtree_projection
        .as_ref()
        .map(|projection| &projection.package);
    let subtree_package_correct = subtree_package.is_none_or(|package| {
        package.state_partition_omissions == 0
            && package.state_partition_duplicates == 0
            && package.cover_overlap_violations == 0
            && package.event_bound_violations == 0
            && package.represented_relation_events == package.original_relation_events
            && package.cover_token_events <= package.original_relation_events
    });
    let subtree_correctness = search_mode != PostingSearchMode::Subtree
        || (subtree_cases == cases.len() as u64
            && subtree_reconstruction_mismatches == 0
            && subtree_histogram_mismatches == 0
            && subtree_kth_mismatches == 0
            && subtree_retained_id_differences == 0
            && subtree_package_correct);
    let complete_relation_decode = search_mode != PostingSearchMode::Epoch
        || classes.values().all(|metrics| {
            metrics
                .get("decoded_relations")
                .and_then(serde_json::Value::as_u64)
                == metrics
                    .get("total_relations")
                    .and_then(serde_json::Value::as_u64)
        });
    let correctness_passed = package_unchanged
        && closure_parity
        && upper_bound_violations == 0
        && dense_activation_field_mismatches == 0
        && zero_mass_semantic_losses == 0
        && complete_relation_decode
        && tie_boundary_losses == 0
        && typed_terminal_losses == 0
        && typed_union_schedule_parity
        && merged_target_losses == 0
        && impact_relation_accounting_losses == 0
        && impact_ordering_violations == 0
        && modal_correctness
        && subtree_correctness;
    let maximum_complete_latency_us = classes
        .values()
        .filter_map(|metrics| metrics.get("latency_us_p99")?.as_u64())
        .max()
        .unwrap_or_default();
    let package_bytes = std::fs::metadata(package_path)?.len();
    let epoch_accumulator_resident_bytes_max = classes
        .values()
        .filter_map(|metrics| metrics.get("epoch_resident_bytes_max")?.as_u64())
        .max()
        .unwrap_or_default();
    let epoch_zero_mass_full_scans = classes
        .values()
        .filter_map(|metrics| metrics.get("epoch_zero_mass_full_scans")?.as_u64())
        .sum::<u64>();
    let impact_zero_mass_full_scans = classes
        .values()
        .filter_map(|metrics| {
            metrics
                .pointer("/impact_work/zero_mass_full_scans")?
                .as_u64()
        })
        .sum::<u64>();
    let maximum_impact_search_us = classes
        .values()
        .filter_map(|metrics| metrics.pointer("/timing/impact_search_us_p99")?.as_u64())
        .max()
        .unwrap_or_default();
    let latency_tested = !matches!(
        search_mode,
        PostingSearchMode::Modal | PostingSearchMode::Subtree
    );
    let latency_feasible = latency_tested && maximum_complete_latency_us <= HOT_LATENCY_LIMIT_US;
    let package_feasible = package_bytes <= PACKAGE_LIMIT_BYTES;
    let modal_projected_package_bytes = modal_package
        .map(|package| package.projected_package_bytes)
        .unwrap_or_default();
    let modal_projected_package_feasible = search_mode != PostingSearchMode::Modal
        || modal_projected_package_bytes <= PACKAGE_LIMIT_BYTES;
    let subtree_projected_package_bytes = subtree_package
        .map(|package| package.projected_package_bytes)
        .unwrap_or_default();
    let subtree_projected_package_feasible = search_mode != PostingSearchMode::Subtree
        || subtree_projected_package_bytes <= PACKAGE_LIMIT_BYTES;
    let epoch_memory_feasible = search_mode != PostingSearchMode::Epoch
        || epoch_accumulator_resident_bytes_max <= EPOCH_ACCUMULATOR_LIMIT_BYTES as u64;
    let zero_mass_scan_feasible = match search_mode {
        PostingSearchMode::Epoch => epoch_zero_mass_full_scans == 0,
        PostingSearchMode::Impact => impact_zero_mass_full_scans == 0,
        PostingSearchMode::Wand => true,
        PostingSearchMode::Modal => true,
        PostingSearchMode::Subtree => true,
    };
    let impact_intrinsic_feasible = search_mode != PostingSearchMode::Impact
        || (maximum_impact_search_us <= IMPACT_INTRINSIC_LIMIT_US
            && latency_feasible
            && zero_mass_scan_feasible);
    let physical_representation_tested = search_mode != PostingSearchMode::Impact;
    let feasibility_passed = latency_feasible
        && package_feasible
        && epoch_memory_feasible
        && zero_mass_scan_feasible
        && impact_intrinsic_feasible
        && physical_representation_tested
        && modal_projected_package_feasible
        && subtree_projected_package_feasible;
    let passed = correctness_passed && feasibility_passed;
    let modal_denominator_complete = modal_cases == cases.len() as u64;
    let modal_default_gate = modal_default_cohort_resolved == modal_cases;
    let modal_oracle_gate = modal_oracle_threshold_certified == modal_cases;
    let modal_greedy_gate = maximum_modal_greedy_events <= PROJECTION_EVENT_LIMIT as u64;
    let modal_equality_gate = maximum_modal_equality_layer <= PROJECTION_EVENT_LIMIT as u64;
    let modal_scan_gate = maximum_modal_signed_residual_events <= PROJECTION_EVENT_LIMIT as u64;
    let subtree_denominator_complete = subtree_cases == cases.len() as u64;
    let subtree_cover_gate = maximum_subtree_cover_tokens <= SUBTREE_PROJECTION_WORK_LIMIT as u64;
    let subtree_closure_gate =
        maximum_subtree_closure_nodes <= SUBTREE_PROJECTION_WORK_LIMIT as u64;
    let subtree_cohort_gate =
        maximum_subtree_symbolic_cohorts <= SUBTREE_PROJECTION_WORK_LIMIT as u64;
    let subtree_retained_id_expansion_gate =
        maximum_subtree_retained_id_expansions <= SUBTREE_PROJECTION_WORK_LIMIT as u64;
    let subtree_combined_work_gate =
        maximum_subtree_combined_work <= SUBTREE_PROJECTION_WORK_LIMIT as u64;
    let report_verdict = if search_mode == PostingSearchMode::Subtree {
        if !correctness_passed || !subtree_denominator_complete {
            "FAIL_CORRECTNESS"
        } else if !subtree_projected_package_feasible {
            "REJECT_REPRESENTATION"
        } else if !subtree_cover_gate {
            "REJECT_DECODER_SUBTREE_COVER"
        } else if !subtree_closure_gate
            || !subtree_cohort_gate
            || !subtree_retained_id_expansion_gate
            || !subtree_combined_work_gate
        {
            "REJECT_SPARSE_READOUT_TOPOLOGY"
        } else {
            "ADMIT_PHASE8H_1_PREFLIGHT"
        }
    } else if search_mode == PostingSearchMode::Modal {
        if !correctness_passed || !modal_denominator_complete {
            "FAIL_CORRECTNESS"
        } else if !modal_projected_package_feasible {
            "REJECT_REPRESENTATION"
        } else if !modal_default_gate || !modal_oracle_gate {
            "DEFAULT_COHORT_OPEN"
        } else if maximum_modal_fractional_lower_bound > PROJECTION_EVENT_LIMIT as u64 {
            "REJECT_HEAD_CERTIFICATE"
        } else if !modal_greedy_gate || !modal_equality_gate {
            "SCHEDULER_WATCH"
        } else if !modal_scan_gate {
            "DISCOVERY_PASS_READOUT_OPEN"
        } else {
            "ADMIT_PHASE8G_1A_PREFLIGHT"
        }
    } else if search_mode == PostingSearchMode::Impact {
        if !correctness_passed {
            "FAIL_CORRECTNESS"
        } else if !impact_intrinsic_feasible {
            "REJECT_INTRINSIC"
        } else {
            "ADMIT_PHYSICAL_REPRESENTATION_SCREEN"
        }
    } else if passed {
        "PASS"
    } else {
        "FAIL"
    };
    let feasibility_verdict = if search_mode == PostingSearchMode::Subtree {
        match report_verdict {
            "ADMIT_PHASE8H_1_PREFLIGHT" => "P0_PASS_B0_UNTESTED",
            "REJECT_REPRESENTATION" => "P0_PACKAGE_FAIL",
            "REJECT_DECODER_SUBTREE_COVER" => "P0_COVER_WORK_FAIL",
            "REJECT_SPARSE_READOUT_TOPOLOGY" => "P0_READOUT_WORK_FAIL",
            _ => "P0_CORRECTNESS_FAIL",
        }
    } else if search_mode == PostingSearchMode::Modal {
        match report_verdict {
            "ADMIT_PHASE8G_1A_PREFLIGHT" => "P0_AND_SCAN_PASS_B0_UNTESTED",
            "DISCOVERY_PASS_READOUT_OPEN" => "P0_DISCOVERY_PASS_READOUT_OPEN",
            "SCHEDULER_WATCH" => "P0_SCHEDULER_WATCH",
            "REJECT_HEAD_CERTIFICATE" => "P0_HEAD_CERTIFICATE_FAIL",
            "REJECT_REPRESENTATION" => "P0_PACKAGE_FAIL",
            "DEFAULT_COHORT_OPEN" => "P0_DEFAULT_COHORT_OPEN",
            _ => "P0_CORRECTNESS_FAIL",
        }
    } else if search_mode == PostingSearchMode::Impact {
        if impact_intrinsic_feasible {
            "B0_PASS_B1_UNTESTED"
        } else {
            "B0_FAIL"
        }
    } else if feasibility_passed {
        "PASS"
    } else {
        "FAIL"
    };
    let subtree_gates = serde_json::json!({
        "projection_work_limit": SUBTREE_PROJECTION_WORK_LIMIT,
        "fixed_denominator_complete": subtree_denominator_complete,
        "maximum_cover_tokens": maximum_subtree_cover_tokens,
        "maximum_activated_closure_nodes": maximum_subtree_closure_nodes,
        "maximum_symbolic_cohorts": maximum_subtree_symbolic_cohorts,
        "maximum_retained_terminal_id_expansions": maximum_subtree_retained_id_expansions,
        "maximum_combined_work": maximum_subtree_combined_work,
        "cover_gate": subtree_cover_gate,
        "closure_gate": subtree_closure_gate,
        "cohort_gate": subtree_cohort_gate,
        "retained_id_expansion_gate": subtree_retained_id_expansion_gate,
        "combined_work_gate": subtree_combined_work_gate,
        "projected_package_bytes": subtree_projected_package_bytes,
        "projected_package_feasible": subtree_projected_package_feasible,
    });
    let gates = serde_json::json!({
        "correctness_verdict": if correctness_passed { "PASS" } else { "FAIL" },
        "feasibility_verdict": feasibility_verdict,
        "package_limit_bytes": PACKAGE_LIMIT_BYTES,
        "current_package_feasible": package_feasible,
        "modal_projection_event_limit": PROJECTION_EVENT_LIMIT,
        "modal_fixed_denominator_complete": modal_denominator_complete,
        "modal_default_cohort_resolved": modal_default_cohort_resolved,
        "modal_oracle_threshold_certified": modal_oracle_threshold_certified,
        "maximum_modal_greedy_events": maximum_modal_greedy_events,
        "maximum_modal_consumed_equality_layer": maximum_modal_equality_layer,
        "maximum_modal_fractional_event_lower_bound": maximum_modal_fractional_lower_bound,
        "maximum_modal_signed_residual_events": maximum_modal_signed_residual_events,
        "modal_greedy_gate": modal_greedy_gate,
        "modal_equality_gate": modal_equality_gate,
        "modal_scan_gate": modal_scan_gate,
        "modal_projected_package_bytes": modal_projected_package_bytes,
        "modal_projected_package_feasible": modal_projected_package_feasible,
        "subtree": subtree_gates,
        "impact_physical_representation_tested": physical_representation_tested,
        "epoch_accumulator_limit_bytes": EPOCH_ACCUMULATOR_LIMIT_BYTES,
        "epoch_accumulator_resident_bytes_max": epoch_accumulator_resident_bytes_max,
        "epoch_memory_feasible": epoch_memory_feasible,
        "epoch_zero_mass_full_scans": epoch_zero_mass_full_scans,
        "impact_zero_mass_full_scans": impact_zero_mass_full_scans,
        "zero_mass_scan_feasible": zero_mass_scan_feasible,
        "maximum_intrinsic_complete_excluding_impact_preparation_us": maximum_complete_latency_us,
        "hot_latency_limit_us": HOT_LATENCY_LIMIT_US,
        "latency_tested": latency_tested,
        "latency_feasible": latency_feasible,
        "maximum_impact_search_plus_replay_us": maximum_impact_search_us,
        "impact_intrinsic_limit_us": IMPACT_INTRINSIC_LIMIT_US,
        "impact_intrinsic_feasible": impact_intrinsic_feasible,
    });

    let artifact = serde_json::json!({
        "corpus": corpus_path,
        "package": package_path,
        "package_sha256_before": package_sha256_before,
        "package_sha256_after": package_sha256_after,
        "package_bytes_unchanged": package_unchanged,
        "package_bytes": package_bytes,
        "centers": memory.package.terminal_count(),
        "atoms": memory.package.atoms.len(),
        "forward_relations": forward_relation_count,
        "complete_forward_relations_verified": forward_relation_count,
        "posting_verification_scope": "all_forward_relations_once_before_heldout_search",
    });
    let configuration = serde_json::json!({
        "group_relations": posting_index.as_ref().map_or(0, |index| index.group_relations),
        "posting_groups": posting_index.as_ref().map_or(0, |index| index.groups.len()),
        "directory_resident_bytes": posting_index.as_ref().map_or(0, PostingBoundIndex::resident_bytes),
        "directory_packed_projection_bytes": posting_index.as_ref().map_or(0, PostingBoundIndex::packed_projection_bytes),
        "global_envelope_projection_bytes": posting_index.as_ref().map_or(0, PostingBoundIndex::global_envelope_projection_bytes),
        "search": search_mode.report_name(),
        "terminal_shards": terminal_shards,
        "requested_k": requested_k,
        "heldout_per_class": heldout_per_class,
        "heldout_cases": cases.len(),
        "fixed_case_manifest_sha256": fixed_case_manifest_sha256(&cases, requested_k),
    });
    let proof_integrity = serde_json::json!({
        "closure_parity_complete": closure_parity,
        "dense_activation_field_mismatches": dense_activation_field_mismatches,
        "zero_mass_semantic_losses": zero_mass_semantic_losses,
        "complete_relation_decode": complete_relation_decode,
        "upper_bound_violations": upper_bound_violations,
        "impact_relation_accounting_losses": impact_relation_accounting_losses,
        "impact_ordering_violations": impact_ordering_violations,
        "modal_reconstruction_field_mismatches": modal_reconstruction_mismatches,
        "modal_kth_or_equality_mismatches": modal_kth_mismatches,
        "modal_untouched_upper_bound_violations": modal_upper_bound_violations,
        "modal_package_accounting_complete": modal_package_correct,
        "subtree_reconstruction_field_mismatches": subtree_reconstruction_mismatches,
        "subtree_activation_histogram_mismatches": subtree_histogram_mismatches,
        "subtree_kth_or_equality_mismatches": subtree_kth_mismatches,
        "subtree_retained_id_symmetric_difference": subtree_retained_id_differences,
        "subtree_package_accounting_complete": subtree_package_correct,
        "tie_boundary_losses": tie_boundary_losses,
        "typed_terminal_losses": typed_terminal_losses,
        "typed_union_schedule_parity": typed_union_schedule_parity,
        "merged_target_losses": merged_target_losses,
        "generated_target_strings": 0,
        "queue_truncations": 0,
        "runtime_authority_changed": false,
        "package_format_changed": false,
        "modal_projection_runtime_reachable": false,
        "subtree_projection_runtime_reachable": false,
        "dense_oracle_used_by_runtime": false,
    });
    let setup_timing = serde_json::json!({
        "posting_validation_or_index_build_ms": index_ms,
        "decoder_index_build_ms": decoder_index_ms,
        "wall_ms": started.elapsed().as_millis(),
    });

    Ok(serde_json::json!({
        "schema": "lay.l11.posting-search-phase8-proof.v8",
        "verdict": report_verdict,
        "gates": gates,
        "artifact": artifact,
        "configuration": configuration,
        "proof_integrity": proof_integrity,
        "classes": classes,
        "modal_field_projection": modal_package
            .map(serde_json::to_value)
            .transpose()
            .map_err(io::Error::other)?,
        "subtree_field_projection": subtree_package
            .map(serde_json::to_value)
            .transpose()
            .map_err(io::Error::other)?,
        "setup_timing": setup_timing,
    }))
}

fn file_sha256(path: &Path) -> io::Result<String> {
    Ok(format!("{:x}", Sha256::digest(std::fs::read(path)?)))
}

fn fixed_case_manifest_sha256(cases: &[FixedHeldoutCase], requested_k: usize) -> String {
    let mut hasher = Sha256::new();
    hasher.update(b"lay.l11.fixed-heldout-manifest.v1");
    hasher.update((requested_k as u64).to_le_bytes());
    for case in cases {
        hasher.update(case.class.as_bytes());
        hasher.update([0]);
        hasher.update(case.terminal_id.to_le_bytes());
        hasher.update(case.surface.as_bytes());
        hasher.update([0xff]);
    }
    format!("{:x}", hasher.finalize())
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
    fn complete_forward_posting_batch_matches_scalar_access() {
        let memory = memory();
        let atom_ids = (0..memory.package.atoms.len().min(64) as u32).collect::<Vec<_>>();
        let batch = memory
            .complete_forward_couplings_batch(&atom_ids)
            .expect("decode complete posting batch");
        assert_eq!(batch.len(), atom_ids.len());
        for (atom_id, batched) in atom_ids.into_iter().zip(batch) {
            let scalar = memory
                .complete_forward_couplings(atom_id)
                .expect("decode scalar complete posting");
            assert_eq!(batched.as_ref(), scalar.as_ref(), "atom {atom_id}");
        }
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
        let mut postings = query_postings(&memory, Some(&index), "frmo").unwrap();
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
            let postings = query_postings(&memory, Some(&index), surface).unwrap();
            let dense = exact_posting_closure(&postings, memory.package.terminal_count(), 3);
            let wand = wand_posting_closure(
                &postings,
                memory.package.terminal_count(),
                3,
                WAND_TERMINAL_SHARDS,
                None,
            );
            assert_eq!(wand.beta_k, dense.closure.beta_k, "{surface}");
            assert!(
                retained_equal(&wand.retained, &dense.closure.retained),
                "{surface}"
            );
            assert_eq!(wand.metrics.upper_bound_violations, 0, "{surface}");
        }
    }

    #[test]
    fn epoch_shards_match_dense_field_and_complete_relation_decode() {
        let memory = memory();
        let terminal_count = memory.package.terminal_count();
        for surface in ["form", "frmo", "far", "фрма", "форма"] {
            let postings = query_postings(&memory, None, surface).unwrap();
            let expected_relations = postings
                .iter()
                .map(|posting| posting.relations.len())
                .sum::<usize>();
            let dense = exact_posting_closure(&postings, terminal_count, 3);
            for shard_count in 1..=terminal_count as usize {
                let mut accumulator = ShardedEpochAccumulator::new(terminal_count, shard_count);
                let epoch = accumulator.search(&postings, 3);
                assert_eq!(epoch.closure.beta_k, dense.closure.beta_k, "{surface}");
                assert!(
                    retained_equal(&epoch.closure.retained, &dense.closure.retained),
                    "{surface} shards={shard_count}"
                );
                assert_eq!(
                    activation_field_mismatches(&dense.touched, &accumulator.touched_activations()),
                    0,
                    "{surface} shards={shard_count}"
                );
                assert_eq!(epoch.metrics.posting_relations_total, expected_relations);
                assert_eq!(epoch.metrics.posting_relations_decoded, expected_relations);
            }
        }
    }

    #[test]
    fn epoch_repeated_query_does_not_revive_stale_activation() {
        let memory = memory();
        let terminal_count = memory.package.terminal_count();
        let first = query_postings(&memory, None, "form").unwrap();
        let second = query_postings(&memory, None, "фрма").unwrap();
        let dense = exact_posting_closure(&second, terminal_count, 3);
        let mut accumulator = ShardedEpochAccumulator::new(terminal_count, 3);
        accumulator.search(&first, 3);
        let actual = accumulator.search(&second, 3);
        assert_eq!(actual.closure.beta_k, dense.closure.beta_k);
        assert!(retained_equal(
            &actual.closure.retained,
            &dense.closure.retained
        ));
        assert_eq!(
            activation_field_mismatches(&dense.touched, &accumulator.touched_activations()),
            0
        );
    }

    #[test]
    fn epoch_wrap_clears_state_and_preserves_parity() {
        let memory = memory();
        let terminal_count = memory.package.terminal_count();
        let first = query_postings(&memory, None, "form").unwrap();
        let second = query_postings(&memory, None, "форма").unwrap();
        let dense = exact_posting_closure(&second, terminal_count, 3);
        let mut accumulator = ShardedEpochAccumulator::new(terminal_count, 4);
        accumulator.search(&first, 3);
        accumulator.force_epoch_wrap();
        let actual = accumulator.search(&second, 3);
        assert_eq!(actual.metrics.epoch_wraps, 4);
        assert_eq!(actual.closure.beta_k, dense.closure.beta_k);
        assert!(retained_equal(
            &actual.closure.retained,
            &dense.closure.retained
        ));
        assert_eq!(
            activation_field_mismatches(&dense.touched, &accumulator.touched_activations()),
            0
        );
    }

    #[test]
    fn epoch_zero_mass_retains_all_terminals() {
        let terminal_count = 8;
        let mut accumulator = ShardedEpochAccumulator::new(terminal_count, 3);
        let actual = accumulator.search(&[], 3);
        assert_eq!(actual.closure.beta_k, 0);
        assert_eq!(actual.closure.retained.len(), terminal_count as usize);
        assert!(actual
            .closure
            .retained
            .iter()
            .all(|(_, activation)| activation.hits == 0 && activation.mass == 0));
        assert_eq!(actual.metrics.zero_mass_full_scans, 1);
        assert_eq!(actual.metrics.posting_relations_decoded, 0);
    }

    #[test]
    fn epoch_shard_ranges_partition_terminal_domain_once() {
        for terminal_count in 1..=17_u32 {
            for requested_shards in 1..=terminal_count as usize + 3 {
                let accumulator = ShardedEpochAccumulator::new(terminal_count, requested_shards);
                let ranges = accumulator.shard_ranges();
                assert_eq!(ranges.first().map(|range| range.0), Some(0));
                assert_eq!(ranges.last().map(|range| range.1), Some(terminal_count));
                assert!(ranges.iter().all(|(low, high)| low < high));
                assert!(ranges.windows(2).all(|window| window[0].1 == window[1].0));
            }
        }
    }

    #[test]
    fn epoch_posting_order_cannot_change_closure_or_field() {
        let memory = memory();
        let terminal_count = memory.package.terminal_count();
        let mut postings = query_postings(&memory, None, "frmo").unwrap();
        let mut forward_accumulator = ShardedEpochAccumulator::new(terminal_count, 3);
        let forward = forward_accumulator.search(&postings, 3);
        let forward_field = forward_accumulator.touched_activations();
        postings.reverse();
        let mut reverse_accumulator = ShardedEpochAccumulator::new(terminal_count, 3);
        let reverse = reverse_accumulator.search(&postings, 3);
        assert_eq!(forward.closure.beta_k, reverse.closure.beta_k);
        assert!(retained_equal(
            &forward.closure.retained,
            &reverse.closure.retained
        ));
        assert_eq!(
            activation_field_mismatches(&forward_field, &reverse_accumulator.touched_activations()),
            0
        );
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

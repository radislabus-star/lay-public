//! Proof-only impact-ordered feasibility screen over complete postings.

use std::cmp::Reverse;
use std::collections::{BTreeMap, BTreeSet, BinaryHeap};
use std::mem::size_of;
use std::sync::Arc;
use std::time::Instant;

use super::{exact_contribution, is_keyboard_channel, PostingClosure, QueryPosting, SearchMetrics};
use crate::nanda_wave::lexical_grokking::model::WaveCoupling;
use crate::nanda_wave::lexical_grokking::runtime::{ForwardActivation, ObservedAtom};

const PACKED_CELL_HEADER_BYTES: usize = 10;

#[derive(Clone, Debug)]
struct ImpactCell {
    strength: u8,
    position_mode: u8,
    contribution: u64,
    terminal_ids: Vec<u32>,
}

#[derive(Debug)]
struct ImpactPosting {
    atom: ObservedAtom,
    relations: Arc<[WaveCoupling]>,
    cells: Vec<ImpactCell>,
    keyboard: bool,
}

#[derive(Clone, Copy, Debug, Default)]
pub(super) struct ImpactPreparationMetrics {
    pub(super) preparation_us: u64,
    pub(super) postings: usize,
    pub(super) cells: usize,
    pub(super) relations: usize,
    pub(super) largest_cell_relations: usize,
    pub(super) raw_terminal_id_bytes: usize,
    pub(super) delta_varint_bytes: usize,
    pub(super) packed_projection_bytes: usize,
    pub(super) resident_payload_bytes: usize,
    pub(super) relation_accounting_losses: usize,
    pub(super) ordering_violations: usize,
}

#[derive(Debug)]
pub(super) struct PreparedImpactQuery {
    postings: Vec<ImpactPosting>,
    pub(super) metrics: ImpactPreparationMetrics,
}

impl PreparedImpactQuery {
    pub(super) fn build(postings: &[QueryPosting<'_>]) -> Result<Self, String> {
        let started = Instant::now();
        let mut prepared = Vec::with_capacity(postings.len());
        let mut observed_atom_ids = BTreeSet::new();
        let mut metrics = ImpactPreparationMetrics {
            postings: postings.len(),
            ..ImpactPreparationMetrics::default()
        };

        for posting in postings {
            if !observed_atom_ids.insert(posting.atom_id) {
                return Err("impact query contains a duplicate atom posting".to_string());
            }
            let mut groups = BTreeMap::<(u8, u8), Vec<u32>>::new();
            let mut previous_terminal = None;
            for relation in posting.relations.iter().copied() {
                if previous_terminal.is_some_and(|previous| previous >= relation.peer_id) {
                    return Err(
                        "impact source posting terminal IDs must be strictly increasing"
                            .to_string(),
                    );
                }
                previous_terminal = Some(relation.peer_id);
                groups
                    .entry((relation.strength, relation.position_mode))
                    .or_default()
                    .push(relation.peer_id);
            }

            let mut cells = groups
                .into_iter()
                .map(|((strength, position_mode), terminal_ids)| {
                    let relation = WaveCoupling {
                        strength,
                        position_mode,
                        ..WaveCoupling::default()
                    };
                    ImpactCell {
                        strength,
                        position_mode,
                        contribution: exact_contribution(posting.atom, relation),
                        terminal_ids,
                    }
                })
                .collect::<Vec<_>>();
            cells.sort_unstable_by(|left, right| {
                right
                    .contribution
                    .cmp(&left.contribution)
                    .then_with(|| right.strength.cmp(&left.strength))
                    .then_with(|| left.position_mode.cmp(&right.position_mode))
            });

            metrics.ordering_violations += cells
                .windows(2)
                .filter(|window| window[0].contribution < window[1].contribution)
                .count();
            let cell_relations = cells
                .iter()
                .map(|cell| cell.terminal_ids.len())
                .sum::<usize>();
            metrics.relation_accounting_losses +=
                usize::from(cell_relations != posting.relations.len());
            metrics.cells = metrics.cells.saturating_add(cells.len());
            metrics.relations = metrics.relations.saturating_add(cell_relations);
            metrics.largest_cell_relations = metrics.largest_cell_relations.max(
                cells
                    .iter()
                    .map(|cell| cell.terminal_ids.len())
                    .max()
                    .unwrap_or_default(),
            );
            metrics.raw_terminal_id_bytes = metrics
                .raw_terminal_id_bytes
                .saturating_add(cell_relations.saturating_mul(size_of::<u32>()));
            metrics.delta_varint_bytes = metrics.delta_varint_bytes.saturating_add(
                cells
                    .iter()
                    .map(|cell| delta_varint_bytes(&cell.terminal_ids))
                    .sum::<usize>(),
            );
            metrics.resident_payload_bytes = metrics
                .resident_payload_bytes
                .saturating_add(cells.capacity().saturating_mul(size_of::<ImpactCell>()))
                .saturating_add(
                    cells
                        .iter()
                        .map(|cell| {
                            cell.terminal_ids
                                .capacity()
                                .saturating_mul(size_of::<u32>())
                        })
                        .sum::<usize>(),
                );
            prepared.push(ImpactPosting {
                atom: posting.atom,
                relations: Arc::clone(&posting.relations),
                cells,
                keyboard: is_keyboard_channel(posting.atom.channel),
            });
        }

        metrics.resident_payload_bytes = metrics.resident_payload_bytes.saturating_add(
            prepared
                .capacity()
                .saturating_mul(size_of::<ImpactPosting>()),
        );
        metrics.packed_projection_bytes = metrics
            .delta_varint_bytes
            .saturating_add(metrics.cells.saturating_mul(PACKED_CELL_HEADER_BYTES))
            .saturating_add(
                metrics
                    .postings
                    .saturating_add(1)
                    .saturating_mul(size_of::<u32>()),
            );
        metrics.preparation_us = elapsed_us(started);

        if metrics.relation_accounting_losses != 0 || metrics.ordering_violations != 0 {
            return Err("impact preparation violated relation accounting or ordering".to_string());
        }
        Ok(Self {
            postings: prepared,
            metrics,
        })
    }

    fn activation_for_terminal(&self, terminal_id: u32) -> ForwardActivation {
        let mut activation = ForwardActivation::default();
        for posting in &self.postings {
            let Ok(relation_index) = posting
                .relations
                .binary_search_by_key(&terminal_id, |relation| relation.peer_id)
            else {
                continue;
            };
            let relation = posting.relations[relation_index];
            activation.mass = activation
                .mass
                .saturating_add(exact_contribution(posting.atom, relation));
            activation.hits = activation.hits.saturating_add(1);
            if posting.keyboard {
                activation.keyboard_hits = activation.keyboard_hits.saturating_add(1);
            } else {
                activation.surface_hits = activation.surface_hits.saturating_add(1);
            }
        }
        activation
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub(super) struct ImpactSearchMetrics {
    pub(super) cells_total: usize,
    pub(super) cells_consumed: usize,
    pub(super) equality_layers_consumed: usize,
    pub(super) relation_events_total: usize,
    pub(super) relation_events_consumed: usize,
    pub(super) unique_centers_touched: usize,
    pub(super) threshold_checks: usize,
    pub(super) certified_unseen_threshold: u64,
    pub(super) certified_partial_beta: u64,
    pub(super) uncertain_closure_size: usize,
    pub(super) exact_replay_posting_probes: usize,
    pub(super) exact_replay_relation_hits: usize,
    pub(super) largest_equality_layer_cells: usize,
    pub(super) largest_equality_layer_relations: usize,
    pub(super) full_exhaustions: usize,
    pub(super) zero_mass_full_scans: usize,
    pub(super) epoch_wraps: usize,
    pub(super) scratch_resident_bytes: usize,
    pub(super) accumulation_us: u64,
    pub(super) closure_scan_us: u64,
    pub(super) exact_replay_us: u64,
    pub(super) exact_readout_us: u64,
    pub(super) search_us: u64,
}

#[derive(Clone, Copy, Debug)]
pub(super) struct ImpactUpperBound {
    pub(super) terminal_id: u32,
    pub(super) mass_upper: u64,
}

pub(super) struct ImpactSearchResult {
    pub(super) closure: PostingClosure,
    pub(super) metrics: ImpactSearchMetrics,
    pub(super) touched_upper_bounds: Vec<ImpactUpperBound>,
    pub(super) unseen_mass_upper: u64,
}

pub(super) struct ImpactThresholdSearch {
    terminal_count: usize,
    epoch: u32,
    center_epochs: Vec<u32>,
    top_epochs: Vec<u32>,
    activations: Vec<ForwardActivation>,
    seen_masks: Vec<u64>,
    seen_words_per_center: usize,
    touched: Vec<u32>,
}

impl ImpactThresholdSearch {
    pub(super) fn new(terminal_count: u32) -> Self {
        let terminal_count = terminal_count as usize;
        Self {
            terminal_count,
            epoch: 0,
            center_epochs: vec![0; terminal_count],
            top_epochs: vec![0; terminal_count],
            activations: vec![ForwardActivation::default(); terminal_count],
            seen_masks: Vec::new(),
            seen_words_per_center: 0,
            touched: Vec::new(),
        }
    }

    pub(super) fn search(
        &mut self,
        query: &PreparedImpactQuery,
        requested_k: usize,
    ) -> Result<ImpactSearchResult, String> {
        let search_started = Instant::now();
        let words_per_center = query.postings.len().div_ceil(u64::BITS as usize);
        let epoch_wraps = self.begin_query(words_per_center);
        let effective_k = requested_k.max(1).min(self.terminal_count.max(1));
        let mut top_k = PartialTopK::new(effective_k);
        let mut cursors = vec![0_usize; query.postings.len()];
        let mut heads = query
            .postings
            .iter()
            .map(|posting| posting.cells.first().map_or(0, |cell| cell.contribution))
            .collect::<Vec<_>>();
        let mut unseen_mass_upper = heads.iter().copied().fold(0_u64, u64::saturating_add);
        let mut queue = heads
            .iter()
            .copied()
            .enumerate()
            .filter_map(|(posting_index, contribution)| {
                (contribution != 0).then_some((contribution, Reverse(posting_index)))
            })
            .collect::<BinaryHeap<_>>();
        let relation_events_total = query
            .postings
            .iter()
            .map(|posting| {
                posting
                    .cells
                    .iter()
                    .map(|cell| cell.terminal_ids.len())
                    .sum::<usize>()
            })
            .sum::<usize>();
        let cells_total = query
            .postings
            .iter()
            .map(|posting| posting.cells.len())
            .sum::<usize>();
        let mut metrics = ImpactSearchMetrics {
            cells_total,
            relation_events_total,
            epoch_wraps,
            ..ImpactSearchMetrics::default()
        };
        let mut certified_beta = 0_u64;

        while let Some((layer_contribution, _)) = queue.peek().copied() {
            let mut layer_cells = 0_usize;
            let mut layer_relations = 0_usize;
            while queue
                .peek()
                .is_some_and(|(contribution, _)| *contribution == layer_contribution)
            {
                let (_, Reverse(posting_index)) = queue.pop().expect("impact queue head exists");
                let posting = &query.postings[posting_index];
                let cell = posting
                    .cells
                    .get(cursors[posting_index])
                    .ok_or_else(|| "impact cursor exceeds posting cells".to_string())?;
                if cell.contribution != layer_contribution {
                    return Err("impact queue contribution differs from cell head".to_string());
                }
                layer_cells = layer_cells.saturating_add(1);
                layer_relations = layer_relations.saturating_add(cell.terminal_ids.len());
                for terminal_id in cell.terminal_ids.iter().copied() {
                    let terminal_index = terminal_id as usize;
                    if terminal_index >= self.terminal_count {
                        return Err("impact terminal exceeds center count".to_string());
                    }
                    self.touch(terminal_index);
                    let old_mass = self.activations[terminal_index].mass;
                    let new_mass = {
                        let activation = &mut self.activations[terminal_index];
                        activation.mass = activation.mass.saturating_add(cell.contribution);
                        activation.hits = activation.hits.saturating_add(1);
                        if posting.keyboard {
                            activation.keyboard_hits = activation.keyboard_hits.saturating_add(1);
                        } else {
                            activation.surface_hits = activation.surface_hits.saturating_add(1);
                        }
                        activation.mass
                    };
                    self.mark_posting_seen(terminal_index, posting_index);
                    top_k.update(
                        terminal_id,
                        old_mass,
                        new_mass,
                        &mut self.top_epochs,
                        self.epoch,
                    );
                }
                cursors[posting_index] = cursors[posting_index].saturating_add(1);
                let next = posting
                    .cells
                    .get(cursors[posting_index])
                    .map_or(0, |next| next.contribution);
                unseen_mass_upper = unseen_mass_upper
                    .saturating_sub(heads[posting_index])
                    .saturating_add(next);
                heads[posting_index] = next;
                if next != 0 {
                    queue.push((next, Reverse(posting_index)));
                }
            }

            metrics.cells_consumed = metrics.cells_consumed.saturating_add(layer_cells);
            metrics.relation_events_consumed = metrics
                .relation_events_consumed
                .saturating_add(layer_relations);
            metrics.equality_layers_consumed = metrics.equality_layers_consumed.saturating_add(1);
            metrics.largest_equality_layer_cells =
                metrics.largest_equality_layer_cells.max(layer_cells);
            metrics.largest_equality_layer_relations = metrics
                .largest_equality_layer_relations
                .max(layer_relations);
            metrics.threshold_checks = metrics.threshold_checks.saturating_add(1);
            certified_beta = top_k.beta();
            if threshold_is_certified(&top_k, unseen_mass_upper) {
                break;
            }
        }
        metrics.accumulation_us = elapsed_us(search_started);
        metrics.unique_centers_touched = self.touched.len();
        metrics.certified_unseen_threshold = unseen_mass_upper;
        metrics.certified_partial_beta = certified_beta;
        metrics.full_exhaustions = usize::from(queue.is_empty());

        if certified_beta == 0 {
            metrics.zero_mass_full_scans = 1;
            let replay_started = Instant::now();
            let mut retained = Vec::with_capacity(self.terminal_count);
            for terminal_id in 0..self.terminal_count as u32 {
                let activation = query.activation_for_terminal(terminal_id);
                metrics.exact_replay_relation_hits = metrics
                    .exact_replay_relation_hits
                    .saturating_add(activation.hits as usize);
                retained.push((terminal_id, activation));
            }
            metrics.exact_replay_posting_probes =
                self.terminal_count.saturating_mul(query.postings.len());
            metrics.exact_replay_us = elapsed_us(replay_started);
            metrics.uncertain_closure_size = retained.len();
            metrics.scratch_resident_bytes = self.resident_bytes();
            metrics.search_us = elapsed_us(search_started);
            return Ok(ImpactSearchResult {
                closure: PostingClosure {
                    beta_k: 0,
                    retained,
                    metrics: SearchMetrics {
                        posting_relations_total: relation_events_total,
                        posting_relations_decoded: metrics.relation_events_consumed,
                        posting_groups_total: cells_total,
                        centers_evaluated: self.terminal_count,
                        posting_relations_skipped: relation_events_total
                            .saturating_sub(metrics.relation_events_consumed),
                        posting_iterators: query.postings.len(),
                        candidates_scored: self.terminal_count,
                        scheduler_iterations: metrics.cells_consumed,
                        ..SearchMetrics::default()
                    },
                },
                metrics,
                touched_upper_bounds: Vec::new(),
                unseen_mass_upper,
            });
        }

        let closure_started = Instant::now();
        let mut uncertain_ids = Vec::new();
        let mut touched_upper_bounds = Vec::with_capacity(self.touched.len());
        for terminal_id in self.touched.iter().copied() {
            let terminal_index = terminal_id as usize;
            let seen_head_sum = self.seen_head_sum(terminal_index, &heads);
            let mass_upper = self.activations[terminal_index]
                .mass
                .saturating_add(unseen_mass_upper.saturating_sub(seen_head_sum));
            touched_upper_bounds.push(ImpactUpperBound {
                terminal_id,
                mass_upper,
            });
            if mass_upper >= certified_beta {
                uncertain_ids.push(terminal_id);
            }
        }
        touched_upper_bounds.sort_unstable_by_key(|bound| bound.terminal_id);
        uncertain_ids.sort_unstable();
        metrics.closure_scan_us = elapsed_us(closure_started);
        metrics.uncertain_closure_size = uncertain_ids.len();

        if uncertain_ids.len() < effective_k {
            return Err(format!(
                "impact uncertain closure is smaller than K: {} < {effective_k}",
                uncertain_ids.len()
            ));
        }
        let replay_started = Instant::now();
        let mut replayed = Vec::with_capacity(uncertain_ids.len());
        for terminal_id in uncertain_ids {
            let activation = query.activation_for_terminal(terminal_id);
            metrics.exact_replay_relation_hits = metrics
                .exact_replay_relation_hits
                .saturating_add(activation.hits as usize);
            replayed.push((terminal_id, activation));
        }
        metrics.exact_replay_posting_probes = metrics
            .uncertain_closure_size
            .saturating_mul(query.postings.len());
        metrics.exact_replay_us = elapsed_us(replay_started);

        let readout_started = Instant::now();
        let mut masses = replayed
            .iter()
            .map(|(_, activation)| activation.mass)
            .collect::<Vec<_>>();
        masses.select_nth_unstable_by(effective_k - 1, |left, right| right.cmp(left));
        let exact_beta = masses[effective_k - 1];
        let retained = replayed
            .into_iter()
            .filter(|(_, activation)| activation.mass >= exact_beta)
            .collect::<Vec<_>>();
        metrics.exact_readout_us = elapsed_us(readout_started);
        metrics.scratch_resident_bytes = self.resident_bytes();
        metrics.search_us = elapsed_us(search_started);

        Ok(ImpactSearchResult {
            closure: PostingClosure {
                beta_k: exact_beta,
                retained,
                metrics: SearchMetrics {
                    posting_relations_total: relation_events_total,
                    posting_relations_decoded: metrics.relation_events_consumed,
                    posting_groups_total: cells_total,
                    centers_evaluated: metrics.unique_centers_touched,
                    posting_relations_skipped: relation_events_total
                        .saturating_sub(metrics.relation_events_consumed),
                    posting_iterators: query.postings.len(),
                    candidates_scored: metrics.uncertain_closure_size,
                    scheduler_iterations: metrics.cells_consumed,
                    ..SearchMetrics::default()
                },
            },
            metrics,
            touched_upper_bounds,
            unseen_mass_upper,
        })
    }

    fn begin_query(&mut self, seen_words_per_center: usize) -> usize {
        self.epoch = self.epoch.wrapping_add(1);
        let wrapped = self.epoch == 0;
        if wrapped {
            self.center_epochs.fill(0);
            self.top_epochs.fill(0);
            self.epoch = 1;
        }
        self.touched.clear();
        if self.seen_words_per_center != seen_words_per_center {
            self.seen_words_per_center = seen_words_per_center;
            self.seen_masks.clear();
            self.seen_masks.resize(
                self.terminal_count
                    .saturating_mul(self.seen_words_per_center),
                0,
            );
            self.center_epochs.fill(0);
            self.top_epochs.fill(0);
        }
        usize::from(wrapped)
    }

    fn touch(&mut self, terminal_index: usize) {
        if self.center_epochs[terminal_index] == self.epoch {
            return;
        }
        self.center_epochs[terminal_index] = self.epoch;
        self.activations[terminal_index] = ForwardActivation::default();
        let start = terminal_index.saturating_mul(self.seen_words_per_center);
        let end = start.saturating_add(self.seen_words_per_center);
        self.seen_masks[start..end].fill(0);
        self.touched.push(terminal_index as u32);
    }

    fn mark_posting_seen(&mut self, terminal_index: usize, posting_index: usize) {
        let word = posting_index / u64::BITS as usize;
        let bit = posting_index % u64::BITS as usize;
        let offset = terminal_index
            .saturating_mul(self.seen_words_per_center)
            .saturating_add(word);
        self.seen_masks[offset] |= 1_u64 << bit;
    }

    fn seen_head_sum(&self, terminal_index: usize, heads: &[u64]) -> u64 {
        let start = terminal_index.saturating_mul(self.seen_words_per_center);
        let mut sum = 0_u64;
        for (word_index, bits) in self.seen_masks
            [start..start.saturating_add(self.seen_words_per_center)]
            .iter()
            .copied()
            .enumerate()
        {
            let mut remaining = bits;
            while remaining != 0 {
                let bit = remaining.trailing_zeros() as usize;
                let posting_index = word_index
                    .saturating_mul(u64::BITS as usize)
                    .saturating_add(bit);
                sum = sum.saturating_add(heads.get(posting_index).copied().unwrap_or_default());
                remaining &= remaining - 1;
            }
        }
        sum
    }

    fn resident_bytes(&self) -> usize {
        self.center_epochs
            .capacity()
            .saturating_mul(size_of::<u32>())
            .saturating_add(self.top_epochs.capacity().saturating_mul(size_of::<u32>()))
            .saturating_add(
                self.activations
                    .capacity()
                    .saturating_mul(size_of::<ForwardActivation>()),
            )
            .saturating_add(self.seen_masks.capacity().saturating_mul(size_of::<u64>()))
            .saturating_add(self.touched.capacity().saturating_mul(size_of::<u32>()))
    }

    #[cfg(test)]
    pub(super) fn force_epoch_wrap(&mut self) {
        self.epoch = u32::MAX;
    }
}

struct PartialTopK {
    limit: usize,
    entries: BTreeSet<(u64, u32)>,
}

impl PartialTopK {
    fn new(limit: usize) -> Self {
        Self {
            limit,
            entries: BTreeSet::new(),
        }
    }

    fn update(
        &mut self,
        terminal_id: u32,
        old_mass: u64,
        new_mass: u64,
        top_epochs: &mut [u32],
        epoch: u32,
    ) {
        let terminal_index = terminal_id as usize;
        if top_epochs[terminal_index] == epoch {
            let removed = self.entries.remove(&(old_mass, terminal_id));
            debug_assert!(removed);
            self.entries.insert((new_mass, terminal_id));
            return;
        }
        if self.entries.len() < self.limit {
            self.entries.insert((new_mass, terminal_id));
            top_epochs[terminal_index] = epoch;
            return;
        }
        let Some(&minimum) = self.entries.first() else {
            return;
        };
        if (new_mass, terminal_id) <= minimum {
            return;
        }
        self.entries.remove(&minimum);
        top_epochs[minimum.1 as usize] = 0;
        self.entries.insert((new_mass, terminal_id));
        top_epochs[terminal_index] = epoch;
    }

    fn is_full(&self) -> bool {
        self.entries.len() == self.limit
    }

    fn beta(&self) -> u64 {
        if self.is_full() {
            self.entries.first().map_or(0, |entry| entry.0)
        } else {
            0
        }
    }
}

fn threshold_is_certified(top_k: &PartialTopK, unseen_mass_upper: u64) -> bool {
    top_k.is_full() && unseen_mass_upper < top_k.beta()
}

pub(super) fn residual_upper_bound_violations(
    dense_touched: &[(u32, ForwardActivation)],
    touched_upper_bounds: &[ImpactUpperBound],
    unseen_mass_upper: u64,
) -> usize {
    let mut bound_index = 0_usize;
    let mut violations = 0_usize;
    for (terminal_id, activation) in dense_touched.iter().copied() {
        while touched_upper_bounds
            .get(bound_index)
            .is_some_and(|bound| bound.terminal_id < terminal_id)
        {
            bound_index += 1;
        }
        let upper = touched_upper_bounds
            .get(bound_index)
            .filter(|bound| bound.terminal_id == terminal_id)
            .map_or(unseen_mass_upper, |bound| bound.mass_upper);
        violations += usize::from(activation.mass > upper);
    }
    violations
}

fn delta_varint_bytes(terminal_ids: &[u32]) -> usize {
    let mut previous = 0_u32;
    terminal_ids
        .iter()
        .copied()
        .map(|terminal_id| {
            let delta = terminal_id.saturating_sub(previous);
            previous = terminal_id;
            varint_len(delta)
        })
        .sum()
}

fn varint_len(mut value: u32) -> usize {
    let mut bytes = 1_usize;
    while value >= 0x80 {
        value >>= 7;
        bytes += 1;
    }
    bytes
}

fn elapsed_us(started: Instant) -> u64 {
    started.elapsed().as_micros().min(u128::from(u64::MAX)) as u64
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::nanda_wave::lexical_grokking::atoms::AtomChannel;
    use crate::nanda_wave::lexical_grokking::compiler::compile;
    use crate::nanda_wave::lexical_grokking::runtime::LexicalGrokkingMemory;
    use crate::nanda_wave::lexical_grokking::training_corpus::TrainingWord;
    use std::sync::Arc;

    fn posting<'a>(atom_id: u32, position: u8, relations: Vec<WaveCoupling>) -> QueryPosting<'a> {
        QueryPosting {
            atom_id,
            atom: ObservedAtom {
                position,
                weight: 3,
                channel: AtomChannel::CharacterGram,
            },
            relations: Arc::from(relations),
            groups: &[],
            global_maxima: [0; super::super::POSITION_BUCKETS],
        }
    }

    fn memory() -> LexicalGrokkingMemory {
        let words = [
            "form",
            "farm",
            "foam",
            "from",
            "frame",
            "format",
            "formal",
            "forms",
            "формат",
            "ферма",
            "форма",
            "формы",
        ]
        .into_iter()
        .enumerate()
        .map(|(terminal_id, surface)| TrainingWord {
            terminal_id: terminal_id as u32,
            surface: surface.to_string(),
            training_surfaces: Vec::new(),
        })
        .collect::<Vec<_>>();
        LexicalGrokkingMemory::from_package(compile(&words).expect("compile impact test package"))
    }

    fn assert_same_closure(left: &PostingClosure, right: &PostingClosure) {
        assert_eq!(left.beta_k, right.beta_k);
        assert!(super::super::retained_equal(
            &left.retained,
            &right.retained
        ));
    }

    #[test]
    fn impact_cells_account_for_every_relation_once() {
        let query = PreparedImpactQuery::build(&[posting(
            0,
            128,
            vec![
                WaveCoupling {
                    peer_id: 1,
                    strength: 90,
                    position_mode: 64,
                    ..WaveCoupling::default()
                },
                WaveCoupling {
                    peer_id: 3,
                    strength: 120,
                    position_mode: 128,
                    ..WaveCoupling::default()
                },
                WaveCoupling {
                    peer_id: 7,
                    strength: 90,
                    position_mode: 64,
                    ..WaveCoupling::default()
                },
            ],
        )])
        .unwrap();
        assert_eq!(query.metrics.relations, 3);
        assert_eq!(query.metrics.cells, 2);
        assert_eq!(query.metrics.relation_accounting_losses, 0);
        assert_eq!(query.metrics.ordering_violations, 0);
        assert_eq!(query.postings[0].cells[1].terminal_ids, [1, 7]);
    }

    #[test]
    fn impact_cell_heads_descend_for_every_observed_position() {
        let relations = (0..64_u32)
            .map(|terminal_id| WaveCoupling {
                peer_id: terminal_id,
                strength: ((terminal_id * 29) % 255) as u8 + 1,
                position_mode: ((terminal_id * 47) % 256) as u8,
                ..WaveCoupling::default()
            })
            .collect::<Vec<_>>();
        for position in 0..=u8::MAX {
            let query =
                PreparedImpactQuery::build(&[posting(0, position, relations.clone())]).unwrap();
            assert!(query.postings[0]
                .cells
                .windows(2)
                .all(|window| window[0].contribution >= window[1].contribution));
        }
    }

    #[test]
    fn partial_top_k_tracks_monotonic_updates_and_ties() {
        let mut top_epochs = vec![0; 6];
        let mut top = PartialTopK::new(3);
        for (terminal_id, mass) in [10, 20, 20, 5, 15, 30].into_iter().enumerate() {
            top.update(terminal_id as u32, 0, mass, &mut top_epochs, 1);
        }
        assert_eq!(top.beta(), 20);
        top.update(0, 10, 40, &mut top_epochs, 1);
        assert_eq!(top.beta(), 20);
        top.update(2, 20, 50, &mut top_epochs, 1);
        assert_eq!(top.beta(), 30);
    }

    #[test]
    fn equality_at_unseen_threshold_cannot_certify() {
        let mut top_epochs = vec![0; 3];
        let mut top = PartialTopK::new(2);
        top.update(0, 0, 40, &mut top_epochs, 1);
        top.update(1, 0, 30, &mut top_epochs, 1);

        assert!(!threshold_is_certified(&top, 30));
        assert!(threshold_is_certified(&top, 29));
    }

    #[test]
    fn impact_search_matches_dense_closure_across_k_and_posting_orders() {
        let memory = memory();
        let terminal_count = memory.package.terminal_count();

        for surface in ["form", "frmo", "форм", "форам"] {
            let base = super::super::query_postings(&memory, None, surface).unwrap();
            assert!(
                !base.is_empty(),
                "fixture must resolve postings for {surface}"
            );

            for requested_k in [1, 2, 4, terminal_count as usize] {
                let dense = super::super::exact_posting_closure(&base, terminal_count, requested_k);
                let mut reference = None;

                for schedule in 0..3 {
                    let mut postings = base.clone();
                    match schedule {
                        1 => postings.reverse(),
                        2 => {
                            let by = postings.len() / 3;
                            postings.rotate_left(by);
                        }
                        _ => {}
                    }
                    let prepared = PreparedImpactQuery::build(&postings).unwrap();
                    let mut search = ImpactThresholdSearch::new(terminal_count);
                    let result = search.search(&prepared, requested_k).unwrap();

                    assert_same_closure(&dense.closure, &result.closure);
                    assert_eq!(
                        residual_upper_bound_violations(
                            &dense.touched,
                            &result.touched_upper_bounds,
                            result.unseen_mass_upper,
                        ),
                        0
                    );
                    if let Some(reference) = reference.as_ref() {
                        assert_same_closure(reference, &result.closure);
                    } else {
                        reference = Some(result.closure);
                    }
                }
            }
        }
    }

    #[test]
    fn exact_posting_replay_preserves_all_activation_fields_and_state_resets() {
        let memory = memory();
        let terminal_count = memory.package.terminal_count();
        let postings = super::super::query_postings(&memory, None, "frmo").unwrap();
        let dense = super::super::exact_posting_closure(&postings, terminal_count, 4);
        let prepared = PreparedImpactQuery::build(&postings).unwrap();
        let mut search = ImpactThresholdSearch::new(terminal_count);

        let first = search.search(&prepared, 4).unwrap();
        assert_same_closure(&dense.closure, &first.closure);
        assert!(first
            .closure
            .retained
            .iter()
            .any(|(_, activation)| activation.surface_hits != 0));
        assert!(first
            .closure
            .retained
            .iter()
            .any(|(_, activation)| activation.keyboard_hits != 0));

        let repeated = search.search(&prepared, 4).unwrap();
        assert_same_closure(&first.closure, &repeated.closure);
        assert_eq!(repeated.metrics.epoch_wraps, 0);

        search.force_epoch_wrap();
        let wrapped = search.search(&prepared, 4).unwrap();
        assert_same_closure(&first.closure, &wrapped.closure);
        assert_eq!(wrapped.metrics.epoch_wraps, 1);

        for ((first_id, first), (dense_id, dense)) in first
            .closure
            .retained
            .iter()
            .zip(dense.closure.retained.iter())
        {
            assert_eq!(first_id, dense_id);
            assert_eq!(first.mass, dense.mass);
            assert_eq!(first.hits, dense.hits);
            assert_eq!(first.surface_hits, dense.surface_hits);
            assert_eq!(first.keyboard_hits, dense.keyboard_hits);
        }
    }
}

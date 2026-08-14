use std::collections::BTreeMap;
use std::mem::size_of;
use std::time::Instant;

use rayon::prelude::*;

use super::{
    add_relation, is_keyboard_channel, ForwardActivation, PostingClosure, QueryPosting,
    SearchMetrics,
};

#[derive(Clone, Copy, Debug, Default)]
pub(super) struct EpochSearchMetrics {
    pub(super) posting_relations_total: usize,
    pub(super) posting_relations_decoded: usize,
    pub(super) partition_operations: usize,
    pub(super) touched_centers: usize,
    pub(super) positive_centers: usize,
    pub(super) zero_mass_full_scans: usize,
    pub(super) epoch_wraps: usize,
    pub(super) resident_bytes: usize,
    pub(super) accumulation_us: u64,
    pub(super) readout_us: u64,
}

pub(super) struct EpochSearchResult {
    pub(super) closure: PostingClosure,
    pub(super) metrics: EpochSearchMetrics,
}

struct EpochShard {
    low: u32,
    high: u32,
    activations: Vec<ForwardActivation>,
    epochs: Vec<u32>,
    touched: Vec<u32>,
    epoch: u32,
}

impl EpochShard {
    fn new(low: u32, high: u32) -> Self {
        let len = (high - low) as usize;
        Self {
            low,
            high,
            activations: vec![ForwardActivation::default(); len],
            epochs: vec![0; len],
            touched: Vec::new(),
            epoch: 0,
        }
    }

    fn begin_query(&mut self) -> bool {
        self.touched.clear();
        self.epoch = self.epoch.wrapping_add(1);
        if self.epoch == 0 {
            self.epochs.fill(0);
            self.epoch = 1;
            true
        } else {
            false
        }
    }

    fn accumulate(&mut self, postings: &[QueryPosting<'_>]) -> (usize, usize, bool) {
        let wrapped = self.begin_query();
        let mut decoded = 0_usize;
        let mut partitions = 0_usize;
        for posting in postings {
            let relations = posting.relations.as_ref();
            let start = relations.partition_point(|relation| relation.peer_id < self.low);
            let end =
                start + relations[start..].partition_point(|relation| relation.peer_id < self.high);
            partitions += 2;
            let keyboard = is_keyboard_channel(posting.atom.channel);
            for relation in relations[start..end].iter().copied() {
                let local = (relation.peer_id - self.low) as usize;
                if self.epochs[local] != self.epoch {
                    self.epochs[local] = self.epoch;
                    self.activations[local] = ForwardActivation::default();
                    self.touched.push(relation.peer_id);
                }
                add_relation(
                    &mut self.activations[local],
                    posting.atom,
                    relation,
                    keyboard,
                );
                decoded += 1;
            }
        }
        (decoded, partitions, wrapped)
    }

    fn current_activation(&self, terminal_id: u32) -> ForwardActivation {
        let local = (terminal_id - self.low) as usize;
        debug_assert!(local < self.activations.len());
        debug_assert_eq!(self.epochs[local], self.epoch);
        self.activations[local]
    }

    fn resident_bytes(&self) -> usize {
        self.activations.capacity() * size_of::<ForwardActivation>()
            + self.epochs.capacity() * size_of::<u32>()
            + self.touched.capacity() * size_of::<u32>()
    }
}

pub(super) struct ShardedEpochAccumulator {
    terminal_count: u32,
    shards: Vec<EpochShard>,
}

impl ShardedEpochAccumulator {
    pub(super) fn new(terminal_count: u32, requested_shards: usize) -> Self {
        if terminal_count == 0 {
            return Self {
                terminal_count,
                shards: Vec::new(),
            };
        }
        let shard_count = requested_shards.max(1).min(terminal_count as usize);
        let shards = (0..shard_count)
            .map(|shard| {
                let low = (u64::from(terminal_count) * shard as u64 / shard_count as u64) as u32;
                let high =
                    (u64::from(terminal_count) * (shard + 1) as u64 / shard_count as u64) as u32;
                EpochShard::new(low, high)
            })
            .collect();
        Self {
            terminal_count,
            shards,
        }
    }

    pub(super) fn search(
        &mut self,
        postings: &[QueryPosting<'_>],
        requested_k: usize,
    ) -> EpochSearchResult {
        if self.terminal_count == 0 {
            return EpochSearchResult {
                closure: PostingClosure {
                    beta_k: 0,
                    retained: Vec::new(),
                    metrics: SearchMetrics::default(),
                },
                metrics: EpochSearchMetrics::default(),
            };
        }

        let accumulation_started = Instant::now();
        let shard_metrics = self
            .shards
            .par_iter_mut()
            .map(|shard| shard.accumulate(postings))
            .collect::<Vec<_>>();
        let accumulation_us = elapsed_us(accumulation_started);

        let readout_started = Instant::now();
        let touched_count = self
            .shards
            .iter()
            .map(|shard| shard.touched.len())
            .sum::<usize>();
        let mut touched = Vec::with_capacity(touched_count);
        for shard in &self.shards {
            touched.extend(
                shard
                    .touched
                    .iter()
                    .copied()
                    .map(|terminal_id| (terminal_id, shard.current_activation(terminal_id))),
            );
        }

        let effective_k = requested_k.max(1).min(self.terminal_count as usize);
        let positive_centers = touched
            .iter()
            .filter(|(_, activation)| activation.mass > 0)
            .count();
        let beta_k = if touched.len() < effective_k {
            0
        } else {
            let mut masses = touched
                .iter()
                .map(|(_, activation)| activation.mass)
                .collect::<Vec<_>>();
            masses.select_nth_unstable_by(effective_k - 1, |left, right| right.cmp(left));
            masses[effective_k - 1]
        };

        let zero_mass_full_scan = beta_k == 0;
        let mut retained = if zero_mass_full_scan {
            let touched = touched.into_iter().collect::<BTreeMap<_, _>>();
            (0..self.terminal_count)
                .map(|terminal_id| {
                    (
                        terminal_id,
                        touched.get(&terminal_id).copied().unwrap_or_default(),
                    )
                })
                .collect::<Vec<_>>()
        } else {
            touched.retain(|(_, activation)| activation.mass >= beta_k);
            touched
        };
        retained.sort_unstable_by_key(|(terminal_id, _)| *terminal_id);
        let readout_us = elapsed_us(readout_started);

        let decoded = shard_metrics.iter().map(|(decoded, _, _)| *decoded).sum();
        let partition_operations = shard_metrics
            .iter()
            .map(|(_, partitions, _)| *partitions)
            .sum();
        let epoch_wraps = shard_metrics
            .iter()
            .filter(|(_, _, wrapped)| *wrapped)
            .count();
        let relation_total = postings.iter().map(|posting| posting.relations.len()).sum();
        let resident_bytes = self.shards.iter().map(EpochShard::resident_bytes).sum();

        EpochSearchResult {
            closure: PostingClosure {
                beta_k,
                retained,
                metrics: SearchMetrics {
                    posting_relations_total: relation_total,
                    posting_relations_decoded: decoded,
                    centers_evaluated: touched_count,
                    candidates_scored: touched_count,
                    ..SearchMetrics::default()
                },
            },
            metrics: EpochSearchMetrics {
                posting_relations_total: relation_total,
                posting_relations_decoded: decoded,
                partition_operations,
                touched_centers: touched_count,
                positive_centers,
                zero_mass_full_scans: usize::from(zero_mass_full_scan),
                epoch_wraps,
                resident_bytes,
                accumulation_us,
                readout_us,
            },
        }
    }

    pub(super) fn touched_activations(&self) -> Vec<(u32, ForwardActivation)> {
        let mut touched = self
            .shards
            .iter()
            .flat_map(|shard| {
                shard
                    .touched
                    .iter()
                    .copied()
                    .map(|terminal_id| (terminal_id, shard.current_activation(terminal_id)))
            })
            .collect::<Vec<_>>();
        touched.sort_unstable_by_key(|(terminal_id, _)| *terminal_id);
        touched
    }

    #[cfg(test)]
    pub(super) fn force_epoch_wrap(&mut self) {
        for shard in &mut self.shards {
            shard.epoch = u32::MAX;
        }
    }

    #[cfg(test)]
    pub(super) fn shard_ranges(&self) -> Vec<(u32, u32)> {
        self.shards
            .iter()
            .map(|shard| (shard.low, shard.high))
            .collect()
    }
}

fn elapsed_us(started: Instant) -> u64 {
    started.elapsed().as_micros().min(u128::from(u64::MAX)) as u64
}

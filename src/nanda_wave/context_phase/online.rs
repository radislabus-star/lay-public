use std::collections::{BTreeSet, HashMap, VecDeque};
use std::io;
use std::sync::mpsc::{channel, sync_channel, Receiver, SyncSender};
use std::sync::Arc;
use std::thread::{self, JoinHandle};

use super::super::phase_field::{
    add_hashed_atom, add_phase_vector, add_rotated_vector, empty_vector, hash_text, max_coherence,
    phase_center_from_sum, phase_micro, PhaseCell, PhaseCenter,
};
use super::{
    canonical_relation_scene_wave, canonical_scene_wave, ContextCandidateProfile,
    ContextPairPhaseProfile, ContextPhaseMode, ContextPhasePackage, PairKey, SurfaceMutationField,
    TokenSemanticState, CELLS, MAX_CONTEXT_ATOMS, MAX_CONTEXT_TOKENS, MAX_EXACT_PAIR_PROFILES,
    MAX_HARD_PAIR_CENTERS_PER_BANK, MAX_PAIR_CENTERS_PER_BANK, MAX_PAIR_PROFILES,
    MAX_RELATION_PAIR_PROFILES, MAX_SIGNATURE_PROFILES,
};

const MAX_POSITIVE_CENTERS: usize = 8;
const MAX_ANTI_CENTERS: usize = 24;
const MAX_PENDING_ANTI_CENTERS: usize = 4;
const CENTER_SPLIT_COHERENCE: f32 = 0.76;
const MAX_COMPETITORS: usize = 4;
const MAX_L2_TRAINING_SURFACES: usize = 4;
const PROFILE_CALIBRATION_SAMPLES: usize = 32;
const COMPETITION_CALIBRATION_SAMPLES: usize = 2_048;
const ADMISSION_SKETCH_LANES: usize = 2;
const ADMISSION_SKETCH_WIDTH: usize = 1 << 16;
pub(super) const L2_PROBE_BATCH_FRAGMENTS: usize = 128;

#[derive(Clone, Copy, Debug)]
pub(super) struct OnlineContextPhaseConfig {
    pub(super) min_profile_support: u32,
    pub(super) signature_schema: u32,
    pub(super) max_semantic_states: usize,
    pub(super) max_profiles: usize,
    pub(super) max_positive_phase_centers: usize,
    pub(super) max_negative_phase_centers: usize,
    pub(super) max_hard_negative_phase_centers: usize,
}

impl OnlineContextPhaseConfig {
    pub(super) fn production(min_profile_support: u32) -> Self {
        Self::production_with_signature_schema(
            min_profile_support,
            super::SIGNATURE_SCHEMA_RELATION_ROLES,
        )
    }

    pub(super) fn production_with_signature_schema(
        min_profile_support: u32,
        signature_schema: u32,
    ) -> Self {
        Self {
            min_profile_support: min_profile_support.max(2),
            signature_schema,
            max_semantic_states: 32_768,
            max_profiles: 16_384,
            max_positive_phase_centers: 65_536,
            max_negative_phase_centers: 24_576,
            max_hard_negative_phase_centers: 8_192,
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub(super) struct OnlineContextPhaseStats {
    pub(super) fragments: u64,
    pub(super) transitions: u64,
    pub(super) l2_lattice_probes: u64,
    pub(super) l2_lattice_negative_examples: u64,
    pub(super) l2_lattice_empty_results: u64,
    pub(super) l2_lattice_max_competitors: u32,
    pub(super) l2_target_not_retained: u64,
    pub(super) hard_negative_false_winners: u64,
    pub(super) positive_reinforcements: u64,
    pub(super) positive_splits: u64,
    pub(super) anti_reinforcements: u64,
    pub(super) anti_splits: u64,
    pub(super) dropped_semantic_states: u64,
    pub(super) dropped_profiles: u64,
    pub(super) evicted_provisional_semantic_states: u64,
    pub(super) evicted_provisional_profiles: u64,
    pub(super) dropped_pending_negative_profiles: u64,
    pub(super) evicted_pending_negative_profiles: u64,
    pub(super) rejected_incompatible_modes: u64,
    pub(super) dropped_pair_profiles: u64,
    pub(super) evicted_provisional_pair_profiles: u64,
}

#[derive(Clone, Debug)]
struct SemanticBuilder {
    sum: Vec<PhaseCell>,
    center: Vec<PhaseCell>,
    support: u32,
}

impl SemanticBuilder {
    fn new() -> Self {
        Self {
            sum: empty_vector(CELLS),
            center: empty_vector(CELLS),
            support: 0,
        }
    }
}

#[derive(Clone, Debug)]
struct ProfileBuilder {
    positive: Vec<PhaseCenter>,
    negative: Vec<PhaseCenter>,
    hard_negative: Vec<PhaseCenter>,
    positive_examples: u32,
    negative_examples: u32,
    positive_calibration: ScalarReservoir,
    negative_calibration: ScalarReservoir,
}

#[derive(Default)]
struct PairProfileBuilder {
    low_wins: Vec<PhaseCenter>,
    high_wins: Vec<PhaseCenter>,
    hard_low_wins: Vec<PhaseCenter>,
    hard_high_wins: Vec<PhaseCenter>,
    relation_low_sources: u64,
    relation_high_sources: u64,
    observations: u32,
}

impl ProfileBuilder {
    fn new() -> Self {
        Self {
            positive: Vec::new(),
            negative: Vec::new(),
            hard_negative: Vec::new(),
            positive_examples: 0,
            negative_examples: 0,
            positive_calibration: ScalarReservoir::new(PROFILE_CALIBRATION_SAMPLES),
            negative_calibration: ScalarReservoir::new(PROFILE_CALIBRATION_SAMPLES),
        }
    }

    fn threshold_micro(&self) -> i32 {
        let positive_floor = self.positive_calibration.percentile(10).unwrap_or(0);
        let negative_ceiling = self.negative_calibration.percentile(90).unwrap_or(0);
        if self.negative_calibration.seen > 0 && positive_floor > negative_ceiling {
            negative_ceiling + (positive_floor - negative_ceiling) / 2
        } else {
            positive_floor.saturating_mul(7) / 10
        }
    }
}

#[derive(Clone, Debug)]
struct PendingNegativeBuilder {
    negative: Vec<PhaseCenter>,
    negative_examples: u32,
}

impl PendingNegativeBuilder {
    fn new() -> Self {
        Self {
            negative: Vec::new(),
            negative_examples: 0,
        }
    }
}

#[derive(Clone, Debug)]
struct ScalarReservoir {
    values: Vec<i32>,
    seen: u64,
    limit: usize,
}

#[derive(Clone, Copy)]
struct CompetitionCalibrationCase {
    target_hash: u64,
    target_relation_role: bool,
    context_hashes: [u64; MAX_CONTEXT_ATOMS],
    context_len: u8,
    competitor_hashes: [u64; MAX_COMPETITORS],
    competitor_relation_roles: [bool; MAX_COMPETITORS],
    competitor_len: u8,
}

impl CompetitionCalibrationCase {
    fn new(
        target_hash: u64,
        target_relation_role: bool,
        context_hashes: &[u64],
        competitor_entries: &[(u64, bool)],
    ) -> Self {
        let mut context = [0_u64; MAX_CONTEXT_ATOMS];
        let context_len = context_hashes.len().min(MAX_CONTEXT_ATOMS);
        context[..context_len].copy_from_slice(&context_hashes[..context_len]);
        let mut competitor_hashes = [0_u64; MAX_COMPETITORS];
        let mut competitor_relation_roles = [false; MAX_COMPETITORS];
        let competitor_len = competitor_entries.len().min(MAX_COMPETITORS);
        for (index, (hash, relation_role)) in
            competitor_entries[..competitor_len].iter().enumerate()
        {
            competitor_hashes[index] = *hash;
            competitor_relation_roles[index] = *relation_role;
        }
        Self {
            target_hash,
            target_relation_role,
            context_hashes: context,
            context_len: context_len as u8,
            competitor_hashes,
            competitor_relation_roles,
            competitor_len: competitor_len as u8,
        }
    }

    fn context(&self) -> &[u64] {
        &self.context_hashes[..usize::from(self.context_len)]
    }

    fn competitors(&self) -> &[u64] {
        &self.competitor_hashes[..usize::from(self.competitor_len)]
    }

    fn competitor_relation_role(&self, index: usize) -> bool {
        self.competitor_relation_roles[index]
    }
}

struct CompetitionCalibrationReservoir {
    cases: Vec<CompetitionCalibrationCase>,
    seen: u64,
}

impl CompetitionCalibrationReservoir {
    fn new() -> Self {
        Self {
            cases: Vec::with_capacity(COMPETITION_CALIBRATION_SAMPLES),
            seen: 0,
        }
    }

    fn observe(&mut self, case: CompetitionCalibrationCase, seed: u64) {
        self.seen = self.seen.saturating_add(1);
        if self.cases.len() < COMPETITION_CALIBRATION_SAMPLES {
            self.cases.push(case);
            return;
        }
        let draw = crate::stable_hash::mix64_golden(seed ^ self.seen.rotate_left(29)) % self.seen;
        if draw < COMPETITION_CALIBRATION_SAMPLES as u64 {
            self.cases[draw as usize] = case;
        }
    }

    fn bytes(&self) -> u64 {
        (self.cases.capacity() * std::mem::size_of::<CompetitionCalibrationCase>()) as u64
    }
}

struct BoundedFrequencySketch {
    counters: Box<[u16]>,
}

impl BoundedFrequencySketch {
    fn new() -> Self {
        Self {
            counters: vec![0; ADMISSION_SKETCH_LANES * ADMISSION_SKETCH_WIDTH].into_boxed_slice(),
        }
    }

    fn observe(&mut self, token_hash: u64) -> u16 {
        let mut estimate = u16::MAX;
        for lane in 0..ADMISSION_SKETCH_LANES {
            let mixed = crate::stable_hash::mix64_golden(
                token_hash ^ (lane as u64).wrapping_mul(0x9e37_79b9_7f4a_7c15),
            );
            let index =
                lane * ADMISSION_SKETCH_WIDTH + (mixed as usize & (ADMISSION_SKETCH_WIDTH - 1));
            self.counters[index] = self.counters[index].saturating_add(1);
            estimate = estimate.min(self.counters[index]);
        }
        estimate
    }

    fn bytes(&self) -> u64 {
        (self.counters.len() * std::mem::size_of::<u16>()) as u64
    }
}

fn pair_key_hash(key: PairKey) -> u64 {
    crate::stable_hash::mix64_golden(key.low_hash ^ key.high_hash.rotate_left(29))
}

impl ScalarReservoir {
    fn new(limit: usize) -> Self {
        Self {
            values: Vec::with_capacity(limit),
            seen: 0,
            limit,
        }
    }

    fn observe(&mut self, value: i32, seed: u64) {
        self.seen = self.seen.saturating_add(1);
        if self.values.len() < self.limit {
            self.values.push(value);
            return;
        }
        if self.limit == 0 {
            return;
        }
        let draw = crate::stable_hash::mix64_golden(seed ^ self.seen.rotate_left(23)) % self.seen;
        if draw < self.limit as u64 {
            self.values[draw as usize] = value;
        }
    }

    fn percentile(&self, percentile: usize) -> Option<i32> {
        if self.values.is_empty() {
            return None;
        }
        let mut values = self.values.clone();
        values.sort_unstable();
        let index = values
            .len()
            .saturating_sub(1)
            .saturating_mul(percentile.min(100))
            / 100;
        values.get(index).copied()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ClusterUpdate {
    Reinforced,
    Split,
    RejectedAtCapacity,
}

#[derive(Clone, Copy)]
enum ProfileBank {
    Positive,
    Negative,
}

pub(super) struct L2ProbeRequest {
    context_tokens: Vec<String>,
    target: String,
    target_hash: u64,
    context_hashes: Vec<u64>,
    target_margin_before_update: Option<i64>,
    signature_schema: u32,
    surface_field: Arc<SurfaceMutationField>,
}

pub(super) struct L2ProbeResult {
    target_hash: u64,
    target_signature: u64,
    target_relation_role: bool,
    context_hashes: Vec<u64>,
    target_margin_before_update: Option<i64>,
    target_retained: bool,
    competitors: Vec<(u64, u64, bool)>,
}

type ProbeJob = (usize, L2ProbeRequest);
type ProbeReceipt = (usize, L2ProbeResult);

pub(super) struct L2ProbePool {
    senders: Vec<SyncSender<ProbeJob>>,
    receipts: Receiver<ProbeReceipt>,
    workers: Vec<JoinHandle<()>>,
}

impl L2ProbePool {
    pub(super) fn new() -> Self {
        let worker_count = if cfg!(test) {
            1
        } else {
            thread::available_parallelism()
                .map(|workers| workers.get())
                .unwrap_or(1)
                .max(1)
        };
        Self::with_worker_count(worker_count)
    }

    fn with_worker_count(worker_count: usize) -> Self {
        let worker_count = worker_count.max(1);
        let (receipt_sender, receipts) = channel();
        let mut senders = Vec::with_capacity(worker_count);
        let mut workers = Vec::with_capacity(worker_count);
        for _ in 0..worker_count {
            let (sender, jobs) = sync_channel::<ProbeJob>(2);
            let receipt_sender = receipt_sender.clone();
            senders.push(sender);
            workers.push(thread::spawn(move || {
                while let Ok((ordinal, request)) = jobs.recv() {
                    if receipt_sender
                        .send((ordinal, execute_l2_probe(request)))
                        .is_err()
                    {
                        break;
                    }
                }
            }));
        }
        drop(receipt_sender);
        Self {
            senders,
            receipts,
            workers,
        }
    }

    pub(super) fn worker_count(&self) -> usize {
        self.senders.len()
    }

    pub(super) fn execute_batch(
        &self,
        requests: Vec<L2ProbeRequest>,
    ) -> io::Result<Vec<L2ProbeResult>> {
        let request_count = requests.len();
        if request_count == 0 {
            return Ok(Vec::new());
        }
        for (ordinal, request) in requests.into_iter().enumerate() {
            self.senders[ordinal % self.senders.len()]
                .send((ordinal, request))
                .map_err(|_| io::Error::other("L2 probe worker stopped"))?;
        }
        let mut receipts = Vec::with_capacity(request_count);
        for _ in 0..request_count {
            receipts.push(
                self.receipts
                    .recv()
                    .map_err(|_| io::Error::other("L2 probe receipt channel closed"))?,
            );
        }
        receipts.sort_by_key(|(ordinal, _)| *ordinal);
        Ok(receipts.into_iter().map(|(_, receipt)| receipt).collect())
    }
}

impl Drop for L2ProbePool {
    fn drop(&mut self) {
        self.senders.clear();
        for worker in self.workers.drain(..) {
            let _ = worker.join();
        }
    }
}

pub(super) struct OnlineContextPhaseLearner {
    config: OnlineContextPhaseConfig,
    semantic: HashMap<u64, SemanticBuilder>,
    profiles: HashMap<u64, ProfileBuilder>,
    signature_profiles: HashMap<u64, ProfileBuilder>,
    pair_profiles: HashMap<PairKey, PairProfileBuilder>,
    pair_frequency: BoundedFrequencySketch,
    exact_pair_profiles: usize,
    relation_pair_profiles: usize,
    exact_pair_admission_floor: u32,
    relation_pair_admission_floor: u32,
    pending_negative: HashMap<u64, PendingNegativeBuilder>,
    admission_frequency: BoundedFrequencySketch,
    pending_negative_frequency: BoundedFrequencySketch,
    provisional_semantic_order: VecDeque<u64>,
    provisional_profile_order: VecDeque<u64>,
    pending_negative_order: VecDeque<u64>,
    positive_phase_centers: usize,
    negative_phase_centers: usize,
    hard_negative_phase_centers: usize,
    competition_calibration: CompetitionCalibrationReservoir,
    surface_field: Arc<SurfaceMutationField>,
    stats: OnlineContextPhaseStats,
}

impl OnlineContextPhaseLearner {
    #[cfg(test)]
    pub(super) fn new(config: OnlineContextPhaseConfig) -> Self {
        Self::new_with_surface_field(config, Arc::new(SurfaceMutationField::default()))
    }

    pub(super) fn new_with_surface_field(
        config: OnlineContextPhaseConfig,
        surface_field: Arc<SurfaceMutationField>,
    ) -> Self {
        Self {
            config,
            semantic: HashMap::new(),
            profiles: HashMap::new(),
            signature_profiles: HashMap::new(),
            pair_profiles: HashMap::new(),
            pair_frequency: BoundedFrequencySketch::new(),
            exact_pair_profiles: 0,
            relation_pair_profiles: 0,
            exact_pair_admission_floor: 0,
            relation_pair_admission_floor: 0,
            pending_negative: HashMap::new(),
            admission_frequency: BoundedFrequencySketch::new(),
            pending_negative_frequency: BoundedFrequencySketch::new(),
            provisional_semantic_order: VecDeque::new(),
            provisional_profile_order: VecDeque::new(),
            pending_negative_order: VecDeque::new(),
            positive_phase_centers: 0,
            negative_phase_centers: 0,
            hard_negative_phase_centers: 0,
            competition_calibration: CompetitionCalibrationReservoir::new(),
            surface_field,
            stats: OnlineContextPhaseStats::default(),
        }
    }

    #[cfg(test)]
    pub(super) fn ingest_fragment(&mut self, tokens: &[String]) {
        let requests = self.ingest_fragment_positive(tokens);
        for request in requests {
            self.apply_l2_probe(execute_l2_probe(request));
        }
    }

    pub(super) fn ingest_fragment_positive(&mut self, tokens: &[String]) -> Vec<L2ProbeRequest> {
        if tokens.len() < 3 {
            return Vec::new();
        }
        self.stats.fragments = self.stats.fragments.saturating_add(1);
        let exact_hashes = tokens
            .iter()
            .map(|token| super::context_exact_hash(token))
            .collect::<Vec<_>>();
        let frequencies = exact_hashes
            .iter()
            .map(|token_hash| self.admission_frequency.observe(*token_hash))
            .collect::<Vec<_>>();
        self.update_semantic_states(&exact_hashes, &frequencies);
        let mut requests = Vec::new();

        for index in 1..tokens.len() {
            let target_hash = exact_hashes[index];
            if !self.ensure_profile(target_hash, frequencies[index]) {
                continue;
            }
            let start = index.saturating_sub(MAX_CONTEXT_TOKENS);
            let context_hashes =
                super::context_atom_hashes(&tokens[start..index], self.config.signature_schema);
            let target_relation_role = self.config.signature_schema
                >= super::SIGNATURE_SCHEMA_RELATION_ROLES
                && super::relation_role_candidate(&tokens[index]);
            let target_vector =
                self.relation_vector(&context_hashes, target_hash, target_relation_role);
            let target_margin_before_update = self
                .profiles
                .get(&target_hash)
                .is_some_and(|profile| !profile.positive.is_empty())
                .then(|| self.profile_margin_micro(target_hash, &target_vector))
                .flatten();
            let positive_centers_before = self
                .profiles
                .get(&target_hash)
                .map(|profile| profile.positive.len())
                .unwrap_or_default();
            let positive_updated =
                self.update_profile_bank(target_hash, ProfileBank::Positive, &target_vector);
            let positive_mode_split = positive_updated
                && self
                    .profiles
                    .get(&target_hash)
                    .is_some_and(|profile| profile.positive.len() > positive_centers_before);
            let target_examples = {
                let profile = self.profiles.get_mut(&target_hash).expect("profile exists");
                profile.positive_examples = profile.positive_examples.saturating_add(1);
                profile.positive_examples
            };
            self.stats.transitions = self.stats.transitions.saturating_add(1);
            let target_margin = self
                .profile_margin_micro(target_hash, &target_vector)
                .unwrap_or_default();
            self.profiles
                .get_mut(&target_hash)
                .expect("profile exists")
                .positive_calibration
                .observe(
                    micro_i32(target_margin),
                    target_hash ^ self.stats.transitions,
                );

            if !self.should_probe_l2(target_examples, positive_mode_split) {
                continue;
            }
            self.stats.l2_lattice_probes = self.stats.l2_lattice_probes.saturating_add(1);
            requests.push(L2ProbeRequest {
                context_tokens: tokens[start..index].to_vec(),
                target: tokens[index].clone(),
                target_hash,
                context_hashes,
                target_margin_before_update,
                signature_schema: self.config.signature_schema,
                surface_field: Arc::clone(&self.surface_field),
            });
        }
        requests
    }

    pub(super) fn apply_l2_probe_batch(
        &mut self,
        pool: &L2ProbePool,
        requests: &mut Vec<L2ProbeRequest>,
    ) -> io::Result<()> {
        for result in pool.execute_batch(std::mem::take(requests))? {
            self.apply_l2_probe(result);
        }
        Ok(())
    }

    fn apply_l2_probe(&mut self, result: L2ProbeResult) {
        let L2ProbeResult {
            target_hash,
            target_signature,
            target_relation_role,
            context_hashes,
            target_margin_before_update,
            target_retained,
            competitors,
        } = result;
        if !target_retained {
            self.stats.l2_target_not_retained = self.stats.l2_target_not_retained.saturating_add(1);
        }
        let observations = competitors
            .into_iter()
            .map(|(hash, signature, relation_role)| {
                (
                    hash,
                    signature,
                    relation_role,
                    self.relation_vector(&context_hashes, hash, relation_role),
                )
            })
            .collect::<Vec<_>>();
        if observations.is_empty() {
            self.stats.l2_lattice_empty_results =
                self.stats.l2_lattice_empty_results.saturating_add(1);
        }
        self.stats.l2_lattice_max_competitors = self
            .stats
            .l2_lattice_max_competitors
            .max(observations.len().min(u32::MAX as usize) as u32);
        self.stats.l2_lattice_negative_examples = self
            .stats
            .l2_lattice_negative_examples
            .saturating_add(observations.len() as u64);
        if observations.is_empty() {
            return;
        }

        let competitor_hashes = observations
            .iter()
            .map(|(hash, _, relation_role, _)| (*hash, *relation_role))
            .collect::<Vec<_>>();
        self.competition_calibration.observe(
            CompetitionCalibrationCase::new(
                target_hash,
                target_relation_role,
                &context_hashes,
                &competitor_hashes,
            ),
            target_hash ^ self.stats.transitions.rotate_left(17),
        );
        let relation_scene = target_relation_role
            || observations
                .iter()
                .any(|(_, _, relation_role, _)| *relation_role);
        let scene = self.relation_vector(&context_hashes, 0, relation_scene);
        // The signature key already carries the candidate's L2 state. Its
        // center must therefore encode only the surrounding scene; adding a
        // lexical target rotation here would make the alleged transfer field
        // another exact-word memory in disguise.
        self.update_signature_positive(target_signature, &scene);
        for (hash, signature, _, _) in &observations {
            // The same compact morphology/L2 state can be correct in one
            // scene and wrong in another. Keep the losing scene in a separate
            // anti-bank so signature transfer is interference, not a global
            // suffix preference.
            if relation_scene {
                self.update_signature_negative(*signature, &scene);
            }
            self.update_pair_winner(target_hash, *hash, &scene, false);
            self.update_pair_relation(target_hash, target_signature, *hash, *signature, &scene);
        }
        let false_winner = target_margin_before_update.and_then(|target_margin_before_update| {
            observations
                .iter()
                .filter_map(|(hash, _, _, vector)| {
                    let profile = self.profiles.get(hash)?;
                    if profile.positive_examples < self.config.min_profile_support {
                        return None;
                    }
                    let margin = self.profile_margin_micro(*hash, vector)?;
                    (margin >= target_margin_before_update
                        && margin >= i64::from(profile.threshold_micro()))
                    .then_some((*hash, margin))
                })
                .max_by_key(|(_, margin)| *margin)
                .map(|(hash, _)| hash)
        });
        if let Some(hash) = false_winner {
            self.update_pair_winner(target_hash, hash, &scene, true);
        }
    }

    fn update_pair_winner(&mut self, winner: u64, loser: u64, scene: &[PhaseCell], hard: bool) {
        let Some(key) = PairKey::new(winner, loser) else {
            return;
        };
        self.update_pair_key(key, winner == key.low_hash, scene, hard, None);
    }

    fn update_pair_relation(
        &mut self,
        winner: u64,
        winner_signature: u64,
        loser: u64,
        loser_signature: u64,
        scene: &[PhaseCell],
    ) {
        let Some(key) = PairKey::relation(winner, winner_signature, loser, loser_signature) else {
            return;
        };
        // Generalized L2-state relations are ordinary competition evidence.
        // A one-off false winner remains candidate-specific hard evidence.
        let source = PairKey::new(winner, loser).map(pair_key_hash);
        // A generalized relation transfers L2 roles, not lexical identity.
        // Its low/high banks therefore follow canonical signature order.
        self.update_pair_key(
            key,
            winner_signature < loser_signature,
            scene,
            false,
            source,
        );
    }

    fn update_pair_key(
        &mut self,
        key: PairKey,
        winner_is_low: bool,
        scene: &[PhaseCell],
        hard: bool,
        relation_source: Option<u64>,
    ) {
        let estimated_frequency = self.pair_frequency.observe(pair_key_hash(key));
        let is_relation = key.is_relation();
        let profile_limit = if is_relation {
            MAX_RELATION_PAIR_PROFILES
        } else {
            MAX_EXACT_PAIR_PROFILES
        };
        let existing_kind = if is_relation {
            self.relation_pair_profiles
        } else {
            self.exact_pair_profiles
        };
        let is_new = !self.pair_profiles.contains_key(&key);
        let admission_floor = if is_relation {
            self.relation_pair_admission_floor
        } else {
            self.exact_pair_admission_floor
        };
        if is_new && existing_kind >= profile_limit {
            if u32::from(estimated_frequency) <= admission_floor {
                self.stats.dropped_pair_profiles =
                    self.stats.dropped_pair_profiles.saturating_add(1);
                return;
            }
            let evict = self
                .pair_profiles
                .iter()
                .filter(|(existing_key, _)| existing_key.is_relation() == is_relation)
                .min_by_key(|(existing_key, profile)| (profile.observations, **existing_key))
                .map(|(existing_key, profile)| (*existing_key, profile.observations));
            let Some((evict_key, evict_observations)) = evict else {
                self.stats.dropped_pair_profiles =
                    self.stats.dropped_pair_profiles.saturating_add(1);
                return;
            };
            if u32::from(estimated_frequency) <= evict_observations {
                self.stats.dropped_pair_profiles =
                    self.stats.dropped_pair_profiles.saturating_add(1);
                return;
            }
            self.pair_profiles.remove(&evict_key);
            if is_relation {
                self.relation_pair_profiles = self.relation_pair_profiles.saturating_sub(1);
                self.relation_pair_admission_floor =
                    self.relation_pair_admission_floor.max(evict_observations);
            } else {
                self.exact_pair_profiles = self.exact_pair_profiles.saturating_sub(1);
                self.exact_pair_admission_floor =
                    self.exact_pair_admission_floor.max(evict_observations);
            }
            self.stats.evicted_provisional_pair_profiles = self
                .stats
                .evicted_provisional_pair_profiles
                .saturating_add(1);
        }
        debug_assert!(self.pair_profiles.len() <= MAX_PAIR_PROFILES);
        let pair = self.pair_profiles.entry(key).or_default();
        if is_new {
            if is_relation {
                self.relation_pair_profiles = self.relation_pair_profiles.saturating_add(1);
            } else {
                self.exact_pair_profiles = self.exact_pair_profiles.saturating_add(1);
            }
        }
        pair.observations = pair.observations.saturating_add(1);
        if let Some(source) = relation_source {
            let bit = 1_u64 << (source as u32 & 63);
            if winner_is_low {
                pair.relation_low_sources |= bit;
            } else {
                pair.relation_high_sources |= bit;
            }
        }
        let bank = match (winner_is_low, hard) {
            (true, false) => &mut pair.low_wins,
            (false, false) => &mut pair.high_wins,
            (true, true) => &mut pair.hard_low_wins,
            (false, true) => &mut pair.hard_high_wins,
        };
        let mut ignored_total = 0;
        let max_centers = if hard {
            MAX_HARD_PAIR_CENTERS_PER_BANK
        } else {
            MAX_PAIR_CENTERS_PER_BANK
        };
        let _ = update_bounded_cluster(bank, scene, max_centers, &mut ignored_total, usize::MAX);
    }

    pub(super) fn snapshot(&self) -> ContextPhasePackage {
        let mut semantic_states = self
            .semantic
            .iter()
            .filter(|(_, builder)| builder.support >= 2)
            .map(|(token_hash, builder)| TokenSemanticState {
                token_hash: *token_hash,
                support: builder.support,
                center: builder.center.clone(),
            })
            .collect::<Vec<_>>();
        semantic_states.sort_by_key(|state| state.token_hash);

        let mut profiles = self
            .profiles
            .iter()
            .filter(|(_, builder)| builder.positive_examples >= self.config.min_profile_support)
            .map(|(token_hash, builder)| ContextCandidateProfile {
                token_hash: *token_hash,
                positive_examples: builder.positive_examples,
                negative_examples: builder.negative_examples,
                threshold_micro: builder.threshold_micro(),
                positive: builder.positive.clone(),
                negative: builder.negative.clone(),
                hard_negative: builder.hard_negative.clone(),
            })
            .collect::<Vec<_>>();
        profiles.sort_by_key(|profile| profile.token_hash);
        let mut signature_profiles = self
            .signature_profiles
            .iter()
            .filter(|(_, builder)| builder.positive_examples >= self.config.min_profile_support)
            .map(|(signature, builder)| ContextCandidateProfile {
                token_hash: *signature,
                positive_examples: builder.positive_examples,
                negative_examples: builder.negative_examples,
                threshold_micro: builder.threshold_micro(),
                positive: builder.positive.clone(),
                negative: builder.negative.clone(),
                hard_negative: Vec::new(),
            })
            .collect::<Vec<_>>();
        signature_profiles.sort_by_key(|profile| profile.token_hash);
        let mut pair_profiles = self
            .pair_profiles
            .iter()
            .filter_map(|(key, builder)| {
                let low_ready =
                    !key.is_relation() || builder.relation_low_sources.count_ones() >= 3;
                let high_ready =
                    !key.is_relation() || builder.relation_high_sources.count_ones() >= 3;
                let profile = ContextPairPhaseProfile {
                    low_hash: key.low_hash,
                    high_hash: key.high_hash,
                    low_wins: if low_ready {
                        builder.low_wins.clone()
                    } else {
                        Vec::new()
                    },
                    high_wins: if high_ready {
                        builder.high_wins.clone()
                    } else {
                        Vec::new()
                    },
                    hard_low_wins: builder.hard_low_wins.clone(),
                    hard_high_wins: builder.hard_high_wins.clone(),
                };
                (!profile.low_wins.is_empty()
                    || !profile.high_wins.is_empty()
                    || !profile.hard_low_wins.is_empty()
                    || !profile.hard_high_wins.is_empty())
                .then_some(profile)
            })
            .collect::<Vec<_>>();
        pair_profiles.sort_by_key(|profile| (profile.low_hash, profile.high_hash));
        let mut profile_thresholds = profiles
            .iter()
            .map(|profile| profile.threshold_micro)
            .collect::<Vec<_>>();
        profile_thresholds.sort_unstable();
        let global_threshold_micro = percentile_i32(&profile_thresholds, 25).unwrap_or(1).max(1);
        let competition_threshold_micro = self.learned_competition_threshold();

        ContextPhasePackage {
            semantic_states,
            profiles,
            signature_profiles,
            pair_profiles,
            transitions: self.stats.transitions,
            corpus_fragments: self.stats.fragments.min(u64::from(u32::MAX)) as u32,
            global_threshold_micro,
            competition_threshold_micro,
            pairwise_threshold_micro: competition_threshold_micro,
            signature_schema: self.config.signature_schema,
        }
    }

    pub(super) fn stats(&self) -> OnlineContextPhaseStats {
        self.stats
    }

    pub(super) fn profile_count(&self) -> usize {
        self.profiles.len()
    }

    pub(super) fn pending_negative_profile_count(&self) -> usize {
        self.pending_negative.len()
    }

    pub(super) fn pending_negative_center_count(&self) -> usize {
        self.pending_negative
            .values()
            .map(|profile| profile.negative.len())
            .sum()
    }

    pub(super) fn semantic_state_count(&self) -> usize {
        self.semantic.len()
    }

    pub(super) fn phase_center_count(&self) -> usize {
        self.positive_phase_centers
            .saturating_add(self.negative_phase_centers)
            .saturating_add(self.hard_negative_phase_centers)
    }

    pub(super) fn positive_phase_center_count(&self) -> usize {
        self.positive_phase_centers
    }

    pub(super) fn negative_phase_center_count(&self) -> usize {
        self.negative_phase_centers
    }

    pub(super) fn hard_negative_phase_center_count(&self) -> usize {
        self.hard_negative_phase_centers
    }

    pub(super) fn competition_calibration_cases(&self) -> usize {
        self.competition_calibration.cases.len()
    }

    pub(super) fn pair_profile_count(&self) -> usize {
        self.pair_profiles.len()
    }

    pub(super) fn pair_center_count(&self) -> usize {
        self.pair_profiles
            .values()
            .map(|profile| {
                profile.low_wins.len()
                    + profile.high_wins.len()
                    + profile.hard_low_wins.len()
                    + profile.hard_high_wins.len()
            })
            .sum()
    }

    pub(super) fn estimated_bytes(&self) -> u64 {
        let phase_cell_bytes = std::mem::size_of::<PhaseCell>() as u64;
        let semantic =
            self.semantic.len() as u64 * u64::from((CELLS * 2) as u32) * phase_cell_bytes;
        let centers =
            self.phase_center_count() as u64 * u64::from((CELLS * 2) as u32) * phase_cell_bytes;
        let profiles = self.profiles.len() as u64 * 384;
        let signature_profiles = self.signature_profiles.len() as u64 * 320;
        let pairs = self.pair_profile_count() as u64 * 128
            + self.pair_center_count() as u64 * u64::from((CELLS * 2) as u32) * phase_cell_bytes;
        let pending_negative = self.pending_negative.len() as u64 * 96;
        let queue_entries = self
            .provisional_semantic_order
            .len()
            .saturating_add(self.provisional_profile_order.len())
            .saturating_add(self.pending_negative_order.len());
        let admission = self
            .admission_frequency
            .bytes()
            .saturating_add(self.pending_negative_frequency.bytes())
            .saturating_add(self.pair_frequency.bytes())
            .saturating_add(queue_entries as u64 * std::mem::size_of::<u64>() as u64);
        semantic
            .saturating_add(centers)
            .saturating_add(profiles)
            .saturating_add(signature_profiles)
            .saturating_add(pairs)
            .saturating_add(pending_negative)
            .saturating_add(admission)
            .saturating_add(self.competition_calibration.bytes())
    }

    fn should_probe_l2(&self, positive_examples: u32, positive_mode_split: bool) -> bool {
        positive_examples >= self.config.min_profile_support
            && (positive_examples.is_power_of_two() || positive_mode_split)
    }

    fn ensure_profile(&mut self, token_hash: u64, estimated_frequency: u16) -> bool {
        if self.profiles.contains_key(&token_hash) {
            return true;
        }
        // A one-off token must not consume a full candidate phase profile.
        // Frequency is only an admission signal here: it never ranks or
        // authorizes a candidate. The first observation remains represented
        // by the compact semantic field; a profile is born on a second
        // independent surface and still needs two phase scenes before the
        // snapshot can expose it to runtime readout.
        if u32::from(estimated_frequency) < self.config.min_profile_support {
            self.stats.dropped_profiles = self.stats.dropped_profiles.saturating_add(1);
            return false;
        }
        if self.profiles.len() >= self.config.max_profiles {
            let eligible = u32::from(estimated_frequency) >= self.config.min_profile_support;
            if !eligible || !self.evict_provisional_profile() {
                self.stats.dropped_profiles = self.stats.dropped_profiles.saturating_add(1);
                return false;
            }
        }
        let mut profile = ProfileBuilder::new();
        if let Some(pending) = self.pending_negative.remove(&token_hash) {
            profile.negative = pending.negative;
            profile.negative_examples = pending.negative_examples;
        }
        self.profiles.insert(token_hash, profile);
        self.provisional_profile_order.push_back(token_hash);
        true
    }

    /// Trains context geometry for a compact L2 state, not a word identity.
    /// The hot reader may use this only to strengthen an existing exact
    /// profile, so this bank cannot invent lexical authority by itself.
    fn update_signature_positive(&mut self, signature: u64, vector: &[PhaseCell]) {
        if !self.signature_profiles.contains_key(&signature)
            && self.signature_profiles.len() >= MAX_SIGNATURE_PROFILES
        {
            return;
        }
        let profile = self
            .signature_profiles
            .entry(signature)
            .or_insert_with(ProfileBuilder::new);
        let outcome = update_bounded_cluster(
            &mut profile.positive,
            vector,
            MAX_POSITIVE_CENTERS,
            &mut self.positive_phase_centers,
            self.config.max_positive_phase_centers,
        );
        match outcome {
            ClusterUpdate::Reinforced => {
                self.stats.positive_reinforcements =
                    self.stats.positive_reinforcements.saturating_add(1)
            }
            ClusterUpdate::Split => {
                self.stats.positive_splits = self.stats.positive_splits.saturating_add(1)
            }
            ClusterUpdate::RejectedAtCapacity => {
                self.stats.rejected_incompatible_modes =
                    self.stats.rejected_incompatible_modes.saturating_add(1);
                return;
            }
        }
        profile.positive_examples = profile.positive_examples.saturating_add(1);
        let margin = phase_micro(max_coherence(vector, &profile.positive).unwrap_or_default());
        profile.positive_calibration.observe(
            micro_i32(margin),
            signature ^ self.stats.transitions.rotate_left(5),
        );
    }

    fn update_signature_negative(&mut self, signature: u64, vector: &[PhaseCell]) {
        let Some(profile) = self.signature_profiles.get_mut(&signature) else {
            return;
        };
        let outcome = update_bounded_cluster(
            &mut profile.negative,
            vector,
            MAX_ANTI_CENTERS,
            &mut self.negative_phase_centers,
            self.config.max_negative_phase_centers,
        );
        if matches!(outcome, ClusterUpdate::RejectedAtCapacity) {
            self.stats.rejected_incompatible_modes =
                self.stats.rejected_incompatible_modes.saturating_add(1);
            return;
        }
        profile.negative_examples = profile.negative_examples.saturating_add(1);
        let margin = phase_micro(max_coherence(vector, &profile.negative).unwrap_or_default());
        profile.negative_calibration.observe(
            micro_i32(margin),
            signature ^ self.stats.transitions.rotate_left(11),
        );
    }

    fn evict_provisional_profile(&mut self) -> bool {
        while let Some(token_hash) = self.provisional_profile_order.pop_front() {
            let Some(profile) = self.profiles.get(&token_hash) else {
                continue;
            };
            if profile.positive_examples >= self.config.min_profile_support {
                continue;
            }
            let positive_centers = profile.positive.len();
            let negative_centers = profile.negative.len();
            let hard_negative_centers = profile.hard_negative.len();
            self.profiles.remove(&token_hash);
            self.positive_phase_centers =
                self.positive_phase_centers.saturating_sub(positive_centers);
            self.negative_phase_centers =
                self.negative_phase_centers.saturating_sub(negative_centers);
            self.hard_negative_phase_centers = self
                .hard_negative_phase_centers
                .saturating_sub(hard_negative_centers);
            self.stats.evicted_provisional_profiles =
                self.stats.evicted_provisional_profiles.saturating_add(1);
            return true;
        }
        false
    }

    fn update_negative_relation(&mut self, token_hash: u64, vector: &[PhaseCell]) {
        if self.profiles.contains_key(&token_hash) {
            if self.update_profile_bank(token_hash, ProfileBank::Negative, vector) {
                let margin = self
                    .profile_margin_micro(token_hash, vector)
                    .unwrap_or_default();
                let profile = self.profiles.get_mut(&token_hash).expect("profile exists");
                profile.negative_examples = profile.negative_examples.saturating_add(1);
                profile.negative_calibration.observe(
                    micro_i32(margin),
                    token_hash ^ self.stats.transitions.rotate_left(7),
                );
            }
            return;
        }

        let frequency = self.pending_negative_frequency.observe(token_hash);
        if !self.pending_negative.contains_key(&token_hash) {
            if self.pending_negative.len() >= self.config.max_profiles
                && (frequency < 2 || !self.evict_pending_negative_profile())
            {
                self.stats.dropped_pending_negative_profiles = self
                    .stats
                    .dropped_pending_negative_profiles
                    .saturating_add(1);
                return;
            }
            self.pending_negative
                .insert(token_hash, PendingNegativeBuilder::new());
            self.pending_negative_order.push_back(token_hash);
        }
        let outcome = {
            let pending = self
                .pending_negative
                .get_mut(&token_hash)
                .expect("pending negative profile exists");
            update_bounded_cluster(
                &mut pending.negative,
                vector,
                MAX_PENDING_ANTI_CENTERS,
                &mut self.negative_phase_centers,
                self.config.max_negative_phase_centers,
            )
        };
        match outcome {
            ClusterUpdate::Reinforced => {
                self.stats.anti_reinforcements = self.stats.anti_reinforcements.saturating_add(1)
            }
            ClusterUpdate::Split => {
                self.stats.anti_splits = self.stats.anti_splits.saturating_add(1)
            }
            ClusterUpdate::RejectedAtCapacity => {
                self.stats.rejected_incompatible_modes =
                    self.stats.rejected_incompatible_modes.saturating_add(1);
                return;
            }
        }
        if let Some(pending) = self.pending_negative.get_mut(&token_hash) {
            pending.negative_examples = pending.negative_examples.saturating_add(1);
        }
    }

    fn update_hard_negative_relation(&mut self, token_hash: u64, vector: &[PhaseCell]) {
        let Some(profile) = self.profiles.get_mut(&token_hash) else {
            return;
        };
        let outcome = update_bounded_cluster(
            &mut profile.hard_negative,
            vector,
            MAX_ANTI_CENTERS,
            &mut self.hard_negative_phase_centers,
            self.config.max_hard_negative_phase_centers,
        );
        if matches!(outcome, ClusterUpdate::RejectedAtCapacity) {
            self.stats.rejected_incompatible_modes =
                self.stats.rejected_incompatible_modes.saturating_add(1);
            return;
        }
        self.stats.hard_negative_false_winners =
            self.stats.hard_negative_false_winners.saturating_add(1);
    }

    fn evict_pending_negative_profile(&mut self) -> bool {
        while let Some(token_hash) = self.pending_negative_order.pop_front() {
            let Some(pending) = self.pending_negative.get(&token_hash) else {
                continue;
            };
            if pending.negative_examples >= 2 {
                continue;
            }
            let centers = pending.negative.len();
            self.pending_negative.remove(&token_hash);
            self.negative_phase_centers = self.negative_phase_centers.saturating_sub(centers);
            self.stats.evicted_pending_negative_profiles = self
                .stats
                .evicted_pending_negative_profiles
                .saturating_add(1);
            return true;
        }
        false
    }

    fn update_semantic_states(&mut self, hashes: &[u64], frequencies: &[u16]) {
        for (index, token_hash) in hashes.iter().copied().enumerate() {
            let mut delta = empty_vector(CELLS);
            let start = index.saturating_sub(4);
            let end = (index + 5).min(hashes.len());
            for (absolute, neighbor_hash) in hashes[start..end].iter().copied().enumerate() {
                let absolute = start + absolute;
                if absolute == index {
                    continue;
                }
                let relative = absolute as isize - index as isize;
                let position = relative.unsigned_abs() as u64;
                let direction = if relative < 0 { 0x4c } else { 0x52 };
                add_hashed_atom(
                    &mut delta,
                    neighbor_hash ^ (direction << 56),
                    token_hash ^ position.rotate_left(11),
                    1.0 / (position as f32).sqrt(),
                );
            }
            if !self.semantic.contains_key(&token_hash) {
                if self.semantic.len() >= self.config.max_semantic_states
                    && (frequencies.get(index).copied().unwrap_or_default() < 2
                        || !self.evict_provisional_semantic_state())
                {
                    self.stats.dropped_semantic_states =
                        self.stats.dropped_semantic_states.saturating_add(1);
                    continue;
                }
                self.semantic.insert(token_hash, SemanticBuilder::new());
                self.provisional_semantic_order.push_back(token_hash);
            }
            let builder = self.semantic.get_mut(&token_hash).expect("state exists");
            // Profiles must see one stable semantic coordinate system. The
            // first two observations establish the anchor; later evidence
            // raises confidence without rotating already learned relations.
            if builder.support < 2 {
                add_phase_vector(&mut builder.sum, &delta);
                builder.center = phase_center_from_sum(&builder.sum);
            }
            builder.support = builder.support.saturating_add(1);
        }
    }

    fn evict_provisional_semantic_state(&mut self) -> bool {
        while let Some(token_hash) = self.provisional_semantic_order.pop_front() {
            let Some(state) = self.semantic.get(&token_hash) else {
                continue;
            };
            if state.support >= 2 {
                continue;
            }
            self.semantic.remove(&token_hash);
            self.stats.evicted_provisional_semantic_states = self
                .stats
                .evicted_provisional_semantic_states
                .saturating_add(1);
            return true;
        }
        false
    }

    fn relation_vector(
        &self,
        context_hashes: &[u64],
        candidate_hash: u64,
        relation_roles: bool,
    ) -> Vec<PhaseCell> {
        let projected_hashes;
        let scene_hashes = if relation_roles {
            context_hashes
        } else {
            projected_hashes = context_hashes
                .iter()
                .step_by(2)
                .copied()
                .collect::<Vec<_>>();
            &projected_hashes
        };
        let mut vector = if relation_roles {
            canonical_relation_scene_wave(
                scene_hashes,
                ContextPhaseMode::Full,
                |atom_index, hash| {
                    (atom_index % 2 == 0)
                        .then(|| {
                            self.semantic
                                .get(&hash)
                                .filter(|state| state.support >= 2)
                                .map(|state| (state.center.as_slice(), state.support))
                        })
                        .flatten()
                },
            )
        } else {
            canonical_scene_wave(scene_hashes, ContextPhaseMode::Full, |_, hash| {
                self.semantic
                    .get(&hash)
                    .filter(|state| state.support >= 2)
                    .map(|state| (state.center.as_slice(), state.support))
            })
        };
        if let Some(state) = self
            .semantic
            .get(&candidate_hash)
            .filter(|state| state.support >= 2)
        {
            let semantic_weight = super::candidate_semantic_relation_weight(state.support);
            add_rotated_vector(
                &mut vector,
                &state.center,
                candidate_hash ^ 0x0052_454c_4154_494f,
                semantic_weight,
            );
        }
        phase_center_from_sum(&vector)
    }

    fn update_profile_bank(
        &mut self,
        token_hash: u64,
        bank: ProfileBank,
        vector: &[PhaseCell],
    ) -> bool {
        let outcome = {
            let Some(profile) = self.profiles.get_mut(&token_hash) else {
                return false;
            };
            let (centers, max_centers, total_centers, total_limit) = match bank {
                ProfileBank::Positive => (
                    &mut profile.positive,
                    MAX_POSITIVE_CENTERS,
                    &mut self.positive_phase_centers,
                    self.config.max_positive_phase_centers,
                ),
                ProfileBank::Negative => (
                    &mut profile.negative,
                    MAX_ANTI_CENTERS,
                    &mut self.negative_phase_centers,
                    self.config.max_negative_phase_centers,
                ),
            };
            update_bounded_cluster(centers, vector, max_centers, total_centers, total_limit)
        };
        match (bank, outcome) {
            (ProfileBank::Positive, ClusterUpdate::Reinforced) => {
                self.stats.positive_reinforcements =
                    self.stats.positive_reinforcements.saturating_add(1)
            }
            (ProfileBank::Positive, ClusterUpdate::Split) => {
                self.stats.positive_splits = self.stats.positive_splits.saturating_add(1)
            }
            (ProfileBank::Negative, ClusterUpdate::Reinforced) => {
                self.stats.anti_reinforcements = self.stats.anti_reinforcements.saturating_add(1)
            }
            (ProfileBank::Negative, ClusterUpdate::Split) => {
                self.stats.anti_splits = self.stats.anti_splits.saturating_add(1)
            }
            (_, ClusterUpdate::RejectedAtCapacity) => {
                self.stats.rejected_incompatible_modes =
                    self.stats.rejected_incompatible_modes.saturating_add(1);
                return false;
            }
        }
        true
    }

    fn profile_margin_micro(&self, token_hash: u64, vector: &[PhaseCell]) -> Option<i64> {
        let profile = self.profiles.get(&token_hash)?;
        let positive = max_coherence(vector, &profile.positive).unwrap_or_default();
        // Cold corpus competition is represented by PairKey. A candidate
        // profile alone cannot know which target it lost to, so generic
        // negatives must not distort calibration of unary geometry.
        Some(phase_micro(positive))
    }

    fn learned_competition_threshold(&self) -> i32 {
        let mut correct_gaps = Vec::with_capacity(self.competition_calibration.cases.len());
        let mut wrong_gaps = Vec::new();
        for case in &self.competition_calibration.cases {
            let target_vector =
                self.relation_vector(case.context(), case.target_hash, case.target_relation_role);
            let Some(target_margin) = self.profile_margin_micro(case.target_hash, &target_vector)
            else {
                continue;
            };
            let strongest_competitor = case
                .competitors()
                .iter()
                .enumerate()
                .filter_map(|(index, hash)| {
                    let vector = self.relation_vector(
                        case.context(),
                        *hash,
                        case.competitor_relation_role(index),
                    );
                    self.profile_margin_micro(*hash, &vector)
                })
                .max();
            let Some(competitor_margin) = strongest_competitor else {
                continue;
            };
            let gap = target_margin.saturating_sub(competitor_margin);
            if gap > 0 {
                correct_gaps.push(gap.min(i64::from(i32::MAX)) as i32);
            } else if gap < 0 {
                wrong_gaps.push(gap.saturating_neg().min(i64::from(i32::MAX)) as i32);
            }
        }
        correct_gaps.sort_unstable();
        wrong_gaps.sort_unstable();
        let positive_floor = percentile_i32(&correct_gaps, 25).unwrap_or(1).max(1);
        let negative_ceiling = percentile_i32(&wrong_gaps, 90).unwrap_or(0).max(0);
        if positive_floor > negative_ceiling {
            negative_ceiling + (positive_floor - negative_ceiling) / 2
        } else {
            positive_floor
        }
    }
}

fn update_bounded_cluster(
    centers: &mut Vec<PhaseCenter>,
    vector: &[PhaseCell],
    bank_limit: usize,
    total_centers: &mut usize,
    total_limit: usize,
) -> ClusterUpdate {
    let best = centers
        .iter()
        .enumerate()
        .map(|(index, center)| (index, center.coherence(vector)))
        .max_by(|left, right| left.1.total_cmp(&right.1));
    if let Some((index, coherence)) = best {
        if coherence >= CENTER_SPLIT_COHERENCE {
            let center = &mut centers[index];
            add_phase_vector(&mut center.sum, vector);
            center.center = phase_center_from_sum(&center.sum);
            center.support = center.support.saturating_add(1);
            return ClusterUpdate::Reinforced;
        }
    }
    if centers.len() >= bank_limit || *total_centers >= total_limit {
        return ClusterUpdate::RejectedAtCapacity;
    }
    centers.push(PhaseCenter::from_sum(vector.to_vec(), 1));
    *total_centers = total_centers.saturating_add(1);
    ClusterUpdate::Split
}

fn execute_l2_probe(request: L2ProbeRequest) -> L2ProbeResult {
    let L2ProbeRequest {
        context_tokens,
        target,
        target_hash,
        context_hashes,
        target_margin_before_update,
        signature_schema,
        surface_field,
    } = request;
    let mut seen = BTreeSet::new();
    let target_relation_role = signature_schema >= super::SIGNATURE_SCHEMA_RELATION_ROLES
        && super::relation_role_candidate(&target);
    let lattice = l2_lattice_probe(
        &context_tokens,
        &target,
        MAX_COMPETITORS,
        surface_field.as_ref(),
    );
    let competitors = lattice
        .competitors
        .into_iter()
        .filter_map(|competitor| {
            let token = crate::word_reader::last_text_word(&competitor).unwrap_or_default();
            let competitor_hash = hash_text(&token.to_lowercase());
            (competitor_hash != target_hash && seen.insert(competitor_hash)).then(|| {
                (
                    competitor_hash,
                    super::candidate_l2_signature_for_schema(&competitor, signature_schema),
                    signature_schema >= super::SIGNATURE_SCHEMA_RELATION_ROLES
                        && super::relation_role_candidate(&competitor),
                )
            })
        })
        .collect();
    L2ProbeResult {
        target_hash,
        target_signature: super::candidate_l2_signature_for_schema(&target, signature_schema),
        target_relation_role,
        context_hashes,
        target_margin_before_update,
        target_retained: lattice.target_retained,
        competitors,
    }
}

pub(super) struct L2LatticeProbe {
    pub(super) competitors: Vec<String>,
    pub(super) target_retained: bool,
}

pub(super) fn l2_lattice_competitors(
    context: &[String],
    target: &str,
    limit: usize,
    surface_field: &SurfaceMutationField,
) -> Vec<String> {
    l2_lattice_probe(context, target, limit, surface_field).competitors
}

pub(super) fn l2_lattice_probe(
    context: &[String],
    target: &str,
    limit: usize,
    surface_field: &SurfaceMutationField,
) -> L2LatticeProbe {
    if limit == 0 {
        return L2LatticeProbe {
            competitors: Vec::new(),
            target_retained: false,
        };
    }
    let context_prefix = context.join(" ");
    let normalized_target = target.to_lowercase();
    // The corpus target is a teacher label. It may generate a damaged surface,
    // but it must never rank the resulting L2 candidates: ordering them by
    // distance to `target` would leak the answer into both learning and proof.
    // Keep the bounded, deterministic order emitted by the real L2 readout.
    let mut seen = BTreeSet::new();
    let mut competitors = Vec::with_capacity(limit);
    let probe_surfaces = surface_field.damaged_surfaces(target, MAX_L2_TRAINING_SURFACES);
    if target.chars().count() == 1 && target.chars().all(char::is_alphabetic) {
        let direction = if target.chars().all(crate::keyboard::is_cyrillic_letter) {
            crate::dict::Direction::Ru2Us
        } else if target.is_ascii() {
            crate::dict::Direction::Us2Ru
        } else {
            return L2LatticeProbe {
                competitors,
                target_retained: true,
            };
        };
        let projected = crate::dict::convert(target, direction).to_lowercase();
        if projected != normalized_target && seen.insert(projected.clone()) {
            competitors.push(projected);
        }
        return L2LatticeProbe {
            competitors,
            target_retained: true,
        };
    }
    if std::env::var_os("LAY_L3_REAL_L2_PROBE").is_some() {
        return real_l2_lattice_competitors(
            &context_prefix,
            &normalized_target,
            probe_surfaces,
            limit,
            |context_prefix, damaged| {
                crate::nanda_wave::l2_field::cold_probe_surfaces(context_prefix, damaged)
            },
        );
    }
    for damaged in probe_surfaces {
        for candidate in crate::nanda_wave::l2::correction_l2_word_candidates(
            &context_prefix,
            &damaged,
            limit.saturating_mul(4),
        ) {
            let candidate = candidate.surface.to_lowercase();
            if candidate != normalized_target && seen.insert(candidate.clone()) {
                competitors.push(candidate);
                if competitors.len() >= limit {
                    return L2LatticeProbe {
                        competitors,
                        target_retained: true,
                    };
                }
            }
        }
    }
    L2LatticeProbe {
        competitors,
        target_retained: true,
    }
}

fn real_l2_lattice_competitors<F>(
    context_prefix: &str,
    normalized_target: &str,
    probe_surfaces: Vec<String>,
    limit: usize,
    mut probe: F,
) -> L2LatticeProbe
where
    F: FnMut(&str, &str) -> Vec<String>,
{
    let mut seen = BTreeSet::new();
    let mut competitors = Vec::with_capacity(limit);
    let mut target_retained = false;
    'surfaces: for damaged in probe_surfaces {
        for candidate in probe(context_prefix, &damaged) {
            if candidate == normalized_target {
                target_retained = true;
            } else if competitors.len() < limit && seen.insert(candidate.clone()) {
                competitors.push(candidate);
            }
        }
        if target_retained && competitors.len() >= limit {
            break 'surfaces;
        }
    }
    L2LatticeProbe {
        competitors,
        target_retained,
    }
}

fn percentile_i32(values: &[i32], percentile: usize) -> Option<i32> {
    if values.is_empty() {
        return None;
    }
    let index = values
        .len()
        .saturating_sub(1)
        .saturating_mul(percentile.min(100))
        / 100;
    values.get(index).copied()
}

fn micro_i32(value: i64) -> i32 {
    value.clamp(i64::from(i32::MIN), i64::from(i32::MAX)) as i32
}

#[cfg(test)]
mod tests {
    use super::*;

    fn phase_vector(identity: u64) -> Vec<PhaseCell> {
        let mut vector = empty_vector(CELLS);
        add_hashed_atom(&mut vector, identity, identity.rotate_left(13), 1.0);
        phase_center_from_sum(&vector)
    }

    #[test]
    fn repeated_mode_reinforces_and_incompatible_mode_splits() {
        let first = phase_vector(11);
        let second = phase_vector(29);
        let mut centers = Vec::new();
        let mut total = 0;

        assert_eq!(
            update_bounded_cluster(&mut centers, &first, 2, &mut total, 2),
            ClusterUpdate::Split
        );
        assert_eq!(
            update_bounded_cluster(&mut centers, &first, 2, &mut total, 2),
            ClusterUpdate::Reinforced
        );
        assert_eq!(centers[0].support, 2);
        assert_eq!(
            update_bounded_cluster(&mut centers, &second, 2, &mut total, 2),
            ClusterUpdate::Split
        );
        assert_eq!(centers.len(), 2);
    }

    #[test]
    fn l2_probe_schedule_covers_new_modes_without_probing_singletons() {
        let learner = OnlineContextPhaseLearner::new(OnlineContextPhaseConfig {
            min_profile_support: 2,
            signature_schema: super::super::SIGNATURE_SCHEMA_RELATION_ROLES,
            max_semantic_states: 8,
            max_profiles: 8,
            max_positive_phase_centers: 8,
            max_negative_phase_centers: 4,
            max_hard_negative_phase_centers: 4,
        });

        assert!(!learner.should_probe_l2(1, true));
        assert!(learner.should_probe_l2(2, false));
        assert!(!learner.should_probe_l2(3, false));
        assert!(learner.should_probe_l2(3, true));
        assert!(learner.should_probe_l2(4, false));
    }

    #[test]
    fn generalized_pair_requires_three_independent_exact_surfaces() {
        let mut learner = OnlineContextPhaseLearner::new(OnlineContextPhaseConfig::production(2));
        let scene = phase_vector(73);
        learner.update_pair_relation(11, 17, 19, 29, &scene);
        learner.update_pair_relation(23, 17, 31, 29, &scene);
        assert!(learner.snapshot().pair_profiles.is_empty());

        learner.update_pair_relation(37, 17, 41, 29, &scene);
        let package = learner.snapshot();
        assert_eq!(package.pair_profiles.len(), 1);
        assert!(!package.pair_profiles[0].low_wins.is_empty());
    }

    #[test]
    fn pair_bank_retains_multiple_incompatible_sentence_scenes() {
        let mut learner = OnlineContextPhaseLearner::new(OnlineContextPhaseConfig::production(2));
        let first_scene = phase_vector(73);
        let second_scene = phase_vector(991);
        learner.update_pair_winner(11, 19, &first_scene, false);
        learner.update_pair_winner(11, 19, &second_scene, false);

        let package = learner.snapshot();
        let profile = package.pair_profiles.first().expect("pair profile");
        let winner_bank = if profile.low_hash == 11 {
            &profile.low_wins
        } else {
            &profile.high_wins
        };
        assert_eq!(winner_bank.len(), 2);
    }

    #[test]
    fn incompatible_mode_never_blurs_a_full_bank() {
        let first = phase_vector(11);
        let second = phase_vector(29);
        let mut centers = Vec::new();
        let mut total = 0;
        update_bounded_cluster(&mut centers, &first, 1, &mut total, 1);
        let before = centers[0].clone();

        assert_eq!(
            update_bounded_cluster(&mut centers, &second, 1, &mut total, 1),
            ClusterUpdate::RejectedAtCapacity
        );
        assert_eq!(centers[0], before);
    }

    #[test]
    fn learner_state_is_bounded_and_snapshot_is_sorted() {
        let mut learner = OnlineContextPhaseLearner::new(OnlineContextPhaseConfig {
            min_profile_support: 2,
            signature_schema: super::super::SIGNATURE_SCHEMA_RELATION_ROLES,
            max_semantic_states: 3,
            max_profiles: 2,
            max_positive_phase_centers: 2,
            max_negative_phase_centers: 1,
            max_hard_negative_phase_centers: 1,
        });
        for text in [
            "один два три четыре",
            "пять шесть семь восемь",
            "один два три четыре",
        ] {
            learner.ingest_fragment(&super::super::super::llmwave::tokenize(text));
        }
        let package = learner.snapshot();

        assert!(learner.semantic_state_count() <= 3);
        assert!(learner.profile_count() <= 2);
        assert!(learner.phase_center_count() <= 4);
        assert!(learner.stats().dropped_semantic_states > 0);
        assert!(learner.stats().dropped_profiles > 0);
        assert!(package
            .semantic_states
            .windows(2)
            .all(|items| items[0].token_hash < items[1].token_hash));
        assert!(package
            .profiles
            .windows(2)
            .all(|items| items[0].token_hash < items[1].token_hash));
    }

    #[test]
    fn late_repeated_token_displaces_only_unproven_entries() {
        let mut learner = OnlineContextPhaseLearner::new(OnlineContextPhaseConfig {
            min_profile_support: 2,
            signature_schema: super::super::SIGNATURE_SCHEMA_RELATION_ROLES,
            max_semantic_states: 3,
            max_profiles: 2,
            max_positive_phase_centers: 4,
            max_negative_phase_centers: 2,
            max_hard_negative_phase_centers: 2,
        });
        // Fill the tiny hot-profile budget with one-surface builders. A
        // recurrent later token must be allowed to evict these provisional
        // entries; stable profiles are never evicted by admission.
        assert!(learner.ensure_profile(hash_text("черновик-один"), 2));
        assert!(learner.ensure_profile(hash_text("черновик-два"), 2));
        for text in [
            "один альфа бета",
            "два гамма поздний",
            "три дельта поздний",
            "четыре эпсилон поздний",
        ] {
            learner.ingest_fragment(&super::super::super::llmwave::tokenize(text));
        }
        let package = learner.snapshot();
        let late_hash = hash_text("поздний");

        assert!(learner.stats().evicted_provisional_profiles > 0);
        assert!(learner.stats().evicted_provisional_semantic_states > 0);
        assert!(package
            .profiles
            .iter()
            .any(|profile| profile.token_hash == late_hash));
        assert!(package
            .semantic_states
            .iter()
            .any(|state| state.token_hash == late_hash));
    }

    #[test]
    fn one_off_tokens_stay_in_the_compact_field_until_a_second_surface() {
        let mut learner = OnlineContextPhaseLearner::new(OnlineContextPhaseConfig {
            min_profile_support: 2,
            signature_schema: super::super::SIGNATURE_SCHEMA_RELATION_ROLES,
            max_semantic_states: 16,
            max_profiles: 16,
            max_positive_phase_centers: 16,
            max_negative_phase_centers: 16,
            max_hard_negative_phase_centers: 16,
        });
        let token = hash_text("редкая-форма");

        assert!(!learner.ensure_profile(token, 1));
        assert_eq!(learner.profile_count(), 0);
        assert!(learner.ensure_profile(token, 2));
        assert_eq!(learner.profile_count(), 1);
    }

    #[test]
    fn parallel_probe_pool_returns_receipts_in_input_order() {
        let pool = L2ProbePool::with_worker_count(4);
        let requests = ["дождь", "машина", "работа", "память"]
            .into_iter()
            .map(|target| {
                let context_tokens = super::super::super::llmwave::tokenize("сегодня на улице");
                L2ProbeRequest {
                    context_hashes: super::super::context_atom_hashes(
                        &context_tokens,
                        super::super::SIGNATURE_SCHEMA_RELATION_ROLES,
                    ),
                    context_tokens,
                    target: target.to_string(),
                    target_hash: hash_text(target),
                    target_margin_before_update: Some(0),
                    signature_schema: super::super::SIGNATURE_SCHEMA_RELATION_ROLES,
                    surface_field: Arc::new(SurfaceMutationField::default()),
                }
            })
            .collect::<Vec<_>>();
        let expected = requests
            .iter()
            .map(|request| request.target_hash)
            .collect::<Vec<_>>();

        let receipts = pool.execute_batch(requests).unwrap();

        assert_eq!(
            receipts
                .iter()
                .map(|receipt| receipt.target_hash)
                .collect::<Vec<_>>(),
            expected
        );
    }

    #[test]
    fn one_letter_layout_competition_is_visible_to_l3_learning() {
        let context = super::super::super::llmwave::tokenize("выбрали Apple");
        let latin = l2_lattice_competitors(
            &context,
            "b",
            MAX_COMPETITORS,
            &SurfaceMutationField::default(),
        );
        let cyrillic = l2_lattice_competitors(
            &context,
            "и",
            MAX_COMPETITORS,
            &SurfaceMutationField::default(),
        );

        assert!(latin.iter().any(|candidate| candidate == "и"), "{latin:?}");
        assert!(
            cyrillic.iter().any(|candidate| candidate == "b"),
            "{cyrillic:?}"
        );
    }

    #[test]
    fn l2_training_lattice_deduplicates_case_only_target_surfaces() {
        let surface = SurfaceMutationField::from_corrections_jsonl(
            "{\"from\":\"аллаа\",\"to\":\"аллаха\"}\n\
             {\"from\":\"аллаа\",\"to\":\"аллаха\"}\n",
            2,
        )
        .unwrap();
        let context = super::super::super::llmwave::tokenize("нет бога кроме");
        let competitors = l2_lattice_competitors(&context, "Аллаха", MAX_COMPETITORS, &surface);

        assert!(
            competitors
                .iter()
                .all(|candidate| candidate.to_lowercase() != "аллаха"),
            "{competitors:?}"
        );
    }

    #[test]
    fn semantic_anchor_freezes_when_it_first_becomes_usable() {
        let mut learner = OnlineContextPhaseLearner::new(OnlineContextPhaseConfig {
            min_profile_support: 2,
            signature_schema: super::super::SIGNATURE_SCHEMA_RELATION_ROLES,
            max_semantic_states: 16,
            max_profiles: 8,
            max_positive_phase_centers: 8,
            max_negative_phase_centers: 4,
            max_hard_negative_phase_centers: 4,
        });
        let target = hash_text("центр");
        learner.update_semantic_states(&[hash_text("один"), target, hash_text("два")], &[1, 1, 1]);
        learner
            .update_semantic_states(&[hash_text("три"), target, hash_text("четыре")], &[1, 2, 1]);
        let frozen = learner.semantic.get(&target).unwrap().center.clone();

        learner
            .update_semantic_states(&[hash_text("пять"), target, hash_text("шесть")], &[1, 3, 1]);
        let state = learner.semantic.get(&target).unwrap();
        assert_eq!(state.support, 3);
        assert_eq!(state.center, frozen);
    }

    #[test]
    fn pending_anti_wave_never_grants_authority_and_survives_profile_admission() {
        let mut learner = OnlineContextPhaseLearner::new(OnlineContextPhaseConfig {
            min_profile_support: 2,
            signature_schema: super::super::SIGNATURE_SCHEMA_RELATION_ROLES,
            max_semantic_states: 8,
            max_profiles: 8,
            max_positive_phase_centers: 8,
            max_negative_phase_centers: 4,
            max_hard_negative_phase_centers: 4,
        });
        let competitor_hash = hash_text("ложный");
        let anti_vector = phase_vector(41);

        learner.update_negative_relation(competitor_hash, &anti_vector);

        assert_eq!(learner.pending_negative_profile_count(), 1);
        assert_eq!(learner.pending_negative_center_count(), 1);
        assert!(learner.snapshot().profiles.is_empty());

        assert!(learner.ensure_profile(competitor_hash, 2));
        assert_eq!(learner.pending_negative_profile_count(), 0);
        let profile = learner.profiles.get(&competitor_hash).unwrap();
        assert_eq!(profile.positive_examples, 0);
        assert_eq!(profile.negative_examples, 1);
        assert_eq!(profile.negative.len(), 1);
        assert!(learner.snapshot().profiles.is_empty());

        learner.update_profile_bank(competitor_hash, ProfileBank::Positive, &phase_vector(73));
        learner
            .profiles
            .get_mut(&competitor_hash)
            .unwrap()
            .positive_examples = 2;
        let package = learner.snapshot();
        assert_eq!(package.profiles.len(), 1);
        assert_eq!(package.profiles[0].negative.len(), 1);
    }

    #[test]
    fn full_negative_bank_cannot_starve_positive_subcenters() {
        let mut learner = OnlineContextPhaseLearner::new(OnlineContextPhaseConfig {
            min_profile_support: 2,
            signature_schema: super::super::SIGNATURE_SCHEMA_RELATION_ROLES,
            max_semantic_states: 8,
            max_profiles: 8,
            max_positive_phase_centers: 2,
            max_negative_phase_centers: 1,
            max_hard_negative_phase_centers: 1,
        });
        learner.update_negative_relation(hash_text("чужой"), &phase_vector(7));
        assert_eq!(learner.negative_phase_center_count(), 1);

        let target_hash = hash_text("цель");
        assert!(learner.ensure_profile(target_hash, 2));
        assert!(learner.update_profile_bank(target_hash, ProfileBank::Positive, &phase_vector(17)));
        assert!(learner.update_profile_bank(target_hash, ProfileBank::Positive, &phase_vector(31)));

        assert_eq!(learner.positive_phase_center_count(), 2);
        assert_eq!(learner.negative_phase_center_count(), 1);
    }

    #[test]
    fn generic_negative_is_calibration_only_and_hard_evidence_is_pairwise() {
        let mut learner = OnlineContextPhaseLearner::new(OnlineContextPhaseConfig {
            min_profile_support: 2,
            signature_schema: super::super::SIGNATURE_SCHEMA_RELATION_ROLES,
            max_semantic_states: 8,
            max_profiles: 8,
            max_positive_phase_centers: 8,
            max_negative_phase_centers: 8,
            max_hard_negative_phase_centers: 8,
        });
        let target = hash_text("кандидат");
        let vector = phase_vector(71);
        assert!(learner.ensure_profile(target, 2));
        assert!(learner.update_profile_bank(target, ProfileBank::Positive, &vector));
        assert!(learner.update_profile_bank(target, ProfileBank::Positive, &vector));

        let positive_margin = learner.profile_margin_micro(target, &vector).unwrap();
        assert!(learner.update_profile_bank(target, ProfileBank::Negative, &vector));
        assert_eq!(
            learner.profile_margin_micro(target, &vector),
            Some(positive_margin),
            "one ordinary L2 alternative is calibration, not destructive authority"
        );

        assert!(learner.update_profile_bank(target, ProfileBank::Negative, &vector));
        assert_eq!(
            learner.profile_margin_micro(target, &vector),
            Some(positive_margin),
            "generic competition remains calibration-only even when repeated"
        );

        let competitor = hash_text("ложный_победитель");
        learner.update_pair_winner(target, competitor, &vector, true);
        let package = learner.snapshot();
        let pair = package.pair_profiles.first().expect("hard pair profile");
        assert!(
            !pair.hard_low_wins.is_empty() || !pair.hard_high_wins.is_empty(),
            "a witnessed false winner is destructive only with its target PairKey"
        );
    }

    #[test]
    fn l2_competitor_lattice_is_deterministic_and_never_contains_the_teacher_target() {
        let context = vec!["на".to_string(), "улице".to_string(), "идет".to_string()];
        let field = SurfaceMutationField::from_corrections_jsonl(
            concat!(
                r#"{"from":"дожь","to":"дождь"}"#,
                "\n",
                r#"{"from":"день","to":"дней"}"#,
            ),
            1,
        )
        .unwrap();
        let first = l2_lattice_competitors(&context, "дождь", 4, &field);
        let second = l2_lattice_competitors(&context, "дождь", 4, &field);

        assert_eq!(first, second);
        assert!(first.len() <= 4);
        assert!(first.iter().all(|candidate| candidate != "дождь"));
    }

    #[test]
    fn real_l2_probe_keeps_standalone_survivor_order_without_teacher_ranking() {
        let mut observed = Vec::new();
        let lattice = real_l2_lattice_competitors(
            "на улице",
            "дождь",
            vec!["дожь".to_string(), "дожд".to_string()],
            4,
            |context, damaged| {
                observed.push((context.to_string(), damaged.to_string()));
                match damaged {
                    "дожь" => vec![
                        "дожди".to_string(),
                        "дождь".to_string(),
                        "дождя".to_string(),
                    ],
                    "дожд" => vec!["дождя".to_string(), "дождик".to_string()],
                    _ => Vec::new(),
                }
            },
        );

        assert_eq!(
            observed,
            vec![
                ("на улице".to_string(), "дожь".to_string()),
                ("на улице".to_string(), "дожд".to_string()),
            ]
        );
        assert_eq!(lattice.competitors, ["дожди", "дождя", "дождик"]);
        assert!(lattice.target_retained);
    }
}

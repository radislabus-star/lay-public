//! Bounded single-flight cache for immutable L1.1 -> Productive V90 fields.
//!
//! Text materialization, boundary candidates, L3/L4 evidence and final
//! mutation authority stay outside this cache.

use std::collections::{HashMap, VecDeque};
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::{Arc, Condvar, Mutex, OnceLock};

use sha2::{Digest, Sha256};

use super::productive_v1::PreparedCanonicalTokenField;
use crate::nanda_wave::L11SeedSurface;

const CANONICAL_FIELD_SCHEMA_VERSION: u16 = 2;
const CANONICAL_FIELD_READY_LIMIT: usize = 128;
const CANONICAL_FIELD_IN_FLIGHT_LIMIT: usize = 32;
const CANONICAL_FIELD_WAITERS_PER_KEY_LIMIT: usize = 8;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(super) struct CanonicalTokenKey {
    scene_bytes: Vec<u8>,
    contour_identity_sha256: [u8; 32],
    l11_seed_lattice_sha256: [u8; 32],
    l11_package_sha256: [u8; 32],
    canonical_l2_package_sha256: [u8; 32],
    productive_package_sha256: [u8; 32],
    field_schema_version: u16,
}

impl CanonicalTokenKey {
    pub(super) fn new(
        scene_bytes: Vec<u8>,
        contour_identity: &[u8],
        l11_seeds: &[L11SeedSurface],
        l11_package_sha256: [u8; 32],
        canonical_l2_package_sha256: [u8; 32],
        productive_package_sha256: [u8; 32],
    ) -> Self {
        Self {
            scene_bytes,
            contour_identity_sha256: sha256_bytes(
                b"lay-canonical-contour-identity-v1\0",
                contour_identity,
            ),
            l11_seed_lattice_sha256: l11_seed_lattice_sha256(l11_seeds),
            l11_package_sha256,
            canonical_l2_package_sha256,
            productive_package_sha256,
            field_schema_version: CANONICAL_FIELD_SCHEMA_VERSION,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum FieldCacheDisposition {
    Produced,
    Waited,
    ReadyHit,
}

#[derive(Debug)]
pub(super) struct FieldCacheReadout {
    pub(super) field: Arc<PreparedCanonicalTokenField>,
    pub(super) disposition: FieldCacheDisposition,
    pub(super) generation: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum FieldCacheError {
    Saturated,
    Superseded,
    PrepareFailed(String),
    Poisoned,
}

impl std::fmt::Display for FieldCacheError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Saturated => formatter.write_str("canonical field cache is saturated"),
            Self::Superseded => formatter.write_str("canonical field generation was superseded"),
            Self::PrepareFailed(error) => {
                write!(formatter, "canonical field preparation failed: {error}")
            }
            Self::Poisoned => formatter.write_str("canonical field cache lock is poisoned"),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct FieldCacheStatsSnapshot {
    pub(super) generation: u64,
    pub(super) ready_entries: u64,
    pub(super) in_flight_entries: u64,
    pub(super) producer_computations: u64,
    pub(super) waiter_joins: u64,
    pub(super) ready_hits: u64,
    pub(super) transient_failures: u64,
    pub(super) superseded_publications: u64,
    pub(super) saturated_requests: u64,
    pub(super) invalidations: u64,
}

impl FieldCacheStatsSnapshot {
    pub(super) fn delta(self, before: Self) -> Self {
        Self {
            generation: self.generation,
            ready_entries: self.ready_entries,
            in_flight_entries: self.in_flight_entries,
            producer_computations: self
                .producer_computations
                .saturating_sub(before.producer_computations),
            waiter_joins: self.waiter_joins.saturating_sub(before.waiter_joins),
            ready_hits: self.ready_hits.saturating_sub(before.ready_hits),
            transient_failures: self
                .transient_failures
                .saturating_sub(before.transient_failures),
            superseded_publications: self
                .superseded_publications
                .saturating_sub(before.superseded_publications),
            saturated_requests: self
                .saturated_requests
                .saturating_sub(before.saturated_requests),
            invalidations: self.invalidations.saturating_sub(before.invalidations),
        }
    }
}

pub(super) fn get_or_prepare(
    key: CanonicalTokenKey,
    prepare: impl FnOnce() -> Result<PreparedCanonicalTokenField, String>,
) -> Result<FieldCacheReadout, FieldCacheError> {
    canonical_field_cache().get_or_prepare_field(key, prepare)
}

pub(super) fn clear() {
    canonical_field_cache().invalidate();
}

pub(super) fn stats() -> FieldCacheStatsSnapshot {
    canonical_field_cache().stats()
}

pub(super) fn generation_is_current(generation: u64) -> bool {
    canonical_field_cache().generation_is_current(generation)
}

fn canonical_field_cache() -> &'static SingleFlightCache<PreparedCanonicalTokenField> {
    static CACHE: OnceLock<SingleFlightCache<PreparedCanonicalTokenField>> = OnceLock::new();
    CACHE.get_or_init(|| {
        SingleFlightCache::new(
            CANONICAL_FIELD_READY_LIMIT,
            CANONICAL_FIELD_IN_FLIGHT_LIMIT,
            CANONICAL_FIELD_WAITERS_PER_KEY_LIMIT,
        )
    })
}

struct SingleFlightCache<T> {
    ready_limit: usize,
    in_flight_limit: usize,
    waiter_limit: usize,
    state: Mutex<CacheState<T>>,
}

struct CacheState<T> {
    generation: u64,
    ready: VecDeque<ReadyEntry<T>>,
    in_flight: HashMap<CanonicalTokenKey, InFlightEntry<T>>,
    stats: FieldCacheStatsSnapshot,
}

struct ReadyEntry<T> {
    key: CanonicalTokenKey,
    value: Arc<T>,
}

struct InFlightEntry<T> {
    flight: Arc<Flight<T>>,
    waiters_joined: usize,
}

struct Flight<T> {
    generation: u64,
    state: Mutex<FlightState<T>>,
    done: Condvar,
}

enum FlightState<T> {
    Computing,
    Ready(Arc<T>),
    Failed(String),
    Superseded,
}

enum CacheRole<T> {
    Producer(Arc<Flight<T>>),
    Waiter(Arc<Flight<T>>),
}

impl<T> SingleFlightCache<T> {
    fn new(ready_limit: usize, in_flight_limit: usize, waiter_limit: usize) -> Self {
        assert!(ready_limit > 0);
        assert!(in_flight_limit > 0);
        assert!(waiter_limit > 0);
        let generation = 1;
        Self {
            ready_limit,
            in_flight_limit,
            waiter_limit,
            state: Mutex::new(CacheState {
                generation,
                ready: VecDeque::new(),
                in_flight: HashMap::new(),
                stats: FieldCacheStatsSnapshot {
                    generation,
                    ..FieldCacheStatsSnapshot::default()
                },
            }),
        }
    }

    fn get_or_prepare(
        &self,
        key: CanonicalTokenKey,
        prepare: impl FnOnce() -> Result<T, String>,
    ) -> Result<FieldCacheReadoutGeneric<T>, FieldCacheError> {
        let role = {
            let mut state = self.state.lock().map_err(|_| FieldCacheError::Poisoned)?;
            if let Some(index) = state.ready.iter().position(|entry| entry.key == key) {
                let entry = state
                    .ready
                    .remove(index)
                    .expect("ready cache index was just observed");
                let value = entry.value.clone();
                state.ready.push_back(entry);
                state.stats.ready_hits = state.stats.ready_hits.saturating_add(1);
                return Ok(FieldCacheReadoutGeneric {
                    value,
                    disposition: FieldCacheDisposition::ReadyHit,
                    generation: state.generation,
                });
            }
            if state.in_flight.contains_key(&key) {
                let waiter_limit_reached = state
                    .in_flight
                    .get(&key)
                    .is_some_and(|entry| entry.waiters_joined >= self.waiter_limit);
                if waiter_limit_reached {
                    state.stats.saturated_requests =
                        state.stats.saturated_requests.saturating_add(1);
                    return Err(FieldCacheError::Saturated);
                }
                let flight = {
                    let entry = state
                        .in_flight
                        .get_mut(&key)
                        .expect("in-flight key was just observed");
                    entry.waiters_joined += 1;
                    entry.flight.clone()
                };
                state.stats.waiter_joins = state.stats.waiter_joins.saturating_add(1);
                CacheRole::Waiter(flight)
            } else {
                if state.in_flight.len() >= self.in_flight_limit {
                    state.stats.saturated_requests =
                        state.stats.saturated_requests.saturating_add(1);
                    return Err(FieldCacheError::Saturated);
                }
                let flight = Arc::new(Flight {
                    generation: state.generation,
                    state: Mutex::new(FlightState::Computing),
                    done: Condvar::new(),
                });
                state.in_flight.insert(
                    key.clone(),
                    InFlightEntry {
                        flight: flight.clone(),
                        waiters_joined: 0,
                    },
                );
                state.stats.producer_computations =
                    state.stats.producer_computations.saturating_add(1);
                CacheRole::Producer(flight)
            }
        };

        match role {
            CacheRole::Waiter(flight) => self.wait_for_flight(flight),
            CacheRole::Producer(flight) => {
                let result = match catch_unwind(AssertUnwindSafe(prepare)) {
                    Ok(result) => result,
                    Err(_) => Err("field producer panicked".to_string()),
                };
                self.publish(key, flight, result)
            }
        }
    }

    fn wait_for_flight(
        &self,
        flight: Arc<Flight<T>>,
    ) -> Result<FieldCacheReadoutGeneric<T>, FieldCacheError> {
        let mut state = flight.state.lock().map_err(|_| FieldCacheError::Poisoned)?;
        loop {
            match &*state {
                FlightState::Computing => {
                    state = flight
                        .done
                        .wait(state)
                        .map_err(|_| FieldCacheError::Poisoned)?;
                }
                FlightState::Ready(value) => {
                    return Ok(FieldCacheReadoutGeneric {
                        value: value.clone(),
                        disposition: FieldCacheDisposition::Waited,
                        generation: flight.generation,
                    });
                }
                FlightState::Failed(error) => {
                    return Err(FieldCacheError::PrepareFailed(error.clone()));
                }
                FlightState::Superseded => return Err(FieldCacheError::Superseded),
            }
        }
    }

    fn publish(
        &self,
        key: CanonicalTokenKey,
        flight: Arc<Flight<T>>,
        result: Result<T, String>,
    ) -> Result<FieldCacheReadoutGeneric<T>, FieldCacheError> {
        let mut state = self.state.lock().map_err(|_| {
            complete_flight(
                &flight,
                FlightState::Failed("cache lock poisoned".to_string()),
            );
            FieldCacheError::Poisoned
        })?;
        let owns_slot = state
            .in_flight
            .get(&key)
            .is_some_and(|entry| Arc::ptr_eq(&entry.flight, &flight));
        if !owns_slot || state.generation != flight.generation {
            if owns_slot {
                state.in_flight.remove(&key);
            }
            state.stats.superseded_publications =
                state.stats.superseded_publications.saturating_add(1);
            drop(state);
            complete_flight(&flight, FlightState::Superseded);
            return Err(FieldCacheError::Superseded);
        }
        state.in_flight.remove(&key);

        match result {
            Ok(value) => {
                let value = Arc::new(value);
                if let Some(index) = state.ready.iter().position(|entry| entry.key == key) {
                    state.ready.remove(index);
                }
                state.ready.push_back(ReadyEntry {
                    key,
                    value: value.clone(),
                });
                while state.ready.len() > self.ready_limit {
                    state.ready.pop_front();
                }
                let generation = state.generation;
                complete_flight(&flight, FlightState::Ready(value.clone()));
                drop(state);
                Ok(FieldCacheReadoutGeneric {
                    value,
                    disposition: FieldCacheDisposition::Produced,
                    generation,
                })
            }
            Err(error) => {
                state.stats.transient_failures = state.stats.transient_failures.saturating_add(1);
                complete_flight(&flight, FlightState::Failed(error.clone()));
                drop(state);
                Err(FieldCacheError::PrepareFailed(error))
            }
        }
    }

    fn invalidate(&self) {
        let Ok(mut state) = self.state.lock() else {
            return;
        };
        state.generation = state.generation.wrapping_add(1).max(1);
        state.stats.generation = state.generation;
        state.stats.invalidations = state.stats.invalidations.saturating_add(1);
        state.ready.clear();
        let flights = state
            .in_flight
            .drain()
            .map(|(_, entry)| entry.flight)
            .collect::<Vec<_>>();
        drop(state);
        for flight in flights {
            complete_flight(&flight, FlightState::Superseded);
        }
    }

    fn stats(&self) -> FieldCacheStatsSnapshot {
        let Ok(state) = self.state.lock() else {
            return FieldCacheStatsSnapshot::default();
        };
        FieldCacheStatsSnapshot {
            generation: state.generation,
            ready_entries: state.ready.len() as u64,
            in_flight_entries: state.in_flight.len() as u64,
            ..state.stats
        }
    }

    fn generation_is_current(&self, generation: u64) -> bool {
        self.state
            .lock()
            .is_ok_and(|state| state.generation == generation)
    }
}

#[derive(Debug)]
struct FieldCacheReadoutGeneric<T> {
    value: Arc<T>,
    disposition: FieldCacheDisposition,
    generation: u64,
}

impl SingleFlightCache<PreparedCanonicalTokenField> {
    fn get_or_prepare_field(
        &self,
        key: CanonicalTokenKey,
        prepare: impl FnOnce() -> Result<PreparedCanonicalTokenField, String>,
    ) -> Result<FieldCacheReadout, FieldCacheError> {
        let readout = self.get_or_prepare(key, prepare)?;
        Ok(FieldCacheReadout {
            field: readout.value,
            disposition: readout.disposition,
            generation: readout.generation,
        })
    }
}

fn complete_flight<T>(flight: &Flight<T>, completed: FlightState<T>) {
    if let Ok(mut state) = flight.state.lock() {
        if matches!(&*state, FlightState::Computing) {
            *state = completed;
        }
    }
    flight.done.notify_all();
}

fn l11_seed_lattice_sha256(seeds: &[L11SeedSurface]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(b"lay-l11-seed-lattice-v1\0");
    hasher.update((seeds.len() as u64).to_le_bytes());
    for seed in seeds {
        match seed.terminal_id {
            Some(terminal_id) => {
                hasher.update([1]);
                hasher.update(terminal_id.to_le_bytes());
            }
            None => hasher.update([0]),
        }
        hasher.update((seed.surface.len() as u64).to_le_bytes());
        hasher.update(seed.surface.as_bytes());
        hasher.update([u8::from(seed.authority)]);
        hasher.update(seed.score_milli.to_le_bytes());
    }
    hasher.finalize().into()
}

fn sha256_bytes(domain: &[u8], bytes: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(domain);
    hasher.update((bytes.len() as u64).to_le_bytes());
    hasher.update(bytes);
    hasher.finalize().into()
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::mpsc;
    use std::thread;

    use super::*;

    fn key(id: u8) -> CanonicalTokenKey {
        CanonicalTokenKey {
            scene_bytes: vec![id],
            contour_identity_sha256: [id.wrapping_add(1); 32],
            l11_seed_lattice_sha256: [id; 32],
            l11_package_sha256: [1; 32],
            canonical_l2_package_sha256: [2; 32],
            productive_package_sha256: [3; 32],
            field_schema_version: CANONICAL_FIELD_SCHEMA_VERSION,
        }
    }

    fn seed(terminal_id: u32, surface: &str) -> L11SeedSurface {
        L11SeedSurface {
            terminal_id: Some(terminal_id),
            surface: surface.to_string(),
            authority: false,
            score_milli: 700,
        }
    }

    #[test]
    fn canonical_key_covers_scene_seed_order_and_all_package_identities() {
        let seeds = vec![seed(1, "форма"), seed(2, "формы")];
        let baseline =
            CanonicalTokenKey::new(vec![9], b"contour-a", &seeds, [1; 32], [2; 32], [3; 32]);
        let reversed = seeds.iter().cloned().rev().collect::<Vec<_>>();

        assert_ne!(
            baseline,
            CanonicalTokenKey::new(vec![8], b"contour-a", &seeds, [1; 32], [2; 32], [3; 32])
        );
        assert_ne!(
            baseline,
            CanonicalTokenKey::new(vec![9], b"contour-b", &seeds, [1; 32], [2; 32], [3; 32])
        );
        assert_ne!(
            baseline,
            CanonicalTokenKey::new(vec![9], b"contour-a", &reversed, [1; 32], [2; 32], [3; 32])
        );
        assert_ne!(
            baseline,
            CanonicalTokenKey::new(vec![9], b"contour-a", &seeds, [4; 32], [2; 32], [3; 32])
        );
        assert_ne!(
            baseline,
            CanonicalTokenKey::new(vec![9], b"contour-a", &seeds, [1; 32], [4; 32], [3; 32])
        );
        assert_ne!(
            baseline,
            CanonicalTokenKey::new(vec![9], b"contour-a", &seeds, [1; 32], [2; 32], [4; 32])
        );
    }

    #[test]
    fn simultaneous_requests_execute_one_field_producer() {
        let cache = Arc::new(SingleFlightCache::<usize>::new(4, 2, 2));
        let calls = Arc::new(AtomicUsize::new(0));
        let (started_tx, started_rx) = mpsc::channel();
        let (release_tx, release_rx) = mpsc::channel();

        let producer_cache = cache.clone();
        let producer_calls = calls.clone();
        let producer = thread::spawn(move || {
            producer_cache.get_or_prepare(key(7), || {
                producer_calls.fetch_add(1, Ordering::SeqCst);
                started_tx.send(()).expect("announce producer");
                release_rx.recv().expect("release producer");
                Ok(41)
            })
        });
        started_rx.recv().expect("producer started");

        let waiter_cache = cache.clone();
        let waiter_calls = calls.clone();
        let waiter = thread::spawn(move || {
            waiter_cache.get_or_prepare(key(7), || {
                waiter_calls.fetch_add(1, Ordering::SeqCst);
                Ok(99)
            })
        });
        while cache.stats().waiter_joins == 0 {
            thread::yield_now();
        }
        release_tx.send(()).expect("release producer");

        let producer = producer.join().expect("producer thread").expect("producer");
        let waiter = waiter.join().expect("waiter thread").expect("waiter");
        assert_eq!(calls.load(Ordering::SeqCst), 1);
        assert_eq!(*producer.value, 41);
        assert_eq!(*waiter.value, 41);
        assert!(Arc::ptr_eq(&producer.value, &waiter.value));
        assert_eq!(producer.disposition, FieldCacheDisposition::Produced);
        assert_eq!(waiter.disposition, FieldCacheDisposition::Waited);
    }

    #[test]
    fn transient_failure_is_removed_and_retried() {
        let cache = SingleFlightCache::<usize>::new(4, 2, 2);
        let first = cache.get_or_prepare(key(1), || Err("temporary L1 failure".to_string()));
        assert_eq!(
            first.expect_err("transient failure"),
            FieldCacheError::PrepareFailed("temporary L1 failure".to_string())
        );

        let second = cache
            .get_or_prepare(key(1), || Ok(17))
            .expect("retry after transient failure");
        assert_eq!(*second.value, 17);
        assert_eq!(second.disposition, FieldCacheDisposition::Produced);
        let stats = cache.stats();
        assert_eq!(stats.producer_computations, 2);
        assert_eq!(stats.transient_failures, 1);
        assert_eq!(stats.in_flight_entries, 0);
    }

    #[test]
    fn old_generation_cannot_publish_after_invalidation() {
        let cache = Arc::new(SingleFlightCache::<usize>::new(4, 2, 2));
        let (started_tx, started_rx) = mpsc::channel();
        let (release_tx, release_rx) = mpsc::channel();
        let producer_cache = cache.clone();
        let producer = thread::spawn(move || {
            producer_cache.get_or_prepare(key(3), || {
                started_tx.send(()).expect("announce old generation");
                release_rx.recv().expect("release old generation");
                Ok(31)
            })
        });
        started_rx.recv().expect("old generation started");
        cache.invalidate();
        release_tx.send(()).expect("release old generation");

        assert_eq!(
            producer
                .join()
                .expect("old generation thread")
                .expect_err("old generation must be rejected"),
            FieldCacheError::Superseded
        );
        let current = cache
            .get_or_prepare(key(3), || Ok(32))
            .expect("new generation");
        assert_eq!(*current.value, 32);
        assert_eq!(current.generation, 2);
        assert_eq!(cache.stats().superseded_publications, 1);
    }

    #[test]
    fn ready_lru_and_in_flight_frontiers_are_bounded() {
        let cache = Arc::new(SingleFlightCache::<usize>::new(2, 1, 2));
        cache.get_or_prepare(key(1), || Ok(1)).expect("key one");
        cache.get_or_prepare(key(2), || Ok(2)).expect("key two");
        cache.get_or_prepare(key(3), || Ok(3)).expect("key three");
        assert_eq!(cache.stats().ready_entries, 2);

        let (started_tx, started_rx) = mpsc::channel();
        let (release_tx, release_rx) = mpsc::channel();
        let producer_cache = cache.clone();
        let producer = thread::spawn(move || {
            producer_cache.get_or_prepare(key(4), || {
                started_tx.send(()).expect("announce bounded producer");
                release_rx.recv().expect("release bounded producer");
                Ok(4)
            })
        });
        started_rx.recv().expect("bounded producer started");
        assert_eq!(
            cache
                .get_or_prepare(key(5), || Ok(5))
                .expect_err("second in-flight key must be rejected"),
            FieldCacheError::Saturated
        );
        release_tx.send(()).expect("release bounded producer");
        producer
            .join()
            .expect("bounded producer thread")
            .expect("bounded producer result");
        assert_eq!(cache.stats().ready_entries, 2);
        assert_eq!(cache.stats().saturated_requests, 1);
    }
}

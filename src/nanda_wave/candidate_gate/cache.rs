//! Bounded LRU cache for completed live readouts.
//!
//! This cache stores already-authorized display candidates. It has no access to
//! L2/L3/L4 scoring or `TransitionDecisionCore`, so a cache hit cannot invent
//! a new decision route.

use super::LiveCompletionCandidate;
use std::collections::VecDeque;
use std::sync::{Mutex, OnceLock};

const LIVE_COMPLETION_CACHE_LIMIT: usize = 128;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct LiveCompletionCacheKey {
    pub(super) identity: CacheIdentity,
    pub(super) context_tail: String,
    pub(super) partial: String,
    pub(super) max_suffix_chars: usize,
    pub(super) active_composition: bool,
    pub(super) allow_short_lexical: bool,
    pub(super) limit: usize,
}

#[derive(Debug, Clone)]
struct LiveCompletionCacheEntry {
    key: LiveCompletionCacheKey,
    candidates: Vec<LiveCompletionCandidate>,
}

/// Material dependencies, independent of the client's publication authority.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct CacheIdentity {
    material_generation: u64,
    revision: u64,
}

#[derive(Default)]
struct LiveCompletionCache {
    revision: u64,
    entries: VecDeque<LiveCompletionCacheEntry>,
}

impl LiveCompletionCache {
    fn clear(&mut self) {
        self.revision = self.revision.wrapping_add(1);
        self.entries.clear();
    }

    fn get(&mut self, key: &LiveCompletionCacheKey) -> Option<Vec<LiveCompletionCandidate>> {
        if key.identity.revision != self.revision {
            return None;
        }
        let index = self.entries.iter().position(|entry| &entry.key == key)?;
        let entry = self.entries.remove(index)?;
        let candidates = entry.candidates.clone();
        self.entries.push_back(entry);
        Some(candidates)
    }

    fn store(&mut self, key: LiveCompletionCacheKey, candidates: &[LiveCompletionCandidate]) {
        if key.identity.revision != self.revision {
            return;
        }
        if let Some(index) = self.entries.iter().position(|entry| entry.key == key) {
            self.entries.remove(index);
        }
        self.entries.push_back(LiveCompletionCacheEntry {
            key,
            candidates: candidates.to_vec(),
        });
        while self.entries.len() > LIVE_COMPLETION_CACHE_LIMIT {
            self.entries.pop_front();
        }
    }
}

pub(super) fn identity() -> Option<CacheIdentity> {
    // Never acquire a dependency lock while holding the readout-cache mutex.
    let material_generation = super::super::l2_field::candidate_material_generation();
    let revision = live_completion_cache().lock().ok()?.revision;
    Some(CacheIdentity {
        material_generation,
        revision,
    })
}

pub(super) fn identity_is_current(expected: CacheIdentity) -> bool {
    identity() == Some(expected)
        && super::super::l2_field::candidate_material_generation() == expected.material_generation
}

pub(super) fn get(key: &LiveCompletionCacheKey) -> Option<Vec<LiveCompletionCandidate>> {
    if !identity_is_current(key.identity) {
        return None;
    }
    let candidates = live_completion_cache().lock().ok()?.get(key)?;
    identity_is_current(key.identity).then_some(candidates)
}

pub(super) fn store(key: LiveCompletionCacheKey, candidates: &[LiveCompletionCandidate]) {
    if !identity_is_current(key.identity) {
        return;
    }
    if let Ok(mut cache) = live_completion_cache().lock() {
        cache.store(key, candidates);
    }
}

pub(super) fn clear() {
    if let Ok(mut cache) = live_completion_cache().lock() {
        cache.clear();
    }
}

fn live_completion_cache() -> &'static Mutex<LiveCompletionCache> {
    static CACHE: OnceLock<Mutex<LiveCompletionCache>> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(LiveCompletionCache::default()))
}

#[cfg(test)]
pub(super) fn revision_for_tests() -> u64 {
    live_completion_cache().lock().unwrap().revision
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key() -> LiveCompletionCacheKey {
        LiveCompletionCacheKey {
            identity: identity().unwrap(),
            context_tail: String::new(),
            partial: "про".into(),
            max_suffix_chars: 16,
            active_composition: true,
            allow_short_lexical: true,
            limit: 12,
        }
    }

    #[test]
    fn inflight_completed_readout_cannot_repopulate_after_clear() {
        let before = key();
        store(before.clone(), &[]);
        assert_eq!(get(&before), Some(Vec::new()));
        let (started_tx, started_rx) = std::sync::mpsc::channel();
        let (resume_tx, resume_rx) = std::sync::mpsc::channel();
        let stale = before.clone();
        let worker = std::thread::spawn(move || {
            started_tx.send(()).unwrap();
            resume_rx.recv().unwrap();
            store(stale.clone(), &[]);
            assert!(!identity_is_current(stale.identity));
        });
        started_rx.recv().unwrap();
        clear();
        let after = key();
        assert_ne!(after.identity, before.identity);
        store(after.clone(), &[]);
        resume_tx.send(()).unwrap();
        worker.join().unwrap();
        assert_eq!(get(&before), None);
        assert_eq!(get(&after), Some(Vec::new()));
    }

    #[test]
    fn material_generation_is_part_of_completed_readout_identity() {
        let current = key();
        store(current.clone(), &[]);
        let mut stale = current.clone();
        stale.identity.material_generation = stale.identity.material_generation.wrapping_sub(1);
        assert!(!identity_is_current(stale.identity));
        assert_eq!(get(&stale), None);
        store(stale.clone(), &[]);
        assert_eq!(get(&stale), None);
        assert_eq!(get(&current), Some(Vec::new()));
    }
}

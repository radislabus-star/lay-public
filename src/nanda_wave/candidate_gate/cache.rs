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

pub(super) fn get(key: &LiveCompletionCacheKey) -> Option<Vec<LiveCompletionCandidate>> {
    let Ok(mut cache) = live_completion_cache().lock() else {
        return None;
    };
    let index = cache.iter().position(|entry| &entry.key == key)?;
    let entry = cache.remove(index)?;
    let candidates = entry.candidates.clone();
    cache.push_back(entry);
    Some(candidates)
}

pub(super) fn store(key: LiveCompletionCacheKey, candidates: &[LiveCompletionCandidate]) {
    let Ok(mut cache) = live_completion_cache().lock() else {
        return;
    };
    if let Some(index) = cache.iter().position(|entry| entry.key == key) {
        cache.remove(index);
    }
    cache.push_back(LiveCompletionCacheEntry {
        key,
        candidates: candidates.to_vec(),
    });
    while cache.len() > LIVE_COMPLETION_CACHE_LIMIT {
        cache.pop_front();
    }
}

fn live_completion_cache() -> &'static Mutex<VecDeque<LiveCompletionCacheEntry>> {
    static CACHE: OnceLock<Mutex<VecDeque<LiveCompletionCacheEntry>>> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(VecDeque::new()))
}

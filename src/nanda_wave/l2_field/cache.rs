//! Bounded cache for the immutable L1.1 -> canonical L2 readout.
//!
//! L3/L4 evidence and the final decision are intentionally outside this cache:
//! online deltas must be observed on every request.

use std::collections::VecDeque;
use std::sync::{Mutex, OnceLock};

use super::runtime::CanonicalL2FieldReadout;

const CANONICAL_L2_READOUT_CACHE_LIMIT: usize = 128;

#[derive(Clone)]
struct CanonicalL2ReadoutCacheEntry {
    original: String,
    readout: CanonicalL2FieldReadout,
}

pub(super) fn get(original: &str) -> Option<CanonicalL2FieldReadout> {
    let Ok(mut cache) = canonical_l2_readout_cache().lock() else {
        return None;
    };
    let index = cache.iter().position(|entry| entry.original == original)?;
    let entry = cache.remove(index)?;
    let readout = entry.readout.clone();
    cache.push_back(entry);
    Some(readout)
}

pub(super) fn store(original: &str, readout: &CanonicalL2FieldReadout) {
    let Ok(mut cache) = canonical_l2_readout_cache().lock() else {
        return;
    };
    if let Some(index) = cache.iter().position(|entry| entry.original == original) {
        cache.remove(index);
    }
    cache.push_back(CanonicalL2ReadoutCacheEntry {
        original: original.to_string(),
        readout: readout.clone(),
    });
    while cache.len() > CANONICAL_L2_READOUT_CACHE_LIMIT {
        cache.pop_front();
    }
}

pub(super) fn clear() {
    if let Ok(mut cache) = canonical_l2_readout_cache().lock() {
        cache.clear();
    }
}

fn canonical_l2_readout_cache() -> &'static Mutex<VecDeque<CanonicalL2ReadoutCacheEntry>> {
    static CACHE: OnceLock<Mutex<VecDeque<CanonicalL2ReadoutCacheEntry>>> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(VecDeque::new()))
}

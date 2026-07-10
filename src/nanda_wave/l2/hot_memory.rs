//! Hot L2 memory lifecycle and status.
//!
//! This module owns warmup and observability for the compact surface/center
//! runtime. Candidate generation stays in the L2 facade; cold corpus material
//! is never consulted here as runtime authority.

use super::{
    broad_prefix_index, l2_short_position_seed_index, surface_motif_memory, BROAD_PREFIX_INDEX,
    L2_ACTIVE_SOURCE_TARGET, L2_FOUNDATION_LIVE_SCAN_LIMIT, L2_FOUNDATION_SOURCE_LIMIT,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct L2SurfaceMemoryStatus {
    pub active_source_target: usize,
    pub hot_center_words: usize,
    pub hot_center_records: usize,
    pub hot_center_motifs: usize,
    pub hot_center_token_refs: usize,
    pub hot_center_bytes: usize,
    pub broad_source_words: usize,
    pub broad_prefix_keys: usize,
    pub broad_word_refs: usize,
    pub decoder_source_words: usize,
    pub decoder_states: usize,
    pub decoder_arcs: usize,
    pub decoder_hot_bytes: usize,
    pub foundation_source_limit: usize,
    pub foundation_live_scan_limit: usize,
    pub generated_forms_loaded: bool,
    pub generated_forms_words: usize,
}

pub(crate) fn warm_up_surface_motif_memory() {
    let _ = surface_motif_memory().center_count();
}

pub(crate) fn warm_up_ime_word_candidate_memory() {
    // Live IME must not pay cold OnceLock construction during the first word.
    crate::lexicon::warm_up_for_ime();
    super::super::l2_surface_decoder::warm_up();
    let _ = broad_prefix_index().stats();
    let _ = l2_short_position_seed_index().len();
}

pub fn ime_word_candidate_memory_is_warm() -> bool {
    super::super::l2_surface_decoder::is_warm() && BROAD_PREFIX_INDEX.get().is_some()
}

pub fn l2_surface_memory_status() -> L2SurfaceMemoryStatus {
    let hot = surface_motif_memory();
    let broad = broad_prefix_index().stats();
    let decoder = super::super::l2_surface_decoder::stats();
    let generated_forms_loaded =
        crate::russian_lexicon::russian_generated_form_dictionary_is_warm();
    let generated_forms_words = if generated_forms_loaded {
        crate::russian_lexicon::russian_generated_form_dictionary().len()
    } else {
        0
    };
    L2SurfaceMemoryStatus {
        active_source_target: L2_ACTIVE_SOURCE_TARGET,
        hot_center_words: hot.source_word_count(),
        hot_center_records: hot.word_records().len(),
        hot_center_motifs: hot.center_count(),
        hot_center_token_refs: hot.token_refs().len(),
        hot_center_bytes: hot.hot_bytes(),
        broad_source_words: broad.source_words,
        broad_prefix_keys: broad.prefix_keys,
        broad_word_refs: broad.word_refs,
        decoder_source_words: decoder.source_words,
        decoder_states: decoder.states,
        decoder_arcs: decoder.arcs,
        decoder_hot_bytes: decoder.hot_bytes,
        foundation_source_limit: L2_FOUNDATION_SOURCE_LIMIT,
        foundation_live_scan_limit: L2_FOUNDATION_LIVE_SCAN_LIMIT,
        generated_forms_loaded,
        generated_forms_words,
    }
}

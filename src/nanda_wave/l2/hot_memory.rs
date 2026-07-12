//! Hot L2 memory lifecycle and status.
//!
//! This module owns warmup and observability for the compact surface/center
//! runtime. Candidate generation stays in the L2 facade; cold corpus material
//! is never consulted here as runtime authority.

use super::{
    surface_motif_memory, L2_ACTIVE_SOURCE_TARGET, L2_FOUNDATION_SOURCE_LIMIT, SURFACE_MOTIF_MEMORY,
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
    warm_up_surface_motif_memory();
    let _ = super::l2_surface_foundation_contains("и");
    super::super::l2_surface_decoder::warm_up();
}

pub fn ime_word_candidate_memory_is_warm() -> bool {
    super::super::l2_surface_decoder::is_warm() && SURFACE_MOTIF_MEMORY.get().is_some()
}

pub fn l2_surface_memory_status() -> L2SurfaceMemoryStatus {
    let hot = surface_motif_memory();
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
        broad_source_words: 0,
        broad_prefix_keys: 0,
        broad_word_refs: 0,
        decoder_source_words: decoder.source_words,
        decoder_states: decoder.states,
        decoder_arcs: decoder.arcs,
        decoder_hot_bytes: decoder.hot_bytes,
        foundation_source_limit: L2_FOUNDATION_SOURCE_LIMIT,
        foundation_live_scan_limit: 0,
        generated_forms_loaded,
        generated_forms_words,
    }
}

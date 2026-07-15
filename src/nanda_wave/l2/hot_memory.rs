//! Hot L2 memory lifecycle and status.
//!
//! This module owns warmup and observability for the compact surface/center
//! runtime. Candidate generation stays in the L2 facade; cold corpus material
//! is never consulted here as runtime authority.

use super::{surface_motif_memory, L2_ACTIVE_SOURCE_TARGET};

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize)]
pub struct L2SurfaceMemoryStatus {
    pub active_source_target: usize,
    source_words: usize,
    l1_centers: usize,
    l1_postings: usize,
    l2_word_centers: usize,
    grapheme_nodes: usize,
    grapheme_arcs: usize,
    pub decoder_states: usize,
    pub decoder_arcs: usize,
    training_surfaces: usize,
    artifact_bytes: usize,
    artifact_mmap_backed: bool,
    raw_word_table: bool,
    pub generated_forms_loaded: bool,
    pub generated_forms_words: usize,
}

pub(crate) fn warm_up_surface_motif_memory() {
    let _ = surface_motif_memory().stats();
}

pub(crate) fn warm_up_ime_word_candidate_memory() {
    // Live IME must not pay cold OnceLock construction during the first word.
    warm_up_surface_motif_memory();
    let _ = super::l2_surface_foundation_contains("и");
}

pub fn ime_word_candidate_memory_is_warm() -> bool {
    super::super::lexical_phase::default_memory().is_some()
}

pub fn l2_surface_memory_status() -> L2SurfaceMemoryStatus {
    let hot = surface_motif_memory().stats();
    let generated_forms_loaded =
        crate::russian_lexicon::russian_generated_form_dictionary_is_warm();
    let generated_forms_words = if generated_forms_loaded {
        crate::russian_lexicon::russian_generated_form_dictionary().len()
    } else {
        0
    };
    L2SurfaceMemoryStatus {
        active_source_target: L2_ACTIVE_SOURCE_TARGET,
        source_words: hot.source_words,
        l1_centers: hot.l1_centers,
        l1_postings: hot.l1_postings,
        l2_word_centers: hot.l2_word_centers,
        grapheme_nodes: hot.grapheme_nodes,
        grapheme_arcs: hot.grapheme_arcs,
        decoder_states: hot.decoder_states,
        decoder_arcs: hot.decoder_arcs,
        training_surfaces: hot.training_surfaces,
        artifact_bytes: hot.hot_bytes,
        artifact_mmap_backed: hot.mmap_backed,
        raw_word_table: hot.raw_word_table,
        generated_forms_loaded,
        generated_forms_words,
    }
}

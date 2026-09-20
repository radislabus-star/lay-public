//! Hot L2 memory lifecycle and status.
//!
//! This module owns warmup and observability for the compact surface/center
//! runtime. Candidate generation stays in the L2 facade; cold corpus material
//! is never consulted here as runtime authority.

use super::{surface_motif_memory, L2_ACTIVE_SOURCE_TARGET};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    OnceLock,
};

static IME_WORD_CANDIDATE_MEMORY_READY: AtomicBool = AtomicBool::new(false);
static IME_WORD_CANDIDATE_MEMORY_WARMUP: OnceLock<()> = OnceLock::new();
// English asks for 12 display candidates and Russian asks for a 24-candidate
// donor field. The live gate doubles each request, then ime_readout retains
// four lexical competitors per gate candidate. Warm both exact cache keys.
const IME_EN_HOT_MATERIAL_LIMIT: usize = 96;
const IME_RU_HOT_MATERIAL_LIMIT: usize = 192;
const IME_RU_BOOTSTRAP_PREFIXES: &[&str] = &["пр"];
const IME_EN_BOOTSTRAP_PREFIXES: &[&str] = &["ex"];

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize)]
pub struct L2SurfaceMemoryStatus {
    pub available: bool,
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

pub(crate) fn warm_up_surface_motif_memory() -> bool {
    surface_motif_memory().is_some()
}

pub(crate) fn warm_up_ime_word_candidate_memory() -> bool {
    IME_WORD_CANDIDATE_MEMORY_WARMUP.get_or_init(|| {
        let started = std::time::Instant::now();
        let available = warm_up_prefixes(IME_RU_BOOTSTRAP_PREFIXES, IME_RU_HOT_MATERIAL_LIMIT)
            && warm_up_prefixes(IME_EN_BOOTSTRAP_PREFIXES, IME_EN_HOT_MATERIAL_LIMIT);
        if !available {
            super::super::record_ime_runtime_trace(|| {
                format!(
                    r#"{{"kind":"ibus_l2_ime_warmup","stage":"candidate_memory_unavailable","prefix_warmup_us":{},"candidate_memory_ready":false,"available":false}}"#,
                    started.elapsed().as_micros(),
                )
            });
            return;
        }
        IME_WORD_CANDIDATE_MEMORY_READY.store(true, Ordering::Release);
        super::super::record_ime_runtime_trace(|| {
            format!(
                r#"{{"kind":"ibus_l2_ime_warmup","stage":"candidate_memory_ready_published","prefix_warmup_us":{},"candidate_memory_ready":true}}"#,
                started.elapsed().as_micros(),
            )
        });
    });
    IME_WORD_CANDIDATE_MEMORY_READY.load(Ordering::Acquire)
}

fn warm_up_prefixes(prefixes: &[&str], material_limit: usize) -> bool {
    let prefixes = prefixes
        .iter()
        .map(|prefix| (*prefix).to_string())
        .collect::<Vec<_>>();
    super::ime_readout::warm_up_lexical_readout_cache(&prefixes, material_limit)
}

pub fn ime_word_candidate_memory_is_warm() -> bool {
    IME_WORD_CANDIDATE_MEMORY_READY.load(Ordering::Acquire)
}

pub fn l2_surface_memory_status() -> L2SurfaceMemoryStatus {
    let memory = surface_motif_memory();
    let hot = memory.map(|memory| memory.stats()).unwrap_or_default();
    let generated_forms_loaded =
        crate::russian_lexicon::russian_generated_form_dictionary_is_warm();
    let generated_forms_words = if generated_forms_loaded {
        crate::russian_lexicon::russian_generated_form_dictionary().len()
    } else {
        0
    };
    L2SurfaceMemoryStatus {
        available: memory.is_some(),
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

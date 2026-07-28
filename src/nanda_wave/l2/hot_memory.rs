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
const IME_HOT_MATERIAL_LIMIT: usize = 48;
const IME_BOOTSTRAP_PREFIXES: &[&str] = &[
    "а", "б", "в", "г", "д", "е", "ё", "ж", "з", "и", "й", "к", "л", "м", "н", "о", "п", "р", "с",
    "т", "у", "ф", "х", "ц", "ч", "ш", "щ", "ъ", "ы", "ь", "э", "ю", "я", "a", "b", "c", "d", "e",
    "f", "g", "h", "i", "j", "k", "l", "m", "n", "o", "p", "q", "r", "s", "t", "u", "v", "w", "x",
    "y", "z", "пр", "ст", "по", "на", "за", "вы", "об", "до", "от", "не", "ко", "ра", "re", "co",
    "de", "in", "ex", "un", "pr",
];

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
    IME_WORD_CANDIDATE_MEMORY_WARMUP.get_or_init(|| {
        let prefixes = IME_BOOTSTRAP_PREFIXES
            .iter()
            .map(|prefix| (*prefix).to_string())
            .collect::<Vec<_>>();
        super::ime_readout::warm_up_lexical_readout_cache(&prefixes, IME_HOT_MATERIAL_LIMIT);
        IME_WORD_CANDIDATE_MEMORY_READY.store(true, Ordering::Release);
    });
}

pub fn ime_word_candidate_memory_is_warm() -> bool {
    IME_WORD_CANDIDATE_MEMORY_READY.load(Ordering::Acquire)
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

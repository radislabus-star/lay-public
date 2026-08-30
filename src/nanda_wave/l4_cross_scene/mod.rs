//! Candidate-relative cross-scene L4 memory.
//!
//! The runtime lane is deliberately read-only and advisory. It cannot birth a
//! candidate or grant automatic edit authority; a supported transfer is exposed
//! as `SuggestOnly` until a later promotion gate changes that policy.

#[cfg(any(test, feature = "research-tools"))]
mod compiler;
mod encoder;
mod format;
#[cfg(any(test, feature = "research-tools"))]
mod incremental;
mod merge;
mod model;
#[cfg(any(test, feature = "research-tools"))]
mod proof;
mod runtime;
mod segments;
#[cfg(any(test, feature = "research-tools"))]
mod usage_adapter;

#[cfg(any(test, feature = "research-tools"))]
pub(crate) use compiler::CrossSceneCompileConfig;
pub(crate) use encoder::{
    candidate_relation_id, context_signal_from_text, keep_relation_id, relation_class_from_context,
};
pub(crate) use format::read_package;
#[cfg(any(test, feature = "research-tools"))]
pub(crate) use incremental::update_package_from_inbox;
pub(crate) use model::{
    L4CrossSceneDisposition, L4CrossSceneInput, L4CrossSceneL2Signal, L4CrossSceneProfileKey,
    L4CrossSceneReadout, L4CrossSceneRecommendation,
};
#[cfg(any(test, feature = "research-tools"))]
pub(crate) use proof::prove_cross_scene_word_lists;
pub(crate) use runtime::{reload_shadow_package, shadow_readout};
pub(crate) use segments::{enqueue_episode, status_json as inbox_status_json};
#[cfg(any(test, feature = "research-tools"))]
pub(crate) use usage_adapter::{
    compile_usage_events_path, compile_usage_events_with_corrections_path,
};

pub(crate) const CELLS: usize = 64;
pub(crate) const V1_ENCODER_VERSION: u32 = 1;
pub(crate) const V1_ENCODER_HASH: u64 = 0x4c34_4353_454e_4531;
pub(crate) const ENCODER_VERSION: u32 = 2;
pub(crate) const ENCODER_HASH: u64 = 0x4c34_4d55_4c54_4932;
pub(crate) const MAX_SYMBOLS: usize = 512;
pub(crate) const MAX_PROFILES: usize = 512;
pub(crate) const MAX_PAIR_PROFILES: usize = 4096;
pub(crate) const MAX_CENTERS_PER_BANK: usize = 4;
pub(crate) const MAX_HARD_CENTERS_PER_BANK: usize = 2;
pub(crate) const MAX_AMBIGUITY_CENTERS_PER_BANK: usize = 4;
pub(crate) const SPLIT_COHERENCE: f32 = 0.72;

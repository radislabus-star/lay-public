//! Candidate-relative cross-scene L4 memory.
//!
//! The runtime lane is deliberately read-only and advisory. It cannot birth a
//! candidate or grant automatic edit authority; a supported transfer is exposed
//! as `SuggestOnly` until a later promotion gate changes that policy.

mod compiler;
mod encoder;
mod format;
mod model;
mod proof;
mod runtime;
mod usage_adapter;

pub(crate) use compiler::CrossSceneCompileConfig;
pub(crate) use encoder::{
    candidate_relation_id, context_signal_from_text, keep_relation_id, relation_class_from_context,
};
pub(crate) use format::read_package;
pub(crate) use model::{
    L4CrossSceneDisposition, L4CrossSceneInput, L4CrossSceneL2Signal, L4CrossSceneProfileKey,
    L4CrossSceneReadout, L4CrossSceneRecommendation,
};
pub(crate) use proof::prove_cross_scene_word_lists;
pub(crate) use runtime::{reload_shadow_package, shadow_readout};
pub(crate) use usage_adapter::compile_usage_events_path;

pub(crate) const CELLS: usize = 64;
pub(crate) const ENCODER_VERSION: u32 = 1;
pub(crate) const ENCODER_HASH: u64 = 0x4c34_4353_454e_4531;
pub(crate) const MAX_PROFILES: usize = 128;
pub(crate) const MAX_PAIR_PROFILES: usize = 1024;
pub(crate) const MAX_CENTERS_PER_BANK: usize = 4;
pub(crate) const MAX_HARD_CENTERS_PER_BANK: usize = 2;
pub(crate) const MAX_AMBIGUITY_CENTERS_PER_BANK: usize = 4;
pub(crate) const SPLIT_COHERENCE: f32 = 0.72;

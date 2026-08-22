mod anchor_recovery_package;
mod anchor_recovery_reduce;
mod boundary_birth;
mod calibrate;
mod candidate_state;
mod compiler;
mod composite;
mod conflict_cohort;
mod contour_birth;
mod corpus;
mod delta;
mod events;
mod evidence_reduce;
mod format;
mod format_validation;
mod geometry;
mod induce;
mod live;
mod material_frame;
mod orchestrator;
mod packaged_runtime;
mod phase;
mod proof;
mod records;
mod reduce;
mod runtime;
mod scene;
mod score;
mod semantic_estimator;
mod spool_sort;
mod transition_reduce;
mod trie;
mod types;

pub(in crate::nanda_wave::l2_field) use live::{
    canonical_live_scene_bytes, materialize_live_productive_v1_field,
    prepare_live_productive_v1_field, CanonicalContourRelation, CanonicalContourSeed,
    CanonicalFormGrounding, CanonicalSurfaceGrounding, PreparedCanonicalTokenField,
};
pub(super) use orchestrator::{
    audit_productive_anchor_recovery_v1, compile_productive_paradigm_field_v1,
    reinduce_productive_paradigm_field_v1, resume_productive_paradigm_field_v1,
    ProductiveOrchestratorConfigV1,
};
pub(in crate::nanda_wave::l2_field) use packaged_runtime::PackagedProductiveRuntimeV1;
pub(super) use proof::{
    prove_productive_paradigm_field_v1, prove_productive_paradigm_field_v1_semantic,
};
pub(super) use semantic_estimator::{
    estimate_productive_semantic_transducer_heldout_v1, estimate_productive_semantic_transducer_v1,
};

pub(super) const PRODUCTIVE_V1_SCHEMA_VERSION: u16 = 1;
pub(super) const PRODUCTIVE_V1_INNER_FOLDS: u64 = 5;
pub(super) const PRODUCTIVE_V1_SPLIT_BUCKETS: u64 = 10_000;

pub(super) const L2_SCENE_PHASE_CELLS: usize = 60;
pub(super) const MAX_POSITIVE_SUBCENTERS: usize = 4;
pub(super) const MAX_ANTI_SUBCENTERS: usize = 4;
pub(super) const MAX_HARD_NEGATIVE_SUBCENTERS: usize = 2;
pub(super) const MAX_AMBIGUITY_SUBCENTERS: usize = 8;

const _: () = {
    assert!(super::CANONICAL_L2_LEMMA_FRONTIER == 256);
    assert!(super::CANONICAL_L2_ACTIVE_LEMMA_LIMIT == 256);
    assert!(super::CANONICAL_L2_FEATURE_LIMIT == 16);
    assert!(super::CANONICAL_L2_FORM_LIMIT == 32);
    assert!(super::CANONICAL_L2_ATOM_RELATION_LIMIT == 196_608);
};

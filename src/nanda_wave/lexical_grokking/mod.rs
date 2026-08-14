//! Shadow-only recursive L1 lexical grokking proof.
//!
//! The package stores typed atom postings and compact phase centers. It refers
//! to decoder terminal IDs and never stores source or damaged strings.

mod anti_postings;
mod atoms;
mod behavior_fingerprint;
mod compaction;
mod compiler;
mod composite;
mod corruption;
mod crystal;
mod format;
mod model;
mod ngram_graph;
mod pairwise;
mod peak_search;
mod posting_codec;
mod posting_spool;
mod proof;
pub(super) mod restoration;
mod runtime;
mod service;
mod training_budget;
mod training_corpus;
mod v8;
mod wave_basis;

pub use behavior_fingerprint::fingerprint_l1_behavior;
pub use compaction::compact_depth0_package;
pub use composite::initialize_manifest as initialize_l11_composite_manifest;
pub use composite::{admit_delta as admit_l11_delta, admit_tombstone as admit_l11_tombstone};
pub use corruption::ScaleTrainingSurfacePolicy;
pub(crate) use corruption::{split_damages, DamageExample};
pub use posting_codec::analyze_package as analyze_l1_forward_compression;
pub use proof::{
    crystallize_l1_lexical_grokking, crystallize_l1_lexical_grokking_with_rss_budget,
    crystallize_l1_lexical_grokking_with_surface_policy, export_l1_fixed_latency_surfaces,
    prove_l1_lexical_grokking, prove_l1_lexical_grokking_complete_postings,
    prove_l1_lexical_grokking_composite, prove_l1_lexical_grokking_package,
    prove_l1_lexical_grokking_scale_package, prove_l1_lexical_grokking_scale_package_range,
};
pub use runtime::{
    benchmark_diverse_restoration as benchmark_l1_diverse_restoration,
    benchmark_package as benchmark_l1_lexical_grokking,
    inspect_package_header as inspect_l1_package_header,
    query_package as query_l1_lexical_grokking, restore_surface as restore_l1_surface,
    L1RestorationHost, L1RestorationHostStats,
};
pub use service::{
    authoritative_restore_surface, default_l11_model_dir, default_l11_socket_path,
    discover_installed_l11_package, ensure_l11_service_started, l11_seed_surfaces,
    request_l11_authoritative_surface, request_l11_decoded_surfaces, request_l11_seed_surfaces,
    send_l11_service_request, send_l11_service_request_with_timeout, InstalledL11Package,
    L11SeedSurface, L11ServiceEnsureReport, L1ServiceHealth, L1ServiceRequest, L1ServiceResponse,
    L1ServiceStats,
};
pub use v8::{build_lazy_v8_package, build_lazy_v8_package_with_shard_size};

#[cfg(test)]
mod tests;

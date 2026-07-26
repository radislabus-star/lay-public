//! Shadow-only recursive L1 lexical grokking proof.
//!
//! The package stores typed atom postings and compact phase centers. It refers
//! to decoder terminal IDs and never stores source or damaged strings.

mod anti_postings;
mod atoms;
mod compaction;
mod compiler;
mod corruption;
mod crystal;
mod format;
mod model;
mod ngram_graph;
mod pairwise;
mod posting_codec;
mod posting_spool;
mod proof;
mod restoration;
mod runtime;
mod training_budget;
mod training_corpus;
mod wave_basis;

pub use compaction::compact_depth0_package;
pub use corruption::ScaleTrainingSurfacePolicy;
pub use posting_codec::analyze_package as analyze_l1_forward_compression;
pub use proof::{
    crystallize_l1_lexical_grokking, crystallize_l1_lexical_grokking_with_rss_budget,
    crystallize_l1_lexical_grokking_with_surface_policy, prove_l1_lexical_grokking,
    prove_l1_lexical_grokking_complete_postings, prove_l1_lexical_grokking_package,
    prove_l1_lexical_grokking_scale_package,
};
pub use runtime::{
    benchmark_package as benchmark_l1_lexical_grokking, query_package as query_l1_lexical_grokking,
    restore_surface as restore_l1_surface,
};

#[cfg(test)]
mod tests;

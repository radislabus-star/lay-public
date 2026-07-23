//! Shadow-only recursive L1 lexical grokking proof.
//!
//! The package stores typed atom postings and compact phase centers. It refers
//! to decoder terminal IDs and never stores source or damaged strings.

mod atoms;
mod compiler;
mod corruption;
mod crystal;
mod format;
mod model;
mod ngram_graph;
mod pairwise;
mod posting_codec;
mod proof;
mod restoration;
mod runtime;
mod wave_basis;

pub use posting_codec::analyze_package as analyze_l1_forward_compression;
pub use proof::{
    prove_l1_lexical_grokking, prove_l1_lexical_grokking_complete_postings,
    prove_l1_lexical_grokking_package,
};
pub use runtime::{
    benchmark_package as benchmark_l1_lexical_grokking, query_package as query_l1_lexical_grokking,
    restore_surface as restore_l1_surface,
};

#[cfg(test)]
mod tests;

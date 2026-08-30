//! Compact lexical phase memory.
//!
//! Corpora are cold compiler inputs. Runtime maps a shared binary artifact
//! containing L1 atom postings, L2 phase centers, and a reversible grapheme
//! graph. It never expands the corpus into a `Vec<String>`.

mod format;
mod runtime;

#[cfg(any(test, feature = "research-tools"))]
mod compiler;

#[cfg(any(test, feature = "research-tools"))]
pub(crate) use compiler::compile_words;
pub(crate) use runtime::{
    default_memory, default_memory_if_warm, LexicalPhaseCandidate, LexicalPhaseMemory,
};

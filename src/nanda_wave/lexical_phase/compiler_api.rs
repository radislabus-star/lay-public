//! The existing cold compiler surface shared with the training executable.
//! Runtime lookup and package format implementations stay private.
pub use super::compiler::compile_words_with_training;
pub use super::format::{checksum, read_header, ArtifactHeader, MAX_WORD_CHARS, VERSION};

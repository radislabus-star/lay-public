//! Lightweight char n-gram scorer for short local decisions.
//!
//! This is not a generator. It only compares ready candidates and answers:
//! which text looks more natural for the model trained from local dictionaries
//! and optional local user word lists.

mod cache;
mod model;
mod sources;
mod static_models;
mod tokenize;

pub use cache::{default_ru_cache_path, load_ru_cache, save_ru_cache};
pub use model::{CharNgramModel, Lang};
pub use sources::build_ru_model_from_sources;
pub use static_models::{en_score, ru_candidate_is_better, ru_candidate_margin, ru_score, warm_up};
pub use tokenize::tokenize_text;

#[cfg(test)]
#[path = "ngram_tests.rs"]
mod tests;

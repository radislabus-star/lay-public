# lay lexicon data

This directory contains lexical data for the deterministic correction engine.
The Rust core should load these lists through `src/lexicon.rs` instead of
embedding word lists directly in production logic.

Regression examples from user logs belong in `tests/`, not here, unless they
are intentionally promoted into a user-visible dictionary or configuration file.

# lay lexicon data

This directory contains lexical data for the deterministic correction engine.
The Rust core should load these lists through `src/lexicon.rs` instead of
embedding word lists directly in production logic.

Regression examples from user logs belong in `tests/`, not here, unless they
are intentionally promoted into a user-visible dictionary or configuration file.

`ru_technical_loanwords*.txt` is for Russian-written forms of technical English
loanwords that should be treated as valid words. They protect normal mixed tech
speech from typo rules such as adjacent transposition. The stem/suffix pair is
for small regular paradigms; do not use it as a broad chat-log dictionary.

`l2_surface_hot_ru.txt` is the hot Russian surface bank for NANDA L2 center
memory. It is generated from a broad local corpus by balanced first-letter
sampling, then mixed with curated positive cases at build time. Keep it as
lexical substrate, not as a place for one-off autocorrect patches.

# Text Correction Gate Architecture

This note is the working architecture contract for text mutation paths.

## Pipeline Tree

```text
input stream
|
+-- L1 surface sensors
|   |
|   +-- character shape
|   +-- layout shape
|   +-- token boundary
|   +-- local n-gram evidence
|
+-- L2 candidate field
|   |
|   +-- deterministic layout/typo candidates
|   +-- learned surface candidates
|   +-- boundary candidates
|   +-- completion candidates
|
+-- L3 phrase/context gate
|   |
|   +-- boosts or suppresses L2 candidates
|   +-- may forecast phrase-local continuations
|   +-- must not directly own destructive text edits
|
+-- correction core
|   |
|   +-- builds the candidate lattice
|   +-- assigns candidate roles through correction_source_contract
|   +-- records status-only quality/latency counters
|
+-- text edit gate
|   |
|   +-- the only public owner for planned destructive replacement actions
|   +-- authorizes edit plans through text_edit safety
|
+-- output backend
    |
    +-- daemon replay
    +-- native text replace
    +-- IME backend display/commit
```

IME is a display and commit backend. It is not a second correction brain.

## Scoreboard

```text
correction_gate
|
+-- requests
+-- total_candidates
+-- apply_candidates
+-- suggest_only_candidates
+-- keep_original_candidates
+-- veto_candidates
+-- deterministic_candidates
+-- nanda_candidates
+-- selected_apply
+-- avg_us
+-- max_us

input_gate / recent_actions
|
+-- total_candidates
+-- apply/suggest/keep/veto split
+-- deterministic vs NANDA split
+-- selected source/error class
```

These metrics are status-only. They must not log raw user text.

## Debt Queue

```text
P0: keep all destructive text mutation behind text_edit::authorize_replacement
P1: keep source role decisions behind correction_source_contract
P2: split correction_core only by route, not by file size
P3: keep IME display isolated from correction ownership
P4: make candidate quality/latency regressions visible before release
```

`src/keyboard/event_words/decision.rs` is route-critical because it decides
manual replay layout, but it must remain outside candidate generation and text
replacement ownership.

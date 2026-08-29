# TD-005: Make The Lint Gate Truthful

Status: `DONE`
Priority: `P0`
Class: CI and maintainability
Size: `M`
Depends on: TD-004

## Why Now

CI declares `cargo clippy --all-targets -- -D warnings`, but that exact command
fails on the current public baseline. A permanently red gate protects nothing.

## Evidence

- `cargo check --all-targets`: 385 warnings and zero errors.
- Warning classes: 377 `dead_code`, 6 `unused_imports`, 2
  `dropping_copy_types`; 348 diagnostics are unique by code/file/message.
- `cargo clippy --all-targets -- -D warnings`: exit 101 with 5,097 stderr lines.
- Dead code is concentrated in proof/research paths, but live files also contain
  unused imports and methods.

## Target State

One local script and CI execute the same lint contract on the exact TD-004 lint
toolchain. Ordinary warnings are hard errors. Historical dead-code debt has an
explicit machine-checked baseline that can only decrease until TD-008 removes
proven residue; new dead code fails immediately.

## Scope

- Fix all non-`dead_code` warnings rather than suppressing them globally.
- Generate a deterministic dead-code inventory keyed by diagnostic code, file,
  item/message, and target context.
- Add a checked baseline count/list and reject additions or path drift.
- Make CI call the same repository script used locally.
- Keep `-D warnings` for every warning class outside the temporary dead-code
  budget.
- Document how TD-008 reduces the baseline and transfers retained proof/compiler
  rows to the explicit TD-104 decision.

## Non-Goals

- Do not add crate-wide `#![allow(dead_code)]`.
- Do not delete proof APIs without identifying their producer/consumer route.
- Do not treat every test-only helper as production dead code.
- Do not combine warning cleanup with algorithm changes.

## TDD Plan

1. Freeze the unique current dead-code inventory.
2. Add a fixture or script self-test proving one new dead item fails.
3. Add a fixture proving a removed item lowers the accepted baseline.
4. Repair non-dead diagnostics.
5. Switch local and CI entrypoints together.

## Acceptance Gates

- `scripts/check-lay-lints.sh` passes on baseline.
- Adding a temporary unused function causes the script to fail.
- Removing a baseline item requires lowering the baseline; stale baselines fail.
- CI contains no separate divergent clippy command.
- `cargo check --all-targets` has zero non-dead-code warnings.
- `git diff --check` passes.

## Risks And Guardrails

- Diagnostic text can change across Rust versions. Bind the inventory to the
  exact lint toolchain from TD-004 and use a stable normalized key.
- A count-only budget can hide churn. Require item identity as well as count.
- Do not normalize away file ownership.

## Independent Review Brief

Try to add a new dead function, rename an old one, and introduce an unrelated
warning. All three cases must produce the intended result. Score 1-10.

## Completion Record

- Base commit: `cf03e3d0`
- Separate stale TD-009 owner-contract correction:
  `67797979043f0312ba6564ff040902e27203875b`; test-only, no runtime change.
- Implementation commit: `9445102ed2e240ad9ec3f23cf497ae2cc6695d68`.
- Exact lint baseline: schema `lay.dead-code-baseline.v3`, 368 entries,
  SHA-256 `5452d74d5c58e1471be3a3ff95f78082d2a72b95f47a11f8335de16912807056`.
- Review pass 1: `3/10`. Findings: red baseline after removing broad
  allowances, generic diagnostic collapse, acceptance of incomplete Cargo
  streams, a divergent full-check Clippy route, and a mixed structural-test
  correction.
- Review pass 2: `6/10`. Findings: the writer could ratchet debt upward,
  same-shaped source items could collapse, and `build-finished` was not
  required to be the terminal record.
- Review limit: two passes. The second-pass findings were corrected and proven
  by objective adversarial gates; no third score was invented.
- Final contract:
  - CI, changed, full, and documented routes use
    `scripts/check-lay-lints.sh`;
  - every non-`dead_code` rustc/Clippy diagnostic is fatal;
  - same-shaped dead items retain exact multiplicity through occurrence rows;
  - empty, malformed, failed, truncated, and post-`build-finished` streams fail;
  - `--write-baseline` accepts only an equal or strictly reduced inventory;
  - item-local `#[expect(..., reason = "...")]` replaces broad suppression.
- Verification:
  - exact Rust 1.97.1 lint contract: PASS, 368/368, zero non-dead diagnostics;
  - parser/self-test including same-shape multiplicity, stream finality, and
    monotonic writer: PASS;
  - injected dead row through the supported writer: rejected and no candidate
    baseline published;
  - exact Rust 1.88.0 default and `lexical-compiler` all-target checks: PASS;
  - architecture graph and source binding refreshed; architecture gate: PASS;
  - focused correction, IME, daemon, context-phase, L3/L4, and mutation-owner
    suites run during implementation: PASS;
  - `cargo fmt --all --check`, Python/shell syntax, and `git diff --check`: PASS.
- Existing semantic residuals were reproduced on the clean base and remain
  owned by TD-007: one phrase-reader expectation and three Nanda L3
  expectations. They were not rewritten or waived in this task.
- Untested: live desktop behavior and installation; this task changed no
  runtime authority.

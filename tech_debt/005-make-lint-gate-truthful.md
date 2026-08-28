# TD-005: Make The Lint Gate Truthful

Status: `READY`
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

- Commit: pending
- Review score: pending
- Verification: pending

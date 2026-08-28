# TD-006: Build Hermetic Test Lanes

Status: `READY`
Priority: `P0`
Class: test architecture
Size: `L`
Depends on: TD-001, TD-005

## Why Now

The test suite mixes semantic correctness, live installed packages, global
environment mutation, singleton caches, and wall-clock latency in one process.
Failure counts therefore do not identify the failing contract.

## Evidence

- Full run: 1,539 pass, 88 fail, 11 ignored in 109.26 seconds.
- A clean temporary HOME still leaves 33 failures in `correction_core`, proving
  that environment leakage is not the only cause.
- `candidate_gate` latency fails in isolation with a 514,743 us outlier and a
  multi-second cold stage.
- `context_phase` latency fails under the full suite but passes alone at p99
  1,266 us, proving contention sensitivity.
- Runtime modules use HOME/XDG/package environment variables and many
  `OnceLock`/global caches.

## Target State

Tests are split into explicit lanes with different authority:

1. hermetic correctness: no installed user packages, network, daemon, or wall
   clock budgets;
2. package integration: exact pinned fixture packages and isolated directories;
3. performance: serialized, warmed, explicit budget enforcement;
4. live desktop smoke: opt-in and process-isolated.

## Scope

- Add a single test runner that creates isolated HOME, XDG paths, sockets, and
  package roots while preserving Cargo/Rust toolchain locations.
- Centralize process-global environment guards and serialize tests that mutate
  environment or singleton package state.
- Move timing assertions out of ordinary parallel unit-test authority. Keep
  functional assertions in unit tests and run timing tests in an explicit lane.
- Emit per-lane counts and elapsed time.
- Make `check-lay-changed.sh`, full checks, and CI consume the same lanes.
- Generate a complete test manifest keyed by target and full test name. Lane
  membership must be disjoint and its union must equal Cargo's discovered set,
  apart from explicitly documented ignored tests.
- Add a temporary known-failure manifest for the exact TD-007 semantic
  denominator. Each row records test identity, cluster, failure signature, and
  owner; a new, renamed, or disappeared failure is an error until disposition.

## Non-Goals

- Do not mark current semantic failures ignored here; TD-007 owns them.
- Do not relax latency budgets.
- Do not mock away package identity checks.
- Do not run live desktop mutation in CI.

## TDD Plan

1. Prove a seeded file in real HOME cannot affect the hermetic lane.
2. Prove two environment-mutating tests cannot overlap.
3. Prove timing tests are absent from parallel correctness execution.
4. Prove the serialized timing lane enforces the existing budget.
5. Prove a seeded new failure, missing test, renamed test, and unexpectedly fixed
   failure all make manifest validation fail.
6. Record the residual semantic failure list for TD-007.

## Acceptance Gates

- Repeated hermetic correctness runs have identical test counts and failure
  names.
- No test reads current `~/.local/share/lay` or writes current user config.
- Timing lane runs with one test thread and explicit warm/cold semantics.
- Full runner reports semantic failures separately from infrastructure failures.
- No existing test is silently dropped from all lanes.
- CI exits success only when the observed known-failure set is exact and every
  other test passes; a changed failure set is never silently green.

## Risks And Guardrails

- Changing HOME can break rustup/cargo discovery; preserve `CARGO_HOME` and
  `RUSTUP_HOME` explicitly.
- Singleton caches may need process isolation, not only a mutex.
- Keep exact package fixtures byte-pinned.

## Independent Review Brief

Audit test enumeration for omissions and try contaminating real HOME, XDG, and
package paths. Verify timing tests cannot run in parallel. Score 1-10.

## Completion Record

- Commit: pending
- Review score: pending
- Verification: pending

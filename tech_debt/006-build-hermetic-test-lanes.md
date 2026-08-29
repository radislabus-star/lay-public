# TD-006: Build Hermetic Test Lanes

Status: `DONE`
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

- Base commit: `5aba0e9afb45171517f0aa313cc2a5b5f45288bc`.
- Implementation commit: `9a15193a60d40d5840e39f6ca280f034a34b736d`.
- Manifest: 2,372 exact rows; 2,315 correctness, 35 package, 11
  performance, and 11 ignored. Isolation is 27 process / 2,345 target.
- Hermetic boundary: network, IPC, PID, HOME/XDG, and `/run` are isolated;
  repository/host bytes are read-only; live D-Bus/Wayland sockets are absent;
  external Cargo configuration is rejected before compilation.
- Sealed semantic denominator: 116 exact failures, split 96 correctness / 20
  package and owned only by TD-007. Observation SHA-256:
  `2c03eafe5a3bd7b71b2cc67c6cc8d774f148ef20b92684d1efeb354712d2d2b0`.
  Independent repeat SHA-256:
  `d054266e4b0708d622e81db1b27547d87207a9140c6e3b5733e8b1f1ade2673b`.
  Both runs have identical target/test identities, normalized signatures, and
  source closure `021466aced1f5dee84465531ed42ddc31a0b5b88ef8ddf5671f09146273f67b2`.
- Run summaries: `b85456d65d35bed8e708d4b0c005beebbb9684bf67e8fe8a9757cf6eebf99c47`
  and `eac01f2022400f460e167c49bbdf3488917e04aff21ba25cd5842da1ab3332bd`;
  both verdicts are `PASS_WITH_EXACT_KNOWN_FAILURES` with zero infrastructure
  failures.
- Performance is explicit and truthfully red: 8/11 PASS, 3/11
  `BLOCKED_PERFORMANCE`; receipt SHA-256:
  `4269ff79210076ab0ba8bb4033ac38f67198a68733f9dbcdf3ff918c853cdaef`.
  No latency or RSS budget was relaxed.
- Review pass 1: `4/10`, agent
  `01a04e79-df40-77b0-bda1-327b4eccbcce`. Review pass 2: `6/10`, agent
  `01a04e97-8f9e-76f0-9135-1d6a0969bbf2`. The second pass found live `/run`
  socket exposure, external Cargo config authority, three omitted env
  mutators, incomplete evidence provenance, and missing per-lane totals. All
  findings were corrected and covered by objective gates; per the two-pass
  limit, no third score was invented.
- Verification: 16/16 runner self-tests PASS; manifest drift check PASS;
  package lane exact PASS; two full hermetic runs exact PASS; Python/shell/JSON
  syntax, `cargo fmt --check`, `git diff --check`, and graphify update PASS.
- Untested: managed live desktop mutation and installation. Runtime authority
  changed: no.

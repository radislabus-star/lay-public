# Lay Tech-Debt Queue

Audit baseline: 2026-08-29, commit
`a2ace675230a92c902a13c2f254c6a8d1c8c81c1`. Reproduction commands,
toolchain identities, output hashes, and limits are recorded in
[`BASELINE_2026-08-29.md`](BASELINE_2026-08-29.md).

This directory is the executable debt queue. A task is not complete when code
merely compiles: its acceptance gates, independent review, completion record,
commit, and push must all be present.

## Operating Rules

1. Execute the table order below unless a task discovers a higher-priority
   regression. `TD-009` was inserted after `TD-002` because the isolated live
   proof exposed a user-visible Double Shift race that blocks later IME gates.
2. Start with a failing test or a frozen baseline. Do not mix behavior changes
   with move-only refactors.
3. Before edits, record the base commit, exact command, environment/toolchain,
   feature set, output or receipt hash, untested scope, and revert boundary.
4. After implementation, run a fresh-context independent code review. The
   reviewer reports findings first and a score from 1 to 10.
5. Allow at most two correction passes. An unresolved correctness finding keeps
   the task open. A score below 8/10 triggers correction, but the numeric score
   does not override verified fixes after the two-pass limit; record the actual
   pre-correction scores and objective final gates without inventing a third
   review. Two passes that leave findings unresolved move the task to
   `REPLAN_REQUIRED`; they do not authorize a weakened acceptance gate.
6. Mark the task `DONE`, record tests and review evidence, then commit and push
   before starting the next task.
7. Preserve the Lay 1.0.54 Double Shift ownership contract. IME work must also
   recheck candidate visibility, layout synchronization, and terminal
   passthrough.
8. Tasks `101` through `104` are decision proposals, not admitted edits. They
   require a separate cost/risk decision after the near-term queue is complete.

## Scoreboard

| Signal | Measured baseline | Meaning |
|---|---:|---|
| Rust source | about 245k lines | Large single-package build surface |
| `src/nanda_wave` | 163,770 lines / 193 files | Runtime, compiler, proof, and research code share one crate |
| `src/bin` | 50,562 lines / 216 files | Many binaries plus substantial adapter state |
| Static Rust tests | 2,336 `#[test]` declarations | High raw count, but authority and determinism are uneven |
| Full test run | 1,539 pass / 88 fail / 11 ignored | `cargo test --all-targets` is not a usable release signal |
| `cargo check --all-targets` | 385 warnings / 0 errors | 348 unique diagnostics; warning ownership is not controlled |
| Warning classes | 377 dead code / 6 unused imports / 2 dropping-copy | Research/proof surface dominates, with some live-route residue |
| CI clippy | exit 101, 5,097 stderr lines | Declared `-D warnings` gate is currently red |
| Architecture receipt | stale and fresh verdict `WATCH` | Six violations require explicit disposition before resealing |
| 50-pass audit | 49 checks executed, several false-scope failures | Name and expected denominator are incorrect |
| Claimed MSRV | Rust 1.75 | Source uses APIs newer than 1.75; CI tests only floating stable |
| One-shot experiment scripts | 112 files / 99,664 lines | Reproducibility code dominates active `scripts/` navigation |
| Tracked receipt files | 3,072 / about 89 MB | Compact evidence is substantial but manageable |
| Local ignored receipt payloads | about 3.6 GB; 16 files over 10 MB | Checkout doubles as an artifact store |
| Cargo target | about 2.2 GB of 12 GB budget at audit start | Within budget |

## Failure Clusters

The 88 full-suite failures must not become 88 example-specific patches:

| Cluster | Failures | First shared issue |
|---|---:|---|
| `correction_core::tests` | 31 | Superseded semantic expectations |
| `ime_correction` | 27 | Superseded semantic/authority expectations |
| Nanda L2/L3/bridge/candidate routes | 17 | Mixed current and historical contracts |
| Text edit / typing / phrase routes | 9 | Old reason or selection assertions |
| Architecture contract | 2 | Stale receipt and owner text |
| Timing assertions | 2 | Performance checks run inside contended unit-test execution |

## Execution Queue

| Order | Task | Priority | Size | Status | Product value |
|---:|---|---|---|---|---|
| 001 | [Repair architecture and audit gates](001-repair-architecture-and-audit-gates.md) | P0 | M | DONE | Makes structural checks truthful again |
| 002 | [Isolate live runtime smoke cases](002-isolate-live-runtime-smoke.md) | P0 | M | DONE | Makes user-visible proof safe and case-independent |
| 003 | [Converge the manual-toggle visible postcondition](009-fix-manual-toggle-visible-postcondition-race.md) | P0 | M | DONE | Removes the isolated Double Shift commit race without timing sleeps |
| 004 | [Fix pending preedit refresh convergence](003-fix-preedit-refresh-convergence.md) | P0 | S | DONE | Removes visible stale/duplicated IME suffix |
| 005 | [Enforce the real MSRV and pinned lint toolchain](004-enforce-real-msrv.md) | P0 | S | DONE | Replaces false and floating compiler contracts |
| 006 | [Make the lint gate truthful](005-make-lint-gate-truthful.md) | P0 | M | READY | Restores an enforceable green CI contract |
| 007 | [Build hermetic test lanes](006-build-hermetic-test-lanes.md) | P0 | L | READY | Separates correctness, environment, and timing failures |
| 008 | [Reconcile superseded semantic tests](007-reconcile-semantic-contract-tests.md) | P0 | XL | READY | Converts the full suite into current authority evidence |
| 009 | [Classify and remove obvious dead code](008-reduce-dead-code-and-proof-surface.md) | P1 | L | READY | Removes proven residue without a workspace rewrite |

## Decision Queue

These are intentionally not part of automatic Stage 2 execution:

| Task | Priority | Status | Decision needed |
|---|---|---|---|
| [Decompose the IBus engine state owner](101-decompose-ibus-engine-state-owner.md) | P2 | DISCUSSION_REQUIRED | Is state isolation worth the regression risk now? |
| [Separate Nanda runtime from research tooling](102-separate-nanda-runtime-and-research.md) | P2 | DISCUSSION_REQUIRED | Workspace split versus current single-crate convenience |
| [Externalize research payload lifecycle](103-externalize-research-payloads.md) | P2 | DISCUSSION_REQUIRED | Storage policy without weakening immutable evidence |
| [Isolate proof/compiler build surfaces](104-isolate-proof-compiler-build-surface.md) | P2 | DISCUSSION_REQUIRED | Is feature gating worth its measured build-surface benefit? |

## Backlog Review

- Independent reviewer: fresh-context agent `01a04a99-29db-73f3-8133-c5e65940b10a`
- Initial score: `6/10`
- Initial findings: eight; four high and four medium
- Corrective passes: `1/2`
- Corrections: graph `WATCH` disposition and deletion freshness; route order;
  exact baseline provenance; smoke ownership; pinned toolchains; exhaustive lane
  and failure manifests; semantic milestone ledger; bounded dead-code scope.
- Final reviewer: fresh-context agent `01a04aab-a529-7112-8276-b6b1eb403437`
- Final score: `9/10`; no high or medium findings
- Stage 1 verdict: `TECH_DEBT_BACKLOG_REVIEWED_READY`

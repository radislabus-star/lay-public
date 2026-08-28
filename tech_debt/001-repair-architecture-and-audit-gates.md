# TD-001: Repair Architecture And Audit Gates

Status: `READY`
Priority: `P0`
Class: proof and release infrastructure
Size: `M`
Depends on: none

## Why Now

The repository advertises structural gates that currently fail for mixed
reasons: one stale receipt, six unresolved graph violations, false-positive
scope checks, and a wrong check denominator. Every later cleanup would otherwise
choose between ignoring a red gate and editing against an unclassified signal.

## Evidence

- `scripts/check-architecture.sh` first fails because the generated architecture
  receipt fingerprint is stale, but a fresh read-only calculation still returns
  `WATCH`; receipt regeneration alone cannot make the gate pass.
- The fresh `WATCH` contains six violations: two forbidden `text_edit::cursor`
  imports and one foreign `TransitionDecisionCore` owner; two visible-tail
  receipt constructor sites; one `Vec<String>` hot full-word authority; and one
  duplicate `normalize_surface()` symbol class.
- `scripts/check-lay-audit-50.sh` executes 49 checks while requiring 50.
- The profanity check scans immutable receipts even though its intent is to
  protect live runtime source.
- The sleep check reports proof helpers and training binaries as runtime hot-path
  violations.
- Centralized file/permission checks flag explicitly separate package and socket
  durability owners without expressing the allowed ownership model.
- Several advisory line budgets have drifted, but a size overrun alone is not a
  structural failure and must remain advisory.

## Current Graph Disposition Worklist

| Graph check | Current violation | Disposition before implementation |
|---|---|---|
| `decision-authority` | `context_phase/compiler.rs` imports `text_edit::cursor` | pending contract trace |
| `decision-authority` | `context_phase/proof.rs` imports `text_edit::cursor` | pending contract trace |
| `decision-authority` | `TransitionDecisionCore` appears in `decision/live_field.rs` | pending owner trace |
| `typed-transition-capability` | two visible-tail receipt constructor sites | pending constructor-authority trace |
| `hot-field-memory` | `LexicalPhaseMemory` owns hot `Vec<String>` | pending representation contract trace |
| `fast-verifiable` | three `normalize_surface()` symbols | pending semantic-owner trace |

`pending` is not an allow. TD-001 must replace every row with one evidence-backed
disposition before writing a new receipt.

## Target State

`check-architecture.sh` and `check-lay-audit-50.sh` are deterministic, scoped to
their stated authority, and green on the corrected baseline. The generated
receipt binds exactly the current graph inputs, including file additions,
deletions, and renames. Every graph violation has a documented disposition and
every audit label matches its actual check count and scope.

## Scope

- Refresh the architecture graph receipt through its canonical producer.
- Add a disposition table for every current graph violation with exactly one of
  `FIX_RUNTIME_ARCHITECTURE`, `FIX_GATE_MODEL`, or `BLOCKED_CONTRACT_DECISION`.
  Each row cites the active contract and a validating test.
- Repair graph freshness so extra manifest/graph entries for deleted or renamed
  Rust files are rejected, not silently ignored.
- Correct the 50-pass denominator by adding the missing meaningful check or by
  renaming/versioning the audit if 49 is the intentional contract.
- Restrict runtime-source scans to active runtime source and exclude sealed
  receipts, fixtures, proof-only modules, and training controllers where the
  invariant does not apply.
- Replace blanket file-operation ownership checks with explicit, narrow owners:
  private user files, immutable model/package writes, and Unix socket mode
  changes are different routes.
- Keep line budgets advisory and update stale references only when the current
  boundary is accepted.
- Add regression tests for the gate scripts where a future scope drift would be
  easy to reintroduce.

## Non-Goals

- No runtime behavior change.
- No deletion of immutable receipts.
- No broad module split based only on line count.
- No weakening of text-mutation, candidate-authority, or Double Shift owner
  checks.
- No hand-authored allowlist for the six current graph violations.

## TDD Plan

1. Freeze the fresh graph payload and classify all six violations before source
   or gate edits.
2. Add graph freshness tests for unchanged, modified, added, deleted, and renamed
   source files; prove delete/rename fail against a stale graph.
3. Add or extend fixtures for one active-source violation and one receipt/proof
   false positive.
4. Prove the active-source violation fails and sealed evidence stays outside the
   runtime denominator.
5. Repair each graph violation according to its disposition, then repair audit
   scopes and the exact check count.
6. Regenerate and validate the architecture receipt only after the computed
   verdict is `PASS`.

## Acceptance Gates

- `scripts/check-architecture.sh` passes.
- `scripts/check-lay-audit-50.sh` passes and prints exactly the declared number
  of `OK` rows.
- `python3 scripts/architecture_graph_gate.py --check-receipt --format text`
  passes.
- `python3 scripts/architecture_graph_gate.py --format json` reports `PASS`, not
  merely a fresh `WATCH`.
- Added, modified, deleted, and renamed Rust sources all exercise the intended
  graph-staleness result in focused tests.
- A seeded active runtime violation still makes the relevant gate fail.
- A seeded matching string under immutable receipts does not affect the runtime
  gate.
- `git diff --check` passes.

## Risks And Guardrails

- Exclusions can accidentally hide live code. Use path-specific ownership
  classes, not a broad `docs|proof|tests` regular expression.
- Receipt regeneration must be deterministic; do not hand-edit hashes.
- Do not convert hard owner checks into advisory warnings.
- A real unresolved conflict stops as `BLOCKED_CONTRACT_DECISION`; a green
  receipt is not more important than the architecture contract.

## Independent Review Brief

Check for false negatives introduced by exclusions, denominator honesty, and
whether the generated receipt was produced rather than edited. Score 1-10.

## Completion Record

- Commit: pending
- Review score: pending
- Verification: pending

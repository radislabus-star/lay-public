# TD-008: Classify And Remove Obvious Dead Code

Status: `DONE`
Priority: `P1`
Class: compile surface and ownership
Size: `L`
Depends on: TD-005, TD-006, TD-007

## Why Now

The library compiles runtime, compilers, proofs, abandoned experiments, and
test-only implementations together. This creates hundreds of dead-code
diagnostics and makes every normal change traverse a much larger surface.

## Evidence

- The canonical TD-005 baseline contains 368 exact `dead_code` rows under Rust
  1.97.1; occurrence identity is retained rather than collapsed by message.
- Concentrations include `text_edit/output_transaction.rs`, V13 experiment
  kernels, lexical typed-basin code, productive delta/phase code, and target
  evidence helpers.
- `src/nanda_wave` is 164,071 Rust lines in the main library crate.
- Some live files contain genuinely unused methods; other items are proof-only
  and should not be judged by production reachability.

## Target State

Every TD-005 baseline item has an owner classification. Proven live-unused and
obsolete items are removed in owner-scoped batches with no behavior change.
Proof/compiler feature isolation is measured here but remains a separate TD-104
decision. This task does not force the baseline to zero by hiding legitimate
proof code.

## Scope

- Classify every baseline item as live-unused, proof-only, compiler-only,
  compatibility API, or obsolete, with producer/consumer evidence.
- Remove live-unused and obsolete items in small owner-scoped commits.
- Keep runtime contracts and serialized formats unchanged.
- Lower the TD-005 baseline after every batch.
- Measure clean/default check time, rebuilt crate count, warning count, and
  relevant artifact sizes before and after. Use those facts to decide TD-104.

## Non-Goals

- No crate-wide allow.
- No mass deletion based only on compiler reachability.
- No behavior/scoring changes during removal.
- No rewrite of immutable experiment controllers.
- No new feature combination or package boundary; TD-104 owns that decision.

## TDD Plan

For each owner batch:

1. Record consumers and feature/target route.
2. Freeze runtime and proof tests.
3. Delete only proven residue without algorithm changes.
4. Run default runtime and all-target checks.
5. Lower warning baseline and commit the batch.

## Acceptance Gates

- Every TD-005 dead-code row has one reviewed ownership classification.
- All live-unused and obsolete rows are removed and the baseline is reduced by
  their exact count; retained proof/compiler/API rows remain explicit debt.
- Runtime binary behavior and package/format parity tests pass.
- No new feature combination or package boundary is introduced.

## Risks And Guardrails

- Rust reachability does not encode external format or future-proof authority.
  Check receipts and command entrypoints before deletion.
- Preserve public APIs or version their removal explicitly.
- Do not proceed from classification into TD-104 feature gating without the
  separate decision and measured benefit.

## Independent Review Brief

Look for removed external consumers, compatibility APIs mislabeled dead, and
accidental algorithm changes hidden in deletion diffs. Score 1-10.

## Completion Record

- Implementation commit:
  `8149b11050ff462a54e12ac188d62b7753e1ed9d`.
- All 368 TD-005 rows are classified: 236 proof-only, 17 compiler-only,
  113 compatibility API, 2 live-unused, and 0 obsolete.
- The two live-unused owner groups were removed. The canonical baseline moved
  monotonically from 368 to 366 rows with zero additions.
- Independent review: `8/10`; its one low-severity generic-symbol evidence
  finding was corrected, with zero open findings.
- Remote verification: 2,359 / 2,359 correctness and package tests PASS;
  semantic and infrastructure failures 0 / 0; lint 366 / 366 and zero non-dead
  diagnostics.
- Cold compile measurement did not prove a speed change: 43.91 s before and
  43.83 s after, with unchanged rebuilt-unit counts. Target artifacts decreased
  by 10,742 bytes. Retained proof/compiler isolation remains owned by TD-104.
- Completion receipt:
  `tech_debt/evidence/td008-completion-v1.json`, SHA-256
  `56e1d2b8f59d22c5386d42ee7f2acfce6160d5824bb03035def5ed4cd198a947`.
- No release build, installation, service restart, or live runtime change was
  performed.

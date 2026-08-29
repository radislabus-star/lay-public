# TD-007: Reconcile Superseded Semantic Contract Tests

Status: `READY`
Priority: `P0`
Class: behavioral proof
Size: `XL`
Depends on: TD-006

## Why Now

Most full-suite failures are old semantic expectations. Leaving them red makes
it impossible to know whether later correction changes are regressions. Updating
assertions mechanically would be equally unsafe.

## Evidence

The TD-006 hermetic ledger seals 116 failures: 30
`correction_ranking_admission`, 28 `ime_authority`, 23
`typing_assist_surface`, 9 `nanda_l2_field`, 7 `remaining_semantic`, 6
`architecture_integration`, 5 `edit_safety_contract`, 3 `candidate_birth`, 3
`nanda_l3_context`, and 2 `phrase_boundary`. Typical drift includes old source
IDs, an old selected candidate, an old veto reason, or an expectation that a
candidate applies without current authority evidence.

## Target State

Every non-ignored semantic test expresses the current documented authority
contract and passes hermetically. Superseded experiments are retained as
historical evidence outside the live test denominator. No fixture-specific
runtime branch is added.

## Scope

- Group failures by first shared mechanism: candidate birth, lattice retention,
  ranking/admission, edit safety, or source/receipt naming.
- Trace each group through `L1.1 -> L2 -> L3 -> L4 -> DecisionCore -> verifier`.
- Compare each old expectation with current architecture documents and active
  integration contracts.
- Maintain a complete ledger for every baseline failure:
  `test -> first loss layer -> authority source -> disposition -> replacement
  proof`. Removing, renaming, or moving a test without a ledger row is forbidden.
- If runtime violates the current contract, fix the mechanism and rerun the
  whole group.
- If the test encodes a superseded contract, replace it with a current invariant
  or move the historical case into an explicitly non-authoritative archive.
- Preserve per-error-class quality requirements and false-accept gates.

## Reviewable Milestones

Each milestone gets its own fresh-context review, completion record, commit, and
push before the next begins, while this task remains open until all are green:

1. `correction_core` and candidate-source failures.
2. `ime_correction` failures.
3. Nanda L2/L3/bridge/candidate failures.
4. text-edit, typing, phrase, and remaining semantic failures.

Before milestone 1 edits source, freeze the exact canonical heldout command and
input hashes from the current L1.1 proof route. The latest accepted full
denominator is the 13 x 20,000 evidence referenced by
`docs/l1-l11-exact-peak-search-refactoring-plan.md`; copy its receipt path and
command into the milestone record rather than inferring them.

## Non-Goals

- No one-example runtime branches, literal phrase rules, or hand-weighted
  exceptions.
- No weakening of SafetyGate, edit-plan validation, or verifier authority.
- No mass snapshot update to whatever the runtime currently emits.
- No aggregate result that hides a failing damage class.

## TDD Plan

For each mechanism group:

1. Freeze current failing cases and the first loss layer.
2. Add one minimal mechanism-level contract test if none exists.
3. Repair the mechanism or replace the superseded assertion.
4. Run the whole group plus fixed heldout metrics.
5. Reject a change if aggregate or any required error class regresses.

## Acceptance Gates

- Hermetic correctness lane has zero unignored semantic failures.
- `cargo test --all-targets` is green when run through the canonical isolated
  runner, excluding only explicit live/performance lanes.
- Fixed heldout reports include aggregate and every required damage class.
- Every damage class has strict `unique top-1 > 95%`; lattice coverage, clean
  preservation, false certainty, package/RSS, and latency remain separate
  conjuncts and all required gates pass.
- `false_accepts = 0` where the existing contract requires it.
- The 116-row ledger closes with exactly one evidence-backed disposition per
  baseline failure and no dropped replacement proof.
- No runtime source contains fixture text or test IDs.
- Architecture documents record changed contract interpretations.

## Risks And Guardrails

- Old tests may catch real regressions despite stale names. Determine the first
  loss before changing the assertion.
- Current runtime behavior is not authority by itself.
- Keep candidate retention distinct from automatic-apply authority.

## Independent Review Brief

Review findings first. Sample every failure cluster, inspect architecture
evidence for each changed expectation, and search runtime diffs for fixture
literals or weakened gates. Score 1-10.

## Completion Record

- Commit: pending
- Review score: pending
- Verification: pending

# TD-007 Milestone 1 Independent Review V2

Status: `FINDINGS_REPAIRED_AFTER_FINAL_ALLOWED_REVIEW_PASS`

The same independent reviewer performed the two allowed passes. No third pass
was requested, so the recorded score is not inflated after the fixes.

## Pass 1

Score: `4/10`

Material findings:

- source semantics and installed-runtime mutation were conflated;
- eight package-bound cases lacked executable pinned proof;
- 13 compare-only FullWave cases were not active current invariants;
- the unique-nearest replacement proof did not assert unique rank and verified
  transition authority;
- ledger disposition vocabulary and base identities were stale.

All five groups were repaired before pass 2.

## Pass 2

Score at review time: `5/10`

Findings:

1. the first package proof was vacuous because absent candidates could pass;
2. archived lane observations did not contain the comparator verdict or the
   exact known-failure ledger identity;
3. machine evidence still used an ambiguous `runtime_authority_changed` field;
4. the package receipt was bound to an intermediate tracked diff.

The reviewer independently confirmed without findings:

- no fixture-specific runtime branch;
- no SafetyGate, DecisionCore, edit-plan, or verifier weakening;
- exact baseline partition `35 closed + 81 remaining = 116`;
- frozen disposition counts `5 / 22 / 8 / 0`;
- all 13 FullWave replacements are active correctness tests;
- no install, restart, desktop contact, or live-runtime mutation.

## Post-Review Repairs

Finding 1:

- the mock L1.1 seed was replaced by the real isolated pinned L1.1 service;
- one exact L1.1 surface must be retained with owned canonical/Productive
  provenance and remain `SuggestOnly`;
- the other seven historical surfaces must produce no decision and no selected
  transition; any emitted candidate must be owned Productive L2 evidence and
  `SuggestOnly`;
- the eight surface/outcome contracts are frozen in
  `tests/fixtures/td007-package-set-v1.json` and copied into the receipt.

Finding 2:

- `scripts/test_lanes/cli.py` now writes observations only after exact-known
  comparison succeeds;
- each archived observation includes `verdict`, manifest SHA, known-ledger SHA,
  full and lane-local failure counts.

Finding 3:

- lane and package receipts now declare
  `runtime_authority_scope = installed_live_runtime` and
  `installed_runtime_authority_changed = false`;
- the milestone completion evidence separately declares
  `source_runtime_semantics_changed = true`.

Finding 4:

- the replacement package receipt binds the complete test source closure,
  controller, case manifest, package identities, and unchanged closure before
  and after execution instead of an intermediate whole-worktree diff.

The final acceptance decision relies on the post-repair executable gates and
their immutable hashes, not on a claimed post-hoc review score.

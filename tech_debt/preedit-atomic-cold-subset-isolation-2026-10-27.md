# P2 — Make the atomic preedit materialization fixture independent of preceding tests

Status: OPEN, detected while verifying short-function Space conversion. This is
separate from the Space feature; do not change its ranker or transport as a workaround.

The unchanged test `preedit::tests::atomic_output_refresh_materializes_candidates_before_publication`
at `src/bin/lay_ibus_engine/preedit/tests.rs` expects suffix `ерка` for `пров`.
Its complete canonical IME cohort passes, but a fresh hermetic subset of native
document, Space-prefetch and preedit tests produces `ерить`. Both outputs are
reported exactly; no assertion or source model has been weakened or retrained.

Evidence on the same final preparation source
`5a6f6719ccf88ebd891bbe9f867cd85912773260aeebde6ac8f7d578afaf45ca`:

- CROOT/short-function-space-preview-gates-v3/targeted-engine.json:1100/1100 PASS.
- CROOT/short-function-space-cold-subset-v3:177/178 PASS; only the atomic test fails.
- Earlier filtered preview:176/177 PASS with the same suffix mismatch.
- Raw failed Rust output stays in each `tests.log` or `targeted-engine.log`.

CROOT is `/home/e/projects/lay-development-runner/kde-install-control-20261020`.
The versioned discovery/execution APIs are `scripts/test_lanes/discovery.py`
and `scripts/test_lanes/execution.py`; use `cargo_discovery` and `run_target`,
which preserve process isolation. Select correctness rows whose names contain
`native_document`, `space_autocorrect`, or `preedit::tests::`, in a new sandbox.
Use existing remote resource and Cargo guards. Never run against live learning.

First determine which missing predecessor, initialization state or usage-prior
fixture changes the candidate set. Compare cold single-test, cold subset and
full target with provider readiness/candidate evidence; a full-target PASS
alone does not explain the difference. Do not call it a production ranking
regression or a proven old defect without an appropriate before/after control.

Minimal solution choices:

1. 9/10: make this publisher test establish a bounded deterministic candidate
   fixture through existing test/provider facilities, in an isolated process
   where needed. Keep exact selected-surface and published-payload assertions.
   Candidate ranking quality remains covered by its own fixed proof set.
2. 8/10: if evidence identifies a missing documented provider initialization,
   make that test prerequisite explicit using its current owner and recheck
   the cold single/subset/full target. Avoid a broad warm-up without evidence.
3. 2/10: accept either suffix or remove the surface assertion. Rejected because
   it can hide a publisher/candidate mismatch.
4. 0/10: change production scores or add a literal word condition. Forbidden.

Acceptance: unchanged or stronger semantic assertions pass cold single-test,
fixed cold subset and complete IME cohort with fresh isolated data; no live
cleanup, runtime model, ranker, delivery, desktop or display behavior changes.
Keep the failed receipts, record the causal predecessor/state and bounded fix.

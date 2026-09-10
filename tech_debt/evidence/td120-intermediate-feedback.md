# Parent intermediate remote checks — not final review

2026-09-05. While Sol's runtime patch was still in progress, parent transferred
an isolated source snapshot and compiled it remotely through both guards.
No local Cargo. Remote source manifest:
`/home/e/projects/lay-td120-121-SUdh2I-intermediate-source.sha256`.
Log `/home/e/projects/lay-td120-121-SUdh2I-intermediate.log`.

Compilation2.44s PASS; current ten tests8 PASS/2 FAIL,0 ignored,0.34s.
This is not final source parity, full acceptance or independent review.

Two concrete findings to resolve before the final test run:

1. `retire_current_word_autocorrect_suppression` matches CurrentWord against
   `self.committed_tail.autocorrect_suppression.take()`. On ExactReplay or V1
   the let-else returns AFTER removing the transport scope. Ordinary retirement
   must leave other variants untouched. Exact temporary-empty control catches
   local scope loss (shared state still contains exact scope at that point).
2. `td120_duplicate_open_result_preserves_existing_guard_once` constructs
   daemon_bridge(backspaces0, replacement b) on tail a; its FIRST edit is refused,
   so this does not reach duplicate semantics. Use a legitimately authorized
   open-result edit and prove duplicate handling without weakening runtime
   admission or removing the existing-guard preservation assertion.

Expanded tests were still being authored when this snapshot ran; these are
implementation feedback, not one of the fresh independent review passes.

## Expanded intermediate v3

Remote log `/home/e/projects/lay-td120-121-SUdh2I-intermediate-v3.log`.
Build2.48s,27 selected,19 PASS/8 FAIL,0 ignored,0.40s. Four atomic tests pass;
real callback/foreign-ABA test fails at ibus_interface.rs:352 expected frame1
but got0. Three tests use refused initial daemon_bridge edits (open duplicate,
committed producers, failed/noop guard preservation) and must use legitimate
producers, not bypass transition validation. Both ExactReplay and V1 cases
catch the same take-on-nonmatching-variant bug. Boundary test line544 also
fails; check which fixture character remains an existing lexical/layout
continuation before treating it as a hard boundary.

O6 guard absence passes all profiles. O6 apply fails at closed exact preparation
before any output; separate test agent is investigating exact prerequisites
without fabricating a certificate or weakening expected conversion.
Fix implementation/fixture mechanisms once, rerun full27+ denominator. Final
source parity is not claimed while the implementation remains in progress.

Parent noticed the later callback test switched its initial failing
`process_atomic_key_event` preparation to `td120_install_test_pending` that
inserts a fabricated pending entry. Keep such settlement injection only as a
lower-level supplemental test. It must NOT replace the required actual atomic
prepare -> real FocusIn/cancel/ABA callback -> settlement route proof. Diagnose
why the valid preparation was unhandled and correct the fixture/preconditions;
do not mark its missing producer/output coverage PASS using a hand-built entry.

## Intermediate v4 and full-bin regression

Snapshot v4:28/28 TD120 tests PASS, build2.48s, tests0.38s; O6 exact Space
matrix including all profiles and protected control now passes. Not final
parity while Sol is finishing edits.

Full current bin test snapshot:310 executed,309 PASS/1 FAIL,0 ignored,
14.54s; log `/home/e/projects/lay-td120-121-SUdh2I-bin-regression-v1.log`.
Failure `atomic::tests::deep_clone_isolates_shared_and_engine_state` at line551
still expects live.active_path.is_none(), but common fixture was changed to
bind focus. Preserve its isolation assertion by comparing live owner/state to
its real pre-clone baseline; do not delete the test or weaken clone isolation.
An intentional lock-poison panic elsewhere is a passing negative control,
not a second regression failure.

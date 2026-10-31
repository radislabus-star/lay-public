# 1.0.123 — preserve clock uncertainty in native diagnosis

Preflight 2026-10-31T11:18:09.110963+00:00,immutable baseline 5c6eee98698f1bbd1187f31434bea80d7569d98e/published122.
Offline tooling only. Three allowed Python paths;all runtime,model,GNOME/KDE
launch/config,update clients and delivery code are frozen by the owning ADR.

## First loss and dependency map

```text
Saved r19 actor result.json (no new typing)
  late_targets:observed,next_gesture_start;clock_mapping_uncertainty_ms
  → scripts/native_diagnostics/receipts.py::assertion_class
       currently discards margin → unconditional TARGET_AFTER_NEXT_GESTURE
  → summarize::cases/groups/rapid_timing
  → existing scripts/lay-diagnose.py report CLI (unchanged)

scripts/tests/test_native_diagnostics.py (22existing tests outside root)
  +5regressions → tests/test_native_diagnostics_suite.py::load_tests
  → existing canonical unittest discover -s tests → CI
```

Saved actual capture(CACHE/actual-release120-firefox-wire-result.json,from120
investigation) reports4late targets. Signed observed-minus-next-start deltas
are-2.090,+19.011,+80.325,+26.285ms with recorded margin3.205ms. Classify3outside
recorded margin and1overlap;the original row staysFAIL. The margin is not an
independently calibrated physical error bound. Preserve missing causal capture
and first_failed_transition UNKNOWN;this does not diagnose an IME cause.

## Minimal implementation and proof boundary

Reuse current report,grouping,privacy,finite-number and CLI contracts. Add
per-target numeric evidence and aggregate ordering inside the existing
rapid_timing block,so CLI exposes it without a new command/runner. Bound literal
parsing;malformed/missing/contradictory data remainsunverified. No eval or raw
text export. Preserve original status/counts,actor and input deadlines.

A standard unittest adapter brings the same22old+5new tests into existing
Python discovery;do not copy/move their bodies or create another runner. It
rejects an empty suite. Private prototype27GREEN and real1FAIL/3after1overlap
proofs are retained but not promoted as canonical/native evidence. Now prove
canonicalRED on oldparser,thenGREEN on the same tests and actual savedCLIreplay.
Full release gates/build/load/nativecontrols/CI/publication/updaters follow.
Historical native344/15/81andTD168remainunchanged;no native-fix/speedup claim.

Canonical RED:CROOT/release123-native-timing-red-v1/result.json.27cases;19expected assertion failures across5new methods,zero errors/skips. All22old test bodies and1338other files intact. Parserstillbaseline duringRED. Tested private parser copied only after this result;canonicalGREENnext.

Scope guard rejected the firstGREEN preparation before snapshot/upload:offline receipts.py falls into generic shared category. Restored exactbaseline parser,declared that already-authorized exactfile as shared tooling seam in the same owningADR/scope,then reapplied identicalapprovedbytes. No guard weakened,no outsidefile allowed,no tests/install ran with rejected scope. CanonicalREDremainsvalid(same source);GREEN preparation retry follows.

CanonicalGREEN:CROOT/release123-native-timing-green-v1/result.json;all27tests PASS,real CLI savedreceipt retainsFAIL1and3after-recorded-margin+1overlap,causalUNKNOWN preserved,totalworker1.229s. All22old test bodies/1338otherfiles unchanged. Versionmetadata123;fullrelease gates next.

## Verified outcome 2026-10-31T11:52:50.627959+00:00

All17release gates PASS;3617Rust/34targets/494isolates(canonicalexit0),
459PythonPASS10SKIPof469,all10release artifacts0warnings. Source1329SHA
24fbb98d13b4d9a2f139e9a7bc65780f34a48147de33e12f720c8506f7e48fcd.
The new root adapter brings all27diagnostic tests into fullPython/CI. All22old
test bodies and1338otherfiles frozen;only3declared Python paths change.

Remote123installed;all10installed hashes,actualloaded daemon/IME andexisting
factorypath verified after2/2frozenr19native controls(4flips). Config/layout/
globalIBus preserved,localGNOME untouched. ActualsavedCLIreplay remains1FAIL
with3outside-recorded-margin+1overlap andcausalUNKNOWN. No fullmatrix or speedup
claim. Publication pending;structured final-verification JSON owns details.

Published 2026-10-31T12:23:35Z at https://github.com/radislabus-star/lay-public/releases/tag/v1.0.123. Commit50593607;all4CI/SecurityPASS,identicalannotatedtagmirrors/peeledcommit,bothactualinstalled menuupdaters/sources verified. No local install/restart. Ten requestedstable releases114–123 independentlyreadback fromGitHubAPI.

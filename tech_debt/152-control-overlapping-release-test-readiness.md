# TD-152 — establish readiness in the overlapping-release regression

Priority: P1, release CI blocker. Status: DONE_REPORTING_SOURCE; original CI cause UNKNOWN, fresh CI pending.
Baseline b411182292fcc0ed46a6d0e0127a28f5ae132486; installed87 unchanged.
Root implements; agents review only. No production change is authorized merely
by a flaky-looking fixture. C08/C10 and the existing deferred-display owner.

## Facts and hypothesis

GitHub CI38901069439 failed in Hermetic Rust correctness/package lanes. Only
unexpected identity: bin:lay-ibus-engine / context_admission::adapter::tests::
word_scope::residuals::td121_overlapping_handled_releases_publish_only_the_latest_completion.
Same source remote full3210/3210 passed; MSRV and Security38901069484 passed.
Local raw failed log: /home/ubu/.cache/lay/development/td150-install88-20261016/CI-b411-failed.log.
CI API returned zero artifacts and the step log omits the assertion/panic.
Therefore assertion cause is UNKNOWN, not an established runtime regression.

Fixture prewarms an independent general completion request, drives overlapping
handled presses and expects the latest completion immediately on last release.
Possible causes: missing exact cache/readiness prerequisite, a one-second outer
P2P fixture deadline including materialization, or an actual stale publication.
Inspect controlled callback order and raw failure before choosing a fix. TD-151
Space failure is separate unless common causal evidence is produced.

## Historical readiness options — deferred until cause is proved

1. Recommended9/10 if proven: establish exact readiness using the existing real
   materialization/cache entrypoint BEFORE the release being tested. Explicitly
   cover key-before-ready separately; keep all visible effect and current-frame
   assertions. No sleeps, deadline enlargement, hidden retry or runtime change.
2. 7/10 if the runtime is wrong: a minimal correction in the existing deferred
   publication/owner boundary with a real controlled-order RED and adverse tests;
   requires a new explicit decision and full input/native acceptance.
3. 2/10: add retries, raise timeouts, ignore the failing identity or weaken the
   completion assertion. Rejected: masks unknown cause or changes the contract.

## Historical readiness execution — superseded by narrowed reporting repair

After the active Chrome native lease has closed, run the exact existing identity
once in the guarded remote isolated runner and save raw panic/effect/timing.
Use test-only callback stage diagnostics if needed, then deterministic injected
material readiness or withheld completion through existing helpers. Do not run
repeated random trials looking for a favorable result. Preserve both release
orders, no display on the first release, one latest publication on the second,
exact buffer, zero learning and empty handled-release set.

If no raw failure reproduces, retain UNKNOWN and pursue a causal prerequisite
proof; a green rerun alone cannot close this task. A fixture change must detect
the controlled missing-ready or stale-frame violation, preserve all test IDs,
classifications, exemptions and all old assertions. Any harness/error reporting
change must preserve bounded raw failures on disk and print only a short error.

Remote focused proof and affected display/cancellation controls, full source,
MSRV/lints/canon as required by actual change, independent context-free review
max2 passes. Publish and require a genuine fresh CI result after the repair;
do not re-run the old CI until green. Record elapsed repro/focused/full times
separately. Runtime/routes/models/user config remain unchanged for a fixture-only
fix. If a production correction is needed, replan its owner and native scope.

Consequence analysis: fixture readiness has no shipped runtime CPU/RSS, latency,
package/reload or training effect. Main risk is testing only a warm happy path.
Keep explicit pending-material and revocation tests in the selected fixed set;
never synthesize a candidate by writing internals or omit before-ready behavior.
Rollback only the test change; no user data or accepted runtime to roll back.


Independent fresh-context plan review pass1/2:9/10,
/root/td151_td152_plan_review_20261016. Exact prerequisite is scene-aware
materialization/cache from the actual PrecognitionInput (scene/context/limits/
active-composition/material generation), not a generic superficially similar
warmup call. If a prerequisite is proved and repaired but the original panic is
unavailable, record fixture proof, new CI result and original cause UNKNOWN
separately. Preserve genuine pending-worker and revocation controls.


## Reproduction result and narrowed repair

Single unchanged overlapping-release baseline in td151-reset-diagnosis-359fg2lz
PASS; no original assertion reproduced. Do not invent a cache mismatch or alter
that fixture/deadline to obtain green. Its existing key-before-ready and revoked
frame controls are preserved in the TD151 focused13/13 set. Original CI cause
remains UNKNOWN; a fresh CI result on the actual final source is still required.

A separate information-loss defect is now proved: compare_known_failures raises
before reporting any raw assertion to CI. Full raw blocks already exist on disk,
but CI's compact exception contains identity only. Extend the existing contract
test to require a bounded raw first-failure excerpt for unexpected/signature-
mismatched cases while preserving fixed-only behavior and all failure checks.
Remote td152-reporting-red-4a4ljc9c:16/17 PASS,0.137s; intended RED is missing
'prefix expected marker suffix' in the existing contract error, not runtime.
Archive d00088814eb2f27c585d1ba2a56ab427e4d7df2e8fb1e9954b164579eb5bdd26.

Recommended minimal repair9/10: append JSON-escaped first observed raw failure,
at most2000 characters plus explicit truncation marker, to the unchanged contract
error. No new dependency/workflow action, artifact upload, test classification,
exception or runtime change. Keep full logs on disk. This closes the proved CI
evidence-loss defect; it does not establish or claim the cause of the old CI
failure. If the new CI fails, its real assertion determines the next correction.
A speculative fixture warmup is deferred until that missing prerequisite is proved.


Plan review2/2:9/10. Final selected repair anchors at the existing panic-location
pattern when present, so long preceding stdout cannot crowd out the assertion.
It selects only unexpected/signature-mismatched evidence; fixed-only rejection
is unchanged. Added contract cases cover long preamble, bound/truncation, missing
raw block and an unchanged known failure preceding the unexpected failure.
Original readiness plan above is historical/deferred; no runtime or fixture edit.
Decision2026-10-16-bounded-ci-failure-evidence.json records the reporting-only scope.


Final remote reporting contract17/17 PASS (0.059s unittest execution) inside
td151-final-focused-ut_63nbx/PYTHON.log and the same frozen source archive
as final focused13/13. No old test ID/classification/deadline changed. Full source
and genuine final-source GitHub CI still pending. Original CI cause UNKNOWN.


Full remote source td151-full-source-gvt96j4h/RESULT.json3211/3211 PASS,
3175correctness+36package, zero semantic/infra failures,693.46s. Manifest3237
keeps all old classifications and adds only TD151's real adapter regression;
15ignored/11performance separate. ActualMSRV1.88both, strictlints530/354same,
canon15, architecture/audit50, Nodeobserver, harness122(oneexplicitSkip), panelPASS.
Archive844618fca16f9ca6a39e1e6dae964313362884fdadaa1c5c1f0f392689e2bd5e.
Full source metadata fetched/SHA bound; final graph and strengthened Python
encoding/fixed-only assertions are checked separately in final canonical gate.
Those added test assertions do not change runtime or test classification.


Code review1/2:9/10, no P1. P2 encoding/fixed-only cases added to the existing
contract test: quotes/backslash/CR/LF/ESC JSON roundtrip, fixed-only no excerpt.
Current17/17 rerun PASS in final canonical td151-final-canonical-1gumzyss,
48.56s total with AST15 helpers/canon15; no runtime change after full3211 proof.
Original CI cause remains UNKNOWN; final-source CI is a separate required gate.


Final independent code review2/2:9/10, no open P1/P2,
/root/late_managed_release_plan_review_20261013 (reused thread; new context-free
thread launch was refused by platform). Root implemented the grouped P2 test
repair;17/17 current cases pass. This closes the proved evidence-loss defect,
not an unobserved completion-runtime defect. Fresh final-source CI remains
required before installation; old failure and its unknown cause stay recorded.


Final release closure —2026-10-16: functional source77af7ade, full3211/3211,
CI38916364078 +Security38916364185 firstattemptSUCCESS, independent review9/10.
Installed matching CLI/daemon/IME/panel1.0.88;6actualpayloads match native-tested
e34 bytes. Whole432=349P50F33B and normal optional80=58P18F4blockedpreconditions
remain explicit, no all-client acceptance claim. GlobalIBus/L1.1/L3, models,
settings/sources/mode unchanged; no localfunctionalkeys. Rollback/procSHA evidence:
/home/ubu/.cache/lay/development/td153-install88-20261016/INSTALL.json.
All old negative receipts and each card's narrower source/ACL/ARM scopes remain.

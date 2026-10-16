# TD-150 — Retain completed native replay evidence across matched Reset

Status: DONE_SOURCE3210/3210, independent code review9/10; matched build/native/install pending. Priority: P1 release gate.
Baseline: e6f3227c50f13b443fc161e11b52e8ff56feaa33, version1.0.88.
Root implements; independent plan/code review, at most two passes each.

## Captured failure and first failed transition

The one permitted unchanged-byte Firefox confirmation independently failed
fullscreen/space/field8 (plaintext contenteditable), fourth of five Double
Shift gestures. The first three exact intermediate targets were observed.
Fourth expected the Russian word plus its original boundary; actual remained
the previous English projection. Fifth cannot satisfy the required transition
from that unchanged source. This is not cleared by the earlier46 positive
single-letter controls, and no third unchanged-byte full rerun is permitted.

Primary remote receipt:
/home/e/projects/lay-development-runner/firefox-legacy-native-20261012.6onP5c/firefox-standard-fullscreen-7k5s0h3f/RESULT.json
Per-cell trace-space-8.jsonl SHA256
7124aec5c834c8c428e1f85cafe81afebd789f2001f92f9d00d5b6d240461622.
Separate immediate read-only copy of the receipt/current trace:
/home/ubu/.cache/lay/development/td149-install88-20261016/native-recheck-passive-early/.
The per-cell trace is bounded512KB, completeness UNKNOWN; the relevant replay,
matched Reset and explicit refusal are present, so no missing-event inference
is needed for this scoped observation.

Trace: source activation761, exact suppression epoch7502. Reset occurs during
native deletion/insertion, including after four replacement letters. Native
closing Space and release settle KnownStart at epoch7516, with seven mirror
characters. Matched Resets30619/30621 and client snapshots of length1 then7
follow. Both rereceipt captures reject missing predecessor. Bridge nonce2783
is received/live; ManualToggleV3 refuses context_authority, allowed=false,
refresh_allowed=false. No duplicate output or alternative backend is attempted.

First source gap: capture_context_reset_rereceipt_candidate falls back to the
existing completed exact replay proof, but rejects a mirror ending in whitespace.
The ordinary pre-Space candidate may be unavailable when an earlier Reset
retired the word prefix. The fully matched native replay still has its bounded
same-owner proof; a deterministic callback-stream RED must verify this cause
before changing runtime. Do not claim the original normal/short/field4 failure
has this same cause without its own evidence.

## Options and selected minimum

1. 9/10: allow the existing completed exact replay proof to seed the existing
   matched-Reset rereceipt for a closed word. Reuse paired local/shared replay,
   source token, exact full replacement, finite expiry, native projected count,
   epoch/path/owner identity, reducer Reset provenance and actual exact client
   end-caret confirmation. No new owner, cache, timer, executor or route.
2. 5/10: delay subsequent gestures or increase deadlines. Changes input latency
   and does not restore missing evidence; not selected.
3. 1/10: ignore Reset, admit unknown suffix generically or change output route.
   Rejected: weakens C03/C05/C08 and violates the route constraint.

## Minimal TDD and verification

- Extend the existing inherited closed native replay adapter harness. Inject a
  real matched Reset after a nonfinal replacement letter, then finish the
  actual native callbacks including Space/release. Supply late partial and
  exact client receipts through the real existing observation route. Assert
  current unmodified source declines the next exact manual projection; after
  repair it delegates through the existing owner only after confirmation.
- Cover ASCII/NBSP boundary and Unicode external prefix with exact surfaces,
  mode, proof, disposition and emitted-effect assertions. A missing receipt,
  wrong prefix/selection, incomplete replay, extra epoch, expiry, owner/focus
  change and non-Reset lifecycle remain unable to grant an edit. Reuse existing
  negative controls where they already prove these boundaries.
- Preserve default Double Shift, opt-in Double Space and undo, privacy, pending
  undo priority, both source token spend and existing exact verifier. Fixture
  strings are not runtime conditions; do not normalize arbitrary whitespace.
- Remote guarded focused RED/green and controlled old-selector reversion;
  complete fixed source correctness/package proof, pinned MSRV/lints and canon
  with unchanged classifications/exemptions. No local functional keys/builds.
- Record a new explicit decision before modifying the protected owner. Update
  this owner with proof paths, exact tested hashes, limitations and review9/10
  target. Refresh the canonical graph remotely, then build matched new bytes.
- Changed-byte ten-field browser acceptance and latency remain a separate
  required gate. Keep original full matrices and both distinct Firefox failures
  immutable; no best-row splicing or pass-on-retry claim.

## Consequence analysis before production edit

Candidate/lattice retention, rankers, models, packages/delta reloads and learning
remain unchanged: this proof concerns an explicit literal manual projection,
not a new correction/feedback producer. No new allocation container, timer,
cache, queue, owner or executor is introduced; the existing bounded proof clone
is reused only after a complete same-owner replay, with unchanged700ms expiry.
CPU/RSS cost is the existing finite proof predicate/clone for this formerly
rejected branch; actual latency/tails require changed-byte native measurement.
The material risk is stale/non-Reset lifecycle falsely restoring word authority.
Preserve local/shared equality, source token, exact replacement/count/epoch,
path/focus/owner/expiry and matched reducer Reset; partial observations retain
inert evidence only, and exact full end-caret client receipt is still required.
Future package/online changes cannot extend that proof: existing identity and
input invalidation remain decisive. IME and daemon consume the same handoff;
no backend fallback or new delivery route. Missing/contradictory receipt,
selection, lifecycle change, expiry and partial output fail closed without
retry or alternate edit. Roll back only this capture change and its tests by
Git revert; user learning/models are not rolled back. Maintenance cost is one
removed redundant rejection plus causal coverage in the existing test module.

Independent fresh-context plan review9/10, /root/td150_completed_replay_plan_review.
Requested consequence paragraph is recorded above; no production change yet.
First fixture run td150-before-code-zdwjcuwo:2/3 PASS, new test fails an incorrect
epoch-equality assertion after existing manual handoff publication (15 versus14), not
the intended authority assertion. It does NOT establish RED for this defect.
The simple synchronous Reset schedule already succeeds on source baseline.
Refine using existing deferred observer/callback helpers: receive second-letter
callbacks, observe Reset ahead of their dispatch, finish those callbacks, then
handle the matched Reset. This preserves the actual race rather than adding
a synthetic state setter. Exact NBSP surface acknowledgement leaves the replay
epoch unchanged; existing handoff publication advances it exactly once.

Genuine RED td150-observer-red-wtmv6a4m/RESULT.json:2/3 PASS, new regression
fails the intended exact client receipt/manual-handoff authority assertion,
not setup or normalization. Archive3aecb3f0a9ff968a8f6e64fcd240351315f1733aab5afe3b76489292d117e999;
14.23s focused cycle. The actual observer revokes the predecessor before deferred
native callbacks dispatch; the completed closed-word proof is then not captured
by either late matched Reset. Runtime remains baseline in this RED receipt.

## Minimal implementation and focused proof

Production diff removes only the whitespace rejection from the completed-replay
fallback and adds its reason comment. Paired replay/source/expiry/epoch checks,
ordinary candidate capture, matched Reset, exact full SSI, token spending and
the existing delegate remain unchanged. No route/model/ranker/learning change.

Final focused receipt td150-focused-proof-vfotdhss/RESULT.json:11/11 PASS,
26.61s including controlled reversion. New regression covers24 actual schedules:
empty/Unicode prefix × ASCII/NBSP boundary × positive plus selection, foreign
prefix, extra epoch, expired paired proof and local/shared mismatch. Negative
cases assert refusal and no CommitText/DeleteSurroundingText. Eight existing
native/revocation/capability adverse controls plus both late-receipt tests pass.
Controlled restoration of only the removed guard makes the new positive case
fail the same authority assertion (1/1 detected); current source restored and
SHA verified. Archive53e0722f541dd5d87771a52c0e7ff98a6833c38f7d24de33c99dd20ae437f10c.

Retained intermediate focused attempts: td150-focused-green-i269mjsp2/3 and
td150-focused-final-k1jhrm3k10/11 failed overstrict epoch expectations in the
new fixture, after the authority assertion succeeded. The final assertions
check the existing exact counter contracts (ack0, handoff+1) and exact surfaces;
no production guards or full-gate expectations were relaxed to hide a failure.
Original native failures remain unresolved by a focused-source result alone.

## Complete source gate and independent review

Fresh remote full receipt /home/ubu/.cache/lay/development/td150-full-source-5m8befqy/RESULT.json
and SUMMARY.json:3210/3210 PASS (3174 correctness+36 package), zero semantic
or infrastructure failures,694.58s. Manifest3236 retains every previous identity
and classification, adding exactly the new regression;15 ignored/11 performance
remain separate. Actual Rust1.88 default/lexical-compiler all-target locked/offline
checks, pinned strict default/research lints with unchanged530/354 semantic
inventories, canon15, architecture/audit50, Node observer, harness122(one explicit
skip) and both panel syntax checks PASS. No exception added. Frozen archive
b4c76d85d909dea315dfe5ec4235de719a7da6f45e10b6842549617a4d60f383.
994/994 tested input files match locally, including the generated graph receipt;
the993 functional inputs remain bound for the separate final canonical refresh.
Cargo.lock remains the audited rustls46 identity; no dependency change here.

Fresh-context independent code review /root/td150_completed_replay_code_review,
pass1:9/10, no material P1/P2 or required production correction. Reviewer checked
source-token spending, partial receipt inertness and non-Reset revocation floor.
Root implemented all changes. Its minor documentation requests are reflected in
the decision and TD-121 experiment index. Full source result arrived after this
source review; it does not certify new executable/native behavior. Matched new
bytes and the complete browser gate remain required before installation.

## Stop conditions and deferred scope

If RED does not expose the claimed capture gap, revise the diagnosis before a
production edit. Do not weaken reducers/verifier/SafetyGate or grow a client
branch. A complete native receipt still reports every FAIL/BLOCKED/missing time;
source PASS does not prove all clients. Optional-rich interop and the untraced
original short-word failure remain open under TD-121 unless independently
resolved. Installed1.0.87 and user configuration/models remain unchanged now.

Final canonical refresh td150-final-canonical-w9iu0xaq/RESULT.json PASS48.15s,
exact new decision coverage+15canon tests. Four canonical files fetched/SHA
verified;993 functional inputs still match full proof. Nine private release
adapters parse under the remote guard; no native keys or installation yet.

## Published source and matching changed-byte candidate

Source b411182292fcc0ed46a6d0e0127a28f5ae132486 published/read back on origin
work branch and public work/main refs; private origin/main untouched.
Final independent source/evidence/private-adapter review2/2:9/10, no material
corrections. Original native actors, body, activity ledger and eighteen original
default wrapper bodies retain their exact hashes; only unique jobs/tags/output
locations and read-only passive capture are adapted. No test or guard weakening.

Build /home/ubu/.cache/lay/development/td150-install88-20261016/BUILD.json:
PASS83.53s, source archivea0620bfaf04e23e8e631d6b3ae6a5aa9a0394eac1b0edd6311841b0c651a3620,
same lock/audit database, audit0. Matching CLI/daemon bytes remain identical to
the earlier88 candidate; IME now has the bounded proof repair, SHA256
d0ef25351c2d0bd0d350bf25d45236450c385a0619a4fe51f285fe5a8e384b13.
CLI ac7f3fbc49d3de30c6fbb82ec676ea55fad4695b8f617479bf652099bf37f344;
daemon625feb964731db33dac2cbcae6fb09873b1b73d382e45606dea64b36d40226a9.
All993 functional inputs match the full source proof. Local installation still87.

Changed-byte REPAIR both full matrices are COMPLETE (nine original jobs each):
Firefox169P28F19B, Chrome179P23F14B, total348P51F33B/432,0NOT_TESTED;
errors empty and owned GUI cleanup/VT1 verified in both collectors.
Original fullscreen/space8 repeat is now5/5 and Firefox standard60/60;
420/420 native DS median98.694ms/p95112.273ms. Chrome406/420 (14 missing)
median82.307ms/p9597.386ms; all prior ChromePASS retained. These are visible
client timings from synthetic evdev write-return, not kernel release latency.

Firefox lost functional/normal/space_en_ru/7 while original short4 passes.
This blocks installation; no unchanged-byte rerun to obtain green. Immediate
raw row/trace and same-baseline comparison are under the build's
space7-failure/ and native/firefox-vs-READV.json. Existing replay repair is
retained; new first-transition investigation is [TD-151](151-preserve-prepared-space-frame-on-matched-reset.md).
Fresh CI38901069439 failed one old overlapping-release fixture (no raw panic
published), despite remote3210/3210; Security38901069484 passed. [TD-152](152-control-overlapping-release-test-readiness.md)
tracks it separately. Original cause is UNKNOWN. Installed87 unchanged.
Optional80 on these bytes NOT_TESTED; older E6 options are separate evidence.

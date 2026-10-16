# TD-149 — Minimal rustls security patch before release1.0.88

Status: DONE_SOURCE after plan9/10 and independent code review9/10; built e6f3227c candidate BLOCKED_NATIVE, follow-up TD-150. Priority:P1 release prerequisite.
Baseline829f47c749159f71057fe3776b4a8141d1b445a2; TD-147 source3209/3209 published.
User authorized sequential fixes/tests/review/push and final installation.
Root alone implements; independent review max2 passes. No IME route, authority,
model, global IBus or user configuration change is authorized by this task.

## Verified cause and evidence

Current Cargo.lock selects rustls0.23.45. Public Security run38873397675 on
05e1e448 fails the unchanged locked audit with four advisories issued2026-10-14:
RUSTSEC-2026-0346,0347,0348,0349. All four upstream RustSec pages list
>=0.23.46 as patched. Exact original CI log:
/home/ubu/.cache/lay/development/issues-final-pass-20261016/security-05e1e448.log.
This is dependency exposure, not evidence that Lay exercises every exploit.
Brotli/ECH/custom session-store applicability must not be invented.

Primary references:
- https://rustsec.org/advisories/RUSTSEC-2026-0346.html
- https://rustsec.org/advisories/RUSTSEC-2026-0347.html
- https://rustsec.org/advisories/RUSTSEC-2026-0348.html
- https://rustsec.org/advisories/RUSTSEC-2026-0349.html

## Options and selected minimum

1.10/10: update only rustls to0.23.46 with the guarded Cargo entrypoint;
   inspect exact lock delta and preserve all compatible dependencies/features.
2.5/10: update the entire HTTP/TLS stack. Larger unrelated diff and rebuild,
   unnecessary for these four independently documented fixes.
3.1/10: suppress advisories or skip audit. Rejected: leaves dependency exposure
   and weakens a required guard.

Select option1. If the patch resolver changes additional packages, identify
why and constrain it before proceeding; do not hand-edit an unverified checksum.
Do not alter Cargo.toml features, rust-version, security deny policy or baselines
merely to obtain PASS. Keep version1.0.88 for the consolidated unpublished release.

## Execution and proof

1. Complete TD-147's full source gate and code review; commit/push its exact
   implementation before this dependency change. Historical browser432 is not
   acceptance of either changed build.
2. Remote guard only: update -p rustls --precise0.23.46, save resolver log,
   compare package identities/checksums and exact lock delta. No local build.
3. Audit before and after against one freshly fetched RustSec database (record
   commit and clean identity, then use --no-fetch after the patch), with the same
   audit binary and unchanged deny-unsound policy;
   retain database identity and exact result. No ignore list or network-error PASS.
4. Check promised Rust1.88 active targets/default and lexical-compiler feature
   within the shared Cargo disk budget; strict Rust1.97.1 default/research lints,
   unchanged complete correctness/package proof and existing HTTP client tests.
   Verify old test IDs/classification, zero failure exemptions and source parity.
   Do not create tests duplicating upstream TLS internals.
5. Independent read-only source/lock/evidence review1–10; repair concrete defects
   in at most2 passes. Mark DONE_SOURCE only when actual gates pass, commit/push.
6. Build matched three-core artifacts, then the standard remote Firefox/Chrome
   ten-field native matrix plus new shortcut cases and timings. Keep all failures,
   blocked cases, missing timing and native-vs-source scopes separate. Local
   reversible installation preserves source selection/config/models/protected
   processes, and verifies loaded version/bytes. No accepted-native claim early.

## Risks and stop conditions

The patch recompiles TLS dependents; source/native results from older bytes cannot
be transferred silently. Shared-cache size must remain below12GiB; inventory
before any disposable-cache cleanup. A current CI completion-timing failure is a
separate observation, not an authority to weaken completion assertions. Persist
its raw failure if reproduced and repair the first actual transition, not literals.

GitHub issue47 still requires the reporter's current post-login result; this
security task cannot prove the old Orange Pi report resolved.

## Accepted plan review and tool identity

Independent read-only plan review9/10, /root/issue46_feature_plan_review (reused
thread due platform context-thread limit). Preserve before expected FAIL and
after result against one fresh DB; all dependency-resolving checks/builds are
locked, and audit/source/build/install share the exact final Cargo.lock SHA.
No upstream TLS-internals tests or independent repeated native matrix.

Official audit0.22.2 GNU asset is incompatible with remote glibc; retain that
failure as a tool limitation, do not upgrade the host libc. Verified working
musl asset: https://github.com/rustsec/rustsec/releases/download/cargo-audit/v0.22.2/cargo-audit-x86_64-unknown-linux-musl-v0.22.2.tgz
GitHub asset439292222, archive SHA2567fb9497f8594b389e5fce5ef9b92db08432996895b2e0c5a0167a69ed445c428;
binary SHA25666f9c8f530950d106c1869ce27ef5a4008bcea53c7716c3ed8117586337ee7d8.
Actual --version: cargo-audit0.22.2. Private tool path
/home/e/.cache/lay/development-tools/cargo-audit-0.22.2-musl/bin/cargo-audit;
INSTALL.json alongside it. Global PATH, project runtime and models unchanged.

## Actual dependency proof — 2026-10-16

Remote frozen audit/patch receipt: /home/ubu/.cache/lay/development/td149-audit-patch-02br9q8z/RESULT.json.
Expected before FAIL4/4 exact listed advisories on rustls0.23.45; after PASS0
on0.23.46 with identical official audit binary and fresh clean RustSec commit
479c888ce4401313bca61ba4aba2d0f6244c4a09; after used --no-fetch. Both raw JSON
and stderr retained alongside receipt. Resolver changed exactly one package,
only its version/checksum; every other package and rustls dependency list match.
Final Cargo.lock SHA256 ae3ec42af3ee131b6cdd257b3d7dc4a6d401075a519f9cbd6f6e9786ca9c5141.
Original SHA25678e7cfc3f4a32c0c5a3c55fdc30b29014978394ef62a887a6346212331fc9433.
First worker stopped before any audit because remote Python3.10 has no tomllib;
kept failure /home/ubu/.cache/lay/development/td149-audit-patch-0c_opoc7/RUN.log.
Replacement worker compares exact Cargo-emitted package blocks fail-closed; no
OS/Python/project dependency was added. Source/MSRV/native/build/install pending.

Initial full source attempt td149-full-source-dkoquc_y/RESULT.json stopped at
MSRV setup: offline cache lacked the newly selected rustls crate; no correctness
lane executed. Preserve FAIL receipt as infrastructure prerequisite, not a
semantic regression. Add guarded cargo fetch --locked --target x86_64-unknown-linux-gnu
before offline checks, verify final lock unchanged, then fresh full execution.

Second full attempt td149-full-source-raapw8zq: actual MSRV1.88 both configs
and pinned strict lints PASS. Canon stopped before correctness execution because
the decision omitted regenerated manifest anchor from protected_paths. Add only
that exact metadata coverage plus zero-exception ledger; do not change canon
guard, test IDs/classifications or exemptions. Guarded package-release cache
cleanup follows recorded inventory/backups before the fresh complete source run.

## Final source result

Fresh frozen source receipt: /home/ubu/.cache/lay/development/td149-full-source-b0w2y6u9/RESULT.json,
archive308545ee32430f8ca5a001808d60a2f68e6b268fb914a65f5cb8c46f4274077d.
3209/3209 PASS (3173 correctness+36 package), zero semantic or infrastructure
failures; 3235 manifest IDs/classifications unchanged,15 ignored+11 performance
remain separate. Actual Rust1.88 locked/offline all-target checks default and
lexical-compiler PASS, strict Rust1.97.1 default/research lints PASS with identical
530/354 semantic warning inventories. Canon exact-decision coverage and15 canon
tests, architecture, audit50, browser observer Node checks, harness122 (one
explicit real-cgroup test skipped) and panel syntax PASS. Total569.9s.
Local parity993/993 functional/source files; generated architecture receipt
gets separate final canonical refresh. Final lock matches the audited SHA.
Independent code review1/2:9/10, no material P1/P2; other293 package blocks
byte-identical, checksum independently verified against crates.io. Reviewer
/root/issue46_feature_plan_review reused due new-thread platform limit. Root
implemented all changes. No extra review pass required without new code.

Disposable release-package cache cleanup receipt:
/home/e/projects/lay-development-runner/td149-cache-release-cleanup/RESULT.json.
Three87 executable backups verified before cleanup; no running target/release
process; target12701499392→10719010816bytes. Installed runtime and models unchanged.

Consolidated native harness review8→9/10,2/2passes; exact original actors/jobs
retained. Existing activity ledger now checks active VT8+seat identity before
every key-down, releases remain possible; collectors keep full216/40 partial
denominators and missing timings, and actual config/ledger hashes. This safety
check changes harness cost; retain that limit for old/new speed comparisons.
Six remote no-key safety/denominator selfchecks PASS. Private harness receipts:
/home/ubu/.cache/lay/development/td147-release88-native/HARNESS-REVIEW.json.
This is harness/source evidence, not physical acceptance.

Final canonical refresh PASS47.96s, exact decision coverage+15 canon tests:
/home/ubu/.cache/lay/development/td149-final-canonical-8bhhvtib/RESULT.json.
All four canonical metadata files fetched and SHA verified;993/993 functional
files retain final source proof identity. New native/build/install still pending.

## Built candidate and first native result (not installation)

Published source e6f3227c50f13b443fc161e11b52e8ff56feaa33 verified on origin
work branch and both public work/main refs. Current GitHub CI38887748692 and
Security38887748572 both SUCCESS; MSRV and installer jobs succeeded separately.
Build receipt /home/ubu/.cache/lay/development/td149-install88-20261016/BUILD.json,
archivea677c68268c9983a3545fff86fa3b5a4caf4ee9fe10ed10c69eb6c557c88f5a6,
205.9s, same audited Cargo.lock; final same-DB audit0. No runtime source edited
after this build. Local installed runtime remains87.

Firefox full216 completed and retained at
/home/ubu/.cache/lay/development/td149-install88-20261016/native/firefox/RESULT.json:
169 PASS/28 FAIL/19 BLOCKED, NOT_TESTED0. Standard60/60 and panel6/6 PASS;
functional103/28/19. Comparison native/firefox-vs-READV.json: one lost PASS,
functional/normal/short/field4, no gains. Do not discard or rerun this receipt
until green. Original single-character projection а→f stayed а; initial raw
seed а caret1, while old READV/PLAINV/PLAIN successful seeds show аж caret1
(completion after caret). Root cause NOT YET established; completion visibility
is a hypothesis, not authority to weaken any verifier/admission or alter routes.
The existing bounded512KB final trace does not establish the failed callback;
need a separate bounded diagnostic preserving that case immediately. Chrome
full matrix running; optional shortcuts80 and local installation still pending.

Firefox native Double Shift standard420/420 timings: median99.593867ms,
p95112.090050ms,max120.080199ms. Functional DS132/210 (78missing), Space37/50
(13missing), Tab0/10 and selection0/10 retained. Same evdev-write-return→AT-SPI
clock, not kernel/paint or universal laptop latency. New active-VT ledger guard
means harness overhead is not byte-identical to old comparison.

Initial Firefox setup attempt: controller omitted existing owned-keyboard ACL
admission, stopped before input; native-setup-failed-attempt1/firefox/RESULT.json
retains all216 NOT_TESTED and every missing timing. Second setup hit exclusive
seat-receipt filename collision after creating finite owned seat, before GUI.
Verified exact seat PID/starttick/UID/session/VT/RuntimeMax1h and preserved
SEAT-RECOVERY.json before continuing that setup. The standard protocol/actors
and product bytes stayed unchanged; original PERF-keyboard-admit.py prerequisite
now runs before GUI readiness. This is setup recovery, not extra functional PASS.

## Complete current matrix and bounded controls

Current88 default matrix COMPLETE432: Firefox169/28/19, Chrome179/23/14,
total348 PASS/51 FAIL/33 BLOCKED, NOT_TESTED0. Chrome receipt
/home/ubu/.cache/lay/development/td149-install88-20261016/native/chrome/RESULT.json;
no lost Chrome PASS; one prior FAIL became BLOCKED. Firefox one short/phone
loss remains unclassified. Standard DS timings Chrome406/420,14missing,
median82.533846ms,p9599.414483ms. Full raw timing denominators retained.

Optional enabled configurations are separate80: Firefox27 PASS/10 FAIL/3
BLOCKED_PRECONDITION; Chrome33/6/1. Receipts native-shortcuts/{browser}/RESULT.json
in the same private release cache. Do not add these to the432 denominator.
All first6 Firefox fields passed; Firefox rich7/8/10 failed projection and
first-word undo, second-word seeds blocked on native NBSP prefix; iframe input9
rapid pairs reached3/5 then declined pair4. Chrome33successful cases include
19/20 exact Backspace undo actions; rich7/10 projection failure snapshots already
show converted letters plus retainedNBSP2 while current test expectsASCII2
(not proof of a runtime defect). Search2 projections completed but later probe
failed with an unaccepted visible suffix; preserve overallFAIL. Do not relax
raw-output checks or declare every fixtureFAIL a runtime bug. Source/runtime
authority remains unchanged; private modes with options and raw unobserved burst
remain NOT_TESTED.

Bounded old87/current88 diagnostics, no installation or code rollback:
 native-diagnostic/{reference87,candidate88}/firefox/RESULT.json:3/3 each,
 Nanda previews disabled using normal config, live composition stays enabled;
 native-diagnostic/{reference87on,candidate88on}/firefox/RESULT.json:20/20 each,
 normal Nanda, ten base/short pairs with physical Shift gaps0..700ms.
Both versions used the same controlled88 panel, frozen data, native actors and
guards. These46 total controlled positive cells do NOT explain or clear the
original single failure. No speculative production patch is justified yet.

Prepare at most ONE full Firefox confirmation for this unresolved concern:
original byte-identical9 wrappers/canonical actors, same88 binary/config20fe
and frozen data; unique651..659 jobs, separate native-recheck receipts. A passive
read-only watcher captures trace at each failure so the callback cannot be lost
from the bounded final trace. Original default216 FAIL and all controls remain
immutable. No third same-byte full rerun, no test/guard/threshold weakening and
no claim of a root-cause fix from a passing confirmation. Installed87 and all
user config/models/processes remain unchanged.

The single full Firefox confirmation native-recheck/firefox/RESULT.json is
COMPLETE216/9 jobs,169 PASS/28 FAIL/19 BLOCKED, zeroNOT_TESTED. Original short4
passes, but standard/fullscreen/space/field8 loses a priorPASS. Retain both full
receipts unchanged; REJECT_LOST_PASS remains and installation is not permitted
for these bytes. No third same-byte full repeat. Per-cell trace-space-8.jsonl
captures the actual live bridge refusal after completed native replay/lateReset.
TD-150 now owns the causal source repair; its fresh changed-byte gate is required.

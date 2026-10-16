# TD-149 — Minimal rustls security patch before release1.0.88

Status: DONE_SOURCE after plan9/10 and independent code review9/10; final build/native/install pending. Priority:P1 release prerequisite.
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

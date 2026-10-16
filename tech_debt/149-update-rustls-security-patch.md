# TD-149 — Minimal rustls security patch before release1.0.88

Status: PLANNED. Priority:P1 release prerequisite, after TD-147 source commit.
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
3. Audit against freshly fetched RustSec data with unchanged deny-unsound policy;
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

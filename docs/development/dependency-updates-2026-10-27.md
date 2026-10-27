# Sequential dependency updates — 2026-10-27

The user accepted the order #65 → #67 → #64 → #68 → #66. Each update is
integrated with the current public main, verified independently, and merged
only after its required checks pass. Do not batch the five dependency changes.
The full browser failure investigation remains queued after this sequence.

## Baseline and boundaries

- Accepted stable release: 1.0.104; public main before this sequence:
  `85859d971aada5f577683164db60404b3598023b`.
- Working checkout: `/home/ubu/projects/lay-dependency-updates-20261027`,
  branch `codex/dependency-updates-20261027`. The old dirty main is not built.
- Heavy checks run on the dedicated remote worker through the existing
  resource lease and Cargo disk guard. Preserve the local GNOME installation.
- Keep the existing GNOME/KDE dispatch, delivery routes, DecisionCore,
  ContextAdmissionReducer, edit verification and model/learning ownership.
- A green old PR run is not current integration evidence. Record the exact
  dependency versions, source closure, executable hashes and fresh CI.
- Do not turn a missing or failing check into PASS, alter safety assertions,
  refresh semantic expectations, or suppress new warnings to accept an update.
- No release/tag, installation or desktop restart is implied by a dependency
  merge. Test environments and temporary offline compiler fixtures are separate
  from user data and installed models.

| PR | Dependency | State | First known failure or required boundary |
|---|---|---|---|
| 65 | serde_json 1.0.151 → 1.0.154 | Merged1afcb292; source/CI PASS | Current-main integration; parser/writer contracts |
| 67 | rayon 1.10.0 → 1.12.0 | Preflight written | Stale architecture source binding; parallel-runtime contracts |
| 64 | arboard 3.6.1 → 3.7.0 | Queued | Installer preservation failure and test inventory drift; causality unproven |
| 68 | zbus 5.15.0 → 5.16.0 | Queued | D-Bus/IBus integration and transitive zvariant update |
| 66 | sha2 0.10.9 → 0.11.1 | Queued | Removed io::Write and output LowerHex implementations |

Read-only triage receipt:
`/home/ubu/.cache/lay/development/dependabot-pr-triage-20261027/triage-report.json`.
The actual zbus lockfile target is 5.16.0; its branch name mentions 5.19.0.

## PR #65 preflight, before changing production code

The current-main merge is `988cfd279b5d6ff971f92afcaa7a0cdff492fed5`.
Comparison of complete lockfile package maps establishes exactly one changed
package: serde_json 1.0.151 → 1.0.154. No Cargo.toml or Rust source change.
Upstream describes invalid UTF-8 error-position fixes and optional
arbitrary_precision/preserve_order behavior changes. The direct Lay dependency
uses default features; inspect actual resolved features during verification.

Options: retain 1.0.151 (safe baseline, no upstream fixes, 7/10); take only the
reviewed lockfile patch (small reversible closure, 9/10); run an unrestricted
Cargo update (unrelated dependency changes and ambiguous attribution, 3/10).
Select the single-package patch. There is no demonstrated need to rewrite a
JSON consumer or introduce compatibility wrappers.

Consequences to verify:

- Candidate retention, ranking and mutation authority: Rust decision code and
  package schemas are unchanged. JSON decoding/configuration still affects
  observed state; run the complete fixed correctness/package lanes rather
  than infer preservation from the small diff.
- Latency, CPU, RSS and allocation: no new timer, queue, cache or owner.
  Parser cost may change; run the existing enforced performance lane and
  resource/disk checks. No speedup or complete workload equivalence is assumed.
- Cache identity/invalidation: Cargo.lock belongs to the test source closure.
  Old build/test receipts cannot certify the changed dependency. Freeze files
  and modes, bind the new result, and preserve the existing cache owners.
- Package/delta reload and learning: retain serialization/deserialization,
  event/spool, integrity, configuration and inverse-feedback tests. Run the
  existing tiny offline lexical-package byte-parity fixture. Do not train,
  rewrite installed packages, or clear shared learning.
- Concurrency/stale results: no scheduler/ownership change is proposed; existing
  concurrent-state and actual D-Bus tests remain required. A malformed payload
  must still fail closed; an error-position difference is not success.
- IME/daemon compatibility: build all ten installed release binaries under the
  locked research-tools profile; verify stable/MSRV feature profiles, lint,
  Python/Node helpers, public installer tests and current security audit.
- Rollback/failure: keep the exact old lockfile entry and accepted runtime.
  Stop on a required failure, identify its first mechanism and fix only that
  mechanism. Merging cannot imply native browser acceptance.
- Maintenance: no extra API layer or permanent fallback. The patch is removed
  by reverting its dependency commit. Later package formats and online updates
  continue through the same schema/integrity and authority contracts.

Verification denominator: unchanged discovered manifest (3537 Rust
correctness/package tests), 11 enforced performance tests, existing Python/
Node suites, both Rust 1.88 profiles, lint/canon/architecture/audit/security,
ten actual release binaries, and offline package byte parity. Record actual
executed counts and existing skips; do not replace these with planned counts.
Physical browser matrix is NOT_TESTED in this dependency source gate.

## Results

PR65 source gate is complete for closure
`e73e7b03798ef4ef955168d013740a558f581d3c190caf4152bc2d578607bf2d`
(1289 files), head `a5d063b25b50e107dee9323aba27e0f68f947c60`:
3537 Rust correctness/package PASS, 11 performance PASS, 421 Python with10
existing skips and zero failures, Node/public/install/lint/canon/audit/both
MSRV profiles PASS. Ten installed-profile release binaries actually built;
zero compiler warnings; offline package bytes equal baseline. Resolved JSON
features are default/std; optional affected modes are not enabled.

The first development attempt failed before Rust execution because the
updated crate was absent from the offline cache. Fetching locked dependencies
fixed preparation. An additional Python test exposed our incorrect launch
MemoryHigh28G; the existing worker contract requires24G with MemoryMax28G.
Correcting the launch restored421 Python tests. No Lay implementation or
assertion was changed. Original failed receipts remain retained. Successful
Rust proof was reused; unchanged graph/canon proof was explicitly reused.

Fresh PR65 CI passed5/5 on the verified head. Merged at2026-10-27T12:31:05Z
as `1afcb2920ddf294df9463d0a82cf37a55dd5f86e`; merged tree equals the verified
PR tree. Native browser acceptance remains separate and untested. See
`../../tech_debt/evidence/dependency-pr65-2026-10-27.json`.
Each later dependency requires its own preflight and source/CI/merge receipt.

## PR #67 preflight — before applying the Rayon change

Owning dependency: the existing Rayon global/explicit thread pools consumed by
L1/L2 parallel enumeration, joins, diagnostics and compiler/evaluation tools.
The old PR stopped at the architecture source-fingerprint/receipt check;
there is no demonstrated Rayon semantic failure in that receipt.
Upstream 1.12 fixes an invalid Unicode Range<char> boundary; 1.11 requires
Rust 1.80, which is below Lay's Rust 1.88 MSRV. Existing call sites need no
proposed API migration. Actual lockfile diff must still be checked after merge.

Options: retain exact1.10 (7/10); advance the existing exact pin to1.12 (9/10);
replace the pin with an open minor range (5/10, larger future update boundary).
Select only the exact1.12 update; no new parallelism settings or pool owner.

The shared consequence analysis above applies, with these specific risks:

- Parallel collection, join order and scheduling may change. Candidate
  retention, deterministic ties, top-1 surfaces, verifier decisions and online
  feedback must remain governed by their current reducers, not Rayon order.
  Use the whole unchanged correctness/package proof; do not add fixture words.
- Preserve existing pool sizes, deadlines and worker concurrency. Run all
  enforced performance checks; do not claim a latency or RSS improvement from
  a newer version number. Record the actual execution envelope.
- Cargo.toml and Cargo.lock change, so regenerate the real architecture graph,
  binding and PASS-only receipt with the existing wrapper, then verify them.
  Do not edit a fingerprint or relax the stale-receipt guard manually.
- Package/delta bytes, cache keys, loaded models, learning semantics and
  schema versions remain unchanged. Bind new tests/builds to the changed lock
  and compare the existing offline compiler fixture byte-for-byte.
- Stateful races, stale results, panic propagation and fail-closed behavior
  retain the existing contracts. A regression blocks this dependency merge;
  it must not trigger a new runtime fallback, owner or delivery path.
- Both IME and daemon consume this library indirectly. Build the complete ten
  installed binaries, verify both MSRV profiles and fresh CI/security before
  merging. Native per-window acceptance remains separate and untested here.
- No new compatibility abstraction to maintain. Rollback is the exact prior
  manifest/lock entries plus their generated architecture binding. Installed
  runtime and data are preserved throughout dependency source verification.

Expected proof: unchanged complete Rust manifest, existing performance,
Python/Node, lint/canon/architecture/audit, two MSRV profiles, complete installed
release binary set, offline package parity and five fresh PR checks.
Confirm actual counts and exact changed package set; planned counts are not
test results. No additional fit, training, global IBus restart or field input.

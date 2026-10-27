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
| 67 | rayon 1.10.0 → 1.12.0 | Merged3b7873bf; source/CI PASS | Stale architecture source binding; parallel-runtime contracts |
| 64 | arboard 3.6.1 → 3.7.0 | Merged68eb82a6; source/native clipboard/CI PASS | Installer preservation failure and test inventory drift; causality unproven |
| 68 | zbus 5.15.0 → 5.16.0 | Preflight written | D-Bus/IBus integration and transitive zvariant update |
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

PR67 integration changes exactly the Rayon package1.10.0→1.12.0 and its
existing exact manifest pin. The versioned remote self-test entrypoint prepared
the owned mirror; the existing update-architecture-graph.sh wrapper then
refreshed the graph/binding/generated receipt and passed its unchanged guard.
Only graph metadata and the generated receipt were imported, after verifying
all original canonical snapshot hashes/modes. Rust behavior source untouched.
Dependency fetch is complete before the offline full Rust lane.

### PR67 source result and CI metadata correction

Head cb789bc39ce67b0a65f14c028e14229c9a8c3b02, source closure
39975b2b9ed1797d21d19c4546ad589175a5314cf8ad5e30ee6555fd2b1350a8:
3537Rust,11performance,421Python/10existing skips, Node/public/install,
lint/architecture/audit/bothMSRV profiles and all10release binaries PASS.
No release compiler warnings; offline package bytes equal baseline.

First fresh CI passed4/5; canon failed because GitHub's stored PR base was
8687eea0 from18October, although the actual merge ref used main1afcb292.
That incorrect comparison treated already accepted historical decisions as
new PR records. REST PATCH base=main updated the stored base to1afcb292.
No old decision, checker, workflow or runtime source was changed for this fix.
The unchanged checker was independently executed on the remote worker with
exact canonical changed/added path sets: PASS, zero protected changes. A
negative control still rejects a protected edit without a new decision.
The earlier worker --baseHEAD check uses synthetic metadata history and is
not sufficient PR-delta evidence; use the explicit canonical delta receipt.

This documentation/evidence commit leaves the complete runtime source closure
unchanged; reuse the3537/11/build proof and obtain a new CI event with the
correct base. Before later PR pushes, align base metadata through REST and
verify it against current main. The installed gh pr edit command fails on a
retired Projects-classic GraphQL field; no gh/system upgrade was needed.

After retaining/hash-checking all10 prior release binaries and proving no
active executable used the target, disposable Lay package cache was reduced
from11871985664 to3364528128 bytes. Pass guarded workers as script files;
stdin is not forwarded to their background command. No installed files,
models or learning data were removed. Full native browser proof is queued.

PR67 second CI passed5/5 on83b6663cd54b38c4cfe5998d86376e99ec123713,
a metadata-only descendant with exactly the verified runtime source closure.
Merged as3b7873bfa62a013850db12cf98a0716facc17fa7 at2026-10-27T13:55:58Z;
merge tree equals the verified accepted head. Do not repeat67 source gates.

## PR #64 preflight — before applying arboard 3.7

Existing owner: `src/main.rs::read_clipboard` and `write_clipboard`, the only
arboard call sites. No proposed consumer-code change. The old PR installer
failure occurred in shell-only dirty-checkout preservation; its Rust job
refused test-manifest drift. Neither demonstrates an arboard runtime defect.
The current base already contains the corrected installer/inventory work.

Options: keep3.6.1 (7/10); apply only the reviewed3.7 lockfile update (9/10);
also adopt the new wait_once API (4/10, changes blocking behavior without a
demonstrated need). Select the lockfile update. Compare complete package
multisets including duplicate package names and inspect Windows-only removals.
Do not run an unrestricted dependency update.

Consequences and proof boundaries:

- Clipboard construction/get_text/set_text are the existing CLI boundary.
  Candidate generation, ranking, retained lattices, model/delta reload,
  caches, learning/feedback, verification and IME ownership remain in their
  current modules; arboard has no call site there. Run the fixed full proof
  rather than treat unused-code inspection as a behavior certificate.
- Preserve API errors, UTF-8 text, case, newlines and clipboard handover.
  Test old and new retained CLI binaries in a fresh private Xvfb display with
  a real GTK owner/receiver and the installed xclipboard manager. The receiver must read actual clipboard
  text, not accept CLI exit/stdout alone. No shared clipboard/input/focus.
- Upstream's X11 Drop implements CLIPBOARD_MANAGER/SAVE_TARGETS with a100ms
  handover bound. Preserve it; a test needs the manager capability. Missing
  support is not a successful persistence test. The installed xclipboard manager supplies the real handover; the receiver
  never manufactures expected output.
- Keep the four identical UTF-8/case/multiline/emoji cases for both binaries
  and retain every result. This is private X11 CLI integration, not Wayland,
  browser IME, physical input, universal clipboard persistence, or input-latency
  proof. Record observer/process timings with their actual scope.
- New version may change allocation/thread/drop costs. Existing performance
  gates and bounded complete builds remain required; no new pool, cache,
  retry, fallback, timeout, desktop dispatch or delivery path is introduced.
- Cargo.lock changes test/build identity. Preserve source mode/hash checks,
  both MSRV profiles, all ten actual installed-profile binaries, zero new
  warnings, existing tests, package-byte parity and fresh CI/security.
- Windows/macOS release changes are upstream facts, not acceptance of those
  systems by this Linux proof. No supported-platform claim is expanded.
- On failure keep the old lockfile/runtime and distinguish fixture/setup,
  pre-existing behavior and a demonstrated dependency regression. Do not
  modify source guards or clipboard routing just to obtain PASS.
- Rollback is the single lockfile change. No compatibility adapter or new
  long-term runtime owner is added. Retain each dependency-stage binary set
  for later native regression attribution; installed runtime/data remain intact.

Fixture qualification: retained pre-update CLI67 (arboard3.6.1) passes all4
cases with the installed xclipboard manager, including exact multiline UTF-8
and emoji. Both raw clipboard contents and CLI status were checked after
process exit. A custom GTK-manager attempt and standalone private Klipper did
not provide this handover; retain their failed/setup receipts separately.
Standalone Klipper had no CLIPBOARD_MANAGER owner, and restored old/empty
contents. This does not establish a regression in arboard3.7 (not applied),
or behavior of a full Plasma session. Do not change CLI/IME delivery to fix a
fixture capability mismatch. The candidate must use the same frozen positive
xclipboard actor and all4 exact cases, with baseline control retained.

### PR64 accepted result

Merged68eb82a6ba1cdecb4582732eae16997e43c9940e at2026-10-27T14:49:01Z.
Frozen head860bc1fe17e8bf59f87577bdb8c379345531b856, source closure
9b682ccb3e03ebfb8c5d8f1a8b4d63ab23e492f9b4bb5067748ff63eaca41abc.
3537Rust,11performance,421Python/10existing skips,17followup gates,
all10actual release binaries,zero warnings,both MSRV profiles,package parity,
fresh CI5/5 PASS. Actual canonical delta and protected-edit negative control
PASS; no guard changes. Merge tree equals tested head. Native clipboard8/8
PASS: all4 UTF-8/case/multiline/emoji cases on each retained old/new CLI,
identical frozen xclipboard actor and real receiver after process exit.
Process times6.2–7.9ms,n4/version include5ms observer polling; these are not
IME/keyboard response times. Browser/Wayland/physical matrix NOT_TESTED.
Source run run-d9qbu0l3; followup dependency-pr64-gates-v1; exact evidence in
tech_debt/evidence/dependency-pr64-2026-10-27.json. No runtime installation.

## PR #68 preflight — before applying zbus 5.16

Current public main is68eb82a6ba1cdecb4582732eae16997e43c9940e.
Existing owners are the IBus connection/factory and bridge modules, daemon
D-Bus clients, tray D-Bus interfaces and existing desktop dispatch. The PR is
lockfile-only: zbus5.15→5.16 with its macros/zvariant closure. Inspect the
complete package multiset rather than trusting the misleading branch label
(zbus-5.19.0). Keep vendored zbus_names=4.3.2 and its compatibility feature.
No first broken production transition has been demonstrated for this upgrade;
the existing old-base CI successes do not establish current compatibility.

Options: hold5.15 (7/10); reviewed bounded lockfile upgrade with real bus and
full-source proof (9/10); unrestricted update or transport redesign (3/10).
Select the bounded upgrade. Do not add compatibility bypasses speculatively.

Consequences and checks:

- C01: freeze actual source files/modes, lockfile, package closure, compiler
  artifacts and candidate hashes. Worker synthetic Git metadata is not the
  canonical source commit. Correct PR base metadata before pushing; compare
  the canonical changed/added sets through the unchanged canon checker.
- C02/C03: ranking, candidate retention, mutation decisions and word/context
  ownership remain with their existing reducers. Transport can change arrival
  order; the complete fixed proof must preserve stale-result refusal and exact
  focus/epoch/transaction identity. No second authority or connection pool.
- C04/C05: retain one command per gesture and one typed delivery execution.
  No changed wire method, fallback, client-specific path, retry, timeout or
  desktop detection. Keep existing80ms source/40ms selected-text boundaries.
- Real-zbus peer-to-peer tests cover factory callback rendezvous, marker and
  ping signatures, bounded bursts, ordered readiness and observer cancellation;
  ibus_wire_compat and daemon/tray tests remain in the fixed full3537 set.
  All10 actual release binaries and both MSRV profiles must compile cleanly.
- zvariant's borrowed/owned values, signatures and error propagation may
  change; inspect transitive versions, default features and vendor pin. Do not
  reinterpret a failed D-Bus call as valid observation or grant edit authority.
- Concurrency/allocation/startup/latency can change despite no Rust diff.
  Run existing enforced11performance cases and bounded resource/disk checks.
  New library version is not a speedup claim. No new cache, thread, scheduler,
  queue or lifetime owner is introduced in Lay.
- Serialization, model/delta reload, feedback and integrity remain governed
  by current modules. Retain package/learning tests and offline package-byte
  parity. No shared model/learning mutation, training or cleanup.
- Additional integration uses the existing immutable V3 actual-input-context
  driver in a fresh private bus/PID/network/mount namespace. Copy/hash the9
  required dependency roles, preserve the active receipt bytes and mount path,
  bind the actual retained engine and service. No global IBus or GUI restart.
- Qualified baseline64 fixture's immediate lifecycle lane FAILED before its
  first completed case: driver.py806 sees visible/output changed after refocus.
  Receipt: remoteCROOT/dependency-ibus-baseline64-lifecycle-v1/receipt.json.
  This precedes the zbus upgrade and is not blamed on5.16. Preserve failure;
  never change driver assertions, readiness, deadlines or scenario bodies.
- Test the existing manual-toggle3 and terminal-delivery4 lanes with the same
  admitted post-exact-ready schedule on both retained engines. This is private
  actual IBus/GNU Readline transport coverage, not cold restoration quality,
  desktop/browser acceptance, physical keyboard or immediate-Space latency.
  Retain unavailable/failing cases in their original denominators. Any new
  regression blocks merge. The baseline lifecycle refusal remains separate
  open evidence, not silently removed or relabeled PASS.
- All source, package, installer, lints, graph, audit and fresh CI/security gates
  remain required. Rollback is the reviewed lockfile closure; no installed
  runtime or data is changed by the dependency merge.

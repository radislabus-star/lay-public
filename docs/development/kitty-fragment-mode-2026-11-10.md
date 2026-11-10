# Kitty letter-fragment mode completion, 2026-11-10

## Preflight and observed failure

Source baseline: `a73529c0aa30891993691525b63118e78097b319`, initially clean,
checkout `lay-dependency-updates-20261027`. This is the user's continuing,
explicitly authorized fragment integration repair; one implementation owner, without Harness2. Independent read-only review is
required by AGENTS.md and the user's review workflow.
The user authorized necessary fixes and tests without repeated approval.
Primary binary installation remains through the user's updater.

At 13:27 UTC the installed CLI/daemon were 1.0.125 but the running IME was
the deleted 1.0.123 executable, SHA256
`2a4c636a3be959ce096754909599a202146b7711157c2563e6980f5c8909d429`.
Both autoflip flags were enabled. The installed full-restart controller was
invoked only after `OwnsActiveText=false`; it completed at 13:31:16 UTC in
11.839 seconds. The loaded IME now matches the installed 1.0.125 bytes,
SHA256 `9cf89abafc22c47aa16c4f064308f360bb68dab77d178cf97d7493a6463fd905`.
Configuration and selected RU source were preserved. No binaries were installed.

An eight-second native test in a new window of the user's existing Wayland
Kitty process proved `ghbd -> прив` before Space. Subsequent physical T/N
produced `привtn`. A second observation added independent mode readbacks:
internal IME mode is RU while both GNOME selected source and IBus engine remain
`lay-ime-us`. This is a C09 failure after a successful text operation.

Evidence directory:
`/home/ubu/.cache/lay/development/kitty-fragment-runtime-20261110/`.
`before.json`, `after.json`, `restart.log`, `native-check-2/RESULT.json`,
`native-check-3-mode-observation/RESULT.json`. Both native probes restored
source, engine, focus and unchanged configuration. The first diagnostic launch
failed to parse a GVariant boolean before any window/input action; retained as
a diagnostic helper error, not a product failure.

## Dependency map and exact scope

```
paired letter: ibus_interface.rs -> fragment_layout/reply.rs
  -> detector/request: fragment_layout.rs
  -> existing request_word_layout_flip: shift.rs
  -> existing committed-tail writer: committed_tail.rs -> state.rs
  -> caller's complete_fragment_word_mode: fragment_layout.rs
  -> sync_layout_for_fragment_word_mode: layout_sync.rs
       KDE -> existing owned-preedit group switch
       GNOME managed input -> existing native InputMode publication
       GNOME native terminal -> currently no desktop operation (first failure)
  -> next native terminal key: compositor's selected layout -> Kitty
```

Historical first-candidate scope (rejected below): only the new fragment mode entry in
`src/bin/lay_ibus_engine/layout_sync.rs` and its existing
`layout_sync/tests.rs`. For a proven native terminal on GNOME, reuse the
existing checked `LayoutSwitchRequest::switch_stack`. Do not alter the old
layout functions, GNOME/KDE transports, text writer, Space/Tab/Double Shift,
ownership/admission/verifier, rendering, installer, extension or config.
No new backend, classifier, word owner or delivery fallback.

The earlier rejected general GNOME source-switch experiment remains rejected.
This hypothesis is restricted by the existing `uses_native_terminal_input()`
evidence; managed GTK/browser preedit continues to retain its engine. A
successful source test does not prove terminal continuation or manual return.

## Verification and stop conditions

First reproduce the missing desktop operation in tests, both directions.
Negative controls retain GNOME managed/atomic behavior and KDE group-only
behavior; disabled/autoswitch and missing current authority remain refusals.
Assert no extra text, word-epoch or buffer mutation from mode completion.
Run the remote explicit IME gate before a candidate native probe. The native
probe must see the prefix flip before Space, correct next physical letters,
the second word, and manual returns; preserve all failures. Candidate bytes
must be identified and the user's installed runtime restored afterward.
Do not publish or install an unqualified candidate. After two attempts without
new distinguishing evidence change observation method, never retry for green.

Preflight verdict: runtime123/125 mismatch repaired; native continuation FAIL.
Production source was unchanged at preflight. Ordinary KDE matrix remains dated
329 PASS / 59 FAIL / 52 BLOCKED of 440. No fresh full-browser acceptance.
Upstream read-only triage: open PRs 78 (ureq), 77 (libc), 76 (x11rb); no action.

## First candidate rejected; second caller-only hypothesis

The direct-stack candidate passed 1183/1183 remote IME tests. Source closure
`8511fb11312de9b3e6bfc18e81800ac3f506cd4efa4e7517d17e8dab15c5c9bb`,
binary `f133278a6e431aa9e8a74934faff7606e148bcdc7345e1f7e16303a3c2cd9e21`.
Receipt: local `run-r29ikp8i/RESULT.json`, remote `run-zGQ9Jm/RESULT.json`.
Native `candidate-short4/RESULT.json` shows correct `привет hello` but IBus
settles at `xkb:us::eng` and the bridge refuses admission. This is FAIL, not
acceptance. Source patch retained in `rejected-direct-mode.patch`; both
layout_sync source/test files restored exactly to baseline. The installed
125 runtime and its models/config/source were restored after every probe.

Diagnostic setup failures were kept separately: candidate filename initially
prevented the existing exact-name process discovery; source activation could
temporarily remove Kitty focus; intermediate metadata reads can return a
context transition refusal. The final settled-mode assertion remains strict.

The next hypothesis changes only the new caller in `committed_tail.rs` and
adds its regression in `fragment_layout.rs`. Its native GNOME request keeps
the writer's default layout owner. The existing state.rs operation already
schedules `sync_layout_after_committed_text` on the existing worker using
committed-layout intent. No changes to that worker, layout_sync, transports,
reducer, verifier, or ordinary callers. Managed GNOME/KDE still use the
previous caller-owned completion. Owning ADR:
`2026-11-10-kitty-fragment-existing-mode-owner.json`.

Remote resource refusals were real: free swap was 41 MiB under the existing
2 GiB reserve; then global memory PSI exceeded the unchanged 2% guard.
Added one temporary 2 GiB swap file, without fstab change. The latter pressure
came from the unrelated audio-summary service's 2.5/3 GiB cgroup despite
20 GiB available RAM. Under the user's dedicated-host resource authorization,
raised only its runtime high/max to 5/6 GiB, preserving its PID and activity.
PSI fell to 0; the unmodified Lay guard admitted the build. Exact original
limits and swap path are in `/var/tmp/lay-kitty-fragment-build-*-receipt.json`
on the worker and must be restored/removed after this work when safe.

## Second-candidate source evidence

The new regression directly invokes the existing committed-tail operation with
an actual FragmentLayoutRequest and terminal purpose/cell-width proof, matching
the neighboring operation tests. The outer letter/current-field readiness is
checked separately in the native actor. No production guards were changed to
satisfy the fixture. One initial command used an unqualified exact filter and
selected zero tests: retained, not counted as proof. The corrected fully
qualified regression ran one test PASS (both conversion directions).

The remote focused gate subsequently ran 1181/1181 PASS, 185.3 seconds:
local `/home/ubu/.cache/lay/development/run-ovbggku8/RESULT.json`,
remote `/home/e/projects/lay-development-runner/run-WyWWmX/RESULT.json`.
This is source evidence only. Candidate release binary/native result pending.

## Second-candidate native result; caller output settlement

Candidate2 SHA256 fd5fcc05d91fe08f447dcc15637e9d8424d4693816454ce4b5f55c95ca94a6db.
`candidate2-short/RESULT.json`: PASS, both words before Space and subsequent
physical keys correct, selected GNOME/IBus Lay engines agree. Installed125,
config and model sources restored. The new unit regression also fails on the
unchanged baseline (one selected, one failed); candidate source restored.

`candidate2-manual/RESULT.json`: FAIL. Each odd manual return produces привtn,
not ghbdtn. The trace's first `source_seal_precondition` records
`tail_epoch_matches=false`; source engine epoch13 differs from the reducer.
The new fragment caller runs after the paired key was settled, publishes
replacement epochs, and has no context_bridge_token to settle that output.
The target activation therefore begins with UnknownStart and loses the prefix.
This is a caller-accounting defect, not proof that source sealing should weaken.

Next refinement stays in committed_tail.rs and fragment_layout.rs tests.
Reuse begin_context_bridge_output, as kde/native_document.rs already does for
actual deferred results after their original callback. Require the current
token's unchanged revalidate_bridge before writing; no waits, new owner,
reducer rule or transport. Its RAII scope settles the existing writer's epoch
and revokes on error. Managed GNOME/KDE and manual callers stay unchanged.

The required lay-diagnose report command was attempted on the retained native
receipt and rejected it as unsupported: the analyzer accepts only native RFB
browser actors. This is an analyzer scope limitation, not passing analysis;
the terminal trace and PTY receipt above supply the direct causal evidence.

## Candidate3 native acceptance and release preparation

Candidate3 SHA256 afce5fed6a7b41071726cb72bf92548d8d746f274d7c15b56df55e432fa9ea7b,
functional source closure3146d01b6e49d989670d44e6f4d191620fba524c67d789fc8fee5a27c1984017.
Focused IME1182/1182 PASS,202.2s, localrun-cxw3cbuc / remoterun-qtwGlI.
Controlled removal of output settlement reproduces the new epoch assertion
failure; existing caller regression also detects the original baseline.

Actual primary Wayland Kitty: four scenarios/19 ordered target assertions PASS:
manual eight returns plus next letter, fast35ms typing before Space,
two-word ENG->RU and RU->ENG continuation, and ordinary Space with fragmentsOFF.
Each probe independently restored installed125 bytes, selected source/engine,
real config and model sources. Exact receipts, source hashes and timing scope
are in candidate3-native-acceptance.json under the evidence root.
No general-browser or other-terminal acceptance is claimed.

Prepare126 version metadata, discovered test manifest and unchanged empty-failure
ledger binding, architecture graph/receipt, all mandatory source/build gates.
No permanent primary install; the user obtains the release through the updater.
A separate read-only cold review is requested by the existing review agreement;
implementation remains with the single owner. Publication pending required gates.

Independent read-only review returned9/10 with no required fixes. It checked
current-token admission, output-scope completion/error revocation, deferred layout
ownership and both new regressions. No independent native acceptance was claimed.
Reviewed source SHAs: committed_tail.rs
6c291fd59f56744d5491962f938d590aad0d00d88a9a7b0395574d65775bd61c;
fragment_layout.rs 589f7979f1c90f51698d8db39f25d33912328bb9dd1020c9e541f6305c926123.

The first126 integration build stopped at the12GiB Cargo cache budget before
test execution. Removed2.95GiB of unused pre-Nov10 debug Lay artifacts; no
release or live executable removed. Cleanup receipt:
/home/e/projects/lay-development-runner/kitty-fragment-debug-cache-cleanup-20261110.json.

After cleanup, the126 focused integration gate passed22/22 (Space input gate
and text mutation monopoly),28.3s: localrun-axp7su93, remoterun-6unSxF.
Discovered manifest adds exactly the two new tests:3708 identities,3652
correctness/package and11performance,45ignored. The failure ledger stays empty.
The required AST refresh and architecture check PASS:28914nodes/73486links,
all11 architectural checks PASS, binding792Rust sources. No graph edge repair,
guard change or reuse of an unrefreshed graph was needed for this candidate.
Remote log: /home/e/projects/lay-development-runner/kitty126-architecture.log.

## Full126 release gate

PASS,1134.44s, remote kitty126-release-gate-20261110/RESULT.json:
3652correctness/package,11performance,475Python PASS/12environment SKIP,
all ten installable126 binaries, installer contracts, lints, architecture,
CLI/ngram checks and actual Rust1.88.0 all-targets in both feature modes.
Full frozen snapshot2323files checked before/after. Local/remote compiled
closure1338files SHA ac4f49e121237b19096d48bc465156d719c5a275c08f6641cc0e780e352309c3.
Native candidate and release have byte-identical Rust sources/dependencies;
the version-only Cargo changes and generated architecture JSON are recorded.
The release gate builds real126 executables separately from native acceptance.
Detailed portable record: evidence/release-1.0.126-verification.json.
Publication still requires GitHub CI and final updater discovery.

## Publication completed

Stable v1.0.126 published2026-11-10T16:11:57Z, release440595717.
Code cd61dadc689b7819b34c23163a7759fd7fe15ab0; annotated tag
6175ad9ccccff6b6128183aa2a6cca78616fdf7b matches origin/public and peels to that code.
Public CI4/4 PASS (source run42847032868, security run42847033199).
Actual installed updater on both hosts:125->126, exit0/applyfalse;
primary0.668s and KDE0.696s. No permanent binary install for this repair.
Publication details: evidence/release-1.0.126-publication.json.
Our temporary2GiB swap was removed after builds. The unrelated system service's
5GiB/6GiB runtime memory limits remain: its current allocation exceeds its old
3GiB maximum, so forcing that limit back would endanger the active process.
Private cleanup receipt: kitty-fragment-runtime-20261110/resource-cleanup.json.

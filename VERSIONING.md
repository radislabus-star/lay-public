# Versioning

Before `1.0.0`, `lay` used the patch version as a public release counter:

```text
0.x.<public-release-number>
```

Git does not store reliable "push event" history in the repository, so the
project treats each pushed public release commit as one version step. The bump
script increments the current `Cargo.toml` patch number; it does not derive the
number from `git rev-list`.

From `1.0.0`, the package follows semantic versioning. GNOME Shell's required
integer metadata version remains monotonic and is encoded as
`major * 1,000,000 + minor * 1,000 + patch` for stable releases.

Current source version:

- `1.0.101`

1.0.97 preserves verified KDE closed-word replay after an ordinary new-word
letter is erased, including one owned Space/NBSP presentation transition.
Full source/build proofs and the limited native Firefox result are recorded in
[release notes](docs/release-1.0.97.md); remaining client failures stay visible.

1.0.96 repairs graphical-session startup and upgrade migration, GNOME 51 API
compatibility, installer diagnostic output and locked security dependencies.
The release gate must build every installed binary and retain exact results.
See [release notes](docs/release-1.0.96.md).

1.0.94 adds keyboardless VNC/X11 input through the existing daemon gesture
pipeline, repairs KDE startup and controls, and restores owned hint/Tab input
for stock clients without surrounding snapshots. Release evidence and supported
scope are recorded in [release notes](docs/release-1.0.94.md).

1.0.92 makes unaccepted IME hints muted gray in IBus-aware renderers and
translucent in GNOME's system-colored auxiliary panel. It also aligns the
package and GNOME panel version metadata. See [release notes](docs/release-1.0.92.md).

1.0.91 published the accepted KDE input fixes and the closed 432-case native
matrix: 391 PASS / 37 FAIL / 4 BLOCKED. Those results remain evidence for that
candidate, not fresh acceptance of the presentation change.

1.0.90 restores the update action and groups GNOME/KDE daily controls.
See [release notes](docs/release-1.0.90.md).

1.0.90 validation: 3,212/3,212 remote source tests, 10/10 updater tests and
7/7 Qt offscreen checks passed. Review: 9/10. Release, security and architecture
gates passed. After installation, CLI and panel report 1.0.90; loaded daemon
and IME hashes match the build. The 23 protected files, global IBus, L1.1/L3,
configuration, input sources and mode are preserved. Installed updater and KDE
tray links point to the accepted checkout. Receipts:
`/home/ubu/.cache/lay/development/menu90-20261018/installation/INSTALL.json`
and `POST-VERIFY.json` alongside it. New native input acceptance and real Plasma
desktop acceptance remain NOT_TESTED; the historical 432-case matrix below is retained.

1.0.89: exact acknowledged Space-series replay (TD154/155), source3211/3211,
review9/10, first-attempt source-main CI/Security. New whole432=349P50F33B,
0lost old defaultPASS; optional80=64P12F4preconditions,9gains/3lostFFrapidPASS.
Exact three native-tested core binaries installed; display-only APP_VERSION
alignment separately proves992 exact inputs plus one literal and preserves all
original receipts. Current install receipt:
`/home/ubu/.cache/lay/development/td155-install89-20261017/install-aligned/INSTALL.json`.
CLI/panel89 and loaded hashes verified;23protected files, globalIBus/L1.1/L3,
models/config/sources/mode preserved. [Release details](docs/release-1.0.89.md).

Historical 1.0.88 source includes backed present-tense verb forms (TD-148), opt-in IME
Backspace undo and Double Space (TD-147), session-scoped installer input access
(TD-146), rustls0.23.46 (TD-149), completed native replay proof across Reset
(TD-150), prepared Space proof retention (TD-151), and bounded CI failure
evidence (TD-152) and an explicit test-only readout readiness prerequisite
(TD-153). The final matched source/native/install receipts are linked
from [the public issue register](docs/development/public-issues-2026-10-16.md).
CLI/daemon/IME and panel1.0.88 were installed and loaded with matching hashes.
Its functional-source CI/Security first attempts passed; whole432+80 receipts and six-payload
byte equivalence are recorded. Models/settings/sources/mode and globalIBus/L1.1/L3
are preserved. See the owning TD153 receipt record.

1.0.87: source patch for [late device access, GitHub #49](tech_debt/144-recover-late-device-access.md).
The existing daemon main thread retries missing uinput and input listeners once
per second. Existing output devices and live listeners are retained; recovered
availability does not replay failed edits. Listener lifetime changes invalidate
old word/gesture intent through the existing field epoch. IME delivery routes,
decision authority and models retain their contracts. Source publication was
followed by the user-directed installation of CLI/daemon/IME and panel 1.0.87.
Loaded binary identities and device readiness are verified in
`/home/ubu/.cache/lay/development/td144-install87-20261016/INSTALL.json`.
Physical-client acceptance and native latency remain NOT TESTED / NOT MEASURED.
No new GitHub Release was created.

Описание предыдущего релиза и его известных ограничений:
[Lay 1.0.86](docs/release-1.0.86.md).


1.0.86: TD-143 native-terminal restoration learning. Successful authorized
terminal correction emission retains a temporary origin; admitted native
Backspace/retyping and exact restoration produce existing UserCorrection
preference evidence. First-word learning waits for ordinary Space settlement
when the existing reducer temporarily has UnknownStart. Word-edit authority,
delivery routes, models, SSI and deferred Atomic behavior retain their contracts.
Final source/build/installation evidence is owned by:
`/home/ubu/.cache/lay/development/td143-release86-20261015/INSTALL.json`.
Installed and loaded core identities match the868/868 tested source and verified build.
GlobalIBus,L1.1,L3,config,sources,mode and24protected files preserved; zero test keys.
Review7→8/10;
physical Kitty acceptance and native latency remain NOT TESTED / NOT MEASURED.
[Native follow-up](tech_debt/143-native-terminal-restoration-learning.md).


1.0.85: user-requested installation of TD-142 personal restoration learning,
source `a09fb3a5`. The existing three-core-binary transaction builds CLI, daemon
and IME remotely and synchronizes the two extension version files. All740Rust
files and dependency content are unchanged from the final2966/2966 source gate;
only package/extension version metadata advances. Models, input ownership and
IME delivery routes retain their existing contracts. Other helper binaries and
L1.1/L3 services remain as installed. This is not full release/client acceptance.
Current installed/loaded state and rollback evidence are owned by:
`/home/ubu/.cache/lay/development/td142-release85-20261015/INSTALL.json`.
[Installation decision](docs/architecture/decisions/2026-10-15-user-directed-restoration85-install.json).
New native behavior and latency remain NOT TESTED / NOT MEASURED.

1.0.84: installed verified CLI/daemon/IME and synchronized GNOME extension metadata.
ObservedIbusEngine replaces CLI spawning only in exact engine verification;
accepted output pacing, delivery routes, models and input authority are unchanged.
Source1134PASS and full matched READV432 preserve all349 previousPASS;
one Chrome search/undo case advancesBLOCKED→FAIL (349PASS51FAIL32BLOCKED).
Native-tested83 logic is reused for metadata-only84 after exact parity of739Rust
files, dependency lock and extension code. Local84 functional typing was not run.
Installation verified CLI/tray84, /proc binarySHA parity,24protected files,
unchanged globalIBus/L1.1/L3 identities, config, sources and mode. Receipt:
`/home/ubu/.cache/lay/development/remote-firefox-setup-j2he3qf6/install83-20261015T033053Z/INSTALL.json`
(the directory keeps a legacy name; the receipt records84).
[Performance and limits](docs/development/double-shift-latency-2026-10-15.md),
[release decision](docs/architecture/decisions/2026-10-15-observed-readback-release84-metadata-only.json).

1.0.83: installs the frozen WJ CLI/daemon/IME core on the user workstation,
with synchronized extension version metadata. Source848PASS and native88
32PASS/49FAIL/7BLOCKED are retained evidence for previous WJ1.0.82 logic;
new-byte physical acceptance is left to the user by explicit instruction.
No model, delivery-route or word-authority change is introduced by the bump.
See [installation scope](docs/architecture/decisions/2026-10-14-user-directed-version83-install.json).


1.0.80: installed Firefox Reset/native replay and native Space-boundary repair.
Preserves the admitted replay seed and exact successor lifecycle through observed
Reset/callback retirement. Double Shift detector, ManualToggleV3, SafetyGate and
execution routes retain their existing owners. Full source gate PASS; both
canonical runs 3010/3010. Temporary same-byte IME80/daemon79 controls pass Space,
eight Double Shift with word/mode/icon changes, and actual Backspace→Tab in three
Firefox field types. Wide matrix remains 87/93; unresolved failures are recorded.
Exact IME80/daemon80/CLI80 and extension version80 were installed on 2026-10-06;
global IBus, L3/L11, input sources, configuration and35 protected paths preserved.
The user directed installation before real-keyboard confirmation; that check is
NOT_PERFORMED, not PASS. Combined installed80 client acceptance and every Firefox
site are not established. Publication reuses the existing bytes and evidence.
See [Firefox evidence](docs/architecture/firefox-input-stability-2026-10-05.md).

1.0.79: installed contextual-completion and terminal-command-preference release.
Repairs current-prefix evidence, preserves confirmed L4 ordering and refreshes
shared usage memory on the existing worker. Full source gate: 2981/2981; fresh
Kitty command acceptance and 46 ordinary client controls pass. Physical scope is
partial: repeated Double Shift still fails in Firefox textarea; observer gaps
remain. Inherits experimental 1.0.78 agreement/native input fixes.
The positive-only last1/tail2 corpus pilot is not included: validation has no
attestation gains. Do not advertise full-sentence semantic understanding.
See [release evidence](docs/architecture/contextual-completion-release-2026-10-05.md).

1.0.78: experimental native agreement ordering after existing candidate admissions.
Release/install preparation is authorized by the user; live installation and
publication are pending. Offline research percentages do not certify this native
provider, scorer or current-prefix context. Full quality gates remain unmet.
See [deployment scope](docs/architecture/ranker-release-preflight-2026-10-04.md).

1.0.77: publishes the user-accepted short-word RU/EN auto-flip after Space.
The exact installed IME accepted by the user remains SHA-256
`58dcaacc5c1d403f095fb94aa9d88080a566361a3d4198ee3eb319cbc45accb5`.
The source gate passed 2,940 selected tests; physical acceptance was 360/360
transitions in owned local Firefox, Chrome, isolated Tor, GTK, Qt and Kitty
fields. This source publication does not rebuild or reinstall the running
binary. User WhatsApp, GitHub issue, WPS and existing Tor fields have separate
untested or open behavior. See the [accepted contract](docs/architecture/accepted-space-autoflip-2026-09-28.md).

1.0.76: repairs IBus context-admission bootstrap when a full restart starts
Lay before IBus exposes GlobalEngine. The observer begins without text authority
and waits for the verified engine-change and focus receipt. Kitty terminal
input is restored when IBus omits its content type after restart. Tor's
no-SurroundingText route and the WhatsApp second-key suggestion regression
remain open. See
[release scope](docs/release-1.0.76-ibus-startup-2026-09-25.md).

1.0.75: publishes the user-accepted Firefox repeated Double Shift and
first-word-after-Space repair from source checkpoint `51420a4`. The exact
installed IME accepted by the user has SHA-256
`4bbe07233808d1d14ecd072b87c052d760c22fe17bcac5f63d8adf9e8d1c7328`.
The release changes source version metadata; it does not replace the installed
binary. Tor's no-SurroundingText route remains open. See
[release scope](docs/release-1.0.75-firefox-cyclic-2026-09-25.md).

1.0.74: `PUBLISHED_VERIFIED` (2026-09-21). This patch release publishes the
accepted TD-125 browser autocorrection-boundary repair. Public `main` contains
the release commit `ecd5af2a`; annotated tag `v1.0.74` and the GitHub Release
point to that commit. The post-publication documentation commit records the
public readback without moving the immutable release tag.

Historical 1.0.73: `PUBLISHED_VERIFIED` (2026-09-20). This release contains the
#47 pure-uinput Double Shift and independent Auto Switch fixes, the `-ять`
present-form protection, safe IME hot restart and exact Firefox round trips.
Its final transaction and publication readback are recorded in
[release evidence](docs/release-1.0.73-finalization-2026-09-20.md).

1.0.72: the current Firefox repair transaction is recorded in the
[execution receipt](tech_debt/evidence/td121-release-1.0.72-execution-2026-09-14.md).
It binds the final release gates, native tests, installed and loaded artifacts,
launcher activation and publication. R12 development acceptance is 563/563
focused IME tests, two independent source reviews and 4/4 exact native scenarios.
Historical C20 acceptance does not establish acceptance of the later repairs.
A human hardware-keyboard check remains unobserved; the user authorized
independent verification, installation and push while away.

Historical 1.0.71: `INSTALLED_VERIFIED_PHYSICAL_PENDING` (2026-09-11 local date).
This is the installed release transaction for the accepted terminal
word-boundary replacement, clean repeated-word handling, candidate comparison,
and Backspace/pending-learning source repairs. Fresh
1.0.71 full identity `7585f7df394fa6d06c15181923d68b553f07f0bf788f3184a4b2baf5da7c36e7`
binds 699 source files and all 10 release binaries; candidate IME SHA-256 is
`2ee1479cfc84c40ce3e8673e573d2dbfc42226f0e4dbdeb7cc310f6f1a63fcc5`.
Changed/full gates passed `2756/2756` each; native controls passed `17/17`;
fixed89 stayed exact-output identical across three profiles; diagnostic18 is
`14/18` with `7/11` dirty restored and `7/7` clean preserved.

Installed runtime receipt:
`~/.cache/lay/development/release-1.0.71-20260911-hiu22jcy/installation-1.0.71-attempt2.json`.
Post-install runtime receipt:
`~/.cache/lay/development/release-1.0.71-20260911-hiu22jcy/post-install-runtime-1.0.71.json`.
The global IBus PID 4715, LayRU config, input sources, and eight immutable
model/data dependency payloads were preserved. The L1.1 executable dependency
changed from the accepted 1.0.70 service hash to
`db825d2f244282392fe507ebeee335ef3e66ee33fc35f6e701a688abc7845034`. Input journals were cut over at
`2026-09-11T09:54:01.913546Z` by deleting exactly 10 old files without archive;
new logs belong to 1.0.71 after that reset. Physical keyboard acceptance remains separate. Final document graph completion
is established only by
`~/.cache/lay/development/release-1.0.71-20260911-hiu22jcy/final-document-graph/fetch-receipt.json`
with `PASS`; publication refs are established only by
`~/.cache/lay/development/release-1.0.71-20260911-hiu22jcy/publication.json`.
Evidence: [release 1.0.71](docs/release-1.0.71-preflight-2026-09-11.md).

Historical 1.0.70: `INSTALLED_VERIFIED_PHYSICAL_PENDING` (2026-09-10 local date).
The revised GitHub-issues build was installed: daemon `133a1f79e57b`; the other
nine binary hashes match the first 1.0.70. Changed/full each pass 2726 tests;
the public installer passes eight regressions. Current installation receipt:
`~/.cache/lay/development/run-j4e7w_0n/installation-1.0.70.json`.
Public delivery targeted `public/main` and `v1.0.70`; its exact remote refs were
recorded in `~/.cache/lay/development/github-issues-70-3ps08xs0/publication.json`.
The Ubuntu issue outcomes and physical keyboard remain separate unobserved
claims. [Scope and evidence](docs/public-issues-42-44-release-1.0.70.md).

First 1.0.70 installation, retained as historical IME evidence:
Observed-boundary Backspace/retype admission: native baseline 3/4 -> candidate
4/4, mandatory tests 2723/2723, three fixed89 profiles with unchanged outputs,
native 13 and first-word GUI 7/7 PASS. Final review 9/10, H0/M0/L0.
Ten installed binaries and four loaded executable hashes match; CLI and loaded
extension report 1.0.70. Global IBus PID 4715 and model/config/input sources
were preserved. Receipt: `~/.cache/lay/development/run-p4d_z81t/installation-1.0.70.json`.
Physical acceptance and publication were pending at that historical handoff. General TD-123 quality and
the compound layout-plus-two-letter error remain OPEN. Do not rebuild or
reinstall merely to publish the verified bytes.

Historical publication marker before the 1.0.71 transaction: `1.0.70`; verify exact refs in `~/.cache/lay/development/github-issues-70-3ps08xs0/publication.json`.

1.0.69 local runtime: `DELIVERY_ACCEPTED` for hint/Tab scope (2026-09-09).
First-word suffix display and explicit Tab append pass 7/7 final-byte native
cases; mandatory correctness/package tests pass 2717/2717. Three fixed89
profiles retain their metrics and false-output counts; 13 native controls pass.
All ten binaries and four loaded processes match the accepted release hashes;
CLI and loaded extension report 1.0.69. Global IBus PID 4715, model inputs,
configuration and input sources were preserved. Receipt:
`~/.cache/lay/development/run-btt4ilfz/installation-1.0.69.json`.
After the retained intermittent-hint report, the user accepted the result
with "Push отлично" and explicitly requested publication. Client identity was
not specified. A new automatic-layout-correction report and general TD-123
quality remain OPEN. The exact commit and remote ref are recorded separately
in `~/.cache/lay/development/run-btt4ilfz/publication-1.0.69/`. Do not rebuild
to publish these verified bytes.

1.0.68 local runtime: `DELIVERY_ACCEPTED` (2026-09-09 07:19 UTC).
Admitted IME suggestions no longer wait for optional cursor metadata. All ten
release binaries and four loaded processes have verified hashes; source, CLI
and loaded extension show 1.0.68. Global IBus PID4715, model packages, config
and input sources were preserved. Current receipt:
`~/.cache/lay/development/run-hvrxwcxk/installation-1.0.68.json`.
The user accepted physical typing with "работает !" and authorized push.
Publication branch: `origin/codex/cleanup-20260908`; verify its remote ref against
the publication receipt in `~/.cache/lay/development/run-hvrxwcxk/publication-1.0.68/`.
This delivery acceptance does not close general TD-123 quality. Do not
bump/rebuild again merely to publish these already verified bytes.

TD-123 1.0.67 runtime history: `INSTALLED_VERIFIED_PHYSICAL_PENDING` (2026-09-09).
All ten verified release binaries are installed; source, CLI, loaded extension
and four live process executable hashes agree. Global IBus PID 4715 was preserved.
First receipt: `~/.cache/lay/development/run-o0dqrl5c/installation-1.0.67.json`.
Current receipt: `~/.cache/lay/development/ime-cross-app-failure-_ul2eo18/reinstallation-1.0.67.json`.
The user identified the reported cross-app IME/dot issue as pre-existing.
An unnecessary rollback was reversed by reinstalling the identical 1.0.67 bytes,
without a rebuild or global IBus restart; behavioral diagnosis remains open.
Physical acceptance and publication remain pending. Do not bump or rebuild
again merely to publish these already verified 1.0.67 bytes.

Do not rely on commit counts. Before publishing or pushing, run the bump script
or verify the version fields manually.

Before each push or local release:

```bash
scripts/bump-lay-version.sh
```

A source version bump is not a release. Publication status may be recorded only
after one release transaction completes all of these steps:

```text
sync every version surface
-> build the complete release binary set
-> verify the built version and SHA-256 manifest
-> snapshot the currently installed runtime
-> atomically install the verified bytes
-> restart only Lay-managed processes
-> verify source, installed and loaded versions
-> verify live /proc executable SHA-256 parity
-> preserve the global ibus-daemon PID
```

If any step is pending, report the state as `SOURCE_BUMPED_RUNTIME_STALE`, not as
a released version. Never push a version-only state whose matching binaries
were not built and installed or explicitly marked as an unreleased source
snapshot.

If the version was already bumped but the GNOME tray/runtime is stale:

```bash
scripts/bump-lay-version.sh --sync-only
```

The version must be updated in:

- `Cargo.toml`
- `Cargo.lock`
- `extension/lay@radislabus-star.github.io/metadata.json`
- `extension/lay@radislabus-star.github.io/tray_support.js`

1.0.99 fixes apt polkit package selection and removes premature updater memory
reclaim within its existing hard budget. See [release notes](docs/release-1.0.99.md).

1.0.100 is the in-progress shared native replay observation repair.
Exact final client receipts and the existing verifier remain mandatory;
release acceptance and publication are pending.

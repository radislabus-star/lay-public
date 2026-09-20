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

- `1.0.74`

1.0.74: `RELEASE_CANDIDATE` (2026-09-20). This patch release publishes the
installed and physically accepted TD-125 browser autocorrection-boundary repair.
The release tag remains forbidden until the exact 1.0.74 source passes the full
release gate, all ten versioned binaries are built and installed, the four-client
matrix remains exact, and source/installed/loaded identities agree.

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

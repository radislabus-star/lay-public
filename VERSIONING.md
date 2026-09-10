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

Current installed source version:

- `1.0.70`

1.0.70: `INSTALLED_VERIFIED_PHYSICAL_PENDING` (2026-09-10 local date).
The revised GitHub-issues build is installed: daemon `133a1f79e57b`; the other
nine binary hashes match the first 1.0.70. Changed/full each pass 2726 tests;
the public installer passes eight regressions. Current installation receipt:
`~/.cache/lay/development/run-j4e7w_0n/installation-1.0.70.json`.
Public delivery targets `public/main` and `v1.0.70`; its exact remote refs are
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
Physical acceptance and publication are pending. General TD-123 quality and
the compound layout-plus-two-letter error remain OPEN. Do not rebuild or
reinstall merely to publish the verified bytes.

Last published version: `1.0.69`.

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

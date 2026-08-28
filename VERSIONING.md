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

Current publication branch version:

- `1.0.47`

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

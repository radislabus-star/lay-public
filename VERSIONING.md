# Versioning

`lay` uses the patch version as a public release counter:

```text
0.1.<public-release-number>
```

Git does not store reliable "push event" history in the repository, so the
project treats each pushed public release commit as one version step. The bump
script increments the current `Cargo.toml` patch number; it does not derive the
number from `git rev-list`.

Current publication branch version:

- `0.2.237`

Do not rely on commit counts. Before publishing or pushing, run the bump script
or verify the version fields manually.

Before each push or local release:

```bash
scripts/bump-lay-version.sh
```

If the version was already bumped but the GNOME tray/runtime is stale:

```bash
scripts/bump-lay-version.sh --sync-only
```

The version must be updated in:

- `Cargo.toml`
- `Cargo.lock`
- `extension/lay@radislabus-star.github.io/metadata.json`
- `extension/lay@radislabus-star.github.io/lay-impl.js`

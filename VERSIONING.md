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

- `0.1.141`

Do not rely on commit counts. Before publishing or pushing, run the bump script
or verify the version fields manually.

Before each push:

```bash
bash scripts/bump-version-from-git.sh
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
cargo build --release --bins
```

The version must be updated in:

- `Cargo.toml`
- `Cargo.lock`
- `extension/lay@radislabus-star.github.io/metadata.json`
- `extension/lay@radislabus-star.github.io/lay-impl.js`

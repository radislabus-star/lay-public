# Script Ownership

The top-level script directory contains two different classes of tools. Their
paths are intentionally stable because receipts refer to them directly.

## Active Operations

These scripts build, check, install, package, or control the current product:

- `cargo-guard.sh`
- `check-*.sh`
- `install-*.sh`
- `package-extension.sh`
- `lay-runtime-control.sh`
- corpus and model builders without a V10/V11 experiment prefix

Changes to these files can affect current development or runtime operations.

## Historical Research

The following patterns are immutable or append-only reproducibility tools for
completed Slice 8B experiments:

- `lay-v10-*`
- `lay-v11-*`
- `lay_v10_*.rs.inc`

They are not active runtime entrypoints. They stay at their recorded paths so
old receipts remain intelligible, but `.graphifyignore` excludes them from the
active architecture graph. Their current 122-file identity is pinned in
`research/SHA256SUMS` and verified with:

```bash
sha256sum --check scripts/research/SHA256SUMS
```

New one-shot research tooling belongs under `scripts/research/<topic>/` unless
a frozen paper contract requires an exact top-level path. Active operational
scripts must not import or dispatch into the historical V10/V11 controllers.

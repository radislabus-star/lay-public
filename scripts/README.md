# Script Ownership

Active scripts remain here. Completed V10/V11 experiments are preserved in
the [pre-cleanup snapshot](../ARCHIVE.md).

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

The 112 `lay-v10-*` / `lay-v11-*` controllers, ten `lay_v10_*.rs.inc`
fragments and their `research/SHA256SUMS` are archived together at commit
`cb40ef29f6c78c97757dd0059c0ba798cb1f0789`, under their original paths.
Restore the complete snapshot to reproduce those experiments; historical
receipts often also require their separately preserved ignored payloads.

New one-shot research tooling belongs under `scripts/research/<topic>/`.
Current product operations must not dispatch into archived controllers.

## Large Evidence Payloads

`research-evidence-store.py` owns the ignored large-payload lifecycle. The
tracked TD-103 inventory and catalog identify the historical receipt paths;
the bytes live once under
`/home/ubu/projects/lay-immutable-evidence/content-addressed-v1` and remain
openable through relative symlinks at their original paths in the preserved
source checkout. The new clean worktree does not contain those ignored
projections; run historical lifecycle commands in that source checkout.

Verify every projection and object:

```bash
scripts/research-evidence-store.py verify --all
```

Restore a regular file for an external tool, then return it to object-backed
storage:

```bash
scripts/research-evidence-store.py materialize --path <inventory-path>
scripts/research-evidence-store.py externalize --path <inventory-path>
```

The external root is on the same filesystem and is an ownership boundary, not
an independent backup. Never remove it as disposable build cache. The tool
refuses unlisted paths, verifies SHA-256 before every projection change, and
recovers interrupted parent-mode and source-rename transactions from its
durable journal.

The frozen v1 object tree is sealed `0555`. Existing objects remain readable
for verify/materialize/externalize operations; admitting new object hashes
requires a new inventory and storage transaction rather than silently
unsealing this tree.

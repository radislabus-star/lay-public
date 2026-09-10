# Rust Lint Policy

Lay's lint gate has one repository entrypoint:

```bash
scripts/check-lay-lints.sh
```

CI calls the same script. The script first verifies the exact Rust `1.97.1`
toolchain from [the toolchain policy](rust-toolchain-policy.md), then enforces
independent default-feature and `research-tools` contracts across all Cargo
targets.

## Non-Dead Diagnostics

Rustc and Clippy warnings outside `dead_code` are errors. The Clippy route is:

```text
cargo clippy --locked --all-targets -- -D warnings -A dead-code
```

The narrow `-A dead-code` is paired with the exact inventory below; it is not a
general warning waiver. Existing wide proof/compiler APIs and bounded inline
enums use item-local `#[expect(..., reason = "...")]` only where changing the
representation would be a separate refactor. A stale expectation becomes an
`unfulfilled_lint_expectations` error, and a new unannotated occurrence fails.

## Dead-Code Ledger

Two ledgers own the feature-dependent diagnostic surfaces:

- [`dead_code.default.json`](../scripts/lint-baseline/dead_code.default.json)
  owns the default-feature inventory;
- [`dead_code.json`](../scripts/lint-baseline/dead_code.json) owns the
  `research-tools` inventory.

They cannot be modeled as a subset relation. Enabling `research-tools` both
adds proof/research code and consumes helpers that are unused in the default
build. The current sealed inventories are therefore 542 default rows and 366
`research-tools` rows, with neither lane treated as authority for the other.

Each V4 row owns its diagnostic code, exact primary source byte span, normalized
source subject, source path, and Cargo target context. The normal comparison is
exact in both directions:

- a new item, renamed item, moved path, or changed target fails;
- removing an item leaves a stale baseline row and fails;
- count-preserving or same-shaped cross-location churn cannot pass because row
  identities must match.

The ledger is temporary debt, not acceptance that the code is useful. TD-008
must delete proven residue and lower this file. Retained proof/compiler rows
that need build-surface isolation belong to the explicit TD-104 decision.

## Updating The Ledger

After deliberately removing or re-owning dead code:

```bash
scripts/check-lay-lints.sh --self-test
scripts/check-lay-lints.sh --write-baseline
git diff -- scripts/lint-baseline/dead_code.json
git diff -- scripts/lint-baseline/dead_code.default.json
scripts/check-lay-lints.sh
```

`--write-baseline` rejects every added logical item or multiplicity increase,
stages both feature-scoped candidates, and publishes them only after both rustc
inventories and both hard Clippy routes pass. Exact locations make the ordinary
gate fail closed after source movement; the explicit writer may re-anchor an
equal or reduced logical inventory. Review both files whenever that happens.
A compiler update requires completing the toolchain update procedure first
because diagnostic identity is compiler-bound.

Optional `direct-llm` remains outside this default-feature lint contract.

# Rust Lint Policy

Lay's lint gate has one repository entrypoint:

```bash
scripts/check-lay-lints.sh
```

CI calls the same script. The script first verifies the exact Rust `1.97.1`
toolchain from [the toolchain policy](rust-toolchain-policy.md), then enforces
two separate contracts for default features and all Cargo targets.

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

[`scripts/lint-baseline/dead_code.json`](../scripts/lint-baseline/dead_code.json)
contains the canonical unique dead-code rows. Each row owns its diagnostic code,
source path, normalized source subject, same-shaped occurrence ordinal, and
Cargo target context. Byte offsets are used only while parsing to distinguish
source items from duplicate Cargo emissions; they are not stored in the ledger.
The comparison is exact in both directions:

- a new item, renamed item, moved path, or changed target fails;
- removing an item leaves a stale baseline row and fails;
- count-preserving churn cannot pass because row identities must match.

The ledger is temporary debt, not acceptance that the code is useful. TD-008
must delete proven residue and lower this file. Retained proof/compiler rows
that need build-surface isolation belong to the explicit TD-104 decision.

## Updating The Ledger

After deliberately removing or re-owning dead code:

```bash
scripts/check-lay-lints.sh --self-test
scripts/check-lay-lints.sh --write-baseline
git diff -- scripts/lint-baseline/dead_code.json
scripts/check-lay-lints.sh
```

`--write-baseline` rejects every added row, stages a strict reduction, and
publishes it only after both the rustc inventory and hard Clippy route pass. A
compiler update requires completing the toolchain update procedure first
because diagnostic identity is compiler-bound.

Optional `direct-llm` remains outside this default-feature lint contract.

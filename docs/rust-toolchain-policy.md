# Rust Toolchain Policy

Lay has two explicit compiler contracts. They change independently.

## Supported Floor

The minimum supported Rust version is `1.88.0` for the locked active target
graph with default features and with `lexical-compiler` enabled:

```bash
CARGO_TARGET_DIR=target/msrv-1.88.0 \
  LAY_RUST_TOOLCHAIN=1.88.0 \
  scripts/cargo-guard.sh check --locked --all-targets

CARGO_TARGET_DIR=target/msrv-1.88.0 \
  LAY_RUST_TOOLCHAIN=1.88.0 \
  scripts/cargo-guard.sh check --locked --all-targets \
    --features lexical-compiler
```

Pinned identity:

```text
rustc 1.88.0
commit 6b00bc3880198600130e1cf62b8f8a93494488cc
cargo 1.88.0 (873a06493 2025-05-10)
```

The immediately lower `1.87.0` compiler is rejected before project
compilation because locked `image 0.25.10` declares `rust-version = 1.88.0`.
The MSRV therefore describes the current locked dependency graph as well as Lay
source. Optional `direct-llm` and its native dependency are outside this
contract; the README's MSRV claim has the same feature scope.

## Development And Lint Toolchain

`rust-toolchain.toml` pins normal development, formatting and lint diagnostics
to `1.97.1`:

```text
rustc commit 8bab26f4f68e0e26f0bb7960be334d5b520ea452
cargo 1.97.1 (c980f4866 2026-06-30)
clippy 0.1.97 (8bab26f4f6 2026-07-14)
rustfmt 1.9.0-stable (8bab26f4f6 2026-07-14)
```

Run the same identity assertions used by CI before collecting warnings or
changing the lint baseline:

```bash
scripts/verify-rust-toolchain.sh lint
```

The lint verifier intentionally checks unqualified `rustc`, `cargo`,
`clippy-driver`, and `rustfmt` from the repository root. This proves the same
toolchain selected by subsequent plain CI commands, including the
`rust-toolchain.toml` override.

## Updating

1. Change the development pin in `rust-toolchain.toml`, the exact CI action
   revision and the `lint` identities in `scripts/verify-rust-toolchain.sh`.
2. Run the identity verifier before generating a new warning inventory.
3. Change `package.rust-version` only when the support floor changes. Test the
   proposed floor and the immediately lower release in separate guarded target
   directories with `--locked --all-targets`.
4. Update this document and the README badge in the same change.
5. Do not update dependencies merely to make a floating toolchain gate green;
   dependency changes require their own review.

# TD-004: Enforce The Real MSRV And Lint Toolchain

Status: `DONE`
Priority: `P0`
Class: build and release contract
Size: `S`
Depends on: TD-001

## Why Now

`Cargo.toml` and README claim Rust 1.75+, while CI tests only current stable and
the source uses APIs introduced after 1.75. The published compatibility promise
is therefore unverified and likely false.

## Evidence

- `rust-version = "1.75"` and README badge `Rust-1.75+`.
- Current local build uses Rust 1.97.1 / Cargo 1.97.1.
- The source contains many `Option::is_none_or(...)` calls, stabilized after
  Rust 1.75.
- CI installs only `stable`; it has no MSRV job.

## Target State

The project either compiles on the lowest declared compiler or declares the
lowest compiler that actually passes. CI verifies that exact version. The
diagnostic/lint baseline uses a separately pinned exact toolchain rather than a
floating `stable`, so TD-005 can compare stable warning identities.

## Scope

- Determine the lowest practical toolchain by compiling/checking the active
  targets, not by syntax search alone.
- Prefer raising the honest MSRV over backporting many harmless standard-library
  calls unless supported distributions require the older compiler.
- Pin an MSRV CI job and keep stable clippy/tests separate.
- Pin the exact lint/stable toolchain and document its update procedure
  separately from changing MSRV.
- Align Cargo metadata, README, and contributor/release docs.

## Non-Goals

- No dependency update unless required by the chosen MSRV.
- No promise for optional `direct-llm` unless its native dependency is included
  in the tested matrix.
- No broad syntax rewrite solely to preserve Rust 1.75.

## TDD Plan

1. Run metadata/check on candidate installed toolchains below and above the
   expected floor.
2. Record the first passing version and failure class immediately below it.
3. Add the CI job.
4. Verify stable remains green.

## Acceptance Gates

- Exact declared MSRV passes the documented build/check route.
- The immediately lower tested version fails for a recorded compiler or
  dependency reason, or the chosen floor is justified by support policy.
- README and Cargo metadata agree.
- CI tests the exact MSRV and the exact lint/stable version.
- Local and CI commands report the same compiler commit before diagnostic
  inventory generation.

## Risks And Guardrails

- Sharing target directories across compiler versions can create misleading
  artifacts; use guarded isolated targets and delete them afterward.
- Do not trigger an uncontrolled toolchain download in a quota-constrained temp
  HOME.

## Independent Review Brief

Verify the tested compiler bytes/version, target set, optional feature scope,
and documentation consistency. Score 1-10.

## Completion Record

- Base commit: `b8699f3b`
- Implementation commit: pending closure commit
- Review pass 1: `7/10`; two medium findings and one low finding
  identified the unqualified CI-tool mismatch, missing `lexical-compiler` MSRV
  coverage, and unnecessary host-triple restriction.
- Review pass 2: `10/10`; no findings after correction.
- Exact floor: Rust `1.88.0`, rustc commit
  `6b00bc3880198600130e1cf62b8f8a93494488cc`, Cargo
  `1.88.0 (873a06493 2025-05-10)`.
- Exact lint toolchain: Rust `1.97.1`, rustc commit
  `8bab26f4f68e0e26f0bb7960be334d5b520ea452`, Cargo
  `1.97.1 (c980f4866 2026-06-30)`.
- Lower-bound evidence: Rust `1.87.0` exits `101`; locked
  `image 0.25.10` and Lay both require Rust `1.88`.
- Verification:
  - exact 1.88 default-feature `--locked --all-targets`: PASS;
  - exact 1.88 `lexical-compiler` `--locked --all-targets`: PASS;
  - exact 1.87 lower-bound rejection: PASS;
  - MSRV/lint identity verifiers and hostile active-toolchain test: PASS;
  - TOML, YAML, and shell syntax: PASS;
  - `scripts/cargo-guard.sh fmt --check`: PASS;
  - `scripts/check-lay-changed.sh`: PASS;
  - `git diff --check`: PASS.
- Untested and unsupported by this contract: optional `direct-llm` and its
  native dependency.
- Runtime authority changed: no.

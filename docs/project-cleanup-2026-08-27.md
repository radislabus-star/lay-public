# Lay Project Cleanup - 2026-08-27

## Scope

This cleanup separates active runtime source, historical research tooling,
immutable evidence, and disposable generated state. It is behavior-preserving:
no release build, installation, service restart, runtime authority change, or
scientific rerun is admitted.

## Baseline

Measured before cleanup:

| Surface | Baseline |
|---|---:|
| tracked files | 4,909 |
| tracked bytes | 620,614,184 |
| top-level scripts | 173 |
| V10/V11 one-shot scripts | 112 |
| local receipt payloads | 3.6 GiB |
| ignored receipt files | 2,100 |
| Cargo target | 6.9 GiB |
| graphify output | 781 MiB |
| graph nodes | 50,049 |
| graph edges | 143,289 |
| receipt path occurrences in graph | 94,031 |
| loose Git objects | 492.50 MiB |
| dangling blobs | 625 / 514,548,825 bytes on disk |

The large graph was not an architecture graph: it indexed copied source trees,
ELF-derived outputs, and controller snapshots beneath structural-gate receipts.

## Ownership Boundaries

| Surface | Owner | Cleanup rule |
|---|---|---|
| `src/`, runtime tests | product source | refactor only with behavior checks |
| compact receipt files | Git proof ledger | remain tracked and immutable |
| nested execution evidence | local evidence archive | remain on disk, ignored by Git |
| V10/V11 one-shot controllers | research archive | stable paths retained |
| `graphify-out/` | generated architecture index | derive only from active corpus |
| `target/`, graph caches | generated cache | disposable after verification |
| installed 1.0.44 | runtime authority | unchanged by cleanup |

## Admitted Work

1. Exclude receipt trees and completed one-shot controllers from graphify.
2. Prune only unreachable Git blobs that byte-match ignored local evidence.
3. Remove disposable dated graph snapshots and stale AST cache.
4. Split oversized source modules by ownership without changing algorithms.
5. Document active and historical script routes without breaking stable paths.

## Verification

- `git diff --check`
- scoped Rust checks through `scripts/cargo-guard.sh`
- relevant Python and UI contract checks when touched
- graph corpus contains zero receipt paths and zero V10/V11 controller paths
- runtime process identities remain unchanged
- `graphify update .` after source edits

## Result

Completed without changing installed runtime authority.

| Surface | Before | After |
|---|---:|---:|
| graphify output | 781 MiB | 30 MiB |
| graph nodes | 50,049 | 19,774 |
| graph edges | 143,289 | 50,442 |
| receipt path occurrences in graph | 94,031 | 0 |
| historical controller paths in graph | present | 0 |
| loose Git objects | 492.50 MiB | 1.53 MiB |
| dangling blobs removed | 0 | 613 |
| dangling object bytes removed | 0 | 512,977,257 |
| Cargo target | 6.9 GiB baseline / 7.7 GiB after gates | removed |
| `proposal_admission.rs` | 3,423 lines | 535 lines |

The proposal-admission implementation is now separated into:

| File | Role | Lines |
|---|---|---:|
| `proposal_admission.rs` | facts, public decisions, orchestration | 535 |
| `structural_guards.rs` | structural and context predicates | 1,236 |
| `surface_support.rs` | lexical support and repair predicates | 855 |
| `trace.rs` | test-only measurement plumbing plus non-test no-op macros | 467 |
| `tests.rs` | focused unit fixtures | 338 |

Historical research paths remain unchanged and are pinned by 122 entries in
`scripts/research/SHA256SUMS`. All entries verify.

Evidence preservation:

- ignored local evidence: 2,100 files / 3,729,707,060 bytes, unchanged;
- release receipt SHA-256 remains
  `49d3383bd151beed259421d153f0d1aaf0e0a6ff44daa4fa6a6656c84d7b725a`;
- GNOME correction receipt SHA-256 remains
  `d88ec4eec38203ea7747980fbf2d43159cd69ab20d7da278d1d5ebdb20d4155e`.

Final verification:

- structural cleanup route gate: `PASS`;
- proposal-admission unit route: 7 passed, 0 failed;
- typing-transition authority contract: 21 passed, 0 failed;
- non-test `cargo check --lib`: passed;
- `scripts/check-lay-changed.sh`: passed;
- transition shadow replay: `PASS-shadow`, 469 records, 0 false applies;
- unsafe edit scoreboard: `PASS`, 0 gate failures;
- research archive SHA-256: 122/122 passed;
- `git diff --check`: passed;
- graph receipt/controller path checks: 0/0;
- Lay process IDs before and after cleanup: unchanged.

One intermediate integration compile exposed an incorrectly outer-gated
test-trace include. The include boundary was repaired, and both normal-library
and test-library configurations passed afterward.

No full historical `correction_core` sweep was rerun: this refactor changes no
decision logic, and the established wide suite contains known baseline
failures. No release build, install, runtime restart, or authority migration was
performed.

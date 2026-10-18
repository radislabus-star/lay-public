# zstd 0.14 dependency maintenance

Date: 2026-10-18. Scope: the dependency update originally proposed in public
PR #58, applied to current source after the CI action maintenance change.

The original PR's Rust check stopped at a stale architecture receipt. That
failure does not establish a codec regression. Cargo.toml participates in the
receipt input, so the supported remote graph refresh is required for this
dependency change without relaxing the architecture guard.

Move to the `zstd = "0.14"` release line and lock its current patch release
0.14.3, published on October 16. Upstream 0.14 fixes prepared-dictionary lifetime
safety and changes the license to BSD-3-Clause; subsequent patches fix additional
context, buffer and decoder ownership/error paths.
The crate still declares Rust 1.64 support, below Lay's Rust 1.88 MSRV. Primary
references are the upstream [0.14.0 release](https://github.com/gyscos/zstd-rs/releases/tag/v0.14.0),
[0.14.2 release](https://github.com/gyscos/zstd-rs/releases/tag/v0.14.2),
the [0.14.3 source comparison](https://github.com/gyscos/zstd-rs/compare/v0.14.2...v0.14.3)
and [crate metadata](https://crates.io/crates/zstd/0.14.3).

The resolved lockfile changes only zstd 0.13.3 to 0.14.3, zstd-safe 7.2.4 to
8.2.0 and zstd-sys 2.0.16 to 2.1.2. The native codec remains zstd 1.5.7;
that fact alone is not cross-version compatibility proof. Registry checksums
for all three locked crates match their primary crates.io metadata.

Lay uses `zstd::bulk::compress` and `zstd::bulk::decompress` in the lexical v8
codec and two posting-bounds encoders. It does not use the changed streaming
decoder or prepared-dictionary APIs. Existing V8 relation/readout parity,
general format and posting-bound coverage remain the owning source checks.
Do not change model bytes, compression settings, format identifiers, runtime
authorities or delivery routes to accommodate the dependency update.

Acceptance requires remote lockfile resolution, a fresh architecture receipt,
independent review and all exact-head GitHub checks, including the complete
correctness/package lanes, locked MSRV checks, lint/build and a fresh RustSec
audit. The public PR and private follow-up receipt own measured results. New
physical-client acceptance is NOT_TESTED; cold-load and input latency are
NOT_MEASURED. The newer bulk decoder pre-scans frame sizes, so unchanged codec
version does not establish unchanged load cost. A passing round trip alone
does not prove old-package compatibility or improvement in restoration quality.

## Measured source verification

The supported remote graph refresh passed. The complete guarded remote
development check then passed all 3,212 selected tests (3,176 correctness and
36 package tests), formatting and the helper self-tests. The frozen source
archive SHA-256 was
`df3a7abcaa7213cc4a779e13b8f8261b2a3aaab8f02ced9a8b333c859816bfce`.
The later edits only narrow this document's coverage claim and record these
results. Independent review: 9/10, no blocking findings.

Receipts:

- `/home/ubu/.cache/lay/development/tab-graph-tc0e6q95/RESULT.json`
- `/home/ubu/.cache/lay/development/run-0qv4lpc1/RESULT.json`
- `/home/ubu/.cache/lay/development/public-followup-20261018/zstd-review.json`

The preceding `run-gz2bz_p4` stopped before Rust tests because the offline cache
lacked the newly locked dependencies. Those dependencies were fetched under
the existing remote resource guard before the successful fresh run; no product
test or guard was changed to obtain PASS. Exact-head GitHub acceptance remains
pending until the PR checks complete. Nothing was installed or restarted;
runtime authority is unchanged.

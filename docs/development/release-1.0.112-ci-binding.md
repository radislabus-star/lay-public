# Release 1.0.112 CI observation binding

## Before-edit boundary

Baseline: `1ea1d44e6723c86e7553681de966e41ee39e6891`, public tag
`v1.0.112`. This is a test-evidence metadata repair, with no runtime changes.
The accepted tag and running KDE binaries remain unchanged.

First failed transition: `scripts/test_lanes/cli.py::run_non_timing` finishes
the selected targets, then `contracts.py::load_known_failures` rejects the
ledger's obsolete `manifest_sha256`. CI run
[40967927798](https://github.com/radislabus-star/lay-public/actions/runs/40967927798)
records 34 distinct target summaries, exactly matching all 3559 correctness
and 36 package test entries in the published manifest, with zero failures.
The overall CI run failed; later lint and release-build steps did not execute.

Allowed changes are the known-failure ledger's observation binding, a receipt
derived from that actual CI output, and their owning documentation/scope.
Do not add allowed failures, alter the manifest or validator, change CI gates,
or edit any Rust, tray, GNOME, delivery, ownership, focus or rendering source.

## Repair and verification

Preserve `failure_count: 0` and `failures: []`. Bind the ledger to the current
manifest and the new independently attributable CI observation. The observation
must distinguish executed tests from the failing final ledger check and must
not claim the later skipped steps passed. Include the CI source commit, URL,
log hash, exact per-target coverage and actual observed event timestamp.

Validate both hashes and the unchanged failure list using the existing loader.
Verify the declared diff and absence of runtime changes. Push the metadata
repair and read back automatic CI; do not move the accepted release tag.
Source-test success remains separate from native desktop acceptance.

## Result

The existing `load_known_failures` loader accepted the repaired ledger and
receipt. Exact target coverage is 34/34, selected tests 3595/3595, allowed
failures remain zero. The declared scope check passed. Runtime source, the
published manifest and the loader have no diff against the accepted baseline.
Automatic CI40970843062 passed the complete correctness/package step after
push. It then failed the separate Clippy argument-count diagnostic; the owning
continuation is release-1.0.112-ci-lint.md. No native checks were repeated.

## Final automatic CI readback

Observed 2026-10-30T00:02:46Z: public main `fca6f8427b76782b246fb2b7cdfaff7fd97f9d4b`
passed [CI40973676834](https://github.com/radislabus-star/lay-public/actions/runs/40973676834)
and [Security40973676896](https://github.com/radislabus-star/lay-public/actions/runs/40973676896).
Complete actual transcript matches34 distinct targets/3595 selected tests with
zero failures and no missing target. Both exact dead-code inventories117/129
and hard Clippy routes pass; GNOME/KDE/shell syntax, CLI smoke, all release
binaries and ngram-cache probe pass. CI ran23:31:07–23:59:27UTC on29October.
Private complete readback: CACHE/release112-final-ci-readback.json; full
transcript: CACHE/release112-final-ci-job.log. This is proof for main after the
CI maintenance fixes; immutable accepted v1.0.112 still points to1ea1d44e.
No native full matrix, installation or runtime restart was performed for these
CI fixes. Read-only runtime verification at00:02UTC on30October still finds
CLI1.0.111 on both hosts and the exact accepted remote IME05d62373/daemon35527e63
processes. Installed updater discovery of1.0.112 is independently saved.
Native timeout evidence stays OPEN and browser totals remain338/24/78/440.

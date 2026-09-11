# Release 1.0.71 installed evidence

Status: `INSTALLED_VERIFIED_PHYSICAL_PENDING`.

This document is the owning release evidence for 1.0.71. The release includes
the accepted repairs for terminal word-boundary replacement, clean repeated-word
handling, candidate comparison, and within-word Backspace/pending-learning
feedback. No code, model package, source version,
guard, verifier route, or input-source route changed during this final
document update.

## Installed runtime

The second installer attempt completed at `2026-09-11T09:54:08Z`.

- Installation receipt: `/home/ubu/.cache/lay/development/release-1.0.71-20260911-hiu22jcy/installation-1.0.71-attempt2.json`
- Installation receipt SHA-256: `8e6124c06c0f41543be7c077d84964cfcb3f919cd73d84ddd4a2cc19a665b9be`
- Post-install runtime receipt: `/home/ubu/.cache/lay/development/release-1.0.71-20260911-hiu22jcy/post-install-runtime-1.0.71.json`
- Post-install runtime receipt SHA-256: `fa9a979c2c1eefd367cbe7c7f24ebe15d655d0e85db0558fb32938449122d3a8`
- Installed candidate: `2ee1479cfc84c40ce3e8673e573d2dbfc42226f0e4dbdeb7cc310f6f1a63fcc5`
- CLI version: `lay 1.0.71`
- Loaded extension version: `1.0.71`
- Selected engine: `lay-ime-ru`
- Preserved global IBus PID: `4715`

Loaded Lay processes after installation:

| Component | PID | SHA-256 |
| --- | ---: | --- |
| IME | `4152536` | `2ee1479cfc84c40ce3e8673e573d2dbfc42226f0e4dbdeb7cc310f6f1a63fcc5` |
| daemon | `4152503` | `7680d8680563d48d8591106cc852960137339535d4ee377d86a7b5763f63780e` |
| L3 online | `4152477` | `f40ee7d5ea2b20e58da86937acfa80c3e3cddd5677be8574e5d285265713a7cf` |
| L1.1 serve | `4152326` | `db825d2f244282392fe507ebeee335ef3e66ee33fc35f6e701a688abc7845034` |

All 10 release binaries were replaced by the verified 1.0.71 artifacts. The
user config, GNOME input sources, LayRU selection, IBus daemon, and eight
immutable model/data dependency payloads were preserved. The L1.1 executable
dependency changed from the accepted 1.0.70 service hash to
`db825d2f244282392fe507ebeee335ef3e66ee33fc35f6e701a688abc7845034`; the new
proof manifest is `1092cd2f72ddd3ae52da6436ceba47f4e26814f137eded46515d3667d3b38314`.

## Input-journal cutover

The installer cut over input journals at `2026-09-11T09:54:01.913546Z`. Exactly
10 old journal files were deleted without archive. No local journal cleanup is a
public installer feature; it is this local install-state reset.

After the deletion, L3 was started alone with `--job-mode=ignore-dependencies`
and observed the new empty usage journal before daemon or IME resumed:
`source_offset=0`, `source_tail_hashes=[]`. The before/after checks required no
IME process, stopped daemon, and stopped-unit state with inactive/failed,
`MainPID=0`, `ControlPID=0`, and empty `Job`.

The L1.1 readiness path on the successful second attempt observed
`socket_refused -> warming -> ready`, then ran one strict guard. The installation
receipt has `command_failures: []` by absence of the field.

The first failed attempt remains recorded as an incident:
`/home/ubu/.cache/lay/development/release-1.0.71-20260911-hiu22jcy/installation-1.0.71.json`
SHA-256 `e7eebd08471600267822629223a736cf8909207d2bd4705344a29aab734c0278`.
The recovery and provenance-correction receipts remain part of that incident
history, not the final installed state.

## Proof receipts

- Full identity: `/home/ubu/.cache/lay/development/release-1.0.71-20260911-hiu22jcy/release-1.0.71-full-acceptance-completed-identity.json`
- Full identity SHA-256: `7585f7df394fa6d06c15181923d68b553f07f0bf788f3184a4b2baf5da7c36e7`
- Acceptance summary: `/home/ubu/.cache/lay/development/release-1.0.71-20260911-hiu22jcy/acceptance-summary-1.0.71.json`
- Acceptance summary SHA-256: `b08915f4bdf76e1951d25a98feb0abf4118fe308b189c3ddf41580ddf196a68d`
- Installation review SHA-256: `69889a16f90fc6fb70f957b6103b655b1ea77be170274e021418b52cf57b276f`

The fresh full identity binds 699 source files and all 10 release binaries. The
699 source files are unchanged by this document update.

## Metrics and scope

| Lane | Result | Scope |
| --- | --- | --- |
| changed gate | `2756/2756` | Release changed gate over accepted source repairs. |
| full gate | `2756/2756` | Release full gate; performance, ngram and audit extras were not selected. |
| client checks | `17/17` | Private client controls over the 1.0.71 dependency binding. |
| fixed89 | `267/267` exact-output same | Regression parity across three profiles; quality is `UNKNOWN` outside this fixed set. |
| diagnostic18 | `14/18` | `7/11` dirty restored, `7/7` clean preserved, `0` wrong outputs. |

Fixed89 candidate profile totals:

| Profile | All correct | Dirty correct | Clean correct | Wrong outputs |
| --- | ---: | ---: | ---: | ---: |
| strict | `56/89` (`62.921%`) | `15/47` (`31.915%`) | `41/42` (`97.619%`) | `1` |
| normal | `60/89` (`67.416%`) | `19/47` (`40.426%`) | `41/42` (`97.619%`) | `4` |
| experimental | `60/89` (`67.416%`) | `20/47` (`42.553%`) | `40/42` (`95.238%`) | `5` |

Fixed89 per-class candidate rows:

| Class | strict | normal | experimental |
| --- | ---: | ---: | ---: |
| restoration_regressions | `7/24` (`29.167%`) | `9/24` (`37.5%`) | `10/24` (`41.667%`) |
| missing_letter | `3/8` (`37.5%`) | `5/8` (`62.5%`) | `5/8` (`62.5%`) |
| repeated_letter | `0/4` (`0.0%`) | `0/4` (`0.0%`) | `0/4` (`0.0%`) |
| transposition | `5/5` (`100.0%`) | `5/5` (`100.0%`) | `5/5` (`100.0%`) |
| context_fixture | `0/6` (`0.0%`) | `0/6` (`0.0%`) | `0/6` (`0.0%`) |
| clean_missing_letter_control | `3/4` (`75.0%`) | `3/4` (`75.0%`) | `3/4` (`75.0%`) |
| clean_repeated_letter_control | `4/4` (`100.0%`) | `4/4` (`100.0%`) | `4/4` (`100.0%`) |
| clean_valid_word | `34/34` (`100.0%`) | `34/34` (`100.0%`) | `33/34` (`97.059%`) |

Diagnostic cases still not restored: `nfr b ` with the final space is now
unchanged rather than wrong, plus `видешь`, `видешь!`, and `выровнить`. This is
not a new general quality claim and does not close the remaining TD-123 quality
work.

## Limits

Physical keyboard acceptance is pending. No public publication, Ubuntu/ARM
compatibility, RSS, latency, universal quality, or general heldout PASS is
claimed here. The fixed89 lane proves regression parity and exact-output
stability, not general correctness. The diagnostic lane is the fixed private
poor-input probe only.

Exact final document graph completion is established only by
`/home/ubu/.cache/lay/development/release-1.0.71-20260911-hiu22jcy/final-document-graph/fetch-receipt.json`
with `PASS`.

Exact publication refs are established only by
`/home/ubu/.cache/lay/development/release-1.0.71-20260911-hiu22jcy/publication.json`.
After publication, the public documentation path is:
`https://github.com/radislabus-star/lay-public/blob/v1.0.71/docs/release-1.0.71-preflight-2026-09-11.md`.

# Public issue pass — 2026-10-16

User requested sequential repair/testing/review/publication, replies and closure
where resolved, followed by installation of the final version. Source checkout:
`lay-space-boundary-shift-20261007`, branch `codex/space-boundary-shift-20261007`.
Starting HEAD `db5cc783`; installed runtime1.0.87 from `50d314b3`.
No force push to the divergent private `origin/main`.

| Issue | Current disposition | Evidence / owning task |
| --- | --- | --- |
| [#43](https://github.com/radislabus-star/lay-public/issues/43) | CLOSED, original ARM64 installation failure resolved | Reporter confirmed successful install; maintainer inspected successful build16:17. [Closure](https://github.com/radislabus-star/lay-public/issues/43#issuecomment-6206451592). No new ARM64 run or compilation speedup claimed. |
| [#44](https://github.com/radislabus-star/lay-public/issues/44) | CLOSED, original AMD64 installation failure resolved | Reporter confirmed successful install; maintainer inspected successful build6:27. [Closure](https://github.com/radislabus-star/lay-public/issues/44#issuecomment-6206452385). Native switching tracked separately. |
| [#48](https://github.com/radislabus-star/lay-public/issues/48) | CLOSED, source798fa391 pushed | [TD-146](../../tech_debt/146-session-scoped-input-access.md),11 access +8 public-build tests/shell PASS, review9/10; [closure](https://github.com/radislabus-star/lay-public/issues/48#issuecomment-6210186091). Real seat ACL NOT_TESTED. |
| [#59](https://github.com/radislabus-star/lay-public/issues/59) | CLOSED, source05e1e448 pushed; [reply](https://github.com/radislabus-star/lay-public/issues/59#issuecomment-6211314244) | [TD-148](../../tech_debt/148-recognize-backed-ti-present-forms.md), same-stem backed verb forms |
| [#46](https://github.com/radislabus-star/lay-public/issues/46) | DONE_SOURCE3209/3209, code review9/10; release native pending | [TD-147](../../tech_debt/147-explicit-undo-and-double-space.md); preserve existing restoration learning and single-owner routes. Source complete; final native verification/install pending. |
| [#47](https://github.com/radislabus-star/lay-public/issues/47) | OPEN, current reporter startup result missing | [TD-145](../../tech_debt/145-verify-issue47-current-version.md); source and isolated core verified, latest reporter version1.0.74. |

Live queue and readback receipts:
`/home/ubu/.cache/lay/development/issues-final-pass-20261016/`.
GitHub comments/closure are recorded there and read back after each mutation.

## Current release gate

Published b4111822 (TD150), source3210/3210, review9/10, built88/audit0.
Its matched complete432: Firefox169PASS/28FAIL/19BLOCKED, Chrome179/23/14,
total348/51/33, NOT_TESTED0. Whole new receipts:
`/home/ubu/.cache/lay/development/td150-install88-20261016/native/`.
Original fullscreen/space8 repeat now5/5; Firefox standard60/60 and420/420
median98.694ms/p95112.273ms. Chrome406/420 (14 missing)82.307/p9597.386ms.
No Chrome lostPASS, one Firefox lostPASS functional/normal/space_en_ru/7.
Installed87 unchanged; installation BLOCKED_NATIVE_AND_CI.

[TD151](../../tech_debt/151-preserve-prepared-space-frame-on-matched-reset.md)
proves and minimally repairs prepared Space frame loss before owned Reset
observation restoration. Causal RED2/3, final focused13/13, controlled old selector
1/1 detected, independent code review9/10; full3211/3211 and final canonical PASS; final code/evidence review9/10. No route/model/
deadline change. New build/new complete432/options80 remain pending.

[TD152](../../tech_debt/152-control-overlapping-release-test-readiness.md)
keeps original CI38901069439 cause UNKNOWN: unchanged isolated overlap baseline
PASS. Proved reporting evidence loss RED16/17 →17/17 green; bounded panic-anchored
JSON excerpt, unchanged failure policy and no speculative runtime/fixture change.
Full3211/3211 and current17/17 encoding assertions/canon PASS, final code review9/10. Final-source GitHub CI remains pending.

## Preserved historical receipts

E6 complete432 also348/51/33 but loses normal/short4; its one allowed unchanged-
byte full Firefox confirmation loses standard/fullscreen/space8 instead. Both
complete receipts remain rejected/immutable; no third same-byte rerun. TD150's
new bytes pass both old rows, but the later Space7 loss is separate.
`/home/ubu/.cache/lay/development/td149-install88-20261016/native/` and
`native-recheck/`. Optional80 on E6 only:60PASS/16FAIL/4blocked preconditions;
NOT proof for B411 or final TD151 bytes. Complex rich-editor limits remain open
under TD121. Historical READV349/51/32 is not mixed into a new matrix.

[TD149](../../tech_debt/149-update-rustls-security-patch.md): rustls-only patch,
same fresh DB before4/after0; source3209/3209, MSRV1.88 both configurations,
strict lints and review9/10. Current lock retains that exact audited identity.
Reporter-specific issue47 remains OPEN; no new reporter result. No client
messages are sent by native tests. No publication or source proof equals local
installation or universal physical-client acceptance.

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

Current source e6f3227c1.0.88 complete native432: Firefox169/28/19 and
Chrome179/23/14, total348/51/33; NOT_TESTED0. Exact receipts in private
td149-install88-20261016/native/. One lost historical Firefox PASS is under
validation; no lost Chrome PASS. Historical READV349/51/32 remains separate.
Optional shortcut80:60PASS/16FAIL/4blocked preconditions; no universal acceptance.
Source3209, compiled bytes, installed87, native clients and timing stay separate.
No client messages are sent.

The one same-byte full Firefox confirmation also completes216/9 jobs at169/28/19
but loses standard/fullscreen/space/field8; original short/field4 passes there.
Both receipts remain separate and installation is BLOCKED_NATIVE. No third
unchanged-byte full rerun. The saved per-cell trace yields a causal source RED
and minimal [TD-150](../../tech_debt/150-retain-completed-native-replay-across-reset.md)
completed-replay Reset capture repair; focused11/11 plus controlled reversion1/1.
Full source/review/new-byte build and client proof are still pending.

Release prerequisite [TD-149](../../tech_debt/149-update-rustls-security-patch.md):
rustls-only security patch, same fresh DB before4/after0; full source3209/3209,
MSRV1.88 both configs and strict lints PASS, review9/10; native/build/install pending. This does not close the reporter-specific issue47.

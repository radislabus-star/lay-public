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

Last complete native browser receipt remains Firefox170/27/19/216 and
Chrome179/24/13/216, total349/51/32/432; historical NOT_TESTED0.
This is not acceptance of1.0.87 or subsequent patches. New source results,
installed identities, native client acceptance and typing latency remain
separate scopes. No client messages are sent during tests.

Release prerequisite [TD-149](../../tech_debt/149-update-rustls-security-patch.md):
rustls-only security patch, same fresh DB before4/after0; full source3209/3209,
MSRV1.88 both configs and strict lints PASS, review9/10; native/build/install pending. This does not close the reporter-specific issue47.

# Six sequential debt releases — 2026-10-25

User authorization: take the six groups reported in this session in their stated
order, finish and publish each release, and continue without asking for routine
approval. Work directly: no Harness2 or subagents. Starting source is
07867bf6683f7fa673e530281a0c72b2e55f6b88, public stable v1.0.96.

The sequence below reserves versions, not results. A version is published only
after its actual change, required source/build gates, applicable native
acceptance, and installed updater discovery are recorded. Unrelated known
limits stay visible. No repeat-until-green, changed denominators, blanket lint
suppression, new input route, or unproved cross-desktop fallback is permitted.
Preserve the accepted local GNOME runtime; install candidates only in the owned
remote KDE session. Evidence must distinguish source, loaded bytes, observed
client effects, BLOCKED prerequisites and NOT_TESTED coverage.

| Order | Planned release | Scope | State |
| --- | --- | --- | --- |
| 1 | 1.0.97 | KDE Firefox functional failures and default native observation prerequisites | RELEASED: bounded closed-erasure repair; complete source/build gates, raw native limits and updater discovery recorded |
| 2 | 1.0.98 | KDE Chrome Space layout and moved-boundary failures | RELEASED at2026-10-26T14:25:12Z,5052c3df;295P66F79B. Both installed updater discoveryPASS; retained native failures stayOPEN. |
| extra installer blocker | 1.0.99 | Ubuntu26.04 missing transitional polkit package (#70 installer) | SOURCE/BUILD_GREEN; apt regression11/11, updater9/9; final CI and bounded compilation proof pending |
| 3 | 1.0.100 | Shared long-sequence lifecycle/rapid gestures, remaining inline presentation, terminals and native coverage | QUEUED; Firefox residuals from97 retained here |
| 4 | 1.0.101 | Build warnings: measured reachability and minimal research/runtime boundaries | QUEUED |
| 5 | 1.0.102 | Remaining core/structure tasks: reconcile TD-122/123/132/134 against current proofs, repair demonstrated mechanisms | QUEUED |
| 6 | 1.0.103 | Reconcile debt queue and final acceptance/release/updater evidence | QUEUED |

## Stage 1 baseline and next action

Latest complete native baseline is the unchanged r18 KDE/X11 RFB grid from
2026-10-24: Firefox81PASS/103FAIL/36BLOCKED and Chrome168PASS/10FAIL/42BLOCKED,
220 cases per browser. This is not 113 independent root causes. GTK/Qt25/25
and short terminal20/22 controls do not replace either browser grid.

Read the retained Firefox per-case results and first missing production
observation/admission receipts before editing. The known witness accepts Tab,
then leaves `проверка ` unchanged after Double Shift. Default Firefox observation
was BLOCKED before input; process-local accessibility activation was used for
the full Firefox baseline. Keep those setup and functional failures separate.
Do not import Chrome's stale-lease hypothesis as a Firefox diagnosis.

Owning architecture document:
`docs/architecture/desktop-presentation-isolation-2026-10-23.md`.
Existing debt/evidence entrypoint:
`tech_debt/kde-native-receipts-followup-2026-10-24.md`.

## Administrative issue closure

On2026-10-25, at the user's explicit request, #47 was closed as `not_planned`
because reporter feedback had not arrived. This is not a new GNOME bug-fix
receipt. Closure comment:
https://github.com/radislabus-star/lay-public/issues/47#issuecomment-6402315617.
The residual remains unproved independently of the public issue state.

## Per-stage finish record

Stage1 published2026-10-25T23:33:05Z as stablev1.0.97, functional3236fe6c,
tagf53bf27a. CI40243726181/Security40243726160 SUCCESS,3512 correctness/package,
11performance,417Python (10 skipped), both MSRV1.88 and all-ten-bin release.
Final native on system IBus1.5.26: Firefox110P72F38B,Chrome170P8F42B of220
each. Exact oldPASS losses remain recorded; this is scoped publication, not
all-client or regression-free acceptance. Private IBus upgrade was rejected
after16 gains/10 losses and fresh old-version3PASS controls. Local installed
updater offers96->97, remote updater97 current; no manual local install.
See release-1.0.97-{verification,native,updater}.json. Chrome Space8 remain
stage2, shared Firefox long-sequence/rapid debt stays stage3. Later stage
reconciliation must not mark those residuals closed from this publication.

Stage1 first mechanism: Firefox closed-word native erasure after the decoder
probe loses the existing pending receipt. RED1085PASS/1FAIL of1086 under the
remote dedicated guard; exact regression and run paths are in the owning
desktop document. Minimal KDE-only observation-range candidate is under
focused GREEN verification. Native acceptance and release remain pending.

2026-10-25 continuation: A passed1086 source checks and all-ten-bin build but
FAILED its first actual Firefox Tab/second-toggle witness. No public release.
The missing no-Reset permutation was added (48 combinations). Rejected
ObservedWord and synthetic Reset alternatives are recorded in the owner doc.
Current existing-managed-commit/native-erasure/actual-matched-Reset source
passes the exact case (run-FGVInx). Next: full IME, B build/install, same native
witness, then remaining Firefox groups and unchanged full r18 matrix.

For each stage record the first failed owner/transition, alternatives and
chosen minimal repair, changed source identity, RED/GREEN identities, exact
compiler artifacts and loaded bytes, unchanged native manifest and all raw
verdicts, latency sample coverage/median/p95/max, full release gates, public
commit/tag/URL, and actual installed updater discovery. Update this file and
the owning document at each milestone; keep rejected experiments visible.

## New installer blocker —2026-10-26

Public#70 arrived during final98 native verification: obsolete apt binary
`policykit-1` has no candidate on Ubuntu26.04. The installer task records the first failure,
primary package evidence and minimal binary-package fix. Handle immediately
after current98 publication, without changing98 tested source or interrupting
its full Firefox proof. User already authorized issue fixes, releases and replies.

Stage2 final proof:3515source/package,11performance,all10optimized binaries
PASS; original frozen full native440 recorded295P66F79B including11 oldPASS
toFAIL losses. Preparation improves6 original ChromeFAIL cases but remaining
shared observation/dispatch failures stay OPEN in stage3; do not mark universal
compatibility or all browser debt closed. Publication/updater still pending.

2026-10-26T14:25Z: stage2 published98 after CI40292706780 andSecurity40292706831
SUCCESS. New P0 issue70 is an extra isolated installer release99 inserted
before original stage3. Original six groups remain; reserved later versions
shift by one (100 shared native,101 warnings,102 core,103 reconciliation).
Local installed update check currently75 because another update owns its lock;
this is WAIT, not no-update/PASS. Remote installed98 check isPASS/current.

2026-10-26T14:41Z: local installed updater now exits0/offers96->98; remote
exits0/installed98current. The earlier local busy75 receipt is preserved.
Its separate old96->97 automatic build timed out90min without installation;
measured early memory reclaim and bounded source policy repair join installer99.
No manual local binary install or IME restart was issued.

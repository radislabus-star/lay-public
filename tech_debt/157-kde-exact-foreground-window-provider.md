# TD-157 — Exact foreground-window identity on KDE

Status: IN PROGRESS; native foreground-boundary witness and source provider
implementation qualified. Current private C passes3260 source cases and the
complete KDE J grid175 PASS/233 FAIL/24 BLOCKED of432, zero NOT_TESTED.
Compared to E:27 FAIL-to-PASS, one Firefox URL3/moved_boundary PASS-to-FAIL
(root UNKNOWN),148/149 previous PASS retained. This aggregate does not close
this provider's native same-PID window/title/recreation/provider-loss lifecycle
controls, which remain NOT_TESTED. Same-byte GNOME J completes354/46/32:
all353 preceding E PASS retained, one FAIL-to-PASS, Chrome every cell unchanged.
All finite sessions CLOSED. Source/provider and installed/native acceptance
remain distinct; no full KDE acceptance or user installation/publication.
Exact current receipts and limits: owning architecture document's frozen-Space-
backend stage-complete entry, cache `kde-space-matrix-j/` and
`gnome-space-regression-j/`. The historical plan/evidence below retains its
original bytes and333-failure baseline; it is not the current grid.
Priority: P1 for KDE support, after the bounded evidence step below. This is a
separate platform capability, not a one-line relaxation of a failing guard.

## Ordered KDE repair plan — 2026-10-18

This is the entrypoint for the KDE follow-up. It does not replace TD-156's
browser owner or authorize a release. The 333 FAIL cells are observations,
not 333 independent runtime defects. A read-only inventory now verifies all
six primary receipt hashes and all 432 unique `(application, field, plan)`
identities against `kde-editors/COMBINED.json`:

| First failed actor step | FAIL | What the receipts establish |
| --- | ---: | --- |
| Clear rich field before the case | 90 | Firefox/Chrome fields 7, 8, 10 report raw `\n`, caret 0 instead of the required empty seed; intended operation not reached |
| Establish seed or return prerequisite | 6 | Five GTK/Qt prefix seeds and one GTK focus-return prerequisite fail before their Double Shift |
| Check target after actor's Double Shift | 186 | 148 ordinary/burst, 20 selection and 18 focus-return target checks fail; recognition, authorization and delivery still need causal evidence |
| Check target after actor's Space | 51 | 42 layout auto-flip and 9 moved-boundary targets fail; text, caret, selection and layout checks are conjunctive |
| **Total FAIL** | **333** | Original verdicts preserved; no new native run |

The separate 24 BLOCKED cells are 21 unavailable literal completion-panel
prerequisites, two missing second fields and one unsupported PTY selected range.
Do not infer a general Tab defect from a detector tied to one literal ending.
The actual-published-hint supplement remains a distinct protocol. The 75 PASS
cells remain controls, including clean typing, Backspace and some moved-boundary
corrections. A capability repair may expose another earlier-hidden failure;
there is no defensible forecast that it turns all 333 into PASS.

Exact inventory:
`/home/ubu/.cache/lay/development/browser-final-fixes-20261018/kde-plan-triage/TRIAGE.json`,
SHA-256 `ee84c5365b20bf25e9fdd53fcae47ed85e7a10b99e6e2c71bcf12523441a6daa`.
Its groups name the failed actor step, not the runtime root cause. It retains a
primary receipt and row index for each failure. Older KDE auxiliary bytes are
not the current GNOME/NBSP candidate; freeze a new identity before implementation.

Use this priority order, with one connected implementation at a time. TD-159
supplies each capability's relevant evidence; its complete closure is not a
global prerequisite for TD-157 or TD-158. Once qualified observations identify
the foreground boundary and its required test prerequisites are ready, begin
TD-157. Unresolved seed/Space families remain open in TD-159/TD-156 and do not
block that independent capability. Apply the same boundary-specific gate to
TD-158 after the foreground capability is available:

| Order / priority | Owner | Smallest useful result / gate |
| --- | --- | --- |
| 1 / P0 evidence | [TD-159](159-kde-native-prerequisites-and-causal-witness.md) | Distinguish raw/display/committed state and mode; obtain a bounded first-transition witness. No global text normalization or changed historical verdicts |
| 2 / P1 runtime | **TD-157, this task** | Stable KWin foreground identity reaches the existing consumer and safely recovers after provider loss; prove the capability before expecting conversion |
| 3 / P1 runtime | [TD-158](158-kde-exact-replay-layout-handoff.md) | KDE-specific activation, readback and failure restoration within the existing exact-replay transaction; text delivery unchanged |
| 4 / P1 residuals | [TD-156](156-close-browser-functional-residuals-and-verify-plasma.md) | Reclassify newly reached failures by first owner. Diagnose Space separately; do not import a GNOME single-cell rank diagnosis as a KDE explanation |

Do not wait for every GNOME quality issue to be solved before implementing the
two bounded KDE capabilities. Conversely, do not expand the provider into a
ranker or editor-ownership redesign. Each task retains its own closure gate;
KDE compatibility remains open until the changed candidate's full grid and
remaining exceptions are reported. The full original GNOME grid is a separate
regression requirement for shared runtime changes.

## Problem and verified boundary

The remote worker now has Plasma/KWin 5.24.7, KWrite/Konsole 21.12.3. The finite
Plasma Wayland baseline and editor follow-up retain75PASS/333FAIL/24BLOCKED,
zero NOT_TESTED of432 functional cells across six applications. This cumulative
grid is not one simultaneous run. Its exact receipts and qualifications are in
`docs/architecture/firefox-input-stability-2026-10-05.md`, TD-156 entries dated
2026-10-18T05:24:00Z,08:24:09Z and08:53:23Z. Do not assume all333 failures share
one cause or count prerequisite failures as reached Double Shift operations.

One separate current-candidate KWrite native letter reaches the stock Qt IBus
plugin and Lay and produces actual `п`, caret1. That observation narrows the
connection hypothesis but certifies neither manual replay nor correction; its
overall session exits ABORTED and is independently closed. Preserve that verdict.

Before TD157, production source had KDE layout switching in
`src/bin/lay_daemon/layout_kde.rs`, while exact foreground identity was GNOME-only:

- `src/bin/lay_daemon/focus_guard.rs::poll_focused_window_state_after` returns
  without polling for other backends.
- `src/bin/lay_daemon/layout_controller.rs::call_focused_window_info_once`
  rejects them with `exact focused-window identity requires GNOME`.

There are two further boundaries; adding a provider alone does not remove them:

- `focus_guard.rs::focused_window_identity_from_json` constructs GNOME-scoped
  identities and can fall back to application/window metadata. The KDE consumer
  must recognize a backend-scoped stable compositor identity explicitly. A title,
  PID, application ID or their concatenation must not become exact KDE identity.
- `layout_controller.rs::activate_target_layout_once_for_exact_replay` separately
  rejects non-GNOME backends. Its GNOME activation/readback/restore contract is
  not made portable by a working focus provider. Keep that refusal in this task;
  a KDE exact-replay layout handoff needs its own reviewed decision and task.

No test observer can supply production edit authority. A working layout switch
does not establish the target window, editor, current owner or snapshot lineage.

## Options and recommendation

1. **9/10, recommended for this stage:** implement the exact KDE foreground
   provider behind the existing focus/layout contracts, after establishing which
   supported KWin interface can supply the required stable identity and lifetime.
   Keep one admission reducer and the existing output routes. The production
   bridge must be packaged and recoverable; the test-only observer is evidence
   about feasibility, not an implementation to copy without a contract.
   KWin 5.24.7 source explicitly exposes `activeClient`, `clientActivated`,
   `clientRemoved` and the constant `Toplevel::internalId` UUID. These are
   grounded feasibility leads, not an accepted production bridge. Bind identity
   to the current compositor/provider lifetime and verify the IPC sender;
   neither a persisted file nor an observer-owned UUID grants authority.
2. **4/10:** use accessibility application identity alone. It can aid observation,
   but a PID/title cannot distinguish multiple windows or reused editors; it does
   not satisfy the existing authority contract.
3. **1/10, reject:** skip foreground identity when the backend is KDE. This turns
   missing evidence into edit permission and leaves cross-window corruption open.

The recommended option is bounded platform work. Do not ship an observer
or bypass merely to change a cell from FAIL/BLOCKED to PASS.

Version-pinned primary references, checked 2026-10-18:
[KWin workspace properties/signals](https://github.com/KDE/kwin/blob/v5.24.7/src/scripting/workspace_wrapper.h),
[KWin stable toplevel property](https://github.com/KDE/kwin/blob/v5.24.7/src/toplevel.h).
The [KDE packaging tutorial](https://develop.kde.org/docs/plasma/kwin/) describes
the supported script packaging mechanism; current documentation alone must not
be used to assume Plasma 6 API names exist on the installed Plasma 5.

## Minimum implementation route

1. Read `ARCHITECTURE.md` and the current C01–C10 owner map. Identify the exact
   existing focus identity, freshness, epoch and unavailable-state contracts.
   Record which of those KDE already implements and the first missing transition.
2. Inspect primary documentation/source for the supported KWin version(s). Prove
   identity stability across title changes, destruction/recreation, two windows
   of one process and compositor restarts. Do not invent an API/property from the
   private diagnostic script. State Wayland/X11 and version scope explicitly.
3. Add an explicit decision under `docs/architecture/decisions/` before changing
   the owning runtime contract. Define lifetime, invalidation, packaging and the
   backend-scoped stable identity consumed by the current owner. Preserve the
   existing GNOME contract; do not promote its metadata fallback to KDE authority
   or introduce a second identity owner.
   Evaluate a packaged KWin script adapter first; document how the existing
   daemon owns its observation ingress, authenticates the compositor's unique
   bus owner, handles initial focus/no active client, and invalidates the old
   lifetime on service/script loss. Reuse existing polling/freshness boundaries;
   avoid a second daemon, writable observation-file authority or per-key process
   launch. A pushed snapshot is not an exact current-window read unless its
   ordering/freshness contract is proved. If the proposed bridge cannot satisfy
   the current exact-read contract, replan before runtime edits rather than
   treating a timestamp or sequence number as permission.
4. Implement only the missing provider, its narrow consumer admission and its
   lifecycle. Missing bridge,
   ambiguous identity, restart or stale reply must remain unavailable/refused.
   Keep KDE layout readback, GNOME behavior, reducer/verifier and delivery routes.
   Account for `focus_guard.rs::focused_window_state`: its current error path
   latches `FOCUS_INFO_UNAVAILABLE`, after which it stops polling. Define which
   existing owner permits bounded recovery after bridge/compositor availability
   returns. It must refuse while unavailable, discard stale in-flight replies and
   accept only a fresh identity in the new lifetime; no permanent silent disable
   and no second retry loop/owner.
5. Add meaningful source tests first for identity/lifetime and stale/in-flight
   replies. Keep semantic assertions independent of a particular producer tag.
6. Build the frozen candidate remotely under the existing resource/Cargo guards.
   Use isolated data/runtime, preserve user sessions and verify independent
   cleanup. No experimental installation on the user's local desktop.

## Acceptance and pitfalls

Require a focused source regression for two same-process windows, window title
change, destroyed/recreated windows, stale reply after focus change, compositor
restart, provider absence and GNOME regression controls. Never infer identity
from an equal text surface, a caption or an end cursor. A window provider does
not automatically solve browser DOM editor identity; keep that limitation visible.
Include available -> provider error/restart -> safe refusal -> fresh new-lifetime
identity recovery, with a bounded retry policy owned by the existing consumer.

Then run the affected native protocols with exact source/binary/config identities,
real key input and exact client text/caret/selection. Cover KWrite, Konsole, GTK,
Qt and the ten-field Firefox/Chrome fixtures. Qualify each prerequisite before
claiming the intended operation was exercised. Include normal and supported
window modes, active/Space Double Shift, five repeated toggles, focus return,
selection, decoder/Backspace return, correction/inverse and actual published Tab
completion. Password/privacy behavior retains its own safety expectations.
Treat those operations as regression observations with their own prerequisites,
not as a promise that the provider fixes replay, ranking or Tab. Exact replay
may remain safely refused by the independent GNOME-only handoff. Prove the
foreground capability with native focus changes and the existing consumer's
observations separately from downstream text conversion.

Measure required/sample counts and median/p95 for reached operations. Software
VKMS/QPainter timing is not hardware rendering performance. PTY text alone does
not prove terminal preedit pixels; SNI registration alone does not prove the
visible tray menu. Keep visual checks and unavailable environments separate.

Preserve the old KDE baseline and original browser432 matrix. Report the new
candidate's complete grid separately; do not silently recast old failures or
combine old/new bytes. Request one isolated review scored1–10; at most two passes.
Close this provider task only after its identity/lifecycle contract, source proof,
native focus evidence and owning documentation agree, without a new regression
attributable to it. Eliminating unrelated existing cells is not its closure gate;
retain their FAIL/BLOCKED verdicts and track the independent exact-replay handoff
as a remaining capability before claiming complete KDE support. Publication and
installation remain a separate explicit step.

## Out of scope

No new text delivery route, generic replay fallback, ranker retraining, global
synchronous IBus, weakened SafetyGate or per-client literal exception. Do not
redesign all KDE integration or add a new IPC owner if the current contract can
express the missing capability. Handle unrelated prerequisite/observer failures
in their actual owners. KDE exact-replay layout activation/readback/restore is a
separate follow-up, not an implicit expansion of this provider task.

## Task-description review

Previous revision: first isolated review 8/10. It addressed the independent GNOME-only
replay handoff, backend identity admission, unavailable-state recovery and
capability-specific closure; second and final review was 10/10. Those scores
apply to that earlier provider-only description. The ordered multi-task revision
above received its own context-free review: first pass 9/10, no P1, one P2
about a possible global TD-159 dependency. The task gates now explicitly allow
capability work after its own qualified witness without waiting for unrelated
seed/Space families. The second and final context-free description review is
10/10, with the P2 resolved and no remaining substantial P1/P2 or blockers.
Current implementation has passed the full affected source gate (3238 selected,
zero failures; `run-sy49nis0/RESULT.json`) and the graph/architecture gate. Independent implementation review pass1 scored8/10, with oneP2: shared-lock
contention incorrectly invalidated healthy input. The minimal Busy distinction
and causal shared-production-lock regression pass15 focused tests; controlled
Busy-as-loss RED fails as intended. Full affected checks PASS3239/3239 (`run-g0ss9and/RESULT.json`); the
second/final independent review scored9/10, with no remaining substantive
P1/P2 findings. The reviewer reused the source and causal GREEN/RED receipts
and ran no jobs. The candidate is ready for one private changed-code GTK native
witness; this is not native acceptance. Native compatibility and all physical
lifecycle/latency gates remain unproved.

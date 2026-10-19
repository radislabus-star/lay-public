# TD-158 — KDE layout handoff for existing exact IME replay

Status: IN PROGRESS; exact-replay implementation reviewed10/10. Historical
build-B host-namespace GTK native roundtrip and following-letter/caret/selection
checks PASS, independently CLOSED,135.57/140.31ms. Current private C adds the
separate existing IME Space owner's KDE backend:3260 source PASS, final fresh
backend review9/10, actual-engine/next-letter prerequisites4/4 PASS.
Complete KDE J175 PASS/233 FAIL/24 BLOCKED, zero NOT_TESTED of432;27 FAIL-to-PASS,
one Firefox URL3/moved_boundary PASS-to-FAIL, cause UNKNOWN. This does not close
KDE no-regression or exact focus/provider lifecycle acceptance. GNOME J354/46/32
retains all353 preceding E PASS and recovers one FAIL; Chrome unchanged in
every cell. All sessions independently CLOSED. Exact current evidence/limits:
owning architecture document's frozen-Space-backend stage-complete entry,
cache `kde-space-matrix-j/` and `gnome-space-regression-j/`. No installation
or publication; preserve independent residual families and all old receipts.
Priority: P1, after TD-157's exact foreground capability. KDE plan entrypoint:
[TD-157](157-kde-exact-foreground-window-provider.md). Evidence prerequisite:
[TD-159](159-kde-native-prerequisites-and-causal-witness.md), limited to this
capability's qualified witness and verification prerequisites. Unrelated
seed/Space families may remain open; full TD-159 closure is not a dependency.

## Problem, owner and scope

Qualified witness (2026-10-18T14:12Z): private daemon55536386/IME5d84ca28,
3239 source checks. Actual GTK Entry committed seed and physical Double Shift
are present; fresh KWin identity passes and replay refuses at
`requires the GNOME layout backend`. Native actor COMPLETE, oneFAIL,22 key
events, no held keys;431 cells unselected. Outer orchestration ABORTED because
guarded Plasma shutdown required owned descendant cleanup. Independent CLOSED
has no errors, empty cgroup, VT1 and policy removed; no complete harness PASS.
Receipt: `/home/ubu/.cache/lay/development/browser-final-fixes-20261018/kde-provider-native-b/evidence/native-windows-6hswhg6h/RESULT.json`,
SHA2565ffc6d471b3e9c35e65b763e4dda38aac1f94a8701e57cde6210401c4eaf3dcd.
Actual layout tuples: `[shortName,displayName,translatedLongName]`, configured
US/RU; keyboard and KWin have the same unique owner. The new explicit decision
is `2026-10-18-kde-exact-replay-layout-handoff.json`. Existing engine/decoder
divergence remains a pre-mutation refusal of this exact fallback; owned-IME
mode changes with a fixed source remain unchanged.

In the witnessed pre-TD158 bytes,
`src/bin/lay_daemon/layout_controller.rs::activate_target_layout_once_for_exact_replay`
rejected every non-GNOME backend independently of foreground identity.
`ExactReplayLayoutHandoff::restore_initial_best_effort` also called the GNOME
activation function directly. Removing only the initial backend check can switch
the wrong stack, claim success from a process exit, or restore through the wrong
desktop after a later failure.

The existing transaction in `exact_ime_tail_replay.rs` already owns capture,
handoff, focus checks, authorization, output and settlement. Keep it the owner;
the backend supplies layout facts and performs the existing bounded handoff.
`layout_kde.rs` already reads/switches KDE layouts. Its ordinary switch path is
not automatically an exact one-shot replay proof. `ContextAdmissionReducer`,
`AuthorizedEdit`, verifier and all text output routes keep their authority.

The baseline contains 186 failed target checks following actor Double Shift,
but their common native failure point is not yet proved. This source limitation
is a required capability to remove safely, not a promise to fix all those cells.
The 51 failed Space targets are a separate family and do not use this task as
permission for a new automatic replay route.

## Options

1. **9/10, recommended:** make the existing handoff backend-aware, with narrow
   KDE activation/readback/restoration operations and existing transaction
   ownership. Reuse the working KDE query/parse code where its exact guarantees
   suffice; add only the missing guarantees.
2. **4/10:** run the ordinary KDE switch helper and assume success. It does not
   by itself bind exact focus, mapping lifetime, IBus engine/mode and restoration.
3. **1/10, reject:** delete the GNOME check or silently fall back to synthetic
   replay. Missing platform evidence must remain a refusal.

## Minimal implementation sequence

1. Read C01, C03–C07 and the TD-121 experiment log. Freeze checkout, source dirty
   archive, three binary hashes, configuration, packages, model/learning baseline,
   KWin/IBus/toolkit versions and test protocol. Record a failing first transition
   using TD-159's evidence. No new fit, literal exception or timeout increase.
2. Record a new owning-contract decision before changing runtime behavior. Define
   exact preconditions, target mapping, readback, focus/lifetime checks and failure
   effects. Keep GNOME behavior and one physical gesture/one action unchanged.
3. Use the installed version's documented KDE layout interface. In KWin 5.24.7,
   `KeyboardLayoutDBusInterface` exposes `setLayout(uint)` returning `bool`,
   `getLayout()`, `getLayoutsList()`, `layoutChanged` and `layoutListChanged`:
   [version-pinned source](https://github.com/KDE/kwin/blob/v5.24.7/src/keyboard_layout.h).
   Inspect the actual service/interface on the remote host; do not infer it from
   a successful `qdbus` process or a different Plasma version. Verify the returned
   boolean and fresh readback, and validate which configured index is RU/US.
4. Define and test the concrete relation between KDE group, selected IBus engine
   and Lay `InputMode`. Lay may retain one engine while its mode changes; do not
   assume the GNOME source-ID mapping or require an engine-name migration. An
   equal visible word or one successful keyboard indicator is insufficient.
5. Put KDE operations behind the existing handoff's backend choice. The handoff
   must retain enough validated backend/lifetime/initial-layout information to
   restore through the same backend. Do not introduce a second transaction,
   timer or retry owner. Check focus and provider lifetime at the existing
   transaction boundaries. Refuse stale or changed mappings/service ownership.
   `layout_kde.rs::LAYOUT_IDS` is currently a process-wide `OnceLock`; account for
   list changes and compositor replacement explicitly. Repair/invalidate this
   cache only as required by the selected contract, without adding a parallel
   cached layout authority.
6. Exercise failure points before and after layout activation: service absence,
   false return, bad index, readback mismatch, provider/compositor loss, focus
   change, modifier cleanup failure, rejected edit authorization and settlement
   failure. Before text execution, a refusal must not mutate text; after an
   execution attempt, preserve the existing verified partial-failure semantics
   rather than promise an unproved text rollback. Restoration is one bounded
   attempt, observes its own result and never retries an edit. Report a failed
   restoration distinctly; do not redirect it to a different newly focused
   application's per-window layout context. On a changed focus/lifetime, choose
   the safe refusal defined by the contract and record any unresolved layout.
7. Add meaningful tests through existing production adapters. Cover successful
   RU→US→RU, same-gesture no double action, unchanged engine with changed mode,
   all failure boundaries, restoration backend and stale mapping/lifetime.
   Demonstrate a regression catches the old refusal or a narrowly controlled
   wrong-backend restoration. Source proof does not establish client behavior.
8. Run the affected remote source scope and obtain one context-free review
   scored 1–10, at most two passes. Only then freeze/build the private candidate
   under resource/Cargo guards. Never install it on the user's desktop.

## Native acceptance and closure

Start with reached active-word and after-Space Double Shift in owned GTK/Qt
fields, KWrite and Konsole; verify exact text, caret, selection, keyboard decoder
and actual KDE/IBus/Lay mode after each direction. Include five consecutive
gestures, selection where supported, two same-process windows and focus return.
Missing exact focus/editor evidence remains a safe refusal. PTY output does not
prove preedit rendering, and unsupported selected ranges remain separate.

After those focused controls, run the affected ten-field Firefox/Chrome standard
including normal/private/fullscreen, active/Space and the functional plans; keep
actual published Tab and fixed-literal Tab evidence separate. Complete the
candidate's separate 432-cell KDE grid across all six applications, with exact
per-cell before/after identities and prerequisites. If shared runtime code is
changed, retain the complete affected GNOME browser regression requirement;
an unchanged baseline receipt cannot certify those changed bytes. Run required
release gates only if publication is subsequently authorized.

For each reached operation record sample/required counts, median/p95/max and
missing timing reasons. Keep successful and refused timings separate. Compare
only same-environment, same-protocol runs; VKMS/QPainter is not hardware latency.
Keep original 75/333/24 KDE and 354/46/32 GNOME counts immutable. Report newly
exposed failures; do not expand the denominator or combine old/new binaries to
claim improvement. Independence from quality ranking, DOM editor ownership and
rich-field setup remains visible.

Close when backend-aware activation/readback/restoration, source proof, review,
native reached-operation evidence and owning documentation agree, with no
new regression attributable to the change. Unrelated failures stay open in
TD-156. Full KDE support is not established merely by closing this capability.

## Out of scope

No new IME delivery route, generic replay fallback, engine renaming, new model
training, SafetyGate/verifier relaxation, global IBus restart or desktop default
change. No promise of zero failures across unsupported environments. Tray UI
and terminal preedit pixels need separate visual proof rather than assumptions
from successful IPC/PTY bytes.

## Description review

First context-free review of the coordinated TD-157/158/159 plan: 9/10, no P1,
one P2 about a possible global evidence-task dependency, now clarified across
the three descriptions. Second and final review: 10/10, previous P2 resolved,
no remaining substantial P1/P2 or blockers. These scores describe the task text,
not runtime correctness or native acceptance.

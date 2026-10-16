# TD-147 — Explicit Backspace undo and double-Space (#46)

Status: PLAN_ACCEPTED8/10; no feature production edits yet. Priority: P2.
Source baseline798fa391 plus TD-148 in progress; installed1.0.87.
Issue: https://github.com/radislabus-star/lay-public/issues/46

## Actual boundaries and first missing transitions

The existing IME exact undo is owned by SharedState.pending_auto_undo and
committed_tail::undo_last_ime_autocorrect_with_effect_progress. It validates
the recorded original/replacement and existing execution proof; its caller
finishes layout synchronization. managed::process_pressed_key clears it before
ordinary Backspace. Existing Backspace/retype restoration learning is a different
accepted behavior and must remain unchanged with the new option off.

Physical gestures belong to the daemon's existing FSM. AtomicV1 has its exclusive
detector; legacy IBus cannot acquire another detector. Merely mapping KEY_SPACE
would bypass ordinary boundary accounting. WordBuffer stores one trailing-space
bit, whereas a double-Space leaves two actual spaces. Pure-uinput cannot consume
a native Backspace that has already reached the application.

## Designs and minimal revised contract (before code)

1. **8/10, selected for review:** two separate opt-in functions. Backspace undo
   is available in IME-owned fields, defaults off, reuses only the existing exact
   auto-undo executor and its layout completion. Double-Space is literal manual
   projection of the last word after two ordinary spaces, preserving both spaces.
   Ordinary spaces keep existing invalidation of the prior undo record; use
   Backspace for immediate auto-undo. Keep default Double Shift behavior intact.
   Explicitly make double-Space last-word scope; no multi-tap scope/delay.
2. **9/10, larger:** double-Space is fully equivalent to Double Shift, retaining
   exact auto-undo priority through the extra ordinary Space. Requires a typed
   observed-suffix witness and generalized recorded BoundaryShift inverse. First
   review recommended this, but it adds state/plan migration beyond two small
   keyboard options. Reserve as stage2 if equivalence is actually required.
3. **3/10:** intercept all keyboard input for universal Backspace, or introduce
   an IBus legacy gesture detector. Rejected: changes delivery/ownership.

The smaller selected contract makes the two requested actions explicit: Backspace
undoes the latest autocorrection, while double-Space changes the visible word's
keyboard layout. Both typed spaces remain. UI/help must describe these effects
and Backspace's IME requirement, without implying pure-uinput Backspace support.
No silent fallback to another detector/backend or delayed asynchronous undo.

## Implementation boundary and consequences

- Add undo_on_backspace=false to Rust/GUI defaults and an IME-labeled setting.
  Before clearing pending, try exact undo on an unmodified Backspace only.
  Consume press/release once after actual handling. Reuse current C09 completion
  so layout/InputMode/next decoding agree. When no effect is proven, ordinary
  deletion proceeds. On indeterminate/partial output consume this key, retire
  the record and never perform a second deletion/backend. No retry timer.
- Keep ordinary restoration/retype learning and all existing defaults. One exact
  shortcut reversal records one existing feedback event, no duplicate manual
  retype vote. No model/weight/data migration.
- Add double-space to existing config/FSM/UI. Both Space presses pass ordinary
  boundary accounting; fire once after the second complete release. Cancel on
  interleaved keys/modifiers/repeat; count actually delivered repeated spaces
  normally. Four clean taps are two commands. Reuse existing dispatch/execution,
  force only its last-word scope and disable multi-tap aggregation for this key.
- Extend the EXISTING WordBuffer's trailing-space observation from a bit to a
  bounded count, retaining the boolean accessor for callers. Count only real
  Space events or exact observed/replayed surfaces. Rebuild the count from
  accepted replacement text; clear it on the same reset/current-word transitions.
  Exact last-word replay must include/delete/reinsert the actual suffix count.
  Do not invent spaces, add another text buffer, or imply internal multiword gaps
  are proven by this trailing counter. New double-Space uses last-word scope.
- Generalize AtomicV1's existing exclusive detector and ensure handled-release
  caching does not swallow Space releases before FSM observation. Proposal/prior
  receipts retain state authority; stale/rejected frames cannot mutate live state.
- No rank/lattice/SafetyGate change; literal manual projection never uses lexical
  correction. No additional queue/cache/owner/timer, per-letter RPC, or delayed
  ordinary Space. State stays bounded inside existing WordBuffer/FSM/memento.
  Hot-path cost is setting/key checks plus constant-sized suffix accounting.
  Metadata-only defaults are backwards compatible; unknown triggers in old
  daemons are not compatible, so install matching daemon/IME/panel bytes together.
  Rollback by disabling settings or restoring exact binaries, without erasing
  user learning. Existing delivery routes and global IBus are preserved.

## TDD, review and native completion

Real process_key_event regression: exact applied correction then Backspace must
restore original+boundary once, correct mode and no duplicate learning. Before
production changes this must fail. Next Backspace is ordinary. Cover default off,
all modifiers, stale/focus/caret/selection/private/password/expired/missing proof,
single-token and BoundaryShift inverse, atomic prior-receipt rejection, terminal
output and partial/error-before-effect behavior. Preserve old restoration tests.

Actual FSM/WordBuffer/executor tests: one/two/four taps, held repeat, interleaving,
modifiers, ordinary space accounting, zero/one/two/multiple trailing spaces,
replay/undo/reset/backspace, existing DS/scope behavior, exact preceding text,
and no late first-Space autocorrection after a manual command. Do not use a second
simulation to establish these effects. Prove controlled violations are detected.

Remote-only affected/full source gates and independent code review (max2 passes),
canonical graph, matched build, then remote native ten-field Firefox/Chrome with
normal/private/fullscreen and active/after-Space, rapid gestures/next decoder and
timings. Keep historical432 separate from added new-trigger cases, all non-PASS
and missing timings explicit. No universal native success before actual receipts.
Commit/push/reply after validated implementation; final local installation only
with the existing reversible matching-artifact transaction.

First read-only plan review:9/10 for the larger equivalence design,
`/root/issue46_feature_plan_review`. This revised smaller contract requires one
final source-plan review, not another round of exploratory implementations.

Final plan review2/2:8/10, `/root/late_managed_release_plan_review_20261013`;
the historical thread was reused after blank-thread creation hit the limit.
The following source-inspected conditions are part of the accepted design:

1. `committed_tail` invalid inverse must return zero-effect rejection, never
   `Some(false)` (which also denotes a successful English target). Preserve the
   actual effect-progress receipt through Backspace/C09 completion. Retire a
   partially executed/indeterminate undo and consume its key without retry.
2. Double-Space's explicit literal intent must bypass BOTH pending-auto-undo
   priority and remembered multiword replay scope. Existing Double Shift keeps
   both current priorities. `replace_words=1` alone does not establish this.
3. Keep Space's separate ordinary boundary path, including Space3 after a
   projection; do not clear its retained buffer via the typing-key helper.
   Four taps must preserve four spaces and produce two commands. Repeat2
   cancels the gesture until release, still counts its delivered space/pending
   offset, and cannot repeat correction or learning.
4. AtomicV1 observes Space release before handled-release cache and transfers
   gesture state through speculation/memento/commit_native_observation. Its
   physical `shift_active` continues representing Shift only. Rejected/stale
   observations must not advance live state.
5. Use a checked u16 ASCII-space count in the existing WordBuffer. Overflow
   invalidates replay authority; never saturate then delete an inexact suffix.
   Replayed/observed constructors derive the real count. Mixed whitespace must
   be represented exactly by the current contract or rejected. This does not
   certify internal multiword gaps; the new command is last-word only.

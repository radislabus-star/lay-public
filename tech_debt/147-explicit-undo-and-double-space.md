# TD-147 — Explicit Backspace undo and double-Space (#46)

Status: DONE_SOURCE3209/3209, review7→9/10; consolidated release native gate/install pending. Priority: P2.
Source baseline05e1e448 (1.0.88 source); installed1.0.87 from50d314b3.
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

Additional source-inspected boundary before code: physical_input_grab already owns queued manual gestures and is hardcoded to LeftShift. Parameterize that existing observer for configured Space: forward/count each actual Space once before its complete-release gesture, including repeats, and never synthesize an extra Shift pair for Space. Default queued Shift behavior stays unchanged. New Backspace completion must use the existing Caller/IME layout-postcondition contract, avoiding a second deferred layout operation.

## Genuine RED and first implementation proof

Remote td147-before-code-b88x7s0q/RESULT.json:6 selected,6 genuine failures (3 WordBuffer suffix/refusal,2 real IME key callbacks,1 configured key). Both IME callbacks fail handled=true assertions at the actual Backspace event, not during fixture preparation. No missing-field compile failure: the option is deserialized, which the baseline ignores. After Backspace/counter/mapping foundation, td147-foundation-8rwe8fn9/RESULT.json is6/6 PASS. This first result does not prove physical Space dispatch, Atomic handoff, error/privacy cases or native behavior. Subsequent implementation adds the existing FSM observation before ordinary Space processing and fires only after the complete second release; repeat accounting has no correction/learning.

## First Atomic completed-word owner loss (before repair)

Source path: admitted Atomic key frame -> existing gesture -> manual_toggle_active_text_target -> ImeCommittedTail -> legacy exact-tail delegation. Atomic ownership rejects that external RPC, so a completed word cannot project in the already-owned frame. This also affects the existing Atomic Double Shift after Space; it is not a browser exception. Options9/10: use the existing authorized Atomic replacement executor within that same admitted frame;7/10: leave the option unavailable in Atomic clients;2/10: legacy RPC fallback (rejected). Select9/10, require genuine source RED before the small shared selector repair. Legacy RPC/stock transport routes, verifier and safety gates stay unchanged. Decision2026-10-16-atomic-manual-tail-frame-owner.json records the explicit scope. Plan reviews remain capped at2; implementation review must assess this shared owner repair.

The first partial-output test failed because TestEngineOutput records a commit attempt before injecting its ambiguous failure. Correct its assertion to exactly one attempted commit; retain one delete, consumed key, retired authority, no retry and no second deletion. Do not claim the attempt was accepted client text.

Extended source pack td147-frame-owner-green-tkyox27w:14/15, including the actual four-Space Atomic key/receipt stream with2 projections and4 retained spaces. A separate direct Atomic helper fixture still lacked real activation/frame history; initialize it through actual ProcessAtomicKeyEvent +accepted prior receipts rather than weakening edit admission. The direct pre-repair RED alone is insufficient to locate authority loss; add a controlled selector reversion against the actual complete Space stream.

Queued input boundary: pure-uinput scoped replay currently holds the physical grab but does not run the exact-tail branch queued gesture callback. New Space must drain that same retained grab after the first successful replay and reuse the current manual executor for each queued complete pair before later typing. No deferred command queue or second output route. Preserve its partially observed pair in the existing dshift_state after completion.

The direct helper fixture also seeded English text while declaring the current layout Russian. The existing literal projection correctly rejects that no-op; match the fixture's English layout to its actual English text. This is a fixture correction, not a verifier or runtime relaxation. Only a controlled selector reversion against the complete key/receipt stream can establish the shared owner regression.

Remote td147-final-owner-emhwioq5/RESULT.json:15/15 source PASS. Controlled restoration of only the old shared selector then causes both the admitted completed-tail helper and actual four-Space key/receipt stream to fail (2/2 detected); original bytes restored afterward. This confirms the owner repair independently of the earlier invalid fixture. Extend the Backspace rejection test with focus change, password/PIN, transition to private content and missing receipt. Native acceptance and latency remain unmeasured for these bytes.

Strict lint td147-lint-br9i3il0 stops on two pre-existing unused serial assignments after actual lifecycle operations in a replay-progress test. Remove only those dead assignments; preserve both lifecycle calls, serial values passed to them, assertions and all manifest IDs. No warning suppression or baseline change.

After those assignments, the strict inventory also exposes an older retired callback wrapper and one test-only wrapper still compiled in production. Remove the completely unreferenced Atomic wrapper and compile the three-test legacy convenience wrapper only under cfg(test); production continues through the same observed callback owner. Four existing library warnings differ only in recorded source offsets from prior work. Refresh those inventory locations only if semantic warning identities and multiplicities remain exactly equal; no new allowed warning or suppression. The earlier no-baseline-change expectation is superseded only for this location refresh, not for the warning contract.

Strict Clippy then identifies a real MSRV prerequisite: the existing File::lock call requires Rust1.89 while Cargo/CI promise1.88. Preserve the same independent lock-file transaction with the existing libc flock, retry interruption, keep descriptor ownership until drop and fail closed on other errors. A real independent-descriptor exclusion/release test covers the contract. Move the TD148 test module to the end of its file for the unchanged items-after-test-module rule; all existing test IDs and assertions stay unchanged. These are release prerequisites, not shortcut transport changes.

Implementation review1/2:7/10, /root/confirmed_second_handoff_plan_review_20261013 (reused independent thread, root-only implementation). P1: queued emit failure retained authority; repair tracks delivery validity only within the existing drain, retires mirror/cancels gesture on failure, disables remaining queued replays, keeps forwarding physical input once and retires the final mirror. P2: Atomic checked the gesture deadline at release while daemon checks second press; determine Atomic eligibility at second press and still fire only on complete release. Added actual accounting/observer tests for either failed Space followed by later successful taps and delayed second release/repeat. Fresh complete gates and final review2/2 required.

Other strict lint prerequisites preserve existing behavior: remove Copy-only test clones, combine identical test branches, group existing gesture/handoff arguments, box the same owned callback payload (one small per-callback allocation; subject to native timing). No warning suppression, semantic baseline allowance increase or safety relaxation.

First complete source attempt td147-full-source-0iq_m42g:3207/3209,2 genuine reported test failures; do not label this receipt PASS. New stale-focus negative fixture changed only a counter, not the existing field lifecycle; exercise actual bind_focus_receipt first-field/new-field and assert retirement. The deliberately wrong action-kind fixture was placed in src and correctly rejected by the unchanged runtime mint-owner guard; keep this test fixture in tests/common, compiled only by cfg(test) IME tests. No runtime guard allowlist, admission or verifier relaxation. All other source identities, strict default/research lint530/354, canon15 and harness122 (one explicitly skipped) passed. Semantic warning identities/cardinalities remain equal; only positions refreshed.

Repaired source pack td147-focused-final-n5gdbyuh:34/34 PASS, including all16 unchanged text mutation monopoly guards, all18 shortcut/suffix/lock cases and both new review counterexamples. Controlled old Atomic selector still fails2/2 as expected; restored bytes verified. The engine fixture already has its first focus receipt; replacing it through bind_focus_receipt performs actual field retirement. No runtime edits were needed for these two fixture repairs. First full receipt remains BLOCKED_CONTRACT (2 unexpected failures), retained unchanged; a fresh complete run is required.

## Final source result and remaining release boundary

Fresh normal complete source receipt /home/ubu/.cache/lay/development/td147-full-final-0_9ucici/RESULT.json and SUMMARY.json:3209/3209 (3173 correctness+36 package),33 executed targets,0 semantic/infra errors and zero failure exceptions; manifest3235 includes all original identities,18 additions and unchanged15 ignored/11 performance identities. Strict pinned default/research lint PASS, semantic inventories530/354 unchanged, canon15 and harness122 (one explicitly skipped) PASS; changed panel syntax PASS. Frozen archivece93727d2cd6b6c3ff29191e287832ed48182c3385d3030f1a3df0d06f2d203a; raw/final formatted source hashes retained. The failed first full receipt remains BLOCKED_CONTRACT and is not rewritten.

Final independent code review2/2:9/10, /root/confirmed_second_handoff_plan_review_20261013, reused independent read-only thread because new-context threads were unavailable. No remaining inspected P1/P2; reviewed8 key-source/helper hashes match the34-case proof and warning semantic identities are unchanged. The final full result arrived after that review; native acceptance/latency were not part of its verdict. Root implemented every change.

Publish this source phase before TD-149's one-package security patch. The final matching1.0.88 build shares one remote default432-cell browser gate, then separate optional shortcut cases and timing. Source DONE does not imply native completion. Installed user1.0.87, global IBus, delivery routes, config and models remain unchanged at this milestone. Callback boxing adds one small owned-callback allocation, so final native timing is required.

Final canonical AST refresh /home/ubu/.cache/lay/development/td147-final-canonical-6km3pybt/RESULT.json:PASS, exact decision coverage and15 canon tests,48.4s. All functional inputs match the3209-test proof; only generated architecture metadata/documentation differ. Installed authority unchanged.


## Final1.0.88 normal-window option evidence

Whole80 COMPLETE: Firefox25PASS/12FAIL/3BLOCKED_PRECONDITION, Chrome33/6/1,
total58/18/4; each40distinct rows/5jobs,0collectorerrors, allprocessesgone/
noheldkeys/VT1. Whole receipts: td151-install88-20261016/native-shortcuts/.
Byte-equivalence binds these original e34-labeled receipts to final77af core
and panel payloads; source labels are not rewritten. Native opt-in configuration
SHA48f6e88fa12da65d5f7acf3e352feb076eeab6a73b4848bf07f3084f142b35ba.
DoubleSpace40:26PASS/14FAIL; Backspace40:32PASS/4FAIL/4blockedpreconditions.
Failures may precede the action or occur at the next-letter/ordinary-Backspace
probe; these are complete-scenario statuses, not14 proven detector failures.
The32successful undo scenarios include ordinary deletion. Three Firefox failures
occur at the actual backspace_undo target (fields7/8/10), one at preceding Space
(field9); four other scenarios lack seed preconditions. Successful timing samples
must not hide these failed/missing cases. Private/fullscreen options remain NOT_TESTED.

DoubleSpace measured36/60FF med117.086ms/p95144.668,48/60Chrome99.855/p95119.052;
missing24/12 explicit. Backspace undo13/20FF med20.749/p9525.141,19/20Chrome
21.027/p9524.334; missing7/1 explicit. First-Space and ordinary-Backspace coverage
are stored separately in the full receipts. Options remain off/defaultDoubleShift.

Observed comparison with older E6 whole80: Chrome no changedPASS; Firefox lost
3PASS (search2 rapid,iframe9 singlepair,iframe9 second-wordSpace/undo), gained
iframe9 rapid, net-2. These outcomes and original negative raw receipts remain,
not a causal runtime-regression proof and not waived as PASS. Compatibility
follow-up remains under TD121: group by first failed transition through actual
Space/ACK/manual intent/context admission/verifier/output; no literal-word fix,
route bypass, relaxed oracle, repeated same-byte full run or forced authority.
Current source/option implementation is complete within exact current-context
contract; universal native acceptance is explicitly NOT established.


Final release closure —2026-10-16: functional source77af7ade, full3211/3211,
CI38916364078 +Security38916364185 firstattemptSUCCESS, independent review9/10.
Installed matching CLI/daemon/IME/panel1.0.88;6actualpayloads match native-tested
e34 bytes. Whole432=349P50F33B and normal optional80=58P18F4blockedpreconditions
remain explicit, no all-client acceptance claim. GlobalIBus/L1.1/L3, models,
settings/sources/mode unchanged; no localfunctionalkeys. Rollback/procSHA evidence:
/home/ubu/.cache/lay/development/td153-install88-20261016/INSTALL.json.
All old negative receipts and each card's narrower source/ACL/ARM scopes remain.

# Fragment native result across its own mode change — scope proposal

Status: ACCEPTED_USER_REQUEST on 2026-11-09. The user explicitly approved only
native_document.rs and native_document/tests.rs in reply to
call_bd15088f4ce44e2ea69c3d76d5e10589. The scope and owning ADR are recorded in
active-field-integration-scope.json and
../architecture/decisions/2026-11-09-fragment-native-mode-continuation.json.
Implementation and mandatory source/native acceptance are still pending.
This permission does not approve any other protected path or incomplete release.

## Proven current failure

Accepted checkout: /home/ubu/projects/lay-dependency-updates-20261027,
codex/browser-matrix-repair-20261027, base4fae57d50b031984c1e0ddc8796e78e142b2a595.
Latest frozen v20: a51cf303799c8ed7c90ece77bd87fde4c38a77c28b5a3d1b2e3301dc79542624.
It is diagnostic only, not installed or promotable. Both desktops retain123.

Reproducer: actual KDE Chrome text field, physical XTest codes for ghbdtn,
12ms press/12ms release, DOM observation only, no injected text. The detector
reaches all six letters and the existing native range/CommitText writer
dispatches ghbd -> прив. The current caller then waits for the result before
completing its new mode. Both subsequent actual source reads still report
native_selection_present. The existing transient input gate says allowed=true
and all field/focus/lease/layout checks remain current. The caller eventually
reports partial failure and closes its word; later native input stays in the
source mode, giving привtn. No assertion, timing or input cadence was relaxed.

Exact retained evidence on the remote worker:

- letter-fragment-native-automatic-frame-chrome-first-20261109/RESULT.json
- same directory ime-trace.jsonl and case/result.json
- same directory analysis-current-result/report.json
- build letter-fragment-native-automatic-frame-v20-20261109/RESULT.json

All paths are under /home/e/projects/lay-development-runner/. The actor receipt
alone has provenance limitations; the controller independently pins the same
candidate PID/hash/config through the run and verifies restoration of installed
123, IBus PID, original KDE map/group and unchanged live config.

Earlier distinct new-caller errors are separately fixed, not reasons to change
shared guards: duplicate tail publications incremented epoch9 to11; the inner
hook missed the canonical early inert release; an unnecessary quiet-input
bridge scope refused when later keys were pending. v20 has none of those three
errors. The remaining native selection is an asynchronous delivery stage.

## Existing contract and dependency map

```text
ibus_interface.rs::process_key_event
  -> unchanged WindowInteraction::process_legacy_key
       window_interaction/observation.rs:6167
       (its authenticated early inert release stays inert)
  -> fragment_layout.rs::request_fragment_after_letter_release
       current pair/focus/admission -> on_letter -> DecisionCore
  -> shift.rs::request_word_layout_flip [typed Fragment origin]
  -> committed_tail.rs::toggle_committed_tail_target_from
  -> unchanged state.rs::replace_committed_tail_with_effect_progress
       native exact selection + CommitText
       publish original epoch + original pending result
  -> fragment_layout.rs::complete_fragment_word_mode
       approved layout_sync.rs::sync_layout_for_fragment_word_mode
         -> existing native InputMode / existing KDE group transaction
```

Native result ownership is carried by
kde/native_document.rs::NativeDocumentBinding. Its is_current checks field,
token, focus, lease, layout generation and tail epoch. Existing for_result
copies the same binding, drops geometry and binds only result epoch/generation.
Existing observer::arm manages the subscription, and native_observation_failed
already permits intermediate selection for the SAME pending current result.
The common actual-result verifier remains the only acknowledgement owner.

Forward mode is already separate from actual text acknowledgement in state.rs:
an authorized exact autocorrect may select the next-key mode while keeping
outcome evidence pending. Do not turn a dispatched text into confirmed evidence.

## Recommended bounded change — 9/10

Allow only these TWO additional protected files:

1. src/bin/lay_ibus_engine/kde/native_document.rs
2. src/bin/lay_ibus_engine/kde/native_document/tests.rs

The already authorized fragment_layout.rs calls the new narrow native owner
entry. Its detector, typed request and existing verified writer remain in place.
layout_sync.rs keeps ONLY its already approved separate entry; no old body or
branch changes. No GNOME file, daemon, renderer, key decoder, state.rs,
composition_commit.rs, admission reducer/adapter or output implementation changes.

Add a capture/finish operation for this SAME authorized mode transition:

- Before mode publication, capture only an already current pending native
  result with an exact expected snapshot, its original receipt identity/epoch,
  the same current native transport owner and previous layout generation.
  The ticket is local to this call, not a new persistent owner/cache/queue.
- Perform the existing authorized InputMode plus KDE group operation. Reuse
  its live layout-intent token and completion result. Do not run another switch.
- Only after verified success and decoder promotion, revalidate the same
  current intent, context, field, focus, lease, original pending identity and
  epoch. Reject replacements, newer input/result ownership or changed fields.
- Reuse existing for_result to carry the SAME result/transport across only this
  operation's known layout-generation change. All other coordinates stay bound;
  expected text, immutable snapshot identity, original dispatch time, feedback
  and word provenance remain unchanged. No KnownStart promotion.
- Discard old geometry and use existing observer::arm for the same field.
  A retained subscription grants no edit proof or delivery acknowledgement.
- Remove the new caller's unsuccessful synchronous post-dispatch wait. Actual
  result confirmation, mismatch, selection, expiry and invalidation still go
  through the original observer/verifier. Source observation before proposing
  an edit remains bounded and independent.
- No retry of text output or alternate writer. Failure after dispatch retains
  the existing partial/no-retry contract; stale tickets never restore a right.

The implementation must NOT just ignore layout generation or overwrite stale
metadata. Validity must be established before the caller's own mode operation
and independently rechecked after it. If existing owner APIs cannot express
that proof, stop for another decision; do not expand the scope silently.

## Alternatives

- Deferred Fragment mode metadata in the existing pending result: 6/10.
  Requires shared receipt-type/constructor changes and native confirmation
  continuation, plus proof for a following letter arriving before that result.
  Broader than the recommended existing-binding continuation.
- Longer waits in the key callback: 2/10. Current actual reads already show
  intermediate selection while later input queues. This increases latency and
  has no proved completion guarantee; no repeated sleeps to obtain a PASS.
- Reconfigure Chrome to own the entire word as preedit or add another writer:
  rejected for this task. It changes protected delivery/display contracts.

## Required source proof and acceptance

New tests in the native owner's existing fixture family:

- Valid current original receipt, authorized own-mode change: binding survives
  only its mode generation; actual document is still unconfirmed, no text or
  selection output, no fabricated geometry or word authority.
- Focus, field, bus owner, lease, context token/revocation, epoch, pending receipt,
  selection provenance, unexpected layout change and failed mode: no carry.
- A valid carry does not confirm a result. A later exact actual unselected
  snapshot confirms through the existing verifier; mismatch/expiry never does.
- No pending native result, optionOFF, GNOME and ordinary Space/manual origins:
  no new continuation. Only the new Fragment operation and its exact manual
  return call the new entry.
- Keep all existing native observer, scope, ownership and source guards intact.

Native: first rerun the fixed-speed saved Chrome first-word case; same-binary
short10 (both browsers/directions/positions/OFF), then ordinary Space correction
with optionON, Tab, held Shift, five rapid Double Shift operations, native fields
and terminal controls. Record live bytes, actual text/mode, original restoration
and both end-to-end and internal timings. Run full current source/build/release
gates only after short native controls are green. Supported45+ GNOME native
acceptance remains a separate required gate. No publication with pending checks.

One current source result is also unresolved: the full IME run was1162/1163PASS
with a process-isolated level-modifier/Space-lease failure; the same artifact's
isolated test passed. Retain both; no waiver or test weakening is proposed.

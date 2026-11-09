# Preserve typed owned text when a completion arrives late

Status: ACCEPTED_USER_REQUEST, 2026-11-09. Explicit approval of both files in
call_009a4484e97a47958ab01aed4f686cac, followed by authority for necessary fixes
and all tests without repeated questions. Exact current scope/ADR is recorded
before code. This does not waive isolation, source or native acceptance gates.

## Evidence and first failed transition

Checkout lay-dependency-updates-20261027, base4fae57d50b031984c1e0ddc8796e78e142b2a595.
Diagnosticv24 SHA5b74e46d6dd3f47f2d9266f396a7a32596f8a7ef43e7501c4c780a8053e45464.
Same-binary short10: Chrome5PASS; FirefoxEN-RUfirstPASS, RU-ENfirstFAIL;
three unexecuted cells. Exact controller/actor/trace under
/home/e/projects/lay-development-runner/letter-fragment-native-mode-short10-v24-20261109/.
In firefox-letter-ru-en-first, actual hello is observed beforeSpace; then
ibus_preedit clear at tail_epoch10/composition_chars5, no input callback.
The following worker log is late generation12 with display_age211655us.
Only afterwards do the four Shift key callbacks and manual RPC occur; RPC
correctly refuses context_authority because preedit_visible is now false.
The runtime stayed on the same candidatePID/hash/privateconfig. Original123,
IBusPID/config and KDE map/group restoredPASS. Saved analyzer retains eight
actor-level provenance warnings; controller identity checks and correlated
IME/DOM ordering are the scoped evidence, not analyzer UNKNOWN alone.

Source dependency:
precognition_worker.rs::apply_completed (freshness rejection)
 -> preedit.rs::retire_late_precognition (1590)
 -> clear_preedit (always, even composition.buffer nonempty)
 -> visible owned letters disappear
 -> window_interaction/authority.rs manual guard correctly refuses.
The worker, preedit and admission bodies are byte-identical to publicv1.0.123.
Do not weaken manual guard or cancel normal predictions in the fragment caller.

## Minimal proposed fix (9/10)

ONLY two additional files:
- src/bin/lay_ibus_engine/preedit.rs
- src/bin/lay_ibus_engine/preedit/tests.rs

Change only retire_late_precognition to call its existing sibling:

```rust
pub(crate) async fn retire_late_precognition(
    &mut self,
    emitter: &mut EngineOutput<'_, '_>,
) -> fdo::Result<bool> {
    self.retire_pending_precognition(emitter).await
}
```

That existing operation already handles both cases: empty composition hides
only stale completion; nonempty composition clears predicted suffix/candidates
and republishes the same typed buffer at its existing cursor. It does not
CommitText, delete surrounding text, change a token, admit a word or change
layout. It preserves the existing worker cancellation and pending=false guard.
No worker, ownership/admission/verifier, text writer or desktop adapter changes.
This is explicitly SHARED display behavior (KDE and GNOME), not a Chrome-only fix.

Tests: retain existing empty-composition/idempotence test; add controlled
nonempty owned-text cases in the existing test family for both backends,
empty/nonempty suffix, original caret, no commits/deletes, no tail/epoch/layout/
admission promotion, and publication failure without retrying text. First run
new regression RED against existing method, then GREEN with sibling reuse.
Do not change existing expectations to hide a failure. Same saved native
Firefox RU-ENfirst must pass including both manual returns; then short10 on the
same frozen binary, ordinary Space/Tab/heldShift/rapid/manual/terminal controls,
full current source/build/release gates and supportedGNOME native acceptance.
No version publication with mandatory gates pending.

## Rejected alternatives

- Cancel background predictions in every Fragment callback (3/10): would make
an optional detector interfere with otherwise working completion behavior.
- Increase freshness timeout or warm/retry untilPASS (2/10): hides the same
late cleanup defect; never proves preserved typed letters.
- Bypass manual authority check (1/10): admits an invisible/stale word and does
not correct the preceding unwanted clear.

Rollback removes only sibling reuse and new tests, retains original receipts
and does not mutate installed123. Source change authorized; source/native verification still required.

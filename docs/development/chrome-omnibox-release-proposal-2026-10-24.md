# KDE Chrome address-bar caret: bounded release experiment

Status: proposed; not implemented or installed; common-hook scope approval
is still pending. Current retained private1.0.94.1 is2100/PID3349193 after
rejection of the separate8a66 Space trial. This proposal does not certify a fix.
A fresh8a66 upper-address-bar probe independently found both own endings ерка
but rect:null at prefix пров and проверка пров:2hintFAIL,TabBLOCKED.
Receipt:chrome-941-keytime-space-omnibox-20261024/result.json. Config/keys/
owned window restored;actual Chrome alive. These are outside full220 totals.

## Measured failure

On Chrome155.0.8059.84 with GTK3/im-ibus1.5.26, the actual upper address-bar
field contains the fixture prefix with caret4 and no selection. AT-SPI returns
zero rectangles for character3, character4 and empty range4..4, and -1 from
GetOffsetAtPoint at three positions. Default text attributes provide colours
and direction, without a font or glyph metrics. Exact caret geometry cannot
be reconstructed from these measured native responses.

The IBus trace has only the pre-fourth-letter cursor position. Registry
subscription completes in351/595us and text events reach actual-document
reads. Word-scope observed_suffix_chars3 is lineage history, NOT a document
length; the earlier interpretation of that counter is withdrawn.

Receipts:remote control root/chrome-941-omnibox-{observer-trace,geometry,native-capabilities}-20261024.

## Proposed minimum experiment:8/10, native outcome not yet known

Let the GTK3 client receive its original matched alphabetic key release after
Lay has committed that character. Reuse the existing handled-press ledger and
canonical release-completion fact; no generated key, timer, second detector or
new text executor. The ordinary letter still uses the same CommitText route.

The eligibility policy and presentation invalidation belong in the KDE native
input adapter. Require KDE, actual Chrome native binding/current notification
owner, managed ordinary letter, matching committed-key floor, current focus,
lease/layout and live admission, and no command modifier/sensitive/atomic or
owned composition. Keep Shift/Alt/Ctrl, navigation, Backspace, Tab, Space and
non-Chrome clients on the existing path. Never renew an edit receipt.

Before delegating that real release, clear its old display coordinates. Reuse
normal SetCursorLocation and the existing focus/activation geometry guard.
Allow the normal guarded IBus anchor only when the complete current native
word is verified and native character geometry is unavailable. A stale native
word/geometry must continue to suppress display. Do not reuse the old position.

Expected mechanism is a hypothesis to verify:GTK3 async IBus replays an
unhandled original event to its owner, and Chrome's GTK context sets cursor
location during dispatch. Test that the resulting fresh report follows the
last letter before considering the hypothesis accepted.

## Required common call sites

The implementation stays in KDE. Two existing dispatchers must call it:

1. `src/bin/lay_ibus_engine/window_interaction/observation.rs` at the canonical
   `record_text_inert_release_completion` managed-release branch (around5280).
   Consume/record the original release once, retain completion/prediction
   settlement, but return false to the client for the qualified KDE case.
   The internal handled fact must remain separate from the returned delivery
   flag so display settlement is not skipped.
2. `src/bin/lay_ibus_engine/ibus_interface.rs` at the legacy paired
   `consume_handled_release` branch (around867), returning false for the same
   qualified KDE policy. This keeps canonical and legacy entrypoints aligned.

GNOME policy returns the previous handled value and performs no state change.
No common correction, preedit style, verifier or GNOME integration change.
These two call sites are outside the user's requested KDE-only file boundary;
obtain explicit clarification before applying them.

## Required proof before keeping the experiment

- TDD matched letter release exactly once; unchanged token/scope/text/correction
  authority, no extra commit/delete/forwarded or generated key.
- Refuse wrong key/floor/focus/lease/layout/activation, modifiers/command,
  privacy, atomic operation, other application and GNOME. Test uppercase with
  a held Shift and overlapping letter releases.
- Clear the old rectangle; accept only a new same-activation cursor report.
  Keep native geometry preferred when available, stale native evidence hidden.
- On identified installed bytes:upper first word accepted by Tab, second word
  in the same field, Tab, exact text, white suffix-only visual frame, timings.
- Same frozen220 page-field proof and uninterrupted/five-gesture controls.
  No acceptance if aggregate proof regresses. Retain all original failures and
  blocked cases; do not alter actor cadence or add an extra diagnostic key.

## Other options

- Reuse the last IBus rectangle without fresh evidence:2/10,rejected because it
  can cover the last typed character.
- Estimate font/glyph widths or find a caret in screenshots:2/10,rejected for
  complexity and unreliable placement.
- Fix and distribute the Chrome GTK/native-accessibility implementation:5/10
  for this immediate task; a separate upstream/client project if needed.

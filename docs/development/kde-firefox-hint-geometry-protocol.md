# KDE Firefox hint geometry: retained native acceptance protocol

This protocol supplements `tests/manual/browser_ime_standard.md` for the
KDE/Firefox display adapter. It certifies this display case, not a complete
browser matrix, another desktop, another client or every document size.

## Preparation and identity

1. Identify the canonical checkout, scoped source diff, compiled candidate and
   actual loaded IME SHA. Retain source and runtime receipts separately.
2. Use the KDE-only native Firefox launcher with the existing compatibility
   library and process-scoped `GNOME_ACCESSIBILITY=1`. Preserve the user's
   browser/profile and global session settings. Verify real process, library
   mapping, GTK IBus, absent synchronous mode and the KDE desktop selector.
3. Use an owned browser fixture/profile on the remote KDE desktop. Activate its
   actual window before native keyboard/pointer input. DOM inspection may
   observe/setup the viewport; it must not insert the tested text or fake keys.
4. Use the standard ten-field page. Pin actual Firefox PID, URL/field identity,
   editable focused native node, engine PID/inode, source and layout around
   input. Release held keys and restore the prior engine/layout/config afterward.

## Display case in every field

In each ordinary, private and fullscreen window:

1. Type the first word incrementally: `п`, `пр`, `при`. At each step record
   actual native text, caret and selection; the last character's screen bounds;
   fresh Lay hint stream/revision/text length and rectangle; and key-release to
   first correct display observation. Do not mistake DOM text presence for
   proof that an opaque hint window is not covering it.
2. Save a real screenshot before accepting a completion. The typed prefix must
   remain visible; the suffix must start at the actual character end.
3. Finish `привет` manually, insert one Space, then type the second word as
   `п`, `пр`, `при`. Repeat all three geometry measurements and save a second
   screenshot before Tab. One correct first-word sample cannot certify this.
4. Press Tab and verify the complete actual text `привет привет ` with caret
   at its end and no selection. Keep actual rich-text NBSP representation in
   evidence; normalize it only through the existing test observation contract.
5. Run two consecutive Double Shift gestures without intermediate input:
   `привет привет ` -> `привет ghbdtn ` -> `привет привет `. Verify the next
   physical letter/Backspace control. Report this independently from geometry
   and Tab: a passing display does not turn a refused edit into PASS.
6. In a separate prepared row, type both full words at the retained native
   actor rate (12ms key down, 12ms inter-key gap). Verify whole text, caret,
   selection and stability. Record input duration and post-Space latency.

HTML URL/email controls strip trailing ASCII Space. Record their three real
first-word geometry samples, then BLOCK the two-word exact-Space step before
that unsupported preparation. Do not alter field types, fabricate a separator,
weaken the verifier or call those unexecuted steps PASS.

## Readiness and safety

Firefox can temporarily report CharacterCount0/CaretOffset-1 after a managed
commit. The strictly checked native reader must continue rejecting that shape.
The Firefox display adapter may resample transitional observations within its
one original80ms observation deadline, following the existing Chrome reader's
deadline/yield pattern. Privacy, selection, focus-state and transport errors
stop immediately. Never infer coordinates from font width, reuse an unproven
composition-start anchor, extend the deadline or install edit authority from
this display read.

The inherited native document reader is bounded to4096 characters. Larger
documents, other DPI scales, Wayland, USB/HID paths and other clients/desktops
need separate physical evidence; a short X11/RFB fixture does not certify them.

## Evidence and counts

Retain all row outcomes and actual reasons. Distinguish prefix-coordinate
points, complete field scenarios, Tab, fast typing and Double Shift. Preserve
failed/unreached gesture denominators; latency statistics for successful
transitions must not conceal failures. Store counts, SHA, sample sizes, median,
p95, maximum, screenshots and exact receipt paths together.

For a render-only fix, if a control fails, compare identical controls on retained
reference and candidate bytes before assigning regression. Restore the current
candidate in a finally path and verify loaded bytes and protected processes.
Never turn delivery of a job, a source test or an attempted browser launch into
native PASS. If repeated work stops yielding new evidence/progress, call the
user with the concrete blocker and one short manual reproduction.

The2026-10-29 report records162/162 exact coordinate points,24/24 Tab and24/24
fast-input cases in three window modes. Its separate post-Tab Double Shift
failures in rich/plain/iframe-rich remain open; the normal-window old/current
byte comparison reproduces them on both binaries. The full historical440-cell
KDE matrix is not recounted by these diagnostic rows.

# Lay 1.0.76 — IBus startup admission and Kitty focus

After a full IBus restart, Lay could start before IBus exposed GlobalEngine.
The IME then discarded its context-admission observer for the process lifetime:
words stayed in whole-word preedit and Double Shift requests were rejected at
`bridge_admission`, even though all services were active. Lay now treats only
IBus's exact “No global engine” startup response as an unset profile, retaining
the observer without text authority until IBus confirms a Lay profile and a
fresh input context.

The accepted installed IME is SHA-256
`a8b9d1686ec61fd7014ab7d2a43731deec1a830a25b78987b03fc622135589ea`.
Kitty sometimes omits the terminal content type after IBus restarts. A bounded
GNOME focused-window probe now binds exact Kitty identity to its IBus focus
receipt and restores native terminal input in that case. In an owned Kitty
window, Tab and two Double Shift gestures completed
`просто → ghjcnj → просто`; the user then confirmed the running version works.
The receipt is
`/home/ubu/.cache/lay/development/kitty-focus-proof-20260925/kitty-tab-cycle-aligned/receipt.json`.
The guarded changed-source gate passed 2,904/2,904 selected correctness and
package checks; see
`/home/e/projects/lay-development-runner/kitty-focus-changed-v3-20260925.log`.
L1.1, L3, daemon and IBus/IME were restarted after installation; GNOME and IBus
both selected `lay-ime-ru`. The installed and loaded IME hashes matched.

The physical Firefox WhatsApp composer still has a known second-key suggestion
regression on this source lineage. The Tor Browser route without
SurroundingText also remains open. The user's existing Chrome, Firefox,
WhatsApp and KDE windows were not retested on this exact binary. These limits
and the earlier failed candidates are recorded in
[TD-121](../tech_debt/121-preserve-word-across-ime-layout-handoff.md).

The user-confirmed Firefox cycling repair from 1.0.75 remains in place.

## Earlier held-Shift candidate (2026-09-25)

Firefox also resets its IBus context after a managed `CommitText` while Shift
can remain physically held. The Reset route now retains the observed Shift
state; an actual Shift release, focus loss, or Disable still clears it. The
focused IME lane passed 598/598 after a 597/598 RED run on the old behavior.
The full release gate then passed 2,899/2,899 correctness and package tests,
lint and release build. Only that earlier IME candidate was installed on the T480; its
installed and loaded SHA-256 was
`5adea0c1f4c83e16931bd80891b92e29df7de1fce09b7e28050e9b6add3a490a`.
L1.1, L3, daemon, and IME were restarted, and `lay-ime-ru` was restored. The
earlier installation receipt at
`/home/ubu/.cache/lay/development/held-shift-install-20260925/INSTALL.json`
records the checks and rollback binary.

Scoped native-client probes on this exact installed binary (2026-09-25) used a
kernel virtual keyboard. The checked matrix is
`/home/ubu/.cache/lay/development/chrome-current-20260925/MATRIX.json`:

- Fresh ordinary Chrome Wayland `textarea` and `contenteditable`: held Shift
  produced `АРОДЕЗИАК` in all capitals; Tab accepted the visible completion;
  two daemon-owned Double Shift gestures completed
  `просто → ghjcnj → просто`. Receipts under
  `/home/ubu/.cache/lay/development/chrome-current-20260925/` are
  `held-shift-retry/held-shift-receipt.json`,
  `contenteditable-held-shift/held-shift-receipt.json`,
  `tab_cycle/receipt.json`, and `contenteditable/tab_cycle/receipt.json`.
  The DOM event logs show the intermediate text, not just the final result.
- Fresh owned Firefox `textarea`: held Shift produced `АРОДЕЗИАК` despite
  client Resets between managed commits; Tab and two Double Shift gestures
  completed `просто → ghjcnj → просто`. In a separate field, Space corrected
  `публекует ` to `публикует `, then one Double Shift restored
  `публекует `. Receipts under the same cache root are
  `firefox-held-shift/held-shift-receipt.json`,
  `firefox-tab/tab_cycle/receipt.json`, and
  `firefox/correction_undo/receipt.json`.
- Fresh owned Kitty terminal: Tab and two Double Shift gestures finished at
  `просто `. Receipt: `kitty-tab-cycle/receipt.json` under the same cache root.

The separate fresh Chrome `textarea` attempt to check undo after applied
autocorrection did **not** apply autocorrection: `публекует ` stayed unchanged
at Space, whose IME trace took `managed_fallback_commit`; Double Shift then
performed ordinary layout conversion to `ge,ktretn `. Applied-correction undo
is **NOT TESTED**, while this Chrome correction scenario is **FAIL**. Receipt:
`correction_undo/receipt.json` under the same cache root. The long-lived
authenticated Chrome profile was reached through its verified CDP route, but
GNOME kept Kitty in the foreground; its test tabs were closed without sending
keys. That existing Chrome field and hardware-keyboard input remain
**NOT TESTED**. The probes changed no source or installed binary. The scoped
test daemon was stopped and the normal daemon restored; engine and GNOME
layout at that point were `lay-ime-ru`, and L1.1/L3/daemon were active. No
commit or publication followed those earlier probes.

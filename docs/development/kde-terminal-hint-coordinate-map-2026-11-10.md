# KDE terminal: late suggestion and current cursor coordinates

This bounded follow-up uses the user's current authorization to complete the
feature and all tests. B65 remains frozen. No installation or publication is
authorized by a source test result alone.

The complete B65 fragment suites pass all 216 supported browser cases. Ordinary
browser controls retain 59 failures and 52 structural blocks; matched runs on
the exact installed 1.0.123 bytes reproduce the first shared correction/rapid
gesture mechanisms. This follow-up does not claim to fix those mechanisms.

Two terminal fixture and product observations must remain separate:

- The private raw PTY reader repainted its complete leading newline from the
  current cursor position. Its trace proves a further 18px downward movement
  per letter while the logical paragraph stays unchanged. The corrected private
  renderer anchors the buffer in its reserved screen row. All input and oracle
  functions remain identical; the original failures are retained.
- With the corrected renderer, the first Kitty suggestion/Tab passes in all
  three matched modes: 124 ON, 124 OFF and original123 OFF. The complete ten-case
  sequence still loses the coordinates of a later Kitty suggestion. This is not
  evidence that the detector replaced text or that a release is ready.

The current presentation path is:

```text
IBus SetCursorLocation
  → hint_view.rs::remember_hint_geometry
    stores coordinates + focus + existing activation identity

current candidate arrives from existing prediction worker
  → hint_view.rs::prepare_inline_terminal_hint
    unconditionally erases the stored native-terminal coordinates
    calls the existing HidePreeditText wake
  → hint_view.rs::publish_inline_hint
    no later cursor callback means rect=None although suggestion text is ready
```

Proposed bounded change:

```text
same SetCursorLocation
  → same read-only frame + the currently observed tail epoch
same candidate
  → KDE-only policy in kde/terminal.rs
    same word epoch + existing current focus/activation coordinates?
      yes: retain that coordinate frame
      no: existing clear-and-wake behavior
  → same hint publisher
```

The epoch is presentation metadata. It cannot grant a word owner, prepare an
edit, acknowledge a client mutation or bypass a verifier. GNOME and nonterminal
clients keep the preceding behavior. Both actual text writers, the letter
detector, Space rules, model selection and decoder operations stay unchanged.

Options considered: retain only the existing current word's validated KDE
terminal coordinates (9/10); retain every old coordinate (3/10, can cover typed
letters); add waits/retries (3/10, does not fix ownership of the coordinate);
alter generic text delivery (1/10, unnecessary). The first option is selected
for RED/GREEN and one short native comparison. Reject it if that proof fails.

Exact production scope: `src/bin/lay_ibus_engine/hint_view.rs` and
`src/bin/lay_ibus_engine/kde/terminal.rs`. Regression tests reside in the existing
hint-view test module. The approved scope and ADR are updated before editing.

Required evidence: a fresh same-word coordinate fails against old code, then
passes; old epoch, wrong focus, missing activation, GNOME and nonterminal
controls preserve the preceding clear behavior. No text/tail/authority mutation
may result. Short actual first/second-word Kitty and Konsole hint/Tab controls
must precede full affected source checks and fresh actual-byte acceptance.

Raw receipts: remote `letter-fragment-terminals-v67-20261110`,
`letter-fragment-class-comparison-v69-20261110` (33 matched observations),
`letter-fragment-terminals-stable-render-v72-20261110`. The private renderer AST
comparison is `terminal-renderer-proof-v71.json` in the task cache.

## Native rejection and exact rollback

The source experiment is REJECTED. RED74 reproduces erasure; GREEN75 passes
seven source tests. Strict native77 nevertheless proves that a cursor report
bound to a current local word epoch can still describe an earlier glyph.
New76 Kitty first-word x12 fails actual-text expected39; second-word75 fails84.
Old65 first-word also fails12vs39; its second-word84 passes. Konsole first-word positions pass on old/new; its second-word result is
oldFAIL/newPASS. This mixed result does not accept the rejected policy. The saved earlier positive screenshot visibly covers
typed letters; a non-null rectangle is insufficient acceptance.

Only the two experimental source files and active scope were restored exactly
from prechange74. The remaining B65 source closure is again the previously
verified one; no runtime policy from this experiment survives. No full gate,
installation or release of76 occurred. Keep the stronger private placement
oracle and reject this epoch-retention idea unless a future proposal obtains
independent native position evidence. Do not reinterpret a local word epoch,
frame, queue completion or correct Tab result as proof of glyph placement.

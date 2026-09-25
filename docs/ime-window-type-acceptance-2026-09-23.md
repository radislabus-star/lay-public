# IME acceptance by window type

The user's acceptance requirement is physical behavior on the **exact installed
binary** in each relevant window/input class. A passing library or private-bus
test does not promote a client type. Each row needs three separate observations:
Tab accepts a visibly offered completion; two consecutive Double Shift
gestures make a round trip; Double Shift cancels an applied autocorrection.
Record the resulting text, selected layout, binary hash, and trace receipt for
each observation. Any newly encountered input class gets a row before a claim
of complete window-type coverage.

The 2026-09-23 candidate `f17191f4…321548c` physically failed in Firefox:
Tab returned `handled=false` with a visible completion, and the user reported
only one direction of Double Shift. It was rolled back to the accepted binary
`daaa47bf…687b88`. The revised source is **not persistently installed**. Incident receipt:
`/home/ubu/.cache/lay/development/firefox-double-shift-live-20260923-0355/incident-receipt.json`.

| Window/input class | Local representative | Visible IME + Tab | Double Shift round trip | Double Shift autocorrection undo | Exact installed candidate receipt |
| --- | --- | --- | --- | --- | --- |
| Firefox web textarea | Firefox Snap, owned fresh Wayland window | MIXED: `...044512` accepted visible Tab; new `...103142` returned `handled=false` and Tab blurred the field before Shift | PASS in `...044512`: `просто ` → `ghjcnj ` → `просто ` | PASS in `...044418`: actual `публекует ` → `публикует `, then Double Shift restored `публекует ` | Earlier Tab/undo receipts below; new failed receipt below |
| Firefox browser address bar after adjacent-tab transfer | Same owned Firefox window, fresh neighboring tab, address bar focused | NOT TESTED: Tab completion in browser chrome | FAIL: first Double Shift status 1, then Firefox `FocusOut → FocusIn → Reset` and second RPC `not_handled/context_authority`; no round trip | NOT TESTED | `persistent-firefox-window-20260923-103450/addressbar-neighbor-physical-receipt.json` |
| Firefox contenteditable | Firefox Snap, owned fresh editable div | PASS: Tab accepted visible `про` + `сто` as `просто ` | PASS: `просто ` → `ghjcnj ` → `просто ` | OPEN: Space returned `full_no_apply` on `публекует`, so undo was not exercised | Tab receipt `...045217`; no-apply receipt `...045242` (full paths below) |
| Firefox GitHub issue #46 comment editor | Existing authenticated Firefox Snap window, real GitHub composer | USER PASS: IME, completion, Tab and autocorrection work; synthetic `про` + Tab blur had no captured visible suffix | USER FAIL: first Double Shift flips the word, but the next does not flip it back; required cycle `A → B → A → B` remains open. Scoped-uinput probes also showed context-admission failures | USER PASS for Shift rollback after autocomplete; rollback after an independently logged applied autocorrection remains unverified | User clarification on 2026-09-23; diagnostic receipts `github-issue46-live/tab-ru-prefix-receipt.json` and `double-shift-ru-receipt.json` under the cache root below |
| Chrome web textarea | Google Chrome, owned fresh profile | PASS: Tab accepted visible completion as `просто ` | PASS: `просто ` → `ghjcnj ` → `просто ` | OPEN: `публекует` was not autocorrected; ordinary layout conversion followed | Tab receipt `...045527`; no-apply receipt `...045555` (full paths below) |
| Chrome contenteditable | Google Chrome, owned fresh editable div | PASS: Tab accepted visible completion as `просто ` | PASS: `просто ` → `ghjcnj ` → `просто ` | OPEN: `публекует` was not autocorrected; ordinary layout conversion followed | Tab receipt `...045641`; no-apply receipt `...045706` (full paths below) |
| GTK entry/dialog | GTK 4 owned entry | PASS: Tab accepted visible `про` + `сто` as `просто ` (`handled=true`) | PASS: `просто ` → `ghjcnj ` → `просто ` | OPEN: `публекует ` was not autocorrected, so undo could not be exercised | Tab receipt `...044838`; no-apply receipt `...044900` (full paths below) |
| GTK multiline editor | Owned GTK 4 TextView (Gedit crashed before input) | PASS: Tab accepted visible `про` + `сто` as `просто ` | PASS: `просто ` → `ghjcnj ` → `просто ` | OPEN: `публекует` was not autocorrected; ordinary layout conversion followed | Tab receipt `...045845`; no-apply receipt `...045907` (full paths below) |
| Qt multiline text widget | Owned PySide6 `QTextEdit` | PASS: Tab accepted visible `про` + `верка` as `проверка` | PASS: two status-3 exact-tail delegations completed a round trip | OPEN: Space reported `prefetch_not_ready`, so no applied correction existed to undo | Tab receipt `...051244`; no-apply receipt `...051308` (full paths below) |
| VTE terminal | Owned GNOME Terminal window | PASS: Tab accepted visible `про` + `сто` as `просто ` | PASS: IME handled both local plans, `просто ` → `ghjcnj ` → `просто ` | OPEN: Space reported `prefetch_not_ready`; no correction to undo | Tab receipt `...050152`; no-apply receipt `...050218` (full paths below) |
| Kitty terminal | Owned Kitty window | PASS: Tab accepted visible `про` + `сто` as `просто ` | PASS: IME handled both local plans, `просто ` → `ghjcnj ` → `просто ` | OPEN: Space reported `prefetch_not_ready`; no correction to undo | Tab receipt `...050327`; no-apply receipt `...050351` (full paths below) |
| Office document editor | LibreOffice Writer | NOT TESTED: isolated LibreOffice fixture startup failed | NOT TESTED | NOT TESTED | Launch receipt `writer-fixture/LAUNCH-RECEIPT.json` |
| Electron text editor | VS Code isolated Snap profile | NOT TESTED: owned window was not ready before sender guard | NOT TESTED | NOT TESTED | No-input receipt `...051019` (full path below) |
| Qt message composer | Telegram Desktop draft | NOT TESTED | NOT TESTED | NOT TESTED | — |

For each row, test the same field both before and after a focus transfer when
that client supports it. The matrix is an inventory of locally available
representatives, not a claim that every possible application has been tested.
No browser automation may send external messages while exercising a composer.

The Firefox row ran on temporarily installed and loaded engine SHA-256
`dc030fdcc7e6a3096825be2fb2612cb35d13ef372e50a02a66108145032c142c`.
Each scoped test restored installed and loaded SHA-256
`daaa47bf400b8fb06d124a31c0790422f8a830aa26cecfdfd8152b2687687b88`
and the active Lay daemon afterwards. The two receipts are in
`/home/ubu/.cache/lay/development/firefox-double-shift-live-20260923-0355/installed-candidate-firefox-td121_tab_two_toggles-20260923-044512/`
and
`/home/ubu/.cache/lay/development/firefox-double-shift-live-20260923-0355/installed-candidate-firefox-td121_known_autocorrect_undo-20260923-044418/`.
The owned fresh Firefox window is one textarea representative; the user's
existing Firefox window and other input classes remain untested on these bytes.

The GTK 4 entry used the same temporarily loaded candidate SHA. Its dynamic
Tab check compared the final text with the actually visible preedit suffix,
because the suggestion varied between runs; the `...044838` receipt reports
`ok=true`, `handled=true`, one completion acceptance and two exact-tail
delegations. The autocorrection attempt reached Space with a nine-char
surrounding snapshot but took `managed_fallback_commit`, leaving the typo
unchanged. GTK reported IBus capabilities 41, with surrounding text but without
the exact-refresh capability; the Firefox adapter reported 1073741865 with
that capability. The following Double Shift converted it to `ge,ktretn `. The two
receipts are at
`/home/ubu/.cache/lay/development/firefox-double-shift-live-20260923-0355/installed-candidate-gtk-td121_tab_two_toggles-20260923-044838/`
and
`/home/ubu/.cache/lay/development/firefox-double-shift-live-20260923-0355/installed-candidate-gtk-td121_known_autocorrect_undo-20260923-044900/`.
This is an open GTK autocorrection precondition, not evidence that undo
itself failed. The installed and loaded baseline was restored after both runs.

An owned Firefox `contenteditable` div used the same browser release adapter.
Its Tab and two-toggle case passed (`просто` after the full round trip), but a
second case retained `публекует ` at Space with `full_no_apply` at rank. The
following Shift made an ordinary layout conversion. Receipts:
`/home/ubu/.cache/lay/development/firefox-double-shift-live-20260923-0355/installed-candidate-firefox-contenteditable-td121_tab_two_toggles-20260923-045217/`
and
`/home/ubu/.cache/lay/development/firefox-double-shift-live-20260923-0355/installed-candidate-firefox-contenteditable-td121_known_autocorrect_undo-20260923-045242/`.
The installed and loaded baseline was restored after both runs.

Both owned Chrome input classes accepted Tab and completed two Double Shift
gestures. The Chrome window verifier used the unique local page title and the
owned process group: GNOME attributed the window to a renderer PID without the
browser process's `--user-data-dir` argument. Two earlier admission attempts
stopped before any input while that identity rule was too narrow. Neither
Chrome undo case applied autocorrection before Shift; both reported a normal
word followed by an ordinary layout conversion. Chrome advertised IBus
capabilities 41 (surrounding text present, exact refresh absent). Receipts:
`/home/ubu/.cache/lay/development/firefox-double-shift-live-20260923-0355/installed-candidate-chrome-td121_tab_two_toggles-20260923-045527/`,
`/home/ubu/.cache/lay/development/firefox-double-shift-live-20260923-0355/installed-candidate-chrome-td121_known_autocorrect_undo-20260923-045555/`,
`/home/ubu/.cache/lay/development/firefox-double-shift-live-20260923-0355/installed-candidate-chrome-contenteditable-td121_tab_two_toggles-20260923-045641/`, and
`/home/ubu/.cache/lay/development/firefox-double-shift-live-20260923-0355/installed-candidate-chrome-contenteditable-td121_known_autocorrect_undo-20260923-045706/`.
The user's existing Chrome profile and focus-transfer case remain untested on
this candidate. The accepted installed and loaded engine was restored after
every run.

An owned GTK 4 `TextView` supplied the multiline-widget representative. Its
dynamic Tab check reported `handled=true`, one completion acceptance and two
status-3 exact-tail delegations. Its separate correction attempt left
`публекует ` unchanged before Shift and therefore could not exercise undo.
Receipts:
`/home/ubu/.cache/lay/development/firefox-double-shift-live-20260923-0355/installed-candidate-gtk-textview-td121_tab_two_toggles-20260923-045845/`
and
`/home/ubu/.cache/lay/development/firefox-double-shift-live-20260923-0355/installed-candidate-gtk-textview-td121_known_autocorrect_undo-20260923-045907/`.
Gedit itself was not opened; this row does not assert Gedit-specific behavior.

An owned GNOME Terminal VTE window used a timeout-limited `read` command to
capture text without executing typed content. Tab accepted the visible suffix,
and two Double Shift gestures completed locally inside IME (`ManualToggleV3`
status 1 twice, with opposite layout plans). Its undo stimulus did not apply
correction: Space reported `prefetch_not_ready`, then ordinary layout
conversion occurred. Receipts:
`/home/ubu/.cache/lay/development/firefox-double-shift-live-20260923-0355/installed-candidate-gnome-terminal-td121_tab_two_toggles-20260923-050152/`
and
`/home/ubu/.cache/lay/development/firefox-double-shift-live-20260923-0355/installed-candidate-gnome-terminal-td121_known_autocorrect_undo-20260923-050218/`.
The candidate was temporarily loaded and the accepted binary restored after
each run. Other terminal emulators are separate rows.

An owned Kitty window ran the same timeout-limited text capture. Tab accepted
the visible suffix and two IME-local Double Shift plans made a round trip;
the applied-correction undo stimulus again received `prefetch_not_ready` at
Space, leaving no correction to cancel. Receipts:
`/home/ubu/.cache/lay/development/firefox-double-shift-live-20260923-0355/installed-candidate-kitty-td121_tab_two_toggles-20260923-050327/`
and
`/home/ubu/.cache/lay/development/firefox-double-shift-live-20260923-0355/installed-candidate-kitty-td121_known_autocorrect_undo-20260923-050351/`.
The accepted installed and loaded engine was restored after both runs.

Gedit 48.1 standalone launch was attempted separately for the same row, but
its process exited with SIGSEGV before window focus or any injected input;
GLib-GIO reported a `g_dbus_action_group_get` assertion. This is a client
startup blocker, not a Tab/Shift result. Scoped no-input receipt:
`/home/ubu/.cache/lay/development/firefox-double-shift-live-20260923-0355/installed-candidate-gedit-td121_tab_two_toggles-20260923-050642/`.

The isolated LibreOffice Writer fixture could not be prepared: headless ODT
conversion exited 1 with `com::sun::star::deployment::DeploymentException`
and no file. No Writer input was sent; this remains a startup blocker, not a
Tab/Shift result. Receipt:
`/home/ubu/.cache/lay/development/firefox-double-shift-live-20260923-0355/writer-fixture/LAUNCH-RECEIPT.json`
(SHA-256 `7d802cac8d45335b0cd6ac030f00231fc6b84b5168f64b0187ed60d9edfb1e09`).

VS Code was launched with a distinct temporary Snap profile and local empty
file. Its window appeared after the sender's readiness deadline, so the guard
sent **no** test input. The Snap main process detached from the launcher; its
exact test-profile process group was terminated and verified absent. This is
a harness/startup attempt, not a VS Code IME verdict. Receipt:
`/home/ubu/.cache/lay/development/firefox-double-shift-live-20260923-0355/installed-candidate-code-td121_tab_two_toggles-20260923-051019/`.
The accepted installed and loaded engine remains active.

An owned local PySide6 `QTextEdit` supplied a Qt multiline-widget
representative. Tab accepted the actually visible `про` + `верка` suggestion
as `проверка`, then two Double Shift gestures completed a text round trip
through status-3 exact-tail delegations. Passed receipt:
`/home/ubu/.cache/lay/development/firefox-double-shift-live-20260923-0355/installed-candidate-qt-textedit-td121_tab_two_toggles-20260923-051244/`
(trace SHA-256 `c00a316242f87d135128e2913ccae29eaad4182b53a8ab3f1a0564c54deb1c3e`).
The separate `публекует` attempt left the typo unchanged at Space because
autocorrection reported `prefetch_not_ready`; the following Double Shift made
an ordinary layout conversion. This is no applied-correction undo verdict.
Negative receipt:
`/home/ubu/.cache/lay/development/firefox-double-shift-live-20260923-0355/installed-candidate-qt-textedit-td121_known_autocorrect_undo-20260923-051308/`
(trace SHA-256 `7b4656cf7e11559fc0fb5ccf09f70a7a264d9c95189bfe8abcd24c73ecc1d8d9`).
The accepted installed and loaded engine SHA was restored after both runs.
This local widget does not establish Telegram Desktop behavior.

At 10:31 a new temporary `dc030f…` run in another owned fresh Firefox
textarea typed `про`, then Tab returned `handled=false` and moved DOM focus
out of the field. The guarded run stopped before either Double Shift; this
contradicts the earlier Tab PASS for that input class, so the class is not
stable. Failed receipt:
`/home/ubu/.cache/lay/development/firefox-double-shift-live-20260923-0355/installed-candidate-firefox-td121_tab_two_toggles-20260923-103142/`.

The persistent owned Firefox window PID 1702519 was then left open for the
user. In its browser address bar, a controlled physical-uinput run after
`Ctrl+L` and selection made two layout conversions, but case drift changed
`просто` to `ПрОсТо`; it is not a clean acceptance result. Receipt:
`/home/ubu/.cache/lay/development/firefox-double-shift-live-20260923-0355/persistent-firefox-window-20260923-103450/addressbar-physical-receipt.json`.
The closer reproduction opened a fresh neighboring tab in the **same** window
and used the newly focused address bar without `Ctrl+L` or selection. The
first Double Shift was handled (`ManualToggleV3` status 1); Firefox then sent
`FocusOut → Disable → FocusIn → Enable → Reset`. The new context installed
`UnknownStart`, with zero committed-tail and preedit characters at the next
Shift; the second RPC returned `not_handled`, `reason=context_authority`,
`bridge_token_live=true`, `manual_toggle_allowed=false`. The daemon did
recognize both physical double taps. The sampled accessibility text stayed
empty, so no visually correct two-way cycle is established. Exact receipt:
`/home/ubu/.cache/lay/development/firefox-double-shift-live-20260923-0355/persistent-firefox-window-20260923-103450/addressbar-neighbor-physical-receipt.json`.
The trace and daemon log are in the same directory. The candidate was
automatically rolled back to installed and loaded `daaa47bf…687b88` after
the probe, and the window was not closed.

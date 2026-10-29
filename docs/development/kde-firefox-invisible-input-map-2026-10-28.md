# KDE Firefox: typed letters disappear until Tab

Status: NATIVE_RENDER_PASS; separate preexisting post-Tab Shift controls remain open.
This is separate from
the accepted Double Shift after autocorrection case in the first ordinary input.

The user initially reported all fields except field 1, then corrected this:
«а нет в 1 тоже». The reported scope is therefore all ten Firefox fields,
including the first. Letters disappear while typing and appear after Tab.
Do not turn the earlier Double Shift acceptance into display acceptance.

## Frozen baseline

Canonical checkout: `/home/ubu/projects/lay-dependency-updates-20261027`,
branch `codex/browser-matrix-repair-20261027`, HEAD
`b835ca478198d6cb765a2246329d5646966c118c` plus the already recorded adapter
setup. Remote Firefox PID719893, IME3556137 and daemon374525 were verified alive
in the KDE `:71` session. Existing adapter remains loaded; no runtime rebuild,
service restart, synthetic typing, or rollback is part of this diagnosis.
Manual input remains with the user. GNOME and Chrome remain outside this task.

## Dependency map and competing explanations

1. `scripts/compat/firefox-ibus-reset-notify.c:35` adds the existing exact-refresh
   capability; `:97` requests real GTK surrounding after a handled release.
2. `src/bin/lay_ibus_engine/managed.rs:611` selects the existing literal-preedit
   owner. Exact refresh can enable this path at an observed boundary.
3. `src/bin/lay_ibus_engine/preedit.rs:607` publishes the literal buffer before
   background candidate computation. `:707` sends nonempty UpdatePreeditText
   with visible=true and, on the show transition, ShowPreeditText.
4. `src/bin/lay_ibus_engine/display.rs:24` selects GNOME inline versus KDE
   auxiliary completion presentation. `preedit.rs:636` still publishes the
   owned literal prefix separately before the auxiliary suffix in KDE.
5. `src/bin/lay_ibus_engine/text/attrs.rs:26` gives the typed prefix an explicit
   no-underline attribute. It does not apply hint foreground/background to it.
6. `preedit.rs:373` can clear a stale pending display frame; `:503/:535` clears
   visible composition. Need an actual event sequence to establish whether
   any such clear hides a still-owned literal buffer in this case.

The initial suspicion around `managed.rs:576` is not a demonstrated cause:
`commit_observed_literal_preedit` explicitly excludes exact-refresh clients,
including the current Firefox adapter setup. Do not change this path on that
hypothesis alone.

GNOME uses the same installed compatibility-library SHA, but its loaded IME
executable is older/deleted; the current updater checkout does not identify
those loaded bytes. An assumed identical GNOME runtime is not valid evidence.

## Observation and limits

Existing detailed IME trace last modified at 2026-10-28T15:48:46Z, before the
reported display failure. Firefox stderr last modified at18:08:21Z contains
surrounding-capability warnings. Those older warnings do not prove this cause.

Bounded passive diagnostic tool:
`CROOT/kde-firefox-visible-input-passive-v1.py`, result
`CROOT/kde-firefox-visible-input-passive-v1/result.json`.
It records IBus message metadata, string lengths, visibility booleans and
attribute integers, without retaining input text. No application input,
configuration edits, focus changes or process restarts. CROOT is
`/home/e/projects/lay-development-runner/kde-install-control-20261020`.

The external monitor returned no events; connection observability was not
established. A separate `busctl list` attempt timed out. This result is
NOT_CAPTURED, not evidence that the user typed nothing or that frames were absent.
Do not reuse its startup label PASSIVE_READY as proof of a working observer.

Next bounded diagnostic uses the existing `debug_action_log` flag for 240 seconds
in the private KDE config
`/home/e/.cache/lay/kde-live-20261019/config/lay/config.json` only. Preserve all
other bytes/settings and restore the original flag automatically in finally;
preserve concurrent user config edits. The flag is excluded from
`LexicalAuthorityConfigIdentityV1` (`src/lexical_authority_frame.rs:113–164`), so
this is diagnostic logging, not a new candidate/word authority or output path.
No source, client environment, service, browser, desktop or GNOME change.
Collect only fresh existing trace records; redact text from the exported packet.

First distinguish a nonempty visible frame followed by an engine clear from a
nonempty frame reaching Firefox without a clear. These imply different owners
of the first failed transition. Do not patch a shared route before that evidence.

Full matrix is historical and unchanged: Firefox127P/55F/38B of220;
Chrome179P/0F/41B of220; total306P/55F/79B of440. This user report has not
recounted that matrix. No release or public push in this task.

## Fresh physical/source evidence and accepted direction

The built-in 240-second trace completed with21 key records,8 preedit records,
zero observation gaps, unchanged executable PIDs/hashes and exact config restore.
Receipt: `CROOT/kde-firefox-visible-input-builtin-v3/result.json`. The first
built-in attempt v2 stopped on an omitted default flag before modifying config;
v3 handles that legitimate omitted=false default and restored the original bytes.

User reports that each typed letter disappears when the suffix appears, and
Tab makes the letters visible. A one-character reproduction shows update/show
of1 character, then another visible update of1 character on candidate readiness;
there is no clear between them. Clear appears later on Backspace.

Screenshot `CROOT/kde-firefox-visible-input-builtin-v3/held-p-firefox.png`
shows `ривет` over the beginning of field1. X11 identifies a separate
`lay-kde-tray.py` tooltip (0x2c00006),57x24 at362,307, over that same location.
The current `InlineHintSnapshotV1` has suffix length5 and rect[362,307,0,24].
`hint_view.rs:135–145` forwards the raw IBus composition-start rectangle;
`scripts/lay-kde-tray.py:338–354` treats it as a suffix-start rectangle.
Mozilla `widget/gtk/IMContextWrapper.cpp::SetCursorPosition` independently
explains the GTK contract: while composing, horizontal text uses the target
clause's first character bounds, not necessarily the end caret. The first
failed display transition is using that composition-start anchor for an opaque
suffix-only view. Do not interpret this as a missing correction candidate.

Rejected before implementation: using GNOME-style whole-word completion in
KDE preedit. Existing `preedit/tests.rs::kde_legacy_unaccepted_completion_is_never_document_composition`
protects client-forced blur commits from accepting a hint. Preserve that guard.
Guessed font-width offsets also cannot prove exact placement on arbitrary pages.

The user chose «В строке, с точными координатами Firefox (рекомендую)» and
finished the held input with Tab. The existing native Firefox controller already
uses process-scoped `GNOME_ACCESSIBILITY=1` on KDE, preserving the KDE desktop
and global settings. Reuse this flag in the KDE-only launcher. Current manual
Firefox is not registered with AT-SPI; no forced-disabled preference is present.
Verify registration after a normal same-profile Firefox restart before coding
the geometry bridge. No global accessibility setting or Firefox preference edit.

At2026-10-28T19:40:01Z, after the Firefox-only accessibility setup, the user
reported that the first letter was visible, but the second word overlapped
again. This is not an accepted fix. The same unchanged IME projected the
three-character suffix at[435,315,0,23], while AT-SPI observed10 actual
characters, caret10, no selection, two words and the last character bounds
[458,315,12,22]. The required suffix start is470, not435. Receipt:
`CROOT/kde-firefox-second-word-overlap.json`; screenshot with the same stem.
The first accessibility probe returned zero character bounds during setup;
the settled subsequent read returned valid bounds. Zero bounds must never be
used as a guessed suffix position. Accessibility enables the reader but alone
does not fix the second-word transition.

The launcher is installed in FirefoxPID988548, same window/profile/adapter;
verified after page load without another browser restart. Protected daemon,
IME, IBus, tray and config hashes stayed unchanged. Receipt:
`CROOT/kde-firefox-owned-hint-setup-v1/verified-after-page-load.json`.
The initial setup script checked the title before page load and exited;
the read-only follow-up, not that early failure, establishes the settled setup.

The isolated Rust implementation is `kde/firefox/hint_geometry.rs`, attached inside the
existing `native_document` module to reuse its private bus reader without
adding a public observation API or a new authority owner.

Exact seams are in the new ADR/active scope. The only shared output
change is choosing a display rectangle for current KDE Firefox owned preedit;
all other clients retain their previous rectangle. New code lives under
`kde/firefox/`. Reuse native AT-SPI reads in a helper returning only a
rectangle, never install NativeDocumentBinding or feed this read into correction,
word ownership, verifier or execution. No old-start fallback on a failed read.

## Checked implementation and installed candidate

Remote source checks:10/10 short and1120/1120 IME correctness/package testsPASS;
1123 IME tests are catalogued, including three unchanged performance tests not
run here. Only three new semantic tests were added; no existing test or target
row changed or disappeared. Graph/architecture contractsPASS. Receipt:
`CROOT/kde-firefox-owned-hint-source-v1/result.json`.

Canonical scope exported from HEADb835ca478198d6cb765a2246329d5646966c118c
and verified against the actual remote files; synthetic remote HEAD is not used
as the scope baseline. Source-content SHA after the test registry update:
bcc3d0f5b7c3b0f106f2c20b8ee76b718e9657e1cae3a94ead364b2d8cd1edf9.
Receipt: `CROOT/kde-firefox-owned-hint-scope-verified.json`.

Release IME buildPASS,0 warnings,74.034seconds. The build retained its exact
candidate and restored the old activation path until the separate idle-checked
installation. Receipt: `CROOT/kde-firefox-owned-hint-build-v1/result.json`.
After the user finished the held input, only IME was replaced. Actual loaded
PID1118793 SHA29b6bc2f7a707dbdf66a877c9c4046f8c8c4bbad7b0124f0f6ae3ff5bd9a93bf;
daemon374525, IBus3168326, tray3228081, manualFirefox988548 and config unchanged.
Receipt: `CROOT/kde-firefox-owned-hint-deployment-v1/result.json`.

The user then explicitly authorized automatic testing: «все дальше сам все
тестируй», «без меня кажется ты нашел». Use the existing owned native Firefox
fixture/RFB actor with the same KDE launcher/library. Preserve the user's manual
window. First one ordinary field, then ten fields, with real prefix bounds,
hint position, Tab and consecutive Double Shift controls and separate timing.
Do not broaden into GNOME/Chrome or recount the historical full matrix.

Working agreement: if successive checks stop producing new evidence or progress,
explain the concrete blocker and ask the user for a short manual reproduction.
Do not repeat blind edits or an unchanged failed test indefinitely.

## First native pass and the missing common display gate

The single ordinary field passed six first/second-word prefix measurements,
Tab and two consecutive Double Shift transitions on the installed candidate.
All three second-word rectangles exactly matched the actual character end.
Geometry reads took15.509–18.027ms. Receipt:
`CROOT/kde-firefox-owned-hint-short-native-v3/{result.json,firefox/result.json}`.
The v1/v2 probes had stopped on the unchanged committed-text GTK font metrics:
line height differs1px, and the caret advance can differ1px from the character
box. Native v3 retains that measured difference and allows1px for the visual
boundary; the original35px overlap cannot pass. No runtime change was made
between those measurement versions.

The ten-field run then exposed the first shared missing gate: some Firefox
fields already commit the typed prefix, so `legacy_word_preedit_active` is
false, yet the raw IBus anchor can still be at its beginning. In search,
hintX775 while the observed character is[776,364,11,22]. The first candidate's
legacy-only guard skipped the geometry helper entirely. Therefore apply the
same read-only, exact-prefix-checked display read to either representation of
the current nonempty Firefox word. Keep KDE, exact-refresh capability, atomic
and sensitivity guards, native field/process identity and prefix validation.
This changes display eligibility only, never text ownership or edit authority.

That first ten-field receipt is3PASS/6FAIL/1BLOCKED, not acceptance:
`CROOT/kde-firefox-owned-hint-ten-fields-native-v3`. Several FAILs are this
shared skipped-geometry case. Rich/plain fields passed all six geometry points
and Tab but failed the separate post-Tab Double Shift control; do not relabel
their display result as a Double Shift success. URL/email two-word preparation
is separately unsupported by trailing-Space sanitization in these HTML controls.

The second source candidate also passed source checks but its first native
field stopped before acceptance: `composition.buffer` is empty for literal
committed text, even while `committed_tail` contains the typed current word.
Its eligibility change therefore still skipped the reader. The existing
`preedit.rs::visible_case_completion_suffix` already selects those two word
representations. Reuse that contract in the Firefox geometry helper with the
existing `last_tail_token_text()` accessor, preserving case and returning no
current prefix after a committed separator. Add a semantic test for both
representations, uppercase/Unicode and closed boundaries. Do not create a new
word owner, parser or correction source.

## Proven Firefox metadata-read race

The third candidate passed11/11 short and1121/1121 IME source checks but the
first fresh-profile native field produced no hint. The guarded geometry read
returned `native_document_shape`; rejection cleared selectability correctly.
The visible typed letter and fast two-word input still worked. User has been
called for an optional manual comparison after these unsuccessful native passes.

A one-field owned-window D-Bus diagnostic establishes the first failed transition,
not an assumed client limitation. Engine peer:1.6767/PID1300026 read
CharacterCount0, GetText(0,0)="", CaretOffset-1, CharacterCount0. The observer
then received CharacterCount1 only8.376ms later, followed by actual held-text
and caret1. The CommitText signal and managed-key return preceded the failed
read. Receipt: `CROOT/kde-firefox-owned-hint-shape-diagnostic-v6/wire-state-transition.json`;
raw wire is scoped to the owned test Firefox app. Its native per-row trace is
complete. Overall append-offset trace collection can fail on debug-log rotation;
never treat its missing file as a complete capture or export old global history.

Reuse the pattern already implemented in
`kde/chrome/observation.rs::read_with_owned_append_resampling`: keep one original
deadline and the same1ms cooperative yield between actual reads. Firefox-only
display eligibility waits for a strictly validated snapshot containing the
current prefix and nonzero real character bounds, within the unchanged80ms
OBSERVATION_BUDGET. Resample only transitional shape/change/prefix/bounds
observations. Privacy, selection, focus-state and transport failures stop
immediately; no reconnect or old-start fallback. No snapshot validator, Chrome
function, delivery route or runtime owner changes. Tests assert strict terminal
failure behavior and zero native reads after an expired budget.


## Final accepted evidence scope — source checks and physical native display

See `kde-firefox-hint-geometry-2026-10-29-verification.json` and the retained
`kde-firefox-hint-geometry-protocol.md`. Latest scoped source checks13/13 and
1123/1123PASS; release build0warnings. Actual loaded/installed/activation SHA
004f6285a1725f280a4549c18ffa8d8937d6c27728b4dd0edfddf85a068e1c28,
PID1443794 after the baseline comparison, base version1.0.111, unpublished.
The original daemon/IBus/tray/manual Firefox processes and config are preserved.

Native Firefox159.0 (fixture capabilities and retained manual process metadata),
all10 fields in normal/private/fullscreen mode:162/162 measured rectangles
exactly meet the actual character end (horizontal error0 in every point).
First-word coverage30 fields; complete two-word coverage24 fields; URL/email
second-word steps6BLOCKED by their trailing-Space sanitization. Tab24/24 and
fast native typing24/24PASS. Samples are X11/RFB/DPR1, bounded short fields;
long documents, Wayland, USB input, GNOME and Chrome are not physically retested.

The full54 diagnostic rows remain39PASS/9FAIL/6BLOCKED. The9 failures are the
separate Double Shift-after-Tab control in rich/plain/iframe-rich in all3 modes.
Four identical normal-window controls on retained old8b84bf5b and current004f6285
bytes give the same ordinary-inputPASS and the same3 rich failures. Receipt:
`CROOT/kde-firefox-post-tab-compare-v3/result.json`. An earlier comparison v2
stopped on a tooling receipt-path error before keyboard controls; its restoration
succeeded. Do not count that failed setup as a physical comparison.

Aggregate measured hint latency: median32.271ms,p9535.002ms,max116.503ms,n162,
including native observer/model/transport polling. The80ms native read deadline
is unchanged; it is not the whole end-to-end hint latency. Tab median17.921ms,
p9522.294ms,n24. Successful Double Shift median183.767ms,p95216.531ms,n30;
9 failed first transitions and9 unreached second transitions remain outside
those success-only timings. Do not claim a global Double Shift speed gain.

No public release, commit or push was performed. The historical440-row browser
matrix remains306PASS/55FAIL/79BLOCKED; these diagnostic rows do not replace it.

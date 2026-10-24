# IME hint presentation in every application and system

Current requested behavior (2026-10-21): every suggested, unaccepted suffix
character is black `#000000` on an opaque white `#ffffff` background, inline
after ordinary typed text. This explicitly supersedes the earlier muted-gray
request and rejected 50% translucency. Lay must not change actor opacity.
User-entered and committed text retain their normal styling. This is
presentation under C06; it adds no text, visibility, acceptance, input-source
or mutation authority. Earlier dated receipts below describe historical bytes.

## Universal acceptance requirement — 2026-10-20

The user clarified that this applies everywhere, in every system, including
Chrome and terminals: entered characters remain ordinary text. Only the
unaccepted suggestion may use the requested contrasting hint styling. Coloring,
selecting, reversing or underlining the whole typed word is a presentation FAIL,
even if completion, correction and Double Shift pass. Functional native counts
must not be used as visual acceptance. Inspect both the owned typed prefix and
the proposed suffix in real clients, on light and dark backgrounds where relevant.

Initial native evidence reproduces whole-prefix reverse video in Kitty 0.21.2
and Konsole 21.12.3. Their input-method renderers ignore per-span IBus styling;
see the primary source links and receipts in
[remote input evidence](../development/remote-x11-input-2026-10-20.md).
Chrome's composition converter also ignores foreground and maps underline NONE
to a default solid composition underline. This is a confirmed compatibility gap,
not accepted behavior and not an exemption from the user's requirement.

Current changed-byte Kitty observation (2026-10-20): engine
`c33f9f506a3805598f5838f67c6341c121e78ea038d3c0465e2444eb663775af`,
`terminal-final-benchmark-v1/hint.png`, independently inspected. Ordinary
desktop launch shows entered `пров` as normal terminal text and only auxiliary
`ерка` as a muted suggestion. Raw PTY already contains the prefix before Tab.
Kitty visual verdict: PASS for this owned KDE/X11 window. Functional benchmark
is 20 PASS / 3 FAIL, including rapid-five and following control failures; visual
success does not reclassify them. Other clients and systems remain OPEN under
TD-160. Exact receipts and final native matrix belong to the remote-input log.

No test may claim that a payload attribute alone fixes a renderer that discards
it. A universal presentation solution, if it changes composition ownership or
commit timing, needs an explicit canon decision and native regression proof;
client name or screenshot alone grants no edit capability. Runtime authority and
delivery routes remain unchanged in this investigation.

Fresh Chrome native observation (2026-10-20, remote KDE):
`universal-hint-prefix-v1/result.json` is functional PASS for one Tab case and
its following manual-conversion controls. `hint-text.png`, captured before Tab,
visibly underlines the whole typed prefix. Visual verdict: FAIL. It does not
establish any other field or platform. The minimal solution options, ownership
pitfalls and required native proof are recorded in
[TD-160](../../tech_debt/160-universal-ime-prefix-presentation.md). The common
browser protocol now requires a separate visual verdict and screenshot.

## Inline appearance follow-up — 2026-10-20

Latest bounded result (remote runtime rechecked 16:06 UTC): installed engine
`9e751d3c` has 943/943 source checks and ordinary KDE-menu Kitty native visual
PASS. Its prefix is ordinary white, continuation gray and inline, and Tab
accepts once. Raw PTY Tab/blur is 2/2 PASS; one reaction sample is 1.786365 ms.
The user's old Kitty was launched without GLFW_IM_MODULE and remains open;
new native menu launch obtains ibus from the desktop entry. Exact receipts:
`user-kitty-native-v2`, `kitty-kde-menu-launch-v1`, `inline-kitty-geometry-v5`
under the 2026-10-20 remote control root. Chrome typed-prefix underline and a
manual-conversion control remain open, as does the GNOME queued-frame review
finding. The full changed-byte browser matrix and GNOME native appearance
are not tested. See the preflight and current lay2 handoff; the following
implementation checkpoint is historical.

The user rejected the separate auxiliary field. The preceding Kitty PASS
establishes ordinary-prefix / muted-suffix styling only; it does not establish
the now-required inline placement. The user's loaded GNOME engine is 1.0.90,
not the installed menu's newer version. Its document-preedit hint explains the
appearance, but reverting that route would restore the measured Firefox blur
commit defect. Keep the existing blur protection.

The bounded implementation projects only display text/geometry through the
existing bridge into the existing KDE tray and GNOME extension. It adds no
document composition, key handler, candidate or acceptance owner. Full
consequence analysis, alternatives, unavailable-frontend behavior and scope:
[inline preflight](ime-inline-hint-preflight-2026-10-20.md), with decision
`2026-10-20-inline-readonly-hint-view.json`. Neither emitted geometry nor a
successful signal is client-paint or edit acknowledgement.

Remote experiment: glyph-only painting over black/white owned Qt fixtures
preserved X11 focus, but is not native IME acceptance. Its exact screenshot and
receipt are in the preflight. First compiled source failed by stack overflow in
an unchanged native-transfer regression. Bounding the geometry snapshot and
new nested futures passed the same unchanged-stack IME target: 941 selected,
941 executed, 941 PASS, 0 FAIL, 132.2 seconds. Receipt:
`/home/ubu/.cache/lay/development/run-ubqe2a89/RESULT.json`.
The intervening refused launch is BLOCKED_RESOURCE, not a correctness result.

Frontend source checks and candidate build are underway. No inline candidate
is installed or physically accepted at this checkpoint. Native baseline remains
Firefox 85/75/0/0 and Chrome 9/7/0/144 (PASS/FAIL/BLOCKED/NOT_TESTED). GNOME native,
Wayland, inline placement and Chrome owned-prefix styling remain unproven.
Runtime input authority is unchanged.

## KWin qualification correction — 2026-10-20 12:16 UTC

Authenticated KWin observation has an empty Kitty desktopFileName and exact
resourceClass kitty. The metadata classifier correction passes four tests but
its native result remains 1 PASS / 2 FAIL and prefix visual FAIL. Diagnostic
candidate `02e015ee` shows the probe is not called: admitted engines have no
legacy focus_receipt, as intended by the current ownership contract.

A bounded candidate uses the existing live AdmissionToken field-receipt
projection for the existing probe and revalidates it at use and completion.
Legacy engines retain their real focus receipt; admitted engines never fall
back to one. A cached classification cannot outlive the live admitted token.
No word completeness, client acknowledgement or deletion authority is added.
Semantic RED and 5/5 targeted GREEN are recorded in
`kwin-context-receipt-{red,green}-v1.log`. The context-receipt build still probes
after the first letter. Reusing the same probe before the first printable mode
selection passes 5/5 focused checks (`kwin-first-key-green-v2.log`) and the
ordinary-launch native hint case (`terminal-kwin-first-key-v1`): typed `пров`
is already ordinary PTY text; screenshot `terminal/hint.png` shows only gray
auxiliary `ерка`; Tab produces `проверка ` in 7.44 ms. Installed engine is
8b7fefb3ba2359e453aa0ea059c63a31afa9aa69eb7c4cd26bd634a2915997e9.
The probe costs 4.996 ms in that run. This is Kitty visual acceptance only.

The native control run `terminal-kwin-first-key-controls-v1` has 2 PASS / 1 FAIL:
hint/Tab and moved boundary pass, but the second manual conversion delegates
to the daemon after RU/EN engine activation. Fresh trace shows new cursor
geometry before Ready and no later printable callback to qualify the new
field. This is native RED for the same qualification order. The next bounded
change invokes the existing one-shot helper inside the existing bridge output
guard before manual-owner admission. It adds no owner, probe policy, timer,
fallback route, text acknowledgement or deletion proof.

The changed-byte native control receipt `terminal-kwin-manual-controls-v1`
passes all three rows on engine
e6d1ea292fe6be9f2353cda70675abccb9575e87187089dfcabf1e1effdb650f:
Double Shift both ways 14.20 / 15.65 ms, Tab 1.62 ms and moved-boundary pair.
Before Tab, raw PTY contains `пров`; independently inspected `terminal/hint.png`
shows that prefix in ordinary terminal text and only auxiliary `ерка` muted.
Two Double Shift samples and one Tab sample are diagnostic measurements, not
population median/p95 or the complete rapid-gesture protocol. Final source
and wider native gates remain pending. GNOME/Wayland and non-Kitty appearance
are not certified by this receipt.
Canon decision: `decisions/2026-10-20-kde-ime-terminal-focus.json`.

## Ordinary terminal launch evidence — 2026-10-20 09:45 UTC

The preserved user's remote Kitty had no `GLFW_IM_MODULE`. Earlier fixtures
explicitly selected its IBus client and therefore did not certify ordinary
launch. The same installed v3 engine now has normal desktop-command RED/GREEN:
before setup, `пров` followed by Tab yields `пров\t`; after reversible client
setup it yields `проверка ` in 2.04 ms. `desktop-launch.json` proves the test did
not inject GLFW_IM_MODULE and resolved the installed desktop entry. Receipts:
`kitty-menu-before-setup-v1` and `kitty-menu-after-setup-v1` under the remote
evidence root. The existing user process was not restarted. New menu launches
use the client's documented IBus option; future login gets an unset-variable
default. Explicit other input methods and user edits are preserved. This
changes client setup, not the output executor or text ownership contract.

This is one functional Tab result. The screenshot still reverses the whole
typed prefix, so universal presentation remains FAIL. It proves neither the
old user process, another terminal, Wayland, nor the full browser matrix.

## Source baseline

Baseline: public v1.0.91, commit
`513fb674d6b820adb9532c1c4c1ae91193009801`, checkout
`/home/ubu/projects/lay-browser-final-fixes-20261018`, styling branch
`codex/ime-muted-hint-20261019`.

Inline suffixes already carried IBus foreground `0x888888` and a suggestion-only
underline. Legacy auxiliary hints used plain `make_ibus_text`, losing color at
the payload builder. The GTK IBus candidate panel applies auxiliary IBusText
attributes. GNOME Shell's auxiliary callback copies only `text.get_text()` and
discards those attributes, losing color again at the shell renderer.

Verified read-only against installed local GNOME Shell 50.1 resources
`/org/gnome/shell/ui/ibusCandidatePopup.js` and
`/org/gnome/shell/misc/ibusManager.js` in
`/usr/lib/gnome-shell/libshell-18.so`. The popup belongs to IBusManager, not the
keyboard indicator. Primary sources:
[GNOME 50.1 candidate popup](https://github.com/GNOME/gnome-shell/blob/50.1/js/ui/ibusCandidatePopup.js)
and [IBus GTK candidate panel](https://github.com/ibus/ibus/blob/main/ui/gtk3/candidatepanel.vala).
Source inspection is not physical-client acceptance.

## Minimal implementation

- One Rust foreground constant supplies both inline and auxiliary hints.
  Auxiliary ranges use Unicode scalar counts, including non-BMP characters;
  empty hints have no attributes. Plain document and property payloads remain
  unstyled. Output calls, ordering, visibility and delivery routes are unchanged.
- The existing GNOME extension dims only IBusManager's auxiliary label when
  the actual engine is exactly `lay-ime-ru` or `lay-ime-us`. Read-only engine
  signals update opacity to 128/255 (approximately 50%), preserving the system
  theme's color and any already lower opacity. A text notification covers
  asynchronous initial engine discovery. Other engines and bus disconnect
  restore the original opacity. Disable disconnects listeners and restores only
  the opacity still owned by Lay. Missing private display actors leave extension
  initialization usable.
- The extension loader includes the helper in its existing fresh-module
  directory. The extension ZIP includes all JavaScript modules; shipping only
  the former loader/implementation pair omitted its imported helpers.

No polling, theme override, new process, input interception or change to
Tab, Space, Double Shift, focus, candidate generation or learning is added.
KDE uses the existing IBusText display contract. GNOME's private display actor
is version-sensitive and requires compatibility checks when Shell changes.

## Verification and boundaries

Remote source gate completed: 910 PASS / 0 FAIL (892 IME binary tests and 18 tray
tests), 156.1 seconds of recorded remote execution, including auxiliary publication/clear payloads,
unchanged typed-prefix styling and Unicode ranges. Exact receipt:
`/home/ubu/.cache/lay/development/run-t8n_7c4y/RESULT.json`.
GNOME opacity and lifecycle controls: 6/6 PASS; module load in GJS: PASS.
Receipt `/home/ubu/.cache/lay/development/browser-final-fixes-20261018/ime-muted-presentation-source-one/RESULT.json`
retains these successful components and the separate canon failure: the remote
mirror's old Git HEAD incorrectly selected historical decisions. No historical
record or guard was weakened. The final scope used the exact local diff against
the pinned 1.0.91 base and passed the unchanged decision and link validators.

Final receipt:
`/home/ubu/.cache/lay/development/browser-final-fixes-20261018/ime-muted-presentation-source-two/RESULT.json`,
recorded at `2026-10-19T13:48:45.809241+00:00`:

- Architecture canon, its refusal controls and the canonical AST-only graph
  update/guard: PASS. No rule, historical decision or baseline was weakened.
- Extension ZIP: PASS; version `1.0.92`, all nine JavaScript modules present
  exactly once and byte-identical to source. ZIP SHA-256:
  `9cf92adfc6f2dd1a5e8cb17a62e6aa49e6c33859682737ecf7edd4183db1963d`.
- GNOME 45.0, 46.0, 47.0 and 50.1 upstream manager/popup sources expose the
  display actors and engine signals used by the helper. This checks source
  compatibility, not successful rendering in those sessions.

The [portable source receipt](../../tech_debt/evidence/ime-muted-hint-source-20261019.json)
pins the changed runtime/test bytes, result hashes and component verdicts.

1.0.92 publication metadata aligns Cargo, extension and panel versions; it does
not change the tested runtime logic or reopen functional verification.

No local installation or global IBus restart is authorized by this change.
The user will install through the update menu. New presentation bytes have not
yet been physically accepted. The previous accepted native matrix remains
391 PASS / 37 FAIL / 4 BLOCKED over 432 KDE cases, including Firefox and Chrome
150 PASS / 10 FAIL each; it must not be relabeled as testing this styling change.
Fresh GNOME acceptance of the same runtime remains NOT_TESTED. The remote
worker currently has GNOME Shell 42.9, outside the extension's declared
45/46/47/50 support; a mock or older Shell render check cannot certify a full
supported GNOME extension session.

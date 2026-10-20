# lay2 — current working handoff

## Current checkpoint — final native packet, 2026-10-20

Full changed-byte native run is complete: 121 PASS / 86 FAIL / 0 BLOCKED /
144 NOT_TESTED out of 351. Firefox 85/75/0/0; Chrome 9/7/0/144 (focus
preparation abort after 16); GTK 10/0, Qt 13/2, fresh Kitty 3/0, Konsole 1/2.
Same runtime pins as the checkpoint below; matrix verifies unchanged PIDs and
hashes. This is not an overall native improvement over v1's 126/81/0/144.
Retain source/native/visual distinctions. Owning final table, scope and raw
receipt paths: `docs/development/remote-x11-input-2026-10-20.md`; compact
artifact `docs/development/release-1.0.94-native-summary.json`.

One additional bounded rapid-five diagnostic completed, no runtime edits:
first manual RPC succeeds, four following ones arrive but are refused by
`context_authority`. Debug flag restored. Root boundary is repeat manual
authority after the first conversion/layout handoff, not missing keys or
debounce. The exact defective propagation statement is not yet established.
Prepared minimal follow-up TD-161; do not remove the guard. Kitty appearance
remains visually PASS; universal appearance TD-160 remains OPEN.

Final source gates and remote installation are complete. Remaining work in
this accepted release packet: refresh documentation graph, commit/push/tag
1.0.94 with explicit native limitations, then verify the installed menu updater
against the new stable public release. Do not install the local runtime.
Do not start another source gate, native matrix or Harness review cycle for
documentation/generated-only changes. Actual setup GUI receipt is
`final-menu-setup-v1/gui-window.json`; live tray click remains NOT_TESTED.

## Current checkpoint — 2026-10-20 13:46 UTC

Final source evidence is complete as a documented continuation: 3,360
correctness/package checks, 11 serialized performance checks, both lint scopes,
canon/graph, helper checks and research/default release builds PASS. Original
failed attempts remain retained. Exact resource sampler correction is test-only;
limits are unchanged and measured RSS headroom is only 4 KiB. See the owning
remote-input document and `2026-10-20-exact-resource-snapshot.json`.

Remote installed 1.0.94 now pins engine
`c33f9f506a3805598f5838f67c6341c121e78ea038d3c0465e2444eb663775af`
and daemon
`1802bb5e816ebb5e8eac3d8de7baa26ed6702f00b4dec7145b7f2ac47ba5b6dd`.
`deployment-final-v1/deployment.json` records loaded bytes and preserved GUI
processes. Local user runtime remains untouched; update through the menu only.

`terminal-final-benchmark-v1/result.json`: 20 PASS / 3 FAIL from ordinary
desktop-launch Kitty, RFB input and an owned raw PTY. Twenty after-Space Double
Shift transitions pass (median 16.261 ms, p95 20.614 ms); ten Tab cases pass
(median 2.492 ms, p95 12.478 ms). Independently inspected `hint.png` confirms
ordinary `пров` and only auxiliary gray `ерка`. These are functional/visual
results for this client and environment, not universal acceptance.

Rapid-five active word fails after the first visible conversion at measured
162–164 ms gesture cadence. The following after-Space and moved-boundary rows
also fail; whether they share the preceding stale state needs independent
evidence. Retain these FAILs; do not count their missing transitions as speed
samples. A one-case diagnostic is prepared but not executed. No runtime edits
or new route follow from an unproven cause.

The broader changed-byte native matrix is running under the single remote
resource lease. Completed Firefox: 85 PASS / 75 FAIL / 160. Chrome and fresh
GTK/Qt/terminal rows are pending; do not splice unfinished rows into a completed
matrix. Receipt root remains `kde-install-control-20261020` on the remote runner.

Client setup/tray are installed remotely. Helper checks 28/28 PASS; actual
installed setup GUI opened and was independently inspected, then only that
owned window was closed (`final-menu-setup-v1/gui-window.json`). Clicking the live
tray action itself is NOT_TESTED. TD-160 remains OPEN for universal appearance.
Final documentation/graph refresh, publication and installed updater release
check remain pending. Do not start another broad gate or Harness review cycle.

## Current checkpoint — 2026-10-20 12:34 UTC

Remote installed engine e6d1ea292fe6be9f2353cda70675abccb9575e87187089dfcabf1e1effdb650f
now passes ordinary-launch Kitty functional 3/3 and prefix/suffix appearance.
Native receipt `terminal-kwin-manual-controls-v1`: both Double Shift directions
14.20/15.65 ms, Tab 1.62 ms, moved-boundary pair correct. Before Tab raw PTY
contains `пров`; independent screenshot inspection shows ordinary prefix and
only auxiliary gray `ерка`. These are tiny diagnostic samples, not median/p95.

The fix reuses the existing one-shot canonical qualification with the live
AdmissionToken field receipt before first-letter mode selection and before
manual authority admission inside the existing bridge guard. No new owner,
delivery route, synthetic acknowledgement or deletion proof. Native REDs and
5/5 source GREEN are retained in the presentation owner and canon decision
`2026-10-20-kde-ime-terminal-focus.json`. This certifies Kitty on KDE/X11 only.
TD-160 universal appearance remains OPEN for other client/platform mechanisms.
Final source graph, complete release gates, wider native evidence, setup/menu
installation, updater verification and publication remain pending. Local user
runtime is untouched and must update through the menu.

Previous-byte full native baseline `all-native-modes-v1/summary.json` completed:
Firefox 88 PASS / 72 FAIL (160); Chrome 10 PASS / 6 FAIL / 144 NOT_TESTED
(focus preparation abort after 16 rows); five GTK/Qt fields 25 PASS / 0 FAIL;
Kitty 2 PASS / 1 FAIL; Konsole 1 PASS / 2 FAIL. Total 126 PASS / 81 FAIL /
144 NOT_TESTED, 351 planned rows. Native RFB/DOM/PTY checks on KDE X11, not
physical evdev or GNOME/Wayland proof. Functional and visual verdicts separate.

IME setup/menu local changes: generic GTK/Qt/GLFW unset-only defaults, reversible
Kitty desktop setup, read-only diagnostics, a visible setup action on KDE/GNOME,
and a setup explanation after failed IME startup. Disposable-home tests and
actual offscreen Qt actions: 27/27 PASS (`setup-validation-v1.log`); these source
changes are not installed yet. No existing user window is closed; another
explicit IME selection is preserved. Automatic explanation needs final platform
scoping review. Final source graph/release/native/updater/publication gates
remain pending. Do not manually install the local user's runtime.

## Current checkpoint — 2026-10-20 09:57 UTC

Installed v4 engine e139bfa90ab341ea82b15cfaf74d480b4a90d2b9dea8c40685b04fc4a2d76ef1
now passes ordinary-launch Kitty Double Shift after Space in both directions
(164.76/163.32 ms) and Tab (2.70 ms); pair still FAIL. Native receipt:
`terminal-menu-replay-v4/result.json`. Trace `terminal-pair-menu-v4-trace`
shows pair candidate selected but verifier declines; no verifier changes.

A bounded candidate reuses the canonical KWin provider for the existing IME
Kitty focus qualification, which previously called GNOME only. Source is shared,
not copied; existing field receipt, caps, KnownStart and replacement checks stay.
Decision `2026-10-20-kde-ime-terminal-focus.json`. Initial compile failure is
NOT semantic RED; `kde-ime-terminal-focus-red-v2.log` is semantic RED, targeted
`kde-ime-terminal-focus-green-v1.log` passes. Combined isolated IME/daemon gate
is running (`kde-terminal-isolated-v1`). Candidate release/native proof pending.
Two daemon-specific consumer tests moved to focus_guard; provider security tests
stay with the shared canonical provider. No new owner/route or client ack.
Universal visual requirement and final release/native/updater/publication gates
remain pending. Do not install/restart the local user runtime.

## Current checkpoint — 2026-10-20 09:46 UTC

Ordinary Kitty launch mismatch is reproduced and repaired in the remote KDE
profile. The preserved user Kitty process lacked GLFW_IM_MODULE; the old test
fixture supplied it. A normal installed desktop-command launch, without an
injected variable, fails before setup (Tab inserts a literal tab) and passes
after setup (`пров` -> `проверка `, 2.04 ms). Receipts:
`kitty-menu-before-setup-v1` and `kitty-menu-after-setup-v1`; both use the same
installed v3 engine. Installer/uninstaller now own reversible client setup;
four disposable-profile checks pass. Existing user process is not restarted.
Universal prefix appearance remains visual FAIL; this functional result does
not certify it or an already-running terminal.

Replay v3 native remains 1 PASS / 2 FAIL. The new production-path regression
finds a cap-9 daemon-word-buffer transfer: its authenticated transfer has no
exact-V2 handoff, so the v3 condition still drops LegacyReplayV1. V4 retains
the one-shot guard on the existing authenticated owner/path/epoch transfer,
independently of exact-V2 metadata; exact-V2 expiry remains unchanged.
Semantic RED: `replay-daemon-transfer-red-v1.log`; GREEN: 2/2 and isolated
922/922 (`replay-daemon-transfer-green-v1.log`, `replay-daemon-isolated-v4`).
Release v4 build succeeded; deployment and changed-byte native proof pending.
Final graph, full release/native gates, updater and publication remain pending.

## Current checkpoint — 2026-10-20 09:28 UTC

Universal typed-prefix appearance applies in every system and remains visual
FAIL, tracked by TD-160. Do not treat functional source/native results as visual
acceptance. Local user runtime must still update only through the menu.

Replay suppression v2 is installed remotely; its source gate passed 921/921,
but owned Kitty still has Double Shift and moved-boundary FAIL, with Tab PASS
11.46 ms. The full trace identifies a second loss at the authenticated factory
activation, where `install_context_activation(Transfer)` discards LegacyReplayV1.
Production-path RED is `replay-activation-transfer-red-v4.log` (v1-v3 are invalid
fixture evidence). Two focused regressions and the full isolated IME candidate
gate pass: 2/2 and 922/922. Release v3 build is running; changed-byte native proof
is pending. See `remote-x11-input-2026-10-20.md` for pins and receipt paths.
Final graph, release gates, native matrices, updater and publication are pending.

## Current checkpoint — 2026-10-20 09:00 UTC

Universal hint appearance is mandatory in every application and system, recorded
in AGENTS.md, the common browser protocol, the presentation owner document and
TD-160. Fresh Chrome native Tab/control case passes functionally but its typed
prefix is visibly underlined: visual FAIL. Terminals also remain visual FAIL.
Do not count either as presentation acceptance or repeat discarded-attribute
experiments against unchanged clients. No prefix commit/ownership change made.

Decoder-v1 native trace now reaches Latin `ghbdtn`, then Space autocorrection
changes it back. First remaining transition: premature legacy suppression RPC
consumes the existing layout handoff before target engine activation. New
real-zbus test is semantic RED, 0/1; source candidate retains the original
bounded handoff. GREEN and changed-byte native proof are pending. The local
same-mode preflight refinement also needs its 10-test KDE transaction proof.
Detailed current evidence: `remote-x11-input-2026-10-20.md`. Do not publish yet.

## Current checkpoint — configured replay decoder mismatch (2026-10-20 08:43 UTC)

The complete `terminal-daemon-shared-dispatch-kitty-v1-engine/engine-trace.jsonl`
proves replay delivery, with Latin keyvals decoded back into the original Russian
word. KDE's configured replay preflight changes and verifies the desktop group
only; unlike the existing exact-tail layout transaction, it never switches and
checks the focused IBus decoder. This is the next first failed C09 transition.
The candidate reuses that existing pinned KDE layout transaction before any
Backspace, retaining the configured output executor, mutation preflight, optional
tail lease, focus/lifetime checks and one-shot restoration on preflight failure.
No client branch, new output route or authority migration is introduced.

The 150-ms lifetime probe is rejected: native Double Shift and pair correction
still fail, Tab passes; `terminal-lifetime-delay-probe-v2/restoration.json` proves
restoration. The decoder candidate's source and native gates are pending.

## Current checkpoint — terminal native failure and client styling (2026-10-20 08:36 UTC)

The 08:22 checkpoint below is historical. Installed shared-dispatch daemon is
`0ad86367196412deb6c53dfb5a3c8e062971658589c29b61af25192dbf3cc5a1`;
engine remains `4ec866ff4dafceff7add2013078d915a7a9dd561e2e06f753d22405f09fc82da`.
Owned regression is RED/GREEN and 26/26 pass, but real terminal Double Shift
still FAILS after the transport's first refusal was removed. The latest complete
terminal suite has 2 PASS / 7 FAIL, with a valid Konsole fixture. See the current
checkpoint in [remote input evidence](remote-x11-input-2026-10-20.md) for exact
receipts and the rejected diagnostic lifetime probe. Full trace subsequently
proved a KDE/IME decoder mismatch, not missing replay keys.
Whole-prefix reverse highlighting is reproduced in Kitty and Konsole; their
client rendering ignores per-span IME styles. Do not misreport it as text selection
or claim the gray auxiliary suffix fixes that prefix presentation.

## Current checkpoint — single-map latency and terminal isolation (2026-10-20 08:22 UTC)

Installed receipt `deployment-single-map-v1/deployment.json` binds IME
`4ec866ff4dafceff7add2013078d915a7a9dd561e2e06f753d22405f09fc82da`
and daemon `6e9b37a6dcb6c144d07597c16a776aa0a9d4da5241e24f28da2e2576819b9110`.
The single padded XKB SetMap keeps the generation fence and map verification;
25 owned Xvfb cases pass. The native Chrome rapid-five-after-Space case now passes
with transformation times 126.95/117.87/147.57/119.93/111.86 ms
(`single-map-v1-chrome-rapid/result.json`). This supersedes the earlier multi-map
FAIL experiments; it is one diagnostic row, not the full browser matrix.

Terminal native evidence uses owned windows and a raw PTY child, never a shell.
Kitty 0.21.2 requires its documented `GLFW_IM_MODULE=ibus`; with that environment,
Kitty and WezTerm accept Tab completion but still fail Double Shift and the moved
boundary pair. Konsole launch is BLOCKED by the fixture, not a product FAIL.
The exact daemon trace in `terminal-daemon-kitty-v1/daemon.log` at epoch
1792484032 shows DelegateDaemon -> ConfiguredBackend/Shared -> X11 output refused
because no isolated replay is active. The candidate requires the existing grab
for X11 manual dispatch, preserving the selected output route and evdev policy.
The manual-dispatch regression fails semantically in `shared-dispatch-red-v2.log`
and passes after the repair. Owned Xvfb is 26/26 in `shared-dispatch-green-v3.log`.
The initial compile failure (red-v1) is not semantic RED. A proposed SetMap
notification-count test was invalid: duplicate notifications and the controlled
old implementation produced the same observation; it is retained only in cache
as `invalid-map-notification-oracle.rs`, not as acceptance proof.

Chrome's whole-word underline is independently explained by its GTK preedit
consumer: it turns PANGO_UNDERLINE_NONE into a solid composition span and defaults
to whole-composition underline without spans. IBus GTK forwards NONE correctly;
Qt handles it correctly. No client-specific route bypass or premature prefix
commit is authorized. Actual selection and composition styling must be reported
separately. Remaining terminal presentation and pair-edit safety need native proof.

Evidence root: `/home/e/projects/lay-development-runner/kde-install-control-20261020`.
Final full gates, generated graph, final native browser/GTK/Qt/terminal runs,
installed updater readback and publication remain pending. Local user runtime
must not be installed or restarted. Runtime decision/output authority is unchanged.


## Current checkpoint — KDE rapid replay latency (2026-10-20 07:44 UTC)

Completion source proof is now GREEN: `stock-owned-completion-red-v1/result.json`
records the controlled semantic RED; `stock-owned-completion-green-v1/result.json`
passes 3/3 real-zbus cases; `stock-owned-completion-full-v1/SUMMARY.json` passes
920/920 IME correctness cases (3 performance cases are separate). Installed
`deployment-stock-owned-v4/deployment.json` binds engine
`4ec866ff4dafceff7add2013078d915a7a9dd561e2e06f753d22405f09fc82da`
and daemon `e1d8478b0020be4edcb566368842c3222d1e81c004b8d50c55ae7d277dedaf1f`.

Native rapid Chrome is still FAIL. The earlier retention-policy refusal is gone:
`chrome-ime-stock-rapid-v7/engine-trace.jsonl` and v8 show completed exact replays.
The next first failed transition is the KDE layout readout: D-Bus profile
`rapid-v8-bus-profile.log` measures getLayoutsList at 890.10 ms and 619.50 ms
against a 250 ms method timeout. KDED was waiting about 1030 ms for KGlobalAccel.
This establishes the delayed service boundary, not yet the cause of that delay.
The v9 syscall-only trace does not record text payloads and independently retains
the rapid failure. All temporary debug flags were restored.

Candidate experiment groups naturally aligned XKB map components, retaining
separate requests for byte-sized components with known generated-code padding
constraints. It keeps the same ownership/generation fence, original-input thaw,
map equality checks, layout owner and IME delivery route. Owned Xvfb v1 is an
invalid harness invocation (missing LAY_OWNED_X11_TEST), not a semantic RED.
Corrected `compact-map-owned-v2.log` is 25/25 PASS. Native effect remains pending.

Final native matrices, full release gates, installed updater and publication
remain pending. Do not install or restart the local user's runtime.


## Current continuation — stock native replay completion (2026-10-20 07:21 UTC)

Remote installed v1.0.94 engine is
`86635ee6aa486cbea15769a10e10282ba8820472b85bf6300ee417baf44f1c7e`,
daemon `e1d8478b0020be4edcb566368842c3222d1e81c004b8d50c55ae7d277dedaf1f`
(`deployment-stock-owned-v3/deployment.json`). Native Chrome hint/Tab/manual
roundtrip now passes 1/1 (`chrome-ime-stock-owned-v4/browser/result.json`).
Suppression semantic RED v3 and GREEN v3 establish the repaired second gate.
Readout-only full IME is 920/920; it does not cover subsequent completion edits.

Rapid Chrome five remains FAIL: all five gestures are recognized, while the
completed replay's KnownStart boundary is hidden pending SSI that the stock
client cannot supply. The failure cancels suppression, delays subsequent work
and can make the next isolation attempt busy. Fresh trace/daemon evidence:
`chrome-ime-stock-rapid-v6/engine-trace.jsonl`,
`stock-owned-rapid-diagnostic-v6/daemon.log`. Earlier v5 daemon log is empty;
unit journal collected with `-n` contains the actual events.

The scoped source change preserves only the existing locally tracked stock
manual source after complete native callbacks. Exact/observed clients still
require their existing receipt; no native reset ACK or learning proof is made.
Completion focused GREEN v1 is 3/3; controlled RED and the complete engine set
are running in `stock-owned-completion-proof-v1.log`. New completion bytes are
not installed yet. Final native matrices, graph, full gates, installed updater
and publication remain pending. Do not install/restart the local user runtime.

## Current continuation — stock-client manual source (2026-10-20 06:49 UTC)

`deployment-reviewed-v1/deployment.json` installs engine
`5f1412f52adf0efcd655a04c07be6a3b230cc7325f71de43f26e2a53f371e270`
and daemon `e1d8478b0020be4edcb566368842c3222d1e81c004b8d50c55ae7d277dedaf1f`.
GTK Entry/TextView and Qt Line/Plain/Rich pass 25/25 native rows in `reviewed-v1-*`.
Chrome now displays the hint and Tab inserts the completion, but its following
Double Shift fails. These are not a complete browser acceptance receipt.

Fresh `chrome-ime-reviewed-v1/engine-trace.jsonl` shows KnownStart after Tab and
all Shift callbacks, capabilities 41, no surrounding callback/snapshot and manual
outcome 3. The first failed transition is VisibleTailV4's second handoff gate,
which demands an external snapshot even for the current locally tracked word.
`stock-owned-source-red-v4/test.log` independently reproduces that rejection via
real zbus. v1-v3 are fixture failures, not semantic RED evidence.

Decision `2026-10-20-stock-owned-manual-source.json` narrowly permits that source
for the existing explicit manual handoff, retaining the authenticated word scope,
shared owner, expiry and tail epoch. It never invents surrounding observations,
client delivery acknowledgement or learning success. Scoped positive/negative
tests and the complete engine correctness set are running. Repeated replay still
needs native verification; do not infer it from this source edit.

Next: inspect `stock-owned-full-ime-and-release.log`, install only remotely with
an audited deployment, then run the exact Chrome case followed by complete native
browser/GTK/Qt evidence. Full gates, graph, updater and publication remain pending.

## Current continuation — review fixes and Chrome hint contract (2026-10-20 06:20 UTC)

Installed bytes remain the 05:51 receipt below; latest changes are NOT installed
or accepted in native clients. Fresh Chrome tracing establishes capabilities 41
with no surrounding snapshot: each letter commits and no hint is composed.
The scoped repair uses the existing owned literal preedit only after authenticated
Ready, with precognition enabled and a nonterminal client. It preserves UnknownStart
and exact-refresh, selection, observed-text, privacy and output contracts.
The broad prototype regressed 45/918 tests and was rejected; scoped IME correctness
is 918/918 in `stock-hints-full-ime/SUMMARY.json` (three performance cases separate).

Harness2 task `20261020T051543Z-451b2973` completed status/core ok, eight calls,
score 6/10 REQUEST_CHANGES. All 13 frozen input pins independently match. Its three
findings now have source fixes: same-connection generation-fenced X11 writes and
native XKB map copy, exact field/path/newer-epoch/focus settlement, and a 4096-event
output drain budget. Owned Xvfb is 25/25 in `review-fixes-owned-v3.log`. Controlled
violations reproduce the ID-reuse and event-budget failures; exact-settlement RED
is confirmed in `review-settlement-red.json`, with byte-exact source restoration.
This is source/protocol evidence, not acceptance of the new runtime bytes.

Next: finish daemon correctness, release build, remote atomic deployment, five
GTK/Qt fields and the complete Firefox/Chrome matrix with latency. Then refresh
manifest/graph, full gates, installed updater and release publication. Never install
or restart the local runtime; the user updates it through the menu. Remote evidence
root: `/home/e/projects/lay-development-runner/kde-install-control-20261020`.

## Current continuation — Qt focus fixed, browser diagnosis open (2026-10-20 05:51 UTC)

Installed 1.0.94 daemon SHA256
`f0b35ad98aea686fb9bae3682bb19e77429a364a1f976006d6d5545de18c2e4e`,
engine `4d7d1d6eb2849c051b6a90a5fdd99cd064056c0c3e9decdc0502f9e8117ee529`.
`deployment-focus-v4/deployment.json` preserves global desktop processes and
verifies the live XInput2 source 5/master 3 and KDE us,ru/index 1. All five real
GTK/Qt fields pass their five rows (25/25): Double Shift before/after Space,
rapid five targets plus decoder, visible-hint/Tab outcome and moved boundary.
Receipts `focus-v4-{gtk-entry,gtk-text,qt-line,qt-plain,qt-rich}/result.json`
under `/home/e/projects/lay-development-runner/kde-install-control-20261020`.
Screenshots still require independent visual inspection for these bytes.

The child-grab experiment is REJECTED_NATIVE_QT: it preserves core focus but
sends XI FocusOut/Grab/Inferior, revoking the IME lease. No child-grab remains.
The minimal repair retains the same-client replay focus through original-input
thaw, then removes the replay master. Original-master focus is never assigned;
held failed output is released first. 23/23 owned Xvfb regressions pass in
`focus-order-green-v1.log`. Replay creation preserves exact root XKB metadata.

Chrome trace diagnostics had used the daemon config rather than the distinct
engine LAY_CONFIG_PATH. Capture must use actual engine environment and a fresh
trace marker. The first corrected diagnostic failed before input due to /proc
permissions; v4 handles disappearing/inaccessible processes and is running.
Old copied traces are not exact-current causal evidence. Complete browser
matrix, source/full gates, graph, final review, updater, commit/push remain open.
Local runtime must not be installed/restarted; user updates it through the menu.
Harness2 task 20261020T051543Z-451b2973 remains running against 13 frozen files;
it excludes the latest focus-order correction. Inspect independent pins/report.

Updated: 2026-10-20T05:01:23.589893+00:00.

## Current continuation — replay resource lifetime (2026-10-20 05:01 UTC)

The first Chrome failure is now causal, not a timing guess. Native ordinary
`abc` is 0/2 with Lay's persistent extra master, 2/2 when only the Lay daemon
is stopped; Firefox is 2/2 in both states. Receipts: `plain-input-chrome-on`,
`plain-input-chrome-off`, `plain-input-firefox-on`, `plain-input-firefox-off`
under the remote evidence directory. Chromium TouchFactory selects the last
master keyboard and filters other device IDs. The old GTK/Qt 25/25 receipt
remains valid for its prior bytes only.

The new decision `2026-10-20-remote-x11-replay-master-lifetime.json` retains
persistent selection ownership but creates the output pair only during the
existing original-master Sync freeze; remove before thaw, with existing exact
IME tail settlement before successful retirement. No route/recognizer change.
The fresh slave-Sync/no-XIAllowEvents probe still loses originals, so it is not
a fallback. Idle single-master regression: RED old adapter, GREEN new. First
new run 7/21 PASS found unchanged hierarchy rows falsely revoked acquisition;
fix admits only own changed rows, rejects foreign changed rows.
`lifetime-owned-v2.log`: 21/21 owned Xvfb PASS. New hierarchy source regression
and 1.0.94 daemon/engine/CLI build are running in `lifetime-release-v1.log`.
The new lifetime bytes are NOT installed or accepted in clients yet. Current
installed 1.0.93 SHA values are recorded in the previous continuation below.
Native browser full matrix, renewed GTK/Qt proof, source/full gates, graph,
installed updater verification and commit/push are pending. Local installation
remains out of scope.

Harness2 lifetime review is running with 13 frozen files. This is a distinct
revision from the two completed persistent-master reviews; inspect report and
input pins independently before accepting. Do not infer native PASS from it.


Updated: 2026-10-20T04:41:22.618975+00:00.

## Latest continuation — native remote X11 input works in GTK/Qt

The current installed 1.0.93 development candidate passes all 25 actual RFB
GTK/Qt rows across Entry/TextView/LineEdit/PlainTextEdit/TextEdit. This includes
active/Space Double Shift roundtrips, all five rapid intermediate targets,
real Tab acceptance, and moved boundary `должн ыбыть ` -> `должны быть `.
Five root screenshots were independently viewed: muted auxiliary hint `ерка`
is visible for typed `пров`. These are native client results, not source tests.
Remote receipts: `native-final-*/result.json`, `native-final-proof/summary.json`
under `/home/e/projects/lay-development-runner/kde-install-control-20261020`.
Local contact sheet: `/home/ubu/.cache/lay/development/browser-final-fixes-20261018/remote-x11-input-20261020/native-final-hints-contact.png`.
Cold startup of the shipped script now passes without the private install wait;
source tests are 9/9 after two new RED regressions for transient KDE snapshots.
KDE map preserved; unrelated desktop PIDs and binary hashes unchanged.

Installed engine SHA-256:
`14685982b618c343cdc331c149abce27ced35361e25999222becd96792ea96ba`.
Daemon SHA-256:
`d0061fe174e43dc5f715d975a99692f4144467be1bbc953a288f6fba4b7e9086`.
Hermetic IME: 916/916; owned adapter tests: 21/21 after four RED regressions.
Stock IBus reserved sender panic, VNC level-modifier word-lease cancellation,
pointer-batch fences and enabled-controls admission are repaired in source.

Browser proof is NOT complete: Snap geckodriver cannot launch under the resource
cgroup; direct snap binary additionally requires GLIBC_2.39. A separate ordinary
Mozilla runtime is being prepared. Chrome native diagnostic receives X-server
keys but observes no field text; investigate the first missing transition before
running the complete ten-field matrix. Preserve these FAIL/BLOCKED diagnostics;
the historical 150/10 + 150/10 table does not certify current stock KDE bytes.
Pending: browser root cause/proof, final changed/full gates, graph refresh,
version/publication choice and commit/push. No local installation.

Harness2 USED: final task 20261020T034412Z-295879d7, status/core ok, 8 calls,
0 failed, no fallback. Score 8/10 REQUEST_CHANGES; its pointer fence finding
and the intermediate controls finding have source fixes and RED->GREEN tests.
Independent manifest verification checked all 25 frozen input files, zero hash
mismatches. Acceptance: `/home/ubu/.cache/lay/development/browser-final-fixes-20261018/remote-x11-input-20261020/harness-final-independent-acceptance.json`.
This establishes review use only; token/speed savings and native acceptance
are not inferred from that review. Two bounded review passes are exhausted.

## Current continuation — remote X11 input

The user requests completing, checking and pushing the pending repair while
asleep. Remote installation is authorized; local GNOME installation and global
IBus/Plasma restarts remain out of scope. The current owner is
[remote X11 input](remote-x11-input-2026-10-20.md). Earlier sections below retain
their historical evidence and do not certify new bytes.

- Revised adapter freezes the original master synchronously; originals remain
  in the server. Dedicated owned master/XTEST resources supply output through
  the existing daemon FSM/executor. No asynchronous capture recovery survives.
- Remote `master-native-tests-v12.log`: nine owned Xvfb cases PASS;
  `session-admission-v1.log`: one source case PASS;
  `input-access-native-v1.log`: 19 diagnostic cases PASS. Receipts live under
  `/home/e/projects/lay-development-runner/kde-install-control-20261020/`.
- First installation was rolled back because acquisition incorrectly depended
  on the layout controller being X11. KDE's controller remains KDE; the new
  condition uses the X11 session. Corrected release daemon SHA-256:
  `0835a80941679fd110a3916e102a0057c158b17e312702a1bf058975e6d85b7b`.
  `deployment-master-v3` installed it, preserving other processes. The first
  actual GTK/RFB Double Shift is FAIL (`gtk-real-rfb-v4`): typed text remains
  unchanged. Qt/browser acceptance for the adapter is still NOT_TESTED.
- Harness2 task `20261020T011522Z-8abba303` completed status/core ok with no
  failures/fallbacks. Its 3/10 rejection concerns the abandoned slave design,
  not the revised implementation. Current frozen-master review:
  `20261020T015956Z-5013c9da`, project lay2, gpt-6.1-sol/xhigh; pending final
  report. Its intermediate review is 7/10 REQUEST_CHANGES. The current repair
  addresses generation-safe teardown, startup revalidation, key-name
  invalidation, Pause mapping and smoke settling. Remote v14: 13/13 PASS;
  smoke observer: 4/4 PASS. Deployment v4 installed daemon SHA
  `cb5985f55900527aa7ed8ea76eb5d2747822feaa998f389308261f9fb1a61a80`.
- Real RFB GTK v6/v7: Russian seed is correct; first Double Shift FAIL.
  `focused.xkb` versus `seed-0.xkb` proves IBus RU -> KDE US/RU symbol-map
  replacement, unchanged physical names. Blanket map revoke killed the daemon
  and lost FSM state. Idle layout-switch regression is RED on the previous
  code (`master-layout-red-v1.log`, one selected failure). The repaired existing
  map owner preserves physical names and reconciles only owned output maps
  before an unheld frame. Names, topology, held-output changes and preparation
  races still reject. Decision and owning document are updated. Compile v15
  failed on AttributeSet::is_empty; corrected to iter().next().is_some().
  v16: 14/14 isolated native/source tests PASS. Release v5 is building.
- Next: inspect installation receipt, exercise the real VNC GTK field, then
  Qt hint/Tab and the browser protocol; resolve concrete first failures, inspect
  independent review, refresh discovery/architecture and run canonical checks
  before commit/push. No native-client acceptance or push is yet claimed.

## Checkout and active work

- Working checkout: `/home/ubu/projects/lay-browser-final-fixes-20261018`.
- Branch: `codex/ime-muted-hint-20261019`; base HEAD:
  `fd3c19fa11da8618b4ed4f684f5e0c6252014bcc`.
- Preserve the uncommitted work. The dirty changes do not inherit acceptance
  from the published base. Do not build the older `/home/ubu/projects/lay` as
  an equivalent runtime.
- Installation task complete: installed the verified startup diagnostic into
  remote KDE and exposed explicit daemon controls; see
  [the installation owner](kde-install-control-2026-10-20.md). The user now
  explicitly requests immediate installation of finished fixes on remote KDE;
  this does not authorize local GNOME installation or global IBus/Plasma restarts.
  Startup input-access diagnosis/recovery is documented
  in [the owning development note](input-access-recovery-2026-10-20.md).
  Installed processes and user fields are preserved. Earlier bounded owned
  Qt/GTK hint/Tab proof is scoped to its recorded environments. A fresh actual
  VNC-KDE GTK probe observed the Tab text effect, with hint publication still
  unproved; VNC Double Shift remains unfixed. Full stock KDE is not accepted.
  A fresh complete stock-KDE browser matrix is
  NOT_TESTED. Historical browser 300 PASS / 20 FAIL / 0 BLOCKED belongs to the
  modified IBus/KWin stand, not stock-KDE acceptance.
- Existing architecture/evidence owners remain
  [the input stability document](../architecture/firefox-input-stability-2026-10-05.md)
  and [the experiment log](td121-core-experiments.md). This handoff records
  session continuity; it creates no architecture rule or runtime authority.

## Remote KDE installation and missing keyboard — 2026-10-20

- **Installed** the verified daemon, shared diagnostic/resources, runtime
  controller and final KDE tray into remote KDE :71. Daemon PID 1163855,
  SHA-256 `1de1a316968f5ea5b1d1872d306e0341a2bc2b2a43ddf4217df7b4a6754ac8b2`;
  tray PID 1182223, SHA-256
  `6e7428877198d9a8961d93862cc959b6c9c43d1eb04bf18114af183f408a9eb0`.
  Only daemon/tray were restarted; IBus/IME/Plasma/KWin/user Kitty identities
  and executable hashes were preserved. Version is 1.0.93 plus an unpublished
  patch, not a newly published release. No local GNOME installation.
- KDE now has explicit start/stop/restart under **Демон Lay**. These commands
  target only the daemon unit. Five-second wait and useful failures, truthful
  retained-menu checkmark and unchanged full-runtime toggle are covered by
  six controller/actual-Qt tests; seven diagnostic UI tests and 18 Rust tray
  contracts also PASS. These are not physical Double Shift/Tab acceptance.
- Harness² **USED**, lay2 task `20261020T001156Z-ba776735`: status/core ok,
  gpt-6.1-sol/xhigh, eight successful calls, no fallbacks, frozen inputs 4/4
  unchanged. The pre-fix review score is 7/10; findings were independently
  repaired and controlled RED/GREEN verified, with no invented post-fix score.
- The installed startup diagnostic was visually observed in the owned window;
  it correctly reports **NO_KEYBOARD** and disables privilege setup. Current
  kernel input inventory has zero USB/HID keyboards, only excluded Lay output
  plus power/video/audio devices. x11vnc delivers to private Xvfb :71 without
  an evdev input source. An input-group grant does not repair this absence.
  The user confirmed **VNC-only input, no connected keyboard**. Stock x11vnc
  UINPUT was rejected by an isolated file-capture experiment: six control/Latin
  mappings worked, both legacy/Unicode Cyrillic я mappings emitted no events.
  No kernel input device was created, and VNC was not restarted or changed.
- An owned actual-KDE GTK field independently showed `пров` → `проверка `
  after Tab, cursor 9. This is one XTest client text-effect diagnostic, not
  visible-candidate, Qt, physical-input or browser-matrix acceptance. The
  separate hint panel was not proved. Its observation interval includes waits,
  so it is not latency evidence. **VNC Double Shift remains unfixed** because
  remote XTest keys do not reach the existing daemon gesture owner. No second
  IME recognizer or output-route bypass was added. Next work is compatible
  remote input acquisition with one existing gesture owner; permissions or an
  idle attached keyboard cannot translate VNC keys into evdev events.
- Follow-up isolated RFB/XI2 proof: X11 RawKeyPress/Release can observe a VNC
  Shift, but VNC and program-generated XTest share source identity. Evidence:
  `x11-observation-probe/result.json`; no runtime listener was added. Remaining
  proof must cover source ordering, feedback exclusion and required input
  isolation, not just receiving the keys. At 00:49:56Z the same ten relevant
  process PIDs and installed daemon/tray hashes remain present; receipt
  `x11-observation-preservation.json`. No further installation or restart.
- Installed menu updater entrypoint/resolved source and latest stable 1.0.93
  verified on remote KDE; local installed check also succeeded. Full current
  native browser matrix and physical trigger acceptance remain **NOT_TESTED**.
  Do not turn the historical modified-stand matrix into stock-KDE evidence.
- Evidence root:
  `/home/ubu/.cache/lay/development/browser-final-fixes-20261018/kde-install-control-20261020`.
  Owning note: `docs/development/kde-install-control-2026-10-20.md`.
  Preserve the recorded first deployment rollback and first repair-test
  failure; neither is a successful keyboard recovery. Canonical source graph
  refresh PASS: `graph-source-final-v2/RESULT.json`, six outputs and 730 Rust
  pins verified. `runtime-final.json` at 2026-10-20T00:40:14Z verifies original
  IBus/IME/Plasma/KWin/Kitty and x11vnc process preservation, expected
  NO_KEYBOARD, unchanged daemon/tray PIDs and restored user focus. This later
  evidence-only handoff note changes no executable source or runtime authority.

## Startup input-access recovery — 2026-10-20

- Implemented a shared GTK diagnostic, automatic once-per-daemon launch on the
  existing discovery failure, and **Диагностика → Доступ к клавиатуре…** in
  GNOME/KDE menus. Packaging ships the existing TD-146 helper and comparison
  resources. Explicit button plus normal Polkit authorization is required for
  provisioning; no additional input owner, listener or delivery route exists.
- First actual failure on remote KDE :71 is discovery of a supported keyboard.
  The only typing-capable event device is excluded `lay-virtual-keyboard`.
  VNC keys do not establish evdev keyboard input. NO_KEYBOARD disables the
  administrator operation; permissions cannot repair this topology. User Kitty
  separately lacks its version's `GLFW_IM_MODULE=ibus`; its window is preserved.
- Final remote source proof: 320 daemon checks, 29 diagnostic/access checks,
  27 release/menu checks and seven GTK/Qt UI regressions PASS. Packaging/public
  shell fixtures and GJS module loading PASS. Tests execute in isolated worker
  homes/Xvfb/session buses and never acquire privileges or alter user input.
- Review findings corrected: KDE preserves failed-child status before a fast
  re-launch; ENODEV/ENOENT/EIO do not masquerade as permission denial. The two
  corresponding tests fail on the frozen old implementations and pass now.
- Evidence root:
  `/home/ubu/.cache/lay/development/browser-final-fixes-20261018/input-access-recovery-20261020`;
  final tests `revision3/checks.json`, controlled RED plus current device probe
  `revision3/regression-proof.json`; daemon source receipt
  `/home/ubu/.cache/lay/development/run-0nlc0wf1/RESULT.json`.
- Harness² uses per-task `harness-600s.toml`: 600 seconds per model call rather
  than 240; global configuration is unchanged. Current actual task
  `20261019T234336Z-393df45a`, project lay2, gpt-6.1-sol, xhigh. The final static
  review is 8/10 with the P2/P3 findings above; status/core ok, eight successful
  calls, zero failures/fallbacks. Owner accepted the bounded review after reading
  the artifact and independently proving both repairs. Frozen inputs predate
  those fixes and do not review final bytes. Final report: task directory
  `report.json`; independent receipt: evidence-root `acceptance.json`.
  Earlier 240-second failure is retained. A separate attempt with contradictory
  read restrictions was stopped and rejected as unread, not accepted as review.
- Actual device-open preflight: 20 samples, NO_KEYBOARD, zero supported
  keyboards, uinput available; median 0.370 ms, p95 0.427 ms. This is not typing
  latency. Earlier real GTK screenshot proves rendering and disabled repair,
  not automatic user-session launch or physical Double Shift/IME Tab behavior.
- Canonical remote graph PASS: `graph-source-final-v2/RESULT.json`, six output
  hashes verified, 730 nongenerated Rust pins unchanged. Installed updater
  `--check` PASS on both machines: installed/latest stable 1.0.93. Exact runtime
  process hashes retained in `runtime-preservation.json`. Dismissing a busy
  diagnostic dismisses its eventual result; failed/timeout result delivery after
  dismissal remains untested and no automatic reopening is promised.
- No release, installation, privilege change, IBus restart or user-window
  automation. Actual startup popup, seat ACL acquisition and full native stock
  browser/Double Shift/Tab acceptance remain NOT_TESTED. The historical modified
  stand browser table must not be relabelled as stock-KDE acceptance.

## Current stock-KDE repair evidence

- Missing Qt5 `Qt.labs.platform` dependency on 192.168.3.94 is installed;
  real QQmlEngine component import/creation passed. Evidence:
  `qt-labs-platform-import-check-green.log` under the local evidence root
  `/home/ubu/.cache/lay/development/browser-final-fixes-20261018`.
  Existing Plasma/IBus were not restarted; applet reload is not certified.
- The direct external Engine.Enable A/B is INVALID: foreign-sender binding
  correctly cancelled the metadata observer. Do not loosen that guard or
  infer that an authenticated post-Ready snapshot request cannot work.
- One request was added after the existing current-owner Ready publication,
  using its latch, capability/sensitive checks and post-await live token.
  No new owner, timer, synthetic snapshot or edit route. Source proof:
  `stock-surrounding-ready-green-source-two/RESULT.json`, four focused PASS,
  902/902 selected IME PASS; 905 discovered, three performance exclusions.
- Candidate SHA-256:
  `ed48bcf70cda5b0c3059263562578957bf295457b2863c8b40189ddf09e54743`.
  New source tests only changed two cfg(test) fixtures, so exact previously
  built production bytes were reused after every other pin matched.
- `stock-ready-v6-screen-acceptance-20261019.json` binds exact candidate bytes
  and both screenshots. Both fields show gray auxiliary `вет` beside `при`;
  Tab produces `привет `, with observed latency Qt 4.5 ms and GTK 2.5 ms
  (one sample each; hint latency unmeasured). Earlier preedit-only hint FAIL
  was a reader error. Preserve v4/v5/v6 receipts for the corrected GTK focus
  child guard, premature Qt Tab snapshot and IBus popup keyboard grab.
  Normal candidate child 407179, parent stock IBus 3700659, survives cleanup.
- Independent cold-start self-SIGABRT on child 405181 remains OPEN;
  `stock-factory-lifetime-20261019.json` and strace establish the abort, not
  its cause. A later same-byte child survives; do not kill it just to retry.
- The complete guarded remote graph refresh passed at 20:25:49Z. Receipt:
  `stock-surrounding-ready-architecture-source-one/RESULT.json` under the
  local evidence root above. All six copied graph outputs match its hashes;
  source pins were preserved. This adds no native or release acceptance.
- Separate unresolved issue: the independent cold-start self-SIGABRT. Preserve
  the surviving child and the bounded native verdict. Full stock browser
  coverage and physical Double Shift remain NOT_TESTED. No global IBus
  restart, local GNOME installation or publication is authorized by this
  experiment.

## Current bounded Kitty repair — 2026-10-20 local date

- Read/search failure was a diagnostic path error (rg exit 2), not an engine
  crash. Use absolute paths in the accepted checkout; locate files before
  reading them. The corrected search returned 0. No input route was changed
  to repair a search.
- Real Kitty 0.21.2 and Plasma on private display `:71` lack
  `GLFW_IM_MODULE=ibus`. That Kitty version requires the flag to connect to
  IBus. Owned diagnostics without it received literal `при\t` and emitted
  no Lay callbacks. The user's Kitty window/text was not changed.
- The owned v4 diagnostic adds that flag to its own child only. It connects
  to the existing private IBus, but Ready clears a consumed first `п`:
  visible prefix becomes `ри`, auxiliary completion `тм`, Tab commits
  `ритм `. This is a native FAIL. Evidence root is the same local cache;
  see `kitty-native-diagnostic-v4-20261020.json`, prefix PNG, client log and
  trace delta. That v4 result belongs to historical `ed48bcf7…` bytes.
- New uncommitted fix preserves only acquisition-bound literal preedit:
  existing request/nonce/revocation/target/native-context identity plus local
  focus serial. No awaiting acquisition under the engine interface guard,
  new authority owner, KnownStart, or delivery route. Old-code controlled RED:
  `activation-preedit-red-source-one/RESULT.json`.
- The intermediate source gate passed eight focused and 910 selected isolated
  checks, but cold review found a Ready-during-rendezvous race (7/10 before
  correction). Two controlled tests reproduce it on that patch:
  `activation-preedit-red-source-race-one/RESULT.json`, both semantic FAIL.
  The second/final review pass prescribed immutable authenticated ingress
  provenance, early publication under reducer->stamps, and no successor
  revocation for literal completion. The subsequent additional Harness cycle
  below was explicitly requested by the user.
- The correction uses typed PendingLiteral in the existing bounded stamp
  store. Only its exact-header non-authoritative receipt can survive an owner
  change on the same connection. Ordinary key/lifecycle guards remain strict.
  Decision: `2026-10-20-native-pending-literal-preedit.json` under decisions.
  Ten focused and 912 selected isolated checks passed in
  `activation-preedit-green-source-race-two`, but its private candidate
  681919 (6687b7f4…) still failed owned Kitty v5. No global restart occurred.
- Current root is warm ordinary FocusIn: a retained source owner masks the
  target during tentative Transfer; a different-context reply converts the
  same request to SourceFree. The new receipt covers that canonical fallback
  and CompatibilityProperty while retaining SourceFree-only preservation.
  One valid held-reply RED is in race-three; its second failure was a Native
  fixture Get/Barrier error, now corrected. Thirteen focused tests and the
  isolated IME/build gate passed in race-four: 13 focused and 915/915 selected
  IME tests, 918 discovered and three performance exclusions. Exact release
  SHA-256: `955fdd5070bb83a1eec30ccf6d875bfff2e5c4b7faa6abe7dd1cfad09e8bd134`.
- That candidate is running as normal private factory child 746568, parent
  IBus 3700659. Only the previous private factory child was replaced, with
  a verified backup; IBus/Plasma and the user's Kitty were preserved.
  `activation-warm-preedit-private-candidate-20261020.json` records the change.
- Actual owned Kitty v6 now preserves `при`, shows gray auxiliary `вет`,
  and Tab commits exactly one `привет `. Trace confirms PendingLiteral before
  SourceFree Ready. Independent receipt:
  `kitty-native-v6-screen-acceptance-20261020.json`; Tab latency 1.200848 ms,
  one sample. The child's GLFW flag is still local to that child; this does
  not certify the unchanged user Kitty or fix its launch environment.
- Owned Qt and GTK on the same bytes also show `при` plus gray `вет`; Tab
  and clean Space each produce `привет `. Screen and actual field receipt:
  `stock-ready-preedit-v1-screen-acceptance-20261020.json`. Tab latency Qt
  4.9 ms, GTK 3.1 ms, one sample each; no hint-latency or p95 claim. The raw
  preedit-only reader's BLOCKED verdict is retained separately, because the
  hint is an auxiliary popup. Source/debug flags and prior focus were restored.
- User-requested additional Harness2 cycle is USED and completed:
  `20261019T215217Z-9876d10b`, lay2/gpt-6.1-sol/xhigh, status/core_status ok,
  eight successful calls, zero failed calls or fallback stages. Selected c1
  review is 7/10: no demonstrated blocker, with missing integration evidence
  in the frozen excerpts. All eight input pins (94,528 bytes) stayed intact.
  `harness2-lay2-warm-preedit-review-20261020/acceptance.json` binds the report,
  selected review and independent owner checks. The owner verified the omitted
  outer duplicate-publication guard, reducer-lock scope, Ready replace_owner,
  and actual observer-driven race tests; source/native receipts remain separate.
  The snapshot's two claimed REDs are corrected to one real failure plus one
  Native Get/Barrier fixture error. No frozen input was rewritten. Library
  entries remain zero; acceleration and token savings are not established.
- Harness² USED, project `lay2`, task `20261019T210733Z-1565f93b`:
  final/core status ok, eight calls, zero failed calls/fallbacks. Independent
  acceptance and all 12 frozen-input pins are recorded in
  `harness2-lay2-kitty-review-20261020/acceptance.json`. The snapshot predates
  the implementation. Static diagnosis is accepted; the deferral suggestion
  is not adopted, and there is no native, acceleration or token-saving claim.
  The project experience library is still empty.
- Graph refresh PASS at 2026-10-19T22:06:15Z:
  `activation-preedit-architecture-source-one/RESULT.json`. All 730 non-generated
  source pins still match the tested snapshot, and all six generated outputs
  match the receipt. This subsequent evidence note changes no runtime source.
- This bounded repair/review is finished. Preserve the user's old Kitty/text,
  IBus/Plasma, all earlier FAILs and the separate cold-start SIGABRT. Full stock
  browser and physical Double Shift/manual-transfer acceptance on these bytes
  remain NOT_TESTED. The mixed CompatibilityProperty-to-Native enrichment
  ordering after early literal input was not exercised. Do not infer broad
  KDE acceptance or repeat the finished proof without a changed concern.

## Harness² agreement for this existing session

The user's instruction applies specifically to the existing Kitty `lay2`
window, Codex session `01a115ea-09f2-7003-81c5-26037edf60eb`.

1. Receive the instruction after the active turn. Preserve the current work
   and session. Do not restart or replace them to enable Harness².
2. Read the authoritative integration handoff:
   `/home/ubu/projects/harness2/LAY2_HANDOFF.md`, together with its
   `LOCAL_STATUS.md`. Preserve other windows, Codex transport, Hindsight,
   IME delivery routes and architecture rules.
3. Check `harness2-codex doctor` before first use. The real route uses project
   `lay2`, model `gpt-6.1-sol`, effort `xhigh`, and the separate `lay2` library.
   Do not silently reduce effort or substitute another project key.
4. For the next suitable bounded Lay engineering task, use the actual adapter:

   ```bash
   harness2-codex run --project lay2 --task-file /absolute/path/task.md --input-dir /absolute/path/frozen-inputs
   ```

   Select a real question from the active work and freeze only its relevant,
   non-sensitive diff, contracts and tests. Request artifacts in `output/`.
   Preserve input hashes. Keep source excerpts small enough to inspect and
   allow recovery of omitted ranges if tool output is truncated.
5. Respect the launcher's existing `human-safe-run`, project locks and call/time
   limits. Do not retry blindly or create a demonstration task for each chat
   message. This review route does not install, restart, publish, type into
   clients or change the running IME.
6. Inspect the report, failures, fallback stages and actual artifacts. The
   owning agent independently checks the result. Model scores or tool exit
   codes do not establish source correctness, installed-runtime behavior or
   native-client acceptance. Preserve the affected proof and browser gates.
7. Report `READY_NOT_USED` after handoff/doctor only. Report `USED` only with a
   real `lay2` task ID, report path and independent acceptance outcome. Queued
   delivery and doctor are not actual Harness² use. Speed/token savings are
   unproven without a comparison.

## Verified use and acceptance boundary

- Doctor passed before first use and was rechecked while recording this
  handoff: Codex 0.163.0, saved login available, `gpt-6.1-sol`, `xhigh`.
  Recheck receipt:
  `/home/ubu/.cache/lay/development/browser-final-fixes-20261018/harness2-doctor-handoff-confirmation-20261019.log`.
- Current status: **USED** in the session above. Task
  `20261019T185450Z-46217ba4`; report:
  `/home/ubu/.local/state/harness2-codex/projects/lay2/tasks/20261019T185450Z-46217ba4/report.json`.
- Execution: `degraded`, core `ok`, eight calls, one 240-second intermediate
  timeout and fallback at `c1`. Selected review: 9/10, no demonstrated defect.
  The owning agent accepted only a bounded static review of the Unset-profile
  confirmation functions/call sites and five supplied controls after reviewing
  artifacts and source and checking frozen/current-source hashes. Timing and
  stale-request coverage gaps remain. No new native acceptance follows.
- Independent acceptance and readable outcome:
  `/home/ubu/.cache/lay/development/browser-final-fixes-20261018/harness2-lay2-unset-review-20261019T185444Z/acceptance.json`
  and `RESULT.md` in the same directory.
- Earlier task `20261019T184905Z-a3a6da3f` timed out without a final review and
  was rejected. Retain that evidence. Do not launch another cycle merely to
  acknowledge the repeated setup message.

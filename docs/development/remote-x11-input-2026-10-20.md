# Remote X11 keyboard input — 2026-10-20

## Publication and installed updater — 2026-10-20 14:01 UTC

Stable [v1.0.94](https://github.com/radislabus-star/lay-public/releases/tag/v1.0.94)
was published at 14:01:24 UTC from
`5b3c6c5d92ca8d27ac59d2422b64b3e3d04950ae`. The public main branch and
annotated tag resolved to that commit when checked after publication.
The installed local `lay-update --check` returned exit 0 at 14:01:52 UTC:
installed 1.0.93, stable 1.0.94, update available. Its resolved script is
`/home/ubu/.local/lib/lay/update.sh`; the installed GNOME menu invokes
`~/.local/bin/lay-update --menu`. No local installation or runtime restart
was performed. The remote installed runtime and all native verdicts below
are unchanged. This is release-discovery proof, not a completed local upgrade
or a live tray-click test. No functional checks were repeated for publication.
Structured receipt: [release-1.0.94-publication.json](release-1.0.94-publication.json).

## Final 1.0.94 evidence — 2026-10-20

Source and installed runtime are different proof scopes. The source composite
gate below passes; the complete changed-byte native run retains failures.
Machine-readable aggregation: [release-1.0.94-native-summary.json](release-1.0.94-native-summary.json).
All raw receipts remain under
`/home/e/projects/lay-development-runner/kde-install-control-20261020`.

Loaded engine SHA-256:
`c33f9f506a3805598f5838f67c6341c121e78ea038d3c0465e2444eb663775af`.
Loaded daemon SHA-256:
`1802bb5e816ebb5e8eac3d8de7baa26ed6702f00b4dec7145b7f2ac47ba5b6dd`.
Deployment receipt: `deployment-final-v1/deployment.json`, 13:33:45 UTC.
Input status is ready with `x11-xinput2`; source functional files match 122/122
(`source-final-functional-equality.json`). Runtime PIDs and hashes remain
unchanged throughout `all-native-modes-final-v2`, completed 13:47:28 UTC.

| Client | PASS | FAIL | BLOCKED | NOT_TESTED | Planned |
| --- | ---: | ---: | ---: | ---: | ---: |
| Firefox | 85 | 75 | 0 | 0 | 160 |
| Chrome | 9 | 7 | 0 | 144 | 160 |
| GTK Entry | 5 | 0 | 0 | 0 | 5 |
| GTK TextView | 5 | 0 | 0 | 0 | 5 |
| Qt line | 4 | 1 | 0 | 0 | 5 |
| Qt plain | 4 | 1 | 0 | 0 | 5 |
| Qt rich | 5 | 0 | 0 | 0 | 5 |
| Kitty | 3 | 0 | 0 | 0 | 3 |
| Konsole | 1 | 2 | 0 | 0 | 3 |
| Total | 121 | 86 | 0 | 144 | 351 |

These are owned native RFB input / DOM or raw-PTY observations on KDE/X11.
Firefox includes normal, private and fullscreen modes. Chrome aborts during
next-field focus preparation after 16 rows; its remaining 144 are NOT_TESTED,
not inferred successes. Qt line/plain fail after-Space roundtrip; Konsole
fails after-Space conversion and the moved-boundary pair. Per-client receipts
are `all-native-modes-final-v2/<client>/result.json`. Compared with v1's same
351-row scope (126/81/0/144), this run has five fewer PASS; it is not an overall
native improvement or universal acceptance. Reasons for every row delta are
not yet isolated; do not ascribe all of them to one new code change.

Firefox's 75 failures group as: active rapid-five 19, after-Space rapid-five 20,
after-Space roundtrip 18, typo/undo 10, Tab 6, EN/RU Space 1, moved boundary 1.
Successful ordinary active roundtrip is 30/30. Some email-field mismatches end
with the correct word but no trailing Space; retain their FAIL verdict until
the native observation and client normalization are independently classified.
Do not silently change expectations to improve the total.

Separate ordinary desktop-launch Kitty benchmark:
`terminal-final-benchmark-v1/result.json`, 20 PASS / 3 FAIL / 23 rows. Twenty
after-Space transitions: median 16.261 ms, p95 20.614 ms, max 21.272 ms. Ten Tab
cases: median 2.492 ms, p95/max 12.478 ms. Failed rapid-five transitions are not
successful speed samples. Independent inspection of `hint.png` shows ordinary
typed `пров` and only muted auxiliary `ерка`; prefix is already in the raw PTY
before Tab. This closes the measured Kitty appearance case only, not TD-160.

The isolated `terminal-rapid-diagnostic-v1` retains FAIL and restores temporary
debug configuration. Its five RPCs are one successful observed-terminal-suffix
conversion followed by four `context_authority` refusals with a live bridge
token. The trace segment is parsed records 1970–2154; earlier buffered fixture
records are present too. First failed transition is repeat manual authority
after the conversion/layout handoff, not gesture delivery or debounce. Exact
defective propagation statement remains unproved; record implementation work
under [TD-161](../../tech_debt/161-retain-observed-manual-suffix-across-layout-handoff.md)
without bypassing the guard. The fresh matrix's Kitty 3/3 includes the pair:
the benchmark's following failures do not prove a fresh independent pair bug.

Client setup and tray scripts are installed remotely; 28 helper tests pass.
Actual installed `lay-ime-setup --gui` opens and closes successfully with the
expected explanation: `final-menu-setup-v1/gui-window.json` and
`installed-setup-window.png`. Only that owned window is closed. The live tray
click itself is NOT_TESTED; action wiring is source-tested. Other explicit
IMEs/custom desktop commands are preserved. Existing user Kitty processes are
not restarted; generic environment defaults apply on next login.

GNOME, Wayland, physical evdev, WezTerm and all unobserved client/theme states
are NOT_TESTED by this packet. Local user runtime is untouched and must update
through the menu. No new delivery route, text owner, client acknowledgement,
deletion grant or verifier exception is introduced by the final qualification
correction. Universal Chrome/Konsole appearance remains OPEN under TD-160.

## Precise RSS sampling — 2026-10-20 13:23 UTC

The fresh gate passed 3,360 correctness/package tests, then failed one of 11
resource/performance tests: exact-authority RSS delta 15,420 KiB exceeded the
unchanged 15,360 KiB limit. Three bounded reproductions measured 15,096, 15,288
and 15,096 KiB with identical 7,928 KiB Pss_Anon deltas. The failed gate remains
failed; these repetitions do not erase it. The test-only sampler now reads
`Rss`, `Pss` and `Pss_Anon` from one `/proc/self/smaps_rollup` page-table snapshot
instead of using asynchronous `VmRSS` for RSS. All four resource limits remain
unchanged. Decision: `2026-10-20-exact-resource-snapshot.json`. Corrected proof
and all 11 serialized performance checks pass. The precise sample is RSS/PSS
15,356 KiB, Pss_Anon 7,932 KiB and repeat Pss_Anon 0 KiB. RSS headroom is only
4 KiB; this is not a memory-reduction or stability claim. The complete source
evidence is a composite continuation: 3,360 correctness/package tests, the
corrected 11 performance checks, both lint feature scopes, canon/graph,
helper/adapter checks and research/default release builds pass. The original
failed attempt remains in `release-gate-attempt-4`; corrected results are in
`precise-resource-canon.log` and `release-final-full-gate.log`. A temporary
continuation wrapper's missing `ROOT` is a runner failure, fixed before
resuming remaining checks; it is not a product test failure. No runtime
authority changes.

## Final source gate repairs — 2026-10-20 13:08 UTC

The preceding full source run passed 3,358 correctness/package checks and all
11 serialized performance checks, then stopped at lint inventory. This is
source evidence for that candidate, not native acceptance or a completed
release gate. Receipt: `release-gate-attempt-3/release-final-full-gate.log` in
`/home/e/projects/lay-development-runner/kde-install-control-20261020`.

Two test-support items caused new dead-code entries: an exact-handoff fixture
method was compiled into production, and the shared KWin provider's contention
helper had no engine-test consumer. The method is now test-only; a shared
regression asserts that an occupied provider returns `Busy` without waiting or
starting IPC. No warning baseline or runtime authority was changed.

Clippy then reported oversized native-resource enum variants. Input resources
and the larger X11 output resource now use `Box`, allocated once when opening
the resource. Gesture recognition, transport selection, frame ordering, the
executor and its validation are unchanged. Both feature scopes now pass lint
inventory and Clippy (`lay_lint_contract=PASS`). A fresh full source gate,
default release build and changed-byte native check are still required.

## Current checkpoint — ordinary launcher and authenticated replay (2026-10-20 09:50 UTC)

`deployment-replay-daemon-v4/deployment.json` pins installed engine
`e139bfa90ab341ea82b15cfaf74d480b4a90d2b9dea8c40685b04fc4a2d76ef1`
and daemon `2ed8a90908745c2895292bccec5423426512c74206a9bbb2cdf5722194c4dd3e`.
Source regression identifies cap-9 DaemonWordBuffer transfer without exact-V2
handoff: `replay-daemon-transfer-red-v1.log` is semantic RED. Existing transfer
authentication, field/path/epoch checks and bounded grant now retain V1's
one-shot suppression independently of V2. Two targeted and 922 isolated IME
tests pass; V2 expiry checks are unchanged.

Changed-byte actual RFB/raw-PTY `terminal-menu-replay-v4/result.json` is 2 PASS /
1 FAIL: Double Shift after Space passes twice (164.76/163.32 ms), Tab passes
(2.70 ms), moved-boundary pair still fails. `terminal-pair-menu-v4-trace`
confirms the pair candidate is selected and the verifier declines it; no safety
gate has been weakened. These are three owned Kitty rows, not a full matrix.

The user's preserved ordinary Kitty lacks GLFW_IM_MODULE. Earlier fixtures
explicitly selected IBus and did not certify its launcher. On the same v3
runtime, normal desktop-command RED/GREEN is recorded in
`kitty-menu-before-setup-v1` / `kitty-menu-after-setup-v1`: literal Tab before,
`пров` -> `проверка ` after reversible setup, 2.04 ms. Both resolve the installed
desktop command without a fixture IME variable. New menu launches are fixed;
existing user process remains unchanged. Screenshot still reverses the typed
prefix: universal presentation remains FAIL.

Next bounded experiment reuses the canonical KWin focus provider for the IME's
existing Kitty qualification, whose current observation calls GNOME only.
Decision: `2026-10-20-kde-ime-terminal-focus.json`; source/native results pending.
No route, executor, ownership rule, or client acknowledgement is added.

## Current checkpoint — authenticated activation suppression (2026-10-20 09:28 UTC)

Installed `deployment-replay-suppression-v2/deployment.json` pins engine
`713835332164ca1b5c81dfafcc3c005fb43ee7151e76976c9233a884e29d7ff1`
and daemon `2ed8a90908745c2895292bccec5423426512c74206a9bbb2cdf5722194c4dd3e`.
Its isolated IME gate is 921/921 PASS, but native owned Kitty remains 1 PASS /
2 FAIL in `terminal-replay-suppression-v2-engine/terminal/result.json`: Tab
11.46 ms passes; Double Shift after Space and moved-boundary correction fail.
The complete trace shows correctly decoded Latin replay followed by Space
autocorrection restoring Russian. No functional or visual acceptance follows
from the source gate.

The production transfer independently discards `LegacyReplayV1` in
`install_context_activation(Transfer)`. Semantic RED is
`replay-activation-transfer-red-v4.log`; v1-v3 are fixture setup failures and
are not semantic evidence. The regression now uses the actual manual bridge,
factory, source FocusOut/Disable, target metadata/FocusIn and activation marker.
Two suppression regressions pass in `replay-activation-transfer-green-v1.log`.
The complete isolated IME candidate gate `replay-activation-isolated-v3` passes
922/922; release build and changed-byte native proof are still pending.

Universal prefix/hint presentation remains OPEN/FAIL under TD-160. The user
reaffirmed that it applies in every system. No ordinary-prefix commit migration,
new text owner, delivery route or local runtime installation is implied here.

## Current checkpoint — suppression settlement and universal presentation (2026-10-20 09:00 UTC)

Installed decoder-v1 native trace now proves target Latin replay is decoded as
`ghbdtn`; the following Space autocorrection restores `привет`. Decoder repair
therefore reaches its target, but functional terminal Double Shift still FAILS.
Trace: `terminal-kde-decoder-v1-engine/engine-trace.jsonl`. The legacy suppression
RPC consumes the already prepared layout handoff before target activation;
the subsequent owner bind clears its protection. A semantic real-zbus regression
reproduces the loss (`replay-suppression-transfer-red-v1.log`, 0/1). Candidate
removes that premature consumption without extending any deadline. Expired,
unprepared and other-field controls are included; GREEN/native still pending.

Fresh Chrome `universal-hint-prefix-v1/result.json` has 1/1 functional PASS,
but `hint-text.png` underlines the whole typed prefix: presentation FAIL.
The user requires ordinary entered text and a muted suggestion everywhere, in
every system. Kitty/Konsole reverse highlighting and Chrome underline are not
accepted exceptions. See TD-160 for the measured rendering root and minimal
ownership-migration options. Attributes alone cannot prove visual repair.
No prefix timing/ownership, output route, verifier or local runtime was changed
for presentation. Full native matrices and publication remain pending.

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

## Current checkpoint — terminal delivery and presentation (2026-10-20 08:36 UTC)

`deployment-shared-dispatch-v1/deployment.json` records installed engine
`4ec866ff4dafceff7add2013078d915a7a9dd561e2e06f753d22405f09fc82da` and daemon
`0ad86367196412deb6c53dfb5a3c8e062971658589c29b61af25192dbf3cc5a1`.
The Shared/X11 isolation regression has semantic RED in
`shared-dispatch-red-v2.log` and 26/26 owned GREEN in `shared-dispatch-green-v3.log`.
This removes the pre-output refusal; it does not establish terminal acceptance.

The complete native PTY suite `terminal-shared-dispatch-suite-v1.json` has
2 PASS / 7 FAIL: Tab passes in Kitty and Konsole; Double Shift and moved-boundary
correction fail in all three terminals, and WezTerm Tab fails in that run.
Konsole now launches a real owned raw-PTY child, so its old fixture BLOCKED is
superseded. `terminal-daemon-shared-dispatch-kitty-v1/daemon.log` reports seven
Backspaces and seven replay events at epoch 1792484835–36. The initial trace
excerpt was misread as missing letters; the complete trace contains the replay.
It receives Latin `g/h/b/d/t/n` but the still-Russian decoder reconstructs
`привет`. The daemon's `done` line is not client acceptance. A diagnostic-only
150-ms lifetime probe did not improve the result; its v2 restoration receipt
confirms source, installed daemon and readiness restored at 08:39 UTC. No sleep
is retained. This supersedes the early-retirement hypothesis.

Native screenshots `terminal-shared-dispatch-{kitty,konsole}-v1/terminal/hint.png`
reproduce reverse highlighting of the entire entered prefix, with the gray suffix
in an auxiliary popup. Primary client sources independently explain this:
[Kitty 0.21.2 screen_draw_overlay_text](https://github.com/kovidgoyal/kitty/blob/v0.21.2/kitty/screen.c#L632)
reverses all overlay characters; its IBus consumer forwards only the preedit text.
[Konsole 21.12.3 inputMethodEvent](https://github.com/KDE/konsole/blob/v21.12.3/src/terminalDisplay/TerminalDisplay.cpp#L2285)
stores only the preedit string, and
[TerminalPainter](https://github.com/KDE/konsole/blob/v21.12.3/src/terminalDisplay/TerminalPainter.cpp#L614)
draws a cursor over its complete rectangle. Neither consumes the span styles in
this path. This is client composition rendering, not a selected document range.
Lay's gray suffix attributes cannot override those hard-coded client renderers.
No prefix commit, client-specific bypass, delivery-route or authority change was
made for presentation. Copies of the primary sources are retained in the local
`remote-x11-input-20261020/client-style-sources/` evidence cache.

Final whole-browser native proof, source gates, generated graph and publication
remain pending. The historical browser table remains 150/10 + 150/10; none of
these terminal or owned-server checks changes that table.

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
The new owned-Xvfb regression is pending RED/GREEN; red-v1 is a fixture compile
failure and must not be reported as semantic RED.

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

## Current installed evidence and stock manual source — 06:49 UTC

The reviewed v1 deployment installs engine SHA256
`5f1412f52adf0efcd655a04c07be6a3b230cc7325f71de43f26e2a53f371e270`
and daemon `e1d8478b0020be4edcb566368842c3222d1e81c004b8d50c55ae7d277dedaf1f`,
with the existing desktop identities preserved. Five GTK/Qt fields pass 25/25
native rows (`reviewed-v1-*`). Chrome's hint and Tab work; the following manual
conversion fails, so this is not complete browser acceptance.

Fresh Chrome trace `chrome-ime-reviewed-v1/engine-trace.jsonl` shows a live
KnownStart after Tab, no surrounding callback and ManualToggleV3 outcome 3.
VisibleTailV4 then rejects the source solely for lack of a snapshot. Real-zbus
`stock-owned-source-red-v4/test.log` reproduces that first failed transition;
earlier v1-v3 failed their fixtures and are not defect reproduction evidence.
The scoped manual-source decision reuses the current locally tracked word and
existing authenticated handoff, retaining owner/epoch/expiry and all observed
snapshot contracts. It does not confirm client delivery or learning. Native
repeated replay and final source gates remain pending for this new source edit.

## Review repairs and Chrome hints — 06:20 UTC

The installed focus-v4 bytes below remain unchanged. Fresh Chrome trace
`chrome-ime-fresh-v4/engine-trace.jsonl` has capabilities 41, no surrounding
snapshot, per-character commit and no preedit. Source now allows the existing
owned literal preedit after authenticated Ready only for precognition-enabled
nonterminal clients without exact refresh or a snapshot. UnknownStart remains.
Broad prototype 45/918 FAIL rejected; scoped source 918/918 correctness PASS
(`stock-hints-full-ime/SUMMARY.json`; three performance cases separate).

Harness2 task `20261020T051543Z-451b2973` is complete: status/core ok, eight
calls, score 6/10 REQUEST_CHANGES, 13/13 frozen hashes independently verified.
The three findings now have source fixes and regressions: generation-fenced
finite X11 writes, exact field/path/epoch/focus settlement, bounded event drain.
Direct native XKB copy replaces the external compiler under the server fence;
combined payload and overbroad type-resize flags were rejected with BadLength.
Owned adapter regressions are 25/25 PASS in `review-fixes-owned-v3.log`.
Controlled violations reproduce each defect; settlement RED uses the hermetic
D-Bus harness (`review-settlement-red.json`). Sources restored byte-exactly.
New binaries/native matrix, daemon/full gates, graph and publication are pending.
The historical browser table is not current candidate acceptance.

## Qt retirement ordering — 05:45 UTC

The child-grab experiment is rejected by native Qt, despite its 23/23 Xvfb
result: GTK Entry/TextView pass 10/10, but Qt Line/Plain/Rich fail 15/15
(`focus-v3-*`). The original master emits XI FocusOut/Grab/Inferior, which
revokes Qt's IME lease before the edit. Decision
`2026-10-20-remote-x11-subtree-grab.json` records the rejected experiment;
the implementation no longer creates a child window or relaxes FocusOut tests.

A fresh controlled probe (`focus-thaw-order-fresh-v2`) preserves both Qt's
widget focus and KWin's active window when the original master is thawed while
the already-directed output master still exists, then that output master is
removed. There is no original-master focus assignment or window activation.
The existing held-output failure path still releases/removes held resources
before its unconditional input thaw. All 23 owned Xvfb regressions pass
(`focus-order-green-v1.log`), including FIFO and focus guards. Release bytes are
installed and pass all 25 GTK/Qt native rows (`focus-v4-*`), with desktop PIDs
preserved (`deployment-focus-v4`). This does not establish browser acceptance.

Chrome boundary diagnosis remains open. The earlier engine-trace copy did not
have a fresh-session marker and cannot establish the current Chrome callbacks.
The source-before-clear probe still fails Tab; its incremental trace is empty,
so the daemon and IME use different LAY_CONFIG_PATH values. The corrected capture
uses the actual engine environment and a fresh marker before admitting events.
No complete browser matrix certifies this candidate.

## Qt focus retirement and Chrome boundary diagnosis — 05:35 UTC (superseded)

Lifetime v2 restores ordinary browser typing (2/2 Chrome, 2/2 Firefox) and
passes GTK Entry/TextView (10/10 native rows). Qt loses the active KWin window
after its first successful conversion. `lifetime-debug-qt-v3/field` records
the original master ungrab's core Nonlinear FocusOut/Ungrab pair; KWin ignores
the paired FocusIn/Ungrab and deactivates the window. Removing the replay
master alone does not reproduce the loss (`focus-removal-probe-v1`). The
same-client child grab preserves concrete core focus and the owned active
application (`focus-grab-child-probe-v2`). Decision
`2026-10-20-remote-x11-subtree-grab.json` keeps the original FIFO freeze and
existing output owner; no focus-restoration call or client bypass is added.
The new retirement regression is RED on v2 (`focus-thaw-red-v2.log`) and all
23 owned Xvfb regressions are GREEN (`focus-thaw-green-v1.log`). The first
`focus-thaw-red.log` failed fixture setup and is not causal RED evidence.
New bytes are being built; installed Qt acceptance remains FAIL until renewed.

Chrome's IME connection is present: the engine receives every managed letter
and Tab, but its boundary remains `unknown_start` without fresh surrounding
text (`chrome-ime-diagnostic-v1`). The stock Ready request already exists;
do not add a second request owner or manufacture an empty snapshot. A bounded
source-before-clear diagnostic will distinguish fixture activation ordering
from the client/engine activation defect. Browser rich text receipts must
retain raw NBSP while applying the protocol's existing visible-space matching.
No complete browser matrix certifies the current candidate.

All named receipts are under the remote evidence directory
`/home/e/projects/lay-development-runner/kde-install-control-20261020`.

## Layout metadata root cause — 2026-10-20 05:14 UTC

Installed lifetime v1 (1.0.94) restores ordinary browser input: Chrome 2/2 and
Firefox 2/2. Native GTK/Qt still reject most Double Shift operations before
deletion. The daemon reports `KDE layout index or service lifetime changed`.
`layout-property-probe-v1/result.json` establishes the first transition:
AddMaster changes root `_XKB_RULES_NAMES` from `us,ru` to the server default
`us`, while KDE's current index stays 1. Copying the original map onto owned
devices does not restore that property synchronously. KDE's delayed repair
does not make this a valid transaction.

The minimal resource-layer repair preserves the exact root property (including
absence) inside the short AddMaster server lock. No compiler, key frame, IBus
call or client wait runs under that lock; other clients cannot race a layout
write between snapshot and restoration. The KDE consistency checks stay strict.
The new regression fails on v1 (`layout-metadata-red.log`) and passes with the
repair; all 22 owned Xvfb tests pass (`layout-metadata-green.log`). This proves
server-resource behavior only. The repaired daemon is installed: daemon SHA256
`db154ffc185b4a1943b1390240b0c395ec4f0cc741743d82574e52b284084bc1`, engine
SHA256 `4d7d1d6eb2849c051b6a90a5fdd99cd064056c0c3e9decdc0502f9e8117ee529`.
`deployment-lifetime-v2/deployment.json` records coherent startup/readback and
unchanged IBus/KWin/Plasma/VNC/terminal processes. Native acceptance is running;
complete browser matrix, source gate, review and push remain
pending. Old GTK/Qt and browser counts do not certify these bytes.

The separate Harness2 lifetime review attempt `20261020T050107Z-9ea16585`
failed after its configured 240-second call timeout, with no review result.
It is not an accepted review. A bounded task-local timeout may be increased
for a new frozen revision without changing Codex transport or global config.

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


## Cold-start snapshot follow-up (2026-10-20 04:34 UTC)

Deployment of the final adapter exposed a first failed transition before IME
activation: KDE briefly reports list `[us]` and index `1` while the owned X11
devices appear. The failed installer rolled back its four replaced files;
`deployment-native-final/failure.json` and
`deployment-native-final-ready/readiness.json` retain both observations.
Waiting privately for the old map made deployment succeed, but is not a shipped
startup fix. The minimal product change will retry only changing/out-of-range
KDE snapshots, with a finite attempt count, and require list/index/list
consistency. Unsupported, malformed and permanently inconsistent snapshots
must still fail before source mutation. No KDE setter, background owner,
input-delivery route, or readiness claim is added. Source regression tests and
a cold-start native receipt are required separately.

## Problem and owner

The user confirmed VNC-only input on 192.168.3.94 without a physical keyboard.
Xvfb receives XTest keys directly; evdev cannot observe them. Changing device
permissions cannot repair that missing transition. The accepted source is
`lay-browser-final-fixes-20261018`, branch `codex/ime-muted-hint-20261019`,
starting HEAD `fd3c19fa` (1.0.93 plus the existing pending fixes).

C04 retains the daemon trigger FSM as the sole Double Shift recognizer. The
new facade changes acquisition and key-frame transport resources only. C01/C02
semantic authority, C05 typed dispatch, focused-window leases, IME delivery,
PhysicalInputGrab and verifier authority remain in force. No IME recognizer,
new lexical exception, daemon output worker or alternative edit owner is added.

## Selected mechanism and alternatives

The session-local XI2 adapter admits only one ordinary master keyboard, one
XTEST input slave and one virtual server keyboard without a Device Node.
Explicit device selection and other topologies retain evdev. The same daemon
loop, modifier/FSM state and typed dispatcher consume the physical XKB key
names. Replay raw events are excluded by their owned source identity.

During an edit XIGrabDevice freezes the original **master keyboard in Sync
mode**. It never calls XIAllowEvents: the X server owns every queued original
key until ungrab, including rejection and RAII/drop. A separate Lay-owned
master/XTEST pair supplies key frames during that freeze. This is necessary:
output on an existing slave of the frozen master also freezes. The replay
master has the currently focused window and verified current XKB group. A
bounded map copy uses xkbcomp on owned device IDs only. A changed symbol map
may be reconciled before output, with no synthetic key held; unchanged frames
do not invoke a compiler. Installation includes its package dependency.

Alternatives assessed before integration:

- Async slave grab with client capture/replay was rejected. The owned C probe
  demonstrated lost original input on rejection/close; source tests were not
  sufficient. Never revive it without new causal evidence.
- Sync/passive slave grabs and ReplayDevice did not restore the captured first
  key in this X server. The selected master freeze avoids taking that key.
- Using the existing virtual slave as output cannot escape the master freeze.
  This demonstrated failure justifies the additional owned master resource.
- VNC-to-uinput loses Cyrillic in stock x11vnc mapping and Xvfb does not consume
  evdev output. No dummy keyboard or kernel forwarding path is created.
- An IBus Shift recognizer or a whole-server grab during an edit is rejected:
  the former breaks C04; the latter blocks other X clients including IBus.

## Resource, concurrency and removal consequences

Two X connections, one owned hierarchy pair and an unmapped selection window
are session resources, not semantic owners. A brief X server grab serializes
startup claim and orphan cleanup only; it is released before map compilation,
focus operations, IBus calls or edits. No new background thread, timer or
unbounded history exists. Pending raw batches have the existing bounded event
vector; a queued ordinary prefix rejects edit acquisition instead of replaying
already delivered input. Every held key, including unknown keys, rejects a
new freeze. A finite drain budget cannot stall buffered protocol events.

The selection window prevents two active adapters. Connection death destroys
that window; a later adapter may remove only an orphan pair with Lay's exact
name and no foreign attached devices. Live output and cleanup validate device
identity as well as numeric IDs. Successful teardown removes the owned pair;
rejection unfreezes originals without an output retry. Hierarchy and physical
key-name changes revoke the capability. Ordinary KDE/IBus symbol-map changes
preserve the input binding. Before output, source/master agreement and all
owned maps are checked; only owned maps may be reconciled, and only with no
synthetic key held. A map change during a held output key rejects the frame.
Semantic comparison resolves used key types, since xkbcomp may compact unused
type indexes. Full per-device snapshots detect changes without assuming equal
compiler representation across different devices.

RSS/CPU and latency require native measurements. There is no new model, package
of lexical data, candidate ranking or learning cleanup. The existing evdev path
is preserved for physical keyboards and Wayland. Removing this adapter removes
only its acquisition/output resources and optional installer dependency.

## Evidence and rejected attempts

Evidence directory on the remote host:
`/home/e/projects/lay-development-runner/kde-install-control-20261020/`.
Local investigation/review directory:
`/home/ubu/.cache/lay/development/browser-final-fixes-20261018/remote-x11-input-20261020/`.

- Owned C probes `xi2-master-sync.c`/`xi2-master-fence.c` observed app=0 while
  originals were frozen, replay C=2 events, then original A/B=4 events in FIFO
  after thaw. The existing-slave output negative control stayed frozen.
- `master-native-tests-v6.log`: 6/7; group observation failed because the test
  application had not enabled XKB and received its legacy compatibility state.
  `master-native-tests-v8.log`: 7/7 with an XKB-aware application and independent
  core/replay/source state readback. No runtime workaround was added for this.
- `master-native-tests-v9.log`: 8/9; the new map-revocation test left the owned
  test server in DE before the group test. Each fixture now explicitly owns
  its US/RU setup. Preserve this failed setup receipt.
- Full raw-map equality across independently compiled devices and direct
  XKBSetMap copying were rejected (unused type-index differences / BadLength).
  Neither failed method remains a runtime fallback.
- Harness2 task `20261020T011522Z-8abba303` reviews the rejected slave design;
  its findings are not approval of the revised master-freeze implementation.
- `master-native-tests-v10.log`: 9/9 PASS; the release daemon compiled. First
  installed attempt was rolled back: acquisition incorrectly depended on the
  layout controller being `X11`, while KDE/X11 correctly selects the KDE
  controller. Acquisition now checks the X11 session independently, preserving
  the KDE layout authority. A session-admission regression covers KDE, GNOME,
  plain X11 and Wayland rejection. The failed deployment receipt is retained in
  `deployment-master-v1/`; it provides no live adapter PASS.

## Master review and actual VNC failure

Harness2 task `20261020T015956Z-5013c9da` produced an intermediate frozen-source
review at `executions/local-cx-agg1/.../base1/workspace/output/review.md`:
7/10, REQUEST_CHANGES. This is not final review acceptance. Its five findings
are addressed in the current implementation: bind replay device names to the
selection window's resource ID, subscribe before compilation and revalidate
the original capability afterward, subscribe to XKB key-name changes, map the
physical PAUS key, and require settled smoke results with a server core-key
release readback. The selection window already exists; its resource ID provides
the transport generation without another owner, counter or persisted state.

`master-native-tests-v13.log` retained a setup failure: Xvfb reset the map after
setxkbmap disconnected before the fixture opened a connection. The fixture now
keeps its application connection alive while installing its own map.
`master-native-tests-v14.log`: 13/13 PASS, including live predecessor/successor
ID reuse, names-only revocation, Pause mapping and the session-admission case.
`vnc-observer-v1.log`: four negative/settling controls PASS. These are isolated
source/transport checks, not a product acceptance result.

Installation `deployment-master-v3/deployment.json` installed daemon SHA-256
`0835a80941679fd110a3916e102a0057c158b17e312702a1bf058975e6d85b7b`,
retaining IBus, engine, Plasma, KWin, VNC and user Kitty processes. The real
`gtk-real-rfb-v4/result.json` is FAIL: VNC entered `привет`, but the first Double
Shift left it unchanged. Earlier setup attempts did not establish GTK focus;
a real RFB click in the owned field establishes focus. Those setup failures
remain preserved and do not count as product results.

`gtk-debug-rfb-v1` exposed a separate startup race: a KDE map update during
copying caused native open to fail, then evdev waited indefinitely with no
keyboard. For an otherwise eligible native session, startup now returns the
error to the existing service supervisor; unsupported topologies retain evdev.
This changes retry selection, not gesture or delivery authority. Fresh installed
client diagnosis is still required for the subsequent release daemon.

Acceptance at that checkpoint: implementation and isolated test evidence only. The first
installed real RFB GTK Double Shift is FAIL; Qt/GTK hint/Tab for the new adapter,
ten-field Firefox/Chrome protocols and latency are pending.
Physical evdev, remote Wayland, other VNC servers and multi-master topologies
are NOT_TESTED. Historical browser counts must not certify this adapter.

## Normal layout replacement was incorrectly treated as device loss

Deployment v4 installed daemon `cb5985f55900527aa7ed8ea76eb5d2747822feaa998f389308261f9fb1a61a80`.
`gtk-real-rfb-debug-v6` and v7 reached an actually focused GTK Entry, selected
`lay-ime-ru`, and entered the expected Russian seed. The first gesture remained
FAIL. Journals show daemon termination at `X11 keyboard map changed on device
3`. The v7 `focused.xkb`, `seed-0.xkb` and `failure.xkb` establish the first
failed transition: the map changes from `pc+ru` to `pc+us+ru:2`, keeping the
physical key names. Treating this normal KDE/IBus operation as transport loss
discarded the existing FSM's word state before the gesture.

Decision `2026-10-20-remote-x11-layout-map-refresh.json` separates physical
input identity from mutable symbol maps. It does not grant edit permission or
change any IME route. The former blanket map-change test is replaced by an
idle RU/multigroup transition proof and a held-output negative proof; names-only
and topology revocation proofs remain required. The added positive regression
must fail on the previous implementation. New source/native results are pending.

The earlier missing-engine observation was not reproduced by the bounded
activation trace: ordinary `ibus engine lay-ime-ru` started the configured
candidate successfully. Manual startup also stayed alive. This is no basis
for claiming an IME startup fix or restarting IBus. The candidate component
still runs SHA `955fdd5070bb83a1eec30ccf6d875bfff2e5c4b7faa6abe7dd1cfad09e8bd134`;
installed engine bytes differ and require a separate final deployment record.

## Focus-preserving isolation and final frozen-source review

Final Harness2 task `20261020T015956Z-5013c9da` is **6/10 REQUEST_CHANGES**,
not the intermediate 7/10 above. The task completed status/core ok with eight
successful calls and no failed calls or fallbacks; its frozen source was not
changed. The report is at
`/home/ubu/.local/state/harness2-codex/projects/lay2/tasks/20261020T015956Z-5013c9da/report.json`.
It is a review of frozen inputs, not approval of subsequent code. Findings and
independent repairs:

- R1: generation-specific device names plus server-serialized validation and
  disposal prevent a stale Drop from releasing/removing a successor's XI IDs.
- R2: initial and refreshed map admission compares interpreted key types,
  symbols, modifier/action/behavior mappings, compatibility interpretations and
  relevant interpretation controls. Compiler-only explicit flag metadata is
  normalized across devices; full per-device snapshots still detect its change.
- R3: clear owned lock/latch state, copy the verified original group, and read
  back both owned devices; retain planned pressed Shift. A regression seeds
  CapsLock and a group latch and independently checks both devices and output.
- R4: smoke receipt requires settled text and a late final-state check plus
  server-held-key readback. Transient-match and late-mutation controls fail.

`master-native-tests-v19.log`: 16/16 isolated checks PASS. v17 was an inherited
Wayland test-environment failure; v18 exposed xkbcomp's equivalent explicit
metadata representation. These attempts are retained, not product acceptance.

The installed v5 GTK/RFB trace `gtk-real-rfb-debug-v10` enters `привет` but
fails before deletion: `exact IME tail retention policy changed`. Independent
`gtk-focus-grab-probe-v2/result.json` proves the causal difference: grabbing
the focused GTK window preserves widget focus, admitted engine/context and
tail epoch/text; grabbing root produces GTK FocusOut and unknown context.
No IBus restart or route change was used.

The regression initially rejected all core focus notifications. Diagnostics
v4/v5 show core MPX NotifyGrab Out/In even when original focus and grab window
are identical, with no XI2 FocusOut. The final regression therefore observes
the actual XI2 focus boundary, includes a root-grab FocusOut negative control,
and retains original-input freeze, output and FIFO thaw assertions. It does
not infer GTK acceptance from core bookkeeping. The suspected wrong-device
focus query was disproved: core and explicit XI focus matched the fixture.

`master-native-tests-v21.log`: **17/17 PASS**, including the new negative
control, modifier/action disagreement and lock/latch cases. The adapter now
grabs the concrete current focus and rechecks it after acquisition. It still
uses Sync mode, never consumes queued original events, and does not add an
IME gesture recognizer or edit owner. Decision:
`2026-10-20-remote-x11-focus-preserving-isolation.json`.
Installed real GTK/Qt/browser proof for these new bytes remains pending at
this checkpoint; historical browser totals do not certify them.

## Distinct KDE keyboard and foreground service lifetimes

Deployment v6 installed daemon
`8bacd50672a75427d70cc5a223aba5ecad7f26f7caa19f789332508e6f6e33d9`,
preserving IBus/IME/Plasma/KWin/VNC/Kitty identities. Real GTK/RFB v11 still
FAILs, but the first failed transition moved: the exact tail lease survives
the focused-window grab, then KDE handoff refuses before deletion with
`KDE keyboard and foreground service owners differ`. The text remains intact.

Independent live D-Bus GetNameOwner/GetConnectionUnixProcessID observations:
`org.kde.keyboard` -> `:1.34`, PID3701072, `/usr/bin/kded5`;
`org.kde.KWin` -> `:1.7`, PID3700656, `/usr/bin/kwin_x11`.
The existing two layout transaction adapters incorrectly require one process
to supply both services. Their existing snapshots now capture both unique
owners separately and require both lifetimes to stay stable. Keyboard calls
remain directed to the captured keyboard owner; exact focus checks, mode,
engine, mapping, group, intent lifetime, restoration bounds and edit authority
are retained. Decision:
`2026-10-20-kde-distinct-layout-foreground-lifetimes.json`.

## KDE/IBus compete for the XKB map: preflight

Installed v7 is daemon `992126a84efc30757012be4a678f61519e5410460eceda608d4841c2c25ceace`
and IME `bafec16d36e36ee23e10111104bbd483e9094015894b48e256f086a17789a56f`.
`gtk-real-rfb-debug-v12` advances past service ownership but refuses before
deletion: `KDE layout us is absent`. The independent focused GTK receipt
`native-gtk-entry-v1/result.json` reads `getLayoutsList=[ru]`, index 0 and
`lay-ime-ru` before and after the refused gesture. Outside the field KDE reports
`[us,ru]`. IBus `use-system-keyboard-layout` is false; Lay's engine descriptors
request their individual XKB layout. Thus normal IBus activation replaces the
dual KDE map. This also explains unavailable Latin control shortcuts during
fixture cleanup; subsequent rows with failed setup are not independent feature
failures.

Before another runtime edit compare these routes:

1. Let KDE remain the XKB owner using IBus's existing
   `use-system-keyboard-layout=true`, with the already configured KDE RU/US
   list (9/10). Prove this bounded session setup causally before integrating
   installer repair. Preserve engine names, source selection and edit routes.
2. Change the exact handoff to allow whole-map replacement after engine
   activation (6/10). This cannot cover the existing owned-preedit operation
   which deliberately keeps its engine. It adds map-transition/rollback
   complexity and would not repair ordinary KDE shortcuts.
3. Rewrite engine descriptors to use a shared dual-group layout (4/10). IBus
   caches component metadata; it affects GNOME and existing aliases, and would
   require a larger integration change/restart. Rejected here.

Chosen next proof is option 1 on the dedicated remote session, preserving
before/after configuration and all unrelated processes. There is no candidate,
lattice, rank, learning, package or feedback change; false edit authority must
remain refused by the unchanged transaction. No new runtime owner/cache/queue,
timer or fallback is introduced. Existing layout deadlines and CPU/RSS hot path
remain unchanged; setup only writes the existing setting and uses KDE's normal
configuration reload. A failed/missing readback remains setup failure, never
PASS. A setting change can affect other IBus engines in this user session and
must be documented as KDE integration behavior, never applied on GNOME. Keep
configured additional layouts, variants/options and the existing selected
engine; do not replace user source lists. If the causal proof fails, restore
the prior setting and map before another design. Installer/update persistence,
future package compatibility and removal of the setup hook require explicit
source tests plus installed native acceptance; a diagnostic setting alone is
not a shipped repair.
Source and installed acceptance for this repair are pending at this checkpoint.

## IBus wire sender crash: compatibility preflight

Measured first failure after the map/owner repairs: the installed v7 engine
aborts on the first actual RFB Double Shift. `engine-parented-diagnostic-v1`
keeps its diagnostic parent alive throughout the gesture and records SIGABRT
and the zbus 5.15.0 `Invalid field reconstruction / Invalid unique name` panic.
`engine-parented-wire-v2/wire.strace` independently records the triggering
IBus signal: `/org/freedesktop/IBus`, interface `org.freedesktop.IBus`, member
`GlobalEngineChanged`, sender **`org.freedesktop.IBus`**, body `lay-ime-us`.
The preceding ordinary method replies/callbacks use `org.freedesktop.DBus`.
The wire bytes retain the original sender; this is not an inferred timeout,
manual-parent death, unavailable keyboard, or a client text failure.

Root: zbus_names 4.3.2's Value conversion constructs a name which its later
string validation rejects. zbus caches that field and panics during header
reconstruction; release `panic=abort` kills the whole IME. Stock IBus 1.5.26
`bus_ibus_impl_emit_signal` uses its IBus service name for these signals.
The current upstream zbus_names 4.3.5 still has the same validation boundary;
a blind version bump is not a demonstrated fix.

Design comparison before production edits:

1. Vendor the exact existing small zbus_names 4.3.2 crate, retaining its
   upstream license, and add an explicitly enabled compatibility feature for
   the one reserved IBus daemon sender (9/10). Keep the exact original string,
   existing authenticated IBus connection and all SenderBindings role checks.
   A wire regression must reproduce the panic on the unpatched dependency and
   reconstruct the actual signal after the patch. Invalid/arbitrary service
   senders remain rejected. This is the selected bounded compatibility repair.
2. Fork zbus header Sender from UniqueName to BusName (5/10). This models the
   wire more generally, but changes a public type through message builders,
   matching and consumers, expanding the accepted surface on every bus. The
   larger fork adds maintenance and upgrade cost without evidence it is needed.
3. Stop subscribing to GlobalEngineChanged and infer changes elsewhere (2/10).
   Rejected: it removes lifecycle evidence, creates stale-result races and
   another inference route, and does not repair parsing other IBus signals.

Consequence analysis: no candidate generation, lattice retention, ranking,
learning, feedback or model-package/delta authority changes. Runtime latency
deadlines and rollback bounds stay unchanged; validation adds one fixed-length
comparison with no per-event allocation, timer, queue or cache. No new edit,
gesture or observation owner is introduced. The parser accepts the actual
reserved IBus daemon name, not arbitrary well-known clients; authenticated bus
sender assignment plus existing bootstrap-bound sender roles still decide
authority. Normal session-bus daemon names and colon-prefixed unique names
must keep their existing meaning; wrong sender/path/member/role and stale
connection observations must remain refused. A future package update cannot
silently remove this contract: Cargo.lock pins the vendored source and the
wire regression covers it. Source archives/build/update packaging must include
the vendor directory. Expected regressions to exclude are broader name
acceptance, sender normalization, invalidation bypass, wrong feature wiring,
and omission from public source packages. Rollback is one Cargo patch/feature
and the vendored directory, together with its compatibility decision; remove
the fork once upstream carries the same tested wire support. Proof scope is
wire parsing/role controls, complete existing IME checks, then installed native
GTK/Qt Double Shift, hints and Tab. Source PASS is not client acceptance.

### Reserved-sender source proof

`ibus-wire-red-v1.log`: exact recorded GlobalEngineChanged frame on the
unpatched dependency reproduces the header reconstruction panic (one pass,
two failures). `ibus-wire-green-v1.log`: all three wire/name-boundary checks
PASS after the opt-in patch. `ibus-wire-ime-focused-v1.log`: the existing
hermetic correctness lane selects 915 IME checks; all PASS.

The earlier raw whole-target invocation is retained as
`ibus-wire-ime-all-v1.log`: 916 pass, two failures caused by shared test
process environment/trace and asynchronous usage-writer state. The adapter
source explicitly requires process/data isolation. No privacy code or test
assertions were weakened; the authoritative focused runner uses existing
hermetic isolation. This is a source verdict only. v8 release build and native
client acceptance are pending.

Second frozen Harness² review: task `20261020T034412Z-295879d7`, project
`lay2`, gpt-6.1-sol/xhigh, 600-second per-call bound, selected current adapter,
KDE owner and wire compatibility sources. Running is not an accepted review.

### KDE source startup preflight

Measured baseline: ordinary IBus profile activation with
use-system-keyboard-layout=false replaces KDE's dual map with a single layout;
with true the dual map survives. Selecting lay-ime-ru does not itself move
KDE from US to RU. The private fixture's explicit group selection is setup,
not a shipped repair. Receipts: kde-system-layout-setup-v1.json and
gtk-debug-rfb-v12/v13 under the remote evidence root.

Selected design (9/10): keep the existing runtime-control source-selection
owner. On explicit KDE IME enable, preserve KDE's system keyboard map through
the existing IBus preference and choose the Lay profile matching KDE's actual
current named layout, without changing any KDE group/list/variant/options.
Recheck the current named layout around the normal IBus selection and refuse
a missing/unsupported/ambiguous map. Existing GNOME activation stays intact.
Alternative (5/10): add an asynchronous activation-to-KDE-group job in the IME
worker; this would require a new bootstrap authority and race user changes.
Alternative (3/10): rewrite KDE to a fixed [us,ru] map; it destroys additional
user languages and variants and is rejected. Baseline has a demonstrated
map/profile mismatch and cannot serve as successful setup.

Consequences: no candidate/lattice/rank/learning/feedback/model-package delta
changes, per-key work, cache, text mutation or gesture owner. The bounded
startup-only DBus reads add process latency, not typing latency; every call
is limited to one second. The IBus preference is intentionally persistent in
the current user session configuration so later engine selections keep KDE's
map. Failure to write/read it aborts IME enable before changing the source.
Read/parse/map failures and a concurrent named-layout change must fail rather
than choose a guessed profile. The existing user-initiated source setter and
its readback retain authority; there is no KDE setter in this repair.
GNOME and daemon-only controls must not run this preparation. Updater file
installation does not invoke it or restart IBus. Package/removal cost is only
existing shell code; remove it if upstream IBus/KDE supplies this startup
contract. Proof: execute the real shell against isolated command stubs for
US/RU/reordered/extra layouts, unsupported/ambiguous/read/write failure and
GNOME/daemon separation, then native remote startup/readback. Rollback removes
these startup functions; preserve the user's map and record the preference.

### v8 source and native checkpoint — 04:10 UTC

- Engine v8 installed at 04:03:05Z, SHA-256
  bb4ef5f033abb25247fc86aa88ec3919ee43f05e4f1692e283846323857e3007;
  deployment-engine-v8-absent/deployment.json records readback, backups and
  preserved unrelated process identities. No global IBus restart.
- check-changed-wire-v8-fmt.log passes the changed-source gate: 3297 correctness
  and 36 package checks. This precedes the later KDE startup-script changes.
  Physical Double Shift ownership checks pass 4/4; test-lane manifests were
  refreshed from actual discovery, without known-failure additions.
- native-batch-v8.json records actual RFB input in five GTK/Qt field kinds:
  15 PASS / 10 FAIL / 25. All active/Space roundtrips and rapid-five rows pass;
  hint/Tab and moved-boundary rows fail. These are partial native receipts,
  separate from the historical browser matrix and unit-test totals.
- The hint actor expects a document preedit; the source publishes candidate
  display through IBus auxiliary text. Keep the original failed receipt and
  inspect actual panel visibility and Tab effect independently. Boundary
  trace records prefetch_not_ready at Space. No correction authority change
  is justified by this observation alone.
- Source KDE startup tests pass 7/7; frozen baseline produces 12 failing
  assertions. The new script is not installed yet. Native browser acceptance,
  installed startup acceptance, final review and push remain unfinished.

### Final observed failures and review preflight — 04:19 UTC

The VNC wire records ISO_Level3_Shift press, Cyrillic letter press, modifier
release, then letter release. The existing Alt-completion branch treats both
modifier edges as an arbitrary client edit and clears the exact preedit start
and Space worker. This explains cancellation before Space even after a 300 ms
pause. Candidate generation is not the first lost layer: the input-frame lease
is destroyed before the correction result can be consumed.

Selected repair (9/10): retain an admitted word across the zero-text modifier
preparation and the used-as-modifier release; existing unhandled shortcuts,
standalone failed completion, focus/layout changes and admission gaps still
revoke it. Keep Alt/AltGr completion on a standalone release and its existing
output contract. Add a wire-order regression plus a client-shortcut negative
control. Alternatives: remove AltGr completion (5/10, changes the public key
contract); add VNC-specific delay/retry (2/10, hides cancellation and adds races).
No new owner, timer, worker, lexical rule, delivery route or privacy exception.

Harness task 20261020T034412Z-295879d7 completed: 8 successful calls, zero
fallbacks, frozen inputs unchanged; final score 8/10, REQUEST_CHANGES. It finds
an accumulated pre-click key prefix crossing the pointer epoch. An intermediate
static review also identifies omitted enabled XKB controls in fingerprints.
Both are bounded adapter defects, not evidence of a native wrong-field edit.
Selected fixes: retire only adapter observations preceding the pointer fence
(including acquisition-pending observations), preserving X server delivery;
include enabled controls and their interpretation parameters in canonical
snapshots, excluding only transport metadata and repeat timing. Add owned-Xvfb
negative controls before production changes, then rerun the fixed adapter set.
These changes strengthen existing leases and do not change C04 recognition.

The four new owned-Xvfb controls reproduce the review defects before repair:
adapter-final-red.log has 17 PASS / 4 FAIL. The same fixed set after repair is
21/21 PASS in adapter-final-green.log, including server delivery preservation,
pending-prefix retirement, an empty-key acquisition fence and enabled Overlay1
admission rejection. No user desktop input was used by these source probes.

The modifier regression fails at the worker Ready assertion, which establishes
the lost lease even when the reconstructed frame still compares equal. The
bulk log modifier-red/logs/bin-lay-ibus-engine.log contains that valid failure.
The outer focused run is not an accepted full receipt: its final source-closure
check correctly rejected an overlapping test-file synchronization. The final
rerun freezes the complete source until it exits. Existing standalone failed
Alt-release and confirmed-reset Alt-completion controls remain unchanged.


## Installed native evidence — 2026-10-20 04:41 UTC

- `final-ime-focused/SUMMARY.json`: hermetic IME 916/916 PASS;
  `adapter-final-red.log` 17/21 then `adapter-final-green.log` 21/21.
- `native-final-proof/summary.json`: cold shipped startup PASS, exact KDE
  map retained, unchanged runtime/desktop process identities throughout proof.
- Five `native-final-<kind>/result.json`: 25/25 actual RFB GTK/Qt PASS, including
  moved boundary without an added wait. Hint suffix visibility independently
  reviewed in all five `hint-root.png` images; the fixture marks visual proof
  separately from real Tab text acceptance. No physical evdev keyboard tested.
- Stock-browser proof remains incomplete. Snap launch/GLIBC failures are
  setup BLOCKED. Chrome X-server key receipt with empty native field is an
  unresolved diagnostic FAIL. Neither is replaced by source checks.
- Harness2 final task `20261020T034412Z-295879d7`: independent 25-file manifest
  hashes match; second/final static score 8/10 REQUEST_CHANGES. Pointer-fence
  finding addressed with the four added RED->GREEN owned-Xvfb tests. Frozen
  inputs/reports and independent acceptance remain in the private evidence path.

Runtime authority has not changed. These receipts establish only the listed
installed hashes, five field kinds and this X11/VNC/KDE session. Other systems,
Wayland remote input, and updated browser acceptance are not established.

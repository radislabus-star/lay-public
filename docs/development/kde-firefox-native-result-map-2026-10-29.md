# KDE Firefox: completed native output awaiting client receipt

Status: source and release build PASS; both candidates loaded, native acceptance
in progress. The preserved first preedit-space fix was installed as
IME1670433/e163afd765fb13d8abc8d4de2a83003a3b2b1c5a2ccb60d51f899086f4b47ccf,
daemon374525/eb6098b445c48ae5a681fb8b1d33051e28a5da3646d90baa4cd47b09a7228de5.
Canonical HEADb835ca478198d6cb765a2246329d5646966c118c, source1304files/
e8de03875180fc7fa0117fdf48726d224293a049434870c6ba578e4e3ce86c9d.

## First failed transition and comparison

The private ordinary-input native trace in
CROOT/kde-firefox-space-reflow-modes-v1/private/firefox/row-1-native-trace.jsonl
records complete native deletion/insertion before effect-scope cancellation770,
final actual text snapshot776, then reverse-toggle context_authority refusal814.
The source result is client-unverified; exact scope was removed before that
actual receipt could settle its existing Reset lineage. Normal rich eight-v1
also fails a later gesture. Do not merge these with the fixed Tab reflow case.

GNOME's existing completion/Space confirmation is unchanged. GNOME's normal
local IME execution differs from this KDE remote replay's output settlement;
copying another delivery route is not a repair. Old waiting-only settlement
was rejected: all reads remained unknown until its unchanged deadline. A new
owned-fixture forwarded-release C experiment also gave20P1F1B, not an overall
improvement; it is rejected and production libraries/launchers are unchanged.

## Dependency map and bounded new read

1. `src/bin/lay_daemon/exact_ime_tail_replay.rs` emits the existing authorized
   deletion/insertion, checks focus and awaits the existing actual-field result.
   Its cleanup currently cancels the exact scope on any unverified result.
2. New `src/bin/lay_daemon/kde/firefox_replay_receipt.rs` selects only actual
   KDE/Firefox through the canonical backend/KWin observation, exact current
   window identity and real executable. Other clients/desktops retain cleanup.
3. `src/bin/lay_daemon/layout_controller.rs` -> `layout_controller/ime_bridge.rs`
   supplies one text-free `NativeReplayProcessedV1` observation. No mutation,
   alternate transport, fallback, larger wait or output retry.
4. `src/bin/lay_ibus_engine/bridge.rs` -> `window_interaction/authority.rs`
   admits that read to the current target engine without mutable engine access.
5. `src/bin/lay_ibus_engine/tail_memory.rs` checks the paired exact scope,
   original epoch/path/layout, actual processed native press count, complete
   mirrored tail and original expiry. It returns a boolean, never a field
   receipt, edit permission or client-success/learning acknowledgement.

Only after successful output/focus checks, an unverified final result may retain
the existing scope when this read proves every native press was processed.
Return client-unverified and release input/output isolation. The later actual
snapshot still passes the unchanged C03/C05/C07 verifier. No C03 validator or
word-completeness rule is relaxed. Completed scopes cannot consume another key:
the existing process_exact_replay_press retires them before ordinary input.
Original expiry/focus/input cancellation still applies; no new timer/state owner.
Incomplete/expired/mismatched/missing reads retain the existing cleanup.

## Exact source seams and gates

Source RED-v1 exposed two missing test-module enum imports; RED-v2 exposed a
fixture using the default uinput configuration rather than managed IME input.
These are preparation failures, not semantic RED. With explicit managed IME
setup, RED-v3 processes all six Backspaces and six native ASCII keys and gives
the intended `native presses complete without a client receipt` failure. Its
other three controls pass. No native runtime was replaced in these experiments.

Source GREEN-v1:4/4 targeted IME checks and7/7 targeted daemon checks PASS.
Complete affected source lanes:1127/1127 IME and330/330 daemon PASS. Five existing
performance tests and26 existing ignored daemon tests are not counted as PASS.
Canonical discovery retains every old row and adds exactly two IME metadata
proof tests plus one KDE/Firefox policy test. Source closure after that test
registry update:1305files SHA45efaccc7a36dea3b6845d92dbd69005f49f38cf9b66d9a4d3febecad6620030.
Graph/architecture PASS. Source checks remain separate from native acceptance.

Release build: both binaries, 0 warnings, 74.917 seconds. Archived Cargo
compiler-artifact executables bind the 1305-file source above to daemon
15fcc650f2aad3f3ea30d4152011adcd07971ae500f1737db96b3732ae8b715e and IME
31a7b4be84ad7e8ff38730346f808dcdf3b29958552816a11bb2e2469fc09d2d.
Installed/loaded candidates: daemon PID1894209 and IME PID1893290. Input ready;
IBus3168326, tray3228081, manual Firefox988548 and exact config SHA remain
unchanged. Only the idle IME factory and existing daemon unit were replaced.
Build and deployment receipts are CROOT/kde-firefox-native-result-build-v1/
result.json and CROOT/kde-firefox-native-result-deployment-v1/result.json.

Initial native control: 4/4 PASS, ordinary text/rich/plain/iframe-rich, each
after Tab with two real Double Shift gestures. Debug traces contain six true
NativeReplayProcessedV1 reads, proving the new retention branch was exercised;
they are not counted as client confirmation. Actual native/DOM result checks
pass separately. Config restored; protected runtime identities unchanged.
Receipt: CROOT/kde-firefox-native-result-short-v1/native/firefox/result.json.
Repeated-toggle/decoder/held-Shift set:19PASS/2FAIL/1BLOCKED of22. Text active
reverse refuses after the next-letter Backspace; plain post-Tab-eight refuses
Tab before the first gesture. Neither failed row reaches the new processed-result
read. Email remains BLOCKED by native/DOM disagreement. Retain every raw row in
CROOT/kde-firefox-native-result-eight-v1. Three-mode controls were not run after
that failure. Next bounded preflight is
`kde-firefox-owned-preedit-observation-map-2026-10-29.md`; no full acceptance yet.

The new owning ADR declares the above six existing files, new KDE module,
canonical test manifest and necessary bridge-contract test registration. Earlier
first-fix scoped paths remain allowed but should not be reworked. No C adapter,
GNOME/Chrome/terminal path, ranking or physical trigger is changed. The existing
text emission and layout-control calls stay intact; post-output cleanup is the
explicitly changed seam above. First prove processed-versus-confirmed semantics
and safety negatives, then targeted/full affected binaries, warning-free builds,
exact loaded bytes and the retained native control. No public release implied.

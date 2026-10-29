# KDE X11 exact replay: focus lost across input acquisition

## Current outcome — installed, bounded native PASS

Final report: `kde-x11-grab-focus-2026-10-29-verification.json`.
Daemon2374910 SHAae77e705ba6795cfb99f1b074b027a70e2f7277c81f480380d969880fd07d644
is installed/loaded on remote KDE:71. Release0warnings,67.005s, source1307files
SHA50938ebe30ad78da2d90389893b75d35273a80017e799fb018c949a02ec10ebf.
IME2067215/bad527..., IBus3168326, tray3228081 and actual user's Firefox988548
were preserved. Config6e62eab3... unchanged; KDE/IBus coherently Russian.
No GNOME installation/restart, commit, push or public release. Base stays1.0.111.

The SAME existing Firefox process/profile, owned window67109872:10 unique
native scenarios PASS,0FAIL/0BLOCKED. Input/textarea/rich each cover8 active
toggles,8 after-Space toggles and Tab plus2 toggles. The original second-word
after-Tab rich failure additionally passes8 gestures. Total62 target/decoder
checks,7 held/released-Shift pairs and4 actual Lay hint/Tab acceptances.

Keep the original7PASS/2preparationBLOCKED receipt. Only those two rich setup
slots were repeated after recording the native empty-editor <br> terminator;
raw newline/caret0 remain in the report. Actual nonempty targets have no hidden
stripped terminator. Product code was not changed for this observation issue.
The final second-word target is actual `привет привет NBSP`, caret14, unselected.

Timing is native release to first observed stable target, polling included:
unfinished-word median18.690ms/p95199.821ms (24 samples across3 fields), after
Space186.456/214.356ms (24), after first-word Tab201.693/240.826ms (6).
Tab accept median19.339ms/p9520.423ms (4). Exact second-word-series measurements
are in the final report. These are bounded samples, not universal performance.

Full historical matrix306P55F79B/440 was NOT rerun/recounted. Other applications,
private/fullscreen and physical HID are not certified by this same-process
normal-window report. Two daemon perf tests were not executed. Earlier native
FAIL receipts, rejected experiments and worker setup errors remain evidence.
No further source changes required for this bounded fix. User acceptance is
separate from these tests; do not resume the rejected C idle-refresh proposal.

## Original preflight and progression

Preflight before runtime edits. One mechanism, remote KDE/X11 only. Preserve
GNOME, correction, common admission/verifier, layouts and delivery contracts.
Canonical base b835ca478198d6cb765a2246329d5646966c118c; keep the retained patch.

## Evidence and current owner

CROOT/kde-live-input-recovery-v1/native-same-session-wire-v4: same user's
FirefoxPID988548, owned textarea, active composition roundtrip PASS19/18ms.
No config/debug change for this case. This is not all-window acceptance.

`native-same-session-space-wire-v1` and `native-same-session-space-provider-v2`:
after Space the first toggle FAILS. Native field/window remains focused and
unchanged. ManualToggleV3status3; first VisibleTailV4 supplies exact `привет `,
RU, original epoch and authenticated context receipt. KWin returns the same
actual Firefox UUID/class/PID. After native acquisition and before layout/output,
the next VisibleTailV4 is unknown-context with a changed epoch and empty source.
Existing cleanup cancels the original handoff. No need to loosen that rejection.

Path: manual_trigger_runtime/ime.rs captures the common lease; then
trigger_dispatch.rs::run_exact_ime_tail_replay creates PhysicalInputGrab;
native_input/x11.rs::Input::grab freezes the original master keyboard and
activates the existing owned replay master; exact_ime_tail_replay.rs verifies
the same lease before layout. The first invalid transition is across acquisition.

The existing thaw path in x11.rs already retains same-client focus via its
owned output master. The freeze test selects core FOCUS_CHANGE but asserts only
XI2 FocusOut absence. First strengthen that existing test to reject core FocusOut
too. Genuine RED is required before accepting a freeze-focus implementation.

## Bounded alternatives

1. Reuse the existing same-client replay-master focus preservation at acquisition,
   selected only by canonical KDE policy: provisional8/10. No new text owner or
   output route. Needs failure cleanup, held-key/map/focus guards and queued
   input/repeated gesture tests. GNOME policy must keep its existing behavior.
2. Grab only the remote slave device:5/10. XI2 detaches a grabbed slave and changes
   modifier/delivery semantics. It broadens this task into event routing; defer.
3. Ignore FocusOut or restore the old word because text matches:1/10, rejected.
   It weakens common admission and permits stale/foreign-field authority.
4. Skip input isolation:0/10, rejected by the existing executor contract.

RED-v3 proves the core event on dedicated owned Xvfb: the strengthened existing
test fails with `transport acquisition emitted core FocusOut`. Source1306files
SHA268e5aa494d253ceb619ce06bf17124e149f0a5e2a214a5426ec08a40f7cfd41.
RED-v1 executable-path and RED-v2 X11-environment setup errors are retained and
are NOT mechanism evidence. Actual RED-v3 native test time0.064s; no installation.

Choose option1. KDE-only `kde/native_input_focus.rs` supplies the canonical
desktop predicate and reuses the checked owned-master focus operation. The
existing Input::grab implementation takes a testable policy argument internally;
its production caller uses current_config's existing desktop resolution.
For KDE, activate/direct the SAME owned replay master to the concrete existing
client focus before freezing the original master. Other desktop ordering stays
unchanged. Preserve every existing held-key/pending/map/focus/epoch guard.

The existing ungrab cleanup must also retire a prepared owned master if the
subsequent original-master grab fails; never pretend a grab succeeded. No new
lifetime owner: the existing Input/ReplayMaster/drop path owns preparation and
retirement. This covers failed creation, competing grabs, source/focus changes
and queued input. No keyboard output is permitted during preparation.

The existing replay.maps readiness slot MUST stay empty until the original
master is actually grabbed and its existing boundary checks pass. Prepare the
same map proof on the acquisition stack, then move it into that existing slot;
never expose an active-but-unisolated output capability. Add a native negative
for pre-freeze output and a competing-grab cleanup test. This is a requirement
of the selected design, not permission to loosen Output::prepare.

Allowed shared seams: native_input/x11.rs and its x11_tests.rs, plus the new
KDE leaf and existing retained exception paths. The stricter native focus test
explicitly exercises KDE preservation; full native transport controls run with
canonical KDE and GNOME configurations to prove the other ordering unchanged.
Any additional seam
requires updating this map/scope before editing. No canon/test weakening.

## Proof and rollback

Run strengthened test on dedicated owned Xvfb, not the user's display. Retain
RED/error logs, exact source/compiled artifact. If the core event hypothesis is
not reproduced, reject it and do not write the proposed runtime fix. Once RED
is proved, settle the minimal implementation/desktop selection before coding,
then native transport controls, full daemon source lane and0warning build.
Install only an idle matching daemon and verify loaded bytes; preserve IME,
IBus, tray, original Firefox/window/config. Short same-process active/Space/Tab
controls precede any broader client acceptance. Existing failures stay recorded.

This is field integration, not autocorrection-only. Source PASS, installed bytes
and physical client acceptance are separate. No release, push, agents or Harness.

## First implementation source result

GREEN-v1 short:3/3 owned-Xvfb semantic checks PASS, including the former RED,
pre-freeze output refusal and competing-grab resource/foreign-owner preservation.
Native test times0.064/0.064/0.115s. Source1307files SHA
262107f46b7f83bcf5b03e346bce78c686bbe1c196e1d5e8900a3c03d1f8f7a3.
Architecture/field-scope checks pass; Cargo test compilation2.74s. No runtime
installed. Next: all native transport controls in canonical KDE/GNOME policies,
full affected daemon source lane, then0warning release and actual client proof.

GREEN-v2 full: native transport28/28 with canonical KDE config and28/28 with
canonical GNOME config; daemon correctness/package330/330 PASS. The3 short
checks overlap these native totals. Two new ignored native tests are registered;
all previous canonical rows retained unchanged. Two daemon performance tests
were not run. Final source1307files SHA
50938ebe30ad78da2d90389893b75d35273a80017e799fb018c949a02ec10ebf.
Receipt:CROOT/kde-x11-freeze-focus-source-green-v2/result.json. No installation.
Build only daemon; retain existing IME2067215/IBus3168326/tray3228081 and user's
Firefox988548. Verify exact source/build scope and loaded daemon bytes before
the same-process native reproduction. No acceptance claim before that result.

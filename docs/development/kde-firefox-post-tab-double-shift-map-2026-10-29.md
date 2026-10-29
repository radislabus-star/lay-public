# KDE Firefox: Double Shift after completion acceptance

Status: first failed transition proved; scoped source RED/GREEN underway.
The user reports Double Shift still fails and requests autonomous continuation.
The accepted display fix remains installed and is not reopened by this task.

## Baseline and reproduction

Canonical checkout: `/home/ubu/projects/lay-dependency-updates-20261027`, branch
`codex/browser-matrix-repair-20261027`, HEAD
`b835ca478198d6cb765a2246329d5646966c118c` plus the retained display patch.
Remote KDE/X11, native Firefox159.0 with the existing client adapter.
IME PID1443794 SHA004f6285a1725f280a4549c18ffa8d8937d6c27728b4dd0edfddf85a068e1c28.
Daemon374525, IBus3168326, tray3228081, manualFirefox988548 are protected.

Retained native display receipt:
`/home/e/projects/lay-development-runner/kde-install-control-20261020/kde-firefox-owned-hint-ready-ten-fields-native-v5/firefox/result.json`.
In rich/plain/iframe-rich, type the first full word and Space, type `при`, accept
`привет` with Tab, then Double Shift. Expected `привет ghbdtn `; actual
`привет привет `, caret14, no selection, focused editable native field.
DOM records the final rich separator as NBSP. This is an observation, not yet
proof that the separator causes the failure. The same normal-window controls
fail on retained old and current IME bytes; ordinary text passes on both.

## Existing dependency map; no alternative route

1. `src/bin/lay_daemon/manual_trigger_runtime/ime.rs`: the daemon owns the
   physical gesture and sends the existing ManualToggleV3 request.
2. `src/bin/lay_ibus_engine/bridge.rs` -> `shift.rs`: current owner admits the
   request and selects its existing executor/delegation. Retained rich trace
   records `double_shift_defer_exact_ime_tail`, status3. The gesture reached IME;
   a global missing-Shift explanation does not explain this recorded failure.
3. `src/bin/lay_daemon/layout_controller/ime_bridge.rs`: capture the exact
   committed-tail receipt and create the existing bounded replay lease/plan.
4. `src/bin/lay_daemon/exact_ime_tail_replay.rs`: validate lease/focus/layout,
   invoke the existing output executor and verify settlement.
5. `src/bin/lay_ibus_engine/tail_memory.rs`, `committed_tail.rs`, and the existing
   observation/authority contracts provide the current field/tail proof.

First unresolved transition: status3 delegation -> exact receipt/replay.
Next measurement: two unchanged native controls (ordinary text and rich) with
passive D-Bus timing/return capture. Reuse the retained actor/controller; modify
only its outer diagnostic selection/output path. Keep original cadence/assertions.
No daemon, tray, IBus or user Firefox restart; debug config restores exactly.

## First failed check and selected minimal change

Receipt `CROOT/kde-firefox-post-tab-wire-v1` reproduces1PASS/1FAIL in12.2s.
Rich ManualToggleV3 returns3, then VisibleTailV4 returns unknown-context with
empty tail/field receipt. Ordinary input returns the exact committed tail and
completes both toggles. The rich pending expected tail FNV5a181987736dcf2d is
`привет\u00a0привет `; actual raw field is `привет привет\u00a0`. The old
separator is reflowed from NBSP to ASCII while the newly emitted final Space
becomes NBSP. No mutation is issued after this source mismatch.

The GNOME/common `acknowledge_owned_preedit_space_postcondition` is identical
in canonical source and installed update-source22e9fead (function SHA0618bfcdb5e5b5e230dee93ea94cb2bab25be7e8454d0e528468300b43d2937e).
It acknowledges only the new final Space and deliberately rejects earlier
prefix changes. Keep that behavior and its existing negative test. The adjacent
KDE/Chrome presentation adapter already demonstrates exact-field separator
acknowledgement through the same tail owner.

Options: global whitespace equivalence1/10 (breaks exact proof); change the
Firefox delivery route2/10 (unnecessary authority/output migration); exact
KDE/Firefox presentation acknowledgement9/10 (selected, same receipt/owner).
New owning decision:2026-10-29-kde-firefox-preedit-space-reflow. Exact new seams:
KDE Firefox presentation module and attachment; one postcondition dispatch call;
the existing common tail owner gains a strictly checked presentation operation.
The prior display patch remains unchanged apart from classifier visibility.

Source experiment notes: red-v1 compiled and ran21 controls, but its owned-prefix
fixture appended history after preedit had started, invalidating that fixture's
start witness. This is a setup failure, not a semantic RED. Move retained prefix
input before the first preedit key. Red-v2 stopped in hash preflight before any
source mutation because a newly included tracked observation file was listed as
absent. Red-v3 corrects its expected baseline hash. Preserve both setup receipts;
only the intended `owned prefix=true` assertion may certify semantic RED.

Red-v3 exposed a second fixture preparation issue: pushing retained historical
input into that synthetic private client selected literal rather than preedit
typing. Red-v4 keeps the existing accepted preedit/Tab fixture unchanged and
models earlier retained text only in the pending-result owner contract. Its
native acquisition/key history is proved by the separate unchanged RFB actor.
Red-v4 now gives20/21PASS and the intended `owned prefix=true` failure; all
other19 existing controls and the new negative control pass. This is semantic
RED. The implementation factors the existing single shared owner update (same
epoch, path, buffer and suppression checks) and allows this strictly matched
postcondition to use it; existing final-Space/replay callers retain their checks.
No field mutation, new owner, native binding or layout operation is added.

## Installed first fix; distinct repeated-toggle receipt failure

Green-v1:21/21 targeted and1125/1125 correctness/package PASS;3 preexisting
performance tests not run. Build-v1:0warnings,74.621s. Source1304files SHA
e8de03875180fc7fa0117fdf48726d224293a049434870c6ba578e4e3ce86c9d;
installed/loaded IME PID1670433 SHAe163afd765fb13d8abc8d4de2a83003a3b2b1c5a2ccb60d51f899086f4b47ccf.
Daemon/IBus/tray/manual Firefox unchanged; exact original config restored.
Native short controls4/4PASS, source-specific reflow trace observed. All three
formerly failing rich fields now pass in normal/private/fullscreen. The54
display diagnostic rows are47PASS/1FAIL/6BLOCKED,162 prefix points; the separate
FAIL is private ordinary input's reverse toggle, after a successful first one.
Retain the raw private receipt, not the outer modes-v1 wrapper's incomplete
summary (it stopped on its failing assertion). Fullscreen-v1 is the remaining
real mode, not a retry of the private failure.

Native eight-v1:22rows,20PASS/1FAIL/1BLOCKED. Text/plain/iframe-rich each pass
eight toggles, every next-letter decoder, held Shift and released Shift. Rich
passes the first two but refuses the third. Email active has an actual
native/DOM observer disagreement, BLOCKED, not a Lay PASS. Existing roundtrip
controls retain their results. The new series is separate from the54 rows.

Private ordinary trace row1: after native delete/insert, final exact suppression
is cancelled at event770, before the final actual14-character snapshot at776;
second ManualToggleV3 refuses context_authority at814. The old waiting-only
daemon experiment in kde-post-replay-receipt-map was rejected and must not be
reapplied. Its missing final observation is the next diagnostic target.

Next bounded experiment changes only the compatibility library in an owned
KDE Firefox fixture process. The shared GNOME C source and installed libraries
remain byte-identical. A small KDE C wrapper includes the existing adapter,
preserves its original filter/reset/capability behavior and additionally requests
actual surrounding text after a forwarded printable key release. Instrument
handled/forwarded metadata, retain native control semantics and compare exact
receipts. This is a hypothesis, not an accepted fix or a new input/output route.
No production launcher, manual browser, daemon, IME, tray or IBus replacement
is authorized by this experiment alone. Promote only with causal native evidence.

The forwarded-release experiment compiled warning-free, retained the original
adapter contracts2/2 and passed one variant contract. LibrarySHA
dc8645acaa9af5d37d0c20d6d2927cf2a3bc89b7fb1da45a5064467954827018;
base C source remainsbdf572052c0d66b547e89386b1b493f9cf104714d730f0c37cec30f9918d6b5b.
It observed both handled and forwarded releases. Native22 gives20P1F1B, now with
textarea's reverse failure while rich's eight pass. This does not demonstrate
an overall correction; REJECTED and not installed. Production library/launcher
and manual Firefox remain unchanged. Keep CROOT/kde-firefox-forwarded-release-
build-v1 and native-v1 receipts, not a green-only subset.

## Editing boundary and next gate

This map authorizes no implementation expansion by itself. Before editing,
record the first failed check, compare the existing GNOME/neighbor mechanism,
declare exact allowed files in a new owning decision and source-scope receipt.
Keep word ownership, verifier and delivery contracts. A generic hypothesis,
NBSP normalization or another fallback must not substitute for actual evidence.
Run the short causal RED/GREEN first, then the affected Firefox proof and ordinary
held-Shift/display/Tab controls. Preserve FAIL/BLOCKED and source/runtime/native
verdicts separately. Full historical KDE matrix remains306P55F79B/440.

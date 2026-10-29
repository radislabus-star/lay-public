# One task: KDE Firefox, five consecutive Double Shift gestures

Status: the cadence experiment was REJECTED. Original daemon and all source
bytes restored. Continue the separately scoped post-replay receipt diagnosis.
User explicitly requested continuation one problem at a time, concrete file
names in dependency maps, and short tests before a longer proof.

## Baseline and selected symptom

The preceding Chrome fix is frozen in local commit
`b835ca478198d6cb765a2246329d5646966c118c`; it is not a new public release.
Canonical checkout `lay-dependency-updates-20261027`, branch
`codex/browser-matrix-repair-20261027`. Runtime source SHA
`2ce6b24786bab558599a46345c9ec3583f419847420a1f37ebf5a08e93933c5e`.
Remote KDE engine PID3556137/SHA
`8b84bf5b6c569550af8e4061ea4cca83e74382d243ae693aeb243a1e22a035d3`
independently rechecked before this task. Local GNOME remains outside scope.

One selected case: normal Firefox window, ordinary text field, active word,
five consecutive Double Shift gestures at the unchanged r19 cadence. The old
111 receipt has five gesture starts approximately211ms apart, three matched
targets instead of five, and three `ManualToggleV3` responses with status3.
Status3 means `DelegateExactImeTail` in `src/manual_toggle.rs`; it is not a
successful local replacement and its boolean is not a layout result.

Old receipt: CROOT/isolated-release111-full-native-v1/firefox/firefox/
row-3-native-trace.jsonl and its220-row result. These old measurements do not
certify the current binary. First action is one fresh native reproduction;
do not infer where a gesture is lost from the three IME replies alone.

## Existing dependency map, with file responsibilities

Paths below are relative to `src/bin/lay_daemon/` unless prefixed `IME:` or
`src/`. IME paths are relative to `src/bin/lay_ibus_engine/`.

```text
native_input/x11.rs: actual native input
  → daemon_runtime.rs:251: existing input loop
  → manual_trigger_runtime/event.rs:102 + trigger_fsm.rs: physical Shift pair
  → trigger_dispatch.rs: one manual-toggle action
      → layout_controller/ime_manual_toggle.rs
      → layout_controller/ime_bridge.rs: ManualToggleV3
      → IME: window_interaction/authority.rs + shift.rs
      → src/manual_toggle.rs: explicit typed outcome / exact plan
      → exact_ime_tail_replay.rs: existing delegated exact-tail executor
          ├─ physical_input_grab.rs + native_input/replay_master.rs
          │    existing isolation and queued physical input
          ├─ layout_controller/kde_exact.rs: controlled KDE layout handoff
          ├─ text_output/key_emit.rs → native_input/x11.rs
          │    existing verified Backspace / typed replay
          └─ layout_controller/ime_bridge/settlement.rs
               current word/frame settlement before the next action
```

An already owned IME composition takes the existing `Handled` branch inside
`IME: shift.rs` instead of delegating to `exact_ime_tail_replay.rs`. These two
current dispositions must not be conflated. No new Firefox owner, detector,
delivery path, timeout or queue is proposed by this map.

## Fresh native diagnosis, before runtime edits

One isolated normal/text/rapid-five/active run passed all five transformations:
latencies 11.683, 9.220, 10.958, 11.952 and 9.872 ms; clock mapping uncertainty
1.726 ms. Its trace contains five status1 (`Handled`) responses and active
IME composition. Receipt: CROOT/firefox-rapid-five-baseline-v1/result.json and
firefox/result.json. Exact config bytes were restored; engine PID/SHA stayed
unchanged. This proves only the isolated starting condition.

The subsequent unchanged full r19 run reproduced the selected normal/text/
rapid-five/active failure after preceding edits. Its row3 trace has five
status3 (`DelegateExactImeTail`) responses and no preedit updates. Thus five
commands reached IME in this fresh failure; missing trigger recognition is
not established for this row. The first failed transition within delegated
execution is still under investigation. Do not call the isolated PASS a fix
and do not change the Shift FSM or force Firefox onto another delivery path.

Full receipt root: CROOT/firefox-rapid-five-full-baseline-v1. Completed result:
127 PASS / 55 FAIL / 38 BLOCKED / 0 NOT_TESTED, 220 rows, 451.301 s.
Failure groups: rapid-five active27, rapid-five space24, moved-boundary3,
roundtrip active1. Config bytes and engine PID/SHA were restored/preserved;
held keys empty. The controller's whole-run trace anchor rotated out: global
`trace_complete=false` must remain visible. Diagnose from the selected row's
own trace plus the timestamped DBus monitor rather than claiming a complete
whole-run causal trace.

For selected row3, first ManualToggleV3 starts at1793195375.984722 and returns
status3 at1793195375.986671 (1.949 ms). The next VisibleTailV4 result is the
exact committed `привет`, RU, epoch3408, same context receipt. A cancellation
reply follows at1793195376.016014; no target layout activation precedes it.
All five calls return status3, 1.471–2.451 ms. The first unexplained failure
therefore precedes actual layout change/delete, rather than proving a slow
Shift detector or candidate ranker. No Firefox runtime code has been changed.

Next bounded diagnostic retains the original actor's first three normal/text
active cases (`consecutive-two`, `roundtrip`, `rapid-five`) in original order.
Only its case selector may be extended to a comma-separated allowlist; gesture,
input, readiness and assertion functions stay byte-identical to r19. No retry
to obtain green. Record the exact actor SHA and semantic-body equality.
The daemon's existing debug flag is startup-only. A temporary runtime drop-in
may enable LAY_DEBUG_LOG and redirect only this service's stderr to the owned
diagnostic file, restarting the same daemon bytes for this one diagnostic.
Finally remove only that drop-in and restart the original service. Preserve
IBus parent, engine and tray PIDs/bytes, all other drop-ins, config and local
GNOME. This is logging setup, not a source or delivery-route change.

| Boundary | What must be measured / kept separate |
| --- | --- |
| Physical five pairs → trigger FSM | Count physical presses/releases versus recognized semantic gestures. Preserve ordinary held Shift. |
| Recognized gesture → IME request | Count requests, current field/focus, exact outcome enum; never treat delegation as completion. |
| Exact-tail lease → replay | Existing shared proof and executor; count authorized mutations, layout acknowledgements, replay completion and elapsed time. |
| Replay isolation → queued user input | Determine whether later physical gestures are retained, forwarded or cancelled. Do not change a global input handler from a Firefox symptom. |
| Finished mutation → next target | The complete word must be visible before the next original gesture. No larger input gaps, waiting between gestures or weaker assertions. |

## Scope and next gate

The initial diagnosis was read-only. The implementation preflight and ADR below
now declare the exact first failed transition, shared scope and allowed files.
The user authorized fixing this one Firefox problem; a common integration seam
must be disclosed and declared explicitly rather than hidden in an adapter.
Chrome observation/presentation, GNOME, terminal integration, semantic ranking,
ownership/verifier and existing output routes are protected.

Use the existing immutable native actor r19, SHA
`360e6b9a0735fef75aba16c8d89d57d1e9447bf0cf5603c70213227843c24913`,
selector `text:rapid-five:active`, normal Firefox, one owned fresh test window.
The established process-local accessibility activation is retained. No global
IBus/desktop restart, installation, shared learning cleanup or local input.
The bounded same-binary daemon logging restart is explicitly described above.
Temporary trace configuration must restore its exact previous bytes.

Implementation follows the scoped preflight below. Obtain the short causal
native RED/GREEN before a longer Firefox proof. Other Firefox failures remain
separate work.

## Implementation preflight: KDE X11 exact-replay cadence only

The three-case diagnostic reproduced2 PASS / 1 FAIL in14 s. All five target
strings were produced; 130–188 ms release-to-target latencies include late
intermediate targets. The first pre-layout cancellation from the full run did
not recur. Keep it as an unresolved observation, not the cause of this run.
The startup-only debug drop-in was removed and the original daemon restarted
with identical SHA `eb6098b445c48ae5a681fb8b1d33051e28a5da3646d90baa4cd47b09a7228de5`;
IBus parent, engine and tray remained unchanged. CROOT/
firefox-replay-prefix-diagnostic-v1/result.json records exact identities.

Passive wire run CROOT/firefox-replay-prefix-wire-v2 again reproduced2 PASS /
1 FAIL. Its five ManualToggleV3 calls took about2 ms. The first call's sequence
is exact capture → input isolation/focus → KDE layout stack → suppression →
physical delete/insert. Existing post-output settlement returned an unavailable
field receipt and the operation was treated as indeterminate; do not hide that
warning or modify its admission in this pacing task. Version1 of the wire
launcher failed before native input because the GUI bus was accidentally used
for a systemd query; retain that setup failure. Version2 keeps manager and GUI
buses separate and changed no live service.

Source-measured fixed waits in `text_output/key_emit.rs`: Backspace pacing2 ms
per character plus16 ms after deletion; insert pacing1 ms per key event. Six
unshifted letters therefore add28+6 ms of intentional sleep, independently of
actual X11 requests. This is a removable delay hypothesis, not a measured34 ms
speedup. The established bounded fast routines already exist in the same
module and are used by `text_output/replacement.rs`; reuse them unchanged.

| Option | Assessment / score |
| --- | --- |
| Change the gesture interval or wait for each target | Rejected1/10: changes user-visible input cadence instead of repairing execution. |
| Replace delegated Firefox editing with another delivery route | Rejected2/10: changes frozen field integration and authority; no evidence calls for it. |
| Parallelize/rebuild the KWin focus provider or layout read protocol | Deferred5/10: larger shared integration change, fresh-focus proof needs its own task; not necessary to test the existing fast output. |
| Select existing fast cadence only for the already isolated KDE X11 exact replay | Selected8/10: removes fixed waits without replacing emission, focus/lease checks, routing or settlement. Native acceptance is still required. |

Files and dependencies for this one change:

```text
exact_ime_tail_replay.rs: existing executor, one immutable cadence selection
  → text_output.rs: module wiring / type export only
      → kde/exact_replay_cadence.rs: KDE + actual X11 variant selects timing
          ├─ KDE X11: existing bounded fast key_emit functions
          └─ GNOME / other desktop / evdev: original paced key_emit functions
  → same field/focus/lease checks, suppression, output order and settlement
```

This is explicitly a **shared KDE exact-replay integration** change. It is not
Firefox-only source even though Firefox supplied the selected failure. Other
KDE clients that enter this exact same existing executor may see the changed
cadence. Common executor wiring is allowed only in the two exact files above;
all other executable bodies remain unchanged. The scope guard's existing
`kde-focus` category names the daemon KDE directory; the focus provider/policy
themselves are protected and must remain byte-identical.

Consequence check before code:

- Candidate/lattice retention, ranking, learning, false authority and word
  ownership do not change. Double Shift stays an exact key projection.
- Select cadence once from the resolved backend and actual VirtualDevice enum.
  Do not infer X11 from the broader `requires_client_settlement` capability;
  a future transport may need settlement without qualifying for this policy.
- Existing bounds, frame emission, modifier cleanup and all checks before and
  after output stay in place. No batch, retry, queue, timeout extension, new
  owner, new output route or cross-client state is added.
- Faster asynchronous output can reveal a client ordering/settlement defect.
  Native assertions, current indeterminate receipts and negative paths stay
  strict; failure rejects the timing experiment instead of changing authority.
- The known post-output missing-field receipt is separate debt. This task may
  prove visible five-gesture cadence only; it cannot claim executor settlement
  or the earlier pre-layout cancellation fixed without separate evidence.
- GNOME, KDE evdev and non-KDE X11 retain the exact previous emitter functions.
  A future layout/transport enum variant must take that conservative branch.
- Rollback boundary: only the new cadence selection/policy and its two wiring
  seams. Preserve the accepted Chrome fix and every existing guard/output body.

Proof gates: unchanged baseline short failure; inspect source diff and exact
executor-body correspondence; short remote daemon source checks including
physical gesture/ordinary Shift/output bounds; warning-free release daemon;
same immutable three-case native diagnostic with no added pauses; only after
PASS, full220 Firefox and short existing-route Chrome/terminal/held-Shift
controls. Record timing distributions and failures independently. No GNOME
installation, global IBus restart, public release or local runtime replacement.

Owning ADR: `docs/architecture/decisions/2026-10-28-kde-x11-exact-replay-cadence.json`.

Source checkpoint: official remote check run-O35cFz, localrun-zi3yutk1,
329/329 daemon tests PASS, fmt PASS, 160.028 s. Initial checkrun-50ok1dqa
stopped at formatting before tests; retain it. The first architecture refresh
then stopped before release compilation/activation because the cadence method
reused the catalogued `emit_backspaces` owner name. Preserve that owner rule:
rename only the new policy methods to `emit_exact_backspaces` and
`replay_exact_keycodes_after_modifier_cleanup`; adjust the existing order
guard's literal call references without removing an assertion. No inventory
identity/count changes are needed. Retest that guard after the mechanical
rename and keep the successful329-test source result's exact earlier hash
separate from the renamed source and future installed bytes.

CROOT: `/home/e/projects/lay-development-runner/kde-install-control-20261020`.

## Experiment result — do not repeat the pacing-only approach

Final source329/329 PASS (localrun-5pojzqgd/remoterun-IuB6dj), architecturePASS,
release daemon66.609s/0warnings. Candidate SHA56120235015f7922f7d8dbbe0c90d1b497dd3046dd4bad4bc25726e3df52f2ad
failed the unchanged three-case short native check:1PASS2FAIL. No long run.
All five gestures reached IME; only the first delegated replay completed its
admission, later calls were refused by context_authority. Input remained ready.
Original daemon SHAeb6098b445c48ae5a681fb8b1d33051e28a5da3646d90baa4cd47b09a7228de5
restored asPID246741, with engine/IBus/tray/config unchanged. Source and graph
restored to1301files/SHA2ce6b24786bab558599a46345c9ec3583f419847420a1f37ebf5a08e93933c5e
locally and remotely. The failed source is archived in CACHE/
kde-native-cadence-rejected-source-v1.tar.gz and its decisions/receipts retained.

The failed trace proves an earlier causal transition: row3 event600 cancels
the exact-replay suppression scope; event602 then sees a delayed three-letter
old deletion snapshot with replay_scope_present=false and event603 discards
the reset lineage. Event610 has the final six-letter snapshot, but subsequent
callbacks remain unknown_start and repeat manual toggles are refused. The
existing IME already knows how to retain such prior replay snapshots while the
exact scope exists. Do not rewrite that handler or weaken its authority.

Next owning preflight: `kde-post-replay-receipt-map-2026-10-28.md`.

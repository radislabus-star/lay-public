# KDE Firefox: rapid native result retired before its final actual receipt

Source703848cb,daemon515781/0a71,IME795642/689fa38c,tray795755/scriptc3043b69;
IBus3168326,userFirefox988548,config6e62eab3. No agents/Harness/GNOME/publication.
Preserve completed Tab/outline fixes. Current native32/32Tab and ordinary22
21P0F1B; fullFirefox126P55F39B,Chrome earlier179P0F41B/220each unchanged.

## First actual failed transition

CROOT/kde-firefox-hint-outline-native-prefix-v1/firefox/result.json and row2/3
native traces, both complete with all lines JSON parsed. Prefix3=1P2F0B/15.265s.
Row2 active rapid-five ends RU, not expected Latin. Four delegated manual RPCs;
fifth not_handled/context_authority. Fourth output press/release callbacks finish
at tail1665; Reset8174 arms receipt at index1554; daemon_exact suppression is
cancelled at1563(epoch1653); full actual SSI6/6 arrives1565; fifth refused1601.
No fourth NativeReplayProcessed log. This absence is meaningful only in this
complete row and does not yet distinguish early-return from no method call.
Row3 afterSpace visits all5states but late targets, latencies163/142/256/240/213ms.
RFB/DOM clock uncertainty1.33ms. Fix only the missing fifth transition first.

```text
exact_ime_tail_replay.rs::execute -> existing complete output/focus checks
 -> kde/firefox_replay_receipt.rs::processing_readiness
       current_client_is_scoped returns bool, including read errors as false
 -> layout_controller/ime_bridge/settlement.rs::wait_for_committed_tail_settlement
       readiness first, then exact V4; fixed original80ms budget
 -> layout_controller/ime_bridge.rs::native_replay_processed (existing RPC)
 -> lay_ibus_engine/window_interaction/authority.rs::native_replay_processed_v1_inner
       existing bridge token and target-path check; early false has no trace
 -> exact result succeeds OR retain_processed_result OR existing cleanup cancel
 -> observation.rs Reset/actual-SSI confirmation (unchanged until root proved)
```

Hypotheses: classifier unavailable collapsed to generic readiness; bridge/profile
readiness error or target-path mismatch; cancellation before final actual SSI.
Do not promote a hypothesis to cause. Existing daemon log is disabled via its
OnceLock. First read exact existing settlement warning in one temporary daemon-
only logging probe; no source build needed, then restore environment. Do not
raise80ms budget or reapply rejected all-fast closed-word pacing without proof.
Diagnostic9/10; speculative budget increase4/10; new detector/owner2/10.

Temporary LAY_DEBUG_LOG=1 requires two idle daemon restarts, same exact0a71
bytes, same DISPLAY71/KDE/source/config; IME/tray/IBus/userFirefox untouched.
Created drop-in reserved under lay-daemon.service.d; only that file is removed
on restore. Unit Environment hash compared after restoration, not persisted
in plaintext. Existing actor e698 and cadence/assertions unchanged. Logs kept
private and filtered to exact replay reasons, no unrelated env export. Native
results with logging are diagnostic, not performance/full-matrix promotion.


Existing logger diagnostic v1 complete:2P1F; active5/5PASS (original missing-
fifth not reproduced), afterSpace lateFAIL. Journal10 complete/pendingprocessed
messages gives no discarded-branch reason. Restore verified: daemon863172/same
0a71, unitEnvironment hash exact, config6e62, temporary dropin removed; IME795642,
tray795755,IBus3168326,userFirefox988548 unchanged. No source change.
Do not infer80ms elapsed from pending log: common waiter propagates a V4
unavailable error immediately; its retention branch omits that reason. Next
passive native D-Bus monitor observes read-only processing ACK, V4 return shape
and cancellation, paired by serial, one owned3case probe. No further restart.


Passive monitor setup: GI v1 selfcheck saw method call but not paired return;
CLI text v2 selfcheck/parser failed before any native keys. Both stopped and
runtime/config preserved; do not treat either as product failure or cause proof.
PCAP v3 uses existing dbus-monitor timestamped wire records decoded by installed
Gio.DBusMessage; paired empty-path NativeReplayProcessed=false selfcheck must
pass before actor starts. Summaries retain only source/length/epoch/path/field
presence/booleans, never visible word text; private raw capture stays on KDE.
PCAP v3 completed: native3=1PASS/2FAIL, both rapid rows reached all five
states but some targets were late against the unchanged next-gesture deadline.
146 paired method events, zero unpaired calls; NativeReplayProcessed returned
true for all10 delegated gestures. Each ACK and V4 RPC took roughly0.5–2ms.
The first ACK appeared128–175ms after ManualToggleV3; the immediate V4 result
was source-free unknown, so no80ms wait is established. Original missing-fifth
failure was not reproduced. Runtime863172/795642/795755/config preserved.
Receipt: /home/ubu/.cache/lay/development/browser-matrix-repair-20261027/passive-replay-bus-v3-result.json;
paired summaries: passive-replay-bus-v3-events.jsonl in the same directory.

Existing explicitly ignored KWin reader benchmark (read-only,24samples) on
current703848cb source: median2.6136ms,p954.9099,max8.2701ms. No keys, install,
restart or exported window metadata. Runtime/config unchanged. This does not
justify a new detector/provider or removal of fresh focus checks. Receipt:
/home/ubu/.cache/lay/development/browser-matrix-repair-20261027/kde-focus-live-latency-readonly-v3-result.json.
Its first wrapper failed a source-identity type assertion before Cargo/input;
that preserved result is setup failure, not a product FAIL.

## Queued separate user report — GNOME Chrome hint styling

On2026-10-29 the user reports the completion letters in GNOME Chrome also lack
visual distinction and suspects a shared/global styling change. They explicitly
request finishing this KDE Firefox task first, then examining the common setting;
no rollback is requested now. Treat this as OPEN and user-reported, not disproved
by the absence of local GNOME edits in this continuation. No local GNOME source,
settings, process, installation or restart is authorized by this queued report.
The isolation claim remains unproven for styling until the common policy and
installed GNOME adapter are mapped. Never use KDE outline PASS as GNOME evidence.

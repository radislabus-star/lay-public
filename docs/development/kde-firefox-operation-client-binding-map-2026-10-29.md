# KDE Firefox: one client classification per existing replay operation

Canonical checkout lay-dependency-updates-20261027, branch
codex/browser-matrix-repair-20261027, base b835ca47. Current remote source
703848cb; daemon863172/0a71fd83, IME795642/689fa38c and KDEtray795755/c3043b69.
Preserve accepted Tab/outline, all dirty earlier fixes and the user Firefox.
No local GNOME change, Harness/agents, release or new transport/owner.

## Proven source defect and native evidence boundary

`kde/firefox_replay_receipt.rs::current_client_is_scoped` maps an unavailable
KWin observation, PID metadata, executable or exact identity to false. An open
raw-word replay already requires this classifier to return true to select fast
pacing. After output the same operation classifies again to select processing
readiness, and once more to retain a processed but not-yet-observed result.
Those later failures can discard the earlier client policy even while every
fresh common focus check still succeeds. A policy read is not edit authority.

Native prefix row2 proves fourth-output cancellation before final exact SSI
and fifth context_authority refusal. It does NOT prove which classifier failed.
PCAP v3 subsequently shows all10 native ACKs true and all targets reached, with
timing FAILs. Do not call this patch a proven fix for all55 full-matrix FAILs.
Read-only KWin24 sample median2.61ms/p954.91ms does not justify replacing that
provider. Traced syscall probe slowed the program, and its wrapper timed out on
cleanup; recovered check proves TracerPid=0, exact original runtime/config and
no held keys. Do not promote its latency/native counts as acceptance.

## Exact dependency and file boundary

```text
src/bin/lay_daemon/exact_ime_tail_replay.rs
  existing focus + field lease + layout + suppression checks
  -> kde/firefox_replay_pacing.rs (UNCHANGED)
       raw leased word + isolated native output + exact KDE Firefox => fast
  -> existing delete / insert + fresh focus checks (UNCHANGED)
  -> kde/firefox_replay_receipt.rs
       reuse proved fast-profile classification; otherwise classify once
       same native-processing RPC + same final fresh exact focus check
  -> layout_controller/ime_bridge/settlement.rs (UNCHANGED)
       same deadline, actual field/epoch/path/layout/tail requirements
```

Only runtime bodies in firefox_replay_receipt.rs change; the common executor
has the explicitly disclosed construction/call seam. No new word/context owner,
timer, retry, queue, delivery route, cached field/focus authority or rendering
policy. Client classification cannot itself authorize mutation, establish the
actual result or retire the lease successfully. GNOME/non-Firefox keep generic
readiness and cannot acquire scoped result retention from this policy.

No changes to key_emit, native_input, layout handoff, bridge/RPC, settlement,
correction, hint_view, tray, GNOME/Chrome/terminal sources/config/services.
The cumulative scope receipt retains earlier accepted source; those historical
shared paths are not permission to rewrite them during this task.

## Options and gates

Reuse the operation's verified client policy:8/10, selected. Re-read metadata
and raise timeouts:3/10, cannot make an unavailable observation reliable. Replace
focus/output provider or broaden fast pacing to closed words:2/10 here, larger
change without this mechanism's proof. No reapplication of rejected fast-closed
experiments.

RED must show a previously verified client falling to generic policy on a
later unavailable read. GREEN must also refuse all missing native ACK / changed
focus cases, avoid executing scoped ACK work for a foreign client and preserve
existing desktop/window/executable classifier negatives. Then daemon source
suite, warning-free release build, exact loaded bytes, short native probe,
ordinary22 and Tab/held-Shift controls. A source PASS is not native acceptance.
Keep the historical full matrix Firefox126P55F39B and Chrome179P0F41B until a
new complete matrix actually finishes; no changed cadence/assertions/counts.

Separate queued issue: user reports GNOME Chrome suffix is visually ordinary.
Finish this task before mapping the common setting. No rollback or local GNOME
change is requested now. KDE outline evidence does not cover GNOME styling.


Source proof: red-v1 is the intended19/20 controlled failure for a prior
verified scope followed by an unavailable classifier. Green-v1:20/20 focused
and336/336 daemon correctness/package tests PASS; graph/architecture53.464s
PASS. Registry gains exactly the two policy tests, no removed/changed tests.
Source after registry e14b4344bc8c1cf6133d265630abe46291f6d3987db92ff6d01119c77669624c.
Receipt: /home/ubu/.cache/lay/development/browser-matrix-repair-20261027/client-binding-green-v1-result.json.
No native result or loaded-byte change follows from this source proof alone.


Loaded daemon1015506/SHA271d84aa0058196406e3225e1d04485c132c7d243f6b86952c47f61fa7d1b574; build0warnings67.09s. All other runtime/config preserved. Native prefix3:1P2F,16.168s. Active rapid has4delegated outputs,8positive native ACKs, no suppression cancellation, one context_authority refusal at trace1456 and exact actual SSI confirmation1458. The final target is wrong; no native acceptance claim. AfterSpace reaches all5states but misses deadlines. Current classification fix therefore does not close the whole rapid symptom. Source proof retained, no revert/reapply. First next transition is the existing same-gesture AwaitingExactSnapshot response, before any new output; inspect current IME pending-source policy rather than repeat the rejected post-output waiting-only experiment.
Receipt current KDE prefix row2: /home/ubu/.cache/lay/development/browser-matrix-repair-20261027/client-binding-prefix-row2.jsonl.


Setup-only comparison before further production edits: current userFirefox988548
loads existing library924b5982 through its process-local preload. Daemon1015506
has no preload; the owned geckodriver browser inherits that env and uses the
Firefox ELF directly. All native trace capability rows are41/exact_refresh=false.
The production KDE launcher explicitly preloads the unchanged notify library
and GTK IBus. One owned three-case comparison aligns only that child-tree
environment with production; actor e698, cadence/assertions/denominator unchanged,
no global environment, installed bytes, current user browser or GNOME changed.
Require the actual private capability bit in the owned trace before treating
the setup as effective. Preserve old no-adapter failures and distinguish results
by launch configuration. This is not a new delivery route or source repair.


Production-launch prefix complete2P1F14.756s. Actual capabilities1073741865
(41|1<<30) observed, same library924b5982. Active5PASS, afterSpace5states but
deadlines late23–37ms. No-adapter active rejection is not relabeled fixed by
this one comparison; keep its receipt. Production-launch Tab16/16 and held
Shift/title/upper16/16 PASS, ordinary22 still running. No source/runtime/config
changes in launch alignment. Full counts unchanged.


Production-launch ordinary22=20P1F1B71.881s: text postTab-eight second
flip fails before suppression; email native/DOM disagreement stays BLOCKED.
Complete row1 trace has first delegated request410,suppression497,nativeACK751,
then next delegation846 with no second suppression. This differs from the
stock earlier lost-before-confirmation symptom. No speed-profile change is
justified from it. One existing-log diagnostic now selects only this first
postTab-eight row, unchanged actor/adapter/current271d bytes, temporary daemon
LAY_DEBUG_LOG=1 (two idle restarts), exact env/config/protected runtime restored.
The explicit pre-output warning can identify first lease/layout failure; no
new source instrumentation, common-handler fix or replay retry is added.

# KDE Firefox: acknowledge the existing Space key's actual NBSP readback

Canonical checkout lay-dependency-updates-20261027, branch
codex/browser-matrix-repair-20261027, baseb835ca47. Current source775c2f90,
daemon1134112/736c26f5, IME795642/689fa38c, KDEtray795755/scriptc3043b69,
IBus3168326,userFirefox988548/config6e62 unchanged. SOLO/noHarness/agents,
no local GNOME install/change/restart/publication. GNOME Chrome style report is
separately queued; finish this current KDE Firefox work first, no rollback.

## First failed transition — measured

The diagnostic-only source775c passed21focused tests, architecture and0warning
release build. Ordinary native22=21P0F1B75.448s. Its refusal file shows real80ms
timeouts after native delivery, with source passive:committed-tail. Those are
not merely immediate missing-field errors. Prior intermittent text postTab
second-flip failure remains unclosed and has no pre-output cause proved yet.

Passive native3 wire receipt closed-tail-wire-shape-v1:710paired events,
0unpaired, current runtime/config preserved. Text closingSpace2flips match
literalSpace and settle. Plain postTab8 and plain closingSpace2 all show whole
exact target length14/7, exact prefix retained by checksum, target mode, current
path/field and advanced epoch; their final codepoint is NBSP160. Emission and
PreparedExactReplay expect ASCII32. The common literal ends_with rejects this
client representation and polls to80ms. No shortened-tail/prefix loss occurred.
The3native rows PASS, but the supervisor exited1 on a diagnostic selector-order
assertion: actor runs plain/postTab8 before plain/roundtripSpace. Preserve that
setup error; the independent paired wire is still the causal metadata source.
Next comparison uses actor canonical selector order, unchanged physical input.

Receipts under /home/ubu/.cache/lay/development/browser-matrix-repair-20261027:
closed-tail-wire-shape-v1-result.json and closed-tail-wire-shape-v1-events.jsonl.
Raw private PCAP remains remote, no word payload exported. Known fixture hashes
were matched locally to check source/target and exact Unicode boundary.

## Exact file and dependency boundary

```text
src/bin/lay_daemon/exact_ime_tail_replay.rs::prepare_exact_replay [UNCHANGED]
  exact projected token + acknowledged Space/NBSP count -> ASCII Space key frames
  same lease/field/focus/layout/suppression -> same delete/insert [UNCHANGED]
  ProcessingReadiness (same operation-bound KDE Firefox classification)
    -> kde/firefox_replay_receipt.rs: client-specific readback comparison
  -> layout_controller.rs: explicit callback forwarding seam only
    -> layout_controller/ime_bridge/settlement.rs: same one waiter and deadline
       same source/path/field/advanced-epoch/mode/native-processing checks
       -> adapter's tail comparison as final text condition
trigger_dispatch.rs: queued caller selects that same client policy once; others literal
layout_controller/ime_bridge.rs: existing strict consumer tests keep literal policy
```

These six runtime/test source files are the exact new seams. Cumulative source
scope lists earlier accepted changes; it is not permission to modify others.
No change to IME source, output/key encoding/pacing/input isolation, layout
handoff, candidate/learning, ownership, SafetyGate/verifier authority, hint/tray,
GNOME/Chrome/terminal policy or runtime settings. The common waiter does not
detect clients or normalize text itself. It requires an explicit comparator.

The expected readback is produced only by prepare_exact_replay's validated
plan. Its terminal ASCII run is exactly the acknowledged boundary count. The
KDE Firefox adapter may map only that closing run to the same number of ASCII
Space/NBSP characters. Every preceding byte, including internal whitespace,
remains literal. Raw words with no emitted boundary retain literal comparison.
An expected NBSP not produced by this key plan is not normalized. Other clients
and desktops retain literal readback. Ordinary and queued consumers select the
same single pure comparison policy; the queue uses the existing exact window
classification once and all original focus/field checks stay mandatory. Missing/extra boundaries, newline, tabs,
zero-width characters, changed words/prefixes and all identity failures refuse.

The neighboring IME Space acknowledgement already models this real Firefox
representation with full actual snapshots; reuse its strict byte/count approach
rather than a generic whitespace trim. Do not change that working producer or
rewrite the transport to compensate for the consumer comparison.

## Options and verification

Client-specific exact closing-boundary comparison9/10, selected. Global Unicode
whitespace normalization2/10 (changes unrelated clients/ranges); larger timeout
1/10 (cannot make unequal bytes equal); another delivery route1/10. This task
does not authorize reapplying rejected fast-closed pacing or shrinking expected
readback to the last word.

Source RED must fail on literal-only NBSP readback. GREEN keeps all negatives
and default strict caller tests; consumer callback cannot override any other
receipt condition. Focused24 then full340daemon source, graph/canon, exact+3
registry identities, warning-free daemon build and exact installed/proc bytes.
Short paired native3 must lose the8+2 pointless80ms waits while preserving actual
text and mode. Then ordinary22 and Tab/heldShift controls. Keep source/runtime/
native scopes separate, all old failures and the full historical table:
Firefox126P55F39B, Chrome179P0F41B, total305P55F80B/440 until a new full run.


RED v1:23/24 with the intended closingNBSP failure; all safety/default-client
controls pass. GREEN v1:24/24+340/340 daemon, graph53.586s PASS, sourceb11a9104.
Not built or installed: before-install source review caught the queued consumer
would still compare ASCII literally after the normal operation settles. Both
existing consumers now select the same KDE/Firefox pure policy, without another
normalization algorithm or input/delivery path. Additional assertions exercise
normal/queued equality and foreign-client strictness. Same24/340 identities.
GREEN v2 repeats affected proof for this final consistency refinement, exact
registry delta0 (the3new identities were already registered byv1). Preserve all
receipts; never claimv1full proof certifiesv2source. No unproved production fix
or old rejected pacing code has been installed.


Final GREENv2:24/24focused and340/340daemon correctness/package PASS,
graph/canon53.623s PASS, canonical registry exact(no added/removed/changed).
Sourceb616e1a3aa3097fb435636184866e49b01c11d1d0ca95e23a616cae67a93ffde,1309files.
Normal and queued paths now use one pure scoped comparator; all default-client
and negative receipt checks passed. No native claim before changed bytes run.
Receipt CACHE/space-readback-green-v2-result.json.


Installed finalb616e1a3 source: daemon1245755/SHA35527e63ebff48a4cc6711a71e62999e737298c252583990f1a308a388988cec,0warning67.212s. All other runtime/config bytes/PIDs preserved. Paired native3 on same actor/canonicalorder:3PASS/0FAIL/0BLOCKED,12delegated flips,146events/0unpaired, supervisor0, no daemon refusal/pending events. Before710paired events/184V4calls/159processing calls; after146/48/13(one selfcheck). All12gestures now use exactly4V4reads. The10rich operations finish last read median266.027→182.166ms,p95342.910→205.853ms. This is ManualToggleV3 call-to-last-V4 time, not a hardware-release/UI latency claim. Four separately observed RFBsend→DOM latencies167.38–189.94ms; postTab8 carries no perflipUI samples, gapexplicit. No discarded80ms polls remain in this short proof. Runtime/config exactpreserved, row2IMEtrace incomplete so use independentpairedwire for causality. Comparison receipt CACHE/space-readback-wire-comparison-v1.json.
Native ordinary22+Tab16+heldShift16 on the same installed bytes is next; full220counts not rerun yet.


Same installed35527e63/b616e1a3 controls collected2026-10-29 (ordinary finished17:26:21.662724UTC):
ordinary22=21PASS/0FAIL/1BLOCKED78.806s (email native/DOM observation mismatch
retained, supervisor1); lowercaseTab16/16 21.802s; heldShift/title/uppercase
Tab16/16 23.308s. Config/runtime preserved. Receipts CROOT
kde-firefox-space-readback-native-retained22-v1-verification.json,
kde-firefox-space-readback-tab-native-v1-verification.json,
kde-firefox-space-readback-title-tab-native-v2-verification.json.
Next full220 uses unchanged r19 actor and production-owned Firefox child launch
(unchanged notify924b5982 library+GTKibus+a11y; capability receipt required).
No old counts relabelled; no local GNOME or renderer change.

Full220 started on same installed bytes with immutable r19 actor; wrapper/controller hashes a51f570a/9bee911f. Production-owned child launch only. No source/runtime changes during run; guard kernel limits verified20CPU/24Ghigh/28Gmax/1Gswap/512tasks. Full result remains pending.


Final full220 samebytes:159PASS/24FAIL/37BLOCKED,0NOT_TESTED,431.581s.
Original160108P24F28B;additional60=51P0F9B. All24failrapid-fiveafterSpace:
23late-target assertions,1missingtarget fullscreen/search. Everyactive-word
rapid-five passes. Runtime/config unchanged/restored,heldkeysnone,actual
production capability observed. GlobalIMEtrace incomplete;9/24failed row
traceanchors complete, not alone a full causal proof. Do not attribute all31
fewerFAIL to last comparator patch: production child launcher also aligned.
Local receipts CACHE/space-readback-full-v1-result.json and-summary.json;
remote CROOT/kde-firefox-space-readback-native-full-v1-verification.json.
Current narrow Space/NBSP settlement fix proven; remaining closed-word rapid
performance is separate unresolved group. No further pacing/source edit made.
GNOMEstyle read-only baseline recorded in handoff; cause unproved, no changes.

Timing summary, RFB-send-release mapped to DOM (not independent serverhardware
release), missingcoverage explicit:
{
  "rapid_active": {
    "rows": 30,
    "expected_samples": 150,
    "observed_samples": 150,
    "median_ms": 4.2169042862951756,
    "p95_ms": 8.540216600522399,
    "max_ms": 41.22512927278876,
    "scope": "RFB send release mapped to DOM, not independent server hardware-release timestamp"
  },
  "rapid_space": {
    "rows": 30,
    "expected_samples": 120,
    "observed_samples": 115,
    "median_ms": 184.46651985868812,
    "p95_ms": 281.8041085265577,
    "max_ms": 303.3417244441807,
    "scope": "RFB send release mapped to DOM, not independent server hardware-release timestamp"
  }
}

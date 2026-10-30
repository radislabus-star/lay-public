# Lay1.0.114 — bounded KDE Firefox pacing improvement

Status: published2026-10-30T16:19:38Z as v1.0.114,immutable source7b7fe8e0.
Public CI41018481523 and Security41018481499 bothSUCCESS. Actual installed
updaters:localGNOME113 sees114available;remoteKDE114 confirmscurrent.
Publication receipt:evidence/release-1.0.114-publication.json. Queue1/10.
Publication did not rebuild,reinstall or restart runtime.
This fixes the slow-profile selection for an
acknowledged closed word, not every remaining rapid-input compatibility issue.

Only runtime file: src/bin/lay_daemon/kde/firefox_replay_pacing.rs. It selects
the existing bounded fast profile for isolated, freshly scoped Firefox native
replay and an exact leased word followed only by acknowledged Space/NBSP.
No new transport, owner, retry, deadline, candidate policy or verifier change.
Ordinary typing,GNOME,Chrome and terminal code are frozen. An opt-in diagnostic
records only booleans/lengths,never word text. Version-only metadata is114.

Verified source:3611/3611 correctness/package in34targets,including487 process-
isolated executions; canonical wrapper exit0,0semantic/infrastructure failures.
Strict lint/default+research inventories,435Python tests(10skips),Node,shell,
installer,architecture50 and Rust1.88 default+lexical compilerPASS. All10
installed release programs compiled0warnings. Source closure1327files
c195993d4f0b850c5d45c9b6eda213546eec16b5165d1308dcf13e6be998bbbe;
remote synthetic GitHEAD is NOT canonical source authority.
Receipt CROOT/release114-full-v2/result.json and repository evidence/
release-1.0.114-tests.json. Source returns verified against frozen prehashes.

Native scope:full Firefox220=159PASS/21FAIL/40BLOCKED;20rapid-after-Space
late targets remain;1privateiframe ownership refusal precedes this pacing leaf.
Four oldFAIL becomePASS,one oldPASS becomesFAIL before changed leaf,three
emailcases becomeBLOCKED. NetPASS unchanged; no claim all failures fixed.
115rapid-space samples median140.06/p95239.59/max292.76ms; prior historical
184.47/281.80/303.34 is not a single-variable causal comparison.
Paired samehost/currentIME baseline113 vs candidate5samples each:median
161.23->131.55ms,max175.93->244.97,both cadenceFAIL/all5targets. No worst-case
improvement claim. PostTab plain+iframe16/16flips/decoder/heldShiftPASS.

Actual installed114:daemon3962945/ae2844b2ca3867075a38585c38786b8e4c12b4a28fd56b7909c51f6d521b7970;
IME3963028/3f5816621c96e7e3735c1c47c5b8b0aa9dbb21ac99dcd5bf301e5fb8f2c51d02;
tray3963083. Configuration6e62eab3c278c733fedb47e45ef6306105a3700222cf9f7a4e5bff4bd92264d1
unchanged,new fragment modeOFF. Global IBus3168326 and models preserved.
Final exact-byte native2/2roundtripcases=4flipsPASS:Chrome42.28/52.52ms,
Firefox121.62/120.27ms. These small controls do not replace the full matrix.
Receipts CROOT/release114-final-deployment-v1/result.json and
release114-final-native-v1/result.json. No localGNOME install/restart.

Unverified: universal compatibility,remaining20rapid deadlines,privateiframe
C03refusal,fullChrome rerun,GNOME/terminal physical revalidation. No runtime
mutation authority changed. Public commit/tag/updater readback pending.

---

## Preserved preflight and experiment history

# Release114 preflight — KDE Firefox rapid closed-word replay

Status: first current task; not a fixed or released version. Baseline ac50a7819e62c1b8d5d4b74845c7d6c2be060797.
User2026-10-30 requests ten substantive sequential releases without routine pauses.
SOLO/no Harness. No local GNOME install/restart.

Scope: explicitly authorized KDE Firefox field integration; only daemon
kde/firefox_replay_pacing.rs may change runtime behavior. Shared executor,
layout handoff, IME source, GNOME/other clients and all deadlines frozen.
Existing eligible isolated native replay selects an already implemented pacing
profile; it gains no edit permission. ADR freezes32+ adjacent integration files.

Map: exact_ime_tail_replay.rs::execute -> kde/firefox_replay_pacing.rs
-> unchanged text_output/key_emit.rs -> kde/firefox_replay_receipt.rs
-> strict current IME result verification. First failed historical transition:
visible target arrives after next gesture in23/24 failed rows; one missing.
No new claim that every failed row shares a proved internal cause.

Options: reuse existing bounded fast profile9/10 after dependency/control proof;
raise deadline3/10 (hides latency); new output route1/10 (unnecessary).
Use preserved candidate, first causal RED/GREEN, build source-bound daemon,
install only on dedicated KDE and run rejected post-Tab controls + short rapid
prefix before any full matrix. A failure is retained; no retry-until-green.

Historical full matrix: Firefox159/24/37/220;Chrome179/0/41/220.
Source tests never replace native denominator or establish installed bytes.

Evidence/measurements/remaining scope: pending.

RED: release114-pacing-red-v1 selected0tests, setup failure only. Corrected exact
manifest filter in v2 selects1test, fails with expected closed-word eligibility
assertion on accepted function. Remote receipt release114-pacing-red-v2/result.json.
No install. GREEN uses exact preserved candidate leaf, not new executor code.

Native controls v1: iframe-rich postTab8/8 and next-letter/heldShiftPASS;
first ordinary field BLOCKED before input (DOM focus), rapidv1 BLOCKED before
input (owned native accessibility app absent/ambiguous). Not product FAIL/PASS.
Distinct observer evidence: saved frame shows initial page, first field not
focused; r19 launches bareFirefox; current accepted reflow actor uses existing
owned geometry launcher. Reuse that existing launcher with process-local
accessibility, no desktop/global settings or test cadence edits. Retain both
BLOCKED receipts. lay-diagnose actorJSON reports missing controller identity
fields; do not infer causal completeness from file anchor.

Source GREENv2:340/340 canonical hermetic daemon tests154.161s, release daemon
build185.410s,0warnings. Newbyte9c487a4f loaded3721716; IME3131409/30ceff92,
tray/IBus/originalFirefox/config6e62 preserved. Prefix4PASS with actual accepted
production launcher alignment (current space-readback controller). Full220
running; partial13FAIL at162rows means this is NOT a completed group fix.

Full220 candidate completed420.807s:159PASS21FAIL40BLOCKED. Delta vs
historical fixed-cell baseline:4rapid-spaceFAIL->PASS,1privateiframe-rich
consecutive-activePASS->FAIL,3emailPASS->BLOCKED. No net PASS improvement.
Rapid-space115/120latency samples:median184.467->140.060ms,
p95281.804->239.592ms,max303.342->292.759ms. Different current environment
and bytes mean this is observation, not controlled one-variable speed proof.
Receipt CROOT/release114-native-full-v1; comparison CACHE/release114-native-comparison-v1.json.

Existing rapid-dbus-profile-v4 collector initially failedsetup missing tray_pid
in daemon-only deployment receipt; original receipt unmodified. Supplementary
binding adds freshly verified tray3131455. v2 paired native4=3PASS1FAIL,
all5targets; latencies135.1/151.5/164.1/214.5/161.4ms.83daemon freshKWinreads
and7IMEreads; daemon layout-list30calls,one74.69ms. No IBus private-bus calls
observed in measured window; do not blame CLI without boundary evidence.
Source current pacing selection is not recorded, so fast eligibility isUNKNOWN
in actual gestures. Next extend existing opt-in trace ONLY in the allowed
pacing leaf with boolean eligibility/result, no text, extra RPC or deadline
change. This distinguishes ineffective eligibility from other stage cost
before any protected shared-route proposal. Runtime currently candidate,
config restored, globalIBus/IME/tray preserved.

Causal diagnostic:9pacing trace records for current3803109, all scoped=true,
fast=true;7closed-word records suffix7/original6. Eligibility is reached.
Do not re-edit that predicate to chase remaining delays. Earlier per-keyKWin
claim corrected: X11Output::prepare validates XI devices/focus, NOT KWin.
Actual profile splits5gestures: pre-suppression67..191ms (9KWinreads each),
suppression-to-ACK45..74ms (5KWinreads each). No unusedCLI optimization.

Separate fullrow155 first failure: IME ibus_manual_toggle_rpc stage not_handled
reason=context_authority, marked-preedit witness absent. It precedes delegation
to this pacing leaf. Root still unassigned; retain as separate queuedC03issue.
Do not claim its repair in this performance release.

Release scope narrowed to measured latency reduction only, NOT closing all
rapid failures. Kept opt-in boolean-only pacing diagnostic now uses existing
cached config accessor, no per-gesture new config-file read. Version-only
metadata114 prepared locally; no local GNOME deployment. Full release gates
and paired native comparison pending.

PostTabcontrols v2with explicit process-local a11y:2/2PASS,16/16flips +
next-letter/heldShiftPASS. Same unchanged actor6857. Samehost paired comparison
original113daemon median161.23ms/max175.93 over5 vs candidate median131.55ms/
max244.97 over5; all5targets each, both rapid-deadlineFAIL. Small sample; no
p95/maximum improvement claim from this pair. First finalsource gate refreshed
graph54.257s then discoverycompile failed: attempted privatecached accessor is
not exported from config_runtime. Restore the already compiled LayConfig::load
call in the SAME leaf; no shared visibility change. Source-only failed build
never installed. Retain full-v1 as failedsetup, rebuild gate after correction.

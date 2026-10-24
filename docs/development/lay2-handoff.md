## Active repair plan — 2026-10-24

Continue [the ordered repair and release plan](weekend-repair-plan-2026-10-24.md).
The user authorized completing build/CI/security fixes, all three open public
issues (#69, #63, #47), testing, publication and reporter replies without further
questions. No agents or Harness2. Do not interrupt the user's local update.

2026-10-24: source correctness/package 3511/3511, performance11/11,
MSRV1.88 both configurations, strict lints, audit and real ten-binary release
build PASS. Remote KDE runs exact1.0.96 for CLI/daemon/IME; all loaded hashes
match the final compiler artifacts. Source/full-observation and the metadata-only
removal of the old `.1` IME label are retained under
`tech_debt/evidence/release-1.0.96-{test-observation,verification}.json`.
KDE Chrome frozenr18:168P/10F/42B of220; Firefox with owned-process accessibility
activation:81P/103F/36B of220. The default Firefox observer attempt remains a
separate immutable BLOCKED receipt. These red native results do not certify
complete KDE compatibility. GTK/Qt/terminal and issue47 flag controls are next.
The weekend task supersedes the older Chrome-only testing hold below. No
manual local GNOME installation/restart, Harness2, stale-authority bypass or
repetition of the rejected key-time Space experiment is authorized here.

## Current workflow override — 2026-10-23: Harness2 removed

The user explicitly removed Harness2 from Lay. Continue directly in the current
session. Do not launch its adapter or doctor, follow LAY2_HANDOFF.md, or treat
older pending-Harness-review steps below as current requirements. Historical
reports are evidence only. This overrides the earlier mandatory-use agreement
and recalled memory. Architecture, source checks, release/resource guards and
native client acceptance remain required. Accepted GNOME runtime is preserved;
the active task remains GNOME/KDE isolation, then Chrome only on KDE.

## Current publication — 2026-10-24: public stable1.0.95

The user explicitly requested publication after narrowing the visible KDE
symptom to the unresponsive tray; this supersedes older private-only holds.
Public tagv1.0.95 points to036f25e52999ce3543ae0c0937e7f27777945c98,
published2026-10-24T11:21:53Z:
https://github.com/radislabus-star/lay-public/releases/tag/v1.0.95.
Version/package/menu metadata is1.0.95. All801Rust files still match retained
run-8ddt9w31; the later tray export line and version metadata are separate.
Updated graph binds761src Rust files; remote guarded architecture check PASS.

Actual installed updater entrypoints on local GNOME and remote KDE resolve to
~/.local/lib/lay/update.sh. Both `lay-update --check` returned0 after publication
and reported installed1.0.94 -> stable1.0.95. Nothing was manually installed,
restarted or typed for publication. Loaded IME evidence at11:24:34Z:
GNOME PID2711382 SHAfa5e8950cc4c8c39e108464c9d06da0c00a32d26782f6dc2c030007f70e49a44;
KDE PID3349193 SHA2100cb595447fc2c7e034ed10fc6d6f8580b34a6191a9ba9c55eeba15703126a.
The GNOME PID differs from older handoff entries; accepted bytes are the same.

GitHub checks are NOT GREEN. CI40089564155: installer regressions and MSRV
jobs PASS; main correctness lane compiled but stopped on test-manifest drift.
Security40089564154: Audit locked dependencies FAIL (RustSec advisories).
Do not suppress advisories, remove tests or silently retag this published tag.
See docs/development/release-1.0.95-publication.md and release notes for limits.
Open PRs checked during publication:64,65,66,67,68; none merged or upgraded.
The real KDE tray click remains unresolved despite correct menu DBus export.
The user's latest clarification was tray-only, with Lay functionality possibly
working; do not replace that clarification with a blanket whole-KDE failure.
No new full browser/terminal/GNOME acceptance was performed for publication.

## Live desktop follow-up — 2026-10-24T10:49:46Z: Firefox frame now reaches Remmina

The user reported that Remmina still showed Chrome after Firefox was opened.
Do not equate an active X11 window or a server screenshot with client visibility.
Firefox302931 is running with the existing normal profile; visible windows
65011733 and newly opened65011889 are on KDE DISPLAY:71. The new window was
unmaximized to1360x740 and raised. No user browser/profile was closed or erased.
The existing Remmina connection was reconnected; its screenshot still retained
the preceding fullscreen frame. Server capture showed the new smaller window.
Then x11vnc remote control `fb` and `reset` recreated its framebuffer/polling
memory; a screenshot taken through Remmina now shows the new Firefox window.
Local proof: /home/ubu/.cache/lay/development/firefox-after-vnc-reset-20261024.png.
This proves frame delivery after reset, not the underlying cause of the stale
buffer, continued update stability, user acceptance or Lay functionality.
The panel clock remains12:57; do not claim that the entire desktop was frozen.

Reverified current engine3349193 SHA2100cb595447fc2c7e034ed10fc6d6f8580b34a6191a9ba9c55eeba15703126a;
daemon2167138 and tray337494 are active. No engine/daemon/IBus/GNOME restart,
input-route change, publication or synthetic text input in this follow-up.

Earlier in this task the private desktop reached504/512 tasks and rejected
forks. Only its runtime TasksMax was raised to1024 (now510/1024); CPU/RAM caps
were preserved. Remote receipt: /home/e/.cache/lay/kde-desktop-thread-limit-repair-20261024.json.
The KDE-only tray fix adds `self.tray.setContextMenu(self.menu)` after menu
construction in scripts/lay-kde-tray.py:376. Installed script SHA3f936370408e513b9ed764f087713a04e9521d9b31e475f5d53267c05be04e10;
native DBus menu now exports real actions instead of an invisible separator.
Remote receipt: /home/e/.cache/lay/kde-tray-menu-repair-20261024.json.
Source verification:7PASS/1SKIP in tests.test_kde_daemon_control; actual icon
click acceptance and final derived graph refresh for this Python change remain
unverified. No Rust changes were made for the tray/desktop incidents.

One isolated r18 tab smoke per browser: Chrome1PASS; Firefox1BLOCKED before
typing (`owned native application unavailable or ambiguous`). Receipt:
/home/e/projects/lay-development-runner/kde-install-control-20261020/kde-user-live-tab-smoke-20261024/result.json.
Fresh-profile Chrome PASS does not resolve the user's ordinary-profile failure.
The last full retained2100 matrix below remains historical, not a new run.

## Retained core state — 2026-10-24: retained2100 restored; Space trial rejected

CURRENT remote KDE private1.0.94.1 engine3349193,
SHA2100cb595447fc2c7e034ed10fc6d6f8580b34a6191a9ba9c55eeba15703126a,
restored2026-10-24T03:43:33.583365UTC. Receipt:
chrome-941-keytime-space-rejected-rollback-20261024/result.json. The retained
compiled artifact is hash-identical to the earlier full220 tested bytes.
All801 active Rust sources match tested run-ah5rCj/localrun-8ddt9w31;
only our two trial files were restored from its verified archive. New source
and regression test retained at localpacket/rejected-keytime-source,not lost.
Source proof51native+3desktop PASS,630P/15oldF,sameTD121coverageerror.

LAST FULL FOR THESE BYTES:172PASS/9FAIL/39BLOCKED/0NOT_TESTED,432.247s,
chrome-941-field-continuity-reviewed-full-20261024. All three manual gesture
classes0FAIL. Remaining3Space ENRU+6moved-boundary;upper real omnibox is a
separate unresolved failure,not included in220. Do not claim full acceptance.
After restoration,frozenr19 rich two-word diagnostic1PASS:
проверка привет -> проверка ghbdtn -> проверка привет. Receipt:
chrome-941-keytime-rollback-two-word-20261024. Same restored bytes/config;
this single diagnostic does not add220-matrix rows or rerun the full proof.

REJECTED trial8a66b576988efdeb2e04710c89681e84259d2c6f9f34e06a4af73f60c9de00fc:
key-time Space preparation after native installation. Source exact RED became
GREEN52native/631P15oldF,but native full220 regressed169P/10F/41B:
ENRU6P2F2B vs5P3F2B,pair0P8F2B vs2P6F2B,manual0FAIL but2PASS->BLOCKED.
Same frozenr18/cadence,432.106s,stable runtime/config. Therefore not promoted.
Do not reapply this placement,increase waits,or accept stale leases without a
new demonstrated first-transition solution and aggregate/per-class evidence.
Receipts chrome-941-keytime-space-{build,deployment,smoke,full}-20261024;
local keytime-space-summary.json,source-reversion.json and raw client records.

Fresh upper-omnibox test on rejected8a66 produced own suffix ерка for both
пров and проверка пров,but both had rect:null,hence2hintFAIL andTabBLOCKED.
No guessed caret,Google hint PASS or absence-of-candidate claim. Receipt:
chrome-941-keytime-space-omnibox-20261024. Existing proposal at
chrome-omnibox-release-proposal-2026-10-24.md is UNAPPROVED:two tiny common
release dispatch calls outside the KDE folder;no common/GNOME edits allowed
until user answers. The user already received that scope question;do not
send another duplicate. No agents/Harness/publication or local GNOME restart.

20protected remote processes/config/tray unchanged in rollback;daemon/IBus
not restarted. Current compiled source and native verdict are explicitly
separate from rejected trial diagnostics. Final derived graph refresh PASS;761src Rust and all801tested Rust hashes verified;receipt chrome-941-final-retained-graph-20261024/result.json.

## Previous2100 full result — 2026-10-24: manual recovered;nine functional failures

CURRENT private KDE1.0.94.1 engine3112354,
SHA2100cb595447fc2c7e034ed10fc6d6f8580b34a6191a9ba9c55eeba15703126a,
installed2026-10-24T02:46:18.066790UTC. Source run-ah5rCj/localrun-8ddt9w31,
51native+3desktop PASS,630P/15oldF,sameTD121coverageerror. Reviewed build/deploy
chrome-941-field-continuity-reviewed-{build,deployment}-20261024.20protected
processes/config/tray unchanged;engine-only restart. No daemon/IBus restart,
agents/Harness/GNOME changes or publication. Old9c00 build never installed.

Frozen r18 FULL220 completed172PASS/9FAIL/39BLOCKED/0NOT_TESTED in432.247s.
Receipt:chrome-941-field-continuity-reviewed-full-20261024. All manual gesture
classes have zero FAIL:consecutive-two53P/0F/7B,roundtrip51P/0F/9B,
rapid-five53P/0F/7B. Space ENRU5P/3F/2B,moved-boundary2P/6F/2B,
Tab8P/0F/2B,inverse0P/0F/10B(TD137 stops before shared-learning mutation).
Compared withad5e153/25/42:18F->P,2P->F,4B->P,1P->B;two regressed Space
cases(search,iframe-rich) remain failures,not waived. Remaining9 rows:
9text-pair,17search-ENRU,39tel-pair,59textarea-pair,69rich-pair,
79plain-pair,87iframe-input-ENRU,97iframe-rich-ENRU,99iframe-rich-pair.
Same2100PID/bytes/config restored. Manual successful samples473,
median49.623ms,p95linear64.097ms,max94.288ms;success-only,not an
end-to-end or matched-case speedup claim. Threads12,zbus4,FD10,pids.max512,
pids.events max0,pids.peak19;no cgroup memory-pressure/OOM events.

Frozen r19 two-word diagnostic completed1PASS/0FAIL/0BLOCKED in5.789s:
rich active exact surfaces проверка привет -> проверка ghbdtn ->
проверка привет,65.032ms/59.067ms. Receipt:
chrome-941-field-continuity-reviewed-two-word-20261024. Diagnostic only,
not an extra full-matrix row. Upper real Chrome omnibox remains separateFAIL;
no geometry/style/user acceptance claim. Its common two-hook proposal remains
UNAPPROVED;do not edit common dispatch or GNOME. No release/push authorized.

Short20 prior proof14P/4F/2B,48.205s,with four actual constrained forward
field_receipt_rebound events and zero install refusals. Three rebound
roundtrips PASS. Receipt:chrome-941-field-continuity-reviewed-smoke-20261024.
Full,short,two-word and resource receipts copied to local packet:
/home/ubu/.cache/lay/development/chrome-941-review-20261024/.

Retained diagnostic (no longer running):private KDE1.0.94.1 engine2911086,
SHAd9ed79c6a37a082b94f751f02c2a35c2df3400bbc962e65e2a84098c548aff77,
installed2026-10-24T01:53:17.966320UTC. Source run-0uAfsx/localrun-uo6qy460:
49native+1desktop PASS,626P/15oldF,sameTD121coverage error. No functional change
sincead5e:exact transient-input predicate order/calls/results retained,with
metadata of the actual failed clause. Build/deploy
chrome-941-transient-gate-diagnostic-{build,deployment}-20261024.
20protected processes/config/tray unchanged;engine-only restart,no daemon/IBus
restart. No agents/Harness/GNOME changes/publication.

Completed one selected44 diagnostic:36P/6F/2B,110.789s,Search/Rich,normal/private/fullscreen,same
r18 case bodies/cadence;only field selector/manifest changed. Actor
browser-native-chrome-transient-guard-diagnostic-20261024-r20.py,
SHAd28155fa2d3b7a9fc165433b5333cf74fddb14a0884c50dbb20bfa8ca07c5dc6.
Control root/chrome-941-transient-gate-diagnostic-selected-20261024. This is
diagnostic only,not the full220 proof. Do not replace153P/25F/42B full totals.
All45transient input gates allowed retention;there is NO failed predicate to
remove. The actual lost key is the NEXT read:row41press3866 source_read succeeds,
then install_refused/field_identity_changed with ONLY desktop_matches=false;
AT-SPI bus/owner/application/field alltrue. Canonical KWin desktop identity
contains local provider lifetime,which changes after errors. The two actual raw
IDs are not in the trace,so lifetime-only attribution remains conditional.

New decision2026-10-24-kde-native-field-transport-continuity.json:within KDE,
one explicit forward-continuity helper for fresh native fields;keep PID,bus,
KWin owner,UUID,AT-SPI identity and canonical lease guards;reject older lifetime.
Raw provider/notification receipt identities stay strict. Added RED tests for
fresh same-field reconnect plus production key router and9negative identities.
RED run-PwVFdo/localrun-hmqr4nn9:627P/16F,only new failure is the exact fresh
install identity guard;9negative identities pass. KDE implementation prepared:
desktop/window_identity.rs projects physical equality and forward receipt order;
NativeFieldIdentity Eq is physical,same_receipt_as stays strict for observers,
continues_from guards fresh install/preflight. Common final verifier source
unchanged and still requires current token/document/exact snapshot. Four KDE
files only(new module+desktop reexport+native adapter+tests). GREEN pending;
installedd9ed still diagnostic. GREEN v1 run-PbBWkH/localrun-kpj37uic has
630P/15oldF,sameTD121error;51native+3desktop PASS,including production next-key
router. Built9c0086a806bc802262274432519f960859812263167415959421443ae0fe84f4
at chrome-941-field-continuity-build-20261024,NOT installed. Review repaired
C08 stale completion:older same-field receipt is rejected WITHOUT clearing the
newer binding/token/scope/epoch. Strengthened that negative assertion. Source
recheck running;reviewed build/install controllers use prefix
chrome-941-field-continuity-reviewed (still expect oldd9ed/PID2911086).
Do not install v1. Do not edit canonical provider outside KDE or remove guards.

Retained functional trial (no longer running):private KDE1.0.94.1 engine2749711,
SHAad5e9debdef716518e513f6d33a463f9c071c3070e45a5a3a7591d5ca48e463a,
installed2026-10-24T01:10:14.050514UTC. Build/deploy
chrome-941-input-isolation-{build,deployment}-20261024. Source49native+1desktop
PASS,626P/15oldF,sameTD121error;source run-aEBiNt.20protected processes/config/
tray preserved,engine-only restart;daemon2167138,IBus2155328,Chrome2170966,
tray336128 unchanged. No publication/agents/Harness/localGNOME changes.

Frozen short20 chrome-941-input-isolation-smoke-20261024:14P/4F/2B,46.595s.
Two UnknownObject records are followed by managed CommitText and successful
Double Shift (text/consecutive-two and iframe-rich/rapid-five). The earlier
claim of same-callback native causal proof is WITHDRAWN:adjacent records do not
prove the same callback,and full native failures still contain missing letters.
Candidate is NOT accepted
overall:versus previous short20,text moved-boundary F->P but text Tab P->F when
KWin observer is unavailable at Tab. No edit is permitted without fresh proof.
The provider availability defect itself is NOT fixed. Full220 completed:
chrome-941-input-isolation-full-20261024,153P/25F/42B/0NT,448.674s,same frozen
r18 actor/hash/cadence. Versus6e45:13FAIL->PASS,16PASS->FAIL,1BLOCKED->PASS,
2PASS->BLOCKED. Remaining25=14manual+3ENRU Space+8moved-boundary;Tab8P/2B.
Success-only432manual samples:median47.150/p9562.061/max89.445ms. Samead5e
runtime/config before/after. No overall improvement and no acceptance.

All14manual failures still have a5-letter model for6-letter browser text;
12contain native_desktop_unobserved,2native_deadline. The flat semantic source
fixture does not establish why the live key callback refuses retention. Added
opt-in transient_input_gate metadata to KDE native_document.rs:exact failed
clause(handoff/token/scope/binding/notification owner),callback-entered and
existing owner flags. Boolean order/calls/decisions unchanged,no new RPC.
Diagnostic source/build passed the same controls and installedd9ed;selected44
is running. Do not remove a failed guard or infer which one failed yet.

Retained private KDE1.0.94.1 diagnostic engine2644205 (no longer running),
SHA42aba9cdf6c582eb568824c4f23083e1c6652b9905cd20ae39f4fdfd8eaf0916,
installed2026-10-24T00:43:46.147992UTC. Source run-x8hCpM/localrun-uzf7t15q:
47native PASS,623PASS/15unchanged baseline FAIL,same TD121 coverage error.
Build/deploy chrome-941-focus-diagnostic-{build,deployment}-20261024.
Only new runtime delta since previous6e45 is error metadata in kde/desktop.rs,
same Option/result/authority paths.20protected processes/config/tray unchanged;
no daemon/IBus restart. Frozen r18 short20 native diagnostic completed14P/4F/2B,
48.377s:chrome-941-focus-diagnostic-smoke-20261024. All3desktop errors are
provider_unavailable/UnknownObject /1;no Busy observed. KWin5.24.7 uses list
size for IDs and deleteLater for unload;actual lifecycle overlap remains
unproven. Do not claim focus theft or implement a Busy-only fix. Full220
totals remain previous6e45,not this short diagnostic.

New decision2026-10-24-kde-native-observation-input-isolation.json:KDE-only
classification of unknown desktop observation;reuse transient-read handling to
retain canonically admitted ordinary key history with NO edit/display proof.
Known changed window/field/owner remains hard refusal. No common hooks/route/
provider mutation. Added semantic RED tests(ordinary CommitText,exact old/new
snapshot recovery,11negative owner guards). RED run-J6gkKn/localrun-1b6lydbo
has623P/17F:exactly2expected failures at the old refusal,same15baseline failures
and TD121 error. KDE-only runtime implementation now prepared:typed unavailable
desktop result plus existing transient-read handling;one actual occupied-provider
facade control. GREEN run-aEBiNt/localrun-1k5v07l1:49native+1desktop controls
PASS,626PASS/15unchangedbaselineFAIL,sameTD121coverageerror.800Rust files
compared with installed42ab source request;only3KDE files differ. Build and
installation ofad5e completed under the existing remote guard;no rebuild during
installation. The whole native matrix is not yet accepted.
Common-release experiment approval still pending and independent.

Retained previous functional candidate engine2537115,
SHA6e45a482b4439f84e81aa28d14823f1b034a93c3340a55900e44d23b1d1196d2,
installed2026-10-24T00:18:43.107145UTC. Source run-KlCYnz/localrun-_x7mnrbq:
47native controls PASS,623PASS/15baseline FAIL/442filtered,sameTD121coverage error.
Only additional runtime change:KDE native-document eligibility, then existing
canonical Space preparation before the manual older-prefix display gate.
Same-text snapshot clearing already works;no cache/revision runtime change.
Build/deploy chrome-941-space-word-proof-{build,deployment}-20261024.
20processes/config/tray preserved;engine-only restart,no IBus/daemon restart.

Frozen r18 single exact text-Space case PASS,actual привет(space),authorized
canonical existing frame;replacement29.865ms. Receipt control root/
chrome-941-space-word-proof-text-space-20261024. In this one trace the available
frame also has manual_word_current=true,so it does NOT independently prove
native benefit from the newly separated older-prefix guard. Do not overclaim.
Frozen FULL220 completed:chrome-941-space-word-proof-full-20261024,
157PASS/22FAIL/41BLOCKED/0NOT_TESTED,446.251s. This candidate is NOT accepted:
versus previousDFA163/19/38,15PASS->FAIL,12FAIL->PASS,3PASS->BLOCKED.
Remaining22=14manual+3ENRU Space+5moved-boundary. All8supported Tab rows PASS.
Manual success-only433samples:median48.878/p9563.127/max92.825ms.
Runtime/config restored and identical;12threads/4zbus/10fds stable,no task/OOM
events. Local evidence:~/.cache/lay/development/chrome-941-review-20261024/
space-word-proof-{full-client,full-runtime,summary}.json. The graph's760Rust
hashes match this installed source;4derived graph files synchronized locally.

One exact newly failing manual row2(normal/text/roundtrip/active) first loses
the managed letter в:ProcessKeyEvent242 accepts admission, then native read
returns native_no_desktop2460us;release243 refused_timeout. Browser contains
привет but model only5letters,so final manual context_authority refusal is
correct. Do not bypass it. Provider error was suppressed by kde/desktop.rs
read_until(...).ok()?. Added opt-in metadata-only error trace there and installed
the identified42ab diagnostic. Same None result and authority/delivery unchanged.
Next:collect short20 to distinguish actual KWin provider errors. Do not guess
Busy/focus theft or claim an unchanged full matrix for these diagnostic bytes.

Current daemon2167138/IBus2155328/ordinary Chrome2170966/tray336128 unchanged.
Upper address bar is an additionalFAIL outside220:native char/range rectangles
allzero,hit tests-1,no font metrics. Subscription/read succeeds,IBus position
lags final managed letter. Release experiment remains PROPOSAL ONLY pending
user clarification for2common hooks;do not edit them without an answer.
Duplicate UI requests concern the SAME question,not two approvals. No agents,
Harness,push/release or localGNOME runtime changes. Preserve this scope.

### Retained DFA full proof and diagnosis



Installed private KDE1.0.94.1:engine2328263,
SHAdfa2f7cf86512d2597c5a9abb36349f4ba98af46a4afc3adc8f5d1957da831a3,
installed2026-10-23T23:23:03.983214UTC. Source run-L1h9GM/localrun-u8vxxgx7:
45native controls PASS,621PASS/15baseline FAIL/442filtered,same TD121 error.
Control root/chrome-941-observer-trace-{build,deployment}-20261024.20protected
processes/config/tray preserved during that install. Current daemon2167138,
IBus2155328,ordinary original-profile Chrome2170966. No release/push/agents/
Harness/local GNOME install. Local graph updated from exact remote AST update,
all760 Rust source hashes matched DFA then;new Space proof source graph
will update during its build. Original graph backup retained.

Corrected diagnosis:observed_suffix_chars is word history, NOT document length.
Earlier claims that the native document read only3chars were an interpretation
error and are withdrawn. Actual upper text/caret4/no-selection is observed;
AT-SPI char3/char4/range4..4 gives zero geometry, hit tests give-1, attributes
only colours/direction. IBus report remains at the prior position. Subscription
succeeds351/595us, events reach actual reads. Measured root is unavailable exact
native geometry; a stale coordinate must not be shown over typed text.
Receipts:chrome-941-omnibox-{observer-trace,geometry,native-capabilities}-20261024.

Proposal docs/development/chrome-omnibox-release-proposal-2026-10-24.md:8/10
bounded original-letter-release experiment, no generated key or text delivery
change. All policy in KDE, but two common dispatch hook sites are required.
Respect user's KDE-only boundary:clarification required before those edits.
This hypothesis is NOT implemented/native accepted. Do not add caret guessing,
synthetic keys, weaken word authority or blindly copy Kitty's lifecycle wake.

Frozen full220 on currentDFA completed:163PASS/19FAIL/38BLOCKED/0NT,
control root/chrome-941-observer-trace-full-20261024. Same actor hash/cadence,
446.581s,engine before/after identical. Rapid-five53PASS/7BLOCKED/0FAIL.
Remaining:8boundary-pair+4ENRU Space+7manual;all7manual fail at first gesture.
Case transitions versus8d99:22FAIL->PASS,6PASS->FAIL,4BLOCKED->PASS,1FAIL->BLOCKED.
Thus aggregate improved but6prior passes still fail;NOT stable/complete.
Manual successful timings461samples:median48.449/p9563.329/max94.633ms.
Resources11->12threads,4zbus/10fds stable,pids.events0,memory events0.
Receipt control root/chrome-941-observer-trace-review-20261024.json.
Upper bar remains additionalFAIL,not part of220. One common-hook approval is
pending (duplicate UI question refers to the SAME decision;any answer applies).
New KDE-only source RED tests check same-text postCommit observation and
independent canonical Space proof despite older-prefix manual refusal.
Runtime not changed during this source investigation.
RED run-2VIaLv/localrun-xmziojcl confirms canonical Space work is blocked by
manual older-prefix gate.GREEN source run-KlCYnz/localrun-_x7mnrbq:47/47native controls PASS,
623PASS/15baseline FAIL/442filtered,same TD121 error. Build in progress.
Only runtime change:KDE native-current/unsealed/empty composition gate, then
canonical Space scheduling, then existing manual guard for display. Cached
snapshot/observation revision logic unchanged:managed commit already clears
its snapshot;the second new test fixture was corrected to reflect this.
New controllers build/install-chrome-941-space-word-proof.py expect oldDFA
PID2328263/native47;not executed. Common-hook approval remains pending.

## Current acceptance sequence — 2026-10-23: KDE Chrome 1.0.94.1 first

The user explicitly requested sequential development installations on remote
KDE, without a GitHub release: finish Chrome, install/display 1.0.94.1 for the
user's own acceptance; only after that acceptance move to Firefox 1.0.94.2,
then other KDE applications/terminals one at a time with successive revisions.
Do not skip ahead or publish these trial versions. Keep accepted GNOME bytes.

Current Chrome priority is two consecutive Double Shift gestures without any
intermediate decoder key, then five consecutive gestures, active/one-Space
words and the existing ten-field/three-window-mode protocol with timings.
The old roundtrip inserts a decoder key between gestures; retain it but do not
claim that it proves the newly requested uninterrupted pair.

### Active work — 2026-10-24: resolve the remaining36 FAIL

The user explicitly requested continuing the36 Chrome failures; do not wait
for manual acceptance or move to Firefox. The last complete baseline is8d99,
full frozen r18 matrix143PASS/36FAIL/41BLOCKED; installed runtime is in the
Current state block above. Add a SEPARATE Chrome upper
address-bar first/second-word test:the user confirmed the browser omnibox,
not page URL field3. The existing220-cell page matrix does not cover it.
Preserve the current ordinary Chrome/profile/review text; use an owned target
for synthetic input and do not submit an address/search with Enter.

Read-only failure census at remote control root/
chrome-941-fail36-cause-census-20261024.json groups16functional+20manual.
Only10/20 manual failures contain a source-seal epoch mismatch;6/13 matched
PASS controls do too. Reset alone is NOT established as a common cause.
Space EN/RU representative has worker_generation0 and every key's Space frame
absent. A later actual native receipt already satisfies the EXISTING canonical
revision-bound exact_managed_surrounding proof, even with UnknownStart. The
native refresh currently queues display only. Test that it also schedules the
existing Space worker against that exact canonical frame; never promote scope,
fake exact-refresh capability or use a manual-only grant for automatic edits.
RED confirmed:run-kWdQMd/localrun-0nkcsvff, positive fails only because no
work is queued; twelve negative controls pass. Source618passed/16failed.
KDE-only producer now calls the existing canonical frame capture/scheduler
before display recovery. GREEN source:run-HTssSt/localrun-8303g4x0,
43/43 native controls PASS;619passed/15baseline failures/442filtered,
same TD121 coverage/stack error. Private candidate build/native pending.
Decision2026-10-24-kde-native-space-prefetch-handoff.json. GNOME, delivery,
ranker and common verifier unchanged. Keep version1.0.94.1/private; no release.

Upper-address-bar baseline on installed8d99:control root/
chrome-941-omnibox-baseline-20261024/result.json. Own fresh hint appears for
first/second word in the same actual omnibox; Tab yields exactly
проверка проверка(space). Hint116.152/125.352ms,Tab46.679ms. Three logic checks
PASS in one owned window of the ordinary Chrome632014 profile; Enter never
sent, config/engine/focus restored. Screenshots retained. Styling and
intermittent absence are NOT closed by this single control. This is separate
from the220 page-field matrix, which remains143/36/41 on baseline8d99.

### Retained source proof — 2026-10-24: notification arm versus word proof

Installed656f remains current. Upper Tab-first native check fails already at
first prefix:actual field4chars versus final refresh3chars/tail4, suffix erka
rect:null. Passive exact-peer AddMatch records the final TextChanged on the
actual field; no Registry.RegisterEvent or activation flag in this observer.
No new event type is justified. See control root/
chrome-941-omnibox-{tab,events}-candidate-20261024. Both owned windows now close
with WM_DELETE and Chrome stays alive. Test code resolves daemon/profile PID
dynamically; old hardcoded PIDs are invalid.

Observer arm uses binding.is_current (word/edit proof), whereas its existing
wake predicate observer::current explicitly survives an ordinary key epoch.
RED confirmed run-DiUn2V/localrun-ky2nxh7s:619PASS/17FAIL/442filtered,
same TD121 coverage error. Only two added failures:CommitText suppresses
needed notification subscription;old predicate misses field-owner refusal.
needs_subscription now reuses existing observer::current;every word/edit
check stays unchanged. GREEN run-iUXikS/localrun-2i951l8k:45/45native
controls PASS,621PASS/15baseline FAIL/442filtered,same TD121 coverage error.
Observer-arm build currently running; install/native pending.
Decision2026-10-24-kde-native-observer-subscription.json. Only KDE observer.rs
and its tests change; no edit guard, common module or GNOME change.

### Retained trial — 2026-10-24: Space producer656f; Chrome incomplete

KDE engine2167556/656f908e136787d3cb6007d5933cb3b48d0c23248d1abe9ccfea66a429f56a3e,
private1.0.94.1 installed2026-10-23T22:43:31.615577UTC. Source43/43 native
controls PASS, same15old failures/TD121 error. Build/deploy at control root/
chrome-941-space-prefetch-{build,deployment}-20261024. No release/push.
R18 fixed20 on these installed bytes:11PASS/7FAIL/2BLOCKED. Text EN/RU Space
now PASS; iframe-rich EN/RU and both moved-boundary rows still FAIL. Four
manual failures remain in this short run. It is not a complete native
acceptance and does not replace the prior full220 143/36/41. Resource after:
12threads/4zbus/10fds,Tasks512/events0,memory high/max/oom0.

Diagnostic recovery:the private omnibox actor incorrectly used XDestroyWindow
via xdotool windowclose. Chrome retained the destroyed test windows as focused
native nodes402 and504 alongside the real empty field335. GetActions on a
retained Close button disconnected the native peer and Chrome632014 exited.
Do NOT reuse XDestroyWindow or invoke actions on these destroyed nodes. Actor
cleanup now sends ICCCM WM_DELETE_WINDOW to the still-live owned XID and
requires its disappearance; no destructive fallback. Fresh actor window URL
has a unique owned fragment. Preparation may wait750ms for actual native focus;
no fixture text before readiness. Source/product focus guards were not relaxed.

Whole KDE desktop previously exhausted512 tasks, then IBus3278176 exited.
Temporary TasksMax1024 did not restore the dead server; this is not a proven
sole cause of its exit. KDE IBus now2155328 in its own bounded recovery unit.
Same-byte daemon restarted2167138/hash1802bb5e... with per-unit runtime
UnsetEnvironment=IBUS_ADDRESS, avoiding the dead inherited endpoint. Runtime
contract/source routing unchanged. Candidate engine then installed normally
using OwnsActiveText=false; installer preservation flags apply only to that
installation, not this preceding recovery. Other-user IBus1233115, KDE desktop
and tray336128 preserved. Accepted local GNOME untouched.

Chrome restored from original profile in lay-kde-chrome-session-20261024.service,
PID2170966, first restored window0x2e00002. Same profile, no new profile or
force-accessibility flag. Session metadata backup only:3files/116876bytes;
control root/chrome-941-browser-session-recovery-20261024. Browser gets an
independent bounded application unit (300%CPU/4GiB/512tasks); original whole
KDE TasksMax512 restored. Old review XIDs/Chrome632014 are invalid now.
Old hardcoded daemon3282573 is invalid:resolve current systemd MainPID.
Separate omnibox Tab-first candidate check is in progress. Do not claim
intermittent/styling acceptance from a logical hint snapshot alone.

### Retained complete baseline — KDE Chrome8d99; acceptance FAIL

Installed2026-10-23T20:59:18.806122UTC, PID1810604, engine SHA256
8d99bb1bcc869c00761e95c516657d68ece1398b9235b192434d264939fcbe02.
Installed file, activation entrypoint and actual process match.21 protected
processes/config preserved; daemon/tray/IBus/desktop not restarted. Source
HEAD2ec7392a4437586865f8dc4d951b4f71ff04dcb4, archive939fe6f0054d787e579c3cfec15cc953bbb45061a6de1fc853afa6ae180eb2da.
Local source check run-b_rr6rz2/remote run-7PoXXO:41 native controls PASS,
617passed/15old named failures/442filtered and the same TD121 missing-status
coverage/stack issue. Full source gate remains FAIL, not release green.
Architecture/build PASS. Local graph copied from the tested remote update;
all760 Rust source hashes match. Seven Rust changes versus run-RxUsup: args
trial label and the KDE native_document adapter/bus/bus tests/presentation/
observer/tests. No common verifier, delivery contract or GNOME behavior change.
Native reads explicitly require supports_native_prefix(current Chrome PID).

Real-peer notification RED:sequential watch_changes blocks a fresh snapshot
behind64 TextChanged messages in capacity8 while engine refresh is busy,
returning native_deadline250ms (run-iXKM7W/localrun-97q6of_8). GREEN:the same
regression passes when continuously draining notifications and coalescing a
dirty refresh, without treating event payloads as proof. No extra OS threads,
queue/deadline increases or authority bypass. Source controls41/41 PASS.

Actual frozen r19 second-word pair passes on installed8d99, no intervening key:
проверка привет -> проверка ghbdtn -> проверка привет,62.056/41.497ms. Same
actor failed its first gesture on4fa pooled transport before the drain fix.
Do not promote this one diagnostic into complete Chrome acceptance.
Frozen r18 full220 now143PASS/36FAIL/41BLOCKED/0NOT_TESTED in465.95s. Actor
SHA17dc93b5f2769d5dde62bfe8f5a539fb68af40744139e823c5f18a848b4ce7e6.
Previous906 full was146/34/40:aggregate client improvement is NOT established.
36FAIL=16functional corrections (8 Space EN/RU,8 moved boundary) and20manual
rows (roundtrip7,rapid-five8,consecutive-two5).41BLOCKED=10TD137 native inverse,
24HTML URL/email separator limitations,7native/DOM setup disagreements.
Successful manual rows provide402 timing samples:median49.649ms,p9563.878,
max93.987ms; failed rows are not latency successes. Source fixes alone do not
certify the complete native matrix. One retained failure (normal/rich/closed
roundtrip,row65) shows Reset re-receipt rejection then source-seal refusal with
tail_epoch_matches=false. Its first causal producer still needs isolation;
do not assume it is a styling error, remove the guard, promote UnknownStart,
add a word exception or implement an unverified extra owner.

Resource proof over full220:11->12 threads (one additional Space prefetch
worker),zbus executors4->4,fds10->10,TasksMax512,pids.events max0,peak19,
memory high/max/oom0. Previous906 resource RED:510threads/502zbus executors,
508fds/499 accessibility sockets and2872 task-limit failures. The connection
exhaustion fix is measured; remaining lifecycle/client failures are not fixed.

After the long run, cold/same-field first/second word checks pass4/4 in text
and rich, using the unchanged r9 case/cadence with only provenance receipt
updated in r10. Hint51.694/80.017/51.245/66.167ms; Space46.516/210.955ms;
second-word Tab44.769/35.459ms. Actual screenshots show plain typed prefix,
only the suffix ерка in a white rectangle, no composing prefix. This is a
bounded two-field visual/control result, not universal styling acceptance.

Remote control root:/home/e/projects/lay-development-runner/kde-install-control-20261020.
Receipts:chrome-941-notification-drain-{build,deployment,two-word-native,full}-20261023;
chrome-941-first-second-word-r10-candidate-20261023; each has result.json.
Local review packet:/home/ubu/.cache/lay/development/chrome-941-review-20261023/summary.json.
Manual ten-field window opened in existing ordinary Chrome PID632014/profile
/home/e/.cache/lay/kde-live-20261019/config/google-chrome,window0x2e0001b,
RU lay-ime-ru. No text was injected into that review window. Native proc env
is cleared by Chrome; absence of DISPLAY there is NOT proof of a headless
process. X11 window/PID and open-profile files establish it is the visible
ordinary Chrome. Do not create another user profile or close this window.
GNOME PID2202067/hashfa5e8950cc4c8c39e108464c9d06da0c00a32d26782f6dc2c030007f70e49a44
freshly verified unchanged after the final tests. No release/commit/push/tag.

The user explicitly requested this installed private trial for manual testing.
Chrome is NOT marked complete. User feedback is pending; continue Chrome only,
with remaining first transitions recorded above. Firefox1.0.94.2 and other
KDE applications/terminals remain untested at this stage and wait for explicit
Chrome acceptance. Do not turn a manual acceptance request into another broad
source rewrite or repeat the full matrix without new changes/evidence.

### Retained stage — KnownStart/read80 and resource diagnosis

Current KDE1.0.94.1:PID1576855/hash906f8dfe3e391bc5de5f6c4540d90cebdba676f01c284b99e8d66a8d45a9c296,
installed2026-10-23T20:06:15.693641UTC.37 native controls/architecture/build
pass; old15 failures/TD121 coverage error. Source localrun-7zbr3sep/remote run-Zpx3i0.
Control root/chrome-941-known-word-{build,deployment,smoke,full}-20261023.
Same unchanged r19 active pair nowPASS52.314/58.577ms, exact raw prefix retained:
проверка привет -> проверка ghbdtn -> проверка привет; decoder passes.
R18 fixed20 still13PASS/5FAIL/2BLOCKED:4 correction failures,1 ordinary roundtrip
context_authority(no wire native_deadline); successful toggle34samples median49.16,
p9564.40,max80.91ms. Cold r7 is3/4, ordinary second erka rectnull. Its inline
native_trace is a JSON list, not a path: filter it; never dump the whole list.
Measured correct native extents then source_read deadline40183/40113us;7 read
deadlines in that receipt. Root is incomplete actual-read evidence, not styling.
Full r18 completed146PASS/34FAIL/40BLOCKED/0NOT_TESTED of220 on906f8dfe;
retain its raw rows/traces and scope, no full acceptance claimed.
Next source is only OBSERVATION_BUDGET40->80ms, selection40/activation20 unchanged;
max read-preflight+selection120ms. No new tests mirror the constant; repeat the
37 existing semantic controls and the same native protocols on installed bytes.
No source/common/GNOME routing change; no release. User acceptance and Firefox
pending. Source check/build/install not yet run for read80.

CRITICAL measured resource failure before promoting read80: current PID1576855
has510 threads (502 zbus executors), TasksMax512/pids.events max2872,508fds.
Socket peer mapping:499 native accessibility connections to dbus-daemon3278315,
2 session to3278166,1 IBus to3278176. Memory high/oom0; only6/4462 CPU-throttled
periods. Receipt control root/native-bus-resource-exhaustion-20261023.json.
read_focused opens a new private bus per fresh context and retained bindings/
subscriptions retain old connections. Never raise TasksMax to conceal this.
Read80-only source check passed37 native controls, same15/TD121 (remote run-x9P9LN,
localrun-f9gt9137); it was NOT built/installed. Current source adds a transport-
only exact-address cache in kde/native_document/bus.rs, no mutex across I/O,
no document/owner cache, and disconnected-socket eviction by current executor
identity. Three tests in bus/tests.rs cover fresh data/reuse, concurrent winner,
cancel/failure and stale disconnect. Source verification not run yet; expected
native count40. Read80 is retained as a separate bounded-read adjustment, not
the resource fix. Next build must allow bus/tests.rs in the Rust delta.

### Retained stage — closing Space

Current KDE1.0.94.1:PID1515342/hash840f48e6f28d252fb6509b9cf40c591804311f2fd20f175ca7b644af7752cde2,
installed2026-10-23T19:51:42.117214UTC.36 native controls/architecture/build
pass, same15 old failures/TD121 coverage error. Source localrun-stfiskr3 /
remote run-twiaPY. Control root/chrome-941-closing-space-{build,deployment,smoke}-20261023.
Unchanged r18 fixed20:13PASS/5FAIL/2BLOCKED. All12 Double Shift scenarios
(two uninterrupted, roundtrip+decoder,5-burst x active/oneSpace x text/iframeRich)
PASS;36 successful toggle samples median47.41/p9562.72/max71.62ms. Remaining
failures:4 first-word autocorrection/boundary cases and1 ordinary hint wait
(inline frame still old lay-ime-us/24/textempty), not declared fixed.
Cold/same-field r6 finished OBSERVED; inspect4-row transition_pass before
claiming its current-byte acceptance. Receipt chrome-941-first-second-word-r6-candidate-20261023.
Additional frozen r19 two-word active diagnosticFAIL: KnownStart/exact_managed
true; DOM/native actual older prefix Space differs from VisibleTailV4 NBSP;
first manual RPC delegates3. No wrong mutation. Receipt chrome-941-two-word-native-diagnostic-20261023.
Next KDE-only source uses current KnownStart + exact active last-token boundaries
for that native manual range, leaving UnknownStart/closed-word and all common
owners/verifiers unchanged. Source tests not run/installed yet. Full220 and user
acceptance pending. No Firefox or public release.

### Retained stage — space presentation

Current KDE1.0.94.1:PID1431270/hash528f952694ca88720514d2e3130059638d57b73e4578a43f4edacd85e902aec1,
installed2026-10-23T19:32:50.611135UTC.34 native source controls/architecture/
build pass; old15 named failures and TD121 coverage error unchanged.
Source localrun-l9veci3l/remote run-VIeHbI. Installation control root/
chrome-941-space-presentation-deployment-20261023-r3/result.json. Initial
preflights changed no runtime: invisible hint matched the exact closed r4
fixture; r3 proved its native peer/profile gone and OwnsActiveTextfalse while
preserving unrelated ordinary Chrome and20 protected processes.
Cold/same-field r5 native hint+Space/Tab4/4 PASS, first and second word in
ordinary and rich fields. Hint51.576/79.399/53.438/76.363ms;
Space36.706/110.376ms, second-word Tab34.717/24.205ms. No underline in the
already committed prefix; full screenshots retained, final visual check pending.
Control root/chrome-941-first-second-word-r5-candidate-20261023/result.json.
Unchanged r18 fixed20:10PASS/8FAIL/2BLOCKED in smoke-r2. Initial smoke lost
its renderer/WebDriver session before seed/manual calls; do not count it as
product execution or erase it. Source/runtime were unchanged for the retry.
Next KDE-only source: recognize the owned literal closing Space through the
same tail owner; permit pending layout Space presentation without learning
feedback by checking authorized emitted layout_sync_text. Native source tests
now36/36 PASS (remote run-twiaPY/localrun-stfiskr3); same15 named baseline
failures/TD121 coverage issue. RED run-vKp9P1's positive fixture lacked actual
Chrome capability41; only the fixture was corrected. Release candidate build
currently running, session80793, control root/chrome-941-closing-space-build-20261023.
Not installed yet. No general normalization or whole-word guard change.
After install: same frozen r18 fixed20, then cold/same-field4 and one additional
two-word native diagnostic (r19, prefix supplied only by ordinary input).
Full220 not rerun; user acceptance and Firefox remain pending. No release.

### Retained stage — intermediate selection

Current KDE1.0.94.1:PID1355635/hash37333022a36ade457d877b1acb99e458ec0a37a3aa148508fb2cbe80cfa590cc,
installed2026-10-23T19:13:32.434459UTC. Existing transient-read path now keeps
an already pending exact native result across its intermediate selection,
clearing current document/edit proof.31 native source controls pass; same15
baseline failures and TD121 coverage/stack issue. Architecture/build pass.
Source localrun-tx5b1bv0/remote run-sWsLX8. No GNOME/source-route changes.
Control root/chrome-941-intermediate-selection-{build,deployment,smoke}-20261023.
Unchanged r18 fixed20 completed9PASS/9FAIL/2BLOCKED. All five ordinary-field
closed-word changes now confirm; cold first pair and rich closed/decoder fail.
Full220 was not repeated. Cold/same-field r4 diagnostic3/4: rich word2 stores
old NBSP in VisibleTailV4 while DOM and native GetText have ordinary Space;
native character bounds are correct. Receipt at control root/
chrome-941-first-second-word-r4-diagnostic-20261023/result.json. R3 failed its
proof-path preflight before input; r4 corrects only that path. Extra read-only
VisibleTailV4/InputState occur only after a failed hint wait.
Next KDE-only source experiment separates current-word readout from full-tail
manual authority and reuses the existing owned-Space owner for the exact final
slot of a pending native layout result. Common verifier/rules and GNOME remain
unchanged; no general whitespace normalization. New private module:
kde/native_document/client_presentation.rs. Source tests not run/installed yet.
Protected20 processes/config exact restore on installation; no daemon/tray/
IBus/desktop restart. No release/publication. User acceptance pending.

### Retained stage — selection acknowledgement

Current KDE1.0.94.1:PID1318553/hashbd2a751ed648a029e54fc7475293aff7eee5699e3feaeea9bf50d90debe0927e,
installed2026-10-23T19:06:32.308599UTC. Dedicated selection40ms experiment
has29 native source controls/architecture/build pass, same15 baseline failures
and TD121 coverage error. Source receipt localrun-7kxl7v2o/remote run-xnEKSH.
Unchanged r18 fixed20 now9PASS/9FAIL/2BLOCKED in52.084s; no native_deadline.
Both active five-bursts pass. Full220 not repeated; user acceptance pending.
Control root/chrome-941-selection-ack-{build,deployment,smoke}-20261023.
Config exact restoration, runtime hash and20 protected processes pass.

Next source change: preserve an already pending exact native result across
its intermediate selection, clearing all current document/edit proof. Two
tests add positive recovery and eleven stale/missing-owner controls. Not
installed yet; retain unchanged r18 actor for the next comparison.

### Retained stage — deferred result

Installed KDE1.0.94.1:PID1090108/SHA256
8c674d4df775404c1199140289b2785b95481637ffc0c4231f054f2ad385045e,
installed2026-10-23T18:00:29.741561UTC. Deferred native-result settlement is
installed after causal RED/GREEN;29 native controls pass, same15 old named
failures and TD121 coverage/stack failure remain. Architecture/build pass.
The UNCHANGED r15 actor PASSes both consecutive gestures without another key:
привет->ghbdtn65.634ms->привет60.579ms, no selection, caret6. One pair row PASS;
config/trace/runtime identity restoration passed. Receipt at remote control
root/chrome-941-deferred-native-pair-20261023/result.json. User acceptance and
full Chrome scope are NOT complete. No publication or GNOME installation.

Full Chrome r16 completed:81PASS/43FAIL/96BLOCKED,0NOT_TESTED,220 rows.
Output at remote control root/chrome-941-full-r16-20261023/chrome/result.json.
220-cell manifest:old160
plus60 uninterrupted pairs. ActorSHA
57a5fa2cb7ca077625c784f02bd103ab6092eaf40da54343cc946022a0546854.
R16 keeps cadence/targets, reuses v34's two-sided clock and actual next-gesture
start burst check, returns event payloads on demand, verifies native readiness
after clear and our fresh erka hint/rectangle before Tab. Retain the immutable
actor/wrapper. Ordinary text
active5-burst PASS; closed-word5-burst failed after the first change. Other
failure groups/setup blocks exist; exact-pair repair is not full acceptance.
Passive trace-at-progress-1.jsonl under that output records selection_present8,
document_changed2, no_admission23; some source seals still show epoch mismatch.
Group by first shared transition; keep C07/epoch guards.34 blocks are intentional
(10 inverse,24 URL/email Space);62 are native/DOM disagreement (45 initial
readiness,9 after-clear readiness,8 caret/text observation). Runtime PID1090108/
8c and config restoration pass. Full trace boundary is incomplete; per-browser
native/manual wire logs are complete. Elapsed415.831s.

Measured rich-field representation: DOM raw empty after clear; native GetText
newline. R18 diagnostic compares real rendered DOM innerText during preparation
only when raw comparison disagrees; within-gesture observations/targets/cadence
are unchanged. It records per-row traces and passive KDE/IBus mode signals.
Fixed20 diagnostic completed7PASS/11FAIL/2BLOCKED(inverse before input),
57.741s; all20 row trace boundaries complete; control root/
chrome-941-mode-diagnostic-20261023-r18. ActorSHA
17dc93b5f2769d5dde62bfe8f5a539fb68af40744139e823c5f18a848b4ce7e6.
Retain the immutable actor/wrapper; do not splice it into the old matrix.
Its text/rapid-five/space row proves native_deadline on gesture2 with actual
selection[0,7], then three context-authority refusals. KDE-only dedicated
selection acknowledgement40ms is a new experiment, not installed yet;
activation/subscription20ms and source observation40ms remain unchanged.
Other active-burst/source-state and rich presentation failures remain separate.
Same8c isolated closed-word5-burst with unchanged r16 actor later PASSed:
chrome-941-five-closed-diagnostic-20261023/result.json. Retain original failures;
investigate state/order dependence. R17 rich pair also PASSed, but additional
within-gesture innerText/innerHTML reads are diagnostic only. Actual fresh DOM
stayed a plain text node before/after that pair; no functional fix is inferred.

Cold/same-field4-row actor on8c still3/4: both first-word Space and normal
second-word Tab pass (44.645ms); rich second-word erka has rectnull, Tab not
attempted. Prior NBSP becomes ASCII Space while typing. Receipt:
chrome-941-first-second-word-candidate-20261023/result.json at control root.
This remaining display/append issue is not fixed. Actual native geometry was
measured earlier; inspect the first failed presentation eligibility predicate.

### Previous stage — retained registration-correction evidence

Earlier installed KDE trial1.0.94.1 was PID992714, SHA256
bbc4fcfe9fc68b10713eaac03f31f935ad4b43f3a68d8ceda40dee0481958eee,
installed2026-10-23T17:32:50.159225UTC. Source phase budgets and the shared
two-parameter text/selection Registry registration are installed.27 native
controls pass; full source still has the same15 named failures plus TD121
coverage/stack failure. Architecture and guarded release build passed.

The unchanged r15 actor now reproduces the exact user report: first
привет->ghbdtn PASS46.703ms; second ghbdtn->привет FAIL/context_authority.
The whole two-transition row remains0PASS/1FAIL. Passive wire proves the
selection registration and TextSelectionChanged; no external listener was
added. Trace: actual result confirmed at epoch11, deferred layout publication
advances engine epoch12; FocusOut69 cannot seal (tail_epoch_matches=false),
new lay-ime-us activates source_free_ready and loses the word.

Causal isolated RED is confirmed: native_document_deferred_result_keeps_the_next_layout_handoff_epoch
fails at the existing admission publication API after the actual native result,
because observation ran outside ContextBridgeOutput. A structural extraction
preserved old behavior for RED. Current correction scopes that same canonical
result verifier in the existing live output guard only for a native pending
result without an enclosing guard. New tests retain UnknownStart, cover an
existing guard and reject changed epoch/focus/lease/layout/field/privacy/text/
selection/backend. This correction is NOT built/installed/accepted yet.
GNOME/common reducer/verifier/output routes unchanged.

Latest receipts at remote control root:
chrome-941-selection-registration-build-20261023/result.json,
chrome-941-selection-registration-deployment-20261023/result.json,
chrome-941-registration-native-pair-20261023/result.json,
chrome-941-deferred-handoff-red-20261023/result.json.
Decision2026-10-23-kde-native-deferred-result-settlement.json owns the next step.

Before that trial, same-bytes recovery moved
the IME from an obsolete TasksMax32 test unit to a bounded TasksMax512 unit:
PID798365, SHA256bdd16973edcadee7943e0ce7e894eaea062d955b5ca23e69a71980fddb5ef152.
The earlier PID558724 aborted in LayoutSwitchWorker::start with WouldBlock;
the task limit is a supported resource hypothesis, not proof of a source leak.
Receipt: remote control root/chrome-same-bytes-runtime-recovery-20261023/result.json.

The r12 actor completed220 rows:0PASS/53FAIL/167BLOCKED. Normal/private field
preparation lacked Chromium's real GetAttributes activation request;132 rows
could not confirm native focus,34 were preblocked protocol cases,1 disagreed
on native/DOM text. All53FAILs reached confirmed native fullscreen fields.
Passive wire evidence records121 ManualToggleV3 calls:46 native_deadline,
67 byte0/false and8 byte3/false;38 AddSelection replies were true. The exact
first-success/second-failure user sequence is NOT yet reproduced: these rows
stop at the first gesture. Same PID/hash and exact config restored afterward.
Receipt: remote control root/chrome-all-fields-20261023-r12/result.json;
local wire analysis: desktop-isolation-20261023/chrome-r12-evidence/wire-analysis.json.

R13 reuses production GetAttributes activation: normal-field setup now passes.
Its single active-word pair still FAILs at the first gesture, byte0/false:
context_authority, current native reads but UnknownStart/partial observed suffix.
Source controls currently change layout AFTER preparation/clear; inspect that
ordering before treating it as the user's no-intermediate-input sequence.
Receipt: remote control root/chrome-selection-deadline-20261023-r13/result.json.

R14 moved source controls before target preparation; context_authority refusal
remained after clearing an already empty field. R15 also avoids unnecessary
Ctrl+A/Backspace when the field is already empty: on old bdd bytes it reaches
native_deadline, on phase-only b7 bytes it reaches an accepted selection but
still times out. R15 actorSHA623d8f981ce14d299d65ba34c455ba923bf1e8080f88e70d5ccd2fdc010cebf3
is unchanged between those two trials. The new wire records RegisterEvent
serial882 with three parameters, empty reply, NO EventListenerRegistered;
AddSelection883=true; later actual native selection[0,6]. This is the same
legacy Registry issue already solved in the adjacent text-change registration.
Do not add an external listener to make acceptance pass.

Source run-w6K0AD:26 native-document controls ok; same15 distinct named failures
as run-RxUsup and the same TD121 coverage/stack failure. Full source verdict
is not PASS. Architecture refresh/build passed; no new failure identities.
Release cache was atomically moved out of the disposable source mirror to
/home/e/projects/lay-development-runner/candidate-target-cache-20261023;
future builds must reuse it rather than have workspace rsync delete it.
Receipt chrome-941-cache-preservation.json; no runtime/cache copy was made.

Firefox opened earlier but its Snap log also reports an input-event queue
overflow/drop and IBus Permission denied. Launch success is not input success;
this remains pending behind Chrome and must preserve the existing profile.

## Completion sound hook — 2026-10-23

The user explicitly requested an actual completion hook. User-level Codex
`notify` now invokes `/home/ubu/.local/bin/codex-turn-done-sound` for
`agent-turn-complete`. The handler accepts only Lay checkout paths and plays
the existing task-done.wav through installed pw-play, with a five-second bound.
It ignores other events/projects and stores no message text. Native config
loading and one actual handler/audio invocation passed. Existing lifecycle
hooks and transport are unchanged. Config backup is under
`~/.local/state/codex-turn-done-sound/backups/20261023T141147Z/`.

Notify is loaded at Codex startup. Do not restart this live session merely to
load it; keep the manual finish sound until its next normal launch. The prior
task-done shell helper uses missing paplay; the new handler does not depend on it.
The user then requested half volume: handler now uses0.305 rather than0.61;
actual playback at the new level succeeded. Native notify still needs the next
normal Codex startup; keep manual completion playback in this live session.

## Current plan progress — 2026-10-23, desktop split / KDE tray / Chrome

User approved the minimal three-stage plan: finish explicit desktop isolation,
reproduce Chrome first/second word on the unchanged KDE build, fix only the first
observed failure, then install/identify/test KDE. No Harness2; GNOME accepted
engine2202067/fa5e8950 remains preserved. Notify sound hook above is a separate
completed user request; it does not replace the active engineering task.

Niri/X11 now use a separate baseline white style in display.rs. New regression
other_backends_keep_pre_refactor_white_suffix_style measured RED run-AcEve4,
GREEN run-Z4hI8b. Same16 pre-existing failures plus TD121 stack overflow remain;
five display tests ok. No engine was installed at that check; the later KDE
development installation is recorded below. Source receipts local
run-qqrspk0b/run-8jwls34_.

User also reported disabled-looking/unusable KDE enable checkbox. Real daemon
was active; tray systemctl used KDE's private bus instead of manager bus.
Fixed scripts/lay-runtime-control.sh via command-local user_systemctl and new
daemon-is-active action; tray now queries the same controller. Enhanced existing
isolated tests assert manager bus versus preserved desktop bus:20/20GREEN.
Installed only these two scripts on KDE, restarted only tray unit
lay-kde-tray-candidate-20261021.service, now PID336128. Daemon3282573 and
engine3278357/b8249005 unchanged, live config6e62eab3 unchanged. Actual popup
visually confirms checked "Lay enabled", IME mode, assistance/auto-replace.
Receipts at remote control root/tray-service-bus-20261023/{red.log,green.log,
installed.json,activated-menu.png}; local image desktop-isolation-20261023/
tray-activated-menu.png. Initial DBusMenu/right-click capture attempts did not
show the real manually opened popup; no client PASS was taken from those.

Corrected/reused private Chrome script chrome-only-first-second-word-20261023.py
types first word+one Space, then second word+Tab in SAME text and rich fields.
Remote receipt control root/chrome-first-second-word-20261023-baseline/result.json:
text word1 SpacePASS; text word2 hint visible but Tab unhandled, value stays
"проверка пров". Rich word1 SpacePASS; word2 hint rectnull, Tab not attempted;
rich also normalizes first separator NBSP to Space while typing. Trace complete,
all config/engine/daemon restoration true. Native hint successes n3:51.361,
82.250,104.092ms; second rich hint not ready by1.2s. Text Tab timed out1215ms.
This partial diagnostic does not replace the historical100PASS/76FAIL/
0BLOCKED/144NOT_TESTED of320. Only normal text/rich tested here.

The duplicate-read regression reached the real two-word frame and measured
RED revision4 instead of3 in run-KeCRJ8, then GREEN in run-RxUsup after the
bounded kde/native_document.rs fix. All26 native_document tests report ok;
the same16 ordinary failures and TD121 stack overflow remain, with no new
failure identities. Common Tab/pending/verifier and delivery routes unchanged.
run-k6w8deph stopped before tests at fmt; run-l41bc7ck refused a changed source
snapshot, and the first fixture run-w1PB5F failed its missing CommitText-receipt
precondition. None of those is successful product evidence.

Remote architecture refresh and release build passed under the existing guard.
Build receipt control root/chrome-duplicate-read-20261023-build/result.json
identifies actual Cargo executable and retained candidate, SHA256bdd16973edcadee7943e0ce7e894eaea062d955b5ca23e69a71980fddb5ef152.
All759 local Rust files match the remote graph binding; refreshed graph and
architecture receipt retrieved, previous generated files retained privately.
At2026-10-23T15:23:42.283116UTC the development engine-only install completed:
PID558724, actual/proc and installed hashes match candidate, display1.0.94.2.
Daemon3282573, tray336128, config6e62eab3 and9 protected processes preserved.
Accepted local GNOME2202067/fa5e8950 unchanged. No publication/release acceptance.

Identical r2 actor on old and new bytes handles only Chrome's NBSP/ASCII-space
normalization of the single separator; readiness/field/fresh frame/caret and
composition checks retained. Baseline text word2 Tab FAIL; candidate text word2
Tab PASS with exact value "проверка проверка " in105.716ms. Candidate4 rows:
text word1 SpacePASS, word2 TabPASS; rich word1 SpacePASS, word2 hint geometry
FAIL and Tab not attempted. Thus3PASS/1FAIL, not full Chrome acceptance.
Receipt control root/chrome-first-second-word-20261023-r2-candidate/result.json.
Successful hint samples n3:82.451/286.832/51.444ms; one rich hint timeout1.2s.
Timing is key-to-DOM/frame observation including polling, not a speedup proof.
All config/engine/daemon restoration and trace boundary checks passed.

The remaining rich-field transition is bounded: passive existing a11y log
records GetCharacterExtents(offset12)=(157,506,11,21), while published rect=null.
Chrome supplies actual geometry; do not repeat attribute changes or fabricate
a fallback position. Its GetText changes prior separator U+00A0 to U+0020
while typing the second word. Full owned-tail agreement is a hypothesis for
the eligibility refusal, not yet measured inside the live engine. No new
authority/Tab guard relaxation was made. Next needs a causal source fixture
and the first failed presentation predicate before another runtime change.

Additional user request: Firefox would not start on KDE. Existing Snap/profile
launch on private desktop bus fails "is not a snap cgroup"; same launch on
manager bus opens actual Firefox window39845909, PID550967, existing profile,
no cleanup. Unit lay-support-firefox-kde-20261023-manager is resource bounded.
Receipt /home/e/.cache/lay/development/firefox-kde-start-20261023/result-manager.json;
local screenshot desktop-isolation-20261023/firefox-opened.png. Snap logs IBus
Permission denied: browser launch PASS does not certify Firefox IME. No default
launcher or global session-bus setting changed. Firefox remains open.

Owning evidence: desktop-presentation-isolation-2026-10-23.md. Remote control
root is /home/e/projects/lay-development-runner/kde-install-control-20261020.

### 2026-10-22T04:07:42.919260+00:00 — v61 source464/464PASS; final Harness2 task20261022T040352Z-3055d027 running

12focused/464fixed source PASS; architecture and diff PASS. Post-request timeout RED and shared cancellation correction are recorded at the top of docs/architecture/ime-inline-hint-preflight-2026-10-20.md with exact receipts. Current remote runtime remains v60b; v61 engine-only build/install/native pending. New24-function Harness2 review explicitly allows read-only file inspection; prior blocked selected review rejected. Native6-row Writer acceptance is next after independent review/source identity. Chrome underline FAIL remains open, Kitty accepted. PRs5open #64–68 freshly checked04:04:25UTC; no mutations. Full browser matrix unchanged.

### 2026-10-22T03:41:23.274890+00:00 — v61 source463/463PASS; final scoped Harness2 review running

Remote fixed proof `stock-replay-v61-reset-affected/result.json`:463PASS/0FAIL, execution97.597s. Complete gate `stock-replay-v61-reset-source-gate/result.json`:PASS, architecture receipt PASS, diff check PASS, wall163.822s, ended2026-10-22T03:39:48.060203UTC. The same fixed denominator includes all4 unchanged regressions exposed by the two rejected variants.11focused controls PASS. Restoring the existing computation branch fixes ordinary Reset without allowing unproved completed replay; no request/edit route or authentication owner added.

Harness2 projectlay2 task `20261022T034019Z-2e25183e`, gpt-6.1-sol/xhigh, per-run480s; frozen13complete functions incl2full tests,36421inputbytes, `stock-replay-v61-refresh-final-review-r2/`. RUNNING, not accepted yet. Report expected at `/home/ubu/.local/state/harness2-codex/projects/lay2/tasks/20261022T034019Z-2e25183e/report.json`. Install only after independent report/input/current-source verification. Then remote engine-only build/install and6 short native Writer rows. Runtime still v60b, native v61 NOT_TESTED; Chrome prefix underline FAIL remains open, accepted Kitty preserved. Release/updater/GNOME/full browser matrix not revalidated.

Regular read-only upstream check2026-10-22T03:38:58.441348UTC:5open PRs, #68zbus5/5CI, #67rayon4/5, #66sha2 3/5, #65serde_json5/5, #64arboard3/5. Receipt `/home/ubu/.cache/lay/development/browser-final-fixes-20261018/shift-hint-20261021T1242Z/upstream-pr-review-20261022T033858Z.json`. No merge/close/message/dependency change.

### 2026-10-22T03:36:13.639936+00:00 — v61 bounded replay guard: preserve the pre-existing computation branch

Measured source result: `stock-replay-v61-bounded-affected/result.json` is461PASS/2FAIL of463 (96.09s execution). The previous two replay-refusal controls now pass, but `firefox_bridge_publication_survives_shift_received_after_reset_suffix_binding` and `td121_current_metadata_unproved_release_only_retires_the_handled_press` fail at the first manual RPC (NotHandled instead of the prior pending read). Both unchanged fixtures have ordinary context-reset computation authority and no completed replay. The new missing-SSI condition unintentionally applied the stock completed-replay restriction to both branches of `context_reset_rereceipt_manual_wait_allowed`.

Chosen minimal correction: explicitly preserve `context_reset_rereceipt_computation_allowed()` inside that condition. Its existing exact-replay quarantine excludes completed replay; completed replay still needs exact-refresh capability or an already queued authenticated read. Alternative reverting the whole condition is rejected: it reintroduces the proven premature replay handoff. A new branch-specific queue/owner is unnecessary. No test assertions, receipt validation, token/lease/epoch/deadline or delivery route changes. Expected effect is only restoring the two established reset paths, not granting edits from missing SSI. No allocations, timers, cache/package, candidate/ranking or learning changes; one existing pure predicate is reused. Verify11 focused controls (all9 prior plus these2), then the same463 fixed proof. Failure rejects this correction. Runtime remains v60b PID1994712, SHA82d751b1ea32e0736a115392bb13948bd46aa364bd76b94868caa96494f2c245; source PASS is not physical-client PASS. Native/install/release/updater remain NOT_TESTED for v61.

Receipt root: `/home/ubu/.cache/lay/development/browser-final-fixes-20261018/shift-hint-20261021T1242Z`; runtime authority unchanged. Rollback boundary is only the added ordinary-computation alternative, preserving source-prefix A and all established refusals.

### 2026-10-22T03:28:51.078049+00:00 — v61 unrestricted refresh rejected; bounded refresh9/9GREEN

Fixed affected gate463:461PASS/2FAIL, receipt `/home/ubu/.cache/lay/development/browser-final-fixes-20261018/shift-hint-20261021T1242Z/stock-replay-v61-affected/result.json`. Both original assertions remain unchanged: `td121_inherited_full_prefix_refuses_contradictory_final_receipt` and `td121_native_replay_progress_never_authorizes_pre_final_manual_effects`. The unrestricted capability extension permitted a partial SSI request and an immediate manual handoff from missing SSI; never installed, archived `/home/ubu/.cache/lay/development/browser-final-fixes-20261018/shift-hint-20261021T1242Z/rejected-unrestricted-stock-refresh-v61/`.

Causal ablation restores only exact-refresh eligibility while retaining real-source-prefix fix:8/8PASS (`/home/e/projects/lay-development-runner/kde-install-control-20261020/stock-replay-v61-ablation/result.json`). Source-prefix fix preserved. Bounded replacement requires a prior whole source surface or whole replacement word without its boundary for ordinary-client reads. Missing/partial SSI cannot start a manual gesture; existing queued wait retains its700ms deadline. Exact-refresh sentinel behavior unchanged. Focused9/9PASS, including both previously failed unchanged controls; compilation12.89s/execution2.19s. Receipt `/home/ubu/.cache/lay/development/browser-final-fixes-20261018/shift-hint-20261021T1242Z/stock-replay-v61-refresh-bounded/result.json`.

Next: rerun the same463proof after architecture refresh, then final bounded Harness2 review; no candidate installation before these gates. Live remains v60b. Upstream read-only rechecked2026-10-22T03:20:54.931121UTC:5open PRs68/67/66/65/64; CI5/5,4/5,3/5,5/5,3/5. Receipt `/home/ubu/.cache/lay/development/browser-final-fixes-20261018/shift-hint-20261021T1242Z/upstream-pr-review-20261022T032054Z.json`; no merges/upgrades/messages.

### 2026-10-22T03:19:27.878934+00:00 — v61 focused GREEN7/7

Existing pending-manual refresh extension now passes all7selected checks after causal RED6/7. Tests cover original delayed and final-letter-without-Space actual receipts, empty/Unicode external prefix, ordinary/exact-refresh capabilities, Space/NBSP and incomplete/contradictory/expired/atomic/sealed/no-surrounding refusals. Source compilation12.94s, execution1.73s. Remote receipt `/home/e/projects/lay-development-runner/kde-install-control-20261020/stock-replay-v61-refresh-green/result.json`. No runtime changed. Next fixed affected scope463checks plus architecture refresh, final bounded review, engine-only build/native qualification.

### 2026-10-22T03:18:04.121006+00:00 — current v61 handoff

Live v60b unchanged. Preserve5/5GREEN real-source-prefix change; rejected release hook archived and removed. Existing700ms manual refresh stock extension has causal6PASS/1FAIL; remote GREEN pending. Harness2 task20261022T024721Z-67fc0e30 completedok/8calls/0failed/review8of10; independently checked, not client acceptance. Owning details: `docs/architecture/ime-inline-hint-preflight-2026-10-20.md` top entry. Open upstream PRs68/67/66/65/64 checked2026-10-22T02:54:36.527396UTC; regular read-only check rule recordedAGENTS.md. Next: full affected tests, review, short engine-only native qualification. Chrome underline and LibreOffice closed reverse remainFAIL; Kitty preserved.

## 2026-10-22T02:39:26.091289+00:00 — causal RED: observed source prefix is lost on engine handoff

Remote `stock-replay-source-v61-red/result.json`: four existing tests PASS; new `stock_observed_boundary_handoff_preserves_real_prefix_without_target_receipt` FAILS at actual replay source prefix `None` vs `Some("")`. Fixture uses real SSI, successful local Space with no invented final SSI, authenticated manual RPC, actual factory/activation handoff and suppression RPC. Target still has no SSI/ACK; the failure is retained source provenance, not target acknowledgement. Unicode-prefix case remains behind the first RED assertion and is not yet claimed tested.

Harness2 USED task `20261022T023435Z-586d6f4c`, lay2/gpt-6.1-sol/xhigh, bounded frozen plan review RUNNING; no verdict yet. Inputs matched remote origins before launch. Proposed correction carries explicit actual-snapshot versus observed-word/local-boundary provenance through the existing reducer receipt; a separate real result-read request may be needed after final native key release. No new owner, fake SSI/ACK, relaxed verifier, client-name bypass or delivery route. Live v60b unchanged, LibreOffice reverse/Chrome styling still FAIL. Next: source fix and strict negative checks, independent Harness2 review, then short remote native proof before acceptance.

## 2026-10-22T02:22:58.218994+00:00 — v60b installed; LibreOffice reverse closed toggle remains FAIL

Remote engine-only install at2026-10-22T02:10:03.342822UTC: PID1994712/SHA25682d751b1ea32e0736a115392bb13948bd46aa364bd76b94868caa96494f2c245. Candidate/source closure matches the462/462 remote source/architecture/build proof (1258files/7db8e45bc327d05d3bbb814ab079803d3e500c5f8042b4bbe4b7f668172e5cd2). Deployment `/home/e/projects/lay-development-runner/kde-install-control-20261020/stock-local-boundary-v60b-deployment/result.json` preserves8protected processes/config/tray; no daemon/global IBus/desktop restart. No local installation or publication.

Harness2 USED task20261022T014733Z-2b7280e4, lay2/gpt-6.1-sol/xhigh: reportdegraded, coreok,8calls/1timeout/c1fallback; bounded8/10/no demonstrated source blocker. Thirteen frozen hashes/eleven full-origin excerpts/seven remote full source files independently match. Acceptance `/home/ubu/.cache/lay/development/browser-final-fixes-20261018/shift-hint-20261021T1242Z/stock-local-boundary-v60b-review-acceptance.json` is SOURCE_ONLY, not client proof. Full report `/home/ubu/.local/state/harness2-codex/projects/lay2/tasks/20261022T014733Z-2b7280e4/report.json`.

Native LibreOffice v60b default profile5PASS/1FAIL: first/second hint+Tab, active Double Shift roundtrip, physical Shift+A, Space auto-flip PASS. Closed target becomes `Ghbdtn `; independent ordinary input with actualxkb:us::eng and no Lay also yields `Ghbdtn `, proving Writer sentence capitalization. Controlled NEW owned profile with only sentence capitalization disabled retains the six strict assertions:5PASS/1FAIL; first closed toggle gives exact`ghbdtn ` in180.165ms, reverse gives no change. Active toggles198.12/181.522ms. No user profile/config change. All7restorations true. Receipts `/home/ubu/.cache/lay/development/browser-final-fixes-20261018/shift-hint-20261021T1242Z/libreoffice-v60b/`, `libreoffice-default-capitalization-v60c/`, `libreoffice-no-capitalization-v60c/`. Actual screenshots show ordinary prefix without IME underline; application spellcheck/header marks are distinct.

Short native regression `/home/ubu/.cache/lay/development/browser-final-fixes-20261018/shift-hint-20261021T1242Z/stock-local-boundary-v60b-native-controls/summary.json`: ordinary-launch Kitty hint/Tab/closed roundtrip/next decoder PASS; closed samples15.324/16.833ms. Chrome textTab1/1PASS, visual prefix underline remains separate FAIL;6restorations true. No physical-evdev/full browser/GNOME acceptance claimed. Historical browser100PASS/76FAIL/0BLOCKED/144NOT_TESTED of320 unchanged.

Next: causal source fixture for stock-client native replay followed by second closed gesture. Actual replacement letters have real SSI, final forwarded Space has none; readout becomes unknown-context (epochs287/288). Preserve replay receipt/quarantine guards; never promote forwarded key delivery to client ACK. v60b remains live. Chrome attribute-only proposal already rejected; no repeated attribute tweaks.

Fresh read-only PR check2026-10-22T02:21:23.575226UTC:5open; #68/#65CI5/5,#67CI4/5,#66/#64CI3/5. Receipt `/home/ubu/.cache/lay/development/browser-final-fixes-20261018/shift-hint-20261021T1242Z/upstream-pr-review-20261022T022123Z.json`. AGENTS.md330–339 mandates regular checks; no merges/messages.

## 2026-10-22T01:52:21.977835+00:00 — v60b related source proof and frozen review

v60b remote graph/build gate PASS: 462/462 selected checks, architecture PASS; engine candidate SHA-256 `82d751b1ea32e0736a115392bb13948bd46aa364bd76b94868caa96494f2c245`. Source closure1258files/7db8e45bc327d05d3bbb814ab079803d3e500c5f8042b4bbe4b7f668172e5cd2. Receipts: `/home/e/projects/lay-development-runner/kde-install-control-20261020/stock-local-boundary-v60b-postgraph/result.json` and `stock-local-boundary-v60b-build/result.json`. Existing stale-observed-client refusal retained unchanged. Installed engine remains v59/b1ae6be2; candidate native behavior NOT_TESTED.

Harness2 USED, projectlay2/modelgpt-6.1-sol/xhigh, task `20261022T014733Z-2b7280e4`, still running. Thirteen frozen inputs, eleven excerpts independently verified, seven full origin files match both workspaces. Report pending: `/home/ubu/.local/state/harness2-codex/projects/lay2/tasks/20261022T014733Z-2b7280e4/report.json`. Next only after review: engine-only install and six-row LibreOffice probe, ordinary-launch Kitty and Chrome controls. No daemon/IBus/tray/desktop restart, local installation or publication.

Fresh read-only PR check2026-10-22T01:42:22.155387UTC:5open; #68/#65 CI5/5, #67 CI4/5, #66/#64 CI3/5. Receipt `/home/ubu/.cache/lay/development/browser-final-fixes-20261018/shift-hint-20261021T1242Z/upstream-pr-review-20261022T014222Z.json`. No merges/messages. Chrome typed-prefix underline remains separate FAIL; historical full browser100PASS/76FAIL/0BLOCKED/144NOT_TESTED of320 unchanged.

## 2026-10-22T01:41:22.048889+00:00 — v60b source lifecycle clarified

Focused r2 proves first pre-Space word has UnknownStart despite an exact real client snapshot. Capture is inert and now requires a live matching token; post-Space manual readout still requires KnownStart. Focused r4 proves the record exists after Space, but the existing authenticated manual handoff republishes identical text at the next tail epoch. v60b carries only its already-current source across that one successful handoff; unrelated epoch changes still refuse it. No exact snapshot or delivery ACK is published. Failed compile in r3 was a missing test-only import, corrected explicitly; all receipts retained. Broader source, review and native verdicts pending.

## 2026-10-22T01:35:23.453310+00:00 — v60a rejected by the existing observed-client guard; v60b narrowed candidate

v60a related gate **460 PASS / 1 FAIL** (`stock_owned_manual_source_rejects_stale_unknown_and_observed_contexts`, observed-client case). A historical callback plus any local commit cannot prove the current word. Existing refusal/assertions are retained unchanged. Receipt: `/home/e/projects/lay-development-runner/kde-install-control-20261020/stock-local-boundary-v60-postgraph/result.json`; build stopped before release compilation. Running engine remains v59/b1ae6be2.

v60b hypothesis: preserve exact real pre-Space word observation only across one successful local ASCII Space. Capture in existing managed CommitText path; bind exact tail, existing tail epoch, focus serial, owner lease and actual observation revision. Clear at new input, observation, revocation or field close; revalidate existing KnownStart/current token and all manual handoff/refusal gates. This inert source record creates no owner, generation, timer, queue, executor, synthetic snapshot or delivery ACK. Epoch/owner/revision/text mismatch and native replay quarantine refuse it. Causal test and existing observed-client refusal must both pass; broader proof, Harness2 review and native client acceptance pending.

## Current continuation — 2026-10-22T01:20:17.868395+00:00: v60 candidate, source gate pending

Installed remote engine remains v59 PID1761235 SHA256 b1ae6be27b67b539e349819a76f51b72464eddfa5ee1cb22bb34bafeb63ef2b0. LibreOffice native5/6 and Kitty controls remain the latest installed evidence; Chrome prefix-underlining remains FAIL.

Causal v60 baseline RED `stock-local-boundary-v60-red-r4/result.json` reproduces observed-letter callbacks followed by locally committed Space with no later SSI: VisibleTailV4 rejects a known current word as unknown-context. r0-r3 had invalid fixture setup/assertions and are not product evidence. Candidate reuses `context_stock_owned_manual_source_is_current` for a known locally committed boundary without fabricating SSI or native replay ACK; excludes ExactReplay/quarantine and retains snapshot/selection/owner/sensitive/exact-refresh guards. Decision `docs/architecture/decisions/2026-10-22-stock-local-boundary-manual-source.json`. Initial green-driver compilation failed in the new negative test; qualified the content-purpose constant through the existing content-type setter. Source, Harness2 review and changed-byte native checks are pending. No v60 deployment or route/executor changes.

Fresh PR triage2026-10-22T01:19:31UTC: #68/#65 CI5/5; #67 CI4/5; #66/#64 CI3/5. Receipt `upstream-pr-review-20261022T011931Z.json`; no merges/messages. Regular checks are in AGENTS.md330-339. Historical full browser100PASS/76FAIL/0BLOCKED/144NOT_TESTED of320 remains unchanged.

## Current continuation — 2026-10-22T01:03:41.191392+00:00: v59 installed; LibreOffice5/6 native PASS

Remote KDE engine1761235 SHA256 b1ae6be27b67b539e349819a76f51b72464eddfa5ee1cb22bb34bafeb63ef2b0 was installed at2026-10-22T00:57:27.641506UTC. Source/build receipt `stock-deferred-v59-build/result.json` pins1258 files SHA10bfe5bfcfd3a62956ac4929d2c58a3910f6da1237b8a10e31136d50f332a2c2; postgraph459/459 source PASS. Engine-only deployment preserves8 protected processes/config/tray, no daemon/global IBus/desktop restart. Evidence root `/home/e/projects/lay-development-runner/kde-install-control-20261020/`.

Harness2 USED task20261022T004203Z-1eaf86cb, lay2/gpt-6.1-sol/xhigh, statusok,8 calls/0 failed/0 fallback, bounded8/10/no supported blocker. Eight frozen hashes and six full source origins independently matched before installation. Independent worker generation/frame checks inspected; full concurrent worker/package-cache proof not claimed. Local acceptance `shift-hint-20261021T1242Z/stock-deferred-v59-review/independent-acceptance.json`.

`libreoffice-presentation-probe-v59/result.json`: first hint/Tab, second-word hint/Tab, active Double Shift roundtrip, ordinary physical Shift+A and Space auto-flip PASS; closed Double Shift FAIL. All7 restorations true. Active-toggle samples187.526/176.048ms, no p95 claim. This closes only the second-word display failure on these exact native bytes. `stock-deferred-v59-native-controls/summary.json`: ordinary launcher Kitty hint/Tab/closed Double Shift PASS(2.826s), Chrome text-field Tab1/1 PASS(5.740s),6 restorations true.

Chrome presentation remains FAIL: independently viewed `stock-deferred-v59-native-controls/chrome-tab/hint-text.png` shows typed prefix underlined. Passive `chrome-ibus-wire.log` has208 Engine ProcessKeyEvent and0 SetSurroundingText; observed-literal transfer cannot run. Installed Chrome155.0.8059.84 source `ui/base/ime/linux/composition_text_util_pango.cc` creates a thin composition span even for PANGO_UNDERLINE_NONE, and defaults to a full underline when no spans exist. Actual client backend and possible ownership transfer still require proof; no attribute-only or client-name workaround applied.

LibreOffice closed failure has a concrete bridge trace: ManualToggleV3 serial480005 returns(3,false); VisibleTailV4 serial480006 returns passive:unknown-context/empty source at epoch82/current engine8. Real SSI reaches the whole word, then CommitText emits only Space; no post-Space SSI arrives. First failed gate is source readout, not loss of physical Shift. Local raw evidence `shift-hint-20261021T1242Z/libreoffice-v59/`. A new v60 callback regression is diagnostic-only and is not covered by v59 review/runtime; no production change chosen yet.

Latest read-only PR check2026-10-22T01:01:35.327785UTC: five open, #68zbus and #65serde_json5/5CI; #67rayon4/5; #66sha2/#64arboard3/5. No merge/messages. Receipt upstream-pr-review-20261022T010135Z.json. Historical browser100PASS/76FAIL/0BLOCKED/144NOT_TESTED of320 unchanged; no full matrix/GNOME/release/push/local install.

## Current continuation — 2026-10-22T00:44:38.922459+00:00: v59 source459/459 PASS; installed engine still v56

Real held-key+actual second-word SSI causes Client pair frame change. Baseline
causal RED: stock-deferred-v59-red-r2/result.json; chosen existing pending-display
refresh, no new owner/authority/route. Focused GREEN2/2 including matching-release
publication and inexact-source no-rebinding. Full related gate
stock-deferred-v59-source/result.json:459/459 PASS, discovery0.279s/execution96.99s.
Native v58 confirms LibreOffice second-word hint and closed Double Shift FAIL;
those remain open until new exact bytes are tested. v56 installed PID1592735/SHA
6d51bef57c15490dc300cbfa8f374d0fe20b02cf7a8ee31d37c3e65bcd4f3b08 unchanged.
Harness2 task20261022T004203Z-1eaf86cb is RUNNING, lay2/gpt-6.1-sol/xhigh, frozen8
inputs; no review acceptance yet. Next guarded graph refresh/pinned source proof
and engine build, independent review, engine-only install, native LO/Kitty/Chrome.
No global IBus/daemon restart, local installation, publication or matrix rerun.
See owning preflight for consequences, invalid fixture attempts and baseline
caret/selection display controls. Latest PR check00:23:43UTC, five open; rule
already saved in AGENTS.md. Historical browser100/76/0/144 of320 unchanged.

## Previous continuation — 2026-10-22T00:16:59.030581+00:00: v56 installed; qualified native LibreOffice4/6, Kitty/Chrome controls PASS

Installed remote engine PID1592735 SHA256 `6d51bef57c15490dc300cbfa8f374d0fe20b02cf7a8ee31d37c3e65bcd4f3b08` at2026-10-22T00:06:21.817892UTC, engine-only. Remote `stock-observed-v56-postgraph/result.json` pins457/457 source PASS after the generated graph receipt changed; initial failed metadata reconstruction is retained, not excluded. `stock-observed-v56-build-r2/result.json` identifies the compiler artifact (81.48s); `stock-observed-v56-deployment/result.json` verifies eight protected processes, config and tray unchanged. No daemon/global IBus/desktop restart, publication or local installation.

Harness2 USED: projectlay2, gpt-6.1-sol/xhigh, task20261021T235242Z-f11aa21c, complete statusok,8 calls/0 failed/0 fallback, selectedc1 bounded8/10. All10 frozen input hashes verified unchanged; existing plain-commit, ownership, cancellation and freshness implementations independently inspected. Local receipt `shift-hint-20261021T1242Z/stock-observed-v56-review/independent-acceptance.json`. Source review is not client acceptance.

Initial v56 native outputs (six blank LO rows, failed controls, zero Engine.ProcessKeyEvent) are setup-blocked, not six proved product defects. A screenshot contained the Lay popup, but its later X11 state was IsUnMapped; a keyboard grab was not established. The qualified `libreoffice-presentation-probe-v57/result.json` adds a native click inside its owned editable paragraph and requires ordinary `a` before running cases. On the same v56 bytes:4 PASS (first hint/Tab, active Double Shift roundtrip, physical Shift+A, Space auto-flip),2 FAIL (second-word hint absent, after-Space Double Shift unchanged). Passive wire now shows352 ProcessKeyEvent,106 SetSurroundingText; all seven restoration checks true. First-word screenshot shows the literal prefix outside the IME underline; general visual acceptance remains partial, including brief initial preedit and editor spellchecking. The click also exposed Writer's header guide; do not confuse it with IME styling.

Qualified controls `stock-observed-v57-native-controls/summary.json`: ordinary desktop-launch Kitty hint/Tab and closed Double Shift roundtrip PASS, Chrome text-field Tab1/1 PASS; six restoration checks true. Chrome ordinary-prefix underline remains OPEN: functional Tab PASS does not certify styling. Two measured active LO toggles160.747/175.966ms, no p95 claim from two samples. No full browser matrix or GNOME rerun; historical100PASS/76FAIL/144NOT_TESTED of320 unchanged. Next: identify the first shared fresh-source/word-boundary loss behind LO's two remaining failures and finish Chrome prefix styling without adding ownership or weakening the verifier.

## Upstream PR checkpoint — 2026-10-22T00:23:43.259749+00:00

User requires regular read-only checks of https://github.com/radislabus-star/lay-public/pulls. Rule saved in AGENTS.md: task start, substantive long-session milestone, before push/release and at handoff. Live GitHub CLI shows5 open: #68 zbus and #65 serde_json each5/5CI PASS; #67 rayon4/5; #66 sha2 and #64 arboard3/5. CI green is not review/acceptance. No merge, closing or contributor message performed. Cached browser page still reports9 stale PRs and is not current evidence. Receipt: local shift-hint-20261021T1242Z/upstream-pr-review-20261022T002343Z.json. Continue active LibreOffice fix; do not replace it with dependency work.

## Current continuation — 2026-10-21T23:58:21.804873+00:00: v56 affected source457/457 PASS; remote build/native pending

Remote stock-observed-v56-verified/result.json:457 PASS,0 FAIL; discovery12.991s, execution93.537s. Earlier stock-observed-v56-full/result.json remains456 PASS/1 FAIL: the Reset freshness fixture required persistent owned preedit while declaring stock-client mode. It now explicitly establishes exact-refresh capability before real initial input. All Reset, fresh-read, duplicate and edit-refusal assertions remain; separate actual stock callback and revoked-owner tests prove literal transfer. The next discovery failure was a missing explicit capability-constant import; it is fixed, with no guard weakened. Exact source closure is pinned by the passing receipt.

Harness2 task20261021T235242Z-f11aa21c is running; completed base1 stage8/10 finds no demonstrated blocker in the frozen bounded design. Independently checked plain commit, live WordScope/token matching, generation cancellation, snapshot clearing and freshness floors. Clarification: the helper requests no layout switch; ActiveCompositionCommit::plain retains the existing executor's profile/layout reconciliation when required. Do not claim it performs no possible layout operation. Review is source-only; no native acceptance presumed.

Installed v55 engine1460682/b2b109b1 unchanged. Next: guarded remote graph/build, source-pinned engine-only deployment, then actual LibreOffice first/second-word screenshots, hint/Tab, Shift, active/closed Double Shift and Space; short ordinary Kitty and Chrome controls. Preserve user apps, daemon/tray/global IBus/desktop, existing delivery/authority contracts and historical browser counts100PASS/76FAIL/144NOT_TESTED of320. Chrome ordinary-prefix issue remains open.

## Current continuation — 2026-10-21T23:49:59.097429+00:00

Remote runtime remains v55 engine1460682/b2b109b1; native LibreOffice typed prefix still underlined, closed Double Shift FAIL. v56 causal5/5 PASS after actual delayed-SSI and pending-prediction RED; full457 source gate running. Transfer only literal live owned preedit via existing plain commit, cancel old display generation, discard pre-commit projection, retain fresh-ACK floors. Existing AdmissionToken/WordScope checked; no new owner/transport/client bypass, no candidate/hint acceptance. No runtime change yet. Harness2v55 full report completed8OK/8of10 static, independently nativeFAIL. Next: fullgate, frozen v56 Harness2 review, graph/release build and engine-only install on exact pins, actual LO screenshots/Tab/Shift/DoubleShift plus ordinary Kitty and Chrome controls. All builds/tests/graph/native remote guarded. No publication/local install or daemon/IBus/desktop restart. See owning preflight and2026-10-22-stock-observed-literal-transfer decision.

## Current continuation — 2026-10-21T23:40:13.096653+00:00

Remote engine v55 PID1460682/b2b109b1 installed engine-only; protected runtime unchanged. Native LO v55 functional5/6, closed Double ShiftFAIL; presentationFAIL in both screenshots. First key reaches engine before first SSI (empty InputContext SSI not forwarded). Thus v55 source452PASS did not solve actual first transition. Proposed v56: existing literal-preedit -> ordinary commit transfer on exact real projection, then do not treat later missing snapshots as lack of source capability. No fake ACK/new owner/route. Preflight contains alternatives/consequence analysis before production changes. New causal RED before edit, affected source gate and actual LO/Kitty/Chrome acceptance after. Harness2v55 task20261021T232546Z-71dfa570 still running; bounded8/10 stage accepted source only, native disproves applicability. No local build/install, no publication.

## Current continuation — 2026-10-21T23:29:11.969895+00:00

Owning checkout: /home/ubu/projects/lay-browser-final-fixes-20261018. Remote only.
User accepts Kitty; Chrome underlines typed prefix; LO v49 reproduces same issue.
One-conjunct observed-stock ManagedCommit candidate: v55 causal26/full452 PASS.
Remote stock-observed-v55-build running; engine still v39 PID1208224/f36012e4.
Harness2 projectlay2 task20261021T232546Z-71dfa570 running (8 frozen inputs).
Do not install until bounded review accepted. Then engine-only deploy using
stock-observed-v55-install.py; no daemon/IBus/tray/desktop restart. Native LO
v55 six-row script prepared in local cache; preserve user LO1285267. Check
visual ordinary prefix, suffix-only opaque black-on-white hint, Tab, Space,
Shift and Double Shift; short Kitty regression. Chrome actual AT-SPI source
v51 is qualified but no provider/executor adopted, plain prefix still open.
See owning ime-inline-hint-preflight-2026-10-20.md and exact receipts.

## 2026-10-21T19:02:21.754237+00:00 — Kitty startup installed; next mechanism is Chrome lifecycle during native replay

Remote helpera2859159 and unchanged wrapper984a84c7 installed;19/19scoped sourcePASS. New review Kitty561418 on :71 has IBus; keep old432607 and all user apps open. Ordinary-command native hint/Tab,10/10rapid targets,Shift4/4PASS; RU punctuation and old geometry FAIL remain. Engine326051/cf76bc unchanged. Harness2USED20261021T183803Z-de6d1dd7 completed8callsOK; review8/10 found PATH false success, independentlyRED/fixed/GREEN after review. Exact receipts and safety limits: latest [preflight](../architecture/ime-inline-hint-preflight-2026-10-20.md), [startup decision](../architecture/decisions/2026-10-21-kitty-command-startup.json).

Chrome omnibox after-Space remainsFAIL. All deletion keys arrive; Reset/FocusOut/In cycles revoke local owner while stale shared exact-replay suppression consumes native Backspace. Fresh trace in shift-hint-20261021T1242Z/chrome-omnibox-engine-e-v23.jsonl. No core patch yet. Need causal source RED through existing lifecycle helpers and a solution preserving C05/C07/C08; do not blanket preserve old authority or bypass verifier. Active conversion and hint/Tab pass separately. Invalid early observer fixtures are NOT_TESTED. Global browser100/76/0/144 of320 unchanged. All builds, tests, graph and physical checks remain remote/guarded. No publication, GNOME or full-matrix claim.

## 2026-10-21T18:12:18.096679+00:00 — checks completed on current remote KDE bytes; open client failures explicit

Current engine326051/cf76bc925c96a738 remains installed; fresh audit confirms source6d488123/1257files, both installed/activation files, config/tray and8protected processes. GTK/Qt25/25functionalPASS,5desktop hint captures reviewed. Terminal retained: Shift8/8, DoubleShift20/20intermediate targets,Tab2/2functionalPASS; RUpunctuation2FAIL, Lay causalityunproved. Browser short3PASS/4FAIL:20/20targets,15/16late deadlines. Kitty missing-hint captureFAIL remains even though opt-in diagnostic gets validrect. Chrome underline/Firefox iframe overlap remainFAIL. Historical browser100PASS/76FAIL/144NOT_TESTED of320 unchanged.

Harness2 task20261021T174153Z-6dce90b1 finished8/8OK/no fallback;7/10incomplete static review, no demonstrated defect. Preserve missing-wrapper/truncated-read gaps; frozen inputs/12excerpts independently match. Owning [preflight](../architecture/ime-inline-hint-preflight-2026-10-20.md), latest entry, records exact receipts and `/home/ubu/.cache/lay/development/browser-final-fixes-20261018/shift-hint-20261021T1242Z/native-short-checks-v21-local-review.json`. No production patch in this verification milestone; no fullmatrix/GNOME/release/push/globalrestart. Next: isolate geometry invalidation order and remaining browser latency, without reopening rejected frame/settle experiments. Keep all further builds/tests/graph work remote and guarded; use Harness2lay2/xhigh with independent acceptance.

Documentation graph refresh receipt: `/home/e/projects/lay-development-runner/kde-install-control-20261020/native-short-docgraph-v21/result.json`. The compiled/native source identity above is the historical build pin; the refresh may change only generated graph metadata. It does not rebuild or certify a different executable.

## 2026-10-21T17:54:00.062569+00:00 — ordinary Space repair passes native controls; open failures retained

Scoped development engine326051/SHA`cf76bc925c96a738e2b4a9a4a37619713e0efb3d544b54d4d9b75f56337604b0` installed17:43:55UTC. Causal real-Space RED1/1, affected isolatedGREEN266/266, graph/architecture PASS. Independently recomputed local/remote1257-file source+mode identity `6d488123b563c16a7972862a88a79cad1cd4257428db6592c067f567eea922cd`. Only engine replaced; daemon1802bb5e, globalIBus, tray, config and protected8 preserved. No new delivery/edit/learning authority.

Native normal-profile browser receipt `/home/e/projects/lay-development-runner/kde-install-control-20261020/stock-user-space-browser-native-v21/summary.json`: Firefox iframe ordinarySpace roundtrip+decoder PASS; Firefox iframe and Chrome text Tab roundtrip+decoder2/2PASS. Four after-Space rapid-five cases remainFAIL: all20/20 intermediate targets appeared (v20:13/20), but15/16 intergesture deadlines missed. Twenty release-to-DOM samples median167.92ms,p95234.95ms,max243.08ms; retained clock uncertainty, no relaxed cadence or latePASS.

Native terminal receipt `/home/e/projects/lay-development-runner/kde-install-control-20261020/stock-user-space-terminals-v21/summary.json`: Kitty/Konsole Tab+roundtrip+decoder2/2PASS; active/Space five-toggle4/4cases and20/20targetsPASS; heldShift RU/US left/right8/8PASS. RU punctuation2FAIL (`!,Ж()_` instead of `!?:()_`), Lay causality not proved. **Independent screenshot inspection: Konsole ordinary prefix+complete opaque suffix PASS for this capture; Kitty hint absent FAIL**, despite functionalTabPASS. Kitty snapshot has suffix`ерка` but`rect=null`; do not merge functional and presentation verdicts. Chrome prefix underline and Firefox iframe prefix overlap remainFAIL. Restoration checks alltrue.

Harness2 frozen v21 review task20261021T174153Z-6dce90b1 stillrunning; no review acceptance claimed. Local evidence manifest `/home/ubu/.cache/lay/development/browser-final-fixes-20261018/shift-hint-20261021T1242Z/v21-receipts-manifest.json` retains39explicitJSON/PNG files, no browser profiles. Historical full browser100PASS/76FAIL/144NOT_TESTED of320 unchanged. No fullmatrix, GNOME, release, push, local install or global restart. Next: explain first lost Kitty hint geometry and existing browser latency, without repeating rejected frame/settle experiments.

## 2026-10-21T17:29:19.515707+00:00 — exact new runtime checked; stock completion failure repaired in bounded native cases

Installed scoped KDE development engine249471/SHA882385aeec5233cb721998a22e22fb3f8177bf21b021061a17cce14c999d2f45 at17:20:24UTC. Guarded build and architecture PASS; local/remote canonical1257-file bytes+modes match SHA75470100e8aef679d1c7b7f6d2e897f637de881b3ac45d8f324f6ff39b1011c8. Reused262/262 isolated sourceGREEN; no redundant rerun. Daemon49153/1802bb5e, globalIBus, eight protected processes and config preserved. Delivery and mutation authority unchanged.

Actual owned normal-profile Firefox iframe-rich and Chrome text each PASS: native `пров`, hint screenshot, real Tab to `проверка `, DoubleShift both directions, next-letter decoder. Firefox raw NBSP retained. Receipt `/home/e/projects/lay-development-runner/kde-install-control-20261020/stock-completion-space-browser-native-v17/summary.json`; repeat read-only geometry diagnostic `browser-hint-geometry-v19` also2/2functionalPASS. Source fixture setup failures v17/v17b are excluded from valid causalRED v17c.

Short current-byte terminal receipt `/home/e/projects/lay-development-runner/kde-install-control-20261020/stock-completion-terminals-v18/`: Kitty/Konsole hintTab-to-DoubleShift-to-nextletter2/2PASS; active/Space five-bursts4/4cases with20/20 intermediate transitionsPASS; left/right ordinaryShift RU/US8/8PASS. Twenty burst transitions median19.88ms,max21.55ms from secondShift press to PTY exact target (not full-gesture/end-to-end browser latency). RU punctuation2FAIL retained; Lay causality not proved. Independently viewed current Kitty/Konsole captures: typedprefix ordinary and entire suffix black on opaquewhite. This visual result covers those two captures only; old Kitty missinghint capture remains historicalFAIL. All restoration checks true.

Current browser captures stillFAIL appearance: Chrome typedprefix underlined; Firefox iframe suffix covers/hides prefix. Prior rapid after-Space browser failures remain open. Historical fullbrowser100PASS/76FAIL/144NOT_TESTED of320 unchanged; no fullmatrix/GNOME/private/fullscreen/release gate. Harness2 new review20261021T171945Z-cece8c8c running; v16 degraded7/8 review independently audited, static scope only. No push/release/local install/global restart.

## 2026-10-21T17:17:45.055918+00:00 — stock first-receipt candidate source GREEN; native pending

`stock-completion-space-v17c-green/result.json`:262/262 exact affected identities PASS in isolated bwrap sandboxes, default stack;12.45s compile plus10.07s execution. Test artifact4041b945407ab517cc0a984fe62844fd5608dab27722dfb072fc1219b2574dd0. Source-only; currently installed174016/aecf29d3 still has the retained native Firefox failure. Conditional whole-field expectation keeps KnownStart/UserInput/output routes unchanged. New negative cases cover old field prefix, wrong word/boundary count, trailing field text, selection/caret, expiry/epoch/foreign owner, current token/mode/owned composition/exact edit and incomplete/stale observed starts.

Harness2 v16 review task20261021T165016Z-bbdbb45b completed degraded:7/8callsOK, dom1 timed out240s, no fallback; selected bounded static review9/10, no findings, explicit missing implementation proof gaps. This does not cover the new v17 delta and does not override nativeFAIL. Build and physical acceptance remain pending. Historical full browser100PASS/76FAIL/144NOT_TESTED of320 unchanged.

## 2026-10-21T17:09:17.464644+00:00 — actual Firefox fails; initial observation absent

Installed engine174016/aecf29d3 retains the v16 source257/257 PASS but Firefox iframe Tab-to-DoubleShift FAIL. Chrome text control PASS. Trace `firefox-iframe-completion-trace-v17` proves stock caps41, exact refresh unavailable, and no surrounding snapshot until AFTER Tab. The v16 regression supplied a prior snapshot, so its qualified receipt cannot run in the actual failure. This native failure is retained and is not reclassified by source success.

Short native receipt `completion-space-short-native-v16`: GTK/Qt25/25PASS; terminal20/20 individual DoubleShift transitions and ordinaryShift8/8PASS; RU punctuation2FAIL baseline. Viewed screenshots: Chrome typed-prefix underlineFAIL; Firefox iframe prefix hiddenFAIL; Kitty hint absent at capture (functionalTabPASS only); Konsole black-on-white complete suffix bounded visualPASS. All protected runtime/config/layout restorations true. Full historical browser100PASS/76FAIL/144NOT_TESTED of320 unchanged. No release/push.

Next causal regression uses stock caps41 and NO initial surrounding callback. A candidate must acknowledge only a subsequently observed exact whole-field owned word plus one boundary, without inventing a prior prefix, downgrading a known stale witness, or changing edit/learning authority. Receipt paths are under `/home/e/projects/lay-development-runner/kde-install-control-20261020/`; local copies under `shift-hint-20261021T1242Z/remote/v16/`.

## 2026-10-21T16:54:11.126138+00:00 — completion receipt source257/257; native proof pending

First failed transition proved in remote Firefox iframe: Tab commits ASCII Space, client reports NBSP, next ManualToggle delegates but V4 sees unknown-context. One real-key source regression RED; existing exact owned-Space receipt reused for matching authorized completion, with all field/owner/prefix/selection/expiry guards. Affected isolated257/257 GREEN, no runtime/delivery authority change. Decision `2026-10-21-completion-owned-space-receipt.json`; owner entry has exact receipts and limits. Harness2 task20261021T165016Z-bbdbb45b running. Current engine4076822/5934db2f and original daemon49153/1802bb5e unchanged; next scoped development engine build/deployment and native Firefox/Chrome/Kitty/Konsole controls. Presentation and browser fast-Space burst failures remain open; full historical table unchanged. Rejected frame-batching review finished8/10, independently audited, candidate still removed. No release/push/global restart.

## 2026-10-21T16:18:36Z — real native checks; rejected batching restored

Current remote daemon is original1802bb5e/PID49153; engine5934db2f/PID4076822 unchanged. Candidate0941177b passed52 affected source checks and exact1257-file source identity but improved neither browser case counts nor timing. Native browser subset remains10PASS/10FAIL each; all ten after-Space bursts fail. GTK/Qt25PASS, terminal20toggle transitions/hint-Tab2/ordinaryShift8PASS; Russian punctuation2FAIL retained. Fresh terminal screenshots inspected: typed prefix ordinary, entire suffix black/opaque white. Chrome prefix underline remainsFAIL; Firefox iframe presentation/Tab unaccepted.

Candidate source and installed daemon were restored at16:16:20UTC; no rejected batching delta is retained. Receipt `x11-frame-fence-rollback-v11/result.json` verifies protected process/config/unit preservation and mode/layout restoration. Next diagnostic `firefox-space-admission-v12` confirms five DelegateExactImeTail replies in1.22–2.35ms but targets163–173ms, overlapping subsequent gestures. First expensive transition lies after admission in this case; intermittent missing Firefox targets remain open. Do not weaken the timing gate or reuse cached focus authority. Owning document `docs/architecture/ime-inline-hint-preflight-2026-10-20.md` contains exact hashes, scope and receipts.

Harness2 first frame review20261021T155646Z-81acd72b timed out; narrowed task20261021T160217Z-07b2a168 is still running. Review acceptance pending. Full historical matrix100PASS/76FAIL/144NOT_TESTED of320 unchanged; no release, push, local install or global restart. Rejected4/8ms settling and persistent replay master remain forbidden retained experiments.

## 2026-10-21T15:56:03.767890+00:00 — finite X11 frame experiment, not installed

Metadata-only native profile `bridge-v7b-space-dbus-profile-v10` retains five Chrome targets but misses fast deadlines. Bounded profile: 85 fresh KWin reads across the run; one getLayoutsList call80.17ms; output also performs serial same-connection fenced writes. Do not infer that frame batching fixes the whole latency. Candidate decision `2026-10-21-x11-closed-frame-fence.json` keeps pacing and all safety/settlement contracts; remote owned-Xvfb proof, source build, bounded Harness2 review and native comparison pending. Current daemon1802bb5e and engine5934db2f unchanged. Full browser table unchanged; Chrome prefix underline and Firefox iframe presentation/Tab remain unaccepted.

## 2026-10-21T15:44:34.075357+00:00 — browser fast after-Space failure is shared, not closed

Engine4076822/5934db2f unchanged. Fresh normal-mode20-case diagnostics per browser: Firefox10PASS/10FAIL, Chrome10PASS/10FAIL. All10 active five-toggle bursts PASS; all10 after-Space FAIL, with late targets and Firefox lost toggles. Native focus scaling/timing fixture corrected, original receipts retained; full historical320 table unchanged. Next: measure first expensive/lost transition in existing delegated replay, not weaken guards or change route. Harness2 task20261021T151518Z-2319fcc6 finished8/8OK, no fallback, adapter-only static review9/10/no findings; independent Qt/242source receipts recorded in latest owning preflight. Chrome prefix underlineFAIL; Firefox iframe presentation/Tab unaccepted. No release/push/global restart.

## 2026-10-21T15:33:20.095711+00:00 — activation fence source GREEN, native Qt PASS

Current remote engine PID4076822 SHA5934db2f64e5fee37e376b85eb955b6e1bdc3b2aaa7de7c681156dc846b39b01. Candidatev7b:242/242 Rust+14JS+2PythonPASS; all25GTKQt and20terminalDoubleShift transitionsPASS. Same four TD121 stack-crashing fixture cases pass after one-await-loop shaping, with default stack/assertions/deadline unchanged. Full source gate remains NOT_TESTED. Firefox retains five exact burst targets but old onset timingFAIL; Chrome focusBLOCKED due measured DPR1.0625 CSS/RFB coordinate error. Diagnostic-fixture corrections pending native acceptance. Full browser table unchanged. Owning result and receipt paths: `docs/architecture/ime-inline-hint-preflight-2026-10-20.md` latest entry. Harness task20261021T151518Z-2319fcc6 still running. No release/push or global restart.

# lay2 — current working handoff

## 2026-10-21 15:11 UTC — checks are native; activation-race fix in progress

Installed development engine c542167e/PID3916379: Konsole10/10 and normal
desktop-launch Kitty10/10 Double Shift with next-letter controls PASS.
Terminal hint/Tab2/2, ordinary held Shift8/8 PASS; RU punctuation baseline FAIL.
Source200/200, JS14/Python2, architecture PASS. Exact1257-file content identity
is recorded in the owning preflight; differing git HEADs are not equivalent.

Qt plain after-Space FAIL reproduced twice. DBus plus admission trace proves
a mode read races native activation by about0.9ms, returns Busy and cancels
exact replay before any text deletion. Controlled source regressions now
detect both premature Busy and incorrect timeout; concurrent-Bridge safety
control passes on old source (bridge-fence-red-v7b). New adapter wait is local,
GREEN/build/deployment/native proof PENDING. Keep existing20ms deadline,
fresh Ping, ownership/field/lease validation and a single output executor.

Firefox normal smoke9/16 PASS; Chrome4/8 executed PASS and8NOT_TESTED after
initial iframe focus BLOCKED. Chrome typed-prefix underline and Firefox
iframe prefix/Tab remain FAIL. Full historical browser table unchanged.
See docs/architecture/ime-inline-hint-preflight-2026-10-20.md, latest entry,
for receipts and limits. No publication or global service restart.


## 2026-10-21 14:34 UTC — current-terminal qualification before manual admission

Candidate059f8334: affected source200/200, hint JS14/14 and Python2/2,
architecture PASS. Development-only deployment-v4 pins PID3863797 and both
actual activation/installed executables; daemon/tray/config and eight protected
processes unchanged. Native `terminal-candidate-v4-{konsole,kitty}-five`:
after-Space five consecutive Double Shift PASS in each (14.36–22.10 ms visible
latency, ten samples); active-word first PASS, second FAIL in both. The table
of complete browser runs is unchanged. Do not claim terminal acceptance.

Trace `terminal-candidate-v4-konsole-active-trace/engine-trace.jsonl` shows
cursor geometry arriving before activation Ready; the second active-word RPC
returns context_authority with a live bridge token and no current terminal
qualification. The existing refresh runs only after eligibility, so UnknownStart
never reaches it. Move the same current-field probe before eligibility and
compute bridge liveness after the await. Preserve all owner/field/epoch/token,
sensitive, atomic, exact-refresh and contradictory-source guards. No new owner,
backend fallback, selection relaxation or delivery executor. Native five-toggle
control is the exact regression: both spaced and active words, then one next
letter checking the decoder. Existing affected source guards also rerun; no
source-order assertion or artificial desktop success is used as acceptance.
Decision: `2026-10-21-terminal-manual-admission-probe-order.json`.
Implementation/proof of the ordering repair PENDING. Chrome typed-prefix
underline, RU punctuation, full-source stack overflow and full GNOME/native
browser matrices remain open.

Remote evidence root:
`/home/e/projects/lay-development-runner/kde-install-control-20261020/`.

## 2026-10-21 — ordinary Shift restored; bounded desktop module relocation

Latest runtime checkpoint: 2026-10-21T09:27:20.035659+00:00. User priority was
inability to type uppercase with ordinary Shift. The same RU failure reproduced
without IME; xev showed KeyPress arriving after a Shift release. Selected
live `nomodtweak` restores 8/8 RU/US left/right Shift controls across normal-launch
Kitty/Konsole. Persisted only `-nomodtweak` in the existing session launch script;
all ten protected processes, engine d4215880, daemon 1802bb5e and config bytes are
unchanged. No restart, install, model change or IME delivery-route change.
RU punctuation remains a measured baseline failure: each six-row terminal suite
is 5 PASS / 1 FAIL. Do not label whole terminals or universal input PASS.

The user's consistency principle is recorded in AGENTS.md and decision
`2026-10-21-desktop-integration-consistency.json`. Existing native-terminal focus
probes moved into `window_interaction/desktop.rs` and `desktop/{kde,gnome}.rs`.
One existing backend resolver, explicit dispatch, unchanged shared authority and
typed execution; KDE fails closed and does not fall through to GNOME. Explicit
X11/Niri compatibility is preserved. Mechanical body/assertion parity PASS;
7/7 existing source controls before and after PASS, remote rustfmt PASS. This
source relocation is not installed and does not repair Konsole Double Shift.
Guarded graph refresh and architecture checks PASS. Independent final comparison
matches all 1257 canonical source files (hash and mode) and six graph artifacts
between the owning checkout and remote workspace. Only generated graph metadata
was refreshed after the seven controls; this does not certify a newly compiled
runtime. Final evidence: local evidence root `acceptance.json`,
`final-pin-verification.json` and `RESULT.md`.

Owner/evidence: `docs/architecture/ime-inline-hint-preflight-2026-10-20.md`.
Remote receipts under the existing KDE control root:
`ordinary-shift-*/`, `desktop-focus-module-{before,after}-v1/`.
Local evidence:
`/home/ubu/.cache/lay/development/browser-final-fixes-20261018/ordinary-shift-20261021T0838Z/`.
Harness2 first task 20261021T083111Z-15f62b07 FAILED after a 240s base timeout.
Narrow causal review 20261021T085025Z-03eb4287 completed with per-task 600s calls:
8/8 OK, no failures/fallbacks, report/core OK. Selected review recommends A 8/10
for the tested RFB session; all eight inputs match across three copies, and
report/call receipts/selected review were independently checked. Acceptance:
local evidence root `vnc-review/audit.json` (transport review only).
It covers the transport diagnosis only, not the mechanical source relocation.
Post-review native Double Shift control: Kitty round trip PASS (20.885/17.491 ms,
two samples only); Konsole after-Space FAIL unchanged. No burst/full-matrix claim.

Unchanged open gates: Konsole Double Shift after Space/moved boundary, RU
punctuation, the last full source stack-overflow failure and GNOME native scope.
Latest complete available browser matrix remains historical:
Firefox 92P/68F/0B/0NT, Chrome 8P/8F/0B/144NT, 160 each. No browser rerun or regrade.

## Current checkpoint — native candidate d421 and Kitty presentation reference

Remote KDE DISPLAY=:71 now runs PID511466 / engine SHA-256
`d4215880a93739b4226efe1aabb7f68507e219adbf4ddae0de827bb1a05c1e51`.
The actual IBus XML executable is `/home/e/projects/lay-td119-gate-v1/target/release/lay-ibus-engine`,
not the installed symlink. Deployment-v3 updated both and verified `/proc` bytes;
IBus XML, global daemon, desktop and protected user windows were preserved.
Receipt root `/home/e/projects/lay-development-runner/kde-install-control-20261020/`:
`suffix-readout-artifact-v3/result.json`, `suffix-readout-deployment-v3/result.json`.
Build-v1's hardcoded artifact hash was stale; never use it as installed proof.

Current readout repair: 27/27 exact source controls, native Konsole hint/Tab/blur
2/2 PASS, full Kitty4/4 PASS, full Konsole2 PASS/2 FAIL. GTK/Qt five kinds22/25
functional PASS; Space Double Shift fails in GTK TextView, Qt LineEdit and
Qt PlainTextEdit. Native receipts are under `live-regression-20261020/`:
`konsole-readout-v2`, `konsole-readout-full-v3`, `kitty-readout-v2`, `all-native-v1`.
Copied local evidence root is
`/home/ubu/.cache/lay/development/browser-final-fixes-20261018/kde-live-regression-20261020T175149Z/`.
Source full gate FAIL: canonical `suffix-readout-canonical-source-v2` aborts
unchanged deep native-transfer regression on the ordinary Rust test stack.
Do not claim historical943/943, targeted27/27 or nativePASS as full source PASS.

Latest user: remove50% transparency, match Kitty. GNOME auxiliary style now uses
solid gray without touching opacity; first opacity-only JS check12/12 PASS with
controlled RED8/12. Inspecting live screenshots also found GTK Entry's huge
hint: entire field-height was interpreted as font height. New frontend-only
repair keeps natural theme font for rectangles taller than two natural lines;
normal terminal sizing is preserved. Added real JS view geometry regressions;
these final checks/native redeployment are still PENDING. Do not overwrite
local accepted Kitty or claim new frontend presentation installed yet.

Full native browser candidate run is active:
`live-regression-20261020/browsers-d421-v1/`, wrapper `run-browsers-current.py`.
One GUI focus owner at a time. Actual RFB input, DOM read-only observation,
runtime before/after SHA pin, normal/private/fullscreen, ten fields,160/browser.
Last complete historical browser table remains Firefox85P/75F/160 and
Chrome9P/7F/144NOT_TESTED/160 until complete new receipts are independently read.

Real Harness2 task `20261020T183025Z-e1b9e0e1` is running, projectlay2,
gpt-6.1-sol/xhigh, per-task timeout600s. Frozen bounded readout review, not a
source edit or client acceptance. Independently read report and source pins
before accepting; this review does not include later frontend styling changes.
Still OPEN: copied Plasma Kitty pin bypasses managed launcher, untested actual
KWrite/WezTerm, actual browser/GNOME presentation, deep source stack overflow.
No release, push or completion sound in this unfinished iteration.

## 2026-10-20 18:18 UTC — confirmed Konsole failure and causal suffix repair

Active checkout/branch remain `lay-browser-final-fixes-20261018` / `codex/ime-muted-hint-20261019`, HEAD `2ec7392a`. Do not build the old dirty main. The installed candidate `02fb706d...` is NOT accepted: actual owned Konsole raw-PTY check is0/4. Native letters are ordinary, but suggestion/Tab and Double Shift fail. Trace confirms caps41 without a snapshot and current KWin qualification. New source RED reproduces lost UnknownStart display and rejected suffix Tab (25/27); isolated display/append repair is GREEN27/27, execution0.481s, compile/discovery12.669s. Whole-word edit/Space and delivery guards are unchanged. Remote guarded graph/release build is running; new native acceptance is pending. Owner: `docs/architecture/ime-inline-hint-preflight-2026-10-20.md`; decision `2026-10-20-native-terminal-suffix-readout.json`.

Actual Harness2 USED task `20261020T175347Z-5c7e3e0c`, gpt-6.1-sol/xhigh; 7/8 calls succeeded, one240s timeout, degraded. Frozen pre-repair triage rates acceptance4/10; no source edits. Independently verified actual engine with full desktop environment, normal Chrome direct launch, Kitty pinned launcher difference, native Konsole trace and causal RED/GREEN. Receipt: `/home/ubu/.cache/lay/development/browser-final-fixes-20261018/kde-live-regression-20261020T175149Z/harness2-acceptance.json`. The report is limited diagnosis, never client acceptance.

Normal user Kitty PID391181 starts from the copied Plasma icon `$XDG_DATA_HOME/plasma_icons/kitty.desktop` with `Exec=kitty`, bypassing managed `applications/kitty.desktop` GLFW setup. Managed launch native Double Shift/hint/Tab pass, prefix plain and suffix gray (screenshots inspected). Its blur screenshot hides hint and leaves PTY unchanged: the v1 observer incorrectly required cached projection text to be empty despite rect=null. v2 requires no drawable rectangle plus unchanged PTY, and restores owned focus before the next independent case. Original receipts preserved. No existing user app was restarted.

## Active client regression — 2026-10-20 17:51 UTC

User rejects candidate 02fb706d: Konsole and Kitty fail; Chrome/default application launches also reported failing. These are user FAIL reports, not yet reproduced in owned fields. Do not reuse source 26/26 as client PASS. User explicitly requires Harness2 for substantive Lay debugging/fixes/review, project lay2 gpt-6.1-sol/xhigh; save actual task/report and independent verification. User now asks the owner to list and physically test all relevant remote KDE windows before returning another candidate. Preserve current user windows, fix shared startup/session failure first, then use normal launches and existing native field protocols; no broad build until first failed transition is recorded. Doctor PASS at this turn; new task is being started from frozen evidence in /home/ubu/.cache/lay/development/browser-final-fixes-20261018/kde-live-regression-20261020T175149Z.

## Current short Konsole candidate — installed 2026-10-20 17:33:49 UTC

The user accepts Kitty's display, takes over manual Konsole testing, requests
short checks rather than another full matrix, and requires one local completion
sound immediately before the final reply. No synthetic input was sent in this
iteration. Preserve both user Kitty windows and Konsole.

Remote KDE DISPLAY=:71 now loads engine PID 351233 / SHA-256
`02fb706d1dcca49066c2e21ee95acc7251ce4b2de634ae261726060765a59038`
(development candidate, version still 1.0.94; not a new published release).
Installation receipt:
`/home/e/projects/lay-development-runner/kde-install-control-20261020/konsole-engine-deployment-v1/result.json`.
Status is **INSTALLED_FOR_USER_TEST**, never physical-client PASS. Loaded bytes,
daemon input readiness and hint snapshot v1 read back successfully. Original
daemon, tray PID 28753, IBus daemon, desktop, Kitty and Konsole processes/bytes
are unchanged. KDE RU layout, selected lay-ime-ru and configuration are retained.
The local GNOME runtime was not installed or restarted.

Implementation reuses the existing exact KWin terminal qualification for stock
Konsole caps41, allowing typed letters to remain native ordinary text. Pending
cursor geometry survives installation only for the same current request/nonce,
Ready outcome, target and local focus serial. Coordinates grant no input/edit
authority. Explicit exact-refresh and atomic exclusions remain authoritative.

Source gate `konsole-fast-v4`: **26/26 PASS**, execution 0.496244297 s;
discovery/compilation 12.811072 s. All identities are selected from the existing
manifest/discovery and execute the real adapter/key/output paths. Retain v1's
fixture failure, v3's generated-metadata precondition refusal (0 tests), and
v3-red's real late-SurroundingText failure (25/26). The latter was found by
independent review and fixed with one condition preserving native mode only
while `uses_native_terminal_input()` remains true; late exact-contract fallback
also passes. No rejected runtime fix was reverted/reintroduced.

Final guarded graph/build receipt is `konsole-final-build-v1/result.json` under
the same remote root: architecture PASS, release build 69.720850 s. All 1254
build-source pins and six graph artifacts match the owning local checkout.
The source-test/build preparation delta is only regenerated
`src/generated/architecture_graph_receipt.json`, recorded explicitly; functional
sources remain identical. Local copies are under
`/home/ubu/.cache/lay/development/browser-final-fixes-20261018/remote-x11-input-20261020/`.

Harness2 **USED**, project lay2, gpt-6.1-sol/xhigh, task
`20261020T172003Z-f3051d4a`. Final report independently inspected and bound at
2026-10-20 17:45:35 UTC: status/core_status ok, all eight calls ok, zero failed
calls or fallback stages. Selected c1 review is 7/10 and requests the same
late-capability repair above. The frozen review predates that repair and is
not post-repair/native approval. Its conditional geometry-clear concern stays
OPEN with the existing PTY position-freshness gap; it is not a new demonstrated
native failure. The owner verified the one-condition repair, causal RED/GREEN,
all six unchanged frozen inputs, 1254 final source pins and six graph artifacts.
Report:
`/home/ubu/.local/state/harness2-codex/projects/lay2/tasks/20261020T172003Z-f3051d4a/report.json`.
Independent binding and review SHA:
`/home/ubu/.cache/lay/development/browser-final-fixes-20261018/konsole-short-review-20261020T171947Z/acceptance.json`.
The experience library remains empty; acceleration/token savings are not
demonstrated. Do not start another broad review or native matrix merely to
repeat this unchanged result.

Next: user's Konsole presentation/Tab feedback. Current changed-byte native
Konsole, GNOME and full browser acceptance remain NOT_TESTED. Retain the last
complete native matrix: Firefox 85 PASS / 75 FAIL / 160; Chrome 9 PASS / 7 FAIL /
144 NOT_TESTED / 160. PTY producer-lag and broader TD-160/TD-161 gaps remain OPEN.

## Earlier native checkpoint — 2026-10-20, runtime rechecked 16:06 UTC

The actual remote KDE desktop is DISPLAY=:71. Its newly opened, ordinary KDE
menu Kitty (PID 49770) has GLFW_IM_MODULE=ibus from the managed desktop entry.
Independent inspection of `user-kitty-native-v2-hint.png` shows ordinary white
`пров` followed inline by gray `ерка`, with no prefix underline or selection;
`user-kitty-native-v2-after-tab.png` shows `проверка `. No shell Enter was sent.
The original user Kitty PID 14815 lacks GLFW_IM_MODULE and remains open;
do not claim that already running process acquired the new launch environment.
The native menu launch and desktop cache refresh succeeded. The preceding
`kioclient5` launch timed out and remains a failed diagnostic, not a PASS.

Installed engine PID 28589 / SHA-256
`9e751d3ccff8581f7c2fb2d0b4c4da189a401d402cf3cf30f0b605578cd257c2`
passed 943/943 IME source checks on unchanged stack limits (`run-4jbmf5hc`).
Raw PTY `inline-kitty-geometry-v5` passes 2/2 rows: explicit Tab appends only
the current suffix; blur leaves only the typed prefix. One measured Tab
reaction is 1.786365 ms; the menu-window script's 42.369224 ms is dispatch
duration and must not be presented as reaction latency.

Remote receipts live under
`/home/e/projects/lay-development-runner/kde-install-control-20261020/`:
`inline-deployment-v2`, `inline-source-build-v5`, `kitty-kde-menu-launch-v1`,
`user-kitty-native-v2`, `inline-kitty-geometry-v5` and
`inline-chrome-geometry-v5`. Local screenshots/inspection receipt live under
`/home/ubu/.cache/lay/development/browser-final-fixes-20261018/remote-x11-input-20261020/`.
The current Chrome bounded row fails its following Double Shift control;
Tab itself works. Chrome's typed-prefix underline remains OPEN TD-160.

Harness2 task `20261020T144651Z-7cc524f2` finished: 8 calls succeeded, review
7/10. Independently accepted only as bounded code review of its frozen inputs;
GNOME queued-positive-after-focus invalidation finding remains OPEN. Do not
promote the candidate as universally accepted. GNOME/Wayland native appearance
and the full changed-byte browser matrix are NOT_TESTED. Retain the last native
matrix: Firefox 85 PASS / 75 FAIL / 160; Chrome 9 PASS / 7 FAIL /
144 NOT_TESTED / 160. No global IBus, desktop or local runtime restart.

Next: fix the GNOME view lifecycle finding with a causal regression, and trace
stock-browser ordinary-input authority before changing commit timing. Never
turn a local CommitText success or geometry into a widget acknowledgement.

## Earlier implementation checkpoint — 2026-10-20 14:55 UTC

The user rejects the separate auxiliary popup and requests the same inline
appearance as their GNOME. The actually loaded local GNOME engine is still
1.0.90 (SHA-256
`2095dbb95bebd2eab79ad2512925ca3f0aa6905d4d907d07f11d1bbaac3cc82d`),
which used document preedit for hints. Do not restore that transport: the
later Firefox blur protection remains required. Local runtime is untouched.

Uncommitted implementation projects read-only hint text and cursor geometry
through the existing session bridge into the existing KDE tray / GNOME
extension. It adds no keys, acceptance authority or document edits. Missing
geometry/frontend paints nothing; a successful signal is not a painted or
accepted hint. Preflight and consequences:
`docs/architecture/ime-inline-hint-preflight-2026-10-20.md`; new decision
`2026-10-20-inline-readonly-hint-view.json` remains design-only until evidence.

Remote paint-only fixture and the shared frontend protocol tests passed.
The first Rust build passed compilation but overflowed the original bounded
test stack in an unchanged native-transfer regression (`run-i6zr0viz`).
The corrective revision puts geometry snapshots behind Arc and boxes the
new nested display futures. One subsequent launch was BLOCKED_RESOURCE
(`run-m_mgw5vu`), not a test failure. A fresh IME-only check is running in
remote `run-8NyiWb`; do not infer its result from this checkpoint.

Harness2 implementation review `20261020T144651Z-7cc524f2` is running against
frozen inputs from before the Arc/Box correction, with an isolated 900-second
timeout. The earlier 240-second design attempt timed out and was rejected.
Neither an invocation nor doctor is accepted review evidence by itself.

No candidate installed yet. Next: finish the same source target on unchanged
stack limits, independently assess the bounded review, check actual frontend
startup, then install remotely and test owned ordinary-launch Kitty and Chrome
(before-Tab screenshot, document, focus, Tab, blur and latency) before the
affected ten-field protocol. Preserve all old FAIL/NOT_TESTED counts; new view
does not resolve Chrome's independently owned-prefix underline or TD-161.
Do not restart global IBus, desktop components or the user's old terminals.

## Current checkpoint — release published, 2026-10-20 14:01 UTC

Stable public release **v1.0.94** was published at 14:01:24 UTC from commit
`5b3c6c5d92ca8d27ac59d2422b64b3e3d04950ae`. The private working branch,
public main and annotated release tag were pushed without force. Release:
https://github.com/radislabus-star/lay-public/releases/tag/v1.0.94.

The installed local menu updater entrypoint resolves to
`/home/ubu/.local/lib/lay/update.sh`. Its actual `lay-update --check` finished
at 14:01:52 UTC, exit 0: installed 1.0.93, stable 1.0.94, update available.
The installed GNOME menu calls that entrypoint with `--menu`. No local install
or runtime restart was requested or performed. Remote 1.0.94 installation and
the native evidence below remain unchanged. Publication/updater receipt:
`docs/development/release-1.0.94-publication.json`.

This release packet is finished; TD-160 and TD-161 remain OPEN. No additional
functional checks were run for the publication or bookkeeping-only changes.
Next engineering work must retain the native failures and establish the exact
authority-transfer defect before changing the existing owner contracts.

## Release evidence checkpoint — final native packet, 2026-10-20

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

At this evidence checkpoint, source gates and remote installation were
complete; documentation refresh, publication and updater verification were
pending. Their completion is recorded in the current checkpoint above.
Do not install the local runtime manually.
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
# 2026-10-21 13:49 UTC — current candidate is not client-accepted

Remote engine PID3722599 SHA256 `c5dd29ec6254fa5434294da32a04d06bbb0233ecfdccfeb3a396e9be8fdfaca6`;
tray user unit `lay-kde-tray-candidate-20261021.service`; daemon/config unchanged.
Installation receipt `shift-hint-deployment-v3/result.json` under remote
`kde-install-control-20261020`. Scoped source197 PASS does not certify clients.
Kitty Double Shift round trip PASS13.8248/18.9078ms. Konsole first flip
PASS15.8173ms, return FAIL: post-handoff SurroundingText has cursor7/anchor0;
the existing verifier selection check rejects it correctly. Do not weaken it.
Tab target commitment passes in the three owned controls; actual terminal
hint rendering FAILS because white background is ignored and black suffix
is unreadable. Chrome typed-prefix underline remains open. Ordinary held
Shift Konsole RU/US left/right4/4 PASS; RU punctuation still FAIL.
Evidence and next contract-boundary investigation are in the current section
of `docs/architecture/ime-inline-hint-preflight-2026-10-20.md`.
Harness2 review task `20261021T130834Z-bf270816` remains running; intermediate
findings are not a final report. Historical browser totals/full-source
stack-overflow failure are unchanged. No local installation, push or release.

# 2026-10-21 19:43 UTC — replay-orphan-v26, source gate only

Keep the v23 ordinary Kitty command repair and user windows. The active task
is the Chrome after-Space Double Shift failure. Native v24/v25 establishes
client Reset/FO/FI and stock IBus1.5.26 duplicate engine cycles, not window focus
theft. Corrected no-surrounding/enabled-IME regression is RED before a bounded
orphan-marker retirement, then GREEN; 13 selected isolated guards PASS. This
does not establish full replacement or changed-byte native acceptance. Running
engine remains PID326051 SHA256 cf76bc925c96a738e2b4a9a4a37619713e0efb3d544b54d4d9b75f56337604b0.
Next: pinned development build, scoped engine deployment, real Chrome full
roundtrip and preserved terminal controls. Do not repeat noncausal SSI fixtures
or weaken field authority. Harness2 narrowed review task
20261021T193939Z-13826323 is running; preceding design task timed out and is
not an accepted review. All evidence/limits are in the owning preflight entry.


## 2026-10-21T20:05:39.601697+00:00 — v26 native partial; v29 causal pending-literal repair

Remote development engine709815/SHA2dd9972b23ec0f555cc9965f9bb18320e76fa4940fcd7b61c11cb888283cd349 was installed19:48UTC; daemon49153, IBus, tray, configuration and user windows preserved. V26 native Chrome omnibox: heldShift4/4, hintTab, seven ordinaryBackspaces, active-word roundtrip and active-five PASS. After-Space first replacement now PASS; inverse and Space-five FAIL. This is seven bounded rows, not the full browser matrix. Chrome prefix underline remains visualFAIL.

V27/v28 independently paired private-marker wire/observer traces: observer processing follows wire receipt within approximately0.1ms; delay precedes wire delivery. V27 has approximately100ms marker delay beyond the unchanged50ms acquisition deadline. V28 last acquisition finishes within its deadline, but its first g/h arrive with authenticated pending_literal stamps, are committed rather than kept in preedit, and are then lost from the local mirror at Ready. Native text is whole; the next manual request owns only its suffix. Delayed versus incomplete acquisition are separate remaining mechanisms.

V29 causal source RED holds the real marker after a real context reply, sets caps41 with no SSI and enabled IME. The first letter is wrongly committed (composition empty vs p). Minimal repair removes only the live-token condition from literal-preedit eligibility; the existing request/nonce/revocation/focus receipt still governs preservation at Ready. No boundary, client-text authority, delivery route or timeout is changed. Initial isolated affected gate17/17PASS; added explicit no-CommitText/no-delete assertions are being checked before build. New-byte v29 native NOT_TESTED. Harness2 task20261021T200308Z-22f7cc91 is running; no approval inferred.

V26 Harness2 task20261021T193939Z-13826323 completed8/8calls, no failure/fallback, bounded review9/10. Independently inspected marker identity cleanup and13source checks. Frozen review covered the positive fixture; later foreign-path/lease controls were independently tested outside that snapshot. Native v26 is partial, not full acceptance. Receipts: replay-orphan-build-v26/result.json, replay-orphan-deployment-v26/result.json, replay-orphan-native-v26/chrome/result.json and replay-orphan-native-v28/chrome-wire-trace/{result.json,engine-trace.jsonl,ibus-engine.log} under the remote kde-install-control-20261020 directory. Local causal logs pending-stock-literal-{red,green}-v29.log under shift-hint-20261021T1242Z. Full historical browser100PASS/76FAIL/144NOT_TESTED of320 remains unchanged. No release, push, local install or global restart.

## 2026-10-21T20:22:19.503452+00:00 — pending literal v29 native result

Remote development engine789121/SHA1477678ecceff657b788c18cad1158b7d4b183578e6da7dde3d3401ca9a04e6f installed at2026-10-21T20:12:20.184737Z. Daemon49153, stock IBus, tray, configuration and user windows preserved. Source gate17/17PASS, compiler artifact resolved from Cargo JSON; source closure1258files/SHA c921a2880b257dedc776f139016686d7b86f7202a551e991c77232db0d17593c. Remote receipts pending-stock-literal-build-v29b/result.json and pending-stock-literal-deployment-v29/result.json. No release/push/local install.

Native Chrome v29 seven-row probe:5PASS/2FAIL. Held Shift RU/US left/right4/4, hint/Tab, seven ordinary Backspaces, active-word Double Shift roundtrip and active-five PASS. After-Space roundtrip and after-Space-five FAIL. Active observed target timings12.275–12.669ms; a successful Space target251.699ms. This small probe establishes neither an overall latency percentile nor full browser acceptance. Receipt pending-stock-literal-native-v29/chrome/result.json.

V30 independent private-bus trace identifies the next first failure: acquisition131/nonce168 Get starts1792613610246622us, marker emitted0248318 and received0288342 (~40.024ms, inside the unchanged50ms deadline). All six replacement letters remain in local preedit after v29, then Space commits the whole word before Ready. Ready clears the committed local mirror; the inverse manual request sees tail0. No X11 focus change to an IBus window occurs. Receipt pending-stock-literal-native-v30/chrome-wire-trace/{result.json,engine-trace.jsonl,ibus-engine.log,x11-focus.log}. Debug configuration restored; engine bytes unchanged. A separate earlier acquisition exceeded50ms; increasing the deadline alone cannot repair this within-budget loss.

Harness2USED task20261021T200308Z-22f7cc91, gpt-6.1-sol/xhigh,8/8callsOK, no failure/fallback,9/10 bounded static review. Independently inspected report, selected artifact and current receipt fences; later no-CommitText/no-delete assertions separately passed17/17 outside frozen snapshot. Independent acceptance under local shift-hint-20261021T1242Z/pending-stock-literal-review-v29/independent-acceptance.json. Native remains partial FAIL despite static approval.

Next unresolved mechanism: preserve only new literal input consumed within the exact pending acquisition across a closing Space; do not resurrect revoked owner/client text, add a delivery route, fake a boundary, or increase timeout as a substitute. Production patch not yet selected. Chrome prefix underline visualFAIL and Firefox iframe overlap visualFAIL remain. Full historical browser100PASS/76FAIL/144NOT_TESTED of320 unchanged; no new GNOME or full matrix acceptance.


## 2026-10-21T20:42:40.139736+00:00 — pending literal Space v31

Context-admission source gate461/461PASS; remote engine878924 SHA716232d39e3b419982e0497c1e0702d670c691485ff3ac428e70815827fa2cda. Original native Chrome5PASS/2FAIL, still after-Space manual failures; no overall acceptance. Debug-enabled v32 single Space roundtrip PASS is timing-altered evidence only. Next: passive no-debug v33 wire failure, then affected native Firefox/ordinary Kitty/Konsole. Protected10 processes/config unchanged. Harness2 task20261021T203236Z-62aa6552 running. Owning record: docs/architecture/ime-inline-hint-preflight-2026-10-20.md CURRENT; receipts pending-literal-space-build-v31, pending-literal-space-deployment-v31, pending-literal-space-native-v31. No release/push/local install; retain prior fixes and active sessions.

## 2026-10-21T21:22:21.234437+00:00 — v35 current continuation

Remote enginePID1015535/SHA1ab291b; 468/468 source PASS. Native Chrome omnibox5PASS/2FAIL after Space; short Firefox21/28 and Chrome21/28 PASS. Kitty/Konsole hint/Tab and active/Space five-toggle PASS. Ordinary uppercase Shift PASS in all four; RU punctuation oracle unverified. Browser visual FAIL. No daemon/IBus restart. Next: full passive IBus wire around failed inverse; preserve exact owner/reducer/WordScope contracts. See owning preflight entry and v35 receipts. H2v34 completed degraded, selected6/10; eviction defect fixed in v35, final follow-up pending. Historical full matrix unchanged100PASS/76FAIL/144NOT_TESTED.


## 2026-10-21T22:22:46.914117+00:00 — v39 native delivery preserved; inverse and pixels still FAIL

Installed remote engine PID1208224 SHA f36012e4874026f390f814dac563de2802a3a1226c9651be1530f281a3c68bce. Guarded source gate450/450PASS. No daemon/IBus/tray restart;10 protected processes/config unchanged. Existing descriptor delivery ordinal + explicit delivery lease retain native replacement across real Reset/refocus; old word token remains revoked. Omnibox first after-Space target now complete,233.512ms; inverse stillFAIL. Short suites: omnibox5PASS/2FAIL; Firefox21/7; Chrome20/8. Owned ordinary Kitty/Konsole hint,Tab,5xactive,5xSpace and four Shift uppercase casesPASS; Russian punctuation oracle needs real no-IME control. Old user Kitty without GLFW_IM_MODULE preserved,not certified. ScreenshotsFAIL: Chrome typed prefix underline; Firefox iframe prefix missing visually. Full320 matrixNOT_RERUN. Harness2USED20261021T214419Z-cb269f86,8calls0failed/fallback; accepted frozen pre-v39 design only,not patch/native. Owning record: docs/architecture/ime-inline-hint-preflight-2026-10-20.md CURRENT; receipts remote pending-literal-space-{build,deployment,native}-v39; local native-replay-reset-design-v38/independent-acceptance.json. Next: true no-IME punctuation control, fresh exact manual source/settlement failure, and coherent typed-prefix presentation. No push/release/local install.

## 2026-10-21T22:46:10.614072+00:00 — plain-prefix scope and Shift oracle

User accepts Kitty only; Chrome typed-prefix underline and LibreOffice presentation remain open. v39 engine f36012e4 remains unchanged. v41 physical-keycode Shift tests6/6 in each of new Kitty/Konsole windows; old RU punctuation mismatch also occurs in detached no-IME Kitty control, so old RFB oracle is invalid for attributing it. v42 existing-profile run BLOCKED with zero input because old Chrome/Kitty processes are gone. Do not retry stale PIDs. LibreOffice1285267 is GTK3 VCL. Matching Chrome155 primary source shows NONE/default native composition underlines and missing surrounding retrieval; neither removing attrs alone nor reintroducing unaccepted hints into preedit is a safe fix. Native actual-source probe v43 is in progress; Harness2 design task20261021T224439Z-58fdfa4b running (not accepted yet). Decision2026-10-21-plain-prefix-observation-probe.json permits diagnosis only. Do not implement a new owner or fabricate SSI. Owning document: docs/architecture/ime-inline-hint-preflight-2026-10-20.md.

## 2026-10-21T23:04:20.091127+00:00 — LibreOffice actual-surrounding prefix candidate

v39 f36012e4 remains installed; native LO v49 Tab PASS but prefix underline FAIL, with10 actual surrounding callbacks and7 restorations. Chrome v43–v46 accessibility source qualification BLOCKED before text input; no provider adopted. Source regression stock-observed-v50-red confirms actual stock boundary unnecessarily starts owned preedit. Minimal candidate uses existing ordinary ManagedCommit for that observed stock branch, keeps explicit exact-refresh/opaque paths and all freshness/verifier guards. Source/native checks pending. Kitty acceptance retained. See owning preflight CURRENT and decision2026-10-21-stock-observed-ordinary-prefix.json.

### 2026-10-22T04:39:00.577165+00:00 — v61 installed, physical inverse still FAIL

Harness2 `20261022T040352Z-3055d027`: 8/10,8/8 calls; independent source464/464/architecture/diff PASS. Remote engine2468001 SHA`c98f3682f7c08a45858a675b47c1562d41d5a3ba3515578afbf1405584b4e433`. Writer native5PASS/1FAIL: first closed flip186.222ms PASS, inverse FAIL, seven restorations. Full owning-doc entry above records first lost source transition and receipts. Next exact replay reproduction, no repeated publication of source PASS as client PASS. Chrome prefix underline FAIL; no full browser rerun. PRs5open read-only checked04:21:03UTC; rule in AGENTS.

## 2026-10-22T04:55:30.519513+00:00 — v62 causal settlement identity RED

Correction to the earlier shortened trace: actual target-engine SSI during replay includes the delayed old deletion prefix `п`, empty text and partial target `gh`; the first **whole** new word `ghbdtn` arrives after cleanup. The previous no-intermediate-SSI and wrong-fixture diagnostics are setup diagnostics, not product proof. Current actual-order fixture reproduces the exact native pending read: unknown state, empty text, advanced epoch, same engine path, **empty field identity**. Receipt `stock-replay-v62-settlement-actual-order-red/result.json`. X11 settlement checks the exact field before waiting, rejects this answer immediately, and Drop sends the observed CancelExactManualToggleSuppressionV2 before the whole client observation. Unknown itself is an intended quarantine.

Minimal candidate: preserve only the exact field identity under a complete paired live replay provenance and live authenticated bridge. Continue returning unknown source and no text; expiry, mismatched pair or revoked field cannot retain identity. No edit/receipt/ACK, cancellation, timeout, route or executor change. Decision `docs/architecture/decisions/2026-10-22-native-replay-settlement-identity.json`. Source/review/native still pending; installed engine remains v61 SHA c98f3682f7c08a45858a675b47c1562d41d5a3ba3515578afbf1405584b4e433. Harness2 diagnosis task `20261022T044902Z-19387e9d` running; its packet says first actual SSI and must be qualified to first whole-target SSI on independent acceptance.

Upstream PR review refreshed at 2026-10-22T04:54:12.000596+00:00: five open #64-68, no mutations. Receipt `upstream-pr-review-20261022T045412Z.json`.

## 2026-10-22T05:05:44.898970+00:00 — v62 identity-only source proof

The actual-order causal RED is now GREEN with the minimal VisibleTailV4 identity-only candidate. The same test covers empty/Unicode prefixes and rejects expired, mismatched local/shared, missing-source-provenance and revoked shared-path cases; unknown state, empty text, no retention and no ACK remain required. Fixed affected proof465/465 PASS, architecture/diff PASS, wall162.145s; receipt `/home/ubu/.cache/lay/development/browser-final-fixes-20261018/shift-hint-20261021T1242Z/stock-replay-v62-settlement-source-gate/result.json`. Graph was updated by the same remote gate. Candidate build running; installed bytes still v61 c98f3682f7c08a45858a675b47c1562d41d5a3ba3515578afbf1405584b4e433. This is source acceptance only; final review and physical Writer inverse remain pending. No route/executor/daemon/IBus/config authority change.

## 2026-10-22T05:21:06.822279+00:00 — combined settlement candidate source proof

The second causal RED established that an actual whole target without its final Space was eligible for refresh but emitted no RequireSurroundingText. The candidate now requests a real SSI through the existing output adapter only for changed eligible actual observations under current completed paired provenance. It adds no state, deadline, ACK, inferred snapshot or edit route. Focused proof PASS (2 prefixes x 8 live/refusal cases), including no repeated edit, no fabricated ACK, duplicate-read loop refusal and settlement only after the actual final SSI. Receipt `/home/ubu/.cache/lay/development/browser-final-fixes-20261018/shift-hint-20261021T1242Z/stock-replay-v63-refresh-guarded-green-r3/result.json`; original RED `/home/ubu/.cache/lay/development/browser-final-fixes-20261018/shift-hint-20261021T1242Z/stock-replay-v63-refresh-actual-receipt-red/result.json`. The extra raw layout/owner/atomic fixture assertions were corrected as an out-of-scope oracle assumption, with the failing fixture and explanation retained at `/home/ubu/.cache/lay/development/browser-final-fixes-20261018/shift-hint-20261021T1242Z/stock-replay-v63-test-oracle-scope.md`; existing canonical negative tests are unchanged. Affected 465-case proof, final code review and native Writer inverse remain pending. Installed engine remains v61 c98f3682f7c08a45858a675b47c1562d41d5a3ba3515578afbf1405584b4e433. Runtime authority unchanged. Chrome typed-prefix underline remains FAIL.

## 2026-10-22T05:28:52.930583+00:00 — independent consumer audit blocks v63 installation

Combined v63 source465/465, architecture and release build PASS, but NOT INSTALLED. Independent audit of the actual Writer wire found initial V4 policy true and pending unknown reply false. `ImeDelegatedTailLease::bound_v4_reply` rejects this synthetic policy change before field settlement. Thus the current v63 candidate is not sufficient, even if its frozen source review approves it. Added a causal assertion matching the actual lease policy; RED run underway. Required minimal fix: under the same fully proven pending field, return the current actual personalization policy while text remains empty and no ACK/learning action exists; all unproved contexts remain denied. Installed runtime remains v61. Existing PR read-only review at05:23:32UTC: five open #64–68.


### 2026-10-22T05:33:57.502125+00:00 — v64 actual consumer-policy RED/GREEN; source/native pending

- Actual Writer V4 starts with personalization/retention=true, while the pending
  unknown reply returned false. Existing daemon `bound_v4_reply` rejects this
  before identity/settlement; v63 source/build PASS did not close this path.
- A causal assertion failed false vs true before the production change, then
  passed after the same pending proof gates retained the actual content policy.
  No text, learning action or replay ACK is exposed; unproved/sensitive contexts
  remain denied. Exact replay consumer and its policy-change rejection unchanged.
- Focused proof: 1 test / 16 scenarios PASS, including actual interleaved native
  ordering, duplicate refusal and actual final settlement. RED: `/home/ubu/.cache/lay/development/browser-final-fixes-20261018/shift-hint-20261021T1242Z/stock-replay-v64-policy-actual-lease-red-result.json`;
  GREEN: `/home/ubu/.cache/lay/development/browser-final-fixes-20261018/shift-hint-20261021T1242Z/stock-replay-v64-policy-actual-lease-green-result.json`.
- Runtime authority unchanged. Installed engine remains v61; v63 is not installable.
  v64 affected source gate, review, build and native client acceptance pending.
- GitHub PR check 2026-10-22T05:33:57.502125+00:00: 5 open, #64 arboard, #65 serde_json, #66 sha2,
  #67 rayon, #68 zbus; read-only, no merge/close/update. Receipt:
  `/home/ubu/.cache/lay/development/browser-final-fixes-20261018/shift-hint-20261021T1242Z/upstream-pr-review-20261022-current.json`.


### 2026-10-22T05:48:33.209266+00:00 — v64 source/build PASS; final bounded review pending

- v64 affected proof: 465/465 PASS, architecture receipt and diff checks PASS,
  wall 163.356 s. Build 82.345 s, candidate SHA-256
  `9c789fe107c660a778dea1324f6f9a4ac100c63ad0e68ab99c7cb51ce7ee6fe0`.
  Receipts: `/home/ubu/.cache/lay/development/browser-final-fixes-20261018/shift-hint-20261021T1242Z/stock-replay-v64-settlement-source-gate-result.json` and
  `/home/ubu/.cache/lay/development/browser-final-fixes-20261018/shift-hint-20261021T1242Z/stock-replay-v64-build-result.json`. Installed bytes remain v61.
- Final v63 Harness2 report (20261022T052243Z-ae6066b2) was ok, 8/8 calls,
  0 failed, score 7/10; unlike the preliminary draft it flags the policy-consumer
  mismatch. It is SUPERSEDED_NOT_INSTALLABLE. Independent actual wire establishes
  initial true versus pending false; source construction directly preserves
  the actual V4 bit into Some(permission) then the lease. Evidence:
  `/home/ubu/.cache/lay/development/browser-final-fixes-20261018/shift-hint-20261021T1242Z/stock-replay-v64-consumer-policy-independent.json`.
- Final bounded v64 review RUNNING: lay2 / gpt-6.1-sol / xhigh, task
  `20261022T054215Z-8c4ebd23`; source/native acceptance not inferred from doctor.
- One owned actual HTML input in Chrome, no force-accessibility flag: empty,
  active `пров` (caret 4), committed `пров ` (caret 5), no selection, same actual
  focused editable object/PID observed through AT-SPI; 7 restorations PASS.
  Read-only source diagnostic PASS, not a product fix; screenshot still shows
  typed-prefix underline (visual FAIL). No source adapter or runtime authority
  was added. Receipt `/home/ubu/.cache/lay/development/browser-final-fixes-20261018/shift-hint-20261021T1242Z/chrome-active-field-source-v64-probe/result.json`.
- Fresh upstream triage 2026-10-22T05:47:26.917090Z: 5 open; #65 serde_json
  and #68 zbus CI clean; #64 arboard installer/tray FAIL, #66 sha2 MSRV/tray
  FAIL, #67 rayon tray FAIL. No PR mutation. Receipt
  `/home/ubu/.cache/lay/development/browser-final-fixes-20261018/shift-hint-20261021T1242Z/upstream-pr-status-20261022-current.json`.
- Next: independently accept exact v64 review bytes, remote engine-only
  deployment, 6 short Writer rows and Kitty/Chrome controls. No daemon, global
  IBus, tray, desktop or user configuration restart/change. Browser matrix remains
  Firefox 92/68/0/0 of160; Chrome 8/8/0/144 of160.


### 2026-10-22T06:13:38.506100+00:00 — v64 installed; Writer inverse after Space remains FAIL

- Final Harness2 review USED: lay2 / gpt-6.1-sol / xhigh, task `20261022T054215Z-8c4ebd23`,
  score 9/10, ok, 8 calls, 0 failed, no fallback. Frozen/current input
  identities independently verified; source-only acceptance. Report `/home/ubu/.local/state/harness2-codex/projects/lay2/tasks/20261022T054215Z-8c4ebd23/report.json`.
- Remote engine-only deployment completed at 2026-10-22T06:05:37.471367+00:00; PID 2765777,
  SHA-256 `9c789fe107c660a778dea1324f6f9a4ac100c63ad0e68ab99c7cb51ce7ee6fe0`. Daemon/global IBus/tray/desktop and user config unchanged.
- Native Writer: **5 PASS / 1 FAIL**, all 7 restoration checks PASS. Hint/Tab in first
  and second word, active-word Double Shift, ordinary held Shift and Space flip PASS.
  Closed-word first flip 194.099 ms PASS; inverse stays `ghbdtn ` FAIL.
  Active roundtrip 185.766 / 170.433 ms. Canonical private-policy consumer test 1/1 PASS.
- Actual trace now retains same field and true policy through pending replay. The
  last surrounding callback is `ghbdtn` at 695.423 ms, without the visible final
  Space. RequireSurroundingText emitted at 696.004 ms; no later SSI before cancellation
  at 771.216 ms. The test's supplied final `ghbdtn ` callback is not native evidence.
  Remaining first transition: client final snapshot, not pending identity/policy.
- Controls: Kitty hint/Tab and Double Shift roundtrip PASS; Chrome one HTML-input
  Tab functional PASS. These short checks do not replace the browser matrix.
  Chrome typed-prefix underline remains separate visual FAIL.
- Receipts `/home/ubu/.cache/lay/development/browser-final-fixes-20261018/shift-hint-20261021T1242Z/stock-replay-v64-native-acceptance.json`,
  `/home/ubu/.cache/lay/development/browser-final-fixes-20261018/shift-hint-20261021T1242Z/libreoffice-no-capitalization-v64/result.json`,
  `/home/ubu/.cache/lay/development/browser-final-fixes-20261018/shift-hint-20261021T1242Z/stock-replay-v64-native-controls-summary.json`; timelines in the Writer folder.
- Next: inspect installed IBus 1.5.26/GTK3 3.24.33 client request semantics before
  another candidate. No fabricated final snapshot/ACK, blind timeout extension,
  or route/owner bypass. Runtime authority unchanged.


### 2026-10-22T06:22:33.646337+00:00 — current PR and visual check; next narrow Harness2 diagnosis

- GitHub read-only check at 2026-10-22T06:14:01.896471+00:00: five open #64–68. #65 serde_json
  and #68 zbus CI clean; #64 arboard installer/tray FAIL, #66 sha2 MSRV/tray FAIL,
  #67 rayon tray FAIL. Nothing merged, closed, posted or upgraded. Regular-check
  rule remains in AGENTS.md and global code-tools.md; no unattended timer claimed.
- v64 actual screenshots inspected: Writer one hint and ordinary-launcher Kitty
  one hint render entered prefix normally and suffix black on white. Chrome
  HTML input still underlines entered prefix: visual FAIL. Functional Tab PASS
  does not certify appearance. Receipt `/home/ubu/.cache/lay/development/browser-final-fixes-20261018/shift-hint-20261021T1242Z/stock-replay-v64-native-visual-acceptance.json`.
- Primary installed-matching IBus1.5.26 code confirms RequireSurroundingText only
  sets needs_surrounding_text, GTK requests before key processing and after
  CommitText callback. Writer final boundary is not observed after native key.
- Harness2 USED diagnosis task `20261022T061922Z-1ad50116` RUNNING, lay2 /
  gpt-6.1-sol / xhigh, frozen packet `/home/ubu/.cache/lay/development/browser-final-fixes-20261018/shift-hint-20261021T1242Z/stock-replay-v65-final-character-design-review`.
  Compare final literal key commit in existing exact replay, no-op/request and
  generic actual-source adapter. No production change from this proposal yet.


## 2026-10-22T09:34:30.248627+00:00 — v65 native readback hypothesis failed

- Source affected gate466/466 PASS; exact build SHA256 `092198d7d7705e572ba7b259b35d57888e24ed2caabecba2a870e8070b884528`. Harness2 `20261022T090301Z-6e694fc2`: ok,8calls,0failed,no fallback; independent source review9/10, native not certified. Canonical-format-only testfile equivalence is explicitly retained in the independent review packet.
- Installed only the remote engine PID3353802; daemon/tray/global IBus/desktop unchanged. Deployment receipt `/home/e/projects/lay-development-runner/kde-install-control-20261020/stock-replay-v65-deployment/result.json`. No publication/local installation.
- Actual owned Writer6rows:5PASS/1FAIL. Closed-word first toggle149.216ms PASS, inverse FAIL. Active toggles133.397/136.580ms. Short controls: Kitty hint/Tab PASS, Konsole hint/Tab PASS, Chrome Tab PASS, Firefox Space roundtrip FAIL. Visual acceptance remains separate; no full browser matrix rerun.
- Writer bus trace proves v65 branch ran: final Space CommitText at relative592.813ms, client delivery593.638ms, followed by actual SSI `ghbdtn` caret6 at595.127ms. No final matching `ghbdtn ` SSI or ACK. The proposal that this commit representation forces final fresh SSI is rejected natively. Source PASS cannot override this result.
- Native receipts `/home/e/projects/lay-development-runner/kde-install-control-20261020/libreoffice-no-capitalization-v65/result.json` and `stock-replay-v65-native-controls/summary.json` beside it. All recorded restoration checks true.
- Next: one read-only GTK surrounding trace in a newly owned Writer window to distinguish Writer getter data from IBus caching. No extra retries, timer, fabricated SSI, owner or client-specific permission. v65 remains installed pending that observation. Chrome typed-prefix underline remains visual FAIL; no Chrome production change adopted.
- Read-only upstream review at2026-10-22T09:17:25.177798+00:00:5open PRs #64,#65,#66,#67,#68; no merge/comment/closure.


## 2026-10-22T09:38:13.025113+00:00 — v65 rejected for native ordering; bounded rollback

One owned Writer GTK trace reproduced `ghbdt n` instead of `ghbdtn ` on the first closed-word toggle. The last native letter was still pending when v65 CommitText Space reached the client; final-key hybrid delivery is not ordered with earlier GTK native fallthrough. This is a native rejection of v65 despite466 source PASS and9/10 static review. Receipt `/home/e/projects/lay-development-runner/kde-install-control-20261020/libreoffice-no-capitalization-v66-gtk-trace/result.json`, actual GTK source log beside it;94 getter observations, no fake SSI. Probe was read-only and loaded only in its own Writer process; all7 restoration checks true. Rejected mechanism: do not retry final-key CommitText, extend timers or loosen ACK. Restore exact saved v64 engine SHA256 `9c789fe107c660a778dea1324f6f9a4ac100c63ad0e68ab99c7cb51ce7ee6fe0`, engine-only, preserve desktop/daemon/tray. Then same one-row read-only probe on v64 to locate original missing boundary. This is an explicit causal rejection, not an undocumented revert.


## 2026-10-22T10:06:07.869874+00:00 — Writer source clipping established; rejected replay removed

Exact v64 engine restored at 09:38:15.076429 UTC, PID3402648, SHA256 `9c789fe107c660a778dea1324f6f9a4ac100c63ad0e68ab99c7cb51ce7ee6fe0`. Nine protected processes and user config unchanged; engine-only rollback receipt `/home/ubu/.cache/lay/development/browser-final-fixes-20261018/shift-hint-20261021T1242Z/stock-replay-v65-rollback-native-ordering/result.json`. The rejected22production lines are removed locally; test cleanup preserves the original native final-key order.

Read-only owned Writer v67 on v64: first closed-word toggle PASS145.359ms; inverse FAIL. All7 restoration checks true. Native document contains `ghbdtn ` at caret7, but98 actual `gtk_im_context_set_surrounding` calls return only `ghbdtn` at caret6 through the inverse gesture. Installed-matching LibreOffice7.3.7.2 source locates the cause: Writer `GetSurroundingText` uses sentence boundaries (edtwin.cxx6428–6459), cursor is separately calculated (6461–6499), VCL clamps it to string length (winproc.cxx2257–2300), GTK forwards it unchanged (gtkframe.cxx5699–5718). This is stronger than a timing-only hypothesis. Receipts `/home/ubu/.cache/lay/development/browser-final-fixes-20261018/shift-hint-20261021T1242Z/libreoffice-no-capitalization-v67-v64-gtk-trace/`; primary source copies `/home/ubu/.cache/lay/development/browser-final-fixes-20261018/shift-hint-20261021T1242Z/lo-7.3.7.2-sw_source_uibase_docvw_edtwin.cxx` and `lo-7.3.7.2-vcl_source_window_winproc.cxx`. No fabricated SSI/ACK, producer bypass or new authority.

Restore source gate v69:465PASS/1FAIL in newly added regression expectation for expired replay. Its final key may be consumed with no engine output; source policy unchanged. Native-order regression requires pass-through for current provenance and zero CommitText/DeleteSurroundingText/ForwardKeyEvent for every tested invalidation, no ACK and no invalid final provenance. Existing negative receipt tests retained. Full affected466proof pending after correction.

Harness2 task `20261022T094446Z-6a30e4b1`: base-failed, one240.115s timeout, no deliverable, not an accepted review. Narrower17043byte frozen packet rerun as `20261022T100101Z-fca5bf95`, lay2/gpt-6.1-sol/xhigh, per-run config600s, shared settings unchanged. Review is RUNNING, no result claimed. Next decision: whether a bounded actual full-field source can bind focused native field to current IBus owner without inventing acknowledgement. Chrome typed-prefix underline remains visual FAIL; Kitty accepted behavior preserved. Full browser matrix unchanged and not rerun. No push, release or local installation.


## 2026-10-22T10:51:05.807040+00:00 — restored source green; actual-word ordering candidate under review

v73 source466/466PASS, architecturePASS, native unchanged: Writer5PASS/1FAIL(inverse afterSpace), Kitty controlsPASS, Chrome Tab functionalPASS/prefix-underline visualFAIL. Remote enginePID3402648 remains exactv64SHA2569c789fe107c660a778dea1324f6f9a4ac100c63ad0e68ab99c7cb51ce7ee6fe0;9protected processes/config and4local/remote source hashes reverified10:50:24UTC. v70 H2 diagnosis accepted; exact field join/delete coordinates remain blockers. New v74 H2 task20261022T103628Z-e7cc3589 RUNNING, fullcanon included: review one owned finalSpace held until actual whole-word SSI, existing reducer only, no ACK/learning/newowner. New native prerequisite is not yet proved. Source/receipts/limits in owning `docs/architecture/ime-inline-hint-preflight-2026-10-20.md`, same timestamp; next inspect v74 result, then bounded prerequisite experiment before implementation. Browser matrix unchanged/notrerun FF92/68/0/0,Chrome8/8/0/144 of160each. Public PR page rechecked5open#64–68; no mutations. No new production edit, installation, restart orpush.


### 2026-10-22T11:34:49.287336+00:00 — Writer final-Space prerequisite observed

Continue only in `/home/ubu/projects/lay-browser-final-fixes-20261018`. Installed remote v64 PID3719097 restored, SHA9c789fe107c660a778dea1324f6f9a4ac100c63ad0e68ab99c7cb51ce7ee6fe0. Harness2 task20261022T103628Z-e7cc3589 reviewed conditional design, not finished fix. v77 engine-held native Space produced actual whole-word SSI; all protected state restored. v78 new real-path contract is RED0/2; production implementation pending. Source baseline v73 466/466. Exact evidence, authority limits and next step: owning `docs/architecture/ime-inline-hint-preflight-2026-10-20.md`, section `2026-10-22T11:34:49.287336+00:00`. Browser matrix unchanged; Writer inverse and Chrome prefix underline remain FAIL. No push/local install/daemon or desktop restart authorized.


## 2026-10-22T12:18:05.261283+00:00 — Writer held boundary current checkpoint

v64 runtime unchanged PID3719097/SHA9c789fe107c660a778dea1324f6f9a4ac100c63ad0e68ab99c7cb51ce7ee6fe0. v85 token and v87 field-identity TDD fixes implemented; current source provenance restricts held Space to ObservedWordAndLocalSpace, exactly one trailing boundary. Existing full-snapshot native delivery restored; v88b20/22 with only two expected locally sourced test migrations. v91 migrated cancellation/identity test PASS. Affected470 gate v92 RUNNING. H2 review20261022T115225Z-f7d6b553 RUNNING on earlier v83 packet; final refinements require independent review. Next graph/source linkage, compiler-artifact build and short Writer7-row native test. Full evidence/limits in owning preflight document at the same timestamp. No new installed/client PASS, push or local installation.


### 2026-10-22T12:20:01.188559+00:00 — affected source gate green

Writer held-boundary v92:470/470 affected source checks PASS,82.963s; source receipt remote `/home/e/projects/lay-development-runner/kde-install-control-20261020/writer-held-boundary-v92-affected/result.json`. All16 full-snapshot native cases remain unchanged. Final actual-word callback now emits only one retained Space, no result ACK, including paired/provenance/token/field/cancellation controls. Native Writer remains NOT_TESTED for these bytes. Next generated graph/architecture refresh records all source-file hashes and may change only `src/generated/architecture_graph_receipt.json`; build must link this receipt explicitly to the passed proof, not claim identical source hashes.


### 2026-10-22T12:30:17.946391+00:00 — Writer v92: bounded native acceptance, review follow-up open

- Remote diagnostic engine `457c02b4f9707a85415138ac550574720d49a424dae9e6b0707727198946100b`, linked to source closure `75d6d4071f38be83503072d54059396ba6c76723c764aa7f17bfbefc16a4dab0`: affected source 470/470 PASS. Graph link changes only the generated architecture receipt.
- Actual owned remote KDE Writer, isolated no-capitalization profile: 7/7 PASS (first/second-word hint+Tab, active/closed Double Shift roundtrip, five consecutive closed flips, ordinary held Shift, Space autoflip). All nine conversions assert exact document text, caret at end, no selection and owned focus. Median 144.966 ms, maximum 257.007 ms, n=9, from second Shift release to observed target. This is not a full Writer visual/client-profile proof.
- Native receipt: `/home/ubu/.cache/lay/development/browser-final-fixes-20261018/shift-hint-20261021T1242Z/writer-held-boundary-v92-native/result.json`; detailed client: `/home/ubu/.cache/lay/development/browser-final-fixes-20261018/shift-hint-20261021T1242Z/libreoffice-no-capitalization-v92-held-boundary/result.json`. Both client 7/7 and runner 9/9 restoration checks true. Previous v64 bytes restored at 2026-10-22T12:25:48.867505+00:00, engine PID3926674; daemon/tray/IBus/desktop/Kitty preserved. Runtime authority and installed release unchanged; no publication.
- Harness2 implementation review task `20261022T115225Z-f7d6b553` still running on frozen v83. Earlier token/field findings already addressed before v92. New review hypothesis: a complete-word SSI with a caret one character behind may survive generic inert-prefix tolerance while a Space is actionable. Next: reproduce this exact cancellation case in a source test before a minimal fix; do not weaken generic replay guards. Current v92 native proof remains scoped to its exact tested bytes.
- Browser table remains Firefox92/68/0/0 of160; Chrome8/8/0/144 of160. Chrome prefix underline still FAIL; no browser rerun or inferred PASS.


### 2026-10-22T12:42:06.405147+00:00 — actionable replay boundary: caret contradiction RED→GREEN

- Harness2 task `20261022T115225Z-f7d6b553` finished `ok`: eight gpt-6.1-sol/xhigh calls, zero failed calls/fallbacks, source files unchanged. Its frozen v83 review scores 6/10 REQUEST_CHANGES; current-token and pending field/privacy findings were already fixed in v85/v87. This is not a review score for current source. Final report: `/home/ubu/.local/state/harness2-codex/projects/lay2/tasks/20261022T115225Z-f7d6b553/report.json`.
- Independently reproduced the additional caret cancellation finding: the generic inert historical-prefix predicate accepts a one-character caret lag, which must not retain an actionable held Space. Actual RED `writer-held-boundary-v93b-caret-red/result.json` (0/1) fails at non-end caret. The earlier v93 invocation selected an incorrect test path and ran no tests; it is a harness-selection error, not product RED.
- Minimal current fix: while a final Space is held, selection or any non-end caret cancels before historical-prefix tolerance. Generic inert native-replay tolerance and delivery remain unchanged. Regression covers partial, complete and foreign snapshots under empty/Unicode prefixes; late exact SSI cannot revive output, source authority, ACK or learning.
- v94 held boundary 5/5 PASS; full affected set 471/471 PASS in 83.721 s. Source closure b10a7f05ece5788c0efa5f9d481d2c3f85519630ac80764b20be0574b0fb1075. Remote evidence: `/home/e/projects/lay-development-runner/kde-install-control-20261020/writer-held-boundary-v94-affected/result.json`. Current bytes are not release-built or native-tested yet. Previous v92 native 7/7 remains valid only for its exact SHA457c02b4f9707a85415138ac550574720d49a424dae9e6b0707727198946100b.
- Next: graph/source linkage → compiler-artifact build → same seven owned Writer rows → independent wire/restoration audit → bounded final Harness2 follow-up. Engine-only diagnostics remain temporary; installed v64 PID3926674, config, daemon/tray/IBus/desktop/Kitty unchanged. No release/local install/push. Chrome typed-prefix underline remains FAIL. Browser matrix unchanged: Firefox92/68/0/0 of160; Chrome8/8/0/144 of160.


### 2026-10-22T12:53:20.557606+00:00 — v94 native 6/7: whole-word refresh absent

Source471/471 and compiled artifact5061dc0c000f15a15c123a7b1c102df00b440f74f02f5e760c85d70d76a7e1b0 were independently linked. Native Writer6/7 PASS; the fourth flip of the five-consecutive row loses final Space. Last actual SSI is “приве” caret5, followed by held Space/RequireSurroundingText; the whole word “привет” arrives only with a later test key, after cancellation. There is no non-end caret in this failure, so do not blame or revert the caret guard without evidence. Primary libibus RequireSurroundingText sets needs_surrounding_text only; current hold has an observation-progress gap. Previous v92 seven-row PASS remains bounded and is not repeat reliability proof. Exact receipts: `/home/ubu/.cache/lay/development/browser-final-fixes-20261018/shift-hint-20261021T1242Z/writer-held-boundary-v94-native/result.json` and `liveness-audit.json`. All client7/7 and runner9/9 restorations true; v64 restored PID3998187. No installed release, local install, daemon/IBus/tray/desktop restart or browser statistics change. Next bounded Harness2 review evaluates queue-preserving final-Space delivery and independent source authority; do not add delays, fake receipts or unrelated routes.


### 2026-10-22T13:13:39.118326+00:00 — v95b causal RED; ordered final-Space candidate pending checks

The original v95 test attempt hit a test peer-draining error, not the product defect; preserve it as harness error. Corrected v95b RED reproduces missing dispatch with actual partial SSI, zero forwarded keys versus the authorized Space press/release pair. Receipt: /home/e/projects/lay-development-runner/kde-install-control-20261020/writer-forward-boundary-v95b-red/result.json.

Existing v94 native remains6PASS/1FAIL; v64 restoredPID3998187. Harness2 liveness task20261022T125330Z-dca62ad9 first completed base review: current liveness6/10, native return7/10, conditional forward experiment8/10. Full aggregation still running; this does not review the new implementation.

Candidate moves only that retained Space pair to existing EngineOutput forward at the original callback; paired release is consumed, preceding native keys unchanged, no CommitText/retry. RAII covers request and both output awaits. Dispatch itself supplies no source, result ACK or learning; source-only completion additionally retains the genuine whole-word and all existing token/field/pair/caret/epoch/privacy checks. Source and actual native ordering remain unaccepted pending deterministic checks and short own Writer run. Decision: docs/architecture/decisions/2026-10-22-ordered-replay-boundary-forwarding.json. No permanent runtime adoption, release or local install. Browser totals unchanged; Chrome prefix underline remainsFAIL.


### 2026-10-22T13:24:39.060087+00:00 — ordered final boundary: delivery lease separated from source proof

v96's six held-boundary cases failed before output because the dispatch callback was incorrectly checked with the settled-source predicate. v97 keeps the existing reducer `revalidate(token)` delivery check while the current callback is outstanding; the stronger `exact_replay_source_token(token, epoch)` check still gates source settlement after actual complete-word SSI. No source guard was weakened. v97 focused6/6PASS; affected470/472PASS with two old delivery assertions rejecting the newly explicit ordered-key experiment. v98 migrates those two tests to require exactly one Space press/release only in the live paired branch, no text commit/deletion, no forwarded output in invalidated or release branches, no output from duplicate actual snapshots, and unchanged source/ACK/privacy checks. The old CommitText-on-SSI expectation is replaced by a strict no-text-output assertion. v98 affected gate is running; neither source nor native acceptance is claimed yet. Native helper now maps the existing read-only GTK surrounding/key-filter observer only into the owned Writer child, with original typing/gesture cadence unchanged. Remote accepted engine remains v64/PID3998187; no daemon/tray/IBus/desktop restart or local install/publication.

Receipts remote under `/home/e/projects/lay-development-runner/kde-install-control-20261020/`: `writer-forward-boundary-v96-held-green/result.json` (FAIL0/6), `writer-forward-boundary-v97-held-green/result.json` (PASS6/6), `writer-forward-boundary-v97-affected/result.json` (470PASS2FAIL), and pending `writer-forward-boundary-v98-affected/result.json`. Test migration preimage: `/home/ubu/.cache/lay/development/browser-final-fixes-20261018/shift-hint-20261021T1242Z/writer-forward-boundary-v98-test-migration-preimages/`. Harness2 task20261022T125330Z-dca62ad9 remains in progress on the prior v94 design packet, not current implementation certification.


### 2026-10-22T13:27:42.728814+00:00 — ordered boundary candidate v98 source closure green

472/472 affected source checks PASS; discovery13.638s, isolated execution82.883s. Source before graph `b481bbc4a9669c22abd63c3cac5a64226103e2d6ca7681f2605c614d30b5fe81`. Remote graphify/architecture update PASS, only source change `src/generated/architecture_graph_receipt.json`; compiler input closure `e43c68cca5e7ffd6c572af621dd9cbb552b05f5ad8d822ae9d824db11dea95b6`. Six generated graph/binding files synced back with preimages at `/home/ubu/.cache/lay/development/browser-final-fixes-20261018/shift-hint-20261021T1242Z/writer-forward-boundary-v98-graph-sync/`. Candidate build is running through remote guard; no native PASS or runtime adoption. Next exactly seven own Writer checks with unchanged key timing and read-only GTK order observer.


### 2026-10-22T13:37:33.728509+00:00 — v98 Writer source/native closure, final implementation review pending

Source472/472PASS, graph/architecturePASS, compiler input SHA e43c68cca5e7ffd6c572af621dd9cbb552b05f5ad8d822ae9d824db11dea95b6. Compiled engine SHA 56e7ac524b4228cfbc7a8f789655215ca8b79beee8935fa81004d90535ed44c4; build 70.596s, artifact resolved from compiler output. Own Writer7/7PASS: prefix/second-word hint+Tab, active and closed two flips, five closed flips, ordinary held Shift+A, Space auto-flip. Nine exact conversions with end-caret/no selection, median148.489ms/max219.919ms; this is a bounded client run, not a general speed or reliability claim.

Independent read-only GTK audit confirms all five closed conversions: actual whole-word/end-caret retrieval follows the last native letter and precedes delivery of the forwarded planned Space. Each client receives exactly one Space press/release pair. This is one observed schedule, not a universal FIFO or result-ACK proof. Dispatch still supplies no source, acknowledgement or learning; existing whole-word/token/field/pair/caret/epoch/privacy settlement guards remain mandatory.

Receipts: /home/ubu/.cache/lay/development/browser-final-fixes-20261018/shift-hint-20261021T1242Z/writer-forward-boundary-v98-build/result.json; /home/ubu/.cache/lay/development/browser-final-fixes-20261018/shift-hint-20261021T1242Z/writer-forward-boundary-v98-native/result.json; /home/ubu/.cache/lay/development/browser-final-fixes-20261018/shift-hint-20261021T1242Z/writer-forward-boundary-v98-native/ordering-audit.json. All native restoration9/9 and client restoration7/7 checks true. v64 SHA9c789fe107c660a778dea1324f6f9a4ac100c63ad0e68ab99c7cb51ce7ee6fe0 restored PID4140889 at 2026-10-22T13:28:21.088403+00:00; candidate not adopted.

Harness2 task20261022T125330Z-dca62ad9 completed ok: eight calls, zero failed, no fallback, frozen inputs/source unchanged. Selected review scores the prior v94 liveness design6/10 and denies its publication. Its request-await/drop concern is covered by current deterministic tests; this design packet does not certify v98 bytes. Acceptance: /home/ubu/.cache/lay/development/browser-final-fixes-20261018/shift-hint-20261021T1242Z/writer-boundary-liveness-v94-review/acceptance.json. Next bounded review covers final v98 implementation, followed by Chrome typed-prefix underline investigation.

Chrome typed-prefix underline remains visualFAIL; no Kitty regression claim from Writer-only evidence. Browser matrix unchanged/not rerun: Firefox92PASS/68FAIL/0BLOCKED/0NOT_TESTED of160; Chrome8PASS/8FAIL/0BLOCKED/144NOT_TESTED of160. No release, push, local installation or daemon/IBus/tray/desktop restart.

### 2026-10-22T14:22:03.524764+00:00 — Writer review closed; Chrome native field binding measured

Writer v98 final Harness2 task `20261022T134131Z-5580fa6d`: ok, eight gpt-6.1-sol/xhigh calls, zero failed/fallback, source unchanged. Selected review8/10, no blocking finding and no further production code required for bounded Writer adoption. Independent acceptance verifies all seven frozen inputs, all nine source origins and existing source472/472/native7/7/order5/5 receipt linkage; baseline/candidate/config remote hashes rechecked at14:20:17UTC. Receipt: `/home/ubu/.cache/lay/development/browser-final-fixes-20261018/shift-hint-20261021T1242Z/writer-forward-boundary-v98-final-review/acceptance.json`. Candidate remains NOT_ADOPTED. Unproven successor-before-old-cleanup and release-after-failure schedules are assurance gaps, not reproduced defects. Corrected stale native-ordering metadata; frozen packet preserved.

Chrome probes v99 and v100b PASS: two native editable fields share one IBus context and equal text/caret but have distinct AT-SPI node paths. Native X11 PID71108 matches the accessibility bus peer credentials; focused+editable Collection queries uniquely match the intended node in all six snapshots. Query median1.126709ms/max1.936602ms,n6; this is not end-to-end correction latency. v100's empty match was a diagnostic MATCH_EMPTY rule error, corrected to MATCH_ALL for unused dimensions in v100b. All seven restoration checks true; protected nine processes preserved. Receipts: `chrome-two-field-binding-v99-probe/result.json`, `identity-audit.json`, and `chrome-focused-collection-v100b-probe/result.json` under the same evidence root.

Important correction: Chrome capabilities41 **does include** SurroundingText32, as defined by IBus1.5.26 and our constant. It is not a missing advertised bit. The actual v100b log contains ten RequireSurroundingText signals and zero SetSurroundingText callbacks. Cached Chromium GTK source connects neither retrieve-surrounding nor delete-surrounding. Verify the exact installed version/backend before choosing an executor. Native text includes preedit projection and is not commit/result ACK. No native provider or Chrome visual fix has been implemented. Next: establish the real existing delete capability and bounded source-to-executor handoff; do not forge SSI or route Chrome through terminal erase. Browser matrix unchanged; Chrome underline remains FAIL.


### 2026-10-22T15:03:16.931162+00:00 — continuation: Chrome identity probe complete, design review pending

v102b native ABA diagnostic PASS: one own Chrome155.0.8059.84, eight/eight unique focused/editable node matches, authenticated native/X11 PID202155, same IBus context2024, actual A→B→A events and equal returned text/caret after native End. All seven restoration checks and nine protected process identities true. v102 failed its diagnostic selection assumption; preserve both receipts. Native log includes real Focus/Reset, so existing canonical token/fence must be evaluated before any extra owner/generation. Exact installed Chromium GTK has no connected retrieve/delete-surrounding; ATK source has no EditableText; Pango NONE/empty/zero-length styles do not solve underline. Chrome prefix underline still FAIL, no production/provider/executor changes.

Evidence /home/ubu/.cache/lay/development/browser-final-fixes-20261018/shift-hint-20261021T1242Z/chrome-native-aba-v102b-probe/identity-audit.json. Harness2 task20261022T143718Z-b6db8805 is still running (session14192), base result only preliminary. Writer v98 source472/native7/final review8 accepted in bounded scope, still not adopted; remote baseline v64PID4140889 retained. Next: finish independent design acceptance and choose the smallest source-to-existing-executor contract retaining Tab, manual/automatic operations and ordinary input. No local install/publication/global restart/new engine names. Browser matrix unchanged.


### Chrome native range/commit primitive v103b — 2026-10-22T15:26:31Z

One owned Chrome field: exact native selection [0,4] + actual existing engine/InputContext CommitText("1") produced `1 `, caret 1/no selection; other field unchanged. Selection dispatch 0.163 ms, observed commit 37.996 ms, n=1. Seven restoration guards and nine protected processes preserved. This is a native primitive only; Chrome presentation remains FAIL, browser matrix unchanged. v103 diagnostic physical-glyph assumption failure is retained.

Authority/alternatives/consequences and the next in-callback native-query discriminator are recorded once in [the owning preflight](../architecture/ime-inline-hint-preflight-2026-10-20.md). Receipt: `/home/ubu/.cache/lay/development/browser-final-fixes-20261018/shift-hint-20261021T1242Z/chrome-native-selection-v103b-probe/acceptance.json`. No production code or installed bytes changed.


## 2026-10-22T15:40:30.803525+00:00 — Chrome callback diagnostic

v104 one in-callback native read PASS (743us,n=1); exact v64 restored PID297899,9+7 restoration checks true. Chrome underline remains FAIL. v105 exact-selection diagnostic preflight recorded in `docs/architecture/ime-inline-hint-preflight-2026-10-20.md`; receipt `/home/ubu/.cache/lay/development/browser-final-fixes-20261018/shift-hint-20261021T1242Z/chrome-native-callback-v104-probe/acceptance.json`. No production fix/adoption or matrix change.


## 2026-10-22T15:47:10.284494+00:00 — Chrome native selection refuses without ACK

v105 diagnostic FAIL `selection_not_observed`1165us; no literal commit; v64 restored PID327512,9+7 restoration checks true. New causal v107 event-driven discriminator preflight in owning IME document. No production implementation/adoption or matrix change.


## 2026-10-22T15:54:33.284419+00:00 — Chrome native event-selection primitive

v107 bounded primitive PASS2443us,n1: actual event confirms exact selection before standard CommitText; result1+Space, second field unchanged. v64 restored PID345082,9+7 checks true. See owning IME document and `/home/ubu/.cache/lay/development/browser-final-fixes-20261018/shift-hint-20261021T1242Z/chrome-native-selection-event-v107-probe/acceptance.json`. Chrome visual FAIL unchanged. Harness2 task20261022T154252Z-2489f752 tool-ok8/8 but deliverable blocked/unread, rejected; one corrected read-only second pass pending. No source authority/runtime adoption/matrix change.


## 2026-10-22T17:09:34+00:00 — KDE-only module boundary v109

User scope: separate KDE folder, GNOME frozen, then Chrome only; reuse existing mechanisms and do not repeat rejected experiments. Final remote compile and exact29/29 existing module tests PASS14.254s; canonical remote graph/architecture PASS51.226s with verified resource limits. Existing files retain logic/module identities, and the common KWin read body is relocated exactly into the KDE adapter; four local GNOME/five protected common files unchanged. Harness2 USED task20261022T170428Z-3a440f56,8/8 successful calls/no fallback, frozen static review9/10/no actionable defects. The final small helper extraction is independently checked outside that frozen review. Acceptance and limitations in acceptance.json; current runtimePID345082/hash/config/protected processes unchanged. Owning analysis and evidence: [IME preflight](../architecture/ime-inline-hint-preflight-2026-10-20.md), evidence root `/home/ubu/.cache/lay/development/browser-final-fixes-20261018/shift-hint-20261021T1242Z/kde-module-boundary-v109/`.

Retain v108 native-range478-test abort (stack overflow in pending_transfer_focus_in_then_focus_in_id_enriches_one_prearm_request after32 tests); no full-path PASS/native adoption. Chrome rendering still FAIL; no runtime/client/release change, full browser100PASS/76FAIL/0BLOCKED/144NOT_TESTED of320 unchanged.


### 2026-10-23T02:38:46Z — active remote KDE Chrome task
Only Chrome; GNOME read-only, no other clients, publication or global restart. Harness2 USED tasks20261023T001629Z-628ade67 and20261023T013040Z-a5285e30 (static reviews only). v132 installed PID2343180, SHA4d2963d2141f944a27506e602d45972d47d08935dc2995ae90f4c31937db00d3, display1.0.94.2. Source60/60 and canonical graph PASS. Physical five-field cycle: all prefixes composing=false, hint+Tab3/5, screenshot still shows hint one letter early. NOT FIXED/NOT ACCEPTED. Next inspect UnknownStart display readout and caret geometry; never promote lineage or edit authority. Detailed evidence: docs/architecture/ime-inline-hint-preflight-2026-10-20.md latest entry; remote control root /home/e/projects/lay-development-runner/kde-install-control-20261020. Full matrix unchanged100PASS/76FAIL/144NOT_TESTED of320. Preserve other dirty work.


### 2026-10-23T03:13:13.837553+00:00 — v133 physical rejection; bounded native character geometry v134

v133 selected71 source/architecture checks passed, but installed PID2412141 SHA f722cba67cec9d32e7aca0db2549f76c87b790dc411e0b101a3244b3d11e7657 fails the physical five-field Chrome cycle: all ready=false, four rect=None and a stale fifth suffix. Receipt chrome-only-focus-cycle-v133-native/result.json. The common terminal HidePreedit lifecycle does not cause Chrome to report post-commit geometry; reject this Chrome trial and restore the terminal-only helper. Keep the independently guarded UnknownStart readout repair. No client acceptance is claimed.

A read-only focused-field diagnostic returns actual text/caret but initial zero-sized character bounds. After the first character-extents query, measurements at60ms,200ms,500ms give final character [67,285,11,21], endpoint78; empty-range and at-caret queries remain zero. Receipts chrome-only-native-caret-v133-{native,settle-native,offset-native}/result.json. Component [28,278,545,35], focused editable field is VISIBLE and SHOWING. This supports querying actual last-character geometry, not a guessed width or a zero-range API. Chromium primary reference: https://raw.githubusercontent.com/chromium/chromium/main/ui/accessibility/platform/ax_platform_node_auralinux.cc (GetCharacterExtents enables inline text boxes). These diagnostics alone do not establish product latency or acceptance.

v134 trial adds one optional character-bounds query to the existing bounded KDE Chrome actual snapshot, retains geometry with that exact snapshot/field/token/lease/layout/tail, and discards it when a result reuses transport. Native geometry suppresses the stale IBus rectangle; missing/zero/invalid bounds hide the projection. It does not change actual document validity, edit admission, reducer/verifier/executor, capability/ACK/SSI, text delivery, GNOME or other-client policy. Existing ordinary native notifications provide updates; no added timer/retry/service. Source/build/install/physical gates pending. Full320 evidence unchanged100PASS/76FAIL/144NOT_TESTED.


### 2026-10-23T03:37:03.613127+00:00 — v134-r2 installed Chrome evidence and v135 review corrections

v134 initial compilation failed comparing the NativeSnapshot wrapper to a text snapshot; no installation occurred. v134-r2 repaired the comparison and selected72/72 source tests plus canonical architecture PASS. Receipt chrome-only-native-geometry-v134-r2-source/result.json; source closure82eba0235bfcedc74870b436bc0671f6396531a1c10fdab741b9d7b8a992f649. Installed only the owned development IME PID2503399, SHA94970cce6c2cea64bb1cbb6824d04a35c5320bdecc50db4d00e8e5adec02d542, display1.0.94.2. Activation/duplicate bytes match;20 protected processes, config, tray, daemon and global IBus unchanged. Receipt chrome-only-native-geometry-v134-r2-deployment/result.json.

Physical KDE Chrome only: the five-field focus/return cycle passes5/5, with actual prefix out of composition, fresh suffix/coordinates, exact Tab result and a single separator. Ten normal-window field types pass10/10; incognito text/rich/iframe-rich pass3/3. Hint observations: five-cycle50.909–79.064ms, ten-types45.450–107.771ms, incognito52.001–54.364ms; Tab24.172–45.281ms across these runs. These include probe/transport observation overhead, not isolated renderer latency. Screenshots prefix-0 and prefix-2 from the five-cycle were independently viewed: plain un-underlined prefix, joined visible completion. Receipts chrome-only-focus-cycle-v134-r2-native/result.json, chrome-only-ten-types-v134-r2-native/result.json, chrome-only-private-v134-r2-native/result.json. Ordered trace anchors are complete; config/engine/daemon restoration passes.

Fullscreen passes2/3 and FAILS iframe-rich: actual DOMпров/composing=false but stale suffixосто with rect=None. Three20ms read deadlines precede a later install_refused/field_identity_changed; application,bus,owner,field all match but canonical desktop lifetime differs. This is an observed timeout/lifetime rejection, not evidence of physical focus theft. Receipt chrome-only-fullscreen-v134-r2-native/result.json. Do not ignore desktop lifetime or weaken admission/identity to hide it. v134 is not final Chrome acceptance. Full320 matrix remains100PASS/76FAIL/144NOT_TESTED.

Harness2 task20261023T032145Z-c4544391 is still running (gpt-6.1-sol/xhigh). Its frozen base review gives6/10 with three independently verified source findings: the outer selection preflight still requests geometry, real cursor callbacks can skip missing geometry once readout is current, and geometry can finish after the last field/privacy checks. This preliminary review is not the task's final report and does not cover v135.

v135 corrections are confined to KDE native_document: propagate the geometry flag through the entire selection preflight (false); complete geometry and revalidate actual text/caret/selection before the final focused-field/privacy checks; allow an existing real cursor callback to read missing native bounds despite current readout. Same20ms bound, no timer/retry/extra observer, synthetic callbacks or authority widening. New private p2p AT-SPI test peers hold geometry while changing field/privacy/text/caret/selection, prove zero geometry calls for edit snapshots, and exercise bounded required-text/geometry failures. Existing managed fixture covers missing/ready/stale geometry callback eligibility. Source/build/installation/physical gates for v135 are pending.


### 2026-10-23T03:48:57.656314+00:00 — v135-r2 source, installation and fullscreen failure; v136 bounded observation trial

v135-r2:75/75 focused source tests and canonical architecture PASS; closure54b2385212ae01a829d8802d19ceb95b7fc4eaf2a0ac0094b12e10eb2ef5a19b. Privacy is checked before every text read, including post-geometry revalidation; held-geometry privacy race test asserts zero private text reads. Installed only the owned development engine PID2578781, SHA72d8856d177dba70ea0974e6881db49b3022150d3b40abc3fabd9205ff3d3037, display1.0.94.2.20 protected processes/config/tray/daemon/global IBus unchanged. Receipts chrome-only-geometry-order-v135-r2-source/result.json and chrome-only-geometry-order-v135-r2-deployment/result.json.

Physical fullscreen Chrome: text/rich pass with fresh suffix/native bounds/plain DOM prefix and exact Tab; iframe-rich FAILS with native_deadline then canonical desktop identity lifetime rejection. Complete trace and restoration; receipt chrome-only-fullscreen-v135-r2-native/result.json. Independently viewed prefix-0 screenshot: typed prefix ordinary and joined suffix, no underline. The full320 matrix remains100PASS/76FAIL/144NOT_TESTED. v135-r2 is not final Chrome acceptance.

Passive native bus timing identifies GetNSelections13.118ms and GetState/GetRole~7.2ms responses inside the same20ms total bound as two canonical KWin observations. v136 trial separates actual-document observation40ms from unchanged edit/subscription20ms. No identity bypass, retries, new wakeup, admission timeout or authority change. Decision2026-10-23-kde-chrome-observation-budget.json records the latency tradeoff and physical gate. Source/build/client verification pending. GNOME and all other clients untouched.


### 2026-10-23T04:26:27.550003+00:00 — v136 measured admission expiry; v137 Chrome-only timing trial

v136 source75/75 and canonical architecture PASS. Installed development PID2609412, SHA564b5c072b9b72f51fa4e20b16aa29bfda9a2ba6d6b5b300365b990230214dc9, source closurec8726c6e445dc20cdeb8b62c10e2db4f214ea7893595ffed179c939854e0775b. Protected20 processes/config/tray/daemon/global IBus unchanged. Receipts chrome-only-observation-budget-v136-source/result.json and chrome-only-observation-budget-v136-deployment/result.json under /home/e/projects/lay-development-runner/kde-install-control-20261020.

Physical Chrome: first fullscreen3/3, normal10/10, private3/3 and focus-cycle5/5 PASS; cold fullscreen repeat2/3 with iframe-rich FAIL. Total23/24, not complete acceptance. Its actual prefix remains composing/underlined; fresh inline frame is empty. The exact marker delay is61.488ms against50ms, not a geometry or identity rejection. Failure retained in chrome-only-fullscreen-cold-repeat-v136-native/result.json and prefix-2.png. Additional passive-wire fullscreen run3/3 PASS, chrome-only-marker-wire-v136-native/result.json; wire arrival and observer ingress agree within100us for the observed markers, but the expired marker was not recaptured, so internal IBus cause remains unknown. Never replace the prior failed receipt with this pass. Full320 matrix remains100PASS/76FAIL/144NOT_TESTED.

Harness2 USED task20261023T032145Z-c4544391 finished ok/ok,8 calls/no failures/no fallback, source unchanged, frozen-v134 review6/10. Report at /home/ubu/.local/state/harness2-codex/projects/lay2/tasks/20261023T032145Z-c4544391/report.json. Independent source verification confirms the three requested fixes in v135/r2: no geometry through selection preflight; real callbacks may refresh missing geometry; final field/privacy/text/caret/selection validation follows geometry, with private-text race tests. That frozen review does not certify v136/v137 and experience-library entries stayed empty.

v137 pending: reuse the same adapter acquisition with an explicit finite deadline selected only after a fresh canonical KDE observation identifies the actual Chrome executable. Chrome100ms; defaults/native-ID/GNOME remain50ms. The classification uses existing20ms native I/O bound; no retry/wake/queue/cache or new edit authority. Other KDE compatibility clients retain their deadline but may incur the additional bounded focus observation; their physical latency is outside this Chrome-only scope. Decision2026-10-23-kde-chrome-activation-budget.json records this tradeoff and supersedes only the earlier no-admission-budget-change statement. Source, installed-runtime and physical proof for v137 remain PENDING.


### Chrome activation request ordering trial — 2026-10-23T04:37:12.137353+00:00

v137 source: 157/157 selected checks and canonical architecture PASS; installed compiler artifact SHA-256 `52b5377dc6468e8a0cee968905b8835f9181593ce5f0b26daf1e88fa3e603e1c`, PID 2724331, version 1.0.94.2. Receipt: `/home/e/projects/lay-development-runner/kde-install-control-20261020/chrome-only-activation-budget-v137-source/result.json` and `chrome-only-activation-budget-v137-deployment/result.json`. Config, tray and protected process identities preserved; only owned development IME replaced.

Physical fullscreen Chrome: 2 PASS / 1 FAIL, receipt `chrome-only-fullscreen-v137-native/result.json`. Text and rich field passed hint/Tab; iframe produced `ров` instead of `пров`. No activation expiry. Actual trace: first letter serial106 ingress Passive before FocusIn102 completed its timing classification; request10 was created afterward. Ready had no exact pending-literal receipt for that earlier letter, so installation discarded it. This is a new request-ordering regression, not a successful client acceptance. Historical browser matrix unchanged.

v138 trial moves the bounded KDE timing classification into the existing acquisition worker AFTER synchronous begin_request. The same request/reducer/Get/nonce/marker/expiry and literal receipt checks retain authority; no receipt is fabricated or relaxed. A delayed-selector regression must prove the first letter belongs to that acquisition and survives Ready. GNOME keeps the original no-probe path. Physical scope remains Chrome only; other clients untested. Source/build/installation/native verdicts for v138 remain PENDING. Decision: `docs/architecture/decisions/2026-10-23-kde-chrome-activation-budget.json`.


### Chrome queued compatibility focus trial — 2026-10-23T04:59:04.498150+00:00

v138-r2 source 158/158 PASS and installed SHA422e08ba7e8be8dfbfea9e7e4458ba123e4723dd252cb6efd89342a31c5d8ea8 are not client acceptance. Fullscreen 3/3 PASS, normal 9/10 PASS: iframe-input retains the full literal `пров` in composition but shows no suffix. Native trace requests36–39 are revoked during queued FocusIn/FocusOut processing, without expiry or accepted marker. First failed transition: a delayed obsolete FocusIn creates a request after a newer FocusOut ingress; final FocusIn revokes the duplicate pending request. Private/focus-return/cold-repeat were stopped and remain NOT_TESTED for v138-r2.

The real p2p regression reproduces the old defect (EXPECTED_RED, 2026-10-23T04:59:04.498150+00:00; exact receipt `/home/e/projects/lay-development-runner/kde-install-control-20261020/chrome-only-queued-focus-v139-red/result.json`). Trial: retain one reducer/stamp authority, publish focus metadata while holding its existing reducer lock, refuse superseded KDE platform-timing compatibility callbacks atomically before request mutation; keep the GNOME/default/native-ID API semantics and all marker/expiry rules. No new cache/queue/owner. Source, installed and physical verdicts for this trial remain PENDING. Historical failures are retained.

Harness2 task `20261023T042842Z-9b5aeab8` is USED, completed status ok, 8 calls / 0 failed / no fallback. Frozen v137 review HOLD6/10 found request-before-classification ordering and scope-cost concerns; it does not certify v138/v139 or client behavior.


### Chrome full lifecycle refusal trial — 2026-10-23T05:26:32Z

v139-r2 source160/160 and architecture PASS; installed development PID2860528, SHA926f63989037f718e6cbeb7842fe871ac3fe6f75ad41ddce64c40c4b2754b73d, source closurec37caef853d6021a7a2b95353d794aeca77fd9a61f0117e223892869a9417808. Chrome-only native24/24 PASS (normal10, private3, fullscreen3, focus-cycle5, cold fullscreen3), complete trace/restoration. Hint median51.23ms/p95 106.62ms; Tab34.73/44.84ms, including observation overhead. Initial iframe-input screenshot masks the last typed letter; follow-up screenshots at150ms/600ms show the full ordinary prefix and adjacent suffix. Preserve the anomalous frame; its cause is NOT established. Receipts: chrome-only-activation-budget-v139-r2-source/result.json, chrome-only-activation-budget-v139-r2-deployment/result.json, chrome-only-{normal,private,fullscreen,focus-cycle,fullscreen-cold-repeat}-v139-r2-native/result.json, chrome-only-v139-r2-iframe-pixels-native/result.json. The full browser matrix remains100PASS/76FAIL/144NOT_TESTED of320. No full-matrix or other-client acceptance.

Harness2 USED20261023T050346Z-c702884a failed its240s base call without a verdict. Task-local600s limit is used by still-running USED20261023T051104Z-f89a405c; its frozen base review HOLD6/10 flags stale full-callback cleanup. Independently verified actual caller: SupersededFocus still reaches finish_focus_in; missing metadata calls fail_context_activation against the currently installed owner. New real p2p full-lifecycle regressions FAIL3/3 on unchanged v139 engine source at2026-10-23T05:26:12.218939Z (chrome-only-queued-lifecycle-v140-red/result.json), proving successor-work cancellation.

v140 trial propagates a distinct ignored observation through the existing KDE compatibility FocusIn callback. Missing/evicted metadata and superseded ingress cannot revoke the current owner or run finish_focus_in cleanup. Existing bool/native-ID/default API and reducer/Get/marker/expiry authority remain; no new cache, observer, owner or delivery route. Three regression tests cover retained/revoked/evicted metadata with successor Pending, Ready and Installed, checking request/fence/ready/token and local work/snapshot preservation. Source/build/install/native gates remain PENDING. This trial changes shared lifecycle glue only for the KDE compatibility refusal; it does not prove other-client runtime behavior.

Errata preserved separately: prior prose red timestamp04:59:04 was the recording time; the actual v139 red receipt is04:57:17.535568Z. v138 deployment reused a review-field count157; its actual source count is158. First v139 compilation failed before installation; v139-r2 corrected the existing5-argument API wrapper and compiled160 tests. Historical receipts are not rewritten.


### Chrome v140-r2 focused development verification — 2026-10-23T05:40:52.712505+00:00

Chrome-only KDE development artifact1.0.94.2, PID2957531, SHA`b8249005a395273c44a35ba1b3c194e6060d8d37489d7f93eefae36c2c1ccd3a` installed. Source163/163 and canonical architecture PASS; native Chrome24/24 PASS across ten field types/private/fullscreen/focus-return/cold repeat. Ten first-ready screenshots show ordinary prefix and adjacent unadorned suffix. Hint median52.40ms/p95 82.99ms; Tab34.58/35.52ms. Full matrix remains100PASS/76FAIL/144NOT_TESTED of320; no other-client or release acceptance.

Full evidence and retained cached-test erratum: [owning display document](../architecture/ime-inline-hint-preflight-2026-10-20.md), section “Chrome v140-r2 focused development verification”; aggregate `/home/ubu/.cache/lay/development/browser-final-fixes-20261018/shift-hint-20261021T1242Z/chrome-only-v140-r2-acceptance.json`. Harness2 USED`20261023T051104Z-f89a405c` finished8/8 calls/no fallback, frozen-v139 HOLD6/10. Both caller-cleanup concerns were independently reproduced and repaired in v140; no repeat review score claimed. GNOME-specific files unchanged, no global restart or push. Leave installed candidate for the user's Chrome check; do not revive other-client work without instruction.

### Persistent remote KDE resource setting and white-suffix check — 2026-10-23

The user explicitly made the 2 GiB free-swap admission floor permanent on
`e@192.168.3.94`. `/home/e/.bashrc` now exports
`LAY_RESOURCE_MIN_SWAP_FREE_MIB="${LAY_RESOURCE_MIN_SWAP_FREE_MIB:-2048}"`
before its noninteractive return; `.profile` already sources `.bashrc`.
Explicit command overrides remain valid. This is a host-user environment
setting, not a change to the checked-in guard or local GNOME environment.
The prior file was backed up, Bash syntax passed, and a new SSH invocation of
the unmodified guard inherited 2048 MiB. Its harmless Python child verified
the actual cgroup limits: CPU `200000 100000`, RAM `2147483648` bytes,
swap `268435456` bytes. RAM/pressure admission and the shared lock remain.
Receipt and backup:
`/home/e/.cache/lay/development/persistent-kde-swap-20261023T075313Z/`.

The first Chrome probe was resource-blocked at the old 4 GiB floor; the next
attempt stopped in read-only preflight because the restarted private KDE
session had not loaded its existing kded5 keyboard module. Loaded that module
through `org.kde.kded5.loadModule("keyboard")`; `org.kde.keyboard.getLayout`
then worked. No binary, IME route or service restart was needed for recovery.

The single actual native Chrome probe then passed two fields (`text`, `rich`):
literal non-composing `пров`, current suffix `ерка`, full opaque white suffix
rectangle with black letters, ordinary adjacent prefix, and exact Tab
completion. Independently inspected both dark-background screenshots. Hint
latencies 51.569/52.820 ms; Tab 25.334/34.780 ms (two samples only). Engine
PID3278357 remains version1.0.94.2, SHA
`b8249005a395273c44a35ba1b3c194e6060d8d37489d7f93eefae36c2c1ccd3a`.
Config/engine/daemon restoration passed. No production source or installed
artifact was changed in this check; no other-client or full-matrix claim.
Remote receipt:
`/home/e/projects/lay-development-runner/kde-install-control-20261020/chrome-only-white-suffix-v141-r2-native/result.json`.
Local copies/screenshots:
`/home/ubu/.cache/lay/development/browser-final-fixes-20261018/shift-hint-20261021T1242Z/white-suffix-v141-review/`.
Harness2 lay2 review `20261023T075418Z-0f56df82` completed with status `ok`,
8 successful calls, zero failed calls or fallback stages, and static score
8/10. Its frozen evidence justifies no production edit; it does not certify
physical rendering. Report:
`/home/ubu/.local/state/harness2-codex/projects/lay2/tasks/20261023T075418Z-0f56df82/report.json`.
Independently checked the omitted KDE common body: it reexports the existing
shared serializers. The native observations above cover only two fields.
Historical full matrix remains 100 PASS / 76 FAIL / 0 BLOCKED /
144 NOT TESTED of320.

Follow-up: the user's active Chrome was its address bar, outside those two
field checks. User reports Google autocomplete rather than a Lay hint; omnibox
acceptance/cause remain unproven. Reverified engine1.0.94.2 and exact b8249005
bytes in the current session. The running tray PID3284870 lacked the existing
KDE StatusNotifierWatcher service. Loaded kded5 `statusnotifierwatcher`;
registered `lay` item maps back to PID3284870 and a new panel screenshot
confirms its icon. No browser/service restart or input to the user's window.
Receipt `tray-watcher-recovery.json` under the persistent-kde-swap evidence
directory; detailed scope is in the owning presentation preflight document.
Fresh-session persistence of the KDE module recovery was not tested.

## User-authorized local GNOME recovery — 2026-10-23 08:46:46 UTC

The user requested a full local Lay runtime restart after reporting GNOME/Kitty failure. Ran the installed `lay-runtime-control restart`, then disabled/enabled the installed Lay GNOME extension. Daemon PID729034 ->805991; IME PID729186 ->806098. IME remains1.0.94, SHA-256 d48b64a615b813374bff30f0fe5358ec64296e24505d0bfa6ad832fde3ddce96. GNOME bridge and IBus both report lay-ime-ru; one IME process, daemon active, extension ACTIVE with no reported errors. Kitty, Codex, GNOME Shell, global IBus and the existing model watcher were preserved. No code change or installation.

This proves component restart/availability only; physical Kitty/Chrome acceptance is NOT_TESTED. The trigger of the earlier11:33 local restart and the reported visual regression remain UNKNOWN. Remote KDE second-word Chrome/omnibox and browser-network complaints remain open; host DNS/HTTPS work, which does not establish browser networking. Browser full matrix unchanged: Firefox92/68/0/0 of160; Chrome8/8/0/144 of160 (PASS/FAIL/BLOCKED/NOT_TESTED).

Receipt: `/home/ubu/.cache/lay/development/browser-final-fixes-20261018/gnome-runtime-restart-20261023T084646Z/receipt.json`.

## GNOME lower-hint investigation — 2026-10-23T09:15:20.693714+00:00

Timeline correction from user: the lower Kitty hint was already present BEFORE our11:44 local restart. Do not attribute its onset to that restart. Read-only diagnosis: live GNOME engine remains1.0.94, PID806098, SHA d48b64a615b813374bff30f0fe5358ec64296e24505d0bfa6ad832fde3ddce96. Installed updater source is5b3c6c5d92ca8d27ac59d2422b64b3e3d04950ae. In that source preedit.rs:636-664 routes candidate suffixes through publish_auxiliary_hint; output.rs:257-260 defines auxiliary support solely as is_legacy(), with no desktop restriction. This is COMMON presentation behavior, not an isolated KDE module. Git blame attributes the publisher to513fb674 (1.0.91 release), retained by1.0.94. Source v1.0.90 directly calls publish_preedit_payload at1318/1333; source comparison independently verified.

The owning preflight document:1074-1087 previously recorded a still-running deleted GNOME1.0.90 executable and identified the same first differing publisher transition. That historical runtime is not reverified now. Current installed GNOME extension contains ime_hint_style.js, no inline-hint module. Thus a shared release change explains how GNOME can receive the auxiliary presentation while KDE work continues. The exact event/time that switched this user session from the older inline behavior remains UNPROVEN; do not assert that our latest KDE1.0.94.2 development binary was installed locally. No code/config/install/runtime changes in this investigation, no native synthetic input. Browser totals unchanged100PASS/76FAIL/0BLOCKED/144NOT_TESTED of320.

## GNOME global hint rollback choice — 2026-10-23T11:42:48.305170+00:00

User clarified the lower hint affects ALL GNOME windows, including Kitty and WeChat, and requested repository rollback rather than new presentation logic. Global legacy publication remains the proven shared cause (preedit.rs:636, supports_auxiliary_text = is_legacy), not a per-app bug. Fresh local runtime check: engine PID806098,1.0.94,d48b64a615b813374bff30f0fe5358ec64296e24505d0bfa6ad832fde3ddce96; daemon PID805991,1.0.94,27b6111982716fd9565c3b88786a2fcf94c886e8c89870ced76b68d6473c3eda.

Created isolated `/home/ubu/projects/lay-gnome-hint-recovery-20261023` at installed-source5b3c6c5d. Withdrew the terminal-only new-module draft via stash ed0841b4; NEVER built or installed it. Recovery worktree is CLEAN. Preserve the separate dirty KDE checkout and remote1.0.94.2 runtime.

Only an UNAPPLIED preview exists: `/home/ubu/.cache/lay/development/gnome-hint-rollback-20261023/gnome-inline-rollback.preview.patch`. It adds resolved GNOME backend to the existing direct-preedit fallback in one function, restoring an old publication route; it is NOT a complete v1.0.90 revert or all-client acceptance. The explicit presentation decision of2026-10-18 and legacy_unaccepted_completion_is_never_document_composition forbid hint-bearing document preedit because Firefox can finalize saved hints on blur. Typed-word preservation and stale auxiliary cleanup are also unverified. An asynchronous user choice about this tradeoff is PENDING; elapsed time is NOT approval. Do not apply, build or install this candidate without resolving that choice; do not invent another overlay/module while waiting.

Harness2 USED: lay2 task20261023T112703Z-6f2c6591,gpt-6.1-sol/xhigh,8 calls,zero failures/fallbacks,status ok. Static proposal score6/10; no runtime/client PASS. Report `/home/ubu/.local/state/harness2-codex/projects/lay2/tasks/20261023T112703Z-6f2c6591/report.json`. Independently verified frozen snapshot correspondence to5b3c6c5d, unchanged input hashes, clean worktree and clean patch applicability. Receipts `independent-check.json` and `acceptance.json` in the preview directory. No source/runtime/config changes, tests, builds, installation or restarts in this checkpoint. Historical browser matrix unchanged100PASS/76FAIL/0BLOCKED/144NOT_TESTED of320.

## GNOME rollback explicitly approved — 2026-10-23T12:10:43.918944+00:00

The user explicitly approved restoring GNOME inline completion despite the documented Firefox blur limitation, then fixing Firefox separately, then isolating KDE/GNOME, in that order. This supersedes the PENDING choice above; do not ask again. Isolated recovery worktree at5b3c6c5d now contains the exact reviewed two-line production change in preedit.rs, explicit presentation regression scope and decision2026-10-23-gnome-inline-recovery.json. No new presentation module or text route. The former terminal-only draft remains stashed, never built. Remote KDE is preserved.

Remote focused IME check in progress: local `/home/ubu/.cache/lay/development/run-yozvvvt6`, remote `/home/e/projects/lay-development-runner/run-kCYkjx`. Compilation finished; current old auxiliary-only fixture failures under implicit GNOME are not a passing check or proof of a new document edit. Keep actual failures and evaluate first assertions; do not hide them. The first attempt run-qqu_gkjt stopped at fmt before tests; formatting corrected. Exact old GNOME engine is backed up at `/home/ubu/.cache/lay/development/gnome-hint-rollback-20261023/before-install/lay-ibus-engine`, SHAd48b64a615b813374bff30f0fe5358ec64296e24505d0bfa6ad832fde3ddce96. OwnsActiveText=false at preflight; installed runtime not yet replaced.

## GNOME restored and user accepted — 2026-10-23T12:21:21.420271+00:00

GNOME engine installed atomically from remote frozen build, restarted via existing lay-runtime-control. PID2202067, SHAfa5e8950cc4c8c39e108464c9d06da0c00a32d26782f6dc2c030007f70e49a44; daemon PID2202010 unchanged bytes, IBus4618/GNOME3894/Kitty161163/training3282 preserved. RU source/readback agree. Backup is before-install/lay-ibus-engine in gnome-hint-rollback-20261023. User explicitly confirmed “Вот работает как надо!” and “Отлично!”. Accepted scope: current GNOME inline appearance only; no full-matrix or Firefox-blur acceptance. No more functional verification of this accepted scope unless requested. Source check remains909PASS/30FAIL (old auxiliary-only first assertions), build PASS. Receipt /home/ubu/.cache/lay/development/gnome-hint-rollback-20261023/installed-runtime.json and user-acceptance.json.

Latest order supersedes earlier order: first isolate GNOME/KDE presentation, then ONLY Chrome in KDE. Firefox blur limitation remains separate and unresolved. Preserve accepted GNOME runtime. Dedicated recovery checkout /home/ubu/projects/lay-gnome-hint-recovery-20261023 at5b3c6c5d has exact installed production patch; dirty KDE checkout and remote1.0.94.2 preserved.

## User pause, then analysis before further code — 2026-10-23T13:35:59Z

The user paused work, then requested analysis before continuing implementation;
also asked whether Harness caused the regression. This turn performed read-only
source/runtime/log inspection and recorded the audit, with no production edits,
tests, builds, input, installs or restarts. Runtime identities freshly match the
accepted GNOME SHA fa5e8950cc4c8c39e108464c9d06da0c00a32d26782f6dc2c030007f70e49a44
and prior KDE SHA b8249005a395273c44a35ba1b3c194e6060d8d37489d7f93eefae36c2c1ccd3a.

Separation source is partially implemented, not installed. Comparing ALL process
logs corrects the prior count: identical 16 ordinary failures before/after,
plus the same TD-121 stack overflow; final source gate run-jk6rjfe5 remains FAIL.
Six relevant style/publisher checks are ok, not native acceptance. Audit found
unrequested Niri/X11 style change in display.rs and that chrome-only-white-suffix-
v141-probe.py tests first words in different empty fields, not a second word in
one field. White KDE QLabel is rendered in scripts/lay-kde-tray.py:303; attrs.rs
alone does not describe that presentation path.

Harness2 task20261023T124213Z-9b3b632a: base-failed,240-second timeout,one call,
no selected output. Reviewed tool-event metadata: only output/static-check.py
created inside the isolated task. Eleven full input source files match current
production source; the Niri/X11 change was already in the review input. No
successful independent review exists for the separation. Prior global regression
mechanism is shared publication in513fb674 without a desktop selector; this is
not proof of Harness authorship. Do not blame a tool without matching evidence.

Owning audit: docs/architecture/desktop-presentation-audit-2026-10-23.md.
Next proposed steps: complete the minimal explicit desktop boundary while
preserving other backends, obtain bounded review, then reproduce Chrome second
word on the unchanged KDE binary and fix only its first observed failed transition.
Keep accepted GNOME runtime, unresolved Firefox limitation and historical browser
matrix100PASS/76FAIL/0BLOCKED/144NOT_TESTED of320 separate.

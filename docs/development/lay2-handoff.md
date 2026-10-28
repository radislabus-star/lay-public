# Lay — one KDE Chrome observation fix COMPLETE, handoff 2026-10-28

## Current task after the completed seven releases

User explicitly narrowed work to ONE improvement, dependency map before code,
isolate by responsibility, short tests before long. Owning map/preflight:
`docs/development/kde-chrome-owned-append-map-2026-10-28.md`. Baseline
`32d073687ae8d631042ba127148dcaf492a394d1`. Scope is KDE Chrome older-owned-prefix
observation on Space, demonstrated normal/tel `ghbdtn ` failure; other failures
are not tasks in this run. New ADR and active scope name the exact extraction
seam, tests and inventory. No Harness or subagents.

Existing read coordinator moved to `kde/chrome/observation.rs`; initial function
body mechanically equal SHA12eec6ab9d6514a8f34500c2c54fb60200b74db788a7317cf2ff152d76cbb1dd.
RED three source cases:2PASS/1expectedFAIL (two pending reads returned too early).
GREEN3/3. Only functional change is actual pending-prefix reads within the same
80ms deadline, 1ms async yield, unchanged source-error/identity/authority fences.
Source after graph update:2ce6b24786bab558599a46345c9ec3583f419847420a1f37ebf5a08e93933c5e,
1301files. Component75/75, fmt and graph/architecturePASS. No shared verifier,
executor, model, presentation, GNOME/terminal/focus code change.

Remote candidate loaded in place of engine only:PID3556137,
SHA8b84bf5b6c569550af8e4061ea4cca83e74382d243ae693aeb243a1e22a035d3.
CLI version remains111; this is explicitly an unpublished candidate, not a new
stable release. Daemon3228075/tray3228081, private IBus parent3168326 unchanged;
no daemon/tray/global IBus/local GNOME restart. Backups/build/rollback receipt:
CROOT/chrome-owned-append-deployment-v1. Exact live config hash6e62eab3c278c733fedb47e45ef6306105a3700222cf9f7a4e5bff4bd92264d1.

Short native: immutable r19 six Chrome casesPASS (tel/text Space, tel consecutive
two active/Space, moved pair and Tab). Additional number control initiallyFAIL:
the RFB fixture delivered IBuskeyval61 '=' before Lay instead of '+'. Original
FAIL retained. Fixture corrected to explicit physical Shift+Equals, reusing
release109-stock-shift-v3.py's established method; expected phone text unchanged.
The corrected number and held-Shift controlsPASS in fresh short-native-v2;
six originalPASS reused with unchanged candidate/source:8/8PASS. Long immutable
r19 Chrome-only220 COMPLETE:179PASS0FAIL41BLOCKED,NOT_TESTED0. The original
normal/tel/space-en-ru FAIL becomesPASS (87.20ms). No new functionalFAIL.
Per-case comparison also records two coverage movements:private/email/roundtrip
active PASS→BLOCKED(native/DOM disagreement),fullscreen/email/rapid-five active
BLOCKED→PASS. Never say all blocked identities or all prior PASS were unchanged.
Config restored, loaded engine before/after identical, zero held keys.
Broader official source check alsoPASS:3555/3555 (3519correctness+36package),
including1117IME. Localrun-n2we7ij5/remoterun-vkUi8t; fmt/toolingPASS,
0semantic/infra failures,41ignored and11performance notrun by these lanes.
Final runtime independently verified unchanged after this check. The selected
ONE fix is COMPLETE; do not open another client fix automatically. No new
release/publication/local install is part of this task. Owning machine-readable
record:docs/development/kde-chrome-owned-append-2026-10-28-verification.json.
Current cache prefix chrome-owned-append-*, CACHE/CROOT below. No publication
or local manual update is requested by this one-task instruction.

## Previous seven-release series — COMPLETE

User seven-release request105–111 is COMPLETE: all seven stable published, four CI checks and local/remote updaters verified for each. No Harness2 or subagents used. No manual local GNOME install/restart/input. Work directly in the accepted checkout, all builds/tests/graph refresh/native input remote under existing guards. Keep C01–C10 and common correction/ownership/verifier/executor. Preserve every FAIL/BLOCKED/NOT_TESTED.

Canonical checkout `/home/ubu/projects/lay-dependency-updates-20261027`, branch `codex/browser-matrix-repair-20261027`. Private origin/main is divergent future113: never force/rebase/push it. Push only the workbranch and fast-forward public/main. Public repository radislabus-star/lay-public.

## Publication state

105–111 stable published with four public CI and actual updater checks verified for each. 105=4ebd03f3,106=f88c4ba9,107=e178be59,108=058a247f,109=96a98674,110=4b629cfe1966755de8d32d6c36c893e45c35c0eb,111=22e9fead9744918b99c175454362d9e705caf608. Stable111 published 2026-10-28T10:00:55Z. Publicmain/tag111 source22e9fead; workbranch may have a later publication-record-only commit. No runtime difference or empty tags.

105 repairs observed-boundary pair/NotReady preparation;106 moves KDE preparation;107 isolates Chrome qualification and current-token read resampling;108 separates shared reply settlement and incomplete identity refusal;109 moves existing KDE terminal/focus policy;110 fixes actual VTE purpose10+advertised surrounding admission and relocates unchanged Chrome presentation.111 adds source-scope enforcement and existing-executor component recipes, with no runtimeRust change from110.

## Current verified111source

Sourcebfc7270982b8f71a90020c60ca4650040215fb84e3a56ce616f8e187229c52e2,1300files. Full3555/3555(3519correctness+36package),1117IME,41ignorednotrun,11performance,18gatesv1,10releasebinaries0warnings,MSRV1.88default+lexical. Official localrun-oj99ol62/remoterun-HbHRYB; frozenrequest includes field_integration_scope base4b629cfe and26changedrows. Canonical actualGit delta is sealed locally; remoteGitHEADa8fbda3c is synthetic and NEVER canonical.

Tooling causalREDoldentrypoint exactly2missing effects→GREEN33/33(including19oldfixturecontracts). Five actual source recipes:Chrome75/75in5.28s,terminal100/100in33.30s,focus8/8in.71s,replay16/16in.76s,GNOME9/9in.74s; warmcache/profile, overlap, not additive full-suite count. New scope guard rejects cross-client/shared/unknown paths, omitted/deleted declarations and dependency/code changes disguised as own-version metadata. Exact exceptionpaths/ADR predeclared, including publication evidence110/111. FrozenCIworkflow/scopeconfig/ADR unchanged. Canonical guards not weakened; no server branch protection claim.

## Current installed/native111result

Remote192.168.3.94,e,SSH existingkey. Display:71, user managerbus unix:path=/run/user/1000/bus; actualGUI/IMEprivatebus comes from daemonenvironment. Never replaceGUIbus withmanagerbus. ProtectedIBusparent3168326, no globalIBus/desktop restart.

Engine3228031SHA159435bd88be77b5aa0ffef217d64b168785f7190da46dec6f43b9543d1a09a5;daemonSHAeb6098b445c48ae5a681fb8b1d33051e28a5da3646d90baa4cd47b09a7228de5. Exactdeployment-v1. Ordinary terminal functionalKitty11/Konsole11/VTE11allPASS;GTKQt25/25PASS. VTE improvedfrom3/11to11/11in110. Source111inherits exactruntimeRust; onlyversionlabels/toolingchanged.

Full immutable r19actual browser matrix,220each(160original+60two-word):Firefox125PASS56FAIL39BLOCKED;Chrome178PASS1FAIL41BLOCKED;aggregate303PASS57FAIL80BLOCKED/440,NOT_TESTED0. Worse than107309/51/80; no browser improvement claim. FirefoxFAILclasses50rapid-five,3moved-boundary,2roundtrip,1consecutive-two. ChromeFAILnormal/tel/space-en-ru. Preserve raw outcomes/targets/cadence; no retries-to-green.

Visual separate:Konsole/VTEpre-TabframesPASS(ordinaryprefix,whiteending). Kittyone111captureFAIL(no visibleending; own snapshottextерка,rectnull)despitefunctionalTabPASS;110capturehadvisibleending. Root attributionopen, no broad/globalhintfallback allowed. Do not presentfunctional33/33as allrenderingPASS. TerminalRFB→PTYtiming n15each:Kittymedian20.27p9526.72ms,Konsole20.83/25.41,VTE20.28/26.19;actorpolling/gestureincluded,notpureengine. NativeGNOMEDesktop/Wayland/USB/realChromeomnibox not newly tested.

External SSH command ended143 duringcombinednativephase, but originalremotechildjobscompleted all440. Recoveredaggregatev2 validates originalcomplete manifests,loadedSHA,configrestoredto deploymenthash andzeroheldkeys; no input repeated. Outerphaseaggregate missing remains recorded. Fullbrowserelapsed872.71s. Do not rerun completed native proof just for a clean outer exit.

## Final evidence and continuation boundary

CACHE=/home/ubu/.cache/lay/development/browser-matrix-repair-20261027; CROOT=/home/e/projects/lay-development-runner/kde-install-control-20261020. Versioned tech_debt/evidence/release-1.0.111-verification.json contains source,gates,CI,updater,raw receipt links,all57browser FAIL identities and Kitty visual FAIL. Owning110/111docs retain both the rejected first VTE candidate and actual purpose-policy repair. Original completed native proofs are valid despite SSH143; no input repeated.

Sourcecommit22e9fead blobs/executable bits match the frozen1300-file sourcebfc; six additional CI/scope/ADR/instruction files match request hashes. Publictag/stableAPI111 and actual updaters verified; local manual install/restart never called. Remote installed111 engine3228031 SHA159435bd88be77b5aa0ffef217d64b168785f7190da46dec6f43b9543d1a09a5. Publication-only metadata must not reopen tests/build/install/restart/native input. User receives local update via menu/autoupdater; final updater observed local107 and remote111.

The seven-release task is finished. Remaining per-client work is NOT claimed fixed: Firefox50rapid-five+3boundary+2roundtrip+1consecutive case,ChromeTelSpace case,Kitty missing hint geometry. For a new fix map the first failed transition, declare baseline/components/exact shared seams/newADR in active scope before code; use existing canonical recipes and full affected proof. No global rendering fallback, parallel owner/verifier/executor, desktop environment changes or shared-learning cleanup.

Final upstream refresh is stored in CACHE/release111-final-upstream.json. All three entry/architecture/development reminders are in this checkout; old dirty /home/ubu/projects/lay is NOT this source.

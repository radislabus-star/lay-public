## Active: Kitty letter-fragment continuation — 2026-11-10

Accepted checkout: this branch, baseline a73529c0, primary installed125.
The old loaded123 IME was replaced only through the existing full-restart
operation; no permanent candidate installation. Loaded125 hash is recorded in
`docs/development/kitty-fragment-mode-2026-11-10.md`.
Evidence root: `/home/ubu/.cache/lay/development/kitty-fragment-runtime-20261110/`.
One implementation owner, no Harness2. Independent read-only review completed.
Only current Kitty issue; do not revive other queues.

Candidate1 direct source switch rejected (IBus fallback/admission failure);
layout_sync source/tests restored to baseline. Candidate2 preserves existing
writer layout ownership for native GNOME: 1181/1181 source checks PASS and
actual two-word before-Space continuation PASS. Eight manual returns FAIL:
привет -> привtn. First source seal has tail_epoch_matches=false because the
new letter caller publishes text after key settlement without output settlement.

Current candidate3 changes only committed_tail.rs and adds fragment_layout.rs
tests. It uses existing begin_context_bridge_output for that native GNOME
fragment caller, revalidates the current token before output, and retains the
existing writer/worker. Adjacent KDE native postcondition uses the same scope.
No reducer, delivery, Space, KDE or managed GNOME implementation changes.
Two focused regressions PASS; removing the output scope makes the new exact
settled-epoch assertion FAIL. Revocation/failing-output controls included.
Full focused IME gate PASS1182/1182. Candidate3 native PASS: four scenarios,
19 ordered assertions covering both directions/two words, eight complete manual
returns, fast typing and fragment-OFF Space. All installed125/config/source/model
restoration checks PASS. Exact receipts in candidate3-native-acceptance.json.
Independent cold review 9/10, no required fixes; no independent physical claim.
Candidate IME SHA afce5fed6a7b41071726cb72bf92548d8d746f274d7c15b56df55e432fa9ea7b.

Release126 manifest and architecture refresh PASS. Frozen request local
run-tp6k0t5c / remote run-wAaRru. All runtime Rust files and dependencies match
the native candidate; only version metadata and generated evidence changed.
Full gate PASS at /home/e/projects/lay-development-runner/kitty126-release-gate-20261110:
3652/3652 correctness/package,11/11performance,475Python PASS/12environmentSKIP,
lints and adapter regressions PASS; all ten126 binaries, installer and actual
Rust1.88 default/research all-targets PASS. Total1134.44s. Frozen2323file ledger,
compiled1338file SHA ac4f49e121237b19096d48bc465156d719c5a275c08f6641cc0e780e352309c3.
Local controller log: kitty-fragment-runtime-20261110/release126-gate.launch.log.
Next: commit/push/CI/tag/release, verify actual
updater discovery. Primary125 remains installed; no forced candidate install.
Initial126 short gate stopped at Cargo cache12GiB budget (not a test failure).
Removed2.95GiB of unused pre-Nov10 debug Lay artifacts only; all release/live
executables and current-day debug artifacts preserved. Repeat short gate.
No release before mandatory gates. Analyzer lay-diagnose rejects terminal
receipts (browser-only support); retained trace supplies direct evidence.
Temporary remote resources require safe restoration after builds, exact paths
in owning record. Historical KDE: 329 PASS / 59 FAIL / 52 BLOCKED of 440.
Resource note: vdo-audio-summary.service is a SYSTEM unit, not a user unit.
Its PID3261319 is still active with3.47GBcurrent/3.23GBanon; do not restore the
old3GiBmax while below its live allocation. Temporary properties are runtime
only (5GiBhigh/6GiBmax). Remove our unused2GiBswap after all heavy gates finish.

## Completed: stable updater repair / Lay 1.0.125 — 2026-11-10

The user's Space-only fragment report and blocked124 update were traced to
actual deployment: primary installed/running123 with flag=true; tag123 has no
letter caller. Exact retained124 native logs show ghbd->прив and рудд->hell
before Space. Later KDE inspection found CLI124 but original123 IME bytes;
ordinary updates retain active input, so loaded bytes must be checked separately.
The existing explicit full-restart action loads new IME after user updates.

Stable125 published2026-11-10T10:09:02Z, release440199326:
https://github.com/radislabus-star/lay-public/releases/tag/v1.0.125
Code8695d9c96f8e68248182f3c0de893f8a2b2dcfb0; annotated tag object
1c00f4ea4c552c872f46c2e8dc2e8e0cf2dc5eda identicalorigin/public and peels to
that code. Allfour public CI jobs PASS (run42792473022 +security42792473247).
The only production behavior change from124 is update.sh: default official
API403/429 may use same-repository HTTPS latest-release redirect, strict tag
validation and unchanged exact-tag/no-downgrade/confirmation/source guards.
No .rs changes. Cargo/tray version/date are125; generated receipt metadata only.

- Original updater RED3tests/5subcases; reviewed updater/installer GREEN37.
- Remote development source3650 PASS; canonical release3650 correctness/package
  +11performance,475 headlessPython+12environmentSKIP, allten actual125
  installables, actualMSRV1.88 bothlocked modes, architecture/lint/CLI/ngram/
  installer PASS. Frozen source1338 SHA
  f79d39f81734e4e873923fa552d9c11e014fc8725d23626b18191ab0b7df2f63
  matches local/remote; complete2317-file ledger seals rootupdater/extension too.
- Rootupdater SHA b7c05b3eff7199cadca53d8403482dd6ca4c957e5cc396dc511721e9d6b2d836.
  Script-only bootstrap under existing lock on bothhosts, exact reviewed bytes.
  Actual installed --check afterpublication: primary123->125 in0.592s,
  KDE124->125 in0.682s, bothexit0/applyfalse. Latest API/website select125.
  No binary install, live config/model/learning modification or service restart.
  User obtains versions through menu; do not force primary installation.
- AST graph update was REJECTED_WATCH (three unchanged sinks lost capability
  edges). Rawgraph/report retained. Accepted Rustgraph/sourcebinding restored;
  existing PASS-only receipt refreshed forCargometadata. No runtime/guard change.
- New taskscope basee334f2f4 closes previous46 input seams. Only exact updater,
  generatedreceipt,trayversion/date sharedfiles +tooling allowed. No new input
  integration authority. Re-declare the next actual task before runtime edits.
- No fresh125 native acceptance claimed. Latest ordinaryKDE matrix remains
  Firefox143/51/26, Chrome186/8/26 PASS/FAIL/BLOCKED:329/59/52 of440. This
  updater patch does not repair those existing client failures.

Owning analysis: docs/development/fragment124-update-rate-limit-2026-11-10.md.
Notes: docs/release-1.0.125.md. Complete evidence:
docs/development/evidence/release-1.0.125-verification.json and
release-1.0.125-publication.json in the same directory. C is
/home/ubu/.cache/lay/development/letter-fragment-request-20261102;
fullreceipt C/updater125-release-RESULT.json, devreceipt
/home/ubu/.cache/lay/development/run-cyg_x58o/RESULT.json.

The scoped updater task has reached stable publication; no runtime implementation
or installation is pending. Publication-only documentation follows the code
commit without retagging or repeating accepted gates. Final Git/API/readback
receipt is C/updater125-final-readback.json. A future task starts from a new
concrete user issue and its preflight, not these superseded checkpoints.
Private origin/main47fe54e6 is preserved. SOLO/noagents/noHarness/noquestions.
Historical sections below do not restart completed experiments.

## Completed: Lay 1.0.124 — 2026-11-09T23:30:20.930515+00:00

The isolated letter-fragment task has reached its stable release. No pending
runtime implementation or installation remains for this task. Existing native
compatibility failures are explicitly retained below; this is not an all-client
PASS. Historical checkpoints after this section are superseded evidence, not
instructions to resume old experiments.

- Stable release: https://github.com/radislabus-star/lay-public/releases/tag/v1.0.124
- Published at 2026-11-09T23:23:04Z; release ID 439761658. Latest, not draft,
  not prerelease, independently read back from the public API.
- Canonical code commit: 702fb6afe6a0eec98ce879983b3b54b03250819f.
- Annotated tag object: 37103074f12bea31406d8e73241d5e1ee7ca5929. Origin and
  public tag objects and peeled commits match. Keep this tag on the code commit.
- Four public CI jobs passed: installer, MSRV 1.88.0, RustSec, and Rust 1.97.1
  with complete correctness/package, lint, syntax, CLI, release and ngram checks.
- Exact frozen source: 1,338 files, SHA-256
  6ae8643762400ebc7c4fc3cc37f0abd73fb153ebd3ceeab71b1eb77663d5b3b5.
  All ten frozen artifacts63/65 are byte-identical. Full local/remote release
  gate64 applies to this exact source. This metadata-only publication record
  does not change it or reopen functional verification.
- Both actual installed menu entrypoints returned success with installed
  1.0.123, stable 1.0.124, update available. Mode was --check; apply=false.
  User obtains versions through the menu. No forced primary installation.
- New fragment feature on KDE: Chrome108/0/12 and Firefox108/0/12
  PASS/FAIL/structural BLOCKED, 240 total. Primary GNOME:32 physical mode/word
  transitions plus3 native Tab/held-Shift cases independently verified.
- Ordinary KDE browser suite remains329 PASS /59 FAIL /52 BLOCKED of440:
  Firefox143/51/26; Chrome186/8/26. Failed observations were not waived.
  Immediate-Space layout conversion with fragments ON passes16 supported
  fields, with4 structural URL/email blocks; it is a subset of that suite.
- Matched comparisons:33 observations by first shared failure class;
  12 strict terminal placement checks;6 primary GTK Space cases. Persistent
  failures reproduce on exact originally installed123. The primary Space
  fixtures retain the same typos in124 ON/OFF and123 OFF; equality is not
  successful correction. Do not generalize one class representative to all59.
- A nonzero hint rectangle is insufficient: strict Kitty placement fails
  first/second word in allthree configurations; Konsole passes both in allthree.
  Epoch-based coordinate retention passed7 source tests but failed native
  placement. It was rejected and exactly reverted in two source files and
  the active scope. Do not revive it without independent native position proof.
- Source evidence:3,650 Rust correctness/package plus11 performance;467
  headless Python cases plus10 graphical and2 real-system cases;28 owned-Xvfb
  opt-ins; allten installables, lints, architecture, installer and actual MSRV.
  Seventeen research/live opt-ins were not invoked. Raw failed whole graphical
  and legacy-producer diagnostics remain separate from independent proofs.

Publication and update-discovery receipt:
`docs/development/evidence/release-1.0.124-publication.json`.
Complete scoped verification and known limits:
`docs/development/evidence/release-1.0.124-verification.json` and
`docs/development/release-1.0.124-letter-fragments.md`.
The final publication receipt also exists in C/publication-final-1.0.124.json.
C is /home/ubu/.cache/lay/development/letter-fragment-request-20261102.
Work was SOLO, without agents or Harness. Private origin/main47fe54e6 was
preserved. Publication-only documentation can follow the code commit without
retagging or repeating accepted source/native tests. Finish with branch/tag
readback and the configured half-volume completion sound.

## Historical checkpoints — superseded

## CURRENT 2026-11-09T23:28:29.462031+00:00: STABLE1.0.124 PUBLISHED; CI4/4PASS; installed updaters both123→124; metadata-only followup

Code702fb6afe6a0eec98ce879983b3b54b03250819f; tagv1.0.124 object37103074f12bea31406d8e73241d5e1ee7ca5929 identicalorigin/public andpeeledcode702. Stablepublicrelease439761658 published2026-11-09T23:23:04Z, URLhttps://github.com/radislabus-star/lay-public/releases/tag/v1.0.124, draftfalse/prereleasefalse/latestverified. CI installer/MSRV1.88/RustSec/Rust1.97+tray+extension allPASS. Both REAL installed menu entrypoints lay-update --check return0 installed1.0.123 stable1.0.124 available; applyfalse. No forcedinstallation. Local andremote menus resolve actual installed updater scriptSHAAebfb65..., stableGitHubAPI; remote trayPID1271545 active(aliaslay-kde-tray, resolvesNov2bundle .py). Earlier auditexpected.pyargvwaswrong; no tray failure/restart.

Source1338SHA6ae8643762400ebc7c4fc3cc37f0abd73fb153ebd3ceeab71b1eb77663d5b3b5 exactlyfrozen65/full64 remains unchanged bythispublication-onlydocs. Currentfeature216P0F24structuralB/240 +primaryGTK35 independentlyverified. OrdinaryKDE440 remains329P59F52B, NOTPASS; classbaseline33/strictplacement12/primarySpace6 comparisonscomplete andrawfailuresretained. Immediate-Space layout16P0F4B withinordinarysuite. Source3650Rust+11perf/467headlessPython+10GUI+2system/28ownedXvfb/alltenbuilds/MSRV/lints/arch allP;17research/live optinsnotinvoked. Coordinate-epoch experimentREJECTED_NATIVE,2sources/scoperestoredexactB65; do not revivebasedonlocalwordmetadata. No newruntimeeditneededforpublication.

Publicationreceipt docs/development/evidence/release-1.0.124-publication.json; completeknownlimits release-1.0.124-letter-fragments.md+verification.json. C/publication-final-1.0.124.json exactAPI/updaterevidence. Finalremaining: commit/push ONLY thesepublication/owningdocs withacceptedfunctionalproof reuse, verifybranchheads/tag/latest andcleanstatus, finalcheckpoint, completionsoundthenfinalRussianreport. TagmustremainCODE702; docscommitcanfollowwithoutretag. Privateorigin/main47fe54e6 preserve. SOLO/noagents/noHarness/noquestions. Userinstallsviamenu. MemorycitationlastMEMORY.md5193-5197 rollout01a115ea-09f2-7003-81c5-26037edf60eb.

## CURRENT 2026-11-09T22:56:25.477810+00:00: code702fb6af pushed; public CI3/4PASS, Rust1.97 stillrunning; no tag/release yet

Codecommit702fb6afe6a0eec98ce879983b3b54b03250819f now originfeature/publicfeature/publicmain, privateoriginmain47fe54e6 preserved. Postcommit source1338 SHA6ae8643762400ebc7c4fc3cc37f0abd73fb153ebd3ceeab71b1eb77663d5b3b5 exactly equals frozen65/full64 (onlygithead changed). PublicCI source run42715425381,security42715425643: installerPASS,MSRV1.88PASS,RustSecPASS, Rust1.97 checkjob129495573112 RUNNING. Waitall4beforetag/release. Rawcheckreceipt C/public-ci-current-1.0.124.json. No permanentinstall, source123 restoredbothhosts. Finalletter-native216P0F24B/240; ordinary329P59F52B/440 retained, baseline33+strictplacement12+primarySpace6 comparisons complete. Rejectedcoordinateexperiment exactlyreverted; currentSourceHintView/KdeTerminal have no diff. No more runtime edits planned. Final publication docs-onlyfollowup can reuse accepted functional evidence; do not rerun unchanged broadtests.

## CURRENT 2026-11-09T22:44:13.500615+00:00: final source restored exactly B65; all bounded comparisons completed; prepare124 commit/CI/publication

SOLO/noHarness/noagents/noquestions; continue through publication, no final yet. No commit/push/tag/release/permanentinstall so far. B76 coordinate retention REJECTED_NATIVE_PLACEMENT, exact2sources+scope reverted to prechange74, CURRENTSOURCE48→46 protected paths restored. Local source1338 SHA6ae8643762400ebc7c4fc3cc37f0abd73fb153ebd3ceeab71b1eb77663d5b3b5 matches frozen65/full64; allten63/65 binaries byte-equal. Do not reintroduce epoch-retention: current local word does NOT prove postpaint coordinates. Native77 disproved it despite GREEN7 source tests. Final source has NO hint_view.rs or kde/terminal.rs diff.

Full source gate64PASS still applies EXACTcurrentcode:3650correctness/package+11performance,467Python headless+12SKIP with10GUI/2real optins separate,28ownedXvfb,allteninstallables,lints,architecture,actualMSRV1.88. Nativefragment65 COMPLETE216P0F24structuralB/240; GNOME35 independentnativePASS; rawTabproducerFAIL retained. Ordinarybrowser66 COMPLETE329P59F52B/440 (FF143/51/26;Chrome186/8/26), NEVER wholePASS. Matched69 COMPLETE33 observations collapse persistent failures by first shared class, original123 reproduces rank and rapid refusals. Stronger strictplacement80 COMPLETE12:Kittyfirst/secondFAIL124ON/124OFF/original123OFF;Konsolefirst/secondPASSallthree. Nonzero rect oldpositives canoverlaptypedletters, notplacementacceptance. Renderer71onlyprivatefixed; retainedraws67/72/77/80. PrimarySpace79/80 COMPLETE6 actualphysicalGTKcases:both errorclasses retainidenticaltypo124ON/OFF/original123OFF; functionalcorrectiontargetsFAILallthree, equalityonly. Original123sources/mode/installedhashes restoredbothhosts; no permanentinstall. Raw78mis-profiled/producerFAIL preserved too.

Releaseacceptance is BOUNDED OPTIONAL LETTER FEATURE, not allLaydefects fixed. docs/development/release-1.0.124-letter-fragments.md +evidence/release-1.0.124-verification.json contain all rawfailed verdicts/limits; C/release-1.0.124-notes-draft.md finalized honestly. Scope/canon local79PASS. Next cleanmetadata/source review then COMMIT/PUSHfeature origin+public/publicmainfastforward (privateorigin/main47fe54e6 preserve), waitpublicCI, annotated124tagboth, stablepublicrelease, check actualinstalled menu updater --checkbothhosts. No forceprimaryinstall; userupdatesmenu. Recordpublicationreceipt/docs-onlyfollowup afteractualsuccess, reuse functionalsourceproof. Completion soundONLYtrueend; memorycitationlastMEMORY.md5193-5197 rollout01a115ea-09f2-7003-81c5-26037edf60eb.

## CURRENT 2026-11-09T22:25:30.632926+00:00: B65 fragment240 complete; ordinary440 FAIL baseline-comparison33; bounded KDE hint experiment GREEN7/build76 RUNNING

SOLO/noagents/noHarness/noquestions; user authorizes necessary fixes and all tests through release124, keep working. No commit/push/tag/release/permanent installation. Original123 restored both hosts. Remote current job is freeze-kde-hint-v76.py, exec session43281, log C/freeze-kde-hint-v76.launch.log; do not edit R source while its source closure assertion/build is running.

Full source/release64 PASS and GNOME35 native belong to B63/B65 exact same ten binaries. Final LOCAL=REMOTE B65 source is1338 SHA6ae8643762400ebc7c4fc3cc37f0abd73fb153ebd3ceeab71b1eb77663d5b3b5. Earlier f0a differs ONLY generated architecture_graph_receipt graph binding; eight actual final generated files fetched in milestone70, allten runtime bytes63/65 independently equal. B65 native fragments COMPLETE Chrome108P0F12structuralB/120 and Firefox108P0F12structuralB/120, each two manual returns+nextglyph.

NEW completed ordinary native66: Chrome186PASS8FAIL26structuralB/220; Firefox143PASS51FAIL26structuralB/220; total329/59/52/440. Never claim whole Lay passes. Matched native69 COMPLETE33 observations:124 ON,124 OFF,exact originally installed123 OFF for each first shared failure class. Rank/correction and rapid-five refusals reproduce on original123; raw failures all retained. Single KittyhintON initiallyFAIL whileOFF124/123PASS, but these nonzero-rect verdicts are NOT placement acceptance (see screenshot).

Private terminal fixture bug: redraw full text containing leading newline from current cursor moved y18px per letter. Actor71 fixes only child renderer with fixed screen origin; all keyboard/oracle functions initially unchanged. Native72 firstKittyhint/Tab passes all3matched modes, fullKitty8P2F(tab-second no geometry,Space etbya),Konsole9P1F(Space). Space ranks no_apply on original123 too. Saved C/kitty-hint-before-epoch-v72.png visually proves old positive nonzero-rect hint actually overlaps typed letters (onlyp+erity visible), so strengthen placement oracle; do not relabel old whole tests PASS.

BOUNDED NEW SOURCE EXPERIMENT, after map/ADR/preflight: src/bin/lay_ibus_engine/hint_view.rs stores read-only tail_epoch in cursor geometry and can retain it only via KDE policy; src/bin/lay_ibus_engine/kde/terminal.rs checksKDE+existingnative qualification+sameepoch+existing currentfocus/activation coordinate. Existing clear/wake retained for stale/missing geometry, GNOME, other paths. No text writer, Space, detector or output permission changes. Activeprotectedpaths48 (prior46+these2), decision docs/architecture/decisions/2026-11-10-kde-current-terminal-hint-coordinate.json; owner docs/development/kde-terminal-hint-coordinate-map-2026-11-10.md. Exactprechange backup C/kde-hint-prechange-v74 contains both2sources+oldscope. RED74 fresh geometry present then erased None vs expectedSome, actualRusttestFAIL (no compileerror). GREEN75 seven hint_view testsPASS including oldepoch/focus/missingadmission/GNOME negatives, no text/tail/scope/mode mutations. Current source catalog/empty ledger still B65! Refresh atomically ONLY if native short proof supports this change. Do NOT publish unverified candidate.

Next: await freeze76 (allten compiler-artifact retained diagnostics). Prepared private actor/control76 single and full: input functions exactly71, stronger tab_case asserts hinted x follows actual rawPTY text from empty-paragraph client coordinate anchor; first+second expectedx=anchorx+len(lastline)*cellwidth. Anchor captured BEFORE prefix. Same1.2s hintdeadline/target/input. Short current76 Kitty/ Konsole first+second hint/Tab +screenshots FIRST. Need new coordinator (none yet). If this one bounded experiment fails actual placement, reject/revert only these2sources+scope from prechange74 and preserve raw evidence; do not wander into delivery. FrozenB65 source/fullgates remain safe fallback. If passes, add nonterminal/no-edit-permission negative controls as needed, refreshmanifest+emptyledger together, graphupdate, fullrelease/lint/MSRV gates, freeze exactfinal10, fresh supported fragment240 and primary35 plus terminalplacement acceptance; old B65native cannot certify changed bytes.

C=/home/ubu/.cache/lay/development/letter-fragment-request-20261102, R=/home/e/projects/lay-development-runner/letter-fragment-release-20261102. W actual checkout samebranch/HEAD4fae57d5. Separate 20CPU24Ghigh28Gmax1Gswap dedicated remote lease, RUST_TEST_THREADS1; primary human-safe-run1536M tasks192, no bypass. Never run forbidden oldliveconfig cleanup. Native runs use normal installed models +private10learningpaths, exact PID/SHA and original123 restoration. No forced primary install; updater only. Source and final evidence draft docs/development/release-1.0.124-letter-fragments.md +evidence/release-1.0.124-verification.json currently PENDING, must update/reconcile after this source experiment.

Publication still pending: source/scoped evidence, precise known limits, commit and pushoriginfeature/publicfeature + publicmain fastforward (privateorigin/main47fe54e6 must NOT overwrite), publicCI, sameannotated124tag bothremotes, stablepublicreleaseAPI, actualinstalled menu updaters --check on both hosts must discover124. Finalsound ONLYtrueend via codex-turn-done-sound, nofinalnow. Memorycitation final last MEMORY.md5193-5197/rollout01a115ea-09f2-7003-81c5-26037edf60eb.

## CURRENT 2026-11-09T21:42:44+00:00: source release gate64 PASS; frozen65 fragment240 complete; GNOME35 actual PASS

SOLO, no Harness, no questions, continue through tested release124. Same accepted checkout/branch/HEAD; no commit/push/tag/release/permanent installation yet. Original123 restored both hosts. All ten installables frozen in remote letter-fragment-final-candidate-v65-20261110; actual IME/daemon/input bytes identical to primary-tested63. Final source1338 SHA6ae8643762400ebc7c4fc3cc37f0abd73fb153ebd3ceeab71b1eb77663d5b3b5; post-gate64 graph binding metadata fetched in milestone70. All ten63/65 runtime binaries are independently byte-identical. Earlier f0a hash belongs to63 before final graph-binding refresh.

Full source/release gate64 PASS: 479 Python headless467PASS12SKIP; 3650 Rust correctness/package and11performance PASS; default/research strict lints, architecture, all10release installables, ownedXvfb28, public installer/binary contracts, actual Rust1.88 both all-target checks PASS. GUI10 separately PASS(v52 raw whole run remains FAIL7 private-bus fixture guard failures); two real system opt-ins PASS(v60). No waived tests. Source inventory3706, known_failures empty; historical dead-code baselines117/129 unchanged.

KDE final-native65 COMPLETE: Chrome108PASS0FAIL12structuralBLOCKED/120; Firefox108PASS0FAIL12structuralBLOCKED/120. Four letter cases(first/second,both directions) x10fields x3modes; each two manual returns and physical next-glyph verification. Exact24 structural blocks are URL/email second-word whitespace sanitization, never PASS. Actual frozen bytes, normal installed models with private mutable learning, field and runtime restoration verified. Receipt /home/e/projects/lay-development-runner/letter-fragment-final-native-v65-20261110/RESULT.json SUPPORTED_PASS_WITH_STRUCTURAL_BLOCKS.

Primary GNOME current63/65 bytes: native eight-model32/32 and Tab-first/second + held-Shift3/3 independently verified,20+6 negative oracle controls. Raw Tab actor legacy assertion FAIL retained separately. Latency32 samples second physical Shift PRESS to first native replacement: median12.535-14.545ms, p95 89.818-91.827ms, max95.780-97.796ms; syscall timing bounds, GTK/GNOME scope only. Original123 installed files/InputMode/source/engine restored.

Next running: native-terminals-v67.py (ordinary DesktopExec Kitty/Konsole10each, same65 bytes; preserve failures including old fast Space prefetch). Then prepared native-standard-v66.py standard220each Chrome/Firefox with intermediate rapid-five oracle. Serialize KDE native suites. No further production changes planned absent a newly proved causal defect. Final source review, precise release notes/evidence, commit feature branch to origin/public + public main fastforward, CI, annotated124 tag/stable public release, installed updater --check must discover124. Do not overwrite private origin/main or force primary installation. Completion sound only at actual end.

## CURRENT 2026-11-09T21:18:02.742379+00:00: last-native-readiness fix GREEN25+actual Chrome1PASS;fullgate64RUNNING;GNOME63RUNNING

SOLO/noagents/noHarness/noquestions/noendearly. No commit/push/tag/release/permanentinstall. Latestuserenvironment date2026-11-10 Istanbul; sourceexperimentsdates09/10byrunID. ActiveWsamebranchHEAD4fae57d5;installed/restoredoriginal123bothhosts. C=/home/ubu/.cache/lay/development/letter-fragment-request-20261102,R=/home/e/projects/lay-development-runner/letter-fragment-release-20261102.

v59FULLB58bothcomplete:Firefox108PASS0FAIL12exactstructuralB;Chrome107PASS1FAIL12exactstructuralB,0NOTTESTeach; originals/config/maps/modelsrestoredPASS. Rootletter-fragment-final-native-v59-20261109. Table215P1F24B/240. Oneoldremainingprivateiframe-richEN-RUsecondrawghbdtn:postreplysourceReadsshowactualcursor==tailchars11,known/frame/exact_managed/nativecurrent truebutwholeprefixreadinessfalse. Existingnative_managed_readout_is_currentalreadyhandlesoldseparatornormalization. Readsource beforeedits; no newwait/owner/queue/adapterchange.

NewADR2026-11-09-fragment-current-native-word-readiness.json isactiveDecision,same46protectedpaths. Owningmapfragment-letter-reply-order-map-2026-11-09.md. ProductionONLYfragment_layout.rs: onePRIVATEread-readinesspredicate current_external_snapshot_agrees_with_owned_tail OR(KDE && context_word_is_known && existingnative_managed_readout_is_current). UseSAMEpredicateinitialpendingcheckandboundedrefreshloopbreak. Comment:observationonly;fullframe/request-source/current-authority/DecisionCore/verifier/singlewriterstillmandatory. UnknownStartfallbackunchanged;GNOME/terminalnewbranchnegative. NOeditobservation.rs/authority.rs/tailmemory.rs orotheradapters. Groundedcurrentwordproofnotpermission.

Newtestreply/tests.rsonefunction(+1catalog),exactcurrentNativeBinding/fullFrame/nativeRange/freshpostCommitreceipt positivebeforeassert. Initialredv61fixturelackedcapbits—NOTcausalproof;reusedadjacentset_client_capabilities(41). MEANINGFULREDv61bASCIIsameprefixpasses/NBSPtoSpacefailsONLYoldpendingguardafterallpreconditionspass. GREENC62newnegativeownerNoneincorrectlyremovedcachedownerwhilecanonicaltokenstillvalid;correctednegativeusingexistingrevoke_context_wordatendAFTERotherclones,notruntimeguardchange. GREENv62b25/25PASS inclselection/wrongword/stalereceipt/focus/lease/epoch/layout/token/scope/binding/privacy/sealed/GNOMEandactualcanonicalrevocation. ExistingUnknownStartrefusalteststillPASS. No testloosened.

DIAGNOSTICv63realprivateiframe-richEN-RUsecondsourcecompiled/currentfieldPASS1/1,2manualreturns+physicaldecoder,beforeSpace;latency22.464msONESAMPLE;119otherfullsuitecasesNOTTESTEDbythisdiagnostic. SourceRTorig123/config/maps/modelsrestorationPASS. Rootletter-fragment-word-ready-v63-20261110/RESULT.json. v63preflight:write-manifest3706,updateUNCHANGEDemptyknownledgerbindingBEFOREgraph,graphfresh,all10compiler-artifactbinsfrozen. Ledgernow56ce40aba2b3263311c42377ab669e83c3f63605ca82f7b5ab212353578386c9;failures[]/count0/oldacceptedobservationSHAsamefe3f... . No newwaiver. CandidateB63 remoteletter-fragment-final-candidate-v63-20261110;source1338SHAf0a1010f57a6bfbb1e0c9aa697788ea0b1205c6dc9a5736e6c6fa343ffd8ae10;IMEa69654a20d29496d0cd08de35fbc8486c195c2a302b4276e4264cffcf52e132f;daemon57aaf3.../input3676...unchanged. LocalC/primary-candidate-v63SHAverified3bins;8generatedfilesfetchedW.

ShortactorcoverageBUGfixedPRIVATEtoolonlynative-letter-selected-actor-v62.py: --onlyexpected_rows1takesprecedenceover--fragment-suite120;explicitcoverage_scope/unselected_full_suite_rows119. ALLfunctionAST(input/setup/target/observer/actions)IDENTICALtor24v49. Full120actorUNCHANGED. Oldv54/v54bcoverageFAILreceiptsretained. Shortcontroller native-letter-short-control-v63.pysameprivate-model/restorationlifecycle.

RUNNING sourcefullgate64:exec66530,teeC/full-gate-v64.launch.log,remoteletter-fragment-full-gate-v64-20261110/RESULT.json. Graph+all479PythonPASS(12skip),nowcanonicalfullRust3650+11perf/currentstrictlints/allbins/CLI/ngram,then28ownedXvfb(UNIQUEv64namespace),publicissue/install-binarysmokes,realRust1.88toolchainandbothall-targetdefault/researchchecks,targetbudget. RootTargetwas4.514GB/12GBbeforejob,MSRVcacheabsentsoampledisk. AllheavyremoteD20CPU28GBonelease. DON'Tlaunchremoteotherheavy/nativeuntil66530finishes. InitialprematurelaunchscriptnotyetrsyncedFAILwasorchestrationonly,separatelogretained;waitedcopy24598completedthenactual64running. DoNOTrepeatthiserror:sequentialdependenciesMUSTwaitexecsessionbeforelaunch.

RUNNING primaryGNOME35onB63:exec46422,guard1536M/tasks192/actualhumanjobcgroup;C/gnome-proof-v63.py +teeC/gnome-proof-v63.launch.log. SameinputTSVs/fixedsampler(v58cDROPunusedVisibleTailV3RPC;twoREQUIREDstandardGNOMEpropertiesstillbothqueried); unchangedverify-gnome-native.py+20negativecontrols,32intermediatephysical/uinput/nativeWord/decoder/modes. Thenfirst+secondTab/heldShift3andprivateindependentverify-gnome-tab-v63.py+6controls. ALLsupportfilenamesv63/v63b;actualB63foldernotB58. EarlierB58GNOME35PASSretained;v58rawfirstsampler63/64globalFAILdueUNUSEDshiftreleaseTailRPCretained,strictcheckerNEVERweakened. Primaryoriginal123mustrestore;no permanentprimaryinstall.

PreparedANDuploaded C/native-final-v65.py+freeze-current-v65.py:REQUIREsourcegate64PASS;freezeall10B65compilerartifacts,compare3runtimeSHAswithB63;thenUNCHANGEDfullfragment120Chrome+Firefox(r24actorv49controller),strict108P0F12exactstructuralBeach onlypass;keeprawBLOCKED. Rootfinal-native-v65-20261110,modesnormal/private/fullscreen10fields4lettercasesincl2manualreturns+decoder. IfB65bits==B63reuseGNOME63physicalproof;ifdifferentneednewbyteproof. Sourcegraphcompiledreceiptmayaffectbytes;neverguessequality. DO NOTrebuildduringinstall.

Afterfeature65supported0FAIL:standard220eachbrowserusingpreparednative-standard-control-v60.py(r49sameactorremovefragment-suiteonly,expected220);no fullcurrentstandardyet(oldv26notcurrent). NeedordinaryDesktopExecKitty/Konsole10knownlinecases/oldfastSpaceON-OFFnormalpositive; C/native-terminal-features-control-v47.pytakesLAY_NATIVE_TERMINALkitty/konsole andcandidateenv,clonelearningprivate,actualDesktopExecnoinjectedIMEflags. Forbiddenolderkde-terminal-native-mode-native-v1.pyMUTATESLIVEconfig,neverrun. PositivesKnowStartonlyphysicalEnterAFTERsourceselection;CtrlAresetunknownbydesignandmustnotweakenC03. OldKonsole37ms/charSpaceetbyaFAILbothONandOFFprefetch_not_ready,notfeaturegate;preserveoldfailedproofandnormal100mspositiveifneeded. No literalruntimewordexceptions. Existingv47Konsole9P1oldfastF,Kittyextendednotrun;v44both2Plegacy5open+5closed.

Systemoptinsv60BOTHrealcgroupdescendant/L11transientownedunitPASS;installedfilesunchanged. GUIv52actual10casesPASSunderownedXvfb/privatebus;whole479mixed470P7resourcefixtureF2skipretained(contextprivatebusunavailableactualmanager),notproductPASS. Combinedwithfreshheadless467P12skip+2realoptins givesall479uniquePythoncovered. No GUI/guard/controllercodechangedbylatestRusthelper;reusecurrentevidence. Sourcev57bwholePASS3649+11perf+10bin+28xvfb,old0waivers;notnewsource64substitute.

Beforepublishneedcurrentfullgates+actualsupportednative/ABI/version/artifact/source review,accuratereleasenotes/source124alreadyset. UserprimaryupdatesONLYmenu,neverforceinstallprimary. Actualinstalledupdater--checkv123baselinePASS;AFTERpublicationmustdiscover124usinginstalledupdaterandverifyGitHubtag/sourceversion. StablelatestAPIlastverified123published2026-10-31T12:23:35Z;remoteactivebranchHEAD4fae57d5same. Don'treviveoldPR/issue/Harnesswork. FinalsoundONLYtrulyfinished:codex-turn-done-sound withWcwd;mandatorymemorycitationLASTMEMORY.md5193-5197 rollout01a115ea-09f2-7003-81c5-26037edf60eb.

## CURRENT 2026-11-09T20:53:56.892516+00:00: final sourcev57bPASS;B58 frozen;GNOME35PASS;Chrome107/1/12;Firefoxv59RUNNING

SOLO/noHarness/noagents/noquestions, persist currentfragmenttask throughrelease. NOcommit/push/release/permanentinstall. Original123restoredprimaryandChromeKDE. exec53712 isRUNNINGfullnativev59 (Chromecomplete/restoredPASS,Firefoxrunning). Sourcegatev57 failedONLYledger binding AFTERall3649Rusttests0fail/sourceunchanged. Fixedonly known_failures.json manifest_sha256 old74a79...→77208b2744251a19191d9b69d3ca12c897382da422d4b8e82648a3e23e072e9a,failures[]/count0/oldobservationSHApreserved; owningADRexplicitlyrecordsstrictmetadata repair. Actualcanonicalgatev57bFULLPASS733.6s incl3649Rust+11perf,strictlints,all10installablebuilds,28ownedXvfb,budgets. Prefix479Python(467P12skip) reusedfromv57 becausecodeunchanged. Systemoptinsv60 BOTHrealcgroup/L11transientservicePASS; installedfilesunchanged. Togetherwith10actualGUIcasesv52,unique479Pythoncoveragecomplete; retainv52mixed7resourcefixtureFAILunderprivatebus separately. No rule/testweakened.

B58 remote final-candidate-v58-20261109/RESULT.json andlocalC/primary-candidate-v58. Current source1338files SHAa458e5798d7982014589912e10f3196bb093048358c7d720c3e9ea8366daa131,HEAD4fae57d5. IME43cc12403e4a62196c605fcbee71aee6f01e8a40c739e83dc82813f4d5751dd0; daemon57aaf3e090d7fcfc67b1b777fdc366ca47aae5bbbaf95faf79d107dfba337e8f; input3676bd0d8761426951071d26c08b67e8ba231c047a4e753526954eaef4866258. All8generatedfilesfetchedW. B58stilldiagnosticuntilmandatorynative.

PrimaryGNOME B58: firstv58raw4cases allOK/restored123 butindependentoracleFAILbecause63/64samplerrows,1unusedVisibleTailV3diagnosticdeadlineerror; missingpair7shift-release(RUclosed),all32requireddecoder-probemoderowspresent. DoNOTclearrawerror orweakenverifier. Sampler-onlyv58c REMOVEDunusedVisibleTailV3query; stilltwoactualstandardGNOMEproperties,unchangedphysicalTSVs/verifier/assertions. NamesALLnewv58c exceptBremainprimary-candidate-v58. Rerun4casesrawPASS,unchangedstrictverify-gnome-native.py+20negativecontrolsPASS32/32. EvidenceC/gnome-gtk-eight-model-v58c-20261109/NATIVE_ACCEPTANCE.json. Actualfirst+secondTab/heldShift B58rawPASSandindependentC/verify-gnome-tab-v58.pyPASS3/3+6negativecontrols; evidencegnome-gtk-tab-shift-v58b-20261109/NATIVE_ACCEPTANCE.json. Tabpress→visibleboundsfirst2.417–4.503ms,second0.949–2.959ms. Bothmodel/originalsource/engine/filesrestorationPASS. SourceRustunchangedbyallmeasurementfixes. CompletionbeepNOTdone.

NEW fullChromeB58v59 result107PASS/1FAIL/12structuralBLOCKED/0NOTTEST/120. Exactold3failidentitiesallPASSnow. OnlyFAILrow78 PRIVATE iframe-rich letter-en-ru-second(auto expectedтестпривет remainsтестghbdtn). Complete/detailedtrace +RESTORATIONPASS rootR/../letter-fragment-final-native-v59-20261109/chrome. Firefoxsame120RUNNINGunderexec53712; don'tmutateRsource/startotherremoteheavyuntilitfinishes/restores. LatestcompleteFFold108/0/12;currenttable215/1/24of240mixedgenerationsclearlylabelled.

ResidualCAUSEnewdistinguishingproof: row78haspost-reply ibus_letter_fragment_reply atEpoch1220and1222;boththen native_observation_pending. Intermediate5requestcorrectlycancelledby6thinput(reply_scope_changed). BUT lastfreshread actual_cursor11/localtailchars11,native_binding_current=true,native_document_current=true,known=true,exact_managed=true,frame_available=true,manual_word_current=true;stillpending. This isNOTjustoneletterbehindanymore. Featurepre-readguardand2-readlooprequire current_external_snapshot_agrees_with_owned_tail (entireoldprefix). Currentexactnativewordcanalreadybevalidwhileolderseparatornormalized. Actualsourceexistingkde/native_document.rs::native_managed_readout_is_current explicitlydocumentsChromeolderseparatornormalization; delegatesexact_managed_surrounding_word_is_current (window_interaction/observation.rs1072:ManagedCommit,noprivacy/atomic/composition,latestpostcommitreceiptmatchingfocus/epoch/revision,liveToken,exactsuffix+bothwordboundaries). FreshreadbeforewordchangecurrentNativeRangeexecutable. NeedreuseEXISTINGwordproofONLYasfeatureREADINESS,stillrequireKnownStart/fullcaptureFrame/unchangedverifier/samewriter. Don'tclaimliteralNBSPcausefullyobserved:localprefixcodepointsnotlogged;wholeprefixmismatchwithvalidcurrentwordproved. No morewaiting/newqueue/owner/adapterrewrite. Likelyminimalhelperinfragment_layout.rs: current_external_snapshot_agrees... OR (backendKDE && context_word_is_known && native_managed_readout_is_current);useSAMEhelperinpendingpredicateandrefreshloopbreak. Existingcapture_fragment_layout_frame fallbackforUnknownStartstillunchangedwhole-tail;mandatorycompleteFrame/route+input_frame_authority+WordLayoutOrigin::Fragmentexistingwriterremain. GNOMEnewbranchmuststayexcluded. FirstTDDactualKnownNativeword+olderseparatorreflow REDcurrentpendingguard/GREENreuse; negativesstale/unknown/selection/wrongword/focus/epoch/nativebinding/privacy. BEFOREeditmap/ADRexactscope(existingfragmentparentandtests,nosharedtailmemoryedit),thenfocusedsource/shortfailedrowfirst,fullcurrentgates/native. C09currentlyscope46alreadyallowsfragmentparent. Newtest+1 requireswrite-manifestANDemptyledgerbindingbeforearchitecture/fullgate;don'trepeatv57metadataoversight.

Sourceplaces:fragment_layout.rs224captureFrame,~300pendinghelper,2-readloop. window_interaction/observation.rs1072exact_managed...1110suffix+boundaries;native_document.rs529native_managed_readout...535,read_with_owned_append_resamplingexisting80msdeadlineifneeded(butresidualiswholeprefixveto). authority.rs988text_target_edit_route factsnative_range_supported=native_document_is_current. native_document/tests.rs2451separatorreflowexistinggeneralfixture. ReuseexistingtestfixtureLayIbusEngine::fragment_reply_test_fixture() (testonly) fromnative_document;ManagedCommitSnapshotFloortypecanfindactualsetterbeforeguessing. Don'tinventedpathsearches:rg--filesFIRST! Severalguessfield_edit*,key_event.rs/client_context.rs causedtoolrgexit2, NOTproducterrors.

AnalyzercommanddirFAILEDINVALID_EVIDENCE becausecustomcontrollercaselayoutnotrecognized; existinganalyzer load_bundle supportsFILEwithrows+expected_rows. Correctnextcommand scripts/lay-diagnose.py report <chrome/case/result.json> --output <newCdir>;readwarnings. Priorwrongdiroutputanalysis-v59-chromenotcreatedprobably,usenewname. No assertionchanged.

PreparedbutUNRUN: C/native-standard-control-v60.py(r24actorunchanged,removefragment-suite,argmanifest220);C/native-final-v59.py currentusesB58noextrafreezeassertv57bPASS;C/system-optins-v60.pydone. Needcurrentstandard220bothbrowserstomeetprotocol,ordinaryDesktopExecKitty/Konsoleknownline10controls+positiveSpaceON/OFFnormalvsold37msfastdebt;MSRV1.88actualtoolchaininstalledoptionalCIcompat. Don'treviveoldissues/dependencytasks. Neednewactualinstalledupdater--checkdiscovers124AFTERpublication,noforcedprimaryinstall. FinalsoundONLYtruecompletion, memorycitationlast.

## CURRENT 2026-11-09T20:12:20.664032+00:00: reply ordering implemented;1179IMEPASS;full final gatev57RUNNING

SOLO/noagents/noHarness/noquestions. Active W lay-dependency-updates-20261027,HEAD4fae57d5; branch codex/browser-matrix-repair-20261027. Stable/public+restoredinstalled123bothhosts. Source124unpublished. No commit/push/release/permanentinstallation. Latestuser authorizes allnecessarychanges/alltests, persist throughrelease.

Current complete fragment matrices remainFirefox108P0F12structuralB/120 (v44+email6v51);Chrome105P3F12structuralB/120(v51). TwoChromeautomaticsecondword failures provednative snapshot oneletterbehind untilkeyreply. ThirdnormaltextEN-RUsecondmanualmissnotyetexplained. Map/ADR:fragment-letter-reply-order-map-2026-11-09.md and2026-11-09-fragment-native-letter-reply-order.json; scope46paths. Newfragment_layout/reply.rs usespinnedzbusResponseDispatchNotifierbool(no wirechange), SAMEengine viaObjectServer,get_mut,metadataonlyowner/focus/lease/tailEpoch/text/mode/token stamps; refactoredoldguards/proposal/apply reused, existingfreshnativeproof+samewordflip/verifier/writer. DefersONLYpositive managedKDE native pending, neverSpace/OFF/GNOME/terminal/ownedpreedit. Newreply5sourcecontrols incl20invalidations,exactboolwire/privatebus/staledocument. ExistingfiveRustdirectcalltestfiles unwrapbool viahandled,behaviorassertionspreserved; existingnativefixture reusedbytest-onlyhelper in2alreadyauthorizedfiles. No protectedsourceguard weakened.

Sourcev53c24/24PASS;v53d1179/1179IMEPASS(159.84stest,176.08stotal). v54 frozen10installables DIAGNOSTIC_ONLY_NOT_PROMOTABLE; includesOLDgraphv53 compiledreceipt,sofinalgraphbytesneednewfreeze. v54bactualnormaliframe-rich RU-ENsecondselectedrowPASS44.08ms,2manualreturns/physicaldecoder PASS;controllerwholeFAIL ONLYbecauseactorwith --fragment-suite+--only requires120rowsbutselected1. RawFAIL+restorationPASS retained. Donot rerun/alterthisactor'scoverage assertion; run unchangedFULL120 instead. v54firstattemptselectednamebad->0rows, retained. Newsource/reply jobset scriptsunder C.

RUNNING remote fullgate /home/e/projects/lay-development-runner/letter-fragment-full-gate-v57-20261109/RESULT.json; localC/full-gate-v57.launch.log,exec10688. Strictdefault/researchlintsPASS71.6s, manifestrefreshPASS(new3705tests expected),finalarchitecturegraphPASS55.2s,all479PythonPASS(with12skip), nowfull-release-gate includesallRust+11perf andallreleasebins. ThenownedXvfb28andbudget. DoNOToverlapremoteheavy/nativeleases. One dedicated20CPU28GBguard. No newsourceeditsplannedwhilegate.

Aftergate: runprepared C/freeze-current-v58.py underguard -> remote final-candidate-v58; actual all10compiler-artifact copies, fullgatev57pointer, diagnosticstatusuntilnative. Fetch8generatedgraph/testmanifestfilesfromR intoW, andB58RESULT+IME/daemon/input toC/primary-candidate-v58; verifySHA. Currentarchreceiptcontainsgraph+sourcefingerprints compiledintobins, soearlierB54native iscausalshort proofONLY; finalnative usesB58bytes. GNOMEwrappers/spec alreadyprepared withALLv42namessafelyreplacedv58:C/gnome-eight-model-v58.py,gnome-tab-shift-model-v58b.py,gnome-eight-model-v58-verify-spec.json. Sameinputwrapperv38readsactualbinaryfromenvB58(nohardcodedoldbytes). Primaryhuman-safe-runcheck1536M/tasks192PASS(12GiBavailable); neverpermanentprimaryinstall. Usecorrectv58support filenames; don'trepeatoldv43overwritebug. VerifyGNOME32actualuinput/native/modeswithverify-gnome-native.py+20negativecontrols; Tab/heldShift3separateactual proofneeded.

Nextremote fullfragment120each: UNCHANGED native-letter-fields-control-v49.py (r24actor) withenvLAY_NATIVE_BROWSER,LAY_NATIVE_RUNnewdir,LAY_NATIVE_CASE=standard,LAY_NATIVE_CANDIDATE_RECEIPT=B58/RESULT.json,LAY_NATIVE_DIAGNOSTIC_DAEMON=1. RawactorBLOCKEDfor12URL/email2ndword cases isstructural; don'ttreataswholePASS. Require108supportedPASS,0FAIL,exact12structuralB; keepobservationalblocksunknown. Thenappropriate standard220each/termKittyandKonsoleknownline/Tab/heldShift/fastOFF-ONmatches asnecessary. Existingstandardv26historical,notcurrent. Allcontrollersprivate models/configs andrestoreoriginal123; neverforbiddenoldterminalwrapperthatwritesliveconfig.

GUIv52ownedXvfb/privatebus ran479,470P7F2SKIP; all7Fresource-guardfixtures(realuser-managerqueriedfromprivatebus). EXACT10GUIcases allPASS(6InputAccessUi,1KdeInputAccessUi,1KdeDaemonMenu,1KdeCheckboxMenu,1SettingsUi). RawwholeFAILretained. NoGUIcodechanged; canreusecomponentproofwithout rerunning. Preparedv55GUIscriptnotrun/unneeded. Remaining2optins realcgroup andL11systemd; runafterheavyjobunderrealbus, ownprivateleases, noinheritACTIVEforcgroup subjectguard. Tests.createONLYowntransientunits/sleepprocesses,nolivesocket/modeltouch. Fullsourcegateheadless12skipneedscombineGUI10+2optins foruniqueall479coverage.

Terminalproofv47/v48: correctKnownStartrequiresphysicalEnterAFTERsourceselection; CtrlAresetleavesUnknownStart deliberatelyguarded. Konsole9P1oldfastSpaceF, ordinaryKitty/Konsole2Peach legacy5open+5closed; Kittyextendednotrun. Old37ms/charSpace etbya→tebyaFAILwithFragmentONandOFF(prefetch_not_ready), preserveoldshareddebt/no literalwordfix. Fixednormal100mspositiveON/OFFcontrolmayclarify. OldFirefoxrapidprefix2P1F notfixed; rejectedKWinreusev43exactlyreverted3files. Donotretrythatoptimization.

Publishonlyaftermandatorygates/nativecomplete. FreshGitHubAPIlatestv123published2026-10-31T12:23:35Z,targetmain; remoteactivebranchHEADsame4fae57d5. Actualprimaryinstalled updater --checkv123baselinePASS; mustverifydiscovers124AFTERpublication, noforcedprimaryinstall. Noissuecomments/dependencyworkactive. FinalsoundONLYtrueendusingcodex-turn-done-sound; mandatorymemorycitationMEMORY.md5193-5197 rollout01a115ea-09f2-7003-81c5-26037edf60eb.

## CURRENT 2026-11-09T19:15:00.275180+00:00: mandatory full gatePASS; final frozen v50; Chrome120running

SOLO/noHarness/noagents, no questions; authorized all necessary fixes/all tests through release. No commit/push/tag/release/permanent Rust installation. Original123 restored both hosts between owned tests. R=/home/e/projects/lay-development-runner/letter-fragment-release-20261102, C=/home/ubu/.cache/lay/development/letter-fragment-request-20261102. Continue current fragment task, not historicalwave/PRwork.

FULL GATEPASS remote letter-fragment-full-gate-v45-20261109/RESULT.json (907s). All34Python modules479total=467PASS/12SKIP; all mandatory Rust correctness/package3644 +performance11 PASS, strictdefault/researchlintsPASS, format/architecture/node/python/shell/CLI/ngram/allreleasebinsPASS; actual10installablebinary buildPASS. Actual current isolatedXvfb resource guards28/28PASS remote letter-fragment-owned-xvfb-v45-20261109. BudgetbeforeafterPASS. No runtime install.

AllPython initially foundstale whole-file SHA for alreadyauthorized fullRestart dispatcher. FIX only tests/test_release_live_install_controller.py: advance full-file pin92843...→066d84... and ADD reverse-projection test stripping only new helper/import gate/dispatch/usage token to recover exact acceptedv123bytes. Controlled old-dispatch mutation mustfail oldpin. NewADR2026-11-09-full-restart-controller-proof-pin.json; ownerfull-restart doc updated. BothfocusedcontrolsPASS, fullPython479PASSwith12skips. No runtimecodechange. Activefieldscope narrowed to39paths/tooling/approvedword-mode-delegation ADR after rejectedv43focusreuse; scopePASS.

FINALFROZEN candidate remote letter-fragment-final-candidate-v50-20261109/RESULT.json, BUILT_FROZEN_NOT_INSTALLED,1336files source580ea4e28534ead2f2ce29d8f546bb40863ca7307a949f02e36bf005e60f854f, HEAD4fae57d5 working124. All10installables resolvedfromfresh actualCargo compiler-artifact JSON; records/files retained. Actual IME/daemon/input bytes EXACTv44/v42: IMEa49d131e9d9bef72fc4f2aab5ea3b46e64a323a02d2ed262baffcb63bd5e577a; daemon57aaf3e090d7fcfc67b1b777fdc366ca47aae5bbbaf95faf79d107dfba337e8f; input3676bd0d8761426951071d26c08b67e8ba231c047a4e753526954eaef4866258. Thus v42GNOME32/32,Tab3 and freshv42closed8 andKDEshort10 are same executable bytes, not newbuildassumption. Need fetchfinal8generated files +RESULT localandverifycurrent sourceclosure beforecommit. Canonicalmanifest3700=3608correctness+36package+11perf+45ignored, failures[]; no waiver.

Firefox native fragment120 COMPLETED raw105PASS/0FAIL/15BLOCKED, remote letter-fragment-field-modes-firefox-v44-20261109. CORRECTIONtoolderhead:3email observationblocks were NORMAL en-ru-first, PRIVATE ru-en-first, FULLSCREEN ru-en-first (NOT en-ru inallmodes). EveryautomaticbeforeSpace passed normal+private emailcases; mismatchoccurred during laternative/manual snapshots.12otherblocks URL/emailsecondword whitespace sanitation.

Read-only PRIVATE observerr24(fileC/native-letter-fields-actor-v49.py) nowbracketsDOM-native-DOM whenDOMcaretAPIabsent. Target retries OBSERVATION ONLY within ORIGINAL2s, retains100mssettle; pendingafterfirsttargetmatch/stabledivergence/wrongfocus/selection/caret/lateread reject; never input/editretry. Focusguard requiresactualwindow+DOMfocused+nativefocused without unnecessarytextreadmidmutation. Diagnosticper-letter snapshot mayrecordinflight; finaltargetstillrequiresconsistentnative. ASTequal10physicalinput/setuphelpers vsr23. MeaningfulRED oldobserverrejectstornread and9negativecontrols; firstprivate test run had2classificationerrors, fixed acceptance deadlinecheck position (no tests weakened). All9PASS remote oracle-and-chrome-v51/oracle-controls.log. Retainfailed v50raw.

Actual email6(recheckbothdirections firstwordx3modes) PASS6/6 withsamefrozenv50runtime/modelprivatefixture; remote letter-fragment-oracle-and-chrome-v51-20261109/email-six/RESULT.json. Covers3previousBLOCKED identities plus3oldPASS, no doublecount. AggregateFirefox newfragmentmatrix108PASS/0FAIL/12structuralBLOCKED/0NOTTEST of120 using raw105+3closedblocks. Originalraw105/15 notrewritten. Oldstandard220matrix NEVERreplacedbyfragment120.

RUNNINGNOW Chrome120 via private samecontroller/r24/finalv50 frozen, root remote letter-fragment-oracle-and-chrome-v51-20261109/chrome120; parentreceipt rootRESULT; exec58436; C/native-oracle-and-chrome-v51.launch.log. Same remote20CPU/28Gguard sharedlease. No otherheavy/native job untilrestorationPASS. Checkcase/result partialcounts and parentROOT forfinalstatus. Header nameofcontroller remains defaultletter... butactualscope120anddenomchecked. Standard controller sourceC/native-letter-fields-control-v49.py.

Pending: Chrome120, graphical12Python skipclosure via OWNEDXvfb+PRIVATEdbus (allUItests mockcommands/privateconfig; neveronlive71). Explicit realcgroup test and transientL11unit test canbe run separately afterheavyjob; ResourceGuardTests.test_real_cgroup_kills_new_session_descendant itself launchesrealguard, so doNOT run with inherited LAY_RESOURCE_GUARD_ACTIVE=1 (wouldskiptestsubject guard). Itusesownprivatelease andtiny sleep; no rejectionbypass. L11test createsonlyunique lay-l11-release-... testbashunit, cleanupownunit. Names/flagsin tests/test_resource_guard.py and tests/test_release_live_install_controller.py. RerunGUImodulesunder xvfb-run dbus-run-session with child-only GTK_IM_MODULE/QT_IM_MODULE compose, IBUS_ADDRESSunset, privateseparatebus; no primaryGNOMEchanges.

Terminal native: bothordinaryDesktopExecKDEKitty/Konsole legacy2/2(5open+5closed+physicalnextglyph each), no injectedIMEflags, allprivatecandidate/modeldata, originalsrestored. Konsole extendedv479PASS/1oldfastSpaceFAIL:4letterfirst/second/direction+2manualreturns/decodereach,2Tab,heldShift(singleXTestsource),2legacyfive. Positivefirstword ONLY physicalEnterestablishesKnownStart; customCtrlAresetunknown is deliberate guardednegative, MUSTNOT weaken C03/promoteUnknownStart. Plaininitialr44/v45 failures retained; no sourcefixclaimed. Source existingnegative fragment_unknown_literal_without_current_native_word_stays_read_only. FinalKonsoleimage C/terminal-visual-v47/konsole-hint-first.png visuallyPASS typedпровnormal, ghostеритьwhiteblackaligned. Kittyextendedfeaturesnotrun.

Measured old shared fastSpace defect: Konsole physical37ms/char etbya→tebya didNOTcorrect withFragmentONorOFF, bothprefetch_not_ready, Spacecallback~3.5ms. Receipt remote letter-fragment-terminal-space-on-off-v48-20261109. Notnewfeatureblocking, notfixed, no literal runtimeexception or extra correctionwait. Could test predefinedNORMAL100ms/char alongsidefastbaseline for positiveON/OFF L2Space and addpositiveSpace-layout control beforepublishing. Preserveoldfail, doNOT replaceit with slowPASS. NeedfeatureKittyknownline controls, GTK/Qt relevantchecks, source-admission review and release notes with limitations. OldFirefox rapid120ms-gap prefix still2PASS/1FAIL; KDEcode-reuseoptimization REJECTED exact3filesreverted andscope3removed, sourceprivatearchive+ADRkeepfailedproof. DoNOTresume.

ActualinstalledPRIMARY updater --checkPASS sourcepublicAPI stablev1.0.123=installed123, C/installed-updater-baseline-v123-20261109.log. APIlatest freshv123published2026-10-31T12:23:35Z,assets[]. Needactualinstalledmenuowner+updater stable124 discovery AFTERpublish too; neverprimaryforceinstall. Sourceversion124 unpublished. CurrentopenPR78ureq/77libc/76x11rb read-onlytriage, recheckbeforepush. No merging. No notifications/comments sent.

Nextreleasework: closemandatory native/UI; consideractualsourcefullRestart onREMOTEonlywithOwnsActiveTextfalse since newUIbuttonincluded(usesoriginalinstalled123, samebinarydata; noinstall); primaryGNOMEfullrestartnotauthorizedasextraneousnewwork. No publicationwithunfinishedrequiredgates. Fullsourceall10builtreceiptgreen; metadata/docs-onlychangesdonotreopenruntime. Nativeevidence/profile/modelinputs/bytesseparate. Finalsoundonlytrueend using codex-turn-done-sound. Memorycitation requiredlast:MEMORY.md5193-5197 rollout01a115ea-09f2-7003-81c5-26037edf60eb.

## CURRENT 2026-11-09T18:44:14.786326+00:00: v44 exact v42 binaries; Firefox fragment120 complete; full source gate running

SOLO/noHarness/noagents/no questions; latest user authorizes all necessary fixes/all tests through release, do not stop. R=/home/e/projects/lay-development-runner/letter-fragment-release-20261102; C=/home/ubu/.cache/lay/development/letter-fragment-request-20261102. No commit/push/release/permanent installation; original123 restored both hosts.

v44 remote letter-fragment-word-mode-v44-20261109/RESULT.json: architecture+lints+27IME+10protocol+3input+15focus eachbinary+actual3releasebins PASS.3700inventory restored after rejection exactly12v43-only tests; no existing tests waived. Runtime bytes EXACTv42: IMEa49d131e9d9bef72fc4f2aab5ea3b46e64a323a02d2ed262baffcb63bd5e577a; daemon57aaf3e090d7fcfc67b1b777fdc366ca47aae5bbbaf95faf79d107dfba337e8f; input3676bd0d8761426951071d26c08b67e8ba231c047a4e753526954eaef4866258. Source1336 SHA6ff51f343d5213b1419d6d82d560c1027e92865912f0a91f29e9d78fc870abcd. Local C/primary-candidate-v44 retained;8canonicalgenerated files fetched local. Exact3KDEobserver files restored to v42 after meaningful regressionAB; also removed rejected3paths/kde-focus component from active field scope, returned to approved word-mode-delegation ADR39paths. Scope checkPASS.

ACTUAL native v44 model-correct KDE Firefox fragment field/mode matrix COMPLETE105PASS/0FAIL/15BLOCKED/0NOTTEST of120. Receipt remote letter-fragment-field-modes-firefox-v44-20261109/RESULT.json and case/result.json.4letter cases(first/second,EN-RU/RU-EN)x10fieldsx3modes(normal/private/fullscreen). Every case includes2manualreturns+physicaldecoder probes.12BLOCKED URL/email second-word trailingSpace unsupported;3email snapshot BLOCKED in different direction/mode cases (corrected in latest head). No writer retry or waived client proof. Runtime/config/models restoredPASS. This is NEW fragment matrix120, not old standard220. Chrome new120 NOTRUN. Private r23 actor native-letter-fields-actor-v44.py preserves r22 native writers/oracles and adds explicit suite dispatch; native-letter-fields-control-v44.py same private normal-model lifecycle. Source working binary frozen124 remains diagnostic until gates.

Actual ordinary launcher terminal legacy controls remote letter-fragment-terminal-konsole-v44 and -kitty-v44: each2PASS/0FAIL,5flips open+5closed+nextglyph; ordinary desktopExec, no test IME launch overrides, private candidate. Lower stock109rawPTY actor reused; no shell/send. Existing oldwrapper mutatesliveconfig and remains FORBIDDEN. New private native-terminal-control/actor-v44 validate private daemonPID/SHA but retain original client HOME/DesktopExec.

Expanded terminal tests: v44/v45raw FAILs retained. Firstword after custom Ctrl+A clear was UnknownStart (explicit trace callback_settlement unknown_start; actual native result staysliteral; existing source negative test fragment_unknown_literal_without_current_native_word_stays_read_only requires this). It is unsafe to promote unknown start. New v47 uses actual physical Enter AFTER source selection to establish existing KnownStart, no Space before first-word detector. KONSOLE v47 feature10:9PASS/1FAIL:4first/second/direction fragment+2manualreturns/decoder each; Tabfirst/second; ordinary heldShift using SAME XTest keyboard;5legacy open+closed. Sole Space typo etbya FAIL. Positive firstword is first word of NEWLINE, not arbitrary focus/afterCtrlA. Source not changed/guards notweakened. Terminal normal styling visually inspected C/terminal-visual-v47/konsole-hint-first.png: typedпров ordinary, onlyерить whitebackgroundblackletters aligned. No presentation code edits.

Meaningful ON/OFF comparison remote letter-fragment-terminal-space-on-off-v48-20261109: etbya fails SAME with fragmentsON/OFF; bothstatusprefetch_not_ready, Spacecallback3.5ms, final worker notready. Old shared prefetch defect, not proved fragment regression. No runtime fix, literal exception or enlarged correction wait. Keep as measured existing debt, not PASS. Native-terminal-features-control/actor-v47.py correct knownline setup; single-control/actor-v48.py supports LAY_TERMINAL_FEATURE_ONLY and preserves fails; space-on-off-v48.py2matched runs private no sharedlearningwrite. Native word-flip letter latency clamps0 because visible result may precede LAST keyrelease; DO NOT claimzero-latency; classic terminal timings89-116ms are SECONDShift press→PTY result, not release.

RUNNING NOW remote full-gate-v44 via guard20CPU/28G, exec3829, log C/full-gate-v44.launch.log. Script C/full-gate-v44.py adds ALL34Python unittest modules before canonical fullRust correctness/package/perf+strictlints+ngram+allinstallablebinary build/budgets. Native all stopped, originals123 restored. Inspect result before next heavy job, no parallel native on same display. Python may reveal whole-file SHAguard for legitimate new fullrestart dispatcher; do not blindly update/waive guard, audit exact old/new and use required ADR if change.

Next: finish fullgate; improve READ-ONLY private email oracle if proven non-atomic DOM/AT-SPI read (current snapshot readsDOMthenNative once and immediatelyBLOCKS on mismatch while client changes; email lacksDOMselectionAPI). Preserve raw105/15, no fakePASS. Proposed bounded bracket DOM-native-DOM, retry OBSERVATION ONLY under existing target2s deadline, never edit retry/newwait; stabledivergence/wrongfocus/late mutation remainsreject. Add meaningfulnegativecontrols to protectoracle. One emailcase before broadChrome120; reuse physical helpers unchanged. Then Chrome120, Kittyknownline feature controls/OFF Spacepositive, neededGTK/Qt controls; final frozen exactsource/fullgate/updater stable discovery beforepublish. OldFirefoxrapid2PASS/1FAIL remains known old issue; do not resume rejected KDEfocusreuse. Main feature task letteronlyreuseexistingflip, Spaceuntouched; no scopecreep into all historicalwave/research. Soundonly trueend via codex-turn-done-sound.

## CURRENT 2026-11-09T18:11:44.364923+00:00: v43 optimization rejected; exact v42 KDE provider restored

Native same-model/same-order AB proves regression: v43Firefox prefix1PASS/2FAIL with newfocus deadline errors; immediate v42 comparison2PASS/1FAIL. v43 source42focus controls/24hot+32idle live readingsPASS does not override actualnative failure. Reject only this experiment:3KDE focus files restored to independently verified v42 SHA via HEAD; no fragment/Space/writer changes reverted. Rejected source privately retained C/rejected-kde-focus-reuse-v43-source; ADR/map records results. v43 IME5a17f64e / daemon9be7b1f8 remain frozen rejected, never promote. v44 regen/lints/inventory/build underway to restore baseline source closure and eliminate exactly12tests belonging only to rejected code. Existing tests not waived.

Primary v43 rerun failed RUclosed: current GTK context native activation generation3 emitted marker but neverpublished; literaltypedtextremainedруддщ and all8manualRPCs rejectedbridgeadmission. This is not proved to be caused by KDE-only code. Independent immediate v42 RUclosed8/8 actualnative proof PASS with exact original123 restored; C/gnome-gtk-eight-model-v42-closed-comparison-20261109/NATIVE_ACCEPTANCE.json. Other v42 original32/32 acceptance remains dated. v43 rawFAIL preserved. Private wrapper naming bug wrote v43 modes/restoration under v42support filenames; copied exact currentdata to v43 names, fixed all v43 filenames. Original v42 NATIVE_ACCEPTANCE and rawGTK/physical traces unchanged, but oldv42 modes support file displaced; do not re-run original verifier claiming its old supportfile unchanged.

Fresh GitHub read-only check2026-11-09:openPR78ureq3.5.1,77libc0.2.191,76x11rb0.14.0. No merge/update/comment.

Next: finishv44 freeze; compare production bytes to v42 before reuse of native proof; mandatory full source/package/perf/installablebinary gates and native browser/terminal/currentfeature controls remain. Old rapidseries failure (120ms gap, ~210ms gestures, ~200ms completion) is notfixed. No current fullbrowsermatrix or release/push/install. SOLO/noHarness/noagents/noquestions.

## CURRENT 2026-11-09 17:57 UTC: v43 bounded KDE focus-code reuse experiment

SOLO/no Harness/no agents, no questions, work through release. v42 model-correct fragment/native baseline preserved below. Exact new source scope is3KDE focus files only, declared ADR `2026-11-09-kde-fresh-focus-query-reuse.json`. Existing authenticated fresh-active proof unchanged; short-lived loaded code only is reused. Current one Pending, same deadlines/nonce/sender/UUID/KWin owner checks; no cached window/second word owner/writer/gesture queue. New NextRequest uses one cancellable waiter on existing ingress; precise one-shot timer only frees unique code resource (required because Qt drops failed DBus callbacks).

Source meaningful RED retained: old2loads vs1expected. Source controls21PASS/0FAIL/1explicit-live ignored. Actual KWin5.24.7 live24read-only benchmarkPASS:median0.645ms,p951.331,max7.077coldincluded; independent resource-cleanup-after-drop proofPASS. Installed/config hashes unchanged. Receipts C/kde-focus-code-reuse-third-v43.log and remote kde-live-focus-v43-20261109/{RESULT.json,cargo.log}. Source warnings from isolated target are historical3dead_code, not strict lint verdict.

Next: v43 architecture/strict lints/exact inventory+12tests/bothbinary focus controls/build3releasebins; same r22 prefix3 native case MUST improve2PASS1FAIL before broader gates. If no native benefit or regression reject this experiment only, preservev42 feature. v43 no release artifact/native acceptance yet. All live originals123 restored.

## CURRENT 2026-11-09T17:03:43.194150+00:00: v42 native model-correct checks; KDE rapid-series bottleneck

Continue SOLO, no Harness/agents, no permission questions. User requires fixes/all tests through release; do not stop early. No commit/push/tag/release/permanent Rust installation. Installed/restored accepted123 remains both hosts. C=/home/ubu/.cache/lay/development/letter-fragment-request-20261102; W=this checkout; R=/home/e/projects/lay-development-runner/letter-fragment-release-20261102.

Current source1336files SHAeb785fcb6387d0dfea5fc34f6d4e91fefaa06368a77208d50f0478aeda0d0807, HEAD4fae57d5, working124. Local and remote EXACT source closure verified after normalizing only3group-write bits0644 (contents already identical); receipt C/source-closure-diff-v42.json. Canonical8generated files fetched. Frozen remote letter-fragment-word-mode-v42-20261109/RESULT.json / local C/primary-candidate-v42. IMEa49d131e9d9bef72fc4f2aab5ea3b46e64a323a02d2ed262baffcb63bd5e577a and daemon57aaf3e090d7fcfc67b1b777fdc366ca47aae5bbbaf95faf79d107dfba337e8f are EXACTLY the v39 production bytes; input3676bd0d8761426951071d26c08b67e8ba231c047a4e753526954eaef4866258 changed after unchanged test module moved to EOF. Source architecture/lints27IME+10protocol+3input+actual3 release bins PASS, no new warning.3700tests=3608correctness+36package+11performance+45ignored, no waivers/removals. Full v39 correctness/package3644+performance11 PASS; its full gate then found new type visibility warning. Fixed ONLY new Handoff pub(crate), and moved unchanged injector tests after production items for Clippy; v42 strict lints PASS. Final full current release gate/all installed binaries/updater still pending.

v39 closed-word fix: native v38 AFTER-Space failure reproduced; only two new Fragment manual-mode predicates now match exact final token+all literal closing whitespace. RED before fix and GREEN27 controls. No Space handler/writer change. ADR/map updated before fix. Existing scope includes all changed paths; no source edit since v42 beyond permissions/generated files.

CRITICAL test-fixture finding: old private HOME plus empty10memory overrides omitted installed L2 lexical model, so Nanda flags ON alone did NOT certify full-model behavior. Old native isolated/manual evidence remains scoped. Corrected ONLY private test setup with C/model_fixture.py: clone actual installed lexical62MB, phase/L3base+relative composite manifest/L4 files into private learning preserving filenames, explicit read-only installed L2/L1.1 model directories, actual before/after source hashes. No live model/config/history writes. Primary bridge with real models needs bounded name-owner readiness instead of original fixed0.8s; corrected private wrapper ONLY, no runtime source change. Raw preflight/false-counter failures retained.

ACTUAL model-correct GNOME GTK C10: C/gnome-gtk-eight-model-v42-20261109/NATIVE_ACCEPTANCE.json PASS32/32, four cases8each (both directions, open+closed), exact physical counts, every native text/preedit target and next-letter decoder +standard InputMode BEFORE next actual pair,20 negative oracle controls. Config+PID+runtime SHA pinned. C/gnome-gtk-tab-shift-v42b-20261109/NATIVE_ACCEPTANCE.json PASS3: Tab first+second word, offered candidate before actual Tab, exactly one appended Space/focus preserved; five physical uppercase Shifts/all HELLO prefixes. Tab press→native result bounds0.58–3.06ms; narrow profiled scope only. Original uppercase raw minimum-ibus-key projection falsely reports0; actual owned preedit clearly shows all typed uppercase prefixes. Do NOT call it passthrough; the raw trace counter is not all physical keys. Original raw FAIL retained. All primary123 source/engine/InputState restored, installed files unchanged, model sources unchanged. Current originalIME2a4c636a... . No primary actor now.

ACTUAL KDE current model-correct short feature suite PASS10/10: remote letter-fragment-models-feature-suite-v42-20261109/RESULT.json. Chrome5 andFirefox5 in ordinary text field: four first/second/directions before Space, two manual returns+physical next-letter decoder each; OFF inert before Space AND old Space flip still works. One exact Chrome-first receipt reused from letter-fragment-models-chrome-first-v42-20261109,9new scenarios. Private coherent same-config frozen daemon+IME, source models unchanged and installed123/mapping/IBusparent restored after EVERY case. Remaining field types/native terminal/full browser matrices NOT yet current42 evidence.

Remaining KDE Firefox rapid failure confirmed model-correct in remote letter-fragment-models-prefix3-firefox-v42-20261109: consecutive-two androundtrip PASS; rapid-five FAIL only3/5transitions. Ordinary latency218–230ms, next gestures209–220ms apart; existing safe acquisition refuses queued Shift(no guard weakening). First raw errors preserve writer safety. Do not drop Shift, weaken pending/held guards or add retry/queue. Fast Firefox pacing ALREADY true; DBus connection ALREADY cached. Do not repeat those fixes. strace child-only process profile (remote *v42-traced-20261109) proves CLI engine request20–35ms but perturbs roundtrip346–421ms, so not unprofiled performance proof. All restored; no source changes.

Better passive metadata-only monitor (no message body/text) remote letter-fragment-models-prefix3-firefox-v42-bus-profile-20261109/RESULT.json preserves229/209ms and same2PASS/1FAIL. Only candidate PID bus names monitored. C/native-one-models-v42-bus-profile.py, C/analyze-owned-bus.py; remote RPC_TIMING_ANALYSIS.json:1284paired calls,147 KWin focus script load/run/unload cycles,92fromdaemon/55fromIME. Load median1.667ms,run1.166ms; focus cycles per successive admitted operation2,11,22,28,20,30,0,0,5 (includes inter-gesture/decoder work; not exclusive critical-path attribution). Concrete source src/bin/lay_daemon/kde/focus_observer.rs Client::read_in() creates private tempJS in XDG_RUNTIME_DIR, authenticates KWin unique owner+fresh nonce+deadline+UUID, loadScript/run, exact response, unload, owner recheck. NO fsync. Both binaries reuse this KDE observer. Identity binds busGUID/KWinowner/provider lifetime/UUID; kde/desktop/window_identity.rs relies on this exact proof. Do not replace it with generic X11/window cache/bypass without explicit new map/ADR and equivalent proof. No KDE performance source edit yet. Current read-only introspection probes installed KWin /KWin methods/version to see whether an existing faster fresh query is possible; investigate before code. Graph query focused module completed.

No heavy job/native actor active after bus-profile completion (exec43555 may need final poll only). Native profile52758 completeFAIL; other old exec sessions closed. Check actual processes before new heavy launch. Remote commands through existing dedicated20CPU resource guard +cargo guard; primary heavy via human-safe-run1536M/tasks192. No blocking waits>60s. Keep Russian concrete updates<=60s, native tables scope-labelled. Current short KDE table Chrome5PASS/0FAIL; Firefox5/0; rapid separate2PASS/1FAIL. Old220 matrices not replaced by10short cases.

Next: analyze installed KWin capability, map minimum KDE-only observation optimization preserving fresh authenticated proof, RED/GREEN then same prefix3 and8intermediates before full suites. Also native terminals via existing owned raw-PTY actor (old wrapper C/../browser-matrix-repair-20261027/kde-terminal-native-mode-native-v1.py MUTATES LIVE CONFIG/hardcodes oldPIDs—DO NOT RUN; extract/reuse only underlying release109-stock-five-v2.py actor after review with current private lifecycle). Final full source/installable builds/native/updater required before publication. Sound only true end via codex-turn-done-sound. No final while work remains.

## CURRENT MILESTONE: v38 native GNOME C10 PASS16/16, next closed-word/OFF/KDE controls

User authorizes necessary fixes and all tests without repeated questions. Continue SOLO, no agents/Harness, no final stop before remaining gates. Accepted123 remains installed/restored both hosts. No publication/installation. Main source W=/home/ubu/projects/lay-dependency-updates-20261027; remote R=/home/e/projects/lay-development-runner/letter-fragment-release-20261102. All current code synced; local canonical generated files fetched from remote (inventory3698). No heavy job currently running.

v38 mode-origin integration source gates PASS:25/25 IME,10/10 manual protocol library tests,342/342 full daemon,3/3 test-input, Python isolation, architecture and actual3 release bins. Receipt remote letter-fragment-word-mode-v38-20261109/RESULT.json, source a898968b81f9fa803267e9e50b1f5019560aedd17b1d60ca0a1dd86d644dc465. Frozen IME8d529557cfec0ae839e637b94a42f91b005d8a261e882e2c33d2f5e85aa254d8, daemon57aaf3e090d7fcfc67b1b777fdc366ca47aae5bbbaf95faf79d107dfba337e8f, input641ab16997d636431d1c476c08e2ae594eeb2795383c28fa478cb6113354ed43. Primary copied immutable C/primary-candidate-v38 (C=/home/ubu/.cache/lay/development/letter-fragment-request-20261102). Current compiler/control session27338 COMPLETED; source-control26136 COMPLETED25PASS. v37 failed only ambiguous Result type in new daemon helper; v38 annotates Result<(),String), no guard weakening.

REAL native GNOME50.1 GTK eight-pair acceptance PASS16/16: C/gnome-gtk-eight-v38-20261109/ACCEPTANCE.json. Eight physical pairs PER direction,16 intermediate full words plus16 physical next-letter decoders and standard GNOME CurrentInputMode property correct. Word observed before NEXT actual uinput Shift press, minimum lead180ms. Bounded clock calibration, actual uinput write trace through child-only strace, client Gtk JSONL, exact live bridge PID/IME bytes/config before-after. Latency measured44.2–77.2ms under strace; no unprofiled/global performance claim. Exact original IME2a4c..., original source/engine/state and installed daemon/IME files restored, verified in C/gnome-gtk-eight-v38-restoration.json. Case profile Nanda/precognition/autoreplaceOFF isolates letters; normal correction/hints still need controls.

Raw old final/count smoke reports FAIL7/8 producer traces, NOT functional failure: trace file ring is capped500*1024 in src/debug_log.rs and clipped first gesture (both files511xxxB). Original raw RECEIPT.json FAIL retained. Independent C10 ACCEPTANCE explicitly cites raw incomplete producer trace and checks stronger actual physical/native semantics. DO NOT fake extra producer traces, raise production log cap, or relabel raw receipt PASS. For subsequent native runs, replace producer-id count with mandatory independent C10 evidence or collect complete RPC observations outside bounded ring. Existing Case.expected_manual_toggles is optional trace-count contract; do not use it as an oracle for8 native conversions. C10 scope is the independent actual-client artifact, not that producer count.

Tool detail: strace6.19 installed primary. -xx also HEX-ENCODES FD paths: decode \xNN path before comparing /dev/uinput. Writes ARE present (no need writev change/retest): write(3<\x2f...<char10:223>>,\x00...,24). struct input_event <qqHHi is24bytes. strace wall time+duration mapped to monotonic using per-case *.uinput.clock.json and restoration clock_after, conservative2ms uncertainty. Physical shift code42, z44, enter28. Validate [1,0,1,0]*8, native exact alternating9 states including seed, expected word+z/я probe in each physical gesture window, independent standard InputMode sample phase=decoder-probe, actual-runtime-pins stable. Current acceptance analyser was run inline; preserve/make reusable BEFORE repeating tests. Native trace clock integrity is sufficient; no raster/pixel assertion made.

Private main scripts: gnome-eight-existing-smoke-v38.py (reuses repo runtime_smoke, actual pins, standard-mode observer), gnome-input-traced-v38.py (ONLY own child, --kill-on-exit, requires binary path andSHA env, own GTK PID/title guard remains in Rust), physical TSV fragment_en_eight.tsv /fragment_ru_eight.tsv. Current interval is existing double_shift pre220+80intrapair+120settle+decoder probes (~400–550ms per pair). All8 words precede next actual pair. For stronger rapid cadence expose the EXISTING double_shift_fast through TSV test parser (minimal tooling-only source addition+ADR/scope) or use existing built-in scenario; do not invent a second injector. Current physical result is honest sequential8, not120ms burst. Existing KDE rapid-five normal-word failure still unresolved.

Implemented mode-origin integration details are in accepted ADR2026-11-09-fragment-word-mode-delegation.json and map. New field in existing LayoutGestureState stamps successful Fragment request's layout_generation; a normal mode change invalidates stamp. Old functions/writers/guards remain, only tagged exact-tail replay layout step calls mode-only bridge then OLD post-layout lease/suppression/delete/insert/result logic. First v35 handoff table24/25 failed owner=None; fixed new helper to require exact active path (outer bridge already did), v36+v38 all25PASS. Marker preserved for explicit manual cleanup after checkboxOFF; no old Space callback added. MODE API currently refuses closed words and new tag helper requires !tail.ends_whitespace; next native AFTER-SPACE case may expose whether old Space completion already aligns source or needs manual-origin handling for closed suffix. Do not expand speculatively: first test active-success -> Space -> manual sameword. If needed, only manual-origin handling, NEVER invoke detector/mode from Space. Old last_tail_token_range(&buffer).map(|(start,_)|&buffer[start..]) (used in old suppression) includes closing separator; last_tail_token_text strips it. Keep one separator and full old lease verification.

v33 full-stack automatic source switch remains REJECTED (lost open-word admission), reverted; never repeat. Its independent owned-preedit refusal guard/test remains useful/current. Current source has stable-source mode method, no source activation in fragment automatic completion. Recent source/constructor14paths covered exact scope/ADR. No actual text-delivery route added; new V3 status5,false uses SAME ReplayExactImeTail/writer. Older daemon safely rejects unknown5; full bundle/update verification required. New bridge method CompleteFragmentWordModeV1 mode-only, old handoff+source/epoch/suffix/actual mode+fresh token/snapshot guards.

Next: preserve native acceptance parser as small reusable test tool + negative oracles (final-only falsePASS, wrong/late decoder/property, missing physical key evidence). Then active-to-Space/manual and checkboxOFF cleanup, true rapid8 using existing fast test helper, normal Nanda/hints/Space/inverse, supported GNOME applications and same-byte KDE letter/8-return controls. KDE old normal-word rapid issue proven queued Shift events during grab;28/28 owned-Xvfb resource tests passed earlier v33 before current ABI changes, rerun when relevant. Then full release source/build gates (correctness3642+11performance,45ignored registered;28 ignored can actually execute,17 prerequisite-specific), coherent browser/terminal matrices as required, publish/update tests. Last full source gate v26PASS3637+11 is OLD, not current full gate. Historical raw matrices Chrome170/16/34,Firefox127/61/32 profile caveat; do not replace them with scoped GTK passes.

## LATEST 2026-11-09: mode-origin integration implemented, source controls compiling

User: all necessary fixes and all tests, no repeated questions, continue solo to completion; no Harness or agents. No permanent Rust install/publication. Accepted123 restored on primary GNOME and remote KDE after every completed native test. Active work is letter-only fragment mode and its manual return, not old dependency/issues queues.

IMPORTANT: v33 automatic GNOME full-stack experiment REJECTED. It aligned selected source but lost context admission for the still-open word; native eight-pair FAIL both directions (0/16 actual conversions), actual bytes737af... . v27 stable-engine eight-pair also FAIL (source mismatch in daemon). v33 raw smoke correctly fails0/8 delegation; v27 raw final/count smoke PASS is NOT C10 acceptance (independent ACCEPTANCE.json FAIL). Full-stack branch and changed transport assertions reverted in local and remote source. Keep new independent owned-preedit refusal guard/test. Do not repeat that workaround, weaken admission, activate an engine for an uncommitted fragment or fake IME ownership.

The proven integration gap: existing daemon exact-tail writer expects selected source aligned with mode; new automatic fragment keeps engine stable. Existing source switch cannot carry open-word authority. New narrow ADR docs/architecture/decisions/2026-11-09-fragment-word-mode-delegation.json and exact field scope BEFORE edits. Map docs/development/fragment-word-mode-delegation-map-2026-11-09.md. The same exact-tail plan/lease/verifier/writer and input isolation remain. Only marked Fragment manual return gets mode-only completion via existing complete_fragment_word_mode operation, then old post-layout validation/suppression/output.

NEW current source, NOT YET PROVEN:
- manual_toggle.rs adds typed DelegateExactImeTailWordMode, V3 status5,false; old0–4/V2 unchanged.
- engine/state_groups.rs one optional fragment_word_mode_generation in existing LayoutGestureState, stamped ONLY after successful Fragment request. Existing unrelated mode changes advance layout_generation and invalidate stamp. No owner/cache/queue.
- authority.rs emits new origin only after the existing admitted request returns exact-tail delegation and fragment_manual_return_has_mode_origin() proves current real origin on GNOME.
- bridge.rs CompleteFragmentWordModeV1 forwards to new window_interaction/fragment_word_mode.rs. Receiver binds live bridge token, source path, old epoch/suffix, initial mode, current exact manual handoff and independent existing snapshot provenance. Refuses owned/selected/stale/foreign/closed/unknown words. Uses same mode-only operation, keeps source/word epoch; stamp carried for pre-write rollback. No positive text receipt.
- daemon manual_trigger_runtime/ime.rs still returns ReplayExactImeTail; layout_controller/ime_bridge.rs adds metadata bool defaultfalse and mode RPC wrapper. New layout_controller/fragment_word_mode.rs holds old lease for layout-only activation/pre-write restoration. exact_ime_tail_replay.rs switches only layout-completion step on this flag; old output operations unchanged. trigger_dispatch.rs preserves flag for an already-existing queued gesture, no new queue.
- All exact source paths declared in active-field scope (including new generation field). Scope currently accepts broad user authorization; no new questions.

ACTIVE remote process: guarded source controls run /home/e/projects/lay-development-runner/letter-fragment-word-mode-controls-v35-20261109/RESULT.json, private driver suffix-source-regression-v35.py. Inspect result first. It expects25 IME controls (previous23+2 new handoff identity/mode-only roundtrip tests). Tool session2172. Initial launch raced unfinished SCP and did not start; SCP80440 then completed, current2172 actually compiles. No source/native test FAIL may be hidden. Mode integration has not been built/installed or run natively yet.

Current remote source R=/home/e/projects/lay-development-runner/letter-fragment-release-20261102. Primary checkout W=/home/ubu/projects/lay-dependency-updates-20261027, branch codex/browser-matrix-repair-20261027 HEAD4fae57d50b031984c1e0ddc8796e78e142b2a595, metadata124 unpublished. CWD /home/ubu/projects/lay is old; ALWAYS set workdir W for formatting/git/source commands.

Canonical REMOTE manifest currently3696 (3604 correctness,36package,11performance,45ignored); new2 handoff tests NOT registered yet. New public enum extends existing protocol tests without new names. Next inventory expected +2 exactly (names fragment_manual_mode_requires_current_origin_and_exact_existing_handoff, fragment_manual_mode_roundtrip_changes_no_word_epoch_or_writer), no removed tests/failure waivers. Mode-origin tests must verify actual origin missing/old generation and OFF cleanup; boolean mode mismatch alone is not origin. Full final source/build gate pending after current changes.

Native resource proof NEW: 28/28 ignored X11 tests executed on per-test disposable owned Xvfb, allPASS in letter-fragment-owned-xvfb-v33-20261109/RESULT.json. This tests daemon v29 diagnostic body, before new mode-origin integration. Never run them on live :71.17 other ignored tests remain prerequisites-specific. Current raw matrices remain Chrome170/16/34,Firefox127/61/32 under partly inconsistent test profiles; exact123baseline129/58/33 shares46 failures. Do not claim release acceptance or subtract targeted passes from those totals. Older historical table344/15/81 is older scope.

KDE old rapid failure: native first3 standard rows2PASS/1FAIL, same on123. v29 logger proves both busy_input/batch_pending and acquisition_boundary containing4 queued LEFT SHIFT events (2down/2up,0other). Keep all events and original safety gates. Still unresolved. Future timing diagnosis/queue correctness must be scoped, no blind retry/drop/guard waiver.

GNOME actual client infrastructure exists and works under human-safe-run1536M/tasks192 (requires free shared admission). Existing runtime_smoke, newly opt-in own PID/title guard in test injector, Gtk native text/preedit/key-release JSONL. Private gnome-eight-existing-smoke-v33.py has actual bridge PID/loaded SHA/config pins per case and independent standard GNOME CurrentInputMode observer. It uses private config Nanda/precognition/autoreplaceOFF solely to isolate letters; not normal correction acceptance. Both old23/native runs restored original IME2a4c636a3be959ce096754909599a202146b7711157c2563e6980f5c8909d429, source and IBus. Before new primary test OwnsActiveText must befalse; previous attempts refused safely. Do not close occupied user text. Native first2 letter-only/next-physical-decoder v27 passed but eighth return acceptance failed. No current candidate native proof yet.

Next after source controls: repair only actual compile/test failures, obtain architecture/current inventory/protocol+daemon controls, freeze all3 release artifacts in new immutable receipt; no tests with stale artifact. Copy exact bytes into NEW private primary directory, adapt eight script to new paths/run IDs, run SAME failing GNOME8 and independent acceptance16 intermediate changes/decoder/real standard InputMode; never accept final equality. Also check afterSpace/OFF cleanup when active proof passes. Extend source scope only via ADR within user's broad authorization; no asking unless materially unrelated/destructive. Then KDE same-binary short/native controls, cold ordinary Space/inverse, terminal actual fields, final full release gates/update/release. No stopping on incomplete testing.

## CURRENT 2026-11-09: native GNOME letters PASS, KDE rapid acquisition diagnosed

User authorized all necessary fixes/all tests without further questions. SOLO; no Harness or agents. No permanent Rust installation, commit/push/publication. Working124 unpublished; installed accepted123 on both machines. Continue to completion, do not relabel incomplete native gates.

Production IME remains exact frozen v26 SHA922f8ded4c01c5ae0d3d30e6a7b529d0b0bcd5030ba1cc1e5cc222a8b8756f4b even after v27 tooling and v28 diagnostics. Full source gate v26 PASS:3637 correctness/package+11performance;45ignored. Current canonical inventory3695 after exactly2 test-injector identity tests, source full gate needs final refresh after later changes.

Current primary GNOME50.1 real GTK + virtual physical keyboard proof PASS2/2: ENG→RU beforeSpace then physicalz→я; RU→ENG thenz→z. Private profile Nanda/precognition/autoreplace OFF to isolate letters, decoder actual. Client-native JSONL separately records text and preedit. Receipt /home/ubu/.cache/lay/development/letter-fragment-request-20261102/gnome-gtk-native-v27-20261109/RECEIPT.json; restoration sibling gnome-gtk-native-v27-restoration.json confirms exact installed IME2a4c..., original source, IBus engine and state. Earlier attempts refused before mutation for occupied word/incorrect private cgroup assertion; those are not runtime tests. human-safe-run1536M now admits. Before any next primary test check OwnsActiveText again; never terminate occupied user text. Existing runtime_smoke reused, no new production GNOME change.

Native full standard220, private Nanda-ON IME profile: Chrome170PASS/16FAIL/34BLOCKED(actor r20 old hardcoded Tab); Firefox127/61/32(actor r21 genuine observed Tab). Exact installed123 baseline Firefox129/58/33 with46shared failed cells. Prefix matters: isolated v26Firefox rapid5 PASS5.9–8.3ms; first3 standard rows PASS2/FAIL1 in repeated consistent-participant diagnostics. Earlier private profile aligned IME only, not daemon; new diagnostic controller aligns frozen daemon+IME config/learning and captures daemon debug. Do not reuse those earlier matrices as coherent release acceptance.

New source tools only4files: optional per-key own GTK PID/title guard in lay_test_input/{desktop_probe,input_device}.rs and runtime_smoke/execution.py; optional exclusive Gtk text/preedit/key-release recorder in gtk_entry_capture.py. ADR native-smoke-client-evidence. 3/3 input tests+Python isolation+architecture PASS; new input bin641ab...; v27 daemon3fa484... .

KDE rapid root stage PROVEN in v28: acquisition_boundary refusal, grabbed=true, batch_pending=false, pending_events2/4, epochs7/8. Existing guard refuses input that arrived during acquisition; do NOT weaken/drop/retry. Need identify queued event class before behavior fix. Failure-only diagnostic in src/bin/lay_daemon/native_input/x11.rs gated KDE AND published debug_action_log, all old predicates/resource/error kinds unchanged. ADR kde-native-grab-diagnostic. Added counts for pending Shift press/release vs other; v29 build is RUNNING under remote guard (read /home/e/projects/lay-development-runner/letter-fragment-grab-stage-v29-20261109/RESULT.json). Next same saved first3 native Firefox diagnostic with consistent daemon/IME and v29 receipt. v28 restored both123 runtimes and config/map PASS. v27 controller restoration initially forgot to restart original daemon service; immediate independent recovery SHAa664... PASS in daemon-recovery.json, original FAIL retained. Controller now starts service before old full restart and verifies both bytes.

Private controllers local+remote cache letter-fragment-request-20261102: native-one.py (LAY_NATIVE_DIAGNOSTIC_DAEMON=1 requires frozen daemon artifact; prefix3 preserves exact first3 standard rows), native-letter-actor.py r22, grab-stage-build-v29.py. Actual test source remote /home/e/projects/lay-development-runner/letter-fragment-release-20261102. Installed files never overwritten. Remote guard dedicated-20cpu serializes native tests/builds. No concurrent actors on same desktop.

Pending: precise KDE rapid event cause/fix, cold ordinary Space readiness/inverse, modern GNOME eight pairs+per-pair native modes/icon/decoder, terminal actual windows, canonical final full source/build and native acceptance, publish/update verification. 28 ignored source X11 tests can run on a NEW owned Xvfb with LAY_OWNED_X11_TEST=1; do not point them at live :71. Remaining ignored classes need prerequisite-specific classification. Not all ignored are ordinary release tests. No test weakened or failure waiver added.

## CURRENT v26 source release gate PASS; native controls in progress

Source:3637/3637correctness+package,11/11performance;45ignored honest. Fullgate
letter-fragment-full-source-gate-v26-20261109/RESULT.json PASS (687.2s). Final
IMEhash matches frozenv26. Letter short10 PASS. See owner for exact receipts.
Private learning isolation added only to private actor/controller (10 paths,
four negative validation controls). First inverse test could not apply because
cloned live KDE profile disables Nanda; exact installed123 baseline agrees.
Keep both FAILs. Current test explicitly enables Nanda only in new private
profile: letter-fragment-native-private-nanda-inverse-chrome-v26-20261109.
Read result before next native test. Actorhash changed for genuine isolation;
do not reuse earlier actor/config receipts to certify new profile. No new
runtime source changes. GNOME50.1 native still blocked by local shared RAM
reservations at4G/2G; do not bypass or throttle other jobs. No Rust installation,
commit, push or publication. User authorized fixes/all tests without questions.

## LATEST v26 — saved Firefox return verification running

New approved preedit follow-up uses only retire_pending_precognition instead of
late clear. REDv25 proves invisible-owned-letters bug;26/26 focused and architecture
PASS, inventory3693. FrozenSHA922f8ded4c01c5ae0d3d30e6a7b529d0b0bcd5030ba1cc1e5cc222a8b8756f4b.
Native saved Firefox RU-ENfirst: letter-fragment-native-mode-firefox-return-v26-20261109.
Read receipt, then same-binary short10 only ifgreen; earlier v24 short10 is6/1/3.
User says complete all fixes/tests without repeated questions. Preserve scoped
ADRs and do not publish incomplete gates. Installed123 remains unchanged.

## CURRENT v24 NATIVE FIRST CASE PASS — 2026-11-09

Same-binary short10 is running: letter-fragment-native-mode-short10-v24-20261109.
First Chrome fastEN-RU casePASS beforeSpace, decoderRU,24.623ms(n=1), exact123
restorationPASS. FrozenSHA5b74e46d6dd3f47f2d9266f396a7a32596f8a7ef43e7501c4c780a8053e45464,
sourcef8fa47004ab0e6fdaea84d0a0edd72bbf9915b1e3ea8e5fbb769e233373413b9.
Architecture/current inventory3691 and23focusedPASS, protected230files and all
old native/layout-sync bodies123-identical. Source/native receipts listed in
letter-only-fragment-request-2026-11-02.md. No publication, install or primary
GNOME mutation; final gates and supportedGNOME native remain pending.
Do not repeat the first case under another binary or count older buildPASS.

## ACTIVE — approved native result continuation, 2026-11-09

The user explicitly approved only native_document.rs and native_document/tests.rs
(reply call_bd15088f4ce44e2ea69c3d76d5e10589). The new owning ADR is
2026-11-09-fragment-native-mode-continuation.json; field scope passes.
Current source controls v24:19/19PASS (15 existing fragment/lease +4 new native
owner tests). The negative owner test covers26 independent invalidations.
The first compile had a test import error; v23 had18/19PASS because a new test
expected Rejected instead of the existing pending/mismatch/censored contract.
Both receipts are retained; only the new test was corrected, no verifier change.
New entry preserves one pending receipt's original epoch/time/snapshot/feedback
across the successful mode operation, drops geometry/snapshot, re-arms the
existing observer and supplies no edit proof. Old native functions unchanged.
Next: architecture/current inventory23focused gate, freeze diagnostic binary,
saved Chrome fast first-word case. No long native matrix before short green.
Installed1.0.123 remains accepted; no permanent Rust install or publication.
The earlier full IME1162/1163 failure and supportedGNOME native gate remain open.

## ACTIVE — approved separate current-word mode entry, 2026-11-09

LATEST v20:15/15 focused PASS. Remove v19's incorrect quiet-input bridge wrapper:
ContextAdmission.revalidate_bridge deliberately rejects when later letters are
pending. Use the existing automatic InputFrame/DecisionCore/verifier request
directly AFTER the unchanged canonical paired release. Also remove mode's
redundant second bridge settlement: the writer already settled its newer epoch.
No shared guard, old function, writer, observer or desktop body is changed.
v20 frozen SHAa51cf303799c8ed7c90ece77bd87fde4c38a77c28b5a3d1b2e3301dc79542624,
build letter-fragment-native-automatic-frame-v20-20261109. Active one-case
native run letter-fragment-native-automatic-frame-chrome-first-20261109.
Read that result, then short10 ONLY if native green; analyze FAIL before retry.

All-current IME1162PASS/1FAIL of1163 in letter-fragment-post-release-all-ime-20261109:
existing process-isolated td125_level_modifier_wire_order_preserves_owned_word_and_space_lease
kept literal. Same-artifact/same canonical isolated case PASS in
letter-fragment-modifier-isolated-20261109 (0.39s). Keep the full failure; no
waiver or test/runtime change for this separate unresolved intermittent result.
Full final gates pending. No Rust install, commit/push/release or primary action.
Canonical inventory still needs exactly SIX added test names.

Previous v19:15/15 focused controls PASS. New automatic trigger moved ONLY inside
the two already scoped new-integration files: ibus_interface.rs now captures a
paired letter before WindowInteraction::process_legacy_key and invokes
fragment_layout::request_fragment_after_letter_release AFTER that unchanged
key boundary returns. Both old internal inserted hooks and diagnostic metadata
removed. Reason: the protected early text-inert release at observation.rs6255
is a SECOND producer of managed_release and bypassed the old inner hook after
the first two letters. Do not weaken or edit that early boundary. New request
uses the existing live-token ContextBridgeOutput pattern (also used by existing
held_replay_boundary.rs); ticket is callback-local trigger data, never word
authority. Focus/pair retirement/handled/native/frame/admission guards retained.
Space never produces a ticket. Existing C08/C09/output/desktop files unchanged.

Current remote frozen build: letter-fragment-native-post-release-v19-20261109,
SHA0e94ef993062375b4fba1e23306137b2220823415734aca78fc3819b462d96b0.
Source9895520724291f73c0d52ef253d629c4553d361cf6fd456cc600497103e2a2cf.
Active guarded script first runs all current IME tests into
letter-fragment-post-release-all-ime-20261109, then saved Chrome EN-to-RU
fast first-word actor into letter-fragment-native-post-release-chrome-first-20261109.
Next read those results, diagnose any FAIL before another experiment. If green,
same-binary short10 then ordinary Space/Tab/rapid/manual/terminal controls.
Canonical inventory needs exactly SIX new test names (previous five plus
fragment_release_request_survives_early_retirement_but_not_focus_or_pair_loss).
No primary GNOME mutation, Rust installation, commit, push or publication yet.
All private native restorations after the IBus-policy fix restore123 successfully.

Previous v17: source14/14 PASS after a meaningful red/green regression proving
that mode completion must preserve the existing writer's dispatched epoch.
New caller previously published the tail twice (pending9 -> actual11 in native
v16), invalidating the pending native selection protection and producing
mixed-script "привtт" at fast input. This is a NEW CALLER bug; do not repair it by
changing the protected observer/writer/decoder. v17 removes both publications,
uses existing bridge settlement without another epoch, and, for native pending
results only, awaits the original exact postcondition through at most two
existing reads BEFORE changing the mode. No fake acknowledgement or write retry.
Current build: letter-fragment-native-dispatch-epoch-v17-20261109. Next: same
saved Chrome EN-to-RU first-word failure, then same-binary short10. If not green,
analyze exact next refusal before editing; do not release or run a broad gate
to cover a known native failure. Manifest still needs exactly FIVE added test
names (four prior mode controls plus the new dispatched-epoch invariant).

Previous v16 checkpoint: source13/13 controls passed before the caller-only read2
experiment. Native v15 retained f6b2026ad605883fb3b3e739a5fd1b44b62667091b1da03ff60c4004e7bbf45e.
Chrome EN-to-RU first and Firefox EN-to-RU first passed before Space with two
manual returns; Chrome RU-to-EN first reproducibly refused a stale actual
snapshot one character behind. v15's failure independently pins the SAME live
candidate PID/hash and unchanged private config through actor completion, with
enabled hook booleans. Thus do not chase a hypothetical config/daemon replacement
as this failure's cause. Common verifier correctly refuses; do not weaken it.

Current v16 is building under remote guard (source dir letter-fragment-release-20261102,
build receipt dir letter-fragment-native-caller-read2-v16-20261109). Only new
fragment_layout.rs now requests at most two existing bounded observations within
the SAME positive letter callback; no timer/observer starts a proposal, no write
retry, no protected observer rewrite. First rerun the saved failing Chrome
RU-to-EN first-word case, then same-binary ten-case batch. Temporary release
instrumentation removed; new hook logic unchanged. All post-policy native
restorations PASS, installed Rust123 remains on KDE and primary GNOME.

Private controller is /home/e/.cache/lay/development/letter-fragment-request-20261102/native-one.py.
It MUST keep live IBus use-system-keyboard-layout=true in its PRIVATE keyfile
backend; otherwise ibus CLI resets the physical us/ru map. It now also verifies
actual candidate config route/hash and post-actor bridge PID/hash even on FAIL.
Retain all original failures and their diagnostics; do not label the earlier
incomplete restoration as passing (separate recovery receipt exists). Full
current source/native release gates remain pending.232 protected files and all
old layout_sync bodies were compared byte for byte with123: unchanged. Latest
source-scope receipt37files PASS; generated local graph still needs remote sync.

User explicitly approved ONLY the separate existing-operation entry in
layout_sync.rs for Fragment and manual return. Owning ADR2026-11-09 and exact
scope updated BEFORE implementation; only one protected source added. Do not
change old layout-sync functions/branches or desktop adapters. Caller mode
publication and decoder promotion stay in new fragment_layout.rs; committed
caller and its unfinished-mode manual return use existing verified text writing.
See current top section of docs/development/letter-only-fragment-request-2026-11-02.md.
No runtime/install/release acceptance yet. Installed remote daemon/IME123.

## Previous audit — shared layout-sync bodies remain frozen

2026-11-05 user explicitly reiterated caller analysis BEFORE shared changes.
The preceding layout_sync.rs expansion request is suspended, not accepted or
proved necessary. Do not resume it as the active implementation plan.
Read the current caller-audit section at the top of
docs/development/letter-only-fragment-request-2026-11-02.md.

Established mismatch: committed_tail.rs1177 NEW Fragment caller takes mode
ownership with with_caller_layout_postcondition, then calls a full-stack
manual-mode switch while letters continue. Existing text writer is already
reused. Existing owned-preedit operation is correctly selected by shift.rs288
for genuine IME ownership; it cannot be borrowed for committed Chrome text.
InputMode/decoder alone leaves KDE's native group and daemon mode inconsistent.
Group-only primitive exists privately, but its exposed engine entrypoint is
owned-preedit-guarded. Do not invent ownership, manual intent or a new route.
No global impossibility or stable123 shared defect is established by this
audit. Next: prove an admissible existing caller entrypoint for committed
open-word mode, or precisely report the missing capability before any edit.

This audit changed only owning docs/handoff. No Rust, runtime, installation,
config, service or exact protected-file scope changes. Prior native v13PASS /
v14FAIL and red8PASS2FAIL of10 were reread, not rerun. Matrix unchanged.

## Previous proposal — C09 scope expansion, suspended pending caller audit

2026-11-05: ordinary conversion works per user; the fragment mode is the task.
Local daemon/IME are verified live/installed123; saved fragment flag=false;
no config or runtime mutation. A separate exact layout_sync.rs decision was
requested asynchronously under the frozen-field rule and remains unanswered.
Only tests in the already allowed fragment_layout.rs changed: no mid-word
engine replacement, native InputMode plus KDE group, and reconciliation when
returning to the selected factory. Remote TDD red is 8PASS/2FAIL of10, expected
failures at these exact transitions, receipt letter-fragment-c09-red-20261105.
No protected production file was edited and no candidate was installed.

Independent preflight-v11 completed: architecture graph/canon PASS, actual
manifest3681(3589correctness/36package/11performance/45ignored), focused13/13PASS.
Generated graph/binding/receipt and manifest/zero-waiver binding fetched locally.
Protected-source equality and remote source-scope35files PASS. Actual installed
menu updaters on both hosts exit0: installed123/lateststable123, no installation.
Do not run release gates/builds merely to certify the known failing native
variant. An explicit asynchronous decision is pending for ONE extra protected
source, layout_sync.rs; no answer means no dependent implementation/install.

No fragment candidate is installed/published. KDE full-restart MENU is installed
and was visually/service-verified; its KDE lifecycle action was not invoked.
Both installed Rust runtimes remain123; primary GNOME has not been changed.

Current diagnostic source (candidate124) passes1157IME tests and9focused mode /
suffix / stale controls, but native v14 rejects its synchronous engine switch:
151614us, partial receipt, Chrome final `приве`. Previous async variant also
lost letters in Firefox. Both are rejected; DO NOT install/tag/release or repeat
mid-word engine replacement. Frozen v11 artifact SHA7de672b6...; exact receipts
remote letter-fragment-native-v14-caller-mode-20261102, source gate
letter-fragment-caller-mode-all-ime-20261102. Scope/writers/reducers/desktop
bodies remain unchanged; layout_sync.rs has ZERO diff from123.

Best native owned-word behavior was v10/a05c6669...: Firefox early conversion,
two manual shifts, next-letter and actual KDE group PASS in v13 after restoring
the ALREADY SAVED us,ru keyboard map. Config SHAf57ba809 unchanged; original
runtime map was onlyru, exact dump preserved in configured-keymap receipt. No
settings-file edit or IBus restart. Existing actor now checks native mapping
before typing, verifies actual group before decoder, and controller restores
the original list/group. Earlier Chrome5text/decoder PASSs did not verify the
physical group; strict short-v10 catches that new C09 failure.

Required next decision is detailed in owner's "C09 boundary decision" section:
allow ONLY additional src/bin/lay_ibus_engine/layout_sync.rs for existing group /
InputMode current-word completion, no engine replacement while typing, and
KDE same-factory group reconciliation. No text writer/state/owner/verifier /
desktop adapter body or legacyGNOME behavior changes. New tests stay in the
already allowed fragment_layout.rs. Until explicit decision, continue only
independent source/graph/inventory work; no protected implementation/install.
Final native GNOME45+ is also pending; remote shell42.9 unsupported. Full browser
table unchanged165/15/40Firefox,179/0/41Chrome of440. No final sound yet.
All active/running labels below are historical. Exact owner:
docs/development/letter-only-fragment-request-2026-11-02.md.

## Previous milestone — Chrome short5/5PASS; Firefox open-word preparation repair

Native short-v9 retained candidate SHA264afcde...: all five Chrome cases PASS
(both directions/word positions, modeOFF + ordinary Space, manual round trips
and physical XTest decoder probes). Firefox first ENG→RU FAIL with `привт`:
NEW fragment path commits the four-letter prefix and starts background engine
handoff; next `е` is decoded but lost during focus/engine conflict. Receipt
letter-fragment-native-short-v9-20261102/firefox-letter-en-ru-first/RESULT.json;
saved analysis retains FAIL/eight warnings. Remaining four NOT_TESTED.

Same existing ownership checks show only display_pending=true prevents the
owned-preedit transaction. NEW adapter now retires pending prediction through
existing retire_pending_precognition ONLY for a positive proposal/current exact
legacy preedit, then revalidates its input frame. The already authorized shared
word-flip seam lets Fragment use existing replace_verified_owned_preedit with
protect=false. No preedit.rs/layout_sync.rs/reader/owner/writer/desktop body is
changed; no manual-intent grant. This corrects the earlier commit-first choice,
without a new route or a generic guard relaxation.

Current tool session47070 runs all1156affected IME source tests against this
repair; output remote letter-fragment-owned-preedit-all-ime-20261102/RESULT.json.
Await it, build NEW diagnostic variant using cached build-native-diagnostic.py,
then rerun ONLY the saved Firefox failure. If green complete remaining short
cases, reuse Chrome evidence only where relevant source did not change; final
release/graph/manifest/native gates still pending. All native wrappers restored
accepted123; no permanent fragment install/publication or primaryGNOME change.
RemoteGNOME42.9 remains unsupported for current45+ bridge native acceptance.
Full browser table still165/15/40Firefox and179/0/41Chrome of440total.
Exact owner: docs/development/letter-only-fragment-request-2026-11-02.md.
All active/running labels below refer to historical steps.

## Previous milestone — exact fragment suffix repaired; native follow-up pending

KDE full-restart tray is still the installed script bundle, PID3962487; actual
service/link/label rechecked on November2. Rust runtime remains123. No fragment
installation/publication or primary GNOME mutation occurred.

Native v8 physical-letter Chrome first ENG→RU PASS, including both manual
round trips and physical decoder probes. Short batch: first RU→ENG PASS,
second ENG→RU FAIL, remaining seven NOT_TESTED. Exact first failure is NEW
request's expected full buffer versus existing executor's exact replaced-word
snapshot: expected `тест ghbd`, actual `ghbd`. No write occurred.
Saved analyzer retains FAIL1/eight metadata warnings; full trace/controller
pins are separate evidence. No complete-matrix total changed.

One argument in the authorized fragment arm of committed_tail.rs now uses the
sealed action's original token instead of the whole buffer. Existing nearby
candidate acceptance already follows this contract. No protected owner,
writer, observation/desktop adapter or generic guard changed. New source
regression reproduces the old second-word failure; after fix2/2PASS (eight
desktop/direction/word-position combinations plus unknown-literal control).
Receipts remote letter-fragment-suffix-regression-{before,before2,after}-20261102.

Native-v9 proves the expected-suffix repair reached the next existing guard:
no_proven_delete_backend. NEW adapter skipped its existing native observation
when KnownStart already supplied a frame, despite no executable edit route.
Only that new caller's condition now requires a frame AND the existing
text_target_edit_route/can_execute capability, otherwise requests the same
bounded reader once. No reader/writer/ownership body or generic guard changed.
Complete affected IME selection now PASS1156/1156,177.88s including build,
remote letter-fragment-backend-all-ime-20261102/RESULT.json. The preceding
suffix-all-ime run was1155PASS/1FAIL in a new test fixture that omitted the
SurroundingText capability; fixed its setup and asserts Some before invoking
the detector. Preserve earlier FAILs, all eight focused controls then PASS.
Next: retain a NEW diagnostic124 artifact from actual Cargo record, run the
one failed native second-word case, then short10case set. Old v7/v8 retained
binary and previous v5 full gate do not certify this source. After short native
green, format, regenerate actual test manifest/zero-waiver binding and graph,
run final full release/locked10binary gates. Remote GNOME42.9 cannot certify
current45+ bridge; primary GNOME remains untouched. Native physical evdev is
NOT_TESTED; adapted actor uses existing XTest physical X11 codes for letters
and RFB for focus/gestures, with actual DOM/AT-SPI/scope/PID/inode checks.

Owner: docs/development/letter-only-fragment-request-2026-11-02.md.
Everything labeled current/running below is historical.

## Previous milestone — native fragment-purpose admission; candidate not installed

Current source includes a narrow NEW-origin admission repair under native test.
Earlier v5 full release gate COMPLETED PASS:3622source +11performance checks,
strict lints, architecture, all release bins and explicit10installed-binary build.
That evidence predates the current native admission edit and is not its release
acceptance. Retained v5 complete binary set is immutable and reports1.0.124.

RUNNING native tool session48473: one Chrome first-word control against
letter-fragment-native-admission-v7-20261102/lay-ibus-engine, SHA
412a1e1fd78a9a12806ce551457140eef42b48619bf6734b095806a58d5f4505.
Receipt directory letter-fragment-native-v6-admission-20261102. Poll the session.
This is DIAGNOSTIC_ONLY_NOT_PROMOTABLE, not an installation or release.
The wrapper restores123 through the existing controller and independently
verifies loaded/installed bytes, config and unchanged IBus PID. Wait/reap its
candidate child before the sole-process check (fixed a prior observer false
timeout). Primary local GNOME has not been changed.

Proven first failure in v3/v4/v5: six native presses/releases, zero command
modifiers, literal DOMghbdtn; the NEW adapter rejects every release because it
uses a Space-specific frame, while UnknownStart and actual native reads coexist.
No writer/reader-body fix is justified. New fragment_layout.rs now combines the
existing current native binding + both actual word boundaries/full-tail agreement
+ current scoped readout identity; it does NOT promote UnknownStart or use a
manual grant. Pure positive proposal may reuse WindowInteraction's existing
observation only with empty composition and no pending postcondition/Space work.
shift.rs's NEW Fragment gate uses that bounded frame and bypasses only the
manual-only unknown-word branch. Generic Space, state.rs, readers, owners,
reducers, output, desktop adapters remain unchanged. Added one unknown-literal
negative test and pending-postcondition control; actual manifest/full checks need
refresh after this behavioral repair if native control is green. Do not weaken
generic guards or change protected source bodies to rescue a failure.
KDE full-restart menu is installed and visually confirmed; its lifecycle action
has not been invoked on KDE. Both Rust runtimes remain accepted1.0.123.

Candidate metadata is1.0.124. Independent real Git source/build worktree:
/home/e/projects/lay-development-runner/letter-fragment-release-20261102.
Its target directory must remain separate from the LIVE shared factory target.
BaseHEAD4fae57d50b031984c1e0ddc8796e78e142b2a595 imported by verified base.bundle;
official snapshot2290files SHA020dbb501f2c504fe239fe50277def38bc5ad47e909c3cd05820aca19968df55
verified before metadata overlay. No124 install, tag or release exists.

Initial architecture WATCH was FIXED without guard changes: admission now
lives in existing decision.rs and reuses from_verified_action with the actual
candidate/report; the redundant new decision submodule was removed.
Latest graph/binding/receipt and actual manifest3678tests were fetched locally.
Inventory:3586correctness/36package/11performance/45ignored, zero failure waivers.

Actual fixed streaming proof40,000cases: clean wrong flips12/20000, no repeated
reversals; wrong-layout early restoration5235/10000RU and7575/10000EN; others
abstain and retain ordinary Space correction. Do not claim perfect quality or
full L1 restoration proof. Rejected aggressive policy had283clean false flips;
no threshold sweep, training, model bytes or dictionary was changed. Optimized
detector347901calls: median2237ns/p954269ns/max178607ns; cold946408ns;
standalone probe peak6400KiB, not whole-runtime RSS or native GUI latency.
Receipt: remote letter-fragment-open-proof-20261102-v5-production/RESULT.json.

Next: finish full gate + locked installed-binary build, then use the existing
frozen native RFB actor for BEFORE-Space first/second words, both directions,
OFF controls and Space/Tab/Shift controls on KDE. Actor v2 needs only new case
definitions; preserve its ownership, PID/inode assertions and native focus.
Old113 deployment scripts are historical; do not run them. The current factory
executes LIVE shared-target/release/lay-ibus-engine123, so never build over it.
Native installation/restoration must independently verify installed, factory and
loaded bytes. No candidate installation, primary GNOME mutation or publication
has occurred. Remote GNOME is Shell42.9 and cannot load current45+ Lay bridge;
its existing103files do not establish current GNOME acceptance. Do not upgrade
GNOME or alter primary input settings to force a PASS. Historical native browser
table stays Firefox165/15/40, Chrome179/0/41 of440total.
All active/current labels below this milestone refer to historical work only.
Exact owner/evidence: docs/development/letter-only-fragment-request-2026-11-02.md.

## Previous milestone — focused fragment repair PASS

Private candidate-boxed-boundary-v5 COMPLETED6/6PASS, including the OLD isolated residual at DEFAULT stack and all5new tests. Compile33.44s, tests0.191s. The two forwarding APIs now Box::pin the same existing operation, following native_document's established storage boundary. This fixed the transitive async/poll stack regression with no writer/owner/layout/old-test changes. Source file hashes verified against local checkout before/after. Parent direct-forward v4 was insufficient; its FAILED and GDB evidence retained.

NEXT launched from current canonical checkout: full affected IME dev-check --target bin:lay-ibus-engine (session ID is in newest tool output; update this header on result). Library run-uie85788 remains1870/1870PASS because library dependencies/source unchanged. No new fragment runtime installation/release yet. Local real updater --check PASS: installed123/latest stable123.

## Historical narrow candidate control — direct Future forwarding

Baseline comparison COMPLETED: public123 production source passes the same default-stack residual in0.113s; candidate fails. Current measured future sizes are1264bytes(optional),16768bytes(key),7600bytes(word flip). Replaced only the two newly added async FORWARDING wrappers (shift.rs manual-toggle-with-disposition and committed_tail.rs toggle-with-disposition) with synchronous functions returning the SAME operation Future directly, explicit borrow lifetimes. No state, writer, ownership or layout helper changed. Code has been remotely formatted; compilation/test of these two signatures is pending.

ACTIVE tool session94937: a six-case focused diagnostic under the existing remote dedicated20cpu guard, Cargo discovery and hermetic run_target. Path /home/e/projects/lay-development-runner/letter-fragment-controls-20261102/candidate-forwarding-v3. It clones the canonical workspace, applies ONLY the two forwarding edits, and verifies EVERY Rust source file plus Cargo/model/test inventory SHA against the current local checkout. Selected:5new fragment tests + the unchanged isolated residual. This is not full affected-gate/native acceptance. Poll returned session94937. If green, run canonical dev-check --target bin:lay-ibus-engine once; library1870PASS remains valid because library source did not change.

Private helper exact bytes: local ~/.cache/lay/development/letter-fragment-request-20261102/focused-probe.py; remote same cache. Full source hash map forwarding-source-hashes.json. The prior .cache probe prep failures are retained; actual baseline/current diagnostics now execute under projects, outside the canonical sandbox cache mask. No candidate runtime has been installed or published; remote menu-only installation remains correct.

## Historical diagnostic update — baseline comparison

Latest tool session65124 is running the narrow baseline control, after prior source checks completed. Remote directory /home/e/projects/lay-development-runner/letter-fragment-controls-20261102/baseline123-control-v2. Poll this session. It uses project dedicated20cpu resource/Cargo guards and existing hermetic test-lane functions, not a new runtime path. Production src and Cargo+scripts are restored from actual public v1.0.123 commit5059360769266e55e3f393ba28eb57d6d5969d5c inside that PRIVATE control copy only; actual working checkout is untouched. Baseline tar SHA91c12b53d6fdf1d083be02c0f76db65f9d114d9ec711a7db13505ff69f1f2999. It compiles only the IME test target and runs ONLY the unchanged isolated failing residual test. This is a causal baseline control, not release/native acceptance.

Current new-source metadata probe already PASS: optional fragment future1264bytes, ordinary key future16768bytes, existing flip operation future7600bytes. Thus do not assume a massive future type without evidence. GDB lacked usable DWARF type names; use measured Rust sizes. The first preparation attempt in .cache failed BEFORE baseline compilation (sandbox masks .cache/lay and nested SSH heredoc quotes were stripped). Corrected script was transferred as a file; new control is under projects, outside that mask. Failed receipts retained.

If tagged baseline passes, investigate added async forwarding poll frames in shift.rs and committed_tail.rs (new with-disposition wrappers) rather than edit ownership/output or raise RUST_MIN_STACK. They can potentially forward the existing Future synchronously, retaining APIs and all branch semantics; measure/test before claiming. If baseline also fails, this is not sufficient evidence of a fragment regression: analyze the exact baseline failure before modifying production. Do not repeat another full3050test run until this focused comparison distinguishes the cause.

## Historical preflight — letter-only fragment implementation, 2026-11-02

The user explicitly requests the previously agreed LETTER-only detector for BOTH GNOME and KDE. KDE full-restart menu was missing because the old tray was still loaded; the three verified menu/control scripts are now installed on KDE, live tray3962487 active and SNI registered. Native screenshot confirms «Полный перезапуск Lay» under «Lay включён», version123. No KDE full lifecycle restart invoked. Runtime Rust binaries remain accepted123. See full-restart-2026-11-02.md and local ~/.cache/lay/development/full-restart-20261102/kde-live-menu.png.

Current fragment owner/preflight/ADR: docs/development/letter-only-fragment-request-2026-11-02.md and docs/architecture/decisions/2026-11-02-letter-only-fragment-request.json. EXACT source boundary sealed in active-field-integration-scope.json, immutable base4fae57d50b031984c1e0ddc8796e78e142b2a595. Do NOT restore rejected124 stash. No Harness or agents.

New code: library layout_fragments/letter.rs creates an exact automatic request through existing DecisionCore new child decision/fragment_layout.rs, using unchanged687981byte model and Boundary::Open, initial conservative margin16. Boundary typing_rule_graph/rules/fragment.rs now ALWAYS calls the preceding legacy rule, regardless of flag. New IME fragment_layout.rs validates current frame/word and submits WordLayoutOrigin::Fragment to existing shift.rs operation. Fragment skips manual undo and Space suppression. Owned composition uses the existing verified commit + automatic layout sync; committed tail uses existing automatic request + selected winner/exact tail. Protected state/output/preedit/layout_sync/desktop/daemon mechanism files UNCHANGED. Hooks now ONLY paired managed-letter release and existing paired native-letter release, not press/Space/Enter. Normal feature-OFF paths must skip the adapter outside its async call.

Remote source checks: run-n92lfyf6 compilation PASS34.90s, then stale process-isolation registry. run-vsai6qx9 exposed two incorrect NEW test fixtures (active_path missing; password transition clears buffer) and an OLD isolated residual stack overflow. Fixtures corrected without weakening production. run-wxpyjdiu: all three new IME tests PASS; bulk681PASS0FAIL; old isolated td121_repeated_closed_source_retains_scope_across_delayed_erasure_receipts still stack-overflows (log remote run-p2kgkT/tests/logs/process/...). Box::pin ONLY new admitted request future initially did not prove fix. Replanned in owner: add actual future-size bounded test, move managed trigger to release to avoid atomic Delete-before-Commit ordering, and skip native adapter completely on flagOFF. DO NOT raise RUST_MIN_STACK or weaken old test. Latest code after those refinements has NOT run IME checks yet.

COMPLETED library check run-uie85788:1870/1870PASS175.3s. Latest IME check run-38o0v0a1: all5new testsPASS, bulk683PASS0FAIL; same isolated default-stack residual still aborts. No native build/install yet. New IME source includes two extra tests (real paired letter vs Space/Enter/unpaired release and <=16KiB adapter future with directly written size metadata). Manifest manually declared their exact names; future actual discovery still required. Previous actual discovery had3047rows(1156IME+1891lib); new expected total3049. Test discovery isolation entries replaced obsolete boundary flag tests with two config-mutating invariance tests. Manifest sorted through existing canonical(); default limits/gates intact.

Next: finish library check, run latest IME exact/source check and resolve OLD residual stack overflow using measured future sizes/baseline comparison, not blind edits. Then inspect complete diff/frozen mechanisms, build candidate ONLY after source checks, native GNOME/KDE with existing actor, first/second word and BOTH directions BEFORE Space; Space typo correction remains active, Tab/Shift/Double Shift controls. Existing full KDE matrix344/15/81of440 is HISTORICAL, not changed. New fragment code NOT installed or released on either desktop.

Important native-build risk: remote live IME has been launched from /home/e/projects/lay-td119-gate-v1/target/release/lay-ibus-engine and old factory could also point to that cache. Verify actual factory exec before any release build; do not overwrite a live factory path accidentally while building. Use an isolated candidate release target if needed. Old private native helper's restoration proof was previously wrong; never trust its reported restoration without independent installed/live/factory byte parity.

Open PR API checked2026-11-02T02:38:15+03:00: none. Graphify/human-headroom skill static scans were refused by shared local RAM guard; do not claim clear scans or bypass them. Heavy work only remotely through existing dedicated20cpu/24Gsoft28Ghard/1Gswap/T512 guards. Graph refresh still pending remotely. Completion sound hook required at true completion only, half-volume.

## Next patch — user-confirmed inclusion

The user confirmed that the installed full-restart menu feature must be included in the next patch. Include the existing verified GNOME/KDE menu changes, runtime-controller full-restart action and lifecycle worker, tests and owning ADR. The preceding local GNOME implementation is accepted; retain its exact verified source and evidence. KDE native GUI acceptance remains untested. This confirmation does not request immediate publication or restore the rejected fragment experiment.

## CURRENT — full restart menu installed and checked — 2026-11-01T23:27:19.773770+00:00

User confirmed the preceding local GNOME 1.0.123 runtime works excellently and requested complete restart from a menu button. That diagnostic is CLOSED by user acceptance; do not resume its old failure hunt. Current task complete: new top-level `Полный перезапуск Lay` is installed in GNOME; KDE menu source has the same action, no remote KDE installation was done.

The existing controller now has an explicit full-restart command which launches a separate transient user unit and imports the original desktop/layout helpers. Automatic/quick restart remains unchanged. New lifecycle worker resets current L1.1 socket owner, L3, daemon, IBus/managed IME, GNOME extension; restores RU/EN and drops the retired IBus socket address only in its own environment. No Rust/input/correction/display/delivery source, model or browser setting was changed.

Verified source: 9 new Python tests,12 existing startup tests,19 canonical remote tray-contract tests PASS; Bash/JS syntax and exact source scope PASS. Local actual Qt menu test could not run: PyQt5/PyQt6 absent (8 daemon-shell PASS,1 GUI ERROR; separate persistence test SKIP). Graphify scan/update refused shared RAM admission; no bypass and no fresh graph claim. No new release/commit/publication.

Actual installed controller cycle PASS in6819ms. New current PIDs: IBus1761828, daemon1761913, IME1762198, L1.1 1762230, L3 1761914. All component hashes equal preceding123 bytes; loaded IME/CLI/tray123, GNOME+IBus LayRU, fresh L1.1 socket owner verified. Menu label is observed in live GNOME tree. Hidden AT-SPI item had no action; cycle used the exact CLI command behind the menu, not a claimed physical click. No browser foreground/input or new native matrix.

Installed entry ~/.local/bin/lay-runtime-control points to ~/.local/lib/lay/runtime-control-full-restart-20261102/lay-runtime-control.sh, with its adjacent lay-full-restart.sh. Original symlink and GNOME modules backed up in ~/.cache/lay/development/full-restart-20261102/before-install and staged-install.json. Owning doc docs/development/full-restart-2026-11-02.md; canonical source check run-h1vdvwzm; live receipts in that cache directory. All older runtime PID statements below are historical.

Queued future task remains letter-only fragment detector → existing flip with typed origin; Space ignored; no delivery rewrite. Do not restore the rejected124 stash.

## Current local GNOME recovery / Chrome diagnostic — 2026-11-01T22:44:27.090632+00:00

User reports all right-column test controls failing on installed 1.0.123. Requested a complete local Lay restart, then explicitly required Chrome to remain in the background. No remote work is authorized for this local diagnostic. No Lay runtime source edit or installation was made in this turn.

Completed authorized restart of IBus, L1.1, L3 and daemon; reloaded the installed GNOME extension. Current PIDs: IBus 1343894, L1.1 1344087, L3 1343949, daemon 1344025, IME 1345111. CLI/live IME/tray all report 1.0.123; live and installed IME SHA-256 both 2a4c636a3be959ce096754909599a202146b7711157c2563e6980f5c8909d429. GNOME and IBus both Lay RU. Keyboard device access and L1.1 socket present. Fragment option remains OFF; no config edit. Earlier PID references below are superseded.

Controlled local Chrome was opened using the existing chrome-agent launcher/profile, debug port 9222. Standard route verification has a false mismatch against flattened argv; the already registered chrome-agent-flat-argv route verifies PASS against the exact command fragments and owner PID 1369312. Its owned ten-field fixture is open. After user prohibited bringing Chrome forward, no more focus activation or native key injection was performed. Background DOM inspection found all ten controls editable, neither disabled nor read-only. A 45-second passive observation recorded ZERO input/gesture events; native Double Shift/Tab/autocorrection remain NOT TESTED after restart, and the reported defect is NOT resolved by evidence. Do not claim native PASS, causality, speedup or a source fix. Do not substitute CDP-dispatched keys for native Lay acceptance.

Receipts: /home/ubu/.cache/lay/development/local-chrome-123-check (before/after runtime identities, restart steps, actual foreground-window identity before the background restriction, background fields, passive observation summary). Current source differences remain the prior two GNOME tray version strings plus this handoff; no restoration of rejected fragment experiment. Static graphify skill scan could not run because the shared resource guard rejected admission; no bypass, graph query, build or source test was performed.

Next queued task remains below and must not start until this diagnostic is settled: separate letter-only fragment detector requests the existing flip mechanism with typed origin; ignores Space and does not rewrite delivery.

## Next task explicitly queued by user — letter-only fragment detector

The local 1.0.123 diagnostic is closed by user acceptance. The requested full-restart menu action is implemented and locally verified; the following fragment task remains queued and has not started.

Then: letter insertion → separate fragment detector → request to the already existing word-layout flip mechanism with a typed origin marker. Space continues to invoke the previous functions; the fragment detector ignores Space and must not block its functions. Do not rewrite delivery or input routes for the detector. Map the existing entrypoint and exact allowed files before coding. The rejected unpublished 1.0.124 experiment remains archived.

# CURRENT — user rollback completed, stable1.0.123 — 2026-11-01T21:45:07.003394+00:00

The user explicitly requested rollback after rejecting the unpublished fragment
integration and changes to established input/delivery paths. DO NOT resume the
old1.0.124 experiment, restore its stash or continue its queued release plan.
The implementation of that experiment was NOT accepted or published.

Source rollback:
- Checkout /home/ubu/projects/lay-dependency-updates-20261027, branch
  codex/browser-matrix-repair-20261027, HEAD4fae57d50b031984c1e0ddc8796e78e142b2a595.
- This HEAD differs from publicv1.0.123 commit5059360769266e55e3f393ba28eb57d6d5969d5c
  only in publication/evidence documents. Runtime source equals stable123.
-57 tracked changes restored and28 untracked experiment files preserved in
  stashb2a0786bfed29e90e4736dcadce22ee183a67593. Every saved source file hash
  verified against the stash. No reset/forcepush/tag/release/build.
- Complete rollback receipt and patch: /home/ubu/.cache/lay/development/fragment-letter-only-20261031/rollback-to-123-20261101T213602Z/rollback.json.
  Four ignored generated graph cache outputs preserved there and retired;
  tracked graph/source receipts restored with the baseline.

Completed separate user request: fix GNOME tray showing117 while application123.
Both accepted source and installed extension had static APP_VERSION117 and
metadata117. Only metadata version fields and APP_VERSION changed to123.
Only GNOME extension reloaded via existing check-gnome-extension-runtime.sh;
actual loaded DBusVersion123 confirmed. Primary daemon1651569 andIME1651666
PIDs/SHA256 unchanged. No main GNOME binary installation/restart, no IBus restart.
Receipt /home/ubu/.cache/lay/development/fragment-letter-only-20261031/rollback-to-123-20261101T213602Z/gnome-tray-version/verification.json.
Current source code changes are ONLY these two version-metadata files plus
this current handoff. No fragment/controller/observer/layout/shared route changes.

Remote KDE rollback verification found a real restoration defect in the private
native helper: installedIME123 but live factory124 after restoring an older
factory cache; its earlier intermediate restoration booleans did not certify
final live bytes. Do not reuse that helper for acceptance until restoration is
fixed and independently reviewed in a separately authorized task.
Explicit user rollback completed using existing source path, installed123 copied
to factory cache and only candidate factory restarted. First activation raced
registration (raw RESTORATION_INCOMPLETE preserved); subsequent normal activation
succeeded. Final remoteIME3617885 reports123; live/installed/factory hashes ALL
b765fde82d7f49418b0175880a0eb97aed1dde0ba5321524c60307e668c94a06.
Remote daemon/globalIBus/config unchanged, source lay-ime-ru restored.
Final remote receipt /home/e/.cache/lay/development/fragment-letter-only-20261031/
user-rollback-runtime123-20261101T214324Z/final-verification.json.
Fragment option is OFF in both primaryGNOME and remoteKDE, checked not changed.

User's required future design (not newly implemented):
1. Only insertion of a letter invokes fragment detection. Space/Enter do not
   invoke it; it cannot disable, intercept, cancel or block ordinary Space rules.
2. Detection submits a request to the ALREADY EXISTING layout conversion
   mechanism; reuse its established owner/verifier/executor and delivery.
3. Necessary feature-specific behavior uses an explicit typed origin flag
   at a narrow documented seam. Do not rewrite existing paths to fit a new
   feature or make a parallel field/word owner or replacement backend.
4. Map exact existing command entrypoint and affected files before another
   implementation. User rejects broad shared-route changes.

No new implementation is running. StableGitHub and installed primaryCLI123
verified current. No124 publication. Completion sound is required at turn end.

--- Pre-experiment stable handoff (historical) ---

# COMPLETE —10/10 stable releases published and verified

Updated 2026-10-31T12:27:04.686164+00:00. All requested114–123 independentlyreadbackAPI/stable. Latest123published2026-10-31T12:23:35Z,releasecommit5059360769266e55e3f393ba28eb57d6d5969d5c,identicalannotatedtagdc576872048f6867ac6b0858e6491ffa5e9878d5onorigin/public. All4releaseCI andbothactualinstalled menuupdaters/sourcesPASS. Local122sees123available;remote123current. No agentlocalinstall/restart. Source1329+4metadata1333matches24fbb98d,test3617Rust/469Python(459PASS10SKIP)/17gates/tenzero-warning. Loadedremoteall10+daemon/IME/factoryverifiedafter2nativecontrols/4flips.

Scope:121sixmechanicalKDEmoves/fourpathseams;122shortkde-focus8→48;123offlineclock-margin diagnosis+27testsincanonicalroot. Lasttwohavenoruntimebodychange;1338otherfiles/22oldtestbodies frozen. Recordedclockmarginisanestimate;real savedrowstillFAIL1/3after+1overlap/CausalUNKNOWN. No fullmatrix/speedupclaim.

Completionreport docs/development/ten-release-completion-2026-10-31.md andversionedfinal-verification/publicationJSONsownproof. Thisfinalupdateispublicationdocumentationonly;source/runtime/versionmatchreleased123. FinalmetadataCommit/sourceidentity are recordedinCACHE/release123-publication-metadata-commit.json andHindsightafterpush. DoNOTrepeat123build/install/native/testsforpublication.

Knownscope:historicalKDE344PASS15FAIL81BLOCKEDof440(notrescored). TD168initialFirefoxNotHandled/TD166independentfragment/TD167initialChromeabstentionOPEN;TD162DEFERRED. TD169/170DONE. Ten-releaserequestcomplete,notallnativebugsclosed. Nextworkrequiresnewscopedtask/baseline/map;preservefieldroutes/commoncorrection/ownership/verifier/executor. Solo/noHarness/noagents/RDP;allheavyremote. Completionhookhalfvolumeatfinalturnend.

# LATEST —10/10 published; final publication records only

Updated 2026-10-31T12:24:36.158043+00:00. All10requestedstable releases114-123 ACTUALLYPUBLISHED/readbackAPI 2026-10-31T12:24:36.158043+00:00. Latest123 2026-10-31T12:23:35Z commit5059360769266e55e3f393ba28eb57d6d5969d5c,tagdc576872048f6867ac6b0858e6491ffa5e9878d5. All4releaseCI/identicaltagmirrors/bothactualmenupdatersPASS. Remote123all10/load/factory/native2controlsverified;local1.0.122notmodified. Source24fbb98d/1329+4metadata1333;3617Rust/459Python10skip/17gates/tenzero-warning. FullKDEtablehistorical344/15/81notrescored;TD168/166/167OPEN,TD162DEFERRED. Source/runtimegoal10COMPLETE;finalmetadata-onlycommit/push/sourceidentitycheck remain,thenfinalhookhalfvolume+report. No newversion/tag/build/install/nativeinput formetadata. DoNOTrepeataccepted123verification. Completionreportten-release-completion-2026-10-31.md ownsfinalscope.

# ARMED —final123CI/publication waiting;9/10 published

Updated 2026-10-31T12:01:18.449478+00:00. 123committed/pushed5059360769266e55e3f393ba28eb57d6d5969d5c;1333Gitblobs/execbits matchtested24fbb98d(source1329),Gitmode51576721. Full17gates/3617Rust/459Python10skip/all10zero-warning/loaded123/native2controlsPASS. Frozen1338otherfiles+22oldtestbodies. Actorstatus1FAILpreserved inreal savedCLI3after-recorded-margin1overlap;notanativefix. Currentremote123verified,localGNOMEuntouched. Publisher ALREADYARMED toolsession13665 waits4CI(mainpending,other3PASS)thenstable123+bothactualupdaters. DoNOTduplicateorchangeHEAD/sourcebeforepub.9of10untilAPIconfirm. Afterpub runverify-installed-updater-entrypoints.py123;mirrorexactannotatedtagorigin/readbackbothpeels/API;versionpubreceipt/queue10DONE/TD170DONE/owner/completionreport. Finalmetadata-onlycommit/push(sourceunchanged;useuniqueidentityoutput,doNOToverwrite123releaseidentity505). No newversion/tag/functionalitytests/install needed forpublication. Verifyall10releases114-123exist/stable. Thenfinalcompletionhookhalfvolume andself-containedRussianreport;matrix344/15/81historicalnotrescored,TD168/166OPEN,TD162DEFERRED. Noagents/Harness/RDP. FinalmemorycitationMEMORY5241required.

# LATEST —123verified;final commit/CI/publication next;9/10published

Updated 2026-10-31T11:52:50.627959+00:00. Source1329SHA24fbb98d13b4d9a2f139e9a7bc65780f34a48147de33e12f720c8506f7e48fcd. Only3offlinePythonfiles/Cargoversion/generatedmetadata;1338otherfiles+22oldtestbodiesfrozen. RED27/19expectedassertfail;GREEN27+actualsavedCLI1FAIL/3after1overlap/CausalUNKNOWN. Full17gates/3617Rust/459Python10skip/all10zero-warning;remote123all10+loaded+factorybytes/native2controlsPASS. Strictsourceimport/inventorycomplete. No more functionaltests/build/install needed forpublication. Nextcommit/immutableGitblobverify/push/4CI/stable/tagmirrors/bothactualupdaterchecks. DoNOTendbefore123publication/readbackscomplete;then10/10requested114-123. Finalsoundhookonlyactualcompletion. Knownmatrix344/15/81unpaired/TD168OPEN,TD166OPEN/TD162DEFERRED;neverclaimallnativebugsclosed. LocalGNOMEuntouched.

# LATEST —123full gates running;9/10 published

Updated 2026-10-31T11:32:53.846051+00:00. 122publishedCOMPLETE9of10(tagf8db0aeb,commit5c6eee98,all4CI/tags/bothupdatersPASS). DoNOTrepeat122. Current123canonicaloffline-tooling3files:receipts.py,scriptnative-diagnostic-tests,newrootload_testsadapter. NewADR2026-10-31-native-timing-certainty-123/scopebeforepublicedit;1338otherfiles+22oldtestbodyhashesfrozen. CanonicalRED27run19expectedassertfail0errors;GREEN27PASS+realCLIsavedreceiptstill1FAIL/3afterrecordedmargin1overlap/CausalUNKNOWN,total1.229s. InitialGREENprepareguardrejectedmissing shared declaration;restoredbaseline anddeclaredexact already-authorized receipts.py before reapplying;no guard/sourceoutside3paths modified. Cargo123metadata. Full17gates ALREADYRUNNING toolsession77771 phase release123-native-timing-full-v1 via release123-native-timing-full-worker.py. Format/graph/manifestPASS;3617Rust running. Runtime remote122unchanged;doNOTduplicatefull/build/install. AfterPASS strictimport/artifactinventory/install/load/nativecontrols/commitverify/push/CI/pub/updaters/tagmirrors. Then10of10willbecomplete;actualfinalhook onlyaftercompletion. Privateprototypeproofsremainnotcanonical,butcanonicalGreenalreadydone. Matrix344/15/81unpaired unchanged/TD168OPEN;no native/speedupfixclaim. Solo/noagents/Harness/RDP/localGNOMEchanges.

# LATEST —122published;9/10;123next

Updated 2026-10-31T11:13:32.169715+00:00. 122publicationCOMPLETE 2026-10-31T11:11:30Z commit5c6eee98698f1bbd1187f31434bea80d7569d98e,all4CI/tagmirrors/bothactualupdatersPASS;tagf8db0aeb33b7db1a4a45f007c4d25b513101e336.9of10published114-122. Publisher65223FINISHED;doNOTrepeat122. Next123publicpreflight before3sourcefiles;privateprototype27GREEN+real1FAIL/3after1overlap ready butNOTcanonical/notinstalled. CanonicalRED/GREENstillrequired,thenfull17gates/build/load/native/CI/pub/updaters. Currentremote122exact,local119notmodified. Noagents/Harness/RDP. Matrix344/15/81unchanged/TD168OPEN. Finalsoundonlywhenall10done.

# ARMED —122CI/publication waiting;private123prototype ready

Updated 2026-10-31T11:05:16.480258+00:00. 122committed/pushed5c6eee98698f1bbd1187f31434bea80d7569d98e;1332Gitblobs/execbits match tested742e0997.17gates/3617Rust/432Python10skip/all10zero-warning/loaded122/native2controlsPASS. PublisherALREADYARMED toolsession65223 waits4CI(mainpending,other3PASS)thenstable122+bothupdaters. DoNOTduplicateorchangeacceptedcheckoutsource/HEADuntilpubfinished.8of10untilAPIconfirm;mirrorexacttag/readbackthen123. PrivateofflineprototypeONLY for123 in CACHE/release123-clock-prototype:27testsGREEN(all22old+5new),actualsavedreceiptretains1FAIL and3after-recorded-margin+1overlap. PRIVATE notcanonical/notinstalled. RED27run17assertfails0errors retained;later2depth/boundarysubcasesadded,canonicalRED/GREENstillrequired. PrivateGreenphase release123-clock-prototype-green-v2;green-v1archiveunused. Next123publicpreflight beforeediting3files(receipts.py,scripts/tests/test_native_diagnostics.py,newtests/test_native_diagnostics_suite.py). ExistingcanonicalPythonrootmisses22script-tests;adapterwillincludeall27. No123repo/Cargo/runtimeedit yet. Currentremote122all10/factory/loadverified;localGNOMEuntouched,TD168OPEN/matrix344/15/81unchanged. Noagents/Harness/RDP;endonlyafter10published/finalhook.

# LATEST —122verified, commit/CI/publication next;8/10published

Updated 2026-10-31T10:42:57.679691+00:00. Source122742e0997.. /1328. Runtimebodiesfrozen1323others;only3recipe tuples+2regression methods/Cargoversion+generatedmetadata. RED4assertfail/GREEN5unit48canonical. Full17gates/3617Rust/432Python10skip/all10zero-warning;remote122all10+loaded+factorybytes/native2controlsPASS. Sourceimport/inventorydone. No more functional tests/build/install needed forpublication. Next commit/verifyblobs/push/all4CI/stable/tagmirrors/bothactualupdaters. DoNOTendturnat122:8of10publisheduntilAPIconfirm,then123privateplan next-native-clock-certainty-plan.json. Privateplan hasreal3after+1overlapdata and22analyzertestsoutsidecanonicalroot;no123sourceedit. TD168OPEN/matrix344/15/81unchanged,localGNOMEuntouched.

# LATEST —122full gates running;8/10 published

Updated 2026-10-31T10:21:02.595714+00:00. 121publication COMPLETE8/10(tag0342a141,commit9f0a158b,all4CI/tags/updatersPASS). DoNOTrepeat121. Current122tooling-only:canonicalkde-focus8skips40realtests. NewADR2026-10-31-kde-component-coverage-122/scopebeforeedits;1323othersourcehashesfrozen. RED2methods4assertionfail0errors;GREEN5Pythoncontracts+actual48canonicalcasesPASS2.796s. Changedonly3recipe tuples and2regressionmethods,Cargo122metadata. Full17gates ALREADYRUNNING toolsession59592 via release122-kde-component-full-worker.py phase release122-kde-component-full-v1. Format/graph/manifestPASS;Rust3617running. Remote actual121unchanged,baselinehashes in release121-isolation-deployment-v2. DoNOTduplicatefull/build/install;poll59592. Afterfullstrictimport/sourceinventory/buildwarnings/remoteinstall/load/nativecontrols/commit/CI/pub/updaters;then123provisionaltimingdiagnosticprecision. Noagents/Harness/RDP/localGNOMEchanges. InitialFFTD168OPEN,norescore344/15/81.

# LATEST —121published;8/10;next canonicalKDEcoverage

Updated 2026-10-31T10:06:12.823640+00:00. 121published 2026-10-31T10:05:20Z commit9f0a158bb4c53bef2c61560075a9b62ce0fabdac. All4CI/identicalannotatedtagmirrors/bothactualupdatersPASS;local119notmodified,KDE121current.8of10complete114-121. Publisher47690FINISHED;doNOTrepeat121gates/install/native/publication. InitialFirefoxNotHandledv1retainedTD168OPEN;120comparison2PASS/restored1212PASS,nocauseclaim/matrixrescore. Next122canonicalkde-focus recipe8skips40realtests:privateplan+regressiondraft+fullworkerready,CACHE. No122source/preflightyet. Mustcomplete122-123,solo,noHarness/noagents/heavyremote/finalsoundonlyactualcompletion.

# ARMED —121committed/pushed; publisher waitsCI

Updated 2026-10-31T09:43:17.549807+00:00. Lay121 committed/pushed9f0a158bb4c53bef2c61560075a9b62ce0fabdac; exact1332blobs/execbits match tested556d8b7c.17gates/3617Rust/430Python10skip/all10zero-warning. Remote121all10+actualfactorypath/loadedhashes verified. InitialFirefoxv1NotHandled retainedTD168;baseline1202PASS/restored1212PASS,rootUNKNOWN. PublisherALREADYARMED toolsession47690 waits4CIthenstable121+bothupdaters;doNOTduplicateorchangeHEAD/sourceuntilfinished.7of10untilAPIconfirmed;thenmirrortag/readbackandcontinue122KDEcomponentrecipe8skips40actualtests,nextprivateplanready. No GNOME modification.

# LATEST —121verified, publication pending;7/10 published

Updated 2026-10-31T09:36:46.512828+00:00. Exact121 source1328SHA556d8b7ccb081cb27bd9591d8b7f518cb5762c3b8e1d0a732db3bd3eedf4cfbb.
Six KDE moves/four literal seams;51focused/3617Rust/17gates/ten binaries0warnings.
Remote current121 loaded/hash-verified,all ten installed+factorytargetsamebytes,
config/mode/globalIBus preserved. LocalGNOME untouched. Initialnative-v1FFFAIL
retainedTD168,notfixed;120comparison2PASS andrestored121v2controls2PASS.
No fullmatrix rescore:344/15/81of440. Finalreceipt versioned. Sourceimportdone.
Next commit/verifyblobs/push/4CI/tag/stable/bothactualinstalled updaterchecks
andtagmirror. No more tests/builds/install/nativeinput needed for publication.
After121publishedcontinue122canonicalKDErecipe8currentlyskips40realtests;
123provisional timingcertainty tooling. Complete10requested,do notstopat121.
Solo/noagents/Harness/RDP;heavyremote;finalsoundonlywhenactuallyending.

# LATEST — 120 published; 7/10; next KDE physical source boundaries

Updated 2026-10-31T08:50:34.115761+00:00. v1.0.120 published08:47:13Z,
canonicalcommit a9b9bb9fab96f6b6d4a8fa11d5dceb6ccead065b. All4CI/SecurityPASS;
annotatedtag050a7443d52f2678a14a4d42e604649fd9226c01 identicalorigin/public and
peeledcommita9b9 verified. Bothactualinstalled updater --checkPASS: local119
sees120available;KDE120current. Resolvedsources ~/.local/lib/lay/update.sh on
bothhosts SHAaebfb65e... /latestpublicAPI checked. No localGNOMEinstall/restart.
Publisher88171finished;tagmirror/readbackfinished. DoNOTrepeat120publication,
sourcegates/install/nativeinput. Actual publication proof nowversioned.

Queue7of10published114–120. User sayscontinue121–123,notstopafter120. Next task
is one cohesive mechanical KDE source-file isolation, from exact120baseline.
Private plan CACHE/next-kde-isolation-map-draft.json has6moves/4sharedseams with
expected/normalizedhashes. PreserveRustmodulealiases/parents and functionbodies,
onlyinclude/pathliterals change. Existingneutraldbus_connection remains shared;
doNOTduplicate/refactorGNOME behavior. scripts/check-architecture.sh path moves
with same180lineceiling; newowningADR before any source/guardedit.

Deletiontransport prerequisite: canonicaldev-check alreadyuses marker-owned
rsync --checksum --delete underlease. Privateoverlay helper lacksdeletions;
using it blindlywould leaveoldremote source paths. Preferexistingcanonical
mechanism,or explicitpreimage-verified transport handling;oldpaths mustbeabsent.
No121code/versionbump yet. Workingdirtyonly120publication/handoff metadata.
Beforeeditingwriteconsequence/scope/ADR;useexistingcomponent/sourcegates,full
mandatorychecks/nativecontrols/CI/publication/updaters. No newroutes/owner/cache.

Source120565282ef.../1328,1332inventory/10binaries0warnings,3617/34/494isolates,
Python430PASS10SKIPof440,17gates. LoadedKDEdaemon535954SHA2a8ff8c0..,
IME536040SHAb51116c3..,inode24803136;tray536089;globalIBus3168326/config5d02..
preserved,LayRU/layout0. Native2controls4flipsPASS(unpaired,not5rapid claim).
HistoricalFF165/15/40+Chrome179/0/41=344/15/81of440 NOTrescored. Timingcertainty
private evidence confirms oldwire4late reports=3definitelylate+1ambiguous;
rowstillFAIL, no totals changed. Analyzer receipts.py76–77 unconditionally
labels late_targets asTARGET_AFTER_NEXT_GESTURE; futureseparatetoolingcandidate.

SOLO/noagents/Harness/RDPviewer,allheavyremote. No completion sound yet;
onlyatactualturncompletion. Memorycitationfinalrule5241stillapplies.

--- Prior120publication/CI checkpoints are historical below ---

# ARMED — 120 committed/pushed; automatic publisher waits CI

Updated 2026-10-31T08:20:20.009574+00:00. Canonicalcommit a9b9bb9fab96f6b6d4a8fa11d5dceb6ccead065b,
Cargo120. Pushed origin devbranch, public devbranch and publicmain; all refs
fast-forwarded fromb06. Git blob/executable-bit verifierPASS:1328source+4release
metadata files exactlymatch tested565282efa1867e90966cbd1a9882301a7218700458aaf05c436bc0562be1038a;
Git group-write normalization documented, no code effect. Working source clean
at push. LastpublicPR queryempty. Source/native/runtime verification below
COMPLETE; doNOTrepeat builds/tests/install/nativeinput for publication.

Publisher ALREADY ARMED in tool session88171:
human-safe-run512M wait-publish-verified-release.py1.0.120 a9b9bb9f v1.0.119.
It waits all4public CI/Security, then creates/pushes annotatedpublictag, publishes
stable and invokes BOTH actual installed updater --check entrypoints. DoNOT
start secondpublisher or changeHEAD/source before it completes. Currently
installer/MSRV/mainchecks inprogress,Securityqueued.6of10published until actual
APIconfirms120. Afterpublisher success mirror SAME annotatedtagobject toorigin,
independently readbackboth tagrefs/peeledcommit and actualupdaters/entrypoints.
Then add actualpublication metadata to queue/owner and continue121–123 genuine
fixes. No finalturn/soundafter120alone;userasked10. No agents/Harness/RDP.

Cache current artifacts: release120-commit.json,release120-committed-source-identity.json,
release120-tested-inventory.json,release120-artifact-inspection.json,
release120-installed-runtime-verified.json,release120-final-loaded-identity.json,
release120-browser-controls.json. AllCROOT rawFAILs and theirnotes retained.
Native2controlsPASS/4flips;historical344/15/81 NOT rescored. Remote120realdaemon
535954,IME536040,tray536089/globalIBus3168326/config5d02..preserved;localGNOME
untouched. Source17gates/3617Rust/494isolates/430Python+10SKIP/10binaries0warnings.

--- Earlier 120 checkpoint (historical pending-status text below) ---

# LATEST — 120 source/installed/native verified; commit/CI/publication next

Updated 2026-10-31T08:16:18.768316+00:00. Work SOLO, no Harness/agents/RDPviewer. Canonical checkout
/home/ubu/projects/lay-dependency-updates-20261027, branch codex/browser-matrix-repair-20261027,
HEAD b06a294b (119). Candidate120 is installed on remote KDE; NOT pushed/tagged
or published yet.6/10requested114–123 published. Local GNOME untouched.

Runtime leaf ONLY kde/exact_replay_diagnostics.rs: actual published debug flag
instead of extra diagnostic LayConfig::load; normal backend owner/order same.
Source-only extracted predicate RED/GREEN2tests documented; scope limits
explicit. Other edits are test-isolation metadata plus cfg(test)actual_disable
helper: receive real P2P stamp BEFORE handler, reuse adjacent accepted pattern.
Controlled inverse order reproduced original atomic.active assertion0.043s;
GREEN6original tests/all3helpercallers+negativecontrols. All test bodies,
expected effects,1ms runtime rendezvous,1s fixture bounds unchanged.308other
integration files+exact residual remainder frozen. Old119semantic checks kept
via exact approved-helper normalization; no oldADR/testguard weakened.

FULL-v4 PASS:3617/34targets/494isolates,canonicalexit0,Python430PASS10SKIPof440,
all17gates incllints/audit50/MSRV1.88bothprofiles/installer/Node. All10release
compiler-executable artifacts0warnings. Source1328/SHA565282efa1867e90966cbd1a9882301a7218700458aaf05c436bc0562be1038a;
1332testedsource+metadata inventory. Strict imported73files/preimages, preserved
newer localhandoff onlybecause remote unchanged. Actual failure variantsv1/v2/v3
and non-equivalentdefaultfeaturecontrol retained; do not repeat source gates.
Knownledger expectedfailuresZERO, refreshed from actual full results.

Installed once via release120-refusal-install-v1.py; its stale119versionassert
reportedFAIL AFTER successful copy/restart. Separate read-only verification
confirmed120/all10files/loadedbytes, no secondinstall/restart. Prelaunch tomllib
Python3.10mismatch corrected before mutations. Raw failedreceipt retained.
Current daemon535954 SHA2a8ff8c05b8386fc8092d54a067cfae15c53935894734513d48f32a8cf7075c1;
IME536040 SHAb51116c3b3fbf2af5031d5b3178b594df5fc764aa79ea86ec133956438c75196,
inode24803136;tray536089;globalIBus3168326/config5d02...unchanged,LayRU/layout0.
Post-native identity receipt CACHE/release120-final-loaded-identity.json.
Frozenr19 ordinary after-Space roundtrip native controls2/2PASS/4flips:
Chrome84.406/79.622ms,FF185.095/140.536ms(n2each,unpaired,NO speedup claim).
HistoricalFF165/15/40+Chrome179/0/41=344/15/81of440 NOT rescored.

Source owner release-1.0.120-kde-refusal-config.md; companion ci120-disable-
receipt-order.md / ci120-level-modifier-test-isolation.md and theirnewADRs.
Finalscope initiallyrejected TD163statusdoc asshared; exactmetadata path added
to new120scope/ADR, no code/test/guard change. Recheck full currentdiff before
commit. TD165topstatus corrected to actual119published acceptance.

NEXT: scope+whitespace checks, commit exact reviewed source/docs, verify Git
blobs/executablebits against CACHE/release120-tested-inventory.json (cwdcanonical
checkout), save CACHE/release120-committed-source-identity.json. Push origin+
public devbranch and publicmain, wait all4CI/Security, publish120 ONLY after
identity/CI accepted, previous stable119. Notes CACHE/release-1.0.120-notes.md
have actual source/native facts and NO pendingmarker. Mirror exact annotated
tag origin, verify both actual installed updater entrypoints --check. THEN
continue121–123 genuine fixes, no filler or stopping after120. Private next
KDE isolation map exists; canonical dev-check rsync --delete is already
safe for renames, privateoverlay helper alone lacks delete transport. No new
public tooling bug is claimed from that private limitation. Sound at actual
turn completion only. LastpublicPR query empty; fresh querybeforepush required.

--- Earlier checkpoints (historical) ---

# LATEST —119published6/10; nextoneFirefoxrapid-after-Spacebaseline

2026-10-31T05:18:45.810504+00:00. v1.0.119published2026-10-31T05:14:02Z,canonicalcommitb06a294b51d95e8c19ad25754dcf9b9798a867d7. All4CI/securityPASS;annotatedtag7430a6d8a5ac26ec6fb1e0e37e2fea1c7fbb8235 identicalorigin/public,peelsb06a294b. Actualinstalled updater --checkPASSboth:local115sees119available,KDE119current. Entrypointsresolveinstalledupdate.sh SHAaebfb65e.../publiclatestAPIverified. No localinstall/restart. SIXofTENrequestedreleases114–123published.119DONE_SCOPED,source/native/publicationreceiptsin docs/development/evidence/. Publisher/mirroringallfinished;doNOTrerun. Workingdocupdatesafterpubawaitnextcommit;runtime/sourcecode unchangedafterb06.

119sourcefull-v4:3616Rust/34targets/492isolated,Python430PASS10SKIPof440,othergates/MSRV/all10build0warnings. Testedf4e065cf840f1f440748f620fe9182ae65376e2439bf5e35185fcb87ae69f71e1328files/1332inventorymatchescommittedblobs/executablebits. All10installedfilesandloadedruntimeequalfinalbuild. KDEdaemon3936773SHAedb16210...,IME3936914SHA11be42ba...,tray3934974source8aa9cb8b...;globalIBus3168326 preserved,config5d02dfda...,LayRU/layout0restored. Cargo target11471745024under16GiB. Finalmenu39Qt/15native/2browsercontrolsPASS,menumed41.512ms,p95/max58.479ms(n8,nottyping). SourceKDEmenuonly+privatecfgtestfixtures;commoninput/owner/verifier/controller/hint/deliveryunchanged,bothInlineHintclasses118byte-identical. KSNIprocessflag+onefilterhandlesnativeprojection;oneconfigapplypermenucloseavoidsstartlimit. OwnedtransienttesttraystaleIBUS_ADDRESSrepairedonlyitsunit;no globalenv/theme/IBusrestart. Failedv1/v3recordsretained.

NEXT:select ONE current119Firefoxnormaltext:rapid-five:space baseline beforeanyinputcode. Frozenr19actorSHA360e6b9a...;childFirefoxshim924b5982...;usefreshactualdaemonGUIenv(no staleIBUS_ADDRESS),own newwindow,unchangedcadence/deadlines,nolearningcleanup. ProductionconfigfragmentTrue/debugFalse. Beforemutatingcode,updateexistingrapid-latencymap/owningpreflightanddependencytree;no guessedsharedroutechanges. Sourcepacingfile alreadyallowsword+Space/NBSP,doNOTreviverejectedoldfastcandidate. Historicalfull116220receipt15FAILalllateTARGET_AFTER_NEXT_GESTURE;all5visiblefirstnormaltextcase,nottextloss. Historicalanalyzerv3CROOT/release120-historical-firefox-analysis-v3/report.json showsactive145n med4.117/p957.367ms,closed120nmed136.061/p95227.389/max272.356ms. Oldmetadata/capturewarnings,firstfailedtransitionUNKNOWN. Newbaselineisrequired;no120nativeinput/sourceedityet. Ifshortpass,establishfreshfullaffectedresult;dontinventpatch/release. HistoricalKDEtable344/15/81of440unchanged. Noagents/Harness/localGNOMEinstall/restart/RDPviewer.

# Current state —118 published;119 KDE menu in progress

Recorded 2026-10-31T01:26:03.911359+00:00. Five of ten stable releases published114–118.118 is685ba6f24840bf994669d0c71fd1c8ea268b9326,public latest118,all4CI/annotated public+private tags/actual installed updater checks PASS. Local installed115 was not changed;remote118 remains loaded. Pub evidence release-1.0.118-publication.json. SOLO,no agents/Harness/RDP viewer.

ACTIVE119 FINAL SOURCE: source119-menu-full-v2 is running under guarded remote session1758 (host launcher), no native GUI activity while correctness executes. Focused release119-menu-and-fixture-green-v1 is GREEN:original3fixture identities PASS8.650/1.422/0.065s and actual Qt Context/Trigger+persistence PASS0.266s;runtime118pids/config unchanged. Full-v1 FAIL retained. Final source removes setContextMenu export so existing app popup handles both buttons. Additional TEST-ONLY residuals.rs selected function moves unchanged1s bound per independent refusal fixture;all32case bodies/assertions identical,word_scope.rs unchanged. Measurement receipt measure-v1:9completed,max153.594ms;case10starts954.003ms,aggregate times out1.019s. Timing instrumentation fully removed. Supporting ADR2026-10-31-ibus-fixture-case-budget-119.json and primary119scope record this. No production IME source/timers changed.

After full-v2 passes: strict import via import-verified-source-return.py (newer handoff may be preserved only because returned unchanged), remote inspector release119-inspect-tested-source-v1.py (expects1332files), all10compiler artifacts/0warnings, then release119-menu-install-v1.py. Private native actor release119-menu-native-v1.py prepared/syntax-checked,NOT executed:requires actual fresh panel/icon coordinates and full/install receipts; checksNoDBusMenu,BOTHnativebuttons,6mouse+2keyboardtoggles,samepopup/configreadback,Escape/outside andrawconfig/layout/globalIBusrestoration. Ordinary commands/master/auto-update native mutations are intentionally untested,realQtfixture covers them. Need capture fresh panel after installation;oldLicon1362,878 at1600x900 is only baseline. Actorgeometry321x340 assumes unchangedsource layout and refuses mismatch. Review actor before executing;do not run during source correctness. NewenginePIDsaftercheckboxrestarts must be read for subsequent frozenr19browsercontrols. No119 install/publication yet. After acceptancecommit,immutable1332-file identity,public/privatepush,all4CI,stable119,tagsmirrors,bothactualinstalledupdaters.


119 owner release-1.0.119-kde-checkbox-menu.md,ADR2026-10-31-kde-checkbox-menu-119.json. One production file scripts/lay-kde-tray.py:small CheckboxMenu + in-place boolean refresh. Qt RED proved closure;39/39 focused + extended controls PASS. GUI test isolates QApplication in a child process after earlier combined-process139. InlineHintProjection/View source exact118;all core/IME/GNOME/controller/delivery frozen. No119 install/commit/tag yet; Cargo source119.

Native actual RIGHT click proved second menu owner:Plasma3278174 opens imported QMenu,while LEFT is Lay3119657. Need remove setContextMenu registration,existing _activated Trigger/Context opens the SAME Lay popup. Qt/Plasma official source confirms NO_DBUSMENU semantics. No global Plasma changes/new tray/input route. See baseline popup screenshots/JSON in CROOT.

Full119-v1 finished FAIL on3process-isolated IBus tests (private P2P replay/ACK/handoff-sealed),all other observed targets passed. Do not import/promote FAIL as accepted source. Preserve receipts;diagnose3exact fixtures in canonical isolation with native UI quiet,then full-v2 for the final UI source. Do not modify shared IME runtime/raise timeouts/waive tests. Current remote sources include full-v1 generated graph/manifest;prepare next pin from full-v1 result returned hashes. Active runtime118PIDs daemon3119526,engine3119609,tray3119657/globalIBus3168326 preserved. Target/release engine file may be rebuilt;never clean active release files.

Next:bounded UI second pass+3fixture diagnostic,full119 gates/all10 binaries,strict returned-source import/artifact inspection,controlled KDE install and actual BOTH-button/checkbox config-restoration acceptance,then commit/source identity/CI/release/updaters. Historical browser matrix344PASS15FAIL81BLOCKED/440 unchanged. Initial118Chrome supplemental failure remainsTD167 OPEN;independent fragment migrationTD166 OPEN. No local GNOME install/restart. TD162 remains deferred false-positive proof problem.

## Previous118 verification details (historical record)

Recorded 2026-10-31T00:50:03.512085+00:00. Four of ten requested releases are published (114–117). Stable117 is e65b63b0; no118 commit/tag/publication yet. Work solo, no agents or Harness. No local GNOME install/restart, no RDP viewer.

The minimal118 fix changes only production `src/typing_rule_graph/rules/fragment.rs`: after optional abstention, retain a real ordinary candidate only when its prefix is unchanged, the last token equals exact physical layout projection, and existing TOKEN plain-layout safety permits it. Model/weights/registry/input/ownership/rendering/delivery are frozen. Independent provider prototypes A/B/C were rejected and archived; TD166 remains OPEN. The five original provider tests are unchanged.

Evidence: safe-red-v1 reproduced lost ordinary outputs; safe-green-v1 passes the same real core and five old tests. Strict fixed20k safe-quality-v1 passes BOTH baselines: RU clean10000/restored693/wrong0; EN clean9987/restored2965/wrong0; OFF exact, no lost correct ordinary targets. Thirteen existing clean-English errors remain. Source quality hash21c4d6d7... before version metadata.

Full-v1 stopped before tests on12GiB cache budget. Guarded old debug-test-ELF cleanup freed4050679296bytes, preserved two newest per target and active release IME; no limits weakened. Full-v2 PASS:3616 actual tests across34 targets,492 process-isolated,all other gates/MSRV/all10release binaries. Canonical wrapper exit1 was ONLY old manifest binding; zero ledger rebound after inspecting every actual PASS. Public CI still must give canonical exit0. Inspector verifies1331files,10retained compiler-artifact binaries,zero warnings. Full source1327 SHA58c50f93e3a5daffcac535d23779f1a7e206021b7e865c29fe9b7bee4421ea5e.

KDE118 installed via runtime-control coordinator: daemon3119526 SHA3e6cda78..., IME3119609 SHA7d0d33e1..., tray3119657. Global IBus3168326 preserved; exact user config5d02dfda... with fragmentON restored. Runtime engine factory executes target/release/lay-ibus-engine; never Cargo-clean active release files. No source edits after full gate.

Native `release118-fragment-safe-native-v1`: 5 PASS/1 FAIL/0BLOCKED. Chrome roundtrip PASS60.619/65.235ms; first фиикум Space retained original (FAIL); second prefix case PASS102.119ms. Firefox roundtrip PASS160.003/164.166ms; first/second Space PASS18.675/18.275ms. Initial Chrome FAIL is retained and unexplained. Its debug trace suffix was empty despite actor native_trace_complete=true; do NOT claim complete causal trace. No delivery code was changed to force a result.

Bounded diagnostic `release118-chrome-first-diagnostic-{off,on}-v1`: same Chrome roundtrip+first Space with temporary debug and fragment flag; both modes2/2PASS. Raw config restored exactly after each, no restart/install/code/cadence change. Drained trace files record real gate/apply/lease events for the successful runs, not the missing original failure. Latest ON first Space applied worker5/tail214; original first failure cannot be causally explained from absent logs. These diagnostic passes do not erase the initial failure.

Default restoration confirmation `release118-chrome-first-default-confirm-v1` finished 2/2 PASS with debug disabled and exact original config: Chrome roundtrip50.665/37.949ms, first Space49.817ms. This is not a new6/6suite and does not erase the initial failure (OPEN TD167). ACTIVE: commit the verified118 source, compare immutable blobs with the1331-file tested inventory, push public/main and private development branch, require all4CI success, publish118, verify actual installed updaters and both annotated tag mirrors. No further functional reruns for publication metadata. Then TD165 KDE checkbox-menu UI task.

Next: finish native decision, collect inventory/receipts, update owning docs/tasks, commit verified source, public+private-dev push, all4public CI checks, annotated tag/release118, verify both actual installed updaters and mirrored tags. Then continue119 KDE checkbox UI and later queue; no filler releases. TD163 published117 status corrected. TD164 only safe-fallback part is implemented; independence remains166. Full historical native table344PASS/15FAIL/81BLOCKED of440 unchanged (FFfull116; Chrome historical). Supplemental initial5/6 and diagnostic4/4 are separate, never added into the440 table.

--- Prior checkpoints are history ---

# CURRENT —118 full-v2 RUNNING after safe cache cleanup — 2026-10-30T23:56:09.610695+00:00

Strict20k safequalityPASS andsixfocusedPASS unchanged. Full-v1 infrastructureFAILatmanifest BEFORE tests,notcodefailure:canonicaldiscovery guard12GiB killed13GiB;outer16overrideisnotpassedbyhermeticdiscovery. DO NOT relaxguard/cleanenv. Guardedcleanup deleted4,050,679,296bytes olddebug workspace testELFs,two newest pertarget+allactive preserved. RunningIMEis target/release/lay-ibus-engine2298008SHA5f728b... andisunchanged;release/libs/models/evidence notdeleted. Newphase release118-fragment-safe-full-v2 sameworker-v1,activeSSHsession63249. Preparedinstaller nowmustread full-v2,notfailedv1.4/10published,118NOTinstalled/published.

--- Prior checkpoints are history ---

# CURRENT —118 strict quality PASS;full all-source/all10 gate RUNNING — 2026-10-30T23:49:34.124068+00:00

Remotephase release118-fragment-safe-full-v1,SSHsession9993,worker release118-fragment-safe-full-worker-v1.py. Cargo118now,sourceScopeONEproductionfilefragment.rs;registry/candidates/IDs/rules restoredexact117,51frozenhashes enforced. SafeGREENsix functionsPASS,source1327SHA21c4d6d7... beforeversionmetadata. Strictsafe-quality-v1 PASS:RU10000clean693restored0wrong;EN9987clean2965restored0wrong;OFFexact,0lostordinarycorrecttargets,40000modelparity;proof256.628s. Projectevidence release-1.0.118-fragment-quality.json. OldENclean13errorsremain. IndependentprototypeA/B/Crejected,archivedpatch+inventory,unmetindependence/physical-punctuation/known-originalevidence remainsOPEN TD166. DoNOT reapply failedprototypes orclaimindependencefixed.

Fullgateexpects3616canonical/34targets/492isolated/all10zero warnings,MSRV1.88bothfeatures,lints/Python439+10skips/Node/publicinstaller/audit50. OnlyafterallPASS installKDE118. PreparedNOTRUN release118-fragment-safe-install-v1.py/native-v1.py and privateparent-pinnedactor dependency-browser-native-fragment-safe-r1.py (addsfirst/secondSpacecaseonly,nocadence/deadlinechange);normalcontrolsusefrozenr19 unchanged. Exactconfigpreserved,globalIBusmustnotrestart. No localGNOMEinstall/restart,noRDPviewer.

117e65published/updatersverified4/10;TD163markedDONEscoped,queueupdated4/10. TD165UI next after118publication,TD162rejectedpair-split remainsdeferred. FreshGitHubActionsartifactAPIs public+privateboth explicit0artifacts,none deleted. Sourceprototypesneverinstalled.KDEstill117daemon2297924/IME2298008/tray2298081/globalIBus3168326/config5d02...;nativefulltable344PASS15FAIL81BLOCKED/440unchanged. SOLO/noagents/noHarness. Afterfull:strictimport,retained10compiler-artifactbinaries/KDEproof,commit/publicCI/tag/release118,bothactualupdaters,bothannotatedtagmirrors;thencontinue119–123,no filler releases.

--- Prior checkpoints are history ---

# CURRENT —118 minimal safe fallback6GREEN;strict corpus next — 2026-10-30T23:33:03.517886+00:00

4/10 published114–117;stable117e65b63b0,updaters/tags verified. Independent provider prototypeA/B/C REJECTED/archived CACHE/td164-rejected-independent-providers.patch+inventory,NEVERinstalled. All4other runtimefiles/registry/candidates restoredexact117. Scope nowONE productionfile src/typing_rule_graph/rules/fragment.rs +oneactualcoretest/discovery;newownerrelease-1.0.118-fragment-safe-exact-fallback.md,ADR2026-10-31-fragment-safe-exact-fallback-118. NoOpinion fallsbackonlytorealordinarycandidatewithunchangedprefix/fullkeyprojectionandexisting TOKEN plain-layoutsafety;no newword/weights/IDs/targetL2filters. Fullindependence/punctuation unmetcontractsOPEN TD166,do notclaimclosed. TD165KDEmenu queued separately.

ActualsafeREDv1 newcore101/old5PASS;safeGREENv1 samecorePASS/old5publictestsunchangedPASS,strictsource-returnimported.51frozenhashes/live117PIDs/configunchanged. Cargo still117prototype,no118install/bump/fullgateyet. Nextstrictfixed20k safe-quality-v1must satisfyBOTHoldOFF+oldON baselines:no lostcorrectlegacy,OFFexact;RUclean10000/restored>=693/wrong0;ENclean>=9987/restored>=2964/wrong0. CandidateCpartial/source-word/punctuation regressions retainedonlyasfailedexperimentreceipts.

Privategeneric runner run-lay-guarded-phase.py WORKER PHASE [args] handles safeSSHupload/guardedstream/results download;prepare-fragment-release-source.py usespreviouspin/result;strictimporter nowhasexplicitapproved6testsetforsafe-phaseprefix. Nextfullworker release118-fragment-safe-full-worker-v1.py expects3616/34/492isolated/all10zero warnings,ADRnew. NOlocalCargo/GNOMEinstall/restart,SOLO/noHarness/noagents,noRDPviewer. Fullnative344PASS15FAIL81BLOCKED/440unchanged (FFfull116,Chromehistorical).

--- Prior checkpoints are history ---

# CURRENT —118 variant B strict corpus running,117 live protected — 2026-10-30T22:31:26.407642+00:00

Active remote phase release118-fragment-quality-v2,localSSHsession77106. Six focused tests PASS9.198s,debugcompile24.5s,releaseexample149.484s. Existing model/weights frozen; ONLY new source metadata changed Plain→existingContextual policy after rawdiagnostic proved correct pamelia/бесстрашиях candidates were safety-rejected. Corpus requiresBOTHbaselineOFFandacceptedON retention/clean/wrong gates; do not accept OFF-onlyGREEN. No118versionbump/fullgate/install yet. Rawdiagnostic captured1testPASS butwrapperFAILon --nocapture parse;FAILpreserved,notacceptance. LatestremoteSHAdaemon761d3b.../IME5f728b.../globalIBuscb8cca... match117,config5d02/PIDs unchanged. PrototypeB currentfivecandidatefiles only; NO input/routing/owner/verifier/desktop edits. Fullgate118privateworker prepared,notrun. 4/10stable114–117. Fullnative344/15/81 of440 unchanged. SOLO/noHarness/noagents.

--- Prior checkpoints are history ---

# CURRENT — 4/10 published;118 variant A rejected, not installed — 2026-10-30T22:22:21.820359+00:00

Stable117 e65b63b0 published and updater/tags verified.118 six focused tests pass but fixed20k ON quality regresses vs accepted ON: RU406/97 vs693/0; EN2965/105 vs2961/0 restored/wrong. Private quality worker GREEN compares OFF only; DO NOT PROMOTE. Owner/ADR now record strict BOTH-baseline gates. Next inspect raw fragment proposal/safety/scores, not blind FAST early-return edits: common core already collects with keep_fast=true. Five shared candidate files only;47protected routes/model files and KDE117 runtime frozen. No active heavy worker. No local GNOME changes. Fullnative344/15/81 of440 unchanged. Continue ten-release queue SOLO/noHarness/noagents.

--- Prior checkpoints are history ---

# CURRENT —117 published,4/10 done;118 source quality running — 2026-10-30T22:01:44.727948+00:00

Stable117published2026-10-30T21:50:57Z,e65b63b0c5583c30360f8385509f060d2d0d190c,all4CIchecksSUCCESS. Annotatedtagmirrored/verifiedbothremotes;actualinstalledupdaterlocal115→117available,remote117current,both--checkexit0. Proofdocs/development/evidence/release-1.0.117-publication.json (NEWafter118pin,willnotbeoverwritten). No localGNOMEinstall/restart.4/10done. Source1175e39d358...,1327files;10productionbinariesequalinitial117 nativebytes,so no secondinstall. KDEproc2297924/2298008/2298081/globalIBus3168326,config5d02 fragmentON unchanged.

ACTIVE118 (uncommitted,notinstalled):correction-only registry separation,five runtimecandidatefiles +oneisolatedsemanticcoretest/discovery;model/assets/scoringunchanged47hashesfrozen. NewADR2026-10-31-independent-fragment-candidates-118,baselinee65;ownerrelease-1.0.118-independent-fragment-candidates.md. SemanticREDloss4oldtargets+independenthelloabsent;GREENnew6testsPASS (core+5controls),CROOT/release118-fragment-core-green-v1,result importedstrictly. Two new own providers useexisting directional priorities/family/PlainLayoutAutocorrect/Normal safety,legacyFAST andregistrycallbacks direct. No field/desktop routes changed,no newowner/ranker/executor.

Active remote qualityphase release118-fragment-quality-v1,worker release118-fragment-quality-worker-v1.py,exec48250. Datasetoriginal20000SHA89d13ce432d06b479fefb48d6f8b39b49ec2c75ca9d51dca76fd0823c9fb1c26;rawmodel40000Rust/trainerparity. Perlang OFFmustmatcholdcounts;ONmustnotregressclean/restored/wrong andzero lostordinarytargets. Rejectifanyfails,no wordexceptions. No118versionbump/fullbuild/installuntilqualityPASS. Installed117protected. Avoideditingpinnedcoordinationdocs excepthandoff(skiponlywhenreturnedunchanged);117task/queueDONEupdatesdeferreduntil118import toavoidlocaldrift. Publication117proofnewfileis safe.

Next afterquality:importvalidatedsource,fullsource+all10builds,exactKDEbytes/Spacefirst+second native,commitCI/publish118/updater;thenTD165KDEmenu task119.10releasesrequested114–123,6remain. FullnativebaselineFF116165/15/40,Chromehistorical179/0/41;total344/15/81of440.

--- Prior checkpoints are history ---

# CURRENT — final117 source passed; bytes unchanged; commit/CI pending — 2026-10-30T21:30:31.781297+00:00

Still3/10 stable114115116;117 NOT PUBLISHED. Currentfinalsource1327 SHA5e39d3585d3afeb746a4ce758bc84a5ac7d5829b034d439ae9c4f98dd197b256. Fullphase release117-ci-fixture-full-v1 PASS3615/34/491isolated,canonicalexit0,439Python10skips,all10binaries0warnings. All10production SHAidentical to initial117;runningKDE daemon2297924/IME2298008/tray2298081,IBus3168326 andconfig5d02... unchanged. Reuse original117native2controls4flips;do not reinstall/restart matching bytes. No activeheavyworker now. Production source unchanged45frozenfiles;only residuals.rs6test-precondition lines vs75cd,alloriginalassertions/timers restored. Controlledage RED/GREEN and3narrowcontrols verified.

Resultimport complete. Missing observation DOCUMENT preimage reconstructed from3frozen/immutable proofs in separate import-only audit;executed pin unmodified. All sourcepreimages strict,newer localhandoff preserved onlyifremoteunchanged. Privatepreparer nowcarries allpriorreturned paths. Source ownerci117-completion-cache-fixture.md. Needpostreturnactualscope-check BEFOREcommit,source inventory+immutableblobverification,push,independentCI+Security,publish117/tagbothremotes,bothactualinstalledupdaters. No blindretry75cdoldCI;newfixturechange goesnewcommit. GlobalIBus/GNOME untouched.

NextTD164 private draft CACHE/td164-next-plan-draft.md recommends separate2fragment sources in EXISTINGregistry,legacycallbacks untouched,no new owner/route. Real CLI117OFF/ONlostabbrevproof saved;metadata task164 updated,codeunmodified. TD165queuedKDEmenu. Fullnative baseline344/15/81of440 remains FF116/historicalChrome;15FAILrapid-five. Continue10releasequeue,notstopafter117.

--- Prior checkpoints are history ---

# CURRENT — final fixture restored; full117 gates running — 2026-10-30T21:15:15.620716+00:00

The temporary diagnostic mentioned in prior history BELOW HAS BEEN REMOVED. Current residuals.rs matches the exact75cd base plus ONLY6 prerequisite lines (private l2_surface_words_by_usage(1) assert before the existing cold-cache-miss check). All original assertions,1second test wrapper and150ms production display policy retained. No diagnostic sleep,print,early stop or timeout bypass remains.45production hashesfrozen,zero production logic changes since75cd.

Controlled clock RED ci117-cache-age-red-v1:worker cachedbefore=true,after age boundary=false. Otherwise-identical GREEN after refreshing private dependencies BEFORE cold request:before/after=true. Exact CI age was not logged; do not invent it. Then full original target +pending no-publication+stale config/material controls3/3PASS at2CPU; ci117-cache-final-v1. Final source imported with preimage/hash verification.

Active remote full source/build phase release117-ci-fixture-full-v1,worker release117-ci-fixture-full-worker-v1.py,exec41457. Expected3615/34targets/491isolated,all10builds; keep actual verdicts. Still stable116,3/10published,117 nottagged. Public/privateHEAD75cd,its CI mainfailed oldfixture;other requiredchecksPASS. KDE running previous verified117source094e5c50/proc2297924,2298008,2298081/globalIBus3168326,config5d02...fragmentON unchanged. No local GNOMEinstall/restart.

Do not edit pinned source while full job runs. Private importer may preserve ONLY a newer local handoff when returned handoff bytes are proven identical to its pin; all source still strictly preimage/hash checked. New owner ci117-completion-cache-fixture.md andADR2026-10-30-ci117-cache-fixture,active baseline75cd,test-only sharedscope. After fullPASS import,actualscope-check BEFOREcommit (generated manifest/known ledger alreadydeclared);source inventory,commit/push,CI+Security,publication,tag bothremotes and both actual installed updaters. Current real fullnative table344/15/81 of440 remains historical. TD164 CLIoff/on lostabbrev proof saved,TD164/165future stillqueued.

--- Prior checkpoints follow as historical evidence ---

# ACTIVE —117 blocked by CI fixture, controlled test-only diagnostic — 2026-10-30T21:04:30.565535+00:00

SOLO,noHarness/noagents. Still3/10 published114–116; stable116. SourceHEAD75cd04bdbd3394f15ccf29dbcd9e0d4f8f23f67c pushed bothremotes.117 NOT TAGGED/PUBLISHED. Two CI attempts:2e81 failed scope (generated manifest/known ledger seams),fixed75cd;75cd scope/MSRV/security/installerPASS,but main failed one process-isolated existing IME test at residuals.rs:2277 cachedSome. Raw CACHE/release117-ci-job-v2.log. No blind rerun. No active publisher session now.

KDE currently117 from initial validated source094e5c50d440fa35c1235d40855e944be583e70409777fc4249832655e338446,1327files,3615Rust/491isolated/439Python10skips/all10release0warningsPASS. Runtime daemon2297924 SHA761d3bd6cda1fb008b008df9e66cbf5b139af1821f7c3a00d9af6c870dd11c97;IME2298008 SHA5f728b4f8f5578bf8cb5f869d45524a33e0cbf7bfe8011704ecf66853c9c4b03;tray2298081;IBus3168326 unchanged. Config5d02... fragmentON preserved. Initial install failed only savedengineRU→US assertion; normal IBus activation restoredRU, no secondrestart. Audited recovery/native2controls4flipsPASS. Actual publish must still require complete CI and both installed updater readbacks; no local GNOME install/restart.

CI diagnosis: exact old test had3 fresh isolated PASS at2CPU,~1.018s each (CROOT/ci117-worker-isolation-diagnostic-v2/result.json). v1 parser failed on nocapture interleaving but actualtestPASS;v2 uses show-output. Found cached getter refuses when usage memory older1s; refreshing dependency snapshot clears completed cache, so doing it AFTER worker is rejected. Existing l2_surface_words_by_usage(1) forces private dependency refresh WITHOUT computing readout; candidate fix is use before required cold-cache miss/worker. Cacheability can also be false for unavailable canonical material; CI cause remains hypothesis until controlled proof. No production edits planned.

CURRENT SOURCE IS A TEMPORARY DIAGNOSTIC TEST VARIANT, NEVER COMMIT IT. Only residuals.rs selected test changed: outer1sec fixture wrapper bypassed, explicit privateusage snapshot aged800ms, cached-before check,wait300ms,originalcache assertion,then stop at cache boundary. Actual worker150ms deadline remains unchanged. Saved exact canonical file CACHE/ci117-residuals-base.rs and variant CACHE/ci117-residuals-expiry-red.rs. Must restore all sleeps,bypass,early-stop/prints from base before final source. Owning docs ci117-completion-cache-fixture.md,new2026-10-30-ci117-cache-fixture ADR,active scopebaseline75cd test-only;45production hashesfrozen. Remote guarded phase ci117-cache-age-red-v1 worker ci117-cache-age-worker-v1.py was started (exec18394). Never import diagnostic archive; use returned hashes only to prepare next overlay. Local handoff may be newer than pinned metadata deliberately; no diagnostic importer will overwrite it.

TD164 future read-only diagnostic already proved one lost ordinary output through exact117 production CLI with isolated layout-only config: фиикум→abbrev withfragmentOFF,ONnone;layout_ru_to_en candidate disappears.8inputs×OFF/ON;CROOT/td164-runtime-cli-diagnostic-v1/result.json,CACHEsame-result.json. No source/runtime/config changed. Some otherbaselineoutputsabsent in both modes;do not claim all Russian words blocked. Actualconfig fragmentON,auto_switch_layout missing→defaulttrue,auto_replace/typing_assisttrue,textbackendIME,layoutbackendKDE. TD164/165 future usertasks stillnotfixed. TD162 variantsrejected,no reapply. Full native table344PASS15FAIL81BLOCKED/440 (FF116165/15/40,Chromehistorical179/0/41).

# CURRENT117 validated, CI/publication pending — 2026-10-30T20:17:59.088524+00:00

Still3/10 published until117 publication/updater receipts. New117 owner release-1.0.117-debug-config-publication.md (not rejected glued experiment). Full source1327 SHA094e5c50d440fa35c1235d40855e944be583e70409777fc4249832655e338446,3615Rust/491isolated/439Python10skips/all10binaries0warningsPASS. Source returned and imported with preimage/hash verification. RuntimeKDE117 daemon2297924 SHA761d3bd6cda1fb008b008df9e66cbf5b139af1821f7c3a00d9af6c870dd11c97;IME2298008 SHA5f728b4f8f5578bf8cb5f869d45524a33e0cbf7bfe8011704ecf66853c9c4b03;tray2298081;globalIBus3168326 unchanged. Initial install selectedUS instead of savedRU; rawFAIL retained. Existing IBus activation restoredRU without restart; audited deployment-recovery-v1. Native2controls/4flipsPASS,config5d02... exact. User fragmentflagON preserved. No local GNOME install/restart.

Before publishing117: verify exact committed source inventory, freshPRs, CI+Security, annotatedtag bothremotes, latestrelease and both actual installed updater --check. Then TD164/165 future user reports, one at a time, map/RED first. Full matrix remains344/15/81 of440,not a fresh117 full matrix. Adjacent KDE refusal debug loader still untouched; owner names this remaining source-only finding.

# Active update — 2026-10-30T19:50:07.945148+00:00

Resume TD163, the existing KDE debug probe config-publication bug. New117 owning preflight: release-1.0.117-debug-config-publication.md and 2026-10-30-kde-debug-config-publication-117 ADR. TD164/165 remain queued for later as user requested. TD164 initial hypothesis that legacy FAST accepts allow_layout_auto=false was disproved by rules.rs::apply_fast_layout_en_to_ru: it also requires layout_auto_allowed(ctx). Do not bypass this gate or revive the rejected TD162 split. No runtime source edited yet.

# CURRENT — 3/10 published; next TD163; TD162 rejected, no117 install

Canonical HEAD7d9787bf46cd1acf0393805735d4655da3500b6c,stable116.
CI41104503630/Security41104503608PASS;tagb477b817bothremotes/updaterbothPASS.
All canonical production/testfixtures/version files RESTORED exactlyto116:1327
sourcefiles SHA545f941b2b3cc8fa18b25600fc03ec607d6c0e630015bcc0f76e872fd74efcbe.
Remote source restored underguard CROOT/td162-source-rollback-v1/result.json,
same exacthash,NO BUILD/INSTALL/RESTART. Current source edits are documentation
only(rejectedexperiment record,newTD164/165reports,queue,publication116proof).

TD162 тыего remainsOPEN,user originallyaskedfuture. Afunction-chain variant
failed;Bknown-part/min3passed3focusedtests but FULL512.54stestfailed existing
test:typing_assist_mixed_corpus::live_journal_false_positive_candidates_are_rejected:
таможе(space) became там(space)оже(space). DoNOT alterthat existingtest.
Diagnosticsv5:его/оже bothpart=true,function=false,clean=false,reference=true,
hot=true,common=false,standalone=false. No simpleboolean proof distinguishes.
BNEVERinstalled,noreleasebinarybuilt,notpublished. Full receipts/patch archived
CACHE/td162-rejected-candidate*. Runtime2files/fixtures/versionsrestoredtogether.
DoNOT reapplyA/B;new pair-level evidence/calibration required. Oldowner117glued
file andADR are explicitlyREJECTEDexperimentlabels,notanactual117release.

Next userTD164 P1.fragmentONblocksnormalSpace. Readonlyinspection shows
fragment.rs::apply_with_mode globallyrequiresctx.allow_layout_auto forFAST
aswell asLAYOUT rules. Hypothesis:legacyFASTallowsfalse inexact Space path.
Needfreshgraph andactualsrc/typing_rule_graph/rules.rs:60,then realOFF/ONcommon
pipelineRED beforeanycode. No runtimefix yet. Avoidguessedlayout.rs(pathabsent).
Read dirs via rg--files first. Skillscan97896 mayhavecompleted,polloncebefore
query. SOLO/noHarness/noagents,no localGNOMEinstall orRDPviewer.

Latestremote observed116:daemon1843722(same41acd9b9bytes,externallyrestarted),
IME1751630/1399f098,tray1751764,globalIBus3168326unchanged. CurrentUSERconfig
SHA5d02dfda,fragment_layout_autoflip=true,auto_replace/typing_assist=true.
Neverrestoreold6e62snapshot. Actualsource117full-v1stoppedbeforeoverlayonstalePID;
v2freshPIDguardranfullsourceFAILEDgatingabove. All jobsfinished,no currentheavy
worker. Cargo cache54e7bytesoldrlib/rmetacleaned,noexecutables/models;onephase
configuredexistingCargoceiling16GiB(363GiBfree),defaultguardfileunchanged.

Privateprepare-fragment-release-source.py nowcarriespreviously touchedpaths
innewsnapshot so rollbacktoGitbaselineactuallypropagates toremote. Native
gluedactors/install117helperspreparedbutNOTRUN;do notuse forTD164withoutmap.
Current native table344PASS15FAIL81BLOCKED/440;FF116165/15/40,Chromehistorical
179/0/41;all15FAILrapid-fiveafterSpace. No updatedfulltableon117.

# Current — queue3/10 published;117 TD162 candidateB short proof running next

116 stable published2026-10-30T18:33:54Z,source7d9787bf46cd1acf0393805735d4655da3500b6c;
public CI41104503630/Security41104503608SUCCESS;tagb477b817 onbothremotes verified.
Actual installed updater local115→116available,remote116current. No localinstall.
Source bytes equal tested545f941b/1327files. Publication proof nowin evidence.

Current117corecandidateonly glued_phrase.rs+guards.rs;no version bump/installyet.
A(helperfunctionchain)rejected,notreapply. Actualdiagnosticv2/v3:exactparts known,
его notfunctionlist;weakтылего/тчего proposals veto split;no directauthorizedtypo.
Bretains ONLYzero-repair known confident split pastproposalveto;existing confidence
minright3(alignssplitter).User-case passedpure+connected;initialnewsyntheticgoldens
wyemu/capswerewrong:clean surfacecertificate mustpreserveвыему,casingpolicy
ЯБЫЛ→Я был/ТЫЕГО→Ты его. v4explicitobservationonly confirmed,notproductverdict.
Correctedgoldensbeforegreen-v3,keptuser-case/context unchanged;runtimeBunchanged.
Next same3semanticchecks;thenwidernegative/sourcegates,117metadata/fullbuild,
remoteKDEinstallandnativefirst/secondwordproof,commit/CI/publish/updater.

IMPORTANT freshrealremote config now fragment_layout_autoflip=true(userchanged);
neveroverwritewithold116snapshot6e62. Installerreads/preservescurrentconfig.
TD164P1fragmentONblockslegacySpace andTD165KDEmenucheckboxcloses savedfuture
reports,notreproduced/fixed.117scopeexplicitlydeclaresnewMarkdownnotes.
SOLO/noHarness/noagents. Keep116native344/15/81historicaltableuntilnewfull.

# New queued user reports — 2026-10-30T18:11:30Z

TD164 P1: fragment autoflip reportedly does not work and when ON blocks the
legacy Space flip. TD165 P2: KDE menu closes after checking each option;keep
open for consecutive toggles. Exact tasks in tech_debt/164-*,165-*. Not reproduced
or fixed. User explicitly requested later work; do not interrupt current116
publication/TD162 candidate. Queue updated;117 scope explicitly includes these
new Markdown notes to avoid repeating116 CI documentation-boundary omission.

116 publication target now7d9787bf(metadata-only successor of5a9f952c).
FirstCI41102635991failed scope on newTD162/163notes,guard unchanged. Addedexact
2docpaths,source/compiler/nativebytes unchanged and independentlyreverified.
Pushedboth; watcher78561 awaiting actual newCI+Security. No116tag/releaseyet.
117draft restored locally,candidate plan/testfixtures ONLY,no runtimefix.
Real production splitterRED CROOT/release117-glued-diagnostic-v1;nextconnected
existing fixture tests on old baseline,then one existing predicate gate.

# Active queue2/10 — 1.0.116 verified, commit/CI/publication next

Canonical HEAD21f82bb2 (stable115);current116source+metadata+evidence dirty.
Stable115 actual installed updater local+KDEPASS;local user independent update
finished naturally. No local install/restart by this task. SOLO/noHarness.

116 source COMPLETE:3614/3614,34targets,490isolated,439Python10skips,lints/MSRV/
architecture/all10binariesPASS. Actualsource1327files SHA545f941b2b3cc8fa18b25600fc03ec607d6c0e630015bcc0f76e872fd74efcbe.
Bootstrap wrapper exit1 retained(obsolete manifest after3newidentities),actual
zeroledger renewed;independent CI must accept final ledger. Owner/ADR116.
KDEONLYloaded116:daemon1751547/41acd9b9,IME1751630/1399f098,tray1751764;
config6e62exact,globalIBus/models/localGNOME unchanged.

FullFirefox220 COMPLETE165PASS15FAIL40BLOCKED. Exactprivateiframe row155PASS;
newprefixbranch observedin normalplainTab row80PASS. Globaltracerotated,211
complete individualtraces/9incomplete. 10FAIL→PASS,4PASS→FAILtiming,2PASS→BLOCKED,
2BLOCKED→PASS. All15remainingFAILrapid-fiveafterSpace. No causal speedupclaim.
Chromehistorical179/0/41;aggregate344/15/81/440. FreshChrome+FFroundtrip2/2cases
4flipsPASS,configrestored. Native/source receipts in evidence/release-1.0.116*.
FinalnativeCROOT/release116-final-native-v1/result.json;fullCROOT/release116-firefox-full-v1.

Next:commit exacttestedsource/docs,verifytree againstclosure,pushboth after
PRcheck,waitCI+Security,publish116,actualinstalled updaterboth. Then3/10.
No morebuild/inputduringpublication. WhilecloudCIruns,diagnoseTD162 separately
without changing frozen116files;private diagnostic preparedCACHE/release162-glued-diagnostic.rs.

Latestuser clarified original independent ENG↔RU fragment contour: already
implemented/published113,f7e1ccfc,owncompact687981byte model,menu switch default
OFF,source files layout_fragments.rs and typing_rule_graph/rules/fragment.rs.
No originalCarambaweights copied;10/10focusednative113;scope/qualitylimits in
fragment-layout-contour-2026-10-30.md. User now asks to record «тыего»→«ты его»
for a followingrelease:TD162 tech_debt/162-glued-pronoun-space.md,next after
current116. Reuse glued_phrase.rs,prove first candidate loss,no deliverychange.
Userexplicitmemorynote saved in extensions/ad_hoc/notes;no runtimecode forTD162.

SOLO/noHarness/noagents;continue user10realreleasequeue,do not claim10done.

---

# Historical queue1/10 — 1.0.115 source/build verified,publication pending

Stable114 published/readback verified below. Canonical baseline740fc6ec.
Current115 changes ONLY scripts/test_lanes/discovery.py plus its tests,version
metadata,owning docs/scope and generated graph/actual zero-failure observation.
Preflight:release-1.0.115-compiler-diagnostics.md;ADR compiler-diagnostics-115.
Short source7/7 PASS;real failing rustc crate now preserves101/E0425/item/span.
Full guarded remote release115-full-v1 COMPLETE:canonical3611/3611,487isolated,
439Python(10skips),lints/MSRV/architecture/all10release binariesPASS,0warnings.
Imported20frozen-prehash-verified source/graph/registry returns. SourceSHA1adba121.
Remote runtime remains114;115 is NOT installed or published. No GNOME changes.
Frozen37 adjacent runtime/integration hashes. No agents/Harness.
After gates:import source returns with pinned prehashes,preserve this handoff;
commit/push,CI+Security,tag/publish115,actual installed updater both machines.
Only then count2/10. Do not rerun complete source tests for report/import errors.

Private next candidate116 map/test atCACHE/release116-prefix-progress-*. It
targets observed Firefox row155 interleaved owned append between delayed
strict-prefix SSI callbacks. No116 canonical edits yet. Need real adapterRED
and actual selector before any runtime fix; don't treat forced Firefox=true
fixture alone as proof that actual native dispatch recognizes the sequence.
No new ownership/cache/retry/deadline or unknown-start bypass. If mechanism
cannot be proved, retain nativeFAIL and move to another independently proven
defect rather than inventing a fix/version. Continue until10 real releases.

---

# Queue1/10 — 1.0.114 published2026-10-30T16:19:38Z

Canonical tested source7b7fe8e033d92e411068d598ae70d18fb81a4323 tagged114
on both remotes; public stable release/readback confirmed. Public CI41018481523
and Security41018481499SUCCESS. Actual installed updater local113 sees114;
remoteKDE114 confirmscurrent. No publication rebuild or runtime restart.
Publication proof:evidence/release-1.0.114-publication.json.
Continue115 compiler diagnostic preservation in tooling only,then remaining8.
No agents/Harness,no localGNOME installation. Earlier checkpoints preserved below.

---

# Historical 1.0.114 publication pending — queue0/10,2026-10-30

Canonical HEADac50a781 plus final verified114source/docs/graph/zeroledger.
Read release-1.0.114-firefox-closed-pacing.md for exact source/native scope.
Onlyruntimeleaf KDEFirefox pacing; no agents/Harness or localGNOMEinstall.
Full-v2 canonical3611/3611,all10binaries0warnings,lint/MSRV/architecturePASS.
Actual KDE114daemon3962945/ae2844b2,IME3963028/3f581662,tray3963083;
config6e62exact,globalIBus3168326/originalFirefox988548/models preserved.
Final installednative2cases4flipsPASS;postTab16/16PASS. FullFirefox159/21/40
retained,Chromehistorical179/0/41;aggregate338/21/81/440. No all-FAILfix claim.
Next commit/push exactsource,waitpublicCI+Security,annotatedtag114andrelease,
verifyactualinstalledupdater local+KDE --check. Then count1/10andcontinue9.
Do not rebuild/retest accepted artifacts for publication. Privatecompletion
docs may follow without moving publishedtag. MandatoryPRcheck currently0open.

Following bounded candidate: Cargo discovery currently discards rustc JSON
errors in a deletedTemporaryFile,leaving only 'could not compile due previous
error'. This hid currentfull-v1 unavailablecachedaccessorerror. Map
scripts/test_lanes/discovery.py::cargo_discovery579 -> compile subprocess
JSONstdoutTemporaryFile -> raiseDiscoveryErrorwithoutreport. Puretooling fix
withred/greenfixture inexistingtestlane unittest,zero deliveryroute changes.
Do not start code before114publication; source/rawnativeC03 issue remains
separatelymappedbelow and must not be erased.

---

# Active2026-10-30 — ten bug-fix releases requested,0/10published

Read ten-release-queue-2026-10-30.md and release-1.0.114-firefox-closed-pacing.md.
SOLO/no Harness. Canonical HEADac50a781; current source version114prepared,
NOT published. First release scope is measured KDE Firefox pacing improvement,
NOT a claim that all rapid failures are closed. Only runtime leaf changed:
src/bin/lay_daemon/kde/firefox_replay_pacing.rs;36adjacent hashes frozen.

Source phases: REDv2 exact test proves old closed-word rejection; green-v2
340hermetic daemon testsPASS and0warning build9c487a4f. Raw broadtestv1
was canceled as invalid unisolatedsetup; REDv1 selected0, retained notPASS.
Diagnostic trace41e5a3f built/installed temporarily and proved fast_profile=true
for9observed calls,7closedwords. Current actual daemon3835325/9c487a4f
is the noninstrumented candidate restored for a controlled comparison.
IME3131409/30ceff92,tray3131455,IBus3168326,originalFirefox988548 preserved.
ConfigSHA6e62restored; no localGNOME install/restart or globalIBus restart.

Native full220 CROOT/release114-native-full-v1:159PASS21FAIL40BLOCKED
420.807s;20rapid-space lateness +1privateiframe-rich consecutive-active
context_authority refusal BEFORE pacing/delegation. Delta vs oldfull:4FAIL
toPASS,1PASStoFAIL,3emailPASStoBLOCKED, so netPASS unchanged.
Chrome historical179/0/41/220 remains; aggregate338/21/81/440 mixes explicit
oldChrome evidence with newFirefox. No universal0FAILclaim.
Rapid115samples candidate median140.06,p95239.59,max292.76ms vs historical
184.47/281.80/303.34; historical comparison is NOT one-variable causal proof.
Samehost original1139e06bae7 vs candidate9c paired prefix5latencies:
median161.23->131.55ms,max175.93->244.97,bothrapidFAIL/all5targets.
Do not claim maximum improvement or statistical certainty from5samples.
PostTab native v2:plain+iframe2/2cases16/16flips,decoder/heldShiftPASS.
Nativeprefix once4PASS laterprofile3PASS1FAIL; retain both, no retry-green.
Use existing kde-firefox-space-readback-full-controller-v1.py with owned
processa11y flag,LD_PRELOAD alignment and --deployment actual receipt.
Do NOT use release114-native-controller-v1.py for standard acceptance.

Current longsourcegate:session30114 CROOT/release114-full-v2 under dedicated
20CPU/24Ghigh28Gmax/1Gswap/512tasks/target12GiB guard. Local script
CACHE/release114-full-worker-v1.py. format/architecture-refresh44.661s/
manifest25.123sPASS; actual complete canonical3611 execution running.
Full-v1 failedcompile from attempted private current_config accessor; fixed
in sameleaf to alreadycompiled LayConfig::load, no shared visibilitychange.
Six generated graph files returned from full-v1 accepted with hashes. Local
graph now will become stale again after corrected leaf; import full-v2returns
ONLY after frozen-prehashchecks. Current Wclosure is bound by
release114-full-v2-pinned.json; remote synthetic GitHEAD never authoritative.

Next:finish source/full build gates; inspect actual3611statuses inclprocess
isolation487 rather than guessed14. Worker handles complete logs correctly
and binds actual release114 observation. Do not rerun completed alltests
for reporting errors. Then retain exactall10artifacts, install114on KDE using
existing coordinator(daemon+individualIME+tray,keepglobalIBus), tiny native
version/controlcheck, strictsourcecommit,pushpublicmain,CI+Securitygreen,
annotatedimmutabletag/release114,installed updater--check local+remote.
No manual localGNOME installation. Count first release only once published
and verified; continue remaining9 without ending turn.

Following priorities:newnative row155 complete trace shows pendingReset
receipt mismatch(revisiongap2,identity_current=true,initial_matches=true),
then current managedword witness absent,unknown_start,manual RPCrefused
context_authority. Map authority.rs823 -> observation.rs935 +KDEFirefox
preedit_echo. Need causal reducer/adapter fixture BEFORE any edit; do NOT
weakenunknown-start authority or edit commonSafetyGate/output. Initial hint
KWin80ms deadline remainsstageUNKNOWN; actual privateIBusfixture unknown
context alsoopen; GNOME Chrome styling report retained separately.

Profiling:actual83daemonKWinqueries plus7IME, layout-list outlier74.7ms.
Correction to earlier inference: X11Output::prepare does NOT call KWin;
its device/focus checks are separate. ActualKWin requests occur in existing
focus/layout guards; don't batchoutput based on the rejected inference.
No IBusCLI callsobserved in measured5gesturewindow; no speculativeCLIpatch.

Private prepare-fragment-release-source.py fixed to queryunknown remote
filehashes rather than assume remote docs match localHEAD. Overlayprechecks
and fullclosureguards remain. Current fullpin includes handoff oldercontent;
localhandoff update is docs only, preserve it when accepting generatedreturns.

---

# Current handoff — 1.0.113 COMPLETE, 2026-10-30

The requested isolated fragment-layout contour is implemented, source/native
verified and released. User required completion through release; no Harness or
subagents were used. Do not resume the old Firefox work automatically.

Authoritative source checkout:lay-dependency-updates-20261027,
branch codex/browser-matrix-repair-20261027. Published main/tag v1.0.113 point to
f7e1ccfc855df4c33e2e82d681ba2573e8337059. Any later commit in this branch for
this completion is documentation only. origin/main was not changed.

Read release-1.0.113-publication.md,fragment-layout-contour-2026-10-30.md and
its evidence JSON. CI40993117623 and Security40993117500 are SUCCESS; complete
canonical3611/3611,all10 installed release binaries,strict lint/MSRV PASS.
10/10 focused native Chrome/Firefox KDE checks including automatic both
language directions,first/second word,physical punctuation-key letters and
manual roundtrip after Space. Real menu:Lay enabled1,new switch0 after config
restoration. Exact config SHA6e62eab3c278c733fedb47e45ef6306105a3700222cf9f7a4e5bff4bd92264d1.
KDE installed1.0.113:daemon9e06bae7cce850f507117d5fd0ec99164e8ef48e1d8736f463cf537fe24bd7d8,
IME30ceff920904527a5176f415895a755375a2aa54d33e0de632b4afce9447bed9.
PID values from earlier entries are historical; query current owner.
Local GNOME installed1.0.112; actual menu updater --check offers1.0.113.
No local installation/restart performed; user receives updates through menu.

New flag fragment_layout_autoflip defaults OFF,uses existing config publication
and menu switches. New pure scorer/adapter only; no IME,decoder,focus,ownership,
keyboard,manual Double Shift or output source changes. Own compact model687981
bytes; not original Caramba weights. Exact reproduction and40000 interpretation
parity verified. Quality tradeoffs and13 residual clean EN ambiguities recorded;
no perfect-accuracy or Wave-promotion claim. Private no-display client failed
before first scenario with flag ON/OFF; not counted as functional PASS.

Historical FULL native matrix unchanged:Firefox159PASS/24FAIL/37BLOCKED/220;
Chrome179/0/41/220;total338/24/78/440. Focused10-case result never replaces it.
Prior Firefox initial KWin query80ms hint refusal remains a separate,unassigned
issue; no timeout/route fix was made in this release.

---

The following entries are historical and superseded by the current handoff
where source/version/runtime/task status differs.

# 2026-10-30T00:02:46Z — COMPLETE: accepted112 published and main CI green

Stable release v1.0.112 is published at immutable tag1ea1d44e. Public main
fca6f842 carries the strict zero-failure ledger rebinding and two equivalent
lint-only fixes. CI40973676834 and Security40973676896 SUCCESS:34 targets,
3595/3595 tests,117/129 exact dead-code inventories, both Clippy profiles,
syntax, CLI smoke, actual all-release-binary build and ngram cache PASS.
Read CACHE/release112-final-ci-readback.json for verified final state, plus
release112-final-publication-readback.json and release112-final-installed-runtime.json.
Both actual installed updater entrypoints discover111 ->112. Read-only current
CLI version remains111 on both machines; exact remote daemon35527e63/IME05d62373
processes still running. No local/remote installation or service restart in
publication/CI work. Source code and field behavior are frozen. Private completion
documentation may follow fca6f842; that docs-only commit does not reopen checks.

NEXT single native issue: KDE Firefox initial foreground observation may exceed
80ms before any geometry/font sample, leaving typed text intact and no published
suffix. Actual failing stage within KWin query still UNKNOWN. Map:
docs/development/kde-firefox-focus-deadline-map-2026-10-30.md. One unchanged-runtime
profile3/3PASS did not reproduce or close it. Do not repeat identical inputs for
green, raise deadlines, cache authority or edit common delivery to hide it.
Historical full matrix stays Firefox159/24/37/220 and Chrome179/0/41/220.
SOLO, no Harness/agents, preserve GNOME, independent existing KDE providers and
the common correction/ownership/verifier/executor contracts.

# 2026-10-29T23:29:55Z — publication CI lint fixes ready

Accepted v1.0.112 remains published and its tag/runtime unchanged. Strict ledger
binding repair59dd712a passed all3595 correctness/package tests in publicCI
40970843062. That run then exposed hint_view argument-count lint. Guarded
follow-up additionally exposed one manual two-character comparison in the
KDE Firefox receipt leaf. Changes: one item-local expect attribute and one
equivalent character-array pattern; no delivery/ownership/rendering behavior
change. Five exact receipt tests and both complete feature-scoped lint routes
PASS, graph/binding/receipt refreshed. Owner: release-1.0.112-ci-lint.md.
CROOT/release112-ci-lint-v2/result.json retains both source closures and steps.
No runtime install/restart. Remote accepted process identities unchanged.
Both installed menu updater entrypoints discover111 ->112; private receipt
CACHE/release112-installed-updater-readback.json.
Next: push lint-only repair, read automatic CI to its final release-build step.
Final CI readback will be saved at CACHE/release112-final-ci-readback.json.
Firefox80ms initial KWin-query timeout remains OPEN; profile3/3PASS did not
reproduce it. Preserve historical browser matrix338PASS/24FAIL/78BLOCKED/440.

# 2026-10-29T23:06:34Z — next task: strict CI observation binding

Accepted 1.0.112 is published at public commit1ea1d44e and immutable tagv1.0.112.
Automatic CI40967927798 executed all34 targets/3595 selected tests with0failures,
then rejected stale known_failures.json manifest binding. Later lint/build steps
were skipped. New owning preflight: release-1.0.112-ci-binding.md. Repair is only
ledger hash/path metadata + real CI observation, zero allowed failures, no runtime
source, installs, services or input changes. Existing loader and scope check PASS;
commit/push and fresh automatic CI readback remain pending.

# Publication1.0.112 verified; next task is KDE Firefox desktop deadline

# Lay1.0.112 publication — verified

User accepted the running version and requested push. Published2026-10-29T22:43:44Z.
- Public release:https://github.com/radislabus-star/lay-public/releases/tag/v1.0.112
- Public main and peeled annotated tag:1ea1d44e6723c86e7553681de966e41ee39e6891.
- Annotated tag object:0429988c5956038f625023020553f4f401ad42d6.
- Latest-release endpoint independently returns tagv1.0.112,draft=false,prerelease=false,target1ea1d44e.
- Private development branch and tag pushed too; private origin/main remains untouched.
- No build/test/install/service restart/browser input repeated for publication.
- Only source graph/binding/receipt metadata generated for the selected accepted source.
- Original accepted daemon pacinga3815fec is in commit. Held uninstalled experiment5d13ae93 is archived privately as CACHE/held-firefox-closed-pacing-20261030.rs; active checkout restored to published original bytes after preserving it.
- Remote accepted runtime remains IME05d62373 +daemon35527e63 +trayc40bd104/module47768ee0, version1.0.111 until an authorized update. Publication metadata version1.0.112 does not claim new installed binaries.
- Previous1146/1146IME,23/23focused,zero-warning build,terminal4/4cases20/20flips and limited Firefox typography evidence reused. Initial80ms Textarea desktop-observation failure remains open; no0FAIL claim.

Next owning preflight:docs/development/kde-firefox-focus-deadline-map-2026-10-30.md. SOLO/no Harness/agents. Fresh diagnosis only; accepted runtime and local GNOME preserved.

# 2026-10-29T22:05:07Z — installed terminal fix and Firefox natural suffix

Current remote KDE runtime: IME2201796 SHA05d62373257e754997a6f742067036aa3519398fbe18acab1d73b392acc7eb6d, sourceab56c71b484c1bd638cc62588120b84621d3f20224f9279ca4d59c38b707a942. Tray2201829 scriptc40bd1041b25b1230e9dc511ac6ea91bc5f24d21c47d60b2846007adb77696f3 +module47768ee05f1f082fdf56ae8ab6d08c8da736e7f0ecade9be9d1e5ba90018c85f. Daemon1722873/35527e63, global IBus3168326, original Firefox988548 and config6e62eab3 unchanged. Final readback confirms installed/loaded IME equality and exact tray assets. Only KDE tray and idle IME factory restarted. Local GNOME untouched. Version1.0.111 unpublished; no push/release in this turn. Held fast daemon source remains UNINSTALLED. SOLO/no Harness/agents.

Terminal Double Shift: first shared failure was selected US decoder despite actual Cyrillic native literal with KDE RU. Narrow existing KDE terminal observer now sets existing decoder from actual observed script. Native final after Firefox style: Konsole/Kitty active+Space4/4cases20/20flips PASS, with deliberately mismatched US/RU entry before each, next-letter f, final physical KDE group0, sourceUS, complete per-case trace, all original state restored. Median20.392ms, p95 nearest-rank24.774ms, max24.779ms, second Shift PRESS to actual raw-PTY receipt including12ms hold; cadence187-201ms, not unrestricted burst/render latency. CROOT/kde-terminal-after-firefox-style-native-v1/result.json.

Firefox style: current field font DejaVuSans18px, old Qt theme NotoSans17px caused separate-element appearance. Display-only font metadata is read in KDE/Firefox adapter and carried by optional suffix_style. Renderer uses matching native character-height font and grey glyph-only mask; no frame or opacity; no guessed horizontal shift. Terminal frames omit this data and old renderer pixels match exactly in2 controls. Full source1146/1146 +focused23/23, Python protocol4 +Qt renderer probe, zero-warning IME build75.237s, graph and exact+1 test registry passed. All43 protected integration files unchanged. Read-only bus call visibility seam only; delivery/executor/DecisionCore/GNOME unchanged.

Native typography first4-case run3PASS/1FAIL: first Textarea hint absent due native_deadline80204us with0geometry samples (before font read). Its complete row-3 trace is retained. Targeted first-Textarea recheck1/1PASS WITHOUT SOURCE CHANGE. All four positions have native font metadata +Tab and screenshots across these runs, but do not claim intermittent failure fixed or new4/4 clean run. Current first failure map: hint_geometry.rs -> kde/desktop.rs::observe_focused_window_until before read_ready_geometry. Origin unassigned; next task is read-only contention/request timing map, no timeout increase or generic routing edit. Keep this visible in report.
Pixel second-word hint and accepted afterTab actual field ink bounds exactly[300,371,330,380]; font/baseline/gap fixed in measured case. Local CACHE/firefox-natural-suffix-after-second.png and firefox-natural-suffix-accepted-second.png; ink-comparison.json. Successful hint snapshot readiness22-65ms after last char release (not paint), Tab actual DOM readback16.9-20.4ms including observer polling; N4, no general performance acceptance.

Receipts: CROOT/kde-firefox-natural-suffix-green-v1/result.json, kde-firefox-natural-suffix-build-v1/result.json, kde-firefox-natural-suffix-deployment-v2/result.json, kde-firefox-natural-suffix-native-v1/firefox/result.json, kde-firefox-natural-suffix-textarea-recheck-v1/firefox/result.json. Deploymentv1 stopped preflight on unfinished prefix with no mutation; one ordinary native Space preserved exact при ->при plus Space in our identified recovery fixture (kde-firefox-natural-suffix-input-finish-v1/result.json), then deploymentv2 proceeded idle.

Latest FULL historical browser matrix unchanged: Firefox159PASS/24FAIL/37BLOCKED/220, Chrome179PASS/0FAIL/41BLOCKED/220, total338/24/78/440. New native/Qt/source probes are not a full matrix and must not overwrite it. No0FAIL whole-Lay claim.

# 2026-10-29T21:33:59Z — terminal Double Shift fix installed and native proved

Installed/native milestone — 2026-10-29T21:33:59Z:
IME2098587 SHA447d62583383da8be7d127ad9e67794239cf4c7e58f1ff6be372104e4e73979c; source2e47eb13916bde36aa92df9681f40a18e618ae63a0981c99fe837ef521197261, zero-warning release74.683s. Only idle IME factory replaced; daemon1722873/35527e63, tray795755/c304, IBus3168326 and original Firefox988548 unchanged. First intentionally-mismatched Konsole Space case5/5 flips PASS with complete trace. Four-case Konsole/Kitty active+Space:4/4PASS,20/20 actual raw-PTY flips, next-letter f, KDE physical group0 and IBusUS all verified; original mode/layout/focus/config/runtime restored. Receipts CROOT/kde-terminal-native-mode-first-native-v1/result.json and kde-terminal-native-mode-four-native-v1/result.json. Four-case internal trace anchor lost to bounded rotation (trace_complete=false), so complete internal callback lineage is not claimed; per-case PTY and keyboard receipts retained. Median19.628ms, p95 nearest-rank25.441ms, max26.445ms from second Shift press to PTY receipt, includes12ms hold, with189-202ms inter-gesture cadence; not release-to-render or unrestricted rapid performance.
Current accepted terminal rendering unchanged. Firefox typography remains independent. Browser full historical counts unchanged159/24/37 and179/0/41.

SOLO/no Harness/agents. Preserve held fast-daemon leaf source: it remains uninstalled. Next: Firefox-only hint typography; read actual presentation metrics before edits. Local GNOME untouched.

# CURRENT — User accepts Firefox function; terminal source desync reproduced and current state repaired

SOLO/no agents/Harness. Local GNOME untouched. Running remote KDE remains daemon1722873/SHA35527e63, IME1852753/SHA6688be16 (sourcef821a88d), tray795755/scriptc3043b69, IBus3168326, originalFirefox988548. NO fast daemon or style draft installed.

User latest: terminal hint presentation looks OK in BOTH consoles; terminal Double Shift fails. Firefox hint appears separated by a space, smaller and above body text; removing a border alone insufficient. Hold all-KDE style changes. Exact original trayc304 restored locally. Remote WORKSPACE still uninstalled glyph-only draft5450833c until next explicit source overlay; installed script ALWAYS c304. Glyph mask source test3/3+4shapes+graph PASS but draft WITHDRAWN, never loaded. Pacing candidate source25/25+340/340+graph PASS but HELD UNINSTALLED after user accepts current Firefox function. Preserve candidate and rejection history; do not activate.

New measured terminal failure: entering KDE RU(index1) with IBus lay-ime-us, unchanged Konsole actor with mode-preparation disabled fails FIRST Double Shift both space and active words (0flips; привет stays привет). Exact failing trace complete; CROOT/kde-terminal-source-mode-diagnostic-20261029-v1/result.json. Original default actor hid this by forcing RU in setup. State was aligned through existing ibus engine lay-ime-ru while OwnsActiveText=false, no restart/config/binary change. Receipt CROOT/kde-terminal-align-existing-source-v1/result.json. Keyboard SwitchMode=Global, LayoutList=us,ru; do NOT assert per-window cause.

Then source-mode-preserving, separately selected cases in both default-profile Konsole/Kitty:4/4PASS,20/20flips plus next-letter controls; each source restored to coherent RU. Receipt CROOT/kde-terminal-source-mode-aligned-proof-20261029-v1/result.json, actorSHA57693710 (optional --only and preserve-source-mode guard; original case asserts/default sequence retained). Native trace complete; runtime/config/focus restored. Runtime code NOT fixed for recurrence yet. Need locate first transition that desynchronizes KDE/IBus and fix only its KDE adapter; no parallel owner or global Inline switch.

Presentation map: GNOME gnome/display.rs Inline; KDE kde/display/common.rs Auxiliary. preedit.rs:642 selects, auxiliary sends InlineHintV1 via hint_view.rs to KDE QLabel. Renderer receive picks system QFont/FixedFont and height*0.78, independently of client font; this explains unsupported font/baseline fidelity, not proof of the one-space x-gap. Terminal accepted rendering preserved; Firefox-only geometry/font next.

Full browser table unchanged Firefox159P24F37B+previousChrome179P0F41B,total338P24F78B/440. User manually accepts Firefox functionality; do not re-test it for publication or cosmetic-only work. Next source preflight uses kde-hint-glyph-only-source-v1-pinned.json + glyph-only-source-v1-result.json (remote5450833c), overlay restores old script and updates docs/graph.

--- Previous checkpoints follow ---

# CURRENT — Narrow IME installed; native short2 PASS; retained22 running

SOLO/no agents/Harness. Sourcef821a88d;20/20focused+1143/1143IME+architecture+exactregistry+2expectedPASS,32frozen unchanged. Zero-warning build74.601s. Installed IME1852753/SHA6688be16b33ec9a744a8313b8927805d85ebe182ad66cd4c302618895f387c84. Daemon1722873/SHA35527e63, tray795755, IBus3168326, originalFirefox988548/config6e62 preserved. Deployment CROOT/kde-firefox-native-erased-boundary-deployment-v1/result.json. Native short2PASS21.619s; retained22currently session56338. Table unchanged Firefox159P24F37B,previousChrome179P0F41B/220. Next finish22, measure canonical4row rapid prefix before reconsidering rejected fast daemon; old f99666a5 is still rejected, NOT installed. Next source preflight baseline kde-firefox-native-erased-boundary-build-preflight-pinned.json + native-erased-boundary-green-v1-result.json. Remote docs currently build-preflight version; newer runtime notes local only. GNOME untouched.

--- Previous checkpoint follows ---

# CURRENT — Native erased-boundary RED proved; GREEN pending

SOLO/no agents/Harness. Current daemon1722873/SHA35527e63; IME795642/689fa38c, tray795755, IBus3168326, originalFirefox988548/config6e62 unchanged. Last restore CROOT/kde-firefox-acknowledged-closed-diagnostic-rollback-v3/result.json. Exact wire-v3 captured and offline one-way-aware decoder accounts for every Engine call; packet72313 shows closing NBSP after native g72310 and Reset72312. REDv2 reproduced first failed transition19/20; v1 fixture setup failure retained. Local narrow KDE leaf implemented, not installed. Owner rapid-latency-map; new ADR2026-10-29-kde-firefox-native-erased-boundary-echo. Runtime3paths only: KDE preedit_echo, two unchanged predicate visibilities in tail_memory, existing observation seams. Frozen32 others. Local pacing restored originala3815fec; candidate f99666a5 remains rejected. Next short GREEN then full IME/graph/build/native controls. Matrix unchanged Firefox159P24F37B, previousChrome179P0F41B,total338P24F78B/440.

--- Previous checkpoint follows ---

# CURRENT — Closed fast pacing rejected; baseline restored; inspect delayed snapshot

SOLO/no agents/Harness. Canonical lay-dependency-updates-20261027, branch
codex/browser-matrix-repair-20261027, base b835ca47. Owner
kde-firefox-rapid-latency-map-2026-10-29.md. Latest full table unchanged:
Firefox159P24F37B/220,previousChrome179P0F41B/220,total338P24F78B/440.

Candidate f99666a5/sourcec749b933 passed25/25focused+340/340daemon+architecture,
zero-warning build67.347s and35 frozen integration files, BUT native short2
was1PASS/1FAIL20.615s: plain postTab8PASS,iframe-rich retained ghbdtn instead
of required привет. REJECTED; no full220 or performance acceptance.
Original35527e63 restored, current daemonPID1707260/native inputready;
IME795642/689fa38c,tray795755/c304,IBus3168326,userFirefox988548/config6e62
unchanged. Current restore receipt CROOT/kde-firefox-acknowledged-closed-
diagnostic-rollback-v2/result.json. No job running;83802 completed.

Local pacing leaf restored EXACT originala3815fec; rejected source saved privately
CACHE/firefox-replay-pacing-acknowledged-closed-rejected.rs. REMOTE source still
candidate c749b933 until explicit restore/next pinned overlay. Do not confuse
local source,remote source and installed35527e63. Next source preflight must
start from kde-firefox-acknowledged-closed-build-preflight-pinned.json plus
acknowledged-closed-green-v1/result.json; remote docs overlay from build was
applied. A source-only graph restore is still required after local leaf restore.

Short2 row anchors rotated/incomplete. Saved trailing slice shows a real
surrounding_receipt_mismatch after native first g: expected1-char suffix but
actual7-char prior prefix; later native processed=true yet next z start cannot
capture predecessor, followed by context_authority refusal. Hypothesis is
trailing Space/NBSP representation of the temporarily erased word boundary,
NOT proved raw text yet. Relevant source tail_memory.rs::
exact_replay_contains_prior_snapshot and observation.rs::
observe_word_start/context_reset_rereceipt processing. Do NOT loosen authority.
Need exact Engine.SetSurroundingText packet and key order before any related fix.

Bounded diagnostic reinstalled rejected f996 only for passive capture, with
finally rollback; it stopped BEFORE NATIVE INPUT because busctl list on IBus
private bus timed out6s. GUI monitor selfcheck paired;engine capture absent.
CROOT/kde-firefox-acknowledged-closed-wire-v2/result.json retained. Original
bytes restored automatically (1707260). Next validate IBus monitor READ-ONLY
on baseline first; use Engine method-call + all-return filters, not busctl
process listing (unsupported on this private bus). Existing passive recorder
CACHE/kde-firefox-space-readback-wire-v1.py is parameterized; reuse it. No more
candidate install until monitor preflight succeeds. Candidate source/proof and
prior failures remain available; do not reapply fast pacing for acceptance.

Guard profile dedicated20cpu; explicit CARGO_TARGET_DIR=/home/e/projects/
lay-td119-gate-v1/target. Guarded python3 - does NOT receive heredoc stdin;
that attempt exited0 without executing script or touching runtime. Use existing
script file or correctly shell-quoted python3 -c; zero exit alone is no proof.
Native actors and production preload unchanged; all local GNOME untouched.
No publication. Continue current24rapid-afterSpace group; cold fresh single
row context_authority refusal is separately retained, not claimed fixed.

--- Previous checkpoint; latest state above supersedes it ---

# CURRENT — Offline diagnostics delivered; Firefox backlog unchanged

2026-10-29 18:34 UTC. User requested a mechanism/tool and instructions to reduce
development loops and locate failures accurately. Implemented
`scripts/lay-diagnose.py` with independent receipt and paired-wire analyzers;
documented the single-hypothesis loop in AGENTS.md, DEVELOPMENT.md and
`docs/development/native-diagnostics-workflow.md`. SOLO; no agents or Harness.

Guarded remote verification: 22/22 tooling tests, real saved matrix/wire
integration and architecture graph/canon PASS. Report analysis 17.63 ms; wire
comparison 5.18 ms; commands about 64 ms. No total-debugging-speedup claim.
Final receipt: CACHE/native-diagnostics-tooling-final-v2/result.json. Graph
return accepted by hashes. All 32 existing integration files frozen unchanged.
No runtime source, installed binaries, GNOME settings, services or native input
were changed by this tooling task. No new browser acceptance or publication.

Source overlay now includes tooling and regenerated graph metadata. Next remote
source preflight must use `native-diagnostics-tooling-final-v2-pinned.json` and
`native-diagnostics-tooling-final-v2/result.json`, NOT the older b616 baseline.
Remote source closure: e94eaf182e3fd70a8136da5efcee40a1fa9b9d1a6fde2655d3827568309263bf,
1314 files. This is source identity, not an installed runtime binary/version.
Installed daemon 35527e63 / IME 689fa38c remain the prior accepted test setup.

Latest native matrix remains Firefox 159 PASS / 24 FAIL / 37 BLOCKED / 220;
Chrome 179 PASS / 0 FAIL / 41 BLOCKED / 220 (previous full result);
total 338 PASS / 24 FAIL / 78 BLOCKED / 440. All 24 failures are rapid-five after
Space: 23 late-target assertions and one missing-target assertion. Analyze the
saved report before further input; only 9/24 failed row anchors are intact and
global causal capture is incomplete. Do not infer the first broken transition
from counts. GNOME hint styling remains a separate open task; do not edit it
while continuing the scoped KDE Firefox work. No source/build/native job pending.

--- Previous checkpoint; tooling delivery and source baseline above supersede it ---

# KDE Firefox full220 completed159P24F37B

Same installed daemon1245755/35527e63, sourceb616e1a3; IME795642/689f,
tray795755/c304, IBus3168326,userFirefox988548/config6e62 preserved.
No new runtime/source changes while full test ran. SOLO/noHarness/agents;
no localGNOME edit/install/restart, no publication.

Latest full native table: Firefox159PASS/24FAIL/37BLOCKED/220;
Chrome179PASS/0FAIL/41BLOCKED/220(previous full); total338PASS/24FAIL/78BLOCKED/440.
No NOT_TESTED. Frozen220 manifest unique and normal/private/fullscreen complete;
production-owned notify capability observed; guard20CPU/24Ghigh/28Gmax/1Gswap/
512tasks verified. Full elapsed431.581s. Runtime/config restored; no heldkeys.
Full controller/actor exit1 because actual24FAIL. Original160108P24F28B;
additional60consecutive51P0F9B. Full aggregateIMEtrace incomplete;9/24failed
row trace anchors complete, not alone a causal proof. Do not claim all evidence
complete. Historical full126P55F39B retained; environment now aligned with
production launcher, so do not attribute all31 fewer FAIL to last patch alone.

All24FAIL are rapid-five AFTER SPACE in8supported fields x3window modes:
23latency assertion failures(all expected targets but after nextgesture start),
1fullscreen/search missing expected target. Active-word rapid-five passes.
BLOCKED:10inverse(sharedlearning TD137),24URL/email trailSpace sanitization,
3email native/DOM disagreement. Do not waive/convert these to PASS.

Current Space/NBSP readback fix has focused causal and native proof; remaining
closed-word rapid performance is a distinct unresolved group. Read-only source
map shows firefox_replay_pacing.rs only chooses existing fast profile for open
word; closed words still use original paced emitter. No new pacing change has
been made. Do NOT simply reapply rejectedfastclosed experiments; need new causal
receipt and owner/scope map first. Next inspect current full complete traces or
one unchanged-actor paired wire probe; preserve cadence120ms between gestures.
Do not edit GNOME renderer under this task. User current question about where
work is slow/loops has been answered; no aggregate work-time accounting exists.

Local proof: CACHE/space-readback-full-v1-result.json and -summary.json;
remote CROOT/kde-firefox-space-readback-native-full-v1-verification.json.
Previous focusedcontrols21P0F1B+Tab16/16+heldShiftTab16/16 and source24+340PASS
remain valid. Source baseline pin kde-firefox-space-readback-build-preflight-
pinned.json / space-readback-green-v2-result.json. Fulljob55351 completed; no
source/build/native job pending. GNOME styling read-only map below, cause still
unproved; defer changes until discussed after current Firefox work.

--- Previous current checkpoint; latest facts above supersede it ---

# CURRENT — KDE Firefox Space/NBSP fix installed; focused controls verified

Canonical /home/ubu/projects/lay-dependency-updates-20261027, branch
codex/browser-matrix-repair-20261027, baseb835ca47. SOLO/noagents/Harness.
User latest question: where the work is hard/slow and repeats; answered with
async client/daemon/IBus confirmation ordering and avoidable test-environment
mismatch, no invented time percentages. Continue existing task. User says finish
current KDE Firefox first, then inspect common hint styling. GNOME Chrome
indistinguishable hint report OPEN; no local GNOME edit/install/restart/rollback.
No publication. Latest full Firefox126P55F39B, Chrome179P0F41B, total305P55F80B/440.

Remote CURRENT daemon1245755/SHA35527e63ebff48a4cc6711a71e62999e737298c252583990f1a308a388988cec,
sourceb616e1a3aa3097fb435636184866e49b01c11d1d0ca95e23a616cae67a93ffde,
1309files. IME795642/SHA689fa38c/source703848cb; KDEtray795755/scriptc3043b69;
IBus3168326,userFirefox988548/config6e62 unchanged. LAY_DEBUG_LOG absent.
Native inputready, installed/proc/cache daemon bytes verified, idle daemon-only
replacement; no IME/tray/globalIBus/browser replacement.

ROOT proved paired wire: whole target and retained prefix are correct, metadata
source/path/field/advanced epoch/mode all match, but Firefox rich closing Space
reads back NBSP160 instead of emittedASCII32. Literal settlement wasted80ms.
Six exact source seams and callback policy described in active ADR/owner
2026-10-29-kde-firefox-acknowledged-space-readback. Only same-count closing
ASCII/NBSP equivalence for KDEFirefox; every prior byte and full envelope strict;
all other clients literal. Normal/queued consumers use one pure client policy.
No output/pacing/owner/verifier/layout/hint/input changes in this phase.

RED23/24 intended. GREENv1 24/24+340/340 never built (queued consistency caught).
Final GREENv2 24/24focused+340/340daemon, graph53.623PASS, exactregistry delta0,
release0warning67.212s. Return accepted. Next source baseline pin
kde-firefox-space-readback-build-preflight-pinned.json and result
space-readback-green-v2-result.json. No Rust worker running.

Native causal3PASS/0FAIL/0BLOCKED:12flips,146paired events/0unpaired versus710old;
V4reads184->48, processing159->13(includes1selfcheck), no refusal/pending events.
Rich10 ManualToggle call-to-last-V4 median266.027->182.166ms,
p95342.910->205.853ms. This is not hardware-release/UI latency. Separate4RFBsend
DOM samples167.38..189.94ms; postTab8UI sample coverage missing. Old wire proof
had selector-order assertion error after3nativePASS, retained, not recastPASS.
Receipts space-readback-wire-comparison-v1.json / wire-v1-result.json.

Same final bytes controls finished: ordinary22=21P0F1B78.806s, lowercaseTab16/16
21.802s, heldShift/title/uppercaseTab16/16 23.308s. All config/runtime preserved,
heldkeysnone. Ordinarycontroller exit1 because existing email native/DOM
observation disagreement BLOCKED; no failing case. Do not relabel it PASS.
CROOT receipt prefixes kde-firefox-space-readback-native-retained22-v1,
-tab-native-v1, -title-tab-native-v2 verification.json.

RUNNING: full220 Firefox normal/private/fullscreen on same final daemon using
unchanged r19 actor/hash360e6b9a. Prepared CACHE controllers
kde-firefox-space-readback-full-controller-v1.py and -full-v1.py;
production-owned child launch aligns unchanged924b5982 notify preload + GTKibus
and a11y, actual capability bit must be observed. Existing userFirefox maps
that productionlib; early frozen-actor baseline lacked it (caps41), retain old
FAILs. No host/global env edits. Launched native full via guard, session55351; remote
controller1294861 in lay-verify-1000-1294833.scope, kernel cpu.max2000000/100000,
memory.high24G,max28G,swap1G,pids512 verified. Current full matrix is historical
until verified complete. Do not reapply rejected waiting-only80ms or fastclosed
variants; no literal/runtime exceptions, newroutes/owners/queues/timers.

CROOT=/home/e/projects/lay-development-runner/kde-install-control-20261020.
CACHE=/home/ubu/.cache/lay/development/browser-matrix-repair-20261027.

--- Earlier chronological evidence below; CURRENT above supersedes state ---

# CURRENT 2026-10-29 — KDE Firefox launch-alignment diagnostic

Canonical checkout /home/ubu/projects/lay-dependency-updates-20261027,
branch codex/browser-matrix-repair-20261027, base b835ca47. SOLO/no agents/Harness.
No local GNOME edits/install/restart, no publication. User latest: finish current
KDE Firefox fix, then discuss common styling. GNOME Chrome indistinguishable
hint report OPEN, no rollback requested. Historical full matrix remains
Firefox126P55F39B, Chrome179P0F41B, total305P55F80B/440. Native diagnostics separate.

Current remote daemon1015506/SHA271d84aa0058196406e3225e1d04485c132c7d243f6b86952c47f61fa7d1b574,
source e14b4344bc8c1cf6133d265630abe46291f6d3987db92ff6d01119c77669624c.
IME795642/SHA689fa38c70359430e0ebf612b5e3c039a2d7f8a5b78855c6805ea9cc89372dfe,
source703848cb. KDEtray795755/scriptc3043b69, IBus3168326,userFirefox988548/config6e62 unchanged.
New source only KDEfirefox_replay_receipt.rs and exact_ime_tail_replay.rs call seam:
reuse already verified readonly Firefox policy, same current focus/ACK/field checks.
RED19/20 intended; GREEN20/20+336/336daemon, graph PASS,0warning67.09s build.
Installed idle daemon-only, inputready. No change to output/pacing/display/route.
Owner/ADR: kde-firefox-operation-client-binding-map-2026-10-29.md / same-date ADR.

Native no-adapter prefix3=1P2F16.168s: active4delegated replays/8positive ACKs,
context_authority refusal event1456, exact actual SSI confirmed1458. No suppression
cancellation. AfterSpace all5states but late. The source patch does NOT close
this native symptom; retain it as partial evidence, no acceptance/push.
Existing same-gesture AwaitingExactSnapshot is relevant; stockcaps41 and partial
prior surfaces currently refuse refresh. DO NOT reapply rejected post-output
waiting-only approach (Oct28:all reads unknown until80ms deadline).

New setup discrepancy independently found: userFirefox988548 maps existing
924b5982 notify library via LD_PRELOAD; daemon env has no preload and native actor
launches ELF via geckodriver inheriting that absent setting. Native trace allcaps41,
exact_refresh=false. Production KDE launcher explicitly uses unchanged library
+GTKibus+a11y. One comparison aligns ONLY owned browser/driver child environment,
immutable actor e698, original cadence/assertions and3cases. Require observed
private capability bit before accepting launch effect. Controller
kde-firefox-production-launch-prefix-controller-v1.py; wrapper
kde-firefox-production-launch-prefix-v1.py, remote phase
kde-firefox-production-launch-native-prefix-v1. Probe completed2P1F14.756s: active rapid5PASS, afterSpaceall5states but lateFAIL. Actual capability flag observed by controller (asserted); fetch detailed result/trace before causal claims. Next same-byte ordinary22/Tab/held-Shift controls using production-aligned child env. Source/runtime/user browser/
local GNOME unchanged. If effective, compare causal actual receipts and targets;
retain no-adapter failures explicitly instead of relabeling them green.

All paths CROOT=/home/e/projects/lay-development-runner/kde-install-control-20261020,
CACHE=/home/ubu/.cache/lay/development/browser-matrix-repair-20261027.
Latest source pin kde-firefox-client-binding-build-preflight-pinned.json,
source result client-binding-green-v1-result.json for future exact overlay.

--- Earlier chronological evidence follows; later CURRENT entry supersedes state ---

# Lay — KDE Firefox operation client binding RED underway

ACTIVE ADR2026-10-29-kde-firefox-operation-client-binding, owner
kde-firefox-operation-client-binding-map-2026-10-29.md. Canonical checkout
/home/ubu/projects/lay-dependency-updates-20261027, branch
codex/browser-matrix-repair-20261027, baseb835ca47. SOLO/noHarness, no local GNOME,
no publication. Continue until full FAIL=0, no cadence/denominator reduction.
User reports GNOME Chrome hint letters look ordinary; finish current KDE Firefox
fix first, then map common styling. NO rollback/restart/change to local GNOME now.

CURRENT daemon863172/SHA0a71fd83ad69a9b75534e0215aba9626cda3f955f90caba684b8041326037bd7
sourcec14dc808f6977c9b1a749a5b1d6353f0f5cc468f1b2bdf78e996bd4157476a08.
IME795642/SHA689fa38c70359430e0ebf612b5e3c039a2d7f8a5b78855c6805ea9cc89372dfe,
IBus3168326,tray795755(new Python scriptc3043b69),userFirefox988548/config unchanged. Raw-word-only pacing
source334/334 daemon+28KDE/28GNOME,0warning67s build. HISTORICAL pre-queued IME
ordinary22 was20P1F1B; CURRENT combination21P0F1B, NOT full acceptance. Failing iframe-rich row20 is Tab
BEFORE any Double Shift: two owned CommitText appends occur before an earlier
caret SSI. Source cache tracks only one append, delayed SSI destroys proof,
later exact receipt cannot recover. Complete causal row20 has no malformed
lines. Unrelated row22 first line malformed; verification wrapper error retained.
All source returns through raw-word-pacing GREEN accepted; no job pending.

QUEUED APPEND installed IME698174/fbf3808d,source d8a90c49 after1140/1140,
18/18short,0warning74.497sbuild. Native16Tab=16P0F0B including all4Textarea;
ordinary22=21P0F1B/74.449s,Email native/DOM disagreement. Originalfailed iframe
postTab8 nowPASS. Partial traces1/12/15/20 refused for causal claims; no assertion
that new queued-echo branch was observed physically. Full220 not yet rerun.

HINT LOOK actual2frames: DOMpri,projectionvet,screenshot merged black-on-white.
Running tray source3f936370 border:none confirmed. New source common read-only
frame optional outlined boolean, true ONLY existing authenticated KDE Firefox
geometry; KDE renderer1pxborder only on true, resets each frame. Other clients
and GNOME defaultfalse; no detector/RPC/input/authority/output change.
RED malformed optional flag expected failure;SHORT28/28Rust +Python3/3 +unchanged
GNOME Node fixture testsPASS,source97671c03. Outline INSTALLED after0warning74.544sbuild, idle IME+KDEtray activation.
Source703848cb,IME795642/689fa38c,tray795755/scriptc3043b69.
After two actual frames: DOMpri unchanged, ONLY suffixvet outlined; look2/2PASS.
After runtime native32Tab(16lower+16heldShift)=32P0F0B; ordinary22=21P0F1B
74.399s, same Email observer disagreement. Native runtime/config protected.
Full1141/1141IME +28/28short,Python/GNOMEprojection/graph/registryPASS;
source703848cb813f677ebe5027ac418bb939c4c5e0e4a75006ead533c110408ce3a2.
NEXT rapid prefix3 diagnostics before full220: unchanged actor e6985..,
roundtrip-active,rapid-active,rapid-afterSpace, existing exact cadence.
Prefix phase hint-outline-native-prefix-v1 COMPLETE1P2F0B. Active fifth refused
context_authority; fourth suppression cancelled before actual full SSI. Both
row2/3 complete+clean. Need existing daemon warning to distinguish classifier
unavailable from readiness/path error. Existing-log diagnostic COMPLETE2P1F: active rapid PASS, afterSpace lateFAIL.
Same0a71 bytes, original unitEnvironment hash/config restored, created dropin
removed. FINAL daemon863172/0a71; other processes unchanged. Journal10 entries
only complete/pending-processed, no missing-active failure reproduced. Do not
assume pending log means80ms timeout: common V4 unavailable can return early.
Passive D-Bus observer v1/v2 setup failed BEFORE any native input, preserved
runtime. PCAP v3 paired wire selfcheck before native3, now exec session19120
running kde-firefox-passive-replay-bus-probe-v3.py. No source/build changes or
further service restart. Need result + events-summary.jsonl after completion.

Title-case raw V1=0P16F INVALID INPUT PRECONDITION: no actual Shift held and
prefix already lowercase before Tab. Raw preserved; not a Tab verdict. Corrected
V2 explicitly holds Shift and verifies actual prefix before Tab, pinnedactor
cffb780c10f9c7effffaea713ee7dabee7bb0cc8c64c9bc1bc1407caaade3bea; native16/16PASS,24.023s.
No runtime Shift/case change made. Source returns through hint-outline-short-v1
accepted. Source full hint-outline-green-v1 complete and all returned files accepted.
Style build/install/native complete. No source/build jobs pending.
Source control files untouched after source returns accepted; no agents/Harness.

USER latest: Tab fails Textarea6; earlier tel4/plain8 first-word reports still
open. Second-wordTab8 confirmed. Artifacts disappeared (not proved fixed), hint
looks ordinary text. Handle appearance separately ONLY KDE/Firefox after Tab
root; typed letters ordinary, distinguish suffix. Passive60s watch143samples,
2frame changes,0visiblehints; it did not reproduce or identify artifact cause.
Original Firefox/text untouched. No questions pending; continue autonomously.

LATEST FULL Firefox126P55F39B/220 (previous daemon4403); Chrome earlier179P0F41B/220;
total305P55F80B/440,0NOT_TESTED. Full Firefox55F:50rapid-five +5moved-boundary;
extra consecutive-two54P0F6B. Narrow22/Tab/source results do not replace totals.

REJECTED pacing experiments preserved:390047/5df1 source d8bc prefix126msmedian
vs180baseline but ordinary22=19P2F1B;462653/3295 source7b43 ordinary22=18P3F1B.
Second guard erroneously excluded ASCII Space but not NBSP; final raw equality
policy fixes phase distinction and all19sourcecontrols pass. Do not reapply old
candidates or attribute current Tab failure before DS to pacing without evidence.

## Source progress history (current runtime and native status above)
Work solo, no Harness/agents, no local GNOME action or publication.

First measured plain Tab failure: sixth managed CommitText succeeds -> actual
6chars/caret5 SSI -> receipt rejection -> release/Reset -> actual6/6 SSI.
This loses the first-word start before the second word/Tab. Complete row15 in
kde-firefox-empty-preedit-native-retained22-v1. Separate URL first-toggle FAIL
is not proved to share it. Retained22=19PASS2FAIL1BLOCKED. Full440 historical
unchanged306PASS55FAIL79BLOCKED; do not recount from narrow/source results.

Genuine new RED-v2 selects9/8, failing inert caret-echo retention. RED-v1 compile
setup error is not causal proof. SHORT-v1..v3 found current snapshot invalidated
by owned CommitText, and normal keys did not capture the existing boundary
candidate. C04 invalidation remains: one real full prior field is inert data
inside the existing candidate/pending receipt, spent at first SSI. Same original
revoked predecessor/epoch/revision and successful single-character append must
match. Existing strict-prefix slot retains progress; no new owner/type/timer/RPC.
KDE leaf owns exact presentation predicates; actual pure SSI selector is tested.
SHORT-v4 reached retention; two new assertions confused read-only computation
with edit grants. Existing computation function preserved; tests now assert
actual grant alternatives/zero output. No old test assertion changed.
SHORT-v5 now11/11PASS:14 real refusal variants,6 Unicode prefix cases and
Reset/no-Reset positives. Source1307files SHA190aa8a3b797b6bb9137518f90059ff5d2923fa9a12bd55a773d2e551debefef.

GREEN-v1 full1134/1134 IME correctness/package PASS, graph/manifest PASS;
3 new tests, all old rows unchanged; short11 overlaps. Source1307files
SHA122f3cf1ff0ac8f277bf0c3f1b470ae899c11f680a0c25d2c52d3743b5a519ce.
NEXT0warning IME-only build, matching
idle component activation, native causal trace/short and retained22 before full
Firefox220. Prepared workers kde-firefox-owned-append-caret-{build,install,native}-v1.py
pin the CURRENT installed diagnostic IME3669264/0a26d84e418a495b1fa13d91ed6a1745cf3a453cd3e0d17b85641bed8ff374f4,
from earlier rejected source5df213...; current source candidate is NOT installed.
Daemon2374910/ae77e705..., IBus3168326, tray3228081, actual userFirefox988548 and
exact config6e62eab3... protected. Native source-free/profile claims require real
receipts. Optional user check requested after repeated refusal; continue
independent work without treating elapsed time/reply as permission.

Prior empty-return feature NOT PROMOTED: source1134/native short2 PASS, but new
branch execution not established by retained traces, broader22 failed earlier. Only its local source/test/
manifest overlay withdrawn by frozen hashes; accepted daemon focus fix and all
older dirty work preserved. Audit/cache: empty-preedit-experiment-withdrawal-v1.json,
rejected-empty-preedit-production-overlay-v1.tar.gz. Current candidate extends
only the measured owned-append mechanism; do not reintroduce that rejected
feature without evidence for its missing native premise.

## Superseded empty-preedit experiment progress

Current bounded owner: `kde-firefox-empty-preedit-return-map-2026-10-29.md`.
Unchanged installed runtime retained22 now20PASS/1FAIL/1BLOCKED; both old failures
passed. Remaining plain post-Tab reverse fails after successful native target,
one-character decoder probe and handled Backspace: stale15/14 SSI discards the
existing confirmed closed-word receipt before exact14/14 SSI. Failed row trace
is complete; whole-run trace rotated. Runtime/config/IBus/tray/user Firefox
preserved. Genuine source RED9selected8PASS, only the new final-field assertion
fails. SHORT-v1 compile found two included native-cursor constructors requiring
mechanical None initialization, now declared/repaired. SHORT-v2 compiled10/11:
positive regression green, negative caught Reset inheriting the new inert echo.
Added refusal of only that unconfirmed echo as a Reset candidate. SHORT-v3 now
11/11PASS;18 negative variants ×2 separators, direct-exact KDE/GNOME controls.
GREEN-v1 full1134/1134 IME correctness/package PASS, short11 overlaps;
graph/architecture PASS, exactly3 new manifest rows and no old changes.
Source1307files SHA5df213b877c535632454b4760a6af4d3a21013b72df44036b6f9c2bb0a61f4cd.
IME0warning74.374s build and guarded idle component installation completed:
engine3669264 SHA0a26d84e418a495b1fa13d91ed6a1745cf3a453cd3e0d17b85641bed8ff374f4.
Daemon2374910/ae77..., IBus3168326/tray3228081/userFirefox988548 and exact config
preserved. Native plain/rich post-Tab eight run is active; retained22 follows
only on PASS. These are not yet native acceptance. GNOME, prior daemon focus
fix, admission/executor and all prior work remain protected.

## Completed preceding KDE X11 freeze-focus fix

Current owner:`kde-x11-grab-focus-map-2026-10-29.md`; final structured report:
`kde-x11-grab-focus-2026-10-29-verification.json`. One bounded case completed.
Work solo; no Harness/agents, no publication. Do not resume the old unbuilt C
idle-refresh proposal or restore rejected daemon experiments.

Loaded remote KDE:71 daemon2374910 SHA
ae77e705ba6795cfb99f1b074b027a70e2f7277c81f480380d969880fd07d644.
Source1307files SHA50938ebe30ad78da2d90389893b75d35273a80017e799fb018c949a02ec10ebf,
canonicalbaseb835ca478198d6cb765a2246329d5646966c118c, branch unchanged.
Base version1.0.111/unpublished candidate;0warning67.005s release. Only daemon
installed/restarted. IME2067215/bad527..., IBus3168326, tray3228081 and actual
Firefox988548 protected; config6e62eab3... unchanged, KDE/IBus bothRU.

Cause proved: original master grab emitted core FocusOut, losing the exact
word/context before output while the KWin window stayed the same. Existing
freeze test checked XI2 only. Genuine RED-v3 precedes fix (two worker setup
errors are not evidence). New `kde/native_input_focus.rs` retains same-client
focus using the existing owned replay master before KDE freeze. Output map
readiness remains unpublished until actual isolation/boundary checks. Prepared
failure cleanup uses existing Input/ungrab owner. GNOME ordering, admission,
verifier, decisions and delivery contracts preserved; no local GNOME action.

Source330/330 daemon plus28/28 native KDE and28/28 native GNOME policy PASS;
two daemon perf tests not run. Native SAME user's Firefox process/profile,
owned window67109872, normal mode:10unique scenarios PASS/0FAIL/0BLOCKED,
62 checked target/next-decoder gestures,7 held/released Shift pairs,4 Lay Tabs.
Includes input/textarea/rich active-eight,Space-eight,Tab/two and the original
rich second-word-after-Tab/eight failure. Original first7P2preparationB retained;
only those2slots rerun after known empty-editor native terminator handling.
Raw observations remain; actual nonempty targets need no hidden stripping.

Historical full matrix306P55F79B/440 NOT rerun/recounted. Other apps/modes/HID
not certified by this focused normal-window proof. No broad acceptance claim.
Original user window67108885 preserved; additional owned fixture67109807 and
verified fixture67109872 still open. The90s passive prompt received no gesture;
it expired and restored config. Async old prompt is not current acceptance.
User may manually verify the installed candidate. No further source work is
needed for this bounded case; any later publication reuses accepted evidence
according to AGENTS, rather than reopening tests on an unchanged accepted build.

## Superseded source/build progress

Active owner:`kde-x11-grab-focus-map-2026-10-29.md`. Same user's Firefox988548
reproduces post-Space Double Shift FAIL: before native grab exact VisibleTailV4
has the correct word/context/epoch, after grab it is unknown-context with changed
epoch. KWin confirms the same real window. Active composition can toggle19/18ms,
Tab works in both input/textarea (~18ms) after KDE RU/IBus RU alignment.

Genuine owned-Xvfb RED-v3 proves core FocusOut at acquisition; two prior worker
setup errors are not evidence. New KDE-only leaf `kde/native_input_focus.rs`
reuses the existing same-client private-master focus before original-master
freeze. Output maps readiness stays unpublished until actual isolation/boundary
checks. Existing Input/ungrab owner cleans prepared-but-not-grabbed resources.
GNOME keeps its previous ordering; admission/verifier/delivery unchanged.

GREEN-v2:28/28 native KDE,28/28 native GNOME,330/330 daemon correctness/package
PASS. Three short checks overlap native totals. Two daemon perf tests not run.
Source1307files SHA50938ebe30ad78da2d90389893b75d35273a80017e799fb018c949a02ec10ebf.
Canonical base remainsb835ca478198d6cb765a2246329d5646966c118c with retained patch;
remote syntheticHEADa8fb... is not the canonical base. Scope/ADR predeclared.

Next: daemon-only0warning release, matching idle daemon installation and loaded
bytes, then same-process active/Space/Tab native controls. Current daemon1894209
15fcc650... and IME2067215 bad527... still installed. Preserve IBus3168326,
tray3228081, original Firefox988548/window67108885 and exact config6e62eab3...
Owned windows67109807/67109872 from current diagnostic may be cleaned only as
owned fixtures. No user gesture occurred during the90s passive capture; config
was restored. Async prompt unanswered, not acceptance/authorization.
Historical matrix306P55F79B/440 unchanged. No release/push/Harness/agents/GNOME
installation or service restart. Earlier mode mismatch alignment alone did not
repair committed-word replay; preserve both old FAIL receipts.

## Superseded live input diagnosis

Active owner: `kde-live-input-recovery-2026-10-29.md`. User reports Tab and
Double Shift unavailable across remote KDE. Work solo, no Harness/agents.
Do not resume the unbuilt C idle-refresh experiment: current real failure
has passed initial admission and reached exact replay delegation.

LIVE verified: IME2067215 SHAbad527220d1034e51444c567770511794f6b3c43c439599484698b84651d9bac;
daemon1894209 SHA15fcc650f2aad3f3ea30d4152011adcd07971ae500f1737db96b3732ae8b715e.
The erasure echo candidate IS installed (old pending-build text below is stale).
Its source1131/1131PASS, short8/8,0warning74.366s build; source1306files
SHA040fb2cfafa9dcdf44e217693f55bb36c602b97d2073ff9fbc73946ac51bd757.
Earlier owned-profile22rows19P2F1B and active-eight4rows/32gesturesPASS are
separate scopes, not certification of the existing user's FirefoxPID988548.

Found KDE Russian index1 with IBus lay-ime-us. With OwnsActiveText=false,
aligned ONLY IBus to lay-ime-ru. No restart, install or config/active-window
change. Alignment alone did NOT fix Double Shift. XInput2device5/master3 ready.

Fresh SAME FirefoxPID988548 owned-window67109872: Tab2/2PASS (~18ms), active
first Double Shift2/2FAIL, input and textarea, actual focus intact. Trace proves
ManualToggleV3status3, confirmed Reset receipt, then no replacement. Inspect
existing daemon exact-tail capture/execution, not the old bridge-admission case.
CROOT/kde-live-input-recovery-v1/native-same-session-v1/result.json.

Wire-only repeatsv2/v3 blocked BEFORE input by transient native focus/caret
precondition; not two extra product failures. Following user's call-if-stuck
instruction, armed one90s passive capture (`passive-user-gesture-v1`) and async
asked one Double Shift on owned textarea containing `проверка `. No native
injection/window switching during capture. Read reply/capture, restore debug
config automatically. Original user window67108885 preserved; owned windows
67109807(preparation title retry) and67109872 belong to this diagnostic.
Historical full matrix306P55F79B/440 unchanged. No GNOME/runtime-source change,
push or release. Preserve evidence; update owner after this bounded capture.

## Superseded erasure source-check progress

One active route: `kde-firefox-owned-erasure-echo-map-2026-10-29.md`.
User wants autonomous native testing and GNOME comparison; no Harness/agents.
Processed-result candidate is installed: IME1893290 SHA31a7b4be84ad7e8ff38730346f808dcdf3b29958552816a11bb2e2469fc09d2d,
daemon1894209 SHA15fcc650f2aad3f3ea30d4152011adcd07971ae500f1737db96b3732ae8b715e.
It passed source1457/1457,0warning dual build,4/4 native short; longer native
22 is19PASS/2FAIL/1BLOCKED. Ordinary active reverse fails after decoder/Backspace;
plain Tab fails separately before the first gesture. No full acceptance yet.

Current uninstalled source records one authenticated successful local owned
Backspace effect inside the existing start witness. Only KDE/Firefox can retain
one inert erased-character echo, then require the immediately following full
actual external snapshot before carrying the same witness. Manual held-key
barrier unchanged. New AT-SPI re-projection proposal was rejected BEFORE build
or install because a new native field alone cannot bind the old word identity;
its unbuilt experiment was removed and archived outside the checkout.

Erasure source RED-v1:7/8, exact final-confirmation failure. GREEN-v1 finds the
missing shared-owner negative; never installed. SHORT-v2:8/8 PASS with the active
path checked at preparation/settlement, all16 negative variants passing.
Source1306files SHAb1252c376227cda9d5a492e88011796c649fea0fc7705eecb440a4a215b8310e.
Full GREEN-v2:1131/1131 IME,8/8 short, graph/architecture PASS; only4 new tests,
all old rows retained,3 performance tests not executed. Source1306files SHA
040fb2cfafa9dcdf44e217693f55bb36c602b97d2073ff9fbc73946ac51bd757.
Next:0warning build, replace only idle IME, verify
loaded bytes, short native active roundtrip, then the retained22 rows/window
modes. Preserve IBus3168326/tray3228081/manualFirefox988548 and exact config.
Historical full matrix306PASS55FAIL79BLOCKED/440 is unchanged; do not recount it
from selected controls. No public release. Call user only if work stops yielding
new evidence/progress; otherwise continue autonomously.

## Superseded progress below

### KDE Firefox Double Shift, first fix installed

One active task: `kde-firefox-post-tab-double-shift-map-2026-10-29.md`.
User wants autonomous continuation and GNOME comparison. No Harness or agents.
First cause, two-separator reflow after Tab, is fixed in KDE/Firefox presenter
and the existing tail owner. Source21/21 targeted and1125/1125 IME PASS; release
0warnings,74.621s. Source1304files SHAe8de03875180fc7fa0117fdf48726d224293a049434870c6ba578e4e3ce86c9d.
Installed IME PID1670433 SHAe163afd765fb13d8abc8d4de2a83003a3b2b1c5a2ccb60d51f899086f4b47ccf.
Active scope ADR2026-10-29-kde-firefox-preedit-space-reflow. Preserve this fix.

Short native post-Tab4/4PASS. All3 previously failing rich fields pass in all3
modes. Same54 display rows47PASS/1FAIL/6BLOCKED;162 prefix points. Normal and
fullscreen16P0F2B each; private15P1F2B. Distinct private FAIL: ordinary text
reverse toggle after a successful first one. Eight-v1:22rows20P1F1B; rich third
gesture fails, text/plain/iframe-rich each pass8 toggles, per-step decoder and
final held/released Shift. Email native/DOM disagreement is BLOCKED. Raw receipts
are authoritative; modes-v1 outer summary stopped at its private assertion.

Next cause: post-output cleanup cancels replay provenance before final actual
client observation. Private trace events770(cancel),776(final exact snapshot),
814(context_authority refusal) prove this ordering. Old waiting-only daemon
experiment was rejected; do not reinstall it. A new forwarded-release C wrapper
was tested only in owned fixtures: original contracts2/2, variant1/1PASS, native
22 gives20P1F1B with a different reverse failure. REJECTED as an unproved overall
fix; production launcher/library/manual Firefox unchanged. Retain CROOT/
kde-firefox-forwarded-release-{build,native}-v1 evidence, not a green-only subset.

Second scoped implementation: `kde-firefox-native-result-map-2026-10-29.md`,
ADR/active scope2026-10-29-kde-firefox-processed-native-result. Text-free read-only
NativeReplayProcessedV1 distinguishes completed native presses from actual
client confirmation. Only KDE Firefox post-output cleanup can retain the existing
bounded result scope and release isolation while returning client-unverified.
No output retry, larger timeout, new owner, C03/field verifier or delivery change.
Source RED-v3 is genuine (earlier import/config fixture failures retained);
GREEN-v1:4/4 IME short,7/7 daemon short,1127/1127 IME and330/330 daemon PASS.
Five performance tests and26 ignored daemon tests not executed. Source1305files
SHA45efaccc7a36dea3b6845d92dbd69005f49f38cf9b66d9a4d3febecad6620030.
Canonical discovery adds only2 IME+1 daemon tests; existing rows preserved.
Graph/architecture PASS. Next: build both components, install idle IME first
then restart only the same daemon unit, verify exact bytes and native controls.
Source PASS is not native acceptance. Runtime below still first-fix e163/eb609.

Daemon374525/IBus3168326/tray3228081/manualFirefox988548 preserved. Config
6e62eab3c278c733fedb47e45ef6306105a3700222cf9f7a4e5bff4bd92264d1 restored.
No commit/push/release. Full historical matrix306P55F79B/440 unchanged. Call the
user if successive attempts stop producing new evidence/progress.

## Completed display fix and retained native scope

Final report: [kde-firefox-hint-geometry-2026-10-29-verification.json](kde-firefox-hint-geometry-2026-10-29-verification.json).
Protocol: [kde-firefox-hint-geometry-protocol.md](kde-firefox-hint-geometry-protocol.md).

Current source is /home/ubu/projects/lay-dependency-updates-20261027,
branch codex/browser-matrix-repair-20261027, baseHEADb835ca478198d6cb765a2246329d5646966c118c
plus the scoped uncommitted patch. Source checks13/13 targeted,1123/1123 IME
correctness/packagePASS;1126 catalogued tests includes3 performance tests not
run here. Release build0warnings. No commit, push, release or version bump.

Actual remote KDE IME PID1443794 SHA
004f6285a1725f280a4549c18ffa8d8937d6c27728b4dd0edfddf85a068e1c28,
base version1.0.111. Built source-content SHA
4f69beb0ce810ca5798fcc9e6c278461a7c9af2d0ebb3d706a1232f85255a1f6.
Loaded, installed and activation bytes match after reference/current comparison.
Daemon374525, IBus3168326, tray3228081, manualFirefox988548/profile preserved;
config restored exactly, input ready. GNOME/Chrome and delivery routes unchanged.

Native Firefox159.0 (fixture capabilities and retained manual application.ini),
three real modes (normal/private/fullscreen),10 fields:162 exact prefix
coordinate points,0 overlaps;24/24 Tab and24/24 fast two-word inputsPASS.
URL/email first-word geometryPASS, their second-word exact-Space stepsBLOCKED.
Full diagnostic rows39PASS/9FAIL/6BLOCKED;9FAIL are separate post-Tab Double Shift
controls in rich/plain/iframe-rich. Normal-window differential comparison proves
PASS for ordinary input and the same3 rich failures on old8b84bf5b and current
004f6285 bytes. Do not claim all Lay/Double Shift issues closed or recount the
historical full matrix (Firefox127/55/38, Chrome179/0/41;306/55/79 total440).

The new adapter uses typed composition or existing committed tail, preserving
case. Strict AT-SPI snapshot/bounds reads wait for actual Firefox metadata within
one unchanged80ms budget, using the neighboring Chrome pattern locally. No new
word owner, edit binding, permissive snapshot, coordinate guess or delivery route.
All executable changes are the KDE/Firefox module, its native_document attachment,
and hint_view display dispatch; launcher accessibility is process-local KDE only.

The user authorized self-run native tests. If successive checks stop yielding
new evidence/progress, explain the concrete blocker and call the user for one
manual check instead of cycling. This happened once; wire evidence then resolved
the read race without requiring another manual answer. Keep that agreement.

Next separate issue, if requested: post-Tab Double Shift in fields7/8/10. The
failure is reproduced on reference and candidate; its exact cause is not yet
proved. Investigate its existing owner/first failed transition before any edit.
Do not change correction/delivery routes as a follow-on to this render patch.

## Historical steps below are superseded by the verified state above

# Lay — separate KDE Firefox invisible-input report, 2026-10-28

The accepted post-autocorrection Double Shift fix below stays accepted. New,
separate user FAIL: typed letters disappear until Tab in all ten Firefox fields,
including field1 (latest correction «а нет в 1 тоже»). Firefox-only launcher
now enables process accessibility; currentFirefoxPID988548, same profile,
adapter and window. Daemon/IME/IBus/tray/config unchanged. First letter became
visible but the user reproduced second-word overlap. Fresh actual bounds at
2026-10-28T19:40:01Z end at470; unchanged hint projection starts435.
See CROOT/kde-firefox-second-word-overlap.json. Isolated KDE/Firefox geometry
implementation now passed10/10 short and1120/1120 IME checks; release build
0warnings. Loaded only IME:PID1118793, SHA29b6bc2f7a707dbdf66a877c9c4046f8c8c4bbad7b0124f0f6ae3ff5bd9a93bf.
Daemon/IBus/tray/manualFirefox/config unchanged. Native acceptance pending.
Receipts: CROOT/kde-firefox-owned-hint-{source-v1,build-v1,deployment-v1}/result.json.
User has now authorized automatic testing without their involvement. Preserve
the manual window; use the existing owned native Firefox actor, short first,
then ten fields; measure actual prefix/hint positions, Tab and Double Shift.
If successive attempts produce no new evidence/progress, report the concrete
blocker and call the user for a short manual check instead of cycling blindly.
See kde-firefox-invisible-input-map-2026-10-28.md for exact files, rejected first
hypothesis and bounded passive observation. Keep GNOME and Chrome unchanged.
Do not resume the rejected daemon experiments or recount the full matrix from
this separate case. Automatic input was authorized only after installation.

## Previous accepted case — 2026-10-28T18:18:58.453828+00:00

User explicitly confirmed «да это ты починил». The original first ordinary
KDE Firefox input now supports Double Shift after autocorrection with the
installed Firefox client adapter. This exact case/setup is finished. No rerun,
build, reinstall, browser/service restart or automatic native input to reconfirm.
Reverse toggle, other fields and rapid bursts are not separately certified by
this message. No broader continuation or publication requested in this turn.

Accepted browserPID719893/window67108885, Firefox158.0.1, same profile;
existing adapterSHA924b59822e26f5ad3e6e0daf2a8f762b15e6905bb0b3683ce56bee55744a6f22.
Firefox (Lay) menu entry is in actual KDE XDG_DATA_HOME; OnlyShowIn=KDE.
Daemon374525/IME3556137/IBus3168326/tray3228081/config unchanged; no GNOME change.
Source/Rust guards, receipts and rejected daemon experiments remain as below.
Full historical matrix306P55F79B/440 is unchanged; one user manual PASS is separate.
Owning map kde-firefox-post-autocorrect-comparison-2026-10-28.md and receipt
kde-firefox-client-adapter-2026-10-28-verification.json contain confirmation.

## Setup history retained below; pending-status text is superseded

# Lay — KDE Firefox adapter installed; user manual check pending

One user case: first ordinary Firefox input on KDE, Double Shift after
autocorrection. Exact original/replacement word not supplied. Initial
user reportFAIL, after-setup result unknown. User authorized this fix and normal
restart; manual mode remains, no automatic browser text input.

Installed existing client adapter, not a core fix. New source
scripts/compat/lay-firefox-kde-native and README; C/Snap launcher and functional
Rust unchanged. Scope ADR2026-10-28-kde-firefox-existing-client-adapter.
Library0warnings,2/2 existing testsPASS, shell and graph/architecturePASS.
Same GNOME library bytes:924b59822e26f5ad3e6e0daf2a8f762b15e6905bb0b3683ce56bee55744a6f22.

REMOTE MANUAL FIREFOX IS OPEN AND USER-OWNED:PID719893/window67108885,
Firefox158.0.1, standard10-field page; mapped adapter/inode verified, async GTK
IBus, same actual user profile. No WebDriver or text injection. Do not close or
restart it again without authorization. Firefox (Lay) menu is indexed in actual
private KDE XDG_DATA_HOME, OnlyShowIn=KDE; no default/MIME change. Launcher
~/.local/bin/lay-firefox-kde-native; library ~/.local/lib/lay/compat/
liblay-firefox-ibus-reset-notify.so. Desktop path in verification receipt.

Canonical /home/ubu/projects/lay-dependency-updates-20261027,
branch codex/browser-matrix-repair-20261027,
HEADb835ca478198d6cb765a2246329d5646966c118c,
source1302files/d44e72639f97c0a94802308b97319f60f69b61cc982d0c7c426db505c847edf9.
No commit/public push/release or local GNOME change. Scope guardPASS.
Daemon374525/eb6098b445c48ae5a681fb8b1d33051e28a5da3646d90baa4cd47b09a7228de5;
engine3556137/8b84bf5b6c569550af8e4061ea4cca83e74382d243ae693aeb243a1e22a035d3;
IBus3168326/tray3228081/config unchanged. Cached release daemon9dd150... is
REJECTED, not installed; never copy it to live. Both old daemon experiments
remain rejected/restored. Do not repeat unchanged pacing/pending approaches.

Owning map: kde-firefox-post-autocorrect-comparison-2026-10-28.md.
Compact evidence: kde-firefox-client-adapter-2026-10-28-verification.json.
V1 profile-lock and binary-cache observation mistakes retained; corrected by
read-only v2, without rerun/reinstall or product assertion changes. Setup receipts
are not functional acceptance. Skill scans partial, assessed for scoped policy/
query usage; no skill installation, delegation or cache cleanup.

Latest complete matrix Firefox127P55F38B/220,Chrome179P0F41B/220,
total306P55F79B/440,NOT_TESTED0. No update from setup or two source tests.
Next: user repeats same-word autocorrection and Double Shift in field1, then
reverse toggle/held Shift manually. If FAIL, preserve it and inspect actual first
transition. No broad Rust rewrite or long matrix before short manual result.
No active background build. CROOT /home/e/projects/lay-development-runner/
kde-install-control-20261020; CACHE /home/ubu/.cache/lay/development/
browser-matrix-repair-20261027.

## Previous completed task — KDE Chrome observation

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


Current update: PCAP v3 completed1P2F,146paired/0unpaired; all10native processing ACKs true in0.5–2ms; no80ms wait established. Existing KWin reader24sample median2.6136ms,p954.9099,max8.2701; no keys/restarts, runtime/config unchanged. See current rapid-result-retirement owner. User GNOME/Chrome hint indistinguishable report is OPEN and queued AFTER this fix; no rollback or local GNOME change now. Next diagnostic observes only daemon clock_nanosleep/wait4 timing under an owned strace attachment; no payload/arguments exported, exact detach/runtime checks.


2026-10-29 current source experiment: operation-local readonly Firefox classification reused across processing/retention, same fresh focus/native ACK/actual field contracts. Changes only KDE firefox_replay_receipt.rs and mechanical call seam in exact_ime_tail_replay.rs; no pacing/output/display/route change. Remote RED worker kde-firefox-client-binding-red-v1 running; frozen source no local Rust edits until returned archive accepted. First controlled RED is prior verified scope followed by later unavailable classifier; no installed production change yet. Historical full table remains305P55F80B/440.


READONLY GNOME hint map while KDE full pending: installed style file
/home/ubu/.local/share/gnome-shell/extensions/lay@radislabus-star.github.io/ime_hint_style.js
SHAfba87e9dee827eb0fd0f91e5ac4f51b384d41fa1794f90c65635953aec069d10,
line3 black#000/white#fff, line87 border-width0. Same source style file has no
working diff from b835ca47; last history036f25e5(Oct24 release95), olderb72c50aa.
This proves installed-file contents only, not loadedGJS/UI cause. Local daemon
1627200/SHA2819aa4c,IME1627356/SHA22784990 read only; not restarted/modified.
Do not claim this proves regression source or dismiss user report. No stylefix
authorized/applied in this phase; map/compare first after current Firefox task.

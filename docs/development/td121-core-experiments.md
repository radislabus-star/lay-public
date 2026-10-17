# TD-121: журнал изменений ядра и запрет повторения отвергнутых подходов

## 2026-10-17 — PR53 explicit hot-material fixture replan

CI39074346533 passes the Reset repair but fails immediate publication after a
matching handled release at `e3557fb5`. Missing exact scene/input cache readiness
is a permitted source path, not a historically timed CI diagnosis. Runtime
correctly schedules background computation on a miss. Preserve the intentional
handler-before-observer release path; do not apply Reset choreography there.

The prior review budget is closed; explicit new bounded plan review9/10 groups
three hot-material fixtures using the existing cfg(test) readout contract and
production materializer. Assert unchanged input/frame, cache readiness, no
publication/learning; keep overlap's display checks and all cold/refusal controls.
Fresh827/827 remote source gate PASS. Private omission of release-time draining
is RED at the prepared fixture's publication assertion; production restoration
and9 exact hot/cold/refusal/Reset/timeout controls PASS. `READOUT-FIXTURE-PROOF.json`
records exact bytes, identities and scope. `READOUT-SOURCE-REUSE.json` byte-verifies
unchanged Rust/dependency/config outside the two private IME test files;
combining fresh827 with prior3211 unchanged targets is composed evidence,
not a new full remote run. Current-head full CI remains required.
The [owning entry](../architecture/firefox-input-stability-2026-10-05.md)
and task receipt directory retain both failed CI runs. No runtime/installed or
native authority change is proposed.

## 2026-10-17 — PR53 handler-first Reset fixture schedule

Baseline `8af0b6b` passes3211 remote source cases; GitHub run39071732596
loses the inert phase before actual SSI in the handler-first cursor regression.
Concurrent handler/observer scheduling against the production1ms deadline is
the hypothesis. Poll the real handler to Pending, complete the actual observer,
then resume the same handler; retain all assertions and production deadlines.
This changes test causality only. Focused827/827 and subsequent full3211/3211
remote gates PASS. Controlled removal of pre-Reset live-source capture makes
the regression RED at the original inert-phase assertion; byte restoration
returns PASS. Unobserved Reset refusal and absolute-timeout/no-waiter-leak
controls also PASS; no production change is retained. The ablation is not an
exact historical revision. Receipts `FOCUSED-PASS.json`, `FIXTURE-PROOF.json`
and `REPAIRED-SOURCE-PASS.json` retain exact source, identities and durations;
keep original failed CI and pre-repair source receipts. Final docs/graph refresh
is metadata-only; current-head CI is still required before merge.
The [owning entry](../architecture/firefox-input-stability-2026-10-05.md)
records scope and `/home/ubu/.cache/lay/development/pr53-serde-20261018/` evidence.
Installed89/runtime owners/native acceptance remain unchanged.

## 2026-10-17 — `ltkfq` refusal reproduced in current learned source-state memory

User redirected work from GitHub to diagnosis only. The exact layout target
is generated and initially eligible, but current feedback has negative
state-specific aliases for complete source `ltkfq`. L4 returns `Rejected`;
DecisionCore records `reason=rejected`; committed_tail.rs:140 consumes the
resulting `NoApply(Rank)`. The prefixed source has a different full-text state
identity and remains allowed. Its apparent first-word dependence is reproduced
with current user data, not by a numeric word-index runtime branch.

Private ablation of only those six source-state rejection aliases restores
the first-word decision while retaining generic negative word feedback.
Removing only `rejected_words[делай]` does NOT restore it. Stored weight8 is
not eight proven user interactions. No production code, guard, installed
configuration/model or user learning was edited; no physical input injected.
See [the complete diagnosis and immutable receipt scopes](first-word-learning-diagnostic-2026-10-17.md).
Browser default432 remains349P/50F/33B, NOT_TESTED0. No native fix is claimed.

## 2026-10-17 — autocorrection phrase diagnostic, installed 1.0.89 unchanged

User input: `Вклюай балансировку скорость точно лучш еубдет `.
Expected streaming result: `Включай балансировку скорость точно лучше будет `.
Two damaged spans contain three edits: missing consonant, misplaced Space and
adjacent transposition. The three intervening clean words must be preserved.

Read-only executable diagnosis ran on e@192.168.3.94 under the existing
dedicated-20cpu resource guard. CLI SHA256
`7ea210de19b96e1c03ccfc6ceed0948d4db035aec56e434d1568ea06eb2c7bfe`
matches the locally installed CLI89. The actual local configuration was copied
privately, SHA256
`20fe9fa76066463698ef4edee29733d9ce61bea7f97c901cf4fce22b03856bf5`:
`auto_replace`, `typing_assist` and `nanda_autocorrect` enabled, normal safety.
Existing remote L1.1 v9, L2 v13/DAFSA/productive v90 and all three phase model
artifacts were verified against the local installed model identities. Only the
private admission receipt's artifact/proof paths were relocated; original model
and quality-proof bytes were retained. Private feedback/usage is a copy of the
pre-existing remote snapshot, not asserted identical to current local learning.

Final authoritative diagnostic receipt:
`/home/ubu/.cache/lay/development/autocorrect-phrase-20261017/RESULT-READY.json`;
remote raw output and receipt:
`/home/e/projects/lay-development-runner/autocorrect-phrase-20261017-ready/`.
The private L1.1 service reported `ready` BEFORE all measured queries; its
PID was reaped in `finally`, with `service_closed=true`. Earlier receipts
`RESULT.json` and `RESULT-MATCHED.json` remain separate: the first lacks the
matching model environment, the latter admits queries before L1.1 readiness.
The first direct L1.1 probe returned `still warming` and establishes no lexical
verdict. Do not use those earlier attempts to diagnose missing lexical support.

Final six CLI cases: 3 expected behaviors observed / 3 unresolved, not native
acceptance and not a new browser denominator:

| Case | Final selected action |
| --- | --- |
| `Вклюай ` | No apply. L2 productive typed exact retains `Включай `, but admission is `SuggestOnly/short_internal_consonant_drift`. |
| `убдет ` | Selects `будет ` through the existing adjacent-transposition candidate. |
| `лучш ебудет ` | Selects `лучше будет ` through the existing boundary candidate. |
| `лучш еубдет ` | No apply. The boundary producer offers only `лучше убдет `; `edit_transition_not_verified` rejects this intermediate pair. |
| Entire reported phrase | Same unresolved final-pair transition; no apply. A single CLI resolution acts on the final tail, and cannot establish earlier per-Space correction of its first word. |
| Entire clean expected phrase | No rewrite selected; clean control preserved. |

First semantic limitations, separately from delivery:

1. Direct warmed L1.1 restore/lattice for `вклюай` abstains
   (`outside_calibrated_basin`) without retaining `включай`. L2 recovers the
   desired candidate, so its final loss of apply eligibility is the generic
   short internal-consonant admission guard. There is no grounded L1.1 Winner
   in this measurement that could justify bypassing that guard. Inspect general
   missing-letter support/authority evidence before proposing a runtime fix;
   never add this word or remove the guard to turn the example green.
2. The existing pair producer moves a boundary character but does not compose
   that operation with repair of the residual right token. Both individual
   operations succeed in controls; their desired combined final candidate is
   absent. The verifier correctly rejects `лучше убдет`, whose right token
   remains corrupt. Any future bounded composition belongs in the existing
   candidate/decision contract with one fully validated final pair, not two
   independently emitted IME edits and not a relaxed verifier.

Next discriminating evidence, before production edits: an actual remote IME
per-Space trace on the owned Kitty test field, with the same model identities
and explicit FieldSnapshotOnly scope. CLI FullReferenceAllowed, cold process
startup/explanation timings (~1.5 s here), direct L1.1 readouts and physical
client acceptance are different scopes. No UI latency or installed-client fix
is claimed. No production code, runtime authority, delivery route, installed
configuration, learning or model bytes changed; no new architecture promoted.
Latest complete native browser receipt remains Firefox170P/27F/19B and
Chrome179P/23F/14B, total349P/50F/33B of432, NOT_TESTED0; those layout results
do not certify this autocorrection phrase.

## V whole native result rejected

Full432339P60F33B, onegain/three lostPASS vsU; all20cursorPASS preserved.
Both scopes closed/VT1/all18primaries hashed. Do not publish or count sourcePASS
as acceptance. Source firstscope-loss proof next; review unavailable, no score.
[Owning evidence](../architecture/firefox-input-stability-2026-10-05.md) and V decision contain all precise receipts.


## V canonical53 PASS; proceed scoped source gate

Final signature cef52d3805060d185cc64a325e389215fb4d08c9c0a4da5b19f969a0f1e260a7,
23.901 s, all208 Rust hashes verified. Early KnownStart routing veto remains
rejected; new fixture checks actual source/effects, including spent-vs-live
provenance. See owning document and V decision. Native unchanged; task OPEN.


## V rejected early guard; no runtime rollback

Canonical source gate841/842 rejected early KnownStart contradiction veto; that
experiment was never built/installed. Original typed routing restored; inert
actual progress repair retained. V3=3 is routing, not accepted source. See owning
V rejection evidence and decision; adverse fixture now checks actual V2/hand-off/
replay/edit-effect boundaries. Do not reintroduce the rejected early gate.


## V focused53 PASS; task OPEN

53/53 source PASS, eight actual Reset schedules and ten adverse cases; independent
review unavailable (thread limit). See [owning evidence](../architecture/firefox-input-stability-2026-10-05.md)
and `2026-10-14-private-boundary-first-target-chunk.json`; native341/58/33 unchanged.


## V adverse handoff: separate baseline-existing contradiction located

The new progress positive test passes all four prefix/source-separator variants;
actual complete SSI confirms native delivery and the real ManualToggleV3 delegates
exactly once. Test-only final readiness now uses the established closed-boundary
contract (`!completed_native_replay_awaits_exact_receipt` + native ACK + actual RPC),
instead of predicates specific to an open UnknownStart suffix.

The adverse owner-change test still FAILs at lease handoff: current actual SSI
contains only partial replacement at the old caret, yet ManualToggleV3 returns
`(3,false)`. Controlled U baseline reproduces the same rejection assertion with
all three affected U production files hash-verified, 0 PASS / 1 FAIL, 13.666 s,
signature `9f13df4fc5ab2576f71ae36d6ee45420ff60e3eea7a828f292139214069b54e3`.
Temporary U source was restored to V in a finally block; private runtime unchanged.
This is a pre-existing admission gap, not a rollback of the retained cursor fix.

First lost guard: factory transfer keeps KnownStart while old native completion
scope is retired; `context_allows_manual_toggle` treats KnownStart as sufficient
despite an actual contradictory document/caret receipt. Recommended repair (9/10)
uses the existing actual-snapshot/owned-tail agreement predicate for this manual
admission; broad pending-replay transfer (7/10) is deferred; weakening or dropping
the adverse fixture (1/10) is rejected. Keep current token/reducer and final
verifier authority, active-preedit ownership, delivery routes and model bytes.

Receipts: `/home/ubu/.cache/lay/development/remote-firefox-setup-j2he3qf6/V-U-baseline-handoff1-RESULT.json`
and `V-U-baseline-handoff-wrapper.json`. Native matrix remains 341/58/33 of 432;
no new native run and TD-121 remains OPEN.


## V first clearing call measured; minimal inert receipt planned

Remote source proof 50 PASS / 1 FAIL (51), signature
`c6e0ca85fbb9cd78a0b2c5677ba6c01af64c36e8963a9132e3d98316f5cd7452`,
21.513 s, all 208 Rust source hashes match. Actual erasure SSI is present;
`exact_replay_mirror_transition` clears it on the first replacement press.
The pending Reset receipt is absent at that point and is recreated after Space.
This explains why adjacent actual target growth cannot be established from the
current snapshot alone. Native equivalence remains a required later check.

Recommended option (9/10): one inert actual progress receipt in the existing
client observation state, bound to paired replay scope, actual SSI revision,
focus and layout generation, spent at the next adjacent SSI. Shared mutable
scope witness (7/10) has a wider equality/revision impact. Shape-only admission
(1/10) is rejected. Do not restore old client text as current or grant manual,
Space/Tab, learning or edit authority from the retained witness. Final fresh
full text/end-caret receipt, reducer provenance and verifier remain mandatory.

Receipt: `/home/ubu/.cache/lay/development/remote-firefox-setup-j2he3qf6/V-first-clear-red51-RESULT.json`.
No model training, delivery route change, runtime installation or native input.
U complete native result remains 341 PASS / 58 FAIL / 33 BLOCKED (432), TD-121 OPEN.


## V source-only RED51: actual prior receipt lost before first target piece

Nativeprivate7 firsttargetchunk failure has a source choreography reproducer:
realerasureSSI→nativecompletedSpace→firstgrowingtarget/stationarycaret.
50oldcontrolsPASS/1newFAIL, initial21.6367s; afterprefailurestate assertions
same50P1F21.5750s sig a923a7c6939336982a91753b38534d7dc59eaaa29c1ff5eb50fc52d92f43045d.
Prioractual=None even though actualerasureSSI had beenreceived. Pending/current
boundarytoken match KnownStartlineage7/suffix0 and completedboundaryprovenance
with fullprefix is present. NoRootruntimechange yet: locatefirstactualsnapshot
clearingcall and exactfailedprogressconjunct beforechoosingminimalretention.
Do notinfer permissionfromprefix/geometry or weaken finalidentity/SafetyGate.
CoreU20cursor gains retained; full432341P58F33B is unchanged. GUIOFF/VT1;
sourceproofs/readsfmt executedREMOTEonly. Next test-onlydiagnostic may assert
snapshot after each actualnative key; no unchangedrandomnative retry.


## Current U full browser result:341PASS/58FAIL/33BLOCKED432

Both original9-job matrices completed on Uae225/cbc060; both exactscopes
closed/allfivePID-starttick pairs gone/VT8→VT1. Firefox162P35F19B216:
all10cursor stableIDs gainedPASS, oneRoldPASS lost(privateSpacefield7
rapid5stopsafterfirstflip), no otherchanges. Chrome179P23F14B216:
all216statuses equalR. Combined10newPASS/1lostPASS/9netPASS across432.
Bothbrowsers now20/20cursorcasesPASS; incomplete/no promotion because
58FAIL33BLOCKED and one requiredclass regressed. Keep provedcapture-order
repair; do not blindlyrollback or rerununchangedcaseuntilrandomPASS.
ProofSHAs:FF152185ead8273ea3cd266052c7b3e23743267761051c5d17837b70ce4cebaf39,
Chrome5e6679c4565f1f8bef57159b2ffb4900aeb2fbdcf9127fe88d769132f58128d7.
Speeds standard:p95FF118.806ms416/420, Chrome111.629ms406/420;
functionalDS141.682ms122/210and128.741ms158/210, missingexplicit.
Clockevdevwrite-return→exactAT-SPI, kernelreleaseNOTMEASURED.
Chrome3visitedLayfilesprivateunchanged/publicrecent_actionspositive;
tracecoverageUNKNOWN/no completeauditclaim. All18primaryactorJSON retained
in immutableRAWarchives; localcopy70516pendingthenverifyhashes.
Nextfirstsharedfailure: privateFFspace7 completednativeReplay intermediate
text4/caret0 rejects currentidentity before target7arrives. Exact331-event
packet retained`U-firefox-private7-first-burst-loss.json`; no rootcause
at specific source transition yet. Source840/48PASS retained; newcausal
proof before nextcoreedit. Freshindependentreview NOTRUN agentthreadlimit.
No modelrefit/localexecution/input/install/RDP/publication/TD121DONE.


## U full Firefox complete:162PASS35FAIL19BLOCKED216; Chrome running

All10cursor stableIDs gainedPASS againstR; exactlyone oldPASS lost:
standard/private/space/field7, rapid5 burst stops after firstnativeflip.
All216IDs compared; no otherstatus changes. Uoverall stillFAIL/no promotion.
FFsourcebytesae225/cbc060; original051–059all0/noheldkeys, exactfiveGone/
cgroupempty/VT1; primary9JSON preservedinto rawarchive immutablemanifest.
Proof`nativeae225-firefox-proof.json`, compare`U-firefox-vs-R-stable-ids.json`.
Standard416/420timings:p95118.806ms; panel48/48p95117.754ms;
functionalDS122/210p95141.682ms, Space37/50p9523.996ms, Tab/selection0/10.
Missingexplicit; evdevwrite-return→AT-SPI, kernelreleaseNOTMEASURED.
Chrome samebytes liveactual33036/driver32147, zero-keyUBscope3985161;
normal100 thenprivate101sinkbrackets thenoriginal102–108, activeafseat.
NoheavywhileGUIlive. Rcompleteboth432table332P67F33B retaineduntil bothU216
complete. Source840+48causal/adversePASS does not certify all nativecases.
One private7 regression remains firstnextmechanism afterChromefinish;
do not discard10cursor gains or rerununchangedcaseforrandomPASS.


## U scoped source gate PASS840/840; native acceptance pending

Fresh immutable`run-uA2KuH`, local`run-eorp9z31/RESULT.json`, archive
b4c98b96e437fa9c978b2da6f0f944659885fa000f44236661b4aded1776a88a.
IME789+sixcontracts51PASS; all737Rust hashes match. PriorRdaemon273 reused
on unchangedsource, no repeatedexecution; development scope, not release.
Source48 causal/adversePASS retained; U AST/privatebuild LIVE56245.
Root capture-order repair only, matchedRestore/finalmanualauthorityunchanged.
No newroute/owner/deadline/model fit; Rfull332P67F33B remains native baseline.
Freshcontextfree review NOTRUN due agentthreadlimit; no score invented.
T originalASTreceiptretained; later graph/report artifactsdrifted after edits/
hooks, localcopyhashassertionfailed. Do not claimT artifactverification.
Binding/generatedreceiptstillmatch; UfreshAST willbindcurrentRust separately.
Next fresh finiteownedSeat-af after oldT exactclosed/VT1, fulloriginal9jobs
Firefox/Chrome each withspeed/private-sinks. No localinstall/input or promotion.


## U exact causal RED confirmed; minimal receive-order repair pending proof

Source47:46oldcontrolsPASS/1newcaseFAIL,19.7413s, signature
1ba5e257c6b37f064cc49875e9f71ab3effa717b9399e4d704c3e35271fd1bc4.
ActualReset handler concurrentwith canonicalobserver loses inertphase at the
firstLeftReset, matching nativeT firstloss. All208Rust hashes verified before
codeedit. Root changed only capture_native_cursor_reset: existingexactlive
preReset identity OR existingobservedReset provenance can supply the inert
clone. Restoration stillrequires independentlyauthenticatedMatchedReset,
sameactivation/currenttoken/exactreduceroldsourceprovenance. Otheroldguards,
UnknownStart, manualonly/endcaret/actualSSI, readout/Space/Tab/learningrefusal
and everyroute unchanged. This repairs eventorder; no additional owner,wait,
timeout or wordcondition. Focused47/furtheradverse/sourcegate/native pending.
Rfull332P67F33B remains acceptance; no core success or promotion claimed yet.


## U first loss localized: handler-first Reset capture; causal source running

T same35withoutwire:standard20PASS, functional9P5F1B,cursorFAIL; both
actual83008/driver60771closed0, parent9279280/noheldkeys, exactfiveGone/CGempty/VT1.
Failure trace1178:phasepresent/identitycurrent/localtokenlive, awaitingLeft
actualeffect; resetspent/atomic/sensitive/sealed/quarantine allfalse. Capture
returnsNone BEFORE Reset11620 observer_ingress1180. NextLeft has no phase.
Allothercapture conjunctions proved; required reducerReset provenance was
queried before canonicalreceive recorded it. SuccessfulSpositive logging
allowed observer-first timing; it was not a functionalrepair. Exactpacket
`T-reset-order-first-failure.json`; tracecoverageUNKNOWN/bounded512000B.
New source47 fixture calls real handler-first Reset concurrent with real
observer (futurezip), no withheldreceiver/artificialtimeout. SourceRED before
coreedit. Minimal9/10option:inertclone from existingcurrentpreReset identity
or existingobservedReset, then restore onlyafter authenticatedMatchedReset
and unchangedreducerprovenance/sameactivation guards. OtherResetowners audited,
no blanketpredicate relaxation. Freshreviewspawn again rejectedthreadlimit;
NOTRUN/no scoreinvented. Rfull332P67F33B remains solecomplete acceptance.


## T failure-only diagnostics: source gate running, no core repair

S fresh15, SHnormal20+15withwire and SI same35withoutwire allcursorPASS.
The20normal history and passive wire absence did not reproduce RcursorFAIL.
Built-in positive-path logging can still change scheduling; fullRhistory
remains untested on diagnostic bytes. T removes successful-path diagnostic
calls, retains only records after original refusal/discard/capturefailure.
Original R predicates, mutation, owner/route and deadline remain; no refit.
Fresh scoped IME+6contracts gate LIVE8158, oldSIexactcgroupempty/5PIDgone/VT1.
SI actual75 was owned-model-server descendant cleanup after both actorparents0;
PID3738319/t14390320 stopped with authenticated pidfdSIGTERM; no unrelated
process changed. Receipt`SI-owned-orphan-cleanup.json`, strongSIclosure retained.
Scores/options/limits: `decisions/2026-10-14-cursor-failure-only-diagnostics.json`.
FullR332P67F33B remains acceptance; no partial PASS promotion.


## S native diagnostic complete:10PASS4FAIL1BLOCKED15; cursor PASS only

Original public Firefox field1 15plans +passive wire on metadata-only S167ca/37082
completed. Exact five processes gone, cgroupempty, VT8→VT1. Cursor phase
seeded, allactualreceipt transitions preserved and exacttargetPASS. This is
not a full R-state-history replay or a functional repair: retained R full
cursor10FAIL remains. AllfourRShiftcallbacks in the failing excerpt were
admitted; the invalid source Shift-timeout fixture is not that native cause.
OtherS failures:prefix/undo/selection/focus, TabBLOCKED. DiagnosticDS12/21
p95142.068ms, Space4/5p9521.512ms; missingexplicit. Observer/logging overhead
NOT_MEASURED, asynctracecoverageUNKNOWN, bounded512000Bprefixnotpreserved.
Actor+wire hashes verified/local`S-diagnostic-field1/{RESULT.json,WIRE.json}`.
First failed R cursor-phase transition stillUNKNOWN. Compare retained states
before any new behavior change. Rfull332P67F33B remains the acceptance table.


## S diagnostic source gate PASS837/837; native pending

Immutable remote gate`run-QyJfLT`, local`run-9xuz3ad5/RESULT.json`, archive
925cb3c43883f4a97a7e4a7fb3b12102626811fdb9997f5a0201fa885338bb64.
IME786+sixcontracts51 PASS, all737Rust hashes match; development scope, not
release. R daemon273 evidence retained on unchanged source; not re-executed.
Remote AST/private build sequential LIVE69800. Native diagnostic not yet
started; GUIclosed/VT1. R complete332P67F33B remains sole complete baseline.
Metadata logging only, no new ownership/edit/delivery/training authority.


## S45 coverage PASS; scoped metadata diagnostic pending

Corrected source45/45 PASS19.0167s; all208 source hashes verified, no runtime
fix established. Existing trace switch now records cursor phase/grants,
actual scalar caret/count, last key and Reset capture/restore/discard.
Numeric/boolean metadata only, no word/token dump or new delivery/admission
rule. Two source files only; existing conditions and mutations unchanged.
One original public Firefox field1 diagnostic follows a fresh scoped source
and immutable private build. Full332P67F33B remains R acceptance. Diagnostic
overhead/async writer completeness UNKNOWN. Fresh context-free review spawn
was rejected by agent thread limit; no review or score is fabricated.


## S RED interpretation corrected: artificial Shift stamp timeout

The additional assertions locate loss at the first handler-first Shift
release. The helper deliberately waits for the Shift callback to return
before running ingress observation. Shift is excluded from text-inert
completion by its protected ordered-admission contract. Withholding the
observer past that deadline causes expected refusal; this RED does not prove
a runtime defect or the Firefox root cause. Keep all three RED receipts.
Corrected source fixture uses existing received observer-first Shift callbacks;
original movement/insert release-order variants remain. No production edit.


## S source-only causal RED:44PASS/1FAIL45

Actual prediction publication plus four received Shift callbacks reproduces
loss of cursor manual-source authority on unchanged R production source.
Receipt: `/home/ubu/.cache/lay/development/remote-firefox-setup-j2he3qf6/S-causal-red-RESULT.json`;
signature6928aa83a832ba5a3db159add7d2261b0155e6b41590bff649b3adfe11211c04,
18.8101s. All44R controls passed. First exact event not yet localized;
assertions added before/after Shift press/release before runtime edits.
This does not change the native332P67F33B table or establish a Firefox fix.
Both GUI scopes closed/VT1, no keys running. R10Chrome gains retained;
no model refit, local execution, delivery-route edit or installation.


## Current R browser result:332PASS/67FAIL/33BLOCKED432

Both original9-job matrices completed on R64dfd/20adb; both exact scopes
closed/all5PID-starttick pairs gone/VT8→VT1. Firefox153P44F19B216,
all216verdicts equalQ. Chrome179P23F14B216:all10cursor scenarios gained
PASS, zero lostPASS across216stable IDs. Combined10newPASS/0lostPASS
across432, no promotion because67FAIL/33BLOCKED remain. This is a native
Chrome improvement, not a Firefox fix or model retraining.
Chrome receipt native64dfd-chrome-proof.json SHA
acf2ea807da3db679563bccf53f7eb17c82cb4c3e1ee25ead7a7ca2e9ac783d1.
Firefox a6747721a0a3d14daf281eb9a909c79ccdde73fcdd559f5a85cb855ead111a76.
Standard:p95Firefox118.867ms420/420, Chrome110.429ms406/420;
functionalDS Firefox142.074ms100/210, Chrome126.096ms158/210.
Space37/50and38/50, Tab0/10each, selection0/10FF10/10Chrome.
Allmissing remainexplicit;evdev write-return→exactAT-SPI, kernelrelease
NOT_MEASURED. Scopedprivate writer proof:3visitedfilesunchanged, public
recent_actions positive; tracecoverageUNKNOWN, no universalauditclaim.
Source1109/1109+review9 retained; R mechanism stays as next baseline.
Next: prove the first native Firefox cursor transition lost in R; source
fixtures currently omit actual visible prediction/gesture/interleaving.
No unchanged failed input retry, no core rollback, no production installation.


## R full Firefox native FAIL153PASS/44FAIL/19BLOCKED216

Original9 jobs finished0/noheldkeys. Exact scope closed, all5 PID/starttick
pairs gone, VT8→VT1. Full R Firefox receipt `native64dfd-firefox-proof.json`
SHAa6747721a0a3d14daf281eb9a909c79ccdde73fcdd559f5a85cb855ead111a76,
localbdir:standard60PASS, panel6PASS, functional87P44F19B. All216stableIDs
match Q:0newPASS/0lostPASS. Native cursor repair remains unproved/FAIL.
Standard420/420 timing samples:p95118.8668ms; functional DS100/210
p95142.0744ms, Space37/50p9526.0197ms; Tab/selection0/10, missingexplicit.
Clockevdev write-return→exactAT-SPI; kernel releaseNOT_MEASURED.
Raw native64dfd-firefox-raw-1791942539350123327 plus all9primary JSON
receipts retained unchanged with supplement. Source1109/review9 remain
separate. NextChrome original9 onR, then causal source/live boundary work.
No promotion, production installation, core rollback or unchanged retry.


## R current native partial: cursor still FAIL, no promotion

New R bytes pass Firefoxstandard60/60 andpanel6/6. Original functional
fields1–2 give18PASS/10FAIL/2BLOCKED30; both cursor cases still refuse the
first exact Double Shift although actual source and caret match. This is
not a native fix. Source1109/1109 and review9 are retained separately.
Excerpt `R-native-cursor-first-failure.json` under localbdir retains the
capped async trace and precise source/candidate binding. It does not log
CursorEditing presence or prove its first exact loss. At firstLeft there
is a visible display prediction, full actualSSI6/caret6 and UnknownStart;
manual RPC later refuses context_authority. No new hypothesis is promoted
without a causal source schedule. Finish original full matrices; no retry
or actor changes and no core rollback. Full Qtable stays322P77F33B432.


## R final source gate PASS1109/1109; native acceptance pending

Focused44/44 and fresh eight-target1109/1109 PASS, immutable remote
run-wznQ1e, archive `0e127076801a8318e4de8fb4d35f3089087f69f0afd42748eaafb5155a6e5e62`. Local receipt
`/home/ubu/.cache/lay/development/run-6x6vdp47/RESULT.json`; all737Rust file hashes match this checkout.
Final independent review2/2 closed9/10; no third review. Old failure receipts
and deterministic six-ID registration RED remain retained. The tested fixture
correction uses the existing exact dispatcher Ping barrier; runtime routes
are unchanged. Next: remote AST, immutable private build and full original
Firefox/Chrome9jobs each with latency/privacy. TaskOPEN; Qfull table remains
322PASS/77FAIL/33BLOCKED/432 until new native matrices complete.


## R current next gate

Final read-only review2/2 is completing. Registration causal RED is retained
and the existing strict dispatcher barrier now covers Cycle09 registration
and the interior fixture's transition to metadata. First barrier44 passed
all old controls/erasure and found one queued exact metadata reply; that
follow-up uses the same barrier, not a runtime or timeout change. Final
focused44 running; then fresh broad gate. Prior full failures remain retained.
No candidate build/staging or changed-byte native result.

## R fixture registration handoff: causal RED six IDs

Second broad1108/1109 failure is retained: manual V3 erasure refusal despite
valid snapshot/revision/epoch and exact authority. New registration oracle
proves six focused fixture IDs entered engine registration with unfinished
exact absent-object callback replies. Erasure:Reset63011+release63012; cursor:
End release95047. Receipt `/home/ubu/.cache/lay/development/remote-firefox-setup-j2he3qf6/R-registration-causal-red/RESULT.json`,38PASS/6FAIL/44. This proves
the completion gap; the historical receipt does not prove which callback was
auto-dispatched again. The strict existing standard Ping barrier is now reused
by Cycle09 manual/visible/private-visible helpers before object registration.
No runtime change or weakened expected verdict; focused44 running, then a
fresh broad gate. Final read-only review pass2/2 is running on concrete code
and this evidence. Private Q bytes and full browser table remain unchanged.

## R full source first gate:1108/1109, guard preserved

First broad R receipt `/home/ubu/.cache/lay/development/run-sgdcr974/RESULT.json`
(remote run-QFIH5m, archive f77d37cdee89ccc659cd2b403452e4a5c58144609104e3525d50a9ca1b37c00d)
executes1109, passes1108, fails one existing navigation safety test. The
explicit manual gesture correctly refused an unconfirmed/interior cursor
source but did not spend its inert pending phase. Keep the existing test
unchanged: that refusal now discards only the cursor phase. It cannot be
revived by a later SSI. Focused43 running; fresh full gate and final review
remain required. No build/native staging or authority change.

## R focused41 GREEN; full source and native still pending

All eight authenticated cursor re-edit positives now PASS in both release
orders, with empty/prefixed and restored/changed text. Q38 prior controls and
two purpose/adverse-receipt tests also PASS:41/41,18.453s. Receipt
`/home/ubu/.cache/lay/development/remote-firefox-setup-j2he3qf6/R-first-green/RESULT.json`. The remaining handler-first failure came from normal
Reset cleanup publishing a new tail epoch before the inert receipt was rebound.
The same existing pending slot is now rebound before cleanup; no release
settlement, word-start grant or extra owner is invented. Generic readout,
Space, Tab and learning remain excluded until explicit manual consumption.
Additional identity/metadata checks and full source gate are pending. No new
private build/native run: full Q table remains322PASS/77FAIL/33BLOCKED.

## R cursor provenance: causal RED and unwired draft — 2026-10-14T00:46:00.937428+00:00

Actual cursor trace first loses locally owned suffix on Left, before End. The
source predecessor is UnknownStart with an existing exact manual grant; this
experiment does not grant KnownStart. Authenticated source RED39:38 prior
controls PASS, one new test FAIL across all eight prefix/edit/order variants.
Observer-first refuses authority; handler-first additionally refuses admission.
Receipt: `/home/ubu/.cache/lay/development/remote-firefox-setup-j2he3qf6/R-causal-red/RESULT.json`, proof signature
`f0f6fef006b6ddfdd9a5022092778a87d920d2879b320b81ddfc82a40eba6fc0`.

Local CursorEditing helpers occupy the existing pending receipt slot. They are
unwired, uncompiled and uninstalled. Plan review1/2 scored8/10; final review2/2
remains reserved for verified implementation. No general suffix/Space/Tab grant
is permitted. Current installed private test bytes and full432 table remain Q
322PASS/77FAIL/33BLOCKED. GUI is closed; laptop runtime authority unchanged.
Next: wire exact authenticated key effects, actual SSI, matched Reset and
manual-only consumption; reject stale or contradictory receipts.


Обновлено: 2026-10-13T23:24:26Z. Checkout: `/home/ubu/projects/lay-space-boundary-shift-20261007`,
ветка `codex/space-boundary-shift-20261007`, HEAD `d362e7c82cb14f3d2aab123b9130707f1843ba02`.
Владелец архитектуры и полных доказательств:
[firefox-input-stability-2026-10-05.md](../architecture/firefox-input-stability-2026-10-05.md).
Этот журнал индексирует эксперименты и определяет порядок следующего шага;
полные журналы, исходники и измерения остаются в указанных receipts.

## Текущая опорная версия

**Q — SOURCE PASS / BOTH FULL BROWSER MATRICES FAIL / NO PROMOTION.**
Неизменный source gate: `/home/ubu/.cache/lay/development/run-61bj0v7o/`,
remote `run-cht3Zw`, archive SHA-256
`e1f0e3864aeb3f1f9669270960b3b0119367ce2e130d17f5f7a5214a56adf8f5`.
Focused38/38 и свежие восемь development targets1105/1105 PASS;
два независимых review9/9, remote AST734 Rust hashes совпадают.
Это выбранный development scope, не весь репозиторий и не релизный PASS.

Текущие private test bytes:
IME `399ffa890002bc6ce55b869bc30138f77656574b55be173fd59c91d37f7a5291`,
daemon `c49c44cb7776051433fc7d77ad0d5ac4d5f78f04b402895ffe1363950c3e3d97`.
Оба GUI закрыты: exact scopes empty, исходные PID/starttick gone,
VT8→VT1. Локальная установленная версия не менялась.

Q Firefox: standard60PASS/60, panel6PASS/6,
functional87PASS/44FAIL/19BLOCKED/150 — 153PASS/44FAIL/19BLOCKED/216.
Все216 stable IDs сравнены с O: один новый PASS (private/active/field1),
ноль потерянных PASS; остальные215 verdicts совпадают.
Diagnostic private20/20 PASS сохранён отдельно от исходных девяти jobs.
Standard speed420/420 measured, p95 120.982ms; kernel release NOT_MEASURED.
Первичный receipt: `native399ff-firefox-proof.json`, SHA-256
`93af9b30e3faf40622f4c6bda2dee877b5766ec4b6eed1dd35ad762753b72de6`.

Q Chrome:169PASS/33FAIL/14BLOCKED/216; все216 stable-ID verdicts совпали с O.
Standard58PASS/2BLOCKED, panel6PASS, functional105PASS/33FAIL/12BLOCKED.
Standard speed406/420 measured, p95 113.861ms;14missing остаются явными.
Primary receipt `native399ff-chrome-proof.json`, SHA-256
`73a88c47a24bff2d7d54cff2e9db09bec3f06865f8f23f5d74f926a0fd3d1365`.
Private writer-root scoped proof PASS: три посещённых файла не изменились;
public positive control изменил recent_actions.jsonl. Trace coverage UNKNOWN.
Текущая полная Q таблица:322PASS/77FAIL/33BLOCKED/432;1новыйPASS/0потерянных.
Receipts и stable-ID comparison лежат в
`/home/ubu/.cache/lay/development/remote-firefox-setup-j2he3qf6/`.

## Реестр последних экспериментов

| ID | Гипотеза / изменение | Проверка | Решение и запрет повторения |
|---|---|---|---|
| M | При действительном replay текст может опережать неподвижный caret; сохранить только строгий парный прогресс до точного конца | causal RED, focused5 PASS, включён в N1092; отдельный native M не заявлен | Сохранить механизм и отрицательные проверки. Не расширять произвольный caret lag |
| N | PRIVATE запрещает обучение/историю, но разрешает явный Double Shift в обычном поле | focused19 PASS, full1092 PASS, Chrome-private20 PASS + реальные writer roots; полные browser matrices FAIL | Сохранить разделение приватности и защищённых полей. Не возвращать общий запрет Double Shift по PRIVATE |
| O-R0 | Уже подтверждённое слово теряется, если managed release обрабатывается раньше observer | causal3:1FAIL/2controlsPASS; `o-late-managed-release-red-ad5ufwag/RESULT.json` | Причина воспроизведена. Старый тест closed-word/preedit не заменяет active managed-commit repro |
| O-R1 | Любой plain release текущего владельца сделать Passive, cleanup обработать раньше ожидания stamp | focused17 PASS, но full1096:1081PASS/15FAIL; `run-epln3f0l/RESULT.json`, remote `run-WE0tvu` | **REJECTED.** Ломает обычный release fence и ordered replay settlement. Не повторять широкую Passive-классификацию. Код сохранён отдельно; ни сборки, ни установки |
| O-R2 | Сохранить обычный Key/fence; только факт уже завершённого подтверждённого release хранить в существующем bounded stamp store и поглощать при точном позднем ingress | Source1101 PASS, review9; full Firefox152P45F19B/Chrome169P33F14B | Сохранён узкий механизм; source PASS не закрывает native FAIL |
| P | Поздний release после подтверждённого Reset должен завершить уже обработанный press, сохранив ранее полученные context metadata без восстановления old word authority | Focused33/full1104 PASS, review9/9; private diagnostic19P1F | Сохранён. Первый новый отказ — точный acknowledged NBSP потерян при held-Space Reset; полный native P не повторяли после диагностического FAIL |
| Q | Сохранить точное acknowledged представление owned Space, включая NBSP, через существующее одноразовое held-Space Reset | Focused38/full1105 PASS, review9/9; diagnostic20P; full Firefox153P44F19B | Сохранён; Chrome Q следующий. Никаких новых owner/route/таймеров или послаблений proof |

## Отвергнутые решения

- Увеличить stamp timeout: маскирует порядок обработки и добавляет задержку;
  не доказывает право на слово.
- Повторять ввода/refresh до случайного PASS: не исправляет первый переход,
  меняет сценарий и знаменатель доказательства.
- Восстанавливать KnownStart по наличию буфера или handled-cache: cache не
  доказывает текущего владельца, содержимое и границу слова.
- Делать observer Passive для всех ordinary releases до исполнения callback:
  отвергнуто полной source-проверкой O-R1, даже при focused17 и review9.
- Ослаблять SafetyGate, edit verifier, protected fields, точный snapshot или
  добавлять исключение по слову/браузеру: не является исправлением механизма.

## Порядок работы, предотвращающий круг

1. Перед правкой прочитать этот журнал и найти первый потерянный переход.
   Записать отдельный ID, точную опорную версию, изменяемый механизм и ожидаемый
   результат. Не начинать второй эксперимент одновременно.
2. Отвергнутый механизм повторять только с новым причинным доказательством,
   которое опровергает прежнюю причину отказа. Указать ссылку на него до правки.
3. Сохранить исходники и receipts каждой неудачи. Возвращать только изменения
   текущего отвергнутого эксперимента, сохраняя проверенные M/N и прежние fixes;
   не откатывать ядро целиком и не использовать dirty main как базу.
4. После узкого causal/controls PASS включить **все ранее упавшие проверки этого
   механизма** в обязательный набор. Затем пройти весь прежний affected scope,
   сохранить совпадающие/новые/потерянные PASS по стабильным ID.
5. Code review и native build после общего source PASS. Review score не заменяет
   проверку. Лимит review каждого описанного эксперимента — максимум два прохода;
   доказательства отвергнутого варианта не переносить на другой код.
6. Native на точных новых байтах: исходные Firefox/Chrome60+6+150, скорости с
   required/measured/missing, реальные writer roots для приватности. Source PASS
   не закрывает native FAIL/BLOCKED/NOT_TESTED.
7. Менять статус на DONE/commit/push только после обязательных доказательств.
   Перед передачей обновлять текущее состояние и ровно один следующий шаг.

## Следующий шаг

R-D0: на неизменных Q bytes выполнить один bounded public Firefox field1
диагностический corpus15 с неизменным actor/body и пассивным exact IBus wire
capture, включая FocusIn/Out, Reset, SSI, key callbacks и output signals.
Это diagnostic, не новая полная матрица. Обычный trace512KiB ротационный,
prefix_preserved=false, поэтому он не доказывает начало всех старых отказов.
Нужен первый измеренный переход для общих focus/cursor failures; лишь затем
варианты решения/score/recommendation, causal source RED и review. Не менять
preedit mode по гипотезе: текущий source уже отправляет CLEAR=0. TD-121 OPEN.

### Исторический следующий шаг O-R2

Закончить source draft O-R2 и его отрицательные проверки: observer-first fence
сохраняется; handler-first matched managed/native release не оставляет orphan;
completion одноразов, связан с полным header и key tuple, ограничен старой
capacity и не даёт прав после FocusOut/Reset/ContentType/owner replacement.
В новый focused набор обязательно включить все 15 упавших строк O-R1, затем
пройти тот же общий development scope N. GUI остаётся выключен до source PASS.

## 2026-10-13: проверка повторов и первый O-R2 source verdict

Проверены22 сохранённые candidate BUILD с неизменными source request:22 разных
отпечатка файлов IME, точных повторов0. Исключены отдельные test paths; inline
tests/comments могут оставаться в файлах. Это проверка точных байтов, не
семантической идентичности и не всех промежуточных незаписанных правок. Receipt:
`core-built-candidate-repeat-audit-20261013.json` в текущем evidence cache.

O-R2 первый source focused28:27PASS/1FAIL, 16.835374405s,
`o-completed-release-source-okgy8u25/RESULT.json`. Все ранее упавшие O-R1 test IDs
проходят. Единственный новый test incorrectly demanded identical generation
after a Reset whose local scope/metadata was not eligible for the fast path.
Требования разделены без изменения runtime: отдельный actual admitted
UnknownStart positive сохраняет точный token/scope; unproved post-Reset negative
не получает KnownStart или manual authority. Старые guard tests не изменены.
Focused29 и общий affected scope ещё не проверены; не начинать native/review.

### 2026-10-13T19:39:31Z: точный повтор найден и автоматический stop проверен

`o-completed-release-source-lw6i8hi9` и `...-y1q6d2oi` имеют одинаковые
source hashes и29 выбранных тестов; оба28PASS/1FAIL. Причина повторного запуска:
локальная замена assertion остановилась на ошибке, но отдельный следующий shell
процесс всё равно передал прежний файл и запустил тесты. Это ошибка исполнения,
не новая гипотеза. Она сохранена явно, не скрыта аудитом22 готовых сборок.

Редактирование и передача теперь выполняются одним Python orchestration: ошибка
редактирования останавливает последующие шаги. В текущий remote source helper
добавлен fingerprint205Rust source + точный набор тестов и stop для неизменённого
FAIL. Старые первичные receipts не переписывались; signatures хранятся sidecar.
Проверка стопа на заведомо прежнем source map: `...-v05x8q54/RESULT.json`,
`STOP_REPEATED_FAILED_SOURCE`, новых тестов0, до компиляции. Это self-test guard,
не новый тест ядра. Helper:
`/home/ubu/.cache/lay/development/remote-firefox-setup-j2he3qf6/o-completed-release-source-probe.py`.
Новый исправленный тест требует закрытый bridge для неподтверждённого позднего
callback; допустимые managed/native/UnknownStart positive сохраняют no-orphan
и exact token/scope. Runtime код этой коррекцией тестов не менялся.

### 2026-10-13T19:40:35Z: O-R2 focused29 GREEN

29/29 PASS, 16.580660436s, source proof_signature
`605223c4c965ebf5cc94a386237f614601cfe743988b94e71a9d22348f1ba7ac`, primary
`o-completed-release-source-49f0u4qb/RESULT.json` SHA-256 `54110681c04193e4a2704f885199b166dee6b259cf9240cd21c4e79510d0a17e`.
Все8 ранее упавших стабильных test IDs O-R1 включены, исходные guard tests
не изменены. Допустимые managed/native и UnknownStart сохраняют точный token,
scope, epoch и не оставляют orphan; неподтверждённый post-Reset callback сохраняет
закрытый bridge. Fetch9files включает7 форматированныхRust + manifest3143 и
known_failures; все205Rust SHA совпадают. Общий affected source scope pending;
review/native не начинать до его PASS.

### 2026-10-13T19:46:44Z: O-R2 full development GREEN

1100 selected/executed/PASS,0FAIL,259.9s. Receipt `/home/ubu/.cache/lay/development/run-5erfwyt3/RESULT.json`,
remote immutable `/home/e/projects/lay-development-runner/run-CdwrRH`, source archive
`f30df6eefaee323eff17e0ec81acae2ec8fe7ffa93efbe7d90284870aa1f20de`. Same eight explicit targets as N; FMT PASS.
This is the full recorded affected development scope, not entire-repository
release acceptance. All former O-R1 source regressions remain in the gate.
Current replacement review1/2 starts only now; no native build/acceptance yet.

### 2026-10-13T20:03:16Z: review1 proof issue fixed, production preserved

Review1/2 score8, no confirmed production defect. Authentication negatives used
the same serial as their positive and could be masked by duplicate rejection;
now each row has its own complete HeaderKey. Focused29 refreshed PASS in
`o-completed-release-source-qx3flqni/RESULT.json`. Dirty completion original
assertions now run observer-first and handler-first; a new actual queued
Reset-before-release/handler-first schedule proves old token stays revoked and
late completion leaves no orphan or restored word. Focused30/30 PASS,
16.818284944s, `o-completed-release-source-esjw0_ey/RESULT.json`,
SHA `1cd6b66ba80eaf10b45bcfc5a255d4993ff0eca9b9795de64145a4a9bba9504f`, all205Rust match. Four production files unchanged.
General source refresh needed for new fixture set, then final review2/2.
Previous1100 source receipt worker time is251.6305s;259.9s was controller wall,
not worker latency. Neither is native response time.
Dependency: current zbus async-io observer and callback share the same single
executor thread and ingress apply/refresh/publish has no await. Cross-owner
replacement still has store-level proof; actual Reset schedule now added.
Physical matrices/speed/private-write proofs remain NOT_TESTED for O-R2.

## O-R2 supplemental full1101: fixture FIFO failure retained

`run-9fun3w7c/RESULT.json` / immutable remote `run-bFjOId`:1101 executed,
1099PASS/2FAIL, controller255.2s. The new lifecycle test expected reply92501
but got the earlier92500; handler-first dirty-preedit expected an Error reply
but received its genuine Signal first. Both are raw test-peer FIFO assumptions,
not a demonstrated runtime defect. Focused30PASS does not override fullFAIL.
The new release helper now uses the established serial-aware next_peer_message
filter via the normal output collector; it does not synchronously consume one
raw message or discard signals. Lifecycle and new post-Reset fixture use the
same reader. Production4files untouched, no build/native yet.

### 2026-10-13T20:19:12Z: refreshed source GREEN with unchanged daemon proof reused

After the fixture FIFO repair, focused30/30 PASS16.868418486s in
`o-completed-release-source-0omlkfgl/RESULT.json`, SHA
`eb01a2cdb243ec395c1bfcc17a943e346cca2adf65194abc96278ef03989bf31`.
Fresh full IME bin and six integration contracts:828/828 PASS, worker
104.381220s, local `/home/ubu/.cache/lay/development/run-1t0exf6o/RESULT.json`, immutable remote
`/home/e/projects/lay-development-runner/run-T9RNFT`, archive
`c08324a27694bf79ab515f4cf20b8d61a9ca4f94384168839f55fa1b18d6f483`. Daemon273 passed in the prior eight-target
`run-bFjOId`; its relevant source/dependencies/configuration are unchanged.
Binding receipt: `O-R2-daemon-reuse-binding.json` in current evidence cache.
Combined recorded affected scope is fresh828 plus reused273; this is not a
fresh1101 run or entire-repository release acceptance. Prior full1099PASS/2FAIL
remains FAIL on disk. Four production files unchanged since first full1100GREEN.
Review2/2 now permitted; build/native are pending, N private runtime unchanged,
GUIOFF, laptop unchanged. TD121 remains OPEN.

2026-10-13T20:25:34Z — O-R2 final independent code review2/2:9/10, no blocking production
defect, build/native permitted. Reviewer `/root/completed_release_final_code_review_20261013`
independently verified immutable source and reused daemon identity. Owner generation
replacement remains store-level proof; actual Reset-before-late-ingress tested.
Fresh AST/architecture PASS46.5811s, `O-R2-graph-1791922806460946036.log`,
4 graph/receipt artifacts fetched and hash-verified. Runtime Rust unchanged.
New native/latency/private-writer proof NOT_TESTED. Next immutable build from
`run-T9RNFT`, then private staging/original browser actors; TD121 OPEN.

2026-10-13T20:32:43Z — O-R2 immutable build PASS183.408107s from current
`run-T9RNFT`; IME `c99b31aaa4a37309bc56747c1edcfc5bad5004c010d76da054d0868ab566acce`, daemon
`6e67bfaed56891dec88cc0520a819ac9954a572ac360b2da187c811346be88aa`. Staged only in remote private RUN,
verified N backup `staged-before-O-text-inert-release-1791923355841483469`.
Original actor hashes unchanged. Fresh finite seat-z/owned keyboard ACL and
zero-key PID/tick/executable/CG attachment proof validated; original Firefox
051-059 queued216 required. Scope `0::/user.slice/user-1000.slice/user@1000.service/app.slice/lay-verify-1000-2527394.scope`.
Native semantic verdict, speeds and O private writer proof pending; no promotion,
laptop unchanged, TD121 OPEN. Controller seat activation initially used an
unsupported verb; corrected to `loginctl activate` before GUI/input. No source
edit/build/test repeat occurred because of that controller error.

2026-10-13T20:46:23Z — O-R2 Firefox full native FAIL retained. Standard59PASS/1FAIL,
normal20PASS/private19PASS1FAIL(active field1)/fullscreen20PASS; panel6PASS.
Functional150:87PASS/44FAIL/19BLOCKED, no NOT_TESTED. Source bindings and original
actors unchanged. Primary `/home/ubu/.cache/lay/development/remote-firefox-setup-j2he3qf6/nativec99b3-firefox-proof.json`; raw
`/home/e/projects/lay-development-runner/firefox-legacy-native-20261012.6onP5c/nativec99b3-firefox-raw-1791924302611571081/RAW.json`. DS414/420 measured, median105.260/p95118.295/max133.731ms;
functional DS101/210 and Space37/50; Tab0/10/selection0/10. Clock evdev write-return
to exact AT-SPI target; kernel release NOT_MEASURED. Scope2527394 closed, all bound
PID/ticks gone, original VT1 restored before assembly. New private1 failed reverse
after probe removal; trace Reset reason missing_predecessor_or_post_reset_token,
live bridge token but manual authority false. This is measured sequence, not yet
a source causal root. Preserve O-R2 and preceding source fixes; do not repeat O-R1
or weaken authority. Next Chrome same exact bytes, then source causal proof for
remaining native sequence. No promotion/commit/push/laptop change; TD121 OPEN.

2026-10-13T21:02:42Z — O-R2 Chrome full native FAIL retained:58PASS/2BLOCKED standard,
private20PASS, panel6PASS; functional105PASS/33FAIL/12BLOCKED. Blocked seed fields
normal/fullscreen Space are field2 in both O-R2 and N; all216 case statuses match N.
DS406/420 measured median91.789/p95109.825/max124.717ms; functional DS138/210,
Space38/50, Tab0/10, selection10/10. Clock evdev write-return→exact AT-SPI target,
kernel release NOT_MEASURED. Primary `/home/ubu/.cache/lay/development/remote-firefox-setup-j2he3qf6/nativec99b3-chrome-proof.json`,
raw `/home/e/projects/lay-development-runner/firefox-legacy-native-20261012.6onP5c/nativec99b3-chrome-raw-1791925219090151196/RAW.json`. Actual private20/20 and3visited writer-root files
unchanged; enabled public positive recent_actions.jsonl changed. Four quiescent
snapshots, no extra audit input; `O-chrome-private-sink-proof.json`, trace completeness
UNKNOWN. Owned scope2584204 closed/all5PIDticks gone21:00:18UTC; VT1 restored.
Both original216row browser matrices remain FAIL. Retain source30/fresh828+reused273
and two reviews(8,9) separately; they cannot close native authority/functional
failures. Next source causal reproduction of managed append/native backspace/Reset
missing predecessor using existing reducer/real headers, then minimal mechanism
repair. No whole-core rollback/broadPassive/newowner/route/guard weakening.
TD121 OPEN/no promotion/commit/push/laptop change.

2026-10-13T21:16:21Z — P causal RED:1FAIL/2controlsPASS, 13.272402s,
`/home/ubu/.cache/lay/development/remote-firefox-setup-j2he3qf6/P-source-probes/p-confirmed-release-source-d_r20233/RESULT.json`, all205 current Rust hashes match. New actual managed press,
Reset, exact full SSI, handler-first matched release reproduces lost authority;
observer-first control reaches expected state. Production O-R2 unchanged. First
live failure is release10896 refused_timeout plus later orphan Key, before the
Backspace symptom, correcting the earlier downstream-only diagnosis. Difference
from original simple O test: actual Reset/confirmed receipt between press/release.
Decision2026-10-14-confirmed-reset-owned-release.json:9/10 current-owner handled
zero-effect completion without unrelated local word-scope equality;6/10 rebind
word scope;2/10 longer timeout. Terminal scope guard, unpaired fences, current
owner/metadata/one-shot bounds preserved. Plan review1/2 pending; no runtime edit
or new native claim. Next strengthen exact same source case through Backspace,
then implement the minimal eligibility distinction after independent review.

2026-10-13T22:03:12Z — P33 source30PASS/3FAIL, `p-confirmed-release-source-48tbgg95/RESULT.json`,
16.826417s. New independent reducer unit proves word-only Reset clears admitted
field metadata; original pending confirmed receipt republishes it only on a later
manual request. One eligibility predicate cannot repair this alone. Decision P
now retains existing field attributes only in observed_word_reset; all other revoke
paths unchanged, None never becomes Some, word token/lineage still revoked. No
new owner/cache or input transport. Original observer-first may publish a new
token/scope; exact equality applies to the new handler-first retirement branch.
O-added unproved negative will assert actual manual NotHandled/no edit/learning
instead of using an IPC readiness refusal as a proxy for word authority; all
original canonical mutation/unpaired/lifecycle guards retained. Source repair
pending, both full nativeFAILs retained and no installation/build repeated.

2026-10-13T22:08:18.907214Z — P33 metadata repair31PASS/2FAIL, `p-confirmed-release-source-9v962y99/RESULT.json`, 16.850514s. Real Reset/SSI/release/Backspace/manual positive and reducer metadata unit now PASS. Both remaining negatives stop before exercising release: matching current token/UnknownStart scope is valid and is not word edit permission. Assert explicit UnknownStart plus retained actual ManualToggle NotHandled and exact state/no-output/no-learning instead. Production unchanged for this fixture-only correction; full source/native remain pending.

2026-10-13T22:09:57.874111Z — P33 fixture clarification32PASS/1FAIL, `p-confirmed-release-source-mnfo7dfm/RESULT.json`, 16.879893s. Original O unproved-word actual NotHandled passes. New no-SSI fixture hits existing legacy LocalPending mapping to Handled(current layout), not a text effect. New negative asserts actual RequireSurroundingText/pending flag, unchanged tail/epoch, UnknownStart/no exact handoff, no Commit/Delete/Forward or learning. Runtime outcome mapping untouched; third P33 current rerun10393 has changed source/test fingerprint. Native totals retained.

2026-10-13T22:10:34.881049Z — P focused33/33PASS,16.866132s, `/home/ubu/.cache/lay/development/remote-firefox-setup-j2he3qf6/P-source-probes/p-confirmed-release-source-5aj0pbhy/RESULT.json`, SHAd1ab1791552004dd1b8b2848dd0b24a09f989c58f115cd5722b4afdb7c22a958. All205 remote-formatted production/test Rust hashes verified against local. Actual handler-first and observer-first Reset/SSI/release/Backspace/manual positive; private/password/None metadata lifetime unit; original O30 controls and both actual word-authority negatives pass. Next fresh full eight targets (IME,daemon,six contracts), independent context-free P review2/2, graph, immutable build. O native321P78F33B retained; no P native claim, no promotion/laptop changes.

2026-10-13T22:16:36.687991Z — P fresh eight-target source1104/1104PASS, worker256.078348s/controller264.3s. Local `/home/ubu/.cache/lay/development/run-daqc9vwf/RESULT.json`, remote `/home/e/projects/lay-development-runner/run-qrlOJ6`, archive2c5e359a99d92b86a7c50c4a9032d59023d8bd112839020704814d2f13b908e3. All205 IME/daemon Rust hashes match local. No reused daemon denominator. Independent context-free P final code review2/2 running; no P build/native yet, previous nativeFAIL totals retained.

2026-10-13T22:18:37.210691Z — P final independent context-free code review2/2 CLOSED9/10, `/root/confirmed_reset_release_final_review_20261014`; no material findings. Existing full-token/auth/metadata/one-shot and terminal-scope fences retained; admitted metadata lifetime repair accepted; actual pending/no-edit and unproved NotHandled negatives accepted; original O controls unchanged. Source33/33 +fresh1104/1104PASS; P nativeNOT_TESTED and prior321P78F33B remain FAIL. Next remote final AST, immutable buildrun-qrlOJ6, private staging/finite seat, focused original Firefox private standard before full matrices.

2026-10-13T22:20:46.801269Z — P remote final AST PASS20.854183s, `/home/ubu/.cache/lay/development/remote-firefox-setup-j2he3qf6/P-final-graph-receipt.json`; all734 Rust files remain identical to immutable gate request; four generated graph artifacts fetched with SHA verification. Source33/fresh1104/reviews9,9 complete; immutable run-qrlOJ6 candidate build live session32723. GUIOFF; O binaries unchanged until explicit private staging after build. No local AST/tests/input/install, no native acceptance claim.

2026-10-13T22:27:55.785145Z — P immutable run-qrlOJ6 build183.471569s: IME48235f612f8ce4e2c1b19f9a2605ebe6d3f73efe7de3aa2652eeae2f6efca7ae/8242960B; daemon2feb97733e6ff4ac7ae56795248d7ca9e6f3bd4e3a4a29339741839cc544dfd0/9264592B. Private-only stage verified O backup `staged-before-P-confirmed-reset-release-1791930233861523463`. New finite seat-aa PID2875048/t12779527/session3050/VT8. One controller validation failed because loginctl needs repeated -p flags; same new seat verified/resumed, no GUI/input/build repeated. Owned key ACL immediate event11/PID2876945/t12783267/cg2876924. Setup002003005 allparent0/noheldkeys; P-firefox-zero-key.json all5PID/tick/exe/cg/actorpins sourcebound. Original actor private20 diagnostic910 LIVE actualexec9635/read-only monitor7603, source33+1104 and review9,9 complete. No P full matrix or promotion claim yet; prior O matrices retained.

2026-10-13T22:33:55.036992Z — P original-actor private diagnostic19PASS/1FAIL/20; `/home/ubu/.cache/lay/development/remote-firefox-setup-j2he3qf6/P-native-private-diagnostic/RESULT.json`, SHA8b6c1770c41ed84a4e9f0edc196378c9c5fde0778044483be09d265ebe349057. Old private active1 passes, but private Space8 burst allfiveFAIL while both ordinary gesturesPASS; no same-source full matrices launched. Native clock ordinary40/40 p50105.768/p95117.956/max119.609ms; burst95/100 p50108.678/p95117.061/max117.989ms, five missing samples excluded; kernelreleaseNOT_MEASURED. Matchedtrace1ee794df4feaf0a99a2517d3bf8138095b4f5a908720e797ef192d757882e17d lines2028–2035: actual owned Space NBSP acknowledged and exact SSI7 confirmed at epoch2344, then Reset9439 arms unconfirmed proof and clears snapshot; managed Space release9440 accepted observer-first, no orphan; first manual refused context_authority with bridgeTokenLIVE. Need new source causal test for confirmed owned boundary/late Reset/no newSSI; preserve P fixes and reject timeout/transport/broadPassive repair. Actual9635+monitor7603 CLOSED0, diagnostic parent0/noheldkeys; strongscope2876924empty/all5ticksGone/VT1 restored22:30:00UTC, SIGTERM15 after intentionalSTOP. Source33/fresh1104/reviews9,9 retained; PnativeFAIL/no promotion.

2026-10-13T22:33:55.036992Z — User requires the Firefox/Chrome/total pseudographic PASS/FAIL/BLOCKED/total table in EVERY Lay status. Rule added to AGENTS.md; use latest completed full receipts (currently O321P78F33B/432), display diagnostics/source gates separately, never replace full totals with partial runs.

2026-10-13T22:37:47.856794Z — Q causal plan after P diagnostic: existing owned closing-Space capture still requires literal ASCII Space despite independent exact acknowledged NBSP producer. New decision2026-10-14-acknowledged-owned-space-reset.json rates existing Unicode whitespace predicate9/10; new state4/10, normalization1/10 rejected, timeout/extra SSI3/10. No runtime edit. New source-only actual confirmed owned ASCII/NBSP Space followed by held-key Reset/no newSSI and both release orders; retain P33, original closing-Space14 and contradictory boundary19-case guards. No word/phrase/browser literal exception, transport/owner/SafetyGate change.

2026-10-13T22:39:47.262642Z — Q causal source RED37controlsPASS/1newFAIL/38,17.653259s, `/home/ubu/.cache/lay/development/remote-firefox-setup-j2he3qf6/Q-source-probes/q-acknowledged-space-source-baf1zjgq/RESULT.json`, SHA8a44eb0d3c3f742e8d4c0aea877f86bbba748f237c7d7be9f174e75f4202549e; all205 hashes match formattedlocal. Actual ASCII held-Space path passed; NBSP exact acknowledgedsnapshot becomesNone on matchedReset while ASCIIretained. No runtime edits yet. All P33 plus original one-shot/missing/select/cursor/foreign/navigation/content/other-separator Space guards pass. Independent Q plan review1/2 next, then minimal existing whitespace predicate if accepted. GUIOFF/VT1; Pprivatebytes unchanged/no new build/fullnative.

2026-10-13T22:44:32.335317Z — Q plan1/2 CLOSED9/10; production delta exactly existing closing-Space .ends_with(char::is_whitespace) plus explanation, no other runtime rule. Focused38/38PASS17.764434s, `/home/ubu/.cache/lay/development/remote-firefox-setup-j2he3qf6/Q-source-probes/q-acknowledged-space-source-x49x5hkq/RESULT.json`, SHA4548d4e7e6a30f8031ad38ac80d1aaf4f1990d07d8e8661891e7ad6d36e143f0; all205 formatted Rust hashes matchlocal. Actual ASCII/NBSP/empty+nonemptyprefix/bothreleaseorders/heldReset/no freshSSI/ManualDelegate positive passes; all P33 and original held-Space14/contradictory19 controls unchanged/pass. Next freshfull8-target gate, finalQreview2/2, AST/build/privateNative. Existing fullO432321P78F33B remainFAIL, P diagnostic19/20 retained; no Q native/build/promotion yet.

2026-10-13T22:50:25.959739Z — Q fresh eight-target1105/1105PASS, worker255.938386s/controller264.9s; `/home/ubu/.cache/lay/development/run-61bj0v7o/RESULT.json`, remote `/home/e/projects/lay-development-runner/run-cht3Zw`, archivee1f0e3864aeb3f1f9669270960b3b0119367ce2e130d17f5f7a5214a56adf8f5. All734Rust files match frozenrequest; no reused daemon/test counts. Context-free Q final review2/2 running, root production unchanged after source gate. Next final AST/immutablebuild/native; QnativeNOT_TESTED and both previous full matricesFAIL remain, no promotion.

2026-10-13T22:52:25.251774Z — Q final independent context-free code review2/2 CLOSED9/10, `/root/acknowledged_owned_space_final_review_20261014`; no material findings. Sole runtime predicate bounded by exact acknowledgement/heldkey/current snapshot/one-shot Reset; original adverse14/19 assertions unchanged. Source38/38/fresh1105/1105 and all734Rust hashbinding complete. Next finalremoteAST, immutable run-cht3Zw build, private stage/renewfiniteownedseat, originalprivate20 beforefull9browserjobs+speed. NativeQNOT_TESTED, priorfulltableunchanged/no promotion.

2026-10-13T22:56:27.366662Z — Q final remote AST PASS20.821410s, `/home/ubu/.cache/lay/development/remote-firefox-setup-j2he3qf6/Q-final-graph-receipt.json`; all734Rust remain identical to immutable run-cht3Zw. Four graph outputs fetched/SHAverified. Source38/full1105/reviews9,9 complete; private candidate immutable build16080LIVE. GUIOFF/VT1, Pprivatebytes unchanged pendingQstage. No localAST/tests/input/install or new native claims.

2026-10-13T23:00:26.868337Z — Q immutable buildrun-cht3Zw183.311931s: IME399ffa890002bc6ce55b869bc30138f77656574b55be173fd59c91d37f7a5291/8243216B, daemonc49c44cb7776051433fc7d77ad0d5ac4d5f78f04b402895ffe1363950c3e3d97/9264592B. Private stage Q backup verified `staged-before-Q-acknowledged-space-1791932221320869900`; original actors unchanged. Verifiedownedaasentinel stoppedafteroldGUIclosed; newfiniteab2979358/t12975486/session3113/VT8. Native actual74111 LIVEscope2980379, immediateownedkeyboardACL event11/PID2980400/t12977453. Setup002003005 all0/noheldkeys; zeroQ-fivePIDsource/CG/pinsPASSsetup only. Originalprivate20diagnostic911/monitor87364 LIVE, receiptfirefox-standard-private-fg01xqzw/RESULT.json; no fullQ/nativeacceptance claim. Source38/fresh1105/reviews9,9/AST734 unchanged complete; fulltablepriorO321P78F33B retained. NoheavyworkwhileGUI/no laptopinstall/input/RDP/promotion.

2026-10-13T23:02:52.488396Z — Q original private20 diagnostic20/20PASS; `/home/ubu/.cache/lay/development/remote-firefox-setup-j2he3qf6/Q-native-private-diagnostic/RESULT.json`, SHA6a200e40701f9adce1e1b0afa8b83392b5cc12b169c2210a7be9b0bdde5572de. Both prior private active1 and privateSpace8burst pass. Speedordinary40/40 p95120.396ms, burst100/100 p95118.921ms; kernelreleaseNOT_MEASURED. Parent9110/noheldkeys. SAMEownedGUI actual74111/scope2980379, all5 identities revalidated; original9 Firefox jobs051-059 nowqueued, full216 completion monitor72210LIVE/autostopafterall9parent0/noheldkeys. One controller filename lookup initially lacked leadingzero and queued zero jobs; corrected before enqueue, no native tests/build repeat. Diagnostic never aliasedas052. NoheavywhileGUI; primary432table remainsO321P78F33B until fullnewreceipt.


2026-10-13T23:24:26Z — Q full Firefox original9 complete; standard60P/panel6P/functional87P44F19B =153P44F19B/216. All216 stable IDs vs O:1newPASS private-active-field1,0lostPASS,215same. Immutable native399ff-firefox-proof.json SHA93af9b30e3faf40622f4c6bda2dee877b5766ec4b6eed1dd35ad762753b72de6; RAW native399ff-firefox-raw-1791933808802713777/RAW.json. Standard420/420 p95120.982ms, panel48/48 p95120.451ms, functional DS101/210 p95136.333ms/Space37/50 p9523.736ms/Tab0/10/selection0/10, missing remain explicit/kernelreleaseNOT_MEASURED. Diagnostic20/20 remains separately indexed. Actual74111+monitor72210 CLOSED0/ownedSTOP/SIG15; exactCGempty+fivePIDticksGone/VT8→1 receiptQ-firefox-closed.json at23:22:08UTC. No promotion; ChromeQNOT_TESTED, current mixedtable QFF/OChrome322P77F33B/432.

Collector primary-role scope fixed because Q diagnostic20 plus original3 standards produce four receipts. New assemble-scoped-candidate-browser-proof.py selects exact original9 jobs, validates parent0/noheldkeys/terminalCOMPLETE/receiptstatus/full dimensions/speed/actor+candidate pins, retains mapped diagnostic20 separately, rejects unknown same-candidate attempts and failed diagnostic evidence; never picks latest/best or discards a current attempt. Independent context-free infra reviews1/2 score6 and2/2 score8; three false-PASS gaps fixed, finalP2 historical-candidate binding fix applied after pass2/no third review. Remote10/10 fixture checks PASS Q-collector-fixture-checks.json, scopeCOLLECTOR_FIXTURES_ONLY_NOT_NATIVE_ACCEPTANCE: real216dimensions+diag40/100speed, terminalsave/cleanup/status mismatch, unknownkind, missing/duplicate diagcells, bad/nonfinite speed, older failedcandidate retained without blockingcurrent. First fixture harness glob omitted Q in diagnostic filename and failed before any collector scenario; corrected glob then10PASS, no native/code/source rerun. Next exactsameQbytes Chrome original100–108, publicpositive/privatewriter brackets, then stableID comparison before source work. Runtimeauthority/routes unchanged.


2026-10-13T23:38:07.284338+00:00 — QChrome native firstlaunch blocked sharedheavy lease beforeGUI/input; unrelated L3scope3068225 was not stopped/bypassed, resumed onlyafter lease becameavailable. Next startup actual68855/CG3079574 setup002003005/zeroSHApassed but native seat3113/VT8 Active=no/foregroundVT1 after priorFirefox restoration: allnormal20 BLOCKED exacttypedseed (340 emitted keys/firstfieldempty); private terminalBLOCKED/20NOT_TESTED. Monitor31077 CLOSED1 and ownedSTOP, actual68855CLOSED0, exactCGempty/all5ticksGone. Preserved complete attempt/archive Q-chrome-inactive-seat-attempt-1791934375202890764/ATTEMPT.json; receipt hashes0293af... and9e9129...; not erased or claimedas productregression/fullmatrix. Root preparation error: startup lacked ownedseat activation after restoringVT. Added controller require_active_seat beforeanyinput/everyoriginaljob, exactUID/PIDtick/loginctlActive+VT+Leader/foreground8. Remote no-input controls rejectinactive oldseat and acceptverifiedownednewseat, Q-chrome-active-seat-gate.json. Verifiedownedab stoppedonlyafterGUIempty; renewedfiniteac3093791/t13194367/session3181/VT8 onehour and actualActive=yes. Q-chrome-finite-seat.json properties are BEFOREactivation (Active=no); following positive gate records actualactivated seat. Source/actors/newprivatebytes unchanged. Freshactual9896/CG3094715, immediatekeyboardACL event11/owner3094736/t13196213; setup002003005all0/noheldkeys/QchromezerofivePIDpins. Originalfullmonitor92118 LIVE:100 normal19P1B, snapshotsnormal-before/after/private-beforequiescent3files, original101private running. NoheavywhileGUI. Fulltable remainsQFF/OChrome322P77F33B432 until original9 verified. Runtime/laptoproutes unchanged.

Read-only residual inspection: Firefox44F groups prefix7/undo10/cursor10/selection7/focus10;19B Tab10/prefix3/movedboundary3/selection3. Several seeds expose unaccepted suggestion after caret (e.g. fullприветное withtypedprefixпривет/caret6); focusreturnedcaret9 andselectionrawаж/caret1 are recorded observations, not root attribution. Hypothesis that output mistakenly uses PREEDIT_MODE_COMMIT is rejected by current source: preedit.rs31/632 already sends PREEDIT_MODE_CLEAR=0. No renderer or focus edit based on this hypothesis. Need actual first-transition/wire evidence after bothQmatrices before a new core experiment.


2026-10-13T23:49:40.941365+00:00 — QChrome original9 complete706.906s; primary169P33F14B/216 =standard58P2B+panel6P+functional105P33F12B. ALL216stable-ID verdicts matchO,0new/0lostPASS. Primarynative399ff-chrome-proof.json SHA73a88c47a24bff2d7d54cff2e9db09bec3f06865f8f23f5d74f926a0fd3d1365; RAWnative399ff-chrome-raw-1791935217299974275/RAW.json plus separateprimary9receiptretentionindex. Native9896/monitor92118CLOSED0/ownedSTOP/SIG15; CG3094715empty/all5ticksGone/VT8→1 at23:46:56.746492UTC. Privacy3visitedfilesunchanged+normalpositive recent_actions changed, Q-chrome-private-sink-proof.json PASS_VISITED_LAY_SINK_ROOTS, no universalcoverageclaim. Standard406/420p95113.861ms, panel48/48p95109.729; functionalDS138/210p95127.697,Space38/50p9523.970,Tab0/10,selection10/10p9521.469; allmissingexplicit/kernelreleaseNOT_MEASURED. BothfullQ432322P77F33B/0NT; across432 IDs1newPASS/0lostPASS; sourceQ38/fresh1105/reviews9,9 unchanged. Compactrepoevidence tech_debt/evidence/td121-Q-full-browser-native-2026-10-14.json; QdecisionFULL_BROWSER_NATIVE_FAIL_RETAINED_NO_PROMOTION. ResidualChrome33F undo9/cursor10/focus10/prefix1/spaceEN-RU1/spaceRU-EN1/cleanEN1;12B Tab10/undo1/movedboundary1, plusstandard2searchSpaceB. NextR-D0 diagnostic unchanged Q bytes/publicFirefoxfield1 original15plans+passive exactwire, after activeownedseat/zero setup; source edits onlyafter firstcausaltransition/plan. Routine512KiBtrace hasprefix_preserved=false so firstfield sourceattributionUNKNOWN. TD121OPEN/no publication/laptopchanges.


2026-10-14T00:02:09.383916+00:00 — R-D0 unchangedQbytes/publicFirefoxfield1/originalunmodified15plans/passiveexactIBuswire COMPLETE:9P5F1B, equals previousfullQfield1 statuses; NOTprimary216 replacement. WireR-public-functional-wire-9r0p60_3/WIRE.json, wire1698153B SHA5354a52c07cead7645253a87cede2cb5dfa2e874b5c2466cf60e6f90dc56de86; actorfirefox-functional-b38lucfg/RESULT.json, exactactor/bodypins76d329.../45ae44... preserved. Monitor89850parent9200/noheldkeys/ownedSTOP, actual38026CLOSED0, CG3151956empty/all5ticksGone/VT1restore R-firefox-closed.json. FivePIDsourcebinding R-firefox-zero-key.json, ownedseat3181Activeyesverifiedbeforeallkeys; immediateevent11ACL owner3151977/t13300102/setup002003005 all0. No source/core changes.

Measured focus: initialclientSSI textпривет/caret6, engineghostное/CLEAR0 atserial1569; clientReset2214/engine1481 -> engineclear empty,false,CLEAR0 signal1571 -> inputcontext forwards clears2785–2789. Focusreturns clientSSI2224 textприветное/caret9 beforeengineSSI1495; no engineCommitText ofное in this captured focus interval, only following typedы commit1584. Thus client text changes before new typedletter despite actualclear; further frontend/presentation root unknown, separate from cursor.

Measured cursor: sourcealreadyUnknownStart BEFORE1350 (1347/1349 settlements), exact locally observed typedtail6/currentSSI6/caret6 supplied prior suffix-manual eligibility. FirstArrow1350 clears tail6→0 via close_committed_tail_field, ghostcleared; nativearrowReset/SSIs walkcarets5/4/3, BackspaceSSI5caret2, managedиCommit+SSI6caret3 butlocalbufferonly1. End1370 -> actualSSI6/caret6 serial1371 BEFORE Reset1372, resetrejectmissingpredecessor token/clears snapshot, release1373 accepted. ActualManualRPC1257 context_authority refused, bridgealive/manualallowedfalse/refreshfalse. Cannot fix by treatingarbitraryUnknownStart asKnown or onlyretainingEndSnapshot: first owned-suffix provenance lost earlier. R-wire-focused-excerpts.json preserves all matchingclockedwire forfocus/cursor/prefix; routine512KiBtrace now containscursor tail and correspondingserials. New independentcontext-free cursorplanreview1/2 /root/cursor_authority_plan_review_20261014 READONLY running; directionsboundedexisting-editproof versusunsafe genericrefresh/no-opworkaround. No production changes until causal RED+defensible minimalplan. FullQtable still322P77F33B432; source38/1105/reviewsQ9,9 complete/no promotion.


2026-10-14T00:11:12.886616+00:00 — R cursor plan independentreview1/2 CLOSED8/10; measured initialUnknownStart plus confirmedreset exacttail6, firstArrow clearslocaltail6. Revised recommended8 typed inert manual-only editing phase in existing single receipt slot, one expected effect/currentactualsnapshot, sameowner/purpose guards, no general confirmation/Space/Tab promotion. GenericUnknown SSI1/10, End-onlysnapshot3/10, extraSpace2/10 rejected. Decision2026-10-14-manual-cursor-edit-provenance.json. Root test-only native_cursor_edit.rs/include in existing residual harness prepared: actual3Left/Reset/SSI/release, nativeBackspace, managedinsert(c or z)/Reset/SSI/release, End/SSI-before-lateReset/release, realManualToggleV3/exactVisibleTailV2; prefix/empty/bothreleaseorders; oldtokeninvalid/noKnownStart/no extra text/learning. OriginalQ38 controls selected; source R39 causalRED next remote, productionQ unchanged. Fulltable322P77F33B unchanged; GUIOFF/VT1.


2026-10-14T07:31:41.532363+00:00 — W configured repeated closed-source54 and delayed final Backspace-release54 BOTH PASS (23.965s/23.910s; receipts /home/ubu/.cache/lay/development/remote-firefox-setup-j2he3qf6/W-configured54-pass-RESULT.json and /home/ubu/.cache/lay/development/remote-firefox-setup-j2he3qf6/W-delayed-backspace-release54-pass-RESULT.json). Source fixture does not reproduce V fullscreenSpace9 native failure; first loss UNKNOWN. Next WD diagnostic adds only numeric/boolean metadata after existing exact-replay rejection (scope pair/current, command modifier, unexpected key or failed mirror transition), before scope revocation, no successful callback logging. No functional predicate/route/owner/timer/model changes. Decision2026-10-14-native-replay-failure-only-diagnostics; one originalstandard60 normal/private/fullscreen sequence after fresh scoped gate/AST/private byte proof, no passivewire. Partial diagnostic never replaces fullV339P60F33B432. Independent review NOT RUN threadlimit/no score; TD121OPEN.


2026-10-14T07:38:59.124904+00:00 — WD fresh scoped gate843/843 PASS (IME792+sixcontracts51; worker116.301s/controller124.8s), immutable remote run-P4dPQe/archivebec4e26e89e7271e5288992bf2d6c43b711c3e5281fb85de8a11eece70aa4847; all737Rust hashes verified against local source. Remote AST20.797s, all4 emitted artifacts fetched/hashverified. Root audit: remove only failure-only metadata helper+five calls and production tail_memory exactly equals V frozen source. No predicate/positive path/owner/timer/route/model fit changes. Private immutable build still running actual84885; no new native result yet. User says current сборка good; local installed bytes not established by that remark; preserve local runtime. Latestcomplete full432 remains339P60F33B; diagnostic-only/no publication/taskOPEN.


2026-10-14T07:49:51.288202+00:00 — WD standard60 diagnostic COMPLETE60PASS0FAIL0BLOCKED, original3parents0/noheldkeys, actual3321/driver32363 CLOSED0; exact5originalticksGone/CGempty/VT8→1. Archive /home/ubu/.cache/lay/development/remote-firefox-setup-j2he3qf6/WD-standard-diagnostic-1791963994703664507/RESULT.json SHA66f029589a4c787c1938e184dae8b4173ad0fddb5cff04bb85cd3b12eab7beb6; all80files copied/hashverified. ZERO ibus_exact_replay_rejection records. V fullscreenSpace9 did not reproduce, causeUNKNOWN/no functional fix claimed, no unchanged repeat. Ordinary p95normal121.472/private119.020/full120.476ms all120/120; burst118.723/116.910/115.019 all300/300; kernelreleaseNOTMEASURED. Next WE one original057 field5,6 functional30 using unchanged gatedWDbytes, no passivewire; snapshot existing capped trace once at observed row completion (bounded30x512KiB) to retain failure-window callbacks missing in V. Originalactor/inputcadence untouched. Same decision extended; no source/build repeat, authority/route/model/laptop changes. Latest complete full432 still339P60F33B; taskOPEN/reviewNOTRUN/threadlimit/no publication.


2026-10-14T08:06:33.894355+00:00 — WE original057 fixed30 COMPLETE20PASS8FAIL2BLOCKED/64.105s; captured30 row traces/no errors, copied/hashverified41files; resultSHA9c3f7e23ac6a8db002b9b2ff1b76de05a35afd5cd80c2385dad9a715e54abf93 /home/ubu/.cache/lay/development/remote-firefox-setup-j2he3qf6/WE-functional-diagnostic-1791964524400800201/RESULT.json. actual48580/driver47466CLOSED0/WE5Gone/CGempty/VT1/oneparent0/noheldkeys. ComparedV stable30 two Space losses nowPASS, allotherstatuses equalV; no functional code change, no complete matrix promotion. Prefix5 actualreplacement succeeds then layout_sync false; nextz decodedя onoldRUowner. Prefix6 layoutsyncfalse thensecondRPCbridge_admission refusal. Both start actual printable_legacy_preedit, so regular-preedit-mode speculation ruledout. First refusing ownership/dispatch predicateUNKNOWN; do not alter layout guards. Selection actualCommitа/SSI1 thenvisibleuncommittedж may be observationissue, verdictunchanged. Undo leaseNotReady3458us thenoldworksuperseded, no model/ranker/deadline fix justified yet. Allfivefailurewindows retained withactualcallbackanchors11/19/15/15/8 andcoverageUNKNOWN. LatestfullV432339P60F33B unchanged; taskOPEN/reviewNOTRUN/threadlimit.


2026-10-14T08:09:09.265446+00:00 — WF diagnostic planned from measured WEprefix loss, not an assumed functional fix. Existing owned-preedit false result with nonempty composition emits only guard booleans; existing failed blocking layout dispatch emits its enum. No successful callback logging, guard/policy/route/timer/owner/model edits. Decision2026-10-14-owned-preedit-manual-refusal-diagnostics; fresh scoped843/AST/immutableprivate build then original057fixed30/case-end trace capture once. First rejecting guard stillUNKNOWN; no broadened mode scope or optimistic decoder. GUIclosed/WEexact5Gone/CGempty/VT1. Latestfull432339P60F33B/taskOPEN/reviewNOTRUNthreadlimit.


2026-10-14T08:26:49.865780+00:00 — WF original05730 reproduces exactlyWE20P8F2B/64.100s; all30rowtraces+41files retained/hashverified, resultSHAf521df86c405005cd1744ad8cf69ae7940979f60b45ae4a8dff14e8f39babab4. actual56847/driver94588closed0/WF5Gone/CGempty/VT1. Measured prefix5/6 solefalse owned-preedit condition exact_refresh=true; all17otherguardfacts valid (legacy owner/mirror/visible/currenttoken/scope, no selection/pending/etc). Blocking dispatch ContextConflict switchedfalse follows. First producer mistake localized; no functional edit yet. WG newdecision9/10 bounded birth-witness-based admission under existing owner/InputMode transaction; broadveto removal2/10/backgroundfallback6/10. Test-only builder preserves unmarked41default and supplies markedcap from birth, real prefix/SSI/typedword,8manualgestures/decoder/Backspace/sameowner+snapshot. Nextcausal57 RED incl54controls+existingknownOwnedpositive+16refusalguards. Latestfull432339P60F33B unchanged/taskOPEN/reviewNOTRUN/threadlimit/no promotion.


2026-10-14T08:50:46.048180+00:00 — WG focused57GREEN24.179s sig4439f4e... after explicit bounded marked-prefix proof and actual localBS restoration. Focused58GREEN24.732s sigce143f87... adds12refusal classes; allold16guards (including raw marker flip) remain. New marked8gestures preserve owner/path/widgetprefix snapshot/revision and assert nextz decoder pluslocalBS, noCommit/Delete. Actual source afterprobe witness loss wasgenericBSrevocation; boundedexisting prepare/finish localBS nowretains samebirthwitness afterverifiedonechar/oneepoch, currentprefix/token/identity. Final pending-display-on-localerase check and authorize-before-negative mutation are in fresh scopedgate80258 currentlyLIVE; no nativeWG yet. Reviewdiff /home/ubu/.cache/lay/development/remote-firefox-setup-j2he3qf6/WG-review-diff.patch relativeimmutableWF source contains only3WGsource files. LatestfullV339P60F33B unchanged; TD121OPEN/no install/promote/push/refit.


2026-10-14T08:54:27.645506+00:00 — WG first fullscoped845=844PASS1FAIL (all794IMEPASS including final pending-displayBS/new12refusals). Failure canonical typed_authority_mints_have_only_named_runtime_callers refused new fixture direct plan_ime_manual_toggle_edit. Production unchanged; no weakening of guard/canon/test. Corrected fixture calls actual named runtime manual producer, then asserts no new InputMode/visible owned-preedit effects when birthproof invalid; older independently proved routes permitted. Fresh845 source required beforebuild/native. Context-free WG reviewer spawn failed agentthreadlimit/NOT_RUN/noscore. LatestfullV339P60F33B/taskOPEN.


2026-10-14T09:01:32.977661+00:00 — WG fresh scoped845/845PASS (IME794+sixcontracts51; worker116.261s/controller124.6s), run-LItDwT/localrun-r32mz8sc/archive0d08b7e50a1996d3e17759a80686bb904a59b01a53fdb0dae5ed5256c22ea4f9/all737Rust verified. New marked-owned positive8gestures includes pending-display-beforelocalBS; 12negatives invoke named runtime producer and prove no owned InputMode/preedit transaction. Allold16guardadverses remain; no fixture mint/canon weakening. Sourcegate includes latest negativeordering and local erasure proof-only retention (no manual authority while pending). Currentbuild97576LIVE AST+immutable private binary; no nativeWG acceptanceyet. Full original9/browser protocol next, all432 stable IDs/speeds/privacy retained, no rowwatcher. ContextfreeWGreviewNOTRUN agentthreadlimit/noscore/taskOPEN; latestfullV339P60F33B/nopublish/laptopinstall/refit.


2026-10-14T09:40:16.451011+00:00 — WG Chrome zero-key setup twice rejected before any behavior input, bothclosed exact5Gone/CGempty/VT1. Root cause second capturedFocus payload unchanged (clientgnome-shell, lay_ime_ru/1), only wire serial35→47/time changed. Existing zero setup dictionary equality conflates repeated observational header with identity. Protocol decision2026-10-14-browser-zero-key-focus-identity: compare nonemptyclient+engine path and retain bothheaders; existingfreshIC/globalRU/IMEPIDticksha/focuswindow/kernelpeer/portal rechecks unchanged. Add pure helper+five semantic tests (header-only change allowed; missing/malformed/client/path changes refused), revise005pin only; no sourceRuntime/model/deliveryRoute/binary change. CurrentWGfullFF164P33F19B/216 (5Vgains1lostSpaceRuEn1), all9primary+RAWmanifest copied/hashverified, partialnotfulltable. Original432baselineV339P60F33B staysuntilChrome216. ReviewerNOTRUNthreadlimit/TD121OPEN.


2026-10-14T09:49:43.073719+00:00 — WG fullFirefox216 COMPLETE164PASS33FAIL19BLOCKED/712.986s; standard60PASS/panel6PASS/functional98P33F19B. V5gains1lostPASS(spaceRuEn1)/U3gains1loss, all10FFcursorPASS. Exact5Gone/CGempty/VT1/original9parents0/noheldkeys; all9primary+RAWmanifest locallyhashverified. Std420/420p95118.485ms/panel48/48p95118.855/functionalDS124/210p95137.233/Space36/50p9525.347/selection1/10/Tab0, missingexplicit/kernelreleaseNOTMEASURED. Oddprefix1/5stillfirsttargetFAIL, even2/6PASS; nextcauseUNKNOWN/no speculativefix/rollback/promotion.
Chrome original behavior neverstarted in two failedzero setups; bothclosedexact5Gone/CGempty/VT1. Measured Focus payloadclient+enginepathunchanged, ONLYwireheader serial/time changed. Protocolhelper5unitPASS/new005SHA dde6db54.../helperbd2bc86f...; allotherPIDticksha/window/IC/RU/kernelpeer/portalchecksunchanged. PostPythonAST20.705s/all4hashverified/all737Rustunchanged/no rebuild. Newownedfiniteaj855015/t16866706/session4022expires1791974636. Chrome ZEROreadyPASS now original100–108 LIVE actual86410/driver55556/WGfourprivacybrackets. Original normal20=19PASS1BLOCKED; privatequeued. No sourceedit/heavywhileGUI. Latest completeBOTHtable remainsV339P60F33B untilcurrentChrome216 complete. ReviewNOTRUNthreadlimit/TD121OPEN/no publish/laptopinstall/model fit.


2026-10-14T10:02:47.127755+00:00 — WG BOTH original216/browser COMPLETE432=343PASS56FAIL33BLOCKED (FF164/33/19, Chrome179/23/14). Chrome696.994s actual86410/driver55556closed0/exact5Gone/CGempty/VT1/9parents0/noheldkeys. Chrome ALL216statuses exactly V/U; FFV5gains1loss =>net4morePASS/fewerFAIL, NOT complete/promotion due56F/33B+lostFFspaceRuEn1+reviewNOTRUN. All20cursorPASS. BothRAWmanifests andALL18primaryJSON locallyhashverified WG-both-primary-retention-verified.json; FFproofSHAf8e190.../ChromeSHAba7c69c0235ae4a35a4edf61bb0e265130a2b1d972b21c13ab50941e31796a02. WGChromeRAW1791971933462849002 manifest1dc10ee... retained. StdChrome406/420p95110.127ms/panel48p95108.761/functionalDS158of210p95124.664/Space38of50p9525.409/selection10p9523.074/Tab0; FFspeedspriorentry. Missing explicit/kernelreleaseNOTMEASURED/no statisticalimprovementclaim. WGprivate3filesunchanged/publicpositive recent_actions changed; notfullaudit/modeltraining. Compactrepoevidence tech_debt/evidence/td121-WG-full-browser-native-2026-10-14.json. Nextfirst newfailedprefix-bindingpredicate +SpaceRuEn1firstauthorityloss unknown; no wordbranch/guardweakening/unchangedrandomretry/blindrollback. Latestcurrenttable343P56F33B; taskOPEN/no publish/laptopinstall/refit.


2026-10-14T10:07:50.009976+00:00 — WH failure-only diagnostic planned from completeWG343P56F33B432. Oddprefixfirsttarget failures unresolved, currentcappedfunctional traces don'testablishnewbindingpredicate. Trace birthcap/focus/lease/layout/revision/currentfullprefix andmode fields onlyafter originalmanual-entry orpost-publication refusal. No new positive-callback logging, authority/predicate/expiry/effectorder changes. Fresh scoped845/AST/privatebuildthenoriginal055fields1,2fixed30once+row-endtracecapture, nopassivewire/cadencechange/wordorfieldruntimeexceptions. Decision2026-10-14-marked-preedit-birth-binding-failure-diagnostic. FFSpaceRuEn1 firstauthorityloss remainsseparateUNKNOWN; no ranker/model/deadlinefix. AllGUIclosed/VT1/taskOPEN/reviewNOTRUN.


2026-10-14T10:28:06.548781+00:00 — WH diagnostic original055 fields1,2 complete20PASS8FAIL2BLOCKED/30 in65.101s; all41artifacthashes verified, receiptSHA2bf4a45c..., actual90438/driver30713closed0, exact5ticksGone/CGempty/originalparent0/noheldkeys/VT1. Metadata-only bytes; no full432replacement (WG343P56F33B remains). First field1prefix RPC refuses context_authority before owned manual branch, bridge token live, manualToggle/refresh false, UnknownStart. No new binding-refusal records; deeper cause UNKNOWN. Field2prefix firstownedmode/nextz/localBS pass, secondgesture sole birth-binding failure revision10→12; focus/lease/layout614/fullunchangedclientprefix/cursor/unselected/currentowner allvalid. Must reproduce actual duplicate SSI path before bounded inertbirthwitness refresh; retain stale-revision-without-real-receipt refusal and all priorguards. No dropping revisionguard/counterfeitSSI/client-specific code/deadline/model change. Diagnostic SpaceRuEn1 FAIL→PASS andprefix2 PASS→FAIL with identicalfunctional logic show timing sensitivity, not fixes. TD121OPEN/reviewNOTRUN/threadlimit/no score/no promotion.


2026-10-14T10:34:38.798112+00:00 — WI real repeated unselected prefix SSI causal RED59=58PASS1FAIL/25.088s sig627929b3... remote wi-marked-refresh-source-l2b76iqx; actual callback advances revision while same owned birth remains pinned. Earlier fixture private-field compile error is not causal RED and was corrected without exposing private fields. Minimal production observe_external_surrounding_text refreshes same birth armed_revision only if pre-ingress full marked-owned proof/currenttoken valid, previous actualsnapshot exists, new fullsnapshot exactly same, revision consecutive. No new authority/snapshot/payload/route; all old guards unchanged. Expanded positive8modegestures/probeBS/16actualreceipts and new14adverse classes now focused60 running. Field1 earlier UnknownStart context_authority remainsseparateUNKNOWN. LatestcompleteWG343P56F33B432; TD121OPEN/reviewNOTRUN/threadlimit/no nativeWIyet.


2026-10-14T10:41:00.717849+00:00 — WI fullaffected847PASS/worker118.484s/controller129.2s/sourcearchivea208be55.../all737Rustmatch, remote run-QaJV76/localrun-ow_hrxg8; source-only, nativeNOTTESTED. Before expensive candidate build, investigate separate WHprefix1 failure with source-only WJ causal actual closingSpace→Reset→prefixSSI→pairedrelease→ownedword ordering. Native KnownStart2002 becomesUnknownStart2016, fullownedword starts2025, RPCcontext_authority2208. Key preedit_chars means suggestion suffix, not compositionword; no partially-owned inference. Test-only optionalhelper keeps alloldbehaviordefaultfalse; no furtherproductionchangeyet. WJ61 focused running, causal validity notassumed. LatestcompleteWG343P56F33B unchanged; reviewNOTRUNthreadlimit/TD121OPEN.


2026-10-14T10:43:14.826506+00:00 — WJ source causal61=60PASS1FAIL/25.090s sig e13e46a7..., actual closingSpaceResetSSIrelease reproduced UnknownStart with wholeownedlegacyword/currenttoken/fullbirthsnapshot; failure is explicit marked-boundary KnownStart requirement. Minimal manual-only proof now uses exactbirth/fullprefix/revision/metadata/currentowner and permits unchangedUnknownStart; no promotion, no genericedit/Space/Tab/SafetyGate change. New sourcepositive8gestures+decoder/localBS/actualSSI asserts UnknownStart and genericmanual refusal remain; WI14adverses expanded tobothreal lineages (28variants), oldknown/unmarked controls retained. WJ61GREEN running. LatestcompleteWG343P56F33B432; source ordiagnosticnotnativeacceptance; independentreviewNOTRUNthreadlimit(no additionalspawnattempt), taskOPEN/no publication.


2026-10-14T10:58:38.956923+00:00 — WJ broad KnownStart removal FAILED full848 (844P4F) beforebuild; oldC06 append/manual-denial, staleSSI, externalepoch andowned-return epoch guards remained unchanged. Refined UnknownStart grantrequires retired handled input; focused65 (old4 failures included) passed. Additional bothlineage released-epoch adverse thencausalRED65=64P1F sig7333505b... exposed latent markedmanual missingepoch binding. Barebirth+chars guard failed three positives after actualpublish_tail_handoff increments epoch1 (signature53b57...); itwasnot promoted/retriedunchanged. Final bounded scalar manual_tail_epoch_offset inexistingbirthslot updatesONLYafter sameverifiedproducer exactone-step actualhandoff (androllbackhandoff); Space/deletion do notreadit; existing localBSrebasesoldstart andzerosmanualoffset. Focused65 passed sig052dfc..., finalheld press/release assertions+failure-onlyepoch/retired metadata passed65 sigac996d..., no new positivecallbacklogs. Nowextendexisting outputfailure proof toallthreeactualmodes (oldunmarked,markedKnown,markedUnknown), including propertyfail rollback+nextsuccess; finalfocused66 running. No newowner/SSI/knownstart/timer/model/deliveryroute, nativeWI/WJNOTTESTED, fullWG343P56F33B unchanged, reviewNOTRUNthreadlimit/TD121OPEN.


2026-10-14T11:02:05.493161+00:00 — WJ FINALfreshscoped848/848PASS: IME797+sixcontracts51, worker116.147s/controller124.700s, localrun-ab157r6z/RESULT.json remote run-w03rUs, archiveb46f7c455b4aa50f6dae7cd72df68e8f0b91feccb3b43c5d4a3c22355f089d45/all737Rusthashesmatch. Finalfocused66PASS26.469s/eae59bd0..., old4fullfailures preserved andpass; heldmutableinput/releasedepochfault/currentowner/15outputfailure variants andmodefailure rollback+nextsuccess proved. Private immutablebuild+AST requested; no GUI until actualbuild0/4ASThashesfetched/oldWHscopeempty. Stockroutes/knownstart/snapshot authority unchanged. Sourceproof isnot nativeacceptance; fullWG343P56F33B table remainsuntilneworiginalboth432. TD121OPEN/reviewNOTRUNagentthreadlimit/noscore; no laptopruntime/publication/refit.


2026-10-14T11:04:11.461242+00:00 — USER_STEERING: user requests finish build now and explanation, stops further improvement/native-test phase. Complete alreadyrunning immutableWJ build48504 from final848PASS source; produce reviewable paired artifacts and manifest. No new functional tests, browser input, runtimeinstall/restarts, commit/push/tag/release inferred from this build instruction. LatestcompleteWG432343P56F33B remains previousbytes; WJchangedbyte nativeNOT_TESTED/reviewNOTRUN preserved, TD121OPEN. Source3causalmechanisms fixed underoriginalguards; remaining native failures mixed/firstcausepartlyUNKNOWN, no browserceilingsclaim.


2026-10-14T11:14:04.842486+00:00 — USER_STEERING_ADD_NATIVE_APPS: exact completedWJ build84f851/084169 requested in Telegram, WezTerm and other GTK/Qt windows, with native IME appearance and speed. This revokes only earlier no-further-window-run scope; productionsource staysfrozen848PASS. No remote Telegram orWezTerm installed/authenticated; hostGtk3/gedit/textEditor/GNOMETerm exist, privateguest lacksQt/editors. Provision officialapp packages onremote withinowned profiles, reuse existingfiniteGNOME/IBus/keyboardfixture, no RDP/laptopruntime/IMEroutechanges. Telegramcomposerrequiresuserauth; asyncquestionpending, no message send/Enter/clearinguserdrafts. Newappresults mustseparate currentbytes frompriorWG full343P56F33B browsermatrix; no universalacceptance/platformceiling claims.


2026-10-14T11:42:47.104472+00:00 — WJ otherclient native scope LIVE54582/driver50662, zero attachment PASS exactIME84f851/daemon084169/stockGNOME50/IBus. Newfiniteak1228560/t17536201/session4242VT8. Guardedsameprivatecontext, source/scriptsfrozen6hashes WJ-native-app-protocol.json, 88scenarios290latencies planned; realGtk2/Qt3fields/defaultWezTermPTY/image+TelegramloggedoutUI. User permits actualTelegramdrafts, sendsforbidden; accountstillmissing, no credential/sessioncopy ormessageinputperformed. Full browserWG343P56F33B separate; testsQtframeworknotTelegramcomposer. Runtime/transports/model/userlaptop unchanged; no newcodefixes.


2026-10-14T11:53:02.319349+00:00 — WJ native-app setup1 BLOCKED before device open/keys: host stock IBus discoveryNone; private guest/host machine IDs differ. Archived setup +13hashes /home/e/projects/lay-development-runner/firefox-legacy-native-20261012.6onP5c/WJ-native-app-setup1-1791978664353212704/ARCHIVE.json; exact5 original ticks gone/CGempty/VT8→1. No appcase verdict (88NOT_TESTED). Test-only decision2026-10-14-native-app-stock-ibus-shared-machine-identity: app-only bwrap ro-bind host machine-id, stock discovery/owner guards retained; GTK observer returnsFalse; early actor errors produce bounded receipts; missing native caret BLOCKED. ExactbuiltIME84f851/daemon084169 and source848 stayfrozen; no global IBus/laptop/production routes/model changes. Next setup must prove ordinary host discovery equals existing private GNOME-owned bus before first key. Telegramdrafts authorized/sendforbidden, no authenticated remote composer; fixtureQt acceptance separate. Latest complete WG343P56F33B432 unchanged.


2026-10-14T11:56:23.548210+00:00 — WJ native-app setup2 confirms stock host discovery now CONNECTED toexactowned private IBus. Nexttestguard failedbeforekeys: default vendor path /usr/bin points hostUbuntu22, while ownedGNOME50IBus uses officialguestUbuntu26. Exactownedsha24338c equalsrootfs-v2privatevendor, differshostcb8cca; no corefailure. Setup2 archive /home/e/projects/lay-development-runner/firefox-legacy-native-20261012.6onP5c/WJ-native-app-setup2-1791978934553235625/ARCHIVE.json; actual15548CLOSED0/driver64007CLOSED1/5GoneCGemptyVT1/88NOTTESTED. Extendtesthelper with explicitprivatevendorfilepath forhostactor only, retainingallotherguards/browserdefault; preflight fullhostguard before firstkey. Source848/buildbytesunchanged;WGmatrix unchanged.


2026-10-14T12:01:16.731291+00:00 — WJ native-app setup3 fullordinaryIBus/owner/package preflight PASS. Actor reached realGTK defaultwindow but aborted BEFOREfirstkey because host Atspi has clear_cache(), not clear_cache_single(); InputDevice needsEV_SYNwrite, notsyn(). MeasuredAPIinventory verifies replacement methods, old nonexistent methodsfalse; script corrected, no runtime source change. Setup3archive /home/e/projects/lay-development-runner/firefox-legacy-native-20261012.6onP5c/WJ-native-app-setup3-1791979158775405222/ARCHIVE.json; actual70082CLOSED0/driver66485CLOSED1/5GoneCGemptyVT1; keys0/rows0/88NOTTESTED. Terminalwholephysicalkey/windowledger enabled andperappledger retained; frozennewtestprotocol8hashesrevision4. source848/build84f851/084169/WG343P56F33B unchanged.


2026-10-14T12:11:30.937575+00:00 — WJ firstnativepartial4:4PASS0FAIL1BLOCKED83NOT_TESTED/88planned, keys380/noheldkeys. RealGTKTextView short8,long8,mixed8,english8 all8gestures+nextletter/backspace PASS; prefix8 BLOCKEDactualnativepreeditcaret unavailable. Archive remote WJ-native-app-partial4-1791979559627252021/ARCHIVE.json. Native screenshot hostCLI timedout8s beforeburst gestures, actoraborted; actual61978CLOSED0/driver66371CLOSED1/5goneCGemptyVT1. Preserve5finalized rows/noretry; remainder83 continueafter boundedtestcollectorrepair. ReadonlyprivateGNOME capturemethodusing officialShell.Screenshot(stream), before/afterFG/pixelSHA; exactremoval restoresunchangedalloriginalinputmethods; noIMEroute/source/runtimeauthority/laptopchange. Source848/build84f851/084169/fullWG343P56F33B remainseparate; TelegramcomposerBLOCKEDnoauth.


2026-10-14T12:25:46.396906+00:00 — WJ nativepartial5 completedbothGTK32:15PASS13FAIL4BLOCKED56NOT_TESTED/88; exactsamecorebytes,632newkeys plusprior380. ActualcompositorPNGs nowcaptured/noFocuschange,4originalinputmethods byte-subtraction unchanged. DefaultQtstartup writesnewIBusdiscovery1334619/addressAp3BE075 replacingbound1331237/vGKsEUpY; strictguardstopsBEFOREQtkeys, firstparentcauseUNKNOWNafterteardown. actual80654CLOSED75(descendantcleanup)/driver90161CLOSED1; lateownedL11descendantgoneBEFOREroot signal0, exact5Gone/CGempty/VT8→1. Archive /home/e/projects/lay-development-runner/firefox-legacy-native-20261012.6onP5c/WJ-native-app-partial5-1791980456252180239/ARCHIVE.json. App-onlysession prewarm/focusdefaultQt0keys beforeoriginal5s/finalGNOMEownedbusfreeze, reuseexactQt process; noafter-inputbusrebind/routeflag/productionchange. Retainall32GTKrows/noretry, continue56. Source848/buildfrozen/fullWG343P56F33B unchanged.


2026-10-14T12:34:57.278877+00:00 — WJ setup6 Qtzero-keyprewarm/focus andfinalGNOMEchildIBus binding PASS, but exactFirefoxbyteguard refusedactualdc8d81e againstoriginalb30896 BEFOREappactor; keys0. Archive /home/e/projects/lay-development-runner/firefox-legacy-native-20261012.6onP5c/WJ-native-app-setup6-1791981057620922261/ARCHIVE.json; actual10085CLOSED0/driver51423CLOSED1/5goneCGemptyVT1. Restoreoriginalcachedofficialtar0d92bf exactfirefox-binb30896 (guarded18641CLOSED0), preservechangedtree/profile, privateDisableAppUpdate policy/newemptytestprofile (no allow-downgrade/guardweakening/IMEroutechange). Finiteak retired/newownedal1393969/t17871395/session4321VT8; native7 willretain32GTK15P13F4B andrun56remaining. Source848/buildpair unchanged/fullWG343P56F33B separate.


2026-10-14T12:40:37.899485+00:00 — WJ setup7 ordinaryQtprewarm/busbind/originalFirefoxbyteguard PASS. Actornewkeys0: Qt widget grab_focus acknowledgment doesnotensuretoplevelactive; exactFGguardstops. actual41643CLOSED0/driver73536CLOSED1/5goneCGemptyVT1,56remainingNOTTESTED. Newprivatecompositorsetupmethod activatesonlyoneexistingwindowofpinnedprocessinsameownedCG; actoralso pinsPID/tick/CG, waitsactualFG+fieldfocus, no SSI/inputauthority/IMEroutechange. Capturemethodremainsreadonly; focusactivationdeclarednativeUIsetup, originalinputmethodsbodiesunchanged. Retainall32GTK including13FAIL; source848/buildpair/fullWG343P56F33B unchanged.


2026-10-14T13:13:40.965631+00:00 — WJ NATIVE88 COMPLETE scoped32PASS49FAIL7BLOCKED0NOT_TESTED. BasicWezTerm inlinepreedit/candidate-at-caret andSpacecommit image pair PASS (PTYemptybefore; preciseприветSpaceafter); focus-renderotherpositions outside88NOTTESTED. Perapp GTKTextView12P2F2B,Entry3P11F2B,Qt5Line/plain/rich each4P11F1B,Wez5P3F. Speed44/290 measured,246missing/13early-marker samples,median75.906/p9586.828/max91.946ms; notkernelrelease/paintproof. Telegramnoauth composerBLOCKED; firstlaunchthreadlimit512 refusal/noceilingbypass, send0, imageactuallyWez/notTelegramproof. Finalactual5286CLOSED0/driver33110CLOSED0/3setupparents0/noheldkeys/5GoneCGemptyVT1. Immutablearchive /home/e/projects/lay-development-runner/firefox-legacy-native-20261012.6onP5c/WJ-native-app-acceptance-1791983329525303054/ARCHIVE.json35files, rawSHAec139383..., rootvisualassessmentseparate; allprior32GTKresults carriedbynativehash/noretry. Newnativefirstmeasurement/regressionbaselineWGUNKNOWN/runtimefirstrefusalUNKNOWN; no newsource/model/IMEroute/laptopchange. Owningreport docs/development/wj-native-apps-2026-10-14.md. FullWJbrowsersNOTTESTED; oldWG343P56F33B remains. TD121OPEN/reviewNOTRUN/nopublication.


2026-10-14T13:31:13.911550+00:00 — WJcore artifact packaged (guarded14642CLOSED0) andfetched/localall11payloadhashes verified: lay-1.0.82-WJ-core-candidate-20261014.tar.gz8150727B SHAe31fbb4fbc8462d8b41b5fb9b2fa06426f158535f82bd74560d88b65cc1937de. Telegramsolo4CPUaffinity underunchanged512task/20CPU ceiling failsbeforebind/keys: TDWebViewsocket pathname>108 underlongprivateXDG_RUNTIME_DIR. Failedsetup31787CLOSED1/driver82220CLOSED1/ownedCGemptyVT1/0keys/sends; archive remoteWJ-telegram-failed-launch-1791984600374689538/CLOSED.json. AddonlyhostTelegramshort/runtimealias /run/user/1000/lay-tg-6onP5c tosameexistingdirectory (sameinode/device), no endpoint/IMEroutechange; source/binaries/native88/fullWG unchanged. Newownedfiniteam seat replacesretiredal. Actor preparesloginUI only; auth remains userboundary, no composerinputbeforelogin.


2026-10-14T13:45:54.489206+00:00 — Telegramactual7.3.1 loginUI nowvisuallyconfirmed inownedprivateGNOME scope actual96785LIVE, TG1584358/0keyssends0. Separateauth-onlysetup hasnoIME/nativebehaviorverdict; original005browserattachment archived (requiredrestorebeforefunctionalinput). Reducednativeappconcurrency/TelegramCPUaffinity4 withinunchanged512task/20CPUceiling; shortexistingruntimealias sameinode/dev fixes108-byteTDWebView pathnamefatal. FirstdirectAtSpiStartMessaging returnsack butscreenstillwelcome; actuallogin remainsuseraction/authboundary. Authquestionpending; elapsedtime notauth/permission. Source848/corepair/native8832P49F7B/fulloldWG343P56F33B unchanged. Pairedcore8.15MB archiveSHAe31fbb4f...all11payloadhashesverified; no installer/publication/localruntimechange.

## Local Firefox1.0.83 iframe diagnostic — 2026-10-14T15:24:54.088336+00:00

User reports Double Shift failure and explicitly requests checking here on the local machine; identifies actual field10, “Редактор внутри iframe”. This authorizes the local diagnostic exception to the prior remote-only test scope. Installed CLI/IME/daemon bytes and user profile are preserved. Actual Firefox157.0 snap8995, PID767837; own standard test window4063878607, all native input bound to that title/window/PID/starttick, actual field focus and existing daemon-attached AT keyboard. Browser/account profiles unchanged; no native text input into WhatsApp or other user sites. No runtime install/restart, delivery-route change, model fit or learning cleanup.

Exact field10 test with simplytyped `просто`: active roundtrip PASS (both surfaces and next-character decoder/Backspace), post-Space5-gesture burst FAIL. First conversion becomes `ghjcnj ` with native caret7 in108.444ms, then gestures2–5 leave it unchanged. All4 reach a live bridge but manual RPC returns `context_authority`; first refusal at authority.rs706, word_known=false, managed_word_mode=false, snapshot present/unselected, witness absent, manual toggle/refresh both false. Between first delegation and second refusal the newUS owner109 sees Reset serials3246/3256/3260/3276/3279; final3279 has refused_timeout. Second marker313 settles in under1ms, so the observed failure is edit admission after successful dispatch, not a missing gesture. The exact first loss of replay/reset authority still requires a causal source test; timeout alone is not asserted as cause. Do not weaken the guard or use a literal word/frame bypass.

Scoped transformation timing3/7 samples,4 missing; median103.207ms, p95/max108.444ms; evdev write-return→AT-SPI observation, not human-release or paint. Input2plan verdicts1PASS1FAIL; no conversion failure excluded from timing denominator. First observed refusal, raw sequence and all earlier setup scopes retained at `/home/ubu/.cache/lay/development/firefox83-local-sequence-kwjodjvq`; root assessment `/home/ubu/.cache/lay/development/firefox83-local-sequence-kwjodjvq/ASSESSMENT.json`, raw SHA1c6e2634a55e57b0aaa831d0666d33805b3cf8a582142f1a5b0bf5a17b6e98ff. Field1/7 local active roundtrips and field1/7 five-target controls pass in separate scopes; they do not certify iframe10. Remaining8-field actor sent0keys because Firefox was behind Kitty while user replied; classify16plans BLOCKED_SETUP, not product failures. Earlier observer artifacts are explicitly retained rather than silently promoted.

Latest complete WG browser counts remain Firefox164P33F19B/216, Chrome179P23F14B/216, total343P56F33B/432. FullWJ/1.0.83 browser acceptance NOT_TESTED; TD121 OPEN and independent WJ review NOT_RUN retained. One local reproduction finished; no product source edits or rollback.


2026-10-14T16:32:23.959123+00:00 — Local iframe10 follow-up hypothesis: complete native replay delivers target to actual editor, while prior/partial same-client SSI remains unconfirmed (`observed_replay_prior_surface`); next bridge marker is live, manual/refresh admission false. Owner remains existing ContextAdmissionReducer/WindowInteraction. Proposed9/10 reuse same-gesture exact client refresh for only a current fully completed paired replay; zero text effect until actual exact final receipt. Local-buffer promotion2/10 and deadline widening3/10 rejected. Test-only late_native_replay_refresh.rs added under existing residual harness: replay completion, actual Reset and old SSI, next real ManualToggleV3 must request fresh exact receipt without Delete/Commit; exact final SSI then same gesture delegates. Prefix/ASCII-NBSP variants. Baseline1.0.83 coreb742b64/bc89b4, local RESULTkwjodjvq1P1F, full oldWG343P56F33B. No production change yet.


2026-10-14T16:48:35.800357+00:00 — CausalRED exactRPC0vs4 confirmed in iframe-refresh-red-7e7n5ns2 after preserving earlier797PASS and correcting fixtureKnownStart-vsReset provenance check. Manual-only current completed-paired-native refresh implemented via existing RequireSurroundingText/pending_manual_toggle/700ms; final exactSSI required, no sourceKnownStart/candidate/verifier/route/model change. Decision2026-10-14-completed-replay-same-gesture-refresh.json. Telegram user grants any available field/noauth needed; currentSnapATSPItree onlyapplication/nochildren, do not cloneprofile or waitlogin. Installed83 untouched; source/native/review pending.


2026-10-14T17:12:42.739789+00:00 — Completed-replay refresh: causalRED0vs4 retained; minimal four-file manual-only wait predicate, same marked client and final receipt. Focused2GREEN plus full799IMEPASS, six contracts/ASTpending; installed nativeNOT_TESTED. NoReset unmatched projection is an adverse control, not an expanded admission claim. Context-free reviewer spawn rejected threadlimit/NOT_RUN/no score. Telegram existing profile activated normally, any empty nonsensitive field allowed without auth/sends. TD121OPEN/latestcompleteWG343P56F33B/432.


2026-10-14T17:36:14.668777+00:00 — Installed330eda0c... causaliframe10 active2targets+postSpace5 nowPASS7/7 (median115.234/p95max222.170ms), oldkwjodjvq fail retained; source850/850+ASTPASS. Telegram search2endpoint/composer1endpoint visually observed, user explicitly acceptsTelegram; NO MORE Telegram tests. Scope/proof report completed-native-replay-refresh-2026-10-14.md. ReviewerNOT_RUNthreadlimit; full new-byte matrixNOT_TESTED/latestWG343P56F33B; TD121OPEN/no push/tag/release.


2026-10-14T23:03:55.418448+00:00 — Performance authorized: diagnostic two-file daemon timing only, actual first expensive stage stillUNKNOWN. Same fixed actor/native baseline next; old seven mixed samples/observer gaps retained. No accepted local/TG retest or runtime change.


Performance follow-up2026-10-15: independent fresh-context read-only review completed via separate ephemeral Codex after built-in thread quota rejection:9/10, no blocking source defect; receipt `/home/ubu/.cache/lay/development/remote-firefox-setup-j2he3qf6/PERF-pacing-review/REVIEW-1.md`. Applied its two nonblocking test-only suggestions (explicit2ms×n then16/4ms schedules; prohibit an extra ordinary delete). Runtime production sections unchanged; follow-up source gate pending. Clean PLAIN Firefox original first8 jobs186 rows complete; final30 interrupted at field9 pre-input SHOWING guard, six priorPASS/remainingNOT_TESTED retained in `firefox-functional-c3xe2k7g/RESULT.json`. Added only pre-input750ms visibility wait; no key/target/clock/deadline or guard relaxation, new SHA67f5012c... explicitly pinned. Failed zero-key pin mismatch0behavior input retained. Finish original final30 in fresh exact-owned scope; do not erase interrupted evidence or substitute it for final complete acceptance. Candidate/native speed stillPENDING; accepted local bytes untouched.


Performance attempt1 closed2026-10-15: clean PLAIN Firefox216=170PASS/27FAIL/19BLOCKED. Standard420/420 measured, median106.640ms/p95119.943ms. All216 IDs preserved in `/home/ubu/.cache/lay/development/remote-firefox-setup-j2he3qf6/PERF-PLAIN-firefox/RESULT.json`. Four-ms daemoncc65fbe7... rejected after normal20=19PASS/1FAIL: field6/space/burst lost a baselinePASS. Same-window clean baseline140 samples median106.081ms/p95117.449ms; shortened candidate137/140 observed samples around89ms, three missing. Speed does not permit promotion. First failed transition is third burst bridge_context_authority refusal (missing witness), before that gesture deletion; causal dependence on4ms NOT_ESTABLISHED. Do not weaken admission or retry unchanged untilPASS. Raw primary `firefox-standard-normal-k9zlr898/RESULT.json`, SHA7963d6c9732369749f1c3e1afe8849c1f4d7800736706e2b287c586b81587449, rejection `/home/e/projects/lay-development-runner/firefox-legacy-native-20261012.6onP5c/PERF-FAST4-REJECTED/RESULT.json`. Incomplete private job stopped; all pinned GUI gone/scopeempty/VT1. No current full candidate matrix, ChromeNOT_TESTED. Next single bounded replan8ms (predicted8ms saving); no other production change. Fresh327 source check running. Accepted local runtime remains unchanged; no model training or route edits.


Eight-ms Firefox A/B complete: all216 stableID verdicts identical (170PASS/27FAIL/19BLOCKED), no lostPASS or changedverdict. Same-source actors/60standard+6panel+150functional, closure5pinnedgone/CGempty/VT1/noheldkeys. Standard420/420each: median106.640→93.547ms,p95119.943→115.286ms, pairedmedian improvement11.133ms(n420). Panel48/48each:102.928→92.392ms; functionalDoubleShift134/210each:104.109→93.041ms,p95140.378→131.893ms,76missingretained. Space37/50each:17.538→18.004ms, pairedchange-0.243ms; no speedupclaim forSpace. Tab0/10 and selectionbaseline0/10/candidate1/10 remain insufficient for speedclaims, neitherconvertedtozero. Compare receipt /home/ubu/.cache/lay/development/remote-firefox-setup-j2he3qf6/PERF-FAST-firefox-vs-PLAIN.json,SHAe0ea886f21657229fd92c4840a13ef74a22e2c89acb8378111f917282161c8b3. Raw9primaries/logs/bodies retainedlocallyBDIR/PERF-FAST-firefox and remotelyRUN/PERF-FAST/firefox. Nativeclock evdev write-return→exactAT-SPI, notkernel/paint. PLAINChrome nowrunning; both-browsercomplete current-source verdict stillPENDING, latestcompleteWGmatrixretained. Acceptedlocalruntimeunchanged.


Clean PLAIN BOTH baseline complete: Firefox170PASS/27FAIL/19BLOCKED +Chrome179PASS/23FAIL/14BLOCKED =349PASS/50FAIL/33BLOCKED of432,0NOT_TESTED. Exact acceptedBC89daemon/330IME in both, same private frozen data/usage and fixed216/browser; compare with futureFASTboth, never mix phases. Local primariesBDIR/PERF-PLAIN-firefox/RESULT.json and PERF-PLAIN-chrome/RESULT.json; remoteRUN/PERF-PLAIN/{browser}/RESULT.json. Chrome standard406/420measured,14missing retained:median92.055ms,p95111.443ms. ChromeprivacyPASS_VISITED_LAY_SINK_ROOTS normalpositivecontrolrecent_actions.jsonlchanged/privateunchanged (localBDIR/PERFPLAIN-chrome-private-sink-proof.json); scopevisitedLaywriterroots only. Closureall5goneCGemptyVT1/noheldverified. Latestcompleteboth table now this349/50/33baseline, replaceshistoricalWG343/56/33 forstatus. New8msChrome running; final8msboth gatePENDING. Runtimeauthority/installedbytes unchanged.


Eight-ms attempt REJECTED after fullBOTH: Firefox170P27F19B,Chrome178P24F14B,total348P51F33B/432,0NOT_TESTED; acceptedcleanbaseline349P50F33B. Chrome lost exactlyfunctional/normal/space_ru_en/field8: expectedlaySPACE, actualдфнlaySPACE. FirstobservedfailureSpace target, not a DoubleShift action; exactcausaldependenceon8msUNKNOWN, no waiver. Chrome406/420standardmedian92.055→86.585,p95111.443→102.286,paired6.147ms; privacybothPASS. Gains do not justifylostPASS. ComparisonlocalBDIR/PERF-FAST-chrome-vs-PLAIN.json, raw9primaries/logs/bodiesBDIR/PERF-FAST-chrome andremoteRUN/PERF-FAST/chrome. Closureall5gone/CGempty/VT1/noheld. PreparedinstallergatesrejectlostPASSand wasNOTRUN; acceptedlocalBC89daemon/330IME unchanged. RevertONLYown3-filepacingincrement toexactaccepted83copies, retainallfailedsource/build/native/reviewproofs. Do not silentlyreintroduce4or8ms or retryuntilPASS. Next bounded alternative: measuredread-onlyIBus engine query cost5.362msCLI vs0.386msnative; inspectexistingconnectionfirst, no input-route/owner/cache changes withoutdecision.


ObservedIbusEngine source1133/1133PASS,0fail,272.7s; local/home/ubu/.cache/lay/development/run-trdx7u5g/RESULT.json,remote/home/e/projects/lay-development-runner/run-d9azvU,archiveab3839443c57f28ae5f69da22a9cd31f6a83a25cfcf2e047ece79d2c8f63bf03. Selectedbothprivatebinaryowners+sixsharedcontracts, notlibrary/releaseacceptance. Tennewtyped-wire/realRPC/fallback/mismatch/timeout/latereply regressions; actualbackendfresh whileSharedprojectionpoisoned, nogetterstateaccess. Newconsumeronlyverify_engine_once. Query150ms usingexistingstdpoll/async_ioTimer, existingdaemon250ms; legacyerrorfallback remainsoldunboundedCLI. Generalread/ensure andphysical16ms/2msunchanged. Firstcompilefailureretained(run-tqfufdi9); twoAPI/dependencyassumptionscorrectedwithoutnewdependency. NoGetGlobalEngine subscriptions in existingnarrowadmissionstreams(adapter.rs:199-245), no exemptionadded. Newbinarybuild/review/nativePENDING, no localinstall, rejected4/8msstillrejected.


Observed-engine finalsource1134/1134PASS,0fail,271.9s; local/home/ubu/.cache/lay/development/run-3recp7ey/RESULT.json,remote/home/e/projects/lay-development-runner/run-dWWz50,archiveaf0d9b36e84e9612b54708a485c2df608fb0a82afeedd7a6135fd20ccea67d4f. Finalcontext-freecodereview2=9/10,noblockingfindings,receiptBDIR/PERF-readback-review/CODE-REVIEW-2-RESULT.md. Review1=7/10rejectedunsafe150mswhole-callcancellation+uncheckedDict. Corrected: no backend cancellation, one pending observation viaRAII atomic resourceflag; existingdaemon250msONLYreplywaitafterfullysentsessionframe, serverhandlerdetached. BackendcompletionandlegacyCLI remainunbounded,no150ms/overall250mspromise. TypedattachmentsEXACTa{sv}. FivefocusedtestsPASS includingactualone-bytepartialwrite/backpressurepastcaller250, continuedframeandfreshfollowingquery,latebackend350replynotcached,busybudget1,poisonedSharedandtypedwire. Nonblockinggapbackenddisconnectnotdistincttest(clientdisconnect/errorcleanupcovered)retained;notnativewaiver. GeneralCLIread/ensure,allinput/outputpaces16ms+2ms,modelsanddeliveryrouteunchanged. Newdefaultbinarypair/zero-keyactualRPCparity/native432+privacy stillPENDING. Nothinginstalled.


Safeobserved-engine defaultpairbuilt183.905s aftercanonicalAST/architecturePASS. IMEf633f6aac492817031b025c02c8746a27eac6541f345ea60873415a4b52fedc3; daemonba6d9d77163a13284c3540481806c8b710570160bb8e571164309014a318a19e; immutableRUN/PERF-read-build/BUILD.json,frozenfinalrun-dWWz50/source+explicitmetadataoverlays. Threephysical-outputsourcehashesexactaccepted16ms/2ms;no codechangedafter1134/review9. Fourgraphartifactsfetchedverified. READFirefoxzeroattachmentPASS,newpairloadedwithpinnedUID/CG/PIDtick/peer/windowIC. ActualObservedIbusEngine30pairsallmatchCLI,zero keys:RPCmedian0.541ms(first1.542),CLI5.398ms,p950RPC1.199/CLI6.130. APIprobeR/PERFREAD-firefox-engine-readback.json,notnativeinputspeed. Original9READFF216running;no install/localruntimeauthoritychange.


READFirefox attempt1 incomplete duepreinputvisibility: originalnormal20PASS,private18PASS+field9setupBLOCKED+field10NOT_TESTED,remaining176unqueued(aggregate38P1B177NT/216). No field9behaviorinput/runtimefailureobserved. RetainedRUN/PERF-READ-visibility-blocked-1 includingalloldactors/pins/rawreceipts. Strengthenedonlynativefocus_editor precondition: ownedforeground+scroll_toANYWHEREifhidden+originalfocus+750msSHOWINGwait. Guardunchanged; scopevalidationexactnonfocuscodeparityPASS,0keys; newexplicitactorhashes standard4678321d/functional33091ab0/panel850f2b1a. No source/binarychanges: samef633IME/ba6daemon,source1134/review9valid. WholefreshREADFF216restartedtoavoidmixingdifferentharnessmetadata, unitread-visible, matrix43682/GUI33921; oldpartialneverpromotedorreplaced. Decision2026-10-15-browser-fixture-native-viewport-preparation.json,publicprotocolupdated.


READ Firefox attempt2 NOT_PROMOTABLE / INCOMPLETE, retained RUN/PERF-READ-firefox-incomplete-2 (all9 primaries, jobs, actor versions, candidate metadata, logs; local BDIR mirror). Counts156PASS23FAIL15BLOCKED22NOT_TESTED/216. Fullstandard60cells andpanel6PASS, standard420/420:median98.687ms,p95111.197ms vsPLAIN106.640/119.943. Preliminarygain only, notbothbrowser acceptance. LostoldPASS functional/normal/prefix/field1: firstRU→US16.552ms, reverse textcorrectonfailure snapshotbutmode evidence absent inoldactor; daemonlog says focusedIMEhandledbothgestures, nofallback. CauseUNKNOWN; donotwaiveorinstall. Lastjobfield9/space_ru_en BLOCKEDbeforebehavioralinput: hiddenafterfocus/setup, remaining22NOT_TESTED. Boundprocessesgone/CGempty/VT1. Newboundeddiagnostic: oneprefixroundtrip with existingmodeRPCsamples;15preinputiframefocus/layoutsetups with nativeobjectidentity/states/rect, zero behaviorletters. No production sourcechanges/rebuild/review or localinstallation; source1134/review9 unchanged.


Matched native initial-stack protocol prepared, decision2026-10-15-matched-native-initial-stack-readiness.json. Exact frozen Rust parity accepted83run-Qq20tO vsgetterrun-dWWz50 verified: exactly6 intended source files differ, no physical-output changes. Old prefix loss remains unresolved/causalityUNKNOWN. Bounded diagnostic15preinputsetups/300SHOWING samples+freshobjects didnotreproduce; parent finalizationNameError __file__ meansnotcompletedPASS. Newharness strengthens preinputreadiness100ms focus/SHOWING + independent source/mode/CLI agreement withoutactivation; records existing targetmode read, no newtimedRPC. Fixed single matched PLAINV+READV216/browser, no unchanged retries; allhistorical losses retained andmustbeexplained beforeinstall. Productsource1134/review9unchanged.


PLAINVFirefox full216 complete170PASS27FAIL19BLOCKED,0NOT_TESTED; unchanged acceptedBC89daemon/330IME, newstrictsetupharness. Original9elapsed776.247s. Iframe last30complete20P5F5B; prefixfield1PASS with observedRU. AllboundgoneCGemptyVT1. Primary RUN/PERF-PLAINV/firefox/RESULT.json. PLAINVChrome nowrunning; do not mix this newFFrow with olderChrome phase. Fourfixedphase sequence continues with no core/source/review changes.


PLAINV BOTH full control complete: Firefox170P27F19B +Chrome179P23F14B =349P50F33B/432,0NOT_TESTED. RUN/PERF-PLAINV/{firefox,chrome}/RESULT.json; identical strictsetupactorhashes/frozendata; original9elapsed776.247/774.219s. ChromeprivacyPASS_VISITED_LAY_SINK_ROOTS, normalrecent_actions.jsonlchanged/privateunchanged3files. BoundgoneCGemptyVT1/noheldeach. This is latest COMPLETE BOTH status, replacing rejectedFAST8 table. READVFirefox newf633IME/ba6daemon nowstrictzeroPASS;30actualgetter/CLI namesmatch, RPCmedian0.649738/p950.757272ms vsCLI5.469076/5.757048, firstRPC1.274141ms. Not native latency/acceptance. Newfull216FFthenChrome remainsrunning. No source/core/review/install change.


READVChrome firstfunctional30:19PASS8FAIL3BLOCKED versusPLAINV19PASS7FAIL4BLOCKED. Exactchangefunctional/normal/undo/field2: baselineBLOCKED/setup exactfunctional seed(rawпревеликое,caret6);candidate obtainedexactпревет seed, thenSpace targetFAIL(rawпреветSPACE,caret7). No baselinePASS lost inthisgroup. This isadditionalobservedcoverage of previouslyBLOCKED behavior, notproof of regression orrepair. KeepB→F explicit; no correction/model/routechanges, and no retry ofthiscase. FullChrome216stillrunning; release84metadata onlyPREPARED/notbumped, no installation. VERSIONING.md requires synchronizednewreleaseversion; ifremainingproofgatespass, prepare84coreCLI/daemon/IME+extensionmetadata asbyte-identicalRust-source/dependencylogic descendant oftested83. Reusefunctionalproofpermetadata-onlyexception; build3corebinsremoteunderexistingguards, no broadbump-script/modelrefresh.


READV BOTH full native complete349PASS51FAIL32BLOCKED/432,0NOT_TESTED. All349historical/matchedPASS retained; Chromeundo/field2 BLOCKEDsetup→FAILSpace explicitlyrecorded. Firefox420/420107.254→98.507median118.259→111.112p95,paired8.235;Chrome406/420(14missing)92.161→83.306median112.631→100.073p95,paired9.801ms. BothprivacyPASS. Lastguardexit75cleanupretained; onlyownedprivateL1.1PID3971292/tick23111169 closedbyverifiedpidfdSIGTERM,CGempty/5boundgone/VT1/seatclosed,0repeatedinputs. Full9COMPLETEreceiptscollected unchanged afterrecovery. Decision2026-10-15-observed-readback-release84-metadata-only.json. Sourceversion84metadataONLY manuallysynchronized perVERSIONING; installed83unchanged(SOURCE_BUMPED_RUNTIME_STALE), full3corebins84buildpending. AllRust/input/dependencies/extensionlogic parity required; no newfunctionaltests forversion-only change. No models/globalIBus/sourceIDs/deliveryroutes changed.


TD-141 COMPLETE / installed1.0.84. Build RUN/release84-build/BUILD.json:232.239s, all739Rustfiles/dependencies/extensioncode exactlyequal tested83 except version surfaces, canonicalarchitecturePASS. CLI60de3f20…, daemon6d0237e6…, IME7e338ae2…. Local transaction /home/ubu/.cache/lay/development/remote-firefox-setup-j2he3qf6/install83-20261015T033053Z/INSTALL.json: CLI/tray84 and/procSHAverified,24protectedfiles, globalIBus/L11/L3processidentities+config/source/mode unchanged,0functionalkeys. NativeREADV432 logicproof349P51F32B/0NT,0lost349previousPASS, Chromeundo/searchB→F retained; metadata-only84 proofcarryexplicit, no localfunctional84claim. Freshsource/harnessreviewsfinal9/10(twoeach),1134sourcePASSunchanged. TD121legacyfunctionalworkremainsOPEN. No models/deliveryroutes/runtimeauthority change; endpointremainsfreshread-only onexistingconnections.


## TD-147 opt-in shortcuts — before production edit

2026-10-16T08:25:11.537592+00:00; baseline05e1e448 source88, installed87 unchanged. Hypothesis: new opt-in Backspace must reach the existing exact undo owner before next-key invalidation; double-Space needs ordinary boundary accounting and exact suffix count before the existing gesture dispatch. First failures and accepted plan8/10 are recorded in tech_debt/147-explicit-undo-and-double-space.md and decision2026-10-16-opt-in-existing-owner-shortcuts.json. No early KnownStart veto, parallel legacy detector, transport bypass, models or training changes. Start with real executable RED; source, native, timing and installed evidence remain separate. Historical complete matrix349P51F32B/432; new bytes NOT_TESTED.

## TD-150 — completed native replay Reset capture, focused verdict

2026-10-16; baseline e6f3227c, built88/installed87. The single same-byte full
Firefox confirmation completed216 at169P28F19B but lost fullscreen/space/field8;
its per-cell trace records late matched Reset without a predecessor and a live
bridge context_authority refusal. Both complete E6 Firefox receipts and original
short4 failure remain immutable; no third same-byte full rerun. Existing deferred
observer/callback stream reproduces the exact authority failure in
td150-observer-red-wtmv6a4m,2/3 controlsPASS. Remove only the whitespace rejection
in the existing completed-replay capture fallback, preserving all proof gates.
td150-focused-proof-vfotdhss11/11 PASS, controlled old guard1/1 detected; fresh
plan9/10 and independent code review9/10(pass1), root all implementation.
Full source3210 expected is running; no new-byte build/native/install claim.
[Owning task](../../tech_debt/150-retain-completed-native-replay-across-reset.md)
contains exact hashes, failed fixtures, adverse cases, consequences and limits.
No early KnownStart veto, new owner/route/timeout/model/learning change.

TD-150 full source verdict: td150-full-source-5m8befqy3210/3210 PASS,
3174 correctness+36 package, zero failures/exemptions,694.58s. Manifest3236
keeps all old classifications/15ignored/11performance and adds only the causal
regression. MSRV1.88 both configurations, strict lints530/354 unchanged, canon15,
architecture/audit50, Node/harness/panel PASS;994 tested files matched locally.
Fresh source review9/10; final canonical/matched build/new native pending.
Both original rejected E6 full Firefox receipts remain unchanged.


## TD-151 / TD-152 — new b411 release blockers before production edits

2026-10-16. TD-150 final build a0620bf..., IME d0ef253..., Firefox complete
169P28F19B/216. Captured fullscreen/space8 repeat now5/5, standard60/60,
420/420timings median98.694ms, p95112.273ms.
One lostPASS functional/normal/space_en_ru/7: exact inline preparations14/15
at epoch11522 precede matched Reset46045, pairedrelease and Space46047;
lookup NotReady worker15/wait0. Trace /home/ubu/.cache/lay/development/td150-install88-20261016/space7-failure/firefox-functional-l3nq1f8n-7-space_en_ru.jsonl.
Preparation trace alone precedes finish_request; causal test must establish
stored readiness and precise identity loss. Hypothesis: temporary post-Reset
capture mismatch retires a same-word prepared slot before restoring the exact
owned observation. No runtime edits; actual adapter RED pending. Chrome full
continues under sole remote heavy lease; do not splice the current pair.
[TD-151](../../tech_debt/151-preserve-prepared-space-frame-on-matched-reset.md).

Separate CI38901069439 failed overlapping_handled_releases_publish_only_the_latest_completion;
no raw panic/artifact, same remote3210 passed. Diagnose exact material readiness
and bounded callback order without assuming a runtime defect or adding retries.
[TD-152](../../tech_debt/152-control-overlapping-release-test-readiness.md).
Installed87/config/models/routes unchanged, installation blocked by both gates.


B411 complete matched new pair: Firefox169P28F19B / Chrome179P23F14B /
348P51F33B/432,0NOT_TESTED, both9jobs/emptyerrors/cleanupVT1. Only new
lostPASS Firefoxnormal/space_en_ru/7; Chrome0lost. TD-150 oldfullscreen8
now5/5; FFstandard420/420median98.694,p95112.273ms, Chrome406/420
(14missing)82.307,p9597.386. Release/installation blocked pending TD-151/152,
no unchanged-byte repeat. New source-only causal diagnosis snapshot frozen
3fc2c350..., td151-reset-diagnosis-359fg2lz, contains tests only/no runtime edit.


TD151 causal RED td151-reset-diagnosis-359fg2lz2/3PASS17.25s: real inline
exact-ready before matched Reset; after Reset slotgone although pending/current
frames exactly equal pre-Reset prepared. Old exact-refresh and CI baselinePASS.
First loss is existing soft-reset retention before owned observation restoration;
prefer existing stable pending predecessor frame, then former managed-echo
fallback. Decision2026-10-16-retain-stable-pending-space-frame-on-reset.json;
no changed identity guard, verifier, timeout, route or model. Green/control/full
and new native pending; installed87 unchanged. CI originalcause stillUNKNOWN.


TD151/152 fresh full source td151-full-source-gvt96j4h3211/3211 PASS,
3175correctness+36package,0failure/exemption,693.46s, archive844618fc... .
MSRV1.88both/strictlints530-354same/canon15/architecture-audit50/Node/harness122
(oneexplicitSkip)/panelPASS. Newmanifest3237 addsone TD151regression only;
oldclassifications/15ignored/11performanceunchanged. Finalgraph+test-only TD152
encoding assertion check pending, then matchingbuild/native/CI; installed87.


TD151 final canonical td151-final-canonical-1gumzyss PASS48.56s,17currentPython
encoding/fixed-only cases +15canon +15privatehelperAST;4canonicalmetadatafetched,
993functionalfullproofinputsidentical. Final independent TD151/TD152 code reviews
both9/10 pass2/2/noP1-P2; newcontext-free thread unavailable(platformlimit),
reused unrelated reviewthreads explicitly. Rootallimplementation. Source tasks
complete; genuine newCI/matchingbuild/native432+options80/install remainpending.
All oldrejectedreceiptsretained; no route/model/deadline/learning change.

## TD154 — consecutive owned Space acknowledgement, before production edits

2026-10-16T20:19:23Z. Baseline fe614235 / installed release88 IME8cb7bca... .
Default whole432 remains349P50F33B. Optional normal80 remains58P18F4B;
Double Space40=26P14F. Read-only preserved trace35041eb5... field7 rapid10:
first owned Space confirmed asNBSP, second owned Space then matched Reset and
adjacent SSI rejected despite matching identity/length/cursor. The source
acknowledger forbids a previously observed separator immediately before its
new final Space. Hypothesis: retain exact previous separators and acknowledge
only the newly appended character using current owner/receipt guards.
[TD154](../../tech_debt/154-acknowledge-consecutive-owned-space-boundaries.md)
records alternatives9/3/1, scope, adverse tests and native limits. Six FF rich
cases share this mechanism. Four Chrome failures are strict ASCII target
comparison despite projected surface; two fail only a later probe. Two FF
release-order cases remain independent, no causal repair claimed.
Causal source RED and plan review pending; zero new native keys/runtime edits.

TD154 causal RED td154-consecutive-space-red-bcubbzg9:1/2PASS14.724s,
archive6c666828... . Original single-boundary cases pass; actual second complete
Space callback with adjacent exact SSI leaves abc+NBSP+ASCII instead of observed
abc+NBSP+NBSP. Existing adverse receipt test PASS. Production unchanged; causal
transition identified, new protected-path decision recorded before runtime edit.

TD154 first green td154-consecutive-space-green-zftayg2t10/10PASS17.231s:
second owned boundary retained under exact raw previous-prefix equality;
mixed/multiple boundaries and all existing/new adverse controls PASS. One
runtime expression; fresh-context plan review8→9/10, two passes complete.
New1.0.89 metadata prepared, installed88 unchanged. Controlled reversion and
full source gates next; no new native success claimed.

TD154 full preflight _7ih78_h stopped before full tests at Clippy's new fixture
MutexGuard/await diagnostic. Replace explicit drop with lexical scope, retain
all assertions and strict lint gate. Controlled one-expression rollback did
detect original second-boundary failure; no runtime revert or guard weakening.

TD154 final full source td154-full-source-final-50j7ij8d PASS3211/3211,
0failure/exemption,715.353s; archiveb2793f28... . All3237 IDs unchanged, both
actualMSRV1.88/strictlints/canon/architecture-audit50/Node-harness-panel PASS.
994 functional input hashes verified locally, controlled old guard RED detected
and exact source restored. Actual private observer predicates17/17 PASS including
ghost refusal, zero native keys. Independent code review/new build/whole native
and installation pending; installed88 and historical349/50/33 unchanged.


TD151 new-byteFirefox216 complete170P27F19B/zero lostPASS, Space7 nowPASS;
standard60/panel6,420/420DS median98.548/p95111.391ms. ChromeREPAIR2running,
source e34/build119269.../IME8cb7bca..., installed87. Source3211 unchanged.
FreshCI38908900651failed exact oldoverlapfixtureNone!=Some(ерка) atlastrelease,
no timeout; newboundedreportingexposedactualpanic. Security38908900839PASS.
[TD153](../../tech_debt/153-establish-overlapping-release-material-readiness.md)
now owns controlled readiness proof/test-only repair; no runtime edit or CI retry.
If finaltest-onlybuild allsix installedpayloadsarebyteidentical, explicitlybind
wholeoldnativeevidence ratherthan repeatinput; otherwise changed-bytegate required.


TD151 paired e34 native complete349P50F33B/432, NOT_TESTED0; FF170/27/19,
Chrome179/23/14, zero lostPASS in each vsREADV. Both scopes empty/five pinned
processes gone/no heldkeys/VT1. Chrome406/420standard DS med82.533/p95100.712ms,
14missing retained. TD153 plan review1/2=9/10; narrow cfg(test) exact-input/cache
accessor allowed, actual frame/display prerequisite + MISS/materialize/HIT proof
required before fixture repair; original CI missing-cache cause not yet proven.


TD153 controlled diagnosis td153-readiness-diagnosis-hvepdrsr PASS25.411s:
unchanged baseline PASS; exact authority/display-ready + usage-refresh cacheMISS
reproduces same None/Some(ерка) RED; actual materialize/cache-onlyHIT makesboth
releaseorders PASS, originalsourceSHAs restored. OriginalCIcachecause unproved.
Minimal cfg(test) prerequisite now owns readiness atlastrelease, no runtime edit;
plan review2/2=9/10, current focused/full/CI/equivalence pending.


TD153 focused11/11PASS21.95s; code review1/2=9/10/noP1-P2. Full private-helper
preflight falsely marked unchanged manifest as changed, canon rejected correctly
before full tests; actual3237 IDs/SHA unchanged. Corrected helper does not weaken
canon and resumes remaining gates from byte-identical fe493... frozen source,
reusing only independently recorded successful MSRV/lint/discovery checks.


TD153 final full completion PASS3211/3211/0fail/exempt,564.631s, all3237IDs
unchanged,994proofinputs verified. Exact snapshotfe493... priorMSRV/lints reused;
canon/architecture/audit50/Node/harness/panel PASS. Private6payload whole432
equivalence +mandatoryoptional80/CI-firstattempt-main qualification now explicit;
actualnewbuild/equality/options/CI/install still pending, installed87 unchanged.


TD154 source76361aff built89candidate203.644s, not published/installed. First
optionalFirefox40=25P12F3BLOCKED_PRECONDITION, complete native-shortcuts/firefox/RESULT.json
under td154-install89-20261016. Raw trace field7rapid10 confirms ACK24/46,
receipt25/47, consumed56, exact delegation57/status3at59. Daemon log rejects
exact replay requires preserved ASCII spaces or one acknowledged Space. TD154
producer repair confirmed through handoff; final replacement NOT_MET. TD155
owns the next existing exact-replay consumer, no producer revert. Default432
and Chrome optional newbytes NOT_TESTED; old349/50/33 remains unchanged.

TD155 causal unchanged-production td155-exact-series-red-lggpkxju:5/6PASS,5.310s,
archivedf8eb00b703b1be766b1bfd2e108b6b16b363ace80e70291810ed040d9e62b4d.
Extended actual existing executor regression fails two-NBSP representability
after single-boundary controls; other five current safety controls PASS.
Production unchanged; next uniform exact literal/count consumer validation
requires explicit decision and actual replay-stream regression.

TD155 first focused11/12PASS17.342s, archive89cb6922..., actual daemon6/6PASS.
Actual IME completed multi-Space stream loses exact manual authority at final
raw two-NBSP receipt: existing outer acknowledgement and paired-tail mirror only
change final Space. No new native run. Task amendment requests existing complete
replay witness for exactly its newly emitted Space positions, preserving byte-
exact old prefixes and every current owner/counter/provenance guard; fresh plan
review pass2 and new protected-path decision precede any IME runtime edits.

TD155 second focused mq6xj6cd:11/12PASS16.740s, archive89875b89... .
Stopped at a new negative fixture assertion: missing boundary is inert
AwaitingExactSnapshot(status4), not NotHandled(status0). No text mutation or
daemon delegation occurred. Preserve strict0 for all original adverse cases;
new contradiction assertions permit only inert0/4 with zero text effects and
no authoritative handoff. Do not change runtime to satisfy an overspecified
fixture. Complete multi-NBSP positives remain unproved until next run.

TD155 focused td155-series-green-9qqzqfio:12/12PASS25.671s,
archive1dc6f8200beb96ac4fd2ec432df723f5c243cb36c406ca115b4f658365f7ed89.
Uniform exact consumer + existing completed paired Space-series acknowledger
pass real callback/Reset/raw-SSI and strict contradiction controls. No state,
route/model/config/default changes. Full controlled/source gates pending; no
new native or installed evidence. Only later test failure diagnostics add the
explicit boundary count; assertions/runtime unchanged.

TD155 full wstyurem FAIL716.943s: unchanged3237IDs/no waivers; IME827 has2
old owned-prefix regressions. New outer candidate confused full owned tail with
pending token suffix. Correct geometry uses suffix only, retaining exact old
prefix for the same tail/scope owner. Both original one-boundary failures and
new two-boundary owned-prefix controls join focused proof. No publication/build/
install and no producer revert. First independent code review9/10 did not
identify this gap; pass2 required after corrected source proof.

TD155 corrected prefix-focused-jnf0564c14/14PASS30.197s, archivedf903f2a... .
Both old one-Space/owned-prefix failures and added two-Space owned-prefix
variants pass. Geometry compares only current pending token suffix; full owned
prefix stays exact for the same atomic tail/scope owner. New full will detect
three independent counterfactual restrictions/errors, no waiver or route change.

TD155 final g48ali01 sourcePASS3211/3211,0failure/exemption,730.988s,
archive47036521... . All3237IDs/classifications unchanged, actualMSRV/lints/canon/
audit50/Node-harness122/panel PASS. IME827/827 including both old regressions;
daemon294/294. All3counterfactuals detected and source exactly restored.
994proof inputs matched locally after verified generated/lint metadata sync.
Final code review2/canonical+observer17 and new89 build/native/CI/install next.

TD155 final canonical rwecv4fk PASS48.657s, archive9cd6b176...,993functional
inputs equal full g48ali01;18helper AST/hash identities and actual remote
observer17/17PASS,0nativekeys. Generated graph copied back; no runtime delta
after full3211. Final code review2/build/new native/CI/install pending.

TD155 final fresh-context code review9/10 ACCEPT pass2/2, noP1/P2,
/root/td155_space_series_final_code_review. Corrected suffix/full-tail boundary
and18helper pins verified; no runtime delta after final3211. Source task DONE;
new matching build/native/CI/release/install still pending, installed88 unchanged.


TD155 source8562c2bb published normally to public/main and both working branches;
matched89 build83.417s archive48ce41a5d4ad8b23e16be3a06dfef6205879313f3c6a0452964493801cfb0737,
daemon9b9f7dca / IMEb69b4ba0. BASE td155-install89-20261017 holds BUILD/REVIEW/
SOURCE-PUSH. Setup metadata errors caught before any functional job: invalid numeric
seat label, then reused TD154 zero-key output name. Alphabetic guards retained;
only fresh private names changed, recorded separately from independent source
review. Original partial40 NOT_TESTED and prep retained as SETUP-ATTEMPT1;
current cgroup/pinned processes/keyboard owner independently confirmed gone before
restaging. No false native PASS; first actual Firefox optional40 now running.
Security main first attempt succeeds; CI running. Installed88/source3211 unchanged.


TD155 actual first functional Firefox optional40 COMPLETE27PASS/10FAIL/
3BLOCKED_PRECONDITION,0NOT_TESTED,5jobs. Single pair10/10PASS including7/8/10;
rapid10=3/10, undo-first7/10, undo-second7PASS/3BLOCKED. Old installed88 FF40
25P12F3B gives5gains but3lostPASS (rapid fields1/4/6), no causal regression verdict
or unchanged retry. Whole contrast OPTIONAL-FIREFOX-CONTRAST.json. DoubleSpace
latency106.802median/137.423p95ms,33/60 measured,27missing; not an all-client speed claim.
Captured one failed field2 rapid second pair in bounded ring: marker nonce53
sees unsettled callback741, bridge rejects with18.494ms left in20ms; same Space
release then enters/settles normally. No replay delegation for this failed gesture.
OPTIONAL-FIRST-BRIDGE-REFUSAL.json SHA da9bef0a41bf50b7c183d6b845e279653ad6e4db1abe9502fcd83d53e5bac806
under td155-install89-20261017; trace completeness UNKNOWN. No admission edit yet;
all lostPASS causality UNKNOWN. DefaultFirefox216 now running; standard normal/
private40/40PASS is partial evidence only. Installed88/default349P50F33B preserved.


TD155 new-byte default Firefox216 COMPLETE170PASS/27FAIL/19BLOCKED,
0NOT_TESTED, all9jobs and cleanup valid, no lostPASS or gains vs whole88 REPAIR2.
Native DoubleShift420/420:98.465median/111.253p95ms; functional134/210 measured,
76missing, retained separately. BASE td155-install89-20261017/native/firefox/
RESULT.json and firefox-vs-REPAIR2.json. Chrome216 now running; latest complete
both-browser table stays old349P50F33B until new Chrome whole completes.
CI38972539257 and Security38972539134 exact8562c2bb firstattempt SUCCESS,
BASE/CI.json. No source edit, v89 release/install still pending; installed88.


TD155 new-byte default BOTH COMPLETE349PASS/50FAIL/33BLOCKED/432,
0NOT_TESTED. Firefox170/27/19, Chrome179/23/14; whole comparisons both0lostPASS/
0gains, all18jobs COMPLETE, no primary splicing. BASE td155-install89-20261017/
native/{firefox,chrome}/RESULT.json and browser-vs-REPAIR2.json. Native standard
Firefox420/420:98.465median111.253p95ms; Chrome406/420:82.182/98.873ms,14missing.
Functional/other timing coverage remains separate in full receipts. Firefox
optional40 already27P10F3B; Chrome optional40 now running on same exact build.
No release/install yet, no source changes after final3211 or first-attempt CI.


TD155 new whole native432=349P50F33B,0lostdefaultPASS, optional80=64P12F4B
(FF27/10/3, Chrome37/2/1),9observedgains/3lostoptionalrapidPASS recorded explicitly.
Chrome DoubleSpace60/60 observations98.756median118.154p95ms; both remaining
ChromeFAIL are field2 Backspace-return extra raw`zoom`, not failed flips; third
case undo-second seed BLOCKED. Entire native/CI/review gate qualified and owned
finite seats CLOSED. First install FAILED panel version and automatically restored
six88 files, recovery VERIFIED; loaded old hashes independently confirmed.
Root cause exact APP_VERSION='1.0.88' while Cargo/metadata='1.0.89'. Fix only exported
version literal, preserve3exact native-tested binaries/all old receipts. ADR
2026-10-17-panel-version89-metadata-alignment.json and PANEL-VERSION-ALIGNMENT.json
pin old/new bytes; private aligned installer requires full exact parity with that
single reversible literal difference, fresh output/backup, original limits intact.
Metadata graph refresh/package review pending; no rebuild/functional retest needed.


TD154/155 final installation1.0.89 VERIFIED, publication pending. Whole new
native432349P50F33B with0lostdefaultPASS; separate normal80=64P12F4B,
9gains/3lostFFrapidPASS and all prior receipts retained. Source3211/3211,
source review9/10 pass2/2; metadata package review9/10 pass2/2 is separate.
Metadata canonical fy4002ss PASS48.443s:992exact inputs+one APP_VERSION literal,
3core binaries unchanged; graph/links/decisions valid,0newfunctionaltests/keys.
BASE td155-install89-20261017/install-aligned/INSTALL.json verifies CLI/daemon/
IME/panel89, loaded IMEb69b4ba0/daemon9b9f7dca,23protected files and identical
globalIBus/L1.1/L3 process identities, config, sources and RUmode. Initial failed
INSTALL.json remains immutable with verified88 rollback. No delivery/model/input
route changes or localfunctionalinput. All owned remote finite seats CLOSED.
Source/native mechanism scope complete; retained compatibility failures belong
also to TD121 and do not become PASS. Source-only public tag/release next.


Publication verified 2026-10-16T22:44:16.720061+00:00: source-only tagv1.0.89 ->
e9d40144fff79c59ea7189599b6957b2cec326bb, public releasehttps://github.com/radislabus-star/lay-public/releases/tag/v1.0.89.
Normal publicmain+both working-branch pushes/readback, private origin/main untouched;
no force/delete. #46 material updatehttps://github.com/radislabus-star/lay-public/issues/46#issuecomment-6226098048
posted once and exact body read back. #47 remains NEEDS_REPORTER_RETEST; prior
#59/#60/#43/#44/#48 closures unchanged. Source/runtime work DONE for TD154/155;
TD121 known compatibility limits remain OPEN. Only publication metadata changes
follow the exact tested core; no repeated tests/builds/input/install/restarts.
BASE/PUBLICATION.json, METADATA-PUSH.json, ISSUE46-REPLY.json.

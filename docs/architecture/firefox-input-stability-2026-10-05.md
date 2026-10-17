# Firefox input stability — 2026-10-05

## 2026-10-17 — PR53 explicit replan for hot readout fixtures

CI39074346533 at `e3557fb5` passes the repaired handler-first cursor Reset
case, but fails `td121_inactive_completion_waits_for_matching_handled_release`
with no selected suffix after release. Four other checks pass. Its trace does
not identify the release-order arm or cache state. No runtime defect is proven.

The inspected permitted failure path is a missing hot-material precondition:
the fixture primes a generic query before engine setup, whereas synchronous
release/receipt publication requires cached material for the actual current
scene/input. On a miss the existing runtime correctly schedules background work.
This differs from the prior Reset transport-order hypothesis. The managed
release fast path intentionally supports handler completion before ingress;
its helper, routes and deadlines remain unchanged.

After the prior two-pass review, this distinct failure receives an explicit
bounded replan (independent plan review9/10). Group single-release, overlapping
release and exact-SSI hot-material fixtures. A helper inside the existing
cfg(test) readout contract validates the actual stored computation frame or
existing pending-reset frame, invokes production materialization, and verifies
the actual cache hit with unchanged input/stored frame. No publication or
learning is granted. Overlap retains its stronger display-identity checks;
cold-worker/refusal scenarios and the prior Reset fix remain unchanged.

Fresh remote IME verification passes827/827 (134.7s end-to-end).
A private controlled omission of release-time draining makes the prepared
fixture RED at its publication assertion. Exact production-byte restoration
returns PASS. All three hot fixtures, the cold-worker, revoked-frame,
unproved-release, prior Reset, unobserved-Reset and absolute-timeout controls
pass:9 exact positive/control invocations after the single expected RED.
This proves sensitivity to a controlled violation, not a historical CI cause.
`READOUT-FOCUSED-PASS.json`, `READOUT-FIXTURE-PROOF.json` and
`READOUT-CONTROLLED-VIOLATION.log` retain exact identities and effects.

`READOUT-SOURCE-REUSE.json` verifies all Rust, dependency and configuration
bytes outside the two private IME test files unchanged from the prior3211
full gate. This composes fresh827 evidence with unchanged other targets;
it is not a new remote full execution. Fresh full current-head CI remains
mandatory. Exact receipts and both failed CI logs remain under
`/home/ubu/.cache/lay/development/pr53-serde-20261018/`.
Installed89 and runtime authority are unchanged. Historical browser default432
remains349P/50F/33B, NOT_TESTED0; new dependency/native acceptance NOT_TESTED.

## 2026-10-17 — PR53 source-fixture receive-order repair

At serde update head `8af0b6b6660ec7868512a3c0551572b23afce29a`, remote
development correctness/package lanes passed 3211/3211. GitHub run 39071732596
failed only the handler-first cursor Reset inert-phase assertion. The helper
used concurrent `zip` scheduling against the unchanged 1ms callback deadline;
transport scheduling is the hypothesis, not an independently timed CI trace.

The test now polls the real Reset once to Pending, completes the actual
canonical observer, then resumes that same future. All matrix variants,
manual-only effects and refusal assertions remain. This checks semantic order,
not 1ms transport performance. Runtime rendezvous, provenance, deadlines,
delivery routes, guards and installed 1.0.89 bytes are unchanged.

Remote focused IME verification passes827/827 (137.3s end-to-end).
The private controlled violation removes only pre-Reset live-source capture:
the same regression fails at its inert-phase assertion, then passes after
exact production-byte restoration. Unobserved Reset refusal and absolute
timeout/no-waiter-leak controls both pass on that restored binary. This is
an invariant ablation, not an exact historical source revision. Its complete
compile/run proof took22.894s; these durations are not input-latency measures.
The subsequent full remote correctness/package gate passes3211/3211
(478.5s end-to-end), with no semantic or infrastructure failure.

Exact receipts: `/home/ubu/.cache/lay/development/pr53-serde-20261018/`,
`FOCUSED-PASS.json`, `FIXTURE-PROOF.json`, `CONTROLLED-VIOLATION.log`,
`REPAIRED-SOURCE-PASS.json` and `repaired-SUMMARY.json`. Full gate local receipt
is `/home/ubu/.cache/lay/development/run-ipc8rpn9/RESULT.json`.
Final documentation/generated-graph publication is metadata-only relative to
that tested Rust/lockfile snapshot; current-head CI remains the merge gate.
The original `SOURCE-PASS.json` and `CI-FAILED.log` stay historical evidence.
Browser default432 remains349P/50F/33B, NOT_TESTED0 in that retained native
receipt; this source/dependency change has no new native acceptance.

## W repeated-source proof: configured54 PASS; delayed erase release coverage next

First W54 test53P1F did not prove the replay was active before SSI and used
factory default config rather than native helper's IME config; it is not a
causal native reproduction. Restoring the correct fixture configuration and
asserting active replay before SSI yields54/54 PASS,23.965s, signature
`7345b8908d6cf3ef8408fb016e38c8baf454dc835b3e1dbb8305b487f0062460`.
No W production change. Repeated confirmed UnknownStart source plus original
SSI and delayed erasure SSI alone do not reproduce native failure.

Actual fullscreen trace further shows last Backspace press30716, Reset/oldSSI
30717–30720, then last Backspace release30721. The new source schedule now defers
that release, using actual callback/observer receipts and unchanged deadlines;
54 controls run remotely. No diagnosis from missing trace windows or late
field statistics. Native339P60F33B remains current, bothGUIclosed/VT1; no models,
routes, laptop changes or publication. W decision retains receipts and limits.


## V both complete:339 PASS60 FAIL33 BLOCKED432; reject whole-version promotion

Firefox160P37F19B; Chrome179P23F14B, exact432 stable IDs. Chrome all216 statuses
unchanged vsU. Firefox one gain(privateSpace7) andthree lostPASS(fullscreenSpace9,
functional moved_boundary6, space_en_ru5): nettwo extraFAIL vsU341P58F33B.
All20 cursor fields remainPASS. Initial321P78F33B stillhas18fewerFAIL overall,
but that does not excuse V required-class regressions. Candidate is rejected as
complete; retained as a diagnostic source while causal mechanisms are resolved,
no public promotion/push or blind rollback/unchanged retry until randomPASS.

Source53+842PASS/AST737unchanged/build proofs remain scoped source evidence.
Both native originals9once/parents0/noheldkeys; exactallfive process-starttick
pairs gone/cgroups empty/VT1. FF715.986s, Chrome695.899s. BothRAW manifests and
all18 primary JSON hashes verified locally: `V-both-primary-retention-verified.json`.
Chrome proof SHA`cd4b7e72bb965ed4cbd629df12817b2e6a12153c8b9b99602a17e879243adde4`;
FF proof SHA`ba5347ff2444188d16e9816c01e5439f2d54746858cc030357cf89f9ce19547e`.

Chrome native write-return→AT-SPI p95 standard109.723ms406/420,
panel115.303ms48/48, functionalDS128.913ms158/210, Space23.243ms38/50,
selection21.532ms10/10; Tab0/10 missing. FF standard118.521ms418/420,
panel120.636ms48/48, functionalDS140.422ms122/210, Space23.356ms35/50,
Tab/selection0/10 missing. Kernel releaseNOT_MEASURED; no claim of statistical
speed improvement. Chrome visitedLay3private sinks unchanged/publicrecent_actions
positive changed, allfour observer snapshots retained, tracecoverageUNKNOWN.

Next first-loss proof: fullscreenSpace9 had source7SSI immediately after exact
suppression; observed native erasure7→0; delayed erasureSSI6→5 after mirrorempty;
emptyactualSSI arrives afterthree replacement letters. All later Reset captures
lack provenance, before full finalclient7/7. This differs from V private target
caret-lag loss. First scope-clearing call remainsUNKNOWN until source proof.
Two functional Space rows retain unchanged literal input; their capped512k trace
containszero metadata timestamps in the failure windows, so causeUNKNOWN; no
invented candidate/ranking/training diagnosis. Independent reviewNOT_RUN/thread
limit; TD121OPEN. No model fit, source-level literal word exceptions, routechange,
laptopinput/install or RDP. Detailed owning decision and raw localbdir preserved.


## V Firefox complete:160 PASS37 FAIL19 BLOCKED; Chrome running

All216 stable IDs verified against U; exact216 changes: private/Space/field7 FAIL→PASS;
fullscreen/Space/field9 PASS→FAIL; functional moved_boundary/field6 PASS→FAIL;
functional space_en_ru/field5 PASS→FAIL. Allten cursor cases retain PASS.
Thus V is not accepted/promoted: restoring one selected case cannot hide three
required-class regressions. No unchanged repeat or blind rollback; all source
and native evidence retained. Full9 jobs once/parents0/noheldkeys,715.986 s;
actual99325 anddriver3337 closed0, exactfive PIDs gone/cgroupempty/VT8→1.

Proof `/home/ubu/.cache/lay/development/remote-firefox-setup-j2he3qf6/native522f2-firefox-proof.json`,
SHA `ba5347ff2444188d16e9816c01e5439f2d54746858cc030357cf89f9ce19547e`.
Standard59P1F, panel6P, functional95P36F19B. Native write-return→AT-SPI p95:
standard118.521ms (418/420), panel120.636ms (48/48), functional DS140.422ms
(122/210), Space23.356ms (35/50). Tab0/10 andselection0/10 missing; kernel
release NOT_MEASURED. Scope and missing samples remain explicit.

Original18 primary receipts retention proceeds; Firefox RAW manifest immutable.
Chrome now same V bytes in new CG155510, GNOME155739/Chrome156343/IBus156458/
IME156538/daemon156783 (exactticks VB-Chrome), event11 owner155542/t15516929
ACL granted immediately. Actual25438/driver33161, original100–108 once, four
private/public read-only sink brackets. No heavy work whileGUIalive. FullU
341/58/33 table stays previous complete both-browser result until Chrome ends;
independent V review NOT_RUN/threadlimit, TD-121 OPEN/no publication.


## V private candidate live; original Firefox matrix running

Remote AST20.836 s with737 Rust unchanged; allfour artifacts fetched/hash-verified.
Immutable gated build183.803 s: IME
`522f232d88cdabe3af4f4bfc383822650ee3f05ac3496a75feb59b34dbb076ad`,
daemon `c15d36013a7b48260cb16336cbad2d2927b7e6cca76830cde737a91082996e2c`.
Only private RUN/bin staged; backup `staged-before-V-native-progress-1791956684983127385`.
Source gate842 PASS; original actor hashes unchanged; no models/routes/laptop change.

Finite owned seat-ag PID102037/t15421544/session3757/VT8, expiresUnix1791960185.251534.
Actual native scope103649, GNOME103863, Firefox104308, IBus103955, IME104209,
daemon105236, exact ticks in `VB-firefox-zero-key.json`. Virtual keyboard event11
owner103670/t15425042 ACL granted immediately. Original051–059 queued once via
wait-V-firefox-setup-and-matrix; actualsession99325/driversession3337. Complete
five-process/cgroup closure and VT1 restoration required before Chrome/build.
No heavy work while this GUI lives. Table341/58/33 is previous completed U432,
not a partial V score; independent review still NOT RUN and TD-121 OPEN.


## V scoped source842 PASS; immutable private candidate build running

842/842 PASS (IME791 + six contracts51), worker117.237 s/controller125.5 s.
All737 Rust file hashes match the immutable source archive
`b80393ad176658954fbb46dee1ca9a27317b05e3fceafa3f06275fb7aa03f2f7`.
Local `/home/ubu/.cache/lay/development/run-jmmjw42b/RESULT.json`, remote
`/home/e/projects/lay-development-runner/run-e4XAFt`. R daemon273 proof is reused
on unchanged source and is not a new execution. This is scoped development proof,
not full release/heldout/physical-client acceptance. Source retains the original
manual routing; only inert actual replay observation retention remains.

Remote guarded AST and candidate build now use this fixed source. No GUI active,
private U runtime unchanged, laptop untouched, no model fitting. Independent V
review NOT RUN (thread limit); native341/58/33 remains current; TD-121 OPEN.


## V canonical focused53 PASS; only inert observation retention changed

Final 53/53 PASS, 23.901 s, all 208 Rust hashes verified; signature
`cef52d3805060d185cc64a325e389215fb4d08c9c0a4da5b19f969a0f1e260a7`.
Original KnownStart/manual typed-routing code remains unchanged. New adverse
proof checks actual source refusal, no handoff/target receipt/live native
provenance or output effects. Spent suppression presence is inert protection
and is not mistaken for a new source or delivery authority.

Eight positive schedules and ten adverse schedules pass with the existing
owner/reducer/full-final-receipt requirements. Source gate reruns because the
rejected early guard was removed and final safety assertions now test the
canonical stage; no required canonical test weakened. Current native341/58/33,
private U bytes unchanged; independent review NOT RUN (thread limit), TD-121 OPEN.
Receipt: `/home/ubu/.cache/lay/development/remote-firefox-setup-j2he3qf6/V-canonical-final-green53-RESULT.json`.


## V source gate rejects early guard; canonical source boundary retained

842 selected: 841 PASS / 1 FAIL, 125.6 s, receipt
`/home/ubu/.cache/lay/development/run-cf_cq3rn/RESULT.json`. Existing canonical
`exact_replay_requires_current_unselected_external_tail_at_each_lease` requires
V3 typed route `(3,false)` even for contradictory text; actual source capture
then refuses authority, handoff and replay. The new early KnownStart veto moved
that boundary to the wrong stage, so it is rejected and its exact prior code
restored. No canonical test changed, no new binary or installation.

Earlier baseline V3=3 alone did not establish an unsafe edit or source. The new
adverse fixture now asserts actual V2 source refusal, absent published handoff,
absent replay scope, absent native ACK and no Commit/Delete effects. This corrects
a new fixture's mistaken routing-as-authority assumption rather than lowering a
production guard. The inert actual replay observation repair remains in place.
Independent review still unavailable; native341/58/33 unchanged; TD-121 OPEN.


## V focused proof complete; scoped source gate next

Remote 53/53 PASS in 23.835 s, signature
`ac95b8aadbdda390c86980b2a936a082e57792b007cb342b8a8db52ec38044e9`,
all 208 Rust hashes verified. Eight positive schedules cover prefix, source
Space/NBSP and real Reset between replacement letters; ten adverse schedules
cover absent actual erasure receipt, foreign/future text, selection, caret before
prefix, actual focus/factory lease/layout changes, broken paired scope and private
metadata. Full actual final receipt is required before the real V3 delegates.

Current change retains one actual adjacent replay observation as inert metadata
bound to existing paired scope/revision/focus/layout. It is spent at the next SSI
and purged on scope retirement/revocation/new arm, field close and metadata
revocation. It never becomes current client text. Existing KnownStart manual
admission now respects a contradictory current document/caret observation.
No delivery route, model fitting or laptop runtime change.

Independent context-free V review: NOT RUN; spawn rejected with `agent thread
limit reached`, no score fabricated. Source proof is not independent review or
physical acceptance. Receipt: `/home/ubu/.cache/lay/development/remote-firefox-setup-j2he3qf6/V-final-green53-RESULT.json`.
Native matrix remains 341 PASS / 58 FAIL / 33 BLOCKED of 432; TD-121 OPEN.


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


## R final independent code review9/10; final source gate running

Review2/2 closed9/10: no production blockers identified; no third review.
The same bounded pending receipt keeps manual-only proof and UnknownStart,
old erasure typed phases remain distinct, and the existing Drop spends failed
consumer/target/output authority. The review confirms the missing fixture
registration barrier; it does not claim historical callback replay proof.
Final focused44/44 PASS,18.9666s, all208Rust hashes match; receipt
`/home/ubu/.cache/lay/development/remote-firefox-setup-j2he3qf6/R-registration-green44/RESULT.json`.
The new registration oracle remains alongside all old verdict assertions;
Cycle09 and metadata transitions reuse the existing strict Ping/reply barrier.
Fresh broad source gate is still running. Build/staging/native acceptance
remain pending; no runtime or delivery route change for the fixture repair.


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

## Accepted surrounding receipt revision — 2026-10-12

[Decision](decisions/2026-10-12-accepted-surrounding-receipt-revision.json):
the existing consumer now advances its baseline after every accepted actual
full SSI. Repeat confirmation compares all adjacent text/cursor/anchor; both
legs of an inert strict-prefix echo compare the complete raw prefix and require
the end of the whole snapshot, so right external context cannot appear or vanish.
One phase flag in the same pending record distinguishes confirmation carried
through an authenticated owned append (whose old snapshot is intentionally
invalidated) from already observed full confirmation. Only that producer sets
the flag; accepting the full client reply clears it. Generic Reset/erasure do
not set it, and the manual consumer still uses its existing one-shot token.
No new snapshot owner, mirror receipt, delivery path or client-specific rule.

Causal RED715/717 isolates the two new failures
(`/home/ubu/.cache/lay/development/run-qqizvac0/RESULT.json`). Intermediate
experiments and rejected assumptions are retained in the decision. GREEN
**719/719 PASS** (`/home/ubu/.cache/lay/development/run-4q6g4h4z/RESULT.json`),
archive `f12d2da7ac5e6927b39bde9ae896c6124c6a94990517c584c65344cacbb3f744`.
Source review pass1 scored7/10: external-context changes could bypass the
full comparison through the prefix echo. Root guarded both legs and added
context-change negatives. Final causal source GREEN719/719 also covers missing
observations on either echo leg (`/home/ubu/.cache/lay/development/run-brcnk01b/RESULT.json`,
remote `run-RUhDB0`, archive
`525495694fab3a2f1768c6fc82142acf4c57ca6dbef847efc46c6beaf6cd0a94`).
Review pass2 also scored7/10 and exposed changed right context through an
echo. Actual callback causal RED718/719 (`run-9q4rxy3u/RESULT.json`) confirms
that blocker. Root replanned the echo to end-of-whole-text on both legs and
added right-context appearance/disappearance refusals. Replan GREEN719/719
(`run-qejdztou/RESULT.json`, remote `run-YNL1J1`, archive
`01cdd8ebb522763ce89293659f3f80504ddde47016836e7604e9ca8722d06663`).
The two review passes are exhausted; no third review is claimed. Changed-byte
native acceptance remains pending. The measured56/60 native result below belongs to
the preceding candidate, not these changed bytes. Laptop authority changed:
**false**.

## Current remote browser evidence — 2026-10-12 16:42 UTC

Status: **OPEN_CURRENT_SCOPE / SOURCE_PASS_NATIVE_PARTIAL**. Frozen source
archive `a0f1d5b3283dcd969d1a9d94c30e5f6966019a4c1aa1223bc2a17b23541d51e8`
passes715/715 focused IME tests; fresh source review9/10 has no blockers.
Private candidate IME SHA256
`49c7f5d9b632563deca390e8cfdeb85419b641fbeac8a96a286c334e0325ac59`, daemon
`a1e80130e5afceba2c99bdcd09b669f525f62e38fbd9820b55c614423e302d3f`.
All native receipts below belong to isolated remote run
`/home/e/projects/lay-development-runner/firefox-legacy-native-20261012.6onP5c/`,
official Firefox157.0.1, GNOME50.1 and stock private GTK IBus module.

The independently seeded shared standard records **56/60 PASS,4/60 FAIL**:

- `firefox-standard-normal-lyhnvndi/RESULT.json`:20/20 PASS.
- `firefox-standard-private-f2zhkoua/RESULT.json`:19/20 PASS; field9 active
  fifth burst gesture misses its target.
- `firefox-standard-fullscreen-x3kwptwr/RESULT.json`:17/20 PASS; field4 active
  first burst gesture, field7 after-Space second ordinary inverse, and field8
  after-Space third burst gesture fail.

Each ordinary and five-gesture scenario begins with a fresh empty field/RU
seed. Targets preserve full raw closed text; active preedit uses exact typed
prefix/caret with unaccepted display suffix retained in the receipt. Native
Ctrl+Shift+P proves private-window creation; F11 and exact monitor bounds prove
fullscreen. Owned mode cleanup succeeds without changing production input.
The older combined `firefox-standard-qualified-normal-uupeg6lb/RESULT.json`
(11/20 PASS,9/20 FAIL) retains an additional continuity failure but lacks fresh
seeding between ordinary/burst scenarios; it does not replace this denominator.

`firefox-c09-c10-s_86d2iv/RESULT.json` observes actual visible GNOME Lay RU/EN
labels, next-letter decoder and Backspace after eight individual pairs in
input/textarea/rich, active and after-Space: **5/6 cells PASS,44/48 transitions
PASS,4/48 missing**. Plain input after-Space fails at pair5; all other cells
pass. Measured evdev write-return to AT-SPI target latency: median108.873ms,
p95116.810ms, max124.166ms (44 observed transformations). Kernel release timing
is **NOT_MEASURED**. Async production-trace completeness remains **UNKNOWN**.

Fullscreen field4 loses its pending receipt on a later short SSI after an
accepted full SSI; the accepted callback leaves `armed_revision` unchanged.
Its text bytes are not present in the production trace, so a strict-prefix
echo is still a hypothesis until passive wire evidence. The other replay
failures likewise require raw separator/provenance evidence; equal lengths
do not authorize correction. No generic mirror recovery is permitted.

Stock Chrome155.0.8059.39-1 launches privately but exits-11 before a test
window, with GtkSettings missing `gtk-modules`; Chrome native matrix and
functional corpus are **NOT_TESTED**, environment diagnosis remains open.
Firefox functional corpus is also **NOT_TESTED**. Native evdev diagnostics do
not certify human hardware, laptop Snap parity or all user windows. Installed
laptop bytes and IME delivery routes changed: **false**. TD-121 remains open.

## Exact IBus dedup root and owned-return implementation — 2026-10-12

The Backspace-notification experiment still gives4/4 native second-inverse FAIL
(`firefox-first-word-native-2jn246cv/RESULT.json`). Source review9/10 and2/2
stub tests establish C correctness only. Wire receipt
`frontend-proof/native-dedup-wire.json` proves fresh identical GTK replies
reach IBus InputContext after the final erasure; stock IBus1.5.34-rc2
`bus_engine_proxy_set_surrounding_text` forwards only changed text/cursor/anchor.
The prior inference that Firefox did not supply text was too broad. Root
reverted the ineffective C/test extension and retained its exact patch/receipts.

[Owned-return continuity decision](decisions/2026-10-12-owned-preedit-return-continuity.json)
retains the original actual confirmed observation only across an authenticated,
unchanged-owner local preedit/cancellation chain (9/10). It does not fabricate
a new receipt, permit matching-mirror recovery, promote generic KnownStart or
remove verifier checks. Patching stock IBus (5/10) is disproportionate; generic
old-snapshot recovery (1/10) is rejected. External Reset/native deletion still
require fresh independent evidence. Source RED713/714 fails only the new no-SSI cancellation delegation
(`/home/ubu/.cache/lay/development/run-2y6qfi2c/RESULT.json`). Implementation now
retains the actual original observation with its unchanged revision only after
the exact settled cancellation. The independent external-Reset refusal and
existing contradictory-receipt guards remain in the source proof. Focused source
GREEN714/714: `/home/ubu/.cache/lay/development/run-9o8jqgpm/RESULT.json`.
First fresh review8/10 found proof gaps rather than a production defect. Root
added no-SSI interruptions before final cancellation and compares the actual
original snapshot/revision from before the first owned key. Their gate and
second review finished: source715/715 PASS
(`/home/ubu/.cache/lay/development/run-uasvr98v/RESULT.json`), fresh review9/10
with no blockers. The missing-original-observation negative correctly refuses
managed preedit setup; the other eight cases interrupt an actual open preedit.
Native acceptance and all full matrices remain open. Runtime authority changed on laptop: **false**.

## Superseded GTK Backspace notification experiment — 2026-10-12 15:31 UTC

Status: **OPEN_CURRENT_SCOPE / SOURCE_PASS_NATIVE_FAIL**. Final713-test source
candidate IME `68407cc5e4c7f48be2480b85c3174565d8fc2554da01b201dc9022b10bd3a29e`
and daemon `ee022e4da8e1513aa18f766618b9418d08895c8c1c9ed46d1c41afec452bae30`
are staged only in the existing isolated remote run. Native receipt
`firefox-first-word-native-m27wxizw/RESULT.json` shows first inverse, exact
next-letter decoder and full raw Backspace return in all four fields1/7/8/10;
all four second inverses FAIL. `RequireSurroundingText` is emitted after the
proved erasure; no following SetSurroundingText arrives. The first lost
transition is now frontend notification, rather than predecessor capture.
`firefox-first-word-native-2m04kq42/RESULT.json`: four completed sequences,
five causally distinct alternating gestures each, PASS with final decoder and
Backspace. This focused result does not close the browser gate.

`firefox-standard-normal-mcxl13dw/RESULT.json`:20 declared cells,0PASS,
10FAIL after Space,10BLOCKED active. Active setup sees raw `приветное` with
caret6: the typed word is followed by an unaccepted preedit completion. This
observer setup failure must be qualified using an exact typed-prefix/caret
witness before rerunning; it is not a runtime PASS. Private/fullscreen,
Chrome, C09/C10 panel icon and kernel-release latency remain NOT_TESTED or
NOT_MEASURED. All three receipts are under
`/home/e/projects/lay-development-runner/firefox-legacy-native-20261012.6onP5c/`.

Pinned upstream IBus1.5.34-rc2 GTK source disconnects its
`require-surrounding-text` handler after initial retrieval. Its native setter
accepts a newly allocated IBusText even when committed text is unchanged.
The existing Lay GTK adapter supplies printable-release and Reset requests,
but excludes Backspace (Unicode0). The next bounded experiment is specified
by [the Backspace notification decision](decisions/2026-10-12-gtk-backspace-surrounding-notification.json).
It adds one notification on an outermost handled Backspace release, with the
same module/modifier filters and original GTK return. It grants no authority
and leaves the existing retrieve/SetSurroundingText route and exact IME
validator unchanged. Installed laptop runtime authority changed: **false**.
C-adapter source RED rejects the three newly asserted Backspace notifications;
GREEN passes both existing test methods with all positive/negative stub cases.
Receipts: `frontend-proof/RED.json`, `frontend-proof/GREEN.json` in that run.
Native acceptance of the changed adapter is still pending.

## Current owned Space boundary diagnosis — 2026-10-12

Status: **OPEN_CURRENT_SCOPE / SOURCE_FOCUSED_PASS_NATIVE_PARTIAL**. Baseline
source `d362e7c82cb14f3d2aab123b9130707f1843ba02`, isolated remote IME
`5fe100db732bcc945d9c7d584a2c22bd06eb93c19bee09cc39788c85470dee7a`
and daemon `8cbbfb83fbb7f2e0812632b0358c7ab312182e352516a73ec1f040c11df54238`.
On official Firefox157.0.1 / private Ubuntu26.04+GNOME50.1 with the stock
GTK IBus module, rich field7 reports raw `привет\u00a0`, cursor/anchor7. Real
SetSurroundingText also contains U+00A0; the locally owned Space expects
U+0020. The first failed transition is `surrounding_receipt_mismatch`, then
`ManualToggleV3/context_authority`; daemon recognizes the physical Double Shift.
This is a remote native evdev diagnostic, not laptop Snap parity or 60-cell
acceptance. Receipt:
`/home/e/projects/lay-development-runner/firefox-legacy-native-20261012.6onP5c/firefox-first-word-native-cbclqcoi/RESULT.json`.

Plain field1 completed its first inverse in105.8376ms on the evdev write-return
clock; kernel release time was NOT_MEASURED. Its recorded FAIL_NEXT_LETTER_DECODER
is an observer false failure: raw `ghbdtn apt`, caret8 has the correct typed
`a` before an unaccepted `pt` completion. Original receipt is retained unedited;
this annotation does not convert it to full browser PASS. Earlier diagnostic
without the stock GTK IBus module measured a fallback frontend, not this route.

[Decision](decisions/2026-10-12-owned-space-client-boundary-ack.json)
keeps existing IME delivery and exact validators. Source RED:710/711 (initial
raw Space acknowledgment), then980/982 (native replay preflight and final ACK),
receipts `/home/ubu/.cache/lay/development/run-3e6e7_3y/RESULT.json` and
`/home/ubu/.cache/lay/development/run-plsx71oz/RESULT.json`. First fresh review7/10
found the downstream replay gap; implementation now covers the exact source
lease, unchanged uinput Space, paired complete-replay acknowledgment, next toggle
and prefix/selection/caret/focus/content refusal. Final IME gate711/711 PASS (`/home/ubu/.cache/lay/development/run-c06md18z/RESULT.json`);
daemon271/271 PASS (`/home/ubu/.cache/lay/development/run-_28jumzx/RESULT.json`, unchanged
daemon production source). Review pass2 scored9/10 before the final root-owned
completed-boundary witness selection. That final selection retains the original
receipt token/provenance and is covered by the 711-test gate. Changed-byte native
results are recorded below. Laptop installed runtime authority changed: **false**.

Changed-byte private runtime: IME
`7fa3280ac25b115675dcc26ba8a05b8df53fb4698b9dafedfe88aca7270b4fa4`, daemon
`3caa050f5b10d88a4f68bf006e8f40eeb4b8025e0129f6ca896900ce13acd974`, frozen
source archive `bbe3677207b3baa78619c01903e82974463cb4b0925d53a07eb328d550690c3b`.
Build receipt: private run `candidate-build-1791813022070284361/BUILD.json`.
Native receipts in the same private run:

- `firefox-first-word-native-_vfthobd/RESULT.json`: fields1/7/8/10 each complete
  the first inverse after Space (104.592/90.904/110.827/116.064ms). Exact KeyZ
  before the caret confirms the US decoder; Backspace restores the full raw
  target. The following inverse fails in all four fields with
  `ManualToggleV3/context_authority`. This is **4 FAIL roundtrip**, not PASS.
- `firefox-first-word-native-zrt8npu7/RESULT.json`: a fresh fixture, no intervening
  KeyZ/Backspace; **4/4 pairs of native inverses PASS** in fields1/7/8/10. Eight
  measured targets range92.008–118.035ms. Rich fields retain raw U+00A0. This
  proves the bounded acknowledgment/replay chain, not full standard acceptance.
- `firefox-first-word-native-ts3qkoa8/RESULT.json`: fixed cadence316.3–335.1ms
  between five complete gestures, no target-dependent waiting. Fields7/8/10
  observe all five alternating raw targets and finish with the correct KeyZ
  decoder/full Backspace return: **3 PASS sequences**. Field1 misses gestures3
  and5: **1 FAIL sequence**. Its unchanged even target at gesture4 is not
  independently proved to be a transformation; exclude it from successful
  transformation latency aggregates. Missing targets retain null latency.

All timings use native evdev write-return to AT-SPI observation. Kernel release
time and GNOME panel icon are NOT_MEASURED; native evdev input is not human
hardware acceptance. Original receipts, including setup failures, are preserved.
The first-word acknowledgment defect is repaired in this focused remote scope.
A distinct same-owner append/Backspace trajectory loses the closed-word receipt;
rapid repeated replay in field1 also remains under investigation. The full
60-cell Firefox/Chrome matrices, functional plans and C09/C10 are NOT_TESTED for
these bytes. Do not close TD-121 or publish this as accepted runtime. The owned
private GUI was stopped after the diagnostic; no production route changed.

The next experiment is specified by the separate
[owned preedit erasure decision](decisions/2026-10-12-owned-preedit-erasure-boundary.json).
It retains an already confirmed closed-word predecessor within the existing
preedit-start witness (9/10), rather than a broad WordScope history rewrite
(6/10) or separate history owner (3/10). Source TDD adds ordinary/NBSP returns,
an authenticated subsequent Reset and missing/selection/caret/word/bounds/
focus/content/revision refusals. Source RED712/713, only the expected erasure
predecessor assertion fails: `/home/ubu/.cache/lay/development/run-thdr7v4o/RESULT.json`.
The existing witness now carries that inert predecessor; exact final-character
owned Backspace can arm a fresh, unconfirmed receipt after key settlement.
Initial GREEN713/713 PASS: `/home/ubu/.cache/lay/development/run-a7ualtqx/RESULT.json`.
A direct pre-receipt ManualToggleV3/no-output assertion is covered by the final
focused713/713 PASS gate: `/home/ubu/.cache/lay/development/run-zxvs5asm/RESULT.json`.
Fresh-context review pass1:9/10, no blocking findings. Static scope only;
capture-settlement race injection, changed-byte native and full browser proof
remain unproved. The production change uses one inert predecessor in the
existing witness and the unchanged receipt validator; no new authority owner.


Changed erasure bytes were tested privately: IME
`e439ac5056c8a589da70b4aa833556886930537fcb374e5d0541d49b93e6d4af`, daemon
`b706f7ad2da817cca65a52e755e291c471c300f7df5c60501fc61e24e8fc6e10`, source
archive `1594e8703181f430e98437a9730a18964cd95ef3ac8341da815f1e2bf620cd4d`.
Receipt `firefox-first-word-native-j4aruwo9/RESULT.json` in the same private run:
all four first inverses and decoder/Backspace returns pass, but all four following
inverses fail. First inverses take106.213/109.978/110.396/115.907ms. The final rich
trace has no SSI or Reset between probe and Backspace, unchanged surrounding
revision9, and exact-refresh capabilities. The first missing internal witness is
not yet measured. Source713/713 therefore does **not** establish a native repair.
The next source regression includes the actual completed native replay before
starting and erasing the next owned preedit; bounded diagnostics expose witness
presence/eligibility without adding output authority.

Source with the actual completed replay probe also passes713/713
(`/home/ubu/.cache/lay/development/run-d2p83dsf/RESULT.json`). The live discrepancy
remains; a private diagnostic build is pending to identify the first failing guard.

`firefox-first-word-native-q9rq8o81/RESULT.json` is **BLOCKED** by an actor loop
mistake (`list.index(x): x not in list`) after the declared field1 sequence.
Field1 alone observes five causally distinct alternating targets in108.756–114.726ms,
with exact decoder and raw Backspace return. No field7 text is entered after the
erroneous continuation. Retain this scoped result and the whole-run BLOCKED status;
fix the one-field actor loop before a completed rerun. Debug trace rotates at
512000bytes: an old byte offset does not establish continuity or absent events.
Runtime authority on the laptop remains unchanged; the remote private GUI is
stopped for the next source gate. Full matrices and C09/C10 remain NOT_TESTED.

The bounded diagnostic bytes (IME `d187ca14560905127a55ea1183a6d0319cbf2eb26757cd69aa3cb0e15d8a600a`)
identify the next first failed transition. Receipt `firefox-first-word-native-xeuob0j6/RESULT.json`:
field7 still fails its second inverse, but captured start/predecessor/confirmed
receipt and captured erasure candidate/owned-start/owned-word/exact-refresh are
all true. Hiding the local preedit returns the raw word without a fresh client
receipt; this source path emits no `RequireSurroundingText`. The positive source
test had independently supplied that receipt. Add a source assertion requiring
one existing refresh output only after authenticated final local erasure, then
retain no output authority until the unchanged validator accepts the answer.
Immediate refresh9/10 is selected; gesture-time coordination7/10 adds avoidable
state, old-snapshot authority2/10 is rejected. No delivery route changes.

Corrected one-field actor completed in `firefox-first-word-native-c0m11833/RESULT.json`:
all five causally distinct native targets and final decoder/full raw Backspace
return pass. Keep the earlier BLOCKED actor result as history. Both diagnostics
remain narrower than the browser standard. Remote diagnostic GUI is stopped for
TDD; laptop runtime authority changed: false.

Refresh-output TDD RED710/713 (`/home/ubu/.cache/lay/development/run-4xnz0f1t/RESULT.json`):
only the three newly strengthened paths fail, each missing the required single
`RequireSurroundingText`. The callback now requests one existing refresh only
after captured final owned erasure handled the key and armed a current
unconfirmed receipt after settlement. The receipt stays inert; temporary capture
diagnostics are removed. Added Backspace release/no-repeat-output coverage.
GREEN and changed-byte native proof are pending.

Final refresh source GREEN713/713 (`/home/ubu/.cache/lay/development/run-rhe1jmp4/RESULT.json`).
The Backspace release has zero additional output; pre-receipt manual output
remains refused. Fresh-context final review and new-byte native proof are pending.
Private release build uses the exact frozen remote `run-2mNMz7/source`; archive
`90c40795fcea16061610a4049e744a50f068e8e7b476116b9cd5045d462385bd`.

Final fresh-context review pass2:9/10, no blockers. Reused fixture serial is
member-scoped and does not alias the observer registry. Capture-settlement race,
refresh-send failure and multichar shortening fault injections are not directly
covered. Candidate IME `68407cc5e4c7f48be2480b85c3174565d8fc2554da01b201dc9022b10bd3a29e`,
daemon `ee022e4da8e1513aa18f766618b9418d08895c8c1c9ed46d1c41afec452bae30`
are staged privately. Native launch waits for the shared remote verification
lease held by another L3 proof; no bypass or interruption. No laptop installation.

## Current functional browser coverage, TD-140 — 2026-10-10

The user's explicit correction is that greeting-only round trips are insufficient.
At parent `92965082038074096d82f693997e107e75fe51b7`, the first source gap is
`createRun`'s hard-coded greeting plan. It has no way to assert Space, undo,
completion, editing or focus transitions. This is a fixture coverage defect,
not newly proved production failure. Options: more words only 6/10; predefined
plans plus the existing bounded observer 9/10 (selected); new native framework
4/10. [Decision](decisions/2026-10-10-browser-functional-regression-cases.json)
extends the shared protocol/changed-file guard without changing authority.

The shared page now defines six manual corpus plans and nine fixed functional
plans in `browser_ime_cases.js`; Chrome uses exactly the same code. Coverage:
short/long/mixed case, preceding text, EN-first, Space both directions, clean EN,
correction/undo, moved-word boundary, Tab, caret/Backspace/retyping, selection
replacement and focus return followed by fresh typing and gestures. Expected
surfaces are fixtures only; they never enter product decisions or mutate input.
Retain the original 60-cell baseline and eight-gesture C10. Additional selections
follow the changed mechanism, not a Cartesian product of incompatible controls.
Separate Space/Tab timing and missing denominators from Double Shift timing.
Unknown caret/selection or missing trusted trigger stays BLOCKED. HTML does not
observe completion hint visibility, GNOME icon, IBus InputMode or native send time.

Controlled TDD RED: the new EN-first test fails the old observer at its initial
`type_ru` phase; all original 25 Node cases pass. Receipt:
`/home/ubu/.cache/lay/development/browser-functional-red-20261010/RESULT.json`.
The independent plan review rated 9/10; its scope/timing/fresh-focus findings
are included. Final remote expanded source proof PASS: 49/49 Node (0 FAIL/0 SKIP),
4/4 fixture Python, 2/2 existing compat, 15/15 canon, JS/bash/canon checks.
The model proof exercises 240 corpus plans and 90 functional plans under ten
field identities; these are abstract snapshots, not real editor interactions.
Remote headless Firefox 900×600 render keeps all ten fields visible, with zero
keys sent. Receipt:
`/home/ubu/.cache/lay/development/browser-functional-source-final49-20261010/RESULT.json`.
[Compact evidence](../../tech_debt/evidence/browser-functional-regression-cases-2026-10-10.json)
pins source/receipt hashes and separate source/native denominators.
Fresh-context implementation review pass 1 rated 7/10. Repaired all three
material findings: pending Space/Tab cannot accept manually repaired text;
focus return requires a trusted visit to another registered editor; absent
completion before Tab reports BLOCKED, while an observed Tab failure reports
FAIL. Their three new semantic regressions are in the final 49-case proof.
The other-field observation uses the same bounded record budget. Final review
pass 2: 9/10, all three material findings closed; two passes, no third.
[Review record](../../tech_debt/evidence/browser-functional-review-2026-10-10.md).
Mandatory source-bound/PASS-only publication graph receipt:
`/home/ubu/.cache/lay/development/browser-functional-graph-20261010/RESULT.json`.
TD-140 is DONE_FOCUSED_SOURCE: predefined fixture/source coverage only.
New native profile execution is NOT_TESTED. The preceding 54 FAIL / 4 BLOCKED
native cells remain unchanged; TD-121 remains OPEN_CURRENT_SCOPE.
Runtime IME 1.0.82 loaded/installed SHA256
`5fe100db732bcc945d9c7d584a2c22bd06eb93c19bee09cc39788c85470dee7a`
is unchanged. No installation, restart, delivery route or owner change.

## Initial shared standard and native burst diagnostic — 2026-10-10 (historical)

The user adopted the ten-field Firefox window as the standard Lay browser
diagnostic, requested the same Chrome window and required response-time
measurements. [Standard](../../tests/manual/browser_ime_standard.md) and
AGENTS.md require one shared fixture, separate Firefox/Chrome entries, all
normal/private/fullscreen × active/after-Space cells, ordinary round trips and
five rapid complete Double Shift gestures. Five gestures supplement the
unchanged C09/C10 eight-gesture/icon/decoder acceptance. The new
[decision](decisions/2026-10-10-browser-ten-field-standard.json) extends the
affected guard/rule paths; no production input owner or delivery route changes.

The shared observer now has roundtrip and burst5 modes. Burst5 requires ten
observed Shift taps, all five targets, the final KeyZ decoder and Backspace.
Targets cannot precede their own pair; intervening non-Shift keys invalidate
the uninterrupted burst. Partial and failed-case timing samples are retained.
The page shows median/p95/maximum and missing samples. DOM time means second
Shift keydown to first exact target; signed DOM release delta is separate.
Native timing instead uses the recorded injected release timestamp. Missing
clocks stay UNKNOWN. Source contract is25Node cases and3fixture Python checks
plus the existing2compat checks; final remote/review evidence follows below.
Current shared-fixture native acceptance and Chrome input are NOT_TESTED.

The explicitly authorized predecessor-fixture Firefox82 experiment completed
all60selected cells: **2PASS /54FAIL /4BLOCKED /0NOT_TESTED cells**. This is not
300 completed transformations: only41cells completed five native gestures;
12cells retained all five targets. Of54FAIL,39failed burst text/mode/timing and
15failed initial word setup. Normal2/17/1, private0/17/3, fullscreen0/20/0
(PASS/FAIL/BLOCKED). Input was synthesized through the already attached AT
keyboard's evdev interface; no human-hardware acceptance is claimed.
Controllers resumed untouched cells and retained every
selected failure; aborted setup receipts remain historical. Runtime IME82
5fe100db and daemon PID/starttick/loaded bytes remained unchanged.

Measured native response for60targets from12complete sequences: median
101.679ms, p95(nearest rank)160.260ms, max259.984ms. Nineteen mapped targets
missed their next-pair/last-target boundary. Intended denominator300targets;
48cells lack complete per-gesture timing mapping. Cadence:18ms down,12ms up,
75ms extra gap within a pair,90ms extra between pairs; actual timestamps are
retained. No target waits between the five gestures; last-target limit500ms.
Late correct text and absent transformations remain distinct from PASS.

First observed failing API transition: `ManualToggleV3` reaches the existing
IME context gate with a live bridge token, but `manual_toggle_allowed=false`
and `manual_refresh_allowed=false`, returning `NotHandled/context_authority`
(authority.rs653–683). Captured Reset receipts also include missing predecessor
and surrounding-receipt mismatch. Trace continuity is UNKNOWN due bounded
rotation/truncation; counts are observed lower bounds. This does not identify
the exact failed predicate for every field or establish one cause for all54
FAIL. Preserve the reducer/receipt safety rules and delivery routes; no runtime
repair has been selected or installed in this change.

Exact final receipt:
`/home/ubu/.cache/lay/development/firefox-five-shift-native-i2wy1pcr/RESULT.json`.
[Compact native evidence](../../tech_debt/evidence/td121-firefox-five-shift-native-2026-10-10.json)
pins each selected row to its actual originating capture and all four contributing
receipts, input/DOM/trace identities and limitations. Native harness review was
two passes6/10 and8/10 before root's later capture/recovery fixes; it does not
certify their final source. Product code, installed bytes, authority and delivery
routes are unchanged. Original TD121 acceptance remains OPEN_CURRENT_SCOPE.

Shared standard final source proof:25/25Node observer cases (0FAIL/0SKIP),
3/3fixture Python,2/2existing Firefox compat and15/15canon tests PASS;
JS/bash/canon PASS. Remote900×600 render includes all10fields and the new
sequence/speed controls, with zero keys sent. Fresh-context fixture review
pass1 6/10, pass2 9/10. Fixed premature target attribution, extraneous burst
keys and loss of partial timing. The final prescribed pass2 pre-release
reversal fix clears disappearing provisional targets; its25th regression is
included in the final source receipt. Two passes, no third.
Receipt `/home/ubu/.cache/lay/development/browser-standard-source-final25-20261010/RESULT.json`.
[Source evidence](../../tech_debt/evidence/browser-ten-field-standard-2026-10-10.json)
pins the final files and proof. Mandatory remote source-bound/PASS-only graph
wrapper receipt:
`/home/ubu/.cache/lay/development/browser-standard-graph-20261010/RESULT.json`.
No current-fixture native result or Chrome acceptance is inferred from source
proof, rendering or merely opening a window.

## Initial ten-field manual fixture proof — 2026-10-10 (historical)

Source checkout lay-space-boundary-shift-20261007, parent14fec28d; live IME82
5fe100db is not altered by this fixture. User requests one screen with10input
types because Double Shift intermittently fails in Firefox. Current first
failed production transition remains UNKNOWN until an exact focused capture.
Historical source/native receipts below keep their original denominators.

Selected minimal solution9/10: one owned HTML page, actual manual keyboard
input, sequential focus and an observer of both conversions/next-letter
decoding. Ten separate app windows6/10 add focus noise without better field
coverage. JavaScript-dispatched Shift2/10 cannot validate native IME/daemon
delivery. No production owner or Firefox-specific delivery bypass is added.

[Page](../../tests/manual/firefox_double_shift.html) and
[protocol](../../tests/manual/firefox_double_shift.md) separate two word modes,
current-batch summaries and user-labelled window modes. Every PASS_DOM requires
seven surface milestones including both target observations, two complete
trusted DOM Shift pairs and actual KeyZ decoder probes. Held repeats, foreign
callbacks, untrusted input, paste/drop, capture overflow and focus/visibility
loss cannot yield PASS. Unknown caret and active composition remain separate
observations; raw NBSP is preserved with render-equivalent comparison only.

The page edits only its own field during setup and changes focus only on the
operator's Start/Next action. It never projects a word, dispatches synthetic
keys, calls IME APIs, changes global layout, installs/restarts services or
cleans learning. Native keys remain daemon-owned. Browser chrome, password,
cross-origin/user-site editors, RPC plan count, GNOME icon, exact runtime bytes
and Wayland/X11 backend are outside the DOM proof. Private mode is declared,
not inferred. Missing iframe/field support remains BLOCKED, not excluded.

The layout has a compact desktop grid for a page viewport of at least900×600
at100% zoom. Smaller/narrow windows may scroll. Trusted Escape reports failure
without leaving the active field, including both iframes. The source page check
asserts actual tags/types/contenteditable and both srcdoc editors, not card IDs
alone.

New fixture source contract15Node cases and two Python checks join the existing
Firefox full/changed check routes without removing any old checks.
[Decision](decisions/2026-10-10-firefox-ten-field-manual-probe.json) records the
guard extension. Remote source proof and fresh-context fixture review are
separate from native Firefox0/10 NOT_TESTED. No new local execution grant is
implied.

Remote source proof PASS:15/15 Node observer cases (0FAIL/0SKIP),4/4 Python
Firefox checks (existing compat2 plus new fixture2),15/15 canon tests, JS/bash
syntax and protected-path canon PASS. Existing dedicated-20cpu resource guard
and heavy lease were active; no Cargo compile was needed. Exact frozen archive
SHA2563a8841da80a8d27c50ca76bb42eb4be56aa0d7e5c9fc4577d11a74839f8948db.
Receipt `/home/ubu/.cache/lay/development/firefox-ten-field-source-_un5vy17/RESULT.json`,
SHA2569ac25324fce2bb4b57998962b532ced6929f76f46e48a9d3ee156170bed4feee;
adjacent run.log retains all six commands and denominators. Remote headless
Firefox render900×600 shows all ten fields together; screenshot SHA256
3c205e035f9264959fa25aa559804bd8ff90e4922a824582957d631b709c4375.
That render sent no keys and proves layout only, not any manual conversion.

Fresh-context review pass1:8/10. Corrected keyboard-only failure reporting
(trusted Escape in the focused editor, including iframe), compact desktop
layout and semantic checking of actual HTML field types/srcdoc. Pass2:9/10,
found one last Escape edge: the reporting snapshot could complete erase_ru
before FAIL_REPORTED. Root applied the prescribed capture-without-advancement
fix and extended the existing Escape case; the final remote receipt above
includes this regression. No third review was requested. The earlier source
receipt7kkg9vpf remains historical, before this edge fix.
Mandatory graph refresh uses the existing source-binding/PASS-only receipt
wrapper; exact controller receipt:
`/home/ubu/.cache/lay/development/firefox-ten-field-final-graph-20261010/RESULT.json`.
Compact [source evidence](../../tech_debt/evidence/td121-firefox-ten-field-probe-2026-10-10.json)
pins tested fixture/guard hashes and separates native NOT_TESTED.
No delivery route, runtime input owner, installed bytes or original
Firefox acceptance denominator changed.

## Current route and goal

Source: lay-contextual-completion-20261005, commit8c5cb609. Actual installed
IME4a48c5526f62bd52555ff312d1ba9ec8ca72656f211fdbf1726c6516af93cdd1,
daemon4a61557717887e5daa5a2929641f8005d4156293d93e4c10ef147d49fad9ccef.
Current owner is ContextAdmissionReducer / WindowInteraction, manual detector
is daemon ManualToggleV3. C01/C03/C04/C06/C08/C09/C10 apply. User authorizes
Firefox repair and repeated diagnostic trials with exact rollback; global IBus,
input source IDs and input/edit owners remain protected.

Goal: retain safe text authority across legitimate Reset/key callback schedules
and prove Tab+one space, Backspace/retype/Tab, held Shift, eight individually
visible Double Shift pairs with actual next-letter/Backspace and RU/EN icon,
focus return and applied auto-flip undo in owned input/textarea/contenteditable
fields. Source PASS, installed bytes and physical acceptance remain distinct.
100-command protocol is frozen privately; user changed priority to Firefox
before the100-command run. Do not present that protocol as executed evidence.

## Established facts and unresolved boundary

Old79 physical textarea pair6 (current Reset source unchanged) refuses text
authority after replay; release41836 accepted before Reset41837. Merged
missing_predecessor_or_post_reset_token cannot establish which option or
predicate failed. Minimal controlled2/2 and extended3/3 serial regressions PASS
but do not reproduce that physical refusal. Prior success-logging diagnostic
trials did not reproduce; do not treat this as a repair.

Current79 baseline V17_RUN_20261005T210502_053fd288844e: input/textarea each
pass8word/icon pairs+nextletter. Editable fails before gestures while its
observer snapshot is stale; later state is exact ghbdtn/focused. Original FAIL
is preserved, classification is an observer limit, not a proved owner failure.
Current required composition run210730_39392a22665d: textarea/editable each
PASS8pairs+8actual letters+8Backspaces. Input BLOCKED because the pre-Backspace
snapshot cannot separate the typed probe from the changing rendered suffix.
No production change has occurred. These successes do not erase old failures.

## Selected diagnostic and two alternatives

Selected: failure-only, nonempty Reset capture/arm metadata in the existing
opt-in trace. Original predicates evaluate first; only an already rejected
branch emits diagnostic metadata. No success record, new detector, callback,
RPC, queue, timer, cache or authority is added. Capture reasons and option
presence distinguish the first unknown transition on actual callbacks.

Alternative1: repeat the detached serial fixture. It is viable for regression
closure but cannot distinguish the missing physical capture predicate; already
PASS. Alternative2: success-and-failure V2 logging. It can sample all states,
but unnecessary work perturbs the race and previous trials were inconclusive.
Failure-only logging reduces that cost. Neither alternative justifies removing
a safety check or selecting a speculative runtime repair.

## Consequence check before code

- Lattice/ranking/false authority: diagnostic cannot create/drop/rerank candidates
  or grant text rights. Return values and predicate order remain identical.
- Latency/tails: no successful capture logging. Rejected nonempty capture adds
  one bounded snapshot/JSON record through the existing logger; timestamped
  diagnostic observations are after-predicate samples, not atomic witnesses.
  Async schedules can still change; NOT_REPRODUCED stays inconclusive.
- CPU/RSS/allocation: counts/flags/owner paths only, bounded to existing state;
  no corpus/model scan. Trace append/serialization are the only extra work.
- Cache/reload/package: existing config trace cache, packages and candidate
  revisions unchanged. Future package/delta updates cannot grant extra rights
  through this diagnostic; current input hashes remain protected in trial.
- Learning: no new event/classification/weight. Physical Tab/Enter tests may
  create normal owner-controlled feedback, separated from diagnostic authority.
- Concurrency: local state sampled under the existing callback lock; no new
  reducer snapshot/revalidation/await. Preserve observer-before-handler order.
- Failure/rollback: only temporary IME bytes; exact current79 backup and fresh
  empty owned GTK field before exchange and restore. Retain stale/rejected
  attempts; no global IBus/L3/L11 restart or live-profile replacement.
- Compatibility: daemon and existing IBus contracts unchanged; same legacy
  callbacks and ManualToggleV3. Metadata has no raw user text.
- Maintenance: diagnostic is a private patch, not default release behavior.
  Remove it on exact finally rollback; a causal repair needs a new decision,
  controlled old-vs-new regression and full gate/physical proof.

## Finite proof and promotion boundary

Before trial, check exact baseline source1472 rows, sole observation metadata
delta plus owning docs/decision/generated graph, fmt/canon and guarded remote
IME compile. Bind toolchain1.97.1/research-tools, immutable artifact SHA and
backup SHA; root is sole20CPU heavy executor. No compile receipt is a fix.
One diagnostic scenario pass across all3Firefox fieldtypes, retaining first
nonempty refusal with serial, owner/token/scope/count and arm-option metadata.
If no body refusal, report NOT_REPRODUCED rather than patching speculation.
Any real repair must preserve stale/foreign owner/caret/selection refusal,
exact target retention, physical single-owner detector and verifier authority;
run changed/full release gates and obtain exact changed-byte physical proof.

Evidence/private audits live under release79/firefox-repair-20261005; source
audit CURRENT_RESET_SOURCE_AUDIT.md SHA726da33c6c983a8f604688415911c768d2ce07ebfa98c87c5798179a65a51942.

## Current measured controls after preflight

Baseline V17_RUN_20261005T212109_833b68730035:27/33 PASS. Three apparent
Backspace/second-word Tab failures stop at the pre-Tab DOM event-floor wait;
Tab was not dispatched. Actual applied-worker generations and new tail epochs
republish the same full word/cursor after Backspace; Firefox need not emit a
new compositionupdate when the rendered text is unchanged. This is a test
readiness defect, not measured Tab refusal. The other3failures produce кторое
instead of lexical correction которое; separate candidate/decision diagnosis
remains pending and is not a client ownership failure.

Prospective owned-field native-Tab contrast V17_RUN_20261005T212856_4721b211359d
PASS3/3: same scenario, exactly one physical Tab, exact word+one space, ended
composition and exact caret in input/textarea/contenteditable. Only the
diagnostic stimulus allows retained event; pre-dispatch candidate authority
UNKNOWN. Existing strict checks/old receipts unchanged. First shim run
V17_RUN_20261005T212809_20c916601458 selected0because renamed case missed its
filter; retained as zero execution, then explicit case-name binding corrected.
No runtime behavior changed for these controls.

Remote diagnostic-only build PASS:2focused tests,1520closure checks, only
declared observation/doc/generated deltas; artifact5b6878582a9e90b1a9a7a87e43c7b707a024e84531ac25d9e1b74f70faf60783.
Temporary diagnostic run REQUIRED79_COMPOSITION_PROBE_V2_RUN_20261005T212958_35fcc8b4121b
PASS3/3, each8word/icon pairs+8actual letters+8Backspaces; new prospective
probe observer retains exact typed/caret/focus proof under one KEY_F deadline.
Exact current79 installed/loaded IME4a48 restored, original mode/source/engine
and protected identities verified by trial-failure-only/TRIAL_RESULT.json.
Old Double Shift refusal NOT_REPRODUCED; no production repair selected.

Failure-only metadata nevertheless separates a nonempty rejection: local suffix3
versus UnknownStart lineage0, token matches owner and scope; candidate absent
but post-reset token present. This is a real intermediate state in a passing
trial, not proof of the old terminal failure. Next discriminant: actual callback
ordering/current recovery versus old release41836-before-Reset41837, then a
controlled production-adapter regression before any authority change.

## First failed transition established on current bytes

Current79 source4a48 production proof
REQUIRED79_COMPOSITION_PROBE_V2_RUN_20261005T213513_7c328aaa27b3: input pair5
FAIL; textarea/editable8pairs PASS. Input Reset530 observer ingress precedes
sixth native insert528; mirror6 changes, settlement528 refuses, later stamp
timeouts and subsequent exact6 cannot restore missing predecessor.
Independent temporary diagnostic input2
REQUIRED79_COMPOSITION_PROBE_V2_RUN_20261005T213857_5814afa32f8f: pair6 FAIL,
exact4a48 rollback PASS. Prefix4 press375 settled; observer queued376..380
and Reset381 precede native E377 and T379. Release376 retires harmlessly,
press377 mirror5 settlement refuses and local revoke clears stamps. Native
press379 still mirrors6 but no received callback remains. Capture rejects
local6/Unknown0 with token_scope=false, later missing predecessor token.
First failed transition is settling a received native replay insert retired by
an already observed word Reset, before capture. Changing suffix equality would
mask lost proof. These two failures share a mechanism; old release41836 before
Reset41837 remains a separate UNKNOWN, not retroactively explained.

## Repair design boundary before production code

Selected hypothesis: retain only bounded native append provenance in the
existing pending Reset receipt after authenticated same-owner Reset retires
received inserts. It stays unconfirmed and cannot edit, predict authoritatively
or learn positively. A later authenticated Reset and exact collapsed client
receipt must align the projected tail epoch through the existing reducer before
restoring observed suffix authority. No second owner/executor/detector/source ID.
Implementation is NOT selected yet: current generic revoked-key predicate does
not prove Reset origin, and unchanged-epoch soft settlement cannot accept the
projected epoch. Neither condition may be loosened. A typed proof extension of
existing owner state needs explicit admitted origin and epoch-chain verification.

Alternatives: generic retired-key skip avoids a second poison but loses epoch
and predecessor; insufficient. Delaying replay keys or adding a client RPC/
retry route would hide event-order failure and change latency/transport; rejected.
No new generation, timer, queue/cache or broad recovery fallback is justified.

Consequence analysis: candidate/ranking/model/learning routes and packages stay
identical. Only a proved Reset/native-replay state transition may retain inert
provenance. No false authority from dispatch, mirror, timeout or projection.
Unknown/foreign owner, FocusOut, selection, wrong surface, ContentType/sensitive
change, stale activation/serial/token, discontinuous epoch or incomplete replay
must fail closed. Existing verifier/SafetyGate and output plans remain unchanged.
New work must be bounded to existing callback/receipt state; no per-key IO/model
scan. Source/RSS/latency and mandatory release/physical gates remain required.
All callback evidence and current token/epoch must be coherent under existing
locks; no forced two-thread gap inside a synchronous executor segment.
Rollback remains fresh owned-empty exact installed bytes, preserving configs,
models, global IBus/L3/L11 and input IDs. Maintenance proof must reject old code
on the actual observer-first adapter schedule and retain sequential controls.

Next private experiment:4settled native inserts → observer receives remaining
key headers and Reset before handlers → actual existing native callbacks →
authenticated Reset+exact6 → probe/Backspace → ManualToggle. No token/scope
assignments. Also selection, wrong surface and FocusOut negative schedules.
Root is sole guarded remote executor; no production Rust modified yet.

## Controlled observer-first causal regression — executed

Remote `firefox-observer-first-20261005/RESULT.json`: OLD_FAILURE_REPRODUCED;
33.793s, exact HEAD8c5cb609 and 1472 baseline rows; production bytes unchanged.
Positive `firefox_observer_first_native_insert_reset_recovers_only_after_exact_receipt`
actually FAIL1/1 with expected `observer-ahead Reset lost validated replay provenance`.
Negative selection/wrong-surface/FocusOut each PASS1/1; three unchanged FULL_V2
sequential controls each PASS1/1. Compile/discovery4/nonzero/source closure PASS.
Local receipt: /home/local-user/.cache/lay/development/contextual-completion-20261005/20261004T211126Z/release79/firefox-repair-20261005/observer-first-import/RESULT.json.
This proves the original adapter schedule, not a repaired build, quality or physical acceptance.
No runtime authority changed.

## Selected causal repair, before implementation

Use the existing ExactReplay suppression to retain its already admitted source
token and bounded source epoch. The existing reducer records the last generic
revocation using its existing revocation counter; authenticated Reset alone
preserves that watermark. ContentType Set, missing input, malformed ingress,
foreign connection/profile and ordinary revocation invalidate old provenance.
FocusOut is excluded immediately by existing pending lifecycle/activation checks.
This watermark is necessary: current revocation cannot distinguish Reset from
changed ContentType; storing only equal content values misses change-away/back.
It adds no generation, owner, timer, queue or controller.

Only a received key retired by that Reset-only chain, carrying either the actual
validated native replay effect or an unchanged release, skips the second poison.
It does not settle a key, advance reducer epoch or grant word authority. The
received Reset remains available. Its actual callback may arm a completed replay
candidate in the existing pending receipt. Require complete exact plan, local/
shared identity, unexpired existing lease, original admitted token/epoch, bound
external prefix, current owner/activation and fresh collapsed exact target.
At first exact receipt the existing reducer aligns only latest_tail_epoch;
Unknown0 remains unchanged. Consume this completed witness once. Existing soft
Reset suffix and epoch equality, verifier, edit plans and output routes stay
literal. Further managed input/probe/Backspace uses the existing pending path.

Alternative retaining a per-key projected pending witness needs more state and
origin/epoch handling; completed replay reuses the existing bounded projection.
Generic retired-key skipping cannot establish Reset origin and is rejected.
Independent design audits identified expiry, ContentType continuity and pending
lifecycle as mandatory gates. Old unrelated41837 remains UNKNOWN. Required proof:
old positive RED, repaired positive GREEN, adverse selected/wrong/focus/content/
incomplete/stale/malformed paths reject, sequential and route guards unchanged.

## First changed-source focused run, rejected before promotion

Remote firefox-reset-repair-focused-20261005/RESULT.json: ACTUAL_TEST_FAILURE,
33.882s; compile PASS, 4 physical_double_shift_owner source guards PASS1/1 each.
All 7 causal/negative/sequential tests FAIL at shared legacy_key fixture observer:
its wire body was (), while separately supplied callback arguments were triples.
Strict malformed-input invalidation correctly rejects this inconsistent fixture.
No changed-byte repair acceptance or runtime change. Preserve this failed receipt.
Fix semantic fixture transport to send the real key tuple; rerun original-source
RED with corrected transport, then repaired-source GREEN. No assertion/deadline
changes are admitted. Original old RED is historical, not the corrected-input proof.
Independent review repaired concrete source defects before further execution: retain
completed witness across repeat Reset; consume source proof in local/shared scope
once after exact receipt; capture/revalidate the original admitted token rather
than mint a successor after a same-epoch revocation. Source review PASS_WITH_LIMITS,
no runtime/physical claim. Existing reducer/soft-reset/output contracts stay intact.

## Corrected real-tuple RED/GREEN, executed

Frozen OLD `firefox-observer-first-typed-v2-20261005/RESULT.json`:
OLD_FAILURE_REPRODUCED,34.780s; exact original production rows/modes preserved,
positive FAIL1 with original marker; other10 PASS. Only shared test key transport
and causal fixtures/docs differ from original. Frozen repaired
`firefox-reset-repair-focused-v2-20261005/RESULT.json`: ALL_PASS,34.931s,
11/11 actual executions: positive recovery,3 negative,3 sequential,4 explicit
physical_double_shift_owner guards. Same real key tuples and assertions.
Local immutable imports: firefox-repair-20261005/{old,green}-typed-v2-import.
Source review blockers0 after recorded repairs; still no changed-byte physical
or release acceptance. Additional non-Reset/expired/incomplete guards and full
mandatory gates remain pending. Installed exact79 unchanged.

## Additional adverse execution and fixture exchange repairs

GREEN3 actual16/17; only malformed-negative fixture timed out because it assumed
cancelled bridge refused before Ping. Actual begin_bridge_fence always sends Ping
then publishes marker before complete fence cancellation; source confirms this.
The repaired fixture responds to exact actual Ping and requires exact Cancelled,
unchanged timeout, no word rights/output/learning. GREEN4 actual17/18: all six
new adverse cases PASS; an existing strengthened malformed/read-only negative
failed its empty-output assertion because bridge marker remained undrained.
Consume only its exact signal path/interface/member/nonce; keep empty client
output assertion literal. These are fixture exchange failures, no runtime PASS.
Header-only normal key fixtures now send valid (uuu) zeros; semantic key helpers
send their actual callback tuple; explicit malformed fixtures retain unit body.
No source authority changed since corrected GREEN2. Receipts retained at remote
firefox-reset-repair-focused-v{3,4}-20261005/RESULT.json and local imports.
Final focused and release/physical acceptance are pending.

## Final focused source proof

GREEN5 ALL_PASS18/18,36.767s, corrected typed causal/6adverse/oldread-only
negative/3sequential/4physical-owner contracts; no ignored or zero executions.
Remote firefox-reset-repair-focused-v5-20261005/RESULT.json; local green-v5-import.
Original corrected-input RED remains positive FAIL+10PASS, production8c exact.
Final source closure sealed; only guarded remote rustfmt imported. Generic soft
Reset equality and SafetyGate/output routes remain unchanged. Full release gates
and changed-byte physical acceptance still pending; installed79 unchanged.

## First release-source attempt — canon refusal preserved

Remote firefox-reset-release-80-20261005/RELEASE_GATE_RESULT.json FAILED83.138s.
Fmt/fetch/discovery/fresh AST graph PASS; canonical discovery old rows literal,
new delta exactly11 tests, zero known-failure exceptions. Canon refused protected
committed_tail.rs missing from ADR path list. Its only code change initializes
source_token=None in an existing expired replay test fixture; no runtime route.
Add the actual affected test/struct paths to the existing new decision. Do not
weaken canon, reroute code or reinterpret this as release acceptance. Latest
source80, full tests/build and physical acceptance remain pending.

## Second release attempt — version declaration mismatch

Remote firefox-reset-release-80-v2-20261005/RELEASE_GATE_RESULT.json FAILED109.943s.
Canon PASS after explicit affected-path declaration. Changed gate stopped at
version consistency: source tray_support.js APP_VERSION remained1.0.79 while
Cargo and metadata were1.0.80. Update the existing declaration to1.0.80; no
Rust or installed extension/runtime changes. Source tests/full build/physical
acceptance remain pending; preserve failed logs in release80-v2-import.

## Third release attempt — nested fixture future stack abort

Remote firefox-reset-release-80-v3-20261005/RELEASE_GATE_RESULT.json FAILED276.739s.
Canon/version/fmt PASS; daemon270/270 and IME bulk458/458 PASS. The isolated
td121_aborted_second_exact_capture_does_not_consume_enter process aborted with
stack overflow before a test status; canonical verdict BLOCKED_INFRASTRUCTURE,
not semantic PASS. Exact process log retained in release80-v3-import.
The common word-scope bounded helper embeds the complete large nested caller
future inside another async frame and race; new proof fields increase those
inline frames. Candidate fixture repair boxes work in the same bounded race,
without an additional async enclosing frame. Preserve default stack, Timer1s,
all assertions/test identities and production source. No special failing-test
branch, stack increase, ignore or ledger waiver. Test the exact original abort
and all18 focused contracts before another full gate. Runtime79 unchanged.

## Bounded fixture allocation proof

GREEN6 ALL_PASS19/19,37.213s: the formerly aborting exact second-capture test
now executes PASS with the default stack and unchanged1s deadline. All18 causal,
adverse, sequential and single-detector source contracts remain PASS. Production
Rust unchanged from GREEN5; the sole additional source delta is common test
future allocation. Remote firefox-reset-repair-focused-v6-20261005/RESULT.json,
local green-v6-import. Independent allocation review PASS_WITH_LIMITS/blockers0.
Full release and physical acceptance still pending; live79 remains exact.

## Separate fast Space correction observation

Baseline79 V17_RUN_20261005T212109_833b68730035 rnjhjt cases use RU decoding
and produce кторое: this is an omission case, not layout restoration. First
measured refusal is correction lease not_ready at Space (3450/3483/3472us),
followed by managed_fallback_commit of the literal word. Editable full worker
gen13/epoch4502 completes after Space as superseded. Candidate presence/rank
remain UNKNOWN; trace.rs candidates=0 is a hardcoded telemetry field. Do not
call this an empty lattice or rank error. This repair does not change the
correction budget or authorize a late edit. Next discriminating evidence is
that same final-generation lattice/Apply/NoApply reason, separately from
publication readiness. Baseline exact logs and failures remain preserved.

## Fourth full attempt — systemic safety regression, rejected

firefox-reset-release-80-v4-20261005/RELEASE_GATE_RESULT.json FAILED471.778s;
changed suite BLOCKED_CONTRACT with exactly one unexpected semantic failure:
td121_batched_reset_callbacks_cannot_restore_invalidated_provenance, text gap.
IME667selected/1failure. Preserve release80-v4-import/process failure; neither
ledger waiver nor runtime installation is allowed. First lost safety boundary:
contradictory SurroundingText clears pending but retains local/shared ExactReplay
source_token; later Reset creates completed fallback and matching receipt can
resurrect it. Prior one-shot spend only covered confirmation, not cancellation.
Repair the existing cancellation boundary: definitive rejection spends the
current matching local/shared seed through existing consume operation, including
pending with completed_replay=None. Retain intentional prior-surface/strict-prefix
branches and repeated Reset; no blanket reset/revoke, new controller or route.
Also cancel seed on actual capability/content/input-gap invalidation. The exact
old negative's six gap variants and all prior focused proofs must pass before
new mandatory full verification. Source claim remains rejected; installed79 exact.

## Cancellation focused proof and capability review

GREEN7 ALL_PASS22/22,39.239s: original batched invalidation negative passes all
six gaps; seven prior-surface contradiction variants and positive batched
receipts pass, alongside all19 prior contracts. Runtime79 unchanged. Source
review additionally found raw PREEDIT bit could change while surrounding/exact
refresh stay unchanged, avoiding legacy-preedit derived-condition detection.
Add independent raw-bit cancellation; strengthen the original batched negative
with that seventh variant (surrounding remains enabled). Original six variants
and all assertions remain literal. This new source is not certified by GREEN7;
final focused/full gates and physical acceptance pending.

## Final cancellation/preedit source proof

GREEN8 ALL_PASS22/22,39.254s. Original batched negative now passes all seven
gap variants, including raw PREEDIT-only toggle with surrounding retained;
all previous assertions/deadlines remain. Positive observer-ahead recovery,
repeated/batched Reset, incomplete/expired/malformed refusals and four physical
DoubleShift source ownership guards remain PASS. Local green-v8-import and
remote firefox-reset-repair-focused-v8-20261005/RESULT.json bind exact source.
Only guarded rustfmt imported. Failed full v4 remains rejected/historical;
new full gates and changed-byte physical acceptance still pending. Live79 exact.

## Fifth whole-suite attempt — independent normal proof must survive gain

Before changed-suite closure IME667selected reports two failed existing source
contracts: firefox_deferred_manual_refresh_retains_retired_preedit_from_previous_prefix
and firefox_manual_toggle_defers_until_the_client_replies_after_rpc. Both enable
preedit/exact-refresh after an independently settled normal Reset predecessor;
blanket new capability cancellation discarded that valid pending proof. Reject
this source; installed79 unchanged. This establishes that raw capability gain
is not a universal field identity gap. Split existing helper actions by typed
provenance: capability/preedit/exact-refresh change retires the newly retained
ExactReplay seed and any pending completed_replay that depends on it; preserve
independently settled normal pending. Actual surrounding loss, content/input
gaps and contradictory receipts still discard all pending plus the replay seed.
The newly added seventh raw-PREEDIT variant assumed the wrong universal oracle;
replace that new variant with a dedicated observer-ahead seed-only negative,
keeping the original six variants and all pre-existing assertions literal. Add
both existing deferred-preedit positives to focused proof. No runtime word/app
exceptions, ignored tests, waived failures or weakened safety conditions.

## Typed cancellation replan and withdrawn new oracle

Full v5 failed475.050s, CHECK_CHANGED391.256s, exactly the two existing
deferred-preedit contracts above (release80-v5-import). Earlier GREEN8 is
focused evidence for rejected bytes, not promotion evidence. Withdraw only
the newly added seventh universal capability variant: terminal_delivery.rs
is now identical to HEAD, with all original six gaps and assertions preserved.
The original positive contracts remain unchanged and become focused controls.
Capability changes spend the matching added replay seed and pending only when
completed_replay=Some; independently grounded normal pending=None survives.
Definitive surrounding/content/input/snapshot contradiction still clears both.
No new owner, transport, deadline, mutation, fixture-specific runtime rule or
ledger exception. A separate appended native observer-ahead fixture must prove
raw PREEDIT retirement of this specific seed before another full gate. Current
source is NOT_TESTED; installed exact79 remains unchanged.

## Typed cancellation focused proof

GREEN9 ALL_PASS25/25,39.936s. Both original normal deferred-preedit positives
pass unchanged alongside the new native raw-PREEDIT seed-only negative and all
22 prior contracts. Old terminal_delivery source remains literal HEAD. Exact
remote firefox-reset-repair-focused-v9-20261005/RESULT.json and local
green-v9-import bind source before/after; only guarded formatting imported.
Full gates and changed-byte physical acceptance remain pending; installed79 exact.

## Transport admission failure before source execution

Release v6 FAILED0.188s at input_Cargo.lock; no clone/build/test executed.
Python tar data-filter stripped group-write bits on all seven input overlays
(0664→0644), while every payload hash matched. Preserve failed receipt in
release80-v6-import. Next fresh transport restores only declared file modes
after archive SHA and input hash validation; release admission stays literal.
This is input transport repair, not a source/runtime functional change.

## Both complete semantic gates pass; bounded proof representation replan

Release v7 FAILED921.267s: CHECK_CHANGED400.386s and full canonical2993/2993
PASS, then strict Clippy rejected the new inline admission proof enlarging
AutocorrectSuppression to296 bytes (CurrentWord24), plus unnecessary mutable
fixture borrow. Exact failures preserved in release80-v7-import and guarded
CLIPPY_DIAGNOSTIC.err; no runtime installation. Clippy/gate not waived.
Alternative1 boxes the entire existing ExactReplay variant: wider caller and
allocation migration. Alternative2 boxes only the newly added inert source
proof: preserves every existing variant/caller and proof value/equality; chosen.
Allocate the optional proof before Shared lock. Existing one-shot invalidation,
owner/revision/expiry checks and source authority stay literal. No detector,
queue, owner, deadline or output route added. Reborrow shared test harness as
immutable without assertion changes. Require focused proof plus strict lint
before another whole release gate; source/full/physical claims remain pending.

## Bounded proof representation independent review

Value/equality/one-shot review finds semantic blockers0: Box compares immutable
AdmissionToken values, as_deref preserves original references, source clone into
pending remains the same unboxed value. Initial proof allocation is before
Shared lock. Derived replay clones can allocate another bounded proof Box,
including existing Shared scope cloning; no allocation-free claim is made.
This affects active existing replay only and disappears after seed consumption.
Immutable Arc would avoid these additional clone allocations but introduces
shared reference-counted storage; zero-extra-allocation is not a current gate.
Retain bounded Box implementation and require real-input timing/focus/output
acceptance under unchanged deadlines; no new allocation/polling loop or owner.

## Bounded representation focused and early strict lint proof

GREEN10 ALL_PASS25/25,40.095s. Guarded check-lay-lints.sh then PASS for both
default and research-tools scopes: original inventory and warnings-denial stay
unchanged, source rows literal after lint. Local green-v10-import retains all
logs and raw proof. No runtime/graph import or source release yet; full release
and changed-byte client acceptance remain pending.


## Full v8 source gate and first changed-byte physical trial — 2026-10-06

Measured full v8 PASS_FULL_SOURCE_RELEASE_GATE,1258.344s; canonical correctness
and package selection2993/2993 PASS, strict default/research lint, source canon,
installer/public-issue regressions and Rust1.88 compatibility PASS. Exact receipt:
`/home/local-user/.cache/lay/development/contextual-completion-20261005/20261004T211126Z/release79/firefox-repair-20261005/release80-v8-import/RELEASE_GATE_RESULT.json`.
Immutable IME SHA2566a01e6b73374c9eb9ecd7ee5380f4b9f08a8da684d8f4f6a3043e065cd25537d.
This is source proof, not physical acceptance.

First temporary physical trial executed three owned Firefox field types.
Input and contenteditable PASS8/8 individually observed pairs each, including
actual next-letter input, actual Backspace, decoder and rendered GNOME icon.
Textarea FAIL at pair1: six initial letters appear literally without composition;
first key already passive/refused, before any replay in that field. Thus the
prior observer-ahead native-replay repair has a positive bounded physical result,
but this candidate is not admitted for stable installation or publication.
Matrix receipt:
`/home/local-user/.cache/lay/development/contextual-completion-20261005/20261004T211126Z/release79/windows/REQUIRED79_COMPOSITION_PROBE_V2_RUN_20261006T002610_eea0ca2f31d2/WIDE_MATRIX_V2_20261006T002647.json`.
Trial receipt:
`/home/local-user/.cache/lay/development/contextual-completion-20261005/20261004T211126Z/release79/firefox-repair-20261005/repair80-trial-c1c6f978f9a2499e866f1f53d7d7b6b0/TRIAL_RESULT.json`.
Finally rollback PASS_EXACT79_RESTORED; source/protected runtime identities
unchanged. Scope was temporary IME80 with79 CLI/extension/services. No global
IBus restart, input-source migration or stable80 installation occurred.

New first failed transition is lifecycle admission, not candidate ranking or
replay: transfer owner20 installed at trace L2734; FocusOut605 callback timeout
L2735 precedes observer ingress L2737; observer Disable606 L2738 and FocusIn608
L2739 arrive; Disable callback L2741 precedes FocusIn608 timeout L2744. First
key612 passive/refused L2749–2751; ManualToggle later rejects bridge_admission,
lifecycle_pending/revoked with zero unsettled keys. Trace source:
`/home/local-user/.cache/lay/development/contextual-completion-20261005/20261004T211126Z/release79/windows/REQUIRED79_COMPOSITION_PROBE_V2_RUN_20261006T002610_eea0ca2f31d2/traces/20261006T002634_firefox.textarea_each_shift_pair_real_letter_delete_composition/after_flush_ibus_engine_debug.jsonl`.
Exact interleaving of cleanup and observer publication is not logged. Hypothesis:
delayed source callback globally revokes the same retained owner and clears the
later authenticated FocusIn stamp. Require deterministic production-entrypoint
RED and negative transfer/owner guards before any additional runtime edit.

Replan: compare (1) increasing timeout/retrying activation, rejected because it
changes deadlines and hides the lost successor; (2) local-only cancellation on
all failures, not admitted because missing lifecycle observation cannot retain
old word/transfer authority; (3) preserve later authenticated lifecycle stamps
while retiring failed source word and seal through existing reducer/adapter,
preferred only after causal proof. Keep existing owners, exact sender/path/context
checks, original1ms callback budget,50ms activation budget and output routes.
New ready grants must be UnknownStart until fresh input proves a word. Broad
93-case/retained-Tab trials, actual Instagram/WhatsApp and changed-byte real
keyboard confirmation remain NOT_RUN/NOT_TESTED, not inferred from local fields.


## Retired Disable causal RED and narrow implementation

Controlled old-v8 production RED reproduced34.896s,7 exact tests: expected
positive FAIL with `retired Disable erased successor FocusIn stamp`,6 existing
focus/DoubleShift controls PASS. Original residuals preserved byte-for-byte;
only a new fixture appended. Receipt:
`/home/local-user/.cache/lay/development/contextual-completion-20261005/20261004T211126Z/release79/firefox-repair-20261005/disable-red-v1-import/RESULT.json`.
The fixture uses real expired FocusOut callback and actual observer ingress,
valid-wire FocusInId, then controlled Reset retiring the source before delayed
Disable. Reset is a mechanism discriminator, not an assertion about the physical
trace. Original production bytes of v8 remain bound by its full1475-row manifest.

Choose the narrower fourth alternative after independent source review:
`disable_retired_by_later_ingress` requires typed observed Disable owner, current
retained reducer owner, exact source ticket owner, Revoked status and exact
disable_position. Only this already retired event drops matching local/Shared
source through existing discard_context_activation, preserving later stamps.
Every other denial still invokes the original strict global invalidation.
First callback timeout remains a refusal. No seal/transfer/token/output granted,
no deadline increase, retry, queue, detector, transport or new input owner.
New activation still uses existing native reply+marker SourceFree UnknownStart.
Independent causal/safety audit:
`/home/local-user/.cache/lay/development/contextual-completion-20261005/20261004T211126Z/release79/firefox-repair-20261005/TEXTAREA_FIRST_LIFECYCLE_AUDIT.md`.
Current changed production bytes are NOT_TESTED; focused/full/physical proofs
remain required. Installed exact79 unchanged.


## Retired Disable focused GREEN

GREEN28/28 PASS46.971s, including the formerly failing real-callback fixture,
all25 previous replay/cancellation contracts and two existing focus controls.
Original assertions and fixed callback/activation budgets unchanged. Exact
focused source1476 rows imported, only guarded rustfmt changes admitted; proof:
`/home/local-user/.cache/lay/development/contextual-completion-20261005/20261004T211126Z/release79/firefox-repair-20261005/disable-green-v1-import/RESULT.json`.
Independent runtime-delta review found blockers0: deleting the21-line adapter
helper and10-line observation branch restores exact v8 production hashes.
Shared cleanup still requires matching path and source owner generation, so a
successor published during the check cannot be cleared as the old source.
New full gate and physical textarea proof pending; installed79 unchanged.


## Full v9 declaration failure before broad execution

Full v9 FAILED36.186s before changed/full suites: discovery correctly found one
new canonical `bin:lay-ibus-engine` correctness identity, while private input
mistakenly declared target `lay-ibus-engine`. This is a declaration mismatch,
not a source semantic failure. Keep failed receipt in release80-v9-import.
Correct only the private added-test target to the canonical `bin:` identity for
a fresh full v10; discovery/equality/ledger gate remains literal. GREEN28 source
proof remains valid for unchanged production; no live installation occurred.


## Full v10 and physical acceptance closure — 2026-10-06

Full source release gate PASS1256.802s: canonical2994/2994, zero semantic or
infrastructure failures, changed correctness/package, strict lints, public-issue
and installer regressions, MSRV, fresh graph/canon. Exact1476-row source and
10 generated outputs imported. New immutable IME SHA256
`0902a3b87efd8b9398a4c6a49323731ed95c86775e422f3d1e3d338700eaecce`.
Gate: `/home/local-user/.cache/lay/development/contextual-completion-20261005/20261004T211126Z/release79/firefox-repair-20261005/release80-v10-import/RELEASE_GATE_RESULT.json`.
This establishes source/build acceptance only.

Three finite diagnostic exchanges ran the same new IME with accepted79
CLI/daemon/extension/services; all ended PASS_EXACT79_RESTORED. No global IBus
restart, source migration, new input owner or stable80 installation occurred.
Same-byte daemon UInput attach/detach bookkeeping PID changes are retained in
trial receipts; L3/L1.1/global IBus/config/model/proof guards remain unchanged.
Summary with exact three trial receipts: `/home/local-user/.cache/lay/development/contextual-completion-20261005/20261004T211126Z/release79/firefox-repair-20261005/V10_PHYSICAL_SUMMARY.json`.

| Actual client proof | PASS / executed | Scope and remaining limitation |
|---|---:|---|
| 8 Double Shift pairs with letter/delete/icon after each | 2/3 fields | input/editable8/8; textarea pair1 refused |
| Ordinary first/second-word Tab | 6/6 | exact proposal plus space |
| Held Shift and next letter | 3/3 | physical typed effect |
| Backspace variants, original fresh-event observer | 15/18 | three after-word partial-delete rows fail before Tab dispatch |
| Separate retained-hint actual Tab after partial delete | 3/3 | independent control, never upgrades original rows |
| Nonletter controls | 50/60 | failed rows include initial mode setup; not evidence that each symbol produced a hint |
| Space autocorrection | 2/6 | layout/missing-letter outcomes still incomplete |

Wide original93 rows complete in168.972s, original180s bound unchanged.
Retained-hint control3 rows11.779s. All predicates and historical failures remain
literal. Local input/textarea/contenteditable are not proof of Instagram,
WhatsApp or every third-party editor. No real-keyboard confirmation obtained.
Normal release promotion is refused while physical contracts fail.

### New first failed transition and explicit bounded replan

Current textarea trace has sourceSeal602 all preconditions true, fresh
owner21/activation21 installed, all native printable keys admitted/settled. The
old late-Disable timeout did not occur in this run; its fix is source proved,
but this physical run does not certify exercising that branch.
The new first loss is confirmed_strict_prefix_receipt at2781, then full-length snapshot
at2783 with revision_gap2, identity_current=true and no replay scope, rejected
at2784. Manual bridge later succeeds, but context_authority is false at2819.
Exact full-surface equality is not logged; gap2 alone proves sufficient rejection.
The strict-prefix branch intentionally retains inert lineage but does not advance
its revision witness; the following full receipt therefore cannot confirm it.
Trace: `/home/local-user/.cache/lay/development/contextual-completion-20261005/20261004T211126Z/release79/windows/REQUIRED79_COMPOSITION_PROBE_V2_RUN_20261006T012112_32d2d404ae1e/traces/20261006T012129_firefox.textarea_each_shift_pair_real_letter_delete_composition/after_flush_ibus_engine_debug.jsonl`.

Previous native-replay and retired-Disable repair rounds are closed at their
recorded limits; start a distinct bounded causal repair, not retry-until-green.
Compare: blanket revision-gap relaxation (reject), advancing every inert prefix
(reject: hides generic gaps/replay), or a single inert exact prefix-echo witness
from a previously confirmed managed append chain, consumed only by its next
exact bounded full snapshot with unchanged owner/token/epoch and no replay.
This third alternative is HYPOTHESIS, not implemented or granted authority.
First prove old failure in real production callback tests, then positive/negative
contracts, independent review, source gate and new physical bytes. Original
SafetyGate/verifier, exact replay fences and all old assertions remain mandatory.

Five independently audited wide mode failures stop before symbols are sent.
Their tails have activation_ready after publication, without activation_install
or post_activation_update. Owner of that separate investigation remains the
existing activation mechanism; no synthetic wake key, fallback or deadline
change is authorized by a failing setup. Exact cause/repair still pending.

### Space79 discriminator: observation with failed preservation gate

The separate finite Space79 diagnostic consumed a ready full NoApply(rank)
lease4us, epoch437/gen13; final word remained uncorrected. Prepared total141.952ms
(queue74.174/eval67.771ms). `rank` only means InputGate did not ApplyReplacement:
Keep, SuggestOnly, Veto, missing candidates or candidate-authority failure remain
indistinguishable. Target retention/numerical rank UNKNOWN; literal candidates0
is not coverage proof. Overall wrapper FAIL_BINDINGS, because actual input
updated word_usage_events.jsonl and word_usage_counts.json. All other protected
bytes/helper/sourceIDs/IME/L3/L1.1/global IBus checks passed. This is not a PASS
receipt or a product latency result; statistics were not rolled back over user
events. Readout: `/home/local-user/.cache/lay/development/contextual-completion-20261005/20261004T211126Z/release79/firefox-repair-20261005/SPACE_READY_WAIT79_V2_RANK_READOUT.md`.


## Confirmed managed prefix echo causal RED and narrow implementation

RED on unchanged production v10: expected1FAIL29PASS48.161s. The positive
actual-key/CommitText/Reset/surrounding fixture fails with the exact marker
`confirmed native append delayed prefix lost next exact receipt authority`,
before bridge/RPC preparation. Seven negative scenarios and original28 replay,
lifecycle and physical-owner guards pass. Receipt: `/home/local-user/.cache/lay/development/contextual-completion-20261005/20261004T211126Z/release79/firefox-repair-20261005/prefix-red-v1-import/RESULT.json`.
The controlled fixture supplies full exact equality absent from metadata.

Choose one inert revision witness in existing PendingContextResetRereceipt.
Only a previously confirmed ManagedCommit chain, no local/shared ExactReplay,
no quarantine or completed replay, same current token/owner/epoch, can retain
its exact strict-prefix echo. It is spent at the next observation. Only checked
armed+2/echo+1 with unchanged identity and original full bounded snapshot may
use ordinary confirmation/consume. First-unconfirmed, duplicate prefix, wrong
surface, selection, focus/content/navigation gap remain denied. All constructor
rearms/native appends clear the witness; no timer, polling, input owner, routing
or generic revision-gap relaxation added. Original fixture assertions literal.
ADR: decisions/2026-10-06-managed-strict-prefix-echo-rereceipt.json.
Independent design reviews exclude inherited replay aliases explicitly; new
source awaits focused/full/physical proof, installed79 unchanged.


## Managed prefix echo GREEN v1: causal assertion passes, new dependent oracle fails

Focused30 executed48.110s:29PASS1FAIL; receipt `/home/local-user/.cache/lay/development/contextual-completion-20261005/20261004T211126Z/release79/firefox-repair-20261005/prefix-green-v1-import/RESULT.json`.
The new positive reaches and passes exact_manual_handoff confirmation, then
fails its added cycle09_tail_is_authoritative check before manual consumption.
All seven negative scenarios and original28 controls pass. Keep complete FAIL;
confirmation is not full manual-route acceptance. Independent runtime-delta
review found hard blockers0, unchanged exact receipt/consume functions and
literal original fixture prefix. Review the new dependent VisibleTailSource
oracle against the actual existing contract before another prospective proof;
no old assertion or historical result may be rewritten. Installed79 unchanged.


## New dependent oracle correction, unchanged production witness

Independent source audit proves the new fixture retained the literal boundary
space from known_engine plus four actual CommitText effects. VisibleTail returns
the whole committed_tail, so expecting only the four-letter word was an incorrect
NEW dependent oracle. Readout itself can admit confirmed Reset rereceipt and
does not consume it; no contrary ownership claim is made. Change only this new
post-causal oracle to exact expected space+typed word, assert buffer preservation,
actual ManualToggle3/false, then full authoritative VisibleTail+buffer equality
using the already accepted manual-first sequence. All older assertions unchanged.
The causal marker and every step before it are byte-identical, so its old RED
still proves detection; GREENv1 remains complete FAIL48.110s. Runtime production
witness source unchanged; next prospective GREENv2 required.


## Managed prefix echo GREEN v2 closes focused source round

ALL_PASS30/30, zero failed,48.139s. Positive runs ASCII and Unicode native-key
callbacks with exact CommitText/full-snapshot proofs, preserved boundary+word,
actual ManualToggle3/false and strict whole-buffer VisibleTail. Consumption
keeps UnknownStart and bounds its observed suffix; no KnownStart grant claimed.
Seven negative scenarios and all original28 controls pass. Independent V1
production delta review hard blockers0; unchanged production reused for v2.
Exact1477-row focused source imported, only guarded formatted2 Rust outputs.
Receipt: `/home/local-user/.cache/lay/development/contextual-completion-20261005/20261004T211126Z/release79/firefox-repair-20261005/prefix-green-v2-import/RESULT.json`; import: `/home/local-user/.cache/lay/development/contextual-completion-20261005/20261004T211126Z/release79/firefox-repair-20261005/prefix-green-v2-import/IMPORT_RESULT.json`.
New bytes still require full source release gate and changed-byte physical
acceptance; exact79 remains installed. V1 FAIL and old physical outcomes retained.

Next connected repair investigates no-key activation publication/consumption.
Existing adapter acquisition task owns native Get/marker; existing engine install
and native InputMode publisher remain sole application owner. Compare existing
later-callback-only consumption (demonstrably insufficient), same-task exact
completion delivery (proposed), and polling/synthetic wake/timer/fallback
(rejected). Require causal old-source callback proof before selecting delivery.
No source/engine-ID change or second layout decision is proposed.


## Acquisition completion causal RED v1: rejected control set

Measured remote source-only RED v1 executes35 tests:33PASS/2FAIL,50.576s.
Native actual-wire positive fails at the predeclared
C09_ACQUISITION_COMPLETION_WITHOUT_KEY_MISSING_INPUT_MODE marker, proving
completed acquisition alone does not deliver its installed native mode.
An unchanged older control, firefox_manual_toggle_defers_until_the_client_replies_after_rpc,
also fails at tests.rs:558: serve_ping_and_marker receives a message without a
member header. This second failure rejects the frozen causal_failure_exact
protocol. Verdict FAIL_ADMISSION_OR_EXECUTION, not accepted causal RED and not
permission to promote the delivery patch. First wire message type/phase of the
older failure remains UNKNOWN pending bounded audit; no old assertion, deadline
or membership filter has changed. Production delivery patch remains unapplied.
Receipt: `/home/local-user/.cache/lay/development/contextual-completion-20261005/20261004T211126Z/release79/firefox-repair-20261005/activation-red-v1-failed-import/RESULT.json`.
Installed exact79 unchanged; runtime authority false.

Independent V17 editable boundary audit identifies the actual missing delimiter
before '(' and before subsequent Reset. Clear-empty/Hide and CommitText(' ')
await precede the local tail becoming5, but client receipt remains4/4; final
fresh DOM is5/5. The signal send does not prove insertion acknowledgment.
Exact wire delivery/client application remains UNKNOWN. This is a separate
Space boundary failure, not a proved nonletter hint failure.
Audit: `/home/local-user/.cache/lay/development/contextual-completion-20261005/20261004T211126Z/release79/firefox-repair-20261005/V17_012249_FIREFOX_EDITABLE_NO_HINT_BOUNDARY_AUDIT.md`.
No field-specific exception, probe, timer or authority change applied.


## TEST29 header diagnostic and delivery review round one

Source-only exact old control, header metadata only before the unchanged Ping
assertion: ALL_PASS1/1,32.134s. All five Ping stages receive MethodCall/Ping.
The earlier headerless offender was not reproduced and its type remains UNKNOWN.
This prospective PASS does not rewrite the earlier35-test FAIL. No assertion,
reply filter, timeout or production source changed; no runtime import occurred.
Receipt: `/home/local-user/.cache/lay/development/contextual-completion-20261005/20261004T211126Z/release79/firefox-repair-20261005/test29-header-v1-import/RESULT.json`.
Next controlled witness will exercise an actually unregistered engine and its
exact detached Reset reply serial before any bridge; not a blind retry.

Independent delivery review round one REQUEST_CHANGES_ONE_STATIC_RACE.
Separate ready and pending reads can miss matching publication and return Denied
after publication clears pending. Completion must inspect matching request,
fence and path consistently using existing pending→ready→reducer lock order
and existing changed listener. Generic Bridge wait remains unchanged. The
initial pending.ready suspicion was withdrawn: Acquisition marker processing
returns before the Bridge-only ready flag. No implementation based on that
withdrawn suspicion was made.
Review: `/home/local-user/.cache/lay/development/contextual-completion-20261005/20261004T211126Z/release79/firefox-repair-20261005/ACTIVATION_COMPLETION_DELIVERY_INDEPENDENT_REVIEW.md`.
Observed Acquisition/ready grants deliberately survive fence expiry in the old
callback contract. Activation deadline is not a new token wall-clock expiry;
autonomous delivery latency beyond blocked engine interface remains unmeasured.
New delayed-predecessor listener fixture is NOT_RUN and does not claim a
deterministically reproduced mutex-race seam. Exact79 remains installed.


## Controlled detached Reset witness closes harness diagnosis

ALL_PASS1/1,32.194s. For actual registered→removed engine objects, direct
observed Reset serial19315 and19415 each generates exactly TypeError,
UnknownObject, the matching reply serial and no member, before any bridge.
No filter/drain occurs between Reset and the raw witness. This establishes
a concrete scheduling-dependent harness gap; the historical header itself
remains UNKNOWN. Correct only the two absent-engine helper flags through
the existing exact-serial consumer; all prior assertions/deadlines stay literal.
Decision: 2026-10-06-detached-reset-controlled-peer-reply.json.
Receipt: `/home/local-user/.cache/lay/development/contextual-completion-20261005/20261004T211126Z/release79/firefox-repair-20261005/test29-detached-v1-import/RESULT.json`.
No blanket filter, sleep, authority or runtime change. A fresh strict causal
RED over the corrected source-control schedule is required; prior REDv1 FAIL
and header-diagnostic PASS retain their original separate scopes.


## Acquisition completion strict RED v2 accepted; connected source change

OLD_FAILURE_REPRODUCED36 executed/35PASS/1expectedFAIL,50.600s. Actual
no-key native completion reaches the exact missing InputMode marker; all30
prefix/manual controls, four native publisher controls and exact detached
Reset witness pass. RED production source unchanged. Only two absent-engine
helper flags use exact-serial consumption; all original assertions/deadlines
and remaining fixture prefix bytes are preserved. Exact guarded formatted
residuals imported, then appended GREEN-only delayed-predecessor listener.
A local length-based import assertion initially counted the two false→true
character reductions incorrectly and stopped before fixture mutation; corrected
exact transformed-prefix comparison passed. No source proof result changed.
Receipt: `/home/local-user/.cache/lay/development/contextual-completion-20261005/20261004T211126Z/release79/firefox-repair-20261005/activation-red-v2-import/RESULT.json`.
V1 rejected control set and historical UNKNOWN header remain recorded.

Applied independently reviewed V2 same-task acquisition delivery. Consistent
pending→ready→reducer checks remove the reviewed split-read race; exact full
ready token is checked before successor request, existing listeners/deadline
and cancellation retained, generic Bridge wait unchanged. The served engine
revalidates full current outcome after awaits, respects atomic exclusivity,
uses original install/ACK and guarded native InputMode publisher. No new
owner, queue, detector, RPC, source migration, model or text-edit capability.
Review: `/home/local-user/.cache/lay/development/contextual-completion-20261005/20261004T211126Z/release79/firefox-repair-20261005/V2_REVIEW.md`.

Applied separate opt-in existing-gate Clear/Hide/Commit metadata diagnostic
plus legacy callback serial binding and preserved output/error-order unit.
No raw text, new callback state, locks, timers or output semantics. Join only
unique PID/path/existing callback-clock; orphan/ambiguous is UNKNOWN. Signal
success does not establish client insertion; logging can perturb timing.
Review blockers0 for patch93af4ae1; this remains diagnostic-only source scope.
The combined source now requires GREEN, full gates and physical acceptance.
Runtime authority unchanged: accepted exact79 remains installed.


## Combined GREEN v1 compile failure, bounded correction

FAIL_ADMISSION_OR_EXECUTION29.143s: discovery0 returns101 before execution.
Compilation E0252 found EnginePath imported twice in observation.rs: new
header import and existing owning-section import. Remove only the duplicate
new import; no predicate, assertion, timeout, identity or test selection changes.
No test executed, so no semantic PASS/FAIL denominator claimed. Static design
reviews did not claim compilation. Original failed receipt retained:
`/home/local-user/.cache/lay/development/contextual-completion-20261005/20261004T211126Z/release79/firefox-repair-20261005/activation-green-v1-failed-import/RESULT.json`.
Installed exact79 unchanged.

Opt-in prepared InputGate diagnostic independently reviewed blockers0: borrow
the already computed result once, before original conversion; no new ranker,
pool generation or admission query. Actual action/reason/returned pool size,
selected receipt and existing frame binding remain metadata only. Scoreboard
apply is selected-index count, suggest includes unselected Eligible; no claim
of complete authority-admitted pool. final_decision_present is prepared
conversion, not physical Apply. Existing source/generation/prepared-vs-superseded
trace binds observations; no late edit permission. Debug/log overhead is outside
original route.total_us and still needs runtime latency measurement.
Review: `/home/local-user/.cache/lay/development/contextual-completion-20261005/20261004T211126Z/release79/firefox-repair-20261005/space-gate-metadata/SPACE_GATE_METADATA_INDEPENDENT_REVIEW.md`.


## Combined GREEN v2: positives pass; explicit harness replan

FAIL_ADMISSION_OR_EXECUTION40 executed/28PASS/12FAIL,51.054s. Compilation
succeeded. Native and compatibility actual served no-key acquisition/property
positives both PASS; output-order/error-propagation invariant PASS. This proves
the causal repair only in those source fixtures, not full source or client
acceptance. Twelve older/new listener controls encounter TypeError where Signal
or Ping is expected. Terminal excerpts expose reply serials21020,22600,41100.
The exact error names of these unseen headers remain UNKNOWN.
Receipt: `/home/local-user/.cache/lay/development/contextual-completion-20261005/20261004T211126Z/release79/firefox-repair-20261005/activation-green-v2-failed-import/RESULT.json`.

Explicit replan before further changes: new delivery must obtain the actual
served interface. Existing harness often sends synthetic callbacks to an
unregistered engine and then directly invokes a local callback. Investigate
whether interface lookup starts zbus dispatch earlier, making their unresolved
reply transport observable. Controlled detached-Reset witness already proves
exact UnknownObject replies for19315/19415, but does not label all twelve.
Compare an existing-server-only public lookup if the dependency provides one,
versus explicit controlled transport semantics for manual callback fixtures.
Reject blanket error filtering, changing assertions/deadlines, per-case runtime
bypasses, skipping controls or retry-until-green. Keep all normal real-wire
callback paths unchanged. No further runtime producer or ownership change
until this common first transition is established. Independent agents audit
only dependency API and shared fixture seams; ROOT owns any eventual edit.
New guard fixture prepared separately NOT_RUN; no PASS claim for its negatives.
Production remains uninstalled; accepted exact79 unchanged.


## Controlled FIFO header diagnostic confirms shared transport class

Source-only diagnostic executes three existing controls with unchanged
assertions/filters/deadlines:0PASS/3FAIL,32.740s. Unexpected headers are exactly
Error/UnknownObject/memberNone with replyserial30100 in forward_marker,
21020 and41100 in legacy_effects. This confirms those three first failures
are synthetic detached callback transport, not rejected engine output. Remaining
nine failure headers are not individually labelled by this sample.
Receipt: `/home/local-user/.cache/lay/development/contextual-completion-20261005/20261004T211126Z/release79/firefox-repair-20261005/fifo-header-v1-failed-import/RESULT.json`.

Dependency source zbus5.15.0 public object_server() starts lazy dispatch.
No public existing-server-only lookup is available; private ensure/root APIs
cannot be a runtime workaround. method_message already sets NoReplyExpected;
zbus rejects missing objects before per-method reply suppression. Therefore
adding that existing flag again cannot repair this transport class. Registered
normal ProcessKeyEvent replies (including48bool checks), bootstrap/Get/Ping
and real-wire defaults must retain strict behavior.

Selected bounded direction: common test-only manual/detached callback sender
records exact expected optional reply serials with the existing UnknownObject
filter, preserving order and all effect/state/denial assertions. Duplicated
callback serials must not collapse accounting or introduce sleeps between
batched adverse events. No blanket ignore, owner registration cache, client
exception, production timing or normal-reply route change is allowed.
Independent common-helper proposal/review still pending; no new test seam
applied yet. Diagnostics2files remain private, unimported. Exact79 installed.


## Exact manual callback transport: reviewed implementation, execution pending

Applied reviewed patch735a2643d6593da7decacf39e343f80b78f06b0053b0480db5c98ef752e613f2 to four controlled-test files under ADR2026-10-06-manual-callback-exact-reply-accounting. Final independent review PASS_WITH_LIMITS, zero blockers; execution NOT_RUN. One counted declaration per explicit manual send preserves duplicated serials; existing strict UnknownObject filter, normal48 boolean replies, served no-key fixtures and original assertions/deadlines remain literal. Added exact repeated-serial witness and separately reviewed post-interface-wait delivery refusal fixture (atomic active, foreign adapter, consumed grant), both NOT_RUN. No production change in this step; current activation completion implementation and telemetry remain uninstalled. Accepted exact79 is unchanged. Next scope: original40 tests plus these two fixtures and actual registered48 replies, on guarded remote source only.


## Combined GREEN v3: 42/43, residual first transition differs

FAIL_ADMISSION_OR_EXECUTION43 executed/42PASS/1FAIL,53.947s. Native/compatibility no-key positives, new repeated-serial accounting, post-interface-wait atomic/foreign/consumed-grant refusals and registered48 boolean replies PASS. All prior twelve transport-failing cases except one now pass; remaining read-only exclusion control fails at observer Denied, before original bridge-revocation assertion. Exact member not yet known. Receipt: /home/local-user/.cache/lay/development/contextual-completion-20261005/20261004T211126Z/release79/firefox-repair-20261005/activation-green-v3-failed-import/RESULT.json. No failed-source import, installation or promotion. Explicit replan: instrument only the existing fixture before its unchanged observer assertion to identify member/key/release and first denial. Do not weaken assertions, change deadlines or repeat until green. Private diagnostic remains source-only; accepted exact79 unchanged.


## Remaining control diagnostic and bounded wire-type correction

Diagnostic0/1FAIL31.740s: first packet ProcessKeyEvent/keyval97/keycode30/state0 reaches observer Denied. Receipt: /home/local-user/.cache/lay/development/contextual-completion-20261005/20261004T211126Z/release79/firefox-repair-20261005/activation-denied-diagnostic-failed-import/RESULT.json. Static source analysis identifies manual inline builder lost the original typed_key_message u32 argument constraint: keycode literals default to i32, while strict ingress requires (u32,u32,u32). This type-inference explanation is distinct from the measured denial; historical wire signature was not logged. Restore exact original unsigned tuple constraint in this test branch only. All prior assertion/deadline/callback values remain literal; production unchanged. Following explicit replan, run all43 again to verify the corrected seam, not retry unchanged bytes. Exact79 remains installed.


## Combined GREEN v4: causal repair and controls all pass

ALL_PASS43/43,zero failed/ignored,53.950s. Original40 identities, exact repeated-serial witness, three post-interface-wait delivery refusal subcases and actual registered48 boolean replies pass. Both native and compatibility served no-key InputMode positives pass; prefix rereceipt and original rejection controls pass. Corrected unsigned tuple restores old wire contract without assertion/deadline changes. Exact formatted declared Rust source imported only after full input/parent/after closure; all source rows match. Receipt: /home/local-user/.cache/lay/development/contextual-completion-20261005/20261004T211126Z/release79/firefox-repair-20261005/activation-green-v4-import/RESULT.json. Scope: focused source only; new full release/physical gates NOT_RUN. Current production implementation remains uninstalled; accepted79 bytes unchanged. Next: one guarded full source gate with all new identities and fresh graph/canon, then finite native Firefox input trials with exact79 rollback.


## Full source v11: canon documentation rejection, no promotion

FAILED83.682s: compile/discovery and fresh architecture graph PASS; CANON refuses two new decision records missing reason and uncovered composition_commit metadata. No full suite or immutable release artifact was produced. Receipt: /home/local-user/.cache/lay/development/contextual-completion-20261005/20261004T211126Z/release79/firefox-repair-20261005/release80-v11-failed-import/RELEASE_GATE_RESULT.json. Explicit bounded documentation correction: complete original detached-Reset and manual-transport record schema and add separate Space-boundary metadata observation decision. Runtime Rust, old guards/assertions/deadlines and focused43/43 proof stay literal. New full gate must run from that proven exact parent plus only these docs; no failed generated output imported. Installed exact79 unchanged.


## Full source v12: additional manual transport coverage failures

FAILED472.934s at CHECK_CHANGED388.942s. Canon/schema and fresh graph PASS; all original canonical rows preserved plus exactly9 new identities. Engine target678 selected reports11 failures; focused43/43 is not full acceptance. First excerpts: expected Signal receives Error in generic forward/drain helpers and native_transfer boundary; some residuals instead unwrap absent header member. Those error names are UNKNOWN until exact header capture. Raw11 process logs retained in /home/local-user/.cache/lay/development/contextual-completion-20261005/20261004T211126Z/release79/firefox-repair-20261005/v12-engine-partial-failures; full receipt /home/local-user/.cache/lay/development/contextual-completion-20261005/20261004T211126Z/release79/firefox-repair-20261005/release80-v12-failed-import/RELEASE_GATE_RESULT.json; canonical summary /home/local-user/.cache/lay/development/contextual-completion-20261005/20261004T211126Z/release79/firefox-repair-20261005/release80-v12-failed-import/CHECK_CHANGED_SUMMARY.json. No final generated import or immutable artifact; no live changes.

Explicit replan: earlier22 manual callsite mapping was incomplete. Classify every actual send/build/handler ownership in five controlled fixture files, preserving all real served RPC/48bool routes. Confirm one representative remaining header class, then extend the existing explicit per-emission declaration seam systematically. Do not automatically declare all NoReply packets, blanket-filter errors, change old assertions/deadlines, or add production ownership/cache/bypass. Production patch freezes while test transport closure is established. This replaces selective failure-by-failure repair; ROOT owns the next sealed source-only run and requires full fixed proof. Exact79 remains installed.


## Full v12 aggregate and exact remaining transport header proof

CHECK_CHANGED target log totals:3003 selected,11 failures,2992 passes across32 target rows. Hermetic summary BLOCKED_CONTRACT because the original empty known-failure ledger refuses those11 unexpected failures; infrastructure_failures0 and no ledger edits. Other later release gates remain NOT_RUN.

Private source-only header diagnostic repeats exactly11 original failing identities:0/11PASS,11FAIL36.710s. Original assertions and exact receive filter unchanged. Every first unexpected header is measured TypeError/UnknownObject/memberNone with reply serial respectively3219,3224,7504,7604,3400,3600,3237,3200,3180,3215,3215. This proves the common unaccounted manual transport transition for these exact fixtures; subsequent unseen behavior remains UNKNOWN. Receipt /home/local-user/.cache/lay/development/contextual-completion-20261005/20261004T211126Z/release79/firefox-repair-20261005/manual-transport-closure-diagnostic-failed-import/RESULT.json; per-identity headers /home/local-user/.cache/lay/development/contextual-completion-20261005/20261004T211126Z/release79/firefox-repair-20261005/manual-transport-closure-diagnostic-failed-import/HEADER_PACKET.json. Diagnostic tests.rs remains private and unimported. Complete manual-versus-served callsite inventory is being built before any extension of the existing declaration helper. Accepted exact79 remains installed; no new production change or runtime authority.


## Complete explicit manual transport closure imported for execution

Final patchc4101c04457a02b3c506454711dcedf5d339eb7eeb697226042860d63bf6b67b applied with exact five-file before/after bindings, exactly3 changed test files. Complete158-site inventory76676f5563d5c87d92b83798aa43887e23d58ec89bea5b7e154a02a6f6dcad04 maps105/105 previously undeclared manual sites, preserving normal5 registered RPC sites, raw2 adverse ingress, remaining Signal caller of original mixed helper, old typed key builder/u32 ABI, all old assertions/deadlines/observer order, exact filter/counter and already tracked legacy_key. Independent actual-byte review PASS_WITH_LIMITS, zero blockers. Execution NOT_RUN at import. ADR complete-manual-test-transport-closure governs this test-only step. New runtime production unchanged and uninstalled. Next fixed scope54 identities = prior43 + every11 full-proof failure; full3003 gate remains required. Exact79 remains live.


## Complete manual closure GREEN: all fixed54 pass

ALL_PASS54/54,zero failures/ignored,59.313s. Includes the prior43 authority/mode/Space/error-order/normal48 controls and every11 previously failing full-proof identity. Exact source closure and formatted three-test-file import completed, all other source rows literal. Receipt: /home/local-user/.cache/lay/development/contextual-completion-20261005/20261004T211126Z/release79/firefox-repair-20261005/manual-closure-green-v1-import/RESULT.json. This establishes the common harness transport repair in its fixed scoped set; full3003 and later release gates must still pass. Current production changes remain uninstalled, physical Firefox for these bytes NOT_RUN. Exact79 remains live.

## Full source v13: semantic gates pass, new test lint refuses release

FAILED920.128s at FULL_GATE437.897s. CHECK_CHANGED and the repeated full
canonical correctness/package set both PASS3003/3003, zero known semantic or
infrastructure failures, including678 engine tests. Fresh graph/canon PASS.
Default dead-code inventory532 matches the unchanged baseline. Strict Clippy
then refuses two clone_on_copy calls on Option<WordScope> in the new
post-interface-wait refusal fixture; no production lint failure is reported.
The original lint wrapper discarded its temporary JSON; a separately guarded,
read-only scoped Clippy diagnostic establishes both exact locations12144/12186.
Receipt: `/home/local-user/.cache/lay/development/contextual-completion-20261005/20261004T211126Z/release79/firefox-repair-20261005/release80-v13-failed-import/RELEASE_GATE_RESULT.json`.
Exact diagnostic: same directory `CLIPPY_DIAGNOSTIC.err`.

Bounded correction removes only those two redundant clones in the new test.
Copied values, assertions, deadlines and runtime Rust stay unchanged; no lint
allow, baseline update, known-failure entry or skipped gate. Recheck that exact
fixture and strict Clippy before another full release gate. Later MSRV/build/
installer/physical gates are NOT_RUN; no failed generated import or release
artifact was admitted. Exact79 installed bytes remain unchanged.

## Test-copy correction: focused and strict lint PASS

The changed post-interface-wait refusal fixture passes1/1,31.778s; exact
formatted source closure1482 rows matches ROOT without a formatter import.
Receipt: `/home/local-user/.cache/lay/development/contextual-completion-20261005/20261004T211126Z/release79/firefox-repair-20261005/lint-clone-green-v1-import/RESULT.json`.
Separate guarded strict lint then PASS for default and research-tools profiles:
non_dead_diagnostics0 in each; unchanged dead-code inventories532/356.
Receipt in the same directory: `STRICT_LINT.out` and `STRICT_LINT.err`.
Only two Copy-value test expressions changed. This combines the prior54/54
focused proof and v13 two3003/3003 semantic passes with the newly measured
lint correction; it does not replace a full gate for the exact final source.
Full source v14 is the next required gate; physical acceptance and ordinary80
installation remain NOT_RUN. Accepted exact79 and runtime authority unchanged.

## Full source v14: second semantic pass exposes asynchronous state loss

FAILED884.987s at FULL_GATE401.133s. CHECK_CHANGED PASS3003/3003;
the repeated canonical full set reports3002/3003, one engine failure in
residual_reset_rereceipt_failures_never_delete_gui_text. The unchanged
DeleteSurroundingText/CommitText denial assertion passes; the subsequent
unchanged local-tail assertion fails at3314: after="", before="abcde".
The seven-case gap identity and first state mutation are not yet measured.
This is different evidence from earlier UnknownObject transport failures.
Receipt: `/home/local-user/.cache/lay/development/contextual-completion-20261005/20261004T211126Z/release79/firefox-repair-20261005/release80-v14-failed-import/RELEASE_GATE_RESULT.json`.
Raw exact case: same directory `REFUSED_MANUAL_TAIL_FAILURE.log`;
hermetic summary `FULL_GATE_SUMMARY.json` retains BLOCKED_CONTRACT and the
original empty known-failure ledger. No failed source/generated/artifact import.

Explicit replan before another full gate: inspect the new no-key acquisition
completion alongside the old denied-manual and delayed-interface registration
sequence. Separate a real stale/evolved-context installation from a controlled
fixture registration transition; neither is established yet. Capture actual
gap, token/owner/scope/tail at the first transition, then use controlled event
ordering rather than sleeps or retry-until-green. Do not weaken denial or
tail assertions, existing deadlines, activation guards or verifier authority.
Two independent source reviews are pending; production is frozen. Later lint/
MSRV/release/physical gates NOT_RUN on v14. Exact79 remains installed.

## V14 tail-loss diagnostic: nonfailure capture, cause still UNKNOWN

One private logging-only source run of the unchanged seven-gap test passes
1/1,32.001s. Metadata before the bridge and after its old helper captures
unchanged abcde for mismatch/selection/second_receipt/focus_out/caps9;
navigation/sensitive already have empty tails before the bridge. Original
assertions, callback order and budgets stay literal. Receipt:
`/home/local-user/.cache/lay/development/contextual-completion-20261005/20261004T211126Z/release79/firefox-repair-20261005/refused-tail-gap-diagnostic-v1-import/RESULT.json`;
raw metadata in the same directory `TEST_0.err`. Diagnostic source is private,
never imported. Logging may perturb scheduling; this passing sample does not
identify the original failed gap or establish a repair.

Two independent static reviews identify different concrete possibilities:
activation installation/discard can clear the local tail, but the original
SourceFree grant normally ACK-consumes at the first key; alternatively, queued
manually handled messages may reach automatic zbus dispatch after late engine
registration. Direct bridge authority refusal does not itself clear the tail.
Reports: `V14_RERECEIPT_TAIL_FLAKE_STATIC_AUDIT.md` and
`V14_ACQUISITION_TAIL_STATIC_REVIEW.md` in the same Firefox private directory.

### Finite collection replan and withdrawn barrier

The proposed retrospective absent-dispatch barrier is **DO_NOT_APPLY / NOT_RUN**.
Its remaining optional reply counts can include packets received before zbus
subscribed its automatic dispatcher; zbus does not replay those packets. Waiting
for every count therefore does not prove dispatcher closure and can time out.
The private correction is
`V14_ABSENT_DISPATCH_BARRIER_PROTOCOL_CORRECTION.md`, beside the reports above.
Any future controlled transport closure must first obtain an actual dispatcher
readiness acknowledgement, then establish completion of each exact emission.

The next source-only diagnostic is a fixed 32 calls of the literal original
seven-gap test, without stopping after success. A bounded cfg(test) memory
recorder observes installation, discard and focus cleanup using metadata only;
it prints event payload only after an observed tail mismatch. Caught original
assertion failures, gap exposures and lost diagnostic events remain separate
from successful completion of collection. Zero observed failures cannot close
the original V14 failure. This private overlay is not imported into ROOT or
the installed runtime. Runtime authority remains unchanged.

The fixed collection completed all 32 original calls and 224 gap entries with
zero assertion failures, zero observed tail changes and zero lost diagnostic
events. Test execution took 3.767 seconds; guarded preparation/compilation plus
collection took 35.453 seconds. The receipt is
`refused-tail-fixed32-diagnostic-v1-import/RESULT.json` in the same private
Firefox directory. The recorder perturbs scheduling; this nonfailure result
does not identify the V14 mutator and does not certify the runtime. The original
V14 failure remains open. No diagnostic source was imported and installed79
hashes still match the accepted bytes.
Next proof uses an actual held engine interface and queued callback, with a
trailing reply-bearing standard Peer.Ping as a completion fence. The real
engine interface has `spawn = false`; its dispatcher therefore finishes the
earlier engine call before dispatching the trailing Ping. Readiness of the
dispatcher is established by a separate actual Ping before the fixture emits
callbacks. This proves or rejects duplicate dispatch under a controlled
violation; it must not be described as reproducing the original V14 mutator.

The independently inspected two-seam transport isolation design is readiness
in the existing harness bootstrap and strict absent-path completion before
the refusal helper publishes its engine. It retains observer-first ingress,
original assertions, callback budgets and normal registered RPCs. Unexpected
messages remain failures. No per-key wait, production change or acceptance
waiver is planned. Runtime production is frozen; accepted exact79 and input
authority remain unchanged.

The first controlled-dispatch witness attempt stopped at compilation: Rust
E0716 in the private four-field preedit decoder borrowed `Value<'_>` from a
temporary message body. No test executed. The two proposed transport helper
files are unaffected. Receipt:
`dispatch-isolation-witness-v1-failed-import/RESULT.json`; elapsed 28.754 seconds.
The second finite repair binds that existing message body before decoding;
wire types, old assertions and all runtime source remain unchanged. Any further
unresolved issue requires another explicit replan rather than an unreported
third repair.

The repaired controlled witness and original seven-gap test passed 2/2 in
32.474 seconds including compilation. Receipt:
`dispatch-isolation-witness-v2-import/RESULT.json` (SHA256
`359907463dc2293b2977ce74a8510535f3c25473ebe1a3c6b4144673577f7529`).
One real NoReply Reset packet executed both manually and through the served
engine; the inert existing alt-completion flag was cleared by the second body.
Measured tail remained `abcde`, epoch remained 6 and token/reducer revocation
remained 4. No text-edit signals occurred. This establishes duplicate fixture
delivery, not a reproduction of V14 tail loss; the old mutator remains UNKNOWN.

Only the independently reviewed two-file helper isolation was imported into
ROOT. No private witness, recorder or diagnostic test was imported; all 3003
canonical identities remain. The next fixed proof covers the original failure,
every direct caller of the changed refusal helper and the earlier transport
closure controls. Production/runtime authority remains unchanged.

The helpers-only fixed affected proof passed 58/58 in 61.892 seconds; receipt
`dispatch-isolation-helper-green-v1-import/RESULT.json` SHA256
`572135413f62111cb620c0ccccdafe9ae9ffbc5738e7e17608dcc80f1fc70c92`.
Exact formatter-produced helper bytes were imported. Its source snapshot
contains 1483 rows and zero private diagnostic tests. Full V15 starts from that
proven source and overlays only the current owning document/decision; all
3003 canonical identities and original zero-failure ledger remain mandatory.
Original V14 causal attribution remains UNKNOWN; changed production bytes still
require physical-client proof. Installed79 remains unchanged.


### Full V15 and finite same-byte Firefox trials — 2026-10-06

Full V15 passed all 15 release stages in 1260.297 seconds, including both
3003/3003 canonical suites, strict Clippy, public issue/installer regressions,
Rust 1.88 checks, and the generated-graph/canon checks. Exact imported source
contains 1483 rows and ten generated graph files. IME candidate SHA256 is
`0d0c0617d61fc22f94d1c82014a38a513fa36b4d4c01bbe20fca92e3a7271bc5`.
Full receipt: private Firefox directory `release80-v15-import/RELEASE_GATE_RESULT.json`
SHA256 `df684e39b37144df98fd5c642ea9d5456b860226e5a7aced67a34717c6b2dbd6`;
source manifest `SOURCE_AFTER.json` SHA256
`d8a3085057a73c87f534a9d142daba975d087a8a62b0e692222f18b45ef585a2`.
This is source acceptance, not an all-Firefox runtime claim. Original V14
causal attribution remains UNKNOWN.

All source rows remained frozen during three predeclared finite temporary
IME80 trials. Every trial restored exact accepted79 bytes and closed its
source/protected-runtime checks successfully. Global IBus, L3 and L1.1 were
preserved; ordinary full80 installation did not occur. The synthetic device
is not actual physical-keyboard acceptance.

| Same-byte scope | Measured result | Remaining limitation |
|---|---|---|
| Eight sequential Double Shift pairs, next letter, Backspace and rendered icon in input/textarea/editable | 3/3 fields PASS, 38.423 s | owned local fields, daemon79 and synthetic device |
| Fixed wide matrix | 86 PASS / 92 executed, 6 FAIL, 1 BLOCKED, 140.460 s | all three autocorrect-rnjhjt and partial-delete-after-word rows failed; editable second-word Tab was not sent |
| Actual Tab with retained visible hint after partial deletion of second word | 3/3 PASS, 11.954 s | cannot reinterpret wide preflight failure as PASS |
| Digits/special-symbol hint suppression within wide matrix | 60/60 PASS | explicit fixed examples only |

Summary with exact trial/matrix hashes and paths:
`V15_PHYSICAL_SUMMARY.json` in the private Firefox directory. Trial receipts:
`repair80-trial-99131b0ba28f454d9b3cd71a066d6ae1/TRIAL_RESULT.json`,
`repair80-trial-886328e7fa3a49d38ff9b4d64f6aba9f/TRIAL_RESULT.json`, and
`repair80-trial-d400a5c5070744a2b763bfea25b7ba76/TRIAL_RESULT.json`.
Wide physical matrix SHA256
`beebd07a8d8b5e7dee5a634a5b67229d2031d82cceaf330de3ce5f30704a70dc`.
Three wide partial-delete failures stop before Tab: browser composition-event
freshness admission fails after actual native Backspace update. Exact retained
Tab success establishes a different measured boundary, not a waiver of the
wide predicate. The three autocorrect cases leave `кторое `; the first loss
in candidate/decision/verifier authority is under receipt analysis.

A fourth predeclared one-attempt-per-field Space-DOM diagnostic sampled the
own field immediately after Space and before the next symbol. It passed
textarea and editable, failed input: no sample in the bounded interval showed
`тест `; later samples showed `тест(`. This classifies SPACE_NEVER_OBSERVED_IN_BOUNDED_SAMPLES,
not proof that a space never transiently existed or identification of a Gecko
branch. Elapsed 13.361 s; 2 PASS / 3 executed, zero BLOCKED. Exact79 rollback
and all1483-row closure passed. Receipt:
`repair80-trial-space-dom-ca01836cc0784d42aaace321e9616c9a/TRIAL_RESULT.json`;
inside its `SPACE_DOM_RUN_3c7414a6cc1347af9646beb8c859234e`, matrix SHA256
`7748a50b60ce9284126c43dd1709a05be25916fd9fe6b341ed8c282d4adefc58`
and failed input panel SHA256
`277043928d01acfdc508efff63af8e5fa362485de5e5ee19e5854be65982981b`.
The next diagnosis joins exact Space callback identities to output-await
metadata and own DOM receipts. Successful emission cannot establish client
insertion. No additional text executor, field-name bypass, lowered safety gate
or changed production path is authorized by these failures.

Instagram/WhatsApp, actual physical keyboard, combined daemon80/IME80 and
ordinary80 installation remain NOT_TESTED. Installed runtime remains exact79.

### Native Space transport experiment: source admission — 2026-10-06

The exact failed input callback emitted Clear, Hide and Commit(separator)
successfully, yet the bounded own-field DOM samples did not contain the space.
The causal alternative is confined to a private four-file prototype: ordinary
managed Legacy NoApply/manual-suppressed Space clears/hides the suffix and
passes the original key through; Atomic, legacy active-word composition and
authorized correction paths retain their existing behavior. The existing
native-input observer closes the boundary. No new owner, editor, key detector,
candidate policy or application-name condition is introduced.

Focused source controls passed 12/12 in 38.582 s, including four protected
Double Shift ownership tests. The first run passed 11/12; the new Atomic test
failed during suppression setup, before its transport assertions. A test-only
repair publishes its token through existing push_tail_char; production bytes
did not change during that repair. Exact receipt:
`native-space-green-v2-import/RESULT.json`, SHA256
`42bdf940fec0ba94761abc456f1c366202ac83b9cd67e1b7f51e6f9937f0dbbc`.

The two unchanged old Legacy Space contract tests were also executed and both
failed at their original handled-true assertions. This is an actual 0/2 result,
not a release waiver: `native-space-old-contract-v1-import/RESULT.json`, SHA256
`4e7261b707e29c4fa30b92352dcf403b469e97c2da67396e58156cdf444e069e`.
Preserving next-word authority after Reset remains an unproved consequence.

Target-only guarded compilation passed in 181.857 s. Experimental IME SHA256:
`76eca112c20563c375705b4a86b93f530c2f5473ed66b296e23390176613e50a`.
Receipt: `native-space-artifact-v1-import/RESULT.json`. Source import contains
1483 exact rows; focused and build manifests have different serialization
hashes but identical parsed rows and file hashes/modes. Both original receipts
are retained and checked independently. The prototype remains private; ROOT
production source is unchanged. This is EXPERIMENTAL_NOT_RELEASE, not full
source acceptance, physical-client success or installation authority. The
one-attempt-per-field Space-DOM comparison is prepared; next-word Tab/manual,
actual keyboard and authenticated site fields remain NOT_TESTED. Installed79
is unchanged at this source-admission milestone.

The finite native-Space DOM comparison subsequently passed 3/3 fields in
11.229 s. Each field's first fresh sample already contained the exact word and
one space; the first sample after the next symbol preserved that space.
Receipt: `native-space-experiment-trial-2ace3c72ce634cd485ae3bd93b06c1da/TRIAL_RESULT.json`,
SHA256 `5971fdca903cb46d998d5bc1cc4c97fef252239954f1563eba9bf31f9b1a4cee`.
Matrix SHA256 `ed03e4464d6b0654b805f64bf7786951a5d21b8a1fc290bec80e07a87a57d397`.
All source/input pins remained unchanged and exact79 restoration passed.
Global IBus and L3/L1.1 remained intact; candidate authority was temporary and
limited to this causal experiment. This supports native Space as a transport
alternative for the measured loss, without identifying the actual Gecko branch
or proving all-field reliability. No full80 installation occurred. The next
finite check examines next-word completion and actual Tab; the two preserved
old contract failures and Reset/manual availability remain unresolved.

The same private IME then passed the separate finite next-word test 3/3 in
11.278 s: native Space is independently observed, a fresh current hint is
proved for the next prefix, and one actual Tab produces `тест почему `,
caret12, ended composition and preserved focus in input/textarea/editable.
The strict V17 freshness predicate was retained; no V18 retained-surface
waiver, authority override or Double Shift stimulus was used. Receipt:
`native-space-next-word-tab-trial-fda80a156f5041b9a76b1af7907436e1/TRIAL_RESULT.json`,
SHA256 `1e1c74dce6d7d40318396dc320bce0f407e44feebd07ca246b50a3cfacaeaab9`;
matrix SHA256 `f73b5f721496b130407328dc71ddcf51cd516b7d4a3ef9dc58bc00be07aceea9`.
All source/input pins and exact79 rollback passed. This proves the following
word's Tab in these six combined finite field cases, not last-word manual
authority, Backspace variants, real keyboard or authenticated site editors.

Read-only analysis locates the remaining predecessor loss at false Space
PRESS in `advance_context_reset_rereceipt_after_key`: the local native append
is not owned append, so the existing pending predecessor is cleared. The
next boundary still settles KnownStart, consistent with the measured next-word
Tab success. A prospective production migration must preserve an eligible
predecessor as unconfirmed and require the original strict real client receipt
before reuse; it must retain all old downstream/manual/negative checks.
Transport-specific false/noCommit assertions need an explicit decision and
must detect duplicate IME insertion, not waive the old semantic guarantees.
Read-only receipts: `native-space-diagnostic/NEXT_WORD_AUTHORITY_ANALYSIS.md`
SHA256 `93d80ee7b21f7da630862f015376eb39247c52ecf407a763d403c34e1b778190`
and `PRODUCTION_CONTRACT_OPTIONS_REVIEW.md` SHA256
`2e57542956adcaad42fcc839a4a6c2a8867080b3c1675b871798a4653701f13c`.
The production repair is NOT_IMPLEMENTED at this milestone; ordinary runtime
remains exact79.

Independent preflight review accepted the connected native transport proposal
with a required call-local eligibility binding captured before await from the
actual Legacy output and pre-effect managed mode. Post-effect false return,
exact one-Space delta, epoch+1, unchanged revision and current same-owner
KnownStart must be revalidated. No persistent flag or new authority owner is
needed. Receipt: `native-space-diagnostic/PREFLIGHT_INDEPENDENT_REVIEW.md`,
SHA256 `b837c7450465ce9d55e17a45678ec0af9ceab88e8b362b7ce25f88763db73ba0`.
Decision: `decisions/2026-10-06-native-space-observed-boundary.json`. This
authorizes one private connected RED/repair and explicit transport-test
migration; production import, complete source acceptance, physical acceptance
and ordinary installation still require their own results. The original
downstream/manual/negative semantics and failure ledger remain mandatory.

The first connected tests-only RED attempt did not reach runtime assertions.
Discovery compilation stopped on E0616 because residuals.rs accessed the
private completed_replay and strict_prefix_echo_revision fields. Actual
receipt: native-space-connected-red-v1-import/RESULT.json, SHA256
1fd3b3fe969174151288d7b39b86cbed47eb901fb8889fb5141c424d08d07505;
28.879 s, compile_succeeded=false, no test execution. The immutable A parent
and ROOT runtime source were unchanged. This is a test-authoring failure,
not a reproduced semantic failure or quality result. The next tests-only
repair must use existing observable predicates without widening production
field visibility. The causal marker and all prior controls remain required.

The corrected tests-only connected RED compiled and reproduced the exact
first loss in 35.401 s: 8 identities executed, one expected causal failure
(native Space erased eligible boundary predecessor), seven unchanged controls
PASS including all four physical_double_shift_owner_ controls. Receipt:
`native-space-connected-red-v2-import/RESULT.json`, SHA256
21337cf89a09a9378ca5bd41e006eee21e56ab5bc36947e4a16f5095af56490d.
Immutable A production was unchanged; compilation visibility repair used
existing observable no-replay and no inherited-display/manual predicates.
This establishes the causal regression detector, not repair success. Later
positive Reset/receipt/manual cells remain unexecuted after that first panic.
Runtime authority is unchanged; exact79 remains ordinary installed.

The private connected kernel passed its focused GREEN in 33.870 s: all five
identities passed, including the causal test with four internal cells
(ASCII/Unicode x ordinary/manual-suppressed) through actual Reset, exact
receipt, ManualToggle and authoritative visible-tail consumption; all four
unchanged physical_double_shift_owner_ identities passed. Receipt:
`native-space-core-green-v1-import/RESULT.json`, SHA256
d99b4daae990534fb3018ad174fc95f09bb91988cd15466e7a6f518d2c36e373.
Only the reviewed observation kernel and tests-only RED overlay changed on
immutable A. Callback-local eligibility and strict post-effect guards retain
existing pending provenance unconfirmed; original confirmation consumers
remain. This is focused source evidence, not complete affected-closure or
release acceptance. The conflicting old A pendingNone oracle awaits explicit
strengthening in the connected transport migration; it is not waived. ROOT
production and ordinary installed79 remain unchanged; no GUI experiment or
runtime authority change occurred in this source check.

The connected first affected-closure run executed all 117 unique identities
in 104.712 s: 115 PASS and two FAIL. Both stopped at the same old handled-true
expectation in the implicit ASCII Space helper, before their downstream
Backspace/command-gap semantics. Exact failures: residual_observed_boundary_
backspace_cannot_cross_a_command_input_gap and residual_observed_boundary_
backspace_keeps_start_and_retires_old_token. Raw receipt:
`native-space-connected-green-v1-import/RESULT.json`, SHA256
9462e38b742d1303c015968d172976f368dbdcf21647a0fba69f5cd87e54b5f8.
This is not connected PASS. One shared test-transport migration remains;
its original downstream guards must remain and all 117 will be rerun.
Actual canonical selection is 111 baseline plus six added identities,
without duplicates; the lexical index missed two implicit Space callers.
The compiled/formatted observation kernel hash is
633dac8291b22c44b8932e4171d15b10b38dd2c1458d0d409e5d204a84eac3cf,
distinct from frozen raw input 6fb00d28. All runtime source remains private;
ordinary installed79 and runtime authority are unchanged.

Connected Round2 passed all 117 exact identities in 104.515 s, with zero
failures/errors and unchanged compiled runtime kernel633dac. The only Round2
change was the shared Space setup; all non-Space/downstream assertions stayed
literal. Receipt: `native-space-connected-green-v2-import/RESULT.json`,
SHA2561477b664b6cd674af4af162cd58adc8e80562305ed11ea98e76540da5f29acbf.
Nine actual formatted source files were imported against their exact ROOT
before hashes; `native-space-connected-green-v2-import/ROOT_SOURCE_IMPORT.json`
records the import. Independent connected+Round2 review e8098dd15b2cf4621d89e4af84c6abe1a98e855ab931bfd6ff535cbf8fd85d46
found no additional source/authority blocker. Old failed receipts remain.
This establishes affected-source acceptance only. Complete source release
gate, changed-byte physical matrix, actual keyboard and ordinary installation
remain NOT_TESTED. ROOT source changes, installed79 does not; runtime authority
is unchanged. The full gate must retain all old canonical rows and zero ledger.

## Explicit affected-closure replan after full V16 — 2026-10-06

Full V16 did not pass: 3009 correctness/package identities selected, two
TD125 failures and 3007 passes; CHECK_CHANGED failed after388.802s, overall
472.991s. Receipt `release80-v16-import/RELEASE_GATE_RESULT.json`, SHA256
2d9912c3183dd3ceded78eaf80210704673db886c14f4c572d51b6b90846b671.
Raw IME log `release80-v16-import/bin-lay-ibus-engine.log`, SHA256
4c040edcc40b96fa23db1a88e1bca0826f7cea99b722d5757556bdbc4d55b8ab.
Both actual panics require handled=true at the uncapped Legacy NoApply Space
setup: td125_soft_reset_discards_owned_preedit_from_local_and_shared_authority
(line534) and td125_cursor_zero_backspace_cancels_owned_preedit_and_tracks_native_delete
(line593). Their later reset/delete/local/shared authority assertions were
not reached. This evidence establishes a missing transport-test migration,
not downstream semantic acceptance. Source graph and canon passed; later
full release stages/artifact and changed-byte clients remain unproved.

The previous two-round 117-identity closure plan is ended, not silently
extended. The explicit new plan in the owning ADR expands selection to
the complete TD125 legacy preedit module plus prior117, retains every old
identity and failure ledger, and allows at most two reviewed repair rounds.
Only those two direct transport expectations may migrate: require native
false, no Space Commit/Delete/Forward, exact Clear/Hide or no effects, one
local separator and unchanged downstream Reset/Backspace/authority guards.
No runtime change is currently indicated. After focused proof, run the entire
required source gate, then the changed-byte physical matrix and actual
keyboard. No failed assertion is waived; installed79 remains unchanged.

The explicitly replanned closure passed all136 unique identities in114.511s,
including the two previously failing TD125 sites and all19 module identities.
Receipt `native-space-full-closure-green-v1-import/RESULT.json`, SHA256
cedc88318aa6451ba168acbaf2c0eeb896c809f4fb325801088b15edc78807a4. Independent review
bfe13aa043003ceb8dccbadaa9eb70ead7f08cdd408f35b0351e52a727bc1ea9
found zero blockers; all original downstream semantic assertions remain.
Only the actual formatted test file was imported against parent5276a008.
The runtime kernel633dac and installed79 remain unchanged. This source-only
result does not establish full release or changed-byte physical acceptance.
Full source gate V17 is the next requirement; no failed receipt is discarded.

## V17 full-repeat failure and explicit dispatcher-boundary replan

V17 first correctness/package canonical pass was3009/3009 in399.612s.
FULL_GATE repeated canonical execution and failed on one pending-auto-undo
bridge test; overall886.238s, full gate401.900s. Failed receipt
`release80-v17-import-failed/RELEASE_GATE_RESULT.json`, SHA256
6b6c594fdf1a3c3af8581f2c6f8be5a2a5402a58b5692be7c63dfe57ab6e8950.
Actual isolated log `release80-v17-import-failed/failed-undo.log`, SHA256
0b5c6627b6a85eed4a4806ddee5d53f55ccef50704a4db4eb6a1896f182886c0
shows residual_pending_auto_undo_bridge_keeps_ime_ownership_without_delegation
at residuals5891: Ok(0,false), expected(1,false), elapsed0.01s. Ping and observer
assertions passed; later ownership/no-delegation/awaiting-exact assertions
were not reached. This is not a full source PASS or proven timeout cause.

The previous transport-test-only closure repair passed and is not silently
expanded into a runtime repair. A new explicit plan in controlled-manual-
dispatch-isolation ADR investigates the shared absent→served fixture boundary.
Static analysis identifies a final manually consumed Space RELEASE3001 that
may still reach the automatic object dispatcher after engine registration;
a duplicate may reuse its cached stamp, but the header has already been
settled; second reducer settlement refuses and revokes word authority.
The old failed run does not contain that serial/refusal trace, so original
causal timing remains UNKNOWN. A bounded controlled duplicate witness and
existing exact absent-dispatch fence will distinguish this mechanism before
repair. All semantic assertions, production guards, timers and failure ledger
remain mandatory; max two reviewed rounds. No runtime source change is
authorized by this fixture plan, installed79 remains exact and unchanged.

The new tests-only baseline handoff detector compiled and failed its exact
first assertion in33.736s; four unchanged physical-owner controls
passed. Receipt `undo-registration-red-v1-import/RESULT.json`, SHA256
64c26f67b081de2333b3f86199e0cd10e94f10b1c597e3e9d79d5736231bbfa7, statusOLD_FAILURE_REPRODUCED. Scope:
baseline known_engine leaves the declared final manual release3001 unfinished
at return. This does not reproduce or identify the historical V17 timing.
The later forced registered duplicate cell was not reached. Runtime source
and installed79 remain unchanged. GREEN137 is prepared, not yet accepted.

The reviewed registration-boundary GREEN passed137/137 in115.212s.
Receipt `undo-registration-green-v1-import/RESULT.json`, SHA256
19be9d8d469ea3c38bde137307b7ea6f6057a6631eb1d4c6e7b4ecd664dca95e. The new test actually reached its
controlled registered duplicate cell: same original tail/owner/epoch but
UnknownStart and revoked token after duplicate settlement; the strict bridge
refusal and empty output passed. Every previous semantic assertion remains.
Only actual formatted residual test source was imported against f9ebe2a4;
independent review a5419119 reports zero blockers. This supports the narrow
initial manual setup completion fence, not original V17 causal timing and
not later numeric/Reset/publication seams. Production source and installed79
unchanged. Full source gate V18, changed-byte Firefox and hardware remain
pending; no failed evidence was waived.

## V18 repeated canonical PASS, stale Clippy expectation blocks release

Both3010canonical runs passed. The actual full gate failed at default Clippy
afterwards, overall924.463s, full gate439.445s. Receipt
`release80-v18-import-failed/RELEASE_GATE_RESULT.json`, SHA256
597ae874b99f7997b1d33cac748d4a8afe510cf4bb34ee39fdd31dad037f945e. Its canonical_full field remains null because
FULL_GATE returned nonzero; the postfailure shared SUMMARY capture separately
reports3010/PASS, SHA256 26706d3440bd1a55c803aab3443e9cd82df58f3436c54c4df8def412194a18b2.
A guarded explicit bin Clippy reproduced exit101 in15.489s: obsolete
expect(clippy::collapsible_if) on composition_commit now unfulfilled.
Diagnostic SHA256 50d870da4dda17c61f328b324194167fc52e308112ea13499f2c5b6a149ba9ee. The old accepted bytes had
a nested branch; current native Space implementation no longer emits the lint.
Explicit decision remove-stale-composition-lint-expectation removes only
that annotation, preserving strict -Dwarnings, lint baselines and all runtime
statements. Independent review and strict all-target Clippy are pending;
then complete source and changed-byte physical proof remain mandatory.
Installed79 unchanged, no source release PASS is claimed.

Read-only current-hint audit also corrects an earlier overly broad lineage
claim: old observer-matrix V18 on full-source V15 had its first two Backspace
settlements UnknownStart. The Tab settlements KnownStart occur after accepted
boundary, not proof of pre-Tab KnownStart. Actual3/3 effect and old three
pre-Tab observer failures remain distinct. Source permits exact current
owned-preedit append under UnknownStart if all live scope/start gates pass.
Audit UNKNOWN_START_CURRENT_HINT_ADMISSION_AUDIT.md SHA6a1d3b47 and
V15_BACKSPACE_HINT_FRESHNESS_AUDIT.md SHA6a3441c8 are private read-only
evidence; current full-release V18 has no changed-byte physical result.

Annotation-only removal passed actual strict all-target default and research
Clippy in77.068s, including dead-code inventories532/356 and zero other
diagnostics, source/parent unchanged. Receipt
`stale-composition-expect-clippy-v1-import/RESULT.json`, SHA256
459aa389f257034d3a58afd7d0a974f05f4bc1e0698814eeffd9f8d4e1264220. Independent review59a25725 confirms
only the four-line expectation deleted and all other bytes literal.
Full release V19 is still required; installed79 remains unchanged.

V19 stopped at CANON in84.557s before canonical execution: the new lint
decision lacked required verification/not_tested text fields. This is a ROOT
document-format error; existing runtime and strict Clippy remain unchanged.
Failed receipt `release80-v19-import-failed/RELEASE_GATE_RESULT.json`, SHA256
08cbcd3036fdcdf49af3f10c763d63b635ee9a86bb3d2408401e6530ed85a05b. The missing fields now explicitly
record measured Clippy proof and untested full-source/physical scope.
No validation rule changed or failure waived; next full gate is V20.

## V20 full gate and actual installation80 — 2026-10-06

Latest runtime: **1.0.80 installed and loaded**, not an all-Firefox acceptance.
User repeated «установи 80» after the hardware prerequisite and install-without-check
option were explained. Decision:
[one local user-directed installation](decisions/2026-10-06-user-directed-version80-install.json).
The real-keyboard check is **NOT_PERFORMED_BY_USER_DIRECTION**, not PASS. The normal
AGENTS requirement and immutable original installer remain unchanged for future work.

Evidence directory:
`/home/local-user/.cache/lay/development/contextual-completion-20261005/20261004T211126Z/release79/firefox-repair-20261005`.

- Full `release80-v20-import/RELEASE_GATE_RESULT.json`: PASS,15 command stages,
  both canonical3010 PASS, no failures; SHA7685d75b6da165cae6c7eda937d58a29971299ce9b0d6fcc1a6f7db05e629767.
  Frozen1485-file `SOURCE_AFTER.json` SHA2b0a6cc5f5496815a2367325ff9f0f141c0844755630d1ed97d3a097fb7c1ae3.
- `V20_PHYSICAL_SUMMARY.json` SHA2d53e23ee18148c896be3408c78bf54eb4d7307f105ff54e1602c836db81ab50:
  same IME03a8 bytes, unchanged daemon79; all five exact79 rollbacks PASS.
  Space3/3; native-Space eight-pair3/3 (24 word/mode/icon transitions and next letter);
  active composition eight-pair3/3; retained proposal actual Backspace→Tab3/3 PASS.
  Wide87PASS/93executed,6FAIL,0BLOCKED; second-wordTab3/3, nonletters60/60,
  heldShift3/3 PASS. Original three strict DOM composition-event prerequisites stop
  before Tab; they remain FAIL. The separate actual-Tab control does not rewrite them.
- `V20_WIDE_FIRST_LOSS_AUDIT.md` and `V20_AUTOCORRECT_POLICY_READINESS_AUDIT.md`:
  all three correction cases retain `кторое `; the separator arrives. One Space
  result not_ready, two ready→SuggestOnly; target membership and exact refusal are
  UNKNOWN. The canonical config enables correction; no blanket Russian typo ban
  is established. These findings do not justify a literal exception or gate waiver.
- `USER_DIRECTED80_PRIOR_BINDING_FAILURE.json`: first installation admission
  rejected a draft pointing to initial79 rather than the actual accepted79
  frequency/command installation. No destination mutation. Corrected only the
  caller's prior receipt to pinned830b5c4a…; exact old-byte guards unchanged.
- Actual `ordinary80-install-109a2c631bd746c9bdf687efe77e8308/INSTALL.json`:
  PASS; SHAbd7b1757999440f739b033eec8b783fb3368c0deda1fe4068bd596e0433e4ad3.
  Three imported binaries plus metadata/tray version80; current IME and daemon
  /proc hashes and `lay 1.0.80` independently verified.35 protected paths, input
  sources/config and global IBus/L3/L11 preserved. Actual79 five-file backup and
  exact guarded rollback are in that transaction's `ROLLBACK_ACTUAL79.json`.

Not tested: genuine keyboard on V20; combined installed80 physical behavior;
Instagram/WhatsApp/every Firefox field. No full release publication or user
acceptance is inferred. These documentation outcomes were appended only after
successful transaction source closure; they do not change compiled V20 bytes.

### 2026-10-12 18:00 UTC — current remote native failures and bounded next fixes

Current private IME SHA-256 `0fec60dbac2dd1c80eeeb0a68cf9c73239ce5a42dc008233f3c2df0891937394`, daemon `e6cad39f961a0d044b9566bdeb490c56718ab7276a51f34fc3fd6a4f46c56674`: focused source **719/719 PASS**, remote Firefox standard **54/60 PASS, 6 FAIL**. All 30 active-word cells PASS; after Space 24/30 PASS. Normal receipt `firefox-standard-normal-6tanwkh5/RESULT.json` 19/20; private `firefox-standard-private-mldozt4e/RESULT.json` 18/20; fullscreen `firefox-standard-fullscreen-svmli4dp/RESULT.json` 17/20, all under `/home/e/projects/lay-development-runner/firefox-legacy-native-20261012.6onP5c`. These are new-byte results; older 56/60 results do not certify these bytes.

C09/C10 actual-panel receipt `firefox-c09-c10-54n29p7c/RESULT.json`: 4/6 cells PASS, Space fields 1/6 fail. Wire receipt `firefox-c09-wire-ty70xw2j/WIRE.json`, raw wire SHA-256 `756c1329a94d174519a171be6fd91cbbae795081f4a36a9b00d5f46912d11927`. First observed failures: same-owner Reset before handled closing Space release loses the actual confirmed snapshot; exact replay receives a whole planned raw prefix with cursor one character before its end and drops inert lineage. Raw SSI bytes for other separator failures remain unproven. Timing measures remote evdev write-return to actual AT-SPI target; physical kernel release timing NOT_MEASURED, trace completeness UNKNOWN. Chrome matrices and functional corpus NOT_TESTED: stock private launch failed before a usable window (X authorization errors and thread-limit error). No laptop install, delivery route or production session changed.

Plan review: closing-Space Reset carry 8/10, replacement-only inert off-caret retention 9/10, combined 8/10. Separate decisions record exact scope and causal tests. Private native-actor review pass 2 is 7/10; four receipt weaknesses (latched focus/input interruption, exact cleanup window identity, browser/body provenance, C09 input-mode agreement) are to be fixed before accepting new receipts. Two review passes exhausted for that actor; no third review claimed.

### 2026-10-12 — bounded closing-Space Reset and off-caret replay source PASS

New decisions `2026-10-12-owned-closing-space-reset-observation.json` and `2026-10-12-inert-replay-off-caret-receipt.json` implement only the first matched Reset of a held, managed closing Space with its original already-confirmed whole raw snapshot, and inert retention of exactly one already-observed replacement character right of the caret. Visible preedit is excluded: eligible clear has no asynchronous output; all other Reset paths need fresh client SSI. Original observation revision is unchanged. No new persistent owner/state, delivery route or safety bypass.

Causal A RED `run-5p_ecn8b/RESULT.json` proves original snapshot loss; B RED `run-19i1mgw_/RESULT.json` proves off-caret lineage loss. Latest focused source **721/721 PASS**: `/home/ubu/.cache/lay/development/run-scnveyr3/RESULT.json`, remote `/home/e/projects/lay-development-runner/run-4VdbH0/RESULT.json`, frozen archive SHA-256 `a94c7461d9550ad12cc1b31d0f2fde0b926aab9b8ddc2b120c22d5828909ed8f`. Positive proof includes actual source-lease acquisition and refusal of a second acquisition. Negative proofs include released/second Reset, missing/changed/revised snapshot, epoch, selection, command/navigation, sensitive content, pending append, visible preedit, observer-only later revocation and genuinely planned-but-unobserved replay prefix.

Fresh code review pass 1: A8/B9/combined8, no production defect, two proof gaps. Root chose a narrower no-visible-preedit scope and added the causal future-prefix test. Pass 2/2: **A9/B9/combined9, no blockers**, source files checked against frozen source archive. Native acceptance of these changed bytes is **PENDING**; prior native 54/60 FAIL scope remains separate.

### 2026-10-12 19:14 UTC — new-byte native baseline and publication-order experiment

Private IME SHA-256 `821209323cc1c4cdb49a9b84ecb403dd4dedf93d88833706284ecc4195ea20c0`, daemon `edf5810c9dbf17bf71e4814c3234d0b1f22417b71bf8eff0706578ff58286c5e`, candidate build `candidate-build-1791829967553251195/BUILD.json`. Focused source 721/721 PASS does not establish native acceptance. Firefox standard **52/60 PASS, 8 FAIL**: normal `firefox-standard-normal-z9kpwn61/RESULT.json` 16/20; private `firefox-standard-private-b5vacc_h/RESULT.json` 17/20; fullscreen `firefox-standard-fullscreen-u3qu375x/RESULT.json` 19/20. Active words 28/30; after Space 24/30. All receipts remain under `/home/e/projects/lay-development-runner/firefox-legacy-native-20261012.6onP5c`. Actual-panel C09/C10 `firefox-c09-c10-jc38t_3h/RESULT.json` **5/6 PASS, 1 FAIL**, active field7. Two later wire diagnostic cells passed; they do not replace the standard's failures.

Fixed functional baseline **150 rows: 77 PASS, 47 FAIL, 26 BLOCKED**, preserved at `job-results/firefox-current821-functional-baseline.json`, local mirror `/home/ubu/.cache/lay/development/remote-browser-evidence-20261012/current821/firefox-functional-baseline.json`. Seven selection rows fail in the actor's ambiguous GI method call rather than Lay. Prefix setup uses the wrong native punctuation key under RU, producing a different literal seed in all ten rows. Correct those harness defects before subsequent functional acceptance; do not remove the rows or rewrite the historical reports. Tab remains BLOCKED when the fixed required completion is unavailable. Other functional failures require causal diagnosis; aggregate raw FAIL is not a claim that every row proves a runtime defect.

Primary stock-IBus receipt `job-results/owned-append-publication-ordering.json` binds the actual messages to failed normal ACTIVE6. After actual CommitText and whole end-caret SSI, a new inactive completion appears before the matching literal release; the next SSI contradicts both raw text and caret, and strict second-receipt rejection discards the lineage. Source-prefix SHA-256 `786af55c763435b3dec4ee11a5f7fb52b73147b7f0dcb24f3ea4fda068ae8128`. Managed-append failure is outside the exact-replay inert off-caret proof. It is not yet established that all eight standard failures share this cause.

Decision `2026-10-12-inactive-completion-release-order.json`: A presentation ordering fence8/10, B separately proved contradictory-cache lineage5/10, C extra refresh2/10; fresh plan review8/10. Authority loss confidence9/10; early publication causing the client contradiction7/10. Root will first prove the held-key source failure, then test A without relaxing SSI, candidate, edit or verifier authority. Native success remains PENDING; contradictory SSI after fencing rejects this remedy hypothesis. Stock Chrome environment is still BLOCKED, its input matrices NOT_TESTED. Normal Firefox ordinary timing35/40 measured, p50=113.68ms, p95=122.00ms, max123.48ms; burst80/100 measured, p50=110.00ms, p95=118.31ms, max136.04ms. All timing remains evdev write-return to observed AT-SPI surface; kernel release timing NOT_MEASURED and trace completeness UNKNOWN. GUI/controller and their exact recorded process identities were stopped before the next remote source check. No production install, input-delivery route or laptop session changed.

### 2026-10-12 19:42 UTC — release fence source proof and selector review

Causal source RED `run-6vd2evmi/RESULT.json`:721/722 PASS, only completion-before-held-release proof fails. Initial release-fence GREEN `run-7kofiey1/RESULT.json`:722/722 PASS; frozen archive `be1c766c11a868e3ac883401553b1e678da52e00295c2f43208b4de2b4a8df8a`. Additional GREEN `run-6oafx7hi/RESULT.json`:725/725 PASS covers four no-connection legacy output entrances, both real overlapping release orders and11 stale-frame/revocation cases. All receipts are local under `/home/ubu/.cache/lay/development/` with remote-only execution.

Fresh read-only code review pass1/2: **7/10**, one P2 blocker. Up/Down refresh/cycle bookkeeping can record a new prediction before the release fence prevents its publication; a later boundary could learn that unseen prediction. Root adds a causal selector test and will guard before its refresh/cycle with the same existing predicate and client-navigation refusal. All prior invariants and source identities remain separate. Private initial build SHA `5f05a8dca7e5aaee458e328904025e1a10c70b7cd9cd634acf204652c34e3e35` has not been launched or promoted; it still precedes the selector fix. New native acceptance remains PENDING.

2026-10-12 19:46 UTC — inactive completion presentation ordering, selector blocker resolved.
The pass-1 Up/Down finding received its own causal RED (725 PASS / 1 FAIL,
`/home/ubu/.cache/lay/development/run-shb_ujsg/RESULT.json`) before the selector guard.
Guarding the existing selector before refresh/cycle prevents an unpublished target
from creating prediction or learning records. The existing client-navigation refusal
remains authoritative. Remote focused source is now **726/726 PASS**, receipt
`/home/ubu/.cache/lay/development/run-wu_3e0t_/RESULT.json`, remote
`/home/e/projects/lay-development-runner/run-mXMvRr/RESULT.json`, archive SHA-256
`2e71844c9f2b6836fc915cf33488c9302a05ddad5b505894909aa01fb539bb11`.
This source result includes both handled-release orders and 11 frame-revocation cases.
Fresh-context final review (pass 2/2) and private native verification remain pending.
The installed laptop runtime and IME delivery routes did not change. The earlier
Firefox 52/60 and functional 77/47/26 receipt remains the native baseline, not PASS.

2026-10-12 20:05 UTC — candidate6385 native result: **53/60 PASS,7 FAIL**
(normal18/20, private18/20, fullscreen17/20); panel4/6. Native changed-byte
acceptance is **FAIL**, despite source726/726 and final scoped code review9/10.
The bounded field6 diagnostic shows last literal release before new completion,
but initial Reset can erase the handled ledger and inverse after native Backspace
can lose the confirmed predecessor. Primary packet:
`/home/ubu/.cache/lay/development/remote-browser-evidence-20261012/current6385/native-inverse-backspace-reset-ordering.json`.
Actual full appended SSI precedes matched Reset and release; actual shortened SSI
follows native Backspace, but manual authority is refused after predecessor loss.
New owning decision `2026-10-12-managed-commit-reset-ack.json` records variants
8/2/3/1 and the required causal proof before production editing. The remaining
failures have not all been attributed to this mechanism. The functional run is
still active; Chrome remains environment-blocked. Laptop runtime unchanged.

2026-10-12T20:28:20.228398+00:00 — managed effect Reset causal proof. Remote focused source **726 PASS / 2 FAIL / 728** before production editing: `/home/ubu/.cache/lay/development/run-w9oscn04/RESULT.json`, remote `run-3bakzV`, archive `0dbd028845ad649983a6c8b497a1b98d61a9590f885c5d4b7c93daa546b65a87`. Both failures are actual callback schedules: consumed press release erased by matched Reset, and already-confirmed whole post-append SSI erased before native Backspace. Production change is pending GREEN/review/native evidence.
Firefox candidate6385 fixed functional baseline finished **85 PASS / 46 FAIL / 19 BLOCKED / 150**, remote `firefox-current6385-functional-baseline.json` under current RUN/job-results. All 15 plans per field were attempted; blockers and failures remain separate. Not all failures have a demonstrated shared cause. Chrome remains environment-blocked; its native matrices are NOT_TESTED.

2026-10-12T20:50:01.972340+00:00 — managed effect Reset ACK source **732/732 PASS**, remote FMT PASS. Receipt `/home/ubu/.cache/lay/development/run-u4e4v7gl/RESULT.json`, remote `/home/e/projects/lay-development-runner/run-Meq4tP/RESULT.json`, frozen archive `4d196d64df36cc8f74c6ecbd41a00019b29333147c40b35a319c3bb7e07d11c5`. Actual callback tests retain consumed key-up without SSI; preserve only first confirmed whole observation; require actual fresh shortened SSI after native Backspace. Eleven refusal variants, autorepeat, failed output and queued matched Reset callbacks passed. Layout generation received causal RED730/731 before its effect-identity guard. Existing callback now reports explicit matched/refused/cleanup, separating actual activation-bound release pairing from strict word/text authority. No secondary owner or client bypass. Source-only verdict: code review pass1 and private browser proof pending; prior native53/60 and functional85/46/19 remain FAIL baseline. Laptop runtime unchanged.

2026-10-12T21:02:26.372521+00:00 — managed Reset ACK code review pass1/2 **7/10, P1 blocker**: older authenticated Reset can import the latest token after observer-ahead ContentType revocation and carry an old observation. New actual-callback test covers changed purpose, password and change-back while local properties still show their predecessor. Causal RED pending before production guard; built08564ff0 candidate is NOT staged or launched. Existing native6385 FAIL receipts remain current.

2026-10-12T21:08:13.108555+00:00 — managed Reset ACK pass1 P1 closed by existing reset-only provenance check on the original predecessor before carry and after awaits. Causal RED731/733 includes all three ContentType/password/change-back schedules; other failure was strict detached transport reply ordering. Existing exact Ping acknowledgement fence closes that fixture race without weakening semantics. Remote GREEN **733/733**, FMT PASS: local run-z_votg3m/RESULT.json, remote run-o62yzf/RESULT.json, archive71f6fb1fd604f1c8618e600b5c782ff75264b6810be9ca971f48f28240f83f6c. Code review pass2 and exact new-byte native proof pending. Built085 was never staged; current6385 Firefox53/60 FAIL, functional85/46/19 baseline remains separate.

2026-10-12T21:36:08.974402+00:00 — private candidate965 native completed: Firefox standard **57/60 PASS,3 FAIL** (normal20/20, private20/20, fullscreen17/20), panel **6/6 PASS**, fixed functional **86 PASS/45 FAIL/19 BLOCKED/150**. Receipt `/home/ubu/.cache/lay/development/remote-browser-evidence-20261012/current965/firefox-current965-native-baseline.json`. All fields/modes attempted; no universal acceptance. Remaining burst loss first shows independently confirmed full native replay SSI, then extra Reset, then live-token manual authority refusal. Primary packet `current965/native965-replay-late-reset-loss.json`, raw archive `native965-firefox-raw-1791840837619141922/RAW.json` under remote RUN; trace-completeness UNKNOWN retained. New decision `2026-10-13-confirmed-native-replay-reset-ack.json` proposes bounded existing-effect ACK8/10 versus route/cache2/genericcarry1/TTL1; plan review and actual causal RED required before code. All exact GUI processes stopped; laptop runtime unchanged. Functional failures remain separate and not all attributed to this mechanism.

2026-10-12T21:46:50.450473+00:00 — confirmed native replay late Reset causal RED **733 PASS/1 FAIL/734**, remote FMT PASS: `run-ljbxvwly/RESULT.json`, remote `run-JTWVY7/RESULT.json`, frozen archive71ccd14a5af29f7f8c1acac20030e5e6bab259592ce3136742914bdd0caed7a8. Actual full SSI is confirmed and replay provenance consumed before extra actual Reset; the same original whole snapshot is then lost without new SSI. Fresh read-only plan review recommends existing receipt one-use metadata (9/10), only produced after successful full native confirmation/alignment/consumption; generic spent scope is not proof. Keep native verifier and scope expiry; no scope retirement, route change, second owner or generic released-key carry. Implementation/GREEN/code review/native proof pending. Current965 native remains FAIL57/60, panel6/6, functional86/45/19.

2026-10-12T21:53:31.356451+00:00 — confirmed native replay Reset ACK source **738/738 PASS**, FMT PASS, fresh read-only code review pass1/2 **9/10, no material blockers**. Receipt `/home/ubu/.cache/lay/development/run-7uzynhst/RESULT.json`, remote `/home/e/projects/lay-development-runner/run-k1i9on/RESULT.json`, archive42a7f0b6774983c729673e741ca414ac005f17e5973444f584aeece1f03e7dea. Existing pending receipt gains one-use focus/lease/layout metadata only after full actual SSI/alignment/successful consume. First matched Reset replaces it with None, even missing snapshot/output; identical SSI cannot rearm. Spent local/shared complete current replay scope remains intact for the native delivery verifier. Original whole snapshot/revision and predecessor Reset-only chain checked before/after awaits. Sixteen refusal variants, actual ContentType/password/change-back, single-use SSI and missing-output cases pass, together with unchanged held managed/Space negatives. New private build and changed-byte native proof pending; previous965 Firefox57/60 and functional86/45/19 remain FAIL baseline. Laptop runtime unchanged.

2026-10-12T22:06:10.366903+00:00 — candidateaa4 changed-byte native **54 PASS/6 FAIL/60**, panel6/6. Normal19/20, private19/20, fullscreen16/20. Existing candidate is rejected; source738/738 and scoped review9/10 do not certify windows. Receipt `currentaa4/firefox-currentaa4-native-baseline.json` under private remote evidence; raw exact `nativeaa4-firefox-raw-1791842724037470179/RAW.json` under RUN. Functional150 is NOT_TESTED for this rejected experiment (first30-slot report initialized but zero actions; remaining120 unstarted), prior965 full86/45/19 retained. Bounded actual full SSI→lateReset→manual refusal persists; hypothesis generic fully settled producer bypasses pending.completed_replay branch. New actual callback causal test added (739 source selection) and running before production change. All exact GUI processes gone; laptop runtime unchanged. Trace completeness UNKNOWN retained.

2026-10-12T22:08:29.721396+00:00 — generic fully settled native producer causal RED **738 PASS/1 FAIL/739**, FMT PASS. Receipt `/home/ubu/.cache/lay/development/run-zn4fsc19/RESULT.json`, remote `/home/e/projects/lay-development-runner/run-v8gsir/RESULT.json`, archive62d597e1ce84eb8514600d11a79e961466eb3d5bdb520cf98dc325c0154f0129. Actual generic full SSI confirms target after all native callbacks settle; pending.completed_replay seed is absent, so first C implementation did not create its effect ACK. A later actual Reset drops original snapshot. Minimal refinement selects existing complete current paired replay provenance in that same actual confirmation function; original prefix/caret/selection, reducer epoch alignment, reset-only provenance and successful consume remain required. No new owner or delivery route. New GREEN, final code pass2 and changed-byte native proof pending; candidateaa4 rejected54/60 preserved.

2026-10-12T22:12:55.036518+00:00 — producer refinement source **738/739**, new fully settled positive/32 refusal variants PASS; original closed-word/preedit return regression FAIL. Existing test `firefox_completed_boundary_replay_waits_for_final_exact_receipt` shows capability cleanup erased a separately confirmed ordinary SSI solely because the new native ACK marker existed. Minimal fix spends the ACK marker while preserving its independently confirmed whole receipt; actual incomplete replay seed cleanup remains unchanged. No test expectation, deadline or verifier weakened. Candidate remains unaccepted; new GREEN/codepass2/native proof pending.

2026-10-12T22:18:40.852688+00:00 — final confirmed native replay Reset ACK source **739/739 PASS**, FMT PASS; fresh read-only code review final pass2/2 **9/10, no material blockers**. Local `/home/ubu/.cache/lay/development/run-fxyxkhcf/RESULT.json`, remote `/home/e/projects/lay-development-runner/run-iN8RBM/RESULT.json`, archive4a26cf32a8a5a436098b6534fd13dd4a3deb2c8a99c0fca645938da88ee1c35d. Both original full SSI producers require current paired completed provenance, whole prefix/caret/selection, reducer alignment and successful consume before one-use ACK. Capability cleanup clears only that marker, retaining independently confirmed ordinary SSI; all original regression assertions/deadlines unchanged. Final private build running; changed-byte native acceptance remains NOT_TESTED. Rejected aa4 standard54/60 and functional150NOT_TESTED retained. Delivery routes and laptop runtime unchanged.

2026-10-12T22:21:56.507099+00:00 — final739 private build completed181.972s and staged only in remote RUN. IME `be71027845b3e97dbfdd5787d8ab53788cd1dc411c5f537c7aa2fdfad8f7d10f`, daemon `8743552cbb1509424cd717e3487dd9346035f2ee2e41bab83fe0015025f5f758`, immutable `candidate-build-1791843670251005239/BUILD.json` under RUN; source archive `4a26cf32a8a5a436098b6534fd13dd4a3deb2c8a99c0fca645938da88ee1c35d`. Setup/preflight passed, Firefox standard+panel+fixed functional queue running. Actual native verdict PENDING; no acceptance claim. Runtime authority/delivery routes/laptop installation unchanged.

2026-10-12T22:29:51.316911+00:00 — native be710 first new shared loss: actual native Backspace shortened SSI is confirmed, then late matched Reset clears it (normal active field10; native trace hash6013b418c893f73780527abcc6eefd3957b1af94d9050d93ce98651c661f34fe). Exact receipt `job-results/nativebe710-backspace-late-reset-causal.json` under RUN. Native erasure is a separate producer from completed exact replay C. Plan ADR2026-10-13-native-erasure-confirmed-reset-ack recommends bounded one-use erasure receipt ACK (9), forced SSI/route change(2), arbitrary snapshot carry(1), suppress Reset/longer timer(1). No production editing yet; plan review pending. Full native matrices continue; rich separator mismatch separate and wire unproven.

2026-10-12T22:37:31.949608+00:00 — final739 be710 changed-byte native FAIL: standard **52 PASS/6 FAIL/1 BLOCKED/1 NOT_TESTED/60**, panel **5 PASS/1 FAIL/6**, fixed functional **86 PASS/45 FAIL/19 BLOCKED/150** (all150 attempted). Exact receipt `/home/ubu/.cache/lay/development/remote-browser-evidence-20261012/currentbe710/firefox-currentbe710-native-baseline.json`, immutable raw `nativebe710-firefox-raw-1791844517133584746/RAW.json` under RUN; owned GUI scope empty before freeze. Speed coverage `/home/ubu/.cache/lay/development/remote-browser-evidence-20261012/currentbe710/nativebe710-speed-coverage.json`: standard374/420 measured p50=108.58,p95=118.09,max=138.08ms; panel43/48 p50=111.53,p95=117.08,max=119.64ms; functionalDS101/210 p50=112.28,p95=144.42,max=159.37ms. Missing timings retained; kernel-release NOT_MEASURED, trace completeness UNKNOWN. Stock Chrome after private mutter-x11-frames still exits-11 before window; zero input keys, all216 Chrome cells NOT_TESTED, environment cause unknown. Native Backspace late-Reset D plan review9/10; actual causal test740 prepared, production unchanged since C. Separate rich SSI mismatch waits for exact wire analysis. No delivery route or laptop runtime change.

2026-10-12T22:54:35.258912+00:00 — native Backspace erasure causal RED **739 PASS/1 FAIL/740**, FMT PASS; local `/home/ubu/.cache/lay/development/run-kevupqdf/RESULT.json`, remote `/home/e/projects/lay-development-runner/run-uYEhdc/RESULT.json`, archivefbd23df28f5f416df931d968a29bab9fad12b5090fcdbd2b4b6623ca06795380. Original shortened actual snapshot missing after late matched Reset. Production prepared with inert whole actual pre-erasure seed then first full actual shortened receipt ACK using its current token only for matched Reset. First contradictory SSI spends seed; older generic predecessor/C06 and C spent paired replay scope unchanged. Seven new adverse/effect tests prepared (747 focused correctness selected), source/native verdict pending. All owned Chrome debug processes/scope empty after retained zero-key crash diagnostics; child default schemas also exit-11, null indirect call observed by GDB, actual crash cause UNKNOWN. Laptop runtime and input delivery routes unchanged.

2026-10-12T22:58:16.563116+00:00 — D first implementation gate **746 PASS/1 FAIL/747**, FMT PASS; local run-jdx90ps4/RESULT.json, remote run-2ADOHX/RESULT.json, archive571b3a4c163532988725672e4af1632eb519c9ee3800d495aeec14a84590038e. Original causal and unchanged739 tests PASS; new Reset-before-receipt test exposed unconditional word-generation equality despite matched Reset legitimately advancing lineage. Minimal repair retains direct same-generation revalidation OR exact inert predecessor pinned by matched Reset plus unchanged reset-only source predicate; whole actual text/geometry, epoch/revision/current pending identity still required. No test expectation or predicate weakened. Fresh D reviewpass1 independently located same cause; remote GREEN pending. Native be710 rejected, Chrome zero-key environment crash remains UNKNOWN, all216 Chrome native NOT_TESTED.

2026-10-12T23:00:33.580468+00:00 — native erasure D source **747/747 PASS**, FMT PASS, fresh static code reviewpass1/2 **9/10 no material blockers**. Local `/home/ubu/.cache/lay/development/run-cah22ovv/RESULT.json`, remote `/home/e/projects/lay-development-runner/run-OvGO7U/RESULT.json`, archive98c334b015c3c782c16e78881c9bed802b6e3418b69638981a5cd4146acf21cd. Exact pinned Reset bridge handles legitimate generation advance; whole actual geometry/prefix/current pending, one-use/no-output/no-snapshot/content change-back and original tests PASS. Changed-byte private build pending; native acceptance NOT_TESTED for these new bytes, rejectedbe710 and missing timings retained. Delivery routes/laptop runtime unchanged.

2026-10-12T23:05:15.429456+00:00 — remote private D build182.199s, immutable `candidate-build-1791846216405329810/BUILD.json` under RUN; IME706ec6cb1a4c356ecad3766815ca0563b6ec5526cb2907b60a5a3197ad16a7f8, daemon2467d7aedf603f965e25403e716dd3eb0f546a3cd842513b227fd11c144d9250, source747GREEN98c334b015c3c782c16e78881c9bed802b6e3418b69638981a5cd4146acf21cd. Only private staged files updated after verified backup/exact GUI empty. Full FF60+panel6+150 queued with unchanged functional body and passive exact Engine SSI/Reset wire recording (timing observer overhead not separately measured); actual changed-byte native verdict PENDING. Chrome detailed zero-input GDB diagnostic queued after native proof. Source/build OFF while GUI lease active; laptop runtime/delivery routes unchanged.


2026-10-12T23:42:18.802828+00:00 — exact D706 Firefox native proof **FAIL**. Standard55 PASS/5 FAIL/60, ordinary120/120 PASS; C09/C10 panel6/6 PASS. Initial fixed functional79 PASS/40 FAIL/16 BLOCKED/15 NOT_TESTED/150 remains retained; unchanged supplemental field10 adds8 PASS/3 FAIL/4 BLOCKED, unique150 coverage87 PASS/43 FAIL/20 BLOCKED. No expected target or functional body changed (SHA45ae44b96e767a61bec7fa6cbf5c82888fe65368fc3798d42b0ebd81c6cea5a6). Exact proof `/home/ubu/.cache/lay/development/remote-firefox-setup-j2he3qf6/native706-firefox-proof.json`, remote same filename under RUN/job-results. Raw `native706-firefox-raw-1791848428889609934/RAW.json`; GUI732253 owned scope verified empty before freeze. Delivery routes/runtime authority/laptop runtime unchanged.

Speed D706: standard406/420 measured,14 missing,p50=109.80,p95=118.59,max=133.90ms; panel48/48,p50=111.06,p95=119.26,max=123.11ms; functional DS103/210,107 missing,p50=113.28,p95=146.26,max=165.23ms; Space37/50,13 missing,p50=17.55,p95=23.65,max=24.78ms; Tab0/10,selection0/10. Clock remains evdev write-return to first actual AT-SPI target, kernel-release NOT_MEASURED; passive-wire overhead NOT_MEASURED; asynchronous trace completeness UNKNOWN. No speed improvement claim.

D706 rich7 diagnostic: five independently seeded existing focused actors retained. Exact Engine wire serial48937 Reset followed48938 actual SSI `привет\u00a0`,7/7,no selection; primary same reset rejected7/7 despite current identity. Existing ASCII-space/NBSP acknowledgment succeeded in preceding epoch12622 but failed at epoch12637. Exact guard responsible still UNKNOWN; no new production repair. Wire `firefox-receipt-wire-xl38kbni/WIRE.json`, primary `firefox-rich7-primary-snapshot-do_2z9ip/SNAPSHOT.json` under RUN.

Chrome environment experiment: vendor155.0.8059.39 null indirect GTK call traced to `gdk_screen_get_default` fallback while GTK4 gtk-xft-dpi=-1; official vendor source gtk_ui.cc confirms fallback. Private standard `config/gtk-4.0/settings.ini` sets98304 (96DPI); fresh GTK4 reads98304 and stock Chrome opens with identical flags/input environment/private IBus. Probe input keys0; delivery/authority unchanged. Both browsers simultaneously hit Tasks512 thread ceiling; retain resource ceiling and run Chrome separately. Receipts `/home/ubu/.cache/lay/development/remote-firefox-setup-j2he3qf6/chrome-gtk4-dpi-before.json` and `/home/ubu/.cache/lay/development/remote-firefox-setup-j2he3qf6/chrome-gtk4-dpi-environment-fix.json`. All216 Chrome native slots NOT_TESTED at start of standalone phase; opening alone is no acceptance proof.


2026-10-12T23:55:24.300218+00:00 — E actual callback causal RED747 PASS/1 FAIL/748, FMT PASS; local `/home/ubu/.cache/lay/development/run-t3ucaxhe/RESULT.json`, remote run-POHj5u/RESULT.json, archiveb1b55f1bff14b5353667514e784ed39271fd828eda69808432a5d3d35d4a0121. Final raw NBSP receipt is rejected after valid source manual handoff and native replay with no new target source-surface echo. Source receipt stores tail/token/revision/expiry, but no whole external prefix. Refined minimal plan adds optional validated full source prefix to the SAME receipt at publication and carries it into existing paired scope; existing narrow route, validators/expiry/revision/C-D one-use remain unchanged. Native causal link to every burst or functional failure unproved. Decision `2026-10-13-inherited-replay-full-prefix-proof.json`; no E production edit yet. Chrome standalone stock opens ten fields at Tasks259 but GNOME spawns replacement IBUS -r --xim (PPID757205), original757193 zombie/custom socket lost, before input; zero keys,216 native NOT_TESTED, receipt `native706-chrome-startup-raw-1791848847565997194/RAW.json`. Owned GUI756723 closed/scopeempty. Test setup discovery repair pending; no product delivery/laptop change.

2026-10-13T00:19:31.776626+00:00 — E source **751/751 PASS**, remoteFMT PASS; local `/home/ubu/.cache/lay/development/run-ems8366g/RESULT.json`, remote run-xecacu/RESULT.json, archivee9218ccb00a87eda778dcedbe6805a2b0eb0ff8871697a7a661e76e791b9767e. Same existing source receipt retains validated optional raw whole-prefix and transfers it into paired replay scope under unchanged tokens/revisions/expiry. No target SSI manufactured or delivery rule relaxed. Three positive shared fixtures use their selected actual external prefix at source; all original assertions retained, foreign-prefix negatives remain. A fourth sporadic native-erasure source failure passed isolated and final751 without production change. Initial compiled747P4F receipt retained in ADR. Manifest adds exactly4process tests (total3116/correctness3054/process301); original rows/isolation preserved. Codepass1=9; final scoped E review2 and private build pending. D706native Firefox55/60 FAIL and87/43/20 functional remain current; E nativeNOT_TESTED. Private common GNOME-owned IBus discovery setup prepared with zero-key browser IC attachment proof and pinned bus owner guards; all216 Chrome native stillNOT_TESTED. No laptop runtime change.

2026-10-13T00:30:10.615458+00:00 — E final source code review **9/10 pass2/2**, no material blockers. Immutable private build181.995s `candidate-build-1791850863976247797/BUILD.json` under RUN; IME36952d5576dbd6014ccf0059321cd6dbfe8680d8153db91bab36b2168833893d, daemon4e822d22f070b3aa687e5e9a2691e5b1b40b892723850aeee830744ef42c3bef. Both privately staged after verified oldGUI scopeempty and backup `staged-before-inherited-prefix-1791851073683404343`; no installed runtime change. E nativeNOT_TESTED. Private readiness review7 then8 exposed unsupported IBus PID method and stale IC publication after blocking observation. Root replaced method with kernel UNIX_DIAG socket peer proof plus actual current Gtk browser IC/Engine FocusInId, pinned bus/process and final same window/field/IC/RU checks; pidfd-bound nonchild cleanup closes reuse races. Remote AST and actual socketpair probe PASS, functional body SHA45ae44... unchanged. Private desktop topology changes from competing fixed-address bus to stock GNOME-owned discovery; product delivery APIs/authority unchanged and laptop parity not claimed. Zero-key attachment remains unproved until actual browser phase. Source closure daemon+22contract targets active; lexical_compiler separately requires lexical-compiler feature, not an empty default-profile PASS.


2026-10-13 00:50:54 UTC — E source closure: daemon plus22 contracts406 selected,405 initialPASS and one static Space signature assertionFAIL. The latter still checked the pre-keycode helper and split line; only source matching updated to current awaited helper, all semantic boundary assertions retained. Focused retest1/1 andFMT PASS (`run-a0ts1y42`); other405 fresh results reused (`run-rgar_ble`, remote run-8yriho). Explicit lexical-compiler feature gate1/1 PASS,0ignored/filtered; no empty default profile accepted. E core751/751 source evidence unchanged.

Private Chrome preparation: current stock GNOME bus initially lacked Lay due cached registry; private vendor `ibus write-cache` fixes catalog while preserving stock components. libdbus99-byte path limit resolved for passive monitor by the SAME socket relative path (actual absolute102); runtime discovery unchanged. Actual Chrome IC is `gnome-shell`, confirmed by FocusInId and kernel Chrome/GNOME/IBus/IME peer chain. Foreign IC.GetEngine is disallowed by vendor IBus, so zero-key observer uses actual focused RU engine path, global descriptor and pinned IME bytes/peer, with final same field/window/IC freshness. Attachment `PASS_SETUP_ATTACHMENT_ONLY`, input_keys0; receipt `/home/ubu/.cache/lay/development/remote-firefox-setup-j2he3qf6/chrome-stock-gnome-setup-attachment.json`. Old failures/raw preserved; private harness two review passes remain7/8, no third score claimed. Chrome full60+panel6+fixed150 jobs100–108 now queued/running on private E36952… bytes, verdictPENDING; no laptop input/runtime change or delivery route change.


2026-10-13 01:07 UTC — current E369 Chrome native **FAIL**. Unique standard60:20PASS/38FAIL/2BLOCKED, panel6:3PASS/3FAIL, fixed functional150:96PASS/43FAIL/11BLOCKED. Exact proof `/home/ubu/.cache/lay/development/remote-firefox-setup-j2he3qf6/native369-chrome-proof.json`, remote same filename under RUN/job-results; final mutable metadata/log raw `native369-chrome-raw-1791853639321307194/RAW.json`, owned Chrome scope812797 verified empty. Standard speed158/420 measured,p50=92.70,p95=113.11,max=122.66ms; panel27/48; functional DS129/210,Space38/50,Tab0/10,selection10/10. Missing samples remain missing; kernel-release NOT_MEASURED, trace continuityUNKNOWN. No speed-improvement or laptop-parity claim.

Private first refusal20NOT_TESTED retained; exact AT-SPI Incognito suffix differed from compositor title, source observer now matches that observed complete title and retains PID/foreground-window/active-frame guards. Supplemental20 allFAIL from actual client PRIVATE hints2112/current sensitive policy; no policy weakened. Normal/fullscreen each10activePASS,9SpaceFAIL and1search Space seedBLOCKED. Common first loss measured: completed native replacement callbacks precede ordered SSI5/5,6/6,7/7; second strict-prefix receipt revokes remaining replay lineage before final full actual snapshot. Exact diagnostic wire `firefox-receipt-wire-dx734unk/WIRE.json`,SHAad5f45ad327f8b7073bb3f1a211189430d196581891bfc717459f54e7334e2ba. Minimal F proposal in `2026-10-13-native-replay-progress-receipts.json`; no F production edit yet, causal source fixture required. Firefox same E bytes remainsPENDING, isolated phase starting.


2026-10-13T01:35:30.124569+00:00 — E369 Firefox same-byte native FAIL: standard58PASS/2FAIL/60 (normal18/20,private20/20,fullscreen20/20), panel6/6PASS, fixed functional87PASS/44FAIL/19BLOCKED/150. Proof `/home/ubu/.cache/lay/development/remote-firefox-setup-j2he3qf6/native369-firefox-proof.json`; immutable raw `/home/e/projects/lay-development-runner/firefox-legacy-native-20261012.6onP5c/native369-firefox-raw-1791855295972601475/RAW.json`. Standard DS414/420 timings,p50=110.41,p95=120.88,max=135.01ms; panel48/48; functionalDS100/210,Space37/50,Tab0/10,selection0/10; missing timings retained, kernel-release NOT_MEASURED. Stock Firefox Gtk actually uses IBus portal because private IBUS_USE_PORTAL=0 is present (vendor treats presence as enabled); dual passive FocusIn monitors identify browser sender PID830595 and distinct portal/IBus context paths plus owned vendor portal/broker/kernel peer chain. Setup observer refined only; product/env/routes unchanged. Owned829920 GUI scope verified empty before source work. F plan review1=8/10 clarified current KnownStart/no-Reset final vs original UnknownStart/Reset alignment; actual callback causal752 gate now RUNNING, production F edit NOT_STARTED. Other functional refusals are separate evidence, not assigned to F.


2026-10-13T02:09:13.337838+00:00 — F actual source RED751/754 (`run-bn0msgwv`, remote run-4XdLiS): current KnownStart allows realManual before full native client SSI and after regression despite inert pending. Original751 tests PASS; no native F bytes built/staged. Added same current completed-replay confirmation check to existing ManualToggle/VisibleTail admissions, with original whole snapshot and existing one-use native ACK; generic non-replay guards unchanged. Positive source NBSP fixture now asserts actual existing planner ASCII Space target; old Reset raw-NBSP contract unchanged. Prior compile error E0616 had zero executed tests and is separately retained. Two F source reviews9/10 preceded this final consumer guard; no later score claimed. Fresh754 executable gate RUNNING; delivery/runtime authority unchanged. Decision: `decisions/2026-10-13-native-replay-progress-receipts.json`.


2026-10-13T02:38:58.064779+00:00 — F focused diagnostic4/4 PASS (`f-focused-mgiat_xr/RESULT.json` under private remoteRUN), using existing hermetic process runner. Whole target16 variants cover noReset/Reset, empty/Unicode external prefix, source Space/NBSP, actual target Space/NBSP; every whole final explicitly has native ACK, spends both equal scope copies, duplicate final carries same ACK without new suppression revision/epoch, then real consumers delegate. KnownStart-only native admission closes causal pre-final bypass; original UnknownStart test passed.19 contradictions forbid native F ACK. Independent old Reset fresh exact owned-tail authority is explicitly tested separately and remains permitted, with ACK absent. Extra key seed effect is separately consumed before consumer no-effect check. Added existing complete current boundary provenance to owned-Space NBSP acknowledgment; original whole-prefix/caret/selection/pair/expiry guards retained. Full754+FMT now next gate; private candidate remains oldE369 and nativeF NOT_TESTED. Two earlier F9/10 source reviews precede these supplemental guards; no third score claimed.

2026-10-13T02:54:53.693834+00:00 — F final source **754/754 PASS**, remoteFMT PASS, local run-wcpiedjc/RESULT.json and remote run-3grlen/RESULT.json, source archive ddd11f4f0c88ff997469672993ef4b77a893142720c4bb47bdd06a100df79dc8. Separate context-free supplemental consumer review pass1 **9/10, no blockers** covers later KnownStart0 admission/confirmed duplicate carry/raw NBSP ACK and current final fixtures; earlier F producer reviews remain 2/2, without a third producer score. Immutable private build182.154s candidate-build-1791859642943640689/BUILD.json under remote RUN; IME41443ef97c6c0add6776f6ea6c8de47ff63646e6d8ddb662e6a685edfdb43bda, daemon687912d4334cb99913ed227a24d796f1a086287900185d0990a6bdb9474db654. Staged after exact old GUI scope empty and verified E backup staged-before-progress-receipts-1791860051555324067. Changed-byte native acceptance **NOT_TESTED**; both prior E Firefox and Chrome FAIL matrices retained. Delivery routes/runtime ownership unchanged, no laptop installation. Native60+panel6+fixed150 with required/measured/missing speed coverage next; TD121 stays open.

2026-10-13T03:15:01.541450+00:00 — F414 Chrome native **FAIL**, standard20 PASS/38 FAIL/2 BLOCKED/60, C09/C10 panel3 PASS/3 FAIL/6, fixed functional96 PASS/42 FAIL/12 BLOCKED/150, all150 attempted. Exact proof `/home/ubu/.cache/lay/development/remote-firefox-setup-j2he3qf6/native41443-chrome-proof.json`, raw `native41443-chrome-raw-1791861061998153359/RAW.json` under RUN; GUI907333 scope empty before freeze. Speed DS standard158/420 p50=92.276,p95=112.166,max=125.648ms; panel27/48 p50=89.859,p95=107.482,max=112.553; functional129/210 p50=91.480,p95=124.394,max=133.573. Space38/50 p50=19.402,p95=23.752,max=24.775; Tab0/10, selection10/10 p50=20.157,p95/max=22.535. Missing timings retained, kernel-release NOT_MEASURED/traceUNKNOWN, no speed improvement claim. Final native receipt now confirms whole replacement (F progress mechanism active), but after actual managed probe/Backspace a second ManualToggle delegates then factory VisibleTail is passive:unknown-context. Exact separate diagnostic `firefox-receipt-wire-c5csr4o6/WIRE.json`, SHA86256f339f36d6cdda7c0a532cf4a8c61645478532184f551e831550569dcf52; no SSI after probe/Backspace in captured complete isolated sequence. Precise new guard remains UNKNOWN until causal TDD. ChromePRIVATE intentional Sensitive policy and other functional causes remain separate. Firefox next, TD121 open; delivery/runtime ownership/laptop installation unchanged.

2026-10-13T03:35:38.967274+00:00 — F414 Firefox native **FAIL**, standard59 PASS/1 FAIL/60 (normal20/20, fullscreen20/20, private19/20; private Space field10 first burst never transforms despite preceding ordinary2PASS), C09/C10 panel5 PASS/1 FAIL/6, fixed functional87 PASS/44 FAIL/19 BLOCKED/150. Proof `/home/ubu/.cache/lay/development/remote-firefox-setup-j2he3qf6/native41443-firefox-proof.json`, raw `native41443-firefox-raw-1791862238737626823/RAW.json` under RUN, GUI921407 scope empty before freeze. Speed DS standard415/420 p50=110.598,p95=122.423,max=149.831ms; panel42/48 p50=113.398,p95=121.448,max=125.545; functional102/210 p50=114.853,p95=141.508,max=151.930. Space37/50 p50=18.147,p95=24.383,max=26.015; Tab0/10, selection0/10. Missing timings/nonPASS preserved, kernel-release NOT_MEASURED/traceUNKNOWN. Both changed-byte native verdicts FAIL; normal/fullscreen specific success is not universal acceptance. Source754+supplemental review9 remains valid, source/build next only after exact GUIoff. Next causal fixture reproduces actual managed native probe+Backspace without another SSI between confirmed target and subsequent ManualToggle/VisibleTail capture. Exact responsible guard still UNKNOWN; original typing/precognition configuration must be represented. TD121 stays open, no laptop/runtime delivery changes.

2026-10-13T03:54:40.370784+00:00 — H actual managed-preedit probe causal **RED4 PASS/1 FAIL/5 focused**, receipt h-focused-dg2gqi6k/RESULT.json under RUN. Source full actual snapshot is lost at first uncommitted owned-preedit letter, although no CommitText/DeleteSurroundingText occurs; handled local Backspace restores mirror to original closed word but factory capture remains passive:unknown-context. Root is unconditional raw snapshot invalidation in preedit::push_tail_char, a mirror-only producer wrongly treated as committed document mutation. Recommended9: retain SAME actual snapshot only for existing owned legacy-preedit mirror input; force-refresh6, weakened source/factory guard1 rejected. H runtime not yet edited; plan ADR2026-10-13-owned-preedit-keeps-committed-client-receipt, review pending. Prior choreography timeout retained separately, not causal proof. Both F native FAIL/full source754GREEN retained. No route/authority/laptop change.

2026-10-13T04:07:12.805327+00:00 — H plan review1 scored9/10, zero blockers. Explicit producer split implemented: ordinary document append always invalidates actual SSI; only existing owned legacy-preedit end-append retains the SAME raw observation using the existing mirror updater. No new owner/state, route, timer or source guard. Positive proof covers12 Reset/assistance/prefix/NBSP variants; actual Space commit and second unhandled client Backspace negative controls added. Source/full756 and changed-byte native remain PENDING, prior F failures retained.

2026-10-13T04:16:59.844443+00:00 — H second causal guard pinned in h-focused-uii6c80u/RESULT.json: only Reset=true fails; actual raw snapshot/predecessor remain current, but owned local erasure requires custom exact-refresh capability. Actual stock Chrome advertises caps41 with SurroundingText and no custom refresh bit. Minimal H decision uses the existing current observed start witness for local-only erasure, while retaining exact raw geometry/identity/revision/selection/token/settled-output proof. No fake capability or fresh SSI; focused/full gates and plan pass2 pending. Actual Space/native deletion negative final factory refusals PASS; delegation3 alone is not client acceptance.

2026-10-13T04:21:38.839124+00:00 — H plan pass2/2 scored9/10, zero blockers after capability41→40→41 witness retirement. No native ACK copy/rearm or additional source owner; final local cancellation uses the existing ordinary confirmed-observation path. Focused6/6 PASS h-focused-yhpje5ul before final capability negative addition. Final source756+FMT and new context-free code review are running/pending; changed-byte native remains NOT_TESTED.

2026-10-13T04:28:56.354252+00:00 — H final source **756/756+FMT PASS**, local run-itjvhvzx/RESULT.json, remote run-GzZyxH, archive293586743b2f690f503b9dfb7ec58477b032bea2dacf48c3b6e0bf940f0f5f94. New context-free code review pass1 **9/10**, zero blockers. Common old C06 oracle now verifies SAME actual snapshot/revision and noCommit/Delete; all downstream adverse/once-only tests pass unchanged. Final focused6/6 h-focused-osa7e9ru includes12 positive+6 negative variants and capability41→40→41. Earlier manifest/setup/oracle failures retained separately. Native changed-byte proof NOT_TESTED; both F native failures remain history. Private build and both60+6+150/speeds next. TD121 OPEN; delivery/source ownership/laptop runtime unchanged.

2026-10-13T04:35:32.571746+00:00 — H five changed input contracts **49/49+FMT PASS**, local run-cpu1h6i_/RESULT.json, remote run-kVVAhb, archivea0a7d87624dc6c14f70065f69c8868b8912f8ff4f443ff99edb23ad6f4adba1e. IME756 source/review9 unchanged; private build from immutable source run-GzZyxH is running. Both changed-byte native scopes remain NOT_TESTED; TD121 OPEN. No delivery route, production installation or source authority change.

2026-10-13T04:42:11.401243+00:00 — H private build **PASS182.147s**, immutable run-GzZyxH, BUILD candidate-build-1791866176185256887; IME9458b59db6e570af9282c9eddf0ffaccb206d2ae621c944ee75c0baa252f8d1f, daemonecee1d7ee238a481d017001afe917664a6384730185f38a5f84b3c0e5d42ec3a. Verified backup staged-before-owned-preedit-1791866230042334578; private staging only. Chrome actual zero-key attachment/current PID+tick+exeSHA+ownedCG PASS_SETUP_ONLY, receipt native9458-chrome-zero-key-gate.json. Full60+6+150/speeds RUNNING, Firefox NOT_TESTED. Native behavior not yet accepted; TD121 OPEN. Source756/49+FMT/review9 remain separately scoped; no laptop install, delivery-route or source-authority change.

2026-10-13T04:59:12.079062+00:00 — H Chrome complete native **FAIL**: standard38PASS20FAIL2BLOCKED/60 (normal/fullscreen19P1B each; private20FAIL), actual panel6/6PASS, fixed functional96P42F12B/150. Receipt job-results/native9458b-chrome-proof.json; RAW native9458b-chrome-raw-1791867238165741500/RAW.json under RUN. Compared with F:18 standard+3 panel gainedPASS, previousPASS lost0, all150 functional classifications unchanged; comparison native9458b-chrome-versus41443.json. Standard speed266/420, p50/p95/max93.814/112.434/122.196ms; panel48/48,93.233/122.232/126.707ms; functionalDS129/210,90.626/131.230/145.693ms, Space38/50,18.859/24.783/27.954ms, Tab0/10, selection10/10. Clock evdev-write-return→firstexactAT-SPI; kernel-release NOT_MEASURED/trace UNKNOWN. ChromeGUI40532 closed and exactCG967433 empty before archive/new phase. Firefox SAME private bytes full60+6+150/speed now RUNNING in fresh verified stock zero-key attachment/owned scope981940, exec27826. TD121 OPEN; Sensitive policy/private, Search seed and functional prefix-decoder/other failures remain. No delivery/source-authority/laptop change.

2026-10-13T05:13:27.020584+00:00 — H Firefox native **FAIL**:42P6F1B11NOT_TESTED/60, actual panel6/6PASS, functional86P45F19B/150. Proof job-results/native9458b-firefox-proof.json; RAW native9458b-firefox-raw-1791868151324673810/RAW.json. ComparedF:17 prior standardPASS unavailable (6actualFAIL,1BLOCKED,10NOT_TESTED); priorFAIL privateSpace10 nowNOT_TESTED; one moved-boundary field2 functionalPASS lost. Source causal effect unproven; do not promote H. New normalEmailSpace fifthburst miss and fullscreeniframe10 initial misses; privatefixture field9 hidden caused safe stop/11NOT_TESTED, retained separately from product failures. Speed standard312/420,p50/p95/max109.043/120.571/131.112ms; panel48/48,108.852/118.103/123.030ms; functionalDS101/210,111.469/143.584/151.423ms,Space36/50,18.178/24.371/25.630ms,Tab0/10,selection1/10. Kernel-release NOT_MEASURED/trace UNKNOWN. GUI27826 closed/exactCG981940 empty/all ownedPIDticks gone before archive. First inspected failed Email trace has actual snapshot6chars/cursor0, revisiongap1/identity_currentfalse then ManualToggle context_authority not_handled; wire order/raw identity must be established before changes. Bounded existing receipt-wire diagnostic next, no runtime edit. Both H browserverdictsFAIL, TD121 OPEN; source756/49/FMT/review9 does not certify clients. Routes/sourceauthority/laptop unchanged.


2026-10-13T05:49:50Z — H Firefox passive diagnostic: the original normal20
sequence completed20/20PASS; the earlier Email fifth-burst failure was
**NOT_REPRODUCED**, and the original full Firefox/Chrome FAIL receipts remain
unchanged. Two preceding isolated cells (normal Space Email5, private active
Phone4) also passed; these diagnostics do not establish acceptance or a causal
H improvement. No runtime source, delivery route or ownership changed.

Diagnostic receipt: `/home/ubu/.cache/lay/development/remote-firefox-setup-j2he3qf6/H-full-normal-wire-diagnostic.json`;
remote `job-results/H-full-normal-wire-diagnostic.json`, frozen
`full-normal-wire-frozen-1791870590364254491/RAW.json` under the recorded RUN.
Wire `firefox-receipt-wire-g23efdal/WIRE.json`, SHA
`d25ae4856c19a6a36c842a6ae0c356e42c2f7e1b6f8e2cbf9aa21bfe03e6d375`;
actual actor `firefox-receipt-actor-a4ooluu3/RESULT.json`, SHA
`27d25ae3917db105262b081249ccb73e68857500b0514668ba16a20ff335c332`.
Remote source-only probe proves only the receipt prefix differs from the pinned
original actor; all20 actions/guards/cases/order preserved. Helper reviews8/10
then9/10, two passes; final cleanup finding was applied after pass2 and no third
review score is claimed. Monitor was alive after the actor; wire completeness
UNKNOWN and observer overhead NOT_MEASURED remain.

Speed140/140 measured,0missing: p50=109.699ms,p95=118.915ms,max=123.963ms,
evdev write-return→first exact AT-SPI target; kernel release NOT_MEASURED.
GUI exec51199 completed0; owned scope1011639 empty/all bound PIDticks gone
before freeze. Seat-t is idle finite1h, expires about06:43UTC; no GUI lease
remains. All executable verification was remote; laptop runtime untouched.

Comparison of the failed Email trace and successful diagnostic identifies a
shared transition: all native replacement press/releases, including closing
Space, have been observed before the failed raw6/cursor0 observation; the
existing Reset receipt is then discarded. Later raw6/6 and raw7/7 observations
arrive, but ManualToggle rejects context authority. In the successful case,
prior observed surface retains inert lineage and final7/7 confirms it. Failed
raw surface bytes and whether they satisfy exact observed-prefix provenance
are still UNKNOWN because that original trace records only counts/initial-char
metadata. Do not widen off-caret retention or bypass UnknownStart based on
this evidence. Existing one-character inert guard and negative multi-character
caret tests remain intact. Functional decoder/prefix failures remain a separate
reproducible class to trace through the current owners. TD121 OPEN; H NOT_PROMOTED.


2026-10-13T06:07:34Z — H Chrome original30 diagnostic **FAIL_REPRODUCED**,
16PASS10FAIL4BLOCKED; unmodified fixed15 plans in each of fields1,2. Exact
receipt `job-results/H-chrome-decoder-diagnostic.json`, local copy under the
recorded bdir, frozen `decoder-diagnostic-frozen-1791871454180975840/RAW.json`.
Immediately captured prefix traces retain the first failed layout transition:
actual live legacy composition with KnownStart takes IME_ACTIVE_COMPOSITION
CommitText; native layout sync reports ok:false; ManualToggle reports
Handled(targetUS). In field1 the actual next KEY_Z has keyval122 but IME decodes
я. CreateEngine ingress during that request has no_live_acquisition then
transfer_pending/passive. Field2 also has failed layout sync, with no exact
text target. Precise dispatch failure classification remains UNKNOWN; do not
weaken native acquisition authority or force a decoder promotion.

GUI exec56431 stopped/completed0, exact owned1021075 scope empty/all bound
PIDticks gone before freeze. Collector sent0keys, retained raw traces/hash;
async trace completeness UNKNOWN. Speed DS25/42 measured,17missing,p50=93.010,
p95=127.166,max=127.496ms; Space6/10,4missing,18.808/22.403/22.403ms;
Tab0/2;selection2/2,20.408/21.870/21.870ms. Kernel release NOT_MEASURED.
No runtime source/transport/laptop change; earlier both full H FAIL retained.

Candidate systemic cause: `context_owns_active_preedit_manual_toggle` admits
only UnknownStart, so the stronger KnownStart lineage of the same live owned
legacy word takes the old commit/native-activation path. New explicit plan
`docs/architecture/decisions/2026-10-13-known-owned-preedit-manual-mode.json`
ranks reuse of the existing owned local word/InputMode operation9/10, wider
native-acquisition migration7/10, truthful partial-failure reporting6/10.
Recommended minimal correction requires causal source RED first. Exact-refresh
marker exclusion and every source/word/selection/caret/lifecycle guard remain;
no scope claim for the separate intermittent Email receipt failure or Firefox
marker clients. No new production edit yet; TD121 OPEN.

2026-10-13T06:24:49Z — I plan review pass1 **8/10**. Added the route-specific
consequence preflight to the explicit I decision: existing authority and
transport, intentional KnownStart route selection, preceding Space background
supersession, bounded allocation/latency, unchanged learning, compatibility,
partial output and live-owner rollback. Added required transaction controls
for preedit/handoff/InputMode failures and owner loss; retain UnknownStart and
disabled-auto-switch controls. The causal fixture establishes every retained
guard separately before its real ManualToggle bridge call. Remote causal RED
is still required; production I NOT_STARTED, native I NOT_TESTED. Both H full
FAIL receipts remain immutable and TD121 OPEN.

2026-10-13T06:28:37Z — I causal source **RED confirmed** on the unchanged H
production: real native word/Space and surrounding callbacks establish
KnownStart, published owned preedit and every retained guard; the real bridge
then emits CommitText. Receipt I-causal-source-RED.json, remote
i-known-preedit-1vkvb153/RESULT.json. Earlier incorrect manifest module path
ran0tests, retained NOT_EXECUTED, not causal evidence. Plan final pass2 **9/10**,
no material plan blockers. Removed only the completeness-specific exclusion
from the existing owned-preedit predicate; all actual ownership/marker guards
retain authority. Focused GREEN and consequence controls pending; native I
NOT_TESTED, no private build/stage/install, TD121 OPEN.

2026-10-13T06:43:06Z — I original causal fixture GREEN, remote
i-known-preedit-0y164n7h/RESULT.json / local I-causal-source-GREEN.json; the
wrong adjacent append-predicate edit was caught by the unchanged causal
failure, reverted before any build/stage, and the append guard stays unchanged.
Expanded five-fixture consequence run **3PASS2FAIL**:
i-known-preedit-wpgzoyle/RESULT.json / I-consequence-source-RED.json. Eight real
bridge mode signals/decoder/Backspace pairs,16 retained refusal guards and
disabled-auto-switch control passed. Controlled older Space dispatch still
invoked its switch closure, and actual adapter revocation during preedit
publication falsely completed. Same owned executor now invalidates existing
background work, supersedes the existing latest-layout generation and
revalidates its full grant after both output awaits. Partial output retains
typed indeterminate failure; no fallback or output rollback through a revoked
owner. Added InputMode-owner-loss control. Focused/full GREEN/review/native
pending; controlled closure is source evidence only. No new transport, source
switch owner, timer or installed runtime; both H native FAIL retained.

2026-10-13T06:45:35Z — I focused source **5/5 PASS**, remoteFMT PASS,
I-focused-source-GREEN.json / remote i-known-preedit-fsi9obv1/RESULT.json.
Eight real bridge gestures preserve prefix/receipt/revision/owner/profile and
actual preedit/InputMode, next key and Backspace;16 refusal guards and five
failure stages cover partial error/live rollback/no fallback. Controlled old
Space dispatch now superseded with0 switch calls; disabled auto-switch and
UnknownStart source controls retained. Manifest3126 rows uses actual module
paths and recomputed lane/isolation counts; existing rows retained. Full IME
source gate and fresh context-free code review next. Native I NOT_TESTED,
no build/stage/install, TD121 OPEN; both H full FAIL retained.

2026-10-13T06:58:20Z — I full IME **761/761+FMT PASS**, run-hgdbqqie /
remote run-SeD1J8, archive8e61fe9c7a3258b343d6b6b71eaaea384f0fba0d852beb201661f20631cc9dce.
Six architecture contracts **50PASS1FAIL/51**, run-5yvl0zmx / remoteRRnP1y:
the new negative fixture directly called a typed authority mint outside its
named runtime producers. Canon and guard unchanged; fixture now invokes the
normal manual runtime producer and checks that all16 exclusions prohibit the
owned preedit/InputMode operation while retaining other existing route choices.
Final focused5/5+FMT PASS, I-final-focused-source-GREEN.json / remote
i-known-preedit-se91cgum/RESULT.json. No production change for this fixture fix.
Final combined IME/contracts gate required on the corrected fixture snapshot.

Fresh context-free code review pass1 **9/10, no material I blockers**. Its
nonblocking inherited limit is recorded in the I decision: failed rollback
transport and owner loss during rollback are NOT_TESTED; the unchanged H
best-effort rollback block is not claimed as covered by forward-output loss
controls. Review pass2 limited to the normal-producer fixture correction and
bounded coverage statement. Native I NOT_TESTED; no build/stage/install.

2026-10-13T08:40:18Z — I final combined source gate **812/812+FMT PASS**:
761 IME tests and51 tests from six affected architecture contracts, local
run-_imkjucb/RESULT.json / remote run-WPGb6I. Immutable source archive
6b2b96f7492f9dbca282e730c329063dbaad9d6aac5271e6558834a5d2132a56.
The negative fixture uses the normal runtime producer; canon and mint guard
unchanged. Fresh context-free plan/code reviews each final **9/10**, two passes
each complete, no material I blockers. Failed rollback transport and owner
revocation during rollback remain NOT_TESTED in the inherited executor;
forward-output loss and successful live rollback do not certify those cases.
I private build/stage and both unchanged native60+6+150 scopes remain pending.
GUI OFF; old finite seat-t is inactive/MainPID0. Private H9458b/ecee1 unchanged,
both full H FAIL retained, TD121 OPEN. Next build only from this immutable
GREEN gate, then private stage and new finite owned native seat. No laptop
installation, no delivery-route change, no promotion/commit/push yet.

2026-10-13T08:47:53Z — I private release build **PASS182.143s**, immutable
run-WPGb6I GREEN source; I-private-build.json pins IME
e84d497cb9416772a3e0abb8a48f4b80da3bbc27c55032f1fdf8c3ce216384cf and daemon
52a92c557b7833721570158f06b7ea9ee7b72d60dad5affa109d6221e110d767.
Verified H private binaries backed up under
staged-before-known-owned-preedit-1791881128440822378 before private staging.
Fresh finite seat-u1058231/t7865922/session2328/VT8, owned GUIscope1058419,
controller1058483/t7868366; exec14025 LIVE. Setup002/003/005 complete0,
zero-key Chrome attachment PASS_SETUP_ATTACHMENT_ONLY, actual current
GNOME/IBus/Chrome/IME/daemon PIDticks, bytes and shared owned scope verified.
Unchanged original Chrome full9 wrappers queued:60 standard+6 panel+150
functional+speed. Native Chrome RUNNING, Firefox NOT_TESTED, no acceptance.
Heavy source/build/AST checks OFF while GUI lease live. Both H FAIL retained;
TD121 OPEN, no laptop installation, delivery-route change, commit or push.

2026-10-13T09:03:27Z — I full unchanged Chrome proof **FAIL**, retained at
nativee84d4-chrome-proof.json, RAW nativee84d4-chrome-raw-1791882005188000210.
Standard38PASS20FAIL2BLOCKED/60; panel6PASS/6; functional105PASS33FAIL12BLOCKED/150.
Against H the exact same216 cells have **9 prefix FAIL->PASS**, fields1,3–10;
**0 previous PASS lost, no other status changes**. Search2 remains FAIL; private
Sensitive policy, cursor/focus/Space-undo and completion blockers unchanged.
This is measured native support for I's shared owned-word mechanism, not full
promotion. Speed std266/420 p50=93.615,p95=111.847,max=125.453ms;
panel48/48 91.462/108.779/109.183; functionalDS138/210
90.594/125.577/132.900; Space38/50 19.154/22.784/24.868;Tab0/10;
selection10/10 19.661/21.676/21.676. Clock is evdev write-return to exact
AT-SPI surface; kernel release NOT_MEASURED. Missing samples retained.

Chrome exec14025 CLOSED0, exact scope1058419 empty/all bound PIDticks gone
before immutable assembly; expected STOP signal traceback does not imply PASS.
Firefox fresh exec15638 LIVE/scope1072872, controller1072892/t7961774,
GNOME1073145/t7964494, Firefox1073496/t7964592, IBus1073223/t7964555,
IME1073399/t7964582, daemon1074228/t7965097. Same I private bytes and finite
seat-u2328 verified; fresh setup002003005 complete0 and zero-key attachmentPASS.
Unchanged full Firefox9 wrappers queued051–059:60+6+150+speed RUNNING.
No heavy source/build/AST while GUI lease live. TD121 OPEN; no laptop install,
delivery-route change, promotion/commit/push. H failures remain immutable.

2026-10-13T09:20:56Z — I full unchanged Firefox proof **FAIL**, immutable
nativee84d4-firefox-proof.json / RAW nativee84d4-firefox-raw-1791883154511052960.
Standard55PASS3FAIL1BLOCKED1NOT_TESTED/60; panel6PASS/6;
functional87PASS44FAIL19BLOCKED/150. Against H16 standard PASS gained,
**3 previous PASS lost**: normal active6, space2, space8. Functional moved_boundary2
FAIL->PASS, no functional PASS lost. Firefox causality remains UNKNOWN;
these lost PASS are retained blockers, not dismissed by aggregate gains.
Chrome nine causal prefix gains/no lost PASS remain separately recorded.
Both full I browser scopes FAIL; **I NOT_PROMOTED**, TD121 OPEN.

Speed std395/420 p50=109.536,p95=120.605,max=135.594ms; panel48/48
108.237/120.337/122.413; functionalDS100/210 111.792/149.464/154.676;
Space37/50 17.564/23.987/24.591;Tab0/10;selection1/10 17.075ms.
Clock evdev write-return to exact AT-SPI; kernel release NOT_MEASURED.
Firefox STOP exec15638 CLOSED75: guard reported descendant_cleanup; one
private lay-l1.1-serve1073433/t7964589 remained. Its exact scope/comm/UID/tick
were validated and pidfd terminated; scope1072872 then empty/all bound PIDticks
gone before immutable freeze. No new heavy launch until cleanup completed.

Three normal-window failures share live bridge but unavailable manual/refresh
permission. In active6/space2 native transfer installed UnknownStart and fresh6/6 or
7/7 SSI still left manual permission false. Status3/false means delegation,
not a reported target layout; the earlier repeated-US inference is withdrawn. Space8 also has fresh7/7 SSI before
denial. Per-case512000-byte tail receipts have one malformed initial line;
ordered suffix facts are bounded, exact payload/complete wire UNKNOWN.
Investigate current existing owner/word/receipt transfer and prove source RED
before new production changes; no blind grant rebind or weakened guards.
No installed laptop/runtime delivery change, commit or push.

2026-10-13T09:39:17Z — Next bounded causal plan recorded in
2026-10-13-confirmed-native-word-second-handoff.json. Two native normal cases
fail v3 field lease validation AFTER controlled source handoff, BEFORE deletion:
active6/burst2 at1791882238, space2/burst2 at1791882271; actual field receipt
empty against the original context-admission/InputContext_3 identity. Subsequent
old handoff cancellation/next manual authority fails. Real clientcaps1073741865
include the exact-refresh marker; existing cycle09 transfer defaults41.
Test a second same-field transfer after fully confirmed native replay using
actual marker capabilities. Source snapshot/retirement publication loss is a
hypothesis, not a proved token-order defect. Preserve normal space8 separately;
status3/false is delegation only. Options9(existing proof producer repair),
7(existing readiness transaction),1(rejected field/marker bypass).
No new production edit; source causal RED and independent plan review required.

2026-10-13T10:26:31Z — J focused source investigation completed; no production edit.
Remote j-second-handoff-oywvbd2n/RESULT.json PASS, one process-isolated
fixture, 32 subcases, 13.711s, remote formatting completed. Capabilities41 and
41+exact-refresh marker, empty/Unicode prefixes, Space/NBSP, and four receiver
orders: absent SSI; matching SSI after install; matching SSI after FocusIn and
before installation; SSI before FocusIn. The first three orders preserve the
whole original field receipt/tail/epoch (24 positives). Pre-FocusIn SSI remains
passive with empty authority (8 adverse controls). No proof was minted by
assigning KnownStart/token/confirmation. Fixture hashb0e03a57824b1421de9797763aaa490da4c0c991ea6ae1acd4a04c38b8f8c9ff.
Initial absent-SSI probe _94lk41o PASS did not reproduce native failure.
Experimental qgh13lrr and8cjiwtk7 failures asserted authority for unbound
pre-FocusIn data; those positive expectations were invalid, not causal runtime
RED. Preserve their receipts. Plan review2/2 final8/10 rejects generic FocusIn
snapshot preservation without independent activation binding; 9/10 pass1.
Post-FocusIn variants now pass, so no producer repair is justified by this test.
Next: unchanged I private binaries, existing reviewed passive wire helper and
original normal20 native actor, join real callback ordering to the refused
v3 receipt. No change to guards, snapshot expiry, observed flags, native
transport, runtime ownership or delivery. I native Chrome/Firefox remain FAIL,
I NOT_PROMOTED, TD121 OPEN. Scope is focused source diagnostic only.

2026-10-13T10:40:10Z — I unchanged original normal20 native sequence with existing
reviewed passive wire completed: 20/20 PASS, diagnostic NOT_REPRODUCED.
This does not replace full I Firefox/Chrome FAIL or the three lost Firefox
PASS; observer overhead NOT_MEASURED and wire completeness UNKNOWN.
Speed140/140, p50=110.167ms, p95=119.311ms,
max=130.202ms; evdev write-return -> first exact AT-SPI, kernel
release NOT_MEASURED. Immutable raw:/home/e/projects/lay-development-runner/firefox-legacy-native-20261012.6onP5c/I-normal-wire-frozen-1791887807789967674/RAW.json; wire
shacb942d222324e61e722c874be8c4564e1130d96a7549fcf361e17b28f3e1a627. Actorfirefox-receipt-actor-n7nklx8v; wire
firefox-receipt-wire-5cxwxcyu; job903 exit0/noabort/noheldkeys.
Explicit STOP closed41959/0, scope1102654empty and every bound PIDtick gone
before freeze; I-normal-wire-closed.json. Seat-v2422 finite, no active GUI.
Original failed active6 callback order is bounded source evidence: native
Get accepted -> Enable -> caps1073741865 -> SSI6/6 -> Shift release callback
serial2846 (refused ownership while activation pending) -> marker ingress ->
transfer_install -> v3 empty field -> abort before deletion. These timestamps
are real recorded order; SSI body remains UNKNOWN in the old failed trace.
J32 had direct preinstall SSI and no intervening release. Extend source-only
fixture with real SetSurroundingText/Shift release before marker, and Reset
after confirmed first native replay. No production change or causal claim yet.
Compilation typo KEY_SHIFT_L in private test was corrected to native keysym
0xffe1; sasamvl6/CARGO.jsonl is compile failure, not semantic RED.

2026-10-13T10:46:10Z — K causal source RED and minimal correction plan recorded in
2026-10-13-pending-native-shift-word-lineage.json. Sourcev7zrblwv compiles and
selects1 real bridge fixture; receiver3 actual SSI+Shift release before marker
installs exact SSI but poisons source-seal lineage toUnknownStart0, resulting
v3 fieldempty before any delete. This matches recorded pending Shift release
order in original normalactive6; original failed SSI body remainsUNKNOWN.
Owner branch already treats Shift asLegacyShiftObservation; pending target
branch currently calls target_input_before_admission unconditionally. Proposed
small correction reuses that classifier before owner split, keeping neutral
Shift passive without early authority; possible mutation and malformed bodies
remain destructive/refused. No new state/transport/timer, FocusIn preservation
or guard relaxation. Options9(existing classifier),6(new reducer event),
2(delay/retry rejected). Root/source localization and independent review next.
I bothnativeFAIL/3lostFirefoxPASS remain, TD121OPEN, no promotion.

2026-10-13T10:59:13Z — K classification-only source80subcases PASS6eqoom1n, then
review identified and actual fixture proved second ordering REDlp8t6epy:
pending Shift ingress -> markerready -> handlerinstallsgrant -> oldPassive
stamp rejected -> newly installed wordrevoked -> v3fieldempty. Earlier root
REDjkz0xczu localized source-seal generation6KnownStart ->7UnknownStart at
Shift before marker, with actual SSI-only control passing. Neither RED is a
compile/helper failure. Planreview1/2 score8 requires delayed callback coverage.
Minimal typed no-authority PassiveShift disposition/observation is planned;
never convert Passive to authorizedKey, preserve exact lexical tail and output
postcondition, keep atomic and owned/abandoned same-path Shift unchanged. C03/C04
explicitly recorded. No second production callback correction yet.
Platform rejected a fresh subagent spawn (thread limit); reused an independent
read-only reviewer, not claimed fresh/context-free. I bothfullnativeFAIL remain.

2026-10-13T11:12:40Z — K plan review2/2 final9/10, no remaining plan blocker; reused
independent reviewer because platform thread limit, not fresh/context-free.
TypedPassiveShift callback correction source96subcases GREENy6x_wrfi; final
left/right press/release both before/after-marker/no-learning version
GREEN1zwmta6m (16.495s). No earlyowner/token/output, actual SSI/field/tail/epoch
proof retained only after validatedTransfer; atomic/ownedabandon paths unchanged.
Added6-case pending-factory adverse fixture; focused2testresult PENDING.
Compilekkwybjgq was test &String->ObjectPath typeerror, fixedas_str and exact
u32wiretuple types; notsemanticRED. Full IME/contracts/FMT/code review/native K
remainpending; privateI bytes and both fullnativeFAIL retained. No promotion.

2026-10-13T11:17:10Z — K final focused source selected2 PASS1z52y6xe (17.090s):
96 positive/control subcases with actual four left/right Shift edges both
callback phases, plus6 pending-factory adverse cases (printablepress/release,
caret,command,atomicShift,malformed). Possible mutation still loses word proof;
malformed body still revokes/refuses; no revived v3field/text authority. Existing
same-path held-Get EarlyKey tests unchanged. The adverse helper initially lacked
RU in its bootstrap allowlist (37m0jfdd setupDenied); configured both profiles
before admission as the factory case requires, not a runtime guard change.
Manifest3128 rows including2new process-isolatedcases; full763IME+51contracts
=814+FMT pending. K code review pass1/max2 pending, native K NOT_TESTED. I/H
fullnativeFAIL retained; no runtime install/promotion/commit/push.

2026-10-13T11:26:24Z — K full remote source gate GREEN814/814 +FMT, run-og427fqt
(local)/run-vWGi5z(remote), archive115d01b34552fc224ca7f46c9ee25f31f49c67a2c8977f59110ba26b129107d2. 763IME and51six
contracts, zero failure; development scope, not native/release acceptance.
Plan2/2 final9; codepass1score9/no productionblockers, finalpass2pending.
Adverse6 proof limit retained: ingress + installedreadout, not adverseactual
handler/ManualToggle settlement. Exact immutable GREEN source privatebuild
started; no bytes staged or nativeKclaim yet. Both I/H fullnativeFAIL retained.

2026-10-13T11:28:39Z — K code review2/2 final9/10, no materialblockers; verified
local primary RED/focused GREEN and unchanged productionhashes. Reviewer final
Full814pending line predates root final gate result; actualFull814+FMT PASS
run-og427fqt/run-vWGi5z is recorded above. Native K remainsNOT_TESTED; private
build80415 running from immutable GREEN archive115d01b3..., not mutable source.

2026-10-13T11:35:07Z — K privateBUILD PASS182.138s, exact immutableGREENrun-vWGi5z
archive115d01b3...; IMEf577c86d5185cd5f9333c652344150b50be711bef3f773e0948241dc24db2579,
daemon5bead464c0e537c571109a175344c96bd4fc2e0aa6e2ebad76903f94d13e53d9.
Verified I backupstaged-before-pending-shift-1791891140219834426, private
RUN/bin/candidate metadata staged; no laptop installation/promotion.
Fresh finite seat-w1143151/t8867062/session2517/VT8 verified and activated.
Chrome native launch returned75 BLOCKED_RESOURCE(lock_busy) before GUI/input.
Actual leaseholder is unrelated L3proof guard1143027/t8854383/session2514;
its job is not killed/throttled or bypassed. Native K NOT_TESTED, wait resource.
Source814+FMT/reviews2/2each9 complete; both earlier full I/H FAIL retained.
Build warning limit: inherited default library deadcode plus1IME warning for
old key-callback wrappers now test-used/production-unused; no runtime edit
merely to remove warnings during this bounded native proof.

2026-10-13T12:00:00Z — K unchanged full Chrome native scope remains FAIL:
standard38PASS20FAIL2BLOCKED/60; panel6PASS/6;
functional105PASS33FAIL12BLOCKED/150. All216 keyed statuses equal I;
zero gained/lost PASS. Receipt: job-results/nativef577c-chrome-proof.json;
raw: nativef577c-chrome-raw-1791892753154035871/RAW.json, in the existing
remote6onP5c runner. STOP79243 CLOSED0; scope1146526 empty/all bound PIDticks
gone before immutable assembly (K-chrome-closed.json). No runtime authority or
delivery-route change. Private20FAIL and two search-after-Space preparation
BLOCKED remain open; K is NOT_PROMOTED, Firefox NOT_TESTED at this milestone.
Speed standard266/420 p95=113.694ms, panel48/48 p95=110.965ms,
functionalDS138/210 p95=127.844ms, Space38/50 p95=22.842ms,
Tab0/10, selection10/10 p95=23.828ms. Full p50/max/missing denominators in proof;
clock evdev write-return to exact AT-SPI, kernel release NOT_MEASURED.

2026-10-13T12:17:16Z — K full Firefox native scope FAIL:
standard57PASS3FAIL/60; panel6PASS/6; functional87PASS44FAIL19BLOCKED/150.
Against I four standard PASS gained and two lost (normalSpace7/privateSpace1);
normalactive6 still fails. All panel/functional keyed statuses unchanged.
Proof job-results/nativef577c-firefox-proof.json; immutable raw
nativef577c-firefox-raw-1791893836955781505/RAW.json in remote6onP5c runner.
STOP94565 CLOSED0; scope1162458 empty/all bound PIDticks gone before freeze;
K-firefox-closed.json. Speed standard408/420 p95=119.426ms;
panel48/48 p95=120.200ms; functionalDS102/210 p95=141.484ms;
Space37/50 p95=24.072ms; Tab0/10, selection0/10. Full p50/max/missing counts
are in proof; evdev write-return -> exact AT-SPI, kernel release NOT_MEASURED.
Both K browser scopes remain FAIL, no promotion or runtime authority change.
Source-only follow-up isolates completed-native-boundary receipt retention
for a one-character caret lag. Exact old SSI body UNKNOWN; other native
failures remain separate. No production edit or guard relaxation is justified
until a real source RED and contradictory controls are recorded.

2026-10-13T12:35:34Z — L source RED recorded: l-completed-boundary-caret-w76feg3j/
RESULT.json, selected3, one failed positive and two passing negative controls,
13.403s. Existing prior-replay recognizer accepts one-character caret lag
after real completed replay; completed-boundary receipt producer discards it.
Plan: reuse current paired boundary provenance solely to retain unconfirmed
receipt, with the full final actual field receipt still owning all authority.
Decision: 2026-10-13-completed-native-boundary-caret-receipt.json; options9/6/2.
K production hashes unchanged; current test fixture RED. Other native failure
mechanisms and both browser FAIL receipts remain open; no promotion.

2026-10-13T12:51:05Z — L focused source GREEN5/5, 19.096s, receipt
l-completed-boundary-caret-green-9_juypt7/RESULT.json. Pass1 review6 blocked
later-branch widening; recommended9 variant implemented solely in the earlier
completed-boundary target-prefix classifier. Exact target/external prefixes,
monotonic text ordering and paired current provenance retained; full target
with lagged caret remains inert. Final exact authority checks and generic Reset
branch unchanged. Positive16 + pre-final manual6 + contradiction20 controls,
original probe/backspace and second-handoff96 fixture pass. Full814+FMT and
second/final review pending; L native NOT_TESTED, both K native scopes FAIL.
No runtime authority or delivery-route change or promotion.

2026-10-13T13:01:49Z — L full source814/814 +FMT GREEN:
run-qc62j1kj/run-Ij7oGU, archive2bdb006eee9392b10be1514a11ca63c6dbb4390c1da5eb2ca596cde8a64283a7.
L independent reused review2/2 final9/10, no blockers. Full source result
completed after reviewer's pending statement; source validation is complete
for this immutable snapshot. Private build started; L native NOT_TESTED.
K Firefox/Chrome fullFAIL and lost PASS retained, no promotion. Current Chrome
private first-field refusal is separately bound to purpose0/hints2112 with
PRIVATE; L leaves its existing sensitive-content policy unchanged.

2026-10-13T13:26:38Z — L Firefox full native FAIL:
58PASS2FAIL/60; panel6PASS/6; functional87PASS44FAIL19BLOCKED/150.
natived858c-firefox-proof.json / natived858c-firefox-raw-1791897998595953698/RAW.json.
Against K three standard PASS gained/two lost (privateSpace7/fullscreenactive10);
all panel/functional statuses unchanged. Normal20/20 does not close full scope.
STOP96641 CLOSED0; scope1211266 empty/all bound PIDticks gone before freeze,
L-firefox-closed.json. Speed411/420 p95=118.258ms;
panel48/48 p95=121.505ms; functionalDS102/210 p95=140.736ms;
Space37/50 p95=24.894ms; Tab0/10 and selection0/10. Full p50/max/missing
counts in proof; same evdev write-return clock, kernel release NOT_MEASURED.
Source/review completed, L private bytes d858c220.../6e725f33...,
Chrome scope pending, no promotion or runtime authority/delivery change.

2026-10-13T13:44:32Z — L Chrome full native FAIL:38PASS20FAIL2BLOCKED/60,
panel6PASS/6, functional105PASS33FAIL12BLOCKED/150. All216 statuses equal K;
zero gains/lost PASS. natived858c-chrome-proof.json /
natived858c-chrome-raw-1791899073168725688/RAW.json. STOP45819 CLOSED0,
scope1255277 empty/all bound PIDticks gone before freeze, L-chrome-closed.json.
Speed266/420 p95=112.498ms; panel48/48 p95=110.825ms;
functionalDS138/210 p95=125.027ms; Space38/50 p95=22.711ms;
Tab0/10, selection10/10 p95=22.965ms. Full p50/max/missing counts in proof,
same evdev write-return clock, kernel release NOT_MEASURED.
Both L browser scopes FAIL; no promotion or changed runtime authority.
Existing focused passive-wire diagnostic for two remaining Firefox cells is
next; it does not replace these full receipts or change delivery/actor guards.

2026-10-13T14:00:58Z — Existing focused wire helper, unchanged L bytes:
privateSpace7/fullscreenactive10 both PASS, diagnostic NOT_REPRODUCED14/14.
Primary full L Firefox/Chrome FAIL and lost PASS retained. Wire WIRE.json
ovia66bk/8vgxh3wk; immutable
L-focused-wire-frozen-1791899632565641505/RAW.json. STOP33546 CLOSED0,
scope1315013 empty/all pinned processes gone before freeze; original VT1
restored. Observer overhead NOT_MEASURED, wire completeness UNKNOWN.
Original fullscreenactive10 shows real native erase6/replay6, preedit0, then
SSI1/cursor1 -> SSI4/cursor1 rejection; exact old text body UNKNOWN.
Next source-only discriminator: advancing replay target text with unchanged
caret. No production edit or broader cursor tolerance without a causal RED
and preserved contradictory/refusal controls.

2026-10-13T14:07:39Z — M actual growing target with stationary caret: source RED, production unchanged.

- Primary remote receipt: `/home/e/projects/lay-development-runner/firefox-legacy-native-20261012.6onP5c/m-replay-stationary-caret-uzrq9eyo/RESULT.json` (SHA-256 `ce11f19851447a882273653779c83e5d2df2cac45c338e49059db42508a8adcc`); three selected semantic tests, one positive FAIL and two existing controls PASS, 14.240936661s. The actual one-character/caret1 to three-character/caret1 observation loses the inert pending receipt at revision3 while KnownStart remains present. Three L production hashes were pinned unchanged.
- Original L fullscreenactive10 trace measured SSI1/cursor1 then SSI4/cursor1 after real complete native replay and Reset. The four-character payload itself is UNKNOWN. The generalized source fixture proves the observation mechanism, not the historical browser payload. Original L native FAIL and both lost-PASS cells remain blockers; focused NOT_REPRODUCED does not supersede them.
- Decision: `docs/architecture/decisions/2026-10-13-native-target-text-stationary-caret.json`. Preferred variant9/10: one shared inert progress predicate for two actual adjacent target observations, strict text extension and nonregressing caret with existing completed replay/owner/identity proof. Generic caret-lag widening5/10 and new delay/retry2/10 rejected. Preserve existing final full exact/end-caret and all contradiction guards.
- M plan review pending, 0/2 passes; no M production change or native test yet. L full814+FMT is historical for its exact prior fixture. GUI OFF; installed private L bytes unchanged; TD121 OPEN, NOT_PROMOTED.

2026-10-13T14:25:37Z — M plan review1/2 PASS9/10, production implementation ready for remote focused tests.

The independent reused reviewer found no material plan blocker and required prefix-shape/progress-permission separation. Implemented one existing-owner shared completed-native progress predicate, used by boundary and Reset inert-retention paths; no generic prior-snapshot widening or final-authority change. Positive fixture now includes successive growing snapshots with stationary caret and full-target lag; original controls retained and extended after wider-lag progress. Focused/full source and M native remain PENDING/NOT_TESTED; private installed L bytes remain unchanged.

The user selected Double Shift in Chrome private ordinary fields without learning or history. This is the next distinct policy correction, not yet implemented; password/PIN/hidden-field protection remains required.

2026-10-13T14:28:38Z — M focused source GREEN5/5 (20.619596118s); final review2/2 pending.

Receipt `/home/e/projects/lay-development-runner/firefox-legacy-native-20261012.6onP5c/m-replay-stationary-caret-green-n78cbnej/RESULT.json` SHA-256 `45206e24635f30aa895a3a8d73623bf772b2b4629f4fc9e201e039bb396f9109`; all61 source rows match the actual checkout after fetching remote formatter output. Positive32 variants (old16 retained plus actual successive stationary-caret growth/full-target lag16), pre-final manual refusals10, contradictions44 including unchanged wide-lag body, growing-text backwards caret and caret before the target, original second-handoff96 and probe/backspace control pass. Only source behavior is proven. Existing paired replay/current identity, old Reset decreasing-prior and one-character recognizer, final whole exact/end-caret single-spend authority remain intact. Full source gate follows the separately authorized Chrome-private policy correction; M native NOT_TESTED, old L native FAIL retained, installed private L bytes unchanged.

2026-10-13T14:32:30Z — M final independent code review2/2 PASS9/10, no material blockers; both M review passes closed. Focused5/5/all61 source bindings verified; full source and native remain pending. Original one-character recognizer and final authority/owner/atomic/sensitive guards byte-identical to baseline. Root proceeds to the user-selected private no-learning policy; M source receipt remains an immutable snapshot.

2026-10-13T14:44:24Z — N private manual policy source RED4 selected (1FAIL/3PASS), production M unchanged.

Receipt `/home/e/projects/lay-development-runner/firefox-legacy-native-20261012.6onP5c/n-private-manual-policy-red-s1abodtn/RESULT.json` SHA-256 `c5f41f3b2df2e3da6f301f8824c6d382304ffd62884334153b2b46ee1ee043a7`, 64.944438341s. First loss is the PRIVATE/protected-content conflation in content_is_sensitive, before actual native acceptance or persistence proof. User explicitly selected private Double Shift without learning/history. Decision `docs/architecture/decisions/2026-10-13-private-manual-toggle-without-personalization.json`: preferred9/10, separate protected-content/personalization predicates, explicit writer gates and one typed privacy fact in existing exact-tail readout/lease; no new owner or delivery route. Public/legacy privacy handling and diagnostic payloads must be covered. Local-only guard removal4/10 and separate private route/global log disabling2/10 rejected. Plan review0/2 pending; N production unchanged; M review2/2 closed; installed private L unchanged, all old native FAIL retained.

2026-10-13T14:44:24Z — N independent plan review1/2 score8/10, recommended9/10 after two adopted clarifications. Added explicit early IME-owned daemon payload/typed-tail/user-correction sink denial under the existing backend policy; later V4 lease cannot authorize preceding input. Added retention-transition volatile context/shared-handoff/pending cleanup and PRIVATE-to-public boundary proof. Existing generic ReplaceTail PRIVATE refusal preserved. Precognition tick already has IME gate, not counted as an additional leak. No N production changes or persistence/native proof yet; implementation proceeds under this aligned plan, with final pass2 remaining.

2026-10-13T16:34:32Z — N implementation and diagnostic scope correction, full/native still pending.

First N RED helper configured4 but actually ran all766 IME bin tests (765PASS/1policyFAIL), because selected rows were incorrectly supplied as the complete harness registry. Primary receipt stays immutable; its first policy loss is valid, selected denominator superseded by raw log counts. A later wrong-scope probe ran daemon274 (272PASS/2terminal fixtureFAIL) and stopped with ExecutionError; no semantic RESULT or PASS. Corrected15 selection uses the complete discovered per-target registry. Its first precise run `n-private-source-od063_bz/RESULT.json` has3 daemon fixtureFAIL: publishing runtime config did not change the on-disk config subsequently loaded by daemon (and load resets library learning flags). Fixture now writes/restores its isolated config; ordinary native positives and unknown-privacy IME negative are explicit, with process isolation.

Both binaries compile (n-compile-1791908102223797847,12.925530272s) before the final test additions. Current exact15 rerun is active exec16355. Implemented protected/private policy split, explicit learning gates, early IME raw retention refusal, transition volatile/shared cleanup, and existing reducer's settled ContentType projection before imported receiver text. AdditiveV4 returns one bound retention fact through the existing exact lease; old public V3 stays compatible, legacy private refuses, unknown retention never authorizes journals. IME raw keyval/keycode/manual payload and daemon private/unknown errors/journals are gated. Generic ReplaceTail still excludes PRIVATE.

Target cache9985687552/12884901888bytes. Installed private L remains d858c220.../6e725f33..., GUI OFF. N final review2/2, full source gate (must include daemon), immutable build and all native/browser speed proof remain pending; old L full native FAIL/lostPASS retained. TD121 OPEN, NOT_PROMOTED.

2026-10-13T16:39:39Z — N precise15 source GREEN16.397955494s; primary `n-private-source-vqqgajio/RESULT.json` SHA4a64c6c09eac7946db219a41e806f4a35c64a556d0c46c7c0651b99d5381e3d8. Complete registry supplied to each target; all205 Rust source rows bound after remote formatter fetch, manifest3132. Both terminal raw-input positives explicitly use isolated on-disk uinput config; unknown-privacy IME Enter negative has debug/history enabled and zero writer files. Existing M5 retained, private volatile/learning/cleanup and exact lease privacy/field/unknown errors pass. Final N review2, full source including daemon and native/persistence acceptance remain PENDING; L private bytes unchanged.


N final review2 found raw-key trace callers bypassing the existing engine privacy wrapper and daemon autorepeat/cancellation/filter diagnostics retaining key codes under unknown IME privacy. Root routed the four IME callers through `trace_key` and uses the existing daemon backend retention predicate for those diagnostics. No new delivery route or runtime authority. The new actual-header private active/Space ManualToggle→V4 fixture produced causal RED16 (15 PASS /1 FAIL), `n-private-source-zswp4mpj/RESULT.json`: after a private active-word Reset, the first factory receiver had hints0 rather than2112. The first loss is `settle_soft_reset_rereceipt`: its metadata-bearing `SettledWordState` was accepted but ContentType ignored. Root now preserves this already validated fact in the existing reducer for soft Reset and bridge-output settlement; the existing bridge-output producer includes its ContentType. Rerun/full source/native/persistence proof pending. The last full source attempt `run-r7mlsrjd/RESULT.json` stopped before tests because the focused probe generated noncanonical manifest ordering; the probe now uses the existing `discovery.canonical`. Both previous native FAIL scopes and latency missing counts remain retained; installed private L bytes unchanged.


2026-10-13T17:06:01Z — N focused16/16 source GREEN, primary `/home/e/projects/lay-development-runner/firefox-legacy-native-20261012.6onP5c/n-private-source-3g89g7f9/RESULT.json`, local `/home/ubu/.cache/lay/development/remote-firefox-setup-j2he3qf6/N-source-probes/n-private-source-3g89g7f9/RESULT.json`, SHA `bf70a5e4b8785113ae0603104731e29fa3841f18efffbcc802110248147356d7`. All205 runtime source rows match remoteformatted local checkout. The actual-header PRIVATE2112 active/Space fixture proves first ManualToggle, V3 refusal/V4 retention=false, exact same-field factory import with private metadata, existing native replay press/release callbacks, final actual SSI and second ManualToggle. With diagnostics explicitly enabled and a nonempty file, all observed key records redact keyval/keycode/decoded/counts and no manual-plan payload is recorded. M controls and protected-content/unknown daemon-retention controls retained. Final review2/full source gate/private native/browser/persistence proof pending. GUI OFF, installed private L bytes unchanged; no laptop runtime change or delivery-route change; TD121 remains OPEN.


N final independent review2/2 closed9/10, no material production blockers after the concrete trace/metadata fixes. Supplemental source receipt `/home/e/projects/lay-development-runner/firefox-legacy-native-20261012.6onP5c/n-private-source-ijtoej0k/RESULT.json` is GREEN19, SHA `1bcb32958d583b931e4c72834735f00daab16c8538ceb0728186601d7ba0245e`, all205 source rows match remoteformatted local. Prediction negative retains its existing public positive; system/reverted/deferred negative observes zero private sink files while enabled public controls persist real events. The actual private Manual/V4 fixture also changes ContentType to public and proves the old epoch cannot rearm suppression or expose old private text. Production code unchanged since GREEN1090, full gate refreshed for supplemental test-only changes before immutable private build/native. Native journal-family acceptance and prior browser failures remain pending.


2026-10-13T17:23:12Z — Refreshed N full source1092/1092 and FMT PASS: `/home/ubu/.cache/lay/development/run-8laey58b/RESULT.json`, SHA `a6cae95a024b9690e0d0f440f795d66ea13e291e4932125571edbaa6c609f7f7`, immutable source gate `/home/e/projects/lay-development-runner/run-OYRPtS`, archive `8afcb980ed79cdf5c325813bf22351309ee04032f96c4a9c4bc739f9192f94d9`. Both bins and six input contracts, focused19 and independent review2/2 score9 are source evidence only. Candidate build starts from that immutable source, validates every source row before/after, stages only into the private remote runtime after verified backup. Original Firefox/Chrome actors and all prior FAIL/BLOCKED receipts retained; browser/native/private-journal acceptance still pending.


N Firefox native primary `/home/ubu/.cache/lay/development/remote-firefox-setup-j2he3qf6/native20df1-firefox-proof.json`, SHA `aec4711c443050689eb72fe3fbc53f08beccea0c058b5de4e88b33e33320f81b`, immutable RAW `/home/e/projects/lay-development-runner/firefox-legacy-native-20261012.6onP5c/native20df1-firefox-raw-1791913652042738849/RAW.json`. Actual20df1/765f7 private bytes from sourceGREEN1092, independent review9. Standard59PASS1FAIL/60: normal20/20,fullscreen20/20,private19/20; all30 after-Space cells PASS. ComparedL two standard failures restored (fullscreen active10/private Space7), one prior PASS lost (private active10). The failed row first RU→EN is PASS; the return after z+Backspace misses target. Its trace shows earlier owned z release serial15334 callbackstamp refused_timeout, callback_elapsed2527us, then managed_release=true and observer_ingress only afterward. Later Backspace SSI6 cannot recover and manual RPC denies context authority. This first-loss ordering is measured; the exact code repair needs causal source proof, and no guard/timer bypass is authorized by the trace alone. Functional78PASS38FAIL15BLOCKED19NOT_TESTED; last functional9/10 job error `fixture field is hidden` leaves19 untested and9 previous functional PASS lost to coverage/visibility, no gains. Keep those outcomes separate from product regressions. DS speed414/420,median106.554774ms,p95119.139740ms,max125.594719ms; clock evdev write-return→first exact AT-SPI target,kernelreleaseNOT_MEASURED. Native scope1965691 closed/allpinnedPIDticksgone beforefreeze. Chrome whole gate and actual private writer-root audit pending; TD121 remains OPEN.


2026-10-13T18:42:56Z — O causal source RED, `/home/ubu/.cache/lay/development/remote-firefox-setup-j2he3qf6/O-source-probes/o-late-managed-release-red-ad5ufwag/RESULT.json` SHA `4a2799565ae39c40132d9bb836d3469c725701ac186d9d761f6738a3f84e5869`: one owned managed-release word-proof failure, two controls PASS. Valid press/commit accepted; authenticated release is handled with unchanged text before observer ingress, but generic admission/settlement erases known word proof. The old producer treats late ordinary release as a possible mutable key. Preferred bounded option9 is existing text-inert release cleanup plus canonical passive classification, with native header/path validation and no new word authority/owner/timer/queue/transport; timeout increase5 and generic KnownStart/settlement queue3 rejected. Production unchanged; plan review and stronger negative controls required before implementation. N Chrome private20/20 and writer-root audit PASS for visited roots; full N native stillFAIL, prior outcomes retained.


N Chrome native primary `/home/ubu/.cache/lay/development/remote-firefox-setup-j2he3qf6/native20df1-chrome-proof.json`, SHA `9d7b0b2290e9afd761411456694b4139a8c5b6f8a0091ee72f0133b594b7f49b`, RAW `/home/e/projects/lay-development-runner/firefox-legacy-native-20261012.6onP5c/native20df1-chrome-raw-1791915477741593947/RAW.json`; standard58PASS2BLOCKED/60 (normal19PASS1BLOCKED,private20PASS,fullscreen19PASS1BLOCKED), panel6PASS/6, functional105PASS33FAIL12BLOCKED/150. All20 old private failures became PASS; no standard PASS lost, all156 other statuses equalL. DS406/420 median92.079734ms,p95110.070222ms,max121.588875ms; clockevdev write-return→first exact AT-SPI target,kernelreleaseNOT_MEASURED. Actual private20-cell writer-root observation `/home/ubu/.cache/lay/development/remote-firefox-setup-j2he3qf6/N-chrome-private-sink-proof.json`: usage and Lay history/data file hashes unchanged, normal enabled-public control changed recent_actions.jsonl; bothlearning/debug configuredtrue, before/aftertwoquiescent samples, no extra audit input. Scope is these visited Lay roots/inputs; complete trace coverage remainsUNKNOWN. BothNnativefullverdictsFAIL are retained; TD121OPEN. Scope2046996 closed/allboundPIDticksgonebeforefreeze/restoreVT1. OsourceRED firstlateownedrelease proof, plan review1/2pending, no production edit.

### 2026-10-13 O: authenticated text-inert release (source proof pending)

First failed transition: the native release handler can precede its observer by
more than the fixed stamp budget; generic refusal revoked an already proved
word, then late ingress left an unsettled mutable entry. Source RED
`o-late-managed-release-red-ad5ufwag/RESULT.json`: one causal failure and two
controls passed; receipt hash and options are in decision
`2026-10-13-late-managed-release-word-proof.json`. Independent plan review1/2:8/10.

The existing adapter authenticates a plain legacy release against the live
owner/token and settled ContentType, before the mutation rendezvous. Only an
existing managed press or paired native terminal press with current focus/frame
can retire through this path. The observer classifies these inherently text-inert
releases on the current owner's path as passive metadata, leaving no orphan.
Pending-target input, unpaired releases, Shift/Alt, command modifiers, AtomicV1,
lifecycle and missing mutable callbacks retain their existing protections.
Display draining still uses the current owning-frame checks. No cache/queue,
owner, deadline or delivery route was added; UnknownStart is never promoted.

Remote focused/source, final review2/2 and original native/speed proofs are
pending. Both N scopes are closed; N private runtime bytes and laptop are
unchanged. TD-121 stays OPEN and full N native FAIL is retained.

O remote focused17/17 PASS, 16.117371187s, receipt
`o-late-managed-release-source-ga44kkbz/RESULT.json`, SHA-256
`a7543b39515eba20c7d16aecb5baf2d7bedbbe6a799c5f0d8d04c3921f9f620f`;
all205 IME/daemon Rust file hashes match fetched formatted source. The existing
ordinary unpaired Space release must preserve prior scope too: it has no text
effect, and an authenticated duplicate cannot create authority. Already observed
mutable stamps are explicitly refused via the existing nonwaiting stamp lookup.
Missing settled metadata accepts the initial default(0,0) only; later explicit
metadata must match. Two earlier implementation probes remain FAIL on disk;
no bytes from them were built/installed. Final review2/2 and source/native gates
are pending; delivery routes remain unchanged and TD-121 remains OPEN.

O broad Passive implementation REJECTED: full source1096 executed/1081PASS/15FAIL,
remote immutable `run-WE0tvu`, local `run-epln3f0l/RESULT.json`. The first shared
mechanism is removal of the original ordinary release bridge fence and ordered
replay key settlement. Focused17 and independent code review9/10 did not cover
this whole-path gap. Both O reviews are exhausted; score9 does not certify a
replacement. Rejected source was preserved and adapter/observation restored from
the authoritative N source archive. No O bytes were built or installed.

Refinement under investigation: retain canonical Key admission for every ordinary
release. Record only an already completed authenticated zero-effect managed/native
release in the existing bounded callback stamp store. Its later actual canonical
ingress can consume that completion fact, without an orphan mutable key or any
new word authority. Before callback completion the ordinary release fence stays
closed. No second owner/cache/queue, timeout tuning or delivery route is needed.
Source and native proof remain pending; TD-121 stays OPEN.

2026-10-13T19:20:44Z — User requested one core experiment log to prevent repeated edits and
whole-core rollback. Created [TD-121 experiment log](../development/td121-core-experiments.md)
and linked it from AGENTS.md. It retains exact N baseline/source/runtime identity,
O-R1 full15FAIL rejection and the single unverified O-R2 next step. Prior scores
and focused PASS cannot override broader failures. No source/native check or
runtime installation is claimed for this documentation change.

2026-10-13T19:40:35Z — O-R2 completed-release fact in existing bounded stamp store:
focused29/29 PASS, 16.580660436s,
`o-completed-release-source-49f0u4qb/RESULT.json`, SHA `54110681c04193e4a2704f885199b166dee6b259cf9240cd21c4e79510d0a17e`.
All prior O-R1 failed test IDs and unchanged original release/bridge/replay guards
pass. New admitted UnknownStart positive preserves exact token/scope without
promotion; unproved post-Reset negative keeps the bridge closed. Existing stamp
capacity and one-shot full-header/key tuple, observed mutation no-relabel,
revocation/owner replacement inert retirement and cancellation are covered.
Source repeat guard proved STOP/no new tests on the accidentally repeated failed
state; primary receipts stay immutable. All205 formatted Rust hashes match local.
General source scope pending; no new review/build/native claim, N private bytes
unchanged/GUIOFF/laptop unchanged. See decision2026-10-13-completed-release-canonical-ingress
and the single [experiment log](../development/td121-core-experiments.md).

2026-10-13T19:46:44Z — O-R2 general development1100/1100 PASS/FMT PASS,
local `/home/ubu/.cache/lay/development/run-5erfwyt3/RESULT.json`, immutable remote `/home/e/projects/lay-development-runner/run-CdwrRH`,
archive `f30df6eefaee323eff17e0ec81acae2ec8fe7ffa93efbe7d90284870aa1f20de`. Exact recorded8target affected scope;
not release or native acceptance. Current replacement code review1/2 pending
after this broad GREEN; no reuse of rejectedO-R1 score. N private runtime and
laptop unchanged, GUIOFF. TD121 OPEN.

2026-10-13T20:03:16Z — O-R2 review1/2 score8 found an authentication fixture masking gap,
not a confirmed runtime defect. Unique HeaderKey serials added and focused29
refreshed PASS. Original dirty-completion checks now run both schedules; actual
queued Reset-before-release with handler-first completion then canonical late
ingress preserves revoked predecessor and no-orphan. Supplemental30/30 PASS,
16.818284944s, `o-completed-release-source-esjw0_ey/RESULT.json`,
SHA `1cd6b66ba80eaf10b45bcfc5a255d4993ff0eca9b9795de64145a4a9bba9504f`; all205 current formatted Rust hashes match. Four production files
are unchanged. General fixture-source refresh then review2 pending.
Old1100 gate worker251.6305s vs controller259.9s distinguished; no native latency
claim. O-R2 runtime NOT_TESTED, N private bytes and laptop unchanged.

O-R2 refreshed full1101 result:1099PASS/2FAIL, `run-9fun3w7c/RESULT.json`,
remote `run-bFjOId`. New fixtures assumed a detached Error reply was the next
raw message: prior press reply92500 or genuine preedit Signal arrived first.
Both corrected to use the existing serial-aware peer filter/output collector,
with all semantic scope/epoch/authority/display assertions retained. Production
unchanged; repeated source/native remains pending, no build or promotion.

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

## User-directed local version1.0.83 installation — 2026-10-14

User ended automated client verification and explicitly requested installation with a higher version, saying they will test it. The frozen WJ source from run-w03rUs was copied immutably; only Cargo package/lock version, extension version/date and VERSIONING.md changed before the build. Remote guarded default-feature release build of existing CLI/daemon/IME took184.545s; no source test, browser/native input or model fit was repeated. Shared Cargo target10,117,763,072B remains under12GiB.

Installed CLI/daemon/IME and extension display report1.0.83. Loaded IME SHA256 `b742b64dfd8d1d570e3eb8e354b18ba7040a8941ddc1e9982ff7513bdca6767e`, daemon `bc89b4d0a0ffe253898b1b2db0ba9825a7e92d9829c53bf5537af6927846e034`; CLI `a8b160cb2b5f2e1d7b0ae6e6961bc3ae33bd83e7311eaed95c688dbc2b5b8451`. Three binaries and two extension version files were backed up and hash verified before replacement. Existing atomic-copy, extension reload and Lay channel control were reused, with a focused empty installation witness and zero input keys. Global IBus/L11/L3 PID/starttick/loaded bytes, layout, configured input sources and config remained exact.24 other installed executable/extension files retained their hashes. Delivery routes, static models and input authority were not edited.

Install receipt `/home/ubu/.cache/lay/development/remote-firefox-setup-j2he3qf6/install83-20261014T141659Z/INSTALL.json`; decision `docs/architecture/decisions/2026-10-14-user-directed-version83-install.json`. Core archive `/home/ubu/.cache/lay/development/remote-firefox-setup-j2he3qf6/lay-1.0.83-core.tar.gz`, SHA256 `95dba65eb694f24edc245461d5515dda5c837bbb84b89a2d063111017da29149`. Existing ancillary tools/models are preserved. No GitHub release/tag/push performed.

Source848PASS and native88 results remain historical WJ1.0.82 logic evidence, including49FAIL/7BLOCKED; physical acceptance of1.0.83 is USER_WILL_TEST. Latest complete browser matrix remains WG343P56F33B/432 on old bytes. Full WJ/1.0.83 browser matrix and authenticated Telegram composer are NOT_TESTED. TD121 OPEN and independent final WJ review NOT_RUN remain explicit. No new acceptance, speed sample or rollback execution is claimed.

Remote Telegram diagnostic stopped: all pinned processes gone, cgroup empty, VT1 restored, finite seat stopped. Both private screenshot/focus helper copies and original005 browser attachment guard were restored; owned short runtime alias removed. Telegram profiles/archives retained, messages/sends0. Exact remote closure receipt: `firefox-legacy-native-20261012.6onP5c/WJ-telegram-close-install83.json`.

## Local Firefox1.0.83 iframe diagnostic — 2026-10-14T15:24:54.088336+00:00

User reports Double Shift failure and explicitly requests checking here on the local machine; identifies actual field10, “Редактор внутри iframe”. This authorizes the local diagnostic exception to the prior remote-only test scope. Installed CLI/IME/daemon bytes and user profile are preserved. Actual Firefox157.0 snap8995, PID767837; own standard test window4063878607, all native input bound to that title/window/PID/starttick, actual field focus and existing daemon-attached AT keyboard. Browser/account profiles unchanged; no native text input into WhatsApp or other user sites. No runtime install/restart, delivery-route change, model fit or learning cleanup.

Exact field10 test with simplytyped `просто`: active roundtrip PASS (both surfaces and next-character decoder/Backspace), post-Space5-gesture burst FAIL. First conversion becomes `ghjcnj ` with native caret7 in108.444ms, then gestures2–5 leave it unchanged. All4 reach a live bridge but manual RPC returns `context_authority`; first refusal at authority.rs706, word_known=false, managed_word_mode=false, snapshot present/unselected, witness absent, manual toggle/refresh both false. Between first delegation and second refusal the newUS owner109 sees Reset serials3246/3256/3260/3276/3279; final3279 has refused_timeout. Second marker313 settles in under1ms, so the observed failure is edit admission after successful dispatch, not a missing gesture. The exact first loss of replay/reset authority still requires a causal source test; timeout alone is not asserted as cause. Do not weaken the guard or use a literal word/frame bypass.

Scoped transformation timing3/7 samples,4 missing; median103.207ms, p95/max108.444ms; evdev write-return→AT-SPI observation, not human-release or paint. Input2plan verdicts1PASS1FAIL; no conversion failure excluded from timing denominator. First observed refusal, raw sequence and all earlier setup scopes retained at `/home/ubu/.cache/lay/development/firefox83-local-sequence-kwjodjvq`; root assessment `/home/ubu/.cache/lay/development/firefox83-local-sequence-kwjodjvq/ASSESSMENT.json`, raw SHA1c6e2634a55e57b0aaa831d0666d33805b3cf8a582142f1a5b0bf5a17b6e98ff. Field1/7 local active roundtrips and field1/7 five-target controls pass in separate scopes; they do not certify iframe10. Remaining8-field actor sent0keys because Firefox was behind Kitty while user replied; classify16plans BLOCKED_SETUP, not product failures. Earlier observer artifacts are explicitly retained rather than silently promoted.

Latest complete WG browser counts remain Firefox164P33F19B/216, Chrome179P23F14B/216, total343P56F33B/432. FullWJ/1.0.83 browser acceptance NOT_TESTED; TD121 OPEN and independent WJ review NOT_RUN retained. One local reproduction finished; no product source edits or rollback.


## Completed native replay: same-gesture exact refresh — 2026-10-14T17:12:42.739789+00:00

Local iframe10 after-Space repetition has a completed paired native replay and actual Reset but its last surrounding receipt still reports the prior surface. The current token remains UnknownStart; no completion sentinel is visible. Causal source RED uses the actual delivered replay/Reset/old-receipt/ManualToggleV3 sequence and returns NotHandled0 instead of AwaitingExactSnapshot4: `firefox-legacy-native-20261012.6onP5c/iframe-refresh-red-7e7n5ns2/RESULT.json`. The earlier fixture setup mistakes are retained; the valid Reset/boundary provenance union is proved before the RPC assertion.

Minimal fix: manual-only wait eligibility under the same current reducer owner/token and completed paired replay, marked exact-refresh capability and actual unselected prior receipt at end caret. Reuse existing RequireSurroundingText and existing700ms gesture lease; await the existing actual final receipt before delegation. Space/Tab/prediction eligibility, transports, verifier and SafetyGate remain unchanged. Decision: `decisions/2026-10-14-completed-replay-same-gesture-refresh.json`.

Focused2PASS includes four prefix/separator combinations and six adverse mechanisms (unmatched projection, incomplete replay, selection, foreign prefix, extra epoch, expired scope), checking exact outcome and absence of edit signals before the receipt. Full IME source799/799PASS in119.9s controller time: `/home/ubu/.cache/lay/development/run-w68csclm/RESULT.json`, source archive65315e4a9b1eb6c4d65be73923a1c9fe93d24aa448d8b921529f64684ce8e5f4. Six integration contracts and architecture refresh pending. This is source evidence only; changed-byte installed/native acceptance NOT_TESTED.

The user explicitly allows any available nonsensitive Telegram text field without login or message sending. Existing local SnapTelegramPID5431 was activated through its ordinary GNOME overview tile; no new profile or account login. Its conversion failure has not yet been causally mapped.

Fresh context-free review spawn rejected by agent thread limit; review NOT_RUN, no score. TD121 remains OPEN; no commit/push/tag/promotion. Latest complete browser evidence stays WG343P56F33B/432 on earlier bytes.


2026-10-14T17:20:43.111177+00:00 — Completed-replay fix source closure:799IME+51integration=850/850PASS, canonical remote graph+architecture guardsPASS. Only six Rust paths differ from installed83source: four production eligibility call sites/files, residual module include, new causal fixture. Default-feature frozen gated IME built remotely183.364s; candidate SHA330eda0c21a93a446eb96224f59c2f63f54c1291aee486d6a7d8778ad34fd191. Only this IME installed through existing fresh-empty-field+atomic-copy+channel-ime guard. Installation receipt `/home/ubu/.cache/lay/development/remote-firefox-setup-j2he3qf6/install-iframe-refresh-20261014T171918Z/INSTALL.json`; newIMEPID1525739/tick20955217, same daemon/IBus/L11/L3 identities/config/layout/sources and28protected files. Version1.0.83 remains; this changed hash is a fix trial, not publication or full native acceptance. Causal iframe10 recheck nowRUNNING; full browsermatrixNOT_TESTED/reviewNOT_RUN/threadlimit/TD121OPEN. No globalIBus restart or route/model change.


2026-10-14T17:36:14.668777+00:00 — Completed-replay increment source850/850+ASTPASS, installedIME330eda0c.../version1.0.83; exact original iframe10 scene now2/2PASS with all7 targets timed (median115.234ms, p95/max222.170ms). Telegram explicit USER_ACCEPTED “Телга отлично работает!”; further Telegram tests stop. Changed scope and exact receipts in [final increment report](../development/completed-native-replay-refresh-2026-10-14.md). Current full browser matrix NOT_TESTED; latest completeWG343P56F33B/432 retained. Independent reviewNOT_RUN/threadlimit/no score; TD121OPEN; no publication or universal claim.


2026-10-14T22:13:08.447907+00:00 — Performance request read-only audit: prior7 samples mix active2(153.558/222.170) and afterSpace5(89.897–120.116/median113.214). p95max on7 is not a stable population percentile. Four burst target observations follow next pair firstdown due observer gaps; alternating surfaces observed, exact completion before next pair NOT_PROVEN, do not promote to strict standard. Source existing replay sleeps34–37ms for6/7 chars;10ms readiness poll and actual client/IPC costs need grouped stage measurements. Installed330eda0c... andbc89b4... preserved/live verified, no newinput/fit/install/restart/routes. Options9continuousprofile+largestmeasuredstage/6boundedpacingA-B/4fasterpoll. [Facts and next bounded experiment](../development/double-shift-latency-2026-10-15.md). LatestcompleteWG343P56F33B; currentfullmatrixNT; TelegramUSER_ACCEPTED remains finished.


2026-10-14T23:03:55.418448+00:00 — Manual latency optimization: add inert existing-sink stage durations only in two daemon files, disabled via existing debug flag; same leases/requests/routing/pacing/deadlines. Source/private native pending; installed accepted83 unchanged. Decision2026-10-15-manual-replay-stage-timing.json. No new owner, polling, transport or raw payload.


2026-10-14T23:26:10.228789+00:00 — Profiling baseline324sourcePASS+AST/defaultdaemon adec3cdf...; sameinstalledIME330eda0c..., installedlocalbytesunchanged. Actual remote native timing median replay69/delete28/layout24/dispatch5ms (partial original standard+panel), CLIglobalengine30pairs median5.362ms vsnative0.386ms. Select only current bounded replay trailing deletion wait16→4ms candidate; leave2ms/key and ordinary/unbounded16ms, all keyframes/authority/verification/currenttransports. Source and native A/B PENDING, predicted12ms is not a measured speedup. Decision2026-10-15-isolated-replay-post-delete-settle.json. Private firstzero attempt failed optionalGI.serialize metadata after0input, retained; corrected read-only rawsamebusGetGlobalEngine thenfreshstrictattachmentPASS.


2026-10-14T23:31:24.206429+00:00 — Temporary stage diagnostic retired after localization: source inspection shows no per-field privacy fence, so timing writes would violate existing Chrome no-history all-sink quiescence even without raw text. Installed user runtime unchanged. Keep collected private diagnostic FF216=169P28F19B and324source/AST/binary as diagnostic history only. Final candidate removes BOTH diagnostic additions completely; sole runtime delta is bounded post-delete wait16→4 with shared frame emitter. Matched performance baseline/candidate must both use NO instrumentation; rerun true accepted83 baseline and preserve Chrome privacy bracket without exceptions. Source snapshot of intermediate327gate remains separate if stillrunning. No relaxed guard/route/model change.


Performance follow-up2026-10-15: independent fresh-context read-only review completed via separate ephemeral Codex after built-in thread quota rejection:9/10, no blocking source defect; receipt `/home/ubu/.cache/lay/development/remote-firefox-setup-j2he3qf6/PERF-pacing-review/REVIEW-1.md`. Applied its two nonblocking test-only suggestions (explicit2ms×n then16/4ms schedules; prohibit an extra ordinary delete). Runtime production sections unchanged; follow-up source gate pending. Clean PLAIN Firefox original first8 jobs186 rows complete; final30 interrupted at field9 pre-input SHOWING guard, six priorPASS/remainingNOT_TESTED retained in `firefox-functional-c3xe2k7g/RESULT.json`. Added only pre-input750ms visibility wait; no key/target/clock/deadline or guard relaxation, new SHA67f5012c... explicitly pinned. Failed zero-key pin mismatch0behavior input retained. Finish original final30 in fresh exact-owned scope; do not erase interrupted evidence or substitute it for final complete acceptance. Candidate/native speed stillPENDING; accepted local bytes untouched.


Performance attempt1 closed2026-10-15: clean PLAIN Firefox216=170PASS/27FAIL/19BLOCKED. Standard420/420 measured, median106.640ms/p95119.943ms. All216 IDs preserved in `/home/ubu/.cache/lay/development/remote-firefox-setup-j2he3qf6/PERF-PLAIN-firefox/RESULT.json`. Four-ms daemoncc65fbe7... rejected after normal20=19PASS/1FAIL: field6/space/burst lost a baselinePASS. Same-window clean baseline140 samples median106.081ms/p95117.449ms; shortened candidate137/140 observed samples around89ms, three missing. Speed does not permit promotion. First failed transition is third burst bridge_context_authority refusal (missing witness), before that gesture deletion; causal dependence on4ms NOT_ESTABLISHED. Do not weaken admission or retry unchanged untilPASS. Raw primary `firefox-standard-normal-k9zlr898/RESULT.json`, SHA7963d6c9732369749f1c3e1afe8849c1f4d7800736706e2b287c586b81587449, rejection `/home/e/projects/lay-development-runner/firefox-legacy-native-20261012.6onP5c/PERF-FAST4-REJECTED/RESULT.json`. Incomplete private job stopped; all pinned GUI gone/scopeempty/VT1. No current full candidate matrix, ChromeNOT_TESTED. Next single bounded replan8ms (predicted8ms saving); no other production change. Fresh327 source check running. Accepted local runtime remains unchanged; no model training or route edits.


Eight-ms Firefox A/B complete: all216 stableID verdicts identical (170PASS/27FAIL/19BLOCKED), no lostPASS or changedverdict. Same-source actors/60standard+6panel+150functional, closure5pinnedgone/CGempty/VT1/noheldkeys. Standard420/420each: median106.640→93.547ms,p95119.943→115.286ms, pairedmedian improvement11.133ms(n420). Panel48/48each:102.928→92.392ms; functionalDoubleShift134/210each:104.109→93.041ms,p95140.378→131.893ms,76missingretained. Space37/50each:17.538→18.004ms, pairedchange-0.243ms; no speedupclaim forSpace. Tab0/10 and selectionbaseline0/10/candidate1/10 remain insufficient for speedclaims, neitherconvertedtozero. Compare receipt /home/ubu/.cache/lay/development/remote-firefox-setup-j2he3qf6/PERF-FAST-firefox-vs-PLAIN.json,SHAe0ea886f21657229fd92c4840a13ef74a22e2c89acb8378111f917282161c8b3. Raw9primaries/logs/bodies retainedlocallyBDIR/PERF-FAST-firefox and remotelyRUN/PERF-FAST/firefox. Nativeclock evdev write-return→exactAT-SPI, notkernel/paint. PLAINChrome nowrunning; both-browsercomplete current-source verdict stillPENDING, latestcompleteWGmatrixretained. Acceptedlocalruntimeunchanged.


Clean PLAIN BOTH baseline complete: Firefox170PASS/27FAIL/19BLOCKED +Chrome179PASS/23FAIL/14BLOCKED =349PASS/50FAIL/33BLOCKED of432,0NOT_TESTED. Exact acceptedBC89daemon/330IME in both, same private frozen data/usage and fixed216/browser; compare with futureFASTboth, never mix phases. Local primariesBDIR/PERF-PLAIN-firefox/RESULT.json and PERF-PLAIN-chrome/RESULT.json; remoteRUN/PERF-PLAIN/{browser}/RESULT.json. Chrome standard406/420measured,14missing retained:median92.055ms,p95111.443ms. ChromeprivacyPASS_VISITED_LAY_SINK_ROOTS normalpositivecontrolrecent_actions.jsonlchanged/privateunchanged (localBDIR/PERFPLAIN-chrome-private-sink-proof.json); scopevisitedLaywriterroots only. Closureall5goneCGemptyVT1/noheldverified. Latestcompleteboth table now this349/50/33baseline, replaceshistoricalWG343/56/33 forstatus. New8msChrome running; final8msboth gatePENDING. Runtimeauthority/installedbytes unchanged.


Eight-ms attempt REJECTED after fullBOTH: Firefox170P27F19B,Chrome178P24F14B,total348P51F33B/432,0NOT_TESTED; acceptedcleanbaseline349P50F33B. Chrome lost exactlyfunctional/normal/space_ru_en/field8: expectedlaySPACE, actualдфнlaySPACE. FirstobservedfailureSpace target, not a DoubleShift action; exactcausaldependenceon8msUNKNOWN, no waiver. Chrome406/420standardmedian92.055→86.585,p95111.443→102.286,paired6.147ms; privacybothPASS. Gains do not justifylostPASS. ComparisonlocalBDIR/PERF-FAST-chrome-vs-PLAIN.json, raw9primaries/logs/bodiesBDIR/PERF-FAST-chrome andremoteRUN/PERF-FAST/chrome. Closureall5gone/CGempty/VT1/noheld. PreparedinstallergatesrejectlostPASSand wasNOTRUN; acceptedlocalBC89daemon/330IME unchanged. RevertONLYown3-filepacingincrement toexactaccepted83copies, retainallfailedsource/build/native/reviewproofs. Do not silentlyreintroduce4or8ms or retryuntilPASS. Next bounded alternative: measuredread-onlyIBus engine query cost5.362msCLI vs0.386msnative; inspectexistingconnectionfirst, no input-route/owner/cache changes withoutdecision.


Next bounded performance proposal (no code yet): decision2026-10-15-observed-ibus-engine-readback.json. ExistingLayImeBridge already owns the actualstockIBusconnection(server.rs:253); reuse it for a fresh GetGlobalEngine getter on existingLayImeRPC, rather than a new daemonbus/cache or GNOMEprojection. Compare validactualengine normally; onlytransport/unsupported/decodingerror fallsback to originalCLI. Read no text/state, hold no sharedstatelock, write no tracing/history. Preserve16ms+2msphysicalcadence and everyexistinglease/verificationdeadline. Existing30-pair measuredreadcost motivates this proposal, not a claimed speed result. Options9/10(existingconnection),5/10(newconnection/cache),1/10(projectedlayout,REJECT). Freshcontextplanreview running; sourceimplementation and nativeproofNOT_TESTED. Old4/8mspromotionrejectionsremainbinding.


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

# Firefox input stability — 2026-10-05

## 2026-10-18 — TD-156 presentation root and source proof

Follow-up: final code review 8/10 after two passes. Exact replay/capability
cleanup and Reset hide-failure retry now have entrypoint controls. New atomic
handover regression is causally RED at unsupported auxiliary clear in
`/home/ubu/.cache/lay/development/run-k8bw_66_/RESULT.json` (remote `run-ezO1J4`).
Guard that output by the existing transport capability, preserving authority
retirement and actual composition clear. The unchanged ordinary-stack isolated
native-transfer regression passes after heap-pinning the shared clear-output
future; pinning only the emitter was insufficient. Focused result remains
796P/38F/834: atomic RED plus 37 old presentation assertions at four shared
sites. Align typed-only preedit/auxiliary expectations without relaxing exact
edit, refusal or learning assertions. Full source and changed-byte native
acceptance are pending; original browser matrix remains 349P/50F/33B/432.

Exact unchanged menu-90 private bytes were exercised in a finite remote GNOME
Wayland seat. Firefox 157.0.1 / BuildID 20261005135250, GTK 3.24.52 and IBus
1.5.34~rc2 supersede stale harness version labels. Original field-1 fifteen-plan
diagnostic: **11 PASS, 3 FAIL, 1 BLOCKED**; not a replacement for the original
216-field browser denominator. Receipt:
`/home/e/projects/lay-development-runner/firefox-legacy-native-20261012.6onP5c/firefox-functional-rvdztq1d/RESULT.json`.
Owned seat/cgroup closed, no held keys, VT1 restored; closure
`TD156-baseline/closed.json` at 2026-10-18T01:43:25.251649Z.

Passive client log `TD156-baseline/firefox-ime.log.moz_log` establishes a
suffix-only composition still present after GTK Reset returns; later clear
does not prevent committed ghost text on focus return. Pinned Mozilla source
revision `737ec77d798755890b062aab541a90d6c75afd19`,
`dom/events/TextComposition.cpp:449–477,721–791`, saves composition data before
IME notification, restores it on an empty commit response and may synthesize
the saved commit. This disqualifies a Reset-only GTK compatibility patch.
No such patch was implemented. Independent plan review: Reset-only 2/10,
standard auxiliary presentation 8/10, lookup-table alternative 6/10.

Chosen bounded design and full consequence analysis are in TD-156 and
`decisions/2026-10-18-separate-legacy-ibus-hint-from-document-composition.json`.
Use standard auxiliary text on the same legacy IBus owner/connection; keep
typed composition and verified Tab authority. Owned and suffix-only legacy
hints both require separation. Atomic proposal protocol/behavior is preserved,
outside this repair's acceptance claim; no extra opcode or out-of-transaction
effect. No ranking, verifier, model, edit-delivery or local-runtime change.

Separate semantic probe `TD156-baseline/core-decision.json` finds the undo
fixture's intended target still present as SuggestOnly, refused by
`productive_v90_lattice_requires_common_l3`; a larger wait cannot repair that
semantic precondition. It does not prove an undo failure. Chrome's remaining
first transitions require their own proof. Changed-byte native, visible
auxiliary panel, latency and Plasma remain **NOT_TESTED**. Latest complete
original native matrix remains Firefox 170/27/19 and Chrome 179/23/14 of 216
each (349/50/33 of 432). Runtime authority changed: **no**.

Source RED before the production change: the new legacy publication regression
fails because an unaccepted suffix is exposed as document composition;
827 existing cases pass. Exact receipt:
`/home/ubu/.cache/lay/development/run-nk_vyipc/RESULT.json`
(remote `run-5RHuJS`). The first implementation passes all three new legacy
display tests, but an existing atomic pending-display failure control fails.
The runner also reports a missing isolated status, so that run is not GREEN:
`run-2ii4lknx/RESULT.json` (remote `run-PydwIg`). Restrict the new failure
cleanup to legacy auxiliary output; retain atomic transaction semantics and
old inert pending display. Add background publication/clear failure controls
for owned and committed-tail contexts. Remote affected-closure and independent
code review are in progress; no changed binary is installed or accepted.

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

### TD-156 source closure checkpoint — 2026-10-18T02:28:18+00:00

Remote focused IME closure is PASS: **834/834**, receipt
`/home/ubu/.cache/lay/development/run-aozwx8jf/RESULT.json`, remote
`run-lfWZP1`. The actual first Atomic V1 Backspace handover regression and the
unchanged ordinary-stack isolated native-transfer test both pass. No protocol,
stack limit, safety gate or delivery authority was relaxed.

The preceding `run-rg14z0xt` / `run-ftkZO2` was 809 PASS / 25 FAIL. The failures
were grouped presentation contracts and historical document-preedit fixtures.
Current legacy publication fixtures now assert auxiliary updates and exact
hide/Space/Tab effects; historical delayed-document controls replay an explicit
document-preedit publication through the existing output and witness entrypoints.
They no longer pretend that a new auxiliary hint is document composition.
Current deferred publication and stale-release tests check auxiliary effects as
well as preedit; ownership, refusal and learning assertions remain.

Canonical manifest refresh `tab-manifest-ds_n3fce` / `tab-manifest-meIIjJ`: PASS,
3245 discovered; the focused closure selects 834. Independent production review
remains the completed two passes (7/10 then 8/10); no third review was added.
Full source gate, changed-byte native acceptance, visible panel, latency and
Plasma remain pending. No local installation/restart and no runtime authority
change occurred. The historical 349 PASS / 50 FAIL / 33 BLOCKED native matrix
remains the latest complete receipt.

### TD-156 complete test execution, stale inventory binding — 2026-10-18T02:38:48.108823+00:00

Canonical full run `run-0st554ds` / remote `run-y5ihP7` returned **FAIL**,
not a release proof. Anchored per-target execution records show 33 selected
targets, 3219 tests and zero test failures. Final aggregation refused the stale
`known_failures.json.manifest_sha256` after the regression inventory refresh.
Rebind that hash only; keep zero allowed failures, the historical observation
and every runner check. No runtime source change or rerouted delivery is made.
Refresh the graph and rerun the canonical full gate before the private build.
Native changed bytes and KDE remain NOT_TESTED.

### TD-156 full source closure — 2026-10-18T02:49:43Z

After the inventory hash rebind, canonical full run
`/home/ubu/.cache/lay/development/run-zri9cql9/RESULT.json`, remote
`run-y0fkQZ`, is **PASS**, 3219 selected correctness/package tests, zero allowed
or observed semantic failures. Runtime authority is unchanged. Canonical graph
refresh `tab-graph-eqmwqrop/RESULT.json` also passed. The source archive SHA-256 is
`4ef4ee0b19bae9930e359ead5344df02fd0fe3e2460fe66472ed4a9b4a344ac5`.
The same tested executable/test/dependency inputs were frozen for the private
release worker at `td156-auxiliary-build-20261018`; its remaining gates/build are
in progress. This proves source closure only. Changed-byte native Firefox,
Chrome, auxiliary visibility/Tab, latency and actual Plasma are **NOT_TESTED**.
No local installation/restart or delivery/decision authority change occurred.
The latest complete native matrix remains the historical 349/50/33 of 432.

### TD-156 protected inventory metadata correction — 2026-10-18T02:53:14Z

The first private build stopped before release compilation: the explicit
changed-inventory canon check reported
`protected_change_without_new_decision:scripts/test-lanes/manifest.json`.
The ADR already described the new regression inventory and zero-failure hash
binding but omitted the protected manifest path. Add that path and clarify
the unchanged lane/ignored-test policy; no code, test expectation or guard
is relaxed. Preserve the failed immutable build at
`td156-auxiliary-build-20261018`. The retry is a distinct immutable stage
`td156-auxiliary-build-20261018-b`, archive
`2a86fd5b8f28a2e118e9c619cab6b54315d0f5b630727e95e4fa865fd951b7aa`.
All executable/test/dependency inputs still match the full 3219-test PASS.
The retry checks canon against the actual changed/new inventory again.
Native changed bytes, timings and Plasma remain NOT_TESTED.

### TD-156 private build and native startup — 2026-10-18T02:59:32Z

Private build `td156-auxiliary-build-20261018-b/BUILD.json` passed security,
linters, actual changed-inventory canon and release compilation, with source
parity to the full 3219-test PASS. Build/check duration: 234.52 s. IME SHA-256
is `5056dd3e50089fb2809b2cdcc717b8f98db804c660ef8c5deb64cdcceb493697`;
daemon is `1411ab174a7171c265f3c1af73b1c9a6a2dd8dd1e08d05827dac1c89208cfb15`.
These are private candidate bytes, not an installed/accepted release.

The first native controller invocation was rejected before any full-matrix
job: its `TD156AUX` zero-key tag violated the unchanged alphabetic-tag guard.
Preserve that aborted attempt under `TD156-auxiliary/attempt-zero-tag-rejected`,
restore frozen test data, close the empty owned GUI/seat and restore VT1.
A repeated finite-seat label also met its existing immutable receipt; preserve
that receipt and use the unique `tdaux-firefoxb` seat label. Correct only the
controller zero-key tag to `TDAUX`; keep phase identity `TD156AUX`, all original
actors, 216 identities, setup guards and cleanup checks unchanged. The next
zero-key setup passed with exact candidate bytes and the original Firefox
normal job is queued. Full matrices, visible auxiliary acceptance, latency
and Plasma remain pending. No local runtime installation/restart occurred.

### TD-156 Firefox native result — 2026-10-18T03:18:37Z

The changed-byte original 216-cell Firefox matrix is complete: **176 PASS /
20 FAIL / 20 BLOCKED**, zero missing identities. Receipt:
`/home/ubu/.cache/lay/development/browser-final-fixes-20261018/native-firefox/RESULT.json`;
remote `TD156-auxiliary/firefox-matrix/RESULT.json`. All nine original parents
exited zero, no held keys, exact candidate/process identities verified; GUI
cgroup and owned seat closed and VT1 restored (`TD156AUX-firefox-closed.json`).
Compared with 1.0.89, seven selection cells (fields 1–6 and 9) change FAIL to
PASS. One earlier PASS (base, field 9) is now BLOCKED because the fixture field
was hidden. No earlier PASS became FAIL. Do not count that untested cell as
preserved behavior or combine this row with old Chrome as a candidate matrix.

Basic Double Shift remains 60/60 PASS, panel 6/6 PASS. Basic latency has full
420/420 measurement coverage: median 99.15 ms, p95 112.28 ms, max 124.70 ms;
baseline median 98.46 ms, p95 111.25 ms. These are evdev write-return to exact
AT-SPI target observations, not kernel event delivery times. Functional Double
Shift covers 146/210 timings (median 97.85 ms, p95 131.00 ms); Space 37/50
(median 18.85 ms, p95 25.24 ms); selection 7/10 (median 17.10 ms, p95 20.20 ms).
Missing timings remain missing. There is no general speedup claim.

Supplementary auxiliary diagnostic B (`firefox-functional-j4bnb4u_/RESULT.json`)
observes an actual SHOWING auxiliary panel in all ten fields. Seven fields
complete Tab and both accepted/unaccepted focus-retirement checks. In rich
fields 7, 8 and 10 the raw successful Tab target ends in NBSP, while the
diagnostic incorrectly required ASCII space. Preserve its raw 7 PASS / 3 FAIL;
the three failures are observation mismatches, with later retirement steps
NOT_TESTED, not three established product defects. Diagnostic C prospectively
accepts the existing HTML boundary normalization for Tab, and requires exact
raw text across subsequent focus changes. Original 216-cell actors are
unchanged. Chrome is running on the same candidate bytes. Plasma is pending.
Runtime authority, models and local installed processes are unchanged.

### TD-156 complete browser candidate and KDE preparation — 2026-10-18T03:30:51Z

The same candidate now has both complete original matrices: Firefox **176 PASS /
20 FAIL / 20 BLOCKED**, Chrome **178 PASS / 25 FAIL / 13 BLOCKED**; combined
**354 PASS / 45 FAIL / 33 BLOCKED of 432**, zero missing identities. Chrome:
`/home/ubu/.cache/lay/development/browser-final-fixes-20261018/native-chrome/RESULT.json`.
Both sets retain all nine zero-exit job parents, source/config/process binding,
no held keys, empty owned GUI cgroups and exact owned-seat closure with VT1
restored. Chrome closure: `TD156AUX-chrome-closed.json`, 03:29:10 UTC. Its GUI
shutdown printed a SIGTERM during child cleanup; independent closure proves
the pinned processes/cgroup are gone. This does not erase that shutdown output.

Compare against the immutable local **1.0.89** receipts listed by TD-156, not
the remote PERF-READV Chrome receipt (that older receipt has a different undo
cell). Exact identity delta is in `native-v89-comparison.json`: seven Firefox
selection FAIL become PASS; two formerly PASS cells become BLOCKED (Firefox
base/9 hidden fixture; Chrome selection/2 seed unavailable); three formerly
BLOCKED Chrome cells now reach FAIL (two search-field Backspace returns and
one undo precondition); Chrome prefix/2 changes FAIL to BLOCKED. No earlier
PASS becomes FAIL. The candidate is not fully accepted: the two lost PASS
observations and all residuals remain explicit.

Chrome basic timings cover **408/420**, missing 12: median 82.19 ms,
p95 100.52 ms, max 115.41 ms; panel 48/48, median 82.35 ms, p95 88.95 ms.
Functional Double Shift covers 156/210, median 82.08 ms, p95 110.19 ms;
Space 39/50, median 18.41 ms, p95 23.97 ms; selection 9/10, median 20.18 ms,
p95 22.41 ms; original fixed-candidate Tab 0/10. These are successful measured
evdev-write-return to exact AT-SPI target observations with all missing counts
retained. They do not establish acceleration or universal latency.

Supplementary Chrome diagnostic C (`chrome-functional-suhfbc_f/RESULT.json`,
local `native-chrome/AUX-DIAGNOSTIC-C.json`) is **8 PASS / 2 FAIL**. All ten
fields show the actual auxiliary panel and accept its published word by Tab.
Fields 7 and 8 preserve the exact raw text across focus changes but return
with caret zero rather than ten. Later unaccepted-retirement checks for these
two fields are NOT_TESTED. Establish ordinary-client focus behavior before
attributing that caret movement to Lay; do not force the caret to obtain PASS.
This supplementary result does not replace the original fixed 216 identities.

First remaining mechanisms: 20 Space-precondition failures before undo is
attempted; 20 focus-return failures (16 Double Shift context-admission, four
new-letter/caret transitions); five Chrome search-field Backspace-return
failures. Original Tab fixtures require an unavailable specific candidate
(20 BLOCKED). Other seed/visibility prerequisites remain BLOCKED. Chrome wire
trace for search/2 shows empty CLEAR preedit and hide successfully emitted,
Backspace consumed, but stale AT-SPI text. Pinned Chromium 155.0.8059.39
`InputMethodAuraLinux` source is saved as `chromium155-input-method.cc`, SHA-256
`2b798560e9b12ac9d1974b1f96c3c17708710568f2933631b6bd063c932fc7fd`.
Its empty-preedit path invokes ClearCompositionText; engine output alone does
not distinguish client/document state from an accessibility cache defect.

Both browser seats are closed before the authorized remote minimal Plasma
installation. Package installation, genuine Plasma/Wayland client acceptance,
production Qt tray menu and latency remain distinct pending gates. Private
KDE helpers are test harness code, not a replacement runtime owner or a fake
GNOME/KDE bridge. Local runtime, delivery routes and model authority unchanged.

### TD-156 genuine Plasma setup and old-IBus failure — 2026-10-18T03:51:26Z

Minimal Plasma installation completed under the remote canonical guard in
103.70 s: Plasma Desktop/Workspace and KWin 5.24.7, KWrite/Konsole 21.12.3.
330 new packages, zero upgrades/removals; dpkg audit clean. No display manager
was installed or changed, no reboot and no local runtime replacement.
Exact package/installation receipts: local
`/home/ubu/.cache/lay/development/browser-final-fixes-20261018/kde-install/`,
remote `/home/e/projects/lay-development-runner/td156-plasma-install-20261018/`.

The finite owned VT8 session starts actual installed KWin/Plasma using VKMS
DRM and libinput. KWin's file capability makes `/proc/PID/exe` root-readable;
setup now verifies the actual executable/hash, PID, start tick and cgroup with
sudo. Stock Polkit 0.105 required a temporary `org.freedesktop.login1.chvt`
setup permission for the already-sudo-authorized test user. Every attempt has
an independent 90-second expiry, verified exact-file removal before clients or
keys, and a setup receipt. This is test-session admission, not an IME route
change. The stock IBus abstract socket is accepted only with the same private
path boundary and exact owned `SO_PEERCRED` process identity.

On host IBus **1.5.26**, candidate IME SHA
`5056dd3e50089fb2809b2cdcc717b8f98db804c660ef8c5deb64cdcceb493697`
self-aborts **before any injected key**. Full signal/write trace identifies
`zbus 5.15.0/src/message/fields.rs:219`: `Invalid field reconstruction:
InvalidName("Invalid unique name...")` on the connection executor. It is not
resource exhaustion, a test timeout, or evidence of failed text editing.
Receipt/trace: remote `td156-kde-native-20261018/attempt-ime-zbus-name-panic/`.
A retained L1.1 child caused guard `descendant_cleanup`; the exact owned scope
was explicitly closed and its empty state recorded. All prior failed setup
attempts remain separate zero-key evidence; none is counted as native PASS.

Upstream IBus 1.5.31 release/commits 927ce00 and 2b14625 document the unique-name
change. Exact raw offending sender has not been wire-captured, so association
with the old sender convention is a hypothesis supported by the stack and
version history. Do not weaken name validation or create an unchecked route.
For KDE client coverage with the unchanged candidate, use the existing pinned
private **IBus 1.5.34-rc2** root already used by the original browser receipts;
host Plasma/toolkits remain installed packages. Host 1.5.26 compatibility stays
open and is not certified by that separate environment. Source tests remain
3219 PASS; original browser matrix remains 354 PASS / 45 FAIL / 33 BLOCKED.
KDE input and production tray acceptance remain NOT_TESTED at this checkpoint.
No runtime authority, delivery route, model or local installed process changed.

### TD-156 matching host-built IBus and sequential KDE clients — 2026-10-18T04:05:03Z

The earlier proposal to combine the pinned rootfs IBus daemon with host client
libraries was not accepted. `attempt-mixed-ibus-versions/` retains the raw empty
client-text failures and exact scope closure; they do not establish Lay editing
failures or native KDE acceptance. The host client library is 1.5.26, whereas the
pinned daemon is 1.5.34-rc2. The rootfs library itself requires a newer libc/GLib
than this Ubuntu host, so it cannot be injected into host Plasma applications.

Built unmodified upstream IBus tag 1.5.34-rc2 against host GLib 2.72/libc 2.35,
under the canonical dedicated-20cpu guard, into the private test prefix
`/home/e/projects/lay-development-runner/td156-kde-native-20261018/ibus-host`.
Final configure/build/private install/relocation check PASS in 12.85 s.
Source archive SHA-256:
`101167b87e281915594a1cc277382e9e707dabf3182a50ec88c56fd3a8e7f0ff`.
Daemon SHA-256 `9210b84c05315cc250ff66350bda4577ca99cbc45d70f6807f135964b7f9a504`;
libibus SHA-256 `b79ab00fa6101f2debb65b057fb79f29551e15459284591707cc06ff6707297e`.
Receipt: `td156-kde-prepare/host-ibus-build-e/RESULT.json`, copied into the
native run as `IBUS-HOST-BUILD.json`. Earlier dependency/configuration failures
remain in host-ibus-build through host-ibus-build-d. No upstream IBus source
patch, host IBus replacement, global service restart or dependency mutation in
Lay was made. Surrounding-text support remains enabled. Native GTK/Firefox maps
confirmed the private libibus; Qt uses its installed direct-DBus IBus plugin.
Host IBus 1.5.26 panic compatibility remains an open product limitation.

The first matching-library attempt launched all six client applications at
once and hit the existing 512-task ceiling: pids.peak=512, pids.events max=14,
no memory high/max/OOM events. The actor could not start the device-close thread;
there is no accepted native matrix from that attempt. Raw logs and the empty
owned scope proof are retained in `attempt-concurrent-clients-task-cap/`.
Keep the resource ceiling. The next finite Plasma session launches one owned
client process group at a time, verifies its UID/cgroup before closure, and
publishes its actual identity before the native actor starts. GTK first checks
ordinary US/RU typing and Space commit. Result evidence is saved before device
cleanup. Temporary chvt setup permission was again removed before any input.

At this checkpoint the sequential KDE session is ready and its GTK job is
running; 432 KDE-native cells and the production tray menu are NOT_TESTED until
their own complete receipts. Existing source proof remains 3219 PASS, and the
complete original browser matrix remains 354 PASS / 45 FAIL / 33 BLOCKED.
Candidate bytes, runtime authority, delivery routes, models and local installed
runtime remain unchanged. KDE Wayland/VKMS coverage does not certify X11 or
hardware rendering.


### 2026-10-18T04:14:51Z — KDE wire diagnosis and first completed native client

The sequential matching-IBus attempt still failed ordinary GTK input with
pids.peak=304 and pids.events max=0. It disproves the earlier hypothesis that
matching client/daemon library versions alone fixes that failure. Daemon-side
GDBus capture then proved actual CommitText("h" ... "o") reception from Lay
and relay to the IBus InputContext, while GTK remained empty. A second capture
on the actual GTK process showed it using the session-bus IBus portal and
receiving no CommitText. The private harness set `IBUS_USE_PORTAL=0`; unmodified
upstream `src/ibusbus.c:534` selects the portal whenever that variable exists,
regardless of its value. This was a harness configuration error. The variable
is now absent in the private launcher/session, using ordinary toolkit IBus
discovery. This changes no Lay source, output contract or installed route.

Raw evidence remains under the remote native run:
`attempt-matched-ibus-empty-gtk/`,
`attempt-daemon-commit-relayed-empty-gtk/` and
`attempt-forced-portal-client-wire/`. A stale per-session keyboard-ACL marker
and a later missing private KDE layout configuration each stopped setup; those
zero-letter attempts remain separately archived. The launcher now rejects old
keyboard authorization markers and restores the exact three private Plasma
configuration files from its original preparation (systemdBoot=false,
us/ru layouts and QPainter). It requires the actual KDE keyboard-layout service
before publishing readiness. These are harness repairs, not product PASS.

The next untraced native run uses the same candidate engine SHA-256
5056dd3e50089fb2809b2cdcc717b8f98db804c660ef8c5deb64cdcceb493697.
Ordinary GTK hello / hello-space / привет / привет-space all PASS, with actual
committed widget text, caret and layout. GTK's complete 32-case receipt has
8 PASS / 22 FAIL / 2 BLOCKED; six successful measured Space operations have
median 15.51 ms, p95/max 16.95 ms (evdev write return to observed exact native
target). This is only the GTK receipt, not a complete KDE acceptance result.
Receipt: `td156-kde-native-20261018/native-windows-4qbgq4t1/RESULT.json`.

A real product refusal is now visible: the daemon recognizes the physical
Double Shift, delegates the exact IME tail, then rejects its focused-window
lease with `exact focused-window identity requires GNOME`.
Source: `src/bin/lay_daemon/layout_controller.rs:154`; first failure is exact
focus ownership, before replay dispatch. Preserve this guard. A test-only KWin
observer is not production focus authority and must never be substituted into
the runtime to force PASS. Some other rows fail while checking an uncommitted
preedit source; retain the strict raw verdict and classify that observation
boundary before attributing every failed row to product logic.

Qt and subsequent native clients are still running sequentially. No KDE full
matrix or tray-menu verdict exists yet. The original complete browser matrix
remains Firefox 176/20/20, Chrome 178/25/13 (PASS/FAIL/BLOCKED). Source proof
remains 3219 PASS. No runtime authority, route, model, installed local version,
or resource ceiling changed.

### 2026-10-18T04:32:35Z — KDE completed widget/Chrome evidence and explicit gaps

The same private candidate completed these genuine Plasma/Wayland grids:
GTK 32 cells: 8 PASS / 22 FAIL / 2 BLOCKED; Qt 48 cells: 12 PASS / 33 FAIL /
3 BLOCKED; Chrome 160 cells: 21 PASS / 132 FAIL / 7 BLOCKED. Raw receipts are
`td156-kde-native-20261018/native-windows-{4qbgq4t1,09f1gu1e,tfmyvia8}/RESULT.json`
on the remote worker; exact copies are in local
`/home/ubu/.cache/lay/development/browser-final-fixes-20261018/kde-native/`
as `gtk.json`, `qt.json`, and `chrome.json`. These are a separate KDE grid,
not replacements for the original GNOME 216-cell browser matrices.

KWrite's `native-windows-vyvg_pky/RESULT.json` contains one accessibility-setup
BLOCKED and no functional cells. Konsole's `native-windows-y29e61gj/RESULT.json`
aborted before keys with `owned field not focused/showing`. Their 16-cell grids
remain NOT_TESTED. Do not count a setup verdict as 16 client acceptances.

Chrome's 132 raw FAIL include 45 failed empty-field prerequisites: rich editors
expose a newline after clearing. Those cells did not exercise their intended
operation. Other strict prerequisite failures include preedit/caret observation.
Keep raw FAIL; neither normalize a newline into empty nor call all 132 failures
independent product defects. The confirmed daemon refusal remains the
GNOME-only exact focused-window lease at `layout_controller.rs:154`, before
replay. The test KWin observer reports Wayland `windowId` zero, so that test
field cannot become a production window identity. PID alone is insufficient.

Successful measured Space operations: GTK 6, median 15.51 ms, p95/max 16.95 ms;
Qt 9, median 15.90 ms, p95/max 17.44 ms; Chrome 14, median 15.29 ms,
p95/max 17.34 ms. Scope is evdev-write return to exact observed native target;
these measurements exclude refused, failed and unavailable operations and do
not establish physical rendering or Double Shift latency on KDE.

The production PyQt6 tray registered its actual StatusNotifierItem. The separate
menu observer returned BLOCKED because actual SHOWING menu labels were not
available. Receipt: `td156-kde-native-20261018/batch-2-chrome-menu/MENU.json`.
It sent no keys and invoked no update/download/install action. Registration
alone is not proof that Settings/Check for Updates/toggle are visibly usable.

The first Firefox attempt aborted after 14 key events because its pinned IME
process disappeared. The code/signal was not captured, so its cause remains
unknown. Raw evidence is `native-windows-i0whl6wq/RESULT.json` and
`attempt-browser-ime-exited/`. No OOM/panic conclusion is inferred. A subsequent
zero-key setup failed on an existing private Firefox profile; that failure and
scope closure remain in `attempt-stale-firefox-profile/`. Profiles are now
archived before a fresh private Firefox session. The single current diagnostic
captures the unchanged engine's stderr and exit code in `logs/ime-process.log`
and `logs/ime-exit.jsonl`; it does not trace keys, change the runtime protocol,
or extend the finite seat. Earlier complete GTK/Qt/Chrome receipts are retained.

Completed widget and Chrome/menu GUI scopes have explicit empty-cgroup closures
in `batch-1-native-widgets/CLOSED.json` and `batch-2-chrome-menu/CLOSED.json`.
Chrome's remaining wrapper/engine/model descendants were terminated only after
its actors finished, with exact UID/cgroup/process identity checks. Its recorded
engine return code -15 is intentional cleanup, not the unexplained Firefox
exit. The temporary chvt setup permission for every session was removed before
input. No local installation, production route, runtime authority, model,
resource ceiling or source bytes changed. Full KDE acceptance is still pending.


### 2026-10-18T04:40:53Z — completed finite Plasma baseline and closure

The final unchanged-byte Firefox diagnostic completed all 160 cells: **28 PASS /
125 FAIL / 7 BLOCKED**, no actor errors and no held keys. No unexpected IME
exit recurred in this diagnostic. The earlier 14-key Firefox disappearance
remains unexplained and is retained separately; this successful process lifetime
does not establish its cause or universal crash freedom. After all actors had
finished, the exact remaining wrapper and engine were checked by UID, start tick,
executable hash and owned cgroup before intentional SIGTERM. The recorded engine
return code -15 belongs to that cleanup, not to a spontaneous failure.

Final KDE-native grid: **69 PASS / 312 FAIL / 19 BLOCKED / 32 NOT_TESTED = 432**;
400 observed cells. GTK3 contributes 8/22/2 of 32; Qt5 12/33/3 of 48; Firefox
28/125/7 of 160; Chrome 21/132/7 of 160. KWrite and Konsole each retain 16
NOT_TESTED cells. KWrite's separate accessibility-setup BLOCKED receipt and
Konsole's zero-key setup abort do not fill their functional grids. Production
tray acceptance is separately BLOCKED: actual StatusNotifierItem registration
was observed, but SHOWING menu labels were unavailable. No menu action was
invoked. These are test-environment and client-specific evidence limits, not
accepted KDE functionality.

Keep every raw failure. In each browser, 45 FAIL occur while checking exact empty
rich-editor prerequisites that expose a newline instead; the intended operation
was not exercised in those cells. Remaining FAIL must be diagnosed from their
first transition, not assigned wholesale to a single defect. Confirmed product
Double Shift refusals reach the GNOME-only exact focused-window lease in
`src/bin/lay_daemon/layout_controller.rs:154`, before replay. The existing guard
must remain. A production KWin identity source within that same focus contract
is required for KDE Double Shift acceptance; the test observer, PID or a zero
Wayland window ID is not an equivalent identity.

Successful Space-operation timing (evdev write return to exact native observed
target, successful samples only): GTK 6 samples, median 15.51 ms / p95 16.95 ms /
max 16.95 ms; Qt 9, 15.90 / 17.44 / 17.44 ms; Firefox 21, 15.15 / 17.03 /
17.08 ms; Chrome 14, 15.29 / 17.33 / 17.33 ms. Failed, missing and refused
operations remain in the 432-cell grid. There is no successful KDE Double Shift
latency result and no overall acceleration claim. This VKMS/QPainter Wayland
baseline does not certify X11 or hardware rendering. The host IBus 1.5.26
compatibility limitation remains open; the successful native setup used the
recorded private, matching host-built upstream IBus 1.5.34-rc2.

Exact final receipts: remote
`/home/e/projects/lay-development-runner/td156-kde-native-20261018/RESULT.json`
and `CLOSED.json`; Firefox full receipt `native-windows-uy___xxg/RESULT.json`.
Verified local copies and file hashes are under
`/home/ubu/.cache/lay/development/browser-final-fixes-20261018/kde-native/`,
with `FINAL-FETCH.json`. Earlier attempts and all raw matrices are retained.
Final closure at 2026-10-18T04:37:20.434989+00:00 proves empty owned cgroup,
all pinned processes gone, no held keys, VT1 restored and the finite test seat
stopped. Temporary setup authorization is absent. No local runtime installation,
restart, production input-route, model, resource ceiling, source-byte or runtime
authority change was made by this KDE baseline. The original complete browser
matrix remains 354 PASS / 45 FAIL / 33 BLOCKED; source closure remains 3219 PASS.
KDE installation/testing is recorded; TD-156 browser repair remains IN PROGRESS.


### 2026-10-18T05:07:00Z — Chrome document diagnostic and completed KDE editor coverage

The finite six-probe Chrome Search diagnostic is complete and closed. All input
was native evdev under the existing identity, foreground and input-ledger guards;
a read-only fixture mirror observed the document value/caret. Search field 2
exposed stale AT-SPI text after Backspace both after Double Shift plus Space
(`ghbdtn z`, caret 7 versus document `ghbdtn `, selection 7/7) and in the
no-Double-Shift control (`max z`, caret 4 versus document `max `, selection 4/4).
The document samples were still correct about one second after the action. This
supports an accessibility-observation defect for those two diagnostic failures,
not a need to inject another Backspace or change IBus delivery. It is not new
native acceptance and does not turn the original five Search FAIL into PASS.

The mirror itself has a sampling limitation: it accepts a frame newer than the
pre-Backspace timestamp without a causal event barrier. Two text-field controls
were native PASS but mirror FAIL because their sampled value still contained the
probe letter. Do not count the six document verdicts as an acceptance matrix.
Before another document diagnostic, bind the sample to the actual event or a
bounded stable post-action observation; never alter document contents to obtain
PASS. Actor receipt:
`firefox-legacy-native-20261012.6onP5c/chrome-functional-iosshji1/RESULT.json`,
SHA-256 `a0252708bb15249b32efd5b3cd1e96b27802b2dd15ad8a24cc4e9821204ec9c2`.
Local verified evidence: `browser-final-fixes-20261018/search-document/`
(`actor-result.json`, `RESULT.json`, `CLOSED.json`). Closure at
2026-10-18T04:51:12.976385+00:00: owned cgroup empty, pinned processes gone,
parent released-key list empty, VT1 restored, finite seat stopped. The actor has
no independent `no_held_keys` field; do not invent that assertion.

The two untested KDE editors now have complete, separate native receipts.
KWrite failed its first launch because installed version 21.12.3 rejects `--new`.
Konsole's original selector could choose a hidden editable node ahead of the
visible terminal. Zero-key discovery found one SHOWING Text editor in KWrite
and one SHOWING Terminal in Konsole. KWrite was already FOCUSED despite
`grab_focus()` returning false. The follow-up verifies actual FOCUSED/SHOWING
states, foreground, non-zombie process state, UID, start tick, executable and
owned cgroup before native keys. No production focus or delivery rule changed.
Helper setup failures (Python standard-module shadowing, unsupported AT-SPI
cache method, and exclusive log reuse on reopening an app) occurred before
functional input and are retained in the follow-up attempt logs. The successful
finite actor executes each planned cell once; these are not retries until green.

KWrite: **3 PASS / 11 FAIL / 2 BLOCKED of 16**. Native Backspace and clean
Russian/English preservation passed. Konsole: **3 PASS / 10 FAIL / 3 BLOCKED
of 16**, with the same three passing plans. Missing second field and actual
completion panel are BLOCKED; raw PTY selection is additionally unavailable in
Konsole. Double Shift targets, automatic Space conversion and moved-boundary
correction did not pass. Both actors completed with no errors and no held keys.
Successful measured Space samples only: KWrite n=2, median 16.01 ms, p95/max
17.80 ms; Konsole n=2, median 14.44 ms, p95/max 14.64 ms. These timings certify
clean Space preservation, not correction or Double Shift speed. Konsole's PTY
receipt proves consumed committed bytes; visual preedit is NOT_TESTED.

In these two editors, the first observed manual refusal is
`ibus_manual_toggle_rpc/not_handled/context_authority` (15 recorded refusals);
the daemon then reports `physical manual trigger blocked by focused IME owner`.
Surrounding-text capability is present (caps 41). This is earlier than the
GNOME-only exact-window lease seen in the previous GTK diagnostic. Do not assign
all KDE failures to that later lease or bypass the focused owner. A subsequent
KDE repair must establish the first missing context-admission proof and supply
production KWin focus identity through existing contracts. Test-only workspace
observations cannot authorize production edits or replay.

Cumulative same-byte KDE functional grid, across the recorded finite sessions:
**75 PASS / 333 FAIL / 24 BLOCKED / 0 NOT_TESTED = 432**. This is not one
simultaneous run or KDE acceptance. GTK32 stays 8/22/2; Qt48 12/33/3; Firefox160
28/125/7; Chrome160 21/132/7; the editor receipts supply the original 32
NOT_TESTED cells. Preserve the first 69/312/19/32 baseline unchanged. The 90
browser empty-rich-editor prerequisite failures remain raw FAIL; intended
operations were not reached. The visible production menu remains separately
BLOCKED; X11, hardware rendering and terminal visual preedit are not certified.

Remote editor run:
`/home/e/projects/lay-development-runner/td156-kde-editor-followup-20261018/`.
KWrite `native-windows-ypr3vcyz/RESULT.json`, SHA-256
`1fb288344c829fbbb866d59d5998eabc3d6980c883507c4efaa8a55ab1ce20fc`;
Konsole `native-windows-_y9s11lm/RESULT.json`, SHA-256
`94a7f8e394b5da5a6e130a35aa0594ace16275e79256333d950c5e0f04da8f25`.
Verified local copies: `browser-final-fixes-20261018/kde-editors/evidence/`
with `FETCH.json` and `VERIFIED.json` (15 files, 612865 bytes), plus the parent
`COMBINED.json`. All six app receipts have identical candidate metadata.
Final editor closure at 2026-10-18T05:05:35.757637+00:00 verifies empty owned
scope, pinned processes gone, actor and parent key cleanup, VT1 restored,
finite seat stopped and temporary setup authorization absent. The initial
closure helper treated expected `systemctl is-active` exit 3 as an exception
after stopping the seat; the final readback independently confirms closure.

No production Rust bytes, delivery route, runtime authority, model or local
installation changed during these diagnostics. Main TD-156 remains IN PROGRESS;
original complete GNOME/native browser matrix stays **354 PASS / 45 FAIL /
33 BLOCKED**, and the unchanged source proof remains **3219 PASS**. Next work
is the recorded browser first-transition diagnosis; the KDE capability gaps
are explicit and must not be hidden by treating harness corrections as product
fixes or changing a refusal to PASS.

### 2026-10-18T05:24:00Z — Focus-return proof and rich-editor seed preflight

Six additional native focus-return cells (Firefox and Chrome, fields 1/7/8)
now follow the existing protocol's explicit End after returning focus. Every
cell observes exact `привет`, caret 6, RU, then exact `приветы`, caret 7,
then exact Backspace return to `привет`, caret 6. Every first Double Shift
still fails with `context_authority`. Rich fields originally returned caret 0;
End fixes that setup defect, not the missing whole-word admission proof.
The rejected surrounding receipt covers seven characters while the owned edit
lineage covers only the newly typed character. Same IBus context, same text or
an end cursor cannot establish identity of a browser's focused DOM editor.
No refusal, delivery route or admission guard was relaxed.

Receipts: remote `firefox-functional-5xl75b0m/RESULT.json` and
`chrome-functional-wa46_sq9/RESULT.json` under the existing 6onP5c run.
Verified local evidence: `browser-final-fixes-20261018/focus-end/evidence/`,
16 files; archive SHA-256
`363d92d6edeade7adc0b77f12f43394c51e9ccd00fc8c28c93faa143639e7232`.
Both finite sessions closed with empty owned scopes, pinned processes gone,
no parent-released keys, VT1 restored and seat units stopped at
05:18:03.127907Z and 05:18:33.827325Z. These six FAIL are supplemental;
the complete native matrix remains 354 PASS / 45 FAIL / 33 BLOCKED.

The next first-transition group is nine Firefox setup BLOCKED cells:
prefix, moved-boundary and selection in rich fields 7/8/10. Their entire native
seed is correct except U+00A0 at physical Space positions. The old actor uses
byte-exact ASCII seeds, so the intended operation is never reached.

Chosen minimal test-only change: a shared pure full-field seed predicate permits
U+00A0 only where an expected U+0020 was physically typed, only for rich fields.
Require exact character count, caret at full-field end and no selection.
Preserve the returned raw seed. Subsequent prefix conversions must keep its
actual separator bytes, and selection replacement must first prove full bounds.
No trimming, general whitespace normalization, hidden suffix, interior caret,
changed letter/punctuation or additional selection is accepted. Plain fields
remain byte-exact. Unit negative cases prove these rejection boundaries.

Alternatives: retaining ASCII-only checks leaves valid rich seeds BLOCKED;
normalizing all later targets could hide an altered supposedly immutable prefix.
The chosen predicate changes setup observation only, before measured actions.
Native action cadence, release clocks, exact subsequent targets, focus/identity
guards, original receipts and 432-cell denominator remain unchanged. A separate
finite 9-cell series per browser will exercise this one harness change once,
with Chrome as control; it cannot overwrite the old full receipt.

Consequence scope: no production Rust/model/package, candidate retention,
ranking, false authority, cache, reload, learning, concurrency or failure route
changes. No new runtime owner, fallback, timer or allocation is introduced.
The test predicate is O(seed length) before timing; source-only PASS will not
certify native behavior. Rollback removes the helper and supplemental actor;
negative historic evidence is retained. Runtime authority remains unchanged.
Decision: `2026-10-18-native-rich-seed-space-observation.json`.

### 2026-10-18T05:31:15Z — Rich seed series complete; first pair-scope loss identified

The shared seed helper's 11 guarded remote tests pass, including malformed,
selection, interior-caret, hidden-suffix and non-Space Unicode negatives.
The compiled supplemental actor retains identical ASTs for type_text, target,
action, probe, roundtrip, boundary_seed and functional_metrics. Only seed
observation, raw prefix retention and full selection bounds change. SCOPE.json
is a zero-key post-run check; the first attempted stdin-fed guarded check did
not execute its Python body and is not counted as evidence.

One finite nine-cell native series per browser is complete. Firefox: **6 PASS /
3 FAIL / 0 BLOCKED**; prefix conversion and selection pass in fields 7/8/10,
with raw prefix bytes preserved. All three moved-boundary corrections fail at
Space. Chrome control: **9 PASS / 0 FAIL / 0 BLOCKED**. These are supplemental
cells, not a replacement full native matrix. Preserve the old nine Firefox
BLOCKED cells and the complete **354 PASS / 45 FAIL / 33 BLOCKED** receipt.

The surviving Firefox first mechanism is bounded more narrowly than a general
correction/ranking failure. For the recorded rich owned-preedit case, the shared
core publishes Apply, an eligible selected candidate and a transition receipt;
the adapter returns no decision at verifier stage. Source inspection identifies
an earlier missing physical scope: capture_space_boundary_pair_scope uses
rsplit_once(' ') and an ASCII-only left-prefix witness. A committed first word
rendered with U+00A0 cannot produce the exact editable-pair scope. The downstream
single-token adapter therefore correctly refuses a change to the previous word.
plan_space_boundary_edit is also ASCII-only. Do not relax that verifier to
compensate for missing raw physical scope or replace a prefix using normalized
bytes. No production fix has been made for this newly exposed mechanism.

There is a second boundary to prove before implementation: after committing the
owned second-word preedit, Firefox may render the old internal NBSP as ASCII
Space and the new trailing Space as NBSP. The failed raw surface is
`должн ыбыть\u00a0`, whereas the seed was `должн\u00a0ыбыть`.
Supporting the initial pair parser alone cannot certify the exact subsequent
commit acknowledgement, edit receipt or inverse. A production design must keep
the same owner and delivery path while binding the actual full raw source at
that transition; broad whitespace normalization is not an acceptable shortcut.

Options for that next implementation preflight: (9/10) teach the existing pair
scope and decision projection about a single observed physical-Space separator,
with exact revalidation after client canonicalization and negative stale-owner,
selection, suffix and extra-space cases; (2/10) normalize arbitrary runtime text,
which loses byte-bound proof; (3/10) force a different rich-client commit/replay
route, which changes delivery instead of repairing the missing scope. The first
option is recommended, but its post-commit proof remains unresolved; production
editing stays deferred until that consequence is bounded. No new owner, bypass,
model exception or weakened authority is justified by these fixtures.

Supplemental successful evdev-write-return to exact AT-SPI timings only:
Firefox Double Shift n=12, median 34.39 ms / p95=max 72.00 ms; selection n=3,
18.38 / 19.29 ms. Firefox Space correction has zero successful samples.
Chrome Double Shift n=12, 49.04 / 71.00 ms; selection n=3, 18.67 / 21.49 ms;
Space correction n=3, 19.78 / 20.17 ms. These small successful-only samples
are not a matched performance comparison or kernel-release timing.

Receipts under the existing remote 6onP5c run: Firefox
`firefox-functional-j0gbnv1n/RESULT.json`, SHA-256
`cc60df7edf696c6dcc0672f09be639e52e509b73d3f10efac5aa2d2add7226c6`;
Chrome `chrome-functional-lzu_bedy/RESULT.json`, SHA-256
`1592ea4314a8e0f5ef9fefa64f18978edfe6c1d1949f2a746f56ff97a94aee90`.
Local evidence: `browser-final-fixes-20261018/seed-space/evidence/` (20 verified
files), SUMMARY.json and VERIFIED.json; archive SHA-256
`4b65b15733d474e4b24e2c34ed3381fd39ee463b86db0aed0b16bee376b240c0`.
Both actors have no errors; parent released-key lists are empty. Closures at
05:26:24.722286Z and 05:27:14.519327Z verify empty scopes, pinned processes
gone, VT1 restored and finite seat units stopped. Core source proof stays
3219 PASS; the additional 11 tests cover only the test-observation helper.
No production Rust bytes, delivery route, runtime authority or local installation
changed in these focus/seed diagnostics. TD-156 remains IN PROGRESS.

### 2026-10-18T05:37:30Z — Bounded pair-presentation implementation preflight

The recommended scope repair is now specified in decision
`2026-10-18-observed-space-pair-presentation.json`. It keeps the existing owner,
core winner, safety test, pending client-commit receipt and output route. Bind the
actual whole-field snapshot at the existing continuation; accept ASCII/NBSP only
at the two already owned separator positions, with all other text and coordinates
exact. Preserve the observed whitespace signal in the replacement and record the
actual raw source in the sealed inverse. No string-wide normalization is allowed.
This is a projection of the existing decision, not permission to adopt unseen
text, repair candidate authority, change delivery or extend a deadline.

New regression `space_pair_owned_nbsp_commit_preserves_observed_surface_and_inverse`
starts from the exact client-observed committed NBSP prefix and processes the
second word through production printable/Space handling. It requires an exact
pair, no deletion before the second client
commit, one exact full-pair replacement after canonicalization, and exact raw undo.
Run it against the preceding implementation first; retain the failure as RED.
Production implementation and changed-byte native behavior remain NOT_TESTED at
this preflight. The full browser354PASS45FAIL33BLOCKED and earlier source3219PASS
remain historical receipts, not proof of this newly added regression.

### 2026-10-18T05:49:01Z — Exact NBSP pair regression RED and first GREEN

The initial first-word acknowledgement fixture did not reach the new mechanism:
its unit setup lacked the complete native admission/reset sequence. Its failure
in `nbsp-space/red-paseb8kw/RESULT.json` is a fixture failure, not root-cause RED.
The corrected fixture begins with the exact already observed `должн\u00a0`
prefix, types the second word using production key processing, and requires the
actual second Space acknowledgement before any deletion. The native receipt
above independently establishes the real first-word NBSP observation.

Controlled RED overlays only that corrected test module on frozen preceding
production archive `97fe78bb1b6350b99a2526d036eb0006e69602d9b6c0fbb8630b6d9981023455`;
all other 1717 archive members are hash-identical. Archive
`b5a9c5ea889c549bdd8fba258811fef6aa293cc00980e934c823a660941eba2f`
fails at the missing exact NBSP pair, before selection or mutation. Receipt:
`browser-final-fixes-20261018/nbsp-space/baseline-red-kjskwcbq/RESULT.json`,
remote `td156-nbsp-baseline-red-RGgBQf`. The original archive and receipts stay
unchanged.

An intermediate parser repair exposed a second ASCII-only parser in the existing
pair planner. Test-only full-lease diagnostics establish core `apply`, eligible
selected candidate, selected transition present, bound frame, and adapter
`Verifier` refusal. The core already preserves the actual NBSP in its selected
target. Therefore the final planner uses the same bounded pair parser for source
and target and passes the original strings and receipt through the original
SafetyGate; no initial target resealing or receipt-normalization rule is needed.
Intermediate failed trial: `nbsp-space/green-ng9bfhd3/RESULT.json` (archive
`965a7542f4dd0b2e8205c3f16d1e66934d90213d04aabab1a6df98523006bae8`).

First GREEN: one owned-NBSP regression passes, including no deletion before
acknowledgement, Firefox's internal NBSP-to-ASCII / final ASCII-to-NBSP
presentation, one exact replacement and exact raw recorded inverse. Receipt
`nbsp-space/green-himwgn7z/RESULT.json`, remote `td156-nbsp-green-0QYnWj`, archive
`3283ffe81798fef5b10637b6b511d4c65de5e6a6d92fc41b1fe25701d4e12baa`.
This is a source regression only. The broader pair/gate closure, changed-byte
native acceptance and speed remain pending. The original complete browser
354 PASS / 45 FAIL / 33 BLOCKED is unchanged. Production delivery, installed
runtime authority, models, ranker, learning and local installation are unchanged.

Focused follow-up at 05:49:38Z: all 22 production-owner pair tests and all five
sealed edit-gate tests pass on archive
`2c1df00c0d1ffcc3a92e3980ac813b914bd5d7bdb9a28a30a089ce992c0f9d81`.
This includes the new 14-mutation rejection test (external prefix, word, extra
space, tab, narrow NBSP, newline, suffix, selection, caret, owner, focus, epoch,
material, expiry), both worker/client completion orders, ASCII compatibility,
partial failure retirement, exact-once correction and recorded inverse. Receipt
`nbsp-space/green-cd6t3jlb/RESULT.json`, remote `td156-nbsp-green-gT5byC`.
Selected changed Rust files were formatted under the remote resource guard;
`nbsp-space/format-tdvmwosu/RESULT.json` records all nine before/after hashes.
Full source closure and changed-byte native behavior remain pending.

### 2026-10-18T06:02:41Z — Full NBSP closure rejects unrecorded protected-byte successor

Remote full run `run-PNDFPe`, local receipt
`/home/ubu/.cache/lay/development/run-mcm5gdsg/RESULT.json`, archive
`d363d36e5f264882e0543766d1390dc047ce724ae71a83377d8bfb4b82510795`,
finishes FAIL after 509.5s. The single reported source-contract failure is
`td113_unsuperseded_protected_artifacts_match_the_v4_preflight_baseline`:
October8's immutable gate successor is
`547bc3797badb0b6b2e2dccc46ef8574a05aa760fe45363245283f4f75553d55`;
the formatted bounded NBSP implementation is
`6ed5eda0f5ad0a25c61a2c81a9cf7b33711db127fc9e217edf56c934a2fc286f`.
This is a valid protection refusal, not a complete source PASS or a native failure.
The original manifest, October7/October8 decisions and failed receipt remain intact.

The explicit October18 presentation decision now records its predecessor's exact
decision/hash/path/mode, its exact successor bytes, and root static review with
the focused27PASS receipt. The same existing test pins the immutable new decision
and validates the entire three-decision chain, actual successor hash and mode.
It retains every other protected-artifact assertion, all seven test identities
and the empty failure allowance. The decision is now immutable after this pin;
later validation belongs here. This records the changed byte boundary without
weakening SafetyGate, semantic proof or source protection. Run the seven affected
contracts first, then the full source gate again. Build/native input remain pending.

Protected-chain follow-up at 2026-10-18T06:03:41Z: all seven TD113 hybrid source contracts
PASS in 12.7s, local `run-6tdb482o/RESULT.json`, frozen archive
`e3c8635ec109770aa68bf1bea287c6d2669196e453eced76f67dc39e7c1f798a`. Exact gate bytes, permissions, immutable
predecessor decisions and all unrelated preserved artifacts are checked. This
focused result is not full correctness/package or native acceptance. Production
NBSP bytes are unchanged from focused27PASS; only the explicit successor chain
and its existing source-contract assertions were added after the full refusal.

### 2026-10-18T06:15:09Z — Bounded NBSP implementation: full source closure PASS

Fresh canonical remote run `run-dUQNEJ` passes all 3222 selected correctness/package
tests: 3186 correctness and 36 package tests, no reported semantic or infrastructure
failures. Worker time is 493.16s; the canonical launcher reports 501.6s overall.
Local immutable receipt `/home/ubu/.cache/lay/development/run-kf2jyn8d/RESULT.json`,
source archive `8688ea790ec8d4e3a545c133ed0210011cf792873ee03f084c1449a1b6343cf2`.
Formatting and the 122-test runner self-test also pass (one explicit real-cgroup
integration case remains skipped as declared by that suite). The original first
full FAIL and all protected predecessor/successor decisions remain unchanged.

This closes source verification for the bounded raw pair implementation, not
native acceptance or a release. The guarded target occupies 7,084,486,656 bytes
of its 12,884,901,888-byte budget. Next: build the frozen verified inputs, stage
only the private remote candidate, then run the separate nine-cell raw-rich
series and unchanged complete 216-cell protocol per browser. Complete preceding
native counts remain Firefox176/20/20, Chrome178/25/13, total354/45/33. No local
installation, model/ranker change, new runtime owner or delivery route.

### 2026-10-18T06:18:32Z — Release preflight rejects incomplete test-decision metadata

The first NBSP build stage `td156-nbsp-build-20261018` stops before release
compilation. Its actual changed/added-file canon check rejects
`2026-10-18-native-rich-seed-space-observation.json`: the test-only decision used
`problem/change/proof` instead of the required `reason/decision/verification`
and omitted the invariant/path lists and explicit not-tested scope. The earlier
canon unit suite, 50-pass audit, lint and security checks had completed; they do
not override this valid refusal. Exact negative evidence is retained in
`/home/ubu/.cache/lay/development/browser-final-fixes-20261018/build-nbsp/BUILD-RUN.log`
and its unchanged remote stage. No BUILD.json was issued.

Repair only that unpinned test-decision schema, preserving its prior measured
results and supplemental-only scope. No Rust, test actor, expected surface,
failure allowance, immutable protected decision or verifier changes. Reuse the
3222-test source receipt only after exact executable/test/package byte parity;
use a separate build stage and run the actual inventory canon check first.
New native input is still pending. Runtime authority remains unchanged.


### 2026-10-18T06:23:30Z — NBSP private release built, native phase started

The second, distinct private build passed the strict actual changed/added-file
architecture inventory, 15 canon tests, the 50-pass audit, architecture/lint
checks and guarded offline release compilation. The preceding build's ADR
schema refusal remains preserved; its only repair was test-only ADR metadata.
No Rust bytes changed after the 3222-test source closure. The new frozen source
archive is `f4e9ac774848485d3856443cfb49656cecdda870a0fd86b72dd63d0c72e10648`.
Exact build receipts: local `browser-final-fixes-20261018/build-nbsp-b/BUILD.json`,
`RESULT.json`, `CHECK.log` and `FINAL-AUDIT.json`; remote
`/home/e/projects/lay-development-runner/td156-nbsp-build-20261018-b/`.
Build status is BUILT_VERIFIED_SOURCE_NATIVE_PENDING, version 1.0.90,
227.359 seconds including preflight; source HEAD c8b244ef609ad9c5610b3e441311342abf17e9c5
plus the frozen dirty snapshot. Exact executable SHA-256:

- lay: `f8aec338504dadd72bd06fa78f4b3e8bd1863376897fce95dd3081e2d2e0c3c9`.
- daemon: `e6792f7cb165735e5052e48865490c96110973619c5bc5656d32e7c99de68cd7`.
- IME: `959d828c2d0557281ce8a30ee31689501538bb2e89f9506f5ecfa05b9ea05d35`.

Before staging, all old pinned native processes were dead, the owned cgroup
was empty and VT1 was active. The private harness preserved its old binaries,
extension, manifest, jobs, results and learning/usage data under
`TD156-nbsp-candidate-20261018`, restored the frozen isolated test data and
original double-lshift/undo-disabled configuration, then bound the three new
executables and matching extension. Local staging receipt:
`browser-final-fixes-20261018/nbsp-native/STAGED.json`.
Only this stopped private harness changed; no local installation or service
restart. Delivery routes, runtime authority and model bytes are unchanged.

The finite nine-cell Firefox rich-editor native phase has now started. Its
result, new-byte full browser matrices and speed measurements are PENDING.
The complete preceding-byte matrix remains Firefox 176P20F20B / Chrome
178P25F13B, total354P45F33B of432. KDE remains unaccepted with its separately
recorded cumulative75P333F24B of432; do not combine these denominators.


### 2026-10-18T06:25:54Z — NBSP changed-byte rich-editor native 18/18 PASS

The two finite supplemental native phases completed against the new private
IME `959d828c2d0557281ce8a30ee31689501538bb2e89f9506f5ecfa05b9ea05d35`:
fields7/8/10, prefix + moved_boundary + selection, nine cells per browser.
Firefox9PASS/0FAIL/0BLOCKED; Chrome9/0/0. Actor errors are empty. The unchanged
key/target functions were pinned by SCOPE.json; only the previously documented
bounded raw-rich seed observer admits the actual NBSP seed. No DOM setter,
normalization of runtime text, changed expected word or output-route bypass.
All three Firefox physical pair seeds are raw `должн\u00a0ыбыть`; final exact
observed targets are `должны быть\u00a0`. The preceding-byte supplemental result
was Firefox6P3F and Chrome9P, so the three reached Firefox pair failures now
pass while all previously passing supplemental cells remain PASS.

Exact primary receipts, also fetched and individually hash-verified:

- Firefox `firefox-functional-mwmo_m9n/RESULT.json`, SHA
  `c96063ee95d9f0122f9be359f33cbdee78d8573e72a101bc21a3147416cd6aa5`.
  Local archive `nbsp-native/evidence/firefox/evidence.tar.gz`, SHA
  `775866b66279e8c6d2c75185b8aa1a877e35e38f2e8062124544608705391e85`.
- Chrome `chrome-functional-j3c2_zmp/RESULT.json`, SHA
  `c8ba1779fe03d0fa595075a20e94206c565818cf300e8ae1a79e4c1b1b2fbcd7`.
  Local archive `nbsp-native/evidence/chrome/evidence.tar.gz`, SHA
  `ee6a42d197119a62fe8dbe3bf20880ca87f6495eaed17dae9b3fa80dc54e447e`.

Each archive has19 verified files including full actor/IME trace, candidate
binding, scope and CLOSED receipt. Both phases returned parent exit0 with no
held/released keys; all pinned processes dead, owned cgroup empty, seat stopped,
VT1 restored. Closed at06:24:01.100308Z and06:24:48.011451Z respectively.

Action-only physical-write-return to exact AT-SPI target latency (ms):
Firefox DoubleShift n12 median41.61/p9580.43, Space n3 median18.95/p9519.93,
selection n3 median18.87/p9521.06. Chrome DoubleShift n12 median50.22/p9575.50,
Space n3 median18.84/p9520.33, selection n3 median17.41/p9520.01. All attempted
actions measured; n3 p95 is the sample maximum. This supplemental series is
not a matched full-matrix speed claim or evidence for the unreached old failures.

Full original-protocol216/browser remains pending. Its first controller setup
was rejected before any session/input because `tdnbsp-full-firefox` violates
the existing two-word seat-label allowlist. Preserved original phase
`TD156-nbsp-native-20261018/full-firefox/FAILURE.json` records input_keys0,
empty scope, dead old pinned processes and VT1. No runtime defect inferred.
A distinct `run-full-b.py` changes only phase directory (`full-b-`) and valid
seat label (`tdnbspfull-`), not actors/key actions/targets. Controller SHA
`278747675701756462e475decf052d08003a8f25383aa11d26e1a613af43843b`;
local FULL-CONTROLLER-CORRECTION.json records the old/new binding. Its fresh
Firefox full phase has started once; no product retry-until-green.

Verdict: bounded NBSP pair mechanism has source3222PASS plus18/18 supplemental
native acceptance. No full-matrix, focus, undo-precondition, Search-observer or
KDE acceptance is inferred. Complete preceding matrix354P45F33B remains the
status denominator until the new full closure. Runtime authority unchanged;
no local installation, model change or delivery-route change.


### 2026-10-18T06:32:35Z — full-controller closure and next Search observation proof

The second full-controller refusal was before the first matrix job: the outer
controller and inner matrix independently used the same exclusive zero-key
receipt tag. Its original matrix.log, full-b-firefox/FAILURE.json and CLOSED.json
are retained; the outer receipt observed input_keys0, and matrix_jobs_queued0.
All pinned processes died, scope emptied, seat stopped and VT1 restored.
A separate full-c controller retains both zero-key checks with distinct unused
tags TDNBSPFULLC and TDNBSPMATRIXC, valid two-word seat labels and new phase
paths. Actors, original nine job payloads, expected surfaces, output routes and
limits are unchanged. Remote FULL-CONTROLLER-C-AUDIT.json at06:27:04.101619Z
checks both labels, all18 original job paths, absent new proof paths and all
candidate-pinned actor hashes before launch. New controller SHA
81d481ae039bac6a615eba18a8d3c1f7d13882c690287088a28d92f373ecfcca;
inner matrix SHA63e76a4035fff764e5434737f3722dce7e9b67cb6627290e205194d6bc07c4bb.
The first two original Firefox jobs now completed20PASS each; this40-cell partial
is not a replacement for the last complete354P45F33B matrix.

The remaining Chrome Search investigation is test observation, not a proposed
runtime edit. Existing six-cell DOM mirror diagnostics retain two actual
AT-SPI/DOM disagreements after Backspace (including a no-DS control), but two
other controls can sample a fresh animation frame before the key event. A new
frame timestamp alone cannot establish post-key causality.

Options: (1) accept any new DOM frame, 3/10, rejected because of the observed
race; (2) bounded read-only DOM event ledger plus a strictly newer trusted
keyup for the same field before evaluating its raw value/caret, 9/10, selected
for a diagnostic only; (3) alter browser accessibility/runtime delivery, 2/10
for this evidence gap, broader and without a proved need. If the trusted keyup
is not exposed, report BLOCKED; do not substitute timeout or expected text for
the event proof. Keep original AT-SPI snapshots and verdicts alongside the new
oracle, including no-Double-Shift clean controls. Do not relabel the five
original Search failures from these supplemental cells.

Planned before any new native input: pure negative controls reject fresh frame
without keyup, old sequence, wrong field/page, wrong/untrusted key and regressed
event time; pin unchanged original physical key functions and candidate bytes;
validate the finite controller/unique proof names. Then one six-cell Chrome
series, original active/Space/plain-English controls in Search and plain input,
after both full matrices close. No retry-until-green or extra input barrier.

Consequences: a bounded16-event private fixture ledger is read-only and never
writes field content or selection. No production owner, candidate lattice,
DecisionCore, verifier, callback deadline, cache/model generation, reload,
learning or native output contract changes. Its extra accessibility nodes make
this diagnostic unsuitable for a matched speed claim. Restore the original
fixture at the next setup; preserve the diagnostic/negative controls as evidence.
Main task IN PROGRESS; native complete/browser status and KDE acceptance unchanged.

### 2026-10-18T06:39:28Z — causal Search observer source audit, native pending

The finite diagnostic planned above is prepared under local
`browser-final-fixes-20261018/search-causal/` and remote
`TD156-search-causal-20261018/`. `SCOPE.json` reports
SOURCE_SCOPE_PASS_NATIVE_PENDING: all ten pure causal-observation controls pass,
the read-only JavaScript passes `node --check`, and the expanded original actor
compiles. Its protected physical key, Double Shift, target, decoder, field and
runtime guard functions are AST-identical to the pinned original. The six cells
retain the original native AT-SPI verdict separately from the post-trusted-keyup
raw DOM value/caret verdict. Missing causal evidence remains BLOCKED.

Audit binds expanded actor SHA
`ddf90f99b5543a18a5fd842d7e2f8b5d4a9408e1ca5aeed2bc81db30935c0014`,
controller `9b79e23b3fc6880ca030b5e61b7dc95c1170a11dfd6a39295b41ea4700ff07a9`
and probe `b80e8e2093cf1e1de1fce4a7e6b60aa700ec74a01d85a24551e6a34fab991ff7`.
Unique phase/zero-key paths are unused and the seat label satisfies the existing
allowlist. Source audit used zero native keys. It has not run against Chrome;
both original full matrices must close before its one finite native series.
No runtime, model, learning, delivery-route or authority change follows from
this test-observer audit. The new-byte Firefox full matrix is still partial;
the last complete browser status remains preceding-byte354P45F33B of432.

### 2026-10-18T06:44:49Z — new-byte Firefox original216 closed; one regression retained

The unchanged original nine-job Firefox matrix completed176PASS/21FAIL/19BLOCKED,
zero NOT_TESTED. All216 cell identities match the preceding receipt. Exact
verified local primary index is
`browser-final-fixes-20261018/nbsp-native/evidence/full-c-firefox/SUMMARY.json`;
39 fetched files/archive SHA
`24ca4a4e6319889983747ebc35764a4623afa926d11ed2c0d49025efdd9c9251`.
`phase/CLOSED.json` records06:40:16.122595Z, every pinned process gone,
empty owned cgroup, no parent-released keys, VT1 and stopped owned seat.
Candidate/config/zero-key bindings and unchanged original protocol are verified.
The separate18/18 rich-seed supplemental acceptance is not added to216.

Two cell transitions require distinct treatment: functional field9/base changes
BLOCKED to PASS after its visibility precondition is reached; field4/short
changes PASS to FAIL at the first Double Shift (`а` remains `а`, expected `f`).
Do not claim the former as a production repair or discard the latter as noise.
The daemon observes the physical double gesture and reports it handled by the
focused IME. The group's capped512KiB IME trace starts later than this cell;
the engine-side first refusal/effect is UNKNOWN, not inferred from this gap.
The other failures are10 Space targets before undo and10 focus-return failures.
Blocked cells are10 unavailable fixed Tab completions and9 byte-exact rich seeds.

Fixed timing denominators, physical-write-return to exact AT-SPI target, ms:
standard420/420 median98.73/p95111.96; panel48/48 median101.04/p95109.91;
functional Double Shift146/210 median96.32/p95128.56 (64missing);
Space37/50 median18.21/p9523.44 (13missing); Tab0/10;
selection7/10 median18.21/p9521.50. These are not kernel-release or all-attempt
latency claims. The original Chrome216 phase started only after verified Firefox
closure; its result remains pending. The last complete two-browser table still
belongs to preceding bytes354P45F33B/432.

Next finite diagnostic, after Chrome and the six-cell Search series: exactly the
two original email field4 plans base+short, unchanged candidate and full original
plan/key/target bodies. A small group retains the first failure inside the capped
trace. `TD156-short-email-20261018/SCOPE.json` reports zero keys,
SOURCE_SCOPE_PASS_NATIVE_PENDING, all original functions AST-identical and body
SHA unchanged. Expanded actor4a4035756a9ed18ed6bc95a3960ee67b67b5c7c307ff18fa629cf353078950cf;
controller3042cd8adfc2ebf6fe8ea163b0b589a63c568dd168ff23db154336576273df99.
Unique phase/job/proof/valid seat names are pinned. The controller restores the
pristine fixture before the two cells; the preceding read-only DOM diagnostic
must not contaminate this observation. This is one diagnostic, not retry-until-
green or a replacement for the failed original matrix cell.

Verdict: whole new candidate native acceptance is FAIL; the bounded NBSP mechanism
has its separate3222-test source proof and18/18 supplemental native proof. No
production edit, authority relaxation, model change or local installation follows
from the new refusal. TD156 remains IN PROGRESS.

### 2026-10-18T06:54:10Z — new-byte original Chrome216 closed; complete432 retained

The unchanged original Chrome matrix is COMPLETE178PASS/25FAIL/13BLOCKED,
zero NOT_TESTED. All216 cell identities and verdicts match the preceding
auxiliary-byte Chrome matrix. Verified local index:
`browser-final-fixes-20261018/nbsp-native/evidence/full-c-chrome/SUMMARY.json`.
All39 collected files are SHA-verified; archive
`0a00b2e178a3b2418d77f21c2f4ccc2118478d3438a2785ac92b31b6470412fa`.
Closure06:53:35.410743Z records the owned cgroup empty, pinned processes gone,
no held keys, restored VT1 and stopped owned seat. The frozen configuration,
zero-key proof and candidate identity match the new NBSP build archive
`f4e9ac774848485d3856443cfb49656cecdda870a0fd86b72dd63d0c72e10648`.

The latest complete same-byte browser table is now Firefox176/21/19 and
Chrome178/25/13, total354PASS/46FAIL/32BLOCKED of432, zero NOT_TESTED.
The historical preceding-byte354/45/33 table is not overwritten. The additional
18/18 rich-editor NBSP supplement remains separate from the original432.
Whole-candidate native acceptance remains FAIL, including the newly observed
Firefox email short-word regression.

Chrome failure reasons: two strict standard Backspace returns, three functional
Backspace returns, ten Space targets before undo, eight exact focus-return
Double Shift targets and two focus-return new-letter targets. BLOCKED retains
ten unavailable fixed Tab completions and three exact Search seed failures.
No failure is relabeled from this aggregation.

Fixed timing coverage, write-return to exact AT-SPI target, ms: standard408/420
median83.05/p9599.82/max109.90; panel48/48 median82.24/p9597.76/max110.48;
functional Double Shift156/210 median81.27/p95114.73/max125.96;
Space39/50 median19.57/p9524.13/max25.30; Tab0/10;
selection9/10 median19.99/p9522.03/max22.03. Missing target samples stay in the
required denominator; no full-speed improvement claim follows.

Only after verified original closure, the already source-audited six-cell
Chrome Search causal diagnostic started in its unique owned seat. It keeps
native AT-SPI and post-trusted-keyup raw DOM verdicts separate. The two original
Firefox email plans remain pending behind that series. No runtime code,
delivery route, model, learning data or local installation changed.

### 2026-10-18T06:55:43Z — causal Search oracle remains BLOCKED, no native promotion

The six-cell finite Chrome diagnostic completed its controller with six BLOCKED
cells: both Search and plain-text controls, active/Space/clean-English, fail
the mandatory post-letter trusted-keyup evidence barrier. No Backspace oracle
verdict was reached. The native letter target was observed, but no retained
post-letter DOM sample establishes why the keyup predicate could not be met.
Do not infer a browser keyup defect, an IME text defect or a successful return
from this evidence gap. In particular, no native/document PASS comparison was
established and the five original native Search FAIL cells remain FAIL.

Exact primary is `chrome-functional-mwh4kqp6/RESULT.json` under the pinned
remote runner. Local verified20-file bundle is
`browser-final-fixes-20261018/search-causal/evidence/chrome/`; archive
`98f418a4b8dabf81244d88d1065730680d35e0cbc9020b43aa7cb49e8c031080`.
Closure06:54:25.452007Z records empty owned cgroup, pinned processes gone,
VT1, stopped seat and no held keys. The ten source controls remain source proof
only. This supplement is not added to the original432 denominator.

The once-only two-cell Firefox email diagnostic started only after this closure;
its controller restores the pristine fixture. Runtime and model authority are
unchanged, with no local installation or delivery-route edit.

### 2026-10-18T06:57:36Z — original email control passes; Search keyup assumption disproved

The once-only original Firefox email base+short control completed2/2PASS,
including four exact Double Shift targets and four next-letter/Backspace
round trips. Primary `firefox-functional-r3apd5x7/RESULT.json` SHA
`4c0b4391ba44cbcaa890424c4ad2c841fa2a60d9b557565aa8b5dd7ddf1804a5`;
local17-file verified bundle
`browser-final-fixes-20261018/short-email/evidence/firefox/`, archive
`de7489492137a82d926802783150abd1f9235ab9431928390159e017ce6b8c89`.
Closure06:55:19.167891Z: owned cgroup empty, pinned processes gone,
VT1, stopped seat, no held keys. This control does not reproduce or erase the
full-matrix short-word FAIL. Its first engine-side failure remains UNKNOWN.
Do not rerun the same controls until green or attribute the failure to NBSP.

New causal evidence for the Search observation refusal: the retained capped
IME trace contains five Z `managed_release` events, all handled=true, following
one managed-commit and four legacy-preedit Z presses. The first cell is outside
the capped trace and is not inferred. Source
`ibus_interface.rs:867` explicitly consumes the paired release of a handled
press. Thus a mandatory DOM keyup is not a valid universal completion barrier
for these managed IME edits. This explains why even the plain control could
block; it does not establish the DOM's actual post-Backspace value.

Minimal next observation options: keep requiring a consumed keyup (3/10; cannot
observe this managed route), use a fresh trusted DOM input event on the same
page/field after the exact native letter target (9/10; selected), or use only
frame/time freshness (2/10; preserves the already rejected race). The selected
test-only successor must preserve all raw values and the original native
AT-SPI verdict, use a new namespace and a fixed six-cell limit, require a newer
event sequence/frame/time and trusted same-field `input`, and keep absent
events BLOCKED. Retain post-barrier refusal snapshots. Pure negative controls
must reject stale/other-field/untrusted/non-input events and must preserve a
wrong raw value as wrong. Existing six BLOCKED cells and original five FAIL
cells remain immutable. No production input, output, deadline, authority,
delivery-route or model change is authorized by this observer adjustment.

### 2026-10-18T07:01:49Z — causally fresh Search document proof separates stale AT-SPI

The selected read-only input-event successor passed ten pure source controls,
JavaScript syntax and unchanged protected native-key/target AST audit with zero
input keys. Expanded actor SHA
`d59a44cec3c70202b06414e6f4b1d1cf8296ec564fc21d2b54796a453619d979`;
controller `27a2a5e4ba30ff77b8c95ba16753fe7690fb2918bdbf17ec23a49bc0fb46da48`;
probe `6aeb17f521c31cf9948ba0740c8dce9fe70b670825445dfd74bab502816433aa`.
The JavaScript is byte-identical to the preceding read-only mirror; only the
test predicate now requires a newer trusted same-field `input`, rather than a
DOM keyup that this managed route consumes. No observed text is normalized.

Its once-only six-cell Chrome series is COMPLETE: native AT-SPI4PASS/2FAIL,
causally fresh raw document value/caret6PASS/0FAIL/0BLOCKED. Both failures are
Search after Space: exact raw DOM returns to `ghbdtn ` or `max ` with a collapsed
caret7 or4, while AT-SPI still exposes the deleted `z` after that same caret.
Both have a newer trusted same-field `input` with type insertCompositionText,
followed by compositionend, on the same pinned page. The native-letter target
and newer post-letter input proof precede physical Backspace; the post-Backspace
ledger sequence exceeds that exact post-letter sequence. The active Search
control and all three plain-input controls agree across both observers.
No frame-only freshness, injected DOM edit or expected-value substitution is used.

Exact primary `chrome-functional-ucf1lvbn/RESULT.json` SHA
`26485af60f87dbd49072c5151e56ae5eecac1e238794b02458fa439f31d08f6e`;
local verified20-file bundle
`browser-final-fixes-20261018/search-input-causal/evidence/chrome/`, archive
`d183924ae2dbb89fcd8ef68f3e75eae4b57ff245bc85f093b596fe7f5a951140`.
Closure07:00:15.623311Z: owned cgroup empty, pinned processes gone, no held keys,
VT1 and stopped owned seat. The supplemental overall verdict deliberately keeps
the two native FAIL cells. It does not relabel the original five Search failures,
cover their other window modes/directions, certify rendering/tray acceptance,
or supply comparable latency. The original browser432 stays354/46/32.

Measured conclusion: in these two current Search controls, the first divergence
is the accessibility representation of completed text; no faulty IME deletion
is shown by the actual field value. Do not alter the production delivery route
to compensate. The one original Firefox short regression remains unexplained
despite its separate2/2 successful control. Focus-return authority and Space
ranker admission remain distinct unfinished mechanisms. TD156 stays IN PROGRESS.

### 2026-10-18T07:07:48Z — bounded first-refusal capture for the unexplained email cell

The original Firefox field4 short-word FAIL remains open after the separate two-
cell control passed. Its capped end-of-group trace lost the first engine event.
The next experiment is one unchanged original two-field functional group,
fields3,4 and all15 original plans in their original order: exactly30 cells,
one invocation, no retry-until-green. This tests the preceding field lifecycle
as a possible trigger; that trigger is a hypothesis, not a measured cause.

Options: repeat the already successful two-cell control (2/10; little new
evidence), enlarge production logging and rebuild (4/10; changes candidate
bytes and timing), or copy the existing bounded trace immediately after the
target failure is saved (9/10; selected). The test-only save wrapper must call
the original save first, perform no extra work until field4/short is FAIL, and
copy raw trace bytes plus its existing offset/inode and an exact result snapshot
to exclusive new diagnostic files. Capture failure stays a diagnostic error;
it cannot change any cell verdict or trigger another input. The original native
key, target, plan, deadline and focus methods remain AST-identical.

Use a new owned seat, phase, job and zero-key-proof namespace, restore the
pristine fixture, retain current candidate/config identity and verify source
scope with zero input before launch. Collect every captured file with hashes
and verify closed cgroup/processes/keys/seat. A successful30-cell control will
not erase the original FAIL or establish its cause. Missing causal trace remains
UNKNOWN. No runtime authority, delivery route, model or local installation
change follows from this diagnostic. The original browser432 remains354/46/32.

### 2026-10-18T07:16:49Z — original two-field group closed; full-prefix capture selected

The first controller launch was a zero-input setup refusal: the outer resource
guard occupied the same lease required by launch-native's existing inner guard.
No keyboard or new native runtime was created. The controller's early-finally
path lacked a ready file; the exact pinned owned seat was separately closed,
VT1 restored and previous dead identity restored. Preserve
`short-group/setup-refusal/CLOSED-SETUP.json` at07:11:36.308982Z. Do not bypass
the inner resource guard. The distinct successor controller runs directly;
launch-native still owns the same bounded resource lease and scope.

Six pure capture controls PASS. Both namespace scope audits report zero keys,
unchanged original native functions/body and expanded actor SHA
`d77617534634677e2eeb4432537333a56b1019065d9b1fa6bcd8e4dc2ffd5869`.
The once-only native30-cell successor is COMPLETE24PASS/4FAIL/2BLOCKED. Both
short plans pass. Four failures are the existing two Space preconditions and
two focus-return targets; both unavailable Tab candidates remain BLOCKED.
No failure capture triggers. Primary `firefox-functional-t8tkv4ha/RESULT.json`
SHA `2cbf743ba5d8efcf65127422c368a4282e706e2d617b0aecb48f267dcb8c5d3a`;
verified18-file local bundle `short-group-b/evidence/firefox/`, archive
`a52bc3877194672639419e1684616a12f92fdeea8091dcb7eb338f32207b6d63`.
Closure07:13:18.513450Z records empty owned scope, pinned processes gone, no
held keys, VT1 and stopped seat. The original short FAIL remains unexplained.

Do not repeat either small successful control. The remaining diagnostic
hypothesis is accumulated state from the original full216 prefix (window modes,
panel and earlier functional groups), absent from both fresh-seat controls.
One finite original Firefox216 replay is selected solely to capture that first
refusal; no further replay follows a successful control without new evidence.
Keep all original nine queued job bodies byte-identical and every candidate,
fixture, native-key/target/deadline and zero-key guard unchanged. The existing
bounded controller wait may read only newly created original result files and
copy the existing trace on field4/short FAIL. Reuse the tested capture body;
no polling worker, runtime trace change or modified native actor. Missing or
partial result reads grant no verdict. Use an exclusive new namespace and
collect any captured raw files with hashes. This tests a lifecycle hypothesis,
not a model hypothesis or a repair. Successful replay cannot explain or erase
the earlier intermittent FAIL. No production route, owner or local-runtime
change is justified by these results.

### 2026-10-18T07:28:33Z — current published Tab supplement preflight

The fixed original Tab prerequisite requires document preedit `проверка` with
caret4. The current selected completion was observed as `проверить`, and the
new legacy publisher deliberately keeps an unaccepted hint outside document
composition. Thus those20 original BLOCKED cells cannot establish a Tab defect
or certify the new presentation. Keep their receipts and denominators.

Options for this acceptance gap: changing the old expected candidate/preedit
would hide a changed prerequisite (2/10); forcing a literal candidate in the
ranker would violate the semantic contract (1/10); a separately versioned native
supplement using the currently visible hint is the minimal suitable proof
(9/10). Select the latter without any runtime/model/route change.

Reuse the pinned native actor, finite session controller, physical key functions,
focus/identity/activity guards and unchanged deadlines. In one finite job per
browser, cover the ten existing fields in normal mode. First prove a fresh empty
field and no visible Cyrillic completion in the owned GNOME accessibility tree.
Type the existing prefix `пров`, require the exact native document prefix/caret,
then one unique, visible, geometrically nonempty current hint in that pinned
GNOME process. Derive the expected completed word from the actual displayed
suffix, rather than a hardcoded dictionary answer. Recheck source and hint
immediately before the one physical Tab; require the exact resulting word plus
one boundary, RU mode, hidden old hint, and native next-letter/Backspace return.
Reject missing/ambiguous/stale UI, selection, malformed/oversized suffix or a
changed prefix. No direct DOM edit, forced candidate or acceptance RPC.

This is a20-cell supplemental protocol, never a replacement432. Report Tab
write-return-to-exact-target median/p95/max and measured/required coverage, plus
all BLOCKED/FAIL. A standard screenshot-client attempt in the owned session may
collect one frame per browser; capture permission failure remains a separate
visual gap. Do not add a privileged Shell method or change delivery to obtain
pixels. Native accessibility visibility is not automatically pixel acceptance.
The helper is bounded read-only observation in the existing actor, with no
runtime cache, timer, retry, longer deadline, model reload or learning change.
Pure negative controls must reject false evidence before native input. Closure,
receipt hashes, runtime identity and all prior original failures remain required.

### 2026-10-18T07:34:56Z — full-prefix closure and independent NBSP review

The finite original Firefox216 replay completed on unchanged candidate IME
`959d828c2d0557281ce8a30ee31689501538bb2e89f9506f5ecfa05b9ea05d35`:
176PASS/21FAIL/19BLOCKED/0NOT_TESTED. Chrome's latest complete178/25/13 remains
unchanged, giving354/46/32 of432. The equal aggregate hides two cell transitions:
field4/short FAIL→PASS; field1/prefix PASS→FAIL on the reverse Double Shift.
That latter field retained `тест, ghbdtn`, caret12, instead of returning to
`тест, привет`; the first transformation and intervening letter/Backspace probe
passed. This is an intermittent failure, not a demonstrated repair or a reason
to select a greener receipt. Actual pinned field4 is **phone**, field5 is email;
earlier descriptions of the numeric field4 control as email were mislabeled.
The numeric receipts and jobs were not changed.

The passive observer correctly made no short-failure capture because that cell
passed. The first functional receipt's final trace is capped at512000bytes and
does not retain the newly failing prefix transition. Its async completeness
remains UNKNOWN. No causal attribution, new Rust change, acceptance promotion,
or identical repeat is justified by this run. Collected38files at
`/home/ubu/.cache/lay/development/browser-final-fixes-20261018/full-prefix-capture/evidence/firefox/`;
archive SHA-256 `ddc6fd32a76fa7f6e12b0beeb55dd33d5a9f2207cbe5785ffbdbb1452af9ef2b`.
Remote phase `TD156-full-prefix-capture-20261018/firefox/CLOSED.json` records
07:31:45.125221Z, empty owned cgroup, all pinned processes gone, no parent key
release, VT1 restored and exact owned seat stopped. Original job bodies,
targets, guards and deadlines remain byte-identical.

Independent read-only NBSP review, first pass: **9/10, no blocking findings**.
The reviewer confirmed the bounded two-slot rebind, unchanged sealed winner,
whole-snapshot/caret/owner/epoch/material guards, character-based counts, raw
inverse and unchanged single-word fallback. Existing3222 source and18/18 rich
supplement receipts are execution evidence, not executions by that reviewer.
Two gaps remain: extend the existing both-order regression with NBSP presentation
and a single-word control (P3); obtain physical inverse proof for the exact raw
NBSP pair. The rich supplement proves correction/selection, not that inverse.
KDE's75/333/24 of432 remains a preceding-byte separate platform result; no KDE
acceptance, local installation, delivery-route or runtime authority change.

### TD-156 — active-word probe cancellation source experiment, 2026-10-18T07:46:23Z

The closed full-prefix replay remains 176 PASS /21 FAIL /19 BLOCKED of216.
Its passive daemon log is now independently retained at
`/home/ubu/.cache/lay/development/browser-final-fixes-20261018/full-prefix-capture/daemon-causal/RESULT.json`:
483639 bytes,7038 lines,SHA256
`9b04fb4aebeea018494ad84e886811dea259fb9e588aea3580b071c43b954631`.
Lines4474–4485 record both physical Double Shift gestures: the first was handled
by the focused IME; after the control letter and Backspace, the second was
blocked by that owner. This excludes a missing physical gesture for this cell,
but the capped IME trace does not establish the exact reducer refusal.

Options: infer authority from equal visible text (1/10, violates the predecessor
contract); increase a native timeout (2/10, cannot establish missing authority);
extend the existing observed-callback regression to an active word with no final
separator (9/10, selected). The current regression covers only closed words.
The experiment adds empty-separator controls to the same real handoff/replay,
managed letter and ordinary Backspace chain, alongside its ASCII/NBSP controls.
It supplies no receipt after the local cancellation. A source failure must be
localized before any production edit; a source PASS only narrows the hypothesis
and does not erase the native intermittent failure. Runtime authority and
delivery routes are unchanged. Execution pending.

### TD-156 — current Tab qualification and active-word source result, 2026-10-18T07:54:28Z

The Tab source review completed its second and final pass at9/10, no practical
source blocker. Four evidence controls and four independent cleanup controls
passed under the remote resource guard (`published-tab/SOURCE-CONTROLS-v2.log`).
The native supplement then closed **0 PASS /0 FAIL /20 BLOCKED**:19 cells reached
the1000-node GNOME accessible-tree traversal limit before obtaining a hint;
Chrome field6 returned an AT-SPI application-lifetime error. Every action list
is empty: no acceptance Tab was sent and there is no Tab latency sample.
No selector or ownership rule was widened to obtain PASS. Original20Tab BLOCKED
cells remain unchanged; this failed evidence provider is not a Lay product FAIL.
Pixels remain NOT_VERIFIED. Next work here is a bounded non-cyclic visible-tree
observer or an existing trusted hint publisher, not a larger timeout or a literal
completion/ranker override.

Verified local receipts under
`/home/ubu/.cache/lay/development/browser-final-fixes-20261018/published-tab/evidence/`:
Firefox23files,archiveSHA256
`a7e48069d9a9c03dce93aa852baa3f31f8898a9bbcac96d771c7d97527e2d25f`,
CLOSED07:44:30.651840Z; Chrome23files,archiveSHA256
`4205c6d09fa5f94917cb1f32ed4c7cd6e2ad8659d4d20dd3d9e5d4b0c86fdbb1`,
CLOSED07:46:09.239673Z. Both scopes are empty, pinned processes gone,
parent-released keys empty, VT1 restored and exact owned seat stopped.

The initial source extension exposed two fixture assumptions: an active word
does not use the closed-word replay acknowledgement, and its continuation letter
emits `CommitText`, not owned `UpdatePreeditText`. Those two exploratory FAIL
receipts are retained (`active-probe/green-_l6_u_gw`, `green-b_rt0qbm`), not
reported as product RED. The old closed-preedit regression remains intact.
A dedicated active-native regression now models the actual commit plus ordinary
client Backspace. With empty and Unicode prefixes, and independently present or
absent letter/erasure receipts, **eight variants PASS**: a fresh actual erasure
receipt releases the exact reverse handoff in4/4; absence rejects it in4/4.
No receipt is invented, no deletion is emitted for the client Backspace, and
the exact raw word, current layout and tail are asserted after successful handoff.
Execution receipt:
`/home/ubu/.cache/lay/development/browser-final-fixes-20261018/active-probe/green-6zamm44h/RESULT.json`,
sourcearchive`6ff32ca6c74bb768c6b15a0a938bc3b4cf6709fca513f04140dbbb2855889a71`.
This establishes the source authority boundary; whether the original Firefox
failure lacked that exact receipt is still not proven by its capped trace.
Production bytes and delivery routes remain unchanged. The added semantic
regression needs the next full source closure after final formatting.

### TD-156 — raw pair inverse preflight, 2026-10-18T07:54:28Z

The remaining NBSP native gap is inverse delivery, not another correction proof.
Options: infer inverse from the adapter test (3/10, no physical proof); repeat the
whole432-cell matrix (2/10, does not add this missing operation); run six separate
inverse cells (9/10, selected). One original Space correction and one existing
Double Shift inverse in fields7,8,10 of each browser; no repeats or model changes.
The expected inverse is the **entire actual raw seed plus the actual terminal
boundary from the correction**, with exact whole-field/caret/no-selection/RU
checks and an exact letter/Backspace round trip. Only the established narrow
rich-editor Space seed observation is reused. The existing owned controller,
zero-key/runtime/focus guards, key functions and deadlines remain unchanged;
the reviewed independent seat/VT cleanup is retained. Prepared scenario:
`/home/ubu/.cache/lay/development/browser-final-fixes-20261018/pair-inverse/`.
Review and native execution pending. Original matrix cells are not replaced.


### TD-156 — finite raw-pair inverse result and qualification limits, 2026-10-18T08:02:32Z

The six-cell supplementary probe completed once: Firefox0P3F0B, Chrome0P3F0B.
These are raw diagnostic FAIL, not six independently established product defects
and not replacements for original matrix cells. Both first-field Space corrections
passed (17.995ms Firefox,18.723ms Chrome). Chrome's exact immediate inverse also
passed (16.450ms); its next letter changes the terminal NBSP to ASCII, so the
whole-raw-field letter assertion fails. Firefox's inverse restores both original
words but exposes an ASCII internal separator instead of the pre-Space seed NBSP.
Its exact raw target fails. The current probe inferred raw inverse from the
**pre-Space seed**, not a separately observed post-Space/pre-correction field;
that is insufficient to distinguish client Space canonicalization from an
incorrect inverse. Do not normalize these failures into PASS or change runtime
inverse storage to match this unqualified oracle. The sealed source action and
recorded causal precondition still require their exact observed source.

After each first-field inverse, later fields8/10 do not reach correction: all
four stop at Space target. Both traces record `ibus_auto_undo/restored_exact_original`.
Fresh isolated usage evidence records rejected-candidate feedback at that first
inverse, and subsequent metadata downgrades the pair to SuggestOnly. This makes
cross-field learning a concrete test dependency; the precise causal contribution
is not independently isolated. Fresh frozen data at phase start does not make
later cells independent after undo. Do not disable production learning, add
literal fixture exceptions or clear live data to get PASS. A qualified successor
must isolate each inverse cell and bind the actual post-Space raw source, with
separate representation and semantic outcomes. No further native replay was run.

Independent source review completed two passes:9/10 then10/10 after adding
patched-body/helper/inverse hashes, exact upstream source files to the archive,
phase allowlist and corrected wording. This static review did not detect the
client canonicalization or learning interaction; it is not native acceptance.
Zero-key scope checks pass with only `fresh` changed in the upstream body;
all native key functions, output guards, deadlines and independent cleanup remain.

Verified receipts under
`/home/ubu/.cache/lay/development/browser-final-fixes-20261018/pair-inverse/evidence/`:
Firefox21files, archiveSHA256
`d66d9910d7bd3eac52d96fdb1239b6f2e86b78e186ba1d5567995f99d0d0048c`,
CLOSED07:59:09.427525Z; Chrome21files, archiveSHA256
`f7bb0bcd7d20a2160a80eb02a3754381078a224e1ea5b15b404e078fdcacefe6`,
CLOSED07:59:55.228836Z. Both scopes are empty, all pinned processes gone,
VT1 restored, no parent-released keys and exact owned seat stopped.
`pair-inverse/CAUSAL-LEARNING.json`, SHA256
`ba67144d4bdce964299d07e18b6c9063d61814ee52dddfc6c42408c66a5d8bf5`,
retains only this synthetic pair's new feedback events and hashes of their
isolated source logs. Personal usage and installed models are untouched.

The final active-word regression formatting completed remotely under the shared
guard, with only `context_admission/adapter/tests/residuals.rs` fetched:
`active-probe/format-a8ze00o9/RESULT.json`, resulting fileSHA256
`5025490334cc4eedaef3c0e96161012f0aa215a4abd2032705cb8a13cf5bae17`.
Final graph/manifest refresh and affected source closure are next. Production
Rust bytes, runtime authority, models, delivery routes and local installation
are unchanged. Original browser354P46F32B/432 (zero NOT_TESTED) and the
preceding-byte KDE75P333F24B/432 remain separate. Whole task IN PROGRESS.

### TD-156 — Qt connection qualification and source closure bookkeeping, 2026-10-18T08:24:09Z

The retained KWrite/Konsole follow-up trace is178848bytes/1030records, below
its cap. It has438 context-admission observations and33 capability notifications,
but no `ibus_key`, surrounding-text, preedit or `ProcessKeyEvent` admission.
This is a gap in the old native proof, not evidence of a reducer failure after
an input callback. The old result remains75P333F24B/432 on the preceding auxiliary
bytes; no row is reclassified. The selected Qt IM environment, stock plugin,
Wayland socket naming and private bus-file location were checked. The hypothesis
that the launcher overwrote its IBus settings was not supported.

Options for the next causal observation: infer a disconnected Qt plugin from
old trace absence (2/10, cannot distinguish client setup from delivery); repeat
all432 KDE cells (3/10, broad work without a new first-transition witness); inspect
one fresh KWrite connection on the current candidate (9/10, selected). The latter
was limited to one physical KEY_G press/release, no Space or Double Shift. It
reused the actual KWin foreground observer and existing guarded seat/native path.
The scenario review scored6/10 then8/10. Review findings were fixed: exact unit
pinning; current logind seat/identity/STOP before key-down; cleanup after partial
renewal, absent SESSION and worker timeout; cleanup errors cannot exit success;
final passive-input drain/interruption invalidates the diagnostic. No third review
was claimed. Syntax and all ten remote script hashes were verified before launch.

Exactly one fresh native KWrite letter was sent: empty field to `п`, caret0to1,
no selection, actual AT-SPI Text observation, no physical interruptions. The actual
process loaded stock `libibusplatforminputcontextplugin.so` and Wayland platform.
Qt logged two handled filter events. The IME trace has two distinct key admission
serials33/36, press/release, both accepted; the decoded press is `п`, committed by
`printable_managed_commit`, followed by actual surrounding text length/caret1.
Thus the current candidate can receive real Qt key input in this setup. This
single-letter observation does not explain the older zero-key trace, identify a
source repair, prove manual-toggle authority, or establish KDE functional PASS.
The binary candidate is unchanged NBSP IME959d828c2d0557281ce8a30ee31689501538bb2e89f9506f5ecfa05b9ea05d35;
old KDE editor evidence used IME5056dd3e50089fb2809b2cdcc717b8f98db804c660ef8c5deb64cdcceb493697.
Do not attribute the difference to NBSP logic without a controlled cause.

The actor and controller completed the one-key observation, but the launcher
returned `BLOCKED_RESOURCE reason=descendant_cleanup`. Local orchestration is
therefore **ABORTED / session exit**, never COMPLETE/PASS. Logs show private
Plasma service connection errors after intentional compositor shutdown; these
are teardown observations, not proof that input caused a compositor failure.
The independent closure then removed the exact residual L1 service, verified an
empty owned cgroup, all original pinned processes gone, actor no held keys,
parent released-key list empty, VT1 restored, exact finite seat inactive and
temporary policy absent. Closure time2026-10-18T08:22:27.531380Z.

Bounded receipt: `/home/ubu/.cache/lay/development/browser-final-fixes-20261018/kde-link/RUN.json`;
hash-verified39-file archive under `kde-link/evidence/`, archiveSHA256
`e3813d3c09034910bb429de2396d6b7024f06d37bd65d172e7151f5181540c5f`.
`LINK-RESULT.json`, Qt module log, raw trace, exact scripts and CLOSED.json are
retained separately from the orchestration ABORT. No original browser or KDE
cell, installed runtime, delivery route, model or real user learning data changed.

The next full source check selected3223 tests across33 test targets, including
837 IME tests and the new eight-variant active erasure-receipt regression; every
actual target reports zero failures. Formatting and122 tooling tests pass (one
explicit skip). However the overall check is **FAIL**, after test execution,
because known_failures.json retained the preceding manifest hash. Receipt:
`/home/ubu/.cache/lay/development/run-r4_246uk/RESULT.json`, source archive
`f118799287ff87a96735d1c4161cc1bf56306866ec03c1990e49a9eb2b4db567`.
The binding was corrected to current manifestSHA256
`5300de97eecb8de4cb46c2bef488981253057d659da417ee4f0dabae6a0a9474`;
known failure count remains zero, observation and comparison rules unchanged.
This metadata correction still needs a clean final source closure. The previous
3222-test PASS remains the latest completed full-gate PASS, not a3223-test PASS.
Current original browser matrix remains Firefox176P21F19B, Chrome178P25F13B,
total354P46F32B/432, zero NOT_TESTED. KDE compatibility is not accepted.

### TD-156 — clean source closure and next causal boundary, 2026-10-18T08:36:04Z

The metadata-only manifest binding correction now has a clean full development
closure:3223 selected tests, zero target failures, formatting and tooling PASS.
Receipt `/home/ubu/.cache/lay/development/run-90h0cm7a/RESULT.json`, source archive
SHA256 `ea727cffbf60a42c06599f5d9964da05d6870a2d1d6bf99a7057a304e872f315`;
source HEAD c8b244ef609ad9c5610b3e441311342abf17e9c5 plus the recorded dirty
snapshot. Report elapsed491.81s; orchestration500.3s. The earlier manifest FAIL
remains retained; this clean result supersedes it for source closure only.
It does not certify installed bytes, client behavior, KDE, or a release.

The retained original Firefox field1 normal prefix row supplies the next exact
boundary: `тест, привет` -> first Double Shift `тест, ghbdtn` in17.958ms; the
ordinary `z`/Backspace probe restores the exact surface and caret12. The inverse
Double Shift is recognized by the daemon but refused at context authority.
The complete matrix trace reached its cap before that first failed transition.
The eight-variant source regression proves that an actual erasure receipt
permits this handoff and an absent receipt preserves refusal; it cannot establish
which native receipt arrived. Do not remove the authority guard to make it pass.

A single unchanged original prefix plan, fresh isolated data/runtime and a fresh
bounded trace is prepared at
`/home/ubu/.cache/lay/development/browser-final-fixes-20261018/prefix-causal/`.
No new prefix native action has occurred at this entry. The first static review
scored6/10; cancellation, exact reached-stage qualification, script/fixture pins
and independent-closure requirements are being corrected before launch. Trace
size/inode continuity is not asynchronous delivery completeness, which remains
UNKNOWN until records can be inspected. The original browser matrix remains
354P46F32B/432; KDE75P333F24B/432 remains separate and unaccepted. Runtime authority,
delivery routes and installed user binaries are unchanged.

### TD-156 — prefix controls and complete-line log compaction, 2026-10-18T08:53:23Z

The previously staged single original field1/prefix control completed once with
fresh isolated data/runtime. Both Double Shifts and both native letter/Backspace
probes PASS; two toggle observations are31.7560/32.5305ms. The retained trace is
161239bytes,715JSON records, same inode below the cap. Both actual manual plans
use `ime_active_composition`; this does not demonstrate a missing-erasure failure
or repair the original intermittent context-authority refusal. Review of this
single-plan diagnostic closed6/10 then9/10 after cancellation, qualification and
independent-closure fixes. Exact27-file archiveSHA256
`1078c251aea93efcdfc74120b0f1fba01bdf91d88d8c9d82070d4e162303b3ab`:
`/home/ubu/.cache/lay/development/browser-final-fixes-20261018/prefix-causal/evidence/firefox/`.
`prefix-causal/ANALYSIS.json` records supplemental PASS only. Independent CLOSED
at2026-10-18T08:40:28.964381Z verifies the empty owned scope and restored VT1.

A separate predecessor-history comparison then executed the unchanged first five
original field1 plans, base/short/long/mixed/prefix, in original order. The raw
actor receipt is COMPLETE with5PASS and ten measured Double Shifts, median
87.5711ms,p95145.2872ms. These different workloads/checkpoints are not a controlled
speed comparison with the two-toggle run. The overall orchestration is
**ABORTED / exit1**, because strict JSON parsing after native input encountered a
fragmented first trace line. No successful QUALIFICATION was produced. Preserve
the raw five rows separately from this ABORT; never relabel it an accepted run.
All41 bounded raw trace checkpoints were retained without another native run.
The final trace is512000bytes: first line invalid,2231 subsequent JSON records
valid, SHA256 `dd284c2b066d89f1570ef7b2fe7f6b2e24e6e1824903979d0d643b962bcebb44`.
Independent CLOSED at2026-10-18T08:46:02.704992Z verifies an empty owned scope and
restored VT1. Exact65-file archiveSHA256
`f72dc1943fcd25fd4d9e0ec34aa49379f8fd0c1c70f0bc7add3e095635c12be7`:
`/home/ubu/.cache/lay/development/browser-final-fixes-20261018/prefix-history/evidence/firefox/`.
`prefix-history/ANALYSIS.json` explicitly records raw actor PASS / orchestration
ABORT. Asynchronous trace delivery completeness remains UNKNOWN in both controls.
Neither control reproduced the original refusal or changes any original cell.

The fragmented line has a distinct, reproducible production diagnostic cause:
`src/debug_log.rs::tail_line_boundary` selected the newline before the byte cut,
then `tail_with_max_bytes` hard-sliced the oversized suffix inside its first
record. The prior helper test even expected11bytes under a10-byte budget. This
is not evidence of the Firefox input failure's cause. Options evaluated: ignore
invalid lines in the collector5/10 (loses evidence silently); increase the cap3/10
(postpones corruption); retain whole newest records within the same cap9/10
(selected). The repair chooses the next complete line boundary, retains exact
fits and drops a single oversized record whole. The500KiB cap, UTF-8 validity,
private-file writer, queue and flush timing remain unchanged.

Controlled remote RED with strengthened/new tests on the old algorithm fails
0PASS/3FAIL; receipt `log-compaction/red-4826a6ya/RESULT.json`, source archive
`e0ed46a9d46edfd1e4ab9a81eaaf0491a535b5cb782aa8e6cf032acbc51b352e`.
Remote GREEN passes3/3; receipt `log-compaction/green-mau5ulk_/RESULT.json`, archive
`b9b15f58c4ff61091e3d69b36ddc5941761b000f43ea89be9de32e4b9a926d45`.
Both paths are under the same `browser-final-fixes-20261018/` cache. The property
test checks every byte budget, zero/exact fits, Cyrillic/emoji and parseable JSON
against the longest whole-record suffix. Remote rustfmt completed at
`log-compaction/format-lulrdapm/RESULT.json`. Isolated read-only review scored10/10,
no blocker, one pass. This is source-level diagnostic proof only; no candidate
build, native acceptance or speed improvement is claimed for these new bytes.
The full3223-test PASS in the preceding entry covers the preceding source snapshot,
not this later added regression/repair. No input rerun is needed to fabricate a
success for the earlier parser ABORT.

The KDE code inspection also confirms a separate implemented-source limitation:
`focus_guard.rs` polls exact foreground state only for GNOME;
`layout_controller.rs::call_focused_window_info_once` rejects other backends with
`exact focused-window identity requires GNOME`. Existing KDE layout switching is
not an exact foreground-window authority provider. This is a blocker to claiming
KDE manual replay compatibility, not proof that all333 KDE failing cells share
that cause. Any KDE provider requires its own owning-contract decision and
native first-transition proof; neither a test observer nor weakened admission
can substitute for production authority.

Original matrix remains Firefox176P21F19B, Chrome178P25F13B, total354P46F32B/432,
zero NOT_TESTED. KDE75P333F24B/432 remains separate and unaccepted. Installed user
runtime, delivery routes, reducer, SafetyGate and verifier authority are unchanged.

### TD-156 — source inventory and separate KDE capability boundary, 2026-10-18T09:01:51Z

The remote inventory refresh following the debug-log regression is PASS at
`/home/ubu/.cache/lay/development/tab-manifest-uctnv_cj/RESULT.json`.
The manifest now selects3188 correctness +36 package cases =3224, with15 ignored
and11 performance cases separately listed. Manifest SHA256 is
`8cc6f4957c0a02f6165a418b04567b56eac1d48f976320e0e893019f759c91bc`;
the known-failure file binds that inventory and still permits zero failures.
This compilation/list evidence is not a full3224-test execution. Retain the
preceding full3223-test PASS and the later focused debug-log3/3 proof separately.
The final remote graph/check-architecture result is retained through the pointer
`/home/ubu/.cache/lay/development/browser-final-fixes-20261018/GRAPH-CLOSURE.json`;
its verdict must be read from that receipt, not inferred from this planned step.

Read-only grouping of the latest complete original negatives retains46 FAIL:
20 Space target preconditions before undo,20 focus-return operations, one
intermittent Firefox prefix inverse, and five Chrome field2/Search Backspace
returns. The latter conflict with successful raw client text and stale
accessibility observations; preserve original FAIL without claiming five
established input bugs. The32 BLOCKED retain20 unpublished fixed-Tab prerequisites
and12 rich functional seed prerequisites. Derived negative-cell inventories live
at `browser-final-fixes-20261018/{firefox,chrome}-current-negative-cells.json`.
No qualification or source test silently changes a native cell.

KDE has an additional independently verified source boundary:
`layout_controller.rs::activate_target_layout_once_for_exact_replay` requires
GNOME, apart from the missing exact foreground provider. Its activation/readback/
restore handoff stays refused on KDE; implementing a foreground provider alone
cannot enable exact replay. `focus_guard.rs` also constructs GNOME-scoped IDs
with metadata fallback and latches `FOCUS_INFO_UNAVAILABLE` after a provider
error. A future KDE provider must explicitly admit a backend-scoped stable
compositor ID and own bounded recovery/in-flight invalidation in the existing
consumer. Titles, PID and stale replies cannot establish authority.

These corners are documented in planned Stage2 task
`tech_debt/157-kde-exact-foreground-window-provider.md`. Isolated task-description
review scored8/10 then10/10, two passes; the second pass found no substantial
blocker. This is a reviewed plan, not implementation or native KDE acceptance.
Its closure is limited to foreground capability; independent replay, ranking,
Tab and fixture failures remain with their owners. No runtime authority changed
in this documentation/inventory checkpoint. Original browsers354P46F32B/432 and
the separate KDE75P333F24B/432 remain unchanged and TD-156 remains IN PROGRESS.

### TD-156 — bounded published-hint observation and native supplement, 2026-10-18T09:24:15Z

First failed transition of the preceding Tab supplement was the observer's
1000-node cap (19 cells), plus one application-lifetime failure. Source inspection
shows hidden-subtree enumeration and no alias/cycle guard; the old receipts do
not measure which branches caused each cap. Increasing the cap scored2/10;
cycle-safe visible traversal8/10 was selected; runtime publisher/replay changes
1/10 would change the wrong owner. The new read-only helper preserves1000 unique
objects and depth12, adds4000 edges, uses app.bus_name/path, prunes hidden nonroot
objects and refuses incomplete evidence. Missing children cannot look like an
empty or unique hint. GI/identity/budget loss remains BLOCKED after Tab.

The auxiliary selector remains strict: pinned GNOME application, visible unique
Cyrillic suffix, bounds and source-qualified content-container shape; prefix,
caret, selection and unchanged current hint are rechecked before the physical
Tab. No DecisionCore, reducer, verifier, delivery route, model or layout adapter
was changed. Independent source review7/10 ->9/10, two passes, repaired missing
children, post-Tab observation classification and missing source-log export.
Remote GREEN-v3 passes21/21 controls; the earlier GREEN CLI exit2 ran zero tests
and remains recorded. The initial hidden-subtree witness detects the prior DFS.

Three later setup refusals are preserved separately, not native-cell outcomes:
digit-bearing seat label, digit-bearing zero-key tag, then reuse of an immutable
seat receipt. The first creates no seat; the second is closed by the existing
finally; the third needs independent pinned PID/starttick/unit closure because
renew-seat is outside that finally. All execute zero diagnostic cells. The
append-only receipt is preserved, never overwritten. Final letters-only fresh
labels and output-absence checks precede staging. These mechanical naming fixes
are later than the9/10 snapshot; no third review or broader score is claimed.
The existing controller lifecycle gap remains a follow-up, with all failed setup
logs and independent closures retained under published-tab-visible/.

One actual diagnostic per browser then completes with unchanged original actors
and isolated data. Firefox10P0F0B and Chrome7P3F0B, total17P3F0B/20, zero NOT_TESTED.
All20 physical Tabs reach the exact normalized full candidate-plus-Space target;
this alone is not the row verdict. Chrome field2/Search fails whole-field
Backspace return: caret is10 but AT-SPI still reports the appended letter.
Fields7/10 fail the next-letter raw comparison: NBSP after Tab becomes ASCII
Space when the letter is inserted, with correct Cyrillic text and caret11.
Retain all three FAIL; no silent normalization or source-only reclassification.
This supplement does not include the original Tab plan's two Double Shifts and
cannot replace its20 BLOCKED cells.

Release-to-Tab-target timing: Firefox10/10 samples median17.851ms, nearest-rank
p95 19.803ms; Chrome10/10 median19.904ms, p95 22.746ms. Fully accepted Chrome rows
have7/10 samples, median19.363ms, p95 21.909ms. Report these scopes separately;
no overall speedup, Double Shift timing or KDE latency follows. The actor cannot
resolve its screenshot client in the private runtime PATH; the host package
`gnome-screenshot` is installed. Pixels remain NOT_VERIFIED, independently of
SHOWING/VISIBLE accessibility evidence.

Verified native archives,41 files each:
- Firefox `a25b4c8d90c09f2f0e71385a44deffbb0cc086caec594e5bbf115615fd83574a`,
  `published-tab-visible/evidence/firefox/`, CLOSED09:19:07.885503Z.
- Chrome `028fcf0c66fdc6de2857d7480e42d1077d151bb2f78bb9908131c8b6cedb11d8`,
  `published-tab-visible/evidence/chrome/`, CLOSED09:19:53.335765Z.

Paths are under `/home/ubu/.cache/lay/development/browser-final-fixes-20261018/`;
SUMMARY.json retains action and full-row denominators. Both closures verify empty
owned scopes, pinned processes gone, no parent-released keys, stopped seats and
VT1. Both actually load IME959d828c2d0557281ce8a30ee31689501538bb2e89f9506f5ecfa05b9ea05d35
and daemon e6792f7cb165735e5052e48865490c96110973619c5bc5656d32e7c99de68cd7.
These are preceding private candidate bytes, not the later debug-log source fix.

The helper is retained as `tests/manual/browser_ime_hint_tree.py`, byte-identical
to the natively used helper SHA256
f426cd3cc569f4a454178b250ba55a52003963afbf0f3d3c06c67b15c93eb94b.
Its permanent regression controls and existing seed controls pass21/21 remotely:
`published-tab-visible/permanent-source/RESULT.json`, archive
fc93590a7ecbbf48413955b115bec69d050488fef1d6ca3c60a892570e5b52c6.
This is Python observer evidence, not a new full3224 Rust-suite run. The shared
browser standard records separate dynamic-hint scope, negative completeness,
raw surfaces, timing and setup/closure requirements. Final graph/check-architecture
closure is read from GRAPH-CLOSURE.json after the current documentation refresh.

### TD-156 — source closure and bounded Space-authority evidence, recorded 2026-10-18T09:48:38Z

The current full remote development check passes3224 selected correctness/package
tests with zero failures. Receipt `run-1ygbhlgc/RESULT.json` under
`/home/ubu/.cache/lay/development/`, source archive
6e8c3e89196e37f51aea9b6c19ea1b8e3a188b8bef168e4d799b9a47314f4f96,
HEADc8b244ef609ad9c5610b3e441311342abf17e9c5. Cargo formatting and lane self-test
also pass. This closes the source snapshot including the later whole-line
debug-log regression; earlier3223/3222 receipts retain their scopes. No release,
installation, service restart or new physical acceptance follows from this run.

Independent Space-precondition diagnosis review scores7/10 then9/10, two passes. It
correctly rejects treating producer `productive_v90_lattice_requires_common_l3`
as the final DecisionCore refusal or as proof that an L1.1 Winner was erased.
The physical trace stops at prefetch `not_ready`; the old CLI had no final
L3/L4/admission details. Raising the4ms deadline or promoting this literal target
would change the wrong contract. The minimum next step scores10/10: use existing
opt-in DecisionCore tracing with fixed native bytes/config/packages. A bounded
extension of that trace, only if an actual admission ambiguity remains, scores9/10;
an independently qualified undo control scores6/10 and cannot repair Space.

The remote read-only diagnostic now completes. Existing private rootfs and
packages are mounted read-only; inherited HOME is not reassigned, private HOME
content is mounted at its existing value. The L1.1 service is explicitly owned,
package hash checked, Health READY/package identity verified, and closed after
the queries. No compositor, keyboard device, browser, model fit or feedback write
is started. All pinned inputs are unchanged afterwards; guard exits0.

Results for `превет` / `превет `:
- A separate direct L1.1 restore query: ABSTAIN, reason outside_calibrated_basin, distance1,
  seven candidates. `привет` remains terminal353325, backward663/positive999;
  those scores do not constitute a Winner or edit authority.
- Existing CLI has72 candidates, all SuggestOnly, selected:none in both of its
  existing resolutions. Target ProductiveL2V90TypedExact is retained, rank1.160,
  posterior0.629; L3=0/not_applicable, pairwise=false; L4=12 but state_specific=false
  and attract=repel=0, so no exact positive L4 witness is present.
- Normal configured safety does not enable the experimental strong-L2 suggestion
  path; the boundary path does not apply to the target's L2Surface source role.
  For the shown target producer lane, the ordinary predicate lacks the required
  L3/L4 certificates. CLI lexical_authority_frame is None by design. This is bounded
  evidence of unavailable lexical apply authority, not a lost grounded Winner,
  a verifier defect, or proof of the final native prefetch decision.

The separate direct service response is not the actual
CompositeL2LatticeV1.original_l11_verdict of this CLI resolution. The live route
reconstructs a RestorationReadout from contour seeds; multiple seeds without a
unique authoritative seed can become Tied. That composite verdict was not logged
and must not be inferred as ABSTAIN from the direct query. The second review
identified this distinction; it is corrected without another experiment or a
third review. The producer-lane conclusion is not a final verifier refusal.

Exact receipt under the TD156 cache: `core-authority-d/{RESULT.json,FETCH.json,
l11-health.json,l11-readout.json,stdout.txt,stderr.txt,guard.log}`. CLI SHA
f8aec338504dadd72bd06fa78f4b3e8bd1863376897fce95dd3081e2d2e0c3c9,
config20fe9fa76066463698ef4edee29733d9ce61bea7f97c901cf4fce22b03856bf5,
L1.1 artifact bf5a1619a89038466ef786305cf35eda5f4af5b9f12b9140f7d3cac407e2f2a7.
Cold service/query orchestration2397.549ms is not typing latency.

Preserve preceding diagnostic setup outcomes: `core-authority/` fails before CLI
because a mount target is absent; `core-authority-b/` completes with an absent
L1.1 service and only one deterministic suggestion, hence is not equivalent;
`core-authority-c/` reaches the service while WARMING and refuses before CLI.
None is a native matrix outcome. No product policy was changed to make the final
diagnostic match. A production quality change must follow the existing agreement/
ranker roadmap and full fixed proof, not a fit or special case for these20 cells.

Original browsers remain354P46F32B/432 (zero NOT_TESTED), separate KDE remains
75P333F24B/432. TD156 stays open; TD157 is planned capability work. Final generated
graph/architecture closure is recorded by the TD156 cache GRAPH-CLOSURE.json
pointer after this documentation checkpoint. Runtime authority is unchanged.

Original browsers stay176P21F19B +178P25F13B =354P46F32B/432. The older KDE
auxiliary-byte432-cell baseline remains separate75P333F24B/432 and unaccepted;
the current-byte KWrite one-letter diagnostic does not certify that full matrix.
The planned TD-157 exact foreground capability and the independent GNOME-only
exact-layout handoff remain distinct KDE blockers. TD-156 stays IN PROGRESS;
no local runtime, release or production model was installed or changed.

### TD-156 — native Space admission boundary and final implementation review, recorded 2026-10-18T10:05:25Z

A fresh, finite native Firefox normal-window field1 run reaches the unchanged
original undo plan's Space prerequisite. It physically types `превет` and sends
one Space. The cell fails `Space target`: actual `превет `, caret7, focused,
zero selections. Undo is NOT REACHED. This one diagnostic does not reclassify
the original matrix or establish the cause of the other19 Space prerequisites.

The structured trace now resolves the earlier `not_ready` uncertainty for this
cell. Matching worker_generation11, tail_epoch16 and engine path
`/io/github/radislabus_star/LayIme/engine/lay_ime_ru/2` show72 candidates, all72
SuggestOnly, no apply/keep/veto, input gate `suggest_only_candidate`, no final
decision or selected transition receipt. `authority_admitted_pool_count` is
null/UNKNOWN, not measured numeric zero. Prefetch publication is prepared;
the terminal is `no_apply`, stage rank. The lease is READY after3277us, within
the existing4ms wait. Queue34175us + evaluation24318us + publication5us =58499us;
the Space key then takes3392us on managed fallback commit (autocorrect3306us,
commit84us). These are internal NO-CORRECTION timings, not correction acceptance
latency. Increasing the wait cannot repair this measured no-apply outcome.

Native evidence establishes a producer/admission no-apply before delivery for
this cell. It does not expose native candidate identities/certificates or the
actual composite original_l11_verdict. The separate read-only CLI diagnostic
retains the intended target, but matching72 counts is not identity proof across
processes. Do not turn its direct L1.1 ABSTAIN into the composite verdict, claim
a lost Winner, bypass SafetyGate, or train a model against this literal fixture.

Exact local receipts: TD156 cache `space-native-authority/SUMMARY.json` and
`space-native-authority/evidence/firefox/phase/{RESULT,QUALIFICATION,CLOSED}.json`;
remote actor `firefox-functional-plfrmh6k/RESULT.json`, SHA
d13aa623599feb867cbb7e5a24af80da4b488dce63ac159a78018b9a07a84cea.
The19-file fetched archive hash is
c8a6654448c5c79ebb1060f54aaabaf4a5870781d6480c86e7aa348a20b92296.
Qualification counts18 evdev events, one action, zero decoder probes and435
parseable trace records; async delivery completeness remains UNKNOWN. The
candidate CLI/daemon/IME, config and source-archive hashes are pinned by
SUMMARY.json; IME959d828c2d0557281ce8a30ee31689501538bb2e89f9506f5ecfa05b9ea05d35.
These native bytes precede the diagnostic logger covered by full source3224.
At09:57:24.130792Z the owned cgroup is empty, pinned processes are gone, no keys
remain held, the finite seat unit is stopped and VT1 is restored.

The final independent context-free implementation review scores9/10, one pass,
with no new blocker or required code fix. It reviews the current runtime diff,
raw Space/NBSP pair/inverse guards and observer completeness; source and receipt
inspection only, no native rerun. It confirms that legacy auxiliary hints do not
become document composition, presentation tolerance remains bounded to exact
Space positions, and owner/epoch/material/one-shot/SafetyGate authority remains.
It explicitly leaves native inverse, focus and other negative cells open.

Source closure remains full3224PASS; original native354P46F32B/432 and separate
older KDE75P333F24B/432 remain unchanged. No runtime authority, delivery route,
installed runtime, production training data or release changed. TD156 remains IN PROGRESS.

### TD-156 — full published-Tab round-trip supplement, recorded 2026-10-18T10:13:25Z

The earlier published-hint supplement omitted the original Tab plan's Double
Shift actions. This new finite remote run adds both original `roundtrip()`
directions, exact surface/mode checks and decoder-letter/Backspace after each.
Each of ten normal-window fields starts empty, types the exact prefix `пров`,
observes the complete owned visible auxiliary candidate and rechecks that same
path/text before one physical Tab. EN text is derived with the pinned original
keyboard map. The post-Tab raw Space/NBSP is retained for subsequent checks.
No runtime code, delivery, model or production data is changed by this run.

Independent diagnostic review scores9/10, one pass. It catches a controller
receipt-filter error before native input: the actor emits PREPARING and terminal
receipts, so only terminal records may satisfy the exactly-one assertion. The
reviewed correction is present in both executed phases. The controller requires
ten unique terminal rows, unchanged runtime, uninterrupted activity and balanced
native keys. A row PASS requires Tab plus both Double Shifts and both probes.
The inherited renew-seat-before-finally setup gap remains documented; the exact
finite seats are independently confirmed closed after these successful runs.

Results, separate from the unchanged original432 grid:

| Browser | PASS | FAIL | BLOCKED | NOT_TESTED | Accepted Tab | Reached DS | Completed probes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| Firefox | 10 | 0 | 0 | 0 | 10/10 | 20/20 | 20/20 |
| Chrome | 9 | 1 | 0 | 0 | 10/10 | 19/20 | 18/20 |

Chrome Search field2 fails the first Backspace return after its first successful
Double Shift. AT-SPI reports `ghjdthbnm z` with caret10, while the pre-probe target
was `ghjdthbnm ` with caret10. This is retained FAIL; the second Double Shift and
remaining probe are NOT REACHED. The previous separate raw-client/Search causal
evidence must not be replaced by an assertion that this new observation proves
a product deletion or Double Shift defect. Other9 Chrome fields complete both
directions, including rich fields7/8/10. This is a changed sequence from the prior
Tab-then-RU-probe supplement, not proof of an intervening runtime repair.

User-space evdev write-return to exact AT-SPI target timings (ms): Firefox Tab
n10/10 median34.449, p9535.640, max35.640; Double Shift n20/20 median117.829,
p95134.797, max139.379. Chrome Tab n10/10 median31.665, p9535.157, max35.157;
Double Shift n19/20 median105.440, p95131.041, max131.041. These are reached-action
samples, not timings of20 fully passed Chrome transitions or kernel-release
measurements. Software-rendered private-seat timing is not hardware rendering
performance. Pixels remain NOT_VERIFIED; the optional screenshot client remains
unavailable in the private runtime PATH.

Exact receipts under TD156 cache `published-tab-roundtrip/`: SUMMARY.json,
REVIEW.json, CLOSURE-VERIFIED.json and evidence/{firefox,chrome}/. Firefox actor
`firefox-functional-f41o4zn3/RESULT.json`, SHA
13b8fcc8f9b76aea8e18e23ee978bb5ca048741b743045bd03f878ecc2369560;
archive56855fb665447e63fb716c81d7cc8e2df936950a57f650382ac1f2ced77dfeb4.
Chrome actor `chrome-functional-3ds1wsvj/RESULT.json`; exact receipt hash is in
SUMMARY.json, archive4c6538f86301a40ba9d61f7a6c7f3cd2b38cc861de9825d0bccaa504e15142e7.
Each archive has20 hash-verified files. Phase closure times are10:08:57.838020Z
and10:10:11.141594Z; independent zero-key verification10:11:14.676281Z confirms
both stopped exact seat units, empty cgroup, gone pinned processes and VT1.

The private native binaries still predate the whole-line debug-log repair. Both
trace copies reach512000 bytes, lose the old prefix and have one invalid leading
line; causal trace completeness is NOT QUALIFIED. Native action/client receipts,
identity/activity guards and exact closure are retained independently. Do not
claim these capped logs locate an intermittent core failure. New source3224
already covers the logger fix; no rebuild/rerun is invented here.

This supplement now exercises the complete functional sequence for the current
published candidate. It still cannot replace the old fixed-literal/preedit Tab
prerequisite or prove ranking of `проверка`. Original354P46F32B/432 and older
KDE75P333F24B/432 stay unchanged. Source3224PASS and implementation review9/10
remain separate. TD156 stays open; no release or local installation is performed.


### TD-156 — exact prefix failure-window capture preflight, recorded 2026-10-18T10:24:24Z

A source audit of the preceding full-prefix collector reveals a concrete
selection defect: its observer still selects [3,4], row16, field4/short. It
therefore does not preserve the earlier [1,2], row4, field1/prefix refusal.
This is a diagnostic defect, not proof of a runtime cause or repair.

One finite remote diagnostic is prepared under TD156 cache
`prefix-window-capture/PLAN.md`. It reuses unchanged private runtime bytes and
the exact original first five Firefox jobs: standard normal/private/fullscreen,
panel, then functional fields1/2. These are96 selected original cells, preserving
the original predecessor history; the remaining120 are NOT SELECTED. The native
actors, keys, expected targets and cadence remain unchanged. Only the existing
controller wait observer now selects the intended fresh current-candidate FAIL
and saves its receipt/raw bounded trace once, without touching actor output.

Hypothesis remains that a probe erasure receipt/order determines native inverse
admission; passing source controls alone do not determine that native order.
Actual gesture and callback records must qualify the causal window. Capped old
logger bytes and any fragmented first record are retained; asynchronous delivery
completeness stays UNKNOWN. A no-reproduction result cannot close the original
FAIL. This experiment addresses the documented missing-window evidence rather
than repeating an unchanged test for a new score.

Before native input: five remote observer controls PASS (0.006s), including a
rewrite of RESULT after selection. The observer parses and archives the same
bytes rather than rereading the actor's nonatomically updated file. Independent
read-only review closed both collector findings in two passes, final9/10;
`close-seat.py` is included in the evidence archive.
After input: exact receipt hashes, unchanged-runtime/identity/activity checks,
verified finally closure and independent exact finite-seat/process/cgroup/VT
closure. No build, learning, model fit, delivery change, local install or runtime
authority change. Original354P46F32B/432 and separate KDE75P333F24B/432 remain.

### TD-156 — original prefix history complete, NO REPRO, recorded 2026-10-18T10:40:17Z

The single prepared96-cell diagnostic is complete:90PASS/4FAIL/2BLOCKED,
zero NOT_TESTED within its selected scope. The remaining120 original Firefox
cells are NOT SELECTED; this is not a new complete216 or432 matrix. The five
unchanged original jobs run in their original order: normal20PASS, private20PASS,
fullscreen20PASS, panel6PASS, then functional fields1/2 at24PASS/4FAIL/2BLOCKED.
Runtime bytes, frozen starting data, actor bodies, targets and cadence are
unchanged. This addresses the old collector's wrong target, not a new runtime
implementation.

The intended field1/prefix row passes both native Double Shifts, at18.542802ms
and18.447612ms, with both actual decoder-letter/Backspace returns, exact target,
caret, selection and mode. Therefore the intermittent original inverse refusal
is **NOT REPRODUCED**, not repaired or disproved. No failure capture is created,
as required for a passing row. The first66 original predecessor cells passing
does not establish that their history deterministically causes or prevents the
old failure. No repeat-until-green or additional native replay is performed.

The four reproduced failures are field1 and field2 Space prerequisites and
focus-return Double Shift. The two blocked rows still require the old fixed
literal Tab completion; they are separate from the actual-published-Tab native
supplement. Do not relabel either class, infer undo coverage after a failed
Space prerequisite, or count a blocked Tab row as a reached edit.

Reached gesture latency from the actor's second evdev Shift-release write return
to the exact AT-SPI client target (software seat, not physical key/hardware timing):

| Original job | Measured / recorded gestures | Median ms | p95 ms | Max ms |
| --- | --- | --- | --- | --- |
| Normal |100/100|99.079|115.312|128.544|
| Private |100/100|99.953|110.532|120.593|
| Fullscreen |100/100|96.488|106.549|109.487|
| Panel |48/48|101.066|114.340|119.943|

These are diagnostic-scope samples, not a universal speed improvement or a
measurement of a refused action. The end log retains512000raw bytes,
2288valid JSON records and one fragmented leading line. The corrected failure
capture is not exercised natively because its target passes; the five remote
observer controls and read-only review9/10 after two passes remain source-level
collector proof. Asynchronous delivery completeness remains UNKNOWN; the capped
end log cannot reconstruct an absent failure or establish its callback order.

Evidence: TD156 cache `prefix-window-capture/SUMMARY.json` and
`prefix-window-capture/evidence/firefox/FETCH.json`;31fetched files are individually
hash-verified, archive SHA-256
`69e42926463113da1cab25cb72af64ade012f509979a3318f9c85333a047605c`.
The exact target actor is `firefox-functional-05s91wx2/RESULT.json`, SHA-256
`6621ec8b159201d320ed5c19db4d77ba95c6bfdcfa8cbba18ad29b0853de3ebf`.
Owned finally closure at10:38:49.017535Z has empty cgroup, gone pinned processes,
no parent key release and VT1. Independent zero-key exact-unit closure at
10:39:09.762746Z is CLOSED with no errors.

The next runtime edit still needs the first failed native ownership transition;
neither a passing isolated/history control nor a partial log grants new authority.
Space quality remains with the existing restoration/authority roadmap; the READY
native lease already rules out increasing the wait for the one measured cell.
Focus needs editor identity/ownership evidence; equal window/text/end caret do
not supply it. KDE foreground and replay handoff capabilities remain stage2 in
TD157. No model fit, gate relaxation, delivery change, local install, publication
or runtime-authority change. Full source3224PASS, original354P46F32B/432 and
separate older KDE75P333F24B/432 are retained. TD156 remains IN PROGRESS.

### TD-157 — KDE failure inventory and bounded capability plan, 2026-10-18T10:57:35Z

The user requested improvement of the KDE repair plan for the retained333FAIL.
This entry records read-only receipt analysis and planned work, not an
architecture experiment or native repair. Checkout remains
`/home/ubu/projects/lay-browser-final-fixes-20261018`, HEAD
`c8b244ef609ad9c5610b3e441311342abf17e9c5` with the intentional dirty source.
No production source, input route, model, installed binary or runtime authority
changes in this planning block.

All six primary receipts named in `kde-editors/COMBINED.json` are freshly
SHA-256-verified and yield432unique application/field/plan identities,
75PASS/333FAIL/24BLOCKED, zero NOT_TESTED. The333FAIL partition by the first
failed actor step is disjoint:

| Step | GTK | Qt | Firefox | Chrome | KWrite | Konsole | FAIL |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| Clear rich field | 0 | 0 | 45 | 45 | 0 | 0 | 90 |
| Seed or return prerequisite | 3 | 3 | 0 | 0 | 0 | 0 | 6 |
| Target after actor Double Shift | 15 | 24 | 66 | 66 | 8 | 7 | 186 |
| Target after actor Space | 4 | 6 | 14 | 21 | 3 | 3 | 51 |
| Total | 22 | 33 | 125 | 132 | 11 | 10 | 333 |

Ninety rich clear failures are fifteen later cases in each browser's fields7,8,10,
with accessible raw`\n`, caret0 rather than the required empty seed; their
intended operations are not reached. Six further prerequisite failures are five
GTK/Qt prefix seeds and one GTK focus-return prerequisite.186post-gesture failures are
148ordinary/burst,20selection and18focus-return exact-target checks.51Space
failures are42layout auto-flips and9moved-boundary cases. The actor's target
combines exact text, caret, selection and KDE layout; its error is not a
qualified observation of which runtime conjunct failed. Requested native keys
also do not by themselves prove daemon recognition or authorization. Therefore
neither186 nor333 may be attributed wholesale to the known GNOME-only source
guard. The24BLOCKED remain21literal completion-panel prerequisites,2missing
second fields and1unsupported PTY selected range; none is regraded.

Inventory:
`/home/ubu/.cache/lay/development/browser-final-fixes-20261018/kde-plan-triage/TRIAGE.json`,
SHA-256`ee84c5365b20bf25e9fdd53fcae47ed85e7a10b99e6e2c71bcf12523441a6daa`.
It preserves each failed primary row index, input hashes and source actor hashes.
It is a derived inventory, not a new native receipt. The KDE auxiliary-byte
baseline stays distinct from the current GNOME/NBSP bytes and whole-line-log
source closure; new candidates must freeze their own exact identities.

Current source independently confirms GNOME-only polling/exact focus reads,
GNOME-scoped identity admission, a permanent unavailable-state latch and a
separate exact-replay activation guard. `ExactReplayLayoutHandoff` also restores
through GNOME directly. Ordinary KDE switching is not an exact replay proof.
KWin5.24.7 primary source exposes activeClient/clientActivated/clientRemoved,
a constant internalId UUID and indexed layout operations/readback. These are
feasibility leads for a version-scoped provider, not accepted bridge authority
or proof that a diagnostic observer is safe for production. Backend/lifetime,
authenticated ingress, freshness and no-cross-window restoration need their
own reviewed contract before runtime edits.

The single KDE plan entrypoint is
[TD-157](../../tech_debt/157-kde-exact-foreground-window-provider.md).
Its ordered work references [TD-159](../../tech_debt/159-kde-native-prerequisites-and-causal-witness.md)
for bounded native prerequisites/causal observations, this provider task for exact
foreground identity and lifecycle, and [TD-158](../../tech_debt/158-kde-exact-replay-layout-handoff.md)
for backend-aware activation/readback/restoration within the existing replay
owner. Newly reached ranking/Space/editor residuals stay withTD156 and the
existing authority path. A provider-only PASS cannot close the handoff or all
KDE compatibility. Preserve the full original browser grid as shared-code
regression evidence and report the new KDE candidate's grid separately.

The evidence task is not a global runtime dependency: TD157 may start after a
qualified native witness identifies its foreground boundary and the required
verification prerequisites are ready. Unrelated seed/Space families remain
open inTD159/TD156. TD158 has the same boundary-specific evidence gate and
depends on foreground capability, not on closure of every inventory family.

No new source/native tests, native typing, fit, installation, release or
publication in this analysis. Previous source3224PASS and native matrices stay
historical for their own bytes: original Firefox176P21F19B +Chrome178P25F13B
=354P46F32B/432, separate KDE75P333F24B/432. Task-description review is a
separate gate; its score must not imply implementation acceptance. TD156 remains
IN PROGRESS, KDE capabilities planned and compatibility unaccepted.

The coordinated plan received two context-free description-review passes,
9/10 then10/10. One P2 identified an ambiguous global TD159 dependency; the
boundary-specific gates above resolve it. Final review reports no substantial
remaining P1/P2 or blockers. Review scope is documentation/receipt consistency,
not runtime execution. Exact review record:
`/home/ubu/.cache/lay/development/browser-final-fixes-20261018/kde-plan-triage/REVIEW.json`.

### 2026-10-18T12:47:41Z — TD159 bounded witness prepared; no native result yet

The reviewed-plan implementation starts with the five fixed stage witnesses
fromTD159, using the existing shared actor and remote private Plasma/IBus seat.
Source/runtime candidate remains unchanged. Cache`kde-witness/` pins twelve
scripts, all remote syntax/hash checks PASS; six remote guard-admitted helper
and actual target-adapter controls PASS. They preserve original target arguments,
deadline, results and exceptions, detect missing observations, and keep raw
newline distinct from empty. Marks buffer in memory; observation does not add
synchronous writes between key bodies and targets. Exact selected identities
and raw receipt hashes gate diagnostic completion.

First context-free harness review found two bootstrap P1 and three observation
P2 findings. Valid two-part seat identity, dedicated timer/policy with bounded
closure, exact-cell checks, buffered marks and adapter controls address them.
Second/final review and the single finite native attempt are pending at this
checkpoint. Remote control receipt: cache`kde-witness/REMOTE-CONTROLS.json`.
These controls do not qualify native causality or changed-client acceptance.

No runtime authority, delivery-route, model, installed binary, original verdict
or release change. KDE remains75P333F24B/432; original GNOME browser grid remains
Firefox176P21F19B +Chrome178P25F13B =354P46F32B/432, zeroNOT_TESTED.

### 2026-10-18T12:53:10Z — TD159 partial native witness qualifies TD157

The one finite five-case diagnostic budget is ABORTED after three GTK FAIL
observations, not a new complete grid. Entry ordinary roundtrip qualifies the
native committed seed, physical gesture recognition and delegation of the exact
IME tail. Its first runtime refusal is `exact focused-window identity requires
GNOME` (bounded cell2 daemon trace and full daemon log lines37–38). TD157's own
boundary-specific gate is therefore met; unrelated browser prerequisites need
not block the provider. The separate exact-layout activation barrier was not
reached and remains protected.

TextView prefix failed before the gesture: display includes `тест, привет`,
committed content is `тест, ` and preedit is `привет`. Entry Space restored
committed `hello ` with correct caret/selection, while layout and IBus stayed RU.
These are distinct observations, not a common ranker diagnosis. Fragment reads
have verified hashes/offsets, no shrink or truncation, and unknown asynchronous
completeness; post-case conjuncts are not atomic timeout snapshots.

Firefox actor cleanup raised `RuntimeError: can't start new thread` in evdev
device close; original/resource cause is not yet established. No Firefox case
qualified; Chrome was not started. The five-case budget is spent, not permission
for an unchanged retry. Independent closure is CLOSED: owned cgroup empty,
pinned processes gone, VT1, owned seat inactive and temporary policy absent.
No reached operation samples, so no median/p95 success latency is available.

Final harness review was 9/10 after two passes, with its remaining cleanup-policy
P2 repaired before input. Exact evidence under the TD156 development cache:
`kde-witness/CAUSAL-WITNESS.json` SHA9556b2a27e104b97c341c94021cc415c5e1ff14c3fbd7894b3019e76faf01980;
`evidence/native-windows-lxqo62i_/RESULT.json`
SHA5be8a0ec42fe2b764846683e525734c341c1ff9b97122550256fc9aa4d8e57ed;
60-file archive SHAa5d554a8f183ef30cca72d9bde47c4a3a0391ef99615046bc6b94b13ef5c1cd2.

No runtime/source authority, delivery-route, model or installation change yet.
KDE75P333F24B/432 and original GNOME Firefox176P21F19B +Chrome178P25F13B
=354P46F32B/432 remain historical, unchanged and from different runs/bytes.

### TD157 implementation preflight — fresh KWin foreground, before production edits

Baseline is checkout`codex/browser-final-fixes-20261018`, HEADc8b244ef, with its
existing dirty IME fixes preserved; native witness uses IMESHA959d828c2d055728.
First failed transition and exact receipts are recorded above. Existing daemon
focus polling/application and IME lease/verifier remain the owners. The proposed
provider changes only platform evidence; exact replay layout refusal stays until
TD158. Decision`2026-10-18-kde-fresh-foreground-provider.json` records alternatives,
consequences and rollback before production edits.

Select a fresh, request-bound one-shot embedded KWin script (9/10), over a
pushed observer (4/10, no exact read) or private-ABI binary plugin (2/10, package
and compositor risk). KWin5.24.7 primary source confirms internalId/activeClient,
script load/run/unload and asynchronous callDBus. Its public getWindowInfo map
does not expose active state; interactive queryWindowInfo is unsuitable. The
adapter must authenticate KWin's unique bus owner, scope identities to bus and
connection lifetime, reject late/unsolicited responses and clean up before a
new load. No cached timestamp or title/PID is an exact identity.

Bound a complete read at250ms, cleanup at25ms, and error recovery by the existing
500ms poll owner. These are design bounds, not measured performance. Source
tests must assert sender/nonce/deadline/lifecycle semantics, not only producer
tags. Real IPC and compositor focus tests remain gates. The query cost and
unchanged GNOME plus separate new KDE full matrices must be reported before
claiming compatibility. No source-test count can stand in for those native
results. No additional owner, text route, ranker change, installation or release.

### 2026-10-18T13:18:18Z — TD157 provider source and authenticated IPC checks

Implemented the preflight's fresh KWin provider, serialized behind the existing
focus owner. It supplies UUID/bus-owner/lifetime identity and existing ignore/tab
metadata only. Missing/invalid/timed-out evidence pauses host handling and clears
identity; recovery requires a fresh request. The GNOME-only exact layout guard,
IME delivery routes, models, installed runtime and original matrices are unchanged.

First focused source run failed compilation: an implicit test-module path selected
the daemon's unrelated test module; GUID comparison also used incompatible types.
Both were repaired. Next daemon run selected307:304PASS/3FAIL. These three new
IPC tests exposed an unsuitable peer-to-peer fixture: assigning unique names
filters calls to the bus name; its method names also differed from KWin's actual
lowercase API. Replaced this fixture with a private real dbus-daemon, real service
ownership/sender headers, bounded startup and owned-process cleanup. No production
fallback or test-only runtime branch was added.

Provider diagnostic now selects exactly14 tests from actual Cargo discovery:
14PASS,0FAIL, execution0.194s (20.253s including compilation). Real IPC covers fresh
queries, stale nonce, foreign sender, failed unload blocking new load, and service
owner replacement. Receipt: development cache `kde-provider-IPC.json`, remote
`tab-fmt-1cm0uA/provider-tests/RESULT.json`. This is component source/IPC evidence,
not a full development or compositor result.

A single controlled RED removed sender authentication in an isolated remote copy.
The real-bus foreign-sender regression failed at its expected refusal assertion.
Restored provider source SHA7cb57af6c8824e4ba804f25fbf94f0a8587977f1f8f83e6a2cdbc3162293e7c4
matches its original. RED receipt: `tab-fmt-1cm0uA/provider-auth-red/RESULT.json`.
This is a controlled violation, not a claimed reproduction of the old missing
provider. Canonical inventory was regenerated remotely; the zero-known-failure
ledger remains empty and strict. Its inventory binding changes mechanically;
the retained zero-observation is historical, not new candidate PASS evidence.

Full affected source gates, independent review, actual KWin focus/loss/recovery,
native latency/CPU/RSS, and GNOME/new KDE client matrices remain pending. KDE is
still75P333F24B/432; original GNOME is Firefox176P21F19B +Chrome178P25F13B
=354P46F32B/432. Both have zeroNOT_TESTED; this partial source run replaces neither.

The subsequent graph/architecture gate correctly refused file-opening and
permission traits outside their registered owner. Kept the guard unchanged:
script creation now calls a minimal create-new/no-follow helper in the existing
`private_file` owner; directory validation reads MetadataExt mode. The provider's
file collision/private-mode regression continues to exercise the consumer.

### 2026-10-18T13:36:58Z — TD157 full affected source gate

The full remote development check is PASS: 3238 selected tests, no recorded
semantic or infrastructure failures, formatting and runner self-test also PASS.
Receipt: `/home/ubu/.cache/lay/development/run-sy49nis0/RESULT.json`; remote
`/home/e/projects/lay-development-runner/run-vPf73C`. Frozen archive SHA-256:
3010f5f94496483bb0b06860684183cf80a6da43ddc166289ba1c305c30368f8.
Remote commands took504.800s; complete orchestration took514.426s. These are
development-check durations, not input latency. The preceding graph gate also
passed after using the existing private-file owner, with unchanged guard rules:
`/home/ubu/.cache/lay/development/tab-graph-k0gaykxb/RESULT.json`.

This qualifies the source for independent code review, then a frozen candidate
build and one changed-candidate GTK foreground-boundary witness. Actual KWin
UUID serialization, physical focus freshness/loss/recovery and input latency
remain unproved. The separate GNOME-only exact-layout guard remains TD158.
No installation, release, model change or client acceptance is implied.
Historical KDE75P333F24B/432 and GNOME354P46F32B/432 remain unchanged.

### 2026-10-18T13:43:31Z — TD157 review pass1 and bounded contention repair preflight

Independent context-free implementation review:8/10, noP1, oneP2. The global
provider's try-lock conflates another listener's in-flight read with service
loss. There is a production listener per keyboard; a colliding idle poll can
therefore clear window identity, advance the shared field epoch and cancel a
healthy active word/gesture. The 14 component tests use independent clients
and did not cover this shared production path. Source PASS does not resolve it.

Amended the same explicit provider decision before repair: represent Busy
separately; a background poll defers without changing availability or epochs,
while an exact read still requires fresh evidence and refuses on contention.
No cached snapshot, unbounded waiting, new retry loop, identity owner, input
route or GNOME behavior change. Add a bounded, ordered two-thread regression
through the actual shared provider lock and existing focus reducer, asserting
word/gesture/epoch survival on Busy and invalidation on real service failure.
Then rerun full affected source gates and obtain the second/final review.

The one-case changed-candidate GTK harness has12 compiled scripts and8 remote
observer controls PASS (`kde-provider-native/REMOTE-CONTROLS.json`). Original
key/target AST bodies and1.8s target deadline are unchanged. New read-only
metadata records actual KDE layout API/mapping and post-case Lay input mode;
activity-close errors are retained separately from the original actor error.
No new native case has run and the historical matrices remain unchanged.

### 2026-10-18T13:50:34Z — TD157 contention regression before final source gate

Separated provider Busy from service loss in the existing focus consumer. A
bounded ordered two-thread test holds the actual shared provider lock and checks
that polling preserves the active word, pending Double Shift, window identity
and shared field epoch. Exact admission still refuses Busy; actual service loss
still clears identity and invalidates the operation through the existing reducer.
No cached identity, new retry, delivery route or GNOME branch was introduced.

Remote focused proof:15 tests PASS,0 failures; execution0.226450s. One controlled
remote-only reintroduction of Busy-as-loss makes the new regression fail at
`!state.focus_ignored`; original source restored byte-for-byte, SHA-256
ada4d8d57d67cac970eb06104cef6c048438eb7b980a5baba1bbad1f065f62ac.
Receipts: `/home/ubu/.cache/lay/development/browser-final-fixes-20261018/kde-prepare/provider-contention/{GREEN,RED,RESULT}.json`.
The22.758s combined build/test experiment is not native input latency.
Formatting and remote canonical manifest refresh PASS (`tab-fmt-sk34vwey` and
`tab-manifest-7ai32c65`); the empty known-failure ledger retains its historical
zero-observation and is rebound mechanically to the regenerated inventory.
The previous3238-test full PASS predates this repair and does not certify it.
Full affected checks and the second/final code review are next. Native behavior
and latency remain unproved; KDE75P333F24B and GNOME354P46F32B are unchanged.

### 2026-10-18T14:01:00Z — TD157 contention repair full affected source PASS

Repeated the complete affected remote development gate after the Busy repair:
3239 selected, zero recorded failures; formatting and runner self-test PASS.
Receipt: `/home/ubu/.cache/lay/development/run-g0ss9and/RESULT.json`; frozen
archive SHA-256433caec7f63aaf4f325939144bffe6024d16a3706d373fc30d1aa10b036db47d.
The514.5s orchestration duration is a development-check cost, not input latency.
Graph/architecture gate PASS: `tab-graph-30sa9au8/RESULT.json`. The original
3238 receipt is retained separately and is not substituted for this changed code.
Second/final independent implementation review has been requested. Only after
that review may the frozen private candidate be built and the one changed-code
GTK witness run. No new native verdict, installation, release, model or route
change. Historical KDE75P333F24B and GNOME354P46F32B remain unchanged.

2026-10-18T14:04:06Z — TD157 final independent implementation review:9/10,
no remaining substantive P1/P2 findings. Shared production-lock Busy handling,
exact-read refusal and causal GREEN/RED regression were inspected; the reviewer
reused the full3239 source receipt and ran no jobs. The bounded GTK witness
changes were also inspected without a critical finding. This closes the second
and final code-review pass, not the native provider gate. Next: freeze and build
the reviewed source privately, then one changed-candidate GTK witness. KWin
physical lifecycle behavior and input latency remain NOT_TESTED; the independent
TD158 layout handoff may still refuse. Historical matrices unchanged. Review:
`/home/ubu/.cache/lay/development/browser-final-fixes-20261018/kde-provider-review-pass2.json`.

2026-10-18T14:13:26Z — TD157 private build, TD158 native first failure.
Remaining architecture/audit/lint gates PASS; full3239 source proof retained.
Build receipt: `/home/ubu/.cache/lay/development/browser-final-fixes-20261018/build-kde-provider/BUILD.json`.
Source archive2c30f03065b667fa20b37bab3b635ad183f46e4a64c947c0c12c20803af7fe27;
daemon55536386ac7c0e547213c1dde29c15c67127dff7afda1957a9aeb763b335c71e;
IME5d84ca28241054495cfef1ecf2a4ececb09d69d097a25be20258651ed7a9d47d;
CLI f8aec338504dadd72bd06fa78f4b3e8bd1863376897fce95dd3081e2d2e0c3c9.
Harness-a aborted before keys with undefined plan enumeration. Preserved its
receipt, fixed only the global plan list; actual actor-selector controlled RED
reproduces NameError, all9 harness controls PASS remotely, key bodies unchanged.
Harness-b native actor COMPLETE, oneGTK Entry FAIL,22 key events, no held keys.
Committed seed `привет`, caret6, no selection/preedit, KDE/IBus/Lay mode RU.
Fresh KWin UUID passes the previous foreground boundary; first failure is now
`exact IME tail replay requires the GNOME layout backend`. Target `ghbdtn`
absent; no successful latency sample. This qualifies TD158, not full provider
lifecycle acceptance. Observed KWin API getLayoutsList `(a(sss))`, US/RU rows,
keyboard/KWin same unique owner; interface XML SHA256
d7a87628e3cc02fb333dc659b1c1a114005b059c53a423ed03d8c4bbaf0e06f0.
Outer orchestration ABORTED at session exit, guard reports descendant cleanup
during Plasma shutdown. Independent CLOSED: errors[], empty cgroup, pinned
processes gone, actor no-held-keys true, VT1, seat inactive, policy absent.
Do not promote that outer failure to PASS. Artifact archive SHA256
154f3c16c4986360487b224ef837e52f0f4fcfb6214df6250750cca160607844;
native receipt SHA2565ffc6d471b3e9c35e65b763e4dda38aac1f94a8701e57cde6210401c4eaf3dcd,
under `/home/ubu/.cache/lay/development/browser-final-fixes-20261018/kde-provider-native-b/evidence/`.
431 cells unselected; old KDE75P333F24B/GNOME354P46F32B unchanged. The TD158
explicit decision precedes runtime edits. No install, publication, model or
delivery-route change.

### 2026-10-18T14:34:00Z — TD158 transaction implementation and first source feedback

Decision `2026-10-18-kde-exact-replay-layout-handoff.json` preceded runtime
edits. The existing exact-replay handoff now captures its GNOME or KDE backend.
The KDE transaction checks fresh keyboard/KWin owner, complete bounded layout
mapping, exact window, group readback, selected IBus engine and actual Lay mode.
It keeps the existing replay admission, edit executor and settlement owners;
there is one bounded restoration through the captured backend, never an edit
retry. Initial fixed-source/decoder divergence still refuses before mutation.
Ordinary KDE switching and GNOME activation were not migrated.

First daemon-only source receipt: `/home/ubu/.cache/lay/development/run-folb1717/RESULT.json`,
317 selected, 313 PASS, 4 FAIL, zero unrelated failures. The four new test
failures exposed an ambient-backend dependency: `target_layout(...).1` selects
non-IME XKB aliases under the hermetic daemon harness. The exact IME transaction
now uses a small canonical Lay-engine alias helper shared with the existing
IME branch, independently of that ambient choice. New tests still exercise
the production transaction, not a reimplementation. This receipt is not PASS.

Independent review pass1: 8/10, one P2: unavailable VisibleTailV3 replies use
false as a sentinel and have no real engine path. Blindly projecting the bool
could falsely verify the US mode or restoration. Mode readback now rejects
missing/root/invalid object paths, while retaining private redacted actual
decoder replies with a real path. A focused regression covers both sentinel
refusal and actual RU/US private decoders. Review record:
`/home/ubu/.cache/lay/development/browser-final-fixes-20261018/kde-prepare/TD158-REVIEW-PASS1.json`.

Corrected source proof and controlled missing-decoder-readback RED are pending.
No changed-candidate native acceptance, latency, install or publication.
Historical KDE75P333F24B and GNOME354P46F32B remain unchanged.

### 2026-10-18T14:36:03Z — TD158 corrected daemon source proof and causal RED

Daemon-only development gate: **318 selected, 318 PASS, zero FAIL**,
169.3 seconds, `/home/ubu/.cache/lay/development/run-0p_4fpyh/RESULT.json`, frozen archive
`e7e94800333bf01ce581f593bb7846de376607b06fd03e0b67bcaeaa4e4f8c79`. This is a focused source gate, not full closure/native.

The eight production-transaction regressions and actual-mode sentinel regression
pass (9/9). Controlled remote-only removal of actual decoder readback makes
`engine_name_success_does_not_hide_failed_decoder_readback` fail at its intended
assertion: an engine-name success cannot certify the decoder. Original source
bytes were restored and hashes match. Receipts `/home/ubu/.cache/lay/development/browser-final-fixes-20261018/kde-prepare/handoff-causal/GREEN.json`, `RED.json`,
`RESULT.json`. This diagnoses an invariant; no runtime authority changed.

Generated graph, updated manifest, full affected source proof and second/final
independent review remain pending before a new private build/native witness.
Historical native matrices remain unchanged; no install/publication.

### 2026-10-18T14:37:26Z — TD158 final source review and generated graph

Second and final independent read-only review: **10/10**, no remaining P1/P2.
Pass1 sentinel projection is resolved; the canonical engine helper preserves
existing GNOME aliases. Review record `/home/ubu/.cache/lay/development/browser-final-fixes-20261018/kde-prepare/TD158-REVIEW-PASS2.json`.
This review establishes source findings only. Required native evidence remains
both directions/next-letter decoder, refusal/restoration, speed and full matrices.

Generated graph/architecture refresh PASS:
`/home/ubu/.cache/lay/development/tab-graph-q09plif7/RESULT.json`; six generated
files fetched only after source-stability verification. Manifest refresh/full
affected source gate pending. No native totals, installed bytes or authority
changed by this source/generation step.

### 2026-10-18T14:50:57Z — TD158 full gate refused stale zero-failure inventory binding

Full development receipt `/home/ubu/.cache/lay/development/run-8pocyh0d/RESULT.json`
is **FAIL**, 513.2 seconds: all 33 test targets reported zero assertion
failures across 3248 executed rows, then the strict contract rejected
`known-failure manifest is bound to a different test manifest`. No complete
source PASS is claimed. The refreshed 3274-test inventory added the TD158
regressions; its zero-known-failure ledger still named the preceding inventory.
Only `known_failures.json.manifest_sha256` is rebound to the actual manifest;
zero failure rows and the existing observation bytes remain unchanged. The
production loader validates the complete ledger. No guard was weakened and
no failure was accepted. Receipt:
`/home/ubu/.cache/lay/development/browser-final-fixes-20261018/kde-prepare/TD158-known-failures-binding.json`. A fresh complete gate is required.

The next isolated native harness is prepared and pinned, not run. Its owned
Plasma shutdown pins same-UID/same-cgroup non-ancestor processes and the private
DBus broker, signals only verified owned descendants with bounded TERM/KILL,
and preserves resource-guard failure for survivors. Four controls cover scope,
PID reuse, cgroup moves and survivors; nine existing actor controls also PASS.
Original actor/resource errors and cleanup errors remain separate. These 13
source controls do not prove native cleanup or change any acceptance totals.
No runtime input owner, IME delivery route, resource ceiling, installation or
publication changed. Historical KDE75P333F24B and GNOME354P46F32B remain.

### 2026-10-18T14:59:37Z — TD158 full affected source closure PASS3248; native pending

Canonical development gate **PASS3248/3248**, zero actual/known semantic or
infrastructure failures, 471.1 seconds in the source receipt.
Receipt `/home/ubu/.cache/lay/development/run-3b5ymuu_/RESULT.json`; remote `run-1b9UOG`,
frozen archive `70db3455a142e5b189a4904a19f72cfd62da982ae90d180f22775158ad1b57eb`. This supersedes the
failed stale-inventory run as source proof, without changing that original FAIL.
The zero-known-failure contract remains strict. Focused318/318, nine transaction/
readback checks, causal missing-decoder-readback RED/restoration, graph refresh
and final independent review10/10 remain recorded above. No third review.

Next: build this frozen candidate privately, verify executable hashes, then run
one owned GTK Entry RU→US→RU witness including the following key and caret.
Successful/refused timing and native cleanup are measured separately. This
source gate establishes neither installed behavior nor physical client PASS.
Runtime input/text delivery authority is unchanged; no installation/publication.
Historical KDE75P333F24B and GNOME354P46F32B remain the latest complete matrices.

### 2026-10-18T15:02:09Z — TD158 private build stopped by visibility lint; declaration-only repair

Private build stage `td158-handoff-build-20261018-a` is FAIL before release
compilation: `scripts/check-lay-lints.sh` rejects private_interfaces/private_bounds
on the new captured KDE handoff types. Source PASS3248 remains evidence for
the prior source bytes, not a successful private build. Receipt/log:
`/home/ubu/.cache/lay/development/browser-final-fixes-20261018/build-kde-handoff/BUILD-RUN.log`,
`CHECK.log`. No candidate was installed or tested natively.

Four declarations in private `kde_exact` receive crate visibility: Snapshot,
Io, Session and Handoff. Fields and the child module remain private; no runtime
statement, output route, authorization or restoration behavior changes. Inverse
substitution restores the exact preceding file bytes. Record:
`/home/ubu/.cache/lay/development/browser-final-fixes-20261018/kde-prepare/TD158-visibility-repair.json`. Strict lint, graph/source
binding and fresh affected source proof are required for the repaired bytes.
This is a declaration repair within the existing TD158 decision and reviewed
transaction, not a third functional implementation/review pass or a rule waiver.
Historical matrices remain unchanged.

### 2026-10-18T15:06:41Z — TD158 declaration repair passes strict lint and refreshed graph

Guarded immutable-snapshot command runs `scripts/check-lay-lints.sh` followed
by `scripts/update-architecture-graph.sh`; both PASS. Six generated outputs are
fetched only after source-stability/hash checks. Receipt `/home/ubu/.cache/lay/development/tab-graph-ccthuvu1/RESULT.json`,
source archive `387b20746628a76c99a488ec979181d167b1ea5927cc61c758cc0517854222be`. No warning
allowlist, compiler flag, runtime body or authority rule was weakened.

Fresh complete affected source execution is active at
`/home/ubu/.cache/lay/development/run-0z2k24yk`; no result is claimed yet.
The next private stage is `td158-handoff-build-20261018-b`, followed by the
fresh pinned owned native stage `td158-kde-handoff-native-20261018b`. Thirteen
unchanged actor/cleanup controls PASS; only stage/seat/policy/build paths differ
from prepared-unused stage A. Native input has not begun. Earlier failure and
unexecuted artifacts remain auditable. Historical complete matrices unchanged.

### 2026-10-18T15:13:57Z — TD158 repaired declarations pass the complete source gate

Fresh immutable-snapshot gate PASS3248/3248, zero actual/known semantic or
infrastructure failures; all three canonical command return codes are zero.
Receipt `/home/ubu/.cache/lay/development/run-0z2k24yk/RESULT.json`, remote
`run-7IjTVP`, archive
`4675521392446586c8c1c5c41fd1b4c895515417fe4a387471b5e887bfafb0a6`.
Receipt elapsed 496.2 seconds; orchestration 507.4 seconds. This proof includes
the four visibility declarations and follows strict lint/graph PASS; the
preceding failed private build remains a failure. No source body or authority
change followed this snapshot. Next: private build B from these exact tested
inputs, then one native GTK Entry roundtrip with following-key decoder checks.
KDE75P333F24B and GNOME354P46F32B remain the latest complete native results.
No installation, publication, text-delivery migration or native PASS claimed.

### 2026-10-18T15:18:26Z — TD158 private build B verified; native witness started

Private build B passes the remaining canonical gates and produces version
1.0.90 in 188.7 seconds, without installing it. Receipt
`/home/ubu/.cache/lay/development/browser-final-fixes-20261018/build-kde-handoff-b/BUILD.json`;
frozen build archive `a0697c071e66f006519f34e90535d7366484937bd394aee06180b8e0a03c9b7b`
is bound to full source PASS3248 `run-0z2k24yk`. Local fetched bytes match the
remote receipt: daemon `1da74fb6054f5e4964194efb710799ada980662da7bce8e4a0d5fbf21585a222`,
IME `5d84ca28241054495cfef1ecf2a4ececb09d69d097a25be20258651ed7a9d47d`,
CLI `f8aec338504dadd72bd06fa78f4b3e8bd1863376897fce95dd3081e2d2e0c3c9`.
Dependency lock parity and zero-vulnerability audit pass. This is source/build
proof only. The pinned stage `td158-kde-handoff-native-20261018b` has begun
one GTK Entry roundtrip in its owned finite VT8 seat; no native outcome yet.
It must check both directions, following-key decoding, caret, selection,
release of held keys and independent closure. Old matrices remain unchanged.

### 2026-10-18T15:21:23Z — TD158 native handoff works; emitted text is not observed

Pinned native B completes its orchestration and independent closure, but the
single GTK Entry roundtrip is FAIL at its first `ghbdtn` target. Seed is actual
committed `привет`, caret6, no selection/preedit; physical Double Shift reaches
the exact replay. KDE group, selected IBus engine and actual Lay mode become
US, and the daemon reports six Backspaces plus replacement `ghbdtn` emitted.
The actual GTK widget and AT-SPI still contain `привет`; no replacement key
arrives in the IME trace. Emission/logged completion is not client acceptance.
No successful latency or both-directions/following-key proof exists.

Receipt `kde-handoff-native-b/evidence/native-windows-8bw9ubig/RESULT.json`
under `/home/ubu/.cache/lay/development/browser-final-fixes-20261018`, SHA256
`3db5236f95637216826950df5e8dbed5ab04d2e3f88cde4ab178d2eefc0cea99`;
34-file evidence archive
`9f122747efd2a8aa84c85a9f5a3d40843d560158f2e893d9f41bbbedf4b11dc6`.
Actor22keys, no held keys, one failed cell/431 unselected. Independent CLOSED
2026-10-18T15:18:33Z: no errors, empty cgroup, pinned processes gone, VT1,
seat stopped and policy removed. Owned descendant cleanup completes within
the unchanged guard. Historical full matrices and installed runtime unchanged.

Next diagnostic uses the same binaries and exact actor, adding only a bounded
read-only observation of the newly created daemon virtual keyboard: kernel
events, udev properties, access and KWin input descriptors. Its setup must bind
the unique new device to the pinned owned daemon and wait for observation
readiness before the original case. No event is injected through the observer,
no runtime edit is proposed before locating the lost transition, and no full
matrix is started from this failed witness.

### 2026-10-18T15:32:29Z — TD159 kernel emission and sandbox hotplug cause qualified

Same build-B bytes and one-cell actor complete another finite native diagnostic
with independent CLOSED and unchanged limits. Read-only observer captures the
unique daemon-created `/dev/input/event12`: correct keyboard udev properties,
six Backspace press/release pairs followed by G/H/B/D/T/N pairs, 24 EV_KEY and
12 EV_SYN records in 38.19 ms. KWin descriptors never include that event node.
Thus emitted replay is lost at new device discovery/open, before IME input.
Receipt `kde-replay-device-native-a/evidence/PASSIVE-DEVICE-RESULT.json` under
the TD-156 cache; archive SHA256
`a6c85e842a079805eeebd296fd5b74aba868f2ef4b884d56fbeecf10826da328`.
One failed cell/431 unselected is diagnostic scope, not a new complete grid.

A separate zero-key, single-device three-context probe causally isolates the
test sandbox's user namespace. Host UID1000 and root-mounted bwrap followed
by `setpriv` to UID1000 both receive two libudev add events with sender UID0.
Unprivileged bwrap in its private user namespace receives both raw netlink
messages with mapped sender UID65534, but libudev receives zero events. All
workers have UID/EUID1000 and zero permitted/effective/inheritable/ambient
capabilities. The successful sandbox retains the host user namespace. Receipt
`kde-prepare/hotplug-scope-a/RESULT.json`: CAUSAL_ENVIRONMENT_WITNESS_PASS,
zero keys, disposable device closed and guarded workers terminated.
This agrees with systemd v249 `device-monitor.c` rejecting nonzero sender UID
and bwrap v0.6.1 creating a user namespace for non-setuid unprivileged launch.
Primary code: <https://github.com/systemd/systemd/blob/v249/src/libsystemd/sd-device/device-monitor.c>,
<https://github.com/containers/bubblewrap/blob/v0.6.1/bubblewrap.c>.

Next change is test-launch-only: keep identical private filesystem/data mounts,
use privileged bwrap solely for mount setup, then drop the entire private
session to UID/GID1000 with zero permitted/effective/inheritable/ambient
capabilities before DBus/KWin/apps. Record and assert actual early-session
identity, namespace and capabilities. Reuse exact build B, actor, case targets,
deadlines, passive device observer and cleanup; require previous CLOSED before
opening a fresh owned seat. No runtime edit, authority change or resource-limit
increase. Native acceptance remains pending; no claim that all 333 failures
share this cause. Historical GNOME354/46/32 and KDE75/333/24 remain unchanged.

### 2026-10-18T15:37:28Z — host-namespace launcher access control qualified without keys

First host-namespace stage A aborts before Python can enter `kde-session.py`:
bwrap creates temporary ancestor directories owned by root and not searchable
by UID1000. No KWin/client/actor or key input starts. `RUN.json` preserves
ABORTED_CLEANUP_FAILED because the readiness SSH worker outlasted its 50-second
join; independent CLOSED still confirms the owned seat inactive, VT1 and no
temporary policy. Session/cgroup/held-key fields are null, not claimed PASS.
Sources and logs remain at `kde-replay-hostns-native-a` in the TD-156 cache.

Zero-key `kde-prepare/hostns-path-probe.json` reproduces the inaccessible
ancestors under the actual full mount list, then passes after `--chmod 0755`
only on temporary `/home` ancestor directories, never mounted user data.
Applications still run as UID/EUID1000 in the host user namespace. Fresh stage
B preserves every actor/case/deadline/binary, adds this directory access fix,
requires prior stage-A CLOSED and no old orchestration worker, and passes all
13 existing controls locally. Its real pre-DBus environment assertion remains
mandatory. No native result or matrix replacement yet.

### 2026-10-18T15:40:04Z — temporary socket-directory prerequisite qualified

Stage B reaches the unprivileged Python session but aborts before KWin or keys:
private `/tmp` is root-owned0755. Complete orchestration and independent CLOSED
confirm empty guard cgroup, stopped owned seat, VT1 and removed policy. Null
actor/pin fields remain null; no native behavior is graded. Fourteen-file
archive SHA256 `5c5b038ace2f43be0e255d3b8d77297c03b1af7b907be8241dd8d2d062d10559`
at `kde-replay-hostns-native-b/evidence.tar.gz` preserves this startup failure.

Zero-key actual-mount probe `kde-prepare/hostns-path-probe-b.json` confirms
UID1000 cannot create temporary files/sockets in0755, and can after setting
only the private tmpfs `/tmp` to standard1777. Stage C adds that mount option
and verifies real temporary-file/socket creation plus access to every private
session directory before DBus or desktop startup. UID, host namespace and
zero-capability assertions remain. Fourteen scripts frozen; all13 unchanged
controls PASS locally and remotely. Prior closures are required. Same candidate,
actor, target and timing limits; native result pending, historical totals unchanged.

### 2026-10-18T15:40:47Z — TD158 exact roundtrip accepted in corrected KDE environment

Fresh stage C using unchanged build B passes the original GTK3 Entry roundtrip:
actual `привет → ghbdtn → привет`, following native Z decodes `z` then `я`,
Backspace restores each target, exact caret/no-selection targets all reached.
KWin now opens daemon virtual keyboard event12 before its first event; passive
observer captures72 records for both replays. UID/GID1000, host user namespace,
zero permitted/effective/inheritable/ambient capabilities and temporary file/
socket access are measured before DBus. This qualifies the environmental repair
and the reached foreground/layout/replay/client path; no runtime text route
was changed by the harness fix.

Primary receipt `kde-replay-hostns-native-c/evidence/native-windows-2n4bwqcb/RESULT.json`
under the TD-156 cache, SHA256
`668e2b4f24165f23fe1f3c832ee8aad9ea1940cabaec3df23621d775b36a1f82`;
38-file archive SHA256
`1271cc808d125478eda822aa0cfbdcf443291b689d5358472c011302b901dc1c`.
Two successful Double Shift samples135.57/140.31ms, median137.94ms, p95/max
140.31ms. Definition: actor evdev write return to exact native target observation;
VKMS/software desktop, n2, not physical hardware latency or a broad benchmark.
One PASS/431 unselected. Independent CLOSED15:40:24Z: empty cgroup, original
pins gone, no held keys, VT1, owned seat inactive, policy absent, no errors.

Next expand this same candidate and qualified environment to all432 original
KDE cells. Separate finite per-application owned sessions may bound work under
the unchanged resource ceiling, but final aggregation must require all six
receipts with identical candidate/config/model pins, exact432 unique identities,
no unselected cells and independently CLOSED seats. GNOME shared-code regression
and focused foreground lifecycle controls remain pending. Historical complete
GNOME354/46/32 and KDE75/333/24 are retained until fresh complete receipts exist.

### 2026-10-18 15:48:32 UTC — complete KDE matrix D running

Started at15:45:34UTC from the unchanged private build B and the causally
qualified H5C environment. The frozen plan contains exactly432 distinct original
KDE case identities across GTK32, Qt48, KWrite16, Konsole16, Firefox160 and
Chrome160. Six finite owned sessions run sequentially under the existing resource
ceiling. All78 local and78 remote pre-input harness controls passed. No native
target, actor key cadence or deadline changed; the bounded hotplug observer is
omitted from this longer acceptance run after the separate causal qualification.

Each app receipt must contain every required cell, pinned identical binaries,
configuration, model files and initial learning data, and independent CLOSED
evidence (empty cgroup, pinned processes gone, no held keys, policy absent, VT1).
The aggregator refuses incomplete, duplicate or unselected coverage. At this
checkpoint GTK is20PASS/10FAIL/2BLOCKED, Qt30/15/3; these are partial evidence and
do not replace KDE75/333/24. No runtime installation, publication or authority
change occurred. Shared GNOME regression and foreground lifecycle checks remain
pending. Plan, progress, frozen controls and forthcoming aggregate receipt:
`/home/ubu/.cache/lay/development/browser-final-fixes-20261018/kde-functional-matrix-d/`.

### 2026-10-18 15:58:30 UTC — matrix D partial; bounded resource qualification

Matrix D stopped at15:49:58UTC without a new complete432-cell receipt.
Four independently CLOSED phases retain112 actual results: GTK20/10/2,
Qt30/15/3, KWrite3/11/2, Konsole3/10/3 (PASS/FAIL/BLOCKED), total56/46/10.
Firefox reached its first cell but fork failed with EAGAIN at TasksMax512;
the actor's cleanup thread also could not start. Primary native receipt is
ABORTED_CLEANUP_FAILED, with16 sent events and no held keys. Independent cleanup
then proves CLOSED, empty cgroup, original pins gone, VT1, expired policy absent.
Chrome was not run. These phases do not replace the historical KDE75/333/24.
The saved Firefox archive SHA256 is
`34f2e8f83e908cb6c7d3ff383e7556549e18996027f3a41d811653c4d9e7ac0b`;
progress, primary receipts and closure evidence remain in matrix D above.

Zero-key resource probe A independently reproduces the resource limit in36
samples (279–512 tasks, pids.events max18). At the peak Firefox accounts for244
threads, Plasma46, kded24; unrelated GNOME backup/calendar/indexing services
also start inside the private KDE profile. No keys were sent and the owned
session is independently CLOSED. Local result:
`/home/ubu/.cache/lay/development/browser-final-fixes-20261018/kde-load-probe-a/RUN.json`;
remote inventory: `/home/e/projects/lay-development-runner/kdlp-a/RESOURCE-RESULT.json`.

Probe B is running as a zero-key qualification, not native acceptance. It uses
short private socket paths, privately overrides only DejaDup, Evolution alarm
and Tracker autostart entries with Hidden=true, and sets LP_NUM_THREADS=4 for
the software renderer. This is test-session configuration only: no host/user
autostart or browser preference, binary, model, input route or acceptance target
changes; TasksMax512 remains enforced. Qualification requires no limit events
and at least64 task slots free during the finite observation. Renderer timing
is environment-specific and must not be presented as a production speedup.
Probe B result is still pending; GNOME regression and foreground lifecycle
controls remain pending. No installation, publication or runtime authority
promotion occurred.

### 2026-10-18 16:04:59 UTC — resource cause qualified; browser phases E running

All four zero-key resource probes are independently CLOSED. Probe B, with
LP_NUM_THREADS4 and three private autostart exclusions, avoids limit events but
peaks at461 tasks and fails the declared64-slot headroom requirement. Probe C
with LP_NUM_THREADS1 peaks at456 and fails the same requirement. Their failures
are retained, not reclassified. Native acceptance was not tested by these probes.

Probe D additionally disables Firefox's spare content-process prelaunch only
in the disposable profile: `dom.ipc.processPrelaunch.enabled=false`. The browser
retains multiprocess isolation and its ordinary IME/accessibility settings.
The [Mozilla implementation](https://searchfox.org/firefox-main/source/dom/ipc/PreallocatedProcessManager.cpp)
owns that preference; this change bounds idle speculative workers, not the
active field's input route. Probe D passes unchanged qualification at377/512
peak tasks, pids.events max0, zero keys, no held keys and independent CLOSED
at16:01:11UTC. These renderer/browser settings qualify only this test environment;
latencies are not directly comparable to the old environment or user hardware.
All raw finite resource inventories are now saved locally alongside RUN.json
under `kde-load-probe-{a,b,c,d}/` in the TD156 development cache.

Matrix E started16:01:49UTC. It revalidates the112 closed D cells without new
input and runs the remaining320 original browser identities in fresh owned
sessions with short private socket paths and the qualified resource profile.
All26 local and26 remote harness controls pass. Actor case bodies, targets,
deadlines and cadence are byte-identical after only run-path substitution;
candidate/configuration/models/initial learning seeds must match all six phases.
No complete matrix is claimed while either browser is unfinished. Historical
KDE75/333/24 and GNOME354/46/32 remain the last complete receipts at this entry.
The same build's exact GNOME216+216 regression is prepared locally and will
start only after both KDE seats close. Foreground lifecycle controls remain
pending. No user installation, release or runtime text-route change occurred.

### 2026-10-18 16:13:06 UTC — complete KDE improvement:149/259/24; same-build GNOME running

Matrix E is COMPLETE at16:12:19UTC:432 original unique application/field/plan
identities,149PASS/259FAIL/24BLOCKED, zero NOT_TESTED. All six phases have
independent CLOSED evidence (empty owned cgroup, pinned processes gone, no held
keys, VT1 and temporary policy removed). Candidate, configuration, models and
initial learning seeds match; no acceptance target, deadline or cadence changed.
The four native-app phases reuse the closed D receipts. The two browsers use
the qualified short-path/private resource environment described above; this is
a cumulative six-session result, not a simultaneous screen or hardware test.

| Application | Previous PASS/FAIL/BLOCKED | Current PASS/FAIL/BLOCKED | Total |
| --- | ---: | ---: | ---: |
| GTK3 | 8/22/2 | 20/10/2 | 32 |
| Qt | 12/33/3 | 30/15/3 | 48 |
| Firefox | 28/125/7 | 72/81/7 | 160 |
| Chrome | 21/132/7 | 21/132/7 | 160 |
| KWrite | 3/11/2 | 3/11/2 | 16 |
| Konsole | 3/10/3 | 3/10/3 | 16 |
| **Total** | **75/333/24** | **149/259/24** | **432** |

Exact comparison rechecks all old primary receipt hashes and matches every
identity. Transitions:74FAIL→PASS,259FAIL→FAIL,75PASS→PASS,24BLOCKED→BLOCKED.
There are no PASS→FAIL transitions in this comparison. This22.22% reduction in
failed cells reflects the new candidate AND corrected test environment; it is
not an isolated effect estimate for one production patch. Historical receipts
remain unchanged. Build B is still private archive
`a0697c071e66f006519f34e90535d7366484937bd394aee06180b8e0a03c9b7b`,
with3248/3248 source checks; no model fit or user installation occurred.

Complete receipt:
`/home/ubu/.cache/lay/development/browser-final-fixes-20261018/kde-functional-matrix-e/RESULT.json`,
SHA256`d09b8ce737358c5f4b7ca62d9d3a4de613f006e658db6fac055a16043949dc70`.
`BASELINE-COMPARISON.json` beside it retains all74 changed cells, per-app timings
and observed actor-stage groups. Firefox primary receipt
`firefox/evidence/native-windows-lts0991w/RESULT.json`,
SHA256`7a0ee344061422c3db4c58cda43c7338bc7bd19411da41c013e90003e5c6f4a6`,
archive`45e557efa66b720e6b07e938a1ffa1db3f72665ad6c075215d52c822f2ab220d`.
Chrome primary receipt`chrome/evidence/native-windows-3sh13vn3/RESULT.json`,
SHA256`3c45725325ca7487970be7ce713215548b12aa1caa30b000331c358a4ec3788a`,
archive`a79d9fda6c39593e612f1d25d646c5f2459e812c1537674ee4ac829cfc58cdc7`.

Remaining259 FAIL group by last observed actor step as90clear,44seed,74target
after Double Shift and51target after Space. These are not four established root
causes. Rich fields7/8/10 still retain raw AT-SPI`\n`, caret0; no normalization
or verdict change was introduced. Chrome's archived trace records43manual-toggle RPC
refusals at`context_authority`, with live bridge token but manual-toggle and
manual-refresh admission false; its activation installations report unknown
start. This is a newly exposed admission boundary, not proof all browser cases
share it. KWrite/Konsole have the separately recorded related admission refusal.

The separate Space/layout source gap remains:IME`layout_sync.rs`'
`switch_complete_layout_stack` calls`activate_gnome_layout_for_ime` without a KDE
branch. Correct native text with wrong next-letter decoder still FAILs the
conjunctive target. Any repair must keep the existing reducer's layout-intent
token and owner; the exact-replay handoff does not automatically repair Space.

Per-phase successful-operation measurements (also includes operations preceding
a later failed cell):GTK Double Shift n36, median136.10ms,p95168.53ms;Qt n54,
137.52/154.68ms;Firefox n130,161.63/173.14ms. Chrome/KWrite/Konsole have no
successful measured Double Shift, so no median/p95 exists. These are evdev-write
return to native observed target in VKMS software-rendered sessions, not kernel
release or physical-machine latency. Environment differences preclude a claimed
production speedup from these figures. The combined result additionally exposes
PASS-cell-only timings with its different explicit scope.

The same-build GNOME216+216 regression started16:12:25UTC only after KDE closed.
Its verified staging backs up the previous private runner bytes and changes
only that private candidate. Original job bodies, configuration and frozen
learning seeds are retained. Evidence root:
`/home/ubu/.cache/lay/development/browser-final-fixes-20261018/gnome-kde-regression-e/`.
Its results are pending; the previous GNOME354/46/32 certifies older bytes only.
Native foreground lifecycle controls also remain pending. TD157/158/159 and
overall KDE acceptance remain open; neither publication nor runtime authority
promotion follows from the improved aggregate.

### 2026-10-18 16:17:30 UTC — remaining admission boundary needs key-ingress qualification

Read-only parsing of archived native traces now preserves exact hashes and
member counts in matrix E's`ADMISSION-TRACE-OBSERVATIONS.json`. Firefox has157
observed ProcessKeyEvent calls and156 legacy key callback entries. Chrome,
KWrite and Konsole have zero such entries in their archived intervals, despite
FocusInId and activation publications. Their installed activations remain
unknown-start (77,17,17 respectively); archived manual context-authority
refusals number43,8,7. The trace observer explicitly includes ProcessKeyEvent
and ProcessKeyEventAtomicV1 in its callback-member set. No inference from
`publication.context_path=null` is valid:the separately logged context replies
do contain accepted real context paths.

This is not yet a wire-level proof that the client sent no key, nor proof the
reducer should authorize an edit. Before altering admission, qualify one real
stock-client letter and bind client/IBus/engine process and transport identity.
Keep lifecycle/acquisition, key ingress and known-start word authority distinct.
TD156 now records this first evidence step, the independently confirmed Space
backend gap and rich-field clear qualification as three separate minimal
follow-ups. No runtime code, source test, native verdict or authority changed
in this read-only analysis. Same-build GNOME regression is still running.

### 2026-10-18 16:20:01 UTC — GNOME fullscreen original-cell failure captured

The in-progress same-build regression retains a FAIL in the original Firefox
standard/fullscreen/active/field8 cell. First ordinary RU→US Double Shift,
following letter and Backspace pass; the second ordinary US→RU gesture leaves
actual`ghbdtn`, caret6, focused and unselected, instead of`привет`. The other
later gestures in this cell remain untested rather than being counted as PASS.
The normal/private standard jobs are20PASS each; full browser totals are pending.

The existing harness captured a frozen, SHA-verified trace without extra typing:
`gnome-kde-regression-e/live-failure-capture/firefox-fullscreen-active-8.jsonl`
under the TD156 cache, SHA256
`f40cb89d46ffaf7a02734d647362b638e61ae45de1af7d3ae6e7d6f8beddb49a`.
Original primary receipt is remote runner
`firefox-standard-fullscreen-1185w8n1/RESULT.json`; its trace-completeness field
remains UNKNOWN. A read-only daemon snapshot and`OBSERVATION.json` are preserved
beside the local trace.

The first recorded refusal is bridge admission at nonce2276. Marker diagnostic
reports`unsettled_callbacks`, idle reducer, one unsettled callback, current
owner/activation624, fence present and marker observed. The bridge is taken
with17,634us left; manual-toggle rejects at`bridge_admission`. A release
ProcessKeyEvent then enters and settles as accepted. This observation supports
an admission-ordering follow-up; it does not establish that the new KDE provider
caused the native change, that the layout handoff was reached, or that any guard
should be relaxed. Do not increase the deadline, retry the original cell until
it passes or discard the FAIL. Full exact comparison against preceding GNOME
bytes is pending and must retain this cell even if another cell improves.

### 2026-10-18 16:29:30 UTC — source qualifies early bridge refusal, not its regression attribution

Read-only inspection of the current adapter shows the exact terminal step:
`ContextAdmissionObserver::process_marker` samples the reducer's bridge token
once, then sets the pending bridge `ready=true` even for a `None` token.
`wait_for_fence` returns on that flag; `finish_bridge_fence` takes the negative
result and returns Denied. Accepted `settle_key_callback` refreshes acquisition
readiness only. Thus the original fullscreen trace's remaining17.634ms was not
spent awaiting the already observed callback; the immediate negative result
is explained by the current source. This does not identify which production
change caused the observed timing, nor establish that delayed settlement may
legitimately authorize every such bridge.

TD156 records the minimal controlled interleaving and its stale-owner, changed
activation, post-marker input, cancellation and original-deadline controls.
The existing pending fence/reducer must remain the sole owner. No runtime
change, timeout increase, native repeat or verdict change was made here.
Firefox's complete same-build GNOME result is175PASS/22FAIL/19BLOCKED of216;
Chrome is still running. Prefix's failure window is missing from its capped
trace, so it is not attributed to this fullscreen ordering observation.

### 2026-10-18 16:40:06 UTC — same-build GNOME complete; retain two new failed cells, no promotion

The original216+216 GNOME protocol is COMPLETE at16:38:39UTC, zero NOT_TESTED.
Both finite native sessions have independent CLOSED receipts:Firefox at
16:25:37UTC and Chrome at16:38:37UTC, empty owned cgroups, pinned processes gone,
no held keys, VT1 restored and the exact temporary seat units stopped. The
private candidate is the same three build-B binaries as KDE matrix E, archive
`a0697c071e66f006519f34e90535d7366484937bd394aee06180b8e0a03c9b7b`.
Original job bodies, harness hashes, configuration and frozen learning seeds
are retained. No additional typing, repeated failing cell or acceptance-target
change was used to produce this complete result.

| GNOME browser | Preceding NBSP bytes PASS/FAIL/BLOCKED | Build B PASS/FAIL/BLOCKED | Total |
| --- | ---: | ---: | ---: |
| Firefox | 176/21/19 | 175/22/19 | 216 |
| Chrome | 178/25/13 | 178/25/13 | 216 |
| **Total** | **354/46/32** | **353/47/32** | **432** |

Exact comparison revalidates18 primary receipt hashes, configuration and harness
binding, and all432 original identities against the immediately preceding
complete NBSP candidate, rather than only the first baseline. Firefox retains
174PASS→PASS,20FAIL→FAIL,19BLOCKED→BLOCKED,2PASS→FAIL and1FAIL→PASS.
The two new failed cells are standard/fullscreen/active/field8 and
functional/normal/prefix/field1. The phone field4/short cell changes FAIL→PASS.
Chrome retains all178PASS,25FAIL,13BLOCKED, with no changed verdicts.
The full comparison is not a passing no-regression verdict. The two failed
cells remain open even though their causal attribution to a changed source
mechanism has not been established. Fullscreen's first bridge refusal and the
terminal negative-marker source behavior are qualified above. Prefix's capped
trace lacks its failure interval; do not assign it the fullscreen cause.

Evidence root:
`/home/ubu/.cache/lay/development/browser-final-fixes-20261018/gnome-kde-regression-e/`.
Complete`RESULT.json` SHA256
`297a938228e96a66cb13af0fdd9d13a04d5551b4e0f1141477a5fbeaa6333810`;
`PRECEDING-COMPARISON.json` SHA256
`8edd4d15801431c65a8063fc4cc626ce847c25d765e77bbbcfbfdef1ffac5262`.
Firefox`evidence/full-e-firefox/SUMMARY.json` SHA256
`a69b96a7fc4cff8d696d1b05a5b9d58add8887f782c7cb49c34db5d057452fa7`;
Chrome`evidence/full-e-chrome/SUMMARY.json` SHA256
`d135800476062633ea99b63671c673e04b63e9e8a6944d48243ba3dfb4adf506`.
Each summary binds its nine primary receipts, binary/configuration identity,
failure/block reasons and independent closure.

Native latency is measured by the unchanged observer from evdev-write return
to the observed client target, not hardware key-release time. Samples include
successful operations preceding a later failed cell; missing operations are
not zero-latency successes. Required/measured coverage and milliseconds:

| Browser / operation | Measured / required | Missing | Median | p95 | Maximum |
| --- | ---: | ---: | ---: | ---: | ---: |
| Firefox standard Double Shift | 414/420 | 6 | 98.76 | 110.98 | 131.30 |
| Firefox panel Double Shift | 48/48 | 0 | 99.12 | 109.40 | 112.75 |
| Firefox functional Double Shift | 147/210 | 63 | 101.49 | 128.68 | 147.48 |
| Firefox Space | 37/50 | 13 | 18.43 | 24.25 | 24.60 |
| Firefox selection | 7/10 | 3 | 18.56 | 21.01 | 21.01 |
| Firefox Tab | 0/10 | 10 | undefined | undefined | undefined |
| Chrome standard Double Shift | 408/420 | 12 | 82.57 | 98.54 | 196.26 |
| Chrome panel Double Shift | 48/48 | 0 | 82.02 | 99.19 | 103.35 |
| Chrome functional Double Shift | 156/210 | 54 | 80.85 | 104.51 | 123.97 |
| Chrome Space | 39/50 | 11 | 18.80 | 24.63 | 24.69 |
| Chrome selection | 9/10 | 1 | 19.92 | 21.56 | 21.56 |
| Chrome Tab | 0/10 | 10 | undefined | undefined | undefined |

These finite remote software-session measurements do not establish a production
speedup. The comparison artifact retains the preceding measurements as well,
including changed coverage and maxima. KDE's separately qualified software
environment and different protocol cardinalities must not be pooled into a
GNOME browser latency claim.

Bounded stage verdict:KDE improves from75/333/24 to149/259/24, preserving every
prior KDE PASS; GNOME's complete same-build comparison retains two newly failed
Firefox cells. Source checks remain3248/3248 PASS for the frozen build, not
proof of native acceptance. Remaining work is recorded in TD156–159:qualify
stock-client key ingress, the existing Space intent's KDE backend, rich clear
observations, the bridge ordering boundary and native foreground lifecycle
controls. TD157/158 and complete KDE acceptance remain IN PROGRESS. No user
installation, model training, publication, runtime authority promotion or text
delivery-route change occurred. No later source edit was made to hide these
native verdicts.

### 2026-10-18T17:25:02Z — TD156: qualify stock Qt key ingress before another owner change

The user's next target is the retained259 KDE FAIL. One finite private KWrite
roundtrip uses unchanged frozen build B, original target/actions/deadlines and
the qualified host-user-namespace Plasma launcher. It adds a passive method
capture on the private IBus address and records the stock client identity.
This is a diagnostic supplement, not a replacement matrix or an admission
change. Exact scripts and pins:development cache `kde-key-ingress-f/SCRIPTS.json`.
The observed absence of an engine callback is not yet proof that a client
sent no method. Capture failure or unsupported monitoring remains UNKNOWN.
Existing authority guards, delivery routes, models and user runtime stay fixed.
The complete149/259/24 KDE and353/47/32 GNOME results remain authoritative
until new original-identity matrices are completed. Cleanup must independently
prove the finite session CLOSED, all owned processes gone and no held keys.
### 2026-10-18T17:34:35Z — TD156/159: key-ingress result and Space backend preflight

The finite ingress experiments used the unchanged private build B, not an
installed release. F retained a real KWrite roundtrip FAIL; its controller also
failed because the observer still expected the unselected cells. Its passive
monitor could not connect (`Abstract socket name too long`), so it establishes
no absence of client traffic. G corrected that selection assertion and used the
short private root `/home/e/projects/lay-development-runner/kig`, with the same
three binary hashes. G is COMPLETE_DIAGNOSTIC_ONLY: original KWrite roundtrip
PASS (both directions and next-letter probes); Chrome field 1 roundtrip FAIL.
The captured wire contains KWrite ProcessKeyEvent traffic but no such traffic
from a focused Chrome input context; during Chrome the engine sees IBus's
`fake` context. This is evidence to qualify Chrome's platform integration,
not permission to treat an unknown context as a valid word start. A short
private IBus address is an environment prerequisite, not a runtime patch.

Evidence: development/browser-final-fixes-20261018/kde-key-ingress-f/
td156-kde-ingress-kwrite-20261018f-evidence.tar.gz SHA256
`bf08612703f7d544d614933b9e8c90ad2ddd1daf59c20ee6364755307a9b6013`;
G's `kde-key-ingress-g/kig-evidence.tar.gz` SHA256
`b301d957cec6ebcbb17a68440358a058f0cef1a0e20ca1edd4b7f21784aafb82`.
Each archive has a separate per-file SHA manifest and primary native RESULT.
Both sessions are CLOSED, cgroups empty, pinned processes gone, parent keys
released, temporary policy removed and VT1 restored. F's actor key state is
unknown (null); G explicitly reports no held keys. Neither probe replaces the
complete KDE 149/259/24 of432 or GNOME 353/47/32 of432 receipts above.

**First failed production transition selected next:** corrected Space text and
decoder already reach their targets in GTK/Qt, while the KDE keyboard group
and selected IBus engine retain the source language. `layout_sync.rs` always
calls GNOME ActivateLayout. The existing request generation, LayoutIntentToken,
worker and conditional IBus completion already own this operation; no new text
executor or layout initiator is justified. The count51 labels Space failures,
not51 proven instances of this single root cause.

Design comparison: **9/10 selected**, a KDE adapter inside the existing layout
request, with backend captured from that engine's configuration, fresh typed
keyboard/KWin owner and bounded layout mapping/index, unique RU/US target,
one setLayout and exact readbacks. **4/10 rejected**, use the ordinary daemon's
OnceLock layout index: smaller edit, but stale mappings and its legacy fallback
cannot establish this transaction's postcondition. **6/10 deferred**, extract a
shared cross-binary backend crate now: viable long-term, but it expands the
change into the accepted daemon replay path before evidence justifies that
migration. A bypass or second layout worker is inadmissible.

Consequence analysis before production editing:

- Candidate/lattice retention, ranking, false lexical authority, models,
  package/delta reloads and feedback/learning are unchanged: this adapter is
  reached only after the existing authorized correction has committed text.
  It grants no word start, edit authority or lexical verdict.
- The request captures LayoutBackend once from the owning engine config.
  No global cache, queue, retry, polling or new generation is introduced.
  Every request reads the actual service and mapping; restart, remapping,
  duplicate target or independent group change refuses completion. Existing
  request-generation and context-token checks apply before backend mutation,
  before IBus completion and at completion. No lock is held across IPC.
- All KDE IPC shares one180ms deadline, including connection setup. Existing
  IBus read/request subprocess limits remain80/120ms; KDE also requires an
  engine readback after completion. The read/request/read sequence can delay a
  single poll beyond the async deadline by at most280ms of bounded work.
  The deadline is checked again after that poll. There is no per-RPC180ms
  multiplication, settle sleep or extra worker; native tail latency must be
  measured rather than inferred from these limits. The accepted mapping is
  limited to32 entries/8192 UTF-8 bytes after typed D-Bus decoding; zbus message
  decoding retains its existing allocation behavior. Package size adds code.
- Typed setLayout(false), wrong readback, owner/mapping changes, expired
  deadline or stale context return failure. A late result cannot promote a
  decoder or authorize text. No text replay/retry or blind layout rollback
  follows failure: a changed focus/service must not receive a restorative
  mutation. The already committed word remains subject to existing failure
  semantics; unresolved layout state is a real FAIL.
- GNOME activation and conditional IBus completion remain the existing route.
  KDE is selected only by the existing resolved backend. Niri/X11 are outside
  this change and retain prior behavior. Daemon replay is untouched. Possible
  regressions are choosing the wrong backend, stale in-flight engine completion,
  stricter deadline refusal and KDE mapping variants; tests must discriminate
  these. No future dictionary update changes this backend contract.
- Maintenance boundary: one private child adapter and ordered tests owned by
  layout_sync, using existing zbus/async-io dependencies. A later shared-backend
  extraction may remove duplicated typed observation code only with both
  consumer contracts proved; it is not a prerequisite for this repair.

Proof gate: ordered adapter tests and backend selection must detect the old
GNOME-only behavior or a recorded controlled violation; cover both directions,
already-target no-op, false set, missing/duplicate mapping, owner/remap changes,
stale generation/context before and after mutation, readback mismatch and
deadline. Run the full affected source scope, independent review (at most two
passes), guarded private build, native focused Space/next-letter controls then
all432 KDE cells and affected216+216 GNOME browser cells on matching bytes.
Keep every old FAIL and all denominator identities. No new native PASS,
performance claim, installation or publication exists at this preflight.

### 2026-10-18T17:39:11Z — TD159: discriminate missing XIM environment

G's captured READY env has DISPLAY=:0, GTK_IM_MODULE=ibus and QT_IM_MODULE=ibus,
but no XMODIFIERS. Before changing Chrome or IME authority, H selects exactly
one original Chrome field1 roundtrip on unchanged build B, same short private
root class, and adds only the standard private-session XMODIFIERS=@im=ibus.
No Ozone flag, IME route, target, deadline, model or user environment changes.
The passive IBus wire capture remains enabled. Both actor and observer select
the same one identity; the controller requires one result and closure. This
can discriminate an XIM environment prerequisite, not certify all Chrome
platforms or replace a complete matrix. Scripts are pinned under
development/browser-final-fixes-20261018/kde-key-ingress-h/SCRIPTS.json.

At2026-10-18T17:41:36Z H is ABORTED before setup/input: the copied prepare
script retained the literal `root/'kig'` and correctly refused its existing
directory. No new session was started; zero keys. Keep H/RUN.json as failure
evidence. I repeats this still-unanswered discriminator in a new `kii` root
after checking every script for old root literals and AST-checking both
one-cell selectors. Its separately pinned scripts are `kde-key-ingress-i`.
This is a corrected setup experiment, not a repeated semantic trial until PASS.
Focused IME source proof meanwhile passed848/848: development/run-uw1qir_u/
RESULT.json; it is not native acceptance or the full affected source gate.

### 2026-10-18T17:49:49Z — TD156/159: rejected XIM hypothesis and controlled RED

I completed its one original Chrome field1 roundtrip on unchanged build B:
FAIL, text remained `привет`, while the private engine reported unknown context.
The passive wire contains zero ProcessKeyEvent calls. Adding only private
XMODIFIERS therefore does not repair this observed failure; do not repeat it
or promote this environment hypothesis into runtime authority. This does not
establish the underlying Chrome platform/protocol cause. Primary native
receipt: `kde-key-ingress-i/evidence/native-windows-rjlh10_y/RESULT.json`, SHA256
`fe42d824c95b9b7f4e4e0bd722c10bc0af3798d8c0d192e3530645ae36e3f4e4`.
Archive `kde-key-ingress-i/kii-evidence.tar.gz`, SHA256
`6cc6863fe926aabd0356f3c827d039d2f01f6314aa8365c4d462a75b1813dd97`.
RUN.json records CLOSED, empty cgroup, pinned processes gone, no held actor
keys, policy absent and VT1 restored at17:42:16Z. This is one diagnostic cell,
not a replacement for either complete matrix.

The new backend-selection regression detects a controlled GNOME-only
violation: change only its production KDE dispatch condition to false in a
separate remote copy of the848-test source snapshot. Exactly one registered
test was executed and failed as expected (exit101):
`layout_sync::tests::kde_space_layout_stack_uses_its_captured_backend_in_both_directions`.
This is a counterfactual violation, not a build of an old checkout. Evidence:
`space-controlled-red/COLLECTED-RESULT.json`; original test.log SHA256
`6611675b328e78d7430b9ff47cfca64b1bceb79225fb37dcaef87319b58733e3`.
The original collector incorrectly required FAILED on the same output line as
the test name; nocapture interposed the assertion panic. Its run.log is
retained and the existing output was collected with the corrected parser,
without rerunning the test. The unmodified focused source passed848/848.
Full affected source validation is still running; code review, candidate
build and native acceptance of the new backend remain pending.

At2026-10-18T17:54:13Z the full source run `development/run-q3quljix/RESULT.json`
is FAIL after508.8s: executed target logs contain no semantic failures, but
the final contract rejects the old known_failures.json manifest binding. It
must remain a failed receipt. Comparing the frozen build B manifest to the
new manifest proves exactly11 added correctness identities, no removed tests
or lane changes (3274 to3285 total registered identities, including excluded
lanes). Rebind only that metadata hash to
`c51e330ae29cc3f35f92d78f8ae49d89ee96a50854df7eb5f6fd2d771ab499cb`.
The expected failure list remains empty, failure_count remains0 and the
historical observation bytes/hash remain unchanged. No test or guard is
weakened. The decision covers this binding update; rerun the complete gate
before review/build. This is a verification-metadata repair, not a runtime
repair round or a native result.

### 2026-10-18T17:59:11Z — TD156: exact next-letter prerequisite before full matrices

The frozen KDE Space case body ends after observing corrected committed text,
group, engine and mode; unlike ordinary Double Shift cases it does not type a
next-letter probe. Keep all432 original case bodies unchanged. Before the new
candidate's full matrix, select two existing Space directions in GTK3 Entry
and Qt5 RichTextEdit, in two separate finite sessions with short private IBus
addresses. After each original Space target, type physical Z once and require
the exact corrected surface plus `z`/`я` and target mode. Retain Space and
next-letter timing separately. Four supplemental cells, outside both complete
denominators, no repetition-until-PASS. A failed prerequisite stops promotion
to the complete matrix; retain its primary report and closure, diagnose its
first transition, then replan. This is physical native input through existing
routes, not DOM injection or a runtime exception. Scripts are prepared by
`prepare-kde-space-probes.py` and executed by `run-kde-space-probes.py` under
development/browser-final-fixes-20261018. Candidate build/review is still a
prerequisite; no probe has been executed at this preflight.

### 2026-10-18T18:03:05Z — TD156: complete affected source PASS, review pending

Full affected correctness/package gate passed3259 selected identities in481.3s:
`development/run-3upe8j3z/RESULT.json`, SHA256
`9c1a25031bd15f56a9cb0ffb64eaac5912b3ea16768b75950da3ace739018da2`.
Its source archive SHA256 is
`852cf3123f49bb7beba74d0c88a142a0fdec8adbfed1c23bd6d90126bb816ffd`;
remote run is `lay-development-runner/run-RmBCvl`. Format, lane self-test and
complete all-lane command returned0. The old binding-failure receipt remains
FAIL and is not replaced. New backend code is unchanged since848 focused
PASS/controlled RED. A fresh-context read-only independent review is requested
as `td156_kde_space_fresh_review_20261018`; root retains the remote execution
lease. Private build/native checks remain pending. This source verdict grants
no installation, publication or physical-client acceptance.

### 2026-10-18T18:07:55Z — TD156: review1, repair the IBus observation/mutation gap

Fresh-context independent read-only review scored8/10 and found one P2:
the KDE engine completion reads IBus (bounded80ms), then may launch its engine
mutation without rechecking the existing request generation/context. A newer
direct request can finish during that read; final rejection cannot undo the
old request's subsequent overwrite. This is a not-yet-started mutation, not
an already-sent command. No other actionable finding was reported.

Repair round1 keeps the same helper and owner: pass the existing is_live
predicate into the engine completion and check it after the read immediately
before conditional mutation. GNOME supplies an unconditional predicate to
preserve its current sequence. KDE production and test dispatch must share
the actual engine-completion body, including its target readback. The new
controlled-order regression completes a newer direct generation during the
old engine read, in both directions, and requires the newer engine to remain
selected with no old write. No route, queue, worker, timer, lock, retry or
rollback is added. Repeat focused/full source gates, prove the new regression
detects a controlled omission, then request the second/final review pass.
The3259 PASS remains historical for pre-repair bytes; build/native are pending.

### 2026-10-18T18:14:31Z — TD156: review repair focused PASS and controlled RED

The repaired IME passed849/849 selected tests in139.5s, no failures:
`development/run-b474gske/RESULT.json`, archive SHA256
`be5b94b32c577948646816d877103911115b42599a67a1fec9f112264128cf5b`.
The new ordered generation regression detects omission of only the post-read
authority check in a separate frozen-source copy: exactly one selected test,
expected exit101/one failure,27.35s. Evidence:
`space-ibus-controlled-red/RESULT.json`; test.log SHA256
`e8043018efb570e061dedbd7361ade67f3dd90a07e18cc9b94199f9c64e31d3f`.
The original controlled GNOME-only backend violation remains independently
recorded. No old checkout was rebuilt as an equivalent counterfactual.

Manifest refresh adds exactly this one correctness identity (3285 to3286
total registered), removes/changes none. Only known-failure manifest binding
changes to `711bb3089df9fce73e7da75ab1e3479989f79fc4ebfa8cfe50c48eb12f90d235`;
failures remain empty/count0, observation path/hash unchanged. Details:
`space-ibus-controlled-red/MANIFEST-COMPARISON.json` and `MANIFEST-BINDING.json`.
Graph refresh passed at `development/tab-graph-du_ly9tj/RESULT.json`.
Remote format itself succeeded in `development/tab-fmt-zo67_vwi/RESULT.json`;
its first collector raised NameError from a misnamed local variable. Existing
formatted output was then fetched with source-stability verification, without
repeating formatting; fetch.json retains that distinction. Full source gate
and second/final code review are still required; no new native result.

### 2026-10-18T18:23:34Z — TD156: repair round1 complete affected source PASS

The full correctness/package gate passed3260 selected identities in515.6s:
`development/run-6bpsnmpr/RESULT.json`, SHA256
`9437f355d48a478740b35c3d3c6bff3fd4d5de83790c0b2276b69183cecebf68`. Its source archive SHA256 is
`397148efa78cab9cdda9a174c99e123b32814b0727e957bd5b7236b0b424a822`. Format, lane self-test, complete all-lane command
and the unchanged zero-failure binding returned0. Runtime bytes have not
changed since849 focused PASS and the post-read-check controlled RED.
Second/final independent read-only review is requested from
`td156_kde_space_fresh_review_20261018`. Native/build remain pending; no
installation, publication, delivery-route or lexical/model changes.

### 2026-10-18T18:24:33Z — TD156: final source review9/10, private build authorized by scope

Second/final independent read-only review scored9/10. Its P2 is closed; no
remaining source-scope blockers were found. It confirms the post-read check,
newer-completed-direct-generation regression in both directions, shared
production/test engine completion, unchanged GNOME behavior, full3260 PASS,
focused849 PASS and exact controlled RED. Review records are
`space-ibus-controlled-red/REVIEW-PASS1.json` and `REVIEW-PASS2.json`.
No reviewer executed tests, remote operations or native input. This ends the
two-pass source review/repair cycle, not physical acceptance.

Build one frozen private candidate C under the existing remote guard, bind it
to the3260 receipt, then run the four supplemental Space/next-letter controls.
Only if they all pass, run the original432 KDE identities and separate original
216+216 GNOME identities. Both complete matrices use identical three binaries,
with all initial data/config pins and finite closure evidence retained. The
KDE run uses qualified short private IBus paths in all six apps; this environment
change must remain visible when comparing candidate effects. No installation
or publication is authorized by this source verdict.

### 2026-10-18T18:31:06Z — private C build and supplemental protocol correction

- Built the frozen source archive `7daed3ca739db9656b54fc2d59b125e2f7a93ea49d706d09ff61e249547cf6ff` after full 3260 PASS and final independent source review 9/10. Receipt: `/home/ubu/.cache/lay/development/browser-final-fixes-20261018/build-kde-space-c/BUILD.json`. Canon/strict lints/security audit passed; zero known vulnerabilities in the pinned local database. Build 177.4 s. Private IME SHA-256 `0cd6027b7773042615a5b30725c7d089c7cdd023d993b1df4f6c3a43d4820bb0`; lay and daemon bytes unchanged from B. No installation/publication.
- Preparation failures retained: `aborted-kde-space-next-letter-j-preparation/PREPARATION-FAILURE.json` (Qt job stem) and `kde-space-next-letter-j/PREINPUT-FAILURE.json` (supplemental selector versus copied original432 completeness expectation), under the same development directory. Both occurred before native session/input. Only supplemental scope corrected to exactly2 selected cells; the original432 controls/targets/deadlines remain unchanged.
- First actual supplemental GTK attempt: `kde-space-next-letter-k/gtk/evidence/native-windows-a86nox49/RESULT.json`, SHA-256 `c59c4ffe8ecf570a5e04610d470a896fffe6aa4027140e113620752bca2e3cff`. Both original committed Space targets reached, including KDE mode, in 20.67 / 27.36 ms. Both next-letter assertions FAIL: committed prefix/caret plus actual Gtk.Entry Pango display and preedit show correct `z` / `я`, but the reused target helper accepts only committed/AT-SPI text, which excludes the active preedit. This is a measured observer mismatch, not evidence that the letter failed to render. Receipt remains FAIL; Qt2 NOT_TESTED. Session independently CLOSED, no held keys, cgroup empty, pinned processes gone, temporary policy absent, VT1 restored.
- Explicit supplemental v2 protocol before further input: retain the original committed Space target; immediately read the actual stock IBus engine and require target engine; type physical Z, record the uncommitted widget state; type physical Space; require the exact committed prefix+letter+Space and target KDE mode; require actual target IBus engine again. This tests completion without requiring ordinary preedit to be prematurely committed. Compound timing includes the second Space and must not be presented as isolated next-letter latency. Fresh roots `ksvg`/`ksvq`, report `kde-space-next-letter-l`. Four controls are outside both complete-matrix denominators and must all PASS before complete J starts. No retries of the old native protocol or runtime changes were made.
- Latest complete native KDE remains 149 PASS /259 FAIL /24 BLOCKED, NOT_TESTED0. Four-control failures do not replace that result. GNOME J and full KDE J remain NOT_TESTED at this entry.

### 2026-10-18T18:34:44Z — four native prerequisites PASS, complete KDE J in progress

- Supplemental v2 report: `/home/ubu/.cache/lay/development/browser-final-fixes-20261018/kde-space-next-letter-m/RUN.json`: 4 PASS /0 FAIL /0 BLOCKED /0 NOT_TESTED. GTK Entry and Qt RichTextEdit, both directions. Actual stock IBus target engine required immediately after Space and after physical Z+Space commit; exact committed text, caret/no selection, KWin mode and existing activity/foreground checks all passed. Original Space observations 22.33–29.02 ms; compound next-letter+commit 73.91–91.71 ms (includes second physical Space; not isolated letter latency). Both sessions CLOSED with no held keys, empty cgroup, pinned processes gone, no temporary policy, VT1.
- Primary GTK receipt `kde-space-next-letter-m/gtk/evidence/native-windows-_113_9dl/RESULT.json`, SHA-256 `be065762dc7756a702d298c549492b43f58817f6024bd2383cc7f44b10362083`. Qt `kde-space-next-letter-m/qt/evidence/native-windows-sv85624u/RESULT.json`, SHA-256 `985bfd41038997885ed979e6821086da7d165649e33a711499496787403a5c57`. Paths relative to the same development directory.
- The intervening `kde-space-next-letter-l` attempt aborted before authorization/actor/input: copied seat label collided with the prior exclusively created seat receipt. Independent cleanup stopped the newly created finite seat and restored VT1; actor/cgroup fields correctly remain null. Preserved original ABORTED receipt. M uses fresh short roots, unique labels/timers/policies, validated before input. No runtime changes or repeated successful native attempt were made.
- Complete original432 J started 18:33:02Z with the same C bytes, unchanged case identities/targets/deadlines. `kde-space-matrix-j/STARTUP-VALIDATION.json` binds prerequisite hashes and actual unique phase labels/policies. A local additional label assertion had assumed browser labels `tdkde-space<app>`; actual frozen labels derive from the short roots `tdkde-ksf`/`ksc`. The assertion failed before runner launch; subsequent AST inspection corrected this assumption without modifying any uploaded/running harness. Required independent runner pin/exclusive-root checks remain in force.
- First complete phase GTK: 24 PASS /6 FAIL /2 BLOCKED, compared to 20/10/2 in E. This is partial evidence, not a new complete432 total. Qt in progress; complete KDE and GNOME J NOT_TESTED until their own receipts exist. No installation/publication; text delivery routes unchanged.

### 2026-10-18T18:48:38Z — complete private KDE C matrix: 175 PASS, 233 FAIL, 24 BLOCKED

- Complete original432 identities, zero NOT_TESTED: `kde-space-matrix-j/RESULT.json`, SHA-256 `3ab5526d6ae397e2f59b85ca71c591b349f7b0b7e2d85665eb402c44d09835bc`, finished2026-10-18T18:46:22Z. All paths in this entry are under `/home/ubu/.cache/lay/development/browser-final-fixes-20261018`. Apps PASS/FAIL/BLOCKED: GTK24/6/2, Qt36/9/3, KWrite11/3/2, Konsole3/10/3, Firefox80/73/7, Chrome21/132/7. All six finite sessions independently CLOSED: cgroup empty, pinned processes gone, temporary policy absent, VT1 restored. The same private C binaries/configuration/model/initial-data pins were used in all phases.
- Exact E-to-J comparison: `kde-space-matrix-j/BASELINE-COMPARISON.json`, SHA-256 `10d32ae2606da703f61977f3f0ce9ddd620464ce1603d133d3322e429805d029`. Same432 identities and initial data; 27 FAIL-to-PASS and one PASS-to-FAIL, all other verdicts unchanged. Retains148/149 previous PASS. Recoveries: GTK4, Qt6, KWrite8, Firefox9; Chrome and Konsole have no cell changes. Net improvement26, 259 to233 FAIL. Because four app environments now also use qualified short private IBus addresses/background exclusions, this is candidate-plus-environment evidence, not isolated attribution of every recovery to the backend patch.
- Retained regression: Firefox URL field3/moved_boundary now commits `должн ыбыть ` instead of `должны быть `; caret12, focused, no selection. Target KDE group, stock IBus engine and Lay input mode are all ru. This does not establish a group-switch failure. Case trace `124-ime-trace.jsonl` retains43 bytes (a prior-record tail and reset), SHA-256 `3cd93a765979863c5621b1c561a6e1b6dc50b7f3f381bb71d30f363182f92788`; daemon interval871 bytes, SHA-256 `2972c6724fdc8d6624b64ab1e5c155cc344301f30629fd1986b5ef33616107a8`. Full archived IME trace511992 bytes/2171 valid records does not supply a qualified admission/candidate window for this failure. Asynchronous trace completeness is unknown. The post-case KWin caption says Not Responding; its observation is not atomic with the failed transition and is not a causal diagnosis. No unchanged retry erases this FAIL; root cause remains UNKNOWN.
- Measured latency, with missing opportunities retained: Double Shift240/540 observations, median141.37ms, p95172.94ms, max209.66ms; Space73/135, median28.98ms, p9532.26ms, max33.43ms. Selection/focus-return/Tab each0/27 timed observations; backspace has no timed operation. Measurements include successful intermediate operations in later-failed cells. Coverage differs from E, so these are not speed-improvement claims. Evdev write-return to observed exact target under software VKMS is not kernel-release/hardware latency.
- Four supplemental next-letter prerequisites remain separate4/4 PASS and do not change either full denominator. Full source3260 PASS/final independent review9/10 remain source proof only. New-byte GNOME216+216 has started in fresh finite sessions and is NOT_YET_COMPLETE at this entry. KDE no-regression gate is not passed because one prior PASS is lost. TD156–159 remain IN PROGRESS; no local/user installation, public publication, new text-delivery route or lexical authority change. No further runtime edit is part of this frozen native stage.

### 2026-10-18T19:14:31Z — frozen Space-backend stage complete; KDE acceptance remains open

Current evidence is private C, source archive `7daed3ca739db9656b54fc2d59b125e2f7a93ea49d706d09ff61e249547cf6ff`, IME SHA-256 `0cd6027b7773042615a5b30725c7d089c7cdd023d993b1df4f6c3a43d4820bb0`. Source3260/3260 PASS, focused849 PASS, controlled omission RED and final fresh-context independent code review9/10 (two passes maximum) are complete. Rust/dependency source stayed unchanged during the native stage. Existing IME layout owner now selects its request-local KDE backend; no text delivery route, model, lexical decision authority, installed user runtime or public release changed.

| Complete native grid | PASS | FAIL | BLOCKED | NOT_TESTED | Total |
| --- | ---: | ---: | ---: | ---: | ---: |
| KDE GTK | 24 | 6 | 2 | 0 | 32 |
| KDE Qt | 36 | 9 | 3 | 0 | 48 |
| KDE KWrite | 11 | 3 | 2 | 0 | 16 |
| KDE Konsole | 3 | 10 | 3 | 0 | 16 |
| KDE Firefox | 80 | 73 | 7 | 0 | 160 |
| KDE Chrome | 21 | 132 | 7 | 0 | 160 |
| **KDE all six apps** | **175** | **233** | **24** | **0** | **432** |
| GNOME Firefox | 176 | 21 | 19 | 0 | 216 |
| GNOME Chrome | 178 | 25 | 13 | 0 | 216 |
| **GNOME two browsers** | **354** | **46** | **32** | **0** | **432** |

KDE J retains148/149 E PASS, with27 FAIL-to-PASS and one PASS-to-FAIL: net26 fewer failures (259 to233). Twenty recoveries are Space layout-direction plans, seven Double Shift plans. All other verdicts unchanged. Final remaining FAILs by last actor step: clear90, seed44, target-after-Double-Shift67, target-after-Space32. These are observation-stage families, not runtime root-cause counts. Separate qualified environment changes mean this is not isolated patch attribution. The retained new Firefox URL3/moved_boundary FAIL has correct group/engine/mode but an unqualified candidate/admission trace, so its cause remains UNKNOWN. The previous entry retains its exact slices and limits. No repeated unchanged attempt regrades it.

Same-byte original GNOME216+216 completed2026-10-18T19:12:51Z:
- `gnome-space-regression-j/RESULT.json`, SHA-256 `4b41bcbee8446bf574f8b0a97fbb489322b732f6146b52148ca10cd2d9e6e03f`.
- Exact immediately preceding E comparison: `gnome-space-regression-j/PRECEDING-COMPARISON.json`, SHA-256 `48944ced59172f0e17c3549dc4d6d1d192d242930d09baeb2446cd8299107f3f`. Same432 identities, all18 primary receipts hash-verified, configuration/harness pins unchanged. All353 preceding GNOME PASS retained. Firefox fullscreen/active/field8 changes FAIL-to-PASS; every other Firefox verdict and every Chrome verdict is unchanged. This satisfies this bounded GNOME E-to-C nonregression observation, not zero-failure acceptance or repair of the previously recorded bridge-settlement race. The older prefix FAIL remains FAIL.
- Primary summary SHA-256: Firefox `09e4a53401f4c498163fb3dcf6c48f03d9d974a7859e66a16623d18027c0ea54`; Chrome `3714bfc0669be0fe7985029eda25336b8ad9f3bfad2e6ee53aef21a0585ced62`. Both finite sessions independently CLOSED with empty cgroups, pinned processes gone, no held parent keys, finite seats stopped and VT1. All six KDE phases likewise CLOSED; no owned session/input remains active.

GNOME measured timing (required/measured/missing retained in primary summaries):

| Browser / operation | Measured / required | Median ms | p95 ms | Max ms |
| --- | ---: | ---: | ---: | ---: |
| Firefox standard | 420/420 | 99.13 | 112.05 | 122.01 |
| Chrome standard | 408/420 | 81.77 | 99.48 | 110.89 |
| Firefox panel | 48/48 | 100.76 | 115.31 | 118.30 |
| Chrome panel | 48/48 | 81.90 | 96.88 | 101.35 |
| Firefox functional Double Shift | 147/210 | 101.17 | 126.99 | 153.97 |
| Chrome functional Double Shift | 156/210 | 80.60 | 103.04 | 120.46 |
| Firefox functional Space | 37/50 | 18.98 | 25.27 | 27.82 |
| Chrome functional Space | 39/50 | 19.23 | 24.60 | 26.93 |

Functional selection: Firefox7/10,18.38/21.73/21.73ms median/p95/max; Chrome9/10,19.69/22.31/22.31ms. Tab0/10 in each browser, no successful timing sample. Missing coverage and intermediate successful operations in failed cells remain explicit; these software-VKMS observations do not establish a hardware or speed-improvement claim. KDE timing and the separate4/4 actual-engine/next-letter prerequisite are in the preceding entries and do not share GNOME denominators.

Verdict: **bounded implementation/source/native stage complete; KDE no-regression and full compatibility acceptance NOT PASSED**. TD156–159 remain IN PROGRESS. Native provider lifecycle controls, Chrome/Konsole IME ingress qualification, rich-field raw/document clear qualification, terminal preedit pixels and visible KDE tray/menu are not established by these matrices. Prior runtime refusal/trace uncertainty is preserved. Next work first qualifies the retained regression window and the stock-client ingress/clear families under existing owners; do not reimplement the completed Space backend or weaken admission/verifier. Further runtime repair requires a fresh written bounded plan and its existing review/test gates. No installation, publication or new native input is part of this closure. Final generated-graph receipt pointer: `kde-space-matrix-j/FINAL-GRAPH-POINTER.json` under the same TD156 cache; it records source/artifact validation separately from native acceptance.

### 2026-10-18T19:35:35Z — resumed two-environment residual reduction; diagnostic preparation

The user requests continued work towards at most20 FAIL in each environment.
This is a target, not an achieved result or authority to shrink either fixed
denominator. Complete native baselines remain KDE J175/233/24/0 of432 and
GNOME J354/46/32/0 of432 (PASS/FAIL/BLOCKED/NOT_TESTED). No new native attempt,
runtime edit, private build, installation or publication has occurred in this
stage. Root owns the single remote/native/heavy lease and integration; one
agent owns shared runtime diagnosis and one owns trace-observer diagnosis.

Current GNOME retained families are20 typo-undo Space prerequisites,16
post-refocus Double Shift context refusals,4 post-refocus return-letter cases,
5 Chrome Backspace cases and one Firefox prefix inverse. The previously
qualified20 Space prerequisites are SuggestOnly/lexical-authority limitations
(see5580ff), not an excuse to weaken admission or train literal fixture text.
The16 focus cases have live bridge tokens but no whole-word focused-field
proof after a correct UnknownStart transition. Existing exact snapshots prove
top-level windows or already-admitted tails; they do not identify an arbitrary
DOM field. A fresh-field proof needs an explicit existing-owner design before
implementation. Backspace traces for the two standard Chrome failures were
fetched read-only and hash-verified under
`gnome-bridge-ready-next/retained-backspace/`: SHA-256
`38fdad6f29a4f4a8b6b310944ed7c9ebee1a009a588261bb9b1bbe0c23f0aa1b`
and `21db00d5008edc22a5b2542be9db3c05bda7222d2eb3da1c45f6626c479da936`.
Their observed raw text retains the probe letter while caret has moved back;
the first failed runtime transition is still under investigation.

KDE actor-step counts are not causal counts: selection/focus_return omit a
post-seed stage mark, so38 last-labelled seed failures already reached Double
Shift. Do not diagnose all44 as missing seed ingress or merge unknown-context
with the separate admitted-tail rejection. The diagnostic agent is preparing
immutable bounded snapshots/prefix controls for same-inode log compaction;
the retained URL3/moved_boundary cause remains UNKNOWN.

For the90 rich clear prerequisites, root prepared a separate six-field
reversible native presentation diagnostic, not a rerun of original cells.
Its written alternatives, exact seven physical steps, negative real-newline
control, invariants and rollback are in cache
`kde-rich-presentation-next/PREFLIGHT.md`; selected identities and pinned scripts
are in its `PLAN.json` and per-browser `SCRIPTS.json`. The pure diagnostic
candidate collector distinguishes caret0 placeholder LF from a real newline
at caret1 and preserves whole raw text. Original acceptance predicates are
unchanged. Remote dedicated-20cpu source controls pass7/7 per browser (four
actual cleanup controls and three presentation negative controls), logs
`kde-rich-presentation-next/{firefox,chrome}/SOURCE-CONTROLS.log`.
Fresh independent review and actual native diagnostic remain pending.
No diagnostic result may regrade the retained432 or supply runtime authority.

### 2026-10-18T19:41:16Z — bounded trace-observer control and retained audit

Fresh review9/10 (no material P1/P2) preceded one remote dedicated-20cpu
guarded execution of `kde-causal-window-next/remote_entry.py`. Result COMPLETE:
`kde-causal-window-next/remote-results-a/RESULT.json`, SHA-256
`2195580bf2cbcf2eae6c58b7a7c66b6f52467a83f44ada725a35039d25f6589f`.
Finite private-filesystem controls3/3 PASS: append gives APPEND_PREFIX_MATCH,
complete-line same-inode nonshrinking compaction gives PREFIX_CHANGED, and
replacement gives INODE_CHANGED. The retained actor's legacy offset misses
new control serials999/1000 after compaction. Control receipt SHA-256
`e8d723bd5d2f6301042c91b83eeeaeba55ad8dc4a95e546204ad2dd5ca2e1be3`.
This qualifies the bounded endpoint predicate, not Rust producer completeness
or interval continuity; both remain UNKNOWN. Temporary control files removed.

Hash-verified retained audit is CAUSAL_WINDOW_UNQUALIFIED, SHA-256
`87ad4fc0ebaaf74b4cf453f2bb01002332128f67178d1e7d4f62513515ee6ff9`.
Firefox URL3/moved_boundary row124 remains FAIL: missing boundary prefix bytes,
unbound neighbouring serials and unknown async loss/flush prevent a causal
claim. Corrected observable-family counts total233, without regrading cells.
No native input, client launch, runtime edit or authority change occurred.
KDE J175/233/24/0 and GNOME J354/46/32/0 of432 each remain current complete
native baselines. The separate six-field rich presentation diagnostic is
pending its second review after enforcing exact frozen C/config/data pins.

### 2026-10-18T19:44:30Z — rich presentation diagnostic blocked by NBSP collector

The diagnostic's second independent review9/10 closed the C/config/data-pin
gap. One finite Firefox seat executed all three supplemental fields7/8/10:
0 PASS/3 FAIL/0 BLOCKED, each at one_line; no original cell was rerun. Initial
Ctrl+A/Backspace yielded raw empty/caret0. Native `hello ` then yielded
`hello\u00a0`/caret6/no selection/focused, while the diagnostic collector
incorrectly required ASCII Space. It never reached Enter/multiline/return
qualification. No claim about the original90 LF clear failures follows.
Primary `kde-rich-presentation-next/firefox/evidence/native-windows-ptnetory/`
`RESULT.json`; evidence archive SHA-256
`c317de30e087a35be7d4cf0d6cbb9535e10b32d760d554afc81c1b2dc883bac2`.
Closure CLOSED, empty cgroup, pinned processes gone, no held keys, policy
absent and VT1. Chrome's identical invalid collector was not executed.

The changed diagnostic under `kde-rich-presentation-space-next/PREFLIGHT.md`
reuses the existing positional typed-Space rich observer, preserves the exact
newline/caret negatives and all original acceptance/runtime predicates.
Its source controls/review/native attempt are pending. No production edit,
new private build, installation, learning or authority change. Complete KDE
J175/233/24 and GNOME J354/46/32 of432 each remain unchanged.

### 2026-10-18T19:54:36Z — bounded rich presentation control remains unqualified

The changed diagnostic has controlled RED1EXPECTED and19 source controls PASS
per browser, fresh static review9/10. Its first Firefox attempt stopped at the
unchanged letters-only seat-name guard before a seat, cgroup or GUI was created:
`tdkde-kr2f` was invalid. The zero-input abort is retained under
`kde-rich-presentation-space-next/firefox/RUN.json`; independent closure is
CLOSED, cgroup null and VT1. No functional conclusion follows from it.

The corrected `krbf`/`krbc` orchestration labels passed the same guard and a
focused final review9/10. Finite Firefox and Chrome controls each completed
with0 PASS/3 FAIL/0 BLOCKED, diagnostic only. Field8 in both browsers reached
all seven physical steps: initial empty is raw `""`/caret0; after multiline
Ctrl+A/Backspace it is raw `"\n"`/caret0; a real Enter produces `"\n\n"`/caret1;
Backspace returns raw `"\n"`/caret0. The diagnostic failed its strict initial
versus cleared raw-equality assertion. This establishes changing accessibility
representation; it does not alone prove document emptiness or authorize
stripping newline in an acceptance predicate.

Fields7/10 stop after the first Enter: Firefox reports two U+FFFC objects,
caret1; Chrome reports `hello\u00a0` plus U+FFFC, caret6. Parent Text is not a
complete multiline document view. Blind FFFC flattening and generic LF removal
are rejected. Next observation must retain the bounded child/Text/Hypertext
relationship, real-newline negative and raw snapshots; a DOM mirror may only
be supplemental evidence, never runtime edit authority or silently native PASS.

Primary receipts under `kde-rich-presentation-space-seat-next/`:
Firefox `firefox/evidence/native-windows-aglx627c/RESULT.json`, SHA-256
`e2c6f5a227d78285fa798dca206ba1e81e7178cc1b31d2208c04c1e58a0a8f8b`;
Chrome `chrome/evidence/native-windows-08b_ijp3/RESULT.json`, SHA-256
`52bf97b99072168c5b39245a72b863a8ff7a12f3fb5fc49d126a5a530357fb6c`.
Evidence archives respectively
`2bc9944349a0deb24d23b8b49330dca99ca9120413a2aad51ccbfb3af183dcc3`
and `f5f51cac93323822b3d39753b03d30f15aaf50952de119c91bb0269b39225129`.
Both independent closures verify CLOSED, empty cgroups, pinned processes gone,
no held keys, temporary authorization absent and VT1. Step-observation delays
are retained in the primary receipts; they are not Double Shift latency samples.
Original complete KDE175/233/24 and GNOME354/46/32 of432 are unchanged.
Production source, runtime authority, models, installation and publication:
unchanged by this experiment.


### 2026-10-18T20:19:30Z — stock native-Wayland prerequisite, zero-input setup abort

The private KDE IBus1.5.34-rc2 previously disabled Wayland/UI. Pinned upstream
Chrome155.0.8059.39 defaults text-input-v3 to enabled; its GTK Wayland factory
returns null. KWin Plasma5.24 launches its selected input method with a private
WAYLAND_SOCKET; the same pinned IBus GTK3 panel supports --enable-wayland-im.
This is a missing-platform-prerequisite hypothesis, not a proven Lay defect.
Primary downloads/hashes: cache chrome155-ime-source/SOURCE-PINS.json. Options,
invariants, source/archive pins,4-cell/native432 gate and rollback are in
kde-native-ibus-integration-next/PREFLIGHT.md. No browser flag or delivery
fallback is permitted to obtain a PASS.

Remote dedicated-20cpu build PASS: knib-build/workflow-vapi-continuation/BUILD.json;
local copy kde-native-ibus-integration-next/BUILD.json. Prefix remains private.
IBus archive SHA101167b87e281915594a1cc277382e9e707dabf3182a50ec88c56fd3a8e7f0ff;
panel SHA f53615d853847470abf639257756ec7009a077e03240b57759660f16adebf03c.
ldd-r is clean. Preserve both preceding failed attempts: GitHub tarball lacks
referenced generated dictionary headers; host Vala lacks wayland-client.vapi.
The continuation uses pinned same-version generated headers with dictionaries
still disabled, then the exact upstream workflow copy of wayland-client.v0.56.vapi
(SHA b1f8f6baac5f712b880677d61be7ef870680cb508e890bfff4d9977342d2dc0a).
Final make2.332s/install1.903s/relocations.013s are continuation timings, not total
build duration. Only missing authorized remote development packages were added;
apt plan recorded0 upgrades/0 removals. No user IBus or Lay installation changed.

Fresh diagnostic review9/10 and remote11/11 controls PASS preceded knic.
Original Chrome normal fields1/7 roundtrip and space_burst5 bodies are unchanged.
KWin launched the expected panel; setup then aborted before actor input. Native
plasma.log identifies GLib-GIO-ERROR: host1.5.26 schema lacks panel/use-custom-theme.
Do not infer missing IBus peer API from the secondary timeout. Retain
kde-native-ibus-integration-next/chrome/RUN.json and evidence/{ERROR,CLOSED,
PREPARED,SOURCE-CONTROLS}.json plus plasma.log. Closure verifies empty cgroup,
VT1 and removed temporary authorization. No browser-native verdict or latency
sample was produced. No unchanged retry is justified.

Separate knid continuation compiles only the pinned matching XML into the new
session's private-schemas (XML SHA d9ba6742625cd40dcf97556e4dd86aa41b963eae4c807f097d2cc738b12473c2).
The exact path survives sanitized environment and private activation; system
schema fallback remains. No global schema install, Lay bytes, browser argv,
model, ownership or output route changes. Source13/13 PASS includes actual
compile/lookup and tampered-schema refusal; native awaits scoped review.
See kde-native-ibus-integration-schema-next/CHANGE.md and chrome/SCRIPTS.json.

The separate GNOME four-cell observation-only package has fresh review9/10,
source36/36 PASS (31 pure plus5 real private GIO) and controlled RED2 baseline
PASS ->2 mutant FAIL without modifying working code. Receipt remains remote
TD156-focused-field-capability-20261018/SCOPE.json. Native is NOT_TESTED;
post-verdict observations cannot establish historical field binding or atomicity.
Full original grids stay KDE175PASS/233FAIL/24BLOCKED and GNOME354/46/32 of432,
NOT_TESTED0. Runtime authority, user installation and publication unchanged.

### 2026-10-18T20:41:33Z — localize setup and observer failures before native replay

The knid zero-input continuation passed private-schema setup, then refused a
GetConnectionUnixProcessID call. The pinned IBus dispatcher does not implement
that method, despite declaring it in its interface XML. No keys were entered;
retain kde-native-ibus-integration-schema-next/chrome/RUN.json. Closure at
20:21:44.645098Z verifies empty owned cgroup, VT1 and absent temporary policy.

The separate knie continuation checks SO_PEERCRED on its actual GIO transport
and supported GetNameOwner/NameHasOwner calls, without claiming panel-PID
binding or registration continuity. Source20/20 PASS, fresh review9/10.
Stock KWin-selected panel and private IBus registration then succeeded, but
SetGlobalEngine failed with "Operation was cancelled" before any keyboard
input. Retain kde-native-ibus-integration-bus-next/chrome/{RUN,SCRIPTS}.json
and remote knie/launch.log (23:36:51.446), NATIVE-INPUT-METHOD.json. Script
manifest SHA f315e6c998adb353e35f388a4167e03a11a1e15dd1810e013c1f26c61071b59f.
Closure at20:36:54.521882Z confirms empty scope, VT1 and absent policy. A race
between the stock panel's default xkb selection and setup's Lay selection is
the next hypothesis; neither native Chrome ingress nor that cause is proved.

Raw-capture source controls first exposed Python-version-dependent AST dumps;
immutable source-two uses exact-source pinning instead. Source76/76 PASS,
38/browser. Firefox then captured3 rich fields x7 stages, all PARTIAL and
UNQUALIFIED because clear_cache_single is unsupported on the actual accessible.
Text, children and Hypertext reads remain available: fields7/10 expose U+FFFC
anchors after Enter, while field8 returns text. Clearing leaves LF/caret0;
a real Enter changes topology/caret. This does not authorize stripping LF or
flattening U+FFFC. Retain kde-rich-raw-capture-next/firefox/RUN.json and evidence;
primary RESULT SHA248cfbce44d6ec8d7bec802b832c83ac68c56797ca11edb42d902ce7d0926b62,
evidence archive SHA3c01c6cf8d240e739ce0fdb50399a0d39feea2cf6364b5809ca1d6ae9242f369.
Closure at20:36:01.918737Z confirms no held keys, empty scope, VT1 and no policy.
Chrome raw native remains NOT_TESTED; do not run the known unsupported API.

GNOME's observation-only continuation is partial: Search Double Shift returned
PASS105.365ms, one input/refocus case failed, then the composite mirror
prerequisite timed out. Retain gnome-bridge-ready-next/native-control/partial-firefox/
and FOCUS-CONTINUITY-RICH-RETURN-FINDINGS.md. It proves distinct DOM fields reused
InputContext_3, so that key cannot restore prior field ownership. Separately,
all4 original rich-return failures had caret0 before append and before Double
Shift. Neither finding grants text-only mutation authority or regrades the grid.

The browser protocol now specifies zero-key real-API/setup preflight, grouping
by the first common failed transition, one targeted repro before a full grid,
and bounded rejected-payload evidence from existing reads. No new runtime
owner, typing-path RPC, model/training, delivery route, user installation or
publication. Latest complete matrices remain KDE175/233/24 and GNOME354/46/32
of432 each, NOT_TESTED0; all diagnostics above remain separately scoped.

### 2026-10-18T20:54:02Z — complete rich observation and stop a GI failure before input

The separate immutable rich API continuation uses the actual supported
Accessible.clear_cache. Its zero-key installed-GI method preflight and all76
source controls passed (38/browser). Firefox krsf and Chrome krsc each captured
3 fields x7 physical stages:42/42 complete bounded trees, stable root readback,
no tree issues or read errors. These are raw CAPTURED/UNQUALIFIED observations,
not42 behavioral PASS and not Double Shift latency samples. Field7/10 expose
U+FFFC paragraph anchors after Enter; field8 exposes text. Exact empty-after-clear
representations differ: Firefox LF/caret0/no children; Chrome LF/caret0/one
static LF child. Real Enter changes caret/topology; arbitrary LF stripping or
U+FFFC flattening remains prohibited. A clear-only predicate is being prepared
separately with real-Enter and nonempty-child negative controls.

Receipts under kde-rich-raw-capture-api-next: Firefox
firefox/evidence/native-windows-m41mioff/RESULT.json SHA
9564c726bb6de1d68e5e9810e6cbc1872e7e79a40cec16c1c67f8f3f3215f795,
archive b63570fbf998e05b48d28a857386618b00eac273069bcd43ea26ce04a974f27a;
Chrome chrome/evidence/native-windows-qgljln4i/RESULT.json SHA
3d082ae51a0711caed54fbb5ac507ed40d23cb34a4a7c4982f97a6a9d5835734,
archive5f0859cf154a968a9c7951bdcc29a39be072e333707a6237daebfc655c1dd282.
Both independent closure checks are CLOSED, empty cgroups, no held keys,
VT1 and absent temporary policy, at20:48:09.884547Z and20:51:22.139841Z.

The separate knif setup candidate keeps the stock panel as sole initial engine
selector, with private RU-first preload/order and observer-only Properties.Get
readback. Fresh root source review9/10; source-only, no installation. Controlled
RED observed the old empty preload (1 expected failure,0 errors). Normal remote
controls passed26 tests, then the synthetic GI serialization/decoding path
aborted in test_delayed_ru_is_observed_without_any_selector_rpc with GLib
reference/type assertions (exit134). The37-test suite did not complete; no
SOURCE-CONTROLS PASS, native session or keys followed. Retain
kde-native-ibus-integration-engine-next/chrome/{SOURCE-RED,SOURCE-ABORT}.json
and SCRIPTS SHA0441a62c8ebfa0f2c1819da1bc4b0165797333b00797ab4803fa89131a10677a.
Floating serialized-fixture ownership and temporary decode lifetimes remain
distinct hypotheses; a finite zero-key API probe must distinguish them before
another full suite or native attempt. This is a harness failure, not a new Lay
client failure or a proved KDE ingress repair.

Runtime binaries, source authority, delivery routes and the complete original
matrices are unchanged: GNOME354/46/32, KDE175/233/24 of432, NOT_TESTED0.

### 2026-10-18T20:58:52Z — isolate GI fixture ownership without starting a session

The finite zero-key ownership probe ran against both actually loaded providers:
host IBus1.5.26 and private IBus1.5.34. Both inspect controls passed and confirmed
that EngineDesc.serialize_object returns a floating full-transfer Variant.
Both sink-pair controls decoded lay-ime-ru and released nested containers,
then failed during final owner collection with g_atomic_ref_count_dec under
G_DEBUG=fatal-warnings (exit -5). ref_sink returned a second Python wrapper
for the same reference; this candidate does not balance their ownership and
must not be used as a fix. Keep this separate from a malformed wire payload or
an actual live Properties.Get failure: neither was demonstrated by the probe.

Retain kde-native-ibus-integration-ownership-probe-next/RESULT.json and
{host,private}-{inspect,sink-pair}.json. The probe SHA is
6603e639f256b343fa861e2f013501df8ecbb2c36cea9ecc054b460bc568a253.
These are2 PASS inspect controls and2 FAIL ownership controls, no bus, session,
keys or latency samples. The next step is a differently owned, finite API
control that must exit normally, including garbage collection, before the
37-test source suite or any native input. Do not repeat sink-pair unchanged.
No runtime authority, routes, installation, graph/source snapshot or original
matrix verdict changed.

### 2026-10-18T21:02:54Z — balanced synthetic GI fixture passes the finite API control

The different parse-owned control passed on both actual providers (host
IBus1.5.26 and private1.5.34):2 processes,4 object cases, empty stderr, normal
exit0 including final garbage collection. Stock GLib print/parse produced an
equal independent nonfloating value; the original floating serialization was
released before embedding. EngineDesc decoded as the expected RU engine and
IBus.Text remained the wrong type. No manual ref/ref_sink/unref, custom decoder,
bus, session or keys were used. Per-process wall times were0.04885/0.04915s;
these are API-control times, not IME latency.

Receipts: kde-native-ibus-integration-ownership-parse-next/{RESULT,
host-parse-owned,private-parse-owned}.json. Probe SHA
c550497053863002bef10622ef3275323348539dfe0a386b9a24648236084b4d;
driver SHA90cdfd392710046c705ad2d6e17ffbcb128ce7004c3da18d3b5a7a2eadf33491.
Only the synthetic source-test fixture may now adopt this ownership fix. The
real observer decoder and production code are unchanged; the full37 source
controls must still pass before the separately scoped4-case native attempt.
Earlier source abort and failed sink-pair controls remain failed. Complete
GNOME354/46/32 and KDE175/233/24 matrices of432 remain unchanged.

### 2026-10-18T21:09:09Z — both narrow source gates pass before native admission

The knig synthetic-fixture change passed all37 controls with real host GI,
G_DEBUG=fatal-warnings, normal exit,0 failures and0 errors in0.258s. All22
script pins were checked. Receipt: kde-native-ibus-integration-engine-fixture-next/
chrome/SOURCE-CONTROLS.json. The four original Chrome identities remain1/roundtrip,
1/space_burst5,7/roundtrip and7/space_burst5. The finite knig native attempt is
separate and pending; source PASS does not certify compositor integration.

Separately, the exact rich-clear candidate passed12/12 pure source controls,
0 failures/errors/skips. Receipt: kde-rich-clear-qualified-next/
REMOTE-SOURCE-ONE.json; actual remote source attempt is
/home/e/projects/lay-development-runner/rich-clear-qualified-source-20261019/
SOURCE-ATTEMPTS/source-one/RESULT.json. Its artifact manifest SHA is
9a8e65044fd39c7a5294f7c974a56f4b29f10a229b3a61a01a274f9788c8d72f.
The24 fixture excerpts were checked against the pinned raw native trees,
including12 physical-clear positives and12 real-Enter/nonempty/outside-clear
negatives. A fresh native clear-only bundle is still required; these controls
do not regrade the90 original rich-clear failures. Runtime authority, delivery
routes, installed binaries, original matrices and measured-path timing are
unchanged. These two source gates are distinct from IME latency measurements.

### 2026-10-18T21:10:23Z — knig reaches the four native identities; upgrade is not a remedy

The finite knig run completed in21.118s with all four original identities,
normal worker exits and no held keys. Matching private IBus1.5.34, its schema,
stock panel registration and panel-owned initial RU selection passed setup.
No selector RPC from the setup observer was needed. Thus the earlier setup
failure is removed from this diagnostic, but the hypothesis that this setup
change alone remedies the observed Double Shift failures is rejected.

Observed results are4 FAIL, separately scoped diagnostic only: Chrome7/roundtrip
and1/roundtrip retain привет after the first Double Shift;1/space_burst5 retains
привет with Space after the first gesture. Chrome7/space_burst5 stops at clear:
actual text LF, caret0; it does not test the gesture. The daemon records all
three reached gestures: both roundtrips are blocked by the focused IME owner,
while the Space case rejects exact committed-tail capture before replay.
These are the next trace-analysis branches, not evidence for relaxing either
owner authority or replay validation. No repeated native run is needed merely
to reconfirm the same first failed transitions.

Receipt: kde-native-ibus-integration-engine-fixture-next/chrome/RUN.json
SHA379fc777ae921fa70bb69be241369c1b03dddfaf1436990ebbab0f1ba949c933.
Actual native RESULT SHA
f23280dbd9e667ebf616558cb21ec8d2944b2d4e72c2a42a8442ae91d6f03bf2.
All92 exported artifacts were verified against EVIDENCE.json; archive SHA
fd39dccae919f294b9e235ee30ca9298dd96b0a0e92897b048374d6bbb833648.
CLOSED at21:09:23Z and independent closure at21:10:23Z prove empty cgroup,
original pinned processes gone, no held keys, VT1, stopped owned seat and
temporary policy absent. The run did not produce a successful latency sample;
21.118s is the complete diagnostic time, not IME response latency. Original
complete GNOME354/46/32 and KDE175/233/24 of432 remain unchanged, as do source
authority, delivery routes and installed runtime.

### 2026-10-18T21:19:36Z — finite rich-clear source gate stops before native input

The new twelve-cell package retains the original J setup and physical case
sequences. Its controlled RED reproduces exactly six structural-LF fixture
failures per browser. Actual-host zero-key AT-SPI API preflight passes.
The first Firefox source gate runs22 controls:21 PASS and1 FAIL. The failing
immutable-function comparison hashes version-dependent ast.dump output. Local
Python3.14 omits empty fields and has type_params; the older remote parser uses
different defaults. Reproducing the older format gives the exact remote abort()
hash6864d3a47cafa12b3bd2488ebe4757bc424b20d6158687036eeb4b906a42b913.
The initial session-unit-literal explanation was incorrect: abort() contains no
unit literal. This is a harness contract mismatch, not a native Lay failure.
Chrome normal controls and both native phases are NOT RUN. The failed attempt
is retained at kde-rich-qualified-native-next/source-one/RESULT.json; no keys,
clients, runtime changes or semantic acceptance claims follow from this gate.
The next correction must canonicalize only empty/default AST fields, preserve
every nonempty semantic field and reject unrelated actor changes. Actor code
must remain unchanged. Original complete matrices are unchanged.

### 2026-10-18T21:21:52Z — capture rejection diagnostic, source candidate only

Offline analysis of the retained knig trace proves two roundtrip IME RPC
rejections with context_authority, a live bridge token and manual-toggle
authority false. The accepted settlements retain unknown_start. The Space
case delegates exact committed tail but its capture-time validation reason is
unknown; a later actor VisibleTailV4 read is not the original capture reply.
Findings: kde-native-ibus-integration-engine-fixture-next/
OFFLINE-FINDINGS-native-ie7wmiq5.md. No repeat of the unchanged setup is justified.

Applied a rejection-only diagnostic candidate to the existing daemon capture
validator: five static original-failure codes plus parsed source kind, tri-state
permission and binding-presence flags from the same existing reply. No text,
paths or focus receipt enters the new diagnostic. Existing detail redaction,
shared validators, first-failure order, successful outcomes, routes and owner
authority remain unchanged. There is no additional RPC, wait or success log.
Candidate file SHA
ee3e5560031a5d0084b9556f27b4aaaa8415a0d776bb585a7a7a40bd289ac738;
patch in kde-ime-tail-capture-diagnostic-next. Source tests, new diagnostic
binary and native diagnostic are NOT RUN; this entry does not certify them.
Original complete matrices and installed runtime remain unchanged.

### 2026-10-18T21:27:27Z — focused qualified-clear native evidence

The source-two contract uses portable explicit AST serialization, preserving
nonempty semantic fields and negative mutation checks. Actual-host controls:
22 PASS per browser, zero skips; both zero-key GI API preflights PASS. The
original J actor, setup and physical operations remain pinned. Source-one is
retained; its initial unit-literal explanation is superseded by the measured
Python AST-format cause above.

Native original roundtrip/short cells in fields7/8/10: Firefox5 PASS/1 FAIL;
Chrome0 PASS/6 FAIL. Firefox qualifies5/6 clears; the first field10 clear has
empty native text and caret0 but lacks the required root attributes. It fails
closed before Double Shift. Chrome qualifies6/6 clears and fails the first
Double Shift with unchanged text. This separates setup qualification from
actual behavior; it is not a full-matrix improvement or a predicate promotion.
Receipt roots: kde-rich-qualified-native-source-two/{firefox,chrome}/RUN.json.
Native result hashes: Firefox
e7e56bed081f6539ad4ad79c5d87dc2ff90e6a2ed174e681cb463703d75dba4d;
Chrome ba2545c41ce365a18f3a034fa19b1a08995df260d2902f2d577f8523f9d78395.
All60 exported artifacts/browser verify; archive hashes:
f7e7065ee55904e4a32152dd2d7a14a3930cf359f942e2e184e81deb541072ea;
7e7fac5d6f9808950ea9e27f01803109349882df03f0b43e05dbe79f1679a1b8.
Independent CLOSED at21:24:47Z and21:27:27Z proves empty cgroups, pinned
processes gone, no held keys, VT1, stopped owned seats and absent policies.
Successful Firefox operation samples are subset-only evdev-write-return to
observed exact target; failures have no successful latency. No comparison
with prior kernel-release latency is justified. Full GNOME354/46/32 and
KDE175/233/24 of432 remain unchanged. Runtime authority/routes unchanged.

### 2026-10-18T21:29:31Z — diagnostic source formatting and architecture gate

The capture-rejection candidate is formatted; current file SHA
8eae0978dde2e3a4e1c5d2e4ea6b79812cdd409252549524c21b10ed6b2f75c7.
The immutable pre-format candidate remains retained. Fresh no-context review:
9/10, no blockers, static scope only. Graph update/source binding and C01–C10
architecture checks PASS after formatting. Comparison with frozen C covers751
Rust/Cargo/build files and finds only this diagnostic source changed; no
runtime decision change. The15 focused Rust tests and private diagnostic
daemon build are still NOT RUN. Existing3260-test evidence covers old bytes.

### 2026-10-18T21:37:14Z — capture diagnostic source tests and private build

The immutable private source is frozen C plus the formatted diagnostic and its
bound graph/docs overlay. The first runner selected the delegated-tail module:
all13 actual tests PASS, including the two new diagnostic/privacy tests. Its
expected15 assertion then stopped the runner before build. This was a filter
inventory error, not a Rust test failure; the original failed runner receipt is
retained. A separate continuation ran only the two existing RPC-module tests
that the first filter omitted. Their union matches all15 declared file tests:
15 PASS, zero FAIL/ignored; no repeat of the already passing13.

Private release daemon build PASS, features empty, binary SHA-256
8a020332f85dc063ee5b7ab193ff0ef5b2d2d3f939bb125536cb6dff5a35eae7
(9652224 bytes). Receipt: kde-ime-tail-capture-diagnostic-build-next/
executed-b/B-RESULT.json, SHA-256
809525db09693d00c275789be023c56dab941d06999e0eb8900fe73c4e61bd98;
referenced architecture/test/build log hashes verify. The actual shared Cargo
target is7135436800 bytes, below12884901888. Native diagnostic NOT RUN;
installation/publication NOT RUN. The existing validators, runtime authority,
delivery routes and complete native matrices remain unchanged.

### 2026-10-18T21:47:13Z — four-cell capture diagnostic, explicit rejection

Frozen package: kde-native-ibus-integration-daemon-diagnostic-next/chrome,
ARTIFACT-PINS SHA637a19ed4b1621574631c9f3b194f5bd0bc0f4dcf22feb0a3749453ce99e3243.
Actual-host controlled RED has one expected failure/zero errors; the corrected
package has51 source PASS/zero errors/skips, including the installed zero-key
GI API preflight. No keys were used by these source controls.

Retained knig private IBus/schema/KWin-selected stock panel and four original
Chrome cells; only the separately proved diagnostic daemon and qualified
source-two clear prerequisite changed. Native diagnostic completes22.761s
wall time: fields7/1, roundtrip/space_burst5, all4 FAIL. This is experiment wall
time, not successful Double Shift latency. All4 now reach the gesture. Both
roundtrip cases report context_authority. Both post-Space captures explicitly
report source_unknown, unknown source kind, permission denied, engine present
and field receipt absent. Text remains unchanged.

The last plain-input sequence records owned tail7 after managed Space, then
external SSI6; four Shift callbacks settle accepted with owner24/KnownStart.
Delegated exact-tail disposition follows, but the capture readout refuses the
handoff. This positively identifies the capture-time refusal; a later actor
VisibleTailV4 readout cannot replace the rejected exact snapshot. Existing
authority.rs exact-handoff validation requires an external/inherited snapshot
agreeing with the owned tail and must remain intact. No new SSI7 ingress is
established in the retained trace; producer/interval completeness remains
UNKNOWN, so global non-delivery or loss inside Lay is not proved. No retry,
extra RPC, wait, output route or authority relaxation was added.

Native RESULT: kndg/native-windows-7i3bz9q3/RESULT.json, SHA
8dbbd2bd45f65a79743fdfac170102c504f031464b93e40fdf942b0a272dc977.
Independent CLOSED at21:47:13Z proves empty cgroup, pinned processes gone,
no held keys, VT1, stopped owned seat and absent temporary policy. All94
exported files SHA/size verify in the local package/evidence; archive SHA
36da7b2cf90b6be5c6018e19d493bb96c48a92b8282e9ce15817cd104ab80d7a.
An initial export-only attempt matched a finished JSON marker as a directory;
it stopped after independent closure and before archive creation. The corrected
export selects the exact RESULT directory; no native operation was repeated.

GNOME retained-source review independently separates four rich-return append
premise failures (caret0 before the new key) from five Backspace observations;
it establishes no common product authority defect for those nine cases.
Cache report: gnome-bridge-ready-next/BACKSPACE-RICH-RETURN-NEXT.md.
No full-matrix cells are regraded: GNOME354/46/32 and KDE175/233/24 of432
remain the latest complete native evidence. Installation/publication NOT RUN;
runtime authority and delivery routes unchanged.

### 2026-10-18T22:04:40Z — Chrome wire proves a stale compositor acknowledgement

Frozen kwdh changes only Chrome's child environment to WAYLAND_DEBUG=client,
retaining kndg IME/daemon/models, original fields, keys, cadence and clear.
ARTIFACT-PINS SHA931fabefc26440579f3c7cfbb3e317a683fa299e79224ffa336de0e2f093794d.
Earlier kwdg source-only candidate50 PASS/1 FAIL failed its immutable daemon
phase check, never ran native input, and is retained. Corrected kwdh:51 actual
host source PASS, one expected controlled RED/zero errors, zero-key GI PASS.
Fresh static package review9/10/no blockers is not physical acceptance.

Fields7/1 × roundtrip/space_burst5:4 native FAIL, all reach Double Shift,
22.900s orchestration wall time; no successful gesture-latency samples.
RESULT SHAe426e98f7ab06329767023be67257036a585e16a28a25ccd2f851848c8a18519.
Independent CLOSED22:04:40Z verifies empty cgroup, pinned processes gone,
no held keys, VT1, stopped owned seat and absent policy. All95 export files
verify size/SHA; archive SHA2f98e42b3bd520b81bca92eabf0f52cc6d110c85195149859ca2077e66c4df6a.
Local receipt: /home/ubu/.cache/lay/development/browser-final-fixes-20261018/
kde-chrome-wayland-wire-diagnostic-source-two/chrome/EVIDENCE.json.

Retained Chrome log183536 bytes, SHA
95b554bdeda36e1dc2e72891c76582df6609af586cb73f1017a4a128ea1ea6ff,
starts with creation of text-input-v3 object33:60 commits,51 done events,
last done serial59. The four gestures retain commit/last-done pairs
19/18,28/27,52/51,60/59; last outgoing SSI respectively five characters,
empty, five, six. Final post-Space native field has seven characters. Its SSI6
is at line2481, commit60 at2482, four Shift events2512–2577, without done60.
The stale snapshot is already at Chrome's outgoing boundary; a new full
snapshot lost solely inside Lay cannot explain that case. Renderer-internal
pending SetSurroundingText contents remain NOT PROVED. Reproducible sibling
analyze-wire.py writes WIRE-ANALYSIS-source-one.json, SHA
5efbc35dd4fd3c4ccff36634824d845a330b814f4dba50c5cc26437ace6386b5.

Pinned Chromium155 source gates SendPendingImeData on commit/done equality.
Installed KWin5.24.7 uses KWaylandServer5.24.6, whose client commit handler
does not acknowledge it. Primary KDE commit
a74c4361564217919437986feb91a3f85c5a5c33 (2022-06-11) supplies compatibility
acks with pending/current preedit preservation; absent5.25.0, present5.25.1.
The XML does not mandate ack after every client commit. A bare done clears
active preedit; use the full upstream lifecycle fix. API/source/archive pins
are retained under chrome155-ime-source and kde-upstream-done-backport-source-one.

Next private experiment: exact upstream two-file fix plus original autotest
against5.24.6; verify ABI and isolated loading before the same four cells.
Build/backport native comparison NOT RUN. No Lay route, authority, installed
byte or model changed. Full GNOME354/46/32 and KDE175/233/24 of432 unchanged,
NOT_TESTED0; other KDE failures are not assigned this cause without evidence.

### 2026-10-18T22:24:28Z — upstream KDE acknowledgement backport passes protocol controls

Private KWaylandServer5.24.6 candidate applies upstream KDE commit
a74c4361564217919437986feb91a3f85c5a5c33. Its production implementation
matches the pinned5.25.1 implementation; only that implementation, its private
header and the original text-input-v3 autotest differ from the255-file source
inventory. Source archive SHA
c418ed6e88eb5c5a4b193d78954d39c56ffea28eb250e2fcdee69d136e588ff3
matches Ubuntu's exact installed-version packaging; its Debian archive has no
patches. This is source provenance, not a reproducible installed-binary claim.

Actual host controls: stock dependency reproduces testEvents missing done
(2 PASS/1 FAIL); candidate completes29 PASS/0 FAIL/0 SKIP. Loader diagnostics
verify both actual loaded paths. All2379 exported ABI symbols are retained.
Candidate library SHA
4dada4f53718c8f8dcef7b2490231963d1e83943bc1dacc932a7845d314a30fd;
stock library remains SHA
319b486e7e843e406bc7226c5ea3c1d32009014fb5c57759a07154d028bf5b41.
Receipt: /home/e/projects/lay-development-runner/
kde-upstream-done-backport-source-two/PROTOCOL-RESULT-source-four.json,
SHA b37d44e9c353eb0e51ad834dca475bb1f0f53851c2d297a30d4089020e3c0a2b.
Local exact copy: /home/ubu/.cache/lay/development/browser-final-fixes-20261018/
kde-upstream-done-backport-source-one/REMOTE-PROTOCOL-RESULT-source-four.json.

Earlier attempts are retained: an overlong private Wayland socket prevented
connection before protocol testing; the first corrected test loop did not
reach Display's aboutToBlock flush and timed out. Final test uses short private
socket paths and blocking signal waits, without adding production flushes.
The test adapter initializes/clears pending state, positively checks repeated
active preedit and checks no resurrection after commit. These controls reject
both a missing acknowledgement and a bare done that loses composition.

Private native package source-two has51 actual-host source PASS, zero-key GI
PASS and one expected schema controlled RED. Fresh static review9/10 finds
no P1/P2 blockers. Frozen artifact manifest SHA
85013e811ecf3b4df7093d45b0b587dfb315d812489ecb6219bb592d546ba7e6.
The experiment will bind only the candidate dependency read-only inside the
existing private bwrap namespace. Before input, root verification must bind
exact KWin PID/starttick/exe/cgroup to mapped dependency SHA/dev/inode/size,
and verify the host library remains stock. The stock KWin executable and
capabilities are preserved; no LD override, extra key, wakeup, retry or flush
is added. Its four original Chrome cells remain NOT_TESTED at this milestone.

No installation/publication, model, Lay delivery route or runtime authority
change. Latest complete native matrices remain GNOME354/46/32 and
KDE175/233/24 of432; protocol29 PASS does not regrade browser cells.

### 2026-10-18T22:30:12Z — KDE acknowledgement fix restores all four Chrome witnesses

Frozen kbpa source-two ran the same Chrome fields7/1 × roundtrip/space_burst5
as kwdh, with the upstream dependency as its only functional intervention.
Lay/IME/models, key bodies, cadence, targets and deadlines remain pinned.
All4 prior native FAIL become PASS. Both roundtrips verify text in both
directions and following-letter decoding; both post-Space bursts complete all
five toggles. Actual client AT-SPI text/caret/selection/focus and layout are
retained.14 successful Double Shift samples: p50 172.759ms, p95/max178.456ms.
This is software-rendered private KDE test latency, not a hardware benchmark.

Before input, root verification links exact KWin process identity and all
KWaylandServer mappings to candidate SHA/dev/inode/size through /proc/PID/root.
The host library remains stock before and after the session. Its canonical
path is bound read-only only in the private mount namespace. Stock KWin binary
and capabilities, Lay delivery routes, runtime edit authority and installed
user bytes are unchanged. No extra key, wait, wakeup, flush or retry was added.

Native RESULT: /home/e/projects/lay-development-runner/kbpa/
native-windows-yzsu7wfi/RESULT.json, SHA
ae0d977155b55db1e92012a53c074a45f9f73c5cf18ed825d6e197dc51bac08a.
Independent CLOSED22:30:12Z proves empty cgroup, pinned processes gone,
no held keys, VT1, stopped owned seat, absent temporary policy and unchanged
host library. All96 exported files verify SHA/size; archive200167 bytes, SHA
5b434264834dd9ffa8939165844bb3c8f0e9112f66020c34a64edac72d7a7667.
Local evidence: /home/ubu/.cache/lay/development/browser-final-fixes-20261018/
kde-private-upstream-backport-native-source-two/chrome/EVIDENCE.json.

Retained client wire has one text-input-v3 object33,455 client commits and last
done455. At the start and end of all14 native gestures, commit and last-done
serials agree; the first outgoing SSI contains the full six-character word.
This removes the exact stale-ack boundary recorded by kwdh. Reproducible
retained-data analysis WIRE-ANALYSIS-source-one.json, SHA
7c3166d580a9a1ec6d2979879cfe29d4e6e5c990d3e7a95c8d8ade77b1876efa.
The full320 browser/432 KDE identities are not established by these four cells;
next expand the same qualified harness using the original J selector and
preserved case bodies. GNOME354/46/32 and KDE175/233/24 of432 remain the
latest complete matrices, NOT_TESTED0. Installation/publication NOT RUN.

### 2026-10-18T22:42:55Z — original J432 expansion starts with qualified cleanup/export

Frozen cache kde-upstream-full-matrix-source-one/PLAN.json, SHA
65ad5f5f2dfa688bfaf824ace0723bf0dbe45efe2b765bfc56e63f07ff501300,
restores precisely the original J432 identities: Chrome160, Firefox160,
GTK32, Qt48, KWrite16 and Konsole16. Actor/stage AST equality to kbpa is
checked after normalizing only phase and the two selector Assign nodes.
Keys, case bodies, targets, PASS predicates, cadence and600/620/780s internal
deadlines remain unchanged. Each phase uses the exact privately bound upstream
dependency and the same Lay bytes/data as the four-cell control. Before input,
KWin process/maps/root and unchanged host-library evidence remain mandatory.

Actual-host source gates:6x51 PASS,6 expected schema RED/zero errors,6 zero-key
GI PASS. These are repeated package controls, not306 distinct native cases.
Outer orchestration has11 passing offline controls for timeout/error cleanup,
missing RUN, failed export and real streamed archive integrity, including
negative snapshot SHA/missing/overflow/orphan fixtures. Static review9/10.
Review corrected two harness defects before native expansion: a worker timeout
must still run the existing owned cleanup and independent export, and raw
snapshot .bin files must accompany their metadata. All snapshot references now
require exact file-set/SHA/size matching; per-file2MiB/4xphase-row bounds remain
finite and both remote archive creation and local hashing stream their input.
Pre-review pin manifests are retained. Final orchestration-pins SHA
a7d6a6cf90d5d010d31f580bffaa443c6e0718b629cbb94a4e2758539bfe3993.
Every phase must have a fresh independent CLOSED with matching phase, unit,
cgroup, original CLOSED SHA, no held keys, VT1 and unchanged host library;
failure stops the sequence before the next phase. Only all432 exact identities
with equal candidate/data/dependency pins can produce a new complete matrix.

The earlier kbpa96-file archive remains unchanged. Its16 raw endpoint snapshot
files (2,859,688 uncompressed bytes), previously omitted by the exporter, are
now recovered without native input into a separately verified supplement:
kde-private-upstream-backport-native-source-two/chrome/
RAW-SNAPSHOT-SUPPLEMENT.json. Archive156859 bytes, SHA
a50ee97d00b20d16affe6928af1923eeb4141d89352e2c76b77af5d34a21511b.
Each byte count/SHA matches the original pinned native RESULT metadata.
Snapshot interval continuity/producer completeness remain UNKNOWN; neither
full endpoint snapshots nor their recovery prove absence of producer events.

At this timestamp kbpb/Chrome is RUNNING, exact private dependency loaded,
host library unchanged. No new full matrix exists. Latest complete matrices
remain GNOME354/46/32 and KDE175/233/24 of432, NOT_TESTED0. No installation,
publication, model, Lay delivery-route or runtime-authority change.

### 2026-10-18T22:55:31Z — Chrome160 complete; Firefox expansion interrupted

The frozen original432 expansion stopped at Firefox; it did not produce a new
complete matrix. Cache `kde-upstream-full-matrix-source-one/PROGRESS.json`
retains ABORTED_PARTIAL_NOT_NEW_COMPLETE_MATRIX, finished22:46:50.552618Z.
Chrome/kbpb completed all160 original identities:130 PASS/30 FAIL/0 BLOCKED.
RESULT SHA454d9c46392466724e2461e88efe443abc78427be24902abe33a744f5049db44,
independent CLOSED22:46:29.531953Z; verified archive SHA
1fea02f75782b4fb0a902abd679d7cbeb886df5362009567fefa581e4dd7fff5.
All21 former Chrome PASS are retained;109 additional PASS are observed in this
private environment. This comparison includes the previously recorded native
IBus platform prerequisites and upstream library backport; it does not isolate
a Lay production-code effect or certify an installed user runtime.
Remaining FAIL families are prefix10, selection10 and focus_return10.
Successful measured operations only: Double Shift180 samples p50 167.812ms,
p95 178.287ms/max182.931ms; Space50 samples p50 29.068ms/p95 31.564ms;
Tab10 samples p50 31.261ms/p95 32.700ms. Refused/missing operations are not zero
latency; virtual remote-session timings do not establish hardware parity.

Firefox/kbpc retained two finished FAIL rows and a third RUNNING row when its
activity guard stopped the actor. First field10/roundtrip fails its clear
prerequisite: empty Text/caret0 but no qualifying rich-node attributes. The next
field10/short clear qualifies shapeA, then the original physical input types
`да`; Double Shift delegates exact committed-tail replay. Two Backspace events
reach the IME and tail length2 becomes0. The daemon's between-delete/insert
window check reads a different KWin UUID, later mapped to resourceClass
ibus-ui-gtk3. The actor separately reports owned app not foreground and latches
AT-SPI window:deactivate. IME trace also records Reset/FocusOutId92/FocusInId93
after the two replay Backspaces. These facts establish refusal and context
invalidation, not why the context changed, whether the panel caused it, or a
backport regression. A claim that the panel physically stole input focus was
too strong and is withdrawn pending causal evidence. Firefox's Not Responding
caption is not a hang proof: the old J caption appears in all160 rows,
including80 PASS. Current actor RESULT SHA
392f569f13efef2e6733b22a3272c301e4bf3ff7de240ea473bb3151726f5030.

Frozen success collector correctly refuses this phase: WITNESS-RESULT absent
leaves original CLOSED.actor_no_held_keys null. Do not edit or promote that
receipt. Separate read-only forensic recovery in
`kde-upstream-aborted-firefox-forensic-source-one/EVIDENCE.json` validates the
ABORTED actor's no_held_keys=true, empty cgroup, gone pinned processes, stopped
owned seat, VT1, removed temporary policy and unchanged host library. Status
is CLOSED_ABORTED_EVIDENCE_ONLY, original null retained. Archive101 files,
120239 bytes, SHA
bb090f96a2e9b14bc0466016afb68a708e381cb2941f423c92b42072cd61cc85;
12 raw endpoint files/129182 bytes include separately identified snapshots of
the unfinished third row. Recovery sent no keys and changed no old receipt.
No later app phase ran. Latest complete GNOME354/46/32 and KDE175/233/24 remain
unchanged; unexecuted cells of this new attempt remain NOT_REACHED. Next work
is the first shared Firefox context/focus transition, not a blind full retry.
No installation, publication, model, Lay route or runtime-authority change.

### 2026-10-18T23:08:06Z — Two-cell Firefox causal observation prepared

Cache `kde-firefox-panel-causal-source-two/` freezes only original Firefox
field `10` / `roundtrip` and `short`. All case bodies, keys, targets, waits,
guards, binaries and dependency remain unchanged. Read-only KWin activation
signals/window properties supplement the existing25ms snapshot. The same
private panel runs through direct `/usr/bin/env WAYLAND_DEBUG=client`, with
its exact executable/parent/cgroup/socket and actual debug environment checked
before keys. Window-transition log is capped at5000 records; overflow means
incomplete/UNKNOWN. Extra logging can perturb timing; no latency parity claim.

Options: unchanged two-cell passive trace9/10, disabling the panel2/10
(changes the behavior under investigation), ignoring foreground guard1/10
(violates authority). The new prepare preserves and pins the independently
closed ABORTED predecessor without rewriting its original null receipt.
Source-one failed the zero-key selector control because field10 was an int
instead of the original string;50 PASS/1 FAIL retained, native never started.
Source-two exact IDs pass51 remote guarded controls, expected schema RED and
actual zero-key GI preflight. Artifact manifest SHA
5931ee7f9b27c3b5a87fcbe1bf1d7d683d997253c5a60589a3c139839f6bfbf4.
Native is NOT_RUN at this entry. A positive actual clientActivated event can
establish KWin active-window change; missing signals/properties cannot prove
absence. Existing IME lifecycle records alone do not prove creation/destruction
of a client input context. No route/authority/install change; full matrices
remain GNOME354/46/32 and KDE175/233/24.

### 2026-10-18T23:08:55Z — Firefox panel activation is directly observed

The frozen two-cell `kbph` diagnostic ended ABORTED: the original iframe
roundtrip failed its rich-clear qualification; the short case failed during
exact replay, followed by `window:deactivate`. Both completed rows are FAIL;
this is neither a complete Firefox matrix nor a promotion of the prior result.
`kde-firefox-panel-causal-source-two/evidence/native-windows-gn2z_pk4/RESULT.json`
SHA e348805cc1ee122d6caa2e02be980ea40347e0a03d38c3d0038c61b76591abbe
records 20 actor input events and `no_held_keys=true`.

Positive passive evidence now goes beyond the older identity-change inference.
`logs/window-events.jsonl` has an actual KWin `clientActivated` event at
562462407.195ms for UUID cabe0a5f-977a-449d-803e-f542fde75876,
caption/app id `ibus-ui-gtk3`, `normalWindow=true`, `wantsInput=true`,
`transient=false`. Firefox UUID dd087107-33ce-4eff-8e3a-8cc3de4e3557 is
active again at562462462.015ms. The panel Wayland trace independently shows
surface25 receiving an unparented xdg_toplevel role and wl_keyboard.enter at
2065034.301ms, followed by leave at2065081.972ms on that same connection.
These are different clock bases: no unmeasured conversion is claimed. Together
they establish temporary activation and keyboard focus of the panel. They do
not yet identify which IBus widget/display request created it, or establish
the cause of other Firefox failures. The daemon rejected the changed exact
foreground between deletion and insertion; this guard remains required.

Original CLOSED SHA
c6d8c3112f211c0134033ea958233da6fddf352f34ff6f5debad9513e67afced
retains its original null actor fields. Separate independently checked
`INDEPENDENT-CLOSED-ABORTED-source-two.json` confirms teardown and exact
RESULT actor-key state; no receipt was rewritten. Export contains92 files,
109832 bytes, SHA
152b6d8a0ea12400ebf79eb2de7c7286e6209fe31b265a19ed682108fdf8b292;
all exported hashes verified. Private KWin dependency remains namespaced;
stock host library unchanged. No install, publication, Lay delivery-route,
model or runtime-authority change. Complete matrices remain
GNOME354/46/32 and KDE175/233/24. Next discriminating step: identify the panel
display operation and correct its role/lifecycle without bypassing focus.

### 2026-10-18T23:25:03Z — qualify passive private-panel method observation

The first shared failed transition remains the unparented IBus toplevel
activation during original Firefox iframe replay. CandidatePanel auxiliary
presentation is a hypothesis; the widget and method are not yet established.
GTK3 Wayland accept-focus/focus-on-map setters are empty in the checked source,
so setting those properties alone is not a qualified repair. Reusing the
existing input-panel role is a candidate, but direct-GTK caret positioning
without enabled compositor text-input is still unproved.

Prepare cache `kde-panel-method-monitor-source-one`, phase `kbpm`: the same
two original iframe identities, bodies, cadence, waits, runtime bytes and
verdicts as `kbph`. Only add bounded passive `dbus-monitor` calls on the
SO_PEERCRED-verified private IBus transport. Readiness requires observation
of GetId from the exact existing connection sender; exit or 2 MiB overflow
invalidates the observation. No panel suppression or authority bypass.
The separate zero-key probe passed and all owned processes exited; its first
startup-warning failure is preserved under `panel-monitor-probe-source-one`.
Second probe uses an isolated precreated IBUS_ADDRESS_FILE and retains
fatal-warnings; it does not alter installed discovery or the user's session.

Options: passive method evidence before repair9/10 (selected); force native
panel role now5/10 (position/visibility unproved); ignore foreground change1/10
(violates the required contract). Remote source controls53 PASS/0 FAIL/0 ERROR;
controlled predecessor RED is the expected one failure; actual GI zero-key
preflight passes. Pins SHA
51f5e676d8bf3f23720c528cff330c2f14578851b1ce0777b8c3cd54ec1fcfc3,
payload SHA
3e58195858431cb3f5dd4a72b45543812eba74584186b4b6f72ff1e843559516.
Native NOT_RUN at this entry; no speed parity or complete-matrix claim.
No installation, publication, route, runtime-byte or authority change.

2026-10-18T23:27:00Z: prepared source-two fixes the review's diagnostic-cleanup P2:
END receipt I/O now follows all mandatory process/device cleanup, catches
OSError and hashes the log after its writer is closed. Missing END remains
UNKNOWN. Source-one and its successful source gates are preserved; neither
preparation sent keys. Source-two remote53 PASS, expected RED, real GI zero-key
PASS. Pins04f52fce45a948a06492a09c5ea3c85de9efb97268863c1ef5253dba58a5440e;
payload1078e50eb64b371e41b1246774fc2ba35aeaf9e30dc277b7b2e2ff9e23bcbf4a.
No actor/runtime/verdict change and native NOT_RUN at this entry.

### 2026-10-18T23:31:30Z — cleared auxiliary hint still maps a normal toplevel

Phase kbpm source-two reached both unchanged Firefox iframe identities:
2 FAIL, then ABORTED on the same window deactivation. RESULT
`native-windows-1807byof/RESULT.json` SHA
d413f475dc2cdedd92d0302ef5f1c34e4f1b11a5e06a9f1808a1e393227cb224;
no held keys. The independently closed95-file export is in cache
`kde-panel-method-monitor-source-two/evidence`; archive SHA
953a69a0d86abfb7968d77ac7054f05f042f77f10dceb349fae1429db0a3890e.
Review source-two9/10, no remaining P1/P2. No full matrix promotion.

The bounded private-bus observer remained alive through cleanup; final log
92687 bytes, SHA
cc09b4219cfa8d0a39dccca9200c66f53873488d3794c9337ccbd96ed014575c,
verified against the stable closed-writer receipt. Positive visible commands
are auxiliary text; PropertyPanel's auxiliary handler only calls
hide_if_necessary. Last auxiliary show was approximately563584457.672ms,
clear approximately563584651.333ms; real KWin clientActivated for the
ibus-ui-gtk3 normal toplevel was563584698.444ms. The first two estimates
map D-Bus realtime using start/end observer clock pairs, whose offset differs
by4.147us; they are approximate correlation, not a latency benchmark.
Wayland keyboard enter/leave independently confirms focus on that surface.

Upstream CandidatePanel delays both an auxiliary payload and update()100ms
each. Clear currently waits another100ms before cancelling the pending show;
this permits a cleared auxiliary hint to map during exact replay. Selected
minimal experiment: cancel the auxiliary timer and apply null immediately;
the existing update() cancels the pending show. Keep nonempty coalescing,
lookup behavior and all Lay runtime bytes/foreground guards/routes unchanged.
This lifecycle repair is8/10 for the demonstrated stale-show defect; role reuse
is8/10 for the broader focus defect but caret placement is unproved; merely
setting GTK focus flags is1/10 because that backend's setters are empty.
This does not prove a legitimate long-lived hint can never activate the
fallback toplevel. That separate presentation-role limit stays open.

Next: isolated upstream panel copy/build, stock RED/candidate temporal
regression if feasible, then the same native identities. Native candidate
NOT_RUN. No installation or user-session change.

### 2026-10-18T23:36:34Z — minimal panel lifecycle candidate built and controlled

Remote guarded private build `ibus-panel-clear-build-source-one` changes only
CandidatePanel's null auxiliary setter: cancel and zero its timeout, apply
null synchronously, retain the existing update() and nonempty coalescing.
Exactly one prefix file changes, `libexec/ibus-ui-gtk3`; candidate SHA
e6c42ff073fe8017cffa866b1bf9be765efa4afcea95474ddb326a72dd18b013,
source SHA55c3201f99c2e1096cbb0a99e0464978e4b37e8b984705d661dd0d99978993bd,
patch SHA cfe6d1e27d9b6360f24615373ab3fd041b47decf7de28476027f0a9ca99591d7.
The original upstream source/prefix remain byte-identical. No system install.

Actual extracted setter/update GLib controls qualified the real order:
clear after payload application while show is still pending. Stock returns
the expected RED with one stale show; candidate returns GREEN with none.
Both subsequently show one ordinary nonempty hint. The fixture requires its
candidate methods to equal the actual built candidate source; real MainLoop
and Timeout, fatal-warnings, finite deadlines. Label/window methods are state
seams; this proves lifecycle, not GTK/KWin/focus acceptance. Receipt in cache
`ibus-panel-clear-controls/RESULT-source-one.json`; remote
`ibus-panel-clear-controls-source-one/RESULT.json`. Review9/10, no P1/P2.

Native phase kbcl `kde-panel-clear-source-one` selects that private executable
in KWin, retains the passive method/wire observers and unchanged original
two iframe actor/stage/controller bodies, cadence, waits and verdicts.
It verifies the independently closed predecessor and candidate build pins.
Source53 PASS/0 FAIL/0 ERROR; expected predecessor RED; actual GI zero-key PASS.
Pins cbc29eefcf0cd2841fed7173a5b965befa2348558e2933abb08059627a5676a7;
payload36dc611f6dd91f7bff8f92a8d737e12027ba98065675ea4416a1806d07cc3ba9.
Native candidate NOT_RUN at this entry. The legitimate persistent-hint role
is still unqualified. Lay bytes, routes, focus guards and authority unchanged;
complete matrices remain GNOME354/46/32 and KDE175/233/24.

2026-10-18T23:38:43Z: native preparation reviewed9/10, no P1/P2. Before keys,
source-two corrects inherited controller metadata: the experiment is an
upstream panel candidate, not instrumentation-only. Actor/stage/verdict logic,
selected binary and prerequisites are unchanged; source-one sent no keys.
Source-two pins65b4fa8d2d624124dd62dca63be5aadb66d2f45c47e15c7478dc4ba49fa47cb0;
payload5939a94fe262c19b4acc4db6ab3127e0fb44c390875913a66070a533f6380af4.
Native remains NOT_RUN until the fresh pinned source gates pass.

### 2026-10-18T23:46:02Z — private null-clear reaches native short-word PASS

Two preparation errors are preserved separately from behavioral failures:
kbcl source-two stopped before launch because outer orchestration and launch
both opened launch.log exclusively; kbcn stopped before actor/input because
bwrap's newly created candidate-prefix parent was not traversable by uid1000.
Neither reached native rows. kbcl zero-input closure and kbcn's independent
setup closure are pinned in `ibus-panel-clear-build-source-one` receipts.
The latter's original null actor_no_held_keys is preserved, not promoted.

Zero-key namespace control reproduces PermissionError without the private
parent chmod and reads the exact candidate as uid1000 with it. Host parent
permissions and the installed KWaylandServer remain unchanged. Receipt
`ibus-panel-clear-build-source-one/PREFIX-READ-PREFLIGHT.json`; this proves
access, not client behavior. Corrected kbct payload
df1fb3e1d4fbd6239a533ab844aed8c592d8446eafb26e7d8e63e6b9fd1e6038;
artifact pins46fa28e5e287cc7800d4391815b13de68ab4aa4470edf0f6c94b23243e9732c2.
Fresh remote53 PASS, expected RED, actual GI zero-key PASS.

Native kbct completes both original Firefox iframe identities: roundtrip FAIL
at the rich-clear prerequisite; short PASS with the unchanged actor/verdict.
The first clear has empty text, caret0, no selection, correct focus/identity
and stable complete tree, but get_attributes() returns an empty mapping.
The next clear on the same accessible reports the required div/textbox/block
attributes and qualifies. Do not weaken the shape predicate or regrade that
first row. Its behavior never reached Double Shift. Firefox remains foreground
in both rows; there is no reproduction of the previous panel focus takeover
in this two-cell run. A legitimate persistent hint's presentation role still
needs its own acceptance. This is one observed recovered identity, not a
complete-matrix improvement or proof of every hint case.

Two successful Double Shift observations: median129.321ms, p95147.563ms,
max147.563ms; write-return-to-exact-target observer timing on private VKMS.
N=2 is diagnostic, not a latency distribution or performance improvement.
RESULT `native-windows-17ofw9y4/RESULT.json` SHA
a3d3c5087383e7d42d5d3855cec4b639a1dc3495202398b29597cda9df821862.
Independent CLOSED verifies no held keys, original pinned processes gone,
empty exact cgroup, owned seat inactive, policy absent, VT1, host library
unchanged.97-file SHA-verified export in cache
`kde-panel-clear-qualified-source-two/evidence`; archive SHA
8f4657903c50cf7ed582e9bb3b188db069d10f6f27ae46cc07194ae1ff28be5a.
Lay bytes/routes/authority and installed runtime unchanged. Full matrices
remain GNOME354/46/32 and KDE175/233/24;430 original cells not selected.

Next: qualify cold metadata readiness within the existing clear deadline,
retaining the first raw sample and all shape/identity/focus guards, before
claiming the original roundtrip behavior. No extra clearing keys or retry of
the behavioral case is authorized by a prerequisite observation.

### 2026-10-18T23:55:33Z — strict clear-readiness source gate, native pending

Decision `2026-10-19-native-clear-readiness-deadline.json` scopes observation
to the existing clear prerequisite: one physical clear, unchanged complete
shape predicate, original1.8s deadline, at most8 captures. Retain every sample;
one individual complete capture must qualify before its post-capture deadline.
Foreground/seat/identity errors and incomplete, unstable or erroneous tree
reads stop immediately. No change to measured case actions or target verdicts.

Remote kbcs source gate:58 PASS, zero skips, actual GI zero-key PASS. The
preceding actual clear callsite fails the new pinned cold-then-complete
control as expected (1 assertion failure,0 errors); the candidate passes.
Other controls cover persistent missing metadata, read/identity failures,
late otherwise-valid capture and guard failure. These are source proof only.
Local package `kde-panel-readiness-stock-source-one`, artifact pins
a96630d79fcc97560835885b1c6de96d4b2abbb41cef6f01e30f738030053e06,
payload01fcbea982bfe576fa7df96cd83e6a03e7e8e54beb92c9d0fc3b1919646faa48.
Remote source receipts are in `kbcs-scripts-source-one/` under the development
runner. Native remains NOT_TESTED. Compare stock and candidate panels under
this same observation contract; extra settling may change the timer race.
Original kbct failure and all complete matrix totals remain unchanged.

### 2026-10-18T23:58:22Z — readiness review corrections, source-three qualified

Independent source-one review7/10 found two P2: an explicit raw/root focus
failure could be retried, and a received tree could be lost if the following
guard/layout check threw. Source-one is rejected for native use despite its
58 passing source controls. No native input used it; local intermediate
source-two and the unrun manifest-review snapshot likewise sent no keys.

Source-three saves a received tree before postchecks with POSTCHECK_PENDING;
postcheck exceptions retain it as PARTIAL_UNQUALIFIED with the failing stage,
then propagate. Explicit raw or root focus failure is hard even if a later
capture could recover. The unchanged strict predicate still supplies all
positive qualification. Added/strengthened tests prove false raw/root focus
cannot retry and guard/layout errors retain the exact tree without retry.

Remote source-three59 PASS, zero skips; original callsite expected RED and
actual GI zero-key PASS. Stock control phase kbcu, cache
`kde-panel-readiness-stock-source-three`; artifact pins
90fdf97bc6345f94b4a507344c0332e8f77a942b29641de6f0ee47d9b9e042a1,
payload60ed93093b76981c88571cdfa03069047174bdd655708cc78d1a6f3ea34ae30f.
Native remains NOT_TESTED, stock/candidate comparison pending. No Lay binary,
runtime authority, delivery route or complete-matrix promotion.

### 2026-10-19T00:16:50Z — source-three native pair closed; one Firefox focus failure isolated

This entry supersedes the preceding source-three native-pending status. Both
finite sessions are independently CLOSED with released keys, empty owned cgroup,
original VT1 restored, owned seat stopped and installed host library unchanged.
The stock actor is ABORTED: iframe10 roundtrip FAIL, subsequent short row RUNNING
at interruption. The candidate actor is ABORTED: iframe10 roundtrip PASS, short
FAIL (`owned app not foreground`). Interrupted rows are never promoted to FAIL
or PASS. The stock export was repaired to preserve an ABORTED actor's last
RUNNING row; this is an evidence-export change only, not a relaxed native gate.

Paired receipt: cache `rich-clear-readiness-source-one/PAIRED-NATIVE-source-three.json`,
SHA aa34369c6a3b547c60a4fb7e308d04268247b48c54386ffbf0aa8901607c9d1b.
Stock RESULT SHA c5d984f0a70fc77177eb52d51c9ec5ebe5ee23d57c3f1602216a5bd8c13cbb49;
candidate RESULT SHA 78bbc13e8644793d31b5a8423eeb7ca5ec2b527f539dbf58a86371e3fb58d694.
Their 97-file verified exports have archive SHA, respectively,
f56fedf26bc0b2e3af45a1f2a26c5d105197cfd203652d0df8f4ac5904aef82a and
0d928dd54689de140185ba77a7f9f2abf65edb188ca9664f3a6d23b8637e9f83.
The timer-clear candidate alone is insufficient for foreground acceptance.

User narrowed active work to ONE KDE failure: Firefox iframe10, committed `да`,
caret2, focused, no trailing Space, physical Double Shift, expected `lf`.
The target was not observed. Daemon trace recognizes the manual trigger, then reports
exact-layout handoff failure because the active window changed to the panel.
The earlier explanation "between deletion and insertion" is not established
and must not be reused as the first failed transition.

A standard-library read-only tool was created in cache
`focus-timeline-source-one/kde-ibus-focus-timeline.py` and copied to remote
`/home/e/projects/lay-development-runner/tools/focus-timeline-20261019/`.
SHA 612baabdc5b865c26d88e5dc2b46c89d7523ae8f3f2c41566003b3b898342258
matches locally/remotely. Three historical row verdicts are preserved. The
remote and local reports agree, including all six input hashes, modulo source
path names. Verification receipt is `focus-timeline-source-one/VERIFICATION.json`.
No new native input, session, build, installation or runtime change was made.

The recorded paired clocks approximately place auxiliary `вай`, visible=true,
35.934ms BEFORE the first Shift key-down write return. The second Shift release
is +140.911ms; KWin's observed panel activation is +199.630ms and includes
observer delay. Start/end clock-offset change is -0.001007ms; unbracketed paired
samples do not establish an accuracy bound. The raw Wayland trace has its own
unmapped clock and is not subtracted from these timestamps. It independently
shows the panel's xdg_toplevel role, nil parent and keyboard enter on its surface.
The nonempty auxiliary request remains active for about218ms. Clearing stale
timers cannot prevent every legitimately requested nonempty panel map.

Source inspection finds the fallback `get_active_candidate_panel()` branch
when `m_wayland_object_path == null`; it skips custom input-panel surface setup.
The context is assigned from Wayland input-method focus signals, not merely
ordinary IBus FocusIn. The harness explicitly sets GTK_IM_MODULE=ibus while
Firefox uses native Wayland. In the exact upstream IBus1.5.34-rc2 source,
bus/inputcontext.c:127-133 explicitly documents CandidatePanel focus in this
configuration. This supports the configuration/fallback hypothesis, but the
internal branch value and originating widget-to-surface binding were not directly
measured. Do not claim the failure is fixed or an iframe-specific Lay core bug.
GTK3.24.33 Wayland set_accept_focus and set_focus_on_map implementations are empty;
the corresponding GTK flags alone are not a justified fix.

Next scope stays one case: prove runtime branch selection and widget/surface binding,
then choose a minimal fix. Do not suppress foreground guards, change input routes
to obtain PASS or restart broad matrices. Full native totals remain GNOME354
PASS/46 FAIL/32 BLOCKED and KDE175 PASS/233 FAIL/24 BLOCKED. Runtime authority,
delivery routes, installed bytes and publication state are unchanged.

### 2026-10-19 — one-case KDE panel presentation policy, two native cases pass

The first failed transition remains the exact-layout handoff losing Firefox
foreground to a mapped IBus popup. Decision
`decisions/2026-10-19-kde-ibus-panel-focus.json` selects an existing compositor
presentation rule rather than changing GTK IBus delivery or ignoring focus.
KWin5.24.6 `XdgToplevelClient::wantsInput()` already applies
`rules()->checkAcceptFocus(acceptsFocus())`; the packaged rule matches exact
normal-window class/caption `ibus-ui-gtk3` and forces acceptfocus=false.
It must not suppress the visible hint, reactivate an application, or loosen any
Lay owner/foreground check. Its namespace is the disposable KDE configuration.
No installed user configuration or Lay binary is changed.

Frozen package `kde-panel-focus-rule-source-two`, phase kbcw, artifact SHA
6ef75511c9fd7de7f721b1580f29743452f8c5ff7f831e091d93ffd52e71c905,
payload SHA17068ce58f2b20838e818637a66c64745b72f048f7a0033bba9616149477cc16.
Remote source59 PASS, both existing controlled REDs detected, actual GI API
zero-key PASS. The source-one package stopped before test collection on a
stale PLAN metadata hash; no native input used it. Source-two pins the final
PLAN once and removes duplicate archive members. No behavioral assertion changed.

The unchanged original iframe10 roundtrip and short cases both complete PASS:
`привет → ghbdtn → привет` and `да → lf → да`, including the following-letter
controls in both layouts. Receipt
`/home/e/projects/lay-development-runner/kbcw/native-windows-h7hpls_q/RESULT.json`,
SHA4536636f7dbbc6346de60110a42e280cfdeb32592ec2a873e2bea4e0a6ce75bc,
has COMPLETE, two terminal PASS rows and no held keys. Two normal IBus panel
surfaces actually map and receive committed buffers; KWin observes both with
wantsInput=false, never active, while Firefox remains the active application.
The panel Wayland trace contains no keyboard-enter/leave event. This positive
witness distinguishes the focus fix from merely avoiding presentation of a hint.

Four successful measured Double Shift operations: median135.655ms,
p95/max144.157ms, write-return to exact native AT-SPI text observation.
This small diagnostic sample is not a broad performance result; kernel key-release
latency is not measured. Verified 100-file export SHA
341fb0cafdaa6cc626028e08f82c064145c6ae313f9606c7369482027156fb02 is retained at
`/home/ubu/.cache/lay/development/browser-final-fixes-20261018/kde-panel-focus-rule-source-two/NATIVE-EVIDENCE.tar.gz`.
Its independent closure confirms VT1 restored, no held keys, empty owned cgroup,
pinned processes gone, temporary policy absent and installed host library unchanged.

Next scope is the same iframe with the original burst5 and space_burst5 actions;
keep the first two cases as controls. The packaged rule has only private native
acceptance, not automatic installer integration. Full native matrices remain
GNOME354/46/32, KDE175/233/24; this is not a matrix promotion. Runtime authority,
delivery routes and installed user configuration remain unchanged.

### 2026-10-19 — original iframe repeat controls, four native cases pass

Phase kbcx selects exactly four existing Firefox iframe10 identities: roundtrip,
short, burst5 and space_burst5. Case actions, targets, timing, original-selector
membership and owner checks remain unchanged. The only presentation policy is
the accepted private kbcw KWin rule, with the same Lay/panel/dependency bytes.
Source-one stopped before native setup: its controller cardinality test still
pinned the two-case selector. Source-two updates that metadata expectation to
the exact four-case selection and retains all behavioral assertions. Remote
59 PASS, both controlled REDs detected, actual GI zero-key PASS.
Artifact202e2937493b64cdbbe22ff2894e63b81f6a8fae5f7139bc1bd41615ed082c78;
payload6811097656913583db6b6db261e19543c05985209841d83cd11dfdddf8b228e1.
Package `/home/ubu/.cache/lay/development/browser-final-fixes-20261018/kde-panel-focus-followup-source-two`.
Native kbcx completes all four original identities PASS, including both five-repeat
cases and following-letter controls. RESULT SHA
c01a8d7517ceb12a1cbc23ac051b190bc0829588e2f60c8b891a0050ac006cf1,
`/home/e/projects/lay-development-runner/kbcx/native-windows-f356mlva/RESULT.json`.
Fourteen successful Double Shift observations: median152.574ms,
p95/max176.499ms, same write-return to AT-SPI clock. An actual mapped IBus panel
again has wantsInput=false while Firefox remains active. Verified120-file export
SHAb5a2b97b5c4ae7490c7cbb6e73b99c6965aa5c517c49f94b8455d056a1428ad6,
`kde-panel-focus-followup-source-two/NATIVE-EVIDENCE.tar.gz` under the same local
development evidence root. Independent closure at2026-10-19T00:49:02.277596+00:00
confirms VT1 restored, released keys, empty owned cgroup, processes gone, absent
temporary policy and unchanged host library. The 428 other original cells are
not selected by this partial diagnostic; complete totals remain unchanged.

### 2026-10-19 — shared rich-field focus mechanism, all24 original cases pass

Phase kbcy extends only the existing selector: fields7,8,10 in Firefox and Chrome,
four original plans per field (roundtrip, short, burst5, space_burst5). Two existing
application jobs run sequentially, 12 rows each. The same native actions, targets,
cadence, clear prerequisite, foreground guards, binaries and KWin rule are pinned.
Remote59 PASS, both controlled REDs detected, actual GI zero-key PASS.
Artifact91eddb443c18a57a45c4c2c5a3315a99a1c460587c4fff19bda944d287d0094c;
payload8bb4c0c851c0e2447553a9d9789861ff799b429010e2bbcd6eb85ac22cb4621d.
Package `kde-panel-rich-followup-source-one` under the retained local development
root. Native results: Firefox12 PASS and Chrome12 PASS, zero FAIL/BLOCKED,
all24 original selected identities complete with no held keys. Firefox RESULT
SHA1578aceb74e6bf8ae03604383cfddbe8531fceeb92b849db87290c4f3b23b9ee,
Chrome RESULT SHAd493dce91f28d05293b6635c2475553b304e83c49cdec7d19f6f01b4926c0825.
Remote receipts respectively `kbcy/native-windows-d67jof3t/RESULT.json` and
`kbcy/native-windows-htp_6aom/RESULT.json` under the fixed remote runner root.
Seven actual normal IBus panels map with wantsInput=false; none activates.
The application remains foreground.84 successful Double Shift observations:
median150.225ms, p95173.559ms, max177.705ms; same write-return/AT-SPI clock.
Firefox median146.819/p95171.678ms; Chrome median158.652/p95174.238ms.
These are measured successful operations in the selected diagnostic, not a
whole-desktop latency or a demonstrated speedup over a different baseline.

Verified324-file native export SHA
8f8c9a87d0b6c5989421f61c7e9c5ab2d0867a3bb9cbafa21b70cc1779c14dad,
`kde-panel-rich-followup-source-one/NATIVE-EVIDENCE.tar.gz` in the local evidence
root. Independent closure at2026-10-19T00:53:44.104230+00:00 verifies VT1,
released keys, no owned processes/cgroup/policy, stopped owned seat and unchanged
host library. The 408 unselected original cells and full matrix totals remain
unchanged. This is not a broader route migration or user installation.

Source packaging now invokes the existing KWin rule through a KConfig helper
from install.sh and unregisters it from uninstall.sh. The helper stages changes
in a temporary copy, retains named/legacy user rules and checks the original
before replacement; removal retains user-edited values and unrelated keys.
All seven final tests pass, zero skips, against the actual installed KConfig
tools in remote disposable files, never a live profile. The no-KDE absent-rule
removal is a clean no-op. Both install.sh and uninstall.sh pass remote bash -n.
Receipt `kde-panel-rule-installer-source-two/RESULT.json` in both retained local
and remote development roots pins the tested source. Native qualification of
the helper-generated rule remains pending.
Runtime authority, delivery routes and installed user bytes remain unchanged.

### 2026-10-19 — complete Firefox160 component with packaged rule, native pending

Phase kbcz selects exactly all160 original Firefox identities from the frozen
432-cell selector. The private launch uses the exact seven-test-qualified
production KConfig helper to create its rule before KWin starts. The helper
and template are pinned package data, copied into a disposable repo-shaped
directory; only the explicit private kwinrulesrc is changed. Every native case
body, target, cadence and guard, plus Lay/panel/dependency bytes, is unchanged.
Artifact31ed958293e594d24ed279687b3971dcc31372dea5dd62d5877343c7f304133d;
payloadb4b4a32f0c337cf56d2d10f4ebdfe7df22132440a70a10d30fea478c034a7bee.
Package `kde-firefox-policy-full-source-one` under the fixed local evidence root.
Remote source59 PASS, both controlled REDs detected, actual GI zero-key PASS.
Native qualification pending. Other272 original identities are not
selected. A complete Firefox component is not a new complete432 KDE receipt.


### KDE panel rule: full Firefox component and installer migration — 2026-10-19T01:10:37.484536+00:00

The first full Firefox attempt `kbcz` was ABORTED after64 terminal rows
(48PASS16FAIL) because the passive IBus method log reached its2MiB file ceiling.
This was a harness resource failure, not a new application verdict. Preserved
export: `kde-firefox-policy-full-source-one/NATIVE-EVIDENCE.tar.gz`, SHA256
`5ca67cc54fc373ad5b03ed931ae7d5ff3f4a73fffeec71eb7cedf14d70bf156f`.
Independent closure verified VT1, released keys, empty owned cgroup and absent
seat/policy; stock dependencies unchanged. No prefix is counted as a full run.

The next immutable attempt `kbda` changes only passive-log allocation from2MiB
per24 selected cells to14MiB for160; overflow still aborts. Original actions,
targets, timing, case selection, guard conditions, Lay/panel/dependency bytes
and helper output are unchanged. All160 original Firefox identities finish:
**123PASS /36FAIL /1BLOCKED**, versus historical80/73/7. Native receipt
`/home/e/projects/lay-development-runner/kbda/native-windows-qflw2qkp/RESULT.json`,
SHA256 `d87d3680e5c961b9de7f93d25ab07a906300d892df397015dbced8d3b22b421e`.
All59 actual normal IBus panels have wantsInput=false; zero panel activations.
178 successful measured DS operations: median164.587ms, p95178.964ms,
max191.026ms.45 successful Space observations: median30.212ms, p9532.225ms.
Clock: evdev write-return to exact native AT-SPI target, not kernel key-release.
Failed/refused operations remain in the denominator and are not latency samples.

Verified export `kde-firefox-policy-full-source-two/NATIVE-EVIDENCE.tar.gz`,
SHA256 `4570cf01cb37212d477b708cc8a106ca1cb31a6f561ab3c5d0b51e8b4710053d`:1680files,
640 bounded raw trace snapshots (194317462bytes),12577801compressed bytes.
Independent CLOSED confirms VT1, released keys, empty cgroup, pinned processes
gone, owned seat/policy absent and unchanged host library. This is one complete
160-cell component; other272 original cells have not been rerun under this rule.
The historical full GNOME354/46/32 and KDE175/233/24 matrices remain separately
labelled historical.24-cell and earlier2/4-cell evidence overlap and are not added.

Remaining Firefox failure groups: prefix10, selection10, focus_return10,
space_en_ru2, space_ru_en2, long1, moved_boundary1. A correctly rendered prefix
can still fail layout: Lay InputMode changes while physical KDE group staysRU
on the owned preedit path. NBSP normalization already exists in the actor;
this is not evidence of a missing NBSP comparison. Selection/focus-return
refuse context authority. A fresh surrounding callback can occur before native
activation publication; the admission transition still needs causal diagnosis.
No guard bypass or Rust fix has been made from that hypothesis.

The actual KWin5 configuration also exposed a packaging lifecycle defect:
named rule registration becomes numeric and leaves an orphan original group.
The helper now follows the exact installed description/popup match, avoiding
duplicate registration and removing unchanged keys from both representations.
The native520-byte configuration is the regression fixture (SHA256
`b352e5a62da7e66bdd296207bbc5d69383b13b9c1d523a89ae0a41560670315c`).
Remote `kde-panel-rule-installer-source-three/RESULT.json`:9/9tests PASS,
zero skips, real KConfig tools; install/uninstall bash syntax PASS. These newer
helper bytes were not substituted into the frozen Firefox run; only its fresh
configuration output was used there. No user installation or publication.
Runtime edit authority and IME delivery routes unchanged. Next: Chrome160 on
the same presentation policy, then one shared admission/layout failure at a time.

### KDE panel policy: Chrome160 complete — 2026-10-19T01:19:00.540675+00:00

The independently closed Chrome component `kbdb` completes the same original
160 identities:130PASS/30FAIL/0BLOCKED. Firefox `kbda` remains123/36/1.
Together the new browser components are253PASS/66FAIL/1BLOCKED of320;
the preceding browser baseline was101/205/14. These are browser components,
not a new432-window matrix; the other112 KDE cells and GNOME were not rerun.
The three Chrome groups are exactly10prefix,10selection,10focus_return.

Chrome receipt:
`/home/e/projects/lay-development-runner/kbdb/native-windows-we_cszj_/RESULT.json`,
SHA256`416f1166f1a09d2fc3f53b938d85f4a6045e63c40f634897e5d9417ebbd0c495`.
Verified export:
`/home/ubu/.cache/lay/development/browser-final-fixes-20261018/kde-chrome-policy-full-source-two/NATIVE-EVIDENCE.tar.gz`,
SHA256`13aca263c2452b03f54e1e5604218e51376a0687b444282e57e6b939f85d155f`;
1680files,640 bounded snapshots,194245107 raw snapshot bytes.
Independent closure at the heading timestamp verifies released keys, empty
cgroup, stopped pinned processes/seat, absent temporary policy, VT1 and
unchanged host library. CLOSED.json SHA256
`d198e3108d3201c7c7cd8aa47b8bfbb31c08a5333ff6d9625e03feeeae08687f`.
Passing-cell Double Shift N180: median167.546ms,p95 nearest-rank178.065ms,
maximum193.015ms; Space N50:median29.229ms,p9531.376ms;
Tab N10:median30.427ms,p9533.698ms. Clock remains evdev write return to
the exact native AT-SPI target; kernel release is not measured. Failed cells
are excluded from these successful timing samples, never counted as success.

### KDE open preedit group: causal RED — 2026-10-19T01:23:08.374972+00:00

Both full browser receipts show every prefix cell with correct text/caret,
no selection and native InputMode=US, but KDE group=RU. The first missing
transition is `replace_verified_owned_preedit`: it publishes native InputMode
and promotes its decoder while intentionally retaining its current IBus
engine, without completing the KDE group. Four additional Firefox Space
layout mismatches are separately attributed; this fix does not certify them.

Decision `2026-10-19-kde-owned-preedit-layout-group.json` reuses the existing
bounded KDE adapter and latest-generation direct dispatch with a required
live layout token. No new owner/worker, IBus engine replacement, text route,
foreground exemption or relaxed native target is introduced. Before the
implementation, remote `kde-owned-preedit-layout-red-source-one/RESULT.json`
records exactly0PASS/3FAIL semantic regressions, no compiler error. Its overlay
SHA256 is`5a03b6745262a4115a75cc205c0c06acc0b92a47baa9dbb67e85a09d0768c8df`.
The tests cover eight intermediate group/mode/preedit states, preservation of
the selected engine and committed prefix, next physical letter, rejected
group rollback under a live owner, and refusal to roll back into a lost owner.
Implementation is present; GREEN, private build and native acceptance remain
pending. Runtime edit authority and IME delivery routes are unchanged.

### KDE owned-preedit group: isolated source GREEN — 2026-10-19T01:38:19.105893+00:00

The three controlled regressions now pass. The official discovery and lane
classification find855 IME tests:852 correctness tests PASS (531 together,
321 in their required individual process sandboxes), plus3 separately isolated
performance tests PASS. No expectation, isolation rule or guard was weakened.
Receipt `kde-owned-preedit-layout-hermetic-source-five/RESULT.json` in the
fixed local and remote development roots pins discovery, source and results.
The source archive is immutable and has no Git metadata; explicit source
closure replaces Git-based discovery only. An earlier all-in-one run's two
cross-test process-state failures do not reproduce under the required lanes.
Two intermediate harness attempts failed on Cargo JSON framing and the
852-correctness/3-performance denominator; neither produced native verdicts.

The private remote IME build is8458304 bytes, SHA256
`387fcc8b8a549ca3472c11d5ff4c870b5bb6343ccdeb172d0a93e477e67f4f22`.
It still embeds the preceding architecture graph receipt. Graph refresh and a
final receipt-bound build must therefore precede native qualification; this
build is not installed or accepted. The fixed runtime behavior has not yet
been tested in a physical client. Latest complete browser components remain
Firefox123/36/1 and Chrome130/30/0 (253/66/1 of320); other112 KDE cells
and GNOME remain historical. Runtime authority and delivery routes unchanged.


### KDE owned-preedit group: private native prefix qualification — 2026-10-19T01:47:30.372107+00:00

The remote graph/architecture gate completed before the final build. Graph
receipt `kde-owned-preedit-architecture-source-two/RESULT.json` SHA256
`46d544db6bc48721f75a18b0bf0d643c7eee9946bf5b89d104b38cb144372d52`;
embedded generated receipt SHA256
`85d4a7fc388d6e5a3f60f08967b0a768eadd117dd063cdb9feb7cf2ba7b71b2f`.
The final private IME is8458304bytes, SHA256
`10155e00a93337d9243f2298793b82fbafd12e73b24529d5134bbb38d1998174`.
`kde-owned-preedit-layout-final-build-source-one/RESULT.json` records the
receipt-bound build, passing embedded receipt test and reuse of852 correctness
and3 separately isolated performance tests under identical Rust source pins.
No local installation or publication occurred.

The original20 prefix identities ran unchanged in `kdbc`: Firefox10PASS,
Chrome8PASS/2FAIL, no BLOCKED. Each passing cell asserts both conversion
directions, exact text/caret/selection, KDE group/native InputMode and the next
physical letter. Firefox N20 successful measured Double Shifts:median29.467ms,
p9530.736ms,max33.044ms; Chrome N16:median29.966ms,p95/max34.134ms.
Clock remains evdev write return to exact native target, not kernel release.
Failing operations are retained and excluded from successful timing samples.
This selected20 result must not be added to, or replace, the old-IME complete
320-cell matrix253PASS/66FAIL/1BLOCKED. A complete new-IME320 run is pending;
other112 KDE cells and GNOME remain historical.

Firefox receipt `kdbc/native-windows-ini8azjx/RESULT.json` SHA256
`1e231bc153f5a176d40194e5187b9000515583d6645816f96423854304cc5d41`;
Chrome `kdbc/native-windows-csofrtg3/RESULT.json` SHA256
`e61ea7e316fdaa6197aa29a6b1c0823f997ad8cef944a42a16dfebf7e1c11a0e`.
Verified export `kde-owned-preedit-native-prefix-source-one/NATIVE-EVIDENCE.tar.gz`
SHA256`1cb2fa2a9523cd17cf4437b93d1b7215ee9f5212e0fabc8ca82dfdbd6ba88ce9`:
286 files,80 bounded snapshots,19372346 raw bytes,1560760 compressed bytes.
Independent CLOSED at the heading timestamp proves released keys, empty owned
cgroup, stopped pinned processes/seat, absent temporary policy, VT1 and the
unchanged host library.60 source controls, controlled schema/readiness RED and
real GI zero-key preflight passed. No target, cadence, delivery route or edit
admission was weakened.

First residuals remain distinct. Chrome field9 iframe reaches exact transformed
text and native InputMode=US, but KDE group remainsRU through the deadline;
existing logs do not prove whether group completion was skipped or reverted.
Chrome field2 Search fails BEFORE Double Shift: AT-SPI returns `тест, приве`
with caret12 while the IME's active composition still contains complete
`привет`. This is a client observation/presentation discrepancy, not proof
that the final letter was lost. Neither result may be recast as PASS.
Next: unchanged complete original320 proof, adding a bounded passive session-bus
observer for actual setLayout calls/layoutChanged signals, qualified before
input and closed with the existing owned session. No repeated-until-PASS run.
Runtime authority and IME delivery routes are unchanged.

### KDE owned-preedit group: complete browser qualification — 2026-10-19T02:04:16.607557+00:00

The original320 browser cells completed with private IME SHA256
`10155e00a93337d9243f2298793b82fbafd12e73b24529d5134bbb38d1998174`:
Firefox139PASS/21FAIL/0BLOCKED; Chrome136PASS/24FAIL/0BLOCKED;
combined275PASS/45FAIL/0BLOCKED. Relative to the preceding complete private
components253/66/1, this is22 additional PASS,21 fewer FAIL and one fewer
BLOCKED. Other112 KDE cells, GNOME, private/fullscreen modes and user runtime
were not regraded. TD159 remains IN_PROGRESS; no user install or publication.

Original native case ASTs,16 plan identities per browser, ten fields, targets,
cadence, deadlines and admission/layout assertions are unchanged.60 source
controls, controlled readiness/schema RED and real GI zero-key preflight pass.
The only additional observer records bounded session-bus setLayout calls and
layoutChanged signals; readiness precedes input and its verified end covers
the last native stage. It adds no runtime authority or retry.

Receipts under `/home/e/projects/lay-development-runner/kbdd/`:
Firefox `native-windows-bie5c_g5/RESULT.json`, SHA256
`fb35045e07d22fe398fb2e32e340e12fd2dee40ac68163a6019bf3313c182481`;
Chrome `native-windows-78llbwrj/RESULT.json`, SHA256
`b84082bc98434b342321b5972e7e5c60420cf0418d9adb0a68190f2ffe5825a0`.
Firefox successful Double Shift N200:median159.264ms,p95178.965ms,
max201.716ms; Chrome N192:median167.206ms,p95178.609ms,max194.445ms.
Space Firefox N49:median29.346ms,p9533.192ms; Chrome N50:median29.259ms,
p9532.845ms. Tab each N10:Firefox median29.998ms,p9532.695ms;
Chrome median29.689ms,p9533.058ms. These are evdev write-return to exact native
target timings, not kernel release timings. Failed/refused operations remain
in the receipts and are excluded from successful speed samples.

Verified local export
`/home/ubu/.cache/lay/development/browser-final-fixes-20261018/kde-owned-preedit-native-full-source-one/NATIVE-EVIDENCE.tar.gz`
SHA256`1dbac3eb00698e72bb162284c48daa88228bde767fa55f6edcf47b12ca5fa6e2`:
28963997 compressed bytes,3290 files,1280 snapshots,457060711 raw snapshot
bytes. Native CLOSED SHA256
`7065d8828967dbda19971a73f8d51c5cc06ec870ccc7ef5573becf9bc63a178e`.
Independent `INDEPENDENT-CLOSED-source-four.json` at the heading timestamp
confirms released keys, empty owned cgroup, stopped pinned processes/seat,
absent temporary policy, VT1, unchanged stock library and passive observer
coverage. The first export exceeded a single-browser50MiB non-snapshot bound;
its failed export and explanation are retained. The same immutable native
receipts exported under a proportional two-browser100MiB bound; no input rerun.

First failed transitions are separated rather than treated as one defect:

- Forty cells (selection10 and focus_return10 in each browser) lack admitted
  current word/selection authority. A fresh surrounding-text callback alone
  does not restore a pre-focus lease or prove an absolute beginning. No guard
  has been weakened to admit these cells.
- Three Chrome prefix cells (fields3,7,9) reach exact transformed text/caret,
  no selection and native InputModeUS, while KDE isRU. In iframe field9 the
  bounded bus trace proves setLayoutUS and layoutChangedUS before the failure;
  there is no later RU setLayout/signal within its deadline. Existing private
  IBus Wayland wire output in `logs/plasma.log` records the final Shift-release
  modifier groupRU echoed after the replacement preedit. KWin5.24.6
  InputMethod::modifiers updates its XKB group without layoutChanged. This
  supports a late old-group echo race; exact cross-clock micro-order is not
  established. Publishing fresh compositor modifiers to its existing input
  method grab is a candidate upstream remedy, not yet an implemented fix.
- Chrome Search field2 fails before Double Shift: bounded AT-SPI text and
  caret disagree while the engine owns the full preedit. This is an observation
  discrepancy, not proven physical letter loss; it remains FAIL.
- Firefox URL field3 space_ru_en reaches exact English text but KDE staysRU.
  No US setLayout is observed. Its first lost transition must be established
  separately from the proven Chrome post-switch reversion.

Source comparison pins for the KWin compatibility hypothesis are in remote
`td159-kwin-layout-causal-source-one/SOURCE-PINS.json`. Official5.24.6,
5.27.12,6.0.0 and master all accept input-method group updates; upgrading alone
is not established as a remedy. Next work must preserve one layout initiator,
existing edit ownership, routes and native targets; no sleep/retry loop or
post-hoc text equality may substitute for the missing protocol transition.


### KDE external-layout modifier publication: protocol experiment — 2026-10-19T02:16:58.421122+00:00

Decision `2026-10-19-kde-external-layout-modifier-publication.json` selects a
private upstream compatibility experiment. Official KWin5.24.6 source archive
SHA256`6af31b9fef370f668e1e73dc213a691ed3ef36571ec51ab014f7ead347b60296`
is separate from the host compositor.95 new build dependency packages were
installed on the remote host with zero package upgrades/removals; no Lay or
compositor installation. The test-only patch exercises external group changes
in both directions without a next key and models the delayed-old/new modifier
echo sequence. Runtime patch, source GREEN and native acceptance are pending.
The latest complete browser matrix remains275/45/0 of320. Other112 KDE and
GNOME are not regraded. No edit authority, delivery route or retry is added.


### Exact compositor version and controlled protocol RED — 2026-10-19T02:30:16.858301+00:00

Version correction: the completed private native run pinned executable SHA256
`7f54cfe04778ec0f8e6c68bbed676c1306684afb387b518992457fc7297135b4`.
This is the remote host KWin5.24.7 executable. Earlier5.24.6 labels describe
the separately replaced KWaylandServer dependency; they must not identify the
compositor version. No host KWin upgrade or installation occurred here.

Official KWin5.24.7 source archive SHA256
`9f8318633e1662c6e32ca447c16257c043a40c1777acaa41464640cbda9b0725`
was fetched and built privately. Its keyboard_layout.cpp is byte-identical to
5.24.6 (SHA256`9d34dd7f407b5a65f096dc23651b7582e437ea5bf13185da1ce95388e01b91f2`).
The test-only patch changes no runtime logic. All four controlled protocol rows
fail on the same condition: successful external setLayout produces zero new
keyboard-grab modifier events before the next physical key. Both directions and
both queued-old-echo variants fail; JUnit errors=0. Qt JUnit groups the four data
rows under one testcase; all failure children, tags and exit4 were verified.

Receipt: `/home/e/projects/lay-development-runner/td159-kwin-layout-causal-source-two/RED-RESULT-four-validated.json`,
with exact JUnit hash, test source, baseline library and timestamp. Earlier
missing virtual backend, overlong socket path and an incorrect Qt signal argument
index are preserved as infrastructure/test failures; none count as controlled
RED. The test now checks the signal parameter count before reading it.

The compatibility candidate adds only pending-modifier publication through the
existing InputMethod after a successful external layout switch. Invalid-layout
refusal stays before publication. Source GREEN, the original InputMethod ordering
suite, KeyboardLayout suite, ABI comparison and native qualification are pending.
Runtime edit authority and delivery routes are unchanged. Latest complete native
browser evidence remains275PASS/45FAIL/0BLOCKED of320; other112 KDE and GNOME
remain outside this receipt.


### External-layout protocol GREEN and retained upstream failures — 2026-10-19T02:38:47.369621+00:00

The final isolated protocol test uses QSignalSpy::wait to enter the real event
loop: KWayland Display flushes outgoing events on aboutToBlock. The previous
QTRY-only loop never reached that hook; its missing-delivery result was a test
harness artifact. No runtime delay, retry or extra flush was added.

With the same final regression fixture, original KWin5.24.7 fails all four rows
(exit4, errors0); the five-line runtime compatibility patch passes all four
(exit0). The existing full InputMethod suite passes modifier forwarding and the
new rows, but fails testOpenClose (geometry1 versus625) and testEnableActive
(extra client-added event). The pristine official runtime AND pristine upstream
test reproduce those same two failures. The unpatched runtime with the new
fixture reproduces only testOpenClose, so testEnableActive is order/scheduling
sensitive; this is not an all-green full-suite receipt. In the full-suite run
prior modifier activity also masks the new regression's RED. The controlled
RED/GREEN gate therefore runs the four rows in their own process, preserving
its independent no-key initial condition. No assertion is weakened or skipped.

KeyboardLayout full suite:10 JUnit cases, zero failures/errors. Baseline and
candidate exported ABI names are identical (4207 each); every4156 stock symbol
is retained. Candidate library SHA256`f2bce8fd91d59f720749acb5180affb8a79978ff57c3fa95530dc868919db37c`;
private baseline SHA256`8dae78e69ddeee5284c25af0d467c10150189bcbfaf62bf601ca84fba701cb2a`.
Host stock library SHA256`cdceca37b6f4e52f970ce93e359abdfbc04b1c97066d5a26e98a48a6cf6ab982`.

Receipts (both FAILED full-suite/comparison verdicts are retained):
`/home/e/projects/lay-development-runner/td159-kwin-layout-causal-source-two/EVENTLOOP-RESULT.json`
SHA256`39b3e7a6bfefc5e4a994e1452bbcb97591b5961c835d7f4cbbe174649f4991ae`;
`/home/e/projects/lay-development-runner/td159-kwin-layout-causal-source-two/BASELINE-REGRESSION-COMPARISON.json`
SHA256`6f0f2033f49f2824ffe734373b6e4b07febbfa1a061e87715927d6bab7857958`.
The strict equality comparison failed because the baseline fixture did not
reproduce the second intermittent upstream failure. This result is not relabeled
PASS. Native comparison is diagnostic only; release qualification stays open.

Next: original prefix cases under the private baseline and candidate KWin5.24.7
libraries, mounted only onto the actual5.24.7 library path in the private native
namespace. Verify mapped device/inode/hash before any key, and verify the host
library unchanged at closure. The Lay binaries, input delivery routes, native
case bodies, targets, deadlines and guards stay identical. No user installation.
Latest complete browser matrix remains275PASS/45FAIL/0BLOCKED of320; other112 KDE
cells and GNOME remain outside this receipt.

### Private KWin native comparison — 2026-10-19T02:58:39.146823+00:00

The private baseline prefix subset completed Firefox10PASS/0FAIL and
Chrome8PASS/2FAIL. The candidate completed Firefox10PASS/0FAIL and
Chrome9PASS/1FAIL. The baseline Chrome post-conversion text/group mismatch was
absent with the candidate. Chrome Search field2 still failed before the first
Double Shift: AT-SPI returned11 characters with caret12 for the12-character
seed. No expected surface, case function, deadline or cadence changed.

The complete fixed KDE browser actor320 then completed under the same private
candidate library: Firefox138PASS/22FAIL/0BLOCKED; Chrome139PASS/21FAIL/0BLOCKED;
combined277PASS/43FAIL/0BLOCKED. Relative to the preceding275/45/0 receipt, all
three Chrome prefix layout mismatches passed; Firefox gained one additional
Space layout failure. This is a net two-cell improvement, not proof that the
intermittent Firefox race regressed or that the whole suite is accepted.

Remaining groups:40 selection/focus-return failures, one Chrome Search prefix
pre-gesture observation discrepancy, and two Firefox Space layout mismatches
(iframe9 RU-to-US, Search2 US-to-RU). Both Firefox failures reached exact text
and caret while retaining the old KDE group. The preceding URL3 Space failure
trace establishes an already applied correction followed by Reset, which
revoked the pending background layout intent before setLayout. The new failures
must be checked against that first failed transition; text equality cannot
restore edit authority and no Reset guard has been relaxed.

Scope clarification from source comparison: this supplemental KDE actor's
selection case invokes Double Shift while the entire committed text remains
selected. Its focus-return case invokes Double Shift immediately after returning
to the field. The canonical browser_ime_standard.md instead checks typing over
the selection, and a new-letter/Backspace sequence after focus return. These are
different actions even where their labels match. Preserve all fixed KDE actor
FAILs and their case identities; do not substitute canonical controls to improve
this count or compare it directly with the canonical GNOME432 total. Other112
KDE actor cells and GNOME have not been regraded. No result here certifies a
release or the user's installed runtime.

Successful evdev-write-return to observed exact native target, milliseconds:
Firefox Double Shift N200 median164.84/p95180.29/max207.30; Chrome N198
median167.98/p95177.80/max202.36. Space Firefox N48 median29.46/p9532.60;
Chrome N50 median30.12/p9532.94. Tab N10 each: Firefox median29.20/p9531.45;
Chrome median30.11/p9532.34. Failed targets remain FAIL with no successful
latency sample; kernel-release latency is not measured.

Private complete native receipts, under remote
`/home/e/projects/lay-development-runner/kbdg/`:
`native-windows-6za8pzbk/RESULT.json` SHA256
`a17cc39e09972a5bc0be727d4c0937f2fa44d96b1abe6647c7ec3fda8eb89ea2`;
`native-windows-45e7d9p8/RESULT.json` SHA256
`e5da6b15ebb2e0c56d9fea795c64fe4cfc2878ca471ed4df16f224956db528f1`.
Export `../kbdg-kwin-layout-candidate-full-native-evidence-source-one.tar.gz`
SHA256`7287abc918cd05267a62f35e2c1a8a538fca3cc24ab92cacca76a6ddfde7a2d7`,
29,420,627 bytes,3291 files,1280 raw snapshots. Independent
`INDEPENDENT-CLOSED-source-three.json` confirms empty cgroup, no held keys,
owned processes/seat stopped, VT1 restored, temporary policy absent, host
KWayland/KWin libraries unchanged, and the exact privately mapped KWin candidate.
Local evidence is in development cache
`kwin-layout-native-candidate-full-source-one/evidence/`.

Native remains diagnostic-only. The strict upstream InputMethod comparison
stays FAILED with its previously recorded baseline/order-sensitive failures;
candidate packaging and release qualification remain open. No Lay binary,
delivery route, edit authority or user installation changed in this experiment.

### Committed layout versus word Reset — 2026-10-19 source RED

The two Firefox Space failures in the complete actor320 receipt have the same
first failed transition as the earlier URL3 failure: authorized committed
replacement and forward-layout request, authenticated Reset in the same
activation, then `ibus_layout_sync status=in_flight_context_conflict`. Exact
client text does not prove layout completion. The strict active-word revocation
check cancels the already issued forward-mode request before KDE setLayout.

The real-adapter regression
`td159_committed_layout_completes_across_same_activation_word_reset` reproduced
one semantic failure on the unchanged runtime. Receipt in development cache
`committed-layout-reset-red-source-one/RESULT.json`, event
2026-10-19T03:05:22.950980Z, focused log SHA256
`406841efb8864e48def8125d0d6e818b577039a5b872809aab5d3e6365f79cd0`;
27.59 seconds including the guarded source check. This is source RED, not
physical-client acceptance or a build/install result.

Decision `2026-10-19-committed-layout-word-reset-scope.json` scopes the minimal
change to the existing Background request for already dispatched committed
text. Its existing reducer retains the last non-Reset revocation floor;
connection, owner, settled activation and latest request generation must still
match. Direct/open-preedit layout requests retain strict revocation checks.
Reset still retires the word token, suffix and exact-manual text authority.
No delivery route, owner, counter, timer, retry or client-specific condition is
added. This changes forward-mode authority and awaits source GREEN and fresh
native evidence. Tests also check repeated Reset, Reset during dispatch,
non-Reset revocation and newer-intent supersession.

A separate six-case read-only range diagnostic completed and independently
closed at2026-10-19T03:02:38.024366Z. Firefox URL3 Space passed; four supplemental
selection/focus-return cases and Chrome Search2 prefix failed. Chrome reported
character_count11/caret12 and the same missing final character through full,
EOF and per-character AT-SPI reads in two observations. This does not establish
a bounded-range observer bug; actual DOM/rendered text remains unverified.
The first preparation attempt failed its AST preservation source gate before
any keys. The corrected gate strips only the exact added post-verdict observer
AST, checks all original actor code, and detects a controlled observer mutation.
No original actions, targets, cadence or verdict were changed.

Diagnostic evidence in development cache
`kwin-layout-native-candidate-observation-source-two/evidence/`; remote phase
`/home/e/projects/lay-development-runner/kbdi/`, export
`kbdi-kwin-layout-candidate-observation-native-evidence-source-one.tar.gz`
SHA256`0a40d3cf5a1e922e3b621f33996999e45aa6f5938bb1481310fde35d5678f4fd`.
The six-case result does not replace the latest complete277PASS/43FAIL/0BLOCKED
of320. Other112 KDE cells, GNOME, the new Reset change and user installation
remain unverified by this diagnostic.

### Committed layout word-Reset source GREEN — 2026-10-19

The remote hermetic gate completed at2026-10-19T03:18:23.841675Z:855
functional/package checks passed,0 failed, including322 checks executed in
isolated processes;3 separately classified performance checks passed. Official
discovery found858 total identities. Receipt in development cache
`committed-layout-reset-hermetic-source-one/RESULT.json`, frozen source request
SHA256`593e33a4830912dac2ed13950ec4d00ca24b3f20247daf288b2452c8d5b35ca1`,
discovery SHA256`73accbfa7e9a805f5160c62220fed88d53a04a2d13bc886c9c310efbbcf810b2`.
The gate uses the official discovery classification and hermetic executor with
an explicit pinned source closure in place of checkout discovery. It took
120.49 seconds including discovery. This establishes source behavior only.

The causal regression now proves exactly one committed forward-layout switch
with decoder promotion after actual authenticated adapter Reset, while the
old word token, strict layout token and old text authority remain invalid.
It also exercises a repeated Reset during dispatch. Reducer tests retain strict
Direct validation and cancellation across non-Reset revocation. No delivery
route changed; the narrowly extended Background authority is recorded in the
explicit decision above and is not client acceptance or learning authority.

Earlier attempts are retained: `committed-layout-reset-green-source-one` failed
to compile because the new test called an observer method on the adapter;
the test was corrected to exercise the existing reducer effect after the real
authenticated wire Reset. Source-two and source-three each passed the focused
semantic regression, but their checkout-dependent focused CLI could not run
on an archive without Git HEAD. No Git history was fabricated and those
receipts remain FAILED. The final hermetic gate supplies the required broader
source verification without relabelling those attempts.

Remote architecture refresh, private binary build and the unchanged320 native
browser scenarios are pending. The last complete native result remains
277PASS/43FAIL/0BLOCKED. Other112 KDE cells, GNOME and user installation are
not covered by this source gate. The strict upstream KWin failure receipts
remain separate and unchanged.

### Committed layout word-Reset complete browser proof — 2026-10-19

Remote graph/canon verification passed at2026-10-19T03:22:13.148317Z;
`committed-layout-reset-architecture-source-one/RESULT.json` SHA256
`8c53122d9b206912208808ccf3a040fa169d552337db66f199954e32f9b13092`.
The final private release build passed its embedded architecture receipt test
at2026-10-19T03:25:51.684727Z. IME SHA256
`89d12c6beb61094ae6dea40ca02d735997f2bd2408ffd56f217c55cec612a437`,
8,456,768 bytes; final build receipt in development cache
`committed-layout-reset-final-build-source-one/RESULT.json`. The build reuses
the exact855 correctness checks and3 separate performance checks documented
above; generated graph receipt validation is an additional build check.

All320 original browser identities then completed with unchanged actions,
targets, deadlines, cadence and verdicts. Firefox140PASS/20FAIL/0BLOCKED,
Chrome139PASS/21FAIL/0BLOCKED: combined279PASS/41FAIL/0BLOCKED. Comparing exact
identities with the preceding277/43 receipt gives277PASS->PASS,
41FAIL->FAIL and2FAIL->PASS; no lost PASS. The two recovered identities are
Firefox iframe9 Space RU->EN and Search2 Space EN->RU. A fixture identity is
evidence only and is not a runtime condition. All40 selection/focus-return
failures and Chrome Search2 prefix remain failures.

Remote phase `/home/e/projects/lay-development-runner/kbdj/` independently
closed at2026-10-19T03:34:22.102927+00:00. Empty cgroup, released keys, stopped
owned processes and seat, restored VT1, absent temporary policy and unchanged
host compositor libraries were verified. Export
`kbdj-committed-layout-reset-native-evidence-source-one.tar.gz` SHA256
`449cd8ada396ad9a83f6bc5d50c2fbc5bb7105a91f53267e95b5a724ffae6e05`,
29,272,279 bytes,3291 files,1280 original raw endpoint snapshots. Local verified
evidence is in development cache
`committed-layout-reset-native-candidate-full-source-one/evidence/`.
Original native receipts are Firefox `native-windows-vkc4hysm/RESULT.json`
SHA256`815d62cd321dbf369dc48439a34e897612599482008a4d4f25b1455203b00d8e`
and Chrome `native-windows-wj8c_ajr/RESULT.json`
SHA256`70fca4877313474025b3d0739192dc9e655c958a3ec6c5e055d331db764cedc1`.

Successful native-operation timings use statistics.median and nearest-rank
p95, from evdev write return to exact native AT-SPI target observation;
kernel release latency is not measured. Firefox Double Shift n200:
median163.012ms/p95179.000ms/max184.488ms; Chrome n198:
167.503/177.418/183.396ms. Space Firefox n50:29.060/32.120/32.773ms;
Chrome n50:30.196/32.369/32.569ms. Tab Firefox n10:
30.177/30.860/30.860ms; Chrome n10:29.296/34.318/34.318ms.

The first observed refusal for the40 supplemental selection/focus-return
cases is ManualToggleV3 context authority, before edit execution. Full raw
SSI endpoint snapshots do contain surrounding observations; absence of a
filtered context-admission trace line is not absence of SSI. In field10's
selection diagnostic, actual AT-SPI selection covers0..7 but the latest SSI
reports cursor0/anchor0. Focus return instead supplies a fresh nonempty SSI
after a new focus activation, while the previous word ownership remains
revoked. These are distinct capabilities/authority boundaries. Neither
loosening the selection guard nor reviving old word rights from matching text
is an accepted repair. No IBus focus-stealing cause is established by these
refusals.

This is a complete supplemental KDE browser component, not the canonical
GNOME432 or full KDE432. The same pinned IME/compositor components are now
being checked on the other112 original GTK/Qt/KWrite/Konsole identities.
Their actions and case functions are preserved, with the selector and exact
job identities changed to that original component. Private upstream KWin
full-suite failures remain FAILED and release qualification is still open.
The narrow Background layout authority change is explicit; delivery routes,
text-edit admission, Direct validation and the user's installation are unchanged.


### Complete original KDE432 after committed-layout Reset correction — 2026-10-19

At `2026-10-19T03:40:17.268585+00:00`, the other112 original identities completed: GTK26PASS/6FAIL, Qt39PASS/9FAIL, KWrite13PASS/2FAIL/1BLOCKED, Konsole3PASS/10FAIL/3BLOCKED. Together with the disjoint complete browser320 this is **360PASS/68FAIL/4BLOCKED of432**, private diagnostic KDE only. All432 `(app,field,plan)` identities are unique; the browser receipt remains Firefox140/20/0 and Chrome139/21/0 of160 each. This does not replace the canonical GNOME browser432 matrix.

Four-app export: `/home/e/projects/lay-development-runner/kbdk-committed-layout-reset-native-evidence-source-one.tar.gz`, SHA256 `3e54238fbe8c8f9611de7db8be7aa82c9f5862d836fa320b09441396046b3fd0`; independently checked closure `INDEPENDENT-CLOSED-source-three.json`, SHA256 `dcf44a909584396f9b53764bf768827d37fd65b96e02842098e66335dd3612f8`. Summary and exact receipt hashes: development cache `committed-layout-reset-native-four-apps-source-one/QUALIFICATION-SUMMARY.json`. The pinned private IME remains `89d12c6beb61094ae6dea40ca02d735997f2bd2408ffd56f217c55cec612a437`; host libraries and user installation remain unchanged. Runtime authority was changed only as already recorded in the committed-layout word-Reset decision.

Residual classes are25 focus-return,26 selection,6 prefix,10 Konsole conversion/Space and1 KWrite moved-boundary failure. The4 BLOCKED cells require a native selection/second field/completion panel absent from those fixtures. Five prefix failures show the expected display with an active preedit but do not satisfy the unchanged committed-target assertion; they remain FAIL. Do not relabel these observations as acceptance.

Next source experiment is `2026-10-19-fresh-focus-exact-manual-source`: acquire a current authenticated snapshot only on explicit manual intent after untouched fresh activation, while preserving UnknownStart and zero observed suffix. Current selection evidence may report a collapsed surrounding cursor despite a native selection, so general acquisition from arbitrary SSI is rejected. No implementation/native result yet. The existing reducer and exact-tail executor own the proposed proof; no daemon fallback, automatic word grant or delivery-route change. Private KWin upstream full-suite FAIL remains retained.


### Fresh-focus explicit manual source — controlled RED and consequence check,2026-10-19

TD159 remains IN_PROGRESS. Controlled source RED at2026-10-19T03:47:38.678080+00:00:
`fresh-focus-manual-red-source-two/RESULT.json`, zeroPASS/one semanticFAIL,
FOCUSED SHA5a1b65a167a421d0c299d702076eda14a19287b1f120adfffd5c6d864d3e585f.
The earlier `fresh-focus-manual-red-source-one` failed in fixture setup before
semantic assertion and remains recorded separately. Both are remote source
receipts under `/home/ubu/.cache/lay/development/browser-final-fixes-20261018/`.
Initial implementation gate `fresh-focus-manual-contract-green-source-two` at
2026-10-19T03:54:44.109339+00:00 isFAIL:3PASS/3FAIL. Negative field guards and
reducer phase/receipt guards pass; all three positive bridge scenarios still
return NotHandled. Do not promote this patch or infer any recovered native cell.

The first failure to repair is the source-only transition from fresh authenticated
UnknownStart plus current SSI to an explicit manual exact-tail lease. A matching
SSI does not upgrade KnownStart or observed suffix. Decision
`2026-10-19-fresh-focus-exact-manual-source.json` compares the selected existing
receipt/executor design with KnownStart restoration, daemon fallback and a
require-another-character workaround; only the first keeps the intended action
and existing delivery ownership.

Consequence check: candidate/lattice retention and lexical ranking are unchanged;
manual admission must not create automatic correction, learning or feedback
rights. The current package/delta identity does not confer this manual authority.
Keep UnknownStart/zero observed suffix and exact existing replay suppression.
Only explicit manual intent allocates a tail, bounded by the existing160-character
limit; ordinary Space and typing hot paths gain no RPC, timer, wait or model load.
The existing700ms handoff lifetime stays unchanged, and speed still needs native
measurement. Current owner, connection, activation, revocation, epoch and SSI
revision must all agree; possible mutation spends eligibility independently of
SSI because native selections can be misreported as collapsed. A new receipt,
Reset after acquisition, focus loss or expiry cannot revive the consumed phase.
Sensitive, atomic, composition, selected/interior and quarantined input stays
closed. The old reducer, bridge, exact-tail executor and verifier retain ownership;
no backend, fallback, historical word cache or alternate source of truth is added.
Worst plausible regression is a stale or selected source gaining manual authority;
negative source tests and the unchanged selection/focus native plans are therefore
required before promotion. Failure retains the previous private build; rollback
removes only this proposed eligibility/receipt acquisition, preserving preceding
proven layout/panel fixes. Future package/model changes do not relax these gates.
Maintenance cost is one reducer-owned input phase and bounded receipt queries;
there is no second word owner or persistent data migration.

Latest complete original KDE432 remains360PASS/68FAIL/4BLOCKED, browser component
279PASS/41FAIL/0BLOCKED of320. No GNOME regrade, user installation, publication,
delivery-route change or upstream KWin full-suite qualification follows here.


2026-10-19T04:00:04.260541+00:00 — Corrected controlled original-source RED:
`fresh-focus-manual-valid-red-source-one/RESULT.json`,0PASS/1semanticFAIL,
FOCUSED SHAcf7ab82c648341724bc73b889fec1b970e938fdc0f0e0915375a0032b60a4d4c.
Earlier initial positive fixtures paired Cyrillic text with an authenticated US
source profile; the existing layout projection intentionally produced no change.
Those failures are retained as fixture-data failures, not additional runtime
defects. The corrected English fixture retains the same explicit delegation
expectation, real adapter/bridge and original production code. Negative fixtures
now assert valid eligibility before introducing each refusal, avoiding vacuous
PASS. Candidate `fresh-focus-manual-contract-green-source-four/RESULT.json` at
2026-10-19T03:58:39.434931+00:00 has6PASS/0FAIL in27.670s; source handoff,
manual-only authority, source bounds, control-edit, sensitive/atomic/selection/
interior/SSI guards, spent phase, receipt revision/expiry/connection and Reset
checks pass. No native cells are regraded; full source qualification follows.


2026-10-19T04:03:15.039132+00:00 — Fresh-focus exact manual source, full source qualification:
`fresh-focus-manual-hermetic-source-one/RESULT.json` is SOURCE_GREEN_NATIVE_PENDING.
861 correctness/package PASS,0FAIL,864discovered (326 isolated functional),
plus3 separately executed performance PASS; total120.788s.
Discovery SHAbdb4d5b085c01c5eeaa8508279b1cbd506619c9e8a97f2b7600e3432a16320bd.
Exact request, frozen overlay, individual test outcomes and logs remain in that
receipt directory. The corrected original-source RED and6 focused GREEN remain
the causal source witness; architecture refresh, private build and native
focus-return/selection qualification follow. This source result regrades no
physical client cell and changes no installed runtime or delivery route.


2026-10-19T04:08:39.887844+00:00 — Fresh-focus source candidate private build:
Architecture refresh PASS at2026-10-19T04:04:44.656794+00:00, exact generated
receipt test PASS, release IME built in211.286s.
Private IME SHA4dcae642e0e82ba72e5ef8ec82b94477327d765163ea12169104f2ed0765038d,8,460,864bytes.
`fresh-focus-manual-final-build-source-one/RESULT.json` pins861correctness/package
and3separateperformancePASS, frozen source, graph and build. Existing daemon,
models, actor bodies, private KWin/panel and output delivery contracts retained.
Original40focus-return/selection native identities are prepared for a causal
subset run before any full matrix regrade; host installation remainsfalse.


2026-10-19T04:11:01.695309+00:00 — Fresh-focus native40 causal subset completed
and independently closed. Private IME SHA4dcae642e0e82ba72e5ef8ec82b94477327d765163ea12169104f2ed0765038d;
unchanged original case bodies, targets, cadence and deadlines. Firefox
focus-return10PASS/0FAIL, selection0PASS/10FAIL; Chrome focus-return0PASS/10FAIL,
selection0PASS/10FAIL. Combined10PASS/30FAIL/0BLOCKED of40. All ten Firefox
focus failures from the previous full receipt recover in this subset. This does
not regrade the320 or432 full receipt: the new bytes have not run those matrices.

Evidence cache `fresh-focus-manual-native-candidate-focus-selection-source-one/evidence/`:
Firefox `native-windows-xtu7pve6/RESULT.json`, SHA43d50babe5fff079fac3d24b7300d1e61837f76c47ecf4d737504105f3081a1c;
Chrome `native-windows-mkkaj3gb/RESULT.json`, SHA49fef4e5d42af07d7d1fe5e567a5efb79def901e4abb5b9a60276e3014480021.
Successful Firefox focus-return latency:10 measurements, median144.824ms,
p95175.679ms, maximum175.679ms, existing evdev-write-return to exact native
AT-SPI target contract. Chrome has no successful focus-return timing sample.
Export `/home/e/projects/lay-development-runner/kbdm-fresh-focus-manual-native-focus-selection-evidence-source-one.tar.gz`,
SHA13415e8119663d6456e065908b2454fbe69cd2c9c962f6858586f60608549b92,
2,970,906bytes,491files including160raw snapshots. Independent closure confirms
VT1, empty owned cgroup and all pinned descendants gone; closure errornull.

First Chrome refusal is before manual-source acquisition: active source-free
owner69, UnknownStart, zero observed suffix, zero local buffers and absent
current SSI at the physical Shift callbacks. Bridge token is live; RPC refuses
context_authority. Retained endpoint shows a six-character actual SSI before
focus loss and subsequent source-free activations without a current snapshot.
Trace interval continuity/producer completeness remain UNKNOWN; missing events
alone do not prove which producer omitted a callback. Upstream IBus1.5.34-rc2
engineproxy.c caches text/cursor/anchor and suppresses identical callbacks even
across focus changes, while Lay correctly revokes the old proof. This is a
specific source-supported hypothesis, awaiting passive private-bus evidence.
The existing ActiveSurroundingText property already requests updates on focus;
do not assume adding another request solves cache suppression. No new runtime
patch, authority relaxation, route change or installation follows from this
diagnosis. Selection remains a separate uncorrected mechanism.

2026-10-19T04:20:48.134972+00:00 — Fresh-focus passive private-bus20 diagnostic:
Firefox10PASS/0FAIL, Chrome0PASS/10FAIL, unchanged focus-return identities and
IME4dcae642. This corroborates the native40 subset and does not regrade a full
matrix. Evidence cache `fresh-focus-manual-native-focus-bus-diagnostic-source-one/evidence/`;
export `/home/e/projects/lay-development-runner/kbdn-fresh-focus-manual-native-focus-bus-diagnostic-evidence-source-one.tar.gz`,
SHAef67afb761a7202409b6f6ecd7ebc979f253b12bc58093499c1175106fe97fc5,
1,519,568bytes,291files including80raw snapshots. Independent CLOSED receipt
SHA2c4d0f6ff9eb0a517c6da8d444069be6de2a24f5e90733ab4430045ea3e2a603:
VT1, empty owned cgroup, pinned processes gone, no held keys or temporary policy.
No installation, delivery-route or additional authority change.

Positive bus witness in the final Chrome case: EngineRU30 received six-character
collapsed SSI at1792383638.988314. After FocusOut32, fake1, FocusIn33, a current
InputContext33 SetSurroundingText arrived at1792383639.051923 with the identical
text/cursor/anchor. Engine proxy source suppresses this identical tuple although
its focused object path changed; its duplicate cache survives focus changes.
Next FocusOut33/fake1/FocusIn34 and RequireSurroundingText occur by
1792383639.086074, but no current InputContext34 SSI is recorded before manual
refusal. That second absence does not establish producer completeness. The
Wayland client source caches actual callbacks but forwards only when asynchronous
context creation has finished AND needs_surrounding_text is already true; it
has no require-surrounding-text callback to deliver an earlier current snapshot.
A metadata-only private client diagnostic is prepared to establish that ordering
before any behavioral upstream patch. Never replay the previous field's cached
SSI or relax Lay's focus revocation in response to this dependency defect.

2026-10-19T04:29:05.407362+00:00 — Metadata-only Wayland-client diagnostic20
independently CLOSED, Firefox10PASS and Chrome10FAIL. Export
`kbdp-fresh-focus-manual-native-client-trace-evidence-source-two.tar.gz`,
SHA2e76b6453db2846e3c78d038482ed950d2f66b2fc9b16a4de6ced450f52b4e20,
1,550,302bytes,292files including80raw snapshots. Independent closure
SHAe2a08f17638b71c8d2f2b35f5ec322bbbf163088f27941bff63ea386e246e341.
Metadata trace in `logs/plasma.log` positively records current12byte callbacks
with contextnone at1792384086785481–1792384086785527; context7 is created at
1792384086787082 with cached1/bytes12, and Require is received at
1792384086788957 with the same current cached observation. Existing handler set
has no Require callback to forward it. Other contexts receive a later repeated
Wayland callback; the distinct cross-focus engine-proxy duplicate suppression
still loses that fresh tuple. This is two measured transitions, not an inferred
focus theft or a reason to reuse Lay's revoked receipt. Baseline native20 remains
10/10/0 and full counts stay unchanged.

The first metadata-only attempt kbdo stopped before READY or any actor creation
because the existing source-panel pin differed from the staged trace bytes.
Preserved CLOSED is independently checked at2026-10-19T04:27:23.333708+00:00:
no actor,0rows, empty cgroup, inactive owned seat, VT1, no temporary policy. The
second attempt pins and launches the staged private panel; actor bodies and
assertions remain unchanged. First export attempt retained the older hardcoded
panel hash and refused; export-source-two.py pins the actually tested trace hash.
These preparation/export failures do not count as native FAIL or PASS.

2026-10-19 — Dependency experiment `2026-10-19-ibus-focus-surrounding-observation`
now carries `scripts/compat/ibus-fresh-focus-surrounding.patch`. No Lay runtime
source or authority change in this experiment. Remote extracted-actual-function
RED is4PASS/4FAIL; proxy-only is6PASS/2FAIL (pre-readiness cases4,5 retained),
Wayland-only is6PASS/2FAIL (focus cases1,3 retained), combined8PASS/0FAIL. These
are decision contracts with recording transport, not upstream integration or
client acceptance. Unicode cursor/anchor, absent current tuple, foreign context
and redundant same-context focus are covered. First RED fixture attempt lacked
stdint.h and is retained as a compilation failure, not semantic RED.
Private IBus build `ibus-fresh-focus-candidate-build-source-two/BUILD.json`
changes only daemon and panel binaries; new panel SHA7fd3fba705337377cf7aa3dbea15bf2fc593bf273fcef96cae69428679430268,
daemon SHAfb548baa5299f28e430cbd1df6d2e56982bac98ef2362c3f17c837a6588378a8.
The first build copied the libtool launcher instead of the ELF daemon and
refused its ELF guard; source-two copies bus/.libs/ibus-daemon. Original sources,
private baseline and host installation preserved. Native comparison pending.


2026-10-19T04:38:00.089950+00:00 — Corrected IBus dependency object lifetime is
source GREEN, native still pending. The first combined build passed8 extracted
decision contracts but its Require handler passed the borrowed cached IBusText
to an API that may unref its input on an identical tuple. Native phasekbdq
therefore ABORTED after Firefox10PASS, during the first Chrome case with zero
terminal Chrome rows; panel GObject errors and SIGSEGV are retained. This is a
regression introduced by that private patch, not an accepted client result.
Independent CLOSED_ABORTED_NATIVE_PARTIAL_ONLY at04:35:25.032208Z verifies VT1,
empty owned cgroup, stopped owned seat and no held keys from both actor reports;
receipt SHA5fc5b8d44df37fbb5e7cba76dacd28c44e0fc1e8081fbeb762dceaf55266ae38.
Remote evidence remains in `kbdq/`; no input rerun is counted as its completion.

Actual upstream InputContext setter + real IBusText/GObject regression gives a
semantic RED against the borrowed-object candidate at04:37:36.040141Z:
`ibus-fresh-focus-lifetime-red-source-two/RESULT.json`,
SHA731cb4fa1442b8d3ebf83e8033cc88fa28888390b0e052a5a6e84fad8235f5dc,
returncode-6 with failed IBUS_IS_TEXT on the cached object. The earlier attempt
without the private ibus pkg-config path is only environment failure, not RED.
Corrected Require creates a fresh floating IBusText for the consuming API;
ordinary immediate callback forwarding remains unchanged. Repeated Require32,
callback replacement, Unicode cursor/anchor and empty-cache refusal pass with
real objects, actual setter and recording D-Bus transport:
`ibus-fresh-focus-candidate-build-source-three/real-object-lifetime/RESULT.json`,
SHAab097a4384b26f8fc5590ad0269d24153a80c1f7e785c571ec5567f8de04737e.
This check covers ownership omitted by the earlier recording-only fixtures;
it is not a native client or upstream integration result.

Corrected immutable private build `ibus-fresh-focus-candidate-build-source-three/BUILD.json`
retains combined8PASS; proxy-only and Wayland-only each6PASS/2FAIL with their
expected distinct loss cases. Patch SHAe23bc0d3f4538fa3ec1b8e1cbfc9f6beaf9fbbb53195f864fcc7a05c5b89186b;
private panel46a335becaa4ed35fc217adebd2fd206ea5dda4af8abd1a06f60136dac7e3a23,
daemona6182ab3078f0c09aab3ecfcb8b913811a54b01e5a478a3b343f3a376e5a3fc7.
Only those two private prefix ELF files differ from the prior clear-panel
prefix. No host installation, Lay authority change or delivery-route change.
Original focus20 native cases are running as phasekbdr on unchanged IME4dcae642;
last complete browser320279/41/0 and original KDE432360/68/4 remain unchanged.


2026-10-19T04:39:52.278710+00:00 — Corrected private IBus native20 COMPLETE:
Firefox10PASS/0FAIL, Chrome0PASS/10FAIL, no crash. Phasekbdr on IME4dcae642,
private panel46a335be/daemona6182ab3. Evidence local
`fresh-focus-surrounding-native-candidate-source-two/evidence`;
Firefox RESULT36c39fc75ef0d893f92f5dee86e56073f52bc0af0470f671b5dd33bca1645a78,
Chrome RESULTf35af0d8c6e77651d301906a73b503e8db9e0f43ae8b24aa5aab68b28c74901c.
Independent closure02bb19e20713d739e0a536c09ab81ad51a6f6227e7c8af5b19f90e68cf502145
confirms empty owned cgroup, processes gone, VT1, seat stopped, no held keys;
export5a7ff25c0be77ad8cda839cc205b51bf8822457d7e6a693602b5425a3788cf50,
292files/80raw snapshots. Full browser320279/41/0 and original KDE432360/68/4
are retained; subset20 is diagnostic only.

Positive Chrome trace now contains actual6-char SSI and ManualToggleV3 status3,
then same-context native factory layout handoff. Target SSI arrives before the
activation marker publishes/installs the transfer. The inherited exact receipt
is intentionally limited to absent target callbacks, but installation does not
bind matching actual target evidence in its existing reducer. V4 therefore
becomes passive:unknown-context; daemon retention-policy refusal follows from
its passive permission=false. Purpose0/hints64 stay unchanged, so this is not
proof of a privacy-policy change. Decision
`2026-10-19-exact-manual-target-observation.json` scopes a RED-first repair of
this existing activation owner. No lease, verifier or delivery-route weakening.


2026-10-19T04:52:58.937879+00:00 — Exact actual target observation transfer is
source GREEN. Same existing activation owner binds the source grant into the
existing reducer only when actual target SSI has the same complete external
prefix, owned tail/epoch, collapsed end caret, supported non-sensitive field and
unexpired original source deadline. UnknownStart/suffix0, later callback
revocation, privacy permissions, verifier and output routes are preserved.
Focused source-four3PASS includes empty/nonempty/Unicode prefixes, private
content, V4 and replay suppression. Different prefix/tail, selection, interior
caret, missing actual tuple and pre-FocusIn retired tuple refuse; a subsequent
identical callback revokes the new reducer receipt. No fixture text is a runtime
condition. Initial GREEN fixture incorrectly requested acceptance of a tuple
before FocusIn (which intentionally clears it), and initially paired a Cyrillic
source word with a US source layout; those fixture failures are retained.
The native-order later-callback regression was a valid semantic RED before the
runtime change. The runtime patch was unchanged while fixture orders were fixed.
Official hermetic gate selected/executed864/864 correctness/package tests,
329 process isolated, allPASS; separate performance3/3PASS. Receipt
`fresh-focus-target-hermetic-source-one/RESULT.json`,
SHA03f20e2286e25cd1f44c669733cd209fd12527d976ff7fa772054c65f4f148b2, at04:52:58.937879Z.
Source facts only; graph/final private build/native pending. Last complete
browser320279/41/0 and original KDE432360/68/4 retained; user host unchanged.

2026-10-19T04:54:20.031584+00:00 — Corrected source-four fixtures were also
rerun against the unchanged original two runtime files. Controlled RED is
1PASS/2FAIL, confirming that matched current target evidence loses authority
and its later-callback assertion cannot acquire a source on the original code.
Receipt `fresh-focus-target-valid-contract-red-source-one/RESULT.json`,
SHA1001d745e7772c3d8fb8dee69eac06e98ad155408158b5e6182f335e100dbeaa.
This closes the fixture-order baseline gap without changing runtime or tests.
Graph/private-build/native gates remain pending; no full-count update.

2026-10-19T05:01:09.831630+00:00 — Private IME built after architecture graph
PASS and the scoped embedded-receipt test PASS. Binary SHA
f800b64b1e088f41f5917f306a50355fc8c7d0c5ae53e7b3598d1e5a09cb7e89,
8,461,376bytes; receipt `fresh-focus-target-final-build-source-one/RESULT.json`
SHA4e3e2dad53787d649d5624ef746075ae187ae1dadb380e510224b1be50a511b6.
The tested864 correctness/package and3 performance results are reused only for
identical Rust sources; only generated graph JSON changed afterward. Private
native focus20 phasekbds uses the corrected IBus panel46a335be/daemona6182ab3.
Its inherited PLAN summary still names the earlier focus/selection scope and
unchanged panel; this is a retained metadata error. Actual ORIGINAL selector,
required20, private build manifests and ELF pins establish focus-only scope and
changed dependencies. No test or assertion is altered; full-run preparation
will correct these labels before freezing its manifest. No host installation.

2026-10-19T05:02:25.372710+00:00 — Private IMEf800b64b with corrected private
IBus completes original focus20: Firefox10PASS/0FAIL, Chrome9PASS/1FAIL. The
Chrome field2 failure occurs while seeding, before any manual gesture: expected
visible `привет`, observed committed `приве` with caret6; retain asFAIL, no causal
claim from a truncated/replaced trace. Nineteen manual focus-return operations
reach native targets. Firefox successful N10 median166.969ms/p95206.714ms;
Chrome N9 median165.968ms/p95174.246ms. This is diagnostic subset evidence.
Receipts `fresh-focus-target-native-candidate-source-one/evidence/`:
Firefox `native-windows-hlu2n35q/RESULT.json`
SHAf41a1f0e76a080053f2a72644c151108a54af1d2f2d756a00d744edd3f28cfc2;
Chrome `native-windows-p6zj0wn9/RESULT.json`
SHA3ffbaa0e544da5d707c2d6f1fad1a70d954d813f1349e72206ea7c17a4a0dff0.
Independent CLOSED SHA243c15e586a5c5ea798992df0d4f7f3f4431359fbe17271e22956ee8ef0ec84a
confirms no held keys, empty owned cgroup, processes gone, VT1, seat stopped,
policy absent and host libraries unchanged. Export
`kbds-fresh-focus-target-native-candidate-evidence-source-one.tar.gz`
SHA0cd060e229ce3bd84021db6a71ffc11bd78d00539cbd612168e6dbcd6cd158e7,
292files/80raw snapshots. Full320 phasekbdt freezes all original identities and
same exact bytes to check aggregate effects and the independent seed failure;
no selection or cadence change. Until complete, browser279/41/0 and KDE432
360/68/4 remain the last full results; GNOME and user install not regraded.

### 2026-10-19 — exact target native full run aborted; owned composition Space gap

IME `f800b64b1e088f41f5917f306a50355fc8c7d0c5ae53e7b3598d1e5a09cb7e89`
and corrected private IBus were tested against the original320 browser identities
in phase `kbdt`. Firefox completed144PASS/16FAIL; Chrome stopped at118PASS/9FAIL,
oneRUNNING row and32not-started cases. This is an ABORTED full run: the passive
panel log reached its finite14MiB limit (`passive panel method log overflow;
evidence incomplete`), not a case timeout. No aggregate full result is promoted.
Next full monitoring keeps a finite64MiB limit without changing input cadence,
case identities or assertions. Last COMPLETE browsers remain279PASS/41FAIL/0BLOCKED,
KDE432360/68/4. Native clients with new bytes are not qualified.

Frozen export: `/home/ubu/.cache/lay/development/browser-final-fixes-20261018/
fresh-focus-target-native-candidate-full-source-one/EVIDENCE.json`; archive
SHA256 `83b3dd19dff31a61a86569d4ea6ad4ece3ceacd626517a3757206613fb946d0f`,
25,243,444bytes,2972files including1152raw snapshots. Independent closure at
2026-10-19T05:10:57.763376+00:00 verified no held keys, empty owned cgroup, pinned
processes gone, VT1 restored, test seat stopped, temporary policy absent, host
IBus/KWin libraries unchanged. `INDEPENDENT-CLOSED-source-three.json` SHA256
`08ab1216cab5a1812e768ece549acb0b1993c508267e071a0bb935ede98b2df1`.

Compared positive actual events in old and new Firefox field10 moved-boundary
traces: old per-character commits acknowledge sentSpace32/observedNBSP160; new
actual fresh SSI enables owned active preedit, whose plain Space commit has only
a generic ASCII suffix receipt. The actual first-word NBSP arrives but does not
update that same-epoch owned tail. Later pair selection reaches selected apply,
but exact final authority correctly refuses the mismatched prefix. New six
Firefox regressions are fields7/8/10 space_burst5 and moved_boundary. Existing
selection failures stay separate. No missing-event inference from rotated logs.

Two real-callback source tests reproduce both empty and Unicode full prefixes,
assert live preedit (no per-letter commits), actual oneSpace CommitText and actual
SSI. Unchanged runtime gives0PASS/2FAIL at the exact boundary assertion:
`active-composition-space-receipt-red-source-one/RESULT.json`,
FOCUSED.log SHA256 `bb0cf83cfc77005cd95a971b98879f5946ceff488e18c93ce0b3bd5cd9ae0bcf`,
2026-10-19T05:15:18.051630+00:00. Owning decision
`2026-10-19-owned-preedit-space-observation.json` chooses the existing exact
receipt/tail-owner acknowledgement; source implementation and native proof
pending. No delivery-route or user installation change.

2026-10-19T05:23:43.785851+00:00 — Owned active-composition Space observation
is source GREEN. The existing postcondition confirms ASCII or an actual full
client snapshot differing only at the newly committed final Space (ASCII32 to
NBSP160). The original shared tail owner acknowledges that exact same-epoch
observation; dispatch identity remains immutable. Changed prefix/word, selection,
interior caret, a changed older boundary, expiry, new input epoch and foreign
shared ownership refuse. No feedback/learning is generated, UnknownStart is not
promoted, and the existing output route and verifier remain authoritative.

Formatted focused tests:5PASS/0FAIL, receipt
`active-composition-space-receipt-green-source-three/RESULT.json` at
05:20:26.869555Z; FOCUSED.log SHA256
`d5ecee3f2027e8249f29fa9ee59c608166785e273e67acf32fd118969cce51bc`.
Official hermetic gate:869 correctness/package PASS,0FAIL,334 process-isolated;
3 separate performance PASS,0FAIL,120.171seconds total. Receipt
`owned-preedit-space-hermetic-source-one/RESULT.json` at05:22:34.647018Z,
SHA256 `10fcedb6c9c275846a0e9c3a950c8329c9c02497060d022f859a262ab1071152`.
Architecture graph update/check PASS, receipt
`owned-preedit-space-architecture-source-one/RESULT.json` at05:23:43.785851Z,
SHA256 `30890f38925f3c25a3d0b5763c5e8779462c8965d3a2c4bf967d6ae8ff7e9f8f`;
generated embedded graph receipt SHA256
`54a88a82ceaa55e30e0cb240218e4edf56465c9d16c4b8fa9ae222071383ca8a`.
Private final build/native all320 are next; these are source facts, not a new
native acceptance. Last complete browser279/41/0 and KDE432360/68/4 remain.

2026-10-19T05:29:09.381256+00:00 — Private final build PASS,211.077seconds,
after the scoped embedded architecture receipt test PASS. IME SHA256
`f04052b184f6a6b40d16db2a3a729cc362462629b2f80040ed626b7fbd082a53`,
8,463,296bytes. `owned-preedit-space-final-build-source-one/RESULT.json`
SHA256 `f6432c77ad0b903cb7a0f0443c43d25cafee49062c4ec590050897eb4f20ec17`
binds the identical869+3 tested Rust source, generated receipt and exact bytes.
All320 unchanged original browser identities are frozen in native phase`kbdu`,
artifact manifest SHA256
`6b5f59d633d13f0bd20c3de22eecf3e1be9ee7dab7aad45cb60edb5a387ca5aa`.
Remote preparation60PASS plus one expected source controlledRED and one expected
readiness controlledRED; zero-key GI preflight PASS. The preparation adapter
requires the preceding explicit CLOSED_ABORTED_EVIDENCE_ONLY receipt and keeps
all independent no-held-key/process/seat/host-preservation conjuncts. Passive
panel capture has a finite64MiB bound; native cases/cadence/assertions unchanged.
Native run pending; no qualification or installation is inferred from this build.

2026-10-19T05:36:43.556129+00:00 — The complete original320 browser component
on exact private IME`f04052b1` is Firefox149PASS/11FAIL/0BLOCKED and
Chrome142PASS/18FAIL/0BLOCKED: total291PASS/29FAIL/0BLOCKED. All identities,
input actions, cadence, targets and assertions are retained. Firefox's six
rich-field Space/boundary regressions from the aborted f800 experiment pass.
Remaining failures are Firefox10selection+1focus_return(field1), Chrome10selection
and8field2 Search seed failures. The latter eight fail before the manual gesture:
AT-SPI reports the text minus its final composing character with the full-length
caret. Visible preedit versus observer completeness remains unproved; keep FAIL.
The first failed authority transition for the isolated Firefox focus case is
still under diagnosis; do not infer absence from the rotated/truncated IME trace.

Native receipts, relative to the development evidence directory
`/home/ubu/.cache/lay/development/browser-final-fixes-20261018/`:
`owned-preedit-space-native-candidate-full-source-one/evidence/native-windows-jab8xhya/RESULT.json`
SHA256`e4d14892c7bfb3d4917c116b37276ff7586c2bf3060ef211dcca55b3e8dc6e35`;
Chrome`native-windows-w6ehvgc3/RESULT.json`
SHA256`d27ee39c9f02294c8b7689cfc1654b219f00bbf416329c684a9a3d8f239ba31b`.
Export archive SHA256
`8d4f6d379d0d52b3b6d1bee91804c7d6af5de071071d2855016643fd34d681f9`,
28,426,250bytes/3292files including1280raw native snapshots. Independent closure
`evidence/INDEPENDENT-CLOSED-source-three.json` SHA256
`a467c6c41430011c01edbc645b34890fcb2a064763c7b7433d4232547085cb3c` is
CLOSED_DIAGNOSTIC_ONLY: no held keys, empty owned cgroup, pinned processes gone,
VT1 restored, owned seat stopped, temporary policy absent, host libraries unchanged.

Measured successful native Double Shift operations: Firefox n200 median30.057ms,
p95174.306ms,max186.217ms; Chrome n185 median29.982ms,p95176.549ms,max182.974ms.
Successful focus-return: Firefox n9 median162.438ms,p95177.442ms; Chrome n9
median172.453ms,p95174.597ms. Native Space: Firefox n50 median29.675ms,
p9532.556ms; Chrome n50 median29.933ms,p9532.096ms. These successful-operation
latencies exclude refusals, whose FAIL rows remain counted; no blanket speed or
compatibility claim follows. Private IBus/KWin dependencies retain their recorded
qualification gaps. No delivery-route change, user installation or release.

The next frozen component is the original112 GTK/Qt/KWrite/Konsole cases in
phase`kbdv`, on identical f040 bytes and private dependencies. Its preparation
passed60source contracts and both expected controlledRED checks. Do not combine
these320 with old112 receipts as a new complete432 result. The prior complete
KDE432 remains360PASS/68FAIL/4BLOCKED until all432 have matching current bytes.


### 2026-10-19T05:42:54.014561+00:00 — matching original112 completed; aggregate regression

The unchanged original112 GTK/Qt/KWrite/Konsole cases completed on the same
private f04052b184f6a6b40d16db2a3a729cc362462629b2f80040ed626b7fbd082a53
IME and private IBus/KWin candidates as the preceding browser320. Results:
GTK14PASS/18FAIL/0BLOCKED (32), Qt21PASS/27FAIL/0BLOCKED (48),
KWrite13PASS/2FAIL/1BLOCKED (16), Konsole3PASS/10FAIL/3BLOCKED (16).
Total112:51PASS/57FAIL/4BLOCKED. Matching full432:
**342PASS/86FAIL/4BLOCKED**, compared with the preceding complete360/68/4:
18 fewer PASS and18 more FAIL. The browser improvement291/29/0 does not
justify promotion of this aggregate regression. No user install or release.

The first shared failed transition in eight of nine failing plans per GTK/Qt
field is initial seed observation before the manual gesture. The fixture's
native preedit event contains the exact expected word while committed text
is empty (or the existing prefix only); the generated target reader checks
only committed text even with `committed_required:false`. This is a measured
observer discrepancy, not evidence sufficient to relabel any FAIL. Establish
the protocol's visible-preedit contract and negative controls before changing
observation. All Space/clean/Backspace plans pass in these GTK/Qt fields.
KWrite/Konsole counts are unchanged. Separate remaining Firefox focus failure:
different native InputContext revokes the reducer acquisition while its pending
acquisition fence still occupies the adapter slot; the successor begin returns
Busy. Preserve current fences/ready publications and stale-marker rejection.

Receipts: `/home/ubu/.cache/lay/development/browser-final-fixes-20261018/owned-preedit-space-native-four-apps-source-one/evidence/`:

- GTK: `native-windows-bjebhsxh/RESULT.json`, SHA256 `87604a2ab599a53d7f5bb45d1e63725190feb3975fbc1ef73da331fcdc69ac6c`.

- Qt: `native-windows-xsylgxh7/RESULT.json`, SHA256 `29c26c54328c4326b29e0d7504df8f7904223982324cd5f091dbe0457ef21c30`.

- KWrite: `native-windows-9jquz99l/RESULT.json`, SHA256 `d186fa11016bba5f337d8beaa15bf0e33c3efa854913f67df473d447f098d0ce`.

- Konsole: `native-windows-3mzsvh1a/RESULT.json`, SHA256 `c11831c8b0da3eed153488773747b9e61aa76da869c5da6589c5897d403c48ab`.

Independent closure `INDEPENDENT-CLOSED-source-three.json`:
CLOSED_DIAGNOSTIC_ONLY, SHA256
`6773f8d8ef063b45e1da641a831398ebddaa229a3f45e6bbdc1f75a995dbb263`.
No held keys, live owned cgroup members or pinned processes; VT1 restored,
owned seat stopped; temporary policy absent; host IBus/KWin bytes unchanged.
Export `kbdv-owned-preedit-space-native-four-apps-evidence-source-one.tar.gz`:
SHA256 `78f2c01262bcbac1c77246df0e43b2886a5a0b3635146830cc74fe76e190d19a`,
8452664 bytes,1223 files,448 raw snapshots. Local archive and every exported
file hash verified. This is diagnostic evidence on private dependencies with
retained qualification gaps; delivery routes and installed authority unchanged.


### 2026-10-19T05:50:23Z — revoked acquisition restart hypothesis before editing runtime

Actual checkout c8b244ef609ad9c5610b3e441311342abf17e9c5 plus preserved dirty
changes, installed runtime unchanged. The pending fence and request are owned
by ContextAdmissionAdapter/ContextAdmissionReducer. In the recorded Firefox
field1 focus_return failure, different-context enrichment revokes native
request307 but begin tests pending occupancy before existing stale acquisition
retirement runs. Proposal9/10: reuse refresh_acquisition_fence_ready before
begin; indiscriminate clearing1/10 and timer/retry3/10 rejected. New ADR
2026-10-19-revoked-native-acquisition-retirement.json fixes the scope and guards.
Three semantic P2P regressions prepared: changed field before reply, changed
field after reply/before marker, and idempotent/current-acquisition/bridge
controls. First obtain2 causal failures+1passing negative on unchanged adapter;
then prove fresh exact-context UnknownStart/suffix0, no inherited text and old
marker/expiry immunity. No implementation, native or delivery change yet.


2026-10-19T05:52:16.312705+00:00 — controlled restart result: RED1PASS/2FAIL
on the unchanged adapter, GREEN3PASS after calling its existing acquisition
refresh before begin. Both different-context orders now complete; negatives
retain same-context request identity, Busy for current acquisition/bridge,
UnknownStart/suffix0 and no inherited text; stale old marker and expiry cannot
revoke the new exact-context token. RED/first GREEN receipts are under
`/home/ubu/.cache/lay/development/browser-final-fixes-20261018/native-focus-restart-{red,green}-source-one/`;
ADR records hashes. This is source evidence only. Full source/format/graph and
native pending; same-byte full432342/86/4 remains the accepted diagnostic count.

2026-10-19T05:55:57.983688+00:00 — formatted restart candidate passed the
official frozen-source isolated IME lane:872 executed,872PASS/0FAIL,
875 discovered including3 performance cases; all3 performance cases also PASS.
Discovery manifest SHA256 ed02b2415c5eb1fc044d91c864e5289dcb88fecba6870f324dc27c59659ee21b.
Full receipt `native-focus-restart-hermetic-source-one/RESULT.json` under the
development evidence root has SHA256
ac014396f1111bf9c0ffc8ffd570819b3db9a3591b7eaf66ad8fbbfc2e9911dc.
The formatted focused repeat also passed3/3. These establish source behavior
only. Next: guarded architecture update, private exact-byte build, unchanged
20 native focus-return identities, then the complete original scope if justified.
The runtime authority change is limited to the existing adapter retirement
boundary; installed runtime and delivery routes remain unchanged. No native
result is inferred from source PASS; complete browser291/29/0 and matching
full432342/86/4 remain diagnostic results for the previous f040 bytes.

### 2026-10-19T06:07:02.445395+00:00 — revoked acquisition retirement: native focus witness

Guarded architecture update and embedded-receipt test PASS; private IME build
9804ab65e4adf62ff109487dc53caf48c6e9f8c45b04b404caef920f73fc0ec4,
8462912bytes, completed06:03:34.697738Z. Build receipt
`native-focus-restart-final-build-source-one/RESULT.json` SHA256
3b89b0e90c56f85106aaaeaeb024c9c6d722c90a399b8ad923bacc6ee2406f08.
No user installation, delivery-route change or release.

First native preparation `kbdw` aborted before creating its private directory,
before a seat/session or input: its inherited closure predicate incorrectly
expected an aborted predecessor instead of the pinned completed `kbdv` receipt.
Preserved `native-focus-restart-native-source-one/RUN.json`. The new immutable
attempt `kbdx` corrected that setup predicate only; original native actions,
identities, targets, deadlines and cadence remained unchanged. Its60source
controls and both expected controlledRED preparations passed.

Original20focus-return identities: Firefox10PASS/0FAIL, Chrome9PASS/1FAIL.
Previously failing Firefox field1 passed. Chrome Search field2 still fails at
initial seed (`приве`, caret6) before the focus-return operation. Successful
focus latency: Firefox n10 median162.599ms,p95180.642ms,max180.642ms;
Chrome n9 median169.101ms,p95176.841ms,max176.841ms. The missing Chrome
operation stays FAIL and is excluded from successful-operation latency.
Receipts under `native-focus-restart-native-source-two/evidence/`:
`native-windows-3simj7a1/RESULT.json` SHA256
fd5858f54bb60c9c7cce076ab628498010275c41bd2b7532364223e2dc2cc9ce;
`native-windows-chgorzhj/RESULT.json` SHA256
eac2052c36170e33b24c808fd3684c949904e136b313b47a7a5c272e885eee63.
Independent closure SHA256467874d5bcaf3b9daec4dc4dc15f68420594124a090283a1b303d09394424a5f:
empty cgroup, no pinned processes or held keys, VT1, owned seat stopped,
temporary policy absent, host libraries unchanged. Verified export SHA256
704f1974aa4117dab9a34a7641007d84d7152324fc1d4502347aa3033ad2fc6d,
1560844bytes,292files,80raw endpoint snapshots. This19/1 focus subset does
not replace complete browser291/29/0 or previous matching432342/86/4.
Full browser repetition pending. Source-only GTK cursor API probe succeeded
without input; native observer qualification and GTK/Qt repetition pending.

### 2026-10-19T06:15:43.590020+00:00 — full original browser320 after acquisition retirement

Remote phase `kbdy` completed the original320 native identities unchanged on
private IME9804ab65e4adf62ff109487dc53caf48c6e9f8c45b04b404caef920f73fc0ec4.
Firefox149PASS/11FAIL/0BLOCKED and Chrome142PASS/18FAIL/0BLOCKED; total291/29/0.
The previous Firefox field1 focus-return failure is now PASS, but Firefox field4
`moved_boundary` changed from PASS to FAIL. Thus no aggregate browser improvement
and no promotion claim. Remaining classes:20 selection,8 Chrome Search initial
seed observations,1 Firefox pair-space acknowledgment. The last reaches an
authorized candidate and `active_pair_waiting_committed_snapshot`, then Reset;
this identifies the pending acknowledgment boundary, not yet the Reset cause.
Trace endpoint continuity is UNKNOWN; do not infer absence from rotated logs.

Receipts under `native-focus-restart-native-full-source-one/evidence/`:
Firefox `native-windows-7udd7bq3/RESULT.json` SHA256
04023af61fdeb4ecca667f7e65bd6eec7b8d8703e91940cb05f9cb141c7888e1;
Chrome `native-windows-eqcj5jtk/RESULT.json` SHA256
a692e9010e0d22b8f5c6c8c84740fa9ebb411c3f39335466628f9ee2be6a981d.
Independent closure SHA25689ad7b9e63f0b348f5062053a4a77d488fa5e54d7e6fecc3abf29444e9396260
checks empty cgroup, no pinned processes/held keys, VT1, stopped owned seat,
removed policy and unchanged host libraries. Fully verified export SHA256
f505a9ab20c80b97840c396cb2a6d8189c2bd970ffe4ddc03f3222dfe1787d4e,
28880716bytes,3292files,1280raw snapshots/448353694bytes.

Successful-operation timing only: Double Shift Firefox n200 median29.759ms,
p95176.138ms; Chrome n185 median30.053ms,p95175.218ms. Focus Firefox n10
median169.168ms,p95175.957ms; Chrome n9 median170.190ms,p95177.983ms.
Space Firefox n49 median29.425ms,p9533.923ms; Chrome n50 median29.402ms,
p9531.952ms. Missing operations stay FAIL and are excluded from these samples.
No installation, publication or delivery-route change. Runtime authority is
the explicit adapter retirement change already recorded by its ADR. Matching
new-byte112 and full432 remain NOT_TESTED; previous f040342/86/4 is retained
as a different-byte diagnostic result, not combined into a new full matrix.

Observer qualification at06:16:44.231986Z passed15 zero-input tests, including
real GTK signal/getter cursor observations, stale/nested-context rejection and
residual-preedit/required-commit negatives. Receipt
`native-preedit-observation-source-one/RESULT.json` is source/API evidence only.
Native GTK/Qt composition observation remains pending. The first112 source
preparation correctly failed one selector test (74PASS/1FAIL) because its oracle
still named browser320; no native directory/seat/input was started. Preserve
`kbdz-scripts-source-one`; correct the expected subset from the pinned432
manifest in a new source-two preparation, keeping exact plans/actions intact.

### 2026-10-19 — Complete matching-byte native112 and qualified composition observation

Closed at 2026-10-19T06:31:38.067380+00:00. Private IME SHA256
9804ab65e4adf62ff109487dc53caf48c6e9f8c45b04b404caef920f73fc0ec4,
unchanged from the preceding original browser320. Original native112 identities,
actions, cadence and deadlines are unchanged. Results: GTK28PASS/4FAIL/0BLOCKED
of32; Qt42/6/0 of48; KWrite13/2/1 of16; Konsole3/10/3 of16. Native total86/22/4;
matching full432 is377PASS/51FAIL/4BLOCKED. Browser320 remains291/29/0.
This is diagnostic qualification, not release acceptance. The difference from
the previous342/86/4 includes a corrected measurement: GTK/Qt now observe the
actual live composition and its actual cursor. Do not describe35 newly observed
PASS as35 runtime defects repaired, or regrade earlier receipts.

Evidence base: `/home/ubu/.cache/lay/development/browser-final-fixes-20261018/`.
Native run `native-preedit-observer-four-apps-source-four`, remote phase `kbea`:

- GTK `evidence/native-windows-k4hp5unc/RESULT.json`, SHA256
  2c47164be5ed4800d1ddd6156fcdb24830f46d5b66d70e104a2516423c3724f3.
- Qt `evidence/native-windows-8_4uq8la/RESULT.json`, SHA256
  54c2e09ab91c5f0a190452caa2c158742196fb3496605888ce8ded1092d78e49.
- KWrite `evidence/native-windows-3gkydah5/RESULT.json`, SHA256
  20f7baa1439a7c9aefee3dbb070b39854429e50580ca5c6e0d4c5c68b68f7cb9.
- Konsole `evidence/native-windows-uslkvd3b/RESULT.json`, SHA256
  46a3ca5420ae9ee2a4f40043710f3294e4bbabc8e2b0f9fd7c766d92fececf38.

The read-only observer requires exact actual committed+preedit text, actual
end caret, focus and empty selection. Nonempty preedit also requires an actual
end preedit cursor from a live matching GTK IMContext invocation or Qt Cursor
attribute. Unknown, stale, nested disagreement and residual preedit fail closed.
No expected-text cursor, synthetic widget mutation or AT-SPI fallback is used.
Committed-required operations still reject live preedit. No browser, KWrite or
terminal observer was changed. This is API/surface evidence, not raster proof.

Qualification: `native-preedit-observation-source-two-red/RESULT.json` retained
the cold-process failure (15PASS/1FAIL). Gtk.IMContext's class had not initialized
its signal before signal_lookup; older tests created a subclass first and hid
that problem. The helper now retains `GObject.type_class_ref(Gtk.IMContext)`.
`native-preedit-observation-source-three/RESULT.json` at
2026-10-19T06:27:31.518659+00:00 records16PASS, including fresh-process startup;
helper SHA256c1a5fbc0758e03b778a7fda082881ecbbb0a224520d06106eb2cf9cd753b03dd,
tests SHA256b8b8acef3b9b66f7139426434a12b043f396cd4b31ebe4f3ff70ce7275b62de0.
Final source gates76PASS plus two expected controlled RED mutations passed at
2026-10-19T06:28:44.688107+00:00. These tests do not establish physical rendering.

Preserved attempts: source-one selector oracle failed before native setup;
source-two phase/override mismatch failed before native setup; source-three
`kbdz` failed GTK startup at signal lookup, before any input (sent=[]), closed
2026-10-19T06:26:02.750553+00:00. None is omitted or counted as a completed112.
Source-four is the first completed qualified native112 on these bytes.

Independent closure `evidence/INDEPENDENT-CLOSED-source-three.json` SHA256
e184d0a10afe525ed53af3cee7b0e581a713864693f3c91122924dc12ee0c0ce proves
empty cgroup, pinned processes gone, no held keys, VT1, owned seat stopped,
temporary policy absent and host libraries unchanged. Fully verified export
`kbea-native-preedit-observer-four-apps-evidence-source-four.tar.gz` SHA256
51e7f945030bc70b5120a5c3961fec6eb746c184472f47e0e0d6a22da47008b8:
8418938bytes,1225files,448raw snapshots/127074645bytes.

Successful Double Shift timing: GTK n40 median27.952ms,p95159.559ms;
Qt n60 median29.190ms,p95146.025ms; KWrite n20 median29.608ms,p95140.885ms.
Space: GTK n10 median28.897ms,p9530.602ms; Qt n15 median28.059ms,p9531.007ms;
KWrite n4 median28.857ms,p9530.447ms; Konsole n2 median28.902ms,p9530.173ms.
Refused/missing operations remain FAIL; Konsole has no successful Double Shift
latency sample. Clock starts at evdev-write return, not kernel key release.

Next shared failure under investigation: GTK loses an unfinished typed word
on focus-away/back, before Double Shift. TextView row14/Entry row30 prove exact
`привет` before focus-away and an empty field after return; no gesture is sent.
Qt preserves the word but refuses the subsequent gesture, a distinct failure.
Current publisher always uses IBus PREEDIT_CLEAR. Upstream bus/client source
explicitly clears that mode on focus-out; PREEDIT_COMMIT preserves visible
typed composition. This is a protocol-mode hypothesis pending exact emitted
mode evidence, source RED/GREEN and physical validation. No after-FocusOut
write or inherited word authority is proposed. Unaccepted hints, atomic
payloads, empty text and revoked owners must not gain commit authority.

No installation, publication or delivery-route change. TD159 remains IN_PROGRESS.

#### Owned typed preedit blur-mode experiment — proposal, 2026-10-19

Decision `decisions/2026-10-19-owned-typed-preedit-blur-mode.json` selects the
existing IBus commit policy only for an exact, current reducer-owned typed
legacy composition. Candidate suffixes remain auxiliary. Atomic, revoked,
sealed, sensitive, mismatched and empty payloads remain CLEAR. No FocusOut
callback output, new owner or inherited word grant is permitted. Alternatives:
exact publisher flag9/10; committing from FocusOut1/10; restoring historical
buffer on FocusIn1/10. The last two risk cross-field writes or duplicate text.
First obtain source RED through production legacy key/reducer setup, then
GREEN and full affected source/native evidence. Actual GTK preservation and
the later Double Shift verdict must be reported separately. No candidate
result or authority change is established by this proposal.

Owned-blur focused source evidence: controlled RED1PASS/2FAIL at06:41:25.049917Z,
`owned-preedit-blur-red-source-one/RESULT.json`; GREEN3PASS/0FAIL at06:42:14.923688Z,
`owned-preedit-blur-green-source-one/RESULT.json`. Both record actual production
reducer/key setup and existing publisher effects, hint separation and explicit
clear. Physical GTK preservation/Double Shift remain NOT_TESTED on new bytes.

Owned-blur full source milestone at2026-10-19T06:47:21.879168Z: official remote
hermetic875PASS/0FAIL correctness plus3PASS/0FAIL performance;878discovered,
875unique correctness identities and3performance identities. Receipt
`owned-preedit-blur-hermetic-source-two/RESULT.json`; formatted focused3/0 in
`owned-preedit-blur-green-source-two/RESULT.json`. The first hermetic preparation
failed before executing tests because its unique-discovery assertion still
expected875instead of878; retain `owned-preedit-blur-hermetic-source-one/RESULT.json`
as a preparation failure. No failing functional result was discarded.
Graph, private build and new-byte native results are pending. Runtime change
is confined to existing preedit commit-mode selection; no delivery-route or
installation change.

Owned-blur architecture gate PASS at2026-10-19T06:49:16.757015Z, receipt
`owned-preedit-blur-architecture-source-one/RESULT.json`. Exact six generated
outputs verified on fetch; graph archiveSHA256
99f19d9922468d0e0cd6bfd55668dbe86bf9cf5143ce25ba9abead4411651619.
Private release build and physical behavior remain pending. No installation.

Owned-blur private build PASS at2026-10-19T06:53:12.574030+00:00: release IME SHA256
510c920197ccf59a9c2701a67feadb4a0879321b5ae38cd2f26ac5906e9fc06a (8463680bytes).
Receipt `owned-preedit-blur-final-build-source-one/RESULT.json`, SHA256
3ed3715719c4bd9a9ced6c4c57243876b8b6cce3699a34d469c7f88b920de646. Embedded architecture receipt test
PASS, target budget PASS; exact source correctness875/0 and performance3/0
reused with source hashes and regenerated graph closure. No installed bytes
changed. Native original112 phasekbeb is prepared, NOT_TESTED. First source-gate
launch stopped before tests because the launch environment omitted required
G_DEBUG=fatal-warnings; retained SOURCE-GATES.json.preparation-failure-one and
gate-green.log.preparation-failure-one in the immutable kbeb script package.
Retry sets the required variable; no guard or test was relaxed.

Owned-blur original native112 CLOSED_DIAGNOSTIC_ONLY on exact IME
510c920197ccf59a9c2701a67feadb4a0879321b5ae38cd2f26ac5906e9fc06a:
87PASS/21FAIL/4BLOCKED. GTK29/3/0, Qt42/6/0, KWrite13/2/1,
Konsole3/10/3. Comparison with exact previous112 identities finds precisely
one status change: GTK3 TextView focus_return FAIL -> PASS; no regression.
Original actions, cadence, deadlines and qualified observer stay unchanged.
Receipt `owned-preedit-blur-native-source-one/RUN.json`; exact comparison
`owned-preedit-blur-native-source-one/COMPARISON.json`.

Both GTK widgets now preserve the typed word across focus-away/back. Entry's
remaining FAIL is not word loss: native returned state contains committed
`привет`, caret6 and selection[0,6]. The fixed protocol requires a collapsed
caret and therefore sends no gesture for that row. TextView returns the same
word with collapsed caret and the physical Double Shift converts it correctly.
Actual monitored Engine.UpdatePreeditText uses COMMIT mode1 for owned typed
payload; explicit empty/hide uses CLEAR0. Qt's three focus_return rows retain
the word but the physical gesture reaches IME and is refused context_authority.
No preedit duplication was observed in these112 cases. This is not proof for
unexecuted fields, browsers or installed runtime.

Independent closure at2026-10-19T06:57:38.766691+00:00 verifies empty cgroup,
pinned processes gone, no held keys, VT1, owned seat stopped, temporary policy
absent and host libraries unchanged. Fully verified export SHA256
 d5fb312ad19eb9f5aa4df5e154dc508c52444d58388c2e07c8a8b77cbe54213f:
8778761bytes,1225files,448raw snapshots/126984185bytes. Independent closure
SHA256 a0b2d4508314bef4915f504ad922f1b6f92557aff545f68d00e7115374e85c51.
The first export preparation retained the old9804 binary pin and correctly
rejected before writing a closure/archive. Retained
`EXPORT-PREPARATION-FAIL-source-one.json`; corrected source-two exporter
checks exact510c. No native repetition or receipt/status editing.

Successful Double Shift: GTK n40 median28.192ms,p95161.323ms;
Qt n60 median28.475ms,p95138.611ms; KWrite n20 median29.045ms,p95140.711ms.
GTK focus_return n1=138.559ms. Space GTK n10 median28.478ms,p9531.153ms;
Qt n15 median29.258ms,p9531.125ms; KWrite n4 median30.998ms,p9533.076ms;
Konsole n2 median29.073ms,p9530.822ms. Refused operations remain FAIL;
Konsole has no successful Double Shift sample. Clock starts at evdev write
return, not kernel release. No latency improvement claim from this one run.

Original browser320 on510c is running phasekbec with the same actions and
physical observers. Until complete independent closure, retain latest complete
old-byte browser291PASS/29FAIL/0BLOCKED. Do not combine that old320 with the
new112 into a matching432 claim. TD159 remains IN_PROGRESS; no installation,
publication, delivery-route change or release readiness.


Owned-blur original browser320 CLOSED_DIAGNOSTIC_ONLY at
2026-10-19T07:06:04.516331+00:00 on the same exact private IME510c:
Firefox150PASS/10FAIL/0BLOCKED, Chrome141PASS/19FAIL/0BLOCKED;
total291PASS/29FAIL/0BLOCKED. Exact320 identities/actions remain unchanged.
Two status changes from previous9804: Firefox field4 moved_boundary FAIL->PASS,
Chrome field8 focus_return PASS->FAIL. The Firefox improvement was previously
a late-ack refusal; one run does not establish a permanent timing repair.
The Chrome regression is unresolved; do not accept or promote this candidate.
Receipt `owned-preedit-blur-browser-source-one/RUN.json`; exact comparison
`owned-preedit-blur-browser-source-one/COMPARISON.json`.

Chrome8 focus_return preserves committed `привет`, caret6, no selection.
Physical Double Shift is recognized; IME manual outcome status3 prepares the
conversion, but the daemon cancels after layout handoff with "exact IME tail
retention policy changed". No actual privacy-policy change is established:
VisibleTailV4 also returns false for passive/unknown-context replies, and
current bound_v4_reply compares retention before classifying readiness.
The case's bounded trace snapshot has PREFIX_CHANGED, so no event absence
claim may be made from its interval. Content hints64/purpose0 do not prove
private mode. Capture exact private RPC replies before changing any guard.

Verified browser export SHA256
f9530b99a518765ef23656da03a7487f56724adf19ac706e823248eabc5816b0:
29277590bytes,3292files,1280 raw snapshots/448392601bytes. Independent closure
SHA2565843e004176e52ddec162470afee99af63a045a50bb0dbc4f89a5fb4e0ae3aa1
verifies empty cgroup, pinned processes gone, no held keys, VT1, owned seat
stopped, temporary policy absent and host libraries unchanged.
Successful Double Shift Firefox n200 median30.172ms,p95176.092ms;
Chrome n185 median30.314ms,p95175.178ms. Focus-return successful only:
Firefox n10 median164.266ms,p95200.066ms; Chrome n8 median165.826ms,p95169.999ms.
Space Firefox n50 median28.864ms,p9532.977ms; Chrome n50 median29.273ms,p9532.054ms.
Failed operations remain FAIL, not latency samples.

Both new-byte components are now independently closed. Exact matching432:
378PASS/50FAIL/4BLOCKED. Derived proof with all six receipt hashes is
`owned-preedit-blur-matching-432-source-one.json` under the development cache.
The previous instruction to avoid combining old browser bytes is superseded
only by this exact-byte closure, not by cross-build aggregation.
TD159 remains IN_PROGRESS. No installed bytes, delivery routes or authority
contracts changed. No GNOME acceptance or release qualification.

Next diagnostic kbed: original20 browser focus_return identities only, same
510c bytes/actions/cadence/deadlines. Add only a bounded passive session-bus
monitor pinned to the exact private IME owner/PID; verify a zero-key probe
before input. Source controls60PASS plus two expected RED controls passed at
2026-10-19T07:15:35.596649+00:00. This subset cannot replace browser320 or
matching432 evidence and does not authorize any permission relaxation.


### TD159 — 2026-10-19 matching target observation revokes exact manual proof

kbed diagnostic completed on private510c, with no runtime changes: Firefox
focus10PASS0FAIL, Chrome focus8PASS2FAIL. This subset does not replace the
complete browser320291PASS29FAIL0BLOCKED or matching432378PASS50FAIL4BLOCKED.
Chrome7 fails after layout handoff; Chrome2 stops at seed-readback (`приве`,
caret6), a separate measurement/seed problem. Chrome8 passes in this subset.

Passive monitor pins the private IME unique bus owner and process and records
77V4calls/77replies. At unix1792394175.367585, RU27 returns exact `привет`,
epoch405, same InputContext16, retention=true. US28 FocusIn occurs at .411020;
its actual SetSurroundingText `привет`,6,6 occurs at .412112. At .444513 V4
returns passive:unknown-context with retention=false. The actual matching
observation precedes the failed query by32ms; no changed privacy metadata
or delayed missing observation explains this failure.

First lost authority: observe_external_surrounding_text always calls
advance_surrounding_observation_revision, which drops both the inherited
target certificate and reducer exact_manual_snapshot. If observation arrives
before installation, existing installer republishes its matching receipt;
after installation it is irretrievably dropped. The former test explicitly
required even identical later callbacks to revoke: this is the policy race.

Decision2026-10-19-exact-manual-observation-refresh: retain only a current
receipt across an actual fully matching prefix/tail/collapsed-end tuple, same
owner/epoch, original expiry. Changed/selected/interior/absent/revoked/consumed/
expired/sensitive proofs cannot revive. UnknownStart remains UnknownStart;
no new owner, delivery path, output mutation or verifier relaxation.
Ratings: existing-certificate refresh9/10; extra waiting2/10 (SSI already
arrived); ignore SSI or accept unknown reply1/10 (unsafe authority bypass).
Three new semantic tests added; runtime refresh not yet implemented/tested.

Diagnostic RUN sha2569037c956839aa515844bea5c63b836fae3a2c49042f5c89c9407f466e10cd9eb.
Export bf278500efe455c34c38a411f2e5cd4952518eb509d3a4bebb6c372b25d5ae1b,
1500826bytes294files80raw18325175bytes. Closure at07:17:12.498445UTC:
00f387e1a0faa768f6f0e1167469ca0546c1f76707d1131749afdd96801933ef,
empty cgroup/pinned processes gone/no held keys/VT1/owned seat stopped/
temporary policy absent/host IBus and KWin libraries unchanged.
All receipts under development cache/owned-blur-focus-rpc-source-one.
No installation/publication/GNOME acceptance. TD159 remains IN_PROGRESS.


Exact-manual observation refresh controlled RED at2026-10-19T07:37:28.945514+00:00:2PASS1FAIL,
expected failure on the first actual target observation after install;
changed/revoked/no-gesture negatives pass. Runtime change uses existing
reducer's current certificate getter and publisher, no new state or route.
Capture precedes invalidation; refresh requires known full prefix equality,
unchanged tail/epoch, collapsed end-of-field, current owner and live handoff.
Original expiry is reused and revalidated by the reducer. Prior unconditional
repeat-revocation test now proves changed-prefix revocation plus no revival.
Focused GREEN at2026-10-19T07:38:47.040662+00:00:3PASS0FAIL.
Receipts exact-manual-observation-red-source-one/RESULT.json and
exact-manual-observation-green-source-one/RESULT.json under development cache.
Native remains NOT_TESTED for these changed bytes; complete matrix unchanged.

Formatted focused GREEN3PASS0FAIL at2026-10-19T07:40:34.855394+00:00.
Getter invalidation has one production caller, the synchronous engine
observation revision advance; callback/output consumption runs under the
same engine mutation serialization. Concurrent reducer revocation/tail/
lineage changes are revalidated at the existing publisher. No second mutable
certificate store was added. Scoped source gates do not certify physical
Chrome readiness. Official hermetic gate is now running remotely.

First official hermetic gate at2026-10-19T07:42:51.245630UTC:
877PASS1FAIL/878 correctness,881 discovered, performance NOT_RUN. Existing
td159_fresh_focus_manual_source_cannot_revive_after_receipt_change_or_reset
catches overbroad source-engine refresh. Do not weaken this source protection.
Narrow to a receiver using existing sealed source handoff path, which transfer
retains while active_path and reducer owner move to the target. Source path
equal to current engine cannot refresh; current target still needs all
certificate/owner/epoch/full-observation/original-expiry checks. No new state.
The recorded failing whole-set gate is the RED evidence for this narrowing.

Receiver-only focused GREEN3PASS0FAIL at2026-10-19T07:44:42.322630UTC.
The sealed existing source path distinguishes the admitted receiver; original
source callback protection remains in the unchanged old test. Second official
hermetic/format gate now running. No native claim or installed changes.

Receiver-only official hermetic GREEN at2026-10-19T07:47:07.618129UTC:
878PASS0FAIL/878 correctness,881 discovered,343 isolated,535 bulk;
all3 performance tests PASS. Remote formatting check passed. The unchanged
source callback/reset protection is retained. Exact receipt is development
cache/exact-manual-observation-hermetic-source-two/RESULT.json. These source
results do not establish native acceptance or browser latency. Next is the
remote architecture graph gate and a pinned private build, followed by the
unchanged original browser320 and native112 protocols. Delivery routes,
installed binaries and desktop applications remain unchanged.

Remote architecture graph gate PASS at2026-10-19T07:48:48.667743UTC;
existing scripts/update-architecture-graph.sh completed successfully and all
six generated outputs were hash-verified on export. Receipt: development
cache/exact-manual-observation-architecture-source-one/RESULT.json. Runtime
change remains limited to receiver certificate refresh; no delivery change.
Pinned private release build is next; native acceptance remains NOT_TESTED.

Pinned private IME build completed at2026-10-19T07:52:50.314986+00:00:
SHA256 fbd4fd2ed5da81857d6c70e77f93a3daf6331b79aa92217399761db484704af4, bytes8466368.
Build receipt /home/ubu/.cache/lay/development/browser-final-fixes-20261018/exact-manual-observation-final-build-source-one/RESULT.json;
receipt SHA256 8c6e34f8c96a33b6b5aab06055560c0a1e40ccc2bcd0f5f0296aed3d8de09588.
All878 source correctness and3 performance checks are bound to identical
Rust sources; generated architecture receipt-only delta has its own embedded
receipt test PASS. No installation. Browser320 kbee uses this exact binary;
native112 kbef must use the same bytes before a matching432 comparison.

Receiver-only exact-manual refresh original browser320 completed; independent
closure at2026-10-19T08:00:33.456117+00:00. Exact private IME
fbd4fd2ed5da81857d6c70e77f93a3daf6331b79aa92217399761db484704af4.
Firefox150PASS10FAIL0BLOCKED/160; Chrome142PASS18FAIL0BLOCKED/160;
browser total292PASS28FAIL0BLOCKED/320. Exactly one changed identity versus
510c: Chrome8 focus_return FAIL->PASS. Remaining319 statuses unchanged, all
320 identities and input/observer protocol scripts equal after phase-only
normalization. No native112 or GNOME acceptance from this browser component.
Confirmed successful DS latency: Firefox n200 median29.955ms p95176.665ms;
Chrome n185 median30.276ms p95172.633ms. Successful focus-return Firefox
n10 median162.632ms p95180.031ms, Chrome n9 median160.834ms p95180.299ms.
Space n50 each: Firefox median29.764ms p9533.471ms; Chrome median29.392ms
p9532.349ms. Clock remains evdev-write-return to confirmed target; failed
operations remain FAIL and are excluded from successful latency samples.
Full raw export SHA25655ffa73af5fca4682d6b464d23adabd650871646cc27d45d1b6a471b963cb5c5,
28586306bytes3292files1280raw448661347bytes. Closure SHA256
93fc2f374fc74bdebd4ba5c9ee6dec89eeb943565d227f600f4b07698a27ff1c; empty owned cgroup,
pinned processes gone, keys released, VT1 restored, seat stopped, temporary
policy absent, host IBus and KWin libraries unchanged. Evidence and complete
post-row comparison: /home/ubu/.cache/lay/development/browser-final-fixes-20261018/exact-manual-observation-browser-source-one/COMPARISON.json.
Same-byte native112 kbef is prepared; no installation/publication.

Receiver-only refresh original native112 independently closed at 2026-10-19T08:08:52.185671+00:00.
GTK29PASS3FAIL0BLOCKED/32; Qt42PASS6FAIL0BLOCKED/48;
KWrite13PASS2FAIL1BLOCKED/16; Konsole3PASS10FAIL3BLOCKED/16.
All112 identities, statuses and phase-normalized input/observer protocol
scripts are unchanged versus510c. Same fbd4 IME bytes as browser320.
Combined432 at 2026-10-19T08:09:08.028376+00:00:379PASS49FAIL4BLOCKED,
previous378PASS50FAIL4BLOCKED; exactly Chrome8 focus_return FAIL->PASS.
Receipt: /home/ubu/.cache/lay/development/browser-final-fixes-20261018/exact-manual-observation-matching-432-source-one.json
Native raw export SHA256 99b8ff39de4d1969856c56e0300128d7d3fa6f655a86078926ead2aca9cad9a9, 8813867bytes 1225files,
448 raw endpoint snapshots/126946779bytes; each member verified.
Native closure SHA256 c319524738a7ce9e8b7695437139c853ec3008803f7f6e349a9ffb6bbac168e0;
empty owned cgroup, pinned processes gone, keys released, VT1 restored,
owned seat stopped, temporary policy absent, host IBus/KWin bytes unchanged.
Export first deferred to an unrelated L3 heavy lease; no lease bypass or
process interruption. Before export, corrected a stale template IME hash and
required agreement with the separately pinned private override; no product
code, native inputs or verdict changed by this evidence-script correction.
Native successful DS latency: GTK n40 median28.241ms p95164.401ms;
Qt n60 median28.588ms p95138.485ms; KWrite n20 median30.078ms p95140.823ms.
Konsole has no successful DS latency sample. This is not an acceleration claim.
Scope remains private KDE diagnostic: upstream KWin qualification failures
and GNOME are not accepted, no installation/publication.
Next shared failure: all three Qt focus_return plans retain committed привет
with collapsed end caret after Double Shift. GTK Entry focus_return instead
stops before gesture because the seed is selected; those mechanisms remain
separate. Analyze the real Qt Reset/observation/gesture order before editing.


### TD159 owned-preedit Reset current-field admission — design at 2026-10-19T08:23:19.659920+00:00

Original native112 on the pinned fbd4fd2e IME confirms three Qt focus_return failures: RichTextEdit, PlainTextEdit and LineEdit retain actual `привет`, collapsed end caret and focus, but the recognized Double Shift is refused with context_authority. Actual Reset callbacks retire owned preedit; source inspection identifies a second broad revocation during its cleanup. This is distinct from GTK Entry selecting its seed before the gesture. Preserve existing cancellation: no old preedit or committed prefix may become a new field source. ADR `decisions/2026-10-19-owned-preedit-reset-manual-source.json` proposes only a new exact manual read after authenticated owned-preedit retirement, actual post-reset surrounding observation and explicit gesture. Two new semantic test functions are prepared; production code is unchanged and controlled RED is pending. Latest complete diagnostic stays379PASS/49FAIL/4BLOCKED; no source/native acceptance claimed for the proposal. Solo remote-only, no installation/publication or delivery-route change.

Owned-preedit Reset first RED attempt at 2026-10-19T08:24:35.046747+00:00: the positive test reached and failed the intended missing-fresh-admission assertion; the negative test also stopped on a detached callback transport reply (serial9203 before9204). Overall source-one is FAIL and is not accepted controlled RED. The fixture now drains each real detached key callback reply before Reset; no product code or expected behavior changed. Source-two is the next independent frozen run.

Owned-preedit Reset controlled RED at 2026-10-19T08:25:08.775874+00:00: source-two1PASS/1FAIL, exactly the required new-current-field manual-read assertion fails. Negative source safety test passes after fixture reply ordering correction. Minimal reducer/state/Reset-adapter implementation prepared at 2026-10-19T08:27:08.569439+00:00; no GREEN or physical acceptance claimed. Test source and output routes are unchanged after RED.

Owned-preedit Reset first implementation at 2026-10-19T08:28:20.797486+00:00: frozen green-source-two remains1PASS/1FAIL on the same intended positive assertion, all new negative checks pass. This is not GREEN and no build/native promotion occurred. Added bounded assertion diagnostics (old/current token, live status, reducer fresh flag, actual snapshot) to identify the first failed guard before further eligibility changes. The new path explicitly excludes existing owned Space/managed-commit/replay observations and pending manual/replay state; those keep their existing contracts.

Owned-preedit Reset fixture diagnosis at 2026-10-19T08:35:08.287755+00:00: green-source-three1PASS/1FAIL identifies hints2048 applied after typing, which correctly clears prior volatile preedit at the privacy boundary. Green-source-four1PASS/1FAIL stops before Reset because actual private input uses the committed route and emits CommitText; it cannot satisfy the owned-preedit-only prerequisite. These are recorded fixture failures, not runtime GREEN. Source-five keeps ordinary/repeated Reset positive cases, establishes private metadata before actual keys, asserts the real private commits/no-owned-preedit, and checks that this path cannot invent fresh owned-source admission. Privacy changes remain a separate negative guard; no privacy/delivery contract changed. Evidence: /home/ubu/.cache/lay/development/browser-final-fixes-20261018/owned-preedit-reset-manual-green-source-three/RESULT.json and /home/ubu/.cache/lay/development/browser-final-fixes-20261018/owned-preedit-reset-manual-green-source-four/RESULT.json. Native remains NOT_TESTED for the proposal; latest complete432 stays379PASS49FAIL4BLOCKED.

Owned-preedit Reset focused GREEN at2026-10-19T08:36:29.866135+00:00: formatted frozen source-six2PASS0FAIL; source-five also2PASS0FAIL before formatting. Positive scenarios one/two actual Reset callbacks, new Unicode-prefixed text, no old prefix/preedit, no automatic authority. Negative scenarios include unobserved Reset, selection, interior caret, sensitive field, revoked owner, privacy change and actual private committed input without owned preedit. Source-six log SHA25642b508a94c74febd2471b77016e2ad401b0a9536659061e6b75b9e0bce98175d. Remote six-file format PASS at2026-10-19T08:35:26.316860+00:00. Official880correctness+3performance gate started; physical result NOT_TESTED. Receipts: /home/ubu/.cache/lay/development/browser-final-fixes-20261018/owned-preedit-reset-manual-green-source-six/RESULT.json and /home/ubu/.cache/lay/development/browser-final-fixes-20261018/owned-preedit-reset-manual-format-source-one/RESULT.json.

Owned-preedit Reset complete source GREEN at 2026-10-19T08:39:14.269243+00:00:883 discovered,880 correctness/package executed PASS (345 isolated,535 bulk),3 performance PASS; no failures or selection change. Source-six frozen Rust and tooling closure unchanged; six-file remote rustfmt check PASS. Receipt: /home/ubu/.cache/lay/development/browser-final-fixes-20261018/owned-preedit-reset-manual-hermetic-source-one/RESULT.json; discovery SHA256138a02c2a8868db0dd09d0267d97d7b94b00397e4bc0bdc9d9beed6089967cde. Source authority delta is only eligibility for a new actual current-field manual observation after authenticated owned-preedit retirement; no old text authority, automatic authority or delivery-route change. Architecture refresh and immutable private build are next, native acceptance remains NOT_TESTED. Latest complete432 stays379PASS49FAIL4BLOCKED.


Owned-preedit Reset architecture/build qualification recorded at 2026-10-19T08:50:12.424214+00:00: remote architecture gate PASS at2026-10-19T08:40:40.266332+00:00, receipt development cache/owned-preedit-reset-manual-architecture-source-one/RESULT.json. Pinned private build at2026-10-19T08:44:37.890917+00:00: SHA2561c1641d4e61e3c7409f93c5dd0ba0525d6cc0e4f66a613ae267b37a8bce467ba,8469312bytes; receipt development cache/owned-preedit-reset-manual-final-build-source-one/RESULT.json,880 correctness/package and3 performance source tests bound to frozen Rust/tooling, embedded architecture receipt PASS.
Browser package source-one zero-key qualification failed59/60: prepare.py expected the ADR filename stem as its semantic id, whereas the pinned ADR id is TD159-owned-preedit-reset-current-manual-source. Binary, tests and source counts were correct. Preserve this failure; no keys were sent for source-one. Immutable source-two corrects only the expected semantic id; pinned binary/source/ADR checks remain. All60 controls PASS and both intentional negative controls fail as expected. All eight input/observer protocol scripts exactly equal previous browser320 after phase-only normalization. Original browser320 kbeg is now running on the new bytes; native112 kbeh follows. No physical result yet for this proposal; latest complete matching432 remains379PASS49FAIL4BLOCKED. No installation/publication or delivery-route change.


Owned-preedit Reset original browser320 independently closed at 2026-10-19T08:56:29.418911+00:00: pinned IME1c1641d4e61e3c7409f93c5dd0ba0525d6cc0e4f66a613ae267b37a8bce467ba, Firefox149PASS11FAIL0BLOCKED/160, Chrome142PASS18FAIL0BLOCKED/160; total291PASS29FAIL0BLOCKED/320. Exact320 identity set and eight phase-normalized input/observer scripts equal fbd4 baseline. Exactly one changed identity: Firefox2 moved_boundary PASS->FAIL, actual `должн ыбыть ` remains instead of `должны быть `. Remaining319 statuses unchanged. Candidate is found and authorized; captured trace reaches active_pair_waiting_committed_snapshot and then a Reset. This is a first-failed-transition lead, not yet proof that the new owned-source path caused the failure; bounded producer completeness remains UNKNOWN. Do not accept the proposal with this browser regression or hide it by repeating until PASS.
Full verified export SHA256a333c0c19c18394b17c0db618df05bf4af4e95325f03b88a27cc5e80fdaed40a,29140833bytes3292files1280raw448607617bytes; closure verifies empty cgroup, pinned processes gone, keys released, VT1 restored, owned seat stopped, temporary policy absent and unchanged host IBus/KWin bytes. Receipt: /home/ubu/.cache/lay/development/browser-final-fixes-20261018/owned-preedit-reset-manual-browser-source-two/COMPARISON.json. Successful DS: Firefox n200 median29.727ms p95174.631ms; Chrome n185 median30.217ms p95173.488ms, evdev-write-return to confirmed target; failed operations remain in counts and this is not an acceleration claim. Native112 kbeh now runs the same pinned bytes after76 protocol controls PASS and both controlled negatives. No matching432 claim yet; GNOME and upstream KDE qualification remain outside acceptance.


Owned-preedit Reset complete native112 and matching432 rejected at 2026-10-19T09:06:46.955350+00:00: GTK29PASS3FAIL0BLOCKED/32, Qt42PASS6FAIL0BLOCKED/48, KWrite13PASS2FAIL1BLOCKED/16, Konsole3PASS10FAIL3BLOCKED/16; native total87PASS21FAIL4BLOCKED/112. All112 statuses match fbd4 baseline. Browser320 plus native112 use identical pinned 1c1641d4 IME bytes, equal protocol scripts after phase-only normalization and independently verified closures. Matching432 is378PASS50FAIL4BLOCKED versus379PASS49FAIL4BLOCKED; exactly Firefox2 moved_boundary regressed. Candidate is REJECTED, no Qt focus_return recovery. Receipt: /home/ubu/.cache/lay/development/browser-final-fixes-20261018/owned-preedit-reset-manual-matching-432-source-one.json. Native export SHA256e38f9087b2a4b727498582f4a5c41ca09a9db7d56d045fb7aeb85bcbe906827d,8558018bytes1225files448raw127212602bytes; component receipt development cache/owned-preedit-reset-manual-native-source-one/COMPARISON.json. No installation/publication or delivery-route change.

Actual Qt RichTextEdit focus_return positive trace order: owned six-character preedit and actual six-character surrounding snapshot precede Reset3359; actual empty surrounding then six-character surrounding follow; Reset3364 occurs on the same owner82; the subsequent Shift callbacks explicitly bind composition0, buffer0, snapshot_chars=null, surrounding_revision5, tail_epoch859 before the context_authority refusal. This proves snapshot absence at the gesture, not global absence of callbacks. Trace producer/interval completeness remains UNKNOWN. The two source regression functions supplied fresh surrounding AFTER the final Reset and therefore did not cover this physical ordering. They establish source admission under that narrower ordering, not Qt acceptance. Next controlled source reproduction must use surrounding between the two Resets and require a new actual read owned by the same explicit gesture; no old snapshot/prefix retention. A read-only request through existing bounded pending-manual continuation is a design candidate, not yet implemented or proved. Firefox pair waiting/reset lead remains a separate regression pending causal diagnosis.


TD159 actual Qt-order controlled RED at2026-10-19T09:09:46.055655+00:00:2PASS1FAIL, fresh_manual_source_available=true but ManualToggleV3 returns0 instead of4 without a new snapshot. Receipt /home/ubu/.cache/lay/development/browser-final-fixes-20261018/owned-preedit-reset-read-red-source-one/RESULT.json. At 2026-10-19T09:11:08.131847+00:00 decision decisions/2026-10-19-fresh-manual-read-same-gesture.json selects one read under current fresh admission and existing same-gesture pending700ms state (9/10), over old-snapshot retention(2/10) or unconditional refresh(4/10). Expected effects: exactly one RequireSurroundingText and zero Delete/Commit before a new current end-caret callback; callback stays read-only and same daemon gesture consumes existing exact-tail handoff. Preserve all Reset/privacy/owner/timeout cancellations, original matrices and route authority. No production edit yet; new physical acceptance NOT_TESTED.


TD159 fresh-read first implementation at2026-10-19T09:14:28.698446+00:00: exact four identities3PASS1FAIL. Actual Qt-order request/resume test PASS, including one read and no callback-local Delete/Commit. Negative matrix fails at privacy_change: close_committed_tail_field clears the pending lease but leaves reducer fresh admission available, and the following same-gesture RPC delegates a newly observed tail. Receipt development cache/owned-preedit-reset-read-green-source-one/RESULT.json. This candidate is not promoted. At 2026-10-19T09:15:32.908949+00:00 narrow cancellation repair selected: field close and surrounding capability loss while an empty-source fresh read is pending must revoke that existing fresh admission before clearing pending state, as Reset cancellation already does. Add capability_loss negative; no new timer, state owner, delivery route or weakened assertion.


TD159 fresh-read cancellation source-two at2026-10-19T09:17:09.813213+00:00:3PASS1FAIL; privacy_change and capability_loss now pass, final expired callback still returns DelegateExactImeTail after700ms. Receipt development cache/owned-preedit-reset-read-green-source-two/RESULT.json. First failed transition: expiry clears pending timestamp without spending fresh read admission, so a late callback is re-admitted by the next poll. Apply existing revoke_pending_fresh_manual_source before expiry cancellation in both callback and manual RPC. Add expired_rpc (no intervening callback) and later-poll checks across all nine gaps. Remote six-file formatting receipt at2026-10-19T09:19:52.153025+00:00 in owned-preedit-reset-read-format-source-three/RESULT.json. No build, client input, installation or promotion.


TD159 fresh-read focused GREEN at2026-10-19T09:20:46.454849+00:00:4PASS0FAIL, exact four identities; nine cancellation gaps including callback expiry and RPC expiry plus later-poll no-renewal. Controlled RED remains2PASS1FAIL. Exactly one fresh actual read, different post-Reset word accepted only via same-gesture existing exact-tail handoff; no callback-local Delete/Commit. Receipt /home/ubu/.cache/lay/development/browser-final-fixes-20261018/owned-preedit-reset-read-green-source-three/RESULT.json. Full hermetic source, graph, private build and physical clients pending; no installation/publication or delivery-route change.


TD159 same-gesture fresh read full hermetic source gate at2026-10-19T09:23:15.969740+00:00:882PASS0FAIL correctness/package across885 discovered identities (535 bulk and347 process-isolated),3performancePASS. Official discovery and hermetic executor over frozen Rust/tooling closure; no ignored test substitution. Receipt /home/ubu/.cache/lay/development/browser-final-fixes-20261018/owned-preedit-reset-read-hermetic-source-one/RESULT.json. All nine cancellation gaps and actual Qt callback/Reset ordering retained. Graph/private build/physical acceptance pending; this does not improve the latest complete432378PASS50FAIL4BLOCKED. Delivery routes unchanged; no installation/publication.


TD159 same-gesture fresh-read architecture gate at2026-10-19T09:24:30.507765+00:00:PASS; graph refreshed remotely with existing AST-only gate, all owning/callsite/delivery checks retained. Receipt /home/ubu/.cache/lay/development/browser-final-fixes-20261018/owned-preedit-reset-read-architecture-source-one/RESULT.json. Prior full source882+3PASS; next immutable private IME build and matching physical clients. No new input/output owner, timeout, fallback, client bypass, installation or publication. Firefox2 pair-space trace independently re-read under exact report and endpoint SHA pins: selected correction awaits committed snapshot then Reset9511 rejects rereceipt for missing predecessor/post-reset token. Endpoints PREFIX_CHANGED; producer/interval completeness UNKNOWN. This positive event sequence is a lead, not proof that the current manual-read change caused or fixes the pair regression.


TD159 same-gesture fresh-read private build at2026-10-19T09:28:28.359252+00:00: IME SHA256a46dd831f94f29d244671f64096ebe1a0e04ae2663fca442f9137814fa547adc,8472256bytes; source882correctness/package+3performancePASS, graphPASS, embedded architecture receipt testPASS. Receipt /home/ubu/.cache/lay/development/browser-final-fixes-20261018/owned-preedit-reset-read-final-build-source-one/RESULT.json. Frozen identical Rust/test tooling proof reused except refreshed generated architecture receipt JSON; all source hashes checked before/after. Scope: private diagnostic build only; native NOT_TESTED, latest complete432 unchanged378PASS50FAIL4BLOCKED. No installation, publication or delivery-route change. Next browser320 kbei then native112 kbej with original input scripts.

TD159 same-gesture fresh-read complete physical result at2026-10-19T09:40:35.346737+00:00: same a46dd831 IME, Firefox150PASS10FAIL0BLOCKED/160, Chrome141PASS19FAIL0BLOCKED/160; browser291PASS29FAIL0BLOCKED/320. Native GTK29PASS3FAIL0BLOCKED/32, Qt44PASS4FAIL0BLOCKED/48, KWrite13PASS2FAIL1BLOCKED/16, Konsole3PASS10FAIL3BLOCKED/16; native89PASS19FAIL4BLOCKED/112. Matching432 is380PASS48FAIL4BLOCKED versus fbd4 baseline379PASS49FAIL4BLOCKED. Exactly Qt5 LineEdit and PlainTextEdit focus_return recover; RichTextEdit focus_return remains FAIL. Exactly Chrome2 moved_boundary regresses. Receipt /home/ubu/.cache/lay/development/browser-final-fixes-20261018/owned-preedit-reset-read-matching-432-source-one.json; per-component COMPARISON.json in owned-preedit-reset-read-browser-source-one and owned-preedit-reset-read-native-source-one. All identities and eight protocol scripts per component match after phase-only normalization; deadlines/cadences unchanged. Independently verified closures restore VT1, release keys, remove temporary policy and leave host IBus/KWin unchanged. Browser export SHA2568861643ed191f9024a605affad3bb063c938007fd842e4c36bdd141fd9cda157; native export SHA25697c2541e2298eb34639936cd418bea3dfd6b2784887d20395a5885968eae8269; all exported members checked. Verdict PARTIAL_NATIVE_IMPROVEMENT_NOT_ACCEPTED: new browser regression, remaining FAIL/BLOCKED, GNOME NOT_TESTED and private upstream KWin qualification FAIL. No installation/publication; runtime authority and delivery routes unchanged.

Chrome2 moved_boundary positive evidence, bound to exact exported report native-windows-i0g_xxxy and endpoint SHA pins: candidate requests an actual committed snapshot; actual surrounding12/cursor12 arrives; existing surrounding_text_immediate_delete_commit dispatches Delete12 and Commit12; actual client AT-SPI observes 'должн ыбыть должны быть ' with caret24 and focus retained. Following surrounding24 remains pending_stale_observation, so verifier does not certify the replacement. This is duplication after dispatch, distinct from prior Firefox2 unchanged-text/Reset ordering. Dispatch applied=true is not physical acceptance. Rotating endpoints have PREFIX_CHANGED and producer/interval completeness UNKNOWN: positive events establish this sequence, not absence of other callbacks or a proved transport cause. Next diagnosis locates deletion acknowledgement versus insertion ordering in the existing owner/executor; no client-specific bypass or relaxed verifier.

Native successful-operation speed samples in the same reports: Firefox Double Shift n200 p50 29.97ms/p95 174.77ms, Chrome n185 p50 30.33ms/p95 173.62ms, Qt n60 p50 28.16ms/p95 138.87ms. Qt focus_return n2 p50 153.61ms/p95 161.93ms. Denominators include successful measured operations only; failed/refused operations remain failures, and these figures are neither all432 latency nor proof of acceleration. Initial browser-package metadata preparation aborted before upload/input because of a stale880 counter assertion (the expected count was882); the aborted artifact was retained, corrected once, and both final protocol controls passed before one physical pass per component. No failed physical cell was retried to obtain PASS.


TD159 IBus Reset delivery-cache controlled RED at2026-10-19T09:47:26.776988+00:00:9PASS3FAIL; original eight upstream cases pass, failures exactly9/10/12 prove equal actual tuple suppressed after Reset, stale delivery cache survives repeated Reset, and invalidation is needed per engine. Actual macro-generated Reset body executes; no second reset implementation. Receipt /home/ubu/.cache/lay/development/browser-final-fixes-20261018/ibus-reset-surrounding-red-source-one/RESULT.json. Qt RichTextEdit actual read queues then cancels after about711ms; proxy-cache source defect is proved, physical causality still a hypothesis. New explicit decision 2026-10-19-ibus-reset-surrounding-delivery-cache.json limits change to cache invalidation before existing Reset, with no cached replay or Lay source authority. Next final-control RED/lifetime then GREEN/private isolated dependency build, unchanged Lay a46dd831 bytes and original physical protocol. No installation/publication or delivery-route change.


TD159 IBus Reset cache RED/GREEN and private dependency build at2026-10-19T09:50:28.580211+00:00: baseline final controls9PASS3FAIL(cases9/10/12) and actual GObject lifetime FAIL as expected; candidate12PASS0FAIL and actual GObject lifetime PASS under fatal GLib warnings. Extracted actual Reset body retires only this proxy's delivery-cache object, forwards the existing Reset, and never publishes cached text. Source SHA baseline2958803ac0d121c82be85487578569b713cf5d4d05aa4b4c28019861b8ae9c57 -> candidatec6a87f20baecc9e20a8e204ec444f47c4ff0d1d8df69629394eda150dcbecde4. New private ibus-daemon SHA0b907f7c969c6621c667a613ef0c5ad0b4cb0ecc74a7e62caeef775afab98af7,662528bytes; only prefix/bin/ibus-daemon changed, panel remains46a335becaa4ed35fc217adebd2fd206ea5dda4af8abd1a06f60136dac7e3a23. Receipt /home/ubu/.cache/lay/development/browser-final-fixes-20261018/ibus-reset-surrounding-build-request-source-one/BUILD.json. Source PASS is not physical acceptance: Qt RichTextEdit causality remains a hypothesis, latest complete432 remains380PASS48FAIL4BLOCKED. Next unchanged Lay a46dd831 and native112 identities/cadences/deadlines with only this new private IBus dependency, then matching browser320. No installation/publication, cached snapshot authority, delivery-route change or host IBus restart.


TD159 Reset-cache original native112 physically complete at2026-10-19T09:58:18.948843+00:00: unchanged a46dd831 IME plus private IBus0b907f7c gives90PASS18FAIL4BLOCKED/112 versus same-IME previous89/19/4. Only changed identity Qt5 RichTextEdit focus_return FAIL->PASS; actual committed native text changes привет->ghbdtn, final caret6/no selection/no preedit. All original112 identities and eight phase-normalized actor/observer protocol scripts match. Independent closure checks pinned processes/cgroup gone, no held keys, owned seat/policy removed, VT1 restored, host libraries unchanged; loaded private IBus build/hash explicitly bound. Export4de89305b8205baf95290c477e3108728af3d271df0127abb2004a900c7eba09,8500318bytes,1227files,448raw snapshots127403307bytes; archive and every member verified. Receipt /home/ubu/.cache/lay/development/browser-final-fixes-20261018/ibus-reset-surrounding-native-source-one/COMPARISON.json. Scope: one native112 pass, source-proved reset-cache defect and observed single-field recovery. Passing RichTextEdit IME endpoints remain unchanged with producer/interval completeness UNKNOWN; no inferred callback sequence or absence. Qt successful measured DS n60 p50=28.88ms/p95=137.89ms; focus-return n3 p50=141.06ms/p95=141.44ms, newly recovered RichTextEdit137.72ms; not all-case latency or acceleration proof. Browser320 with same dependency pending, so latest complete matched432 remains previous380/48/4 and must not be mixed with this native112. No installation/publication or delivery-route/source-authority change.


TD159 IBus Reset-cache full matching physical432 completed and independently
compared at2026-10-19T10:08:39.449724+00:00:382PASS46FAIL4BLOCKED/432 versus
same-Lay previous380/48/4. Firefox150PASS10FAIL/160; Chrome142PASS18FAIL/160;
browser292PASS28FAIL/320; native90PASS18FAIL4BLOCKED/112. Exactly two changed
identities, both FAIL->PASS: Qt5 RichTextEdit focus_return and Chrome field2
moved_boundary. Remaining430 statuses are unchanged. Chrome2 actual committed
text is now "должны быть " caret12/no selection, replacing the prior duplicate
"должн ыбыть должны быть " caret24. This observed recovery does not establish
the old client's precise DeleteSurroundingText processing sequence.

Only private IBus daemon0b907f7c969c6621c667a613ef0c5ad0b4cb0ecc74a7e62caeef775afab98af7
changed; Lay IME remains a46dd831f94f29d244671f64096ebe1a0e04ae2663fca442f9137814fa547adc.
The derived join verifies complete Lay/IBus/KWin/KWayland dependency bundles
match between browser320 and native112, and between their previous baselines.
All original432 identities and eight phase-normalized input/observer scripts
are unchanged. Independent closures verify no held keys, stopped owned seat,
empty cgroups, pinned processes gone, removed temporary policy and unchanged
host libraries. Browser export SHA256
37fceb052de575197fc2de14c11ff3c95fe3b7696b8ca8b0112611acb48e6a3a,
28340016bytes3294files1280raw snapshots448631347bytes. Archive and every member
are verified. Native export remains
4de89305b8205baf95290c477e3108728af3d271df0127abb2004a900c7eba09.

Successful native Double Shift samples: Firefox n200 p50=30.091ms/p95=176.217ms;
Chrome n185 p50=30.075ms/p95=174.198ms. Successful Space n50 per browser:
Firefox29.257/32.028ms and Chrome29.645/31.691ms p50/p95. Failed operations
remain FAIL and are excluded from successful latency samples; no acceleration
or all-case-latency claim. Qt successful DS n60 p50=28.877ms/p95=137.891ms.

Exact join receipt /home/ubu/.cache/lay/development/browser-final-fixes-20261018/ibus-reset-surrounding-matching-432-source-one.json; SHA256 32c67640fd0d27a9d80cc9d5a4c410729f48cbebc839c83cf39d18bd0287807d.
Component comparisons are ibus-reset-surrounding-browser-source-one/COMPARISON.json
and ibus-reset-surrounding-native-source-one/COMPARISON.json under the same
development-cache root. Verdict PHYSICAL_IMPROVEMENT_NOT_ACCEPTED: remaining
FAIL/BLOCKED and private KWin qualification gaps, GNOME NOT_TESTED. Passing Qt
trace producer/interval completeness remains UNKNOWN; no inferred missing
events, callback causality or focus theft. No installation, publication,
host IBus restart, cached snapshot authority or IME delivery-route change.
Next investigate the largest shared remaining class: selected-text conversion
(26 explicit selection FAIL identities; GTK Entry focus-return additionally
ends with a selected range). Preserve first-failed-transition evidence before
editing and keep selection authority distinct from collapsed-word authority.


Reset-cache remote AST-only architecture gate PASS at2026-10-19T10:11:15.933426+00:00; original scripts/update-architecture-graph.sh (including graphify update) returned0. All six generated outputs and export archive were hash-verified before copying back; runtime sources unchanged except generated graph receipt. Exact receipt /home/ubu/.cache/lay/development/browser-final-fixes-20261018/ibus-reset-surrounding-architecture-source-one/RESULT.json; SHA256 983d275226f6fe084ab4cb0c594eb02a5a32761a5ba97fb734003ffc39c8fbca. This source/document gate does not expand physical acceptance or certify a new build. Latest matching432 remains382PASS46FAIL4BLOCKED. Next selected-range diagnostics show positive IME RPC context_authority refusal with manual_toggle_allowed=false/manual_refresh_allowed=false in Qt, Firefox and Chrome; first source admission condition remains to be identified. No installation or delivery-route change.


TD159 Chrome Search four-cell passive text-reader diagnosis, independently
closed at2026-10-19T10:35:56.651714+00:00. Firefox2 roundtrip/short PASS;
Chrome2 roundtrip/short FAIL before Double Shift. Same a46dd831 Lay plus
0b907f7c private IBus and the complete kbel dependency bundle; no runtime or
input-route change. Remote60 canonical source controls and4 actual observer,
error/bounds and entrypoint-wiring tests PASS. Canonical320 actor/controller
and their guards are retained; adjacent four-cell diagnostic wraps state(),
returns the original object unchanged and adds bounded sequential reads only.

Chrome CharacterCount5/1 and caret6/2; count-bounded, count-plus-one and
end-minus-one all return "приве"/"д". Firefox equivalents are full6/2.
This falsifies the narrow hypothesis that merely extending the GetText read
boundary repairs the observation. Positive Chrome Wayland client log entries
contain complete preedit_string("привет",12,12) and ("да",4,4), so full
composition text reaches that endpoint. DOM/rendered equality and precise
Chrome accessibility/composition transition remain UNKNOWN. Next compare the
actual DOM value and current composition without changing Lay delivery.

The preceding kben partial320 attempt was stopped after observing that the
probe had been added to a helper not executed by the actual actor. That run
contains no useful probe samples and is not promoted or silently retried as a
matrix. Independent aborted closure verifies empty cgroup, no held keys,
owned seat/policy removed, VT1 and unchanged host libraries. Its export hash
is a5bdc9fc98c8a11cc08a170bf87fce894d4128e0e80f609464fb850db3cecab8.

Four-cell exact receipt /home/ubu/.cache/lay/development/browser-final-fixes-20261018/search-text-reader-four-cells-source-one/DIAGNOSIS.json,
SHA256 d9da90461c1a36c0accf374fdbf119d06f4c11883943bcc1749385b622b97a8c.
Archive75f7cf84300a6f05ec0af8a179c6e0f7fe72103f71dc5feb9e18e8e72b5c0e62,
345455bytes138files16raw snapshots1940780bytes, every member verified.
Independent closure includes pinned processes gone, no held keys, stopped
owned seat, VT1 restored and host libraries unchanged. Scope4cells/428NOT_TESTED;
latest matching complete432 stays382PASS46FAIL4BLOCKED, browser292/28/0/320.
No Lay build, installation, publication, host IBus restart or authority change.


TD159 Search DOM/AX separation, closed2026-10-19T10:53:09.429600+00:00.
The adjacent kbes four-cell diagnostic is COMPLETE_DIAGNOSTIC_ONLY: Chrome
Search roundtrip/short FAIL before Double Shift; ordinary input roundtrip/short
PASS including both native Double Shift directions, decoder probes and
Backspace restoration. Scope4cells/428NOT_TESTED; canonical full320 and
matching432 receipts remain unchanged at292/28/0 and382/46/4 respectively.

For Search, AX gives "приве" caret6 and "д" caret2, while read-only post-verdict
DOM gives full "привет"/"да", collapsed end caret, expected active field and
focused document. Ordinary input agrees across AX and DOM. Reads are sequential
with approximately45-55ms between AX and DOM start; this is not an atomic
snapshot or proof of a precise Chromium internal defect. Previous full-text
protocol evidence plus DOM argues against compensating with extra Lay output.
Search Double Shift remains NOT REACHED, not PASS. Visual pixels are NOT_TESTED.

The read-only DOM route pins actual browser PID/start/profile, loopback port
ownership, unchanged browser WebSocket, the single exact fixture page and tool
SHA256; no synthetic input or DOM grading authority. Remote60 canonical source
controls plus5 actual wrapper/entrypoint/redirect controls PASS; controlled-red
failures remain preserved. Preceding kbep/kbeq/kber attempts aborted at zero-key
readiness (redirect identity, private tool visibility, socket path length).
Their independent closed receipts remain separate; no failed physical cell
was retried inside those attempts. kbes is independently closed: no held keys,
empty cgroup, pinned processes gone, owned seat/policy removed, VT1 restored,
host libraries and all Lay/private IBus bytes unchanged.

Exact receipt /home/ubu/.cache/lay/development/browser-final-fixes-20261018/search-dom-reader-four-cells-source-five/DIAGNOSIS.json;
SHA25628c561c4508c8735721026f64d9ac18c8fe85beaa0f86ac0f37c3c68373ad44a.
Verified archive1a9a2f76e80ac8439a92570ee6a752de4cb830dd69c46a426dd9d08c25c4be41,
315159bytes136files16raw snapshots1425297bytes. Native receipt SHA256
cceb29c2a4b916c18c7c5779d9f92d4790516f621fa3294145094700fe2b3b67.

Next qualify a separate shared client-text observer with explicit architectural
decision and negative controls. Preserve whole-field, caret, selection, focus,
layout and engine checks; native physical input remains mandatory. Old FAILs
cannot be renamed by a post-verdict DOM sample. No runtime authority, delivery
route, local installation, host IBus restart or publication change.


### TD159 fixed client-text observer qualification, closed2026-10-19T11:15:36.909223+00:00.

Decision: docs/architecture/decisions/2026-10-19-native-client-text-observer.json.
The test-only shared native_dom_text_state adapter validates exact page/field/type,
active document, unmodified whole text, bounded UTF-16 selection and caret.
Browser PID/start/profile/listener/WebSocket and page are pinned around reads.
The DOM transport is fixed before physical input; failed DOM data cannot be
rescued by AT-SPI. Original native keyboard, focus, layout, engine, decoder and
Backspace checks remain. Raw AT-SPI is retained without verdict authority.

Remote source proof: 60 canonical + 11 actual wrapper/entrypoint/closed-predecessor
controls + 21 pure helper tests = 92 PASS. Removing the strict focus check causes
three expected negative-control failures, zero test errors. The initial unit
wrapper miscounted those as two and failed; its logs are preserved. The first
native preparation kbet expected an aborted predecessor status although its
pinned predecessor completed: zero-key abort, runtime root absent, seat inactive.
The corrected exact CLOSED_DIAGNOSTIC_ONLY guard has wrong-status/VT/closure
negative controls. Neither setup failure is presented as native acceptance.

Separate kbeu native Chrome qualification: 4 PASS / 0 FAIL / 0 BLOCKED /
428 NOT_TESTED of the fixed 432 scope. Ordinary input and Search each pass
roundtrip and short plans. Search physically converts привет -> ghbdtn -> привет
and да -> lf -> да. Eight real Double Shift gestures; after each, the expected
next-layout letter and its Backspace removal pass. Original Lay/private IBus/
panel/KWin bytes remain unchanged. This resolves the pre-gesture reader obstacle
for these four cases, not every Search plan or a precise Chromium implementation
defect: DOM and AX reads remain sequential. No text is appended by runtime.

Successful-only observation latency n=8/8: p50 107.807162 ms,
p95/max 121.499741 ms. Clock: evdev write return to exact
DOM target; kernel release not measured. Includes AT-SPI/DOM observer overhead;
not comparable to older AT-SPI timings and not a runtime acceleration claim.

Receipt /home/ubu/.cache/lay/development/browser-final-fixes-20261018/client-text-observer-four-cells-source-two/DIAGNOSIS.json;
SHA256 7a041a5935478d5834dfcea0c9a91a339f960513443460ea4103c21d67aca17c.
Native report SHA256 d6469c2f332423a7b5e2c53e39a5dac3b31c21025db25e0de178c84ba6c8fd73.
Verified archive 019341e72d268289a28d306bd1abbaaabc50ab8d3e48dcec3442129467181ed2, 366163 bytes, 138 files,
16 raw snapshots / 1862625 bytes.
Independent closure 439c6ecc4fbe790aaf81a40a676ccf814116ecd9689e35635c813c1619eac90b: empty cgroup, pinned
processes gone, no held keys, seat/policy removed, VT1 restored, host libraries
unchanged. Observer helper SHA256 e5ba1232ea26fd5971a6122044bfda169227a56acad8059dd8882a1892e2a3c9.

Latest complete browser matrix remains 292 PASS / 28 FAIL / 0 BLOCKED / 320;
matching full432 remains 382 PASS / 46 FAIL / 4 BLOCKED. Do not splice these
four diagnostic PASS into either receipt. Remaining plans, other fields/modes,
Firefox DOM transport, visual pixels and GNOME remain NOT_TESTED here.
Next: use a separately versioned preselected observer in a full affected browser
run after source/architecture gates. Retain native-only fields on their qualified
observer and keep original FAIL/BLOCKED receipts. No runtime authority, delivery
route, build, installation, host IBus restart or publication change.


TD159 full browser observer qualification, closed2026-10-19T11:29:26.292355+00:00.
Source: c8b244ef609ad9c5610b3e441311342abf17e9c5 plus the pinned dirty overlay.
New observer version kbev selects pinned read-only DOM before input for Chrome
text/Search fields 1/2; all other fields retain their original AT-SPI reader.
No reader fallback. Actual complete selected ranges are read before the gesture;
all 20 selection cells have positive full-range evidence, not a boolean alone.

Same original 320 identities, 96 source controls PASS, strict-focus mutation
caught by three expected failures. Full native browser result: Firefox
150 PASS / 10 FAIL / 0 BLOCKED / 160; Chrome150 PASS / 10 FAIL / 0 BLOCKED / 160;
total300 PASS / 20 FAIL / 0 BLOCKED / 320. Eight Chrome Search FAIL->PASS, all
other312 verdicts unchanged; every remaining FAIL is selected-range conversion.
This is corrected test observation plus new physical acceptance for those cells,
not a Lay runtime fix. Earlier FAIL receipts are retained unchanged. Native112
was not rerun in this observer experiment; its earlier separate90/18/4 receipt
is not renamed as a new432 run. GNOME, Firefox DOM transport, other DOM field
types/modes, visual pixels and production installation remain NOT_TESTED here.

Successful-only evdev-write-return to exact-target latency includes observer
cost. Firefox original AT-SPI Double Shift n200 p50=30.210774 p95=177.304895ms;
Chrome original AT-SPI n160 p50=29.899061 p95=173.248793ms; Chrome pinned DOM
n40 p50=108.394723 p95=223.535234ms. Reader groups are not comparable and no
runtime speedup/slowdown is established. Full per-operation counts/quantiles
remain in the receipt; failures and missing samples are not successful latency.

Exact receipt /home/ubu/.cache/lay/development/browser-final-fixes-20261018/native-client-observer-browser-source-one/DIAGNOSIS.json;
SHA256 0f9c19b93047012124b353e8380d354b145c02388db6db44f652b021d9fcbe8f.
Archive SHA2562c29957ba2749fccf3c7e857c076f7cef0089f2223be2f657049234183d90e80,
28540063 bytes,3295 verified members,1280 raw snapshots/448946189 bytes.
Independent closure: empty cgroup, pinned processes gone, no held keys,
owned seat/policy removed, VT1 restored, host libraries unchanged. Lay IME
SHA256a46dd831f94f29d244671f64096ebe1a0e04ae2663fca442f9137814fa547adc
and private IBus/panel/KWin bytes unchanged. Source architecture gate separately
PASS2026-10-19T11:18:45.086539+00:00 at
/home/ubu/.cache/lay/development/browser-final-fixes-20261018/native-client-observer-architecture-source-one/RESULT.json.
No runtime authority, delivery-route, installation, restart or publication change.

Adjacent KWrite causal analysis uses the existing immutable native112 receipt,
not extra input: full12-ime-trace.jsonl-after.bin SHA256
4bad5baa39bc383433fe472d6285389e05fba8c3240b39a20862409b50aa5587
contains the prepared eligible input-gate candidate at generation5/tail_epoch1698
but final_decision_present=false and Space no_apply stage=verifier, followed
by ordinary Space commit. Offset delta continuity is UNKNOWN/PREFIX_CHANGED;
its partial79-byte delta cannot prove absence of IME activity. Exact packet:
/home/ubu/.cache/lay/development/browser-final-fixes-20261018/kwrite-moved-boundary-causal-source-one.json.
The generic verifier label covers post-gate adapter exits; precise rejecting
condition and pair-scope provenance are not yet proven. Next reproduce that
first failed contract in existing owners; do not weaken edit validation.

TD159 KWrite owned Space pair, source reproduction2026-10-19T11:44:13.139026+00:00.
The first controlled regression passes through the existing actual SurroundingText
ingress, not a synthetic revision mutation. Before the unchanged prefix receipt
the existing owned pair scope is current; afterwards it is absent. The extended
fixed four identities record1 PASS /3 FAIL on the old runtime source: both
capability variants and native prefix/preedit-per-key ordering fail, while the
interrupted-proof refusal control passes. Receipt:
/home/ubu/.cache/lay/development/browser-final-fixes-20261018/kwrite-pair-prefix-repro-source-three/RESULT.json.
The earlier first test attempt failed compilation due to a private diagnostic
field access; it is not a semantic red. It remains recorded in source-one.
Source-two independently records0 PASS /2 FAIL for the smaller reproduction.

Decision docs/architecture/decisions/2026-10-19-owned-space-pair-actual-rereceipt.json
selects existing owned-Space proof carry across an adjacent actual exact prefix
or owned-preedit projection. No stale/missing proof reconstruction, new owner,
client-name branch, RPC, wait, timeout, delivery route or verifier bypass.
Existing Space commit continuation still waits for actual post-commit text
before deleting the exact pair. Implementation/source-green and physical
candidate checks are pending; native failure is not closed. Browser300/20/0
remains the earlier independently closed full320 receipt, not candidate proof.

TD159 KWrite source focused green2026-10-19T11:46:51.735907+00:00.
Same four fixed identities now4 PASS /0 FAIL. Both capability variants retain
the exact owned pair through alternating actual prefix/preedit receipts;
per-key KWrite ordering reaches one paired delete and one correction only
after actual client Space-commit observation. Twelve interrupted-proof
variants per capability reject and cannot revive on later equal text.
Receipt /home/ubu/.cache/lay/development/browser-final-fixes-20261018/kwrite-pair-prefix-green-source-one/RESULT.json.
Full hermetic IME/performance, refreshed graph/build and physical native checks
remain pending. No installed runtime or output-route change.

TD159 KWrite full source and private build2026-10-19T11:55:15.439987+00:00.
886/886 official correctness/package IME identities PASS, including347 isolated
process tests; three performance identities PASS. Remote format, refreshed
architecture graph and embedded receipt check PASS. Private binary
ff544e707102459d6af7d43e26d3e57c6294b2e0134402d9877071bacebcf8dc (8474432 bytes).
Receipt /home/ubu/.cache/lay/development/browser-final-fixes-20261018/kwrite-pair-prefix-final-build-source-one/RESULT.json.
Native NOT_TESTED. Frozen build source retains the graph-input document versions;
this later evidence metadata is not an additional runtime source change.
Next fixed original native112, then browser320, same targets/cadence/deadlines.
Runtime authority changed only by exact owned-Space proof carry; delivery routes,
structural verifier and installed runtime unchanged. No publication or restart.

TD159 KWrite fixed native112, closed2026-10-19T12:00:09.731702+00:00.
Pinned IME ff544e707102459d6af7d43e26d3e57c6294b2e0134402d9877071bacebcf8dc;
original112 identities, targets, input cadence, readers and deadlines retained.
GTK29PASS3FAIL0BLOCKED; Qt45PASS3FAIL0BLOCKED; KWrite14PASS1FAIL1BLOCKED;
Konsole3PASS10FAIL3BLOCKED; total91PASS17FAIL4BLOCKED. Exact comparison with
the prior native112 changes only KWrite/moved_boundary from FAIL to PASS;
the other111 verdicts are unchanged. Actual KWrite client reaches
"должны быть " after the original "должн ыбыть " seed. Observed Space latency
30.692999ms is one successful sample, evdev-write-return to exact client target;
it includes observation overhead and does not establish kernel-release latency.
Source tests separately prove one paired correction; the physical KWrite IME
before/after endpoint bytes are equal, so they do not prove an output count or
absence of IME activity. No inference from a bounded/rotating trace's silence.

Native protocol controls76PASS. Initial package source-one controls failed
before any input because copied IME-BUILD-RESULT/AUTHORITY-DECISION fixtures
described the prior build. Preserve that failure; source-two pins the actual
new frozen build/decision and passes the same unchanged assertions. Browser
handoff names the actual independent closure source-three, not an invented
source-one alias. Its hash is
ad64a52b68a394fb170ddbe3615a66d325b1f7724d26cc43a4e97d0085834147.
Empty cgroup, pinned processes gone, no held keys, VT1 restored, own seat and
temporary policy removed, host libraries unchanged independently confirmed.
All1227 exported files verified; archive SHA256
fd3fd9db976785b289c2f9ade8c77c40b874d3e8dedd70d9ec94adc339e6380c,
8762486 bytes,448 raw snapshots/126949255 bytes.
Exact comparison receipt:
/home/ubu/.cache/lay/development/browser-final-fixes-20261018/kwrite-pair-prefix-native-comparison-source-one.json.
Closure/native receipts and raw observations:
/home/ubu/.cache/lay/development/browser-final-fixes-20261018/kwrite-pair-prefix-native-source-two/evidence/.

Browser kbex on the same changed IME is pending; old300PASS20FAIL0BLOCKED/320
is historical and must not be combined into a new432 before the candidate
browser receipt closes. GNOME remains NOT_TESTED. Six native selected-range
FAIL, one GTK focus-return FAIL (also selected), ten Konsole FAIL and four
explicit BLOCKED remain. No installation, output-route or verifier change.

Next Konsole causal probe uses positive existing trace records: new owner163,
engine/149, capabilities41/surrounding supported; actual snapshot11 chars,
caret11/anchor0; each of six accepted printable commits remains unknown_start;
Double Shift RPC has bridge_token_live=true but context_authority refusal.
Raw PTY application observes an empty seed before typing and "привет" afterwards.
The11-character surrounding snapshot is not yet proven to be the PTY command
buffer or a fresh observation of this client. Distinguish terminal display
semantics from stale cross-client cached delivery before proposing an authority
change. Source: native-windows-7abiw7dn/1-ime-trace.jsonl-after.bin, SHA256
9c7dfbbeafa40d61c4d91d28e7067619c39786f766def6f3fbed7a586c60cdec.
PREFIX_CHANGED/unknown interval continuity does not prove missing callbacks.

### TD159 changed pair candidate: closed432 and Firefox coordinate refusal — 2026-10-19

Browser phase kbex independently closed2026-10-19T12:13:38.046328+00:00.
Same private IME ff544e707102459d6af7d43e26d3e57c6294b2e0134402d9877071bacebcf8dc.
Firefox149PASS11FAIL0BLOCKED/160; Chrome150PASS10FAIL0BLOCKED/160;
browser299PASS21FAIL0BLOCKED/320. Exact identity comparison with kbev changes
only Firefox/6/moved_boundary PASS->FAIL;319 others unchanged. Combine only
with the independently closed kbew native112 on the same dependency pins:
390PASS38FAIL4BLOCKED/432. One KWrite gain is offset by one Firefox loss;
candidate NOT_PROMOTED. GNOME NOT_TESTED; upstream KWin qualification failure
retained. No installation or delivery-route/verifier changes.

Comparison receipt:
/home/ubu/.cache/lay/development/browser-final-fixes-20261018/kwrite-pair-prefix-browser-comparison-source-one.json.
Archive SHA2566e494153b84cf50257de5e1b7fbaa8000e774f346a308d71e9a2e5058d8f0c4f;
28542574 bytes;3295 files hash-verified;1280 raw snapshots/449001422 bytes.
Actual independent closure is INDEPENDENT-CLOSED-source-one.json, SHA256
07ba42e71d30547bc09d0a66556fc53d35d668b2ea5f4608aede6af36030a736.
Empty cgroup, pinned processes gone, no held keys, VT1 restored, owned seat
and temporary policy removed; host IBus/KWin libraries unchanged.

Positive causal facts: Firefox engine/120 authorizes the same pair and sends
CommitText "ыбыть ". Private bus monitor logs client :1.8 InputContext_2
SetSurroundingText at1792411525.976481, serial6812: full "должн ыбыть "
with cursor11/anchor11. Forwarded engine message serial5164 preserves the
same tuple. This rules out a Lay-side cursor conversion as its origin.
The subsequent exact-client observer sees cursor12 and unchanged source text;
the continuation has already refused the inconsistent coordinates.
The old successful case's direct endpoint trace was delayed. Its actual
sequence is positively retained in native-windows-unw501lx/
78-ime-trace.jsonl-before.bin, events1760..1782, callback clock
606575.986018406, engine/120: post-commit snapshot12/cursor12/anchor12,
followed by the paired replacement. Do not attribute the endpoint's earlier
no-apply events to this Space action. Current failed case remains FAIL;
old/new event-order causation is still a hypothesis, not a proved code cause.

Next inspect GTK retrieve-surrounding and commit callback ordering. Retaining
arbitrary caret contradictions is rejected: existing caret/selection negative
controls require revocation and no later resurrection. No synthesized cursor,
blind suffix replacement, client-specific bypass or deadline extension.


### TD159 exact Firefox GTK source and bounded ordering diagnostic — 2026-10-19

Preflight recorded at 2026-10-19T12:30:35Z. Exact installed Firefox 157.0.1
source stamp is `737ec77d798755890b062aab541a90d6c75afd19`; official
`widget/gtk/IMContextWrapper.cpp` was fetched from Mozilla release Mercurial
raw-file endpoint, SHA-256
`8d3cb99b88aa684c23f11806589e23ee2a46009b3d47a168a39c8570d372717a`.
The source uses cached content selection for the cursor, queries paragraph text
separately, and retries retrieval on a selection notification only when the
previous paragraph query failed. A valid-but-old offset remains a hypothesis
until a native Gecko log establishes the actual notification order.

The sole diagnostic attempt uses the same pinned ff544e70 IME and private
dependencies with stock Gecko IMEHandler logging in Firefox textarea field6,
all original sixteen functional plans, and a read-only actual GTK/IBus module
map/hash capture. It changes neither canonical320 selection nor targets/keys.
Full preflight and consequence analysis: `/home/ubu/.cache/lay/development/browser-final-fixes-20261018/firefox-gtk-ordering-diagnostic-source-one/DIAGNOSTIC-PREFLIGHT.json`.
No new owner, runtime query, timer, cache, deadline, candidate/ranking/learning,
output, verifier, package or consumer change is authorized by this diagnostic.
Logging can perturb scheduling and I/O; its latency and any successful target
are diagnostic only. Run once under the existing remote resource/seat guards,
close independently, preserve the failed complete299/21 browser receipt.
First unresolved mechanism: whether fulltext12/caret11 is returned before
Gecko updates its selection cache, or originates from composition state instead.
Actual loaded GTK module identity is not inferred from private IBus daemon version.

### TD159 bounded Gecko ordering diagnostic closed — 2026-10-19

The single diagnostic attempt completed sixteen original Firefox/6 plans:
15 PASS/1 FAIL, with `moved_boundary` PASS and the known selected-range FAIL.
This is **diagnostic only**, not a replacement for the closed299/21 browser
receipt or a correction of its failed identity. Synchronous logging perturbs
scheduling; no production speed or repair claim follows from this run.
Private bytes remain ff544e70 and no runtime authority or delivery route changed.

Exact local evidence directory:
`/home/ubu/.cache/lay/development/browser-final-fixes-20261018/firefox-gtk-ordering-diagnostic-source-one/evidence`.
Native receipt `native-windows-ozp6a0di/RESULT.json`; raw Gecko log
`logs/gecko-ime.log.moz_log`. Original session closure is recorded at
2026-10-19T12:32:15.505743+00:00; independent closure/export verification
`INDEPENDENT-CLOSED-source-one.json` at2026-10-19T12:32:52.182390+00:00:
cgroup empty, pinned processes gone, no held keys, owned seat/policy retired,
VT1 restored. Archive SHA256
ded5172ba423ea1240fc8d4d2cea0d4ddb1acfc4241a841e46ba1303292ba4ac,
1283827 bytes,257 files;64 raw snapshots13999572 bytes. Export checks preserve
the canonical320 identities and explicitly reject matrix acceptance here.

Actual `/proc/<FirefoxPID>/maps` pins the GTK module to installed
ibus-gtk3 1.5.26-4,39112 bytes,
SHA256 dca62c7122c26c1ccb3cf76093a27db9baedb987d158e641641d903427c0da54;
the private libibus is1.5.34,1633080 bytes,
SHA256 ed477371ba213df88db980f4093747ed4553df6d2973b7bdbf36bb96ea8f9a31.
Upstream1.5.26 source is a scoped reference, not proof of Ubuntu's exact
compiled module. Both inspected upstream versions request surrounding text
immediately after their GTK commit signal; the old module lacks the new
RequireSurroundingText callback. Do not infer an extra fresh read from the
private daemon/library version or an engine request alone.

Measured successful moved-pair timeline, UTC on2026-10-19:
12:32:05.679740 empty preedit change; .680141 composition end;
.680267 CommitText("ыбыть ") with IsComposingOn=false;
.680321 GetCurrentParagraph cached/predicted cursor12;
.680345 text query still length11, so retrieval fails and arms Gecko's
existing pending retry. At .692059 actual selection notification12 arrives;
.692095 Gecko retries; .692153 full text12/cursor12 succeeds; Lay then
issues the authorized delete at .694673 and corrected commit at .694869.
This positively proves text/selection incoherence at this boundary and shows
why this particular direction recovers. The failing complete-matrix tuple
full12/cursor11 is in range, so Gecko's same range check cannot reject it.
The exact Gecko notification order of that failure remains **UNKNOWN**.
Neither a successful diagnostic nor this source explanation proves that the
preceding KWrite fix caused the Firefox regression.

### TD159 active-composition retirement-order experiment — 2026-10-19

Decision: [2026-10-19-active-composition-commit-retirement-order.json](decisions/2026-10-19-active-composition-commit-retirement-order.json).
Hypothesis: finishing the existing visible composition with CommitText before
retiring presentation removes the extra empty composition termination and
noncomposing insertion path measured above. This is not yet a physical fix.
Baseline remains private ff544e70,390/38/4 across matching432, NOT_PROMOTED.
First failed transition remains actual client full12/cursor11 -> exact pair
admission rejection. Caret/selection revocation, verifier and output routes
remain intact. No arbitrary delay, cursor synthesis or interrupted-proof revival.

Five semantic controls enter the real active-composition commit method:
commit-before-retirement for plain/Space, commit failure preserving typed
composition, postcommit preedit-clear failure without replay, postcommit
auxiliary-clear failure without replay, and unchanged atomic proposal effects.
Capture old-source red before moving output order. A successful CommitText
must retire its buffer even if the later presentation clear fails; otherwise
this experiment could duplicate a word on subsequent input. Keep the existing
clear retry for presentation only. Atomic speculative rollback and all prior
pair/owner/caret controls remain mandatory. No new owner, generation, cache,
timer, query, polling or retry; effect count and package dependencies unchanged.
Risk is generic GTK/Qt signal-order behavior, requiring the original affected
native scope and latency samples before any promotion. No local installation.


### TD159 selected-text scope deferred and commit-order source proof — 2026-10-19

The user explicitly deferred the 26 selected-text failures. They remain FAIL
in the preserved complete receipts and are excluded only from repair scope;
no counter or target was changed. Active repair scope is Firefox moved-pair1,
GTK focus-return1 and Konsole10; blocked4 remains separate. User acceptance
of the existing state authorizes no installation or publication here.

Commit-retirement experiment source red used exactly five production-entrypoint
controls:1 PASS/4 FAIL, receipt
`/home/ubu/.cache/lay/development/browser-final-fixes-20261018/active-commit-order-red-source-one/RESULT.json`
at2026-10-19T12:46:46.557629Z. After moving CommitText before clear and retiring
the committed buffer even after a presentation error, all five pass; receipt
`/home/ubu/.cache/lay/development/browser-final-fixes-20261018/active-commit-order-green-source-one/RESULT.json`
at2026-10-19T12:51:59.938490Z. Atomic proposal asserts exact committed text and
hidden presentation payload. No caret/selection or stale-proof rule changed.
Full IME closure, graph gates and native acceptance remain pending; focused
source evidence cannot establish the Firefox outcome.

### TD159 commit-retirement full source gate — 2026-10-19

At2026-10-19T13:01:43.814839Z the remote dedicated-20cpu guard completed all
891 correctness/package tests (894 discovered;347 isolated) and all three
performance controls without failure. Format, architecture graph and embedded
receipt tests pass. Focused formatted-source proof remains5/5. Private IME
SHA256eaefe2dda24b6bd2630685995da6b0996348579af13217574b07cf347f57434a,
8474432 bytes, is frozen at
`/home/e/projects/lay-development-runner/active-commit-order-final-build-source-one/bin/lay-ibus-engine`.
Exact build/source pins and logs:
`/home/ubu/.cache/lay/development/browser-final-fixes-20261018/active-commit-order-final-build-source-one/RESULT.json`.
Source result is SOURCE_GREEN_PRIVATE_IME_BUILT_NATIVE_PENDING; it grants no
physical-client or installation claim. Runtime authority, delivery routes,
selection/caret revocation, model packages and learning semantics unchanged.

Browser package source-one stopped before input:59/60 source controls pass,
one stale negative-control mutation sets authority_changed=false, which is now
the legitimate value. Preserve that failure in
`active-commit-order-browser-source-one/PROTOCOL-CONTROLS.log`. Source-two
inverts the negative mutation to true and restores false afterwards, retaining
the same mismatch rejection and binary/source-pin assertions. All60 controls,
15 observer-wrapper controls and21 helper controls pass; controlled-red and
readiness-red violations remain detected. No native targets/actions/deadlines
or expected grades changed. Browser kbfa is in progress; matching native112
will follow its independently verified closure. Until then retain the last
closed browser299/21/0 and matching KDE432390/38/4; selected-text repair is
deferred without regrading26 failures.

GTK focus_return examination of the previous closed112 identifies the first
failed conjunction before the gesture: row31 of
`kwrite-pair-prefix-native-source-two/evidence/native-windows-fbimsmp8/RESULT.json`
has correct committed text/caret/layout but actual selected range[0,6] after
focus returns. No Double Shift was sent in that row. GTK's documented default
gtk-entry-select-on-focus is true, but its effective value in that old process
was not measured. Do not claim a Lay gesture failure, erase selection, weaken
the target condition or retroactively regrade this receipt.

Konsole remains a separate unresolved client-contract investigation. Remote
package versions are konsole4:21.12.3-0ubuntu1 and Qt5.15.3+dfsg-2ubuntu0.2.
Positive old trace proves actual tuple11chars/caret11/anchor0 at current
owner163 and a context_authority refusal; the11-character line is consistent
with the fixture's display prompt, but provenance is not proved merely by that
consistency. Upstream Konsole v21.12.3 TerminalDisplay::inputMethodQuery returns
the display line/column and empty ImCurrentSelection, lacks ImAnchorPosition;
its inputMethodEvent consumes commitString but does not implement replacement
offset/length. Upstream Qt v5.15.3-lts-lgpl QIBusPlatformInputContext::update
sends ImAnchorPosition.toUInt directly, and deleteSurroundingText sends a
QInputMethodEvent with empty commitString and the replacement range.
Sources:
https://github.com/KDE/konsole/blob/v21.12.3/src/terminalDisplay/TerminalDisplay.cpp
and https://github.com/qt/qtbase/blob/v5.15.3-lts-lgpl/src/plugins/platforminputcontexts/ibus/qibusplatforminputcontext.cpp.
These are upstream source observations, not an independently instrumented
failure of this Ubuntu object's event handler. Ignoring selection or selecting
an erase transport from cursor geometry alone is rejected. A proposed fix must
first establish truthful client capability and executable replacement semantics
under the existing C09/WindowInteraction contract; no Konsole production edit
or route expansion is authorized by this diagnostic observation.


### TD159 accepted closed 432 and publication-only decision — 2026-10-19

Recorded at 2026-10-19T13:27:26.885956+00:00. The user accepted the result, requested the full
application table and publication, and explicitly chose to update later through
“Проверить обновления”. Repair work stops at this accepted candidate. No further
functional checks, builds, input experiments, installation or restarts are part
of this publication. Version 1.0.91 changes the accepted 1.0.90 source's release
metadata; the runtime implementation is the one already measured.

The same pinned IME/daemon/CLI/configuration and private IBus/KWin dependency
bundle cover both independently closed components: browser320 and native112.
The aggregate created at 2026-10-19T13:19:03.064954+00:00 records
391 PASS / 37 FAIL / 4 BLOCKED, total432, NOT_TESTED0 within that matrix.
Firefox150/10/0, Chrome150/10/0, GTK3 29/3/0, Qt5 45/3/0,
KWrite14/1/1 and Konsole3/10/3. This supersedes the pending component status in
the preceding entry. Firefox field6/moved_boundary alone changes FAIL -> PASS;
all other rows retain their grades. The 26 selected-text failures remain FAIL
and their repair is deferred. The other eleven failures are GTK focus-return1
and Konsole10; four blocked cases remain blocked. Publication acceptance does
not establish that these cases are repaired or that stock KDE is qualified.

The active-composition fix retains the existing authorized CommitText route,
commits before hiding presentation, and retires committed state even when a
later presentation-clear operation fails. Candidate IME SHA256 is
`eaefe2dda24b6bd2630685995da6b0996348579af13217574b07cf347f57434a`.
Runtime authority, delivery routes, models and learning semantics are unchanged.
The previous source gate completed891 correctness/package and3 performance
controls plus the five focused production-entrypoint controls; publication
reuses these results and does not run them again.

Exact aggregate receipt:
`/home/ubu/.cache/lay/development/browser-final-fixes-20261018/active-commit-order-432-comparison-source-one.json`.
Browser closure:
`active-commit-order-browser-source-two/evidence/INDEPENDENT-CLOSED-source-one.json`,
SHA256 `c8409a6e088021369e63be4c879cec6810e9ba7734b59e5cb6552153e664e215`.
Native closure:
`active-commit-order-native-source-two/evidence/INDEPENDENT-CLOSED-source-one.json`,
SHA256 `bcc73f8792332a6c5d83db61724174889ba249db14bfed2e16148d1da6109fca`.
Component paths are relative to that aggregate's cache directory. Both closures
independently establish scope cleanup, no held keys, restored VT1 and unchanged
host libraries; no user installation occurred. The original aggregate's
NOT_PROMOTED label is retained as historical evidence; subsequent user acceptance
and publication are recorded separately here and in the portable evidence index.

Successful observer-inclusive Double Shift latency: Firefox200 samples,
median29.916ms/p95 175.830ms/max184.136ms; Chrome200 samples,
median30.488ms/p95 180.291ms/max225.163ms. GTK40, Qt60 and KWrite20 successful
samples are preserved in the index. Konsole has no successful Double Shift
latency sample. The recovered Firefox moved-boundary Space operation measured
31.587ms. Failures remain in the acceptance denominator; no speedup is inferred.

The KDE dependency bundle remains private, and the normal Lay updater does not
install its IBus/KWin builds. Stock KDE, Plasma6 and matching fresh GNOME are not
qualified by this receipt; fresh GNOME remains NOT_TESTED. Keep unresolved
client-contract work separate from the accepted publication scope.

See [release1.0.91](../release-1.0.91.md) for the complete table and limitations,
and [portable evidence](../../tech_debt/evidence/td159-accepted-native-432-20261019.json)
for counts, exact pins, original receipt/closure hashes and latency samples.

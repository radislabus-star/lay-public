# Release105 verification accepted for tested Chrome scope — 2026-10-28T02:08:21.059088+00:00

V11 source3548/3548 and all18 final gatesPASS;10release artifacts resolved from compiler records, zero release warnings, exact remote installation and fixed native7/7PASS. Full unchangedChrome220 COMPLETE: original128PASS0FAIL32BLOCKED/160; additional52PASS0FAIL8BLOCKED/60; total180PASS0FAIL40BLOCKED, NOT_TESTED0. Normal/text/moved-boundary row9 isPASS. Runtimeengine1437885 SHA9e0731fd2014a0c9fc2948e56ab9a58f8d8298ce066d9db1f664fea8b538717c, source85cf6f9f9590ca5c3c942d67202e1bff9edc3fbe70208f0f2cdb871175400f9d. Space fixed7 {"n": 7, "median_ms": 100.63644195906818, "p95_ms": 147.8799961041659, "max_ms": 147.8799961041659}, DoubleShift pass rows {"n": 367, "median_ms": 48.37643774226308, "p95_ms": 62.78461706824601, "max_ms": 83.4248480387032}. RFB/X11 evidence, no USB/Wayland/current-Firefox acceptance claim.40BLOCKED retained, not PASS. Exact receipts and full observed baseline transitions in tech_debt/evidence/release-1.0.105-verification.json.

Runtime code is frozen and accepted for this measured scope. Next only commit/push, fresh CI, tag/stable release and actual installed updater checks; then106–111. No local GNOME runtime installation/restart/input. No repeating unchanged native acceptance during publication.

# Release105 read-coherence source GREEN83/83 — 2026-10-28T01:33:38.319450+00:00

V27 mechanically extracted the unchanged one-resample protocol and added controlled read tests:81/83 (all previous81PASS; both new tests fail at the skipped resample for native_document_changed). V28 minimal KDE-only read-policy correction83/83 PASS in29.26s. Same original80ms deadline, at most2 reads, second geometry=false; no retry for privacy/selection/admission/deadline errors or a second changed result; coordinator does not install snapshots or emit text. No common ownership/verifier/executor change in this correction. RawCROOT/browser-pair-reflow-unit-v28/result.json, sourcebc25cddcf5663c17dd8998fc28202fdf91ef4909d9c7c66ef40ac59c1fc1939b.

Old fullChrome baseline comparison is by identical220 case keys, not counts alone: V10 observes3FAIL→PASS,1FAIL→BLOCKED,1PASS→FAIL,2PASS→BLOCKED,2BLOCKED→PASS. Causes across snapshots not inferred. The sole actual FAIL is the coherent-source gap; blocked scenarios remain unavailable, not fixes. Next fresh3548 source/full gates/exact10artifacts/native7/fullChrome220. Public104; no105 publication.

# Release105 fullChrome220 reveals coherent-source gap — 2026-10-28T01:21:17.123331+00:00

Complete unchanged actorV10: original126PASS1FAIL33BLOCKED/160; additional53PASS0FAIL7BLOCKED/60; total179PASS1FAIL40BLOCKED/220, NOT_TESTED0. Native7/7 and delayed path remain accepted for that scope, but full acceptance/publication is blocked by row9 normal/text/moved-boundary. The Full ready decision is authorized821; actual source read failed native_document_changed817 (CharacterCount changed across GetText), then protected delete guard correctly refuses no_proven_delete_backend822. Preserve this guard. Existing read_native_document already performs one source-only resample for an old owned-prefix snapshot, but ? returns an incoherent read before that coordination. Select reuse of that same resample/deadline9/10; projected ready deferral7/10 and global waits3/10 rejected for larger scope.

New preflight only kde/native_document.rs, preparation.rs and tests.rs. First mechanical read-coordination extraction/current-behavior RED, then bounded correction. Full raw `/home/e/projects/lay-development-runner/kde-install-control-20261020/browser-pair-reflow-full-native-v10/result.json`; exact causal row9 trace in chrome/chrome/row-9-native-trace.jsonl. New BLOCKED rows are not fixes; compare separately with old baseline.

# Release105 native fixed cases GREEN7/7 — 2026-10-28T00:57:44.556966+00:00

V10 complete source3546/3546, all18 build gates,10 release artifacts, both toolchains and zero installed release warnings pass. Exact10artifacts installed; engine1190174 SHA942e5c464b2079210857e01809ebed872f9cb3bea269548b03ec43ddbf4c68aa. Native fixed cases7/7 PASS. First cold iframe exercises the actual delayed path: original Space commit401 precedes full worker ready414 (17.816ms), native exact-range replacement419 deletes12characters, boundary applied=true426; actual corrected field confirmed. This is physical RFB acceptance, not inferred from the unit test. Same original actor and inputs/deadlines. Space latency n7 median98.568ms p95155.271ms; includes observation polling. No Double Shift latency claim.

FullChrome220 is running, receipt `/home/e/projects/lay-development-runner/kde-install-control-20261020/browser-pair-reflow-full-native-v10/result.json`, session61812. No105 publication yet. Then classify new failures against old baseline, preserve BLOCKED and all denominators, commit/push/CI/release/updater; continue106–111. No local GNOME runtime changed.

# Release105 boundary-token continuity GREEN — 2026-10-28T00:32:56.524815+00:00

V26 controlled81/81 PASS (29.70s). The exact V25 canonical-boundary RED now passes across both completion orders, pre-Space echoes and settled lineage. Native negative checks run before/after boundary settlement, including another boundary and input-gap scope. New common token comparison is tested through real reducer key settlement and rejects changed connection, revocation, owner, activation, generation, completeness, suffix, floor and first-word metadata. It never publishes a token or changes reducer revalidation. Ordinary native is_current remains strict.

Raw `/home/e/projects/lay-development-runner/kde-install-control-20261020/browser-pair-reflow-unit-v26/result.json`; source3d2cede861b057407f342b4bc6035c6ad39362f47217a0570d9e3f9195e278ef. NativeV9 remains6/7 on installed old candidate, not reused as new acceptance. Next freeze inventory (+1 common token test), graph and full3546/18build gates, then exact artifact installation/native7 and Chrome220.

# Release105: canonical Space settlement root — 2026-10-28T00:26:38.282551+00:00

V9 full3545/3545 and all18 build/compatibility/architecture gates PASS; exact10artifacts installed in remoteKDE105, engine996696. Fixed native7=6PASS1FAIL, first iframe only; no release/publication. V24 actual pre-Space echo is harmless in the original fixture80/80. V25 controlled canonical word-boundary settlement gives79/80 RED: pending native receipt still contains the old word token, whereas ProcessKeyEvent settlement advances the current lineage after Space. Source: observation.rs::settle_context_key_callback/advance_context_word_scope, context_admission.rs::WordScope::close_at_observed_boundary; first refusal native_space_boundary_owner_is_current. Raw `/home/e/projects/lay-development-runner/kde-install-control-20261020/browser-pair-reflow-unit-v25-red/result.json`, native `browser-pair-reflow-native-v9/iframe-1/native-trace.jsonl` lines419–455.

Preflight expansion, under the autonomous series authorization: pure AdmissionToken successor comparison in context_admission.rs and its unit test, plus existing KDE native binding/preparation, continuation call and native tests. Existing reducer transitions/revalidate and GNOME source behavior are protected. Exact scope and safeguards in `../architecture/decisions/2026-10-28-kde-native-boundary-token-continuity.json`. New field relationships grant no edit by themselves; existing actual text verification and executor remain mandatory.

# Release 1.0.105: pending Space source regression — 2026-10-27

V21 RED: 52/54 native tests pass, both new controlled tests fail on retirement of the pending worker at NotReady. V22 compilation failed after a helper rename because two calls inside assert! macros retained the old name; corrected mechanically. V23 GREEN: 80/80 native + existing Space boundary tests, exact selected identities, no failures, 28.24s. Raw: `/home/e/projects/lay-development-runner/kde-install-control-20261020/browser-pair-reflow-unit-v23/result.json`. Source SHA213f5875b71f13e3884192a6cbb61d531fe024ca715056aaa4e89ccaed72937c.

The new tests establish retained work and cancellation at the existing executor boundary; they are not native edit acceptance. Runtime remains V8 until verified release artifacts are installed. Version labels now1.0.105, no model/learning/config change. Native seven-case and Chrome220, full source/release/CI and updater gates remain required before publication. No local GNOME install/restart/input.

# 2026-10-28 local — V18 causal capture/retention proof; native not yet qualified

V6all13gates/1106source checks/11performance/10release0warnings passed;
installed engine463975. Native seven-case result still5PASS2FAIL. Actual
iframe-1 trace289/290 has valid preparation and separator ACK; after owned
next letter(297), native_document_changed(302) and allowed same-owner gate
(303), the next actual read(314) reports missing. This is the first proven
retirement: native_observation_failed calls observe_document_snapshot(None).
Missing source truth must remove actual edit evidence, while same-owner
literal input can retain its existing inert append data. Recheck existing
observer ownership after the ordinary no-data observation; preserve no
actual snapshot, mutation lease, new identity or global permission.

V16RED51/52 separately proves missing zero-character ASCII boundary capture;
ordinary rearm passes last_tail_token, which skips trailing whitespace. V17
RED51/52 after one canonical native boundary call then exposes exact transient
loss. V18GREEN52/52 retains same witness through all four existing transient
read classes,7positive orderings; old51negative controls and all50native tests
remain. New checks explicitly assert snapshotNone, Unsupported edit route,
no pair-client snapshot, unchanged scope/token/epoch, exact actual recovery,
and rejection of a contradictory fresh field. No output emitted by these
observations, no edit guard loosened, no new owner, timer, queue or route.
Receipts CROOT/browser-pair-reflow-unit-{v16-red,v17-red,v18}/result.json.
V18 pre-graph source98a64a6847dcd80977739a6c7c24a88b46e05299d22640122ad0f7c0f5e7bde7;
fresh canonical source/build/native proof pending. Full320 table unchanged.

# V15 unitGREEN52/52; V14 compile diagnostic retained

The new install-only native receipt test is RED51/52 on V12, specifically
owned ASCII Space vs actual NBSP before first append. Calling the existing
closingSpace acknowledgement under the existing same-observation-owner guard
makes all52 pass, retaining all previous identities and51 negative variants.
Seven positive orderings cover gate-only and observer postconditions; the
subsequent owned-prefix resample controls remain in every positive variant.
New KDE read-only observation metadata includes canonical chain refusal
before reconciliation, current native binding, actual cursor and tail counts.
V14 compile failed E0624 because that query was private; V15 makes only its
visibility pub(crate), preserving the entire guard body and authority callers.
Compile failure receipt retained at CROOT/browser-pair-reflow-unit-v14/compile-failure.json.
No failed build was installed. V15receipt CROOT/browser-pair-reflow-unit-v15/result.json;
pre-graph source d1450bc727eb010b34869376ec9e4df0f0227159fde5da32eed28700d466d8a4.
Fresh source/build/native proof pending; installed native remains V5FAIL.

# V5 native5PASS2FAIL; distinguish two first-boundary receipt entrypoints

V12sourcea41332a3 passed fresh1106/1106,13gates/11performance,10release
artifacts0warnings. Installed engine353360/daemon353480/tray353489; V5
receipt paths under CROOT/browser-pair-reflow-{gates,deployment,native}-v5.
Five iframe repetitions3PASS2FAIL; rich/plain controls bothPASS. In the
successful iframe-1 trace the new existing resample was actually invoked.
The two failures instead lost pair preparation before finalSpace; no proof
of an overall error-rate improvement or a fix for every stale read.

Correction to the preliminary hypothesis: snapshot length6 DID arrive before
first next-word input in both failures. iframe-3 already logged closingSpace
ACK(line251); iframe-4 did not. Do not blame omitted source delivery or solve
both by broadening a witness constructor. The shared before-first-commit
rebind was inspected: a preceding closed word makes last_tail_token nonempty,
so its early return is relevant; no shared rebind/guard change is justified.

V13RED51/52 extends the existing pair test to native installation without
calling observer postcondition: an actual NBSP left local owned Space
unacknowledged. The minimal fix calls existing acknowledge_native_closing_space
under unchanged observation owner after actual snapshot publication. Existing
postcondition remains idempotent. Seven positive orderings now include gate-
only and observer settlement separately,51 negative variants remain. Record
canonical chain refusal in KDE separator observation before reconciliation,
using reason/counts only, to locate the other missing-preparation transition.
No new read loop, owner, common rebind/frame-builder or delivery change.
Full320 baseline remains200PASS58FAIL62BLOCKED; current seven-case subset
is5PASS2FAIL, not a full native acceptance or permission to publish.

# Browser matrix repair — active plan and diagnostic preflight

Status: ACTIVE, continue implementation/testing; a matrix report is not task
completion. User reiterated continuation after the complete440-row baseline.
Canonical checkout /home/ubu/projects/lay-dependency-updates-20261027, branch
codex/browser-matrix-repair-20261027, basec794d31d1b29c25d48c60d4dab554c5dfba88044.
Working directly, no subagents/Harness2. All compute and native input on the
authorized dedicated KDE worker; preserve local GNOME, client/desktop isolation
and existing IME delivery routes.

## Baseline and immediate first failed transition

Exact native source423f19ac, engine3596189/553680de..., full mainChrome125P4F31B
and Firefox75P54F31B,160each; extraChrome52P0F8B/Firefox54P0F6B,60each.
Complete receipts in browser-native-matrix-2026-10-27 report/evidence. They
remain the current complete totals until another full matched matrix.

First work item is Chrome normal rich/iframe-rich moved-boundary failure.
Candidate selection reaches apply, then existing opt-in trace reports
replacement_changes_uneditable_prefix. Source ime_correction.rs:372-405 leaves
two live explanations: missing editable pair, or rejected pair plan. A later
first-word guard is correctly refusing changes outside a verified span.
Do not infer missing scope from the absence of pair_not_exact: an exact pair
whose planner returnedNone also reaches that guard. Root cause not proven.

## Options before production edit

1. Extend the existing KDE-only opt-in refusal logger at the two missing
   branches, then run fixed Chrome field diagnostics.9/10, selected. No new
   endpoint, state owner, user-text log or timing/readiness change.
2. Add a new runtime-state DBus inspection method.3/10, rejected: larger public
   surface and maintenance/privacy burden; unnecessary for two boolean branches.
3. Add only native/DOM observation payloads in the frozen actor.5/10 for this
   defect: useful for8observation blocks but cannot expose internal pair-plan
   admission. Keep as the separate observer work item.

## Selected change and consequence analysis

Production file scope is src/ime_correction.rs only. Add existing logger calls
for pair_plan_rejected and pair_scope_missing. Existing callback, return values,
ranker, candidate lattice, authority, verifier and text edits remain untouched.
The helper already requires debug_action_log and LayoutBackend::Kde. GNOME
performs no added log serialization or bus call; KDE no-debug keeps its behavior.
No new function, cache, timer, queue, fallback, owner or route.

Candidate retention/ranking/false authority: no candidate/request/plan mutation;
new strings are diagnostic reasons only, never runtime matching conditions.
Latency/tails: at most two extra bounded reason-only records on a refused
projection; unchanged deadlines and no additional I/O/RPC/wait in input logic.
CPU/RSS/allocation: fixed logging cost only when existing KDE debug logger is
enabled, no retained allocation or expanded candidate field. Capture timings
separately; diagnostic performance is not claimed as an optimization.
Cache identity/invalidation: no cache changes. Changed source/binary identities
need fresh focused compile/test and exact loaded-byte verification.
Package/delta reload: unchanged; all material generation checks stay exact.
Learning/feedback: unchanged; native inverse/shared cleanup remains blocked.
Concurrency/stale results: no admission, scheduling or publication order change;
no new read that could acquire a lock, refresh a receipt or invent provenance.
Failure/rollback: before replacing the engine preserve existing installed bytes
and original IBus component. Never install over owned unfinished text. Restore
config/held keys and retain failed diagnostic attempts.
Consumer compatibility: no wire schemas/interfaces, enum values, helper APIs or
existing reason strings are removed; extra opt-in records are additive.
Maintenance/removal: two local logger calls beside their actual branch; no
parallel implementation. Remove after diagnosis only if unnecessary, otherwise
retain as bounded causal metadata under the existing logger.
Second-order risks: debug consumers must not interpret extra reason records as
extra edits. Failure trace count can increase by two but no positive outcome is
issued. Package/online learning changes cannot broaden logging into authority.

## Proof and continuation gates

AST inspect both exact insertion sites first. Verify the diff adds only the
existing logger calls/conditional and every old return/guard/action is retained.
Use official remote dev-check focused engine target, exact nonzero1104tests,
then existing development-preview build/guard workflow and compiler-artifact
records. Focused PASS is not release acceptance; preserve full3541baseline and
require final affected/full gates after actual functional repair.

Reuse the existing native actor/controller, fixed selected Chrome rich and
iframe-rich cases, no inverse, own empty fields, no assert/deadline/cadence
change. Diagnostic cases never overwrite full440counts. Exact current engine
hash, trace anchors, config restoration and no-held-key checks are mandatory.
Record the proven branch, inspect its original owner and write a functional
consequence preflight before any behavior repair. Prefer a KDE/client-owned
producer repair; never weaken the common verifier to compensate.

## Remaining ordered work

1. Chrome pair projection (2FAIL), current item.
2. Chrome rich Space wrong-layout stale identity (1FAIL), use existing frame/
   lease proofs and discriminate the actual mismatching component.
3. Chrome private email reverse toggle (1FAIL) and separate native/DOM gaps;
   add observation only using already read values, no arbitrary equality fixes.
4. Firefox rapid-five50FAIL:30late intermediate targets and20target/sequence
   assertions; ordinary median172ms/p95196 suggests a cadence deficit but does
   not prove its source. Preserve 120ms intergesture gap and immediate rearm.
5. Firefox remaining first roundtrip and3pair cases, group by first shared
   mechanism; do not multiply client-specific runtime exceptions.
6. Final exact full native matrix, affected source/build gates, release and
   installed updater verification when qualified. Unsupported URL/email Space
   and protected inverse cases remain explicitBLOCKED until supported protocol/
   isolated learning proof is intentionally supplied. Do not hide them or shrink
   the denominator.

## Diagnostic result and next controlled regression

Diagnostic preview completed all gates,1104/1104IME,10release binaries, no
release warnings, strict lints/bothMSRV/graph/canon, source parity and tiny
offline package bytes. Source closuref3cf6b4567c2129f805f1cb3efaa11a465a72d33310f8ae104bd673f3080b97f.
Installed remote only:engine3890779 SHA
b8a463eda4a04864f887145592558a62ef033e1dd40e38119dbd4437b4bf612d,
daemon3890813,tray3890819; original IBus/desktop parents retained.
Three fixed Chrome normal-field cases:richFAIL,iframe-richFAIL,plainPASS.
Both failures reportpair_scope_missing, not pair_plan_rejected. Each failing
field also reportsclosing_space_presentation_acknowledged (32→160); plain does
not. This proves missing pair scope, with separator presentation as the current
hypothesis for its producer loss, not yet a proven code-level cause.
Local receipts newCACHE/preview-result.json,canonical-parity.json,
diagnostic-deployment-v1.json,diagnostic-native-v1.json; full traces under
CROOT/browser-pair-diagnostic-native-v1. All config restoration and held-key
checks pass. Full440baseline remains unchanged.

Before functional code, add one KDE-native regression using existing managed
commit and actual native-observation seams: owned Cyrillic token, emittedSpace,
actual closingSpace(ASCII orNBSP), next owned Cyrillic letter, actual document
where Chrome may reflow priorNBSP toASCII. ControlsASCII→ASCII andNBSP→NBSP
must retain exact pair scope; NBSP→ASCII must too. AutoReplace disabled only
in fixture to isolate ownership from ranking/models. Test must inspect actual
commits, exact snapshot, tail and pair scope. No parallel modeled owner and no
production normalization/guard relaxation. First run must distinguish a real
reflow regression from an invalid fixture; preserve any failure as evidence.

## Proven separator regression and functional preflight

Official red proof run-ckymrxb8, remote run-NAWdIG:1105selected,1104PASS,
only new native_owned_word_separator_reflow_preserves_the_exact_pair_scope
FAIL. ASCII→ASCII andNBSP→NBSP controls pass; NBSP→ASCII fails with local
tailпров\u00a0е, real snapshotпров е and missing managed-word-start witness.
This reproduces the actual native-client presentation transition using existing
managed commit/native observation methods. No ranker or literal word condition.

DesignA9/10 selected: recognize precisely this full-field presentation change
in the existing KDE native client-presentation adapter before common snapshot
reconciliation. A thin operation in the existing tail owner validates unchanged
shared owner/epoch/tail and acknowledges only the last NBSP separator as ASCII;
no client text is sent. DesignB3/10: normalize whitespace in the common verifier
or grant pair scope despite mismatched snapshots; rejected as false authority.
DesignC5/10: add persistent sent/rendered mapping or a new controller; rejected
because current native binding, local commit floor and observed snapshots can
prove the same transition without new state.

Affected files: KDEclient_presentation.rs (all platform-specific proof),
KDEnative_document.rs (one observation dispatch), tail_memory.rs (new narrow
mirror acknowledgement by existing owner), KDEtests.rs (regression/negative
controls). No old method or delivery contract changes. C01/C03/C05/C07/C08/C10.

Native proof requires same original observation owner, current authenticated
KDE binding, a successful managed CommitText in the current epoch/focus/lease/
layout, no composition/selection/sensitive/sealed/atomic/pending edit, and an
actual prior snapshot whose observed word is a prefix of the existing owned
word. Compare the complete expected field and caret/anchor: only the final
word separator changes NBSP→ASCII and missing already committed letters may
arrive. Every older prefix byte and right suffix remain exact. Reject other
whitespace, word changes, arbitrary offsets, missing floors and foreign owners.

Consequence review: candidates, lattice and ranking unchanged; no new candidate
or authority from logging/matching alone. Common verifier still requires exact
actual source, so no snapshot/guard relaxation. Same owner/epoch updates both
mirrors atomically; never advances input epoch or KnownStart. Cached frames
contain tail bytes; invalidate existing Space work after acknowledgement and
let original snapshot/prediction path refresh. No cache/timer/RPC/wait/fallback.
CPU/RSS: bounded existing field/tail strings only; one comparison/build on the
NBSP separator case, no per-key model work or retained allocation. Current
native bounds continue to cap snapshot size. Latency deadlines/burst cadence
unchanged; measure source and native completion timing after installation.
Package/material generations and reload policy unchanged. Learning/feedback
receives no positive signal from presentation acknowledgement; existing edit
verification remains necessary. Stale/ABA/focus/layout/owner/selection changes
fail closed. Replay/quarantine and pending edits do not use this new operation.
Rollback uses retained exact diagnostic bytes and original component, not a
second factory. GNOME never calls this native adapter, and all old tail-owner
methods remain literal unchanged. Maintenance cost is one KDE observation
adapter plus one small owner API; no duplicate controller or model rule.

Before installation require the red regression to pass, negative owner/text/
selection/desktop cases to fail closed, all old1104 identities retained, final
focused/full affected gates, strict lint/canon/current graph and actual release
artifacts. Then rerun fixed diagnostic cases before the full native manifest.

### Replan after bounded producer attemptv1

Focused native51:50PASS/only new reflowFAIL. Test-only state capture confirms
old source snapshot=None immediately after successful owned CommitText; native
field/owner/floor/shared epoch/tail all remain current, pendingedit=false.
Do not resurrect the intentionally invalidated actual snapshot. Use the
already existing inert ManagedWordStartWitness instead. New retained snapshot
cache4/10 rejected; existing witness9/10 selected. Native closing-space ACK
currently happens after common observation tried to capture that boundary;
refresh the existing canonical witness after ACK without adding a revision.
Preserve an already current authenticated native zero-character boundary in
the existing first-commit rebind method; do not rebind a foreign native owner.
Non-native exact-refresh behavior stays logically identical. Then compare the
real new native field to that prepared frame, accepting only one separator's
NBSP→ASCII presentation plus a prefix of already committed letters. Keep
partial actual snapshots partial: they retain inert data only and cannot edit.
Update the same witness's prefix/data through its canonical owner after the
same-epoch shared-tail acknowledgement. No new fields, cache, RPC or authority.
Add visibility only for the existing reconcile method and a read-only anchored
character getter; other witness fields stay private. Runtime scope adds
window_interaction/observation.rs solely for this native witness continuity.
All previous consequences and guards remain. Require a partial-receipt
control before actual full-word edit permission, plus foreign/stale/selection
negative cases. V1 and state-capture failures stay in evidence; v1 never installed.

V2bounded subset remains50/51; no functional bytes installed. Source inspection
identifies the boundary arming mistake: last_tail_token_text intentionally
returns the last CLOSED word after whitespace, so generic reconciliation
cannot manufacture a zero-character boundary at the word end. Reuse the
already existing managed_word_start_from_current_snapshot("") constructor
explicitly from the KDE presentation ACK. Restore the original first-commit
rebind method and reconcile visibility byte-for-byte: they are unnecessary
changes for this defect. No change to GNOME or source-free rebind semantics.
This is an explicit producer-level replan, not a third guard relaxation.

V3state now proves existing native preparation witness retained (prepSome),
with shared owner/epoch current. The pre-publication hook incorrectly used
native_document_is_current, which also requires a stored actual snapshot;
that snapshot is intentionallyNone before the incoming actual observation
is published. Validate the already authenticated current native BINDING plus
the explicit incoming actual snapshot instead. Do not change the global
native_document_is_current contract. This fixes only the new hook's entry
condition, preserving every owner/layout/privacy/whole-field check.


V4producer subset51/51PASS, positive exact pair regression included. V5adds
partial actual-observation control and26foreign/unproven negative variants;
51/52PASS, last test stopped at an invalid fixture D-Bus object path containing
a hyphen. V6fixes only that fixture path, keeps every assertion:52/52PASS,
0.56s execution after14.88s compile. All50old native test identities retained.
Source closure0db5bcda... is the retained failed V5; V6exact closure in
newCACHE/reflow-unit-v6.json. No functional bytes installed. Next: strict
inventory delta exactly2new tests, official source/build gates, then install
remote only and rerun fixed Chrome cases before any native improvement claim.


First broad official attempt run-w5gg7avq / remote run-JsZiMm stopped in
test discovery, before test results: default development config used old
/home/e/projects/lay-td119-gate-v1/target and Cargo guard correctly rejected
13GiB>12GiB. Retain FAIL as infrastructure evidence, never source PASS/FAIL.
Use the already prepared dependency-updates-20261027/development.json config
explicitly: candidate-target-cache-20261023 verified5.49GiB/12GiB. Do not raise
budget or delete installed activation binaries under the old release directory.


Strict preparation PASS: exactly2new correctness test rows,0removed/changed
old rows, same isolation/fixtures/configuration; old zero-failure declaration
retained as a declaration only. Graph regenerated and imported with all hashes.
Runtime closure6f56b02a7ce971903ad52a6cc54acd6414ef5824a91d1f50321d37079809cad6.
Official broad dev-check run-pna5hk7d / remote run-5mv4i3 PASS3543selected,
0known semantic/infrastructure failures; wall540.1s. Local exactSUMMARY retained
in newCACHE/reflow-source-summary.json. Do not repeat this source run unless
new source changes/failures justify it. Additional existing17source/build gates
ACTIVE exec99175, CROOT/browser-pair-reflow-gates-v1; no functional install yet.
AST caller witness shows all original common owner lines retained and new
entry only through KDE native document adapter. That adapter already requires
actual active process exe chrome; Firefox/terminals/GNOME do not acquire it.


Functional preview V1 installed remotely: engine4944; all10loaded artifacts,
original protected IBus/desktop parents/config retained. Native exact3cases
richFAIL/iframe-richFAIL/plainPASS. Both stillpair_scope_missing, no new
word_separator_presentation_acknowledged. Source3543/17gates PASS does NOT
establish native repair. Exact CROOT/browser-pair-reflow-native-v1/results.
Live rich sequence proves first closing Space is not acknowledged before
the next letter: tail6/snapshot5 before next CommitText, tail7/snapshot6
after it; only final closing Space gets presentation ACK. Earlier regression
artificially supplied immediate first-Space observation. Add that delayed
actual-boundary ordering as source-only red control before further runtime edits.
The existing52test subset remains the cheap iteration route; do not repeat
3543until a functional change has qualified in controlled and native evidence.


V7source-only exact delayed-boundary ordering is RED51/52 (allold50PASS).
The current first-letter CommitText floor is current, while boundary witness
isNone when the actual first-Space-only snapshot arrives. Old prototype
assumed that receipt arrived before the letter. Existing producer can retain
a real observed boundary plus one completed owned first-letter append as inert
preparation, without promoting callback lineage or constructing an observation.
Selected9/10: let the canonical witness constructor receive the actual observed
start-tail and the explicitly verified one-letter effect epoch; old callers
pass their exact original self.tail/epoch. KDE adapter invokes only on an
actual boundary snapshot after that exact first local append, no selected text,
no pending edit/replay, same owner/focus/layout/epoch and fresh observation.
Reuse existing shared-tail presentation ACK for either observed Space/NBSP
(and exact unchanged delimiter), retaining entire other text and input epoch.
Reject a new timeout/cache (3/10), weakened pair/verifier (1/10), or invented
full snapshot (1/10). No new runtime fields, RPC, output path or GNOME caller.
Add delayed actual-observation/foreign-state controls before another preview.
Earlier producer-only functions may be retained only if this real lifecycle
regression needs them; native V1failure remains open until fixed-case proof.


V8PASS52/52with actual late-boundary control; old constructor body proved
byte-identical after substituting original arguments (witness in newCACHE).
V9adds25late-boundary negative controls to original26. It correctly catches
late=true/no_native_owner: the helper must not run from generic postcondition
after accepting a newly acquired field. V10moves only this new call to native
installation AFTER the real snapshot publication and under the pre-existing
same_observation_owner comparison. No comparator is relaxed; a new/changed
field has no such continuity and cannot seed the inert append witness.
V8/V9functional code NEVER installed. Current remote remains native-failing
fully built V1engine4944. Preserve these failures; no retries of old matrix.


V10PASS52/52:5positive ordering variants,51negative state/shape variants.
Focused official attempt run-h9x4f0un BLOCKED before worker execution by
shared lease (other L3 scope84216). Keep operational blockage separate from
source/native failures. No guard bypass or unrelated job stop. Previous
full3543is V1source only; V10stillneeds fresh1106IME proof beforepreview.


After shared L3 worker completed, fresh official focused run-g64cvnkz /
remote run-xMF72x PASS1106selected/executed/passed,0failed,177.3s. V2preview
failed in local orchestration: focused schema lacks full-run semantic-count
keys and elapsed_seconds. No runtime install. V3reads focused schema explicitly,
requires exact1106selected/executed identities and0failures,manifestMATCH.
Reuses only4already passing unchanged source checks from V2 with full frozen
request byte verification; budget rechecked. Combined V3source gates→install→
fixed3native cases active under one lease, exec72961. Runtime stillV1until
its installer returns a verified deployment. No new functional/native claim.


V3preview PASS1106IME/13required source-build stages,11performance,10release
binaries/no warnings; installed remote engine156603, daemon156644,tray156651.
Fixed3native cases: richPASS,iframe-richFAIL,plainPASS. This is one measured
case improvement, not a new full matrix. Crucially iframe now logs both
late_word_boundary_observed andword_separator_presentation_acknowledged,
then stillpair_scope_missing. Do not generalize the one-append constructor
or alter normalization without new evidence: that first transition succeeded.
Next bounded diagnostic only in KDE native_document.rs existing trace:
log pair-presence in captured frame and live scope, native preparation and
whether its projected tail matches local data (booleans, no user text).9/10.
This distinguishes a producer/prefix loss from stale prepared work. No guards,
returns, action, input, owner, queue, timer or RPC change; debug-only bounded
snapshot clone is not a latency improvement. Reject speculative generalization
(4/10) and pair-verifier bypass (1/10). Require AST/byte-removal equivalence,
fresh focused compile/test and actual release build before fixed iframe trace.
# 2026-10-27 — V4 installed; five fixed iframe trials expose final stale source

Source66d620890b9c58634c5955f91b2e824ae815fe7853822c8165308e8e8dc25a9a
passed1106/1106 canonical IME checks,13 preview gates,11 performance checks and
all10 release artifacts with no warnings. Installed remote engine256768,
SHA6f248b178d6c7cf50a522fd0c10ac204b47a2a02af18b3f1e6550b309cfdeb94;
original IBus parent3168326 unchanged. V4 single iframe trial PASS35.29ms.
One PASS did not close V3's native failure. A predeclared five-trial diagnostic
on the same installed bytes produced3PASS2FAIL, successful latencies72.50,
105.29,81.14ms; all configuration/key cleanup checks passed. Receipts:
`CROOT/browser-pair-reflow-{gates,deployment,native}-v4/result.json` and
`CROOT/browser-pair-reflow-ordering-v1/result.json`.

Both five-trial failures retain a current NativePrepared pair during typing.
At final Space the correction worker selects the right action and logs
`authorized`; final replacement refuses `stale_surrounding_text`. The actual
snapshot still lacks one owned letter. Subsequent missing-pair logs occur
after fallback Space has closed the word and are not the first failed
transition. Earlier missing-managed-frame-identity hypothesis is insufficient;
do not change the shared frame constructor for it.

Consequence map before the next edit: `kde/native_document.rs::read_native_document`
already performs one source-only reread under its original80ms deadline.
`kde/native_document/preparation.rs::native_us_append_observation_is_pending`
currently classifies only US input, excluding the demonstrated Cyrillic pair.
Extend this same predicate to a current NativePrepared pair and rename the
helper consistently; retain all owner, exact prefix/right suffix, caret,
selection and deadline checks. No new read loop, delay, edit permission,
executor, output route, shared preedit change, GNOME or terminal change.
TDD first extends the existing canonical pair test with one completed local
append and an actual prefix missing that letter; a fully current/foreign/
selected/future snapshot must not trigger a reread. This is observation
classification; only the unchanged final exact snapshot may authorize edit.
Full baseline remains Chrome125P4F31B,Firefox75P54F31B; no new full matrix,
USB/evdev, Wayland, local GNOME or stable release claim.

# Release1.0.105 preflight — native Space completion through existing protocol

V21RED measured:54selected,52oldPASS,2newFAIL at pending-slot retention after
the bounded native lookup. The lexical fixture supplies the real authorized
pair action before that lookup. Source b762617a362dd9993a6d0a6c7dfb3249edc98aca27f94623d9f6e299f7866b13;
remote CROOT/browser-pair-reflow-unit-v21-red/result.json and tests.log.

The user authorized the autonomous seven-release series. Read
isolated-release-series-2026-10-28.md and decision
2026-10-28-kde-native-space-continuation.json before production editing.
This explicitly expands the old one-KDE-file scope for the current defect.

Allowed production files: committed_tail.rs, managed.rs,
kde/native_document.rs and kde/native_document/preparation.rs in the IME.
Test files: kde/native_document/tests.rs and space_boundary_pair_tests.rs.
The shared composition commit, admission, verifier, executor and GNOME modules
remain frozen. Generated graph/inventory, version/release metadata and evidence
follow the existing release process. No local install/restart or model rewrite.

Reuse existing prepare/commit/notify/apply boundary continuation. Its receipt
retains a pending calculation, exact before/after snapshots and original
generation. Generalize the existing commit helper name, preserve OwnedPreedit
behavior, and admit NativePrepared only through the current full KDE field.
Native text is already committed: emit only one Space. Native observation must
join the pending boundary before ordinary postcondition settlement consumes it.
No new timer, owner, queue or output executor;3500us callback budget unchanged.

First RED must show current native Pending lookup retires its slot. Then test
both event orders and stale/cancelled states through the production worker and
existing receipt. A fixture without a real native client proves the boundary
and fail-closed refusal only; physical seven and Chrome220 establish edit effects.
All previous native/legacy tests and canonical proof dimensions remain required.

---

# Scoped continuation — KDE Chrome field continuity, 2026-10-28

Physical result V8: fresh1106/1106 and all13 preview gates PASS; all10 retained
release artifacts installed on remote KDE, loaded engine706964 SHA18d0a4b8a8c4adb719fc99fd49a03807026ee3e9e998b27c71193f842a2de5d6.
Source740924fc781779ff53e2653b40b6126083f792fd01d0f04a36a245d3e8022d39.
Same seven physical cases remain6PASS1FAIL, so fullChrome220 NOT_STARTED and no
publication. Space correction latency for6successful samples: median95.79ms,
nearest-rank p95/max129.26ms; failed trial excluded from target latency and
retained as FAIL. This is not Double Shift latency or general acceptance.

Evidence limit: V8 failing trace contains no `field_receipt_rebound` event.
The forward-reconnect fix is proven by the controlled52-test regression, not
by a physical reconnect in this trial. Pair preservation in V8 before Space
must not be advertised as native acceptance of that reconnect branch.

Remaining V8iframe1 first transition is now distinguished from V7: trace377
has pair_in_frame/pair_live/native_preparation alltrue; trace391 still has valid
preparation and complete11-character actual field. Trace395 lookup says
prefetch_not_ready,397 worker5/tail15/wait3us; ordinary Space is committed and
the pending worker is cancelled at405 after11856us. Missing pair observed after
that fallback is a consequence, not proof of the old pre-Space loss. Current
shared consumers committed_tail.rs:232 and managed.rs fallback are unchanged.
Do not raise waits blindly or repeat/revert the already proven reconnect fix.
Next analysis must map computation start/readiness and the existing callback
budget before choosing between a KDE-only scheduling correction and a separately
scoped shared change. The current one-KDE-file scope does not authorize editing
the shared consumer. Receipts: CACHE/raw-browser-pair-reflow-{gates,deployment,
native,combined}-v8--result.json and reflow-v20-scope-and-native-verdict.json.

Measured result: V19RED51/52 becomes V20GREEN52/52 with only the local hook
predicate changed. Source after unit ccc4e4fc72dd3bc6cefab33400299224f244afa3f34b77de6451d80b4ec55df1;
receipt CACHE/reflow-unit-v20.json. All old native identities retained; strict
observer predicate equals c794d31d byte-for-byte. Freeze witness
CACHE/reflow-v20-isolation-witness.json confirms no additional common runtime
change. Installed runtime remains V7; physical outcome of V20 is NOT_TESTED.

Change class: existing authorized KDE/Chrome field-integration repair, not
correction-only work. Baseline source c794d31d with the already recorded
experimental diff; installed V7 source2abed764 / version1.0.104, seven native
6PASS1FAIL. The existing shared observation/tail changes remain disclosed and
unaccepted; this continuation adds no common production changes.

Allowed production file: `src/bin/lay_ibus_engine/kde/native_document.rs`, only
the predicate guarding the newly added separator/boundary preparation hooks.
Allowed test file: its existing `native_document/tests.rs`, including the
already RED forward-reconnect regression and formatting. Generated graph/test
metadata and owning evidence may follow existing tooling. Freeze all other
runtime sources, particularly common observation/tail, admission, decision,
verifier, delivery, display, desktop dispatch and every GNOME module. No env,
launcher, model, deadline, input sequence or executor change.

First failed transition: V7 iframe1 accepts the same physical field after a
forward KWin provider reconnect, but the new data hooks require exact transport
receipt equality. V19RED51/52 proves preparation lost, while its existing asserts
require the old observer subscription to remain invalid and no text mutation.

Designs: (A) use physical-field continuation plus unchanged focus/lease/layout
only for these two data-hook call sites, 9/10; (B) extract the same predicate into
a binding method, 8/10, viable but no other caller needs a new API. Choose A.
Changing `same_observation_owner` globally is rejected: it would broaden observer
subscription and display-cache reuse. Keep both strict users byte-identical.

Consequence check: no candidate/lattice/rank or learning changes; no new edit
authority, fallback, owner or state. Incoming binding must pass existing current
token/admission/tail checks and existing `continues_from`; previous focus/lease/
layout must match. Old transport/cache identity stays exact. Additional work is
bounded identity comparisons with no allocation, RPC, wait or deadline change;
package reloads, dependencies, memory and learning semantics are unchanged.
Missing/old/foreign receipts retain existing rejection, and no-observation
retains Unsupported until fresh actual text. GNOME/common consumers are frozen.
Rollback is one local predicate/two uses; no parallel permanent route.

Proof plan: retain V19RED, all52 native identities/old negatives, fresh1106 exact
IME checks, canonical preview gates/builds, loaded-byte verification, the same
predeclared seven physical cases, then Chrome220 only after seven PASS. Keep
old failures and full-matrix totals until a matching new full run exists.


# TD-151 — preserve the prepared Space frame across a matched Reset

Priority: P1, release blocker. Status: DONE_SOURCE; new-byte native/CI/install gates pending.
Baseline: b411182292fcc0ed46a6d0e0127a28f5ae132486, candidate1.0.88;
installed1.0.87 unchanged. Root implements; independent agents review only.
Owner: existing ContextAdmissionReducer / WindowInteraction and prepared Space
worker, C01/C03/C05/C08/C09/C10. TD-150's completed native replay repair stays.

## Observed failure and first evidence gap

Fresh complete Firefox216:169PASS/28FAIL/19BLOCKED. Original TD-150 fullscreen
repeat now passes all5 steps and standard60/60. One formerly passing identity
is lost: functional/normal/space_en_ru/field7. Requested and observed seed modeUS,
exact raw seed ghbdtn, caret6, no selection or displayed suffix. Space leaves
ghbdtn plus NBSP instead of the expected Russian correction. This is a real
client failure, not cleared by the successful Double Shift matrix.

Local receipt: /home/ubu/.cache/lay/development/td150-install88-20261016/native/firefox/RESULT.json
Comparison: same root/native/firefox-vs-READV.json.
Immediate bounded trace and raw row: same root/space7-failure/
firefox-functional-l3nq1f8n-7-space_en_ru.{json,jsonl}.
Remote actor: /home/e/projects/lay-development-runner/firefox-legacy-native-20261012.6onP5c/firefox-functional-l3nq1f8n/RESULT.json.

Measured trace: final letter creates exact prepared decisions14 and15 at
tail_epoch11522 (54us and26us). A real final surrounding receipt confirms the
same6 characters. Matched Reset46045 follows before release46046 and Space46047.
At Space lookup returns NotReady, worker15, wait0us. The earlier full worker9
at epoch11520 belongs to a4-character prefix and MUST NOT be reused. New raw
trace clocks place final letter callback about40ms before Space; an allegedly
slow final full computation alone does not explain losing an inline exact slot.

Source hypothesis, NOT YET PROVEN: reset_for_ibus_soft_reset's exact-frame
retention captures a different authority representation from the prepared slot
after observe_context_revocation/clear_preedit. A current exact prepared slot
is thereby retired despite an otherwise preservable same-word owned Reset.
Determine which identity field changes with a real adapter callback sequence;
do not make stale work current by comparing text alone. Chrome new full216 is
still running; do not combine it with older Firefox or run heavy work concurrently.

## Options

1. Recommended9/10 if causal RED confirms it: reuse the existing exact owned
   Reset observation and prepared-frame retention at its correct capture point.
   Keep computation inert until the existing current authority permits an edit.
   Compare all identity/material coordinates; preserve matched Reset provenance,
   actual same-owner confirmation and one-use output authorization. No new owner,
   cache, generation, timer, queue, transport, ranker, model or learning change.
2. 4/10: reschedule exactly the same work after Reset. This may discard a ready
   result and reintroduce key-before-ready; only consider if exact retention is
   impossible and the existing mechanism can do it without deadline changes.
3. 2/10: increase Space wait or sleep before native keys. An inline result was
   already prepared; delay does not explain its loss. Not selected.
4. 1/10: ignore Reset, match only a token string, reuse an older prefix result,
   weaken verifier/SafetyGate or add a browser branch. Rejected by C03/C05/C08.

## Minimal causal proof and acceptance

- Extend existing terminal_delivery real legacy P2P harness, not a simulated
  keyboard model. Type an ASCII-layout word through ProcessKeyEvent; deliver
  actual Reset/SSI/release in both orders, including final exact SSI BEFORE
  matched Reset and no extra post-Reset snapshot until Space.
- First inspect/assert exact pre/post frame coordinates and prepared slot.
  Use existing proof slot installation only to freeze readiness deterministically;
  it grants no client edit authority. Preserve a real inline exact preparation
  control. RED must fail ready-slot/observable correction, not fixture setup.
- Positive cases need existing exact owned Reset observation; an unconfirmed
  arbitrary Reset stays inert. Test prefix/Unicode, cold/unavailable worker,
  actual selection, contradictory text, foreign focus/owner, changed config or
  capabilities, epoch, expiry, and wrong content type. Retain existing adverse
  cases rather than duplicate their implementation. Assert surfaces, disposition,
  layout, exact proof and effects; no duplicate Delete/Commit or learning claim.
- If RED points elsewhere, update this diagnosis before any production edit.
  Add an explicit architecture decision identifying exact changed protected
  paths only after cause is known. Preserve the 3.5ms budget and current routes.
- Remote guarded focused RED/green plus controlled restoration of the old guard;
  old TD-150 and affected Space/reset controls. Full fixed source/package gate,
  actual MSRV/lints/canon and fresh independent code review, max2 repair passes.
- New matched build and complete fixed432 browser matrix with latency/missing
  counts, separate optional80 and cleanup. Never rerun unchanged bytes until
  green, splice best rows or reinterpret unknown client results as PASS.

## Consequences / bounds

Lexical lattice/candidate ranking, package identities, online reload, models and
learning are unchanged. Existing prepared-result retention changes only when
its same-owner Reset proof is captured; no new persistent state or executor.
Main risk: retaining stale authority after lifecycle change. Require all existing
identity/material equality and matched owner/Reset checks; inert computation
must not become an authorized edit without current exact receipt/witness.
No additional RPC, polling or wait. Measure latency on actual changed bytes;
do not infer speed from test count. Roll back this isolated change by Git revert,
preserving TD-150 and user data. Stop on a missing causal RED, unexplained proof
coordinate, full-gate regression or new native lostPASS; keep rejected receipts.

CI's overlapping-release failure is tracked separately in TD-152; a shared
cause has not been established. No installation while either gate is blocked.


Independent fresh-context plan review pass1/2:9/10,
/root/td151_td152_plan_review_20261016. Preparation trace precedes actual slot
publication: new test checks the real inline exact-ready slot, then freezes
readiness with the existing proof helper. Observe pre/post frame coordinates;
restore_owned_reset_observation runs after soft-reset retention and cannot
restore a retired slot. No production edit before semantic RED.


## Causal RED and selected minimal repair

Unchanged-production td151-reset-diagnosis-359fg2lz/RESULT.json:2/3 PASS,
17.25s, archive3fc2c35058f507056feb72b3437ebbdff7b0252be7e33007f262d1d87df93df1.
New regression reaches the real inline exact-ready slot, freezes readiness with
existing exact lease, performs actual matched Reset, then fails exactly the
ready-slot retention assertion. The returned pending and restored current
InputFrameIdentity are byte-for-byte/every-field equal to the prepared frame.
Old exact-refresh and unchanged overlapping-release baseline pass; CI original
cause remains UNKNOWN. This is a semantic RED, not a timeout/setup failure.

First failed transition is soft-reset retention BEFORE owned snapshot restoration:
revocation makes the marked receipt temporarily unconfirmed; the managed echo
branch can select the independent surrounding-revision representation while
stable pending predecessor-token identity already exists. restore_owned_reset_
observation later returns the exact previous authority but cannot recover the
retired slot. Selected minimum: prefer existing pending Reset computation frame,
then retain the former managed-echo frame as fallback when no pending frame exists.
No identity comparison, verifier, deadline or output route is relaxed.
Decision2026-10-16-retain-stable-pending-space-frame-on-reset.json records scope.


Remote focused td151-focused-crl7ahna/RESULT.json13/13 PASS,36.97s including
controlled restoration of only the old state.rs capture selector: the new exact
ready-slot assertion fails again1/1, current source restored/SHA checked. Existing
midword owned echo/delayed managed start/selection and fragment/one text owner/
TD150 completed replay/revoked scope/Space contradiction/deferred completion/
pending real worker/overlapping release/exact lease fault matrix remain selected.
The new test also asserts exact CommitText payload/count, not only mirror/layout.
Full fixed source and actual new browser bytes remain pending.
New context-free code-review spawn hit platform thread limit. Reused existing
independent TD150 plan-review thread explicitly for code review, without claiming
fresh context; root remains the only implementation owner. No review waiver.


Strengthened final focused td151-final-focused-ut_63nbx/RESULT.json13/13 PASS,
36.85s including controlled old selector1/1 detected, current source restored.
Exact single CommitText payload and new reporting contract17/17 now included.
Independent code review1/2:9/10, no confirmed P1/P2, reused unrelated prior-plan
thread /root/td150_completed_replay_plan_review due platform thread limit.
Fresh full frozen source td151-full-source-gvt96j4h is running; archive
844618fca16f9ca6a39e1e6dae964313362884fdadaa1c5c1f0f392689e2bd5e.


Full remote source td151-full-source-gvt96j4h/RESULT.json3211/3211 PASS,
3175correctness+36package, zero semantic/infra failures,693.46s. Manifest3237
keeps all old classifications and adds only TD151's real adapter regression;
15ignored/11performance separate. ActualMSRV1.88both, strictlints530/354same,
canon15, architecture/audit50, Nodeobserver, harness122(oneexplicitSkip), panelPASS.
Archive844618fca16f9ca6a39e1e6dae964313362884fdadaa1c5c1f0f392689e2bd5e.
Full source metadata fetched/SHA bound; final graph and strengthened Python
encoding/fixed-only assertions are checked separately in final canonical gate.
Those added test assertions do not change runtime or test classification.


Final canonical td151-final-canonical-1gumzyss/RESULT.json PASS48.56s,
current17/17 Python cases including late encoding/fixed-only assertions,15canon
checks and syntax of9native+6build/install helpers. Four canonical metadata
files fetched/SHA verified; all993 functional full-proof inputs remain identical.
Full-proof source receipt SHA75cff0689ea10b436d602b0d3eaa32d7a1977dd33cd4e9e36a2dab6e2d86ecf1.
No functional source changed after full snapshot; only additional Python test
assertions (separately green here) and metadata/docs. New source not installed.


Final independent code/evidence/private-adapter review2/2:9/10, no open P1/P2,
/root/td150_completed_replay_plan_review (reused thread, not fresh context).
All15 private helper hashes match ADAPTATION; new IDs/tags/output paths retain
original actors/bodies/guards/denominators. Root alone implemented. Final canon
PASS is independently verified above; reviewer does not replace its receipt.
New native and GitHub CI remain gates. Source task complete, installed87 intact.


Published/read back source e34a47ac8436918e9a243158b56b55d1f7eb82ca on origin
work branch and public work/main refs. TD152 reporting commit67debce1 is separate;
private origin/main untouched/no force. Matching build frozen119269e9713aec4e4e84b5326cc8d706aa90d2937f3e53974576db25a94261d7,
/home/ubu/.cache/lay/development/td151-install88-20261016; build/native/CI pending.


Matching build complete83.38s: archive119269e9713aec4e4e84b5326cc8d706aa90d2937f3e53974576db25a94261d7,
CLIac7f3fbc..., daemon625feb96... unchanged from earlier88, new IME
8cb7bca42505a4a7c0aa3a94d8caaac92b887ea6c24dd7e4fe8ee94b7a74b132,
993functionalinputs/SHA match, lock/auditDB same and0advisories. No local install.
New remote Firefox REPAIR2 full216 is running (jobs1051..1059), Chrome next.
Private prelaunch readonly check caught alphabetic-only existing zero-tag/seat
label constraints: corrected only unique labels to LAYREPAIRB/LAYREPAIRBOPT and
repairbverify/repairboptions BEFORE staging/input. Existing guards/actors/routes
unchanged, all15 updated helper hashes/syntax/argument constraints remotePASS0keys,
PARAMETER-PREFLIGHT.json. The earlier adapter review remains about its ancestor;
this constant-only startup correction is recorded explicitly, no third review.
CI38908900651/Security38908900839 pending first attempts, both sourcee34a47ac.


New e34 paired native COMPLETE349P50F33B/432, NOT_TESTED0: FF170/27/19,
Chrome179/23/14, zero lostPASS vsREADV in either browser. Actual normal Space7
now PASS including target/next decoder/return,20.146ms. Both scopes empty,
all pinned processes gone, no held keys, VT1. The remaining CI fixture blocker
is separately owned by TD153; this runtime repair is source/native complete.
Whole receipts: /home/ubu/.cache/lay/development/td151-install88-20261016/native/.


Final release closure —2026-10-16: functional source77af7ade, full3211/3211,
CI38916364078 +Security38916364185 firstattemptSUCCESS, independent review9/10.
Installed matching CLI/daemon/IME/panel1.0.88;6actualpayloads match native-tested
e34 bytes. Whole432=349P50F33B and normal optional80=58P18F4blockedpreconditions
remain explicit, no all-client acceptance claim. GlobalIBus/L1.1/L3, models,
settings/sources/mode unchanged; no localfunctionalkeys. Rollback/procSHA evidence:
/home/ubu/.cache/lay/development/td153-install88-20261016/INSTALL.json.
All old negative receipts and each card's narrower source/ACL/ARM scopes remain.

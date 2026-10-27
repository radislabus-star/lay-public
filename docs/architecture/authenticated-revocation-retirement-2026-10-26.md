# Authenticated revocation retires the old word without erasing received focus

Owner: existing WindowInteraction / ContextAdmission reducer and bounded stamp
store. Invariants: C03, C06, C08, C10. Stage: original fifth release group,
candidate 1.0.102. No new owner, input route, model or verifier permission.

## First failed transition

Baseline 1.0.101 files/modes closure:
8c25e15e9c6f9c481a0b91e5da1b59227749f19cfd6c2db187c8bf7ee27f1a9f.
The actual receive observer rejects Disable without a preceding FocusOut, or a
second FocusOut, and revokes the old reducer word. It records an authenticated
Revocation for that old owner. A later real FocusIn is received before the old
callback runs. observation.rs::observe_context_focus_out/disable validates the
exact old revocation but calls revoke_context_word again. That generic call
clears the shared stamp store, erasing the successor's already received focus.

## Minimal alternatives

9/10: only the two validated revocation branches strip the old preedit mirror
and retire local word witnesses with existing helpers. The receive-side
revocation remains final. Normal lifecycle cleanup runs; a new word still
requires actual context Get and its native marker. No old word can transfer.
2/10: globally preserve stamps for every revocation; this would weaken unknown
or malformed input handling. 5/10: manufacture another focus request from the
callback; this changes acquisition flow instead of retaining received input.

## Measured controlled experiment

Two separate tests use actual P2P receive/callback entrypoints, a genuine owned
preedit, real Disable / duplicate FocusOut and a later received FocusIn. They
assert the old token is invalid, no text edit/learning is issued, and only a
fresh context Get plus marker establishes SourceFree/UnknownStart/zero suffix.
Baseline: 0 PASS / 2 FAIL on the exact lost-successor assertion. Minimal
candidate: 2 PASS / 0 FAIL. Both experiments restore the original source and
verify the full baseline source identity in finally. Installed 1.0.101 bytes
were not changed by these experiments.

Raw receipts:
CROOT/release102-authenticated-revocation-red2/{result.json,test.log,source-restored.json}
CROOT/release102-authenticated-revocation-green/{result.json,test.log,source-restored.json}
Local: CACHE102/red-green-experiments.json.
CROOT=/home/e/projects/lay-development-runner/kde-install-control-20261020;
CACHE102=/home/ubu/.cache/lay/development/release-1.0.102-20261026.
Baseline observation SHA:2db5e35cf272f771ec9aee6b3eaba3ee44ff1419cc784287cd619a6203f10c36.
Candidate experiment SHA:79b35a8c416442b32524852307f3ec81879d9d9f16c5487f79fc4401da9c0696.

## Pending promotion proof

Add negative controls for missing/stale provenance and a superseded owner;
retain all prior tests. Full source/performance/lints/MSRV/optimized binaries,
exact loaded bytes, native controls, CI and updater acceptance remain pending.
Two source tests do not establish browser or terminal acceptance. The separate
Chrome Space stale_lease and WezTerm presentation failures remain OPEN.
Do not force an unrelated file split for TD-132/134 or claim TD-123 quality
promotion; reconcile their current owners explicitly without inventing PASS.


Implementation prepared in the two exact admitted branches, using the existing
strip/retire helpers; generic refusal branches unchanged. Added a negative
control for an unobserved header. Its first draft incorrectly required no live
token after generic revocation; existing reducer semantics deliberately retain
a new-generation UnknownStart/zero-suffix token. Corrected test asserts the
actual no-old-word permission contract, not a changed runtime rule. That revised
negative passes on unchanged101 (CROOT/release102-unobserved-revocation-baseline).
The rejected draft run has11PASS/1FAIL of12 and remains retained under
release102-authenticated-revocation-controls. Full102 verification is next.


## Candidate A rejected: preserve the complete lifecycle cleanup contract

A functional32bb94f2/sourceac2264f5 passes3522+11/all10/CI and records full
native301P61F78B/440 (Firefox126/57/37,Chrome175/4/41), Kitty5P/Konsole5P/
Wez1P4F. No release. Before promotion, paired semantic characterization found
that pre-stripping the composition skipped the factory-mode restoration in
reset_for_ibus_soft_reset. The old observation body passes that real Disable
schedule; A fails on the exact factory-mode assertion. Receipts:
CROOT/release102-disable-mode-characterization/{result.json,*control.log,*candidate.log,source-restored.json}.
Original source is restored in finally; installed A runtime was not changed.

B reuses nearby discard_context_activation plus retire_context_word_locally
ONLY in exact revocation_matches branches. It drops this already revoked local
owner, leaving composition retirement to normal FocusOut/Disable cleanup. That
normal reset preserves all prior mode/preedit/publication behavior and cannot
revoke successor metadata again because the old local owner is gone. Valid
Pending-ticket FocusOut remains unchanged, including its TD125 inert prefix.
No new owner, flag, reset bypass, delivery route or mode-specific setter copy.
This existing-owner reuse9/10 is preferred over a mode-only duplicate5/10 or
wider reset bypass2/10. Both original positive tests now arrange the opposite
factory mode and assert normal restoration as well as retained received focus,
fresh Get/marker/UnknownStart0, invalid old token and no edit/learning.
All test names/lanes stay unchanged. B requires fresh full gates, new CI and
native receipts; A proof is historical, not B acceptance.


## Candidate B rejected: a word is not its focused field

B f5dd7e1a/source d0193b6c passes3522+11/all10/sourceCI but fullnative is
Firefox122P61F37B,Chrome173P6F41B,total295P67F78B;Wez5FAIL. No release.
OldA versusB paired real Disable WITHOUT a following FocusIn proves oldA
retains the focused field while B clears it. That cleanup also removed shared
focus/read continuity, stronger than the word revocation being handled.
Receipt:CROOT/release102-focused-context-characterization/result.json;
source restored in finally. This disproves the earlier B helper selection,
not the requirement to preserve future focus metadata or factory-mode cleanup.

C preserves the inert context using local word retirement, matching oldA.
A canceled owned preedit is stripped locally, and Disable explicitly invokes
the existing factory-mode setter for that strip, before ordinary cleanup.
Word witnesses and pending learning are revoked before the setter. There is
no new flag/cache/owner or reset bypass. The original positive2 tests still
assert futurefocus AND mode; a fourth regression covers ordinaryowned and
privatecommitted Disable without newFocusIn: focus retained, old token invalid,
word unknown, no text edit or learning. Full manifest adds exactly1 correctness
test to B (3523), retaining every prior identity/lane.

Measured evidence favors this bounded9/10 repair. B activation-discard was
5/10 after its broader side effects were measured; it is reserved for actual
activation failure, not word loss in a still-focused client. Neither coarse
context discard nor global stamp retention should be substituted here.
Fresh Cfull/source/native/CI required; no A/B acceptance is reused for changed
behavior. These are retained rejected experiments, not release claims.


## Scope map completed and user authorizes verification — 2026-10-26T23:15:04.130542+00:00

The user required a full influence map before proceeding, then explicitly
authorized fix -> test -> release if accepted. See
`docs/development/ime-impact-map-2026-10-27.md`. C changes two exact admitted
revocation branches of the common owner; delivery, profiles, ranker and verifier
remain unchanged. This scope is explicitly common rather than browser-only.
C full source proof:3523+11/all10, source1289/e72574e997f6e4494ef3647e2b2b9f43826ac2b4bd72240c37c45ede57f5cbf2.
Local source bytes and modes matched; no physical C acceptance is implied.
A fresh native101 baseline runs under the same current conditions before C.
No reliable manual-window trigger for the controlled ordering defect is known.


## C native comparison HOLD; trace collection boundary

Same-condition101 full:Firefox125P55F40B,Chrome173P6F41B,total298P61F81B/440.
C full:Firefox125P58F37B,Chrome172P5F43B,total297P63F80B/440. Nine oldPASS
losses/eight gains; no publication. Row identities retained in CACHE/c/
baseline101-vs-c-native.json. Timingmedians101/C:Firefox163.864/165.170ms,
Chrome47.829/46.969ms; p95245.305/244.198and63.602/63.176. Coverage differs,
so no acceleration or all-window quality claim.

Reading source debug_log.rs proves async production writes:1000ms active flush,
bounded4096 channel and2048-line flush threshold; restoring debug=false can
discard pending records. The frozen actor captures row logs immediately and
its trace_complete flag checks only anchor uniqueness. That is not a causal
completion fence. A successful baseline row's captured final key sequence can
belong to a different operation; don't compare it to a failing row as root proof.
Native DOM/a11y target verdicts remain unchanged; log-based causal attribution
is restricted. Diagnostic collection must drain while logging remains enabled
and validate actual observed key/source sequences. Do not change runtime logging
or input delivery to repair the test reader.

Predeclared bounded follow-up:all nine lost-PASS selectors, one cohort per
version, using the unchanged actor --only selector/cadence/targets. No repeated
full matrices, retry-to-green, safety changes or further IME patch without a
measured first divergent transition. Protocol CACHE/c/native-causal-diagnostic-protocol.json.

## Diagnostic selection correction — 2026-10-27

The first bounded cohort completed, but inspecting actual row identities found
that the frozen actor's `--only` explicitly selects `normal`. Its cases05/06
therefore did not cover the requested private Firefox losses. Preserve those
four results as supplementary normal controls, not private acceptance. Of the
seven correctly selected target identities,101 has5PASS/2FAIL andC6PASS/1FAIL;
iframe-rich moved-boundary fails on both, while Chrome textarea space-en-ru
fails on101 and passes onC. This does not replace the full440 matrices or prove
that each original loss is unrelated to the change.

Amendment CACHE/c/private-scope-protocol.json predeclares exactly two actual
private Firefox rapid-five cases (url/plain), once per version. A copied actor
adds only a validated diagnostic-window argument and uses it in the existing
mode selection. Native input, cadence, target predicates and original frozen
actor remain unchanged. Copied actor SHA256:
a0e865d7a46d1f61aef9b9c2a92634a5ecba0dc9ca1579a10e529bb2e7b23abc.
Wrappers require the observed row.window to equal private; retained deployment
artifacts and loaded process bytes are verified before each version. This is a
correction of missing coverage, not a repeat of already valid target results.
No additional runtime source change or publication is authorized by a test PASS
alone. Full comparison, causal trace completeness and original failures remain
separate acceptance boundaries.


## Scoped lifecycle acceptance; native residuals remain OPEN

All two actual private corrections pass on both versions. The complete bounded
nine-target diagnostic is101=7PASS/2FAIL,C=8PASS/1FAIL, without a new oldPASS
loss in that cohort. Source ordering controls, all3523+11 gates, all10 release
artifacts and exact functional-source CI pass. Accept this bounded lifecycle
repair for scoped publication under the existing six-release plan; do not
promote it as browser-quality, visual or all-client acceptance. The original
full440 nine losses/eight gains, preceding-state differences and trace limits
remain recorded and OPEN. No assertion of regression freedom is made.

Both diagnostic iframe source sequences equal the entire intended input
«должн ыбыть ». Both reach replacement_changes_uneditable_prefix and then
full_no_apply/verifier. That existing refusal is independently reproduced on101;
it is not repaired here. Chrome diagnostic space-en-ru reaches identity_mismatch
on101 and applies onC; one isolated pair does not attribute all full-run churn.

Evidence:tech_debt/evidence/release-1.0.102-{verification,native}.json;
CACHE/c/causal-paired-source-sequences.jsonl; private amendment as above.
No runtime source changed after the measured C gates. Actions storage follow-up
remains queued after the WeChat-family task, per the user's explicit request.


## Published1.0.102 — 2026-10-27T00:23:46Z

Public tagv1.0.102 resolves04a0f65e0a4e22e317a350652cc7638c23849680;
PR71 merged by fast-forward. Both installed lay-update --check exit0: local
101->102 offered, remote102current. Receipt release-1.0.102-updater.json.
No manual local install/restart. Private divergent origin/main47fe54e6 unchanged.
The publication changes evidence only after functional305 validation; no fresh
all-window, GNOME appearance or regression-free acceptance is implied.

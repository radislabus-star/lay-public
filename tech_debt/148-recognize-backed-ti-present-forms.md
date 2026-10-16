# TD-148 — Backed present forms of same-stem -ти verbs (#59)

Status: DONE_SOURCE, final review9/10; publication pending. Priority: P1.
Baseline: `798fa391`, source1.0.87; installed1.0.87 from `50d314b3`.
Issue: https://github.com/radislabus-star/lay-public/issues/59

## Root cause and observed scope

Current isolated release86 CLI preserves all14 tested clean cases (with Nanda
on/off), including `привезу`; relevant library sources are unchanged in87.
This does not prove the low-level form recognizer is complete. Its existing
data-driven ending-to-lemma relation omits same-stem first-person singular
forms backed by -ти infinitives: `привезу` reconstructs unrelated infinitives,
never `привезти`. The first missing authority is
`russian_lexicon::forms::backed::is_backed_russian_verb_form`, before
`composite_russian_typo_candidate::field_knows_original`. Other current layers
can preserve the clean text despite that hole. Do not claim a native repro of
the author's former output, or that personal learning repairs morphology.

## Options, decision and consequences (before code)

1. **9/10, revised:** add a generic attested-affix witness to the existing
   grammatical relation table. A new marked relation requires BOTH center lemma
   membership and exact emission by that lemma's actual Hunspell flags and
   applicable condition/strip/add rule. Unmarked existing relations stay intact.
   No word list, runtime suffix/example branch or per-example weight.
2. **6/10:** close the report from current clean-output checks alone. This leaves
   the demonstrated lower-level authority hole masked by other layers.
3. **3/10:** add surface exceptions or common-word vetoes. Rejected: redundant
   per-word maintenance, authority inflation and violation of the systemic rule.

Selected revised1, subject to second independent review and actual RED. A bare
`у → ти` row was REJECTED after review1 (6/10): dictionary backing alone falsely
licenses найу/расу/гресу/почу from найти/расти/грести/почти. Flag membership alone
also fails: найти and нести share M but use different applicable rules.
These are source-inspected counterexamples, not yet executed RED evidence.

The selected form is a general backed inflection relation with an independent
grammatical witness, not a case-specific exception. It covers
same-stem forms such as несу/везу/ползу/трясу and prefixed counterparts whose
reconstructed infinitive is actually present. It does not claim irregular
stem alternations (иду, плету, жгу), all Russian verbs, or a new morphology
engine. A non-verb dictionary entry must not create an unrelated -ти lemma.

Reuse/extend the EXISTING Russian inflection-metadata owner currently holding
adjective classes; do not create a second cache/loader or use a full generated
surface dictionary. Read dictionary and affix text once during its existing
OnceLock initialization. Retain only (lemma,exact surface) witnesses for table
relations explicitly marked `affix`, using the existing generic suffix-rule
parser and matcher. No hardcoded `-ти`, `[сз]ти`, flag M or exception lists in
runtime. The data table may select a productive relation; the witness determines
whether it is actually licensed for each lemma.

Use a conservative finite witness-entry budget (4096 pairs); if exceeded, discard
this extra authority, preserving adjective metadata and prior behavior. Missing
or unsupported affix data also leaves new relations unavailable. The cache is
bound to the exact texts read at initialization, like existing adjective metadata;
no live file reload is introduced. A new process is required to use replaced
dictionary/affix files. Record actual cold initialization time, count and bounded
retained data in proof; this is not native typing latency. Reuse current warm-up.
No per-key filesystem reads, full generated reference authority, new timer,
queue, memory schema, model fit or learning change. Existing minimum stem length
and spelling guards remain. Main risk is false clean
authority suppressing legitimate typo restoration: test unknown stems and
productive positive/negative families, then the fixed affected source proof.
Do not weaken SafetyGate/verifier or erase L1.1 candidates. Runtime routes,
manual projection and GNOME/IBus layout authority remain unchanged.
Rollback is the data/test commit, without erasing user feedback.

## TDD and completion gates

Add tests against the real backed recognizer (not only broad dictionary/model
recognition), plus the public center/reference routes and real composite typo
source. Positives: several independent stems and real backed prefixed forms;
negatives: invented stems, missing infinitives, near-miss spellings and unrelated
infinitive families, especially dictionary-backed найти/расти/грести and the
nonverb почти. Hermetic dictionary+affix fixtures must distinguish identical
flags with different applicable conditions, missing/malformed metadata, failed
strip and unlicensed forms. A controlled bypass of the witness must fail those
negatives. Verify the finite budget and unchanged adjective classification.
Assert clean surfaces/absence of the competing candidate
and no automatic application in both first-word and multiword positions.
Keep actual typo recovery controls. The recognizer regression must fail before
the data relation changes; the current clean-output characterization may be
GREEN and must be labeled as such.

Use remote-only guarded test discovery with exact nonzero identities and
hermetic data; full affected library/correction contracts before review. Retain
class/aggregate restoration and clean-preservation results for the existing
fixed proof set; no partial selected-word success may override a regression.
One independent source-only review, maximum2 passes. Canonical graph/check,
then commit/push, exact public reply/closure and final user-authorized install.
Physical browser/native behavior NOT_TESTED until separately measured; the
historical349/51/32/432 matrix and old typing timings are not new evidence.

Plan review1:6/10, `/root/confirmed_second_handoff_plan_review_20261013`.
The same historical agent thread was reused due to the blank-thread limit;
only new task/source evidence informed this independent review. The bare-table
proposal is withdrawn; no production morphology edit has been made.

Plan review2/final:9/10. Keep live caller lemma authority, exact selected
(lemma,surface) witnesses, fail-closed unknown/malformed markers, stream before
the4096 unique-pair limit (4096/4097/duplicates/order tests), preserve adjective
metadata on affix failure and test actual IME policy/concurrent initialization.
No further plan review; implementation requires its own independent review.

Actual pre-change RED: `/home/ubu/.cache/lay/development/td148-red-uc3ormoc/RESULT.json`.
Three exact low-level tests selected: licensed forms fail (`несу` from `нести`),
unlicensed-lemma and absent-live-authority controls pass. This is the real old
recognizer, not a compile failure or a claimed native reproduction.

## First integrated proof and scope revision

`td148-green-22px12rf/RESULT.json`:9/10 selected tests PASS. All recognizer,
irregular/nonverb, parser,4096/4097, metadata failure/concurrency and actual IME
policy controls pass. The strict integrated test fails for `ползу` in full Nanda
mode: `layout_then_l2_word_center` selects English `gaspe`. It does not fail in
deterministic-only mode. This is a core-source result, not a native reproduction.

Authority path: live center lemma + exact affix witness → backed form succeeds;
deterministic composite proposal is absent; L2 layout_adapter's existing
`cyrillic_layout_word_center_blocked` retains only exact phase centers/protected
words and drops the new grammatical evidence; unrelated English settling wins.
First integrated loss is that existing source-preservation guard, before L3/L4,
DecisionCore/verifier. No SafetyGate/output change is justified.

Options for the newly demonstrated loss:9/10 extend that existing guard with
ONLY attested/live-center-backed grammatical evidence;7/10 propagate a separate
ranker contradiction record (larger change);2/10 weaken the test/disable Nanda
(rejected). Select9/10. Expose a narrow query for marked attested relations,
separate from old coarse morphology and decoder-only plausibility. Reuse the
same metadata and mandatory live lemma gate. Do not globally upgrade HotField's
decoder authority or add a second controller. Run all fixed clean/typo/layout
proof again; only a complete affected PASS permits code review/publication.

Full remote non-timing execution completed3189 selected tests with0 reported
failures across33 targets (3153 correctness +36 package). The command then
stopped BLOCKED_CONTRACT because the previously stale3177-entry manifest and its
zero-failure ledger had different hashes after discovery updated it to3215.
There are38 added identities, no removed or changed identities;10 are this task
and28 already committed tests omitted from the prior manifest. The existing
ledger permits ZERO exceptions and must continue permitting zero.

Receipt `/home/ubu/.cache/lay/development/td148-all-source-t6im7rni/RESULT.json`
retains the original blocked result; no failing test is quarantined. Finalize
the metadata contract by validating every exact selected identity against its
retained harness log and the unchanged tested source hashes, then rebind ONLY
the zero-ledger manifest hash. Preserve its old historical observation and empty
failure list. Record a separate finalized summary; do not overwrite the original
blocked receipt or rerun unchanged3189 tests merely to rebuild the report.

## Finalized source proof and measured cost

`/home/ubu/.cache/lay/development/td148-all-source-t6im7rni/FINALIZATION.json`
confirms3189/3189 exact identities from368 retained logs,957 tested source hashes
unchanged,33 targets,3153 correctness +36 package. Original command/summary remain
BLOCKED_CONTRACT; separate `SUMMARY-FINALIZED.json` is PASS after the metadata-only
repair. Its source archive SHA-256 is
`f6426ee86fcc257504d63fe26ab47158d0a2e16432f12a50539b5096fb8650aa`.
Manifest3215 includes10 new tests and28 tests already present in HEAD; no old
identity or lane changed. The zero-failure ledger's historical observation,
empty failure list and zero count remain byte-equivalent in meaning; only the
manifest binding changes. No known failure was added or hidden.

The fresh-process,8-concurrent-caller diagnostic measured71 exact pairs,
2018 UTF-8 payload bytes and232971 microseconds for shared cold initialization
with uncontrolled OS page cache. This includes existing dictionary/adjective
metadata work; it is neither incremental overhead nor native key latency.
`METADATA_COST.json`/`.log` retain the actual result. Warm-up remains in the same
owner, no per-key file read or generated-form authority is introduced.

All10 focused task tests pass; a controlled removal of the `affix` marker
causes the unlicensed-inflection negative test to fail. Pre-fix integrated
`ползу → gaspe` failure and genuine recognizer RED remain recorded separately.
The full affected source proof has zero failed clean/restoration tests; it does
not replace heldout quality percentages or physical-client acceptance. Native
new-version behavior NOT_TESTED and native timing NOT_MEASURED.

The user's installed public Hunspell DIC differs from the remote DIC. Five
targeted source tests were therefore run on the remote host with only that
public DIC mounted read-only inside the existing hermetic namespace:5/5 PASS,
including live-IME authority, clean first/later positions and irregular negatives.
`LOCAL_DIC_COMPAT.json` records DIC
`f6047416a0204adbecf3a451b874ec8a97ee37e2cbc714466ef04d8dbcc0d6fc`
and the matching AFF hash. No installed dictionary, model or runtime was changed.

## Independent implementation review1 and grouped repair boundary

Review1/2:7/10, `/root/issue46_feature_plan_review`, independent read-only review
in a reused thread because blank threads hit the platform limit. Two P2 findings
are accepted: `AF` alias numbers were read as literal flags, and the permissive
legacy suffix parser does not establish valid header/count/group membership or
Hunspell's prohibition of whole-lemma stripping without FULLSTRIP.

Before production repair, add real loader regressions for aliases, orphan/
incorrect/duplicate/count-mismatched groups and whole-lemma stripping. Selected
minimal repair: additional authority rejects unsupported AF/FULLSTRIP and
malformed suffix groups; attestation requires a nonempty post-strip stem.
Legacy generated-form parsing and adjective classification stay unchanged.
No new parser owner or generalized Hunspell implementation. Repeat the whole
affected source proof because production eligibility changes, then final code
review2/2; preserve the earlier full/finalized run as pre-review evidence.

Actual review RED `/home/ubu/.cache/lay/development/td148-review-red-_yqscpgd/RESULT.json`:
both new loader regressions fail on the old code (AF alias and whole-lemma strip).
After the grouped eligibility repair, `td148-review-green-apcwb1wc/RESULT.json`
is12/12 PASS plus the retained naive-relation witness-bypass expected FAIL.
The exact new loader validates declared suffix groups, counts, unique group
flags and cross-product markers before using the existing rule parser. AF,
FULLSTRIP and unsupported encoding/flags fail closed for added authority.
Attestation forbids an empty post-strip stem; legacy generated forms keep the
old application function. The full affected source run is now in progress.

Post-repair full run `/home/ubu/.cache/lay/development/td148-review-all-qgkgzc9t/RESULT.json`
is PASS:3191 selected (3155 correctness +36 package), zero semantic,
infrastructure or contract failures. The normal command now completes its own
strict zero-failure ledger validation; no report reconstruction is necessary.
Source archive `28dc7c90e4d2bf508f688806db0ff9f0b31ae7f2ae198b58e0cb47f8913125a1`.
Manifest has3217 entries, only the two review-negative identities were added;
all prior identities/lanes stay unchanged. Canonical refresh/exact decision
coverage,15 canon tests and existing122 Python harness tests pass in that run.
Final independent review2/2 is pending; native new bytes remain NOT_TESTED.

Final independent code review2/2:9/10, no unresolved P1/P2. Reviewer
`/root/issue46_feature_plan_review` checked the grouped repair and retained
proof/source parity without running tests or editing source. This reused thread
is disclosed above; implementation remained root-owned. Latest cold diagnostic
is229688us (71 pairs,2018 UTF-8 payload bytes,8 callers, uncontrolled OS cache).
No additional functional changes followed that review. Final canonical/document
consistency and publication follow; physical acceptance remains a separate gate.

Final canonical refresh/exact change-record coverage and15 canon tests PASS:
`/home/ubu/.cache/lay/development/td148-final-canonical-p8fdr16m/RESULT.json`,
48.01s, archive`fa277967281f4a8a9e1cf5ac7189ae0c6abc65fa208f822e900767e88ac89371`.
All functional files remain identical to the3191-test proof; only generated graph
receipt/documentation differ. Runtime lexical authority gains the bounded
attested evidence; installed runtime authority and IME delivery routes have
not changed. Publication is authorized by the sequential public-issue request.

Published05e1e448 to origin work branch, public work branch and public/main; all remote refs/readback verified. #59 CLOSED completed, reply https://github.com/radislabus-star/lay-public/issues/59#issuecomment-6211314244; /home/ubu/.cache/lay/development/issues-final-pass-20261016/CLOSED-59.json. No new installation/native claim.

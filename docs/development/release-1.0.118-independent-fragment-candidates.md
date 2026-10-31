# REJECTED source experiment, not the released 1.0.118 implementation

# Candidate118 — TD164 independent fragment candidate providers

Preflight 2026-10-30T21:38:21.382488+00:00. Baselinee65b63b0,current117source5e39d358...,1327files.117 was published at 2026-10-30T21:50:57Z; both annotated tags and actual installed updater checks were verified. Its immutable commit and byte-identical accepted KDE binaries remain independent from this source-only task.

Correction-only shared candidate semantics;field integrations frozen on both desktops. Existing registry/decision/owner/verifier/executor retained. Source map:
- src/typing_pipeline/candidates.rs:75,126 wraps early FAST and each registry callback.
- src/typing_rule_graph/rules/fragment.rs::apply_with_mode substitutes six old providers;on no result no old callback is consulted.
- src/typing_rule_graph/ids.rs +definitions.rs register existing sources;rules.rs exports callbacks.
- src/layout_fragments.rs +model.rs +data model are the already implemented method and stay byte-frozen.
- config/defaults.rs derives defaults from registry;config/pipeline.rs appends defaults and preserves saved flags/priorities. No duplicate config schema/menu change needed.

Evidence: exact117production CLI,isolated layout-only config,8inputs×OFF/ON,CROOT/td164-runtime-cli-diagnostic-v1/result.json. фиикум→abbrev OFF,ONnone;ordinary layout_ru_to_en candidate disappears. This is source-level reproduction,not a GUI PASS or proof every word is blocked. NoOpinion at an optional producer must not erase an ordinary candidate.

Options:9/10 separate two optional producers INSIDE current bounded registry,ordinary callbacks unchanged;reuses existing layout rule construction/safety/weights,keeps FAST shortcut with its real producer.6/10 serial legacy-or-fragment fallback:smaller but hides independently valid proposals and retains borrowed source confidence;must not choose without same quality proof.2/10 new pipeline/ranker/delivery route:unnecessary,rejected. Recommend registry separation. New rule safety must be explicit under existing policy;do not lower an existing safety gate or fit priorities/words to fixtures.

First semantic RED: actual core/ordinary candidate remains with flagON;actual geometric positive still available when old automatic source rules disabled;flagOFF with old sources disabled yields no new proposal. Use existing CorrectionRequest setup from examples/fragment_layout_proof.rs and real callbacks,not dummy success. Publish flag only in one process-isolated test. Keep current5provider tests' original protection/master/physical punctuation/short English requirements when migrating to independent providers.

Then narrow model/provider/core tests. Same fixed20000 common corpus OFF/ON,per-language clean preservation/restoration/wrong/abstain and lost-legacy targets;rawmodel40000 parity and speed remain separately measured. Reject required-class regressions,no word exceptions or weakenedtests. Full source/all10release/CI plus exactKDE Space first/second-word acceptance required. Native table stays historical until a full matrix.

Allowed runtime files exactly five above;test discovery and proof manifests are explicit metadata seams. All input focus/observation/ownership/preedit/Tab/Space/Backspace/DoubleShift/layout decoder and delivery files frozen. No GNOME installation or service restart. TD165menu stays separate.

Semantic RED: CROOT/release118-fragment-core-red-v1/result.json. Four real ordinary FullWave/DeterministicOnly corrections succeed with flagOFF,all disappear withON;independent positive hello also absent when old sources disabled. All5oldprovider testsPASS. Tupleleft([None×4],None,None) vs expected([abbrev,pkwy,hexyl,wundt],hello,None),exit101. Live/protected47files unchanged.

Candidate implementation: two new registry IDs;existing directional priorities/family/PlainLayoutAutocorrect policy,Normal required safety for the explicitly enabled heuristic source. Existing ordinary providers and their safety metadata unchanged. Six-source override removed;old FAST shortcut uses only actual old FAST provider,fragment proposals use normal shared ranking. Existing5unit controls migrated from dummy legacy injection to direct optional-provider semantics;master/protected/physical-edge/short-English checks retained. New actual core regression unchanged.

Focused production-core/provider GREEN: CROOT/release118-fragment-core-green-v1/result.json. Same actual core regression nowretains all4ordinary outputs andindependently returnshello withold6sourcesdisabled;flagOFFdisabled control remainsNone. Existing5provider protections pass after semantic migration. Newcore9.341s,old5unit0.066s,compile24.694s.47frozenfiles andalllivePIDs/config unchanged. Source returned under strict preimage/hash verification. Full fixed-corpus quality gate next;not installed/version not bumped yet.

## Variant A rejected after the fixed corpus — 2026-10-30T22:22:21.820359+00:00

The private quality worker reports GREEN against OFF only. That status is insufficient and is explicitly rejected as promotion evidence. Source-only results retain ordinary correct targets but regress against the accepted ON configuration. No118 version bump, installation, runtime restart or publication occurred.

| Language | Baseline | Clean preserved /10000 | Restored /10000 | Wrong | Abstain |
|---|---|---:|---:|---:|---:|
| RU | accepted OFF |10000|390|97|9513|
| RU | accepted ON |10000|693|0|9307|
| RU | variant A ON |10000|406|97|9497|
| EN | accepted OFF |9979|2964|105|6931|
| EN | accepted ON |9987|2961|0|7039|
| EN | variant A ON |9979|2965|105|6930|

Evidence: CROOT/release118-fragment-quality-v1/result.json and proof.log; accepted ON: CROOT/fragment-layout-release-v2/proof.log. Fixed heldout SHA89d13ce432d06b479fefb48d6f8b39b49ec2c75ca9d51dca76fd0823c9fb1c26, unchanged model SHA28dc04cccc131c0b78167758982402ae519509d98bc9f0abcff22782286cdca9.

Promotion must satisfy BOTH baselines conjunctively: RU clean>=10000,restored>=693,wrong=0; EN clean>=9987,restored>=2964,wrong=0; no lost correct ordinary targets; OFF exact parity. No fitted word exceptions, priority changes or weakened gates. This is a correction-core corpus, not native field acceptance.

Next diagnostic map: correction_core/candidate_sources.rs::collect_typing_assist_candidates_with_pipeline -> typing_pipeline/engine.rs (keep_fast=true) -> typing_pipeline/candidates.rs -> typing_rule_graph/rules/fragment.rs and weights.rs -> common candidate selection. The core already retains FAST for later collection; changing the explain-only early return is not a demonstrated remedy. Inspect raw proposal, syntax/safety result and score before another production edit. All47 protected source hashes and installed117 runtime remain frozen.

## Diagnostic and variant B

CROOT/release118-fragment-raw-diagnostic-v1/raw-proposals.log records8raw cases. The private diagnostic executed one exact libtest successfully (1passed/0failed); its wrapper failed parsing interleaved --nocapture output, so its FAIL receipt is preserved and is not functional acceptance. Source was restored before executing the private diagnostic. Fresh readback confirms installed117PIDs/config unchanged.

First demonstrated loss: geometric proper pamelia and бесстрашиях proposals exist, syntax accepts, but PlainLayoutAutocorrect rejects the new sources before collection. That policy is the dictionary-only plain-word risk check, not the existing contextual-evidence policy. Variant B assigns only the two NEW optional source definitions ContextualLayoutAutocorrect, already used by contextual layout sources; all existing definitions, safety functions and downstream admission remain frozen. No weight/priority/model change. Extend the real independent core test with ordinary sources disabled for ghbdtn→привет and зфьудшф→pamelia; require full dual-baseline quality, not just targeted positives.

## Variant B rejected against preservation gates

CROOT/release118-fragment-quality-v2/result.json:6focusedPASS;fixed20000 proof293.367s,40000trainerparity. RUclean9977,restored9809,wrong19,abstain172; ENclean9832,restored9507,wrong0,abstain493. No lost correct ordinary targets;OFF remains exact. Restoration improves strongly but required clean/wrong gates FAIL, so NO install/version bump/publication. Representative RUclean errors include7/8known Russian forms;RUremaining wrong examples retain ASCII physical edge letters in FAST while exact fragment target is retained and eligible. Need independently inspect existing original-word authority and common candidate score trace before another production edit. Do not fit model or ranking priorities to these examples.

Private identity/score diagnostic CROOT/release118-fragment-identity-diagnostic-v1:complete1exacttestPASS10.849s,source restored,47frozenfiles/PIDs/config unchanged. Raw clean-word model votes:RU26,existingoriginal authority protects23;EN160,original authority only16. Merely guarding known original words is insufficient for English. Among four RUphysical-edge score traces,partial FAST and full fragment remaineligible;FASTdecisionranks611/647/677 vsfull492. No rank/weight change is justified. Next measure existing target grounding on the same full corpus before adding producer guards; all NoOpinion cases must retain ordinary source proposals.

## Variant C preflight: reuse established lexical/context evidence

Target diagnostic CROOT/release118-fragment-target-diagnostic-v1:one exact isolated testPASS12.308s;fixed corpus unchanged. Existing RUtarget authority covers8139/10000words;existing English word bank3038/10000. Of raw clean ENmodel mutations,source+RUtarget knowledge leaves36:32short targets<=3characters and4long targets. Existing typing_context::should_enable_ascii_to_ru_layout is the actual lexical/context admission used by ordinary Normal ASCII layout;new providers skipped it entirely. This explains the unsafe Contextual policy exposure. Do not invent tuned thresholds or replace the model.

Planned C production seams remain the same5files,with new logic only in fragment.rs and existing candidates.rs FAST preparation: (1) new provider respects existing user protection,word_recognizer identity and Russian layout surface authority for original Cyrillic words; (2) Russian target requires existing Russian layout surface authority;short Russian targets use the existing public contextual admission;existing English short-target guard remains; (3) optional evidence may reject a FAST proposal only for positive original-word/English-orientation evidence,or a positively selected full physical projection that completes the same half-converted candidate. Unavailable/Ambiguous/Special/NoOpinion must leave the ordinary proposal intact. Common L1.1 retention,all other ordinary callbacks/ranker/DecisionCore/safety/verifier/executor remain unchanged. Source-only frozen47hashes still apply.

Before C production edit,add actual core RED for recognized original words/short unfamiliar ASCII and physical edge completion,with the old-source retention/independence assertions unchanged. Then exactsame core GREEN and dual-baseline fixed corpus. Reject if any clean/restoration/legacy-retention gate fails. No automatic promotion from representative positives.

C short RED follow-up: core-green-v2 preserves all7original/short controls but still fails full physical projection. Bounded real core witness in core-red-v3 proves surviving source is contextual_layout_en_to_ru,not FAST. Existing physical-edge provider test also catches lf.→даю being rejected by the new short guard. core-red-v3 wrapper FAIL correctly remains (4/5old testsPASS);do not count this as a completed GREEN.

C2 refinement before edit: apply the same positive evidence check to ordinary EN→RU registry sources (LAYOUT_EN_TO_RU,CONTEXTUAL_LAYOUT_EN_TO_RU,EXPERIMENTAL_LAYOUT_EN_TO_RU) as FAST;all other callback generation unchanged,NoOpinion alone retains proposals. Reuse existing ascii_layout_prefix_can_be_letter for physical-letter symbols,not a new literal punctuation list. Existing short-context guard must preserve explicit physical-key signals. For Russian targets lacking canonical lexical authority,allow the existing model Known flag only together with such a physical signal;ordinary ASCII-only unknown targets remain ungrounded. No score/weight/model/field-route change. Same exact core and5original provider guards must allpass before fullcorpus.

C3 first failed predicate is now measured: core-red-v4 witness for source ,ehf[ reports selectionRussian/Known,physical_signal=true,but original_word_authority=true. Word recognition trims the physical edge keys and incorrectly transfers the inner ASCII word authority to the complete physical token. Protect exact source identity;when physical symbols are present,ordinary dictionary authority for a trimmed ASCII substring is not authority for the whole source. Explicit user protection remains authoritative.

Refinement: preserve source ASCII dictionary identity only for the complete token or tokens without physical-letter symbols. For unknown Russian targets,the existing Known+physical exception is limited to LEADING physical-key evidence,matching the already accepted strong prefix signal;ordinary sentence-ending punctuation alone cannot create this exception. Complete-projection contradiction also needs a known full target or that leading Known physical signal. Add a literal sentence-punctuation retention control. All five old provider controls remain mandatory.

## Scope decision — 2026-10-30T23:24:11.887735+00:00

Independent provider prototype A/B/C never passed conjunctive quality/old physical/punctuation guards and was never installed/published. It is archived as CACHE/td164-rejected-independent-providers.patch with inventory,all raw receipts retained. All five runtime files and test discovery restored to exact117 before the smaller fix. Do not treat removal of unaccepted experimental tests as closure of independence: TD166 explicitly retains that unmet contract. Existing public five provider tests remain byte-identical. Next118 scope is ONE production file: safe exact ordinary fallback on optional abstention. New owner release-1.0.118-fragment-safe-exact-fallback.md and ADR2026-10-31-fragment-safe-exact-fallback-118.

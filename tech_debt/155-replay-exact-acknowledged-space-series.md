# TD-155 — Replay an exactly leased series of acknowledged Space boundaries

Status: SOURCE_NATIVE_INSTALL_DONE_1.0.89, publication pending; final source review9/10. Priority: P1, next consumer after TD154. Root implements;
fresh-context plan and code reviews, at most two passes each. Authoritative
checkout /home/ubu/projects/lay-space-boundary-shift-20261007, branch
codex/space-boundary-shift-20261007, baseline76361affeef7bb0cb9fbe35c1bf9293137b7156b.
Baseline at task start was installed/published1.0.88. Current1.0.89 is installed;
publication is pending. Exact final scope and immutable receipts are below.

## Root cause, measured evidence and authority boundary

TD154 source3211/3211 and actual optional Firefox normal40 are different scopes.
The first new-byte native40 is25PASS/12FAIL/3BLOCKED_PRECONDITION; do not splice
it into the old complete default432=349PASS/50FAIL/33BLOCKED, NOT_TESTED0.
In rich field7 rapid10 the actual trace now confirms BOTH owned Spaces, consumes
the current exact receipt and delegates ManualToggleV3 status3. The next daemon
log rejects the plan: `exact replay requires preserved ASCII spaces or one
acknowledged Space`. The retained raw word ends in two U+00A0 characters.

Actual first remaining rejection: prepare_exact_replay in
src/bin/lay_daemon/exact_ime_tail_replay.rs. Its existing branch permits one
NBSP in the source lease, re-encoded by the existing physical ASCII Space key.
The existing planner already preserves the separator COUNT in its replacement.
Two/mixed acknowledged Space boundaries are deliberately rejected by the old
one-character consumer guard, despite the exact current lease. TD154 repaired
the producer; keep that repair. No delivery-route defect is demonstrated here.

Immutable evidence under /home/ubu/.cache/lay/development/td154-install89-20261016:
- native-shortcuts/firefox/RESULT.json, complete40; other new modes NOT_TESTED.
- native-shortcuts/firefox/firefox-shortcuts89-hi06zvxw/ime-trace.jsonl,
  SHAe532cd41ac37b38780243c265a9310edc3cfba2f9d2646c3038ed6f599bdb4e8,
  retained lines24/46/57/59; bounded trace completeness UNKNOWN.
- firefox-optional-daemon.log,
  SHAe62067fd5d875d8191b97930dc4752dde8b3f06d56253ae4261c2dfc1fc46d97.
- BUILD.json, source76361aff, archive4088d1308056524d5f74643603f0f909d7fc05967dfe61e80f1b85aa2fc04613.
No new publication, installation or default-matrix success is claimed.

## Alternatives and recommendation

1. **9/10, selected:** extend only the existing exact-replay consumer from one
   acknowledged Space to an exactly leased series of U+0020/U+00A0. Require
   literal projection of the exact original token and exactly the same count
   of existing ASCII Space key outputs. Use this uniform exact validation for
   ASCII and NBSP series; zero boundaries must remain supported. This replaces
   the weaker ASCII-only ends_with branch without changing valid planner output.
   Keep the original raw lease unchanged,
   exact suffix/prefix identity, source/route, all counters/flags, event-text
   round trip, bounded batch, current lease/focus checks, modifier hygiene,
   suppression pairing, backend authorization and verifier. No extra state.
2. **4/10:** continue rejecting multi-Space rich clients and document a limit.
   Smallest code cost but leaves the implemented opt-in Double Space feature
   unavailable in the observed fields. Accept only if the exact contract cannot
   be proved without broader owner work; do not pretend the feature works.
3. **1/10:** normalize arbitrary whitespace or add another client/backend route.
   Reject: erases source proof/count distinctions and increases regression scope.

## Minimal TDD and safety contract

Before production edits, extend the existing exact-replay regression and obtain
an actual RED at the multi-NBSP guard with all old single-Space controls passing.
Cover both layout directions;1/2/4/10 boundaries; pure ASCII, pure NBSP and mixed
ASCII/NBSP; external Unicode prefix; exact output, event round trip, delete and
insert counters and daemon authorization. Assert refusal for tabs, narrow NBSP,
newlines and other Unicode whitespace, wrong projected word, missing/extra
output separators, changed exact suffix/prefix, stale/misclassified lease,
wrong route/source/layout/suppression flags, inconsistent edit fields and over-
budget batches. Keep all old negative checks except the two/mixed NBSP cases
explicitly promoted by the new decision; never widen other whitespace support.

Extend an actual IME callback regression through exact suppression, real
Backspace/printable/Space press/release callbacks, Reset and raw SSI receipts.
Seed every original separator through its own actual owned Space callback and
exact client receipt, not by directly assigning a trusted tail. Retain existing
single-boundary cases and selection/foreign-prefix/epoch/expiry/unpaired-scope
negatives. Verify the replayed separator series and authoritative next manual
tail; source-only planner success is insufficient. Distinguish per-key client
receipts from a batched final receipt. Original owned Spaces require their
individual adjacent receipts. A newly emitted exact replay uses the existing
current paired completed callback witness for ALL of its trailing Space keys;
without that full witness, batched presentation changes remain refused. If that source path exposes another owner
loss, record its first failed transition before proposing another runtime edit;
do not pre-emptively normalize the final snapshot or weaken the verifier.

After causal RED and before runtime edits add an explicit decision under
docs/architecture/decisions/ for this old single-boundary contract change.
Affected invariants C01/C03/C04/C05/C07/C08/C09/C10. Runtime authority stays with
the same exact leased-tail executor; only the representable acknowledged series
expands. Canon/guard rules, delivery routes, source selection, data/models,
learning, config/defaults and isolated replay bounds remain unchanged.
Record each experiment in docs/development/td121-core-experiments.md.

## Verification, publication and completion

All executable work remains on e@192.168.3.94 under the existing shared resource
lease; no RDP, local functional input or global IBus restart. Close the owned
finite Firefox test seat after the failed immutable run. Reuse reviewed native
actors unless actual new evidence requires an explicit observer revision.
New stage paths must preserve the first failed candidate/receipt. No retry of
unchanged bytes until PASS, no partial receipt splicing, no new model training.

Focused affected producer/consumer/adverse tests, controlled reversion of ONLY
the new consumer expression, full release source/MSRV/lint/canon/graph gates
with unchanged manifest classifications and zero exemption, then fresh-context
code review scored1–10 (max two passes). Native: new matched bytes, first a
bounded exact affected field diagnostic if necessary, then complete default
Firefox/Chrome432 and separate optional normal80, all raw primary receipts,
actual next-letter/caret/mode checks and latency median/p95/coverage. Keep optional
private/fullscreen NOT_TESTED when unrun; preserve missing timing samples.
No lost old default PASS is allowed. First-attempt source-main CI/security,
reviewed release notes, normal commit/push/release and reversible exact-payload
installation remain required before claiming a shipped fix. If further native
failures remain, record them by first shared mechanism; do not relabel them.

Rollback: retain installed1.0.88 and its matched payload/config backups until the
new release gates succeed. Retain TD154 as a proven producer repair; no automatic
reversion of earlier source progress to obtain selected native passes.


## Causal unchanged-production RED

td155-exact-series-red-lggpkxju/RESULT.json:5/6PASS,5.310s,
archivedf8eb00b703b1be766b1bfd2e108b6b16b363ace80e70291810ed040d9e62b4d.
Actual existing prepare_exact_replay regression rejects two NBSPs with the exact
old guard message after all earlier one-boundary/preserved-ASCII cases pass.
Other five existing executor/source-order/adverse/bounds controls PASS. No
production edit caused this RED. Existing manifest testID retained.

Fresh-context plan review9/10 ACCEPT, pass1/2, reviewer
/root/td155_space_series_plan_review. Uniform strict ASCII/NBSP validation and
zero-boundary controls made explicit; no blocker. New decision
2026-10-17-exact-acknowledged-space-series-replay.json precedes runtime edits.
Consumer implementation uses the same owner/lease/executor; actual multi-boundary
callback proof is now extended before any new build or native input.


## Actual replay-stream RED and bounded scope correction

td155-exact-series-focused-9akk5_kf/RESULT.json:11/12PASS,17.342s,
archive89cb69224625aceefea4a98f50cea9a97ea48c4bdff4b8101ba03924da49f4bd.
All six actual daemon tests now pass, including exact projected-word/count
negatives. The actual callback test passes old one-Space controls then loses
final exact manual authority for a completed replay with two observed NBSPs.
Other producer/replay/provenance/adverse tests pass. No new native run occurred.

First remaining source transition: acknowledge_owned_space_boundary forms an
expected target by replacing only the final Space of the completed ASCII replay.
Its exact-prefix check therefore rejects the actual two-NBSP target. The same
existing acknowledge_owned_space_tail mirrors only that last character in the
completed paired replay scope. This is a new measured loss, not an assumed
client result. Keep this failed source receipt unchanged.

Minimal amendment for plan review pass2/2: when and only when the EXISTING
current paired COMPLETED exact replay independently proves every replacement
key and complete epoch distance, its existing first adjacent client receipt
may acknowledge U0020/U00A0 presentation at each Space position in that newly
emitted trailing series. The target word, unchanged owned prefix and full
external prefix stay byte-exact. Count/cursor/no-selection, token, revision,
lease, shared-owner pair, freshness and completed provenance stay mandatory.
Both raw tail and the SAME completed scope record the actual full acknowledged
replacement. No new state, owner or delivery mechanism; no acknowledgement of
arbitrary historical separators or incomplete/unpaired/stale replay. Ordinary
non-replay owned Space continues to change only the new final character (TD154).

Before this amendment's runtime edits, add a second explicit decision covering
observation.rs and tail_memory.rs. Extend existing actual callback positives
with ASCII-only/final-NBSP/full-NBSP/mixed targets for1/2/4/10 newly emitted
boundaries. Include extra/missing/non-Space separator, changed projected word,
changed old owned/external prefix, selection/interior caret, extra epoch, expiry
and missing/mismatched pair. Require complete effects before batched receipt;
incomplete replay cannot use the series rule. Keep original one-boundary cases
and their exact negatives. No generic Unicode normalization or verifier change.
Control-revert the consumer expression and then this completed-series receipt
change independently; each must detect its own intended regression. Full source
gates run only after both actual transition controls pass.

Fresh-context amended plan review9/10 ACCEPT, pass2/2. Existing completed pair
must be revalidated, not merely present in pending metadata. Original owned
Spaces need adjacent individual receipts; a new exact replay instead proves all
newly emitted Space keys via its completed callback sequence before final SSI.
Second explicit decision2026-10-17-completed-replay-space-series-acknowledgement.json
precedes protected owner edits. Root implements; no further plan review passes.


td155-series-green-9qqzqfio/RESULT.json:12/12PASS25.671s,
archive1dc6f8200beb96ac4fd2ec432df723f5c243cb36c406ca115b4f658365f7ed89.
Actual consumer covers both directions,0/1/2/4/10 separators, exact prefix and
strict literal/count/flag/Unicode/batch negatives. Actual IME callback matrix
seeds source separators individually and accepts complete raw ASCII/final-NBSP/
full-NBSP/mixed targets; same epoch and exact local/shared/scope equality verified.
Original adverse status0 stays strict. New contradictory targets may refuse
inertly as0 or request exact snapshot4, with no text effects/delegation and no
authoritative handoff; these are the documented existing outcomes, not a new
runtime allowance. Incomplete, scope mismatch/prefix, wrong word, spacing, caret,
selection, expiry and extra-epoch contradictions stay safe. Full gates pending.

Fresh-context independent code review9/10 ACCEPT, pass1/2, no P1/P2,
/root/td155_space_series_code_review. All18 new helper identities match;
normalized changes from reviewed TD154 helpers are only new immutable
phase/seat/path/filename identities; predicates and stock input paths unchanged.
No extra review pass without a new change/finding. Full source gate in progress
at td155-full-source-wstyurem; both independent causal controls detected their
expected regressions. Native/build/CI/install still pending.


Full td155-full-source-wstyurem FAIL/BLOCKED_CONTRACT,716.943s,
archive24a70963653b38ca6393ec2d23df185de9508db16698262e84e56dc97d1b2ded.
All3237 IDs unchanged, both controlled REDs detected, MSRV/lints/canon/audit/Node
passed, but actual IME827 had2failures. No waiver or installation. Existing
firefox_completed_boundary_replay_waits_for_final_exact_receipt and
td121_inherited_full_snapshot_proves_closed_replay_without_source_echo fail
with an unchanged owned leading prefix. First mistake: the new outer candidate
contains the WHOLE owned tail, while pending receipt bounds only its token
SUFFIX. Passing the whole tail to suffix geometry adds the old owned prefix
length and correctly refuses. Source/physical scope never became accepted.

Minimal correction preserves the distinction: require current full tail to
end in the exact pending token, retain that owned prefix byte-for-byte, compare
only the token suffix against receipt geometry, and pass the recombined full
tail to its existing atomic owner. No guard is removed. The existing owned-
prefix regression now also covers2boundaries through actual individual source
receipts; original count1 cases/negatives remain. Both historical failures join
the focused target set before another full gate and code review pass2.

Corrected actual focused td155-prefix-focused-jnf0564c:14/14PASS30.197s,
archivedf903f2ab8bd863f35763390593627cdbe3a688794b3f6a2c9df4628342dc500.
Both old owned-prefix regressions pass; the older completed-boundary test now
also covers two ASCII/NBSP source/target boundaries with exact individual source
receipts and its original prefix/selection/caret/focus/content/off-caret negatives.
No old assertion was weakened. Receipt token suffix and full owned tail remain
distinct; full-tail exact prefix check is stronger than the mistaken draft.
Next full includes a third isolated counterfactual restoring that draft's
full-tail/suffix confusion and detecting the exact old count1 failure. Full
source and final code review pass2 remain required; no build/native/install yet.


Final full source td155-full-source-final-g48ali01/RESULT.json PASS3211/3211,
3175correctness+36package, zero failure/exemption,730.988s,
archive47036521eca9c87863dad30218067923f74547225242c59e6564a680137390bc.
All3237IDs/classifications unchanged; actual MSRV1.88 both configurations, strict
lints/unchanged semantic baseline, canon15, architecture/audit50, Node/harness122
(one explicit skip), panel syntax pass. IME827/827, daemon294/294. Three isolated
controlled regressions detected, exact source restored each time. All994 tested
functional/generated inputs match local after verified metadata sync; tracked
graph copied from remote. Final fresh-context code review pass2 and canonical
parity/observer qualification next; new native/CI/build/install still pending.

Final canonical td155-final-canonical-rwecv4fk/RESULT.json PASS48.657s,
archive9cd6b1765b3b1e1cce8f1386c73146c2677722130309c2ccd483ec76cbfd5634.
All993 functional inputs match successful full source;18 private helper hashes/
ASTs and actual remote observer predicates17/17 pass. Zero native keys; not a
client acceptance claim. Tracked graph/generated receipt copied back; no runtime
code changed after the successful full source gate. Final code review2 pending.

Final fresh-context code review9/10 ACCEPT, pass2/2 total, no P1/P2, reviewer
/root/td155_space_series_final_code_review. Corrected suffix/full-tail distinction,
all current owner/provenance guards and18 helper identities independently checked.
No runtime code changed after final full source; no third review pass. Source
task complete; matching new-byte native/CI/release/install gates remain separate.


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

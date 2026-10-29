# KDE Firefox: owned append is visible before its caret update

CURRENT: candidate IME3881298 SHAa286a257cdeb584ff50fa2477baf204002e16c93b64a3dc70338bd78a6235e28
installed on remote KDE:71; source1307files SHA122f3cf1ff0ac8f277bf0c3f1b470ae899c11f680a0c25d2c52d3743b5a519ce.
Source1134/1134 PASS,0warning74.372s release. Native short plain/rich2/2PASS,
16 gesture/decoder targets. Both8-gesture row traces rotated beyond512KiB,
so the absence of a new trace event is not causal nonexecution proof.
Retained22 COMPLETE20PASS1FAIL1BLOCKED: plain second post-Tab toggle fails after
its successful first target/decoder probe; URL passes this run. Causal marker
not observed in retained complete row traces; no native caret-fix claim.
Historical full440 counts unchanged; no release.

Preflight before source regression/implementation. One active task, solo,
without Harness or agents. Canonicalbaseb835ca478198d6cb765a2246329d5646966c118c,
branchcodex/browser-matrix-repair-20261027 in lay-dependency-updates-20261027.
Preserve installed KDE daemon2374910/ae77... and its verified native-grab fix.
No local GNOME installation/config change, new owner or output/delivery route.

## Measured failure

Measured baseline diagnostic IME3669264/0a26d84e418a495b1fa13d91ed6a1745cf3a453cd3e0d17b85641bed8ff374f4
has retained22=19PASS2FAIL1BLOCKED. Its prior empty-preedit experiment is not
promoted: the new branch never ran in short2 or retained22. That production/test
overlay was withdrawn from local sources by checked hashes, archived with all
source/native failures. Current remote source and installed runtime remain that
diagnostic5df213... source until the pinned next overlay/replacement.

First failure chosen: plain second-word Tab. Before the second word, first word
managed CommitText succeeds for its sixth letter. Existing confirmed receipt is
advanced by the owned append. Firefox reports actual full6-char text/caret5,
then key release, Reset, then full6/6. At the first SSI,
observe_context_reset_rereceipt_surrounding_text rejects
second_surrounding_receipt. Reset then has no predecessor. Second word is
literal preedit without a live start witness; visible hint exists but Tab is
unhandled. No Double Shift has occurred in this failed row.

Raw CROOT/kde-firefox-empty-preedit-native-retained22-v1/firefox/row-15-native-trace.jsonl,
complete row anchor, rows645–690. Cached evidence archive
empty-preedit-native-retained22-v1-evidence.tar.gz. The separate URL first-toggle
failure is not part of this fix without shared causal evidence. Whole-run
trace rotation is not a complete transcript. Keep full440 matrix306P55F79B.

## Existing map and smallest change

```text
managed input CommitText (UNCHANGED)
 -> observation.rs::advance_context_reset_rereceipt_after_key
    existing confirmed + awaiting_owned_append_receipt
 -> actual observe_external_surrounding_text
    -> kde/firefox/preedit_echo.rs: exact client/presentation predicate
    -> observation.rs::observe_context_reset_rereceipt_surrounding_text
       existing strict_prefix_echo_revision retains inert adjacent progress
 -> authenticated Reset / actual whole field with final caret
    -> existing reset receipt and manual/Tab admission (UNCHANGED)
       -> existing verifier/executor/output (UNCHANGED)
```

Prefer9/10: reuse the existing inert strict-prefix slot for an authenticated
owned append whose actual text arrives one caret position early. Require the
previous whole actual field, exact single locally appended character, identical
external prefix, unselected old caret, same live token/owner/epoch, KDE/Firefox.
It grants no edit/Tab/Space authority until the adjacent full actual field
settles the caret; an authenticated Reset follows the existing inert lineage
contract. Repeated/interior/moved/foreign/contradictory observations must not
recreate a receipt. Existing ordinary strict-prefix and GNOME semantics stay.

Alternative wait/retry3/10 cannot repair discarded provenance. Broad
normalization or native-text re-projection2/10 violates exact source/ownership.
No new receipt type, generation, timer, queue, RPC or client-specific executor.

Allowed current runtime seams: observation.rs and existing KDE/firefox
preedit_echo.rs only. New tests use existing owned literal-key, actual Reset and
client observation fixtures in active_composition_space_receipt.rs. Canonical
manifest and generated graph update through the existing remote pipeline.
The included native_cursor_observation.rs was restored exactly to its baseline;
it is not needed by this task.

## Gates

Genuine source RED on the original production path, specifically the measured
full-text/old-caret SSI followed by Reset/full-caret SSI; then focused positives
and independent contradictory/identity/client negatives, full affected IME
source lane and canonical graph/manifest. No old assertion weakening. Build
zero-warning IME only, match loaded bytes and preserve all other processes and
config. Native short must show the new causal predicate, then retained22 must
have no FAIL before unchanged fullFirefox220. Keep source, runtime and physical
results distinct. Native source-free/user-profile claims require their own
receipt; a fresh owned profile cannot certify the user's existing profile.

RED-v1 did not compile: the new fixture passed an immutable Harness reference
to the existing mutable known_engine helper. Corrected only that call; this
setup failure is not causal RED and no product implementation has changed.

RED-v2 is genuine:9 selected8PASS, only the measured caret-echo retention
assertion fails.15.02s compile, source1307files
SHAbe38f2e0d097ac80fa822a02c519b076626f69e64b0c71afe5a974164feb59fd.
Receipt local owned-append-caret-source-red-v2/result.json. Implementation now
reuses strict_prefix_echo_revision and awaiting_owned_append_receipt; no new
receipt fields/types. KDE leaf owns exact one-character/full-context cursor
predicates. Normal observations perform no new native window read; only those
exact candidate/settlement shapes select the existing KDE/Firefox client check.
No output/admission changes; focused GREEN/negatives are pending.

SHORT-v1:10/11 passed; the positive measured-order case still fails at retention.
Do not relax guards. Add bounded fixture-state facts to its assertion to identify
the first unmet existing condition, then correct the mechanism or fixture from
that evidence. Negative-only PASS without a positive is not acceptance.

SHORT-v2 confirms modeManagedCommit, correct active path/capability/live token,
but previous_snapshot=None. Owned CommitText deliberately invalidated that
current snapshot. Preserve this contract. Existing before-key candidate has an
owned_preedit_return_snapshot slot for a real observation; carry that original
full field through one successful local append into one optional inert snapshot
inside the existing PendingContextResetRereceipt. No new owner/type/cache or
current-snapshot restoration. Spend it at the first actual callback. Multiple
local appends without an intervening exact field cannot reuse it. Whole field,
original revoked predecessor, epoch/revision and next local append remain bound.

This necessary data field requires None initialization in the same two included
native_cursor_observation.rs constructors. Their actual cursor logic stays
unchanged; this precise mechanical seam is redeclared before editing. The prior
unaccepted field/type is not restored. No new implementation has been installed.

SHORT-v3 remains10/11. Full producer map identifies the absent link:
`process_legacy_key` captures boundary_candidate only for Space/Tab or
Backspace, not the ordinary literal key. Extend that existing inert capture to
accepted KDE managed key presses with an already confirmed observed predecessor;
the existing after-effect append/epoch checks decide whether it can be retained.
No key is intercepted or delivered differently. The actual SSI selector also
must use this inert pre-effect observation after the current one is invalidated.
Expose its existing pure shape selection as one testable internal method and
exercise it before the receiver; native acceptance must still observe the real
new causal trace, not just a passing source-only injected client flag.

SHORT-v4 now passes actual SSI selection and inert retention. Two failures were
the same over-constrained test assertion: context_reset_rereceipt_computation_allowed
intentionally permits read-only candidate computation with an unconfirmed
current identity; it is not mutation permission. Preserve that existing function.
Replace the new assertions with all actual manual grant alternatives plus
zero client text effects; the predeclared no-edit contract is unchanged. The
duplicate negative had stopped at this computation assertion before injecting
its duplicate, so it did not yet demonstrate a duplicate-retention bug.

SHORT-v5:11/11 PASS,8 retained controls plus3 new tests. Actual selector and
receiver are both exercised; positive includes Reset/no-Reset orders,14 refusal
variants reach their real final observations, and6 Unicode/external-prefix
predicate cases. No old assertion changed; candidate computation is preserved,
all actual manual grant alternatives and zero client text effects are asserted.
Source1307files SHA190aa8a3b797b6bb9137518f90059ff5d2923fa9a12bd55a773d2e551debefef.
Next full IME source/graph/manifest, zero-warning build, matching idle factory,
then actual native causal trace and unchanged retained22. No new installation.

GREEN-v1:1134/1134 affected IME correctness/package PASS, short11 overlaps;
graph/architecture PASS, exactly3 new test identities added and no old rows
changed. Three performance tests not executed. Final source1307files
SHA122f3cf1ff0ac8f277bf0c3f1b470ae899c11f680a0c25d2c52d3743b5a519ce.
Receipt local owned-append-caret-source-green-v1/result.json. Baseline-relative
review owned-append-caret-source-review-v1.json finds no unexpected prior src
changes and byte-preserved prior test source. Review performed solo, not an
independent agent review. Next0warning IME-only release and matching idle
activation; preserve daemon/IBus/tray/user Firefox/config. No native verdict yet.

Build/installation completed: same source, zero warnings,74.372s; loaded
engine3881298/a286a257... replaces the earlier unaccepted3669264/0a26 candidate.
Idle OwnsActiveText=false and no visible hint before component activation;
daemon2374910/ae77..., IBus3168326, tray3228081, user's Firefox988548 and protected
desktop processes retain start/hash. Config6e62eab3... and selected source/layout
preserved; no global IBus or local GNOME restart. Activation/installed backups
and loaded-byte checks in CROOT/kde-firefox-owned-append-caret-deployment-v1/.
Native short2/2PASS,21.809s; incomplete row traces explicitly block causal claims.
Retained22 is a predetermined broader diagnostic, not retry-until-green.

Native retained22:20P1F1B/72.337s, protected process/config/source state preserved.
Plain Tab17.680ms and first toggle204.765ms succeed; second reverse after its
letter/Backspace probe fails. Complete row15, same first failure as earlier
pre-empty-return runtime. Source/root hypothesis remains unpromoted physically.

One fixed passive-wire plain-only diagnostic1/1PASS/13.836s, same frozen actor,
no extra active RPC. The first natural VisibleTailV4 replies show the whole
before/after target with U+00A0 retained, so a universal ASCII/NBSP mismatch is
not established. Passing isolated plain is not passing the preceding22-state
sequence. No code change based on this single success.

Next diagnostic-only change: extend the existing opt-in metadata at
capture_legacy_preedit_start_boundary. Record direct primitive facts about
captured witness/predecessor, current/pending KnownStart, pending confirmation,
actual full-tail equality and whitespace classes. No source text, new I/O,
RPC, timer, owner, mutation grant or input/output routing. Avoid locking/validating
getters inside diagnostic emission. This will distinguish missing source proof
from presentation or callback-order hypotheses before any further functional
change. Keep all FAIL/PASS evidence. Incomplete8-gesture traces mean earlier
"new branch never ran" wording was too strong; only nonobservation is proved.

Diagnostic SHORT-v1:11/11 PASS. Metadata must also report exact-refresh=false
instead of filtering that state out, otherwise lost client capability is
indistinguishable from an uncalled producer. KDE-only, privacy guard and direct
primitive reads remain; no new I/O. Full affected source/graph next.

Diagnostic GREEN-v1: short11PASS, full1133/1134; old native-transfer test
td121_pending_cross_profile_reset_or_content_type_retains_source_free_unknown_request
fails at native_transfer.rs:58 (establish_known_source, before the tested loss
scenario). No diagnostic build/install permitted by that result. The actual
test/assertion is unchanged. Add that exact case to the next focused selection.
Review found trace::enabled refreshes a mutex/config file; use existing atomic
diagnostics_enabled_cached after KDE/closed-tail eligibility instead. This
prevents a new global config read at this source helper. It does not establish
causality for the setup failure; preserve it and require fresh complete gates.

Diagnostic SHORT-v2:12/12 PASS, including the exact unchanged native-transfer
case that failed GREEN-v1. Source1307 SHA32b2fe52de09a5ce88f8a02c70d2cea15cf1e69bc5321f24ddb1e8274ea85303.
This does not explain the prior failure or replace the complete source gate.
Run diagnostic GREEN-v2 once; preserve all earlier evidence. Runtimea286,
native22=20P1F1B and historical440 counts unchanged.

Diagnostic GREEN-v2:12/12 focused and1134/1134 affected IME PASS; graph/manifest
PASS, source1307 SHAdaf5ed414d1e33f7eafd72a1122f01fafbce95159bb6fef49499236c8df28efc.
No old assertion or deadline changed. GREEN-v1 setup failure remains an unexplained
recorded failure; this is one fresh gate following the bounded metadata repair,
not a cause claim. Next zero-warning diagnostic IME build and existing idle
component activation, then unchanged native short/retained22 with predecessor
metadata. FullFirefox220 not started; installeda286 remains until activation.

Diagnostic build0warning74.572s; idle IME-only activation now4098532,
SHA58d4a83fd020029e47b7023e19d5106f8104dbda48837505864bd6c5c1300a47.
All protected processes and exact config preserved; global IBus/local GNOME
untouched. Solo baseline review owned-append-caret-diag-review-v1.json binds
observation.rs to original installeda286 source before comparing the single
metadata-only helper delta. Native short2/2PASS21.416s, runtime/config preserved;
unchanged retained22 with causal facts next. No new functional acceptance or
full matrix count change.

## Measured continuation: empty local preedit after a completed native flip

Diagnostic retained22 again20PASS1FAIL1BLOCKED/71.632s. Complete row15 contains
the missing premise: immediately before probe z, closed predecessor is captured,
pending confirmed/current, native ACK true, exact whole actual tail matches,
scope UnknownStart. No KnownStart workaround is justified. Backspace then
successfully clears/hides the one-character local preedit. At rows1044–1049,
stale15chars/caret14 discards the pending receipt as second_surrounding_receipt;
adjacent actual14/14 arrives after it was discarded. Tab17.346ms, first flip
181.804ms pass; reverse fails. This is the first measured failed transition.

Dependency/authority map (all paths below src/bin/lay_ibus_engine):

```
existing daemon native flip -> confirmed closed-word actual receipt
  window_interaction/observation.rs::capture_legacy_preedit_start_boundary
    -> ManagedWordStartWitness.closed_word_predecessor (captured, native proof)
  existing one-letter legacy preedit -> existing Backspace/Clear/Hide succeeds
    observation.rs::capture_owned_preedit_erasure_reset_rereceipt_candidate
    -> existing PendingContextResetRereceipt, same revoked predecessor/successor
  actual stale15/14 -> observation.rs::observe_context_reset_rereceipt_surrounding_text
    -> currently discarded (first failure), before adjacent actual14/14
  existing authority.rs VisibleTail/manual handoff -> no grant -> reverse refused
```

Plan before edits: reuse only the archived closed-word-erasure echo hunks and
its three regression tests, not whole archived files. Preserve the newly
installed owned-append-caret fix and opt-in metadata. Keep the same existing
receipt as owner; an inert actual-before/erased-char/revision record may survive
one exact stale frame but conveys no authority. Only adjacent whole actual
before-field reconfirms, with token/epoch/focus/owner lease/layout/path checks.
Contradiction, duplicate, intervening snapshot/Reset or foreign client spends
it. Client dispatch must include this pending empty-preedit proof after the
composition itself is empty. No new callback, executor, RPC, queue or timer.

Active decision2026-10-29-kde-firefox-closed-word-erasure uses the same immutable
base and allowed files; native_cursor_observation.rs needs only mechanical None
initializers for the added inert field. GNOME direct-exact behavior and owned
append tests are retained. First restore tests only and require genuine RED;
then minimal implementation/short/full gates/build/native causal short/22.
This is not blindly reverting: prior withdrawal and earlier Tab failure remain
recorded; the new native trace now establishes the previously missing premise.
Receipt local owned-append-caret-diag-native-retained22-v1-row15.jsonl and
...-verification.json. FullFirefox220 remains NOT_STARTED.

Reused tests-only RED-v1:15selected14PASS; only closed_word_erasure_echo_requires_
final_actual_receipt fails on its exact final-field re-confirmation assertion.
Source096a4b77c28f70372e89da68e9c1a534e91a054e11f494ba34610a9e7a6a98a8.
Minimal implementation reuses the archived bounded observer byte-for-byte,
plus original inert-field creation, one-shot Reset refusal and actual client
selector. Current append-caret logic and diagnostic metadata remain. Reviewed
7 AST constructor matches;5 observation and2 native-cursor receive inert defaults,
only the successfully cancelled local preedit creates Some. No old assertion
or deadline changed; next focused15 then complete affected gate. Installed58d4
remains diagnostic until a new source/build/native result.

SHORT-v1 stopped at compilation: the reused call used the archived parameter
name erasure_client, while the current append-caret receiver names the same
argument firefox_presentation_client. Adapt only that call argument; preserve
the current receiver signature and its existing callers. No runtime installed.

SHORT-v2:15/15 PASS, source811bc70fd6e061b4ff98b5b2be0bba3bd35a0c99c48e6eb2365aa71a54493c04.
Reused final-actual positive,18 interrupted/refusal variants ×2 separators,
KDE/GNOME direct-exact controls and current owned-append controls all pass.
Next full affected IME/graph/manifest gate, then build only on complete PASS.

Closed-word-erasure GREEN-v1:1137/1137 affected IME correctness/package PASS,
focused15 overlaps; graph/manifest PASS, exactly3 restored identities added and
no old rows changed. Final source1307 SHA3a09d0a0200e554a4907f3be87abb3cd8fac5e59e5da07a5db759fdc0f9d32a6.
Three IME performance tests not run. Solo source review closed-word-erasure-
source-review-v1.json verifies byte-preserved old tests, archived new block and
observer, two mechanical native-cursor defaults. Next zero-warning IME-only
build/idle activation/native short/retained22. Full220 only after retained22
noFAIL on those exact loaded bytes; old full440 is still historical.

Closed-word-erasure build/activation:0warning74.492s, IME4178957,
SHAebd750a0382926a1f2bfb59c80c153d5f63e704957c1c2ce5006b48aa44a300b,
same final source3a09d0a0. Guarded idle replacement preserved daemon/IBus/tray/
user Firefox/config and RU/RU. Native short2/2PASS16gestures21.424s; whole-row
trace coverage still incomplete and causal nonexecution is not established.
Retained22 running unchanged; full220 only after its noFAIL gate on these bytes.

Closed-erasure retained22:20P1F1B75.938s. Plain now8/8PASS; rich7/8PASS then reverse
fails. Both long row traces incomplete; no new marker observed, no cause claim.
The retained suffix of rich row12 shows a different earlier failure: after its
seventh native flip, NativeReplayProcessedV1 false/cancellation occurs while
remaining target native key callbacks still arrive. Before probe z, no pending
receipt/predecessor; projection trailing ASCII Space differs from actual NBSP.
This is not the confirmed-predecessor/empty-erasure premise. Do not broaden that
handler to accept unknown context or normalize arbitrary text.

Next read-only dependency path: daemon/exact_ime_tail_replay.rs calls the existing
80ms waiter in layout_controller/ime_bridge/settlement.rs. An unavailable bound
V4 observation returns before its deadline via current_v3_reply()?; scoped
kde/firefox_replay_receipt.rs then reads NativeReplayProcessedV1 once. Hypothesis:
that read can occur before all already-delivered native keys are processed;
cleanup then cancels their provenance. Need actual passive wire reply to locate
the first failed read. Existing daemon log disabled and matching journal empty.
One fixed unchanged22 actor with its existing --observe-wire observer next;
no new source, install, active diagnostic RPC or retry-until-green. Full220 gated.

# TD-121: Сохранять целое слово при смене экземпляра IME

R12 development acceptance: **563/563 focused PASS**, two independent reviews,
**4/4 Firefox native PASS**, with two exact visible transitions in every case.
Final release, installation and publication status is recorded in the
[execution receipt](evidence/td121-release-1.0.72-execution-2026-09-14.md).
The [owning evidence](evidence/td121-r5-final-native-analysis-2026-09-14.md#r12-preflight-retain-observed-replay-history-across-an-interleaved-reset)
records the mechanisms, rejected candidates and bounded verdicts.

## Current R7 development — 2026-09-14

The late exact-client receipt repair now has **554/554 focused IME PASS** and
**1,795/1,795 shared-library PASS**, with both independent review passes complete. It computes inert material earlier in the
existing worker, then allows only a cache-only lookup after the current exact
receipt. Publication authority, Alt-before-ready refusal and the 150 ms worker
deadline remain unchanged. The controlled actual-worker cache-miss proof passes;
R7 native four-case acceptance is pending. R6 native 3/4 remains the latest measured
browser result; no R7 installation or release claim has been made. See
[the R7 proof](evidence/td121-r5-final-native-analysis-2026-09-14.md#r7-implementation-and-focused-proof).

## Current R6 native result — 2026-09-14

The completed-replay stale-snapshot repair has a controlled old-code failure,
**548/548 focused IME PASS** and two independent review passes with no remaining
production blocker within that replay repair. The owned synchronous Firefox
experiment is **3/4 PASS, overall FAIL**: first word, mixed prefix and trailing
Space each visibly flip and return with two delegations. Completion fails before
either toggle: with an active second-prefix preedit, the fresh third-character
receipt arrives only immediately before Alt, whose existing pending-work rule
retires the new worker. R6 is not installed or release-accepted; C20 is preserved.
Exact proof, consequences, the unresolved earlier setup failure and this native
result are in [the continuation evidence](evidence/td121-r5-final-native-analysis-2026-09-14.md#r6-native-result-and-first-remaining-loss).

## Current acceptance — 2026-09-14, final R5 native FAIL

TD-121 remains **IN PROGRESS / R5_FINAL_NATIVE_FAIL; DO NOT INSTALL R5**.
The final release gate passed 2,841/2,841 tests and all four isolated client
cells. Its final Firefox native gate passed only **2/4** complete two-toggle
scenarios: mixed prefix and closing Space passed; first word and completion
failed. The earlier R5 development-byte completion PASS is historical evidence,
not final-release acceptance. Installed C20 remains the preserved baseline;
human keyboard acceptance is NOT_TESTED.

Exact receipts under
`/home/ubu/.cache/lay/development/release-1.0.72-td121-firefox-r5-20260914/`:
`raw/RESULT.json`, `final-native/native-control.json`, and
`final-native/two-toggle-visible-proof.json`. No unchanged release gate is
being repeated. Saved-trace analysis and the next discriminating evidence are
recorded in [the R5 continuation analysis](evidence/td121-r5-final-native-analysis-2026-09-14.md).

## Current TD-121 acceptance — 2026-09-13

TD-121 is **IN PROGRESS / FIREFOX_FAST_REPEAT_FAIL**. The current diagnostic
candidate passes 530/530 IME tests and the native Firefox single fast pair.
The valid two-pair case takes 5.98 ms for a 5 ms activation deadline before
typing. The valid autocomplete case loses its receipt on the preparatory Alt
press, before acceptance on release. The latest native denominator is 1/3.
The preceding invalid focus-loss capture remains recorded separately. Producer
counterevidence rejected the earlier right-boundary experiment, whose source
and two tests were removed. Exact scope and next investigation are at the end.
Root owns source and remote execution;
no Firefox/GTK module or browser environment change has been implemented.

The following C20 installation checkpoint is historical. C20 passed the
complete 1.0.72 release gate: 13/13 commands, changed and full gates each
2,807/2,807 with 11 intentional performance skips, and compiled-receipt verification and four isolated final-byte client cells. Release `RESULT.json`
SHA-256: `02458047a539fb85be82b301fd1cf38b19af1f71241f34260713cf5c5dd90534`.
The preceding focused 519/519 proof on the same runtime source passed separately.
The owned GTK entry smoke passed all three exact surfaces: `привет` after one
toggle, `ghbdtn` after two toggles with zero queued key/space/boundary passthrough,
and layout projection `ghjdthrf` after autocomplete with one toggle. Receipt SHA-256:
`27bf83fccd5a15e552dabd9afeada8f1bacfd9038b8f9a0046f2cb3d0fcd1eaf`.

Release 1.0.72 is installed with all ten installed artifacts and all four loaded
owners matching C20; the loaded extension reports 1.0.72. Global IBus identity,
configuration, input sources, immutable models, journals and learner state were
preserved. Installation receipt SHA-256:
`ca7b0cb622f862cdb9a51678e640d27953fe798f3b37f291bdebce4f5e4735a4`.
C12 remains historical 502/502 source-review evidence accepted at 9/10. The
current human report is: physical Double Shift works in other windows, but in
Firefox rapid/repeated Double Shift fails and then stops working in that field.
This is `FIREFOX_FAST_REPEAT_FAIL`, so overall physical acceptance and TD-121
remain open. General TD-123 answer quality is `UNKNOWN` because routing/client
delivery proof is a separate denominator.

Historical attempts and their exact receipts are retained only in
[evidence/td121-private-actual-baseline-2026-09-13.md](evidence/td121-private-actual-baseline-2026-09-13.md).

## Firefox fast-repeat measured consequence and causal RED preflight — 2026-09-13

The bounded live trace has 2,727 JSON rows and SHA-256
`65a86d3c6d37e95883764c8fd3ba81849610522c966579dac668bdd23652e57f`:
`/home/ubu/.cache/lay/development/td121-firefox-fast-repeat-20260913-t__qoo7d/ibus_engine_debug.jsonl`.
It contains no retained manual-toggle or suppression event, so it does not prove
the earlier `LocalPending -> Reset` hypothesis. Application identity is not
logged; Firefox scope comes from the user's report. The later caps9/purpose10
terminal episode begins separately after row 1694.

Two caps41 `UnknownStart` episodes show the same measured sequence. Rows
995–1058 repeatedly arm and confirm exact reset re-receipts while the suffix grows
from one through four characters. The fifth handled append advances the tail to
five characters, then row 1071 rejects the exact five-character callback as
`second_surrounding_receipt`. The later Reset at rows 1087–1090 cannot arm a
successor and records `missing_predecessor_or_post_reset_token`. Rows 1487–1575
repeat the same confirmed-append, second-receipt rejection and later Reset loss.

The first source-supported loss is
`advance_context_reset_rereceipt_after_key`. It deliberately advances the exact
token text, suffix count and tail epoch after one handled printable append, but
stores the successor with `confirmed: true`. Therefore the fresh exact callback
for that advanced token enters
`observe_context_reset_rereceipt_surrounding_text` and is rejected immediately
by its generic `pending.confirmed` second-receipt guard. This conflates two cases:

```text
same token + duplicate SurroundingText        -> reject as second receipt
handled append + next exact SurroundingText   -> currently rejected the same way
```

Only the second route matches the retained Firefox trace. The existing
`second_receipt` negative covers an unchanged token and must remain rejected.
Focus/owner/path changes, selection, mismatch, capability loss, sensitive
content, navigation, boundary input and stale reducer identity must also retain
their current rejection behavior.

The smallest causal RED is one controlled legacy adapter schedule on the existing
reset re-receipt harness: establish `UnknownStart`, Reset, and exact confirmed
suffix `abcd`; process one handled printable append to `abcde`; deliver a fresh
exact `SetSurroundingText("abcde", 5, 5)`; then deliver the next authenticated
Reset. The required assertions are that the appended receipt is not classified
as a duplicate, the successor Reset can arm from the exact advanced identity,
and an early ManualToggle before that fresh receipt remains refused with zero
delete/commit effects. A sibling unchanged-token second callback must still
reject.

The controlled RED then ran under the remote `dedicated-20cpu` guard against an
exact 1,410-file manifest. After removing one incidental assertion about the
post-Reset release return value, the exact test executed **1** test and failed at
the intended first assertion: fresh exact `SurroundingText` for advanced token
`abcde` was rejected as a duplicate. Result:
`/home/ubu/.cache/lay/development/td121-firefox-fast-repeat-20260913-t__qoo7d/causal-red/RESULT.json`,
SHA-256 `871a20f66a24e13bc33a0e2c8d342e2207aa053f12792554acf248b118493824`;
exact log SHA-256
`0f609c58d6991f9a381932507448d0f68e552b05183b79481a1742064a7c21db`,
`rc=101`, 0 passed, 1 failed, 519 filtered. The second prefix was not reached,
and the successor Reset assertion was not reached. An earlier zero-test filter
and the incidental pre-target failure remain labelled invalid predecessors in
the result rather than promotion evidence. No production change, GUI input,
restart, installation or new scored review ran; runtime authority remains the
installed C20 bytes. Verdict scope: the measured append/fresh-receipt mechanism
is reproduced, while the repair and negative-control proof remain pending.

## Firefox confirmed-append consequence analysis — 2026-09-13

This analysis is source-only and precedes any production edit. The baseline has
two distinct states encoded in the same `confirmed` bit. Immediately after the
first exact post-Reset receipt, the pending record is confirmed at observation
revision `R + 1` while its `armed_revision` remains `R`. A second callback for
the unchanged token advances the client revision to `R + 2`; it is a duplicate
and must revoke the pending record. After one verified handled append,
`advance_context_reset_rereceipt_after_key` carries forward the confirmed
rereceipt witness, advances the exact token text, suffix count and tail
epoch, and resets `armed_revision` to the then-current `R + 1`. Firefox's next
exact callback therefore arrives at exactly `armed_revision + 1`, but the
unconditional `confirmed` branch currently rejects it before checking either
the revision or snapshot.

The identity conjunction before that append successor is created is already
strict: key press, non-Shift printable non-boundary input, handled by the engine,
no command modifier, accept-space, Backspace, Enter or navigation; the old tail
must end in the pending token; the committed tail must be an exact one-character
append; the new token must extend the old token by exactly one character;
`tail_epoch` must advance by exactly one; the current admission token must match
the current word scope and revalidate. Focus/owner/path loss, selection,
capability change, sensitive content and the existing revocation routes still
clear the record. Snapshot acceptance additionally requires no selection, exact
token suffix at the cursor, and word boundaries on both sides.

Three production designs were compared:

1. Set the append successor to `confirmed: false` and reuse the existing first
   receipt path. This makes the Firefox callback pass the current code, but it
   withdraws the existing confirmed witness between the observed append and its
   callback. The current RED correctly expects manual handoff to remain false in
   that interval because the client snapshot is still the old token; preserving
   `confirmed` does not bypass that snapshot guard. It does preserve the witness
   for an already-current exact snapshot if a client has synchronously exposed
   one through an independently observed route. That interval is not established
   by the retained Firefox trace and needs a separate reachability assertion
   rather than an assumption. This design needlessly changes the witness and is
   rejected.
2. Add an explicit phase or second flag such as `awaiting_advanced_receipt`,
   retain `confirmed: true`, and let only that phase accept the next exact
   `armed_revision + 1` callback. This preserves the confirmed witness and
   cleanly names the state, but expands every constructor, clone/equality state,
   invalidation proof and test surface for information already represented by
   the revision transition.
3. Keep the record shape and confirmed witness unchanged. In
   `observe_context_reset_rereceipt_surrounding_text`, when `pending.confirmed`
   is true, accept only the exact current snapshot at
   `surrounding_observation_revision == armed_revision + 1`; keep it confirmed
   and return. Reject every other confirmed callback as a second receipt. This
   is selected because the append transition alone rebases `armed_revision` to
   the current revision. The first confirmation does not: its unchanged-token
   duplicate arrives at `armed_revision + 2`, so it remains rejected. No new
   mutable state or authority source is introduced.

The minimum production diff is confined to the confirmed branch of
`observe_context_reset_rereceipt_surrounding_text`: evaluate the existing exact
revision-plus-snapshot conjunction before clearing; when it holds, retain the
record as confirmed and emit a distinct advanced-receipt trace; otherwise keep
the current `second_surrounding_receipt` rejection. The unconfirmed arm path and
its exact `armed_revision + 1` confirmation remain unchanged. The append
producer and all of its token, owner/scope, admission, tail, epoch, key and
boundary guards remain byte-identical. Double-Shift detection, GTK and terminal
delivery routes, timers, deadlines, leases, bridge consumption, SafetyGate,
edit-plan validation and verifier authority are outside the diff.

The selected design has one proof obligation beyond the current RED: revision
arithmetic alone must not manufacture provenance. GREEN must show the accepted
callback follows the verified append transition, retains the same revalidated
post-Reset admission token and predecessor-token relation, matches the advanced
tail epoch/text/suffix and exact current snapshot. Manual handoff must remain
false after append while the stored snapshot is old and become allowed only
after the fresh exact callback. If an already-current-snapshot interval is
constructible, it must separately prove every existing guard before authority;
the confirmed bit alone is insufficient. The
unchanged-token second callback must still clear the record; so must wrong
revision, mismatch, selection, focus/owner/path change, capability loss,
sensitive content, navigation, boundary input and stale admission identity. The
next authenticated Reset must arm from the advanced confirmed record without
reusing its predecessor as current authority.

Before/after evidence must remain separate. Before is the fixed 2,727-row trace
plus the exact 1-test causal RED: it proves the Firefox-shaped loss at the first
fresh advanced receipt, not general client quality. After requires the causal
test to pass independently for both planned prefixes, a dedicated unchanged-token
duplicate negative, the existing reset-rereceipt failure matrix, and the focused
changed gate. GTK, terminal and real Firefox outcomes remain separate client
denominators; the parent's isolated Firefox helper is a synthetic client-visible
smoke, not human physical-keyboard acceptance. No compression, answer-quality or L1 restoration claim follows
from this transport repair: aggregate and every fixed damage class, clean
preservation, lattice coverage, false certainty, package/RSS and latency retain
their existing conjunctive gates. Because the selected diff adds no work outside
one already observed callback and changes no model/material path, resource and
latency impact is expected to be negligible but remains unmeasured until the
normal gates run. Runtime authority remains installed C20 until a separately
reviewed, built, installed and physically verified candidate replaces it.

Cache and lifecycle consequences are bounded to the pending rereceipt record.
No model, material, candidate, completion or bridge cache key changes. Existing
focus, owner/path, capability, content-type, word revocation and admission-token
invalidation owners continue to clear or make the record unusable; package and
delta reload keep their current generation invalidation and do not inherit the
receipt. A stale or concurrent callback can advance the monotonic observation
revision only once through the serialized engine callback owner: wrong order or
an intervening callback misses exact `armed_revision + 1` and clears the record.
The callback neither schedules background work nor publishes completion,
learning or outcome feedback, and it does not retain pending completion learning
across any existing revocation.

The chosen branch performs no allocation beyond the snapshot/string work already
performed by every surrounding-text callback and adds only fixed comparisons and
one trace on the accepted route. CPU, RSS, package-size and latency effects are
therefore expected to stay below measurement resolution, but remain unmeasured
until the normal fixed gates. Failure is fail-closed: any missing identity,
revision or snapshot fact clears the receipt and grants no edit. Source rollback
is removal of the one confirmed-branch exception and its tests; installed
rollback remains the existing verified binary transaction and is outside this
source-only step. Maintenance has no new state to expire or migrate. If future
client evidence removes the redundant advanced callback, the exception and its
trace can be deleted together while the unchanged-token rejection remains the
baseline.

### First GREEN execution outcome — C1

The accepted confirmed-branch change and causal test ran once under one remote
`dedicated-20cpu` lease against an exact 1,410-file source manifest. The exact
causal test passed **1/1**, 0 failed and 519 filtered. Its single test executes
both `abcd -> abcde` and `a -> ab`, requires manual authority to remain false
while the snapshot is old, accepts the fresh revision-bound exact receipt,
rearms the next authenticated Reset, and rejects a subsequent unchanged-token
second receipt. Exact log SHA-256:
`3b9bdb34edd73879cf9bab6ddda9612e5fcd30e1462064166a85003e7cb0756d`.

The same lease then reached a new harness blocker before focused test execution:
focused discovery invokes `git ls-files`, but the exact source archive contains
the 1,410 source files and deliberately no `.git` metadata. Git exited 128, so
the focused denominator is **0 executed**, not a runtime FAIL. Source identity
matched before and after with zero mismatches; nonzero remote shell and lease
identities were recorded. No second lease, further patch, architecture refresh,
release, installation, service, GUI or review action followed. Compact receipt:
`/home/ubu/.cache/lay/development/td121-firefox-fast-repeat-green-20260913-c1/RESULT.json`,
SHA-256 `e2b46fb6f9ddfdc3de20c7a20410a0b62f648add8a883128e1a86e8cab078df5`;
focused blocker log SHA-256
`e09899051f6848dd5c43ea2f73d858399ff5dd7d124b56c2e628c48da3b6acac`.
The repair remains unpromoted until the focused negatives, complete focused IME
and required remote architecture refresh run from a Git-aware exact checkout.

### Git-aware GREEN completion — C2

C1 remains preserved as `INVALID_PACKAGING` for its zero focused denominator.
C2 initialized an isolated Git repository only inside the remote disposable
workspace, added exactly the manifest's 1,410 files, and committed that snapshot
for discovery. The index had 1,410 tracked paths with no missing or extra path;
SHA-256, size and mode verification had zero mismatches before and after tests.
The local repository index was not touched.

Eight exact routes then passed with a nonzero denominator, one test each. Their
guard mapping is:

- fresh advanced revision/exact snapshot, old-snapshot refusal, successor Reset
  and unchanged-token duplicate: the new Firefox causal test;
- mismatch, selection, unchanged duplicate, navigation, focus loss, capability
  loss and sensitive content: `residual_reset_rereceipt_failures_never_delete_gui_text`;
- old owner/path generation: `residual_delayed_old_revocation_cannot_clean_up_different_path_successor`;
- foreign owner, focus loss and stale witness: `residual_reset_unknown_witness_is_not_revived_by_foreign_or_focus_out`;
- word-boundary retirement/rearm: `c20_source_free_boundary_rearms_only_the_next_word`;
- capability invalidation across transfer: `td121_target_capability_change_after_transfer_invalidates_the_inherited_snapshot`;
- duplicate and stale identity: `exact_replay_duplicate_and_identity_mismatches_revoke_without_text_effect`;
- wrong callback order/revision and visible mismatch: `exact_replay_order_and_visible_glyph_mismatches_revoke_without_text_effect`.

Two predecessor invocations of the last two tests omitted their intermediate
`word_scope` module and selected zero tests; they remain explicitly
`INVALID_ZERO_TEST_FILTER`. The corrected canonical names each passed 1/1 and do
not borrow authority from those invalid attempts.

Full focused IME discovery found **520** tests. It selected all **517** correctness
tests and they passed **517/517** with zero failures; the three excluded tests are
the unchanged declared performance lanes. The canonical test manifest reports
one added test, the Firefox causal regression, with no changed or removed test.
Focused summary SHA-256:
`49c7e2905b700e5e67adeba7904b649dbebb1eeab4599c479e8244c78f6a87fc`.

After stable source and document state, remote `graphify update .` completed and
the four changed graph outputs were copied back to the canonical checkout. Its
log SHA-256 is
`93b6518a3b40ae8a9af2c903be54d33f32396e97ebf080e13a4ba2757ec27504`.
Compact C2 receipt:
`/home/ubu/.cache/lay/development/td121-firefox-fast-repeat-green-20260913-c2/RESULT.json`,
SHA-256 `87980080d9814be78f76a55d9e83688b41fc9062a588831d3d63fe00e90b9b75`.
This is source-only GREEN evidence. No release gate, installation, service, GUI,
physical-keyboard action or scored review ran, and installed C20 authority did
not change.

## Firefox compatibility request 3 causal preflight — 2026-09-13

This is a separate loss from the confirmed-append repair. In the C20 synthetic
Firefox `ghbdtn_extra_lshift_enter` trace, request 2 published a source-free
owner 2. The observer then recorded `FocusOut(29)` and `FocusIn(30)` before the
callbacks settled. The FocusOut callback installed the already published old
outcome and then sealed owner 2; the FocusIn callback started compatibility
request 3. Request 3 recorded marker arm and emission, but no publication or
`refused_not_current`; every later key arrived with no owner and Reset was
`stale_context`. DOM focus stayed ready from monotonic 916195.615 through the
first key at 916200.135, so the harness did not intentionally remove focus.

The evidence does not yet identify which readiness prerequisite was lost.
Production `ACQUISITION_BUDGET` is **5 ms**, whereas the normal P2P bootstrap
uses 250 ms or a larger callback budget. Marker `emission=emitted` proves only
that the signal send completed before the deadline. It does not prove that the
observer authenticated the marker before 5 ms, nor that the reducer was Ready
when it did. The later 4.5-second DOM-to-key interval cannot recover an expired
request and does not show that the original Get/reply/marker chain met 5 ms.

Source inspection yields two discriminators. If the marker was not observed by
the deadline, `expire_fence` removes the pending fence and
`context_acquisition_failed(request)` revokes the matching request. If the
marker was observed but `try_ready` lacked a prerequisite, `expire_fence` does
not remove that marker-observed fence; it can remain pending with no publication.
For a Transfer request the exact prerequisites are current request generation,
nonce and lifecycle revision; same connection/context; reply strictly after the
compatibility focus; marker after reply and source seal; pending ticket with
matching id/target, FocusOut, target FocusIn and source seal; empty unsettled
callbacks; and matching Lay profile. A different CurrentInputContext already
converts the same generation and nonce to SourceFree, retaining reply, target
profile and focus; its readiness requires the same ordered reply/marker, empty
unsettled set and matching profile. A second fallback would duplicate this
owner and is excluded.

The controlled RED must reproduce the exact ordering without changing the 5 ms
budget: publish but do not yet install source-free owner 2; observer-process
FocusOut then FocusIn; let the FocusOut callback install the old outcome and
seal it; let the FocusIn callback start compatibility request 3; capture the
exact request generation, nonce, target, expected profile, ticket/source seal,
focus positions, lifecycle revision, unsettled count and deadline. Run two
branches with the Get reply before the existing deadline: old context must keep
Transfer, new context must convert the same request to SourceFree. In each,
forward the exact marker in order, process it, and assert whether status becomes
Ready and one target-bound outcome publishes. Capture every prerequisite both
immediately before marker and after it. Do not repair an unexpected state.

A sibling deadline control must hold the Get reply or marker beyond the same
5 ms deadline and require zero publication, cleared request/fence, zero owner
and no stale token. Wrong nonce, marker-before-reply, wrong sender/path, profile
mismatch, nonempty unsettled callbacks and stale owner/request identities remain
negative controls. If both prompt old/new-context branches pass, the synthetic
failure is not reproduced and the next bounded step is opt-in metadata tracing
for compatibility reply, marker observation, deadline expiry and the failed
`try_ready` predicate. Budget increase, retry, second SourceFree fallback and
test-focus changes are outside this preflight. No second production repair is
authorized by this analysis.

### Controlled discriminator outcome — 2026-09-13

The test-only P2P schedule held source-free owner 2 after publication and before
installation, let the observer receive FocusOut and FocusIn in order, then ran
the FocusOut callback to install and seal owner 2 before the FocusIn callback
started compatibility request 3. Both prompt reply branches reached Ready and
published exactly one target-bound successor. The same-context reply retained a
Transfer whose source owner was the sealed predecessor; the different-context
reply converted the same request generation and nonce to SourceFree. In both
branches the successor owner generation was newer, the pending fence and request
were consumed, the target installed the outcome, and no ready outcome remained.

This result does **not** reproduce the Firefox loss when the production reducers
receive the prompt reply and marker. It narrows the live cause to timing or to a
prerequisite absent from the synthetic schedule. The sibling used the production
5 ms acquisition budget: its controlled Get reply and marker emission completed
within that budget. It then invoked the existing fence expiry owner directly,
without a sleep or retry, and proved only the forced cleanup semantics: pending
fence and request cleared with no owner, token, or ready publication. Direct
expiry before its wall-clock deadline is not a measured wall-clock deadline
PASS. No production code, deadline, retry, fallback, decision, text effect, or
runtime authority changed.

The remote guarded focused target discovered 522 tests, selected all 519
correctness tests, and passed 519/519 with zero failures; the three declared
performance tests remained excluded. `cargo fmt --all --check` also passed.
Receipt:
`/home/ubu/.cache/lay/development/run-01_htg7j/RESULT.json`, SHA-256
`5c243730ae16f5af7a9dad4bb83df891f82f0632ab3378c16a3d96a2af388d83`.
Verdict scope is test-only state ordering and controlled timing/cleanup. Actual
Firefox compatibility reply, marker observation, expiry state and the first
missing `try_ready` predicate remain unmeasured, so real-client quality is
`UNKNOWN`. The next bounded experiment is diagnostics-only metadata for those
four states, with no content text and no authority or deadline change.

The required remote `scripts/update-architecture-graph.sh` chain refreshed and
pruned the graph and rewrote its source binding, then stopped before writing a
receipt because the calculated repository verdict is `WATCH`. The only reported
violations are three `edit-plan-verifier` mutation sinks
`apply_text_replacement_pipeline`, `call_replace_text`, and
`try_ime_replace_tail`; every other listed architecture invariant passed. This
test-only discriminator adds no mutation sink and changes no runtime authority;
the cause of the WATCH relative to the prior PASS receipt remains unresolved.
Exact remote log:
`/home/e/projects/lay-td121-compat-discriminator-graph-20260913/graph.log`.
What was not established: an architecture `PASS` receipt or resolution of those
three repository-level WATCH findings.

### Firefox native-environment comparison and diagnostics-only plan — 2026-09-13

The second retained C20 Firefox comparison removed the base wrapper's
`IBUS_ENABLE_SYNC_MODE=1` only from the Firefox child and matched the user's
observed allowlist: `GDK_BACKEND=wayland`, `GTK_IM_MODULE=ibus`, sync mode
absent. Browser and `/proc` environments agreed. It still failed 0/3 with
`restored=true` and `fatal=null`. The Extra route refused all 20 key events;
compatibility request 4 emitted its marker but never published, matching the
synchronous comparison's owner loss. The Fast route accepted all 16 key events,
recorded six confirmed full-word reset receipts, and accepted and settled all
four Shift events, yet `ManualToggleV3` still returned
`context admission denied`. This proves the loss is not created solely by the
sync-mode environment difference. It does not prove a 5 ms bug: neither trace
records compatibility reply acceptance, marker observation, deadline expiry, a
failed readiness predicate, or the bridge fence failure phase.

Native-environment receipt:
`/home/ubu/.cache/lay/development/td121-firefox-fast-repeat-20260913-t__qoo7d/firefox-c20-user-env/RECEIPT.json`,
SHA-256
`8a5f6f0d5b0ef3a7a4350b7b4fdc3b3c4cba8d19f775e074de1c91485c09f381`.
The retained 195-row Fast trace is
`/home/ubu/.cache/lay/development/td121-firefox-fast-repeat-20260913-t__qoo7d/firefox-c20-user-env/ghbdtn_fast_lshift_enter-89c9e61c9516dd2f6aa5/ibus_engine_debug.jsonl`;
SHA-256
`72297a544d831bad661ca8faa7c1d8d603e0aa7b5fe61b53620afb8961a03d8b`.
The short case contains no fresh-surrounding duplicate, so it neither supports
nor contradicts the confirmed-append repair.

The proposed diagnostics patch is metadata-only and opt-in through the existing
trace switch. It adds one fixed-schema admission diagnostic record with these
fields only: flow (`compatibility` or `bridge`), phase, outcome/reason, request
generation and nonce when applicable, reducer status, unsettled count, current
owner and activation generations, and booleans for fence presence and marker
observation. It records no surrounding text, committed tail, key glyph, context
path, client string, profile name, or D-Bus body.

Exact production insertion points and timing rules:

1. In `finish_compatibility_activation`, retain only scalar refusal metadata.
   Do not write a successful setup trace between marker emission and observer
   marker processing/consumer finish: the critical 5 ms interval is still open.
   Failure records may be emitted only after the corresponding setup decision
   and cleanup have completed.
2. In `process_marker`, capture the first missing readiness predicate in the
   same reducer-lock snapshot that performs the existing marker/readiness
   decision. Complete deadline rejection or publication processing, release all
   locks, perform the existing notification, then write a failure record. Any
   later counts are explicitly `post_decision_snapshot`, never reconstructed as
   the causal predicate.
3. In `expire_fence`, capture the matching fence scalars, perform the existing
   cleanup, release pending and reducer locks, then emit whether expiry removed
   the request or found an already observed marker. The timer, deadline and
   cleanup order remain unchanged.
4. For `ManualToggleV3`, distinguish bridge `begin_ping`, `marker_emitted`,
   `marker_observed`, `wait_failed`, and `finish_failed`. On refusal, report the
   first existing predicate among no current owner, no current activation/token,
   nonempty unsettled callbacks, fence identity mismatch, marker missing,
   deadline exceeded, and bridge-token revalidation failure. Snapshot reducer
   status, unsettled count and owner/activation generations only after the
   decision and with no lock held during the trace write.

Read the atomic, already-warmed cached trace enablement bit before taking any
diagnostics-only lock or allocating a record. No additional timer, retry, RPC,
queue, fallback, authority branch, status
transition, deadline change, or text decision is permitted. Diagnostics cannot
call `try_ready` again or turn an approximate post-decision snapshot into an
authority input. The patch should be limited to `trace.rs`, adapter-local
read-only diagnostic projection and the two existing call sites; it must not
change reducer method signatures used for decisions.

Before any production diagnostics edit, the remote guarded baseline already
passes `cargo fmt --all --check` and all 519/519 selected IME correctness tests
from 522 discovered, including the prompt Transfer/SourceFree discriminator,
the production-budget controlled exchange, unsettled bridge-fence refusal and
post-fence ingress refusal. Baseline receipt:
`/home/ubu/.cache/lay/development/run-01_htg7j/RESULT.json`, SHA-256
`5c243730ae16f5af7a9dad4bb83df891f82f0632ab3378c16a3d96a2af388d83`.
The diagnostics candidate must rerun those exact routes and the complete focused
IME correctness denominator. Trace-schema tests must assert field names,
reason/phase coverage and absence of forbidden text/path/profile/body fields;
they must not assert timing from log order. Actual Firefox evidence remains a
separate client denominator. Production C1 stays frozen pending review.

Scale consequences are bounded but nonzero when tracing is enabled. Failure-only
records add fixed scalar comparisons, brief existing-state lock acquisitions,
one bounded JSON allocation and one debug-log append per diagnosed failure; they
can perturb scheduling and therefore cannot prove an uninstrumented 5 ms timing
result. With tracing disabled, the added path is one atomic enablement load and
branches, with no diagnostic allocation or disk write. The atomic is one
diagnostics-only scalar mirroring the existing `TRACE_CONFIG`; it starts false,
updates only when the existing `enabled()` refresh runs, and shares that cache's
250 ms refresh semantics. A cold diagnostic-only failure can be omitted. No
authority or configuration decision reads this scalar, and it is not a general
diagnostics controller. There is no new queue, learning/feedback event, reload key,
model/material cache, or cross-generation inheritance. Concurrency keeps the
existing lock order; no lock spans notification or disk I/O. CPU, RSS and package
growth must be measured by the candidate build and remain scoped diagnostics
costs, not assumed zero. Rollback/removal deletes the diagnostic projection,
record formatter and call-site records together; no state migration or cache
cleanup is required.

The first candidate run found a concrete timing perturbation: one existing
marker-before-final-settlement test failed while 517 other selected tests passed.
The marker path had refreshed the trace configuration before its admission
decision. The corrected design reads only the atomic, already-warmed cached trace
enablement bit in marker/expiry paths; it takes no cache mutex and performs no
config load or disk access to decide whether to collect diagnostics. Existing earlier trace activity warms
that cache in an opted-in runtime. Diagnostics alone do not initialize tracing.

After restoring the original observed-marker expiry semantics and replacing the
diagnostics cache mutex/config refresh with the atomic mirror, the guarded remote
focused target discovered 523 tests, selected all 520 correctness tests, and
passed 520/520; three declared performance tests remained excluded. Formatting
also passed. The schema test proves the fixed 12-field record and absence of
text, context path, client, profile, body and decoded-key fields. Receipt:
`/home/ubu/.cache/lay/development/run-xsmlst5_/RESULT.json`, SHA-256
`1622d64f99a6c118385baaa57be9ca13303eadc8ca4b66b77dce2746f00675fc`.
Two predecessor runs exposed and rejected diagnostics-induced drift: the first
performed a config refresh in the marker path; the second mistakenly revoked an
observed-marker request during expiry. Neither is accepted evidence. The final
source preserves C1 decisions and changes only opt-in failure observation.

The implemented first candidate is intentionally narrower than the full plan:
it records compatibility marker/expiry failures and bridge marker refusal, which
cover the observed emitted-marker/no-publication and `ManualToggleV3` denial
routes. Bridge `begin_ping`, `wait_failed`, and `finish_failed` phase records
remain proposed and are not present in this candidate. They must be added only
if the first live trace does not discriminate the failure.

After excluding observed Bridge fences from the compatibility expiry record, the
final guarded remote rerun again discovered 523 tests and passed all 520/520
selected correctness tests, with three performance tests excluded and formatting
PASS. Final diagnostics receipt:
`/home/ubu/.cache/lay/development/run-p0dvyx01/RESULT.json`, SHA-256
`618b8e231e8876f2e79c3e63a0bd999db70d86baf1a11970d59a4527c1862a67`.

The proper architecture refresh then ran under the remote dedicated-20cpu guard
in the established `/home/e/projects/lay-development-runner/workspace` root,
with relocated AST cache absent. The graph recovered the canonical
`src_text_edit_executor_authorizededit` node and six `parameter_type` edges;
`edit-plan-verifier` and every other architecture check passed. Coherent graph,
source binding and architecture receipt outputs were copied back. Remote log:
`/home/ubu/.cache/lay/development/td121-diagnostics-candidate-20260913/graph-final.log`,
SHA-256 `08e1c935b38f3279fc1efa83a26efc60fc4ff2f7a0f32f9ab4a3238fc33aeda3`.

Exactly one inactive release candidate, `lay-ibus-engine`, was built under the
same remote guard with the shared target. It was not installed, activated or
used for GUI input. Artifact:
`/home/ubu/.cache/lay/development/td121-diagnostics-candidate-20260913/artifact/lay-ibus-engine`,
SHA-256 `5f94d028d197a49c76d7dfd7eac034471ace0cee78762aa6b9da44553b993956`,
size 7,868,512 bytes. Its exact source/artifact/build binding is
`/home/ubu/.cache/lay/development/td121-diagnostics-candidate-20260913/artifact-binding.json`,
SHA-256 `aab59360949d7bcd534af9c8724a891c8acb7e04ce922176d562b85f5a4ed2b1`.
The source-manifest SHA-256 is
`91e2b2cd4dda03d6c2a48fab2fcad188b274b8bbfdf72d74dfbedf69df0ab7fa`.
Runtime authority remains the prior installed C20 bytes pending the single
owned Firefox diagnostic run and review.

## Pending cross-profile reset consequence and test plan — 2026-09-13

The corrected controlled P2P RED completed at 03:52 UTC. With both `lay-us`
and `lay-ru` admitted, it opened a `lay-ru` target while the current profile was
still `lay-us`, retained the original native reply and marker, rejected Reset
for an unrelated path without changing request identity, then delivered Reset
to the exact target. Production revoked the request, and the test failed at
`word loss must preserve the exact pending request`. This establishes
`apply_word_reset -> discard_pending_target_word` as the first loss point for
this ordering; ContentType and later evidence cases were not reached in the RED.
Combined focused IME was 500/502: all prior tests except the new RED and an
incidental C28 pending-fence assertion passed. Source and dependencies remained
unchanged and runtime authority did not change. Receipt:
`/home/e/projects/lay-development-runner/td121-cross-profile-focused-red-20260913T035000Z-c10/RESULT.json`,
SHA-256 `e39a114f09e10693bd7f41de8e45a8a4917372840d2ad4e638639dba17a81856`.

The admitted repair removes only the premature current-profile equality check
from exact-target word discard. Verified global mode, exact request lifecycle,
ticket identity/status/target/focus, original request/nonce, expected target
profile, marker, and strict readiness remain required. The conversion itself
has no authority; a later matching profile event may make the source-free
`UnknownStart` request ready, while foreign or mismatched evidence revokes it.

The C8 actual-client trace rejects the earlier begin/factory hypothesis. Native
request 12 / nonce 55 was accepted, the context reply was accepted, and its
marker was armed and emitted. The trace then records Set against the old global
profile, `stale_context`, and publication `refused_not_current` after revocation
advanced from 4 to 5. C8 did not independently record the relative position of
`GlobalEngineChanged` and Set; Reset/ContentType-before-profile-change is the
causal hypothesis that the controlled RED must establish.

Static inspection identifies the first candidate mechanism at
`apply_word_reset -> discard_pending_target_word`. The latter currently requires
the reducer's current global Lay profile to equal the ticket's expected target
profile. During a real cross-profile transfer, the current profile is still the
source until the later ordered `GlobalEngineChanged`, so discarding only the
pending word can revoke the entire request and nonce instead of preserving the
existing source-free `UnknownStart` acquisition.

Four bounded choices were compared:

1. Change `begin_source_free` or factory binding. This contradicts C8, which
   proves those stages completed, and would broaden acquisition semantics.
2. Poll readiness or reorder callbacks. This does not address the observed bus
   order and would recreate ordering outside the reducer.
3. Immediately convert the exact pending transfer to its already modelled
   source-free `UnknownStart` request when authenticated Reset or changed
   ContentType targets its exact reservation. The unchanged marker, reply,
   target, expected profile and strict `try_ready` conjunction remain required.
   This adds no deferred mutable state and is the preferred bounded candidate.
4. Retain the transfer request and defer its word discard until the expected
   profile arrives. This can preserve the same authority boundary, but adds a
   pending-reset state and another settlement transition that must itself be
   invalidated, correlated and tested. It is viable only if immediate conversion
   cannot retain the existing identities and is therefore the more complex
   fallback.

The preferred consequence preserves the request generation and nonce but never
the transferred word token, lineage authority, cached candidate material,
autocorrect suppression, completion learning or background result. It creates
an empty `UnknownStart` lineage for the exact reserved target only. Reset for
another path is passive; stale marker/request identities remain unusable; later
foreign or mismatched Lay profile evidence revokes the pending intent. A grant
still requires the existing reply, marker, target binding, matching profile,
settled ingress and `try_ready` checks. Before those facts there is zero engine,
bridge, edit or verifier authority.

Both viable choices leave waits, CPU/RSS limits, caches and model/material
generation unchanged. Neither re-runs inference, retains stale completion
feedback, publishes a candidate, changes consumer-visible text, or changes the
installed runtime. The source-only rollback is removal of the reducer branch
and its tests; the C8 candidate remains the byte-identical runtime reference
until a production edit is justified. Client effects must be measured
separately after the RED and any repair.

The immediate-conversion production condition, if the RED confirms it, is:
preserve the exact pending request/nonce when authenticated Reset or changed
ContentType discards the word for its exact reserved target, then require the
existing matching profile event and marker before any grant. This is the narrow
candidate because it changes no pre-event authority and retains all existing
negative checks.

Before any production change, a controlled P2P regression must reproduce both
Reset and changed ContentType before `GlobalEngineChanged` on a pending
cross-profile transfer. It must assert the original request generation and nonce
survive as a source-free `UnknownStart` request, no activation is available
before matching profile evidence, and the original marker plus matching event
produce exactly one `UnknownStart` grant. Foreign/mismatched profile evidence,
stale request or marker identity, and Reset for an unrelated target must remain
hard refusals. A RED result confirms the suspected first mechanism; it does not
authorize weakening factory, marker, sender, target, or publication checks.

## Cross-profile execution outcome — C10 through C12

C10 is the causal RED recorded above: exact-target Reset before the matching
profile event erased the original request at
`apply_word_reset -> discard_pending_target_word`. C11 then proved the bounded
P2P GREEN for Reset and changed ContentType, including matching, mismatched,
foreign, unrelated-target and stale-identity cases. The same C11 candidate passed
the four independent actual-client cells listed in the current summary. C11's
overall recorder stayed FAILED because its focused run still contained the
separate C28 fixture-order failure; that status does not negate its P2P or client
receipts. C12 changed only the fixture order and executed the final focused set
at **502/502 PASS**. Final review pass 2 accepted this composition at **9/10**,
High 0, Medium 0, with no actionable findings. Runtime authority was unchanged.

## История выбора маршрута и транспортных проверок

**Последний результат, 2026-09-13 02:09 UTC: задача OPEN.** Согласованный
grouped run завершился FAILED: Python harness **36/36 PASS**; Rust discovery
не дошёл до исполнения тестов из-за двух test-only `E0716` в FIFO oracle
(Rust denominator **0 executed**); actual-client **1/2 PASS**. On-профиль
подтвердил exact visible ` пров` и непустой preedit. Off-профиль при ` ljv `
выдал `DeleteSurroundingText(-3,3)` и `CommitText("дом ")`, что oracle отклонил;
cleanup обоих cells чистый. After-audit подтвердил неизменные source, candidate
и девять dependencies, errors 0. Задача не закрыта, authority не менялась.
[Primary RESULT](/home/ubu/.cache/lay/development/td121-grouped-proof-20260913T020000Z-c1-PRIMARY-RESULTS/RESULT.json).

Причина off FAIL установлена в стенде: прежний профиль задавал только
`nanda_autocorrect=false`, оставляя `auto_replace=true` и
`auto_switch_layout=true`. Поэтому наблюдённый exact edit подтверждает неверную
постановку literal-off и не доказывает runtime defect. Исправленный стенд
задаёт false для `nanda_autocorrect`, `auto_replace`, `auto_switch_layout` и
`typing_assist`, сохраняя `nanda_precognition=true`. Его три новых off samples
образуют отдельную группу и не объединяются со старыми off n=3.

До этого grouped запуска дополнительные
проверки выполнены: focused **496 PASS / 5 FAIL из 501**, actual-client
**3 PASS / 6 FAIL из 9**, все девять cleanup чистые. Пять новых source controls
не приняты; два fresh-preedit сценария остановились на неправильном ожидании
NativeUnhandled для managed literal CommitText; второй first-word context
отказал в admission до ввода; без пакетов кандидат начал L3 init и затем
исчез, точная причина раннего выхода ещё UNKNOWN. Установка и authority не
менялись. Formal startup review pass 1 остаётся 9/10; итоговый pass 2 ещё
не проводился. Нового commit/push TD-121 нет.
[Полный актуальный отчёт с деревом, причинами и первичными receipts](evidence/2026-09-13-all-tasks-progress.md#td-121-последние-измеренные-результаты).
Ниже сохранён предыдущий принятый startup checkpoint; его PASS не подменяет
последние FAIL и не закрывает задачу.

После этого FAIL внесён только доказательный test/harness delta. В четырёх
source FAIL наблюдалось `handled=true`; допустимость одного managed CommitText
следует из проверенного runtime path, но точный эффект ещё проверяется и новым
семантическим PASS не объявлен. Четыре новых source controls проверяют
фактическую literal-доставку: NativeUnhandled либо один точный managed
CommitText, без delete/forward/потери/дубля, неверного cursor и без
transfer/stale authority. Их новые ID: `td121_false_global_mode_refuses_authority_and_literal_delivery_is_exact`,
`td121_unverified_bootstrap_failure_is_bounded_and_fresh_literal_has_no_transfer_authority`,
`td121_legacy_letters_and_space_settle_exactly_before_held_get_release` и
`td121_pending_acquisition_owner_loss_rejects_late_completion_and_fresh_literal_is_exact`.
C28 теперь создаёт frame/lease только после exact warmup и реального
autocorrect-on config; последующая controlled config/material смена по-прежнему
обязана отвергнуть старый certificate. Fresh-preedit сохраняет managed Client и
требует точный visible ` пров`, непустой preedit и нулевые delete/forward/дубли.
Нового PASS для этого delta ещё нет; статус остаётся OPEN.

**Sep13, текущий source checkpoint — startup gate, promotion ещё открыта:**
проверенный срез — base commit
`6baa65442227bb31caf32cd56930d9e51559ca71` плюс семь незакоммиченных Rust
файлов, закреплённых archive
`a2820c25cf2e1c484e02f81c1a5078147774d67d50131ac12a81c8d10514c476`.
Этот delta завершает
существующие exact- и L2 one-shot операции до публикации factory/name/bridge.
Один абсолютный join budget равен5000ms; L1.1 readiness в обещание не входит.
Текущий focused `bin:lay-ibus-engine` прошёл495/495 correctness, при этом3
performance tests исключены. Canonical drift состоит ровно из пяти новых
детерминированных startup regressions, без changed/removed tests. На том же
source и release candidate неизменённые `immediate` actual-IBus lanes прошли
manual3/3 и restoration5/5. Старые immediate FAIL ниже остаются историческим
causal baseline и не переименовываются в PASS. Это scoped protocol-client
доказательство, а не quality, startup-latency distribution, GTK или physical
приёмка. Formal fresh review pass1 принял source change:9/10, High0, Medium0;
code repair и второй review не требуются. Canonical full, lints, graph,
autocorrection on/off startup, current first-word, client GTK и physical gates
ещё не выполнены. [Точные identities, timing и receipts](evidence/td121-private-actual-baseline-2026-09-13.md#current-startup-gate-checkpoint--2026-09-13).

В двух `c1` traces warmup join занял1123516us и1124075us. Exact завершился
за109717us и117695us; L2 live readout был полным до join. Первый наблюдаемый
`CreateEngine` factory event следует за `ibus_startup_warmup completed`.
Статический текущий порядок затем регистрирует factory object, запрашивает
`IBUS_ENGINE_NAME` и публикует session bridge. Трейс непосредственно измеряет
границу warmup→первый factory event; он не содержит отдельного timestamp для
name/bridge, поэтому source order не выдаётся за timing proof. Оба значения
около1.12s — два current samples, не p95 и не основание менять5000ms budget.

**Sep13, private actual-IBus baseline:** на frozen source `6baa6544` одна
release-сборка и три unchanged `immediate` lanes дали first-word2/2 и первые
две manual-toggle проверки PASS. Третий manual scenario достиг client-visible
` пров`, но не показал preedit; первый restoration scenario оставил literal
` ljv ` вместо ` дом `, остальные4 restoration cases после assertion не
запускались. Cleanup чистый, runtime/install authority не менялась. Этот
manual failure следует после16 terminal toggles и не является fresh-process
first-prediction proof. На тех же candidate/deps две `post-exact-ready` lanes
затем прошли manual3/3 и restoration5/5. Это измеряет schedule dependence, но
не доказывает готовность candidate memory или причинный source defect; оба
immediate FAIL сохранены. [Точный baseline, comparison receipts и границы
вывода](evidence/td121-private-actual-baseline-2026-09-13.md).

**Sep13, metadata discriminator:** immutable диагностический прогон
`776610cc…afa0f7` подтвердил первый общий механизм. Первый L2 warmup уже был
запущен из `zbus::Connection executor` до exact warmup; поздний lifecycle не
был причиной. Manual epochs52–55 были вовремя применены, но до публикации
candidate memory дали0 candidates; exact был готов за107396us, candidate
memory только примерно через1.14s. Restoration дошёл до Space без exact и
candidate readiness; exact появился через112876us. Значит, engine принимает
первый ввод до завершения двух существующих one-shot операций. Выбран design B:
один ограниченный startup gate до factory/name publication, без изменений
CreateEngine/admission и input budgets. Числовой preflight и полный receipt
находятся в том же evidence-документе; реализация и promotion ещё не доказаны.

**Sep8, последнее сообщение пользователя:** Double Shift не работает на первом
слове без ведущего пробела. На установленном fe643c10 новый private-client
baseline0/2 FAIL. Минимальный terminal-only допуск наблюдённого suffix реализован
в существующем WordLineage; UnknownStart не получает automatic/generic authority.
Review pass1 выявил stale atomic prefix и numeric delegation; оба дефекта дали
causal RED и исправлены. Focused448/448 PASS, run-TcpGeO,15.463s; review2
9/10,H0/M0/L0; changed2683/2683. На6dc95148 отдельные холодные US/RU старты
прошли2/2,16 точных преобразований; manual/preedit3/3, lifecycle3/3,
restoration5/5. Совмещённая смена поля+профиля до текста остаётся1/2 FAIL,
её причина и baseline parity не доказаны. Full gate2683/2683 PASS495.756s.
Sep8 11:50:25 +03:00 установлен6dc95148, PID2836850; физическое подтверждение
запрошено и пока не получено. Только explicit terminal manual eligibility
изменена; это не закрытие всего lifecycle/GTK/cold-preedit контракта.
Точные доказательства и ограничения — в owning journal ниже.

**Sep8, новое наблюдение пользователя:** Double Shift срабатывает один раз,
обратное преобразование не работает. Пользователь снова разрешил тесты.
Текущий узкий маршрут — settlement вывода bridge перед передачей IME;
[наблюдение, последствия и результаты](evidence/release-1.0.66-double-shift-recurrence.md).
Прежний запрет на тесты ниже исторический. Установленные bytes пока прежние.

**Sep8 04:31 +03:00, результат:** установлен `fe643c10` с исправлением
bridge settlement и повторного ContentType. Focused442/442, полный release
gate2677/2677, review9/10,H0/M0; реальный клиент с текущими локальными
зависимостями:16 преобразований на сценарий startup schedule, post-ready
manual/preedit3/3, lifecycle3/3, restoration5/5. Немедленный cold preedit
остаётся FAIL; физическое подтверждение пользователя ожидается. Точные
ограничения и установка — в журнале выше. Старые результаты ниже исторические.

**Sep8,03:05:52 +03:00, последний результат:** по прямой команде пользователя
тесты прекращены; свежая release-сборка последних исходников12db8c00
установлена постоянно и работает как IME147597. Сборка только IME удалённо,
jobs20,58.72s; IBus4715/daemon3757261 не перезапускались. InputState отвечает
`passive:daemon-word-buffer`. Реальный ввод ещё не подтверждён; это не DONE
и не закрытие cold-start. Полный correctness/package прогон успел пройти
2672/2672, но общий release script завершился ошибкой clippy, не PASS.
[Точная точка продолжения без повторного цикла тестов](CONTINUE.md).
Описанные далее «следующие этапы» — исторические планы до этой команды.

**Sep8, текущий маршрут:** после полного чтения2878 строк пользователь
потребовал минимальный надёжный ремонт адаптера. Незавершённая отдельная
debug-надстройка остановлена, существующий debug_action_log уже включён.
Выбран локальный ремонт: отказ reducer в CreateEngine не должен завершать
общий observer и не должен выдавать этому callback никакого допуска.
Подробные последствия/границы записаны в owning journal до правок.

Минимальный ремонт теперь подтверждён causal RED/GREEN:437/437 focused PASS;
независимое fresh-context review9/10,H0/M0. Оптимизированный кандидат
e596b507 собран удалённо, архитектурная проверка PASS. Следующий этап —
обязательные affected/release и actual-client проверки этих bytes, затем
контролируемая физическая проверка с откатом; постоянной установки ещё нет.
Это не закрывает прежний cold-start FAIL или отдельный physical FAIL706b.

**Sep8, финальный source checkpoint:** широкий прогон выявил гонку ожидания
в существующем тесте; oracle выровнен по реальному outbound marker, без sleep
и ослабления assertions, review10/10. Лишнее production-поле идентичности
готового запроса оставлено только под cfg(test), поскольку его единственный
потребитель — тестовый witness; финальное review10/10,H0/M0. Adapter60d50d52,
residuals28dd8b76, focused437/437 PASS. Последующий release gate завершился
ошибкой clippy; текущая установка и границы результата описаны выше.
Все неуспешные промежуточные проверки сохранены в owning journal.

**Sep8,01:30 +03:00:** пользователь подтвердил, что сам перезапустил
установленные Lay/IME. Текущий IME3757358 — старый установленный95348e4a,
не706b preview; InputState снова cancelled. В его сохранённом журнале
CreateEngine98 отказан после состояния transfer_revoked (owner5/activation5).
Это не устанавливает причину отдельного отказа706b. Новых действий с
сервисами при этой проверке не было; optional debug ещё в реализации.

**Sep8,01:11 +03:00:** пользователь сообщил «не работает!». У706b observer
снова cancelled; preview остановлен после возврата на native xkb:ru::rus.
IBus/keyboard daemon сохранены.500KiB trace вытеснил начало отказа потоком
повторных отказов. Пользователь явно запросил debug-режим как отключаемую
опцию; выбран bounded first-failure metadata recorder, default OFF. Это
диагностическая правка, не доказанный новый ремонт переходов.

**Исторический preview,22:18:58 +03:00:** после разрешения пользователя
706b4d81 запущен из cache как PID2841233, установленный файл не заменён.
Старые IBus4715/daemon272240 сохранены. Plain FocusIn работает:242/242
key admissions и242/242 settlements приняты, observer отвечает, preedit
показывается. Это не подтверждение автозамены/Double Shift: ручная приёмка
нового кандидата ожидается; предыдущий physical FAIL сохранён ниже.

**Принятый ремонт,2026-09-07:** пользователь выбрал минимальное исправление
переходов в существующем reducer. Подтверждены RED для refocus без Disable,
перекрытия пустых factory и трёх случаев потери исходного Get при ранней
клавише/Reset/ContentType. Финальный runtime:436/436 focused и2671/2671 affected
PASS, независимое source review9/10,H0/M0. Оптимизированный706b4d81:
реальный lifecycle3/3, restoration после exact-ready5/5; immediate cold0/5
(нет замены первого слова). Старый15728764 также проходит corrected lifecycle3/3:
прежний0/3 был ошибкой ожидания теста для A->dummy->A, не causal RED runtime.
Тест исправлен без изменения production-кода;29/29 tooling PASS, финальное
review исправленного witness9/10,H0/M0. Causal RED
остаются controlled production-callback regressions. Установки, новых
перезапусков на этом source-only checkpoint, commit/push и успешной физической
приёмки этого ремонта не было. Поздний разрешённый preview указан выше.
Проверенный срез20:44: выбран `lay-ime-ru`, процесс установленного95348e4a,
InputState=`metadata observer cancelled`. Native fallback ниже исторический.
[Текущий owning journal и точные RED receipts](evidence/release-1.0.66-factory-recurrence.md#accepted-systemic-lifecycle-repair-2026-09-07-2044-0300).

**Физический отказ,20:24 +03:00:** пользователь сообщил «неработает».
Кандидат15728764 остановлен после проверенного возврата на `xkb:ru::rus`.
151/151 retained key admissions отказаны; observer затем остановлен на
CreateEngine85. IBus4715/daemon272240 неизменны. Предыдущие PASS ниже не
доказывают работу на рабочем столе. Сохранённая1.0.65 проверена только по
наличию/версиям; откат не выполнен и требует отдельного решения пользователя.

**Исторический checkpoint,20:06 +03:00:** временный IME15728764 запущен из cache,
PID2128979; установленный бинарник не заменён.425/425 focused, review9/10,H0/M0,
post-exact-ready client5/5. Immediate cold client FAIL: первый Space раньше
готовности словаря. Bridge=passive:no-focus; физическая проверка ещё нужна.
Глобальный IBus4715 и клавиатурный демон272240 не перезапускались.
[Единый актуальный журнал ремонта и безопасного запуска](evidence/release-1.0.66-factory-recurrence.md).
Результаты ниже исторические; задача не DONE, новый commit/push не выполнен.

**Новый живой отказ,17:17 +03:00:** старые процессы исключены по хешам;
тот же новый IME повторно остановил observer, теперь на `CreateEngine123`
после `transfer_revoked`.526/526 последующих key admissions отказаны.
[Точная последовательность и отдельный ограниченный переплан](evidence/release-1.0.66-factory-recurrence.md).
Предыдущая установка не прошла физическую приёмку; задачу не закрывать.

**Обновление 2026-09-07, 17:03 +03:00:** исправленный IME установлен,
PID1148501, SHA256 `95348e4a...23a668`; bridge отвечает `passive:no-focus`.
416/416 focused, 2651/2651 changed, exact release client5/5, оба lint scope и
architecture PASS; финальное review9/10,H0/M0. Глобальный IBus4715 не менялся.
Физическая проверка пользователем ещё не выполнена, задача не DONE.
[Полные доказательства и два исхода установки](evidence/release-1.0.66-live-observer-incident.md).

**Инцидент 2026-09-07, 16:10 +03:00:** живой IME возвращал
`metadata observer cancelled`; журнал подтверждает остановку обработчика
контекста на `FocusOut` и последующий отказ всех 490 сохранённых key callbacks.
Предыдущие PASS ниже не являются доказательством работоспособности текущего
ввода. [Причина, альтернативы и границы исправления](evidence/release-1.0.66-live-observer-incident.md).

Исторический результат до инцидента: V2 actual-client **5/5** на final release-profile candidate
`dfeb50e8` после явной готовности exact-контура. Полный ` ljv` становится
` дом ` без потери пробела; смешанное слово сохраняется при US→RU и RU→US;
чужое поле и UnknownStart не получают права исправления. Remote changed и full
gates: каждый **2646/2646 PASS**, включая IME411 и protected7. Исправлены lifetime готового результата, enrichment позднего
native receipt для Transfer, отмена незавершённого пустого контекста новым
factory и запрет поздней публикации отменённого fence. Все исправления
связаны с RED/GREEN; pending→reducer порядок блокировок сохранён.
[Точные receipts, анализ последствий и review](evidence/release-1.0.66-final-client-route.md).
Холодные0/5 остаются отдельными failed runs, не заменены post-ready PASS.
Смешанная коррекция как отдельный verdict и физическая клавиатура этим стендом
не доказаны. Lint/architecture/full gates и exact release-profile client прошли;
комплект перенесён с полной SHA-256 проверкой, rollback snapshot готов.
По последующей прямой команде пользователя1.0.66 установлена с rollback
snapshot; загруженные процессы и их SHA-256 проверены независимо.
[Установка](evidence/release-1.0.66-installation.md). Впереди physical
confirmation; статус пока не DONE. Установленная и запущенная версия1.0.66.

Предыдущий source checkpoint: [независимая проверка четырёх исправлений](evidence/td121-residual-acceptance-review.md)
**9/10,H0/M0**, exact current source/retained tests проверены. Семь residual
tests и оба actual-Tab readiness tests присутствуют и PASS в IME402/402.
Защищённый composition successor принят как source checkpoint, не как релиз.
Последующая механическая lint-правка принята10/10,H0/M0 и перепроверена
обоими финальными gates. Следующий обязательный шаг — физическая проверка.
Описанные ниже4/10 и PROPOSED — исторические checkpoints до этой перепроверки.

Продолжение пользователя 2026-09-07: «не останавливайся к 66 версии иди».
Ограниченный план завершения: независимая перепроверка только четырёх уже
исправленных High из pass2 и их causal regressions (не третий общий аудит и
не новый неограниченный repair); отдельный явный successor некорректного
client contract; затем source-bound promotion, canonical/release gates и
проверенная доставка. Оценка TD-12510/10 не переносится на TD-121. Старый
driver и его0/5 остаются историческими доказательствами. Сначала установить
причину cold exact refusal; не повторять старый клиент и не увеличивать waits.
Все сборки/тесты/обновления графа только на remote под прежними ограничителями.
Само продолжение не переводит PROPOSED в ACCEPTED и не разрешает reboot IBus.

Новейший diagnostic checkpoint2026-09-07: отдельный `NameHasNoOwner` после
переноса стенда объяснён отсутствующим L2 lexical artifact; один hash-bound
dependency устранил SIGABRT, production-код не менялся. Обычный новый runner
с nine-role manifest оставляет IME живым и снова показывает `prefetch_not_ready`,
client0/5. Это не acceptance и не третий runtime repair.
[Доказательства и новый отчёт о потере пробела](evidence/td121-client-boundary-diagnosis.md),
[TD-125](125-preserve-autocorrection-left-boundary.md). Терминальный consumer
и противоречие capabilities/assertions frozen driver требуют отдельного proof.

Текущий checkpoint 2026-09-07: IME399/399 PASS за15.49s, final-pass residual7/7.
Настоящий клиент подтвердил перенос полного префикса ` l` → ` ljv` между
экземплярами в одном контексте. Полных cases по-прежнему0/5: и debug, и
оптимизированный кандидат отказали с `prefetch_not_ready`; бюджет не расширен.
[Точный результат](evidence/td121-private-client-proof.md#full-prefix-handoff-pass-cold-debug-correction-not_ready--2026-09-07).
[Final review pass2](evidence/td121-code-review-pass2.md):4/10,H4/M0. Нового
общего круга ревью нет: один явно ограниченный второй repair существующих
инвариантов; при новом непокрытом механизме остановить этот план и сообщить.
RED/исправление/GREEN четырёх receive-order/completeness дефектов выполнены.
Оптимизированный client proof остался красным; очередной такой же прогон
не запускается. Требуется различить холодную lexical readiness и иной отказ
exact-сертификата. Дополнительная узкая проверка исправлений независимым
ревьюером отдельно запрошена у пользователя: текущая оценка4/10 до исправления
не заменена выдуманными8/10 и release-gate не ослаблен.
Релиз/install/DONE пока запрещены.

Исторический checkpoint до legacy proof: combined adapter22/22 и IME390/390 PASS;
кандидат `b7e78375…24e81c`. Изолированный настоящий IBus-клиент завершил0/5
cases: первый literal Space виден клиенту, но следующий snapshot остался
`passive:unknown-context`. Source-free activation установлена; порядок
marker/key и причина отказа legacy callback не записаны. Поэтому этот прогон
не различает ожидаемый ввод до готовности и дефект уже готового legacy пути.
Новые word-scope tests исполняют atomic callback, а не этот legacy envelope.
Следующий узкий шаг — доказать оба порядка на actual legacy entrypoint и
добавить недостающую causal metadata; никаких новых RPC, authority owners,
таймаутов или ослабления positive assertion. Подробности и последствия —
[current analysis](evidence/td121-context-admission-analysis.md#legacy-client-first-boundary-checkpoint--2026-09-07).
Принятые правила разработки записаны в `AGENTS.md`; это не допуск широкой
миграции в1.0.66. Исторические checkpoints ниже не являются final acceptance.

Checkpoint 2026-09-06: frozen reducer/ordered-zbus/rendezvous/adapter —
30/30 PASS на exact zbus 5.15.0: 25 reducer/merge/rendezvous и 5 actual-zbus
controlled p2p tests. Это supersedes прежние helper-only 19/19, но остаётся
отдельным staged source, не подключённым runtime. Production wiring,
ProcessKeyEventAtomicV1/ContentType headers и client proof этим не покрыты.
Evidence: [helper/adapter record](evidence/td121-pure-helper-implementation.md).
Дальнейшее подключение описано в [wiring handoff](evidence/td121-runtime-wiring-sol-task.md)
и начинается после TD-120 source checkpoint и adapter gates. Ни исправление
production input, ни релиз/install/push этой записью не объявляются.

TD-120 checkpoint закрыт и запушен:
`ad4bf0860cc2a79b3004f03b8018cc8d8cbca005`. Подключение TD-121 начато;
изменения adapter для AtomicV1/Properties.Set и actual composition guard
требуют собственных tests/review. Frozen 30/30 не покрывают эти новые bytes.
В context-admission analysis отдельно согласован узкий TD-121 successor
для protected composition source; TD-113 и TD-120 история остаётся неизменной.

Поздний checkpoint 2026-09-06: remote `lay-ibus-engine` suite 379/379 PASS
за 14.35s; [точный лог и SHA-256](evidence/release-1.0.66-execution.md).
Это предшествующие bytes: проверка родителя выявила разрыв receive-order
metadata / handler settlement и отсутствие native-path проверки FocusOutId.
Идёт один ограниченный repair этих owner/ordering инвариантов. Промежуточный
standalone 1.0.66 собран удалённо для client diagnostic, но не допускается
к установке. Реальная correction-on проверка, C/H ledger и независимое
финальное ревью остаются обязательными; 379 не является release denominator.

[Независимый pass 1](evidence/td121-code-review-pass1.md): **3/10, High 6,
Medium 2, REPAIR_REQUIRED** для frozen snapshot до текущего repair. Замечания
охватывают receive-order authority, seal/Disable epoch, old-owner shared writes,
completeness по реальному effect, atomic prior settlement, layout-request
identity, acquisition timeout/locking и native enrichment. Это static review,
не исполненные repro. Следующий проход — второй и заключительный; task DONE
и successor ACCEPTED запрещены до фактического закрытия замечаний и gates.

[Матрица закрытия C01–C30 / H01–H16](evidence/td121-promotion-coverage.md)
разделяет source pointers, исторические прогоны и ещё открытые проверки
финальных bytes. Число строк матрицы не является числом выполненных tests.

[Полный Astra/XHigh analysis](evidence/td121-context-admission-analysis.md)
выбрал единый canonical context contract: bounded metadata observer на той же
IBus connection вне speculative SharedState; native receipt либо необходимый
cached-false property adapter; один self-signal marker через очередь IBus.
Factory/focus ticket, receive/settlement ordering, generation revocation,
sticky UnknownStart и bridge/atomic/layout consumers входят в один контракт.
Оценка8/10 — проектное решение, не результат implementation review.

[Изолированная проверка установленного настоящего IBus](evidence/td121-real-ibus-transport-baseline.md)
выполнила22/22 транспортных cases. Marker после foreign ABA наблюдён20/20;
Get+marker537–1108µs на21 observation, не production percentile. Рабочий IBus
не перезапущен. Adversarial early-reply ordering0/20 наблюдений, callback/ring/
completeness и positive Lay restoration ещё требуют отдельных tests.
Этот результат не переводит C01–C30 в PASS и не закрывает задачу.

Финальный addendum того же анализа ограничил оставшийся допуск тремя tests:
P121-1 actual reducer с задержанной FIFO и положительным контролем;
P121-2 настоящий zbus OrderedStream/Join + четыре lost-wake schedules;
P121-3 sender identity и общий бюджет Space. Чистый helper/tests разрешены до
wiring. Initial missing stamp ждёт только локальное уведомление≤1ms, не RPC;
затраченное время вычитается из существующих3500µs Space wait. Не добавлять
этот budget поверх прежнего. Production authority подключается после P121-1/2/3;
полная positive restoration и C01–C30 остаются обязательными release gates.

Отдельный real-IBus header run23/23 уточнил P121-3: Factory/lifecycle methods
приходят от `org.freedesktop.DBus`, GlobalEngineChanged — от unique owner IBus,
self-marker — от собственной connection. Реальная попытка подделать Sender
другой connection переписана IBus в её настоящий sender. Не сравнивать все
три роли с одним unique name и не hardcode-ить ephemeral names. Artifact/hash
и непроверенный ProcessKeyEvent scope — в том же transport baseline документе.

## Что должно измениться

При переключении раскладки внутри одного поля текущий **полный** токен не
должен терять уже введённое начало. Если приложение отображает `ljv`, нельзя
воспринимать только `jv` как целое слово и исправлять его отдельно в `ом`.
Переход в другое поле, напротив, обязан уничтожать старое право на его текст.

Запрещены восстановление невидимой буквы из словаря, прибавление Backspace,
угадывание по pixel cursor, чтение старого DaemonWordBuffer вместо IME lease,
автоматический повтор изменения и ослабление SafetyGate/verifier.

## Подтверждённый production-эпизод и предел доказательства

Сохранённые журналы 1.0.65, 2026-09-05 12:12:12–12:12:14 EEST:

```text
ctujlyz → сегодня          Applied, 7 символов
ru layout sync ok
us layout sync ok         origin этого события не записан
новый engine us/57
его tail перед j пуст
j → jv                    длина 1 → 2
Space: jv → ом             Applied, backspaces=2
пользователь видит lом
```

Точно установлено: на correction boundary вошёл неполный по наблюдению
пользователя токен. Первый установленный разрыв находится **до**
`InputFrameIdentity → L1.1/L2/L3/L4 → DecisionCore → verifier`, в наблюдении и
переносе текста. Поздние слои проверили замену `jv`, а не `ljv`.

Не установлено: дошла ли именно историческая `l` до старого engine, прошла
native passthrough во время смены, либо её запись потерялась в lossy logger.
Нет времён каждого key/focus события и origin переключения. Координата курсора
не доказывает символ. Не называть этот эпизод timeout смешанного `lом`:
mixed-script candidate в нём вообще не является наблюдаемым входом.

## Системный дефект, установленный в коде

1. `ibus_interface.rs::focus_id` возвращает **false**, хотя `FocusInId` и
   `FocusOutId` реализованы. Это не доказанное ограничение Kitty: сам Lay
   объявляет отсутствие capability. Линия существовала с commit `427b3d7e`
   от 2026-06-19, не является доказанной регрессией только релиза 1.0.65.
2. `FocusOut → should_preserve_focus_handoff` сохраняет свежий ввод до 700 ms.
3. Новый `LayIbusEngine::new_from_component` копирует shared tail.
4. Но `FocusIn → bind_focus_path` принимает копию только при действующем
   `preserve_active_path_until`. Без него очищает **локальный и shared** хвост.
   Совпадения настоящего input-context receipt здесь недостаточно.
5. Обычный modifier layout hotkey не открывает такую же preserve lease, как
   text replacement. Сохранить на старом объекте и принять на новом — разные
   критерии. Проверка только старого объекта этот разрыв не обнаруживает.
6. `LayoutSwitchRequest` хранит target/engine без request generation и focus
   identity. Latest-only отменяет ожидающий request, но не уже выполняющийся.
   Это дополнительный race-risk; его участие в исторической `l` НЕ доказано.

Один из существующих тестов проверяет сохранение при reset старого объекта;
другой — правильное отбрасывание expired lease. Нужен тест передачи **между
двумя объектами** в одном и в разных канонических контекстах.

## Канонический источник identity и ограничение hot upgrade

Установлены `ibus` / `libibus-1.0-5` версии `1.5.34~rc2-1`.
Upstream source соответствующего tag `1.5.34-rc2`, commit
`1f7af28437afd62a6d145bfc81035e698a37411d`:

- `bus/engineproxy.c` читает `FocusId`, кэширует ответ по engine name,
  выбирает `FocusInId(context_path, client)` либо обычный `FocusIn`.
- `bus/ibusimpl.c` создаёт focus capability table при init и уничтожает при
  destroy; в просмотренном файле не найден публичный reset этой таблицы.
- Тот же IBus предоставляет read-only property `CurrentInputContext` с
  object path текущего канонического input context. Старый одноимённый метод
  deprecated; для нового кода использовать property, если путь будет принят.

Пять live read-only запросов property 2026-09-05 вернули один context path.
Round-trip: `2630, 632, 399, 280, 258 us` (первая проверка отдельно).
Это **не p95/p99**, не нагрузочный тест, не доказательство handoff или
неизменности focus между двумя ответами.

Следствие: простая правка `FocusId=true` может не изменить работу уже
запущенного IBus после перезапуска только Lay. Global IBus PID сохраняется;
рестарт IBus/session, новые engine names и изменение пользовательского списка
источников нельзя незаметно включить в этот fix.

## Варианты решения и рейтинг

Оценки ниже — design judgement до baseline proof, не разрешение реализации.

| Вариант | Оценка | Применимость и цена |
|---|---:|---|
| Native `FocusInId` capability + передача по совпадающему каноническому context | **9/10 архитектурно; 6/10 для текущего hot upgrade** | Наименьший постоянный механизм; надо доказать activation при кэшированном false без перезапуска глобального IBus |
| Тот же context-bound transfer; bounded `CurrentInputContext` property adapter там, где native receipt ещё недоступен | **8/10 условно** | Не требует нового owner текста или engine names; дополнительный RPC только на lifecycle transition. Необходимо доказать deadline, freshness и A→B→A, иначе не допускается |
| Единственный shared input-context state вместо local копий engine | 6/10 | Более широкая миграция edit/undo/prefetch/atomic/locks; второй этап, identity всё равно нужен |
| Продлить TTL либо переносить хвост при любом новом path | 2/10 | Нельзя отличить смену раскладки от другого поля; риск правки чужого текста |
| Переводить suffix `jv` и потом достраивать начало | 1/10 | Маскирует потерю наблюдения, создаёт вторую мутацию; запрещено |

Рекомендуемый принцип: **существующее состояние IME + канонический context
IBus + один generation-bound handoff**, не таймер как доказательство поля.
Способ получения canonical receipt остаётся admission gate: сначала native
handshake, затем только при доказанной необходимости bounded compatibility
adapter. Не реализовывать оба способа «на всякий случай».

## Обязательные gate до production-кода

1. Reproduce `l → new engine → [empty] → jv/ом` установленными байтами или
   source-bound тестом двух engine. Подтвердить первую букву до перехода и
   хвост после него. Same-path и different-context контроли обязательны.
2. Зафиксировать ровно один выбранный способ получения context receipt на
   **уже работающем** IBus и поведение при cold capability discovery.
3. Разрешить callback ordering: `FocusIn`, delayed `FocusInId`, `Enable`,
   `Disable`, old-path `FocusOutId`. Старый FocusOutId другого context не
   закрывает новый; одинаковый контекст не равен старому request generation.
4. Доказать отсутствие переноса на другой input context при одинаковом окне,
   app name, тексте и быстрых событиях. `A → B → A` не оживляет прежний lease.
5. Для letters during source transition нужна отдельная проверка delivery.
   Сохранение уже наблюдённого `l` не доказывает доставку клавиши, отправленной
   когда IBus переключает factory. Нельзя обещать устранение этой второй
   гипотезы без client-visible/physical evidence.
6. Согласовать реализацию со scope suppression TD-120 и пройти spec review.

Исторический checkpoint до context-admission analysis: phase1 gate1 был открыт;
private probe остановился до RPC из-за AppArmor,
0/6. В phase2 отдельный безопасный bootstrap выполнил6/6 сценариев;
[новая baseline](evidence/td120-121-installed-baseline-phase2.md) воспроизводит
потерю уже наблюдённой `l` между объектами, включая same FocusInId, с
same-path/different-context контролями. Gate1 закрыт для класса tracking,
не для точной исторической доставки. Тогда gate2 и ordering/production gates
были открыты (`ANALYSIS_REQUIRED`). Этот admission status superseded полным
context-admission analysis и последующим 30/30 frozen helper/adapter proof;
подключение production допускается после TD-120 Git checkpoint. Production
acceptance и client/physical gates по-прежнему обязательны.

## Эскиз внутреннего контракта после admission

- Разделить `engine object identity`, `canonical input context identity`,
  `word lineage`, `layout request generation`, `candidate frame generation`.
  Не использовать одну строку path или один таймер вместо всех пяти.
- Смена engine в том же подтверждённом context сохраняет актуальные текст и
  word lineage; все старые кандидаты/certificates/preedit acceptance права
  инвалидируются. Перенос текста не переносит право применения результата.
- Handoff одноразовый, привязан к origin/target/request/context. Внутренний
  таймаут ограничивает ресурс, но не доказывает совпадение поля.
- Новое поле, no-focus, sensitive purpose, несовпадающая/просроченная receipt
  не наследуют текст. При неопределённости нельзя исправлять неизвестный
  suffix как доказанно полное слово.
- Не делать get-context RPC на каждом символе. Переход имеет bounded
  acquisition; timeout не блокирует клавиатуру и не запускает retries.
- Смена раскладки остаётся одним GNOME-owned действием. Не добавлять второй
  `ibus engine` после ActivateLayout, ещё один detector или uinput replay.
- Нельзя считать чужой более поздний decoder/layout результат разрешением
  текущему engine. Pending и уже in-flight switch проверяются раздельно.

## Consequence analysis

| Измерение | Ожидание, риск и требуемое доказательство |
|---|---|
| Retention / completeness | Уже наблюдённые символы сохраняются только в том же context. Неполный хвост не становится полным через догадку; отдельно проверять незарегистрированные первые клавиши |
| Ranking / false authority | Лексическая конкуренция неизменна; вход становится полным. Копировать candidate authority между engines нельзя |
| Latency / tail | Native receipt предпочтительна; дополнительная property query только на lifecycle, с отдельным измерением cold/warm/failure. Никакого увеличения Space deadline ради handoff |
| CPU / RSS / allocations | Один bounded transfer record у существующего SharedState; без новых resident workers, очередей клавиш, polling или копий corpus |
| Cache / identity | 700ms не identity; FocusId capability cache живёт в IBus. Bus reconnect, old reply, same path/different context, A→B→A требуют отрицательных тестов |
| Package / reload | Material generation и frame сертификаты продолжают инвалидироваться; context receipt не зависит от версии словаря |
| Learning | Источник typed/accepted/censored остаётся привязан к реальному полному слову и edit receipt. Не обучать `jv` как полный ввод, если scope неполон |
| Concurrency | Порядок DBus callback/worker request закреплён тестами, не sleeps. Locks не удерживаются во время внешнего вызова и не нарушают atomic clone/commit |
| Failure / rollback | Нет focus receipt — нет догаданной мутации. Failed switch не делает второй replay и не переносит хвост в другой контекст. Откат не требует удаления словарей |
| Consumers | Both Shift/Alt orders; ManagedCommit width11, narrow TerminalPassthrough, GTK SurroundingText, daemon exact replay, Tab/undo проверяются отдельно |
| Maintenance | Native callback и optional compatibility acquisition сходятся в одну типизированную identity, не в два конкурирующих решения. Full shared-state rewrite не допускается этой задачей |

## Матрица TDD и доказательств

Записать manifest до изменения поведения; фиксировать число конкретных
раскрытых cases, а не выдавать строки таблицы за число выполненных тестов.

| ID | Обязательные ветки | Что проверяется |
|---|---|---|
| H01 | US→RU, RU→US, US→US new path | Сохранность уже наблюдённого полного токена и точные output effects |
| H02 | Same path quick FocusOut/FocusIn | Нет регрессии существующего сохранения |
| H03 | Новый path / тот же FocusInId | Canonical identity важнее object recreation; старые candidate frames непригодны |
| H04 | Different context, same app/window/text | Нулевой перенос текста/guard/edit authority |
| H05 | A→B→A / старый FocusOutId | Старый lease не воскресает и не очищает новое состояние |
| H06 | Перед 700ms / после 700ms и delayed callback | Bounded ресурс не подменяет identity; управляемое время, не flaky sleep-тест |
| H07 | Shift→Alt, Alt→Shift, пользовательский layout intent после auto-sync | Один owner, последнее действующее намерение; нет двойной смены |
| H08 | Pending switch / уже выполняющийся switch | Раздельная invalidation, без stale decoder authority |
| H09 | Буква до / во время / после смены | Доставка и tracking считаются отдельно; потеря/дублирование 0 в проверенном клиенте |
| H10 | ManagedCommit width11 / TerminalPassthrough width2 | Не путать profile с названием приложения либо purpose=terminal |
| H11 | SurroundingText on/off; FocusId cold/warm cached false | Реальные capability combinations; no global IBus restart |
| H12 | No context / disconnected bus / slow query / stale query | Нет опасного fallback, клавиатура не зависает |
| H13 | Mixed token физическая проекция, clean English, protected tokens | Полный frame проходит обычную конкуренцию, не lexical exception |
| H14 | Double Shift / 4 taps / accepted Tab / auto-undo | Один detector, один edit, round-trip без append/reversal |
| H15 | Package reload / config change между enqueue и apply | Старые correction certificates не применяются |
| H16 | Atomic prepare / abort / commit | Нет live side effects до commit и повторного расходования lease |

Для каждого case записывать отдельно: observed keys, CommitText stream,
client-visible text (если измерен), internal tail, input context, engine path,
word/request/frame generations, exact edit plan, verdict, left-context effect.

`ljv → дом` и `lом → дом` — разные корпуса: полностью неверная раскладка и
уже смешанный ввод. Не заменять проверку первого frameless CLI второго.

## Проверки / установка / review

### C18 GUI diagnostic: первый доказанный reducer

GUI diagnostic C18 наблюдал production-порядок `CreateEngine -> FocusOut ->
Disable -> FocusIn -> Enable -> marker -> transfer_ready`; первый target затем
установил transfer. Следовательно, это не постоянная потеря ContextAdmission
owner/receipt и не доказанный timeout acquisition.

Пустой receipt создаёт `VisibleTailV3` после успешного bridge fence. У него два
fail-closed выхода с таким результатом. Первый проверяет active path, live
bridge token и `KnownStart || bounded UnknownStart`. Второй применяется к
`ImeCommittedTail` с exact handoff: он требует live handoff и совпадающий
external `SurroundingTextSnapshot`. В C18 между target `FocusIn/Enable` и
`transfer_installed` отсутствует `SetSurroundingText`; новый engine создаётся с
`surrounding_text_snapshot=None`. Поэтому первый установленный reducer пустого
ответа — проверка fresh-target external snapshot, а не ContextAdmission
publication. Трассы не имеют общей subsecond шкалы, но этот вывод не зависит от
относительного времени daemon: target не получил snapshot ни до, ни после
установки transfer в проверенном интервале.

Контролируемый RED
`td121_same_context_target_without_a_fresh_snapshot_keeps_the_controlled_handoff_lease`
проводит production adapter/factory/focus/marker path, доказывает отдельно:
same canonical context, live target token, совпавший active path, bounded
UnknownStart, live exact handoff и отсутствие target snapshot. Текущий
`VisibleTailV3` затем очищает authority и возвращает empty receipt. Ожидаемый
GREEN сохраняет ровно этот generation-bound lease до первого optional target
snapshot; different-context, mismatch, selection и late mismatch остаются
отказами.

Варианты production-механизма:

| Вариант | Следствие |
|---|---|
| Перенести доказанный source snapshot в target только внутри same-context transfer grant | Узкий предпочтительный механизм: один context/request generation; требует отрицательных A->B->A, different-context и late-target-mismatch proof |
| Привязать exact lease к source snapshot receipt и разрешить его однократное подтверждение target без копирования общего client state | Меньше state migration, но нужен типизированный provenance и такое же доказательство invalidation |
| Сохранить требование нового target callback | Fail-closed остаётся безопасным, но C18 Double Shift ложно отказывает в клиентах, где callback после смены не приходит |

Нельзя считать сам same-context достаточным для мутации, переносить generic
snapshot между engines, добавлять RPC/polling/sleep/retry или ослаблять
SafetyGate/verifier. Candidate generation и ranking не затронуты; production
authority не меняется до GREEN fixed proof и повторного GUI gate.

Реализованное решение: source создаёт отдельный
типизированный exact-manual snapshot receipt только из фактически полученного
supported, unselected и boundary-matching `SurroundingText` под live source
token. Receipt связывает connection/revocation, source owner/activation,
lineage, tail epoch/text и observation revision. Same-context `TransferGrant`
может перенести его ровно один раз к точным target token/owner/epoch и текущей
target observation revision. Он не записывается в generic
`ClientContextState::surrounding_text_snapshot` и не разрешает automatic
correction.

Любое новое target observation, включая `None`, mismatch или selection,
смена context/token/epoch/tail, expiry, source-free/reset, consumption либо
переход A->B->A делает receipt непригодным. Старый source callback не может
отозвать receipt другого owner. Positive scope ограничен `VisibleTailV3` и
следующей exact suppression/replay lease; после consumption повторного права
нет. Это сохраняет существующий deadline, SafetyGate и verifier.

Авторитетный RED выполнен на `e@192.168.3.94` из
`/home/e/projects/lay-development-runner/td121-target-snapshot-red-20260913T-current/source`
под `dedicated-20cpu`, jobs 20, test_threads 1 и 12 GiB Cargo guard. Команда
завершилась ожидаемым `rc=101`; полный log `../RED-PRIMARY.log`, SHA-256
`7f056e9306dd41a5413d7463a4a18caca3132baa991fe3d8160ecf6571a41dfc`.
RED получил `passive:unknown-context` с пустым receipt после всех положительных
path/token/UnknownStart/exact-handoff preconditions.

Первый механизм после исправления прошёл guarded grouped proof на том же remote
host в `td121-typed-snapshot-c19-current`: remote rustfmt `rc=0`; шесть focused
test functions, **15/15 concrete cases PASS**. Denominators: 1 positive
no-target-callback `VisibleTailV3 -> exact suppression` с однократным
consumption; 1 post-install capability invalidation; 3 pre-install target
observations (`None`, mismatch, selection); 1 stale-foreign publish; 9
сохранённых Cycle09 external-tail cases. Primary hashes находятся в
`RECEIPTS-51.sha256`; positive log SHA-256
`6a81b47d3a2a60213dc856730fa96bde96d16b8dfdcefb5f12cce2e68b523d74`,
pre-install controls
`6aff06e999a88e151cd873febd2b0b2b31a7d50eb01dad3f0944273308f6102f`,
Cycle09 matrix
`49be9f9b2d901caa44754c16eb63ef07f3cd3ea1ba887df44abce3d57c266f20`.
Это focused mechanism proof; GTK, full release и human physical authority он не
повышает.

### C18 autocomplete: effect settlement on key release

Проверен production `ProcessKeyEvent` adapter для фактического C18 stuck-tail
маршрута: source имеет `UnknownStart`, наблюдённый хвост `abc`, пустую active
composition, видимый выбранный suffix `def` и точный свежий surrounding snapshot.
Нажатие Alt не обработано; отпускание Alt успешно выдаёт единственный
`CommitText("def ")`, а owned tail становится `abcdef `. Текущий reducer всё же
оставляет `UnknownStart`, потому что `advance_context_word_scope` выходит до
анализа результата для любого release callback. Поэтому следующий точный
`ManualToggleV3` не может получить право, хотя append и word boundary уже
произошли в авторизованном callback.

Authoritative test-only RED выполнен 2026-09-13 на `e@192.168.3.94` в
`/home/e/projects/lay-development-runner/td121-completion-release-red52/source`
под `dedicated-20cpu`, jobs20, test_threads1 и 12 GiB Cargo guard. Test
`td121_successful_completion_release_settles_its_append_and_boundary_effect`
завершился `rc=101` именно на post-effect completeness assertion; log
`57-causal-red.log`, SHA-256
`27e6e1c704f507a78a02de65de87daaf7f0deee8d99e01ff8ddee63a3e7da32b`.
Compile-invalid attempts до этого receipt не являются causal evidence.

Исправление ограничено settlement фактически успешного callback effect:
наблюдённый точный append и граница должны обновлять scope независимо от того,
пришёл effect на press или release. Имя клавиши, suffix и test fixture не могут
быть runtime-условием. Неэффектный, unhandled или отвергнутый completion не
повышает completeness; удаление границы Backspace не возвращает право на
предыдущее слово; whole-word feedback для исходного `UnknownStart` не создаётся.
Runtime authority пока не менялся. До GREEN остаются отдельные negative controls,
A->B->A invalidation нового receipt, original GTK3 и общие release gates.

Исправление C20 принято в source-focused scope. Settlement теперь сначала
проверяет фактический handled exact appended span, независимо от press/release:
непустой span увеличивает только наблюдённый suffix, а последняя фактически
добавленная граница закрывает прежнее слово и открывает `KnownStart` следующего.
Неизменный хвост, error, unhandled/rejected и atomic no-effect не создают effect.
Односимвольный bounded fallback для обычного printable press сохранён отдельно.
В atomic speculation более новый surrounding callback теперь сохраняет вместе
со snapshot/revision флаг callback provenance и очищает унаследованный receipt.

Remote focused group65: семь test functions PASS. Он покрывает causal Alt-release,
три no-effect controls, boundary Backspace, press/release multi-char append без
границы, Tab press с отсутствием predecessor whole-word feedback, delayed
AtomicV1 merge и captured receipt A->B->A invalidation. Полный IME target затем
прошёл **515/515 PASS** (`66-ime-full.log`); scoped clippy завершился `rc=0`.
Это source proof: установленный runtime, GTK3 и human physical authority не
изменились.

Canonical strict Clippy после C20 выявил новый `large_enum_variant` у
`ActivationOutcome`: `Transfer` занимал 336 bytes против 128 bytes у
`SourceFree` (разница 208 bytes). Поэтому scoped `rc=0` не является строгим
lint acceptance. Исправление хранения ограничено optional
`TransferGrant.exact_manual_snapshot`: reducer продолжает хранить receipt
unboxed, существующий consume-filter сначала проверяет token, owner,
activation, lineage, tail epoch и expiry, и только прошедший receipt получает
`Box`. Установка transfer сразу извлекает его обратно в прежний unboxed
target-receipt. Цена — одна bounded allocation и последующий clone только для
редкого manual handoff с действительным exact snapshot; обычные transfer,
source-free и reset пути allocation не получают. Boxing всего `TransferGrant`
отклонён, потому что добавил бы allocation каждому handoff независимо от
наличия manual receipt. Predicate, reducer state, authority, deadline,
invalidation и proof semantics не меняются. Remote canonical storage proof выполнен под `dedicated-20cpu`, jobs20,
`test_threads=1` и 12 GiB Cargo guard в
`/home/e/projects/lay-development-runner/td121-storage-strict69`: rustfmt
apply/check `rc=0`; strict all-target Clippy с `-D warnings -A dead-code`
`rc=0` и сохранённым JSONL diagnostics; полный IME target **515/515 PASS**,
ignored 0; AST-only `graphify update .` `rc=0`; target остался в budget. Primary
logs и SHA-256 закреплены в `RECEIPTS-72.sha256`. Это принимает узкую
storage-правку и текущие source mechanisms. Старый C18 client receipt не
является acceptance нового source; GTK и physical keyboard остаются `PENDING`.

Отдельный canonical `scripts/update-architecture-graph.sh` после diagnostic
build завершился `rc=1`: рассчитан `WATCH`, потому что `edit-plan-verifier`
сообщил три `mutation_sink_without_capability` для существующих
`apply_text_replacement_pipeline`, `call_replace_text` и
`try_ime_replace_tail`. Точный log:
`/home/e/projects/lay-development-runner/td121-storage-strict69/73-architecture-canonical.log`.
Это не дефект storage predicate и не меняет принятые source mechanisms, но
canonical architecture gate остаётся `FAILED`; старый PASS receipt не
перезаписан и финальный release gate не допускается до отдельного разрешения
этого graph/contract расхождения. Build74 сохраняется только как diagnostic GTK
candidate, не как final release binary.

До этого authoritative receipt были ошибочно запущены три предварительные
локальные команды 05:31:45, 05:32:42 и 05:33:20 UTC в
`/home/ubu/projects/lay-cleanup-20260908`: один compile с exact filter выполнил
0 tests, затем два focused RED запуска (последний через `tail`, скрывший shell
status). Они использовали workstation Cargo guard и локальный disposable
`target/`; это **NONCANONICAL** evidence и не remote C19 proof. Файлы и cache не
удалялись. Все последующие fmt/test/graph действия выполняются только на
`e@192.168.3.94` через resource и Cargo guards.

После admission — focused tests на выделенном remote20CPU хосте, только
`scripts/cargo-guard.sh --bin/--lib ...`, затем changed gate и
`scripts/update-architecture-graph.sh`. Никакой локальной Cargo-сборки.
Существующий `scripts/runtime_smoke` управляет рабочими источниками/службами;
не запускать его в качестве «read-only isolated diagnostic».

Fresh-context review Astra/XHigh: ≥8/10, High/Medium=0, максимум 1–2 repair passes.
Release/install отдельно: global IBus PID сохраняется; cached capability
не считается обновлённой по одному version/hash бинарника. Реальная клавиатура
и видимый клиент проверяются до DONE. Статус, commit и push только после всех
обязательных доказательств; текущий документ не является исправлением.

## Отложено во второй этап

Единый экземпляр IME для обеих раскладок, перенос всего input context state
в новый owner, новые engine names ради обхода кэша, системный IBus restart,
универсальная очередь физических клавиш. Для каждого нужно отдельное
обсуждение пользы и рисков; не включать в Sol-задачу молча.

## Ревью спецификации

[Pass 1](evidence/td120-121-spec-review-pass1.md): документ **8/10**;
`ANALYSIS_REQUIRED` подтверждён как корректная граница. В
[pass 2](evidence/td120-121-spec-review-pass2.md) проверена согласованность
с repaired TD-120; оценка TD-121 сохранена, новый полный аудит не заявляется.
Оставшиеся High/Medium замечания к рассмотренным описаниям — 0/0.
На момент этих исторических reviews двухобъектное воспроизведение и canonical
receipt ещё не были доказаны. Поздние baseline/transport/helper результаты
записаны выше отдельно; они не переписывают первоначальный private 0/6 и не
объявляют непроверенное production wiring готовым.

### GTK storage72: text-free Reset advances exact replay epoch

Однократный original GTK3 diagnostic на exact candidate bytes завершился
**0/3**. Source capture не потерял первый символ: authoritative tails были
6/6/9 chars, оба daemon lease checks прошли и target suppression был accepted.
В fast и extra первые два Backspace press прошли native, затем owned soft
`Reset` без текстового эффекта опубликовал неизменный tail; третий press был
`exact_replay_rejected`. В autocomplete тот же Reset пришёл после первого
native Backspace, второй press был rejected. Release rejected press был consumed,
остальные delete/insert events прошли, поэтому client сохранил ровно один
ведущий старый символ: `gпривет`, `gпривет`, `пghjdthrf`. Extra second pair
затем отдельно потерял точный хвост на `passive:unknown-context`.

Raw receipt:
`/home/ubu/.cache/lay/development/release-1.0.72-td121-prepared/td121-storage72-gui-diagnostic-20260913T064500Z/RECEIPT.json`,
SHA-256 `dfbca1d42c8edec6aa89fb6ba198d830d3c7e5c7ba65cbe5cc335808446b41a9`.
Cross-case causal table: `CROSS-CASE-CAUSAL-TABLE.md`, SHA-256
`a7b297fc02287d0a61418921fe27ceafc85e58f9b2c7bcb25b08f75ec2619a9f`.
Candidate cleanup и desktop restoration PASS; это failure receipt, не GTK
acceptance.

Первый общий механизм находится в `reset_for_ibus_soft_reset`: если transient
Reset не имеет `context_reset_rereceipt` и старый exact handoff уже consumed,
текущий код вызывает `publish_tail_handoff()` даже при действующем progressive
exact replay и неизменном тексте. Epoch увеличивается дополнительно к native
Backspace transition; следующий `process_exact_replay_press` правильно
отказывает из-за несовпадения expected tail/distance.

Варианты следующего production-механизма:

| Вариант | Следствие |
|---|---|
| Не публиковать text-free soft Reset, пока `exact_replay_quarantine_active()` доказывает совпавшие local/shared scope, owner lease, path, layout, tail/epoch и deadline | Узкий предпочтительный вариант: сохраняет progressive replay epoch; не даёт нового owner, таймер или право и оставляет все mismatch/expiry отказами |
| Сделать `publish_tail_handoff()` идемпотентным для любого неизменного tail | Слишком широкий контракт: меняет epoch semantics всех callers и может скрыть реальную lifecycle публикацию |
| Разрешить следующему Backspace принять лишний epoch либо повторить delete | Ослабляет fail-closed contour и способен удалить чужой символ; запрещено |

Сгруппированный test-only RED `td121_text_free_soft_reset_during_exact_replay_preserves_progress_epoch`
провёл настоящий adapter `Reset` в наблюдавшемся GTK-порядке `press -> Reset ->
fresh surrounding receipt -> release`: два Reset для plain `UnknownStart`
`ghbdtn` и один для accepted boundary tail `привет `. До первого key plain
scope имел `observed_suffix_chars=6` и совпадал с live token; boundary fixture
сохранил реально инициализированный ведущий Space. Обе строки дошли до одной
causal assertion без IME `CommitText`/`DeleteSurroundingText`.

Измерено: plain reset epochs `[(9,9),(10,11)]`, native delete presses `2/3`;
boundary `[(11,12)]`, native delete presses `1/2`. В обоих случаях итоговый
tail совпал с client sink, но каждый незащищённый text-free Reset потребил один
progress epoch и следующий press. Focused proof закономерно RED (`rc=101`, один
выбранный test). Receipt:
`/home/e/projects/lay-development-runner/td121-soft-reset-red75/80-causal-red.log`,
SHA-256 `3163cedca4d4d78243f538c270bee06a328aa261edaf8e18d7aa8436a62b2a00`.
Не проверялись production fix, queued extra toggle после полного replay и GTK
acceptance. Verdict ограничен первым общим механизмом; runtime authority не
изменена.

### Soft Reset guard consequence

После принятого RED реализован только узкий guard: состояние
`exact_replay_quarantine_active()` вычисляется на входе в
`reset_for_ibus_soft_reset`, до очистки composition, selection snapshot и
других transient полей, и добавляется отрицательным условием только к финальному
`publish_tail_handoff()`. Это сохраняет уже доказанный progressive replay, не
создаёт route/owner/timer, не продлевает deadline и не меняет ranking,
candidate/package/learning либо verifier authority. Проверка на входе существенна:
её вызов после очистки мог бы ошибочно стереть противоречащие selection или
active-composition evidence. Completed-after-expiry ветвь сохраняет прежнюю
семантику predicate; expired in-progress scope остаётся fail-closed.

Predicate использует существующие bounded scope clones/checks и только
последовательные короткие shared-state reads на редком soft-reset пути; новой
синхронизации и per-key работы нет, latency не измерена.
Grouped negative controls через настоящий adapter Reset покрывают stale owner,
stale path, selection, active composition и expired in-progress: tail не
изменяется, IME text effects отсутствуют, invalid scope не становится
qualified; stale path также не меняет shared owner state. Итоговый focused
TD-121 suite после обеих поправок: **45/45 PASS**, receipt
`/home/e/projects/lay-development-runner/td121-soft-reset-red75/89-focused-green.log`,
SHA-256 `eef6c3d027dc34ec4a408dee0c4b093695dbb067852f32c6999b502e1b8e9cd7`.
Rollback — удалить entry capture и
одно финальное условие guard; runtime promotion пока не выполнена.

### Completed replay queued-tail boundary

Отдельный test-only proof провёл полный `UnknownStart` replay `ghbdtn -> привет`,
затем реальный `VisibleTailV3` bridge с bounded marker и следующий реальный
`ManualToggleV3`. Результат различает два механизма: `VisibleTailV3` вернул
`passive:unknown-context` с пустым tail/focus, тогда как следующий manual toggle
самостоятельно прошёл `Ok((3,false))`. Focused RED: `rc=101`, один выбранный
test; receipt
`/home/e/projects/lay-development-runner/td121-soft-reset-red75/85-queued-tail-red.log`,
SHA-256 `dded5a2e4abbd5b2fd0c15469e16476c635ada2778c684708f7cfb521662d4f9`.

Рассмотрены три варианта. Повторное использование уже существующего
`context_observed_suffix_exact_manual_handoff_allowed()` только в read-only
guard `VisibleTailV3` выбрано как минимальное: predicate требует fresh exact
external suffix, полный observed-suffix count, live token/word lineage и
отсутствие selection, composition, sensitive context, atomic action и seal.
Тот же predicate уже допускает непосредственно следующий `ManualToggleV3`.
Отдельный completed-replay reader дублировал бы completion state и эти проверки;
rearm старого handoff либо promotion в `KnownStart` неправомерно добавили бы
mutation authority.

Изменение добавляет predicate рядом с существующими known/live-handoff
условиями только для readout. Нижняя проверка identity-bound handoff
expiry/snapshot остаётся без изменений. Mutation routes, lease validation,
Suppress gate, learning, ranking и deadlines не меняются. Negative controls
должны сохранить `passive:unknown-context` для incomplete suffix,
mismatched/stale snapshot, selection и stale context без text effects,
suppression arming или learning. Итоговый полный GREEN записан ниже; runtime
promotion не выполнена. Rollback — удалить одну local predicate binding и одну дизъюнкцию
из `VisibleTailV3`.

### Final pre-build source gates

На remote source после обеих mechanism fixes один записанный resource envelope
`dedicated-20cpu`, Cargo jobs 20, test threads 1, target budget 12 GiB завершил:
rustfmt check `rc=0`; strict all-target Clippy `-D warnings -A dead-code`
`rc=0`; полный IME target **519/519 PASS**; canonical architecture `rc=0`;
target budget status `rc=0`. Envelope receipt
`RUNNER-90-envelope.log` SHA-256
`278d0d8df136d8dbc4c9d08c1af81d123bbcbf1f2e030c76211a3f6644a83572`;
full IME log SHA-256
`1a7803ec34450da7bf20d3e836259d517a11fc4516f0e9593dd6710b62d2099a`;
strict JSONL SHA-256
`29598f826518066cade132aa6455d1c0071abd6b4a478618d64175017477e9ca`.
Clippy JSON содержит 0 compiler-message diagnostics. GTK и final ten-binary
release pipeline ещё не запускались; эти PASS не являются runtime promotion.

### Cold AST graph repair

Неуспешное generated/cache состояние сохранено вне source в
`/home/e/projects/lay-development-runner/td121-soft-reset-red75/81-failed-graph-before-cold`
(`SHA256SUMS` SHA-256
`92b1b05e9f9c85b4cdf3204762a7cb14c58d1fe2c518c1b1af4bbd2d6953e71e`).
После удаления только remote `graphify-out/cache` canonical
`scripts/update-architecture-graph.sh` завершился `rc=0`: 702 Rust bindings,
receipt PASS 11/11, 22,550 nodes и 59,884 links; typed-transition-capability,
observed-outcome-feedback и hot-field-memory содержат portable source IDs.
Лог SHA-256 `34019171f785bfd83d324124515b30fdd4a9c4571a8ccf4e1e75c4611cf3ca0d`;
binding SHA-256 `cf62eab78d71556ec8ee289566fbe7c08a73bd89465338461b17c21a1721159c`.
Runtime authority не менялась. После финальных source edits binding/receipt
нужно сгенерировать заново; disposable AST cache в release packet не входит.

### Firefox repeated replay: missing final surrounding receipt discriminator (2026-09-13)

Проверен trace реального Firefox с двумя быстрыми переключениями: первые
Backspace callbacks exact replay уменьшают локальный tail, но промежуточный
`Reset` вооружает receipt для трёх символов, после чего Firefox сообщает ещё
пятисимвольный DOM snapshot. Существующий exact mismatch guard правильно
отклоняет его. После нулевого snapshot replay дописывает шесть символов и
финальный `Reset` вооружает новый шестисимвольный receipt, однако до
`VisibleTailV3` не приходит ни одного финального `SurroundingText` callback.
Owner 4, activation и все key settlements остаются live; первое место потери —
отсутствие независимо подтверждённого внешнего snapshot на read-only
`VisibleTailV3`, а не admission, replay или suppression.

Test-only causal sequence воспроизводит общий механизм без слова или
Firefox-specific runtime branch: все mirrored Backspace/replacement callbacks
проходят существующий `ExactReplay` scope без искусственных per-key receipts;
coarse Reset получает сначала mismatched пятисимвольный snapshot, затем empty
snapshot, а финальный Reset остаётся без callback. `VisibleTailV3` закономерно
возвращает `passive:unknown-context`. Даже добавленный контрольный fresh exact
receipt подтверждает только существующий one-shot reset-rereceipt: следующий
`ManualToggleV3` разрешён, но readout остаётся passive, потому что
`VisibleTailV3` этот authority lane не читает.

Существующие completion/postcondition APIs не закрывают потерю: native replay
не является IME commit/delete dispatch и не вооружает
`pending_visible_postcondition`; mirrored expected tail из `ExactReplay` scope
доказывает правильность локального replay, но не независимо наблюдённый GUI
postcondition. Принимать его как внешний snapshot или повышать clipped empty
`UnknownStart` до `KnownStart` было бы false accept. Безопасный production
вариант требует отдельного доказательства доставки либо read-only использования
уже confirmed reset-rereceipt; ни один из вариантов этим test-only изменением не
принят. Mismatch guard остаётся без изменений. Не проверены причина отсутствия
callback в Firefox и DOM/IBus ordering вне наблюдённого trace. Runtime authority
changed: **false**.

Remote `dedicated-20cpu` focused verification: rustfmt check PASS; causal test
**1/1 PASS**; target budget 775,065,600 / 12,884,901,888 bytes. Receipt:
`/home/e/projects/lay-development-runner/td121-post-replay-discriminator/focused.log`,
SHA-256 `0bf56ca9887337fa2fbf9800b9829459b1e0fe6b65d53a91c0fef051d1789bad`.

### Confirmed reset-rereceipt readout consequence (2026-09-13)

Scope: repair only the inconsistency after a fresh exact post-Reset receipt.
`ManualToggleV3` already accepts
`context_reset_rereceipt_exact_manual_handoff_allowed()`, whose conjunctive
predicate requires a confirmed one-shot receipt, exact current snapshot,
unchanged tail/epoch, live current owner/token, revoked matching predecessor,
`UnknownStart` suffix advance, no selection, composition, sensitive content,
atomic action, whitespace boundary or seal. `VisibleTailV3` omits this same
positive predicate even though it performs no text mutation.

Selected design: reuse that predicate only in the read-only admission
disjunction beside known, bounded-handoff and observed-exact-suffix authority.
Readout does not consume the rereceipt; the following `ManualToggleV3` must still
settle it through the existing route. No snapshot is synthesized from replay
expectation, no `UnknownStart` promotion occurs, and mismatch, owner, token,
selection and safety checks remain unchanged. The exact-replay scope continues
to prove local mirroring only.

Rejected alternatives: treating completed replay bytes as an independently
observed postcondition would false-accept a clipped or externally divergent GUI;
arming the existing commit/delete postcondition API for unhandled physical keys
would change its effect ownership contract; polling, timers and a new delivery
route exceed this consequence. A missing final `SurroundingText` receipt remains
passive and is a separate delivery problem.

Required proof: the coarse Firefox sequence must remain passive immediately
after replay and before the final Reset, and after the final Reset while its
receipt is unconfirmed. One fresh exact receipt must change only readout to the
exact committed tail and leave the rereceipt available for the next manual
toggle. Existing incomplete, mismatched snapshot, selection, stale-context and
no-receipt controls must remain passive with zero text, suppression or learning
effects. The live trace order alone does not prove Reset 108 preceded the queued
timeout; no such causal claim is made. Runtime authority change is proposed only
for this confirmed read-only lane and remains unaccepted until RED/GREEN and the
full focused IME denominator pass.

Baseline RED used the exact coarse-order test with only the confirmed-control
expectation changed. It failed at the intended authoritative-readout assertion:
`rc=101`, with the pre-final-Reset/no-receipt and post-final-Reset/unconfirmed
negative stages already passing. Receipt:
`/home/e/projects/lay-development-runner/td121-confirmed-rereceipt-readout/baseline-red-v2.log`,
SHA-256 `fd6bfb752c9752275df536597ef687e37316757fac1fbcee76abcf32b33146c0`.

The production change is one read-only admission binding/disjunction in
`VisibleTailV3`; it calls the existing predicate without consuming its token.
Remote `scripts/dev-check.py check --target bin:lay-ibus-engine` on shared target
`/home/e/projects/lay-td119-gate-v1/target` passed: discovered 524, selected and
executed **521/521**, failed 0, with 3 performance tests excluded and 0 ignored.
Local result:
`/home/ubu/.cache/lay/development/run-cwx9243y/RESULT.json`, SHA-256
`beccce8022fcb1f89c00711dcfbbc9c39ea17112f6b6d587071abe03d3ff301e`;
remote summary SHA-256
`40674c66f8933c18de50d4aa8b6694e9a3446065f2858e9871ac90a1036058f3`.
This candidate changes read-only runtime authority only for an already-confirmed
reset rereceipt; it is not built into a client, installed or promoted.

Full canonical architecture refresh/check on the original remote workspace,
under the dedicated-20cpu guard and shared target lease, passed. Receipt:
`/home/e/projects/lay-development-runner/td121-confirmed-rereceipt-readout/full-architecture.log`,
SHA-256 `b4a07bf0fbc6ac6b2630871b7da8e43777657e4d11d99d057fbbcc58f42770b7`.

### Queued actual Shift-pair delivery consequence (2026-09-13)

The original failed fast-repeat trace and the pinned IBus source expose a
second delivery boundary. While exact native replay owns the physical keyboard
grab, `forward_queued_input` reads the user's later Shift events. Both left and
right Shift branches update local state and `continue`; a completed left pair
invokes the queued replay callback without forwarding any of its four physical
transitions. That callback first waits on `VisibleTailV3`. Therefore Firefox
does not receive the already-captured next key press which would naturally run
IBus `_request_surrounding_text` before filtering the key.

Three designs were compared:

1. **Forward the captured completed pair once before its callback.** Emit the
   exact `left press, left release, left press, left release` sequence through
   the existing `lay-virtual-keyboard`, then invoke the existing callback. The
   virtual device is excluded by name from daemon keyboard discovery, and the
   legacy IBus engine treats Shift as observe-only while the daemon owns the
   gesture. This restores actual user input; it does not add a probe key, text
   mutation, authority owner, timer, queue or fallback. The existing bounded
   settlement loop may observe the resulting client receipt, but emission alone
   does not prove compositor/Firefox processing order.
2. **Firefox refresh after selection notification.** Reuse Firefox's existing
   `retrievedSurroundingSignalReceived` witness to call
   `OnRetrieveSurroundingNative` after its content selection cache advances.
   This is the broader client correction and may also address ordinary initial
   stale snapshots, but it requires an upstream Firefox change and a browser
   build outside this task.
3. **IME-owned printable commit.** Handle replay keys or replace native replay
   with IME commit/delete output. This changes DOM keyboard-event, composition,
   undo and effect-ownership contracts and risks a second mutation route. It is
   rejected here.

The selected experiment is design 1, subject to a strict bijection. Only one
legitimate completed physical left-Shift pair may produce four virtual
transitions and one callback, in that order. Emit failure produces no callback.
Partial taps, repeat events, an intervening non-Shift press, mixed left/right
Shift and right-Shift-only sequences produce neither a completed-pair emission
nor callback. A following typing key or boundary remains after the pair and
callback in FIFO order. Existing daemon virtual-device exclusion must remain
true. If the current counter cannot express interruption and side identity, it
must be replaced by one exact FSM rather than supplemented by a second detector.

This design does not address the separate native MOZ_LOG run where replay never
started: five selection records with explicit offsets 1, 3, 4, 5 and 6 had
`retrieved=true`, followed by Reset; another selection log line had no explicit
offset. Initial stale-snapshot witness loss and queued post-replay receipt loss
remain separate claims. No production behavior, Firefox source, client binary
or installed runtime is changed by the discriminator.

The effect budget is the four already captured Shift transitions and one
existing callback. The selected route adds no text edit, layout switch,
authority owner, learner/package write, cache entry or retained identity; it
uses the current virtual keyboard and queued callback serially on the existing
reader thread. It allocates no per-pair collection and adds four fixed uinput
events, so CPU and RSS impact should remain below measurement resolution; the
material latency is one four-event emit before the callback and remains
unmeasured until GREEN. The virtual-device name exclusion is the self-observation
barrier and must stay independently asserted.

Error cleanup is part of the contract rather than a best-effort add-on. A
failure after a synthetic Shift press must attempt the matching release before
returning, while still suppressing the callback. Cleanup must not emit another
press, complete another toggle, or feed the pair back into the physical queue.
The exact first error remains the returned/logged failure even if release cleanup
also fails. Concurrent queued input remains serialized behind that bounded emit;
typing and boundary events must then be processed once in original FIFO order.
No token, window/field owner, tail epoch, model generation, material package,
completion cache or learning state is created or transferred by the delivery.
Rollback removes the one dispatch emission and exact FSM together, restoring
the swallowed-pair baseline; no persisted state or schema migration exists.

The controlled baseline discriminator introduced a semantic-preserving dispatch
seam: production still ignores its emitter and invokes the existing callback.
The remote `dedicated-20cpu` focused run discovered **266** daemon tests and
executed **264** correctness tests: **260 passed, 4 failed, 2 performance tests
were filtered**. Three failures expose missing delivery: the observed exact
pair was only `[Callback]`; injected emit failure returned `Ok(())` and still
called back; FIFO lacked the four leading Shift transitions. The fourth failure
came from an overstrict repeat expectation, corrected below; it is not a
runtime defect.
The existing virtual-device exclusion test passed inside the 260. The manifest
reports four added tests and no changed or removed test.

This RED does not yet prove a GREEN implementation. The negative discriminator
currently filters non-left-Shift inputs around the old counter, and the FIFO
test appends typing/boundary observations after the dispatch seam. GREEN must
route the complete event stream through one production FSM and the same
production event processor, assert real `WordBuffer` effects, and independently
cover failure after press with release cleanup. It must not repair only the test
helper. Local result:
`/home/ubu/.cache/lay/development/run-qm24tq96/RESULT.json`, SHA-256
`a081ebd4ad11ec0a2a6d6d2f69fe1c83d3d15849386aea47660ae2c414a40df7`;
remote log SHA-256
`452aeed135d73ffb54c8d4b98b5780140d8d6b0df8c208835c27f4e690c30c30`;
remote summary SHA-256
`1c9e3101887c8e6a6bcba34f0b40b16bb41161fad9fcfc802013ae0d69fb4b3c`.
Two earlier runs stopped before tests at formatting and borrow-checking and have
zero denominators; they are invalid predecessors, not delivery evidence.
Runtime authority changed: **false**. No release build, install, service, GUI,
Firefox patch, architecture refresh or scored review ran.

### Queued-pair GREEN consequence correction before production verification

Source review found two additional constraints. The queued replay callback had
captured the layout from the first replay invocation, so every later completed
pair could settle against stale layout state. The callback must receive the
current processor layout on every invocation, and alternating complete pairs
must each produce exactly one toggle while preserving following `WordBuffer`
input in FIFO order.

The fourth historical RED was intentionally overstrict about kernel repeat
events: the main `DShiftState` ignores `value=2`, so
`press, repeat, release, press, release` is two physical taps and must produce
one toggle. A single press with any number of repeats and one release remains
one tap and produces none. This corrects the proof expectation; it is not a
production defect.

The immediate raw-FIFO comparison is rejected because a grab ending after a
captured press can leave the virtual device held while the later release comes
from a different evdev device. The chosen bounded design emits only a fully
captured balanced four-transition pair before its callback. It passes the same
main `DShiftState` through queue drain and preserves any successor state across
common completion instead of creating a detector, timer, wait or held-modifier
owner. A partial pair emits no virtual press and therefore cannot stick one; its
FSM state continues when later physical events return to the ordinary daemon
loop. The baseline limitation that a partial Shift modifier captured during the
grab is not replayed to the client remains unchanged and is outside the reported
repeated-toggle repair.

GREEN must cover all three internal pair split points across drain/completion,
other-key cancellation, ignored repeat events, zero virtual output for partial
input, no stuck virtual modifier, at least two queued complete pairs with current
alternating layout, emit failure with no semantic callback, and following real
`WordBuffer` input in FIFO order. Generic success completion may reset its
predecessor state, but the exact queued route must restore the successor state
harvested during its drain. Reject routes and non-queued manual-trigger callers
retain their existing reset semantics. Build, install, service, GUI and runtime
authority remain outside this experiment.

### Queued-pair root verification and graph recovery — 2026-09-13

The final queued source includes a bounded warning for an emission error and
still performs no callback or retry after that error. Root reviewed the shared
FSM handoff, completion flag, current-layout callback argument and balanced
four-event emission. The deterministic tests cover these production components
and emit-before-buffer effects. They do not execute the entire evdev drain or
prove client-visible Firefox delivery. Partial modifier delivery remains the
unchanged limitation stated above; initial Reset mismatch retention is a
separate unresolved mechanism.

The combined guarded daemon/IME check passed **786/786** selected tests from
791 discovered, with five performance tests excluded and no ignored tests.
Selected and executed identities match exactly. All 705 source/Cargo rows match
the tested archive before this documentation update. Local result:
`/home/ubu/.cache/lay/development/run-g8pk7wpw/RESULT.json`, SHA-256
`28cb564d7a3cdcd74a11dcea7ea8d7d82d37efb6603ca43a7a598940ba3e9d39`;
archive SHA-256
`e2e474c2d475dca75cf039abd57f378d77f799c411e0719b26638440033c9c25`.
Remote summary:
`/home/e/projects/lay-development-runner/run-yiW6cU/tests/SUMMARY.json`, SHA-256
`598b1052e3cf079285da52b09e701d1084936eb3a5837a52d170b19b6c1618b9`.
The run took 154.03 seconds; this is development verification, not a release
gate, human keyboard acceptance, measured latency or Wave-quality proof.

An earlier IME run remains **520/521 PASS, one FAIL**:
`/home/ubu/.cache/lay/development/run-9udxj1nz/RESULT.json`, SHA-256
`f04b331fddf06d5d8ebc97c144d6eec34f81a04b29e1738cd8ca437154754116`.
`td121_successful_completion_release_settles_its_append_and_boundary_effect`
failed its observed-suffix display-frame precondition. An exact rerun and a
fresh canonical run passed without an IME source change. The exact rerun has
session output only; no durable receipt is claimed. The cause of that transient
precondition failure is unproved, and later PASS does not erase it.

The author also ran local Graphify/architecture commands, contrary to the
remote-only execution rule. These are rejected verification attempts. The
resulting graph again contains foreign checkout IDs and lacks the canonical
AuthorizedEdit node; the three capability checks correctly return WATCH. Root
has taken execution ownership. The recovery uses the current frozen source in
the original remote development workspace, whose AST cache is absent, then the
unchanged full architecture wrapper under the dedicated-20cpu resource guard.
No sink exception, gate change or authority promotion is permitted. The remote
refresh completed with both architecture verdicts PASS. Its local log is
`/home/ubu/.cache/lay/development/td121-queued-pair-root-20260913/architecture-final.log`,
SHA-256 `278c611140aa05967bfd268d97024c4ead4af2a8145801518cf3651114993ebb`.
The guard recorded the required 20 jobs, one test thread, CPU 2000%, memory
24/28 GiB, swap 1 GiB and 512 tasks. All 1,054 AST inputs were extracted without
a transferred cache. The result has 22,615 nodes, 60,050 links and zero foreign
checkout-prefixed IDs; all three required AuthorizedEdit sink edges are present.
Graph SHA-256 is
`49abfdb426371bdd751c5dfff98bbc33f34b05c9089a70a708dd430987af55de`;
source binding SHA-256 is
`79b853afe5726256264f92dc4a82a2cdd409f7336aac2d5215762161019b6bc3`.
The same unchanged wrapper runs after recording this result; its companion log
is `architecture-recorded-final.log` in the same directory. No installed
runtime, browser source or runtime authority changed.

### Initial delayed Reset snapshot: causal preflight — 2026-09-13

The first unresolved loss is the initial Reset in the Firefox native-log
receipt recorded in `evidence/td121-firefox-client-baseline-2026-09-13.md`.
The observed two-character tail receives a one-character snapshot, loses its
pending predecessor, then cannot recover after an accepted append and another
same-owner Reset. The metadata does not establish the shorter snapshot's exact
text. The next test therefore distinguishes an actual strict prefix from a
contradictory snapshot using the production legacy adapter, Reset and
SetSurroundingText callbacks. No production edit precedes its causal RED.

Compare three routes: the unchanged baseline permanently drops the lineage;
bounded retention in the existing PendingContextResetRereceipt preserves only
an unconfirmed strict-prefix candidate; changing Firefox's selection refresh
would alter the client and require a separately maintained browser build.
The second route is the proposed experiment. Keeping every mismatch or granting
authority from a partial snapshot is rejected. A shorter nonempty snapshot must
exactly bound a prefix of the observed token, with no selection or following
word continuation. It can retain no mutation authority. A second receipt in
the same armed revision remains a refusal; only a verified one-character append
or an authenticated same-owner Reset can establish a new receipt opportunity.

The candidate may cross an append only with the existing exact tail extension,
single epoch increment, admitted callback and current token/scope checks; its
unconfirmed status must be preserved. Another Reset must replace the token and
revalidate the same owner before arming. A fresh full-token snapshot must still
satisfy the existing exact boundaries, current owner/token, scope and one-shot
consumption guards. Selection, wrong text, empty/clipped data, navigation,
non-append input, focus, capability and content changes revoke the candidate.
The test also checks passive VisibleTail, refused ManualToggle and zero GUI
edits before confirmation, then typed delegation only after confirmation.

This changes candidate retention, not lexical ranking: ordinary Double Shift
has no L1/model route. No SafetyGate, edit plan, verifier, daemon mutation,
completion cache, package generation, reload, learning or feedback behavior is
changed. False authority is the material risk; exact full client evidence and
all identity checks remain conjunctive. The record already exists and is
bounded by the retained tail; prefix comparison adds linear work on that
bounded text, with no new allocation collection, timer, RPC, retry or wait.
CPU/RSS and native latency remain unmeasured. Engine callbacks serialize the
record, while rejected/stale admission and lifecycle changes must clear it;
no new owner, persistent state or source of truth is introduced. Future package
or learner updates do not alter these transport predicates. Terminal and atomic
routes retain their current behavior and require the affected IME test set.

The first proof denominator is four positive schedules: initial lengths two
and five, each with an exact or strict-prefix first receipt, followed by an
append, another Reset and a full receipt. Existing and new adverse controls
must preserve refusal. A baseline failure must occur at retention/recovery,
not at setup, timing, compilation or an unrelated display precondition.
Production implementation is conditional on that result. Rollback removes only
the prefix-retention and unconfirmed-continuation changes together; it restores
the destructive-mismatch baseline without a schema or runtime migration.

The baseline causal RED is now established. The focused run discovered 526
tests, selected/executed 523 identical identities, and passed 522 with one
failure and three performance exclusions. The exact initial receipt passed;
the two-character delayed case reached the intended final assertion with
`(retained_after_first, retained_after_append, retained_after_reset, recovered)
= (false, false, false, false)`. The five-character schedule was not reached
after that assertion. Local result:
`/home/ubu/.cache/lay/development/run-yliu5_5p/RESULT.json`, SHA-256
`96eb21a9e0df195ad1b80e561e591d5a804d97c17170d34bb77228ba491941a8`;
archive SHA-256
`93d001973fef2f89c210c36f7a61c0885890db7150333b790873b80eefbddc99`.
Remote summary `run-xJ9tia/tests/SUMMARY.json` under the development runner,
SHA-256 `de5f767604d32906158a792271addb7163c2e4278512ec3516a7ec8056186522`.
The earlier `run-ke72nq87` failure was a fixture error: it expected a Russian
layout from the explicitly US engine. Correcting that expectation alone
produced this causal RED; that earlier result is not product-defect evidence.

The bounded retention route is selected for production verification. Prefix
comparison uses scalar cursor positions and borrowed slices, not a copied
snapshot or a new collection. It requires current token/owner and exact retained
tail identity even to retain the unconfirmed candidate. Append continuation
also preserves the existing word generation and owner. Adverse text controls
are strengthened to deliver the contradictory snapshot as the first receipt,
so they cannot pass merely because the second-receipt guard fires. No authority,
installation or native-client success follows from this RED.

The corrected production source passes **523/523** selected tests from 526
discovered, with three performance exclusions and exact selected/executed
identity equality. All four positive schedules now reach confirmed typed
delegation; all 12 adverse schedules refuse. Before confirmation, the tested
readout is passive and manual requests emit no GUI edit. All 706 runtime/Cargo
rows match the tested archive. Local result:
`/home/ubu/.cache/lay/development/run-ap8vhmet/RESULT.json`, SHA-256
`b1a7c6b71dade4914c80bb2d8a6d4a177d210fa9209e1dbd7a18e0e27a010242`;
archive SHA-256
`bcf1c25375b76556a17e7bf3f6c8f1b4961f104ef031693840c249197b1de6eb`.
Remote `run-UQPv5j/tests/SUMMARY.json` SHA-256:
`f10bab0fef4857f47f6cfa5cbd1cf123de88568a4f9a59febe0b49355bcec57e`.
Development verification took 16.70 seconds; native latency is unmeasured.

The first production verification (`run-0v3cf9bq`, 522/523) exposed a fixture
transport error after registering then detaching the bridge engine: zbus sends
UnknownObject for the subsequently received callback. The fixture now consumes
only the exact serial/UnknownObject reply with the existing helper before
driving that callback and asserting CommitText. It uses controlled receive order,
not a sleep or generic error filter. Production code was unchanged for the final
PASS. Root reviewed candidate retention, owner/lineage checks and the unchanged
full receipt/consumption predicates; this is an unscored review within the
previously replanned TD-121 repair, not a third formal review pass.

Next, refresh the architecture remotely with the unchanged full wrapper and
build an inactive IME/daemon candidate from the exact bound source. The fixed
owned Firefox Fast/Extra/autocomplete scenarios will use the existing C20 input
binary and both new candidate executables. Record those client denominators
before release/install work. GTK, terminal, release and human keyboard gates
remain separate. Installed C20 and runtime authority are unchanged here.

### Frozen Firefox recovery candidate: native result — 2026-09-13, 12:54 UTC

The remote full architecture wrapper passed twice within its normal invocation;
the graph has 22,622 nodes, 60,090 links, no foreign checkout IDs and the required
AuthorizedEdit edges. Both executables were built under the required guard;
target usage ended at 10,135,805,952 bytes below the 12 GiB limit. Exact 706-row
source binding, graph/receipt/log hashes and artifact identities are in
`/home/ubu/.cache/lay/development/td121-firefox-recovery-candidate-20260913/artifact-binding.json`,
SHA-256 `22d34b48c68de6e3e3d546fa52a1aad3e0c70022dd52a0a10e89bd5e1c724339`.
IME SHA-256 is
`458befda749cc055e4727cea6845116566b302d25aafaf2a96b513ef6e015300`;
daemon SHA-256 is
`22193b24914cf306d531e2733dcdafcf286480206c1dfb09ae7f8e4048c5edbd`.
The first attempted guarded stdin script executed no build and created no
artifact; the saved worker script corrected that transport mistake.

The unchanged owned Firefox harness returned **1/3 scenarios PASS**: Fast has
one toggle and `привет`; Extra has zero of two toggles and unchanged `ghbdtn`;
autocomplete has no preedit/completion, zero of one toggle and `про`. The Extra
surface alone is not a round-trip PASS. Receipt:
`/home/ubu/.cache/lay/development/td121-firefox-fast-repeat-20260913-t__qoo7d/firefox-recovery/RECEIPT.json`,
SHA-256 `4520b280ff6502901ba3e381db379a5aebb06ad9dbed534f5410fbde781f64ac`.
All three exact IME and supervised daemon identities match the candidate.
Restoration is verified, fatal error is null, C20 hashes are loaded again,
and global IBus PID/start 4715/2261 and user Firefox 3124490/83088446 are unchanged.

Extra's 159-row trace, SHA-256
`d0b22a194745b4293aeb5f009014c5a99bd73a6bcef14eaf03eed7dfc332dde0`,
now supplies the first decisive diagnostic. Request 1 publishes owner 1;
FocusOut(19)/FocusIn(20) starts request 2, nonce 2; marker arm/emission appears
at rows 31/32, followed by `marker_not_observed_before_expiry` at row 33.
There is no owner before the first printable callback at row 34. Both later
bridge markers refuse `lifecycle_pending`. This run never reaches queued
replay, and neither native prefix-retention coverage nor queued delivery is
proved. The autocomplete failure has confirmed Reset receipts but no display
worker/update; its first display-authority loss remains separate.

Before changing deadlines or admission, the next bounded experiment is one
Extra case with an external IBus `dbus-monitor --profile` observer. It records
headers/timestamps only for context-property calls/returns and our private
markers, using the same frozen binaries, input and owned browser fixture.
Correlate only the candidate's bus identity and request serials. This can bound
wire-side Get/reply/marker timing; it cannot prove when the engine observer ran
or uninstrumented latency. It adds no application input, mutation, runtime
diagnostic state, timer, retry or fallback. The monitor itself has bounded
process cleanup and can perturb bus scheduling; a non-reproduced expiry must
remain non-reproduced, with no repeated trial until green.

The external profile capability passed a read-only Get control despite IBus
falling back from the unsupported Monitoring interface to eavesdropping. The
single registered Extra experiment returned **1/2 toggles, FAIL**, with final
`привет`. Receipt `firefox-wire-profile/RECEIPT.json` under the same Firefox
evidence parent has SHA-256
`501ee7c701833326f26fa491ea0c3051b6ef0284e38b6fa142c875af5dc0c217`;
its 393-row IME trace hash is
`02fc55200acb0f18dd65f2a5e05619fb9a9defe4e53692f7d7077f88b614972e`.
The 570-line header-only wire profile is in the recovery candidate directory,
SHA-256 `f2e42c5d295edd9d75f5608b0a38e3c7496bf136f55f8d84260a76d664d93793`.
The expiry did not reproduce. Candidate sender `:1.4857` has context acquisition
Get-to-next-marker intervals of 325, 396, 3818 and 453 microseconds (serials
19, 23, 25 and 62); the corresponding Get/reply intervals are 228, 97, 2732
and 194 microseconds. These wire intervals exclude pre-Get scheduling and
engine-side observer processing; the bootstrap GlobalEngine Get is excluded.

This run independently proves delivery of exactly four queued Shift callbacks
after all six native replay glyphs: trace rows 370, 377, 382 and 388, with the
same admitted owner 4. No subsequent SurroundingText callback arrived before
settlement timed out. Thus balanced queued delivery is implemented and reaches
Firefox, but is insufficient to establish its post-edit receipt. The final
Reset at rows 390–393 only arms an unconfirmed candidate. DOM event timestamps
remain server-receipt times and are not used to infer browser event order.

The next experiment compares the two documented GTK IBus transport modes on
the same frozen binaries: synchronous mode 1 and hybrid mode 2, each with the
same fixed Fast/Extra/autocomplete scenarios. Upstream IBus 1.5.29 GTK3 defaults
to mode 0 when the variable is absent; exact distribution patch parity is not
asserted. Its source describes mode 2 as asynchronous waiting with a GSource
loop and synchronous delivery to the event owner. The user's loaded Firefox
environment has the variable absent. The hypothesis is that the asynchronous
native replay callbacks can outrun the client's text/selection updates, so
the queued Shift retrievals still see old text.

This is a six-scenario transport discriminator, not a fallback, browser patch,
deadline change or environment installation. Both modes are registered before
either result is known. Only the owned browser's allowlisted environment value
changes, and its actual process environment must match that value; the user's
Firefox is preserved. The existing original helper, source/config/material
identities, test input, focus checks, oracle and deadlines remain fixed. Record
client outcomes separately by mode and preserve failures. A positive result
would establish only that configured transport, with keyboard latency and
compatibility costs still unmeasured; it would not explain or waive the distinct
5 ms acquisition expiry. No more runtime patch follows without a discriminating
result and a new bounded consequence analysis.

The registered transport matrix rejects both modes as a repair. Mode 1 passed
Fast only (**1/3**); Extra again executed one of two toggles and timed out on
`passive:unknown-context`, while autocomplete produced `про` without completion.
Mode 1 receipt SHA-256:
`b4b2bc40c225dd616e4e9f1ac9f2a1faa94c12287b828283377299a0191e7895`.
Mode 2 has **0/2 valid scenarios**: Fast made no toggle; Extra executed one and
produced reordered `ирптев` instead of `привет`. Its autocomplete capture is
**INVALID**, because the final focused window was not the owned Firefox page;
that is not a measured product text/toggle outcome. Mode 2 receipt SHA-256:
`42f3460c26072dcde76f3d4850710730ad1ad01f92d867222689a36a717e2ec0`.
The receipts are `firefox-sync-1/RECEIPT.json` and
`firefox-sync-2/RECEIPT.json` in the same Firefox evidence parent. The initial
driver's zero-toggle counter for the invalid capture must not be promoted into
a product denominator. All valid captures verify the requested environment and
the exact IME owner; both complete runs verify desktop restoration. The driver
also independently checks C20 loaded hashes and unchanged user Firefox/global
IBus identities after each mode. No user environment was installed or changed.

The detailed matrix and header-only profiles are in
`td121-firefox-recovery-candidate-20260913/transport-discriminator-results.json`
and `firefox-sync-{1,2}-wire.log` under the development cache. No new production
edit or build was made for these experiments. The next unresolved producer is
Firefox's post-selection surrounding-text refresh; the independent engine
autocomplete gate also rejects a confirmed Reset predecessor because the new
WordScope counts only post-Reset characters. Neither is permission to synthesize
a client receipt, weaken edit validation, add probe keys, delays or retries,
or declare the source-only repairs accepted by Firefox.

### Confirmed Reset suffix display: consequence check — 2026-09-13

The fixed native autocomplete trace has a confirmed three-character Reset
predecessor and no display frame. The first source loss is the count test in
`context_observed_suffix_is_current`: the replacement UnknownStart scope has
zero observed characters, while the existing confirmed receipt proves the
complete retained token against fresh client text. This is separate from the
missing post-replay callback and compatibility-acquisition expiry.

Compare the unchanged refusal baseline, eager promotion of the scope when a
surrounding callback confirms the receipt, and reuse of that exact receipt in
the existing display/explicit-append predicate. The last route is selected for
a causal test: keep the existing token, sensitivity, selection, exact text and
boundary guards, and permit the deficient count only when the full current
Reset predicate passes. Eager promotion would change reducer authority merely
on observation and bypass the existing one-shot consumption boundary; reject
it. Display alone must leave UnknownStart and the reducer count unchanged.

All production Tab/Alt completion callers append a trailing space. Their
existing successful-effect settlement already records that actual boundary;
there is no need for another scope transfer, bridge token, owner or persistent
receipt. Prove the real legacy Tab callback emits only the authorized suffix
plus space, then obtain a fresh client snapshot and exact ManualToggleV3
delegation. A fabricated no-space production route is not part of this change.
A subsequent client Reset after acceptance is not established by this proof.

The material risk is accepting a stale hint or promoting a clipped prefix.
Unconfirmed, mismatching, selected, duplicate, revoked and foreign-owner
receipts must yield no frame and no completion effect even with a retained
surface. Whole-word replacement and learning remain refused before the actual
boundary. Candidate sources, ranking, package/delta identity, reload, feedback,
SafetyGate, plan validation and verifier are unchanged. Reusing the bounded
existing predicate adds no owner, allocation collection, timer, RPC, retry or
wait; CPU/RSS and native latency remain unmeasured. Engine serialization and
the existing token revalidation guard background-frame and stale-result races;
future model updates cannot replace those transport checks. Atomic and
terminal behavior must pass the affected IME set. Rollback removes this count
exception alone, with no migration or removal debt.

The first denominator is two observed prefix lengths followed by exact Reset
confirmation, a fixture suffix, actual Tab output/settlement and fresh exact
manual delegation. The suffix fixture tests transport authority, not model
quality or native worker completion. Nine adverse schedules retain refusal.
Record a baseline failure at the first frame assertion before production code
changes; build/install/native success does not follow from a source test.

The causal baseline reached the intended first frame assertion with confirmed
`abc` and failed there. It passed 524/525 selected/executed identities from 528
discovered, including all nine adverse schedules; three performance tests were
excluded. Local `run-fc7wdoq7/RESULT.json` under the development cache has SHA-256
`8618ae6b1b17c922fd1a3386295024201781df22a025b21ee4e6983ad6efcf4d`;
archive SHA-256
`b088ec6ab3ab3d020921aa0703de456533eb32e51454533ace37a86dd5a37d85`.
Remote `run-MXE79o/tests/SUMMARY.json` SHA-256:
`d881fd56e8d412010d77a8923bd4b3c8fd47e197555825d9b2d682e6960020eb`.
The five-character schedule and Tab effects were not reached after the failure.
Proceed with the selected predicate reuse, preserving every other guard.

The four-line count exception passes **525/525** selected/executed tests from
528 discovered, with exact identity equality, no failures and three performance
exclusions. Both prefix lengths preserve the original zero-count UnknownStart
token during readout, emit only the fixture suffix plus space on actual Tab,
settle the observed boundary, retain zero completion-learning feedback and
return exact typed delegation after a fresh snapshot. All nine adverse
schedules refuse frame and Tab effects. No extra scope-transfer code was needed.
Local `run-h55hqd_m/RESULT.json` SHA-256:
`19918138faad5166a7331c9aee1a800c8e1ce767ff3087b9c15af7d099b050a8`;
archive SHA-256
`cc8a46d510f9209fc156e8e30a11dc7fe48af12b28eca9d9e4e3f460878350cc`.
Remote `run-YACFSE/tests/SUMMARY.json` SHA-256:
`0a7905852b26ed18bfd802fa2353bcec90351d601dd85c9075d3700566429581`.
Focused development verification took 17.03 seconds. Root reviewed the exact
guard reuse; no additional formal scored review is claimed.

Next refresh the full architecture remotely, bind/build the inactive candidate
and run the affected owned Firefox autocomplete and queued Extra scenarios
once each, using the original environment, input and deadlines. This must
measure actual worker/preedit/Tab behavior separately from fixture transport.
The unchanged missing post-replay snapshot and intermittent 5 ms acquisition
expiry remain open; header-only mode profiles did not reproduce the expiry
and do not justify increasing its deadline. Installed C20 remains unchanged.

The full remote architecture wrapper passes with 22,627 nodes/60,111 links;
graph SHA-256 `21bc032caa6fe6cea1f3c3967cfb562289bd5e6c95b1434a3b1a33d5bcb3d77d`.
The frozen source/graph/build binding is
`/home/ubu/.cache/lay/development/td121-firefox-suffix-display-20260913/artifact-binding.json`.
The 705-row build manifest was verified before and after the guarded build.
IME SHA-256 `793fa4abe42c7e510fca22326e4049fad57344f34850b0e5909e605bef1a89ba`;
daemon bytes remain `22193b24914cf306d531e2733dcdafcf286480206c1dfb09ae7f8e4048c5edbd`.
Target ended at 10,135,883,776 bytes, below 12 GiB.

Native result is **0/2 PASS**. Autocomplete ends at `про`, without a worker,
preedit or toggle. Extra ends at `ghbdtn` with zero of two toggles; identical
surface is not a round-trip success. Receipt:
`/home/ubu/.cache/lay/development/td121-firefox-fast-repeat-20260913-t__qoo7d/firefox-suffix-display/RECEIPT.json`,
SHA-256 `0ca7b2dd211d1aa1a450df4bf1d39b03196ed15f9bc6cfc8c63545597beb3ab9`.
Both captures are valid; restoration is verified and fatal error is null.
The driver independently verifies restored C20 loaded hashes and unchanged
global IBus 4715/2261 and user Firefox 3124490/83088446.

Extra's 202-row trace SHA-256 is
`9d6332b8e0a65772bac8064ee6c4e3a8f8e618cfedf8e7ab76c008401e47151d`.
It proves the new readout reaches a real worker: the two-character prefix
receives an applied suffix at rows 78–82. After the third managed character,
the first next snapshot is five characters long with cursor/anchor at three
(rows 92–93); it rejects the advanced receipt. A three-character snapshot then
arrives at row 95 but cannot recover the destroyed predecessor. Residual preedit
is a hypothesis consistent with the displayed suffix and these lengths; the
metadata does not independently record the snapshot's text. The earlier guard's
`second_surrounding_receipt` label must not be mistaken for proof of a duplicate.

Autocomplete's 191-row trace SHA-256 is
`517b0e2f6b11c9764fb05fcafdcc3214193a2fa7833666883484b5cc8f471753`.
The first letter commits at row 81 and settles at row 83. The observer receives
its release 43 and then Reset 44 at rows 84–85, before either handler finishes.
Release settlement then refuses at row 90; the Reset stamp times out at row 91.
All later releases/Resets lack the predecessor. This is an earlier causal order
than the display gate and is not a reproduced acquisition timeout.

Do not proceed with the contemplated GTK producer experiment before this
engine-side ordering is understood. Read-only inspection of the public GTK
[reset](https://docs.gtk.org/gtk3/method.IMContext.reset.html),
[retrieve-surrounding](https://docs.gtk.org/gtk3/signal.IMContext.retrieve-surrounding.html)
and [module loading](https://docs.gtk.org/gtk3/running.html) APIs did not change
the client. No module, preload, environment installation or browser patch exists.

The next test-only discriminator receives a real legacy release and a later
authenticated Reset before executing their handlers, in both callback orders.
The release has no tail effect. Observe candidate retention separately from
current authority; then require the actual Reset callback and fresh exact text
before any typed manual delegation. Compare a later effectful callback and
owner/content revocation as adverse cases. The source hypothesis is that
`revoke()` clears the reducer's unsettled keys at Reset ingress, and settling
the older release treats that expected retirement as an invalid callback,
revoking the successor again and clearing its pending callback stamps. This
requires causal proof and a separate consequence check before production edits.

The controlled baseline confirms that mechanism: 529 discovered, 526 identical
selected/executed identities, 525 pass and one intended failure; three
performance exclusions. After the old zero-effect release, the original
one-character predecessor token is replaced by a zero-count token with three
extra revocations. The actual Reset handler and the other three schedules are
not reached. Local `run-2kwdmbnn/RESULT.json` SHA-256:
`1291bbd972aad87806584004e6b3b0ef062bb2a48e2fbbc55c94d72b85cacc5b`;
archive SHA-256
`bf84b87a55518ebc6c79bf335dd4cd69e336f00960e3e22af256f2905dee4a03`.
Remote `run-25fmWS/tests/SUMMARY.json` SHA-256:
`87e195dcac1f6fa0d8feb60ac79e90286b82057dd685695b6ad8b5d487c71a86`.

### Older zero-effect release: consequence check — 2026-09-13

Compare the unchanged destructive settlement, delaying Reset revocation until
its handler, and classifying a received key by the reducer revocation already
current at its ingress. Delaying revocation would let a bridge authorize old
text after a received lifecycle change and is rejected. Select the third route:
copy the existing reducer revocation value into the existing immutable Key
stamp. This is not a new counter, generation owner or queue. A legacy release
with unchanged tail may complete as already revoked only if its original owner
is still current, its ingress revocation differs from the current reducer, and
the engine tail epoch equals the reducer's settled tail epoch. It must neither
settle a key nor publish a token/scope, advance authority, consume a Reset
receipt, clear later stamps or change the reducer. All other callbacks keep the
existing settlement. Atomic settlement is excluded.

This preserves the engine's non-authoritative predecessor until the actual
Reset handler can capture it, or preserves the new pending Reset receipt when
that handler ran first. Old tokens remain invalid immediately after Reset
ingress; only the real Reset and fresh exact client snapshot restore the tested
suffix authority. A same-revocation missing key, effectful callback, foreign
owner or sensitive content must not gain this exemption. Duplicate and missing
callback stamps remain subject to the existing one-shot rendezvous.

The main risk is misclassifying a late effect as harmless and thereby retaining
a stale suffix. Restricting to legacy releases, byte-identical tail and the
settled epoch bounds that risk; Alt completion with an actual append must still
follow effect settlement. No candidate/lattice source, ranking, model/package
reload, cache identity, learning, feedback, SafetyGate, edit plan or verifier
changes. Future packages do not change key ingress provenance. The additional
integer lives in the existing 128-entry stamp store; its current 224-byte enum
budget remains mandatory. There is one extra bounded reducer check on such
releases, no allocation collection, wait, timer, RPC or retry. CPU/RSS and native
latency are unmeasured. Existing lock ordering is preserved with no nested
stamp/reducer lock or new mutable owner. A stale callback never updates a later
scope; the actual lifecycle/content handler still owns invalidation.

The proof denominator is four positive combinations of prefix length and
callback order, plus explicit effectful, same-revocation missing-key,
foreign-owner and sensitive-content controls and all affected IME contracts.
Rollback removes the copied stamp field and the retirement check together;
there is no wire/config migration or new consumer route. The later intermediate
surrounding mismatch and missing Firefox post-replay callback remain separate
unresolved mechanisms; this change cannot claim their acceptance.

The release-retirement change passes all four callback-order/prefix schedules
and the four new adverse controls: an effectful old press commits only literal
user input then loses word authority; a missing key without revocation keeps
the refusal path; a foreign owner remains current; and a real sensitive-content
handler clears the retained predecessor. No control grants a suffix frame or
manual GUI effect. The full affected set is **527/527**, from 530 discovered,
with exact identity equality and three performance exclusions. The existing
224-byte rendezvous outcome and WordLineage size assertions pass unchanged.
Local `run-9u76bv23/RESULT.json` SHA-256:
`abe4ccdbc2bf621b6ccc1109440ed0aada1533a6f241436971ef2776cb4eaee2`;
archive SHA-256
`75056b99eedfd8efdd05904d761a49f723bd85c406d9b173dc26f9e33dfcfea6`.
Remote `run-emZdvD/tests/SUMMARY.json` SHA-256:
`60aef49843323f43f8d5706a3e97459b5b9a2157dfefad95097ca11a5ee7843e`.
The earlier `run-yy01hfcd` passed 526/526 before adding the adverse control test;
it is superseded for coverage, not a separate repair. No native candidate was
built or installed for this change yet.

### Incomplete right boundary: causal discriminator and consequences — 2026-09-13

The Extra trace's five-character snapshot at cursor three may contain the
entire observed token before the cursor and a remaining rendered continuation
after it. This text identity is a hypothesis to test; lengths alone do not
establish it. The next controlled schedule therefore supplies this structural
case through real SetSurroundingText: confirmed Reset prefix, an actual managed
append, exact retained token before the cursor with an unselected continuation
on its right, then a complete exact snapshot. Test two prefix lengths and one
or three incomplete callbacks. A manually seeded suffix only recreates the
preedit-clear/append effect; it is not a quality oracle.

Compare the current destructive rejection, ignoring the right boundary for
authority, and retaining only non-authoritative lineage while that boundary is
incomplete. Reject ignoring the boundary: it could mutate a prefix of an
unobserved longer word. The retention route is conditional on a causal RED.
Keep all owner/token/predecessor, epoch, exact retained text, sensitivity,
UnknownStart and count checks in one shared identity predicate. For the first
next callback, an exact token before the cursor with a proved left boundary and
a continuing non-boundary character on the right may keep the same record as
unconfirmed and advance its existing armed observation revision. Further such
callbacks remain unconfirmed. A later fully bounded fresh snapshot may confirm
it. Exact unchanged-token duplicate receipts after confirmation, shorter-prefix
duplicates, selection, wrong prefix, an unobserved left continuation and stale
owner must retain their existing refusals. No partial snapshot can authorize
readout, append, manual delegation or a GUI edit.

This distinguishes retention from authority; it does not relax SafetyGate,
edit-plan validation, verifier or exact two-sided snapshot matching. The record
already contains all data and remains bounded even if several incomplete
client callbacks arrive. It creates no owner, generation, queue, timer, wait,
RPC, retry or persistent cache. The extra scalar comparisons are linear in the
bounded tail and use existing snapshot storage; CPU/RSS and native latency are
unmeasured. Callback serialization plus current token/epoch identity prevents
stale results from reviving a different tail. Model/lattice candidates, ranking,
package/delta reload, learning and feedback remain unchanged, including after
future package updates. IME atomic/terminal and daemon consumers retain their
current authority contracts. Rollback removes only this incomplete-boundary
retention branch; the shared full identity checks remain equivalent.

The proof must include four recovery schedules, four contradictory controls,
zero authority/effects before full confirmation, and the full affected IME set.
Afterward, the native diagnostic must actually report this structural class
before claiming it explains the Firefox snapshot; otherwise keep the hypothesis
unproved. No additional browser experiment follows merely from this fixture.

The baseline reaches the first incomplete-right-boundary callback and destroys
the record, as predicted. It passes 528/529 selected/executed tests from 532
discovered, with exact identity equality and three performance exclusions.
All four new contradictory schedules pass; the other three positive schedules
and full-snapshot recovery are not reached. Local `run-bwtthtwg/RESULT.json`
SHA-256 `edcd7057ca9611b137757431c2bee2548a15831f7e7b0f853967468da87c71c4`;
archive `1ba240069967c75dca9f4469d5890e395cf4b4860549f4a60a06936fbb4f00e8`.
Remote `run-pGDlkv/tests/SUMMARY.json` SHA-256:
`36eb502c0f191031165d8a32e4848e4bae4cf828591e5ff5544cc8a761611145`.
Proceed with the preflight's non-authoritative retention, factoring existing
left-boundary and full identity predicates instead of duplicating their guards.

The combined source passes **529/529** selected/executed tests from 532
discovered, with exact identity equality and three performance exclusions.
All four incomplete-boundary recovery schedules remain unconfirmed with no
frame or manual effect until the full snapshot; all four contradictory controls
refuse. Existing exact and shorter-prefix duplicate negatives remain unchanged.
The full identity predicate was factored without changing its conditions;
two-sided matching remains mandatory for exact handoff. All 705 runtime/Cargo
file hashes match the tested archive. Local `run-d14shtuu/RESULT.json` SHA-256:
`9af2387b6e5143e069eda5d3da31e28f0111bfb4602a9ef5f4d46c8db4cea020`;
archive SHA-256
`809955328d7f1cc91de14e922474405694f7c5959205596f87a565459ea304a0`.
Remote `run-ECfSSc/tests/SUMMARY.json` SHA-256:
`99af804625f763520a8977d3ba065894abeefe47b73de936bc2d0359ab231e8a`.
Focused verification took 17.07 seconds. Root checked stale-release retirement,
the unchanged full authority conjunction and non-authoritative revision advance;
this is not another formal scored review.

The next fixed native experiment builds the exact combined source after the
remote full architecture wrapper and runs all three original Firefox scenarios.
One separate Extra case with the already-used `MOZ_LOG=timestamp,IMEHandler:4`
is preregistered to discriminate the actual intermediate snapshot text through
Firefox's existing GetCurrentParagraph logging. It is an instrumented producer
trace, not an uninstrumented latency or additional acceptance denominator. No
input, deadline, GTK transport mode, browser patch or installed binary changes.
Retain every failure and verify owned-browser cleanup and restoration afterward.

### Native producer counterevidence: reject right-boundary repair — 2026-09-13

The frozen candidate is
`/home/ubu/.cache/lay/development/td121-firefox-observation-continuity-20260913/`.
Its IME SHA-256 is
`c8a97fb267021733a76903984a84fbd9cb7878f2e1560a4e78488482a75e3192`;
daemon SHA-256 is
`22193b24914cf306d531e2733dcdafcf286480206c1dfb09ae7f8e4048c5edbd`.
The remote full architecture wrapper passed before the guarded build, with
705 runtime/Cargo source hashes checked before and after compilation.
The graph SHA-256 was
`b10542a71a419da370668683e741913ab1f8945967cbb3ec4d211fa18168bf00`;
target usage was 10,136,055,808 bytes, within 12 GiB.

The fixed native baseline passed **0/3** cases: two pairs produced zero of two
required toggles and left `ghbdtn`; one pair produced zero of one and left
`ghbdtn`; autocomplete plus one pair produced zero of one and left `про`.
All three first lose their retained Reset lineage on a surrounding callback
after a real worker displayed its suffix and the next printable key committed.
None exercises `incomplete_right_boundary` or `retired_after_revocation`.
This run does not establish native release-retirement coverage or reproduce
the earlier acquire deadline failure.

Exact receipt:
`/home/ubu/.cache/lay/development/td121-firefox-fast-repeat-20260913-t__qoo7d/firefox-observation-continuity-baseline/RECEIPT.json`,
SHA-256 `79094a9f474e1f0fb7248f73e22fbe122da3f5be576b8cd050beb90877026804`.
The preregistered producer-log case also fails **0/1**, with zero of two
toggles. Its receipt is the sibling
`firefox-observation-continuity-producer-log/RECEIPT.json`, SHA-256
`921ef10352994a9e4e659ab70022093dc9134aac12038342f83affeefecd576a`.

Firefox's existing GetCurrentParagraph logging now proves the actual text:
at 14:26:07.963748 UTC, `gh`, cursor 2; at 14:26:07.965107, `ghyll`,
cursor 3; at 14:26:07.973985, `ghb`, cursor 3. The middle callback contains
the previous prefix and published ghost suffix with an already advanced cursor;
it does not contain the newly committed `b`. Therefore the hypothesis of an
exact current token with only an incomplete right boundary is false for this
failure. The correct client snapshot follows 8.878 ms later, after the lineage
has already been rejected. This is a client presentation/cache-version mismatch,
not independently measured contradictory text from a later user edit.
The case is `ghbdtn_extra_lshift_enter-54d31d1d48276e98f1f9`;
its 623-line `firefox-process.log` SHA-256 is
`783b25d5b48fac32bf6f0e89a7996910f82e88a59a1c27985028c2d522a3631c`.
The 202-row IME trace SHA-256 is
`726d702c60875aca4c27bcb43823fd333ace6199eb7d034a5e10bff73070b97a`.

Verdict: **REJECTED AS FIREFOX REPAIR**, despite the synthetic 529/529 result.
Restore only `observation.rs` and `adapter/tests/residuals.rs` to their exact
`run-9u76bv23/source.tar` bytes, removing the right-boundary branch, factoring
and its two test functions. Preserve suffix-frame admission and older-release
retirement with their prior causal proofs. The discarded experiment and tests
remain reproducible in its sealed source archive; no assertion is weakened.
The immediate next investigation is the actual preedit/CommitText output order
and Firefox's composition/cache handling, before selecting another repair.
No new record, retry, delay, browser patch or transport switch is authorized by
this result. Model/lattice, ranking, package/delta, learning and feedback are
unchanged; general quality, heldout, CPU/RSS and physical-keyboard acceptance
remain unmeasured by these client tests.

Both native receipts report successful restoration and no fatal harness error.
After each variant the installed C20 IME/daemon hashes match, the user's Firefox
PID/start identity is unchanged, and both global IBus process identities are
unchanged. Runtime authority is unchanged: these bytes were confined to owned
smoke processes; no production installation occurred.

### Published-preedit provenance: consequence check before code — 2026-09-13

The producer trace narrows the first failure further: OnCommitCompositionNative
receives `b` while still composing, calls DispatchCompositionCommitEvent, then
GetCurrentParagraph runs before the later empty-preedit/change/end signals.
Lay already sends CommitText before clear_preedit. IBus 1.5.29's
`_ibus_context_commit_text_cb` emits commit and immediately requests surrounding
text. Firefox 155.0.1 immediately updates its composition/selection state while
the child content cache still contains the previously published preedit.
These pinned sources and hashes are in the existing `source-evidence/SOURCES.json`.
An output reorder therefore cannot remove this demonstrated asynchronous gap.

Compare the unchanged destructive rejection, retaining every mismatched client
snapshot as unconfirmed, and retaining only a snapshot exactly explained by a
previous successful Lay preedit publication. Broad mismatch retention loses the
distinction between a known pending presentation and an unrelated edit; reject
it. A producer cache-coherence repair would need a separate GTK/Firefox change
and deployment route; the current engine cannot make that cache synchronous.
Select a bounded publication witness inside the existing Reset re-receipt record.
This is an explicit replan after native counterevidence, not an extra scored
review or a relaxed right-boundary test.

The existing record knows the current observed token but has no representation
of the rendered suffix it has already retired. That missing provenance prevents
it from distinguishing the measured old presentation from arbitrary wrong text.
After a successful legacy inactive UpdatePreeditText publication, and only with
the currently confirmed exact Reset receipt, retain its original prefix scalar
count and exact prefix-plus-published-text. Bound the combined string by the
existing 160-scalar tail limit. New publications replace this one witness;
Reset, owner/content/navigation invalidation and record consumption remove it.
Verified printable appends carry it within the same existing owner and lineage.
Clearing display/candidate authority does not erase this output provenance.

A next client snapshot may retain the record as **unconfirmed only** when the
owner/token/predecessor, epoch, current observed text, unknown scope and scalar
counts remain valid; at least one observed append follows publication; the full
old presentation matches at the current token's start with proved external
boundaries; the unselected cursor lies in that old suffix at the position implied
by the newly observed token; and the observation revision is the next one.
Repeated matching stale presentations advance only that existing observation
revision. They cannot yield a display frame, accept Tab, delegate a manual edit,
settle the word scope or synthesize client evidence. A later exact full client
snapshot may confirm the record and discard the publication witness. Full exact
matching takes precedence if typed input happens to equal the old suggestion.
Wrong presentation/prefix/cursor, selection, unobserved continuation and stale
ownership keep destructive rejection. Existing unchanged-token duplicate
negatives retain their assertions.

The principal risk is retaining an unrelated word after a cursor or programmatic
edit. The publication equality and existing lifecycle/identity checks bound
retention; actual edit authority still requires the unchanged complete exact
client receipt and existing verifier. No candidate/lattice source, ranking,
positive/contradictory model evidence, package/delta identity, learning or
feedback changes. The witness is output provenance only and cannot become an
edit target or a client snapshot. Future packages may change its text but never
its proof role. Long publications outside the bound keep existing refusal;
they are still displayed and candidate competition is unchanged.

There is one optional bounded string in the existing record, at most 640 bytes
of UTF-8 payload, and scalar metadata. Existing record clones also clone this
payload while it exists; this allocation cost must be acknowledged, not called
free. No new mutable owner, queue, generation, timer, wait, RPC, retry or
deadline change. Publish/receive callbacks remain serialized by the engine;
token and epoch checks exclude stale worker output and foreign callbacks.
CPU/RSS and native tail latency remain unmeasured. Atomic, active composition,
terminal transport and daemon edit consumers retain their current contracts.
Rollback removes the witness, publication hook and matching branch together;
there is no wire/config migration. A future coherent producer can remove this
bounded compatibility witness after the original fixed native cases prove it
unnecessary.

Before runtime edits, reproduce the failure through an actual legacy preedit
publication, observed printable appends and real SetSurroundingText callbacks.
Use four prefix/burst combinations with repeated stale receipts, including
Unicode suffixes and external boundaries; verify no authority before a full
receipt, then exact ManualToggleV3/VisibleTailV2. Add adverse publication,
presentation, prefix, selection, cursor, continuation, sensitive and owner
controls. Run the entire affected IME set with exact test-identity equality.
Only then build after the full remote architecture wrapper and rerun the same
three original native Firefox scenarios. Their next first failure remains an
open mechanism; this preflight does not claim it is already solved.

The causal baseline passes **528/529** selected/executed tests from 532
discovered, with three performance exclusions. The new positive test fails
exactly after a verified UpdatePreeditText/ShowPreeditText publication, actual
CommitText append and the first old-presentation client callback: the observed
lineage is erased. The ten adverse controls pass; later positive schedules are
not reached. Local `run-qlu7be8t/RESULT.json` SHA-256:
`c80a0a3fee163a82967e4b739893d1ed9e6f91aa202152252b08ee16c6ca7e55`;
source archive SHA-256:
`a7d04528bb4b84a078b84832a4c8f5cdeef6b7aed3c371647f1be5941f33cdbd`.
Remote `run-R1E4qo/tests/SUMMARY.json` SHA-256:
`a67d5d9e33cffd30130bd1d42864c2cb11a66eecd270b93647a6e7a140ffb779`.
The test fixture publishes through the same production preedit method as real
candidate output; it does not install a provenance witness itself. This is a
causal mechanics failure, not candidate quality evidence. Proceed with the
preflight's publication witness; no other production change is selected.

The witness change passes **530/530** selected/executed tests from 533
discovered, with exact identity equality and three performance exclusions.
The four prefix/burst schedules preserve only lineage across three old client
presentations each, including Unicode, bracketed display and external word
boundaries; the later full client receipt reaches exact manual delegation and
the exact VisibleTailV2 surface. Thirteen adverse controls include no successful
publication, excess witness length, contradictory text/position/selection,
stale owner/content, attempted Tab while stale, Reset, replaced publication and
unchanged-token duplicate. All refuse. Two additional exact-match schedules
prove that typed text equal to the retired suggestion uses the full receipt
without being mistaken for a stale presentation. Learning remains empty.

Local `run-24q9lb4r/RESULT.json` SHA-256:
`c267beadcb408b788c7e7ab4baf1f1a315fdf2a644c60403d518beb4f88de852`;
archive SHA-256:
`d65c15a4607c60f22937c1c883fa10fd8168b2217c89aba5ea1a2175693a0639`.
Remote `run-2cT2H2/tests/SUMMARY.json` SHA-256:
`6a12623e7f9ffb18ba2aa937fa14c0a6435a2e34fb0f81726e0b30efeb0b9722`.
All 705 runtime/Cargo hashes match the tested source. Focused verification took
17.25 seconds. The earlier `run-rn8tunr6` passed 529/529 before the final controls;
it is superseded for coverage. Root verified that the full authority conjunction
is unchanged, no candidate output is suppressed by the witness size bound,
and no witness is installed before a successful legacy publication. This is
unscored implementation review. Native acceptance remains open.

### Published-preedit native result and remaining first losses — 2026-09-13

The exact candidate in
`/home/ubu/.cache/lay/development/td121-firefox-published-preedit-20260913/`
has IME SHA-256
`d69e1506e9ab54de91c9826188cfc3fb3125f40843ad262a993b24a8ac1d110b`;
the daemon remains
`22193b24914cf306d531e2733dcdafcf286480206c1dfb09ae7f8e4048c5edbd`.
The full remote architecture wrapper passed; the graph has 22,642 nodes and
60,185 links, SHA-256
`f4d8a3fc8c6badc41e6838caba902c24aa5008f6b2fac4252f32194bd98d4438`.
The guarded build checked all 705 manifest rows before/after, used
10,136,072,192 target bytes, and froze artifact binding SHA-256
`c8ae411d9af5aff47d9b571005ed76636c68a4ba1e59b53d7ba60b6853d9b1b4`.

The fixed three-case harness reports **1/3**. Separate actual valid client
captures from invalid capture: **1/2 valid cases pass**, and **1/1 capture is
invalid after focus loss before input**. This is not aggregate product
acceptance and cannot close TD-121. Receipt:
`/home/ubu/.cache/lay/development/td121-firefox-fast-repeat-20260913-t__qoo7d/firefox-published-preedit-baseline/RECEIPT.json`,
SHA-256 `744eb0a8b3bf3e7c98e2deab762e5d9e7bd5d93814c2428c9631e2fcd180de92`.

- `ghbdtn_fast_lshift_enter-e9ada829a837e3cf92ba`: **PASS**, exact visible
  `привет`, one of one toggle. The real worker publishes `yll`; IME row 110
  takes `published_preedit_cache`, later exact receipts recover all six observed
  characters, row 204 consumes that exact receipt, and row 205 delegates once.
  This native trace exercises the new witness branch. Its 381-row trace SHA-256
  is `621d06960171615e22cc02114d04c841e7196208e8d1c4ca041498b11579376b`.
  Later replay bookkeeping does not invalidate this completed single-pair
  surface assertion or establish the next pair's acceptance.
- `ghbdtn_extra_lshift_enter-33d8c67d1680654f9ac7`: **FAIL**, zero of two
  toggles, final `ghbdtn`. Before the first printable key, compatibility request
  3 arms/emits its marker, then row 54 records
  `marker_not_observed_before_expiry`, no owner and revoked reducer. Later
  bridge markers refuse `lifecycle_pending`. No preedit or Reset re-receipt is
  reached, so this case neither exercises nor refutes the publication witness.
  Its 181-row trace SHA-256 is
  `f7c81d6fbcc557fffc81af7ebf21f344cef714ecb71442eaeab9ba1d4c84dd6f`.
- `ime_autocomplete_then_double_shift_enter-1de038d955fea2e4119c`:
  **INVALID_CAPTURE**. DOM readiness is at monotonic 932749.870 s, blur at
  932752.346 s, before the sender's first character; no IME printable callbacks
  reach the owned field. The helper times out/interruption cleanup reports an
  empty field. It is not an autocomplete refusal denominator. The harness
  currently checks focus before spawning the sender but lacks cancellation
  after subsequent blur. Before any further GUI input, add a bounded,
  event-driven cancellation of that owned sender on blur; preserve the original
  input sequence and never reacquire or redirect focus after loss. No product
  runtime or acceptance assertion changes are justified by this harness gap.
  Its 97-row trace SHA-256 is
  `313d9981df318514fa440ddeb659f50699f8814f66e44bf8773f7d05ad3edbcd`.

Restoration succeeds with no fatal outer harness error; C20 loaded hashes and
the user's Firefox/global IBus PID-start identities are preserved. Runtime
authority is unchanged. Root's next product investigation is the now-observed
compatibility marker expiry. The existing diagnostic proves expiration but
does not measure Get, emission and receive-stage durations. No budget increase,
retry or alternate admission owner follows from that missing timing evidence.

### Activation timing discriminator: diagnostic-only preflight — 2026-09-13

The 5 ms acquisition budget is shared with bridge fences, so changing its
constant would also alter mutation-time latency and stale-lease behavior.
Source inspection confirms compatibility acquisition already uses the existing
zbus executor; there is no per-request OS thread to remove. The trace writer
already has a bounded asynchronous nonblocking queue. Neither speculative
thread removal nor synchronous-log removal explains the native expiry.

Select the smallest remaining discriminator: opt-in metadata timestamps at
compatibility Get start, accepted reply, marker emission/ingress and matching
fence expiry. Use the existing request/nonce and optional existing deadline;
record wall-clock microseconds for cross-stage correlation and signed remaining
monotonic deadline microseconds. Wall time is diagnostic only and cannot grant
authority or replace the existing Instant deadline. An authenticated own marker
arriving after its fence was removed still records ingress with its nonce, so
it can be correlated without retaining an expired request or adding a cache.

Compare retaining the current insufficient phase-only trace, adding an external
D-Bus monitor, and extending existing opt-in metadata. External monitoring adds
a bus consumer and cannot observe the engine's actual fence/observer schedule;
select the existing diagnostic path. Disabled diagnostics add only their
existing atomic flag check; enabled diagnostics add bounded scalar formatting
and clock reads to the existing queue. Their scheduling perturbation means the
instrumented run cannot claim uninstrumented latency. CPU/RSS remain unmeasured.
No user text, model/lattice/ranking, package/delta, learning, feedback, cache or
invalidation changes; no owner, timer, queue, fallback, RPC, deadline or decision
changes. All fields are captured outside reducer/pending locks. Rollback removes
the timing calls/formatter without protocol or configuration migration. Extend
the existing diagnostic privacy/schema test and rerun all affected IME tests.
After the full remote graph/build gate, execute one fixed three-case native
diagnostic with the event-driven focus-cancel harness, retaining all failures.
Do not repeat it until a new mechanism or corrected harness justifies a new run.

The metadata-only source passes **530/530** selected/executed tests from 533
discovered, with exact identity equality and three performance exclusions.
All 705 runtime/Cargo rows match the checked source. Local
`run-pnj7_9n8/RESULT.json` SHA-256:
`67d9786165bf56acdd52ac65edd5b37cf07b9898533a93886efd999fe318bee3`;
archive SHA-256
`78cd3c1b25672e07ff0de1de4c3cfecaff35d8be556c6bf0589606e3e92e2cfc`.
Remote `run-QBPONX/tests/SUMMARY.json` SHA-256:
`3fa9eb8285786331d9f6765da2a0e420d5c5fe00f813e58f54305bc3f8030f9d`.

The external owned harness now sends blur/completion metadata through an
inherited pipe. A blur after readiness terminates its tracked sender/daemon;
blur before spawn rejects launch; completed capture does not cancel on cleanup
blur; closed/malformed guard input cancels. The helper marks lost-focus capture
invalid immediately. It neither polls the GUI nor reacquires focus, and keeps
the original keystrokes/deadlines. Remote guarded cancellation proof passes
**5/5** with no GUI input. Exact local proof is
`/home/ubu/.cache/lay/development/td121-firefox-fast-repeat-20260913-t__qoo7d/focus-cancel-proof.log`,
SHA-256 `8093e61bb5ee98c1b90fa10305de8dcb1a6816b66d708265a60b2b56b486b8ad`;
the sibling `firefox-focus-cancel-variant.json` SHA-256 is
`e57d44e2ef6b0ecbe29d0a5b6a9524b0292bdfa955a3dfde691607e59a2bb541`.
The guard module SHA-256 is
`7c915fd71b139beafed70f020ce9d70998773b093719f3edd659389ec9b5bfcf`.
These are harness safety mechanics, not Firefox product or physical acceptance.

### Measured activation expiry and Alt preparation loss — 2026-09-13

The diagnostic candidate is
`/home/ubu/.cache/lay/development/td121-firefox-activation-timing-20260913/`:
IME SHA-256 `ff6786b141bdbf8ff54e1e5ecee4b3fc99e5d14709c7f5c8273206a53606a0f6`,
daemon unchanged `22193b24914cf306d531e2733dcdafcf286480206c1dfb09ae7f8e4048c5edbd`.
The remote full architecture wrapper passed, graph SHA-256
`69e82c31506b7c74af818363a1027dd619eee5bf633dba2a4c760d08d7d6c40e`;
guarded build checked 705 manifest rows and used 10,136,145,920 target bytes.
Artifact binding SHA-256:
`41b7ce92c5ca7a9c4d5e5e3090a4e2f5c976a704fb5712517981ca4129e60ae5`.

All three captures are valid and none triggers focus cancellation: **1/3 PASS**.
The exact receipt is
`/home/ubu/.cache/lay/development/td121-firefox-fast-repeat-20260913-t__qoo7d/firefox-activation-timing-baseline/RECEIPT.json`,
SHA-256 `7767eab755adf8b979aea702304680c185427af1a969174ecc7127054741f2d3`.
The single fast pair yields exact `привет` and one toggle. Its 405-row trace
SHA-256 is `cc68a767f9a82e1b813c0672eb049eb39eaa1a5756bd1cc0bed2c525cb0c4f3b`.

In `ghbdtn_extra_lshift_enter-e2a1a46a50ea70d53664`, request 3 starts its Get
with 4,955 microseconds left; accepts the reply 3,119 microseconds later; emits
the marker at +3,590 microseconds; expires at +4,994 microseconds; and observes
the authenticated marker only at +5,933 microseconds. Including the initial
45 microseconds, the chain takes **5,978 microseconds**, exceeding its 5,000
microsecond deadline. No intervening key/lifecycle callback appears between
reply and marker. The fence has already been removed, so later bridges refuse
the revoked lifecycle and zero of two toggles occur. This instrumented trace
identifies an actual end-to-end budget failure, not a missing readiness predicate.
Its 196-row SHA-256 is
`f57e3464ecac9cc4497f845b68ab9acced56a2a7f192ac95f0070eb4c5bffbbf`.
The other twelve activation chains in this fixed run fit the old budget;
instrumented timing is not an uninstrumented latency distribution.

In `ime_autocomplete_then_double_shift_enter-8c981033ef62a58b8a4b`, the real
worker publishes the completion, the published-preedit witness retains the
intermediate cached presentation, and a fresh three-character receipt confirms
the prefix. Alt press 54 is admitted and has no text effect; its settlement
clears `context_reset_rereceipt`. Alt release 55 then reaches `alt_accept` but
returns false. Reset/focus changes follow and the later manual trigger has no
word authority. Both `advance_context_word_scope` and
`advance_context_reset_rereceipt_after_key` unconditionally discard the receipt
on the preparatory Alt press, even though acceptance occurs on release. The
visible `проверка` alone is not proof of IME-owned acceptance: no corresponding
IME append or successful Alt acceptance occurred. Its 242-row SHA-256 is
`f325f7202599cd4343dbbeaf3f973ee8c48443095382716c3f10fc21afe21669`.

Restoration and the user's Firefox/global IBus identities are verified; C20 is
still installed. Runtime authority remains unchanged. The next two mechanisms
are bounded lifecycle acquisition and zero-effect completion-modifier lineage,
not individual word exceptions or a change to candidate quality.

### Separate activation deadline and preserve Alt preparation: preflight

For the measured activation loss compare the current global 5 ms budget,
restarting a 5 ms allowance for each Get/marker phase, and one separate 50 ms
end-to-end deadline for asynchronous focus activation. A per-phase allowance
would fit the observed split but adds phase-dependent deadline semantics and
still confuses activation policy with bridge fencing. Select a single 50 ms
activation deadline while preserving the existing 5 ms bridge/callback budget.
The 50 ms value is explicit bounded scheduling headroom, not a measured latency
claim. Normal completion publishes immediately; no preparation sleep, debounce,
retry, polling or delay before a key/pair is added. Both native and compatibility
activation use the same policy, including a request upgraded to native origin.

Use one extra immutable Duration in existing adapter configuration/state and
the existing pending fence/timer. Current request generation, nonce, context,
profile, lifecycle revision, source seal and settled-key checks remain the full
publication conjunction; exceeding the selected deadline still revokes. A
stale reply or focus change cannot publish merely because it arrived within
50 ms. Requests/fences and callback queues retain their existing bounds. The
operational cost is retaining a failed activation request for up to 50 ms;
key-before-ready still emits only its existing literal behavior, with no word
authority. Bridge mutations and their existing enclosing deadlines do not
inherit the longer budget. CPU/RSS/native tail latency remain unmeasured.

For Alt, compare the current destructive preparation, accepting on press, and
preserving the current receipt across its zero-effect press while retaining
release-only acceptance. Accepting on press would change shortcut semantics;
select preservation. All three existing completion modifiers must share the
rule. An actual effectful release retires the old Reset receipt; its existing
append/boundary settlement owns the new tail. Intervening command input,
navigation, stale owner, sensitivity or lifecycle loss still revoke or refuse.
Keep the existing modifier gesture owner and passthrough/handled results.

Neither change alters candidate/lattice retention, ranking, model evidence,
package/delta reloads, learner/feedback semantics, SafetyGate, edit-plan or
verifier authority. Future packages do not affect marker identity or a
zero-effect modifier. No new owner, persistent cache, queue, fallback or consumer
transport is introduced. Callback serialization and existing token/epoch checks
continue to exclude stale completions. Rollback removes the activation budget
field/selector together and restores the two modifier invalidations separately.
The earlier publication witness and causal proofs remain independent.

Before production edits, add two causal discriminators: default-config
activation must survive one authenticated marker deliberately delivered after
the measured old 5 ms bound, and real legacy Alt press/release must preserve its
confirmed prefix until the append. The timing test uses an existing Timer only
to release a held protocol event past that specific deadline; it is a controlled
deadline fault, not a warmup sleep or retry. Assert actual publication/consumption,
exact tail/effects, all modifier variants, and no learning from unknown origin.
Preserve the explicit 5 ms expiry tests. Add same-delay bridge refusal and
stale lifecycle/nonce controls, and keep all affected IME contracts green.
Afterward rerun the original three native scenarios with the focus-cancel
harness; later Reset/post-replay failures remain separate open mechanisms.

### Activation and Alt causal RED — 2026-09-13

Tests-only snapshot `run-w_cjlui5` (remote `run-eloIwR`) discovers 539 tests,
selects and executes the exact same 536 identities, and passes 532 with four
causal failures; three unchanged performance exclusions remain separate.
The real native and compatibility activation paths both lose their held,
authenticated marker after the old 5 ms deadline. The nonce/lifecycle test
fails at its valid-marker positive control for the same reason. Actual legacy
Alt press loses the confirmed Reset receipt before its effectful release.
The same-delay bridge refusal and all 21 intervening Alt-gap controls pass.
All 530 prior selected identities still pass. No runtime authority changed;
these are controlled regressions, not native Firefox acceptance.

Exact evidence directory:
`/home/ubu/.cache/lay/development/run-w_cjlui5/`.

The RED `RESULT.json` SHA-256 is
`611c8de6f6c0d8b95035855f5f7fd51ee127e9ac9a36c40b328382fe83abac92`,
archive `909ce898b930ba7956838b946297e74933ab77240ff70d55b46c02cb79b95e39`,
and test summary `30da864ccfe5a906b94619159bf28c950c695be0666e0d53c8ac4719a231c05d`.
Against the preceding runtime, only the two test files and the generated
architecture receipt differ among the 705 Rust/Cargo/receipt rows.

The selected implementation uses the existing single fence deadline with a
separate immutable 50 ms activation budget. The test-only explicit budget
override continues to set both budgets, preserving the existing forced 5 ms
and longer controlled schedules. Actual zero-effect completion-modifier
preparation preserves the Reset receipt; an effectful release retires it.

The first post-edit run, `run-39_zp_ey` / remote `run-PdaZzr`, passes 535/536:
activation and negative controls pass, but one Alt release returns false after
the receipt-preservation assertion passes. Its summary SHA-256 is
`336758b3204f7e1382dc082328e146030756ed990720fd5716bae2d87bda9ecf`.
No additional runtime change follows. Additional test assertions establish
the gesture, suffix and current receipt before release; a temporary independent
verifier probe passes and is removed. Both diagnostic `run-rkgnh7pj` and final
`run-jtq8bzie` then pass 536/536. The latter checks exact selected/executed IDs
and all 705 current source rows. The earlier isolated release refusal has not
been causally attributed; scheduling is a hypothesis, not a measured verdict.
Keep that counterevidence and the added failure-state diagnostic. These runs
are component proofs only; native Firefox, physical input, release, resource
latency and runtime promotion remain open.

Final focused evidence:
`/home/ubu/.cache/lay/development/run-jtq8bzie/` (remote `run-2fdLlf`).

Its `RESULT.json` SHA-256 is
`70371762ae6164512b68ce57dc32904858a1f163ab6fb5fd702ce4aa85b64710`,
archive `9fd81bfd22817e91d562018890212805f559d3e2c28dac74d2129086efd69934`,
and summary `578496641a21acfbb0256b68a081d09963e63f03967f5071e576a45ebe5beaac`.

### Native activation/Alt candidate: remaining bridge conflict — 2026-09-13

Frozen candidate directory:
`/home/ubu/.cache/lay/development/td121-firefox-activation-alt-20260913/`.
IME SHA-256 `0ddbf7bc6f5313e2e03496df8fe264a005a75a85772e50e8c49e5cc258c24be6`,
daemon `22193b24914cf306d531e2733dcdafcf286480206c1dfb09ae7f8e4048c5edbd`.
The guarded full graph wrapper and release build pass on the 705-row binding;
graph SHA-256 `1f0c655252372e5f05acecef297d0a26f18042ecf618dfd9163d1b229cce2628`,
artifact binding `d5f957422dd6e3ae85585c33f475181ccbaa7ca9805b8c729dbbd1b7e34eb140`.

The fixed native run is **0/3 harness PASS**: both Shift captures are valid
failures; the autocomplete capture is ultimately cancelled on later DOM focus
loss. Its earlier acceptance and manual-refusal trace remains usable causal
evidence, not an end-to-end PASS. Receipt:
`/home/ubu/.cache/lay/development/td121-firefox-fast-repeat-20260913-t__qoo7d/firefox-activation-alt-baseline/RECEIPT.json`,
SHA-256 `32de0170e733c73bb334bd949ca0b9e307d91c997b884875c04dbaba452c2b80`.
All 13 observed activation chains publish, maximum 2,529 microseconds; none
exercises the extended part of the deadline in this run. This is not a latency
distribution or a claim that scheduling overhead was reduced.

The single-pair trace has a confirmed six-character Reset receipt. At row 216,
the observer receives the final Shift release, serial 60; at 217–218 the bridge
marker records `unsettled_callbacks=1` and publishes a ready fence with no
token. The bridge returns refusal immediately; the read-only release handler
then enters and settles at 219–223. Trace SHA-256:
`aa9a188820ef9e4b67e06b0cd31519399854988501648db6f3e73e8211aee71b`.

In the two-pair trace the first bridge passes its marker and consumes the
confirmed Reset receipt at 244–245. Release 72 then arrives; its settlement
and every following Shift settlement refuse. There is no delegation event or
text effect. Source inspection identifies the remaining interval between
fence acquisition, Reset-suffix binding and `publish_tail_handoff` settlement:
any received key, including an observation-only Shift, conflicts with the
bridge. The exact internal interleaving after row 245 still needs a controlled
proof; do not label the inferred publication failure as an instrumented fact.
Trace SHA-256:
`83bc511b398fee4e9d26416410c5ec488ba8ec11b94ded1ee1c7865a46a6cff0`.

Autocomplete now accepts on Alt release 55: rows 173–177 record the append,
`handled=true`, KnownStart settlement and the exact nine-character client
snapshot (`проверка `). The original sender deliberately calls `double_alt`;
its second Alt release 58 returns false and is followed by FocusOut/FocusIn
and a new source-free owner before manual input. The relationship to Firefox's
menu focus is a hypothesis. Original end-to-end failure stays recorded; a
separately named single-Alt native control can distinguish it without changing
the original fixture or swallowing normal application shortcuts. Trace SHA:
`0354b5b3e1b493d92e6a762733a33d054c91e51e28405b5cae76878fc66ead9f`.

C20 is restored; loaded daemon/IME hashes and the user's Firefox/global IBus
identities are verified. No installation, source acceptance, physical or
general quality promotion occurred.

### Read-only Shift observation at the bridge: consequence preflight

The shared first mechanism is treating every received key as a possible word
mutation, although the protected legacy Shift handler only observes gesture
state and cannot issue a manual edit. Compare: (1) waiting for all callbacks
inside the existing fence deadline, which addresses the pre-marker refusal but
still loses to the same Shift received after the marker; (2) classifying the
proved legacy Shift observation in the existing bounded unsettled queue; and
(3) capturing all physical Shift input before IBus, which expands daemon input
ownership and replay semantics. Select (2), subject to the controlled proofs
below. Do not extend bridge deadlines or introduce a second gesture detector.

The adapter alone can classify an authenticated, correctly decoded legacy
`ProcessKeyEvent` body with a left/right Shift keyval. Atomic events, Alt
release, printable input, navigation, malformed/unknown bodies and all other
keys remain possible word mutations. Retain every callback in the existing
queue and use its existing owner/header/revocation settlement; an observation
is not an early ACK. Only bridge conflict detection may distinguish this
read-only footprint. Activation/transfer readiness and stale-callback guards
retain their current complete-settlement contract. The actual legacy handler
must have a tested, unchanged no-text-effect boundary; no generic key-release
exception is admissible.

Candidates, lattice/ranking, false certainty, model/delta reloads and learning
remain outside this physical projection path. SafetyGate, edit validation and
verifier proofs remain mandatory. An Alt acceptance remains effectful and must
block a concurrent bridge. A stale owner, context/lifecycle or text-changing
key still refuses; the bridge's engine lock and repeated source/focus/epoch/
snapshot checks remain mandatory. Test callback delivery on both sides of the
marker and during exact-tail publication, including the late handler after
delegation. The pure observation must neither overwrite a newer settled word
nor manufacture a known beginning. Reject any implementation that cannot bound
disabled/composition/atomic behavior through that existing handler contract.

Expected cost is one small discriminator per already bounded unsettled entry
and a bounded queue scan for bridge conflicts; no new cache, timer, retry,
queue, owner, RPC or persistent state. Bounds remain 64 key callbacks and the
current 5 ms bridge/callback deadline. Real CPU/RSS/latency costs are unmeasured.
Future dispatch changes must fail the observation-only invariant tests before
they can invalidate this footprint. Rollback removes the classification and
bridge predicate together; the publication witness, Alt preparation and
activation-deadline repairs remain independent. Before production edits,
reproduce both native event-order failures with real typed messages and exact
manual dispositions/tail effects, and preserve all conflicting-input controls.

The dispatch audit finds one necessary boundary correction before the footprint
can be sound: the current disabled-backend cleanup precedes the legacy Shift
branch and may clear a retained composition/tail. Keep managed-input and
handled-release handling as they are, then route non-speculative legacy Shift
directly to the existing observation-only helper before that cleanup. Atomic
speculation keeps its existing detector and cleanup ordering. This enforces
the protected no-output Shift contract for both enabled and disabled text
backends. A Shift alone will no longer hide an obsolete preedit after switching
away from the IME backend; existing lifecycle and subsequent non-Shift handling
retain that cleanup. Test the no-output/no-tail-or-epoch-change invariant across
backend and managed-input settings before relying on the classification. This
is not permission to classify Alt, generic releases or any atomic input as
read-only. The footprint remains in the original bounded callback entry;
all callbacks still require their real eventual settlement.

The tests-only discriminator is `run-ztuni7kn` / remote `run-iK8zg0`:
544 discovered, exactly 541 selected/executed, 538 PASS and three causal FAIL,
with the same three performance exclusions. The real `ManualToggleV3` entry
refuses a typed Shift received before the marker while its actual D-Bus handler
is held by the engine lock. The controlled post-binding interleaving makes
the production exact-tail publication lose its live handoff. The separate
disabled-backend test confirms that the old ordering clears the composition
before reaching the Shift observation branch. All 536 prior identities and
the new conflicting-input/queue-capacity controls pass. Rename the queue test
to describe its actual capacity scope; activation readiness is unchanged by
construction and is not an additional measured claim of that test.

Exact directory `/home/ubu/.cache/lay/development/run-ztuni7kn/`;
`RESULT.json` SHA-256 `ba6aaee7343587e0db050d0a1c6777bb8723825877cec53adea9338a90a0314e`,
archive `2b273e26a85a35e4000a06bb8315c98d69f9b0ebff92708718137f7c680f46c0`,
summary `712c6c47b80fd428fc9a3a1f479a267dd730944422671fb51644b4ff9db9541f`.
The publication interleaving is now causally reproduced; it is no longer only
an inference from the native trace. Runtime authority remains unchanged until
the following uninstalled source repair is independently gated.

### Shift footprint focused GREEN — 2026-09-13

`run-c_f_rxj2` / remote `run-2zpUax` discovers 544 tests and passes all 541
selected/executed identities, with three unchanged performance exclusions.
All 705 Rust/Cargo/generated-receipt rows match the tested archive. The real
bridge/held-callback matrix, both sides of Reset-suffix binding, late callback
settlement, disabled-backend pure-observation matrix, ten conflicting-input/
lifecycle controls and 64-entry callback capacity proof pass. Both formal
scored TD-121 review passes remain exhausted; this is the root-owned replan
proof, not a third scored review or a release/physical verdict.

The source retains every Shift in the existing queue with a `KeyWordEffect`
discriminator. Only the bridge's conflict predicate filters proved legacy
Shift observations. Ordinary settlement, activation/transfer readiness and
queue overflow still account for all callbacks. Legacy Shift reaches its
existing no-output helper before disabled-backend composition cleanup; atomic
speculation keeps the previous path. No word examples or case IDs enter
runtime conditions. Native acceptance, CPU/RSS/latency and installation remain
open, and the original pre-footprint native failures remain counterevidence.

Evidence: `/home/ubu/.cache/lay/development/run-c_f_rxj2/`;
`RESULT.json` SHA-256 `92e9609f26fbeb9a76d00297f11dd7bce1b2b95fdefb9d8d35dfc1ac648982d1`,
archive `d7a62284a5ba777fefeb8dd10b9e25b76995f1fd4f49d18e6aad205e33feee7e`,
summary `b4ca3dcfeae2184b828771a6d1f7dca0a874fec6d749189bdc1e99dffdfa26a9`.
The next gate freezes a fresh native candidate. Keep the original three
scenarios and add a separately named single-Alt autocomplete control to
distinguish the second ordinary Alt's application-focus behavior. Any test
sender extension must be explicit and retain the original scenarios' input
sequence and timing; it is not a runtime shortcut change.

### Single-Alt discriminator preflight — 2026-09-13

Extend only `lay-test-input`'s existing TSV interpreter with one `alt` tap and
add a separately named autocomplete/manual Case. Keep the original three
scenarios, their expected text, all existing tap helpers and fixture bytes.
The new fixture replaces only the second Alt tap with its 10 ms duration as
sleep: the original 220 ms lead, first 10 ms tap, 80 ms gap, 10 ms second tap
and 900 ms settle become 220 ms, one 10 ms tap and 990 ms. This preserves the
first acceptance and manual-trigger timing while distinguishing application
focus behavior after the ordinary second Alt. No extra warmup or retry is
introduced. Assert one completion acceptance, one exact manual projection and
the same `ghjdthrf` final surface in the new control.

This test-utility-only change uses the existing armed, focus-cancelled owned
sender. It changes no production candidate, lattice, ranking, verifier,
learning, cache, package, authority, latency deadline or input owner. The
test binary is rebuilt and separately bound with its TSV and Python Case
sources; no physical-human PASS follows from synthetic input. Rollback removes
the new parser arm, fixture, mapping and Case together. The original failing
double-Alt scenario remains a separate gate and cannot be replaced by this
control. Freeze the footprint candidate only after the canonical architecture
refresh, then run all four cases once with preserved browser/IBus processes
and verify restoration of the installed artifacts.

### Shift-footprint native gate — 2026-09-13

The canonical guarded remote architecture refresh and three-binary release
build pass. The freeze binds 766 Rust/Cargo/build-script/test-data/smoke-source
rows; all three existing input scenarios retain their source and timing. The
new single-Alt Case is separate. Candidate IME SHA-256 is
`650e9564734a647054cd9fe1d75862e90f9ea8763ba071b6ffd1853ada21565d`,
daemon `22193b24914cf306d531e2733dcdafcf286480206c1dfb09ae7f8e4048c5edbd`,
sender `cd6c89bb1beab99a8e6443c0e40051196e753571eaad042d564f90ea1a7f81ae`.
Build and binding evidence:
`/home/ubu/.cache/lay/development/td121-firefox-shift-footprint-20260913/`;
`artifact-binding.json` SHA-256
`b924644cee9a69018491e546c02928637582bd0619dbf5fa1dcb882f2173e378`.

Native result is **1/4 PASS**, not release acceptance. One fast pair produces
exactly one `привет` projection. Four fast taps produce `привет` with only one
of two required delegations. The queued second pair waits for the exact tail
and refuses `passive:unknown-context`; both Shift callbacks and the first
delegation now settle. Both Alt captures finish by timeout, without a completed
DOM Enter result. Their guard receipts say `CANCELLED_GUARD_CLOSED`, not a
received DOM focus-loss event. The earlier progress inference of focus loss
from the mere presence of the cancellation file is corrected here. The original
double-Alt trace proves one accepted completion followed by an ordinary Alt
and a new IBus focus owner. The single-Alt trace has zero accepts: the matching
suffix was not published before its first Alt. Therefore the second-Alt-only
hypothesis does not explain the single-Alt refusal. Keep the two causal paths
distinct and do not retry this candidate to obtain a green receipt.

The first remaining repeat-path loss is in Reset rereceipt advancement during
the already authorized native replay. After the second replacement character,
Reset arms a two-character candidate; the client delivers a one-character
prefix. The next replay character changes the owned mirror but returns
`handled=false` to let the client perform the exact native input. The generic
managed-append-only rereceipt rule discards its predecessor, so the final Reset
cannot reconstruct all six observed characters. This source explanation is an
inference to reproduce causally, not yet a tested fix. Earlier deletion-side
snapshot mismatches also occur in the one-pair PASS; they alone do not explain
the missing second projection. No validator or daemon retry deadline changes
are justified by this result.

There are 22 published activation chains. Measured get-start to marker-ingress
maximum is 5,065 us; this is an observed chain duration, not callback, mutation,
end-to-end latency, speedup or physical acceptance. Installed C20 hashes were
restored; the original Firefox and global IBus PID/start/hash rows are unchanged.
Native receipt:
`/home/ubu/.cache/lay/development/td121-firefox-fast-repeat-20260913-t__qoo7d/firefox-shift-footprint-baseline/RECEIPT.json`,
SHA-256 `cbd88c91685780e03d7d503258841da1d8c4fb85869e3433036789cfed951c07`.
The repeat trace has SHA-256
`7c87c31baad78d9af61ea8245d880b20fd1f4bafd08bb6d70047bff6c04b7658`;
the single-Alt trace has SHA-256
`b34252fe6bf19010daf2fad1011070db24a6d38b6e31a2b40033adc7a155afc1`.
Runtime authority is unchanged outside the four bounded owned test processes;
the footprint source remains uninstalled. Next proof isolates native replay
append/Reset interleavings through the existing exact suppression and callback
paths before changing production code.

### Validated replay append consequence / causal-proof preflight — 2026-09-13

The bounded candidate repair is to let the existing Reset rereceipt advance
on an append that the existing exact-replay handler has actually validated.
Reuse `exact_replay_tail_change_quarantined` together with the current matched
local/shared exact replay scope; retain all current one-character append,
epoch, owner, lineage, boundary and token checks. The flag alone is insufficient:
the replay mirror sets it before its final scope validation. A rejected or
revoked replay must not gain this exception. Backspaces, generic unhandled
input, command modifiers, whitespace and navigation still cannot extend the
receipt. This continues observed suffix provenance, without manufacturing a
known word beginning or treating the replay plan as client-visible text.

Alternatives considered: allowing every unhandled mirrored append would admit
unproven native input; extending the daemon wait cannot repair discarded
lineage. A second replay receipt/controller duplicates the already bound
local/shared suppression lease and is unnecessary. The selected route adds
no owner, queue, timer, persistent cache, fallback or RPC. Its extra work is
limited to keys that both have a pending Reset candidate and were quarantined
by the exact replay; reuses existing bounded tail/lease comparisons. CPU/RSS
and end-to-end latency remain unmeasured. The 700 ms replay lease, 5 ms bridge
budget and all source/target snapshot validations remain unchanged.

Candidate/lattice retention, ranking, model/package/delta reloads, verifier
authority and learning behavior are unchanged: native projection remains
lexical-free and its existing quarantine suppresses feedback/background work.
The risk is falsely retaining lineage after rejection or cross-owner change;
prove actual input rejection, zero IME text effects, unknown completeness and
no second delegation for those controls. Admission settlement precedes receipt
advancement, so a stale callback or lifecycle race must still invalidate the
token. Even a valid advancement remains unusable for mutation until a fresh
exact unselected surrounding snapshot confirms the full current suffix.
Rollback is the one acceptance predicate only. Future replay scope changes
must keep the real-entrypoint negative controls passing.

Before production editing, run tests-only RED through the existing factory
handoff, `SuppressNextAutocorrectV2`, typed legacy key callbacks and Reset.
Cover Reset after one/two Unicode replacement characters, both exact and
delayed prefix receipts, then the real second `ManualToggleV3`/`VisibleTailV3`.
Revoke the existing replay lease or context in negative controls. No substitute
reducer, sleeps, word-conditioned runtime branch or third scored review is
introduced. Alt scheduling/focus behavior is a separate unresolved observation.

The first tests-only run `run-s18e1nx0` / remote `run-YMw19c` discovers 546,
executes 543 and passes the 541 previous checks. Both new tests fail in their
fixture before the Reset interleaving: the factory helper was previously used
only for bridge readout and its FocusIn reloads the isolated test configuration.
Three native Backspaces return unhandled without emptying the mirror. This is
a setup failure, not the promised causal RED. Select the IME text backend in
the new input fixture, assert the live composition and replay scope, and repeat
the tests-only discriminator. Production source remains unchanged.
Receipt directory `/home/ubu/.cache/lay/development/run-s18e1nx0/`;
`RESULT.json` SHA-256 `b5f1cfa075089eae64e8815a116c4e8def67ab6ebc8e4bb6b59dabb2bde0e44c`,
archive `76f68d532311dc41b07c60c1d3b35ea1b7cccb78f87888f3131a0def4fee761f`,
summary `9db94829ea83309c13045f4429c2759956111ab6c2ffefae7f9eafda7c19ceb4`.

The corrected tests-only run `run-1l7yfz91` / remote `run-6roFV2` is the causal
RED: 546 discovered, 543 selected/executed, 542 PASS and exactly the new valid
native-append test FAIL at predecessor retention. The native delete/replacement
entrypoints and prefix Reset run before that assertion. All eight replay/
context negative controls and the prior 541 identities pass. No production
change is present in this archive. Runtime authority remains unchanged.
Evidence `/home/ubu/.cache/lay/development/run-1l7yfz91/`;
`RESULT.json` SHA-256 `b963ad86cefdfc7f01073c849e15d75ef07e97e996701999900fc484e8f29bcf`,
archive `1cfeccf141faab8128d8f11639eda7830bace1466cab17537420b1d38609fb3e`,
summary `19ecece04965fbe979e3164ff7d1a3611240cf75e2bf0754fa3e2e9c2172e2a2`.

The first post-predicate run `run-wnyegl_j` / remote `run-W57jmR` passes all
lineage/receipt assertions and reaches the second real delegation, but the
new test incorrectly expected `(3, true)`. `ImeManualToggleOutcome::as_v3`
defines exact-tail delegation as `(3, false)` independently of current layout;
layout is asserted separately by `VisibleTailV3`. Correct only that fixture
expectation to the typed enum. The run remains 542/543, not GREEN, and the
remaining positive matrix rows have not yet executed. All negative controls
and previous identities pass; the production predicate stays unchanged.
Evidence `/home/ubu/.cache/lay/development/run-wnyegl_j/`;
`RESULT.json` SHA-256 `5ea75a79666a4090ead2445a2cad231937c565320809eb0111125f73dc1b06cb`,
archive `33024dee2832b8336933b81fb9488b6e13a86a779fd48ba93b6a65ac70165e8d`,
summary `1737f11a068bc7183d871d3ea5306697f0a7a3948d2f891dab2a9d463a9071c7`.

### Replay append focused GREEN — 2026-09-13

`run-xmk6pjcw` / remote `run-3dy2dx` discovers 546 tests and executes/passes
exactly all 543 selected identities, retaining the same three performance
exclusions. The three positive Reset/replay schedules and all eight negative
controls pass through their final assertions. All prior 541 identities remain;
all 705 Rust/Cargo/generated-receipt rows match the tested source archive.
No new runtime state was added. The existing rereceipt append predicate now
accepts only a handled append or a quarantined native append whose existing
local/shared replay scope is still current, followed by the old exact append
and identity checks. Native and physical acceptance remain open.

Evidence `/home/ubu/.cache/lay/development/run-xmk6pjcw/`;
`RESULT.json` SHA-256 `2de3f4acac7a54eec5c52885b1f92eac7f5b96973a536890342cc31b07974506`,
archive `edf41ed6f7ebc465dee796b99ae63290e5a57a4a160a0cef0d2625e43f0c69fb`,
summary `3fb1a7e85a484bc998ba989a3c511afc8191eb06caab67bcba6fbad9509cdd32`.
Freeze the next candidate after the canonical graph refresh and retain all
four native input cases. A pending/unpublished completion and an ordinary
Alt-induced IBus focus transition must remain distinct from a successful
completion followed by a manual projection; no ordinary Alt shortcut is
consumed to satisfy the harness. No installation or release authority changes.

### Replay-rereceipt native gate — 2026-09-13

Canonical guarded architecture/build and all 766 freeze rows pass. Candidate
IME SHA-256 is `a848dec1ceb1f05f93410bcd7a2314dab25b2ed375aa3ae86c12a0e33418909f`;
daemon and sender match the preceding freeze. Evidence directory:
`/home/ubu/.cache/lay/development/td121-firefox-replay-rereceipt-20260913/`,
binding SHA-256 `7256d8c072c13d2561509c2e3cbc50a072289f1a5ebf9bbf8592c64520404c59`.
Native receipt is **1/4 harness PASS**: one pair passes; four taps still produce
one of two required projections. The original Alt capture is focus-cancelled
after an actual completion acceptance and later ordinary Alt/IBus refocus; the
single-Alt capture loses DOM focus before any key reaches the IME and is
**not an autocomplete behavior observation**. Both have real
`CANCELLED_FOCUS_LOSS` events, distinct from the previous guard-close timeouts.
Installed C20 bytes are restored and original browser/global IBus identities
are unchanged. No physical/release PASS or third scored review is claimed.

The valid repeat trace now reaches a final Reset with all six characters in
its pending rereceipt (trace row 420), but no subsequent `SetSurroundingText`
callback arrives during the queued wait. Hence exact-tail readout correctly
continues to refuse. This schedule has no mid-insertion Reset, so the new
append predicate's native causal effect is not measured in this capture.
The preceding controlled three-schedule proof remains valid. Retaining
lineage alone cannot authorize a missing client snapshot. Investigate the
actual GTK/IBus/Firefox snapshot request and caching contract before another
runtime patch; do not lengthen the wait, manufacture a snapshot, add a probe
keystroke or relax the two exact-tail validations.

Receipt:
`/home/ubu/.cache/lay/development/td121-firefox-fast-repeat-20260913-t__qoo7d/firefox-replay-rereceipt-baseline/RECEIPT.json`,
SHA-256 `e2b019f487a138ec42208dca5737a5352880f414df8ee182fb43d1e2b432931d`.
Repeat trace SHA-256:
`0484e6ff46d44537e33d0265415796c2679750b214f8102b3dffb8c61975d567`.

### Queued Shift/client progress preflight — 2026-09-13

The pinned Firefox 155.0.1 source at revision
`5fdfd0092780e85643e2cddc0e1b590c8b9ef860`,
[IMContextWrapper.cpp](https://hg.mozilla.org/releases/mozilla-release/raw-file/5fdfd0092780e85643e2cddc0e1b590c8b9ef860/widget/gtk/IMContextWrapper.cpp),
updates selection and may call `ResetIME` without retrieving surrounding text
again. Retrieval is retried only for the existing pending-retrieval condition.
The upstream [IBus GTK module](https://github.com/ibus/ibus/blob/main/client/gtk2/ibusimcontext.c)
requests surrounding text before keys; its RequireSurroundingText callback
disconnects after its first request. This IBus reference is not the exact
client binary revision: Firefox maps the Snap GTK/IBus libraries, not the host
dpkg libraries. Source URLs, hashes and full files are retained in
`/home/ubu/.cache/lay/development/td121-firefox-replay-rereceipt-20260913/upstream/`.
The native trace above establishes the missing final callback; these sources
explain a possible ordering mechanism, not a measured fix.

Hypothesis: emitting both already captured Shift taps before Firefox's
selection update consumes both opportunities to request the completed text.
Split only these two genuine taps. Emit the first closed press/release frame,
then use the existing settlement loop to forward the second once either the
exact tail is already authoritative (ordinary GTK progress) or the existing
authenticated Reset follows a completed exact replay. Continue to require an
actual exact client snapshot before ManualToggleV3 and both edit leases.
No extra key, held modifier, gesture detector, timer, queue, owner or retry is
added. The current queued-loop constant is **80 ms**, unchanged; earlier
mentions of 500 ms in investigation notes are not its source contract. D-Bus
call duration is separate from that loop deadline.

Expose the Reset/completed-replay condition only as an empty passive bridge
status. Derive it from current local/shared replay scope, exact completed epoch
distance, current layout/path/owner, current Reset predecessor and a zero
post-Reset observed suffix. An earlier Reset followed by native appends must
not satisfy that last condition. This status cannot supply a text source,
KnownStart, learning, suppression or mutation authority. The only additional
consumer forwards the second already captured Shift tap; on refusal/error the
remaining tap is forwarded once and mutation stops. An emission failure is
never retried. Ordinary input, atomic/terminal routes and the physical FSM are
unchanged. Remove the old all-at-once pair emitter when this split takes over.

Before changing behavior, exercise actual bridge entrypoints for complete
replay -> Reset -> missing snapshot (passive progress, no edit), early Reset,
partial replay, revoked scope, selection, layout and focus changes. A fresh
exact callback must remain necessary and sufficient for the existing exact
route. Extract the existing daemon settlement loop with injected read/emission
operations, initially retaining its old both-taps-first order, and prove that
order fails a controlled client-progress schedule. New daemon checks must also
cover ordinary GTK completion, stale snapshots, exactly two closed taps,
timeout/read/emission failures and no second mutation. Run the focused IME and
daemon sets remotely with exact identity selection, then freeze/build and run
the same four owned Firefox scenarios. Native liveness remains unproved until
that capture; reject the hypothesis if the client still supplies no exact
snapshot. No third scored review or installation is authorized by this
experiment; the two formal review passes remain exhausted.

### Queued client progress causal RED — 2026-09-13

`run-iz9ctton` / remote `run-ug0Axe` discovers 815 tests, selects/executes
810 (265 daemon, 545 IME), and passes 808 with exactly two expected failures.
The real daemon settlement loop, extracted behind injected transport/clock
operations while retaining both-taps-first ordering, emits the second frame
before the first client read (observed read count 0, required 2). The real
IME bridge reports only unknown context after complete native replay and a
new Reset; the expected empty passive progress status is absent. The negative
matrix and all prior IME identities pass. Five performance exclusions remain
separate. Three former daemon helper test identities are replaced by the FIFO
test and the two settlement-loop tests; the removed pair emitter's release
failure contract remains covered by the existing closed-tap frame tests.

Evidence `/home/ubu/.cache/lay/development/run-iz9ctton/`;
`RESULT.json` SHA-256 `b9d8594b33894e109b634dc3b84c70c1f393c9aedc03868a0b09839944b51755`,
archive `ce43a02f19717ee86ec1a7df443428e368cfbf028a24398ad1f24c35123cb0d3`,
summary `e8d246ca3b38ecd16998c394bbc5f68a8ea7339251c7538c4138e3108f409b9e`.
This is a controlled-order RED, not another native run or a new text authority.

The first post-change run `run-wchb064l` / remote `run-cDxz9w` executes all
811 selected identities and passes 810. All 266 daemon checks and all new
IME controls pass, including completed replay, early Reset, fresh-snapshot
requirement and eight invalidation cases (now including expiry). The only
failure is the older coarse-Reset fixture's exact `passive:unknown-context`
string assertion. It reaches the same now-explicit empty progress condition.
Update that expected passive status and strengthen the existing fixture with
an unmapped VisibleTailSource assertion, empty focus receipt and denied exact
mutation predicate. Its subsequent fresh-snapshot and exact-delegation
assertions remain unchanged. No production change follows this run; reuse the
266 daemon proof and rerun the IME component for this test-only correction.

Evidence `/home/ubu/.cache/lay/development/run-wchb064l/`;
`RESULT.json` SHA-256 `78113f19a1d1324c96225a5011bf1743ecd97200a819866660f64aa682bb7ad4`,
archive `f29122b2ca9a95a13f3ff51fb079666efb3dadb2a613ce651f2025f6cd3ad17d`,
summary `94ef3bb3b7e9fdbb5da5294a0718f2f25f715294012f7a244952bd404fa9949e`.

### Queued client progress focused GREEN — 2026-09-13

`run-08re75wd` / remote `run-UDb4U9` discovers 548 IME tests and passes
exactly all 545 selected/executed identities, with three performance exclusions.
All 730 Rust/Cargo/generated-receipt rows in this archive match the current
checkout. The prior run's 266 daemon identities also match selection/execution
exactly and have no failures; its only changed Rust row is the IME-only
coarse-Reset test expectation, so that daemon proof is reused. Together these
establish the 811 checks at this source stage, not native or physical acceptance.
The old status assertion was updated without changing any runtime byte after
the 266-check daemon PASS.

The new passive status requires unexpired matched replay scopes, the exact
completed epoch distance, current owner/layout/path, an authenticated Reset
predecessor and zero post-Reset observed suffix. Both empty text and empty focus
receipt remain mandatory; every edit parser rejects this status. The original
80 ms queued deadline and 700 ms replay-scope expiry are unchanged. The second
captured tap is emitted once on client progress or existing exact-tail proof;
missing snapshot, repeated progress, read/emit failure and timeout cannot
authorize a second mutation. No new long-lived state or runtime authority.

Evidence `/home/ubu/.cache/lay/development/run-08re75wd/`;
`RESULT.json` SHA-256 `4c3860e9a71d969d9183200c97b7bdba8269eac271ba0f2e11a2d215336f11b4`,
archive `5bbb1b2a6ea9fe0610795e580152f4da6a8e9224e87afb3b95746f40de3f54ae`,
summary `50000cee9208f9b9f3fdfe80faf5e4117a9709bdc8f7ea68ea29312be87e7b1c`.
Next freeze/build uses the canonical guarded architecture refresh and preserves
the same four native Firefox inputs and focus-cancel harness. Its verdict is
still open; this source is not installed.

### Queued client progress native rejection — 2026-09-13

The canonical architecture/build freeze passes all 766 rows. Candidate IME
SHA-256 `0646402d3866f1e1dfc29dd47521751f30465d37e32e8f55e3f26ebaaa9a4144`,
daemon `d05e22eb4e15b814a3d97c84aa04254135a31e8d5288432fab04f148cea4a2e1`,
sender remains `cd6c89bb1beab99a8e6443c0e40051196e753571eaad042d564f90ea1a7f81ae`.
Freeze evidence:
`/home/ubu/.cache/lay/development/td121-firefox-queued-client-progress-20260913/`;
binding SHA-256 `aca5772c1a847de8b9b53cbdd8cb8e746d05f26f21f63c51db4f4fadced23c26`.

The native result is still **1/4 harness PASS**. One fast pair passes. Four taps
still produce only `привет`, one of two required projections. The second pair
now demonstrably straddles Reset: first tap callbacks are trace rows 402–414,
Reset 417–420, stale one-character snapshot 423–424, second tap 425–437, then
another Reset 451–454 and no fresh full snapshot. Thus Reset after the local
mirror completes can still be intermediate Firefox client progress. The new
passive status was emitted, but it cannot solve native liveness. **Reject and
remove this split/status implementation**, retaining its sealed evidence.
Do not increase the wait or forward further probe keys.

Single Alt now has a valid focused capture: one completion acceptance and one
manual delegation occur, but native replay produces `ghjdth f`, not `ghjdthrf`.
The original double-Alt case accepts completion then undergoes IBus refocus;
its harness stops with `CANCELLED_GUARD_CLOSED`, not observed DOM focus loss.
These are separate verdicts. Installed C20 bytes were restored, and original
Firefox/global IBus PID/start identities remained unchanged.

The single-Alt trace establishes a prior transport defect: ProcessKeyEvent
serial 125 enters the observer before 127 (rows 415/417), but handler 127 runs
before 125 (432/437). The expected `h` is therefore overtaken by `r`; the exact
scope rejects `r`, then ordinary managed commits emit the late `h`. Serial 131
(space) likewise executes before 129 (`f`) at 455/461. Several Reset/key stamp
waits expire. This is measured handler reordering, not an inference from DOM
log arrival times: the HTTP DOM recorder's arrival order is not DOM dispatch
order. The fixed 1 ms replay pacing is unchanged.

Receipt:
`/home/ubu/.cache/lay/development/td121-firefox-fast-repeat-20260913-t__qoo7d/firefox-queued-client-progress-baseline/RECEIPT.json`,
SHA-256 `ddb7932777fdedf9fd7e24b22132b14cc41a2aee0061de0b530df1fba65a04f0`.
Repeat trace SHA-256 `76d73a0607799d38afdb66b671d591d0ddef665be70288b816e1a09ad6f7d90c`;
single-Alt trace `7451f904d88ecb751f8243cc4d20bfa6f9b4a2febcde279d1387d3ea73eebd7f`.
The next investigation is the existing zbus interface's default per-method
task spawning and its ordered-dispatch option, including reentrancy/deadlock
consequences. It must not introduce another runtime queue or increase any
admission deadline. No native/physical/release acceptance or scored review.

The rejected experiment is removed by a scoped inverse diff over eleven files.
Each prior file was verified against the replay-rereceipt freeze manifest and
the sealed `run-xmk6pjcw` archive before applying that delta; all eleven hashes
match afterward. Earlier Shift-footprint, Reset append and other accepted
repairs remain intact. Patch and before/after bindings are retained as
`reject-split-delta.patch` and `reject-split-bindings.json` in the rejected
candidate directory. Documentation and original native evidence are retained.

### IBus dispatch ordering preflight — 2026-09-13

The pinned local dependency is zbus/zbus_macros 5.15.0. Its interface macro
defaults to a separately spawned task per method; its own documented contract
explicitly permits handling calls out of receive order. `spawn = false` uses
the existing object-server dispatch loop in receive order. The current Engine
interface leaves the default enabled. The single-Alt trace above demonstrates
the resulting word mutation failure under backlog. No new queue or manual
serial counter is needed if this existing dispatcher contract suffices.

Consequence boundary: serial Engine dispatch also delays later methods on its
IBus connection until a handler returns. Preserve the independently spawned
ContextAdmissionObserver: callback-stamp rendezvous and activation markers
must still progress while a handler is suspended. Focus callbacks initiate
activation rather than wait for the object server to dispatch a later marker;
output calls emit signals. Current inline manual layout sync calls GNOME on
the separate session bus, whose extension returns after requesting activation;
both constructed blocking atomic actions select that GNOME route. The unused
`activate_gnome=false` executor branch is not a currently constructed action.
Do not introduce a synchronous call that requires a later Engine/Factory
handler on this same connection. Factory construction, atomic/terminal paths,
revocation, bridge liveness and real native captures remain required checks;
source inspection alone is not a deadlock/liveness proof.

Controlled proof: register the actual Engine object, hold its existing write
guard, submit a bounded burst of real authenticated key methods in known wire
order and drain the independent ingress observer. Release the guard and check
actual reply order, exact resulting tail, unchanged text-effect ownership and
empty callback backlog. No sleeps, new dispatcher model or retry loop. Run it
against the current default first. Then, if that demonstrates the failure,
change only the Engine interface dispatch option and rerun the complete IME
set, including existing real marker/factory and atomic/terminal contracts.
Native repeat/fresh-snapshot behavior remains unresolved until measured again.

The first backlog-proof setup run `run-7b49zgms` / remote `run-oM82Hw` passes
543/544 selected checks but receives an Error reply before reaching the order
assertion. The fixture inferred its keycode literals as signed integers, so its
D-Bus body did not have the required `(u32,u32,u32)` schema. Give the event
vector that explicit wire type and include error name/serial in any future
diagnostic. This run is not a causal ordering RED and contains no production
dispatch change. Evidence `/home/ubu/.cache/lay/development/run-7b49zgms/`;
`RESULT.json` SHA-256 `9b58bf100a5a0ca3b4801b3b0c847dc4db7f3241d7644fdbc411b178cb13b462`,
archive `8997cd8c4d44707b9289cd1ad9d6800bbb2e94140b0c0ef0f44d820ad3f706a5`,
summary `4d92acd106e6dc847f6be09afc4d91bbdfb7b2904d7bed111b4cdeb9600aa354`.

### IBus actual-dispatch causal RED — 2026-09-13

With the corrected unsigned wire body, `run-tt11wauw` / remote `run-j7VtYt`
discovers 547 tests and passes 543/544 selected/executed identities. Only the
new actual-object-server order check fails: reply order starts `24000, 24047,
24001, ...`, so the final release overtakes 46 preceding methods. The resulting
tail happens to remain `приветпривет` and IME text effects are empty in this
controlled schedule; this is an order violation, not a unit reproduction of
native letter corruption. The native single-Alt trace separately proves that
printable methods can overtake and corrupt output. All 543 earlier identities
pass. No dispatch production change exists in this RED archive.

Evidence `/home/ubu/.cache/lay/development/run-tt11wauw/`;
`RESULT.json` SHA-256 `510a866477990aa9a84cfd836c4f247efc715e4a605f5c40ea32a4596c8e171a`,
archive `60b93423c7f781a63b1b98ff43e679369f1a0264d32be2b99de31cc3243dca37`,
summary `52879a7a101eb80a9186d41b817e7e0de48e99ebe1e96e7dff74fddad2e5c9d3`.
The next production change is the single existing Engine-interface `spawn`
option. The rejected passive status/split remains removed; all safety and
fresh-snapshot requirements remain unchanged.

### IBus ordered-dispatch focused GREEN — 2026-09-13

`run-i44w58wy` / remote `run-LA7zLD` discovers 547 tests and executes/passes
all 544 selected identities, with the same three performance exclusions.
Selected/executed identities agree exactly; all 730 Rust/Cargo/generated-receipt
rows match current source. The actual registered Engine now replies to all
48 queued methods in wire order, keeps every native replay response unhandled,
emits no IME text effect, restores the exact projected tail and leaves no
unsettled callbacks or learning state. All existing factory, marker, bridge,
atomic and terminal unit contracts pass. No new runtime queue, timer, authority
or relaxed validation is present. Native/client liveness is still a separate
gate; this is not physical acceptance.

Evidence `/home/ubu/.cache/lay/development/run-i44w58wy/`;
`RESULT.json` SHA-256 `dd85b500a9193d5d6071d7b16184c4b994104639fae1007a48d99629e0e20127`,
archive `ba74e48209c4d9ca1c19539e2955abbf9dbd25537b80fd1c783a735e3457ed23`,
summary `d41daeb5b5f40965b9932a96ad9c1958e74650fb7e5146f15328e54f5a380bec`.

The native runner's top-level `ibus_sync_mode=1` is not the Firefox child
environment. The pinned `firefox-entry-capture-focus-guard-user-env.py` removes
IBUS_ENABLE_SYNC_MODE and verifies that absence in the actual owned browser,
matching the user's unchanged Firefox process. This helper was already used
in the prior captures. No new sync-mode variant or environment change is
needed; preserve the same four cases and pinned focus-cancel harness for the
ordered-dispatch freeze.

### IBus ordered-dispatch native result — 2026-09-13

The canonical architecture/build freeze passes 766 rows. Candidate IME SHA-256
`8bfe5dd39d329c06de2bbfa12e088e84fb2e84b21899143a4efd7e5348bbca24`;
daemon and sender return to the preceding accepted replay-rereceipt bytes
(`22193b24914cf306d531e2733dcdafcf286480206c1dfb09ae7f8e4048c5edbd`
and `cd6c89bb1beab99a8e6443c0e40051196e753571eaad042d564f90ea1a7f81ae`).
Freeze directory:
`/home/ubu/.cache/lay/development/td121-firefox-ordered-dispatch-20260913/`;
binding SHA-256 `7d0692d24ed4d5c46017d88364c31edc02638f115f11721811ee0161e63531d2`.

Native result is **2/4 harness PASS**. One fast pair produces `привет`.
Single Alt accepts completion once and manual projection once, with exact
`ghjdthrf` final text: the preceding corrupted replay is now correct. Across
all four captures, all 146 observed legacy key callbacks enter in wire order
(44/40/14/48), and callback-stamp timeouts are zero. This is native evidence
for the dispatch repair, not a general latency/quality claim.

Four taps still produce only `привет`, one of two required projections. Its
final Reset arms the full six-character rereceipt at epoch 22 (rows 439–441),
after all four forwarded Shift callbacks, but no subsequent client snapshot
arrives. Keep this remaining defect open. The original double-Alt case completes
with `про`, no accepted completion and no manual projection; it is a valid
failure in this capture, distinct from older cancelled/accepted-prefix cases.
No expectations or inputs changed. Installed C20 bytes are restored; original
browser/global IBus identities remain unchanged. No physical/release PASS.

Receipt:
`/home/ubu/.cache/lay/development/td121-firefox-fast-repeat-20260913-t__qoo7d/firefox-ordered-dispatch-baseline/RECEIPT.json`,
SHA-256 `1bbc70286222f6eb8048094ff534206363f3b5ff032f5c518e4ef730d77605a6`.
Repeat trace SHA-256 `281f69046d886aae3c4823d6ebeb913a69da332f0061ae088ec0bac3d379bd29`;
single-Alt trace `9eb576373dd7d44531686056f44a147c434c261dadb3540dcde388d0abee3103`.
Retain the ordered-dispatch fix. Any next snapshot-contract investigation must
remain separate from this proved ordering repair, without extra probe keys,
fabricated client text, longer waits or weakened edit leases.

### Firefox client module control preflight — 2026-09-13

The actual user's Firefox maps `im-ibus.so`; its local desktop launcher also
explicitly selects `GTK_IM_MODULE=ibus`. The same Snap runtime contains
`im-wayland.so`. Upstream GTK 3.24.32 `imwayland.c` connects reset to
`notify_external_change`, which emits `retrieve-surrounding`; IBus 1.5.29
`ibusimcontext.c` sends Reset without that request and disconnects the
RequireSurroundingText callback after its first invocation. This establishes a
protocol difference, not native compatibility or exact binary source identity.
Pinned primary source URLs, hashes and downloaded files are retained under
`/home/ubu/.cache/lay/development/td121-firefox-wayland-control-20260913/`.

Run a separate, explicitly labelled **client module control**, reusing the
ordered-dispatch frozen binaries and the unchanged four cases/expectations.
Copy the exact pinned focus-cancel helper chain into that evidence directory;
change only the test browser's `GTK_IM_MODULE` to `wayland`. Require actual
child environment and loaded `im-wayland.so` evidence before sending input.
Keep unique profile/title/PID/start/focus guards, unchanged cancellation,
sender/daemon supervision and installed-byte restoration. The original user
browser, launchers, global IBus and input sources are not changed. This control
is not a replacement PASS for the original IBus client route.

Consequences: no production code, candidate/lattice, ranking, package, cache,
learning or feedback changes. Native compositor transport may alter focus,
capability, preedit, key scheduling and layout-handoff behavior; the four fixed
cases and trace must expose these differences without giving them authority.
No new runtime owner, queue, timer, polling, probe key or edit fallback. CPU,
RSS and latency claims remain untested; the existing bounded test deadlines
remain. A new browser route can fail admission or lose composition, so preserve
all failures and stop the owned input on focus loss. Removal is deleting only
the inactive test wrapper; no product configuration rollback is required.

Design comparison: the original IBus route with repeated Require alone cannot
establish notification liveness because its callback has disconnected. Raising
the wait or treating the replay mirror as a client receipt cannot supply the
missing evidence. The already available native GTK Wayland client is a viable
independent discriminator because its reset contract actually requests the
text; whether its compositor bridge preserves Lay's contracts must be measured
before proposing any user route change. A client/library patch would require a
separate build, deployment and maintenance boundary and is not selected here.

### Firefox Wayland module control result — 2026-09-13

The same frozen ordered-dispatch binaries give **1/4 harness PASS**. Four fast
Shift taps now pass with two exact projections, `ghbdtn -> привет -> ghbdtn`,
21 surrounding callbacks and two manual delegations. The owned browser's
environment and mapped `im-wayland.so` were checked before input. This is a
native witness for the missing client-notification distinction, not acceptance
of a new default route. The fast single-pair cell is invalid before input:
readiness raised JSONDecodeError and the helper recorded a focus-identity
mismatch; no keys/commits occurred. Preserve that harness failure separately.

The double-Alt cell is valid and fails with `про`, zero completion acceptance
and zero manual projection. The single-Alt cell times out without Enter reaching
the textarea; its last observed focused value is `про`, zero acceptance and
zero manual projection. Both traces have three managed letter commits but no
preedit or precognition worker. Surrounding callbacks lag by a letter; unlike
the IBus client, these managed commits do not produce per-letter Reset. Alt
then passes through, producing menu-related Reset/refocus. Investigation must
locate why the existing observed-suffix preparation is not eligible here; do
not relax completion or exact edit authority to force this control green.

Receipt:
`/home/ubu/.cache/lay/development/td121-firefox-wayland-control-20260913/native-control/RECEIPT.json`,
SHA-256 `a3c2c4e036ebf2c30296f9f42563389a7a5793a5af5b6cd1c4f921ee5b4611fc`;
artifact binding `da881ba73f83e401d3714fadcbaa3c9cc7f4e6484d0cf7ca8fbe0826223f0d5b`.
Repeat trace `a77adc8e79b9b1a3342c2458e55caecd6ea31448b8e5131b6070bc3299d7f499`;
double-Alt trace `cf3cf4cb5eb0552bc7fd7b9c1e4f1c3de36406dafb78fd5f3ec6cac9cd7ec4d7`;
single-Alt trace `ca7af922997fd0f2996615c3eeb90ab83cc9cffcd85d106046ab6da1e0c5dd0c`.
Original browser/global IBus identities survived and installed C20 bytes were
restored. No user launcher, input source, client library or product authority
changed. Original IBus native 2/4 remains its own unchanged result.

### IBus client scheduling control preflight — 2026-09-13

Before changing suffix preparation or adopting another client transport, measure
the existing IBus client's supported synchronous mode on the same four cases
and frozen bytes. The GTK 1.5.29 source distinguishes asynchronous event
reinsertion from synchronous ProcessKeyEvent; the former sends unhandled keys
back through the GDK queue. The control keeps `GTK_IM_MODULE=ibus` and changes
only the owned browser's `IBUS_ENABLE_SYNC_MODE=1`, with actual environment
validation. The user browser remains asynchronous and unchanged. Preserve the
same guards, inputs, expectations and separate invalid-capture denominator.

This changes test-only client scheduling and Reset/preedit handling, not Lay
authority or deadlines. Synchronous calls can block the test browser while an
IME method runs; the existing case deadline and focus-cancel supervision bound
the experiment. It cannot itself prove general latency or notification liveness.
No production code, ranking, memory/cache identity, package, learning, owner,
fallback or launcher change. Compare measured results with the sealed original
IBus and Wayland results; never reinterpret a successful control as acceptance
of the user's unchanged route. A subsequent production proposal requires its
own consequences and proof. No retries-until-green or extra probe keys.

### IBus synchronous control result — 2026-09-13

The synchronous client control is **1/4 harness PASS** and is not a fix for the
repeat failure. Its valid four-tap capture still ends at `привет` with only one
projection and a queued exact-tail timeout. Valid single Alt produces exact
`ghjdthrf` with one acceptance and one projection. The fast-pair cell fails its
owned-focus check before any input; the runner then reads an empty readiness
pipe and reports JSONDecodeError. The double-Alt cell accepts a completion but
loses the IME word across menu-related refocus and times out without a final
textarea Enter. Neither invalid capture is a runtime PASS or a physical proof.

Receipt:
`/home/ubu/.cache/lay/development/td121-firefox-sync-control-20260913/native-control/RECEIPT.json`,
SHA-256 `da5705dc9ea979072eafc217d3c2fe2ee2bfbc5b7171e1fa02ebf1f5da043de0`;
binding `d9d3aebbbfc4ad109f1705ee49572f317f5be63c39e975937361f76756d38208`.
Repeat trace `b1725e8f9ea275959edd8b0e028f6c28383bac8e43ad0acbb7677950efd5f7c4`;
single-Alt trace `d943420bfc0ebce65aeb42324d8d0a603d276d01c226930f1a80af61ccda8e58`.
Original browser/global IBus identities and installed C20 restoration pass.
Keep the original browser configuration; a sync-mode override cannot resolve
the measured missing-client-snapshot mechanism. No production change.

### Minimal client Reset notification discriminator — 2026-09-13

User steering: preserve useful work, finish the Firefox repair and reduce time,
tokens and unrelated rewriting. Freeze the current Lay runtime source. The next
bounded experiment changes only the owned test client's notification: after a
real GTK multicontext Reset on the IBus module, emit one GTK
`retrieve-surrounding` request. Firefox already updates its parent selection
cache before Reset; its existing retrieval handler supplies text or arms its
own pending child-cache retry. Lay receives only the real resulting snapshot.

Use an inactive, remotely compiled interposer in the fresh test browser. A
thread-local depth guard prevents requests from nested resets; preserve the
real Reset call and all return behavior. Restrict to the IBus multicontext,
record only bounded request/result metadata, and verify the exact library map
and child environment before sending the four unchanged cases. No fabricated
snapshot, probe key, polling, delay, edit permission, selection access outside
the owned test field or change to the user browser/launcher/global IBus.

Compared designs: Lay-side inferred replay completion cannot replace a client
receipt; synchronous IBus was measured insufficient. Wayland supplies the
needed replay snapshots but its completion route remains unproved. Restoring
the actual client's reset notification is the smallest remaining causal
discriminator. This is not an installation design or production acceptance.

Consequences: no Lay candidate/ranking/package/cache/learning changes or new
runtime owner. The test client's extra signal can reenter GTK; retain the depth
guard through signal emission and preserve existing focus cancellation. CPU
and allocation cost is one dynamic symbol lookup group and signal per real
outer Reset, without a timer. No latency/resource claim. Missing symbols or
library mapping fail the owned test before input. Source/build/helper hashes,
four case outcomes and original-process restoration remain separate proof
denominators. Remove only this test adapter after the experiment; production
deployment, compatibility and maintenance require an explicit subsequent plan.

### Client notification discriminator result and real Firefox trace — 2026-09-13

The first four-case Reset-notification batch is not a four-case correctness
denominator: the four-tap case cancelled at the DOM focus guard before input,
and the Alt cases contain interleaved manual input. The first single-pair case
passed with the actual client notification. Do not classify the mixed records
as runtime failures or use their extra acceptances as a new runtime defect.
Batch receipt:
`/home/ubu/.cache/lay/development/td121-firefox-reset-notify-control-20260913/native-control/RECEIPT.json`
SHA-256 `1e8d807e78c577beb854d419128d3c6374104c8bb508eae312a7b86a2f64f0a7`.

The separately guarded four-tap discriminator passed 1/1: exact visible
`ghbdtn -> привет -> ghbdtn`, two manual delegations, and 15 actual GTK
retrieve requests all reporting supplied data. Final Reset at trace row 502
was followed by the real six-character snapshot at row 505 and second manual
delegation at row 509. This reused unchanged ordered-dispatch Lay binaries,
input and expectation; only the owned test Firefox loaded the client probe.
Receipt:
`/home/ubu/.cache/lay/development/td121-firefox-reset-notify-control-20260913/repeat-only/native-control/RECEIPT.json`
SHA-256 `d099310e9840c26457b6bc66cb0843fce8b6e4b66d2442488a61d262ee4e86ed`;
trace SHA-256 `987950ebd3f10f8ba3cf704aa14252e8a3205f17bc842c023e59f7993611406e`.
Verdict: causal notification mechanism demonstrated in the private test client;
ordinary Firefox, physical keyboard and release acceptance remain unproved.
Production runtime authority and installed C20 bytes did not change.

The user's subsequent report is specifically DoubleShift failure in the
ordinary Firefox; ordinary typing and DoubleShift in other applications work.
The active next step is a bounded trace of that installed process, not another
automated browser run. The passive observer in
`/home/ubu/.cache/lay/development/td121-live-firefox-trace-20260913/`
records physical Shift transitions, selected session bridge calls/replies and
IBus callback metadata. It never grabs input, sends keys, restarts services or
records the text of D-Bus payloads. A read-only Ping/GetGlobalEngine control
confirmed both bus observers receive calls and correlated replies. Physical
attempt pending at this checkpoint. Installed debug_action_log was already
true; internal diagnostics are in the existing private
`~/.local/share/lay/ibus_engine_debug.jsonl`, independently of stderr being
`/dev/null`. No configuration change was required.

The user then confirmed an attempt on “Проверка”. The external observer had
already reached its 600-second limit; do not assign the earlier ordinary Shift
holds to this reported gesture. The retained installed internal trace contains
14 Shift callbacks (seven presses/releases), serials 4791–4807, all accepted
with `unknown_start`, stable nine-character tail and no preedit. The preceding
Reset rejected rereceipt with `missing_predecessor_or_post_reset_token`.
This establishes Shift delivery and context state, not the daemon's gesture
recognition or bridge reply. Exact partial evidence and hash:
`~/.cache/lay/development/td121-live-firefox-trace-20260913/first-user-attempt-partial.json`.
The repeated capture `events-user-confirmation.jsonl` has no automatic deadline
and is stopped after the user's confirmation; one repeat was requested.
No runtime, configuration, release or physical-acceptance change.

### Actual physical Firefox refusal and bounded repair — 2026-09-14

The user clarified that IME completion was absent before the first Shift pair.
The uninterrupted installed-process trace is now sealed:
`~/.cache/lay/development/td121-live-firefox-trace-20260913/events-user-confirmation.jsonl`,
SHA-256 `a3c4325065ef18bcfd534fcd27caa094a9d8caa00b131dcb2aa7f61cbf67e5f9`.
Eight initial character commits produced eight Firefox Resets and snapshots,
but zero preedit updates. The first two manual requests delegated exact
eight-character tails and both replayed eight deletes/eight characters. After
further typing, four recognized pairs received `(0, false)` in 0.417–1.374 ms.
Later three terminal requests succeeded; they are a separate client denominator.
No D-Bus errors or process restart. The observer has been stopped.

The earlier retained physical attempt proves the boundary transition exactly:
Space serial 4787 committed the ninth character and settled `known_start`;
Reset 4788 rejected its predecessor; the real nine-character snapshot arrived,
but fourteen subsequent Shift callbacks remained `unknown_start`. Installed
C20 and current source both explicitly exclude KnownStart predecessors and
trailing whitespace from Reset rereceipts. A fresh client notification alone
cannot fix this case: the real snapshot is already present.

User priority: finish faster. One grouped regression now covers an already
known word and an actually observed closing boundary (Space/accepted append),
through the real callback/Reset/snapshot/bridge paths. Establish RED on current
source before changing runtime; then repair this existing proof transfer only.
Keep the separate missing-client-notification experiment separate.

Consequence analysis: retain an exact previously proved suffix in the existing
Reset receipt, including its actual closing boundary, and require the same
authenticated owner, fresh post-Reset token, epoch and exact client snapshot.
Generic word authority remains UnknownStart. No ranking, lattice, model,
package reload, learning, generic verifier or SafetyGate change. Completed
boundaries must never publish a completion for an empty next word. Missing,
selected, changed, stale-owner and stale-epoch snapshots still refuse effects.
An alternative completed-tail cache would duplicate this receipt's lifecycle;
keeping a generic KnownStart across Reset would retain revoked edit authority.
Extending the existing exact-only receipt has the smaller removal/maintenance
boundary. Capture boundary provenance only for an actual owned append, with
bounded strings and no new RPC, polling, queue, deadline or service. Preserve
ordered callback settlement; an intervening edit/revocation invalidates the
candidate. Roll back this scoped source diff if the grouped proof or affected
contracts regress. Focused checks, native Firefox behavior, full release gates
and physical acceptance remain separate; none is implied by the captured trace.

The grouped callback regression reproduced all three losses on the prior
runtime (`run-7uyk82bi`: 544 pass, one grouped failure). The existing receipt
now carries a proved known word or an actually committed closing boundary;
an unconfirmed prefix and an expired replay lease cannot seed this transfer.
After Reset, generic word authority remains unknown and the candidate remains
inert until the exact fresh client snapshot. Closed tails cannot publish a
completion for the empty next word. Two prior tests now check these semantic
effects instead of requiring the internal candidate Option to be empty.

Final focused result: **545/545 PASS**, including all three grouped cases and
the stale-owner, missing snapshot, wrong selection and quarantine contracts.
Receipt: `/home/ubu/.cache/lay/development/run-72tfxnuq/RESULT.json`;
remote logs: `/home/e/projects/lay-development-runner/run-eO490o/tests/`.
The first repair iteration exposed four regressions; all four were corrected
before this result. No new runtime installation or physical acceptance.
Next: freeze the graph and exact binaries, then run the owned Firefox cases
for one pair, immediate repeat, accepted completion and a closing Space.

The frozen boundary build (`td121-firefox-boundary-receipt-20260914`, IME
`b14e71177d56def2d068e47220928643779d8e1a31ef68bcf29e10091d1cc531`)
passes native single-Alt acceptance and exact manual projection. The original
Space case now delegates and projects `ntrcn ` to `текст `, but its original
expected result `ntrcn` is unmet: the preceding autocorrection did not occur.
The repeat still ends at `привет` with one delegation and no final fresh client
snapshot. The fast-pair cell fails readiness before input and is invalid.
Batch receipt:
`~/.cache/lay/development/td121-firefox-fast-repeat-20260913-t__qoo7d/firefox-boundary-receipt-20260914-baseline/RECEIPT.json`.
Original browser/global IBus were preserved and installed C20 was restored.
The all-target clippy preflight separately rejects the earlier eight-argument
diagnostic helper; no lint baseline is changed.

Combine the already measured Reset-notification client adapter with these
exact frozen boundary binaries in one owned four-case check. Reuse its pinned
library, focus cancellation and identity verification without new runtime
logic, altered inputs, expectations or deadlines. This tests the interaction
of the two separately demonstrated defects. It remains an inactive client
control; shipping a Firefox compatibility adapter would require a durable
installation/removal boundary and a browser restart, neither performed here.

The combined check gives **3/4 PASS**: one Shift pair, immediate repeat (two
delegations and exact round trip), and accepted completion followed by one
projection all pass. The unchanged `ime_ntrcn_space_shift_enter` still yields
`текст` instead of its autocorrection-then-undo expectation `ntrcn`. Its trace
shows five confirmed letter receipts, `managed_fallback_commit` on Space and
one exact projection; no autocorrection preceded it. Keep this failure visible.
Receipt: `~/.cache/lay/development/td121-firefox-boundary-receipt-20260914/notification-combined/native-control/RECEIPT.json`.
The original browser and global IBus are unchanged; installed C20 is restored.

The final manual-projection cell explicitly disables auto_replace and asserts
one toggle from `ntrcn ` to `текст ` using the same input script. This distinct
case proves the protected physical-key projection after a closing boundary;
it does not replace or promote the failing autocorrection scenario. The grouped
unit proof continues to cover exact boundary preservation and no empty-word
preedit. General correction quality remains outside this mechanical verdict.

Shipping boundary: retain the measured client adapter as
`scripts/compat/firefox-ibus-reset-notify.c` with the narrowly scoped
`scripts/compat/lay-firefox` Snap launcher. Install only for this user's Firefox
desktop entries after final-byte checks. The library stays in Snap's per-user
common data, with an exact hash and mapped-library check. Preserve the original
desktop file; removal restores its Exec lines and removes the two added files
after a normal browser restart. No session-wide preload or global IBus restart.
The ordinary browser remains untouched until the concrete restart step.

The C adapter is the already measured implementation, with a source comment
clarified. The client ABI dependency and removal condition are documented in
`scripts/compat/README.md`. It preserves the real Reset, requests only on the
outer IBus multicontext call, and retains the reentrancy guard through signal
emission. Missing original Reset fails explicitly; no guessed success or
fallback. At most 64 text-free diagnostics are printed per thread. No model,
learning, ranking or Lay edit-authority changes; no resource/latency claim.
The diagnostic lint repair groups the existing generation/nonce identity in
one parameter without changing trace fields or reducer behavior. Run the exact
manifest/affected/full release gates before the final native-byte check.

Final manifest discovery contains 2,867 identities: 35 additions and one removal
relative to installed C20, with no changed row metadata. The removed
`physical_input_grab::tests::queued_left_shift_pairs_remain_exact_inverse_gestures`
tested the deleted private `QueuedLeftShiftTaps` detector. Its contract is now
covered by the shared FSM partition/repeat tests, current-layout multi-pair
test and native two-toggle round trip. Six additions belong to lay-daemon and
29 to lay-ibus-engine. The zero-failure registry keeps the same empty failures
and observation identity; only its manifest hash is rebound. Exact delta:
`~/.cache/lay/development/td121-firefox-boundary-receipt-20260914/final-manifest-delta.json`.


Final-byte continuation, 2026-09-14: both complete 2,841-test runs passed;
the second full gate then rejected only stale lint source offsets. Rebinding
those existing warning locations preserved warning identity/count and passed
the unchanged full-gate suffix, strict lint, compiled receipt and release build.
Exact evidence: `~/.cache/lay/development/release-1.0.72-td121-firefox-20260914-finish/raw/RESULT.json`.
Overall acceptance is FAILED: combined-first-word loses FocusInId serial 298
at callback rendezvous after the store reaches capacity. An unrelated older
stamp eviction is checked before the retained exact header, so even publishing
the awaited stamp rejects the callback. Consequence: inspect exact retained
stamps after revocation/owner checks, before the conservative missing-stamp
eviction refusal. Keep capacity, deadline and all revocation checks unchanged.
Extend the ring regression across capacities and retained/evicted outcomes;
first run against the original implementation, then apply this order repair.
No added queue, cache, owner or runtime authority beyond an actual stamped
callback; missing/evicted callbacks still fail closed.

Final Firefox two-gesture evidence at `.../final-native/native-control.json`:
first-word and mixed-prefix round trips pass with two delegations and actual
intermediate/returned surfaces. Accepted completion projects once but its
return is blocked; closing-Space input is NOT_TESTED because concurrent
non-atomic report writes caused invalid JSON before input. The collector also
left an extra closing brace in one final state file. Repair only collector
serialization/atomic writes before reusing it. Production browser/global IBus
remain unchanged; installed C20 is restored. Physical acceptance is NOT_TESTED.

The two-toggle accepted-completion loss occurs at the closing Space of native
exact replay: callback returns false for client delivery, so the boundary
receipt producer ignores its validated owned append. At that point no fresh
client snapshot exists and normal readout correctly refuses. Extend the same
boundary predecessor capture to the live exact-replay scope, then retain the
inert closed-tail receipt only if the key actually advanced that scoped tail.
This transfers no edit authority until the exact post-Reset client snapshot;
expired/quarantined-invalid replay remains excluded. Extend the existing coarse
Reset callback regression to both open and closed replay tails, including
passive readout before confirmation and successful subsequent manual mutation.

The final grouped RED is `run-056zl5c2` / remote `run-nEsHLF`: 541 correctness
pass, two intended failures (retained stamp after unrelated eviction; closed
replay tail missing its Reset receipt). Earlier fixture attempts corrected
retained-prefix handling and the Space physical keycode before this causal RED;
those failures were not runtime proof. Runtime repair changes only stamp lookup
order after revocation checks and the existing boundary receipt's recognition
of an actually advanced live replay tail. GREEN: `run-tnbw8qix/RESULT.json`,
remote `run-mwkb79`, focused IME PASS in 18.64 seconds (545 selected identities,
including existing refusal/owner/replay tests). Test identities/count unchanged.
Actual final Firefox and private-client acceptance remain pending. Production
runtime unchanged. Freeze the graph and binaries for four two-gesture checks.

R3 native resumption reused the two completed round trips; the earlier remaining
cells had explicit FOCUS-CANCELLATION and were not accepted. The resumed
closing-Space case now passes both visible projections and exact boundary
return. Accepted completion still fails, validly: actual completion appends
five letters plus Space, then Reset serial 66 has no predecessor receipt and
both manual gestures are blocked. Receipt:
`~/.cache/lay/development/td121-firefox-roundtrip-r3-20260914/native-resumed-two-cases/native-control.json`.
First loss is inside the existing boundary receipt producer: it rejects all
command-modifier states, including Alt's own modifier bit on the successful
Alt release. The previous grouped completion fixture used release-mask only.
Extend that fixture to the actual release state; preserve command chords and
require the same actual accepted append, exact predecessor and fresh Reset
snapshot. No new authority/queue/deadline or target-specific condition.

R4 causal RED: `~/.cache/lay/development/run-27egvxnh/RESULT.json`, 544/545 pass;
only accepted Alt release with state 1073741832 lost its receipt. Release-mask
alone and Tab/Space routes passed. Repair ignores only Alt's own MOD1 bit while
classifying a completion release; Ctrl/Super and other command bits remain.
The actual append, predecessor, owner, token, epoch and fresh client receipt
checks are unchanged. The shared existing MOD1 constant is exported for reuse.
Grouped regression now includes Ctrl+Alt and Super+Alt negative cases.
GREEN: `~/.cache/lay/development/run-w9hz8fni/RESULT.json`, 545/545 PASS in
27.9 seconds, same manifest identities. Freeze the R4 development candidate
and test only the still-unaccepted completion round trip before release gates.
No installation or physical acceptance. Prior R3 native passes remain dated
R3 evidence and cannot establish final R4 native-byte acceptance.

R4 native candidate reaches a distinct earlier first-word loss before Alt:
retired published preedit has 5 characters; Firefox reports cursor 2 then
cursor 1 before the exact 2-character committed text. The first cached receipt
retains the candidate; the old-cursor receipt wrongly destroys it. Later exact
text and Reset cannot recover the typed predecessor, so no completion appears;
unhandled Alt is followed by FocusOut/FocusIn and the capture times out.
Receipt: `~/.cache/lay/development/td121-firefox-alt-release-r4-20260914/native-completion/native-control.json`.
This does not accept or disprove R4's separately reproduced own-Alt-bit fix.

The bounded consequence is to recognize both independently represented cursor
positions for the same actually published retired preedit: its original prefix
and the currently observed appended prefix. Keep owner/token/epoch checks,
exact presentation and boundary matching, selection refusal and no edit
permission until the fresh exact committed snapshot. No new cache or authority.
Extend the grouped publication regression to current/old/current cursor order.
The old negative fixture called the published prefix cursor 'wrong'; move that
position into the positive inert-retention proof and keep the negative cursor
strictly inside the old prefix. All other contradiction cases remain unchanged.

R5 grouped RED: `~/.cache/lay/development/run-mdrvuzd8/RESULT.json`, 544/545
PASS; only the retired-publication current/old/current cursor sequence loses
its inert candidate. Repair evaluates exactly two already represented prefix
positions against the same bounded published surface. No new stored state,
strings, command, timer or authorization. GREEN:
`~/.cache/lay/development/run-4y_owq5d/RESULT.json`, 545/545 PASS in 28.1 seconds,
including the unchanged selection, wrong surface, wrong prefix, unpublished,
stale-owner, sensitive, reset, command-chord and unconfirmed-mutation refusals.
Native completion and all final R5 research-byte release checks remain pending.
Runtime installation and physical acceptance are unchanged.

R5 native completion PASS: `проверка ` -> `ghjdthrf ` -> `проверка `,
two actual delegations, first target event 55 and returned target event 93.
Receipt: `~/.cache/lay/development/td121-firefox-preedit-cursor-r5-20260914/native-completion/two-toggle-visible-proof.json`.
Default-feature IME SHA-256:
`1bd6cbcdb41f6bfa344b8988e3011a7981729fe9aea7b066c8471c12c05ee481`.
Original Firefox/global IBus are unchanged and installed C20 was restored.
This is synthetic native Firefox evidence, not human keyboard acceptance.
All four scenarios must still pass on the final research-feature release bytes.

Final release verification keeps the unchanged `check-lay-full.sh` mandatory.
Its `scripts/check-lay-tests.sh all` is identical to the affected script's lane,
so the external runner executes that Rust lane once inside the full gate.
A hash-bound derived affected-check script retains every other command and
records this single omission and the required `40-full` result explicitly.
It is labelled `30-changed-unique`; no unmodified changed-gate PASS is claimed.
Any full-gate failure still fails release acceptance. Canonical release scripts,
manifest membership, lint baselines and acceptance assertions are unchanged.
This implements the requested no-repeat policy without dropping unique checks.

### 1.0.73 Firefox actual-text notification route

The ordinary Wayland GTK module leaves the final `про` surrounding receipt
unpublished, so completion candidate generation remains correctly blocked.
Selecting direct GTK IBus asynchronously makes printable releases observable.
A post-release request alone repairs completion but disrupts the second
first-word gesture through replay/reset interleavings. The accepted client
route combines two notifications for Firefox's actual text: one after an outer
handled printable release and one after an outer GTK Reset. It keeps
`IBUS_ENABLE_SYNC_MODE` absent.

Cache-only native receipt
`release-1.0.73-final-20260920-r4/native-direct-ibus-combined/native-control-combined.json`
is **4/4 PASS**: first word, mixed prefix, accepted completion including closing
Space, and trailing Space each show both exact surfaces and two manual
delegations. Separate receipt
`native-direct-ibus-modifier/native-control-modifier-r2.json` passes repeated
`Shift+1` as exact DOM `!!`, with two managed commits and no manual toggle. The
modifier proof is synthetic, not a human-keyboard claim. Main Firefox and
global IBus were preserved and installed 1.0.72 was restored.

The source consequence is confined to the Firefox client boundary. The
launcher selects `GTK_IM_MODULE=ibus` and preloads the adapter inside Snap while
explicitly removing synchronous IBus mode. The adapter preserves GTK's real
filter/reset behavior and requests `retrieve-surrounding`; it never reads or
constructs text. All mutation authority remains with the engine's owner,
focus, epoch, cursor, selection, exact-snapshot, replay-scope, lease,
SafetyGate and verifier checks. The admitted release excludes command
modifiers and non-printable keyvals. No runtime detector, word condition,
retry, wait, queue, text cache, candidate exception, model call, learning
effect or new authority is added.

Controlled ABI tests must cover the real-call/return and request counts for the
admitted printable release and Reset, and refusals for press, unhandled,
command-modified, non-printable and non-IBus events. A launcher contract must
prove in-Snap preload, direct IBus, and absent synchronous mode. Because this
changes shipped client code, the earlier r4 release is invalidated. Freeze and
run a fresh full release, then repeat all four native cells and the modifier
control on the exact source-owned adapter before installation or publication.

Consequence analysis: this client notification does not add or remove lattice
candidates, alter rank, or convert a hint into authority. It can only cause an
existing Firefox actual-text callback to arrive; false authority remains
bounded by the existing exact-snapshot and lease checks. Each admitted release
adds one synchronous GTK signal emission and Firefox callback on the UI thread;
the native control shows no missed tail deadline, but does not claim a general
latency bound. Work is constant per event, with no persistent allocation or
Lay RSS growth beyond one shared-library mapping. It creates no Lay cache key,
invalidation rule, model/package identity, delta/reload behavior, learning or
feedback effect. A stale callback follows the existing reducer and cannot
outrun owner/focus/epoch revocation. GTK calls are confined to its normal UI
thread; thread-local recursion guards keep nested filter/Reset notifications
inert.

Compatibility risk is the GTK 3 `GdkEventKey` ABI and Firefox Snap launcher
boundary. Missing real symbols fail the owned browser process instead of
silently bypassing verification; a missing library makes the launcher refuse.
Rollback restores the previous desktop Exec lines and removes the launcher and
library. The adapter should be removed when Firefox's direct Wayland route
publishes both post-commit and post-Reset text reliably. Ordinary Wayland plus
the same filter hook was rejected because printables never cross it; direct
IBus with only post-release notification was rejected because it destabilized
the second replay; synchronous direct IBus was rejected by the modifier loss.
Granting from the engine's committed-tail mirror was also rejected because it
would create a second source of visible-delivery truth. The combined async
actual-text route is the smallest measured route that passes both text and
modifier controls without a new authority owner.

Source-owned development verification passed **2,864/2,864** selected affected
tests at `run-1p9mnig3/RESULT.json` in 341.5 seconds, including the dynamic
fake-GTK adapter regression, launcher contract, and guarded architecture
refresh. This proves the controlled source contract, not release-byte Firefox
or human-keyboard acceptance. Installed runtime authority remains 1.0.72.

The first source-owned r5 release passed every remote release gate and isolated
client but failed exact native Firefox **3/4**. The adapter emitted requests;
the launcher had dropped Snap's existing
`gnome-platform/$LIB/bindtextdomain.so` preload. Firefox then batched all three
managed DOM commits before one late Reset. Its stale one-character callback
arrived before exact `про`, so the existing one-shot receipt correctly rejected
the mismatch and no completion gained authority. Receipt:
`release-1.0.73-final-20260920-r5/native-release-route/text/native-control-release.json`.
First word, mixed prefix and trailing Space passed; completion had zero hint,
accept and delegation. Protected runtime was restored unchanged.

Repair consequence: preserve the preload already supplied inside the Snap
shell, appending it after the Lay adapter, while direct IBus remains async. This
does not change candidate/lattice, ranking, authority, cache, model/package,
learning, allocation, concurrency or daemon/IME behavior. It restores the
measured Firefox/GTK delivery environment and one extra Snap-owned mapping that
the browser normally expects. Missing Lay library still fails the launcher;
rollback remains the prior desktop Exec plus launcher/library removal. A static
launcher contract must require preservation and Lay-first ordering. The C
adapter and exact receipt gates remain unchanged. Fresh release/native proof is
required because the r5 source launcher bytes failed.

The corrected launcher plus its preservation assertion passed the guarded
affected closure **2,864/2,864** in 340.2 seconds at
`run-od4752nt/RESULT.json`. This is development-only evidence; fresh release and
native acceptance remain required and installed authority is still 1.0.72.

The fresh r6 frozen release passed every remote release lane, ten artifact
checks and all four isolated clients (`release-1.0.73-final-20260920-r6`, remote
`run-uyAAv9`), but exact source-route Firefox acceptance is **3/4 PASS, overall
FAIL**. Mixed prefix, accepted completion plus boundary, and trailing Space
round-trip twice. First word reaches `привет`; its queued second gesture aborts
on `context admission denied`, so r6 is rejected for install/publication and
installed 1.0.72 remains authoritative.

The failing trace places the first loss in read-only bridge acquisition. Nonce
11 completes through `bridge_taken`, but one replay `ProcessKeyEvent` entered
before the marker and had not yet settled, so the correct fence result has no
token and becomes `AdapterError::Denied`. The callback settles immediately
afterward; the existing replay receipt remains inert through two prior
surfaces, then the exact six-character Firefox snapshot confirms it. No text,
owner, focus, expiry, cancellation or transport contradiction occurs. The
daemon stops only because `VisibleTailV3` currently flattens this structured
denial into a D-Bus error before its existing bounded settlement loop can
observe the confirmed receipt.

Selected repair: preserve structured adapter errors in a private bridge-token
helper. Only read-only `VisibleTailV3` maps exactly `Denied` to an empty
`passive:unknown-context` reply. Every mutation-capable method and
`Busy`/`Timeout`/`Cancelled`/bus/configuration failure keeps the existing hard
error. No token, tail, epoch, path, receipt or edit permission is returned on
denial. The existing 80 ms settlement bound and fresh fence per observation
remain; there is no new timer, deadline, queue, retry owner or authority.

Required causal proof is a real P2P unsettled-key fence: old code must fail,
then patched code must return passive with zero client effects and recover the
same authoritative tail only after that exact callback settles. Existing
cancelled-observer refusal remains a hard error. Focused/affected/full release,
four final-byte native cells and the modifier control remain conjunctive. This
repair has no candidate, rank, package/model, cache, learning or feedback
effect; its only bounded cost is another fresh bridge read inside the existing
settlement window.

Initial old-code run `run-y7hgog9t` is fixture-only **HARNESS FAIL** (563/564):
the no-effect probe encountered the controlled dispatcher's expected
detached-object error before its marker, so the new readout predicate did not
run. The correction consumes only that exact harness reply with the existing
helper; production, callback order and the intended RED remain unchanged.

Corrected old-code `run-le3cakpe` is the causal **RED** (563/564): only the new
real-fence case failed, with `Failed("context admission denied")` exactly where
an empty passive readout is required. All setup and no-effect checks passed;
runtime source was unchanged.

Patched focused proof `run-3uunyqkd` is **564/564 PASS** in 28.9 seconds. The
new P2P case has zero effects while denied, then reproduces the exact prior
authoritative tuple after callback settlement. The same unsettled state must
also keep `ManualToggleV3` on its hard `Denied` path; the existing
cancelled-observer case remains a hard error. This is development-only evidence.

Affected run `run-b6ancqui` stopped before Rust execution on the required
manifest contract: one added identity (the new P2P proof), no removals and no
changed rows. Refresh and verify that exact delta remotely, then rebind the
unchanged zero-failure registry; no correctness verdict is inferred from this
contract stop.

With the exact one-row manifest refresh, affected receipt `run-wll6ea2b`
passed **2,865/2,865** in 358.3 seconds. A final assertion that the identical
unsettled state still hard-fails `ManualToggleV3` was then added under the same
test identity; focused and frozen full proof must cover it, so the affected
snapshot is not overstated as evidence for that last assertion.

Focused `run-ho7o518l` was blocked before discovery by test-only `E0618`: a
local `bridge` value shadowed the helper on its second call. The guarded exact
snapshot compile identified it. Renaming only those bindings has no runtime or
proof-semantic effect.

Corrected focused `run-c_ir4jlp` is **564/564 PASS** in 34.3 seconds remotely.
It includes passive readout, hard mutation denial in the identical unsettled
state, exact recovery after settlement and prior cancellation controls. Freeze
r7 and run the canonical 2,865-test full lane once; a hash-bound derived changed
lane may omit only its duplicate Rust-all invocation while retaining every
other changed check. r6 bytes remain rejected.

Frozen r7 release receipt `run-bJ5E28` is **PASS**: the canonical full gate
executed **2,891/2,891** tests once, the derived changed lane retained every
other check, clippy and both architecture checks passed, ten release artifacts
were bound, and all four isolated clients passed. Exact native Firefox then
passed **4/4** two-toggle text cases plus the repeated `Shift+1` modifier
control. Every text case has two manual delegations and the exact expected
surface; the modifier case has `!!`, two managed commits and zero toggles.
Main Firefox, global IBus and installed 1.0.72 were preserved through the
source-route transaction. This accepts r7 bytes for installation; it does not
claim a physical human-keyboard proof. Installation and publication state is
owned by `docs/release-1.0.73-finalization-2026-09-20.md`.

The exact r7 artifacts are now installed and loaded as 1.0.73. Ten installed
hashes and all four loaded Lay owners match the accepted release receipt; the
GNOME extension also reports 1.0.73. Global IBus, the existing main Firefox,
the selected `lay-ime-ru` source, source list and configuration were preserved.
The installation receipt is
`release-1.0.73-final-20260920-r7/installation-1.0.73.json`. Publication remains
the only pending release transaction; physical human-keyboard acceptance is
still `NOT_TESTED`.

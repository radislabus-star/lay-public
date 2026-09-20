# TD-121 final R5: saved-trace continuation, 2026-09-14

## Evidence and verdict scope

Evidence root:
`/home/ubu/.cache/lay/development/release-1.0.72-td121-firefox-r5-20260914/`.
The recorded release gate is PASS, 2,841/2,841 tests, with four isolated client
cells PASS. `final-native/two-toggle-visible-proof.json` is FAIL, **2/4**.
Mixed prefix and trailing Space reach both expected surfaces. First word and
completion do not. This read-only continuation analysis has run no tests and
has changed no runtime authority, installed bytes, browser or global IBus.
Performance, human keyboard acceptance and general restoration quality are
NOT_TESTED by this analysis. DOM `monotonic_ns` is the HTTP collector's receive
time, not the browser event creation time; it must not be treated as input
latency or precisely aligned with daemon events.

## First loss and what is still unknown

Line numbers below are one-based in the case's `ibus_engine_debug.jsonl` under
`final-native/native-control/`.

- `ghbdtn_extra_lshift_enter-8eaffe5b7cc34829dd69`: line 74 commits the first
  character; lines 85–88 confirm its one-character Reset receipt. Lines 95–97
  consume that receipt and delegate an exact one-character manual replay.
  Only later, after layout activation, line 315 receives the second original
  letter and rejects it as an unexpected replay key. Lines 327, 344, 358 and
  372 decode the remaining original keys under the new layout. The daemon
  detail independently records all six physical keys before its trigger.
  Thus the bridge overtakes client-delivered input, and the stale short suffix
  is acted on before the remaining input reaches IME. The IBus bridge marker
  cannot establish delivery of events still upstream in Firefox/GTK. Why the
  browser delivery stalls is not established by these untimestamped key rows.
- `td121_completion_two_toggles-16f55b35eec790991890`: lines 137–139 publish
  the two-character prefix's candidate; line 148 commits the third character.
  Lines 151–152 retain the known retired-preedit surface without granting
  authority. Lines 159–160 confirm the exact three-character client snapshot.
  The immediately following press callback (serial 54, lines 161–165) is Alt
  according to the DOM sequence and unchanged input script. Line 166 records
  worker generation 9 superseded after 230 microseconds, with 11 candidates.
  `process_key_event_with_output` retires a pending display at Alt press;
  accepting that stale display would violate the existing contract. The
  pending-generation cancellation is explained; the delay of the confirming
  client snapshot and the correct notification point still need causal proof.

## Next discriminator and bounded consequences

Do not patch word examples or weaken the exact-tail verifier, SafetyGate,
preedit acceptance, owner/epoch checks, or stale-worker cancellation. Compare
two mechanism-level designs before implementation: (A) bind the daemon's
manual command to actual client input delivery; (B) repair the existing client
notification boundary so actual post-edit context arrives without the next key.
Neither is selected from a native PASS alone. A still-upstream key is outside
the current IBus marker's scope; a cached client snapshot cannot prove delivery.

The smallest remaining observation is the two failing native cells with the
same R5 bytes and existing Firefox IME logging enabled, recording browser
event creation times separately from collector receive times. This is a
diagnostic configuration change for unresolved causal evidence, not a repeated
full gate, new release candidate, retry-until-green, or installation. Retain
the existing focus guard, exact process identities, input timing and cleanup.
Use an independent source review to distinguish a client notification defect
from a cross-transport command-ordering defect. Before any production change,
record the chosen design's concurrency, latency, failure/rollback, consumer
compatibility and maintenance consequences here; keep candidate/ranking,
package reload, learning, cache identity, CPU/RSS and authority effects explicit.

## First instrumented capture

`/home/ubu/.cache/lay/development/td121-r5-delivery-diagnostic-20260914/native-control.json`
records first-word PASS and completion FAIL on the same R5 bytes. This single
PASS does not supersede the saved final first-word FAIL. Completion again ends
at three typed characters; this schedule has no fresh surrounding receipt
after the initial one-character confirmation. Browser-created timestamps show
the first character at 1789342104492 ms and the subsequent preedit/two commits
at 1789342105090–5094 ms, immediately followed by Alt. These timestamped DOM
effects still do not identify the blocking upstream call.

The requested `MOZ_LOG=timestamp,IME:5` emitted no Firefox IME records: the saved
exact Firefox source declares `gIMELog("IMEHandler")` at line 33. This diagnostic
configuration error is corrected to `timestamp,IMEHandler:5`, verified in the
owned process's allowlisted environment, for one completion-only capture at
`/home/ubu/.cache/lay/development/td121-r5-imehandler-diagnostic-20260914/`.
No production code changed. The first capture's `runtime-verification.json`
confirms preserved main Firefox/global IBus and restored C20. No build, full
gate, installation, or human keyboard acceptance was performed.

Independent source review agrees that notification alone cannot guarantee
cross-transport input order. A same-event final Shift-release acknowledgement
could bind a daemon-owned request without a second detector, but legacy
ProcessKeyEvent currently exposes no shared physical-event identity. Another
IBus marker or a merely similar release is insufficient; do not implement that
design until its identity can be bounded.

## Verified Firefox logging and next scheduling discriminator

The corrected IMEHandler capture is PASS for completion, including exact
`проверка ` -> `ghjdthrf ` -> `проверка ` at DOM events 16, 54 and 92. This
instrumented single-cell PASS does not cancel either earlier FAIL. Its
447,644-byte Firefox log is private under the receipt's case directory. At
23:31:47.663177 UTC Firefox successfully returns its old two-character cache
after committing the third character. At .665035 its non-composition selection
notification calls ResetIME; the compatibility adapter then obtains the exact
three-character context at .665419, before the next key. The IME trace shows
the previous prefix's worker superseded, so no active old preedit spans this
third commit. This differs from the final R5 FAIL's active old preedit. Firefox
OnSelectionChange deliberately skips Reset for composition-caused changes
(`IMContextWrapper.cpp`, lines 1703–1728). Its separate deferred surrounding
retry requires a previously failed retrieval. A successful stale retrieval
does not arm that retry. The evidence supports a missing post-composition
notification, not a reason to permit stale completion acceptance.

Before adding a notification hook or cross-transport identity, discriminate
with GTK3 IBus's existing synchronous client mode on the R5 bytes and Reset
adapter. GTK3 defaults to asynchronous mode when the variable is absent;
Firefox's logged asynchronous classification is correct, not a mode mismatch.
The older pre-R5 sync control failed later Reset/replay mechanisms and cannot
answer whether these R5 repairs plus synchronous client delivery work together.

Compare three designs: keep asynchronous delivery and extend the client
post-composition notification; select supported synchronous GTK input; add
shared physical-event acknowledgement. The first requires a proven callback
after Firefox's content cache update, which Reset alone does not provide. The
third currently lacks shared event identity. Select the second only as a
four-cell diagnostic control, changing one owned-child environment variable
and verifying it in /proc; remove diagnostic IME logging for this control.
Keep inputs, deadlines, exact expected intermediate/returned surfaces and
guards. A green control is evidence for that configuration, not proof that
arbitrary upstream stalls cannot overtake a daemon request.

Consequences before any launcher change: synchronous IBus may block the
browser during an IME callback; do not add waits, retries or increase deadlines.
It replaces asynchronous reinsertion only in the explicit Firefox launcher,
with no second detector, text source, cache, queue, owner or fallback. Other
GTK/terminal/atomic consumers stay on their existing transports. Candidate
retention/ranking, false-authority checks, package/delta reload, learning and
feedback are unchanged; no candidate-budget, CPU/RSS or latency improvement is
claimed. The existing process deadline and cleanup bound test failure. A
production proposal would require exact configuration/native evidence and
rollback by removing the one launcher environment assignment; installed C20
and the main browser are outside this diagnostic change.

## Synchronous control: rejected, no launcher change

`/home/ubu/.cache/lay/development/td121-r5-sync-delivery-control-20260914/native-control.json`
records **two PASS, one visible failure, one cancellation before input**. The
actual Firefox environment verifies synchronous mode; this control omits the
instrumentation log and keeps browser-created DOM timestamps. Mixed prefix
and completion pass both exact surfaces. Closing Space loses owned DOM focus
before typing and is NOT_TESTED, not a product regression or PASS.

The first-word case delivers all six letters before delegation (IME rows 77,
102, 126, 144, 163, 185; delegation 231), then visibly reaches the full first
target. All six deletes and six replayed letters are admitted as exact native
replay (rows 267–434). The second gesture times out waiting for the client
receipt and the final field remains at the first target. Reset arms the full
six-character target at row 480; a delayed five-character snapshot rejects it
at row 482. Another four-character snapshot follows at 489; the later exact
six-character snapshot at 517 cannot recover the destroyed predecessor.
The metadata trace does not contain these shorter snapshots' text, so their
identity as old source or new target prefixes is still UNKNOWN.

Completion's only pre-third-letter published hint is from the first prefix,
not the second prefix. Thus this PASS does not exercise the final failure's
second-prefix-preedit schedule. Neither this control nor the instrumented
single-cell PASS selects a production repair. The supported sync setting
fails the required conjunction and is rejected; the launcher remains unchanged.

All new captures' `runtime-verification.json` records confirm restored C20 and
preserved main Firefox/global IBus. No Rust code, compatibility library,
launcher, installed artifact, verifier/SafetyGate or deadline was changed.
No Cargo correctness or full/release gate was rerun. The source binding checks
reused all 768 R5 runtime-source hashes without drift.

## Current implementation boundary

The next production change is not yet admitted: an exact IBus snapshot does
not prove delivery of still-upstream physical keys; receipt retention also
needs to distinguish measured stale client surfaces from actual contradiction.
Before another patch, build a controlled event-order proof using the existing
reducer/adapters, covering both early-bridge and late-client-receipt schedules.
Keep exact same-event/owner binding and fresh final observation separate from
inert candidate retention. Do not replace this missing proof with another
native retry, delay, extra Shift/Alt, source-word exception or weaker verifier.
TD-121 remains open, and the original final R5 FAIL remains authoritative.

The document-only architecture refresh has its separate source-bound receipt
at `/home/ubu/.cache/lay/development/td121-r5-continuation-architecture-20260914/RESULT.json`.
That receipt covers graph consistency and architecture checks only; it cannot
change any of the runtime or native acceptance verdicts above.

## R6 replay receipt preflight

The measured completed-replay schedule supplies shorter client snapshots before
the exact final snapshot. The current first-prefix exception retains one inert
candidate but destroys it on the second snapshot. Controlled proof will drive
the production legacy callback, Reset and surrounding-text adapter with source
deletion prefixes and target replay prefixes, then the exact final receipt.
It must prove passive readout and zero text effects before final confirmation,
and the typed exact-tail route afterwards. This is a replay-schedule proof,
not evidence that the daemon has waited for still-upstream physical keys.

Compare (A) retain arbitrary mismatches until an exact value arrives and (B)
recognize only intermediate surfaces of the already validated exact replay.
Reject A: external edits could retain a predecessor without measured output
provenance. Select B using the existing matched local/shared ExactReplay scope,
current owner/token/tail epoch and completed delete/replay distance. Preserve
only an inert predecessor on exact whole-snapshot source deletion prefixes or
target append prefixes. Selection, cursor mismatch, unrelated text, lifecycle
changes and input gaps still reject. The subsequent full client snapshot and
existing exact lease checks remain the only authority. No new cache, timer,
queue, detector, fallback or text source is introduced.

Consequences: candidate/lattice ranking, package/delta reload, learning and
feedback do not change; no lexical path participates. False authority remains
bounded by exact current owner and snapshot checks, and intermediate snapshots
must keep mutation unavailable. The bounded existing replay strings are scanned
on a surrounding-text callback; no hot key-path allocation, RPC, wait or
deadline increase is planned. Retention lasts only while the existing complete
replay quarantine remains valid and uses its existing lifecycle invalidation.
No CPU/RSS or latency improvement is claimed. GTK replay gets this receipt
recognition; terminal and atomic routes are unchanged. The main risk is treating
an external edit identical to a replay prefix as stale: it remains inert and
cannot authorize anything without the subsequent exact current snapshot.
Rollback removes this predicate and its invocation. Controlled old-code failure,
focused IME regression count, affected contracts and native four-cell acceptance
will be recorded separately. Runtime code has not changed at this preflight.

## R6 controlled implementation and next native discriminator

The selected replay predicate and inert-retention branch are implemented.
`run-9e5i2hiu/RESULT.json` under `/home/ubu/.cache/lay/development/` proves the
new controlled regression fails on the old production code at its first source
deletion-prefix receipt: 545/546 PASS. The final expanded focused receipt is
`run-v02qq0_s/RESULT.json`: **548/548 PASS**, 28.2 seconds total. Three added
tests cover four prefix/source-target combinations with duplicate intermediate
snapshots, seven contradiction/input/lifecycle variants, and four preserved
prefix/confirmation/consumption variants. Every positive path stays passive
until the full exact client snapshot; no intermediate snapshot emits a text
effect. Prefix text and the trailing Space are asserted exactly.

Review pass 1 found no production authority bypass and requested those broader
tables. They exposed a fixture error: `run_exact_replay` sent Space with keycode
30 while production `physical_char` prioritizes the physical code. The fixture
now sends Space as keycode 57, matching the existing coarse/native replay helper;
no production key decoder or safety assertion was relaxed. FocusOut asserts the
specific admission-denied error and no text effects. Review pass 2 found no
remaining production or proof blocker for this replay-only change. The native
completion cell must separately report whether the formerly failing active
second-prefix-preedit schedule was exercised; an easier publication schedule
cannot establish that schedule's repair.

Separate unresolved observation: `run-2bmr2sxi/RESULT.json` failed an old test's
initial Reset setup before any surrounding-text call to the new branch. Later
548/548 does not establish its cause or prove it repaired. The setup assertion
now retains owner/token/scope diagnostics on failure. Formatting and an initial
missing type qualification failed before test execution in separate development
runs; these are not counted as runtime regressions or accepted proofs.

The next source-bound experiment is
`/home/ubu/.cache/lay/development/td121-r6-sync-replay-20260914/`.
Build the changed IME and refresh architecture remotely under the existing
guards. Reuse the unchanged R5 daemon/test-input bytes with explicit hashes;
keep the supported synchronous GTK setting confined to the owned Firefox child.
Run the same four native scenarios once, with the same deadlines, focus guard,
four Shift taps and exact intermediate/returned DOM text. This isolates whether
the demonstrated replay repair removes the previous sync control's return
failure. It is not a full release gate, installation, or proof of arbitrary
evdev-to-GDK event identity. The main browser, C20 and global IBus stay protected.
Native acceptance and human keyboard confirmation remain NOT_TESTED for R6.

Rejected synchronization shortcuts remain separate from this replay change:
WordBuffer has no immutable input identity; Wayland loses hardware source-device
identity and may aggregate seat key state. On the inspected native Mutter path,
libinput CLOCK_MONOTONIC timestamps are preserved through Wayland into GDK, but
collisions, a11y rewrites and queue completeness still need a protocol proof.
Kernel same-device injection is FIFO, but neutral MSC metadata is discarded and
a functional-key marker can trigger global/UI actions; no such injection or
new timestamp/protocol route has been implemented.

## R6 native result and first remaining loss

The source-bound R6 experiment above finished with **3/4 native PASS, overall
FAIL**. Receipts in that directory are `BUILD-RESULT.json`,
`native-control.json` and `two-toggle-visible-proof.json`. The changed IME hash
is `a49d804e8e1ad9ad3a8715a3587b75b9c2fd9c2891a51002e70817d4507ef1c5`;
the daemon and input generator reused the recorded unchanged R5 bytes.
Remote architecture refresh and release-mode IME build passed under the guards.
No full R6 release gate or installation occurred. C20 was restored, and the
protected main Firefox/global IBus identities were unchanged.

First word, mixed prefix and trailing Space each show the required intermediate
target and exact returned source, with two manual delegations. Their DOM event
indices are respectively 45/71, 72/106 and 43/69. Completion produces final
`про`, zero accepted hints and zero manual delegations. This is a real failure
before completion acceptance, not the earlier focus cancellation before input.
The subsequent unaccepted Alt opens Firefox's menu and the capture times out.

In completion's `ibus_engine_debug.jsonl`, second-prefix preedit is actually
published at rows 139–141. Third-character commit is row 150; rows 153/155
retain stale published-preedit snapshots, and release is handled at row 161.
The exact three-character receipt arrives at row 163 immediately before Alt.
The matching worker is then superseded at row 170 after 257 microseconds.
The existing `pending_alt_gesture_retires_before_release_and_cannot_accept`
contract must remain intact: a late worker cannot acquire acceptance authority
from an Alt gesture that began while work was pending.

Independent source inspection rejects treating synchronous GTK
`filter_keypress` success as proof of fresh Firefox content. GDBus's private
waiting context does not establish processing of Gecko content-cache IPC;
another retrieve can still return the old snapshot. A post-release retrieve is
at most a diagnostic, not a selected production fix. No such hook was added.

## Kitty comparison requested by the user

The user reports that Kitty works ideally, whereas Firefox's first replacement
works and subsequent replacements break. Whether a long pause also fails is
not yet established by that report. This is user-observed client behavior,
not new physical acceptance of the uninstalled R6 candidate.

Current source uses the same daemon DoubleShift detector and manual projection
plan. Kitty's proven terminal route calls
`toggle_committed_tail_target_with_disposition` and
`replace_committed_tail_with_effect_progress`; `state.rs` builds one
`CommitText` payload from `terminal_erase_prefix(count)` (U+007F repeated) and
the replacement. Its local tail is updated on the same execution path. The
Firefox committed-tail route instead delegates through `ManualToggleV3` to
`execute_exact_ime_tail_replay`, with layout/focus/tail validation and bounded
physical delete/replay. The selector uses capabilities, not a Kitty name test.

The terminal byte protocol cannot be copied directly into a browser editor.
The inspected Firefox 155.0.1 `OnCommitCompositionNative` dispatches a text
composition commit; deletion is a separate `OnDeleteSurroundingNative` /
`DeleteText` path, which selects a range then dispatches a delete command.
One terminal commit therefore does not establish a browser replacement
transaction. A shared replacement principle is under read-only investigation;
no terminal behavior, authority contract or production transport was changed.

## R7 preflight: early readout, current receipt, existing cache

The user subsequently authorized autonomous repair, verification, installation
and push through completion, without waiting for their proposed manual Firefox
check. The known-failing R6 has not been installed. Automated acceptance must
still demonstrate every required surface; it cannot be reported as human proof.

**Measured first loss:** after the third handled append, the existing inert
Reset predecessor already identifies the new suffix, but readout scheduling is
coupled to publication authority. The client confirms it only immediately before
Alt. The completion script has 340 ms between typing and Alt, whereas the worker
display deadline is 150 ms. Keeping an old worker result until confirmation or
restarting its deadline would change the accepted late-result contract.

**Alternatives:** (1) synchronous complete scoring after exact confirmation is
implementable, but moves all cache-miss/cold-source work into the IME callback;
the measured first-prefix readout alone took 5.2 ms. Reject that hot-path cost.
(2) a worker-owned completed-result slot requires a new retained-result lifecycle
and does not satisfy the unchanged 150 ms deadline for this actual schedule.
Reject it. (3) a Firefox post-cache-update notification remains a plausible
producer repair, but no guaranteed GTK callback has been established; the
post-release hook cannot provide that guarantee. (4) select the existing bounded
readout cache: compute inert candidate material earlier, then perform a new,
cache-only lookup when the full exact receipt supplies publication authority.
A miss follows the existing background path and performs no inline scoring.

The singleton precognition worker and its latest-work slot remain the only
computation scheduler. Its existing generation, cancellation, 150 ms deadline
and final `precognition_identity_matches` publication check remain unchanged.
The early route captures the current frame only after both key settlement and
Reset-predecessor advancement. Retained stale snapshots may sustain computation
eligibility, but cannot display, accept, replace, delete or authorize feedback.
No new result slot, timer, retry, source of text, mutation owner or detector is
introduced. Exact receipt does not change the current token/epoch; it changes
only snapshot/revision/confirmation, so the same material request can be reused.

**Cache identity and invalidation:** `candidate_gate/cache.rs` already memoizes
complete shared-gate readouts before a worker is discarded. It is bounded to
128 entries. The audit exposed missing dependency identity, which must be fixed
before adding the cache-only consumer: old computation can repopulate after
`clear`, sidecar reload changes field generation without clearing this cache,
and usage/L3 updates can change scores. Bind keys to existing material generation
and an invalidation revision within this same cache owner. `clear` advances that
revision. Recheck identities on lookup, store and result return. Invalidate before
usage feedback is applied under its existing lock and before the L3 runtime Arc
is replaced under its existing write lock. No cache mutex may be held across
scoring or dependency locks. An invalidation race yields a miss/empty result,
never a stale hit. Cold startup remains nonblocking, and package bytes/reload
authority are unchanged. The immutable phrase memory adds no reload lifecycle.

**Candidate retention, ranking and authority:** reuse the shared gate's exact
candidate vector and order. Share the existing suffix-only projection, declined
target filtering and final proposal selection; no words, source IDs or prefixes
become runtime conditions. The additional experimental semantic phrase branch
is outside this cache. If its current predicate can participate, the inline
route must miss so the complete normal materialization runs. Cache hits confer
no context or mutation authority; the exact current frame remains mandatory.
Thus no ranking formula or candidate source is added or removed. Cache
invalidation can remove previously stale rankings after actual memory updates;
fixed-source parity and reload-race proofs must distinguish that from policy.

**Latency, CPU and RSS:** early readout can perform work later invalidated by an
edit; the existing single latest-work slot bounds concurrent work. Stale client
callbacks may reschedule that bounded slot. The cache-only callback does only a
bounded lookup, projection and publication, never a miss computation or file
load. At most two scalar identity fields per existing cache key are added;
capacity and candidate limits stay fixed. Measure focused/runtime timings and
native outcome, not an estimated speedup. Do not change keyboard spacing or
the worker/display/activation deadlines to obtain PASS.

**Concurrent acceptance:** preserve the decision made at Alt press. Explicitly
prove AltPress -> exact receipt/cache hit -> AltRelease cannot accept a hint that
was not ready at press. Keep the existing pending latch effective throughout
inert computation, and use the existing modifier/acceptance state if needed;
no second gesture owner. Printable input, navigation, focus/owner change,
sensitive content, configuration change, boundary and reload must invalidate or
fail the old request. Publication failure cannot leave an accepted candidate.
Computation and cache warming themselves never record learning or feedback.

**Proof and rollback:** first add a controlled legacy callback regression that
provides cached material but requires exact-receipt publication before a queued
Alt; demonstrate old-code RED. Add separate early-scheduling/zero-effect proof,
cache-only parity/miss tests, invalidation and in-flight-store races, semantic
branch coverage, and the opposite Alt ordering. Use actual reducers, worker or
cache methods, not a second modeled implementation. Run explicit IME and shared
library tests remotely, then both required release gates and final-byte native
four-cell acceptance. Preserve terminal/atomic/DoubleShift owner contracts.
Rollback removes the new early/inline route and its cache-only API together;
retain the independent cache invalidation repair only if its own proof passes.
Maintenance is confined to existing owners and their explicit dependency
invalidation calls. This preflight changes no production code or authority.

Independent preflight audit also requires invalidating usage replacement from
disk and periodic refresh, not only feedback. Initial usage fill must not
invalidate its own first readout: no prior scored entry can exist before the
first usage snapshot. The existing cache lock is released before usage/L3
readout, so invalidation under those existing dependency locks has no reverse
lock edge. Preserve this ordering and test the cold-fill distinction. Preserve
`display_only_pending` for eligible inert work so the existing Alt retirement
latch still sees it. No additional hard blocker remained in this bounded audit.

R7 controlled RED is now measured on unchanged R6 production:
`/home/ubu/.cache/lay/development/run-rwowz8tt/RESULT.json`, 28.4 seconds,
**549/550 PASS**. The only failure is
`td121_exact_receipt_publishes_cached_current_suffix_before_alt`: after the
actual exact client callback the suffix is `None`, although the ordinary shared
readout cache already contains the expected current `верка` material. The
opposite Alt-before-receipt negative passes. This proves the boundary failure
with prepared material; a separate proof must establish early computation and
zero publication while the real client receipt is still missing. The preceding
`run-t3ytpdkm` did not execute tests: its new fixture assigned the enum to the
string-valued configuration field; the fixture was corrected before this RED.


### R7 implementation and focused proof

R7 now separates inert computation eligibility from publication authority in the
existing worker route, and tries only already-computed complete shared-gate
material after an exact current Reset receipt. The worker's final identity
matcher and 150 ms deadline are unchanged. Cache keys bind material generation
and cache invalidation revision; usage replacement/feedback and L3 installation
invalidate before their state changes. The inline projection shares ordinary
candidate ordering and declined-target filtering; applicable experimental phrase
material forces a miss. No inline miss scoring, new runtime cache, timer, queue,
gesture detector or mutation route was added.

`run-llqz9gn5/RESULT.json` executed 2,345 tests in 159.7 seconds:
2,343 PASS, two new IME fixtures failed on an unexpected transport reply.
The real worker starts zbus's object dispatcher; the fixture drives a detached
engine callback itself, so that dispatcher emits `UnknownObject`. The fixture
now starts the dispatcher deliberately and consumes only the exact matching
serial/error reply before invoking the actual engine callback, using the
existing detached-callback proof helper. Client signals and mutation assertions
remain unchanged. Shared library tests were 1,795/1,795 PASS in that run.

`/home/ubu/.cache/lay/development/run-qo879484/RESULT.json` is **2,348/2,348
PASS**, 156.7 seconds: 553 IME and 1,795 library tests. This includes both Alt
orders, early scheduling with zero publication, complete-cache parity/miss,
actual delayed-store-after-clear, material-generation mismatch, usage initial
fill versus replacement/feedback, L3 replacement invalidation, declined-target
projection and semantic-source miss. Independent review pass 1 found no remaining
production blocker after fixing an evidence mismatch: discarded stale readouts
must record zero returned candidates. Native/release acceptance remains pending.

A final controlled worker test adds a test-only one-shot notification to the
existing completion observer. It waits for real materialization and the normal
discard of a completed worker for this detached fixture, without polling or
sleeping. The interface is absent, so this test does not reach the worker
publication matcher; its false predicate is asserted separately. Native evidence
is still required for the registered live engine. An actual package reload
inside the isolated test process supplies the initial cache miss; no installed
process or package file is changed. Its receipt is pending. Runtime installation
and the protected Firefox/global IBus identities remain unchanged.

Independent review pass 2 is complete: no remaining blocker after changing the
completion observer's stage parameter to its actual static-string lifetime.
The test hook is entirely `cfg(test)`. The first attempt to run this added test
was correctly blocked before execution because a document edit overlapped
snapshot creation; it is not a test failure or an executed denominator. Source
and documentation are frozen for the next focused run.

`run-35i2oi4u/RESULT.json` executed 554 IME tests: 553 PASS; the added worker
fixture stopped before scheduling because it incorrectly required an optional
productive package to be installed in the focused sandbox. That sandbox uses
built-in candidate material. The real reload invalidates readout material on
both success and unavailable-package outcomes. The fixture now asserts the
required cache miss directly; the later actual worker, nonempty result,
cache-hit, zero-effect and exact-publication assertions are unchanged.

`run-k4iqjlr0/RESULT.json` then executed 554 IME tests: 553 PASS; the added
fixture repeated its unchanged last surrounding snapshot, which deliberately
does not refresh the worker. Its observer consequently timed out. The fixture
now supplies the other already-supported retained-preedit cursor position and
asserts one actual schedule before waiting. This corrects its event sequence;
no runtime condition, production timeout or candidate assertion changed.


Final R7 focused IME proof is **554/554 PASS**, 28.9 seconds:
`/home/ubu/.cache/lay/development/run-n3pbivsh/RESULT.json`. The added actual-worker
cache-miss chain passes. The complete shared-library result remains 1,795/1,795
PASS in `run-qo879484` on unchanged library production sources. Both independent
review passes are complete. These are focused development proofs; R7 native
and final release acceptance are still pending. The next frozen candidate builds
all three native-smoke binaries (daemon, IME and sender), because the shared
library changed. Its source/build/native evidence directory is
`/home/ubu/.cache/lay/development/td121-r7-current-receipt-20260914/`.


### R7 native result: completion repaired, delayed boundary still fails

R7 build/source receipt:
`/home/ubu/.cache/lay/development/td121-r7-current-receipt-20260914/BUILD-RESULT.json`.
Architecture update PASS (43.464 s), three-bin release candidate build PASS
(178.826 s), Cargo budget before/after PASS. IME SHA256
`3c8e9a353dd8c02602ab1c7b1664bdb3719d91ed83b70c43746ac6298b44b804`;
daemon `a7fe3d2d6afb676f9a91a8a1ac3adf22b7d91da9423ec16c4cc83fa781d104bb`;
sender `292ca7cec1f0fb408731d95373f21c06afdb87a3838f5a34f17c1e5fad3ebd0e`.
All 1,408 frozen source identities were checked after graph output retrieval.

The native gate is **3/4 PASS, overall FAIL**. First word visibly reaches
`привет` at DOM index 44 and returns `ghbdtn` at 70. Mixed prefix reaches
`file проверка` at 75 and returns `file ghjdthrf` at 109. Completion now passes:
accepted `проверка ` becomes `ghjdthrf ` at 55 and returns exactly `проверка `
at 93. Each has two manual delegations. The trailing-Space case has zero
delegations, no intermediate `текст ` and final `ntrcn `; unchanged final text
is not a round-trip PASS. Receipts: `native-control.json` and
`two-toggle-visible-proof.json` in that directory. Runtime verification confirms
C20 restoration and preservation of the main Firefox/global IBus identities.
R7 is rejected for release acceptance; no installation or publication occurred.

The first measured loss is the closed word's inert predecessor. In the failing
Space trace (`native-control/td121_space_two_toggles-ab9c312079c13273cbe6/ibus_engine_debug.jsonl`),
row 175 confirms three characters. Managed appends at 187 and 201 reach five
without a new Reset/snapshot. Space commits the sixth character at 216 and
settles a KnownStart successor at 217. Only then do three authenticated Resets
arrive, followed by snapshots of lengths four, five and six. All three report
`missing_predecessor_or_post_reset_token`. The pre-Space capture gate required
an exact current receipt; absent that receipt it discarded the existing inert
append lineage when closing the word. R6's passing schedule delivered each
Reset/snapshot before the next character. This comparison does not establish
that R7 caused the client scheduling difference. It establishes a required
schedule the combined candidate still fails.

### R8 preflight: retain the observed predecessor across a literal boundary

**Selected change:** for a handled literal Space press, allow the already-current
inert Reset lineage to supply the existing boundary candidate. Reuse
`context_reset_rereceipt_computation_allowed`, existing candidate capture and the
unchanged owned-append settlement. That settlement still requires a suffix-only
actual append ending in whitespace, increased tail epoch, a new revalidated
KnownStart token and the same owner. It creates only `confirmed=false`. Each
later authenticated Reset can rebind that retained predecessor; existing first
strict-prefix handling remains inert. Only the final exact full client snapshot
can grant the existing manual handoff authority. Do not change prefix matching,
SafetyGate, verifier, exact-tail leases, Alt acceptance or the gesture detector.

**Alternatives and consequences:** a new append-event journal could retain the
same evidence across Reset batches, but duplicates the pending lineage owner
and adds an event lifecycle where the current structure already holds every
required identity. Deferring Space until a client notification would add a queue
and change ordinary typing; no guaranteed Firefox post-update callback has been
established. Retaining the current inert candidate at its actual boundary is the
smallest supported route. This changes neither candidate generation/ranking nor
lattice contents, model packages, cache identity/invalidation or learning rules.
No computation, allocation beyond the existing candidate clone, timer, deadline,
worker, queue or fallback is added. Terminal and atomic mutation routes remain
unchanged; the new eligibility requires the existing current Reset lineage.

**Risks and gates:** carrying a suffix across an unrelated edit would create
false authority. Selection, foreign text, navigation, new printable input,
focus/owner change, configuration/content sensitivity and capability loss must
still clear or reject the old lineage. Pending material alone must never delete,
commit, show or permit feedback. A late Reset cannot revive a discarded suffix;
consumed receipts remain one-shot. Prove the actual legacy callback order with
multiple append lengths, literal Space, repeated Reset+strict-prefix receipts,
then full exact receipt and real ManualToggleV3/VisibleTailV2 effects. Prove
contradiction/focus/new-input failures separately. First run these tests against
unchanged R7 production for controlled RED, then the complete focused IME set,
independent review and all four original native cases on changed bytes. Any
remaining native failure blocks installation, regardless of the completion PASS.
Rollback removes only the new literal-Space eligibility term; R7 evidence and
its separate acceptance limitation remain recorded.

R8 controlled RED on unchanged R7 production:
`/home/ubu/.cache/lay/development/run-1febvdnu/RESULT.json`, **555/556 PASS**,
28.1 seconds. The positive grouped test fails for both append lengths at the
same first loss: `retained_after_space=false`, no final exact authority,
ManualToggleV3 returns `(0,false)` and VisibleTail is `passive:unknown-context`.
All five contradictory/intervening-input cases pass separately. The runtime
change now adds only literal-Space-press/current-inert-lineage eligibility to
the existing boundary-candidate capture. Later settlement, Reset/prefix rules
and all mutation authority checks remain unchanged.

The first R8 candidate run (`run-6qm97nvg/RESULT.json`, 28.5 s) is 555/556:
both new tests pass, but the existing first-delayed-prefix negative requires
an unconfirmed-from-start lineage to be discarded at a boundary. That fixture
has only received `ab` for observed `abc`, never a full exact confirmation.
The native loss and both new positives instead have a full earlier confirmation
before additional owned appends invalidate the current snapshot. Narrow the
new literal-Space eligibility to `pending.confirmed` plus the existing current
identity predicate. This reuses the existing flag retained across observed
appends; it does not infer fresh authority from it. A lineage that has never
been fully confirmed cannot cross this new boundary route. Preserve the old
negative assertion unchanged. All later exact-receipt/Reset checks remain.


R8 narrowed candidate focused check: **556/556 PASS**, 28.2 s,
`/home/ubu/.cache/lay/development/run-6wh58qut/RESULT.json`. Both grouped
regressions and the pre-existing never-confirmed boundary refusal pass.
Independent review pass 1 found no blocker: retained data grants no authority,
owner/capability/content/focus invalidation is preserved, and only the final
exact receipt permits the existing bridge. This is development proof only;
the four original native scenarios and release gates are still pending.


R8 review pass 2: PASS, no blocker in the narrowed boundary mechanism. Candidate
build and graph refresh PASS (178.782 s and 43.675 s), with all 1,408 source
identities verified. Exact receipt:
`/home/ubu/.cache/lay/development/td121-r8-literal-boundary-20260914/BUILD-RESULT.json`.
The original four native inputs and schedules were each run once: **3/4 PASS,
overall FAIL**. Trailing Space now reaches `текст ` then exact `ntrcn `;
mixed prefix and accepted completion also pass both transitions. First word
reaches `привет` but never returns: one delegation, then queued settlement
refuses `passive:unknown-context`. Native and visible proofs are
`native-control.json` and `two-toggle-visible-proof.json` in that directory.
Installed C20 was restored, main Firefox and global IBus identities preserved.
No release, installation or push occurred; hardware keyboard acceptance and
performance are unmeasured. No claim that the R8 boundary change caused the
independent first-word scheduling variation.

### R9 preflight: preserve the original predecessor through batched Reset callbacks

**Measured first loss:** in R8 first-word IME trace, one-based lines 469–471
observe Reset serials 142, 144 and 146 before callback 142. Callback 142 installs
the reducer's latest revoked token and arms the retained suffix. Its delayed
four-character snapshot stays inert as a completed-replay prior surface.
Callback 144 captures that same current pending token as its predecessor; the
reducer has already incorporated all three Reset events, so the post-Reset
token equals the captured token. `arm_context_reset_rereceipt` rejects it as
`post_reset_identity` (line 478), discarding the original predecessor. Reset
146 then has no predecessor; the full six-character receipt cannot restore
exact authority. The trace path is
`td121-r8-literal-boundary-20260914/native-control/ghbdtn_extra_lshift_enter-56dac943372a5444bbca/ibus_engine_debug.jsonl`
under the development evidence root. Candidate generation, rank, physical
pair recognition and replay itself are not the first failing layers.

**Selected change:** retain the existing pending receipt's original predecessor
when capturing an unchanged pending lineage across another authenticated Reset.
The existing branch already checks tail epoch, exact suffix bytes/length, current
local token, scope and owner. The arm still requires authenticated owner-matching
Reset, current revalidated post-Reset token distinct from the original
predecessor, and recreates only `confirmed=false`. Final exact snapshot and the
existing one-shot handoff checks remain mandatory. The callback must not invent
a new predecessor generation merely because the observer processed a batch.

**Alternatives and boundaries:** moving revocation from observer to callback
would weaken timely revocation. Creating a separate per-callback token owner or
queue duplicates existing reducer/stamp ownership. Preserving original
predecessor identity in the existing candidate uses no new state, timer, queue,
allocation lifecycle, model/candidate work, deadline, fallback or mutation route.
No SafetyGate, verifier, snapshot/prefix matching, Alt policy or replay changes.
Rollback restores the prior candidate token choice only.

**Risks and proof:** an old predecessor must not hide intervening input, foreign
owner, selection, contradiction, lifecycle loss or an already-consumed receipt.
First reproduce the actual observer-ahead batch using production admission,
legacy Reset/SetSurroundingText callbacks and ManualToggleV3/VisibleTailV2.
Group multiple batch sizes and suffix contexts; intermediate prefixes grant no
effects and only the final exact receipt can release the manual bridge. Run
controlled RED on unchanged R8 production before changing capture; retain all
existing negatives, add grouped batch contradiction/invalidation proof, then
complete focused check, independent review and the original four native cases.
Any native failure still blocks the full release/installation/publication gate.


R9 controlled RED on unchanged R8 production: **557/558 PASS**, 28.9 s,
`/home/ubu/.cache/lay/development/run-b4pvbpda/RESULT.json`. All six grouped
positive contexts fail at the second callback: retained vectors begin
`[true,false]`, no final exact authority, no authoritative VisibleTail, bridge
returns `(0,false)`. Batch sizes 2/3/5 and plain/preserved-prefix boundary tails
show the same first loss. The five independent invalidation cases pass.
Preflight independent review agrees with retaining original provenance and
preserving the equality guard in arm. Implementation retains that predecessor
only through the existing pending identity branch, with explicit predecessor
owner and inequality checks. A duplicate actual wire ingress negative was also
added: the existing stamp store must fail closed, and subsequent old callbacks
cannot restore authority. No callback receipt consumption rule was changed.


First R9 candidate check (`run-5c_0d_ha/RESULT.json`, 29.0 s): 557/558.
All six positive batched schedules now pass. The added duplicate-ingress
negative reaches the expected observer cancellation, then fails in the shared
live-observer-only VisibleTail helper's `unwrap(Cancelled)`. Correct that test
choreography by calling the same production VisibleTailV3 and ManualToggleV3
bridge methods with real peer Ping/marker service and asserting the exact
context-admission denial; do not require a cancelled observer to process another
message. No production, deadline or expected authority changes for this repair.
Independent R9 code review pass 1 found no blocker before this fixture correction.

The fixture correction initially did not compile (`run-6u35zvqh/RESULT.json`,
16.7 s; zero executed tests): its bridge type was not imported in this module.
Qualify the test construction as `crate::bridge::LayImeBridge`. The compact
failed-discovery receipt lacks the compiler JSON diagnostic, so no semantic
verdict is attributed to that attempt.

The qualified fixture compiles (`run-707dxf64/RESULT.json`, 28.8 s), 557/558:
the actual bridge refuses with the more specific `metadata observer cancelled`,
not generic `context admission denied`. Preserve the exact error assertion with
that established cancellation category; both methods remain tested and all
positive cases pass. This is another fixture expectation correction, with no
production or authority change and no additional formal review pass consumed.


R9 final focused check: **558/558 PASS**, 28.8 s,
`/home/ubu/.cache/lay/development/run-caw92c8z/RESULT.json`. This includes all
six positive batches, all six invalidations including duplicate ingress, and
the prior 556 tests. No production change occurred after the original
predecessor-preservation patch; subsequent changes corrected only the new
negative fixture. Final independent review and native acceptance remain pending.

### Firefox launcher deployment preflight

Live inspection confirms main Firefox PID 2267700 has GTK IBus and Wayland,
but lacks `IBUS_ENABLE_SYNC_MODE` and the compatibility library mapping. The
user CLI wrapper and all four user desktop Exec entries still invoke Snap
directly; `~/.local/bin/lay-firefox` is absent. Candidate native runs already use
the canonical source launcher with synchronous IBus supplied by their owned
child environment. A candidate PASS alone cannot establish ordinary-launch
coverage while those routes bypass the tested setup.

The source launcher will set `IBUS_ENABLE_SYNC_MODE=1` inside Snap alongside its
existing GTK IBus and compatibility-library environment. This moves an already
measured supported GTK configuration into the canonical launch point; there is
no new hook, delay, key detector, model work or IME authority. Preserve the
existing backend selection and argument forwarding. A missing library remains
an explicit launch refusal. Test shell syntax in the guarded release job and
all original native inputs through this launcher. After final release acceptance,
atomically install these exact launcher bytes and route the existing CLI and
four user desktop entries through it, preserving each argument list and backups.
No global IBus restart or input-source migration is authorized by this change.
An already-running Firefox retains its original environment; a normal exit and
relaunch preserving session is a separate deployment activation, and loaded
process evidence must establish it before claiming the main browser uses the
repair. No browser restart or user launcher modification has occurred yet.


R9 final review pass 2: PASS. Candidate build/graph PASS (179.148 s/43.682 s),
all 1,408 source identities checked; IME SHA-256
`c01054056b07c50e3342a2c4c6c6eb708c2824e324d8cb15487e4b73b104f9ac`.
Native original four-case proof remains **3/4 PASS, overall FAIL** at
`/home/ubu/.cache/lay/development/td121-r9-batched-reset-20260914/two-toggle-visible-proof.json`.
First word, mixed prefix and completion now each show both exact transitions.
Space shows the first `текст ` and two manual delegations, but no second replay:
the subsequent VisibleTailV3 capture returns `context admission deadline expired`.
Its final DOM remains `текст `; Enter is rejected under existing replay scope,
so capture eventually times out. `CANCELLED_GUARD_CLOSED` follows harness timeout,
not an observed user focus loss (last DOM snapshot is focused=true).
C20/main Firefox/global IBus were restored/preserved, no installation or push.
This result proves R9 receipt retention works at both Space gestures; it does
not establish the separate capture deadline's root cause.

### R10 diagnostic preflight: identify the exact bridge deadline boundary

R9 Space trace `native-control/td121_space_two_toggles-18974305cc27e77c2673/ibus_engine_debug.jsonl`
confirms final receipt at rows 457–460, second receipt consumption/delegation at
487–489 (one-based). Marker 12 appears at 1789355422178740 us and the following
capture's marker 13 at 1789355422185593 us. This 6,853 us difference spans two
separate calls and cannot be used as the capture's own duration. The current
bridge budget is 5 ms. Marker receipt alone does not identify whether Ping,
marker delivery, waiting, or consumption exceeded that budget. Existing expiry
metadata focuses on acquisition requests and does not identify bridge take.
No new retention/ranking loss is observed before the capture's time refusal.

Selected discriminator: extend the existing opt-in admission timing records
with bridge begin, Ping completion, marker emission, wait success/failure,
marker readiness and final take/expired-take phases, all using the existing
nonce and deadline. No user text, new RPC, state owner, queue, timer, retry,
deadline change, candidate work, SafetyGate or verifier modification. Use the
existing nonblocking bounded debug writer. Preserve execution ordering and
lock release/notification ordering; diagnostic calls do not grant authority.
This probe may add trace overhead and is not itself a production liveness fix.
Run the focused IME check and one four-case native measurement on instrumented
bytes. A passing instrumented run alone must not silently erase the R9 timeout;
use the measured phase timings and scoped refusal evidence to select the next
systemic change. No unchanged retry-until-green is authorized by this plan.


R10 metadata-only diagnostic check: **558/558 PASS**, 28.5 s,
`/home/ubu/.cache/lay/development/run-i4v2oh4l/RESULT.json`. The existing
execution contract and strict bridge deadline tests pass. This validates the
probe's compatibility, not the unresolved native capture timeout or latency.


R10 build and graph PASS (179.099 s/43.637 s), all 1,408 source identities
verified in `td121-r10-bridge-timing-20260914/BUILD-RESULT.json`. Independent
probe review found one telemetry-label limitation: `bridge_marker_ready` is
emitted after the matching-slot write attempt even if concurrent expiry removed
that slot. It must NOT be interpreted as proof that `current.ready` was set.
The forthcoming measurement uses begin/Ping/emit/wait/take timing to locate
expiry, treating that unconditional phase only as marker processing completion.
No control-flow/deadline/authority change was found. Correct the label/condition
before final release; a native PASS on this diagnostic build is not the fix for
R9's still-unresolved timeout. This scope avoids inventing a readiness witness
from a telemetry label. Original four inputs, timings and supported client
configuration remain unchanged for the single diagnostic run.


R10 diagnostic native result: **4/4 PASS**, two delegations and both exact DOM
surfaces in every original scenario, including both trailing-Space cases.
Receipts: `td121-r10-bridge-timing-20260914/native-control.json`,
`two-toggle-visible-proof.json`, and `bridge-timing-summary.json` under
`/home/ubu/.cache/lay/development/`. All 67 nonce-correlated bridge groups
have actual successful take records: min 294 us, median 455 us,
max 853 us, no observed refusal or incomplete group. Per-case maxima:
first-word 853 us, mixed 851 us, completion 818 us, Space 633 us. These durations
are the probe's begin-to-taken timestamps, not physical key-to-DOM latency.
The unconditional marker-ready label was not used as readiness authority.
C20/main Firefox/global IBus identities were restored/preserved. No install/push.

**Verdict scope:** the combined restoration/receipt mechanisms now meet the four
native examples on these diagnostic bytes. This run did not reproduce the R9
5 ms timeout and cannot explain or claim to repair that historical refusal.
No evidence supports increasing a deadline or bypassing a fresh fence. Keep
that reliability observation distinct from the 4/4 functional measurement.
Before final release, correct the known telemetry condition and examine the
concrete matching-Bridge expiry cleanup overlap identified in independent
review: `expire_fence` currently preserves any observed marker, although only
Acquisition has a separate ready-activation owner. A controlled timer/ready
ordering test can establish whether a timed-out Bridge leaves the single slot
busy; its relationship to the actual R9 call remains unproven. Do not relabel a
probe PASS as a deadline fix or silently discard the recorded R9 failure.

### R11 preflight: bounded timeout cleanup and completed replay retirement

Two distinct recovery defects are now source-grounded. First, a Bridge wait
may return its timer error before marker processing finishes; `expire_fence`
then retains the matching observed Bridge although it has no separate
ready-activation consumer. The next request can encounter Busy. Second, the
next exact handoff publishes an epoch increment while retaining a completed
prior ExactReplay scope. Its progress distance then exceeds the replay length,
so ordinary Enter can be rejected when the subsequent capture fails. R9's
recorded epochs 10 -> 22 after twelve replay keys, followed by the
source-implied +1 on second handoff, agree with this second mechanism. Neither explains why R9 exceeded 5 ms.

Selected route: use the existing expiry owner to remove only its matching
Bridge regardless of marker observation, retaining Acquisition semantics and
nonce/deadline isolation. Retire only the fully completed, matching local/shared
current ExactReplay through its existing retirement path before the next exact
handoff changes the epoch. Preserve incomplete and contradictory scope refusal.
Also make R10's marker-ready trace conditional on the actual matching-slot write.
No new owner, generation, timer, queue, cache, fallback, deadline or retry.
Alternatives: keep all observed slots (demonstrated Bridge recovery leak), or
clear all observed slots (would erase Acquisition's separately owned ready
result). For replay, unconditional scope clearing would weaken incomplete and
mismatched replay safety; relaxing the distance predicate would let unrelated
epoch changes masquerade as completion. Both are rejected.

Consequences: candidate retention/ranking and model false-authority gates are
unchanged. These changes revoke exhausted transport state and grant no text
mutation authority. Strict capture deadlines, fresh fences, both leases,
SafetyGate and verifier remain. CPU/RSS effects are bounded existing state
checks, no package/model work, cache key/invalidation or reload changes. No
learning/feedback may be emitted by cleanup. Concurrency proof must show stale
expiry cannot clear a successor; replay retirement must preserve owner/token,
tail and receipt identity and reject mismatched/incomplete scopes. Terminal and
atomic consumers remain on their existing routes. The main regression risk is
premature retirement, addressed with controlled positive and negative actual
adapter/legacy callback tests. Maintenance cost is one fence-kind condition and
reuse of the existing exact replay completion predicate, no new source of truth.

Proof plan: controlled real-P2P old-code RED for observed Bridge expiry and
real second-delegation/Enter RED; grouped focused IME proof and two independent
reviews; one new native four-case proof; then canonical changed/full release,
four isolated clients, final-byte native proof and authorized installation/push.
The expiry test forces the existing error-cleanup callback ordering without
sleeping or claiming elapsed-time reproduction. Record each denominator and
R9's unexplained timeout separately. Rollback boundary is these two runtime
cleanup sites plus conditional metadata; existing C20 remains installed until
final acceptance. No runtime authority changes during development.


R11 controlled old-code RED: `run-c0ogcshy/RESULT.json`, 558/560, exactly
`td121_observed_bridge_expiry_releases_only_its_own_slot` (occupied slot) and
`td121_aborted_second_exact_capture_does_not_consume_enter` (Enter swallowed).
Initial patched run `run-m45r2e5u/RESULT.json` passed 560/561; Enter reached the
client, but its new effect collector encountered the existing detached zbus
object-dispatcher's UnknownObject response. Reused the existing exact-serial,
exact-error transport reply consumer for both Enter callbacks; no production
change or effect filtering relaxation. Final focused check **561/561 PASS**,
28.8 s, `run-d7zen3gy/RESULT.json` under `/home/ubu/.cache/lay/development/`.
The four Enter cases cover plain, trailing Space, preserved prefix and completed
expired scope. Separate incomplete/mismatched scope tests retain refusal. The
Bridge test proves two subsequent real fences and stale-expiry isolation; the
existing predecessor test now also checks matching observed Acquisition expiry.
Independent review pass 1 has no production blocker. Final review/native/release
remain pending; this is timeout recovery proof, not the cause of R9's overrun.

R11 final review pass 2: PASS. Build/architecture PASS (178.977/43.797 s),
1,408 source identities verified. Native **3/4 PASS, overall FAIL**:
`/home/ubu/.cache/lay/development/td121-r11-timeout-recovery-20260914/two-toggle-visible-proof.json`.
First word, mixed prefix and completion show both exact transitions. Space has
only one delegation. All 64 bridge calls were taken, 317–704 us; its six bridge
calls took 446–630 us. No timeout occurred in this native failure. C20, the main
Firefox and global IBus are restored/preserved; no release/install/push.

### R12 preflight: retain observed replay history across an interleaved Reset

R11 Space trace `native-control/td121_space_two_toggles-4d33cea9be6bb42c63a4/ibus_engine_debug.jsonl`:
Reset 110/112 after deletion have no nonempty suffix and correctly do not arm.
After four replacement characters Reset 122 arms epoch 20/suffix 4 (row 457).
The delayed empty client surface is rejected at row 459 because R6's historical
surface predicate is completed-only. One more replay letter then has five owned
tail characters but only one post-Reset lineage character; without pending
provenance, pre-Space capture fails its exact-count check. Reset 128/130 after
Space cannot recreate that lost candidate, so the final exact six-character
snapshot never authorizes the second gesture. This is the first shared loss;
R11 recovery changes do not run at that point. Do not attribute scheduling
variation to R11 without a causal comparison.

Selected route: extend the existing inert prior-surface predicate to the already
observed portion of a valid in-flight replay. A source-prefix snapshot must be
between the original tail length and the shortest prefix already reached by
observed deletions. A replacement-prefix snapshot must retain the entire
unchanged prefix and include no more than the already observed replacement
characters. Cursor must be at the end, no selection, same local/shared scope,
owner, layout, exact mirror and epoch. Incomplete replay keeps its existing
expiry requirement; completed replay retains R6's except-expiry behavior. The
pending receipt remains unconfirmed with its current observation revision.
Existing validated append and boundary settlement then advance that same
pending provenance; only the final fresh exact snapshot can grant a handoff.

Alternatives: add a completed-scope capture fallback (could recreate discarded
provenance after contradiction), or ignore arbitrary surrounding mismatches
during replay (would hide foreign content/future surfaces). Both are rejected.
Keep initial empty Reset unarmed; do not relax UnknownStart count, trailing
boundary, current owner/token, predecessor inequality or one-shot receipt gates.
No new cache, owner, timer, queue, fallback or independent source of truth.

Consequences: candidate/ranking/model/package/reload/learning behavior is
unchanged; intermediate snapshots supply zero mutation/display authority. Tail
identity and cancellation still reject selection, cursor movement, foreign
owner, missing shared scope, command gaps and expiry while incomplete. CPU cost
is bounded string/character comparison on an existing snapshot, no allocation
store or model call. Strict deadlines, both leases, SafetyGate and verifier are
unchanged. Terminal/atomic routes retain existing scope exclusion. The primary
risk is admitting a future rather than historical surface; tests explicitly
separate these by observed replay progress. Future package or online updates
cannot alter this transport proof. Removal boundary is the existing prior
surface predicate and its name/metadata label, not a new route.

Proof: controlled actual legacy callbacks with mid-insertion Reset and delayed
deletion snapshot; group both projection directions, plain/trailing Space and
preserved prefixes. Old code must fail retention. Add negative future-prefix,
foreign text, selection/cursor, scope mismatch and active-expiry assertions;
complete the real append/boundary and final Reset/exact snapshot, then actual
VisibleTailV3/manual delegation with no intermediate text effects or learning.
Run focused IME and two independent reviews, then native once on changed bytes.
Full release remains gated on all four native cases; no unchanged retries.


R12 old-code controlled RED: `run-rq0ed_ia/RESULT.json`, 561/562, all six grouped
variants lost pending provenance and final exact/delegation authority. The
initial patched run `run-sjmen2az/RESULT.json` passed 562/563 including both new
proofs. Its one existing assertion expected no pending after a valid historical
delete surface. Updated only that semantic expectation to retained-unconfirmed
for the no-prefix case; the missing-prefix case still rejects. Added explicit
no-handoff/no-display assertions and retained the remaining passive readout,
final exact receipt and one-shot delegation checks. This reflects the written
R12 retention contract, not a weakened text-effect or authority assertion.
Final focused **563/563 PASS**, 29.1 s, `run-7kvhd6mx/RESULT.json` under
`/home/ubu/.cache/lay/development/`. The new positive asserts authoritative
VisibleTailV3 contents and actual manual delegation; eight invalidations and
past-versus-future delete bounds pass. Native/release/install remain pending.


R12 final independent reviews: both PASS. Build/architecture PASS (179.062 and
43.903 s), 1,408 source identities verified. Native **4/4 PASS**, two delegations
and both exact DOM transitions in each original case, including trailing Space:
`/home/ubu/.cache/lay/development/td121-r12-interleaved-replay-20260914/two-toggle-visible-proof.json`.
Native result SHA-256: `a01ddacd252530159bf6eb1ff3c1648206d284e2769016b358749511a3fda7be`.
C20/main Firefox/global IBus were restored/preserved. This accepts development
candidate behavior; final release-byte native acceptance remains required.
R9's historical deadline overrun is still unexplained; neither diagnostic PASS
nor bounded recovery tests establish a universal scheduling guarantee.

### Final transaction evidence boundary

The final build, installed-byte and publication facts are recorded in the
[execution receipt](td121-release-1.0.72-execution-2026-09-14.md). That one receipt
is excluded from Graphify inputs before the final build, as execution evidence
rather than an architecture input. Otherwise writing the accepted binary hash
into an indexed document would change the generated receipt embedded in that
same binary. No runtime source, owning design, test or active architecture
input is excluded. Static owning documents link to this receipt; its updates
cannot grant runtime authority. Refresh and check the graph after final receipt
updates and verify the generated compiled receipt is unchanged. The final
source archive remains immutable; any post-build documentation delta is listed
separately in publication evidence, never represented as retested runtime code.

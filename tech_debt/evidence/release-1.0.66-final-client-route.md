# 1.0.66 final client route — explicit bounded continuation

Date2026-09-07. Status V2_CLIENT_VERIFICATION_PENDING, not release acceptance.
The existing pass2 fixes have independent scoped9/10,H0/M0. Their composition
checkpoint can be accepted separately. TD125 executor is scoped10/10 with
remote440/440. Neither result turns the frozen private client0/5 into a PASS.

## Two independent first breaks

1. The V1 client advertises no SurroundingText with positive terminal geometry,
   but requires DeleteSurroundingText and never interprets DEL. Its consumer
   contract is internally inconsistent. A legitimate successor must declare
   and publish the chosen client contract, keeping original behavior/timing
   assertions, rather than relax positive output expectations.
2. Before any replacement transport, the cold optimized candidate reports
   `prefetch_not_ready`. Factory/bridge/context-marker readiness is not lexical
   readiness: server registration precedes asynchronous exact-authority warmup.
   Source suggests absent frame authority, but no measured refusal reason yet
   distinguishes pending warmup, unavailable package, and rejected certificate.

## Minimal diagnostic before another runtime decision

Add opt-in metadata to existing trace sites only: exact-warmup start/completion
with availability and duration; per-schedule snapshot/certificate/decision
presence, generation, tail revision and existing exact preparation duration.
No user text, packages or candidate IDs are logged. No lookup is repeated and
no RPC, new owner/cache/generation/fallback, await, retry, sleep or deadline is
added. Formatting/enqueue occurs only when existing trace is enabled. This
adds bounded debug overhead and can perturb diagnostic timings; do not call
them original latency acceptance. Production logging policy is unchanged.

Candidate/lattice retention, ranks, verifier/SafetyGate, reload identity,
learning/feedback and stale-frame handling stay unchanged. The warmup result
was previously discarded; observing it must not grant frame authority or
reschedule a failed correction. The existing old/new client routes and
physical-input ownership are untouched. Rollback removes only trace calls.

Options: (1)9/10 observe existing states, then decide; (2)3/10 move warmup into
registration or key handler, risks startup/input stalls; (3)1/10 increase Space
wait or insert test sleeps, changes the contract without identifying cause.
Choose1. Build one exact diagnostic candidate remotely under existing guard;
run it once with declared dependencies. No production installation is implied.

## Maintainable successor provenance under consideration

The immutable original already exists in Git, not just a private cache:
commit `708245298a3f553ac3c52243728c02ba6344a140`, path
`scripts/proof/ime-client/driver.py`, blob
`04f7dbac56c92dbe0238c0754bcb6db4e242256c`, SHA-256
`9ece223f6689323e5cae3fc5f27cf990d37b86dff5e9f6e0ff3212d4dd488750`.
At the V1 checkpoint the active file equalled that blob. Any V2 must explicitly identify
its new proof contract and retain this immutable baseline reference and old
receipts. Avoid a second800-line maintained driver, a new session framework,
or implicit source-slicing loader merely to correct consumer capabilities.
Successor design has been checked read-only; the following limited edits are
now admitted as test tooling, not a production policy change.

## Selected V2 client contract and consequences before editing

**9/10:** evolve the one active driver to an explicitly versioned
SurroundingText consumer. Add the missing advertised capability, answer
RequireSurroundingText and publish actual client visible/cursor/anchor state
on focus and after client mutations. If a printable press is unhandled, apply
its native client insertion once (never again on release). Keep all five
scenario bodies, final text/deletion/identity assertions, original waits and
timeouts. Correct native cursor-event handling only where the client contract
requires it; never infer a commit from an internal Lay tail.

Runner validates exact V2 bytes rather than claiming V1 parity. Run metadata
records the V2 contract/hash and immutable V1 Git commit/blob/actual/normalized
hash. Old saved run files and Git blob remain unchanged; active driver evolution
is explicit. Existing config schema, dependency9, resource limits, private
namespaces, no-global-IBus rule and cleanup stay unchanged. Tests must catch
wrong contract/hash, absent capability/publication, duplicate native insertion,
and mutation of original scenario/timing assertions.

Alternatives: full duplicate driver3/10 (two maintained routes); new session
framework5/10 (unnecessary API/bootstrap churn); changing runtime to emit GTK
deletes to a terminal0/10 (invalid capability/authority). Selected V2 fixes
the actual declared client protocol with no new runtime owner or fallback.

The client is a private protocol consumer, not a GTK widget or terminal.
TD125's real Readline proof remains separately scoped. Publishing snapshots
adds legitimate client protocol traffic and can change observed timings;
report V1 and V2 separately. It must not prime lexical state, grant KnownStart,
inject extra boundaries, sleep/retry, or change cold restoration into a warm
positive. Candidate scoring, caches, reloads, feedback and release authority
are unchanged. Rollback selects the immutable V1 source; false claims of V1
parity after this repair are prohibited. No installation is admitted by V2.

## Measured cold first break and frozen V2 handoff

The diagnostics-only IME source and accepted composition successor passed the
remote union of IME402 and protected-source7 tests:409/409. Evidence:
`/home/e/projects/lay-development-runner/run-xRje9K`; local transfer/result:
`/home/ubu/.cache/lay/development/run-775_yi49`. The optimized diagnostic binary
built in3m11s under the agreed guard. Immutable candidate:
`/home/e/projects/lay-development-runner/release-1066-diagnostic-Ym0t7Y/lay-ibus-engine`,
SHA256 `c8df6be51584b2f8ffaf55922abe7c413f7038c2cd0548141aee7820c962cd1e`.
This is one diagnostic executable, not an accepted complete release bundle.

One unchanged V1 run with that candidate and the nine-role dependency manifest
`ea6c07b1ddd0d504f87578a39a40b552f301c618195ab0e84df9d9acd2f21c76`
completed0/5 with clean private-process cleanup. Receipt:
`/home/e/projects/lay-development-runner/release-1066-diagnostic-Ym0t7Y/client-v1/receipt.json`,
SHA256 `a9af127cce27a4ee7a28e39e43da961ad9335fac5a94dfa8c125f3937302c025`.
The trace orders warmup start, exact preparation with snapshot/certificate/
decision all absent, Space refusal (`not_ready`,3564us lookup), then successful
exact warmup completion (`available=true`,116257us). This proves pending cold
exact authority for this run, not a missing artifact or replacement-transport
failure. Timings include opt-in diagnostic overhead. It is distinct from the
historical seconds-long process-cold Wave/L1.1 admission measurements.

The first V2 implementation was frozen in one active driver, SHA256
`4a536d77f17025f4e9ecc6d55783ffd1d5b2532f3804d58a9d40ce2b2807376a`.
Both V1 and V2 have the exact same `run_cases()` source SHA256
`eea5c141f6a8a10f6f9d6823fe7a526607c9a0f0e5db34dee4da9ad2ae1f98ab`.
Runner metadata declares `lay.ime-client.actual-input-context.v2`, preserving
V1 provenance and the configuration schema. The implementer ran no tests or
builds locally. Next: remote tooling tests and one V2 execution with the same
immutable candidate/dependencies; preserve the V1 failure and report protocol
and temporal readiness separately. No sleep, retry or warm-only substitution
is admitted by this checkpoint.

### V2 first execution: binding error, not product behavior

Remote tooling ran86 tests:85PASS,1optional cgroup skip,0failures,0.954s.
`/home/e/projects/lay-development-runner/run-YrdDmO`, local
`/home/ubu/.cache/lay/development/run-hf5r6gy4/run.log`.
The one V2 client run stopped before any product case: public Python
`IBus.InputContext` has no `require-surrounding-text` signal. Private cleanup
completed; candidate was not started. Receipt
`/home/e/projects/lay-development-runner/release-1066-diagnostic-Ym0t7Y/client-v2/receipt.json`,
SHA256 `c2e23478e3a49e7ec43edf9883cd2cc8943962cbbc7e13a9122f5de3af552e63`.
Guarded remote introspection confirms the public methods are
`needs_surrounding_text()` and `set_surrounding_text()`. String/hash tests did
not detect API incompatibility: their PASS was tooling identity only.

Bounded V2 repair: remove the unsupported signal subscription/handler. At the
existing focus/key publication boundaries require the public needs accessor
before dirty/forced publication. Retain dirty state while no request exists:
libibus caches even unsent snapshots and can suppress an identical later send.
Focus dispatches queued callbacks before publishing. No sentinel state.
No new input, key order, scenario body, wait or runtime change. Test actual
Client methods with controlled transport effects, and validate registered
signals/methods against installed GI metadata remotely. This replaces the
invalid binding rather than providing a fallback or a second consumer route.

The public-API intermediate candidate client (`b6b8cd41...`) reached the first
case with full visible ` ljv ` and no delete, then failed the unchanged positive
assertion. This execution was already started when the reviewer reported the
unsent-cache ordering defect; it is not accepted V2 evidence. Exact warmup
completed108807us after startup, after absent snapshots and Space
`not_ready`3503us. Receipt `client-v2-public-api/receipt.json` alongside the
previous run, SHA256 `7ac6bfc8b6b149ff034a4210dc5f1b9a36a5d4939ceb1c9e0a01cae24b4772b7`.

## Explicit temporal proof split, before implementation

TD121 transfers complete words across context handoff; it cannot fabricate a
lexical snapshot before startup supplies it. The agreed development contract
explicitly distinguishes input-before-ready from input-after-ready. Preserve
both observed cold restoration failures as failures, not waived positive cases.
The next positive proof is explicitly POST_EXACT_READY, not cold acceptance.

Selected9/10: one explicit runner startup-schedule option, default immediate;
post-exact-ready observes the existing candidate warmup-completed trace once
at the first context-ready setup, before the first text key. A private Gio file
notification and one bounded2500ms startup deadline (within existing52s hard
deadline) consume the exact completion record; no sleep/poll, repeated case,
lexical priming, added boundary, runtime query, or changed key/Space budget.
The batched trace can add approximately1s observation delay: report startup
observation separately and never use it as production readiness latency.
All five scenario bodies and output/negative assertions remain identical.
Runner metadata and receipt label the schedule. Immediate stays immediate.

Alternatives:3/10 delay registration until warm (production startup regression);
1/10 repeat the cold run or sleep until it passes (conceals timing). Runtime
authority, caches, reload, feedback, ranks, safety and process ownership are
unchanged; only a private proof observer is added. Missing/unavailable warmup
fails the post-ready run before text input. This is not cold correction PASS,
not a real GTK widget or physical keyboard, and not automatic release admission.

Current V2 candidate after the single binding/lifecycle repair and explicit
startup-schedule addition: driver SHA256
`8e520d3a7a1d90aac9a00f3ec180ada70e3d6d4b125311079545752561277c7e`.
The earlier4a536/b6b8 hashes identify failed historical runs, not current
acceptance. The73d135 intermediate source was not executed as a client.
Rust runtime source/candidate remain unchanged. Six actual-consumer method
tests and four controlled startup-observer tests now complement driver identity
and installed-GI metadata checks. Post-ready behavior remains untested here.

## Post-ready measured result and scoped independent review

The frozen8e520d3a source passed remote tooling in
`/home/e/projects/lay-development-runner/run-0ayT63`, local
`/home/ubu/.cache/lay/development/run-vhqrv4dq/run.log` (exact count in log).
One explicitly post-exact-ready invocation used the same immutablec8df6be5
binary/dependency manifest. Receipt:
`/home/e/projects/lay-development-runner/release-1066-diagnostic-Ym0t7Y/client-v2-post-ready/receipt.json`,
SHA256 `65c6c5f58032e8bb254ccd44de0e9d7bd7b988ee48f807917e52f8cf1d547fdd`.
Candidate exact warmup111953us; batched-event observation997234us. These are
different measurements. File creation/flush was observed successfully.

First complete actual-client case PASS: same canonical context/new engine
retained full ` ljv`, deleted exactly `ljv`, and delivered visible ` дом ` with
left separator preserved. The next case failed during setup, before its text
keys: `InputState` returned `another context fence is pending`. Integrated
denominator1/5, not client acceptance. Private daemon/candidate cleanup clean.
The trace shows automatic layout sync after confirmed correction overlapping
the next field's focus and explicit engine recreation; acquisition stayed
pending and both readiness Shift callbacks were refused. Do not infer the
root cause from the error text alone or retry the same schedule to green.

Fresh-context agent `ime_v2_contract_review` independently accepted the bounded
V2/tooling and two observational runtime traces:9/10,H0/M0. Unsupported public
signal and pre-request cache ordering were repaired in the one bounded pass.
Reviewer withdrew the speculative missing-file monitor and EngineReady issues
after examining actual execution. No general TD121 runtime re-review occurred.
Its source score does not promote the1/5 integration result. Next boundary is
the demonstrated acquisition/bridge-fence interaction, with no increased
deadline, extra readiness key, retry, weakened negative or bypassed verifier.

## Bounded acquisition continuation: consequence analysis before runtime edits

Observed: activation4 becomes source-free-ready, then CreateEngine67 precedes
its source handler installation; target lifecycle arrives, and no subsequent
source seal/target installation appears before the setup refusal. Source
inspection finds two connected premature removals: `open_factory_request`
unconditionally clears the consumed source-free binding still needed by
`bind_source_free_tail_epoch`; `begin_activation_request` unconditionally
discards an earlier ready activation, even if it is the still-current source
required to seal the next handoff. A completed fence also remains in the
pending slot although its result already resides in `ready_activation`.
This is a causal hypothesis until the controlled old-source regression runs.

The existing test `source_free_epoch_binding_survives_next_factory_before_source_handler`
was added as a RED probe only: publish source-free grant, receive/bind next
factory, then install the original current empty-tail epoch exactly once.
It does not change runtime, synthesize client text or grant KnownStart.

Options:9/10 retain only the current uninstalled grant until its source handler
consumes it; separate completed result lifetime from the existing pending-fence
slot (no new slot).5/10 abandon every such transition to source-free UnknownStart
(safe but loses the intended complete-word continuity).2/10 wait in key/factory
handlers or add another receipt queue (latency and ownership complexity).
Select the first only after RED proof and bounded independent analysis.

Candidate sets, rank, lexical snapshots and verifier/SafetyGate stay unchanged.
Every retained result must still pass full revocation/owner/activation/lineage
validation at take and install; a new request never promotes an obsolete result.
No new generation, cache, owner, fallback, wake loop or key deadline. The
existing bounded ready result may live until source installation, but cannot
survive actual revocation/cross-context change. New target readiness requires
source settlement, so a second ready result cannot overwrite an unconsumed
valid source result under the admitted ordering. That last claim needs a
controlled transport regression, not an assumption from separate unit tests.

CPU/RSS remains one pending fence plus one bounded ready result. Clearing only
completed pending state must preserve exact-nonce cleanup so an old timer or
failed waiter cannot clear a new acquisition. Preserve marker-before-seal and
late-expiry tests. Source-free epoch must bind once monotonically; factory
receipt alone cannot authorize text, feedback, suppression or an edit. Package
reload/config/frame invalidation remains separate. UnknownStart stays unknown
until an actual boundary. Source and target callbacks may run in either order;
foreign lifecycle revocation must defeat both old and new work. Rollback is
the bounded source diff, with no data/service migration. This continuation is
explicitly replanned from a new actual-client first break, not an unreported
third general review of the previous four repaired findings.

### Frozen baseline full-gate observation

The pre-continuation snapshot ran all32 selected Cargo targets:2637 tests,
0test failures,357.675s summed target execution (IME402 in5.740s;
daemon239 in141.99s). This excludes the subsequently added local RED probe.
Full-gate log
`/home/ubu/.cache/lay/development/run-vhqrv4dq/full-release-gate.log`, SHA256
`c85ce55c97185a5b8f8561cefb72f96bff7a83fe3fdcb2675925534da8c79dbf`.
Remote raw per-target logs remain at
`/home/e/projects/lay-development-runner/workspace/target/test-lanes-results/logs`.
The gate then refused the outdated known-failure-ledger manifest binding, so
there is NO final SUMMARY/PASS and lint/release build were not reached.
The ledger has0failures and must remain empty; final discovery must rebind its
manifest SHA together with the new manifest. No new failure allowlist is
admitted. This is manifest bookkeeping, not permission to accept a test failure.
Shared target8,411,738,112 bytes plus default canonical lane cache1,492,615,168
bytes remained below12GiB in aggregate at the measured checkpoint. Architecture
refresh before this gate passed and was copied back; later test/docs make that
receipt historical until the final remote refresh.

RED is now measured: remote403selected,402PASS,1FAIL at the exact late epoch
binding assertion, run `/home/e/projects/lay-development-runner/run-9CBD6R`;
SUMMARY SHA256 `e1a6f726ed62613a022a780d99902be1e0b106273ef1162722c51a09a58f3c7d`.
Independent bounded analysis found no veto to the minimal repair, with these
required constraints: preserve only Consumed source-free binding; prune ready
outcomes only after the new reducer transition and only if full identity is
stale; never hold the ready mutex while locking reducer; prevent overwriting a
still-current ready result. Acquisition `complete_activation(fence)` must
continue recognizing the exact completed receipt after its pending slot is
released. Bridge fencing remains unchanged. Cover predecessor-install/next
factory order, stale result rejection, pending-slot release, exact finish and
wrong fence, old timer/new nonce and existing bridge negatives. These are the
admitted bounded implementation/proof obligations, not additional runtime APIs.

Initial lifetime implementation passed412/412 (IME405+protected7) remotely at
`/home/e/projects/lay-development-runner/run-uvlsS0/tests/SUMMARY.json`, SHA256
`b5d394eac18ce78af4a4e5a9245385165d6af5bf67873202f6daa281000d4490`.
The source-free RED and two new real-adapter schedule/API tests pass. Bounded
review found H0/M1 (7/10): a next begin could mutate reducer between publishing
the ready result and clearing its old pending fence, then Busy could revoke
the transition. This is fail-closed but loses handoff liveness.

One repair pass: use the EXISTING pending mutex as the publication transition
lock. Begin holds it over its synchronous reducer transition; refresh holds
it from checking/consuming the matching reducer request through ready-result
publication and exact pending clear. Release before tracing/notifying and
before stale-ready pruning. Lock order is pending→reducer; reducer is dropped
before ready-result lock. No await, external I/O, new mutex or queue is added.
No racy precheck or arbitrary busy retry. Existing stamp/bridge paths must not
acquire pending while holding reducer/ready; independently audit this order.
Earlier412PASS does not include this last synchronization delta.

The single repair also rejects an occupied pending slot under that same mutex
BEFORE reducer mutation. The exact-finish regression now attempts a second
begin while the first acquisition is pending, asserts Busy and unchanged
request/nonce, then completes the original acquisition through the real P2P
observer. This covers begin winning the publication lock without a racy
precheck, extra queue or retry.

Final synchronization delta:412/412 PASS at
`/home/e/projects/lay-development-runner/run-7j9HLR/tests/SUMMARY.json`, local
`/home/ubu/.cache/lay/development/run-udmxvaa6/run.log`; guarded worker14.076s,
focused command11.186s. Prior run `run-yfkbzY` had411PASS/1FAIL because the new
test inspected the pending slot before its detached arming task had emitted
the marker. The corrected test advances the controlled P2P marker, then
attempts Busy before observer consumption; no sleeps or runtime change.
Bounded independent lifetime review:9/10,H0/M0 after the single repair pass.
This is source-contract acceptance, not actual-client or release acceptance.

Evidence retention correction: a subsequent dev-check refreshed its disposable
mirror and removed the default full-gate target/log directory before archival.
The full stdout log/hash above survives; the old per-target baseline logs do
not. No successful release receipt existed. Focused run logs outside the mirror
remain intact. Final full-gate output must be archived outside the mirror before
another snapshot refresh; the outdated raw-log pointer is historical only.

## Next actual-client first break: observer cancellation

Optimized lifetime candidate `fc97e56917a91d43cdff1ee9fbba69f15610dba2ccae34e6150df9c0a26295fc`
built remotely in2m57s. The unchanged V2 post-exact-ready five-case schedule
now completes2/5: full `ljv -> дом` with preserved separator, then distinct
US->RU mixed `lом` retention without edits. Third-case setup fails before its
text with `metadata observer cancelled`; runtime trace records the observer
stopping with `context admission denied` after FocusOutId117/FocusInId118.
Receipt `/home/e/projects/lay-development-runner/release-1066-final-IFSMv9/client-v2-post-ready/receipt.json`,
SHA256 `74ed0978b205beb73b5de66abb0127723ba45140dbd2c5e9b1a5deba62f700d2`.
Private daemon and candidate cleanup complete. This is not5/5 acceptance.

The first denied method is currently absent from the trace; factory admission
overlapping a pending/reflexive activation is a hypothesis, not an established
root. Before another authority change, extend only the existing opt-in refusal
trace with method/serial already available at ingress, and use controlled
P2P reproduction. No text, new RPC, timer, worker, retry, model/rank/feedback
change or SafetyGate relaxation. Trace runs after the reducer guard is dropped;
disabled tracing has no formatting/allocation. It neither admits a rejected
event nor keeps a cancelled observer alive. Runtime recovery alternatives
remain deferred until the rejecting transition is identified. Diagnostic
bytes are not release acceptance and prior failed receipts stay immutable.

### RED: late native identity only enriched SourceFree, not Transfer

The metrics-profile diagnostic a4eac0b5 (1m11s build) completed1/5 and stopped
earlier at missing Barrier after legacy FocusIn96 and native FocusInId100;
no observer cancellation was recorded in that distinct schedule. Receipt
`release-1066-final-IFSMv9/client-ingress-post-ready/receipt.json` under the
same remote root, SHA256
`51b19e590252ea8d97f66cf9ed542afcfd1fe636dbe4a6fe1220e807f5767952`.
This is diagnostic evidence, not a rerun accepted by selecting a better score.

Source first break: `enrich_pending_native_activation` only recognizes
SourceFree. A valid current Transfer request plus delayed native identity is
not enriched, and the caller attempts a second begin against the original
already-focused ticket. A retained consumed source-free receipt can additionally
make the unrelated enrichment inquiry revoke the Transfer outright.
Remote RED test
`context_admission::tests::pending_transfer_native_receipt_enriches_without_rearm_or_losing_ordered_marker`
ran1 test,0PASS/1FAIL at the enrichment assertion,408filtered; no runtime repair
was present. Inner command: `scripts/cargo-guard.sh test --bin lay-ibus-engine
pending_transfer_native_receipt_enriches_without_rearm_or_losing_ordered_marker
-- --exact context_admission::tests::pending_transfer_native_receipt_enriches_without_rearm_or_losing_ordered_marker
--test-threads=1`, in the existing guarded remote workspace.

Options:9/10 extend the existing exact-request enrichment to both request
targets;4/10 discard the pending transfer and reacquire empty UnknownStart
(loses the prefix);2/10 wait for native receipt before running legacy callbacks
(new scheduling dependency and input latency). Select the first. It uses the
same request generation, nonce, validated context reply and ordered marker.
If Get already supplied the same context, preserve its validated receive
position: replacing it with a later native position would incorrectly make an
already observed marker precede the reply. If no reply exists, feed native
identity through the existing `context_reply` transition, including its
different-field -> SourceFree/UnknownStart rule. Explicit same-request
contradictory context/epoch still revokes; unrelated target inquiries do not
mutate the current request. Full source seal and marker remain indispensable.

No candidate/rank/model/package/learning policy changes. Transfer retains only
the same exact context/owner/revocation-bound tail; SourceFree never inherits
text or KnownStart. No allocations beyond existing bounded context identity,
new queue/cache/controller, retries, input wait, deadline change or fallback.
The existing compatibility future notices Native and stops awaiting Get; its
late cleanup remains exact-generation-bound. Required negatives: wrong target,
contradictory native context, stale request/revocation, late native after marker
before source seal, one-shot consume, and same-field actual handler route.
Rollback is the bounded enrichment diff; foreign lifecycle and bus-owner trust
checks remain unchanged. No claim that this explains the untraced cancelled
method in the earlier2/5 schedule until the repaired actual route is measured.

Native enrichment core GREEN:8/8 selected tests,403filtered,0.05s execution,
4.13s compilation. `release-1066-final-IFSMv9/native-enrichment-core.log`, SHA256
`4f5b5705c6cbd0c7f8d7fd36e98dabcc595191a8b3cebb8b02b7c17bc574bb48`.
Metrics candidate999f1ced builds in6.05s, actual2/5; receipt SHA256
`70e082296d4cb01e84357509e0edf9801dc2c0aa6c60e5c05ed9b9a542179197`.
Its new trace identifies denied CreateEngine120. No release acceptance.

### Factory overlap: measured state and bounded supersession analysis

Further diagnostic54599787 records factory ingress state outside reducer lock:
CreateEngine67 encounters `source_free_pending`, owner=None, activation=None,
and stops the observer. Actual1/5; receipt
`release-1066-final-IFSMv9/client-factory-post-ready/receipt.json`, SHA256
`14595c2518445dea5d5473baa6ee0d37b18a5195cc7a436d7cb20a606e769949`.
The different failing case index reflects event ordering, not a different
mechanism: a new factory can arrive while the preceding empty field activation
has no admitted owner yet. Diagnostic candidate05b19bf8 was compiled but never
executed; the used54599787 emits its diagnostic after dropping reducer lock.

RED `next_factory_supersedes_unadmitted_source_free_without_borrowing_authority`:
0/1,411filtered, at the later-factory admission. Log
`release-1066-final-IFSMv9/source-free-supersession-red.log`, SHA256
`fdf13ac7e461db8a406d8b90257775cbcce0a8395aac7d5115e6ba56504b9bca`.

Choose9/10: let the existing reducer supersede ONLY an unadmitted SourceFree
request with no owner/activation and a strictly later factory position. Revoke
its nonce/revision, drop only that Pending/Ready empty activation, then open
the existing SourceFree factory reservation. Alternatives:3/10 permanently
cancel the metadata observer (current fail-closed loss of service);2/10 add a
factory wait/queue (new owner and callback latency). This is not permission to
discard an admitted source tail or a pending Transfer. Consumed SourceFree bind
receipts and established owner/activation are excluded. Foreign/unverified
mode, wrong ordering and malformed headers still deny.

Second-order requirement: a late reply from an OLD request generation must
return false without revoking a newer request; same-generation wrong nonce,
epoch or conflicting reply remains a contradiction. Existing exact-generation
failure cleanup and exact-nonce fence expiry remain unchanged. Observer's
normal refresh clears the now-obsolete pending fence, with no new retry.
Negative proof covers old reply/timer, successor request identity, one-shot
UnknownStart grant, forbidden inherited tail, and retained consumed receipts.
No rank, package, cache, feedback, output policy, CPU worker, timeout or Space
budget change. State remains one request/fence/ready slot. Allocation is the
existing bounded factory reservation; rollback is this reducer delta, no
data/service migration. A real P2P regression covers the native enrichment
path independently; actual five-case validation and scoped review remain gates.

Combined source checkpoint417/417 PASS (IME410+protected7),4.46s compile,
5.706s test execution. SUMMARY `release-1066-final-IFSMv9/native-factory-tests/SUMMARY.json`,
SHA256 `71022d4b0025298d644fb6220f4b5e11fa92f00de286dd567843ea373815b925`.
Independent bounded review9/10,H0/M0 covered native enrichment and reducer
supersession; the subsequent actual client found a remaining publication race,
so that source score is not final integration acceptance.

Single supersession repair pass: dd44509b metrics candidate reaches2/5,
CreateEngine120 now succeeds and the observer stays alive, but new target125
meets Busy. A superseded future can arm its old fence AFTER factory observer
refresh found no pending slot. Therefore cleanup alone is insufficient:
`arm_fence` must check the exact current acquisition generation/nonce/target
and lifecycle revision before publishing under its existing pending mutex.
This is the publication side of the existing invalidation contract, not a new
queue or owner. Lock order remains pending->reducer, no await/trace under that
critical section; Bridge arming is unchanged. Wrong/stale acquisition cannot
occupy a slot, and old cleanup remains exact-nonce/generation. No deadline or
candidate/word authority changes. Controlled delayed-arm RED/GREEN is required
before another actual-client candidate; do not reuse417PASS for this delta.

### Current checkpoint: repaired actual client5/5, release gates still pending

Delayed-arm RED failed exactly at stale-slot admission (0/1,413filtered), log
`release-1066-final-IFSMv9/stale-arm-red.log`, SHA256
`cd72fb050f27e9499e6e404dbd6ee8f4577716da573ba4ec07521e288e38d31c`.
The single repair validates and stores while holding pending->reducer; no
reducer mutation can intervene. Bridge path is unchanged. Expanded regression
also refuses wrong target/nonce and completes the successor through a marker
as SourceFree/UnknownStart once. Combined418/418 PASS,4.46s compile,5.739s
execution. SUMMARY `release-1066-final-IFSMv9/stale-arm-tests/SUMMARY.json`, SHA256
`2184ab00e4577153fee2cb4a76a892b5ca40a5bc0d8e7f7febc32e4eeef059ea`.

Metrics candidate
`d9920add9dd7253c327e0debd24384b912ab5dea033e9f7bc98edd6dd61cf88b`
built in6.11s and completed the unchanged V2 post-exact-ready client5/5:
US->US full correction with left separator; US->RU mixed retention; reverse
mixed retention; different-field negative; UnknownStart negative. Client
service2.722s/CPU2.995s includes startup/trace drain and is not a key latency.
Receipt `release-1066-final-IFSMv9/client-stale-arm-post-ready/receipt.json`, SHA256
`c2f86ff3e69ad8ae5c6a2b1435a2584fff67dcc72d449d1be14e09236b0a56b8`.
Private daemon reaped,0candidate processes remaining. All paths in this
checkpoint are under `/home/e/projects/lay-development-runner/`.

Verdict is PASS for that diagnostic candidate and explicit post-ready
five-case contract. Mixed correction itself, cold immediate startup, physical
keyboard/GTK, full release-profile bytes, canonical/full gates, installation
and push are NOT promoted by this result. The older failed receipts remain
historical; final optimized release must be accepted on its own exact bytes.

### Lint-only production-surface narrowing

The post-client lint cleanup gates test conveniences with `cfg(test)` and
removes APIs or enum state with no caller or constructor. It adds no allowlist
or baseline exception and changes no production admission, text authority,
observer, acquisition deadline, Space deadline, output, feedback, or fallback
behavior. The rollback boundary is only these declaration/import narrowings;
remote lint and release gates remain responsible for verifying the final
optimized bytes.

Clippy completion is mechanical: Copy values replace redundant clones, an
Option early return uses `?`, an already-borrowed tail loses an extra borrow,
and the unused trace expectation is removed. Private deferred-layout variant
names lose their redundant Switch suffix; no wire name or serialized schema
changes. Existing explicit protocol argument lists retain their ABI and use
narrow lint expectations with reasons. These edits do not change candidate
retention/ranking, caches/packages, feedback, deadlines, allocations or output
ordering. Test lock scopes must end at the same explicit pre-await boundary;
do not move a runtime lock or change the tested schedule to satisfy a lint.

The composition helper's explicit authority argument makes eight parameters.
Selected9/10: one function-level `expect(too_many_arguments)` with a reason,
no executable change, followed by a newly reviewed TD121 byte binding and
protected functional revalidation. Rejected4/10: bundle booleans into a new
runtime options type during final release; rejected3/10: module-wide lint
suppression merely to keep the old hash. The prior9d94b7e7 checkpoint stays
historical; no new hash is accepted before the scoped review and named tests.

Final mechanical lint checkpoint: default inventory535/research359, no new
logical dead-code entries; both Clippy scopes report non_dead_diagnostics=0.
Gate PASS, `lints-final-mechanical.log` SHA256
`753bb38a94b4a5c9b4c596866ecf662de6f1e61f73b376ec7d298e2fce9aa916`.
Updated inventories change locations only. Exact final IME411/411 PASS,
4.21s compile and5.777s execution; `ime-lint-final-tests/SUMMARY.json` SHA256
`c122443446e0618c6cdfefc6b1ad480c9254b268e850b5d0391f84ef461c9baa`.
Both named actual-Tab KnownStart acceptance and UnknownStart no-output/no-
feedback tests are explicitly present and PASS. Paths remain under the remote
`release-1066-final-IFSMv9` directory. No live/release acceptance is inferred.

Final composition source SHA256
`33739ccb18d07fd7206f4b98e605f5e45ec5fe3041e6033f4f70190397f01d5f`.
Removing only lines215-218 reproduces the exact prior9d94b7e7 digest; all
executable text is unchanged. Independent reviewer approved that exact
function-level annotation and successor-only rebind after the named tests.
TD120 predecessor remains immutable. Canonical protected check still follows.

Canonical changed gate now PASS:2646/2646 correctness+package cases,
0semantic and0infrastructure failures; manifest2672 includes15ignored and
11separate performance cases not executed by this gate. Lane349.037s,
including daemon239/239 in141.664s, library1757/1757 in127.341s,
IME411/411 in5.573s and final protected7/7 in0.030s. Archived SUMMARY
`changed-final-results/SUMMARY.json` SHA256
`7714e1f598399bc403cfecf0271d36a97559150e27c630185b96abdfd61c462d`.
The changed gate also completed its remote historical-action replay/unsafe-
edit checks; those are not current local keyboard proof. Architecture refresh
PASS binds source fingerprint
`2fe001cd88e9631a8d13bc2f5cfe78db2603690fa1b0726bdedf70f924b933a5`.
The full release gate runs next on this frozen source, without mirror refresh.

### Final release-profile proof and delivery preparation

Full gate PASS:2646/2646,0semantic/0infrastructure failures; test action
349.325s; release build1m26s. SUMMARY `full-final-results/SUMMARY.json`,
SHA256 `38b4ebc01dc0f8ab02aa730cc5f08737238fae0b7c47068dbd93dc7bd421e90e`.
Full log SHA256 `ab4fb126c23ad8609e0b7b59b8fd626fd84be5219dee89321238efcf2415034e`.
Both result trees were archived outside the disposable mirror before reuse.

Final release-profile IME SHA256
`dfeb50e88c8a9170137563ddaf09ae3a44eb8a5441899e3d399baf60fd8e5185`.
Unchanged V2 client, explicit post-exact-ready,5/5 rc0. Receipt
`client-release-post-ready/receipt.json` SHA256
`80978ba5382d14a29a896eaff0b5d069aa5a17eeadb0815a84e3f08a6a26a9dc`.
Service2.754s/CPU2.980s, including startup/drain; not key latency. Cleanup
reaped private daemon and left0candidate processes. Same five-case scope:
one full same-field correction, two mixed-retention cases, different-field
and UnknownStart refusal. Mixed-token correction itself remains UNKNOWN;
cold immediate startup and physical GTK/keyboard are not promoted.

Final canonical exact V13 compile ran remotely with the final compiler:
compiler SHA256 `99d1f51a8e7648cf2161b3732aa2120db7e0efb93bce11831ce07dc1d79447b6`,
sidecar SHA256 `f116a230fc05a04c375bcddf1c85169276c6d149295eed8ccf93e37672a907b9`,
sidecar2460144bytes. Compile report `release-sidecar-compile.json` SHA256
`088b0d25f2b01f20e8b75ce41a82d52f57f0a3dd21cae143b20965ed7b5251ae`.
An initial stdin-fed packaging command was a no-op across the resource scope;
it created no bundle and is not compile evidence. The subsequent explicit
`prepare-bundle.sh` command completed and its `release-bundle.log` records
version, exact successful operation outputs, hashes and byte counts.

Cargo metadata reports14binary targets, not19; nineteen counts installed
files. Fourteen binaries+sidecar+receipt transferred with16/16 hash parity.
Prepared local `target/release/lay --version` is1.0.66; installed remains1.0.65.
Rollback snapshot ready at
`/home/ubu/.local/state/lay/release-backups/1.0.66-preinstall-b4bd39`.
No production restart, install, commit or push. Physical acceptance still
requires user input; do not label TD121/TD125 DONE or waive the pre-install
physical gate. Next action and exact cache paths are in the execution document.

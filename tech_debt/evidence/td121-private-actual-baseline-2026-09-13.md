# TD-121 private actual-IBus baseline and schedule comparison

Owner: [TD-121](../121-preserve-word-across-ime-layout-handoff.md).

## Current startup-gate checkpoint — 2026-09-13

The current proof source is base commit
`6baa65442227bb31caf32cd56930d9e51559ca71` plus an uncommitted seven-file
Rust delta in `src/debug_log.rs`, `src/nanda_wave/mod.rs`,
`src/nanda_wave/l2/hot_memory.rs`, and
`src/bin/lay_ibus_engine/{trace.rs,precognition_worker.rs,space_autocorrect_prefetch.rs,server.rs}`.
The delta is pinned by archive SHA-256
`a2820c25cf2e1c484e02f81c1a5078147774d67d50131ac12a81c8d10514c476`,
request SHA-256 is
`1f2f903e38980a4912c10028dbbb51399bdfbf8aabe33fb9ae7e4d88f431ab1d`,
and source-manifest SHA-256 is
`ae5100664b5663b38065a24bf09e4628826e825d8a3eb6e6d48f17299701fcb2`.
All1409 frozen files matched before and after the current actual-client proof.
The dependency manifest remained
`1092cd2f72ddd3ae52da6436ceba47f4e26814f137eded46515d3667d3b38314`
and all nine declared files validated before and after.

The implementation uses one5000ms absolute deadline to collect completion of
the existing exact and L2 one-shot work before public factory availability.
Config is loaded before that work. The bounded L2 worker starts the existing
IME lifecycle and then completes L2, keeping a synchronous spawn fallback off
the server task. Exact `None` and completed unavailable L2 capability preserve
the existing literal/`ABSTAIN` behavior. Spawn failure, channel loss,
conflicting completion, or deadline expiry returns a bounded startup failure
before publication. In the release profile `panic = "abort"`; a worker panic
terminates the process before publication and is not guaranteed to become a
structured zbus/channel-loss error. The disconnect unit represents unwind or
channel absence, not release-abort conversion.
L1.1 service readiness is not part of the gate. The4ms Space,150ms preedit,
1ms rendezvous, and5ms acquisition budgets are unchanged, as are candidate
ranking, SafetyGate/verifier authority, cache generations, package bytes,
learning, and reload ownership.

Focused discovery found498 tests:495 correctness tests were selected, executed,
and passed; three performance tests were excluded. The discovered manifest is
`/home/ubu/.cache/lay/development/td121-startup-gate-20260913-bgytxm2w/focused-results/DISCOVERED_MANIFEST.json`,
SHA-256 `0f308bbc4fbbf189a4374cc4b079b74ae76e47254e6ae6a731c87c87de567616`.
The summary SHA-256 is
`1976dde01fd1e7dbc23eccc7a2ea8851db32f7f114a1eb9b95d9e74f9e0c533e`;
the focused log SHA-256 is
`ba1dcdbc70bcef391d4c3540e897d09b82c7f0fecc8dbe9759baea0e368a3a78`.
Canonical drift is exactly these five additions, with no changed or removed
test identity:

- `startup_gate_accepts_completed_unavailable_exact_capability`;
- `startup_gate_reports_worker_loss_before_publication`;
- `startup_gate_uses_one_absolute_deadline_after_partial_completion`;
- `startup_gate_rejects_conflicting_worker_completion`;
- `startup_gate_rejects_completion_observed_after_deadline`.

The optimized candidate used by the current client proof is
`lay-ibus-engine 1.0.71`,7840096 bytes, mode0775, SHA-256
`7fe142796cc0065541cecaea5c049efc04c3877ff8ba3d132a8028ea1dd0e510`.
Cargo dep-info SHA-256
`7234279ddba5b16b3b3ae300985d814a713118d0f913815e44608be2ec1562cf`
names the frozen bgy source root; the matching fingerprint SHA-256 is
`33229ec46317f42c25ffdb55b6e8ef00e267e5b685037d616d6cc85b2f07730e`.
The toolchain was rustc1.97.1 `8bab26f4f` and cargo1.97.1 `c980f4866`.
The original release log ends with Cargo's completed `Finished` line and has
SHA-256 `b2b5aca9debe172ed4da11ea9899d81277ca0bd4609b8f5fdb096d54bfe19d0c`.
Its command return code is nevertheless **UNKNOWN**: the parent runner lost
the command row to a broken stdout pipe after the subprocess returned. The
log marker and artifact binding must not be relabelled as a recorded rc0.

The immutable continuation reused those exact bytes without another focused
run or build. Result:
`/home/ubu/.cache/lay/development/td121-startup-continuation-20260913-bgytxm2w-c1/RESULT.json`,
SHA-256 `d437dd398a403201e7971e4023f4f27bcc46516c3909be12035c56a5fd522a80`.
The fresh config SHA-256 is
`1c81328d929c9d58dc47969f25f1b793de87928581d2cd190950a4c1ce0a259d`;
only `candidate.path`, `candidate.sha256`, and `execution_lease_id` differ from
the bound template. The unchanged immediate outcomes are:

- manual3/3, receipt SHA-256
  `a63e912d960a423b6bc39d3b725e059da33ed4f6acfcca69526c50b316983dc3`:
  both eight-round terminal conversions and the post-handoff visible preedit
  passed. Both terminal cases start with a leading boundary (` ghjd` and
  ` a `), so they measure KnownStart/committed-tail terminal and boundary
  effects, not UnknownStart first-word observed-suffix authority. The third
  case follows16 toggles and is not a fresh-process first-prediction claim;
- restoration5/5, receipt SHA-256
  `6919ab2f256789ac2a6f16e1dd82051a0ec041ec7d99a13bdafa553d2790adbe`:
  exact ` ljv` retention and ` дом ` correction, both mixed-retention cases,
  and both UnknownStart/different-field negative cases passed.

Both receipts report `private_daemon_reaped=true` and an empty
`candidate_processes_remaining`; the outer lease was released. Runtime and
installed authority stayed unchanged. The guard's own final status log reports
`8594317312` target bytes against a `12884901888` budget. A separate later
filesystem `du -sb` observation was `8571611097`; it uses a different method
and is not the guard-log measurement.

The c1 traces put the full startup join at1123516us for manual and1124075us
for restoration. Exact completion was109717us and117695us respectively.
Candidate memory was published before the join; both L2 procedure callers
reported live-readout completion before the single `ibus_startup_warmup`
completion. In each trace the first factory acquisition event occurs after
that completion. Current source then installs `IBUS_FACTORY_PATH`, requests
`IBUS_ENGINE_NAME`, and serves the session bridge in that order. Only the
warmup-completion→first-factory ordering is directly trace-timed; name and
bridge order is a source fact. These two approximately1.12s observations are
samples, not a distribution or p95.

Fresh review pass1 accepted the seven-file source change at **9/10, High0,
Medium0**; no code repair or second review is required. This is source
acceptance only.

The old immediate product FAIL remains intact in the sections below. The new
3/3 and5/5 results show restoration of these fixed protocol-client cases; they
do not prove general answer quality, package quality, startup latency, GUI/GTK,
desktop input, or physical keyboard behavior. Quality remains `UNKNOWN` and
physical input remains `NOT_TESTED`. The historical first-word2/2 baseline on
the base source remains separate; a current-delta first-word rerun is pending.

Two continuation/administrative incidents are recorded separately from product
results. First, the parent bgy SSH stdout was interrupted; the release command
completed far enough to leave the full log and bound artifact, but printing its
row raised `BrokenPipeError`, producing immutable parent result SHA-256
`7d0e623d40d862e16b2f0bff0e954e106d11e5352b051817b4527af3963734ed`
and leaving both client lanes not started. Second, the first c1 outer invocation
passed a mistyped parent directory and exited on `FileNotFoundError` before it
read a request or staged a candidate; no Cargo or client command ran. The same
admitted c1 runner was then invoked with the correct parent path. Neither
incident changes a PASS/FAIL denominator.

## L2 package-absence consequence and implementation history

### Accepted L2 absence scope — 2026-09-13 02:51 UTC

Tested: the private absent control ran the exact candidate and captured a
release panic in `lay-l2-ime-startup` at the strict lexical-phase
`default_memory().expect`; no first key was reached. Corrected-off 3/3 and the
existing on/manual/restoration evidence remain separate and unchanged. Unknown:
whether every lazy first-key L2 caller already handles missing material; the
startup failure cannot prove it. Receipt and stderr remain in the immutable
02:42 primary packet. Runtime authority did not change.

Accepted implementation scope: make the single internal
`surface_motif_memory()` boundary return `Option<&LexicalPhaseMemory>`, then
handle absence at every compiler-forced production call site while preserving
loaded candidate/ranking behavior exactly. Warmup completion, material
availability, and candidate-ready are separate facts; unavailable material
must report zero measured stats plus `available=false`, never fabricated
memory. No panic catching and no parallel strict runtime getter. A meaningful
fixed proof must cover startup survival and first printable/Space literal
behavior with packages absent. C28's cfg(test) underscore path fix joins this
snapshot. Before implementation, context phase, installed L2 field, and live
candidate-gate warmup are audited for their own absence behavior.

### L2 absence implementation checkpoint — 2026-09-13 03:01 UTC

Implemented, not yet executed: `surface_motif_memory()` is now the sole
optional lexical-memory boundary. Every production caller either returns an
empty candidate set, `None`, `false`, or a zero-valued phase readout when the
artifact is absent; the loaded-memory calls and ranking code are unchanged.
The warmup return now separates completion from material availability and the
candidate-ready atomic. The pre-publication startup receipt and trace retain
`l2_complete=true` while reporting `l2_available` and
`l2_candidate_ready` independently. Status output reports `available=false`
and zero measured memory counts rather than constructing replacement memory.
Those availability fields describe only the lexical candidate memory used by
the IME; they do not attest every canonical/productive L2 package. Preserving
the loaded-package route is not a new quality result, and the empty
absent-material readout carries no quality claim.

The adjacent absence audit found that context-phase warmup installs its
existing runtime/watcher, installed-field preload already discards its
fallible result, and live candidate-gate warmup returns an empty lattice when
candidate memory is not ready. No panic catch, synthetic memory, second getter,
new authority, or retry was added. The actual absent client now delivers every
character of exact ` ljv ` through the existing `deliver_exact_literal`
oracle, rejects delete/forward/loss/duplicate/mismatched effects, and requires
startup trace facts `l2_complete=true`, `l2_available=false`, and
`l2_candidate_ready=false`. Loaded on/off profiles require both latter facts
to be true.

Not tested: Rust compilation, focused gates, and the mandatory actual absent
first-key/Space client. Only Python syntax, diff whitespace, and bounded source
inspection have run locally. No receipt exists for this snapshot and runtime
authority remains unchanged. The next action is to freeze this source for root
review before any remote build or proof.

## Immediate baseline — 2026-09-13

Frozen source `6baa65442227bb31caf32cd56930d9e51559ca71` was built once
as `lay-ibus-engine 1.0.71`. The private actual-IBus harness then ran the
unchanged `first-word`, `manual-toggle`, and `restoration` scenario sets with
`--startup-schedule immediate` under one `dedicated-20cpu` outer lease.

Measured results:

- `first_word_us` and `first_word_ru` passed all 16 GNU Readline round trips;
- `terminal_manual_toggle_word` and `terminal_manual_toggle_boundary` passed
  all 16 terminal conversions;
- `preedit_visible_after_same_context_handoff` executed and failed. The client
  reached visible ` пров`, while the preedit remained empty and invisible;
  the receipt ended with `TimeoutError: visible post-handoff suggestion:
  last=False`;
- `same_context_us_to_us_authority_restoration` executed and failed. Expected
  visible ` дом ` after the boundary, actual visible text was literal ` ljv `;
- the other four restoration cases were not started after that first assertion.

All three private daemons were reaped and no candidate process remained. Source
identity was 1408/1408 before and after, with zero mismatches. The candidate is
`cba1bba5d22f19fed283dc94029af656bec93d7b86e70ae6d69c99aaeb1a29ff`.
The fresh config is
`9f9010ce9a3281df30cc557a50e99d287c9dd6ad0a38e5a74a60554f902dc59c`;
relative to the actual-receipt-bound template it changed only
`candidate.path`, `candidate.sha256`, and `execution_lease_id`. Dependency
manifest `1092cd2f72ddd3ae52da6436ceba47f4e26814f137eded46515d3667d3b38314`
validated all nine files.

The immutable lane name `manual-toggle-immediate-cold-preedit` identifies its
startup schedule. Its failed third scenario runs after the two eight-round
terminal cases, so this result is not a fresh-process first-prediction cold
proof.

Receipts:

- local result:
  `/home/ubu/.cache/lay/development/td121-baseline-20260913-2NIV8vuY/RESULT.json`,
  SHA-256 `257e9c50f5d1c7750f2106d5e6dca79b1ea44a898a267f5273377ab610cd3a8c`;
- local case packet with exact trace paths:
  `/home/ubu/.cache/lay/development/td121-baseline-20260913-2NIV8vuY/case-packet.json`,
  SHA-256 `a0e4f89eb6b14d0b74579b455d7a04d255f7359fd19d3d9ad16b70d98bbc317f`;
- remote root:
  `/home/e/projects/lay-development-runner/td121-baseline-20260913-2NIV8vuY`.

The initial extraction-mode preflight and the aggregate `FileExistsError`
after all declared commands are preserved as separate administrative failures.
They did not change or relabel the product results. Runtime and installed
authority were unchanged.

## Fixed schedule comparison — executed 2026-09-13

The bounded experiment reused the exact candidate, source snapshot,
actual-bound config template, deployed private IBus bundle, harness bytes, and
nine dependency files. It ran only `manual-toggle` and `restoration` with the
existing `post-exact-ready` schedule, under one fresh outer lease and without a
build. Request:

`/home/e/projects/lay-development-runner/td121-schedule-comparison-20260913-5emnCC/request.json`

Request SHA-256:
`effca546111d3be15fb69a1eeeb001b49139d112d112a7bf54b0fb47fac03b31`.

Both actual commands passed:

- `manual-post-exact-ready`: 3/3 cases. The third case kept client-visible
  ` пров` and published one visible nonempty preedit, `ерить`;
- `restoration-post-exact-ready`: 5/5 cases. The positive US-to-US case deleted
  exactly `ljv` and produced visible ` дом `; mixed retention produced ` lом`
  and ` дjv`; both negative cases preserved literal `ljv `;
- both private daemons were reaped, no candidate process remained, and source
  identity was 1408/1408 with zero mismatches before and after.

Result:

`/home/ubu/.cache/lay/development/td121-schedule-comparison-20260913-5emnCC/RESULT.json`

Result SHA-256:
`e0d3d06d412cd8d3aa6edb6ca8159e117cf72600c06d83820691a428c23c50c4`.
The manual and restoration receipt SHA-256 values are respectively
`c44c8ee0290973b3204d871dad7ba7b7fc0718b37ec8f2581e761b16ae014cc7`
and `7f4c8f08326c10deb920d1de3d55b86e3f741b431255caddbeef9ba238c7aeb2`.
The comparison config SHA-256 is
`d66848bb14d832b6739a8ab1e7ef731ba8108f5cbc7668f9ba1d176659a905a0`;
relative to the immediate baseline config, only `execution_lease_id` differs.

The request inherited the historical `lane.name` strings
`manual-toggle-immediate-cold-preedit` and `restoration-immediate`. Those labels
are administrative carry-over only. Command argv, output names, run metadata,
receipts, and each result row's `startup_schedule` all identify the measured
schedule as `post-exact-ready`; this run is reported by output name so the two
denominators are not mixed.

The comparison answered whether those two observed immediate failures persist after
the existing exact-authority completion observation. That observation is not
identified with `TypingCpu::ime_candidate_memory_is_warm`, and a changed result
does not by itself prove candidate-memory readiness or a causal source defect.
The result did not test a fresh-process first prediction, physical input,
GTK, desktop IBus, installation, package quality, or general correction
quality. The immediate failures remain retained and are not relabelled by the
post-exact-ready passes. Runtime and installed authority did not change.

## Causal boundary before a source change

Measured immediate evidence separates two first effects:

- restoration reached Space before exact authority was available. The trace
  recorded three `authority_snapshot_present=false` preparations,
  `prefetch_not_ready`, a 3575 us lookup, literal ` ljv `, and only afterward
  exact warmup completion at 112725 us;
- the manual third scenario ran after exact warmup had completed at 113568 us,
  yet its display workers returned in 0–1 us with zero candidates and
  `field_cache_disposition=not_requested`. The static first exit at
  `preedit_readout.rs:103` returns exactly that empty/default result while
  `TypingCpu::ime_candidate_memory_is_warm()` is false. This is consistent with
  the candidate-memory-not-warm branch, but the current trace does not record
  that boolean, its warmup start, or its publication time; the branch remains
  an inference rather than measured runtime fact.

The post-ready control combines more than the 115 ms exact warmup. The client
observed the trace file after 994208 us and 991608 us, including flush delay.
At that later point restoration had an exact snapshot/certificate/decision and
applied its prepared lease after a 5 us lookup. The manual preedit produced
11–12 candidates and visible `ерить`. Manual also retained an unrelated
`prefetch_not_ready` for its ` a ` boundary while the declared manual contract
passed. Therefore the comparison proves schedule dependence; it does not prove
that every correction request is ready or that 115 ms is a sufficient wait.

The startup order defines a narrower hypothesis, with two material limits.
`server.rs` publishes the factory and bridge, starts exact-authority warmup on
one thread, then starts the IME runtime lifecycle. With autocorrection enabled,
that lifecycle normally calls `warm_up_l2_for_ime()` after the L1.1 service
ensure result. A cold call does not have to wait for that sequence:
`preedit_readout.rs:103–105` and `candidate_gate.rs:176` themselves request the
same idempotent L2 warmup before returning empty. The current trace does not
show which caller won the atomic start claim or when it ran.

The candidate-memory ready flag also does not mean the complete
`warm_up_l2_for_ime()` procedure has finished. It is published in
`hot_memory.rs:46–52` after the RU/EN prefix caches warm, then
`candidate_gate::warm_up_live_candidate_readout()` still runs. Context-phase
and installed-field warmup precede the flag. Exact readiness and these two L2
milestones have different owners and no common published barrier. These are
source facts, not proof that L1.1 ordering or either L2 milestone caused the
observed failures.

| Consequence | Measured first break | Proven intact | Missing proof |
|---|---|---|---|
| Immediate restoration | Exact snapshot absent; Space lease `not_ready` after 3575 us; literal ` ljv ` | Full token retained through same-context handoff; cleanup | Candidate-memory timeline; whether earlier exact completion alone is sufficient on immediate input |
| Immediate manual preedit | Zero candidates, 0–1 us materialization, field not requested; static cold return is consistent | Handoff/client transfer reaches visible ` пров`; exact warmup had completed | Actual candidate-ready boolean, winning warmup caller, flag-publication time, frame state at the cold return |
| Post-ready restoration | Exact certificate/decision present; prepared lease applied after 5 us; visible ` дом ` | Positive and negative authority cases 5/5 | Which readiness milestone within the approximately 1 s observation window changed the outcome |
| Post-ready manual | 3/3 and visible `ерить`; separate ` a ` boundary retained `prefetch_not_ready` | Preedit frame applied within 150 ms; cleanup | Whether candidate-ready or later live-readout completion is the necessary milestone; general correction readiness remains unproved |

### Minimum discriminator

Before changing startup behavior, add trace-only observations to the existing
owners:

1. record the winning L2 warmup request, complete-procedure start/end, and
   elapsed time without treating the request as readiness;
2. record separately when `hot_memory.rs` publishes candidate-memory ready and
   when `candidate_gate::warm_up_live_candidate_readout()` finishes;
3. record the candidate-memory-ready boolean before materialization and the
   resulting `applied`/`superseded`/`late` frame disposition in the existing
   precognition-worker event;
4. record exact-snapshot presence and candidate-memory readiness separately in
   the existing Space prefetch event.

The observation must contain generations, epochs, timings, and booleans only;
no typed text or package payload. It must not add a wait, retry, polling loop,
new authority owner, cache, readiness alias, or work resubmission. One fresh
immediate run on exact bytes can then distinguish candidate-memory cold exit
from frame supersession/expiry and from missing exact authority. Existing 4 ms
Space and 150 ms preedit deadlines remain unchanged.

When tracing is disabled, the added route must perform no new IO and no event
formatting. Atomic readiness sampling may only feed already-existing trace
events. With the existing bounded opt-in debug log enabled, extra fields and
warmup milestone events can perturb cold scheduling; their timings are
diagnostic observations, not performance acceptance. Candidate generation and
ranking, SafetyGate/verifier authority, cache and frame generations, package
reload, learning, admission, and rollback semantics remain byte-for-byte
unchanged. The implementation may add no lock, RPC, controller, observer
subsystem, or failure authority.

That no-new-IO claim is scoped to the production IME startup sequence, which
loads and publishes `LayConfig` before lifecycle warmup. In an arbitrary
library caller with uninitialized runtime flags,
`runtime_debug_action_log()` may perform its existing lazy config load. The
diagnostic adds no config mechanism. Both the bin trace and library warmup
events use one `debug_log::append_private_line` sink and one centralized
`debug_log::ibus_runtime_trace_path()` preserving the existing
`LAY_IBUS_TRACE_PATH`, `HOME`, and `/tmp` fallback semantics.

## Metadata-only discriminator — executed 2026-09-13

The frozen diagnostic source kept the 4 ms Space budget, 150 ms preedit
budget, candidate generation, ranking, reducer, verifier, and all runtime
authority unchanged. It added only opt-in metadata events to the existing
private trace sink. No typed text, package payload, retry, wait, replay, or
second work submission was added.

The successful immutable run used source HEAD
`6baa65442227bb31caf32cd56930d9e51559ca71`. Source identity was 1407/1407
before and after with zero mismatches. The archive SHA-256 was
`e566f339a94d06cff3cda891665917509eec63f407dad38cf7ae24f58a2add81`;
request SHA-256 was
`663a1f5d94e24650c063bbfe960134d2c9d83e3857c48e207ee89f9d3a34611e`;
runner SHA-256 was
`43511c548b632e88aefa9ec24dae1052ea7a9afffe134a76f4f5dd4ad837a7ab`.
Focused `bin:lay-ibus-engine` correctness was 490/490 PASS, package 0,
with three performance tests excluded. The one release build passed in
178.716 s and produced candidate
`68d80bb15cb36c7bf7c84605a51e95d93dc9d00d55d59bc4bf79c0f96a749b37`.
Both unchanged immediate lanes retained the product failures and cleanly
reaped all private daemons. Runtime and installed authority stayed unchanged.

The metadata resolves the earlier ambiguity:

- the first L2 request was already won by `zbus::Connection executor`; its
  unnamed procedure started before exact-authority warmup. A later direct call
  from `lay-ime-l11-lifecycle` entered the same procedure, but the existing
  one-shot cells shared the underlying work. Therefore merely starting L2
  independently of L1.1 cannot repair this run;
- in the manual lane, exact authority became available at 107396 us. The final
  preedit epochs 52–55, generations 134–141, all sampled candidate memory as
  false before and after materialization, returned zero candidates in 0–1 us,
  and were applied at ages 47–57 us. The frame was neither superseded nor late;
- candidate memory was published only after those frames. Prefix warming took
  5102 us inside the winning one-shot section, while the two procedure callers
  observed publication at 1142862 us and 1080557 us from their own starts.
  Full live-readout warmup completed at 1144746 us and 1082321 us;
- in restoration, epochs 3–5 sampled both exact snapshot and candidate memory
  as unavailable. Space returned `prefetch_not_ready` after a 3575 us lookup;
  exact authority became available later at 112876 us. Cleanup ended the
  process before an L2 completion event, so this lane proves no L2 failure.

The first authority loss is now measured: the engine accepts and applies valid
first input frames before the one-shot exact and candidate operations have
published completion. This is not a handoff, reducer, frame-age, or L1.1-start
defect. The existing request-time 4 ms and 150 ms budgets cannot cover measured
cold work above 100 ms and 1 s respectively. A repair needs one startup
admission boundary before the first engine can accept input; extending the
per-key budgets or replaying an already accepted key would move or duplicate
authority.

Receipts:

- result:
  `/home/ubu/.cache/lay/development/td121-diagnostic-compilefix-20260913-m03hzil4/RESULT.json`,
  SHA-256 `776610ccfc16ea9d3f6c58d07ea50c37c14c6f1af8beece18cde169dfaafa0f7`;
- manual receipt SHA-256
  `8763967a565f7ab3b85a99d0a40a7fe7f052de2506f926d3958bea9f1c27fe5f`;
- restoration receipt SHA-256
  `87877b79afa157caa23fac7674519cd9de1941024acbbdf107a0dd0f24158d28`;
- remote root:
  `/home/e/projects/lay-development-runner/td121-diagnostic-compilefix-20260913-m03hzil4`.

Three non-product execution incidents remain separate. The first remote root
had one pre-launch SSH path typo and then a focused-run refusal because the
disposable source lacked `.git`; Cargo, client, and product counts were zero.
The next fresh root found three trace-only borrow compile errors before any
test or client run; only those three borrows were corrected. One accidentally
started local guarded compile was interrupted immediately with status 130 and
zero tests; it is not evidence. The final run above is the only diagnostic
result used for the causal verdict.

The diagnostic did not test physical input, GTK, desktop IBus, installation,
package quality, general correction quality, or a latency distribution.
Quality remains `UNKNOWN`; routing and timely application of an empty result
do not prove candidate quality.

## Two viable startup-admission designs

Both designs wait for completion of the existing cached one-shot operations:
`WARM_EXACT_AUTHORITY: OnceLock<Option<_>>` and the L2 installed-field/readout
cells. Completion is not capability success. Exact `None` and an installed L2
field `Err` must still let the keyboard register; the affected routes retain
their current literal/abstaining behavior. Spawn or ordinary channel/deadline
failure must return a startup error without publishing a falsely ready engine.
With release `panic = "abort"`, a worker panic terminates the process before
publication rather than guaranteeing a structured zbus error. Exact and L2 warmup
run for both autocorrection settings: the existing disabled lifecycle also
requests L2 so preedit candidates remain available independently of automatic
Space correction.

**A — keep early registration and gate each `CreateEngine`.** Start both
one-shot operations before publishing the factory, but keep the current early
factory/name availability. `CreateEngine` waits on a shared completion future
and returns an engine path only after both required operations have completed.
The installed IBus daemon is 1.5.34-rc2, runs without `--timeout`, and therefore
uses its verified 15000 ms default factory/CreateEngine timeout. The private
harness has a separate 2500 ms RPC bound. An implementation must choose a
shorter internal deadline and preserve both clients.

This keeps the existing login-availability intent and adds the delay only to a
cold engine selection. It also crosses the context-admission boundary:
`CreateEngine` currently observes ingress and creates its factory ticket before
binding the target, while admission has 1 ms rendezvous and 5 ms acquisition
budgets. Waiting before observation ages the intercepted callback; waiting
after observation holds a ticket without a target. Making either ordering
sound requires a new shared readiness coordinator, explicit ticket cancellation
on timeout, concurrent-create tests, and a proof that no hot handoff was
reordered. Source/package identities stay unchanged, but maintenance and
failure-path cost are high.

**B — gate factory/name publication once.** Load startup config, eagerly start
the existing IME lifecycle, run exact and L2 warmup concurrently, wait for the
exact and L2 operations to complete, and only then publish the factory and own
the engine name. The lifecycle keeps its existing single start owner; its later
L2 call shares the measured one-shot cells, and L1.1 service completion is not
folded into this readiness gate. The bridge and `CreateEngine` admission path
remain unchanged. Cold startup latency becomes
approximately the slower one-shot operation: 1.145 s in this single traced
sample, rather than a per-key wait. Warm calls remain cheap through the existing
cells. A bounded implementation uses an internal startup deadline below the
installed IBus 15000 ms factory wait; timeout or worker failure exits before
name ownership so IBus can report the engine unavailable and retry a fresh
process on the next selection.

This changes the current server promise that GNOME can see the factory while
lexical memory warms. No current receipt measures a login regression, and one
trace is not a production latency bound. A bounded distribution of fresh
process-to-ready times on the deployment route, including autocorrection
enabled and disabled, therefore remains a promotion gate. Its ownership cost is lower:
one startup join/deadline in `server.rs`, no callback-ticket changes, no new
runtime authority, and no duplicated package/readiness state.

**Preferred design: B, subject to that latency gate.** It places the boundary
before any factory callback exists and preserves the existing admission
ordering. Design A is viable only with a larger admission refactor whose risk
is unrelated to the measured cold-memory defect. Neither design changes the
4 ms/150 ms input budgets, candidate ranking, SafetyGate/verifier authority,
source IDs, learning, cache generations, or package bytes. The fixed proof must
keep immediate and post-ready denominators separate and reject any aggregate
improvement that loses a case or admits false authority.

## Final implementation preflight

The selected first implementation is design B with one **5000 ms absolute
warmup-join budget** shared by both workers, not 5000 ms per sequential wait.
Config load, IBus connection/admission bootstrap, and factory/name/bridge
publication remain distinct process-to-ready overhead outside this timer. The
local running daemon is `/usr/bin/ibus-daemon` 1.5.34-rc2,
SHA-256 `24338c0e7cfb749d2ac9b9254d7d54029e71b5ac9342aa1b8d78b1d55f5ac1b6`;
it has no `--timeout` override and uses the matching installed source default
of 15000 ms. Both private receipts bind the same copied-daemon SHA, and their
immutable driver starts it without a timeout override. The 5000 ms budget is
4.36 times the single traced 1.145 s completion and leaves 10000 ms inside the
actual IBus factory wait for process launch and D-Bus publication. This is an
engineering bound for the candidate, not a measured production percentile.

Implementation order:

1. load `LayConfig` first so its existing runtime publication completes before
   warmup starts;
2. initialize the existing Space prefetch owner and start exact and L2 warmup
   workers in parallel for both autocorrection settings. The bounded L2 worker
   first calls the existing
   `ensure_ime_runtime_warmup_started(config.nanda_autocorrect)`, then directly
   completes L2. This keeps the lifecycle's synchronous thread-spawn fallback
   off the async server executor and inside the same 5000 ms join. Enabled mode
   may prepare L1.1 concurrently; disabled mode retains preedit L2 warmup
   without starting the L1.1 service;
3. collect exact and L2 worker completions against one monotonic deadline.
   Do not wait for or claim L1.1 service readiness. Preserve
   exact `None` and installed-field failure as completed/unavailable capability,
   with current literal/`ABSTAIN` behavior. Treat spawn failure, channel loss,
   or deadline expiry as startup failure. A release-profile worker panic aborts
   the process before publication; the disconnect unit represents unwind or
   channel absence and does not promise panic-to-zbus conversion. The bounded receiver blocks the
   pre-publication server task; zbus's enabled internal executor runs on its
   separate thread, so the admission observer remains live. The L2 worker owns
   the lifecycle spawn-failure fallback so it cannot block deadline polling;
4. only after exact/L2 completion publish `IBUS_FACTORY_PATH`, request
   `IBUS_ENGINE_NAME`, and publish the session bridge. `CreateEngine`, its
   admission observation/ticket binding, and engine-path publication remain
   byte-for-byte on their current order.

The bounded source change should be owned by `server.rs` plus focused tests for
enabled/disabled lifecycle settings, one shared deadline, completion-without-
capability, timeout/failure before publication, and publication only after the
gate. It must not add a second package cache or readiness alias. Initial
acceptance is the current focused IME bin gate and both fixed immediate lanes on
the exact remote candidate. Broader release, install, physical input, login
latency, and production promotion remain separate gates. If the 5000 ms bound
fails on the fixed remote route, reject the candidate and measure a startup
distribution before changing the number.

The startup work keeps the same package bytes, source IDs, one-shot cache
owners, reload paths, learning state, and eventual L1.1 service owner. Its CPU,
IO, and RSS cost moves before public factory availability; exact, L2, and in
enabled mode L1.1 may still overlap, so peak contention is not claimed lower.
Disabled autocorrection still warms L2 because preedit prediction is a separate
feature. Missing/corrupt package completion keeps the current unavailable or
abstaining capability and does not become a keyboard-start failure. The gate
does not promise that L1.1 work is available on first input; eager lifecycle
start merely preserves or improves its current opportunity to become ready.

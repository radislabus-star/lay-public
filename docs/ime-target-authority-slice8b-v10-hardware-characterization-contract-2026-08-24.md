# Slice 8B V10 Hardware Characterization Evidence Contract

Date: 2026-08-24

Status: `PAPER ROUTE STRUCTURE REVIEWED / B0 SEQUENCING CORRECTION V2`; no
measurement or implementation admission.

Owning architecture:
`docs/ime-target-authority-slice8-lexical-readout-2026-08-23.md`

## Decision

Gate B is split into two evidence routes that must never be merged:

```text
historical exact ELF
  -> one whole-process hardware envelope
  -> proves only the combined A/B/C executable workload

source-preserving diagnostic proxy
  -> phase-selected executor-core windows
  -> proves costs only for the explicitly named proxy and schedule
```

The historical ELF cannot provide exact executor phase attribution. It is
stripped, has no symbol table or phase selector, and its only V10 entrypoint
loads the package, compiles and writes the sidecar, validates the complete
language, builds proof material, starts PSS child processes, runs generated
Gate B, runs every sequential Gate C query through both banded V10 and the
full-row oracle, and finally runs the 20-client pass.

Consequently:

```text
historical whole-process envelope                  ADMISSIBLE AFTER PREFLIGHT
historical per-search/per-edge hardware cost       NOT PROVABLE
historical 1-client executor hardware cost         NOT PROVABLE
historical 20-client executor hardware cost        NOT PROVABLE
diagnostic executor proxy                          PAPER-SPECIFIED ONLY
pristine latency C                                 SEPARATE DENOMINATOR
V12                                                NOT ADMITTED
```

No `perf`, V10 execution, Cargo build/check/test, diagnostic executable, or
runtime mutation is authorized by this document. A separate implementation
preflight must be `READY_TO_IMPLEMENT` before any of those actions.

### B design-window audit note

At `2026-08-24T17:33:52Z`, a double-quoted `rg` pattern containing shell
backticks invoked the bare `perf` executable through command substitution. It
ran without a subcommand and only emitted usage text, which then made the `rg`
pattern invalid.

At `2026-08-24T19:29:33.706Z`, after implementation preflight V2 had passed,
a read-only remote wrapper inspection also invoked `perf --version`. It
completed at `2026-08-24T19:29:34.617Z` and printed only
`perf version 6.8.12`. No controller code or B0 action existed at that point.
This violated the V2 sequencing discipline even though it did not open a PMU
event. V2 is preserved but superseded for execution; a corrected V3 preflight
is required before controller implementation.

The corrected exact audit boundary is:

```text
pre-B2 perf executable invocations                       2
bare usage-only invocation                               1
version-only invocation                                  1
perf stat or perf record                                NO
PMU event opened or measured                            NO
V10 subject executed                                    NO
proxy or benign B2 workload executed                    NO
```

Therefore later evidence may say that no perf/PMU measurement occurred before
B2. It must not say that the `perf` executable was never invoked or was invoked
only once.

## Frozen identities

The B preflight must bind these exact inputs before a capability probe or
subject execution:

```text
P0 original archive
/home/ubu/.local/share/lay/provenance/slice8b-v10-f6178f

P0 PROVENANCE.json
af26f90714460529c5778fb415a3c0aa7eb83ef1f51fdc97230417ac8f5c9faf

P0 correction-v1
/home/ubu/.local/share/lay/provenance/slice8b-v10-f6178f-correction-v1

CORRECTION.json
74f66275a6e1f00b4dedea16ea2b62ab1ad2f4fffa6fada012bca19cc7a1ac90

historical V10 source
f6178f3882c1a216a9c2aaf7ff59dd6bdd78798708516b753ea7208177d9718c

historical release test ELF
1e83ef76df68cd2f0238d1334eb4049f9063608292e72e5454a09f21a4afacc1
Build ID 32e47da137adff6d49f9209ccd2804b6daa728ae

historical V13 DAFSA sidecar
a1aa95be8c43fae8fedf02d3261dd1265dcbfbc8d618330fcd6449ae829df8cd

fixed V7 denominator
33fded73e13f565bdc08a83bd440be67c80f6d85a196453050b09bc9f7ef28e4

historical V10 result
08cccb3c9c63a24e9dc691958fde319de2a94b762fcb9f703ac342fcb72174e1

archived Cargo.toml
90a3ae6b16677ff49b70aab3457b794cd1e114a547f9e113a8b49595f976207b

archived Cargo.lock
e6399be48c393e1557b451fcb04d886f9fbdaa9812109410ac3d9ecdc98b93f1

diagnostic build toolchain
cargo 1.97.1 c980f4866
rustc 1.97.1 8bab26f4
LLVM 22.1.6
release profile, empty RUSTFLAGS, default features
x86_64 baseline features fxsr/sse/sse2

V13 package
140,556,462 bytes
cce259fe0ce5dce67702383363b66f0fe9b9ff5a87d8f01c4fcf342d91218d7b
```

The required V13 package is not contained in P0. Before B execution it must be
copied from the remote historical path
`/home/e/.local/share/lay/nanda_wave/l2/LAY-L2-RU-FULL-v13.bin` into a
B-specific staging tree, checked for exact size and SHA-256, made read-only,
and published by a same-filesystem final rename with zero post-rename
mutations. A mutable installed package path is never a measurement input.

The historical sidecar is an identity reference. The exact ELF always compiles
and writes a sidecar, so B3 must direct `LAY_V10_SIDECAR` to a unique run-local
output and require the resulting SHA-256 to equal the historical sidecar.
`LAY_V10_RECEIPT` must also be run-local. Installed or repository receipts,
sidecars and packages must not be overwritten.

## B0a: immutable provenance and input closure

B0a runs only after a separate implementation preflight admits its scripts. It
is read-only with respect to the sealed inputs and must produce
`lay.v10.hardware-input-closure.v2` with:

- all frozen identities above;
- source and destination path, byte count, mode and SHA-256 for every input;
- original and correction archive manifest verification results;
- ELF class, machine, stripped state and Build ID;
- exact test name and direct-ELF argument vector;
- the two permitted Rust test exit states: success when all current conjuncts
  pass, or the normal Rust assertion failure when the receipt shows that only
  latency failed; integrity or semantic failure is never permitted;
- isolated output root and a complete planned-write inventory;
- remote host identity and measurement lock identity;
- `diagnostic_build_status=NOT_BUILT`;
- `query_schedule_status=NOT_CREATED`;
- `installed_lay_changed=false` and `runtime_authority_changed=false`.

B0a fails closed on any byte, mode, Build ID, package, output-path or host
mismatch. It does not repair, rebuild or fetch a missing input.
Its owner may inspect and seal input provenance only; it cannot build or run the
diagnostic executable and cannot claim that a query schedule exists.

## B0 diagnostic build and schedule-freezer

The query schedule cannot be produced by the sealed historical ELF or by B0a
file inspection. `parse_v7_cases()` is test-only source, and the historical ELF
has no schedule-freezer entrypoint. The implementation preflight must therefore
admit one explicit new route between B0a and B0b:

```text
B0a immutable input closure
  -> build one V10-derived diagnostic executable
  -> run its unmeasured schedule-freezer entrypoint
  -> B0b immutable schedule closure
```

### B0 diagnostic-build owner

The single diagnostic executable must contain all later test-only entrypoints:
schedule freezer, semantic parity, one-client phase controller and 20-client
phase controller. Its production lines `1..1148` remain exactly `39,047` bytes
with SHA-256
`ce9ea2d290602774e2f45444e10e6fdabce6af4a61ae2bdca54c6d2e31a53f26`;
additions are restricted to the test module.
The preflight pins the archived Cargo inputs and toolchain already listed in
this contract, permits exactly one guarded remote build, and records source,
patch, executable and Build ID identities. A freezer defect returns to paper
analysis and a new preflight; it does not permit an unregistered second build
or variant.

The build owner consumes only the accepted B0a closure and emits one executable
identity plus its provenance receipt. It cannot run the schedule freezer,
publish a schedule, open a PMU event or decide B0b completeness.

### B0 schedule-freezer owner

The schedule-freezer is an unmeasured diagnostic run. It must:

- use the exact recovered `parse_v7_cases()` over the sealed V7 denominator;
- preserve all 382 source ordinals and composite attempt identities;
- call the exact `Phase7dCertificateOracle::new()` and `retrieval_lanes()` for
  every damaged surface;
- write composite identity, source ordinal, damaged-surface digest,
  retrieval-lane digest, B5 ordinal and B6 worker/chunk assignment;
- omit `target_surface` and every target-derived value from the measured
  schedule payload;
- write only to a unique run-local staging tree;
- run with no `perf`, PMU event, historical V10 subject, search enumeration,
  latency observation or runtime authority change.

The freezer owner consumes the already sealed diagnostic executable and B0a
inputs. It cannot invoke Cargo, alter source, replace the executable or publish
the final immutable schedule. Its only authority is to emit a run-local staged
schedule and freezer receipt for B0b validation.

## B0b: immutable schedule closure

B0b validates and publishes `lay.v10.hardware-query-schedule.v1`. It binds the
schedule to the B0a input closure, exact V7 SHA, production-prefix SHA and the
one diagnostic executable SHA/Build ID. It requires exactly 382 ordered
entries, 382 unique composite identities, valid retrieval-lane digests, exact
B5 order, and the fixed B6 `20 x ceil-div` partition. It independently rejects
any target field or target-derived value.

The B0b owner cannot build or execute code. It validates the freezer output,
publishes the immutable schedule and emits the closure receipt; any mismatch
returns to analysis without invoking build or freezer again under the same
preflight.

The populated schedule tree is hashed, made read-only and published by one
same-filesystem rename as its final mutation. B5 and B6 may consume only this
B0b SHA-identified schedule. B1 cannot start until B0a, the one build, the
unmeasured freezer and B0b all pass.

## B1: hardware and environment snapshot

The expected machine class is the P0 host: Intel Core i9-13900HK with 20
logical CPUs and hybrid P/E topology. CPU numbering and topology must be
observed, never inferred from the model name.

Before the first run and before and after every accepted run, capture:

```text
identity
  hostname, machine-id digest, boot ID
  kernel release and command line
  microcode version
  DMI product, BIOS version and BIOS date
  vulnerability mitigation state

topology
  online/present CPU masks
  CPU -> package/core/thread/core-type mapping
  SMT sibling masks
  NUMA nodes and CPU/node mapping
  cache level/type/size/shared-CPU mapping

frequency policy
  scaling driver and governor per policy
  energy performance preference
  min/max/current policy frequencies
  intel_pstate status, min_perf_pct and max_perf_pct
  turbo/no_turbo state
  cpufreq boost state when exposed

thermal and power
  package/core temperatures
  thermal throttle counters
  powercap/RAPL availability and limits when readable

host pressure
  uptime and load averages
  /proc/stat procs_running and procs_blocked
  CPU, memory and IO PSI
  MemAvailable, page cache, swap state
  top CPU/RSS processes
  active cargo, rustc, perf and V10 processes
  irqbalance state and interrupt counters by CPU
```

The first accepted snapshot freezes the governor, EPP, turbo, P-state, online
CPU set, SMT state, NUMA topology, kernel, microcode and boot ID for the whole
matrix. The measurement scripts may observe these values but may not modify
them. If the pre-existing profile is unsuitable, B is `BLOCKED_ENVIRONMENT` and
a separate mutation preflight is required.

Run admission requires:

- the exact same topology and policy snapshot throughout the matrix;
- no thermal throttle counter increase;
- package/core temperature below 90 C before and after the measured window;
- CPU PSI `some avg10 <= 2.00` before launch;
- memory and IO PSI `full avg10 <= 0.10` before launch;
- no competing Cargo, rustc, perf or V10 process;
- no swap-in, OOM or major page fault in the measured subject;
- for B5/B6, zero subject CPU migrations; for B3, no CPU outside the frozen
  process cpuset while scheduler migrations inside that cpuset are reported;
- in three quiet samples 250 ms apart before launch, `procs_running <= 2` and
  no non-kernel process other than the admitted controller consumes more than
  1% of one CPU during the sampled interval;
- during the measured window, no unrelated user process accumulates CPU time
  above 1% of subject task-clock according to pre/post process accounting;
- no counter, affinity, topology or policy drift.

An admitted script may wait at most 120 seconds for the frozen temperature and
pressure thresholds. Failure is `BLOCKED_ENVIRONMENT`, not permission to tune
the host or loop indefinitely.

## B2: PMU and perf capability contract

B2 is a capability gate on a benign fixed workload, not a V10 measurement. It
must record:

```text
perf version and executable SHA-256
perf_event_paranoid
kptr_restrict
NMI watchdog state
available PMUs and event aliases
hybrid cpu_core/cpu_atom event resolution
resolved event selector/config bytes and privilege mode
command-mode thread inheritance
control-fd/fifo enable-disable support
per-thread output support
time_enabled and time_running for every event
```

The capability probe must run once on the selected P-core CPU and once across
the full 20-CPU mapping. It must not load V13 or execute V10. Every required
event must be supported on every core type used by its schedule and produce a
machine-readable value. `<not supported>`, `<not counted>`, missing hybrid-PMU
coverage, scaled values, or `time_running != time_enabled` is a hard B2 failure.
No event may be silently replaced with a generic alias or another cache level.

All B counters are command/task scoped, never system-wide. They include subject
user and kernel execution, exclude hypervisor execution, and inherit into the
subject threads; B3 additionally inherits into its child processes. If host
permissions allow only a different privilege scope, B2 is blocked. The receipt
records the resolved selector/config for each P-core and E-core PMU, not only
the human-readable alias.

Preregistered non-multiplexed groups are:

```text
G0 core/time
  task-clock, cycles, ref-cycles, instructions,
  context-switches, cpu-migrations, page-faults

G1 branch/generic-cache
  branches, branch-misses, cache-references, cache-misses

G2 data-cache
  L1-dcache-loads, L1-dcache-load-misses,
  LLC-loads, LLC-load-misses

G3 translation
  dTLB-loads, dTLB-load-misses
```

Software events do not excuse multiplexing of hardware events. If a group is
too large for either hybrid PMU, the contract must be amended and structurally
reviewed before any V10/proxy run; an implementation script may not split or
substitute it adaptively.

## B3: exact historical ELF envelope

B3 consists of exactly one accepted direct execution of the sealed historical
ELF under G0. It uses the exact ignored test
`nanda_wave::l2_field::v13_typed_peak::tests::v10_full_v13_abc_proof`, the
sealed B0 package/V7 inputs, and unique run-local receipt and sidecar outputs.
No Cargo command is involved.

`perf` command mode must include all threads and PSS child processes. The
process affinity mask is the frozen 20-CPU set. Scheduler placement inside that
set remains part of the combined envelope and is not reinterpreted as fixed
worker placement.

The historical run returned nonzero because Gate C's latency conjunct failed.
A B3 rerun may return success if the current latency conjunct passes, or the
normal Rust assertion failure if its receipt shows that only latency failed.
The receipt, not the process exit alone, must prove:

- exact source identities and sidecar SHA;
- Gate A and Gate B semantic/integrity checks pass;
- 382/382 target form and lemma retention;
- zero unresolved and false certificates;
- zero full-row/banded mismatches;
- maximum product states 35,590 and scratch 6,656 bytes;
- every failed promotion conjunct, if any, is limited to latency.

Any other test failure invalidates B3. B3 reports aggregate task time, cycles,
reference cycles, instructions, context switches, migrations, page faults and
whole wall time for the entire mixed executable route. It contributes no
per-request, per-search, per-edge, phase, single-client or 20-client executor
metric and is excluded from every predictive-model fit.

B3 permits at most two total launch attempts: the original and one replacement
for an independently recorded environment invalidation. A second invalid or
subject failure returns `BLOCKED_ENVIRONMENT` or `BLOCKED_PROVENANCE`; it does
not authorize another exact-ELF execution.

## B4: phase-attribution feasibility

B4 is already resolved from sealed source and ELF inspection:

```text
ELF stripped                                            true
.symtab present                                         false
independent historical phase entrypoints                false
sequential Gate C mixes banded and full-row work        true
20-client phase shares one combined entrypoint          true
PSS child work inherited by command-mode perf           true

historical executor phase attribution             IMPOSSIBLE
```

Symbol uprobes, whole-process subtraction, historical internal timers, thread
name inference, time slicing and source-line attribution are forbidden as
phase proofs. They cannot recover an exact denominator from this binary.

The only permitted executor route is a new diagnostic proxy. It must be called
`V10-derived diagnostic executor proxy`, never `historical V10`. Its build and
execution require a separate implementation preflight and these identities:

```text
entire recovered V10 source
f6178f3882c1a216a9c2aaf7ff59dd6bdd78798708516b753ea7208177d9718c

production region, lines 1..1148, 39,047 bytes
ce9ea2d290602774e2f45444e10e6fdabce6af4a61ae2bdca54c6d2e31a53f26
```

The production region must remain byte-identical. Diagnostic code may be added
only inside the test module. It may call the exact `enumerate_lane` and merge
logic through a test-only phase controller. Any duplicated test-only merge
step must be covered by exact output parity. The proxy may not add counters,
branches, hooks or feature flags to the production search or hot loop.

The proxy build additionally pins the archived Cargo.toml/Cargo.lock,
toolchain, release profile, feature set, target features and empty RUSTFLAGS
listed above. It records the complete surviving dependency inventory, added
test-module patch SHA-256, final source SHA-256, executable SHA-256 and Build
ID. Because the full build-time source closure remains `WATCH`, these controls
identify a new build but cannot turn it into a byte-reproducible historical
executable.

This is the same executable built once between B0a and B0b for the
schedule-freezer. B4 does not authorize a later proxy rebuild. The parity and
B5/B6 entrypoints must come from the executable identity already sealed by
B0b.

Before hardware use, the proxy must run an unmeasured parity route over all 382
fixed cases. It must match the exact production function for terminal refs,
peaks/certificate keys, completeness, expanded states and scratch, and match
the independent full-row oracle. It must also reproduce the historical
aggregate invariants listed in B3. This proves scoped semantic parity, not
historical executable identity or byte-reproducible source closure.

## B5: one-client diagnostic schedule

B5 measures only the proxy executor-core window:

```text
pre-window
  load sealed package and historical sidecar read-only
  parse all 382 fixed query identities in frozen source order
  prepare exact Phase7D retrieval lanes
  run one complete unmeasured warmup schedule
  verify parity, affinity, environment and thermal state

measured window
  one process thread
  exact 382 queries once, in frozen source order
  exact V10 banded enumerate_lane calls
  terminal merge, sort and dedup

post-window
  disable counters before serialization
  verify outputs, affinity and environment
  write the run-local receipt
```

The measured thread is pinned to one P-core logical CPU selected from observed
B1 topology: the lowest numeric online P-core CPU whose full sibling mapping is
available. Its SMT sibling is excluded from the subject affinity and must
remain free of competing user work. The exact CPU ID and sibling ID are frozen
before the matrix. Migration count must be zero. Per-query stack and terminal
storage remain allocated by the exact executor inside the measured window;
warmup does not replace or pool them.

B5 excludes query normalization, Phase7D lane construction, package loading,
sidecar mapping, certificate materialization, target lookup, receipt writing
and controller polling. It therefore characterizes the named executor-core
proxy, not complete V10 search latency.

## B6: twenty-client diagnostic schedule

B6 uses the same pre/post boundaries and one measured pass over the same 382
query identities. It preserves the historical fixed partition:

```text
workers       20
chunk size    ceil(382 / 20) = 20
workers 0..18 20 contiguous queries each
worker 19      2 contiguous queries
```

B1 supplies the numerically ascending list of all 20 online logical CPUs.
Worker `i` is pinned to list element `i`; the receipt binds worker ID, CPU ID,
core ID, core type, SMT sibling set, query range and request count. Nineteen
worker threads are created before the window and the main thread is worker 0,
so exactly 20 subject threads participate. All threads wait on one barrier and
events remain disabled until all threads are ready. The measured window starts
with the single barrier release after the enable acknowledgement. Each worker
runs its assigned queries sequentially.

Every worker affinity mask must remain a singleton and aggregate migration
count must be zero. The fixed two-query final shard is reported as historical
schedule imbalance and must not be hidden by pooled metrics. B6 is a full-host
hybrid P/E workload; it is not a homogeneous-core scalability proof.

Any B5/B6 difference is the joint effect of concurrency, fixed worker
partition, SMT placement and heterogeneous P/E execution. B may not attribute
that difference to concurrency or cache pressure alone.

## Run matrix and execution control

B5 and B6 each use G0..G3. Every route/group pair requires three accepted
independent process runs. Order is fixed before execution:

```text
replica 1: B5 then B6 for G0, G1, G2, G3
replica 2: B6 then B5 for G0, G1, G2, G3
replica 3: B5 then B6 for G0, G1, G2, G3
```

No adaptive event selection, query reorder, repetition increase or build
variant is permitted. At most two invalid environmental replacements are
allowed for a route/group pair. A third invalid attempt makes that pair and B
`BLOCKED`, returning to analysis rather than extending the loop.

`perf stat` events start disabled. The proxy loads, warms and parks all subject
threads before an external controller enables one event group. After the
enable acknowledgement, the controller releases the start barrier. The proxy
signals completion and blocks; the controller disables counters before receipt
serialization and process teardown. Capability B2 must prove this control
sequence and child-thread inheritance before any proxy run.

Every raw `perf` JSON, environment snapshot, affinity map, subject receipt,
stderr/stdout digest and controller transcript is retained. Raw artifacts are
published read-only through a staging tree whose final rename is its last
mutation.

## B7: derived metrics and stability

All raw values remain separated by route, event group, replica, PMU/core type
and worker. If B2 cannot provide the required per-thread B6 values, B is
`BLOCKED_PMU`; aggregate-only output is not silently accepted. Valid same-run
ratios are:

```text
instructions/request
cycles/request
reference-cycles/request
task-clock/request
IPC = instructions/cycles
branches/request
branch-miss rate = branch-misses/branches
cache-miss rate = cache-misses/cache-references
L1D load-miss rate = L1D-load-misses/L1D-loads
LLC load-miss rate = LLC-load-misses/LLC-loads
dTLB load-miss rate = dTLB-load-misses/dTLB-loads
throughput = requests/measured-window-wall-time
```

Cross-group numerators and denominators may not be divided. Cross-group
comparison first normalizes each run by its exact request count, then reports
the median and full range of the three replicas. Hybrid totals may be formed
only by summing the same event over `cpu_core` and `cpu_atom`; both components
must remain separately visible.

Stability admission uses `(max - min) / median`:

```text
instructions/request, branches/request                  <= 1%
cycles/request, ref-cycles/request, task-clock/request  <= 5%
miss counters and miss rates                           <= 15%
```

An unstable family is `WATCH_UNSTABLE`, not evidence for or against a cause.
Zero denominators produce `UNDEFINED`, never zero percent. Diagnostic wall time
is reported for context but cannot enter pristine C p99.

Per-edge, per-state and per-transition costs are forbidden until an independent
structural-work receipt supplies matching query, route and schedule identities.
B does not create those denominators. No predictive model is fitted in B.

## Claim boundary

B may prove:

- the exact historical ELF's aggregate mixed-workload hardware envelope;
- whether required PMU events are supported without multiplexing;
- the proxy executor-core instructions, cycles, branch, cache and translation
  behavior under one fixed P-core schedule and one fixed full-host schedule;
- whether per-request physical cost and hardware pressure change between those
  two proxy schedules.

B cannot prove:

- historical V10 executor phase cost;
- complete V10 search or certification cost;
- pristine p50/p99/max latency;
- product/runtime latency or authority;
- causal ownership of a regression from correlation alone;
- per-edge cost without the separate structural denominator;
- quality, candidate retention or false-certificate rates beyond the frozen
  parity prerequisite;
- behavior for query lengths 23..96;
- that packed scalar, SWAR, unrolling or any other V12 design meets a gate.

The final B status vocabulary is:

```text
B_COMPLETE_WITH_DIAGNOSTIC_PROXY
  all B0..B7 evidence valid;
  historical aggregate and proxy claims remain separate;
  historical executor attribution remains NOT_PROVEN.

BLOCKED_PROVENANCE
BLOCKED_ENVIRONMENT
BLOCKED_PMU
BLOCKED_PROXY_PARITY
BLOCKED_PHASE_CONTROL
WATCH_UNSTABLE
```

`B_COMPLETE_WITH_DIAGNOSTIC_PROXY` does not admit V12. It permits only the next
paper task: contracting a predictive model with preregistered features,
fit/validation/held-out partitions and an error upper bound. V12 remains
`NOT_ADMITTED` until that later model contract and all preceding evidence pass.

## Side effects and forbidden actions

Future admitted B work may create only an isolated B0a input closure, one
source-preserving diagnostic build, one unmeasured schedule-freezer output, one
B0b schedule closure, capability receipts, one historical-ELF envelope, the
fixed proxy run matrix, raw perf outputs and immutable receipts on the remote
proof host. Cargo, if later admitted for the proxy, runs only through
`scripts/cargo-guard.sh` on the remote host and builds one preregistered variant.

Forbidden throughout B:

```text
installed Lay changes or version bump
daemon, IBus or extension restart
runtime authority change
V13/V90/L1.1 package rebuild
mutation of P0 or correction-v1
mutation of installed V13 inputs
local-machine heavy build or proof
second or adaptive diagnostic build variant
schedule-freezer under perf or PMU measurement
hot-loop diagnostic counters or hooks
whole-process counter subtraction as phase evidence
unsupported event substitution
counter multiplexing
adaptive repetitions or event groups
quality denominator changes
latency gate relaxation
V12 implementation, push or deployment
```

## Admission sequence

### Structural critique result

The first monolithic worksheets are retained as negative evidence. V1 and V2
mixed candidate groups with foreign source owners. V3 repaired every route
binding and reached 14/14 exact source matches with no conflicts or evidence
gaps, but its single superposed wave still returned `VETO` for weak composite
support. That VETO was not reclassified as a size-only WATCH and was not passed
through hierarchical aggregation.

The corrected verification shape matches this contract's actual ownership:

```text
global owner/claim skeleton                         PASS
local route gates                                  14/14 PASS
route coherence for every checked group                  1.0
conflicts                                                  0
evidence gaps                                              0
authority_ready                                        false
```

Receipts are under:

`docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_HARDWARE_CONTRACT_2026-08-24/`

The acceptance receipt is
`global-route-skeleton-final-receipt.json`; each local owner has one
`route-*-final-receipt.json`. This result is named
`PAPER_ROUTE_STRUCTURE_REVIEWED`, not `STRUCTURALLY_ACCEPTED_WITH_SPLIT` and
not `READY_TO_IMPLEMENT`. It establishes coherence only.

### B0 sequencing correction result

The later independent audit established that the schedule cannot exist at the
start of B0 because both `parse_v7_cases()` and retrieval-lane construction
need a new test-only entrypoint. It also established the precise pre-B2 perf
audit boundary recorded above. The first correction packet still grouped the
diagnostic-build and schedule-freezer owners and correctly returned `VETO` for
an owner conflict. That receipt is retained.

Correction V2 separates all four sequencing owners and the audit owner:

```text
global owner/sequence skeleton                         PASS
local route gates                                  19/19 PASS
B0a closure                                               PASS
diagnostic build                                          PASS
schedule freezer                                          PASS
B0b closure                                               PASS
audit boundary                                            PASS
conflicts / evidence gaps                                0 / 0
authority_ready                                          false
```

The corrected receipts are under:

`docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_HARDWARE_SEQUENCING_CORRECTION_V2_2026-08-24/`

The current global receipt is `global-route-skeleton-v3-receipt.json`. This
supersedes the sequencing shape, not the historical P0 evidence, and still
does not authorize the implementation preflight or any executable action.

```text
this paper contract
  -> NANDA structural critique
  -> corrected paper PASS
  -> separate implementation preflight
  -> B0a immutable input/provenance closure
  -> build ONE source-preserving diagnostic executable
  -> unmeasured schedule-freezer
  -> B0b immutable schedule closure
  -> B1 environment snapshot PASS
  -> B2 benign PMU capability PASS
  -> one B3 aggregate run
  -> same-executable proxy semantic parity PASS
  -> fixed B5/B6 matrix
  -> B7 receipt and claim-boundary audit
  -> predictive-model paper contract only
```

At the present stage the sequence stops after paper critique. Runtime authority
changed: `false`.

### Pre-B2 perf audit correction V3

The version-only invocation changed one frozen provenance fact after
implementation preflight V2. It did not change the owner topology, measurement
denominators, one-build rule or STOP boundary. The fail-closed consequence is:

```text
implementation preflight V1      BLOCKED_BEFORE_CODE, retained
implementation preflight V2      READY_TO_IMPLEMENT, retained evidence
V2 execution authority           SUPERSEDED BY AUDIT DRIFT
controller implementation        NOT STARTED
B0a through B2                    NOT STARTED
required next artifact            implementation preflight V3
```

The exact audit evidence is:

`docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_HARDWARE_PERF_AUDIT_CORRECTION_V3_2026-08-24/PERF_AUDIT_CORRECTION_V3.json`

No further `perf` executable invocation is permitted before an accepted B1
snapshot admits the benign B2 capability step. Correction V3 itself admits no
implementation, Cargo, V10, PMU, remote mutation, B3, parity, B5/B6 or V12.

#### Correction V3 structural result

The first two checker attempts used the JSON triad command on Markdown and
produced zero-byte outputs. Those bytes are retained with the suffix
`failed-attempt-zero-byte`; they are not receipts. The first correct Markdown
gate then found an owner conflict because one group mixed audit, admission and
execution-sequence ownership. Its `VETO` is also retained.

The repaired local worksheet separates those three owners without changing the
audit facts, B0-B2 order or STOP boundary. The repaired local route and global
skeleton passed:

```text
global route skeleton V4       PASS
local correction repair V1     PASS
authority_ready                false
owner conflicts                    0
semantic conflicts                 0
evidence gaps                      0
```

Evidence:

```text
docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_HARDWARE_PERF_AUDIT_CORRECTION_V3_2026-08-24/global-route-skeleton-v4-receipt-v2.json
docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_HARDWARE_PERF_AUDIT_CORRECTION_V3_2026-08-24/route-perf-audit-correction-v3-receipt-v2-veto.json
docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_HARDWARE_PERF_AUDIT_CORRECTION_V3_2026-08-24/route-perf-audit-correction-v3-repair-v1-receipt.json
```

This is coherence evidence only. Controller implementation remains stopped
until a separately named implementation preflight V3 returns
`READY_TO_IMPLEMENT` with `safe_to_implement=true`.

### B0-B2 implementation V3, unrun

Implementation preflight V3 subsequently returned `READY_TO_IMPLEMENT` with
`safe_to_implement=true`. The admitted controller and source-preserving
test-module fragment now exist, but no remote research transition has run.

```text
state                                      B0_B2_TOOLS_IMPLEMENTED_UNRUN
controller self-check                      PASS, 10 focused checks
Rust parse without Cargo                   PASS
production prefix                          39,047 bytes, byte-identical
production prefix SHA-256                  ce9ea2d290602774e2f45444e10e6fdabce6af4a61ae2bdca54c6d2e31a53f26
remote actions                             0
Cargo build/check/test                     0
freezer / perf / PMU / V10 / proxy runs    0 / 0 / 0 / 0 / 0
```

The P0 source-closure ledger has 509 dependency-path records but 508 unique
normalized destination paths. The two records that normalize to the same
`data/lexicon/russian_reflexive_confusion.tsv` destination have identical
mode, size, SHA-256 and status. The controller preserves both provenance rows,
copies the byte-identical destination once and fails on any conflicting alias.
This does not upgrade `full build-time source closure = WATCH`.

Implementation evidence:

`docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_HARDWARE_B0_B2_IMPLEMENTATION_V3_2026-08-24/IMPLEMENTATION_UNRUN_RECEIPT.json`

The next permitted transition is B0a. B3, proxy parity, B5/B6, predictive
modeling and V12 remain `NOT_ADMITTED`; runtime authority changed: `false`.

#### B0a attempt V1: blocked before remote write

The first B0a dispatch failed inside its read-only identity probe. The SSH
wrapper passed a multiline `python -c` argument without remote-shell argv
quoting, so remote Python did not start. Post-failure audit proved that neither
the remote provenance root nor state root or staging path exists; no marker,
Cargo, rustc, `perf`, PMU event, V10 or diagnostic executable was created or
run.

```text
B0a result                         B0A_FAILED_BUILD_UNUSED
remote writes                     0
build/freezer markers             absent
Cargo / rustc / perf / PMU        0 / 0 / 0 / 0
runtime authority changed         false
```

Evidence:

`docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_HARDWARE_B0_B2_IMPLEMENTATION_V3_2026-08-24/B0A_ATTEMPT_V1_BLOCKED_BEFORE_REMOTE_WRITE.json`

The controller is not patched or rerun under V3. The next required artifact is
an independently named implementation preflight for the SSH argv-boundary
repair. Build and freezer markers remain unused.

#### SSH argv repair V5, unrun

The first repair preflight V4 is retained as `BLOCKED_BEFORE_CODE`; it exposed
an unbound forbidden effect, a mixed test kind and a nonterminal failure state.
V5 closed those paper defects and returned `READY_TO_IMPLEMENT`.

The admitted code change now quotes every remote argv item with `shlex.join`
and passes exactly one remote command string to OpenSSH. A local negative
control reproduced the raw-join failure, while multiline code, spaces, quotes,
metacharacters and an empty argument round-tripped byte-for-byte through the
new boundary. Empty argv and NUL fail closed.

```text
repair state                         B0_B2_TOOLS_REPAIRED_UNRUN
controller focused checks            12/12 PASS
remote actions after repair          0
Cargo / perf / PMU / V10             0 / 0 / 0 / 0
build/freezer markers                absent
```

Evidence:

`docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_HARDWARE_B0_B2_IMPLEMENTATION_V5_2026-08-24/SSH_ARGV_REPAIR_UNRUN_RECEIPT.json`

The next permitted transition is a B0a retry under V5. Build remains unused;
B3, parity, B5/B6 and V12 remain `NOT_ADMITTED`.

#### B0a attempt V2: absent output parent

The V5 transport repair worked: remote Python received the exact multiline
argv and started. The probe then failed because it tried to `stat` the future
`/home/e/.local/share/lay/provenance` output parent before B0a had created it.
No remote write or marker occurred.

A single read-only audit then checked the rest of the route instead of probing
assumptions one by one:

```text
source dependency rows / destinations     509 / 508
source byte/mode/SHA mismatches                    0
V13 bytes / SHA                            exact PASS
Cargo / rustc / LLVM                       exact PASS
nearest existing parent device                  66306
home device                                      66306
free bytes                             291,327,811,584
perf executable invoked by audit                    0
```

The correct probe rule is to resolve the nearest existing ancestor, record its
device and require every created stage/final parent to remain on that device.
V5 is retained; no retry occurs under it.

Evidence:

`docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_HARDWARE_B0_B2_IMPLEMENTATION_V5_2026-08-24/B0A_ATTEMPT_V2_BLOCKED_BEFORE_REMOTE_WRITE.json`

Build/freezer markers remain absent and B3, parity, B5/B6 and V12 remain
`NOT_ADMITTED`.

#### Parent/device repair V6: exact-byte host identity blocker

Implementation preflight V6 returned `READY_TO_IMPLEMENT` for one narrow
repair: resolve the nearest existing output ancestor before mutation and bind
every created stage/final parent to its observed filesystem device. The
controller now passes 14 local focused checks, including missing-parent parity
and injected device mismatch.

Before another B0a attempt, the repaired read-only probe was run by itself. It
proved the intended parent route:

```text
requested output parent           /home/e/.local/share/lay/provenance
requested parent exists                                           false
nearest existing parent                  /home/e/.local/share/lay
nearest existing parent device                                  66306
nearest existing parent inode                                23366571
V13 bytes / SHA                                            exact PASS
```

It then stopped on host identity. This is not host drift: direct exact-file
hashing still gives the pinned value, but the controller removed the trailing
newline before hashing.

```text
/etc/machine-id exact-file SHA  5ac0bb537745673ef80942c2df32d586f5753c85de1c8b2453aa4e7317b77441
stripped-byte SHA               ab7e08a8686d34233ac14d8d07a3ed5671ba14f0d2c08b6e3bba9eb2825ab876
failure class                   MACHINE_ID_HASH_NORMALIZATION_MISMATCH
```

Post-probe audit found no provenance root, state root, stage, marker or route
process. No Cargo build/check/test, `perf`, PMU, V10 or proxy execution
occurred. Therefore V6 is retained but cannot authorize a retry. A new
preflight must bind one exact-file-byte machine-id algorithm to the probe,
B0a receipt, build owner, B1 snapshot and B2 comparison.

Evidence:

`docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_HARDWARE_B0_B2_IMPLEMENTATION_V6_2026-08-24/PARENT_DEVICE_REPAIR_READ_ONLY_PROBE_BLOCKED.json`

B0a remains `NOT_STARTED`; build/freezer markers are absent. B3, parity,
B5/B6 and V12 remain `NOT_ADMITTED`; runtime authority changed: `false`.

#### Exact-byte machine-id repair V7, unrun

Implementation preflight V7 returned `READY_TO_IMPLEMENT` with blockers `0`
for the exact-byte identity correction only. The controller now uses SHA-256
of the complete `/etc/machine-id` file bytes in the read-only probe, B0a
receipt, build identity and environment snapshot. No runtime owner calls
`.strip()`; one stripped call remains solely as the trailing-newline negative
control.

```text
local focused checks                                  15/15 PASS
exact-file remote machine-id SHA                  5ac0bb... PASS
known stripped digest                             ab7e08... REJECTED
nearest output ancestor / device    /home/e/.local/share/lay / 66306
V13 bytes / SHA                                      exact PASS
post-probe provenance/state/stages                   false/false/0
```

This was a read-only parity run, not B0a. No remote write, marker, Cargo,
`perf`, PMU, V10 or proxy execution occurred. The next permitted transition is
B0a under V7. The one-build and one-freezer rights are still unused.

Evidence:

`docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_HARDWARE_B0_B2_IMPLEMENTATION_V7_2026-08-25/EXACT_BYTE_MACHINE_ID_REPAIR_UNRUN_RECEIPT.json`

B3, proxy parity, B5/B6 and V12 remain `NOT_ADMITTED`; runtime authority
changed: `false`.

#### Clean aggregate speed comparison V2: SUPERSEDED_UNRUN_BY_C1

The previously prepared Clean V2 B5/B6 route was superseded before execution
when the research order moved to direct C1 latency. No Clean V2 diagnostic
subject ran, no measurement was produced and its final provenance path remains
absent.

```text
subject executed               false
measurement produced           false
clean result published         false
old route physically runnable  false
remote state mode              0555
remote files / writable        2 / 0
remote SHA256SUMS              PASS
rollback                       RETAIN_TOMBSTONE
```

The exact state path is
`/home/e/.local/state/lay/slice8b-v10-clean-speed-v2-20260825`. The old
controller observes this path as consumed and rejects `run` before admission
or subject execution. A second supersession invocation was rejected and left
the immutable local receipt unchanged.

Evidence:

`docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_CLEAN_SPEED_V2_SUPERSESSION_V1_2026-08-25.json`

Receipt SHA-256:
`c6a82710d7bf4cafda33bcec21efe784605219c62d1a5fde252ff5c004351ffa`.
The complete B matrix is `HOLD_UNTIL_C1_FAIL_PAPER_DECISION`. C1 execution
still requires its own implementation preflight; V12 remains `NOT_ADMITTED`
and runtime authority changed: `false`.

#### Dirty B5/B6 speed observation V1: OBSERVED, non-authoritative

The user explicitly selected one measurement of the current loaded mini-PC
without stopping or relocating Nando or `btop`. A separate V1 preflight
admitted exactly one B5 and one B6 proxy execution using the existing sealed
ELF and schedule. It admitted no build, `perf`, PMU, host tuning, service stop,
formal B promotion or V12 action.

```text
environment                         intentionally dirty
schedule                            382 fixed requests

B5 one pinned worker
  whole executor window             1,032.815 ms
  window / request average              2.704 ms
  throughput                          369.863 requests/s
  subject /proc CPU                    1.020 s
  CPU PSI some avg10                  3.89..5.36
  Nando authority during window       104.57% CPU
  btop during window                    3.87% CPU

B6 20 pinned workers, fixed shards
  whole executor window               179.461 ms
  throughput-normalized cost            0.470 ms/request
  throughput                        2,128.596 requests/s
  subject /proc aggregate CPU           1.800 s
  CPU PSI some avg10                  6.02
  Nando authority during window       462.50% CPU
  btop during window                   22.29% CPU

maximum temperature                  69 C
thermal throttle counters            unchanged
stable host projection               unchanged
```

The controller interval starts immediately before publishing
`controller-enabled` and ends on observation of `subject-done`. The subject
polls its start marker every 10 ms, so the whole window may include up to 10 ms
of start synchronization wait. B5's `2.704 ms` is an average over all 382
requests, not a per-query p99. B6's `0.470 ms/request` is total batch wall time
divided by 382 under 20-way execution; it is throughput-normalized cost, not
request latency. Neither route observed per-query latency or p99.

The diagnostic proxy semantic-parity route was not executed, so quality is
`UNKNOWN`. This observation does not change the V7 result:

```text
B1                                  BLOCKED_ENVIRONMENT
B2 / B3 / proxy parity              NOT EXECUTED / NOT ADMITTED
formal B PASS                       false
V12                                 NOT ADMITTED
runtime authority changed           false
```

Evidence:

`docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_DIRTY_SPEED_V1_2026-08-25/REMOTE_RECEIPT.json`

Remote receipt SHA-256:
`9255310624b55d370db55800f88eb6a04172d555235aecc6587d0ac15826af48`.
The remote tree is mode `0555`, has `18` files and zero writable objects.

#### Clean aggregate speed comparison V2: PREPARED, UNRUN

A separate V2 controller is ready for one clean B5-then-B6 comparison over the
same sealed ELF, package, sidecar, V7 denominator, 382-entry schedule, CPU maps
and timing boundaries used by dirty V1. It performs three read-only quiet-host
samples before creating any marker. A blocked readiness check may be repeated;
the measurement itself is one-shot.

```text
preflight                    READY_TO_IMPLEMENT
controller self-check        PASS
readiness checks             repeatable, read-only
future route order           B5 then B6
future build / perf / PMU     none
host tuning or process stop  none
measurement attempt          unused
remote output                absent
diagnostic subject           not executed
```

The blocked-run fault path was executed once. `run` returned exit `3` before
creating the remote state directory or starting `diagnostic-test-elf`.
Current blocker observations included:

```text
Nando authority              about 108% CPU
btop                         4%..12% CPU
CPU PSI some avg10           about 5.4..5.9
IO PSI full avg10            about 2.0..5.4
clean thresholds             CPU <=2.0, IO <=0.10
```

Commands from the repository root:

```bash
scripts/lay-v10-hardware-clean.py ready
scripts/lay-v10-hardware-clean.py run
```

`run` repeats admission immediately before marker consumption. If the host is
still busy, it stops without consuming the attempt. A future successful result
will compare clean and dirty aggregate windows but will still not observe
per-query p99, promote formal B, or admit V12.

Preparation evidence:

`docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_CLEAN_SPEED_PREPARED_V2_2026-08-25.json`

#### Loaded semantic parity V1: PASS, no formal promotion

One separately preflighted semantic-parity execution used the sealed diagnostic
ELF and the immutable 382-entry schedule on the mini-PC under its existing
Nando and `btop` load. It did not repeat B5/B6, collect latency, invoke `perf`
or Cargo, tune the host, or stop a foreign process.

```text
records                              382
terminal mismatches                    0
peak mismatches                        0
completeness mismatches                0
work mismatches                        0
full-row mismatches                    0
target form retained                 382
target lemma retained                382
false certificates                     0
maximum product states            35,590
maximum scratch bytes               6,656

whole process wall, context only  4,007.824 ms
CPU PSI some avg10                 4.90 -> 4.32
Nando authority CPU                100.30%
btop CPU                              4.49%
maximum temperature                  62 C
thermal throttle counters            unchanged
stable host projection               unchanged
```

The process wall value includes test-harness startup, input loading, all parity
oracles and receipt publication. It is not executor latency, per-query p99 or
a replacement for the clean B5/B6 gate. This result proves scoped semantic
parity of the sealed V10-derived diagnostic proxy for the fixed schedule; it
does not prove full historical source closure.

```text
loaded proxy semantic parity       PASS
dirty aggregate speed observation  retained
clean speed V2 attempt              unused
B1                                  BLOCKED_ENVIRONMENT
B2 / B3                             NOT EXECUTED / NOT ADMITTED
formal B PASS                       false
V12                                 NOT ADMITTED
runtime authority changed           false
```

Evidence:

`docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_LOADED_PARITY_V1_2026-08-25/REMOTE_RECEIPT.json`

Remote receipt SHA-256:
`3d7d15b15c57c4aa1d4e7a358a2daa022ba2a4bf005f9c3dbde608befee797ad`.
The remote and local evidence trees are mode `0555`, have zero writable
objects, and pass their `SHA256SUMS` manifests. Installed Lay remained
`1.0.43`; active V11 remained
`d9edfe7346b8636096701d4ac38044d5e386e6741a086a4d8da57ac15bffdf3b`.

#### B1 environment V7: BLOCKED_ENVIRONMENT

B1 used its complete 120-second observation bound and never found three valid
quiet samples. The terminal receipt is sealed read-only:

```text
receipt SHA                12c591b447f0025548e96272a4e8d5d4f23debc8458ae99fce61b2e751673925
root mode / writable       0555 / 0
CPU PSI some avg10         10.43..11.08       required <=2.0
IO PSI full avg10          1.71..2.08         required <=0.10
procs_running              2..3               required <=2
```

The dominant blocker was not the completed Lay build:

```text
process       /opt/nando-wave/bin/nando-operator-certification-authority
PID           150005
cgroup        /system.slice/nando-operator-certification-authority.service
CPU           107.837%..107.841% through the terminal sample batch
allowed CPUs  0-19
```

`btop` also consumed 3.994%..7.988%, and one SSH observer sample consumed
3.994%. B1 modified no governor, EPP, turbo, affinity, service or process and
invoked no `perf` or PMU event. Ignoring this load would invalidate the B2
capability denominator, so the gate is not relaxed.

V7 makes B1 failure terminal. B2 was not executed and is not admitted. Further
progress requires either an explicitly authorized quiet-host maintenance
window that stops or relocates the competing Nando service and closes `btop`,
or a new paper route binding a different measurement host.

Evidence:

`docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_HARDWARE_B0_B2_IMPLEMENTATION_V7_2026-08-25/B1_BLOCKED_ENVIRONMENT_B2_NOT_ADMITTED_RECEIPT.json`

B3, proxy parity, B5/B6 and V12 remain `NOT_ADMITTED`; runtime authority
changed: `false`.

#### B0b immutable schedule closure V7: PASS

The B0b owner independently checked the freezer output and atomically
published the final schedule closure. It executed neither build nor diagnostic
code.

```text
closure receipt SHA       e162c301cb3fa0c557a66689275c558ea9bcba57480d0eccdbb0cfd4003a9b0f
root mode / writable      0555 / 0
entries / unique          382 / 382
target fields present     false
B0a closure SHA binding   48176ec6faae86f43ddda8404542367be5b6c9d6813762dedb829b4946593eb3
schedule SHA binding      2f8346d97246ac3704434279739fa5ed27705465d628890a367c21875d879e78
ELF SHA binding           f7bcee37d5dffd583577d66c982c2a28072889b8aac2ccbed22612aa6d4feb09
```

The next admitted owner is B1 environment observation. B1 may wait for a
bounded quiet window but may not tune host policy or invoke `perf`. Only a
successful B1 snapshot may admit benign B2.

Evidence:

`docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_HARDWARE_B0_B2_IMPLEMENTATION_V7_2026-08-25/B0B_PASS_B1_NOT_STARTED_RECEIPT.json`

B3, proxy parity, B5/B6 and V12 remain `NOT_ADMITTED`; runtime authority
changed: `false`.

#### One unmeasured schedule freezer V7: PASS

The freezer owner consumed `freezer.available` before executing only the
test-only schedule-freezer entrypoint. The wrapper sealed stdout, stderr,
schedule, inner receipt and hashes without `perf` or PMU.

```text
wrapper receipt SHA       c2bd03c307e576ea8b2d8bb113856e84dfb53251decdc21d4b7ad1f75e8cb801
schedule SHA              2f8346d97246ac3704434279739fa5ed27705465d628890a367c21875d879e78
schedule bytes            174,941
entries / unique          382 / 382
source ordinals           0..381
B5 requests / B6 workers  382 / 20
recursive target keys     0
executable SHA unchanged  f7bcee37d5dffd583577d66c982c2a28072889b8aac2ccbed22612aa6d4feb09
```

The root is mode `0555` with zero writable objects. No search enumeration,
latency observation, `perf` or PMU route ran. The schedule is still freezer
output, not the B0b immutable closure. The next owner may only validate and
publish B0b; build/freezer retry is forbidden.

Evidence:

`docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_HARDWARE_B0_B2_IMPLEMENTATION_V7_2026-08-25/FREEZER_PASS_B0B_UNPUBLISHED_RECEIPT.json`

B3, proxy parity, B5/B6 and V12 remain `NOT_ADMITTED`; runtime authority
changed: `false`.

#### One guarded diagnostic build V7: PASS

The diagnostic-build owner consumed `build.available` before starting one
offline release Cargo process group. Build prerequisites passed before marker
consumption; `CARGO_INCREMENTAL=0`, the pinned cargo guard and 12 GiB target
budget were used.

```text
remote executable receipt SHA  34fbe47454e60ca3ab235b97f8dbaaf108a98836faf1e771e65f92c91586cadd
ELF SHA                         f7bcee37d5dffd583577d66c982c2a28072889b8aac2ccbed22612aa6d4feb09
ELF bytes                       20,542,920
Build ID                        9829fb05f34bd353877fb6d71f1f8523e084af55
final source SHA                e0fef5dfd40dbf926fcc1eb612212770a7e5b76ba0e36a87e5149ec61e8e103b
production prefix SHA           ce9ea2d290602774e2f45444e10e6fdabce6af4a61ae2bdca54c6d2e31a53f26
root mode / writable objects    0555 / 0
executed / perf invoked         false / false
```

This object is a V10-derived diagnostic executor proxy, not reconstructed
historical V10; full source closure remains `WATCH`. Rebuild is forbidden.
The next permitted transition is one unmeasured schedule freezer. Its failure
would consume the freezer right and require a new paper revision.

Evidence:

`docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_HARDWARE_B0_B2_IMPLEMENTATION_V7_2026-08-25/DIAGNOSTIC_BUILD_PASS_FREEZER_UNUSED_RECEIPT.json`

B3, proxy parity, B5/B6 and V12 remain `NOT_ADMITTED`; runtime authority
changed: `false`.

#### B0a input closure V7: PASS

The admitted B0a owner published its final immutable closure:

```text
remote path       /home/e/.local/share/lay/provenance/slice8b-v10-hardware-b0-b2-v3-20260824/b0a-input-closure-v2
receipt SHA       48176ec6faae86f43ddda8404542367be5b6c9d6813762dedb829b4946593eb3
mode              0555
writable objects  0
source closure    509 provenance rows / 508 unique destinations
host identity     exact-file machine-id PASS
filesystem        nearest parent and created root device 66306
```

The build and freezer markers now exist as `available`; neither is consumed.
No diagnostic executable or schedule exists and no Cargo, `perf`, PMU, V10 or
proxy subject ran. The next state-machine edge is one guarded remote build
with `CARGO_INCREMENTAL=0`. Any build failure is terminal for this preflight.

Evidence:

`docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_HARDWARE_B0_B2_IMPLEMENTATION_V7_2026-08-25/B0A_PASS_BUILD_UNUSED_RECEIPT.json`

B3, proxy parity, B5/B6 and V12 remain `NOT_ADMITTED`; runtime authority
changed: `false`.

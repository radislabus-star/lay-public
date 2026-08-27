# V10 E1 Traversal D2 U-Instruction Validity Correction V7

Date: 2026-08-25

## Scope

This correction repairs one validity-denominator ambiguity in D2 paper V4 and
the primary-only sequencing overlay V5. It does not change the D2 Rust source,
the D1 evidence, the bucket attribution design, or the secondary-gap ceiling.

The following predecessors remain byte-identical:

```text
D2 V4 contract                    2a34829ac0ccf36f092a55f3c26cbbd3a1d9083834a9174eac036120e964933d
D2 V4 structural review           00438023ff26b3b5b0cc9b46dbb6687c1f9cde5687d2d222901b9d2cfdfc730e
D1 decision                       80530f9f5787f846ce2cf222c1b60e3ae42887ce95a11ac153ec7271cce98baf
D1 PMU correction V2              004bc1f5d7cd493525cfb9287e79e8159f983b41a51a2374eaeb7931c72aad38
T-CAP interpretation V3           f1d572a364312cc6c311ddc49316379b8b63748672c138f5fda50ba615cae2cb
precise capability V3             c62ec8737cecf08e69b6b8d1ce2408e051f31086b58ff53e7b32961cad12e197
secondary-gap correction V5       c6fdde13d9a2719fd69098c35cfe34eccda1ee975111731941ed86a704332f78
secondary-gap route V6 receipt    49be6e26558756188c697fa1ccf64378d855584c05418212913ed8c4457aa999
```

No `perf`, Cargo, Rust compiler, D2 subject, I-ATOM, runtime, or foreign process
is executed by this paper correction.

## Denominator Defect

D2 V4 requires `<=1%` unsampled instructions/request perturbation for fixed and
reversed mappings, but its U routes are exact component-timing routes:

```text
v10_d1_component_twenty
  -> d1_run_component_twenty
  -> d1_component_search
  -> 6 x d1_measure_phase
       CLOCK_THREAD_CPUTIME_ID
       Instant
  -> outer thread-CPU and wall clocks
```

The sealed D1 G0 instruction denominator came from a different route:

```text
v10_d1_twenty_pmu
  -> d1_run_twenty_pmu
  -> d1_search::<false>(..., false)
  -> component_clocks_enabled = false
```

Wrapping U-FIXED or U-REVERSED in `perf stat` would count component timing
instrumentation that is absent from sealed D1 G0. Not running `perf stat` would
leave the instruction conjunct without a producer. Either interpretation is
invalid.

The inspected sources and G0 wrappers are pinned as:

```text
D1 controller source               09b217d43094e532c07e1d7710a31b8e50fbce36d51df470833d77e94c50116a
D1 Rust fragment                   bbd8b8d318810eec721812f21efbeb5f231dacba774cb5ade854e2201c6c7665
P-FIXED-G0 wrapper                  a82b361a45bf70eaaeb8f40a55fb9cdacc136490742a7e981ba33d7539ef7a53
P-REVERSED-G0 wrapper               69d9dfb02a4b894cee760805d69d3412bbfc0e6bcb6777a9d7f8fd929d2d1928
```

## Effective Validity Overlay

The old combined all-U interpretation is replaced by two independent validity
families.

### CPU and thread-time perturbation

```text
U-SINGLE
U-FIXED
U-REVERSED
```

These remain exact D1 component routes. They use the existing primitive phase
samples and produce no instruction count. Their hard validity gates remain:

```text
semantic errors / unresolved                       0 / 0
structural count mismatch                             0
unsampled traversal thread CPU/edge delta            <=5% per mapping
```

### Machine-instruction perturbation

Two new one-shot validity routes are required:

```text
V-FIXED-INSTR
V-REVERSED-INSTR
```

Each V route uses the D2 symbolized ELF but executes the exact clock-free D1
PMU subject:

```text
test                     v10_d1_twenty_pmu
subject                  d1_run_twenty_pmu -> d1_search::<false>
component clocks         false
workers                  20
rounds                   20 measured barriered rounds
queries/round            382
measured requests        7,640
warmup                   one 382-query burst
fixed mapping            worker i -> CPU i
reversed mapping         worker i -> CPU 19-i
errors / unresolved      0 / 0
worker migration delta   0 for every worker
```

The V routes reproduce the exact D1 G0 measurement context:

```text
/usr/bin/sudo -n /usr/bin/perf stat
  --json-output
  --no-big-num
  --delay=-1
  --control=fifo:<control-fifo>,<ack-fifo>
  --event instructions,cycles,branches,branch-misses
  -- <exact subject command>
```

The controller waits for `subject-ready`, enables the group, records the
enable acknowledgement, publishes `controller-enabled`, waits for
`subject-done`, disables the group, records the disable acknowledgement, and
publishes `controller-disabled`. Perf must exit normally and the exact subject
receipt must pass before parsing.

Concrete paths, environment, D2 ELF identity, event argv order, FIFO paths and
one-shot markers must be pinned by the future primary-only implementation
preflight before controller creation.

## Hybrid Aggregation

V-route G0 rows use the effective interpretation already sealed by D1 PMU
correction V2. For every event:

```text
required counted PMUs                    cpu_atom and cpu_core
counted rows                             exactly one per required PMU
counter-value                            numeric scaled count
event-runtime                            positive
sum(pcnt-running)                        98.9% .. 101.1%
abs(pcnt_i - 100 * runtime_i / total)    <= 1.1 percentage points

effective_count(event) =
  sum(scaled_count_i * event_runtime_i / sum(event_runtime))
```

Runtime shares, not rounded displayed percentages, are the aggregation
weights. Missing, duplicate, unsupported, inactive, ambiguous or foreign-PMU
rows fail closed.

Only instructions/request is a hard D2 build-perturbation value:

```text
fixed sealed D1 G0 instructions/request       23,934,876.5598414
reversed sealed D1 G0 instructions/request    23,935,583.225726895

delta = abs(V instructions/request - sealed instructions/request)
        / sealed instructions/request

V-FIXED-INSTR delta                            <=1%
V-REVERSED-INSTR delta                         <=1%
```

Cycles, branches and branch misses preserve the same event-group and
multiplexing context. They are retained as raw validity evidence only and do
not create a new scientific interpretation or hard conjunct.

## Corrected Sequence

```text
primary-only final preflight READY_TO_IMPLEMENT
  -> D2-A immutable closure
  -> ONE symbolized build
  -> ELF / DWARF / PT_LOAD / Build-ID / .text audit
  -> seal D2_BUCKET_MAP
  -> parity
  -> U-SINGLE
  -> U-FIXED
  -> U-REVERSED
  -> V-FIXED-INSTR
  -> V-REVERSED-INSTR
  -> all U and V validity conjuncts PASS
  -> T-SINGLE
  -> T-FIXED
  -> T-REVERSED
  -> at most D2_ATTRIBUTION_WITH_SECONDARY_GAP
```

No T marker may be consumed before all U and V validity gates pass. Each V
marker is consumed before its route and cannot be retried after execution.

Failure classification is frozen as:

```text
wrong or missing producer identity            BLOCKED_DENOMINATOR
G0 event unavailable or incomplete            BLOCKED_CAPABILITY
instruction delta above 1%                    BLOCKED_PERTURBATION
source, ELF, mapping or receipt drift          BLOCKED_PROVENANCE
thermal throttle-counter drift                BLOCKED_THERMAL
```

Ordinary Nando, btop, K1, CPU, IO or PSI load remains part of the loaded-host
measurement condition and is not an environment blocker.

## Primary-Only Preflight Obligations

The future preflight must validate reachable command graphs and concrete argv,
not reject predecessor receipts by naive word search. Reading precise V3
evidence is allowed; executing any precise route is not.

Reachable `perf record` commands are limited to T routes with:

```text
event             task-clock:u
period            100000
freq              0
exclude_kernel    1
precise_ip         0
```

If a T route uses controller-induced SIGINT, return code `0` or `-SIGINT` proves
only controlled shutdown validity. Perf-data readers and full event, period,
loss, throttle, DSO Build-ID, unique IP-normalization, machine-byte-map,
sample-denominator, unattributed-share and paired-U perturbation validation
remain mandatory.

## Claim Boundary

Aggregate instructions/request is allowed solely to test build perturbation.
This correction does not restore or admit:

```text
instruction IP attribution
per-bucket instruction counts or shares
instruction-heavy or stall claims
I-CORE retry
I-ATOM execution
substitute precise events
SWAR, layout, rank, stack or traversal optimization
full B
V12
runtime integration or deployment
```

The maximum D2 result remains `D2_ATTRIBUTION_WITH_SECONDARY_GAP`. This paper
correction alone admits no controller, build, subject or measurement. A passing
owner-separated structural receipt is required before creation of the named
primary-only implementation preflight.

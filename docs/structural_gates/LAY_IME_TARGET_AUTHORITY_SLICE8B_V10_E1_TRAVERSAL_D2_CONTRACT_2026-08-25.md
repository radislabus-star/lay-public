# V10 E1 Traversal Internal Cost Attribution D2 Contract

Date: 2026-08-25

## Question

D1 established that exact E1 traversal owns `95.262%` of single-client
thread-CPU time and that traversal cost rises under the loaded twenty-worker
topology:

```text
single traversal       25.97 ns / examined edge
fixed traversal        44.74 ns / examined edge
reversed traversal     44.70 ns / examined edge
fixed inflation        18.77 ns / examined edge
fixed inflation share  41.95% of fixed traversal
```

D2 attributes this traversal budget to machine-code mechanisms. It does not
change the executor, implement SWAR, alter the DAFSA layout, admit V12, or
modify the installed runtime.

Authoritative predecessors:

```text
D1 decision SHA-256       80530f9f5787f846ce2cf222c1b60e3ae42887ce95a11ac153ec7271cce98baf
D1 correction V2 SHA-256  004bc1f5d7cd493525cfb9287e79e8159f983b41a51a2374eaeb7931c72aad38
D1 executable SHA-256     550f0d80ee49b114ac621b2f5099323480fd45847956a6807393511a8027d8fd
D1 verdict                CLOSED
```

## Corrected Static Closure

D2 uses the recovered V10 production source and the exact D1 test fragment:

```text
V10 source SHA-256        f6178f3882c1a216a9c2aaf7ff59dd6bdd78798708516b753ea7208177d9718c
production prefix         39,047 B
production prefix SHA-256 ce9ea2d290602774e2f45444e10e6fdabce6af4a61ae2bdca54c6d2e31a53f26
D1 fragment SHA-256       bbd8b8d318810eec721812f21efbeb5f231dacba774cb5ade854e2201c6c7665
V13 package SHA-256       cce259fe0ce5dce67702383363b66f0fe9b9ff5a87d8f01c4fcf342d91218d7b
V10 sidecar SHA-256       a1aa95be8c43fae8fedf02d3261dd1265dcbfbc8d618330fcd6449ae829df8cd
V7 denominator SHA-256    33fded73e13f565bdc08a83bd440be67c80f6d85a196453050b09bc9f7ef28e4
schedule SHA-256          2f8346d97246ac3704434279739fa5ed27705465d628890a367c21875d879e78
schedule records          382
```

The recovered V10 source decodes one state per expanded product state. Its
`edge_range()` accepts the already decoded `PackedState`; it does not call
`state()` again. Therefore a redundant second state decode is absent from the
source and may not be assumed by D2.

The exact D1 work ledger for one complete 382-query round is:

```text
expanded states             8,059,788
examined edges             25,145,756
surviving edges             8,059,024
pruned edges               17,086,732
stack pushes                8,059,788
stack pops                  8,059,788
terminal hits                  17,600
```

The source-level call ledger is:

```text
state decode calls          8,059,788
edge-range calls            8,059,788
edge decode calls          25,145,756
transition calls           25,145,756
edge rank additions        25,145,756

V10 state fields          2 x read_u32 + 2 x read_u16
state field helpers        32,239,152
V10 edge fields           3 x read_u32
edge field helpers         75,437,268
total field helpers       107,676,420
```

These are source-level calls, not measured machine loads or instructions.
Inlining, common-subexpression elimination, load coalescing, register spills,
bounds-check elimination and code motion are unresolved until the symbolized
ELF is disassembled. A machine-code map must not manufacture a
`redundant_state_decode` bucket when no distinct machine range exists.

The active V11 source uses a different 8-byte state/edge representation and is
not D2 evidence. D2 characterizes the exact recovered-V10 DAFSA view used by
the D1 executable plus the packed E1 DP state and traversal fragment.

## D2-A Immutable Input Closure

Before any build, a separate closure owner verifies the source, fragment,
Cargo.lock, toolchain, release profile, target CPU/features, package, sidecar,
V7 schedule, D1 decision, D1 correction V2, and the sealed D1 work ledger.

This phase may read and hash files. It may not build, execute a subject, open a
PMU event, create a bucket map, or publish a D2 result.

## D2-B One Symbolized Build

Exactly one new executable may be built after a separate implementation
preflight reaches `READY_TO_IMPLEMENT`. The assembled Rust source must be byte
identical to D1: exact V10 production prefix plus exact D1 fragment. No Rust
hot-loop, observer, test entrypoint or semantic edit is allowed.

Only symbolization settings may differ from D1:

```text
release optimization/profile      unchanged
target CPU/features                unchanged
incremental compilation            disabled
line tables                        enabled
ELF symbols                        retained
frame pointers                     unchanged
PGO/BOLT/instrumentation           forbidden
```

The exact Cargo command, environment and symbolization flags are frozen by the
future implementation preflight. The build marker is consumed before Cargo;
failure is terminal and no retry is allowed under the same contract.

The build receipt must publish source SHA, ELF SHA, Build ID, `.text` SHA,
symbol table presence, DWARF line-table presence and exact toolchain. If the
sealed D1 ELF remains readable, compare its `.text` bytes with D2. A difference
is retained and explained; it cannot be hidden by semantic parity.

## D2-C Pre-Measurement Machine Closure

Before any D2 subject execution, disassemble the symbolized executable and
identify every monomorphization or compiler clone reached from
`d1_enumerate_lane_prepared::<false>`.

Publish and immutably seal `D2_BUCKET_MAP.json` before the first unsampled or
sampled control. Each entry contains:

```text
ELF SHA-256 and Build ID
symbol / clone identity
instruction start and end address
source and inlined frame
bucket and sub-bucket
classification reason
exact machine bytes SHA-256
ambiguous flag
```

The map covers instruction ranges, not source statements. Compiler-generated
instructions are classified from disassembly and DWARF before sample data is
visible. Ambiguous instructions go to `UNATTRIBUTED`; they are never reassigned
after sampling.

The frozen buckets are:

```text
DAFSA_DECODE_MEMORY
  STATE_DECODE
  EDGE_RANGE_CONTROL
  EDGE_DECODE
  SYMBOL_DECODE

TRANSITION
  ALPHABET_ID
  EQUALITY_WINDOW
  FUSED_SCALAR_U64_ADVANCE

RANK
  EDGE_RANK_ADD
  TERMINAL_RANK

STACK_CONTROL
  STACK_POP
  STACK_PUSH
  PRUNE_AND_LOOP
  BUDGET_DEADLINE
  SCRATCH_BOOKKEEPING

TERMINAL
  TERMINAL_PREDICATE
  TERMINAL_DISTANCE
  FORM_REF_COLLECTION

OUTSIDE_TRAVERSAL
UNATTRIBUTED
```

`REDUNDANT_STATE_DECODE` is a reserved absent sub-bucket. It may become
non-empty only if the sealed disassembly independently proves a second state
record decode in the actual D2 machine code. Source call multiplication is not
sufficient evidence.

Instructions in shared out-of-line runtime helpers are
`OUTSIDE_TRAVERSAL` unless an unambiguous frozen callsite/range proves exclusive
ownership. D2 does not use post-result narrative assignment to recover them.

## D2-D Semantic and Unsampled Controls

The new executable first runs the exact D1 semantic parity process. Required:

```text
records                                      382/382
terminal / peak / completeness mismatch      0 / 0 / 0
work / rank / reverse mismatch               0 / 0 / 0
target form / lemma retention                382 / 382
false certificates                           0
transition stress mismatch                    0
maximum product states                       <= 35,590
maximum scratch                              <= 6,144 B
```

Then run one unsampled component process for each frozen mapping:

```text
U-SINGLE     CPU 0, 1 worker, 20 rounds
U-FIXED      worker i -> CPU i, 20 workers, 20 barriered rounds
U-REVERSED   worker i -> CPU 19-i, 20 workers, 20 barriered rounds
```

Each route uses the exact D1 component test and records its existing primitive
phase samples. No new hot-loop counters or timers are allowed.

Validity against sealed D1:

```text
structural count mismatch                         0
unsampled traversal thread CPU/edge delta       <=5% per mapping
unsampled instructions/request delta            <=1% fixed and reversed
```

The single route has no sealed D1 instruction PMU denominator and therefore
has no invented instruction-equivalence conjunct.

## D2-E External IP Sampling

Sampling uses `perf record` outside the exact D1 executable. The hot loop and
test fragment remain byte-identical.

For each mapping, run two separate one-shot processes:

```text
T-SINGLE / T-FIXED / T-REVERSED
  primary event       task-clock:u
  fixed period        100,000 ns

I-SINGLE / I-FIXED / I-REVERSED
  secondary event     precise retired instructions in user space
  fixed period        5,000,000 retired instructions
```

The implementation preflight must pin the exact resolved hybrid instruction
event syntax and prove it on a benign non-D2 process before the first D2
subject. Event substitution, frequency mode, adaptive period change and
post-result rerun are forbidden. If precise instruction sampling is unsupported,
the secondary route is `BLOCKED_CAPABILITY`; it is not silently replaced.

Each sampled process runs the same exact component route and records the
existing traversal thread-CPU denominator. Package load, worker construction,
warmup, serialization and teardown remain outside the measured component
rounds where the existing D1 protocol permits it.

Required sampling validity:

```text
semantic errors / unresolved                         0 / 0
structural mismatch                                     0
sampled traversal CPU/edge vs paired unsampled       <=5%
lost samples                                             0
adaptive sample-frequency/period change                 NO
primary traversal samples per route                 >=50,000
secondary traversal samples per route               >=20,000
UNATTRIBUTED within traversal CPU samples              <=5%
machine-byte hash mismatch                                0
```

Any failed validity conjunct produces no attribution verdict. The result is
`BLOCKED_PERTURBATION`, `BLOCKED_DENOMINATOR`, `BLOCKED_BUCKET_MAP`, or
`BLOCKED_CAPABILITY` as applicable. Thresholds are never relaxed after viewing
samples.

The host remains under its ordinary loaded workload. Nando, btop, K1 and other
foreign processes are evidence, not blockers, and are not stopped, reniced,
re-affined or policy-tuned. Thermal throttle counter drift invalidates the
affected route as `BLOCKED_THERMAL`; load and PSI do not.

## D2-F Attribution

Sample IPs are joined only to the pre-sealed machine-code map. Publish raw
sample counts by event, PMU, mapping, symbol, bucket and sub-bucket before any
derived percentage.

For primary task-clock sampling, compute:

```text
bucket_share(mapping)
  = bucket_task_clock_samples / attributed_traversal_task_clock_samples

bucket_ns_per_edge(mapping)
  = sampled_traversal_thread_cpu_ns_per_edge * bucket_share(mapping)

fixed_inflation_ns_per_edge(bucket)
  = fixed_bucket_ns_per_edge - single_bucket_ns_per_edge

reversed_inflation_ns_per_edge(bucket)
  = reversed_bucket_ns_per_edge - single_bucket_ns_per_edge
```

Also publish instruction-sample shares for the same buckets. Instruction
sampling is an independent diagnostic channel; sample shares are not silently
promoted to exact retired-instruction counts.

Publish:

```text
single / fixed / reversed ns per edge for every bucket and sub-bucket
fixed and reversed inflation contribution per bucket
task-clock and instruction sample counts and shares
OUTSIDE_TRAVERSAL and UNATTRIBUTED counts
per-PMU and per-worker diagnostics where available
sampling perturbation deltas
source ledger versus machine-code closure differences
```

The fixed-route full-inflation feasibility threshold is frozen as:

```text
(44.74 - 25.97) / 44.74 = 41.95%
```

A single bucket can be labelled `CAN_EXPLAIN_FULL_FIXED_INFLATION` only when
its preregistered lower attribution bound is at least `18.77 ns/edge`, or
`41.95%` of fixed traversal. This label means theoretical zero-cost capacity,
not that zero cost is achievable or that the bucket is causal.

## Decision

```text
D2_ATTRIBUTION_VALID
  parity, static closure, unsampled controls, both sampling channels,
  perturbation, sample denominator, bucket coverage and integrity all pass

D2_ATTRIBUTION_WITH_SECONDARY_GAP
  primary task-clock attribution passes but precise instruction sampling is
  unavailable; no single-mechanism optimization paper is admitted

BLOCKED_PROVENANCE
BLOCKED_BUCKET_MAP
BLOCKED_PERTURBATION
BLOCKED_DENOMINATOR
BLOCKED_CAPABILITY
BLOCKED_THERMAL
```

D2 may identify the next paper route:

```text
transition dominant       -> possible SWAR/bit-sliced transition paper
decode dominant           -> V10 DAFSA decode/layout paper
stack/control dominant    -> executor control-flow paper
multiple buckets required -> combined cost-model paper
no sufficient lever       -> reject single-mechanism optimization
```

No D2 result directly admits implementation. Any candidate requires a new
paper contract and implementation preflight. Full B, V12, runtime integration,
deployment and installed authority remain unadmitted.

## One-Shot Sequence

```text
paper structural review
  -> separate D2 implementation preflight
  -> D2-A immutable closure
  -> ONE symbolized build
  -> D2_BUCKET_MAP publication
  -> parity
  -> U-SINGLE -> U-FIXED -> U-REVERSED
  -> T-SINGLE -> I-SINGLE
  -> T-FIXED  -> I-FIXED
  -> T-REVERSED -> I-REVERSED
  -> immutable attribution decision
  -> STOP
```

Every executable route consumes its marker before execution. Any build or
route failure is terminal under the same contract. There are no adaptive
reruns.

## Forbidden Effects

```text
clock_gettime or counter per edge
atomic counter, branch or hook added to traversal
any D1 source or hot-loop edit
second-state-decode removal experiment
SWAR or new transition implementation
DAFSA topology, layout or symbol representation rewrite
rank, stack, pruning or budget rewrite
sample bucket reassignment after measurements
adaptive sampling period/frequency
foreign process control or host policy tuning
third loaded V10 replication or clean C1 marker use
full B, V12, runtime integration or deployment
installed Lay mutation or process restart
```

## Paper Structural Review

The first three monolithic worksheets are retained as failed revisions:

```text
V1  VETO  routes 10 > 8; two owners in d2-map
V2  VETO  abstract contract owner conflicted with concrete route owners
V3  VETO  owner conflicts closed, but unrelated mechanisms remained one
          weak composite packet
```

V4 separates one global sequence skeleton from eight local owner routes. The
global skeleton is `PASS`; all local routes are `8/8 PASS`. Across V4 there are
zero conflicts, evidence gaps, weak triads or owner conflicts. Every receipt is
structural-only and records `authority_ready=false`.

Structural review evidence:

`docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_D2_ROUTE_V4_2026-08-25/`

Effective paper state:

```text
D2 paper contract        REVIEWED
D2 implementation        NOT CREATED
D2 build/measurement     NOT STARTED
full B                    NOT ADMITTED
V12                       NOT ADMITTED
runtime integration       NOT ADMITTED
runtime authority changed false
```

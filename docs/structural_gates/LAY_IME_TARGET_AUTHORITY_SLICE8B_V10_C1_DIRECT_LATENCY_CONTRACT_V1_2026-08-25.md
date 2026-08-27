# Slice 8B V10 C1 Direct Steady-State Latency Contract V1

Status: `PAPER_CONTRACT_PENDING_STRUCTURAL_REVIEW`

Date: 2026-08-25

## Objective

C1 decides whether the exact V10 physical search route already satisfies the
accepted product-search latency thresholds on a controlled proxy host. It runs
before the complete B PMU matrix because latency need is the decision owner;
hardware diagnosis is conditional on `C1_FAIL`.

C1 is not an exact replay of historical Gate C. It preserves the exact search
function, production timing fields, percentile formula and thresholds, while
adding warmup, repetitions, affinity, clean-host admission and round barriers.
The result is named `V10-derived steady-state product latency`, never
`historical V10 latency replay`.

## Frozen production identity

The new executable may append test-only code after the existing test boundary.
Its production prefix must remain byte-identical:

```text
source                                      recovered V10 f6178f3882c1...
production lines                            1..1148
production bytes                            39,047
production prefix SHA-256                   ce9ea2d290602774e2f45444e10e6fdabce6af4a61ae2bdca54c6d2e31a53f26
first test-only line                         1149
```

Any byte, line-boundary or SHA mismatch is `BLOCKED_PROVENANCE`. A source that
is logically reconstructed but not byte-identical cannot use this contract.

The build creates one new named executable identity. It does not mutate or
replace the prior sealed diagnostic ELF `f7bcee37d5df...`. The one-build right
from B is already consumed and grants no C1 build authority; C1 requires one
new build right in its own implementation preflight.

## Frozen inputs

```text
V13 package SHA-256
cce259fe0ce5dce67702383363b66f0fe9b9ff5a87d8f01c4fcf342d91218d7b

V10 sidecar SHA-256
a1aa95be8c43fae8fedf02d3261dd1265dcbfbc8d618330fcd6449ae829df8cd

V7 denominator SHA-256
33fded73e13f565bdc08a83bd440be67c80f6d85a196453050b09bc9f7ef28e4

382-entry schedule SHA-256
2f8346d97246ac3704434279739fa5ed27705465d628890a367c21875d879e78
```

The schedule contains no recursive target fields. Query identity and source
ordinal remain fixed. No generated query, synthetic fixture, query reorder or
target-aware selection is admitted.

## Timing authority

C1 calls the exact production function:

```text
search_typed_peaks(index, package, damaged_surface, SearchBudget::proof())
```

The accepted timing fields are produced inside production code:

```text
started
  -> Phase7dCertificateOracle::new(observed)
  -> retrieval_lanes()
  -> enumerate_lane(...)
  -> terminal merge/sort/dedup
  -> result.search_elapsed_us
  -> certificate materialization
  -> result.total_elapsed_us
```

Only `result.search_elapsed_us` and `result.total_elapsed_us` participate in
PASS. Their integer-microsecond quantization is retained exactly. An outer
call-wall timer may be recorded for diagnostics but cannot enter acceptance,
replace a production field or change a threshold.

The measured loop must make the complete result observable after recording the
primitive sample:

```rust
let result = search_typed_peaks(...)?;
record(result.search_elapsed_us, result.total_elapsed_us, completeness);
std::hint::black_box(&result);
drop(result);
```

`black_box` and `drop` occur after the production-owned timing fields. They are
not included in latency authority and prevent dead-result elimination.

## Separate semantic prerequisite

The new executable identity must run exactly one separate unmeasured process
named `C1-PARITY`. It must finish before any latency process and reproduce:

```text
records                                      382
terminal mismatches                            0
peak mismatches                                0
completeness mismatches                        0
work mismatches                                0
full-row mismatches                            0
target form retained                         382
target lemma retained                        382
false certificates                             0
maximum product states                    35,590
maximum scratch bytes                       6,656
```

Parity failure is `BLOCKED_PROVENANCE` or `C1_FAIL` according to whether the
failure is identity/input or semantic. It cannot be waived by latency.

Parity runs in a separate process. It cannot serve as latency warmup and none
of its timings enter C1.

## Process order

The fixed route order is:

```text
S1 -> T1 -> T2 -> S2 -> S3 -> T3 -> T4 -> S4 -> S5 -> T5
```

This yields mean sequence positions `S=5.4` and `T=5.6`, avoiding the simple
bias of running all single-client processes before all concurrent processes.
Every item is an independent process with fresh process-local package/index
state, one route-specific warmup and one measured sample buffer.

No adaptive route order, replacement process or repetition increase is
allowed after observations are visible.

## Primitive sample format

Measured loops may write only fixed-width primitive fields to preallocated,
thread-local buffers:

```text
query_ordinal             u16
round                     u16
worker_id                 u8 or sentinel for S
search_elapsed_us         u64
total_elapsed_us          u64
completeness/error bits   fixed integer flags
```

Run identity and route identity may be attached during post-process
serialization. Capacity must be exact and allocation-free inside measured
loops. T workers own separate buffers; there is no shared sample lock.

Forbidden inside measured loops:

```text
JSON or text serialization
SHA or terminal digest
full-row execution
certificate comparison
target lookup
receipt publication
dynamic buffer growth
shared aggregation lock
perf or PMU access
external timing as PASS authority
```

All result verification, aggregation, percentile calculation and publication
occur after the measured route has finished.

## S route

Each S process is pinned to one frozen P-core logical CPU. Its SMT sibling must
have no competing user workload.

```text
load package/index once
pin process thread
one complete 382-query exact-search warmup
discard warmup timings
preallocate 38,200 samples
100 measured rounds
  -> 382 queries in frozen source order
```

Each S process produces `38,200` observations. Across five processes:

```text
per query                              500 observations
pooled S                           191,000 observations
```

## T route

Each T process creates and pins twenty workers once. Workers remain alive for
warmup and all measured rounds. The fixed historical partition is:

```text
workers 0..18                      20 contiguous queries each
worker 19                           2 contiguous queries
chunk size                          20
```

Warmup is one complete twenty-worker fixed-shard burst, not a sequential pass.
Warmup timings are discarded.

Each of 250 measured rounds has both barriers:

```text
all workers wait
START barrier
  workers execute exactly one fixed shard
END barrier
next round
```

The END barrier prevents worker 19 from starting the next round while another
worker is still finishing the current round.

Every round starts as a twenty-worker burst. After worker 19 completes its two
queries, the remaining part is a nineteen-worker workload. C1 must call this a
`twenty-worker fixed-shard burst`, never continuous twenty-client saturation.

Per T process:

```text
workers 0..18                  5,000 samples each
worker 19                        500 samples
total                          95,500 samples
```

Across five T processes: `477,500` observations.

## Environment admission

The C1 controller may observe but may not tune host policy or stop a process.
The operator provides the quiet window externally.

Before every process, exactly three quiet samples must pass the frozen
admission contract:

```text
online CPUs                              frozen topology
governor/EPP/turbo                       frozen stable projection
subject affinity                         exact route map
competing process CPU                    <=1.0% per observed process
CPU PSI some avg10                       <=2.0
memory PSI full avg10                    <=0.10
IO PSI full avg10                        <=0.10
procs_running                            <=2
starting maximum temperature             >0 C and <90 C
thermal throttle counter drift           0
```

The implementation preflight may tighten temperature or topology identities
before code, but may not weaken these inherited thresholds.

After each measured process, C1 validates:

```text
governor/EPP/turbo/topology               unchanged
subject affinity and migrations           valid, migrations 0
thermal throttle counters                 unchanged
foreign process/service activity          absent
terminal temperature                      recorded and below frozen bound
```

Post-process PSI is reported but is not compared with pre-process quiet PSI,
because the subject itself creates pressure. Applying the quiet PSI threshold
after T would invalidate the denominator with its own workload.

Any failed pre-admission, foreign interference, topology/policy drift,
migration or thermal violation makes the whole C1 result
`BLOCKED_ENVIRONMENT`. Completed partial samples remain non-authoritative and
no replacement process is admitted.

## Percentile and aggregation

C1 uses the exact historical nearest-rank formula:

```text
index = ceil(n * percentile / 100) - 1
value = sorted[index]
```

Threshold comparison is inclusive and uses integer microseconds.

Required aggregations:

```text
S pooled
  all process x round x query samples

S run
  separate search and total p99 for S1..S5

T pooled
  all process x round x worker x request samples

T run
  separate pooled search and total p99 for T1..T5

T worker fairness
  separate total p99 for each (run, worker)
  fairness value = max of all 5 x 20 worker p99 values

query diagnostics
  per-query search/total p99, worst query and max
```

The following are mandatory diagnostics but not hard conjuncts:

- T pooled and per-run search p99;
- worst-query p99;
- per-route and per-worker max;
- worker p99 spread and identity of both endpoints;
- external call-wall summaries.

No new conjunct or threshold may be added after seeing C1 samples.

## C1 acceptance

Semantic prerequisite:

```text
new executable C1-PARITY                       PASS
```

Execution integrity:

```text
errors                                              0
unresolved                                          0
sample count/identity mismatches                    0
allocation or capacity violations                  0
```

Historical-equivalent latency thresholds:

```text
S pooled search p99                             <=3,000 us
S pooled total p99                              <=5,000 us
S1..S5 search p99                               <=3,000 us each
S1..S5 total p99                                <=5,000 us each
T pooled total p99                              <=5,000 us
T1..T5 pooled total p99                         <=5,000 us each
```

C1 additional fairness threshold, intentionally stricter than historical Gate
C:

```text
max(run x worker shard total p99)                <=5,000 us
```

All conjuncts must pass. Aggregate success cannot hide a process or worker
failure.

## Verdicts

```text
C1_PASS
  all provenance, parity, environment, integrity, latency and fairness
  conjuncts pass.

C1_FAIL
  valid clean evidence exists and at least one semantic, integrity, latency or
  fairness conjunct fails.

BLOCKED_ENVIRONMENT
  clean admission or stable execution environment is invalid.

BLOCKED_PROVENANCE
  source, executable, input, schedule, publication or identity binding fails.
```

For `C1_FAIL`, publish without interpretation:

```text
single_search_excess_us
single_total_excess_us
twenty_worker_total_excess_us
fairness_excess_us
worst_query identity and metrics
worst run/worker identity and metrics
worker spread
```

No small/large miss category is preregistered. A later paper gate decides
whether targeted structural/hardware diagnosis has economic value.

## Publication and failure safety

The controller owns one C1 task state. Build, parity and every S/T process have
create-new consumed markers written before execution. A failed build, parity
or process consumes its named right. No automatic retry is allowed.

Raw primitive samples, environment records, stdout/stderr, executable/source
identities and derived receipts are written to staging. Final evidence is
sealed read-only with a verified SHA256SUMS manifest before one same-filesystem
rename. Final-path mutation after rename is forbidden.

If local indexing or documentation fails after remote publication, remote
evidence remains authoritative and immutable; recovery may only index the
existing object under a new preflight.

## Consequences and claim boundary

If `C1_PASS`:

```text
V10 physical search route does not need V12 for this latency problem
V12 latency-optimization branch                    CLOSED
runtime integration/end-to-end proof               separate future gate
```

C1 does not directly authorize installing or integrating V10. It does not
prove IBus-to-edit latency, L2/L3/L4 authority transfer, verifier latency,
multi-application behavior or runtime safety.

If `C1_FAIL`:

```text
full PMU matrix                                    still not automatic
targeted diagnosis                                 requires new paper gate
V12 implementation                                remains NOT_ADMITTED
```

Throughout C1:

```text
installed Lay mutation                            forbidden
daemon/IBus/extension restart                     forbidden
production source edit                            forbidden
package/sidecar/V7/schedule mutation              forbidden
perf/PMU                                           forbidden
host tuning or process stop by controller         forbidden
generated traffic or synthetic fixtures           forbidden
adaptive reruns/repetitions/order                  forbidden
runtime authority change                          false
```

## Admission sequence

```text
order correction structural review
  -> Clean V2 physical supersession
  -> C1 implementation preflight READY_TO_IMPLEMENT
  -> controller/test-only implementation, UNRUN
  -> one guarded source-preserving build
  -> executable seal
  -> C1-PARITY
  -> clean-host S/T matrix
  -> immutable verdict publication
```

Paper coherence is not execution authority. No C1 build, parity process or
latency process is admitted until the named implementation preflight passes.

## Clean V2 prerequisite result

The Clean V2 physical supersession prerequisite is complete:

```text
status                          SUPERSEDED_UNRUN_BY_C1
subject executed               false
measurement produced           false
old Clean V2 final path        ABSENT
remote state                   0555, 2 files, writable 0
SHA256SUMS                      PASS
rollback                       RETAIN_TOMBSTONE
```

The immutable local index is:

`docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_CLEAN_SPEED_V2_SUPERSESSION_V1_2026-08-25.json`

Its SHA-256 is
`c6a82710d7bf4cafda33bcec21efe784605219c62d1a5fde252ff5c004351ffa`.
This closes only the competing Clean V2 execution right. It does not admit a
C1 build, parity process, latency process, B matrix, V12 or runtime change.
The next edge remains a separate C1 implementation preflight.

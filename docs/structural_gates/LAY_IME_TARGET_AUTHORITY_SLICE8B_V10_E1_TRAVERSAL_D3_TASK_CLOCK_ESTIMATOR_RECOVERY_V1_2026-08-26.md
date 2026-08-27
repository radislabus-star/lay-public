# D3 - D2 task-clock estimator recovery V1

Date: 2026-08-26

## Scope

D2 is terminal and immutable:

```text
D2 terminal verdict                 BLOCKED_PROVENANCE
D2 T-SINGLE                         consumed; never retried
D2 T-FIXED / T-REVERSED             retired unconsumed
D2 attribution numbers              diagnostic only
D2 optimization authority           none
```

D3 repairs only the single-worker measurement envelope. It reuses the exact
audited D2 ELF, Build ID, bucket map, semantic parity and U/V build-validity
evidence. It does not build, alter, execute or integrate another ELF.

The complete executable route registry is:

```text
U2-SINGLE
T2-SINGLE
```

No fixed or reversed route is admitted by D3 V1. Multiworker TID ownership is
not implementation-closed and requires a later paper extension after a D3
single-worker PASS.

## Immutable predecessors

```text
D2 terminal audit receipt
  SHA-256  75dc2703e279524a76c49a455b6081634d1c67274106da45f0d7d610af95e608

D2 estimator-scope correction V1
  SHA-256  88c12093bb3cf76395b3f5c37d48991ebb90a3c34a6581d5801f3cd4fb2001f4

D2 build audit receipt
  SHA-256  4e19e2e806e2b3f04f8c0286a63f64631e7cf6ff143d9a90da4036654bc8339c

D2 bucket-map audit receipt
  SHA-256  8d2e52574adca52d5a090067cdf071afba922363ccee8fc9eb4a4158cb92cba7

D2 parity remote receipt
  SHA-256  d2519797c6b21976dc6246830e187391a344acb9ec63a7b5a7816161cf306f74

D2 U-SINGLE salvage receipt
  SHA-256  9617502776537ca4181bd9bf195e1fd5b8fbd2679f1dfd00737f128cb88bfe0b

D2 U-FIXED remote receipt
  SHA-256  0ef07db4b8a07efb2ed09c3c47d6b0a9ef88e4529dd436d5d375cecf62339b59

D2 U-REVERSED remote receipt
  SHA-256  080917d52f3e36abffb6eab47b9e56c4a1e771b4da21caaf6fa3e2cfe686a0fb

D2 V-FIXED remote receipt
  SHA-256  56c862759c95de6682571aed3d68098dab084319817fba5af3853042e0396bae

D2 V-REVERSED remote receipt
  SHA-256  5d75a502ad9e509dc6810b5494067f602d5ace1237e73f4d7913d5b9b1fc9de2
```

Reused machine identities:

```text
D2 ELF SHA-256                     bb7c68528ec3fec7074919c3c64cce233d48c26efefe9d9b32735c5f5865a178
D2 ELF size                        317,706,232 B
D2 ELF mode                        0555
D2 Build ID                        eb951f1a7526a9f1cb365040c10989aa5d3fc50f
D2 bucket map SHA-256              2b93aedf11101a8bc5fc7ed4ffaa056b50476d96cdd2eabf9d185f59ee60f846
D2 bucket map mode                 0444
```

## Why D2 failed

Whole-process D2 T-SINGLE began recording before `d1_load_inputs()` and before
`d1_pin_current_thread(0)`. Sixteen pre-pinning samples entered shared sealed
traversal ranges, so the actual stream was not the preregistered estimator.
The same run also had 15,847 throttle/unthrottle pairs and an 18.434656% hard
perturbation failure. D2 therefore remains `BLOCKED_PROVENANCE`; no old bucket
value is promoted into scientific evidence.

The target host's sealed and live sample-rate value is:

```text
/proc/sys/kernel/perf_event_max_sample_rate = 8000
```

The old fixed 100,000 ns period requested a nominal 10 kHz stream. The observed
roughly 200 us throttle intervals are consistent with rate limiting toward the
8 kHz host ceiling, but D3 does not claim that this is a uniquely proven cause.
D3 uses a lower fixed 5 kHz request as preregistered mitigation and forbids
sysctl or host-policy tuning.

## Exact launch envelope

Both routes use the same byte-identical D2 test ELF and exact component test:

```text
nanda_wave::l2_field::v13_typed_peak::tests::v10_d1_component_single
```

The subject command is wrapped by the exact remote `/usr/bin/taskset`:

```text
/usr/bin/taskset --cpu-list 6
  /usr/lib/x86_64-linux-gnu/ld-linux-x86-64.so.2
  <exact D2 ELF>
  --exact <exact component test>
  --ignored --nocapture --test-threads=1
```

This gives the test process CPU6 affinity before Rust code begins. The frozen
subject then performs:

```text
d1_load_inputs()                      CPU6
d1_parse_cpus()                       CPU6
d1_pin_current_thread(0)              transition to CPU0
one complete 382-query warmup round   CPU0
twenty measured 382-query rounds      CPU0
```

The measured subject receipt must prove `cpus=[0]`, one worker, 382 queries,
twenty measured rounds, one 382-query warmup and zero measured thread
migrations. The exact taskset SHA-256 is
`63e52c4b99a688ccd7bab6edbc6df2af1acad124eb852adcb4d20043d28eb2d3`.

## U2-SINGLE

U2 executes the exact staged subject once with no perf command and no PMU
event. It produces the paired measured denominator only:

```text
U2 measured edges = 20 * 25,145,756
U2 CPU/edge = measured traversal thread CPU ns / U2 measured edges
```

U2 hard gates are complete subject evidence, exact route geometry, exact
structure, zero errors/unresolved, worker sentinel 255, exact CPU0 final
affinity, zero measured migrations, no thermal throttle-counter drift and no
provenance drift. Comparison with old U-SINGLE is diagnostic only; it is not a
new build-validity gate.

## T2-SINGLE

T2 uses whole-process command wrapping. There is no PID attach, delayed attach,
SIGINT lifecycle, adaptive frequency or adaptive period:

```text
perf record
  --buildid-all
  --sample-cpu
  --timestamp
  --event task-clock:u
  --count 200000
  --output <new sealed perf.data>
  -- <exact staged subject command>
```

The event identity must resolve to:

```text
event             task-clock:u
sample_period     200000 ns
freq              0
exclude_kernel    1
precise_ip        0
inherit           1
```

The scientific sample stream is preregistered before execution as:

```text
exact D2 Build ID
AND exact D2 executable mapping and unique PIE load bias
AND sample CPU == 0
AND normalized IP belongs to a sealed traversal range
```

CPU6 samples are retained in the immutable evidence and excluded by the frozen
CPU-domain estimator. There is no timestamp filter and no post-hoc warmup
subtraction. The CPU0 traversal stream includes exactly one warmup round and
twenty measured rounds.

The corrected sampled denominator is therefore different from U2:

```text
T2 sampled edges = 21 * 25,145,756
T2 CPU/edge = accepted traversal samples * 200000 / T2 sampled edges

U2 measured edges = 20 * 25,145,756
U2 CPU/edge = measured traversal thread CPU ns / U2 measured edges
```

Using a common twenty-round denominator would introduce a structural 5%
warmup bias and is forbidden.

From the sealed U-SINGLE value `26.060419547537165 ns/edge`, the preregistered
feasibility projection is:

```text
expected 21-round traversal CPU      13,761,487,975.2 ns
expected samples at 200,000 ns       68,807.44
samples at the -5% boundary          65,367.07
hard minimum                         50,000
```

## Hard gates and dispatch

U2 dispatch priority:

```text
0 provenance       -> BLOCKED_PROVENANCE
1 thermal          -> BLOCKED_THERMAL
2 semantic         -> BLOCKED_SEMANTIC
```

T2 dispatch priority:

```text
0 provenance       -> BLOCKED_PROVENANCE
1 thermal          -> BLOCKED_THERMAL
2 capability       -> BLOCKED_CAPABILITY
3 bucket_map       -> BLOCKED_BUCKET_MAP
4 perturbation     -> BLOCKED_PERTURBATION
5 sample_coverage  -> BLOCKED_SAMPLE_COVERAGE
```

T2 hard predicates:

```text
exact event identity                       PASS
sample period / frequency                  200000 / 0
lost records                               0
throttle / unthrottle                      0 / 0
adaptive period                            absent
sample-rate before / after                 8000 / 8000
sample-rate or host-policy mutation        absent
subject semantic and structural checks     PASS
subject final affinity / migrations        [0] / 0
exact Build ID / unique normalization      PASS
machine-byte mismatches                    0
accepted traversal samples                 >= 50,000
UNATTRIBUTED accepted traversal samples    <= 5%
T2 sampled CPU/edge vs paired U2            <= 5%
thermal throttle-counter drift             0
```

An incomplete observation, missing predicate, ambiguous dispatch or controller
failure after marker consumption dispatches to `BLOCKED_PROVENANCE`. Every
failure is terminal in D3 V1 and grants no retry.

## Marker and state contract

D3 uses a disjoint task/transaction namespace and only two new markers:

```text
u2-single.available
t2-single.available
```

Both are created atomically only after controller self-check, exact predecessor
closure and independent bootstrap audit admission. Each marker is renamed to
`*.consumed-before-exec` before its route effect. Failure leaves the marker
consumed and retains complete evidence. `T2-SINGLE` cannot be consumed until
the sealed U2 receipt is `U2_SINGLE_PASS`.

Historical D2 markers are read-only predecessor evidence:

```text
t-fixed.available       RETIRED_UNCONSUMED_BY_TERMINAL_D2
t-reversed.available    RETIRED_UNCONSUMED_BY_TERMINAL_D2
```

They are neither renamed nor reused by D3.

## State machine

```text
D2_BLOCKED_PROVENANCE_TERMINAL
  -> D3_PREFLIGHT_READY
  -> D3_CONTROLLER_VERIFIED_UNRUN
  -> D3_NAMESPACE_AUDITED_TWO_MARKERS_AVAILABLE
  -> consume U2 marker
  -> U2_SINGLE_OBSERVED
       FAIL -> terminal BLOCKED_*
       PASS -> U2_SINGLE_PASS
  -> consume T2 marker
  -> T2_SINGLE_OBSERVED
       FAIL -> terminal BLOCKED_*
       PASS -> D3_SINGLE_ESTIMATOR_PASS
  -> STOP
```

A D3 single-worker PASS admits only a separate paper decision about a
multiworker estimator. It does not admit fixed/reversed execution, optimization,
SWAR, decoder/layout/rank/stack changes, runtime integration or deployment.


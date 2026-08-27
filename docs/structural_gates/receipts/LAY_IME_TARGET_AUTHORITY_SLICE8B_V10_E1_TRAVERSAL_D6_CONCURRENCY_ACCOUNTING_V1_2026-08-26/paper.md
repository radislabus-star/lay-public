# V10 E1 Traversal D6 Concurrency Accounting V1

Date: 2026-08-26

## Purpose

D6 explains the frozen `+18.77 ns/edge` traversal inflation as far as the
existing sealed component and PMU evidence permits. It is an offline
cross-evidence accounting route. It runs no subject, perf reader, PMU event,
Cargo command, marker transition, or runtime action.

D6 must keep three levels distinct:

```text
measured accounting       instructions, cycles, task-clock, IPC, component CPU
bounded interpretation    saturation lowers IPC and effective frequency
not yet isolated          SMT, package power/turbo, core count, or shared state
```

## Pinned Evidence

```text
D1 decision
  size                     5,361,257 B
  SHA-256                  80530f9f5787f846ce2cf222c1b60e3ae42887ce95a11ac153ec7271cce98baf

D1 root SHA256SUMS
  size                         7,021 B
  SHA-256                  c6e6f0674f773fd397a1fcc0b383fb1e7d1693a768b06e1c6fd21bb7291dcd83

C-SINGLE component samples
  size                       901,520 B
  SHA-256                  b520bcd979449e60f6a03ce477375e98e774a1999eff02d22840a0e8b07832b9

C-FIXED component samples
  size                       901,520 B
  SHA-256                  cbd7e0b3a0e303a3a4cf40ef05c19af8c1cd2f6589691bfdda46405b88f09329

C-REVERSED component samples
  size                       901,520 B
  SHA-256                  516786df8948c691f3d113fb67fe66d865dfdcdae6dca795211c8e31061e13eb

shared structure
  size                       172,206 B
  SHA-256                  90d24adee563be803c390b41b18b41624b999db37b34c26650cb362f03d06712

loaded PMU combined V3/V4 decision
  size                         4,621 B
  SHA-256                  ea9a19cace1eab5418f783dfb6c18a4de2adb7281356afffd12bb2b28cdacbd1

D4 terminal receipt
  size                         3,685 B
  SHA-256                  f748a7c448f7f142a342efa2b916a8995969044567a3aaa9a7de3aff6ba8645b

D5 offline forensic receipt
  size                         2,115 B
  SHA-256                  d44ade85316f6f6f6eeb0917d3cdea168fc083e1a52b6c3b5e88fdf2df80ae20
```

The D1 root manifest must pass before any calculation. All three component
streams must have exactly 7,640 records of 118 bytes, exact ordinals `0..381`,
rounds `0..19`, zero errors, and zero unresolved records.

## Exact Component Result

The structural denominator is identical in all routes:

```text
examined edges per round       25,145,756
measured rounds                20
total examined edges           502,915,120
```

The traversal phase is field index 3 in the frozen six-phase component record.
Its thread CPU result is:

```text
C-SINGLE                       25.96501044152341 ns/edge
C-FIXED                        44.735012045372585 ns/edge
C-REVERSED                     44.6967635154815 ns/edge

FIXED minus SINGLE             18.770001603849174 ns/edge
REVERSED minus SINGLE          18.73175307395809 ns/edge
```

This is inside traversal. The fixed outer-route increase is
`20.106360335716293 ns/edge`; certificate work adds
`1.2517951717180427 ns/edge`, while all other named non-traversal phases add
less than `0.047 ns/edge` together.

## Paired Core-Class Check

FIXED binds worker `w` to CPU `w`; REVERSED binds the same worker and exact
query chunk to CPU `19-w`. The auditor must compare the same worker/query bytes
across mappings, not compare unpaired CPU averages.

For the sixteen chunks that cross the frozen `cpu_core=0..11` and
`cpu_atom=12..19` boundary:

```text
paired P-core-class rate       42.97686435720271 ns/edge
paired E-core-class rate       46.75424073181106 ns/edge
E minus P                       3.777376374608352 ns/edge
E / P                           1.0878932521277644
```

Core class is material but cannot explain a `18.77 ns/edge` increase by itself.
This comparison is valid only inside the saturated twenty-worker envelope; it
does not predict a worker-count policy.

## PMU Accounting

The sealed B5 one-client and B6 twenty-client executor-core proxy has identical
structural work. Its exact G0 values are:

```text
                              B5                 B6
instructions/request          42,378,604.0864     42,388,812.2425
cycles/request                10,307,715.7016     14,178,114.9040
IPC                            4.1113477819         2.9897354147
task-clock/request             2.7187441387 ms      4.6910737435 ms
effective frequency           3.7913518800 GHz     3.0223602696 GHz
```

Use the identity:

```text
time = instructions / IPC / effective_frequency
```

The B5-to-B6 factors are:

```text
instructions                  1.0002408799519764
inverse IPC                   1.3751543904775503
inverse effective frequency   1.2544341315295626
combined                      1.7254561312355083
```

Applying the exact factors to the independent D1 single traversal baseline:

```text
predicted C-FIXED             44.80148646392056 ns/edge
observed C-FIXED              44.735012045372585 ns/edge
absolute residual              0.0664744185479762 ns/edge
residual / observed            about 0.149%
```

This cross-route agreement establishes that essentially all measured
inflation is accounted for by unchanged instruction work executing at lower
IPC and lower effective frequency under saturation. It does not prove the
microarchitectural cause of either decrease.

A symmetric Shapley decomposition may be published only as an accounting
convention:

```text
instruction-count contribution     0.0084 ns/edge
IPC-loss contribution             10.9427 ns/edge
frequency-loss contribution        7.8189 ns/edge
sum                               18.7700 ns/edge
```

These three additive values are not independently randomized causal effects.

## Production Boundary

The twenty-worker component route is diagnostic test code. It creates twenty
scoped workers, twenty fixed query chunks, and per-round barriers. The
production typed-peak search itself is synchronous per request. The daemon's
typing-assist boundary uses one named worker and a single pending slot.

D6 therefore does not establish that production Lay internally creates twenty
traversal workers. It establishes what happens when twenty independent
traversals saturate the pinned target host. Any service-level concurrent
request source remains a separate runtime question.

## Fastest Next Isolation

Do not repeat D5 sampling. A minimal next route must measure only these
worker-count and placement points on the same target host:

```text
1     one P-core CPU baseline
6     one logical CPU per P core, no SMT sibling
12    both logical CPUs on all P cores
14    one logical CPU per physical core, P plus E
20    full logical CPU saturation
```

For every point, freeze identical per-worker query work and record exact
instructions, cycles, task-clock, IPC, effective frequency, thread CPU/edge,
affinity, migrations, and thermal counters. This single sweep distinguishes:

```text
6 -> 12     incremental SMT penalty
6 -> 14     incremental E-core/package-load effect without P SMT
14 -> 20    incremental P-core SMT saturation
1 -> 6      all-core turbo/package-load effect on P cores
```

No runtime change follows until the sweep selects a latency/throughput policy.

## Verdicts

```text
D6_CONCURRENCY_ACCOUNTING_COMPLETE
    all pinned evidence and exact calculations pass

BLOCKED_ACCOUNTING_PROVENANCE
    any byte, manifest, schema, denominator, record, or calculation drifts
```

Neither verdict grants a code edit, build, install, restart, deployment, PMU
rerun, D5 retry, or runtime-authority change. A separate sweep contract and
implementation preflight are required.

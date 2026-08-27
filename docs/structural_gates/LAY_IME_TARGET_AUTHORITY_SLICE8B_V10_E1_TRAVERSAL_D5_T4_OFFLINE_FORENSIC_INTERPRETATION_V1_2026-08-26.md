# V10 E1 Traversal D5 T4 Offline Forensic Interpretation V1

Date: 2026-08-26

## Purpose

D5 is immutable and terminal at `BLOCKED_PROVENANCE`. This forensic route may
explain the shape of that failure from already sealed bytes. It cannot convert
T4 into PASS, recreate a marker, execute a route, or grant optimization
authority.

The audit was specified after the D5 terminal result was known. Its output is
retrospective diagnostic evidence, not preregistered D5 science.

## Pinned Inputs

```text
D5 terminal receipt
  size                         2,964 B
  SHA-256                      b37e24bd87d063063d83dd30f084d7fda81fc9bdd4f1759a1643b2e6809c741a

U4-FIXED route receipt
  size                        45,133 B
  SHA-256                      229b901d65516d7eb6041668d2974409a721e601a6acb03d52d66929895903e4

T4-FIXED route receipt
  size                        68,243 B
  SHA-256                      d337dcddcd74e95e8009e520347f6e3cf6c1319c46bf7e896862200af8f5cbbf

T4 observation
  size                        61,044 B
  SHA-256                      55501b090feb7d525d5942e732795006c3bc9e626386bf381b7174364ab3fe44

rendered samples
  size                    30,432,151 B
  SHA-256                      e50f8c8f002ea6ea7433c4375239ba101eb54bd3a6b4bde44156e43210ac21a0

raw records
  size                   151,281,354 B
  SHA-256                      bdfd0b7bf9786ad20586606156df78a810c72d0c4fca128b30c9215d461cb89a

perf.data
  size                     7,126,578 B
  SHA-256                      02c140c235641ed177139e8fd5b37eb4cdcc5fc952443326d944c2c7322d62b6

T4 subject receipt
  size                         1,737 B
  SHA-256                      341fdfb60d4874f2a0ec0b7458561bdba22706286aa2be6492733ff1d4a5119f

D2 bucket map
  size                       390,324 B
  SHA-256                      2b93aedf11101a8bc5fc7ed4ffaa056b50476d96cdd2eabf9d185f59ee60f846
```

Every input mode is `0444`. The owning `SHA256SUMS` manifests must pass before
interpretation.

## Forbidden Effects

```text
remote connection              NO
remote read/write              NO / NO
marker mutation                NO
perf reader invocation         NO
perf record/stat               NO / NO
PMU event                      NO
subject or ELF execution       NO
Cargo/rustc                    NO / NO
new ELF or bucket map          NO
runtime authority change       NO
```

`perf.data` is identity evidence only. The audit consumes the already sealed
`samples.stdout` and `raw-records.stdout`; it does not run another reader.

## Questions

The audit answers these bounded questions without assigning a cause in
advance:

1. Is the libtest parent and exact set of twenty direct worker TIDs unique in
   the sealed FORK/COMM/EXIT graph?
2. Which worker samples caused the all-samples singleton-CPU predicate to fail?
3. Do exact D2 traversal samples for each worker remain on one CPU, and does
   their aggregate CPU closure equal `0..19`?
4. What are the timestamp, CPU, IP and DSO of every foreign-CPU worker sample?
5. What diagnostic attribution results when the already frozen scientific
   filter is applied after worker identity reconstruction?
6. How many complete throttle/unthrottle pairs exist, on which CPUs, and for
   what durations?

## Interpretation Algorithm

Worker identity is reconstructed only from the exact lifecycle graph:

```text
one subject TGID
-> one direct libtest child
-> exactly twenty direct terminal worker children
-> exactly twenty matching EXIT records
```

No CPU value participates in worker identity. CPU projections are then
computed separately:

```text
all worker samples
exact-D2 worker samples
exact normalized traversal worker samples
```

The diagnostic scientific join remains:

```text
exact worker TID
AND exact D2 path/mapping
AND unique load bias
AND normalized IP in sealed traversal range
```

There is no timestamp filter, warmup subtraction, DSO basename match, pathname
base guess, sample redistribution, or post-hoc bucket reassignment.

Throttle records are paired by exact CPU and timestamp. Nested, unmatched, or
negative-duration pairs block the forensic audit.

## Diagnostic Calculations

The audit may publish:

```text
accepted traversal sample count
bucket and sub-bucket counts
UNATTRIBUTED percent
sampled traversal CPU/edge at frozen 200,000 ns period
diagnostic sampled-vs-U4 delta
throttle pair counts and durations
fixed-period sample-count projections from the valid U4 denominator
```

These values remain diagnostic because the historical T4 route violated its
frozen zero-throttle gate. They cannot be cited as D5 attribution.

## Verdicts

```text
D5_T4_FORENSIC_DIAGNOSTIC_COMPLETE
    every pinned byte and parser closure passed; diagnostic output is complete

BLOCKED_FORENSIC_PROVENANCE
    any input, manifest, lifecycle, mapping, parser, count, or pairing closure
    failed
```

Both verdicts preserve:

```text
D5 terminal verdict             BLOCKED_PROVENANCE
D5 claim_valid                  false
D5 optimization_authority       false
D5 retry_permitted              false
```

## Decision Boundary

A complete forensic receipt may inform a separate D6 paper correction. It does
not by itself admit D6 code, markers, subject execution, perf, optimization,
build, integration, install, restart, deployment, or runtime authority.

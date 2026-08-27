# V10 E1 Traversal W1 Machine-Cost Decomposition V1

Date: 2026-08-26

## Purpose

This is a read-only existing-evidence route for the single-P-core traversal
baseline. It answers the narrowest question supported by already sealed bytes:

> Which audited machine ranges dominate single-worker traversal time, and which
> exact machine-code shape is the first candidate for a separate mechanism
> paper without changing the exact candidate language or frontier?

The route runs no subject, Cargo, perf reader, PMU event, remote command, marker
transition, or runtime action. It does not continue the closed D7 concurrency
branch and is not named D8.

## Baseline

The exact D7 W1 aggregate baseline is:

```text
traversal                         25.923669775527927 ns/edge
instructions                     361.20658023962375 / edge
cycles                           103.44625074306774 / edge
IPC                                3.491731963652915
runtime-weighted effective freq    3.791036767260203 GHz
diagnostic service p99              2.793 ms
```

D7 has no instruction-IP or task-clock-IP stream. Its ELF is not the D2 ELF:

```text
D7 ELF SHA-256                    26316b349d8192c697facf1ed5929fcc7133fc8bde15bdde6eae53a438e0f138
D7 Build ID                       4d6280e7324975076be3edc4f40802c26910180a
D2 ELF SHA-256                    bb7c68528ec3fec7074919c3c64cce233d48c26efefe9d9b32735c5f5865a178
D2 Build ID                       eb951f1a7526a9f1cb365040c10989aa5d3fc50f
```

Therefore the audited D2 bucket map must never be joined directly to D7 sample
IPs. This route uses D7 only for aggregate W1 counters and uses the valid D4
stream only with its exact D2 ELF and map.

## Pinned Existing Evidence

```text
D7 terminal audit
  size                              11,633 B
  SHA-256                       db8f8fbb2ab0bbf6ba45ca9b4d2ce7c394c3de826d82961ce938adea79024f3e

D4 terminal audit
  size                               3,685 B
  SHA-256                       f748a7c448f7f142a342efa2b916a8995969044567a3aaa9a7de3aff6ba8645b

D4 T3 scientific receipt
  size                              60,563 B
  SHA-256                       dd4e3b7bb49d368fe1461c36fda0968af629293e801500770ca9dc3715a96f09

D4 U3 scientific receipt
  size                              42,897 B
  SHA-256                       db2ba1b3d4e11ac2c4edb24e382f93d282b1b5d605fcbb1636f5e346030dc000

D4 T3 perf-script sample stream
  size                          16,442,675 B
  SHA-256                       455cc9f2812dd37d79bbdb0abb4a3797f0a7fdfb137772610f3c290798be1233

audited D2 bucket map
  size                             390,324 B
  SHA-256                       2b93aedf11101a8bc5fc7ed4ffaa056b50476d96cdd2eabf9d185f59ee60f846

D2 bucket-map audit receipt
  size                               4,363 B
  SHA-256                       8d2e52574adca52d5a090067cdf071afba922363ccee8fc9eb4a4158cb92cba7

D2 ELF
  size                         317,706,232 B
  SHA-256                       bb7c68528ec3fec7074919c3c64cce233d48c26efefe9d9b32735c5f5865a178

assembled D2 source
  size                             204,722 B
  SHA-256                       6cd9edece91ac2e0c0e6dda7658e104dcf8953f1c16b1acff6108ea44ada0181

D2 V-FIXED receipt
  size                              50,090 B
  SHA-256                       56c862759c95de6682571aed3d68098dab084319817fba5af3853042e0396bae

D2 V-REVERSED receipt
  size                              50,181 B
  SHA-256                       5d75a502ad9e509dc6810b5494067f602d5ace1237e73f4d7913d5b9b1fc9de2
```

Every pinned input mode must be `0444`, except the sealed D2 ELF mode `0555`.
The complete existing manifests remain authoritative and must pass.

## D4 Scientific Validity

D4 is not part of the invalid D2 T-SINGLE stream. D4 ended with the independent
verdict `D4_SINGLE_ESTIMATOR_PASS`:

```text
event                              task-clock:u
fixed period                       200000 ns
lost                               0
throttle / unthrottle              0 / 0
accepted traversal samples         66,543
UNATTRIBUTED                       0 / 0.0%
sampled vs paired U3 delta          3.343809548379481%
normalization unique               true
machine-byte mismatches            0
```

Its scientific stream is exact D2 Build ID, CPU 0, accepted TID and normalized
IP in the audited traversal ranges. Eleven CPU-6 staging traversal samples are
excluded without timestamp subtraction. Warmup and twenty measured rounds use
the same frozen 382-query schedule; D4 estimates their combined single-worker
machine-range distribution.

## Bounded Transfer To D7 W1

The D4 and D7 ELFs are distinct, so no exact IP transfer is allowed. Mechanism
ordering is nevertheless usable as bounded prior evidence because:

```text
D4 unsampled U3                    26.07466312804435 ns/edge
D7 unsampled W1                    25.923669775527927 ns/edge
absolute delta                      0.5824536179633081%

D2 V-FIXED                         363.6345951514673 instructions/edge
D2 V-REVERSED                      363.5727721807780 instructions/edge
D7 W1                              361.20658023962375 instructions/edge
largest delta                       0.6721956477738577%
```

Both builds contain the exact production prefix and D1 traversal fragment.
These agreements permit selecting the first mechanism paper. They do not prove
that any D2 IP has the same address, bytes, or exact cycle share in D7.

## Deterministic Offline Reduction

The auditor must independently:

1. Verify every pinned identity and predecessor verdict.
2. Parse the sealed D4 `perf script` rows without running a perf reader.
3. Apply the exact accepted TID, CPU 0, D2 pathname and load-bias closure.
4. Join normalized IPs to the audited D2 map without overlap or fallback.
5. Reproduce all D4 bucket and sub-bucket counts exactly.
6. Disassemble only the pinned D2 ELF with `/usr/bin/objdump`.
7. Require every accepted sample IP to equal a decoded instruction start.
8. Publish instruction, mnemonic, range and source ownership summaries.

`objdump` is a read-only derived-data producer. Its path, version, argv, exit
status and stdout SHA-256 must be retained. No result may depend on the dirty
live source tree.

## Expected Existing-Evidence Closure

The already sealed D4 receipt requires:

```text
TRANSITION                         49,104 / 73.7928858032%
  FUSED_SCALAR_U64_ADVANCE         44,922 / 67.5082277625% of traversal
DAFSA_DECODE_MEMORY                11,138 / 16.7380490810%
STACK_CONTROL                       5,682 /  8.5388395474%
TERMINAL                              610 /  0.9167004794%
RANK                                    9 /  0.0135250890%
```

Within `FUSED_SCALAR_U64_ADVANCE`, the exact source builds seven band cells and
then performs a second minimum reduction over `cells[..len]`. The corresponding
machine block is frozen as:

```text
minimum setup                     0x778d17 .. 0x778d60
vector reduction                  0x778d60 .. 0x778d9d
scalar tail                       0x778d9d .. 0x778db7
combined samples                  20,839
share of traversal                31.3165922787%
share of fused samples            46.3892970037%
```

The exact IP `0x778d74` (`pminub xmm1,xmm0`) owns 11,903 samples, or
`17.8876816495%` of accepted traversal samples. This concentration identifies
the reduction dependency region. It must not be interpreted as 17.89 percent
latency of one `pminub`: task-clock sampling is non-precise and includes skid,
dynamic visit frequency, dependencies and neighboring machine work.

## Claim Boundary

Allowed:

```text
D4 exact machine-range time ordering
exact static D2 machine instructions and source ownership
bounded D4-to-D7 mechanism comparability
selection of one future mechanism paper
```

Forbidden:

```text
exact D7 per-IP attribution
sample share multiplied by 103.446 cycles as measured bucket cycles
claim that pminub itself costs 17.89 percent
claim that all fused-transition work is removable
code edit, Cargo, perf, PMU, subject, marker, remote or runtime action
```

The seven-cell recurrence is required by the exact radius-3 frontier. The
separate post-recurrence minimum scan is the only selected removable-work
hypothesis. Whether a running minimum actually reduces machine work and W1
latency is untested.

## Verdicts

```text
W1_MACHINE_COST_DECOMPOSITION_COMPLETE
    every identity, parse, map join, instruction-start closure and expected
    count passes

BLOCKED_W1_DECOMPOSITION_PROVENANCE
    any identity, manifest, predecessor, parse, map, instruction or expected
    value drifts
```

The positive verdict admits only a separate
`FUSED_MINIMUM_MECHANISM_PAPER`. It does not admit implementation, compilation,
execution, integration, installation, restart, deployment or runtime change.

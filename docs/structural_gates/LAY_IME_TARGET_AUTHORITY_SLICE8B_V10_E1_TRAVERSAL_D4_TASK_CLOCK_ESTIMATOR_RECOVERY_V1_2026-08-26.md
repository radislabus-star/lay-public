# D4 - D2 task-clock estimator recovery V1

Date: 2026-08-26

## Scope and immutable history

D4 is a new single-worker measurement namespace. It repairs only D3's
execution-path admission defect. It does not reopen either terminal namespace:

```text
D2 verdict                       BLOCKED_PROVENANCE
D3 verdict                       BLOCKED_PROVENANCE
D3 U2-SINGLE                     consumed; never retried
D3 T2-SINGLE                     retired unconsumed; never reused
D3 perf / PMU                    0 / 0
```

D3 did not test the 200,000 ns event, task-clock capability, the CPU-domain
estimator, sample coverage, sampling perturbation, or bucket attribution. D4
therefore creates disjoint routes named only:

```text
U3-SINGLE
T3-SINGLE
```

No D2 or D3 marker, state, receipt, ELF, map, runtime, sysctl or host policy may
be modified. D4 grants no optimization or deployment authority.

## Reused exact evidence

```text
D3 terminal audit receipt
  7f4c0fd9466cced361c21858ea2017910adc7b92db7c1edfe02f3be09d5c4299
D3 admission correction V2
  b963c0059fe6efaca746fad2ad9dc4784c7a6d4b9ea524c9faa18c6858197519
D2 terminal audit receipt
  75dc2703e279524a76c49a455b6081634d1c67274106da45f0d7d610af95e608
D2 build audit receipt
  4e19e2e806e2b3f04f8c0286a63f64631e7cf6ff143d9a90da4036654bc8339c
D2 bucket-map audit receipt
  8d2e52574adca52d5a090067cdf071afba922363ccee8fc9eb4a4158cb92cba7
D2 parity receipt
  d2519797c6b21976dc6246830e187391a344acb9ec63a7b5a7816161cf306f74

D2 ELF SHA-256
  bb7c68528ec3fec7074919c3c64cce233d48c26efefe9d9b32735c5f5865a178
D2 ELF size                     317706232
D2 ELF mode                     0555
D2 Build ID
  eb951f1a7526a9f1cb365040c10989aa5d3fc50f
D2 bucket-map SHA-256
  2b93aedf11101a8bc5fc7ed4ffaa056b50476d96cdd2eabf9d185f59ee60f846
D2 bucket-map size              390324
D2 bucket-map mode              0444
```

The target remains `e-MEGA-MINI-M1-13th`, CPUs `0-19`, core PMU `0-11`, atom
PMU `12-19`, with `perf_event_max_sample_rate=8000`. D4 does not mutate these
properties.

## Consequence analysis

The baseline D3 design created scientific markers before proving the effective
UID could traverse and write through the exact future path. The parent was
root-owned `0700`; a mode-only audit observed the defect but still admitted
execution. U2 authority was consumed before the subject failed to publish its
receipt.

Three designs were considered:

1. Change D3 permissions and recreate U2. Rejected because it violates the
   terminal receipt and one-shot authority.
2. Create D4 markers together with a corrected `0755` parent and inspect modes.
   Rejected because it preserves the same ordering defect: mode inspection is
   not an end-to-end UID or evidence-mirror proof.
3. Create no scientific markers; run a one-shot capability closure as UID `e`;
   seal it; independently copy and audit it as `e`; then create and independently
   audit U3/T3 markers. Selected because every access failure terminates before
   scientific authority exists.

D4 changes only disjoint research state and controller/evidence files. It
cannot affect candidate retention, ranking, false authority, IME consumers,
daemon concurrency, package reload, cache identity, online learning, installed
latency, RSS or allocations because no production source, package or installed
runtime is modified. The relevant second-order risks are evidence ownership,
crash boundaries, partial namespace publication, marker recreation, stale D2
identity, host-policy drift and accidental command-graph expansion. These are
closed by immutable predecessors, exclusive creation, fsync/rename boundaries,
read-only audits, exact route registries and terminal fail-closed dispatch.

Rollback is intentionally unavailable after any one-shot effect. Before marker
creation, a failed bootstrap remains terminal `BLOCKED_BEFORE_MARKERS`. After
marker creation, each consumed marker remains consumed and its route is never
repeated. D4 is removed or superseded only by a later paper; no fallback route
or second source of truth is added.

## Pre-marker UID capability closure

The D4 namespace is created once with no U3/T3 markers:

```text
D4 parent                    root:root 0755
bootstrap-v1                root:root 0555 after seal
bootstrap-v1/subject        e:e       0555 after seal
bootstrap evidence files               0444
D4 state parent             root:root 0755
route.lock                  root:root 0400 until marker creation action
```

Before sealing, the bootstrap controller must execute an actual helper as UID
`e` through the exact future parent chain. It must prove:

```text
traverse every ancestor
stat and read exact D2 ELF and D2 bucket map
create a new probe file in the exact D4 subject directory
write fixed bytes and fsync the file
rename the probe atomically and fsync the directory
reopen and read byte-identical bytes
unlink the probe and fsync the directory
publish UID_PROBE.json and UID_PROBE_BYTES.bin as e
```

The producer then seals the complete bootstrap tree. No U3/T3 marker exists at
this point. A separate auditor must use real `/usr/bin/scp` over the configured
SSH identity as user `e` to copy the exact sealed tree, verify the remote and
local manifests byte-for-byte, repeat read-only live projection, and publish an
immutable audit receipt. An SSH/SCP/access failure is
`BLOCKED_BEFORE_MARKERS`, not permission to repair and retry.

Only an exact audit PASS admits one marker-creation action. That action creates
both marker bytes in a private state stage, fsyncs them, atomically publishes
the marker directory, and publishes a state object. A second independent
read-only marker audit is required before U3. Marker creation is never repeated.

## Scientific envelope

Both routes reuse the exact audited D2 ELF and test:

```text
/usr/bin/taskset --cpu-list 6
  /usr/lib/x86_64-linux-gnu/ld-linux-x86-64.so.2
  <exact D2 ELF>
  --exact nanda_wave::l2_field::v13_typed_peak::tests::v10_d1_component_single
  --ignored --nocapture --test-threads=1
```

The frozen behavior is CPU6 input loading, internal pin to CPU0, one complete
382-query warmup round, and twenty measured 382-query rounds. Required subject
evidence includes final `cpus=[0]`, workers `1`, queries `382`, measured rounds
`20`, warmup rounds `1`, migrations `0`, errors `0`, unresolved `0`, and worker
sentinel `255`.

U3 runs once without perf or PMU:

```text
U3 denominator = 20 * 25145756 = 502915120 edges
U3 CPU/edge = measured traversal thread CPU ns / 502915120
```

Only sealed `U3_SINGLE_PASS` admits T3 marker consumption. T3 uses whole-process
wrapping, inheritance and no attach or signal lifecycle:

```text
perf record --buildid-all --sample-cpu --timestamp
  --event task-clock:u --count 200000 --output <new perf.data>
  -- <exact staged subject command>
```

Required event identity is `task-clock:u`, period `200000`, freq `0`,
exclude-kernel `1`, precise-ip `0`, inherit `1`. The preregistered scientific
stream is exact D2 Build ID and unique D2 mapping, sample CPU `0`, and normalized
IP in a sealed traversal range. CPU6 samples remain raw evidence but are not in
the estimator. No timestamp subtraction or warmup removal is allowed.

```text
T3 denominator = 21 * 25145756 = 528060876 edges
T3 CPU/edge = accepted traversal samples * 200000 / 528060876
```

## Gates and dispatch

U3 priority:

```text
0 provenance -> BLOCKED_PROVENANCE
1 thermal    -> BLOCKED_THERMAL
2 semantic   -> BLOCKED_SEMANTIC
```

T3 priority:

```text
0 provenance      -> BLOCKED_PROVENANCE
1 thermal         -> BLOCKED_THERMAL
2 capability      -> BLOCKED_CAPABILITY
3 bucket_map      -> BLOCKED_BUCKET_MAP
4 perturbation    -> BLOCKED_PERTURBATION
5 sample_coverage -> BLOCKED_SAMPLE_COVERAGE
```

T3 hard gates:

```text
event identity and fixed period          exact
lost                                      0
throttle / unthrottle                     0 / 0
adaptive period                           absent
sample-rate before / after                8000 / 8000
host-policy mutation                      absent
subject semantics and structure           PASS
final affinity / measured migrations      [0] / 0
Build ID / PIE normalization              exact / unique
machine-byte mismatches                   0
accepted traversal samples                >=50000
UNATTRIBUTED accepted traversal samples   <=5%
T3 sampled CPU/edge vs paired U3           <=5%
thermal throttle-counter drift            0
```

Unknown predicates, incomplete observations, controller errors after marker
consumption, ambiguous dispatch or conflicting causes at one priority select
`BLOCKED_PROVENANCE`. Every failure is terminal and permits no rerun.

## State machine and claim boundary

```text
D4 paper/preflight PASS
  -> bootstrap once, zero scientific markers
  -> UID capability proof as e
  -> sealed bootstrap evidence
  -> independent live + scp audit
     FAIL -> BLOCKED_BEFORE_MARKERS
  -> create U3/T3 markers once
  -> independent marker audit
     FAIL -> BLOCKED_PROVENANCE, markers remain unconsumed
  -> consume U3 marker, run U3 once
     FAIL -> terminal; T3 retired unconsumed
  -> consume T3 marker, run T3 once
  -> terminal audit
```

The only positive scientific verdict is:

```text
D4_SINGLE_ESTIMATOR_PASS
```

It establishes a valid single-worker primary task-clock attribution result and
admits only a separate paper decision about a multiworker/TID estimator or the
measured dominant bucket. It does not authorize optimization, SWAR, decoder
rewrite, build, integration, install, restart or deployment.


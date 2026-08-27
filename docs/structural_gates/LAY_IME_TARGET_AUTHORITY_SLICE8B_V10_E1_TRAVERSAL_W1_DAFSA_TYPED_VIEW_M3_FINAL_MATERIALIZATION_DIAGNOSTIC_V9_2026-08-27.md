# DAFSA Typed View M3 Final Materialization Diagnostic V9

Date: 2026-08-27

Status: `READY_FOR_STRUCTURAL_REVIEW`

## Question

Which already-required stage inside final candidate materialization accounts for
the traced long tail after the immutable V8R3 `BLOCKED_LATENCY` result?

This is a diagnostic decomposition, not a latency rerun, optimization, source
change or production authority route.

## Namespace

```text
task_id
  slice8b-v10-e1-traversal-w1-dafsa-typed-view-m3-final-materialization-v9-20260827
transaction_id
  ed21c54906eebc5a9a99afc873b3a38b8a6ca5e6003b539d019539403aa2ffb1

remote evidence parent
  /home/e/.local/share/lay/provenance/<task_id>
remote state parent
  /home/e/.local/state/lay/<task_id>
marker
  trace.available -> trace.consumed-before-exec
```

V9 has one executable route, `TRACE`. It creates one marker only after an
independent bootstrap audit and consumes that marker atomically before the sole
subject execution. A failed or interrupted route remains consumed and cannot be
repeated.

## Immutable Predecessors

```text
V8R3 latency decision SHA-256
  45e2e279997f7a93072bcfd74ad11d2566f55b442685d1be2a75e905dd543a8a
effective decision route V4 SHA-256
  2dfd7d05408e3bd4760b9cab345fb50b7cca5f724612eabae996ae39d865c07c
effective route receipt SHA-256
  e49acfdbfa3922e6761fd44ad5ef1573f7ab350de2deeff6f9f3f34b71b3e75d
V8R3 terminal receipt SHA-256
  2115a4ce7c8a79fffde043b87c5aa462014873a0365df3a6e82892f395f6aedc
V8R3 subject receipt SHA-256
  65cd8a6f08d77c192ae0eb24fa3df106ee5030e7a8bbdfdf44d08429f7d9bfd5
V8R3 terminal verdict
  BLOCKED_LATENCY
```

V8R3 remains terminal. Its marker remains consumed and its parent, state,
receipt, journal, logs and scientific evidence are read-only predecessors.

## Sealed Executable And Inputs

V9 executes the already sealed V8R3 test ELF directly. It does not copy, relink
or rebuild it.

```text
ELF path
  /home/e/.local/share/lay/provenance/
  slice8b-v10-e1-traversal-w1-dafsa-typed-view-m3-e2e-v8r3-terminal-projection-20260827/
  bootstrap-v1/m3-v8r3-test-elf
ELF size                         320,613,368 B
ELF SHA-256
  0af3cc6679396650245e924976d8a3bb432dfa6a1086b1c46bcfa7497307afea
ELF Build ID
  c6ddac7181428a303cbc51be61dd3bb115677562
ELF mode                         0555
```

Fixed remote inputs remain the exact V8R1 files:

```text
LAY-L2-RU-FULL-v13.bin
  140,556,462 B / cce259fe0ce5dce67702383363b66f0fe9b9ff5a87d8f01c4fcf342d91218d7b
slice8b-v7-fixed-13x100.json
  1,606,189 B / 33fded73e13f565bdc08a83bd440be67c80f6d85a196453050b09bc9f7ef28e4
LAY-L2-PRODUCTIVE-PARADIGM-v90.p2m
  17,309,944 B / 40fb6a9f0d92c3c7502e47f9c70230d9b86020f622b08a5c799342f13e09ce44
LAY-L2-PRODUCTIVE-PARADIGM-v90.p2r
  2,123,112 B / de7972c80448dc792759d70de99cda6ec48c3d6af337763856601db563ab167e
LAY-L1.1-RU-COMPOSITE-EN300K-PHASE8I-v9.v9.bin
  77,962,328 B / bf5a1619a89038466ef786305cf35eda5f4af5b9f12b9140f7d3cac407e2f2a7
l11-proof.json
  539,536 B / 4983930495e793c1d28c7558fe006ddf8097ee575bebb1afd3f1dba4ddb1d01d
```

The host closure remains:

```text
hostname                         e-MEGA-MINI-M1-13th
kernel                           6.8.0-124-generic
machine-id SHA-256               5ac0bb537745673ef80942c2df32d586f5753c85de1c8b2453aa4e7317b77441
scientific CPU                   0
subject UID                      e
```

## Exact Command Graph

The only subject argv is the exact V8R3 direct test route:

```text
/usr/bin/sudo -n -u e
/usr/bin/env <frozen environment including LAY_L2_FIELD_TRACE=1>
/usr/bin/taskset -c 0
<exact sealed V8R3 ELF>
--ignored
--exact
nanda_wave::l2_field::v13_typed_peak::tests::m3_v8::m3_end_to_end_physical_proof
--nocapture
--test-threads=1
```

The environment changes only output paths into the new V9 namespace and adds:

```text
LAY_L2_FIELD_TRACE=1
```

The existing V8 environment variables continue to bind the same package,
productive package, L1.1 receipt and V7 fixed proof. No dynamic loader wrapper,
attach route, Cargo, rustc, perf, PMU, daemon, IBus, install, restart, generated
traffic or synthetic fixture is reachable.

## Trace Contract

The exact accepted trace line is:

```text
productive_v90_materialization_trace \
surfaces=<u64> emitted=<u64> setup_us=<u64> projection_us=<u64> \
classify_us=<u64> gate_us=<u64> evidence_us=<u64>
```

The subject performs one `382`-case warmup and four measured rounds. Exactly
`1,910` trace lines must be parsed from complete retained stderr:

```text
0..381       warmup / FORWARD
382..763     measured 1 / FORWARD
764..1145    measured 2 / REVERSED
1146..1527   measured 3 / FORWARD
1528..1909   measured 4 / REVERSED
```

The parser assigns only deterministic phase, round, schedule and case ordinal.
It must not invent a source case identifier or a per-request join to V8R3 outer
timings. Every numeric token must parse losslessly and each row receives:

```text
traced_total_us = setup_us + projection_us + classify_us + gate_us + evidence_us
```

The `382` warmup rows are cardinality and diagnostic evidence only. Scientific
stage distributions use the `1,528` measured rows. For each stage and
`traced_total_us`, publish count, p50, nearest-rank p99, maximum and sum,
separately pooled and by measured round.

The measured trace tail is the top `ceil(1% * 1,528) = 16` rows sorted by
`traced_total_us`, with deterministic ordinal tie-breaking. Publish each stage's
share of tail aggregate time and the count of tail rows where it is the largest
stage. A stage is labeled `dominant` only if it is unique and satisfies both:

```text
tail aggregate share                   >= 80%
largest-stage count                    >= 15 / 16
```

No unique dominant stage is a valid complete result, not a failure.

## Subject Receipt Semantics

The unchanged test still evaluates its old latency thresholds and asserts its
old positive verdict. V9 does not reinterpret that assertion as diagnostic
failure. A complete V9 observation accepts either internally consistent pair:

```text
M3_END_TO_END_TEST_OWNER_PASS + exit 0
BLOCKED_LATENCY               + exit 101
```

All non-latency gates must remain true, all semantic counters must remain zero,
the fixed case/round/schedule closure must remain exact, and the receipt must
bind the exact ELF and input identities. Any other subject verdict or exit pair
is dispatched by the V9 failure taxonomy.

Tracing perturbs time through `Instant` calls and synchronous stderr writes.
V9 traced search, materialization and total latency values are diagnostic only.
They do not replace, pass, fail or amend the immutable V8R3 latency result.

## State Machine

```text
LOCAL_CONTROLLERS_VERIFIED_UNRUN
  -> independent live admission
  -> remote bootstrap (no marker)
  -> independent bootstrap audit
  -> trace.available created
  -> independent quiet audit
  -> atomic trace.available -> trace.consumed-before-exec
  -> one direct subject execution
  -> immutable wrapper, stdout, stderr, subject receipt and parsed trace
  -> independent terminal audit
  -> exactly one terminal verdict
  -> STOP
```

The remote parent and state must be absent before bootstrap. Parent ownership is
`root:root 0755`; the subject output directory is owned by `e` and writable only
there. Before marker creation, an actual UID `e` probe must traverse the full
path, read the executable and inputs, and create/fsync/rename/read/unlink a probe
inside a disposable future-subject directory. Evidence mirroring through the
same SSH credential must also pass.

All controller external actions use durable intent before dispatch and durable
completion retaining the complete structured response before verdict
classification. A missing or malformed response leaves affected facts
`UNKNOWN`; it never becomes a zero count or a retry.

## Failure Dispatch

Priority is fixed:

```text
0 provenance
1 semantic
2 capability
3 complete decomposition
```

```text
identity/mode/SHA/namespace/marker/row/parse/order/journal drift
  -> BLOCKED_PROVENANCE
semantic or non-latency V8 gate mismatch
  -> BLOCKED_SEMANTIC
subject launch, trace emission or receipt capability failure
  -> BLOCKED_CAPABILITY
all required observations complete
  -> FINAL_MATERIALIZATION_DECOMPOSED
```

Incomplete observations, unknown predicates, multiple causes at one priority or
dispatch-schema mismatch are `BLOCKED_PROVENANCE`. Every failure retains the
consumed marker and all evidence. No retry or marker recreation is permitted.

## Claim Boundary

`FINAL_MATERIALIZATION_DECOMPOSED` permits only a separate paper decision about
the measured dominant stage, or about the absence of one. It does not admit a
source edit, bypass, cache, gate weakening, candidate exception, build,
deployment or production authority.

```text
runtime authority changed              false
production authority admitted          false
Cargo / rustc                           0 / 0
perf / PMU                              0 / 0
V8R3 marker or evidence mutation        0
new V9 subject executions               exactly 1 after marker consumption
```

## Positive Verdict

```text
FINAL_MATERIALIZATION_DECOMPOSED
```

That verdict answers only the diagnostic question and stops before any
mechanism implementation.

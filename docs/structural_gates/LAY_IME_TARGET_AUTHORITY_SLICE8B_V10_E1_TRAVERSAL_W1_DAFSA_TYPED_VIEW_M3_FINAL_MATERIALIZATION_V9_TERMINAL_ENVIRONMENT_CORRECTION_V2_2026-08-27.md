# DAFSA Typed View M3 Final Materialization V9 Terminal Environment Correction V2

Date: 2026-08-27

## Decision

The V9 `TRACE` subject is never retried. Its one-shot marker remains consumed,
and the remote namespace, raw logs, parsed rows, summary, subject receipt,
wrapper, local terminal V1 receipt, and execution journal remain immutable.

Terminal audit V1 published `BLOCKED_PROVENANCE` for one reason:

```text
scientific environment drift
```

That reason is a deterministic auditor defect. The V1 auditor reconstructed the
expected environment from every command token containing `=`. This incorrectly
treated the libtest argv token `--test-threads=1` as an environment assignment.
The retained wrapper environment is otherwise byte-for-byte equal to the frozen
environment and contains no extra or missing variable.

## Immutable V1 History

```text
V9 terminal V1 receipt SHA-256
  f68b213d6404ae1e82593be8e4663de528e32accc5f7e7b5fead1cf63292616e
V9 terminal V1 SHA256SUMS SHA-256
  c03ec661768262e3a3d366c67d606b3bc6ef5cf1575f4a02dad7107c00ccf62e
V9 execution journal SHA256SUMS SHA-256
  92c6f8cf8e293773dc7f72cbc2f8dfe1ff83d8493564b8079ef6d9307329e879
V9 terminal V1 verdict
  BLOCKED_PROVENANCE

remote TRACE wrapper SHA-256
  574fc73e334132aa0ebd3eeb4b4d044ef73258bda7f5347e9f6c8cd50ffc4c89
remote stderr SHA-256
  2564fa7403a182ce15bb2213f9f8e183153d19076b928d3f3011187edffdfaf9
remote TRACE_ROWS SHA-256
  4d97d55d8b3f32aeca843cf1d44f4018dcfabd1686f6be9e3fd64a60ababcd2b
remote TRACE_SUMMARY SHA-256
  dc037e8899f85b5a7c9f01aeabf57f8b79c5cd18e8f306b5ddb552cd2a5ca027
remote subject receipt SHA-256
  01ea35ea7ead276039dc67fa097a9be6cfe20d2d85a3dc24e33c1ee294e56eb1
```

Both retained manifests pass in full: terminal `10/10`, journal `24/24`.

## Correction Namespace

```text
task_id
  slice8b-v10-e1-traversal-w1-dafsa-typed-view-m3-final-materialization-v9-offline-terminal-audit-v2-20260827
transaction_id
  8fe4616a32137a9f8b42e6a11f677fd3657d43c7c1502093272ca2a0d0d3ee5a
route
  OFFLINE_AUDIT
```

This namespace has no remote parent, remote state, marker, subject, ELF copy,
network action, Cargo, rustc, perf, PMU, daemon, IBus, installation, restart, or
runtime authority action.

## Exact Environment Parser

The corrected parser must locate the unique command segment:

```text
/usr/bin/env
  <environment assignments>
/usr/bin/taskset
```

Only the contiguous tokens strictly between those two sentinels are environment
assignments. Every such token must match:

```text
[A-Z_][A-Z0-9_]*=<value>
```

The parser must reject duplicate keys, a missing or repeated sentinel, an empty
environment segment, or any non-assignment token inside the segment. It must not
inspect tokens after `/usr/bin/taskset` for environment ownership.

The exact retained difference under the defective V1 parser is:

```text
extra expected key    --test-threads = 1
actual extra keys     none
actual missing keys   none
actual value drift    none
```

## Offline Audit Contract

The independent V2 auditor reads only the sealed local evidence. It must:

1. verify exact file mode, size, SHA-256, task, transaction, and manifest
   inventories for terminal V1 and the execution journal;
2. preserve the V1 receipt and its historical verdict unchanged;
3. verify the exact command, ELF path, input paths, one consumed marker, one
   subject execution, and zero Cargo/rustc/perf/PMU/runtime mutations;
4. reconstruct the environment only from the bounded `env..taskset` segment and
   compare it to the retained wrapper environment;
5. independently parse retained stderr into exactly `1,910` rows, assign the
   frozen warmup/round ordinals, and recompute all pooled, per-round, and 16-row
   tail distributions;
6. require exact equality with retained `TRACE_ROWS.json`,
   `TRACE_SUMMARY.json`, and wrapper summary;
7. require all semantic and non-latency gates to remain true and accept only
   the frozen `PASS/0` or `BLOCKED_LATENCY/101` subject pair;
8. publish one new immutable offline terminal receipt and stop.

No timing value from the traced run amends the immutable V8R3 latency verdict,
and no per-request join to V8R3 outer timings is claimed.

## Failure Dispatch

```text
identity, mode, SHA, manifest, command, environment, row, parse, order,
marker, journal, or claim-boundary drift
  -> BLOCKED_PROVENANCE

semantic counter or non-latency gate mismatch
  -> BLOCKED_SEMANTIC

required sealed observation absent or unreadable
  -> BLOCKED_CAPABILITY

all predicates complete
  -> FINAL_MATERIALIZATION_DECOMPOSED
```

Priority is provenance, semantic, capability, complete decomposition. Unknown
or contradictory predicates are provenance failures. No outcome permits a
subject retry or marker recreation.

## State Machine

```text
V9 terminal V1 BLOCKED_PROVENANCE (immutable)
        |
        v
correction V2 paper + structural PASS + implementation preflight
        |
        v
offline auditor static PASS
        |
        v
read exact sealed V1 trees
        |
        v
independent recomputation
        |
        +-- FINAL_MATERIALIZATION_DECOMPOSED
        +-- BLOCKED_PROVENANCE
        +-- BLOCKED_SEMANTIC
        +-- BLOCKED_CAPABILITY
        |
        v
STOP before any mechanism implementation
```

## Claim Boundary

`FINAL_MATERIALIZATION_DECOMPOSED` permits only a separate paper decision about
the measured dominant stage or its absence. It grants no source edit, build,
deployment, production authority, latency pass, optimization, or additional
experiment.


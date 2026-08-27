# Slice 8B V10 Loaded PMU Diagnosis V2 Correction

Date: 2026-08-25

## Decision

V1 terminated at its benign capability probe. The host PMU did count the CPU-0
workload on `cpu_core`, but generic `cycles` and `instructions` also emitted
inactive `cpu_atom` rows with `<not counted>`, zero event runtime and zero
running percentage. The V1 parser incorrectly treated those inactive rows as a
missing logical counter. No B5 or B6 executor window ran.

V1 remains immutable and cannot be retried. V2 uses a disjoint task, state,
marker and final identity. It changes only hybrid-row validation and the
bootstrap/preflight pins. The ELF, package, sidecar, schedule, event groups,
B5/B6 executor entrypoints, affinity, background load and claim boundary remain
unchanged.

## Hybrid Counter Contract

For each preregistered logical event, the parser partitions matching rows into
counted and inactive PMU rows.

```text
counted row:
  numeric counter value
  event runtime > 0
  running percentage = 100.00

inactive hybrid row:
  counter value = <not counted>
  event runtime = 0
  running percentage = 0.00
```

Inactive rows are retained in raw and parsed evidence but add no counter value.
`<not supported>`, a nonzero-runtime uncounted row, a scaled counted row, an
unknown PMU variant, or absence of a required PMU invalidates the window.

Required PMU coverage is fixed before execution:

```text
benign CPU-0 capability       cpu_core
B5, fixed CPU 0              cpu_core
B6, fixed CPUs 0..19         cpu_core AND cpu_atom
unqualified software event   one exact counted row
```

The aggregate logical value is the sum of counted required-PMU rows. This is
validation of perf's hybrid expansion, not adaptive event substitution.

## Sequence

```text
V2 route PASS, authority_ready=false
  -> V2 implementation preflight READY_TO_IMPLEMENT
  -> controller syntax and hybrid-parser self-check
  -> verify immutable V1 failure evidence
  -> new disjoint V2 task identity
  -> same-ELF semantic parity once, without perf
  -> one benign capability probe
  -> B5 then B6 for G0, G1, G2 and G3
  -> immutable publication
  -> STOP
```

Every V2 marker is one-shot. A V2 failure retains evidence and terminates V2.
No V1 marker is restored, renamed or consumed.

## Boundaries

V2 remains a loaded executor-proxy diagnosis. It is not a third dirty latency
run, clean C1, formal B, historical B3, per-edge cost proof, end-to-end Lay
latency proof, V12 admission or runtime promotion. Foreign processes remain
running and are observations only.

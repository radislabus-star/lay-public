# Slice 8B V10 Loaded PMU Diagnosis V4 Continuation

Date: 2026-08-25

## Decision

V3 completed G0 and G1 for B5/B6. It stopped at B5-G2 because perf emitted
`<not supported>` for the non-required atom L1-dcache-load-misses alias. The
required CPU-0/core row was counted at 100 percent, but `perf list` confirms
that this event has no atom variant and therefore cannot support a B5/B6 hybrid
comparison.

V3 remains immutable. V4 is a disjoint continuation and does not repeat G0 or
G1. It runs only the counters confirmed present on both PMU types:

```text
G2C  L1-dcache-loads, LLC-loads, LLC-load-misses
G3   dTLB-loads, dTLB-load-misses

order: B5-G2C -> B6-G2C -> B5-G3 -> B6-G3
```

The missing L1-dcache-load-miss comparison remains an explicit evidence gap.
No raw event, model-specific alias or estimate substitutes for it.

## Sequence

After route PASS and implementation preflight `READY_TO_IMPLEMENT`, V4 may edit
only the controller task/result identity, fixed group list, derived G2C metrics
and self-check. It reuses the exact sealed subject and runs one parity and
benign capability prerequisite before the four continuation windows. Any V4
failure is terminal. V1-V3 files and markers are immutable.

The hybrid runtime-weighted parser from V3 is unchanged. B5 requires one fully
running core row; B6 requires a complete core/atom runtime partition.

## Boundary

V4 plus sealed V3 may characterize aggregate instructions/cycles/branches,
common cache-access/LLC and dTLB behavior under loaded B5/B6. It cannot prove
the missing L1 miss rate, clean C1, formal B, per-edge cost, end-to-end Lay
latency, V12 admission or runtime authority. Foreign processes remain running.

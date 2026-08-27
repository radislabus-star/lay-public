# Slice 8B V10 Loaded PMU Diagnosis V3 Correction

Date: 2026-08-25

## Decision

V2 fixed inactive hybrid rows and passed parity, capability and B5-G0. B6-G0
then produced counted `cpu_core` and `cpu_atom` rows whose event runtimes exactly
partitioned the B6 task runtime. Perf reports each PMU value scaled to the full
enabled interval, so each row's running percentage is below 100 percent. V2
correctly stopped because it had admitted only individually unscaled rows.

V2 remains immutable and cannot be retried. V3 uses disjoint state, markers and
final paths. It changes only the preregistered aggregation of a complete hybrid
runtime partition. Subject bytes, event names, route order, affinity and loaded
environment remain unchanged.

## Counter Reconstruction

For a single required PMU, the V2 rule remains unchanged: one counted row,
positive runtime and `pcnt-running = 100%`; an inactive zero-runtime row for the
other PMU is retained but not summed.

For B6 hardware events, exactly one counted `cpu_core` row and one counted
`cpu_atom` row are required. Let `r_i` be event runtime and `v_i` perf's reported
scaled value. The effective aggregate is fixed as:

```text
R = r_core + r_atom
weight_i = r_i / R
effective_value = v_core * weight_core + v_atom * weight_atom
```

Admission additionally requires positive runtimes, identical PMU coverage for
every hardware event in the group, reported running percentage within 1.1
percentage points of `100 * weight_i`, and total reported percentage in
`[98.9, 101.1]`. Missing, inactive, unsupported, duplicate or unknown required
PMU rows terminate V3. Raw perf values, runtimes, reported percentages, weights
and effective values are all retained.

This is deterministic reconstruction of perf's hybrid partition, not acceptance
of generic PMU multiplexing and not adaptive event substitution.

## Sequence And Boundary

V3 requires a new route PASS and implementation preflight
`READY_TO_IMPLEMENT`, then runs one disjoint parity, capability and fixed
B5/B6 G0-G3 matrix. Any V3 failure is terminal. V1/V2 files and markers remain
immutable.

V3 is still loaded executor-proxy diagnosis only. It cannot establish clean C1,
formal B, historical attribution, per-edge cost, end-to-end Lay latency, V12
admission or runtime promotion. Foreign processes remain untouched.

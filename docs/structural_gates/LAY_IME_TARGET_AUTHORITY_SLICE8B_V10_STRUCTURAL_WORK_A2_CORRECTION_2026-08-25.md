# Slice 8B V10 Structural Work A2 Correction

Date: 2026-08-25

## A1 Terminal Result

A1 consumed its build marker and failed before producing an executable:

```text
Cargo exit                  101
compiler error              recursion limit reached while expanding stringify!
source location             A1Counters::json
production V10 compiled     not reached
observer subject executed   no
run marker consumed         no
perf / PMU / latency        no
retry permitted             false
```

Immutable local evidence:

`docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_STRUCTURAL_WORK_A1_BUILD_FAILURE_V1_2026-08-25`

The failure was caused by expanding one `serde_json::json!` object with 33
macro-generated counter fields. It is not evidence about V10 correctness,
latency, Nando, the mini-PC or the structural counts.

## Sole A2 Correction

A2 may replace only the body of `A1Counters::json`:

```text
recursive macro-generated json! object
    ->
explicit serde_json::Map insertion for the same fields
```

Field names, counter increments, traversal, allocator wrapper, parity checks,
input identities, aggregate identities and acceptance conditions must remain
unchanged. Raising the crate recursion limit is forbidden because the observer
does not need a larger global expansion budget.

The controller changes only to use a disjoint A2 task/result identity and pin
the A2 contract, route and preflight. A1 state, markers and evidence remain
immutable.

## Sequence

```text
A2 correction paper
  -> NANDA structural PASS, authority_ready=false
  -> A2 implementation preflight READY_TO_IMPLEMENT
  -> apply sole JSON-map correction and disjoint controller identity
  -> self-check
  -> ONE new guarded remote build
  -> ONE structural run only after build PASS
  -> immutable publication
  -> STOP
```

All A1 scientific and side-effect boundaries remain in force: loaded host is
not a blocker; no foreign process control, `perf`, PMU, latency acceptance,
formal B, V12, installed Lay change or runtime promotion is admitted.

The successful terminal verdict remains
`STRUCTURAL_WORK_OBSERVED_NO_PROMOTION`.

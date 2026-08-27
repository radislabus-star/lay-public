# Lay IME Target Authority Slice 8B V10 E1 Traversal D2 Primary-Only Preflight Failure-State Correction V2

Date: 2026-08-25

## Verdict

```text
D2_PRIMARY_ONLY_PREFLIGHT_FAILURE_DISPATCH_CORRECTED
```

This immutable overlay supersedes only the unconditional U, V and T
`state_machine.failure_state` values in primary-only implementation preflight
V1. It does not change the D2 design, executable route graph, marker ledger,
build, bucket map, U/V denominators, T lifecycle, validity thresholds, result
ceiling or runtime authority.

V1 remains immutable evidence:

```text
manifest SHA-256  63c723e7ba1c5ad74ba174a3fe9100acbadbe266a8b97027cce139773a712b2f
receipt SHA-256   9cc3db240f4a8472a9b121b72b32acf045b74b288dcf31fa0cc109675a2389ca
historical engine verdict  READY_TO_IMPLEMENT
historical scoped verdict  READY_TO_IMPLEMENT_PRIMARY_ONLY_D2
effective status           SUPERSEDED_READY_DISPATCH_DEFECT
```

The defect is exact: V1 assigns every U failure to `BLOCKED_PERTURBATION`,
every V failure to `BLOCKED_DENOMINATOR`, and every T failure to
`BLOCKED_SAMPLE_COVERAGE`, while its own frozen failure taxonomy distinguishes
the observed causes below. A controller implemented against V1 would have to
violate either the state machine or the taxonomy.

## Effective State Semantics

For U, V and T execution steps, `failure_state` is an intermediate observed
failure state. It is not an unconditional terminal verdict. Before publication,
the controller must apply the corresponding closed dispatch table. It records
all observed violations, selects exactly one terminal verdict using the frozen
priority order, retains the consumed marker and all raw evidence, and stops
without rerun.

An unrecognized cause, missing dispatch evidence, non-unique classification
after applying the priority rule, or dispatch implementation drift maps
fail-closed to `BLOCKED_PROVENANCE`.

## U Dispatch

Priority is first match from top to bottom:

```text
source / ELF / receipt / route / marker drift  -> BLOCKED_PROVENANCE
thermal throttle-counter drift                 -> BLOCKED_THERMAL
errors / unresolved / semantic mismatch /
structural mismatch                            -> BLOCKED_SEMANTIC
traversal thread CPU/edge delta > 5%            -> BLOCKED_PERTURBATION
```

Effective intermediate state:

```text
U_ROUTE_FAILURE_OBSERVED_REQUIRES_DISPATCH
```

## V Dispatch

Priority is first match from top to bottom:

```text
source / ELF / receipt / route / marker drift  -> BLOCKED_PROVENANCE
thermal throttle-counter drift                 -> BLOCKED_THERMAL
exact G0 unavailable or incomplete             -> BLOCKED_CAPABILITY
wrong producer / route / request count /
PMU rows / hybrid aggregation                  -> BLOCKED_DENOMINATOR
instructions/request delta > 1%                -> BLOCKED_PERTURBATION
```

Effective intermediate state:

```text
V_ROUTE_FAILURE_OBSERVED_REQUIRES_DISPATCH
```

## T Dispatch

Priority is first match from top to bottom:

```text
source / ELF / receipt / route / marker drift  -> BLOCKED_PROVENANCE
thermal throttle-counter drift                 -> BLOCKED_THERMAL
exact task-clock event cannot produce valid
evidence                                      -> BLOCKED_CAPABILITY
Build-ID / IP normalization / sealed range /
machine-byte identity failure                  -> BLOCKED_BUCKET_MAP
sampled traversal CPU/edge vs paired U > 5%    -> BLOCKED_PERTURBATION
lost samples / throttle or unthrottle /
adaptive period / traversal samples < 50,000 /
UNATTRIBUTED traversal samples > 5%             -> BLOCKED_SAMPLE_COVERAGE
```

Effective intermediate state:

```text
T_ROUTE_FAILURE_OBSERVED_REQUIRES_DISPATCH
```

## V2 Requirements

The corrected implementation preflight must:

1. Pin the exact V1 manifest and receipt as immutable superseded evidence.
2. Pin this correction by exact bytes, mode, size and SHA-256.
3. Replace only the U/V/T unconditional failure states and add the effective
   dispatch contract, tests and preservation bindings.
4. Prove the frozen non-dispatch core remains exact. Its canonical JSON
   projection SHA-256 is
   `7ec0826f0b9e954803a53b924a42bd008a9e1ff933cb3de51baf33374e24bee3`.
5. Emit `READY_TO_IMPLEMENT_PRIMARY_ONLY_D2` only if the corrected dispatch is
   exhaustive, deterministic and fail-closed.

The frozen core projection contains:

```text
authority_bearing
bucket_map_contract
build_contract
execution_route_graph
failure_taxonomy
marker_ledger
reuses_existing_implementation
runtime_comparisons
sampling_contract
scientific_future_sensitive
side_effects
source_checks
validity_contract
```

## Claim Boundary

This correction admits no controller, Cargo, rustc, build, bucket map, D2
subject, `perf`, PMU event, attribution or optimization by itself. A new V2
implementation preflight must pass first. Even a corrected positive preflight
admits only controller creation, controller self-checks and D2-A closure. Cargo
remains forbidden until a separate D2-A PASS and prior consumption of
`build.available`.

Runtime authority changed: `false`.

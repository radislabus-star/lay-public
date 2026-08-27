# NANDA Triad Worksheet

task_id: lay-v10-exact-fused-band-transition-m1-v3
domain: code
query: Check the bounded eight-route transition microproof and authority boundary

## triads

| id | subject | relation | object | evidence | confidence | subject_role | object_role | route | group |
|----|---------|----------|--------|----------|------------|--------------|-------------|-------|-------|
| s1 | G0 authority | controls | exact ordered transition trace and traversal survival | M1 One Trace parity item 1 | 1.0 | path authority | frozen event stream | trace-parity | m1-trace |
| s2 | G1 candidate | changes_only | equality input while retaining generic recurrence and minimum scan | M1 Compared Variants G1 clause | 1.0 | equality observer | exact generic recurrence | equality | m1-equality |
| s3 | U1 candidate | consumes | G1 equality bits and one packed u64 V10 state | M1 Exact Transition Input clause | 1.0 | fused implementation | exact candidate input | fused | m1-fused-input |
| s4 | U1 candidate | returns | exact next cells metadata minimum terminal distance and survive decision | M1 Exact Transition Output clause | 1.0 | fused implementation | exact candidate output | fused | m1-fused-output |
| s5 | S1 candidate | exists_only_if | frozen before U1 result inspection | M1 Compared Variants S1 clause | 1.0 | optional implementation | preregistration boundary | swar | m1-swar |
| s6 | G0 parity owner | rejects | every cell metadata minimum terminal and pruning mismatch | M1 One Trace parity items 2 through 5 | 1.0 | path authority | semantic divergence | trace-parity | m1-parity |
| s7 | physical measurement owner | attaches_after | exact trace readiness and before fixed GO | M1 One Trace physical item 3 | 1.0 | measurement owner | process PMU window | physical | m1-physical-window |
| s8 | instructions per transition | governs | M1 physical comparison | M1 Authority Metrics primary line | 1.0 | primary metric | microproof verdict | decision | m1-decision |
| s9 | physical measurement owner | records_as_diagnostic | loaded-host cycles and wall | M1 Authority Metrics diagnostic lines | 1.0 | measurement owner | diagnostic metric | physical | m1-physical-diagnostics |
| s10 | foreign-process owner | preserves | Nando btop K1 affinity priority and execution | M1 Forbidden Effects foreign-process line | 1.0 | noninterference owner | foreign work | environment | m1-noninterference |
| s11 | M1 scoped evidence | admits_only | separate full-executor candidate paper contract | M1 Verdict final paragraph | 1.0 | scoped evidence | future paper gate | boundary | m1-next-paper |
| s12 | M1 scoped evidence | does_not_admit | V12 full B runtime integration deployment or latency PASS | M1 Decision Input and Verdict boundary | 1.0 | scoped evidence | forbidden authority | boundary | m1-no-promotion |

## candidate_triads

| id | subject | relation | object | evidence | confidence | subject_role | object_role | route | group |
|----|---------|----------|--------|----------|------------|--------------|-------------|-------|-------|
| c1 | G0 authority | controls | exact ordered transition trace and traversal survival | M1 One Trace parity item 1 | 1.0 | path authority | frozen event stream | trace-parity | m1-trace |
| c2 | G1 candidate | changes_only | equality input while retaining generic recurrence and minimum scan | M1 Compared Variants G1 clause | 1.0 | equality observer | exact generic recurrence | equality | m1-equality |
| c3 | U1 candidate | consumes | G1 equality bits and one packed u64 V10 state | M1 Exact Transition Input clause | 1.0 | fused implementation | exact candidate input | fused | m1-fused-input |
| c4 | U1 candidate | returns | exact next cells metadata minimum terminal distance and survive decision | M1 Exact Transition Output clause | 1.0 | fused implementation | exact candidate output | fused | m1-fused-output |
| c5 | S1 candidate | exists_only_if | frozen before U1 result inspection | M1 Compared Variants S1 clause | 1.0 | optional implementation | preregistration boundary | swar | m1-swar |
| c6 | G0 parity owner | rejects | every cell metadata minimum terminal and pruning mismatch | M1 One Trace parity items 2 through 5 | 1.0 | path authority | semantic divergence | trace-parity | m1-parity |
| c7 | physical measurement owner | attaches_after | exact trace readiness and before fixed GO | M1 One Trace physical item 3 | 1.0 | measurement owner | process PMU window | physical | m1-physical-window |
| c8 | instructions per transition | governs | M1 physical comparison | M1 Authority Metrics primary line | 1.0 | primary metric | microproof verdict | decision | m1-decision |
| c9 | physical measurement owner | records_as_diagnostic | loaded-host cycles and wall | M1 Authority Metrics diagnostic lines | 1.0 | measurement owner | diagnostic metric | physical | m1-physical-diagnostics |
| c10 | foreign-process owner | preserves | Nando btop K1 affinity priority and execution | M1 Forbidden Effects foreign-process line | 1.0 | noninterference owner | foreign work | environment | m1-noninterference |
| c11 | M1 scoped evidence | admits_only | separate full-executor candidate paper contract | M1 Verdict final paragraph | 1.0 | scoped evidence | future paper gate | boundary | m1-next-paper |
| c12 | M1 scoped evidence | does_not_admit | V12 full B runtime integration deployment or latency PASS | M1 Decision Input and Verdict boundary | 1.0 | scoped evidence | forbidden authority | boundary | m1-no-promotion |

## notes

- V1 VETO and V2 size-only WATCH receipts remain immutable.
- V3 has eight routes by merging only same-owner input/output and PMU diagnostic relations.
- Structural PASS remains coherence-only and must leave `authority_ready=false`.

# NANDA Triad Worksheet

task_id: lay-v10-exact-fused-band-transition-m1
domain: code
query: Check the source-preserving transition microproof, physical comparison, and authority boundary

## triads

| id | subject | relation | object | evidence | confidence | subject_role | object_role | route | group |
|----|---------|----------|--------|----------|------------|--------------|-------------|-------|-------|
| s1 | G0 authority | controls | exact ordered transition trace and traversal survival | M1 contract one-trace phase | 1.0 | path authority | frozen event stream | parity | m1-trace |
| s2 | G1 candidate | changes_only | equality input while retaining generic recurrence and minimum scan | M1 variant table | 1.0 | equality observer | exact generic recurrence | equality | m1-equality |
| s3 | U1 candidate | consumes | G1 equality bits and one packed u64 V10 state | M1 transition contract | 1.0 | fused implementation | exact candidate input | fused | m1-fused |
| s4 | U1 candidate | returns | exact next cells metadata minimum terminal distance and survive decision | M1 transition contract | 1.0 | fused implementation | exact candidate output | fused | m1-fused |
| s5 | S1 candidate | exists_only_if | frozen before U1 result inspection | M1 variant table | 1.0 | optional implementation | preregistration boundary | swar | m1-swar |
| s6 | parity owner | rejects | every cell metadata minimum terminal and pruning mismatch | M1 parity phase | 1.0 | proof owner | semantic divergence | parity | m1-parity |
| s7 | physical owner | attaches_after | exact trace readiness and before fixed GO | M1 physical phase | 1.0 | measurement owner | process PMU window | pmu | m1-physical |
| s8 | instructions per transition | governs | M1 physical comparison | M1 authority metrics | 1.0 | primary metric | microproof verdict | decision | m1-decision |
| s9 | loaded-host cycles and wall | remain | diagnostic non-authority metrics | M1 authority metrics | 1.0 | diagnostic metric | loaded environment | diagnostic | m1-environment |
| s10 | foreign-process owner | preserves | Nando btop K1 affinity priority and execution | M1 forbidden effects | 1.0 | noninterference owner | foreign work | environment | m1-environment |
| s11 | M1 pass | admits_only | separate full-executor candidate paper contract | M1 verdict boundary | 1.0 | scoped evidence | future paper gate | boundary | m1-boundary |
| s12 | M1 result | does_not_admit | V12 full B runtime integration deployment or latency PASS | M1 claim boundary | 1.0 | scoped evidence | forbidden authority | boundary | m1-boundary |

## candidate_triads

| id | subject | relation | object | evidence | confidence | subject_role | object_role | route | group |
|----|---------|----------|--------|----------|------------|--------------|-------------|-------|-------|
| c1 | G0 authority | controls | exact ordered transition trace and traversal survival | M1 contract one-trace phase | 1.0 | path authority | frozen event stream | parity | m1-trace |
| c2 | G1 candidate | changes_only | equality input while retaining generic recurrence and minimum scan | M1 variant table | 1.0 | equality observer | exact generic recurrence | equality | m1-equality |
| c3 | U1 candidate | consumes | G1 equality bits and one packed u64 V10 state | M1 transition contract | 1.0 | fused implementation | exact candidate input | fused | m1-fused |
| c4 | U1 candidate | returns | exact next cells metadata minimum terminal distance and survive decision | M1 transition contract | 1.0 | fused implementation | exact candidate output | fused | m1-fused |
| c5 | S1 candidate | exists_only_if | frozen before U1 result inspection | M1 variant table | 1.0 | optional implementation | preregistration boundary | swar | m1-swar |
| c6 | parity owner | rejects | every cell metadata minimum terminal and pruning mismatch | M1 parity phase | 1.0 | proof owner | semantic divergence | parity | m1-parity |
| c7 | physical owner | attaches_after | exact trace readiness and before fixed GO | M1 physical phase | 1.0 | measurement owner | process PMU window | pmu | m1-physical |
| c8 | instructions per transition | governs | M1 physical comparison | M1 authority metrics | 1.0 | primary metric | microproof verdict | decision | m1-decision |
| c9 | loaded-host cycles and wall | remain | diagnostic non-authority metrics | M1 authority metrics | 1.0 | diagnostic metric | loaded environment | diagnostic | m1-environment |
| c10 | foreign-process owner | preserves | Nando btop K1 affinity priority and execution | M1 forbidden effects | 1.0 | noninterference owner | foreign work | environment | m1-environment |
| c11 | M1 pass | admits_only | separate full-executor candidate paper contract | M1 verdict boundary | 1.0 | scoped evidence | future paper gate | boundary | m1-boundary |
| c12 | M1 result | does_not_admit | V12 full B runtime integration deployment or latency PASS | M1 claim boundary | 1.0 | scoped evidence | forbidden authority | boundary | m1-boundary |

## notes

- G0, G1 and U1 are separate physical routes over one G0-authoritative trace.
- S1 is absent unless frozen before any U1 result is observed.
- Structural PASS is coherence-only and must leave `authority_ready=false`.

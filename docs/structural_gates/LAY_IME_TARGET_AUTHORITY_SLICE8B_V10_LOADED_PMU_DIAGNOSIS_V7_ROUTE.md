# NANDA Triad Worksheet

task_id: lay-v10-loaded-pmu-diagnosis-v4-route-v7
domain: code
query: Check the fixed common-event continuation after V3 completed G0 and G1

## triads

| id | subject | relation | object | evidence | confidence | subject_role | object_role | route | group |
|----|---------|----------|--------|----------|------------|--------------|-------------|-------|-------|
| s1 | V3 diagnosis | completed | B5 and B6 G0 and G1 | sealed V3 receipts | 1.0 | prior execution | frozen evidence | provenance | pmu-v4-provenance |
| s2 | event catalogue | excludes | atom L1-dcache-load-misses | sudo perf list without event open | 1.0 | capability evidence | unsupported comparison | support | pmu-v4-support |
| s3 | V4 G2C | contains_only | common core and atom cache events | fixed continuation contract | 1.0 | measurement owner | supported counters | measurement | pmu-v4-g2c |
| s4 | V4 G3 | contains_only | common core and atom dTLB events | fixed continuation contract | 1.0 | measurement owner | supported counters | measurement | pmu-v4-g3 |
| s5 | V4 task identity | is_disjoint_from | V1 V2 and V3 state and finals | named continuation paths | 1.0 | execution owner | immutable history | sequencing | pmu-v4-sequencing |
| s6 | parity owner | precedes | every V4 continuation PMU event | same sealed ELF prerequisite | 1.0 | proof owner | measurement owner | parity | pmu-v4-parity |
| s7 | environment owner | records_without_controlling | foreign loaded-host processes | user operating-condition decision | 1.0 | observation owner | background load | environment | pmu-v4-environment |
| s8 | V4 result | does_not_fill | missing L1 miss comparison or future authority | explicit claim boundary | 1.0 | scoped evidence | evidence gap | boundary | pmu-v4-boundary |

## candidate_triads

| id | subject | relation | object | evidence | confidence | subject_role | object_role | route | group |
|----|---------|----------|--------|----------|------------|--------------|-------------|-------|-------|
| c1 | V3 diagnosis | completed | B5 and B6 G0 and G1 | sealed V3 receipts | 1.0 | prior execution | frozen evidence | provenance | pmu-v4-provenance |
| c2 | event catalogue | excludes | atom L1-dcache-load-misses | sudo perf list without event open | 1.0 | capability evidence | unsupported comparison | support | pmu-v4-support |
| c3 | V4 G2C | contains_only | common core and atom cache events | fixed continuation contract | 1.0 | measurement owner | supported counters | measurement | pmu-v4-g2c |
| c4 | V4 G3 | contains_only | common core and atom dTLB events | fixed continuation contract | 1.0 | measurement owner | supported counters | measurement | pmu-v4-g3 |
| c5 | V4 task identity | is_disjoint_from | V1 V2 and V3 state and finals | named continuation paths | 1.0 | execution owner | immutable history | sequencing | pmu-v4-sequencing |
| c6 | parity owner | precedes | every V4 continuation PMU event | same sealed ELF prerequisite | 1.0 | proof owner | measurement owner | parity | pmu-v4-parity |
| c7 | environment owner | records_without_controlling | foreign loaded-host processes | user operating-condition decision | 1.0 | observation owner | background load | environment | pmu-v4-environment |
| c8 | V4 result | does_not_fill | missing L1 miss comparison or future authority | explicit claim boundary | 1.0 | scoped evidence | evidence gap | boundary | pmu-v4-boundary |

## notes

- G0 and G1 are not rerun.
- Unsupported L1 miss evidence is omitted, not substituted.
- Structural PASS is coherence only; implementation requires a separate preflight.

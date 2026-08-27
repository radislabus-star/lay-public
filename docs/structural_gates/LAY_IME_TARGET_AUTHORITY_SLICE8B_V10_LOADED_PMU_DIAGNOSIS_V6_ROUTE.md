# NANDA Triad Worksheet

task_id: lay-v10-loaded-pmu-diagnosis-v3-route-v6
domain: code
query: Check runtime-weighted hybrid PMU diagnosis after the sealed V2 partition failure

## triads

| id | subject | relation | object | evidence | confidence | subject_role | object_role | route | group |
|----|---------|----------|--------|----------|------------|--------------|-------------|-------|-------|
| s1 | V2 diagnosis | terminated_after | B5-G0 and B6-G0 raw capture | sealed V2 failure receipt | 1.0 | prior execution | frozen evidence | provenance | pmu-v3-provenance |
| s2 | V3 task identity | is_disjoint_from | V1 and V2 state finals and markers | V3 correction | 1.0 | execution owner | immutable history | sequencing | pmu-v3-sequencing |
| s3 | partition validator | weights | counted core and atom values by event runtime | fixed reconstruction equation | 1.0 | validation owner | effective counter | parsing | pmu-v3-partition |
| s4 | B5 validator | requires | one fully running core row | fixed CPU-0 affinity | 1.0 | validation owner | single-PMU evidence | measurement | pmu-v3-b5 |
| s5 | B6 validator | requires | complete core and atom runtime partition | fixed CPUs 0 through 19 | 1.0 | validation owner | hybrid-PMU evidence | measurement | pmu-v3-b6 |
| s6 | parity owner | precedes | every V3 PMU subject execution | same sealed ELF prerequisite | 1.0 | proof owner | measurement owner | parity | pmu-v3-parity |
| s7 | environment owner | records_without_controlling | foreign host load | loaded-host user decision | 1.0 | observation owner | background processes | environment | pmu-v3-environment |
| s8 | V3 result | does_not_admit | clean C1 formal B V12 or deployment | explicit correction boundary | 1.0 | scoped evidence | future authority | boundary | pmu-v3-boundary |

## candidate_triads

| id | subject | relation | object | evidence | confidence | subject_role | object_role | route | group |
|----|---------|----------|--------|----------|------------|--------------|-------------|-------|-------|
| c1 | V2 diagnosis | terminated_after | B5-G0 and B6-G0 raw capture | sealed V2 failure receipt | 1.0 | prior execution | frozen evidence | provenance | pmu-v3-provenance |
| c2 | V3 task identity | is_disjoint_from | V1 and V2 state finals and markers | V3 correction | 1.0 | execution owner | immutable history | sequencing | pmu-v3-sequencing |
| c3 | partition validator | weights | counted core and atom values by event runtime | fixed reconstruction equation | 1.0 | validation owner | effective counter | parsing | pmu-v3-partition |
| c4 | B5 validator | requires | one fully running core row | fixed CPU-0 affinity | 1.0 | validation owner | single-PMU evidence | measurement | pmu-v3-b5 |
| c5 | B6 validator | requires | complete core and atom runtime partition | fixed CPUs 0 through 19 | 1.0 | validation owner | hybrid-PMU evidence | measurement | pmu-v3-b6 |
| c6 | parity owner | precedes | every V3 PMU subject execution | same sealed ELF prerequisite | 1.0 | proof owner | measurement owner | parity | pmu-v3-parity |
| c7 | environment owner | records_without_controlling | foreign host load | loaded-host user decision | 1.0 | observation owner | background processes | environment | pmu-v3-environment |
| c8 | V3 result | does_not_admit | clean C1 formal B V12 or deployment | explicit correction boundary | 1.0 | scoped evidence | future authority | boundary | pmu-v3-boundary |

## notes

- V1 and V2 route failures are immutable and have no retry right.
- Structural PASS is coherence only; implementation requires a separate preflight.
- Atomic publication remains an implementation-preflight obligation.

# NANDA Triad Worksheet

task_id: lay-v10-loaded-pmu-diagnosis-v2
domain: code
query: Check the disjoint hybrid-PMU repair route after the sealed V1 capability failure

## triads

| id | subject | relation | object | evidence | confidence | subject_role | object_role | route | group |
|----|---------|----------|--------|----------|------------|--------------|-------------|-------|-------|
| s1 | V1 diagnosis | terminated_before | every executor PMU window | sealed capability failure receipt | 1.0 | prior execution | frozen failure | provenance | loaded-pmu-v2 |
| s2 | V2 task identity | is_disjoint_from | V1 final state and markers | correction contract | 1.0 | retry owner | immutable history | sequencing | loaded-pmu-v2 |
| s3 | hybrid parser | aggregates | fully counted required PMU rows | fixed cpu_core and cpu_atom coverage contract | 1.0 | validation owner | counter evidence | parsing | loaded-pmu-v2 |
| s4 | hybrid parser | retains_without_summing | zero-runtime inactive PMU rows | fixed inactive-row predicate | 1.0 | validation owner | raw evidence | parsing | loaded-pmu-v2 |
| s5 | B5 measurement | requires | counted cpu_core row | CPU 0 belongs to cpu_core 0 through 11 | 1.0 | measurement owner | PMU coverage | measurement | loaded-pmu-v2 |
| s6 | B6 measurement | requires | counted cpu_core and cpu_atom rows | CPUs 0 through 19 span both PMUs | 1.0 | measurement owner | PMU coverage | measurement | loaded-pmu-v2 |
| s7 | parity owner | precedes | every V2 PMU subject execution | same sealed ELF semantic prerequisite | 1.0 | proof owner | measurement owner | parity | loaded-pmu-v2 |
| s8 | environment owner | records_without_controlling | Nando btop K1 and host load | loaded-host user decision | 1.0 | observation owner | foreign load | environment | loaded-pmu-v2 |
| s9 | V2 publication owner | preserves | raw counted inactive parity and environment evidence | immutable final rename | 1.0 | evidence owner | diagnostic artifact | publication | loaded-pmu-v2 |
| s10 | V2 result | does_not_admit | formal B clean C1 V12 or runtime deployment | correction claim boundary | 1.0 | scoped evidence | future authority | boundary | loaded-pmu-v2 |

## candidate_triads

| id | subject | relation | object | evidence | confidence | subject_role | object_role | route | group |
|----|---------|----------|--------|----------|------------|--------------|-------------|-------|-------|
| c1 | V1 diagnosis | terminated_before | every executor PMU window | exact failure receipt | 1.0 | prior execution | frozen failure | provenance | loaded-pmu-v2 |
| c2 | V2 task identity | is_disjoint_from | V1 final state and markers | named V2 paths | 1.0 | retry owner | immutable history | sequencing | loaded-pmu-v2 |
| c3 | hybrid parser | aggregates | fully counted required PMU rows | parser correction contract | 1.0 | validation owner | counter evidence | parsing | loaded-pmu-v2 |
| c4 | hybrid parser | retains_without_summing | zero-runtime inactive PMU rows | parser correction contract | 1.0 | validation owner | raw evidence | parsing | loaded-pmu-v2 |
| c5 | B5 measurement | requires | counted cpu_core row | fixed B5 affinity | 1.0 | measurement owner | PMU coverage | measurement | loaded-pmu-v2 |
| c6 | B6 measurement | requires | counted cpu_core and cpu_atom rows | fixed B6 affinity | 1.0 | measurement owner | PMU coverage | measurement | loaded-pmu-v2 |
| c7 | parity owner | precedes | every V2 PMU subject execution | fixed sequence | 1.0 | proof owner | measurement owner | parity | loaded-pmu-v2 |
| c8 | environment owner | records_without_controlling | Nando btop K1 and host load | user decision | 1.0 | observation owner | foreign load | environment | loaded-pmu-v2 |
| c9 | V2 publication owner | preserves | raw counted inactive parity and environment evidence | publication contract | 1.0 | evidence owner | diagnostic artifact | publication | loaded-pmu-v2 |
| c10 | V2 result | does_not_admit | formal B clean C1 V12 or runtime deployment | claim boundary | 1.0 | scoped evidence | future authority | boundary | loaded-pmu-v2 |

## notes

- Structural PASS is coherence only; implementation still requires `READY_TO_IMPLEMENT`.
- V1 is immutable and has no retry right.
- V2 does not change events, subject code, affinity, host policy or background processes.

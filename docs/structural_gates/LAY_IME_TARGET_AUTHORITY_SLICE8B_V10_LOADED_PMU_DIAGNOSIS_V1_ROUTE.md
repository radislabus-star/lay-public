# NANDA Triad Worksheet

task_id: lay-v10-loaded-pmu-diagnosis-v1
domain: code
query: Check loaded executor PMU diagnosis ownership sequencing and claim boundaries

## triads

| id | subject | relation | object | evidence | confidence | subject_role | object_role | route | group |
|----|---------|----------|--------|----------|------------|--------------|-------------|-------|-------|
| s1 | loaded PMU contract | follows | closed dirty latency replication 2 of 2 | comparison receipt 01b9c10c | 1.0 | sequence owner | frozen evidence | sequencing | loaded-pmu-sequence-v1 |
| s2 | parity owner | precedes | every PMU subject execution | same sealed ELF semantic prerequisite | 1.0 | proof owner | measurement owner | parity | loaded-pmu-parity-v1 |
| s3 | capability owner | precedes | executor PMU windows | benign fixed perf probe | 1.0 | capability owner | measurement owner | capability | loaded-pmu-capability-v1 |
| s4 | measurement owner | observes | B5 and B6 executor-core windows under G0 through G3 | existing ready and done barriers | 1.0 | observation owner | counter evidence | measurement | loaded-pmu-measurement-v1 |
| s5 | environment owner | records_without_controlling | Nando btop K1 PSI and temperature | user loaded-host decision | 1.0 | observation owner | foreign load | environment | loaded-pmu-environment-v1 |
| s6 | publication owner | preserves | raw perf parity subject and environment evidence | atomic final rename and read-only tree | 1.0 | evidence owner | immutable artifact | publication | loaded-pmu-publication-v1 |
| s7 | diagnosis result | does_not_replace | clean C1 or formal B | explicit denominator boundary | 1.0 | scoped evidence | acceptance routes | boundary | loaded-pmu-boundary-v1 |
| s8 | diagnosis result | does_not_admit | V12 or runtime deployment | explicit user and contract boundary | 1.0 | scoped evidence | future implementation | admission | loaded-pmu-admission-v1 |

## candidate_triads

| id | subject | relation | object | evidence | confidence | subject_role | object_role | route | group |
|----|---------|----------|--------|----------|------------|--------------|-------------|-------|-------|
| c1 | loaded PMU contract | follows | closed dirty latency replication 2 of 2 | frozen comparison evidence | 1.0 | sequence owner | frozen evidence | sequencing | loaded-pmu-sequence-v1 |
| c2 | parity owner | precedes | every PMU subject execution | contract sequence | 1.0 | proof owner | measurement owner | parity | loaded-pmu-parity-v1 |
| c3 | capability owner | precedes | executor PMU windows | contract sequence | 1.0 | capability owner | measurement owner | capability | loaded-pmu-capability-v1 |
| c4 | measurement owner | observes | B5 and B6 executor-core windows under G0 through G3 | contract run matrix | 1.0 | observation owner | counter evidence | measurement | loaded-pmu-measurement-v1 |
| c5 | environment owner | records_without_controlling | Nando btop K1 PSI and temperature | contract loaded boundary | 1.0 | observation owner | foreign load | environment | loaded-pmu-environment-v1 |
| c6 | publication owner | preserves | raw perf parity subject and environment evidence | contract publication sequence | 1.0 | evidence owner | immutable artifact | publication | loaded-pmu-publication-v1 |
| c7 | diagnosis result | does_not_replace | clean C1 or formal B | contract claim boundary | 1.0 | scoped evidence | acceptance routes | boundary | loaded-pmu-boundary-v1 |
| c8 | diagnosis result | does_not_admit | V12 or runtime deployment | contract claim boundary | 1.0 | scoped evidence | future implementation | admission | loaded-pmu-admission-v1 |

## notes

- Structural PASS is coherence only; implementation requires a separate READY_TO_IMPLEMENT preflight.
- Background services remain running and are not assigned causal responsibility.
- Every route and event group is one-shot; no adaptive replacement or retry is admitted.

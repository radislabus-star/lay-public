# NANDA Triad Worksheet

task_id: lay-v10-structural-work-a2
domain: code
query: Check A2 build-failure correction scope and new one-shot route

## triads

| id | subject | relation | object | evidence | confidence | subject_role | object_role | route | group |
|----|---------|----------|--------|----------|------------|--------------|-------------|-------|-------|
| s1 | A1 build | terminated_before | executable and structural subject | immutable cargo log and failure receipt | 1.0 | failed predecessor | absent measurement | failure | structural-a2-failure |
| s2 | A2 source owner | replaces_only | recursive counter JSON expansion with explicit map insertion | A2 correction contract | 1.0 | correction owner | exact implementation defect | correction | structural-a2-correction |
| s3 | A2 build owner | creates_once | disjoint source-preserving diagnostic ELF | new A2 build marker | 1.0 | build owner | sealed executable | build | structural-a2-build |
| s4 | A2 observer owner | preserves | A1 traversal counters allocator and parity semantics | source comparison gate | 1.0 | observation owner | frozen observer contract | observer | structural-a2-observer |
| s5 | environment owner | leaves_running | Nando btop K1 and foreign work | user loaded-host decision | 1.0 | noninterference owner | foreign processes | environment | structural-a2-environment |
| s6 | A2 result | does_not_admit | latency formal B V12 runtime integration or deployment | unchanged claim boundary | 1.0 | scoped evidence | future authority | boundary | structural-a2-boundary |

## candidate_triads

| id | subject | relation | object | evidence | confidence | subject_role | object_role | route | group |
|----|---------|----------|--------|----------|------------|--------------|-------------|-------|-------|
| c1 | A1 build | terminated_before | executable and structural subject | immutable cargo log and failure receipt | 1.0 | failed predecessor | absent measurement | failure | structural-a2-failure |
| c2 | A2 source owner | replaces_only | recursive counter JSON expansion with explicit map insertion | A2 correction contract | 1.0 | correction owner | exact implementation defect | correction | structural-a2-correction |
| c3 | A2 build owner | creates_once | disjoint source-preserving diagnostic ELF | new A2 build marker | 1.0 | build owner | sealed executable | build | structural-a2-build |
| c4 | A2 observer owner | preserves | A1 traversal counters allocator and parity semantics | source comparison gate | 1.0 | observation owner | frozen observer contract | observer | structural-a2-observer |
| c5 | environment owner | leaves_running | Nando btop K1 and foreign work | user loaded-host decision | 1.0 | noninterference owner | foreign processes | environment | structural-a2-environment |
| c6 | A2 result | does_not_admit | latency formal B V12 runtime integration or deployment | unchanged claim boundary | 1.0 | scoped evidence | future authority | boundary | structural-a2-boundary |

## notes

- A1 is terminal and cannot be retried.
- A2 has a disjoint task identity and one new build/run route.
- Structural PASS remains coherence-only with authority_ready=false.

# NANDA Triad Worksheet

task_id: lay-v10-exact-fused-band-transition-m1-run-correction-v2
domain: code
query: Check the run-only asset-path correction and disjoint one-shot V2 route

## triads

| id | subject | relation | object | evidence | confidence | subject_role | object_role | route | group |
|----|---------|----------|--------|----------|------------|--------------|-------------|-------|-------|
| s1 | V1 parity | terminated_before | package load transition trace and PMU | sealed V1 failure receipt | 1.0 | terminal predecessor | absent measurement | failure | m1-v1-failure |
| s2 | V2 controller | corrects_only | two B0a asset paths and disjoint run namespaces | V2 sole correction section | 1.0 | correction owner | exact controller defect | correction | m1-v2-correction |
| s3 | V2 execution owner | reuses_exactly | sealed M1 ELF with no rebuild or source change | remote build audit | 1.0 | run owner | immutable executable | execution | m1-v2-execution |
| s4 | V2 state owner | creates | fresh parity G0 G1 U1 one-shot markers | V2 sequence | 1.0 | state owner | disjoint marker set | state | m1-v2-state |
| s5 | V1 preservation owner | retains | V1 state markers build and failure evidence | V2 terminal boundary | 1.0 | preservation owner | immutable history | preservation | m1-v1-preservation |
| s6 | foreign-process owner | preserves | Nando btop K1 affinity priority and execution | unchanged M1 environment contract | 1.0 | noninterference owner | foreign work | environment | m1-v2-environment |
| s7 | V2 scoped evidence | authorizes_only | original M1 decision with rebuild S1 full B V12 runtime and latency PASS excluded | V2 claim boundary | 1.0 | scoped evidence | bounded scientific result | boundary | m1-v2-boundary |

## candidate_triads

| id | subject | relation | object | evidence | confidence | subject_role | object_role | route | group |
|----|---------|----------|--------|----------|------------|--------------|-------------|-------|-------|
| c1 | V1 parity | terminated_before | package load transition trace and PMU | sealed V1 failure receipt | 1.0 | terminal predecessor | absent measurement | failure | m1-v1-failure |
| c2 | V2 controller | corrects_only | two B0a asset paths and disjoint run namespaces | V2 sole correction section | 1.0 | correction owner | exact controller defect | correction | m1-v2-correction |
| c3 | V2 execution owner | reuses_exactly | sealed M1 ELF with no rebuild or source change | remote build audit | 1.0 | run owner | immutable executable | execution | m1-v2-execution |
| c4 | V2 state owner | creates | fresh parity G0 G1 U1 one-shot markers | V2 sequence | 1.0 | state owner | disjoint marker set | state | m1-v2-state |
| c5 | V1 preservation owner | retains | V1 state markers build and failure evidence | V2 terminal boundary | 1.0 | preservation owner | immutable history | preservation | m1-v1-preservation |
| c6 | foreign-process owner | preserves | Nando btop K1 affinity priority and execution | unchanged M1 environment contract | 1.0 | noninterference owner | foreign work | environment | m1-v2-environment |
| c7 | V2 scoped evidence | authorizes_only | original M1 decision with rebuild S1 full B V12 runtime and latency PASS excluded | V2 claim boundary | 1.0 | scoped evidence | bounded scientific result | boundary | m1-v2-boundary |

## notes

- V1 is terminal evidence, not a retry source.
- V2 changes controller routing only and reuses one sealed ELF.
- Structural PASS remains coherence-only with authority_ready=false.

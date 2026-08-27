# NANDA Triad Worksheet

task_id: lay-v10-b-hardware-route-skeleton-v2
domain: code
query: Check global B route ownership sequencing audit and V12 admission

## triads

| id | subject | relation | object | evidence | confidence | subject_role | object_role | route | group |
|----|---------|----------|--------|----------|------------|--------------|-------------|-------|-------|
| s1 | B evidence contract | assigns | historical exact ELF to aggregate envelope only | sealed source and user claim boundary | 1.0 | route owner | historical observation | proof | global-skeleton |
| s2 | B evidence contract | assigns | diagnostic proxy to executor-core counters only | phase attribution is impossible in exact ELF | 1.0 | route owner | proxy observation | proof | global-skeleton |
| s3 | B evidence contract | sequences | B0a one build freezer B0b before B1 and B2 | test-only parser and user sequencing correction | 1.0 | route owner | execution order | execution | global-skeleton |
| s4 | B evidence contract | distinguishes | bare perf invocation from PMU measurement | verified session trace | 1.0 | route owner | audit boundary | audit | global-skeleton |
| s5 | B evidence contract | keeps_separate | pristine C latency and B hardware counters | P0 frozen denominator contract | 1.0 | route owner | denominator boundary | proof | global-skeleton |
| s6 | B evidence contract | does_not_admit | V12 implementation | current user and P0 boundary | 1.0 | route owner | future implementation | admission | global-skeleton |

## candidate_triads

| id | subject | relation | object | evidence | confidence | subject_role | object_role | route | group |
|----|---------|----------|--------|----------|------------|--------------|-------------|-------|-------|
| c1 | B evidence contract | assigns | historical exact ELF to aggregate envelope only | contract B3 and claim boundary | 1.0 | route owner | historical observation | proof | global-skeleton |
| c2 | B evidence contract | assigns | diagnostic proxy to executor-core counters only | contract B4 B5 and B6 | 1.0 | route owner | proxy observation | proof | global-skeleton |
| c3 | B evidence contract | sequences | B0a one build freezer B0b before B1 and B2 | contract B0 correction and admission sequence | 1.0 | route owner | execution order | execution | global-skeleton |
| c4 | B evidence contract | distinguishes | bare perf invocation from PMU measurement | contract B design-window audit note | 1.0 | route owner | audit boundary | audit | global-skeleton |
| c5 | B evidence contract | keeps_separate | pristine C latency and B hardware counters | contract B7 and claim boundary | 1.0 | route owner | denominator boundary | proof | global-skeleton |
| c6 | B evidence contract | does_not_admit | V12 implementation | contract final B status | 1.0 | route owner | future implementation | admission | global-skeleton |

## notes

- Global owner coherence is separate from the 16 local route gates.
- Structural PASS cannot authorize implementation or measurement.

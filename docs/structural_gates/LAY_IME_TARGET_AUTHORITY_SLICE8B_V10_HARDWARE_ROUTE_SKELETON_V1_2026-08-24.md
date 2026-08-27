# NANDA Triad Worksheet

task_id: lay-v10-b-hardware-route-skeleton-v1
domain: code
query: Check global owner separation for historical envelope proxy counters pristine latency and V12 admission

## triads

| id | subject | relation | object | evidence | confidence | subject_role | object_role | route | group |
|----|---------|----------|--------|----------|------------|--------------|-------------|-------|-------|
| s1 | B evidence contract | assigns | historical exact ELF to aggregate envelope only | sealed source and user claim boundary | 1.0 | route owner | historical observation | proof | global-skeleton |
| s2 | B evidence contract | assigns | diagnostic proxy to executor-core counters only | phase attribution is impossible in exact ELF | 1.0 | route owner | proxy observation | proof | global-skeleton |
| s3 | B evidence contract | keeps_separate | pristine C latency and B hardware counters | P0 frozen denominator contract | 1.0 | route owner | denominator boundary | proof | global-skeleton |
| s4 | B evidence contract | does_not_admit | V12 implementation | current user and P0 boundary | 1.0 | route owner | future implementation | admission | global-skeleton |

## candidate_triads

| id | subject | relation | object | evidence | confidence | subject_role | object_role | route | group |
|----|---------|----------|--------|----------|------------|--------------|-------------|-------|-------|
| c1 | B evidence contract | assigns | historical exact ELF to aggregate envelope only | contract lines 10..46 and 272..308 | 1.0 | route owner | historical observation | proof | global-skeleton |
| c2 | B evidence contract | assigns | diagnostic proxy to executor-core counters only | contract lines 309..427 | 1.0 | route owner | proxy observation | proof | global-skeleton |
| c3 | B evidence contract | keeps_separate | pristine C latency and B hardware counters | contract lines 456..526 | 1.0 | route owner | denominator boundary | proof | global-skeleton |
| c4 | B evidence contract | does_not_admit | V12 implementation | contract lines 527..544 | 1.0 | route owner | future implementation | admission | global-skeleton |

## notes

- This packet checks global ownership only. Route details are checked in
  separate local packets and are not superposed here.
- Structural PASS cannot authorize implementation or measurement.

# NANDA Triad Worksheet

task_id: w1-fused-minimum-m2-boundary-v1
domain: general
query: Does the M2 paper preserve production and scientific authority boundaries for every verdict?

## triads

| id | subject | relation | object | evidence | confidence | subject_role | object_role | route | group |
|----|---------|----------|--------|----------|------------|--------------|-------------|-------|-------|
| s1 | M2 diagnostic build | preserves | production prefix and runtime authority | test-only source and no install contract | 1.0 | experiment artifact | immutable production owner | boundary | m2-production-boundary |
| s2 | positive M2 verdict | admits | separate test-only source decision paper only | authority-boundary section | 1.0 | scoped result | bounded future paper | authority | m2-positive-authority |
| s3 | rejected M2 verdict | closes | minimum-lowering mechanism | explicit rejection branch | 1.0 | terminal negative result | retired mechanism | decision | m2-rejection |
| s4 | blocked route | retains | consumed marker and complete evidence without retry | one-shot state contract | 1.0 | failure owner | immutable failure state | failure | m2-failure |
| s5 | M2 result | does not establish | production Lay latency or production optimization | test-only V13 boundary | 1.0 | scientific claim | forbidden production claim | boundary | m2-claim-boundary |

## candidate_triads

| id | subject | relation | object | evidence | confidence | subject_role | object_role | route | group |
|----|---------|----------|--------|----------|------------|--------------|-------------|-------|-------|
| c1 | M2 diagnostic build | preserves | production prefix and runtime authority | paper forbids production mutation | 1.0 | experiment artifact | immutable production owner | boundary | m2-production-boundary |
| c2 | positive M2 verdict | admits | separate test-only source decision paper only | exact next-authority text | 1.0 | scoped result | bounded future paper | authority | m2-positive-authority |
| c3 | rejected M2 verdict | closes | minimum-lowering mechanism | exact terminal branch | 1.0 | terminal negative result | retired mechanism | decision | m2-rejection |
| c4 | blocked route | retains | consumed marker and complete evidence without retry | exact marker rules | 1.0 | failure owner | immutable failure state | failure | m2-failure |
| c5 | M2 result | does not establish | production Lay latency or production optimization | explicit claim boundary | 1.0 | scientific claim | forbidden production claim | boundary | m2-claim-boundary |

## notes

- Positive, rejected and blocked branches all leave runtime authority unchanged.
- DAFSA decode becomes a future paper only after an M2 rejection, never inside M2.

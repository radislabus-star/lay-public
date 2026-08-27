# NANDA Triad Worksheet

task_id: d5-t4-offline-forensic-evidence-scope-v1
domain: general
query: Is the D5 T4 offline forensic evidence and estimator scope structurally closed?

## triads

| id | subject | relation | object | evidence | confidence | subject_role | object_role | route | group |
|----|---------|----------|--------|----------|------------|--------------|-------------|-------|-------|
| s1 | history owner | preserves | D5 terminal BLOCKED_PROVENANCE and no retry | sealed D5 terminal receipt | 1.0 | history owner | immutable verdict | history | forensic-history |
| s2 | input owner | pins | exact sealed T4 samples raw records receipt observation perf data and D2 map | forensic paper input table | 1.0 | input owner | immutable evidence | inputs | forensic-inputs |
| s3 | lifecycle owner | reconstructs_from | one libtest parent and twenty direct terminal worker TIDs | FORK COMM EXIT algorithm | 1.0 | lifecycle owner | worker identity | lifecycle | forensic-lifecycle |
| s4 | scope owner | computes_separately | all worker sample CPU sets and exact traversal worker sample CPU sets | forensic scope algorithm | 1.0 | scope owner | estimator projections | scope | forensic-scope |
| s5 | map owner | joins_by | exact D2 mapping unique load bias and normalized sealed range | D2 map and MMAP2 contract | 1.0 | map owner | machine attribution | mapping | forensic-map |

## candidate_triads

| id | subject | relation | object | evidence | confidence | subject_role | object_role | route | group |
|----|---------|----------|--------|----------|------------|--------------|-------------|-------|-------|
| c1 | history owner | preserves | D5 terminal BLOCKED_PROVENANCE and no retry | reviewed terminal preservation | 1.0 | history owner | immutable verdict | history | forensic-history |
| c2 | input owner | pins | exact sealed T4 samples raw records receipt observation perf data and D2 map | reviewed input closure | 1.0 | input owner | immutable evidence | inputs | forensic-inputs |
| c3 | lifecycle owner | reconstructs_from | one libtest parent and twenty direct terminal worker TIDs | reviewed lifecycle algorithm | 1.0 | lifecycle owner | worker identity | lifecycle | forensic-lifecycle |
| c4 | scope owner | computes_separately | all worker sample CPU sets and exact traversal worker sample CPU sets | reviewed scope algorithm | 1.0 | scope owner | estimator projections | scope | forensic-scope |
| c5 | map owner | joins_by | exact D2 mapping unique load bias and normalized sealed range | reviewed mapping closure | 1.0 | map owner | machine attribution | mapping | forensic-map |

## notes

- This route grants structural coherence only.
- It cannot change D5 or admit execution.

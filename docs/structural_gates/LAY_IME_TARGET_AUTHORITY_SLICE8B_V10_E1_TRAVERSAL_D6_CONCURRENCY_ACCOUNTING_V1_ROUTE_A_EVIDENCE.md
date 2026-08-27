# NANDA Triad Worksheet

task_id: d6-concurrency-accounting-evidence-v1
domain: general
query: Is the sealed D6 component and PMU accounting evidence structurally closed?

## triads

| id | subject | relation | object | evidence | confidence | subject_role | object_role | route | group |
|----|---------|----------|--------|----------|------------|--------------|-------------|-------|-------|
| s1 | D6 accounting auditor | pins | single fixed and reversed component byte streams plus shared structure | D6 pinned evidence table | 1.0 | accounting owner | immutable component evidence | inputs | accounting-evidence |
| s2 | D6 accounting auditor | reconstructs | exact six-phase CPU totals over 502915120 edges | frozen 118-byte schema and structure denominator | 1.0 | accounting owner | phase accounting | component | accounting-evidence |
| s3 | D6 accounting auditor | pairs | same worker and query chunk across fixed and reversed CPU mappings | worker w maps to w and 19-w | 1.0 | accounting owner | paired core-class comparison | mapping | accounting-evidence |
| s4 | D6 accounting auditor | consumes | exact B5 and B6 instructions cycles task-clock and IPC | sealed combined V3 V4 decision | 1.0 | accounting owner | aggregate counters | pmu | accounting-evidence |
| s5 | D6 accounting auditor | applies | time equals instructions divided by IPC and effective frequency | exact D6 formula and values | 1.0 | accounting owner | predicted traversal rate | synthesis | accounting-evidence |

## candidate_triads

| id | subject | relation | object | evidence | confidence | subject_role | object_role | route | group |
|----|---------|----------|--------|----------|------------|--------------|-------------|-------|-------|
| c1 | D6 accounting auditor | pins | single fixed and reversed component byte streams plus shared structure | reviewed D6 input closure | 1.0 | accounting owner | immutable component evidence | inputs | accounting-evidence |
| c2 | D6 accounting auditor | reconstructs | exact six-phase CPU totals over 502915120 edges | reviewed parser schema and denominator | 1.0 | accounting owner | phase accounting | component | accounting-evidence |
| c3 | D6 accounting auditor | pairs | same worker and query chunk across fixed and reversed CPU mappings | reviewed fixed reversed worker identity | 1.0 | accounting owner | paired core-class comparison | mapping | accounting-evidence |
| c4 | D6 accounting auditor | consumes | exact B5 and B6 instructions cycles task-clock and IPC | reviewed sealed PMU decision | 1.0 | accounting owner | aggregate counters | pmu | accounting-evidence |
| c5 | D6 accounting auditor | applies | time equals instructions divided by IPC and effective frequency | independently recomputed D6 equation | 1.0 | accounting owner | predicted traversal rate | synthesis | accounting-evidence |

## notes

- This route verifies evidence and arithmetic structure only.
- It grants no causal microarchitecture claim or implementation authority.

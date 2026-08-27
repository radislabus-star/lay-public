# NANDA Triad Worksheet

task_id: w1-dafsa-typed-view-m3-v1-route-b
domain: general
query: Are parity, one-shot execution, PMU observation and terminal decision ownership separated?

## triads

| id | subject | relation | object | evidence | confidence | subject_role | object_role | route | group |
|----|---------|----------|--------|----------|------------|--------------|-------------|-------|-------|
| s1 | M3 parity | precedes | every physical route | PARITY marker and state gate precede B0 | 1.0 | semantic verifier | execution admission | parity | ordering |
| s2 | M3 physical controller | executes | B0 T0 T1 B1 once | closed route registry equals marker registry | 1.0 | execution owner | physical evidence | execution | one-shot |
| s3 | perf stat | observes | every physical route | exact inherited FIFO command owns five frozen events | 1.0 | observation owner | PMU evidence | observation | measurement |
| s4 | terminal auditor | alone decides | M3 pass reject or blocked | pair reconciliation and dispatch priority are frozen | 1.0 | proof owner | terminal verdict | decision | terminal |

## candidate_triads

| id | subject | relation | object | evidence | confidence | subject_role | object_role | route | group |
|----|---------|----------|--------|----------|------------|--------------|-------------|-------|-------|
| c1 | M3 parity | precedes | every physical route | no route is reachable before exact parity | 1.0 | semantic verifier | execution admission | parity | ordering |
| c2 | M3 physical controller | executes | B0 T0 T1 B1 once | six one-shot markers include build parity and four physical routes | 1.0 | execution owner | physical evidence | execution | one-shot |
| c3 | perf stat | observes | every physical route | perf has observation authority and no verdict authority | 1.0 | observation owner | PMU evidence | observation | measurement |
| c4 | terminal auditor | alone decides | M3 pass reject or blocked | intermediate route values cannot select a branch | 1.0 | proof owner | terminal verdict | decision | terminal |

## notes

- Lost responses remain unknown and forbid retry.
- There is no perf record, attach route or SIGINT lifecycle.

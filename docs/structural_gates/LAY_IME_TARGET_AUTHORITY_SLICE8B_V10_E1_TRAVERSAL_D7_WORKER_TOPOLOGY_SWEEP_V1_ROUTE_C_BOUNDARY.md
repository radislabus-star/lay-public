# NANDA Triad Worksheet

task_id: d7-worker-topology-boundary-v1
domain: general
query: Are the D7 interpretation and production boundaries structurally closed?

## triads

| id | subject | relation | object | evidence | confidence | subject_role | object_role | route | group |
|----|---------|----------|--------|----------|------------|--------------|-------------|-------|-------|
| s1 | D7 decision owner | publishes_separately | latency capacity throughput point and Pareto frontier | preregistered decision rule | 1.0 | decision owner | diagnostic result | interpretation | d7-boundary |
| s2 | D7 decision owner | does_not_promote | diagnostic worker count into production concurrency policy | D6 production boundary | 1.0 | decision owner | runtime policy | boundary | d7-boundary |
| s3 | D7 decision owner | withholds | queueing latency and service-level concurrency conclusions | no production producer is exercised | 1.0 | decision owner | unmeasured consequence | boundary | d7-boundary |
| s4 | D7 decision owner | forbids | SWAR production edit install restart deployment and D5 retry | D7 authority boundary | 1.0 | decision owner | forbidden action set | authority | d7-boundary |
| s5 | D7 decision owner | requires_next_paper_to_identify | actual concurrency producer and full latency denominator | D7 next-action contract | 1.0 | decision owner | implementation admission | next | d7-boundary |

## candidate_triads

| id | subject | relation | object | evidence | confidence | subject_role | object_role | route | group |
|----|---------|----------|--------|----------|------------|--------------|-------------|-------|-------|
| c1 | D7 decision owner | publishes_separately | latency capacity throughput point and Pareto frontier | reviewed decision rule | 1.0 | decision owner | diagnostic result | interpretation | d7-boundary |
| c2 | D7 decision owner | does_not_promote | diagnostic worker count into production concurrency policy | reviewed production scope | 1.0 | decision owner | runtime policy | boundary | d7-boundary |
| c3 | D7 decision owner | withholds | queueing latency and service-level concurrency conclusions | reviewed missing producer path | 1.0 | decision owner | unmeasured consequence | boundary | d7-boundary |
| c4 | D7 decision owner | forbids | SWAR production edit install restart deployment and D5 retry | reviewed authority boundary | 1.0 | decision owner | forbidden action set | authority | d7-boundary |
| c5 | D7 decision owner | requires_next_paper_to_identify | actual concurrency producer and full latency denominator | reviewed next-action contract | 1.0 | decision owner | implementation admission | next | d7-boundary |

## notes

- D7 can choose a diagnostic frontier, not a deployed setting.
- The target host and frozen workload bound every result.

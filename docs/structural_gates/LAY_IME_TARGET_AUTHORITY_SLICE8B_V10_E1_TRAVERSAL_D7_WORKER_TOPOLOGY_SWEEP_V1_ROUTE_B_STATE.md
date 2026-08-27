# NANDA Triad Worksheet

task_id: d7-worker-topology-state-v1
domain: general
query: Is the D7 build and one-shot failure topology structurally closed?

## triads

| id | subject | relation | object | evidence | confidence | subject_role | object_role | route | group |
|----|---------|----------|--------|----------|------------|--------------|-------------|-------|-------|
| s1 | D7 state controller | creates_after_admission | seven exact available markers and one route lock | paper and implementation preflight PASS | 1.0 | state owner | one-shot state | bootstrap | d7-state |
| s2 | D7 state controller | consumes_before | build marker before one Cargo invocation | build-marker atomic rename and directory fsync | 1.0 | state owner | build authority | build | d7-state |
| s3 | D7 state controller | consumes_before | parity or worker marker before subject and perf | route-marker atomic rename and directory fsync | 1.0 | state owner | route authority | execute | d7-state |
| s4 | D7 state controller | preserves | consumed marker complete logs and partial owned evidence | append-only failure receipt | 1.0 | state owner | immutable evidence | failure | d7-state |
| s5 | D7 state controller | forbids | rerun marker recreation and later route execution after failure | terminal state machine | 1.0 | state owner | retry authority | terminal | d7-state |

## candidate_triads

| id | subject | relation | object | evidence | confidence | subject_role | object_role | route | group |
|----|---------|----------|--------|----------|------------|--------------|-------------|-------|-------|
| c1 | D7 state controller | creates_after_admission | seven exact available markers and one route lock | reviewed bootstrap gate | 1.0 | state owner | one-shot state | bootstrap | d7-state |
| c2 | D7 state controller | consumes_before | build marker before one Cargo invocation | reviewed build transaction | 1.0 | state owner | build authority | build | d7-state |
| c3 | D7 state controller | consumes_before | parity or worker marker before subject and perf | reviewed route transaction | 1.0 | state owner | route authority | execute | d7-state |
| c4 | D7 state controller | preserves | consumed marker complete logs and partial owned evidence | reviewed failure publication | 1.0 | state owner | immutable evidence | failure | d7-state |
| c5 | D7 state controller | forbids | rerun marker recreation and later route execution after failure | reviewed no-retry boundary | 1.0 | state owner | retry authority | terminal | d7-state |

## notes

- A transport failure after marker consumption does not restore authority.
- Historical D1 through D6 trees are immutable inputs and never cleanup targets.

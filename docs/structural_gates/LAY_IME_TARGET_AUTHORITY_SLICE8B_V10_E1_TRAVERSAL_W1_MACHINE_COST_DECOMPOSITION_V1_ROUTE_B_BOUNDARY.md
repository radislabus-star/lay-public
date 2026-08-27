# NANDA Triad Worksheet

task_id: w1-machine-cost-decomposition-boundary-v1
domain: general
query: Are W1 decomposition claims and next-action authority correctly bounded?

## triads

| id | subject | relation | object | evidence | confidence | subject_role | object_role | route | group |
|----|---------|----------|--------|----------|------------|--------------|-------------|-------|-------|
| s1 | W1 decision owner | distinguishes | valid D4 attribution from invalid historical D2 T-SINGLE numbers | D4 independent terminal verdict | 1.0 | decision owner | evidence scope | boundary | w1-boundary |
| s2 | W1 decision owner | forbids | direct D2-map join to distinct D7 ELF | distinct ELF SHA and Build ID | 1.0 | decision owner | provenance violation | boundary | w1-boundary |
| s3 | W1 decision owner | withholds | exact per-bucket D7 cycles and single-instruction latency claims | non-precise task-clock event and distinct ELF | 1.0 | decision owner | unsupported causal claim | boundary | w1-boundary |
| s4 | W1 decision owner | admits_only | separate fused-minimum mechanism paper | dominant valid D4 range plus exact redundant source pass | 1.0 | decision owner | future paper route | next | w1-boundary |
| s5 | W1 decision owner | forbids | code Cargo perf subject remote marker and runtime actions | offline-only W1 contract | 1.0 | decision owner | side effects | authority | w1-boundary |

## candidate_triads

| id | subject | relation | object | evidence | confidence | subject_role | object_role | route | group |
|----|---------|----------|--------|----------|------------|--------------|-------------|-------|-------|
| c1 | W1 decision owner | distinguishes | valid D4 attribution from invalid historical D2 T-SINGLE numbers | reviewed terminal verdict scopes | 1.0 | decision owner | evidence scope | boundary | w1-boundary |
| c2 | W1 decision owner | forbids | direct D2-map join to distinct D7 ELF | reviewed identity mismatch | 1.0 | decision owner | provenance violation | boundary | w1-boundary |
| c3 | W1 decision owner | withholds | exact per-bucket D7 cycles and single-instruction latency claims | reviewed sampling and transfer limits | 1.0 | decision owner | unsupported causal claim | boundary | w1-boundary |
| c4 | W1 decision owner | admits_only | separate fused-minimum mechanism paper | reviewed dominant range and source redundancy | 1.0 | decision owner | future paper route | next | w1-boundary |
| c5 | W1 decision owner | forbids | code Cargo perf subject remote marker and runtime actions | reviewed offline-only contract | 1.0 | decision owner | side effects | authority | w1-boundary |

## notes

- A mechanism paper is not an implementation admission.
- The seven-cell recurrence is required; only the second minimum pass is the selected hypothesis.

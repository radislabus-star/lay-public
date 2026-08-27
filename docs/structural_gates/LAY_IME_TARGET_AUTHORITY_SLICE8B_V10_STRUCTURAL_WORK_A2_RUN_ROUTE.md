# NANDA Triad Worksheet

task_id: lay-v10-structural-work-a2-run
domain: code
query: Check sealed read-only ELF loader execution correction

## triads

| id | subject | relation | object | evidence | confidence | subject_role | object_role | route | group |
|----|---------|----------|--------|----------|------------|--------------|-------------|-------|-------|
| s1 | A2 build | produced | sealed ELF mode 0444 with exact SHA and Build ID | build admission receipt | 1.0 | build evidence | immutable executable | build | structural-a2r-build |
| s2 | admission probe | invoked_without_executing | exact ignored structural test | harness list audit and unconsumed marker | 1.0 | audit owner | unexecuted subject | audit | structural-a2r-audit |
| s3 | corrected run owner | executes | exact sealed ELF through pinned system loader | run correction contract | 1.0 | execution owner | immutable executable | execution | structural-a2r-execution |
| s4 | corrected run owner | consumes_before | sole remaining run marker | one-shot state contract | 1.0 | execution owner | run authority | marker | structural-a2r-marker |
| s5 | corrected run | preserves | structural observer assets parity and counters | unchanged ELF bytes | 1.0 | observation owner | frozen scientific route | observer | structural-a2r-observer |
| s6 | corrected result | does_not_admit | latency formal B V12 runtime integration or deployment | unchanged boundary | 1.0 | scoped evidence | future authority | boundary | structural-a2r-boundary |

## candidate_triads

| id | subject | relation | object | evidence | confidence | subject_role | object_role | route | group |
|----|---------|----------|--------|----------|------------|--------------|-------------|-------|-------|
| c1 | A2 build | produced | sealed ELF mode 0444 with exact SHA and Build ID | build admission receipt | 1.0 | build evidence | immutable executable | build | structural-a2r-build |
| c2 | admission probe | invoked_without_executing | exact ignored structural test | harness list audit and unconsumed marker | 1.0 | audit owner | unexecuted subject | audit | structural-a2r-audit |
| c3 | corrected run owner | executes | exact sealed ELF through pinned system loader | run correction contract | 1.0 | execution owner | immutable executable | execution | structural-a2r-execution |
| c4 | corrected run owner | consumes_before | sole remaining run marker | one-shot state contract | 1.0 | execution owner | run authority | marker | structural-a2r-marker |
| c5 | corrected run | preserves | structural observer assets parity and counters | unchanged ELF bytes | 1.0 | observation owner | frozen scientific route | observer | structural-a2r-observer |
| c6 | corrected result | does_not_admit | latency formal B V12 runtime integration or deployment | unchanged boundary | 1.0 | scoped evidence | future authority | boundary | structural-a2r-boundary |

## notes

- No build or ELF mutation is admitted.
- The run marker is consumed only after loader and ELF identity checks.
- Structural PASS remains coherence-only with authority_ready=false.

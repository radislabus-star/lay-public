# NANDA Triad Worksheet

task_id: m3-final-materialization-diagnostic-v9-route-v1
domain: general
query: Is V9 a closed one-shot diagnostic that can decompose final materialization without changing V8R3 history or production authority?

## triads

| id | subject | relation | object | evidence | confidence | subject_role | object_role | route | group |
|----|---------|----------|--------|----------|------------|--------------|-------------|-------|-------|
| s1 | V8R3 history | remains | immutable terminal predecessor | diagnostic-v9.md:53-54 | 1.0 | historical experiment | retained evidence | history | history |
| s2 | V9 executable route | executes | exact sealed V8R3 ELF without rebuild or copy | diagnostic-v9.md:56-72 | 1.0 | diagnostic executor | immutable executable | identity | identity |
| s3 | V9 input closure | uses | exact V8R1 inputs and target host identity | diagnostic-v9.md:74-99 | 1.0 | diagnostic executor | immutable input tuple | identity | identity |
| s4 | V9 command graph | contains_only | direct traced test route and frozen environment | diagnostic-v9.md:101-126 | 1.0 | closed command graph | sole subject invocation | execution | execution |
| s5 | V9 trace parser | requires | exactly 1910 ordered rows and lossless numeric fields | diagnostic-v9.md:128-155 | 1.0 | evidence consumer | complete trace stream | evidence | evidence |
| s6 | V9 decomposition | evaluates | 1528 measured rows and a 16-row traced tail | diagnostic-v9.md:157-172 | 1.0 | diagnostic estimator | bounded stage distributions | evidence | evidence |
| s7 | V9 subject acceptance | allows | only internally consistent PASS exit0 or BLOCKED_LATENCY exit101 | diagnostic-v9.md:174-188 | 1.0 | receipt validator | accepted subject outcome pair | semantics | semantics |
| s8 | V9 traced timing | cannot_replace | immutable V8R3 latency verdict | diagnostic-v9.md:190-192 | 1.0 | perturbed diagnostic timing | scientific latency authority | semantics | semantics |
| s9 | V9 state machine | consumes_before | one trace marker before one subject execution | diagnostic-v9.md:194-208 | 1.0 | one-shot controller | execution authority | state | state |
| s10 | V9 bootstrap admission | proves_before_marker | real UID path access and durable journal semantics | diagnostic-v9.md:211-221 | 1.0 | pre-marker admission | subject and evidence capability | state | state |
| s11 | V9 terminal auditor | dispatches | provenance semantic capability or complete decomposition | diagnostic-v9.md:223-247 | 1.0 | terminal classifier | exactly one verdict | terminal | terminal |
| s12 | V9 positive verdict | cannot_grant | source edit build deployment or production authority | diagnostic-v9.md:249-272 | 1.0 | bounded diagnostic evidence | forbidden authority | boundary | boundary |

## candidate_triads

| id | subject | relation | object | evidence | confidence | subject_role | object_role | route | group |
|----|---------|----------|--------|----------|------------|--------------|-------------|-------|-------|
| c1 | V8R3 history | remains | immutable terminal predecessor | exact V8R3 terminal receipt and consumed marker | 1.0 | historical experiment | retained evidence | history | history |
| c2 | V9 executable route | executes | exact sealed V8R3 ELF without rebuild or copy | ELF SHA size mode and Build ID checks | 1.0 | diagnostic executor | immutable executable | identity | identity |
| c3 | V9 input closure | uses | exact V8R1 inputs and target host identity | remote SHA size mode and host checks | 1.0 | diagnostic executor | immutable input tuple | identity | identity |
| c4 | V9 command graph | contains_only | direct traced test route and frozen environment | static reachable argv registry | 1.0 | closed command graph | sole subject invocation | execution | execution |
| c5 | V9 trace parser | requires | exactly 1910 ordered rows and lossless numeric fields | anchored regex and cardinality check | 1.0 | evidence consumer | complete trace stream | evidence | evidence |
| c6 | V9 decomposition | evaluates | 1528 measured rows and a 16-row traced tail | deterministic ordinal estimator | 1.0 | diagnostic estimator | bounded stage distributions | evidence | evidence |
| c7 | V9 subject acceptance | allows | only internally consistent PASS exit0 or BLOCKED_LATENCY exit101 | receipt and exit pair validator | 1.0 | receipt validator | accepted subject outcome pair | semantics | semantics |
| c8 | V9 traced timing | cannot_replace | immutable V8R3 latency verdict | explicit trace perturbation boundary | 1.0 | perturbed diagnostic timing | scientific latency authority | semantics | semantics |
| c9 | V9 state machine | consumes_before | one trace marker before one subject execution | atomic rename and state predecessor | 1.0 | one-shot controller | execution authority | state | state |
| c10 | V9 bootstrap admission | proves_before_marker | real UID path access and durable journal semantics | UID probe and response-retention tests | 1.0 | pre-marker admission | subject and evidence capability | state | state |
| c11 | V9 terminal auditor | dispatches | provenance semantic capability or complete decomposition | priority table and complete predicates | 1.0 | terminal classifier | exactly one verdict | terminal | terminal |
| c12 | V9 positive verdict | cannot_grant | source edit build deployment or production authority | closed forbidden action graph | 1.0 | bounded diagnostic evidence | forbidden authority | boundary | boundary |

## notes

- Trace latency is diagnostic and cannot alter V8R3 latency authority.
- Structural PASS is coherence only. Controller implementation requires a separate READY preflight.

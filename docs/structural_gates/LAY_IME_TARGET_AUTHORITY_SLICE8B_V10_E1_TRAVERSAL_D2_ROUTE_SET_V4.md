# NANDA Triad Worksheet

task_id: lay-v10-e1-traversal-d2-route-set-v4
domain: code
query: Source packet for independent local D2 owner-route gates

## triads

| id | subject | relation | object | evidence | confidence | subject_role | object_role | route | group |
|----|---------|----------|--------|----------|------------|--------------|-------------|-------|-------|
| s1 | immutable closure owner | verifies_before | exact inputs and sealed D1 denominators | D2-A immutable closure obligation | 1.0 | closure owner | evidence | closure | d2-closure |
| s2 | one build owner | builds_once | symbolized release ELF from exact D1 Rust bytes | D2-B one-build obligation | 1.0 | build owner | artifact | build | d2-build |
| s3 | machine closure owner | seals_before | complete instruction ranges and machine-byte hashes | D2-C map obligation | 1.0 | map owner | evidence | map | d2-map |
| s4 | control owner | rejects | semantic structural or denominator divergence | D2-D control obligation | 1.0 | control owner | blocked route | control | d2-control |
| s5 | sampling owner | executes_separately | fixed-period task-clock and retired-instruction processes | D2-E sampling obligation | 1.0 | sampling owner | evidence | sampling | d2-sampling |
| s6 | validity owner | vetoes | perturbed lost insufficient or map-mismatched samples | D2-E validity obligation | 1.0 | validity owner | blocked evidence | validity | d2-validity |
| s7 | attribution owner | joins_only | sample IPs and presealed ranges to per-edge bucket inflation | D2-F attribution obligation | 1.0 | attribution owner | result | attribution | d2-attribution |
| s8 | decision owner | emits_only | bounded next-paper route or blocked verdict | D2 decision obligation | 1.0 | decision owner | result | decision | d2-decision |

## candidate_triads

| id | subject | relation | object | evidence | confidence | subject_role | object_role | route | group |
|----|---------|----------|--------|----------|------------|--------------|-------------|-------|-------|
| c1 | immutable closure owner | verifies_before | exact inputs and sealed D1 denominators | reviewed D2-A closure route | 1.0 | closure owner | evidence | closure | d2-closure |
| c2 | one build owner | builds_once | symbolized release ELF from exact D1 Rust bytes | reviewed D2-B build route | 1.0 | build owner | artifact | build | d2-build |
| c3 | machine closure owner | seals_before | complete instruction ranges and machine-byte hashes | reviewed D2-C map route | 1.0 | map owner | evidence | map | d2-map |
| c4 | control owner | rejects | semantic structural or denominator divergence | reviewed D2-D control route | 1.0 | control owner | blocked route | control | d2-control |
| c5 | sampling owner | executes_separately | fixed-period task-clock and retired-instruction processes | reviewed D2-E sampling route | 1.0 | sampling owner | evidence | sampling | d2-sampling |
| c6 | validity owner | vetoes | perturbed lost insufficient or map-mismatched samples | reviewed D2-E validity route | 1.0 | validity owner | blocked evidence | validity | d2-validity |
| c7 | attribution owner | joins_only | sample IPs and presealed ranges to per-edge bucket inflation | reviewed D2-F attribution route | 1.0 | attribution owner | result | attribution | d2-attribution |
| c8 | decision owner | emits_only | bounded next-paper route or blocked verdict | reviewed D2 decision route | 1.0 | decision owner | result | decision | d2-decision |

## notes

- This packet is split by `group`; no monolithic verdict is promoted.
- Every local receipt is coherence-only with `authority_ready=false`.
- Exact subject-relation-object correspondence is checked against distinct contract and review evidence spans.
- Local PASS cannot admit implementation, measurement, V12, runtime integration, or deployment.

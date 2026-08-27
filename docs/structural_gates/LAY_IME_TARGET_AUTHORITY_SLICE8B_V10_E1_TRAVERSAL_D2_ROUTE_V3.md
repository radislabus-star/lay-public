# NANDA Triad Worksheet

task_id: lay-v10-e1-traversal-d2-route-v3
domain: code
query: Check repaired owner-aligned D2 sampling attribution and fail-closed sequencing

## triads

| id | subject | relation | object | evidence | confidence | subject_role | object_role | route | group |
|----|---------|----------|--------|----------|------------|--------------|-------------|-------|-------|
| s1 | immutable closure owner | must_verify_before | exact inputs and sealed D1 denominators | D2-A immutable closure obligation | 1.0 | closure owner | evidence | closure | d2-closure |
| s2 | one build owner | may_build_once | symbolized release ELF from exact D1 Rust bytes | D2-B one-build obligation | 1.0 | build owner | artifact | build | d2-build |
| s3 | machine closure owner | must_seal_before | complete instruction ranges and machine-byte hashes | D2-C pre-measurement map obligation | 1.0 | map owner | evidence | map | d2-map |
| s4 | control owner | must_reject | semantic structural or denominator divergence | D2-D parity and unsampled-control obligation | 1.0 | control owner | blocked route | control | d2-control |
| s5 | sampling owner | must_execute_separately | fixed-period task-clock and retired-instruction processes | D2-E independent-sampling obligation | 1.0 | sampling owner | evidence | sampling | d2-sampling |
| s6 | validity owner | must_veto | perturbed lost insufficient or map-mismatched samples | D2-E sampling-validity obligation | 1.0 | validity owner | blocked evidence | validity | d2-validity |
| s7 | attribution owner | must_join_only | sample IPs and presealed ranges to per-edge bucket inflation | D2-F attribution obligation | 1.0 | attribution owner | result | attribution | d2-attribution |
| s8 | decision owner | must_emit_only | bounded next-paper route or blocked verdict | D2 decision-boundary obligation | 1.0 | decision owner | result | decision | d2-decision |

## candidate_triads

| id | subject | relation | object | evidence | confidence | subject_role | object_role | route | group |
|----|---------|----------|--------|----------|------------|--------------|-------------|-------|-------|
| c1 | immutable closure owner | verifies_before | exact inputs and sealed D1 denominators | D2-A closure list and forbidden effects | 1.0 | closure owner | evidence | closure | d2-closure |
| c2 | one build owner | builds_once | symbolized release ELF from exact D1 Rust bytes | D2-B source identity and consumed marker | 1.0 | build owner | artifact | build | d2-build |
| c3 | machine closure owner | seals_before | complete instruction ranges and machine-byte hashes | D2-C D2_BUCKET_MAP fields and timing | 1.0 | map owner | evidence | map | d2-map |
| c4 | control owner | rejects | semantic structural or denominator divergence | D2-D exact validity conjuncts | 1.0 | control owner | blocked route | control | d2-control |
| c5 | sampling owner | executes_separately | fixed-period task-clock and retired-instruction processes | D2-E pinned events periods and order | 1.0 | sampling owner | evidence | sampling | d2-sampling |
| c6 | validity owner | vetoes | perturbed lost insufficient or map-mismatched samples | D2-E blocked verdict conditions | 1.0 | validity owner | blocked evidence | validity | d2-validity |
| c7 | attribution owner | joins_only | sample IPs and presealed ranges to per-edge bucket inflation | D2-F formulas and publication fields | 1.0 | attribution owner | result | attribution | d2-attribution |
| c8 | decision owner | emits_only | bounded next-paper route or blocked verdict | D2 decision states and no-implementation rule | 1.0 | decision owner | result | decision | d2-decision |

## notes

- V1 is retained as `VETO`: ten routes exceeded the structural target and `d2-map` mixed owners.
- V2 is retained as `VETO`: an abstract paper subject conflicted with each concrete execution owner.
- V3 has eight coherent routes, matched source/candidate ownership, and one owner per group.
- The exact V10 source has one source-level state decode per expanded state; no duplicate decode is presumed.
- This is structural paper review only. `authority_ready` must remain false.
- D2 implementation preflight, controller, build, bucket map, subject execution, and sampling do not exist.
- D2 can select only a future paper route. Full B, V12, runtime integration, and deployment remain unadmitted.

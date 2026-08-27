# NANDA Triad Worksheet

task_id: lay-v10-e1-traversal-d2-route-v2
domain: code
query: Check the repaired D2 sampling-attribution ownership and fail-closed sequence

## triads

| id | subject | relation | object | evidence | confidence | subject_role | object_role | route | group |
|----|---------|----------|--------|----------|------------|--------------|-------------|-------|-------|
| s1 | D2 contract | requires_before_build | immutable exact-V10 and sealed-D1 closure | D2-A Immutable Input Closure | 1.0 | paper authority | frozen input boundary | closure | d2-closure |
| s2 | D2 contract | permits_once | symbolized release build with exact D1 Rust bytes | D2-B One Symbolized Build | 1.0 | paper authority | bounded build effect | build | d2-build |
| s3 | D2 contract | requires_before_execution | sealed instruction-range bucket map | D2-C Pre-Measurement Machine Closure | 1.0 | paper authority | frozen classification evidence | map | d2-map |
| s4 | D2 contract | requires_before_sampling | semantic parity and unsampled denominator controls | D2-D Semantic and Unsampled Controls | 1.0 | paper authority | control evidence | control | d2-control |
| s5 | D2 contract | separates | fixed-period task-clock and retired-instruction sampling | D2-E External IP Sampling | 1.0 | paper authority | independent observation channels | sampling | d2-sampling |
| s6 | D2 contract | withholds_attribution_on | perturbation loss coverage or integrity failure | D2-E Required Sampling Validity | 1.0 | paper authority | fail-closed validity boundary | validity | d2-validity |
| s7 | D2 contract | derives_only | bucket CPU and instruction shares from presealed ranges | D2-F Attribution | 1.0 | paper authority | bounded attribution result | attribution | d2-attribution |
| s8 | D2 contract | authorizes_only | future paper route without implementation or runtime authority | D2 Decision Boundary | 1.0 | paper authority | bounded next-paper decision | decision | d2-decision |

## candidate_triads

| id | subject | relation | object | evidence | confidence | subject_role | object_role | route | group |
|----|---------|----------|--------|----------|------------|--------------|-------------|-------|-------|
| c1 | immutable-closure owner | verifies_before | source fragment schedule D1 decisions and work ledger | D2-A owner prohibition and closure list | 1.0 | closure owner | verified immutable inputs | closure | d2-closure |
| c2 | one-build owner | consumes_after | successful immutable closure and a preconsumed build marker | D2-B source identity and one-shot rules | 1.0 | build owner | single symbolized executable | build | d2-build |
| c3 | machine-closure owner | seals_before | complete address ranges machine-byte hashes and ambiguous status | D2-C map fields and frozen buckets | 1.0 | classification owner | immutable D2_BUCKET_MAP | map | d2-map |
| c4 | control owner | rejects | semantic structural or denominator divergence before sampling | D2-D parity and unsampled validity gates | 1.0 | control gate owner | divergent D2 executable or route | control | d2-control |
| c5 | sampling owner | executes_separately | pinned task-clock and precise instruction processes without substitution | D2-E fixed events periods and route order | 1.0 | observation owner | primary and secondary IP samples | sampling | d2-sampling |
| c6 | validity owner | vetoes | attribution with perturbation lost samples insufficient coverage or map mismatch | D2-E required sampling validity conjuncts | 1.0 | validity gate owner | invalid sampled evidence | validity | d2-validity |
| c7 | attribution owner | joins_only | sample IPs and presealed map to per-edge bucket inflation | D2-F frozen formulas and publication list | 1.0 | aggregation owner | machine-code cost attribution | attribution | d2-attribution |
| c8 | decision owner | emits_only | a bounded candidate paper route or a blocked verdict | D2 decision states and no-implementation rule | 1.0 | decision owner | paper-only next state | decision | d2-decision |

## notes

- V1 is retained as `VETO`: it exceeded the route limit and mixed two owners in `d2-map`.
- This V2 has eight coherent routes and one owner per candidate group.
- The exact V10 source has one source-level state decode per expanded state; no duplicate decode is presumed.
- This is structural paper review only. `authority_ready` must remain false.
- D2 implementation preflight, controller, build, bucket map, subject execution, and sampling do not exist.
- D2 can select only a future paper route. Full B, V12, runtime integration, and deployment remain unadmitted.

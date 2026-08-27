# NANDA Triad Worksheet

task_id: lay-v10-e1-traversal-d2-route-v1
domain: code
query: Check D2 sampling attribution ownership and fail-closed sequencing

## triads

| id | subject | relation | object | evidence | confidence | subject_role | object_role | route | group |
|----|---------|----------|--------|----------|------------|--------------|-------------|-------|-------|
| s1 | D1 sealed evidence | controls | D2 static work and CPU denominators | D2 Corrected Static Closure | 1.0 | evidence authority | frozen denominator | static | d2-static |
| s2 | recovered V10 source | proves_absence_of | source-level redundant second state decode | D2 Corrected Static Closure | 1.0 | source authority | rejected hypothesis | static | d2-static |
| s3 | symbolized build owner | changes_only | debug and symbol retention settings | D2-B One Symbolized Build | 1.0 | build owner | immutable source build | build | d2-build |
| s4 | machine closure owner | produces_before | immutable instruction bucket map | D2-C Pre-Measurement Machine Closure | 1.0 | disassembly owner | bucket evidence | map | d2-map |
| s5 | bucket map owner | assigns_before | every traversal instruction range or unattributed status | D2-C frozen buckets | 1.0 | classification owner | instruction ranges | map | d2-map |
| s6 | parity owner | rejects | any semantic structural rank or certificate divergence | D2-D Semantic Control | 1.0 | proof gate | divergent executable | parity | d2-parity |
| s7 | unsampled control owner | compares | D2 traversal CPU and instruction perturbation against sealed D1 | D2-D validity gates | 1.0 | control owner | perturbation baseline | control | d2-control |
| s8 | task-clock sampler | observes | CPU-time IP distribution without hot-loop edits | D2-E task-clock route | 1.0 | primary observer | machine instruction ranges | primary | d2-primary |
| s9 | instruction sampler | observes_independently | retired-instruction IP distribution without event substitution | D2-E instruction route | 1.0 | secondary observer | machine instruction ranges | secondary | d2-secondary |
| s10 | attribution owner | joins_only | sample IPs to pre-sealed bucket ranges | D2-F Attribution | 1.0 | aggregation owner | bucket samples | attribution | d2-attribution |
| s11 | validity owner | vetoes | attribution with perturbation loss insufficient samples or excess unattributed work | D2-E validity gates | 1.0 | validity gate | invalid attribution | validity | d2-validity |
| s12 | D2 decision owner | authorizes_only | a future paper route excluding implementation V12 and runtime authority | D2 Decision | 1.0 | decision owner | bounded next paper | decision | d2-decision |

## candidate_triads

| id | subject | relation | object | evidence | confidence | subject_role | object_role | route | group |
|----|---------|----------|--------|----------|------------|--------------|-------------|-------|-------|
| c1 | D1 sealed evidence | controls | D2 static work and CPU denominators | D2 Corrected Static Closure | 1.0 | evidence authority | frozen denominator | static | d2-static |
| c2 | recovered V10 source | proves_absence_of | source-level redundant second state decode | D2 Corrected Static Closure | 1.0 | source authority | rejected hypothesis | static | d2-static |
| c3 | symbolized build owner | changes_only | debug and symbol retention settings | D2-B One Symbolized Build | 1.0 | build owner | immutable source build | build | d2-build |
| c4 | machine closure owner | produces_before | immutable instruction bucket map | D2-C Pre-Measurement Machine Closure | 1.0 | disassembly owner | bucket evidence | map | d2-map |
| c5 | bucket map owner | assigns_before | every traversal instruction range or unattributed status | D2-C frozen buckets | 1.0 | classification owner | instruction ranges | map | d2-map |
| c6 | parity owner | rejects | any semantic structural rank or certificate divergence | D2-D Semantic Control | 1.0 | proof gate | divergent executable | parity | d2-parity |
| c7 | unsampled control owner | compares | D2 traversal CPU and instruction perturbation against sealed D1 | D2-D validity gates | 1.0 | control owner | perturbation baseline | control | d2-control |
| c8 | task-clock sampler | observes | CPU-time IP distribution without hot-loop edits | D2-E task-clock route | 1.0 | primary observer | machine instruction ranges | primary | d2-primary |
| c9 | instruction sampler | observes_independently | retired-instruction IP distribution without event substitution | D2-E instruction route | 1.0 | secondary observer | machine instruction ranges | secondary | d2-secondary |
| c10 | attribution owner | joins_only | sample IPs to pre-sealed bucket ranges | D2-F Attribution | 1.0 | aggregation owner | bucket samples | attribution | d2-attribution |
| c11 | validity owner | vetoes | attribution with perturbation loss insufficient samples or excess unattributed work | D2-E validity gates | 1.0 | validity gate | invalid attribution | validity | d2-validity |
| c12 | D2 decision owner | authorizes_only | a future paper route excluding implementation V12 and runtime authority | D2 Decision | 1.0 | decision owner | bounded next paper | decision | d2-decision |

## notes

- This is paper structure only; authority_ready must remain false.
- D2 implementation preflight, controller, build, bucket map and measurements do not yet exist.
- The source-level duplicate-state-decode hypothesis is rejected by recovered V10 source.
- Sampling buckets are frozen before sample data and cannot be reassigned afterward.
- A D2 result can admit only another paper route, never implementation or runtime authority.

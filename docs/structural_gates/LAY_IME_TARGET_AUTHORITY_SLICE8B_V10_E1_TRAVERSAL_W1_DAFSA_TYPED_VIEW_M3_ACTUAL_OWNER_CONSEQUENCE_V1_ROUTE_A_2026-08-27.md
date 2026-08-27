# NANDA Triad Worksheet

task_id: m3-actual-owner-consequence-v1-route-a
domain: general
query: Does the selected M3 owner preserve one generation owner and avoid per-token or independent reload ownership?

## triads

| id | subject | relation | object | evidence | confidence | subject_role | object_role | route | group |
|----|---------|----------|--------|----------|------------|--------------|-------------|-------|-------|
| s1 | canonical exact generation | owns | validated V13 bytes plus one typed materialization | consequence selected design | 1.0 | generation owner | immutable representation | lifetime | lifetime |
| s2 | prepared token field | borrows | canonical exact generation | source cache audit | 1.0 | request-local borrower | generation owner | lifetime | lifetime |
| s3 | productive V90 reload | invalidates | dependent token caches | l2_field mod source | 1.0 | reload owner | dependent cache | reload | reload |
| s4 | typed DAFSA view | is_not_owned_by | per-token cache | consequence rejection | 1.0 | generation representation | request cache | ownership-veto | ownership-veto |
| s5 | typed DAFSA view | has_no | independent live reload path | consequence boundary | 1.0 | generation representation | forbidden reload route | ownership-veto | ownership-veto |

## candidate_triads

| id | subject | relation | object | evidence | confidence | subject_role | object_role | route | group |
|----|---------|----------|--------|----------|------------|--------------|-------------|-------|-------|
| c1 | canonical exact generation | owns | validated V13 bytes plus one typed materialization | selected implementation scope | 1.0 | generation owner | immutable representation | lifetime | lifetime |
| c2 | prepared token field | borrows | canonical exact generation | selected implementation scope | 1.0 | request-local borrower | generation owner | lifetime | lifetime |
| c3 | productive V90 reload | invalidates | dependent token caches | preserved source behavior | 1.0 | reload owner | dependent cache | reload | reload |
| c4 | typed DAFSA view | is_not_owned_by | per-token cache | explicit veto | 1.0 | generation representation | request cache | ownership-veto | ownership-veto |
| c5 | typed DAFSA view | has_no | independent live reload path | explicit veto | 1.0 | generation representation | forbidden reload route | ownership-veto | ownership-veto |

## notes

- PASS is structural coherence only and grants no source edit or runtime authority.

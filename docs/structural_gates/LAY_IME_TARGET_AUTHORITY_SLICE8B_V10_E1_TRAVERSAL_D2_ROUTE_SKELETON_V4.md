# NANDA Triad Worksheet

task_id: lay-v10-e1-traversal-d2-route-skeleton-v4
domain: code
query: Check the global D2 order, fail-closed boundary, and non-admission state

## triads

| id | subject | relation | object | evidence | confidence | subject_role | object_role | route | group |
|----|---------|----------|--------|----------|------------|--------------|-------------|-------|-------|
| s1 | D2 sequence owner | sequences | immutable closure before one build before bucket-map sealing | D2 A through C contract order | 1.0 | route owner | execution order | sequencing | d2-global-v4 |
| s2 | D2 sequence owner | sequences | sealed bucket map before parity controls and sampled subjects | D2 C through E contract order | 1.0 | route owner | execution order | sequencing | d2-global-v4 |
| s3 | D2 sequence owner | separates | unsampled controls task-clock sampling and instruction sampling | D2 D and E independent routes | 1.0 | route owner | observer boundary | observation | d2-global-v4 |
| s4 | D2 sequence owner | withholds | attribution when any validity conjunct fails | D2 E fail-closed verdicts | 1.0 | route owner | claim boundary | validity | d2-global-v4 |
| s5 | D2 sequence owner | does_not_admit | optimization implementation full B V12 runtime integration or deployment | D2 decision and forbidden effects | 1.0 | route owner | later actions | admission | d2-global-v4 |

## candidate_triads

| id | subject | relation | object | evidence | confidence | subject_role | object_role | route | group |
|----|---------|----------|--------|----------|------------|--------------|-------------|-------|-------|
| c1 | D2 sequence owner | sequences | immutable closure before one build before bucket-map sealing | reviewed D2 A through C sequence | 1.0 | route owner | execution order | sequencing | d2-global-v4 |
| c2 | D2 sequence owner | sequences | sealed bucket map before parity controls and sampled subjects | reviewed D2 C through E sequence | 1.0 | route owner | execution order | sequencing | d2-global-v4 |
| c3 | D2 sequence owner | separates | unsampled controls task-clock sampling and instruction sampling | reviewed observer separation | 1.0 | route owner | observer boundary | observation | d2-global-v4 |
| c4 | D2 sequence owner | withholds | attribution when any validity conjunct fails | reviewed fail-closed validity boundary | 1.0 | route owner | claim boundary | validity | d2-global-v4 |
| c5 | D2 sequence owner | does_not_admit | optimization implementation full B V12 runtime integration or deployment | reviewed D2 non-admission boundary | 1.0 | route owner | later actions | admission | d2-global-v4 |

## notes

- Revisions V1 through V3 are retained as `VETO`; V4 uses a global skeleton plus local owner gates.
- Global structural PASS is coherence only and must retain `authority_ready=false`.
- Every required local route must independently pass before the paper contract is reviewed.
- No D2 implementation preflight, build, bucket map, subject execution, or PMU sampling exists.


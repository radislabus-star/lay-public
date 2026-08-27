# NANDA Triad Worksheet

task_id: lay-v10-b-hardware-route-skeleton-v4
domain: code
query: Check global B sequence after pre-B2 perf audit correction V3

## triads

| id | subject | relation | object | evidence | confidence | subject_role | object_role | route | group |
|----|---------|----------|--------|----------|------------|--------------|-------------|-------|-------|
| s1 | corrected B evidence contract | records | two version-or-usage-only pre-B2 perf invocations and zero PMU measurements | exact session audit | 1.0 | route owner | audit boundary | audit | global-skeleton-v4 |
| s2 | corrected B evidence contract | sequences | V3 preflight before controller implementation | changed frozen audit fact invalidates V2 execution use | 1.0 | route owner | execution order | sequencing | global-skeleton-v4 |
| s3 | corrected B evidence contract | sequences | controller implementation before B0a | implementation state machine | 1.0 | route owner | execution order | sequencing | global-skeleton-v4 |
| s4 | corrected B evidence contract | sequences | B0a before one build before one freezer before B0b before B1 before B2 | correction V2 owner sequence remains unchanged | 1.0 | route owner | execution order | sequencing | global-skeleton-v4 |
| s5 | corrected B evidence contract | does_not_admit | B3 parity B5 B6 and V12 | STOP after benign B2 | 1.0 | route owner | later actions | admission | global-skeleton-v4 |

## candidate_triads

| id | subject | relation | object | evidence | confidence | subject_role | object_role | route | group |
|----|---------|----------|--------|----------|------------|--------------|-------------|-------|-------|
| c1 | corrected B evidence contract | records | two version-or-usage-only pre-B2 perf invocations and zero PMU measurements | contract audit correction V3 | 1.0 | route owner | audit boundary | audit | global-skeleton-v4 |
| c2 | corrected B evidence contract | sequences | V3 preflight before controller implementation | contract audit correction V3 | 1.0 | route owner | execution order | sequencing | global-skeleton-v4 |
| c3 | corrected B evidence contract | sequences | controller implementation before B0a | contract admission sequence | 1.0 | route owner | execution order | sequencing | global-skeleton-v4 |
| c4 | corrected B evidence contract | sequences | B0a before one build before one freezer before B0b before B1 before B2 | contract B0 sequencing correction V2 | 1.0 | route owner | execution order | sequencing | global-skeleton-v4 |
| c5 | corrected B evidence contract | does_not_admit | B3 parity B5 B6 and V12 | contract claim boundary | 1.0 | route owner | later actions | admission | global-skeleton-v4 |

## notes

- V1 and V2 preflight evidence remain immutable historical artifacts.
- Global structural PASS is coherence only and cannot replace V3 implementation preflight.

# NANDA Triad Worksheet

task_id: m3-end-to-end-v8-route-a-owner
domain: general
query: Does V8 keep one validated typed generation owner and reject mixed or stale publication?

## triads

| id | subject | relation | object | evidence | confidence | subject_role | object_role | route | group |
|----|---------|----------|--------|----------|------------|--------------|-------------|-------|-------|
| s1 | validated current sidecar | creates_once | immutable typed generation | end-to-end-v8.md:69-75 | 1.0 | validated byte source | immutable generation evidence | construction | construction |
| s2 | exact request | borrows | current generation Arc | end-to-end-v8.md:75-76 | 1.0 | request reader | immutable generation owner | borrowing | borrowing |
| s3 | generation B publication | supersedes | generation A commit authority | end-to-end-v8.md:192-195 | 1.0 | current generation publication | stale result authority | invalidation | invalidation |
| s4 | held generation A Arc | preserves | reader memory safety | end-to-end-v8.md:195-196 | 1.0 | stale immutable evidence | active reader lifetime | reader-lifetime | reader-lifetime |
| s5 | failed generation C construction | preserves | current generation B | end-to-end-v8.md:197-200 | 1.0 | failed unpublished candidate | current owner state | rollback | rollback |
| s6 | request key | cannot_create | typed materialization | end-to-end-v8.md:91-93 | 1.0 | request-local identity | forbidden generation owner | owner-veto | owner-veto |

## candidate_triads

| id | subject | relation | object | evidence | confidence | subject_role | object_role | route | group |
|----|---------|----------|--------|----------|------------|--------------|-------------|-------|-------|
| c1 | validated current sidecar | creates_once | immutable typed generation | one materialization per generation | 1.0 | validated byte source | immutable generation evidence | construction | construction |
| c2 | exact request | borrows | current generation Arc | request borrow API | 1.0 | request reader | immutable generation owner | borrowing | borrowing |
| c3 | generation B publication | supersedes | generation A commit authority | current-generation commit check | 1.0 | current generation publication | stale result authority | invalidation | invalidation |
| c4 | held generation A Arc | preserves | reader memory safety | Arc lifetime check | 1.0 | stale immutable evidence | active reader lifetime | reader-lifetime | reader-lifetime |
| c5 | failed generation C construction | preserves | current generation B | failure injection | 1.0 | failed unpublished candidate | current owner state | rollback | rollback |
| c6 | request key | cannot_create | typed materialization | zero per-request materializations | 1.0 | request-local identity | forbidden generation owner | owner-veto | owner-veto |

## notes

- This is a test-only owner mechanism, not an installed reload implementation.

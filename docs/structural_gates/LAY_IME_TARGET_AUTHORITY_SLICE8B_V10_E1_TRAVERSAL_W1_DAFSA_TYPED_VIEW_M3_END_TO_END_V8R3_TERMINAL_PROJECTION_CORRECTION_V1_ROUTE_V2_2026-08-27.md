# NANDA Triad Worksheet

task_id: m3-end-to-end-v8r3-terminal-projection-correction-v1-route-v2
domain: general
query: Does V8R3 correct terminal projection and response retention without changing the direct-exec scientific route?

## triads

| id | subject | relation | object | evidence | confidence | subject_role | object_role | route | group |
|----|---------|----------|--------|----------|------------|--------------|-------------|-------|-------|
| s1 | V8R2 history | remains | immutable pre-marker BLOCKED_PROVENANCE | terminal-projection-v1.md:7-25 | 1.0 | historical experiment | terminal predecessor | history | history |
| s2 | V8R1 remote state | represents | E2E_CREATED_UNAUDITED producer observation | terminal-projection-v1.md:31-34 | 1.0 | remote producer | immutable observation | remote_projection | remote_projection |
| s3 | V8R1 terminal receipt | represents | independent BLOCKED_PROVENANCE verdict | terminal-projection-v1.md:36-43 | 1.0 | local auditor | terminal authority | local_verdict | local_verdict |
| s4 | V8R3 admission | validates_together | remote producer state and local terminal verdict | terminal-projection-v1.md:29-43 | 1.0 | admission gate | predecessor pair | admission | admission |
| s5 | V8R3 journal | retains_before_dispatch | complete structured external response | terminal-projection-v1.md:79-94 | 1.0 | crash-safe recorder | external observation | journal | journal |
| s6 | V8R3 route | reuses | exact V8R1 ELF and V8R2 direct command | terminal-projection-v1.md:96-116 | 1.0 | execution envelope | sealed scientific subject | execution | execution |
| s7 | V8R3 route | excludes | build loader perf production and retry routes | terminal-projection-v1.md:96-163 | 1.0 | closed command graph | forbidden producers | veto | veto |
| s8 | V8R3 PASS | requires | complete fresh V8 scientific receipt | terminal-projection-v1.md:118-159 | 1.0 | scoped verdict | conjunctive proof | science | science |
| s9 | V8R3 PASS | cannot_grant | production activation or runtime mutation | terminal-projection-v1.md:165-169 | 1.0 | test-owner evidence | production authority | boundary | boundary |

## candidate_triads

| id | subject | relation | object | evidence | confidence | subject_role | object_role | route | group |
|----|---------|----------|--------|----------|------------|--------------|-------------|-------|-------|
| c1 | V8R2 history | remains | immutable pre-marker BLOCKED_PROVENANCE | exact V8R2 journal manifest | 1.0 | historical experiment | terminal predecessor | history | history |
| c2 | V8R1 remote state | represents | E2E_CREATED_UNAUDITED producer observation | exact remote state and wrapper SHA | 1.0 | remote producer | immutable observation | remote_projection | remote_projection |
| c3 | V8R1 terminal receipt | represents | independent BLOCKED_PROVENANCE verdict | exact local terminal receipt SHA | 1.0 | local auditor | terminal authority | local_verdict | local_verdict |
| c4 | V8R3 admission | validates_together | remote producer state and local terminal verdict | independent live snapshot | 1.0 | admission gate | predecessor pair | admission | admission |
| c5 | V8R3 journal | retains_before_dispatch | complete structured external response | journal fault model | 1.0 | crash-safe recorder | external observation | journal | journal |
| c6 | V8R3 route | reuses | exact V8R1 ELF and V8R2 direct command | byte and argv parity tests | 1.0 | execution envelope | sealed scientific subject | execution | execution |
| c7 | V8R3 route | excludes | build loader perf production and retry routes | reachable command graph audit | 1.0 | closed command graph | forbidden producers | veto | veto |
| c8 | V8R3 PASS | requires | complete fresh V8 scientific receipt | independent terminal dispatch | 1.0 | scoped verdict | conjunctive proof | science | science |
| c9 | V8R3 PASS | cannot_grant | production activation or runtime mutation | explicit claim boundary | 1.0 | test-owner evidence | production authority | boundary | boundary |

## notes

- V1 is retained as structurally coherent but carried an avoidable shared-span repair item.
- V2 separates remote producer observation from local terminal verdict evidence.
- Structural PASS is coherence only; implementation requires a separate READY preflight.

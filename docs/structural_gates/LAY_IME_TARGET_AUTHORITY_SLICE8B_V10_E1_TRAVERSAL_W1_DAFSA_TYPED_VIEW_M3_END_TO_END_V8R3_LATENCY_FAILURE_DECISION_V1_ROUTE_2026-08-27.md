# NANDA Triad Worksheet

task_id: m3-v8r3-latency-failure-decision-v1-route
domain: general
query: Does the V8R3 latency decision preserve terminal evidence and admit only a fresh non-authoritative materialization decomposition?

## triads

| id | subject | relation | object | evidence | confidence | subject_role | object_role | route | group |
|----|---------|----------|--------|----------|------------|--------------|-------------|-------|-------|
| s1 | V8R3 | remains | immutable BLOCKED_LATENCY terminal | latency-decision-v1.md:92-106 | 1.0 | historical experiment | terminal verdict | history | history |
| s2 | V8R3 passing evidence | remains_valid_within | exact test-only semantic capacity RSS and reload scope | latency-decision-v1.md:92-96 | 1.0 | scoped predecessor evidence | retained claim | history | history |
| s3 | V8R3 measurement | proves | final materialization dominates measured p99 failure | latency-decision-v1.md:66-75 | 1.0 | measured outer span | bounded latency finding | interpretation | interpretation |
| s4 | V8R3 measurement | does_not_prove | dominant operation inside final materialization | latency-decision-v1.md:73-90 | 1.0 | measured outer span | unmeasured inner cause | interpretation | interpretation |
| s5 | diagnostic successor | uses | exact sealed V8R3 ELF and fixed input identities | latency-decision-v1.md:108-127 | 1.0 | fresh diagnostic route | immutable predecessor bytes | execution | execution |
| s6 | diagnostic successor | consumes_before | one fresh one-shot marker before subject execution | latency-decision-v1.md:108-127 | 1.0 | one-shot route | execution authority | execution | execution |
| s7 | trace parser | requires | exactly 1910 ordered rows across warmup and four measured rounds | latency-decision-v1.md:129-145 | 1.0 | evidence consumer | complete trace stream | evidence | evidence |
| s8 | trace evidence | attributes | aggregate stage dominance without per-request outer join | latency-decision-v1.md:147-168 | 1.0 | diagnostic observation | bounded mechanism claim | evidence | evidence |
| s9 | failure dispatcher | prioritizes | provenance semantic capability then complete decomposition | latency-decision-v1.md:170-180 | 1.0 | terminal classifier | exactly one terminal state | terminal | terminal |
| s10 | diagnostic result | cannot_grant | runtime edit gate bypass deployment or production authority | latency-decision-v1.md:182-202 | 1.0 | diagnostic evidence | forbidden authority | boundary | boundary |

## candidate_triads

| id | subject | relation | object | evidence | confidence | subject_role | object_role | route | group |
|----|---------|----------|--------|----------|------------|--------------|-------------|-------|-------|
| c1 | V8R3 | remains | immutable BLOCKED_LATENCY terminal | exact terminal receipt SHA and consumed marker | 1.0 | historical experiment | terminal verdict | history | history |
| c2 | V8R3 passing evidence | remains_valid_within | exact test-only semantic capacity RSS and reload scope | terminal gate projection | 1.0 | scoped predecessor evidence | retained claim | history | history |
| c3 | V8R3 measurement | proves | final materialization dominates measured p99 failure | frozen outer timing distribution | 1.0 | measured outer span | bounded latency finding | interpretation | interpretation |
| c4 | V8R3 measurement | does_not_prove | dominant operation inside final materialization | absent inner timing in V8R3 receipt | 1.0 | measured outer span | unmeasured inner cause | interpretation | interpretation |
| c5 | diagnostic successor | uses | exact sealed V8R3 ELF and fixed input identities | bootstrap identity contract | 1.0 | fresh diagnostic route | immutable predecessor bytes | execution | execution |
| c6 | diagnostic successor | consumes_before | one fresh one-shot marker before subject execution | atomic marker rename contract | 1.0 | one-shot route | execution authority | execution | execution |
| c7 | trace parser | requires | exactly 1910 ordered rows across warmup and four measured rounds | fixed loop and schedule cardinality | 1.0 | evidence consumer | complete trace stream | evidence | evidence |
| c8 | trace evidence | attributes | aggregate stage dominance without per-request outer join | trace schema and explicit claim boundary | 1.0 | diagnostic observation | bounded mechanism claim | evidence | evidence |
| c9 | failure dispatcher | prioritizes | provenance semantic capability then complete decomposition | frozen terminal table | 1.0 | terminal classifier | exactly one terminal state | terminal | terminal |
| c10 | diagnostic result | cannot_grant | runtime edit gate bypass deployment or production authority | explicit forbidden action graph | 1.0 | diagnostic evidence | forbidden authority | boundary | boundary |

## notes

- Historical trace logs are hypothesis evidence only and do not replace the fresh route.
- Structural PASS is coherence only. Controller code requires a separate READY implementation preflight.

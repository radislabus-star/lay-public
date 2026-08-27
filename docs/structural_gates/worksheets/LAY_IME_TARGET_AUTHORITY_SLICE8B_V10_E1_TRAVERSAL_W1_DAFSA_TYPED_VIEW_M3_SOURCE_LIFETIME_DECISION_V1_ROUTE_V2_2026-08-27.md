# NANDA Triad Worksheet

task_id: w1-dafsa-typed-view-m3-source-lifetime-decision-v1-route-v2
domain: general
query: Does the M3 source decision select one generation-scoped test owner while preserving the production boundary?

## triads

| id | subject | relation | object | evidence | confidence | subject_role | object_role | route | group |
|----|---------|----------|--------|----------|------------|--------------|-------------|-------|-------|
| s1 | M3 terminal PASS | establishes | typed view W1 gain with exact parity | 11.795426 percent CPU gain and zero parity mismatches | 1.0 | measured predecessor | scoped mechanism result | measured-result | measured-result |
| s2 | M3 typed payload | requires | one generation-scoped owner | 3689628 bytes constructed outside traversal denominator | 1.0 | measured lifetime cost | owner obligation | lifetime-owner | lifetime-owner |
| s3 | generation owner | couples atomically | validated bytes and typed view identity | source decision requires one immutable view per sidecar generation | 1.0 | sole future owner | generation identity | generation-identity | generation-identity |
| s4 | future request route | borrows | shared immutable generation view | source decision rejects query-local materialization | 1.0 | future reader | generation-owned view | request-lifetime | request-lifetime |
| s5 | source decision | rejects | unsafe cast native format and independent reload owners | alignment format compatibility and stale identity remain unproved | 1.0 | decision owner | forbidden designs | rejected-designs | rejected-designs |
| s6 | source decision | leaves blocked | production source promotion | authority latency RSS reload quality and rollback remain open | 1.0 | bounded paper authority | production action | production-boundary | production-boundary |
| s7 | source decision | admits no | compile remote experiment install restart or deployment | terminal successor is paper-only and runtime authority is unchanged | 1.0 | current decision | forbidden side effects | stop-boundary | stop-boundary |

## candidate_triads

| id | subject | relation | object | evidence | confidence | subject_role | object_role | route | group |
|----|---------|----------|--------|----------|------------|--------------|-------------|-------|-------|
| c1 | M3 terminal PASS | establishes | typed view W1 gain with exact parity | candidate preserves the measured scope | 1.0 | measured predecessor | scoped mechanism result | measured-result | measured-result |
| c2 | M3 typed payload | requires | one generation-scoped owner | candidate carries measured lifetime cost forward | 1.0 | measured lifetime cost | owner obligation | lifetime-owner | lifetime-owner |
| c3 | generation owner | couples atomically | validated bytes and typed view identity | candidate has one identity owner | 1.0 | sole future owner | generation identity | generation-identity | generation-identity |
| c4 | future request route | borrows | shared immutable generation view | candidate creates no request-local copy | 1.0 | future reader | generation-owned view | request-lifetime | request-lifetime |
| c5 | source decision | rejects | unsafe cast native format and independent reload owners | candidate does not open unproved designs | 1.0 | decision owner | forbidden designs | rejected-designs | rejected-designs |
| c6 | source decision | leaves blocked | production source promotion | candidate does not claim production authority | 1.0 | bounded paper authority | production action | production-boundary | production-boundary |
| c7 | source decision | admits no | compile remote experiment install restart or deployment | candidate terminates at immutable paper decision | 1.0 | current decision | forbidden side effects | stop-boundary | stop-boundary |

## notes

- This gate checks structural coherence only; `authority_ready=false` is expected.
- Any future code still requires its own consequence analysis and implementation preflight.

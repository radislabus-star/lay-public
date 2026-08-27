# NANDA Triad Worksheet

task_id: m3-actual-owner-sidecar-identity-correction-v5
domain: general
query: Does V5 preserve historical sidecar identity while admitting the declared source-bound header change?

## triads

| id | subject | relation | object | evidence | confidence | subject_role | object_role | route | group |
|----|---------|----------|--------|----------|------------|--------------|-------------|-------|-------|
| s1 | historical V11 full SHA | binds | historical Phase 7D source digest | correction-v5.md:15-21 | 1.0 | immutable historical identity | historical source identity | history | history |
| s2 | current V11 header | binds | current Phase 7D source digest | correction-v5.md:24-29 | 1.0 | current generated identity | current source identity | current | current |
| s3 | historical-header projection | must_equal | historical V11 full SHA | correction-v5.md:38-53 | 1.0 | non-header byte proof | immutable historical identity | projection | projection |
| s4 | current payload checksum | must_equal | recomputed current payload checksum | correction-v5.md:38-43 | 1.0 | encoded payload identity | independently recomputed identity | payload | payload |
| s5 | current V13DafsaView | validates | unmodified current V11 bytes | correction-v5.md:55 | 1.0 | current format validator | current generated evidence | validation | validation |
| s6 | owner fixed proof | begins_after | historical-header projection PASS | correction-v5.md:56-57 | 1.0 | semantic proof consumer | provenance gate | execution | execution |
| s7 | V4 failure | forbids | retry under V4 | correction-v5.md:31 | 1.0 | immutable terminal history | forbidden execution | one-shot | one-shot |
| s8 | projected historical clone | cannot_be | parsed searched written or published | correction-v5.md:57-58 | 1.0 | proof-only projection | forbidden runtime input | veto | veto |

## candidate_triads

| id | subject | relation | object | evidence | confidence | subject_role | object_role | route | group |
|----|---------|----------|--------|----------|------------|--------------|-------------|-------|-------|
| c1 | historical V11 full SHA | binds | historical Phase 7D source digest | sealed D2 source plus V11 Gate A | 1.0 | immutable historical identity | historical source identity | history | history |
| c2 | current V11 header | binds | current Phase 7D source digest | current source-bound encoder | 1.0 | current generated identity | current source identity | current | current |
| c3 | historical-header projection | must_equal | historical V11 full SHA | exact SHA-256 gate | 1.0 | non-header byte proof | immutable historical identity | projection | projection |
| c4 | current payload checksum | must_equal | recomputed current payload checksum | exact SHA-256 gate | 1.0 | encoded payload identity | independently recomputed identity | payload | payload |
| c5 | current V13DafsaView | validates | unmodified current V11 bytes | current loader gate | 1.0 | current format validator | current generated evidence | validation | validation |
| c6 | owner fixed proof | begins_after | historical-header projection PASS | controller order | 1.0 | semantic proof consumer | provenance gate | execution | execution |
| c7 | V4 failure | forbids | retry under V4 | immutable V4 receipt | 1.0 | immutable terminal history | forbidden execution | one-shot | one-shot |
| c8 | projected historical clone | cannot_be | parsed searched written or published | execution veto | 1.0 | proof-only projection | forbidden runtime input | veto | veto |

## notes

- V5 changes the interpretation of a source-bound header field only.
- No payload difference, historical receipt rewrite or V4 retry is admitted.

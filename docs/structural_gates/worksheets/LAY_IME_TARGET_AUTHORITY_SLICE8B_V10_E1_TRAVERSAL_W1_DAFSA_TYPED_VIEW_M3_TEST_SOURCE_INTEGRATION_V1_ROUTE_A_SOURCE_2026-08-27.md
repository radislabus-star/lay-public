# NANDA Triad Worksheet

task_id: m3-test-source-integration-v1-route-a-source
domain: general
query: Does the M3 integration preserve exact source provenance and the test-only module boundary?

## triads

| id | subject | relation | object | evidence | confidence | subject_role | object_role | route | group |
|----|---------|----------|--------|----------|------------|--------------|-------------|-------|-------|
| s1 | M3 experiment source | differs_from | current V13 test source | exact SHA and byte-size comparison | 1.0 | measured source | integration baseline | provenance | provenance |
| s2 | source difference | requires | independent semantic and physical reproof | contract critical source boundary | 1.0 | provenance fact | proof obligation | reproof | reproof |
| s3 | v13_typed_peak module | is_guarded_by | cfg test | l2_field mod source | 1.0 | test source owner | compilation boundary | test-boundary | test-boundary |
| s4 | integration edit set | contains_only | current V13 file and typed_exact submodule | selected source design | 1.0 | scoped editor | admitted paths | source-scope | source-scope |
| s5 | production modules | remain | byte-identical | explicit non-change contract | 1.0 | protected source | preserved baseline | production-preservation | production-preservation |

## candidate_triads

| id | subject | relation | object | evidence | confidence | subject_role | object_role | route | group |
|----|---------|----------|--------|----------|------------|--------------|-------------|-------|-------|
| c1 | M3 experiment source | differs_from | current V13 test source | candidate acknowledges no byte transfer | 1.0 | measured source | integration baseline | provenance | provenance |
| c2 | source difference | requires | independent semantic and physical reproof | candidate makes no inherited codegen claim | 1.0 | provenance fact | proof obligation | reproof | reproof |
| c3 | v13_typed_peak module | is_guarded_by | cfg test | candidate stays test-only | 1.0 | test source owner | compilation boundary | test-boundary | test-boundary |
| c4 | integration edit set | contains_only | current V13 file and typed_exact submodule | candidate edit graph is exact | 1.0 | scoped editor | admitted paths | source-scope | source-scope |
| c5 | production modules | remain | byte-identical | candidate has no runtime edit | 1.0 | protected source | preserved baseline | production-preservation | production-preservation |

## notes

- Source-route PASS is coherence only and grants no implementation authority.

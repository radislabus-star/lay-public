# NANDA Triad Worksheet

task_id: m3-actual-owner-semantic-diagnosis-v6
domain: general
query: Does V6 diagnose the sealed semantic mismatch without changing owner behavior or promoting the result?

## triads

| id | subject | relation | object | evidence | confidence | subject_role | object_role | route | group |
|----|---------|----------|--------|----------|------------|--------------|-------------|-------|-------|
| s1 | V5 semantic receipt | freezes | 146 lattice and 72 gate mismatches | diagnosis-v6.md:9-24 | 1.0 | immutable observation | fixed diagnostic target | history | history |
| s2 | 36 punctuation cases | form | bounded diagnosis prior | diagnosis-v6.md:26-36 | 1.0 | sealed input projection | non-authoritative hypothesis | prior | prior |
| s3 | V6 proof instrumentation | observes | existing owner behavior | diagnosis-v6.md:38-54 | 1.0 | diagnostic observer | unchanged semantic route | observation | observation |
| s4 | V6 proof instrumentation | cannot_change | owner material lattice gate behavior | diagnosis-v6.md:51-54 | 1.0 | diagnostic observer | protected implementation | veto | veto |
| s5 | V6 diagnostic PASS | requires | complete 764-request denominator | diagnosis-v6.md:56-67 | 1.0 | diagnostic verdict | fixed denominator | denominator | denominator |
| s6 | V6 diagnostic result | cannot_grant | production or repair authority | diagnosis-v6.md:69-76 | 1.0 | scoped evidence | forbidden authority | authority | authority |

## candidate_triads

| id | subject | relation | object | evidence | confidence | subject_role | object_role | route | group |
|----|---------|----------|--------|----------|------------|--------------|-------------|-------|-------|
| c1 | V5 semantic receipt | freezes | 146 lattice and 72 gate mismatches | immutable V5 receipt | 1.0 | immutable observation | fixed diagnostic target | history | history |
| c2 | 36 punctuation cases | form | bounded diagnosis prior | sealed V7 read-only count | 1.0 | sealed input projection | non-authoritative hypothesis | prior | prior |
| c3 | V6 proof instrumentation | observes | existing owner behavior | test-only instrumentation | 1.0 | diagnostic observer | unchanged semantic route | observation | observation |
| c4 | V6 proof instrumentation | cannot_change | owner material lattice gate behavior | source hash veto | 1.0 | diagnostic observer | protected implementation | veto | veto |
| c5 | V6 diagnostic PASS | requires | complete 764-request denominator | exact receipt predicate | 1.0 | diagnostic verdict | fixed denominator | denominator | denominator |
| c6 | V6 diagnostic result | cannot_grant | production or repair authority | claim boundary | 1.0 | scoped evidence | forbidden authority | authority | authority |

## notes

- V6 is a fresh diagnostic namespace, not a V5 retry.
- The 36-case correlation is explicitly not treated as causal proof before V6.

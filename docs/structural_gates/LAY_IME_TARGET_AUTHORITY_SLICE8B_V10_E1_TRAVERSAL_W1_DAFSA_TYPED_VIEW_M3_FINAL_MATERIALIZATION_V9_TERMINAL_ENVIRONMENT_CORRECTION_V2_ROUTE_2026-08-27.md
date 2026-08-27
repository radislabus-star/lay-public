# NANDA Triad Worksheet

task_id: m3-final-materialization-v9-terminal-environment-correction-v2
domain: general
query: Can V9 terminal classification be corrected offline without retrying the subject or changing immutable history?

## triads

| id | subject | relation | object | evidence | confidence | subject_role | object_role | route | group |
|----|---------|----------|--------|----------|------------|--------------|-------------|-------|-------|
| s1 | V9 terminal V1 | remains | immutable historical BLOCKED_PROVENANCE | correction-v2.md:7-42 | 1.0 | historical audit | retained verdict | history | history |
| s2 | V1 environment parser | misclassified | post-taskset argv assignment as environment | correction-v2.md:7-18 | 1.0 | defective observer | deterministic false predicate | defect | defect |
| s3 | corrected environment parser | accepts_only | contiguous assignments between env and taskset | correction-v2.md:61-88 | 1.0 | bounded parser | exact environment mapping | parser | parser |
| s4 | offline V2 audit | reads_only | sealed V1 terminal and journal trees | correction-v2.md:90-112 | 1.0 | independent evidence consumer | immutable bytes | audit | audit |
| s5 | offline V2 audit | recomputes | 1910 rows distributions and 16-row tail | correction-v2.md:96-107 | 1.0 | independent estimator | retained trace | evidence | evidence |
| s6 | corrected terminal dispatch | prioritizes | provenance semantic capability then completeness | correction-v2.md:114-134 | 1.0 | terminal classifier | exactly one verdict | terminal | terminal |
| s7 | correction route | excludes | SSH subject marker Cargo perf and runtime actions | correction-v2.md:44-59 | 1.0 | closed offline graph | forbidden side effects | veto | veto |
| s8 | corrected positive verdict | cannot_grant | source edit build deployment latency or production authority | correction-v2.md:153-159 | 1.0 | bounded diagnostic evidence | forbidden authority | boundary | boundary |

## candidate_triads

| id | subject | relation | object | evidence | confidence | subject_role | object_role | route | group |
|----|---------|----------|--------|----------|------------|--------------|-------------|-------|-------|
| c1 | V9 terminal V1 | remains | immutable historical BLOCKED_PROVENANCE | exact terminal and journal manifests | 1.0 | historical audit | retained verdict | history | history |
| c2 | V1 environment parser | misclassified | post-taskset argv assignment as environment | exact one-key set difference | 1.0 | defective observer | deterministic false predicate | defect | defect |
| c3 | corrected environment parser | accepts_only | contiguous assignments between env and taskset | anchored parser self-check | 1.0 | bounded parser | exact environment mapping | parser | parser |
| c4 | offline V2 audit | reads_only | sealed V1 terminal and journal trees | closed file registry and zero external calls | 1.0 | independent evidence consumer | immutable bytes | audit | audit |
| c5 | offline V2 audit | recomputes | 1910 rows distributions and 16-row tail | independent parser and summary parity | 1.0 | independent estimator | retained trace | evidence | evidence |
| c6 | corrected terminal dispatch | prioritizes | provenance semantic capability then completeness | synthetic dispatch matrix | 1.0 | terminal classifier | exactly one verdict | terminal | terminal |
| c7 | correction route | excludes | SSH subject marker Cargo perf and runtime actions | AST command graph veto | 1.0 | closed offline graph | forbidden side effects | veto | veto |
| c8 | corrected positive verdict | cannot_grant | source edit build deployment latency or production authority | exact claim-boundary assertions | 1.0 | bounded diagnostic evidence | forbidden authority | boundary | boundary |

## notes

- V1 remains immutable and no subject retry is admitted.
- Structural PASS is coherence only; the offline auditor requires a separate implementation preflight.

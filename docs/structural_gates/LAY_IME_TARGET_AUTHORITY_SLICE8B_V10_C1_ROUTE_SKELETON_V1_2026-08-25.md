# NANDA Triad Worksheet

task_id: lay-v10-c1-route-skeleton-v1
domain: code
query: Check C1-before-B order, Clean V2 supersession, exact timing authority, fairness and failure ownership

## triads

| id | subject | relation | object | evidence | confidence | subject_role | object_role | route | group |
|----|---------|----------|--------|----------|------------|--------------|-------------|-------|-------|
| s1 | order correction owner | sequences | Clean V2 supersession before C1 preflight before one build before parity before latency | C1 order correction corrected sequence | 1.0 | sequencing owner | execution order | sequencing | c1-order-v1 |
| s2 | Clean V2 supersession owner | blocks | old clean aggregate execution through exact state path | sealed Clean V2 state-existence guard | 1.0 | state owner | superseded route | supersession | c1-supersession-v1 |
| s3 | C1 provenance owner | preserves | byte-identical 39047-byte production prefix | exact SHA ce9ea2d29060 | 1.0 | provenance owner | production source | provenance | c1-provenance-v1 |
| s4 | C1 parity owner | precedes | every S and T latency process | separate-process parity prerequisite | 1.0 | proof owner | latency owners | parity | c1-parity-v1 |
| s5 | production timing owner | supplies | search_elapsed_us and total_elapsed_us authority | exact V10 search function | 1.0 | observation owner | acceptance owner | timing | c1-timing-v1 |
| s6 | S owner | produces | five independent 100-round fixed-order sample sets | C1 S contract | 1.0 | execution owner | sample material | single | c1-single-v1 |
| s7 | T owner | produces | five independent 250-round fixed-shard sample sets with START and END barriers | C1 T contract | 1.0 | execution owner | sample material | concurrent | c1-concurrent-v1 |
| s8 | environment owner | admits | every process before execution and validates stable post-state | inherited quiet thresholds and stable projection | 1.0 | admission owner | process execution | environment | c1-environment-v1 |
| s9 | fairness owner | computes | max run-by-worker total p99 and compares with 5000 us | C1 additional fairness conjunct | 1.0 | decision owner | worker samples | fairness | c1-fairness-v1 |
| s10 | C1 verdict owner | distinguishes | C1_PASS C1_FAIL BLOCKED_ENVIRONMENT BLOCKED_PROVENANCE | C1 verdict vocabulary | 1.0 | authority owner | terminal state | decision | c1-verdict-v1 |
| s11 | C1_PASS | closes | V12 latency-optimization branch only | C1 consequences PASS branch | 1.0 | scoped decision | optimization route | decision | c1-pass-consequence-v1 |
| s12 | C1 result | does_not_admit | runtime integration installation full B or V12 implementation | C1 forbidden actions and claim boundary | 1.0 | scoped evidence | later actions | admission | c1-admission-v1 |

## candidate_triads

| id | subject | relation | object | evidence | confidence | subject_role | object_role | route | group |
|----|---------|----------|--------|----------|------------|--------------|-------------|-------|-------|
| c1 | order correction owner | sequences | Clean V2 supersession before C1 preflight before one build before parity before latency | C1 order correction V1 corrected sequence | 1.0 | sequencing owner | execution order | sequencing | c1-order-v1 |
| c2 | Clean V2 supersession owner | blocks | old clean aggregate execution through exact state path | C1 order correction supersession state machine | 1.0 | state owner | superseded route | supersession | c1-supersession-v1 |
| c3 | C1 provenance owner | preserves | byte-identical 39047-byte production prefix | C1 frozen production identity | 1.0 | provenance owner | production source | provenance | c1-provenance-v1 |
| c4 | C1 parity owner | precedes | every S and T latency process | C1 separate semantic prerequisite | 1.0 | proof owner | latency owners | parity | c1-parity-v1 |
| c5 | production timing owner | supplies | search_elapsed_us and total_elapsed_us authority | C1 timing authority | 1.0 | observation owner | acceptance owner | timing | c1-timing-v1 |
| c6 | S owner | produces | five independent 100-round fixed-order sample sets | C1 S route | 1.0 | execution owner | sample material | single | c1-single-v1 |
| c7 | T owner | produces | five independent 250-round fixed-shard sample sets with START and END barriers | C1 T route | 1.0 | execution owner | sample material | concurrent | c1-concurrent-v1 |
| c8 | environment owner | admits | every process before execution and validates stable post-state | C1 environment admission | 1.0 | admission owner | process execution | environment | c1-environment-v1 |
| c9 | fairness owner | computes | max run-by-worker total p99 and compares with 5000 us | C1 acceptance | 1.0 | decision owner | worker samples | fairness | c1-fairness-v1 |
| c10 | C1 verdict owner | distinguishes | C1_PASS C1_FAIL BLOCKED_ENVIRONMENT BLOCKED_PROVENANCE | C1 verdicts | 1.0 | authority owner | terminal state | decision | c1-verdict-v1 |
| c11 | C1_PASS | closes | V12 latency-optimization branch only | C1 consequences PASS branch | 1.0 | scoped decision | optimization route | decision | c1-pass-consequence-v1 |
| c12 | C1 result | does_not_admit | runtime integration installation full B or V12 implementation | C1 forbidden actions and claim boundary | 1.0 | scoped evidence | later actions | admission | c1-admission-v1 |

## notes

- Structural PASS is coherence only and cannot authorize remote supersession, build, parity or latency execution.
- Clean V2 and all earlier evidence remain immutable; supersession is a new state/evidence route.
- C1 is steady-state V10-derived product latency, not an exact historical Gate C replay.

# NANDA Triad Worksheet

task_id: slice8b-v10-e1-traversal-w1-dafsa-typed-view-m3-final-materialization-v9-mechanism-decision-v1
domain: general
query: Does V9 localize the final-materialization tail to the aggregate candidate-admission span without falsely selecting an internal predicate or granting optimization authority?

## triads

| id | subject | relation | object | evidence | confidence | subject_role | object_role | route | group | layer | owner | entrypoint | output | evidence_path | scope |
|----|---------|----------|--------|----------|------------|--------------|-------------|-------|-------|-------|-------|------------|--------|---------------|-------|
| t1 | V9 authoritative terminal | has_verdict | FINAL_MATERIALIZATION_DECOMPOSED | sealed V2 receipt | 1.0 | measured predecessor | scoped result | evidence | history | diagnostic | offline auditor | immutable receipt | exact verdict | V9 V2 terminal receipt | V9 |
| t2 | V9 V1 terminal | remains_immutable | BLOCKED_PROVENANCE | sealed historical receipt | 1.0 | historical predecessor | historical verdict | history | provenance | diagnostic | V1 auditor | parser defect | no mutation | V9 V1 terminal receipt | V9 V1 |
| t3 | V9 correction | performs | zero subject and remote actions | V2 receipt counters | 1.0 | offline interpreter | side-effect boundary | correction | non-change | control | V2 auditor | retained bytes | zero actions | V9 V2 terminal receipt | correction |
| t4 | measured trace | contains | 1528 scientific rows | exact parser closure | 1.0 | evidence stream | denominator | measurement | rows | diagnostic | V2 auditor | rows 382 through 1909 | fixed count | TRACE_ROWS and V2 receipt | measured |
| t5 | gate_us | contributes | 99.2595087301865 percent of tail aggregate | 226939 of 228632 us | 1.0 | measured stage | tail total | measurement | dominance | diagnostic | trace auditor | top 16 rows | unique dominant | V9 V2 terminal receipt | tail |
| t6 | gate_us | is_largest_in | 16 of 16 tail rows | V2 recomputation | 1.0 | measured stage | tail rows | measurement | dominance | diagnostic | trace auditor | per-row stage maximum | exact | V9 V2 terminal receipt | tail |
| t7 | gate_us timer | encloses | admit_candidate_proposal plus live-authority override | exact source inspection | 1.0 | timer span | code path | source | boundary | Rust | materialize_live_candidates | accumulated elapsed | aggregate only | src/nanda_wave/l2_field/productive_v1/live.rs | current source |
| t8 | admit_candidate_proposal | delegates_to | candidate_admission | exact source inspection | 1.0 | route adapter | decision chain | source | call | Rust | TransitionDecisionCore | gate_candidate_with_origin | CandidateGateDecision | decision.rs and proposal_admission.rs | current source |
| t9 | candidate_admission | contains | multiple short-circuit authority predicates | exact source inspection | 1.0 | decision chain | predicate family | source | mechanism | Rust | proposal admission | first matching decision | one decision | proposal_admission.rs | current source |
| t10 | V9 timer | does_not_distinguish | internal predicates from post-call override | timer placement | 1.0 | aggregate observation | submechanism ownership | claim | limitation | diagnostic | V9 trace | one accumulated field | UNKNOWN | live.rs | V9 |
| t11 | tail case ordinals | repeat_across | four measured rounds | exact trace rows | 1.0 | fixed cases | schedule repetitions | measurement | reproducibility | diagnostic | V2 auditor | ordinals 375 371 223 366 | stable ranges | TRACE_ROWS | measured |
| t12 | candidate count | is_not_sufficient_for | gate cost prediction | 48 and 51 surface rows are much cheaper | 1.0 | observed cardinality | causal explanation | inference | boundary | diagnostic | trace comparison | fixed rows | submechanism still unknown | TRACE_ROWS | measured |
| t13 | mechanism decision | forbids | predicate removal cache cap bypass and case special-casing | claim boundary | 1.0 | paper decision | unsafe optimizations | negative | safety | authority | decision paper | verdict | no implementation | mechanism decision paper | successor |
| t14 | admitted successor | is_only | fresh admission-substage diagnostic paper | bounded next tree | 1.0 | paper decision | next research route | decision | next | diagnostic | mechanism decision | positive boundary | paper only | mechanism decision paper | successor |
| t15 | future instrumentation | must_preserve | candidate certificate gate and reason parity | consequence contract | 1.0 | observer | semantic authority | proof | parity | test-only | future paper | exact existing path | zero mismatches | mechanism decision paper | future |
| t16 | future instrumentation | is_absent_from | production builds | consequence contract | 1.0 | test observer | production path | negative | authority | compile boundary | future implementation | cfg boundary | no runtime cost | mechanism decision paper | future |
| t17 | current decision | changes_no | runtime or production authority | paper-only action | 1.0 | decision transaction | deployed state | history | non-change | runtime | current agent | documentation | false | mechanism decision paper | current |

## candidate_triads

| id | subject | relation | object | evidence | confidence | subject_role | object_role | route | group | layer | owner | entrypoint | output | evidence_path | scope |
|----|---------|----------|--------|----------|------------|--------------|-------------|-------|-------|-------|-------|------------|--------|---------------|-------|
| c1 | V9 mechanism result | states | aggregate candidate-admission span is dominant | candidate answer | 0.99 | scoped conclusion | measured mechanism | decision | result | diagnostic | mechanism paper | V9 PASS | exact aggregate claim | mechanism decision paper | V9 |
| c2 | V9 mechanism result | leaves | internal dominant predicate UNKNOWN | candidate answer | 0.99 | scoped conclusion | unresolved submechanism | decision | limitation | diagnostic | mechanism paper | timer scope | no overclaim | mechanism decision paper | V9 |
| c3 | next route | decomposes | existing admission predicates and live-authority override | candidate answer | 0.99 | future diagnostic | aggregate span | research | decomposition | test-only | future paper | opt-in observer | per-stage evidence | mechanism decision paper | successor |
| c4 | next route | preserves | existing short-circuit order and final decisions | candidate answer | 0.99 | observer route | authority semantics | proof | parity | test-only | future implementation | same decision path | zero mismatches | mechanism decision paper | successor |
| c5 | current route | stops_before | source edit compile subject or deployment | candidate answer | 0.99 | paper route | side effects | decision | stop | control | current decision | final status | documentation only | mechanism decision paper | current |
| c6 | production optimization | remains_blocked_until | submechanism proof and separate consequence closure | candidate answer | 0.99 | forbidden action | promotion gates | authority | boundary | production | future decision | independent preflight | no current edit | mechanism decision paper | production |

## notes

- This gate checks coherence only and grants no implementation authority.
- V8R3 remains immutable BLOCKED_LATENCY; V9 diagnostic timings do not replace it.

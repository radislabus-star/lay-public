# NANDA Triad Worksheet

task_id: d3-estimator-recovery
domain: general
query: Can D3 reuse D2 evidence and run only U2-SINGLE then T2-SINGLE with corrected estimator and one-shot markers?

## triads

| id | subject | relation | object | evidence | confidence | subject_role | object_role | route | group | layer | owner | entrypoint | output | evidence_path | scope |
|----|---------|----------|--------|----------|------------|--------------|-------------|-------|-------|-------|-------|------------|--------|---------------|-------|
| t1 | D2 | terminal_verdict | BLOCKED_PROVENANCE | sealed_receipt | 1.0 | predecessor | verdict | history | closure | paper | D2 audit | terminal receipt | no retry | docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_D2_PRIMARY_ONLY_T_SINGLE_TERMINAL_AUDIT_V1_2026-08-26/T_SINGLE_TERMINAL_AUDIT_RECEIPT.json | D2 |
| t2 | D2 T-SINGLE | consumed | historical marker | sealed_state | 1.0 | historical route | marker | history | closure | state | D2 controller | marker rename | consumed-before-exec | remote live state | D2 |
| t3 | D2 T-FIXED and T-REVERSED | remain | available markers | live_projection | 1.0 | retired routes | markers | history | closure | state | D2 controller | terminal D2 | retired unconsumed | remote live state | D2 |
| t4 | old bucket values | have_authority | diagnostic only | correction_v1 | 1.0 | diagnostic | claim boundary | history | claims | paper | D2 audit | invalid estimator | no scientific attribution | docs/structural_gates/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_D2_T_SINGLE_ESTIMATOR_SCOPE_CORRECTION_V1_2026-08-26.md | D2 |
| t5 | D3 | reuses | exact audited D2 ELF and map | machine_identity | 1.0 | recovery route | predecessors | reuse | identity | machine | D3 controller | predecessor closure | no new build or map | D3 paper V1 | D3 |
| t6 | D3 route registry | equals | U2-SINGLE then T2-SINGLE | paper_contract | 1.0 | registry | routes | execution | route graph | control | D3 controller | dispatcher | two routes | D3 paper V1 | D3 V1 |
| t7 | U2-SINGLE | precedes | T2-SINGLE | state_machine | 1.0 | producer | consumer | execution | ordering | state | D3 controller | U2 PASS | T2 admission | D3 paper V1 | D3 V1 |
| t8 | taskset CPU6 | stages | pre-pin input loading | command_contract | 1.0 | launcher | staging scope | U2 and T2 | envelope | process | D3 controller | exact argv | CPU6 before Rust pin | D3 paper V1 | D3 single |
| t9 | subject pin | moves | measured route to CPU0 | sealed_source | 1.0 | subject | scientific CPU domain | U2 and T2 | envelope | subject | D2 ELF | d1_pin_current_thread | CPU0 warmup and measurements | sealed D2 source and DWARF | D3 single |
| t10 | U2 denominator | covers | twenty measured rounds | component_samples | 1.0 | paired denominator | measured work | U2 | denominator | estimator | D3 controller | component sample parser | 20 times edges per round | D3 paper V1 | D3 single |
| t11 | T2 denominator | covers | one warmup plus twenty measured rounds | perf_stream_contract | 1.0 | sampled denominator | sampled work | T2 | denominator | estimator | D3 controller | CPU0 traversal filter | 21 times edges per round | D3 paper V1 | D3 single |
| t12 | common 20-round denominator | would_create | structural warmup bias | arithmetic | 1.0 | forbidden estimator | false comparison | T2 | negative route | estimator | D3 paper | denominator check | forbidden | D3 paper V1 | D3 single |
| t13 | T2 event | fixed_identity | task-clock:u period 200000 | paper_contract | 1.0 | sampling event | perf attr | T2 | command graph | PMU | perf record | whole-process wrap | 5 kHz fixed period | D3 paper V1 | D3 single |
| t14 | host sample-rate | equals | 8000 | sealed_and_live_read | 1.0 | host baseline | limit | T2 | capability | host | D3 controller | proc sys read | stable before and after | D2 P0 and live projection | target host |
| t15 | scientific T2 samples | require | exact Build ID plus CPU0 plus traversal range | preregistered_filter | 1.0 | estimator | accepted sample | T2 | attribution | proof | D3 controller | offline readers | deterministic stream | D3 paper V1 | D3 single |
| t16 | D3 markers | are_disjoint_from | D2 markers | namespace_contract | 1.0 | D3 authority | D2 history | execution | one-shot | state | D3 bootstrap | two new markers | no D2 mutation | D3 paper V1 | D3 |
| t17 | any route failure | consumes | route authority permanently | one_shot_contract | 1.0 | failure transition | marker | execution | terminal | state | D3 controller | consume before effect | no retry | D3 paper V1 | D3 |
| t18 | D3 single PASS | admits | separate multiworker paper only | claim_boundary | 1.0 | result | next paper | decision | claims | authority | future audit | terminal receipt | no optimization | D3 paper V1 | D3 |

## candidate_triads

| id | subject | relation | object | evidence | confidence | subject_role | object_role | route | group | layer | owner | entrypoint | output | evidence_path | scope |
|----|---------|----------|--------|----------|------------|--------------|-------------|-------|-------|-------|-------|------------|--------|---------------|-------|
| c1 | recovery implementation | preserves | terminal D2 history | candidate_answer | 0.99 | candidate route | immutable evidence | history | closure | state | D3 | predecessor verifier | no old marker writes | D3 paper V1 | D3 |
| c2 | executable graph | contains_only | paired U2 and sampled T2 | candidate_answer | 0.99 | candidate registry | allowed routes | execution | route graph | control | D3 | static registry | exact two-route closure | D3 paper V1 | D3 V1 |
| c3 | U2 execution | produces | twenty-round measured CPU per edge | candidate_answer | 0.99 | denominator producer | U2 receipt | U2 | denominator | estimator | D3 | sealed component parser | paired baseline | D3 paper V1 | D3 single |
| c4 | T2 execution | produces | twenty-one-round sampled CPU per edge | candidate_answer | 0.99 | sampled producer | T2 receipt | T2 | denominator | estimator | D3 | accepted sample parser | sampled estimate | D3 paper V1 | D3 single |
| c5 | T2 admission | depends_on | sealed U2 PASS | candidate_answer | 0.99 | consumer | predecessor verdict | execution | ordering | state | D3 | route gate | no early T2 | D3 paper V1 | D3 V1 |
| c6 | CPU-domain filter | excludes | preregistered CPU6 staging samples | candidate_answer | 0.99 | scientific estimator | setup work | T2 | attribution | proof | D3 | cpu equals zero predicate | scope repair | D3 paper V1 | D3 single |
| c7 | fixed 200000 ns period | reduces | requested rate below host ceiling | candidate_answer | 0.95 | mitigation | sample-rate pressure | T2 | capability | host | D3 | perf argv | 5 kHz request | D3 paper V1 | target host |
| c8 | successful D3 V1 | stops_before | multiworker execution and optimization | candidate_answer | 0.99 | terminal result | forbidden authority | decision | claims | authority | D3 audit | terminal receipt | paper-only next admission | D3 paper V1 | D3 |

## notes

- The worksheet checks route, denominator, marker and claim-boundary ownership.
- Actual scientific authority still requires implementation preflight, immutable
  receipts, one-shot execution and independent terminal audit.


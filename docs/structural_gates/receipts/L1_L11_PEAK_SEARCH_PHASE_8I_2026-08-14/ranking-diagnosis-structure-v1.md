# NANDA Triad Worksheet

task_id: l11-phase8i-ranking-diagnosis
domain: code
query: Separate fixed generator objective, typed admission certificates, settlement geometry modes, wave energy, and final authority for Gate C sparse omission losses

## triads

| id | subject | relation | object | evidence | confidence | subject_role | object_role | route | group | layer | owner | entrypoint | output | evidence_path | scope |
|----|---------|----------|--------|----------|------------|--------------|-------------|-------|-------|-------|-------|------------|--------|---------------|-------|
| t1 | fixed heldout reservoir | selects | damaged query and source target | proof.rs:3046 | 1.0 | proof_source | label | proof | ranking-diagnosis | objective | fixed heldout reservoir | prepare_fixed_heldout_cases | immutable cases | src/nanda_wave/lexical_grokking/proof.rs:3046 | proof-only |
| t2 | sampled ambiguity builder | owns | fixed generator objective G | proof.rs:3084 | 1.0 | objective_producer | objective | proof | ranking-diagnosis | objective | sampled ambiguity builder | populate_sampled_ambiguity | objective terminals | src/nanda_wave/lexical_grokking/proof.rs:3084 | proof-only |
| t3 | typed traversal | owns | broad admission E | typed_edit_traversal.rs:348 | 1.0 | admission_producer | candidate_domain | execution | ranking-diagnosis | admission | L1TypedEditTraversal | traverse | terminal certificate map | src/nanda_wave/lexical_grokking/typed_edit_traversal.rs:348 | proof-only |
| t4 | phase7d terminal projection | discards_detail_to | terminal IDs | typed_edit_traversal.rs:1022 | 1.0 | projection | candidate_ids | execution | ranking-diagnosis | adapter | phase7d_terminal_evidence | phase7d_terminal_evidence | terminal ID vector | src/nanda_wave/lexical_grokking/typed_edit_traversal.rs:1022 | proof-only |
| t5 | implicit reconstruction | consumes | terminal IDs | quality.rs:388 | 1.0 | evidence_producer | candidate_ids | execution | ranking-diagnosis | reconstruction | implicit forward owner | evaluate_surface | implicit candidates | src/nanda_wave/lexical_grokking/typed_basin/quality.rs:388 | proof-only |
| t6 | restoration geometry | derives | reconstruction modes M | settlement.rs:454 | 1.0 | geometry_producer | typed_mode_projection | execution | ranking-diagnosis | geometry | runtime settlement owner | apply_candidate_restoration_geometry | candidate geometry fields | src/nanda_wave/lexical_grokking/runtime/settlement.rs:454 | shared-frozen-code |
| t7 | exact settlement | produces | final order W | typed_basin/settlement.rs:411 | 1.0 | ranking_producer | candidate_order | execution | ranking-diagnosis | settlement | exact settlement adapter | settle_exact_case | settled candidates | src/nanda_wave/lexical_grokking/typed_basin/settlement.rs:411 | proof-only |
| t8 | restoration classifier | produces | Winner Tied or Abstain | typed_basin/settlement.rs:550 | 1.0 | authority_producer | readout | authority | ranking-diagnosis | readout | frozen classifier | classify | restoration readout | src/nanda_wave/lexical_grokking/typed_basin/settlement.rs:550 | proof-only |
| t9 | objective labels | observe_after | final order W | quality.rs:475 | 1.0 | observer | candidate_order | observation | ranking-diagnosis | metrics | Gate C recorder | record_damaged | counters and diagnostics | src/nanda_wave/lexical_grokking/typed_basin/quality.rs:475 | proof-only |
| t10 | installed runtime | remains_disjoint_from | ranking diagnostic | phase8i.md:39 | 1.0 | runtime | proof_observer | isolation | ranking-diagnosis | production | installed Lay | daemon and IBus | unchanged bytes | docs/l1-l11-typed-basin-implicit-forward-phase8i.md:39 | production |

## candidate_triads

| id | subject | relation | object | evidence | confidence | subject_role | object_role | route | group | layer | owner | entrypoint | output | evidence_path | scope |
|----|---------|----------|--------|----------|------------|--------------|-------------|-------|-------|-------|-------|------------|--------|---------------|-------|
| c1 | typed traversal | exposes_without_mutation | certificate classes per terminal | proposed diagnostic | 0.95 | admission_producer | diagnostic_evidence | observation | ranking-diagnosis | adapter | phase7d terminal projection | phase7d_terminal_evidence | bounded certificate projection | paper contract section 16.6 | proof-only |
| c2 | implicit reconstruction | exposes_without_mutation | activation coordinates per terminal | proposed diagnostic | 0.95 | evidence_producer | diagnostic_evidence | observation | ranking-diagnosis | reconstruction | Gate C observer | evaluate_surface | bounded activation projection | paper contract section 16.6 | proof-only |
| c3 | exact settlement | exposes_without_mutation | final candidate fields | proposed diagnostic | 0.95 | ranking_producer | diagnostic_evidence | observation | ranking-diagnosis | settlement | Gate C observer | record_damaged | bounded candidate projection | paper contract section 16.6 | proof-only |
| c4 | fixed generator objective G | remains_owner_of | objective unique denominator | proposed diagnostic | 1.0 | objective_producer | metric_denominator | proof | ranking-diagnosis | objective | sampled ambiguity builder | build_objectives | unchanged objective terminals | paper contract section 16.6 | proof-only |
| c5 | diagnostic evidence | classifies | first shared loss mechanism | proposed diagnostic | 0.9 | observer | diagnosis | proof | ranking-diagnosis | metrics | Gate C observer | bounded diagnostics | objective projection or wave gap | paper contract section 16.6 | proof-only |
| c6 | ranking diagnostic | does_not_call | installed runtime | proposed diagnostic | 1.0 | proof_observer | runtime | isolation | ranking-diagnosis | production | lexical compiler feature | proof CLI | no runtime mutation | paper contract section 16.6 | production |

## notes

- `G`, `E`, `M`, and `W` are separate owners and must not be substituted for one another.
- Candidate triads add observation fields only; execution, ordering, objective, and authority remain frozen.
- Any proposal to change ranking requires a separate preflight after this diagnostic identifies one shared mechanism.

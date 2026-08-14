# NANDA Triad Worksheet

task_id: l11-phase8i-ranking-diagnosis-v2
domain: code
query: Add bounded proof observation without changing fixed objective, typed admission, settlement order, authority, or installed runtime

## triads

| id | subject | relation | object | evidence | confidence | subject_role | object_role | route | group | layer | owner | entrypoint | output | evidence_path | scope |
|----|---------|----------|--------|----------|------------|--------------|-------------|-------|-------|-------|-------|------------|--------|---------------|-------|
| t1 | fixed heldout reservoir | produces | immutable damaged cases | proof.rs:3046 | 1.0 | proof_source | label_set | proof | heldout-owner | objective | fixed heldout reservoir | prepare_fixed_heldout_cases | immutable damaged cases | src/nanda_wave/lexical_grokking/proof.rs:3046 | proof-only |
| t2 | sampled ambiguity builder | owns | fixed generator objective G | proof.rs:3084 | 1.0 | objective_producer | objective_set | proof | objective-owner | objective | sampled ambiguity builder | populate_sampled_ambiguity | fixed generator objective G | src/nanda_wave/lexical_grokking/proof.rs:3084 | proof-only |
| t3 | L1TypedEditTraversal | owns | broad typed admission E | typed_edit_traversal.rs:348 | 1.0 | admission_producer | candidate_domain | execution | typed-owner | admission | L1TypedEditTraversal | traverse | terminal certificate map | src/nanda_wave/lexical_grokking/typed_edit_traversal.rs:348 | proof-only |
| t4 | phase7d terminal projection | produces | terminal ID vector | typed_edit_traversal.rs:1022 | 1.0 | projection | candidate_ids | execution | projection-owner | adapter | phase7d_terminal_evidence | phase7d_terminal_evidence | terminal ID vector | src/nanda_wave/lexical_grokking/typed_edit_traversal.rs:1022 | proof-only |
| t5 | implicit forward owner | produces | activation coordinates per terminal | quality.rs:388 | 1.0 | evidence_producer | activation_evidence | execution | implicit-owner | reconstruction | implicit forward owner | evaluate_surface | implicit candidates | src/nanda_wave/lexical_grokking/typed_basin/quality.rs:388 | proof-only |
| t6 | runtime settlement owner | produces | reconstruction modes M | settlement.rs:454 | 1.0 | geometry_producer | typed_mode_projection | execution | geometry-owner | geometry | runtime settlement owner | apply_candidate_restoration_geometry | candidate geometry fields | src/nanda_wave/lexical_grokking/runtime/settlement.rs:454 | shared-frozen-code |
| t7 | exact settlement adapter | produces | settled candidate fields W | typed_basin/settlement.rs:411 | 1.0 | ranking_producer | candidate_order | execution | settlement-owner | settlement | exact settlement adapter | settle_exact_case | settled candidate fields W | src/nanda_wave/lexical_grokking/typed_basin/settlement.rs:411 | proof-only |
| t8 | frozen classifier | produces | Winner Tied or Abstain | typed_basin/settlement.rs:550 | 1.0 | authority_producer | readout | authority | authority-owner | readout | frozen classifier | classify | restoration readout | src/nanda_wave/lexical_grokking/typed_basin/settlement.rs:550 | proof-only |
| t9 | Gate C recorder | produces | bounded counters and diagnostics | quality.rs:475 | 1.0 | observer | diagnostic_evidence | observation | observer-owner | metrics | Gate C recorder | record_damaged | bounded counters and diagnostics | src/nanda_wave/lexical_grokking/typed_basin/quality.rs:475 | proof-only |
| t10 | installed Lay | preserves | installed package daemon and IBus | phase8i.md:39 | 1.0 | runtime | protected_artifact | isolation | installed-runtime-owner | production | installed Lay | daemon and IBus | unchanged installed system | docs/l1-l11-typed-basin-implicit-forward-phase8i.md:39 | production |
| t11 | lexical compiler feature | isolates | proof route from installed runtime | mod.rs:33 | 1.0 | proof_boundary | runtime_boundary | isolation | proof-isolation-owner | build | lexical compiler feature | proof CLI | no production reachability | src/nanda_wave/lexical_grokking/mod.rs:33 | proof-only |

## candidate_triads

| id | subject | relation | object | evidence | confidence | subject_role | object_role | route | group | layer | owner | entrypoint | output | evidence_path | scope |
|----|---------|----------|--------|----------|------------|--------------|-------------|-------|-------|-------|-------|------------|--------|---------------|-------|
| c1 | sampled ambiguity builder | owns | unchanged fixed generator objective G | paper contract section 16.6 | 1.0 | objective_producer | objective_set | proof | objective-owner | objective | sampled ambiguity builder | populate_sampled_ambiguity | fixed generator objective G | docs/l1-l11-typed-basin-implicit-forward-phase8i.md | proof-only |
| c2 | phase7d terminal projection | produces | terminal IDs plus bounded certificate observation | paper contract section 16.6 | 0.95 | projection | diagnostic_evidence | observation | projection-owner | adapter | phase7d_terminal_evidence | phase7d_terminal_evidence | terminal IDs plus certificate classes | docs/l1-l11-typed-basin-implicit-forward-phase8i.md | proof-only |
| c3 | implicit forward owner | produces | bounded activation observation per terminal | paper contract section 16.6 | 0.95 | evidence_producer | diagnostic_evidence | observation | implicit-owner | reconstruction | implicit forward owner | evaluate_surface | activation coordinates per terminal | docs/l1-l11-typed-basin-implicit-forward-phase8i.md | proof-only |
| c4 | exact settlement adapter | produces | bounded settled candidate observation W | paper contract section 16.6 | 0.95 | ranking_producer | diagnostic_evidence | observation | settlement-owner | settlement | exact settlement adapter | settle_exact_case | settled candidate fields W | docs/l1-l11-typed-basin-implicit-forward-phase8i.md | proof-only |
| c5 | Gate C recorder | produces | bounded mechanism diagnosis | paper contract section 16.6 | 0.95 | observer | diagnostic_evidence | observation | observer-owner | metrics | Gate C recorder | record_damaged | first shared mechanism rows | docs/l1-l11-typed-basin-implicit-forward-phase8i.md | proof-only |
| c6 | lexical compiler feature | isolates | ranking diagnostic from installed runtime | paper contract section 16.6 | 1.0 | proof_boundary | runtime_boundary | isolation | proof-isolation-owner | build | lexical compiler feature | proof CLI | no production reachability | docs/l1-l11-typed-basin-implicit-forward-phase8i.md | proof-only |

## notes

- Every candidate group has one existing owner and one matching source route.
- The objective owner, admission owner, ranking owner, authority owner, and installed runtime remain unchanged.
- The proposed change adds observation fields only and cannot feed labels back into execution.

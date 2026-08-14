# NANDA Triad Worksheet

task_id: l11-phase8i-ranking-diagnosis-v3
domain: code
query: Verify that bounded diagnostic fields preserve every existing owner, route, functional object, and production isolation boundary

## triads

| id | subject | relation | object | evidence | confidence | subject_role | object_role | route | group | layer | owner | entrypoint | output | evidence_path | scope |
|----|---------|----------|--------|----------|------------|--------------|-------------|-------|-------|-------|-------|------------|--------|---------------|-------|
| t1 | sampled ambiguity builder | owns | fixed generator objective G | proof.rs:3084 | 1.0 | objective_producer | objective_set | proof | objective-owner | objective | sampled ambiguity builder | populate_sampled_ambiguity | fixed generator objective G | src/nanda_wave/lexical_grokking/proof.rs:3084 | proof-only |
| t2 | phase7d terminal projection | produces | terminal ID vector | typed_edit_traversal.rs:1022 | 1.0 | projection | candidate_ids | execution | projection-owner | adapter | phase7d_terminal_evidence | phase7d_terminal_evidence | terminal ID vector | src/nanda_wave/lexical_grokking/typed_edit_traversal.rs:1022 | proof-only |
| t3 | implicit forward owner | produces | activation coordinates per terminal | quality.rs:388 | 1.0 | evidence_producer | activation_evidence | execution | implicit-owner | reconstruction | implicit forward owner | evaluate_surface | activation coordinates per terminal | src/nanda_wave/lexical_grokking/typed_basin/quality.rs:388 | proof-only |
| t4 | runtime settlement owner | produces | reconstruction modes M | settlement.rs:454 | 1.0 | geometry_producer | typed_mode_projection | execution | geometry-owner | geometry | runtime settlement owner | apply_candidate_restoration_geometry | reconstruction modes M | src/nanda_wave/lexical_grokking/runtime/settlement.rs:454 | shared-frozen-code |
| t5 | exact settlement adapter | produces | settled candidate fields W | typed_basin/settlement.rs:411 | 1.0 | ranking_producer | candidate_order | execution | settlement-owner | settlement | exact settlement adapter | settle_exact_case | settled candidate fields W | src/nanda_wave/lexical_grokking/typed_basin/settlement.rs:411 | proof-only |
| t6 | frozen classifier | produces | Winner Tied or Abstain | typed_basin/settlement.rs:550 | 1.0 | authority_producer | readout | authority | authority-owner | readout | frozen classifier | classify | Winner Tied or Abstain | src/nanda_wave/lexical_grokking/typed_basin/settlement.rs:550 | proof-only |
| t7 | Gate C recorder | produces | bounded counters and diagnostics | quality.rs:475 | 1.0 | observer | diagnostic_evidence | observation | observer-owner | metrics | Gate C recorder | record_damaged | bounded counters and diagnostics | src/nanda_wave/lexical_grokking/typed_basin/quality.rs:475 | proof-only |
| t8 | lexical compiler feature | isolates | proof route from installed runtime | mod.rs:33 | 1.0 | proof_boundary | runtime_boundary | isolation | proof-isolation-owner | build | lexical compiler feature | proof CLI | proof route from installed runtime | src/nanda_wave/lexical_grokking/mod.rs:33 | proof-only |

## candidate_triads

| id | subject | relation | object | evidence | confidence | subject_role | object_role | route | group | layer | owner | entrypoint | output | evidence_path | scope |
|----|---------|----------|--------|----------|------------|--------------|-------------|-------|-------|-------|-------|------------|--------|---------------|-------|
| c1 | sampled ambiguity builder | owns | fixed generator objective G | paper contract section 16.6 | 1.0 | objective_producer | objective_set | proof | objective-owner | objective | sampled ambiguity builder | populate_sampled_ambiguity | fixed generator objective G | docs/l1-l11-typed-basin-implicit-forward-phase8i.md | proof-only |
| c2 | phase7d terminal projection | produces | terminal ID vector | paper contract section 16.6 | 1.0 | projection | candidate_ids | execution | projection-owner | adapter | phase7d_terminal_evidence | phase7d_terminal_evidence | terminal ID vector | docs/l1-l11-typed-basin-implicit-forward-phase8i.md | proof-only |
| c3 | implicit forward owner | produces | activation coordinates per terminal | paper contract section 16.6 | 1.0 | evidence_producer | activation_evidence | execution | implicit-owner | reconstruction | implicit forward owner | evaluate_surface | activation coordinates per terminal | docs/l1-l11-typed-basin-implicit-forward-phase8i.md | proof-only |
| c4 | exact settlement adapter | produces | settled candidate fields W | paper contract section 16.6 | 1.0 | ranking_producer | candidate_order | execution | settlement-owner | settlement | exact settlement adapter | settle_exact_case | settled candidate fields W | docs/l1-l11-typed-basin-implicit-forward-phase8i.md | proof-only |
| c5 | Gate C recorder | produces | bounded counters and diagnostics | paper contract section 16.6 | 1.0 | observer | diagnostic_evidence | observation | observer-owner | metrics | Gate C recorder | record_damaged | bounded counters and diagnostics | docs/l1-l11-typed-basin-implicit-forward-phase8i.md | proof-only |
| c6 | lexical compiler feature | isolates | proof route from installed runtime | paper contract section 16.6 | 1.0 | proof_boundary | runtime_boundary | isolation | proof-isolation-owner | build | lexical compiler feature | proof CLI | proof route from installed runtime | docs/l1-l11-typed-basin-implicit-forward-phase8i.md | proof-only |

## notes

- Candidate triads assert the preserved functional route, not the internal JSON field list.
- The scoped implementation may enrich existing bounded diagnostics but cannot change any object named above.
- Geometry and authority have source triads only because their bytes and behavior remain frozen.

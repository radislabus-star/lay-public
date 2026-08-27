# NANDA Triad Worksheet

task_id: m3-actual-owner-consequence-v1-route-c1
domain: general
query: Does the proposed exact-peak lane preserve each downstream decision owner as a distinct role?

## triads

| id | subject | relation | object | evidence | confidence | subject_role | object_role | route | group |
|----|---------|----------|--------|----------|------------|--------------|-------------|-------|-------|
| s1 | exact-peak lane | passes_born_candidates_to | Productive V90 | consequence candidate semantics graph | 1.0 | discovery producer | candidate rank owner | candidate-flow | candidate-flow |
| s2 | Productive V90 | passes_ranked_material_to | common L3 | architecture V9-D route | 1.0 | candidate rank owner | contextual selection owner | rank-flow | rank-flow |
| s3 | common L3 | passes_selected_candidate_to | DecisionCore | architecture V9-D route | 1.0 | contextual selection owner | authorization owner | selection-flow | selection-flow |
| s4 | DecisionCore | passes_authorized_plan_to | verifier | architecture V9-D route | 1.0 | authorization owner | safety owner | authorization-flow | authorization-flow |
| s5 | exact V13 peak | cannot_directly_authorize | mutation | consequence candidate semantics veto | 1.0 | discovered Born candidate | forbidden mutation authority | authority-veto | authority-veto |

## candidate_triads

| id | subject | relation | object | evidence | confidence | subject_role | object_role | route | group |
|----|---------|----------|--------|----------|------------|--------------|-------------|-------|-------|
| c1 | exact-peak lane | passes_born_candidates_to | Productive V90 | selected route | 1.0 | discovery producer | candidate rank owner | candidate-flow | candidate-flow |
| c2 | Productive V90 | passes_ranked_material_to | common L3 | preserved route | 1.0 | candidate rank owner | contextual selection owner | rank-flow | rank-flow |
| c3 | common L3 | passes_selected_candidate_to | DecisionCore | preserved route | 1.0 | contextual selection owner | authorization owner | selection-flow | selection-flow |
| c4 | DecisionCore | passes_authorized_plan_to | verifier | preserved route | 1.0 | authorization owner | safety owner | authorization-flow | authorization-flow |
| c5 | exact V13 peak | cannot_directly_authorize | mutation | explicit veto | 1.0 | discovered Born candidate | forbidden mutation authority | authority-veto | authority-veto |

## notes

- Each owner is checked in its own route group; no helper receives two decision roles.

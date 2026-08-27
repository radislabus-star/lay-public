# NANDA Triad Worksheet

task_id: m3-actual-owner-consequence-v1-route-c1-v2
domain: general
query: Does the proposed exact-peak lane preserve each downstream decision owner as a distinct role?

## triads

| id | subject | relation | object | evidence | confidence | subject_role | object_role | route | group |
|----|---------|----------|--------|----------|------------|--------------|-------------|-------|-------|
| s1 | exact-peak lane | passes_born_candidates_to | Productive V90 | consequence.md:119-125 | 1.0 | discovery producer | candidate rank owner | candidate-flow | candidate-flow |
| s2 | Productive V90 | passes_ranked_material_to | common L3 | architecture.md:978-980 | 1.0 | candidate rank owner | contextual selection owner | rank-flow | rank-flow |
| s3 | common L3 | passes_selected_candidate_to | DecisionCore | architecture.md:980-981 | 1.0 | contextual selection owner | authorization owner | selection-flow | selection-flow |
| s4 | DecisionCore | passes_authorized_plan_to | verifier | architecture.md:981-982 | 1.0 | authorization owner | safety owner | authorization-flow | authorization-flow |
| s5 | exact V13 peak | cannot_directly_authorize | mutation | consequence.md:121-127 | 1.0 | discovered Born candidate | forbidden mutation authority | authority-veto | authority-veto |

## candidate_triads

| id | subject | relation | object | evidence | confidence | subject_role | object_role | route | group |
|----|---------|----------|--------|----------|------------|--------------|-------------|-------|-------|
| c1 | exact-peak lane | passes_born_candidates_to | Productive V90 | candidate route A | 1.0 | discovery producer | candidate rank owner | candidate-flow | candidate-flow |
| c2 | Productive V90 | passes_ranked_material_to | common L3 | candidate route B | 1.0 | candidate rank owner | contextual selection owner | rank-flow | rank-flow |
| c3 | common L3 | passes_selected_candidate_to | DecisionCore | candidate route C | 1.0 | contextual selection owner | authorization owner | selection-flow | selection-flow |
| c4 | DecisionCore | passes_authorized_plan_to | verifier | candidate route D | 1.0 | authorization owner | safety owner | authorization-flow | authorization-flow |
| c5 | exact V13 peak | cannot_directly_authorize | mutation | candidate veto E | 1.0 | discovered Born candidate | forbidden mutation authority | authority-veto | authority-veto |

## notes

- V2 binds each owner transfer to its own source span and route group.

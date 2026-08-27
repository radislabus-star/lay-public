# NANDA Triad Worksheet

task_id: d5-t4-offline-forensic-diagnostic-authority-v1
domain: general
query: Is the D5 T4 offline forensic diagnostic and authority boundary structurally closed?

## triads

| id | subject | relation | object | evidence | confidence | subject_role | object_role | route | group |
|----|---------|----------|--------|----------|------------|--------------|-------------|-------|-------|
| s1 | throttle owner | pairs_by | exact CPU and monotonic raw-record timestamp | throttle pairing contract | 1.0 | throttle owner | coverage diagnostic | throttle | forensic-throttle |
| s2 | diagnostic owner | labels_only | attribution counts perturbation and period projections as retrospective | forensic diagnostic boundary | 1.0 | diagnostic owner | non-authoritative result | diagnostic | forensic-diagnostic |
| s3 | effect owner | forbids | network remote marker perf subject Cargo rustc and runtime effects | forbidden effects table | 1.0 | effect owner | zero-effect route | effects | forensic-effects |
| s4 | receipt owner | seals | complete local evidence with SHA256SUMS | forensic publication contract | 1.0 | receipt owner | immutable audit | receipt | forensic-receipt |
| s5 | claim owner | limits | complete forensic receipt to separate D6 paper input only | decision boundary | 1.0 | claim owner | future authority boundary | boundary | forensic-boundary |

## candidate_triads

| id | subject | relation | object | evidence | confidence | subject_role | object_role | route | group |
|----|---------|----------|--------|----------|------------|--------------|-------------|-------|-------|
| c1 | throttle owner | pairs_by | exact CPU and monotonic raw-record timestamp | reviewed throttle contract | 1.0 | throttle owner | coverage diagnostic | throttle | forensic-throttle |
| c2 | diagnostic owner | labels_only | attribution counts perturbation and period projections as retrospective | reviewed diagnostic boundary | 1.0 | diagnostic owner | non-authoritative result | diagnostic | forensic-diagnostic |
| c3 | effect owner | forbids | network remote marker perf subject Cargo rustc and runtime effects | reviewed zero-effect route | 1.0 | effect owner | zero-effect route | effects | forensic-effects |
| c4 | receipt owner | seals | complete local evidence with SHA256SUMS | reviewed publication closure | 1.0 | receipt owner | immutable audit | receipt | forensic-receipt |
| c5 | claim owner | limits | complete forensic receipt to separate D6 paper input only | reviewed claim boundary | 1.0 | claim owner | future authority boundary | boundary | forensic-boundary |

## notes

- This route grants structural coherence only.
- It cannot change D5 or admit execution.

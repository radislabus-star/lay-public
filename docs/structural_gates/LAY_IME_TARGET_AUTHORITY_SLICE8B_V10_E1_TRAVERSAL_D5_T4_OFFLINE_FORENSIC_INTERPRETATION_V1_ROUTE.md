# NANDA Triad Worksheet

task_id: d5-t4-offline-forensic-interpretation-v1
domain: general
query: Is the retrospective D5 T4 forensic route structurally closed without changing historical authority?

## triads

| id | subject | relation | object | evidence | confidence | subject_role | object_role | route | group |
|----|---------|----------|--------|----------|------------|--------------|-------------|-------|-------|
| s1 | history owner | preserves | D5 terminal BLOCKED_PROVENANCE and no retry | sealed D5 terminal receipt | 1.0 | history owner | immutable verdict | history | forensic-history |
| s2 | input owner | pins | exact sealed T4 samples raw records receipt observation perf data and D2 map | forensic paper input table | 1.0 | input owner | immutable evidence | inputs | forensic-inputs |
| s3 | lifecycle owner | reconstructs_from | one libtest parent and twenty direct terminal worker TIDs | FORK COMM EXIT algorithm | 1.0 | lifecycle owner | worker identity | lifecycle | forensic-lifecycle |
| s4 | scope owner | separates | all-sample CPU projection from exact traversal CPU projection | forensic interpretation algorithm | 1.0 | scope owner | estimator scope | scope | forensic-scope |
| s5 | map owner | joins_by | exact D2 mapping unique load bias and normalized sealed range | D2 map and MMAP2 contract | 1.0 | map owner | machine attribution | mapping | forensic-map |
| s6 | throttle owner | pairs_by | exact CPU and monotonic raw-record timestamp | throttle pairing contract | 1.0 | throttle owner | coverage diagnostic | throttle | forensic-throttle |
| s7 | diagnostic owner | labels_only | attribution counts perturbation and period projections as retrospective | forensic diagnostic boundary | 1.0 | diagnostic owner | non-authoritative result | diagnostic | forensic-diagnostic |
| s8 | effect owner | forbids | network remote marker perf subject Cargo rustc and runtime effects | forbidden effects table | 1.0 | effect owner | zero-effect route | effects | forensic-effects |
| s9 | receipt owner | seals | complete local evidence with SHA256SUMS | forensic publication contract | 1.0 | receipt owner | immutable audit | receipt | forensic-receipt |
| s10 | claim owner | limits | complete forensic receipt to separate D6 paper input only | decision boundary | 1.0 | claim owner | future authority boundary | boundary | forensic-boundary |

## candidate_triads

| id | subject | relation | object | evidence | confidence | subject_role | object_role | route | group |
|----|---------|----------|--------|----------|------------|--------------|-------------|-------|-------|
| c1 | history owner | preserves | D5 terminal BLOCKED_PROVENANCE and no retry | reviewed terminal preservation | 1.0 | history owner | immutable verdict | history | forensic-history |
| c2 | input owner | pins | exact sealed T4 samples raw records receipt observation perf data and D2 map | reviewed input closure | 1.0 | input owner | immutable evidence | inputs | forensic-inputs |
| c3 | lifecycle owner | reconstructs_from | one libtest parent and twenty direct terminal worker TIDs | reviewed lifecycle algorithm | 1.0 | lifecycle owner | worker identity | lifecycle | forensic-lifecycle |
| c4 | scope owner | separates | all-sample CPU projection from exact traversal CPU projection | reviewed scope separation | 1.0 | scope owner | estimator scope | scope | forensic-scope |
| c5 | map owner | joins_by | exact D2 mapping unique load bias and normalized sealed range | reviewed mapping closure | 1.0 | map owner | machine attribution | mapping | forensic-map |
| c6 | throttle owner | pairs_by | exact CPU and monotonic raw-record timestamp | reviewed throttle contract | 1.0 | throttle owner | coverage diagnostic | throttle | forensic-throttle |
| c7 | diagnostic owner | labels_only | attribution counts perturbation and period projections as retrospective | reviewed diagnostic boundary | 1.0 | diagnostic owner | non-authoritative result | diagnostic | forensic-diagnostic |
| c8 | effect owner | forbids | network remote marker perf subject Cargo rustc and runtime effects | reviewed zero-effect route | 1.0 | effect owner | zero-effect route | effects | forensic-effects |
| c9 | receipt owner | seals | complete local evidence with SHA256SUMS | reviewed publication closure | 1.0 | receipt owner | immutable audit | receipt | forensic-receipt |
| c10 | claim owner | limits | complete forensic receipt to separate D6 paper input only | reviewed claim boundary | 1.0 | claim owner | future authority boundary | boundary | forensic-boundary |

## notes

- Structural coherence grants no scientific, execution, optimization, build,
  integration, installation, restart, deployment, or runtime authority.
- The route is retrospective and cannot supersede D5.

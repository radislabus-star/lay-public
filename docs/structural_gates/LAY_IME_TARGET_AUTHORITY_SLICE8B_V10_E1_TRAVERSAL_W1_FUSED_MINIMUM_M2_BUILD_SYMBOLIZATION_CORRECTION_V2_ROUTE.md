# NANDA Triad Worksheet

task_id: w1-fused-minimum-m2-build-symbolization-correction-v2
domain: general
query: Does the M2 build correction reconcile auditable machine ownership with the frozen physical comparison?

## triads

| id | subject | relation | object | evidence | confidence | subject_role | object_role | route | group |
|----|---------|----------|--------|----------|------------|--------------|-------------|-------|-------|
| s1 | sealed Cargo release profile | strips | symbols and DWARF | Cargo.toml strip true and D7 environment has no overrides | 1.0 | baseline owner | discovered build defect | evidence | symbol-evidence |
| s2 | M2 symbolization exception | retains | debug level 2 and strip none | exact D2 build environment precedent | 1.0 | build owner | auditable B G I ownership | build | symbol-build |
| s3 | M2 build | preserves | opt level LTO codegen panic target and dependencies | correction exact unchanged list | 1.0 | build owner | physical optimization context | build | performance-build |
| s4 | B physical pair | decides | symbolized-build comparability | frozen D7 traversal and instruction gates | 1.0 | measurement owner | perturbation validity | measurement | physical-validity |
| s5 | build auditor | blocks | missing DWARF symbols or folded machine owners | corrected read-only audit contract | 1.0 | audit owner | build verdict | audit | build-audit |

## candidate_triads

| id | subject | relation | object | evidence | confidence | subject_role | object_role | route | group |
|----|---------|----------|--------|----------|------------|--------------|-------------|-------|-------|
| c1 | sealed Cargo release profile | strips | symbols and DWARF | measured Cargo and D7 provenance | 1.0 | baseline owner | discovered build defect | evidence | symbol-evidence |
| c2 | M2 symbolization exception | retains | debug level 2 and strip none | frozen environment values | 1.0 | build owner | auditable B G I ownership | build | symbol-build |
| c3 | M2 build | preserves | opt level LTO codegen panic target and dependencies | correction exact unchanged list | 1.0 | build owner | physical optimization context | build | performance-build |
| c4 | B physical pair | decides | symbolized-build comparability | preregistered perturbation gates | 1.0 | measurement owner | perturbation validity | measurement | physical-validity |
| c5 | build auditor | blocks | missing DWARF symbols or folded machine owners | corrected read-only audit contract | 1.0 | audit owner | build verdict | audit | build-audit |

## notes

- The correction changes evidence retention only, not optimization or candidate semantics.
- No build is admitted by this worksheet.

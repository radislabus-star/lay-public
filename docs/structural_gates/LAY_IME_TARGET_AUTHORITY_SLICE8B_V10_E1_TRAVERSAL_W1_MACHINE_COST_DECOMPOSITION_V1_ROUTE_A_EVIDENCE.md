# NANDA Triad Worksheet

task_id: w1-machine-cost-decomposition-evidence-v1
domain: general
query: Is the sealed W1 existing-evidence machine-cost decomposition structurally closed?

## triads

| id | subject | relation | object | evidence | confidence | subject_role | object_role | route | group |
|----|---------|----------|--------|----------|------------|--------------|-------------|-------|-------|
| s1 | W1 offline auditor | pins | D7 aggregate baseline and D4 single-worker scientific receipts | exact SHA size mode table | 1.0 | evidence owner | immutable predecessor evidence | inputs | w1-evidence |
| s2 | W1 offline auditor | joins | D4 normalized IPs to exact D2 Build ID and audited machine ranges | D4 terminal and D2 map audit | 1.0 | attribution owner | machine-range ownership | attribution | w1-evidence |
| s3 | W1 offline auditor | reproduces | 66543 accepted samples and every bucket and sub-bucket count | sealed perf-script rows | 1.0 | reduction owner | scientific sample closure | reduction | w1-evidence |
| s4 | W1 offline auditor | decodes | exact D2 machine instructions with read-only objdump | sealed ELF and retained command evidence | 1.0 | disassembly owner | instruction-start closure | machine | w1-evidence |
| s5 | W1 offline auditor | localizes | separate minimum reduction machine block | exact source line machine ranges and 20839 samples | 1.0 | mechanism owner | removable-work hypothesis | synthesis | w1-evidence |

## candidate_triads

| id | subject | relation | object | evidence | confidence | subject_role | object_role | route | group |
|----|---------|----------|--------|----------|------------|--------------|-------------|-------|-------|
| c1 | W1 offline auditor | pins | D7 aggregate baseline and D4 single-worker scientific receipts | independently checked identities | 1.0 | evidence owner | immutable predecessor evidence | inputs | w1-evidence |
| c2 | W1 offline auditor | joins | D4 normalized IPs to exact D2 Build ID and audited machine ranges | independently reparsed map join | 1.0 | attribution owner | machine-range ownership | attribution | w1-evidence |
| c3 | W1 offline auditor | reproduces | 66543 accepted samples and every bucket and sub-bucket count | independently reduced sample stream | 1.0 | reduction owner | scientific sample closure | reduction | w1-evidence |
| c4 | W1 offline auditor | decodes | exact D2 machine instructions with read-only objdump | independently checked instruction starts | 1.0 | disassembly owner | instruction-start closure | machine | w1-evidence |
| c5 | W1 offline auditor | localizes | separate minimum reduction machine block | independently matched source semantics and ranges | 1.0 | mechanism owner | removable-work hypothesis | synthesis | w1-evidence |

## notes

- This route uses no new experiment and grants no implementation authority.
- D4 attribution remains bound to D2 ELF; D7 supplies aggregate counters only.


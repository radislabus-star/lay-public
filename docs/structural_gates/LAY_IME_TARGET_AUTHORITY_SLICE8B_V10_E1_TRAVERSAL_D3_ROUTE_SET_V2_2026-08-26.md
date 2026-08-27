# NANDA Triad Worksheet

task_id: d3-estimator-recovery-v2
domain: general
query: Is the D3 single-worker estimator recovery structurally closed by route?

## triads

| id | subject | relation | object | evidence | confidence | subject_role | object_role | route | group |
|----|---------|----------|--------|----------|------------|--------------|-------------|-------|-------|
| s1 | history owner | preserves_terminal | D2 BLOCKED_PROVENANCE and retired old T markers | sealed D2 terminal audit | 1.0 | history owner | immutable history | history | d3-history |
| s2 | reuse owner | admits_only | exact audited D2 ELF map parity and U V evidence | D3 V1 reuse contract | 1.0 | reuse owner | predecessor closure | reuse | d3-reuse |
| s3 | envelope owner | stages_then_pins | CPU6 input loading then CPU0 traversal | D3 V1 launch contract | 1.0 | envelope owner | process affinity | envelope | d3-envelope |
| s4 | U2 denominator owner | measures | twenty rounds times 25145756 edges | D3 V1 U2 contract | 1.0 | U2 denominator owner | measured denominator | u2 | d3-u2-denominator |
| s5 | T2 denominator owner | samples | twenty-one rounds times 25145756 edges | D3 V1 T2 contract | 1.0 | T2 denominator owner | sampled denominator | t2 | d3-t2-denominator |
| s6 | sampling owner | executes_only | task-clock user period 200000 on T2-SINGLE | D3 V1 command graph | 1.0 | sampling owner | perf evidence | t2 | d3-sampling |
| s7 | one-shot owner | orders | U2 PASS before T2 marker consumption | D3 V1 state machine | 1.0 | one-shot owner | route authority | control | d3-one-shot |
| s8 | decision owner | limits | D3 PASS to a separate multiworker paper | D3 V1 claim boundary | 1.0 | decision owner | next authority | decision | d3-decision |

## candidate_triads

| id | subject | relation | object | evidence | confidence | subject_role | object_role | route | group |
|----|---------|----------|--------|----------|------------|--------------|-------------|-------|-------|
| c1 | history owner | preserves_terminal | D2 BLOCKED_PROVENANCE and retired old T markers | reviewed D3 history route | 1.0 | history owner | immutable history | history | d3-history |
| c2 | reuse owner | admits_only | exact audited D2 ELF map parity and U V evidence | reviewed D3 reuse route | 1.0 | reuse owner | predecessor closure | reuse | d3-reuse |
| c3 | envelope owner | stages_then_pins | CPU6 input loading then CPU0 traversal | reviewed D3 launch route | 1.0 | envelope owner | process affinity | envelope | d3-envelope |
| c4 | U2 denominator owner | measures | twenty rounds times 25145756 edges | reviewed D3 U2 denominator | 1.0 | U2 denominator owner | measured denominator | u2 | d3-u2-denominator |
| c5 | T2 denominator owner | samples | twenty-one rounds times 25145756 edges | reviewed D3 T2 denominator | 1.0 | T2 denominator owner | sampled denominator | t2 | d3-t2-denominator |
| c6 | sampling owner | executes_only | task-clock user period 200000 on T2-SINGLE | reviewed D3 sampling route | 1.0 | sampling owner | perf evidence | t2 | d3-sampling |
| c7 | one-shot owner | orders | U2 PASS before T2 marker consumption | reviewed D3 control route | 1.0 | one-shot owner | route authority | control | d3-one-shot |
| c8 | decision owner | limits | D3 PASS to a separate multiworker paper | reviewed D3 decision route | 1.0 | decision owner | next authority | decision | d3-decision |

## notes

- Each group is checked separately; aggregate size is not promoted as PASS.
- These checks establish structural coherence only. The implementation
  preflight owns code admission.


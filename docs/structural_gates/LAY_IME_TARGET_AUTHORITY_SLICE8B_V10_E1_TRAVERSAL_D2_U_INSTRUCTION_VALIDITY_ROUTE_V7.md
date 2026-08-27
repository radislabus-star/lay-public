# NANDA Task Worksheet

task_id: lay-v10-e1-traversal-d2-u-instruction-validity-v7
domain: code
query: Validate the D2 split CPU-time and clock-free instruction validity overlay

## triads

| id | subject | relation | object | evidence | confidence | subject_role | object_role | route | group |
|----|----|----|----|----|----|----|----|----|----|
| s1 | denominator evidence owner | establishes | sealed fixed and reversed D1 G0 instruction baselines | D1 decision and PMU correction V2 | 1.000 | evidence owner | evidence | baseline | baseline-owner |
| s2 | component validity owner | measures_only | U-route traversal thread CPU per examined edge | exact D1 component route with existing clocks | 1.000 | validity owner | measurement | cpu-validity | cpu-owner |
| s3 | instruction validity owner | measures_only | V-route aggregate instructions per request | exact clock-free d1_run_twenty_pmu G0 route | 1.000 | validity owner | measurement | instruction-validity | instruction-owner |
| s4 | PMU protocol owner | preserves | exact four-event FIFO-controlled D1 G0 context | sealed fixed and reversed PMU wrapper argv | 1.000 | protocol owner | protocol | pmu-context | protocol-owner |
| s5 | aggregation owner | applies_only | hybrid runtime weighted aggregate interpretation | sealed D1 PMU correction V2 formula | 1.000 | aggregation owner | interpretation | aggregation | aggregation-owner |
| s6 | sequence owner | gates_before | every T sampling route on all U and V validity PASS | V7 corrected execution order | 1.000 | sequence owner | authority boundary | sequencing | sequence-owner |
| s7 | claim-boundary owner | forbids | instruction IP attribution and instruction-heavy or stall claims | secondary-gap ceiling and V7 aggregate-only scope | 1.000 | boundary owner | veto | claim-boundary | boundary-owner |
| s8 | implementation-preflight owner | gates | controller creation and every D2 executable effect | V7 authority boundary | 1.000 | preflight owner | authority boundary | preflight | preflight-owner |

## candidate_triads

| id | subject | relation | object | evidence | confidence | subject_role | object_role | route | group |
|----|----|----|----|----|----|----|----|----|----|
| c1 | denominator evidence owner | establishes | sealed fixed and reversed D1 G0 instruction baselines | D1 decision and PMU correction V2 | 1.000 | evidence owner | evidence | baseline | baseline-owner |
| c2 | component validity owner | measures_only | U-route traversal thread CPU per examined edge | exact D1 component route with existing clocks | 1.000 | validity owner | measurement | cpu-validity | cpu-owner |
| c3 | instruction validity owner | measures_only | V-route aggregate instructions per request | exact clock-free d1_run_twenty_pmu G0 route | 1.000 | validity owner | measurement | instruction-validity | instruction-owner |
| c4 | PMU protocol owner | preserves | exact four-event FIFO-controlled D1 G0 context | sealed fixed and reversed PMU wrapper argv | 1.000 | protocol owner | protocol | pmu-context | protocol-owner |
| c5 | aggregation owner | applies_only | hybrid runtime weighted aggregate interpretation | sealed D1 PMU correction V2 formula | 1.000 | aggregation owner | interpretation | aggregation | aggregation-owner |
| c6 | sequence owner | gates_before | every T sampling route on all U and V validity PASS | V7 corrected execution order | 1.000 | sequence owner | authority boundary | sequencing | sequence-owner |
| c7 | claim-boundary owner | forbids | instruction IP attribution and instruction-heavy or stall claims | secondary-gap ceiling and V7 aggregate-only scope | 1.000 | boundary owner | veto | claim-boundary | boundary-owner |
| c8 | implementation-preflight owner | gates | controller creation and every D2 executable effect | V7 authority boundary | 1.000 | preflight owner | authority boundary | preflight | preflight-owner |

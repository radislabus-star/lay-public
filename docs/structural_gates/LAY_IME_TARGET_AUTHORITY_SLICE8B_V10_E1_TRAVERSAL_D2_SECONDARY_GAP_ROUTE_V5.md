# NANDA Task Worksheet

task_id: lay-v10-e1-traversal-d2-secondary-gap-v5
domain: code
query: Validate the D2 primary-only secondary-gap execution overlay

## triads

| id | subject | relation | object | evidence | confidence | subject_role | object_role | route | group |
|----|----|----|----|----|----|----|----|----|----|
| s1 | precise evidence owner | establishes | I-CORE required IP identity is unusable | sealed precise V3 receipt and 79 unknown IP samples | 1.000 | evidence owner | evidence | capability | secondary-gap |
| s2 | sequence owner | preserves_unconsumed | I-ATOM marker after I-CORE failure | sealed marker ledger | 1.000 | sequence owner | state | sequencing | secondary-gap |
| s3 | D2 decision owner | already_defines | D2 attribution with secondary gap | reviewed D2 V4 decision | 1.000 | decision owner | decision | decision | secondary-gap |
| s4 | correction owner | permits_only | creation of primary-only final preflight | V5 sequencing overlay | 1.000 | correction owner | authority boundary | admission | secondary-gap |
| s5 | primary sampling owner | excludes | every precise and substitute event route | primary-only route contract | 1.000 | sampling owner | veto | sampling | secondary-gap |
| s6 | attribution owner | caps | result at D2 attribution with secondary gap | V5 hard ceiling | 1.000 | attribution owner | decision | attribution | secondary-gap |
| s7 | implementation-preflight owner | gates | controller creation before any D2 code | V5 authority boundary | 1.000 | preflight owner | authority boundary | preflight | secondary-gap |

## candidate_triads

| id | subject | relation | object | evidence | confidence | subject_role | object_role | route | group |
|----|----|----|----|----|----|----|----|----|----|
| c1 | precise evidence owner | establishes | I-CORE required IP identity is unusable | sealed precise V3 receipt and 79 unknown IP samples | 1.000 | evidence owner | evidence | capability | secondary-gap |
| c2 | sequence owner | preserves_unconsumed | I-ATOM marker after I-CORE failure | sealed marker ledger | 1.000 | sequence owner | state | sequencing | secondary-gap |
| c3 | D2 decision owner | already_defines | D2 attribution with secondary gap | reviewed D2 V4 decision | 1.000 | decision owner | decision | decision | secondary-gap |
| c4 | correction owner | permits_only | creation of primary-only final preflight | V5 sequencing overlay | 1.000 | correction owner | authority boundary | admission | secondary-gap |
| c5 | primary sampling owner | excludes | every precise and substitute event route | primary-only route contract | 1.000 | sampling owner | veto | sampling | secondary-gap |
| c6 | attribution owner | caps | result at D2 attribution with secondary gap | V5 hard ceiling | 1.000 | attribution owner | decision | attribution | secondary-gap |
| c7 | implementation-preflight owner | gates | controller creation before any D2 code | V5 authority boundary | 1.000 | preflight owner | authority boundary | preflight | secondary-gap |

# NANDA Triad Worksheet

task_id: m3-admission-substage-v10r4-ssh-stdin-transport-correction-v1
domain: general
query: Does V10R4 repair only the pre-namespace SSH observer transport while preserving the V10R3 TRACE-only scientific and one-shot contracts?

## triads

| id | subject | relation | object | evidence | confidence | subject_role | object_role | route | group |
|----|---------|----------|--------|----------|------------|--------------|-------------|-------|-------|
| s1 | V10R3 namespace | remains | immutable BLOCKED_PROVENANCE history | correction-v1.md:14-55 | 1.0 | historical execution evidence | terminal predecessor | history | history |
| s2 | V10R3 failure | occurred_before | remote namespace marker or subject | correction-v1.md:35-55 | 1.0 | transport failure evidence | scientific execution boundary | diagnosis | diagnosis |
| s3 | V10R4 observer | transports_via | exact Python stdin bytes | correction-v1.md:57-75 | 1.0 | read-only observer producer | remote Python interpreter | transport | transport |
| s4 | transport admission | precedes | every V10R4 remote mutation | correction-v1.md:77-96 | 1.0 | read-only admission | state authority | admission | admission |
| s5 | V10R4 namespace | does_not_reuse | V10R3 task transaction or marker | correction-v1.md:98-117 | 1.0 | fresh trace route | retired failed authority | namespace | namespace |
| s6 | V10R4 | preserves | V10R3 quiet marker TRACE and terminal contracts | correction-v1.md:119-153 | 1.0 | transport repair | frozen scientific route | preservation | preservation |
| s7 | V10R4 | excludes | BUILD Cargo rustc perf PMU and runtime mutation | correction-v1.md:145-153 | 1.0 | trace-only controller graph | forbidden execution | graph-veto | graph-veto |
| s8 | V10R4 paper | cannot_grant | execution optimization production or deployment authority | correction-v1.md:178-183 | 1.0 | design evidence | forbidden authority | claim-boundary | claim-boundary |

## candidate_triads

| id | subject | relation | object | evidence | confidence | subject_role | object_role | route | group |
|----|---------|----------|--------|----------|------------|--------------|-------------|-------|-------|
| c1 | V10R3 namespace | remains | immutable BLOCKED_PROVENANCE history | exact implementation journal and diagnosis hashes | 1.0 | historical execution evidence | terminal predecessor | history | history |
| c2 | V10R3 failure | occurred_before | remote namespace marker or subject | independent read-only remote absent projection | 1.0 | transport failure evidence | scientific execution boundary | diagnosis | diagnosis |
| c3 | V10R4 observer | transports_via | exact Python stdin bytes | pinned argv stdin and nonce parity test | 1.0 | read-only observer producer | remote Python interpreter | transport | transport |
| c4 | transport admission | precedes | every V10R4 remote mutation | journal order and remote absence audit | 1.0 | read-only admission | state authority | admission | admission |
| c5 | V10R4 namespace | does_not_reuse | V10R3 task transaction or marker | distinct task transaction paths and marker payload | 1.0 | fresh trace route | retired failed authority | namespace | namespace |
| c6 | V10R4 | preserves | V10R3 quiet marker TRACE and terminal contracts | exact registry argv environment parser and threshold parity | 1.0 | transport repair | frozen scientific route | preservation | preservation |
| c7 | V10R4 | excludes | BUILD Cargo rustc perf PMU and runtime mutation | reachable command graph audit | 1.0 | trace-only controller graph | forbidden execution | graph-veto | graph-veto |
| c8 | V10R4 paper | cannot_grant | execution optimization production or deployment authority | implementation preflight and terminal claim boundary | 1.0 | design evidence | forbidden authority | claim-boundary | claim-boundary |

## notes

- Structural PASS is coherence only and does not admit code or execution.
- The transport probe is read-only and creates no scientific marker.
- V10R3 remains terminal and is never retried.

# DAFSA Typed View M3 Admission-Substage V10R3 Sealed-ELF Quiet Recovery V1

Date: 2026-08-27

## Scope

V10R3 repairs only the pre-TRACE quiet-host admission lifecycle. It reuses the
exact independently audited V10R2 ELF and exact V10R2 bootstrap inputs. It
performs no Cargo invocation, rustc compilation, source reconstruction, ELF
rewrite, perf/PMU action, production mutation, daemon restart, or runtime
authority change.

## Immutable V10R2 History

V10R2 remains terminal `BLOCKED_PROVENANCE`:

```text
task_id
  slice8b-v10-e1-traversal-w1-dafsa-typed-view-m3-admission-substage-v10r2-20260827

transaction_id
  fe5b4741c5d5711b48f356569f3be32a87142edd78979edf0aeb72a9616de7e6

execution journal SHA256SUMS
  5bf039fa3555c8ed7bb81fd6a1b7703ced12e235e4c53590ae872d64621e3aed

build audit
  edf6a15846f5918c2524299b60d33be41f02fe6ec56e21e04ba3c72cb3626706

controller failure
  3d861592d043429deb1f66c983ba2f6ba9f4b86dce596965b434893b823bef0f
```

The BUILD route passed. The quiet auditor then rejected one 5-second window
because CPU0 idle was below 95 percent. It did not retain the failing ratio.
The V10R2 TRACE marker remains unconsumed but is retired with that terminal
namespace. V10R2 is never resumed or retried and its marker is never moved,
copied, or recreated.

## Reused Sealed ELF

The sole executable admitted to V10R3 is:

```text
source task          V10R2
ELF SHA-256          0378514225ccec3cadbcfedd21ec77db66518a5eb6789f9acd83525ccf009696
size                 320,986,144 B
mode                 0555
Build ID             9e2e7c1fef9272f87c14876d7194609df6ac948d
.text SHA-256        64fcef95d1f4635fd58bce3be601d50f3dfd9036c406ca51d4e9e67c115d04c6
V10R2 build audit    V10R2_BUILD_AUDIT_PASS_TRACE_ADMITTED
previous executions  0
```

The executable is read and executed in place. It is not copied, relinked,
stripped, patched, rebuilt, or renamed. The exact V10R2 bootstrap input tree is
also reused read-only.

## Fresh Namespace

```text
task_id
  slice8b-v10-e1-traversal-w1-dafsa-typed-view-m3-admission-substage-v10r3-20260827

transaction_id
  8fa454ad944ce8ba9e6256bc55bdf8ff8231cf26bac7eaab591ad8564515436c

routes
  TRACE-REUSE

marker
  trace.available
```

V10R3 has a new local journal, remote parent, state tree, route lock,
controller identities, and marker payload. It has no BUILD route or build
marker.

## Pre-Marker Quiet Closure

The namespace bootstrap creates no marker. An independent auditor then runs one
bounded readiness action:

```text
window length                    5 seconds
CPU0 idle per accepted window    >= 0.95
all-CPU idle per accepted window >= 0.90
required consecutive windows     3
maximum windows                  120
maximum observation duration     10 minutes
conflicting process count        0 in every window
thermal throttle drift           0 over accepted sequence
runtime projection drift         0
```

Every attempted window is retained in the immutable quiet receipt. A failing
window resets the consecutive counter; it does not execute or consume a
scientific route. If three consecutive windows are not obtained in the bounded
schedule, V10R3 terminates `BLOCKED_QUIET_BEFORE_MARKER` with zero markers and
zero subject executions.

Only an immutable quiet PASS receipt admits creation of the sole
`trace.available` marker. The remote controller consumes that marker atomically
immediately before starting the subject.

## Reused Scientific Contract

The scientific route is byte- and argument-identical to V10R2:

```text
subject              m3_end_to_end_physical_proof
CPU                  0
warmup               382 requests
measured schedule    F/R/F/R, 1,528 requests
V9 rows              exactly 1,910
V10 rows             exactly 1,910
stage registry       36
action registry      4
reason registry      43
candidate parity     exact
certificate parity   exact
perf / PMU           forbidden
```

The existing terminal failure priority and scientific gates remain unchanged.
No result admits production activation or an optimization edit by itself.

## State Machine

```text
V10R2_BLOCKED_PROVENANCE (immutable)
        |
        v
V10R3 structural + implementation preflight
        |
        v
fresh TRACE-only controllers verified unrun
        |
        v
live provenance admission
        |
        v
markerless reuse bootstrap -> independent audit
        |
        v
bounded consecutive quiet closure
        | FAIL -> BLOCKED_QUIET_BEFORE_MARKER
        v
create trace.available
        |
        v
consume marker -> ONE direct sealed-ELF subject
        |
        v
independent terminal audit
        |
        +-- PASS -> ADMISSION_SUBSTAGES_DECOMPOSED
        +-- FAIL -> exact frozen BLOCKED_* verdict
```

No failure permits a second V10R3 quiet action, marker recreation, second
subject, fallback ELF, Cargo build, runtime edit, or production promotion.

## Admission Verdict

This paper admits only V10R3 structural review and controller implementation
preflight. It does not itself create the namespace, marker, or scientific
execution authority.

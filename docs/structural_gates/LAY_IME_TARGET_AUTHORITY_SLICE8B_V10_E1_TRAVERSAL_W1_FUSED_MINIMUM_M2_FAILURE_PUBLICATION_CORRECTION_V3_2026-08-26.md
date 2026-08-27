# W1 Fused-Minimum M2 Failure-Publication Correction V3

Date: 2026-08-26

## Defect

The sealed V2 implementation receipt correctly proves that no M2 execution
occurred, but its future local execution path does not satisfy the already
frozen fault-closed publication invariant.

`execute_once()` has a `try/finally` cleanup boundary but no exception-to-
receipt boundary. A transport, SSH, SCP or auditor exception after a remote
producer consumes a marker can therefore return an error without publishing an
immutable local terminal observation. The four static terminal projections
cover returned producer/auditor verdicts; they do not cover a lost response or
an exception between producer completion and terminal audit.

This contradicts the effective V3 implementation-preflight invariant:

```text
controller exceptions preserve evidence and cannot strand a consumed route
without an immutable blocked observation
```

## Immutable History

The M2 paper, symbolization correction, preflights V1-V3, code-route receipts
V1-V2, V2 implementation receipt and all six sealed V2 implementation sources
remain immutable. Their hashes and modes are historical evidence.

The V2 implementation verdict remains true only in its original scope:

```text
M2_CONTROLLER_VERIFIED_UNRUN
network / remote / Cargo / perf / subject / marker = 0
```

It is superseded as future execution authority by this correction. No M2
execution admission exists, no remote M2 namespace exists and no one-shot
marker has been created or consumed, so the controller can be repaired without
repeating or replacing scientific evidence.

## Effective Repair

A new V3 local controller and matching V3 local auditors must be created under
new source paths. The sealed V2 sources must not be edited. The existing sealed
remote controller and Rust fragment may be reused only by exact hash.

Before the first SSH, SCP or remote command, V3 creates and fsyncs a durable
local execution journal containing:

```text
task_id / transaction_id
V3 controller and auditor hashes
V3 implementation receipt hash
future execution-admission hash
retry_permitted = false
remote execution started = false
```

Before every external action, V3 appends and fsyncs one immutable intent row:

```text
sequence
action_id
expected predecessor
expected remote mutation or read-only scope
associated one-shot route, if any
failure_default = BLOCKED_PROVENANCE
retry_permitted = false
```

Only after a complete response is parsed may it append a separate immutable
completion row. Intent rows are never rewritten.

The minimum covered action set is:

```text
remote cache creation and bootstrap upload
remote bootstrap producer
bootstrap auditor
bootstrap-audit upload
marker creation
build producer
build auditor
build-audit upload
parity producer
each of six physical producers
terminal auditor
```

An intent without a matching completion is an immutable terminal provenance
observation. It does not prove whether the remote mutation happened; it proves
that the route cannot be retried safely.

## Exception Dispatch

Any controller exception after journal creation must publish, when the local
process remains alive, an immutable controller-failure receipt:

```text
verdict                 BLOCKED_PROVENANCE
failed action           exact last intent
remote state            exact known projection or UNKNOWN
marker state            exact known projection or UNKNOWN
Cargo/perf/subject count exact known value or UNKNOWN
retry_permitted         false
runtime authority       unchanged
```

Unknown values must remain `UNKNOWN`; they may not be converted to zero. The
controller must return this blocked verdict rather than raising past its public
entrypoint.

If the process is killed before it can publish that receipt, the already fsynced
uncompleted intent is sufficient to forbid rerun. A later independent read-only
recovery audit may mirror existing remote evidence, but cannot recreate a
marker or resume the scientific sequence.

Returned remote `BLOCKED_*` wrappers continue through the normal terminal
auditor. This correction changes only exception publication and future source
identity; it does not change remote producer semantics, failure priority,
candidate definitions, build command, parity, route order, events,
denominators, thresholds or final scientific verdicts.

## Required Static Proof

Before V3 can be sealed, local fault tests must cover at least:

```text
failure before first remote mutation
lost bootstrap response
lost marker-creation response
lost build response after possible BUILD consumption
build-auditor exception
lost parity response after possible PARITY consumption
lost response for each physical route
terminal-auditor exception
hard-crash model: intent present, completion absent
```

Every case must end with exactly one terminal provenance observation, no
second external action after the failed intent and `retry_permitted=false`.

## Effective Verdict

```text
M2_FAILURE_PUBLICATION_REPAIR_REQUIRED_BEFORE_EXECUTION
```

This admits only a new offline implementation preflight for the V3 local
controller/auditor source set. It does not admit network or remote access,
execution admission, namespace creation, markers, Cargo, rustc, ELF, parity,
perf, PMU, subjects, production edits, installation, restart or deployment.

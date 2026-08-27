# Lay IME Target Authority Slice 8B V10 E1 Traversal D2 Primary-Only Preflight Failure-State Schema Repair V3

Date: 2026-08-25

## Verdict

```text
D2_PRIMARY_ONLY_FAILURE_DISPATCH_SCHEMA_REPRESENTATION_REPAIRED
```

Failure-state correction V2 remains the semantic authority for U, V and T
cause-to-terminal dispatch. Its first implementation-preflight representation
is immutable and blocked:

```text
V2 manifest SHA-256  fce06f65c194f48c5b98255170ffdbad556c37fe265f0e99aef4954eea8bba01
V2 receipt SHA-256   477a2e8b70af9782250c8f84b0e3e78ce013aa2da474d7daee1667fe53e144f0
V2 verdict           BLOCKED_BEFORE_CODE
safe_to_implement    false
blockers              11
```

The V2 representation used nonterminal U/V/T values in
`state_machine.steps[*].failure_state`. The NANDA implementation-preflight v1
schema requires every `failure_state` to be a declared terminal and therefore
reported eight `failure_not_terminal` blockers plus three
`state_has_no_terminal_path` blockers. No controller, Cargo, `perf`, D2 subject
or runtime action occurred.

## Effective V3 Representation

The route observation and gate classification are separate transitions.

```text
consume marker and execute route
  -> <ROUTE>_OBSERVED
  -> guarded closed dispatch
       PASS cause                 -> original <ROUTE>_PASS state
       frozen failure cause       -> exact BLOCKED_* terminal
       unknown/incomplete cause   -> BLOCKED_PROVENANCE
```

The route-execution transition has `failure_state=BLOCKED_PROVENANCE` only for
failure to create and seal an observation envelope at all, such as controller
crash, marker/evidence lifecycle failure or missing dispatch input. Scientific
and capability failures that produce an observation are successful observation
transitions and are classified only by the guarded dispatch transitions.

Every dispatch transition:

```text
executes no subject
opens no PMU event
consumes no additional marker
records all observed violations
uses first-match frozen priority
publishes exactly one terminal verdict or PASS
retains marker and raw evidence
permits no rerun
```

If terminal receipt publication itself fails, its schema-level
`failure_state` is `BLOCKED_PROVENANCE` and the raw observation remains
authoritative evidence of the incomplete publication.

## Frozen Cause Tables

V3 must reproduce correction V2 exactly:

```text
U priority:
  provenance -> thermal -> semantic -> perturbation

V priority:
  provenance -> thermal -> capability -> denominator -> perturbation

T priority:
  provenance -> thermal -> capability -> bucket-map
             -> perturbation -> sample-coverage

unknown / missing / non-unique dispatch:
  BLOCKED_PROVENANCE
```

Each U, V and T route receives its own observed state and guarded dispatch
branches. These branches are controller-internal classification, not new
executable routes. The closed executable graph remains exactly eleven routes.

## Supersession

```text
V1  SUPERSEDED_READY_DISPATCH_DEFECT
V2  BLOCKED_BEFORE_CODE_SCHEMA_REPRESENTATION
V3  NEXT EFFECTIVE IMPLEMENTATION PREFLIGHT
```

V3 must pin V1, correction V2, V2 manifest, V2 receipt and this V3 repair by
exact path, mode, size and SHA-256. The thirteen-section non-dispatch core must
remain at canonical projection SHA-256
`7ec0826f0b9e954803a53b924a42bd008a9e1ff933cb3de51baf33374e24bee3`.

## Claim Boundary

This repair changes state-machine representation only. It admits no controller,
Cargo, rustc, build, bucket map, D2 subject, `perf`, PMU event, attribution,
optimization, runtime integration or deployment by itself. A V3 preflight must
pass first. A positive V3 verdict still admits only controller creation,
controller self-checks and D2-A closure. Cargo remains forbidden until separate
D2-A PASS and prior consumption of `build.available`.

Runtime authority changed: `false`.

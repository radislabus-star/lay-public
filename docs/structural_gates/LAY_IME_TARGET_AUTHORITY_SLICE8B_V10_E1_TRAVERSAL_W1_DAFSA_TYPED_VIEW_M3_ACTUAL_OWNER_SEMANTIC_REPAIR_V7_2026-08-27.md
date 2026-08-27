# M3 Actual-Owner Semantic Repair V7

Date: 2026-08-27

Paper verdict: `M3_ACTUAL_OWNER_SEMANTIC_REPAIR_ADMITTED`

Execution authority: absent until the V7 structural route and a separate V7
implementation/execution preflight pass.

## Exact Diagnosis

V6 completed 764 diagnostic owner requests and reconciled every V5 mismatch:

```text
punctuation_suffix
  36 cases x 2 schedules
  lattice-marker mismatches  72
  emitted-surface mismatches 72
  gate mismatches            72

layout_projection
  case 68 x 2 schedules
  input                       [elt.ob[
  expected                    худеющих
  emitted                     [худеющих[
  emitted-surface mismatches  2

unexplained mismatches        0
```

V5 remains terminal `BLOCKED_SEMANTIC`; V6 remains a diagnostic-only PASS.
Neither route may be retried.

## Repair A: Normalized Observed Identity

The exact composite-lattice lane must compare normalized identities:

```text
normalized_observed = normalize_surface(observed)
exact surface is replacement-visible iff
  normalized_exact_surface != normalized_observed
```

The complete exact material and certificate shadow still retains punctuation
certificates and form refs. Only the replacement lattice excludes a normalized
no-op surface. This preserves candidate/certificate parity while preventing a
punctuation-only observation from becoming a second replacement candidate.

## Repair B: Certificate-Directed Replacement Scope

`KeyboardLayout` is already an exhaustive structured certificate class and is
already projected to `TargetRelationV1::ExactLayout`. A typed exact surface with
that certificate denotes replacement of the complete final non-whitespace
token, including physical punctuation-key characters that participated in the
layout projection.

For an exact-born surface:

```text
KeyboardLayout certificate present
  -> replace complete final non-whitespace token
otherwise
  -> existing punctuation-preserving replace_last_text_word
```

The full-token helper preserves text before the final token and all outer
whitespace. It contains no literal case, word, bracket or source-ID exception.
The decision is made from the retained structured certificate class, not by
re-parsing the canonical key string.

This replacement scope does not grant authority. The candidate remains
`CandidateOrigin::L2Surface`, source `ProductiveL2V90TypedExact` and
`SuggestOnly` unless an independent pre-existing owner admits the same surface.
It does not enter the existing layout authority route.

## Edit Boundary

V7 may edit:

```text
material_frame.rs  expose exact surfaces carrying KeyboardLayout evidence
live.rs            normalize observed identity and select replacement scope
v13_typed_peak.rs  retain diagnostic counters in the final owner receipt
```

`typed_edit_traversal.rs`, the typed search, composite capacity, package inputs,
Productive V90 ranking, common L3, DecisionCore, verifier, cache, bridge and
runtime activation remain unchanged. The normal live wrapper supplies an empty
exact lane, so its replacement path remains the existing byte-for-byte behavior.

## Final Owner Gate

One fresh owner proof may run after V7 preflight. It must retain the V5 identity
projection and require:

```text
sidecar identity projection          PASS
382 forward + 382 reversed           complete
candidate/certificate mismatches     0 / 0
schedule/completeness mismatches      0 / 0
lattice marker mismatches             0
emitted surface mismatches            0
gate mismatches                       0
capacity/collision/adapter errors      0
normal empty-exact-lane parity        PASS
```

Positive verdict: `M3_ACTUAL_OWNER_PARITY_PASS`.

This still does not admit end-to-end latency, RSS, reload/generation identity or
production activation. Those remain the next separate decision after owner
parity.

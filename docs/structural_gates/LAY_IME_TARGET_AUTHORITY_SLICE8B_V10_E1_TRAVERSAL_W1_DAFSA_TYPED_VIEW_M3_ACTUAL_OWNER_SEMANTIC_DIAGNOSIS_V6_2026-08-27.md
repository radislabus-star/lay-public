# M3 Actual-Owner Semantic Diagnosis V6

Date: 2026-08-27

Paper verdict: `M3_ACTUAL_OWNER_SEMANTIC_DIAGNOSIS_ADMITTED`

## Frozen V5 Result

V5 is immutable `BLOCKED_SEMANTIC`. Its source-bound identity repair passed:

```text
current V11 SHA-256             f2abc8cb8016319a9a074a47310bdb0cbf21859ea6930f93f154187499dc8dcc
payload SHA-256                 23b347c2026667eb1aab5224392834749d142c5e2f35ae2320f33e3ee0de5a23
historical projection SHA-256   5ebffb813ba0ca1e0080ec01756a2dafc51346297558d37cdd135abfde6acfaa
owner requests                  764 / 764
candidate/certificate mismatch  0 / 0
lattice mismatch                146
gate mismatch                   72
```

V5 cannot be retried.

## Bounded Prior

Read-only inspection of the sealed 382-case V7 input finds exactly 36
`punctuation_suffix` cases where the raw damaged surface differs from
`normalize_surface(damaged_surface)`. Two schedules yield 72 observations. The
current owner preparation compares normalized exact surfaces with raw
`observed`, while the proof denominator compares them with normalized observed.

This explains the exact gate-mismatch count and can explain 144 lattice
increments because the proof checks both the exact-marked lattice and emitted
surface set. It does not explain the remaining two lattice increments. The
correlation is a diagnosis prior, not repair authority.

## Diagnostic Route

V6 may edit only the ignored proof in `v13_typed_peak.rs` to publish:

```text
lattice-marker mismatches by class
emitted-surface mismatches by class
gate mismatches by class/action/reason
bounded mismatch samples with case index, schedule and damaged surface
expected exact surfaces
actual exact-marked surfaces
actual emitted exact-source surfaces
all emitted candidate source/gate rows for sampled cases
```

The owner preparation, material, composite, certificate and live candidate
code remain byte-identical to V5. V6 performs the same identity gate and one
fresh 382 forward plus 382 reversed diagnostic denominator. It does not seek a
PASS owner verdict and cannot activate a repair.

The positive diagnostic verdict requires:

```text
sidecar identity projection       PASS
owner requests                    764 / 764
candidate/certificate mismatch    0 / 0
schedule/completeness mismatch    0 / 0
at least one bounded mismatch sample
raw semantic verdict              BLOCKED_SEMANTIC
```

## Boundary

One new local Cargo invocation is admitted after a separate V6 execution
preflight. Network, remote execution, perf, PMU, sidecar publication,
installation, runtime mutation and production activation are forbidden.

After V6, a separate paper must choose either an exact source repair or a
terminal design rejection. No V6 observation grants a retry under V5.

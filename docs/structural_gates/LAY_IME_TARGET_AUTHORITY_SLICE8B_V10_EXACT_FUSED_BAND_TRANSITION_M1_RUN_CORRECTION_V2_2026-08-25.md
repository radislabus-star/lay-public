# V10 Exact Fused Band Transition M1 Run Correction V2

Date: 2026-08-25

## Terminal V1 Result

The one M1 build succeeded and its ELF is sealed. V1 parity then terminated
before package/index loading because the controller named two nonexistent B0a
aliases:

```text
incorrect sidecar   artifacts/v13-typed-peak-dafsa.bin
actual sidecar      artifacts/LAY-L2-RU-FULL-v13.dafsa

incorrect V7        inputs/denominator-v7.json
actual V7           artifacts/slice8b-v7-fixed-13x100.json
```

Observed V1 boundary:

```text
parity marker consumed             yes
package loaded                     no
transition trace built             no
transition subject executed        no
perf / PMU opened                   no
G0 / G1 / U1 markers consumed      no / no / no
V1 result published                no
V1 failure evidence sealed         yes
```

V1 is terminal and cannot be retried. Its remote state, markers and local
failure receipt remain immutable.

## Sole V2 Correction

V2 reuses the exact sealed ELF without Cargo or source changes. It may change
only:

1. the two asset paths above;
2. result namespace from `result-v1` to `result-v2`;
3. state namespace to a new disjoint `...-run-v2` path;
4. failure namespace from `run-failure-v1` to `run-failure-v2`;
5. controller/run provenance labels from V1 to V2.

The new V2 state creates fresh one-shot `parity/g0/g1/u1` markers. It must not
consume or repair any V1 marker. Build action is forbidden and removed from the
V2 executable route.

## Sequence

```text
V2 correction paper
  -> structural PASS, authority_ready=false
  -> implementation preflight READY_TO_IMPLEMENT
  -> controller-only correction
  -> self-check with exact sealed ELF and actual B0a names
  -> create disjoint V2 state
  -> one parity
  -> one G0, one G1, one U1 PMU replay
  -> immutable result-v2 publication
  -> STOP
```

Loaded host work remains untouched and cannot block instructions/transition.
The M1 threshold, trace contract, variants and claim boundary do not change.
No rebuild, S1, third C1, clean C1 marker, full B, V12, runtime integration,
deployment or installed Lay change is admitted.

V1 failure evidence:

`docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_EXACT_FUSED_BAND_TRANSITION_M1_RUN_FAILURE_V1_2026-08-25/`

Sealed build audit:

`docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_EXACT_FUSED_BAND_TRANSITION_M1_REMOTE_BUILD_AUDIT_V1_2026-08-25.json`

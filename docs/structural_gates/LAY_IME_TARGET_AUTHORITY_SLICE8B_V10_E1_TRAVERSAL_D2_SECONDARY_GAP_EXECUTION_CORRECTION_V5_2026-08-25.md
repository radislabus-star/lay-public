# V10 E1 Traversal D2 Secondary-Gap Execution Correction V5

Date: 2026-08-25

## Scope

This correction preserves D2 paper V4, capability-probe V1, T-CAP recovery V3
and precise-capability V3. It does not reinterpret the unusable I-CORE IPs,
execute I-ATOM, substitute an event, build or run D2, or admit an optimization.

The precise receipt is terminal `BLOCKED_CAPABILITY`. Its exact I-CORE event
opened, the controlled shutdown and all readers passed, but all `79` samples
reported the same `0xffffffffb3c001cd ([unknown])` IP outside every live user
mapping. Required Build-ID plus normalized-IP attribution is impossible.

```text
local precise receipt   c62ec8737cecf08e69b6b8d1ce2408e051f31086b58ff53e7b32961cad12e197
remote precise receipt  f357d86c52f478de29dda64cb9ebeb36fefee75ad7c1a7e501d9c65336ad3b3a
remote manifest         703d549483d30ed74a0a8c7c8e2b8940089cced65d72f1810597589b17f0d7df
I-CORE perf.data        99c384dafc73326993096c0bf6264c0c3a1d09ed6ec25364d116a0db193f5501
I-CORE evlist           5aca9927b19cae18ea93d58855763a9612128632dfe3cc6ddf3f07dc869d5c55
I-CORE samples          e89991dc9ad1627fe64a57f9ffe369b32e199fc058a00ba904c39657af6b7aa1
```

## Effective Interpretation

```text
T-CAP capability                    PASS_FROM_SEALED_EVIDENCE
I-CORE precise-IP capability        FAIL
I-ATOM capability                   NOT TESTED
complete secondary channel          UNAVAILABLE
failure class                       REQUIRED_IP_IDENTITY_UNUSABLE
```

I-ATOM remains unconsumed because the frozen sequence admits it only after
I-CORE PASS. Running it cannot restore a complete cross-PMU secondary channel
and remains forbidden under the consumed precise transaction.

## Existing D2 Decision

D2 paper V4 already preregistered:

```text
D2_ATTRIBUTION_WITH_SECONDARY_GAP
  primary task-clock attribution passes
  precise instruction sampling unavailable
  no single-mechanism optimization paper is admitted
```

Therefore precise failure does not require inventing a new observer merely to
continue the primary question. Task-clock samples remain the authority for
bucket CPU-time share and concurrency inflation. No per-bucket retired-
instruction share, instruction-heavy claim, stall claim or cross-channel
agreement may be published.

## Sequencing Overlay

This correction supersedes only the old implementation-preflight rule that P1
must pass both precise PMUs before any D2 controller can be written.

```text
T_CAP_RECOVERED_FROM_SEALED_EVIDENCE
AND I_CORE_REQUIRED_IP_CAPABILITY_FAIL
AND I_ATOM_UNCONSUMED
  -> D2_SECONDARY_GAP_CONFIRMED
  -> new primary-only final implementation preflight may be created

new final preflight READY_TO_IMPLEMENT
  -> D2-A immutable closure
  -> ONE symbolized build
  -> ELF/DWARF/PT_LOAD audit
  -> seal D2_BUCKET_MAP
  -> parity
  -> U-SINGLE -> U-FIXED -> U-REVERSED
  -> all-U perturbation gate
  -> T-SINGLE -> T-FIXED -> T-REVERSED
  -> at most D2_ATTRIBUTION_WITH_SECONDARY_GAP
```

No `I-*` build or measurement marker exists in the primary-only route. The
preflight must prove source absence of precise and substitute-event routes.

## Hard Ceiling

Even if primary task-clock attribution is valid, this route cannot by itself
admit SWAR, decoder/layout work, rank/stack work, V12, full B, runtime
integration or deployment. It publishes bucket CPU-time and inflation evidence
for a later paper decision, with the secondary gap explicit.

The existing parity, exact work ledger, bucket-map integrity, U-route
perturbation, task-clock period, minimum `50,000` traversal samples, zero lost
samples, at most `5%` unattributed CPU samples, loaded-host policy and thermal
gates remain unchanged. Thresholds are not relaxed.

## Authority Boundary

This paper correction alone admits no code or execution. A structural route
receipt and a new named implementation preflight must both pass before a
primary-only D2 controller can be written. Runtime authority remains unchanged.

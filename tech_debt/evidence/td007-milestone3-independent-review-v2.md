# TD-007 Milestone 3 Independent Review V2

Review mode: fresh context, read-only, two passes maximum.

## Pass 1

Score: `5/10`

Material findings:

1. a proposed target-evidence broadening changed runtime authority without a
   false-authority proof;
2. the first verification receipt no longer matched the reviewed source
   closure;
3. several replacement tests asserted candidate production without the
   downstream authority or safety effect;
4. the sparse-reserve and participle tests stopped before the unified candidate
   gate;
5. the owning architecture document did not yet record the contract changes.

Repairs:

- the runtime broadening was removed completely;
- the final verification was regenerated from the repaired source closure;
- origin-only assertions now include L3 `Suggest`, authority demotion, or
  known-surface safety postconditions as appropriate;
- sparse reserve and reference-backed ambiguity now pass through the unified
  candidate gate using hermetic inputs;
- the architecture document records tested facts, untested scope, and the
  unchanged runtime-authority boundary.

## Pass 2

Score: `7/10`

The reviewer found no runtime-authority regression after the broadening was
removed. One residual P2 remained: the exact-layout sequence proof established
candidate birth but not the downstream readout. That test now also requires the
same exact surface to reach the L3 `Suggest` result.

A second general reviewer process timed out before returning a review and was
stopped. It produced no finding or score and is not counted as a review pass.
No third scored pass was used.

## Final Evidence

```text
review passes used                 2 / 2
score at final review time         7 / 10
material pass-1 findings repaired  5 / 5
residual pass-2 findings repaired  1 / 1
unexpected full-suite failures     0
infrastructure failures            0
source runtime semantics changed   false
```

Final verification:

`tech_debt/evidence/td007-milestone3-verified-v1.json`

Verdict: `MILESTONE_3_REVIEW_COMPLETE`.

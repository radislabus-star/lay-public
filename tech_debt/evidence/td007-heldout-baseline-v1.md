# TD-007 Heldout Baseline V1

Status: `FROZEN_BEFORE_MILESTONE_1`

Base commit:

```text
6f14574452d9ab504f23be21b11288179bc6f0f2
```

## Canonical Quality Route

The accepted full denominator is the immutable Phase 8I Gate C V2 result:

```text
receipt
docs/structural_gates/receipts/L1_L11_PEAK_SEARCH_PHASE_8I_2026-08-14/evidence/
phase-8i-gate-c-13x20000-full-v2-bounded-projection.json

receipt SHA-256
1cc06aac0b1906e21cc307204475ae2f6f71bcb4df6dc93c768404ae2df44bef

time receipt SHA-256
cb5f8dea8b98e469d090dba45761de3063cc786e50a74a6b2eb09b1e7cda6ece
```

Exact recorded command:

```text
/home/e/build/lay-l1-exact-phase2d/target/release/lay-nanda-wave-train
  --prove-l1-typed-basin-quality
  /home/e/build/lay-l1-shadow/artifacts/l11-l2-global-compaction-2026-07-30/corpus-852582.txt
  --memory /home/e/build/lay-l1-shadow/artifacts/l11-l2-global-compaction-2026-07-30/package.v8.bin
  --heldout-per-class 20000
  --workers 20
  --receipt /home/e/build/lay-l1-exact-phase2d/docs/structural_gates/receipts/
    L1_L11_PEAK_SEARCH_PHASE_8I_2026-08-14/evidence/
    phase-8i-gate-c-13x20000-full-v2-bounded-projection.json
```

Immutable inputs inherited by the accepted route:

```text
package bytes          190,139,182
package SHA-256        47fa757acac03b0f76e5397e965b9127884e245e9845ce0f1ca0896fb40f33e9
corpus SHA-256         abb9d71495acc27b7619b24621568ea72263f9bd40201aab9fffa1188fbff8f6
heldout cases          260,000 (13 x 20,000)
heldout SHA-256        ba0a07ee7b784742c4aef08956c6e48dc9836da10d01dc50d516dc946b00c501
clean centers          852,582
```

The corpus and heldout hashes are pinned by the Phase 2D immutable-input
receipt; Phase 8I reuses the same package/corpus route and records the exact
package identity. The full Phase 8I result reports every damage class strictly
above 95% unique top-1, complete-field target retention `260,000/260,000`,
clean preservation `852,582/852,582`, and zero false authority/singleton.

## TD-007 Boundary

The 116 hermetic failures are not authority to change runtime behavior. Each
row must be closed by one of:

```text
CURRENT_CONTRACT_RUNTIME_REPAIR
SUPERSEDED_EXPECTATION_REPLACED
HIDDEN_INPUT_DEPENDENCY_REMOVED
HISTORICAL_CASE_ARCHIVED_WITH_REPLACEMENT_PROOF
```

Forbidden dispositions:

```text
fixture-specific runtime branch
literal phrase or test-name condition
weakened SafetyGate, DecisionCore, edit validation, or verifier
aggregate-only quality claim
dropped grounded candidate
unrecorded deletion or renamed test
```

The full remote quality command is retained as immutable accepted evidence. It
is not rerun for assertion-only or hermetic-fixture repairs. Any runtime change
to candidate birth, retention, ranking, or admission requires a fresh scoped
proof and the full fixed quality gate before release admission.

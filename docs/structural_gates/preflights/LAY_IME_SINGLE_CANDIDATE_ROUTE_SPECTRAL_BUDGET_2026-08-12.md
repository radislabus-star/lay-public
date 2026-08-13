# IME Single Candidate Route Spectral Budget

Status: pre-implementation review.

## Tree

```text
current
  preedit orchestrator
  +-- phrase forecast adapter
  +-- RU/ASCII word adapter -> shared word field
  +-- ASCII word adapter    -> shared word field again
  +-- display arbitration

target
  preedit orchestrator
  +-- phrase forecast adapter
  +-- word adapter -> one shared word field
  +-- display arbitration
  +-- explicit Tab -> unchanged verifier/apply route
```

## Weighted Ledger

```text
+5 route coherence: one word-field request per refresh
+4 hot-path isolation: duplicate L1/L2/L3/L4 query removed
+4 proof/runtime separation: phrase forecast remains a producer, not authority
+3 testability: one producer contract replaces script-specific adapters
+3 observability: one word timing receipt has one meaning
+3 public schema stability: existing timing fields remain readable
+2 navigation: producer names describe phrase versus word ownership
-3 source movement risk in the IBus hot path
-2 test migration for direct ASCII-adapter fixtures
-----------------------------------------------
score: +19
```

Hard veto audit:

```text
public display behavior changed          no
source-specific runtime hardcode added   no
proof fixture used as authority          no
verifier weakened                        no
algorithm mixed into move-only cut       no; evidence-gate is tested separately first
```

Verdict: `PROCEED` only after the structural and implementation preflight gates pass.

## Budgets

```text
word-field calls per refresh        2 -> 1 for ASCII, 1 -> 1 for Cyrillic
word-field authority owners         1 -> 1
phrase producer calls               1 -> 1
display arbitration owners          1 -> 1
apply/verifier owners               1 -> 1
public timing JSON fields           unchanged
global IBus restarts                0
```

## Proof

Required before installation:

```text
replacement evidence matrix         PASS
exact layout ytn -> нет             PASS
RU prefix completion preservation   PASS
EN prefix completion preservation   PASS
single word-field call contract     PASS
lay-ibus-engine focused suite        PASS
release build                        PASS
```

## Code Route Gate V2 Result

Tested: `nanda.code-route-gate.v2` against the current source-backed route and
the proposed target route. Execution calls, authority/data flow, observation,
and proof are separate graphs; every edge is scoped to a named event.

Measured facts:

```text
CURRENT scope                    observed_source
CURRENT source evidence          23 / 23 verified
CURRENT word-field execution     2 paths in ime_refresh_ascii_token
CURRENT verdict                  VETO

TARGET scope                     design
TARGET word-field execution      1 path in ime_refresh_ascii_token
TARGET authority owners          one per singleton role
TARGET verdict                   PASS
TARGET safe_to_edit              false
TARGET next gate                 implementation preflight
```

Not tested: the target route is not implemented and no Lay runtime authority
changed. Its PASS proves only that the paper route is internally coherent.

Receipts:

```text
docs/structural_gates/receipts/LAY_IME_CODE_ROUTE_CURRENT_2026-08-12.json
docs/structural_gates/receipts/LAY_IME_CODE_ROUTE_TARGET_2026-08-12.json
```

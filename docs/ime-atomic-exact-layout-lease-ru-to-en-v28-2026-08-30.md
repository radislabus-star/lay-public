# Lay IME RU-to-EN Exact-Layout Lease V28

Status: `DEPLOYED_VERIFIED`

## Problem

Lay `1.0.54` deterministically recognized `Згыр -> Push` in the typing-assist
rule graph, but the live IME Space route returned `full_no_apply/rank`. V27 had
a closed exact certificate only for `EN -> RU`; `RU -> EN` was deliberately left
to the full field. The candidate therefore existed but had no live apply
authority.

Measured facts:

- installed `lay 1.0.54` explained `Згыр ` as `layout_ru_to_en -> Push `;
- the reproduction trace recorded `ibus_space_autocorrect/full_no_apply/rank`;
- `auto_replace`, `auto_switch_layout`, and `nanda_autocorrect` were enabled;
- no source or runtime change had been made when the defect was reproduced.

## Consequences

The change must preserve the candidate lattice and mutation monopoly. It may
retain one exact projected candidate, but it must not suppress competitors in
the full route, weaken DecisionCore/verifier checks, or add a second IME edit
owner. Known Russian and protected source tokens must remain unchanged, which
is the principal false-authority risk.

The exact lane stays allocation-bounded and read-only after startup. It reuses
the existing English guard, Russian terminal, keyboard relation, worker lease,
frame identity, and cache generation. The reverse keyboard table and exact
Cyrillic protection set are warmed once by the existing IME warmup owner. No
per-key file access, package load, cache, fallback, retry, or new resident model
is admitted. Package/delta reload identity remains bound by the existing
Russian-terminal fingerprint and material generation.

The bidirectional warm set uses a 15 MiB engine RSS envelope. An isolated
mini-PC run measured 14,396 KiB versus the old one-direction 14,336 KiB limit;
the 60 KiB difference is admitted deterministic map/protection data, not a
per-request allocation or an unbounded cache.

Learning and feedback remain downstream of a successful authorized IME edit.
Prepared, stale, rejected, or expired certificates create no positive evidence.
Concurrency semantics are unchanged: the printable-key worker prepares one
lease; Space consumes at most that frame-matching lease. Any profile, layout,
frame, map, lexical fingerprint, protection fingerprint, or configuration
mismatch fails closed.

IME and daemon compatibility is unchanged. Double Shift remains exclusively
owned by the daemon/manual-toggle route. The new certificate cannot commit text,
switch layout, replay keys, or mutate runtime state.

Maintenance cost is one bidirectional certificate implementation over the same
types. If a future multilingual authority replaces RU/EN projection, this V28
branch and its direction enum are removed together; no compatibility fallback
is retained.

## Designs

1. **Selected: symmetric warm-only certificate.** For an exact `Ru/Ru` frame,
   require a plain Cyrillic token, stable case, unknown/unprotected Russian
   source, exact keyboard projection, and known English target. Feed the one
   retained candidate through the existing DecisionCore and verifier.
2. **Rejected: loosen full-field ranking.** This would alter unrelated typo and
   lexical competition, add latency dependence, and increase false authority.
3. **Rejected: direct deterministic commit.** This would create another text
   mutation owner and bypass the verified edit contract.

## Frozen invariants

- eligible profiles are exactly `UsQwerty/Us` and `Ru/Ru`;
- `EN -> RU` keeps its existing source and target guards;
- `RU -> EN` requires source absence from the exact Russian terminal and exact
  Cyrillic protection set, plus target presence in the exact English guard;
- only lower, title, and upper case shapes are admitted;
- only one trailing ASCII space is certified;
- punctuation at either token edge, mixed script, unknown words, known Russian
  words, protected words, cold authority, and identity drift yield `NoApply`;
- retained exact candidates still pass DecisionCore, transition verification,
  backend authorization, and committed-tail replacement;
- no Double Shift, preedit completion, layout-sync, or daemon code changes.

## Proof and release result

The release gate passed:

- `Згыр -> Push` and `згыр -> push` pass through the real Space decision route;
- multiple non-fixture RU-to-EN English targets pass without runtime literals;
- known Russian, protected Russian, mixed-script, punctuation-edge, unknown
  profile, wrong decoder, disabled policy, stale frame, and cold-read cases stay
  blocked;
- direct authority tests prove that either `auto_replace=false` or
  `auto_switch_layout=false` disables the route;
- the existing V27 EN-to-RU oracle/tests and short-word false-positive tests
  pass;
- the hermetic ordinary release denominator is `2,369 passed`, with zero
  semantic and infrastructure failures;
- installed `lay`, `lay-daemon`, and `lay-ibus-engine` report `1.0.55`;
- loaded daemon SHA-256 is
  `749b9c95e541d9c0c8453952f0aecf486bf065b20181fd5729173debd7e97a9b`;
- loaded IBus engine SHA-256 is
  `8247f8fce94b38da478bf63203dfae7fdf63c828e84c0557bf537b8563fbc19c`;
- the loaded GNOME extension reports `1.0.55` through
  `io.github.radislabus_star.LayDaemon.Version`;
- live GTK/IBus smoke proves `Згыр + Space -> Push + Space`, synchronizes the
  active layout to English, blocks the route under either disabled option, and
  preserves both the plain and accepted-autocomplete Double Shift routes;
- the smoke harness restores the original desktop input sources, active engine,
  configuration, and managed runtime after every case.

Authoritative live receipt:

```text
docs/structural_gates/receipts/
  LAY_1_0_55_RU_EN_EXACT_AUTOSWITCH_2026-08-30/
  FINAL_RELEASE_LIVE_SMOKE/RECEIPT.json

SHA-256
fc459523d460362b6be1a380f40194f7661fe31f2b5b387dd19c73ccbf32c429
```

Runtime authority changed only in the completed `1.0.55` installation
transaction. Rollback boundary: reinstall the sealed `1.0.54` binaries.

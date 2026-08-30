# TD-007 Milestone 2 Independent Review V2

Review mode: fresh context, read-only, two passes maximum.

## Pass 1

Score: `5/10`

Material findings:

1. the atomic proof compared settlement with a suffix parsed from the same
   proposal and lacked an independent semantic target check;
2. the derivational-prefix guard covered only exact or one-character-short
   fragments;
3. the frameless helper asserted only `None`, while exact-layout positives did
   not bind certificate identity, transition proof, error class, or edit plan.

Repairs:

- atomic publication now forms the complete target and verifies it as an
  independently expected and lexically known completion;
- prefix-fragment handling covers all nontrivial prefixes of the frozen
  derivational-prefix set, with focused false-split and valid-boundary tests;
- frameless routes require an explicit `Rank` or `Verifier` no-apply stage;
- exact routes bind frame identity, original/projected token, contextual
  replacement, `Layout` proof, `wrong_layout` class, and physical edit plan.

## Pass 2

Score: `7/10`

The reviewer confirmed that the prefix guard, IME authority contracts, exact
28-row ledger partition, and verification binding were repaired. One residual
P2 remained: accepting any known Russian word beginning with `пров` was still
weaker than an independent expected completion.

The residual was repaired after the final review pass by limiting the atomic
proof to the two observed valid deterministic outcomes, `проверка` and
`проверить`, while retaining lexical validation and frame-settlement equality.
The focused atomic proof and final full verification V3 pass. No third review
was used to inflate the score.

## Final Evidence

```text
review passes used                 2 / 2
score at final review time         7 / 10
material pass-1 findings repaired  3 / 3
residual pass-2 findings repaired  1 / 1
unexpected full-suite failures     0
infrastructure failures            0
```

Final verification:

`tech_debt/evidence/td007-milestone2-verified-v3.json`

Verdict: `MILESTONE_2_REVIEW_COMPLETE`.

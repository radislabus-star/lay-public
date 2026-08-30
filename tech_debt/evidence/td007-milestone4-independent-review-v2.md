# TD-007 Milestone 4 Independent Review V2

Status: `FINDINGS_REPAIRED_NO_THIRD_REVIEW`

Base commit before milestone 4:

```text
86eef2f794130cefa00d8ff74b75341a1c1ceddf
```

The review was limited to two fresh-context passes as required by the task.
Scores below describe the source at review time, not an invented score after
the author repaired the findings.

## Pass 1

Score: `3/10`.

Findings and dispositions:

1. Pending automatic undo could select daemon uinput without proving ownership
   of the physical grab. The output route now returns `DaemonUinput` only from
   the pending branch that preserves physical-grab ownership, with an explicit
   regression test.
2. The default canonical route could auto-apply an ambiguous split around the
   `парочинная` family. The ambiguous candidate remains observable but is no
   longer granted automatic mutation authority.
3. Punctuation-shaped layout fallback could rewrite shell-like or quoted text.
   Physical-key punctuation evidence is retained only for the exact contextual
   projection; shell, command, and quoted keep cases are explicit fixtures.
4. The closure evidence did not yet prove an exact partition. The final ledger
   contains 42 unique baseline rows, every replacement proof exists in the
   exact manifest, and the machine check reports no missing proof.
5. Broad corpus assertions reported only aggregate behavior. Replacement
   fixtures now carry per-row semantic postconditions and the hermetic package
   lane executes them independently.
6. Broad noun morphology admitted invalid `-ие` surfaces such as `историе`,
   `армие`, and `станцие`. The backed-form derivation now distinguishes the
   relevant lemma endings and has positive plus negative morphology contracts.
7. A 99% boundary-retention shortcut granted authority without positive
   separation. The residual form was inspected again in pass 2 and then removed
   completely rather than recalibrated.

## Pass 2

Reviewer agent: `01a0521a-f136-7551-aece-696683f0474e`.

Score: `5/10`.

Findings and dispositions:

1. The pending-undo regression test did not reach the pending branch because
   backend rejection ran first. Pending state is now checked before backend
   availability and the focused daemon test proves physical-grab ownership.
2. `SplitPreviousGluedAndRepairTail` was classified from shape alone yet could
   still acquire automatic authority. The operator is now explicitly
   unverified, its admission bypasses are removed, and the composite candidate
   remains `SuggestOnly`; an unrelated verified one-word repair may still win.
3. Backed morphology still derived `рукы`, `ногы`, and `дачы` through a second
   generic route. One shared orthographic predicate now rejects `ы` after
   `г/к/х/ж/ч/ш/щ` in both backed-form entrypoints.
4. The residual 99% boundary shortcut still broadened automatic authority. The
   shortcut and its synthetic threshold test were removed; unsafe moved-prefix
   proposals remain visible but cannot auto-apply without independent proof.
5. The quality receipt lacked executable, argv, time, and source binding. A new
   full 13 x 20,000 plus all-clean run was executed from a freshly built binary,
   and `td007-milestone4-quality-binding-final-v1.json` binds exact argv, binary,
   source closure, inputs, timestamps, exit status, and raw receipt SHA.

The reviewer separately closed canonical ambiguity, punctuation protection,
frameless lexical authority, exact frame routing, per-row fixture coverage, and
the full lane design. No fixture literal or test identity was added to runtime,
and SafetyGate/edit-plan/verifier authority was not weakened.

## Final Verification After Findings

```text
manifest                            2,383 exact rows
ledger                                 42 / 42 closed
correctness                         2,323 / 2,323 PASS
package                                36 / 36 PASS
known semantic failures                         0
quality damaged                    260,000 / 260,000
quality clean                      852,582 / 852,582
minimum class unique top-1                 97.0531%
minimum class lattice coverage             99.2150%
authoritative false authority/singleton         0 / 0
installed runtime authority changed             false
```

Evidence:

- `tech_debt/evidence/td007-milestone4-correctness-post-review-v2.json`
- `tech_debt/evidence/td007-milestone4-package-post-review-v1.json`
- `tech_debt/evidence/td007-milestone4-quality-13x20000-final-v1.json`
- `tech_debt/evidence/td007-milestone4-quality-binding-final-v1.json`
- `tech_debt/evidence/td007-milestone4-ledger-v1.json`

No third review pass was requested or performed.

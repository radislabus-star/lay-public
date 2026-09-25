# TD-127 — Kitty Space autocorrection diverges from the visible Tab completion

Priority: untriaged user-reported correctness defect.
Status: `OPEN / USER-REPORTED / NOT REPRODUCED`.
Recorded: 2026-09-22.
Runtime authority changed: no.

## Observation

The user reports this exact behavior in Kitty:

1. type `которую`;
2. the IME visibly suggests `которую`;
3. pressing `Tab` to accept/autocomplete produces `которую`;
4. pressing `Space` instead produces `котором`.

The visible completion and the explicit `Tab` route therefore agree on
`которую`, while the automatic correction selected at the closing Space
diverges to `котором`.

This is a user observation, not a controlled reproduction. The active Kitty
process identity, layout, preceding context, candidate list, correction trace,
decision proof, edit plan, latency and installed artifact hashes were not
captured. The first failing layer is `UNKNOWN`.

## Required investigation boundary

Do not repair this as a literal-word exception. Freeze one physical Kitty
reproduction and compare the two complete authority paths for the same input
frame:

```text
visible completion -> Tab acceptance -> selected proposal -> edit effect
Space autocorrection -> prepared lease -> DecisionCore -> verifier -> edit effect
```

Record the first shared mechanism where the selected surface changes from
`которую` to `котором`. Keep candidate generation, display ranking, explicit
completion acceptance, automatic correction ranking, verifier admission and
terminal erase/commit as separate observations. A visible suggestion proves
display only; successful `Tab` acceptance does not by itself prove that the
Space route consumed the same proposal.

## Acceptance gate for a later repair

- controlled RED in the Kitty terminal route without a runtime word/phrase
  condition;
- exact candidate surfaces and selected target recorded for both `Tab` and
  `Space` from one frozen input frame;
- one systemic mechanism repair at the first divergence;
- the complete fixed proof and every required error class remain non-regressing;
- physical Kitty result preserves the preceding text and yields exactly
  `которую ` on Space;
- Tab still yields `которую`, and browser/GTK routes retain their existing
  contracts.

No diagnosis, fix, test, installation or runtime change is authorized or
claimed by this record.

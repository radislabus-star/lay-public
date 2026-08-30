# TD-104 Boundary Correction V3

Date: `2026-08-30`

## Observed V2 Failure

The first disposable mini-PC default check exited `101`. The core
`ContextPhasePackage` implementation and live readout use
`sentence::SentenceContextScene`, sentence marker predicates, and pair-view
constants. No installed path or process changed.

Remote evidence:
`tech_debt/evidence/td104-remote-candidate-v2-failure/`.

## Effective Correction

Keep `context_phase::sentence` unconditional. Only its cold compile/proof
re-exports remain feature-gated. Split composite and runtime-refresh re-exports
so runtime owners stay unconditional while cold admission/status functions
remain in `research-tools`.

V1 and V2 remain superseded evidence. The route graph, test denominator,
measurement protocol, and no-deployment boundary are unchanged.

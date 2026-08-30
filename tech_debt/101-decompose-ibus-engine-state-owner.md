# TD-101: Decompose The IBus Engine State Owner

Status: `DONE`
Priority: `P2`
Class: complex stateful architecture
Size: `XL`
Decision dependency: finish TD-001 through TD-008

Admission: `READY_TO_IMPLEMENT` at base
`23bcd577c58e49f339a27baf1220c9490f07b628`; see
`evidence/td101-implementation-preflight-receipt-v1.json`.

## Evidence

- `LayIbusEngine` is a graph hotspot and carries composition, preedit,
  committed-tail, focus, layout, worker-generation, learning, postcondition, and
  bridge state.
- Related files include `engine.rs` 517 lines, `engine/types.rs` 453,
  `state.rs` 1,297, `tail_memory.rs` 1,722, `preedit.rs` about 1,000, and
  `preedit/tests.rs` 2,217.
- Recent regressions repeatedly crossed preedit, tail memory, layout sync,
  committed-tail, and Double Shift ownership.

## Proposed Outcome

Keep one external `LayIbusEngine` owner while extracting cohesive internal state
objects with explicit transitions:

- composition/preedit display state;
- committed-tail and visible-postcondition state;
- manual-toggle/layout synchronization state;
- focus/client capability state;
- background worker identity state.

## Minimum Safe Cut

Move types and pure transition helpers first. Do not split the mutation owner,
add new asynchronous paths, or change D-Bus/public behavior. Every cut must be
move-only and preserve the canonical Double Shift route.

## Decision Questions

- Do recurring regression costs justify this change now?
- Can the same benefit be achieved by stronger transition tests without moving
  state?
- Which state group has enough route-local evidence for a `SPLIT_STRONG`
  boundary verdict?

## Required Preflight If Admitted

- Code-route map for display, decision, mutation, observation, and proof owners.
- Full IME regression class and managed GTK baseline.
- Failure-transition inventory for focus reset, worker cancellation, and layout
  handoff.
- At least 10 weighted budget points with no hard veto under the spectral rule.

## Rejection Condition

Reject or defer if the proposal is primarily cosmetic, creates wrapper modules,
or cannot prove lower regression risk without changing behavior.

## Completion Record

- Result: `TD101_VERIFIED`
- External owner: one `LayIbusEngine`
- State shape: 47 state fields in five cohesive groups; eight direct engine
  fields including `path`, `shared`, and `config`
- Observed route: `PASS`, 35/35 source markers, 11 separated routes
- Remote engine tests: 276/276 PASS
- Protected route contracts: 24/24 PASS
- Remote changed manifest: 2,383 entries, all selected lanes PASS
- Non-dead lint diagnostics: 0
- Independent review: `9.5/10`, `ACCEPT`, no findings
- Installed runtime: unchanged

Evidence:

- `evidence/td101-code-route-observed-v1.json`
- `evidence/td101-code-route-observed-receipt-v1.json`
- `evidence/td101-verification-v1.json`
- `evidence/td101-independent-review-v1.md`
- `evidence/td101-completion-v1.json`

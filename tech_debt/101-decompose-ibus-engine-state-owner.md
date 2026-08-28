# TD-101: Decompose The IBus Engine State Owner

Status: `DISCUSSION_REQUIRED`
Priority: `P2`
Class: complex stateful architecture
Size: `XL`
Decision dependency: finish TD-001 through TD-008

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

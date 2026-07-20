# Lay Library Architecture

This is the short canonical orientation document for agents and maintainers.
The detailed phase-memory contract remains in
`docs/phase-word-recovery-canonical-cutover.md`.
The bounded one-pass L3 learning contract is
`docs/l3-online-phase-field.md`.

## Goal

Lay is one reusable typing-transition library with thin runtime clients.

```text
observe visible state
-> propose candidates
-> score context and memory
-> decide one typed transition or ABSTAIN
-> verify the predicted edit
-> create AuthorizedEdit
-> execute through a backend
-> record accepted/rejected experience and metrics
```

The target library front door has seven operations:

```text
observe -> propose -> decide -> authorize -> execute -> learn -> report
```

Daemon, IME, CLI, tray and eval code must call this library. They must not own
candidate truth, ranking policy or text-mutation authority.

## Current Owners

```text
L1 surface and relation signals       src/ngram, src/lexical_surface_atoms.rs
L2 candidate and phase field          src/nanda_wave/l2*, lexical_phase
L3 phrase/context field               src/nanda_wave/l3*, context_phase
L4 hidden/signed transition memory    src/nanda_wave/l4*, typing_memory
sole chooser                          src/typing_transition/decision*
actor and verifier                    src/text_edit, src/typing_transition/verifier.rs
sealed mutation capability            text_edit::AuthorizedEdit
daemon adapter                        src/bin/lay_daemon
IME adapter                           src/bin/lay_ibus_engine
proof and metrics                     src/bin/lay_nanda_wave_eval, tests
```

## Dependency Law

```text
clients -> library facade -> domain owners -> pure primitives
proof/eval -> library facade and diagnostic receipts
backends <- AuthorizedEdit only
```

Forbidden:

- direct apply from L1, L2, L3, L4, rules or IME;
- candidate ranking inside a backend;
- runtime dependence on fixtures or eval reports;
- a second chooser beside `TransitionDecisionCore`;
- text mutation without `AuthorizedEdit`;
- raw corpora or expanded word strings as hot decision authority;
- duplicate architecture truth in source prose.

## Refactor Budget

The reduction target is the active production surface, not deletion of proof.

```text
public library entrypoints             <= 7 operations
public top-level modules               <= 12
production Rust target                 <= 45,000 physical lines
unsafe multiword apply                 0
unverified left-context mutation       0
candidate coverage                     not below baseline
latency p50/p90/p99                    not worse than baseline
runtime RSS/PSS                        not worse than baseline
```

Proof, fixtures and metrics move behind a clear lab boundary. They may shrink
through shared harnesses and data-driven cases, but their assertions and
denominators must survive.

## Cut Order

1. Freeze baseline in `docs/refactor-baseline-0.2.264.md`.
2. Remove dead reports and duplicate architecture descriptions.
3. Introduce one library facade over existing behavior.
4. Move daemon and IME orchestration behind thin client entrypoints.
5. Hide internal modules from the public crate surface.
6. Consolidate duplicate candidate, token, layout and metric helpers.
7. Replace repeated Rust test code with shared harnesses and fixtures.
8. Delete legacy candidate/apply routes only after zero-caller graph proof.
9. Compare quality, safety, latency and memory with the frozen baseline.

Move-only refactors must not alter ranking. Ranking changes require a separate
quality experiment and their own release proof.

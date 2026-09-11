# Native terminal delivery: independent source review

Review completed 2026-09-10 12:21 UTC by the independent
`replacement_cursor_review` reviewer. First formal review round of at most two.
Verdict: **PASS for the source change; H0, M0, L1 (validation scope)**.
No source edits or test execution by the reviewer. No numerical score assigned.

The review covers the explicit legacy terminal/no-SurroundingText predicate,
retained narrow-cursor compatibility, purpose-change mode invalidation, first
atomic activation, shared captured frame and prefetch-before-hint order,
Space suppression and authorization, native fallback without a second commit,
and unchanged managed-word stickiness after capability loss. Migrated fixtures
retain suffix, completeness, revocation, manual projection and refusal checks;
a FIFO fence verifies absent text mutation without sleeps.

The low finding concerns proof scope: the Space unit matrix disables assistance
while typing and injects a prepared lease. It cannot detect missing native
prefetch scheduling or invisible hints. The real IBus client proof must cover
both with assistance enabled before typing. This calls for verification, not a
production repair. Full gates, actual-client and physical acceptance remain
separate from this source verdict.

Reviewed runtime file identities:

- `src/bin/lay_ibus_engine/engine.rs`: `25e635ad2f5f8052abe7ac6899dec049315f884f69aa2400fd4c9d62437c1555`
- `src/bin/lay_ibus_engine/atomic.rs`: `b6cbee334f90ac06b83fc5fd316170bd5272e3a94b0b159f8ae05f0c186b5e8f`
- `src/bin/lay_ibus_engine/composition_commit.rs`: `344a525b388879af088ff8afaa5ec7bf8ddc9fd96c83c7ce6daa8281fa28efd7`
- `src/bin/lay_ibus_engine/managed.rs`: `80f3e2ddbfceb526eda583917885fcddb792486610bce82128a3c5bfa41f4e37`

This report is frozen evidence. Later verification is recorded in the owning
`docs/ime-terminal-commit-order-2026-09-10.md` document. It does not change the
installed runtime or grant additional mutation authority.

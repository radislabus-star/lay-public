# Lay 1.0.124: letter-triggered layout correction

Status: source and new-feature native gates completed; matched native baseline
comparisons completed. Public CI and publication are pending. The ordinary
compatibility suite retains known failures; this is bounded feature evidence.

The optional fragment detector now runs on an alphabetic key's paired release.
It proposes an ENG ↔ RU conversion while the word is being typed. It calls the
existing word-conversion operation with `Fragment` provenance. Space continues
through the ordinary correction rules; enabling the option cannot replace their
candidate generation or veto their decisions. The default remains off.

The minimal ownership map is:

```text
letter release
  src/bin/lay_ibus_engine/ibus_interface.rs
    → src/bin/lay_ibus_engine/fragment_layout.rs
      → src/layout_fragments/letter.rs
        → src/typing_transition/decision.rs
      → existing current InputFrame and edit authority
      → src/bin/lay_ibus_engine/shift.rs::request_word_layout_flip
        → existing DecisionCore, verifier and output operation

Space
  existing boundary handler
    → src/typing_rule_graph/rules/fragment.rs: ordinary boundary rules
    → existing correction pipeline and verifier
```

For a managed KDE native field whose last letter has not yet become observable,
`fragment_layout/reply.rs` uses the existing zbus reply-dispatch notification.
It re-enters the same engine and verifies the original owner, focus, word,
epoch and mode before obtaining a fresh field snapshot. Dispatch is not an
acknowledgement from the application and never grants permission to edit.
Disabled mode, Space, GNOME and native terminals retain their ordinary reply.
The IBus key reply still has its original boolean wire representation.

Current-word mode completion reuses the existing layout operation. The separate
GNOME exact-tail descriptor in
`src/bin/lay_daemon/layout_controller/fragment_word_mode.rs` does not introduce
another text writer. Manual return is bound to the same word and generation.
Old V3 status values and V2 behavior are preserved; the additional V3 descriptor
is rejected safely by an older daemon.

A separate authorized fix makes a late suggestion retire through the existing
`retire_pending_precognition()` operation, preserving the user's typed text.
The patch also includes the previously verified full-restart menu action for
GNOME and KDE. Neither change replaces the correction or delivery architecture.

## Completed evidence

- Canonical source/release gate64: all correctness, package, performance, lint,
  architecture, release-binary and installer gates PASS. Rust 1.88.0 identity
  and both locked all-target checks PASS. All ten actual installable artifacts
  were retained from compiler-artifact records.
- Rust: 3,650 correctness/package tests and 11 performance tests PASS. The
  catalog contains 3,706 identities: 3,614 correctness, 36 package, 11
  performance and 45 ignored/opt-in. Twenty-eight owned-Xvfb opt-ins were also
  executed successfully; the remaining research/live opt-ins were not invoked.
- Python: 467 of 479 PASS in the headless gate, with 12 expected environment
  skips. The ten graphical cases PASS in their graphical environment; the two
  real-system opt-ins PASS separately. A whole graphical run's seven private-bus
  resource-fixture failures remain in its raw report and are not relabeled PASS.
- KDE, Chrome and Firefox: each **108 PASS / 0 FAIL / 12 BLOCKED / 0 NOT TESTED**
  in the complete fragment suite. Four letter cases cover first/second word and
  both directions, ten fields and normal/private/fullscreen modes. Every case
  checks two manual returns and the next physical glyph. The blocks are exactly
  URL/email second-word scenarios where HTML removes the required ASCII Space.
- Primary GNOME: 32 physical word/mode transitions and three native Tab/held-Shift
  cases independently verified. Twenty plus six controlled malformed-evidence
  cases were rejected. The raw Tab actor's legacy producer assertion remains a
  separate failed diagnostic; its independent native verifier passes.
- GNOME latency, 32 transitions: second physical Shift press to first native
  replacement, measured on one monotonic clock. Median bounds 12.535–14.545 ms,
  nearest-rank p95 bounds 89.818–91.827 ms, maximum bounds 95.780–97.796 ms.
  These are GTK/GNOME observations, not a universal latency claim.

The final source closure has 1,338 files and SHA-256
`6ae8643762400ebc7c4fc3cc37f0abd73fb153ebd3ceeab71b1eb77663d5b3b5`.
All ten frozen artifacts63/65 are byte-identical. Their IME SHA-256 is
`a69654a20d29496d0cd08de35fbc8486c195c2a302b4276e4264cffcf52e132f`;
daemon SHA-256 is
`57aaf3e090d7fcfc67b1b777fdc366ca47aae5bbbaf95faf79d107dfba337e8f`.
All native runs use private mutable learning data, retain original model hashes
and restore the original installed 1.0.123 runtime, configuration and layout.
No permanent primary installation follows from these diagnostic results.

## Ordinary compatibility and release scope

The complete ordinary KDE browser suite remains **329 PASS / 59 FAIL /
52 BLOCKED / 0 NOT TESTED** of440: Firefox143/51/26, Chrome186/8/26.
These are real failed observations and have not been waived or relabeled.
Thirty-three matched observations compare one exact representative per shared
failure class against124 ON,124 OFF and the exact originally installed123 OFF.
Persistent correction and rapid-gesture failures reproduce on123. This does
not establish the result of every one of59 failed rows independently.

The private terminal renderer was corrected to anchor its multiline buffer;
old raw results are retained. A stronger placement oracle then found that a
nonzero rectangle can still cover already typed letters. In12 matched strict
placement checks, Kitty first/second word fails on allthree configurations;
Konsole first/second word passes on allthree. Tab is not sent when its visible
placement precondition fails. The new letter conversions and open/closed
five-gesture controls are separate from this presentation verdict.

Six primary GNOME/GTK Space cases compare two error classes in124 ON,124 OFF
and original123 OFF. Allthree configurations retain the same respective typo.
The correction targets fail; the observed option equality does not turn them
into passing autocorrection cases. All original runtime/source/configuration
and installed hashes are restored.

One additional presentation experiment passed7 source tests but failed the
strict actual Kitty placement proof. A local word epoch does not prove that
client coordinates follow the painted glyph. Exactly its two source files and
scope were restored, returning to the full-gate64/frozen65 source identity.
That policy is rejected and is not part of this release. See
[kde-terminal-hint-coordinate-map-2026-11-10.md](kde-terminal-hint-coordinate-map-2026-11-10.md).

This release's acceptance is the isolated letter-triggered conversion, existing
word-writer reuse, current-word mode/manual-return integration and the separately
verified restart action. It does not claim zero defects across Lay. The complete
machine-readable evidence, including raw failure statuses and skipped scopes,
is [release-1.0.124-verification.json](evidence/release-1.0.124-verification.json).
No canonical source failure is waived; the source failure ledger remains empty.

Public CI, identical annotated tags, stable release API and installed updater
checks must complete before publication is reported as successful. The primary
machine receives releases through its updater rather than forced installation.

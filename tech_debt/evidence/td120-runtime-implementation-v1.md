# TD-120 scoped runtime implementation evidence

Date: 2026-09-05. Base: `cc1e2207519801ca0f9b7c6963897b55953a7751`.
Scope: TD-120 ordinary-word suppression lifecycle and its bounded atomic
settlement only. TD-121, TD-122, release, installation and live mutation are
outside this change.

## Frozen focused test manifest before production edits

The pre-existing RED module was corrected before runtime edits: ordinary
fixtures now bind focus through `bind_focus_path`; the exact handoff fixture
uses separately constructed unbound engines and binds only at the intended
source/target steps; the candidate insertion optimization expects one commit
and no deletion; the closed duplicate result expects no guard. The open-result
duplicate case below freezes preservation and single consumption of an existing
valid guard.

Initial corrected RED/control tests:

- `td120_successful_manual_edit_full_erase_new_word_does_not_suppress`
- `td120_successful_manual_edit_full_token_erase_with_left_context_does_not_suppress`
- `td120_full_erase_and_identical_retype_is_a_fresh_word`
- `td120_partial_erase_and_layout_letter_punctuation_preserve_guard_once`
- `td120_accepted_candidate_with_trailing_boundary_does_not_protect_next_word`
- `td120_auto_undo_with_trailing_boundary_does_not_protect_next_word`
- `td120_rejected_output_does_not_newly_arm_suppression`
- `td120_duplicate_output_does_not_newly_arm_suppression`
- `td120_duplicate_open_result_preserves_existing_guard_once`
- `td120_exact_replay_suppression_survives_temporary_empty_tail_and_exact_revoke`

Additional implementation-gate tests are frozen by these names:

- `td120_active_manual_and_candidate_open_results_arm_current_word`
- `td120_committed_candidate_undo_and_bridge_open_results_arm_current_word`
- `td120_failed_and_noop_replacements_preserve_prior_guard_without_rearming`
- `td120_composition_cursor_full_erase_retires_current_word`
- `td120_partial_composition_edit_preserves_current_word_incarnation`
- `td120_space_enter_and_hard_boundary_retire_current_word`
- `td120_existing_handoff_rebinds_same_incarnation_and_rejects_old_owner`
- `td120_legacy_v1_is_typed_and_retains_compatibility_residual`
- `td120_exact_v2_admission_revoke_and_expiry_keep_transport_scope`
- `td120_atomic_no_conflict_commit_abort_and_duplicate_preserve_effect_counts`
- `td120_atomic_guard_drift_preserves_newer_arm_consume_and_revoke`
- `td120_atomic_equal_final_revisions_compare_against_base`
- `td120_atomic_sensitive_and_shared_exact_cancellation_reject_stale_settlement`
- `td120_atomic_foreign_owner_and_real_aba_callbacks_reject_stale_settlement`
- `td120_public_atomic_and_physical_double_shift_contracts_are_unchanged`
- `td120_backspace_after_tail_trim_preserves_until_the_tracked_token_is_empty`
- `td120_o6_a_former_word_guard_is_absent_before_new_frame_all_safety_profiles`
- `td120_o6_b_exact_space_frame_matrix_all_safety_profiles`

## Implemented mechanisms

- Local and shared suppression now hold one `AutocorrectSuppression` value:
  `CurrentWord`, `LegacyReplayV1`, `ExactReplay`, or `None`. The former shadow
  boolean plus optional-exact authority was removed.
- `CurrentWord` carries one incarnation, runtime owner lease and tracked
  open-token length; the active path remains in shared state. Successful active/committed manual and candidate
  edits, undo and typed bridge replacements arm only after output and tail
  synchronization. Rejected, no-op, failed and duplicate routes do not rearm.
- Ordinary printable continuation, composition cursor edits and committed-tail
  Backspace preserve the incarnation while the tracked token remains open.
  Full deletion, whitespace, Enter and hard punctuation retire only
  `CurrentWord`; layout-letter punctuation and both transport variants retain
  their existing semantics.
- The admitted existing focus handoff rebinds the same ordinary incarnation to
  the new path/lease. Shared state is authoritative for ordinary consumption,
  so a stale local mirror cannot consume or clear the new owner's guard.
- Shared suppression/handoff mutations advance `suppression_revision`. Atomic
  pending state captures the base revision, owner, bounded tail, epoch, focus
  receipt and local owner lease from the cloned snapshot. Settlement preserves
  a newer live suppression block on guard-only drift and rejects both stale
  engine and shared assignment on owner/tail drift, without applying deferred
  layout or learning actions.
- Public V1/V2 and atomic wire formats, exact source/path/epoch/deadline checks,
  exact revoke, terminal ownership and physical Double Shift ownership were not
  changed.

## Known residuals

No-argument `LegacyReplayV1` cannot bind before/after calls to a request, word,
phase or owner. Before-only failure, duplicate calls and delayed-after leakage
remain `KNOWN_RESIDUAL` for TD-122; this patch characterizes but does not repair
that protocol. Canonical context continuity remains TD-121.

## Verification status

Scoped static verification on the final working-tree source: explicit
`rustfmt --check` over the touched Rust files and `git diff --check` completed
cleanly. Static searches found one registration for each TD-120 test module,
no fabricated pending-state helper, and no former suppression shadow fields or
handoff helper names. These checks do not establish compilation or behavior.

Parent ran an INTERMEDIATE snapshot while implementation was in progress:
compilation2.44s, ten tests8 PASS/2 FAIL. See
[concrete parent feedback](td120-intermediate-feedback.md): typed retirement
must not take/drop ExactReplay/V1, and the open-duplicate fixture's first edit
was refused. These must be fixed before final verification. Parent also
registered the independently written O6 module; do not add a duplicate mod.
An expanded parent snapshot later reported 28/28 focused TD-120 tests PASS,
including the O6 exact-Space profile matrix. A full-bin snapshot reported
309/310 PASS: the single failure was a clone-isolation assertion that still
expected the now-bound common fixture to have no active path. The assertion now
compares the live state with its real pre-clone owner baseline. Both snapshots
predate the final source edits and are not final source parity or independent
review.

The two initial findings and the expanded-v3 fixture findings above have been
addressed in source: non-ordinary retirement no longer takes transport state;
open-result producer fixtures use authorized delete-plus-replace transitions;
the hard-boundary fixture no longer uses a physical layout-letter key; and the
foreign/ABA callback test now produces a real atomic frame before exercising
real focus callbacks and settlement. The bounded-tail test now also covers an
append after arming at the trim limit, so tracked length describes the retained
token. These corrections are not claimed PASS until the parent reruns the exact
final source.

The scoped implementation agent ran no local or remote Cargo command. Final
compile, focused-test, changed-gate, runtime and performance verification of the
exact final source remain owned by the parent build/release task.

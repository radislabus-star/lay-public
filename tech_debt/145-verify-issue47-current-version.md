# TD-145 — Verify public issue #47 against current 1.0.87

Status: SOURCE_VERIFIED; NATIVE_STARTUP_NEEDS_REPORTER_RETEST.
Priority: P1. Source baseline: `62ac2e1cac3f1feb3d597f83037342bac84c6af6`;
runtime source `50d314b382c0383764ea8e738a2d2cd274c2628e`, installed 1.0.87.
Issue: https://github.com/radislabus-star/lay-public/issues/47

## Complaint and first failed transitions

The report combines three observations: pure-uinput Double Shift did nothing;
Space layout conversion required both Auto Replace and Typing Assist; later,
version 1.0.74 needed extension disable/enable after login and Space was unstable.
The reporter's last edit is 2026-09-26. There is no reporter result for 1.0.87.
Current source behavior and post-login behavior on the reporter's machine must
have separate verdicts.

Historical maintainer comments attribute the old Double Shift failure to
ordinary configured uinput being treated as IME-delegated replay, requiring an
unavailable IME tail lease. They attribute the old Space failure to gating the
boundary worker and contextual layout policy with unrelated correction toggles.
These historical diagnoses are attributed, not a new old-version reproduction.
The first failed transition for the remaining
post-login complaint is UNKNOWN: no fresh startup journal or client observation
exists for the reporter's current installation. Do not infer an ACL, bootstrap,
Firefox transport or ranking failure from service state alone.

## Current production evidence

- `manual_trigger_runtime/ime.rs::daemon_manual_toggle_plan(false, false)` uses
  ConfiguredBackend + PhysicalGrab. Only the explicit delegated route requires
  the IME tail lease in `correction_runtime.rs`. No new route is introduced.
- `config_runtime.rs::daemon_boundary_correction_active` permits any of the
  three toggles independently, only when the daemon owns text mutation.
- `typing_context/pipeline.rs::typing_assist_pipeline_for_context_with_layout`
  uses auto_switch_layout independently of auto_replace for contextual rules.
- `context_admission/adapter.rs` preserves an observer for an initially unset
  IBus GlobalEngine. This is an implemented recovery, not proof that it caused
  the reporter's post-login failure.
- TD-144 maintains late device access. Its real temporary-file permission
  transition proves the recovery helper; it does not establish the reporter's
  kernel/ACL behavior or a login result on ARM64.

The old comment commits `89ca0ab2` and `bc1b0dbf` are NOT ancestors of this
checkout. Therefore presence of their fixes is verified in current source and
tests rather than claimed from historical commit ancestry.

## Measurements and exact scope

Fresh diagnostic: 12 PASS / 0 FAIL / 0 BLOCKED.
`/home/ubu/.cache/lay/development/issue47-current-20261016/CORE87.json`.
Exact remote release87 CLI SHA-256:
`8fe20d19bbe2a55227038a59312bf5f613f7ed032f2e56e4e723cd67ece42534`.
The public attached config is used with three profiles: all toggles off,
Auto Switch alone, and all three toggles on. Each profile checks `ghbdtn `,
`руддщ `, clean `привет ` and clean `hello `. Auto Switch converts both
directions with the other toggles off; clean controls stay unchanged.
Nanda is explicitly disabled. This does not certify Nanda-enabled combinations.
Each case uses the existing test_lanes filesystem/network sandbox with fresh
data. No live data is cleaned, no keys are injected and no services are restarted.

This CLI calls correction_core with a frameless request. It does not exercise
the daemon's deferred worker, text output, GNOME login or any native field.
CLI elapsed times include process startup and are not typing latency.

Reused current source341/341 PASS:
`/home/ubu/.cache/lay/development/run-7ggr8s_q/RESULT.json`.
The detailed executed-test receipt is
`/home/e/projects/lay-development-runner/run-EXuS7u/tests/SUMMARY.json`;
its four relevant test identities and both detailed-receipt hashes are retained
in `/home/ubu/.cache/lay/development/issue47-current-20261016/EXECUTED-TESTS.json`.
The current receipt contains these exact production-contract regressions:
- `config_runtime::tests::direct_ime_owners_disable_daemon_text_decisions`
- `manual_trigger_runtime::ime::tests::daemon_pending_auto_undo_keeps_physical_grab_ownership_without_consuming_it`
- `tests::typing_assist_rules::exact_layout::typing_assist_auto_replace_off_keeps_layout_only_rules`

The older source86 868/868 receipt contains
`context_admission::adapter::tests::no_global_engine_bootstrap_keeps_observer_for_later_verified_lay_profile`:
`/home/ubu/.cache/lay/development/run-0tmq3hj2/RESULT.json`.
Only the unchanged IME source proof may be reused; this is not an 868-test run
of 1.0.87. All596 Rust files outside the daemon subtree match that earlier tested snapshot;
`/home/ubu/.cache/lay/development/issue47-current-20261016/UNCHANGED-SOURCE.json`.
The native browser matrix stays historical349/51/32/432.

## Options and decision

1. 9/10 — retain the verified implementation, report exact current evidence and
   request the post-login failure on the current version before any toggle or
   restart. Keep issue #47 open for that remaining observation. Selected.
2. 7/10 — add a focused native uinput/control run on the remote test desktop.
   It could confirm that desktop and narrow current source behavior; it cannot
   reproduce an ARM64 Ubuntu login or replace the reporter's startup evidence.
   Reserve it for a new causal hypothesis or a current native reproduction.
3. 5/10 — close the issue on the corrected original source mechanisms alone.
   This resolves the old two source defects but leaves the reporter's last
   post-login observation unaccounted for, so it is not selected.

No production edit, model change, authority change, new version or installation
is required by the observed evidence. No current failing transition is established
for a new runtime patch. If a fresh failure arrives, first classify device access,
trigger observation, bootstrap admission, boundary decision and client output;
repair only the first demonstrated failure with the existing owner.

## Completion and follow-up

Independent review pass1: 9/10, fresh reviewer
`/root/issue47_current_source_audit_review`. No material blocker; optional
precision edits applied to the reused-source scope and detailed-receipt links.
One pass was sufficient. Published a precise issue update with three read-only
diagnostic commands:
`~/.local/bin/lay --version`, `~/.local/bin/lay-runtime-control status`, and
the boot's daemon journal filtered to the new metadata-only `lay:` device-access
messages, with at most20 matching lines. Raw debug journals can contain user
text and must not be requested. Run immediately after login and before extension
toggling. An empty filtered journal is a valid observation. Existing user
text/config is not requested.
Keep the issue open pending that evidence; do not advertise a new native PASS.

Issue update published and read back exactly:
https://github.com/radislabus-star/lay-public/issues/47#issuecomment-6205680011
GitHub state remains OPEN, verified after posting. Source audit complete;
post-login/native follow-up stays explicitly unverified.

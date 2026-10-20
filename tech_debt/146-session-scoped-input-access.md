# TD-146 — Session-scoped device access (GitHub #48)

Status: DONE_SOURCE; publication gates PASS. Priority: P1.
Baseline: `db5cc78336d2af19a6ae6b2f922bcf376b56f4f7`, installed1.0.87.
Issue: https://github.com/radislabus-star/lay-public/issues/48

## Cause, boundary and options

`install.sh` unconditionally requires persistent membership in `input` and
creates the old uinput rule only when the current process lacks write access.
This both needs a new login for new group membership and can omit durable
configuration when a transient permission already exists. The failed transition
is installer provisioning, before the existing daemon opens evdev/uinput.
The existing installer owns provisioning; TD-144 owns late device readiness.
No input owner, detector, executor or delivery route changes here (C01/C07/C10).

1. **9/10, selected:** install an early udev `uaccess` rule for event devices
   and uinput on every run, plus a small uinput modules-load configuration.
   Apply rules to both subsystems. Reuse the daemon's late-access recovery.
2. **5/10:** keep input-group membership and only improve the relogin warning.
   This retains the reported permanent permission expansion.
3. **3/10, separate discussion:** privileged system service/device broker with
   logind TakeDevice/ReleaseDevice. This changes service ownership and device
   lifecycle, beyond a minimal installer correction.

Upstream `73-seat-late.rules.in` runs the `uaccess` builtin after tags are set;
therefore use `/etc/udev/rules.d/70-lay-input.rules`, before73, not99.
Reference: https://github.com/systemd/systemd/blob/main/rules.d/73-seat-late.rules.in
The requested event rule grants access to all matching evdev nodes; the daemon
still selects keyboards/pointers later. This is broader than its discovery filter.
ACLs govern new device opens. Already-open evdev descriptors are not revoked
by an ACL change; do not advertise session switching as a device-fd revocation
guarantee. Existing input-group membership is never removed automatically.
Access depends on the device's active logind seat; ordinary uinput belongs to
seat0. Headless, inactive and multiseat sessions have no universal guarantee.

## Minimal implementation and consequences

- Put provisioning in one installer helper called by `install.sh`. It installs
  two packaged small files, always, and never runs usermod/chgrp/chmod on devices.
  A filesystem-root override, following the existing uninstall convention, is
  for isolated tests; fake privileged/device commands prevent host side effects.
- Remove only the exact old Lay-generated99 rule. Preserve unknown custom
  contents and warn during install AND uninstall/purge. Uninstall removes the
  new Lay-owned rule/module file and keeps group membership. Do not alter other
  udev rules or unrelated packages. With explicit module loading, static_node
  is unnecessary and is not included in the new rule.
  Unknown changed contents at the new70/module paths stop setup before either
  file is changed; uninstall preserves them too. No destructive overwrite.
- Load uinput, reload rules, trigger input event nodes and misc/uinput, then use
  a bounded udev settle. Reload/trigger failures must return failure; module
  unavailability is reported honestly as activation failure, never converted to
  a claim of readiness. A nonzero settle result also fails activation.
  Distinguish installed configuration from observed current device access.
- No inference/ranking/learning/cache identity changes, no extra hot-path work,
  threads, queue, retry of text, or runtime RSS/typing-latency change. One-shot
  installer commands only. No model fit, weight change, IBus restart or source
  migration. Existing group users remain compatible; non-logind/headless use
  may need an administrator-managed access policy rather than a hidden fallback.
- Failure leaves installed permission files available for repair/retry; report
  the failed activation command. Rollback is removal of the two new files and
  restoration of a backed-up old rule if migration removed it. Do not remove
  another application's group access or promise a successful ACL on an inactive
  seat. The local final runtime update must not invoke the full legacy installer.

## Proof and completion gates

Remote-only guarded tests of the real helper with a temporary filesystem and
fake sudo/modprobe/udevadm: fresh setup; repeat setup; currently accessible
devices still get durable rules; exact old-rule migration; custom-rule retention;
module failure remains visible; reload/each trigger failure is nonzero; both
subsystems and bounded settle; no user/group/device-mode mutation; uninstall
cleans only Lay artifacts and keeps unrelated configuration. Verify packaged
ordering/tags and installer entrypoint, not a duplicate helper implementation.
One real install.sh invocation must stop before later stages on helper failure.
Run existing public-install/update/uninstall regressions and shell syntax checks.
Keep an intentional violating-rule/entrypoint test as controlled RED if useful;
do not call a mocked ACL a real logind acceptance.

One fresh no-context implementation review after proof, score1–10, at most two
passes. Canonical remote architecture refresh/check, then commit/push and public
reply/closure with exact scope. No local permission changes for testing.

Native seat ACL acquisition/revocation, cold login and all physical clients:
NOT_TESTED until measured separately. Browser matrix remains349/51/32/432;
typing latency NOT_MEASURED. This task changes installer policy, not runtime
authority. Final user-requested binary installation is a separate transaction.

Plan reviewer: `/root/late_managed_release_plan_review_20261013`, source-only,
8/10. The agent thread was reused because blank-context spawning hit the thread
limit; this review used only the new task/source, not an implementation narrative.

Implementation review1:7/10 (`/root/issue46_feature_plan_review`, independent
new-source scope in a reused thread because of the same thread limit). Two P2:
protect changed contents at new managed paths too, and make the real-installer
negative safe even if helper failures start being ignored. Grouped repair adds
preflight/content-aware removal, install/uninstall/purge tests, fail-closed later
session/network command stubs and an absent session bus/runtime directory.

Final remote proof:11/11 access tests +8/8 existing public-build tests PASS;
the existing public install/update/uninstall shell suite PASS. Receipt:
`/home/ubu/.cache/lay/development/td146-repair-h0w7jdwc/RESULT.json`,
source archive SHA-256 `2113caf90f9a9de63f9edf3d6cbbf5812c3c6b53ada553e2add6f9edc978edc0`.
Two controlled violations correctly FAIL: remove uaccess tags; ignore helper
failure in the real installer. The latter stays inside fake later-stage commands
and an absent session bus. These are source/mock proofs, not actual ACL tests.
An earlier multi-argument bash -n only parsed its first file; a separate per-file
syntax check is required before publication and is recorded separately.

Final implementation review2/2:9/10; no unresolved P1/P2. Same independent
reviewer, no tests executed by reviewer. Runtime Rust, models, configuration,
input delivery, user groups and installed1.0.87 are unchanged by this patch.

Canonical remote refresh/check PASS (48.02s):
`/home/ubu/.cache/lay/development/td146-final-graph-20261016/GRAPH_RESULT.json`.
All four shell files were then individually parsed and source hashes verified:
`td146-final-graph-20261016/SHELL_SYNTAX.json`, PASS. The earlier lock-busy
syntax attempt did not run; it was repeated only after the graph lease ended.


Final release closure —2026-10-16: functional source77af7ade, full3211/3211,
CI38916364078 +Security38916364185 firstattemptSUCCESS, independent review9/10.
Installed matching CLI/daemon/IME/panel1.0.88;6actualpayloads match native-tested
e34 bytes. Whole432=349P50F33B and normal optional80=58P18F4blockedpreconditions
remain explicit, no all-client acceptance claim. GlobalIBus/L1.1/L3, models,
settings/sources/mode unchanged; no localfunctionalkeys. Rollback/procSHA evidence:
/home/ubu/.cache/lay/development/td153-install88-20261016/INSTALL.json.
All old negative receipts and each card's narrower source/ACL/ARM scopes remain.

## Startup diagnostic extension — 2026-10-20

The existing `startup_runtime::maintain_input_devices` retry loop remains the
only device recovery owner. Its first keyboard/uinput failure starts the sibling
`lay-input-access --show-if-needed` once per daemon invocation, asynchronously;
detect-only mode does not start it. The loop reaps the child without waiting for
the window. The obsolete input-group error advice is replaced with the shared
diagnostic entrypoint. No input listener, routing fallback or retry timer is added.

`scripts/lay-input-access.py` reads sysfs capabilities and opens/closes devices
without consuming events. It distinguishes permission failure, missing supported
keyboard, unreadable discovery metadata and unavailable uinput. The three existing
feedback-device exclusions match the daemon. READY means device-open preflight,
not a physical gesture, IME completion or client-edit acceptance.

Only a deliberate **Настроить доступ…** button invokes the existing TD-146
provisioner via normal Polkit authorization. The helper and its three resources,
including the exact legacy-rule comparison file, are shipped inside managed
libexec and removed by existing managed-runtime uninstall. The actual privileged
operation receives TERM after 30 seconds and KILL after a further five-second
grace period after authorization. Closing its window hides
it while it supervises/reaps the child and retains its singleton lock. Authorization
itself remains under the user's normal confirmation/cancellation. Every completed
attempt probes again; exit zero does not imply access. No groups/device modes,
IME routes, services or global desktop environment are changed by the diagnostic.

Remote source proof: 320/320 daemon checks; 29 diagnostic/existing access checks;
27 release/menu checks; packaging and public install/update/uninstall fixtures;
7/7 isolated real GTK/Qt tests, including both close paths during a pending child
and a real Qt menu re-open before the existing status timer. Child failures are
reported before re-launch as well as by the timer. Device disappearance and I/O
failure do not become permission diagnoses. These fixture operations never
acquire privileges.

Read-only real-device preflight reports NO_KEYBOARD, zero supported keyboards;
an isolated GTK window shows that result and disables provisioning. The remote
VNC desktop only exposes excluded feedback typing devices, so changing permission
cannot repair this topology. Actual seat ACL activation, automatic startup popup
in a user's session, physical Double Shift, IME Tab and fresh native browser
acceptance remain NOT_TESTED here. Runtime authority remains unchanged.

Evidence root:
`/home/ubu/.cache/lay/development/browser-final-fixes-20261018/input-access-recovery-20261020/`;
daemon result: `/home/ubu/.cache/lay/development/run-0nlc0wf1/RESULT.json`;
remaining checks: `revision3/checks.json`, `revision3/regression-proof.json`,
`revision2/qt-entrypoint.json`,
`revision2/gtk-real-preflight.json` and `revision2-preflight.json` under that root.
Review and graph-refresh closure are recorded in
`docs/development/input-access-recovery-2026-10-20.md`. No installation/publication
is established by these receipts. Rollback removes the added startup/menu hooks
and managed diagnostic payload; it does not undo administrator rules automatically.

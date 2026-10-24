# Lay repair and release plan — 2026-10-24

Owner: the current session, working directly without agents or Harness2.
Source: `codex/ime-muted-hint-20261019`, initially `a8fbda3c`, Lay 1.0.95.
The user authorized fixing the build and all three open public issues, testing,
publishing the next version, and replying to reporters without further questions.

## Ordered work and evidence

1. **SOURCE VERIFIED — installer build output.** Preserve the complete compiler
   log and original exit/cancellation behavior. Show useful progress and a
   warning count on success; expose full diagnostics on failure. Extend the
   actual public-entrypoint regression tests, prove RED then GREEN. Do not
   silence Rust diagnostics or remove research code to hide warnings.
   RED: two reproduced failures; GREEN: 10/10 public-entrypoint tests. Remote
   receipts: `weekend-build-wrapper-{red,green}.log`. Real build is gate 8.
2. **VERIFIED — dependency security.** Only
   rustls-webpki 0.103.15 → 0.103.16, webpki-roots 1.0.7 → 1.0.10 and
   tokio 1.52.1 → 1.53.3 changed. The real strict audit exits zero with no
   vulnerabilities or unsound advisories. Receipts:
   `~/.cache/lay/development/release-1.0.95-20261024/security-{before,after}-fix.json`.
   Build and compatibility checks remain required; accepted native receipts
   for the previous lockfile do not identify the new compiled bytes.
3. **VERIFIED — restore truthful full test gates.** CI 40089564155 failed
   inventory drift before execution. Remote discovery found 3560 tests versus
   3412 in the old manifest. Regenerate from actual discovery, preserve the
   zero-known-failure policy, run all lanes, fix failing shared mechanisms,
   and retain a fresh observation. Do not whitelist failures or drop tests.
4. **VERIFIED — KDE tray on retained runtime.** Distinguish Lay's exported menu from a responsive
   Plasma panel. Check the actual popup and action, not only DBus GetLayout.
   Preserve active applications and the existing desktop/input owners.
5. **PATCHED, TESTED — issue #69, login ordering.** Reproduce early daemon/L3 startup
   and inspect upgrade symlink migration. Repair the existing service owner;
   preserve remote/headless behavior and input delivery contracts.
6. **PATCHED, TESTED — issue #63, GNOME 51 extension backend.** Check the existing
   backend adapter and supported GNOME versions, add regression coverage for
   old and new APIs, and correct compatibility without a new delivery route.
7. **PARTIAL, RESIDUAL OPEN — issue #47, Double Shift and settings.** Read all comments and
   the attached configuration. Trace the real settings/startup/input path,
   reproduce the first failed transition, and fix its shared cause. Old
   reporter output alone is not current-version proof.
8. **DONE, NATIVE FAILURES RETAINED — final gates.** Run full Rust correctness/package lanes, Python
   and extension checks, strict audit and MSRV checks. Actually build all ten
   installed release binaries with the installer's feature/lockfile profile.
   Check affected native clients and updater success/error/cancellation.
   Record latency and PASS/FAIL/BLOCKED/NOT_TESTED honestly. A platform not
   available for native testing remains explicitly untested.
9. **DONE — publish and communicate.** Prepare 1.0.96 after the gates,
   commit/push, create a new tag/release without changing 1.0.95, verify actual
   GitHub CI/Security and stable-update discovery from GNOME and KDE. Reply
   to each reporter with concrete changes and evidence; close only the issue
   scope actually fixed. Update this plan and the handoff with receipts.

## Boundaries

- No questions are pending. No agents, Harness2, broad cleanup or replacement
  architecture. No advisory suppression, weakened verifier or word exceptions.
- Do not interrupt the user's local 1.0.95 update or manually replace the
  running GNOME installation. Updates remain available through its menu.
- Use the existing remote development workspace and resource/Cargo guards.
- Keep one source of truth here; mark completed steps with precise receipts.
  Historical browser numbers do not count as tests of new bytes.

## Source gate progress

Inventory now contains 3563 tests (3475 correctness, 36 package, 11 performance,
41 ignored). Full execution is `weekend-all-v1` in the remote install-control
root. The zero-failure policy is preserved. Source inspection corrected the
first hypothesis about 15 old failures: KDE prefix policy leaked globally; the
original pair fixture is retained and that desktop scope is now explicit.
First matching-receiver receipt order and a deep fixture stack overflow are
separate proven repairs. No new binary has been installed or accepted yet.

## Desktop and gate milestone — 2026-10-24

The actual tray click opens the menu and shows Lay enabled. Plasma was blocked
in QSemaphore::acquire → QRasterPlatformPixmap → Plasma::Theme::findInCache;
its UI did not service DBus requests. Earlier pthread creation errors occurred
under the former 512-task session limit. The private desktop now has TasksMax
1024. Re-executing only Plasma in the same PID preserves the supervisor
(wait -n would tear down the desktop on child exit). GDB did not detach cleanly;
its own stopped tracee was resumed with SIGCONT. Failed attempt and final
continuation are both retained. UI replies and the real menu popup verify
recovery, with all 18 protected processes unchanged. Remote receipts:
`~/.cache/lay/development/weekend-panel-recovery/`; local frames:
`~/.cache/lay/development/release-1.0.95-20261024/panel-{after,menu}.png`.
This menu is still the retained 1.0.94.1 runtime, not native acceptance of 1.0.96.

Full source pass v1: 3506 PASS / 3 FAIL. The three were test fixture minting
outside tests/common and two stale runtime-controller source assertions. The
monopoly guard remains unchanged; the fixture moved to tests/common. Source
assertions preserve the same manager/stop/restart ordering, with a new actual
controller regression for a stopped daemon. Python: 417 tests, zero failures,
10 skipped. Old controller hash pin updated after its private-manager-bus
change was checked by those tests. The L3 cleanup signal test uses the actual
dedicated-20cpu envelope; no guard mock or profile weakening.

The remote mirror Git base was fd3c19fa despite current source files. Its index
and old HEAD are saved, then only Git metadata was aligned to public a8fbda3c
(no working-tree reset). Canon and 50-pass audit now pass against the actual
baseline. Lints exposed two unused imports and a redundant deadline wrapper;
these are removed. Strict diagnostic parsing still rejects failed builds and
now prints the reasons before temporary log cleanup. Current run:
`weekend-next-gates-r7.log`, controller `weekend-next-gates.sh`, remote
install-control root. Default lint profile PASS; remaining gates pending.

## Final source gate — 2026-10-24T16:19Z

`weekend-next-gates-r9.log`: 3511/3511 correctness/package PASS, zero
infrastructure or semantic failures (daemon 328, IME 1085). The complete
inventory is 3563: 3475 correctness, 36 package, 11 performance, 41 ignored.
All 11 performance/resource gates pass. Both Rust 1.88 MSRV configurations,
both strict lint profiles, canon, audit50, 417 Python tests (10 skips), public
installer/update/error/cancellation tests and JS boundary tests pass. Strict
RustSec has zero vulnerabilities. Ignored/skipped tests are not counted PASS.
The zero-failure ledger now binds the fresh observation at
`tech_debt/evidence/release-1.0.96-test-observation.json` (SHA-256
`88bf41df1439225d08b225dcb9e1d762230799a551b6a8003acd509fddcef1d7`).

All ten release binaries were actually built (222 seconds) and retained under
remote `release-1.0.96-build/`. Source closure:
`649068b8d6feb3bad3da435bfca4281ac8c6f8e4a2ddf2ffe9f727e4d068a585`.
Installation to private KDE succeeded; running daemon/IME hashes match build
artifacts, ten installed programs match, IBus/desktop and protected applications
were preserved. The version check caught the previous hardcoded IME development
suffix `.1`. Only that display label is now changed to the package version.
`release-1.0.96-final-build/` rebuilds the complete installer set. Its proof
reconstructs the original aggregate source hash with exactly args.rs and the
generated architecture receipt restored in memory; it does not claim a second
functional run. Native acceptance will use only the final label's bytes.

The English guard resource repair is measured separately in
`docs/architecture/exact-english-guard-memory-2026-10-24.md`.
Remote GNOME is 42.9 (unsupported by this extension); native GNOME 51/Orange Pi
acceptance is unavailable here and remains NOT_TESTED. No local GNOME
installation or service was changed by this work.

Next: final KDE relabel install, unchanged r18 native browser actor (220 each),
owned GTK/Qt/Kitty/Konsole checks and issue47 configuration controls; then actual
public CI, stable96 publication, installed updater checks and reporter replies.

## Native and publication preparation — 2026-10-24

Final label build/deployment: CLI, daemon and IME all1.0.96. Byte verification
succeeds; no global IBus/desktop/model changes. Final worker closure
`c76a3e93b9ae764c09aa884dd173667191037cb499ea20e7bf51940c91e88365`.
Local content and executable bits are identical. The local closure differs
only because the archive data filter removed group-write bits on29files;
both actual hashes and exact paths are recorded in the verification JSON.
Source bytes are not rebuilt or source gates weakened to conceal this metadata
variation. The current local architecture receipt check is PASS.

Frozenr18 Chrome completed220:168PASS/10FAIL/42BLOCKED. Firefox default setup
has one immutable pre-input observer BLOCKED; no grid is filled by it.
Mozilla Platform.cpp confirms the process-local accessibility activation.
A separate Firefox fixture inherits GNOME_ACCESSIBILITY=1 only in its own
processes, keeps KDE/IM module selection and global properties unchanged:
81PASS/103FAIL/36BLOCKED of220. Native Firefox includes real Double Shift
failures; Chrome failures are Space3 and pair7. These remain known KDE
compatibility limits, not proven regressions caused by this patch and not
claimed repaired by source PASS. Do not repeat the recorded rejected key-time
Space experiment or weaken authorization. GTK/Qt/terminals and issue47's
isolated twelve configuration controls now run using existing scripts.
Public source CI precedes the stable tag; no release96 is published yet.

## Final gate and remaining issue scope — 2026-10-24

Source commit26d5ca24e2534baea04f05c215f0f858229480b9 is pushed to private
origin and public main. CI40113293456 and Security40113293422 both SUCCESS.
The CI log includes the full optimized all-bin build (7m16s), not just a check.
Later publication/evidence changes are documentation only and explicitly skip
CI; the production source matches this verified commit. Real compiled ten-bin
installation, runtime identities and all native results are retained.

GTK/Qt25/25. Kitty/Konsole20PASS/2FAIL; both RU punctuation failures reproduce
without Lay using ordinary xkb engines. Raw FAIL stays immutable. Terminal
screenshots were reviewed: typed prefix plain; only proposed suffix white.
Issue47's attached settings:12/12 independent flag/core controls PASS.
No learning cleanup or model installation was performed.

Actual tray menu96 and enabled Lay were visually verified. Its disabled auto
control came from mismatched private-session PATH: the daemon used the existing
real systemctl wrapper (unset private desktop bus, exec /usr/bin/systemctl),
the tray did not. The owned transient tray unit now inherits that same PATH.
Real timer status and a second actual menu click show enabled/actionable auto
update. No product code, daemon/IME, desktop bus or global environment changed.
Remote receipts: release96-tray-manager-env and release96-final-ui-manager-env.

Speed is recorded per method in evidence/release-1.0.96-native.json: successful
single Double Shift, Chrome204 samples median58.03/p9564.49ms; Firefox138
samples148.82/190.27ms. Chrome five-gesture DOM events260samples44.62/55.01ms,
max clock mapping uncertainty2.928ms. Failed rows excluded explicitly; these
are not a before/after speedup comparison.

Publication must close #69 and #63 after the release and actual installed
updater checks. #47 receives the verified startup/flag result and stays OPEN
for the unverified intermittent GNOME Space residual. Closing that residual
would exceed the evidence. No current supported native GNOME/OrangePi target
or exact current-version reporter reproduction is available. The remaining
KDE mechanisms and no-loop constraints are mapped in
tech_debt/kde-native-receipts-followup-2026-10-24.md.

## Publication finish — 2026-10-24

Stablev1.0.96 published2026-10-24T17:35:01Z, tag commit
50fbc3b2cb906adc43c38e963c87539d2356880e. Source CI/Security commit26d5ca24
is identical for all production inputs; the tag's six-file delta is evidence/
documentation only. Oldv1.0.95 tag was not moved. Both private origin and public
main/tag are pushed. GitHub latest API returns96,draft=false,prerelease=false.

Actual installed updater checks both exit0, using the default public API:
local GNOME installed95 -> stable96 AVAILABLE; remote KDE installed96 ->
stable96 CURRENT. Local installed updater resolves to .local/lib/lay/update.sh;
no manual local installation/restart. Remote check uses the actual live tray
environment. Remote auto-update timer is enabled. The private tray PATH fix
also has a persistent owned service drop-in; the existing systemctl wrapper in
.cache/lay/kde-live-20261019/bin is an active runtime dependency, not junk.
Cargo cache after all builds3941822464bytes, below12884901888 budget.

Public verified replies:
- #69 CLOSED: https://github.com/radislabus-star/lay-public/issues/69#issuecomment-6387397577
- #63 CLOSED: https://github.com/radislabus-star/lay-public/issues/63#issuecomment-6387398163
- #47 OPEN: https://github.com/radislabus-star/lay-public/issues/47#issuecomment-6387398641

The remaining GNOME Space residual is not claimed fixed. Its current-version
exact reproduction/native supported-GNOME observation is missing; the reporter
was asked in that issue for an example and app/field, without asking this user
to return from the weekend. No further source/input change was made merely to
obtain a green native grid. Remaining KDE mechanisms have a concrete P1 map.

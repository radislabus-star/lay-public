# Installer blocker #70 — install supported polkit binaries on apt systems

Priority: P0 installer blocker. State: SOURCE_AND_BUILD_GREEN_PUBLICATION_PENDING. Public issue:
https://github.com/radislabus-star/lay-public/issues/70 (2026-10-26).
Execute immediately after the in-progress1.0.98 publication; preserve its
already verified runtime/source closure while collecting final native proof.

## First failure and owner

`install.sh:base_packages_for_pm(apt)` hardcodes transitional `policykit-1`.
On Ubuntu26.04 the binary package has no installation candidate, although the
real daemon and privilege helper remain available. `apt-get install` exits100
before the remaining installer steps. This is package selection, not an IME,
input-permission or runtime delivery defect. Merely substituting `polkitd`
would omit the `pkexec` helper used by Lay's interactive permission flow.

Primary compatibility evidence: Ubuntu source/binary package listings:
https://packages.ubuntu.com/source/jammy-updates/policykit-1
https://packages.ubuntu.com/jammy-updates/policykit-1
https://packages.ubuntu.com/search?arch=any&keywords=policykit-1&searchon=sourcenames&suite=resolute
The22.04 transitional package already depends on `pkexec` and `polkitd`;
the26.04 source builds both real packages without the transitional binary.
Do not infer architecture-specific package availability from a source name.

## Alternatives

1. Require the two actual binary packages `pkexec polkitd` for apt:9/10,
   recommended. Minimal removal of obsolete indirection; explicit helper and
   daemon requirements; no distro/version branching or new installer owner.
2. Query apt candidates then fall back to legacy name:6/10. More subprocesses,
   stale-list and no-candidate branches; unjustified for currently supported
   Ubuntu22.04+ and current Debian when both binary packages exist.
3. Select by distro release number:3/10. Breaks derivatives and backports,
   requires ongoing version exceptions and does not prove actual capability.

## Minimal implementation and proof

Change only the apt base package list. Preserve other package managers, update
mode, sudo/pkexec flow, reboot semantics and all runtime routes. Add a meaningful
regression to existing platform checks: apt output must include both daemon and
helper, omit the transitional package; existing pacman/dnf/yum/ostree outputs
must stay unchanged. A hermetic apt mock must reject `policykit-1` yet accept the
required real packages, reproducing the reporter's first failed transition.
Use existing installer test patterns rather than a second package resolver.

Run syntax and existing installer/public-release/update tests. Inspect actual
apt candidate availability on the owned remote machine read-only; do not perform
a gratuitous system upgrade. Publish a new version only after required release
checks; verify the actual installed updater discovers it, then reply to/close
#70 with the concrete release and verification scope. A mock/platform PASS is
not a full clean Ubuntu26.04 installation receipt; state that limit explicitly.

2026-10-26: apt exactRED then public-build11/11GREEN; final remote source and
package gates,all10optimized binaries PASS. Ubuntu22.04 apt candidate read-only
confirms both real components. CleanUbuntu26.04 installNOT_TESTED. Final public
CI/publication/updater discovery/reply to70 stillPENDING; do not close from tests.

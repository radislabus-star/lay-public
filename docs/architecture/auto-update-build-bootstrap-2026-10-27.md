# Old automatic-update budgets must be refreshed before compilation

Owner: existing install.sh unit installation and systemd user manager. Stage6,
candidate1.0.103. No new input owner, replay route, model or background helper.

## First failed transition and the smaller solution

An already running old lay-update script downloads a fresh tag and invokes its
install.sh --update. It never replaces itself by executing the new update.sh.
The fresh installer previously copied the new auto-update unit only AFTER
compilation. Thus an old running unit kept MemoryHigh3G while the compiler
needed more of its existing4G hard budget. Source99 removed the new unit soft
limit, but that alone cannot bootstrap a build still inside the old policy.

Recommended9/10: move the EXISTING five-line unit copy/daemon-reload block
before compilation, preserving timer enabled state. No service restart or new
policy interpreter. A new own-unit memory-setting helper5/10 would add checks
and code for behavior already supplied by systemd. Merely update the unit
after compilation2/10 retains the failed transition.

## Measured control, not compiler completion

On26Oct2026 at21:06:21UTC a temporary owned sleep service used64MiB hard and
32MiB soft limits,0.1CPU/4tasks. Replacing its unit with the same hard/CPU/task
limits and no soft limit, then daemon-reload, changed kernel memory.high to
max WITHOUT changing PID3451686, memory.max, cpu.max or pids.max. It was stopped
and its own unit removed in finally; no unrelated service was restarted.
Receipt:CROOT/release103-unit-reload-probe.json. This proves live resource
application, not a compiler, updater completion or32GiB workload acceptance.

Isolated actual install.sh --update with an old3G/4G budget fixture:
old ordering RED1 (compiler entry still sees legacy soft limit); moved existing
block GREEN1. Hard4G,swap256MiB andCPU1 remain required, models/user data and
IME initial setup remain untouched by this fixture. No production source or
installed runtime was changed. Receipts:CROOT/release103-install-ordering/{result.json,red.log,green.log,candidate-install.sh}.

## Promotion protocol

First finish102 full native/publication/updater. Then implement only this move
and the meaningful old-budget regression in the existing test module; update
its old successful-order assertion to the new admitted order. Preserve build
failure-before-binary/updater replacement, timer choice and learning data.
Full source/performance/build/CI plus an actual remote102->103 automatic apply
are required. A check/no-op, mock build or manual candidate installation cannot
be reported as that automatic completion. Record installed versus loaded bytes.

The existing updater deliberately leaves activeIME running. Read-only local
inventory found loadedIME1.0.96/PID3037150 after files100; do not call installed
version the runningIME version or issue an unauthorized local restart. This
policy move does not silently add unsafe idle-poll+SIGTERM reload behavior.
Any deferred runtime activation remains separate from automatic file/build
completion and current physical native acceptance.

CROOT=/home/e/projects/lay-development-runner/kde-install-control-20261020.
CACHE103=/home/ubu/.cache/lay/development/release-1.0.103-20261027.

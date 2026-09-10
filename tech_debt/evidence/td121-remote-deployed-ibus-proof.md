# TD-121 remote deployed-IBus private transport proof

Status: `PRIVATE_EXACT_TRANSPORT_PASS`
Date: 2026-09-06
Scope: remote-only private IBus transport feasibility. No Lay runtime API,
source, package, system installation, desktop bus, keyboard, or production
authority is changed.

## Baseline and variant decision

The accepted deployed baseline used local `/usr/bin/ibus-daemon` version
`1.5.34~rc2`, SHA-256
`24338c0e7cfb749d2ac9b9254d7d54029e71b5ac9342aa1b8d78b1d55f5ac1b6`.
The remote host's system daemon is independently identified as Debian
`1.5.26-4`, SHA-256
`cb8ccaffcfcd09f107a84edd982a8b16f55ba45af4458a4e9c45541bcd4217e5`.
It is a distinct daemon and must never be presented as deployed/source-matched
1.5.34 evidence.

| Variant | Score | Decision |
|---|---:|---|
| Existing remote 1.5.26 private baseline | 5/10 | Retain only as version-distinct feasibility context; it cannot prove the deployed 1.5.34 transport contract |
| Private exact 1.5.34 ELF closure, copied without installation | 8/10 | Selected: bounded, hash-addressed, no host ABI replacement, and the copied interpreter applies only to the owned daemon subprocess |
| Upgrade/install/build IBus or missing tooling on remote | 1/10 | Rejected: mutates infrastructure, needs unavailable build prerequisites, broadens ABI/package authority, and is unnecessary if the explicit closure relocates |

## Consequence boundary and invariants

The selected bundle contains the trusted executable, its requested ELF
interpreter, and every non-virtual `ldd` resolution. The copied interpreter is
called directly with `--library-path` for the daemon command only. It is not
exported through `LD_LIBRARY_PATH`, `LD_PRELOAD`, PATH, GI Python, GDBus, or
the host session, so the remote's Python GI 1.5.26 client remains a separately
identified client talking D-Bus to the private 1.5.34 daemon.

The existing 22-case private transport driver is reused in a fresh scoped root.
It keeps bwrap's read-only host bind, private PID/net/IPC/mount namespaces,
private D-Bus session/socket and XDG paths, empty DISPLAY/WAYLAND state, no
Lay binary or keyboard path, and writable paths restricted to the owned proof
root and bwrap tmpfs. Cleanup is limited to reaping the owned private daemon
and its transient systemd scope. No fallback to the remote system daemon,
loader, or library is admitted after the exact daemon command is selected.

Candidate/lattice, ranking, correction authority, package reload, learning,
daemon/IME ownership, and client-visible input are untouched. This proof can
only establish private IBus transport feasibility for the copied daemon. It
cannot prove TD-121 reducer/wiring, real Lay integration, product restoration,
quality, production resource limits, artifact compilation, or live release.

## Planned closure and proof denominators

Trusted daemon `readelf` requests `/lib64/ld-linux-x86-64.so.2`; `ldd` resolves
14 libraries: gio, gobject, glib, ibus, libc, gmodule, z, mount, selinux, ffi,
atomic, m, pcre2-8, and blkid. `linux-vdso.so.1` is virtual and is not copied.
Before transfer, the manifest must bind every source path, copied destination
name, byte size, mode, SHA-256, interpreter command, and the remote copies'
corresponding identities. A failed relocation, missing dependency, or loader
error is a concrete `BLOCKED_UNBOUNDED_DEPENDENCY` result: do not install,
upgrade, build, or add an unbounded package closure.

The later guarded run denominator is the inherited 22 synthetic IBus transport
cases, reported separately from the prior local 22-case baseline and from any
future Lay integration case. It must record private-daemon and remote-GI-client
versions/hashes, exact command, daemon cleanup, remote bundle path, results,
and what was not tested. No run begins without the parent admitting the remote
lease.

## Setup receipt: closure transferred, transport not yet run

Fresh local source root:
`/home/ubu/.cache/lay/td121-remote-deployed-ibus.aB8ULb`.
Fresh remote proof root:
`/home/e/.cache/lay/td121-remote-deployed-ibus.qFnUzG`.
The transferred tar was created with `--mtime=now`, SHA-256
`e49353d08ebe87529f2efe20c4c016c6904e1ef28e8ee79432cff120d174ddc9`.

Remote rehash confirmed the copied daemon retains
`24338c0e7cfb749d2ac9b9254d7d54029e71b5ac9342aa1b8d78b1d55f5ac1b6`,
the copied interpreter retains
`c5e80a563850d6ab5c2f2482e4202d9c1b71fbf44854b8c399e63527202c64e1`,
and the manifest, driver, D-Bus config and launcher have SHA-256 respectively
`a0f00c2f2d55d702fe336d6713124d145a22ded5d88e4569176c14380708636f`,
`ad637f78e64261e3de350a92d2e05dbdc9c51f98adf0d75fb6aedecbb42f460f`,
`fa33ebaea2f2a489d1d59f0f7e1d0dd046bf52a86bc9e9570086b81f30f01ee4`,
and `58cfa0e42fddad826e55d0423ac15fb1d3568267c60b0e62fb50ae46beb570e2`.
The launcher mode is `0755`.

The remote read-only loader check resolved every non-virtual dependency from
`bundle/lib`; no remote system library appears in that output. The reusable
daemon command is:

```text
/tmp/proof/bundle/lib/ld-linux-x86-64.so.2 --library-path \
/tmp/proof/bundle/lib /tmp/proof/bundle/bin/ibus-daemon --single \
--panel=disable --emoji-extension=disable --config=disable --cache=none \
--address=unix:path=/tmp/proof/ibus.sock
```

This loader check did not start `ibus-daemon`, D-Bus, GI Python, or the
22-case driver. The next action is the parent-admitted guarded private run;
until then the status remains preparation only.

## Mechanical pre-daemon stop and bounded repair

The parent-admitted first run stopped in `driver.py` before private-daemon
startup: `dbus-run-session` supplied a private abstract Unix address rather
than the inherited fixture's path-form spelling. It executed 0 cases, created
no daemon PID, and made no fallback attempt. The raw log is
`/tmp/td121-remote-deployed-ibus-qFnUzG-run1.log` on the remote worker.

The only repair broadens the harness assertion from `unix:path=/tmp/` to the
two private forms admitted by the existing bwrap namespaces: that path form or
`unix:abstract=`. It still rejects a host session address and records the
observed form in the receipt; launcher-level DISPLAY/WAYLAND, loader-variable,
private namespace, private XDG and private bus invariants remain unchanged.
The repaired driver SHA-256 is
`1c515b349d48c7630cb50dd92a15a63de4b49e02950e3e5bb92bcf2e1aa7f88a` and
its amended manifest SHA-256 is
`fc0a99f4daa54396e2f7c09e7953e105898c0ac898d9b4daf23fc6a50bda641f`.
One bounded rerun remains authorized by the parent lease; no further repair or
scope expansion is pre-authorized.

## Second pre-daemon stop: transient-unit collision

The repaired invocation did not enter bwrap or start a daemon. `systemd-run`
rejected the fixed transient unit name because the first failed pre-daemon
attempt remained in `failed` state. Read-only status then reported
`ActiveState=failed`, `Result=exit-code`, `ExecMainStatus=1`; no unit was
active and no daemon PID was created. The attempted `/tmp` raw logs were not
present when subsequently read, so they have no hash and must not be cited as
retained evidence.

The prepared, unrun mechanical fix uses a fresh `BASHPID`/`RANDOM` suffix for
the owned transient unit and `systemd-run --collect`, so a completed owned unit
does not collide with a later scoped run or remain registered. It does not
change the bundle, private daemon command, namespaces, D-Bus policy, client,
or proof cases. The amended launcher SHA-256 is
`e68aa017731fbc63e07f3848d2830bfca6d15257b4ed643ec54c62505d96e59b`;
the amended manifest SHA-256 must be rechecked after transfer. This is the
second bounded harness repair; no third actual run is authorized without a
fresh parent lease decision. A future admitted run must tee its raw output to
the durable remote proof root rather than `/tmp`.

## Third pre-daemon stop: host-specific HOME assertion

The unique-unit run entered the private D-Bus session but stopped before
`Popen`, daemon PID creation or case execution because the inherited local
fixture asserted `HOME=/home/ubu`. The remote user correctly retained its
unmodified `HOME=/home/e`; the launcher does not set or otherwise repurpose
HOME. The durable raw log is
`/home/e/.cache/lay/td121-remote-deployed-ibus.qFnUzG/run3.log`, SHA-256
`e43abc6c700d1dd898a072aaf5cd80f58c16b5d63331c46e13ddb552f1bf2768`.

The bounded repair removes only that host-specific assertion and records the
inherited HOME value in the receipt. It preserves private abstract/path bus
validation, no DISPLAY/WAYLAND, no loader-variable export, private XDG paths,
and no HOME assignment. Revised driver SHA-256:
`0831c096d733e57c403d8a29a80a64ff9a9ca5e4673a69d18b3b8f71121e2b9c`.
Revised manifest SHA-256:
`5002c7bab1dfe34023ebeb6320087e90646909ba5c19bd123a2b1397dab610b6`.
This remains a strictly mechanical pre-daemon correction; the parent-granted
non-privileged harness authority permits the next unique-unit attempt only.

## Final remote private transport receipt

The fresh unique-unit run completed with the copied deployed daemon and no
fallback: **22/22 synthetic transport cases passed**, 0 failed and 0 skipped.
The receipt reports `global_mode=true`, elapsed driver time
`0.15621795784682035` seconds, and owned private daemon PID 5 reaped after
SIGTERM (`returncode=-15`). The inner transient service completed successfully
in 213ms with 219ms CPU. Its unique unit was collected (`LoadState=not-found`,
`ActiveState=inactive`) after completion. The earlier fixed-name failed unit
remains a non-active systemd record; it owns no daemon and is not treated as a
successful or cleaned proof unit.

Durable remote artifacts in
`/home/e/.cache/lay/td121-remote-deployed-ibus.qFnUzG`:

| Artifact | SHA-256 |
| --- | --- |
| Final guarded raw log, `run4.log` | `351e85feebfc1bb6dfd7136f637afc29e0dd489313e4117e0a353b58c98c8779` |
| Final receipt, `receipt.json` | `c042724d9773d70842b896f11a7cf35f5a54042641ba3b9d8a1a694d3dad0afa` |
| Private daemon stderr | `d2523ed0ea830129c2d7c5cb7f1361007f11af331cc548ebc2820cab6f665695` |
| Copied 1.5.34 daemon | `24338c0e7cfb749d2ac9b9254d7d54029e71b5ac9342aa1b8d78b1d55f5ac1b6` |
| Copied ELF interpreter | `c5e80a563850d6ab5c2f2482e4202d9c1b71fbf44854b8c399e63527202c64e1` |
| Final driver | `0831c096d733e57c403d8a29a80a64ff9a9ca5e4673a69d18b3b8f71121e2b9c` |
| Final manifest | `5002c7bab1dfe34023ebeb6320087e90646909ba5c19bd123a2b1397dab610b6` |

The 22-method denominator is one bootstrap real-context/self-marker case, 20
foreign-engine ABA-before-marker cycles, and one other-sender marker case.
The daemon was launched exclusively through the copied interpreter and
`--library-path` command recorded above. The remote GI client and remote system
daemon remain Debian `1.5.26-4`; their system daemon hash is
`cb8ccaffcfcd09f107a84edd982a8b16f55ba45af4458a4e9c45541bcd4217e5`.
The driver's optional GI version symbol is absent (`UNKNOWN`), so the exact
installed `gir1.2-ibus-1.0`, `libibus-1.0-5`, and `ibus` package versions are
the identity evidence rather than an invented GI runtime string.

This is a private exact-daemon transport feasibility result only. It does not
promote TD-121 reducer/wiring, Lay integration, client-visible word retention,
quality, product performance, source compilation, or release authority.

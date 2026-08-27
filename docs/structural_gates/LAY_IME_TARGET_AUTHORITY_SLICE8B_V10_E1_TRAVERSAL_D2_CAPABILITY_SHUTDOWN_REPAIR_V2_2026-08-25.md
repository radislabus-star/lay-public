# V10 E1 Traversal D2 Capability Shutdown Repair V2

Date: 2026-08-25

## Scope

This paper revision repairs the capability-probe sequence after the immutable
V1 controller stopped at its own shutdown return-code check. It does not modify
V1, repeat T-CAP, build or execute D2, admit full B or V12, or change runtime
authority.

```text
immutable V1 execution verdict     BLOCKED_CAPABILITY
effective failure class            CONTROLLER_SHUTDOWN_PROTOCOL
T-CAP capability                   UNKNOWN, not FAIL
I-CORE capability                  NOT TESTED
I-ATOM capability                  NOT TESTED
```

The V1 receipt, consumed marker and retry prohibition remain historical facts.
V2 is a new route: first offline interpretation of existing sealed T-CAP bytes,
then, only after recovery PASS and a separate implementation preflight, one new
precise-only transaction for the two channels that V1 never started.

## Immutable V1 Boundary

Authoritative local evidence:

`docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_D2_IMPLEMENTATION_PREFLIGHT_2026-08-25/P1_REMOTE_EVIDENCE/`

Pinned identities:

```text
remote SHA256SUMS        06c86e7aa849d467c8985ef3b0910090e3d1d8666b576d50d4bc82dc0ab03a55
remote PROBE_RECEIPT     1c41c796458b862813601c8853788675b25b0d221b3447e16237f5f87ed6a8dc
effective local receipt cf8b06fb55f220d6051fcc8c1f193389481698cf0f64f31dc5109bae2e850dc5
T-CAP perf.data         43e67a0b370c14adab60cd229d925f168f7fbaa94930ed2805876d1b01039a99
T-CAP record command    d2aa8bec8fcdcb0bf0fbc702c08592d0e14bd0285a93569b03d36bf9f5f423b3
V1 controller           4b025b93e80cbba66b2ea44a9b0845de883053b5586f9fc4e892a20fdb4f2626
```

V1 ran one task-clock record invocation, ran no perf-data readers and opened no
precise-instruction route. Its remote receipt derived execution counts from
completed subruns; the effective local receipt overlays only that count defect.

## R1 Offline T-CAP Salvage

R1 consumes the existing sealed `perf.data`, `maps-before.txt`,
`maps-during.txt`, record command and benign ELF identities. It may execute
exactly these readers with the pinned kernel-specific perf binary:

```text
perf evlist -v -i SEALED_PERF_DATA
perf script -i SEALED_PERF_DATA -F cpu,event,ip,dso
perf script -D -i SEALED_PERF_DATA
perf buildid-list -i SEALED_PERF_DATA
```

R1 forbids:

```text
perf record / perf stat
PMU or software-event opening
yes or any subject execution
D2 / V10 / V13 execution
copying over or chmod of V1 evidence
event, period or selector substitution
```

R1 publishes `T_CAP_RECOVERED_FROM_SEALED_EVIDENCE` only when:

```text
sealed input identities                         exact
sample rows                                      > 0
sample CPUs                                      {0}
sample event                                     task-clock:u
evlist type / config                             1 / 1
sample_period                                    100000
freq / exclude_kernel / precise_ip               0 / 1 / 0
lost records or samples                          0
throttle / unthrottle records                    0 / 0
yes Build ID             8c99ebc2c856857219acc612c2d9be3172b74be5
maps-before relevant mappings == maps-during     true
every sampled yes IP maps uniquely through executable PT_LOAD
normalized yes IPs inside executable PT_LOAD     all
```

Any reader, identity, event, sample, Build-ID, map or normalization failure is
terminal `BLOCKED_TCAP_EVIDENCE`. R1 never upgrades the immutable V1 verdict.

## R2 Controlled Shutdown Protocol

The future precise-only controller must record these facts before shutdown:

```text
perf poll() is None
fixed two-second interval completed
benign yes PID alive
PID affinity unchanged and exact
shutdown_requested = true
shutdown_signal = SIGINT
shutdown_signal_count = 1
```

Only a signal sent by the controller after all preconditions permits:

```text
accepted returncode = 0 or -SIGINT
shutdown verdict    = CONTROLLED_SHUTDOWN_VALID
```

An accepted shutdown result is lifecycle evidence only. Capability validation
still requires a nonempty finalized `perf.data`, all four readers and every
event/sample/identity check. A process that exits before controller shutdown is
`BLOCKED_RECORD`, regardless of its return code.

## R3 Precise-Only Capability Transaction

R3 has a new task identity and separate one-shot markers. It exists only after
R1 PASS and a named implementation preflight.

```text
task ID       slice8b-v10-e1-traversal-d2-precise-capability-v2-20260825

I-CORE-CAP
  CPU         0
  event       cpu_core/event=0xc0/upp
  period      5000000 retired instructions
  precise_ip  2

I-ATOM-CAP
  CPU         12
  event       cpu_atom/event=0xc0/upp
  period      5000000 retired instructions
  precise_ip  2
```

`I-CORE-CAP` consumes its marker before record. `I-ATOM-CAP` is admitted only
after the complete sealed I-CORE result passes. Any I-CORE failure leaves the
ATOM marker available but unadmitted. Any started route is one-shot and cannot
be repeated under V2.

Each route uses a new taskset-pinned `/usr/bin/yes` PID and the R2 shutdown
protocol, then runs the mandatory readers. It requires exact event type/config,
period, `freq=0`, `exclude_kernel=1`, `precise_ip=2`, pinned sample CPU, exact
event label, samples greater than zero, zero lost/throttle/unthrottle records,
the exact yes Build ID and unambiguous ET_DYN IP normalization.

No foreign process is stopped, reprioritized or re-affined. Ordinary
Nando/btop/K1 load is allowed and is not an environment veto.

## Failure Taxonomy

```text
BLOCKED_CONTROLLER_PROTOCOL
  lifecycle, signal, accepted-return or controller-state defect

BLOCKED_RECORD
  perf exits before requested shutdown or cannot finalize nonempty data

BLOCKED_READER
  sealed/finalized data exists but a required reader cannot parse it

BLOCKED_CAPABILITY
  event unsupported, no valid samples, wrong PMU/precision/period/CPU,
  lost/throttled data or failed Build-ID/IP evidence

BLOCKED_PROVENANCE
  input, binary, host, marker or publication identity drift
```

The first failure is retained without relabeling it as a hardware capability
failure. Reader and controller failures never produce capability PASS.

## R4 Final Decision

```text
T_CAP_RECOVERED_FROM_SEALED_EVIDENCE
AND I_CORE_CAP_PASS
AND I_ATOM_CAP_PASS
  -> D2_CAPABILITY_READY
  -> final D2 implementation preflight may be created

otherwise
  -> named terminal blocker
```

`D2_CAPABILITY_READY` is not D2 implementation authority. The later final
preflight must still bind the exact controller, build route, D1 Rust bytes,
PIE/ASLR normalization, bucket-map generator, U-route perturbation sequence and
all T/I markers before it can return `READY_TO_IMPLEMENT`.

## Explicit Non-Authority

This repair does not admit a D2 build, D2 subject, bucket map, U/T/I D2 route,
full B, SWAR, layout changes, V12, runtime integration or deployment. Runtime
authority remains unchanged.

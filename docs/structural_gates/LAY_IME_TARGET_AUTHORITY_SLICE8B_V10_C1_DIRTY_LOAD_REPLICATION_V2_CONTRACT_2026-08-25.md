# Slice 8B V10 C1 Dirty-Load Replication V2

Date: 2026-08-25

## Decision

The user explicitly requested one repeat of the completed dirty-load direct
latency observation. V1 is immutable and its one-shot route rights remain
consumed. This contract admits one new independent replication named V2.

V2 must preserve exactly:

```text
sealed C1 ELF SHA-256      ead184029a2923cfd24c5d02e91e91f9d69fb01c7daa0fb21ba69f267234c93c
ELF Build ID               665458be5064a689951c6f074276ab0bb4d2beb4
completed parity           382/382, zero mismatches, zero false certs
route order                S1 T1 T2 S2 S3 T3 T4 S4 S5 T5
S rounds/process           100
T rounds/process           250
S pooled samples           191,000
T pooled samples           477,500
CPU mapping                S CPU 0; T CPUs 0..19
timing authority           result.search_elapsed_us / result.total_elapsed_us
percentile                 nearest rank p99
thresholds                 3,000 / 5,000 us and 5,000 us fairness
```

V2 uses only new paths:

```text
remote task/state  slice8b-v10-c1-dirty-direct-latency-v2-20260825
local result       LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_C1_DIRTY_DIRECT_LATENCY_V2_2026-08-25
```

Before and after V2, the controller must verify:

```text
V1 remote final/state              present and unchanged
V1 local result                    present and unchanged
clean C1 final                     absent
all clean S/T markers              available
installed Lay/runtime authority    unchanged
```

The host remains intentionally loaded. Nando, btop and other processes must not
be stopped, tuned, re-affinitized or reprioritized. PSI, temperature, throttle
counters and process CPU deltas are observations, not quiet-host blockers.

The only successful verdict remains:

```text
DIRTY_LOAD_OBSERVATION
thresholds_would_pass = true | false
```

V2 cannot become clean `C1_PASS` or `C1_FAIL`, cannot admit formal B, runtime
integration or V12, and cannot revise the V1 verdict. After V2 publication the
owning document may compare V1 and V2 using only the preregistered metrics.

Exactly one V2 matrix is admitted. Failure consumes the relevant V2 marker;
adaptive retry, a third run, changed repetitions, changed route order and
post-result threshold changes are forbidden without a new user decision and
new preflight.

No Cargo/rustc/build, parity rerun, perf/PMU, installed Lay mutation, daemon
restart, foreign process control or clean C1 marker consumption is admitted.

# Slice 8B V10 Loaded PMU Diagnosis V1

Date: 2026-08-25

## Decision

Dirty direct latency is closed after two independent loaded runs. The user
explicitly rejected treating one busy Nando core as a reason to stop and asked
to continue diagnosis of the V10 code under the normal loaded mini-PC state.

This route is a short executor-core hardware diagnosis. It is not a third C1
latency replication, clean C1, formal B, historical B3, or V12 admission.

## Frozen Subject

```text
host                       e-MEGA-MINI-M1-13th
diagnostic ELF SHA-256     f7bcee37d5dffd583577d66c982c2a28072889b8aac2ccbed22612aa6d4feb09
ELF Build ID               9829fb05f34bd353877fb6d71f1f8523e084af55
package SHA-256            cce259fe0ce5dce67702383363b66f0fe9b9ff5a87d8f01c4fcf342d91218d7b
sidecar SHA-256            a1aa95be8c43fae8fedf02d3261dd1265dcbfbc8d618330fcd6449ae829df8cd
schedule SHA-256           2f8346d97246ac3704434279739fa5ed27705465d628890a367c21875d879e78
requests per route         382
B5 CPUs                    0
B6 CPUs                    0..19
```

No Cargo, rustc, source patch, rebuild, package rewrite, installed Lay change,
daemon restart, foreign process stop, host tuning, reprioritization, or
re-affinity is allowed.

## Sequence

```text
implementation preflight READY_TO_IMPLEMENT
  -> controller self-check, UNRUN remotely
  -> exact same-ELF semantic parity once, without perf
  -> benign sudo perf capability probe
  -> B5 then B6 for G0
  -> B5 then B6 for G1
  -> B5 then B6 for G2
  -> B5 then B6 for G3
  -> immutable publication
  -> STOP
```

The parity prerequisite must report 382 records, zero terminal, peak,
completeness, work and full-row mismatches, 382/382 target-form and target-lemma
retention, zero false certificates, maximum 35,590 product states and 6,656
scratch bytes.

## Counter Groups

```text
G0  task-clock, cycles, ref-cycles, instructions,
    context-switches, cpu-migrations, page-faults

G1  branches, branch-misses, cache-references, cache-misses

G2  L1-dcache-loads, L1-dcache-load-misses, LLC-loads, LLC-load-misses

G3  dTLB-loads, dTLB-load-misses
```

`perf stat` starts disabled. The subject loads and warms, then parks on its
existing `subject-ready` marker. The controller enables counters, receives the
perf acknowledgement, releases the existing executor barrier, waits for
`subject-done`, disables counters, receives acknowledgement, and only then
allows receipt serialization. Counter scaling or a missing/not-counted event
invalidates that group. No adaptive event substitution or rerun is allowed.

`perf_event_paranoid=4` requires the already available passwordless `sudo` for
perf only. The measured test child runs as user `e`; the controller records the
exact command and raw perf JSON. Sudo is not used to change host policy.

## Observation Boundary

The environment remains intentionally loaded. Nando, btop, K1 and every other
foreign process are observations, not blockers and not presumed causes. Each
window records CPU/IO PSI, temperature, throttle counters, subject affinity and
background CPU deltas.

This V1 route has one attempt for parity, one benign probe and one attempt for
each of the eight route/group pairs. A consumed marker cannot be retried.
Failure retains evidence and terminates the route.

## Claim Boundary

The result may report executor-proxy counters per request and compare B5 with
B6. It may distinguish instruction pressure, branch behavior, generic/data
cache behavior and dTLB behavior within this loaded diagnostic denominator.

It cannot prove:

```text
clean C1 PASS or FAIL
historical V10 PMU attribution
formal B
causality assigned to Nando, scheduler, cache, or arithmetic alone
per-edge cost without a matching structural edge denominator
end-to-end installed Lay latency
V12 admission
```

The only successful terminal verdict is
`LOADED_PMU_DIAGNOSTIC_OBSERVED`. Runtime authority remains unchanged.


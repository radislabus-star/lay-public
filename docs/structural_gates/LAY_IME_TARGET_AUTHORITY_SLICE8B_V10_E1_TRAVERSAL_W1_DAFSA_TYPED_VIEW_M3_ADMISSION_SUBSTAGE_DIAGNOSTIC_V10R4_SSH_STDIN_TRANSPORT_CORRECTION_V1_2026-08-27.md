# DAFSA Typed View M3 Admission-Substage V10R4 SSH Stdin Transport Correction V1

Date: 2026-08-27

## Scope

V10R4 repairs only the read-only SSH observer transport that terminated V10R3
before execution admission. It preserves the V10R3 TRACE-only scientific,
quiet, marker, sealed-ELF, parser, failure-dispatch, and claim-boundary
contracts without change.

## Immutable V10R3 History

V10R3 remains terminal `BLOCKED_PROVENANCE`:

```text
task_id
  slice8b-v10-e1-traversal-w1-dafsa-typed-view-m3-admission-substage-v10r3-20260827

transaction_id
  8fa454ad944ce8ba9e6256bc55bdf8ff8231cf26bac7eaab591ad8564515436c

implementation receipt
  7719b69b7a218b4d525aa0cf583c610d035ea6a458561c53a60f36a7a9188286

execution journal SHA256SUMS
  5baf0039407a6a3e752326440e47b13a4005327f365665eecaa78331d1d693c6

failure diagnosis
  cb3c616bd83bb08f7db16c292e3795c248780676e39de5fdbddaf7a723f5d97e
```

The first durable action intent was `live-admission`. Its observer failed
before publishing a receipt because OpenSSH joined the remote argv into a
shell command and did not preserve a multiline `python3 -c` program as one
argument. The remote shell interpreted the following Python lines as shell
commands.

Independent read-only inspection using the previously sealed V10R2 stdin
observer proved after failure:

```text
V10R3 parent     absent
V10R3 state      absent
V10R3 cache      absent
V10R3 markers    0
V10R3 TRACE      absent
Cargo / rustc    0 / 0
subject          0
perf / PMU       0 / 0
```

V10R3 is never retried and its controller, auditor, journal, and diagnosis
remain immutable.

## Corrected Transport

Every multiline remote observer in V10R4 uses the already proven protocol:

```text
local subprocess stdin = exact Python source bytes

ssh ...
  [sudo -n]
  /usr/bin/python3 - <exact positional arguments>
```

The program is not passed through `python3 -c`, shell interpolation, a
temporary remote script, base64 decoding, or an unpinned wrapper. Positional
arguments remain separate ordinary argv values. The local controller retains
bounded stdout and stderr for every nonzero response.

## Transport Admission Before Namespace

V10R4 adds one first, read-only external action:

```text
TRANSPORT-ADMISSION
  -> send a multiline observer through python3 stdin
  -> verify exact frozen nonce
  -> verify exact positional argv round-trip
  -> verify root observation UID
  -> verify exact host identity
  -> no filesystem write
  -> no namespace, cache, marker, ELF, subject, Cargo, rustc, perf or PMU
```

Failure is terminal `BLOCKED_PROVENANCE` before any remote mutation. PASS
admits the unchanged full live provenance observer using the same helper.

## Fresh Namespace

```text
task_id
  slice8b-v10-e1-traversal-w1-dafsa-typed-view-m3-admission-substage-v10r4-20260827

transaction_id
  eeac980119d265ff545142b311256ecb302f70197b3bb634df541302bdc94097

route
  TRACE-REUSE

marker
  trace.available
```

V10R4 has a fresh local journal, remote parent, state tree, cache, route lock,
controller identities, and marker payload. It does not reuse any V10R3
authority or marker.

## Preserved Execution Contract

After transport admission, V10R4 executes the V10R3 state machine unchanged:

```text
full live provenance admission
  -> markerless reuse bootstrap
  -> independent bootstrap audit
  -> up to 120 five-second quiet windows
  -> require CPU0 >= 0.95 and all CPUs >= 0.90
  -> require three consecutive PASS windows
  -> create one fresh trace.available only after quiet PASS
  -> atomic consume-before-exec
  -> one direct sealed V10R2 ELF subject
  -> independent terminal audit
```

The reused ELF remains:

```text
SHA-256   0378514225ccec3cadbcfedd21ec77db66518a5eb6789f9acd83525ccf009696
size      320,986,144 B
Build ID  9e2e7c1fef9272f87c14876d7194609df6ac948d
```

There is no BUILD route, Cargo, rustc, source edit, ELF copy or rewrite,
perf/PMU route, production mutation, deployment, or retry.

## State Machine

```text
V10R3 BLOCKED_PROVENANCE (immutable)
        |
        v
V10R4 controllers verified unrun
        |
        v
TRANSPORT-ADMISSION
        | FAIL -> BLOCKED_PROVENANCE, zero remote writes
        v
full live admission
        |
        v
markerless bootstrap + audit
        |
        v
bounded quiet closure
        | FAIL -> BLOCKED_QUIET_BEFORE_MARKER
        v
fresh marker -> consume -> ONE direct sealed-ELF TRACE
        |
        v
independent terminal audit
        |
        +-- PASS -> ADMISSION_SUBSTAGES_DECOMPOSED
        +-- FAIL -> exact frozen BLOCKED_* verdict
```

## Admission Verdict

This paper admits only V10R4 structural review and implementation preflight.
It grants no execution, optimization, production, or deployment authority by
itself.

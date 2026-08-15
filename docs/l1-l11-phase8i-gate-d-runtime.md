# L1.1 Phase 8I Gate D: Physical Package And Live Runtime

Status: integrity-closure Gate C and rebuilt remote Gate D PASS; corrective Lay 1.0.29 release deployed to daemon, L1.1, and GNOME runtime; managed IBus process preserved; physical input gate pending
Date: 2026-08-15
Input authority: accepted Phase 8I Gate C v2 receipt

## 1. Decision

Gate C proved the compiler-exact typed basin, but it did not create a physical
package or change runtime authority. Gate D promotes that same computation
without introducing a second candidate route:

```text
V9 package
  -> exact support field
  -> forward decoder index
  -> Phase 7D typed basin
  -> implicit exact-u32 forward state
  -> exact-u32 reverse state
  -> frozen nonlinear settlement
  -> one bounded L1.1 lattice
  -> existing L2 -> L3 -> verifier route
```

The current V8 path remains supported only for old V8 artifacts. A V9 request
must never fall back to V8 postings, legacy birth, or a union of both routes.

## 2. Physical V9 Format

V9 wraps the byte-identical compact V7 base already embedded in the accepted
V8 artifact and adds only exact support values that cannot fit in `u16`.

```text
offset  bytes  field
0       8      magic = "LAYL1V9\0"
8       4      version = 9
12      4      header bytes = 32
16      8      compact V7 base bytes
24      8      checksum with this field zeroed
32      N      compact V7 base
32+N    8*K    sorted overflow entries: u32 AtomId + u32 exact support
```

The overflow count is derived from the checked file length. The following are
load-time errors:

- invalid magic, version, header size, length, or checksum;
- a compact base that fails its own checksum or depth-0 validation;
- unsorted or duplicate overflow AtomIds;
- an overflow AtomId outside the atom field;
- an overflow entry whose stored support is not `u16::MAX`;
- an exact support value not strictly greater than `u16::MAX`;
- a saturated atom omitted when the package builder measured a larger value.

For the accepted corpus the preregistered size is:

```text
compact V7 base                         77,960,560 B
V9 header                                      32 B
217 overflow entries                         1,736 B
expected V9 package                    77,962,328 B
```

The existing installed V8 artifact is an immutable build input. V9 is written
to a temporary sibling and renamed only after a successful self-load.

## 3. Runtime Ownership

`LexicalGrokkingMemory::load()` selects one owner from package magic:

```text
V2-V7 -> existing eager owner
V8    -> existing lazy-posting owner
V9    -> Phase 8I exact typed-basin owner
```

For V9, startup validates the package, reconstructs the exact support vector
from stored `u16` values plus overflow entries, and builds one forward decoder
index. A query then executes exactly once:

```text
surface
  -> typed terminal evidence D_E(Q)
  -> reconstruct every admitted terminal from its decoded surface
  -> settle the complete basin with matching exact reverse relations
  -> preserve the full-field Winner | Tied | ABSTAIN readout
  -> apply the unchanged bounded projector for L2 seed transport
```

The full-field authority decision and bounded candidate transport are separate.
Truncating transport must not manufacture a singleton authority. A query error
returns no V9 lattice and increments an observable failure counter; it cannot
invoke the legacy route.

## 4. Service Flip And Rollback

The L1.1 service already loads a replacement snapshot before taking its write
lock. Gate D keeps this transaction:

```text
current V8 snapshot serves requests
  -> load and validate V9 beside it
  -> warm and run health query
  -> atomically replace host snapshot
  -> retain V8 package and previous binaries as rollback pair
```

If loading, warmup, parity, latency, or health fails, the V8 snapshot remains
live. Package discovery is changed only after the V9 receipt and artifact are
durably installed. Global IBus is not restarted. The daemon may be restarted
only after the service/package pair has passed its independent health gate.

## 5. Gate D

All gates are conjunctive:

```text
actual V9 round-trip                         <=195 MiB
exact support overflow parity                    exact
V9/runtime candidate and readout parity          exact
ordinary prefix completion                        PASS
single-process cold load                         <=1 s
L1.1 service steady RSS                      <=250 MiB
first-touch and hot end-to-end p99             <=5 ms
false authority / false singleton                 0 / 0
daemon / IBus / WeChat physical gate               PASS
rollback package and binary pair                   PASS
```

The accepted Gate C quality remains the quality authority only when package
round-trip and runtime parity are exact. A sampled runtime proof cannot replace
Gate C, and proof throughput cannot be reported as product latency.

## 6. Required Evidence

Before live promotion, Gate D records separately:

- physical package bytes, SHA-256, base bytes, overflow count, and checksum;
- load time, decoder-index time, first-touch time, steady RSS, and peak RSS;
- end-to-end p50, p90, p99, and maximum latency on fixed diverse surfaces;
- candidate/readout fingerprints against the accepted Gate C implementation;
- service PID, package path, request count, and health before and after flip;
- daemon PID/RSS, active version, and absence of a second L1.1 service;
- exact rollback paths and hashes;
- what was not physically tested by the agent.

## 7. Forbidden Repairs

- no literal word, suffix, class, or fixture condition;
- no wider frontier or changed Gate C threshold;
- no V9-to-V8 per-request fallback;
- no union of typed-basin and legacy candidates;
- no corpus crystallization;
- no mutation or deletion of the accepted V8 package;
- no deployment before package, parity, latency, RSS, and service gates pass.

## 8. Standalone Gate D Measurement And Latency Correction

The first physical standalone run used the release service and the staged V9
bytes on the remote build host. Runtime authority did not change: installed
Lay `1.0.27` and V8 remained live throughout this measurement.

Measured facts:

```text
V9 bytes                                      77,962,328 B
V9 SHA-256            bf5a1619a89038466ef786305cf35eda5f4af5b9f12b9140f7d3cac407e2f2a7
cold ready, continuous polling                        690 ms
steady RSS                                         164,968 KiB
peak RSS                                           185,924 KiB
query response failures                                  0
direct runtime p99, four fixed runs     4.854 / 4.894 / 4.877 / 4.834 ms
new Unix connection per query p99       5.088 / 5.368 / 5.360 / 5.180 ms
persistent Unix connection p99          5.107 / 4.946 / 4.963 / 4.980 ms
```

The percentile index for both direct and socket measurements is the repository
gate formula `sorted[(N - 1) * p / 100]`. The socket measurements cover 520
fixed surfaces, one warmup pass, four measured passes, `Lattice(limit=32)`,
JSON encoding/decoding, and the Unix transport.

Evidence currently resides on the build host under:

```text
/home/e/projects/lay-phase8i-gate-d/artifacts/gate-d/v9-diverse-optimized-{1,2,3,4}.json
/home/e/projects/lay-phase8i-gate-d/artifacts/gate-d/standalone-health.json
/home/e/projects/lay-phase8i-gate-d/artifacts/gate-d/standalone-socket-latency-comparison.json
/home/e/projects/lay-phase8i-gate-d/artifacts/gate-d/standalone-v9.log
```

Verdict scope: package integrity, cold load, RSS, direct runtime latency, and
service response integrity pass. Actual hot service transport remains rejected
because all four current per-query connection runs exceed `5 ms`. This is not a
quality failure and is not permission to change a settlement coefficient,
candidate bound, corpus, or percentile definition.

The admitted correction has two execution-only parts:

```text
one process-wide L2 lattice client
  -> retain one Unix connection across read-only Lattice requests
  -> serialize concurrent callers through one bounded mutex
  -> after a transport failure, discard the connection and retry once
  -> never use this retry route for Reload or another mutating request

existing V9 surface-index construction pass
  -> retain decoded UTF-8 bytes in one offsets + bytes pool
  -> materialize the same selected terminal surfaces without parent replay
  -> preserve terminal IDs, candidate order, scores, authority, and JSON schema
```

A process-wide client is required instead of one thread-local connection per
caller: the current service assigns a worker to a connection for its lifetime,
so unbounded thread-local connections could pin the complete worker pool. The
single client preserves one live request stream and reconnects fail-closed.

The decoded pool may consume only the UTF-8 surface bytes plus one bounded
offset record per terminal. It is constructed from the same decoder nodes
already traversed while V9 builds exact surface indexes. V8 keeps its existing
decoder behavior and memory profile.

Promotion remains blocked until all of the following pass on rebuilt release
bytes:

```text
candidate and readout fingerprint                         exact
two sequential Lattice requests use one accepted connection
failed read-only transport reconnects at most once
mutating requests never enter the persistent retry route
four fixed end-to-end service p99 runs                 <=5 ms
cold ready                                              <=1 s
steady RSS                                          <=250 MiB
query/response failures                                     0
```

Not yet tested at this point: rebuilt transport latency, rebuilt cold load and
RSS, local standalone parity, installed daemon behavior, physical IBus input,
double-Shift rollback, and WeChat. No runtime authority changes before those
gates pass.

## 9. Production-Client Rejection And Shared Reconstruction

The rebuilt release service passed protocol reuse, reconnect, decoded-surface
parity, package integrity, cold load, RSS, and response integrity. The exact
Rust production client then rejected promotion:

```text
fixed surfaces                                      520
Lattice limit                                        32
response failures                                     0
first four p99 runs          5.218 / 5.270 / 4.909 / 4.981 ms
maximum accepted budget                           5.000 ms
```

Three additional four-run samples reproduced maximum per-sample p99 values of
`5.160`, `5.142`, and `5.214 ms`. The overrun is therefore repeatable and is not
waived as scheduler noise. Installed Lay `1.0.27` and V8 remain live.

The slowest stable requests all produce a large complete typed basin. Source
inspection found one shared mechanism after terminal birth:

```text
terminal ID
  -> decode surface and resolve atom occurrences
  -> derive implicit forward relations

the same terminal ID
  -> decode the same surface again
  -> resolve the same atom occurrences again
  -> derive exact reverse relations
```

The second decode and atomization add execution work but no independent
evidence. The admitted correction is one immutable candidate-local
reconstruction:

```text
terminal ID
  -> decode surface once
  -> resolve atom occurrences once
  -> derive implicit forward relations
  -> derive matching exact-u32 reverse relations from the same occurrences
  -> exact settlement
```

This does not cache a second global relation bank and does not alter V9 bytes.
The forward and reverse vectors remain candidate-local and bounded by the same
complete `D_E(Q)` basin. Runtime uses the co-derived exact reverse slice once;
the proof path independently compares every co-derived relation against the
existing compiler-reference reconstruction. A mismatch is a hard rejection,
not permission to fall back to V8 or reconstruct a second runtime route.

The correction may proceed only with these identities pinned:

```text
package bytes and SHA-256                              exact
candidate/readout fingerprint                         exact
co-derived reverse vs compiler reference              exact
terminal IDs, relation order, strengths and flags     exact
query failures                                            0
four Rust production-client p99 runs               <=5 ms
cold ready                                           <=1 s
steady RSS                                       <=250 MiB
```

Measured facts, hypotheses, and verdict scope remain separate. The repeated
decode is measured from the source route; the expected latency reduction is a
hypothesis until rebuilt release evidence passes. Runtime authority remains V8
until the complete Gate D and rollback transaction pass.

## 10. Shared Reconstruction Acceptance And Live Activation

The shared-reconstruction implementation preserved the physical package and
the complete candidate/readout fingerprint. Remote release evidence closed the
reference latency gate:

```text
focused parity tests                                      23 / 23 PASS
candidate SHA-256             b2f7ba7a9cf5e0f7ccd1feb76d18f7f5aa60db5dfad65ca4c12d6dba0b9b3aa0
production-client p99, four runs          4.438 / 4.314 / 4.042 / 4.146 ms
repeat maximum p99 samples                         4.326 / 4.130 ms
direct runtime p99, four runs              3.883 / 3.883 / 3.878 / 3.863 ms
reference cold ready                                         680 ms
reference steady RSS                                     178,892 KiB
reference peak RSS                                       202,360 KiB
response / query failures                                      0 / 0
```

The accepted release artifact remained:

```text
package bytes                                           77,962,328 B
package SHA-256        bf5a1619a89038466ef786305cf35eda5f4af5b9f12b9140f7d3cac407e2f2a7
terminal count                                             852,582
format                                                          V9
owner                              phase8i_exact_typed_basin
```

The four direct-runtime receipts are:

```text
/home/e/projects/lay-phase8i-gate-d/artifacts/gate-d/v9-diverse-shared-reconstruction-1.json
/home/e/projects/lay-phase8i-gate-d/artifacts/gate-d/v9-diverse-shared-reconstruction-2.json
/home/e/projects/lay-phase8i-gate-d/artifacts/gate-d/v9-diverse-shared-reconstruction-3.json
/home/e/projects/lay-phase8i-gate-d/artifacts/gate-d/v9-diverse-shared-reconstruction-4.json
```

The exact production-client and standalone receipts are:

```text
/home/e/projects/lay-phase8i-gate-d/artifacts/gate-d/standalone-rust-client-latency-shared-reconstruction.txt
/home/e/projects/lay-phase8i-gate-d/artifacts/gate-d/standalone-health-shared-reconstruction.json
/home/e/projects/lay-phase8i-gate-d/artifacts/gate-d/standalone-ready-shared-reconstruction-ms.txt
/home/e/projects/lay-phase8i-gate-d/artifacts/gate-d/standalone-v9-shared-reconstruction.log
```

Local pre-activation smoke used the staged release service and a unique socket.
It proved package compatibility, `852,582` terminals, zero query failures,
`178,920-179,108 KiB` steady RSS and `203,244-203,528 KiB` peak RSS. Passive
startup on the local Intel i7-8650U measured `1,047 / 1,037 / 1,045 / 1,040 ms`.
That local hardware observation is `37-47 ms` above the `<=1 s` reference
target and remains an explicit local cold-start WATCH. It is not hot-query
latency and is not represented as a PASS.

Live activation on 2026-08-15 installed the V9 package and all ten Lay `1.0.28`
release binaries with exact staged SHA-256 parity, then restarted only
`lay-daemon.service`:

```text
Lay version                                                1.0.28
daemon PID                                      3719354 -> 3440152
L1.1 service PID                                3719374 -> 3440215
managed lay-ibus-engine PID                     3719399 -> 3719399
global ibus-daemon PID                                3702 -> 3702
active IBus engine                                        lay-ime-ru
daemon count / L1.1 service count                              1 / 1
live package bytes                                      77,962,328 B
live terminal count                                        852,582
live service query failures                                      0
```

The live package and rollback roots are:

```text
/home/ubu/.local/share/lay/nanda_wave/l1.1/LAY-L1.1-RU-COMPOSITE-EN300K-PHASE8I-v9.v9.bin
/home/ubu/.local/share/lay/nanda_wave/l1.1/LAY-L1.1-RU-COMPOSITE-EN300K-PHASE8I-v9.installed.json
/home/ubu/.local/lib/lay/rollback/1.0.27-pre-1.0.28-phase8i-20260815-025951
```

Live smoke returned the exact clean winner for `морфология`, a safe tie for
`ytn`/`нет`, and fail-closed abstentions for ambiguous `работают` and `мло`.
The daemon journal contained no startup, package, query, or panic error. The
temporary remote V9 service was stopped after activation. Runtime authority
changed from the V8 service owner to the single V9 Phase 8I service owner; no
V8/V9 per-request fallback or union route was introduced.

Not physically tested by the agent after activation: real application typing,
WeChat behavior, and the autocorrection double-Shift rollback gesture. Those
remain the user physical-input gate and are not inferred from process health.

## 11. Post-Activation Integrity Closure

The activation in Section 10 is retained as historical evidence. The current
corrective branch has not replaced the installed Lay `1.0.28` binaries, V9
package, active receipt, daemon, or IBus process. The verified installed package
is still `77,962,328 B` with SHA-256
`bf5a1619a89038466ef786305cf35eda5f4af5b9f12b9140f7d3cac407e2f2a7`.

The post-activation review found four integrity boundaries that are now closed
in source:

1. V9 can be admitted only by `PASS_C_QUALITY`; the legacy `PASS_shadow`
   compatibility verdict remains restricted to V7/V8.
2. Installer and runtime admission recompute the fixed schema-v3 denominator,
   exact 13-class set, every per-class top-1 and bounded-coverage threshold,
   clean preservation, false-authority/singleton counters, grounded loss, and
   package identity. The installer rechecks copied bytes before publishing the
   receipt and does not trust only the aggregate PASS boolean.
3. Startup checks the active integrity-bound receipt before socket ownership and
   again after warmup. Reload checks before loading and immediately before the
   atomic snapshot flip; a changed receipt preserves the current snapshot.
4. Live V9 transport accepts only limits `1..=32`, while L2 requests exactly
   `32`. V9 query errors remain typed protocol errors and cannot reach V8.

The Double Shift route was also brought under the common mutation monopoly.
The active-composition path now carries one typed IME `AuthorizedEdit` through
the commit owner without pre-commit state mutation. The recorded-autocorrection
path restores its pending undo snapshot on both an unhandled replacement and a
backend error.

Current local evidence:

```text
installer integrity regression suite                       PASS
runtime service/admission tests                          13 / 13
mutation monopoly tests                                 16 / 16
observed V9 lattice route                               20 / 20 PASS
observed V9 lifecycle route                             25 / 25 PASS
observed Double Shift route                             21 / 21 PASS
scripts/check-lay-changed.sh                                      PASS
runtime deployment by this branch                                  no
```

Observed-source receipts:

```text
/home/ubu/projects/lay-l1-exact-peak-search/docs/structural_gates/receipts/L1_L11_PHASE8I_INTEGRITY_CLOSURE_2026-08-15/observed-v9-lattice-route-receipt-v1.json
/home/ubu/projects/lay-l1-exact-peak-search/docs/structural_gates/receipts/L1_L11_PHASE8I_INTEGRITY_CLOSURE_2026-08-15/observed-v9-lifecycle-route-receipt-v1.json
/home/ubu/projects/lay-l1-exact-peak-search/docs/structural_gates/receipts/L1_L11_PHASE8I_INTEGRITY_CLOSURE_2026-08-15/observed-double-shift-route-receipt-v1.json
```

This local result is not a replacement Gate C or Gate D PASS. The participating
source packet must still pass the direct V9 schema-v3 `13 x 100` smoke and then
the fixed `13 x 20,000 + all clean` proof on the remote 20-core host. Only after
that proof may rebuilt release latency, RSS, cold startup, rollback, managed
service restart, and physical application input be measured. Global IBus must
not be restarted.

## 12. Integrity-Closure Gate C And Gate D Acceptance

The corrective packet subsequently passed the required direct V9 proof route
without rebuilding or modifying the accepted V9 artifact:

```text
participating-source parity                              789 / 789
smoke                                                      PASS_C_SMOKE
full fixed proof                                         PASS_C_QUALITY
damaged denominator                                            260,000
clean denominator                                              852,582
complete-field target retention                                 100.0%
aggregate unique top-1                                          98.941%
aggregate bounded lattice coverage                              99.868%
primary false authority / false singleton                         0 / 0
clean preservation                                              100.000%
stored exact support vs corpus rebuild                            exact
V8 fallback used                                                  false
```

All thirteen classes passed strict `unique top-1 >95%`, bounded lattice
coverage `>=99%`, complete-field retention `100%`, and class false-authority /
false-singleton `0 / 0`. The compatibility-only observer recorded four false
authorities and zero grounded-candidate losses; it remained outside runtime
authority and did not alter the primary verdict.

The release service was installed into an isolated model root with an
integrity-bound schema-v3 receipt and a unique socket. It did not connect to the
installed Lay runtime. Rebuilt measurements were:

```text
package bytes                                             77,962,328 B
package SHA-256          bf5a1619a89038466ef786305cf35eda5f4af5b9f12b9140f7d3cac407e2f2a7
terminal count                                                852,582
cold ready repeats                         836 / 991 / 793 / 774 ms
initial post-build observation                              1,029 ms WATCH
steady RSS                                                188,536 KiB
peak RSS                                                  205,780 KiB
direct p99, four runs                    3.938 / 3.938 / 3.914 / 3.858 ms
production-client p99                    4.433 / 4.232 / 4.330 / 4.117 ms
candidate fingerprint, four direct runs                         exact
service response failures / query failures                       0 / 0
service instances during measurement                                  1
temporary service after measurement                               stopped
```

The production-client measurement used the public persistent
`request_l11_seed_surfaces()` route, `Lattice(limit=32)`, 520 fixed surfaces,
one warmup pass, and four measured passes. The direct and production transport
p99 values pass `<=5 ms`; repeated isolated cold starts pass `<=1 s`; steady RSS
passes `<=250 MiB`. The first `1,029 ms` observation immediately after release
linking remains an explicit WATCH and is not deleted or relabeled.

Measured facts not covered by this verdict: rebuilt local installation,
rollback execution, managed daemon restart, physical application input, WeChat,
and the physical Double Shift undo gesture. Runtime authority remains the
previously installed Lay `1.0.28` V9 service. Global IBus was not restarted.

The complete local evidence copy is:

```text
/home/ubu/projects/lay-l1-exact-peak-search/docs/structural_gates/receipts/L1_L11_PHASE8I_INTEGRITY_CLOSURE_2026-08-15/evidence/direct-v9-proof-v3/
```

## 13. Corrective Release 1.0.29 Deployment

The integrity-closure source packet was built on the remote 20-core host with
`scripts/cargo-guard.sh build --release --bins`. The release build passed in
`76.01 s`, produced ten binaries totalling `45,346,928 B`, and used a maximum
RSS of `1,104,084 KiB`. All ten copied binaries matched the recorded size and
SHA-256 before installation and again under the installed names.

Before mutation, the complete Lay `1.0.28` runtime was captured at:

```text
/home/ubu/.local/lib/lay/rollback/1.0.28-pre-1.0.29-phase8i-integrity-20260815-170702
rollback bytes                                           789,183,548
rollback SHA-256 verification                         42 / 42 PASS
```

The V9 installer then admitted the byte-identical package only after validating
the schema-v3 full proof and independently published the immutable proof and
active receipts:

```text
package bytes                                             77,962,328 B
package SHA-256          bf5a1619a89038466ef786305cf35eda5f4af5b9f12b9140f7d3cac407e2f2a7
proof verdict                                           PASS_C_QUALITY
proof SHA-256            4983930495e793c1d28c7558fe006ddf8097ee575bebb1afd3f1dba4ddb1d01d
active receipt SHA-256   52076ffdf4f17e8b02d2334ec8d04fc10991a16c7dffccbf03849b8985c8eb2e
runtime admitted                                                   true
```

All ten Lay `1.0.29` binaries and the unchanged canonical L2 package were
installed. The GNOME extension source/runtime parity check passed and its live
DBus version became `1.0.29`. Only `lay-daemon.service` was restarted:

```text
daemon PID                                      3440152 -> 150902
L1.1 service PID                                3440215 -> 150984
global ibus-daemon PID                                3702 -> 3702
managed lay-ibus-engine PID                     3719399 -> 3719399
daemon count / L1.1 service count                              1 / 1
service state                                                   ready
terminal / atom count                              852,582 / 218,763
live requests served                                                8
live query failures                                                  0
journal integrity, query, and panic errors                            0
```

The running daemon and L1.1 service executable hashes equal the release hashes.
At the recorded steady observation, the cgroup used `376,770,560 B` with a
`378,494,976 B` peak. Individual RSS was `350,796 KiB` for the daemon and
`183,592 KiB` for L1.1; these values share mapped pages and must not be added as
an independent cgroup total.

Socket smoke exercised an exact winner, damaged-form target retention without
authority, a layout tie, and fail-closed abstention. It proved transport and
typed verdict delivery only; fixed quality remains owned by the full Gate C
proof. Package authority remains the same single V9 Phase 8I owner, with no V8
fallback or union route.

The managed `lay-ibus-engine` process was deliberately not restarted and still
maps its pre-deployment inode. Therefore this deployment does not claim active
byte parity for that already-running process. Real application typing, WeChat,
and autocorrection followed by Double Shift remain an explicit physical user
gate and are not inferred from daemon health.

Deployment receipt:

```text
/home/ubu/projects/lay-l1-exact-peak-search/docs/structural_gates/receipts/L1_L11_PHASE8I_INTEGRITY_CLOSURE_2026-08-15/deployment-receipt-v1.json
```

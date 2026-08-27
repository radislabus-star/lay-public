# DAFSA Typed View M3 Test Source Integration V1

Date: 2026-08-27

Paper verdict: `M3_TEST_SOURCE_INTEGRATION_DESIGN_READY`

Implementation authority: absent until an exact implementation preflight returns
`READY_TO_IMPLEMENT`.

## Question

How can the M3 typed-view mechanism enter the current V13 test source without
turning proof code into production authority, creating a second reload owner, or
claiming that the M3 machine result automatically transfers to different source
bytes?

## Immutable Evidence

```text
M3 terminal receipt
  a84355e42bad335d45b379c7e76d2b353bed6c23c30593e1c721be0c0058f324
  W1_DAFSA_TYPED_VIEW_PASS

M3 source/lifetime decision
  e7b0f66170776677c2b153254aa01a303fdf8538aec273678196dec723715b24
  M3_TEST_ONLY_TYPED_VIEW_SELECTED

source/lifetime structural V2 receipt
  c3800bb5cef5ff9f35c21b904b9f64eb7a249917055ce94599cafe7f32dac51b
  PASS / authority_ready=false

M3 experiment fragment
  149,110 B
  5a2e164c47c88677b74baf44d500c939749c98deff3092a1621086cf6e800875

current V13 test source
  124,127 B
  d9edfe7346b8636096701d4ac38044d5e386e6741a086a4d8da57ac15bffdf3b
```

## Critical Source Boundary

The positive M3 ELF did not compile the current `v13_typed_peak.rs`. It compiled
a recovered V10 prefix plus the sealed E1/D1/M3 diagnostic fragment. That
fragment contains the specialized packed-u64 traversal, measurement helpers,
route subjects and controller-facing evidence publishers. The current test
source still contains the earlier generic `TraversalKernel` search.

Therefore:

```text
M3 proves the typed-view mechanism in its sealed ELF.
M3 does not prove byte identity or codegen identity for this integration.
This integration must independently re-prove semantics and physical behavior.
```

Copying only the typed records into the generic traversal and quoting the M3
`11.7954%` gain would be invalid. Copying the complete diagnostic fragment into
the source tree would instead import unrelated route, process, perf and receipt
machinery. Both are forbidden.

## Current Ownership Graph

```text
production request
  -> TypingAssistWorker
  -> prepare_typing_assist_after_space
  -> candidate decoder / canonical field bridge
  -> Productive V90 package and cache generation
  -> DecisionCore / verifier

test-only exact-search proof
  -> #[cfg(test)] l2_field::v13_typed_peak
  -> V13DafsaView byte-format validator
  -> generic exact product traversal
```

M3 integration remains entirely in the second graph. It cannot call, replace,
rank for, publish to, or be imported by the production graph in this revision.

## Selected Source Design

The implementation may change exactly two source paths:

```text
edit
  src/nanda_wave/l2_field/v13_typed_peak.rs

create
  src/nanda_wave/l2_field/v13_typed_peak/typed_exact.rs
```

`l2_field/mod.rs` already guards `v13_typed_peak` with `#[cfg(test)]` and must
remain byte-identical. No daemon, bridge, cache, runtime, package-format or
production module may change.

The new submodule owns only:

```text
safe typed state and edge records
one immutable typed view
safe full-view materialization after byte-view validation
the minimal packed-u64 radius<=3 transition helpers used by M3
the minimal exact typed traversal and its parity observation
```

It must not own:

```text
files, environment variables, clocks or processes
Cargo, perf, PMU, CPU affinity or thread creation
receipts, markers, route dispatch or controller protocol
global static state, OnceLock, Mutex, RwLock or a cache
package discovery, reload or generation mutation
candidate ranking, DecisionCore, verifier or text mutation
unsafe code, unchecked indexing or native byte reinterpretation
```

## View Identity And Lifetime

The byte view remains the only format validator. Typed materialization may start
only from a successfully validated `V13DafsaView` and must copy every decoded
field through its safe accessors.

The typed object stores and verifies:

```text
state count
edge count
root state
V13 canonical package identity
symbol digest
state and edge record widths
payload bytes
```

One proof invocation constructs one typed view before processing any query and
borrows it for every forward and reverse search. It cannot be reconstructed in
a query loop. Dropping the proof owner drops the complete typed generation.

This proves the intended lifetime inside the test owner only. A future
production generation owner and reload transaction remain a separate decision.

## Exact Search Boundary

The minimal typed exact core preserves the measured M3 mechanism:

```text
packed radius<=3 Levenshtein row
precomputed equality masks per lane
fused seven-cell u64 advance
the accepted separate minimum scan
DFS product-state stack
edge order and reverse-schedule behavior
rank-prefix arithmetic
terminal-distance and certificate construction
budget, unresolved and scratch semantics
```

M2/M2R1 running-minimum variants are forbidden. Frontier-reduction, DAFSA
format changes, query-local DLA, new pruning, SIMD/SWAR, unsafe casts and
candidate caps are outside this integration.

## Consequence Analysis

### Candidate, lattice and false authority

The revision is compiled only in the test module. It cannot add, drop, rank or
authorize a production candidate. Test parity must nevertheless compare exact
retrieved form refs, peaks, certificate keys, completeness, rank prefixes,
terminal ranks and structural work. Any mismatch is terminal.

### Latency and tail behavior

No production latency claim is made. The M3 `22.9618 ns/edge` remains evidence
for the sealed M3 ELF only. After semantic integration passes, a separate
end-to-end route must measure the candidate in the actual owner graph.

### CPU, allocation and RSS

M3 measured a `3,689,628`-byte typed payload and approximately `1.63..1.68 ms`
one-time materialization outside traversal. The integration must record exact
counts and payload bytes and prove one construction per proof generation. It
must not claim RSS from payload arithmetic; RSS and allocator overhead require
an observed process measurement in the later owner route.

### Cache, package identity and reload

No cache or reload path is added. The test owner is lexical and stack-scoped:
validated byte view, typed materialization, searches, drop. The production
Productive V90 cache generation and explicit reload functions remain untouched.
A future production design must atomically bind typed identity to its package
generation rather than reuse this test lifetime implicitly.

### Learning, feedback and concurrency

The module receives no learning or usage state and emits no authority event. It
creates no thread and shares no mutable state. Existing stale-request rejection
and the one-slot `TypingAssistWorker` queue are unchanged.

### Failure and rollback

Materialization fails before search on count, width, decoded-field, root,
identity, digest or payload mismatch. Search preserves existing unresolved and
budget semantics. Source rollback is removal of the new module declaration and
file; no data, package, cache or runtime state needs migration.

### Compatibility and maintenance

The byte sidecar format and all public APIs remain stable. Keeping the fused
typed core in a dedicated submodule prevents diagnostic controllers from
entering source and keeps the current byte/generic oracle readable. If future
production authority selects this core, it must move through a separate module
ownership decision rather than making the test module importable at runtime.

## Design Comparison

### A. Dedicated test-only typed exact submodule - selected

This carries the measured mechanism, isolates the hot path, leaves the byte
validator and generic oracle unchanged, and gives focused parity tests one
stable entrypoint.

### B. Append the sealed M3 fragment - rejected

It would add thousands of lines of D1 schedules, affinity, filesystem,
measurement and controller protocol to a source module. Those routes do not own
the candidate mechanism.

### C. Refactor the generic traversal behind a shared accessor trait - rejected

It changes the oracle and candidate together, risks virtual/generic codegen
drift, and weakens the independent parity denominator. It is not the mechanism
measured by M3.

### D. Add a production global typed cache now - rejected

It would create an unproved generation/reload owner, allocate production RSS and
cross the authority boundary before end-to-end proof.

## Proof Plan

The implementation is accepted only if all of these pass locally through
`scripts/cargo-guard.sh`:

```text
1. source route closure
   - only the two admitted source paths change
   - l2_field/mod.rs and production files byte-identical
   - no forbidden API or side-effect token in the new module

2. small deterministic fixtures
   - every typed state/edge field equals the validated byte view
   - root, identity and symbol digest equal
   - corruption remains rejected by the byte validator

3. transition parity
   - packed transition equals the existing banded-row oracle
   - all radius<=3, supported query lengths and generated symbol classes

4. exact-search parity
   - forward and reverse schedules
   - exact form refs, peaks, certificate keys and completeness
   - rank prefixes, terminal ranks and all structural work counters

5. fixed 382-case closure
   - exact package 140,556,462 B / cce259fe...
   - exact sidecar 3,689,884 B / a1aa95be...
   - exact V7 evidence 1,606,189 B / 33fded73...
   - one typed materialization for the complete proof generation
   - payload exactly 3,689,628 B

6. build and route boundary
   - focused lib test build and tests only
   - no install, restart, daemon, network, remote, perf or PMU
   - runtime authority changed = false
```

Generated fixtures may test transition mechanics, but the scientific parity
claim depends on the fixed real 382-case closure. Generated fixtures alone can
never promote the candidate.

## Structural Closure

The source-provenance, mechanism/lifetime and proof/authority routes were
checked independently. All three returned coherence-only `PASS` with no weak
triad, conflict, evidence gap, repair item or owner conflict. As required for a
paper contract, none claims implementation authority:

```text
Route A source receipt SHA-256
  ef28f61f08eef5291e5508127230c59781165eb8052fcbf89c1b73768278af61
Route B mechanism receipt SHA-256
  d77802d10d16637c9b1b9c46cbb77eb28373afe5c3e28c07e58b1280ed171684
Route C proof-boundary receipt SHA-256
  f96c7819f47646594dee81db5039251ca9c32037e79a65a3e5db5bdc42c7851c
authority_ready
  false / false / false
```

## Verdicts

```text
M3_TEST_SOURCE_INTEGRATION_PASS
BLOCKED_SOURCE_DRIFT
BLOCKED_BUILD
BLOCKED_PARITY
BLOCKED_LIFETIME
BLOCKED_PROVENANCE
```

Any failure retains the exact source and logs and grants no automatic claim or
production successor.

## Next Tree

```text
M3_TEST_SOURCE_INTEGRATION_DESIGN_READY
  -> structural route PASS
  -> implementation preflight READY_TO_IMPLEMENT
  -> two-path test-only source edit
  -> focused compile and deterministic parity
  -> fixed 382-case local parity and lifetime closure
  -> M3_TEST_SOURCE_INTEGRATION_PASS
  -> separate actual-owner consequence paper
  -> end-to-end p99 + RSS + generation/reload proof
  -> production authority decision
```

No positive result in this paper or its test-only implementation directly
admits production source edits, install, restart or deployment.

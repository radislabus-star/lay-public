# L1.1 Phase 7: Typed Edit-Product Traversal Paper

Status: Phase 7A implemented and proven; Phases 7B-7D pending
Date: 2026-08-14
Owner: proof-only `L1TypedEditTraversal` under `L1PeakSearch`

## 1. Decision

Phase 7 will traverse the Phase 6 decoder trie with typed edit states. It will
not create corrected candidate strings and it will not scan or rank a bounded
candidate list first.

```text
L1QueryField
  lexical symbols + raw/layout symbols + punctuation metadata
-> one L1TypedEditTraversal
  DecoderNodeId x input position x exact operator program
-> terminal events keyed by WordCenterId
  operator certificate + positions + geometry
-> proof comparison only
```

This phase does not settle candidates, change scoring, issue authority, or
modify package bytes. It proves which lexical centers are exactly reachable by
the declared typed geometry. Phase 8 later combines these events with complete
posting search under one `L1PeakSearch`.

## 2. Required Input Geometry

One normalized character stream is insufficient. The query field must expose
two symbol lanes owned by one traversal:

```text
lexical lane
  trim surrounding whitespace
  strip lexical boundary punctuation
  lowercase
  used by identity and edit operators

layout lane
  trim surrounding whitespace
  lowercase without stripping keyboard punctuation
  used only by exact keyboard-layout projection

boundary metadata
  raw leading/trailing punctuation spans
  used to distinguish punctuation suffix from lexical identity
```

This is required for cases such as raw `,` on the US layout mapping to Russian
`б`. Normalizing punctuation before layout traversal destroys the only input
symbol and is therefore a query-encoding loss, not a ranking failure.

The two lanes are not two candidate owners. They are typed observation views
inside one traversal and merge into one terminal-event map.

## 3. Identities

The implementation must keep these namespaces distinct:

```text
DecoderNodeId       package trie node
WordCenterId        primary lexical center
InputPosition       offset in one query symbol lane
TargetPosition      depth on a decoder path
OperatorProgram     exact remaining edit automaton state
TypedCertificate    completed operation and positions
TraversalState      query-local product state
```

A decoder node is not a WordCenterId. Relation-bank `decoder_terminal` fields
are peer WordCenterIds and cannot enter traversal terminal lists.

## 4. Primitive Transitions

Let `O` be the selected observed symbol lane, `i` its current position, `v` the
current decoder node, and `child(v, x)` the deterministic Phase 6 transition.

```text
MATCH(x)
  pre:  O[i] = x and child(v, x) exists
  post: (child(v, x), i + 1)

TARGET_INSERT(x, p)
  meaning: target has a symbol missing from observed
  pre:  child(v, x) exists
  post: (child(v, x), i), record target position p

INPUT_DELETE(p)
  meaning: observed has an extra symbol
  pre:  i < len(O)
  post: (v, i + 1), record input position p

SUBSTITUTE(x, p)
  pre:  i < len(O), x != O[i], child(v, x) exists
  post: (child(v, x), i + 1), record aligned position p

ADJACENT_TRANSPOSE(p)
  pre:  child(v, O[i+1]) = a and child(a, O[i]) = b
  post: (b, i + 2), record position p

LAYOUT_PROJECT(x)
  pre:  x is the opposite-layout symbol for raw O[i]
        and child(v, x) exists
  post: (child(v, x), i + 1)
```

No primitive allocates a target surface. Child traversal operates on Unicode
symbols and node IDs only.

## 5. Canonical Certificates

Certificates are structural output, not scores:

```text
Identity
PunctuationSuffix { raw_start, raw_len }
PrefixTruncation { target_position = 0 }
SuffixTruncation { target_position = target_len - 1 }
MissingLetter { target_position }
ExtraLetter { input_position }
SingleSubstitution { position }
KeyboardLayout { direction }
AdjacentTransposition { position }
NonAdjacentTransposition { first, second }
RepeatedFragment { input_start, source_target_start, len = 2 }
DoubleSubstitution { first, second }
OmissionTransposition { omitted_target, transposed_target }
SparseMultiOmission { first, second }
```

One missing target symbol is canonicalized as prefix, suffix, or interior
missing. It is not emitted under all three names. Different valid positional
witnesses remain distinct certificates when repeated symbols make more than
one edit alignment possible.

Punctuation suffix may coexist with normalized identity because they answer
different questions: the lexical surface is identical, while raw input has a
typed boundary operation.

## 6. Operator Programs

### 6.1 Phase 7A

```text
Identity
  MATCH* then terminal at i = len(O)

PunctuationSuffix
  lexical Identity plus independently verified trailing raw punctuation

Prefix/SuffixTruncation
  exactly one TARGET_INSERT plus MATCH*
  classify by inserted target position at terminal
```

### 6.2 Phase 7B

```text
MissingLetter
  exactly one TARGET_INSERT at an interior target position

ExtraLetter
  exactly one INPUT_DELETE

SingleSubstitution
  exactly one SUBSTITUTE

KeyboardLayout
  LAYOUT_PROJECT for every raw symbol, no mixed lexical edits
```

Single substitution is a generic one-symbol target relation. The fixed
heldout generator uses alphabet successors, but runtime traversal must not
encode that generator-specific restriction.

### 6.3 Phase 7C

```text
AdjacentTransposition
  exactly one ADJACENT_TRANSPOSE

NonAdjacentTransposition
  at first mismatch choose a target child x and retain:
    saved_target_symbol = x
    saved_observed_symbol = O[i]
    first_position
  MATCH at least one intermediate position
  close only when O[j] = saved_target_symbol and the target child at j is
  saved_observed_symbol; require j >= first + 2

RepeatedFragment
  MATCH*; at input position i consume exactly two input symbols without a
  decoder transition only when that symbol pair occurs as a contiguous pair
  in the already traversed target prefix; then MATCH* to terminal
```

`DecoderNodeId` uniquely identifies the traversed target prefix, so repeated
fragment validation may inspect its ancestor chain. The state does not need to
copy or generate the prefix string.

### 6.4 Phase 7D

```text
DoubleSubstitution
  exactly two SUBSTITUTE transitions at distinct positions

SparseMultiOmission
  exactly two TARGET_INSERT transitions at distinct target positions

OmissionTransposition
  product of exactly one TARGET_INSERT and exactly one
  ADJACENT_TRANSPOSE, in either temporal order
```

The omission/transposition product retains both positions in state. Paths with
different operation order are merged only after their canonical completed
certificate is identical.

## 7. State Key And Dominance

The safe dominance rule is exact future equivalence, not lower scalar edit
cost.

```text
TraversalStateKey = (
  lane,
  DecoderNodeId,
  InputPosition,
  OperatorProgram,
  recorded positions,
  pending transposition symbols,
  pending minimum-gap state
)
```

Two states may be deduplicated only when every field in this key is equal. In
that case they have the same future trie transitions and produce the same
certificate set. No state is removed because another state has a lower edit
count or larger score.

There is no queue-size cutoff in Phase 7. Work is finite because input length
is at most `32`, the trie is finite, and every operator program has a fixed
edit budget of at most two.

## 8. Output Contract

```text
BTreeMap<WordCenterId, BTreeSet<TypedCertificate>>
```

Terminal events come only from Phase 6 terminal lists. Results are sorted by
WordCenterId and certificate order before serialization. Traversal scheduling,
child order, worker count, or hash-map iteration cannot alter output bytes.

Required counters:

```text
states generated
states deduplicated
states expanded
queue peak
decoder edges examined
terminal nodes reached
WordCenter terminal events
unique WordCenterIds
certificates emitted
```

## 9. Independent Dense Oracle

Traversal cannot prove itself. The proof oracle enumerates every primary
WordCenterId in a tiny package, decodes its target surface, and derives all
valid certificates by direct bounded sequence relations:

```text
remove target position(s) and compare
remove input position(s) and compare
substitution mismatch positions
swap target position pair and compare
remove repeated input bigram and verify an earlier target occurrence
compose one omission with one adjacent swap and compare
project every raw input symbol through the keyboard map and compare
```

The direct oracle does not traverse the trie and the traversal does not call
the direct recognizers. Equality requires the complete mapping:

```text
WordCenterId -> exact set of TypedCertificate
```

For every family and every tiny query:

```text
traversal terminals == dense terminals
traversal certificates == dense certificates
```

Target labels cannot enter either search. Labels are used only by the fixed
full-matrix projection after traversal has completed.

## 10. Proof Ladder

Each family is admitted separately:

```text
paper route gate
-> implementation preflight
-> exhaustive tiny-package terminal/certificate parity
-> state-order forward/reverse/permuted parity
-> fixed full matrix target retention for the entire family
-> aggregate regressions
-> architecture receipt
```

Full fixed-matrix gates:

```text
7A: identity + punctuation + prefix + suffix cases
7B: all 7A plus missing + extra + substitution + layout
7C: all previous plus adjacent/non-adjacent transposition + repeated fragment
7D: all 13 classes, exactly 260,000 cases
```

Every case must retain its target terminal with at least one certificate of the
declared class. Final Phase 7 requires:

```text
target terminal recall       100% in every class
certificate recall           100% in every class
false class certificate      0 against the direct oracle on exhaustive tiny data
state-order parity            exact bytes
generated target strings      0
literal runtime examples      0
queue truncations              0
```

## 11. Resource Gates

Phase 7 is proof-only, but work must be measured before Phase 8:

```text
p50 / p95 / p99 states expanded by class
p50 / p95 / p99 queue peak by class
p50 / p95 / p99 terminal events by class
maximum state count by class and length
wall time and peak RSS for fixed matrix
```

Failure to meet a production latency projection is not a reason to weaken
completeness in Phase 7. It becomes a Phase 8 indexing/bound problem.

## 12. Rejected Shortcuts

- Generating corrected strings and looking them up in the dictionary.
- Scanning the current top-32/top-128 candidate list.
- Reusing the heldout class label to select an operator program during search.
- Collapsing all states to scalar edit distance.
- Treating adjacent and non-adjacent transposition as one positional witness.
- Stripping punctuation before the layout lane.
- Reading relation-center `decoder_terminal` as a decoder node.
- Dropping states to satisfy a queue or latency target.
- Adding word, suffix, language-example, source-ID, or fixture-specific runtime
  conditions.

## 13. Rollback

Each family patch is isolated. A failed 7B, 7C, or 7D patch is removed without
removing an earlier proven family. Until all Phase 7 gates pass, the entire
traversal remains proof-only and runtime authority stays with
`LegacyBirthSearch`.

## 14. Phase 7A Measured Result

Phase 7A was implemented in
`src/nanda_wave/lexical_grokking/typed_edit_traversal.rs`. It remains compiled
only for tests or the `lexical-compiler` proof tool. The production runtime,
service, `L1PeakSearch` owner, package format, installed package, daemon, and
IBus route are unchanged.

The implementation has one exact traversal owner and four admitted completed
certificates:

```text
Identity
PunctuationSuffix
PrefixTruncation
SuffixTruncation
```

The query encoder preserves three distinct observations: normalized lexical
symbols, unstripped raw/layout symbols, and boundary-punctuation spans. Exact
state equality is the only deduplication rule. There is no scalar edit cost,
queue cutoff, generated corrected surface, target-aware transition, or literal
runtime case.

An independent direct oracle enumerates every primary center in tiny packages,
decodes its target surface, and derives certificates by direct sequence
relations. Across every string of length zero through four over a three-symbol
alphabet, including one- and two-character punctuation suffixes, traversal and
oracle produced the identical complete mapping from `WordCenterId` to the exact
certificate set. Forward, reverse, and deterministic-permuted schedules also
produced identical terminal and metric bytes. Focused gate: `5/5 PASS`.

The fixed real-package gate used the unchanged `852,582`-center V8 package and
the same fixed-heldout sampler as the accepted Phase 5 proof. Results:

| Dimension | Measured |
|---|---:|
| clean dictionary round-trip | `852,582 / 852,582` |
| clean identity retention | `852,582 / 852,582` |
| prefix target/certificate/schedule | `20,000 / 20,000` each |
| suffix target/certificate/schedule | `20,000 / 20,000` each |
| punctuation target/certificate/schedule | `20,000 / 20,000` each |
| fixed Phase 7A cases | `60,000` |
| proof wall time | `9.14 s` |
| average CPU | `1,027%` |
| peak RSS | `591,208 KiB` |
| package hash changed | `no` |

Maximum measured work was `392` expanded states, queue peak `98`, and `53`
unique WordCenter certificate events. Class p99 expanded states were `311` for
prefix and `324` for both suffix and punctuation. This is proof-tool work, not
production hot-path latency.

The package SHA-256 before and after remained
`47fa757acac03b0f76e5397e965b9127884e245e9845ce0f1ca0896fb40f33e9`.
The full Phase 7A evidence SHA-256 is
`f27383188933ef92137f939ee8947d3a59d664d494dc346e790aae3ab50fde89`.
Lexical regression `134/134`, transition authority `20/20`, mutation monopoly
`15/15`, default and feature compile, changed-tree, format, diff, and Cargo disk
budget gates passed.

The first remote compile attempt is retained as rejected evidence: a test-only
length variable inferred as `u32` where `Vec` required `usize`. No proof or
runtime code executed in that attempt. The type was corrected without changing
the paper contract; the repeated gate passed.

Tested: only the four Phase 7A certificate families, complete tiny-oracle map
parity, schedule parity, fixed target and certificate retention, package and
corpus identity, resource counters, and production-source isolation. Not
tested: the 7B-7D operators, complete 13-class typed traversal, posting bounds,
settlement parity, production latency, package representation, or deployment.

```text
/home/ubu/projects/lay-l1-exact-peak-search/docs/structural_gates/receipts/L1_L11_PEAK_SEARCH_PHASE_7A_2026-08-14/phase-7a.json
/home/ubu/.cache/lay/l1-peak-search-phase7a-2026-08-14/full-3x20000.json
```

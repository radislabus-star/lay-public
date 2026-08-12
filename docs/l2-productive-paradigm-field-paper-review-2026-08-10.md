# L2 Productive Paradigm Field: Paper Review

Status: `PASS_PAPER_IMPLEMENTATION`, `CODE_NOT_IMPLEMENTED`,
`RUNTIME_AUTHORITY_UNCHANGED`.

Date: 2026-08-10.

Reviewed documents:

```text
/home/ubu/projects/lay/docs/l2-productive-paradigm-field-canonical-design.md
/home/ubu/projects/lay/docs/l2-productive-paradigm-field-paper-implementation.md
/home/ubu/projects/lay/docs/l2-l11-canonical-architecture.md
/home/ubu/projects/lay/docs/lay-development-plan-after-0.2.340.md
```

## 1. Verdict

The architecture is complete at paper level. A developer can implement V1
without selecting an unspecified graph, score, calibration method, package
layout, split, concurrency owner, or authority rule inside runtime code.

This verdict does not mean that the implementation exists or that any quality,
latency, memory, package, daemon, IBus, or product proof passes.

## 2. Original Review Closure

| Review finding | Paper resolution |
|---|---|
| V39 was not truly unseen | Four separate scopes: BANK_UNSEEN, SLOT_HELDOUT, LEMMA_HELDOUT, CORPUS_UNSEEN. |
| Trie/FST path-state error | V1 is prefix-trie only; a convergent FST is excluded. |
| Undefined evidence algebra | Fifteen typed features, fixed polarity, constrained pairwise logistic objective, deterministic optimizer, Q16.16 runtime score. |
| Top-1 conflicted with honest ambiguity | UNIQUE_RESOLVABLE, MULTI_LABEL, and UNSUPPORTED are separate frozen denominators. |
| One reduce conflicted with calibration | One raw pass, train reduce, frozen coefficients, then calibration-spool replay. |
| L2/L3 ownership was broad | L2 window is exactly two committed lexical tokens on each side; wider/semantic evidence belongs only to L3. |
| Full logical lattice conflicted with 32 | Every productive terminal is evaluated; exact top-32 is a separate lane; all grounded L1.1 candidates remain protected. |
| Complexity omitted paths/lemmas | Complexity counts emitted scalar transitions across every active lemma. |
| Performance/RSS gate was subjective | Package, steady RSS, peak RSS, cold publish, and single/multi-client p99 have numeric ceilings and a fixed workload. |
| Grounded Winner was incompletely protected | Downgrade requires an independently calibrated contradiction certificate. |
| Product scope was ambiguous | V1 supports inflection of lexically grounded lemmas; unknown lemmas and unrestricted derivation are explicit non-goals. |
| Scientific basis was absent | Finite-state morphology, edit traversal, OSA, contextual morphology, calibration, and reject-option references are listed. |

## 3. Additional Defects Closed During Paper Review

The second review also closed:

- imported base IDs are never renumbered by the sidecar;
- a truly productive-heldout lemma receives only a cold binding derived from
  immutable lexical observations after train parameters are frozen;
- heldout identities are excluded from train context neighbor positions;
- edit-program execution and alignment tie-breaking are exact;
- transferable templates require at least two distinct train lemmas;
- normalized event identity, deduplication, raw-source manifest ownership, and
  five-fold lemma splits are deterministic;
- the graph is a radix-built, prefix-split trie with no convergent path state;
- runtime calibration uses observable candidate provenance and a derived edit
  class, never oracle proof labels;
- scene-wave generation contains no literal service-word or phrase branches;
- the 60-cell positive, anti, hard-negative, and ambiguity banks are trained
  independently with evidence-selected subcenter counts;
- generated-path atoms reproduce the real V39 eight-channel character/keyboard
  profile and 64+64-bit wave code through a DFS refcount/undo accumulator;
- V39 speed-parity and trained productive modes have separate hashed package
  flags and comparators;
- all required package records, section IDs, opcodes, hashes, and byte order are
  specified;
- apply authority is external to model bytes and requires a package-, binary-,
  receipt-, and product-matrix-bound promotion manifest;
- cold binding uses a compatibility postings index rather than a global
  paradigm scan;
- the paper maps current `productive.rs`, `runtime.rs`, `productive_format.rs`,
  `model.rs`, `context.rs`, and `bridge.rs` ownership to the new modules;
- the L3 bridge must carry up to 32 grounded plus 32 productive identities and
  may not flatten them back to one top-32 array.

## 4. Paper Gate Checklist

```text
scope and non-goals                               CLOSED
fixed limits and protected lanes                  CLOSED
L1.1 / L2 / L3 / verifier ownership              CLOSED
typed morphology identity                         CLOSED
train/calibration/heldout provenance              CLOSED
one-raw-pass compiler stages                      CLOSED
paradigm induction and compatibility              CLOSED
edit-program interpreter                          CLOSED
prefix graph and exact terminal coverage          CLOSED
character and keyboard OSA state                  CLOSED
incremental atom/phase state                      CLOSED
multimodal positive/anti/ambiguity field          CLOSED
learned score and deterministic optimizer         CLOSED
calibration and contradiction certificate         CLOSED
composite lattice and L3 handoff                  CLOSED
mmap package and delta format                     CLOSED
concurrency owner                                 CLOSED
complexity and numeric resource budgets           CLOSED
fixed proof protocol                              CLOSED
scientific foundations                            CLOSED
```

## 5. Current Source Boundary

The rejected V42 bounded-chunk code still exists locally at:

```text
/home/ubu/projects/lay/src/nanda_wave/l2_field/runtime.rs:518
```

The first code step remains restoration of V39 behavior while preserving V42
receipts. The paper documents do not authorize deletion of unrelated dirty work,
package installation, daemon restart, or IBus restart.

## 6. Required Implementation Evidence

Paper completion moves the project to code and proof; it does not skip them.
The next accepted evidence sequence is:

```text
V39 source restoration receipt
-> package V1 roundtrip and corruption receipt
-> shared geometry and candidate identity parity receipt
-> train/calibration manifest and model receipt
-> per-class SLOT_HELDOUT and LEMMA_HELDOUT quality receipt
-> clean/ambiguity/grounded-authority receipt
-> package/RSS/latency receipt
-> L3/verifier replay receipt
-> physical daemon/IBus application matrix
```

Generated forms remain `SuggestOnly` until the entire sequence and every
conjunctive gate pass.

## 7. Static Review Evidence

```text
git diff --check                                           PASS
canonical design fences                         96, balanced
paper implementation fences                    144, balanced
paper review fences                             10, balanced
canonical design lines                               1,235
paper implementation lines                           2,005
paper review lines                                     154
graphify update nodes / edges / communities   12,918 / 33,074 / 566
graph query sees paper + bridge/runtime/productive/V39               PASS
```

The three owning documents and both linked architecture/plan documents exist at
their absolute paths. The graph query starts from `19.1 Existing ownership
bridge`, the paper implementation, `bridge.rs`, `productive.rs`, `runtime.rs`,
and the V39 baseline node.

No Cargo build, runtime proof, package compilation, daemon install, daemon
restart, or IBus restart was performed. Static document checks do not count as
runtime tests.

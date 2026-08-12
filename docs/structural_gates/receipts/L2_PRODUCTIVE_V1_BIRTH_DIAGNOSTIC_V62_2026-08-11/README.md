# L2 Productive V1 Birth Diagnostic V62

The machine-readable read-only proof result is stored in `receipt.json`.

This diagnostic adds lemma, slot, and exact-surface birth denominators. It does
not change package bytes or runtime authority.

Measured aggregate:

| Cohort | Cases | Target lemma born | Target slot born | Exact target born | Top-16 | Top-1 | Empty lattice |
|---|---:|---:|---:|---:|---:|---:|---:|
| `SEEN_EXACT` | 1,300 | 1,300 | 833 | 833 | 833 | 436 | 0 |
| `LEMMA_HELDOUT` | 1,300 | 117 | 101 | 101 | 100 | 20 | 1,183 |

Parking verdict:

- `SEEN_EXACT` loses the target at slot birth before top-16 ranking. Every
  exact target that is born reaches top-16.
- `LEMMA_HELDOUT` primarily lacks a cold lemma binding. The paper-required
  complete `LexicalLemmaObservationV1` route is not implemented in the packaged
  runtime.
- Next implementation work starts at the lexical-observation/cold-binding
  ownership and geometry-aware bounded slot selection. It must not change
  coefficients, authority gates, L1.1, or canonical L2.

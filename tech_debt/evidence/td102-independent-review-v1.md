# TD-102 Independent Review V1

Date: `2026-08-30`

Reviewer context: fresh read-only subagent.

Score: `6/10`

Verdict: `ACCEPT_WITH_FIXES`

## Findings

1. The two initial lanes measured target selection inside the same unsplit
   library, not a crate-separation prototype. The verdict therefore had to say
   that split benefit was unproven, not that no net benefit was measured.
2. TD-104 needed to cover unconditional compiler/proof/eval roots in lexical,
   L2, L3/context, L4, and top-level eval code, not only the existing partial
   `lexical-compiler` feature.
3. The spectral ledger was an estimate and could not be used as measured
   support.
4. One run per lane and remote `/tmp` logs were insufficient durable evidence.
5. The initial percentage and module visibility counts required correction.

## Resolution

- Verdict changed to `WORKSPACE_SPLIT_NOT_ADMITTED_BENEFIT_UNPROVEN`.
- Three clean repeats per lane were executed on the mini-PC.
- All raw logs and a verified `SHA256SUMS` were copied into
  `td102-remote-raw-v2/`.
- Measurement V2 records paired overhead and explicitly lists unmeasured split
  effects.
- Module visibility counts and arithmetic were corrected.
- The spectral ledger is labeled estimated and non-authoritative.
- TD-104 scope now covers all unconditional cold roots.

A fresh second-pass review is required before completion.

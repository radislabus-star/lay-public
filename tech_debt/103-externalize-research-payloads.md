# TD-103: Externalize Research Payload Lifecycle

Status: `DONE`
Priority: `P2`
Class: repository and evidence storage
Size: `L`
Decision dependency: immutable evidence policy approval

## Evidence

- The local `docs/` tree occupies about 3.6 GB.
- Sixteen ignored receipt payloads exceed 10 MB; several sealed debug ELFs are
  about 318-321 MB each.
- Git tracks 3,072 compact receipt files totaling about 89 MB.
- `scripts/` contains 112 one-shot V10/V11 controllers totaling 99,664 lines.
- `.gitignore` correctly keeps nested payloads out of Git, but the checkout still
  acts as a long-lived artifact store and active script navigation remains noisy.

## Proposed Outcome

Preserve compact receipts, hashes, command manifests, and source snapshots in
Git. Store large local payloads in a content-addressed evidence root outside the
checkout, with a deterministic materialize/verify command. Catalogue completed
controllers as frozen reproducibility artifacts so active scripts remain easy to
navigate.

## Non-Negotiable Constraints

- Never delete the only copy of sealed evidence.
- Never rewrite historical receipt bytes or hashes.
- No migration is complete without full pre/post SHA verification.
- Active paper references must remain resolvable through a documented manifest.
- Do not use Git LFS by default; compare it with local/object-store alternatives
  and the public clone experience first.

## Decision Questions

- Which storage root is backed up and durable enough for authoritative payloads?
- Must public users reproduce old experiments or only verify compact receipts?
- Should frozen controllers move, be indexed in place, or be generated from
  receipt snapshots?
- What disk budget should a normal source checkout enforce?

## Required Preflight If Admitted

- Complete payload inventory with SHA, size, current path, and backup count.
- Recovery drill for one ELF, one perf payload, and one source snapshot.
- Atomic migration and rollback plan.
- Repository clone-size and navigation before/after measurements.

## Rejection Condition

Reject any cleanup that improves disk appearance by weakening evidence,
reproducibility, or recovery.

## Implemented Decision

Decision: `CONTENT_ADDRESSED_LOCAL_OWNER_ADMITTED`.

The existing `/home/ubu/projects/lay-immutable-evidence` root is the sole local
owner for the selected large payload bytes. This is explicitly not called a
backup: independent backup count is `0` before and after, and the owner remains
on the same filesystem as the checkout. That preserves the prior disk-failure
boundary rather than pretending to improve it. A separate backup policy may
copy the sealed objects to another failure domain later without changing these
historical projections.

The admitted migration covers 18 ignored receipt paths, 3,563,522,494 logical
bytes, and 15 unique objects totaling 2,604,090,366 bytes. Every original path
is now a relative symlink to its exact SHA-256 object. The object tree and all
13 payload parent directories are sealed `0555`; files retain their original
`0444` or `0555` modes. The writable transaction journal is outside the object
tree.

Measured outcome:

- `docs/` apparent size after migration: 270,600,104 bytes;
- duplicate physical bytes removed: 959,432,128;
- final projections: 18 symlinks / 0 regular / 0 missing;
- final object verification: 15/15;
- parent modes: 13/13 exact;
- recovery drill: ELF, `perf.data`, and Rust source materialized and
  re-externalized with exact bytes;
- historical controller ledger: 122/122 PASS;
- fault/security unit suite: 29/29 PASS;
- independent reviews: `4/10 REVISE`, then `5/10 REVISE`; every listed
  finding was corrected after the second and final permitted review, with no
  fabricated third score;
- runtime, Cargo, perf, installation, and service state: unchanged.

Lifecycle tool: `scripts/research-evidence-store.py`.

Evidence:

- `evidence/td103-payload-inventory-v1.tsv`
- `evidence/td103-parent-mode-inventory-v1.tsv`
- `evidence/td103-implementation-preflight-v1.json`
- `evidence/td103-implementation-preflight-receipt-v1.json`
- `evidence/td103-payload-catalog-v1.json`
- `evidence/td103-object-seal-receipt-v1.json`
- `evidence/td103-final-verification-v1.json`
- `evidence/td103-recovery-materialize-receipt-v1.json`
- `evidence/td103-recovery-externalize-receipt-v1.json`
- `evidence/td103-independent-review-v1.md`
- `evidence/td103-independent-review-v2.md`
- `evidence/td103-completion-v1.json`
- `evidence/td103-SHA256SUMS`

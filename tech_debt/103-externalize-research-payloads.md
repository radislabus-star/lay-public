# TD-103: Externalize Research Payload Lifecycle

Status: `DISCUSSION_REQUIRED`
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

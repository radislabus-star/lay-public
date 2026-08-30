# TD-103 Consequence Analysis V1

Base commit: `d9b79fca`

## Decision

Admit a content-addressed lifecycle for the 16 ignored receipt payloads larger
than 10 MiB plus one `perf.data` and one source snapshot used by the recovery
drill. The external owner is the existing
`/home/ubu/projects/lay-immutable-evidence` tree. Historical receipt paths stay
resolvable as symlinks to verified immutable objects.

## Measured Scope

- 18 receipt projections;
- 3,563,522,494 logical bytes;
- 15 unique SHA-256 objects;
- 2,604,090,366 unique object bytes;
- 959,432,128 duplicate bytes removable;
- all 18 paths are Git-ignored;
- all 13 payload parent directories are owner-controlled and sealed `0555`;
- independent backup count before migration is zero.

## Transaction

For each entry:

1. verify the regular source against the sealed inventory;
2. copy to a temporary object, fsync, chmod read-only, and atomically publish;
3. verify object size and SHA-256;
4. journal the parent mode, temporarily add owner-write, and fsync the directory;
5. rename the source to a transaction backup;
6. atomically publish a symlink from the historical path to the exact object;
7. verify the historical path still reads the same bytes;
8. remove the transaction backup, restore the exact parent mode, and fsync.

A durable JSONL journal records every boundary. A rerun must recover a stranded
backup or temporary link without recreating, changing, or losing payload bytes.
An interrupted parent-mode transition is recovered from the durable journal
before another action starts.

## Compatibility

- Receipt paths remain openable and `Path.is_file()` remains true.
- Payload bytes, sizes, and target modes remain unchanged when dereferenced.
- Tracked compact receipts and historical controller paths are not edited.
- `materialize` can restore a regular file atomically; `externalize` can return
  it to the verified object projection.
- The store is an ownership move on the same filesystem, not an independent
  backup. The catalog reports that distinction explicitly.

## Failure Boundary

No original path is removed before a complete object exists. Any failure before
object publication leaves the original regular file untouched. Any failure
after the backup rename leaves either the backup or the verified symlink and is
recoverable from the journal and inventory.

No Cargo, perf, PMU, subject, installation, service restart, network request, or
runtime authority change is admitted.

# Slice 7A Local Synchronous Durability Verdict

Status: `REJECTED_LOCAL_SYNCHRONOUS_STORAGE_FOR_SLICE7`

Tested:

- buffered fixed-offset records with `fdatasync` in a preallocated ext4 file;
- direct fixed 4 KiB aligned slots with `O_DIRECT | O_DSYNC`;
- one bounded owner and queue capacity 32;
- prepare, co-commit, tail flush and next-native barriers;
- 64 warmups plus 1,000 measured samples per stratum;
- checksum, torn slot, foreign generation, ring wrap, saturation and
  synchronization failure behavior;
- focused output-transaction tests, `8/8` passed.

Measured:

```text
strategy    stratum       p99 / maximum
buffered    prepare       1.265 / 325.387 ms  FAIL max
buffered    co-commit     1.164 /   1.677 ms  PASS
direct      prepare       1.657 / 330.324 ms  FAIL max
direct      co-commit     1.575 / 162.240 ms  FAIL max
direct      tail flush    1.483 / 154.736 ms
direct      next native   1.593 /  14.571 ms
direct fault matrix                              PASS
```

Verdict scope: buffered ext4 and direct aligned-slot ext4 are both rejected for
the Slice 7 durability contract. The unchanged p99 gate passes, but the strict
maximum gate does not. No third local-storage strategy is admitted. The
abstract ordered transaction state machine is not a live owner and was not
deployed.

Next architecture gate: a new `BackendAtomicReceiptV1` design receipt and
kill-point proof. Slice 7B remains blocked.

Not tested: an accepted backend atomic receipt, live crash/recovery, integrated
IBus latency or physical applications.

Runtime authority changed: `false`.

Installed runtime touched: `false`.

Exact raw receipts: `final-ext4-preallocated-receipt.json` and
`final-direct-aligned-slot-receipt.json`.

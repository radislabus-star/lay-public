# V65 Anchor Recovery Receipt Index

Status: `REJECT_not_closed_and_not_latency_bounded`.

## Tested

- resumed package build from frozen V64/V63 artifacts;
- byte identity of the base `.p2m` against V64;
- mmap recovery sidecar format and evidence support;
- fixed shadow proof at `13 x 10 x 2` and `13 x 100 x 2`;
- H/B/S0/S1/S2/S3/R, probe parity, false singleton, integrity, RSS, package
  bytes, and closed-call latency.

## Measured

```text
base .p2m bytes           17,309,944
base .p2m sha256          9fd8c950398fb8ba47a2c9f2236880239d9f4376b191a691b0d01c47ddd3e438
recovery .p2r bytes        3,514,208
recovery .p2r sha256       744a11a8c88921bc3b63899840982ecadc3260485808dd60ea4858bcd54a8436
recovery paths                36,915
LEMMA_HELDOUT H/B/R    1,281 / 1,277 / 1,277
H -> B                            4
B -> S0                           0
top-16                        1,276
top-1                           205
probe parity             2,600 / 2,600
false singleton                    0
integrity errors                    0
peak proof RSS             235,668 KiB
proof time               1,951.507 s
```

## Not Tested

- slot-heldout, multi-label, and unsupported cohorts;
- integrated L1.1/L2/L3/L4/DecisionCore/verifier authority transfer;
- queue-inclusive single-client and 20-client latency;
- daemon, IBus, or physical product matrix.

Runtime authority changed: `false`. No package was installed.

## Files

- `build-receipt.json`: package and sidecar build facts;
- `micro-13x10-receipt.json`: conditional mechanism micro;
- `full-13x100-receipt.json`: fixed full proof and rejection evidence.

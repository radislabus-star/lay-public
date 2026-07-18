# Refactor Baseline 0.2.264

Captured on 2026-07-18 at commit `d142805`.

All later refactor checkpoints must use the same report and denominator before
claiming that a metric was preserved or improved.

## Physical Lines

| Scope | Lines |
|---|---:|
| Rust `src/` | 93,423 |
| Rust external tests | 3,034 |
| Rust total | 96,457 |
| Shell | 3,588 |
| Python | 5,238 |
| JavaScript | 2,936 |

## IME

| Metric | Baseline |
|---|---:|
| eligible candidate rate | 73.38% |
| all-input candidate rate | 63.14% |
| average candidates | 5.54 |
| latency p50 | 80 us |
| latency p90 | 2,110 us |
| latency p99 | 6,655 us |
| latency max | 10,117 us |

## Candidate Decision

| Metric | Baseline |
|---|---:|
| observed selections | 51 |
| deterministic applies | 42 |
| NANDA applies | 9 |
| total latency p50 | 44 ms |
| total latency p90 | 80 ms |
| total latency p99/max | 206 ms |

## L2/L3

| Metric | Baseline |
|---|---:|
| canonical L2 words | 17,582 |
| expected candidate present | 6/6 |
| expected candidate applied | 6/6 |
| non-final apply | 0 |
| L3 verdict | `NO_CONTEXT_CASES` |
| L3 output changed | 0/16 |

## Safety

| Invariant | Baseline |
|---|---:|
| unsafe multiword apply | 0 |
| unverified left-context mutation | 0 |
| transition replay false apply | 0 |

## Runtime Memory

| Process | RSS | PSS | Private dirty |
|---|---:|---:|---:|
| daemon | 116,600 KB | 87,556 KB | 51,008 KB |
| IME | 95,248 KB | 66,271 KB | 35,960 KB |

Quality verdict at capture time: `WATCH`.

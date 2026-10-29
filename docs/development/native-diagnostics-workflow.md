# Native diagnosis: one evidence packet before the next edit

## Scope and decision (2026-10-29)

This task improves development tooling only. It does not change the daemon,
IME, desktop adapters, delivery routes, installed services, input settings or
test cadence. The current checkout is `codex/browser-matrix-repair-20261027`
at `b835ca478198d6cb765a2246329d5646966c118c`, with existing uncommitted runtime
work preserved. Its field-integration owner and scope remain in force.

The observed development delays were repeated manual interpretation of native
receipts, mismatched browser launch environments, incomplete rolling traces,
and retries before the first failed transition had been demonstrated. Existing
`dev-check.py`, native actors, source gates and opt-in traces already supply
the required execution mechanisms. Replacing them would introduce more setup.

Chosen change: a small, read-only, standard-library analyzer for existing native
matrix receipts and sanitized paired D-Bus events. It should make unsupported
conclusions conspicuous, group failures and produce a reusable timeline.

Alternatives considered:

| Option | Fit / 10 | Decision |
| --- | ---: | --- |
| Instructions alone | 6 | Necessary, but manual receipt checks remain repetitive. |
| Existing receipts + deterministic analyzer + short instructions | 9 | Implement now; no runtime instrumentation or new dependency. |
| Perfetto timeline | 8 | Export its supported Chrome JSON format from existing timestamps. |
| rr record/replay | 4 | Useful for a bounded process defect; does not replace matching the already-running desktop, IBus and browser evidence. Defer system/setup changes. |

Primary references: [Perfetto external formats](https://perfetto.dev/docs/getting-started/other-formats),
[Perfetto UI](https://perfetto.dev/docs/visualization/perfetto-ui),
[rr build/install requirements](https://github.com/rr-debugger/rr/wiki/Building-And-Installing).

## One loop

1. Read the current owning task and identify the exact source, loaded binaries,
   browser launcher, desktop, field, action and expected visible result.
   Resolve uncertain paths with `rg --files` first. `rg` exit 1 means no match;
   exit 2 is a command/path problem and does not establish a product failure.
2. Analyze the saved receipt before rerunning input. Keep FAIL, BLOCKED and
   NOT_TESTED, and distinguish controller/setup errors from product failures.
3. Select one shared failure mechanism. Record the last established transition,
   first unestablished/failed transition, and the evidence needed to distinguish
   competing explanations. Missing evidence means UNKNOWN, not a guessed cause.
4. State one hypothesis and a falsifiable expected observation before editing.
   Reuse the existing actor and capture tools. A changed observer, launcher,
   cadence or assertion must be visible in the comparison.
5. Run the smallest discriminating case, then affected controls. Run the full
   matrix only after the short result warrants it. Preserve unsuccessful runs.
6. After two attempts that add no distinguishing evidence, change the observation
   method or involve the user according to their standing preference. Do not
   repeat the same source/runtime/inputs merely to seek a green result.

Keep this compact record in the EXISTING owning task; do not start another
parallel journal:

```text
Current hypothesis / competing explanation:
Source + loaded bytes + launcher/config identity:
Last proved transition -> first failed or UNKNOWN transition:
Smallest experiment and expected distinguishing observation:
Result + exact receipt + capture gaps:
Rejected approach / new evidence required to revisit it:
Reproduction, check and analysis durations (measured separately):
Next action:
```

## Commands and interpretation

```sh
# Full report plus a compact stdout summary. The output directory must be new.
python3 scripts/lay-diagnose.py report /private/native-run --output /private/analysis-1

# Show per-case changes and changed/missing source, binary, actor and launch identities.
python3 scripts/lay-diagnose.py report /private/after.json --compare /private/before.json --output /private/compare-1

# Sanitized metadata from the existing passive paired D-Bus recorder.
python3 scripts/lay-diagnose.py wire /private/after-events.jsonl --compare /private/before-events.jsonl --output /private/wire-1
```

`report.json` retains counts, unknown coverage, classified assertions,
case identities, timing coverage and provenance gaps. Output is mode 0600 in
a new 0700 directory. Existing results are never overwritten. The stdout view
omits individual case rows and raw error/payload text. Raw captured data stays
private and must not be committed.

The wire command also writes `trace.perfetto.json`. Open it in the existing
[Perfetto UI](https://ui.perfetto.dev/) using **Open trace file**. The exporter
uses documented complete-duration events and separate lanes for overlapping
calls. It emits no field text, text hashes, object paths or raw error strings.
No package installation or runtime instrumentation is required by this tool.

Exit 0 means the requested analysis was produced, even when Lay has FAILs.
Exit 2 means malformed evidence or an output error. Exit 3 means incomplete
wire pairing, including an incomplete comparison baseline. Counts and raw
assertions never identify the root cause by themselves. A trace-file anchor
does not prove that every causal event was captured; wire RPC timing is not
key-to-visible timing. Saved runtime snapshots are not a fresh live probe.

The current producer family is the frozen native RFB actor. A complete220-row
matrix must retain three window modes and the160+60 split. Smaller selections
remain selections; missing rows are visible as NOT_TESTED. Do not use a source
test report as a native receipt. Configuration/launcher hashes missing from an
old report remain unknown; never fill them from the currently installed setup.

## Verification

Run tooling tests only on the remote worker under the existing resource guard.
Check malformed/mismatched counts, duplicate cases, partial matrices, changed
runtime/launcher/actor, missing or incomplete traces, unpaired RPCs, clock
inconsistency and raw-text exclusion. Apply the analyzer to the saved full
Firefox result (159 PASS / 24 FAIL / 37 BLOCKED) and the paired before/after
readback traces (710 / 146 records). A tooling PASS is not a new browser PASS.

Final guarded remote verification on 2026-10-29:

- 22/22 tooling tests passed, including configuration identity and incomplete
  comparison-baseline controls. The architecture graph and canon checks passed.
- The saved Firefox 220-row report retained 159 PASS / 24 FAIL / 37 BLOCKED.
  The failing assertions group into 23 late-target results and one missing
  target. This groups symptoms; it does not establish their cause.
- Only 9/24 failed rows have intact trace-file anchors; the global trace is
  incomplete. Configuration identity and some timing samples are unrecorded.
  These gaps remain explicit instead of being filled from the current setup.
- Both 710/146-record wire captures paired completely. Each retained 12 manual
  calls; VisibleTailV4 calls changed 184 -> 48. RPC counts alone do not establish
  causal improvement or matching runtime/actor identities.
- Report analysis with self-comparison took 17.63 ms; wire comparison took
  5.18 ms. Each command took about 64 ms. These are saved-data analysis timings,
  not a measurement of total debugging speedup or key-to-visible latency.
- All 32 frozen existing integration files remained unchanged. There was no
  installation, native input, service restart or runtime-authority change.

Final receipt: `/home/ubu/.cache/lay/development/browser-matrix-repair-20261027/native-diagnostics-tooling-final-v2/result.json`.
The sibling `matrix/report.json`, `wire/report.json`, `wire/trace.perfetto.json`,
`unit.log` and `architecture-graph.log` retain the detailed evidence.
Remote original: `/home/e/projects/lay-development-runner/kde-install-control-20261020/native-diagnostics-tooling-final-v2/`.
Generated graph files were returned with hash checks. No runtime binary was
built or installed for this tooling task. The Perfetto export format and event
structure were checked; opening and rendering it in the Perfetto UI was not tested.

# 1.0.108 — isolate exact-tail settlement and report unavailable identity correctly

Preflight, before code. Baseline e178be59563fdce592921402ce9e6634e9b2e9c7, tested source708926cec7; remote installed107, stable106 until107CI passes. The user authorized seven isolated integration releases; this is explicitly a shared daemon bridge change, not a Firefox-only runtime claim.

Fresh Firefox107 baseline:130 PASS /51 FAIL /39 BLOCKED /0 NOT_TESTED of220. Fixed cumulative20 diagnostic:13/5/2/0, raw `isolated-release108-cumulative-diagnostic-v1/result.json`. Three isolated active-word controls pass and do not erase cumulative failures. Debug was scoped to lay-daemon and restored. Trace shows unknown/unbound V4 readout reported as a retention-policy change. An unavailable field is not proof that privacy changed. The delivery also misses rapid cadence; no speed fix is claimed here.

Allowed runtime: `src/bin/lay_daemon/layout_controller/ime_bridge.rs` and new `ime_bridge/settlement.rs`. Move the existing wait loop, 80ms bound and exact settled predicate unchanged behind its existing public entrypoint. Keep lease and privacy owner in the bridge. At bound V4 readout, reject missing engine/field identity explicitly before comparing the privacy bit. Both remain fail-closed; add no wait, retry, transport, engine/client special case, new owner or edit authority. Existing executor, native input, focus proof, KDE/GNOME IME and correction source stay exact.

One regression proves unavailable identity cannot be diagnosed as an observed policy flip, while a current identified field with changed permission is still rejected and a matching permission retains its former result. Old test identities/classifications and zero known-failure declarations remain. Compare moved bodies, then full canonical source/release gates, all ten release binaries, short unchanged native Firefox20 and Chrome controls. Preserve every functional FAIL/BLOCKED instead of treating mechanical isolation as their closure.

Source tests, deployment and native receipts: PENDING.

## Source verification

Semantic RED15/16 (the old V4 binder accepts an unbound nonpersonalizing readout), GREEN16/16. All previous15 semantic/RPC tests retained. Raw `isolated-release108-unit-red/result.json`, `isolated-release108-unit-green/result.json`. Both moved production function bodies compare byte-for-byte after their visibility/header; same80ms limit.

Frozen sourceSHA `b31a47181e330d17f46ff2fa95b9a83df9184d72ced4923d86b0d5fa4cf7297f`,1291files. Canonical source3551/3551 PASS (3515correctness+36package),0 known/infrastructure failures; actual full localrun-060t8122/remoterun-Qhnw3W. All original identities/classifications retained, one new daemon test.

GatesV1/V2 correctly reject invalid ADR metadata (missing schema, then explanatory text instead of canonical invariant IDs). No code/test/rule weakening. V3 applies an explicit documentation-only overlay and validates it before expensive gates; original tested request/result/source bytes remain immutable. Overlay records before/after SHA and original request SHA. Canon/graph/fmt/protected gate/source/python/public/installer/node/audit/lints/performance now PASS; MSRV/release/artifacts are pending. Native acceptance remains separate and pending.

## Artifacts and native scope

All18gates PASS,10retained release binaries,0 compiler warnings,MSRV1.88/default+lexical PASS. Exact108artifacts installed only on remoteKDE; originalIBusparent/desktop/config/models preserved. Source+metadata equal frozen b31a47181e33.

Native controls:Chrome9/9PASS including private conversion;Firefox20=12PASS6FAIL2BLOCKED0NOT_TESTED, heldkeys/configrestoration complete. Four rapid-five rows fail (three late targets, one wrong sequence), iframe-rich consecutive-two-after-Space and moved-boundary also fail. These are retained failures, not a full-window PASS or measured latency improvement. Prior debug diagnostic20=13/5/2 is not used to claim unchanged native counts. Existing strict production wait/identity/source predicate bodies remain exact;108 changes diagnostic rejection only. Raw `isolated-release108-native-v1/result.json`; engine bytes/PIDs in deployment receipt.

Current maintenance improvement is source/module isolation and truthful first-refusal diagnostics. It does not close Firefox compatibility. Complete220 per-browser evidence for107 remains labelled107; no current108full matrix is invented. Pending:immutable commitCI,stable publication and real updater --check.

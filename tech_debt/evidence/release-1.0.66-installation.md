# Lay 1.0.66 — installed runtime receipt, 2026-09-07

Status: INSTALLED_AND_LOADED / MINIMAL_ADAPTER_REFUSAL_REPAIR / PHYSICAL_CONFIRMATION_PENDING.

**Latest, Sep8 03:05:52 +03:00:** fresh remote release IME installed permanently
at `/home/ubu/.local/lib/lay/bin/lay-ibus-engine`, PID147597. Installed file and
running process both have SHA256
`12db8c00ebd23d1dbc1603dc1fbba337ddfed0c0aa308258b669df8fbc8c95dc`.
IBus4715, daemon3757261 and input-source list unchanged. Focused=true;
InputState=`passive:daemon-word-buffer`, not observer-cancelled. Remote build
58.72s, only IME, jobs20, no new tests after the user's explicit cancellation.
Full release PASS and physical-functionality PASS are NOT claimed. Old physical
and cold-start failures below remain evidence, not erased by installation.
[Current source, artifacts, backups and next action](../CONTINUE.md).

The remaining entries are historical checkpoints.

22:01 +03:00 repair checkpoint: candidate706b4d81 was built/tested only in
private remote sandboxes, not installed or started on the desktop. Lifecycle
3/3, post-exact-ready restoration5/5; immediate cold0/5 remains open. These
results do not supersede the failed physical acceptance of installed95348e4a.
[Exact candidate and scoped receipts](release-1.0.66-factory-recurrence.md#integrated-source-acceptance).

Superseding20:24 result: user rejected the temporary15728764 preview. Candidate
stopped after verified native `xkb:ru::rus`; installed95348e4a remains unchanged.
Global IBus4715/daemon272240 unchanged. Physical acceptance FAILED, not pending.

Historical20:06 +03:00: installed IME bytes remain95348e4a, but temporary candidate
15728764 is running from cache as PID2128979. Native-input safety staging used;
global IBus/keyboard daemon unchanged.425/425 focused and post-ready client5/5
do not close the retained immediate cold failure or pending physical acceptance.
[Current preview receipt](release-1.0.66-factory-recurrence.md).

Superseding17:17 +03:00 result: the exact installed new IME again has a
cancelled observer, this time after `CreateEngine` sees a revoked transfer.
[Confirmed recurrence, not a stale service](release-1.0.66-factory-recurrence.md).
The following installation/readback remains historical, not physical PASS.

At 17:03 +03:00 an IME-only repair superseded the original IME image below:
PID1148501, SHA256 `95348e4a81056a9a476c9aade72194978a4c359abe81ac6983a381989423a668`.
Live bridge `passive:no-focus`; global IBus and other Lay PIDs unchanged.
[Exact gates, failed first transaction, rollback and successful delivery](release-1.0.66-live-observer-incident.md).
Physical user confirmation is pending; no blanket functionality PASS.

Post-install report and live diagnosis at 16:10 +03:00 confirmed a stopped context
observer and refused input authority. [Current incident and repair scope](release-1.0.66-live-observer-incident.md).
The successful installation transaction below proves deployment, not working
autocorrection, suggestions or physical Double Shift. Physical acceptance remains open.

Release commit: `c87cc3124766f85245ca3962d2c054957b7c4fef`.
Published to `origin/codex/tech-debt-20260831`; post-push `git ls-remote`
returned that exact commit. Installed CLI readback: `lay 1.0.66`.
This documentation-only publication receipt does not change installed artifacts.

The user requested installation now after disclosure of the outstanding
physical-keyboard gate. This release-specific sequencing decision does not
turn unexecuted physical checks into PASS. TD121/TD125 remain open for that
acceptance; general Wave quality and mixed-token correction are not promoted.

## Admitted source and artifacts

- Source fingerprint: `2fe001cd88e9631a8d13bc2f5cfe78db2603690fa1b0726bdedf70f924b933a5`.
- Unchanged installer SHA256: `d8a8e8b82fc9ec94f7b985e310f5831684461f536a53c0c68ea4d1e40726129a`.
- Remote changed/full gates: each2646/2646,0semantic/0infrastructure failures.
- Both lint scopes and architecture gate: PASS. Runtime repair review9/10,
  lint-only review10/10, High0/Medium0 in each bounded review.
- Exact optimized release IME: V2 post-exact-ready client5/5, clean process
  cleanup. Receipt SHA256 `80978ba5382d14a29a896eaff0b5d069aa5a17eeadb0815a84e3f08a6a26a9dc`.
- Fourteen Cargo binaries plus remotely compiled sidecar and its receipt:
 16/16 transfer hashes matched, then revalidated before installation.
- Generated receipt mode normalized0664→0644 as required by the unchanged
  controller. Its content/hash did not change.

## Executed transaction

```sh
scripts/install-live-release-1.0.66.sh --snapshot /home/ubu/.local/state/lay/release-backups/1.0.66-preinstall-b4bd39
```

Exit0, `FORWARD_INSTALL_1_0_66=PASS`.
Log: `/home/ubu/.cache/lay/development/release-1066-final-0pcIZl/install-live.log`.
SHA256: `f960c11d1466b03c251ab18fb36c5ed53e2a1fd95760a3ceeb52e9815f2ac8f7`.
Initial L1.1 health-not-ready/refused-connect messages precede the successful
bounded startup; the final process/package/health checks passed. No rollback
was needed. No local build, test suite, or training was run.

Installed/source/loaded extension versions all1.0.66. Ten canonical runtime/
tool executables updated in the existing19-file installed tree; other existing
auxiliary files were preserved. Snapshot retains bin19/extension9/L2seven
files and the exact installed L3 unit, with modes.

## Independent readback

The read-only reviewer reported NO_MISMATCH: service states active, one managed
IME, engine `lay-ime-ru`, and the following process=installed=prepared hashes:

| Process | PID | SHA256 |
|---|---:|---|
| daemon | 272240 | `80e6bf1fd21da20df89ce69f1d955e4c7240bf50c03bd259fa56d32a62d92a74` |
| L3 | 272161 | `99d1f51a8e7648cf2161b3732aa2120db7e0efb93bce11831ce07dc1d79447b6` |
| IME | 272498 | `dfeb50e88c8a9170137563ddaf09ae3a44eb8a5441899e3d399baf60fd8e5185` |
| L1.1 | 271400 | `1cedb27c59afe0fd1437f37a0451194eb80b046449b10053a73016b64f864b4b` |

Global `ibus-daemon` PID4715 stayed unchanged. No global restart, input-source
migration or unrelated process termination occurred. This is loaded-image
verification, not physical GTK/keyboard, cold-latency or mixed-word quality proof.

# Issue 41: Complete Uninstall Ownership

Date: 2026-08-22
Status: implemented and sandbox-proved; public publication pending
Issue: https://github.com/radislabus-star/lay-public/issues/41

## Observed Defect

The release installer owns two linked locations:

```text
~/.local/lib/lay/bin/<release binary>   managed executable
~/.local/bin/<release binary>           user-facing symlink
```

The old uninstaller removed an explicit, stale subset of the user-facing
names. It did not remove the managed directory. Consequently
`lay-nanda-wave-train`, `lay-l1.1-restore`, `lay-l1.1-serve`, and every managed
binary remained in `~/.local/lib/lay/bin` after `--purge`.

This is an ownership-enumeration defect, not three independent missing names.

## Selected Contract

The uninstaller derives release-link ownership from the symlink target:

```text
link in ~/.local/bin
  -> canonical target is inside the configured Lay managed bin directory
  -> remove link
```

It then removes the managed bin directory. This covers current and future
release binaries without deleting unrelated commands merely because their name
starts with `lay-`.

Removal levels remain distinct:

```text
normal uninstall
  remove active managed bin and links
  preserve settings, models, state, rollback and source

--purge
  perform normal uninstall
  remove ~/.config/lay, ~/.local/share/lay, ~/.local/state/lay
  remove the complete ~/.local/lib/lay tree, including rollback and staging
  remove a clean official source checkout unless --keep-source is set
```

## Consequence Analysis

- Unrelated regular files and symlinks in `~/.local/bin` must survive.
- A symlink is removed only when its canonical target is under the exact
  managed runtime directory.
- A custom `LAY_INSTALL_LIBEXEC_DIR` is honored by both install and uninstall.
- Normal uninstall preserves rollback material; purge intentionally removes it.
- The operation is idempotent when files or directories are already absent.
- The regression proof runs only in a temporary HOME and SYSTEM_ROOT. It must
  never run the uninstaller against the active desktop installation.
- Existing service, GNOME, IBus, udev, settings and clean-source cleanup remains
  unchanged.

## Promotion Gate

The fix may be published only when the sandbox proves all of the following:

1. all ten current release links are absent;
2. the managed bin directory is absent after normal uninstall;
3. rollback survives normal uninstall;
4. unrelated symlinks survive;
5. the complete managed tree is absent after purge;
6. the existing systemd, IBus, udev, settings and source-preservation checks
   still pass;
7. `bash -n`, ShellCheck and the public issue regression suite pass.

Runtime authority is not changed by this source-only uninstall fix.

## Result

Measured on the fixed sandbox route:

```text
current managed release links removed                 10/10 PASS
unknown future managed release link removed             1/1 PASS
custom managed libexec route removed                     1/1 PASS
unrelated user symlink preserved                         1/1 PASS
rollback preserved by normal uninstall                   1/1 PASS
normal uninstall idempotent                              2/2 PASS
complete managed tree removed by --purge                 1/1 PASS
existing systemd/IBus/udev/settings cleanup              PASS
Bazzite and dirty-checkout regressions                    PASS
bash -n / ShellCheck                                     PASS
```

Not tested: destructive execution against the active local Lay installation,
the reporter's Orange Pi 5 host, and an ARM64-specific filesystem layout.

Receipt:
`docs/structural_gates/receipts/LAY_PUBLIC_ISSUE_41_COMPLETE_UNINSTALL_2026-08-22/final-sandbox-proof-v1.json`.

Verdict scope: **ISSUE_41_SYSTEMIC_UNINSTALL_FIX_SANDBOX_PASS**. Runtime
authority remains unchanged.

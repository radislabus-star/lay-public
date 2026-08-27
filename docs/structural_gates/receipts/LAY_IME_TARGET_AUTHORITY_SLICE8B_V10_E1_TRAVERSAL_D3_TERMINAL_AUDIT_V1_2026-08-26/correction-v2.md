# D3 bootstrap execution-admission correction V2

Date: 2026-08-26

## Scope

This correction preserves the immutable D3 bootstrap and bootstrap-audit
receipts. It withdraws only the positive execution admission published by the
bootstrap audit.

```text
historical bootstrap receipt     D3_NAMESPACE_CREATED_UNAUDITED
historical bootstrap audit       D3_BOOTSTRAP_AUDIT_PASS_EXECUTION_ADMITTED
effective bootstrap admission    INVALID
effective D3 verdict             BLOCKED_PROVENANCE
```

The D3 paper, preflight, D2 ELF, D2 bucket map, D2 terminal history and all
existing receipts remain byte-immutable.

## Observed defect

The authoritative D3 parent was published by the root remote controller with
mode `0700`:

```text
/home/e/.local/share/lay/provenance/
  slice8b-v10-e1-traversal-d3-estimator-recovery-v1-20260826
```

The route stage itself was `0755` and the subject output directory was owned by
user `e`, but user `e` could not traverse the root-owned `0700` D3 parent. The
same boundary first prevented the local `scp` evidence mirror. The bootstrap
audit recorded that transport defect and the `0700` parent but incorrectly
still admitted U2 execution.

The one admitted `U2-SINGLE` subject then exited `101`. Its sealed stderr is:

```text
write D1 subject receipt: ".../subject/SUBJECT_RECEIPT.json:
Permission denied (os error 13)"
```

No `SUBJECT_RECEIPT.json`, component samples or structure evidence was
produced. The U2 receipt correctly selected:

```text
selected cause     provenance
selected rank      0
verdict            BLOCKED_PROVENANCE
reason             subject evidence unavailable
```

This is a controller provenance failure. It is not a semantic failure of the
D2 ELF and does not test task-clock capability or the D3 estimator.

## Immutable evidence

```text
D3 bootstrap receipt SHA-256       a7b921799751f38f745a2945ff8b7222428ff16c54e42c35e0e0d99019468529
D3 bootstrap audit receipt SHA-256  0c5a34b1809dbbfa8b3744b65d86f3dfa1d0c1bfb00a623a0bbe5089669b7bb1
U2 route receipt SHA-256             7c74a689079b8c40442c8065ce73a5deb69d990daf3a6404b43461808744888e
U2 observation SHA-256               a85d128fed922a0317206ff7d716c59ea6e6e876320453a37af4430996ea74c9
U2 stderr SHA-256                    42929422fb0590c9083660e3558ec3643cde54dcf13b2bb485e0ac142934a964
U2 remote SHA256SUMS                 76469bd56d932f27cfc10610c8829f4864d74450d17f95f5cc9fb55699ec071d
U2 manifest entries                  13
```

## Terminal state

```text
u2-single.available              absent
u2-single.consumed-before-exec   present
U2 state                         BLOCKED_PROVENANCE
t2-single.available              present, retired unconsumed
T2 state/result/perf.data        absent

U2 subject executions            1
T2 subject executions            0
perf record/stat                 0 / 0
PMU events                       0
runtime authority changed        false
retry permitted                  false
```

No permission change, marker recreation, U2 retry, T2 execution, new ELF, new
bucket map, optimization claim, runtime integration or deployment is admitted.
Any future estimator measurement requires a new paper namespace and must prove
the full parent-directory traversal/ownership chain before creating one-shot
route markers.

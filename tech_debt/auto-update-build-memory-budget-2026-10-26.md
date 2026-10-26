# Automatic update build: premature memory reclaim

PriorityP0; source repairGREEN, actual update completionPENDING.
Measured first failure: a source build exceeds the unit's3GiB MemoryHigh
while staying under its4GiB MemoryMax; millions of high events and28% full
memory pressure stall the compiler and keep update.lock busy for over an hour.
Owning document: docs/architecture/stable-auto-update-2026-10-19.md,
Build memory budget2026-10-26.

Minimal repair removes the soft threshold while retaining hard4GiB/swap256MiB,
1CPU and one Cargo/Rayon job. A runtime-only adjustment to the same existing
hard budget on the exact active unit restored compilation progress without
restarting the updater or manually installing/restarting Lay. Alternatives:
keep3GiB and raise timeout2/10 (does not remove reclaim); raise hard memory5/10
(needs larger host budget); remove early throttle within existing hard cap9/10.
Do not bypass human-safe admission or throttle unrelated desktop apps.

Regression proves the resource contract (new testGREEN, auto-update9/9PASS),
systemd verify0. Full99 source/build gate started before this unit/test-only
addition: preserve its exact source and compose final policy-specific checks
and artifact parity explicitly. Do not pretend earlier evidence covered new
files or native behavior. The new unit becomes effective on installation;
other old updater instances may need an independently verified bounded repair.
Finish by checking actual installed updater release discovery after publication,
record any timeout/build failure honestly, and retain the automatic install
receipt separately from source and native acceptance.

2026-10-26 measured follow-up: actual all10-binary warm-dependency rebuild
PASS in509.595s, peak3,803,045,888bytes under4GiB, zero high/max/OOM events;
all executable hashes equal final-source B. Earlier cached run rejected.
Raw receipt and limits are in release-1.0.99-verification.json. Final CI and
release discovery remain pending; successful automatic installation is not
inferred from this compiler proof. Remote ordinary builds use dedicated20CPU/
28GiB per the user's subsequent authorization, separately from this diagnostic.

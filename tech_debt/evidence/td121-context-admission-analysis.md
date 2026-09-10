# TD-121: canonical context admission and bounded IBus ingress design

Date: 2026-09-05. Source baseline: `cc1e2207519801ca0f9b7c6963897b55953a7751`.
Scope: Astra/XHigh design analysis; no production edit, Cargo, new probe,
installation, service/config mutation, or global IBus restart.
`runtime_authority_changed=false`.

## Decision

Select **one canonical context contract, a bounded metadata observer on the
existing IBus connection, and a one-shot signal-queue fence**. Native
`FocusInId` and the necessary cached-false compatibility property acquisition
feed that same contract. Neither path equality nor callback-local generation
alone grants transfer. This is the smallest identified route which can restore
an observed full token on the already-running cached-`FocusId=false` IBus while
rejecting an invisible `A(Lay) -> B(foreign IME) -> A` excursion.

This is a concrete implementation design, **not a passed transport proof**.
The baseline reproduction gate is now closed; the observer/fence implementation
must pass the transport and resource gates below before its transfer authority
is promoted. A safety-only first-token quarantine is necessary failure behavior,
but does not complete TD-121 or replace the required positive restoration.

Do not implement either tempting shortcut:

- `Get(CurrentInputContext)` plus equality/local `focus_serial` is insufficient.
- Even a receive-order observer plus `Ping`/property reply is insufficient for
  `GlobalEngineChanged`: IBus queues that signal separately and its delivery
  can lag behind the reply. The signal-queue fence below is essential.

## Evidence and limits

The requested TD-121, layout audit, spec pass 2 and current AGENTS were read.
Graphify query `focus handoff identity engine` identified the engine, frame,
tail-memory, atomic and exact-replay paths; conclusions below also inspect code.

### Installed-byte reproduction, independently read

Receipt:
`/home/ubu/.cache/lay/layout-phase2-private-31hE0X/receipt.json`.
SHA-256: `d75822dd10425f422b5f7a2d5b942eeb79e8f742c913446936bd740483acd82c`.
Installed engine SHA-256:
`391b3b44025461a71d6867dc92584dfa1906c1f8ae72ea610a13d2265aaacfab`.

`probe_status=COMPLETED`, six cases executed. Five concern handoff and one
TD-120 suppression. Source `l` was observed explicitly before transition.
US->RU, US->US and same native context/new path lose it internally in
3,148–3,463 microseconds; same-path control retains it; different-context
control clears it. This rules out expiry as an adequate explanation for these
three new-object cases. It does not establish historical physical delivery,
actual desktop client text or correction quality (`NOT_TESTED` in receipt).

The US->RU scripted key stream is `l`, `о`, `м`, while US->US is `l`, `j`, `v`.
Consequently `lом` and `ljv` remain separate full-input proof cases. This probe
does not demonstrate a new Space correction of `jv` or a full `ljv -> дом` fix.

### Repository facts

- `ibus_interface.rs:51–98`: ordinary focus binding resets through path logic;
  native focus calls that ordinary handler; `FocusOutId` ignores its argument.
  `:302–305` advertises `FocusId=false`.
- `engine.rs:69–156`: a native receipt is a local concatenated path/client
  string, and engine-path handoff separately relies on a preserve timer. A new
  canonical receipt can therefore be replaced with inherited shared identity.
- `state.rs:253–285`: constructor copies shared tail before actual focus.
  `:291–340`: source focus preservation uses 700 ms; this differs from target
  acceptance. `tail_memory.rs:580–598` publishes shared copies and exact leases.
- `server.rs:11–35`: all IBus engine objects use one IBus connection; bridge
  consumers arrive on a separate session connection. `factory.rs:18–47`
  creates unique engine paths but captures no causal context ticket.
- `protocol/state.rs:39–54`: no connection-wide receive-order context guard.
  `ClientContextState` generations are per object, not a global focus log.
- `atomic.rs:183–205`: settlement replaces the whole shared snapshot. A new
  revocation generation must not live only inside that copied snapshot.
- `layout_sync.rs:53–117,135–144`: blocking layout calls exist. Setting
  `#[interface(spawn = false)]` globally can obstruct the `CreateEngine` work
  needed by an in-progress switch; it is not an admitted small ordering fix.
- `bridge_actions.rs:43–88,273–306`: reads/controls select shared active path,
  then acquire an engine independently. They need the same new admission check.
  Existing `ImeManualToggleOutcome::NotHandled` is a safe typed refusal:
  `manual_trigger_runtime/ime.rs:56–59` maps it to `Complete(None)`.

### External protocol facts, pinned source inspected

Upstream tag `1.5.34-rc2`, corresponding to the installed base version:

- [engineproxy.c](https://github.com/ibus/ibus/blob/1.5.34-rc2/bus/engineproxy.c):
  `:811–822,1389–1431` caches FocusId by name; `:1463–1473` can resend native
  focus after delayed capability discovery; `:1513–1607` emits asynchronous
  focus/enable/disable calls without awaiting their execution. `:915–918`
  obtains engine proxy connections from the factory connection.
- [inputcontext.c](https://github.com/ibus/ibus/blob/1.5.34-rc2/bus/inputcontext.c):
  `:1913–1948,2021–2054` handles field focus; `:2982–2999,3041–3117,3149–3180`
  shows a successful engine replacement: old `FocusOut -> Disable`, new
  `FocusIn -> Enable`, performed within `new_engine_cb`. `:3255–3270` cancels
  an earlier pending request. Keys on fake contexts are not sent to the engine
  (`:1142–1144`).
- [ibusimpl.c](https://github.com/ibus/ibus/blob/1.5.34-rc2/bus/ibusimpl.c):
  `:1281–1305` returns the *current* focused object path, with no generation.
  `:885–984` moves a global engine across focused contexts, including the fake
  no-focus context. `:661` sets global-engine mode true; the inspected file
  contains no second assignment. `GetUseGlobalEngine` exposes the value
  (`:1638–1665`), although that method is deprecated.
  `:1087–1107,2505–2509` emits `GlobalEngineChanged(name)` for a changed global
  engine name. `CurrentInputContext` does **not** emit PropertiesChanged in
  this source: the property helper is called for `PreloadEngines` and
  `EmbedPreeditText`, not focus.
- [dbusimpl.c](https://github.com/ibus/ibus/blob/1.5.34-rc2/bus/dbusimpl.c):
  `:1550–1653` assigns the real sender and routes incoming no-destination
  signals through `bus_dbus_impl_dispatch_message_by_rule(..., NULL)`.
  `:1968–2085` appends those signals to one locked FIFO `dispatch_queue` and
  delivers them in an idle callback. `ibusimpl.c:2478–2495` uses that same
  queue for `GlobalEngineChanged`. A normal method reply does not drain it.

These are source facts, not a verification of all Debian patches or of the
new observer against live IBus. Exact deployed-source compatibility and the
signal-marker ordering require the private transport proof below.

zbus 5.15.0 source in the local Cargo registry was also inspected:
`message_stream.rs:18–67,190–218`, `message/mod.rs:28–38,226–234`,
`connection/mod.rs:311–341,1025–1031`, `object_server/mod.rs:385–423`,
`connection/socket_reader.rs:47–74`, `zbus_macros/src/lib.rs:231–247,312–322`.
`Message::recv_position()` orders messages from one connection only. Header
serials are not an ordering clock; IBus signals even explicitly use serial 1.
The macro exposes `#[zbus(header)]`, not a direct Message/receive-position
argument. `spawn=true` is the default and permits handler reordering.

## Alternatives, scored as design judgments

| Route | Score | Decision |
|---|---:|---|
| Native receipt + bounded property adapter + receive-order metadata + signal-queue marker | **8/10** | Selected full solution; genuine new admission mechanism, bounded and separate from text/model ownership |
| Boundary quarantine immediately, native identity enrichment, defer all ordinary transfer | **9/10 safety, 5/10 task coverage** | Viable narrow mitigation; cannot satisfy current-runtime positive restoration alone |
| Native receipt only, enable FocusId | **8/10 cold architecture, 3/10 hot-upgrade coverage** | Cannot repair cached false without the forbidden restart/new names; native payload alone also does not solve stale callback application |
| One/two context reads + current per-engine generation or TTL | **2/10** | Observationally indistinguishable ABA; does not establish focus continuity |
| Serialize the entire ObjectServer or migrate all text into one new actor | **4/10 here** | Broader concurrency/latency/reentrancy migration; blocking switch dependency remains; unnecessary for a metadata admission gate |

The selected route adds a single metadata source of truth and a private
fence signal. It does not add a text owner, independent decoder, correction
producer, foreign-key monitor, physical replay route or polling loop.

## Exact minimum implementation contract

### 1. Connection-local metadata, outside speculative SharedState

Create one `Arc<ContextAdmission>` alongside Shared in `server.rs`. It is
shared by factory, engines and bridge, but never deep-cloned/rewound by atomic
preview or settlement. It holds only bounded metadata and monotonic revocation.

Proposed types, with names illustrative and semantics mandatory:

```text
ContextKey = (bus_connection_generation, canonical_context_path)
FocusActivation = (ContextKey, activation_generation)
EngineOwner = (engine_path, owner_generation)
WordLineage = (lineage_generation, KnownStart | UnknownStart)
IngressStamp = (connection_generation, recv_position, sender, message_serial)
FactoryTicket = (target_path, source_owner, source_activation,
                 factory_recv_position, expected_target_profile, nonce)
TailTransfer = (ticket, source_word_lineage, source_tail_epoch,
                source_seal_stamp, target_focus_stamp, consumed)
ContextReceipt = (ContextKey, native_or_property_origin,
                  request_generation, context_reply_position?, barrier_position)
```

Native `client` is useful provenance but not available from the property; do
not compare `path + client` against property `path` or invent an empty client.
Keep existing engine-path fallback labels separate from canonical identity.
An engine constructor starts with no transfer authority; copying text into its
working tail before a ticket is consumed is removed.

### 2. Subscribe before publishing the factory; observe only scoped metadata

Use the existing `ibus_connection`, not a second connection. Register narrow
`MessageStream::for_match_rule` subscriptions for Lay's lifecycle method
members, Factory CreateEngine, ordinary ProcessKeyEvent, Reset, relevant owner
loss, `org.freedesktop.IBus.GlobalEngineChanged`, and the private fence signal.
No eavesdrop, BecomeMonitor, client InputContext key subscription or foreign
text capture. Sender validation must match pinned IBus behavior: engine calls
may be stamped `org.freedesktop.DBus`; do not assume all have IBus's signal
sender. Private markers accept only this connection's actual unique sender,
the exact private path/interface and a pending nonce.

Merge streams by `ordered_stream::OrderedStream`/`Join` using `Sequence`.
`select_all`/callback scheduling order is not a valid merge. Direct dependencies
on the already-transitive stream utilities may be needed; do not invent a
numeric conversion from zbus's opaque Sequence.

The observer continuously drains and records metadata. It must not await a
context RPC, an engine lock, Shared's lock, layout execution or model work.
Acquisition futures publish bounded completion data separately. No text/log
payload is kept in the observer. Unknown/malformed lifecycle closes admission.

Callbacks use `#[zbus(header)]` to identify `(sender, serial)` in a bounded
metadata ring and obtain their actual Sequence. Headers correlate; they never
sort. Record receive and settlement separately. An out-of-order callback cannot
overwrite a newer owner. Missing/evicted/unobserved stamps revoke ordinary
transfer and completeness; they do not wait indefinitely or replay keys.

### 3. One-shot acquisition and the indispensable signal-queue fence

For a legacy FocusIn, begin one bounded
`org.freedesktop.DBus.Properties.Get("org.freedesktop.IBus",
"CurrentInputContext")` using `Connection::call_method`, retaining the raw
reply Message/Sequence. One current request; a later lifecycle event revokes
the nonce. No retry and no per-key RPC. A delayed native ID enriches the same
current activation only if no intervening revocation occurred; it supersedes
the compatibility result and cannot rearm an invalid token.

After the context reply, emit one no-destination private signal
`ContextAdmissionBarrier(nonce)` on the same IBus connection. Its registered
match sends it back to that connection through IBus's FIFO dispatch queue.
Wait for the observer to process that marker, not merely for emit completion.
Every GlobalEngineChanged enqueued before the property snapshot must precede
the marker. Drain all scoped streams through the marker's receive position
before sealing the receipt. Validate both request and lifecycle generations.

For a native-ID transfer, the context value comes from the native call, but
the same one-shot marker is still required to fence pending global-engine
signals. No property is required just to reproduce the native payload.

Bootstrap must establish global-engine mode for this connection epoch and
initial global engine state after subscriptions are active; read the existing
GlobalEngine property, and validate global-engine mode using the exposed
read-only method/source capability. In this pinned upstream mode is constant;
unsupported/false mode stays Unknown. Do not assume an arbitrary IBus version
has the same mode/lifecycle invariant. A disconnected stream or lost bus owner
revokes the epoch and all tickets.

Deadline proposal: one **5 ms total compatibility acquisition budget**, covering
Get plus marker, measured separately from Space. This is an admission target,
not an observed timing. Timeout means Unknown; the first key is never queued
waiting for that budget. If the measured success/latency tradeoff fails, report
that gate rather than lengthening the Space deadline.

### 4. Positive transfer and the invisible-ABA exclusion

Create a ticket only when the target factory object is requested while a known
source owner/activation is active. The factory request must precede source
deactivation. Track the source until its last input callback has settled;
take the latest tail at source seal, not the older constructor copy.

Accept exactly this focus-significant envelope:

```text
Known source A/E0 -> CreateEngine E1 ticket
-> FocusOut(E0) -> Disable(E0) -> FocusIn(E1)
-> same canonical A + successful one-shot barrier -> consume ticket once
```

Normal capability/geometry callbacks may appear between these events; they
cannot manufacture identity. Any additional focus activation/out, reset,
unresolved earlier key, unmatched/competing factory, foreign global engine,
source/target owner change, expiry, bus change or wrong native ID destroys
that ticket. A source seal is valid only after all relevant key observations
preceding FocusOut have corresponding settled outcomes and text revision.
If target FocusIn executes before source FocusOut, keep a pending metadata
claim; do not erase source/new state. A subsequent key may accept a completed
claim, but if it arrived before admission it poisons completeness for that
word and continues literally once.

For same-object re-enable, use a ticket whose source and target engine paths
are equal, opened at that owner's FocusOut and completed only by its matching
Disable/FocusIn envelope, the same canonical A and the same marker fence. No
Factory event is required for this reflexive ownership transition. An
intermediate fake/B focus or foreign-engine epoch still destroys it. This
preserves the private same-path quick-control shape without treating every
same engine path or plain repeated FocusIn as proof of uninterrupted input.
An outstanding atomic proposal prevents source sealing until its existing
submission/settlement contract resolves; the observer does not copy that
proposal into a layout handoff.

The transfer preserves observed tail and word lineage only. It generates a
new target owner, focus activation/frame generation and invalidates candidate
frames, prepared correction, Tab acceptance, pending unsafe feedback and old
decoder/layout result authority. A word-scoped ordinary suppression may move
only with the same admitted lineage, once. Old-source cleanup compares its
owner/ticket before touching shared state.

Why foreign ABA is rejected: an excursion to a foreign *global* engine emits
GlobalEngineChanged(foreign). Receipt sealing waits for the FIFO marker, so a
signal queued before the context snapshot cannot hide behind an early reply.
That foreign event irrevocably increments revocation; returning to a Lay name
does not reverse it. While the global engine remains Lay, field changes are
reported to that attached engine, including fake-context transitions, and
invalidate the candidate envelope. The positive case changes only E0->E1 in
A; the optional Lay->Lay GlobalEngineChanged must agree with the ticket's
expected profile. Same-name engine recreation requires no fabricated signal.

This is explicitly restricted to verified global-engine mode. In per-context
engine mode, an unfocused A can complete a pending engine creation while B's
engine is unrelated; the same apparent local envelope is insufficient. Deny
that mode, do not paper over it with a timer. The private transport suite must
prove foreign ABA with the signal intentionally held until after Get replies.

### 5. Unknown prefix remains a real authority state

`UnknownStart` is sticky through timeout, empty local tail, Backspace-to-empty,
soft Reset, engine recreation, repeated native receipt and package reload.
It is not TD-120's suppression boolean and cannot be copied to another owner
as an ordinary guard. Printables are delivered once through their existing
ManagedCommit/TerminalPassthrough route; no inferred prefix or physical replay.

At the first actual word boundary, finish that word without correction,
completion acceptance, whole-word projection or word learning. Only then arm
KnownStart for the next word. Capture the *previous* word's completeness in its
frame before boundary code resets state; `push_tail_char(' ')` must not bless
the just-closed partial token. Backspacing across the observed boundary revokes
completeness again. Navigation, selection replacement, paste/external edits,
field change and input-route gaps also require a new boundary or a separately
proven complete surrounding-text observation. Cursor 0 in a truncated snapshot
does not prove beginning of document; RequireSurroundingText is not a causal
acknowledgment by itself. Surrounding rehydration is not part of this patch.

ManualToggleV3 on Unknown returns existing `NotHandled` before any ordinary
text edit/delegation; it must not fall into DaemonWordBuffer. Exact undo/replay
keeps its independently proven edit receipt and current lease checks; a stale
pre-handoff undo is not exempt merely because it is called "exact".

### 6. Cross-bridge, atomic and exact transport consumers

Session-bus and IBus receive positions are incomparable. Before an
authority-bearing bridge snapshot/control, use a bounded IBus Ping plus the
same private marker fence, then acquire the selected engine and revalidate
the captured admission token. The existing IBus Ping is variant echo; use its
actual signature, not an assumed empty method. A control deadline failure
returns typed refusal/false without mutation. Plain status reads may report
pending/Unknown; they must not advertise a replay-capable tail from it.

Revalidate owner/revocation again at edit/lease settlement; no lock is held
across any external call. A bridge V1/V2 suppression/cancel action must not
erase a newer ordinary owner. TD-120 retains its separate compatibility
requirements. The exact SurroundingText path still requires grab, exact
VisibleTailV2 capture, both lease checks, one ActivateLayout, both layout
readbacks, late suppression and one replay. The new admission token augments
these proofs. No new physical action or relaxed exact capture is authorized.

Atomic preview records the admission token but cannot change the live
observer. Before commit/native submission settlement, compare live connection,
revocation, owner and shared mutation revision. A stale speculative clone
cannot restore old admission/completeness/guard or overwrite a new owner.
This requires coordination with TD-120 settlement; deep-clone isolation alone
does not establish it. Compare/reject before copying or emitting output.

Layout worker requests must carry context/owner/request generation and reject
stale pending work before external activation. An already-dispatched activation
cannot be retroactively cancelled; its completion does not mint a new decoder
or text lease. Mark changed authority Unknown on a conflicting completion and
perform no compensating second switch. Existing GNOME single-action ownership
remains mandatory.

## Consequences and resource bounds

| Dimension | Expected effect, possible regression, invariant and gate |
|---|---|
| Candidate/lattice retention | Admitted transfer restores the full observed surface; no word-specific candidates, no new narrowing. Unknown frames have no complete-word apply authority. Show full observed input and all existing per-class proof metrics separately |
| Ranking/false authority | Ranking/verifier policy unchanged. Old candidate frames cannot survive owner change. Count mistaken complete-word admissions separately from correction-quality errors |
| Latency/deadlines | Zero per-key RPC, no queued physical keys. One compatibility Get + one signal marker per lifecycle, or native marker. A small local stamp lookup/settlement acknowledgment is added per key. Fail before the existing Space deadline; measure p50/p95/p99 plus failures |
| CPU/RSS/allocations | One continuously polled async observer, no OS thread/model worker; one pending acquisition/transfer. Proposed metadata ring 128 entries, at most source/target owners, bounded identifier copies (e.g. 1 KiB each). Prefer small member-filtered streams over an unfiltered stream retaining arbitrary SurroundingText bodies |
| Backpressure | zbus MessageStream uses bounded async broadcast; a full undrained subscriber can stop its socket reader. Thus observer must never await RPC, model, engine or Shared locks. Stream termination drops subscriptions and revokes admission. Test sustained bursts, observer cancellation, saturated ring and slow getter; do not describe silent queue growth or arbitrary scheduler starvation as solved |
| Cache/identity | IBus FocusId cache is respected, not reset. Canonical receipt records its source, connection and nonce. Callback ring is correlation metadata, not cached text authority; eviction fails closed. No same-path/TTL resurrection |
| Package/delta/config reload | Canonical identity is package-independent; existing material/config/frame generations still reject stale work. Unknown cannot turn Known on reload. Future packages do not gain access to admission bypasses |
| Learning/feedback | No positive, negative, censored or typed-word feedback from an incomplete token. Source accepted-word feedback transfers only with its actual edit receipt; no duplicate feedback from old callbacks/atomic settlement |
| Concurrency | Receive order, handler settlement and effect order are distinct. Local short mutex/atomic revocation only; never admission-lock -> await engine/shared. Do not globally serialize ObjectServer. Old source cannot clear or publish into the new owner |
| Failure/rollback | Timeout/missing marker/bus loss/unsupported mode means Unknown and no text replacement. No retry, replay or guessed prefix. Runtime rollback remains a Lay-binary rollback; no IBus cache/settings migration or dictionary deletion |
| IME/daemon consumers | ManagedCommit width11, narrow terminal width2, native/cached capability, exact terminal single-commit, GTK exact SurroundingText replay, legacy/atomic and bridge methods remain separate proofs. Unknown does not escape into daemon fallback |
| Maintenance/removal | One private metadata helper and typed tickets; remove compatibility Get only after cached-false runtime support is explicitly retired. Native still needs generation/order protection. A later true global focus-generation protocol could replace this helper; no full text-owner migration is needed |

Queue capacities are implementation bounds, not measured resource proof.
Concrete starting proposal: 8 messages per narrow lifecycle/signal stream, 64
for ordinary key stamps, metadata ring 128; report actual total allocated
capacity after subscriptions. Bounds must include retained serialized message
references, not just copied metadata. Do not read arbitrary text/property
payloads into the observer to simplify filtering. Max queue count alone does
not establish a byte/RSS bound. Future bus/protocol changes require rechecking
the FIFO/mode contract, not merely recompiling.

Minimum production surface: new `context_admission.rs` (typed metadata/FSM,
observer/acquisition adapter), server wiring, factory ticket creation, focus
callbacks, tail publish/accept/clear, one completeness field and frame binding,
boundary/preedit/Space/manual guards, bridge admission and atomic settlement
checks. Layout request stamp repair is the existing worker's metadata change.
This is a cross-cutting protocol admission change, larger than a two-line
FocusId fix; it is not a replacement runtime or a new shared text actor.

## Exact proof manifest before production promotion

Create explicit cases below before implementation; table rows are future
cases, not PASS claims. Avoid sleeps: injected clock, stalled callback/reply
barriers and controlled transport scheduling. Current results for these new
implementation cases: **0 executed**.

| ID | Required scenario and exact assertion |
|---|---|
| C01 | Cached false, complete `l`, A/E0->A/E1 US->US: admitted tail `l`; after j/v full frame `ljv`, no duplicate prefix commit |
| C02 | Cached false US->RU: tail `l` retained; physical j/v produce full observed `lом`, not suffix-only frame; test correction as a separate mixed-input case |
| C03 | Cached false RU->US: symmetric exact observed-prefix retention |
| C04 | Native true same A/new path: identical transfer and fresh frame/owner; zero compatibility Get |
| C05 | Initial legacy FocusIn followed by delayed FocusInId: enrich once, cancel old Get; no clearing or completeness upgrade |
| C06 | Same path rapid FocusOut/Disable/FocusIn in A: reflexive ticket retains complete `l`; inserting fake/B activation destroys it; existing exact lease tested separately |
| C07 | Different canonical fields, same app/window/text: zero transfer, guard or feedback |
| C08 | A->B->A while Lay stays global: first B activation irreversibly revokes A's ticket |
| C09 | A(Lay)->B(foreign)->A with foreign lifecycle invisible to Lay: delayed foreign GlobalEngineChanged must precede self-marker; zero resurrection |
| C10 | Deliver Get(A) before queued foreign signal: Get-only design would fail; selected design still waits and rejects |
| C11 | Factory target created before switch-away, then cancelled by foreign engine: returning A cannot consume that ticket |
| C12 | Global-engine=false/unverified: positive handoff refused, original keys remain usable |
| C13 | Missing/spoofed/wrong-nonce marker, lost signal subscription: no admission |
| C14 | Source final key callback completes after target FocusIn callback: no constructor-stale copy; valid final seal transfers exactly once |
| C15 | Source FocusOut callback executes before an earlier input settles: no complete-token certificate from that snapshot |
| C16 | Old FocusOutId(A), Reset or publish arrives after new owner B: B state/guard unaffected |
| C17 | Back-to-back E0->E1->E2 with out-of-order callback completion: only current ticket consumes; no obsolete frame apply |
| C18 | Acquire timeout / reply arrives after timeout: first partial token remains Unknown; late reply cannot bless it |
| C19 | First character unseen during transition, then observed j/v + Space: literal suffix delivered once, zero correction/Tab/manual/learning; next word after Space is eligible |
| C20 | Unknown suffix erased to empty, then new letters: still Unknown until observed boundary |
| C21 | Unknown word's Space closes it before Known rearm: no correction of that just-closed word |
| C22 | Backspace across the rearming boundary: revoke KnownStart; no joining into an older unknown prefix |
| C23 | Ring saturation, absent stamp, malformed lifecycle: no deadlock/unbounded allocation/unsafe transfer |
| C24 | Slow getter while burst input continues: observer drains, keyboard path never waits for RPC |
| C25 | Bus reconnect or owner loss while target pending: all old receipts/tickets/late replies rejected |
| C26 | Bridge enters on session bus while IBus focus/foreign signal is pending: marker-fenced snapshot or typed refusal; never compare cross-connection Sequence |
| C27 | Atomic prepare -> live revoke/owner transfer -> old commit: no state resurrection, lost new guard, extra output or feedback |
| C28 | Config/material reload during pending transfer/candidate work: text authority and frame gates remain independent; stale frame denied |
| C29 | Unknown ManualToggleV3: NotHandled -> daemon Complete(None), no daemon-word or exact-tail fallback |
| C30 | Admitted exact GTK and terminal routes: preserve existing distinct mutation plans, no duplicate sync/suppression/replay |

Run C01–C30 as explicit pure/state/transport cases with their declared scope.
Then the existing H01–H16 expansions remain required: both modifier orders,
pending/in-flight layout intent, boundary and profile combinations, clean and
protected text, 4 Shift taps, accepted Tab and auto-undo. Record their expanded
case count separately instead of calling the two tables "46 tests".

Positive client proof is separate: full `ljv` correction in the cached-false
runtime, mixed `lом` as a distinct frame, different-field control, and exact
letter delivery across the real switching interval. Observe key receipt,
CommitText/output stream, actual client text, tail/frame, owner/word/request
generations and edit effects. The installed-byte baseline's synthetic bus
alone does not prove the selected FIFO marker against real IBus.

Required transport test uses isolated IBus/protocol implementation of the
pinned lifecycle and dispatch queue, not desktop input. Assert custom marker
passes the same queue as GlobalEngineChanged, sender cannot be spoofed, native
and compatibility routes converge, and no process/session restart is needed.
Remote guarded Cargo/changed checks and required architecture refresh follow
implementation. No local Cargo or production smoke is authorized by this file.

## Admission and handoff to implementation

Baseline gate: **closed by the six-case installed-byte receipt**, within its
declared synthetic-RPC scope. Historical delivery and correction quality remain
unproven. Identity design choice: the bounded observer + signal-queue-fenced
canonical receipt above. Before transfer production promotion, C08–C15,
C23–C27 and the deployed IBus queue/mode contract are mandatory; a failure is
not repaired with extra reads, TTL or serial arithmetic.

Implementation must update the owning TD-121/architecture description and
coordinate the ordinary suppression token with TD-120. Keep unknown-prefix
safety, native identity, compatible acquisition, positive preservation,
client-visible restoration, resource measurements and independent review as
separate verdicts. Full TD-121 is not DONE until positive cached-false client
restoration and the negative authority cases pass.

This analysis writes only this evidence file. No implementation experiment or
graph refresh was run here; the parent owns the eventual architecture refresh.

## Addendum: callback rendezvous and finite proof gates

2026-09-05, follow-up design validation. This section supersedes the earlier
instruction to reject an **initially unobserved** callback stamp immediately.
It does not change the one-shot context/marker contract or permit a key to
wait for that context acquisition. No code, build, probe or runtime mutation
was performed for this addendum.

### New deployed-transport evidence and its exact limit

Read [td121-real-ibus-transport-baseline.md](td121-real-ibus-transport-baseline.md)
and the private receipt at
`/home/ubu/.cache/lay/td121-real-ibus-imX7Og/receipt.json`.
The installed IBus passed 22/22 declared transport cases; 20 foreign ABA
sequences preceded the marker. Get+marker maximum was 1,108 microseconds across
21 observations, with no early-property-before-foreign example (0/20).
The real Factory-before-FocusOut envelope was also observed by the parent.
This establishes ordinary deployed feasibility, not adversarial ordering,
zbus wiring, restoration or production latency.

GlobalEngineChanged's observed sender was `:1.0`; the fixture's own connection
had a different unique name. This is evidence against hardcoding the earlier
illustrative sender spelling. Bind subscriptions to the authenticated IBus
owner on this bus epoch; do not hardcode `:1.0`, the fixture's name or a sender
from the separate session bus. The current receipt's engine/factory callback
records do not include message sender fields; that small header-contract check
belongs to the zbus fixture/wiring test, not another broad desktop audit.

### Decision A: bounded local rendezvous, not immediate rejection

The design is implementable with zbus 5.15's existing macro interface. At the
start of the callback, capture a handler timestamp and look up its complete
header key `(connection epoch, sender, serial, path, member)` in the metadata
ring. A fast hit does not allocate a listener or construct a timer. On a miss:

1. Register a local event listener, **then recheck** the ring and revocation.
2. If still pending, await that notification versus one absolute deadline.
3. On notification, recheck exact stamp and generation; unrelated wakes do
   not extend the deadline. Eviction, revocation, observer termination or
   timeout are genuine failure outcomes, unlike the initial scheduling race.

The observer records a received stamp and advances its metadata under a short
mutex, drops that mutex, then notifies. Stamp publication never waits for the
associated key callback to settle. Settlement acknowledgment is a separate
later event used to seal the source tail; conflating these two is a deadlock.
No RPC, context Get, marker wait, engine lookup, lexical work or Shared lock
occurs on the observer's stamp-publication path.

Use an event-driven future, not spin/yield/sleep polling. A starting wait cap
of **1 ms** is an upper bound proposed for the rare local scheduling miss,
not measured normal latency. No application key queue, duplicated key bytes,
physical capture or replay is introduced: the already-dispatched method simply
has a bounded metadata prerequisite. The callback may resume with a known
stamp while canonical context is still pending; in that case the key follows
the existing literal route and UnknownStart rule immediately. It does **not**
wait for the 5 ms context acquisition.

Lock ownership must be stated accurately. zbus's
`ObjectServer::dispatch_call_to_iface` (`object_server/mod.rs:291–332`) acquires
the interface write lock before entering an `&mut self` method and holds it
across that method's future. Therefore this local rendezvous **does hold the
existing engine interface lock**. It holds no additional admission/Shared
mutex across await. This is an acyclic dependency:

```text
engine callback -> wait for metadata publication
observer        -> short metadata mutex -> release -> notify
observer        -X-> engine lock / Shared lock / callback settlement / RPC
```

The engine lock is released on callback completion, including timeout. Another
callback on that same engine can wait behind it for at most this additional
local budget; callbacks on other engines and the observer remain runnable.
There is no global `spawn=false`. A requirement to move this wait *before*
zbus's implicit write lock would require a custom Interface/wrapper and broader
state changes; it is unnecessary for the acyclic rendezvous above. Do not
claim that the macro releases its lock during await or recursively run the
connection executor while holding it.

`event-listener` 5.4.1 is already available transitively, and its listen-before-
recheck protocol supports this operation; an explicit direct dependency is
appropriate if used. `Event::listen()` allocates only on the slow path. Timer
and stream utility dependencies must also be explicit, not accessed through
invented zbus re-exports. Observer cancellation drops its active subscriptions;
keeping undrained MessageStreams alive after its task exits is forbidden.
An entirely starved async executor has no hard real-time guarantee; the design
does not pretend a timer can execute while the executor itself is stalled.

**Space budget correction:** `space_autocorrect_prefetch.rs:18–19,327` has a
3,500-microsecond wait budget inside the stated 4 ms product deadline. The
local rendezvous cannot be added on top. Carry the callback-entry timestamp
or an absolute deadline into the existing Space lookup and give it at most
`3500us - elapsed_since_handler_entry`, clamped at zero. Reserve the existing
overhead margin. This is bounded budget plumbing, not a longer timeout or a
second retry. Full client latency, including zbus scheduling, is still a
separate measurement. Record fast hits, waits, timeouts and evictions in the
existing diagnostic mechanism so normal input cannot silently become Unknown.

### Decision B: two deterministic fixtures, no live-IBus manipulation

The missing proof is finite. Use the actual new admission reducer in its
unit tests, not a second pretend reducer that merely repeats expected results.
A test-only driver models the inspected IBus FIFO dispatch operations and
chooses when to present events. This fixture is not runtime middleware.

**D1 — C09/C10 held-dispatch schedule with positive control.** Start with
source A, complete `l`, a valid target ticket and a pending context request.
Enqueue foreign and return-to-Lay signals into the test FIFO but do not deliver
them. Deliver `GetReply(A)` first. Assert the real reducer stays Pending, with
zero consumed ticket, owner transfer, edit, ordinary guard transfer or feedback.
Enqueue the marker after those signals. Drain foreign, return, marker in FIFO
order. Assert irreversible rejection even though the final path/name are A/Lay.
Repeat with no foreign signals and the matching marker: assert one successful
transfer of `l` and no duplicate output. A wrong-nonce marker must remain
Pending/refused. The first assertion makes removal of the marker prerequisite
an observable regression; the final assertion prevents equality-after-ABA.

This proves the reducer against the exact adversarial schedule absent from
the 22 real-IBus cases. It does not claim the installed daemon was forcibly
scheduled that way. Its validity rests on the independently inspected shared
FIFO queue and the deployed marker baseline, not a new assumption that replies
and broadcast signals use one queue.

**D2 — actual zbus MessageStream merge under adverse poll order.** Use one
in-process Unix socket pair and two ordinary zbus p2p Connections, following
zbus's own `tests/e2e.rs:33–69` fixture. No desktop bus, real IBus process,
keyboard or new long-running server is involved. Register the narrow streams
on the receiving connection, emit foreign/return/marker fixture signals in
that wire order, and use their *actual received* Message::recv_position values.
Do not construct fake zbus Sequences or use Message::from_bytes, whose receive
position would be zero.

A test-only gate around the earlier-signal OrderedStream returns Pending and
stores a waker until explicitly released; it must never falsely return
NoneBefore while an earlier message is hidden. Poll the real production Join
composition with the marker branch ready first. Assert it does not emit the
marker. Release the earlier stream and assert exact foreign, return, marker
order. Include an idle unrelated stream: its genuine NoneBefore result must
allow progress without waiting for a nonexistent event. Feed that merged
sequence into the same reducer and assert D1's rejection. This tests selected
merge wiring and the ordering contract; ordinary Stream/select readiness is
not accepted as a substitute.

For rendezvous, add deterministic poll tests with observer execution held
until after the callback's first lookup, and publication (a) before listen,
(b) between listen and recheck, (c) between recheck and await, and (d) after
await. All four must deliver the original key exactly once without poisoning
completeness when the captured generation remains current. Separately advance
the injected clock to the fixed deadline or revoke the owner: the callback
must terminate safely, with no waiter leak or deadline extension. These are
the bounded tests for C23/C24, not a new architecture project.

### Finite admission sequence; do not require proof of unwritten wiring

The two design unknowns raised for this follow-up are now bounded: the local
event rendezvous has an acyclic implementation using available zbus APIs, and
the missing signal/merge schedule has the two explicit fixtures above.
The written consequence analysis and deployed queue feasibility permit pure
helper implementation and its tests. There is no requirement to reproduce a
rare real-IBus race or implement a custom dispatcher before writing that helper.

Before wiring the new helper into production callback/bridge authority:

- **P121-1:** the actual reducer passes D1 and its no-foreign positive control;
- **P121-2:** the actual zbus Join composition passes D2, and the callback
  rendezvous tests cover all four wake positions plus timeout/revocation;
- **P121-3:** fixture headers resolve the authenticated IBus sender versus own
  marker sender, and budget plumbing demonstrably does not add a second Space
  wait allowance.

These are source/adapter tests, run remotely through the existing Cargo guard;
they may be built before runtime wiring. They need no production service,
source mutation in IBus, ptrace, eavesdrop, physical replay or external proxy.
Once they pass, proceed directly to the already-declared callback/bridge
wiring and C01–C30 promotion checks. Actual callback settlement, resource
measurements, native/cached convergence, positive restoration and client-visible
proof remain wiring/promotion gates, not reasons for another open-ended
pre-code research phase. No test PASS is asserted by this addendum itself.

## Bounded composition source-contract successor (2026-09-06)

TD-120 is independently accepted and pushed as
`ad4bf0860cc2a79b3004f03b8018cc8d8cbca005`. The admitted TD-121 implementation
needs an UnknownStart guard at the actual active-composition completion entry,
not just a caller or candidate-display check. That intentionally changes
`src/bin/lay_ibus_engine/composition_commit.rs`, whose TD-120 successor is
byte-pinned by `tests/td113_hybrid_source_contract.rs` and
`td120-composition-mutation-successor.json`.

Before editing that source-contract test, select a second explicit, scoped
successor binding (9/10). Preserve the immutable TD-113 preflight and the TD-120
successor document, require the TD-121 predecessor to equal the exact TD-120
path/mode/digest, and bind the final TD-121 path/mode/digest to its owning task,
independent review and separately named functional tests. Do not infer review
acceptance before it happens; final proof uses the actual reviewed bytes.
All other protected artifacts remain byte-pinned. This is a provenance chain,
not permission to substitute an arbitrary file or an environment-supplied hash.

Dropping the necessary UnknownStart guard to preserve the old hash is rejected
(1/10, loses actual-entrypoint safety). Replacing the narrow check with a
generic migration/attestation framework is also rejected (4/10 here, unnecessary
policy/API surface). A blanket hash rebaseline or removal/ignore of the old test
is not an admitted alternative.

The contract-test/JSON delta changes no candidate retention, ranking, runtime
authority, latency/deadline, CPU/RSS, allocation, cache, package/delta reload,
learning, concurrency, rollback or IME/daemon consumer behavior. Those remain
governed by the TD-121 runtime design and tests above. Its risks are an
unreviewed successor or accidental widening to other protected artifacts;
negative path/mode/predecessor checks and the exact independent review guard
against them. Maintenance adds one successor record and a bounded chain check,
not a new runtime owner/route/cache/fallback. Rollback restores the exact
TD-120 source and its still-immutable receipt.

Required separate proofs: UnknownStart active completion refuses output and
feedback; a complete admitted word still accepts; other ordinary/exact routes
retain their gates; the TD-113 source-contract test passes for the final
successor and retains predecessor protections. Actual functional test names,
final digest and execution results are recorded only after implementation.

## Transport type qualification and Graphify ambiguity (2026-09-06)

A premature graph refresh during the consolidated repair produced an
`imports_from` edge from `context_runtime.rs:3` to the unrelated productive-L2
format `Header`. Parent read the exact graph edge and source: the actual import
is `use zbus::message::Header;`, an external transport type, not a Lay L2 import.
The extractor conflated equal terminal symbol names. This is a graph-resolution
defect, not evidence of a Rust dependency from IME into that private L2 format.

Use the existing `ibus_interface.rs` spelling `zbus::message::Header<'_>` at
these callback signatures and remove the bare import (selected 9/10 bounded
clarification). A new graph-resolution framework is disproportionate (3/10 for
this release); accepting the false private-L2 import or weakening the gate is
not allowed. Both Rust spellings denote the exact same external type: no
candidate/ranking/authority, latency/allocation, package/cache, feedback,
concurrency, consumer, rollback or public API behavior changes. The normal
consolidated remote compile and final guarded graph refresh cover this delta.
Graphify's similarly name-collapsed `references` edges remain navigation
limitations and must not be cited as real transport-to-L2 dependencies.

## Acquisition execution ownership correction (2026-09-06)

The consolidated runtime review found that the implemented
`activate_context_from_header` still awaits the compatibility `Get` and marker
fence inside the `&mut LayIbusEngine` `FocusIn`/`FocusInId` interface method.
The timeout-cleanup repair prevents a stuck fence, but does not satisfy C24:
zbus retains the interface write lock for that whole future, so a slow getter
still delays the actual key callback by the 5 ms acquisition budget. This
corrects the earlier conclusion above that an acyclic wait alone was
sufficient. Only the callback-stamp rendezvous (absolute cap 1 ms) may remain
inside the implicit interface lock.

Compared routes:

| Route | Score | Consequence |
|---|---:|---|
| Start the existing adapter/reducer acquisition after the local stamp, keep its typed pending fence in adapter state, and let the target engine try-consume a ready result before a later callback | **9/10 selected** | One authority path and one bounded background future; a key arriving first is delivered literally under `UnknownStart` and never waits for the property RPC |
| Add a new engine actor or custom zbus `Interface` wrapper that owns callbacks and acquisition | 4/10 | Can move every await, but creates a broader owner/queue/reentrancy migration that TD-121 does not need |
| Keep the current awaited callback and reduce the 5 ms timeout | 1/10 | Still couples keyboard latency to RPC scheduling and cannot prove C24 |

The selected change must bind pending-result consumption to the exact target
engine path so another engine cannot steal or invalidate it. The adapter remains
the sole reducer, deadline, marker, and revocation authority; no application key
queue, duplicated key bytes, retry, fallback, second source of truth, new text
actor, or global executor mode is introduced. A detached acquisition future
may perform only the already-admitted Get/context reply/marker path and matching
deadline cleanup. It must never mutate an engine object or shared text state.

Consequence bounds before implementation:

- Candidate/lattice and ranking authority do not change. Until a matching
  result is consumed, `context_word_is_known` remains false, so completion,
  correction, Tab/manual mutation, feedback, and learning stay denied.
- The first key received while acquisition is pending must force the existing
  reducer's target lineage to `UnknownStart`, run the literal input path exactly
  once, and complete without waiting for Get or marker. A later ready grant may
  establish identity but must not upgrade that lineage.
- Keyboard tail latency adds only the existing callback-stamp rendezvous, capped
  at 1 ms and already deducted from the Space budget. Get/marker work uses its
  existing 5 ms absolute acquisition deadline outside the engine lock.
- CPU/RSS/allocation cost is one bounded executor future per activation plus the
  existing pending fence/timer. There is no polling, unbounded queue, OS thread,
  model work, cache, package/delta state, or reload interaction.
- Connection generation, request generation, nonce, target path, owner
  generation, and the absolute deadline remain the invalidation identity. Late
  completion and expiry may clear only their matching request/fence.
- A key/FocusOut/Disable racing readiness cannot install a result for another
  path or resurrect a revoked owner. Observer publication still never needs the
  engine or shared-state lock, so the dependency remains acyclic.
- IME and daemon APIs and the GTK/terminal mutation plans are unchanged. The
  compatibility and native acquisition paths still converge in the same
  reducer; delayed native enrichment remains a separate required case.
- Rollback is limited to returning acquisition awaiting to the FocusIn callback;
  that is fail-closed for authority but restores the known latency defect. The
  added pending-consume helper has no durable state and can be removed with the
  asynchronous start methods.

Required focused proof is: hold the controlled zbus `CurrentInputContext` Get,
show the FocusIn-side start returns and an actual key callback/literal route can
complete before the reply, then release Get+marker and show only the matching
target can consume the result; also cover expiry and a mismatched target. This
is scoped C24/concurrency evidence, not C01-C30, client restoration, release, or
deployment proof. Runtime authority changes only after those final production
bytes pass the guarded remote suite.

### Ready-outcome lifetime correction

The first detached implementation draft still left `PendingFence` as both the
5 ms acquisition deadline and the delivery slot. That is incorrect: a marker
which arrived in time was expired before the ordinary first key if the user did
not type within 5 ms of FocusIn. It also left the reducer owner absent until the
engine callback consumed the fence, so an observer-received key after a complete
marker could be misclassified as pre-admission input. Both make positive
handoff depend on executor timing.

The bounded correction separates three states:

1. **Incomplete acquisition:** Get/reply/marker is missing. This alone is
   governed by the 5 ms absolute deadline and is cleared on timeout.
2. **Marker observed, final source prerequisite pending:** the marker position
   is already fenced and may later become Ready when the earlier source callback
   settles/seals. The one bounded request remains generation-controlled; an
   intervening lifecycle/revocation clears it. The deadline is not reused as a
   delivery TTL.
3. **Ready outcome:** the observer-side adapter consumes the Ready reducer state
   into exactly one target-path/request/owner-generation-bound outcome and
   establishes the reducer owner before publishing later key metadata. This
   mutates metadata authority only, never an engine object or shared text. The
   engine later try-consumes that exact outcome and installs the already-proven
   text state. A later lifecycle generation invalidates it; elapsed idle time
   after successful acquisition does not.

This preserves receive-order behavior: marker-before-key classifies that key
under the new owner, while key-before-marker first executes literally and
poisons the retained transfer lineage to `UnknownStart`; the later marker may
establish identity but cannot restore completeness. The ready slot has capacity
one and no timer, polling, retry, key retention, replay, extra owner, or second
text state. Incomplete acquisition remains deadline-bounded, and all stored
outcomes are lazily and eagerly checked against the reducer's current owner and
generation before installation.

Additional focused proofs: marker before deadline followed by the first key
after that deadline still installs the matching grant; marker-before-key yields
a tracked new-owner callback; key-before-marker yields one native-unhandled
literal result and a later `UnknownStart` grant; wrong target and revoked owner
cannot consume the slot. Marker-before-final-settlement must promote when that
settlement occurs without extending or reissuing the acquisition.

### Consolidated epoch, initial-factory and native-enrichment corrections

These are corrections within the selected connection-local admission route,
not a second authority source. The parent inspected the implementation and
remote focused results on 2026-09-06. This addendum records that checkpoint;
it must not be presented as a document written before those edits or as final
release acceptance. No installed process or production authority changed.

- **Empty-tail epoch:** installing source-free metadata must not reset shared
  tail revisions to zero. The engine reserves `shared.handoff_tail_epoch + 1`
  with checked overflow and binds the consumed reducer grant to that revision
  exactly once, checking owner, activation and lineage. The identical revision
  is installed in reducer, engine and shared state. This preserves stale exact
  lease refusal without changing text authority. The remote pre-key lifecycle
  test proves a newer revision and rejects the old same-path epoch; it is not
  an actual GTK replay proof.
- **Initial factory ordering:** with no source owner, the real factory request
  now reserves one target/profile association instead of dropping it as
  Passive. The reservation carries no text or edit authority. Target binding
  and the later FocusIn consume it into the existing source-free request;
  the exact matching Lay signal and ordered marker are still required. A
  later foreign or mismatched profile revokes it. Both relative signal/focus
  orders use the same route; the matching signal also refreshes existing
  readiness notification, not only revocation notification.
- **Late native identity:** a native FocusInId enriches the matching pending
  compatibility request in place, preserving its request generation, nonce and
  deadline. The one detached future drops its pending Get when native identity
  wins and emits the existing marker. A completed compatibility outcome is
  installed/enriched without restarting activation, clearing the tail or
  promoting UnknownStart. No extra Get, marker lane, retry or polling loop is
  admitted. Both pending and completed native schedules passed focused remote
  tests; these are controlled-P2P results, not deployed client restoration.

Compared designs (judgment scores, not measured quality): retain exact factory
binding and enrich one request **9/10, selected**; postpone acquisition behind
a separately scheduled global-signal retry **4/10** (additional cancellation
and liveness route); treat current/guessed profile as sufficient **1/10,
rejected**. For epochs, one checked shared-revision reservation **9/10** is
smaller than migrating all exact consumers to a new revision domain **5/10**;
zero-reset is rejected for stale-lease reuse.

Consequences: candidate retention/ranking and verifier rules are unchanged;
identity alone never proves completeness. Hot-path waits retain the 1 ms
rendezvous and existing total Space allowance; acquisition retains its original
5 ms deadline. Extra state is one bounded factory reservation, not another
text buffer/worker/cache. Existing owner/activation/request/connection changes
invalidate it; checked epoch overflow refuses installation. Package/delta or
config updates grant no context identity and retain their existing certificate
invalidation. No learning/feedback, model load, external layout mutation or
daemon API is added. Late Get/native/lifecycle callbacks cannot own newer text.
Rollback is the unreleased TD-121 source delta; no installed migration exists.
Removal is coupled to this adapter/reducer route, not a future independent
framework. Fixed proof scope and exact raw logs are in the promotion ledger.

### Native-unhandled tail mirror: measured defect and bounded repair contract

The second combined adapter run reached actual `l` followed by Backspace and
failed its empty-tail assertion: the client-native deletion left the live IME
mirror at `l`. Source inspection explains it: the speculative Backspace updates
its tail mirror, but `commit_native_observation` retains only gesture fields.
This is a confirmed C20/C27 bookkeeping defect, not a reason to remove the
empty-tail assertion. Raw result:
`/home/ubu/.cache/lay/td121-remote-sidecar/td121-adapter-combined-pass2-wake-evdev-20260907.raw.log`.
The run was 20 PASS / 2 FAIL; the other failure is still being diagnosed and
must not be dismissed as fixture contamination without evidence.

Before changing this production path, the selected repair contract is a narrow
merge of **native observation bookkeeping only**, after validating the same
live owner/base (**9/10**). Preserve the pre-key tail used by completeness
settlement, synchronize the actual observed deletion/navigation mirror, and
leave unrelated newer suppression/exact leases untouched. Replaying a separate
live observation handler is viable but **6/10**, because it duplicates native
key classification. Committing the entire speculative clone on an unhandled
event is **1/10, rejected**: that could promote deferred layout, learning,
shared ownership or text-output state without an accepted atomic frame.

Implementation narrowing after inspecting the existing publisher: a native
Backspace can settle its mirror through `backspace_committed_tail_only` once on
the live engine, only when the actual speculative route observed that deletion
and the captured admission owner/token is still valid. This is narrower than
reimplementing a field-by-field merge and retains the existing guarded shared
publisher; it adds no client output. The original tail must be captured after
prior atomic settlement but before this observation. Moving that capture before
prior settlement reintroduces the reviewed H5 defect; capturing it after
deletion breaks boundary-crossing refusal. The final implementation must retain
both negative invariants, not merely the empty-tail assertion. Navigation and
other native effects must be reported according to their actual tested scope,
not inferred from the Backspace repair.

This repair must add no client output, physical replay, second text mutation,
new worker, retry, fallback, external call, persistent state or package/model
operation. It changes observation accuracy, not candidate scores or edit
authority. Tail work stays bounded by the existing mirror; there is no new
deadline or extra acquisition. A stale owner/base must not update a newer
mirror/guard, and no generic clone replacement may bypass that check. Material
reload and feedback rules remain unchanged. Failure stays literal/fail-closed;
rollback restores the old known mirror defect without an on-disk migration.

Acceptance: the actual native Backspace removes the observed character from
the live mirror, preserves UnknownStart after deletion to empty, and retains a
live valid token; crossing a real boundary revokes only completeness. Native
navigation and consecutive accepted AtomicV1 boundary/next-word receipts must
remain valid. Existing stale-clone/owner/guard tests and exact routes remain
mandatory. Do not equate controlled callback proof with a physical client edit.

Focused repair result: the guarded remote adapter namespace subsequently
passed **22/22**, 368 filtered, in 0.34 s, after compile exit 0. The internal
atomic result now carries the bounded pre-current-key tail after prior receipt
settlement; the callback uses that preimage for completeness. It is returned
data, not persistent state or a new authority lane. Backspace mirror settlement
reuses the existing guarded live observation helper and emits no client effect.
The C20 boundary, C21 deletion-to-empty, C22 command/navigation, C27 consecutive
receipt, ready-outcome, foreign-profile and two late-native tests all passed in
the same namespace run. Exact log:
`/home/ubu/.cache/lay/td121-remote-sidecar/td121-adapter-combined-pass3-native-preimage-20260907.raw.log`.
This is not a 390-test full-suite result, actual-client restoration, physical
keyboard evidence, final review, installation, or release acceptance. No
installed runtime authority changed. The earlier 17/22 and 20/22 failures and
zero-selected standalone invocation are retained, not relabelled successful.
# Final-pass residuals: explicit bounded replan — 2026-09-07

Final independent review is REPAIR_REQUIRED4/10,H4/M0; exact schedules and
source references are in `td121-code-review-pass2.md`. This is not a third
general review or an accepted release. One consolidated second repair is
limited to the existing reducer/adapter/runtime/bridge invariants below.

Root mechanism: receive-order facts and invalidation exist, but consumers
read incomplete projections of those facts: bridge ignores unfinished work;
native focus uses retained owner as proof of ongoing activation; ready install
checks owner without the rest of its grant; Backspace checks only a retained
boundary character even though Enter closes the mirror.

Options (judgment, not measured performance): existing-state invariant repair
9/10; temporarily disable word transfer6/10 (loses the release's feature);
replace the complete input ownership architecture3/10 for this release (new
routes and regression surface without faster proof). Select existing state.
Spectral ledger: +5 safety,+5 coherent authority,+3 testability; no new owner,
queue,timer,cache,RPC,model rule or fallback. One revocation snapshot may travel
with the existing grant; it copies the current generation, not a new counter.

Route: actual receive observer -> existing reducer -> existing acquisition
grant/settlement -> engine scope; bridge uses a stricter settled read of that
same reducer. Planned invariants before production edits:

1. Bridge issuance AND consumer validation require no unsettled keys and no
   unfinished lifecycle/acquisition. Keep ordinary engine-token revalidate
   usable during its own current key. New ingress after fence must refuse at
   consumer validation; refusal is not a wait/retry or mutation.
2. Native receipt enriches only the same still-settled activation. Outstanding
   FocusOut/Disable uses existing reflexive transfer; changed context uses
   existing source-free acquisition. Retain initial delayed compatibility
   enrichment and exact owner/receipt guards, without double Get/marker.
3. Ready outcome and installation bind connection, revocation, owner,
   activation and lineage from the grant. Never install old KnownStart with
   a freshly read unrelated token. Local known-word authority additionally
   requires scope/token lineage equality. Same-owner Reset/ContentType after
   readiness must invalidate both queued and already-taken grants.
4. Backspace beyond an empty observed mirror is an unproven input gap, even
   if the lost boundary was Enter. Conservatively revoke until the next real
   boundary; deleting the last actually observed letter alone is not such a
   gap. Do not add multiline history or another boundary tracker.

Consequences: candidate generation/lattice/ranking, packages/delta reloads,
learning algorithms and model identities unchanged. Only valid current-word
authority may expose correction/manual/feedback effects; UnknownStart retains
literal text. Conservative Busy/unknown can temporarily reduce correction in
interleaved events; positive controls after settlement and next boundary are
required. No SafetyGate/verifier bypass; no literal-word runtime exceptions.
No new awaits, polling or allocation-proportional scans; checks use bounded
existing state, and token capture replaces the existing post-install clone.
3500us Space deadline and source-free acquisition deadline unchanged. CPU/RSS
and optimized client latency still need measured final proof, not estimates.
Daemon legacy Shift, GTK exact replay, terminal commit, atomic prior settlement
and GNOME ownership remain separate unchanged transports. Speculative authority
must fail closed on concurrent revocation, including after a grant is taken.
Failure/rollback: do not deploy a failing candidate; installed1.0.65 unchanged.
Revert only this bounded delta if systemic regressions appear; no broad reset.
Maintenance cost: reuse existing guards, no parallel controller to remove.

Proof plan: controlled production legacy callback/observer tests for held
FocusOut/key with bridge fences (and ordinary current-key positive); same-object
native refocus A/A and A/B; same-owner revoke before ready consumption and
after take before install; legacy Enter/KPEnter -> Backspace -> Tab/manual
refusal and next-boundary positive. Record RED on pre-repair source, exact
test discovery and GREEN after repair, then full IME suite. Harness tests,
expanded schedules, five-case actual client and physical keyboard delivery
are separate denominators. No additional broad review round; closure evidence
must directly address final-pass findings. Unresolved new mechanisms require
another explicit decision, never a silent release promotion.

## Residual implementation and RED evidence

Production delta uses the existing lifecycle slots and unsettled-key queue for
bridge issuance/consumer checks; ordinary key revalidate remains unchanged.
Native enrichment returns an explicit not-applicable result for an unfinished
lifecycle, allowing the existing transfer/source-free route. Grants now carry
the existing revocation snapshot; installation keeps its captured grant token
instead of fetching an unrelated current one. KnownStart also checks token/scope
lineage equality. Empty-mirror Backspace revokes without new history storage.

New `adapter/tests/residuals.rs` is an included child of the existing word-scope
tests, sharing the real production entrypoint helpers, not a second simulator.
Only callback/bridge method Rust visibility was widened inside the binary to
call those same implementations; D-Bus signatures and policy are unchanged.
Test fixture paths now use the production engine prefix, needed for the
Properties.Set observer route; it would be wrong to remove that runtime filter.
Native FocusInId deliberately reloads config. The controlled fixture reinstates
its explicit IME config afterwards so remote-account defaults are not a hidden
test dependency. Shift setup includes both phases. Space release is consumed
after a handled press; its initially inverted fixture assertion was corrected.

Remote RED: `td121-residuals-red-contract-20260907.log`, **0/7 passed**,0ignored,
392filtered,0.09s; all seven reached the intended invariant failure on the
pre-repair production bodies. The two bridge tests first fail on unresolved
key; both re-focus contexts fail to start acquisition; queue/install tests
fail on same-owner Reset; Enter fails at prefix rejoin. Looped FocusOut,
ContentType and KPEnter variants do not independently reach RED after the
first panic; retain that denominator distinction. The initial
`td121-residuals-red-20260907.log` is a fixture failure for6/7 tests, not full
RED evidence. Exact pre-repair hashes are in
`td121-residuals-red-source-20260907.sha256` in the same remote log directory
`/home/e/.cache/lay/td121-remote-sidecar/`.

An intermediate compile caught private AdmissionToken lineage access; the fix
adds an equality query rather than making token fields writable. Subsequent
checks exposed two fixture issues (host config reload and noncanonical path
for Properties.Set), not permission to weaken runtime constraints. Logs remain
retained. These are preparation/compile cycles inside the same second repair,
not additional independent review passes or final acceptance.

## Final-pass checkpoint and architecture receipt

Focused7/7 PASS; full IME399/399 PASS,0ignored/filtered,15.49s. Exact discovery
has all7 residual identities. Full log SHA256
`8e882054a95b4c4146b7ac9dab5cd74db91271303388e917a480b553bbbad185`.
Optimized actual-client proof still0/5 at prefetch_not_ready; measured results
and the not-yet-proven lexical-warmup hypothesis are recorded in
`td121-private-client-proof.md`. Do not equate metadata admission with lexical
readiness, or either with actual correction/physical input acceptance.

Remote guarded `scripts/update-architecture-graph.sh` exited0 and produced a
PASS receipt plus `lay architecture check OK` after the repair. Exact log:
`/home/e/.cache/lay/td121-remote-sidecar/td121-residuals-architecture-20260907.log`,
SHA256 `7f09986416fed2cfb40b7d1f92701b095f6202b8e36f0a5755c500bf6fc1b692`.
AST navigation map:23514nodes,58701edges,1064communities;696Rust source bindings.
Warning:805non-extracted sources (mostly JSON/docs) produced zero nodes, so
this AST-only map is not semantic documentation coverage. Existing file-size
budget warnings remain; no broad refactor was added to silence them. Generated
graph/binding and `src/generated/architecture_graph_receipt.json` were copied
back from the worker. This task-evidence addendum is outside RECEIPT_INPUTS and
does not change the verified Rust sources or the canonical architecture doc.

Release boundary remains unchanged: installed CLI verified1.0.65; global IBus
PID4715, daemon3453123 and IME3453154 remained live. No production restart,
installation, commit or push. All task Cargo/private-client/graph jobs ended.
Review exception requested but not yet granted; actual final-pass pre-repair
score remains4/10. The current failing client and review gates prevent DONE.

# Legacy client first-boundary checkpoint — 2026-09-07

Status: `DISCRIMINATING_PROOF_REQUIRED`, not a new architecture migration.
User accepted the development simplification rules and directed recording
and execution. AGENTS.md is the rule owner; TD-121 remains the runtime owner.

Facts: candidate SHA256
`b7e783753d03950e81ff31aab40f17257ada78e668cdceaff101d3f28124e81c`
passed22/22 adapter and390/390 IME tests (full harness15.61s). Remote private
run `/home/e/.cache/lay/td121-private-five.cVZ7BN` completed0/5 client cases.
Visible Space committed, but the first snapshot was passive:unknown-context.
Shared active_path/epoch1 identifies successful source-free installation;
the trace has no bootstrap/observer-stopped event. That does not identify
the legacy callback's stamp disposition or prove marker-before-key ordering.
New word_scope tests call process_key_event_atomic_callback, whereas the client
calls ProcessKeyEvent. Helper/atomic results do not close that coverage gap.

Hypotheses still distinguished: (A) the key was received Passive before marker
ownership and handled after readiness installation, (B) callback rendezvous
expired or identity correlation failed, (C) an actual legacy settlement defect
after valid admission. None is selected as a proven runtime cause. The driver
checks engine/process presence, not admission readiness; its positive assertion
therefore lacks a recorded ordering prerequisite. Do not silently weaken it.

Options (engineering judgment):
1. Existing controlled adapter + actual legacy callback scenarios and bounded
   metadata tracing,9/10: smallest discriminating proof; selected first.
2. A causally synchronized actual-client setup using already-existing lifecycle
   events,8/10 conditional: appropriate after identifying the required order;
   no new runtime RPC, polling loop or manufactured grant may establish it.
3. Longer sleeps/acquisition budgets or accepting passive positive controls,
   1/10: masks the distinction and is rejected.

Pre-code consequence analysis for this diagnostic/test delta:
- Candidate retention/ranking, model packages/deltas/reload and learning remain
  unchanged; new fixture text never becomes a runtime condition. No SafetyGate,
  verifier, word-completeness or edit-authorization rule is loosened.
- Use the existing reducer, peer and legacy callback. Controlled test order is
  test-only; no extra production owner, cache, timer, fallback or queue is added.
  If calling the actual method requires a visibility-only change, keep its body
  and production call sites unchanged; do not duplicate the handler in a test.
- Extend existing opt-in trace sites with bounded event/member/disposition,
  owner-generation and refusal-stage metadata only. No user text or raw message
  bodies, new synchronous IO, RPC, readiness polling or eager disabled formatting.
  Existing logger bounds remain; enabled overhead is a cost, not assumed zero.
- Existing1ms callback rendezvous, acquisition deadline and3500us Space allowance
  remain unchanged. Test timeouts bound failed tests; they do not tune runtime.
- Stale-result/order tests must exercise both marker-before-key and key-before-
  marker with actual observation and handler settlement, plus a later legitimate
  boundary. Key-before-ready may remain literal/UnknownStart as designed; no
  historical input may acquire retrospective correction/learning authority.
- Preserve daemon-owned physical Double Shift and distinct legacy/atomic/GTK/
  terminal effects. No production process or global IBus restart is admitted.
- Rollback boundary is the exact diagnostic/test delta, preserving all prior
  repair bytes and evidence. Remove redundant diagnostic detail once equivalent
  existing refusal metadata is proven; retain the smallest regression scenarios.
- If these tests reveal a behavioral change is required, record the first failed
  invariant and a bounded repair analysis here before editing its runtime logic.

Proof denominators: exact discovered legacy test names and selected count,
controlled positive/negative schedules and outputs, prior-failure/controlled-
violation sensitivity, affected regression suite, and actual-client cases
reported separately. No physical-key or release acceptance is inferred.

Parent takeover: both delegated turns stopped at an account usage limit before
the diagnostic candidate was built. No remote Cargo remained. Before completing
the trace delta, retain callback_entered at its original position before trace
work, so diagnostic overhead is not excluded from the existing deadline; gate
additional diagnostic reducer reads/Arc clones behind enabled(). These are
instrumentation corrections, not changed admission/word-completeness rules.

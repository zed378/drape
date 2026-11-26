# Feature Spec - R0-05 Native icon state and independent recovery

**Written:** 2026-11-26 - **before** implementation  
**Revised:** 2026-11-26  
**Task:** R0-05  
**Author:** orchestrator; revision drafted by `codex exec` sessions coordinated by a Claude Code session  
**Status:** draft revision, **not accepted**. Safety review: ACCEPT. Core review: REJECT with five open findings in the [revision record](../records/2026-11-26-R0-05-spec-revision.md). No implementation may start from it until they are closed.  
**Spec refs:** [Windows integration](../../docs/PLATFORM/00-WINDOWS-INTEGRATION.md), [Security](../../docs/SECURITY/00-SECURITY-AND-PRIVACY.md), [System architecture](../../docs/ARCHITECTURE/00-SYSTEM-ARCHITECTURE.md), [Rust migration](../../docs/ARCHITECTURE/03-RUST-MIGRATION.md), ADR-006, ADR-008, ADR-025 and ADR-032 in [Decisions](../DECISIONS.md)

## Problem

Native icon hiding needs independent recovery after normal exit, error, App death, Guard loss, both-process loss and reboot. Standalone restoration must coordinate with a live owner and prevent immediate rehide.

An observed hidden desktop does not establish ownership. A read followed by a toggle is not atomic with a Windows menu action. This specification defines required contracts, not evidence that a Windows mechanism works. Production hiding stays disabled until native evidence and compatibility review establish them.

## What `docs/` Already Decides

| Decision | Source |
|---|---|
| Discover the host dynamically; never assume its hierarchy | Windows integration sections 1 and 3 |
| Read before and after commands; unknown visibility never authorizes a toggle | Windows integration section 3; ADR-006 |
| Preserve pre-hidden icons and detected later user choices | Windows integration section 3 |
| Durable intent and authenticated Guard readiness precede hiding | System architecture; Rust migration |
| Serialize App, Guard and CLI by user/session, generation and actual process identity | System architecture; Security |
| Guard loss requires prompt owned restoration and suspension until rearmed | System architecture; task card |
| Recovery is independent of config/log success; Drop/panic hooks are best effort | CLAUDE.md; System architecture |
| Preserve unsafe recovery evidence and offer manual guidance | Windows integration; Security |
| Never automate Explorer termination or destructive system changes | AGENTS.md; Security |
| Uninstall attempts every established owned restoration; unresolved recovery retains evidence and reports pending manual recovery | Security, User Data Rights |

Read the CLAUDE.md desktop safety section as a whole. “Guard failure leaves native icons visible” stands beside preservation of user-hidden icons: failed readiness issues no hide and preserves pre-transaction visibility. “Every exit path restores owned changes” stands beside retention of durable retry state: every exit path initiates bounded authorized restoration; incomplete restoration retains responsibility and reports `Pending`. These are the governing requirements, not proposed deviations. The task card and architecture wording about failed Guard launch has the same interpretation.

**ADR-032 owner decision:** after a monitoring gap, a valid durable committed verified-hide record with no supersession authorizes restoring a currently hidden desktop. Intent or unverified dispatch alone does not. Continuous monitoring from before dispatch can establish an uncommitted owned hide after positive settlement.

A committed `Safe` transaction is permanently closed to visibility commands. Its historical hide certificate never authorizes another command, including after cleanup failure.

## Layering and Interfaces

- `drape-core` owns pure deterministic validation and transitions. It performs no I/O, parsing, clock reads, process inspection or native calls.
- Adapters supply bounded parsed values, authenticated evidence, monotonic elapsed values and effect replies. Core returns state, ordered effects and typed results.
- `drape-shell-windows` executes coordination, persistence, observation and native effects using reviewed RAII. `drape-guard` executes recovery independently of App, Slint, config and logs.
- `drape-app` parses `--restore-desktop` before rendering. Normal exit, shutdown, error, best-effort panic and uninstall translate to authenticated `Restore(requestId,cause)`. Abort/kill recovery uses `OwnerDead`; restart uses `Start`.
- Panic delivery is best effort and must not block, recursively panic or unwind across FFI. Forced termination guarantees neither delivery nor completion.
- Core handles bounded opaque identity/proof descriptors, never native handles. PID, pathname and timestamp text alone confer no authority.
- Interface approval, recovery-format review and native evidence precede production adapters.

## Desktop Safety

### Required native contract

`Change(target,operationId)` validates execution-time authority, dynamic host, a fresh authoritative read, bounded dispatch and authoritative read-after. Hiding also revalidates Guard health and committed intent/operation identity. Already at target means no visibility command.

Unknown state, unavailable host, invalid authority or supersession prevents dispatch. Detect external changes wherever supported. Unexpected read-after is supersession only with positive external-change evidence; otherwise it is indeterminate.

No atomic Windows setter is assumed. The read-to-command race remains a measured residual window requiring owner review.

Every dispatched command requires positive settlement semantics. Timeout does not prove non-execution. Never replay an indeterminate operation or reuse its ID. An unsettled command blocks every later visibility command. Positive settlement can permit a separately identified restore with fresh eligibility and durable budget consumption.

### Coordination and ownership

Acquire per-user/session coordination before recovery-record inspection or singleton decisions. One authorized actor serializes commits and native commands. A normal App mutex is insufficient.

Every effect binds scope, generation, authority epoch and unique effect ID; commands also bind a unique operation ID. Validate authority at execution and commit. A successor requires confirmed owner death or a proven protocol fence, plus settlement of old writes and commands. Process death alone does not settle an Explorer message.

`FenceSnapshot` revokes queued old authority, settles displaced work, reconciles revisions and returns authenticated owner, record and host classifications. It never infers success from timeout. Unproven fencing permits no mutation. The spike must establish bounded acquisition despite a dead or hung actor.

### Live restore, suspension and Guard loss

A validated restore atomically latches `restoreOnly`, replaces acquisition purpose with `Restore`, retains its waiter and cancels hide/quick-hide/rearm tokens before acceptance. Acceptance alone is not durable suspension acknowledgement. That requires a committed latch or positive fencing of every old hide authority.

Except for committed `Safe`, attempt `RestorePending` before recovery observation. A definitely failed write cannot block restoration supported by earlier valid durable authority. An indeterminate write requires fencing and rereading first.

A CLI with a live or uncertain owner enters `LiveRequest` and emits `RequestOwnerRestore`. It never restores from its own record interpretation. Owner-mediated verified completion can finish the request. Local succession requires positive owner-death/protocol-fence evidence followed by successful `FenceSnapshot` settling all work. Timeout, rejection without that evidence or an unproven fence ends `Pending`. Never terminate an unresponsive owner.

Normal duplicate startup signals the validated owner and exits without restoring. Guard loss immediately suspends hiding, invalidates tokens and fences outstanding work; promptly restore established owned changes. Ambiguous ownership permits observation and manual guidance only.

Suspension survives restoration, restart, recovered IPC, time and replacement Guard readiness. Only explicit resolved-state `Rearm` can authorize a new generation through the commits below. Restoration remains permitted during suspension, subject to ownership and terminal-transaction gates.

## Data and Config

Use separate app-owned `state.json`. Adapters enforce parsing/allocation bounds and exact camelCase wire names. Core validates the complete parsed structure. A language migration alone does not change a schema version.

### Version 1 validation contract

The candidate format is `(schemaVersion,protocolVersion)=(1,1)`, subject to namespace and legacy compatibility review. All listed keys are required; `?` means explicitly nullable. Unknown/duplicate keys, invalid types, bounds or combinations are invalid.

| Fields | Closed type and meaning |
|---|---|
| `schemaVersion`, `protocolVersion`, `recordRevision` | Positive bounded integers; revision strictly increases on every commit |
| `userScope`, `sessionIdentity`, `generation`, `authorityEpoch`, `transactionId` | Nonempty bounded typed identifiers; session includes incarnation |
| `ownerIdentity` | `ProcessIdentity={principal,session,processIncarnation}` |
| `guard` | `?{identity:ProcessIdentity,challenge:Id}` |
| `phase` | `Prepared`, `HidePending`, `Active`, `RestorePending`, `Safe`, `Superseded` |
| `restoreOnly`, `suspensionReason` | Boolean; nullable reason: `Recovery`, `Requested`, `GuardLost`, `OwnerLost`, `AuthorityLost`, `ExternalChange`, `HostLost`, `Failure`, `ExplicitRearm` |
| `originalVisibility`, `baseline` | `Visible` or `Unknown`; nullable observation certificate |
| `hideVerification` | Nullable hide certificate; sole durable verified-hide flag |
| `operation` | Nullable `{id,authorityEpoch,target,host,status}`; target `Hidden`/`Visible`; status `Authorized`/`Settled` |
| `safeVerification`, `supersession` | Nullable visible-observation certificate and nullable supersession certificate |
| `retryPolicyId` | Bounded identifier of the reviewed limits and delay policy |
| `recoveryReadsConsumed`, `restoreAttemptsConsumed` | Nonnegative bounded lifetime totals; monotonic across generations and episodes |
| `retryEpisode` | `{id,origin,explicitSequence,readsConsumed,restoreAttemptsConsumed}`; origin `Initial`/`Explicit`; sequence null for Initial, otherwise positive; counters nonnegative and bounded |
| `lastExplicitRestoreSequence` | Nonnegative durable watermark for accepted explicit recovery episodes |

All identifiers have reviewed encoding/length bounds. Integer overflow prohibits dispatch and returns `Pending`; it never wraps. Process descriptors are structural claims until adapters validate OS identities. Timestamps establish neither identity nor elapsed budget.

An observation certificate is `{binding,host,observer,sequence,visibility}`, with visibility `Visible`/`Hidden`. A hide certificate is `{observation,operationId,operationAuthorityEpoch}` with hidden visibility. A supersession certificate is `{binding,host,observer,sequence,kind}`, with kind `UserVisibilityChange`/`OtherExternalChange`. `binding={userScope,sessionIdentity,generation,transactionId}`. Host/observer are incarnation identifiers; observation sequence is positive and bounded.

Certificate bindings equal their record binding. Operation IDs are unique across transactions. A hide certificate binds that transaction's dispatched hide and execution epoch. A visible baseline precedes the hide. `safeVerification` follows positive settlement of every authorized operation. Historical host certificates require current-host revalidation before action.

Structural consistency does not authenticate persisted evidence. Adapters validate principal/session, provenance, actual process identity, authority, settlement and proof semantics before supplying trusted classifications. Persisted proof text alone never authorizes mutation.

| Phase | Latch/reason | Baseline/original | Guard | Operation | Hide certificate | Safe certificate | Supersession |
|---|---|---|---|---|---|---|---|
| `Prepared` | false/null fresh; true/`ExplicitRearm` rearm | Present/Visible | null | null | null | null | null |
| `HidePending` | false/null | Present/Visible | Present | Hidden, Authorized | null | null | null |
| `Active` | false/null | Present/Visible | Present | Hidden, Settled, matching hide certificate | Present | null | null |
| `RestorePending` | true/non-null | Present/Visible or null/Unknown | Nullable | Nullable, either status/target | Nullable | null | Nullable |
| `Safe` | true/non-null | Present/Visible or null/Unknown | Nullable | null or Settled | Nullable | Present | Nullable |
| `Superseded` | true/`ExternalChange` | Present/Visible or null/Unknown | Nullable | null or Settled | Nullable | null | Present |

A hide certificate requires a visible baseline; original Unknown forbids one. Baseline and committed certificates are immutable within a transaction. Recovery cannot promote uncommitted hide evidence after a monitoring gap.

`RestorePending` preserves committed hide and supersession certificates. Retained supersession requires reason `ExternalChange`, permanently defeats command eligibility and makes every recovery read visible-only. Budget and episode writes cannot remove it.

`Safe` is terminal for visibility. Cleanup retries preserve its phase and certificates; they never write `RestorePending`. A new hide requires a distinct generation/transaction and a fresh bound baseline, never rebinding an old certificate.

Classification precedence is: absent → `Absent`; recognized legacy shape, including `iconsHiddenByDrape` → `Legacy`; malformed/duplicate/structurally invalid → `Invalid`; well-formed unknown version pair → `Unsupported`; complete valid `(1,1)` → `Supported(phase)`. Incomplete claimed `(1,1)` is invalid. Wrong scope/session grants no cross-session authority. Legacy, invalid and unsupported bytes remain read-only.

### Durable ordering

| Transition/write kind | Required ordering |
|---|---|
| Fresh hide or rearm: `Prepared` | Reserve new binding, obtain its fresh visible baseline, commit new generation before `ArmGuard`; rearm retains consent token and latch |
| `HidePending` | Commit Guard identity, operation and `restoreOnly=false` before `Change(Hidden)`; only matching success with valid gates clears the in-memory rearm latch |
| `Active` | Commit hide verification before entering `Owned`; failure initiates restoration |
| `RestorePending` | Attempt durable suspension before recovery observation; definite failure may retain earlier durable authority |
| `Episode` | Commit new explicit episode, watermark and zero episode counters before any read/attempt charged to it; retain lifetime totals and all ownership/supersession evidence |
| `Budget` | Commit `RestorePending` with incremented episode and lifetime read counters before each recovery read |
| `RestoreAttempt` | Commit `RestorePending` with new authorized visible operation and incremented episode/lifetime attempt counters before `Change(Visible)` |
| `Safe` | Commit visible verification and settled operation before matching Guard disarm |
| `Superseded` | Revoke ownership, latch and cancel tokens immediately; attempt persistence; failure never revives ownership |
| Any uncertain write | Fence and reread before another write or command; never overwrite an uncertain revision |

`Episode`, `Budget` and `RestoreAttempt` are write kinds, not phases. Atomic replacement, durability and definite versus indeterminate failure need native evidence. No automatic transition, restart or rearm replenishes episode budgets.

### Restart classification

Every acquired recovery transaction starts restore-only and never resumes a hide. Persisted suspension applies to every entry purpose.

| Record after owner classification and settlement | Restart behaviour |
|---|---|
| Absent | No ownership. Normal startup never hides. Recovery may create restore-only metadata; hidden remains ambiguous; visible may checkpoint `Safe`. |
| `Prepared` | No hide dispatch authorized. Hidden has no owned attribution without separate uninterrupted evidence. Never resume preparation. |
| `HidePending` | Dispatch may have occurred. After a gap, intent alone cannot authorize restoration. |
| `Active` | Fresh hidden observation without supersession may qualify under ADR-032. |
| `RestorePending` | Retain certificates and episode counters. Without committed hide evidence, restoration requires uninterrupted monitoring and positive settlement. Supersession permits visible-only resolution. |
| `Safe` | Matching cleanup only. Never replay visibility or revive historical hide authority, even after failed disarm/release or a later user hide. |
| `Superseded` | Preserve supersession through `RestorePending` and budget writes. Visible-only reads may resolve to `Safe`; hidden requires manual resolution. |
| Invalid, unsupported, legacy | Preserve bytes read-only; report ambiguity/manual recovery. Never silently treat as absent. |
| Wrong session or unvalidated identity | Reject authority, retain evidence; no cross-session action or process control. |

Supported unresolved records can become `Safe` after authoritative visible observation and settlement without a command. Unknown state or absent Explorer never proves success. Unsupported records require reviewed compatibility handling even after manual restoration.

## Behaviour

### Closed machine vocabulary

States are `Idle`, `Acquire`, `Read`, `Write`, `Arm`, `Change`, `Owned`, `Signal`, `LiveRequest`, `Fence`, `Wait`, `Cleanup`, `Suspended`, `Ambiguous`.

Closed parameters: `Read.kind={Baseline,PreHide,Restore}`; `Write.kind={Prepared,HidePending,Active,RestorePending,Episode,RestoreAttempt,Budget,Safe,Superseded}`; `Change.target={Hidden,Visible}`; `Cleanup.stage={Disarm,Release}`; `Wait.resume={Acquire,RestoreRead}`. Purpose is `Normal`, `Recover`, `Restore`, `Hide` or `Rearm`. State data contains role, latch, terminal-Safe marker, tokens, reserved binding, durable snapshot, continuous evidence, budgets, bounded waiters, saved result and pending effect.

External events are `Start(role,purpose)`, `Hide`, `Restore(requestId,cause)`, `Rearm(requestId)`, `GuardLost`, `OwnerDead`, `OwnerUncertain`, `AuthorityLost`, `ExternalChange(evidence)`, `HostLost`, `Tick(now)`. Roles are `App`, `Guard`, `CLI`; external start purposes are `Normal`, `Recover`, `Restore`. Restore causes are `Explicit`, `Exit`, `Shutdown`, `Error`, `Panic`, `Uninstall`.

Effects are `AcquireSnapshot`, `ReadProof`, `WriteRecord`, `ArmGuard`, `Change`, `SignalOwner`, `RequestOwnerRestore`, `FenceSnapshot`, `DisarmGuard`, `ReleaseAuthority`. Each has `Done(effectId,payload)`, `Failed(effectId,reason)` and `Indeterminate(effectId,reason)`. Reasons are `Unauthorized`, `Unavailable`, `Timeout`, `Io`, `Rejected`, `Invalid`, `Exhausted`, `Unknown`.

`Failed` guarantees no uncertain application or outstanding work. Otherwise return `Indeterminate`. A malformed matching reply is indeterminate. Adapters enforce named deadlines and synthesize indeterminate replies at expiry. Stale replies never consume the pending effect. `Done` payloads provide the authenticated classifications required below, not arbitrary adapter-selected transitions.

Results are `Restored`, `AlreadySafe`, `Pending`, `Failed`, `Unavailable`, `Superseded`, carrying reason, suspension and manual-action requirement. Results are returned outputs, not fallible I/O effects. `AlreadySafe` for absent normal startup means no established owned recovery obligation, not verified visible icons.

### Dispatch and ordering rules

Authenticated batches order authority loss, external change, Guard/owner loss, restore requests, replies, ticks, then hide/rearm; authenticated event IDs break ties. Apply these rules before the tables:

1. Invalid authentication, scope, generation, replay or role/purpose produces no effects; requests receive `Failed`. Retain stale settlement evidence only for fencing.
2. **Terminal Safe precedes generic recovery.** Once this transaction's `Safe` commit is confirmed or reconciled, every interruption, restore and cleanup retry routes exclusively to matching cleanup. Fence displaced work first; reacquire if authority is absent. No read/restore budget write or visibility command is permitted for that transaction. Uncertain cleanup never clears this marker.
3. Duplicate restore during active recovery attaches a bounded waiter, emits no effect and preserves purpose `Restore`. Overflow returns `Pending`. It cannot open another budget episode.
4. A first recovery interruption replaces purpose with `Restore`, retains caller/result context, latches suspension and cancels hide/rearm tokens. Losses without callers create internal recovery context.
5. In `Fence`, interruptions update context without another fence. `Acquire`, `Signal` and `LiveRequest` retain owner mediation and use the replaced purpose at their next classification.
6. With an outstanding ordinary effect, interruptions enter `Fence`. With authority and settled work, enter `Write(RestorePending)`; without authority acquire first. Idle/suspended/ambiguous recovery acquires first.
7. `AuthorityLost`/`OwnerUncertain` requires fencing before local writes. A CLI without established authority instead uses `LiveRequest`. Timeout never grants succession.
8. Positive supersession, including reply evidence, first revokes ownership, latches `ExternalChange` and cancels tokens. Fence outstanding work; otherwise persist `Superseded` if authorized or reacquire. Retain the certificate permanently.
9. `HostLost` invalidates current proofs and suspends hiding. Fence outstanding work; otherwise wait boundedly for acquisition or a budgeted recovery read. Do not infer restoration.
10. Busy/suspended `Hide` fails. Only the specified rearm route accepts `Rearm`. Every unlisted state/event pair keeps state, emits no effect and rejects requests with `Failed`.

One ordinary effect may be outstanding. A fence is the sole control operation allowed to settle displaced work. No ordinary effect runs until positive fencing completes. Global rules and the following tables cover every admissible pair; rule 10 covers all others.

### Acquisition mapping

Valid starts are `App/Normal`, `App/Recover`, `App/Restore`, `Guard/Recover`, `CLI/Restore`. All others fail without acquisition.

| Settled snapshot | Purpose/actor | Route |
|---|---|---|
| Other validated live responsible owner | Normal / Recover | `Signal`; `SignalOwner(DuplicateStartup / RecoveryNotice)` |
| Same | Restore | `LiveRequest`; `RequestOwnerRestore` |
| Same | Hide/Rearm | Cleanup release with `Failed` |
| Uncertain owner | CLI/Restore | `LiveRequest`; `RequestOwnerRestore`; no local record interpretation |
| Uncertain owner or unsettled old work | Other | `Fence`; no mutation before positive settlement |
| No responsible owner; invalid/unsupported/legacy | Any | `Ambiguous`, `Pending`; no write |
| Same; supported unfinished phase | Any | Purpose Restore, cancel tokens, latch; `Write(RestorePending)` |
| Same; absent | Normal | Cleanup release; no hide |
| Same; `Safe` | Any except accepted Rearm | Matching cleanup only; saved `AlreadySafe` |
| Same; absent | Recover/Restore | `Write(RestorePending)` |
| Same; absent | Hide | If unlatched, reserve binding and `Read(Baseline)`; otherwise release `Failed` |
| Same; `Safe` | Rearm | Require fresh explicit token, settled work and resolved record; reserve new binding, retain token/latch, `Read(Baseline)` |
| Same; absent or Superseded | Rearm | Release `Failed` |
| Same; Superseded | Hide | Release `Failed` |
| Same; Superseded | Other | `Write(RestorePending)` retaining supersession; visible-only recovery |

`Hide` against a previously suspended `Safe` record requires explicit `Rearm`. Cleanup never disarms another generation. Acquisition revalidates consent, settlement and record identity. A reserved rearm binding does not modify or reopen the old Safe transaction; only successful `Prepared` commits the new transaction.

### Transition table

`P(k)` enters `Write(k)` and emits `WriteRecord`. `F` enters `Fence` and emits `FenceSnapshot`. `S(x)` enters `Suspended`, retains evidence/responsibility and reports x. `Finish(x)` saves x and enters `Cleanup(Release)` with `ReleaseAuthority` when authority is held; otherwise it reports x in Suspended/Idle according to the latch.

`NextRead` checks total elapsed and episode budgets, then emits `P(Budget)` or `S(Pending)`. Every `Read(Restore)` uses this path. After `RestorePending`, an eligible fresh explicit request uses `P(Episode)` first; all other recovery uses existing counters.

| State | Event/reply predicate | Next state and ordered outputs |
|---|---|---|
| Idle | Valid Start; App Hide | Acquire selected purpose; `AcquireSnapshot`, consuming local acquire attempt first |
| Suspended/Ambiguous | Valid Start or Restore | Acquire restore-only route; `AcquireSnapshot` |
| Suspended | Fresh App Rearm with durable Safe and settled work | Create one-use token; Acquire/Rearm; `AcquireSnapshot` |
| Acquire | Done | Apply acquisition mapping using current purpose |
| Acquire | Failed | Wait(Acquire) if local bounds remain; otherwise S(Pending) |
| Acquire | Indeterminate | Ambiguous; Pending; no write |
| Signal | Done/Failed/Indeterminate, Normal/Recover | Finish: successful Normal AlreadySafe; successful Recover Pending; failure Failed; uncertainty Pending |
| Signal | Any reply, purpose now Restore | LiveRequest; `RequestOwnerRestore` after fresh owner validation |
| LiveRequest | Done: authenticated durable/fenced suspension and verified completion | Finish owner's Restored/AlreadySafe |
| LiveRequest | Done: positive owner-death/protocol-fence evidence for succession | F |
| LiveRequest | Other Done; Failed; Indeterminate | Finish(Pending); no takeover |
| Read(Baseline) | Done: fresh authoritative visible, bound to reserved transaction, no supersession | P(Prepared), retaining rearm token/latch |
| Read(Baseline) | Done: authoritative hidden | Finish(Superseded); no ownership or new Prepared |
| Read(Baseline) | Other Done; Failed | Finish(Unavailable); discard consent token |
| Read(Baseline) | Indeterminate | Latch, purpose Restore; F |
| Write(Prepared) | Done | Arm; `ArmGuard` |
| Write(Prepared) | Failed | S(Unavailable); no hide |
| Arm | Done: matching authenticated readiness | Read(PreHide); `ReadProof` |
| Arm | Other Done; Failed | Latch, purpose Restore; P(RestorePending) |
| Read(PreHide) | Done: visible, current authority/host/healthy Guard, no supersession | P(HidePending) |
| Read(PreHide) | Done: positive supersession | Apply rule 8 |
| Read(PreHide) | Other Done; Failed | Latch, purpose Restore; P(RestorePending) |
| Write(HidePending) | Done: gates/token valid | Clear rearm latch; Change(Hidden); `Change` |
| Write(HidePending) | Done: revoked gates/token; Failed | Latch, purpose Restore; P(RestorePending); no hide |
| Change(Hidden) | Done: settled verified owned hide | P(Active), including hide certificate |
| Change(Hidden) | Done: settled no application; Failed | Latch, purpose Restore; P(RestorePending); never retry hide |
| Either Change | Done: positive supersession | Apply rule 8 |
| Write(Active) | Done | Owned |
| Write(Active) | Failed | Latch, purpose Restore; P(RestorePending), retaining uninterrupted evidence |
| Owned | Restore or relevant loss | Apply dispatch rules |
| Write(RestorePending) | Done; Failed with earlier valid durable recovery record | Eligible explicit episode: P(Episode); otherwise NextRead |
| Write(RestorePending) | Failed without earlier valid durable record | S(Pending) |
| Write(Episode) | Done | NextRead |
| Write(Episode) | Failed | S(Pending); no use of proposed budget |
| Write(Budget) | Done | Read(Restore); `ReadProof` |
| Write(Budget) | Failed | S(Pending); no read |
| Read(Restore) | Done: authoritative visible, all work settled | P(Safe); save AlreadySafe unless this episode verified its restore |
| Read(Restore) | Done: hidden, positive new supersession | Apply rule 8 |
| Read(Restore) | Done: hidden with retained supersession | Ambiguous; Pending/manual guidance; no command |
| Read(Restore) | Done: hidden, eligible, attempt/time budgets remain | P(RestoreAttempt) |
| Read(Restore) | Done: hidden, eligible, budget exhausted | S(Pending) |
| Read(Restore) | Done: hidden without eligibility or unknown | Ambiguous; Pending/manual guidance |
| Read(Restore) | Done: host unavailable; Failed | Wait(RestoreRead) if bounds remain; otherwise S(Pending) |
| Read(Restore) | Indeterminate | Ambiguous; Pending; no command |
| Write(RestoreAttempt) | Done | Change(Visible); `Change` with new operation ID and execution-time checks |
| Write(RestoreAttempt) | Failed | S(Pending); no command |
| Change(Visible) | Done: settled verified visible | P(Safe); save Restored |
| Change(Visible) | Done: settled no application; Failed | Wait(RestoreRead) if bounds remain; otherwise S(Pending) |
| Either Change | Indeterminate; unexpected read-after without supersession proof | Latch, purpose Restore; F; permanently prohibit operation-ID replay |
| Write(Safe) | Done | Mark terminal Safe; matching Disarm cleanup, otherwise Release cleanup |
| Write(Safe) | Failed | S(Pending); no disarm or success report |
| Write(Superseded) | Done; Failed | S(Superseded); ownership revoked, Guard inert |
| Any Write; Arm; Read(PreHide) | Indeterminate | Latch, purpose Restore; F |
| Fence | Done: positive settled authenticated snapshot | Apply acquisition mapping directly, purpose Restore, with terminal Safe precedence |
| Fence | Failed; Indeterminate | Ambiguous; Pending; no mutation |
| Wait | Tick before deadline | Keep state |
| Wait(Acquire) | Due Tick, local bounds remain | Acquire; consume local attempt, `AcquireSnapshot`; no record write |
| Wait(RestoreRead) | Due Tick, relevant bounds remain | NextRead |
| Wait | Due Tick, relevant bounds exhausted | S(Pending) |
| Cleanup(Disarm) | Done | Cleanup(Release); `ReleaseAuthority` |
| Cleanup(Disarm) | Failed; Indeterminate | S(Pending); retain terminal Safe and matching cleanup obligation |
| Cleanup(Release) | Done | Suspended if latched, otherwise Idle; emit saved result |
| Cleanup(Release) | Failed | S(Pending); retain terminal Safe where present; no new generation |
| Cleanup(Release) | Indeterminate | Ambiguous; Pending; retain terminal Safe; reacquire/fence before cleanup |

A write succeeds only upon confirmation of the exact committed revision and current authority. Unexpected successful payloads are malformed matching replies and follow the indeterminate rule. Exhaustion is checked before constructing a budget-consuming write.

Restoration eligibility is exactly: nonterminal transaction, current authority and host, authoritative hidden observation, no retained/detected supersession, no unsettled work, and either uninterrupted monitoring from before an owned dispatched hide with positive settlement, or a valid committed hide certificate under ADR-032. `Change(Visible)` rechecks eligibility immediately before dispatch.

Rearm preparation preserves `restoreOnly=true`. Its HidePending commit writes false; only matching success with valid gates/token clears the in-memory latch. Interruptions destroy consent tokens. Restart never reconstructs consent or resumes preparation.

### Bounds and retries

All parameters require measured evidence and review. No numeric values are selected. Missing/invalid policy disables production hiding.

- `B_RECORD`, `B_DEPTH`, `B_IPC`, `B_QUEUE`, `B_WAITERS`, `B_PROOF` bound parsing, allocation and retained data.
- `T_ACQUIRE`, `T_READ`, `T_WRITE`, `T_GUARD_READY`, `T_GUARD_HEALTH`, `T_NATIVE`, `T_IPC`, `T_FENCE`, `T_DISARM`, `T_RELEASE` bound effects and health detection.
- `N_ACQUIRE` and `T_ACQUIRE_EPISODE` bound acquisition per OS process incarnation. Consume an attempt before every acquire dispatch. Retries write no record; automatic respawn cannot replenish this allowance.
- `N_RECOVERY_READ` and `N_RESTORE` bound each durable retry episode across actors/restarts. Commit episode and lifetime counter consumption before every recovery read or restore attempt.
- Initial metadata creates one Initial episode. Automatic recovery, restart, replacement Guard and rearm retain its current counters. A fresh hide generation carries the current episode forward.
- A genuine explicit user restore, including a directly user-invoked standalone `--restore-desktop`, may open one new Explicit episode after fresh coordinated authority, settled work and authenticated current ownership-evidence classification. Visibility commands still require a separately budgeted fresh observation. No visibility command is authorized merely by opening an episode.
- Explicit episode authorization uses a fresh authenticated user-action token and monotonic sequence above `lastExplicitRestoreSequence`. Commit the sequence, unique episode ID and new bounded counters together through `Write(Episode)`. A replay, duplicate waiter, automatic relaunch, non-Explicit shutdown cause or resumed request cannot open another episode.
- A fresh explicit request after a prior episode finishes or exhausts may open another bounded episode. It preserves lifetime totals, certificates, supersession and suspension. It never manufactures ownership, reopens Safe or retries an indeterminate operation ID. Uncertain episode commits require fencing and reconciliation before use.
- `T_RECOVERY_TOTAL` is process-local from first recovery entry using supplied monotonic elapsed values. It never resets within that process, including on explicit requests. New processes do not replenish durable episode limits.
- `D_RETRY(kind,attempt)` defines deterministic bounded delays. Clock rollback cannot increase remaining budget. Exhaustion retains evidence and returns Pending.
- Explorer absence uses bounded recovery reads. Positive settlement permits a new separately budgeted restore only with fresh eligibility. An unsettled command blocks commands even in a new episode.

## Platform Matrix

| Target | Applies? | Required evidence |
|---|---|---|
| Windows 11 23H2/24H2+ | Target | Exact supported builds in interactive checkpointed VMs |
| Windows 10 22H2 | Target release minimum | Exact build and complete recovery/race matrix |
| DPI and monitors | Indirect | Record DPI/count, including mixed-DPI two-monitor host revalidation |
| macOS/Linux/mobile | No | Phase 6; infer no mechanism |

## Security and Privacy

No network, elevation, automated registry mutation, Explorer termination or unrelated process control. Registry experiments require explicit owner approval and a documented restore path. No user file moves, renames or deletions without confirmation.

IPC accepts fixed bounded commands and validates principal, session incarnation, actual peer identity, generation, challenge, operation and request sequence. Reject arbitrary paths/PIDs, spoofed readiness, stale disarm and replayed acknowledgements. Guard waits on the validated intended process, never PID text alone.

ACLs do not isolate a fully compromised same-user process. Logs redact paths/identifiers; recovery stores no input history or file contents. Data deletion retains the only pending recovery evidence. Uninstall attempts every established owned restoration and reports unresolved manual recovery without claiming success.

## UI

R0-05 implements no UI. Typed results expose suspension, retry exhaustion and manual-action requirements. Standalone Pending has a distinct report/exit status from Restored/AlreadySafe. Explicit episode authorization must be distinguishable from automatic recovery.

Manual guidance directs users to Windows desktop context menu, View, Show desktop icons, then request authoritative verification. A user assertion alone never authorizes a toggle. Ambiguous recovery can leave icons hidden until manual restoration; callers must state this consequence.

## Tests

### Numbered Core invariants

1. Identical state/events produce identical ordered outputs; Core performs no I/O.
2. Every hide requires committed intent/operation, authenticated current Guard and execution-time state/health validation.
3. Pre-hidden icons are never claimed; failed readiness issues no hide and preserves pre-transaction visibility.
4. Unknown state, invalid authority, supersession or unsettled work never authorizes a command.
5. Every acquisition honors suspension; only the specified explicit rearm commit clears it.
6. Every recovery interruption replaces purpose with Restore, retains waiters and invalidates hide/rearm tokens before effects, subject to terminal Safe precedence.
7. Duplicate restore attaches without another effect or episode; fencing coalesces interruptions.
8. Live/uncertain-owner CLI requests use LiveRequest; timeout never authorizes takeover; duplicate startup never restores.
9. Every native dispatch requires read-before/read-after; indeterminate IDs are never replayed.
10. Positive fencing settlement permits a newly identified restore only with fresh eligibility and committed budget consumption.
11. After a monitoring gap, only committed verified-hide evidence qualifies under ADR-032.
12. Every supersession path latches and revokes ownership even if persistence fails; budget/episode writes retain supersession and visible-only resolution.
13. Safe requires authoritative visible verification and settlement; disarm follows durable Safe.
14. Once Safe commits, no event or cleanup failure permits another visibility command for that transaction; its historical hide certificate is permanently ineligible.
15. Rearm requires a fresh visible baseline authenticated to the new reserved binding before Prepared; old certificates cannot be rebound.
16. Config/log failures cannot block authorized recovery; uncertain recovery writes require reconciliation.
17. Every parsed record/phase has deterministic validation and restart outcomes; unsupported evidence remains read-only.
18. Acquisition retries write nothing; automatic recovery never replenishes durable budgets; elapsed recovery limits remain explicitly process-local.
19. Explicit episode opening requires fresh authenticated user action, authority/evidence and a committed new sequence; it preserves evidence and lifetime totals.
20. Explorer absence/exhaustion never produces success; stale replies and unlisted pairs emit no effects.
21. Every effect accepts success, definite failure and indeterminate replies, including synthesized deadline outcomes.
22. Every exit initiates bounded authorized recovery; Pending preserves responsibility and recovery evidence without reporting completion.

Required named suites: `transition_matrix_all_pairs`, `effect_reply_and_deadline_matrix`, `crash_at_every_effect_boundary`, `persisted_suspension_all_entry_purposes`, `explicit_rearm_only_clear_transition`, `rearm_fresh_baseline_binding`, `acquisition_role_purpose_matrix`, `uncertain_owner_cli_route`, `interruption_replaces_purpose_keeps_waiter`, `duplicate_restore_precedence`, `acquire_retry_without_authority`, `signal_owner_reply_matrix`, `version_one_validation_matrix`, `phase_field_certificate_matrix`, `restart_all_phases`, `adr032_verified_hide_gap_restore`, `unverified_gap_requires_manual_recovery`, `continuous_guard_uncommitted_hide_restore`, `settled_indeterminate_hide_allows_new_restore`, `indeterminate_operation_never_replayed`, `supersession_reply_latches_before_write`, `supersession_survives_budget_and_episode_writes`, `shutdown_paths_translate_to_restore`, `guard_loss_each_hide_boundary`, `retry_budget_survives_restart`, `explicit_episode_commit_and_replay_matrix`, `automatic_recovery_never_replenishes_budget`, `process_local_elapsed_budget`, `safe_commit_before_disarm`, `safe_terminal_after_cleanup_failure_all_events`.

Adapter tests inject failures at every write, handshake, read, dispatch, settlement and cleanup boundary. CLI tests prove renderer/config independence. Mutation tests removing safety gates must fail named tests. These are required tests, not reported passes.

Native tests record build, DPI, monitors, races and recovery latency across every exit mode, Guard loss, both-process loss, reboot, concurrent actors, menu changes and Explorer absence/replacement. Explorer restart is manual in a checkpointed VM/test account.

## What Was Not Determined

Required native evidence includes authoritative state reading, dynamic host discovery, bounded dispatch/readback, observable user changes, command settlement, coordinator fencing, durable writes, process identity/handle transfer, authenticated IPC, independent Guard operation, explicit-action replay protection and all parameter values.

`WM_COMMAND 0x7402` is an undocumented candidate toggle. `IsWindowVisible` alone is insufficient. `ShowWindow` and registry alternatives are unproven. No mechanism is selected.

Residual windows include user action between read and command, undetectable external changes, Guard death immediately after health validation, both actors dying, power loss before verification/supersession commits, storage corruption, Explorer disappearance and scheduler stalls. No protocol guarantees immediate visibility through those failures.

ADR-032 accepts showing icons after a monitoring gap when a prior committed Drape hide exists, even if an unobserved user hide occurred during that gap. Recorded supersession and committed Safe still prohibit restoration. An uncommitted hide after a gap remains ambiguous.

If settlement, reading, identity, coordination, durability or bounded recovery cannot be established, hiding remains disabled. Inadequate external-change detection or race evidence also keeps hiding disabled pending owner review. Record negative spike results; never weaken safety gates to obtain acceptance. This specification establishes no implementation or native-test results.

## Open Questions

These are proposed backlog entries, not accepted deviations. ADR-032 settles verified-hide monitoring-gap policy. Reading CLAUDE.md as a whole resolves the Guard/exit wording; it is not an open question. Explicit user-initiated bounded episodes resolve supplemental retry-budget policy.

| Proposed entry | Owner question | Fail-safe default and user-visible consequence |
|---|---|---|
| Q-24 | Which explicit product action supplies resolved-state rearm consent? | No timed/automatic rearm; hiding stays suspended until authenticated explicit consent and a fresh baseline succeed |
| Q-25 | Are measured races, detection limits, recovery latency and bounds acceptable? | Production hiding stays disabled until evidence and policy approval |
| Q-26 | Approve version 1 namespace and legacy compatibility treatment | Preserve unsupported/legacy bytes read-only; manual recovery may be necessary |

Native mechanism selection remains R0-05 / existing Q-21 and requires spike evidence.

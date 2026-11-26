# R0-05 specification revision rounds

Date: 2026-11-26. Coordinator: Claude Code session. Authors and reviewers: separate read-only `codex exec` sessions with the governing documents inlined. Status: the [revised spec](../specs/R0-05-native-icon-recovery.md) is a draft, not accepted. No code or native experiment exists for it.

## Rounds

| Draft | Safety review | Core implementability review |
|---|---|---|
| 1 | REJECT, 2 findings | REJECT, 8 findings |
| 2 | ACCEPT | REJECT, 7 findings |
| 3 | ACCEPT | REJECT, 4 findings, one of them a safety defect |
| 4 | ACCEPT | REJECT, 5 findings |

The safety defect in draft 3: after a restoration committed `Safe`, a failed Guard disarm could re-enter recovery and use the old hide certificate to undo a later user choice. Draft 4 makes `Safe` terminal for visibility. The safety reviewer had not reported it.

## Decisions applied

- Owner, [ADR-032](../DECISIONS.md): after a monitoring gap, restore only a hide Drape had verified.
- Coordinator, within existing documents: no atomic native operation is required, because the task card asks to record residual windows; uninstall wording in the Security document was aligned with ADR-032.
- Coordinator, **not an owner decision**: an explicit user restore opens one new bounded retry episode, so that exhausted retries cannot strand a desktop permanently. Automatic recovery never replenishes budgets. The owner may overrule this.

The coordinator stopped after the fourth round as announced, rather than iterate without a bound.

## Open findings on draft 4

Remaining or new blockers, ranked:

1. **Rearm can bypass an outstanding terminal cleanup obligation.**  
   **Location:** Transition rows `Cleanup(Release) + Failed` and `Suspended + Fresh App Rearm`; acquisition mapping `Safe / Rearm`.  
   Failed release enters `Suspended`, retains `Safe`, and explicitly permits “no new generation.” Nevertheless, that state satisfies the rearm row when the failed effect is settled. Acquisition can then reserve and commit a new generation. The table neither gates rearm on completed cleanup nor defines how the old cleanup obligation survives replacement of its record.  
   **Smallest change:** require successful matching disarm and authority release before accepting rearm. While either obligation remains, route requests through terminal cleanup and reject rearm.

2. **Acquisition has no classification for the current actor remaining the responsible owner.**  
   **Location:** Acquisition mapping; `S(x)` shorthand; `Write(Budget) + Failed`; `Suspended/Ambiguous + Restore`.  
   For example, a budget-write failure enters `Suspended` without releasing held authority. A subsequent restore acquires again. A snapshot identifying that same actor as the live responsible owner matches neither “Other validated live responsible owner” nor “No responsible owner.” The required `Acquire + Done` transition is undefined.  
   **Smallest change:** add a same-actor/current-authority classification and its routes, including retained suspension, terminal cleanup, and unfinished recovery. Specify whether reacquisition validates retained authority or replaces it.

3. **Wrong-session records lack a safe acquisition route.**  
   **Location:** Restart classification `Wrong session or unvalidated identity`; acquisition mapping `supported unfinished phase / Any`; dispatch rule 1.  
   An authenticated acquisition reply can contain a structurally valid supported record from another session. That is different from an unauthenticated or wrong-scope event. Restart classification prohibits cross-session action, but acquisition supplies no corresponding branch; its supported-phase row otherwise directs a write.  
   **Smallest change:** classify record scope/session mismatch and unvalidated record identity explicitly before supported-phase routing. Enter `Ambiguous`, preserve the record without writes or process control, and return `Pending` with manual guidance.

4. **`HostLost` has conflicting immediate transitions.**  
   **Location:** Dispatch rules 4, 6 and 9; `Owned + relevant loss`.  
   In `Owned`, with authority and no outstanding effect, rule 6 requires `Write(RestorePending)`. Rule 9 instead requires a bounded wait for acquisition or a recovery read. No precedence or sequencing selects the next state and ordered effects.  
   **Smallest change:** specify that this case first attempts `RestorePending`, then enters `Wait(RestoreRead)`; specify `Wait(Acquire)` when authority is absent. State explicitly how this specialization interacts with rule 6.

5. **Explicit-episode authorization is absent from the closed input contract.**  
   **Location:** External event `Restore(requestId,cause)`; `NextRead`/`Write(Episode)` rules; Bounds and retries, explicit user-action token and sequence requirements.  
   Episode opening requires a fresh authenticated user-action token and a monotonic sequence, but neither is defined in `Restore`, a specified authentication envelope, or a typed effect reply. Consequently, the function cannot determine whether to open an episode or construct its required durable sequence from the declared inputs. `cause=Explicit` alone does not establish the required authorization.  
   **Smallest change:** add optional typed explicit authorization to `Restore`, including the authenticated action token and sequence, and define its validation and preservation through acquisition, fencing and owner mediation. Define absent authorization as ordinary recovery using existing counters.

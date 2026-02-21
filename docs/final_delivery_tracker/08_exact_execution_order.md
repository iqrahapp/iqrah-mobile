# 08 - Exact Execution Order

This is the strict anti-drift queue.  
Agents must execute tickets in this exact order.

## Mandatory Instruction (All AI Agents)

You must follow this file strictly.  
Start from the first unchecked step and proceed sequentially.  
Do not skip, reorder, or parallelize steps unless the product owner explicitly approves it.

## Completion Tracking

Use the checkbox status in this file as the single source of truth:
- `[ ]` not done
- `[-]` in progress (optional, only one at a time)
- `[x]` done

When completing a step:
1. update its checkbox in this file,
2. add completion evidence in the active PR/summary using the closure contract from `01_execution_protocol.md`.

## Hard Rules

1. Do not start step `N+1` before step `N` is marked complete with evidence.
2. Do not skip, reorder, or parallelize steps unless product owner explicitly authorizes it.
3. For each step, update tracker status and include closure contract evidence from `01_execution_protocol.md`.

## Step Queue

- [ ] `01` `[MOB]` `C-001`
- [ ] `02` `[MOB]` `C-002`
- [ ] `03` `[BOTH]` `Q-001`
- [ ] `04` `[MOB]` `Q-013`
- [ ] `05` `[MOB]` `Q-014`
- [ ] `06` `[MOB]` `C-003`
- [ ] `07` `[MOB]` `C-004`
- [ ] `08` `[MOB]` `C-005`
- [ ] `09` `[MOB]` `C-006`
- [ ] `10` `[MOB]` `C-007`
- [ ] `11` `[MOB]` `C-008`
- [ ] `12` `[MOB]` `C-009`
- [ ] `13` `[MOB]` `C-010`
- [ ] `14` `[MOB]` `C-012`
- [ ] `15` `[MOB]` `C-013`
- [ ] `16` `[MOB]` `C-011`

- [ ] `17` `[MOB]` `F-001`
- [ ] `18` `[MOB]` `F-002`
- [ ] `19` `[MOB]` `F-003`
- [ ] `20` `[MOB]` `F-004`
- [ ] `21` `[MOB]` `F-008`
- [ ] `22` `[MOB]` `F-010`
- [ ] `23` `[MOB]` `F-005`
- [ ] `24` `[MOB]` `F-006`
- [ ] `25` `[MOB]` `F-007`
- [ ] `26` `[MOB]` `F-011`
- [ ] `27` `[MOB]` `F-012`
- [ ] `28` `[MOB]` `F-009`
- [ ] `29` `[MOB]` `F-013`
- [ ] `30` `[MOB]` `F-014`

- [ ] `31` `[BOTH]` `D-000`
- [ ] `32` `[BE]` `D-001`
- [ ] `33` `[BE]` `D-002`
- [ ] `34` `[BE]` `D-003`
- [ ] `35` `[BE]` `D-004`
- [ ] `36` `[BE]` `D-005`
- [ ] `37` `[BE]` `D-006`
- [ ] `38` `[BE]` `D-007`
- [ ] `39` `[BE]` `D-008`
- [ ] `40` `[BE]` `D-009`
- [ ] `41` `[BE]` `D-010`

- [ ] `42` `[MOB]` `D-011`
- [ ] `43` `[MOB]` `D-012`
- [ ] `44` `[MOB]` `D-013`
- [ ] `45` `[MOB]` `D-014`
- [ ] `46` `[MOB]` `D-015`
- [ ] `47` `[MOB]` `D-016`
- [ ] `48` `[MOB]` `D-017`

- [ ] `49` `[BOTH]` `D-018`
- [ ] `50` `[BOTH]` `D-019`
- [ ] `51` `[BOTH]` `D-020`
- [ ] `52` `[MOB]` `D-021`
- [ ] `53` `[BOTH]` `D-022`
- [ ] `54` `[MOB]` `Q-006`

- [ ] `55` `[MOB]` `Q-002`
- [ ] `56` `[MOB]` `Q-003`
- [ ] `57` `[BOTH]` `Q-004`
- [ ] `58` `[BOTH]` `Q-005`
- [ ] `59` `[BE]` `Q-007`
- [ ] `60` `[BOTH]` `Q-008`
- [ ] `61` `[BOTH]` `Q-009`
- [ ] `62` `[BOTH]` `Q-010`
- [ ] `63` `[BOTH]` `Q-011`
- [ ] `64` `[BOTH]` `Q-012`

## Mandatory Checkpoints

After step 16:
1. `Gate A` must pass.

After step 30:
1. UI quality checkpoint (reader + word detail + session UX functional).

After step 41:
1. backend release registry smoke test must pass.

After step 48:
1. bootstrap/update/rollback smoke tests must pass.

After step 54:
1. `Gate C` must pass.

After step 64:
1. Gates `A` to `E` must pass.
2. Production readiness signoff may be issued.

## Allowed Exception Path

Only security hotfixes may interrupt this order, and only with explicit product owner approval.  
Any interruption must be logged with reason and reinsertion point.

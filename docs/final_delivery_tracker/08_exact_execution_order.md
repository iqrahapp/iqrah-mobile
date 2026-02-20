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

- [ ] `01` `C-001`
- [ ] `02` `C-002`
- [ ] `03` `Q-001`
- [ ] `04` `Q-013`
- [ ] `05` `Q-014`
- [ ] `06` `C-003`
- [ ] `07` `C-004`
- [ ] `08` `C-005`
- [ ] `09` `C-006`
- [ ] `10` `C-007`
- [ ] `11` `C-008`
- [ ] `12` `C-009`
- [ ] `13` `C-010`
- [ ] `14` `C-012`
- [ ] `15` `C-013`
- [ ] `16` `C-011`

- [ ] `17` `F-001`
- [ ] `18` `F-002`
- [ ] `19` `F-003`
- [ ] `20` `F-004`
- [ ] `21` `F-008`
- [ ] `22` `F-010`
- [ ] `23` `F-005`
- [ ] `24` `F-006`
- [ ] `25` `F-007`
- [ ] `26` `F-011`
- [ ] `27` `F-012`
- [ ] `28` `F-009`
- [ ] `29` `F-013`
- [ ] `30` `F-014`

- [ ] `31` `D-000`
- [ ] `32` `D-001`
- [ ] `33` `D-002`
- [ ] `34` `D-003`
- [ ] `35` `D-004`
- [ ] `36` `D-005`
- [ ] `37` `D-006`
- [ ] `38` `D-007`
- [ ] `39` `D-008`
- [ ] `40` `D-009`
- [ ] `41` `D-010`

- [ ] `42` `D-011`
- [ ] `43` `D-012`
- [ ] `44` `D-013`
- [ ] `45` `D-014`
- [ ] `46` `D-015`
- [ ] `47` `D-016`
- [ ] `48` `D-017`

- [ ] `49` `D-018`
- [ ] `50` `D-019`
- [ ] `51` `D-020`
- [ ] `52` `D-021`
- [ ] `53` `D-022`
- [ ] `54` `Q-006`

- [ ] `55` `Q-002`
- [ ] `56` `Q-003`
- [ ] `57` `Q-004`
- [ ] `58` `Q-005`
- [ ] `59` `Q-007`
- [ ] `60` `Q-008`
- [ ] `61` `Q-009`
- [ ] `62` `Q-010`
- [ ] `63` `Q-011`
- [ ] `64` `Q-012`

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

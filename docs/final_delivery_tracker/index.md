# Final Delivery Tracker

Status: master execution system  
Date: 2026-02-20  
Scope: complete delivery to production readiness

This is the canonical execution layer for:
1. `docs/ai_blueprint-3`
2. `docs/data_platform_blueprint`

Use this folder to run all implementation work to completion.

## Read Order

1. `docs/final_delivery_tracker/01_execution_protocol.md`
2. `docs/final_delivery_tracker/02_master_milestones.md`
3. `docs/final_delivery_tracker/03_ticket_board_core_learning_and_scheduler.md`
4. `docs/final_delivery_tracker/04_ticket_board_data_backend_sync.md`
5. `docs/final_delivery_tracker/05_ticket_board_frontend_product_and_design.md`
6. `docs/final_delivery_tracker/06_ticket_board_quality_security_ops.md`
7. `docs/final_delivery_tracker/07_release_gates_and_definition_of_done.md`
8. `docs/final_delivery_tracker/08_exact_execution_order.md`

## Working Rule

No agent should execute ad-hoc work outside ticket IDs in this folder unless a new ticket is added first.

## Mandatory Agent Instruction

All AI agents must follow `docs/final_delivery_tracker/08_exact_execution_order.md` strictly.

Execution rule:
1. Start from the first unchecked step.
2. Proceed sequentially.
3. Do not skip, reorder, or parallelize without explicit product-owner approval.

Tracking rule:
1. Mark progress directly in `08_exact_execution_order.md` checkboxes:
   Use `[ ]` for not done, `[-]` for in progress (one step at a time), and `[x]` for done.
2. Include completion evidence per step as defined in `01_execution_protocol.md`.

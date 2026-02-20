# 01 - Execution Protocol

## Status Key

- `[ ]` not started
- `[-]` in progress
- `[x]` complete
- `[!]` blocked

## Ticket Closure Contract

A ticket is `complete` only if all are provided:
1. changed file paths,
2. tests executed + results,
3. API/schema diffs (if any),
4. residual risks,
5. linked follow-up ticket for any deferred work.

## Agent Workflow

1. Pick highest-priority unblocked ticket.
   - Priority is fixed by `08_exact_execution_order.md`.
2. Confirm dependencies are complete.
3. Implement only ticket scope.
4. Run required validations.
5. Update tracker status and evidence.

## Anti-Drift Rules

1. Runtime claims must be tied to current code paths.
2. No feature is "done" without tests.
3. No UI feature is "done" without mobile usability checks.
4. No backend contract is "done" without OpenAPI update and check.
5. No data release change is "done" without checksum/activation safety.

## Global Commands

Mobile repo:
1. `flutter test`
2. `flutter test integration_test`

Backend repo (`../iqrah-backend`):
1. `just fmt-check`
2. `just lint`
3. `just test`
4. `just coverage-ci`
5. `just spec-check`

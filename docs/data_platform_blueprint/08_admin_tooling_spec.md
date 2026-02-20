# 08 - Admin Tooling Spec

Goal: make artifact and release operations safe, scriptable, and auditable.

## Recommended First Step: CLI

Build a lightweight admin CLI before a web dashboard.

Suggested commands:
1. `release create --version <v> --notes <text>`
2. `pack upload --package-id <id> --version <v> --file <path>`
3. `release attach --release <id> --package <id> --role <role> --required`
4. `release validate --release <id>`
5. `release publish --release <id>`
6. `release deprecate --release <id>`
7. `release latest`

## Why CLI First

1. Faster to implement.
2. Easier to audit in shell history and CI logs.
3. Works for both local and hosted backend operations.

## Required UX Guarantees

1. Clear dry-run mode for publish.
2. Explicit confirmation on publish/deprecate.
3. Structured JSON output for automation.
4. Non-zero exit codes on validation failures.

## Later: Web Admin Panel

Phase 2 optional dashboard features:
1. Upload progress UI.
2. Release artifact matrix view.
3. Validation report view.
4. Publish/deprecate controls.
5. Audit timeline.

## Security Controls

1. Admin API key must never be hardcoded.
2. Use environment-injected credentials for CLI.
3. Log all publish/deprecate actions with actor identity.
4. Rate-limit upload endpoints.

# Data Platform Blueprint

Status: proposed architecture and execution plan  
Date: 2026-02-20  
Scope: testability hardening + backend-hosted content/data artifacts + repo cleanup

This folder is the canonical plan for fixing data/distribution chaos before further expanding `docs/ai_blueprint-3`.

For full cross-repo execution (including product/UI and production launch), use:
- `docs/final_delivery_tracker/index.md`

## Why This Exists

Current pain points:
1. Heavy generated data mixed into the mobile repo.
2. Manual QA needed to detect regressions.
3. No unified release pipeline for KG/content/morphology/audio/translation artifacts.
4. Repeated local file loss and AI-generated file sprawl.

## Read Order

1. `docs/data_platform_blueprint/01_problem_and_principles.md`
2. `docs/data_platform_blueprint/02_testability_strategy.md`
3. `docs/data_platform_blueprint/03_backend_distribution_architecture.md`
4. `docs/data_platform_blueprint/04_release_model_and_api_contracts.md`
5. `docs/data_platform_blueprint/05_mobile_runtime_changes.md`
6. `docs/data_platform_blueprint/06_backend_implementation_plan.md`
7. `docs/data_platform_blueprint/07_migration_and_cutover.md`
8. `docs/data_platform_blueprint/08_admin_tooling_spec.md`
9. `docs/data_platform_blueprint/09_risks_and_decisions.md`
10. `docs/data_platform_blueprint/10_definition_of_done.md`
11. `docs/data_platform_blueprint/11_execution_tracker.md`
12. `docs/data_platform_blueprint/12_tickets_backend.md`
13. `docs/data_platform_blueprint/13_tickets_mobile.md`

## Key Decision

Backend (`../iqrah-backend`) becomes the authoritative content distribution plane.  
Mobile remains local-first for runtime scheduling/learning and only uses backend for:
1. artifact discovery,
2. artifact download/update,
3. sync/auth.

# AI Blueprint v3

Audit date: 2026-02-20
Scope audited:
- `research_and_dev/iqrah-knowledge-graph2`
- `rust/` (especially `iqrah-core`, `iqrah-api`, `iqrah-storage`, `iqrah-gen`, `iqrah-iss`)
- `/home/shared/ws/iqrah/iqrah-backend`
- Flutter app wiring in `lib/`

This blueprint is intentionally opinionated and code-anchored. It is written to be consumed by another AI assistant that will help drive the next project steps.

## Prerequisite

Before executing large data/distribution changes, read:
- `docs/data_platform_blueprint/index.md`
- `docs/final_delivery_tracker/index.md`

These folders define the canonical implementation and delivery path.

## Executive Truth (Current State)

1. The mobile app currently schedules from `user_memory_states` due items only, using a simple priority formula in `SessionService::get_due_items`.
2. The advanced scheduler (`scheduler_v2`) exists and is well developed, but is not the primary path used by app sessions today.
3. The CBOR import path currently parses records but does not insert nodes/edges into DB; app startup mostly relies on bundled `rust/content.db`.
4. The shipped `content.db` graph is structurally simpler than the Python R&D graph and has schema/data shape drift.
5. Backend is focused on auth/packs/sync. It does not currently own scheduling logic.

## How To Read This Folder

Start in this order:

1. `docs/ai_blueprint-3/01_project_map_and_truth_sources.md`
2. `docs/ai_blueprint-3/02_kg_generation_python_pipeline.md`
3. `docs/ai_blueprint-3/03_runtime_graph_data_in_rust_content_db.md`
4. `docs/ai_blueprint-3/04_scheduler_paths_current_vs_intended.md`
5. `docs/ai_blueprint-3/05_exercise_system_and_memorization_fit.md`
6. `docs/ai_blueprint-3/06_backend_scope_and_sync_contract.md`
7. `docs/ai_blueprint-3/07_gap_analysis_drift_and_risk_register.md`
8. `docs/ai_blueprint-3/08_simplification_roadmap_for_product_goals.md`
9. `docs/ai_blueprint-3/09_claims_vs_runtime_reality.md`
10. `docs/ai_blueprint-3/10_flutter_frontend_reality_and_gaps.md`
11. `docs/ai_blueprint-3/11_memorization_domain_constraints.md`
12. `docs/ai_blueprint-3/12_reality_check_checklist.md`
13. `docs/ai_blueprint-3/13_exercise_catalog_scored_and_runtime_wiring.md`
14. `docs/ai_blueprint-3/14_execution_backlog_file_level.md`
15. `docs/ai_blueprint-3/15_canonical_usage_and_scoring.md`

## What This Fixes vs Previous Blueprint

This version distinguishes clearly between:
- what exists in code,
- what is currently wired at runtime,
- what exists only in ISS or R&D flows,
- what is likely overkill right now,
- what is missing for practical Quran memorization outcomes.

It now also includes:
- complete exercise catalog scoring with runtime wiring status,
- a file-level execution backlog with acceptance criteria,
- a canonical "single-source" usage mode for future AI assistants.

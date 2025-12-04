# Production-Ready Tasks Status

| Task | Status | Priority | TL;DR Summary |
| :--- | :--- | :--- | :--- |
| **Phase 1: Critical Foundation** | | | |
| 1.1 Architecture Doc | ✅ Done | P0 | Documented architecture & node ID contracts. |
| 1.2 Schema Versioning | ✅ Done | P0 | Added `schema_version` table and validation logic. |
| 1.3 Node ID Utility | ✅ Done | P0 | Implemented `node_id` module for consistent ID handling. |
| 1.4 Repo Refactoring | ✅ Done | P0 | Refactored repositories to use new `node_id` module. |
| 1.5 Node ID Validation | ✅ Done | P0 | Added Python validation for node ID stability. |
| 1.6 Schema v2.1 | ✅ Done | P0 | Redesigned Content DB schema (no hardcoded text) & separated test data. |
| 1.7 Consolidate Migrations | ✅ Done | P0 | Consolidated User DB migrations into single clean file `20241126000001_user_schema.sql`. |
| **Phase 2: Knowledge Axis** | | | |
| 2.1 Full Knowledge Graph | ✅ Done | P0 | Generated migration with 3 core axes: memorization (10.6k nodes + sequential edges), translation (12.3k nodes), meaning (697 ROOT:meaning nodes) + PageRank scoring. Tafsir/Tajweed deferred. |
| 2.2 Verify End-to-End | ✅ Done | P0 | Implemented ROOT/LEMMA Rust decoders, created 8 integration tests (all axes, sequential edges, cross-axis propagation), CLI verification script. All tests passing, database verified (36.4k nodes, 177k edges). |
| 2.3 Tajweed Exercises | 🔴 Todo | P0 | Implement specific exercise logic for Tajweed axis. |
| 2.4 Cross-Axis Verify | ✅ Done | P0 | Verify that progress in one axis (e.g. translation) propagates to others. |
| **Phase 3: Data Integrity** | | | |
| 3.1 Transaction Wrapping | 🔴 Todo | P1 | Ensure review recording is atomic. |
| 3.2 Referential Integrity | 🔴 Todo | P1 | Add validation to ensure User DB references valid Content DB nodes. |
| 3.3 Graph Update Mech | 🔴 Todo | P1 | System to safely update graph structure without breaking user progress. |
| **Phase 4: Packages** | | | |
| 4.1 Translator UI | 🔴 Todo | P1 | UI for selecting active translators. |
| 4.2 Package Design | 🔴 Todo | P1 | Design for downloading content packages. |
| **Phase 5: Hardening** | | | |
| 5.1 Error Handling | 🔴 Todo | P2 | Audit and improve error handling/logging. |

## Preferred Execution Order

1.  **Task 2.1**: Generate the data. Everything else in Phase 2 depends on this.
2.  **Task 2.2**: Verify it works.
3.  **Task 2.3**: Add Tajweed specific features.
4.  **Task 2.4**: Verify advanced propagation features.
5.  **Task 3.1 & 3.2**: Improve data safety (can be done in parallel).
6.  **Task 3.3**: Handle updates.
7.  **Task 4.1 & 4.2**: User-facing features.
8.  **Task 5.1**: Final polish.

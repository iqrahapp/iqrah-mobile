# 09 - Claims vs Runtime Reality Matrix

Use this as a fast sanity check when an AI assistant makes architecture claims.

| Claim | Reality (audit) | Source anchor |
|---|---|---|
| "App imports graph from CBOR on startup" | Usually false. App uses bundled `content.db`; CBOR import is fallback and current importer does not persist parsed nodes/edges. | `lib/main.dart`, `rust/crates/iqrah-core/src/cbor_import.rs` |
| "Session scheduling uses scheduler_v2" | False for primary mobile path. App sessions use `SessionService::get_due_items`. | `rust/crates/iqrah-api/src/api.rs`, `rust/crates/iqrah-core/src/services/session_service.rs` |
| "Goal ID drives session item selection" | False in current mobile flow. `goal_id` is persisted in session record, not used to fetch candidates. | `rust/crates/iqrah-api/src/api.rs` |
| "Graph metadata fully drives priority" | False in active path. Priority uses hardcoded node-type importance constants. | `rust/crates/iqrah-core/src/services/session_service.rs` |
| "Advanced exercise set is fully used in sessions" | False. Scheduled flow uses a narrow default subset; many exercise generators are mainly sandbox/preview exposed. | `rust/crates/iqrah-core/src/exercises/service.rs`, `rust/crates/iqrah-api/src/api.rs` |
| "Production graph equals Python R&D graph" | False. Runtime DB is simpler and structurally different. | `knowledge-graph.stats.json`, `rust/content.db` counts |
| "Backend handles scheduling" | False. Backend handles auth/packs/sync/admin; scheduling remains local in app Rust core. | `/home/shared/ws/iqrah/iqrah-backend/openapi.json`, backend `README.md` |
| "Propagation guarantees global progress impact from first item" | Not fully. Propagation only updates targets that already have memory states. | `rust/crates/iqrah-core/src/services/learning_service.rs` |

## Practical Rule For Future AI Agents

When proposing changes, classify every statement as one of:
1. Implemented and active in mobile runtime.
2. Implemented but not active in mobile runtime.
3. R&D/simulation-only.
4. Planned/not implemented.

This single classification step prevents most superficial or misleading planning outputs.

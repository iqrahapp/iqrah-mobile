# 01 - Project Map And Truth Sources

## 1) Repositories And Responsibilities

### Mobile monorepo (`iqrah-mobile`)
- Flutter UI and state: `lib/`
- Rust core and FRB API: `rust/crates/iqrah-core`, `rust/crates/iqrah-api`
- Local storage layer: `rust/crates/iqrah-storage`
- Offline graph/content generator: `rust/crates/iqrah-gen`
- Simulation/evaluation engine: `rust/crates/iqrah-iss`
- R&D Python KG pipeline: `research_and_dev/iqrah-knowledge-graph2`

### Backend repo (`/home/shared/ws/iqrah/iqrah-backend`)
- HTTP API for auth, packs, sync, admin
- PostgreSQL repositories
- OpenAPI contract in `openapi.json`

## 2) Runtime Data Flow (What Actually Runs In App)

1. Flutter startup (`lib/main.dart`) ensures `content.db` exists by copying `rust/content.db` asset.
2. Flutter calls Rust `setup_database(...)` through FRB (`lib/rust_bridge/api.dart`, `rust/crates/iqrah-api/src/api.rs`).
3. Rust initializes content/user SQLite DBs, repositories, and services.
4. Session/exercise requests go through `iqrah-api` -> `SessionService` + `ExerciseService`.
5. Reviews write FSRS + energy updates to user DB (`user_memory_states`) and optional propagation to connected nodes.

Important: CBOR is loaded only as a fallback when DB looks empty/tiny. In normal app startup, bundled DB is used directly.

## 3) Ground-Truth Files For Behavior

Use these first when answering "what does the app do now":

- Startup and data source:
  - `lib/main.dart`
  - `rust/crates/iqrah-api/src/api.rs` (`setup_database`)
  - `rust/crates/iqrah-core/src/cbor_import.rs`
- Session scheduling path used by mobile:
  - `rust/crates/iqrah-api/src/api.rs` (`get_exercises`, `start_session`)
  - `rust/crates/iqrah-core/src/services/session_service.rs`
- Review updates and propagation:
  - `rust/crates/iqrah-core/src/services/learning_service.rs`
- Exercise routing:
  - `rust/crates/iqrah-core/src/exercises/service.rs`
  - `rust/crates/iqrah-core/src/exercises/generators.rs`

## 4) Layers That Exist But Are Not Primary Mobile Runtime Path

- Advanced scheduler engine:
  - `rust/crates/iqrah-core/src/scheduler_v2/*`
- ISS simulation framework:
  - `rust/crates/iqrah-iss/src/*`
- Python CBOR graph generation pipeline:
  - `research_and_dev/iqrah-knowledge-graph2/src/*`

These are substantial and valuable, but they are not equivalent to "what app users currently get".

## 5) Immediate Architecture Reality

- The codebase has strong R&D and simulation sophistication.
- The shipping app path is currently much simpler.
- This is the core drift that made previous documentation feel superficial or misleading.

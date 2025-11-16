# Sprint 8: Headless Test Server Implementation

**Status**: Ready to Start
**Date**: 2025-11-16
**Branch**: `claude/review-sprint-7-gaps-01XuSDse4hHrARp2hTsczVUo`
**Prerequisites**: ✅ Sprint 7 Complete (Phase 1 & 2)

---

## 🎯 Mission

Implement a **headless test server** to enable automated testing of the Iqrah learning system without requiring Flutter. This will allow comprehensive integration testing, performance benchmarking, and API validation using the Rust backend directly.

---

## 📋 Current Project State

### Architecture Overview

The project uses **clean hexagonal architecture** with these crates:

```
rust/
├── crates/
│   ├── iqrah-core/          # Domain logic, services, CBOR import
│   ├── iqrah-storage/       # SQLx repositories, migrations
│   ├── iqrah-api/           # Flutter bridge API (12 functions)
│   └── iqrah-cli/           # CLI tool (existing, minimal)
└── src/lib.rs               # Re-exports iqrah-api
```

### Database Architecture

**Two-database design**:
- `content.db` (read-only): Nodes, edges, Quran text, translations
- `user.db` (read-write): Memory states, session state, propagation logs, user stats

### Key APIs Available

All APIs are in `rust/crates/iqrah-api/src/api.rs`:

1. **Setup & Initialization**
   - `setup_database(content_db_path, user_db_path, kg_bytes)`
   - `setup_database_in_memory(kg_bytes)`

2. **Exercise Management**
   - `get_exercises(user_id, limit, surah_filter, is_high_yield)`
   - `process_review(user_id, node_id, grade)`

3. **Stats & Dashboard**
   - `get_dashboard_stats(user_id)`
   - `get_debug_stats(user_id)`

4. **Session Management**
   - `get_session_preview(user_id, limit, is_high_yield)`
   - `clear_session()`

5. **Search & Discovery**
   - `search_nodes(query, limit)`
   - `get_available_surahs()` *(TODO stub)*

6. **Utilities**
   - `reseed_database()` *(TODO stub)*

### Test Coverage

- ✅ **24 tests passing** (15 in iqrah-core, 9 in iqrah-storage)
- ✅ Unit tests for LearningService and SessionService
- ✅ Integration tests for repositories
- ❌ No end-to-end API tests yet (Sprint 8 goal!)

### Build Status

```bash
cargo build --release --package iqrah-api  # ✅ Builds successfully
cargo test --workspace                     # ✅ All tests pass
```

---

## 🎯 Sprint 8 Objectives

### Primary Goals

1. **Create Headless Test Server** (`iqrah-test-server` crate)
   - HTTP/REST API exposing all `iqrah-api` functions
   - Support for in-memory database mode (fast testing)
   - Support for file-based database mode (persistence testing)
   - Health check and status endpoints

2. **Implement E2E Test Suite**
   - Full learning session workflow tests
   - CBOR import validation tests
   - Energy propagation verification tests
   - Dashboard stats accuracy tests
   - Performance benchmarks

3. **Developer Experience**
   - Simple CLI to start test server: `cargo run -p iqrah-test-server`
   - REST API documentation (OpenAPI/Swagger)
   - Example test scripts (curl, Python, or HTTP client)
   - CI/CD integration ready

### Success Criteria

- [ ] Test server starts and responds to health check
- [ ] Can import CBOR knowledge graph via API
- [ ] Can create and process a full review session
- [ ] Stats calculations are verified accurate
- [ ] Performance benchmarks establish baselines
- [ ] All tests are automatable (no manual intervention)

---

## 🏗️ Recommended Implementation Plan

### Phase 1: Basic Test Server (2-3 hours)

**Create `rust/crates/iqrah-test-server/`**:

```rust
// Cargo.toml
[package]
name = "iqrah-test-server"
version = "0.1.0"
edition = "2021"

[dependencies]
iqrah-api = { path = "../iqrah-api" }
iqrah-core = { path = "../iqrah-core" }
axum = "0.7"          # Web framework
tokio = { version = "1", features = ["full"] }
serde_json = "1.0"
anyhow = "1.0"
tracing = "0.1"
tracing-subscriber = "0.3"

// src/main.rs
use axum::{Router, routing::{get, post}};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Build routes
    let app = Router::new()
        .route("/health", get(health_check))
        .route("/api/setup", post(setup_database))
        .route("/api/exercises", get(get_exercises))
        .route("/api/review", post(process_review))
        .route("/api/stats", get(get_dashboard_stats));

    // Start server
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await?;
    println!("Test server running on http://127.0.0.1:3000");
    axum::serve(listener, app).await?;

    Ok(())
}
```

**Key Files to Create**:
- `rust/crates/iqrah-test-server/Cargo.toml`
- `rust/crates/iqrah-test-server/src/main.rs`
- `rust/crates/iqrah-test-server/src/handlers.rs` (API route handlers)
- `rust/crates/iqrah-test-server/src/state.rs` (Shared app state)

**Update**:
- `rust/Cargo.toml` - Add iqrah-test-server to workspace members

### Phase 2: E2E Test Suite (2-3 hours)

**Create test scenarios**:

```rust
// rust/crates/iqrah-test-server/tests/e2e_tests.rs

#[tokio::test]
async fn test_full_learning_session() {
    // 1. Start test server (in-memory mode)
    // 2. Import CBOR knowledge graph
    // 3. Get exercises for user
    // 4. Process reviews with different grades
    // 5. Verify stats updated correctly
    // 6. Verify energy propagation occurred
}

#[tokio::test]
async fn test_cbor_import_accuracy() {
    // 1. Import known CBOR file
    // 2. Query specific nodes
    // 3. Verify node count matches expected
    // 4. Verify edge count matches expected
    // 5. Verify metadata is accessible
}

#[tokio::test]
async fn test_session_persistence() {
    // 1. Create session with file-based DB
    // 2. Process some reviews
    // 3. Shutdown server
    // 4. Restart server
    // 5. Verify session state persisted
}
```

**Key Files to Create**:
- `rust/crates/iqrah-test-server/tests/e2e_tests.rs`
- `rust/crates/iqrah-test-server/tests/common/mod.rs` (Test utilities)
- `rust/crates/iqrah-test-server/tests/fixtures/` (Test data)

### Phase 3: Performance & Benchmarks (1-2 hours)

**Create benchmark suite**:

```rust
// rust/crates/iqrah-test-server/benches/api_benchmarks.rs

use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn benchmark_exercise_generation(c: &mut Criterion) {
    c.bench_function("generate 20 exercises", |b| {
        b.iter(|| {
            // Measure time to generate exercises
        });
    });
}

fn benchmark_review_processing(c: &mut Criterion) {
    c.bench_function("process 100 reviews", |b| {
        b.iter(|| {
            // Measure review processing throughput
        });
    });
}

criterion_group!(benches, benchmark_exercise_generation, benchmark_review_processing);
criterion_main!(benches);
```

**Key Files to Create**:
- `rust/crates/iqrah-test-server/benches/api_benchmarks.rs`
- Add `[dev-dependencies] criterion = "0.5"` to Cargo.toml

### Phase 4: Documentation & CI (1 hour)

**Create documentation**:
- `rust/crates/iqrah-test-server/README.md` - How to run the test server
- `rust/crates/iqrah-test-server/API.md` - API endpoint documentation
- Update root `README.md` with Sprint 8 status

**Create example scripts**:
- `scripts/test_server_example.sh` - Bash script demonstrating API usage
- `scripts/run_benchmarks.sh` - Script to run performance benchmarks

**CI Integration**:
- Update `.github/workflows/` to run E2E tests
- Add benchmark comparison to CI (track regressions)

---

## 📚 Essential Context

### Where to Find Things

| What You Need | Where to Find It |
|---------------|------------------|
| API functions | `rust/crates/iqrah-api/src/api.rs` |
| Domain models | `rust/crates/iqrah-core/src/domain/models.rs` |
| Service logic | `rust/crates/iqrah-core/src/services/` |
| Database schema | `rust/crates/iqrah-storage/migrations_*/*.sql` |
| Existing tests | `rust/crates/iqrah-core/src/services/*_tests.rs` |
| CBOR import | `rust/crates/iqrah-core/src/cbor_import.rs` |
| Repository traits | `rust/crates/iqrah-core/src/ports/*.rs` |
| SQLx implementations | `rust/crates/iqrah-storage/src/` |

### Key Design Decisions

1. **Why Headless?**: Flutter bridge is complex and platform-specific. Testing Rust APIs directly is faster, more reliable, and enables CI/CD automation.

2. **Why Axum?**: Lightweight, type-safe, built on Tokio (already in use), excellent developer experience.

3. **Why In-Memory Mode?**: Fast test execution (no disk I/O), parallel test execution safe, easy cleanup.

4. **Why File-Based Mode?**: Tests persistence logic, validates migrations, tests real-world usage patterns.

### Common Pitfalls to Avoid

1. **Database Locking**: SQLite can have issues with concurrent writes. Use in-memory mode for parallel tests.

2. **Global State**: The current `iqrah-api` uses `OnceCell` for global state. Test server should initialize fresh state per test or use request-scoped state.

3. **Async Runtime**: Ensure all tests use `#[tokio::test]` and server runs in Tokio runtime.

4. **CBOR Loading**: Use the test CBOR file from `research_and_dev/iqrah-knowledge-graph2/src/iqrah/` or create a minimal test fixture.

---

## 🧪 Testing Strategy

### Unit Tests (Already Complete)
- ✅ LearningService tests (5 tests)
- ✅ SessionService tests (7 tests)
- ✅ Repository integration tests (9 tests)

### Integration Tests (Sprint 8 Focus)
- [ ] API endpoint tests (test each API function)
- [ ] Error handling tests (invalid inputs, edge cases)
- [ ] Concurrent request tests (race conditions)

### E2E Tests (Sprint 8 Focus)
- [ ] Full user journey (setup → exercises → reviews → stats)
- [ ] Energy propagation end-to-end
- [ ] Session persistence across restarts
- [ ] CBOR import validation

### Performance Tests (Sprint 8 Focus)
- [ ] Baseline performance (exercise generation, review processing)
- [ ] Throughput tests (concurrent users)
- [ ] Memory usage profiling
- [ ] Database query optimization verification

---

## 🚀 Quick Start Commands

```bash
# Navigate to project root
cd /home/user/iqrah-mobile

# Create test server crate
mkdir -p rust/crates/iqrah-test-server/src
mkdir -p rust/crates/iqrah-test-server/tests

# Add to workspace
# Edit rust/Cargo.toml and add "crates/iqrah-test-server" to members

# Create initial Cargo.toml for test server
# See Phase 1 above

# Build test server
cargo build --package iqrah-test-server

# Run test server
cargo run --package iqrah-test-server

# Run E2E tests
cargo test --package iqrah-test-server --test e2e_tests

# Run benchmarks
cargo bench --package iqrah-test-server
```

---

## 📊 Expected Deliverables

1. **Working Test Server**
   - Runs on `http://127.0.0.1:3000`
   - Responds to health check
   - Exposes all `iqrah-api` functions as REST endpoints

2. **E2E Test Suite**
   - At least 10 comprehensive E2E tests
   - Tests cover all critical user journeys
   - All tests passing

3. **Performance Benchmarks**
   - Baseline metrics established for key operations
   - Benchmark suite integrated into project
   - CI can track performance regressions

4. **Documentation**
   - Test server README with usage instructions
   - API documentation (endpoints, request/response formats)
   - Example scripts demonstrating API usage

5. **CI Integration**
   - E2E tests run on every PR
   - Benchmarks run periodically (optional on every PR)
   - Clear pass/fail criteria

---

## 🎓 Learning Resources

### Axum (Web Framework)
- [Axum Documentation](https://docs.rs/axum/latest/axum/)
- [Axum Examples](https://github.com/tokio-rs/axum/tree/main/examples)

### Testing Patterns
- `rust/crates/iqrah-core/src/services/learning_service_tests.rs` - Example of mock repositories
- `rust/crates/iqrah-storage/tests/integration_tests.rs` - Example of SQLx testing

### CBOR Import
- `rust/crates/iqrah-core/src/cbor_import.rs` - Full implementation reference
- Knowledge graph source: `research_and_dev/iqrah-knowledge-graph2/src/iqrah/`

---

## ✅ Definition of Done

Sprint 8 is complete when:

- [ ] Test server crate created and builds successfully
- [ ] All 12 API functions exposed as REST endpoints
- [ ] Health check endpoint responds correctly
- [ ] In-memory database mode works
- [ ] File-based database mode works
- [ ] At least 10 E2E tests passing
- [ ] Full learning session test (setup → exercises → reviews → stats)
- [ ] CBOR import validation test
- [ ] Energy propagation E2E test
- [ ] Performance benchmarks established
- [ ] README and API documentation written
- [ ] Example scripts created (bash/curl or Python)
- [ ] CI workflow updated to run E2E tests
- [ ] All workspace tests still passing (24+ tests)
- [ ] Code committed and pushed to branch

---

## 🔍 Next Agent Instructions

**You are an AI agent tasked with implementing Sprint 8.**

### Step 1: Familiarize Yourself (15 min)
1. Read this document completely
2. Review `rust/crates/iqrah-api/src/api.rs` to understand available APIs
3. Look at existing tests in `rust/crates/iqrah-core/src/services/*_tests.rs`
4. Check the current build status: `cargo build --workspace`

### Step 2: Create Test Server Scaffold (30 min)
1. Create `rust/crates/iqrah-test-server/` directory structure
2. Write `Cargo.toml` with axum dependencies
3. Implement basic `main.rs` with health check endpoint
4. Add to workspace and verify it builds

### Step 3: Implement API Endpoints (2 hours)
1. Create handler functions for each API endpoint
2. Implement request/response serialization
3. Add error handling
4. Test manually with curl

### Step 4: Write E2E Tests (2 hours)
1. Create test utilities (start server, make requests, assertions)
2. Write full learning session test
3. Write CBOR import test
4. Write stats verification test
5. Verify all tests pass

### Step 5: Add Benchmarks (1 hour)
1. Set up Criterion benchmark framework
2. Benchmark exercise generation
3. Benchmark review processing
4. Document baseline results

### Step 6: Documentation & Cleanup (1 hour)
1. Write test server README
2. Document API endpoints
3. Create example scripts
4. Update root README
5. Commit and push all changes

### Tips for Success
- Use `TodoWrite` tool to track your progress through phases
- Run `cargo test --workspace` frequently to ensure nothing breaks
- Refer to existing test patterns in the codebase
- Ask clarifying questions if Sprint 8 requirements are unclear
- Document any architectural decisions you make

---

**Good luck! The foundation is solid—Sprint 8 is about validation and automation.** 🚀

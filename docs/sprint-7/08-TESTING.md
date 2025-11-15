# Step 8: Comprehensive Testing

## Goal
Achieve 80%+ test coverage and ensure all functionality works correctly.

## Testing Layers

```
E2E Tests (Flutter)          ← User workflows
       ↓
Integration Tests (Rust)     ← Database operations
       ↓
Unit Tests (Rust)            ← Business logic
```

## Task 8.1: Unit Tests for iqrah-core

Since `iqrah-core` has no database dependencies, we can test it in complete isolation using mocks.

**File:** `rust/crates/iqrah-core/tests/domain_tests.rs`

```rust
use iqrah_core::{MemoryState, ReviewGrade, NodeType};

#[test]
fn test_memory_state_creation() {
    let state = MemoryState::new_for_node("user1".to_string(), "node1".to_string());

    assert_eq!(state.user_id, "user1");
    assert_eq!(state.node_id, "node1");
    assert_eq!(state.stability, 0.0);
    assert_eq!(state.difficulty, 0.0);
    assert_eq!(state.energy, 0.0);
    assert_eq!(state.review_count, 0);
}

#[test]
fn test_review_grade_from_u8() {
    assert_eq!(ReviewGrade::from(1), ReviewGrade::Again);
    assert_eq!(ReviewGrade::from(2), ReviewGrade::Hard);
    assert_eq!(ReviewGrade::from(3), ReviewGrade::Good);
    assert_eq!(ReviewGrade::from(4), ReviewGrade::Easy);
}

#[test]
fn test_node_type_conversion() {
    let nt = NodeType::from("word_instance".to_string());
    assert_eq!(nt, NodeType::WordInstance);

    let s: String = NodeType::Verse.into();
    assert_eq!(s, "verse");
}
```

Run tests:
```bash
cargo test -p iqrah-core
```

## Task 8.2: Integration Tests for iqrah-storage

Test actual database operations using in-memory SQLite.

**File:** `rust/crates/iqrah-storage/tests/content_repository_tests.rs`

```rust
use iqrah_storage::{init_content_db, SqliteContentRepository};
use iqrah_core::ContentRepository;
use sqlx::Executor;

#[tokio::test]
async fn test_content_repository_get_node() {
    // Create in-memory database
    let pool = init_content_db(":memory:").await.unwrap();

    // Insert test data
    sqlx::query("INSERT INTO nodes VALUES ('node1', 'word_instance', 0)")
        .execute(&pool)
        .await
        .unwrap();

    // Test repository
    let repo = SqliteContentRepository::new(pool);
    let node = repo.get_node("node1").await.unwrap();

    assert!(node.is_some());
    assert_eq!(node.unwrap().id, "node1");
}

#[tokio::test]
async fn test_content_repository_get_metadata() {
    let pool = init_content_db(":memory:").await.unwrap();

    // Insert test data
    sqlx::query("INSERT INTO nodes VALUES ('node1', 'word_instance', 0)")
        .execute(&pool)
        .await
        .unwrap();

    sqlx::query("INSERT INTO node_metadata VALUES ('node1', 'arabic', 'بِسْمِ')")
        .execute(&pool)
        .await
        .unwrap();

    sqlx::query("INSERT INTO node_metadata VALUES ('node1', 'translation', 'In the name')")
        .execute(&pool)
        .await
        .unwrap();

    let repo = SqliteContentRepository::new(pool);

    // Test get single metadata
    let arabic = repo.get_metadata("node1", "arabic").await.unwrap();
    assert_eq!(arabic, Some("بِسْمِ".to_string()));

    // Test get all metadata
    let all = repo.get_all_metadata("node1").await.unwrap();
    assert_eq!(all.len(), 2);
    assert_eq!(all.get("arabic").unwrap(), "بِسْمِ");
    assert_eq!(all.get("translation").unwrap(), "In the name");
}

#[tokio::test]
async fn test_content_repository_get_edges() {
    let pool = init_content_db(":memory:").await.unwrap();

    // Insert nodes
    sqlx::query("INSERT INTO nodes VALUES ('node1', 'word_instance', 0)")
        .execute(&pool)
        .await
        .unwrap();

    sqlx::query("INSERT INTO nodes VALUES ('node2', 'lemma', 0)")
        .execute(&pool)
        .await
        .unwrap();

    // Insert edge
    sqlx::query("INSERT INTO edges VALUES ('node1', 'node2', 1, 0, 0.5, 0.0)")
        .execute(&pool)
        .await
        .unwrap();

    let repo = SqliteContentRepository::new(pool);
    let edges = repo.get_edges_from("node1").await.unwrap();

    assert_eq!(edges.len(), 1);
    assert_eq!(edges[0].source_id, "node1");
    assert_eq!(edges[0].target_id, "node2");
}
```

**File:** `rust/crates/iqrah-storage/tests/user_repository_tests.rs`

```rust
use iqrah_storage::{init_user_db, SqliteUserRepository};
use iqrah_core::{UserRepository, MemoryState};
use chrono::Utc;

#[tokio::test]
async fn test_user_repository_save_and_get_state() {
    let pool = init_user_db(":memory:").await.unwrap();
    let repo = SqliteUserRepository::new(pool);

    // Create state
    let state = MemoryState {
        user_id: "user1".to_string(),
        node_id: "node1".to_string(),
        stability: 1.5,
        difficulty: 5.0,
        energy: 0.7,
        last_reviewed: Utc::now(),
        due_at: Utc::now(),
        review_count: 3,
    };

    // Save
    repo.save_memory_state(&state).await.unwrap();

    // Retrieve
    let retrieved = repo.get_memory_state("user1", "node1").await.unwrap();
    assert!(retrieved.is_some());

    let retrieved = retrieved.unwrap();
    assert_eq!(retrieved.stability, 1.5);
    assert_eq!(retrieved.difficulty, 5.0);
    assert_eq!(retrieved.energy, 0.7);
    assert_eq!(retrieved.review_count, 3);
}

#[tokio::test]
async fn test_user_repository_get_due_states() {
    let pool = init_user_db(":memory:").await.unwrap();
    let repo = SqliteUserRepository::new(pool);

    let now = Utc::now();

    // Create overdue state
    let overdue = MemoryState {
        user_id: "user1".to_string(),
        node_id: "node1".to_string(),
        stability: 1.0,
        difficulty: 5.0,
        energy: 0.5,
        last_reviewed: now,
        due_at: now - chrono::Duration::hours(1), // Overdue
        review_count: 1,
    };

    // Create future state
    let future = MemoryState {
        user_id: "user1".to_string(),
        node_id: "node2".to_string(),
        stability: 1.0,
        difficulty: 5.0,
        energy: 0.5,
        last_reviewed: now,
        due_at: now + chrono::Duration::hours(1), // Not due yet
        review_count: 1,
    };

    repo.save_memory_state(&overdue).await.unwrap();
    repo.save_memory_state(&future).await.unwrap();

    // Get due states
    let due = repo.get_due_states("user1", now, 10).await.unwrap();

    assert_eq!(due.len(), 1);
    assert_eq!(due[0].node_id, "node1");
}

#[tokio::test]
async fn test_user_repository_stats() {
    let pool = init_user_db(":memory:").await.unwrap();
    let repo = SqliteUserRepository::new(pool);

    // Set stat
    repo.set_stat("reviews_today", "42").await.unwrap();

    // Get stat
    let value = repo.get_stat("reviews_today").await.unwrap();
    assert_eq!(value, Some("42".to_string()));

    // Update stat
    repo.set_stat("reviews_today", "43").await.unwrap();
    let value = repo.get_stat("reviews_today").await.unwrap();
    assert_eq!(value, Some("43".to_string()));
}

#[tokio::test]
async fn test_user_repository_session_state() {
    let pool = init_user_db(":memory:").await.unwrap();
    let repo = SqliteUserRepository::new(pool);

    let nodes = vec!["node1".to_string(), "node2".to_string(), "node3".to_string()];

    // Save session
    repo.save_session_state(&nodes).await.unwrap();

    // Retrieve session
    let retrieved = repo.get_session_state().await.unwrap();
    assert_eq!(retrieved.len(), 3);
    assert_eq!(retrieved[0], "node1");
    assert_eq!(retrieved[1], "node2");
    assert_eq!(retrieved[2], "node3");

    // Clear session
    repo.clear_session_state().await.unwrap();
    let retrieved = repo.get_session_state().await.unwrap();
    assert_eq!(retrieved.len(), 0);
}

#[tokio::test]
async fn test_migrations_run_correctly() {
    let pool = init_user_db(":memory:").await.unwrap();

    // Check that migration v2 ran (app_settings table exists)
    let row = sqlx::query("SELECT value FROM app_settings WHERE key = 'schema_version'")
        .fetch_optional(&pool)
        .await
        .unwrap();

    assert!(row.is_some());
}
```

Run tests:
```bash
cargo test -p iqrah-storage
```

## Task 8.3: Migration Tests

**File:** `rust/crates/iqrah-storage/tests/migration_tests.rs`

```rust
use iqrah_storage::{init_content_db, init_user_db, migrate_from_old_db};
use tempfile::NamedTempFile;
use sqlx::{SqlitePool, Executor};

#[tokio::test]
async fn test_full_migration_flow() {
    // 1. Create old database
    let old_db_file = NamedTempFile::new().unwrap();
    let old_db_path = old_db_file.path().to_str().unwrap();

    let old_pool = SqlitePool::connect(&format!("sqlite://{}", old_db_path))
        .await
        .unwrap();

    // Create old schema
    old_pool.execute(
        "CREATE TABLE nodes (id TEXT PRIMARY KEY, node_type TEXT, created_at INTEGER)"
    ).await.unwrap();

    old_pool.execute(
        "CREATE TABLE edges (
            source_id TEXT, target_id TEXT, edge_type INTEGER,
            distribution_type INTEGER, param1 REAL, param2 REAL,
            PRIMARY KEY (source_id, target_id)
        )"
    ).await.unwrap();

    old_pool.execute(
        "CREATE TABLE node_metadata (node_id TEXT, key TEXT, value TEXT, PRIMARY KEY (node_id, key))"
    ).await.unwrap();

    old_pool.execute(
        "CREATE TABLE user_memory_states (
            user_id TEXT, node_id TEXT, stability REAL, difficulty REAL,
            energy REAL, last_reviewed INTEGER, due_at INTEGER, review_count INTEGER,
            PRIMARY KEY (user_id, node_id)
        )"
    ).await.unwrap();

    // Insert test data
    old_pool.execute(
        "INSERT INTO nodes VALUES ('WORD_INSTANCE:1:1:1', 'word_instance', 0)"
    ).await.unwrap();

    old_pool.execute(
        "INSERT INTO node_metadata VALUES ('WORD_INSTANCE:1:1:1', 'arabic', 'بِسْمِ')"
    ).await.unwrap();

    old_pool.execute(
        "INSERT INTO user_memory_states VALUES ('user1', 'WORD_INSTANCE:1:1:1', 1.0, 5.0, 0.5, 0, 0, 1)"
    ).await.unwrap();

    old_pool.close().await;

    // 2. Create new databases
    let content_pool = init_content_db(":memory:").await.unwrap();
    let user_pool = init_user_db(":memory:").await.unwrap();

    // 3. Run migration
    migrate_from_old_db(old_db_path, &content_pool, &user_pool)
        .await
        .unwrap();

    // 4. Verify content.db
    let node_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM nodes")
        .fetch_one(&content_pool)
        .await
        .unwrap();
    assert_eq!(node_count, 1);

    let metadata_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM node_metadata")
        .fetch_one(&content_pool)
        .await
        .unwrap();
    assert_eq!(metadata_count, 1);

    // 5. Verify user.db
    let state_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM user_memory_states")
        .fetch_one(&user_pool)
        .await
        .unwrap();
    assert_eq!(state_count, 1);

    // 6. Verify data integrity
    let row = sqlx::query("SELECT energy FROM user_memory_states WHERE node_id = 'WORD_INSTANCE:1:1:1'")
        .fetch_one(&user_pool)
        .await
        .unwrap();

    let energy: f64 = row.get("energy");
    assert_eq!(energy, 0.5);
}
```

Run tests:
```bash
cargo test -p iqrah-storage migration_tests
```

## Task 8.4: Flutter Integration Tests

**File:** `integration_test/app_test.dart`

```dart
import 'package:flutter_test/flutter_test.dart';
import 'package:integration_test/integration_test.dart';
import 'package:iqrah_mobile/main.dart';

void main() {
  IntegrationTestWidgetsFlutterBinding.ensureInitialized();

  testWidgets('App initializes successfully', (tester) async {
    await tester.pumpWidget(const MyApp());
    await tester.pumpAndSettle();

    // Verify app loads
    expect(find.text('Iqrah MVP'), findsOneWidget);
  });

  testWidgets('Can start and complete a session', (tester) async {
    await tester.pumpWidget(const MyApp());
    await tester.pumpAndSettle();

    // Start session
    await tester.tap(find.text('Start Session'));
    await tester.pumpAndSettle();

    // Should show exercise
    expect(find.text('Reveal'), findsOneWidget);

    // Complete one review
    await tester.tap(find.text('Reveal'));
    await tester.pumpAndSettle();

    await tester.tap(find.text('Good'));
    await tester.pumpAndSettle();

    // Should advance to next item or show completion
    // (depends on session length)
  });
}
```

Run tests:
```bash
flutter test integration_test/app_test.dart
```

## Test Coverage Report

Generate coverage report:

```bash
# Install tarpaulin
cargo install cargo-tarpaulin

# Run coverage
cd /home/user/iqrah-mobile/rust
cargo tarpaulin --workspace --out Html --exclude-files 'crates/iqrah-api/*'

# Open report
open tarpaulin-report.html
```

## Success Criteria

- [ ] All unit tests pass: `cargo test -p iqrah-core`
- [ ] All integration tests pass: `cargo test -p iqrah-storage`
- [ ] Migration tests pass
- [ ] Flutter integration tests pass
- [ ] Coverage > 80% for iqrah-core
- [ ] Coverage > 70% for iqrah-storage
- [ ] No test failures

## Next Step

Proceed to `09-VALIDATION.md`

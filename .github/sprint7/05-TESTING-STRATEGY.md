# Sprint 7: Testing Strategy

**Date:** 2025-10-04
**Purpose:** Achieve 80%+ test coverage and production-ready confidence

---

## Testing Philosophy

**Test Pyramid**
```
         ╱────────╲
        ╱  E2E (5%) ╲
       ╱─────────────╲
      ╱ Integration   ╲
     ╱    (20%)        ╲
    ╱───────────────────╲
   ╱   Unit Tests (75%)  ╲
  ╱──────────────────────╲
```

**Principles:**
1. **Fast Feedback:** Unit tests run in <1s
2. **Isolation:** Each test independent
3. **Clarity:** Test name = specification
4. **Coverage:** Target 80%+ for business logic

---

## Layer 1: Unit Tests (iqrah-core)

### Purpose
Test pure business logic in complete isolation

### Tools
- `cargo test` (built-in)
- `mockall` (mocking traits)
- `proptest` (property-based testing)
- `rstest` (fixtures)

### Test Structure

#### Example: Scheduler Tests
`crates/iqrah-core/tests/scheduler_tests.rs`
```rust
use iqrah_core::domain::*;
use iqrah_core::services::*;
use mockall::*;
use rstest::*;

#[fixture]
fn mock_repos() -> (MockContentRepository, MockUserRepository) {
    (MockContentRepository::new(), MockUserRepository::new())
}

#[rstest]
fn test_process_review_increases_energy_on_good_grade(
    mut mock_repos: (MockContentRepository, MockUserRepository)
) {
    let (mut content_repo, mut user_repo) = mock_repos;

    // Arrange
    let initial_state = MemoryState {
        node_id: NodeId::from("test_node"),
        energy: Energy::new(0.5).unwrap(),
        // ... other fields
    };

    user_repo.expect_get_memory_state()
        .times(1)
        .returning(move |_, _| Ok(Some(initial_state.clone())));

    user_repo.expect_save_memory_state()
        .times(1)
        .withf(|_, state| state.energy.value() > 0.5)  // Energy increased
        .returning(|_, _| Ok(()));

    let scheduler = FsrsScheduler::default();
    let service = LearningService::new(
        Arc::new(content_repo),
        Arc::new(user_repo),
        Arc::new(scheduler),
    );

    // Act
    let result = tokio_test::block_on(
        service.process_review("user1", &NodeId::from("test_node"), ReviewGrade::Good)
    );

    // Assert
    assert!(result.is_ok());
    let new_state = result.unwrap();
    assert!(new_state.energy.value() > 0.5);
}

#[rstest]
#[case(ReviewGrade::Again, true)]   // Energy should decrease
#[case(ReviewGrade::Good, false)]   // Energy should increase
fn test_review_grade_affects_energy_direction(
    #[case] grade: ReviewGrade,
    #[case] should_decrease: bool,
) {
    // ... test implementation
}
```

#### Property-Based Testing
`crates/iqrah-core/tests/energy_properties.rs`
```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn energy_always_bounded(
        initial_energy in 0.0f64..=1.0,
        delta in -2.0f64..2.0
    ) {
        let initial = Energy::new(initial_energy).unwrap();
        let result = apply_energy_delta(initial, delta);

        // Property: Energy must stay in [0, 1]
        prop_assert!(result.value() >= 0.0);
        prop_assert!(result.value() <= 1.0);
    }

    #[test]
    fn fsrs_stability_increases_on_successful_review(
        initial_stability in 0.1f64..100.0,
        difficulty in 1.0f64..10.0
    ) {
        let state = MemoryState {
            stability: initial_stability,
            difficulty,
            // ...
        };

        let new_state = scheduler.update_state(state, ReviewGrade::Good).unwrap();

        // Property: Stability increases on Good grade
        prop_assert!(new_state.stability > initial_stability);
    }
}
```

#### Scoring Algorithm Tests
```rust
#[rstest]
#[case(10.0, 0.5, 0.8, 18.6)]  // days_overdue=10, mastery_gap=0.5, importance=0.8
#[case(0.0, 1.0, 0.5, 2.75)]   // not overdue, low mastery, medium importance
fn test_priority_score_calculation(
    #[case] days_overdue: f64,
    #[case] mastery_gap: f64,
    #[case] importance: f64,
    #[case] expected: f64,
) {
    let weights = ScoreWeights {
        w_due: 1.0,
        w_need: 2.0,
        w_yield: 1.5,
    };

    let score = calculate_priority_score(days_overdue, mastery_gap, importance, &weights);

    assert!((score - expected).abs() < 0.01);  // Float comparison
}
```

### Coverage Goal: 90%+ for iqrah-core

```bash
# Run with coverage
cargo tarpaulin --workspace --exclude iqrah-api --exclude iqrah-cli --out Html

# Open report
open tarpaulin-report.html
```

---

## Layer 2: Integration Tests (iqrah-storage)

### Purpose
Test repository implementations with real SQLite databases

### Tools
- In-memory SQLite (`:memory:`)
- `sqlx::test` macro
- Test fixtures

### Test Structure

#### Database Fixtures
`crates/iqrah-storage/tests/common/mod.rs`
```rust
use sqlx::{SqlitePool, sqlite::SqlitePoolOptions};

pub async fn create_test_content_db() -> SqlitePool {
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect(":memory:")
        .await
        .unwrap();

    // Run content schema
    sqlx::query(include_str!("../migrations/content_schema.sql"))
        .execute(&pool)
        .await
        .unwrap();

    // Seed test data
    seed_test_content(&pool).await;

    pool
}

pub async fn seed_test_content(pool: &SqlitePool) {
    sqlx::query!(
        "INSERT INTO nodes (id, node_type, created_at) VALUES (?, ?, ?)",
        "WORD_INSTANCE:1:1:1",
        "word_instance",
        0
    )
    .execute(pool)
    .await
    .unwrap();

    sqlx::query!(
        "INSERT INTO quran_text (node_id, arabic) VALUES (?, ?)",
        "WORD_INSTANCE:1:1:1",
        "بِسْمِ"
    )
    .execute(pool)
    .await
    .unwrap();

    // ... seed more data
}

pub async fn create_test_user_db() -> SqlitePool {
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect(":memory:")
        .await
        .unwrap();

    sqlx::query(include_str!("../migrations/user_schema.sql"))
        .execute(&pool)
        .await
        .unwrap();

    pool
}
```

#### Repository Tests
`crates/iqrah-storage/tests/content_repository_tests.rs`
```rust
use iqrah_storage::content::SqliteContentRepository;
use iqrah_core::ports::ContentRepository;

#[tokio::test]
async fn test_get_quran_text() {
    let pool = create_test_content_db().await;
    let repo = SqliteContentRepository::new(pool);

    let text = repo.get_quran_text(&NodeId::from("WORD_INSTANCE:1:1:1"))
        .await
        .unwrap();

    assert_eq!(text, "بِسْمِ");
}

#[tokio::test]
async fn test_get_translation_for_language() {
    let pool = create_test_content_db().await;
    let repo = SqliteContentRepository::new(pool);

    let translation = repo.get_translation(
        &NodeId::from("WORD_INSTANCE:1:1:1"),
        "en"
    ).await.unwrap();

    assert_eq!(translation, "In the name");
}

#[tokio::test]
async fn test_get_importance_scores_batch() {
    let pool = create_test_content_db().await;
    let repo = SqliteContentRepository::new(pool);

    let node_ids = vec![
        NodeId::from("WORD_INSTANCE:1:1:1"),
        NodeId::from("WORD_INSTANCE:1:1:2"),
    ];

    let scores = repo.get_importance_scores(&node_ids).await.unwrap();

    assert_eq!(scores.len(), 2);
    assert!(scores.contains_key(&node_ids[0]));
}
```

#### User Repository Tests
`crates/iqrah-storage/tests/user_repository_tests.rs`
```rust
#[tokio::test]
async fn test_save_and_retrieve_memory_state() {
    let pool = create_test_user_db().await;
    let repo = SqliteUserRepository::new(pool);

    let state = MemoryState {
        node_id: NodeId::from("test_node"),
        stability: 1.0,
        difficulty: 5.0,
        energy: Energy::new(0.5).unwrap(),
        last_reviewed: Utc::now(),
        due_at: Utc::now() + Duration::days(1),
        review_count: 1,
    };

    // Save
    repo.save_memory_state("user1", &state).await.unwrap();

    // Retrieve
    let retrieved = repo.get_memory_state("user1", &NodeId::from("test_node"))
        .await
        .unwrap()
        .expect("State should exist");

    assert_eq!(retrieved.stability, 1.0);
    assert_eq!(retrieved.energy.value(), 0.5);
}

#[tokio::test]
async fn test_get_due_states_filters_correctly() {
    let pool = create_test_user_db().await;
    let repo = SqliteUserRepository::new(pool);

    // Insert states with different due dates
    let now = Utc::now();

    let overdue = MemoryState {
        due_at: now - Duration::hours(1),  // Overdue
        // ...
    };

    let not_due = MemoryState {
        due_at: now + Duration::hours(1),  // Future
        // ...
    };

    repo.save_memory_state("user1", &overdue).await.unwrap();
    repo.save_memory_state("user1", &not_due).await.unwrap();

    let due_states = repo.get_due_states("user1", now).await.unwrap();

    assert_eq!(due_states.len(), 1);
    assert!(due_states[0].due_at < now);
}
```

### Coverage Goal: 80%+ for iqrah-storage

---

## Layer 3: End-to-End Tests

### Purpose
Test complete user workflows

### Tool: Flutter Integration Tests
`integration_test/app_test.dart`
```dart
import 'package:flutter_test/flutter_test.dart';
import 'package:integration_test/integration_test.dart';

void main() {
  IntegrationTestWidgetsFlutterBinding.ensureInitialized();

  testWidgets('Complete review session flow', (tester) async {
    // 1. Start app
    await tester.pumpWidget(const MyApp());
    await tester.pumpAndSettle();

    // 2. Verify dashboard shows
    expect(find.text('Iqrah MVP'), findsOneWidget);

    // 3. Check stats are 0 initially
    expect(find.text('0'), findsNWidgets(2));  // Reviews + Streak

    // 4. Start session
    expect(find.text('Start Session'), findsOneWidget);
    await tester.tap(find.text('Start Session'));
    await tester.pumpAndSettle();

    // 5. Complete a review
    expect(find.text('Reveal'), findsOneWidget);
    await tester.tap(find.text('Reveal'));
    await tester.pumpAndSettle();

    expect(find.text('Good'), findsOneWidget);
    await tester.tap(find.text('Good'));
    await tester.pumpAndSettle();

    // 6. Complete session
    for (int i = 0; i < 19; i++) {  // 20 items total
      await tester.tap(find.text('Reveal'));
      await tester.pumpAndSettle();
      await tester.tap(find.text('Good'));
      await tester.pumpAndSettle();
    }

    // 7. Verify summary screen
    expect(find.text('Great Work!'), findsOneWidget);
    expect(find.text('You reviewed 20 items'), findsOneWidget);

    // 8. Return to dashboard
    await tester.tap(find.text('Back to Dashboard'));
    await tester.pumpAndSettle();

    // 9. Verify stats updated
    expect(find.text('20'), findsOneWidget);  // Reviews today
    expect(find.text('1 day'), findsOneWidget);  // Streak
  });

  testWidgets('Session persistence on app restart', (tester) async {
    // 1. Start session
    await tester.pumpWidget(const MyApp());
    await tester.pumpAndSettle();

    await tester.tap(find.text('Start Session'));
    await tester.pumpAndSettle();

    // 2. Complete 5 reviews
    for (int i = 0; i < 5; i++) {
      await tester.tap(find.text('Reveal'));
      await tester.pumpAndSettle();
      await tester.tap(find.text('Good'));
      await tester.pumpAndSettle();
    }

    // 3. Simulate app restart (re-pump widget)
    await tester.pumpWidget(const MyApp());
    await tester.pumpAndSettle();

    // 4. Should resume on exercise page (not dashboard)
    expect(find.text('Reveal'), findsOneWidget);

    // 5. Continue session
    for (int i = 0; i < 15; i++) {
      await tester.tap(find.text('Reveal'));
      await tester.pumpAndSettle();
      await tester.tap(find.text('Good'));
      await tester.pumpAndSettle();
    }

    // 6. Should reach summary
    expect(find.text('Great Work!'), findsOneWidget);
  });
}
```

### Running E2E Tests
```bash
flutter test integration_test/app_test.dart
```

---

## Layer 4: Performance Tests

### Purpose
Ensure production-ready performance

### Benchmarks
`crates/iqrah-cli/src/commands/bench.rs`
```rust
use std::time::Instant;

pub async fn benchmark_session_generation(iterations: u32) -> Result<()> {
    let (content_repo, user_repo, scheduler) = setup_test_services().await?;
    let service = LearningService::new(content_repo, user_repo, scheduler);

    let mut total_duration = Duration::zero();

    for _ in 0..iterations {
        let start = Instant::now();

        let _ = service.get_due_items("bench_user", 20, None, false).await?;

        total_duration += start.elapsed();
    }

    let avg_ms = total_duration.as_millis() / iterations as u128;

    println!("Session Generation Benchmark:");
    println!("  Iterations: {}", iterations);
    println!("  Average: {}ms", avg_ms);
    println!("  Target: <50ms");

    if avg_ms > 50 {
        println!("⚠️  Performance degradation detected!");
    } else {
        println!("✅ Performance target met");
    }

    Ok(())
}
```

### Run Benchmarks
```bash
./target/release/iqrah bench session --iterations 1000
./target/release/iqrah bench propagation --iterations 100
```

---

## Continuous Integration Setup

### GitHub Actions Workflow
`.github/workflows/test.yml`
```yaml
name: Test Suite

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest

    steps:
      - uses: actions/checkout@v3

      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable

      - name: Run unit tests
        run: cargo test --workspace --lib

      - name: Run integration tests
        run: cargo test --workspace --test '*'

      - name: Check code coverage
        run: |
          cargo install cargo-tarpaulin
          cargo tarpaulin --workspace --out Xml

      - name: Upload coverage
        uses: codecov/codecov-action@v3
        with:
          file: ./cobertura.xml

      - name: Install Flutter
        uses: subosito/flutter-action@v2

      - name: Run Flutter tests
        run: flutter test

      - name: Run integration tests
        run: flutter test integration_test/
```

---

## Test Coverage Goals

| Component | Target | Current | Strategy |
|-----------|--------|---------|----------|
| iqrah-core | 90% | 0% | Unit tests with mocks |
| iqrah-storage | 80% | 0% | Integration tests |
| iqrah-api | 60% | 0% | E2E + manual |
| Flutter | 70% | ~0% | Widget + integration |

---

## Testing Checklist

### Before Merge
- [ ] All unit tests pass: `cargo test --lib`
- [ ] All integration tests pass: `cargo test --test '*'`
- [ ] Coverage > 80%: `cargo tarpaulin`
- [ ] Flutter tests pass: `flutter test`
- [ ] E2E tests pass: `flutter test integration_test/`
- [ ] Benchmarks meet targets: `iqrah bench`
- [ ] No compiler warnings: `cargo clippy`

### Before Release
- [ ] Full regression test suite
- [ ] Performance benchmarks
- [ ] Memory leak tests (Valgrind)
- [ ] Load testing (1000+ reviews)
- [ ] Data migration validation

---

## Next Steps

1. **Implement Unit Tests First** (Week 1)
   - Start with iqrah-core
   - Achieve 80%+ coverage before moving on

2. **Add Integration Tests** (Week 2)
   - Test each repository implementation
   - Validate SQL queries

3. **E2E Tests** (Week 2-3)
   - Critical user flows
   - Regression prevention

4. **Performance Benchmarks** (Week 3)
   - Establish baselines
   - Monitor regressions

---

## Resources

- **Mockall Guide:** https://docs.rs/mockall/latest/mockall/
- **PropTest Book:** https://altsysrq.github.io/proptest-book/
- **Rstest Docs:** https://docs.rs/rstest/latest/rstest/
- **Flutter Integration Testing:** https://docs.flutter.dev/testing/integration-tests

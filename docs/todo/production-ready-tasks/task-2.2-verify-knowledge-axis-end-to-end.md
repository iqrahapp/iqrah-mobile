# Task 2.2: Verify Knowledge Axis End-to-End Flow

## Metadata
- **Priority:** P0 (Critical - Core Feature Validation)
- **Estimated Effort:** 1 day
- **Dependencies:** Task 2.1 (Full knowledge graph must be generated)
- **Agent Type:** Testing + Integration
- **Parallelizable:** Yes (with 2.3, 2.4 after 2.1 completes)

## Goal

Verify that knowledge axis functionality works end-to-end: session generation with axis filtering, exercise routing by axis, and energy propagation across axes. No new implementation—just comprehensive testing and validation.

## Context

After Task 2.1, we have:
- ✅ Full knowledge graph with all 6 axes in database
- ✅ Rust code already implemented (domain models, exercise service, session service)
- ❓ Unknown: Does it actually work end-to-end?

**This task ensures:**
1. Session service correctly filters by knowledge axis
2. Exercise service routes to correct exercise type by axis
3. Energy propagation works across axes
4. User can complete a full learning flow for each axis
5. No bugs or edge cases

**Why No New Code?**
The Rust implementation was done by AI agents previously. This task validates their work with comprehensive integration tests.

## Current State

**Rust Implementation (Already Exists):**
- `rust/crates/iqrah-core/src/domain/models.rs` (lines 54-147) - `KnowledgeAxis` enum
- `rust/crates/iqrah-core/src/exercises/service.rs` (lines 84-146) - Exercise routing
- `rust/crates/iqrah-core/src/services/session_service.rs` (lines 106-120) - Axis filtering
- `rust/tests/knowledge_axis_test.rs` - Basic parsing tests

**Existing Tests:**
- Unit tests for parsing (pass)
- No integration tests for full flow
- No CLI tests for axis filtering

**After Task 2.1:**
- Database has knowledge nodes (e.g., `1:1:memorization`)
- Migration file imported successfully

## Target State

### Comprehensive Integration Tests

**File:** `rust/tests/knowledge_axis_integration_test.rs` (NEW)

Tests covering:
1. Session generation with axis filtering
2. Exercise generation by axis
3. Review recording and energy propagation
4. Cross-axis energy flow
5. All 6 axes (memorization, translation, tafsir, tajweed, contextual_memorization, meaning)

### CLI Verification Script

**File:** `rust/scripts/verify_knowledge_axis.sh` (NEW)

Script to test all axes via CLI:
```bash
#!/bin/bash
set -e

echo "Testing Knowledge Axis End-to-End..."

for axis in memorization translation tafsir tajweed; do
    echo "=== Testing axis: $axis ==="

    # Generate session
    cargo run --bin iqrah-cli -- schedule \
        --goal memorization:chapters-1-3 \
        --axis $axis \
        --limit 5

    echo "✅ $axis axis session generated"
done

echo "✅ All axes verified!"
```

### Documentation

**File:** `docs/testing/knowledge-axis-verification.md` (NEW)

Document test results and any issues found.

## Implementation Steps

### Step 1: Verify Data Import (30 min)

**Commands:**
```bash
cd rust

# Initialize database with new migration
cargo run --bin iqrah-cli -- init

# Query knowledge nodes
sqlite3 ~/.local/share/iqrah/content.db \
    "SELECT COUNT(*) FROM node_metadata WHERE node_id LIKE '%:memorization'"

# Should return > 400

# Query knowledge edges
sqlite3 ~/.local/share/iqrah/content.db \
    "SELECT COUNT(*) FROM edges WHERE edge_type = 1"

# Should return > 2000
```

**Verify:**
- [ ] Knowledge nodes exist in database
- [ ] All 6 axes present
- [ ] Knowledge edges (type 1) exist

### Step 2: Test Session Generation by Axis (1-2 hours)

**Create integration test file:**

**File:** `rust/tests/knowledge_axis_integration_test.rs`

```rust
use iqrah_core::domain::models::KnowledgeAxis;
use iqrah_core::services::session_service::SessionService;
use iqrah_storage::content::init_content_db;
use iqrah_storage::user::init_user_db;
use tempfile::TempDir;

#[tokio::test]
async fn test_session_generation_memorization_axis() {
    let (content_repo, user_repo) = setup_test_repos().await;
    let session_service = SessionService::new(content_repo, user_repo);

    let sessions = session_service
        .generate_session_with_axis("default", "memorization:chapters-1-3", Some(KnowledgeAxis::Memorization), 5)
        .await
        .unwrap();

    assert_eq!(sessions.len(), 5, "Should generate 5 sessions");

    // Verify all nodes end with ":memorization"
    for session in &sessions {
        assert!(
            session.node_id.ends_with(":memorization"),
            "Node {} should end with :memorization",
            session.node_id
        );
    }
}

#[tokio::test]
async fn test_session_generation_translation_axis() {
    let (content_repo, user_repo) = setup_test_repos().await;
    let session_service = SessionService::new(content_repo, user_repo);

    let sessions = session_service
        .generate_session_with_axis("default", "memorization:chapters-1-3", Some(KnowledgeAxis::Translation), 5)
        .await
        .unwrap();

    assert_eq!(sessions.len(), 5);

    for session in &sessions {
        assert!(
            session.node_id.ends_with(":translation"),
            "Node {} should end with :translation",
            session.node_id
        );
    }
}

#[tokio::test]
async fn test_all_axes_return_results() {
    let (content_repo, user_repo) = setup_test_repos().await;
    let session_service = SessionService::new(content_repo, user_repo);

    let axes = vec![
        KnowledgeAxis::Memorization,
        KnowledgeAxis::Translation,
        KnowledgeAxis::Tafsir,
        KnowledgeAxis::Tajweed,
        KnowledgeAxis::ContextualMemorization,
        KnowledgeAxis::Meaning,
    ];

    for axis in axes {
        let sessions = session_service
            .generate_session_with_axis("default", "memorization:chapters-1-3", Some(axis.clone()), 3)
            .await
            .unwrap();

        assert!(
            !sessions.is_empty(),
            "Axis {:?} should return sessions",
            axis
        );
    }
}

async fn setup_test_repos() -> (Arc<dyn ContentRepository>, Arc<dyn UserRepository>) {
    let tmp = TempDir::new().unwrap();
    let content_db = tmp.path().join("content.db");
    let user_db = tmp.path().join("user.db");

    let content_pool = init_content_db(content_db.to_str().unwrap()).await.unwrap();
    let user_pool = init_user_db(user_db.to_str().unwrap()).await.unwrap();

    let content_repo = Arc::new(SqliteContentRepository::new(content_pool));
    let user_repo = Arc::new(SqliteUserRepository::new(user_pool));

    (content_repo, user_repo)
}
```

**Run:**
```bash
cargo test knowledge_axis_integration --nocapture
```

### Step 3: Test Exercise Generation by Axis (1 hour)

**Add to integration test:**

```rust
#[tokio::test]
async fn test_exercise_routing_by_axis() {
    let content_repo = setup_content_repo().await;
    let exercise_service = ExerciseService::new(content_repo);

    // Memorization node should generate memorization exercise
    let exercise = exercise_service
        .generate_exercise("1:1:memorization")
        .await
        .unwrap();

    assert!(matches!(exercise.exercise_type, ExerciseType::Memorization));

    // Translation node should generate translation exercise
    let exercise = exercise_service
        .generate_exercise("1:1:translation")
        .await
        .unwrap();

    assert!(matches!(exercise.exercise_type, ExerciseType::Translation));

    // Tafsir node should generate translation exercise (tafsir uses translation type)
    let exercise = exercise_service
        .generate_exercise("1:1:tafsir")
        .await
        .unwrap();

    assert!(matches!(exercise.exercise_type, ExerciseType::Translation));
}
```

### Step 4: Test Energy Propagation (2 hours)

**Add to integration test:**

```rust
#[tokio::test]
async fn test_energy_propagation_within_axis() {
    let (content_repo, user_repo) = setup_test_repos().await;
    let learning_service = LearningService::new(content_repo, user_repo);

    // Review verse 1:1 memorization
    learning_service
        .record_review("default", "1:1:memorization", Rating::Good)
        .await
        .unwrap();

    // Check that verse 1:2 memorization received energy
    let state = user_repo
        .get_memory_state("default", "1:2:memorization")
        .await
        .unwrap();

    assert!(state.is_some(), "Verse 1:2:memorization should have energy");
    let state = state.unwrap();
    assert!(state.energy > 0.0, "Energy should be > 0");
}

#[tokio::test]
async fn test_cross_axis_energy_propagation() {
    let (content_repo, user_repo) = setup_test_repos().await;
    let learning_service = LearningService::new(content_repo, user_repo);

    // Review translation node
    learning_service
        .record_review("default", "1:1:translation", Rating::Good)
        .await
        .unwrap();

    // Check that memorization node also received energy (cross-axis edge)
    let mem_state = user_repo
        .get_memory_state("default", "1:1:memorization")
        .await
        .unwrap();

    assert!(mem_state.is_some(), "Memorization node should receive energy from translation");
    assert!(mem_state.unwrap().energy > 0.0);
}
```

### Step 5: CLI Verification Script (30 min)

**File:** `rust/scripts/verify_knowledge_axis.sh`

```bash
#!/bin/bash
set -e

echo "=========================================="
echo "Knowledge Axis End-to-End Verification"
echo "=========================================="

# Initialize database
echo "Initializing database..."
cargo run --bin iqrah-cli -- init --force

# Test each axis
axes=("memorization" "translation" "tafsir" "tajweed")

for axis in "${axes[@]}"; do
    echo ""
    echo "=== Testing axis: $axis ==="

    # Generate session
    output=$(cargo run --bin iqrah-cli -- schedule \
        --goal memorization:chapters-1-3 \
        --axis "$axis" \
        --limit 3 2>&1)

    # Check output contains axis suffix
    if echo "$output" | grep -q ":$axis"; then
        echo "✅ $axis axis: Session generated with correct node IDs"
    else
        echo "❌ $axis axis: Failed to generate sessions with :$axis suffix"
        echo "Output: $output"
        exit 1
    fi
done

echo ""
echo "=========================================="
echo "✅ All axes verified successfully!"
echo "=========================================="
```

**Make executable and run:**
```bash
chmod +x rust/scripts/verify_knowledge_axis.sh
./rust/scripts/verify_knowledge_axis.sh
```

### Step 6: Test Full User Flow (1 hour)

**Manual test scenario:**

```bash
# Scenario: User learns verse 1:1 via memorization axis

# 1. Get session
cargo run --bin iqrah-cli -- schedule --axis memorization --limit 1

# Output: Node 1:1:memorization

# 2. Generate exercise
cargo run --bin iqrah-cli -- exercise --node-id "1:1:memorization"

# Output: Memorization exercise (recitation prompt)

# 3. Record review
cargo run --bin iqrah-cli -- review --node-id "1:1:memorization" --rating good

# Output: Review recorded, energy propagated

# 4. Check energy of next node
cargo run --bin iqrah-cli -- stats --node-id "1:2:memorization"

# Output: Energy > 0 (from propagation)

# 5. Verify cross-axis
cargo run --bin iqrah-cli -- stats --node-id "1:1:translation"

# Output: Energy > 0 (cross-axis propagation)
```

Document results.

### Step 7: Edge Case Testing (1 hour)

**Test edge cases:**

```rust
#[tokio::test]
async fn test_empty_axis_filter() {
    // Session generation without axis filter should include all nodes
    let sessions = session_service
        .generate_session("default", "memorization:chapters-1-3", 10)
        .await
        .unwrap();

    // Should include both content and knowledge nodes
    let has_content = sessions.iter().any(|s| !s.node_id.contains(":memorization"));
    let has_knowledge = sessions.iter().any(|s| s.node_id.contains(":memorization"));

    assert!(has_content || has_knowledge, "Should include various node types");
}

#[tokio::test]
async fn test_invalid_axis_handling() {
    // Verify that invalid axis strings are handled gracefully
    let result = KnowledgeAxis::from_str("invalid_axis");
    assert!(result.is_err());
}

#[tokio::test]
async fn test_knowledge_node_without_base() {
    // Edge case: Knowledge node for non-existent base node
    let content_repo = setup_content_repo().await;

    let node = content_repo.get_node("999:999:memorization").await.unwrap();
    // Should return None or handle gracefully
}
```

## Verification Plan

### Integration Tests

```bash
cd rust
cargo test knowledge_axis_integration --nocapture
```

- [ ] `test_session_generation_memorization_axis` passes
- [ ] `test_session_generation_translation_axis` passes
- [ ] `test_all_axes_return_results` passes (6 axes)
- [ ] `test_exercise_routing_by_axis` passes
- [ ] `test_energy_propagation_within_axis` passes
- [ ] `test_cross_axis_energy_propagation` passes
- [ ] All edge case tests pass

### CLI Script

```bash
cd rust
./scripts/verify_knowledge_axis.sh
```

- [ ] All 4 main axes generate sessions
- [ ] Node IDs have correct axis suffix
- [ ] Script exits with code 0 (success)

### Manual Verification Checklist

- [ ] Generate session for memorization axis → Returns memorization nodes
- [ ] Generate session for translation axis → Returns translation nodes
- [ ] Generate exercise for `1:1:memorization` → Returns memorization exercise
- [ ] Generate exercise for `1:1:translation` → Returns translation exercise
- [ ] Record review for memorization node → Energy propagates to next memorization node
- [ ] Record review for translation node → Energy propagates to memorization node (cross-axis)
- [ ] Session generation without axis filter → Returns mixed nodes

### Performance Check

```bash
# Measure session generation time
time cargo run --bin iqrah-cli -- schedule --axis memorization --limit 20
```

- [ ] Session generation completes in < 50ms
- [ ] No memory leaks or panics

### Regression Check

```bash
cd rust
cargo test --all-features
```

- [ ] All existing tests still pass
- [ ] No new warnings or errors

## Scope Limits & Safeguards

### ✅ MUST DO

- Create comprehensive integration tests
- Test all 6 knowledge axes
- Verify session generation with axis filtering
- Verify exercise routing by axis
- Verify energy propagation (same-axis and cross-axis)
- Create CLI verification script
- Test edge cases
- Document test results

### ❌ DO NOT

- Implement new features (this is testing only)
- Change existing Rust code (unless bug found)
- Modify database schema
- Add UI tests (CLI only)
- Performance optimization (unless critical issue found)

### ⚠️ If Uncertain

- If tests fail → Check Task 2.1 was completed successfully
- If no knowledge nodes in DB → Re-run Task 2.1 migration
- If exercise routing fails → Check `ExerciseService` implementation
- If energy propagation fails → Check `LearningService` implementation
- If unsure about expected behavior → Ask user for clarification

### If Bugs Found

If you discover bugs during testing:
1. Document the bug clearly (input, expected, actual)
2. Create minimal reproduction test case
3. Report to user (don't fix yet, unless trivial)
4. Continue testing other functionality

## Success Criteria

- [ ] Integration test file created with 8+ test cases
- [ ] All integration tests pass
- [ ] CLI verification script created and passes
- [ ] All 6 axes tested (memorization, translation, tafsir, tajweed, contextual_memorization, meaning)
- [ ] Session generation works for each axis
- [ ] Exercise routing works for each axis
- [ ] Energy propagation verified (same-axis)
- [ ] Cross-axis propagation verified
- [ ] Edge cases handled gracefully
- [ ] Performance acceptable (< 50ms for 20 sessions)
- [ ] Documentation created with test results
- [ ] No regressions (all existing tests pass)

## Related Files

**Create These Files:**
- `/rust/tests/knowledge_axis_integration_test.rs` - Integration tests
- `/rust/scripts/verify_knowledge_axis.sh` - CLI verification script
- `/docs/testing/knowledge-axis-verification.md` - Test results documentation

**Test These Files (No Changes Expected):**
- `/rust/crates/iqrah-core/src/domain/models.rs` - Knowledge axis enum
- `/rust/crates/iqrah-core/src/exercises/service.rs` - Exercise routing
- `/rust/crates/iqrah-core/src/services/session_service.rs` - Session filtering
- `/rust/crates/iqrah-core/src/services/learning_service.rs` - Energy propagation

**Dependencies:**
- Task 2.1 must be complete (knowledge graph with axis nodes imported)

## Notes

### Why This Task Matters

Previous AI agents implemented the Rust code, but without full graph data to test against. Now that Task 2.1 provides the data, we can validate:
- Does the implementation actually work?
- Are there edge cases or bugs?
- Does it meet the original design intent?

This is a **critical validation gate** before production.

### Expected Issues

Common issues that might surface:
- Node ID format mismatches (Python generates `1:1:memorization`, Rust expects `VERSE:1:1:memorization`)
- Missing edges between axis nodes
- Exercise service doesn't handle all axes
- Energy propagation weights incorrect

All issues should be documented, even if not fixed immediately.

### Test Coverage Goal

Aim for:
- 100% of axes tested (6/6)
- 100% of services tested (session, exercise, learning)
- 90%+ code coverage in axis-related code paths

### Documentation Importance

The test results documentation serves as:
- Proof that knowledge axis works
- Reference for future debugging
- Evidence of thorough testing for production readiness

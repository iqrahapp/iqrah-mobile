# Scheduler v2 Integration TODOs

This document outlines the remaining work to fully integrate Scheduler v2.0/v2.1 into the Iqrah codebase.

## Status: Repository Traits Extended ✅

- ✅ Added `get_scheduler_candidates()` to ContentRepository
- ✅ Added `get_prerequisite_parents()` to ContentRepository
- ✅ Added `get_goal()` and `get_nodes_for_goal()` to ContentRepository
- ✅ Added `get_parent_energies()` to UserRepository
- ✅ Added `get_bandit_arms()` and `update_bandit_arm()` to UserRepository
- ✅ Added `SchedulerGoal` struct to ports

## TODO 1: Implement SQL Queries in Storage Layer

### File: `rust/crates/iqrah-storage/src/content/repository.rs`

Add these implementations to `SqliteContentRepository`:

#### 1.1 `get_scheduler_candidates()`

```sql
SELECT
    n.id AS node_id,
    COALESCE(m_found.value, 0.0) AS foundational_score,
    COALESCE(m_infl.value, 0.0) AS influence_score,
    COALESCE(m_diff.value, 0.0) AS difficulty_score,
    COALESCE(ums.energy, 0.0) AS energy,
    COALESCE(ums.next_due_ts, 0) AS next_due_ts,
    COALESCE(m_quran.value, 0) AS quran_order
FROM node_goals ng
JOIN nodes n ON ng.node_id = n.id
LEFT JOIN node_metadata m_found
    ON n.id = m_found.node_id AND m_found.key = 'foundational_score'
LEFT JOIN node_metadata m_infl
    ON n.id = m_infl.node_id AND m_infl.key = 'influence_score'
LEFT JOIN node_metadata m_diff
    ON n.id = m_diff.node_id AND m_diff.key = 'difficulty_score'
LEFT JOIN node_metadata m_quran
    ON n.id = m_quran.node_id AND m_quran.key = 'quran_order'
-- Join user DB for memory states (requires separate query or federated connection)
-- For now, fetch from user_memory_states via user_repository
WHERE ng.goal_id = ?
```

**Note**: Since user_memory_states is in a separate database (user.db), you'll need to:
1. Fetch nodes from content.db
2. Fetch memory states from user.db separately
3. Merge in the application layer

#### 1.2 `get_prerequisite_parents()`

```sql
SELECT target_id AS node_id, source_id AS parent_id
FROM edges
WHERE edge_type = 0  -- 0 = Dependency (prereq)
  AND target_id IN (?, ?, ?, ...)  -- Use parameter binding with chunks
```

**Implementation notes**:
- Support chunking for large node_id lists (e.g., 500 at a time)
- Group results by target_id to build HashMap<node_id, Vec<parent_id>>

#### 1.3 `get_goal()`

```sql
SELECT goal_id, goal_type, goal_group, label, description
FROM goals
WHERE goal_id = ?
```

#### 1.4 `get_nodes_for_goal()`

```sql
SELECT node_id
FROM node_goals
WHERE goal_id = ?
ORDER BY priority DESC, node_id ASC
```

### File: `rust/crates/iqrah-storage/src/user/repository.rs`

Add these implementations to `SqliteUserRepository`:

#### 2.1 `get_parent_energies()`

```sql
SELECT node_id, CAST(energy AS REAL) as energy
FROM user_memory_states
WHERE user_id = ?
  AND node_id IN (?, ?, ?, ...)  -- Use parameter binding with chunks
```

**Implementation notes**:
- Support chunking for large node_id lists
- Return HashMap<node_id, f32>
- Missing nodes are NOT included (caller treats as 0.0)

#### 2.2 `get_bandit_arms()`

```sql
SELECT profile_name, successes, failures
FROM user_bandit_state
WHERE user_id = ?
  AND goal_group = ?
```

**Returns**: Vec<BanditArmState>

#### 2.3 `update_bandit_arm()`

```sql
INSERT INTO user_bandit_state (user_id, goal_group, profile_name, successes, failures, last_updated)
VALUES (?, ?, ?, ?, ?, ?)
ON CONFLICT (user_id, goal_group, profile_name)
DO UPDATE SET
    successes = excluded.successes,
    failures = excluded.failures,
    last_updated = excluded.last_updated
```

**Note**: Use current timestamp in milliseconds for `last_updated`

## TODO 2: Add CLI Schedule Command

### File: `rust/crates/iqrah-cli/src/schedule.rs` (NEW)

Create a new `schedule` subcommand with:

```rust
#[derive(Debug, clap::Args)]
pub struct ScheduleArgs {
    /// User ID
    #[arg(long)]
    user_id: String,

    /// Goal ID (e.g., "memorization:surah-1")
    #[arg(long)]
    goal_id: String,

    /// Session size (number of items)
    #[arg(long, default_value = "20")]
    session_size: usize,

    /// Session mode (revision or mixed-learning)
    #[arg(long, default_value = "mixed-learning")]
    mode: String,

    /// Verbose output (show all node details)
    #[arg(long, short)]
    verbose: bool,
}
```

### Implementation Steps:

1. Add `schedule` variant to `Commands` enum in `main.rs`
2. Import repositories (ContentRepository, UserRepository)
3. Call scheduling pipeline:
   - Fetch candidates via `content_repo.get_scheduler_candidates()`
   - Fetch prerequisites via `content_repo.get_prerequisite_parents()`
   - Fetch parent energies via `user_repo.get_parent_energies()`
   - Get goal via `content_repo.get_goal()` (for bandit)
   - Choose profile via bandit (if v2.1 enabled)
   - Generate session via `scheduler_v2::generate_session()`
4. Display results:
   - **Normal mode**: Just list node IDs
   - **Verbose mode**: Table with columns:
     - node_id
     - foundational_score
     - influence_score
     - difficulty_score
     - energy
     - readiness
     - days_overdue
     - unsatisfied_parent_count
     - final_score
     - quran_order

## TODO 3: Add Sample Data for Testing

### File: `rust/crates/iqrah-storage/migrations_content/20251117000002_scheduler_v2_sample_data.sql` (NEW)

```sql
-- Sample node_metadata for testing (Surah Al-Fatihah words)
INSERT INTO node_metadata (node_id, key, value) VALUES
    ('1:1', 'foundational_score', 0.8),
    ('1:1', 'influence_score', 0.6),
    ('1:1', 'difficulty_score', 0.2),
    ('1:1', 'quran_order', 1001001),

    ('1:2', 'foundational_score', 0.7),
    ('1:2', 'influence_score', 0.5),
    ('1:2', 'difficulty_score', 0.3),
    ('1:2', 'quran_order', 1001002),

    -- Add more verses...
;

-- Sample goals
INSERT INTO goals (goal_id, goal_type, goal_group, label, description) VALUES
    ('memorization:surah-1', 'surah', 'memorization', 'Memorize Surah Al-Fatihah', 'Master all 7 verses of Al-Fatihah'),
    ('vocab:roots-common', 'root', 'vocab', 'Learn 100 Common Roots', 'Master the most frequent Arabic roots');

-- Sample node-goal mappings
INSERT INTO node_goals (goal_id, node_id, priority) VALUES
    ('memorization:surah-1', '1:1', 1),
    ('memorization:surah-1', '1:2', 2),
    ('memorization:surah-1', '1:3', 3),
    ('memorization:surah-1', '1:4', 4),
    ('memorization:surah-1', '1:5', 5),
    ('memorization:surah-1', '1:6', 6),
    ('memorization:surah-1', '1:7', 7);

-- Sample prerequisite edges (1:1 must be mastered before 1:2, etc.)
INSERT INTO edges (source_id, target_id, edge_type, distribution_type, param1, param2) VALUES
    ('1:1', '1:2', 0, 0, 0.0, 0.0),  -- 1:1 → 1:2 (prereq)
    ('1:2', '1:3', 0, 0, 0.0, 0.0),  -- 1:2 → 1:3 (prereq)
    ('1:3', '1:4', 0, 0, 0.0, 0.0),  -- etc.
    ('1:4', '1:5', 0, 0, 0.0, 0.0),
    ('1:5', '1:6', 0, 0, 0.0, 0.0),
    ('1:6', '1:7', 0, 0, 0.0, 0.0);
```

### File: `rust/crates/iqrah-storage/migrations_user/20251117000002_scheduler_v2_sample_user_data.sql` (NEW)

```sql
-- Sample user memory states for testing
-- User "test_user" has partially memorized Al-Fatihah
INSERT INTO user_memory_states (user_id, node_id, stability, difficulty, energy, last_reviewed, due_at, review_count) VALUES
    ('test_user', '1:1', 5.0, 3.0, 0.8, 1700000000000, 1700000000000, 10),  -- Mastered
    ('test_user', '1:2', 4.0, 3.5, 0.6, 1700100000000, 1700100000000, 8),   -- Well-learned
    ('test_user', '1:3', 2.0, 4.0, 0.25, 1700200000000, 1700200000000, 3),  -- Learning (below threshold!)
    ('test_user', '1:4', 0.5, 5.0, 0.1, 1700300000000, 1700300000000, 1);   -- Struggling
    -- 1:5, 1:6, 1:7 are NEW (no entries)
```

## TODO 4: Test End-to-End

### Test Commands:

```bash
# Build CLI
cargo build --package iqrah-cli --release

# Test basic schedule
./target/release/iqrah schedule \
    --user-id "test_user" \
    --goal-id "memorization:surah-1" \
    --session-size 5 \
    --mode mixed-learning

# Test with verbose output
./target/release/iqrah schedule \
    --user-id "test_user" \
    --goal-id "memorization:surah-1" \
    --session-size 5 \
    --mode mixed-learning \
    --verbose

# Test revision mode
./target/release/iqrah schedule \
    --user-id "test_user" \
    --goal-id "memorization:surah-1" \
    --session-size 3 \
    --mode revision \
    --verbose
```

### Expected Behavior:

1. **Prerequisite Gate**: 1:4 should NOT be included (parent 1:3 has energy = 0.25 < 0.3)
2. **Eligible nodes**: 1:1, 1:2, 1:3, 1:5 (new), 1:6 (new), 1:7 (new)
3. **Mastery bands**:
   - 1:1 (energy=0.8) = Almost mastered
   - 1:2 (energy=0.6) = Almost there
   - 1:3 (energy=0.25) = Struggling
   - 1:5, 1:6, 1:7 (energy=0.0) = New
4. **Session (size=5)**: Should include mix of new, almost there, struggling according to 10/10/50/20/10 ratios

## TODO 5: Optional Enhancements

- [ ] Add bandit integration to CLI (use `--enable-bandit` flag)
- [ ] Add ability to record session results and update bandit state
- [ ] Add more comprehensive sample data (more surahs, roots, etc.)
- [ ] Add integration tests for end-to-end flow
- [ ] Create a "reset scheduler data" CLI command for testing
- [ ] Add metrics/logging for scheduler performance

## Notes

- All SQL implementations should use proper parameter binding (not string concatenation)
- Support chunking for large parameter lists (SQLite has ~999 parameter limit)
- Handle edge cases (empty results, missing metadata, etc.)
- Consider performance: add indexes if queries are slow
- Test with various scenarios (all new content, all overdue, mixed, etc.)

## Timeline Estimate

- TODO 1 (SQL implementations): 2-3 hours
- TODO 2 (CLI command): 1-2 hours
- TODO 3 (Sample data): 30 minutes
- TODO 4 (Testing): 1 hour
- **Total**: ~5-7 hours of focused development

## Completion Checklist

- [ ] All repository methods implemented with SQL
- [ ] CLI `schedule` command functional
- [ ] Sample data migrated and loadable
- [ ] End-to-end test successful with verbose output
- [ ] CI validation passes (build, clippy, test, fmt)
- [ ] Documentation updated
- [ ] Code committed and pushed

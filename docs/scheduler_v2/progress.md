# Scheduler v2.0/v2.1 Implementation Progress

**Status**: In Progress
**Started**: 2025-11-17
**Branch**: `claude/scheduler-v2-upgrade-01G1rW7HL1YSySyGsFtuSj1x`

---

## Specification Documents

- **Main Spec**: `docs/scheduler_v2/implementation_spec.md`
- **Session Modes Add-on**: `docs/scheduler_v2/session_modes_addon.md`
- **This Progress Doc**: `docs/scheduler_v2/progress.md`

---

## Current Status: Phase 1 - Analysis & Planning

### Completed

✅ **Specification Documents Saved** (2025-11-17)
- Created `docs/scheduler_v2/` directory
- Saved main implementation spec (v2.0 and v2.1)
- Saved session modes add-on spec

✅ **Codebase Analysis** (2025-11-17)
- Comprehensive exploration of Rust codebase structure
- Identified existing scheduler at `rust/crates/iqrah-core/src/services/session_service.rs`
- Analyzed database schemas (content.db and user.db)
- Reviewed repository patterns and testing infrastructure
- Full analysis documented in `/home/user/iqrah-mobile/RUST_CODEBASE_ANALYSIS.md`

### Schema Analysis & Gaps Identified

**Existing Tables (Usable)**:
- ✅ `user_memory_states` - Has `energy`, `due_at`, `user_id`, `node_id`
  - NOTE: `user_id` is TEXT (not INTEGER as spec expects)
  - NOTE: `due_at` is epoch MILLISECONDS (not seconds as spec expects)
- ✅ `edges` - Has `source_id`, `target_id`, `edge_type`
  - NOTE: `edge_type` is INTEGER (0=Dependency, 1=Knowledge), not TEXT
  - SPEC expects `edge_type='prereq'`, we'll use `edge_type=0` (Dependency)

**Missing Tables (Need Migration)**:
- ❌ `node_metadata` - For foundational_score, influence_score, difficulty_score, quran_order
- ❌ `goals` - For goal definitions (goal_id, goal_type, goal_group, label)
- ❌ `node_goals` - For mapping nodes to goals
- ❌ `user_bandit_state` - For v2.1 bandit optimizer

**Schema Adaptation Strategy**:
1. Create migration for missing tables
2. Adapt code to work with:
   - `user_id` as TEXT (instead of INTEGER)
   - `due_at` in milliseconds (convert to/from seconds in code)
   - `edge_type` as INTEGER 0 (instead of TEXT 'prereq')

---

## Implementation Plan

### Phase 1: Database Schema & Migrations ⏳

**Tasks**:
- [ ] Create content.db migration for:
  - `node_metadata(node_id, key, value)` table
  - `goals(goal_id, goal_type, goal_group, label)` table
  - `node_goals(goal_id, node_id)` table
- [ ] Create user.db migration for:
  - `user_bandit_state(user_id, goal_group, profile_name, successes, failures)` table
- [ ] Test migrations with in-memory databases
- [ ] Document schema deviations from spec

**Files to Create/Modify**:
- New: `rust/crates/iqrah-storage/migrations_content/20251117xxxxxx_scheduler_v2_schema.sql`
- New: `rust/crates/iqrah-storage/migrations_user/20251117xxxxxx_scheduler_v2_bandit.sql`

---

### Phase 2: v2.0 Foundations (Iteration 1) 🔜

**Tasks**:
- [ ] Create `scheduler_v2` module in `iqrah-core/src/`
- [ ] Implement core structs:
  - `UserProfile` (w_urgency, w_readiness, w_foundation, w_influence)
  - `CandidateNode` (id, scores, energy, next_due_ts, quran_order)
  - `InMemNode` (data, parent_ids)
- [ ] Implement helper functions:
  - `calculate_priority_score()`
  - `calculate_days_overdue()`
  - `calculate_readiness()`
- [ ] Add constants: `MASTERY_THRESHOLD = 0.3f32`
- [ ] Write unit tests for scoring functions

**Files to Create/Modify**:
- New: `rust/crates/iqrah-core/src/scheduler_v2/mod.rs`
- New: `rust/crates/iqrah-core/src/scheduler_v2/types.rs`
- New: `rust/crates/iqrah-core/src/scheduler_v2/scoring.rs`
- New: `rust/crates/iqrah-core/src/scheduler_v2/scoring_tests.rs`
- Modify: `rust/crates/iqrah-core/src/lib.rs` (add scheduler_v2 module)

---

### Phase 3: v2.0 Data Access Layer 🔜

**Tasks**:
- [ ] Extend `ContentRepository` trait with:
  - `get_candidate_nodes(goal_id, user_id, now_ts)` -> Vec<CandidateNode>
  - `get_prereq_parents(node_ids)` -> HashMap<NodeId, Vec<ParentId>>
  - `get_node_metadata(node_ids, keys)` -> HashMap<NodeId, Metadata>
- [ ] Extend `UserRepository` trait with:
  - `get_parent_energies(user_id, node_ids)` -> HashMap<NodeId, f32>
- [ ] Implement SQL queries with chunking support
- [ ] Handle milliseconds ↔ seconds conversion for timestamps
- [ ] Handle TEXT user_ids (no conversion needed if consistent)

**Files to Modify**:
- `rust/crates/iqrah-core/src/ports/content_repository.rs`
- `rust/crates/iqrah-core/src/ports/user_repository.rs`
- `rust/crates/iqrah-storage/src/content/repository.rs`
- `rust/crates/iqrah-storage/src/user/repository.rs`

---

### Phase 4: v2.0 Core Scheduler Logic (Iteration 2) 🔜

**Tasks**:
- [ ] Implement Prerequisite Mastery Gate
  - Filter candidates where all parents have energy >= 0.3
  - Compute unsatisfied_parent_count
- [ ] Implement session generation orchestrator:
  - `generate_session(user_id, goal_id, profile, session_size, now_ts, mode)`
- [ ] Implement difficulty-based bucketing:
  - Easy: difficulty < 0.4
  - Medium: 0.4 <= difficulty < 0.7
  - Hard: difficulty >= 0.7
- [ ] Implement 60/30/10 composition with fallback
- [ ] Write integration tests with mock data

**Files to Create/Modify**:
- New: `rust/crates/iqrah-core/src/scheduler_v2/session_generator.rs`
- New: `rust/crates/iqrah-core/src/scheduler_v2/composition.rs`
- New: `rust/crates/iqrah-core/src/scheduler_v2/tests/integration_tests.rs`

---

### Phase 5: v2.0 CLI Validation Tool 🔜

**Tasks**:
- [ ] Add `schedule` command to iqrah-cli
- [ ] Implement verbose output showing:
  - node_id, foundational_score, influence_score, difficulty_score
  - energy, readiness, days_overdue
  - unsatisfied_parent_count, final_score, quran_order
- [ ] Add tabular formatting
- [ ] Test with real database

**Files to Create/Modify**:
- New: `rust/crates/iqrah-cli/src/schedule.rs`
- Modify: `rust/crates/iqrah-cli/src/main.rs`

---

### Phase 6: v2.1 Bandit Integration (Iteration 3) 🔜

**Tasks**:
- [ ] Implement SessionResult struct
- [ ] Implement `calculate_session_reward()`
- [ ] Implement ProfileName enum (Balanced, FoundationHeavy, InfluenceHeavy, UrgencyHeavy)
- [ ] Implement `profile_weights()` mapping
- [ ] Implement BanditOptimizer:
  - `choose_arm()` using Thompson Sampling (Beta distribution)
  - `update_arm()` updating successes/failures
- [ ] Implement profile blending (80/20 chosen/safe)
- [ ] Integrate into session generation flow
- [ ] Add bandit-specific tests

**Files to Create/Modify**:
- New: `rust/crates/iqrah-core/src/scheduler_v2/bandit.rs`
- New: `rust/crates/iqrah-core/src/scheduler_v2/profiles.rs`
- New: `rust/crates/iqrah-core/src/scheduler_v2/bandit_tests.rs`
- Modify: Add `rand_distr` dependency to Cargo.toml

---

### Phase 7: Session Modes (Revision & MixedLearning) 🔜

**Tasks**:
- [ ] Implement SessionMode enum (Revision, MixedLearning)
- [ ] Implement Revision mode:
  - Candidate filter: seen & due only
  - Composition: by difficulty_score (60/30/10)
- [ ] Implement MixedLearning mode:
  - Candidate filter: new + due
  - Composition: by energy bands (10/10/50/20/10)
  - Mastery bands: New, Really struggling, Struggling, Almost there, Almost mastered
- [ ] Test both modes extensively
- [ ] Update CLI to support mode selection

**Files to Create/Modify**:
- New: `rust/crates/iqrah-core/src/scheduler_v2/session_modes.rs`
- Modify: `rust/crates/iqrah-core/src/scheduler_v2/session_generator.rs`
- Modify: `rust/crates/iqrah-core/src/scheduler_v2/composition.rs`

---

### Phase 8: Testing & Validation ✋

**Tasks**:
- [ ] Run all unit tests
- [ ] Run all integration tests
- [ ] Test with real data via CLI
- [ ] Validate CI requirements:
  - `RUSTFLAGS="-D warnings" cargo build --all-features`
  - `cargo clippy --all-features --all-targets -- -D warnings`
  - `cargo test --all-features`
  - `cargo fmt --all -- --check`
- [ ] Fix any warnings or errors
- [ ] Performance testing (if needed)

---

### Phase 9: Documentation & Commit 📝

**Tasks**:
- [ ] Update this progress document with final status
- [ ] Add inline documentation to code
- [ ] Update README if needed
- [ ] Commit with descriptive message
- [ ] Push to branch

---

## Key Decisions & Deviations

### Schema Adaptations

1. **user_id type**: Using TEXT (existing) instead of INTEGER (spec)
   - Decision: Keep as TEXT, no conversion needed
   - Impact: None, just type difference

2. **Timestamp units**: Existing schema uses milliseconds, spec uses seconds
   - Decision: Convert in Rust code (divide by 1000 when reading, multiply by 1000 when writing)
   - Impact: Minimal, just arithmetic

3. **edge_type encoding**: Existing uses INTEGER (0,1), spec uses TEXT ('prereq')
   - Decision: Map 'prereq' → edge_type=0 (Dependency edges)
   - Impact: None, semantic mapping

4. **Missing PRIMARY KEY on edges**: Current schema doesn't include edge_type in PK
   - Decision: Work with current schema, filter by edge_type=0 in queries
   - Impact: None for scheduler (only uses one edge type)

### Implementation Choices

1. **Module structure**: Creating `scheduler_v2` as separate module alongside existing `SessionService`
   - Rationale: Non-breaking change, can coexist with v1
   - Migration path: Gradual replacement

2. **Repository extensions**: Adding new methods to existing traits
   - Rationale: Follows existing patterns, easy to test
   - Impact: Requires trait implementations in storage layer

3. **Testing strategy**: In-memory databases for fast tests
   - Rationale: Existing pattern works well
   - Impact: Fast CI, no external dependencies

---

## Testing Strategy

### Unit Tests
- Scoring functions (calculate_priority_score, readiness, days_overdue)
- Bandit reward calculation
- Profile blending
- Helper functions

### Integration Tests
- Mock graph with A→C, B→C, C→D prerequisite chain
- Different user energy states
- Prerequisite gate filtering
- Difficulty bucketing with fallback
- Session mode composition (Revision vs MixedLearning)
- Bandit arm selection and updates

### CLI Validation
- Real database with sample data
- Verbose output verification
- Different session sizes
- Different profiles
- Different modes

---

## Dependencies to Add

```toml
[dependencies]
rand = "0.8"           # For RNG in bandit
rand_distr = "0.4"     # For Beta distribution (Thompson Sampling)
```

---

## Known Issues / TODOs

- [ ] Verify that edge_type=0 (Dependency) is indeed the correct mapping for prerequisites
- [ ] Decide if we need to create sample data for node_metadata, goals, node_goals
- [ ] Determine if existing code uses user_id consistently as TEXT
- [ ] Check if quran_order calculation needs to be implemented or if data is pre-populated

---

## How to Run Tests

```bash
cd rust

# Build with warnings as errors (CI simulation)
RUSTFLAGS="-D warnings" cargo build --all-features

# Run clippy
cargo clippy --all-features --all-targets -- -D warnings

# Run all tests
cargo test --all-features

# Run specific scheduler tests
cargo test scheduler_v2

# Check formatting
cargo fmt --all -- --check

# Fix formatting
cargo fmt --all
```

## How to Use CLI

```bash
cd rust

# Build CLI
cargo build --package iqrah-cli --release

# Run scheduler command (once implemented)
./target/release/iqrah schedule \
    --user-id "user1" \
    --goal-id "memorization:surah-1" \
    --session-size 20 \
    --mode mixed-learning \
    --verbose
```

---

## References

- Main spec: `docs/scheduler_v2/implementation_spec.md`
- Session modes: `docs/scheduler_v2/session_modes_addon.md`
- Codebase analysis: `RUST_CODEBASE_ANALYSIS.md`
- CLAUDE.md: Project instructions for CI validation

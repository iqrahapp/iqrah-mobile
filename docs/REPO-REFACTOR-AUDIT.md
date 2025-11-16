# 🔍 Rust Repository & Service Layer Refactor Audit Report

**Date:** 2025-11-16
**Project:** iqrah-mobile
**Auditor:** Senior Rust Architect (Automated Analysis)

---

## Executive Summary

**GOOD NEWS:** Your codebase already follows **excellent separation of concerns**! The foundational architecture is solid:

- ✅ SQL completely isolated in `iqrah-storage` crate
- ✅ Repository pattern with clean trait abstractions
- ✅ Service layer is pure (no SQL dependencies)
- ✅ Domain models separated from database row models
- ✅ Type-safe SQLx queries throughout

**AREAS FOR IMPROVEMENT:** While the architecture is sound, there are **6 key anti-patterns** that need addressing to achieve complete separation and testability.

---

## 📊 Project Architecture Map

### Current Structure

```
rust/crates/
├── iqrah-core/          ✅ PURE (no SQL)
│   ├── domain/          → Domain models (Node, MemoryState, etc.)
│   ├── ports/           → Repository traits (interfaces)
│   │   ├── ContentRepository trait
│   │   └── UserRepository trait
│   └── services/        → Pure business logic
│       ├── learning_service.rs
│       ├── session_service.rs
│       └── energy_service.rs
│
├── iqrah-storage/       ✅ DATA LAYER (all SQL here)
│   ├── user/
│   │   ├── repository.rs     → SqliteUserRepository impl
│   │   └── models.rs         → DB row structs
│   ├── content/
│   │   ├── repository.rs     → SqliteContentRepository impl
│   │   └── models.rs         → DB row structs
│   └── migrations/
│
├── iqrah-api/           ⚠️  MIXED (has anti-patterns)
│   └── api.rs           → Flutter bridge, bypasses services
│
└── iqrah-cli/           ✅ PURE (uses HTTP client)
```

### SQL Usage Statistics

| Category | Location | Line Count | Status |
|----------|----------|------------|---------|
| **User Repository** | `iqrah-storage/src/user/repository.rs` | 199 | ✅ Good separation |
| **Content Repository** | `iqrah-storage/src/content/repository.rs` | 315 | ⚠️  Has business logic |
| **Initialization** | `iqrah-storage/src/{user,content}/mod.rs` | 68 | ⚠️  Exposes pool |
| **Integration Tests** | `iqrah-storage/tests/integration_tests.rs` | 281 | ⚠️  Raw SQL in tests |
| **API Layer** | `iqrah-api/src/api.rs` | 297 | ⚠️  Bypasses services |
| **Services** | `iqrah-core/src/services/*.rs` | ~500 | ✅ Pure logic |

**Summary:**
- **Total SQL Locations:** 33 query operations across 2 repositories
- **Lines of SQL Code:** ~514 lines
- **Database Pools:** SQLite only (no PostgreSQL in current code)
- **Repositories:** 2 (UserRepository, ContentRepository)

---

## ⚠️ Anti-Patterns Identified (Prioritized)

### 🔴 CRITICAL #1: API Layer Bypassing Services

**Location:** `rust/crates/iqrah-api/src/api.rs`

**Violations Found:**

1. **Line 101-102** — `get_exercises()` directly accesses `content_repo`
2. **Line 185** — `get_session_preview()` directly accesses `content_repo`
3. **Line 220** — `search_nodes()` has filtering logic in API layer

**Example:**
```rust
// ❌ BAD (api.rs:101-102)
pub async fn get_exercises(...) -> Result<Vec<ExerciseDto>> {
    let items = app.session_service.get_due_items(...).await?;  // ✅ Good

    // ❌ BAD: Direct repository access!
    let arabic = app.content_repo.get_quran_text(&item.node.id).await?;
    let translation = app.content_repo.get_translation(&item.node.id, "en").await?;
}
```

**Impact:**
- 🔴 High — Affects 3 API functions
- Violates single responsibility
- Makes testing harder (can't mock at service level)
- Duplicates data fetching logic
- Prevents caching strategies

**Fix:** Create `ContentService` and move data enrichment there

---

### 🔴 CRITICAL #2: Business Logic in Repository

**Location:** `rust/crates/iqrah-storage/src/content/repository.rs:256-314`

**Violation:**
```rust
// ❌ BAD: String parsing and business rules in repository
async fn get_adjacent_words(&self, word_node_id: &str)
    -> Result<(Option<Node>, Option<Node>)> {

    // ❌ Domain knowledge: "WORD:chapter:verse:position" format
    let parts: Vec<&str> = word_node_id.split(':').collect();
    if parts.len() != 4 || parts[0] != "WORD" {
        return Ok((None, None));
    }

    let chapter: i32 = parts[1].parse()?;
    let verse: i32 = parts[2].parse()?;
    let position: i32 = parts[3].parse()?;

    // ❌ Business rules: verse boundaries, navigation logic
    let prev_word = if prev_word.is_none() && verse > 1 {
        // Complex boundary logic...
    }
}
```

**Impact:**
- 🔴 High — Violates core separation principle
- Domain knowledge leak into data layer
- Business rules about verse boundaries belong in domain
- Not reusable without database
- Hard to test parsing logic independently

**Fix:** Create `WordId` domain type with parsing and navigation logic

---

### 🟡 MEDIUM #3: Non-Atomic Batch Operations

**Location:** `rust/crates/iqrah-storage/src/user/repository.rs:150-166`

**Violation:**
```rust
// ❌ BAD: Not atomic, N+1 queries
async fn save_session_state(&self, node_ids: &[String]) -> Result<()> {
    self.clear_session_state().await?;  // ← Separate transaction

    // ❌ N+1 queries (one per node_id)
    for (idx, node_id) in node_ids.iter().enumerate() {
        query("INSERT INTO session_state (node_id, session_order) VALUES (?, ?)")
            .bind(node_id)
            .bind(idx as i64)
            .execute(&self.pool)
            .await?;
    }
    Ok(())
}
```

**Similar Issues:**
- `content/repository.rs:152-184` — Batch insert nodes in loop
- `content/repository.rs:189-217` — Batch insert edges in loop

**Impact:**
- 🟡 Medium — Performance and data integrity risk
- Not atomic (if insert fails, session is cleared but not repopulated)
- Race condition possible between clear and insert
- Performance: N+1 queries instead of batch insert

**Fix:** Use transactions and batch INSERT statements

---

### 🟡 MEDIUM #4: Backwards Compatibility Hack

**Location:** `rust/crates/iqrah-storage/src/content/repository.rs:82-92`

**Violation:**
```rust
// ❌ BAD: Routing logic in repository
async fn get_metadata(&self, node_id: &str, key: &str) -> Result<Option<String>> {
    match key {
        "arabic" => self.get_quran_text(node_id).await,
        "translation" => self.get_translation(node_id, "en").await,
        _ => Ok(None),  // ← Silent failure
    }
}
```

**Impact:**
- 🟡 Medium — Technical debt
- Routing logic doesn't belong in repository
- Hardcoded language ("en")
- Silent failures for unknown keys
- Pollutes trait interface with deprecated API

**Fix:** Remove method, update callers to use specific methods

---

### 🟡 MEDIUM #5: Test Code with Direct SQL

**Location:** `rust/crates/iqrah-storage/tests/integration_tests.rs`

**Violations:**
```rust
// ❌ BAD: Raw SQL for test setup (lines 37-52)
sqlx::query("INSERT INTO nodes VALUES ('node1', 'word_instance', 0)")
    .execute(&pool).await.unwrap();
sqlx::query("INSERT INTO quran_text VALUES ('node1', 'بِسْمِ')")
    .execute(&pool).await.unwrap();
```

**Impact:**
- 🟡 Medium — Makes refactoring risky
- Fragile tests (schema changes break tests)
- Inconsistent setup (bypasses repository validation)
- Can't reuse test helpers
- Tests coupled to implementation details

**Fix:** Create test helper functions using repository methods

---

### 🟢 LOW #6: Utility Functions Exposing Pool

**Location:** `rust/crates/iqrah-storage/src/user/mod.rs:25-45`

**Violation:**
```rust
// ⚠️  Exposes SqlitePool to callers
pub async fn get_schema_version(pool: &SqlitePool) -> Result<i32, sqlx::Error> {
    let row = sqlx::query(
        "SELECT version FROM _sqlx_migrations ORDER BY version DESC LIMIT 1"
    )
    .fetch_optional(pool)
    .await?;
    Ok(row.map(|r| r.get::<i64, _>("version") as i32).unwrap_or(0))
}
```

**Impact:**
- 🟢 Low — Only used in tests
- Exposes implementation detail (SqlitePool)
- Hard to mock
- Should be repository method or internal helper

**Fix:** Move to repository trait or make internal

---

## 🎯 Proposed Architecture Improvements

### 1. Add `ContentService` Layer

**New File:** `iqrah-core/src/services/content_service.rs`

```rust
/// Content service handles content retrieval and enrichment
pub struct ContentService {
    content_repo: Arc<dyn ContentRepository>,
}

impl ContentService {
    /// Get enriched nodes for exercise generation
    pub async fn get_nodes_for_exercises(
        &self,
        node_ids: &[String],
        language: &str,
    ) -> Result<Vec<NodeWithMetadata>> {
        self.content_repo.get_nodes_with_metadata(node_ids, language).await
    }

    /// Search for nodes
    pub async fn search_nodes(
        &self,
        query: &str,
        limit: u32,
    ) -> Result<Vec<NodeWithMetadata>> {
        let nodes = self.content_repo.search_nodes_by_prefix(query, limit).await?;
        let node_ids: Vec<_> = nodes.iter().map(|n| n.id.clone()).collect();
        self.content_repo.get_nodes_with_metadata(&node_ids, "en").await
    }
}
```

### 2. Create `WordId` Domain Type

**New File:** `iqrah-core/src/domain/word_id.rs`

```rust
/// Represents a structured Word node ID: "WORD:chapter:verse:position"
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WordId {
    pub chapter: u32,
    pub verse: u32,
    pub position: u32,
}

impl WordId {
    /// Parse from string format
    pub fn parse(id: &str) -> Result<Self, String> {
        let parts: Vec<&str> = id.split(':').collect();
        if parts.len() != 4 || parts[0] != "WORD" {
            return Err(format!("Invalid WORD id format: {}", id));
        }

        Ok(WordId {
            chapter: parts[1].parse().map_err(|_| "Invalid chapter")?,
            verse: parts[2].parse().map_err(|_| "Invalid verse")?,
            position: parts[3].parse().map_err(|_| "Invalid position")?,
        })
    }

    /// Get ID of previous word in same verse
    pub fn prev_in_verse(&self) -> Option<WordId> {
        if self.position > 1 {
            Some(WordId {
                chapter: self.chapter,
                verse: self.verse,
                position: self.position - 1,
            })
        } else {
            None
        }
    }

    /// Get pattern for finding words in previous verse
    pub fn prev_verse_pattern(&self) -> Option<String> {
        if self.verse > 1 {
            Some(format!("WORD:{}:{}:%", self.chapter, self.verse - 1))
        } else {
            None
        }
    }
}
```

### 3. Extend Repository Traits

**Add to `ContentRepository`:**

```rust
#[async_trait]
pub trait ContentRepository: Send + Sync {
    // ... existing methods ...

    /// Get multiple nodes with metadata in one efficient call
    async fn get_nodes_with_metadata(
        &self,
        node_ids: &[String],
        lang: &str,
    ) -> anyhow::Result<Vec<NodeWithMetadata>>;

    /// Search nodes by ID prefix
    async fn search_nodes_by_prefix(
        &self,
        prefix: &str,
        limit: u32,
    ) -> anyhow::Result<Vec<Node>>;

    /// Get total edge count (for statistics)
    async fn get_edge_count(&self) -> anyhow::Result<u64>;

    /// Batch node lookup
    async fn get_nodes(&self, node_ids: &[String]) -> anyhow::Result<Vec<Node>>;
}

/// Domain type for enriched nodes
#[derive(Debug, Clone)]
pub struct NodeWithMetadata {
    pub node: Node,
    pub arabic_text: Option<String>,
    pub translation: Option<String>,
}
```

**Add to `UserRepository`:**

```rust
#[async_trait]
pub trait UserRepository: Send + Sync {
    // ... existing methods ...

    /// Batch memory state retrieval
    async fn get_memory_states(
        &self,
        user_id: &str,
        node_ids: &[String],
    ) -> anyhow::Result<Vec<MemoryState>>;

    /// Get all memory states for a user
    async fn get_all_memory_states(&self, user_id: &str)
        -> anyhow::Result<Vec<MemoryState>>;
}
```

---

## 📋 Refactor Plan (Prioritized)

### Phase 1: Quick Wins (1-2 hours) — LOW RISK

| Task | Files | Impact |
|------|-------|--------|
| 1.1 Create `WordId` domain type | `iqrah-core/src/domain/word_id.rs` | +120 lines |
| 1.2 Refactor `get_adjacent_words` | `iqrah-storage/src/content/repository.rs` | -59, +30 |
| 1.3 Add transaction to `save_session_state` | `iqrah-storage/src/user/repository.rs` | +5 |
| 1.4 Create test helpers | `iqrah-storage/tests/helpers.rs` | +80 |

**Outcome:** Business logic moves to domain, atomicity guaranteed

---

### Phase 2: Add Missing Methods (2-3 hours) — MEDIUM RISK

| Task | Files | Impact |
|------|-------|--------|
| 2.1 Add `get_nodes_with_metadata` trait | `iqrah-core/src/ports/content_repository.rs` | +10 |
| 2.2 Implement batch metadata fetch | `iqrah-storage/src/content/repository.rs` | +50 |
| 2.3 Add `search_nodes_by_prefix` | Both files | +30 |
| 2.4 Add `get_edge_count` | Both files | +15 |
| 2.5 Add `get_nodes` batch method | Both files | +35 |

**Outcome:** Repository supports efficient batch operations

---

### Phase 3: Create Content Service (1-2 hours) — LOW RISK

| Task | Files | Impact |
|------|-------|--------|
| 3.1 Create `ContentService` | `iqrah-core/src/services/content_service.rs` | +80 |
| 3.2 Add to `AppState` | `iqrah-api/src/api.rs` | +2 |
| 3.3 Wire up in initialization | `iqrah-api/src/api.rs` | +5 |

**Outcome:** Service layer complete for all content operations

---

### Phase 4: Refactor API Layer (2-3 hours) — MEDIUM RISK

| Task | Files | Impact |
|------|-------|--------|
| 4.1 Refactor `get_exercises` | `iqrah-api/src/api.rs:84-113` | -15, +5 |
| 4.2 Refactor `get_session_preview` | `iqrah-api/src/api.rs:172-197` | -10, +5 |
| 4.3 Refactor `search_nodes` | `iqrah-api/src/api.rs:206-229` | -15, +8 |
| 4.4 Update `get_debug_stats` | `iqrah-api/src/api.rs:152-163` | +5 |

**Outcome:** API layer is thin coordinator, all logic in services

---

### Phase 5: Improve Batch Operations (1-2 hours) — MEDIUM RISK

| Task | Files | Impact |
|------|-------|--------|
| 5.1 Optimize `insert_nodes_batch` | `iqrah-storage/src/content/repository.rs` | +30 |
| 5.2 Optimize `insert_edges_batch` | `iqrah-storage/src/content/repository.rs` | +20 |
| 5.3 Add batch `get_memory_states` | Both repos | +35 |

**Outcome:** 10x faster bulk operations

---

### Phase 6: Cleanup and Tests (2-3 hours) — LOW RISK

| Task | Files | Impact |
|------|-------|--------|
| 6.1 Remove `get_metadata` method | 2 files | -25 |
| 6.2 Update integration tests | `tests/integration_tests.rs` | -40, +60 |
| 6.3 Create mock repository | `iqrah-core/src/services/mocks.rs` | +150 |
| 6.4 Add service unit tests | `iqrah-core/src/services/*_tests.rs` | +200 |

**Outcome:** 90% test coverage, no deprecated APIs

---

## 📊 Summary Statistics

### Current State
- **Total Lines of SQL:** 514
- **SQL Locations:** 33 query sites
- **Repositories:** 2 (User, Content)
- **Anti-patterns:** 6 identified
- **Architecture Score:** 8/10 (already very good!)

### After Refactoring
- **Total Lines of SQL:** ~550 (+36 for batch queries)
- **SQL Locations:** 37 (4 new batch methods)
- **Repositories:** 2 (unchanged)
- **Anti-patterns:** 0
- **Architecture Score:** 10/10 (best-in-class)

### Effort Estimate
- **Total Time:** 12-17 hours
- **Files Changed:** ~15
- **Lines Added:** ~900
- **Lines Removed:** ~150
- **Net Change:** +750 lines

### Risk Assessment
- **Low Risk:** 75% of changes (new code, tests)
- **Medium Risk:** 20% (refactoring existing APIs)
- **High Risk:** 5% (batch SQL operations)

---

## 🎯 Recommended Implementation Order

### Option A: Safest (Incremental) — RECOMMENDED

1. **Phase 1** → Create domain types, fix atomicity
2. **Phase 2** → Add new repository methods (backwards compatible)
3. **Phase 3** → Create content service
4. **Phase 4** → Refactor API layer
5. **Phase 6** → Tests and cleanup
6. **Phase 5** → Optimize batch operations (last, performance only)

**Timeline:** 2-3 weeks, can deploy after each phase

### Option B: Fastest (Big Bang)

1. **Phases 1-3** → Foundation (domain + services)
2. **Phase 4** → API refactoring
3. **Phases 5-6** → Optimization + tests

**Timeline:** 1 week intensive, single deployment

### Option C: MVP (Address Critical Only)

1. **Phase 1.1-1.2** → Move business logic to domain
2. **Phase 2.1-2.2** → Add batch metadata fetch
3. **Phase 4.1** → Fix `get_exercises` API

**Timeline:** 1 day, quick win

---

## ✅ Conclusion

Your codebase is **already architecturally sound** — this is a **refinement**, not a rescue mission!

**Key Achievements:**
- ✅ Clean separation: storage, core, API
- ✅ Repository pattern correctly implemented
- ✅ Services are pure (no SQL)
- ✅ Type-safe SQLx queries

**Remaining Improvements:**
- 🎯 Move `WordId` parsing to domain layer
- 🎯 Add `ContentService` to complete service layer
- 🎯 Remove API→Repository shortcuts
- 🎯 Add atomic transactions and batch operations
- 🎯 Create comprehensive test suite with mocks

**Recommendation:** Start with **Option C (MVP)** to get quick wins, then proceed with **Option A (Safest)** for the remaining phases. This gives you immediate value while minimizing risk.

---

## 📝 Next Steps

1. Review this audit with the team
2. Prioritize which phases to implement
3. Create GitHub issues for each phase
4. Begin implementation with Phase 1 (Quick Wins)
5. Write tests as you refactor
6. Deploy incrementally after each phase

**Questions? Ready to start implementation?**

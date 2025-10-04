# Sprint 7: Current State Analysis

**Date:** 2025-10-04
**Status:** Pre-Sprint Planning
**Project:** Iqrah - Qur'an Learning App

## Executive Summary

The Iqrah project has successfully completed Sprint 6 with a working MVP featuring:
- FSRS-6 spaced repetition
- Knowledge graph with energy propagation
- High-yield session mode
- Session persistence and user stats

However, the current architecture is **not production-ready**. This document analyzes critical architectural debt that must be addressed in Sprint 7.

---

## Current Architecture Overview

### File Structure
```
rust/src/
├── api/mod.rs          # Public FRB API layer (mixed concerns)
├── app.rs              # Singleton app instance
├── cbor_import.rs      # Data import logic
├── database.rs         # Schema definitions
├── exercises.rs        # Exercise types
├── fsrs_utils.rs       # FSRS wrapper
├── lib.rs             # Entry point
├── propagation.rs      # Energy propagation
├── repository.rs       # Service + trait definitions
└── sqlite_repo.rs      # EVERYTHING ELSE (1,378 lines!)
```

**Total:** ~5,132 lines of Rust code
**SQL Queries:** 59+ embedded queries throughout codebase

---

## Critical Architectural Issues

### 1. **Monolithic Repository Pattern** ❌
**Problem:** `sqlite_repo.rs` is a 1,378-line god object containing:
- All SQL queries (embedded as strings)
- All business logic
- All data transformations
- No separation of concerns

**Impact:**
- Impossible to unit test in isolation
- Cannot mock database for testing
- Single point of failure
- Violates Single Responsibility Principle

### 2. **No Query Layer / Data Access Abstraction** ❌
**Problem:** SQL is embedded directly in business logic:
```rust
// Line 169-173 in sqlite_repo.rs
let query = format!(
    "SELECT ... WHERE ... ORDER BY (
        1.0 * MAX(0, (?2 - ums.due_at) / (24.0 * 60.0 * 60.0 * 1000.0)) +
        2.0 * MAX(0, 1.0 - ums.energy) +
        {} * COALESCE((SELECT CAST(value AS REAL) FROM node_metadata nm2
                      WHERE nm2.node_id = n.id AND nm2.key = '{}'), 0)
    ) DESC",
    yield_weight, importance_key
);
```

**Impact:**
- Hardcoded scoring logic mixed with SQL
- Cannot test scoring algorithm without database
- Cannot reuse queries
- Difficult to optimize or refactor

### 3. **Single Database = Multiple Concerns** ❌
**Problem:** One `iqrah.db` contains:
- Immutable content (nodes, edges, metadata)
- Mutable user progress (FSRS states, energy)
- Ephemeral session state
- User statistics

**Impact:**
- Content updates risk corrupting user data
- Cannot backup user data separately
- Cannot ship content updates independently
- Migration complexity increases exponentially

### 4. **No Dependency Injection** ❌
**Problem:** Global singleton pattern:
```rust
// app.rs
pub fn app() -> &'static App {
    APP.get().expect("App not initialized")
}

// Usage everywhere:
crate::app::app().service.get_due_items(...)
```

**Impact:**
- Cannot inject mock services for testing
- Tight coupling to concrete implementations
- Cannot run multiple instances (breaks tests)
- Violates Dependency Inversion Principle

### 5. **Inadequate Testing Infrastructure** ❌
**Current state:**
- ✅ 0 unit tests for business logic
- ✅ 0 integration tests for database layer
- ✅ 0 property-based tests
- ✅ Only manual testing via running the full app

**Impact:**
- Regressions go undetected
- Refactoring is dangerous
- Cannot verify correctness
- Slows development velocity

### 6. **Metadata Design Flaws** ❌
**Problem:** All Qur'anic content is in the knowledge graph:
- Translations stored in `node_metadata` table
- Audio URLs in metadata
- Every query must JOIN with metadata

**Impact:**
- Poor query performance (multiple JOINs for every item)
- Cannot update translations without regenerating entire graph
- Scalability issues (metadata grows linearly with nodes)

### 7. **User Data Generation Strategy** ❌
**Problem:** `sync_user_nodes()` pre-generates memory states for ALL nodes:
```rust
// Creates ~50,000+ rows on first run
INSERT INTO user_memory_states (user_id, node_id, ...)
SELECT 'user', id, ... FROM nodes
```

**Impact:**
- Massive database bloat (most nodes never studied)
- Slow initial setup
- Wasted storage and memory
- Inefficient propagation calculations

### 8. **Exercise System Limitations** ❌
**Current:** Simple enum with hardcoded types
```rust
pub enum Exercise {
    Recall { ... },
    Cloze { ... },
    McqArToEn { ... },
    McqEnToAr { ... },
}
```

**Missing:**
- Difficulty progression system
- Impact scoring based on user energy
- Exercise variant selection (a, b, c, d levels)
- Probabilistic exercise generation
- Dynamic distractor selection

### 9. **No Migration Framework** ❌
**Problem:** Schema changes require manual intervention:
- No versioning system
- No rollback capability
- No data migration tooling

**Impact:**
- Breaking changes on every update
- User data loss risk
- Cannot deploy schema updates safely

---

## Performance Bottlenecks

### Query Performance Issues
1. **Metadata JOINs:** Every session query joins `node_metadata` 3-5 times
2. **Priority Score Calculation:** Complex math done in SQL, not indexed
3. **Session Preview:** Fetches full metadata for preview (wasteful)

### Data Volume Issues
1. **User Memory States:** 50,000+ pre-generated rows (99% unused)
2. **Propagation Log:** Unbounded growth (no cleanup strategy)
3. **Session State:** Not cleaned up properly (fixed in recent commits)

---

## Technical Debt Summary

| Category | Severity | Impact | Effort to Fix |
|----------|----------|--------|---------------|
| Monolithic repository | 🔴 Critical | Cannot test, refactor, or scale | 13 SP |
| SQL embedded in logic | 🔴 Critical | Cannot optimize, test, or maintain | 8 SP |
| Single database | 🔴 Critical | Data loss risk, migration hell | 8 SP |
| No DI/testing | 🔴 Critical | Regression risk, slow development | 5 SP |
| Metadata design | 🟠 High | Performance issues | 5 SP |
| User data bloat | 🟠 High | Scalability issues | 3 SP |
| Exercise system | 🟡 Medium | Feature velocity limited | 8 SP |
| Migration system | 🔴 Critical | Cannot deploy safely | 5 SP |

**Total Technical Debt:** ~55 Story Points

---

## What Works Well ✅

Despite the issues, several components are solid:

1. **FSRS Integration:** Clean wrapper, well-tested library
2. **Energy Propagation Algorithm:** Mathematically sound, good logging
3. **Flutter Bridge:** FRB v2 works reliably
4. **Domain Models:** Types like `ReviewGrade`, `MemoryState` are well-designed
5. **Async Architecture:** Tokio + async/await used correctly

---

## Sprint 7 Mission

**Goal:** Transform from "working prototype" to "production-ready system"

**Approach:** Surgical refactoring with zero feature regression

**Success Criteria:**
- ✅ 100% test coverage for business logic
- ✅ Database split (content.db + user.db)
- ✅ Query layer abstraction
- ✅ Dependency injection
- ✅ Migration framework
- ✅ CLI tool for local development
- ✅ All existing features work identically

**Timeline:** 2-3 weeks of focused architectural work

---

## Next Steps

1. Research recommended libraries (see `01-LIBRARY-RESEARCH.md`)
2. Design new architecture (see `02-ARCHITECTURE-BLUEPRINT.md`)
3. Create migration plan (see `03-MIGRATION-STRATEGY.md`)
4. Define testing strategy (see `04-TESTING-STRATEGY.md`)
5. Execute refactoring in phases

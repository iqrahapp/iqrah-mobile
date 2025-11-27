# Task 1.6: Separate Test Data from Production Migrations

## Metadata

- **Priority:** P0 (Critical Foundation)
- **Estimated Effort:** 1 day
- **Dependencies:** Task 1.4 (Repository Refactoring)
- **Agent Type:** Refactoring + Test Infrastructure
- **Parallelizable:** No (Affects all tests)

## Goal

Refactor the database initialization system to separate production schema migrations from test sample data. Ensure production `content.db` initializes with schema only (no hardcoded test data), while maintaining backward compatibility for test suites.

## Context

The current unified migration file (`20241126000001_unified_content_schema.sql`) contains both schema definitions and sample test data (Al-Fatihah verses, translators, languages, etc.). This causes every production database to be populated with test data, which violates these principles:

1. **Production Databases Should Be Empty**: Content should come from package downloads, not migrations
2. **Test Data Should Be Explicit**: Tests should explicitly load sample data, not rely on it being baked in
3. **Scalability**: Full Quran (114 chapters, ~6000+ verses) cannot be in migrations

**Why This Matters:**
- Production applications ship with unnecessary test data in their databases
- The architecture is designed for a package download system, not hardcoded content
- When the full Quran is added, migration files become unmaintainably large
- Tests couple themselves to implicit sample data rather than explicit setup

**Related Documentation:**
- [Architecture Overview](/CLAUDE.md) - Two-database design
- [Content DB Schema](/docs/content-db-schema.md)

## Current State

**Location:** `rust/crates/iqrah-storage/migrations_content/20241126000001_unified_content_schema.sql`

**Current Contents:**
- Lines 163-207: Schema tables (chapters, verses, words, languages, translators, etc.) ✅
- Lines 213-273: Sample data (7 languages, 5 translators, Al-Fatihah verses, words, translations) ❌
- Lines 320-327: Test goal ("memorization:chapters-1-3") ❌

**Test Initialization:**
```rust
// rust/crates/iqrah-storage/src/content/mod.rs
pub async fn init_content_db(db_path: &str) -> Result<SqlitePool> {
    // Runs ALL migrations (including sample data)
    sqlx::migrate!("./migrations_content").run(&pool).await?;
    Ok(pool)
}

// Tests call this directly and get sample data implicitly
let pool = init_content_db(":memory:").await.unwrap();
```

## Target State

### 1. Migration File (Schema Only)

**File:** `rust/crates/iqrah-storage/migrations_content/20241126000001_unified_content_schema.sql`

Remove all INSERT statements. Keep only:
- Schema version table
- Core Quranic structure (chapters, verses, words)
- Translation infrastructure (languages, translators, translations)
- Scheduler v2 tables (goals, node_goals, nodes, edges, metadata)

Result: ~150 lines instead of 350+

### 2. Test Data Module

**File:** `rust/crates/iqrah-storage/src/test_data.rs` (NEW)

```rust
#[cfg(test)]
pub mod test_data {
    use sqlx::SqlitePool;

    /// Seeds the sample data used by integration tests.
    ///
    /// Inserts:
    /// - 7 sample languages (English, Arabic, French, Urdu, Indonesian, Turkish, Spanish)
    /// - 5 sample translators (Sahih International, Yusuf Ali, Pickthall, Khattab, Hilali-Khan)
    /// - Al-Fatihah (Chapter 1) with 7 verses and full Arabic text
    /// - 4 words for verse 1:1 with transliterations
    /// - Verse translations for 1:1 from all 5 translators
    /// - Word-by-word translations for Sahih International
    /// - Chapters 2-3 with placeholder verses (for scheduler tests)
    /// - Scheduler goal "memorization:chapters-1-3"
    pub async fn seed_sample_data(pool: &SqlitePool) -> Result<()> {
        // Implementation: All INSERTs from current migration file
    }
}
```

### 3. Test Initialization Helper

**File:** `rust/crates/iqrah-storage/src/lib.rs` or new `src/test_helpers.rs`

```rust
#[cfg(test)]
pub async fn init_test_content_db(db_path: &str) -> Result<SqlitePool> {
    let pool = init_content_db(db_path).await?;
    test_data::seed_sample_data(&pool).await?;
    Ok(pool)
}
```

### 4. Updated Test Code

```rust
// BEFORE: Tests rely on implicit sample data
let pool = init_content_db(":memory:").await.unwrap();

// AFTER: Tests explicitly seed data
let pool = init_test_content_db(":memory:").await.unwrap();
```

## Implementation Steps

### Step 1: Extract Sample Data INSERT Statements (1 hour)

**File:** `rust/crates/iqrah-storage/migrations_content/20241126000001_unified_content_schema.sql`

Copy all INSERT statements to a temporary location. They will become the body of `test_data::seed_sample_data()`.

### Step 2: Create Test Data Module (1.5 hours)

**File:** `rust/crates/iqrah-storage/src/test_data.rs` (NEW)

Create test-only module that:
1. Defines `seed_sample_data()` function
2. Executes all extracted INSERT statements
3. Handles any errors gracefully
4. Is marked with `#[cfg(test)]`

**Example structure:**
```rust
#[cfg(test)]
pub mod test_data {
    use sqlx::SqlitePool;
    use crate::Result;

    pub async fn seed_sample_data(pool: &SqlitePool) -> Result<()> {
        // Languages
        sqlx::query("INSERT INTO languages (language_code, ...) VALUES ...")
            .execute(pool)
            .await?;

        // Translators
        sqlx::query("INSERT INTO translators (slug, ...) VALUES ...")
            .execute(pool)
            .await?;

        // ... rest of INSERTs

        Ok(())
    }
}
```

### Step 3: Create Test Helper Function (30 min)

**File:** `rust/crates/iqrah-storage/src/content/mod.rs`

Add to `content` module:
```rust
#[cfg(test)]
pub async fn init_test_content_db(db_path: &str) -> Result<SqlitePool> {
    let pool = init_content_db(db_path).await?;
    crate::test_data::seed_sample_data(&pool).await?;
    Ok(pool)
}
```

### Step 4: Remove Test Data from Migration (1 hour)

**File:** `rust/crates/iqrah-storage/migrations_content/20241126000001_unified_content_schema.sql`

Delete lines containing:
- All `INSERT INTO languages ...`
- All `INSERT INTO translators ...`
- All `INSERT INTO verses ... VALUES ('1:1', ...` (keep only in `test_data.rs`)
- All `INSERT INTO words ...`
- All `INSERT INTO verse_translations ...`
- All `INSERT INTO word_translations ...`
- All `INSERT INTO goals ...` (test goal only)
- All `INSERT INTO node_goals ...` (test goal only)

Keep only schema (CREATE TABLE) and necessary setup for Chapters 2-3 placeholder generation.

### Step 5: Update All Test Files (1.5 hours)

Search and replace in test files:
```rust
// Find all:
let pool = init_content_db(":memory:").await.unwrap();

// Replace with:
let pool = init_test_content_db(":memory:").await.unwrap();
```

Files to update:
- `rust/crates/iqrah-storage/tests/integration_tests.rs`
- `rust/crates/iqrah-storage/tests/node_id_repository_test.rs`
- `rust/crates/iqrah-storage/tests/version_test.rs`
- `rust/crates/iqrah-cli/tests/scheduler_integration.rs`
- Any other test files using `init_content_db`

### Step 6: Verify Production Path (30 min)

Ensure production code never calls test functions:
```bash
cd rust
grep -r "init_test_content_db" src/ --exclude-dir=tests
# Should return 0 results
```

## Verification Plan

### Unit Tests

- [ ] `test_data::seed_sample_data()` can be called successfully
- [ ] `init_test_content_db()` initializes empty schema then populates with data
- [ ] `init_content_db()` produces empty schema (no sample data)

### Integration Tests

- [ ] All existing integration tests pass with updated initialization
- [ ] Tests correctly find Al-Fatihah verses after seeding
- [ ] Scheduler tests correctly find 493 verses after seeding
- [ ] Language and translator queries return expected sample data

### Verification Queries

```bash
# After running init_content_db (production)
# Should have zero rows:
SELECT COUNT(*) FROM verses;
SELECT COUNT(*) FROM languages;

# After running init_test_content_db (tests)
# Should have expected sample data:
SELECT COUNT(*) FROM verses;  # 493 (7 + 286 + 200)
SELECT COUNT(*) FROM languages;  # 7
SELECT COUNT(*) FROM translators;  # 5
```

### CI Validation

All tests must pass:
```bash
cd rust
RUSTFLAGS="-D warnings" cargo build --all-features
cargo clippy --all-features --all-targets -- -D warnings
cargo test --all-features
cargo fmt --all -- --check
```

## Scope Limits & Safeguards

### ✅ MUST DO

- Move **ALL** sample data INSERT statements to `test_data.rs`
- Keep migration file **schema-only**
- Create `init_test_content_db()` helper function
- Update **ALL** test files to use new initialization
- Ensure **ZERO** sample data in production initialization
- Keep test data identical (same verses, translators, etc.)

### ❌ DO NOT

- Change the actual sample data (verses, translations, etc.)
- Modify `init_content_db()` public API
- Remove sample data entirely (tests need it)
- Break any existing tests
- Attempt to refactor the data itself (data quality is separate)

### ⚠️ If Uncertain

- Ask: Should library initialization functions ever include test data?
- Answer: No. Libraries should initialize clean. Tests should setup their own data.
- Ask: Will this break production apps?
- Answer: No. Production apps use `init_content_db()` → get empty schema → packages provide data.

## Success Criteria

- [ ] Migration file is schema-only (~150 lines, no INSERTs)
- [ ] `test_data.rs` module exists and contains all sample data INSERTs
- [ ] `init_test_content_db()` function works for all tests
- [ ] All integration tests pass
- [ ] All unit tests pass
- [ ] `init_content_db()` produces empty schema (verified manually)
- [ ] `init_test_content_db()` produces schema + sample data
- [ ] CI checks pass: build, clippy, test, fmt
- [ ] No `init_test_content_db` calls in production code (`src/` directory)
- [ ] Documentation updated (if applicable)

## Related Files

- **Migration File:** `rust/crates/iqrah-storage/migrations_content/20241126000001_unified_content_schema.sql` (to modify)
- **New Module:** `rust/crates/iqrah-storage/src/test_data.rs` (to create)
- **Test Files:** `tests/integration_tests.rs`, `tests/node_id_repository_test.rs`, etc. (to update)
- **CLI Tests:** `rust/crates/iqrah-cli/tests/scheduler_integration.rs` (to update)
- **Storage Lib:** `rust/crates/iqrah-storage/src/lib.rs` or `src/content/mod.rs` (to add helper)

## Notes

### Why After Task 1.4?

Task 1.4 refactors the repository to use the new integer-based architecture. That refactoring is complete and tested before we modify test infrastructure. This ensures we're not debugging two things at once.

### Production Safety

The key safety mechanism: No code in `src/` (production) ever imports `test_data` module. It's marked `#[cfg(test)]` so it only exists in test builds.

### Future Consideration

When the full Quran is implemented, the sample data will be small and focused (just Al-Fatihah for basic tests), while full data comes from package downloads.

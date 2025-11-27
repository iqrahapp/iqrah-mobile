# Task 1.6: Schema v2.1 Redesign & Test Data Separation

## Metadata

- **Priority:** P0 (Critical Foundation - Performance & Architecture)
- **Estimated Effort:** 2 days
- **Dependencies:** Task 1.4 (i64 ID Refactoring)
- **Agent Type:** Schema Migration + Performance Optimization + Test Infrastructure
- **Parallelizable:** No (Affects all content DB operations)

## Goal

Implement a performance-optimized schema redesign (v2.1) for the content database that:

1. **Removes hardcoded text columns** from `verses` and `words` tables
2. **Introduces resource-based content storage** with INTEGER-optimized foreign keys
3. **Separates test data** from production schema migrations
4. **Maintains full test compatibility** with explicit data seeding

This redesign enables efficient content package downloads, reduces database size, and optimizes SQLite B-Tree indexing for high-read-volume queries.

## Context

### The Problem

**Current Schema (v2.0):**
```sql
CREATE TABLE verses (
    verse_key TEXT PRIMARY KEY,
    text_uthmani TEXT NOT NULL,     -- ❌ Hardcoded content
    text_simple TEXT,                -- ❌ Hardcoded content
    ...
);

CREATE TABLE words (
    word_id INTEGER PRIMARY KEY,
    text_uthmani TEXT NOT NULL,     -- ❌ Hardcoded content
    text_simple TEXT,                -- ❌ Hardcoded content
    transliteration TEXT,            -- ❌ Hardcoded content
    ...
);
```

**Issues:**
1. **Schema Rigidity**: Adding new script variants (Indopak, Warsh, etc.) requires schema changes
2. **Storage Bloat**: Duplicate text content for every verse/word in multiple scripts
3. **Performance**: Text columns repeated across millions of rows (6000+ verses × multiple scripts)
4. **Package System**: Architecture designed for package downloads, but schema hardcodes content
5. **Test Data**: Production databases ship with 493 sample verses baked into migrations

### The Solution: Resource Pattern with Integer Optimization

**New Schema (v2.1):**
```sql
-- 1. Metadata only (structural information)
CREATE TABLE verses (
    verse_key TEXT PRIMARY KEY,
    chapter_number INTEGER NOT NULL,
    verse_number INTEGER NOT NULL,
    -- NO text columns
);

-- 2. Resource Registry (INTEGER PK for join performance)
CREATE TABLE script_resources (
    resource_id INTEGER PRIMARY KEY,  -- ✅ Auto-increment
    slug TEXT NOT NULL UNIQUE,        -- 'uthmani', 'simple', 'indopak'
    type INTEGER NOT NULL,            -- ✅ Enum: 1=Text, 2=Vector, 3=Image
);

-- 3. Unified Content Storage (HIGH PERFORMANCE)
CREATE TABLE script_contents (
    resource_id INTEGER NOT NULL,  -- ✅ INTEGER FK (fast joins)
    node_id INTEGER NOT NULL,      -- ✅ INTEGER FK (fast joins)
    text_content TEXT NOT NULL,
    PRIMARY KEY (resource_id, node_id)  -- Composite key
) WITHOUT ROWID;
```

**Why This Matters:**

1. **Performance**: INTEGER foreign keys → faster B-Tree lookups (SQLite optimization)
2. **Storage**: No duplicate text columns → smaller database files
3. **Scalability**: Add scripts without schema changes (INSERT into `script_resources`)
4. **Package System**: Content loaded via packages, not hardcoded in migrations
5. **Flexibility**: Different resource types (text, vectors, audio) in unified table
6. **Test Data Separation**: Production schema clean, tests explicitly seed data

**Related Documentation:**
- [Architecture Overview](/CLAUDE.md) - Two-database design
- [Task 1.4](/docs/todo/production-ready-tasks/task-1.4-refactor-repository-integer-ids.md) - i64 encoding

## Current State

**Location:** `rust/crates/iqrah-storage/migrations_content/20241126000001_unified_content_schema.sql`

**Current Schema (v2.0):**
- `verses` table with `text_uthmani` and `text_simple` columns (lines 51-70)
- `words` table with `text_uthmani`, `text_simple`, `transliteration` (lines 72-82)
- Hardcoded sample data: 7 verses, 4 words, translations (lines 170-273)
- Test goal data: "memorization:chapters-1-3" (lines 320-327)
- No resource abstraction - content is schema-locked

**Current Code:**
```rust
// rust/crates/iqrah-storage/src/content/mod.rs
pub async fn init_content_db(db_path: &str) -> Result<SqlitePool> {
    // Runs migrations including sample data
    sqlx::migrate!("./migrations_content").run(&pool).await?;
    Ok(pool)
}

// Tests implicitly get sample data
let pool = init_content_db(":memory:").await.unwrap();
```

## Target State

### 1. New Schema Migration (v2.1)

**File:** `rust/crates/iqrah-storage/migrations_content/20241126000001_unified_content_schema.sql` (REWRITE)

**Schema-only migration (~200 lines):**

```sql
-- ============================================================================
-- Content Database Schema v2.1
-- Date: 2025-11-27
-- Performance-optimized resource pattern with INTEGER foreign keys
-- ============================================================================

-- ============================================================================
-- SCHEMA VERSION TRACKING
-- ============================================================================
CREATE TABLE schema_version (
    version TEXT NOT NULL PRIMARY KEY,
    description TEXT NOT NULL,
    applied_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

INSERT INTO schema_version (version, description)
VALUES ('2.1.0', 'Resource-optimized schema with INTEGER FKs and separated content storage');

-- ============================================================================
-- NODES REGISTRY (Central authority for all content nodes)
-- ============================================================================
CREATE TABLE nodes (
    id INTEGER PRIMARY KEY,
    ukey TEXT NOT NULL UNIQUE,
    node_type INTEGER NOT NULL
) STRICT;

CREATE INDEX idx_nodes_ukey ON nodes(ukey);

-- ============================================================================
-- CORE QURANIC STRUCTURE (Metadata only, NO text content)
-- ============================================================================

-- Chapters: Structural metadata
CREATE TABLE chapters (
    chapter_number INTEGER PRIMARY KEY,
    name_arabic TEXT NOT NULL,
    name_transliteration TEXT NOT NULL,
    name_translation TEXT NOT NULL,
    revelation_place TEXT CHECK (revelation_place IN ('makkah', 'madinah') OR revelation_place IS NULL),
    revelation_order INTEGER,
    bismillah_pre INTEGER NOT NULL DEFAULT 1 CHECK (bismillah_pre IN (0, 1)),
    verse_count INTEGER NOT NULL,
    page_start INTEGER,
    page_end INTEGER,
    created_at INTEGER NOT NULL DEFAULT (unixepoch())
) STRICT;

-- Verses: Structural metadata (NO text columns)
CREATE TABLE verses (
    verse_key TEXT PRIMARY KEY,
    chapter_number INTEGER NOT NULL,
    verse_number INTEGER NOT NULL,
    juz INTEGER NOT NULL,
    hizb INTEGER NOT NULL,
    rub_el_hizb INTEGER NOT NULL,
    page INTEGER NOT NULL,
    manzil INTEGER NOT NULL,
    ruku INTEGER,
    sajdah_type TEXT,
    sajdah_number INTEGER,
    letter_count INTEGER,
    word_count INTEGER,
    created_at INTEGER NOT NULL DEFAULT (unixepoch()),
    UNIQUE (chapter_number, verse_number),
    FOREIGN KEY (chapter_number) REFERENCES chapters(chapter_number)
) STRICT;

-- Words: Structural metadata (NO text columns)
CREATE TABLE words (
    word_id INTEGER PRIMARY KEY,
    verse_key TEXT NOT NULL,
    position INTEGER NOT NULL,
    letter_count INTEGER,
    created_at INTEGER NOT NULL DEFAULT (unixepoch()),
    FOREIGN KEY (verse_key) REFERENCES verses(verse_key)
) STRICT;

-- ============================================================================
-- RESOURCE PATTERN: Script Resources & Content Storage
-- ============================================================================

-- Resource Registry (Integer-optimized for join performance)
CREATE TABLE script_resources (
    resource_id INTEGER PRIMARY KEY,  -- Auto-increment
    slug TEXT NOT NULL UNIQUE,        -- 'uthmani', 'simple', 'indopak', 'transliteration'
    name TEXT NOT NULL,               -- Display name
    type INTEGER NOT NULL,            -- Enum: 1=Text, 2=Vector/JSON, 3=Image, 4=Audio
    direction TEXT DEFAULT 'rtl' CHECK (direction IN ('ltr', 'rtl')),
    description TEXT,
    created_at INTEGER NOT NULL DEFAULT (unixepoch())
) STRICT;

CREATE INDEX idx_script_resources_slug ON script_resources(slug);
CREATE INDEX idx_script_resources_type ON script_resources(type);

-- Unified Content Storage (High-performance INTEGER FKs)
CREATE TABLE script_contents (
    resource_id INTEGER NOT NULL,  -- FK to script_resources.resource_id
    node_id INTEGER NOT NULL,      -- FK to nodes.id (can be verse or word)
    text_content TEXT NOT NULL,

    PRIMARY KEY (resource_id, node_id),
    FOREIGN KEY (resource_id) REFERENCES script_resources(resource_id) ON DELETE CASCADE,
    FOREIGN KEY (node_id) REFERENCES nodes(id) ON DELETE CASCADE
) STRICT, WITHOUT ROWID;

CREATE INDEX idx_script_contents_node ON script_contents(node_id);
CREATE INDEX idx_script_contents_resource ON script_contents(resource_id);

-- ============================================================================
-- TRANSLATION INFRASTRUCTURE
-- ============================================================================

CREATE TABLE languages (
    language_code TEXT PRIMARY KEY,
    english_name TEXT NOT NULL,
    native_name TEXT NOT NULL,
    direction TEXT NOT NULL CHECK (direction IN ('ltr', 'rtl'))
) STRICT, WITHOUT ROWID;

CREATE TABLE translators (
    translator_id INTEGER PRIMARY KEY,
    slug TEXT NOT NULL UNIQUE,
    full_name TEXT NOT NULL,
    language_code TEXT NOT NULL,
    description TEXT,
    copyright_holder TEXT,
    license TEXT,
    website TEXT,
    version TEXT NOT NULL,
    package_id TEXT,
    created_at INTEGER NOT NULL DEFAULT (unixepoch()),
    FOREIGN KEY (language_code) REFERENCES languages(language_code)
) STRICT;

CREATE TABLE verse_translations (
    verse_key TEXT NOT NULL,
    translator_id INTEGER NOT NULL,
    translation TEXT NOT NULL,
    footnotes TEXT,
    created_at INTEGER NOT NULL DEFAULT (unixepoch()),
    PRIMARY KEY (verse_key, translator_id),
    FOREIGN KEY (verse_key) REFERENCES verses(verse_key),
    FOREIGN KEY (translator_id) REFERENCES translators(translator_id)
) STRICT, WITHOUT ROWID;

CREATE TABLE word_translations (
    word_id INTEGER NOT NULL,
    translator_id INTEGER NOT NULL,
    translation TEXT NOT NULL,
    created_at INTEGER NOT NULL DEFAULT (unixepoch()),
    PRIMARY KEY (word_id, translator_id),
    FOREIGN KEY (word_id) REFERENCES words(word_id),
    FOREIGN KEY (translator_id) REFERENCES translators(translator_id)
) STRICT, WITHOUT ROWID;

-- ============================================================================
-- MORPHOLOGY & LINGUISTIC ANALYSIS
-- ============================================================================

CREATE TABLE roots (
    root_id TEXT PRIMARY KEY,
    arabic TEXT NOT NULL,
    transliteration TEXT,
    root_type TEXT,
    meaning TEXT,
    created_at INTEGER NOT NULL DEFAULT (unixepoch())
) STRICT, WITHOUT ROWID;

CREATE TABLE lemmas (
    lemma_id TEXT PRIMARY KEY,
    arabic TEXT NOT NULL,
    transliteration TEXT,
    root_id TEXT,
    description TEXT,
    created_at INTEGER NOT NULL DEFAULT (unixepoch()),
    FOREIGN KEY (root_id) REFERENCES roots(root_id)
) STRICT, WITHOUT ROWID;

CREATE TABLE morphology_segments (
    segment_id INTEGER PRIMARY KEY,
    word_id INTEGER NOT NULL,
    position INTEGER NOT NULL,
    lemma_id TEXT,
    root_id TEXT,
    pos_tag TEXT,
    FOREIGN KEY (word_id) REFERENCES words(word_id),
    FOREIGN KEY (lemma_id) REFERENCES lemmas(lemma_id),
    FOREIGN KEY (root_id) REFERENCES roots(root_id)
) STRICT;

-- ============================================================================
-- SCHEDULER V2 TABLES (Knowledge Graph)
-- ============================================================================

CREATE TABLE goals (
    goal_id TEXT PRIMARY KEY,
    goal_type TEXT NOT NULL,
    goal_group TEXT NOT NULL,
    label TEXT NOT NULL,
    description TEXT
) STRICT;

CREATE TABLE node_goals (
    goal_id TEXT NOT NULL,
    node_id INTEGER NOT NULL,
    priority INTEGER DEFAULT 0,
    PRIMARY KEY (goal_id, node_id),
    FOREIGN KEY (goal_id) REFERENCES goals(goal_id) ON DELETE CASCADE,
    FOREIGN KEY (node_id) REFERENCES nodes(id) ON DELETE CASCADE
) STRICT, WITHOUT ROWID;

CREATE TABLE node_metadata (
    node_id INTEGER NOT NULL,
    key TEXT NOT NULL,
    value REAL NOT NULL,
    PRIMARY KEY (node_id, key),
    FOREIGN KEY (node_id) REFERENCES nodes(id) ON DELETE CASCADE
) STRICT, WITHOUT ROWID;

CREATE TABLE edges (
    source_id INTEGER NOT NULL,
    target_id INTEGER NOT NULL,
    edge_type INTEGER NOT NULL,
    distribution_type INTEGER NOT NULL,
    param1 REAL NOT NULL DEFAULT 0.0,
    param2 REAL NOT NULL DEFAULT 0.0,
    PRIMARY KEY (source_id, target_id),
    FOREIGN KEY (source_id) REFERENCES nodes(id) ON DELETE CASCADE,
    FOREIGN KEY (target_id) REFERENCES nodes(id) ON DELETE CASCADE
) STRICT, WITHOUT ROWID;
```

**Key Changes:**
- ❌ Removed `text_uthmani`, `text_simple`, `transliteration` from `verses` and `words`
- ✅ Added `script_resources` table with INTEGER PRIMARY KEY
- ✅ Added `script_contents` table with INTEGER foreign keys
- ✅ Schema version updated to 2.1.0
- ✅ No sample data (schema only)

### 2. Test Data Module

**File:** `rust/crates/iqrah-storage/src/test_data.rs` (NEW)

```rust
#[cfg(test)]
pub mod test_data {
    use sqlx::SqlitePool;
    use crate::Result;

    /// Seeds the sample data used by integration tests.
    ///
    /// **Data Seeded:**
    /// - 3 chapters (Al-Fatihah, Al-Baqarah, Al-Imran)
    /// - 493 verses (7 + 286 + 200)
    /// - 4 words for verse 1:1
    /// - 2 script resources (uthmani, simple)
    /// - Text content for all verses and words
    /// - 7 languages (English, Arabic, French, Urdu, Indonesian, Turkish, Spanish)
    /// - 5 translators (Sahih International, Yusuf Ali, Pickthall, Khattab, Hilali-Khan)
    /// - Verse translations for 1:1 from all translators
    /// - Word-by-word translations for Sahih International
    /// - Test goal: "memorization:chapters-1-3"
    /// - Node metadata and edges for scheduler
    ///
    /// **Order of Operations:**
    /// 1. Chapters
    /// 2. Verses (Al-Fatihah full, chapters 2-3 placeholders)
    /// 3. Words (verse 1:1 only)
    /// 4. Nodes (populate registry for verses and words)
    /// 5. Script Resources (uthmani, simple, transliteration)
    /// 6. Script Contents (link resources to nodes)
    /// 7. Languages and Translators
    /// 8. Translations (verse and word)
    /// 9. Goal and node_goals
    /// 10. Node metadata and edges
    pub async fn seed_sample_data(pool: &SqlitePool) -> Result<()> {
        // Step 1: Insert Chapters
        sqlx::query(
            "INSERT INTO chapters (chapter_number, name_arabic, name_transliteration, name_translation,
             revelation_place, revelation_order, bismillah_pre, verse_count, page_start, page_end)
             VALUES
             (1, 'الفاتحة', 'Al-Fatihah', 'The Opening', 'makkah', 5, 1, 7, 1, 1),
             (2, 'البقرة', 'Al-Baqarah', 'The Cow', 'madinah', 87, 1, 286, 2, 49),
             (3, 'آل عمران', 'Al-Imran', 'The Family of Imran', 'madinah', 89, 1, 200, 50, 76)"
        )
        .execute(pool)
        .await?;

        // Step 2: Insert Verses (Al-Fatihah with full details)
        sqlx::query(
            "INSERT INTO verses (verse_key, chapter_number, verse_number, juz, hizb, rub_el_hizb, page, manzil, word_count)
             VALUES
             ('1:1', 1, 1, 1, 1, 1, 1, 1, 4),
             ('1:2', 1, 2, 1, 1, 1, 1, 1, 4),
             ('1:3', 1, 3, 1, 1, 1, 1, 1, 2),
             ('1:4', 1, 4, 1, 1, 1, 1, 1, 3),
             ('1:5', 1, 5, 1, 1, 1, 1, 1, 4),
             ('1:6', 1, 6, 1, 1, 1, 1, 1, 3),
             ('1:7', 1, 7, 1, 1, 1, 1, 1, 10)"
        )
        .execute(pool)
        .await?;

        // Generate placeholder verses for chapters 2-3 using CTE
        sqlx::query(
            "WITH RECURSIVE chapter_verses(chapter_num, verse_num, max_verses) AS (
                SELECT 2, 1, 286
                UNION ALL
                SELECT
                  CASE WHEN verse_num < max_verses THEN chapter_num ELSE chapter_num + 1 END,
                  CASE WHEN verse_num < max_verses THEN verse_num + 1 ELSE 1 END,
                  CASE WHEN verse_num < max_verses THEN max_verses WHEN chapter_num = 2 THEN 200 ELSE 0 END
                FROM chapter_verses
                WHERE chapter_num < 3 OR (chapter_num = 3 AND verse_num < 200)
            )
            INSERT INTO verses (verse_key, chapter_number, verse_number, juz, hizb, rub_el_hizb, page, manzil, word_count)
            SELECT chapter_num || ':' || verse_num, chapter_num, verse_num,
                   CASE WHEN chapter_num = 2 THEN (verse_num / 25) + 1 ELSE 15 + (verse_num / 20) END,
                   1, 1, 1, 1, 1
            FROM chapter_verses"
        )
        .execute(pool)
        .await?;

        // Step 3: Insert Words (verse 1:1 only)
        sqlx::query(
            "INSERT INTO words (verse_key, position, letter_count) VALUES
             ('1:1', 1, 3),
             ('1:1', 2, 4),
             ('1:1', 3, 6),
             ('1:1', 4, 6)"
        )
        .execute(pool)
        .await?;

        // Step 4: Populate Nodes Registry
        // Encode verse nodes: (TYPE_VERSE << 56) | (chapter << 16) | verse
        sqlx::query(
            "INSERT OR IGNORE INTO nodes (id, ukey, node_type)
             SELECT (CAST(2 AS INTEGER) << 56) | (chapter_number << 16) | verse_number,
                    'VERSE:' || verse_key,
                    1
             FROM verses"
        )
        .execute(pool)
        .await?;

        // Encode word nodes: (TYPE_WORD << 56) | word_id
        sqlx::query(
            "INSERT OR IGNORE INTO nodes (id, ukey, node_type)
             SELECT (CAST(3 AS INTEGER) << 56) | word_id,
                    'WORD:' || word_id,
                    3
             FROM words"
        )
        .execute(pool)
        .await?;

        // Step 5: Insert Script Resources
        let uthmani_id = sqlx::query_scalar::<_, i64>(
            "INSERT INTO script_resources (slug, name, type, direction, description)
             VALUES ('uthmani', 'Uthmani Script', 1, 'rtl', 'Standard Uthmani Quranic text')
             RETURNING resource_id"
        )
        .fetch_one(pool)
        .await?;

        let simple_id = sqlx::query_scalar::<_, i64>(
            "INSERT INTO script_resources (slug, name, type, direction, description)
             VALUES ('simple', 'Simple Script', 1, 'rtl', 'Simplified Quranic text without diacritics')
             RETURNING resource_id"
        )
        .fetch_one(pool)
        .await?;

        let translit_id = sqlx::query_scalar::<_, i64>(
            "INSERT INTO script_resources (slug, name, type, direction, description)
             VALUES ('transliteration', 'Transliteration', 1, 'ltr', 'Romanized pronunciation guide')
             RETURNING resource_id"
        )
        .fetch_one(pool)
        .await?;

        // Step 6: Insert Script Contents (Uthmani text for verses)
        // For verse 1:1 through 1:7
        let verse_texts = [
            ("1:1", "بِسْمِ ٱللَّهِ ٱلرَّحْمَٰنِ ٱلرَّحِيمِ", "بسم الله الرحمن الرحيم"),
            ("1:2", "ٱلْحَمْدُ لِلَّهِ رَبِّ ٱلْعَٰلَمِينَ", "الحمد لله رب العالمين"),
            ("1:3", "ٱلرَّحْمَٰنِ ٱلرَّحِيمِ", "الرحمن الرحيم"),
            ("1:4", "مَٰلِكِ يَوْمِ ٱلدِّينِ", "مالك يوم الدين"),
            ("1:5", "إِيَّاكَ نَعْبُدُ وَإِيَّاكَ نَسْتَعِينُ", "اياك نعبد واياك نستعين"),
            ("1:6", "ٱهْدِنَا ٱلصِّرَٰطَ ٱلْمُسْتَقِيمَ", "اهدنا الصراط المستقيم"),
            ("1:7", "صِرَٰطَ ٱلَّذِينَ أَنْعَمْتَ عَلَيْهِمْ غَيْرِ ٱلْمَغْضُوبِ عَلَيْهِمْ وَلَا ٱلضَّآلِّينَ",
             "صراط الذين انعمت عليهم غير المغضوب عليهم ولا الضالين"),
        ];

        for (verse_key, uthmani, simple) in verse_texts.iter() {
            let node_id = sqlx::query_scalar::<_, i64>(
                "SELECT id FROM nodes WHERE ukey = 'VERSE:' || ?"
            )
            .bind(verse_key)
            .fetch_one(pool)
            .await?;

            // Insert uthmani text
            sqlx::query(
                "INSERT INTO script_contents (resource_id, node_id, text_content) VALUES (?, ?, ?)"
            )
            .bind(uthmani_id)
            .bind(node_id)
            .bind(uthmani)
            .execute(pool)
            .await?;

            // Insert simple text
            sqlx::query(
                "INSERT INTO script_contents (resource_id, node_id, text_content) VALUES (?, ?, ?)"
            )
            .bind(simple_id)
            .bind(node_id)
            .bind(simple)
            .execute(pool)
            .await?;
        }

        // Insert word text content (verse 1:1 words)
        let word_texts = [
            (1, "بِسْمِ", "بسم", "bismi"),
            (2, "ٱللَّهِ", "الله", "Allāhi"),
            (3, "ٱلرَّحْمَٰنِ", "الرحمن", "al-Raḥmāni"),
            (4, "ٱلرَّحِيمِ", "الرحيم", "al-Raḥīmi"),
        ];

        for (pos, uthmani, simple, translit) in word_texts.iter() {
            let word_id = sqlx::query_scalar::<_, i64>(
                "SELECT word_id FROM words WHERE verse_key = '1:1' AND position = ?"
            )
            .bind(pos)
            .fetch_one(pool)
            .await?;

            let node_id = sqlx::query_scalar::<_, i64>(
                "SELECT id FROM nodes WHERE ukey = 'WORD:' || ?"
            )
            .bind(word_id)
            .fetch_one(pool)
            .await?;

            // Uthmani
            sqlx::query(
                "INSERT INTO script_contents (resource_id, node_id, text_content) VALUES (?, ?, ?)"
            )
            .bind(uthmani_id)
            .bind(node_id)
            .bind(uthmani)
            .execute(pool)
            .await?;

            // Simple
            sqlx::query(
                "INSERT INTO script_contents (resource_id, node_id, text_content) VALUES (?, ?, ?)"
            )
            .bind(simple_id)
            .bind(node_id)
            .bind(simple)
            .execute(pool)
            .await?;

            // Transliteration
            sqlx::query(
                "INSERT INTO script_contents (resource_id, node_id, text_content) VALUES (?, ?, ?)"
            )
            .bind(translit_id)
            .bind(node_id)
            .bind(translit)
            .execute(pool)
            .await?;
        }

        // Step 7: Insert Languages
        sqlx::query(
            "INSERT INTO languages (language_code, english_name, native_name, direction) VALUES
             ('en', 'English', 'English', 'ltr'),
             ('ar', 'Arabic', 'العربية', 'rtl'),
             ('fr', 'French', 'Français', 'ltr'),
             ('ur', 'Urdu', 'اردو', 'rtl'),
             ('id', 'Indonesian', 'Indonesia', 'ltr'),
             ('tr', 'Turkish', 'Türkçe', 'ltr'),
             ('es', 'Spanish', 'Español', 'ltr')"
        )
        .execute(pool)
        .await?;

        // Step 8: Insert Translators
        sqlx::query(
            "INSERT INTO translators (slug, full_name, language_code, description, license, website, version) VALUES
             ('sahih-intl', 'Sahih International', 'en', 'Clear and modern English translation', 'Public Domain', 'https://quran.com', '1.0'),
             ('yusuf-ali', 'Abdullah Yusuf Ali', 'en', 'Classic English translation', 'Public Domain', 'https://www.al-islam.org', '1.0'),
             ('pickthall', 'Marmaduke Pickthall', 'en', 'First English translation by a Muslim', 'Public Domain', NULL, '1.0'),
             ('khattab', 'Dr. Mustafa Khattab', 'en', 'The Clear Quran', 'CC BY-NC-ND 4.0', 'https://theclearquran.org', '1.0'),
             ('hilali-khan', 'Dr. Muhsin Khan & Dr. Taqi-ud-Din al-Hilali', 'en', 'Noble Quran', 'Public Domain', NULL, '1.0')"
        )
        .execute(pool)
        .await?;

        // Step 9: Insert Verse Translations (verse 1:1 only)
        sqlx::query(
            "INSERT INTO verse_translations (verse_key, translator_id, translation)
             SELECT '1:1', translator_id, 'In the name of Allah, the Entirely Merciful, the Especially Merciful.' FROM translators WHERE slug = 'sahih-intl'
             UNION ALL SELECT '1:1', translator_id, 'In the name of God, Most Gracious, Most Merciful.' FROM translators WHERE slug = 'yusuf-ali'
             UNION ALL SELECT '1:1', translator_id, 'In the name of Allah, the Beneficent, the Merciful.' FROM translators WHERE slug = 'pickthall'
             UNION ALL SELECT '1:1', translator_id, 'In the Name of Allah—the Most Compassionate, Most Merciful.' FROM translators WHERE slug = 'khattab'
             UNION ALL SELECT '1:1', translator_id, 'In the Name of Allah, the Most Gracious, the Most Merciful.' FROM translators WHERE slug = 'hilali-khan'"
        )
        .execute(pool)
        .await?;

        // Step 10: Insert Word Translations (verse 1:1, Sahih International only)
        sqlx::query(
            "INSERT INTO word_translations (word_id, translator_id, translation)
             SELECT w.word_id, t.translator_id, 'In the name' FROM words w, translators t WHERE w.verse_key = '1:1' AND w.position = 1 AND t.slug = 'sahih-intl'
             UNION ALL SELECT w.word_id, t.translator_id, 'of Allah' FROM words w, translators t WHERE w.verse_key = '1:1' AND w.position = 2 AND t.slug = 'sahih-intl'
             UNION ALL SELECT w.word_id, t.translator_id, 'the Entirely Merciful' FROM words w, translators t WHERE w.verse_key = '1:1' AND w.position = 3 AND t.slug = 'sahih-intl'
             UNION ALL SELECT w.word_id, t.translator_id, 'the Especially Merciful' FROM words w, translators t WHERE w.verse_key = '1:1' AND w.position = 4 AND t.slug = 'sahih-intl'"
        )
        .execute(pool)
        .await?;

        // Step 11: Insert Test Goal
        sqlx::query(
            "INSERT OR IGNORE INTO goals (goal_id, goal_type, goal_group, label, description) VALUES
             ('memorization:chapters-1-3', 'custom', 'memorization', 'Memorize Chapters 1-3',
              'Master all 493 verses from Al-Fatihah, Al-Baqarah, and Al-Imran')"
        )
        .execute(pool)
        .await?;

        // Add all verses from chapters 1-3 to the goal
        sqlx::query(
            "INSERT OR IGNORE INTO node_goals (goal_id, node_id, priority)
             SELECT 'memorization:chapters-1-3', id, 1001000
             FROM nodes
             WHERE ukey LIKE 'VERSE:1:%' OR ukey LIKE 'VERSE:2:%' OR ukey LIKE 'VERSE:3:%'"
        )
        .execute(pool)
        .await?;

        // Step 12: Insert Node Metadata (foundational, influence, difficulty scores)
        sqlx::query(
            "INSERT OR IGNORE INTO node_metadata (node_id, key, value)
             SELECT n.id, 'foundational_score',
               CASE WHEN v.chapter_number = 1 AND v.verse_number = 1 THEN 0.85
                    ELSE 0.1 + (CAST(v.chapter_number AS REAL) * 0.01) + (CAST(v.verse_number AS REAL) * 0.001)
               END
             FROM nodes n
             JOIN verses v ON n.ukey = 'VERSE:' || v.verse_key
             WHERE n.ukey LIKE 'VERSE:%'
             UNION ALL
             SELECT n.id, 'influence_score',
               CASE WHEN v.chapter_number = 1 AND v.verse_number = 1 THEN 0.90
                    ELSE 0.1 + (CAST(v.chapter_number AS REAL) * 0.01) + (CAST(v.verse_number AS REAL) * 0.001)
               END
             FROM nodes n
             JOIN verses v ON n.ukey = 'VERSE:' || v.verse_key
             WHERE n.ukey LIKE 'VERSE:%'
             UNION ALL
             SELECT n.id, 'difficulty_score', 0.3 + (CAST(v.verse_number AS REAL) * 0.001)
             FROM nodes n
             JOIN verses v ON n.ukey = 'VERSE:' || v.verse_key
             WHERE n.ukey LIKE 'VERSE:%'
             UNION ALL
             SELECT n.id, 'quran_order', CAST(v.chapter_number AS INTEGER) * 1000 + CAST(v.verse_number AS INTEGER)
             FROM nodes n
             JOIN verses v ON n.ukey = 'VERSE:' || v.verse_key
             WHERE n.ukey LIKE 'VERSE:%'"
        )
        .execute(pool)
        .await?;

        // Step 13: Insert Sequential Prerequisite Edges
        sqlx::query(
            "INSERT OR IGNORE INTO edges (source_id, target_id, edge_type, distribution_type)
             SELECT curr.id AS source_id, next.id AS target_id, 0 AS edge_type, 0 AS distribution_type
             FROM nodes curr
             JOIN verses curr_v ON curr.ukey = 'VERSE:' || curr_v.verse_key
             JOIN verses next_v ON curr_v.chapter_number = next_v.chapter_number
                               AND next_v.verse_number = curr_v.verse_number + 1
             JOIN nodes next ON next.ukey = 'VERSE:' || next_v.verse_key
             WHERE curr.ukey LIKE 'VERSE:%' AND next.ukey LIKE 'VERSE:%'"
        )
        .execute(pool)
        .await?;

        Ok(())
    }
}
```

### 3. Test Initialization Helper

**File:** `rust/crates/iqrah-storage/src/content/mod.rs` (ADD)

```rust
#[cfg(test)]
pub async fn init_test_content_db(db_path: &str) -> Result<SqlitePool> {
    let pool = init_content_db(db_path).await?;
    crate::test_data::seed_sample_data(&pool).await?;
    Ok(pool)
}
```

### 4. Updated Test Code

**All test files** need to update initialization:

```rust
// BEFORE: Tests rely on implicit sample data
let pool = init_content_db(":memory:").await.unwrap();

// AFTER: Tests explicitly seed data
let pool = init_test_content_db(":memory:").await.unwrap();
```

## Implementation Steps

### Step 1: Backup Current Migration (5 min)

```bash
cd rust/crates/iqrah-storage
cp migrations_content/20241126000001_unified_content_schema.sql migrations_content/20241126000001_unified_content_schema.sql.backup
```

### Step 2: Rewrite Migration File (1.5 hours)

**File:** `rust/crates/iqrah-storage/migrations_content/20241126000001_unified_content_schema.sql`

1. Update schema version to 2.1.0
2. Remove `text_uthmani`, `text_simple`, `transliteration` from `verses` and `words`
3. Add `script_resources` table
4. Add `script_contents` table
5. Remove ALL INSERT statements (test data will move to `test_data.rs`)
6. Keep CREATE TABLE and CREATE INDEX statements only

**Verification:**
- Migration file is schema-only (~200 lines)
- No INSERT statements remain
- All tables defined with correct types
- INTEGER foreign keys for `script_contents`

### Step 3: Create Test Data Module (3 hours)

**File:** `rust/crates/iqrah-storage/src/test_data.rs` (NEW)

Implement `seed_sample_data()` function as shown in Target State section:

1. Insert chapters (3)
2. Insert verses (493: Al-Fatihah full + chapters 2-3 placeholders)
3. Insert words (4 for verse 1:1)
4. Populate nodes registry (verses + words)
5. **Insert script resources** (get INTEGER IDs)
6. **Insert script contents** (link resources to nodes using INTEGER FKs)
7. Insert languages (7)
8. Insert translators (5)
9. Insert verse translations (verse 1:1, all translators)
10. Insert word translations (verse 1:1 words, Sahih International)
11. Insert test goal
12. Insert node metadata
13. Insert edges

**Critical Order:**
- Resources BEFORE contents (need resource_id)
- Nodes BEFORE contents (need node_id)
- Use `RETURNING resource_id` to get INTEGER IDs

### Step 4: Update Storage Library (30 min)

**File:** `rust/crates/iqrah-storage/src/lib.rs`

```rust
#[cfg(test)]
pub mod test_data;

pub use content::{init_content_db};

#[cfg(test)]
pub use content::init_test_content_db;
```

**File:** `rust/crates/iqrah-storage/src/content/mod.rs`

Add test helper:

```rust
#[cfg(test)]
pub async fn init_test_content_db(db_path: &str) -> Result<SqlitePool> {
    let pool = init_content_db(db_path).await?;
    crate::test_data::seed_sample_data(&pool).await?;
    Ok(pool)
}
```

### Step 5: Update All Test Files (2 hours)

Search and replace in test files:

```bash
cd rust

# Find all test files using init_content_db
grep -r "init_content_db" crates/*/tests/ crates/*/src/ --include="*test*.rs"

# Update each file:
# - Add import: use iqrah_storage::init_test_content_db;
# - Replace: init_content_db(":memory:") → init_test_content_db(":memory:")
```

**Files to update:**
- `rust/crates/iqrah-storage/tests/integration_tests.rs`
- `rust/crates/iqrah-storage/tests/node_id_repository_test.rs`
- `rust/crates/iqrah-storage/src/content/scheduler_tests.rs`
- `rust/crates/iqrah-cli/tests/scheduler_integration.rs`
- Any other test files using content DB

### Step 6: Update Repository Methods (1.5 hours)

**File:** `rust/crates/iqrah-storage/src/content/repository.rs`

Update methods to query `script_contents` instead of hardcoded columns:

```rust
// OLD: Query text_uthmani column
pub async fn get_quran_text(&self, node_id: i64) -> Result<Option<String>> {
    let text = sqlx::query_scalar::<_, String>(
        "SELECT text_uthmani FROM verses WHERE ..."
    )
    // ...
}

// NEW: Query script_contents table
pub async fn get_quran_text(&self, node_id: i64, script: &str) -> Result<Option<String>> {
    let text = sqlx::query_scalar::<_, String>(
        "SELECT sc.text_content
         FROM script_contents sc
         JOIN script_resources sr ON sc.resource_id = sr.resource_id
         WHERE sc.node_id = ? AND sr.slug = ?"
    )
    .bind(node_id)
    .bind(script)
    .fetch_optional(&self.pool)
    .await?;
    Ok(text)
}
```

**Methods to update:**
- `get_quran_text()` - now takes `script` parameter
- `get_words_in_ayahs()` - fetch from `script_contents`
- Any other methods querying text columns

### Step 7: Update ContentRepository Trait (30 min)

**File:** `rust/crates/iqrah-core/src/ports/content_repository.rs`

Update trait signatures:

```rust
#[async_trait]
pub trait ContentRepository: Send + Sync {
    // OLD:
    // async fn get_quran_text(&self, node_id: i64) -> Result<Option<String>>;

    // NEW: Add script parameter
    async fn get_quran_text(&self, node_id: i64, script: &str) -> Result<Option<String>>;

    // Similar updates for other text-fetching methods
}
```

### Step 8: Verify Production Path (30 min)

Ensure production code never calls test functions:

```bash
cd rust
grep -r "init_test_content_db" src/ --exclude-dir=tests
# Should return 0 results (only in test code)

grep -r "test_data::" src/ --exclude-dir=tests
# Should return 0 results (only in test code)
```

### Step 9: Run All Tests (30 min)

```bash
cd rust

# Clean build
cargo clean

# Run all tests
cargo test --all-features

# Verify test results
# - All integration tests should pass
# - All unit tests should pass
# - Tests should find text content from script_contents table
```

### Step 10: Full CI Validation (30 min)

```bash
cd rust

# Build with warnings as errors
RUSTFLAGS="-D warnings" cargo build --all-features

# Clippy
cargo clippy --all-features --all-targets -- -D warnings

# All tests
cargo test --all-features

# Formatting
cargo fmt --all -- --check
```

## Verification Plan

### Schema Verification

```sql
-- Verify schema version
SELECT * FROM schema_version;  -- Should show 2.1.0

-- Verify verses table has NO text columns
PRAGMA table_info(verses);  -- Should NOT show text_uthmani or text_simple

-- Verify words table has NO text columns
PRAGMA table_info(words);  -- Should NOT show text_uthmani, text_simple, transliteration

-- Verify script_resources exists
SELECT * FROM script_resources;  -- Should be empty in production

-- Verify script_contents exists
PRAGMA table_info(script_contents);  -- Should show resource_id INTEGER, node_id INTEGER

-- Verify nodes includes word nodes
SELECT COUNT(*) FROM nodes WHERE node_type = 3;  -- Should show word nodes in tests
```

### Test Data Verification

```bash
# After running init_content_db (production)
# Should have zero rows:
SELECT COUNT(*) FROM verses;           -- 0 in production
SELECT COUNT(*) FROM script_resources; -- 0 in production
SELECT COUNT(*) FROM script_contents;  -- 0 in production

# After running init_test_content_db (tests)
# Should have expected sample data:
SELECT COUNT(*) FROM verses;           -- 493 (7 + 286 + 200)
SELECT COUNT(*) FROM words;            -- 4 (verse 1:1 only)
SELECT COUNT(*) FROM nodes WHERE node_type = 1;  -- 493 verse nodes
SELECT COUNT(*) FROM nodes WHERE node_type = 3;  -- 4 word nodes
SELECT COUNT(*) FROM script_resources; -- 3 (uthmani, simple, transliteration)
SELECT COUNT(*) FROM script_contents;  -- ~500 (493 verses × 2 scripts + 4 words × 3 scripts)
SELECT COUNT(*) FROM languages;        -- 7
SELECT COUNT(*) FROM translators;      -- 5
```

### Performance Verification

Compare query performance:

```sql
-- OLD schema (TEXT column)
SELECT text_uthmani FROM verses WHERE verse_key = '1:1';

-- NEW schema (INTEGER FK)
SELECT sc.text_content
FROM script_contents sc
JOIN script_resources sr ON sc.resource_id = sr.resource_id
WHERE sc.node_id = 144115188075856897 AND sr.slug = 'uthmani';

-- Verify INTEGER join is using indexes
EXPLAIN QUERY PLAN
SELECT sc.text_content
FROM script_contents sc
WHERE sc.resource_id = 1 AND sc.node_id = 144115188075856897;
-- Should show: SEARCH script_contents USING PRIMARY KEY
```

## Scope Limits & Safeguards

### ✅ MUST DO

- **Remove text columns** from `verses` and `words` tables
- **Add script_resources** with INTEGER PRIMARY KEY
- **Add script_contents** with INTEGER foreign keys
- **Update schema version** to 2.1.0
- **Move ALL test data** to `test_data.rs` module
- **Update all repository methods** to query `script_contents`
- **Update ContentRepository trait** with script parameter
- **Update all test files** to use `init_test_content_db()`
- **Ensure nodes table includes word nodes** (Type 3)
- **Maintain INTEGER optimization** throughout

### ❌ DO NOT

- Keep any text columns in verses/words (defeats purpose)
- Use TEXT slugs as foreign keys in script_contents (performance regression)
- Add test data to migration file (defeats separation goal)
- Break existing tests (must maintain compatibility)
- Modify production initialization (`init_content_db` stays schema-only)
- Change i64 encoding scheme (from Task 1.4)
- Remove translation infrastructure (languages, translators, etc.)

### ⚠️ If Uncertain

- Ask: Should content be stored in columns or resource table?
- Answer: Resource table with INTEGER FKs for performance and flexibility
- Ask: Will this break existing code?
- Answer: Yes, intentionally. Repository methods need script parameter updates.
- Ask: How do tests get text content now?
- Answer: Tests call `init_test_content_db()` which seeds `script_contents` table

## Success Criteria

Schema Implementation:
- [ ] Migration file is schema-only (~200 lines, no INSERTs)
- [ ] Schema version is 2.1.0
- [ ] `verses` table has NO text columns
- [ ] `words` table has NO text columns
- [ ] `script_resources` table exists with INTEGER PRIMARY KEY
- [ ] `script_contents` table exists with INTEGER foreign keys
- [ ] Nodes table includes word nodes (Type 3)

Test Data Implementation:
- [ ] `test_data.rs` module exists and is marked `#[cfg(test)]`
- [ ] `seed_sample_data()` function works and seeds all data
- [ ] Resources inserted first, IDs captured for contents
- [ ] Script contents link resources to nodes using INTEGER FKs
- [ ] `init_test_content_db()` function exists and works

Code Updates:
- [ ] All test files use `init_test_content_db()` instead of `init_content_db()`
- [ ] Repository methods query `script_contents` table
- [ ] ContentRepository trait updated with script parameters
- [ ] No test functions called from production code

CI Validation:
- [ ] `RUSTFLAGS="-D warnings" cargo build --all-features` passes
- [ ] `cargo clippy --all-features --all-targets -- -D warnings` passes
- [ ] `cargo test --all-features` passes (all tests)
- [ ] `cargo fmt --all -- --check` passes

Verification:
- [ ] Production init produces empty schema (verified manually)
- [ ] Test init produces schema + sample data (493 verses, 4 words, 3 resources)
- [ ] Text content queries work via `script_contents` table
- [ ] INTEGER join performance verified with EXPLAIN QUERY PLAN

## Related Files

- **Migration File:** `rust/crates/iqrah-storage/migrations_content/20241126000001_unified_content_schema.sql` (REWRITE)
- **Test Data Module:** `rust/crates/iqrah-storage/src/test_data.rs` (NEW)
- **Storage Library:** `rust/crates/iqrah-storage/src/lib.rs` (update exports)
- **Content Module:** `rust/crates/iqrah-storage/src/content/mod.rs` (add test helper)
- **Repository:** `rust/crates/iqrah-storage/src/content/repository.rs` (update methods)
- **Trait:** `rust/crates/iqrah-core/src/ports/content_repository.rs` (update signatures)
- **Test Files:** All files in `tests/` and `*_tests.rs` (update initialization)

## Notes

### Why Integer Foreign Keys?

SQLite is optimized for `INTEGER PRIMARY KEY` (uses ROWID internally). Using INTEGER foreign keys in `script_contents`:
1. **Faster joins**: INTEGER comparison is faster than TEXT comparison
2. **Smaller indexes**: INTEGER B-Tree nodes are more compact
3. **Better cache utilization**: Smaller keys = more keys per cache page
4. **Lower storage**: INTEGER (8 bytes) vs TEXT (variable, usually 10-20 bytes per slug)

For a table with ~6000 verses × multiple scripts = ~10K+ rows, this optimization is significant.

### Why Resource Pattern?

The resource pattern enables:
1. **Adding new scripts** without schema changes (just INSERT into `script_resources`)
2. **Package downloads** to populate content dynamically
3. **Different content types** (text, vectors, audio) in unified table
4. **Flexible querying** (get all scripts for a verse, or all verses for a script)
5. **Clean separation** between structure (verses/words) and content (script_contents)

### Migration from v2.0 to v2.1

If upgrading an existing database:
1. Create new tables (`script_resources`, `script_contents`)
2. Migrate text content: `INSERT INTO script_contents SELECT ...`
3. Drop old text columns: `ALTER TABLE verses DROP COLUMN text_uthmani` (or recreate table)
4. Update schema version to 2.1.0

However, since no production data exists yet, we can simply replace the migration file.

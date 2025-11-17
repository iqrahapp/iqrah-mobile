# Content Database Schema v2 (Purist Approach)

**Last Updated:** 2025-11-17
**Status:** Implementation Ready
**Priority:** P0 (Required before production)

## Context

The current content.db uses a generic `nodes` table with string-based `node_id` that couples the database to the knowledge graph structure. This creates maintenance issues and violates separation of concerns.

**v2 Design:** Implement a **purist relational schema** where content.db is graph-agnostic, using natural/semantic keys for all tables. The knowledge graph (separate layer) stores these content keys as properties.

## Goal

Provide an **authoritative, implementation-ready specification** for content.db v2 with:
- Natural primary keys (chapter_number, verse_key, word_id)
- Proper normalization
- Correct CHECK constraints with NULL handling
- Performance indexes
- Clear foreign key relationships
- Cascade semantics

## Schema Specification

### Metadata Tables

#### schema_version

**Purpose:** Track content database schema version for migrations and feature gating.

```sql
CREATE TABLE schema_version (
    version INTEGER PRIMARY KEY
) STRICT;

INSERT INTO schema_version (version) VALUES (2);
```

**Usage:**
```rust
let version: i32 = sqlx::query_scalar("SELECT version FROM schema_version")
    .fetch_one(&pool).await?;

if version >= 2 {
    // Use v2 features
}
```

### Core Quranic Structure (Inflexible Data)

#### chapters

**Purpose:** Metadata for each chapter (surah) of the Quran.

```sql
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

CREATE INDEX idx_chapters_revelation ON chapters(revelation_place, revelation_order);
```

**Field Details:**
- `chapter_number` - 1 to 114 (natural PK)
- `name_arabic` - "الفاتحة"
- `name_transliteration` - "Al-Fatihah"
- `name_translation` - "The Opening"
- `revelation_place` - NULL for chapters with disputed/mixed revelation
- `bismillah_pre` - 0 for chapter 9 (At-Tawbah), 1 for all others

**Critical:** `CHECK (revelation_place IN ('makkah', 'madinah') OR revelation_place IS NULL)` - NOT `IN (..., NULL)` which is always false.

#### verses

**Purpose:** Verse text and structural metadata.

```sql
CREATE TABLE verses (
    verse_key TEXT PRIMARY KEY,  -- Format: "chapter:verse" (e.g., "1:1")
    chapter_number INTEGER NOT NULL,
    verse_number INTEGER NOT NULL,
    text_uthmani TEXT NOT NULL,
    text_simple TEXT,
    juz INTEGER NOT NULL CHECK (juz BETWEEN 1 AND 30),
    hizb INTEGER NOT NULL CHECK (hizb BETWEEN 1 AND 60),
    rub_el_hizb INTEGER NOT NULL CHECK (rub_el_hizb BETWEEN 1 AND 240),
    page INTEGER NOT NULL,
    manzil INTEGER NOT NULL CHECK (manzil BETWEEN 1 AND 7),
    ruku INTEGER,
    sajdah_type TEXT CHECK (sajdah_type IN ('recommended', 'obligatory') OR sajdah_type IS NULL),
    sajdah_number INTEGER,
    letter_count INTEGER,
    word_count INTEGER,
    created_at INTEGER NOT NULL DEFAULT (unixepoch()),
    UNIQUE (chapter_number, verse_number),
    FOREIGN KEY (chapter_number) REFERENCES chapters(chapter_number)
) STRICT;

CREATE INDEX idx_verses_chapter ON verses(chapter_number, verse_number);
CREATE INDEX idx_verses_juz ON verses(juz, page);
CREATE INDEX idx_verses_page ON verses(page);
CREATE INDEX idx_verses_sajdah ON verses(sajdah_type) WHERE sajdah_type IS NOT NULL;
```

**Field Details:**
- `verse_key` - PK: "1:1", "2:255", "114:6"
- `text_uthmani` - Arabic text in Uthmani script (canonical)
- `text_simple` - Simplified Arabic (no diacritics, for search)
- `juz` - Division for recitation (30 parts)
- `hizb` - Half-juz (60 divisions)
- `rub_el_hizb` - Quarter-hizb (240 divisions)
- `manzil` - 7-day recitation division
- `sajdah_type` - Prostration type (NULL if no sajdah)

**Partial Index:** `idx_verses_sajdah` only indexes verses with sajdah (14 total) - more efficient than full index.

#### words

**Purpose:** Individual word instances within verses, with position tracking.

```sql
CREATE TABLE words (
    word_id INTEGER PRIMARY KEY AUTOINCREMENT,
    verse_key TEXT NOT NULL,
    position INTEGER NOT NULL,  -- 1-indexed position within verse
    text_uthmani TEXT NOT NULL,
    text_simple TEXT,
    transliteration TEXT,
    letter_count INTEGER,
    created_at INTEGER NOT NULL DEFAULT (unixepoch()),
    UNIQUE (verse_key, position),
    FOREIGN KEY (verse_key) REFERENCES verses(verse_key) ON DELETE CASCADE
) STRICT;

CREATE INDEX idx_words_verse ON words(verse_key, position);
CREATE INDEX idx_words_text ON words(text_simple);  -- For search
```

**Field Details:**
- `word_id` - Surrogate INTEGER PK (auto-increment)
- `verse_key` - FK to verses ("1:1")
- `position` - 1-indexed (first word = 1)
- `UNIQUE (verse_key, position)` - No duplicate positions

**Example:**
```sql
INSERT INTO words (verse_key, position, text_uthmani, text_simple) VALUES
    ('1:1', 1, 'بِسْمِ', 'بسم'),
    ('1:1', 2, 'ٱللَّهِ', 'الله'),
    ('1:1', 3, 'ٱلرَّحْمَٰنِ', 'الرحمن'),
    ('1:1', 4, 'ٱلرَّحِيمِ', 'الرحيم');
```

### Morphological Data (Inflexible)

#### lemmas

**Purpose:** Lemma forms (dictionary headwords) for morphological analysis.

```sql
CREATE TABLE lemmas (
    lemma_id TEXT PRIMARY KEY,  -- Semantic key (e.g., "kataba", "dakhala")
    arabic TEXT NOT NULL UNIQUE,
    transliteration TEXT,
    root_id TEXT,
    description TEXT,
    created_at INTEGER NOT NULL DEFAULT (unixepoch()),
    FOREIGN KEY (root_id) REFERENCES roots(root_id)
) STRICT;

CREATE INDEX idx_lemmas_root ON lemmas(root_id);
```

**Field Details:**
- `lemma_id` - Semantic PK (e.g., "kataba" for كَتَبَ)
- `arabic` - Arabic text, UNIQUE
- `root_id` - FK to roots (e.g., "ktb")

#### roots

**Purpose:** Morphological roots (trilateral/quadrilateral).

```sql
CREATE TABLE roots (
    root_id TEXT PRIMARY KEY,  -- Semantic key (e.g., "ktb", "drs")
    arabic TEXT NOT NULL UNIQUE,
    transliteration TEXT,
    root_type TEXT DEFAULT 'trilateral' CHECK (root_type IN ('trilateral', 'quadrilateral', 'quinqueliteral') OR root_type IS NULL),
    meaning TEXT,
    created_at INTEGER NOT NULL DEFAULT (unixepoch())
) STRICT;
```

**Field Details:**
- `root_id` - PK (e.g., "ktb" for ك-ت-ب)
- `arabic` - Arabic root letters, UNIQUE
- `root_type` - Most are trilateral (3 letters)

#### stems

**Purpose:** Stem patterns (templates) applied to roots.

```sql
CREATE TABLE stems (
    stem_id TEXT PRIMARY KEY,  -- Pattern identifier (e.g., "form-I", "form-IV")
    pattern TEXT NOT NULL,
    description TEXT,
    example TEXT,
    created_at INTEGER NOT NULL DEFAULT (unixepoch())
) STRICT;
```

#### morphology_segments

**Purpose:** Detailed morphological breakdown of word segments (prefixes, stems, suffixes).

```sql
CREATE TABLE morphology_segments (
    segment_id INTEGER PRIMARY KEY AUTOINCREMENT,
    word_id INTEGER NOT NULL,
    position INTEGER NOT NULL,  -- Segment position within word
    segment_type TEXT NOT NULL CHECK (segment_type IN ('prefix', 'stem', 'suffix')),
    text_arabic TEXT NOT NULL,
    lemma_id TEXT,
    root_id TEXT,
    stem_id TEXT,
    pos_tag TEXT,  -- Part of speech (noun, verb, particle, etc.)
    features TEXT, -- JSON: {"case": "nominative", "gender": "masculine", ...}
    created_at INTEGER NOT NULL DEFAULT (unixepoch()),
    UNIQUE (word_id, position),
    FOREIGN KEY (word_id) REFERENCES words(word_id) ON DELETE CASCADE,
    FOREIGN KEY (lemma_id) REFERENCES lemmas(lemma_id),
    FOREIGN KEY (root_id) REFERENCES roots(root_id),
    FOREIGN KEY (stem_id) REFERENCES stems(stem_id)
) STRICT;

CREATE INDEX idx_morphology_word ON morphology_segments(word_id, position);
CREATE INDEX idx_morphology_lemma ON morphology_segments(lemma_id);
CREATE INDEX idx_morphology_root ON morphology_segments(root_id);
```

**Field Details:**
- `features` - JSON blob for flexible morphological features (case, gender, number, mood, etc.)
- `pos_tag` - Part of speech: "noun", "verb", "particle", "pronoun", etc.

### Flexible Content (see 02-translations-and-translators-normalization.md)

#### languages

```sql
CREATE TABLE languages (
    language_code TEXT PRIMARY KEY,  -- ISO 639-1 (e.g., 'en', 'ar', 'fr')
    english_name TEXT NOT NULL,      -- 'English', 'Arabic', 'French'
    native_name TEXT NOT NULL,       -- 'English', 'العربية', 'Français'
    direction TEXT NOT NULL DEFAULT 'ltr' CHECK (direction IN ('ltr', 'rtl'))
) STRICT;
```

#### translators

```sql
CREATE TABLE translators (
    translator_id INTEGER PRIMARY KEY AUTOINCREMENT,
    slug TEXT NOT NULL UNIQUE,           -- 'sahih-intl', 'yusuf-ali' (URL-safe)
    full_name TEXT NOT NULL,             -- 'Sahih International', 'Abdullah Yusuf Ali'
    language_code TEXT NOT NULL,
    description TEXT,
    copyright_holder TEXT,
    license TEXT,
    website TEXT,
    version TEXT DEFAULT '1.0',
    created_at INTEGER NOT NULL DEFAULT (unixepoch()),
    FOREIGN KEY (language_code) REFERENCES languages(language_code)
) STRICT;

CREATE INDEX idx_translators_language ON translators(language_code);
```

#### verse_translations

```sql
CREATE TABLE verse_translations (
    verse_key TEXT NOT NULL,
    translator_id INTEGER NOT NULL,
    translation TEXT NOT NULL,
    footnotes TEXT,  -- Optional commentary/footnotes
    created_at INTEGER NOT NULL DEFAULT (unixepoch()),
    PRIMARY KEY (verse_key, translator_id),
    FOREIGN KEY (verse_key) REFERENCES verses(verse_key) ON DELETE CASCADE,
    FOREIGN KEY (translator_id) REFERENCES translators(translator_id) ON DELETE CASCADE
) WITHOUT ROWID, STRICT;

CREATE INDEX idx_verse_translations_translator ON verse_translations(translator_id);
```

**Cascade Semantics:**
- Deleting a verse → cascades to its translations ✅
- Deleting a translator → cascades to all their translations ✅

#### word_translations

```sql
CREATE TABLE word_translations (
    word_id INTEGER NOT NULL,
    translator_id INTEGER NOT NULL,
    translation TEXT NOT NULL,
    created_at INTEGER NOT NULL DEFAULT (unixepoch()),
    PRIMARY KEY (word_id, translator_id),
    FOREIGN KEY (word_id) REFERENCES words(word_id) ON DELETE CASCADE,
    FOREIGN KEY (translator_id) REFERENCES translators(translator_id) ON DELETE CASCADE
) WITHOUT ROWID, STRICT;

CREATE INDEX idx_word_translations_translator ON word_translations(translator_id);
```

### Package Management (see 03-flexible-content-packages-plan.md)

#### content_packages

```sql
CREATE TABLE content_packages (
    package_id TEXT PRIMARY KEY,  -- e.g., 'sahih-intl-v1', 'mishary-rashid-audio'
    package_type TEXT NOT NULL CHECK (package_type IN (
        'verse_translation',
        'word_translation',
        'text_variant',
        'verse_recitation',
        'word_audio',
        'transliteration'
    )),
    name TEXT NOT NULL,
    language_code TEXT,
    author TEXT,
    version TEXT NOT NULL,
    description TEXT,
    file_size INTEGER,  -- Bytes
    download_url TEXT,
    checksum TEXT,      -- SHA256 for integrity verification
    license TEXT,
    created_at INTEGER NOT NULL DEFAULT (unixepoch()),
    updated_at INTEGER,
    FOREIGN KEY (language_code) REFERENCES languages(language_code)
) STRICT;

CREATE INDEX idx_content_packages_type_lang ON content_packages(package_type, language_code);
```

#### installed_packages

```sql
CREATE TABLE installed_packages (
    package_id TEXT PRIMARY KEY,
    installed_at INTEGER NOT NULL DEFAULT (unixepoch()),
    enabled INTEGER NOT NULL DEFAULT 1 CHECK (enabled IN (0, 1)),
    FOREIGN KEY (package_id) REFERENCES content_packages(package_id) ON DELETE CASCADE
) STRICT;
```

**Cascade Semantics:**
- Deleting a package from catalog → removes installation record ✅

### Text Variants & Transliterations

#### text_variants

**Purpose:** Alternative Arabic scripts (Imlaei, Indopak, simplified).

```sql
CREATE TABLE text_variants (
    package_id TEXT NOT NULL,
    variant_type TEXT NOT NULL CHECK (variant_type IN ('imlaei', 'indopak', 'simple')),
    verse_key TEXT,
    word_id INTEGER,
    text TEXT NOT NULL,
    created_at INTEGER NOT NULL DEFAULT (unixepoch()),
    CHECK (
        (verse_key IS NOT NULL AND word_id IS NULL) OR
        (verse_key IS NULL AND word_id IS NOT NULL)
    ),
    FOREIGN KEY (package_id) REFERENCES content_packages(package_id) ON DELETE CASCADE,
    FOREIGN KEY (verse_key) REFERENCES verses(verse_key) ON DELETE CASCADE,
    FOREIGN KEY (word_id) REFERENCES words(word_id) ON DELETE CASCADE
) STRICT;

-- Partial unique indexes to enforce XOR uniqueness
CREATE UNIQUE INDEX idx_text_variants_verse
    ON text_variants(package_id, verse_key, variant_type)
    WHERE verse_key IS NOT NULL;

CREATE UNIQUE INDEX idx_text_variants_word
    ON text_variants(package_id, word_id, variant_type)
    WHERE word_id IS NOT NULL;

CREATE INDEX idx_text_variants_package ON text_variants(package_id);
```

**XOR Semantics:** Either `verse_key` OR `word_id` is set, never both, never neither.

**Partial Unique Indexes:** Guarantee uniqueness within the applicable scope (verse-level or word-level).

#### word_transliterations

```sql
CREATE TABLE word_transliterations (
    word_id INTEGER NOT NULL,
    package_id TEXT NOT NULL,
    transliteration TEXT NOT NULL,
    transliteration_style TEXT DEFAULT 'ala-lc' CHECK (
        transliteration_style IN ('ala-lc', 'din', 'iso', 'buckwalter') OR
        transliteration_style IS NULL
    ),
    created_at INTEGER NOT NULL DEFAULT (unixepoch()),
    PRIMARY KEY (word_id, package_id),
    FOREIGN KEY (word_id) REFERENCES words(word_id) ON DELETE CASCADE,
    FOREIGN KEY (package_id) REFERENCES content_packages(package_id) ON DELETE CASCADE
) WITHOUT ROWID, STRICT;

CREATE INDEX idx_word_transliterations_package ON word_transliterations(package_id);
```

### Audio & Recitations

#### reciters

```sql
CREATE TABLE reciters (
    reciter_id TEXT PRIMARY KEY,  -- e.g., 'mishary-rashid', 'abdul-basit'
    name TEXT NOT NULL,
    language_code TEXT,  -- Native language of reciter
    style TEXT CHECK (style IN ('murattal', 'mujawwad', 'muallim') OR style IS NULL),
    description TEXT,
    website TEXT,
    created_at INTEGER NOT NULL DEFAULT (unixepoch()),
    FOREIGN KEY (language_code) REFERENCES languages(language_code)
) STRICT;
```

**Recitation Styles:**
- `murattal` - Slow, measured recitation
- `mujawwad` - Melodic, beautified recitation
- `muallim` - Educational/teaching recitation

#### verse_recitations

```sql
CREATE TABLE verse_recitations (
    package_id TEXT NOT NULL,
    verse_key TEXT NOT NULL,
    reciter_id TEXT NOT NULL,
    audio_file TEXT NOT NULL,  -- Relative path or URL
    duration INTEGER,          -- Milliseconds
    file_size INTEGER,         -- Bytes
    format TEXT DEFAULT 'mp3' CHECK (format IN ('mp3', 'opus', 'aac')),
    created_at INTEGER NOT NULL DEFAULT (unixepoch()),
    PRIMARY KEY (package_id, verse_key),
    FOREIGN KEY (package_id) REFERENCES content_packages(package_id) ON DELETE CASCADE,
    FOREIGN KEY (verse_key) REFERENCES verses(verse_key) ON DELETE CASCADE,
    FOREIGN KEY (reciter_id) REFERENCES reciters(reciter_id)
) WITHOUT ROWID, STRICT;

CREATE INDEX idx_verse_recitations_reciter ON verse_recitations(reciter_id);
```

#### word_audio

```sql
CREATE TABLE word_audio (
    package_id TEXT NOT NULL,
    word_id INTEGER NOT NULL,
    reciter_id TEXT NOT NULL,
    audio_file TEXT NOT NULL,
    duration INTEGER,
    file_size INTEGER,
    format TEXT DEFAULT 'mp3' CHECK (format IN ('mp3', 'opus', 'aac')),
    created_at INTEGER NOT NULL DEFAULT (unixepoch()),
    PRIMARY KEY (package_id, word_id),
    FOREIGN KEY (package_id) REFERENCES content_packages(package_id) ON DELETE CASCADE,
    FOREIGN KEY (word_id) REFERENCES words(word_id) ON DELETE CASCADE,
    FOREIGN KEY (reciter_id) REFERENCES reciters(reciter_id)
) WITHOUT ROWID, STRICT;

CREATE INDEX idx_word_audio_reciter ON word_audio(reciter_id);
```

## Cascade Semantics Summary

| Parent Table | Child Table | ON DELETE | Rationale |
|--------------|-------------|-----------|-----------|
| `chapters` | `verses` | CASCADE | Verse cannot exist without chapter |
| `verses` | `words` | CASCADE | Word cannot exist without verse |
| `words` | `morphology_segments` | CASCADE | Morphology tied to word |
| `translators` | `verse_translations` | CASCADE | Remove all translations by deleted translator |
| `translators` | `word_translations` | CASCADE | Same as above |
| `content_packages` | `installed_packages` | CASCADE | Installation record depends on package |
| `content_packages` | `text_variants` | CASCADE | Variants shipped with package |
| `content_packages` | `verse_recitations` | CASCADE | Audio shipped with package |
| `content_packages` | `word_audio` | CASCADE | Same as above |
| `content_packages` | `word_transliterations` | CASCADE | Same as above |
| `verses` | `verse_translations` | CASCADE | Translation tied to verse |
| `words` | `word_translations` | CASCADE | Translation tied to word |
| `words` | `text_variants` | CASCADE | Variant tied to word |
| `words` | `word_audio` | CASCADE | Audio tied to word |
| `reciters` | `verse_recitations` | RESTRICT | Keep reciter metadata even if audio removed |
| `reciters` | `word_audio` | RESTRICT | Same as above |

**RESTRICT vs CASCADE:**
- CASCADE: Child data is meaningless without parent (e.g., translation without verse)
- RESTRICT: Parent has independent value (e.g., reciter bio remains useful)

## Implementation Steps

### Step 1: Create New Migration Files

**Location:** `rust/crates/iqrah-storage/migrations_content/`

**Tasks:**
1. Archive old migration: `mv 20241116000001_content_schema.sql 20241116000001_content_schema_v1_archived.sql`
2. Create new migration: `20241117000001_content_schema_v2_purist.sql`
3. Copy all table CREATE statements from this document into the migration file
4. Add all indexes
5. Insert sample data for testing:
   ```sql
   -- Sample language
   INSERT INTO languages VALUES ('en', 'English', 'English', 'ltr');

   -- Sample chapter
   INSERT INTO chapters (chapter_number, name_arabic, name_transliteration, name_translation, revelation_place, verse_count)
   VALUES (1, 'الفاتحة', 'Al-Fatihah', 'The Opening', 'makkah', 7);

   -- Sample verse
   INSERT INTO verses (verse_key, chapter_number, verse_number, text_uthmani, text_simple, juz, hizb, rub_el_hizb, page, manzil, word_count)
   VALUES ('1:1', 1, 1, 'بِسْمِ ٱللَّهِ ٱلرَّحْمَٰنِ ٱلرَّحِيمِ', 'بسم الله الرحمن الرحيم', 1, 1, 1, 1, 1, 4);
   ```

### Step 2: Update Repository Layer

**Location:** `rust/crates/iqrah-storage/src/content/`

**Tasks:**
1. Update `models.rs`:
   ```rust
   #[derive(sqlx::FromRow, Debug, Clone)]
   pub struct ChapterRow {
       pub chapter_number: i32,
       pub name_arabic: String,
       pub name_transliteration: String,
       pub name_translation: String,
       pub revelation_place: Option<String>,
       pub revelation_order: Option<i32>,
       pub verse_count: i32,
   }

   #[derive(sqlx::FromRow, Debug, Clone)]
   pub struct VerseRow {
       pub verse_key: String,
       pub chapter_number: i32,
       pub verse_number: i32,
       pub text_uthmani: String,
       pub text_simple: Option<String>,
       pub juz: i32,
       pub page: i32,
       // ...
   }

   #[derive(sqlx::FromRow, Debug, Clone)]
   pub struct WordRow {
       pub word_id: i32,
       pub verse_key: String,
       pub position: i32,
       pub text_uthmani: String,
       pub text_simple: Option<String>,
   }
   ```

2. Update `repository.rs`:
   ```rust
   async fn get_chapter(&self, chapter_number: i32) -> Result<Option<Chapter>> {
       query_as::<_, ChapterRow>(
           "SELECT * FROM chapters WHERE chapter_number = ?"
       )
       .bind(chapter_number)
       .fetch_optional(&self.pool)
       .await
   }

   async fn get_verse(&self, verse_key: &str) -> Result<Option<Verse>> {
       query_as::<_, VerseRow>(
           "SELECT * FROM verses WHERE verse_key = ?"
       )
       .bind(verse_key)
       .fetch_optional(&self.pool)
       .await
   }

   async fn get_words_for_verse(&self, verse_key: &str) -> Result<Vec<Word>> {
       query_as::<_, WordRow>(
           "SELECT * FROM words WHERE verse_key = ? ORDER BY position"
       )
       .bind(verse_key)
       .fetch_all(&self.pool)
       .await
   }
   ```

3. Remove old node-based methods:
   - Remove `get_node(node_id)`
   - Remove `get_quran_text(node_id)`
   - Replace with domain-specific methods (get_chapter, get_verse, get_word)

### Step 3: Update Domain Layer

**Location:** `rust/crates/iqrah-core/src/domain/models.rs`

**Tasks:**
1. Add domain models:
   ```rust
   pub struct Chapter {
       pub number: i32,
       pub name_arabic: String,
       pub name_transliteration: String,
       pub name_translation: String,
       pub revelation_place: Option<RevelationPlace>,
       pub verse_count: i32,
   }

   pub enum RevelationPlace {
       Makkah,
       Madinah,
   }

   pub struct Verse {
       pub key: String,  // "1:1"
       pub chapter_number: i32,
       pub verse_number: i32,
       pub text_uthmani: String,
       pub text_simple: Option<String>,
       pub juz: i32,
       pub page: i32,
   }

   pub struct Word {
       pub id: i32,
       pub verse_key: String,
       pub position: i32,
       pub text_uthmani: String,
   }
   ```

2. Remove `Node` struct with generic `node_id`

### Step 4: Update Python Graph Builder

**Location:** `research_and_dev/iqrah-knowledge-graph2/src/iqrah/graph/`

**Tasks:**
1. Update node creation to store **content keys as properties**:
   ```python
   # OLD (coupled to DB)
   node_id = f"WORD_INSTANCE:{verse_key}:{position}"
   graph.add_node(node_id, node_type='word_instance')

   # NEW (stores content key as property)
   node_id = f"WORD_INSTANCE:{verse_key}:{position}"  # Graph internal ID
   graph.add_node(
       node_id,
       node_type='word_instance',
       content_word_id=word.word_id,  # Reference to content.db
       verse_key=verse_key,
       position=position
   )
   ```

2. Update edge creation to include content keys
3. CBOR export includes content references

### Step 5: Update User DB References

**Location:** `rust/crates/iqrah-storage/migrations_user/`

**Tasks:**
1. Rename `node_id` to `content_key` in user_memory_states:
   ```sql
   -- New migration: 20241117000001_content_keys.sql
   ALTER TABLE user_memory_states RENAME COLUMN node_id TO content_key;
   ```

2. Update indexes:
   ```sql
   DROP INDEX idx_user_memory_node;
   CREATE INDEX idx_user_memory_content ON user_memory_states(content_key);
   ```

3. Update repository code to use `content_key` (verse_key, word_id, etc.)

### Step 6: Testing

**Tasks:**
1. Unit tests for each repository method
2. Integration test for full verse retrieval:
   ```rust
   #[tokio::test]
   async fn test_get_verse_with_words_and_translation() {
       let repo = create_test_content_repo().await;

       let verse = repo.get_verse("1:1").await?.unwrap();
       assert_eq!(verse.verse_number, 1);

       let words = repo.get_words_for_verse("1:1").await?;
       assert_eq!(words.len(), 4);

       let translation = repo.get_verse_translation("1:1", 1).await?;
       assert!(translation.is_some());
   }
   ```

3. Performance benchmarks:
   - Verse lookup: < 1ms
   - Full chapter with translations: < 10ms

## Migration Strategy

### Option 1: Rebuild (Recommended)

**Rationale:** Schema changes are so extensive that rebuilding is simpler and safer than migrating.

**Steps:**
1. Generate new content.db from Python with v2 schema
2. User DB unaffected (already tracks separately)
3. Update code to use new keys
4. No complex data migration SQL

**Downside:** Cannot preserve user data if content keys change (but see next doc for mitigation)

### Option 2: Gradual Migration

**If** you need to preserve existing user data tied to old node_ids:

1. Create mapping table:
   ```sql
   CREATE TABLE node_id_to_content_key (
       old_node_id TEXT PRIMARY KEY,
       content_type TEXT NOT NULL,  -- 'verse', 'word', 'chapter'
       content_key TEXT NOT NULL
   );
   ```

2. Populate from old data
3. Migrate user_memory_states using mapping
4. Drop mapping table

**Effort:** 2-3 days additional work

## Validation Checklist

Before merging:

- [ ] All tables have PRIMARY KEY
- [ ] All CHECK constraints use `OR x IS NULL` pattern, not `IN (..., NULL)`
- [ ] All foreign keys specified
- [ ] CASCADE semantics match table in "Cascade Semantics Summary"
- [ ] Indexes created for all common query patterns
- [ ] Partial unique indexes enforce XOR constraints
- [ ] STRICT mode enabled on all tables
- [ ] WITHOUT ROWID used for composite PK tables
- [ ] Sample data inserts successfully
- [ ] Migration runs without errors
- [ ] Repository tests pass

## Performance Expectations

| Operation | Target | Current v1 | v2 Improvement |
|-----------|--------|------------|----------------|
| Get verse by key | < 1ms | ~2ms (string PK) | ✅ Faster (indexed verse_key) |
| Get all words in verse | < 2ms | ~5ms (LIKE query) | ✅ Much faster (FK + position index) |
| Get translation | < 1ms | ~2ms | ✅ Same or faster (integer translator_id) |
| Full chapter load | < 10ms | ~20ms | ✅ 2x faster (better indexes) |

## References

- [02-translations-and-translators-normalization.md](02-translations-and-translators-normalization.md) - Translator system details
- [03-flexible-content-packages-plan.md](03-flexible-content-packages-plan.md) - Package management
- [04-versioning-and-migration-strategy.md](04-versioning-and-migration-strategy.md) - Version tracking

---

**Status:** Ready for implementation
**Estimated Effort:** 4-6 days (migration + repository updates + testing)
**Risk:** Medium (breaking change, requires coordinated update across layers)

# Content Database Architecture

**Related Question:** Q1 - Migration strategy for flexible vs inflexible content

## Overview

The Content Database (content.db) is a **read-only SQLite database** shipped with the application containing:
- Knowledge graph structure (nodes and edges)
- Quranic text (Arabic)
- Translations
- Metadata

## Current Rust Schema

**Location:** [rust/crates/iqrah-storage/migrations_content/20241116000001_content_schema.sql](../../rust/crates/iqrah-storage/migrations_content/20241116000001_content_schema.sql)

### Core Tables

#### 1. nodes
Stores all knowledge graph nodes with their types.

```sql
CREATE TABLE IF NOT EXISTS nodes (
    id TEXT PRIMARY KEY,
    node_type TEXT NOT NULL CHECK (node_type IN (
        'root', 'lemma', 'word', 'word_instance',
        'verse', 'chapter', 'knowledge'
    )),
    created_at INTEGER NOT NULL
) STRICT;
```

**Node ID Examples:**
- `ROOT:ktb` - Root morpheme
- `LEMMA:kataba` - Lemma
- `WORD:1:1:1` - Word in chapter 1, verse 1, position 1
- `WORD_INSTANCE:1:1:1` - Specific instance of that word
- `VERSE:1:1` - Verse 1 of chapter 1
- `CHAPTER:1` - Chapter 1 (Al-Fatihah)
- `WORD_INSTANCE:1:1:1:memorization` - Knowledge axis node (see Q8)

#### 2. edges
Stores relationships between nodes for dependency and knowledge propagation.

```sql
CREATE TABLE IF NOT EXISTS edges (
    source_id TEXT NOT NULL,
    target_id TEXT NOT NULL,
    edge_type INTEGER NOT NULL CHECK (edge_type IN (0, 1)),
    distribution_type INTEGER NOT NULL CHECK (distribution_type IN (0, 1, 2)),
    param1 REAL,
    param2 REAL,
    PRIMARY KEY (source_id, target_id),
    FOREIGN KEY (source_id) REFERENCES nodes(id),
    FOREIGN KEY (target_id) REFERENCES nodes(id)
) WITHOUT ROWID, STRICT;
```

**Edge Types:**
- `0` = Dependency edge (structural: verse depends on chapter)
- `1` = Knowledge edge (semantic: understanding X helps learn Y)

**Distribution Types:**
- `0` = Constant (param1 = constant value)
- `1` = Normal (param1 = mean, param2 = std deviation)
- `2` = Beta (param1 = alpha, param2 = beta)

#### 3. quran_text
Stores Arabic text for each node (primarily word instances and verses).

```sql
CREATE TABLE IF NOT EXISTS quran_text (
    node_id TEXT PRIMARY KEY,
    arabic TEXT NOT NULL,
    FOREIGN KEY (node_id) REFERENCES nodes(id)
) STRICT;
```

**Example Row:**
```
node_id: "WORD_INSTANCE:1:1:1"
arabic: "بِسْمِ"
```

#### 4. translations
Stores translations keyed by node ID and language code.

```sql
CREATE TABLE IF NOT EXISTS translations (
    node_id TEXT NOT NULL,
    language_code TEXT NOT NULL DEFAULT 'en',
    translation TEXT NOT NULL,
    PRIMARY KEY (node_id, language_code),
    FOREIGN KEY (node_id) REFERENCES nodes(id)
) WITHOUT ROWID, STRICT;
```

**Example Rows:**
```
node_id: "WORD_INSTANCE:1:1:1", language_code: "en", translation: "In the name"
node_id: "WORD_INSTANCE:1:1:1", language_code: "fr", translation: "Au nom de"
```

### Schema Features

- **STRICT mode:** Type safety enforced (SQLite 3.37+)
- **WITHOUT ROWID:** Performance optimization for tables with composite PKs
- **CHECK constraints:** Enum-like type safety for node_type, edge_type, etc.
- **Foreign keys:** Referential integrity enforced

## Python Schema Design (Future/Intended)

**Location:** [research_and_dev/iqrah-knowledge-graph2/src/iqrah/content/schema.py](../../research_and_dev/iqrah-knowledge-graph2/src/iqrah/content/schema.py)

This is a **much more comprehensive schema** designed for the future but NOT yet implemented in Rust.

### Additional Tables in Python Design

#### Metadata & Versioning
```python
schema_version (version INTEGER PRIMARY KEY)
content_packages (package_id, package_type, name, language_code, author, version, ...)
installed_packages (package_id, installed_at, enabled)
```

#### Inflexible Data (Core Quranic Structure)
Always shipped, never changed:

```python
chapters (chapter_number, revelation_order, revelation_location, ...)
verses (verse_key, chapter_number, verse_number, juz, hizb, page, sajdah, ...)
words (word_id, verse_key, position, uthmani_text, simple_text, ...)
morphology_segments (segment_id, word_id, position, ...)
lemmas (lemma_id, arabic, transliteration, ...)
roots (root_id, arabic, transliteration, ...)
stems (stem_id, ...)
```

#### Flexible Data (User-Downloadable)
Can be added/updated/removed:

```python
text_variants (package_id, verse_key, text)  -- Imlaei, Indopak scripts
verse_translations (package_id, verse_key, text)  -- Various translators
word_translations (package_id, word_id, text)
word_transliterations (package_id, word_id, text)

reciters (reciter_id, name, style)
verse_recitations (package_id, verse_key, audio_file, duration)
word_audio (package_id, word_id, audio_file, duration)
```

### Key Differences: Rust vs Python Schema

| Aspect | Rust (Current) | Python (Designed) | Status |
|--------|----------------|-------------------|--------|
| **Node storage** | Generic nodes table | Separate tables per type (chapters, verses, words) | Python richer |
| **Morphology** | Not stored | Full morphology_segments table | Not implemented |
| **Versioning** | No version table | schema_version + packages | Not implemented |
| **Flexible content** | No distinction | Package system with installed_packages | Not implemented |
| **Text variants** | Not supported | text_variants table | Not implemented |
| **Audio** | Not supported | reciters + verse_recitations + word_audio | Not implemented |

## Q1: Migration Strategy for Flexible vs Inflexible Content

### Current Rust Approach

**Answer:** There is **NO migration strategy** for flexible content in the current Rust implementation.

**Evidence:**
1. **No version table** - Relies on SQLx's `_sqlx_migrations` table only
2. **No package concept** - All content is monolithic
3. **Migration file uses `CREATE TABLE IF NOT EXISTS`** - No DROP/REPLACE logic
4. **No distinction** between flexible (translations) and inflexible (morphology) content

**Migration Execution:**
```rust
// rust/crates/iqrah-storage/src/content/mod.rs:10-19
pub async fn init_content_db(db_path: &str) -> Result<SqlitePool> {
    let pool = SqlitePool::connect(db_path).await?;

    sqlx::migrate!("./migrations_content")
        .run(&pool)
        .await?;

    Ok(pool)
}
```

This runs all migrations in order, but there's only ONE migration file (20241116000001).

### Python Schema's Intended Strategy

**Design Philosophy:**

1. **Inflexible Content** (Core Structure)
   - Chapters, verses, words, morphology
   - Ships with app, NEVER changes (or extremely rare schema migrations)
   - Versioned via `schema_version` table

2. **Flexible Content** (User Preferences)
   - Translations, audio, text variants
   - Packaged and downloadable
   - Managed via `content_packages` + `installed_packages`
   - Can be added/removed without affecting core data

**Package Installation Flow (Intended):**
```python
# Download translation package
package = {
    'package_id': 'sahih-international-v1',
    'package_type': 'verse_translation',
    'language_code': 'en',
    'author': 'Sahih International',
    'version': '1.0.0'
}

# Install
INSERT INTO content_packages VALUES (...)
INSERT INTO verse_translations (package_id, verse_key, text) VALUES (...)
INSERT INTO installed_packages (package_id, installed_at, enabled) VALUES (...)
```

**Removal:**
```sql
-- Disable package
UPDATE installed_packages SET enabled = 0 WHERE package_id = ?

-- Or fully remove
DELETE FROM verse_translations WHERE package_id = ?
DELETE FROM installed_packages WHERE package_id = ?
DELETE FROM content_packages WHERE package_id = ?
```

### What About the Version Table?

**Python Design:**
```python
CREATE TABLE schema_version (
    version INTEGER PRIMARY KEY
)
INSERT INTO schema_version (version) VALUES (1)
```

**Purpose:**
- Track **content database schema version** (not app version)
- Enable migrations when core structure changes (e.g., adding morphology_segments table)
- Different from package versions (which track individual translation/audio versions)

**Current Rust Status:** NOT IMPLEMENTED. Only SQLx's internal `_sqlx_migrations` table exists.

## Current Content Update Strategy

**How content.db is currently updated:**

1. **Full Replacement:** Ship new content.db file with app updates
2. **No in-place updates:** Users don't download individual translations
3. **No partial updates:** Can't add one new translation without replacing entire DB

**Implications:**
- New translation? Ship entire new content.db (wasteful)
- Fix typo in one verse? Ship entire new content.db
- No user choice of which translations to download

## Recommendations

### Option 1: Implement Python Schema (Full Featured)
**Pros:**
- Supports flexible content downloads
- Clear separation of concerns
- Users can customize content

**Cons:**
- Significant implementation effort
- More complex query logic
- Requires download infrastructure

### Option 2: Keep Simple Schema (Current Approach)
**Pros:**
- Simple and working
- Easier to maintain
- Sufficient for MVP

**Cons:**
- Full DB replacement for any update
- No user choice of translations
- Larger app size (ships all content)

### Option 3: Hybrid Approach
- Keep simple nodes/edges/text schema
- Add `schema_version` table for migration tracking
- Add `content_packages` + `installed_packages` for flexible content only
- Don't split nodes into separate tables (keep generic approach)

**Minimal changes to enable flexible translations:**
```sql
-- Add version tracking
CREATE TABLE schema_version (version INTEGER PRIMARY KEY);
INSERT INTO schema_version (version) VALUES (1);

-- Add package metadata
CREATE TABLE content_packages (
    package_id TEXT PRIMARY KEY,
    package_type TEXT CHECK (package_type IN ('translation', 'audio', 'text_variant')),
    name TEXT,
    language_code TEXT,
    version TEXT
);

-- Link translations to packages
ALTER TABLE translations ADD COLUMN package_id TEXT;
```

## Data Import Process (Current)

Content DB is populated via **CBOR import** from Python-generated graph:

1. Python generates knowledge graph → exports as CBOR
2. Rust imports CBOR via [cbor_import.rs](../../rust/crates/iqrah-core/src/cbor_import.rs)
3. Batch inserts into nodes, edges, quran_text, translations

**See:** [03-knowledge-graph.md](03-knowledge-graph.md) for CBOR import details.

## File Locations

**Rust Implementation:**
- Schema: [migrations_content/20241116000001_content_schema.sql](../../rust/crates/iqrah-storage/migrations_content/20241116000001_content_schema.sql)
- Repository: [src/content/repository.rs](../../rust/crates/iqrah-storage/src/content/repository.rs)
- Models: [src/content/models.rs](../../rust/crates/iqrah-storage/src/content/models.rs)

**Python Design:**
- Schema: [src/iqrah/content/schema.py](../../research_and_dev/iqrah-knowledge-graph2/src/iqrah/content/schema.py)

---

**Navigation:** [← Executive Summary](00-executive-summary.md) | [Next: User Database →](02-user-database.md)

# Flexible Content Import Strategy

**Related Question:** Q6 - How is the importation of flexible data (translations, Arabic scripts, audio, etc.) intended in the current design?

## Overview

The codebase shows a **significant gap** between the designed flexible content system (Python) and the implemented system (Rust).

**Python Design:** Sophisticated package management for downloadable content.

**Rust Implementation:** Monolithic content.db with no package concept.

## Q6: Flexible Content Import Design

### What is "Flexible Content"?

**Flexible content** refers to data that:
- Can be added/removed after app installation
- Is user-selectable (user chooses which translations to download)
- Has multiple variants (many translators, multiple Arabic scripts)
- Can be updated independently of core data

**Examples:**
- Translations (Sahih International, Yusuf Ali, Pickthall, etc.)
- Arabic scripts (Uthmani, Imlaei, Indopak)
- Audio recitations (different reciters, different styles)
- Word-by-word audio
- Transliterations

### What is "Inflexible Content"?

**Inflexible content** refers to core structural data that:
- Ships with app
- Rarely (if ever) changes
- Is the same for all users
- Represents canonical Quranic structure

**Examples:**
- Chapter metadata (Al-Fatihah is chapter 1, revealed in Mecca, etc.)
- Verse structure (verse 1:1 has 4 words)
- Word positions and morphology
- Root/lemma relationships
- Knowledge graph structure

## Python Schema Design (Intended)

**Location:** [content/schema.py](../../research_and_dev/iqrah-knowledge-graph2/src/iqrah/content/schema.py)

### Package Management Tables

**Schema Version Tracking:**
```python
CREATE TABLE schema_version (
    version INTEGER PRIMARY KEY
)
```

**Content Packages Catalog:**
```python
CREATE TABLE content_packages (
    package_id TEXT PRIMARY KEY,
    package_type TEXT CHECK (package_type IN (
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
    file_size INTEGER,
    download_url TEXT,
    checksum TEXT,
    created_at INTEGER NOT NULL,
    updated_at INTEGER
)
```

**Installed Packages Tracking:**
```python
CREATE TABLE installed_packages (
    package_id TEXT PRIMARY KEY,
    installed_at INTEGER NOT NULL,
    enabled INTEGER DEFAULT 1,
    FOREIGN KEY (package_id) REFERENCES content_packages(package_id)
)
```

### Flexible Content Tables

**Text Variants (Alternative Arabic Scripts):**
```python
CREATE TABLE text_variants (
    package_id TEXT NOT NULL,
    variant_type TEXT CHECK (variant_type IN ('imlaei', 'indopak', 'simple')),
    verse_key TEXT,
    word_id TEXT,
    text TEXT NOT NULL,
    PRIMARY KEY (package_id, verse_key, word_id),
    FOREIGN KEY (package_id) REFERENCES content_packages(package_id)
)
```

**Verse Translations:**
```python
CREATE TABLE verse_translations (
    package_id TEXT NOT NULL,
    verse_key TEXT NOT NULL,
    text TEXT NOT NULL,
    footnotes TEXT,
    PRIMARY KEY (package_id, verse_key),
    FOREIGN KEY (package_id) REFERENCES content_packages(package_id),
    FOREIGN KEY (verse_key) REFERENCES verses(verse_key)
)
```

**Word Translations:**
```python
CREATE TABLE word_translations (
    package_id TEXT NOT NULL,
    word_id TEXT NOT NULL,
    text TEXT NOT NULL,
    PRIMARY KEY (package_id, word_id),
    FOREIGN KEY (package_id) REFERENCES content_packages(package_id),
    FOREIGN KEY (word_id) REFERENCES words(word_id)
)
```

**Word Transliterations:**
```python
CREATE TABLE word_transliterations (
    package_id TEXT NOT NULL,
    word_id TEXT NOT NULL,
    text TEXT NOT NULL,
    PRIMARY KEY (package_id, word_id),
    FOREIGN KEY (package_id) REFERENCES content_packages(package_id),
    FOREIGN KEY (word_id) REFERENCES words(word_id)
)
```

**Audio Reciters:**
```python
CREATE TABLE reciters (
    reciter_id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    language_code TEXT,
    style TEXT
)
```

**Verse Recitations:**
```python
CREATE TABLE verse_recitations (
    package_id TEXT NOT NULL,
    verse_key TEXT NOT NULL,
    audio_file TEXT NOT NULL,
    duration INTEGER,
    PRIMARY KEY (package_id, verse_key),
    FOREIGN KEY (package_id) REFERENCES content_packages(package_id),
    FOREIGN KEY (verse_key) REFERENCES verses(verse_key)
)
```

**Word-by-Word Audio:**
```python
CREATE TABLE word_audio (
    package_id TEXT NOT NULL,
    word_id TEXT NOT NULL,
    audio_file TEXT NOT NULL,
    duration INTEGER,
    PRIMARY KEY (package_id, word_id),
    FOREIGN KEY (package_id) REFERENCES content_packages(package_id),
    FOREIGN KEY (word_id) REFERENCES words(word_id)
)
```

### Intended Workflow

**Step 1: User Browses Available Packages**
```sql
SELECT package_id, name, author, description, file_size
FROM content_packages
WHERE package_type = 'verse_translation'
  AND language_code = 'en'
  AND package_id NOT IN (SELECT package_id FROM installed_packages)
ORDER BY name;
```

**Example Results:**
```
package_id              | name                  | author               | file_size
------------------------|----------------------|----------------------|----------
sahih-intl-v1          | Sahih International  | Sahih International  | 512 KB
yusuf-ali-v2           | Yusuf Ali            | Abdullah Yusuf Ali   | 687 KB
pickthall-v1           | Pickthall            | Marmaduke Pickthall  | 534 KB
```

**Step 2: User Downloads Package**
```python
# App downloads from download_url
response = requests.get(package.download_url)
translation_data = response.json()  # Or SQLite file, or CSV, etc.
```

**Step 3: App Installs Package**
```sql
BEGIN TRANSACTION;

-- Insert verse translations
INSERT INTO verse_translations (package_id, verse_key, text)
VALUES
  ('sahih-intl-v1', '1:1', 'In the name of Allah, the Entirely Merciful, the Especially Merciful.'),
  ('sahih-intl-v1', '1:2', 'All praise is due to Allah, Lord of the worlds.'),
  ...;

-- Mark as installed
INSERT INTO installed_packages (package_id, installed_at, enabled)
VALUES ('sahih-intl-v1', 1700000000, 1);

COMMIT;
```

**Step 4: User Queries Translation**
```sql
SELECT vt.text
FROM verse_translations vt
JOIN installed_packages ip ON vt.package_id = ip.package_id
WHERE vt.verse_key = '1:1'
  AND ip.enabled = 1
  AND vt.package_id = 'sahih-intl-v1';
```

**Step 5: User Disables/Enables Packages**
```sql
-- Disable (doesn't delete data, just hides)
UPDATE installed_packages SET enabled = 0 WHERE package_id = 'yusuf-ali-v2';

-- Re-enable
UPDATE installed_packages SET enabled = 1 WHERE package_id = 'yusuf-ali-v2';
```

**Step 6: User Uninstalls Package**
```sql
BEGIN TRANSACTION;

-- Remove translations
DELETE FROM verse_translations WHERE package_id = 'sahih-intl-v1';

-- Remove installation record
DELETE FROM installed_packages WHERE package_id = 'sahih-intl-v1';

-- Optionally remove from catalog (or keep for re-download)
-- DELETE FROM content_packages WHERE package_id = 'sahih-intl-v1';

COMMIT;
```

## Rust Implementation (Current Status)

### Current Schema

**Location:** [migrations_content/20241116000001_content_schema.sql](../../rust/crates/iqrah-storage/migrations_content/20241116000001_content_schema.sql)

**What Exists:**
```sql
CREATE TABLE translations (
    node_id TEXT NOT NULL,
    language_code TEXT NOT NULL DEFAULT 'en',
    translation TEXT NOT NULL,
    PRIMARY KEY (node_id, language_code)
)
```

**What's Missing:**
- ❌ No `content_packages` table
- ❌ No `installed_packages` table
- ❌ No `verse_translations` table (uses generic `translations`)
- ❌ No `text_variants` table
- ❌ No audio tables (`reciters`, `verse_recitations`, `word_audio`)
- ❌ No `word_transliterations` table

**Current Limitations:**

| Feature | Python Design | Rust Implementation | Gap |
|---------|--------------|---------------------|-----|
| Multiple translations | ✅ Package system | ❌ Single translation per language | MAJOR |
| Translation attribution | ✅ Author tracked | ❌ No author info | HIGH |
| User-selectable translations | ✅ Install/uninstall | ❌ Monolithic | MAJOR |
| Arabic script variants | ✅ text_variants table | ❌ Not supported | HIGH |
| Audio recitations | ✅ Full schema | ❌ Not supported | HIGH |
| Translation updates | ✅ Version tracking | ❌ Replace entire DB | HIGH |

### Current Import Process

**How translations currently get into Rust:**

1. **Python generates graph** (includes one translation)
2. **Exports to CBOR** (includes translation data)
3. **Rust imports CBOR** via [cbor_import.rs](../../rust/crates/iqrah-core/src/cbor_import.rs)
4. **Inserts into `translations` table**

**Code:**
```rust
// Conceptual - actual CBOR structure may vary
for record in cbor_records {
    if let Some(translation) = record.translation {
        query("INSERT INTO translations (node_id, language_code, translation) VALUES (?, ?, ?)")
            .bind(&record.node_id)
            .bind("en")  // Hardcoded!
            .bind(&translation)
            .execute(&pool)
            .await?;
    }
}
```

**Problems:**
- Only ONE translation ships with app
- No way to add more without rebuilding content.db
- No way for user to choose translator

## Gap Analysis

### Designed Features (Python) Not Implemented (Rust)

**Package Management:**
- ❌ Package catalog
- ❌ Installation tracking
- ❌ Version management
- ❌ Enable/disable toggles

**Multiple Translations:**
- ❌ Cannot have both Sahih International AND Yusuf Ali
- ❌ No translator attribution
- ❌ No translation selection UI

**Flexible Content Types:**
- ❌ No alternative Arabic scripts (Imlaei, Indopak)
- ❌ No audio recitations
- ❌ No word-by-word audio
- ❌ No transliterations

**Content Updates:**
- ❌ Cannot update single translation
- ❌ Must replace entire content.db for any change
- ❌ No incremental updates

## Recommended Implementation Path

### Option 1: Full Package System (High Effort)

**Implement the Python schema in Rust:**

1. **Add tables to content DB:**
   - `content_packages`
   - `installed_packages`
   - `verse_translations`
   - `word_translations`
   - `text_variants`
   - `reciters`, `verse_recitations`, `word_audio`

2. **Implement package installation service:**
```rust
pub struct PackageService {
    content_repo: Arc<dyn ContentRepository>,
}

impl PackageService {
    pub async fn install_translation_package(
        &self,
        package_id: &str,
        download_url: &str,
    ) -> Result<()> {
        // 1. Download package
        let package_data = self.download_package(download_url).await?;

        // 2. Validate checksum
        self.validate_checksum(&package_data)?;

        // 3. Begin transaction
        let mut tx = self.content_repo.begin_transaction().await?;

        // 4. Insert translations
        for (verse_key, translation) in package_data.translations {
            query("INSERT INTO verse_translations (package_id, verse_key, text) VALUES (?, ?, ?)")
                .bind(package_id)
                .bind(&verse_key)
                .bind(&translation)
                .execute(&mut tx)
                .await?;
        }

        // 5. Mark as installed
        query("INSERT INTO installed_packages (package_id, installed_at, enabled) VALUES (?, ?, 1)")
            .bind(package_id)
            .bind(Utc::now().timestamp())
            .execute(&mut tx)
            .await?;

        // 6. Commit
        tx.commit().await?;

        Ok(())
    }
}
```

3. **Update query logic:**
```rust
// Old
async fn get_translation(&self, node_id: &str, language_code: &str) -> Result<Option<String>> {
    query_scalar("SELECT translation FROM translations WHERE node_id = ? AND language_code = ?")
        .bind(node_id)
        .bind(language_code)
        .fetch_optional(&self.pool)
        .await
}

// New
async fn get_translation(&self, node_id: &str, package_id: &str) -> Result<Option<String>> {
    query_scalar(
        "SELECT vt.text
         FROM verse_translations vt
         JOIN installed_packages ip ON vt.package_id = ip.package_id
         WHERE vt.verse_key = ? AND vt.package_id = ? AND ip.enabled = 1"
    )
    .bind(node_id)
    .bind(package_id)
    .fetch_optional(&self.pool)
    .await
}
```

**Pros:**
- Full feature parity with Python design
- User can download translations on demand
- Smaller initial app size
- Easy to add new translators

**Cons:**
- Significant implementation effort
- Requires download infrastructure
- More complex query logic
- Need UI for package management

**Estimated Effort:** 2-3 weeks

### Option 2: Multi-Translation Support (Medium Effort)

**Add multiple translations without full package system:**

1. **Update translations table:**
```sql
ALTER TABLE translations ADD COLUMN translator TEXT;
ALTER TABLE translations DROP PRIMARY KEY;
ALTER TABLE translations ADD PRIMARY KEY (node_id, language_code, translator);
```

2. **Ship with 3-5 popular translations:**
```sql
INSERT INTO translations VALUES
  ('VERSE:1:1', 'en', 'Sahih International', 'In the name of Allah...'),
  ('VERSE:1:1', 'en', 'Yusuf Ali', 'In the name of God...'),
  ('VERSE:1:1', 'en', 'Pickthall', 'In the name of Allah...');
```

3. **Add user preference:**
```rust
// In user.db
INSERT INTO app_settings (key, value) VALUES ('preferred_translator', 'Sahih International');
```

4. **Query with preference:**
```rust
async fn get_preferred_translation(&self, node_id: &str) -> Result<Option<String>> {
    let preferred = self.get_setting("preferred_translator").await?;

    query_scalar(
        "SELECT translation FROM translations
         WHERE node_id = ? AND language_code = 'en' AND translator = ?"
    )
    .bind(node_id)
    .bind(preferred)
    .fetch_optional(&self.pool)
    .await
}
```

**Pros:**
- Supports multiple translations
- No download infrastructure needed
- Simpler than full package system
- User can switch translators

**Cons:**
- All translations ship with app (larger size)
- Can't add new translations without app update
- No versioning or updates

**Estimated Effort:** 3-5 days

### Option 3: Keep Simple, Document Gap (Low Effort)

**Accept current limitations as MVP scope:**

1. Ship with ONE high-quality translation (e.g., Sahih International)
2. Document that multiple translations are future work
3. Keep Python schema as reference for future implementation

**Pros:**
- No implementation effort
- Simpler codebase
- Faster to market

**Cons:**
- Limited user choice
- Larger gap from Python design
- Harder to add later (migration complexity)

**Estimated Effort:** 0 (documentation only)

## Recommended Path Forward

**Phase 1 (MVP):** Option 3 - Single translation, document gap.

**Phase 2 (Post-MVP):** Option 2 - Multi-translation support.

**Phase 3 (Long-term):** Option 1 - Full package system for audio, scripts, etc.

**Rationale:**
- Get MVP out quickly with single translation
- Add multi-translation when user demand emerges
- Build package infrastructure when adding audio (more compelling use case)

## File Locations

**Python Design:**
- Schema: [content/schema.py](../../research_and_dev/iqrah-knowledge-graph2/src/iqrah/content/schema.py) (lines 122-525)

**Rust Implementation:**
- Current schema: [migrations_content/20241116000001_content_schema.sql](../../rust/crates/iqrah-storage/migrations_content/20241116000001_content_schema.sql)
- Repository: [content/repository.rs](../../rust/crates/iqrah-storage/src/content/repository.rs)

**Gap:**
- Package system: NOT IMPLEMENTED
- Multiple translations: NOT IMPLEMENTED
- Audio support: NOT IMPLEMENTED
- Text variants: NOT IMPLEMENTED

---

**Navigation:** [← Navigation & Algorithms](07-navigation-and-algorithms.md) | [Next: Gaps & Recommendations →](09-gaps-and-recommendations.md)

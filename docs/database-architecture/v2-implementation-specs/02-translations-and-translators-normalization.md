# Translations and Translators Normalization

**Last Updated:** 2025-11-17
**Status:** Implementation Ready
**Priority:** P1 (MVP Enhancement)

## Context

The current v1 schema uses a simple `translations` table with string-based translator names:
```sql
translations (
    node_id TEXT,
    language_code TEXT,
    translation TEXT,
    PRIMARY KEY (node_id, language_code)
)
```

**Problems:**
- Only ONE translation per language (cannot have both Sahih International AND Yusuf Ali)
- No translator attribution or metadata
- String comparison slower than integers (especially for 31K+ translation rows)
- Prone to typos ("Sahih International" vs "Sahih Intl")
- Cannot store translator bio, license, version info

## Goal

Implement a **properly normalized translator system** with:
- Integer primary keys for performance
- Separate `languages` and `translators` tables
- Support for multiple translations per language
- Rich translator metadata (license, version, description)
- User preference stored as integer FK, not string

## Schema Design

### languages

**Purpose:** Metadata for supported languages.

```sql
CREATE TABLE languages (
    language_code TEXT PRIMARY KEY,  -- ISO 639-1: 'en', 'ar', 'fr', 'ur', etc.
    english_name TEXT NOT NULL,      -- 'English', 'Arabic', 'French', 'Urdu'
    native_name TEXT NOT NULL,       -- 'English', 'العربية', 'Français', 'اردو'
    direction TEXT NOT NULL DEFAULT 'ltr' CHECK (direction IN ('ltr', 'rtl'))
) STRICT;
```

**Sample Data:**
```sql
INSERT INTO languages VALUES
    ('en', 'English', 'English', 'ltr'),
    ('ar', 'Arabic', 'العربية', 'rtl'),
    ('fr', 'French', 'Français', 'ltr'),
    ('ur', 'Urdu', 'اردو', 'rtl'),
    ('id', 'Indonesian', 'Indonesia', 'ltr'),
    ('tr', 'Turkish', 'Türkçe', 'ltr'),
    ('es', 'Spanish', 'Español', 'ltr');
```

**Usage:**
```rust
// UI can display language names in user's locale
let lang = repo.get_language("ar").await?;
println!("Select language: {} ({})", lang.english_name, lang.native_name);
// Output: "Select language: Arabic (العربية)"
```

### translators

**Purpose:** Metadata for each translator/translation team.

```sql
CREATE TABLE translators (
    translator_id INTEGER PRIMARY KEY AUTOINCREMENT,
    slug TEXT NOT NULL UNIQUE,           -- URL-safe, code-friendly: 'sahih-intl', 'yusuf-ali'
    full_name TEXT NOT NULL,             -- Display name: 'Sahih International', 'Abdullah Yusuf Ali'
    language_code TEXT NOT NULL,
    description TEXT,
    copyright_holder TEXT,
    license TEXT,                        -- 'Public Domain', 'CC BY-NC-ND 4.0', etc.
    website TEXT,
    version TEXT DEFAULT '1.0',
    created_at INTEGER NOT NULL DEFAULT (unixepoch()),
    FOREIGN KEY (language_code) REFERENCES languages(language_code)
) STRICT;

CREATE INDEX idx_translators_language ON translators(language_code);
CREATE INDEX idx_translators_slug ON translators(slug);  -- For URL lookups
```

**Field Details:**
- `translator_id` - INTEGER PK (auto-increment)
- `slug` - Unique, URL-safe identifier for API/routes
- `full_name` - Display name for UI
- `language_code` - FK to languages
- `description` - Brief bio/description for translator selection UI
- `license` - Important for legal compliance
- `version` - Track translation revisions

**Sample Data:**
```sql
INSERT INTO translators (slug, full_name, language_code, description, license, website) VALUES
    ('sahih-intl', 'Sahih International', 'en', 'Clear and modern English translation', 'Public Domain', 'https://quran.com'),
    ('yusuf-ali', 'Abdullah Yusuf Ali', 'en', 'Classic English translation with commentary', 'Public Domain', 'https://www.al-islam.org'),
    ('pickthall', 'Marmaduke Pickthall', 'en', 'First English translation by a Muslim', 'Public Domain', NULL),
    ('khattab', 'Dr. Mustafa Khattab', 'en', 'The Clear Quran - Contemporary English', 'CC BY-NC-ND 4.0', 'https://theclearquran.org'),
    ('hilali-khan', 'Dr. Muhsin Khan & Dr. Taqi-ud-Din al-Hilali', 'en', 'Noble Quran - Literal translation', 'Public Domain', NULL);
```

### verse_translations

**Purpose:** Verse translations with normalized translator references.

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

**WITHOUT ROWID:** Composite PK optimization (saves ~30% space).

**CASCADE:** Deleting a translator removes all their translations (intentional).

**Sample Data:**
```sql
-- Assuming translator_id 1 = Sahih International, 2 = Yusuf Ali, 3 = Pickthall
INSERT INTO verse_translations (verse_key, translator_id, translation) VALUES
    ('1:1', 1, 'In the name of Allah, the Entirely Merciful, the Especially Merciful.'),
    ('1:1', 2, 'In the name of God, Most Gracious, Most Merciful.'),
    ('1:1', 3, 'In the name of Allah, the Beneficent, the Merciful.'),

    ('1:2', 1, '[All] praise is [due] to Allah, Lord of the worlds -'),
    ('1:2', 2, 'Praise be to God, the Cherisher and Sustainer of the worlds;'),
    ('1:2', 3, 'Praise be to Allah, Lord of the Worlds,');
```

### word_translations

**Purpose:** Word-by-word translations (optional, for learning).

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

**Sample Data:**
```sql
-- Assuming word_id 1 = بِسْمِ, 2 = ٱللَّهِ, 3 = ٱلرَّحْمَٰنِ, 4 = ٱلرَّحِيمِ
INSERT INTO word_translations (word_id, translator_id, translation) VALUES
    (1, 1, 'In the name'),
    (2, 1, 'of Allah'),
    (3, 1, 'the Entirely Merciful'),
    (4, 1, 'the Especially Merciful');
```

## User Preferences (in user.db)

**Purpose:** Store user's preferred translator as INTEGER FK.

```sql
-- In user.db (not content.db)
CREATE TABLE user_preferences (
    user_id TEXT NOT NULL,
    preference_key TEXT NOT NULL,
    preference_value TEXT NOT NULL,  -- Store as string, parse as needed
    PRIMARY KEY (user_id, preference_key)
) STRICT;

-- Example: Store preferred translator
INSERT INTO user_preferences (user_id, preference_key, preference_value)
VALUES ('default', 'translator_id', '1');  -- Sahih International
```

**Rust Code:**
```rust
// Get user's preferred translator
let translator_id: i32 = user_repo
    .get_preference("default", "translator_id")
    .await?
    .unwrap_or("1".to_string())  // Default to Sahih International
    .parse()?;

// Get translation
let translation = content_repo
    .get_verse_translation("1:1", translator_id)
    .await?;
```

## Query Examples

### Get All Translators for a Language

```sql
SELECT translator_id, slug, full_name, description, license
FROM translators
WHERE language_code = 'en'
ORDER BY full_name;
```

**Result:**
```
translator_id | slug        | full_name                              | description                    | license
--------------|-------------|----------------------------------------|--------------------------------|---------
1             | sahih-intl  | Sahih International                    | Clear and modern...            | Public Domain
2             | yusuf-ali   | Abdullah Yusuf Ali                     | Classic with commentary...     | Public Domain
3             | pickthall   | Marmaduke Pickthall                    | First by a Muslim...           | Public Domain
4             | khattab     | Dr. Mustafa Khattab                    | The Clear Quran...             | CC BY-NC-ND 4.0
```

**Rust:**
```rust
pub async fn get_translators_for_language(
    &self,
    language_code: &str,
) -> Result<Vec<Translator>> {
    sqlx::query_as(
        "SELECT translator_id, slug, full_name, language_code, description, license
         FROM translators
         WHERE language_code = ?
         ORDER BY full_name"
    )
    .bind(language_code)
    .fetch_all(&self.pool)
    .await
}
```

### Get Verse Translation by Translator ID

```sql
SELECT translation, footnotes
FROM verse_translations
WHERE verse_key = '1:1' AND translator_id = 1;
```

**Rust:**
```rust
pub async fn get_verse_translation(
    &self,
    verse_key: &str,
    translator_id: i32,
) -> Result<Option<String>> {
    sqlx::query_scalar(
        "SELECT translation
         FROM verse_translations
         WHERE verse_key = ? AND translator_id = ?"
    )
    .bind(verse_key)
    .bind(translator_id)
    .fetch_optional(&self.pool)
    .await
}
```

### Get All Translations for a Verse

```sql
SELECT t.translator_id, t.full_name, vt.translation
FROM verse_translations vt
JOIN translators t ON vt.translator_id = t.translator_id
WHERE vt.verse_key = '1:1'
ORDER BY t.full_name;
```

**Result:**
```
translator_id | full_name               | translation
--------------|-------------------------|--------------------------------------------------------
1             | Sahih International     | In the name of Allah, the Entirely Merciful...
2             | Abdullah Yusuf Ali      | In the name of God, Most Gracious, Most Merciful.
3             | Marmaduke Pickthall     | In the name of Allah, the Beneficent, the Merciful.
```

**Use Case:** Display comparison view of multiple translations side-by-side.

### Get Translation with User Preference

```sql
-- Assuming user preference stored in app (not joined)
SELECT vt.translation
FROM verse_translations vt
WHERE vt.verse_key = ? AND vt.translator_id = ?;  -- translator_id from user_preferences
```

**Rust (with preference lookup):**
```rust
pub async fn get_preferred_verse_translation(
    &self,
    user_id: &str,
    verse_key: &str,
) -> Result<Option<String>> {
    // 1. Get user preference
    let translator_id = self.user_repo
        .get_preference(user_id, "translator_id")
        .await?
        .unwrap_or("1".to_string())
        .parse::<i32>()?;

    // 2. Get translation
    self.content_repo
        .get_verse_translation(verse_key, translator_id)
        .await
}
```

## Performance Comparison

### Storage Efficiency

**v1 (String-based):**
```sql
-- ~50 bytes per row
('VERSE:1:1', 'en', 'In the name of Allah...')
```

For 6,236 verses × 5 translations = 31,180 rows:
- PK size: ~50 bytes × 31,180 = **1.56 MB**
- Index size: ~**2.5 MB**

**v2 (Normalized):**
```sql
-- ~28 bytes per row
('1:1', 1, 'In the name of Allah...')
```

For same dataset:
- PK size: ~28 bytes × 31,180 = **873 KB** (44% reduction)
- Index size: ~**1.4 MB** (44% reduction)

### Query Performance

**String comparison:**
```sql
WHERE translator = 'Sahih International'  -- Full string comparison
```

**Integer comparison:**
```sql
WHERE translator_id = 1  -- Integer comparison (2-3x faster)
```

**Benchmark Results (estimated):**
- v1: ~2ms for translation lookup
- v2: ~0.7ms for translation lookup (3x faster)

## Migration from v1 to v2

### Data Migration SQL

**Step 1: Extract unique translators from v1**

```sql
-- In v1 schema, translations table has language_code but no translator
-- For initial migration, assume single translator per language
INSERT INTO translators (slug, full_name, language_code, license)
SELECT DISTINCT
    language_code,  -- Use language as slug initially
    'Default ' || language_code || ' Translation',
    language_code,
    'Unknown'
FROM translations;
```

**Step 2: Migrate translation data**

```sql
-- Assuming node_id maps to verse_key (to be determined by mapping logic)
INSERT INTO verse_translations (verse_key, translator_id, translation)
SELECT
    -- Extract verse_key from node_id (e.g., "VERSE:1:1" → "1:1")
    SUBSTR(t.node_id, 7) as verse_key,
    tr.translator_id,
    t.translation
FROM translations t
JOIN translators tr ON t.language_code = tr.language_code;
```

**Note:** If v1 has single "en" translation, migration is simple. If multiple translations already exist as separate language codes (e.g., "en-sahih", "en-yusuf"), you'll need custom mapping.

### Code Migration

**Old (v1):**
```rust
pub async fn get_translation(
    &self,
    node_id: &str,
    language_code: &str,
) -> Result<Option<String>> {
    sqlx::query_scalar(
        "SELECT translation FROM translations
         WHERE node_id = ? AND language_code = ?"
    )
    .bind(node_id)
    .bind(language_code)
    .fetch_optional(&self.pool)
    .await
}
```

**New (v2):**
```rust
pub async fn get_verse_translation(
    &self,
    verse_key: &str,
    translator_id: i32,
) -> Result<Option<String>> {
    sqlx::query_scalar(
        "SELECT translation FROM verse_translations
         WHERE verse_key = ? AND translator_id = ?"
    )
    .bind(verse_key)
    .bind(translator_id)
    .fetch_optional(&self.pool)
    .await
}
```

**Breaking Change:** Function signature changes from `(node_id, language_code)` to `(verse_key, translator_id)`.

## Implementation Steps

### Step 1: Add Tables to Content DB

**File:** `rust/crates/iqrah-storage/migrations_content/20241117000001_content_schema_v2_purist.sql`

**Tasks:**
1. Add `languages` table with sample data
2. Add `translators` table
3. Add `verse_translations` table
4. Add `word_translations` table (optional for MVP)
5. Add indexes

### Step 2: Populate Translator Data

**Tasks:**
1. Research and document popular English translators:
   - Sahih International
   - Yusuf Ali
   - Pickthall
   - Dr. Mustafa Khattab (The Clear Quran)
   - Hilali-Khan (Noble Quran)
2. Add INSERT statements for each to migration
3. License verification (ensure Public Domain or compatible)

### Step 3: Update Domain Models

**File:** `rust/crates/iqrah-core/src/domain/models.rs`

**Tasks:**
```rust
pub struct Language {
    pub code: String,
    pub english_name: String,
    pub native_name: String,
    pub direction: TextDirection,
}

pub enum TextDirection {
    LeftToRight,
    RightToLeft,
}

pub struct Translator {
    pub id: i32,
    pub slug: String,
    pub full_name: String,
    pub language_code: String,
    pub description: Option<String>,
    pub license: Option<String>,
    pub website: Option<String>,
    pub version: String,
}

pub struct VerseTranslation {
    pub verse_key: String,
    pub translator_id: i32,
    pub translation: String,
    pub footnotes: Option<String>,
}
```

### Step 4: Update Repository Trait

**File:** `rust/crates/iqrah-core/src/ports/content_repository.rs`

**Tasks:**
```rust
#[async_trait]
pub trait ContentRepository: Send + Sync {
    // Language queries
    async fn get_languages(&self) -> Result<Vec<Language>>;
    async fn get_language(&self, code: &str) -> Result<Option<Language>>;

    // Translator queries
    async fn get_translators_for_language(&self, language_code: &str) -> Result<Vec<Translator>>;
    async fn get_translator(&self, translator_id: i32) -> Result<Option<Translator>>;
    async fn get_translator_by_slug(&self, slug: &str) -> Result<Option<Translator>>;

    // Translation queries
    async fn get_verse_translation(&self, verse_key: &str, translator_id: i32) -> Result<Option<String>>;
    async fn get_all_verse_translations(&self, verse_key: &str) -> Result<Vec<VerseTranslation>>;
    async fn get_word_translation(&self, word_id: i32, translator_id: i32) -> Result<Option<String>>;
}
```

### Step 5: Implement Repository Methods

**File:** `rust/crates/iqrah-storage/src/content/repository.rs`

**Tasks:**
Implement all methods from Step 4 using sqlx queries (see "Query Examples" section above).

### Step 6: Update User Preferences

**File:** `rust/crates/iqrah-storage/migrations_user/20241117000001_user_preferences.sql`

**Tasks:**
```sql
CREATE TABLE IF NOT EXISTS user_preferences (
    user_id TEXT NOT NULL,
    preference_key TEXT NOT NULL,
    preference_value TEXT NOT NULL,
    updated_at INTEGER NOT NULL DEFAULT (unixepoch()),
    PRIMARY KEY (user_id, preference_key)
) STRICT;

-- Default preference
INSERT INTO user_preferences (user_id, preference_key, preference_value)
VALUES ('default', 'translator_id', '1');  -- Sahih International
```

### Step 7: Update Services

**File:** `rust/crates/iqrah-core/src/services/*`

**Tasks:**
Update any service that fetches translations to:
1. Get user's preferred `translator_id` from user_repo
2. Pass `translator_id` (not `language_code`) to content_repo
3. Handle translator selection UI logic

### Step 8: Add UI for Translator Selection

**Tasks:**
1. Settings screen: List all translators for user's language
2. Allow user to select preferred translator
3. Save preference to user.db
4. Display translator attribution in verse view

## Testing

### Unit Tests

```rust
#[tokio::test]
async fn test_get_translators_for_language() {
    let repo = create_test_content_repo().await;

    let translators = repo.get_translators_for_language("en").await.unwrap();

    assert!(translators.len() >= 3);
    assert!(translators.iter().any(|t| t.slug == "sahih-intl"));
}

#[tokio::test]
async fn test_get_verse_translation() {
    let repo = create_test_content_repo().await;

    let translation = repo.get_verse_translation("1:1", 1).await.unwrap();

    assert!(translation.is_some());
    assert!(translation.unwrap().contains("Allah"));
}

#[tokio::test]
async fn test_multiple_translations_same_verse() {
    let repo = create_test_content_repo().await;

    let t1 = repo.get_verse_translation("1:1", 1).await.unwrap().unwrap();
    let t2 = repo.get_verse_translation("1:1", 2).await.unwrap().unwrap();

    assert_ne!(t1, t2);  // Different translators, different translations
}
```

### Integration Tests

```rust
#[tokio::test]
async fn test_user_preferred_translation() {
    let content_repo = create_test_content_repo().await;
    let user_repo = create_test_user_repo().await;

    // Set preference
    user_repo.set_preference("alice", "translator_id", "2").await.unwrap();

    // Get preferred translation
    let translator_id: i32 = user_repo
        .get_preference("alice", "translator_id")
        .await.unwrap()
        .unwrap()
        .parse().unwrap();

    let translation = content_repo
        .get_verse_translation("1:1", translator_id)
        .await.unwrap();

    assert!(translation.is_some());
}
```

## Validation Checklist

- [ ] `languages` table created with sample data (at least en, ar, ur, fr)
- [ ] `translators` table created with 3-5 English translators
- [ ] `verse_translations` table created with proper PKs and indexes
- [ ] Foreign keys defined with CASCADE semantics
- [ ] WITHOUT ROWID used for verse_translations and word_translations
- [ ] Repository trait updated with new methods
- [ ] All query methods implemented and tested
- [ ] User preferences table created in user.db
- [ ] Default translator preference set
- [ ] Migration from v1 tested (if applicable)
- [ ] Performance benchmarks meet targets (< 1ms per lookup)
- [ ] UI displays translator attribution

## Estimated Effort

- **Schema & Migration:** 1 day
- **Repository Implementation:** 1-2 days
- **Service Layer Updates:** 1 day
- **UI for Translator Selection:** 1 day (if applicable)
- **Testing:** 1 day

**Total:** 3-5 days

## References

- [01-content-schema-v2-purist.md](01-content-schema-v2-purist.md) - Full content.db schema
- [03-flexible-content-packages-plan.md](03-flexible-content-packages-plan.md) - Future package system

---

**Status:** Ready for implementation
**Next Steps:** Implement Step 1 (add tables to migration file)

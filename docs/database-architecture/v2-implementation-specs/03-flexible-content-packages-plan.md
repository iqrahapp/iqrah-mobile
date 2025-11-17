# Flexible Content Packages Plan

**Last Updated:** 2025-11-17
**Status:** Phased Implementation Plan
**Priority:** P2 (Post-MVP, Medium Priority)

## Context

Users need the ability to:
- Download additional translations beyond those shipped with the app
- Add alternative Arabic scripts (Imlaei, Indopak)
- Download audio recitations (verse-level and word-by-word)
- Add transliterations
- Manage installed content (enable/disable, uninstall)

**Current State:** Monolithic content.db ships with all content. No download capability.

**Target State:** Package management system where users can selectively download and install content packages.

## Goal

Define a **three-phase implementation plan** for flexible content management:
- **Phase 1:** Single translation, no packages (Current MVP)
- **Phase 2:** Multi-translation support without network downloads
- **Phase 3:** Full downloadable package system

Each phase builds incrementally on the previous.

## Phase 1: Single Translation (Current MVP)

**Status:** ✅ This is the baseline

### Schema

```sql
-- Simple approach
CREATE TABLE verse_translations (
    verse_key TEXT NOT NULL,
    translator_id INTEGER NOT NULL DEFAULT 1,
    translation TEXT NOT NULL,
    PRIMARY KEY (verse_key, translator_id),
    FOREIGN KEY (verse_key) REFERENCES verses(verse_key),
    FOREIGN KEY (translator_id) REFERENCES translators(translator_id)
);

-- Only one translator in DB
INSERT INTO translators (translator_id, slug, full_name, language_code)
VALUES (1, 'sahih-intl', 'Sahih International', 'en');

-- All verses for that translator
INSERT INTO verse_translations (verse_key, translator_id, translation)
SELECT verse_key, 1, translation FROM python_generated_data;
```

### Characteristics

✅ **Simple:** One translation, hardcoded
✅ **Small:** ~10-15 MB content.db
✅ **Fast:** Direct queries, no package logic
❌ **Limited:** No user choice

### Implementation

No changes needed - this is current state.

## Phase 2: Multi-Translation Support (Post-MVP)

**Status:** 📝 Ready to implement
**Effort:** 3-5 days
**Priority:** P2 (Nice to have for MVP, required for broader adoption)

### Goal

Ship with **3-5 popular English translations** in the app, let user select preferred translator.

### Schema Additions

**Already covered in:** [02-translations-and-translators-normalization.md](02-translations-and-translators-normalization.md)

```sql
-- Ship with multiple translators
INSERT INTO translators (slug, full_name, language_code, license) VALUES
    ('sahih-intl', 'Sahih International', 'en', 'Public Domain'),
    ('yusuf-ali', 'Abdullah Yusuf Ali', 'en', 'Public Domain'),
    ('pickthall', 'Marmaduke Pickthall', 'en', 'Public Domain'),
    ('khattab', 'Dr. Mustafa Khattab', 'en', 'CC BY-NC-ND 4.0'),
    ('hilali-khan', 'Dr. Muhsin Khan & Dr. Taqi-ud-Din al-Hilali', 'en', 'Public Domain');

-- All translations for all verses
INSERT INTO verse_translations (verse_key, translator_id, translation) VALUES
    ('1:1', 1, 'In the name of Allah, the Entirely Merciful...'),
    ('1:1', 2, 'In the name of God, Most Gracious, Most Merciful.'),
    ('1:1', 3, 'In the name of Allah, the Beneficent, the Merciful.'),
    ('1:1', 4, 'In the Name of Allah—the Most Compassionate...'),
    ('1:1', 5, 'In the Name of Allah, the Most Gracious...');
```

### User Preferences

**In user.db:**
```sql
CREATE TABLE user_preferences (
    user_id TEXT NOT NULL,
    preference_key TEXT NOT NULL,
    preference_value TEXT NOT NULL,
    PRIMARY KEY (user_id, preference_key)
);

-- Default preference
INSERT INTO user_preferences VALUES ('default', 'translator_id', '1');
```

### UI Flow

1. **Settings Screen:**
   ```
   Translation Settings
   ────────────────────
   [Radio] Sahih International (selected)
   [Radio] Abdullah Yusuf Ali
   [Radio] Marmaduke Pickthall
   [Radio] Dr. Mustafa Khattab
   [Radio] Hilali & Khan
   ```

2. **User selects translator → Save to user.db**

3. **Verse view queries:**
   ```rust
   let translator_id = user_repo.get_preference("default", "translator_id").await?;
   let translation = content_repo.get_verse_translation(verse_key, translator_id).await?;
   ```

### Characteristics

✅ **User Choice:** Select from 5 translators
✅ **No Network:** All data shipped with app
✅ **Simple Implementation:** No download logic
❌ **Larger App Size:** ~40-50 MB (5× translations)
❌ **No User-Added Content:** Cannot download additional translations

### Implementation Steps

**See:** [02-translations-and-translators-normalization.md](02-translations-and-translators-normalization.md) for detailed steps.

**Summary:**
1. Add translators table with 5 entries
2. Populate verse_translations for all translators
3. Add user_preferences table to user.db
4. Build translator selection UI
5. Update verse queries to use preferred translator_id

**Effort:** 3-5 days

## Phase 3: Full Package System (Future)

**Status:** 📋 Design spec
**Effort:** 2-3 weeks
**Priority:** P4 (When audio/script variants become priority)

### Goal

Allow users to **download and install** content packages on-demand:
- Additional translations (beyond the 5 shipped)
- Audio recitations (multiple reciters)
- Alternative Arabic scripts (Imlaei, Indopak)
- Word-by-word audio
- Transliterations

### Schema Additions

#### content_packages

```sql
CREATE TABLE content_packages (
    package_id TEXT PRIMARY KEY,  -- e.g., 'translation-en-arberry-v1'
    package_type TEXT NOT NULL CHECK (package_type IN (
        'verse_translation',
        'word_translation',
        'text_variant',
        'verse_recitation',
        'word_audio',
        'transliteration'
    )),
    name TEXT NOT NULL,                    -- "The Koran Interpreted (A.J. Arberry)"
    language_code TEXT,
    author TEXT,                           -- "A.J. Arberry"
    version TEXT NOT NULL,                 -- "1.0"
    description TEXT,
    file_size INTEGER,                     -- Bytes (for download progress)
    download_url TEXT,                     -- CDN URL
    checksum TEXT,                         -- SHA256 for integrity
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

**CASCADE:** Removing a package from catalog removes installation record.

#### Package-Linked Data Tables

**All already defined in:** [01-content-schema-v2-purist.md](01-content-schema-v2-purist.md)

- `verse_translations` - Already has translator_id (tied to packages)
- `text_variants` - Includes `package_id` FK
- `verse_recitations` - Includes `package_id` FK
- `word_audio` - Includes `package_id` FK
- `word_transliterations` - Includes `package_id` FK

### Package Workflow

#### 1. Browse Available Packages

**UI:**
```
Available Translations (English)
────────────────────────────────
[Download] A.J. Arberry - The Koran Interpreted (5.2 MB)
[Download] Muhammad Asad - The Message (6.1 MB)
[Download] Talal Itani - Clear Quran (4.8 MB)

Available Audio
────────────────────────────────
[Download] Mishary Rashid Alafasy - Full Quran (850 MB)
[Download] Abdul Basit - Murattal (720 MB)
```

**Query:**
```sql
SELECT package_id, name, author, file_size, download_url
FROM content_packages
WHERE package_type = 'verse_translation'
  AND language_code = 'en'
  AND package_id NOT IN (SELECT package_id FROM installed_packages)
ORDER BY name;
```

**Rust:**
```rust
pub async fn get_available_packages(
    &self,
    package_type: PackageType,
    language_code: Option<&str>,
) -> Result<Vec<ContentPackage>> {
    let mut query = QueryBuilder::new(
        "SELECT * FROM content_packages WHERE package_type = "
    );
    query.push_bind(package_type.to_string());

    if let Some(lang) = language_code {
        query.push(" AND language_code = ").push_bind(lang);
    }

    query.push(" AND package_id NOT IN (SELECT package_id FROM installed_packages)");
    query.push(" ORDER BY name");

    query.build_query_as::<ContentPackageRow>()
        .fetch_all(&self.pool)
        .await
}
```

#### 2. Download Package

**Rust Service:**
```rust
pub struct PackageService {
    content_repo: Arc<dyn ContentRepository>,
    http_client: reqwest::Client,
}

impl PackageService {
    pub async fn download_package(&self, package_id: &str) -> Result<Vec<u8>> {
        // 1. Get package metadata
        let package = self.content_repo.get_package(package_id).await?
            .ok_or(Error::PackageNotFound)?;

        // 2. Download from CDN
        let response = self.http_client
            .get(&package.download_url)
            .send()
            .await?;

        let bytes = response.bytes().await?;

        // 3. Verify checksum
        let hash = sha256::digest(&bytes);
        if hash != package.checksum {
            return Err(Error::ChecksumMismatch);
        }

        Ok(bytes.to_vec())
    }
}
```

**Package Format:** SQLite database file with just the package data.

**Example:** `translation-en-arberry-v1.db`
```sql
-- Contains only this translator's data
CREATE TABLE verse_translations (
    verse_key TEXT PRIMARY KEY,
    translation TEXT NOT NULL
);

INSERT INTO verse_translations VALUES
    ('1:1', 'In the Name of God, the Merciful, the Compassionate'),
    ('1:2', 'Praise belongs to God, the Lord of all Being'),
    ...;
```

#### 3. Install Package

**Rust Service:**
```rust
pub async fn install_package(
    &self,
    package_id: &str,
    package_data: Vec<u8>,
) -> Result<()> {
    // 1. Get package metadata
    let package = self.content_repo.get_package(package_id).await?
        .ok_or(Error::PackageNotFound)?;

    // 2. Open package DB (temporary)
    let temp_path = format!("/tmp/{}.db", package_id);
    std::fs::write(&temp_path, &package_data)?;
    let package_pool = SqlitePool::connect(&temp_path).await?;

    // 3. Begin transaction on main content.db
    let mut tx = self.content_repo.begin_transaction().await?;

    // 4. Import data based on package type
    match package.package_type {
        PackageType::VerseTranslation => {
            self.import_verse_translations(&package_pool, &mut tx, &package).await?;
        }
        PackageType::VerseRecitation => {
            self.import_verse_recitations(&package_pool, &mut tx, &package).await?;
        }
        // ... other types
    }

    // 5. Mark as installed
    sqlx::query(
        "INSERT INTO installed_packages (package_id, installed_at, enabled)
         VALUES (?, ?, 1)"
    )
    .bind(&package_id)
    .bind(Utc::now().timestamp())
    .execute(&mut *tx)
    .await?;

    // 6. Commit
    tx.commit().await?;

    // 7. Clean up
    std::fs::remove_file(&temp_path)?;

    Ok(())
}

async fn import_verse_translations(
    &self,
    package_pool: &SqlitePool,
    tx: &mut Transaction<'_, Sqlite>,
    package: &ContentPackage,
) -> Result<()> {
    // Get translator_id (created when package was added to catalog)
    let translator_id = self.get_or_create_translator_for_package(package).await?;

    // Batch import
    let rows: Vec<(String, String)> = sqlx::query_as(
        "SELECT verse_key, translation FROM verse_translations"
    )
    .fetch_all(package_pool)
    .await?;

    for (verse_key, translation) in rows {
        sqlx::query(
            "INSERT INTO verse_translations (verse_key, translator_id, translation)
             VALUES (?, ?, ?)
             ON CONFLICT (verse_key, translator_id) DO NOTHING"
        )
        .bind(&verse_key)
        .bind(translator_id)
        .bind(&translation)
        .execute(&mut **tx)
        .await?;
    }

    Ok(())
}
```

#### 4. Enable/Disable Package

**Toggle without deleting data:**
```sql
UPDATE installed_packages
SET enabled = CASE WHEN enabled = 1 THEN 0 ELSE 1 END
WHERE package_id = ?;
```

**Query only enabled packages:**
```sql
SELECT vt.translation
FROM verse_translations vt
JOIN translators t ON vt.translator_id = t.translator_id
JOIN installed_packages ip ON t.package_id = ip.package_id
WHERE vt.verse_key = ?
  AND ip.enabled = 1;
```

**Use Case:** User temporarily disables a translation without uninstalling (e.g., to reduce clutter).

#### 5. Uninstall Package

**Delete all package data:**
```sql
BEGIN TRANSACTION;

-- Get translator_id for this package
SELECT translator_id INTO @translator_id
FROM translators
WHERE package_id = ?;

-- Delete translations (cascade)
DELETE FROM verse_translations WHERE translator_id = @translator_id;

-- Delete translator
DELETE FROM translators WHERE translator_id = @translator_id;

-- Delete installation record (cascade from content_packages if we delete that too)
DELETE FROM installed_packages WHERE package_id = ?;

COMMIT;
```

**Rust:**
```rust
pub async fn uninstall_package(&self, package_id: &str) -> Result<()> {
    let mut tx = self.content_repo.begin_transaction().await?;

    // Get package metadata
    let package = self.content_repo.get_package(package_id).await?
        .ok_or(Error::PackageNotFound)?;

    match package.package_type {
        PackageType::VerseTranslation => {
            // Delete translator (cascade to verse_translations)
            sqlx::query("DELETE FROM translators WHERE package_id = ?")
                .bind(&package_id)
                .execute(&mut *tx)
                .await?;
        }
        PackageType::VerseRecitation => {
            // Delete recitations (cascade)
            sqlx::query("DELETE FROM verse_recitations WHERE package_id = ?")
                .bind(&package_id)
                .execute(&mut *tx)
                .await?;
        }
        // ... other types
    }

    // Remove installation record
    sqlx::query("DELETE FROM installed_packages WHERE package_id = ?")
        .bind(&package_id)
        .execute(&mut *tx)
        .await?;

    tx.commit().await?;

    Ok(())
}
```

### Package Catalog Management

**How packages get into catalog:**

**Option A: Shipped with app**
```sql
-- Packages pre-populated in content.db
INSERT INTO content_packages (package_id, package_type, name, download_url, ...) VALUES
    ('translation-en-arberry-v1', 'verse_translation', 'A.J. Arberry', 'https://cdn.iqrah.app/packages/...', ...);
```

**Option B: Fetched from server**
```rust
pub async fn refresh_package_catalog(&self) -> Result<()> {
    let response = self.http_client
        .get("https://api.iqrah.app/packages/catalog.json")
        .send()
        .await?;

    let catalog: Vec<PackageMeta> = response.json().await?;

    let mut tx = self.content_repo.begin_transaction().await?;

    for package in catalog {
        sqlx::query(
            "INSERT INTO content_packages (package_id, package_type, name, ...)
             VALUES (?, ?, ?, ...)
             ON CONFLICT (package_id) DO UPDATE SET updated_at = ?"
        )
        // ... bind values
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;

    Ok(())
}
```

**Recommendation:** Option A for MVP (ship catalog), Option B for long-term (dynamic updates).

### Cascade Semantics for Packages

| Action | Cascades To | Rationale |
|--------|-------------|-----------|
| Delete `content_packages` row | `installed_packages` | Installation record depends on package |
| Delete `content_packages` row | Package data (translations, audio, etc.) | Data shipped with package should be removed |
| Delete `translators` row | `verse_translations`, `word_translations` | Translations depend on translator |
| Delete `reciters` row | **RESTRICT** | Keep reciter metadata even if audio removed |

**Important:** Use `ON DELETE CASCADE` for package-related FKs to ensure clean uninstallation.

### Implementation Steps

#### Step 1: Schema (Already Done)

All tables defined in [01-content-schema-v2-purist.md](01-content-schema-v2-purist.md):
- `content_packages` ✅
- `installed_packages` ✅
- Package-linked tables (text_variants, verse_recitations, etc.) ✅

#### Step 2: Package Service

**File:** `rust/crates/iqrah-core/src/services/package_service.rs`

**Tasks:**
```rust
pub struct PackageService {
    content_repo: Arc<dyn ContentRepository>,
    http_client: reqwest::Client,
}

impl PackageService {
    pub async fn get_available_packages(&self, package_type: PackageType) -> Result<Vec<ContentPackage>>;
    pub async fn download_package(&self, package_id: &str) -> Result<Vec<u8>>;
    pub async fn install_package(&self, package_id: &str, data: Vec<u8>) -> Result<()>;
    pub async fn uninstall_package(&self, package_id: &str) -> Result<()>;
    pub async fn enable_package(&self, package_id: &str) -> Result<()>;
    pub async fn disable_package(&self, package_id: &str) -> Result<()>;
    pub async fn refresh_catalog(&self) -> Result<()>;
}
```

#### Step 3: Package Build Pipeline

**Python Tool:** `build_package.py`

```python
#!/usr/bin/env python3
# Generate a package SQLite file

def build_translation_package(translator_id: str, output_path: str):
    """Build a translation package DB."""
    # 1. Create temporary SQLite DB
    conn = sqlite3.connect(output_path)

    # 2. Create schema
    conn.execute("""
        CREATE TABLE verse_translations (
            verse_key TEXT PRIMARY KEY,
            translation TEXT NOT NULL
        )
    """)

    # 3. Populate from source
    for verse in load_quran_data():
        translation = get_translation(verse.verse_key, translator_id)
        conn.execute(
            "INSERT INTO verse_translations VALUES (?, ?)",
            (verse.verse_key, translation)
        )

    # 4. Optimize
    conn.execute("VACUUM")

    # 5. Checksum
    with open(output_path, 'rb') as f:
        checksum = hashlib.sha256(f.read()).hexdigest()

    return checksum

# Usage
checksum = build_translation_package('arberry', 'translation-en-arberry-v1.db')
print(f"Package built: checksum={checksum}")
```

#### Step 4: CDN/Hosting

**Options:**
- AWS S3 + CloudFront
- GitHub Releases (for open-source packages)
- Self-hosted server

**Upload packages to CDN, update catalog with download URLs.**

#### Step 5: UI Implementation

**Screens:**
1. **Package Browser** - List available packages with download buttons
2. **Download Progress** - Show download/install progress
3. **Installed Packages** - List installed, with enable/disable/uninstall actions
4. **Package Details** - Show license, description, size, etc.

**Effort:** 1-2 weeks (most of Phase 3 effort)

## Characteristics Comparison

| Feature | Phase 1 | Phase 2 | Phase 3 |
|---------|---------|---------|---------|
| **Translations** | 1 (Sahih Intl) | 5 (popular) | Unlimited (user downloads) |
| **App Size** | 10-15 MB | 40-50 MB | 15-20 MB (base) + user choice |
| **User Choice** | None | Preset selection | Full control |
| **Audio** | None | None | Downloadable |
| **Network Required** | No | No | Yes (for downloads) |
| **Implementation Effort** | 0 (current) | 3-5 days | 2-3 weeks |
| **Complexity** | Low | Low | High |

## Recommended Timeline

**MVP (Phase 1):**
- Ship with Sahih International translation only
- Fast time-to-market
- Small app size

**Post-MVP Enhancement (Phase 2):**
- Add 4 more translations
- Implement translator selection UI
- **Effort:** 3-5 days
- **Trigger:** User feedback requests more translations

**Future Enhancement (Phase 3):**
- Implement full package system
- Add audio recitation downloads
- **Effort:** 2-3 weeks
- **Trigger:** Audio feature becomes priority OR app size becomes a concern

## References

- [01-content-schema-v2-purist.md](01-content-schema-v2-purist.md) - Full schema including package tables
- [02-translations-and-translators-normalization.md](02-translations-and-translators-normalization.md) - Translator normalization (Phase 2)

---

**Status:** Phased plan ready
**Next Steps:** Implement Phase 2 after MVP launch, Phase 3 based on user demand

# Content Database Schema Documentation

**Version:** 2.0.0
**Status:** Production-Ready
**Date:** 2025-01-16

## Overview

The Iqrah content database is a production-ready SQLite database designed for mobile Quran applications. It implements a clean separation between **inflexible** data (always included) and **flexible** data (user-downloadable packages).

### Key Features

- **Optimized for Mobile**: Target size ~30-40MB for inflexible data
- **Complete Quranic Data**: 114 chapters, 6,236 verses, 77k+ words
- **Morphological Analysis**: 130k+ segments with lemmas, roots, and stems
- **Structural Metadata**: Juz, Hizb, Page, Sajdah tracking
- **Flexible Content**: Downloadable translations, reciters, text variants
- **Fast Queries**: Comprehensive indexes for exercise generation
- **Future-Proof**: Extensible package system for new content

## Architecture

### Content Separation

```
┌─────────────────────────────────────────────────────┐
│                  Mobile App                         │
├─────────────────────────────────────────────────────┤
│                                                     │
│  ┌──────────────────┐     ┌────────────────────┐   │
│  │ Knowledge Graph  │     │  Content Database  │   │
│  │   (CBOR ~10MB)   │────▶│   (SQLite ~40MB)   │   │
│  │                  │     │                    │   │
│  │ - Structure Only │     │ - Inflexible Data  │   │
│  │ - Node IDs       │     │ - Flexible Packages│   │
│  │ - Edges/Weights  │     │ - Indexed Lookups  │   │
│  │ - Scores         │     │                    │   │
│  └──────────────────┘     └────────────────────┘   │
│                                                     │
└─────────────────────────────────────────────────────┘
```

### Data Classification

#### Inflexible Data (Always Included)
- Chapter metadata (114 chapters)
- Verse metadata + Uthmani text (6,236 verses)
- Words with positions (77k+ words)
- Morphology segments (130k+ segments)
- Lemmas (~12k unique)
- Roots (~1.5k unique)
- Stems (~15k unique)

#### Flexible Data (User-Downloadable)
- Text variants (Imlaei, Indopak, Tajweed)
- Translations (multiple languages/translators)
- Word translations (word-by-word)
- Transliterations (multiple schemes)
- Reciters (audio files + URLs)

## Entity-Relationship Diagram

```mermaid
erDiagram
    chapters ||--o{ verses : contains
    verses ||--o{ words : contains
    verses ||--o{ morphology_segments : analyzed_in
    words ||--o{ morphology_segments : composed_of
    morphology_segments }o--|| lemmas : has
    morphology_segments }o--|| roots : derives_from
    morphology_segments }o--|| stems : contains

    content_packages ||--o{ installed_packages : tracks
    content_packages ||--o{ text_variants : provides
    content_packages ||--o{ verse_translations : provides
    content_packages ||--o{ word_translations : provides
    content_packages ||--o{ word_transliterations : provides
    content_packages ||--|| reciters : contains

    reciters ||--o{ verse_recitations : recites
    words ||--o| word_audio : pronounced_as

    verses ||--o{ text_variants : has_variant
    words ||--o{ text_variants : has_variant
    verses ||--o{ verse_translations : translated_as
    words ||--o{ word_translations : translated_as
    words ||--o{ word_transliterations : transliterated_as

    chapters {
        text node_id PK
        int chapter_number UK
        text name_arabic
        text name_simple
        text name_complex
        text name_transliterated
        text revelation_place
        int revelation_order
        bool bismillah_pre
        int verses_count
        text pages
    }

    verses {
        text node_id PK
        text verse_key UK
        int chapter_number FK
        int verse_number
        text text_uthmani
        int juz_number
        int hizb_number
        int rub_number
        int manzil_number
        int ruku_number
        int page_number
        text sajdah_type
        int sajdah_number
        int words_count
    }

    words {
        text node_id PK
        text verse_key FK
        int position
        text text_uthmani
        text char_type_name
        int page_number
        int line_number
    }

    morphology_segments {
        int id PK
        text verse_key FK
        int word_position
        int segment_index
        text segment_text
        text segment_type
        text lemma_id FK
        text root_id FK
        text stem_id FK
        text pos_tag
        text features_json
    }

    lemmas {
        text node_id PK
        text arabic UK
        text transliteration
        text meaning_en
        int occurrences_count
    }

    roots {
        text node_id PK
        text arabic UK
        text transliteration
        text meaning_en
        text root_type
        int occurrences_count
    }

    stems {
        text node_id PK
        text arabic UK
        text transliteration
        text pattern
        int occurrences_count
    }

    content_packages {
        text package_id PK
        text package_type
        text name
        text language_code
        text author
        text description
        text version
        int size_bytes
        bool is_default
        text metadata_json
        timestamp created_at
    }

    installed_packages {
        text package_id PK_FK
        timestamp installed_at
    }

    text_variants {
        int id PK
        text package_id FK
        text verse_key FK
        text word_id FK
        text text
    }

    verse_translations {
        int id PK
        text package_id FK
        text verse_key FK
        text text
        text footnotes_json
    }

    word_translations {
        int id PK
        text package_id FK
        text word_id FK
        text text
    }

    word_transliterations {
        int id PK
        text package_id FK
        text word_id FK
        text text
    }

    reciters {
        text reciter_id PK
        text package_id FK_UK
        text name_arabic
        text name_english
        text style
    }

    verse_recitations {
        int id PK
        text reciter_id FK
        text verse_key FK
        text audio_url
        int duration_ms
        text segments_json
    }

    word_audio {
        int id PK
        text word_id FK_UK
        text audio_url
        int duration_ms
    }
```

## Table Details

### Metadata Tables

#### schema_version
Tracks the database schema version for migration compatibility.

| Column | Type | Description |
|--------|------|-------------|
| version | TEXT PK | Semantic version (e.g., "2.0.0") |
| created_at | TIMESTAMP | Database creation timestamp |

#### content_packages
Catalog of available downloadable content packages.

| Column | Type | Description |
|--------|------|-------------|
| package_id | TEXT PK | Unique package identifier |
| package_type | TEXT | Type: text_variant, translation, word_translation, transliteration, reciter |
| name | TEXT | Display name |
| language_code | TEXT | ISO language code (e.g., "en", "ar") |
| author | TEXT | Author/translator name |
| description | TEXT | Package description |
| version | TEXT | Package version |
| size_bytes | INTEGER | Download size in bytes |
| is_default | BOOLEAN | Whether included by default |
| metadata_json | TEXT | Additional metadata as JSON |
| created_at | TIMESTAMP | Package creation timestamp |

**Package Types:**
- `text_variant`: Alternative text scripts (Imlaei, Indopak)
- `translation`: Verse translations
- `word_translation`: Word-by-word translations
- `transliteration`: Transliteration schemes
- `reciter`: Audio recitations

#### installed_packages
Tracks which content packages are currently installed on the device.

| Column | Type | Description |
|--------|------|-------------|
| package_id | TEXT PK, FK | Reference to content_packages |
| installed_at | TIMESTAMP | Installation timestamp |

---

### Inflexible Data Tables

#### chapters
Chapter (Surah) metadata for all 114 chapters.

| Column | Type | Description |
|--------|------|-------------|
| node_id | TEXT PK | Graph node ID (e.g., "CHAPTER:1") |
| chapter_number | INTEGER UK | Chapter number (1-114) |
| name_arabic | TEXT | Arabic name |
| name_simple | TEXT | Simple English name |
| name_complex | TEXT | Complex English name |
| name_transliterated | TEXT | Transliterated name |
| revelation_place | TEXT | "makkah" or "madinah" |
| revelation_order | INTEGER | Chronological revelation order |
| bismillah_pre | BOOLEAN | Has bismillah prefix |
| verses_count | INTEGER | Number of verses |
| pages | TEXT | Page ranges (as JSON array) |

**Constraints:**
- `chapter_number` BETWEEN 1 AND 114
- `revelation_place` IN ('makkah', 'madinah', NULL)

#### verses
Verse (Ayah) content and structural metadata for all 6,236 verses.

| Column | Type | Description |
|--------|------|-------------|
| node_id | TEXT PK | Graph node ID (e.g., "VERSE:1:1") |
| verse_key | TEXT UK | Verse reference (e.g., "1:1") |
| chapter_number | INTEGER FK | Chapter number |
| verse_number | INTEGER | Verse number within chapter |
| text_uthmani | TEXT | Uthmani Arabic text (default) |
| juz_number | INTEGER | Juz number (1-30) |
| hizb_number | INTEGER | Hizb number (1-60) |
| rub_number | INTEGER | Rub el Hizb number (1-240) |
| manzil_number | INTEGER | Manzil number (1-7) |
| ruku_number | INTEGER | Ruku number |
| page_number | INTEGER | Mushaf page (1-604) |
| sajdah_type | TEXT | "recommended" or "obligatory" |
| sajdah_number | INTEGER | Sajdah position number |
| words_count | INTEGER | Number of words in verse |

**Constraints:**
- `juz_number` BETWEEN 1 AND 30 OR NULL
- `hizb_number` BETWEEN 1 AND 60 OR NULL
- `page_number` BETWEEN 1 AND 604 OR NULL
- `sajdah_type` IN ('recommended', 'obligatory', NULL)

**Indexes:**
- `verse_key` (unique lookup)
- `chapter_number` (filter by chapter)
- `juz_number` (filter by juz)
- `hizb_number` (filter by hizb)
- `page_number` (filter by page)
- `rub_number` (filter by rub)

#### words
Word instance data for all 77k+ words in the Quran.

| Column | Type | Description |
|--------|------|-------------|
| node_id | TEXT PK | Graph node ID (e.g., "WORD_INSTANCE:1:1:1") |
| verse_key | TEXT FK | Verse reference |
| position | INTEGER | Word position in verse (1-indexed) |
| text_uthmani | TEXT | Uthmani Arabic text |
| char_type_name | TEXT | Character type classification |
| page_number | INTEGER | Mushaf page |
| line_number | INTEGER | Line on page |

**Constraints:**
- `UNIQUE(verse_key, position)`
- `position` > 0

**Indexes:**
- `verse_key` (lookup by verse)
- `(verse_key, position)` (unique position lookup)

#### morphology_segments
Morphological analysis segments from Quranic Arabic Corpus (130k+ segments).

| Column | Type | Description |
|--------|------|-------------|
| id | INTEGER PK | Auto-increment ID |
| verse_key | TEXT FK | Verse reference |
| word_position | INTEGER | Word position (1-indexed) |
| segment_index | INTEGER | Segment index within word (1-indexed) |
| segment_text | TEXT | Arabic text of segment |
| segment_type | TEXT | Type: PREFIX, ROOT, SUFFIX, etc. |
| lemma_id | TEXT FK | Reference to lemmas table |
| root_id | TEXT FK | Reference to roots table |
| stem_id | TEXT FK | Reference to stems table |
| pos_tag | TEXT | Part of speech tag |
| features_json | TEXT | Grammatical features (JSON array) |

**Constraints:**
- `UNIQUE(verse_key, word_position, segment_index)`
- `word_position` > 0
- `segment_index` > 0

**Indexes:**
- `verse_key` (lookup by verse)
- `(verse_key, word_position)` (lookup by word)
- `lemma_id` (filter by lemma)
- `root_id` (filter by root)
- `stem_id` (filter by stem)
- `pos_tag` (filter by POS tag)

**Example features_json:**
```json
["MASCULINE", "SINGULAR", "GENITIVE", "DEFINITE"]
```

#### lemmas
Unique lemmas (dictionary forms) extracted from morphology corpus (~12k).

| Column | Type | Description |
|--------|------|-------------|
| node_id | TEXT PK | Graph node ID (e.g., "LEMMA:كتب") |
| arabic | TEXT UK | Arabic lemma text |
| transliteration | TEXT | Transliteration |
| meaning_en | TEXT | English meaning/translation |
| occurrences_count | INTEGER | Occurrence count in Quran |

**Constraints:**
- `occurrences_count` >= 0

**Indexes:**
- `arabic` (lookup by Arabic text)

#### roots
Unique roots (triliteral/quadriliteral) extracted from morphology corpus (~1.5k).

| Column | Type | Description |
|--------|------|-------------|
| node_id | TEXT PK | Graph node ID (e.g., "ROOT:كتب") |
| arabic | TEXT UK | Arabic root text |
| transliteration | TEXT | Transliteration |
| meaning_en | TEXT | English meaning |
| root_type | TEXT | "triliteral" or "quadriliteral" |
| occurrences_count | INTEGER | Occurrence count in Quran |

**Constraints:**
- `occurrences_count` >= 0
- `root_type` IN ('triliteral', 'quadriliteral', NULL)

**Indexes:**
- `arabic` (lookup by Arabic text)

#### stems
Unique stems (words without affixes) extracted from morphology corpus (~15k).

| Column | Type | Description |
|--------|------|-------------|
| node_id | TEXT PK | Graph node ID (e.g., "STEM:كاتب") |
| arabic | TEXT UK | Arabic stem text |
| transliteration | TEXT | Transliteration |
| pattern | TEXT | Morphological pattern (e.g., فاعل) |
| occurrences_count | INTEGER | Occurrence count in Quran |

**Constraints:**
- `occurrences_count` >= 0

**Indexes:**
- `arabic` (lookup by Arabic text)

---

### Flexible Data Tables

#### text_variants
Alternative text scripts (Imlaei, Indopak, Tajweed).

| Column | Type | Description |
|--------|------|-------------|
| id | INTEGER PK | Auto-increment ID |
| package_id | TEXT FK | Reference to content_packages |
| verse_key | TEXT FK | Verse reference (verse-level variant) |
| word_id | TEXT FK | Word node ID (word-level variant) |
| text | TEXT | Variant text |

**Constraints:**
- Either `verse_key` OR `word_id` must be set (not both)

**Indexes:**
- `package_id` (filter by package)
- `verse_key` (lookup by verse)
- `word_id` (lookup by word)

#### verse_translations
Verse translations from various translators/languages.

| Column | Type | Description |
|--------|------|-------------|
| id | INTEGER PK | Auto-increment ID |
| package_id | TEXT FK | Reference to content_packages |
| verse_key | TEXT FK | Verse reference |
| text | TEXT | Translation text |
| footnotes_json | TEXT | Footnotes (JSON array) |

**Constraints:**
- `UNIQUE(package_id, verse_key)`

**Indexes:**
- `package_id` (filter by translator)
- `verse_key` (lookup by verse)

**Example footnotes_json:**
```json
[
  {"marker": "1", "text": "Allah is a proper name..."},
  {"marker": "2", "text": "Ar-Rahman and ar-Raheem..."}
]
```

#### word_translations
Word-by-word translations for vocabulary learning.

| Column | Type | Description |
|--------|------|-------------|
| id | INTEGER PK | Auto-increment ID |
| package_id | TEXT FK | Reference to content_packages |
| word_id | TEXT FK | Word node ID |
| text | TEXT | Translation text |

**Constraints:**
- `UNIQUE(package_id, word_id)`

**Indexes:**
- `package_id` (filter by translator)
- `word_id` (lookup by word)

#### word_transliterations
Word-by-word transliterations for pronunciation help.

| Column | Type | Description |
|--------|------|-------------|
| id | INTEGER PK | Auto-increment ID |
| package_id | TEXT FK | Reference to content_packages |
| word_id | TEXT FK | Word node ID |
| text | TEXT | Transliteration text |

**Constraints:**
- `UNIQUE(package_id, word_id)`

**Indexes:**
- `package_id` (filter by scheme)
- `word_id` (lookup by word)

#### reciters
Reciter metadata for audio packages.

| Column | Type | Description |
|--------|------|-------------|
| reciter_id | TEXT PK | Unique reciter ID |
| package_id | TEXT FK UK | Reference to content_packages |
| name_arabic | TEXT | Arabic name |
| name_english | TEXT | English name |
| style | TEXT | Recitation style |

**Constraints:**
- `style` IN ('murattal', 'mujawwad', 'muallim', NULL)

**Recitation Styles:**
- `murattal`: Measured, clear recitation
- `mujawwad`: Melodious recitation
- `muallim`: Educational/teaching style

#### verse_recitations
Verse-level audio recitations with timing data.

| Column | Type | Description |
|--------|------|-------------|
| id | INTEGER PK | Auto-increment ID |
| reciter_id | TEXT FK | Reference to reciters |
| verse_key | TEXT FK | Verse reference |
| audio_url | TEXT | Audio file URL |
| duration_ms | INTEGER | Duration in milliseconds |
| segments_json | TEXT | Word-level timing segments (JSON) |

**Constraints:**
- `UNIQUE(reciter_id, verse_key)`

**Indexes:**
- `reciter_id` (filter by reciter)
- `verse_key` (lookup by verse)

**Example segments_json:**
```json
[
  {"word_position": 1, "start_ms": 0, "end_ms": 450},
  {"word_position": 2, "start_ms": 450, "end_ms": 890}
]
```

#### word_audio
Word-level audio pronunciations (shared across reciters).

| Column | Type | Description |
|--------|------|-------------|
| id | INTEGER PK | Auto-increment ID |
| word_id | TEXT FK UK | Word node ID |
| audio_url | TEXT | Audio file URL |
| duration_ms | INTEGER | Duration in milliseconds |

**Indexes:**
- `word_id` (unique lookup by word)

---

## Common Query Patterns

### 1. Get Verse with Complete Data

```sql
SELECT
    v.verse_key,
    v.text_uthmani,
    v.juz_number,
    v.page_number,
    c.name_simple AS chapter_name
FROM verses v
JOIN chapters c ON v.chapter_number = c.chapter_number
WHERE v.verse_key = '1:1';
```

### 2. Get Words for Verse with Morphology

```sql
SELECT
    w.position,
    w.text_uthmani,
    m.segment_text,
    m.segment_type,
    m.pos_tag,
    l.arabic AS lemma,
    r.arabic AS root
FROM words w
LEFT JOIN morphology_segments m ON w.verse_key = m.verse_key
    AND w.position = m.word_position
LEFT JOIN lemmas l ON m.lemma_id = l.node_id
LEFT JOIN roots r ON m.root_id = r.node_id
WHERE w.verse_key = '1:1'
ORDER BY w.position, m.segment_index;
```

### 3. Filter Verses by Juz

```sql
SELECT verse_key, text_uthmani
FROM verses
WHERE juz_number = 1
ORDER BY chapter_number, verse_number;
```

### 4. Find Words by Root

```sql
SELECT DISTINCT
    w.verse_key,
    w.position,
    w.text_uthmani,
    r.arabic AS root
FROM morphology_segments m
JOIN roots r ON m.root_id = r.node_id
JOIN words w ON m.verse_key = w.verse_key
    AND m.word_position = w.position
WHERE r.arabic = 'كتب'
ORDER BY w.verse_key, w.position;
```

### 5. Get Verse with Translation

```sql
SELECT
    v.verse_key,
    v.text_uthmani,
    vt.text AS translation
FROM verses v
LEFT JOIN verse_translations vt ON v.verse_key = vt.verse_key
LEFT JOIN content_packages cp ON vt.package_id = cp.package_id
WHERE v.verse_key = '1:1'
    AND cp.language_code = 'en'
    AND cp.author = 'Sahih International';
```

### 6. Get Most Common Lemmas

```sql
SELECT arabic, occurrences_count
FROM lemmas
ORDER BY occurrences_count DESC
LIMIT 100;
```

### 7. Get Recitation Audio

```sql
SELECT
    r.name_english,
    vr.audio_url,
    vr.duration_ms,
    vr.segments_json
FROM verse_recitations vr
JOIN reciters r ON vr.reciter_id = r.reciter_id
WHERE vr.verse_key = '1:1'
    AND r.name_english = 'Mahmoud Khalil Al-Husary';
```

---

## Example Queries for Exercise Generation

### 1. Generate Root-Based Exercise

Find all words derived from a specific root for memorization:

```sql
SELECT DISTINCT
    w.verse_key,
    w.text_uthmani,
    r.arabic AS root,
    v.text_uthmani AS verse_text,
    c.name_simple AS chapter_name
FROM morphology_segments m
JOIN roots r ON m.root_id = r.node_id
JOIN words w ON m.verse_key = w.verse_key
    AND m.word_position = w.position
JOIN verses v ON w.verse_key = v.verse_key
JOIN chapters c ON v.chapter_number = c.chapter_number
WHERE r.arabic = 'صلو'  -- root for prayer
ORDER BY v.chapter_number, v.verse_number;
```

### 2. Generate Lemma Vocabulary Quiz

Find all occurrences of a lemma with context:

```sql
SELECT
    w.verse_key,
    w.text_uthmani AS word,
    v.text_uthmani AS verse,
    l.meaning_en AS meaning
FROM morphology_segments m
JOIN lemmas l ON m.lemma_id = l.node_id
JOIN words w ON m.verse_key = w.verse_key
    AND m.word_position = w.position
JOIN verses v ON w.verse_key = v.verse_key
WHERE l.arabic = 'صَلاة'
ORDER BY v.chapter_number, v.verse_number;
```

### 3. Filter by Part of Speech

Find all verbs in a specific verse:

```sql
SELECT
    w.position,
    w.text_uthmani,
    m.pos_tag,
    m.features_json
FROM words w
JOIN morphology_segments m ON w.verse_key = m.verse_key
    AND w.position = m.word_position
WHERE w.verse_key = '2:1'
    AND m.pos_tag LIKE 'V%'  -- Verb tags start with V
ORDER BY w.position, m.segment_index;
```

---

## Database Size Optimization

### Target Sizes

| Component | Target Size | Actual Size |
|-----------|-------------|-------------|
| Inflexible data | ~30-40 MB | TBD (after build) |
| Default packages | ~5-10 MB | Optional |
| Per reciter | ~100-200 MB | User choice |
| Per translation | ~1-2 MB | User choice |

### Optimization Techniques Applied

1. **Normalization**: Eliminate redundant data
2. **Indexes**: Only critical queries (avoid over-indexing)
3. **Foreign Keys**: Enforce relationships, enable cascading
4. **JSON Storage**: Flexible data (features, footnotes) as JSON
5. **VACUUM**: Reclaim deleted space
6. **ANALYZE**: Update query planner statistics
7. **WAL Mode**: Write-Ahead Logging for better performance

### Compression Considerations

SQLite has built-in page compression, but for additional savings:
- Store audio URLs instead of embedding audio
- Use relative URLs where possible
- Consider ZSTD compression for downloadable packages

---

## Migration Strategy

### Schema Versioning

The `schema_version` table tracks the current schema version. When updating the schema:

1. Increment version number (e.g., 2.0.0 → 2.1.0)
2. Create migration SQL scripts
3. Test on a copy of production data
4. Apply migrations with rollback capability

### Update Strategy

For inflexible data updates:
- **Full Replacement**: Delete and rebuild content.db
- Inflexible data rarely changes (Quran text is fixed)
- Simple, reliable, no migration complexity

For flexible data updates:
- **Incremental Updates**: Install/uninstall packages
- Update package version in `content_packages`
- Replace package data in relevant tables

---

## Security Considerations

### SQL Injection Prevention

- Always use parameterized queries
- Never concatenate user input into SQL
- Validate all input data

### Data Integrity

- Foreign key constraints enabled (`PRAGMA foreign_keys = ON`)
- Check constraints on critical fields
- Unique constraints prevent duplicates

### Access Control

- Read-only access for app queries
- Write access only for package installation
- Separate user permissions for admin operations

---

## Performance Benchmarks

### Expected Query Performance (on mid-range mobile device)

| Query Type | Expected Time | Notes |
|------------|---------------|-------|
| Single verse lookup | <1 ms | Indexed by verse_key |
| Words for verse | <2 ms | Indexed join |
| Morphology for word | <3 ms | Multiple joins |
| Filter by juz/page | <5 ms | Indexed filter |
| Root search (all words) | <50 ms | Full table scan with index |
| Complex exercise query | <100 ms | Multiple joins + filters |

### Database Build Performance

| Operation | Expected Time | Notes |
|-----------|---------------|-------|
| Schema creation | <1 sec | |
| Chapters | <1 sec | 114 rows |
| Verses | <3 sec | 6,236 rows |
| Words | <10 sec | 77k+ rows |
| Morphology | <30 sec | 130k+ rows |
| Indexes | <5 sec | |
| VACUUM/ANALYZE | <10 sec | |
| **Total** | **~60 sec** | Full build |

---

## Future Enhancements

### Planned Features

1. **Tafsir Integration**
   - Add `tafsir` table for verse commentaries
   - Link to content packages for different scholars

2. **Tajweed Rules**
   - Add `tajweed_rules` table
   - Link to morphology segments for rule application

3. **User Progress Tracking**
   - Add `user_progress` table
   - Track memorization, reading progress
   - Store offline (sync to cloud later)

4. **Advanced Search**
   - Full-text search indexes
   - Fuzzy matching for Arabic text
   - Semantic search integration

5. **Cross-References**
   - Add `verse_relations` table
   - Link related verses (duplicates, themes, etc.)
   - Integrate with knowledge graph

---

## References

- [Quranic Arabic Corpus](http://corpus.quran.com/)
- [Tanzil Quran Text](http://tanzil.net/docs/tanzil_text)
- [SQLite Documentation](https://www.sqlite.org/docs.html)
- [Quran.com API](https://quran.api-docs.io/)

---

## Appendix: Node ID Format

All node IDs follow a consistent format for graph integration:

| Entity | Format | Example |
|--------|--------|---------|
| Chapter | `CHAPTER:{number}` | `CHAPTER:1` |
| Verse | `VERSE:{chapter}:{verse}` | `VERSE:1:1` |
| Word Instance | `WORD_INSTANCE:{chapter}:{verse}:{position}` | `WORD_INSTANCE:1:1:1` |
| Lemma | `LEMMA:{arabic}` | `LEMMA:كتب` |
| Root | `ROOT:{arabic}` | `ROOT:كتب` |
| Stem | `STEM:{arabic}` | `STEM:كاتب` |

These IDs are used for:
1. Primary keys in content database
2. Node identifiers in knowledge graph
3. Cross-referencing between systems

---

## Contact

For questions, issues, or contributions, see the main Iqrah repository.

**Schema Version:** 2.0.0
**Last Updated:** 2025-01-16

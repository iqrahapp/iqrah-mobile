-- ============================================================================
-- Unified Content Database Schema v2
-- Date: 2025-11-26
-- This single migration file defines the complete, final schema for the content
-- database, including all necessary tables and test data.
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
VALUES ('2.0.0', 'Unified v2 schema with integer IDs and test data');

-- ============================================================================
-- NODES REGISTRY (Central authority for all nodes)
-- ============================================================================

CREATE TABLE IF NOT EXISTS nodes (
    id INTEGER PRIMARY KEY,
    ukey TEXT NOT NULL UNIQUE,
    node_type INTEGER NOT NULL
) STRICT;

CREATE INDEX idx_nodes_ukey ON nodes(ukey);

-- ============================================================================
-- CORE QURANIC STRUCTURE
-- ============================================================================

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

CREATE TABLE verses (
    verse_key TEXT PRIMARY KEY,
    chapter_number INTEGER NOT NULL,
    verse_number INTEGER NOT NULL,
    text_uthmani TEXT NOT NULL,
    text_simple TEXT,
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

CREATE TABLE words (
    word_id INTEGER PRIMARY KEY,
    verse_key TEXT NOT NULL,
    position INTEGER NOT NULL,
    text_uthmani TEXT NOT NULL,
    text_simple TEXT,
    transliteration TEXT,
    letter_count INTEGER,
    created_at INTEGER NOT NULL DEFAULT (unixepoch()),
    FOREIGN KEY (verse_key) REFERENCES verses(verse_key)
) STRICT;

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
-- POPULATE NODES TABLE (from verses)
-- This must be done after verses are defined but before other tables reference nodes.
-- ============================================================================

-- Sample Chapters (Al-Fatihah, Al-Baqarah, Al-Imran for testing)
INSERT INTO chapters (chapter_number, name_arabic, name_transliteration, name_translation, revelation_place, revelation_order, bismillah_pre, verse_count, page_start, page_end) VALUES
    (1, 'الفاتحة', 'Al-Fatihah', 'The Opening', 'makkah', 5, 1, 7, 1, 1),
    (2, 'البقرة', 'Al-Baqarah', 'The Cow', 'madinah', 87, 1, 286, 2, 49),
    (3, 'آل عمران', 'Al-Imran', 'The Family of Imran', 'madinah', 89, 1, 200, 50, 76);

-- Sample verses from Al-Fatihah (Chapter 1) with full details
INSERT INTO verses (verse_key, chapter_number, verse_number, text_uthmani, text_simple, juz, hizb, rub_el_hizb, page, manzil, word_count) VALUES
    ('1:1', 1, 1, 'بِسْمِ ٱللَّهِ ٱلرَّحْمَٰنِ ٱلرَّحِيمِ', 'بسم الله الرحمن الرحيم', 1, 1, 1, 1, 1, 4),
    ('1:2', 1, 2, 'ٱلْحَمْدُ لِلَّهِ رَبِّ ٱلْعَٰلَمِينَ', 'الحمد لله رب العالمين', 1, 1, 1, 1, 1, 4),
    ('1:3', 1, 3, 'ٱلرَّحْمَٰنِ ٱلرَّحِيمِ', 'الرحمن الرحيم', 1, 1, 1, 1, 1, 2),
    ('1:4', 1, 4, 'مَٰلِكِ يَوْمِ ٱلدِّينِ', 'مالك يوم الدين', 1, 1, 1, 1, 1, 3),
    ('1:5', 1, 5, 'إِيَّاكَ نَعْبُدُ وَإِيَّاكَ نَسْتَعِينُ', 'اياك نعبد واياك نستعين', 1, 1, 1, 1, 1, 4),
    ('1:6', 1, 6, 'ٱهْدِنَا ٱلصِّرَٰطَ ٱلْمُسْتَقِيمَ', 'اهدنا الصراط المستقيم', 1, 1, 1, 1, 1, 3),
    ('1:7', 1, 7, 'صِرَٰطَ ٱلَّذِينَ أَنْعَمْتَ عَلَيْهِمْ غَيْرِ ٱلْمَغْضُوبِ عَلَيْهِمْ وَلَا ٱلضَّآلِّينَ', 'صراط الذين انعمت عليهم غير المغضوب عليهم ولا الضالين', 1, 1, 1, 1, 1, 10);

-- Generate placeholder verses for chapters 2-3 using recursive CTE
WITH RECURSIVE
  chapter_verses(chapter_num, verse_num, max_verses) AS (
    SELECT 2, 1, 286
    UNION ALL
    SELECT
      CASE WHEN verse_num < max_verses THEN chapter_num ELSE chapter_num + 1 END,
      CASE WHEN verse_num < max_verses THEN verse_num + 1 ELSE 1 END,
      CASE
        WHEN verse_num < max_verses THEN max_verses
        WHEN chapter_num = 2 THEN 200
        ELSE 0
      END
    FROM chapter_verses
    WHERE chapter_num < 3 OR (chapter_num = 3 AND verse_num < 200)
  )
INSERT INTO verses (verse_key, chapter_number, verse_number, text_uthmani, juz, hizb, rub_el_hizb, page, manzil, word_count)
SELECT
  chapter_num || ':' || verse_num,
  chapter_num,
  verse_num,
  'Verse ' || chapter_num || ':' || verse_num,
  CASE WHEN chapter_num = 2 THEN (verse_num / 25) + 1 ELSE 15 + (verse_num / 20) END,
  1,
  1,
  1,
  1,
  1
FROM chapter_verses;

-- ============================================================================
-- SAMPLE LANGUAGES
-- ============================================================================

INSERT INTO languages (language_code, english_name, native_name, direction) VALUES
    ('en', 'English', 'English', 'ltr'),
    ('ar', 'Arabic', 'العربية', 'rtl'),
    ('fr', 'French', 'Français', 'ltr'),
    ('ur', 'Urdu', 'اردو', 'rtl'),
    ('id', 'Indonesian', 'Indonesia', 'ltr'),
    ('tr', 'Turkish', 'Türkçe', 'ltr'),
    ('es', 'Spanish', 'Español', 'ltr');

-- ============================================================================
-- SAMPLE TRANSLATORS
-- ============================================================================

INSERT INTO translators (slug, full_name, language_code, description, license, website, version) VALUES
    ('sahih-intl', 'Sahih International', 'en', 'Clear and modern English translation', 'Public Domain', 'https://quran.com', '1.0'),
    ('yusuf-ali', 'Abdullah Yusuf Ali', 'en', 'Classic English translation with commentary', 'Public Domain', 'https://www.al-islam.org', '1.0'),
    ('pickthall', 'Marmaduke Pickthall', 'en', 'First English translation by a Muslim', 'Public Domain', NULL, '1.0'),
    ('khattab', 'Dr. Mustafa Khattab', 'en', 'The Clear Quran - Contemporary English', 'CC BY-NC-ND 4.0', 'https://theclearquran.org', '1.0'),
    ('hilali-khan', 'Dr. Muhsin Khan & Dr. Taqi-ud-Din al-Hilali', 'en', 'Noble Quran - Literal translation', 'Public Domain', NULL, '1.0');

-- ============================================================================
-- SAMPLE WORDS (verse 1:1 only)
-- ============================================================================

INSERT INTO words (verse_key, position, text_uthmani, text_simple, transliteration) VALUES
    ('1:1', 1, 'بِسْمِ', 'بسم', 'bismi'),
    ('1:1', 2, 'ٱللَّهِ', 'الله', 'Allāhi'),
    ('1:1', 3, 'ٱلرَّحْمَٰنِ', 'الرحمن', 'al-Raḥmāni'),
    ('1:1', 4, 'ٱلرَّحِيمِ', 'الرحيم', 'al-Raḥīmi');

-- Populate nodes table from the sample verses with encoded IDs
-- ID encoding: (TYPE_VERSE << 56) | (chapter << 16) | verse
-- TYPE_VERSE = 2, so the formula is: (2 << 56) | (chapter << 16) | verse
-- In SQLite: (CAST(2 AS INTEGER) << 56) | (chapter_number << 16) | verse_number
INSERT OR IGNORE INTO nodes (id, ukey, node_type)
SELECT
  (CAST(2 AS INTEGER) << 56) | (chapter_number << 16) | verse_number,
  'VERSE:' || verse_key,
  1
FROM verses; -- NodeType::Verse = 1

-- ============================================================================
-- SAMPLE VERSE TRANSLATIONS (verse 1:1 only)
-- ============================================================================

INSERT INTO verse_translations (verse_key, translator_id, translation)
SELECT '1:1', translator_id, 'In the name of Allah, the Entirely Merciful, the Especially Merciful.' FROM translators WHERE slug = 'sahih-intl'
UNION ALL SELECT '1:1', translator_id, 'In the name of God, Most Gracious, Most Merciful.' FROM translators WHERE slug = 'yusuf-ali'
UNION ALL SELECT '1:1', translator_id, 'In the name of Allah, the Beneficent, the Merciful.' FROM translators WHERE slug = 'pickthall'
UNION ALL SELECT '1:1', translator_id, 'In the Name of Allah—the Most Compassionate, Most Merciful.' FROM translators WHERE slug = 'khattab'
UNION ALL SELECT '1:1', translator_id, 'In the Name of Allah, the Most Gracious, the Most Merciful.' FROM translators WHERE slug = 'hilali-khan';

-- ============================================================================
-- SAMPLE WORD TRANSLATIONS (verse 1:1, Sahih International only)
-- ============================================================================

INSERT INTO word_translations (word_id, translator_id, translation)
SELECT w.word_id, t.translator_id, 'In the name' FROM words w, translators t WHERE w.verse_key = '1:1' AND w.position = 1 AND t.slug = 'sahih-intl'
UNION ALL SELECT w.word_id, t.translator_id, 'of Allah' FROM words w, translators t WHERE w.verse_key = '1:1' AND w.position = 2 AND t.slug = 'sahih-intl'
UNION ALL SELECT w.word_id, t.translator_id, 'the Entirely Merciful' FROM words w, translators t WHERE w.verse_key = '1:1' AND w.position = 3 AND t.slug = 'sahih-intl'
UNION ALL SELECT w.word_id, t.translator_id, 'the Especially Merciful' FROM words w, translators t WHERE w.verse_key = '1:1' AND w.position = 4 AND t.slug = 'sahih-intl';

-- ============================================================================
-- SCHEDULER V2 TABLES (with integer foreign keys)
-- ============================================================================

CREATE TABLE IF NOT EXISTS goals (
    goal_id TEXT PRIMARY KEY,
    goal_type TEXT NOT NULL,
    goal_group TEXT NOT NULL,
    label TEXT NOT NULL,
    description TEXT
) STRICT;

CREATE TABLE IF NOT EXISTS node_goals (
    goal_id TEXT NOT NULL,
    node_id INTEGER NOT NULL,
    priority INTEGER DEFAULT 0,
    PRIMARY KEY (goal_id, node_id),
    FOREIGN KEY (goal_id) REFERENCES goals(goal_id) ON DELETE CASCADE,
    FOREIGN KEY (node_id) REFERENCES nodes(id) ON DELETE CASCADE
) STRICT, WITHOUT ROWID;

CREATE TABLE IF NOT EXISTS node_metadata (
    node_id INTEGER NOT NULL,
    key TEXT NOT NULL,
    value REAL NOT NULL,
    PRIMARY KEY (node_id, key),
    FOREIGN KEY (node_id) REFERENCES nodes(id) ON DELETE CASCADE
) STRICT, WITHOUT ROWID;

CREATE TABLE IF NOT EXISTS edges (
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

-- ============================================================================
-- POPULATE TEST DATA (using integer IDs)
-- ============================================================================

INSERT OR IGNORE INTO goals (goal_id, goal_type, goal_group, label, description) VALUES
    ('memorization:chapters-1-3', 'custom', 'memorization', 'Memorize Chapters 1-3', 'Master all 493 verses from Al-Fatihah, Al-Baqarah, and Al-Imran');

-- Add all verses from chapters 1-3 to the goal
INSERT OR IGNORE INTO node_goals (goal_id, node_id, priority)
SELECT 'memorization:chapters-1-3', id, 1001000
FROM nodes
WHERE ukey LIKE 'VERSE:1:%' OR ukey LIKE 'VERSE:2:%' OR ukey LIKE 'VERSE:3:%';

-- Add metadata for all verses (foundational, influence, difficulty scores, quran_order)
-- Use deterministic scores based on verse position
INSERT OR IGNORE INTO node_metadata (node_id, key, value)
SELECT n.id, 'foundational_score',
  CASE
    WHEN v.chapter_number = 1 AND v.verse_number = 1 THEN 0.85
    ELSE 0.1 + (CAST(v.chapter_number AS REAL) * 0.01) + (CAST(v.verse_number AS REAL) * 0.001)
  END
FROM nodes n
JOIN verses v ON n.ukey = 'VERSE:' || v.verse_key
WHERE n.ukey LIKE 'VERSE:%'
UNION ALL
SELECT n.id, 'influence_score',
  CASE
    WHEN v.chapter_number = 1 AND v.verse_number = 1 THEN 0.90
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
WHERE n.ukey LIKE 'VERSE:%';

-- Create sequential prerequisite edges (each verse depends on the previous one)
INSERT OR IGNORE INTO edges (source_id, target_id, edge_type, distribution_type)
SELECT
  curr.id AS source_id,
  next.id AS target_id,
  0 AS edge_type,
  0 AS distribution_type
FROM nodes curr
JOIN verses curr_v ON curr.ukey = 'VERSE:' || curr_v.verse_key
JOIN verses next_v ON
  curr_v.chapter_number = next_v.chapter_number AND
  next_v.verse_number = curr_v.verse_number + 1
JOIN nodes next ON next.ukey = 'VERSE:' || next_v.verse_key
WHERE curr.ukey LIKE 'VERSE:%' AND next.ukey LIKE 'VERSE:%';

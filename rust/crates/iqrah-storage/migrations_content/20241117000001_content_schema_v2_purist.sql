-- Content Database Schema v2 (Purist Approach)
-- This database is READ-ONLY at runtime and ships with the app
-- Migration from v1 to v2: Complete schema redesign
-- Date: 2025-11-17

-- ============================================================================
-- SCHEMA VERSION TRACKING
-- ============================================================================

CREATE TABLE schema_version (
    version INTEGER PRIMARY KEY
) STRICT;

INSERT INTO schema_version (version) VALUES (2);

-- ============================================================================
-- CORE QURANIC STRUCTURE (Inflexible Data)
-- ============================================================================

-- Chapters (Surahs)
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

-- Verses (Ayahs)
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

-- Words (Word Instances)
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

-- ============================================================================
-- MORPHOLOGICAL DATA (Inflexible)
-- ============================================================================

-- Roots (Morphological roots)
CREATE TABLE roots (
    root_id TEXT PRIMARY KEY,  -- Semantic key (e.g., "ktb", "drs")
    arabic TEXT NOT NULL UNIQUE,
    transliteration TEXT,
    root_type TEXT DEFAULT 'trilateral' CHECK (root_type IN ('trilateral', 'quadrilateral', 'quinqueliteral') OR root_type IS NULL),
    meaning TEXT,
    created_at INTEGER NOT NULL DEFAULT (unixepoch())
) STRICT;

-- Lemmas (Dictionary headwords)
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

-- Stems (Morphological patterns)
CREATE TABLE stems (
    stem_id TEXT PRIMARY KEY,  -- Pattern identifier (e.g., "form-I", "form-IV")
    pattern TEXT NOT NULL,
    description TEXT,
    example TEXT,
    created_at INTEGER NOT NULL DEFAULT (unixepoch())
) STRICT;

-- Morphology Segments (Word analysis)
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

-- ============================================================================
-- FLEXIBLE CONTENT (Normalized Translators)
-- ============================================================================

-- Languages
CREATE TABLE languages (
    language_code TEXT PRIMARY KEY,  -- ISO 639-1 (e.g., 'en', 'ar', 'fr')
    english_name TEXT NOT NULL,      -- 'English', 'Arabic', 'French'
    native_name TEXT NOT NULL,       -- 'English', 'العربية', 'Français'
    direction TEXT NOT NULL DEFAULT 'ltr' CHECK (direction IN ('ltr', 'rtl'))
) STRICT;

-- Translators
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
    package_id TEXT,                     -- Link to content_packages (NULL for built-in translators)
    created_at INTEGER NOT NULL DEFAULT (unixepoch()),
    FOREIGN KEY (language_code) REFERENCES languages(language_code),
    FOREIGN KEY (package_id) REFERENCES content_packages(package_id) ON DELETE CASCADE
) STRICT;

CREATE INDEX idx_translators_language ON translators(language_code);
CREATE INDEX idx_translators_package ON translators(package_id);
CREATE INDEX idx_translators_slug ON translators(slug);

-- Verse Translations
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

-- Word Translations
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

-- ============================================================================
-- PACKAGE MANAGEMENT
-- ============================================================================

-- Content Packages
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

-- Installed Packages
CREATE TABLE installed_packages (
    package_id TEXT PRIMARY KEY,
    installed_at INTEGER NOT NULL DEFAULT (unixepoch()),
    enabled INTEGER NOT NULL DEFAULT 1 CHECK (enabled IN (0, 1)),
    FOREIGN KEY (package_id) REFERENCES content_packages(package_id) ON DELETE CASCADE
) STRICT;

-- ============================================================================
-- TEXT VARIANTS & TRANSLITERATIONS
-- ============================================================================

-- Text Variants (Alternative Arabic scripts)
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

-- Word Transliterations
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

-- ============================================================================
-- AUDIO & RECITATIONS
-- ============================================================================

-- Reciters
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

-- Verse Recitations
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

-- Word Audio
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

-- ============================================================================
-- KNOWLEDGE GRAPH (Kept from v1 for graph functionality)
-- ============================================================================

-- Knowledge graph edges (kept for energy propagation and relationships)
-- Note: Nodes are now stored in domain tables (chapters, verses, words)
-- The graph uses content keys (verse_key, word_id, etc.) as node references
CREATE TABLE IF NOT EXISTS edges (
    source_id TEXT NOT NULL,
    target_id TEXT NOT NULL,
    edge_type INTEGER NOT NULL CHECK (edge_type IN (0, 1)), -- 0:Dependency, 1:Knowledge
    distribution_type INTEGER NOT NULL CHECK (distribution_type IN (0, 1, 2)), -- 0:Const, 1:Normal, 2:Beta
    param1 REAL NOT NULL DEFAULT 0.0,
    param2 REAL NOT NULL DEFAULT 0.0,
    PRIMARY KEY (source_id, target_id)
) STRICT, WITHOUT ROWID;

CREATE INDEX IF NOT EXISTS idx_edges_source ON edges(source_id);
CREATE INDEX IF NOT EXISTS idx_edges_target ON edges(target_id);

-- ============================================================================
-- SAMPLE DATA FOR TESTING
-- ============================================================================

-- Sample languages
INSERT INTO languages (language_code, english_name, native_name, direction) VALUES
    ('en', 'English', 'English', 'ltr'),
    ('ar', 'Arabic', 'العربية', 'rtl'),
    ('fr', 'French', 'Français', 'ltr'),
    ('ur', 'Urdu', 'اردو', 'rtl'),
    ('id', 'Indonesian', 'Indonesia', 'ltr'),
    ('tr', 'Turkish', 'Türkçe', 'ltr'),
    ('es', 'Spanish', 'Español', 'ltr');

-- Sample translators
INSERT INTO translators (slug, full_name, language_code, description, license, website, version) VALUES
    ('sahih-intl', 'Sahih International', 'en', 'Clear and modern English translation', 'Public Domain', 'https://quran.com', '1.0'),
    ('yusuf-ali', 'Abdullah Yusuf Ali', 'en', 'Classic English translation with commentary', 'Public Domain', 'https://www.al-islam.org', '1.0'),
    ('pickthall', 'Marmaduke Pickthall', 'en', 'First English translation by a Muslim', 'Public Domain', NULL, '1.0'),
    ('khattab', 'Dr. Mustafa Khattab', 'en', 'The Clear Quran - Contemporary English', 'CC BY-NC-ND 4.0', 'https://theclearquran.org', '1.0'),
    ('hilali-khan', 'Dr. Muhsin Khan & Dr. Taqi-ud-Din al-Hilali', 'en', 'Noble Quran - Literal translation', 'Public Domain', NULL, '1.0');

-- Sample chapter (Al-Fatihah)
INSERT INTO chapters (chapter_number, name_arabic, name_transliteration, name_translation, revelation_place, revelation_order, bismillah_pre, verse_count, page_start, page_end) VALUES
    (1, 'الفاتحة', 'Al-Fatihah', 'The Opening', 'makkah', 5, 1, 7, 1, 1);

-- Sample verses from Al-Fatihah
INSERT INTO verses (verse_key, chapter_number, verse_number, text_uthmani, text_simple, juz, hizb, rub_el_hizb, page, manzil, word_count) VALUES
    ('1:1', 1, 1, 'بِسْمِ ٱللَّهِ ٱلرَّحْمَٰنِ ٱلرَّحِيمِ', 'بسم الله الرحمن الرحيم', 1, 1, 1, 1, 1, 4),
    ('1:2', 1, 2, 'ٱلْحَمْدُ لِلَّهِ رَبِّ ٱلْعَٰلَمِينَ', 'الحمد لله رب العالمين', 1, 1, 1, 1, 1, 4),
    ('1:3', 1, 3, 'ٱلرَّحْمَٰنِ ٱلرَّحِيمِ', 'الرحمن الرحيم', 1, 1, 1, 1, 1, 2),
    ('1:4', 1, 4, 'مَٰلِكِ يَوْمِ ٱلدِّينِ', 'مالك يوم الدين', 1, 1, 1, 1, 1, 3),
    ('1:5', 1, 5, 'إِيَّاكَ نَعْبُدُ وَإِيَّاكَ نَسْتَعِينُ', 'اياك نعبد واياك نستعين', 1, 1, 1, 1, 1, 4),
    ('1:6', 1, 6, 'ٱهْدِنَا ٱلصِّرَٰطَ ٱلْمُسْتَقِيمَ', 'اهدنا الصراط المستقيم', 1, 1, 1, 1, 1, 3),
    ('1:7', 1, 7, 'صِرَٰطَ ٱلَّذِينَ أَنْعَمْتَ عَلَيْهِمْ غَيْرِ ٱلْمَغْضُوبِ عَلَيْهِمْ وَلَا ٱلضَّآلِّينَ', 'صراط الذين انعمت عليهم غير المغضوب عليهم ولا الضالين', 1, 1, 1, 1, 1, 10);

-- Sample words from first verse
INSERT INTO words (verse_key, position, text_uthmani, text_simple, transliteration) VALUES
    ('1:1', 1, 'بِسْمِ', 'بسم', 'bismi'),
    ('1:1', 2, 'ٱللَّهِ', 'الله', 'Allāhi'),
    ('1:1', 3, 'ٱلرَّحْمَٰنِ', 'الرحمن', 'al-Raḥmāni'),
    ('1:1', 4, 'ٱلرَّحِيمِ', 'الرحيم', 'al-Raḥīmi');

-- Sample translations for first verse (using translator_id from above)
INSERT INTO verse_translations (verse_key, translator_id, translation) VALUES
    ('1:1', 1, 'In the name of Allah, the Entirely Merciful, the Especially Merciful.'),
    ('1:1', 2, 'In the name of God, Most Gracious, Most Merciful.'),
    ('1:1', 3, 'In the name of Allah, the Beneficent, the Merciful.'),
    ('1:1', 4, 'In the Name of Allah—the Most Compassionate, Most Merciful.'),
    ('1:1', 5, 'In the Name of Allah, the Most Gracious, the Most Merciful.');

-- Sample word translations for first verse, first translator
INSERT INTO word_translations (word_id, translator_id, translation) VALUES
    (1, 1, 'In the name'),
    (2, 1, 'of Allah'),
    (3, 1, 'the Entirely Merciful'),
    (4, 1, 'the Especially Merciful');

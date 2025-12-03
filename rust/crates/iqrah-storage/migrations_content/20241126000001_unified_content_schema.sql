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

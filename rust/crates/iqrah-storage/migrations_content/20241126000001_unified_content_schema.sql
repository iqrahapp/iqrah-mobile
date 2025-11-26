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
    page INTEGER NOT NULL,
    created_at INTEGER NOT NULL DEFAULT (unixepoch()),
    UNIQUE (chapter_number, verse_number),
    FOREIGN KEY (chapter_number) REFERENCES chapters(chapter_number)
) STRICT;

-- ============================================================================
-- POPULATE NODES TABLE (from verses)
-- This must be done after verses are defined but before other tables reference nodes.
-- ============================================================================

-- Sample Chapter and Verses (for testing)
INSERT INTO chapters (chapter_number, name_arabic, name_transliteration, name_translation, revelation_place, verse_count) VALUES
    (1, 'الفاتحة', 'Al-Fatihah', 'The Opening', 'makkah', 7);

INSERT INTO verses (verse_key, chapter_number, verse_number, text_uthmani, juz, page) VALUES
    ('1:1', 1, 1, 'بِسْمِ ٱللَّهِ ٱلرَّحْمَٰنِ ٱلرَّحِيمِ', 1, 1),
    ('1:2', 1, 2, 'ٱلْحَمْدُ لِلَّهِ رَبِّ ٱلْعَٰلَمِينَ', 1, 1),
    ('1:3', 1, 3, 'ٱلرَّحْمَٰنِ ٱلرَّحِيمِ', 1, 1),
    ('1:4', 1, 4, 'مَٰلِكِ يَوْمِ ٱلدِّينِ', 1, 1),
    ('1:5', 1, 5, 'إِيَّاكَ نَعْبُدُ وَإِيَّاكَ نَسْتَعِينُ', 1, 1),
    ('1:6', 1, 6, 'ٱهْدِنَا ٱلصِّرَٰطَ ٱلْمُسْتَقِيمَ', 1, 1),
    ('1:7', 1, 7, 'صِرَٰطَ ٱلَّذِينَ أَنْعَمْتَ عَلَيْهِمْ غَيْرِ ٱلْمَغْضُوبِ عَلَيْهِمْ وَلَا ٱلضَّآلِّينَ', 1, 1);

-- Populate nodes table from the sample verses
INSERT OR IGNORE INTO nodes (ukey, node_type)
SELECT 'VERSE:' || verse_key, 1 FROM verses; -- NodeType::Verse = 1

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

-- Use SELECT to get integer IDs for test data insertion
INSERT OR IGNORE INTO node_goals (goal_id, node_id, priority)
SELECT 'memorization:chapters-1-3', id, 1001000 FROM nodes WHERE ukey = 'VERSE:1:1';

INSERT OR IGNORE INTO node_metadata (node_id, key, value)
SELECT id, 'foundational_score', 0.9 FROM nodes WHERE ukey = 'VERSE:1:1'
UNION ALL
SELECT id, 'influence_score', 0.8 FROM nodes WHERE ukey = 'VERSE:1:1';

INSERT OR IGNORE INTO edges (source_id, target_id, edge_type, distribution_type)
SELECT n1.id, n2.id, 0, 0
FROM nodes n1, nodes n2
WHERE n1.ukey = 'VERSE:1:1' AND n2.ukey = 'VERSE:1:2';

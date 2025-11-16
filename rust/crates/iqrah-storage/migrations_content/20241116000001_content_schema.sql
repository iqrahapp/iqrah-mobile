-- Content Database Schema
-- This database is READ-ONLY at runtime and ships with the app
-- Contains the Qur'anic knowledge graph

-- Core knowledge graph nodes
CREATE TABLE IF NOT EXISTS nodes (
    id TEXT PRIMARY KEY,
    node_type TEXT NOT NULL CHECK (node_type IN ('root', 'lemma', 'word', 'word_instance', 'verse', 'chapter', 'knowledge')),
    created_at INTEGER NOT NULL
) STRICT;

-- Edges: Relationships for energy propagation
CREATE TABLE IF NOT EXISTS edges (
    source_id TEXT NOT NULL,
    target_id TEXT NOT NULL,
    edge_type INTEGER NOT NULL CHECK (edge_type IN (0, 1)), -- 0:Dependency, 1:Knowledge
    distribution_type INTEGER NOT NULL CHECK (distribution_type IN (0, 1, 2)), -- 0:Const, 1:Normal, 2:Beta
    param1 REAL NOT NULL DEFAULT 0.0,
    param2 REAL NOT NULL DEFAULT 0.0,
    PRIMARY KEY (source_id, target_id),
    FOREIGN KEY (source_id) REFERENCES nodes(id) ON DELETE CASCADE,
    FOREIGN KEY (target_id) REFERENCES nodes(id) ON DELETE CASCADE
) STRICT, WITHOUT ROWID;

-- Qur'anic Text (Arabic)
-- Replaces node_metadata for "arabic" key
CREATE TABLE IF NOT EXISTS quran_text (
    node_id TEXT PRIMARY KEY,
    arabic TEXT NOT NULL,
    FOREIGN KEY (node_id) REFERENCES nodes(id) ON DELETE CASCADE
) STRICT, WITHOUT ROWID;

-- Translations (Multi-language support)
-- Replaces node_metadata for "translation" key
-- Language code defaults to 'en' for existing data
CREATE TABLE IF NOT EXISTS translations (
    node_id TEXT NOT NULL,
    language_code TEXT NOT NULL DEFAULT 'en',
    translation TEXT NOT NULL,
    PRIMARY KEY (node_id, language_code),
    FOREIGN KEY (node_id) REFERENCES nodes(id) ON DELETE CASCADE
) STRICT, WITHOUT ROWID;

-- Indexes for performance
CREATE INDEX IF NOT EXISTS idx_nodes_type ON nodes(node_type);
CREATE INDEX IF NOT EXISTS idx_edges_source ON edges(source_id);
CREATE INDEX IF NOT EXISTS idx_edges_target ON edges(target_id);
CREATE INDEX IF NOT EXISTS idx_translations_lang ON translations(language_code);

-- Note: We intentionally do NOT create node_metadata table here
-- All metadata is now in dedicated structured tables

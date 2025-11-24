-- Knowledge Graph Database Schema
-- This database is READ-WRITE and contains the graph structure and user learning state
-- Date: 2025-11-24

-- ============================================================================
-- SCHEMA VERSION TRACKING
-- ============================================================================

CREATE TABLE schema_version (
    version INTEGER PRIMARY KEY
) STRICT;

INSERT INTO schema_version (version) VALUES (1);

-- ============================================================================
-- GRAPH STRUCTURE
-- ============================================================================

-- Knowledge graph edges
-- Nodes are referenced by their ID string (e.g., "VERSE:1:1", "WORD:1:1:1")
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
-- NODE METADATA (Scheduler Scores)
-- ============================================================================

-- Node metadata for storing per-node scores used by scheduler v2.0
-- Keys: 'foundational_score', 'influence_score', 'difficulty_score', 'quran_order'
CREATE TABLE IF NOT EXISTS node_metadata (
    node_id TEXT NOT NULL,
    key TEXT NOT NULL,
    value REAL NOT NULL,
    PRIMARY KEY (node_id, key)
) STRICT, WITHOUT ROWID;

CREATE INDEX IF NOT EXISTS idx_node_metadata_node ON node_metadata(node_id);
CREATE INDEX IF NOT EXISTS idx_node_metadata_key ON node_metadata(key);

-- ============================================================================
-- LEARNING GOALS SYSTEM
-- ============================================================================

-- Goals represent learning objectives (e.g., "Memorize Surah Al-Mulk", "Learn root K-T-B")
CREATE TABLE IF NOT EXISTS goals (
    goal_id TEXT PRIMARY KEY,
    goal_type TEXT NOT NULL CHECK (goal_type IN ('surah', 'root', 'theme', 'custom')),
    goal_group TEXT NOT NULL,  -- For bandit: 'memorization', 'vocab', 'tajweed', etc.
    label TEXT NOT NULL,       -- Human-readable label
    description TEXT,
    created_at INTEGER NOT NULL DEFAULT (unixepoch())
) STRICT;

CREATE INDEX IF NOT EXISTS idx_goals_type ON goals(goal_type);
CREATE INDEX IF NOT EXISTS idx_goals_group ON goals(goal_group);

-- Maps nodes to goals (many-to-many relationship)
CREATE TABLE IF NOT EXISTS node_goals (
    goal_id TEXT NOT NULL,
    node_id TEXT NOT NULL,
    priority INTEGER DEFAULT 0,  -- Optional: priority within goal
    PRIMARY KEY (goal_id, node_id),
    FOREIGN KEY (goal_id) REFERENCES goals(goal_id) ON DELETE CASCADE
) STRICT, WITHOUT ROWID;

CREATE INDEX IF NOT EXISTS idx_node_goals_goal ON node_goals(goal_id);
CREATE INDEX IF NOT EXISTS idx_node_goals_node ON node_goals(node_id);

-- Scheduler v2.0 Schema Extension
-- Adds tables required for the advanced scheduler with prerequisite gates
-- Date: 2025-11-17

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

-- ============================================================================
-- SAMPLE DATA (Optional - for testing)
-- ============================================================================

-- Example goal: Memorize Surah Al-Fatihah
INSERT OR IGNORE INTO goals (goal_id, goal_type, goal_group, label, description) VALUES
    ('memorization:surah-1', 'surah', 'memorization', 'Memorize Surah Al-Fatihah', 'Complete memorization of the opening chapter');

-- Example goal: Learn common roots
INSERT OR IGNORE INTO goals (goal_id, goal_type, goal_group, label, description) VALUES
    ('vocab:common-roots', 'root', 'vocab', 'Learn 100 Common Roots', 'Master the most frequently occurring Arabic roots');

-- Note: node_goals and node_metadata should be populated by the R&D pipeline
-- For testing, you can manually insert sample data:
-- INSERT INTO node_goals (goal_id, node_id) VALUES ('memorization:surah-1', '1:1');
-- INSERT INTO node_metadata (node_id, key, value) VALUES
--     ('1:1', 'foundational_score', 0.5),
--     ('1:1', 'influence_score', 0.3),
--     ('1:1', 'difficulty_score', 0.2),
--     ('1:1', 'quran_order', 1001000);

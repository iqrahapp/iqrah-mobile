-- Scheduler v2 Sample Data
-- This migration adds sample data for testing the scheduler with Surah Al-Fatihah

-- Sample node_metadata for Surah Al-Fatihah verses (1:1 through 1:7)
-- Using simple values for testing
INSERT OR IGNORE INTO node_metadata (node_id, key, value) VALUES
    -- Verse 1:1 - Bismillah
    ('1:1', 'foundational_score', 0.9),
    ('1:1', 'influence_score', 0.8),
    ('1:1', 'difficulty_score', 0.2),
    ('1:1', 'quran_order', 1001001),

    -- Verse 1:2 - Alhamdulillah
    ('1:2', 'foundational_score', 0.8),
    ('1:2', 'influence_score', 0.7),
    ('1:2', 'difficulty_score', 0.3),
    ('1:2', 'quran_order', 1001002),

    -- Verse 1:3 - Ar-Rahman Ar-Raheem
    ('1:3', 'foundational_score', 0.7),
    ('1:3', 'influence_score', 0.6),
    ('1:3', 'difficulty_score', 0.3),
    ('1:3', 'quran_order', 1001003),

    -- Verse 1:4 - Maliki yawm ad-deen
    ('1:4', 'foundational_score', 0.6),
    ('1:4', 'influence_score', 0.5),
    ('1:4', 'difficulty_score', 0.4),
    ('1:4', 'quran_order', 1001004),

    -- Verse 1:5 - Iyyaka na'budu
    ('1:5', 'foundational_score', 0.7),
    ('1:5', 'influence_score', 0.6),
    ('1:5', 'difficulty_score', 0.4),
    ('1:5', 'quran_order', 1001005),

    -- Verse 1:6 - Ihdina as-sirat
    ('1:6', 'foundational_score', 0.6),
    ('1:6', 'influence_score', 0.5),
    ('1:6', 'difficulty_score', 0.5),
    ('1:6', 'quran_order', 1001006),

    -- Verse 1:7 - Sirat alladhina
    ('1:7', 'foundational_score', 0.5),
    ('1:7', 'influence_score', 0.4),
    ('1:7', 'difficulty_score', 0.6),
    ('1:7', 'quran_order', 1001007);

-- Sample goals
INSERT OR IGNORE INTO goals (goal_id, goal_type, goal_group, label, description) VALUES
    ('memorization:surah-1', 'surah', 'memorization', 'Memorize Surah Al-Fatihah', 'Master all 7 verses of Al-Fatihah'),
    ('memorization:surah-1-seq', 'surah', 'memorization', 'Memorize Al-Fatihah (Sequential)', 'Sequential memorization with prerequisites');

-- Sample node-goal mappings for basic goal (no prerequisites)
INSERT OR IGNORE INTO node_goals (goal_id, node_id, priority) VALUES
    ('memorization:surah-1', '1:1', 1),
    ('memorization:surah-1', '1:2', 2),
    ('memorization:surah-1', '1:3', 3),
    ('memorization:surah-1', '1:4', 4),
    ('memorization:surah-1', '1:5', 5),
    ('memorization:surah-1', '1:6', 6),
    ('memorization:surah-1', '1:7', 7);

-- Sample node-goal mappings for sequential goal (same nodes, different goal)
INSERT OR IGNORE INTO node_goals (goal_id, node_id, priority) VALUES
    ('memorization:surah-1-seq', '1:1', 1),
    ('memorization:surah-1-seq', '1:2', 2),
    ('memorization:surah-1-seq', '1:3', 3),
    ('memorization:surah-1-seq', '1:4', 4),
    ('memorization:surah-1-seq', '1:5', 5),
    ('memorization:surah-1-seq', '1:6', 6),
    ('memorization:surah-1-seq', '1:7', 7);

-- Sample prerequisite edges (sequential dependencies)
-- Each verse depends on the previous one being mastered
INSERT OR IGNORE INTO edges (source_id, target_id, edge_type, distribution_type, param1, param2) VALUES
    ('1:1', '1:2', 0, 0, 0.0, 0.0),  -- 1:1 must be mastered before 1:2
    ('1:2', '1:3', 0, 0, 0.0, 0.0),  -- 1:2 must be mastered before 1:3
    ('1:3', '1:4', 0, 0, 0.0, 0.0),  -- 1:3 must be mastered before 1:4
    ('1:4', '1:5', 0, 0, 0.0, 0.0),  -- 1:4 must be mastered before 1:5
    ('1:5', '1:6', 0, 0, 0.0, 0.0),  -- 1:5 must be mastered before 1:6
    ('1:6', '1:7', 0, 0, 0.0, 0.0);  -- 1:6 must be mastered before 1:7

-- Note: edge_type 0 = Dependency (prerequisite)
-- distribution_type 0 = Const (fixed values)

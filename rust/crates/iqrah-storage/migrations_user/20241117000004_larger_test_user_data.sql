-- Sample user data for Al-Baqarah testing
-- Simulates realistic learning progress across 20 verses

INSERT OR IGNORE INTO user_memory_states
(user_id, content_key, stability, difficulty, energy, last_reviewed, due_at, review_count)
SELECT 'test_user', '2:' || verse_num, stability, difficulty, energy, last_reviewed, due_at, review_count
FROM (
    -- First 5 verses: Well mastered
    SELECT 1 AS verse_num, 6.0 AS stability, 3.0 AS difficulty, 0.9 AS energy, 1700000000000 AS last_reviewed, 1700000000000 AS due_at, 15 AS review_count
    UNION ALL SELECT 2, 5.5, 3.2, 0.85, 1700050000000, 1700050000000, 12
    UNION ALL SELECT 3, 5.0, 3.5, 0.8, 1700100000000, 1700100000000, 10
    UNION ALL SELECT 4, 4.5, 3.8, 0.75, 1700150000000, 1700150000000, 9
    UNION ALL SELECT 5, 4.0, 4.0, 0.7, 1700200000000, 1700200000000, 8

    -- Verses 6-10: Moderate mastery
    UNION ALL SELECT 6, 3.5, 4.2, 0.6, 1700250000000, 1700250000000, 6
    UNION ALL SELECT 7, 3.0, 4.5, 0.5, 1700300000000, 1700300000000, 5
    UNION ALL SELECT 8, 2.5, 4.8, 0.4, 1700350000000, 1700350000000, 4
    UNION ALL SELECT 9, 2.0, 5.0, 0.35, 1700400000000, 1700400000000, 3
    UNION ALL SELECT 10, 1.5, 5.2, 0.28, 1700450000000, 1700450000000, 2 -- Below 0.3 threshold

    -- Verses 11-15: Early learning (low energy)
    UNION ALL SELECT 11, 1.0, 5.5, 0.2, 1700500000000, 1700500000000, 1
    UNION ALL SELECT 12, 0.8, 5.7, 0.15, 1700550000000, 1700550000000, 1
    UNION ALL SELECT 13, 0.6, 6.0, 0.1, 1700600000000, 1700600000000, 1
    UNION ALL SELECT 14, 0.5, 6.2, 0.08, 1700650000000, 1700650000000, 1
    UNION ALL SELECT 15, 0.4, 6.5, 0.05, 1700700000000, 1700700000000, 1
);

-- Verses 16-20 have no memory states (energy=0.0, treated as new)
-- This tests scheduler behavior with mix of mastered, learning, and new content

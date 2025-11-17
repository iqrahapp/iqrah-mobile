-- Scheduler v2 Sample User Data
-- This migration adds sample user memory states for testing the scheduler

-- Sample user memory states for "test_user"
-- Simulates a user who has partially memorized Al-Fatihah:
-- - 1:1 (Bismillah): Well mastered (energy=0.8)
-- - 1:2 (Alhamdulillah): Almost mastered (energy=0.6)
-- - 1:3 (Ar-Rahman): Struggling below threshold (energy=0.25)
-- - 1:4 (Maliki): Just starting (energy=0.1)
-- - 1:5, 1:6, 1:7: Not yet started (no memory state)

-- Note: Using millisecond timestamps (current convention)
-- Base timestamp: 1700000000000 (approximately November 2023)

INSERT OR IGNORE INTO user_memory_states
(user_id, content_key, stability, difficulty, energy, last_reviewed, due_at, review_count)
VALUES
    -- 1:1 - Well mastered (energy > 0.7)
    ('test_user', '1:1', 5.0, 3.0, 0.8, 1700000000000, 1700000000000, 10),

    -- 1:2 - Almost mastered (0.4 < energy <= 0.7)
    ('test_user', '1:2', 4.0, 3.5, 0.6, 1700100000000, 1700100000000, 8),

    -- 1:3 - Struggling below mastery threshold (energy < 0.3)
    -- This should block 1:4 from being scheduled due to prerequisite gate
    ('test_user', '1:3', 2.0, 4.0, 0.25, 1700200000000, 1700200000000, 3),

    -- 1:4 - Just starting (0.0 < energy <= 0.2)
    -- Will be blocked by prerequisite gate since 1:3 energy < 0.3
    ('test_user', '1:4', 0.5, 5.0, 0.1, 1700300000000, 1700300000000, 1);

-- Verses 1:5, 1:6, 1:7 have no memory states (energy = 0.0, treated as "new")

-- Expected behavior for scheduler v2:
--
-- For goal "memorization:surah-1" (no prerequisites):
-- - All 7 verses are eligible
-- - 1:1 (energy=0.8) = "Almost mastered" band
-- - 1:2 (energy=0.6) = "Almost there" band
-- - 1:3 (energy=0.25) = "Struggling" band (below threshold but still eligible)
-- - 1:4 (energy=0.1) = "Really struggling" band
-- - 1:5, 1:6, 1:7 (energy=0.0) = "New" band
--
-- For goal "memorization:surah-1-seq" (with prerequisites):
-- - 1:1 eligible (no prerequisites)
-- - 1:2 eligible (1:1 energy=0.8 >= 0.3)
-- - 1:3 eligible (1:2 energy=0.6 >= 0.3)
-- - 1:4 NOT eligible (1:3 energy=0.25 < 0.3) - BLOCKED BY GATE
-- - 1:5 NOT eligible (1:4 is blocked, so 1:5 is also blocked)
-- - 1:6 NOT eligible (chain blocked)
-- - 1:7 NOT eligible (chain blocked)
--
-- So sequential goal should only return 3 candidates: 1:1, 1:2, 1:3

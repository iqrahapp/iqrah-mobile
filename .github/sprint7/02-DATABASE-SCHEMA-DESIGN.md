# Sprint 7: Database Schema Design

**Date:** 2025-10-04
**Purpose:** Define the new two-database architecture

---

## The Two-Database Strategy

### Philosophy
**Separation of Concerns**
- **content.db** = Immutable knowledge graph (shipped with app updates)
- **user.db** = Mutable personal data (user's sacred progress)

### Benefits
1. ✅ Safe content updates (replace content.db without touching user data)
2. ✅ Easy backups (backup only user.db)
3. ✅ Clear ownership (content = read-only, user = read-write)
4. ✅ Performance (smaller user.db, faster queries)
5. ✅ Migration simplicity (only user.db needs migrations)

---

## Database 1: `content.db` (Immutable)

### Purpose
Stores the Qur'anic knowledge graph and ALL metadata.

### Schema

#### Core Graph Tables

```sql
-- Nodes: Entities in the knowledge graph
CREATE TABLE nodes (
    id TEXT PRIMARY KEY,                    -- e.g., "WORD_INSTANCE:2:1:3"
    node_type TEXT NOT NULL,                -- 'word_instance', 'verse', 'lemma', etc.
    created_at INTEGER NOT NULL             -- Unix timestamp (for versioning)
) STRICT;

-- Edges: Relationships between nodes
CREATE TABLE edges (
    source_id TEXT NOT NULL,
    target_id TEXT NOT NULL,
    edge_type INTEGER NOT NULL,             -- 0=Dependency, 1=Knowledge
    distribution_type INTEGER NOT NULL,     -- 0=Const, 1=Normal, 2=Beta
    param1 REAL NOT NULL DEFAULT 0.0,
    param2 REAL NOT NULL DEFAULT 0.0,
    PRIMARY KEY (source_id, target_id),
    FOREIGN KEY (source_id) REFERENCES nodes(id),
    FOREIGN KEY (target_id) REFERENCES nodes(id)
) STRICT, WITHOUT ROWID;
```

#### Metadata Tables (NEW DESIGN)

**Problem with Current Design:**
- Everything is in `node_metadata` key-value table
- Requires multiple JOINs for every query
- Cannot index properly
- Poor query performance

**New Design: Dedicated Tables**

```sql
-- Qur'anic Text (Arabic)
CREATE TABLE quran_text (
    node_id TEXT PRIMARY KEY,
    arabic TEXT NOT NULL,
    transliteration TEXT,                   -- Optional: latin script
    FOREIGN KEY (node_id) REFERENCES nodes(id)
) STRICT, WITHOUT ROWID;

-- Translations (Multi-language support)
CREATE TABLE translations (
    node_id TEXT NOT NULL,
    language_code TEXT NOT NULL,            -- 'en', 'fr', 'ar', etc.
    translation TEXT NOT NULL,
    translator TEXT,                        -- e.g., "Sahih International"
    PRIMARY KEY (node_id, language_code),
    FOREIGN KEY (node_id) REFERENCES nodes(id)
) STRICT, WITHOUT ROWID;

CREATE INDEX idx_translations_lang ON translations(language_code);

-- Audio Resources
CREATE TABLE audio_resources (
    node_id TEXT NOT NULL,
    reciter_id TEXT NOT NULL,               -- e.g., "abdulbasit", "mishary"
    audio_url TEXT NOT NULL,                -- CDN URL or local path
    duration_ms INTEGER,
    PRIMARY KEY (node_id, reciter_id),
    FOREIGN KEY (node_id) REFERENCES nodes(id)
) STRICT, WITHOUT ROWID;

-- Contextual Information
CREATE TABLE node_context (
    node_id TEXT PRIMARY KEY,
    surah_number INTEGER,
    ayah_number INTEGER,
    word_index INTEGER,                     -- Position in ayah (1-based)
    parent_node_id TEXT,                    -- Link to parent (e.g., verse for word)
    FOREIGN KEY (node_id) REFERENCES nodes(id),
    FOREIGN KEY (parent_node_id) REFERENCES nodes(id)
) STRICT, WITHOUT ROWID;

CREATE INDEX idx_context_surah ON node_context(surah_number, ayah_number);
CREATE INDEX idx_context_parent ON node_context(parent_node_id);

-- Linguistic Metadata
CREATE TABLE linguistic_meta (
    node_id TEXT PRIMARY KEY,
    root TEXT,                              -- Arabic root (e.g., "كتب")
    lemma TEXT,                             -- Base form
    part_of_speech TEXT,                    -- 'noun', 'verb', 'particle'
    morphology TEXT,                        -- Detailed morphological info (JSON)
    FOREIGN KEY (node_id) REFERENCES nodes(id)
) STRICT, WITHOUT ROWID;

-- Importance Scores (Pre-computed)
CREATE TABLE importance_scores (
    node_id TEXT PRIMARY KEY,
    influence_score REAL NOT NULL,          -- PageRank-based
    foundational_score REAL NOT NULL,       -- Prerequisite importance
    frequency INTEGER,                      -- Occurrence count
    FOREIGN KEY (node_id) REFERENCES nodes(id)
) STRICT, WITHOUT ROWID;

-- Reciters (Reference Data)
CREATE TABLE reciters (
    reciter_id TEXT PRIMARY KEY,
    name_en TEXT NOT NULL,
    name_ar TEXT,
    style TEXT,                             -- 'murattal', 'mujawwad'
    bitrate INTEGER                         -- Audio quality
) STRICT, WITHOUT ROWID;

-- Surah Metadata
CREATE TABLE surahs (
    surah_number INTEGER PRIMARY KEY,
    name_en TEXT NOT NULL,
    name_ar TEXT NOT NULL,
    revelation_type TEXT,                   -- 'meccan' or 'medinan'
    ayah_count INTEGER NOT NULL,
    chronological_order INTEGER
) STRICT, WITHOUT ROWID;
```

#### Indexes for Performance

```sql
-- Graph traversal
CREATE INDEX idx_edges_source ON edges(source_id);
CREATE INDEX idx_edges_target ON edges(target_id);

-- Surah/Ayah lookup
CREATE INDEX idx_nodes_type ON nodes(node_type);
```

---

## Database 2: `user.db` (Mutable)

### Purpose
Stores all user-specific learning progress and settings.

### Schema

#### Memory & Progress Tables

```sql
-- FSRS Memory States
CREATE TABLE user_memory_states (
    user_id TEXT NOT NULL,
    node_id TEXT NOT NULL,
    stability REAL NOT NULL DEFAULT 0,
    difficulty REAL NOT NULL DEFAULT 0,
    energy REAL NOT NULL DEFAULT 0.0,       -- Custom mastery metric
    last_reviewed INTEGER NOT NULL DEFAULT 0,
    due_at INTEGER NOT NULL DEFAULT 0,
    review_count INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (user_id, node_id)
) STRICT, WITHOUT ROWID;

CREATE INDEX idx_ums_user_due ON user_memory_states(user_id, due_at);
CREATE INDEX idx_ums_user_energy ON user_memory_states(user_id, energy);

-- Review History (Audit Log)
CREATE TABLE review_history (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id TEXT NOT NULL,
    node_id TEXT NOT NULL,
    reviewed_at INTEGER NOT NULL,
    grade INTEGER NOT NULL,                 -- 1=Again, 2=Hard, 3=Good, 4=Easy
    duration_ms INTEGER,                    -- Time spent on exercise
    exercise_type TEXT,                     -- 'recall', 'mcq_ar_to_en', etc.
    previous_energy REAL,
    new_energy REAL
) STRICT;

CREATE INDEX idx_history_user_node ON review_history(user_id, node_id);
CREATE INDEX idx_history_timestamp ON review_history(reviewed_at DESC);
```

#### Propagation Logging

```sql
-- Propagation Events
CREATE TABLE propagation_events (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    source_node_id TEXT NOT NULL,
    event_timestamp INTEGER NOT NULL
) STRICT;

CREATE INDEX idx_prop_events_timestamp ON propagation_events(event_timestamp DESC);

-- Propagation Details
CREATE TABLE propagation_details (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    event_id INTEGER NOT NULL,
    target_node_id TEXT NOT NULL,
    energy_change REAL NOT NULL,
    path TEXT,                              -- Graph path (for debugging)
    reason TEXT,                            -- e.g., "Direct(Good)", "Propagated"
    FOREIGN KEY (event_id) REFERENCES propagation_events(id) ON DELETE CASCADE
) STRICT;

CREATE INDEX idx_prop_details_event ON propagation_details(event_id);
CREATE INDEX idx_prop_details_target ON propagation_details(target_node_id);
```

#### Session & UI State

```sql
-- Active Session (Ephemeral)
CREATE TABLE session_state (
    node_id TEXT NOT NULL PRIMARY KEY,
    session_order INTEGER NOT NULL
) STRICT, WITHOUT ROWID;

-- User Statistics
CREATE TABLE user_stats (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
) STRICT, WITHOUT ROWID;

-- Examples: ('reviews_today', '42'), ('streak_days', '7'), ('last_review_date', '2025-10-04')
```

#### Settings & Preferences

```sql
-- User Settings
CREATE TABLE user_settings (
    user_id TEXT NOT NULL,
    setting_key TEXT NOT NULL,
    setting_value TEXT NOT NULL,
    PRIMARY KEY (user_id, setting_key)
) STRICT, WITHOUT ROWID;

-- Examples:
-- ('default_user', 'language', 'en')
-- ('default_user', 'reciter', 'abdulbasit')
-- ('default_user', 'daily_goal', '20')
```

#### Exercise Difficulty Config (NEW)

```sql
-- Exercise Variants Configuration
CREATE TABLE exercise_variants (
    node_type TEXT NOT NULL,               -- 'word_instance', 'verse', etc.
    knowledge_axis TEXT NOT NULL,          -- 'translation', 'memorization', 'tajweed'
    variant_id TEXT NOT NULL,              -- 'a', 'b', 'c', 'd'
    difficulty_level INTEGER NOT NULL,     -- 1=easy, 2=medium, 3=hard, 4=expert
    optimal_energy REAL NOT NULL,          -- Peak effectiveness (0.0-1.0)
    energy_std_dev REAL NOT NULL,          -- Distribution width
    max_impact REAL NOT NULL,              -- Maximum knowledge yield
    min_impact REAL NOT NULL,              -- Minimum knowledge yield
    config_json TEXT,                      -- Variant-specific config (JSON)
    PRIMARY KEY (node_type, knowledge_axis, variant_id)
) STRICT, WITHOUT ROWID;

-- Seeded on first run with expert-tuned distributions
```

---

## Migration Version Tracking

```sql
-- Migration metadata (user.db only)
-- SQLx handles this automatically, but we'll also use PRAGMA for backup

PRAGMA user_version = 1;  -- Incremented with each migration
```

---

## Query Patterns

### Example: Get Due Items with Metadata

**Old (Single DB):**
```sql
SELECT n.id, n.node_type,
       nm1.value as arabic,
       nm2.value as translation,
       ums.energy, ums.due_at
FROM nodes n
JOIN user_memory_states ums ON n.id = ums.node_id
LEFT JOIN node_metadata nm1 ON n.id = nm1.node_id AND nm1.key = 'arabic'
LEFT JOIN node_metadata nm2 ON n.id = nm2.node_id AND nm2.key = 'translation'
WHERE ums.user_id = ? AND ums.due_at <= ?
-- 4 JOINs!
```

**New (Two DBs):**
```sql
-- Step 1: Query user.db for due node IDs
SELECT node_id, energy, due_at
FROM user_memory_states
WHERE user_id = ? AND due_at <= ?
ORDER BY due_at ASC
LIMIT 20;

-- Step 2: Query content.db for metadata (single query)
SELECT qt.node_id, qt.arabic, t.translation
FROM quran_text qt
JOIN translations t ON qt.node_id = t.node_id
WHERE qt.node_id IN (?, ?, ..., ?)  -- Batch fetch
  AND t.language_code = 'en';

-- 1 JOIN instead of 4!
```

### Example: Session Priority Score

**Old (SQL-embedded logic):**
```sql
ORDER BY (
    1.0 * MAX(0, (?2 - due_at) / (24*60*60*1000)) +
    2.0 * MAX(0, 1.0 - energy) +
    1.5 * COALESCE((SELECT value FROM node_metadata ...), 0)
) DESC
```

**New (Application logic):**
```rust
// Fetch raw data
let user_states = user_repo.get_due_states(user_id).await?;
let importance = content_repo.get_importance_scores(&node_ids).await?;

// Calculate in Rust (testable!)
let scored_items: Vec<ScoredItem> = user_states
    .iter()
    .map(|state| {
        let days_overdue = (now - state.due_at).as_days_f64();
        let mastery_gap = 1.0 - state.energy;
        let importance_score = importance.get(&state.node_id)
            .map(|s| if high_yield { s.influence } else { s.foundational })
            .unwrap_or(0.0);

        let priority = weights.w_due * days_overdue
                     + weights.w_need * mastery_gap
                     + weights.w_yield * importance_score;

        ScoredItem { node_id: state.node_id.clone(), priority }
    })
    .sorted_by_priority()
    .collect();
```

---

## Data Volume Estimates

### content.db Size
- **Nodes:** ~50,000 (complete Qur'an knowledge graph)
- **Edges:** ~200,000
- **Quran Text:** ~50,000 rows (arabic + transliteration)
- **Translations:** ~50,000 × 5 languages = 250,000 rows
- **Audio Resources:** ~50,000 × 3 reciters = 150,000 rows
- **Estimated Size:** ~200-300 MB (compressed)

### user.db Size (Active User)
- **Memory States:** ~5,000 rows (only studied nodes, not all 50k!)
- **Review History:** ~10,000 rows (1 year of daily reviews)
- **Propagation Log:** ~50,000 rows (can be pruned)
- **Estimated Size:** ~10-20 MB

---

## Lazy User Data Generation

**Old Approach:**
```sql
-- Pre-generate 50,000+ rows on first run
INSERT INTO user_memory_states (user_id, node_id, ...)
SELECT 'user', id, 0, 0, 0, 0, 0, 0 FROM nodes;
```

**New Approach: On-Demand**
```rust
async fn get_or_create_memory_state(node_id: &str) -> Result<MemoryState> {
    match user_repo.get_memory_state(node_id).await {
        Ok(state) => Ok(state),
        Err(NotFound) => {
            // Create on first access
            let state = MemoryState::new_for_node(node_id);
            user_repo.create_memory_state(state).await?;
            Ok(state)
        }
    }
}
```

**Benefits:**
- 90% reduction in user.db size for new users
- Faster initial setup
- Only store what's actually used

---

## Foreign Key Handling

**Important:** `content.db` and `user.db` are separate files!

**Problem:** Cannot use SQL foreign keys across databases

**Solution:** Application-level referential integrity
```rust
// Before inserting user_memory_state
if !content_repo.node_exists(node_id).await? {
    return Err(InvalidNodeId(node_id));
}

user_repo.create_memory_state(state).await?;
```

---

## Next Steps

See:
- `03-ARCHITECTURE-BLUEPRINT.md` - How to structure the Rust code
- `04-MIGRATION-STRATEGY.md` - How to migrate existing users
- `05-TESTING-STRATEGY.md` - How to test the new design

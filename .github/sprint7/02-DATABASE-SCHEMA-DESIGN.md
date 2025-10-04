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

-- AI/Expert-Generated Questions (NEW - Sprint 7+)
CREATE TABLE questions (
    question_id TEXT PRIMARY KEY,           -- UUID
    question_text TEXT NOT NULL,
    question_type TEXT NOT NULL,            -- 'mcq', 'type_answer'
    difficulty INTEGER NOT NULL,            -- 1=beginner, 2=intermediate, 3=advanced, 4=expert
    verification_status TEXT NOT NULL,      -- 'unverified', 'expert_verified', 'approved', 'flagged'
    aqeedah_school TEXT,                    -- 'ashari', 'maturidi', 'athari', or NULL for universal
    tafsir_source TEXT,                     -- e.g., 'ibn_kathir', 'tabari', or NULL
    created_at INTEGER NOT NULL,
    verified_at INTEGER,
    verified_by TEXT,                       -- Expert ID
    metadata_json TEXT                      -- Question-specific config (MCQ options, etc.)
) STRICT;

CREATE INDEX idx_questions_difficulty ON questions(difficulty);
CREATE INDEX idx_questions_status ON questions(verification_status);
CREATE INDEX idx_questions_school ON questions(aqeedah_school);

-- Question-to-Node Linkages (Many-to-Many)
-- A question can link to multiple nodes (cross-surah, cross-ayah)
-- A node can have multiple questions at different difficulties
CREATE TABLE question_node_links (
    question_id TEXT NOT NULL,
    node_id TEXT NOT NULL,
    link_strength REAL NOT NULL DEFAULT 1.0, -- How strongly this node relates (0.0-1.0)
    PRIMARY KEY (question_id, node_id),
    FOREIGN KEY (question_id) REFERENCES questions(question_id) ON DELETE CASCADE,
    FOREIGN KEY (node_id) REFERENCES nodes(id) ON DELETE CASCADE
) STRICT, WITHOUT ROWID;

CREATE INDEX idx_qnl_node ON question_node_links(node_id);
CREATE INDEX idx_qnl_question ON question_node_links(question_id);

-- Audio Reference Data (NEW - Sprint 8+)
CREATE TABLE audio_pitch_contours (
    node_id TEXT NOT NULL,
    reciter_id TEXT NOT NULL,
    f0_contour BLOB NOT NULL,               -- CBOR-encoded pitch contour
    sample_rate INTEGER NOT NULL,
    duration_ms INTEGER NOT NULL,
    PRIMARY KEY (node_id, reciter_id),
    FOREIGN KEY (node_id) REFERENCES nodes(id),
    FOREIGN KEY (reciter_id) REFERENCES reciters(reciter_id)
) STRICT, WITHOUT ROWID;
```

#### Indexes for Performance

```sql
-- Graph traversal
CREATE INDEX idx_edges_source ON edges(source_id);
CREATE INDEX idx_edges_target ON edges(target_id);
CREATE INDEX idx_edges_type ON edges(edge_type);

-- Surah/Ayah lookup
CREATE INDEX idx_nodes_type ON nodes(node_type);

-- Hadith support (future - Sprint 9+)
-- Hadiths will be added as nodes with node_type='hadith'
-- Separate edge_types will distinguish Quran-Hadith relationships
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
-- ('default_user', 'aqeedah_school', 'ashari')
-- ('default_user', 'tafsir_filter', 'ibn_kathir,tabari')
```

#### Question Progress Tracking (NEW - Sprint 7+)

```sql
-- User's mastery of expert questions
CREATE TABLE question_memory_states (
    user_id TEXT NOT NULL,
    question_id TEXT NOT NULL,              -- References content.db:questions
    stability REAL NOT NULL DEFAULT 0,
    difficulty REAL NOT NULL DEFAULT 0,
    mastery REAL NOT NULL DEFAULT 0.0,      -- 0.0-1.0, contributes to node energy
    last_reviewed INTEGER NOT NULL DEFAULT 0,
    due_at INTEGER NOT NULL DEFAULT 0,
    review_count INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (user_id, question_id)
) STRICT, WITHOUT ROWID;

CREATE INDEX idx_qms_user_due ON question_memory_states(user_id, due_at);

-- Question Review History
CREATE TABLE question_review_history (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id TEXT NOT NULL,
    question_id TEXT NOT NULL,
    reviewed_at INTEGER NOT NULL,
    grade INTEGER NOT NULL,
    duration_ms INTEGER,
    user_answer TEXT,                       -- For type_answer questions
    previous_mastery REAL,
    new_mastery REAL
) STRICT;

CREATE INDEX idx_qrh_user_question ON question_review_history(user_id, question_id);

-- User Question Flags (reporting issues)
CREATE TABLE question_flags (
    user_id TEXT NOT NULL,
    question_id TEXT NOT NULL,
    flag_type TEXT NOT NULL,                -- 'incorrect', 'unclear', 'offensive', 'duplicate'
    flag_reason TEXT,
    flagged_at INTEGER NOT NULL,
    PRIMARY KEY (user_id, question_id, flag_type)
) STRICT, WITHOUT ROWID;
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

## Critical Logic: Energy Recalculation for Questions

### Problem Statement
When AI-generated questions are added/updated for a KG node, the node's energy MUST be recalculated to reflect the new mastery requirements.

### Energy Calculation Formula

```rust
/// Calculate node energy considering both automatic exercises AND expert questions
fn calculate_node_energy(
    node_id: &str,
    auto_exercise_mastery: f32,        // From existing FSRS reviews
    question_masteries: Vec<f32>,       // From question_memory_states
) -> f32 {
    if question_masteries.is_empty() {
        // No questions linked: use only auto-exercise mastery
        return auto_exercise_mastery;
    }

    // Full mastery requires BOTH auto-exercises AND all questions
    // Formula: E = auto_mastery × (average of all question masteries)
    let avg_question_mastery = question_masteries.iter().sum::<f32>()
                              / question_masteries.len() as f32;

    auto_exercise_mastery * avg_question_mastery
}
```

### Recalculation Triggers

**Scenario 1: New Question Added**
```sql
-- When content.db gets updated with new questions
-- Trigger: On app startup, check content.db version
-- Action: Recalculate energy for all affected nodes
SELECT DISTINCT node_id
FROM question_node_links
WHERE question_id IN (SELECT question_id FROM questions WHERE created_at > ?);
```

**Scenario 2: User Answers Question**
```rust
// After processing question review
async fn after_question_review(user_id: &str, question_id: &str) -> Result<()> {
    // 1. Update question_memory_states with new mastery
    update_question_mastery(user_id, question_id).await?;

    // 2. Get all nodes linked to this question
    let linked_nodes = content_repo.get_question_node_links(question_id).await?;

    // 3. Recalculate energy for each node
    for node_id in linked_nodes {
        recalculate_and_propagate_energy(user_id, &node_id).await?;
    }

    Ok(())
}
```

**Scenario 3: Content Update (New App Version)**
```rust
// On app startup
async fn check_content_version() -> Result<()> {
    let user_content_version = user_settings.get("content_db_version")?;
    let actual_content_version = content_repo.get_version()?;

    if user_content_version != actual_content_version {
        // Content.db was updated - recalculate affected energies
        recalculate_all_nodes_with_questions().await?;
        user_settings.set("content_db_version", actual_content_version)?;
    }

    Ok(())
}
```

### Multi-Node Question Handling

**Example: Question links to 3 ayahs from different surahs**
```
question_id: "Q_ALLAH_NAMES_001"
links: [
  ("AYAH:2:255", strength=1.0),  // Ayat al-Kursi
  ("AYAH:59:24", strength=0.8),  // Al-Hashr
  ("AYAH:112:1", strength=0.6),  // Al-Ikhlas
]
```

**Energy Impact:**
- Mastering this question increases energy for ALL 3 ayahs
- Impact weighted by `link_strength`
- Each ayah's energy calculation includes this question proportionally

### Data Volume Impact

**Questions per Node (Estimated):**
- High-level nodes (Surah): 5-20 questions each
- Mid-level nodes (Ayah): 2-10 questions each
- Low-level nodes (Word): 0-2 questions each

**Total Estimate:**
- 114 surahs × 10 avg = 1,140 surah questions
- 6,236 ayahs × 5 avg = 31,180 ayah questions
- Selected words × 1 avg = ~5,000 word questions
- **Total: ~40,000 questions** (after 1 year of LLM generation + verification)

**Storage Impact:**
- content.db: +15 MB (questions + metadata)
- user.db: +2-5 MB (question_memory_states for active learner)

---

## Future Extensions

### Hadith Integration (Sprint 9+)

**Graph Design:**
```sql
-- Hadiths as first-class nodes
INSERT INTO nodes (id, node_type)
VALUES ('HADITH:bukhari:1:1', 'hadith');

-- Separate edge types for Quran-Hadith relationships
INSERT INTO edges (source_id, target_id, edge_type, ...)
VALUES ('AYAH:2:183', 'HADITH:bukhari:1:1', 2, ...);  -- edge_type=2 for "Hadith Explains"

-- Additional metadata tables
CREATE TABLE hadith_metadata (
    node_id TEXT PRIMARY KEY,
    collection TEXT NOT NULL,      -- 'bukhari', 'muslim', 'tirmidhi'
    book_number INTEGER,
    hadith_number INTEGER,
    narrator_chain TEXT,           -- Isnad
    authenticity TEXT,              -- 'sahih', 'hasan', 'daif'
    arabic_text TEXT NOT NULL,
    FOREIGN KEY (node_id) REFERENCES nodes(id)
) STRICT, WITHOUT ROWID;
```

**Migration Path:**
1. Add new `edge_type` enum values
2. Create hadith-specific metadata tables in content.db
3. Update graph algorithms to handle hadith nodes
4. No changes needed to user.db (reuses user_memory_states)

### Audio Features (Sprint 8+)

**Already prepared in schema:**
- `audio_pitch_contours` table ready for F0 data
- CBOR format for efficient storage
- Multi-reciter support

**Integration:**
```rust
// Load reference contour
let reference = content_repo.get_pitch_contour(node_id, reciter_id).await?;

// Analyze user recording
let user_contour = extract_pitch(user_audio)?;
let similarity = dtw_similarity(&reference, &user_contour)?;

// Store as review
let grade = score_to_fsrs_grade(similarity);
user_repo.record_review(user_id, node_id, grade, "recitation").await?;
```

---

## Next Steps

See:
- `03-ARCHITECTURE-BLUEPRINT.md` - How to structure the Rust code
- `04-MIGRATION-STRATEGY.md` - How to migrate existing users
- `05-TESTING-STRATEGY.md` - How to test the new design
- `07-FUTURE-EXTENSIBILITY.md` - Detailed extensibility strategy (NEW)

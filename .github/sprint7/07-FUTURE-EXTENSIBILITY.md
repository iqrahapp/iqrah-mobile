# Sprint 7: Future Extensibility Strategy

**Date:** 2025-10-04
**Purpose:** Ensure Sprint 7 architecture supports future features without major refactoring

---

## Overview

Sprint 7's architecture must prepare for three major future extensions:

1. **AI-Generated Questions** (Sprint 7+): Expert questions linked to KG nodes
2. **Hadith Integration** (Sprint 9+, ~6-18 months): Hadith as KG nodes
3. **Audio Analysis** (Sprint 8+, ~2-3 months): Qari imitation with pitch analysis

This document ensures our design decisions today don't block tomorrow's features.

---

## 1. AI-Generated Questions System

### Timeline
- **Sprint 7**: Database schema ready, placeholder services
- **Sprint 8**: Basic question display and review
- **Future (~1 year)**: LLM-based question generation platform

### Architecture Ready State

#### Database Schema ✅
```sql
-- content.db: Already designed in 02-DATABASE-SCHEMA-DESIGN.md
CREATE TABLE questions (...);
CREATE TABLE question_node_links (...);

-- user.db: Already designed
CREATE TABLE question_memory_states (...);
CREATE TABLE question_review_history (...);
CREATE TABLE question_flags (...);
```

#### Repository Interface ✅
```rust
// Already defined in 03-ARCHITECTURE-BLUEPRINT.md
pub trait QuestionRepository: Send + Sync {
    async fn get_questions_for_node(&self, node_id: &NodeId) -> Result<Vec<Question>>;
    async fn get_question_memory_state(...) -> Result<Option<QuestionMemoryState>>;
    async fn flag_question(...) -> Result<()>;
    // ... full API ready
}
```

#### Energy Recalculation Logic ✅
```rust
// QuestionService::recalculate_node_energy_with_questions()
// Formula: E_node = E_auto_exercises × avg(E_questions)
// Triggers: After question review, after content.db update
```

### What Sprint 7 MUST Include

**1. Schema Creation**
- Create all question-related tables (even if empty)
- Add indexes for future query patterns
- Include `content_db_version` tracking in user_settings

**2. Repository Stub**
```rust
// iqrah-storage/src/content/questions.rs
pub struct SqlxQuestionRepository {
    content_pool: SqlitePool,
    user_pool: SqlitePool,
}

// Implement QuestionRepository with minimal logic
impl QuestionRepository for SqlxQuestionRepository {
    async fn get_questions_for_node(&self, node_id: &NodeId) -> Result<Vec<Question>> {
        // Query content.db (returns empty for now)
        sqlx::query_as!(...)
            .fetch_all(&self.content_pool)
            .await
    }
}
```

**3. Service Integration Points**
```rust
// iqrah-core/src/services/learning_service.rs
pub struct LearningService {
    content_repo: Arc<dyn ContentRepository>,
    user_repo: Arc<dyn UserRepository>,
    question_repo: Arc<dyn QuestionRepository>,  // NEW
    scheduler: Arc<dyn Scheduler>,
}

// After reviewing a node, check for linked questions
async fn after_node_review(&self, user_id: &str, node_id: &NodeId) -> Result<()> {
    let questions = self.question_repo.get_questions_for_node(node_id).await?;

    if !questions.is_empty() {
        // Recalculate energy considering questions
        self.recalculate_energy_with_questions(user_id, node_id).await?;
    }

    Ok(())
}
```

**4. Migration Support**
```rust
// On app startup
async fn check_content_version(&self, user_id: &str) -> Result<()> {
    let stored_version = self.user_repo.get_setting(user_id, "content_db_version").await?;
    let current_version = self.content_repo.get_version().await?;

    if stored_version != current_version {
        info!("Content database updated: {} -> {}", stored_version, current_version);

        // Recalculate energies for nodes with new questions
        self.question_service.sync_content_version(user_id).await?;

        self.user_repo.set_setting(user_id, "content_db_version", &current_version).await?;
    }

    Ok(())
}
```

### What Can Wait Until Later

**Sprint 8+ Features:**
- Question UI components (display, review screens)
- Question filtering by aqeedah/tafsir
- Question flagging workflow
- Offline AI verification (type-answer questions)

**LLM Platform (~1 year):**
- Question generation pipeline
- Expert verification platform
- Question approval workflow
- Cross-checking system

### Risk Mitigation

**Risk 1: Questions break energy propagation**
- **Mitigation**: Add property tests that verify energy calculations with/without questions
```rust
#[proptest]
fn energy_with_questions_never_exceeds_one(
    auto_mastery: f32,
    question_masteries: Vec<f32>,
) {
    prop_assume!(auto_mastery >= 0.0 && auto_mastery <= 1.0);
    prop_assume!(question_masteries.iter().all(|&m| m >= 0.0 && m <= 1.0));

    let energy = calculate_node_energy(auto_mastery, question_masteries);
    prop_assert!(energy <= 1.0);
}
```

**Risk 2: Multi-node questions cause infinite propagation**
- **Mitigation**: Propagation algorithm already handles cycles with visited set
- Add integration test with cross-surah question

**Risk 3: Content updates corrupt user data**
- **Mitigation**: Version tracking + rollback mechanism
```rust
async fn rollback_content_update(&self, user_id: &str, to_version: &str) -> Result<()> {
    // Restore previous content_db_version
    // Invalidate question memory states added after rollback point
    Ok(())
}
```

---

## 2. Hadith Integration

### Timeline
- **Sprint 7**: Edge type extensibility
- **Sprint 9+ (~6-18 months)**: Full hadith implementation

### Architecture Ready State

#### Knowledge Graph Design ✅
```sql
-- Hadiths as first-class nodes (already supported by flexible node_type)
INSERT INTO nodes (id, node_type, created_at)
VALUES ('HADITH:bukhari:1:1', 'hadith', unixepoch());

-- Separate edge types (already have edge_type column)
-- 0 = Dependency (Quran internal)
-- 1 = Knowledge (Quran internal)
-- 2 = Hadith Explains (Quran → Hadith)
-- 3 = Hadith References (Hadith → Quran)
INSERT INTO edges (source_id, target_id, edge_type, ...)
VALUES ('AYAH:2:183', 'HADITH:bukhari:1:1', 2, ...);
```

#### Metadata Tables (Future)
```sql
-- content.db additions (Sprint 9+)
CREATE TABLE hadith_metadata (
    node_id TEXT PRIMARY KEY,
    collection TEXT NOT NULL,        -- 'bukhari', 'muslim', 'tirmidhi'
    book_number INTEGER,
    hadith_number INTEGER,
    narrator_chain TEXT,             -- Isnad
    authenticity TEXT,                -- 'sahih', 'hasan', 'daif'
    arabic_text TEXT NOT NULL,
    FOREIGN KEY (node_id) REFERENCES nodes(id)
) STRICT, WITHOUT ROWID;

CREATE TABLE hadith_translations (
    node_id TEXT NOT NULL,
    language_code TEXT NOT NULL,
    translation TEXT NOT NULL,
    translator TEXT,
    PRIMARY KEY (node_id, language_code),
    FOREIGN KEY (node_id) REFERENCES nodes(id)
) STRICT, WITHOUT ROWID;
```

### What Sprint 7 MUST Include

**1. Edge Type Enum**
```rust
// iqrah-core/src/domain/models.rs
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EdgeType {
    Dependency = 0,
    Knowledge = 1,
    HadithExplains = 2,      // NEW: Reserved for future
    HadithReferences = 3,     // NEW: Reserved for future
}

impl EdgeType {
    pub fn is_quran_internal(&self) -> bool {
        matches!(self, EdgeType::Dependency | EdgeType::Knowledge)
    }

    pub fn is_hadith_related(&self) -> bool {
        matches!(self, EdgeType::HadithExplains | EdgeType::HadithReferences)
    }
}
```

**2. Node Type Enum**
```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NodeType {
    WordInstance,
    Verse,
    Surah,
    Lemma,
    Root,
    Hadith,              // NEW: Reserved for future
}

impl NodeType {
    pub fn is_quran_node(&self) -> bool {
        !matches!(self, NodeType::Hadith)
    }
}
```

**3. Propagation Algorithm Awareness**
```rust
// iqrah-core/src/domain/propagation.rs
pub fn propagate_energy(
    graph: &Graph,
    source_node: &NodeId,
    energy_delta: f64,
    config: PropagationConfig,
) -> HashMap<NodeId, f64> {
    let mut updates = HashMap::new();
    let mut visited = HashSet::new();

    propagate_recursive(
        graph,
        source_node,
        energy_delta,
        &mut updates,
        &mut visited,
        config,
    );

    updates
}

fn propagate_recursive(
    graph: &Graph,
    node: &NodeId,
    energy: f64,
    updates: &mut HashMap<NodeId, f64>,
    visited: &mut HashSet<NodeId>,
    config: PropagationConfig,
) {
    if visited.contains(node) || energy.abs() < config.min_threshold {
        return;
    }

    visited.insert(node.clone());

    // Get outgoing edges, filter by edge type if needed
    let edges = graph.get_edges_from(node);

    for edge in edges {
        // Skip hadith edges if not enabled
        if edge.edge_type.is_hadith_related() && !config.include_hadith {
            continue;
        }

        let propagated = energy * edge.strength;
        *updates.entry(edge.target_id.clone()).or_insert(0.0) += propagated;

        propagate_recursive(graph, &edge.target_id, propagated, updates, visited, config);
    }
}
```

**4. Configuration Flag**
```rust
pub struct PropagationConfig {
    pub min_threshold: f64,
    pub include_hadith: bool,       // NEW: Default false for now
}
```

### What Can Wait Until Later

**Sprint 9+ Features:**
- Hadith metadata ingestion
- Hadith translation support
- Hadith-specific UI components
- Hadith filtering (by collection, authenticity)

### Risk Mitigation

**Risk 1: Hadith propagation affects Quran mastery calculation**
- **Mitigation**: Separate edge types allow isolated propagation
- User setting: "Enable Hadith Mode" (default off)

**Risk 2: Graph becomes too large**
- **Mitigation**: Lazy loading (only load Quran nodes by default)
```rust
async fn load_graph(&self, include_hadith: bool) -> Result<Graph> {
    let nodes = if include_hadith {
        self.repo.get_all_nodes().await?
    } else {
        self.repo.get_quran_nodes_only().await?
    };

    Ok(Graph::from_nodes(nodes))
}
```

---

## 3. Audio Analysis Features

### Timeline
- **R&D Phase (parallel with Sprint 7)**: Python proof-of-concept
- **Sprint 8 (~2-3 months)**: Production integration

### Architecture Ready State

#### Database Schema ✅
```sql
-- content.db: Already designed
CREATE TABLE audio_pitch_contours (
    node_id TEXT NOT NULL,
    reciter_id TEXT NOT NULL,
    f0_contour BLOB NOT NULL,           -- CBOR-encoded F0 time-series
    sample_rate INTEGER NOT NULL,
    duration_ms INTEGER NOT NULL,
    PRIMARY KEY (node_id, reciter_id),
    FOREIGN KEY (node_id) REFERENCES nodes(id),
    FOREIGN KEY (reciter_id) REFERENCES reciters(reciter_id)
) STRICT, WITHOUT ROWID;
```

#### Repository Interface
```rust
// iqrah-core/src/ports/content_repository.rs
#[async_trait]
pub trait ContentRepository: Send + Sync {
    // ... existing methods ...

    async fn get_pitch_contour(
        &self,
        node_id: &NodeId,
        reciter_id: &str,
    ) -> Result<PitchContour>;
}

pub struct PitchContour {
    pub f0_values: Vec<f32>,        // Fundamental frequency samples
    pub sample_rate: u32,
    pub duration_ms: u32,
}
```

#### Audio Analysis Service (Future)
```rust
// iqrah-core/src/services/audio_service.rs
pub struct AudioService {
    content_repo: Arc<dyn ContentRepository>,
    user_repo: Arc<dyn UserRepository>,
}

impl AudioService {
    pub async fn analyze_recitation(
        &self,
        user_id: &str,
        node_id: &NodeId,
        reciter_id: &str,
        user_audio: Vec<f32>,
    ) -> Result<RecitationResult> {
        // 1. Load reference contour from content.db
        let reference = self.content_repo.get_pitch_contour(node_id, reciter_id).await?;

        // 2. Extract pitch from user audio (using Rust DSP crate)
        let user_contour = extract_pitch(&user_audio)?;

        // 3. Perform DTW alignment
        let (similarity, aligned_path) = dtw_similarity(&reference.f0_values, &user_contour)?;

        // 4. Convert similarity to FSRS grade
        let grade = match similarity {
            s if s >= 0.9 => ReviewGrade::Easy,
            s if s >= 0.75 => ReviewGrade::Good,
            s if s >= 0.6 => ReviewGrade::Hard,
            _ => ReviewGrade::Again,
        };

        // 5. Record as normal review
        self.user_repo.log_review(ReviewRecord {
            user_id: user_id.to_string(),
            node_id: node_id.clone(),
            grade,
            reviewed_at: Utc::now(),
            exercise_type: "recitation".to_string(),
            duration_ms: Some(user_contour.len() as u32 * 1000 / 44100), // Approximate
            previous_energy: Energy(0.0),  // Fetch from state
            new_energy: Energy(0.0),       // Calculate
        }).await?;

        Ok(RecitationResult {
            similarity_score: similarity,
            grade,
            aligned_user_contour: user_contour,
            aligned_reference_contour: reference.f0_values,
        })
    }
}

pub struct RecitationResult {
    pub similarity_score: f32,           // 0.0-1.0
    pub grade: ReviewGrade,
    pub aligned_user_contour: Vec<f32>,
    pub aligned_reference_contour: Vec<f32>,
}
```

### What Sprint 7 MUST Include

**1. Schema Creation**
- Create `audio_pitch_contours` table (empty for now)
- Create `reciters` table with basic metadata

**2. Repository Method Stub**
```rust
// iqrah-storage/src/content/repository.rs
async fn get_pitch_contour(
    &self,
    node_id: &NodeId,
    reciter_id: &str,
) -> Result<PitchContour> {
    let row = sqlx::query!(
        "SELECT f0_contour, sample_rate, duration_ms
         FROM audio_pitch_contours
         WHERE node_id = ? AND reciter_id = ?",
        node_id.as_str(),
        reciter_id
    )
    .fetch_one(&self.pool)
    .await?;

    // Decode CBOR blob
    let f0_values: Vec<f32> = ciborium::from_reader(&row.f0_contour[..])?;

    Ok(PitchContour {
        f0_values,
        sample_rate: row.sample_rate as u32,
        duration_ms: row.duration_ms as u32,
    })
}
```

**3. Exercise Type Extensibility**
```rust
// iqrah-core/src/domain/models.rs
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExerciseType {
    Recall,
    McqArToEn,
    McqEnToAr,
    Recitation,          // NEW: Reserved for Sprint 8
}
```

### What Can Wait Until Sprint 8

**R&D Outputs (Python → Rust):**
- Pitch extraction implementation (crepe → Rust equivalent)
- DTW algorithm (fastdtw → Rust crate)
- Reference contour generation pipeline

**Sprint 8 Implementation:**
- Audio recording in Flutter
- Chart rendering (fl_chart)
- Recitation practice UI
- Audio service integration

### Risk Mitigation

**Risk 1: Audio processing too slow on device**
- **Mitigation**: R&D phase must test on target devices (Android/iOS)
- Fallback: Cloud-based processing for older devices

**Risk 2: CBOR contours too large**
- **Mitigation**: Compression + decimation
```python
# R&D pipeline: Decimate F0 contour to 100 Hz
f0_decimated = decimate(f0_contour, factor=4)  # 400 Hz → 100 Hz
# Store as CBOR with zstd compression
```

**Risk 3: Multiple reciters = storage bloat**
- **Mitigation**: Start with 1 reciter (e.g., Husary), add more in updates

---

## 4. Cross-Cutting Concerns

### Content Versioning

**Problem**: How to update content.db without breaking user.db?

**Solution**: Semantic versioning + migration hooks
```sql
-- content.db
CREATE TABLE schema_version (
    version TEXT PRIMARY KEY,           -- e.g., "1.2.0"
    applied_at INTEGER NOT NULL,
    description TEXT
) STRICT, WITHOUT ROWID;

INSERT INTO schema_version VALUES ('1.0.0', unixepoch(), 'Initial release');
```

**Migration Strategy:**
```rust
pub struct ContentMigration {
    pub from_version: Version,
    pub to_version: Version,
    pub requires_energy_recalc: bool,
    pub affected_nodes: Vec<NodeId>,
}

async fn apply_content_update(
    &self,
    user_id: &str,
    migration: ContentMigration,
) -> Result<()> {
    if migration.requires_energy_recalc {
        for node_id in migration.affected_nodes {
            self.recalculate_energy(user_id, &node_id).await?;
        }
    }

    self.user_repo.set_setting(
        user_id,
        "content_db_version",
        &migration.to_version.to_string(),
    ).await?;

    Ok(())
}
```

### Repository Composition

**Pattern**: Compose repositories instead of bloating interfaces
```rust
// iqrah-core/src/services/learning_service.rs
pub struct LearningService {
    content: Arc<dyn ContentRepository>,
    user: Arc<dyn UserRepository>,
    questions: Arc<dyn QuestionRepository>,  // Optional feature
    audio: Option<Arc<dyn AudioRepository>>, // Optional feature
    scheduler: Arc<dyn Scheduler>,
}

impl LearningService {
    pub fn new(
        content: Arc<dyn ContentRepository>,
        user: Arc<dyn UserRepository>,
        scheduler: Arc<dyn Scheduler>,
    ) -> Self {
        Self {
            content,
            user,
            questions: Arc::new(NoopQuestionRepository), // Default stub
            audio: None,
            scheduler,
        }
    }

    pub fn with_questions(mut self, repo: Arc<dyn QuestionRepository>) -> Self {
        self.questions = repo;
        self
    }

    pub fn with_audio(mut self, repo: Arc<dyn AudioRepository>) -> Self {
        self.audio = Some(repo);
        self
    }
}

// Stub implementation for disabled features
pub struct NoopQuestionRepository;

impl QuestionRepository for NoopQuestionRepository {
    async fn get_questions_for_node(&self, _: &NodeId) -> Result<Vec<Question>> {
        Ok(Vec::new())  // No questions = feature disabled
    }
}
```

### Feature Flags

**Cargo features for optional dependencies:**
```toml
# iqrah-core/Cargo.toml
[features]
default = []
questions = []
audio = ["symphonia", "dasp"]  # Audio processing crates
hadith = []

[dependencies]
# Always included
async-trait = "0.1"
thiserror = "1.0"

# Optional
symphonia = { version = "0.5", optional = true }
dasp = { version = "0.11", optional = true }
```

**Runtime checks:**
```rust
pub fn is_feature_enabled(&self, feature: Feature) -> bool {
    match feature {
        Feature::Questions => cfg!(feature = "questions"),
        Feature::Audio => cfg!(feature = "audio"),
        Feature::Hadith => cfg!(feature = "hadith"),
    }
}
```

---

## 5. Testing Strategy for Future Features

### Property Tests for Questions
```rust
use proptest::prelude::*;

#[proptest]
fn question_energy_never_exceeds_auto_energy(
    auto_mastery: f32,
    question_masteries: Vec<f32>,
) {
    prop_assume!(auto_mastery >= 0.0 && auto_mastery <= 1.0);
    prop_assume!(question_masteries.iter().all(|&m| m >= 0.0 && m <= 1.0));

    let combined = calculate_node_energy(auto_mastery, question_masteries);
    prop_assert!(combined <= auto_mastery);  // Questions can only reduce energy
}

#[proptest]
fn multi_node_question_affects_all_linked_nodes(
    linked_nodes: Vec<NodeId>,
    question_mastery: f32,
) {
    prop_assume!(linked_nodes.len() > 1);

    // Reviewing a question should update ALL linked nodes
    let updates = process_question_review(question_id, ReviewGrade::Good);

    for node_id in linked_nodes {
        prop_assert!(updates.contains_key(&node_id));
    }
}
```

### Integration Tests for Hadith
```rust
#[tokio::test]
async fn hadith_propagation_isolated_from_quran() {
    let repo = setup_test_repo().await;

    // Create Quran node + Hadith node + edge between them
    let quran_node = NodeId::from("AYAH:2:183");
    let hadith_node = NodeId::from("HADITH:bukhari:1:1");

    repo.add_edge(Edge {
        source_id: quran_node.clone(),
        target_id: hadith_node.clone(),
        edge_type: EdgeType::HadithExplains,
        strength: 0.5,
    }).await?;

    // Review Quran node WITHOUT hadith enabled
    let config = PropagationConfig {
        include_hadith: false,
        ..Default::default()
    };

    let updates = propagate_energy(&repo, &quran_node, 0.5, config).await?;

    // Hadith node should NOT receive energy
    assert!(!updates.contains_key(&hadith_node));

    // Now enable hadith
    let config = PropagationConfig {
        include_hadith: true,
        ..Default::default()
    };

    let updates = propagate_energy(&repo, &quran_node, 0.5, config).await?;

    // Hadith node SHOULD receive energy
    assert!(updates.contains_key(&hadith_node));
    assert_eq!(updates[&hadith_node], 0.25);  // 0.5 * 0.5 strength
}
```

### Mock Audio for Testing
```rust
#[tokio::test]
async fn audio_analysis_converts_to_fsrs_grade() {
    let service = setup_audio_service().await;

    // Generate mock user audio with known similarity
    let reference = vec![440.0; 1000];  // A4 note
    let user_audio_good = add_noise(&reference, 0.05);  // 95% similar
    let user_audio_bad = add_noise(&reference, 0.5);    // 50% similar

    let result_good = service.analyze_recitation(
        "user_id",
        &NodeId::from("AYAH:1:1"),
        "husary",
        user_audio_good,
    ).await?;

    assert_eq!(result_good.grade, ReviewGrade::Easy);

    let result_bad = service.analyze_recitation(
        "user_id",
        &NodeId::from("AYAH:1:1"),
        "husary",
        user_audio_bad,
    ).await?;

    assert_eq!(result_bad.grade, ReviewGrade::Again);
}
```

---

## 6. Documentation Requirements

### API Documentation
```rust
/// Calculate node energy considering both automatic exercises and expert questions.
///
/// # Energy Formula
///
/// If a node has no linked questions:
/// ```
/// E_node = E_auto_exercises
/// ```
///
/// If a node has N linked questions:
/// ```
/// E_node = E_auto_exercises × (Σ E_question_i / N)
/// ```
///
/// This ensures that:
/// 1. Questions can only reduce energy (user must master both auto + questions)
/// 2. Full mastery requires 100% on ALL questions
/// 3. Zero questions = backward compatible with current system
///
/// # Examples
///
/// ```rust
/// // No questions: use auto-exercise energy directly
/// let energy = calculate_node_energy(0.8, vec![]);
/// assert_eq!(energy, 0.8);
///
/// // With questions: multiply by average question mastery
/// let energy = calculate_node_energy(0.8, vec![1.0, 0.5, 0.5]);
/// assert_eq!(energy, 0.8 * (2.0 / 3.0)); // ≈ 0.533
/// ```
pub fn calculate_node_energy(
    auto_exercise_mastery: f32,
    question_masteries: Vec<f32>,
) -> f32 {
    // Implementation...
}
```

### Migration Guides
```markdown
# Migrating to Questions Support (v1.1.0 → v1.2.0)

## What Changed

- New tables: `questions`, `question_node_links`, `question_memory_states`
- Energy calculation now considers linked questions
- Content versioning added

## Breaking Changes

None! Existing users see no difference until questions are added to content.db.

## For Developers

If you're building custom tools:

1. Check content version before queries:
   ```rust
   let version = content_repo.get_version().await?;
   if version >= Version::new(1, 2, 0) {
       // Questions available
   }
   ```

2. When adding questions to content.db:
   ```rust
   // Mark affected nodes for energy recalc
   migration.mark_for_recalc(&affected_node_ids);
   ```
```

---

## Summary: Sprint 7 Deliverables for Extensibility

### ✅ Database Schema
- [ ] All question tables created (even if empty)
- [ ] Audio pitch contours table created
- [ ] Edge types support hadith (enum values 2, 3 reserved)
- [ ] Content version tracking in user_settings

### ✅ Repository Interfaces
- [ ] QuestionRepository trait defined
- [ ] ContentRepository includes get_pitch_contour()
- [ ] Stub implementations for disabled features

### ✅ Domain Logic
- [ ] Energy calculation handles questions
- [ ] Propagation respects edge types
- [ ] Exercise type enum includes Recitation

### ✅ Service Layer
- [ ] QuestionService with energy recalculation
- [ ] Content version sync on startup
- [ ] Feature composition pattern (with_questions, with_audio)

### ✅ Testing
- [ ] Property tests for question energy
- [ ] Integration tests for hadith isolation
- [ ] Mock audio analysis tests

### ✅ Documentation
- [ ] API docs for energy calculation
- [ ] Migration guide template
- [ ] Feature flag usage guide

---

## Risk Summary

| Feature | Risk | Mitigation | Status |
|---------|------|------------|--------|
| Questions | Break energy propagation | Property tests + stub implementation | ✅ Mitigated |
| Questions | Content updates corrupt data | Version tracking + rollback | ✅ Mitigated |
| Hadith | Affects Quran mastery | Separate edge types + config flag | ✅ Mitigated |
| Hadith | Graph too large | Lazy loading by node type | ✅ Mitigated |
| Audio | Too slow on device | R&D phase benchmarks required | ⚠️ Monitor |
| Audio | Storage bloat | Decimation + compression | ✅ Mitigated |

---

**Next Steps:**
1. Review this document with team
2. Prioritize Sprint 7 deliverables (schema, stubs, tests)
3. Validate R&D outputs align with architecture
4. Begin implementation of core extensibility features

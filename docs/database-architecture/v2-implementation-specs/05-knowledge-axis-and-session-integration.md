# Knowledge Axis and Session Integration

**Last Updated:** 2025-11-17
**Status:** Design Spec (Post-MVP Feature)
**Priority:** P3 (Advanced Features, Not MVP Blocker)

## Context

The Python knowledge graph generator creates sophisticated **knowledge axis nodes** to represent multi-dimensional learning:
- `WORD_INSTANCE:1:1:1:memorization` - Can you recall the word?
- `WORD_INSTANCE:1:1:1:translation` - Do you understand its meaning?
- `WORD_INSTANCE:1:1:1:tajweed` - Can you pronounce it correctly?

These nodes are correctly generated and imported, but the Rust runtime:
- ❌ Doesn't parse the axis suffix from node IDs
- ❌ Filters out Knowledge-type nodes from sessions
- ❌ Cannot target specific axes in exercises

**Result:** The sophisticated multi-dimensional learning model exists in the graph but is unused at runtime.

## Goal

Provide a **phased implementation plan** to integrate knowledge axis support into the Rust runtime, enabling axis-specific exercises and cross-axis learning synergies.

**Non-Goal:** This is NOT required for MVP. Document the design for future implementation when axis-specific exercises become a priority.

## Design Overview

### Knowledge Axis Concept

**Base Node:** `WORD_INSTANCE:1:1:1` (the word itself, in content.db)

**Knowledge Axis Nodes:** (in knowledge graph)
- `WORD_INSTANCE:1:1:1:memorization`
- `WORD_INSTANCE:1:1:1:translation`
- `WORD_INSTANCE:1:1:1:tajweed`
- `WORD_INSTANCE:1:1:1:contextual_memorization`

**Edges Encode Learning Synergies:**
```
translation → memorization (understanding helps recall)
tajweed → memorization (pronunciation practice reinforces memory)
word.memorization → verse.memorization (knowing words helps know verse)
```

**Rust Implementation Gap:**
```rust
// Current
pub enum NodeType {
    Root, Lemma, Word, WordInstance, Verse, Chapter,
    Knowledge,  // ← Generic, no axis information
}

// Needed
pub enum KnowledgeAxis {
    Memorization,
    Translation,
    Tafsir,
    Tajweed,
    ContextualMemorization,
    Meaning,
}
```

## Phased Implementation Plan

### Phase 1: Domain Model & Parsing (2-3 days)

**Goal:** Add KnowledgeAxis enum and parse axis from node IDs.

#### Step 1.1: Add KnowledgeAxis Enum

**File:** `rust/crates/iqrah-core/src/domain/models.rs`

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum KnowledgeAxis {
    Memorization,
    Translation,
    Tafsir,
    Tajweed,
    ContextualMemorization,
    Meaning,
}

impl KnowledgeAxis {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "memorization" => Some(Self::Memorization),
            "translation" => Some(Self::Translation),
            "tafsir" => Some(Self::Tafsir),
            "tajweed" => Some(Self::Tajweed),
            "contextual_memorization" => Some(Self::ContextualMemorization),
            "meaning" => Some(Self::Meaning),
            _ => None,
        }
    }

    pub fn to_str(&self) -> &'static str {
        match self {
            Self::Memorization => "memorization",
            Self::Translation => "translation",
            Self::Tafsir => "tafsir",
            Self::Tajweed => "tajweed",
            Self::ContextualMemorization => "contextual_memorization",
            Self::Meaning => "meaning",
        }
    }
}
```

#### Step 1.2: Add KnowledgeNode Type

```rust
#[derive(Debug, Clone)]
pub struct KnowledgeNode {
    pub base_node_id: String,  // "WORD_INSTANCE:1:1:1"
    pub axis: KnowledgeAxis,   // Memorization
    pub full_id: String,       // "WORD_INSTANCE:1:1:1:memorization"
}

impl KnowledgeNode {
    pub fn parse(node_id: &str) -> Option<Self> {
        let parts: Vec<&str> = node_id.split(':').collect();

        // Must have at least 2 parts (base + axis)
        if parts.len() < 2 {
            return None;
        }

        // Last part is axis
        let axis_str = parts.last()?;
        let axis = KnowledgeAxis::from_str(axis_str)?;

        // Everything except last part is base node ID
        let base_parts = &parts[..parts.len() - 1];
        let base_node_id = base_parts.join(":");

        Some(Self {
            base_node_id,
            axis,
            full_id: node_id.to_string(),
        })
    }

    pub fn new(base_node_id: String, axis: KnowledgeAxis) -> Self {
        let full_id = format!("{}:{}", base_node_id, axis.to_str());
        Self {
            base_node_id,
            axis,
            full_id,
        }
    }
}
```

#### Step 1.3: Update Node Model

```rust
// Extend existing Node struct
pub struct Node {
    pub id: String,
    pub node_type: NodeType,
    pub knowledge_node: Option<KnowledgeNode>,  // Parsed if node_type == Knowledge
    pub created_at: i64,
}

impl Node {
    pub fn from_row(row: NodeRow) -> Self {
        let knowledge_node = if row.node_type == "knowledge" {
            KnowledgeNode::parse(&row.id)
        } else {
            None
        };

        Self {
            id: row.id,
            node_type: NodeType::from_str(&row.node_type),
            knowledge_node,
            created_at: row.created_at,
        }
    }
}
```

#### Step 1.4: Tests

```rust
#[test]
fn test_parse_knowledge_node() {
    let node_id = "WORD_INSTANCE:1:1:1:memorization";
    let kn = KnowledgeNode::parse(node_id).unwrap();

    assert_eq!(kn.base_node_id, "WORD_INSTANCE:1:1:1");
    assert_eq!(kn.axis, KnowledgeAxis::Memorization);
    assert_eq!(kn.full_id, node_id);
}

#[test]
fn test_parse_invalid_axis() {
    let node_id = "WORD_INSTANCE:1:1:1:invalid_axis";
    assert!(KnowledgeNode::parse(node_id).is_none());
}

#[test]
fn test_construct_knowledge_node() {
    let kn = KnowledgeNode::new(
        "WORD_INSTANCE:1:1:1".to_string(),
        KnowledgeAxis::Translation
    );

    assert_eq!(kn.full_id, "WORD_INSTANCE:1:1:1:translation");
}
```

**Deliverables:**
- [ ] KnowledgeAxis enum added
- [ ] KnowledgeNode struct added
- [ ] Parsing logic implemented
- [ ] Tests pass
- [ ] Documentation updated

**Effort:** 1 day

### Phase 2: Axis-Aware Session Generation (3-4 days)

**Goal:** Enable session service to generate exercises targeting specific axes.

#### Step 2.1: Update Session Service Signature

**File:** `rust/crates/iqrah-core/src/services/session_service.rs`

```rust
pub struct SessionOptions {
    pub user_id: String,
    pub limit: i64,
    pub axis_filter: Option<KnowledgeAxis>,  // NEW: Filter by axis
    pub content_types: Vec<ContentType>,     // Filter by content type (verse, word, etc.)
}

impl SessionService {
    pub async fn get_due_items(
        &self,
        options: SessionOptions,
    ) -> Result<Vec<SessionItem>> {
        // Get due memory states
        let due_states = self.user_repo.get_due_states(&options.user_id, options.limit * 2).await?;

        let mut items = Vec::new();
        for state in due_states {
            // Parse node ID
            let node = self.content_repo.get_node(&state.content_key).await?;

            // NEW: Filter by axis
            if let Some(axis_filter) = &options.axis_filter {
                if let Some(kn) = &node.knowledge_node {
                    if kn.axis != *axis_filter {
                        continue;  // Skip nodes of different axis
                    }
                } else {
                    continue;  // Skip non-knowledge nodes when filtering by axis
                }
            }

            // Filter by content type
            if !options.content_types.contains(&node.node_type.into()) {
                continue;
            }

            // Calculate priority
            let score = self.calculate_priority(&state, &node);

            items.push(SessionItem {
                node_id: state.content_key,
                node_type: node.node_type,
                knowledge_axis: node.knowledge_node.map(|kn| kn.axis),
                energy: state.energy,
                priority_score: score,
            });
        }

        // Sort and limit
        items.sort_by(|a, b| b.priority_score.partial_cmp(&a.priority_score).unwrap());
        items.truncate(options.limit as usize);

        Ok(items)
    }
}
```

#### Step 2.2: Update SessionItem Model

```rust
pub struct SessionItem {
    pub node_id: String,
    pub node_type: NodeType,
    pub knowledge_axis: Option<KnowledgeAxis>,  // NEW
    pub energy: f64,
    pub priority_score: f64,
}
```

#### Step 2.3: Remove Knowledge Node Filter

**Current code filters out Knowledge nodes:**
```rust
// OLD (remove this)
if !matches!(node.node_type, NodeType::WordInstance | NodeType::Verse) {
    continue;
}
```

**New code includes Knowledge nodes:**
```rust
// NEW: Include Knowledge nodes, filter by axis if specified
match node.node_type {
    NodeType::Knowledge => {
        // If axis filter is specified, check it (already done above)
        // Otherwise, include all knowledge nodes
    }
    NodeType::WordInstance | NodeType::Verse => {
        // Include as before
    }
    _ => continue,  // Skip other types
}
```

#### Step 2.4: Usage Examples

```rust
// Get memorization exercises only
let memorization_session = session_service.get_due_items(SessionOptions {
    user_id: "default".to_string(),
    limit: 10,
    axis_filter: Some(KnowledgeAxis::Memorization),
    content_types: vec![ContentType::Word, ContentType::Verse],
}).await?;

// Get translation exercises only
let translation_session = session_service.get_due_items(SessionOptions {
    user_id: "default".to_string(),
    limit: 10,
    axis_filter: Some(KnowledgeAxis::Translation),
    content_types: vec![ContentType::Word],
}).await?;

// Get mixed session (no axis filter)
let mixed_session = session_service.get_due_items(SessionOptions {
    user_id: "default".to_string(),
    limit: 10,
    axis_filter: None,
    content_types: vec![ContentType::Word, ContentType::Verse],
}).await?;
```

**Deliverables:**
- [ ] SessionOptions struct with axis_filter
- [ ] Updated get_due_items implementation
- [ ] Knowledge node filter removed
- [ ] Tests for axis-filtered sessions
- [ ] Tests for mixed sessions

**Effort:** 2-3 days

### Phase 3: Axis-Specific Exercise Generation (1 week)

**Goal:** Implement different exercise types based on knowledge axis.

#### Step 3.1: Exercise Type System

**File:** `rust/crates/iqrah-core/src/exercises/mod.rs`

```rust
pub trait Exercise {
    fn generate_question(&self) -> String;
    fn check_answer(&self, answer: &str) -> bool;
    fn get_hint(&self) -> Option<String>;
}

pub enum ExerciseType {
    Memorization(MemorizationExercise),
    Translation(TranslationExercise),
    Tajweed(TajweedExercise),
    ContextualMemorization(ContextualMemorizationExercise),
}

impl ExerciseType {
    pub fn from_session_item(
        item: &SessionItem,
        content_repo: &dyn ContentRepository,
    ) -> Result<Self> {
        match item.knowledge_axis {
            Some(KnowledgeAxis::Memorization) => {
                Ok(Self::Memorization(MemorizationExercise::new(item, content_repo)?))
            }
            Some(KnowledgeAxis::Translation) => {
                Ok(Self::Translation(TranslationExercise::new(item, content_repo)?))
            }
            Some(KnowledgeAxis::Tajweed) => {
                Ok(Self::Tajweed(TajweedExercise::new(item, content_repo)?))
            }
            Some(KnowledgeAxis::ContextualMemorization) => {
                Ok(Self::ContextualMemorization(ContextualMemorizationExercise::new(item, content_repo)?))
            }
            _ => Err(Error::UnsupportedExerciseType),
        }
    }
}
```

#### Step 3.2: Memorization Exercise

```rust
pub struct MemorizationExercise {
    node_id: String,
    word_text: String,
    verse_context: String,
}

impl MemorizationExercise {
    pub fn new(item: &SessionItem, content_repo: &dyn ContentRepository) -> Result<Self> {
        // Extract base node ID (remove :memorization suffix)
        let kn = KnowledgeNode::parse(&item.node_id)
            .ok_or(Error::InvalidKnowledgeNode)?;

        // Get word text
        let word_text = content_repo.get_quran_text(&kn.base_node_id).await?;

        // Get verse context (for hint)
        let verse_key = extract_verse_key(&kn.base_node_id)?;
        let verse_context = content_repo.get_quran_text(&verse_key).await?;

        Ok(Self {
            node_id: item.node_id.clone(),
            word_text,
            verse_context,
        })
    }
}

impl Exercise for MemorizationExercise {
    fn generate_question(&self) -> String {
        "Recall the word".to_string()
        // UI shows: position in verse, maybe first letter as hint
    }

    fn check_answer(&self, answer: &str) -> bool {
        normalize_arabic(answer) == normalize_arabic(&self.word_text)
    }

    fn get_hint(&self) -> Option<String> {
        Some(format!("Verse context: {}", self.verse_context))
    }
}
```

#### Step 3.3: Translation Exercise

```rust
pub struct TranslationExercise {
    node_id: String,
    word_text: String,
    translation: String,
}

impl TranslationExercise {
    pub fn new(item: &SessionItem, content_repo: &dyn ContentRepository) -> Result<Self> {
        let kn = KnowledgeNode::parse(&item.node_id)
            .ok_or(Error::InvalidKnowledgeNode)?;

        let word_text = content_repo.get_quran_text(&kn.base_node_id).await?;

        // Get translation (requires word_translations table or default)
        let translation = content_repo.get_word_translation(&kn.base_node_id, 1).await?
            .unwrap_or_else(|| "[Translation not available]".to_string());

        Ok(Self {
            node_id: item.node_id.clone(),
            word_text,
            translation,
        })
    }
}

impl Exercise for TranslationExercise {
    fn generate_question(&self) -> String {
        format!("What does '{}' mean?", self.word_text)
    }

    fn check_answer(&self, answer: &str) -> bool {
        // Fuzzy match for translation
        let normalized_answer = answer.to_lowercase().trim();
        let normalized_translation = self.translation.to_lowercase();

        normalized_translation.contains(normalized_answer) ||
        normalized_answer.contains(&normalized_translation)
    }

    fn get_hint(&self) -> Option<String> {
        // Show first word of translation as hint
        Some(self.translation.split_whitespace().next()?.to_string())
    }
}
```

#### Step 3.4: Exercise Service

```rust
pub struct ExerciseService {
    content_repo: Arc<dyn ContentRepository>,
}

impl ExerciseService {
    pub async fn generate_exercise(&self, item: &SessionItem) -> Result<Box<dyn Exercise>> {
        let exercise_type = ExerciseType::from_session_item(item, &*self.content_repo).await?;

        match exercise_type {
            ExerciseType::Memorization(ex) => Ok(Box::new(ex)),
            ExerciseType::Translation(ex) => Ok(Box::new(ex)),
            ExerciseType::Tajweed(ex) => Ok(Box::new(ex)),
            ExerciseType::ContextualMemorization(ex) => Ok(Box::new(ex)),
        }
    }
}
```

**Deliverables:**
- [ ] Exercise trait defined
- [ ] ExerciseType enum with axis mapping
- [ ] MemorizationExercise implemented
- [ ] TranslationExercise implemented
- [ ] TajweedExercise implemented (optional)
- [ ] ExerciseService orchestration
- [ ] Tests for each exercise type

**Effort:** 4-5 days

### Phase 4: Cross-Axis Energy Propagation (2-3 days)

**Goal:** Leverage cross-axis edges for learning synergies.

**Current State:** Energy propagates through edges, but axis-agnostic.

**Enhanced Propagation:**

```rust
pub async fn propagate_energy(
    &self,
    user_id: &str,
    source_node_id: &str,
    source_energy: f64,
) -> Result<()> {
    let edges = self.content_repo.get_edges_from(source_node_id).await?;

    // Parse source as knowledge node
    let source_kn = KnowledgeNode::parse(source_node_id);

    for edge in edges {
        let energy_change = self.calculate_energy_transfer(
            source_energy,
            edge.distribution_type,
            edge.param1,
            edge.param2,
        );

        // NEW: Axis-aware logging
        let target_kn = KnowledgeNode::parse(&edge.target_id);

        let reason = match (&source_kn, &target_kn) {
            (Some(src), Some(tgt)) if src.axis != tgt.axis => {
                format!("Cross-axis edge: {:?} → {:?}", src.axis, tgt.axis)
            }
            (Some(src), Some(tgt)) if src.base_node_id != tgt.base_node_id => {
                format!("Hierarchical edge: {} → {}", src.base_node_id, tgt.base_node_id)
            }
            _ => format!("{:?} edge", edge.edge_type),
        };

        self.user_repo.update_energy(&edge.target_id, energy_change).await?;
        self.user_repo.log_propagation(source_node_id, &edge.target_id, energy_change, &reason).await?;
    }

    Ok(())
}
```

**Benefit:** Detailed logging shows which cross-axis relationships are being exercised.

**Deliverables:**
- [ ] Axis-aware propagation logging
- [ ] Analytics query to find most active cross-axis edges
- [ ] Tests for cross-axis propagation

**Effort:** 1-2 days

## Timeline & Effort Summary

| Phase | Description | Effort | Priority |
|-------|-------------|--------|----------|
| Phase 1 | Domain Model & Parsing | 1-2 days | P3 (Post-MVP) |
| Phase 2 | Axis-Aware Session Generation | 2-3 days | P3 (Post-MVP) |
| Phase 3 | Axis-Specific Exercise Generation | 4-5 days | P3 (Post-MVP) |
| Phase 4 | Cross-Axis Energy Propagation | 1-2 days | P4 (Optional) |
| **Total** | | **8-12 days** (2 weeks) | Post-MVP |

## Decision Point: Implement or Simplify?

**Option A: Implement Full Axis Support (This Document)**
- **Effort:** 2 weeks
- **Benefit:** Full parity with Python design, sophisticated multi-dimensional learning
- **When:** When axis-specific exercises are core to UX

**Option B: Defer to Post-MVP**
- **Effort:** 0 (current state)
- **Benefit:** Faster MVP
- **Trade-off:** Users cannot target specific learning dimensions
- **When:** MVP timeline is tight

**Option C: Simplify Python Graph**
- **Effort:** 2-3 days (update Python)
- **Benefit:** Eliminate gap between design and implementation
- **Trade-off:** Lose sophisticated multi-dimensional model
- **When:** Axis model is deemed over-engineered for actual use case

## Recommendation

**For MVP:** Option B (defer)
- Axis nodes are generated and imported, just not used
- No user-facing impact (users won't miss what they never had)
- Can be enabled post-launch without breaking changes

**Post-MVP:** Option A (implement)
- If user testing shows demand for axis-specific exercises
- Implement in phases (start with Phase 1+2, add Phase 3 based on feedback)

## Testing Strategy

### Unit Tests

```rust
#[tokio::test]
async fn test_session_with_axis_filter() {
    let session_service = create_test_session_service().await;

    let items = session_service.get_due_items(SessionOptions {
        user_id: "default".to_string(),
        limit: 10,
        axis_filter: Some(KnowledgeAxis::Memorization),
        content_types: vec![ContentType::Word],
    }).await.unwrap();

    // All items should be memorization exercises
    for item in items {
        assert_eq!(item.knowledge_axis, Some(KnowledgeAxis::Memorization));
    }
}

#[tokio::test]
async fn test_memorization_exercise() {
    let exercise = MemorizationExercise::new(&test_item(), &test_content_repo()).await.unwrap();

    let question = exercise.generate_question();
    assert!(!question.is_empty());

    let correct = exercise.check_answer("بِسْمِ");
    assert!(correct);

    let incorrect = exercise.check_answer("wrong");
    assert!(!incorrect);
}
```

### Integration Tests

```rust
#[tokio::test]
async fn test_full_axis_workflow() {
    let session_service = create_test_session_service().await;
    let exercise_service = create_test_exercise_service().await;
    let learning_service = create_test_learning_service().await;

    // 1. Get memorization session
    let items = session_service.get_due_items(SessionOptions {
        user_id: "default".to_string(),
        limit: 1,
        axis_filter: Some(KnowledgeAxis::Memorization),
        content_types: vec![ContentType::Word],
    }).await.unwrap();

    let item = &items[0];

    // 2. Generate exercise
    let exercise = exercise_service.generate_exercise(item).await.unwrap();

    // 3. User answers
    let correct = exercise.check_answer("بِسْمِ");
    assert!(correct);

    // 4. Process review
    let result = learning_service.process_review(
        "default",
        &item.node_id,
        ReviewGrade::Good,
    ).await.unwrap();

    assert!(result.new_stability > 0.0);
}
```

## Validation Checklist

- [ ] KnowledgeAxis enum matches Python definition
- [ ] Parsing logic handles all valid node ID formats
- [ ] Session service can filter by axis
- [ ] Knowledge nodes no longer filtered out
- [ ] Each axis has corresponding exercise implementation
- [ ] Cross-axis edges are logged during propagation
- [ ] All tests pass
- [ ] Documentation updated

## References

- [06-knowledge-axis-design.md](../06-knowledge-axis-design.md) - Original gap analysis (from previous audit)
- Python implementation: [graph/knowledge.py](../../research_and_dev/iqrah-knowledge-graph2/src/iqrah/graph/knowledge.py)
- Python knowledge builder: [graph/knowledge_builder.py](../../research_and_dev/iqrah-knowledge-graph2/src/iqrah/graph/knowledge_builder.py)

---

**Status:** Design ready, implementation deferred to post-MVP
**Decision Required:** Implement now (2 weeks) or defer to post-MVP (recommended)?
**Next Steps:** If implementing, start with Phase 1 (domain model & parsing)

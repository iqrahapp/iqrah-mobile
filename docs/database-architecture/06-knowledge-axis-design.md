# Knowledge Axis Design

**Related Question:** Q8 - How much of the axis-based knowledge node design is understood and implemented?

## Overview

The Knowledge Axis design is a **sophisticated multi-dimensional learning model** where each base node (word, verse, chapter) has multiple knowledge dimensions (memorization, translation, tajweed, etc.), and learning in one dimension can influence learning in others.

**Critical Insight:** This is NOT just categorizing nodes - it's about encoding **cross-dimensional relationships** in the graph to model real learning phenomena (e.g., understanding translation helps memorization).

## Design Concept

### The Problem

Traditional spaced repetition treats each fact as independent. But Quranic learning has interconnected dimensions:

**Example:**
- Memorizing the word "بِسْمِ" (bismillah)
- Understanding its translation ("In the name of")
- Knowing its tajweed rules (pronunciation)
- Recognizing its context within the verse

**Reality:** Understanding the translation HELPS memorization. Knowing tajweed REINFORCES memorization. These aren't independent skills.

### The Solution: Knowledge Axes

**Base Node:** `WORD_INSTANCE:1:1:1` (the word itself)

**Knowledge Axis Nodes:**
- `WORD_INSTANCE:1:1:1:memorization` - Can you recall the word?
- `WORD_INSTANCE:1:1:1:translation` - Do you understand its meaning?
- `WORD_INSTANCE:1:1:1:tajweed` - Can you pronounce it correctly?
- `WORD_INSTANCE:1:1:1:contextual_memorization` - Can you recall it in verse context?

**Edges encode learning synergies:**
```
WORD_INSTANCE:1:1:1:translation → WORD_INSTANCE:1:1:1:memorization
(Understanding helps memorization)

WORD_INSTANCE:1:1:1:tajweed → WORD_INSTANCE:1:1:1:memorization
(Pronunciation practice helps memorization)
```

## Python Implementation (Design Authority)

### Knowledge Axis Definitions

**Location:** [graph/knowledge.py](../../research_and_dev/iqrah-knowledge-graph2/src/iqrah/graph/knowledge.py) (lines 1-316)

**Axis Enum:**
```python
class KnowledgeAxis(Enum):
    MEMORIZATION = "memorization"
    TRANSLATION = "translation"
    TAFSIR = "tafsir"
    TAJWEED = "tajweed"
    CONTEXTUAL_MEMORIZATION = "contextual_memorization"
    MEANING = "meaning"  # For roots/lemmas
```

**Node Type → Allowed Axes Mapping:**
```python
class NodeType(Enum):
    CHAPTER = {
        KnowledgeAxis.MEMORIZATION,
        KnowledgeAxis.TRANSLATION,
        KnowledgeAxis.TAFSIR,
    }

    VERSE = {
        KnowledgeAxis.MEMORIZATION,
        KnowledgeAxis.TRANSLATION,
        KnowledgeAxis.TAFSIR,
        KnowledgeAxis.TAJWEED,
    }

    WORD_INSTANCE = {
        KnowledgeAxis.MEMORIZATION,
        KnowledgeAxis.TRANSLATION,
        KnowledgeAxis.TAJWEED,
        KnowledgeAxis.CONTEXTUAL_MEMORIZATION,
    }

    LEMMA = {
        KnowledgeAxis.MEANING,
    }

    ROOT = {
        KnowledgeAxis.MEANING,
    }
```

**Validation Logic:**
```python
# Lines 86-125
def _validate_knowledge_node(self, node_id: str):
    """Ensure knowledge axis is valid for the base node type."""
    parts = node_id.split(":")
    if len(parts) < 2:
        raise ValueError(f"Invalid knowledge node ID: {node_id}")

    # Extract axis (last part)
    axis_str = parts[-1]
    knowledge_axis = KnowledgeAxis(axis_str)

    # Extract base node (everything except last part)
    base_node_id = ":".join(parts[:-1])
    node_type = self._get_node_type(base_node_id)

    # Check if axis is allowed for this node type
    if knowledge_axis not in node_type.value:
        raise ValueError(
            f"Invalid knowledge axis '{axis_str}' for node type {node_type}. "
            f"Allowed axes: {[a.value for a in node_type.value]}"
        )
```

**Example Validation:**
```python
# Valid
_validate_knowledge_node("WORD_INSTANCE:1:1:1:memorization")  # ✅
_validate_knowledge_node("VERSE:1:1:translation")  # ✅

# Invalid
_validate_knowledge_node("WORD_INSTANCE:1:1:1:tafsir")  # ❌ Word instances don't have tafsir axis
_validate_knowledge_node("ROOT:ktb:memorization")  # ❌ Roots only have meaning axis
```

### Knowledge Edge Creation

**Location:** [graph/knowledge_builder.py](../../research_and_dev/iqrah-knowledge-graph2/src/iqrah/graph/knowledge_builder.py)

**Hierarchical Edges (Word → Verse → Chapter):**
```python
# Lines 178-182: Word memorization contributes to verse memorization
self.edge_manager.add_knowledge_edge(
    source=f"{word_id}:memorization",
    target=f"{verse_id}:memorization",
    distribution=Distribution.auto(weight=word.get_letters_count())
)

# Lines 161-165: Verse memorization contributes to chapter memorization
self.edge_manager.add_knowledge_edge(
    source=f"{verse_id}:memorization",
    target=f"{chapter_id}:memorization",
    distribution=Distribution.auto(weight=verse.get_letters_count())
)
```

**Cross-Axis Edges (Translation helps Memorization):**
```python
# Lines 301-306
for node_id in all_nodes:
    self.edge_manager.add_knowledge_edge(
        source=f"{node_id}:translation",
        target=f"{node_id}:memorization",
        distribution=Distribution.normal(mean=0.4, std=0.15)
    )
```

**Meaning:** Understanding translation provides ~40% (±15%) boost to memorization energy.

**Adjacent Word Edges (Contextual Memorization):**
```python
# Lines 390-410
for i, word in enumerate(verse.words):
    if i > 0:
        prev_word = verse.words[i - 1]
        self.edge_manager.add_knowledge_edge(
            source=f"WORD_INSTANCE:{verse_key}:{word.position}:contextual_memorization",
            target=f"WORD_INSTANCE:{verse_key}:{prev_word.position}:contextual_memorization",
            distribution=Distribution.normal(mean=0.35, std=0.12)
        )
```

**Meaning:** Memorizing word N in context helps memorize word N-1 in context (~35% ±12%).

### Node Creation Flow

**Dependency Graph Builder Creates Base Nodes:**
```python
# graph/builder.py
word_node_id = identifiers.for_word_instance(word, verse)
# → "WORD_INSTANCE:1:1:1"

self.node_manager.add_node(word_node_id, NodeType.WORD_INSTANCE)
```

**Knowledge Graph Builder Creates Axis Nodes:**
```python
# graph/knowledge_builder.py
base_node_id = "WORD_INSTANCE:1:1:1"

for axis in [KnowledgeAxis.MEMORIZATION, KnowledgeAxis.TRANSLATION, KnowledgeAxis.TAJWEED]:
    knowledge_node_id = f"{base_node_id}:{axis.value}"
    # → "WORD_INSTANCE:1:1:1:memorization"

    self.node_manager.add_node(knowledge_node_id, NodeType.KNOWLEDGE)
```

## Rust Implementation Analysis

### Domain Model

**Location:** [domain/models.rs](../../rust/crates/iqrah-core/src/domain/models.rs) (lines 6-16)

```rust
pub enum NodeType {
    Root,
    Lemma,
    Word,
    WordInstance,
    Verse,
    Chapter,
    Knowledge,  // ← This is the ONLY acknowledgment of axis nodes
}
```

**Observation:**
- ✅ `Knowledge` type exists
- ❌ No `KnowledgeAxis` enum
- ❌ No mapping of which axes are valid for which node types
- ❌ No validation

### CBOR Import

**Location:** [cbor_import.rs](../../rust/crates/iqrah-core/src/cbor_import.rs) (lines 175-183)

```rust
CborRecord::Node { id, a } => {
    nodes.push(ImportedNode {
        id,  // e.g., "WORD_INSTANCE:1:1:1:memorization"
        node_type: a.node_type,  // NodeType::Knowledge
        metadata: a.metadata,
    });
}
```

**What Happens:**
1. Python exports: `id="WORD_INSTANCE:1:1:1:memorization", node_type=Knowledge`
2. Rust imports: Node with id as string, type as enum
3. ✅ Node is stored correctly
4. ❌ Axis information (memorization) is NOT parsed from the ID
5. ❌ No awareness that this is a "memorization axis of word instance 1:1:1"

### Session Generation

**Location:** [services/session_service.rs](../../rust/crates/iqrah-core/src/services/session_service.rs) (lines 89-92)

```rust
// Filter: Only include word_instance and verse types
if !matches!(node.node_type, NodeType::WordInstance | NodeType::Verse) {
    continue;
}
```

**CRITICAL FINDING:** Knowledge axis nodes are **explicitly filtered out** of sessions!

**Implication:**
- User NEVER reviews `WORD_INSTANCE:1:1:1:memorization` nodes
- Only reviews base `WORD_INSTANCE:1:1:1` nodes
- **The entire knowledge axis system is unused at runtime**

### Energy Propagation

**Location:** [services/learning_service.rs](../../rust/crates/iqrah-core/src/services/learning_service.rs)

```rust
// After review of "WORD_INSTANCE:1:1:1"
let edges = self.content_repo.get_edges_from(node_id).await?;

for edge in edges {
    let energy_change = calculate_transfer(...);
    self.user_repo.update_energy(&edge.target_id, energy_change).await?;
}
```

**What Happens:**
1. User reviews base node: `WORD_INSTANCE:1:1:1`
2. Edges are fetched (including edges FROM axis nodes)
3. Energy propagates to targets

**Problem:** If the session never presents axis nodes for review, their energy only updates via propagation from base nodes, not from direct interaction.

**Example:**
- User reviews `WORD_INSTANCE:1:1:1` (base node)
- Energy might propagate TO `WORD_INSTANCE:1:1:1:memorization`
- But user is NEVER tested specifically on memorization vs. translation

## Gap Analysis

### What's Working

| Aspect | Status | Evidence |
|--------|--------|----------|
| **Python design** | ✅ Excellent | Complete axis definitions, validation, edge creation |
| **Graph generation** | ✅ Correct | Axis nodes created with proper IDs and types |
| **CBOR export** | ✅ Correct | Axis nodes exported with Knowledge type |
| **Rust import** | ✅ Correct | Axis nodes imported into content.db |
| **Edge propagation** | ⚠️ Partial | Energy propagates through edges, but axis-agnostic |

### What's Missing

| Aspect | Status | Impact |
|--------|--------|--------|
| **Axis enum in Rust** | ❌ Missing | Can't parse or validate axes |
| **Axis parsing from ID** | ❌ Missing | Node ID "...:memorization" not parsed |
| **Axis validation** | ❌ Missing | No check if axis valid for node type |
| **Axis-aware sessions** | ❌ Missing | Can't generate "test memorization" exercise |
| **Axis-aware exercises** | ❌ Missing | Can't target specific knowledge dimensions |
| **Knowledge node reviews** | ❌ Filtered out | Axis nodes never presented to user |

## Intended vs. Actual Behavior

### Intended Design

**Exercise Flow:**
```
1. System chooses: "Test WORD_INSTANCE:1:1:1:memorization"
2. Exercise: "Recall the first word of Al-Fatihah" (no translation shown)
3. User succeeds → memorization axis energy increases
4. Energy propagates:
   - To VERSE:1:1:memorization (hierarchical)
   - To WORD_INSTANCE:1:1:1:translation (cross-axis: knowing helps understanding)
```

**Different Exercise:**
```
1. System chooses: "Test WORD_INSTANCE:1:1:1:translation"
2. Exercise: "What does 'بِسْمِ' mean?" (Arabic shown, ask for translation)
3. User succeeds → translation axis energy increases
4. Energy propagates:
   - To WORD_INSTANCE:1:1:1:memorization (cross-axis: understanding helps recall)
```

### Actual Behavior

**Current Flow:**
```
1. System chooses: "WORD_INSTANCE:1:1:1" (base node)
2. Exercise: Generic word review (implementation-dependent)
3. User succeeds → base node energy increases (?)
4. Energy propagates:
   - Through edges from base node
   - Axis nodes never directly reviewed
```

**Problem:** No way to distinguish between testing memorization, translation, tajweed, etc.

## Cross-Axis Learning Synergies

### Designed Synergies

**Python graph encodes these relationships:**

```
Translation → Memorization
  "Understanding meaning helps you remember the word"

Tajweed → Memorization
  "Practicing pronunciation reinforces memory"

Word Memorization → Verse Memorization
  "Knowing words helps you know the verse"

Contextual Memorization (Word N) → Contextual Memorization (Word N-1)
  "Knowing word sequence helps recall"
```

**These are MODELED in the graph as directed edges with probabilistic distributions.**

### Current Rust Usage

**Energy propagation works, but:**
- ❌ No awareness of WHY edges exist (translation→memorization relationship not understood)
- ❌ No ability to target exercises to specific axes
- ❌ No tracking of axis-specific mastery levels

**Example:**
```rust
// Current: Generic propagation
for edge in edges {
    let energy = sample_distribution(edge.distribution);
    update_energy(edge.target_id, energy);
}

// Missing: Axis-aware logic
if edge.source.axis == KnowledgeAxis::Translation &&
   edge.target.axis == KnowledgeAxis::Memorization {
    // This is a cross-axis synergy edge
    // Could apply different logic or tracking
}
```

## Graph Design Understanding

### Q8 Sub-Questions

**1. Does code understand knowledge axis concept?**
- Python: ✅ Complete understanding
- Rust: ❌ Minimal understanding (knows "Knowledge" type exists, nothing more)

**2. Is design ignored?**
- Graph generation: ❌ Not ignored, correctly implemented
- Runtime: ⚠️ Partially ignored - nodes exist but never used

**3. Exercise design awareness?**
- Python: ✅ Node types clearly defined per axis
- Rust: ❌ Filters out Knowledge nodes from exercises

**4. Graph structure exploitation?**
- Directed edges: ⚠️ Used for propagation, but not semantically understood
- Word sequences: ❌ Not exploited (uses ID inference, not edges)
- Hierarchical structure: ⚠️ Implicitly used via edges, but not explicitly modeled

## Example: How Axis Design Should Work

### Scenario: User Learning Al-Fatihah Verse 1

**Nodes in Graph:**
```
VERSE:1:1
├─ VERSE:1:1:memorization
├─ VERSE:1:1:translation
└─ VERSE:1:1:tafsir

WORD_INSTANCE:1:1:1 (بِسْمِ)
├─ WORD_INSTANCE:1:1:1:memorization
├─ WORD_INSTANCE:1:1:1:translation
├─ WORD_INSTANCE:1:1:1:tajweed
└─ WORD_INSTANCE:1:1:1:contextual_memorization
```

**Exercise Types:**

| Exercise | Target Node | Question | Tests |
|----------|-------------|----------|-------|
| Recall | WORD_INSTANCE:1:1:1:memorization | "What is the first word?" | Memory |
| Translation | WORD_INSTANCE:1:1:1:translation | "What does 'بِسْمِ' mean?" | Understanding |
| Pronunciation | WORD_INSTANCE:1:1:1:tajweed | "Pronounce 'بِسْمِ'" (with audio) | Tajweed |
| Context | WORD_INSTANCE:1:1:1:contextual_memorization | "What word comes after 'بِسْمِ'?" | Sequence |

**Energy Propagation After "Recall" Exercise:**
```
User reviews: WORD_INSTANCE:1:1:1:memorization (success)
→ Energy increases for this node

Propagation:
→ VERSE:1:1:memorization (word contributes to verse memorization)
→ WORD_INSTANCE:1:1:1:translation (memorization slightly boosts translation)
→ WORD_INSTANCE:1:1:2:contextual_memorization (knowing word 1 helps word 2 context)
```

**Current Rust:** NONE of this axis-specific targeting exists.

## Recommendations

### Option 1: Implement Full Axis Support (High Effort)

**Changes Required:**

1. **Add KnowledgeAxis enum to Rust:**
```rust
pub enum KnowledgeAxis {
    Memorization,
    Translation,
    Tafsir,
    Tajweed,
    ContextualMemorization,
    Meaning,
}
```

2. **Parse axis from node ID:**
```rust
pub struct KnowledgeNode {
    pub base_node_id: String,  // "WORD_INSTANCE:1:1:1"
    pub axis: KnowledgeAxis,   // Memorization
}

impl KnowledgeNode {
    pub fn from_id(id: &str) -> Option<Self> {
        let parts: Vec<&str> = id.split(':').collect();
        let axis_str = parts.last()?;
        let axis = KnowledgeAxis::from_str(axis_str).ok()?;
        let base_node_id = parts[..parts.len()-1].join(":");
        Some(KnowledgeNode { base_node_id, axis })
    }

    pub fn full_id(&self) -> String {
        format!("{}:{}", self.base_node_id, self.axis.to_string())
    }
}
```

3. **Update session service to include Knowledge nodes:**
```rust
// Remove filter that excludes Knowledge type
// Add logic to target specific axes
```

4. **Implement axis-specific exercises:**
```rust
match node.axis {
    KnowledgeAxis::Memorization => MemorizationExercise::new(...),
    KnowledgeAxis::Translation => TranslationExercise::new(...),
    KnowledgeAxis::Tajweed => TajweedExercise::new(...),
    ...
}
```

### Option 2: Simplify Python Design (Low Effort)

**Changes:**
- Remove axis node generation from Python
- Only create base nodes
- Use node metadata or separate table to track "last tested for memorization" vs "last tested for translation"
- Keep cross-dimensional relationships in documentation, not in graph structure

**Trade-offs:**
- Simpler runtime
- Loses elegant graph-based modeling
- Harder to model cross-axis synergies

### Option 3: Partial Implementation (Medium Effort)

**Phase 1 (MVP):**
- Keep current system (ignore axis nodes)
- Only review base nodes
- Energy propagation continues to work

**Phase 2 (Future):**
- Add axis parsing to Rust
- Implement one axis (e.g., Memorization) for proof of concept
- Gradually add other axes

## File Locations

**Python Design:**
- Axis definitions: [graph/knowledge.py](../../research_and_dev/iqrah-knowledge-graph2/src/iqrah/graph/knowledge.py) (lines 1-316)
- Edge creation: [graph/knowledge_builder.py](../../research_and_dev/iqrah-knowledge-graph2/src/iqrah/graph/knowledge_builder.py) (lines 1-542)
- Identifiers: [graph/identifiers.py](../../research_and_dev/iqrah-knowledge-graph2/src/iqrah/graph/identifiers.py)

**Rust Gaps:**
- Domain model: [domain/models.rs](../../rust/crates/iqrah-core/src/domain/models.rs) - Missing axis enum
- Session service: [services/session_service.rs](../../rust/crates/iqrah-core/src/services/session_service.rs) - Filters out Knowledge nodes (line 89-92)
- Learning service: [services/learning_service.rs](../../rust/crates/iqrah-core/src/services/learning_service.rs) - Axis-agnostic propagation

---

**Navigation:** [← Rust Implementation](05-rust-implementation.md) | [Next: Navigation & Algorithms →](07-navigation-and-algorithms.md)

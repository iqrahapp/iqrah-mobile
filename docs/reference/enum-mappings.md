# Enum Mappings: INTEGER Values for Database Storage

**Date**: 2025-01-25
**Status**: Production Specification
**Purpose**: Defines canonical INTEGER mappings for all enums

---

## Critical Requirement

**ALL implementations (Rust, Python, SQL) MUST use these exact INTEGER values.**

Any mismatch will cause referential integrity violations and data corruption.

---

## NodeType Enum

**Definition**: Type of node in the knowledge graph

### Rust Implementation

**File**: `rust/crates/iqrah-core/src/domain/models.rs`

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, sqlx::Type)]
#[repr(i32)]
pub enum NodeType {
    Verse = 0,
    Chapter = 1,
    Word = 2,
    Knowledge = 3,
    WordInstance = 4,
}
```

### Python Implementation

**File**: `research_and_dev/iqrah-knowledge-graph2/src/iqrah/graph/builder.py`

```python
from enum import IntEnum

class NodeType(IntEnum):
    VERSE = 0
    CHAPTER = 1
    WORD = 2
    KNOWLEDGE = 3
    WORD_INSTANCE = 4
```

### SQL Examples

```sql
-- Query verses
SELECT * FROM nodes WHERE node_type = 0;

-- Query knowledge nodes
SELECT * FROM nodes WHERE node_type = 3;

-- Count by type
SELECT node_type, COUNT(*) FROM nodes GROUP BY node_type;
```

### Mapping Table

| Value | Rust Name      | Python Name      | Description                        |
|-------|----------------|------------------|------------------------------------|
| 0     | Verse          | VERSE            | Quranic verse node                 |
| 1     | Chapter        | CHAPTER          | Surah/chapter node                 |
| 2     | Word           | WORD             | Word definition (unused in MVP)    |
| 3     | Knowledge      | KNOWLEDGE        | Knowledge dimension node           |
| 4     | WordInstance   | WORD_INSTANCE    | Word occurrence in specific verse  |

---

## KnowledgeAxis Enum

**Definition**: Learning dimension applied to content nodes

### Rust Implementation

**File**: `rust/crates/iqrah-core/src/domain/models.rs`

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, sqlx::Type)]
#[repr(i32)]
pub enum KnowledgeAxis {
    Memorization = 0,
    Translation = 1,
    Tafsir = 2,
    Tajweed = 3,
    ContextualMemorization = 4,
    Meaning = 5,
}
```

### Python Implementation

**File**: `research_and_dev/iqrah-knowledge-graph2/src/iqrah/graph/builder.py`

```python
from enum import IntEnum

class KnowledgeAxis(IntEnum):
    MEMORIZATION = 0
    TRANSLATION = 1
    TAFSIR = 2
    TAJWEED = 3
    CONTEXTUAL_MEMORIZATION = 4
    MEANING = 5
```

### SQL Examples

```sql
-- Query all memorization nodes
SELECT n.ukey
FROM nodes n
JOIN knowledge_nodes kn ON n.id = kn.node_id
WHERE kn.axis = 0;

-- Count by axis
SELECT axis, COUNT(*) FROM knowledge_nodes GROUP BY axis;
```

### Mapping Table

| Value | Rust Name               | Python Name              | Applies To    | Description                           |
|-------|-------------------------|--------------------------|---------------|---------------------------------------|
| 0     | Memorization            | MEMORIZATION             | Verse         | Reciting verse from memory            |
| 1     | Translation             | TRANSLATION              | Verse         | Understanding verse meaning           |
| 2     | Tafsir                  | TAFSIR                   | Verse         | Deep contextual interpretation        |
| 3     | Tajweed                 | TAJWEED                  | Verse         | Correct pronunciation rules           |
| 4     | ContextualMemorization  | CONTEXTUAL_MEMORIZATION  | Word          | Memorizing word in verse context      |
| 5     | Meaning                 | MEANING                  | Word          | Understanding word translation        |

### Verse-Level Axes (4)
- 0: Memorization
- 1: Translation
- 2: Tafsir
- 3: Tajweed

### Word-Level Axes (2)
- 4: ContextualMemorization
- 5: Meaning

---

## EdgeType Enum

**Definition**: Type of relationship between nodes

### Rust Implementation

**File**: `rust/crates/iqrah-core/src/domain/models.rs`

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, sqlx::Type)]
#[repr(i32)]
pub enum EdgeType {
    Dependency = 0,
    Knowledge = 1,
}
```

### Python Implementation

**File**: `research_and_dev/iqrah-knowledge-graph2/src/iqrah/graph/builder.py`

```python
from enum import IntEnum

class EdgeType(IntEnum):
    DEPENDENCY = 0
    KNOWLEDGE = 1
```

### SQL Examples

```sql
-- Query dependency edges
SELECT * FROM edges WHERE edge_type = 0;

-- Query knowledge edges (within same verse, cross-axis)
SELECT * FROM edges WHERE edge_type = 1;
```

### Mapping Table

| Value | Rust Name   | Python Name  | Description                                      |
|-------|-------------|--------------|--------------------------------------------------|
| 0     | Dependency  | DEPENDENCY   | Sequential prerequisite (verse N → verse N+1)    |
| 1     | Knowledge   | KNOWLEDGE    | Cross-axis relationship (memorization → translation) |

### Edge Semantics

**Dependency (0)**:
- Sequential verse progression: `VERSE:1:1:memorization` → `VERSE:1:2:memorization`
- Word-to-verse contribution: `WORD_INSTANCE:1:1:1:contextual_memorization` → `VERSE:1:1:memorization`
- Represents prerequisite relationships

**Knowledge (1)**:
- Cross-axis within same content: `VERSE:1:1:memorization` → `VERSE:1:1:translation`
- Represents knowledge dimension relationships

---

## Conversion Methods

### Rust

```rust
impl NodeType {
    pub fn to_int(self) -> i32 {
        self as i32
    }

    pub fn from_int(val: i32) -> Result<Self, Error> {
        match val {
            0 => Ok(NodeType::Verse),
            1 => Ok(NodeType::Chapter),
            2 => Ok(NodeType::Word),
            3 => Ok(NodeType::Knowledge),
            4 => Ok(NodeType::WordInstance),
            _ => Err(Error::InvalidNodeType(val)),
        }
    }
}

impl KnowledgeAxis {
    pub fn to_int(self) -> i32 {
        self as i32
    }

    pub fn from_int(val: i32) -> Result<Self, Error> {
        match val {
            0 => Ok(KnowledgeAxis::Memorization),
            1 => Ok(KnowledgeAxis::Translation),
            2 => Ok(KnowledgeAxis::Tafsir),
            3 => Ok(KnowledgeAxis::Tajweed),
            4 => Ok(KnowledgeAxis::ContextualMemorization),
            5 => Ok(KnowledgeAxis::Meaning),
            _ => Err(Error::InvalidKnowledgeAxis(val)),
        }
    }
}

impl EdgeType {
    pub fn to_int(self) -> i32 {
        self as i32
    }

    pub fn from_int(val: i32) -> Result<Self, Error> {
        match val {
            0 => Ok(EdgeType::Dependency),
            1 => Ok(EdgeType::Knowledge),
            _ => Err(Error::InvalidEdgeType(val)),
        }
    }
}
```

### Python

```python
# IntEnum automatically provides integer conversion
node_type = NodeType.VERSE
assert node_type == 0
assert int(node_type) == 0

# Reverse lookup
node_type = NodeType(0)
assert node_type == NodeType.VERSE

# String representation
axis = KnowledgeAxis.MEMORIZATION
assert axis.name == 'MEMORIZATION'
assert axis.value == 0
```

---

## Validation

### SQL Constraints

```sql
-- NodeType constraint
ALTER TABLE nodes ADD CONSTRAINT check_node_type
    CHECK (node_type >= 0 AND node_type <= 4);

-- KnowledgeAxis constraint
ALTER TABLE knowledge_nodes ADD CONSTRAINT check_axis
    CHECK (axis >= 0 AND axis <= 5);

-- EdgeType constraint
ALTER TABLE edges ADD CONSTRAINT check_edge_type
    CHECK (edge_type >= 0 AND edge_type <= 1);
```

### Test Cases

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_type_conversion() {
        assert_eq!(NodeType::Verse.to_int(), 0);
        assert_eq!(NodeType::Knowledge.to_int(), 3);
        assert_eq!(NodeType::from_int(0).unwrap(), NodeType::Verse);
        assert!(NodeType::from_int(99).is_err());
    }

    #[test]
    fn test_knowledge_axis_conversion() {
        assert_eq!(KnowledgeAxis::Memorization.to_int(), 0);
        assert_eq!(KnowledgeAxis::Meaning.to_int(), 5);
        assert_eq!(KnowledgeAxis::from_int(3).unwrap(), KnowledgeAxis::Tajweed);
        assert!(KnowledgeAxis::from_int(99).is_err());
    }
}
```

---

## Extension Guidelines

### Adding New NodeType

**NEVER reuse existing values. Always append.**

```rust
// ❌ WRONG: Reusing value
pub enum NodeType {
    Verse = 0,
    Chapter = 1,
    NewType = 2,  // BREAKS: Word was 2
}

// ✅ CORRECT: Append new value
pub enum NodeType {
    Verse = 0,
    Chapter = 1,
    Word = 2,
    Knowledge = 3,
    WordInstance = 4,
    NewType = 5,  // Safe: New value
}
```

### Adding New KnowledgeAxis

```python
# ✅ CORRECT: Append after Meaning (5)
class KnowledgeAxis(IntEnum):
    MEMORIZATION = 0
    TRANSLATION = 1
    TAFSIR = 2
    TAJWEED = 3
    CONTEXTUAL_MEMORIZATION = 4
    MEANING = 5
    GRAMMAR = 6  # New axis
```

---

## Cross-Language Compatibility

### Rust → Python
```rust
// Rust serializes enum as integer
let node_type: i32 = NodeType::Knowledge.to_int();  // 3

// Python reads integer
node_type = NodeType(3)  // KNOWLEDGE
```

### Python → SQL
```python
# Python exports integer value
node_type = NodeType.VERSE.value  # 0

# SQL stores integer
INSERT INTO nodes (node_type) VALUES (0);
```

### SQL → Rust
```sql
-- SQL returns integer
SELECT node_type FROM nodes WHERE id = 1;  -- Returns: 3

-- Rust converts to enum
let node_type = NodeType::from_int(3)?;  // Knowledge
```

---

## Quick Reference Card

```
NodeType Values:
  0 = Verse
  1 = Chapter
  2 = Word
  3 = Knowledge
  4 = WordInstance

KnowledgeAxis Values:
  0 = Memorization (verse)
  1 = Translation (verse)
  2 = Tafsir (verse)
  3 = Tajweed (verse)
  4 = ContextualMemorization (word)
  5 = Meaning (word)

EdgeType Values:
  0 = Dependency
  1 = Knowledge
```

---

## References

- [Schema Design](../implementation/schema-design.md) - Database DDL
- [Rust Implementation Guide](../implementation/rust-implementation-guide.md) - Enum usage in Rust
- [Python Generator Guide](../implementation/python-generator-guide.md) - Enum usage in Python

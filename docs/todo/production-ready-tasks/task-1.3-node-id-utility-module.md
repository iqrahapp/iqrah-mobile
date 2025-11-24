# Task 1.3: Implement Node ID Utility Module

## Metadata
- **Priority:** P0 (Critical Foundation)
- **Estimated Effort:** 1 day
- **Dependencies:** None (but Task 1.1 architecture doc is helpful reference)
- **Agent Type:** Implementation
- **Parallelizable:** Yes (with tasks 1.1, 1.2, 1.5)

## Goal

Create a centralized `node_id` module with builder and parser functions to replace ad-hoc string manipulation throughout the codebase, improving type safety and preventing malformed node IDs.

## Context

Currently, node ID handling is scattered across the codebase with inconsistent string manipulation:

**Problems:**
- **Type safety:** `String` type doesn't distinguish between `"VERSE:1:1"` and `"WORD:123"`
- **Ad-hoc parsing:** String splitting logic duplicated in multiple places
- **Error-prone:** Easy to create malformed IDs like `"VERSE:1"` (missing verse number)
- **Hard to refactor:** Changing ID format requires finding all string manipulation code

**Example of Current Mess:**
```rust
// In repository.rs
let parts: Vec<&str> = node_id.split(':').collect();
if parts.len() == 2 && parts[0] == "VERSE" {
    // Parse verse...
}

// In session_service.rs
let node_id = format!("{}:{}", chapter, verse);  // Is this VERSE or something else?
```

This task creates a clean abstraction layer that will be used by Task 1.4 (repository refactoring).

## Current State

**Node ID Usage Locations:**
- `rust/crates/iqrah-storage/src/content/repository.rs` (lines 32-80) - Parsing in `get_node()`
- `rust/crates/iqrah-core/src/domain/models.rs` (lines 54-147) - `KnowledgeAxis` enum exists
- `rust/crates/iqrah-core/src/services/session_service.rs` - Node ID string construction
- Migration files - Hardcoded ID strings like `"1:1"`, `"VERSE:1:1:memorization"`

**No Centralized Module:** All ID handling is inline string operations.

## Target State

### New Module Structure

**File:** `rust/crates/iqrah-core/src/domain/node_id.rs`

```rust
// Builder functions (infallible)
pub fn chapter(num: u8) -> String
pub fn verse(chapter: u8, verse: u16) -> String
pub fn word(word_id: i64) -> String
pub fn word_instance(chapter: u8, verse: u16, position: u8) -> String
pub fn knowledge(base_id: &str, axis: KnowledgeAxis) -> String

// Parser functions (fallible)
pub fn parse_chapter(id: &str) -> Result<u8>
pub fn parse_verse(id: &str) -> Result<(u8, u16)>
pub fn parse_word(id: &str) -> Result<i64>
pub fn parse_word_instance(id: &str) -> Result<(u8, u16, u8)>
pub fn parse_knowledge(id: &str) -> Result<(String, KnowledgeAxis)>

// Type detection
pub fn node_type(id: &str) -> Result<NodeType>

// Validation
pub fn validate(id: &str) -> Result<()>
```

### Error Types

**File:** `rust/crates/iqrah-core/src/domain/error.rs` (or add to existing error.rs)

```rust
#[derive(Debug, thiserror::Error)]
pub enum NodeIdError {
    #[error("Invalid node ID format: {0}")]
    InvalidFormat(String),

    #[error("Invalid node type prefix: {0}")]
    InvalidPrefix(String),

    #[error("Invalid chapter number: {0} (must be 1-114)")]
    InvalidChapter(u8),

    #[error("Invalid verse number: {0} (must be >= 1)")]
    InvalidVerse(u16),

    #[error("Invalid knowledge axis: {0}")]
    InvalidAxis(String),

    #[error("Malformed node ID: {0}")]
    Malformed(String),
}
```

## Implementation Steps

### Step 1: Create Error Types (30 min)

**File:** `rust/crates/iqrah-core/src/domain/error.rs`

If this file doesn't exist, create it. If it exists, add `NodeIdError` enum as shown in Target State.

Make sure to add `thiserror` to Cargo.toml if not already present:
```toml
[dependencies]
thiserror = "1.0"
```

### Step 2: Create node_id Module Skeleton (30 min)

**File:** `rust/crates/iqrah-core/src/domain/node_id.rs`

```rust
use super::error::NodeIdError;
use super::models::KnowledgeAxis;
use super::models::NodeType;

pub type Result<T> = std::result::Result<T, NodeIdError>;

// TODO: Implement builder functions
// TODO: Implement parser functions
// TODO: Implement validation
```

**File:** `rust/crates/iqrah-core/src/domain/mod.rs`

Add:
```rust
pub mod error;
pub mod node_id;
pub mod models;  // Already exists
```

### Step 3: Implement Builder Functions (1 hour)

**In `node_id.rs`:**

```rust
/// Build a chapter node ID: "CHAPTER:1"
pub fn chapter(num: u8) -> String {
    debug_assert!((1..=114).contains(&num), "Chapter must be 1-114");
    format!("CHAPTER:{}", num)
}

/// Build a verse node ID: "VERSE:1:1"
pub fn verse(chapter: u8, verse: u16) -> String {
    debug_assert!((1..=114).contains(&chapter), "Chapter must be 1-114");
    debug_assert!(verse >= 1, "Verse must be >= 1");
    format!("VERSE:{}:{}", chapter, verse)
}

/// Build a word node ID: "WORD:123"
pub fn word(word_id: i64) -> String {
    debug_assert!(word_id > 0, "Word ID must be positive");
    format!("WORD:{}", word_id)
}

/// Build a word instance node ID: "WORD_INSTANCE:1:1:3"
pub fn word_instance(chapter: u8, verse: u16, position: u8) -> String {
    debug_assert!((1..=114).contains(&chapter), "Chapter must be 1-114");
    debug_assert!(verse >= 1, "Verse must be >= 1");
    debug_assert!(position >= 1, "Position must be >= 1");
    format!("WORD_INSTANCE:{}:{}:{}", chapter, verse, position)
}

/// Build a knowledge node ID: "VERSE:1:1:memorization"
pub fn knowledge(base_id: &str, axis: KnowledgeAxis) -> String {
    format!("{}:{}", base_id, axis.as_str())
}
```

**Note:** Also add `impl KnowledgeAxis` method if not already present:
```rust
impl KnowledgeAxis {
    pub fn as_str(&self) -> &'static str {
        match self {
            KnowledgeAxis::Memorization => "memorization",
            KnowledgeAxis::Translation => "translation",
            KnowledgeAxis::Tafsir => "tafsir",
            KnowledgeAxis::Tajweed => "tajweed",
            KnowledgeAxis::ContextualMemorization => "contextual_memorization",
            KnowledgeAxis::Meaning => "meaning",
        }
    }
}
```

### Step 4: Implement Parser Functions (2 hours)

**In `node_id.rs`:**

```rust
/// Parse a chapter ID: "CHAPTER:1" -> 1
pub fn parse_chapter(id: &str) -> Result<u8> {
    let parts: Vec<&str> = id.split(':').collect();

    if parts.len() != 2 || parts[0] != "CHAPTER" {
        return Err(NodeIdError::InvalidFormat(id.to_string()));
    }

    let num = parts[1]
        .parse::<u8>()
        .map_err(|_| NodeIdError::Malformed(id.to_string()))?;

    if !(1..=114).contains(&num) {
        return Err(NodeIdError::InvalidChapter(num));
    }

    Ok(num)
}

/// Parse a verse ID: "VERSE:1:1" -> (1, 1)
pub fn parse_verse(id: &str) -> Result<(u8, u16)> {
    let parts: Vec<&str> = id.split(':').collect();

    // Only accept prefixed format "VERSE:1:1"
    if parts.len() != 3 || parts[0] != "VERSE" {
        return Err(NodeIdError::InvalidFormat(id.to_string()));
    }

    let chapter_str = parts[1];
    let verse_str = parts[2];

    let chapter = chapter_str
        .parse::<u8>()
        .map_err(|_| NodeIdError::Malformed(id.to_string()))?;

    let verse = verse_str
        .parse::<u16>()
        .map_err(|_| NodeIdError::Malformed(id.to_string()))?;

    if !(1..=114).contains(&chapter) {
        return Err(NodeIdError::InvalidChapter(chapter));
    }

    if verse < 1 {
        return Err(NodeIdError::InvalidVerse(verse));
    }

    Ok((chapter, verse))
}

/// Parse word ID: "WORD:123" -> 123
pub fn parse_word(id: &str) -> Result<i64> {
    let parts: Vec<&str> = id.split(':').collect();

    if parts.len() != 2 || parts[0] != "WORD" {
        return Err(NodeIdError::InvalidFormat(id.to_string()));
    }

    parts[1]
        .parse::<i64>()
        .map_err(|_| NodeIdError::Malformed(id.to_string()))
}

/// Parse word instance: "WORD_INSTANCE:1:1:3" -> (1, 1, 3)
pub fn parse_word_instance(id: &str) -> Result<(u8, u16, u8)> {
    let parts: Vec<&str> = id.split(':').collect();

    if parts.len() != 4 || parts[0] != "WORD_INSTANCE" {
        return Err(NodeIdError::InvalidFormat(id.to_string()));
    }

    let chapter = parts[1]
        .parse::<u8>()
        .map_err(|_| NodeIdError::Malformed(id.to_string()))?;

    let verse = parts[2]
        .parse::<u16>()
        .map_err(|_| NodeIdError::Malformed(id.to_string()))?;

    let position = parts[3]
        .parse::<u8>()
        .map_err(|_| NodeIdError::Malformed(id.to_string()))?;

    if !(1..=114).contains(&chapter) {
        return Err(NodeIdError::InvalidChapter(chapter));
    }

    Ok((chapter, verse, position))
}

/// Parse knowledge node: "VERSE:1:1:memorization" -> ("VERSE:1:1", Memorization)
pub fn parse_knowledge(id: &str) -> Result<(String, KnowledgeAxis)> {
    let parts: Vec<&str> = id.split(':').collect();

    // Knowledge nodes have at least 3 parts: prefix:num:axis or prefix:n1:n2:axis
    if parts.len() < 3 {
        return Err(NodeIdError::InvalidFormat(id.to_string()));
    }

    // Last part is the axis
    let axis_str = parts.last().unwrap();
    let axis = KnowledgeAxis::from_str(axis_str)
        .map_err(|_| NodeIdError::InvalidAxis(axis_str.to_string()))?;

    // Everything before the last part is the base ID
    let base_id = parts[..parts.len()-1].join(":");

    Ok((base_id, axis))
}

/// Detect node type from ID string
pub fn node_type(id: &str) -> Result<NodeType> {
    let parts: Vec<&str> = id.split(':').collect();

    if parts.is_empty() {
        return Err(NodeIdError::InvalidFormat(id.to_string()));
    }

    // Check if it's a knowledge node (ends with axis)
    if let Some(last) = parts.last() {
        if KnowledgeAxis::from_str(last).is_ok() {
            return Ok(NodeType::Knowledge);
        }
    }

    // Check prefix
    match parts[0] {
        "CHAPTER" => Ok(NodeType::Chapter),
        "VERSE" => Ok(NodeType::Verse),
        "WORD" => Ok(NodeType::Word),
        "WORD_INSTANCE" => Ok(NodeType::WordInstance),
        _ => Err(NodeIdError::InvalidPrefix(parts[0].to_string())),
    }
}
```

**Note:** Add `KnowledgeAxis::from_str()` if not already present:
```rust
impl KnowledgeAxis {
    pub fn from_str(s: &str) -> std::result::Result<Self, ()> {
        match s {
            "memorization" => Ok(Self::Memorization),
            "translation" => Ok(Self::Translation),
            "tafsir" => Ok(Self::Tafsir),
            "tajweed" => Ok(Self::Tajweed),
            "contextual_memorization" => Ok(Self::ContextualMemorization),
            "meaning" => Ok(Self::Meaning),
            _ => Err(()),
        }
    }
}
```

### Step 5: Add Comprehensive Unit Tests (2 hours)

**File:** `rust/crates/iqrah-core/src/domain/node_id.rs` (at bottom of file)

```rust
#[cfg(test)]
mod tests {
    use super::*;

    // Builder tests
    #[test]
    fn test_build_chapter() {
        assert_eq!(chapter(1), "CHAPTER:1");
        assert_eq!(chapter(114), "CHAPTER:114");
    }

    #[test]
    fn test_build_verse() {
        assert_eq!(verse(1, 1), "VERSE:1:1");
        assert_eq!(verse(2, 286), "VERSE:2:286");
    }

    #[test]
    fn test_build_word() {
        assert_eq!(word(123), "WORD:123");
    }

    #[test]
    fn test_build_word_instance() {
        assert_eq!(word_instance(1, 1, 3), "WORD_INSTANCE:1:1:3");
    }

    #[test]
    fn test_build_knowledge() {
        assert_eq!(
            knowledge("VERSE:1:1", KnowledgeAxis::Memorization),
            "VERSE:1:1:memorization"
        );
    }

    // Parser tests
    #[test]
    fn test_parse_chapter() {
        assert_eq!(parse_chapter("CHAPTER:1").unwrap(), 1);
        assert_eq!(parse_chapter("CHAPTER:114").unwrap(), 114);
        assert!(parse_chapter("CHAPTER:115").is_err());
        assert!(parse_chapter("VERSE:1:1").is_err());
    }

    #[test]
    fn test_parse_verse() {
        assert_eq!(parse_verse("VERSE:1:1").unwrap(), (1, 1));
        assert_eq!(parse_verse("VERSE:2:286").unwrap(), (2, 286));
        assert!(parse_verse("VERSE:1").is_err());
        assert!(parse_verse("CHAPTER:1").is_err());
        assert!(parse_verse("1:1").is_err()); // Unprefixed format not supported
    }

    #[test]
    fn test_parse_word() {
        assert_eq!(parse_word("WORD:123").unwrap(), 123);
        assert!(parse_word("VERSE:1:1").is_err());
    }

    #[test]
    fn test_parse_word_instance() {
        assert_eq!(parse_word_instance("WORD_INSTANCE:1:1:3").unwrap(), (1, 1, 3));
        assert!(parse_word_instance("WORD:123").is_err());
    }

    #[test]
    fn test_parse_knowledge() {
        let (base, axis) = parse_knowledge("VERSE:1:1:memorization").unwrap();
        assert_eq!(base, "VERSE:1:1");
        assert!(matches!(axis, KnowledgeAxis::Memorization));

        let (base, axis) = parse_knowledge("WORD_INSTANCE:1:1:3:translation").unwrap();
        assert_eq!(base, "WORD_INSTANCE:1:1:3");
        assert!(matches!(axis, KnowledgeAxis::Translation));
    }

    #[test]
    fn test_node_type_detection() {
        assert!(matches!(node_type("CHAPTER:1").unwrap(), NodeType::Chapter));
        assert!(matches!(node_type("VERSE:1:1").unwrap(), NodeType::Verse));
        assert!(node_type("1:1").is_err()); // Unprefixed format not supported
        assert!(matches!(node_type("WORD:123").unwrap(), NodeType::Word));
        assert!(matches!(node_type("WORD_INSTANCE:1:1:3").unwrap(), NodeType::WordInstance));
        assert!(matches!(node_type("VERSE:1:1:memorization").unwrap(), NodeType::Knowledge));
    }

    #[test]
    fn test_roundtrip() {
        // Build then parse should return original values
        let chapter_id = chapter(5);
        assert_eq!(parse_chapter(&chapter_id).unwrap(), 5);

        let verse_id = verse(2, 255);
        assert_eq!(parse_verse(&verse_id).unwrap(), (2, 255));

        let word_id = word(999);
        assert_eq!(parse_word(&word_id).unwrap(), 999);
    }
}
```

### Step 6: Update Cargo.toml (if needed)

**File:** `rust/crates/iqrah-core/Cargo.toml`

Ensure dependencies include:
```toml
[dependencies]
thiserror = "1.0"
```

## Verification Plan

### Unit Tests

```bash
cd rust
cargo test node_id
```

- [ ] All builder tests pass (chapter, verse, word, word_instance, knowledge)
- [ ] All parser tests pass (parse_chapter, parse_verse, etc.)
- [ ] Node type detection tests pass
- [ ] Roundtrip tests pass (build → parse → original value)
- [ ] Error cases handled (invalid chapter, malformed ID, etc.)

### Integration with Existing Code (Smoke Test)

```bash
# This should still work (using old parsing logic)
cd rust
cargo test --package iqrah-storage
cargo test --package iqrah-core
```

All existing tests should still pass (we haven't refactored to use the new module yet).

### Manual Verification

```bash
# In Rust REPL or test:
use iqrah_core::domain::node_id;

let id = node_id::verse(1, 1);
assert_eq!(id, "VERSE:1:1");

let (ch, v) = node_id::parse_verse(&id).unwrap();
assert_eq!((ch, v), (1, 1));
```

## Scope Limits & Safeguards

### ✅ MUST DO

- Create `node_id.rs` module with builders and parsers
- Add comprehensive unit tests (15+ test cases)
- **Only support prefixed format** (`VERSE:1:1`, `CHAPTER:1`, etc.) - no backward compatibility
- Proper error types with clear messages
- Validate ranges (chapter 1-114, verse >= 1)

### ❌ DO NOT

- Refactor existing code to use this module (that's Task 1.4)
- Change existing node ID formats in migration files
- Modify `models.rs` extensively (only add `as_str()` and `from_str()` if missing)
- Add complex features like ID comparison, sorting, etc. (keep it simple)

### ⚠️ If Uncertain

- If `KnowledgeAxis` already has `as_str()` and `from_str()` → use them, don't duplicate
- If error types exist elsewhere → reuse them, don't create new ones unnecessarily
- If builder validation seems too strict → use `debug_assert!` not `assert!` (fail in debug, pass in release)
- **Node ID format:** Only prefixed format is supported (`VERSE:1:1`). Reject unprefixed (`1:1`) with clear error

## Success Criteria

- [ ] `rust/crates/iqrah-core/src/domain/node_id.rs` exists with ~300-400 lines
- [ ] All builder functions implemented (chapter, verse, word, word_instance, knowledge)
- [ ] All parser functions implemented (parse_chapter, parse_verse, etc.)
- [ ] `node_type()` detection function works
- [ ] `NodeIdError` enum with clear error messages
- [ ] 15+ unit tests covering happy paths and error cases
- [ ] `cargo test node_id` passes with 0 failures
- [ ] Existing tests still pass (smoke test)
- [ ] All CI checks pass (clippy, fmt, build)

## Related Files

**Create These Files:**
- `/rust/crates/iqrah-core/src/domain/node_id.rs` (new module)
- `/rust/crates/iqrah-core/src/domain/error.rs` (if doesn't exist, or add to existing)

**Modify These Files:**
- `/rust/crates/iqrah-core/src/domain/mod.rs` - Export node_id module
- `/rust/crates/iqrah-core/src/domain/models.rs` - Add `as_str()` and `from_str()` to KnowledgeAxis (if missing)
- `/rust/crates/iqrah-core/Cargo.toml` - Add thiserror dependency (if missing)

**Will Be Used By:**
- Task 1.4 (Repository refactoring) - Will replace string parsing with this module
- Task 2.1 (Knowledge graph generation) - Python will follow these ID formats
- Task 3.2 (Referential integrity) - Will validate IDs using this module

## Notes

**Design Principles:**
- **Infallible builders:** `chapter(1)` always returns valid ID (panics in debug if invalid input)
- **Fallible parsers:** `parse_chapter(id)` returns `Result` because input is untrusted
- **No backward compatibility:** Only prefixed format (`"VERSE:1:1"`) supported. Unprefixed format (`"1:1"`) is REJECTED with clear error

**Why Not Typed Node IDs?**
We considered `enum NodeId { Verse { chapter: u8, verse: u16 }, ... }` but:
- Harder to serialize/deserialize
- Requires migration of existing user data
- String-based IDs are fine for MVP
- Can refactor later if needed

This utility layer gives us 80% of type safety benefits with 20% of the complexity.

# Task 2.3: Implement Tajweed Exercise Type

## Metadata
- **Priority:** P0 (Core Feature)
- **Estimated Effort:** 2 days
- **Dependencies:** Task 2.1 (Knowledge graph with tajweed nodes)
- **Agent Type:** Implementation
- **Parallelizable:** Yes (with 2.2, 2.4 after 2.1 completes)

## Goal

Implement proper tajweed-specific exercise types that test pronunciation rules, stopping points, and recitation characteristics, replacing the current fallback to memorization exercises.

## Context

**Current State:**
- Tajweed axis nodes exist in database (from Task 2.1)
- Exercise service routes tajweed nodes to... memorization exercises (fallback)
- This is incomplete—tajweed needs its own exercise format

**Why Tajweed is Different:**
- **Memorization:** Can you recall the text?
- **Translation:** Do you understand the meaning?
- **Tajweed:** Can you pronounce it correctly according to Quranic recitation rules?

**Tajweed Concepts:**
- Lengthening (madd)
- Nasalization (ghunnah)
- Merged/clear pronunciation (idgham/izhar)
- Stopping points (waqf)
- Heavy/light letters (tafkheem/tarqeeq)

This task implements exercises that test these concepts.

## Current State

**File:** `rust/crates/iqrah-core/src/exercises/service.rs` (lines 84-146)

```rust
pub async fn generate_exercise(&self, node_id: &str) -> Result<Exercise> {
    let axis = self.detect_axis(node_id)?;

    match axis {
        KnowledgeAxis::Memorization | KnowledgeAxis::ContextualMemorization => {
            self.generate_memorization_exercise(node_id).await
        }
        KnowledgeAxis::Translation | KnowledgeAxis::Meaning | KnowledgeAxis::Tafsir => {
            self.generate_translation_exercise(node_id).await
        }
        KnowledgeAxis::Tajweed => {
            // TODO: Implement tajweed exercises
            self.generate_memorization_exercise(node_id).await  // FALLBACK!
        }
    }
}
```

**Exercise Types (Current):**
- `MemorizationExercise` - Implemented
- `TranslationExercise` - Implemented
- `TajweedExercise` - **Missing**

## Target State

### New Exercise Type

**File:** `rust/crates/iqrah-core/src/exercises/tajweed.rs` (NEW)

```rust
pub struct TajweedExercise {
    pub node_id: String,
    pub verse_text: String,
    pub focus_word: Option<String>,
    pub rule_type: TajweedRuleType,
    pub question: String,
    pub options: Vec<TajweedOption>,
    pub correct_answer: usize,
    pub explanation: String,
}

pub enum TajweedRuleType {
    Madd,           // Lengthening
    Ghunnah,        // Nasalization
    Idgham,         // Merged pronunciation
    Izhar,          // Clear pronunciation
    Qalqalah,       // Vibrating pronunciation
    Waqf,           // Stopping points
    Tafkheem,       // Heavy letters
    Tarqeeq,        // Light letters
}

pub struct TajweedOption {
    pub text: String,
    pub is_correct: bool,
}
```

### Exercise Examples

**Example 1: Madd (Lengthening)**
```
Verse: بِسْمِ ٱللَّهِ ٱلرَّحْمَٰنِ ٱلرَّحِيمِ
Focus: ٱللَّهِ
Question: "How long should the 'aa' sound be held in 'اللَّهِ'?"
Options:
  A) 1 count (short)
  B) 2 counts (natural)
  C) 4-6 counts (long)
Correct: B
Explanation: "This is a natural madd (مد طبيعي), held for 2 counts."
```

**Example 2: Waqf (Stopping)**
```
Verse: ٱلْحَمْدُ لِلَّهِ رَبِّ ٱلْعَٰلَمِينَ
Question: "Where is it recommended to stop in this verse?"
Options:
  A) After لِلَّهِ
  B) After ٱلْعَٰلَمِينَ
  C) No stopping allowed
Correct: B
Explanation: "The verse ends at ٱلْعَٰلَمِينَ with waqf lazim (must stop)."
```

**Example 3: Qalqalah (Vibration)**
```
Verse: قُلْ هُوَ ٱللَّهُ أَحَدٌ
Focus: قُلْ
Question: "How should the 'ق' in 'قُلْ' be pronounced?"
Options:
  A) Soft and smooth
  B) With a slight vibration/bounce
  C) Heavily emphasized
Correct: B
Explanation: "ق is a qalqalah letter, pronounced with a slight bounce."
```

## Implementation Steps

### Step 1: Define Tajweed Rule Data Structure (1 hour)

**File:** `rust/crates/iqrah-core/src/exercises/tajweed.rs` (NEW)

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TajweedExercise {
    pub node_id: String,
    pub verse_key: String,
    pub verse_text: String,
    pub focus_word: Option<String>,
    pub focus_position: Option<usize>,
    pub rule_type: TajweedRuleType,
    pub question: String,
    pub options: Vec<TajweedOption>,
    pub correct_answer_index: usize,
    pub explanation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TajweedRuleType {
    Madd,           // Lengthening rules
    Ghunnah,        // Nasalization
    Idgham,         // Merged pronunciation
    Izhar,          // Clear pronunciation
    Qalqalah,       // Vibrating pronunciation
    Waqf,           // Stopping points
    Tafkheem,       // Heavy letters (ر, ل, etc.)
    Tarqeeq,        // Light letters
    General,        // General pronunciation question
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TajweedOption {
    pub text: String,
    pub description: Option<String>,
}

impl TajweedExercise {
    pub fn verify_answer(&self, selected_index: usize) -> bool {
        selected_index == self.correct_answer_index
    }
}
```

### Step 2: Create Tajweed Rule Database (2-3 hours)

**File:** `rust/crates/iqrah-core/src/exercises/tajweed_rules.rs` (NEW)

For MVP, hardcode rules for common patterns:

```rust
use std::collections::HashMap;

pub struct TajweedRuleSet {
    rules: HashMap<String, Vec<TajweedRule>>,
}

pub struct TajweedRule {
    pub pattern: String,        // Arabic pattern (e.g., "ا", "ن", "ق")
    pub rule_type: TajweedRuleType,
    pub question_template: String,
    pub options: Vec<String>,
    pub correct_index: usize,
    pub explanation: String,
}

impl TajweedRuleSet {
    pub fn new() -> Self {
        let mut rules = HashMap::new();

        // Madd rules
        rules.insert("madd".to_string(), vec![
            TajweedRule {
                pattern: "ا".to_string(),
                rule_type: TajweedRuleType::Madd,
                question_template: "How long should the 'aa' sound be held?".to_string(),
                options: vec![
                    "1 count (short)".to_string(),
                    "2 counts (natural)".to_string(),
                    "4-6 counts (long)".to_string(),
                ],
                correct_index: 1,
                explanation: "Natural madd (مد طبيعي) is held for 2 counts.".to_string(),
            },
        ]);

        // Qalqalah rules
        rules.insert("qalqalah".to_string(), vec![
            TajweedRule {
                pattern: "ق".to_string(),
                rule_type: TajweedRuleType::Qalqalah,
                question_template: "How should this letter be pronounced?".to_string(),
                options: vec![
                    "Soft and smooth".to_string(),
                    "With a slight vibration/bounce".to_string(),
                    "Heavily emphasized".to_string(),
                ],
                correct_index: 1,
                explanation: "ق is a qalqalah letter, pronounced with a bounce.".to_string(),
            },
        ]);

        // Ghunnah rules
        rules.insert("ghunnah".to_string(), vec![
            TajweedRule {
                pattern: "ن".to_string(),
                rule_type: TajweedRuleType::Ghunnah,
                question_template: "Should nasalization (ghunnah) be applied?".to_string(),
                options: vec![
                    "Yes, full nasalization".to_string(),
                    "No nasalization".to_string(),
                    "Slight nasalization".to_string(),
                ],
                correct_index: 0,
                explanation: "ن with tanween receives full ghunnah (2 counts).".to_string(),
            },
        ]);

        // Waqf rules (stopping points)
        rules.insert("waqf".to_string(), vec![
            TajweedRule {
                pattern: "end_of_verse".to_string(),
                rule_type: TajweedRuleType::Waqf,
                question_template: "Where should you stop in this verse?".to_string(),
                options: vec![
                    "At the end of the verse".to_string(),
                    "In the middle".to_string(),
                    "No stopping allowed".to_string(),
                ],
                correct_index: 0,
                explanation: "End of verse is a natural stopping point.".to_string(),
            },
        ]);

        Self { rules }
    }

    pub fn get_rules_for_text(&self, text: &str) -> Vec<&TajweedRule> {
        let mut applicable_rules = vec![];

        // Check for qalqalah letters (ق، ط، ب، ج، د)
        if text.contains('ق') || text.contains('ط') || text.contains('ب') ||
           text.contains('ج') || text.contains('د') {
            if let Some(rules) = self.rules.get("qalqalah") {
                applicable_rules.extend(rules.iter());
            }
        }

        // Check for madd letters (ا، و، ي)
        if text.contains('ا') || text.contains('و') || text.contains('ي') {
            if let Some(rules) = self.rules.get("madd") {
                applicable_rules.extend(rules.iter());
            }
        }

        // Check for ghunnah letters (ن، م)
        if text.contains('ن') || text.contains('م') {
            if let Some(rules) = self.rules.get("ghunnah") {
                applicable_rules.extend(rules.iter());
            }
        }

        applicable_rules
    }
}
```

### Step 3: Implement Tajweed Exercise Generator (2-3 hours)

**File:** `rust/crates/iqrah-core/src/exercises/service.rs`

Update the `generate_exercise` method:

```rust
use crate::exercises::tajweed::{TajweedExercise, TajweedRuleSet};

pub struct ExerciseService {
    content_repo: Arc<dyn ContentRepository>,
    tajweed_rules: TajweedRuleSet,
}

impl ExerciseService {
    pub fn new(content_repo: Arc<dyn ContentRepository>) -> Self {
        Self {
            content_repo,
            tajweed_rules: TajweedRuleSet::new(),
        }
    }

    pub async fn generate_exercise(&self, node_id: &str) -> Result<Exercise> {
        let axis = self.detect_axis(node_id)?;

        match axis {
            KnowledgeAxis::Memorization | KnowledgeAxis::ContextualMemorization => {
                self.generate_memorization_exercise(node_id).await
            }
            KnowledgeAxis::Translation | KnowledgeAxis::Meaning | KnowledgeAxis::Tafsir => {
                self.generate_translation_exercise(node_id).await
            }
            KnowledgeAxis::Tajweed => {
                self.generate_tajweed_exercise(node_id).await
            }
        }
    }

    async fn generate_tajweed_exercise(&self, node_id: &str) -> Result<Exercise> {
        // Parse node ID to get verse key
        let (verse_key, _axis) = node_id::parse_knowledge(node_id)?;

        // Get verse text
        let verse = self.content_repo
            .get_verse(&verse_key)
            .await?
            .ok_or(ExerciseError::VerseNotFound(verse_key.clone()))?;

        // Find applicable tajweed rules
        let applicable_rules = self.tajweed_rules.get_rules_for_text(&verse.text);

        if applicable_rules.is_empty() {
            // Fallback: General pronunciation question
            return self.generate_general_tajweed_exercise(&verse_key, &verse.text).await;
        }

        // Select a random rule
        let rule = applicable_rules.choose(&mut rand::thread_rng())
            .ok_or(ExerciseError::NoApplicableRules)?;

        // Build exercise
        let exercise = TajweedExercise {
            node_id: node_id.to_string(),
            verse_key: verse_key.clone(),
            verse_text: verse.text.clone(),
            focus_word: None,  // TODO: Extract focus word
            focus_position: None,
            rule_type: rule.rule_type.clone(),
            question: rule.question_template.clone(),
            options: rule.options.iter()
                .map(|opt| TajweedOption {
                    text: opt.clone(),
                    description: None,
                })
                .collect(),
            correct_answer_index: rule.correct_index,
            explanation: rule.explanation.clone(),
        };

        Ok(Exercise::Tajweed(exercise))
    }

    async fn generate_general_tajweed_exercise(&self, verse_key: &str, text: &str) -> Result<Exercise> {
        // Fallback: Ask general recitation question
        let exercise = TajweedExercise {
            node_id: format!("{}:tajweed", verse_key),
            verse_key: verse_key.to_string(),
            verse_text: text.to_string(),
            focus_word: None,
            focus_position: None,
            rule_type: TajweedRuleType::General,
            question: "How should this verse be recited?".to_string(),
            options: vec![
                TajweedOption { text: "Quickly".to_string(), description: None },
                TajweedOption { text: "Slowly and clearly".to_string(), description: None },
                TajweedOption { text: "With a melodic tone".to_string(), description: None },
            ],
            correct_answer_index: 1,
            explanation: "Quranic recitation should be slow and clear (tarteel).".to_string(),
        };

        Ok(Exercise::Tajweed(exercise))
    }
}
```

### Step 4: Update Exercise Enum (30 min)

**File:** `rust/crates/iqrah-core/src/exercises/mod.rs`

```rust
pub mod memorization;
pub mod translation;
pub mod tajweed;  // NEW

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Exercise {
    Memorization(MemorizationExercise),
    Translation(TranslationExercise),
    Tajweed(TajweedExercise),  // NEW
}
```

### Step 5: Add Tests (2 hours)

**File:** `rust/crates/iqrah-core/tests/tajweed_exercise_test.rs` (NEW)

```rust
use iqrah_core::exercises::service::ExerciseService;
use iqrah_core::exercises::Exercise;

#[tokio::test]
async fn test_generate_tajweed_exercise() {
    let content_repo = setup_test_repo().await;
    let exercise_service = ExerciseService::new(content_repo);

    let exercise = exercise_service
        .generate_exercise("VERSE:1:1:tajweed")
        .await
        .unwrap();

    match exercise {
        Exercise::Tajweed(taj) => {
            // Note: verse_key is the database key (chapter:verse) without VERSE: prefix
            assert_eq!(taj.verse_key, "1:1");
            assert!(!taj.verse_text.is_empty());
            assert!(!taj.question.is_empty());
            assert!(taj.options.len() >= 2);
            assert!(taj.correct_answer_index < taj.options.len());
        }
        _ => panic!("Expected tajweed exercise"),
    }
}

#[tokio::test]
async fn test_tajweed_answer_verification() {
    let content_repo = setup_test_repo().await;
    let exercise_service = ExerciseService::new(content_repo);

    let exercise = exercise_service
        .generate_exercise("VERSE:1:1:tajweed")
        .await
        .unwrap();

    if let Exercise::Tajweed(taj) = exercise {
        // Correct answer should verify as true
        assert!(taj.verify_answer(taj.correct_answer_index));

        // Wrong answer should verify as false
        let wrong_index = (taj.correct_answer_index + 1) % taj.options.len();
        assert!(!taj.verify_answer(wrong_index));
    }
}
```

### Step 6: Add CLI Command for Testing (1 hour)

**File:** `rust/crates/iqrah-cli/src/commands/exercise.rs`

Add support for tajweed exercises:

```rust
async fn display_exercise(exercise: Exercise) {
    match exercise {
        Exercise::Tajweed(taj) => {
            println!("\n=== Tajweed Exercise ===");
            println!("Verse: {}", taj.verse_text);
            if let Some(word) = taj.focus_word {
                println!("Focus: {}", word);
            }
            println!("\nQuestion: {}", taj.question);
            for (i, option) in taj.options.iter().enumerate() {
                println!("  {}) {}", (i + 1), option.text);
            }
            println!("\nCorrect answer: {}", taj.correct_answer_index + 1);
            println!("Explanation: {}", taj.explanation);
        }
        // ... other exercise types
    }
}
```

## Verification Plan

### Unit Tests

```bash
cd rust
cargo test tajweed_exercise --nocapture
```

- [ ] `test_generate_tajweed_exercise` passes
- [ ] `test_tajweed_answer_verification` passes
- [ ] Tajweed exercises generated for multiple verses

### Integration Test

```bash
# Generate tajweed exercise via CLI
cargo run --bin iqrah-cli -- exercise --node-id "VERSE:1:1:tajweed"
```

- [ ] Exercise displays verse text
- [ ] Question is relevant to tajweed
- [ ] Options are provided (2+)
- [ ] Correct answer indicated
- [ ] Explanation included

### Manual Verification

Test with different verses:
```bash
cargo run --bin iqrah-cli -- exercise --node-id "VERSE:1:1:tajweed"
cargo run --bin iqrah-cli -- exercise --node-id "VERSE:1:7:tajweed"
cargo run --bin iqrah-cli -- exercise --node-id "VERSE:2:255:tajweed"
```

- [ ] Different rules applied to different verses
- [ ] Questions make sense
- [ ] Explanations are helpful

### Regression Check

```bash
cargo test --all-features
```

- [ ] All existing tests still pass
- [ ] No warnings or errors

## Scope Limits & Safeguards

### ✅ MUST DO

- Define TajweedExercise struct
- Create tajweed rule database (hardcoded for MVP)
- Implement tajweed exercise generator
- Update exercise service to route tajweed nodes correctly
- Add comprehensive tests
- Add CLI support for displaying tajweed exercises

### ❌ DO NOT

- Implement audio-based tajweed exercises (out of scope)
- Add advanced rule detection (use simple pattern matching)
- Build comprehensive tajweed rule database (just enough for MVP)
- Add UI for tajweed exercises (Rust/CLI only)
- Integrate with external tajweed APIs

### ⚠️ If Uncertain

- If rule database seems complex → Start with 3-5 simple rules
- If text analysis is hard → Use simple substring matching
- If generating questions is difficult → Use templates with placeholders
- If testing is complex → Focus on basic happy path tests

## Success Criteria

- [ ] `TajweedExercise` struct defined
- [ ] `TajweedRuleType` enum with 8+ rule types
- [ ] `TajweedRuleSet` with 5+ rules implemented
- [ ] Exercise service generates tajweed exercises (not fallback)
- [ ] Tests pass (3+ test cases)
- [ ] CLI displays tajweed exercises correctly
- [ ] No fallback to memorization exercises
- [ ] All CI checks pass (build, clippy, test, fmt)
- [ ] Documentation added (inline comments)

## Related Files

**Create These Files:**
- `/rust/crates/iqrah-core/src/exercises/tajweed.rs` - Tajweed exercise struct
- `/rust/crates/iqrah-core/src/exercises/tajweed_rules.rs` - Rule database
- `/rust/crates/iqrah-core/tests/tajweed_exercise_test.rs` - Tests

**Modify These Files:**
- `/rust/crates/iqrah-core/src/exercises/mod.rs` - Export tajweed module
- `/rust/crates/iqrah-core/src/exercises/service.rs` - Update exercise generation
- `/rust/crates/iqrah-cli/src/commands/exercise.rs` - Add CLI display support

## Notes

### MVP Approach

For MVP, we're implementing:
- **Simple rule database** (hardcoded patterns)
- **Template-based questions** (not AI-generated)
- **Text-only exercises** (no audio)

This is sufficient to validate the tajweed axis functionality.

### Future Enhancements

Post-MVP improvements:
- Audio-based exercises (listen and identify rules)
- Advanced rule detection using morphology data
- Comprehensive tajweed rule database
- Integration with tajweed scholars' content
- Real-time recitation feedback

### Arabic Text Handling

Ensure proper Unicode handling for Arabic text:
- Use `String` for Arabic text (UTF-8)
- Test with various verses (short and long)
- Handle diacritics (tashkeel) properly

### Rule Accuracy

**Important:** This implementation provides basic tajweed education. For production, consider:
- Reviewing rules with a qualified tajweed teacher
- Adding disclaimers about rule complexity
- Providing resources for deeper learning

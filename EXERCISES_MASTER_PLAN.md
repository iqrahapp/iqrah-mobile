# Iqrah Exercise System - Master Implementation Plan

**Date:** 2025-11-17
**Version:** 1.0
**Status:** Implementation Ready

---

## Table of Contents

1. [Exercise Catalog](#exercise-catalog)
2. [Architecture Overview](#architecture-overview)
3. [Implementation Priority](#implementation-priority)
4. [Detailed Exercise Specifications](#detailed-exercise-specifications)
5. [Testing Strategy](#testing-strategy)
6. [Implementation Tracking](#implementation-tracking)

---

## Exercise Catalog

This document preserves all exercises from the original design specification and provides a comprehensive implementation plan.

### Category 1: Memorization & Recall (Hifdh)

#### Difficulty Level 1: Recognition & Fluency (E: 0.0 - 0.3)

1. **Verse Reconstruction (Drag & Drop)** - STATELESS
   - Words of a verse presented out of order
   - User drags and drops into correct sequence
   - **Status:** Not implemented (requires UI component)

2. **Next Word MCQ** - STATELESS ✅ **PRIORITY: HIGH**
   - Verse shown with last word missing
   - User chooses correct word from 4 options
   - Difficulty progression via distractor selection
   - **Status:** Partially implemented (basic MCQ exists)

3. **Find the Missing Word (MCQ)** - STATELESS ✅ **PRIORITY: HIGH**
   - Word removed from middle of verse
   - User selects missing word from 4 choices
   - **Status:** Can be implemented with existing infrastructure

4. **Ayah Sequence (MCQ)** - STATELESS ✅ **PRIORITY: MEDIUM**
   - "Which verse comes next?" question
   - 3-4 options from same Surah
   - **Status:** Not implemented

#### Difficulty Level 2: Constrained Recall (E: 0.3 - 0.7)

5. **Echo Recall** - STATEFUL (Already implemented in codebase)
   - Progressive word obscuring based on energy
   - Time-based hint system
   - **Status:** ✅ Already implemented

6. **Cloze Deletion (Text Input)** - STATELESS ✅ **PRIORITY: HIGH**
   - Verse with blank space
   - User types missing Arabic word
   - Optional "show letters" hint
   - **Status:** Not implemented

7. **First Letter Hint Recall** - STATELESS ✅ **PRIORITY: HIGH**
   - Blank with first letter provided
   - Bridges recognition and recall
   - **Status:** Not implemented

8. **Reverse Cloze** - STATELESS ✅ **PRIORITY: MEDIUM**
   - Given a word, type the next word
   - Tests sequential memory
   - **Status:** Not implemented

#### Difficulty Level 3: Fluent Production (E: 0.7 - 1.0)

9. **Full Verse Input** - STATELESS ✅ **PRIORITY: MEDIUM**
   - Given verse number, type entire verse
   - **Status:** Not implemented

10. **Ayah Chain** - STATEFUL ✅ **PRIORITY: LOW**
    - Continuous verse typing flow
    - Continues until mistake or completion
    - **Status:** Not implemented

11. **Find the Mistake** - STATELESS ✅ **PRIORITY: LOW**
    - Verse with one subtle word substitution
    - User taps incorrect word
    - **CRITICAL:** Must ensure substitutions respect Qur'an
    - **Status:** Not implemented (requires careful validation)

---

### Category 2: Translation & Comprehension (Fahm)

#### Difficulty Level 1: Direct Meaning (E: 0.0 - 0.3)

12. **Translation Match (Arabic → English)** - STATELESS ✅ **PRIORITY: HIGH**
    - Arabic word shown
    - Select correct English translation from 4 options
    - **Status:** ✅ Already implemented (`McqExercise::new_ar_to_en`)

13. **Translation Match (English → Arabic)** - STATELESS ✅ **PRIORITY: HIGH**
    - English word shown
    - Select correct Arabic word from 4 options
    - **Status:** ✅ Already implemented (`McqExercise::new_en_to_ar`)

14. **Definition Match** - STATELESS ✅ **PRIORITY: MEDIUM**
    - Word (e.g., "Taqwa") with 4 possible definitions
    - Select most accurate definition
    - Can sample from similar KG nodes for difficulty
    - **Status:** Not implemented (requires definition database)

#### Difficulty Level 2: Contextual Understanding (E: 0.3 - 0.7)

15. **Contextual Translation (MCQ)** - STATELESS ✅ **PRIORITY: HIGH**
    - Verse with highlighted word
    - "In this context, what does [word] mean?"
    - Options show different nuances
    - **Status:** Not implemented

16. **Thematic Tagging** - STATELESS ⚠️ **BLOCKED**
    - Verse displayed, choose relevant theme
    - **Status:** Blocked - requires thematic question bank

17. **Opposites/Synonyms** - STATELESS ✅ **PRIORITY: MEDIUM**
    - Given word, choose opposite or synonym
    - **Status:** Not implemented (requires semantic relationships)

#### Difficulty Level 3: Deep Comprehension (E: 0.7 - 1.0)

18. **Tafsir Identification (MCQ)** - STATELESS ⚠️ **BLOCKED**
    - Match verse to correct Tafsir explanation
    - **Status:** Blocked - requires Tafsir data and AI-generated variations

19. **Cross-Verse Connection** - STATELESS ✅ **PRIORITY: HIGH**
    - Verse shown, select another verse with same subject
    - Ultimate graph-based exercise
    - **Status:** Not implemented (KG has the data!)

20. **Translate Phrase (Text Input)** - STATELESS ✅ **PRIORITY: MEDIUM**
    - Type English translation for Arabic phrase/verse
    - **Status:** Not implemented

---

### Category 3: Grammar & Etymology (Nahw & Sarf)

#### Difficulty Level 1: Identification (E: 0.0 - 0.4)

21. **Identify the Root (MCQ)** - STATELESS ✅ **PRIORITY: HIGH**
    - Word highlighted, select its 3-letter root
    - **Status:** Not implemented (DB has root data!)

22. **Part of Speech Tagging** - STATELESS ✅ **PRIORITY: MEDIUM**
    - Identify word as Noun/Verb/Particle
    - **Status:** Not implemented (morphology data available)

23. **Word Family Sort** - STATEFUL ✅ **PRIORITY: LOW**
    - Drag words into root family buckets
    - **Status:** Not implemented (requires UI component)

#### Difficulty Level 2: Application (E: 0.4 - 0.8)

24. **Grammatical Case (I'rāb) MCQ** - STATELESS ⚠️ **SKIP FOR NOW**
    - Identify case (Marfūʿ, Manṣūb, Majrūr)
    - **Status:** Skip - unclear benefit

25. **Verb Conjugation Challenge** - STATELESS ⚠️ **SKIP FOR NOW**
    - Given root and pronoun, type correct verb
    - **Status:** Skip - unclear benefit

26. **Sentence Structure Tree** - STATEFUL ⚠️ **SKIP FOR NOW**
    - Build dependency grammar tree
    - **Status:** Skip - requires complex UI

---

### Category 4: Recitation & Tajwīd (Tilāwah)

#### Difficulty Level 1: Knowledge (Text-Based) (E: 0.0 - 0.4)

27. **Identify the Rule (MCQ)** - STATELESS ✅ **PRIORITY: LOW**
    - Letters highlighted, select Tajwīd rule
    - **Status:** Not implemented (requires Tajwīd rule database)

28. **Rule Highlighting** - STATELESS ✅ **PRIORITY: LOW**
    - "Tap all instances of Qalqalah"
    - **Status:** Not implemented (requires Tajwīd annotations)

#### Difficulty Level 2 & 3: Application (Audio-Based) (E: 0.4 - 1.0)

29-31. **Audio-based exercises** ⛔ **IMPOSSIBLE FOR NOW**
    - Requires Iqrah-audio app (not developed yet)
    - **Status:** Blocked - no audio infrastructure

---

## Architecture Overview

### Current System (As Understood)

```
┌─────────────────────────────────────────────────────────────┐
│                     Flutter UI (Not Our Concern)             │
└─────────────────────────────────────────────────────────────┘
                              │
┌─────────────────────────────▼─────────────────────────────────┐
│                   iqrah-api (FFI Bridge)                       │
└─────────────────────────────────────────────────────────────┬─┘
                              │                                │
┌─────────────────────────────▼──────────┐   ┌────────────────▼──────┐
│         iqrah-core                      │   │   iqrah-server        │
│  - exercises/                           │   │   (HTTP API)          │
│    - service.rs ✅                      │   │                       │
│    - types.rs ✅                        │   │   For CLI Testing! ✓  │
│    - mcq.rs ✅ (Exercises 12, 13)      │   └───────────────────────┘
│    - memorization.rs ✅                 │              │
│    - translation.rs ✅                  │   ┌────────────────▼──────┐
│  - domain/                              │   │   iqrah-cli           │
│    - models.rs (KnowledgeAxis, etc.)    │   │   (Testing Tool) ✓    │
│  - services/                            │   └───────────────────────┘
│    - learning_service.rs                │
│    - energy_service.rs                  │
│  - semantic/ (grading)                  │
└─────────────────────────────────────────┘
                              │
┌─────────────────────────────▼─────────────────────────────────┐
│                   iqrah-storage                                │
│  - content/repository.rs (ContentRepository trait)             │
│  - Database Schema v2:                                         │
│    - chapters, verses, words ✓                                 │
│    - roots, lemmas, morphology_segments ✓                      │
│    - word_translations, verse_translations ✓                   │
│    - edges (Knowledge Graph) ✓                                 │
└────────────────────────────────────────────────────────────────┘
```

### Knowledge Graph

```
research_and_dev/iqrah-knowledge-graph2/
  - Python-based graph generation
  - Outputs: content.db + knowledge-graph.cbor.zst
  - Can generate for subset of chapters for fast iteration
  - Full Qur'an build: ~10-15 minutes
  - Small subset (chapters 1-10): ~2-3 minutes
```

### Testing Strategy

**Use CLI and Server for Testing:**
- iqrah-cli: Direct Rust testing
- iqrah-server: HTTP API for integration tests
- Both allow inspecting DB state and verifying exercise logic

**Test Data:**
- Can generate small KG (e.g., Al-Fatihah + Al-Baqarah)
- Faster iteration during development
- Full Qur'an graph available at: https://github.com/iqrahapp/iqrah-mobile/releases/tag/iqrah-graph-v1.1.0

---

## Implementation Priority

### 🔥 Wave 1: High-Impact, Easy Wins (Implement First)

**Memorization:**
- ✅ Exercise 2: Next Word MCQ
- ✅ Exercise 3: Find the Missing Word (MCQ)
- ✅ Exercise 6: Cloze Deletion (Text Input)
- ✅ Exercise 7: First Letter Hint Recall

**Translation:**
- ✅ Exercise 12: Already implemented
- ✅ Exercise 13: Already implemented
- ✅ Exercise 15: Contextual Translation (MCQ)
- ✅ Exercise 19: Cross-Verse Connection (Graph-based!)

**Grammar:**
- ✅ Exercise 21: Identify the Root (MCQ)

### 🚀 Wave 2: Medium Priority

**Memorization:**
- ✅ Exercise 4: Ayah Sequence (MCQ)
- ✅ Exercise 8: Reverse Cloze
- ✅ Exercise 9: Full Verse Input

**Translation:**
- ✅ Exercise 14: Definition Match
- ✅ Exercise 17: Opposites/Synonyms
- ✅ Exercise 20: Translate Phrase (Text Input)

**Grammar:**
- ✅ Exercise 22: Part of Speech Tagging

### 🌙 Wave 3: Low Priority / Complex

**Memorization:**
- Exercise 1: Verse Reconstruction (requires drag-drop UI)
- Exercise 10: Ayah Chain
- Exercise 11: Find the Mistake (requires careful validation)

**Grammar:**
- Exercise 23: Word Family Sort (requires UI)

**Tajwīd:**
- Exercise 27: Identify the Rule (requires Tajwīd DB)
- Exercise 28: Rule Highlighting (requires Tajwīd annotations)

### ⛔ Blocked / Skip

- Exercise 16: Thematic Tagging (no question bank)
- Exercise 18: Tafsir Identification (no Tafsir data)
- Exercises 24-26: Grammar advanced (unclear benefit)
- Exercises 29-31: Audio-based (no audio infrastructure)

---

## Detailed Exercise Specifications

### Exercise 2: Next Word MCQ

**Category:** Memorization - Level 1
**Type:** Stateless MCQ
**Knowledge Axis:** Memorization

**Implementation:**

```rust
// Location: rust/crates/iqrah-core/src/exercises/memorization.rs

pub struct NextWordMcqExercise {
    node_id: String,           // VERSE node ID
    verse_text: String,        // Full verse
    missing_word: String,      // Last word (correct answer)
    options: Vec<String>,      // 4 options (shuffled)
    difficulty: Difficulty,
}

impl NextWordMcqExercise {
    pub async fn new(
        verse_node_id: String,
        difficulty: Difficulty,
        content_repo: &dyn ContentRepository
    ) -> Result<Self> {
        // 1. Get verse and its words
        // 2. Extract last word (correct answer)
        // 3. Generate distractors based on difficulty:
        //    - Easy: random words from same verse
        //    - Medium: random words from same surah
        //    - Hard: phonetically/visually similar words from Qur'an
        // 4. Shuffle options
    }
}
```

**Database Queries Needed:**
- Get verse text by verse_key
- Get words for verse (to extract last word)
- Get random words from same chapter (for distractors)
- Optional: Get words by similarity (for hard mode)

**Tests (Minimum 3):**
1. `test_next_word_mcq_generates_correct_question` - Verify question format
2. `test_next_word_mcq_correct_answer` - Verify answer checking
3. `test_next_word_mcq_easy_distractors` - Verify easy distractors from same verse
4. `test_next_word_mcq_medium_distractors` - Verify medium distractors from same surah
5. `test_next_word_mcq_hard_distractors` - Verify hard distractors are similar words

---

### Exercise 3: Find the Missing Word (MCQ)

**Category:** Memorization - Level 1
**Type:** Stateless MCQ
**Knowledge Axis:** Memorization

**Implementation:**

```rust
pub struct MissingWordMcqExercise {
    node_id: String,           // WORD node ID (the missing word)
    verse_with_blank: String,  // "بِسْمِ _____ ٱلرَّحْمَٰنِ ٱلرَّحِيمِ"
    correct_answer: String,    // "ٱللَّهِ"
    options: Vec<String>,      // 4 options (shuffled)
    hint: Option<String>,      // First letter or length hint
}

impl MissingWordMcqExercise {
    pub async fn new(
        word_node_id: String,
        content_repo: &dyn ContentRepository
    ) -> Result<Self> {
        // 1. Get word and its verse context
        // 2. Build verse with blank ("_____" or "______")
        // 3. Generate distractors from same verse
        // 4. Shuffle options
    }
}
```

**Database Queries Needed:**
- Get word by word_id
- Get verse text and all words in verse
- Get word position in verse

**Tests (Minimum 3):**
1. `test_missing_word_generates_blank` - Verify blank placement
2. `test_missing_word_correct_answer` - Verify answer checking
3. `test_missing_word_distractors_from_same_verse` - Verify distractors
4. `test_missing_word_hint_first_letter` - Verify hint generation

---

### Exercise 6: Cloze Deletion (Text Input)

**Category:** Memorization - Level 2
**Type:** Stateless Text Input
**Knowledge Axis:** Memorization

**Implementation:**

```rust
pub struct ClozeDeletionExercise {
    node_id: String,           // WORD node ID
    verse_with_blank: String,  // "بِسْمِ _____ ٱلرَّحْمَٰنِ ٱلرَّحِيمِ"
    correct_answer: String,    // "ٱللَّهِ"
    hint_letters: Option<Vec<char>>, // Optional: jumbled letters
}

impl ClozeDeletionExercise {
    pub async fn new(
        word_node_id: String,
        show_letters_hint: bool,
        content_repo: &dyn ContentRepository
    ) -> Result<Self> {
        // 1. Get word and verse
        // 2. Build verse with blank
        // 3. If show_letters_hint, extract and jumble letters
    }

    fn normalize_arabic(text: &str) -> String {
        // Remove tashkeel, normalize for comparison
    }

    fn check_answer(&self, user_input: &str) -> bool {
        Self::normalize_arabic(user_input) ==
            Self::normalize_arabic(&self.correct_answer)
    }
}
```

**Database Queries Needed:**
- Same as Exercise 3

**Tests (Minimum 5):**
1. `test_cloze_deletion_basic` - Basic functionality
2. `test_cloze_deletion_normalization` - Arabic normalization (with/without tashkeel)
3. `test_cloze_deletion_with_hint_letters` - Verify jumbled letters hint
4. `test_cloze_deletion_wrong_answer` - Verify rejection
5. `test_cloze_deletion_partial_match` - Verify no partial credit

---

### Exercise 7: First Letter Hint Recall

**Category:** Memorization - Level 2
**Type:** Stateless Text Input
**Knowledge Axis:** Memorization

**Implementation:**

```rust
pub struct FirstLetterHintExercise {
    node_id: String,
    verse_with_hint: String,   // "بِسْمِ ٱـ_____ ٱلرَّحْمَٰنِ ٱلرَّحِيمِ"
    correct_answer: String,    // "ٱللَّهِ"
    first_letter: char,        // 'ٱ'
}
```

**Database Queries Needed:**
- Same as Exercise 3

**Tests (Minimum 3):**
1. `test_first_letter_hint_display` - Verify first letter shown
2. `test_first_letter_hint_correct` - Verify answer checking
3. `test_first_letter_hint_normalization` - Verify normalization

---

### Exercise 15: Contextual Translation (MCQ)

**Category:** Translation - Level 2
**Type:** Stateless MCQ
**Knowledge Axis:** Translation

**Implementation:**

```rust
pub struct ContextualTranslationExercise {
    node_id: String,           // WORD node ID
    verse_text: String,        // Full verse for context
    highlighted_word: String,  // Word to translate
    question: String,          // "In this context, what does 'أمة' mean?"
    correct_answer: String,    // "nation"
    options: Vec<String>,      // ["nation", "a period of time", "an Imam", "community"]
}

impl ContextualTranslationExercise {
    pub async fn new(
        word_node_id: String,
        content_repo: &dyn ContentRepository
    ) -> Result<Self> {
        // 1. Get word and verse
        // 2. Get primary translation (correct in this context)
        // 3. Get lemma/root to find alternative meanings
        // 4. Generate distractors (other valid meanings of same root/lemma)
        // 5. Shuffle options
    }
}
```

**Database Queries Needed:**
- Get word, verse, translation
- Get lemma for word (from morphology_segments)
- Get other words with same lemma (different contexts)
- Get translations for those words (alternative meanings)

**Tests (Minimum 4):**
1. `test_contextual_translation_basic` - Basic functionality
2. `test_contextual_translation_distractors_are_valid` - Distractors are actual meanings
3. `test_contextual_translation_correct_in_context` - Correct answer is contextually appropriate
4. `test_contextual_translation_multiple_verses` - Test across different verses

---

### Exercise 19: Cross-Verse Connection

**Category:** Translation - Level 3
**Type:** Stateless MCQ
**Knowledge Axis:** Translation/Meaning
**Leverage:** KNOWLEDGE GRAPH!

**Implementation:**

```rust
pub struct CrossVerseConnectionExercise {
    node_id: String,           // VERSE node ID
    verse_text: String,        // Current verse
    theme: String,             // E.g., "Tawhid", "Patience", "Charity"
    correct_verse: String,     // Another verse with same theme
    options: Vec<String>,      // 4 verse options (shuffled)
}

impl CrossVerseConnectionExercise {
    pub async fn new(
        verse_node_id: String,
        content_repo: &dyn ContentRepository
    ) -> Result<Self> {
        // 1. Get verse
        // 2. Query KG for connected verses:
        //    - Via shared roots/lemmas (grammar connection)
        //    - Via knowledge edges (semantic connection)
        // 3. Select a connected verse as correct answer
        // 4. Generate distractors (verses NOT connected)
        // 5. Shuffle options
    }
}
```

**Database Queries Needed:**
- Get verse
- **Query edges table:** Find verses connected via knowledge edges
- Get words in both verses to find shared roots/lemmas
- Get random verses for distractors

**KG Query Pattern:**
```sql
-- Find verses connected to current verse through knowledge edges
SELECT DISTINCT e2.target_id
FROM edges e1
JOIN edges e2 ON e1.target_id = e2.source_id
WHERE e1.source_id = :verse_node_id
  AND e2.target_id LIKE 'VERSE:%'
  AND e1.edge_type = 1  -- Knowledge edge
  AND e2.edge_type = 1;
```

**Tests (Minimum 5):**
1. `test_cross_verse_connection_finds_related` - Finds related verse
2. `test_cross_verse_connection_distractors_unrelated` - Distractors are unrelated
3. `test_cross_verse_connection_via_shared_root` - Connection via shared root
4. `test_cross_verse_connection_via_knowledge_edge` - Connection via KG edge
5. `test_cross_verse_connection_different_surahs` - Works across surahs

---

### Exercise 21: Identify the Root (MCQ)

**Category:** Grammar - Level 1
**Type:** Stateless MCQ
**Knowledge Axis:** Grammar (Not yet defined, use Memorization for now)

**Implementation:**

```rust
pub struct IdentifyRootExercise {
    node_id: String,           // WORD node ID
    word_text: String,         // "يَعْلَمُونَ"
    correct_root: String,      // "ع-ل-م"
    options: Vec<String>,      // ["ع-ل-م", "ك-ت-ب", "ذ-ه-ب", "ن-ز-ل"]
}

impl IdentifyRootExercise {
    pub async fn new(
        word_node_id: String,
        content_repo: &dyn ContentRepository
    ) -> Result<Self> {
        // 1. Get word
        // 2. Query morphology_segments to get root_id
        // 3. Get root arabic text
        // 4. Generate distractors (random roots, optionally similar)
        // 5. Shuffle options
    }
}
```

**Database Queries Needed:**
- Get word by word_id
- Get morphology_segments for word
- Get root from roots table
- Get random roots for distractors

**Tests (Minimum 4):**
1. `test_identify_root_correct_root` - Finds correct root
2. `test_identify_root_answer_checking` - Verifies answer
3. `test_identify_root_distractors_different` - Distractors are different roots
4. `test_identify_root_multiple_words` - Test across different words

---

## Testing Strategy

### Test Organization

```
rust/crates/iqrah-core/src/exercises/
  ├── memorization.rs         (implementations)
  ├── memorization_tests.rs   (unit tests)
  ├── translation.rs
  ├── translation_tests.rs
  ├── grammar.rs              (NEW)
  ├── grammar_tests.rs        (NEW)
  └── ...
```

### Test Data Setup

**Option 1: Small Test Database**
```bash
cd research_and_dev/iqrah-knowledge-graph2
iqrah build all \
  --morphology ../../research_and_dev/data/quranic-arabic-corpus-morphology.csv \
  --preset basic \
  --chapters 1-2 \  # Just Al-Fatihah + Al-Baqarah
  --content-db test-content.db \
  --output test-graph.cbor.zst
```

**Option 2: Use Sample Data**
- Schema already has sample data for Al-Fatihah (verse 1:1-1:7)
- Sufficient for basic testing

### Test Categories

1. **Unit Tests** (per exercise)
   - Test exercise generation
   - Test answer checking
   - Test hint generation
   - Test distractor quality

2. **Integration Tests** (via CLI/Server)
   - Test full exercise flow
   - Test database queries
   - Test KG traversal
   - Verify energy updates

3. **Validation Tests**
   - Ensure Arabic normalization works
   - Verify no offensive substitutions
   - Check distractor quality

### CI Validation Requirements

**MANDATORY Pre-Commit Checks (per CLAUDE.md):**

```bash
cd rust

# 1. Build with warnings as errors
RUSTFLAGS="-D warnings" cargo build --all-features

# 2. Clippy
cargo clippy --all-features --all-targets -- -D warnings

# 3. Tests
cargo test --all-features

# 4. Formatting
cargo fmt --all -- --check
```

---

## Implementation Tracking

### ✅ Already Implemented

- [x] Exercise 5: Echo Recall (stateful)
- [x] Exercise 12: Translation Match (Ar→En) - `McqExercise::new_ar_to_en`
- [x] Exercise 13: Translation Match (En→Ar) - `McqExercise::new_en_to_ar`

### 🔥 Wave 1: High Priority (Implement Next)

**Memorization:**
- [ ] Exercise 2: Next Word MCQ
- [ ] Exercise 3: Find the Missing Word (MCQ)
- [ ] Exercise 6: Cloze Deletion (Text Input)
- [ ] Exercise 7: First Letter Hint Recall

**Translation:**
- [ ] Exercise 15: Contextual Translation (MCQ)
- [ ] Exercise 19: Cross-Verse Connection (Graph-based!)

**Grammar:**
- [ ] Exercise 21: Identify the Root (MCQ)

### 🚀 Wave 2: Medium Priority

**Memorization:**
- [ ] Exercise 4: Ayah Sequence (MCQ)
- [ ] Exercise 8: Reverse Cloze
- [ ] Exercise 9: Full Verse Input

**Translation:**
- [ ] Exercise 14: Definition Match
- [ ] Exercise 17: Opposites/Synonyms
- [ ] Exercise 20: Translate Phrase (Text Input)

**Grammar:**
- [ ] Exercise 22: Part of Speech Tagging

### 🌙 Wave 3: Low Priority

- [ ] Exercise 1: Verse Reconstruction (requires UI)
- [ ] Exercise 10: Ayah Chain
- [ ] Exercise 11: Find the Mistake
- [ ] Exercise 23: Word Family Sort (requires UI)
- [ ] Exercise 27: Identify Tajwīd Rule
- [ ] Exercise 28: Rule Highlighting

### ⛔ Blocked / Skipped

- [ ] Exercise 16: Thematic Tagging (BLOCKED: no question bank)
- [ ] Exercise 18: Tafsir Identification (BLOCKED: no Tafsir data)
- [ ] Exercises 24-26: Advanced grammar (SKIP: unclear benefit)
- [ ] Exercises 29-31: Audio exercises (BLOCKED: no audio infrastructure)

---

## Implementation Notes

### Key Design Decisions

1. **Use existing infrastructure:** Leverage `ExerciseService`, `Exercise` trait, `ContentRepository`
2. **Follow existing patterns:** Model after `McqExercise` and `MemorizationExercise`
3. **Semantic grading:** Use existing semantic grading for text input exercises
4. **Knowledge Axis:** Map exercises to appropriate `KnowledgeAxis` enum values
5. **Testing:** Use CLI/Server for testing, not Flutter

### Arabic Text Normalization

**Critical for all memorization exercises:**

```rust
pub fn normalize_arabic(text: &str) -> String {
    text.chars()
        .filter(|c| !is_tashkeel(*c))
        .collect::<String>()
        .trim()
        .to_lowercase()
}

fn is_tashkeel(c: char) -> bool {
    matches!(c,
        '\u{064B}' | // Fathatan
        '\u{064C}' | // Dammatan
        '\u{064D}' | // Kasratan
        '\u{064E}' | // Fatha
        '\u{064F}' | // Damma
        '\u{0650}' | // Kasra
        '\u{0651}' | // Shadda
        '\u{0652}' | // Sukun
        '\u{0653}' | // Maddah
        '\u{0654}' | // Hamza above
        '\u{0655}' | // Hamza below
        '\u{0656}' | // Subscript alef
        '\u{0657}' | // Inverted damma
        '\u{0658}'   // Noon ghunna
    )
}
```

### Distractor Generation Strategy

**Progression from Easy to Hard:**

1. **Easy:** Random words from same verse
2. **Medium:** Random words from same surah
3. **Hard:** Phonetically/visually/semantically similar words from entire Qur'an

**Implement progressively:**
- Start with easy/medium (sufficient for MVP)
- Add hard mode later (requires similarity metrics)

### Graph-Based Exercises (Exercise 19)

**Leverage the KG structure:**

```
VERSE:1:1 ---[knowledge edge]---> WORD:1:1:1
                                   |
                                   +--[knowledge edge]---> LEMMA:allah
                                   |                          |
                                   |                          +--[knowledge edge]---> ROOT:alh
                                   |                                                     |
VERSE:112:1 <--[knowledge edge]----+<--[knowledge edge]--+<--[knowledge edge]----------+
```

**Query pattern:**
- Find verses sharing high-importance roots/lemmas
- Find verses connected via knowledge edges
- Rank by connection strength (edge weights)

---

## Success Metrics

### Exercise Quality

For each exercise implementation:
- ✅ 3-5 unit tests (minimum)
- ✅ Integration test via CLI
- ✅ CI passes (build, clippy, tests, fmt)
- ✅ Database queries optimized (use indexes)
- ✅ Answer checking properly handles edge cases
- ✅ Distractors are of appropriate difficulty

### Coverage

- 🎯 Wave 1: 7 exercises (high impact)
- 🎯 Wave 2: 7 exercises (medium impact)
- 🎯 Wave 3: 6 exercises (low priority)
- 🎯 Total: 20 exercises (plus 3 already implemented = 23 total)

### Performance

- Exercise generation: < 100ms
- Database queries: < 50ms
- Graph traversal (Exercise 19): < 200ms

---

## Next Steps

1. ✅ Create this master plan document
2. ✅ Mark todos as complete
3. 🔥 Start Wave 1 implementation
4. 🔥 Test each exercise with CLI
5. 🔥 Commit when CI passes
6. 🚀 Continue to Wave 2
7. 🌙 Optionally implement Wave 3

---

*End of Master Plan*

# 05 - Exercise System And Memorization Fit

## 1) Current Exercise Architecture

Core enum with many variants exists in:
- `rust/crates/iqrah-core/src/exercises/exercise_data.rs`

Generators for many exercise types exist in:
- `rust/crates/iqrah-core/src/exercises/generators.rs`

Routing service:
- `rust/crates/iqrah-core/src/exercises/service.rs`

API exposure:
- scheduled path: `get_exercises`, `start_session` flow
- broad preview path: `get_exercises_for_node` (sandbox oriented)

## 2) What Scheduled Sessions Mostly Use Today

`ExerciseService::generate_exercise_v2(...)` default routing is simple:
- word/word_instance -> `memorization`
- verse -> `echo_recall` (if memorization axis), else `full_verse_input`
- chapter -> `ayah_chain`

So although many exercise types exist, regular session flow uses a small subset.

## 3) What Exists But Is Not Core Scheduled Flow

Through `get_exercises_for_node(...)`, you can generate additional types such as:
- translation/contextual translation
- identify_root
- pos_tagging
- sequence_recall
- find_mistake
- cloze variants
- cross_verse_connection
- etc.

These are useful for experimentation, but not yet integrated into session policy as a principled curriculum.

## 4) Fit To Your Memorization Objective

Your stated target:
- practical memorization with limited time
- strong word/root recall
- prioritization of fragile/high-ROI items
- understanding as a memorization multiplier

Current fit:
- Good: the system has building blocks for word-level and understanding-oriented exercises.
- Weak: scheduled selection does not explicitly optimize for fragile words/roots or semantic bottlenecks.
- Weak: there is no explicit "most fragile lexical units" queue in main session planner.

## 5) High-Impact Missing Pieces

1. A first-class lexical fragility model
- track per-word and per-root error burden
- use this to inject high-value lexical drills each session

2. Explicit recall types for lexical meaning and root family transfer
- not only verse recall, but targeted minimal-pair and root-family recall

3. Weighted exercise policy by learning objective
- define session budget split (for example: 50 percent verse flow, 30 percent word recall, 20 percent root/meaning)
- adapt split by user weakness profile

4. Better intro path
- right now due-only retrieval can starve new users
- need controlled introduction queue tied to goals and lexical priority

## 6) Candid Evaluation

The exercise engine is not "too simple" overall. It is actually broad.
The problem is policy integration: advanced exercise inventory exists, but core session orchestration still behaves like a narrow review loop.

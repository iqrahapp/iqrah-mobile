# Phase 4: Five More Exercises

Document Version: 1.0
Date: 2024-12-28

## Purpose
Implement five additional exercise renderers so the app supports at least six total exercises (Echo Recall + five others). This phase also introduces two new backend exercise variants: SequenceRecall and FirstWordRecall.

## Goals
- Add backend variants for SequenceRecall and FirstWordRecall in `iqrah-core`.
- Expose new variants via `ExerciseDataDto` and FRB bindings.
- Implement five exercise renderers in Flutter (including SequenceRecall and FirstWordRecall).
- Add tests and debug screen coverage for each renderer.

## Dependencies
- Phase 1 (FFI foundation)
- Phase 3 (Echo Recall renderer) for shared UI patterns

## Acceptance Criteria
- At least five new exercise types render and submit results.
- All new exercises can be launched from the debug screen.
- Each exercise has at least one widget test.

## Backend Alignment Notes
This phase adds two new variants to the backend (`SequenceRecall`, `FirstWordRecall`) and surfaces them via FRB. This is a deliberate expansion beyond the existing `ExerciseDataDto` variants.

## Task Breakdown

### Task 4.1: Add Backend Variants (SequenceRecall, FirstWordRecall)
Add new variants to `iqrah-core` and the API DTO, then regenerate FRB bindings.

Files to modify:
- `rust/crates/iqrah-core/src/exercises/exercise_data.rs`
- `rust/crates/iqrah-core/src/exercises/generators.rs`
- `rust/crates/iqrah-api/src/api.rs`
- `lib/rust_bridge/api.dart` (regenerate via FRB)

Rust enum additions:
```rust
// rust/crates/iqrah-core/src/exercises/exercise_data.rs
pub enum ExerciseData {
    // ...
    SequenceRecall {
        node_id: i64,
        correct_sequence: Vec<i64>,
        options: Vec<Vec<i64>>,
    },
    FirstWordRecall {
        node_id: i64,
        verse_key: String,
    },
}
```

DTO additions:
```rust
// rust/crates/iqrah-api/src/api.rs
pub enum ExerciseDataDto {
    // ...
    SequenceRecall {
        node_id: String,
        correct_sequence: Vec<String>,
        options: Vec<Vec<String>>,
    },
    FirstWordRecall {
        node_id: String,
        verse_key: String,
    },
}
```

Generator stubs:
```rust
// rust/crates/iqrah-core/src/exercises/generators.rs
pub async fn generate_sequence_recall(
    node_id: i64,
    ukey: &str,
    content_repo: &dyn ContentRepository,
) -> Result<ExerciseData> {
    // Build a short sequence + distractor options.
    // Implementation must select deterministic verse ranges.
    todo!("SequenceRecall generator");
}

pub async fn generate_first_word_recall(
    node_id: i64,
    ukey: &str,
    content_repo: &dyn ContentRepository,
) -> Result<ExerciseData> {
    let verse_key = node_id::to_verse_key(ukey)?;
    Ok(ExerciseData::FirstWordRecall { node_id, verse_key })
}
```

FFI regeneration:
```bash
flutter_rust_bridge_codegen generate
```

### Task 4.2: SequenceRecall Exercise Renderer
Files to add:
- `lib/features/exercises/widgets/translation_widget.dart`

Widget skeleton:
```dart
class SequenceRecallWidget extends StatelessWidget {
  final String nodeId;
  final List<List<String>> options;
  final void Function(bool correct) onComplete;

  const SequenceRecallWidget({
    required this.nodeId,
    required this.options,
    required this.onComplete,
  });

  @override
  Widget build(BuildContext context) {
    // Render verse context + ordering options
    return Column(
      children: [
        Text('Sequence recall prompt'),
        // Options UI goes here
      ],
    );
  }
}
```

### Task 4.3: FirstWordRecall Exercise Renderer
Files to add:
- `lib/features/exercises/widgets/first_word_recall_widget.dart`

Key logic:
- Fetch verse text and mask first word.
- Accept Arabic input and evaluate.

### Task 4.4: Translation Exercise Renderer
Files to add:
- `lib/features/exercises/widgets/translation_widget.dart`

Widget skeleton:
```dart
class TranslationWidget extends StatelessWidget {
  final String nodeId;
  final void Function(bool correct) onComplete;

  const TranslationWidget({required this.nodeId, required this.onComplete});

  @override
  Widget build(BuildContext context) {
    return Column(
      children: [
        Text('Translation prompt goes here'),
        TextField(onSubmitted: (_) => onComplete(true)),
      ],
    );
  }
}
```

### Task 4.5: Cloze Deletion Renderer
Files to add:
- `lib/features/exercises/widgets/cloze_deletion_widget.dart`

Key logic:
- Use `getWordsForVerse()` and `blankPosition` from `ExerciseDataDto`.
- Render blanks as input fields.

### Task 4.6: First Letter Hint Renderer
Files to add:
- `lib/features/exercises/widgets/first_letter_hint_widget.dart`

Key logic:
- Fetch the target word, show only first letter, allow Arabic input.

### Task 4.7: Missing Word MCQ Renderer
Files to add:
- `lib/features/exercises/widgets/missing_word_mcq_widget.dart`

Key logic:
- Render verse with a blank, options include correct word + distractors.
- Resolve each distractor node ID to word text via WordInstance resolution.

### Task 4.8: Next Word MCQ Renderer
Files to add:
- `lib/features/exercises/widgets/next_word_mcq_widget.dart`

Key logic:
- Show context up to `contextPosition`, MCQ options for next word.

### Task 4.9: Wiring in ExerciseContainer
Update `lib/features/exercises/widgets/exercise_container.dart` to map the new variants to their renderers.

Dart example:
```dart
return widget.exercise.map(
  sequenceRecall: (e) => SequenceRecallWidget(
    nodeId: e.nodeId,
    options: e.options,
    onComplete: _handle,
  ),
  firstWordRecall: (e) => FirstWordRecallWidget(
    nodeId: e.nodeId,
    verseKey: e.verseKey,
    onComplete: _handle,
  ),
  translation: (e) => TranslationWidget(nodeId: e.nodeId, onComplete: _handle),
  clozeDeletion: (e) => ClozeDeletionWidget(...),
  firstLetterHint: (e) => FirstLetterHintWidget(...),
  missingWordMcq: (e) => MissingWordMcqWidget(...),
  nextWordMcq: (e) => NextWordMcqWidget(...),
  // ...
);
```

## Testing Requirements
- Widget tests for each renderer.
- Debug screen QA to ensure exercise launches.

Suggested commands:
```bash
flutter test test/exercises/translation_widget_test.dart
flutter test test/exercises/cloze_deletion_widget_test.dart
```

## Estimated Effort
- 9 to 12 days.

## Deliverables
- Backend variants for SequenceRecall and FirstWordRecall with FFI exposure.
- Five new exercise renderers including SequenceRecall and FirstWordRecall.
- Updated ExerciseContainer routing.
- Basic widget tests for each renderer.

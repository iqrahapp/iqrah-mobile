# Phase 5: Remaining Exercises

Document Version: 1.0
Date: 2024-12-28

## Purpose
Implement the remaining exercise renderers to reach full parity with `ExerciseDataDto` variants and stateful exercises already in the backend.

## Goals
- Complete renderers for all remaining exercise types.
- Ensure each type can be launched from debug and session flows.
- Document any backend data dependencies (morphology, graph edges).

## Dependencies
- Phase 4 (five more exercises completed)
- Phase 1 (WordInstance resolution)

## Acceptance Criteria
- Every `ExerciseDataDto` variant has a renderer.
- Each renderer is covered by a smoke-level widget test.
- Debug screen can launch every exercise variant.

## Task Breakdown

### Task 5.1: Full Verse Input
Renderer for `fullVerseInput`.

Files:
- `lib/features/exercises/widgets/full_verse_input_widget.dart`

Key logic:
- Fetch verse text and compare normalized Arabic input.

### Task 5.2: Ayah Chain (Stateful)
Renderer for `ayahChain` variant.

Files:
- `lib/features/exercises/widgets/ayah_chain_widget.dart`

Key logic:
- Track `currentIndex` and `completedCount` from `ExerciseDataDto`.
- Provide resume support if state is stored.

### Task 5.3: Find Mistake
Renderer for `findMistake`.

Files:
- `lib/features/exercises/widgets/find_mistake_widget.dart`

Key logic:
- Fetch verse words, inject the incorrect word, ask user to select the mistake.

### Task 5.4: Ayah Sequence
Renderer for `ayahSequence`.

Files:
- `lib/features/exercises/widgets/ayah_sequence_widget.dart`

Key logic:
- Provide draggable ordering or MCQ ordering, depending on backend data.

### Task 5.5: Identify Root
Renderer for `identifyRoot`.

Files:
- `lib/features/exercises/widgets/identify_root_widget.dart`

Key logic:
- Display word, provide root options, validate selection.
- Requires morphology data in content DB.

### Task 5.6: Reverse Cloze
Renderer for `reverseCloze`.

Files:
- `lib/features/exercises/widgets/reverse_cloze_widget.dart`

Key logic:
- Show translation, prompt for missing Arabic word.

### Task 5.7: Translate Phrase
Renderer for `translatePhrase`.

Files:
- `lib/features/exercises/widgets/translate_phrase_widget.dart`

Key logic:
- Fetch translation by translator ID and compare user input.

### Task 5.8: POS Tagging
Renderer for `posTagging`.

Files:
- `lib/features/exercises/widgets/pos_tagging_widget.dart`

Key logic:
- Multiple choice based on POS options from backend.
- Requires morphology data for quality options.

### Task 5.9: Cross Verse Connection
Renderer for `crossVerseConnection`.

Files:
- `lib/features/exercises/widgets/cross_verse_connection_widget.dart`

Key logic:
- Render a set of verse options with a theme label.
- Requires graph metadata from backend.

### Task 5.10: Contextual Translation
Renderer for `contextualTranslation`.

Files:
- `lib/features/exercises/widgets/contextual_translation_widget.dart`

Key logic:
- Show the word plus verse context and collect translation.

## Backend Data Dependencies
- `identifyRoot` and `posTagging` rely on morphology data in content DB.
- `crossVerseConnection` relies on graph edges and metadata.
- If data is missing, the renderer must show a clear fallback message.

## Testing Requirements
- One widget test per renderer.
- Debug screen checklist to confirm each variant can launch.

Suggested commands:
```bash
flutter test test/exercises/full_verse_input_widget_test.dart
flutter test test/exercises/identify_root_widget_test.dart
```

## Estimated Effort
- 10 to 15 days.

## Deliverables
- Full exercise coverage across all remaining variants.
- Widget tests and debug validation.

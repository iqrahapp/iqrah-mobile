# Phase 1: FFI Foundation

Document Version: 1.0
Date: 2024-12-28

## Purpose
Expose the minimum FFI surface needed for reliable content access and WordInstance resolution, then wrap it in a clean Flutter service layer with tests. This phase fixes the current WordInstance mismatch and makes content fetches deterministic.

## Goals
- Add or confirm FFI content access functions in `rust/crates/iqrah-api/src/api.rs`.
- Provide a deterministic way to resolve `WORD_INSTANCE` to real word text.
- Build a Flutter service layer that wraps FRB calls with error handling.
- Add an end-to-end test to validate the full data path.

## Dependencies
- Phase 0 completed.

## Acceptance Criteria
- Flutter can fetch a verse by key and display Arabic text.
- Flutter can resolve a word at chapter/verse/position with text.
- Translation lookup works for both verse and word contexts.
- FFI bindings are generated without errors.
- One integration test verifies the FFI data flow.

## Task Breakdown

### Task 1.1: Expand Content Access FFI
Add missing helper functions or confirm existing ones. The actual FFI entrypoint is `rust/crates/iqrah-api/src/api.rs`.

Files to modify:
- `rust/crates/iqrah-api/src/api.rs`
- `flutter_rust_bridge.yaml`
- `lib/rust_bridge/` (regenerate)

Functions to add (or confirm):
```rust
// rust/crates/iqrah-api/src/api.rs
pub async fn get_verse(verse_key: String) -> Result<Option<VerseDto>>;
pub async fn get_words_for_verse(verse_key: String) -> Result<Vec<WordDto>>;
pub async fn get_word(word_id: i32) -> Result<Option<WordDto>>;
pub async fn get_word_translation(word_id: i32, translator_id: i32) -> Result<Option<String>>;
pub async fn get_verse_translation_by_translator(
    verse_key: String,
    translator_id: i32,
) -> Result<Option<String>>;

// New helper to resolve WordInstance
pub async fn get_word_at_position(
    chapter: i32,
    verse: i32,
    position: i32,
) -> Result<Option<WordDto>>;
```

Implementation note for `get_word_at_position`:
- Use `content_repo.get_words_for_verse()` and find the matching `position`.

### Task 1.2: WordInstance Resolution Strategy
Provide a stable resolution strategy for `WORD_INSTANCE:*` IDs.

Option A (Rust helper):
- Add `get_word_at_position()` as above.

Option B (Dart helper):
- Parse the ukey in Flutter, call `get_words_for_verse()`, and match by position.

Recommended: Option A to keep logic in Rust and avoid repeated data parsing in Flutter.

Dart model (new):
```dart
// lib/models/word_instance.dart
class WordInstance {
  final int chapter;
  final int verse;
  final int position;
  final String textUthmani;
  final String? translation;

  const WordInstance({
    required this.chapter,
    required this.verse,
    required this.position,
    required this.textUthmani,
    this.translation,
  });
}
```

### Task 1.3: Service Layer Wrappers
Create explicit service wrappers to normalize errors and hide FRB details.

Files to create:
- `lib/services/content_service.dart`
- `lib/services/translation_service.dart`
- `lib/services/node_id_service.dart`

Example service skeleton:
```dart
// lib/services/content_service.dart
import 'package:iqrah/rust_bridge/api.dart' as api;

class ContentService {
  Future<api.VerseDto?> getVerse(String verseKey) {
    return api.getVerse(verseKey: verseKey);
  }

  Future<List<api.WordDto>> getWordsForVerse(String verseKey) {
    return api.getWordsForVerse(verseKey: verseKey);
  }

  Future<api.WordDto?> getWordAtPosition({
    required int chapter,
    required int verse,
    required int position,
  }) async {
    final word = await api.getWordAtPosition(
      chapter: chapter,
      verse: verse,
      position: position,
    );
    return word;
  }
}
```

### Task 1.4: Fix Translation Routing Heuristic
Replace the current `contains(':')` heuristic in `lib/services/exercise_content_service.dart` with explicit node type parsing.

Files to modify:
- `lib/services/exercise_content_service.dart`

Suggested helper:
```dart
// lib/services/node_id_service.dart
class NodeIdService {
  static bool isVerse(String ukey) => ukey.startsWith('VERSE:');
  static bool isWord(String ukey) => ukey.startsWith('WORD:');
  static bool isWordInstance(String ukey) => ukey.startsWith('WORD_INSTANCE:');
}
```

### Task 1.5: FFI Regeneration
Regenerate FRB bindings after Rust API changes.

Command:
```bash
flutter_rust_bridge_codegen generate
```

### Task 1.6: End-to-End Test
Add a single integration test that covers: get verse, get words, resolve word at position, fetch translation.

File to add:
- `integration_test/ffi_data_flow_test.dart`

Example test shape:
```dart
testWidgets('FFI data flow', (tester) async {
  final verse = await api.getVerse(verseKey: '1:1');
  expect(verse, isNotNull);

  final words = await api.getWordsForVerse(verseKey: '1:1');
  expect(words, isNotEmpty);

  final word = await api.getWordAtPosition(chapter: 1, verse: 1, position: 1);
  expect(word, isNotNull);
});
```

## Testing Requirements
- `flutter test integration_test/ffi_data_flow_test.dart`
- Verify no FRB codegen errors.

## Estimated Effort
- 4 to 6 days.

## Deliverables
- Updated FFI content access helpers.
- WordInstance resolution strategy implemented.
- Service layer wrappers in Flutter.
- A passing integration test.

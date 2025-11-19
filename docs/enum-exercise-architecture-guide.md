# Enum-Based Exercise Architecture Integration Guide

## Overview

This guide describes the integration of the modern enum-based exercise architecture into the iqrah-mobile app. The new system uses lightweight `ExerciseData` enums that store only keys/IDs, allowing Flutter to fetch content based on user preferences (text variants, translators, etc.).

## Architecture

### Key Components

1. **Rust Core** (`rust/crates/iqrah-core/src/exercises/`)
   - `exercise_data.rs` - 18 exercise type enums
   - `generators.rs` - Exercise generation functions
   - `validator.rs` - Answer validation with Arabic normalization
   - `service.rs` - `ExerciseService` with `generate_exercise_v2()` method

2. **FFI Bridge** (`rust/crates/iqrah-api/src/api.rs`)
   - `generate_exercise_v2()` - Exposed to Flutter via flutter_rust_bridge
   - Returns `ExerciseData` enum to Dart

3. **Flutter Service** (`lib/services/exercise_content_service.dart`)
   - `ExerciseContentService` - Fetches verse/word/translation content
   - Supports text variants: Uthmani, Simple, Indopak, Tajweed
   - Implements caching for performance

4. **Database Layer**
   - Batch query methods: `get_verses_batch()`, `get_words_batch()`
   - Optimized for fetching multiple items in a single query

## Text Variant System

### Supported Variants

| Variant | Description                        | Use Case               |
| ------- | ---------------------------------- | ---------------------- |
| Uthmani | Standard Uthmani script            | Default, most accurate |
| Simple  | No diacritics, simplified text     | Beginners, search      |
| Indopak | Indo-Pak script style              | Regional preference    |
| Tajweed | Color-coded tajweed rules (future) | Advanced learners      |

### Usage in Flutter

```dart
import 'package:iqrah/services/exercise_content_service.dart';

// Get user preferences
final prefs = ref.watch(userPreferencesProvider);

// Fetch verse content with selected variant
final service = ref.watch(exerciseContentServiceProvider);
final verseContent = await service.fetchVerseContent('1:1', prefs);

print(verseContent.text); // Text in user's preferred variant
```

## API Usage Examples

### Generating an Exercise (Rust)

```rust
use iqrah_core::exercises::{ExerciseService, ExerciseData};

let service = ExerciseService::new(content_repo);

// Generate exercise for a word
let exercise = service.generate_exercise_v2("WORD:1:1:1").await?;

match exercise {
    ExerciseData::Memorization { node_id } => {
        println!("Memorization exercise for {}", node_id);
    },
    _ => {}
}
```

### Generating an Exercise (Flutter)

```dart
import 'package:iqrah/rust_bridge/api.dart' as api;

// Generate exercise via FFI
final exerciseData = await api.generateExerciseV2(nodeId: 'WORD:1:1:1');

// exerciseData is now available as a Dart enum
// Fetch content based on user preferences
final service = ref.watch(exerciseContentServiceProvider);
final prefs = ref.watch(userPreferencesProvider);

// Fetch verse/word content with selected text variant
final content = await service.fetchVerseContent(verseKey, prefs);
```

### Batch Fetching for Performance

```dart
// Fetch multiple verses at once
final verseKeys = ['1:1', '1:2', '1:3'];
final versesMap = await service.fetchVersesBatch(verseKeys, prefs);

for (final verse in versesMap.values) {
  print('${verse.verseKey}: ${verse.text}');
}
```

## Exercise Types

### Complete List (18 Types)

1. **Memorization** - Recall exact Arabic text
2. **McqArToEn** - Multiple choice Arabic to English
3. **McqEnToAr** - Multiple choice English to Arabic
4. **Translation** - Type English translation
5. **ContextualTranslation** - Translation with verse context
6. **ClozeDeletion** - Fill in missing word
7. **FirstLetterHint** - Memorization with first letter hint
8. **MissingWordMcq** - MCQ for missing word
9. **NextWordMcq** - Predict next word
10. **FullVerseInput** - Type entire verse
11. **AyahChain** - Continuous verse typing
12. **AyahSequence** - Order verses correctly
13. **FindMistake** - Identify error in text
14. **ReverseCloze** - Given answer, find question
15. **CrossVerseConnection** - Link related verses
16. **TranslatePhrase** - Translate phrase/ayah
17. **IdentifyRoot** - Grammar: identify root word
18. **PosTagging** - Grammar: part of speech

### Exercise Data Structure

Each exercise variant stores only IDs/keys:

```rust
pub enum ExerciseData {
    Memorization {
        node_id: String,
    },
    McqArToEn {
        node_id: String,
        distractor_node_ids: Vec<String>,
    },
    FullVerseInput {
        node_id: String,
    },
    AyahChain {
        node_id: String,
    },
    // ... 14 more variants
}
```

## Migration Guide

### Phase 1: Coexistence (Current State)

Both old and new systems work in parallel:

```dart
// Old approach (still works)
final oldExercise = await getExercises(userId: userId, limit: 10);

// New approach (recommended)
final newExercise = await generateExerciseV2(nodeId: nodeId);
```

### Phase 2: Widget Migration (TODO)

Update exercise widgets to use `ExerciseContentService`:

**Before:**
```dart
Text(exercise.correctVerseText) // Direct text access
```

**After:**
```dart
FutureBuilder<VerseContent>(
  future: contentService.fetchVerseContent(
    exercise.verseKey,
    userPrefs,
  ),
  builder: (context, snapshot) {
    if (!snapshot.hasData) return CircularProgressIndicator();
    return Text(snapshot.data!.text);
  },
)
```

### Phase 3: Cleanup (Future)

After all widgets are migrated:
1. Remove old exercise generation code
2. Remove feature flags
3. Archive legacy exercise types
4. Update all documentation

## Performance Targets

| Operation                 | Target  | Notes                        |
| ------------------------- | ------- | ---------------------------- |
| Exercise Generation       | < 100ms | Rust-side generation         |
| Content Fetching (cached) | < 10ms  | In-memory cache hit          |
| Content Fetching (DB)     | < 50ms  | Single verse/word fetch      |
| Batch Fetch (10 items)    | < 100ms | Using batch query methods    |
| Total End-to-End          | < 200ms | Generation → Fetch → Display |

## Caching Strategy

The `ExerciseContentService` implements a simple in-memory cache:

- **Cache Key Format**: `{contentId}_{variant}` (e.g., `1:1_uthmani`)
- **Expiry**: 30 minutes
- **Clear Strategy**: Auto-clear on expiry or manual via `clearCache()`

### Cache Hits vs Misses

```dart
// First fetch - cache miss (50ms)
await service.fetchVerseContent('1:1', prefs);

// Second fetch - cache hit (< 10ms)
await service.fetchVerseContent('1:1', prefs);

// Different variant - cache miss (50ms)
final newPrefs = UserPreferences(textVariant: TextVariant.simple);
await service.fetchVerseContent('1:1', newPrefs);
```

## Testing

### Rust Integration Tests

Location: `rust/tests/exercise_v2_integration_tests.rs`

```bash
cd rust
cargo test --test exercise_v2_integration_tests
```

### Flutter Widget Tests (TODO)

Create mock `ExerciseContentService` for testing:

```dart
class MockExerciseContentService extends ExerciseContentService {
  @override
  Future<VerseContent> fetchVerseContent(
    String verseKey,
    UserPreferences prefs,
  ) async {
    return VerseContent(
      verseKey: verseKey,
      text: 'Mock verse text',
      variant: prefs.textVariant,
      chapterNumber: 1,
      verseNumber: 1,
    );
  }
}
```

## Next Steps

1. ✅ Rust integration complete
2. ✅ FFI bridge exposed
3. ✅ Flutter service created
4. ✅ Batch query methods implemented
5. 🚧 Widget migration (17 widgets to update)
6. 🚧 End-to-end testing
7. 📝 User preference UI for text variant selection
8. 📝 Translator selection UI
9. 📝 Offline content pre-fetching

## Troubleshooting

### Exercise Generation Fails

**Symptom**: `generate_exercise_v2()` returns error

**Possible Causes**:
- Invalid node_id format
- Database not initialized
- Missing content in database

**Solution**:
```rust
// Check node_id format
assert!(node_id.starts_with("WORD:") ||
        node_id.starts_with("VERSE:") ||
        node_id.starts_with("CHAPTER:"));
```

### Content Fetch Returns Empty

**Symptom**: `fetchVerseContent()` returns empty text

**Possible Causes**:
- Database not seeded
- Incorrect verse_key format
- FFI bridge not called

**Solution**: Verify database has content:
```sql
SELECT COUNT(*) FROM verses;
```

### Cache Not Working

**Symptom**: Every fetch goes to database

**Possible Causes**:
- Cache expiry too short
- Different `UserPreferences` instances

**Solution**: Check cache key consistency:
```dart
print('Cache key: ${verseKey}_${prefs.textVariant.name}');
```

## FAQ

**Q: Can I use multiple text variants in the same session?**
A: Yes! The cache supports different variants. Each variant is cached separately.

**Q: How do I add a new exercise type?**
A: Add a new variant to `ExerciseData` enum, create a generator function, add routing logic in `generate_exercise_v2()`, then run `flutter_rust_bridge_codegen generate`.

**Q: Should exercises be cached?**
A: Exercise structures (IDs/keys) are lightweight and regenerated on-demand. Cache the content (verses/words) instead.

**Q: How to implement offline mode?**
A: Pre-fetch content for upcoming exercises using `fetchVersesBatch()` and store in persistent cache (e.g., Hive, SharedPreferences).

## Contact

For questions or issues:
- Check existing tests: `rust/crates/iqrah-core/src/exercises/enum_tests.rs`
- Review TODO doc: `docs/todo/enum-architecture-integration.md`
- Submit GitHub issue with `[Exercise V2]` tag

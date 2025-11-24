# Task 4.1: Translator Selection UI & Preference

## Metadata
- **Priority:** P1 (Package Management Foundation)
- **Estimated Effort:** 3 days
- **Dependencies:** None
- **Agent Type:** Implementation (Rust + Flutter)
- **Parallelizable:** Yes (with 4.2)

## Goal

Implement translator selection functionality allowing users to choose their preferred translation, storing the preference and updating verse displays accordingly.

## Context

**Current State:**
- content.db has 5 translators (Sahih International, Pickthall, etc.)
- verse_translations table has multiple translations per verse
- No preference system—hardcoded to translator_id=1
- No UI to change translator

**Target:**
- User can select translator from list
- Preference stored in user.db
- Verses display using selected translator
- Defaults to Sahih International (id=1)

## Implementation Steps

### Step 1: Add Preference to User DB (1 hour)

**Migration:** `rust/crates/iqrah-storage/migrations_user/20241124000002_add_translator_preference.sql`

```sql
-- Add translator preference to app_settings
INSERT OR REPLACE INTO app_settings (key, value)
VALUES ('preferred_translator_id', '1');  -- Default: Sahih International
```

### Step 2: Update UserRepository (1 hour)

**File:** `rust/crates/iqrah-storage/src/user/repository.rs`

```rust
pub async fn get_preferred_translator(&self, user_id: &str) -> Result<i32> {
    let translator_id: Option<i32> = sqlx::query_scalar(
        "SELECT value FROM app_settings WHERE key = 'preferred_translator_id'"
    )
    .fetch_optional(&self.pool)
    .await?;

    Ok(translator_id.unwrap_or(1))  // Default: Sahih International
}

pub async fn set_preferred_translator(&self, user_id: &str, translator_id: i32) -> Result<()> {
    sqlx::query(
        "INSERT OR REPLACE INTO app_settings (key, value) VALUES ('preferred_translator_id', ?)"
    )
    .bind(translator_id)
    .execute(&self.pool)
    .await?;

    Ok(())
}
```

### Step 3: Update Verse Retrieval Logic (1 hour)

**File:** `rust/crates/iqrah-core/src/services/content_service.rs`

```rust
pub async fn get_verse_with_translation(&self, user_id: &str, verse_key: &str) -> Result<VerseWithTranslation> {
    // Get user's preferred translator
    let translator_id = self.user_repo.get_preferred_translator(user_id).await?;

    // Get verse
    let verse = self.content_repo.get_verse(verse_key).await?
        .ok_or(ContentError::VerseNotFound)?;

    // Get translation
    let translation = self.content_repo
        .get_translation(verse_key, translator_id)
        .await?;

    Ok(VerseWithTranslation {
        verse,
        translation,
        translator_id,
    })
}
```

### Step 4: Add FFI API (1 hour)

**File:** `rust/crates/iqrah-api/src/api.rs`

```rust
#[flutter_rust_bridge::frb(sync)]
pub async fn get_translators() -> Result<Vec<Translator>> {
    let content_repo = get_content_repo()?;
    content_repo.get_all_translators().await
}

#[flutter_rust_bridge::frb(sync)]
pub async fn get_preferred_translator(user_id: String) -> Result<i32> {
    let user_repo = get_user_repo()?;
    user_repo.get_preferred_translator(&user_id).await
}

#[flutter_rust_bridge::frb(sync)]
pub async fn set_preferred_translator(user_id: String, translator_id: i32) -> Result<()> {
    let user_repo = get_user_repo()?;
    user_repo.set_preferred_translator(&user_id, translator_id).await
}
```

Regenerate bridge:
```bash
flutter_rust_bridge_codegen generate
```

### Step 5: Create Flutter UI (1 day)

**File:** `lib/screens/settings/translator_selection_screen.dart` (NEW)

```dart
class TranslatorSelectionScreen extends StatefulWidget {
  @override
  _TranslatorSelectionScreenState createState() => _TranslatorSelectionScreenState();
}

class _TranslatorSelectionScreenState extends State<TranslatorSelectionScreen> {
  List<Translator> _translators = [];
  int? _selectedTranslatorId;
  bool _loading = true;

  @override
  void initState() {
    super.initState();
    _loadTranslators();
  }

  Future<void> _loadTranslators() async {
    final translators = await api.getTranslators();
    final preferred = await api.getPreferredTranslator(userId: 'default');

    setState(() {
      _translators = translators;
      _selectedTranslatorId = preferred;
      _loading = false;
    });
  }

  Future<void> _selectTranslator(int translatorId) async {
    await api.setPreferredTranslator(userId: 'default', translatorId: translatorId);

    setState(() {
      _selectedTranslatorId = translatorId;
    });

    ScaffoldMessenger.of(context).showSnackBar(
      SnackBar(content: Text('Translator updated')),
    );

    Navigator.pop(context);
  }

  @override
  Widget build(BuildContext context) {
    if (_loading) {
      return Scaffold(
        appBar: AppBar(title: Text('Select Translator')),
        body: Center(child: CircularProgressIndicator()),
      );
    }

    return Scaffold(
      appBar: AppBar(title: Text('Select Translator')),
      body: ListView.builder(
        itemCount: _translators.length,
        itemBuilder: (context, index) {
          final translator = _translators[index];
          final isSelected = translator.id == _selectedTranslatorId;

          return ListTile(
            title: Text(translator.name),
            subtitle: Text(translator.language),
            trailing: isSelected ? Icon(Icons.check, color: Colors.green) : null,
            onTap: () => _selectTranslator(translator.id),
          );
        },
      ),
    );
  }
}
```

### Step 6: Add to Settings Screen (30 min)

**File:** `lib/screens/settings/settings_screen.dart`

```dart
ListTile(
  leading: Icon(Icons.translate),
  title: Text('Translation'),
  subtitle: Text('Change Quran translation'),
  onTap: () {
    Navigator.push(
      context,
      MaterialPageRoute(builder: (context) => TranslatorSelectionScreen()),
    );
  },
),
```

### Step 7: Update Verse Display (1 hour)

**File:** `lib/widgets/verse_card.dart`

Ensure verse display uses preferred translator:

```dart
Future<VerseWithTranslation> _loadVerse(String verseKey) async {
  return await api.getVerseWithTranslation(
    userId: 'default',
    verseKey: verseKey,
  );
}
```

## Verification Plan

### Rust Tests

```bash
cd rust
cargo test translator_preference
```

- [ ] Get/set preference works
- [ ] Default is translator_id=1
- [ ] Translation retrieved with correct translator

### Flutter Integration Test

```bash
flutter test integration_test/translator_selection_test.dart
```

- [ ] Translator list loads
- [ ] Selection updates preference
- [ ] Verse display shows correct translation
- [ ] UI reflects current selection

### Manual Testing

1. Open app → Settings → Translation
2. See list of 5 translators
3. Select "Pickthall"
4. Return to verse view
5. Verify translation changed
6. Restart app
7. Verify preference persisted

## Success Criteria

- [ ] Migration adds preference setting
- [ ] Rust API for get/set translator
- [ ] FFI bridge updated
- [ ] Flutter UI screen created
- [ ] Settings screen links to translator selection
- [ ] Verse display uses preferred translator
- [ ] Preference persists across restarts
- [ ] Tests pass
- [ ] Manual testing successful

## Related Files

**Create:**
- `/rust/crates/iqrah-storage/migrations_user/20241124000002_add_translator_preference.sql`
- `/lib/screens/settings/translator_selection_screen.dart`

**Modify:**
- `/rust/crates/iqrah-storage/src/user/repository.rs`
- `/rust/crates/iqrah-core/src/services/content_service.rs`
- `/rust/crates/iqrah-api/src/api.rs`
- `/lib/screens/settings/settings_screen.dart`
- `/lib/widgets/verse_card.dart`

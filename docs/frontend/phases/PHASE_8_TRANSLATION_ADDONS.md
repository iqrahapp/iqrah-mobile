# Phase 8: Translation Addons

Document Version: 1.0
Date: 2024-12-28

## Purpose
Provide a translation pack workflow and user-facing selection UI so exercises can use preferred translations consistently.

## Goals
- List available and installed translation packs.
- Install and enable translation packs.
- Allow the user to select a preferred translator.

## Dependencies
- Phase 1 (FFI foundation)
- Phase 7 (polish and error handling)

## Acceptance Criteria
- Translation packs are listable and installable.
- User can select a preferred translator.
- Exercises show the preferred translation.

## Repo Alignment Note
This repo already has translator and translation tables in `content.db`, and FFI functions for `get_translators_for_language`, `get_translator`, `get_preferred_translator_id`, and `set_preferred_translator_id`.

Missing pieces are pack installation and listing of available packages.

## Task Breakdown

### Task 8.1: Package Listing FFI
Expose package listing and installed packages via FFI.

Files to modify:
- `rust/crates/iqrah-api/src/api.rs`

Rust signatures:
```rust
pub async fn get_available_packages(
    package_type: Option<String>,
    language_code: Option<String>,
) -> Result<Vec<ContentPackageDto>>;

pub async fn get_installed_packages() -> Result<Vec<InstalledPackageDto>>;

pub async fn enable_package(package_id: String) -> Result<String>;

pub async fn disable_package(package_id: String) -> Result<String>;
```

### Task 8.2: Pack Install Flow
Implement a pack install path in Rust and a downloader in Flutter.

Files to modify:
- `rust/crates/iqrah-api/src/api.rs`
- `lib/features/translation/translation_download_screen.dart`

Suggested FFI:
```rust
pub async fn install_translation_pack_from_bytes(
    package_id: String,
    bytes: Vec<u8>,
) -> Result<String>;
```

### Task 8.3: Translation Selector UI
Provide a UI to choose the preferred translator.

Files to add:
- `lib/features/translation/translation_selector_screen.dart`

Dart skeleton:
```dart
class TranslationSelectorScreen extends StatelessWidget {
  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: const Text('Translations')),
      body: FutureBuilder(
        future: api.getTranslatorsForLanguage(languageCode: 'en'),
        builder: (context, snapshot) {
          // Render list, call setPreferredTranslatorId on select
          return Container();
        },
      ),
    );
  }
}
```

### Task 8.4: Exercise Integration
Ensure translation fetches respect the preferred translator.

Files to modify:
- `lib/services/exercise_content_service.dart`
- `lib/providers/due_items_provider.dart`

## Testing Requirements
- Verify pack list -> install -> enable -> translation fetch flow.
- Widget test for translation selector screen.

## Estimated Effort
- 5 to 7 days.

## Deliverables
- Translation pack listing and installation APIs.
- Translation selector UI.
- Preferred translator applied across exercises.

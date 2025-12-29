# Phase 0: Architecture Setup

Document Version: 1.0
Date: 2024-12-28

## Purpose
Establish a shared, repo-accurate architecture baseline before adding new Flutter features or expanding the FFI surface. This phase aligns the plan with the actual codebase structure and confirms the core data flow from Flutter to Rust to SQLite and back.

## Goals
- Align the roadmap with the current repository layout and FFI entrypoint.
- Confirm data flow (Flutter -> FRB -> Rust -> SQLite -> Rust -> FRB -> Flutter).
- Define frontend module layout for exercises, debug tools, session flow, and services.
- Document repo-specific constraints and mismatches from the master plan.

## Dependencies
- None.

## Repo Notes (Critical Alignment)
- FRB entrypoint is configured in `flutter_rust_bridge.yaml` and points to `rust/crates/iqrah-api/`.
- The Rust API module to modify is `rust/crates/iqrah-api/src/api.rs` (not `flutter/rust/src/api.rs`).
- Generated Dart bindings live under `lib/rust_bridge/`.
- `lib/main.dart` initializes FRB via `RustLib.init()` and calls `setupDatabase()`.

## Acceptance Criteria
- `flutter_rust_bridge.yaml` pathing is confirmed correct.
- `lib/main.dart` data path and initialization flow are documented and verified.
- A frontend folder layout for new features is agreed and created (empty folders allowed).
- Architecture notes are recorded in `docs/frontend/PHASES_INDEX.md`.

## Task Breakdown

### Task 0.1: Architecture Baseline and Repo Alignment
- Verify FFI entrypoint and generated bindings.
- Confirm the runtime initialization sequence in `lib/main.dart`.

Files to reference:
- `flutter_rust_bridge.yaml`
- `rust/crates/iqrah-api/src/api.rs`
- `lib/rust_bridge/frb_generated.dart`
- `lib/main.dart`

Code example (existing entrypoint):
```dart
// lib/main.dart
Future<void> main() async {
  WidgetsFlutterBinding.ensureInitialized();
  await RustLib.init();
  await setupDatabase(
    contentDbPath: contentDbPath,
    userDbPath: userDbPath,
    kgBytes: bytes,
  );
  runApp(const ProviderScope(child: MyApp()));
}
```

### Task 0.2: Frontend Module Layout (Empty Structure)
Create these directories to keep phases clean and scoped:
- `lib/features/exercises/widgets/`
- `lib/features/debug/`
- `lib/features/session/`
- `lib/services/`
- `lib/models/`

### Task 0.3: Data Flow Checklist
Document the expected flow in this repo:
- Flutter calls `setupDatabase()` in `rust/crates/iqrah-api/src/api.rs`.
- `iqrah-storage` opens content DB and user DB in `init_content_db` and `init_user_db`.
- Exercise generation uses `ExerciseService::generate_exercise_v2()` and returns `ExerciseDataDto` via FRB.

Code example (FFI signature to verify):
```rust
// rust/crates/iqrah-api/src/api.rs
pub async fn setup_database(
    content_db_path: String,
    user_db_path: String,
    kg_bytes: Vec<u8>,
) -> Result<String> {
    // ...
}
```

## Testing Requirements
- Manual smoke test: app launches and prints DB init log in console.
- Optional: create a small Flutter test to call `getVerse()` after init.

Suggested command:
```bash
flutter test test/smoke/ffi_init_test.dart
```

## Estimated Effort
- 1 to 2 days.

## Deliverables
- Verified architecture notes (included in Phase index).
- Empty frontend module folders for subsequent phases.

# Frontend Implementation Phases Index

Document Version: 1.0
Date: 2024-12-28

## Overview
This index maps each phase document to its scope, dependencies, and deliverables. The phases are ordered and designed to be executed sequentially.

## Phase List

1. PHASE_0_ARCHITECTURE_SETUP
   - File: `docs/frontend/phases/PHASE_0_ARCHITECTURE_SETUP.md`
   - Goal: Align the plan with repo structure, verify init/data flow, and create the frontend module layout.
   - Dependencies: None

2. PHASE_1_FFI_FOUNDATION
   - File: `docs/frontend/phases/PHASE_1_FFI_FOUNDATION.md`
   - Goal: Expand FFI content access, fix WordInstance resolution, and add a Flutter service layer with tests.
   - Dependencies: Phase 0

3. PHASE_2_DEBUG_INFRASTRUCTURE
   - File: `docs/frontend/phases/PHASE_2_DEBUG_INFRASTRUCTURE.md`
   - Goal: Debug screens, node selector, energy snapshot, DB inspector, and logging.
   - Dependencies: Phase 1

4. PHASE_3_ECHO_RECALL_PERFECT
   - File: `docs/frontend/phases/PHASE_3_ECHO_RECALL_PERFECT.md`
   - Goal: Echo Recall renderer, progressive blurring, struggle detection, and metrics pipeline.
   - Dependencies: Phase 1 (Phase 2 recommended)

5. PHASE_4_FIVE_MORE_EXERCISES
   - File: `docs/frontend/phases/PHASE_4_FIVE_MORE_EXERCISES.md`
   - Goal: Implement five additional renderers using existing ExerciseDataDto variants.
   - Dependencies: Phase 1, Phase 3

6. PHASE_5_REMAINING_EXERCISES
   - File: `docs/frontend/phases/PHASE_5_REMAINING_EXERCISES.md`
   - Goal: Complete renderer coverage for all remaining exercise variants.
   - Dependencies: Phase 4

7. PHASE_6_SESSION_MANAGEMENT
   - File: `docs/frontend/phases/PHASE_6_SESSION_MANAGEMENT.md`
   - Goal: End-to-end session flow with persistence and summary.
   - Dependencies: Phase 1, Phase 4/5

8. PHASE_7_POLISH_PRODUCTION
   - File: `docs/frontend/phases/PHASE_7_POLISH_PRODUCTION.md`
   - Goal: Theme, error handling, performance, accessibility, and analytics.
   - Dependencies: Phase 6

9. PHASE_8_TRANSLATION_ADDONS
   - File: `docs/frontend/phases/PHASE_8_TRANSLATION_ADDONS.md`
   - Goal: Translation pack management and preferred translator selection.
   - Dependencies: Phase 1, Phase 7

## Repo-Specific Notes

### Phase 0 Architecture Baseline (VERIFIED AND COMPLETE)

**FFI Configuration:**

- FFI entrypoint: `rust/crates/iqrah-api/src/api.rs`
- FRB configuration: `flutter_rust_bridge.yaml`
- FRB Dart output: `lib/rust_bridge/`
- Generated bindings: `lib/rust_bridge/frb_generated.dart`

**Initialization Flow (`lib/main.dart`):**

1. `WidgetsFlutterBinding.ensureInitialized()`
2. `RustLib.init()` - Initialize Flutter Rust Bridge
3. Load knowledge graph asset bytes from `rootBundle.load(graphAssetPath)`
4. Setup database paths (`contentDbPath`, `userDbPath`)
5. `setupDatabase(contentDbPath, userDbPath, kgBytes)` - Initialize Rust backend

**Frontend Module Layout:**

- `lib/features/exercises/widgets/` - Exercise rendering components (ExerciseContainer exists)
- `lib/features/debug/` - Debug screens and tools (created in Phase 0)
- `lib/features/session/` - Session management UI (created in Phase 0)
- `lib/services/` - Flutter service layer (ExerciseContentService exists)
- `lib/models/` - Dart model classes (created in Phase 0)

**Existing Implementations:**

- `ExerciseContentService` (`lib/services/exercise_content_service.dart`) provides caching, TextVariant support, and batch fetching
- `ExerciseContainer` widget (`lib/features/exercises/widgets/exercise_container.dart`) demonstrates working Memorization, MCQ, and ClozeDeletion renderers

**Backend Notes:**

- Phase 4 adds new backend variants (`SequenceRecall`, `FirstWordRecall`) and expands `ExerciseDataDto` accordingly.

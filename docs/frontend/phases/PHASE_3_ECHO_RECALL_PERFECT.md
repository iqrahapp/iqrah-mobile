# Phase 3: Echo Recall (Perfect)

Document Version: 1.0
Date: 2024-12-28

## Purpose
Deliver a production-quality Echo Recall experience with progressive blurring, per-word timing, and struggle detection. This is the highest-priority exercise and must feel polished and reliable.

## Goals
- Expose Echo Recall stateful APIs via FRB.
- Implement a dedicated Flutter renderer with progressive blur stages.
- Track word-by-word timings and struggles.
- Persist energy updates and return metrics to the backend.

## Dependencies
- Phase 1 (FFI foundation)
- Phase 2 (debug tools) strongly recommended

## Acceptance Criteria
- Word-by-word reading is tracked with timing.
- Progressive blur transitions are smooth and consistent.
- Struggle detection triggers help and blur regression.
- Backend receives complete metrics and updates energy.
- Echo Recall can be launched via debug screen and session flow.

## Task Breakdown

### Task 3.1: Echo Recall FFI Surface
Expose Echo Recall state and actions in `rust/crates/iqrah-api/src/api.rs`.

Rust signatures:
```rust
pub async fn start_echo_recall(
    user_id: String,
    ayah_node_ids: Vec<String>,
) -> Result<EchoRecallStateDto>;

pub async fn submit_echo_recall(
    user_id: String,
    ayah_node_ids: Vec<String>,
    state: EchoRecallStateDto,
    word_node_id: String,
    recall_time_ms: u32,
) -> Result<EchoRecallStateDto>;

pub fn echo_recall_stats(state: EchoRecallStateDto) -> EchoRecallStatsDto;

pub async fn finalize_echo_recall(
    user_id: String,
    state: EchoRecallStateDto,
) -> Result<Vec<EnergyUpdateDto>>;
```

Data models to expose (from `iqrah-core`):
- `EchoRecallState`
- `EchoRecallWord`
- `WordVisibility`
- `Hint`
- `EchoRecallStats`

Note: If FRB ignores these types, annotate with `#[frb(unignore)]` and re-export in `iqrah-api`.

### Task 3.2: Flutter Models
Create Dart models or use generated FRB types to represent Echo Recall state.

Files to add:
- `lib/models/echo_recall_state.dart`

Example model wrapper:
```dart
class EchoRecallWordView {
  final String nodeId;
  final String text;
  final double energy;
  final WordVisibilityView visibility;

  const EchoRecallWordView({
    required this.nodeId,
    required this.text,
    required this.energy,
    required this.visibility,
  });
}
```

### Task 3.3: Echo Recall Renderer
Implement a dedicated widget under `lib/features/exercises/widgets/`.

Files to add:
- `lib/features/exercises/widgets/echo_recall_widget.dart`
- `lib/features/exercises/widgets/blurred_arabic_word.dart`

Widget skeleton:
```dart
class EchoRecallWidget extends StatefulWidget {
  final String userId;
  final List<String> ayahNodeIds;
  final void Function() onComplete;

  const EchoRecallWidget({
    required this.userId,
    required this.ayahNodeIds,
    required this.onComplete,
  });

  @override
  State<EchoRecallWidget> createState() => _EchoRecallWidgetState();
}
```

### Task 3.4: Progressive Blur Logic
Implement 5 to 6 blur levels and smooth transitions.

Blur levels (example):
- none
- center_light
- center_heavy
- ends
- extreme
- full

Dart helper skeleton:
```dart
enum BlurLevel { none, centerLight, centerHeavy, ends, extreme, full }

bool shouldBlur(int index, int length, BlurLevel level) {
  switch (level) {
    case BlurLevel.none:
      return false;
    case BlurLevel.centerLight:
      return index == length ~/ 2;
    case BlurLevel.full:
      return true;
    default:
      return false;
  }
}
```

### Task 3.5: Struggle Detection
Detect long gaps (> 5s) or explicit help requests.

Guidelines:
- If gap > 5000ms, regress blur level by one step.
- Show a subtle help hint (no modal by default).
- Record struggle events for backend metrics.

### Task 3.6: Metrics and Persistence
Send metrics and finalize energy updates via FFI.

Rust DTOs to define if needed:
```rust
pub struct WordTimingDto {
    pub word_node_id: String,
    pub duration_ms: u64,
}

pub struct EchoRecallMetricsDto {
    pub word_timings: Vec<WordTimingDto>,
    pub total_duration_ms: u64,
    pub struggles: u32,
}
```

Dart collection:
```dart
final wordTimings = state.words.map((w) => WordTimingDto(...)).toList();
await api.finalizeEchoRecall(userId: userId, state: echoState);
```

### Task 3.7: Polish and Animations
- Word highlight transitions on tap.
- Smooth blur transitions (200-300ms fade).
- Subtle completion animation and haptics.

## Testing Requirements
- Unit tests for blur logic.
- Integration test for FRB EchoRecall roundtrip.
- Manual QA on device for gestures and timing.

Suggested tests:
```bash
flutter test test/echo_recall/blur_logic_test.dart
flutter test integration_test/echo_recall_roundtrip_test.dart
```

## Estimated Effort
- 8 to 10 days.

## Deliverables
- Echo Recall FFI API and DTOs.
- Echo Recall Flutter renderer with timing + blur + struggle detection.
- Metrics pipeline to backend.

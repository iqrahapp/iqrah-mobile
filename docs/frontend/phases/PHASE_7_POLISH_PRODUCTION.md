# Phase 7: Polish and Production Readiness

Document Version: 1.0
Date: 2024-12-28

## Purpose
Stabilize the UI/UX, improve performance, add accessibility, and ensure the app is ready for real users.

## Goals
- Consistent visual system across screens.
- Robust error handling and offline-safe behavior.
- Performance improvements for exercise rendering.
- Accessibility for text size and screen readers.
- Analytics hooks for core events.

## Dependencies
- Phase 6 (session flow)
- Phase 4/5 (exercise coverage)

## Acceptance Criteria
- App runs at 60fps on mid-range device.
- Errors surface as user-friendly messages.
- Accessibility supports large fonts and screen readers.
- Analytics events fire for session start/complete.

## Task Breakdown

### Task 7.1: Theme and Typography
Create a shared theme file and standardize text styles.

Files to add/modify:
- `lib/theme/app_theme.dart`
- `lib/main.dart`

Dart skeleton:
```dart
class AppTheme {
  static ThemeData dark() {
    return ThemeData.dark(useMaterial3: true).copyWith(
      textTheme: const TextTheme(
        titleLarge: TextStyle(fontWeight: FontWeight.w600),
      ),
    );
  }
}
```

### Task 7.2: Error Handling
Normalize FFI errors and show friendly messages.

Files to modify:
- `lib/services/*`
- `lib/utils/app_logger.dart`

Approach:
- Map common FFI errors to user-facing strings.
- Use a single `ErrorBanner` widget.

### Task 7.3: Performance Optimization
- Cache verse and word lookups in services.
- Avoid rebuilding large widget trees (use `const` and memoization).

Files to modify:
- `lib/services/exercise_content_service.dart`
- `lib/features/exercises/widgets/*`

### Task 7.4: Accessibility
- Ensure RTL text uses proper direction.
- Support large text scale factors.
- Add semantics labels for buttons.

Files to modify:
- `lib/features/exercises/widgets/*`
- `lib/features/session/*`

### Task 7.5: Analytics Hooks
Add simple logging events (local only for now).

Files to modify:
- `lib/utils/app_logger.dart`
- `lib/features/session/session_screen.dart`

Example:
```dart
AppLogger.log(LogCategory.session, 'session_started');
```

## Testing Requirements
- Profile with Flutter DevTools for frame timings.
- Accessibility audit using Flutter semantics debugger.
- Error injection tests for FFI failures.

## Estimated Effort
- 8 to 10 days.

## Deliverables
- Unified theme and styles.
- Error banner patterns across screens.
- Performance optimizations and accessibility compliance.

# Phase 6: Session Management

Document Version: 1.0
Date: 2024-12-28

## Purpose
Implement a complete session flow (start -> practice -> complete) with persistent session state. Align the session model with existing `SessionService` and user DB schema.

## Goals
- Add explicit session APIs to the FFI surface.
- Persist session state and allow resume after app restart.
- Implement a session UI flow in Flutter.

## Dependencies
- Phase 1 (FFI foundation)
- Phase 4/5 (exercise renderers)

## Acceptance Criteria
- User can start a session and receive a session ID.
- Flutter can fetch next item and submit results.
- Session can be resumed after app restart.
- Session summary is displayed at completion.

## Repo Alignment Note
Current user DB schema contains `session_state` only. The master plan describes `sessions` and `session_items` tables, which do not exist in this repo. Choose one of the following:
- Option A: Extend the user DB schema with session tables and update `iqrah-storage` accordingly.
- Option B: Keep sessions as transient lists stored in `session_state` and compute summary client-side.

This phase assumes Option A for full session analytics.

## Task Breakdown

### Task 6.1: Extend User DB Schema (if Option A)
Add session tables and migration.

Files to modify:
- `rust/crates/iqrah-storage/migrations_user/` (new migration file)
- `rust/crates/iqrah-storage/src/user/repository.rs`

Schema sketch:
```sql
CREATE TABLE sessions (
  id TEXT PRIMARY KEY,
  user_id TEXT NOT NULL,
  goal_id TEXT NOT NULL,
  started_at INTEGER NOT NULL,
  completed_at INTEGER,
  items_count INTEGER NOT NULL,
  items_completed INTEGER NOT NULL
);

CREATE TABLE session_items (
  id INTEGER PRIMARY KEY,
  session_id TEXT NOT NULL,
  node_id INTEGER NOT NULL,
  exercise_type TEXT NOT NULL,
  grade TEXT NOT NULL,
  duration_ms INTEGER,
  completed_at INTEGER
);
```

### Task 6.2: FFI Session API
Expose session functions in `rust/crates/iqrah-api/src/api.rs`.

Rust signatures:
```rust
pub async fn start_session(user_id: String, goal_id: String) -> Result<SessionDto>;

pub async fn get_next_session_item(session_id: String) -> Result<Option<SessionItemDto>>;

pub async fn submit_session_item(
    session_id: String,
    node_id: String,
    grade: u8,
    duration_ms: u64,
) -> Result<String>;

pub async fn complete_session(session_id: String) -> Result<SessionSummaryDto>;
```

### Task 6.3: Flutter Session Service
Create a dedicated session service and provider.

Files to add/modify:
- `lib/services/session_service.dart`
- `lib/providers/session_provider.dart`
- `lib/features/session/session_screen.dart`
- `lib/features/session/session_summary_screen.dart`

Dart skeleton:
```dart
class SessionService {
  Future<SessionDto> startSession(String userId, String goalId) {
    return api.startSession(userId: userId, goalId: goalId);
  }

  Future<SessionItemDto?> getNextItem(String sessionId) {
    return api.getNextSessionItem(sessionId: sessionId);
  }
}
```

### Task 6.4: Resume Support
Persist the active session ID and state.

Files to modify:
- `lib/services/session_service.dart`
- `lib/features/session/session_screen.dart`

Approach:
- Store session ID in `SharedPreferences`.
- Call `get_next_session_item` on launch if a session is active.

## Testing Requirements
- Integration test: start session, complete one item, resume, complete session.
- Unit test: session provider state transitions.

Suggested commands:
```bash
flutter test test/session/session_flow_test.dart
```

## Estimated Effort
- 6 to 8 days.

## Deliverables
- Session APIs in Rust and Dart.
- Flutter session flow UI.
- Resume support and summary screen.

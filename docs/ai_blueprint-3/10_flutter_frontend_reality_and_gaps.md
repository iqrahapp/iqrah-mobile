# 10 - Flutter Frontend Reality And Gaps

This file captures what was missing from v3: a focused frontend truth map.

## 1) What Exists

- App bootstrap and DB setup are implemented (`lib/main.dart`, `AppInitializer`).
- Session state management exists with Riverpod (`lib/providers/session_provider.dart`).
- Exercise rendering pipeline exists (`ExercisePage`, `ExerciseContainer` and widgets).
- Basic auth state model/provider exists (`lib/providers/auth_provider.dart`).
- Sync provider/service/mapper exist and are wired to backend endpoints (`lib/providers/sync_provider.dart`, `lib/services/sync_service.dart`).

## 2) Critical Frontend Gaps

### F1 - Core Quran reader UX is still below target
- Missing polished word-by-word flow matching quran.com-level reading experience.
- Missing strong tap-on-word detail loop (root/occurrence exploration as first-class flow).

### F2 - Propagation feedback is not a primary user-visible moment
- Backend/core can compute propagation effects.
- UI does not strongly surface "learning this updated X related nodes" as motivation loop.

### F3 - Exercise quality inconsistency
- Many exercise variants exist in Rust, but frontend quality/UX parity is uneven.
- High-value exercises need intentional UX (especially lexical and contextual ones), not generic text inputs.

### F4 - Session entrypoint is simplistic
- `PracticePage` currently starts `goalId: "daily_review"` as static flow.
- No clear goal-focused mode for contiguous surah/chunk study at UI level.

### F5 - Audio capability uncertain in learning loop
- MediaKit is initialized at startup.
- End-to-end Quran recitation-assisted practice is not clearly productized in main path.

## 3) Important Correction vs Older Claims

Older docs stated backend was "not connected". That is now outdated.
- There is a working sync client path in Flutter for push/pull + timestamps.
- What remains is product hardening and proving real-user reliability, not "starting from zero".

## 4) Minimum Frontend Product Bar (for your vision)

A session and reading flow is acceptable only if user can:
1. Start session in under 2 taps.
2. See why each exercise is being asked (review/new/weak-root context).
3. Finish session and immediately see progress + propagation impact.
4. Open a surah, read word-by-word, tap any word, explore root occurrences.
5. Switch into focused chunk mode (contiguous ayah flow) when needed.

If any of these is missing, the app still feels like tooling, not product.

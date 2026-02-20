# 05 - Mobile Runtime Changes

## Target Behavior

On fresh install:
1. App starts with tiny bootstrap metadata only.
2. App fetches latest release manifest from backend.
3. App downloads required artifacts with resume support.
4. App verifies checksum for each artifact.
5. App atomically activates release.
6. Scheduling/practice runs locally and offline.

On update:
1. Background manifest check.
2. Download new release to staging directory.
3. Verify all required artifacts.
4. Swap active release pointer only when complete.

## Required New Components

1. `ReleaseBootstrapService` (Dart)
- Fetch latest release manifest.
- Decide install/update actions.

2. `ArtifactDownloader` (Dart)
- Range/resume download.
- Integrity verification.
- Retry policy and storage quota handling.

3. `LocalReleaseRegistry` (Rust or Dart persistence)
- Track active release id/version.
- Track installed artifacts.

4. `AtomicActivationManager`
- Stage to temp paths.
- Finalize by atomic rename/pointer swap.

## Current Startup Change

Current `lib/main.dart` copies bundled `rust/content.db`.  
Target: default path should use installed release artifact location and only use bundled fallback in explicit developer mode.

## Offline Guarantees

After at least one successful install:
1. No network required for normal learning sessions.
2. If update fails mid-way, previous active release remains usable.
3. Sync/auth failures never block practice mode.

## Telemetry To Add

1. release manifest fetch success/failure.
2. bytes downloaded and duration.
3. checksum failures.
4. activation success/failure.
5. currently active release version.

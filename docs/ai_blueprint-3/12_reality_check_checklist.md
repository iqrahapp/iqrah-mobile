# 12 - Reality-Check Checklist (Before Any Big Refactor)

Use this checklist before changing architecture. It avoids repeating agent drift.

## A) Runtime Truth Checks

1. Confirm which scheduler path is actually called in mobile session start.
2. Confirm whether goal IDs influence candidate selection at runtime.
3. Confirm exercise-router coverage for all exercise variants used in scheduled flow.
4. Confirm propagation events are surfaced to user, not only logged.
5. Confirm CBOR import persistence status before claiming live CBOR updates.

## B) Frontend Product Checks

1. Complete one full session on device (start -> N items -> summary) with no manual DB hacks.
2. Verify one tap-on-word flow shows root + meaning + cross-occurrence context.
3. Verify dashboard numbers reflect real DB state changes after session.
4. Verify sync push/pull changes local state correctly across two devices/simulated clients.
5. Verify app handles offline mode gracefully with no auth requirement for core learning.

## C) Pedagogy Checks

1. Verify session contains both continuity and lexical understanding work.
2. Verify fragile/high-value lexical items are explicitly prioritized.
3. Verify new users (zero states) still receive meaningful session items.
4. Verify chunk-focused mode is available for surah-contiguous memorization.
5. Verify at least one audio-assisted practice path exists in main user flow.

## D) "Do Not Claim" Rules

Do not claim any of these unless explicitly verified in current app build:
- "scheduler_v2 is what users run"
- "backend sync is not connected"
- "CBOR updates runtime graph in production path"
- "all exercise types are actively used in session scheduling"

# 07 - Migration And Cutover Plan

Goal: move from repo-embedded heavy artifacts to backend-hosted release artifacts safely.

## Phase A - Dual Mode (No Breaking Change)

1. Keep current bundled artifact flow.
2. Add backend release APIs and publish first release.
3. Add mobile bootstrap logic behind feature flag (`remote_release_bootstrap`).
4. Validate install/update in internal builds.

## Phase B - Default To Remote Bootstrap

1. Enable remote bootstrap by default for dev/staging.
2. Keep bundled fallback only for local emergency/dev mode.
3. Collect telemetry and error rates.

Exit criteria:
1. >99% successful fresh installs.
2. zero checksum mismatch activations.
3. no cold-start blocking due to bootstrap race.

## Phase C - Remove Heavy Repo Artifacts

1. Delete generated heavy files from mobile repo history going forward.
2. Keep only:
- tiny deterministic fixtures for tests,
- optional local dev fallback sample.
3. Update onboarding docs and scripts.

## Rollback Strategy

1. Backend rollback:
- mark problematic release deprecated,
- clients fetch previous stable release.

2. Mobile rollback:
- keep previous active release on disk,
- activate previous pointer if new activation fails.

## Operational Playbook

When shipping a new data release:
1. Generate artifacts.
2. Upload packs.
3. Create release draft.
4. Attach artifacts with roles.
5. Run validation endpoint.
6. Publish release.
7. Monitor bootstrap metrics.

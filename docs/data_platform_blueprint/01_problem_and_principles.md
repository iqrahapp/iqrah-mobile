# 01 - Problem And Principles

## Problem Statement

The current project has three structural issues:
1. Data ownership is unclear (mobile repo vs generated artifacts vs backend packs).
2. Runtime correctness is difficult to verify without manual end-to-end checks.
3. Artifact lifecycle (generate, validate, publish, consume, rollback) is not unified.

This causes drift, regressions, and operational fragility.

## Architecture Principles

1. Local-first runtime remains non-negotiable.
- Scheduling and practice run locally on-device.
- Network is optional after core artifacts are installed.

2. Backend is source of truth for distributable artifacts.
- KG/content/morphology/translations/audio indexes are published as versioned artifacts.
- Mobile repo should not carry large generated runtime data.

3. Reproducibility over convenience.
- Every release is immutable and checksummed.
- Every behavior change must be covered by deterministic automated tests.

4. Single release unit for multi-artifact consistency.
- A user should not receive a new KG with old morphology by accident.
- Release model must group compatible artifacts.

5. Safe rollout and rollback.
- New content releases must support canary and rollback.
- Mobile client must validate checksums before activating artifacts.

## What This Plan Is Not

1. It is not moving scheduling logic to backend.
2. It is not requiring always-online usage.
3. It is not replacing sync/auth backend contracts.

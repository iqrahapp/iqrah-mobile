# 03 - Backend Distribution Architecture

## Current Base (Already Exists)

`../iqrah-backend` already provides:
1. Pack registration/upload/publish admin endpoints.
2. Pack manifest/checksum/download public endpoints.
3. Versioned pack records with file integrity checks.

This is the right foundation for content distribution.

## Target Role Of Backend

Backend becomes the authoritative distribution layer for:
1. core content DB artifacts,
2. knowledge graph artifacts,
3. morphology artifacts,
4. translation packs,
5. audio metadata/index packs.

Backend does not run runtime scheduling for users.

## Core Components

1. Artifact Storage
- Immutable files with SHA-256 checksums.
- Versioned by pack and release.

2. Release Registry
- Groups multiple artifacts into one consistent published release.
- Supports `draft -> published -> deprecated`.

3. Validation Pipeline
- Schema/version checks.
- Required-artifact presence checks.
- Optional semantic sanity checks (counts, key metadata).

4. Public Delivery
- Manifest endpoints for client bootstrap.
- Range downloads + checksum verification.

5. Admin Plane
- Upload and attach artifacts to release.
- Validate release.
- Publish, disable, rollback.

## Data Ownership

1. Mobile repo owns source code and small fixtures.
2. Backend repo owns release metadata and artifact hosting.
3. Generation pipeline outputs artifacts, then publishes to backend.

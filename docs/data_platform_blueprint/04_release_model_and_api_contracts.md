# 04 - Release Model And API Contracts

## Why A Release Model

Pack-level publishing alone is insufficient for KG systems because clients need compatible sets of artifacts.

Example bad state:
- new KG + old morphology + old translation indexes.

A release groups compatible artifacts atomically.

## Proposed Tables (Backend)

1. `dataset_releases`
- `id` (uuid, pk)
- `version` (text, unique)
- `status` (`draft|published|deprecated`)
- `notes` (text, nullable)
- `created_at`, `published_at`
- `created_by`

2. `dataset_release_artifacts`
- `release_id` (fk -> dataset_releases.id)
- `package_id` (fk -> packs.package_id)
- `required` (bool)
- `artifact_role` (text enum)
- `created_at`
- unique `(release_id, package_id)`

Suggested `artifact_role`:
1. `core_content_db`
2. `knowledge_graph`
3. `morphology`
4. `translation_catalog`
5. `audio_catalog`
6. `optional_pack`

## Proposed Endpoints

Admin:
1. `POST /v1/admin/releases`
- create draft release.

2. `POST /v1/admin/releases/{id}/artifacts`
- attach package to release with role + required flag.

3. `POST /v1/admin/releases/{id}/validate`
- run validation and return failures/warnings.

4. `POST /v1/admin/releases/{id}/publish`
- publish atomically if validation passes.

5. `POST /v1/admin/releases/{id}/deprecate`
- stop future client assignment.

Public:
1. `GET /v1/releases/latest`
- return latest published release summary + required artifacts.

2. `GET /v1/releases/{id}/manifest`
- full artifact manifest for a release.

3. Keep existing:
- `GET /v1/packs/{id}/download`
- `GET /v1/packs/{id}/checksum`
- `GET /v1/packs/manifest`

## Validation Rules

A release publish must fail if:
1. required roles are missing (`core_content_db`, `knowledge_graph` at minimum),
2. any attached package is not published,
3. checksum or size metadata is missing,
4. artifact file is missing from storage.

Optional warnings:
1. morphology artifact missing when lexical features are enabled,
2. translation coverage below configured threshold.

## Backward Compatibility

1. Existing pack APIs remain valid.
2. Old clients can still fetch direct pack manifests.
3. New clients should prefer release manifest endpoints.

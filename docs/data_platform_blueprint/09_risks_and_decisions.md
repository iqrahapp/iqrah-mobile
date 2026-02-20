# 09 - Risks And Decisions

## Key Risks

1. Bootstrap fragility on unreliable networks.
- Mitigation: resumable downloads + atomic activation + fallback active release.

2. Data incompatibility between artifacts.
- Mitigation: release-level validation and required role checks.

3. Increased backend operational complexity.
- Mitigation: staged rollout, CLI-first admin tooling, strict audit logs.

4. Breaking legacy clients.
- Mitigation: keep existing pack endpoints and dual bootstrap mode during transition.

5. Over-centralization risk.
- Mitigation: preserve local-first runtime behavior and offline guarantees.

## Non-Negotiable Decisions

1. Backend distributes artifacts; mobile runs learning logic locally.
2. Release publish is blocked on validation failures.
3. Checksum verification is mandatory before activation.
4. Manual QA cannot be the primary regression barrier.

## Open Decisions

1. Artifact storage backend:
- filesystem vs object storage (S3-compatible).

2. Retention policy:
- number of historical releases to keep hot.

3. Rollout policy:
- all users at once vs phased cohort rollout.

4. Compatibility policy:
- minimum app version per release.

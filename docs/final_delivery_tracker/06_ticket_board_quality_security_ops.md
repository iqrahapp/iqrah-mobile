# 06 - Ticket Board: Quality, Security, Ops

## Quality And Regression Safety

- [ ] `Q-001` Add cross-repo regression matrix (core flow, bootstrap, sync)
- [ ] `Q-013` Artifact usage inventory and classification
  - Scope:
    - classify each large artifact as `runtime-required`, `r&d-generated`, `fixture`, or `release-asset`.
    - explicitly cover:
      - `research_and_dev/iqrah-knowledge-graph2/content.db`
      - `research_and_dev/iqrah-knowledge-graph2/content-fixed.db`
      - `research_and_dev/iqrah-knowledge-graph2/test-30-content.db`
      - `research_and_dev/iqrah-knowledge-graph2/knowledge-graph.cbor.zst`
      - `research_and_dev/iqrah-knowledge-graph2/test_output/*`
  - Accept:
    - written inventory committed in docs,
    - no deletion/untracking before this ticket is complete.

- [ ] `Q-014` Safe git-hygiene cleanup for generated artifacts
  - Scope:
    - add `.gitignore` rules for generated DB/CBOR/GraphML artifacts in R&D folders,
    - untrack generated artifacts that are not runtime-required,
    - keep runtime-required paths untouched.
  - Accept:
    - tracked generated-binary set reduced,
    - runtime and test flows still pass.

- [ ] `Q-002` Add performance baselines (startup, session generation latency, bootstrap duration)
- [ ] `Q-003` Add memory/storage footprint checks for artifact lifecycle
- [ ] `Q-004` Add crash/error taxonomy and structured logging standards

## Security And Integrity

- [ ] `Q-005` Enforce checksum verification at every artifact activation path
- [ ] `Q-006` Harden auth/sync token handling and failure states in mobile UX
- [ ] `Q-007` Add backend rate limits and abuse protections for admin/upload endpoints

## Observability And Operations

- [ ] `Q-008` Add production dashboards:
  - bootstrap success/failure,
  - sync lag/conflicts,
  - session completion and drop-off.

- [ ] `Q-009` Add alert thresholds and solo-operator incident checklist
- [ ] `Q-010` Add scripted rollback rehearsal (technical dry run + verification checklist)

## Documentation And Handoff

- [ ] `Q-011` Update developer onboarding docs for new release/bootstrap architecture
- [ ] `Q-012` Final architecture and operations signoff docs

## Exit Criteria

1. Regressions are caught automatically before merge.
2. Artifact integrity and sync safety are enforceable, not best effort.
3. Team can operate and recover production incidents confidently.

# Database v2 Implementation Specifications

**Last Updated:** 2025-11-17
**Status:** Implementation Ready

## Purpose

This folder contains **focused, implementation-ready specifications** for the database v2 redesign based on confirmed architectural decisions. Each document is designed to be assigned to an AI agent or developer as a single, self-contained implementation task.

## Context

These specs were created after:

**Gap Coverage:** See [GAP-COVERAGE.md](GAP-COVERAGE.md) for complete mapping of audit gaps to implementation specs.
1. Comprehensive database architecture audit (see parent folder)
2. Analysis of gaps between Python design and Rust implementation
3. Confirmation of design decisions (purist schema, normalized translators, etc.)

## Design Decisions Implemented

✅ **Purist Content DB** - No `node_id`, use natural keys (chapter_number, verse_key, word_id)
✅ **Normalized Translators** - Integer PKs, `languages` + `translators` tables
✅ **Fixed CHECK + NULL** - Correct `OR x IS NULL` pattern throughout
✅ **XOR + Partial Indexes** - Enforced for text_variants
✅ **Explicit CASCADE** - Documented for all foreign keys
✅ **Graph ID Stability** - Immutable node IDs with validation

## Implementation Specs

### Core Documents

| # | Document | Purpose | Priority | Effort |
|---|----------|---------|----------|--------|
| 00 | [00-overview.md](00-overview.md) | Architecture map & roadmap | Reference | N/A |
| 01 | [01-content-schema-v2-purist.md](01-content-schema-v2-purist.md) | **Authoritative content.db v2 schema** | P0 | 4-6 days |
| 02 | [02-translations-and-translators-normalization.md](02-translations-and-translators-normalization.md) | Normalized translator system | P1 | 3-5 days |
| 03 | [03-flexible-content-packages-plan.md](03-flexible-content-packages-plan.md) | 3-phase package management | P2-P4 | 2-3 weeks (Phase 3) |
| 04 | [04-versioning-and-migration-strategy.md](04-versioning-and-migration-strategy.md) | Schema versioning + graph migration | P0 | 2-3 days |
| 05 | [05-knowledge-axis-and-session-integration.md](05-knowledge-axis-and-session-integration.md) | Axis-specific exercises | P3 | 2 weeks |

## Implementation Roadmap

### Phase 1: Core Schema Migration (P0 - 1-2 weeks)

**Before Production Release**

Implement:
1. [01-content-schema-v2-purist.md](01-content-schema-v2-purist.md)
2. [02-translations-and-translators-normalization.md](02-translations-and-translators-normalization.md)
3. [04-versioning-and-migration-strategy.md](04-versioning-and-migration-strategy.md)

**Deliverables:**
- Content.db v2 with purist schema
- Multi-translation support (3-5 translations)
- Version tracking and validation
- Graph ID stability guarantee

### Phase 2: Multi-Translation UI (P1 - 3-5 days)

**MVP Enhancement**

- User translator selection UI
- Preference persistence
- Attribution display

**Note:** Already covered in doc 02

### Phase 3: Package System (P2-P4 - 2-3 weeks)

**Post-MVP**

Implement:
- [03-flexible-content-packages-plan.md](03-flexible-content-packages-plan.md) Phase 3

**Deliverables:**
- Downloadable content packages
- Audio recitations
- Package management UI

### Phase 4: Knowledge Axis (P3 - 2 weeks)

**Advanced Features**

Implement:
- [05-knowledge-axis-and-session-integration.md](05-knowledge-axis-and-session-integration.md)

**Deliverables:**
- Axis-specific exercises (memorization vs translation vs tajweed)
- Cross-axis learning synergies

## How to Use These Specs

### For AI Agents

Each document is:
- **Self-contained** - All information needed to implement
- **Atomic** - One document = one coherent feature
- **Testable** - Includes validation checklists
- **Estimable** - Effort estimates provided

**Process:**
1. Read entire document (Context + Goal + Design + Implementation Steps)
2. Implement according to numbered steps
3. Check validation checklist
4. Submit PR referencing the spec document

### For Project Leads

Use to:
- Assign tasks (one doc per agent/developer)
- Track progress (validation checklists)
- Estimate timelines (effort estimates included)
- Review PRs (ensure implementation matches spec)

## Relationship to Audit Documents

These specs are **derived from** the comprehensive audit in the parent folder:

**Audit Documents (Reference):**
- [../00-executive-summary.md](../00-executive-summary.md) - Original findings
- [../01-content-database.md](../01-content-database.md) - v1 content DB analysis
- [../02-user-database.md](../02-user-database.md) - User DB (still current)
- [../03-knowledge-graph.md](../03-knowledge-graph.md) - Graph structure
- [../04-database-interactions.md](../04-database-interactions.md) - DB interactions
- [../05-rust-implementation.md](../05-rust-implementation.md) - Module responsibilities
- [../06-knowledge-axis-design.md](../06-knowledge-axis-design.md) - Axis design gap
- [../07-navigation-and-algorithms.md](../07-navigation-and-algorithms.md) - Navigation strategies
- [../08-flexible-content-import.md](../08-flexible-content-import.md) - Package analysis
- [../09-gaps-and-recommendations.md](../09-gaps-and-recommendations.md) - Gap summary

**Use audit docs to:**
- Understand **why** v2 redesign is needed
- Understand **current state** of implementation
- See **detailed gap analysis**

**Use these specs to:**
- **Implement** the v2 redesign
- Get **concrete tasks** with step-by-step instructions
- Know **what "done" looks like**

## Success Metrics

### Phase 1 Complete ✅

- [ ] Content.db uses purist schema (no generic `nodes` table)
- [ ] Natural keys in use (chapter_number, verse_key, word_id)
- [ ] Translators normalized with integer PKs
- [ ] 3-5 translations shipped
- [ ] schema_version table exists and validated
- [ ] Graph stability validation in CI/CD
- [ ] All CHECK constraints use correct NULL handling
- [ ] All tests pass

### Phase 2 Complete ✅

- [ ] Users can select from multiple translators
- [ ] Preference persisted in user.db
- [ ] Translator attribution displayed in UI

### Phase 3 Complete ✅

- [ ] Users can browse package catalog
- [ ] Users can download packages
- [ ] Package installation/uninstallation works
- [ ] Audio recitations available

### Phase 4 Complete ✅

- [ ] KnowledgeAxis enum implemented
- [ ] Axis-specific exercises working
- [ ] Cross-axis synergies functional

## Priority Legend

- **P0:** Blocker for production (must implement before launch)
- **P1:** Critical for MVP (decide now)
- **P2:** Post-MVP enhancement (important but can wait)
- **P3:** Advanced features (not MVP blocker)
- **P4:** Future features (defer until demand)

## Questions?

**For implementation details:** Read the specific spec (01-05)
**For architecture context:** See [00-overview.md](00-overview.md)
**For understanding gaps:** See [../09-gaps-and-recommendations.md](../09-gaps-and-recommendations.md)
**For current implementation:** See [../05-rust-implementation.md](../05-rust-implementation.md)

---

**Status:** Ready for implementation
**Next Steps:** Start with Phase 1 (docs 01, 02, 04)

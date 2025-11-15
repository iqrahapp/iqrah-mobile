# Sprint 7: Database Split & Migration - Implementation Guide

## Mission
Transform the single-database architecture into a two-database system with proper migration framework.

## The Problem
- Single `iqrah.db` contains both immutable content AND mutable user data
- Risk of data loss when updating content
- Complex migrations for every schema change
- Cannot ship content updates independently

## The Solution
Split into two databases:
- **content.db** - Immutable knowledge graph (replaced on content updates)
- **user.db** - Mutable user progress (never overwritten)

## Implementation Steps

### Phase 1: Setup & Schema (Steps 1-2)
1. **Setup Workspace** - Create new Cargo workspace with 4 crates
2. **Database Schema** - Define schemas for both databases

### Phase 2: Core Implementation (Steps 3-5)
3. **Implement Core** - Domain logic and repository traits
4. **Implement Storage** - SQLx repositories for both databases
5. **Migration Harness** - Framework for user.db schema evolution

### Phase 3: Integration (Steps 6-7)
6. **Data Migration** - One-time migration from old database
7. **Update API** - Wire everything together in Flutter bridge

### Phase 4: Validation (Steps 8-9)
8. **Testing** - Unit, integration, and E2E tests
9. **Validation** - Final checks and cleanup

## Success Criteria

### Functional
- [ ] Two separate databases created (content.db, user.db)
- [ ] All existing features work identically
- [ ] User data migrated successfully
- [ ] app_settings table created (proves migration v2 ran)
- [ ] PRAGMA user_version = 2

### Non-Functional
- [ ] All tests pass (unit + integration)
- [ ] No compilation warnings
- [ ] Session generation works
- [ ] Review processing works
- [ ] Propagation logging works

## Execution Order

Follow the numbered steps in sequence:
1. `01-SETUP-WORKSPACE.md`
2. `02-DATABASE-SCHEMA.md`
3. `03-IMPLEMENT-CORE.md`
4. `04-IMPLEMENT-STORAGE.md`
5. `05-MIGRATION-HARNESS.md`
6. `06-DATA-MIGRATION.md`
7. `07-UPDATE-API.md`
8. `08-TESTING.md`
9. `09-VALIDATION.md`

## Estimated Timeline
- Phase 1: 1 day
- Phase 2: 3 days
- Phase 3: 2 days
- Phase 4: 1 day

**Total: ~7 days of focused work**

## Ready to Start?

Begin with `01-SETUP-WORKSPACE.md`

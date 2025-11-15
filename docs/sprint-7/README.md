# Sprint 7: Stability & Foundation - Implementation Guide

## Quick Start

Read the documents in order:

1. **[00-OVERVIEW.md](./00-OVERVIEW.md)** - Mission and success criteria
2. **[01-SETUP-WORKSPACE.md](./01-SETUP-WORKSPACE.md)** - Create Cargo workspace
3. **[02-DATABASE-SCHEMA.md](./02-DATABASE-SCHEMA.md)** - Define database schemas
4. **[03-IMPLEMENT-CORE.md](./03-IMPLEMENT-CORE.md)** - Domain logic and traits
5. **[04-IMPLEMENT-STORAGE.md](./04-IMPLEMENT-STORAGE.md)** - SQLx repositories
6. **[05-MIGRATION-HARNESS.md](./05-MIGRATION-HARNESS.md)** - Migration framework
7. **[06-DATA-MIGRATION.md](./06-DATA-MIGRATION.md)** - One-time data migration
8. **[07-UPDATE-API.md](./07-UPDATE-API.md)** - Flutter bridge integration
9. **[08-TESTING.md](./08-TESTING.md)** - Comprehensive tests
10. **[09-VALIDATION.md](./09-VALIDATION.md)** - Final checks

## What Gets Built

### New Structure
```
rust/crates/
├── iqrah-core/      # Domain logic (zero DB dependencies)
├── iqrah-storage/   # SQLx repositories
├── iqrah-api/       # Flutter bridge
└── iqrah-cli/       # Developer CLI
```

### Two Databases
- **content.db** - Immutable knowledge graph
- **user.db** - Mutable user progress

### Migration Framework
- SQLx migrations for user.db
- Automatic schema versioning
- One-time migration from old database

## Success Criteria

✅ Two databases created and functional
✅ All tests pass (unit + integration)
✅ Migration v2 ran (app_settings table exists)
✅ All existing features work
✅ No data loss during migration

## Execution Time

Estimated: ~7 days of focused work
- Phase 1 (Setup + Schema): 1 day
- Phase 2 (Core Implementation): 3 days
- Phase 3 (Integration): 2 days
- Phase 4 (Testing + Validation): 1 day

## Getting Started

```bash
# Start with Step 1
cd /home/user/iqrah-mobile
cat docs/sprint-7/01-SETUP-WORKSPACE.md
```

Follow each step sequentially and verify success criteria before proceeding.

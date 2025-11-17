# Database Architecture Overview

**Last Updated:** 2025-11-17
**Status:** v2 Design - Implementation Pending

## Context

The Iqrah Quranic learning application uses a **two-database architecture** that strictly separates concerns:
- **content.db** - Read-only Quranic content (shipped with app)
- **user.db** - Read-write user learning progress (created on device)

This document provides a high-level map of the database architecture and pointers to detailed design specifications.

## Core Architectural Principles

### 1. Purist Content Database (NEW in v2)

**Critical Design Decision:** The content.db must be **graph-agnostic**.

- **NO `node_id` coupling** with the knowledge graph
- Use **natural/content keys** as primary identifiers:
  - `chapters`: `chapter_number` (INTEGER PK)
  - `verses`: `verse_key` (TEXT PK, e.g., "1:1")
  - `words`: `word_id` (INTEGER PK) + `UNIQUE(verse_key, position)`
  - `lemmas/roots/stems`: semantic keys as PKs
- The **knowledge graph stores these content keys as properties** and joins via those
- Graph lives in separate layer; content DB is pure relational data

**Why?**
- Clean separation of concerns
- Content DB can be queried independently of graph
- Easier to understand, test, and maintain
- Graph can be rebuilt without touching content data

### 2. Two Database Files

```
User's Device:
/data/app/iqrah/
├── content.db    (Read-only, shipped with app, ~10-20 MB)
└── user.db       (Read-write, created on device, ~1-5 MB)
```

**No cross-database foreign keys** - coordination happens in application layer (Rust services).

### 3. Repository Pattern

```
Application Services (iqrah-core)
        ↓
Repository Traits (ports)
        ↓
SQLite Implementations (iqrah-storage)
        ↓
SQLite Databases (content.db, user.db)
```

- Services depend on traits, not concrete implementations
- Easy to mock for testing
- SQL completely isolated in repository layer

## Architecture Diagram

```
┌─────────────────────────────────────────────────────────────┐
│               Application Layer (Rust)                      │
│  LearningService, SessionService, ExerciseService           │
└────────────┬────────────────────────────────┬───────────────┘
             │                                │
             ▼                                ▼
┌────────────────────────┐      ┌──────────────────────────┐
│  ContentRepository     │      │  UserRepository          │
│  (trait in iqrah-core) │      │  (trait in iqrah-core)   │
└────────────┬───────────┘      └───────────┬──────────────┘
             │                              │
             ▼                              ▼
┌────────────────────────┐      ┌──────────────────────────┐
│ SqliteContentRepository│      │ SqliteUserRepository     │
│ (iqrah-storage)        │      │ (iqrah-storage)          │
└────────────┬───────────┘      └───────────┬──────────────┘
             │                              │
             ▼                              ▼
┌────────────────────────┐      ┌──────────────────────────┐
│   CONTENT.DB           │      │   USER.DB                │
│   (Shipped, Read-Only) │      │   (Device, Read-Write)   │
│                        │      │                          │
│   • chapters           │      │   • user_memory_states   │
│   • verses             │      │   • propagation_events   │
│   • words              │      │   • session_state        │
│   • lemmas/roots/stems │      │   • user_preferences     │
│   • morphology         │      │   • app_settings         │
│   • translators        │      └──────────────────────────┘
│   • verse_translations │
│   • text_variants      │
│   • reciters           │
│   • content_packages   │
└────────────────────────┘

┌─────────────────────────────────────────────────────────────┐
│        Knowledge Graph (Separate Layer)                    │
│  Stores content keys (chapter_number, verse_key, word_id)  │
│  Manages nodes, edges, distributions for learning algo     │
│  Generated in Python, imported via CBOR                    │
└─────────────────────────────────────────────────────────────┘
```

## Content DB v2 Schema (Purist Approach)

**Full specification:** [01-content-schema-v2-purist.md](01-content-schema-v2-purist.md)

### Core Tables

**Inflexible Data** (always shipped, rarely changes):
- `schema_version` - Content DB version tracking
- `chapters` - Chapter metadata (PK: `chapter_number`)
- `verses` - Verse text and metadata (PK: `verse_key`)
- `words` - Word instances with positions (PK: `word_id`)
- `morphology_segments` - Morphological analysis
- `lemmas` - Lemma forms (PK: `lemma_id`)
- `roots` - Morphological roots (PK: `root_id`)
- `stems` - Stem forms (PK: `stem_id`)

**Flexible Data** (user-selectable, managed via packages):
- `languages` - Language metadata (PK: `language_code`)
- `translators` - Translator metadata (PK: `translator_id` INTEGER)
- `verse_translations` - Verse translations (PK: `verse_key, translator_id`)
- `word_translations` - Word translations (PK: `word_id, translator_id`)
- `word_transliterations` - Transliterations (PK: `word_id, package_id`)
- `text_variants` - Alternative Arabic scripts (PK: `package_id, verse_key|word_id`)
- `reciters` - Reciter metadata (PK: `reciter_id`)
- `verse_recitations` - Audio metadata (PK: `package_id, verse_key`)
- `word_audio` - Word audio metadata (PK: `package_id, word_id`)
- `content_packages` - Package catalog (PK: `package_id`)
- `installed_packages` - Installation tracking (PK: `package_id`)

### Key Design Features

✅ **Natural primary keys** (chapter_number, verse_key, word_id)
✅ **Normalized translators** (integer FKs, not strings)
✅ **Correct CHECK constraints** (NULL handling fixed)
✅ **Partial unique indexes** (XOR constraints enforced)
✅ **Cascade semantics** (package deletion propagates)
✅ **Performance indexes** (on common query patterns)

## User DB Schema

**Current implementation:** [02-user-database.md](02-user-database.md) (from previous audit)

### Core Tables

- `user_memory_states` - FSRS parameters + energy for each content item (PK: `user_id, content_key`)
- `propagation_events` - Energy propagation audit trail
- `propagation_details` - Detailed propagation breakdown
- `session_state` - Ephemeral session resume state
- `user_preferences` - User settings (e.g., preferred `translator_id`)
- `app_settings` - App configuration (e.g., `schema_version`)
- `user_stats` - Statistics tracking

**Note:** User DB references content via **content keys** (verse_key, word_id, etc.), not graph node_id.

## Key Design Documents

### Implementation-Ready Specifications

| Document | Purpose | Status |
|----------|---------|--------|
| [01-content-schema-v2-purist.md](01-content-schema-v2-purist.md) | **Authoritative content.db v2 schema** | 📝 Ready for implementation |
| [02-translations-and-translators-normalization.md](02-translations-and-translators-normalization.md) | Normalized translator system with integer PKs | 📝 Ready for implementation |
| [03-flexible-content-packages-plan.md](03-flexible-content-packages-plan.md) | Package management system (3 phases) | 📝 Phased plan |
| [04-versioning-and-migration-strategy.md](04-versioning-and-migration-strategy.md) | Schema versioning + graph migration strategy | 📝 Ready for implementation |
| [05-knowledge-axis-and-session-integration.md](05-knowledge-axis-and-session-integration.md) | Knowledge axis implementation plan | 📝 Post-MVP feature |

### Reference Documents (from previous audit)

| Document | Purpose |
|----------|---------|
| [02-user-database.md](02-user-database.md) | User DB schema reference |
| [03-knowledge-graph.md](03-knowledge-graph.md) | Graph structure and CBOR import |
| [04-database-interactions.md](04-database-interactions.md) | How DBs interact (needs update for v2) |
| [05-rust-implementation.md](05-rust-implementation.md) | Module responsibilities |
| [07-navigation-and-algorithms.md](07-navigation-and-algorithms.md) | Navigation strategies |

## Implementation Roadmap

### Phase 1: Core Schema Migration (P0 - Before Production)

**Time Estimate:** 1 week

1. Implement `01-content-schema-v2-purist.md` → New content.db migrations
2. Implement `02-translations-and-translators-normalization.md` → Translator tables
3. Implement `04-versioning-and-migration-strategy.md` → Version tracking

**Deliverables:**
- New SQLx migrations for content.db v2
- Updated repository layer to use natural keys
- Python graph builder updated to reference content keys

### Phase 2: Multi-Translation Support (P1 - MVP Enhancement)

**Time Estimate:** 3-5 days

1. Implement `03-flexible-content-packages-plan.md` Phase 2 → Multi-translation without packages
2. Ship with 3-5 English translations
3. Add user preference UI

**Deliverables:**
- Users can select preferred translator
- Multiple translations available

### Phase 3: Full Package System (P2 - Post-MVP)

**Time Estimate:** 2-3 weeks

1. Implement `03-flexible-content-packages-plan.md` Phase 3 → Full package system
2. Downloadable translations, audio, text variants
3. Package management UI

**Deliverables:**
- Download infrastructure
- Package installation/uninstallation
- Audio recitation support

### Phase 4: Knowledge Axis Integration (P3 - Advanced Features)

**Time Estimate:** 2 weeks

1. Implement `05-knowledge-axis-and-session-integration.md`
2. Axis-specific exercises
3. Cross-axis learning synergies

**Deliverables:**
- Memorization vs translation vs tajweed exercise targeting
- Axis-aware session generation

## Critical Design Decisions Summary

### ✅ Confirmed Decisions

1. **Purist content DB** - No node_id, use natural keys
2. **Normalized translators** - Integer PKs, languages + translators tables
3. **Fixed CHECK constraints** - Proper NULL handling in all CHECKs
4. **Partial unique indexes** - XOR constraints for text_variants
5. **Cascade semantics** - Package deletion cascades to related data
6. **ID stability** - Graph node IDs never change once released

### 🔄 Migration from v1 to v2

**Breaking Changes:**
- `nodes` table removed → Use domain-specific tables (chapters, verses, words)
- `quran_text` table removed → Merge into `verses` and `words`
- `translations` table → Split into `verse_translations` with `translator_id` FK

**Data Migration:**
- Rebuild content.db from Python (simpler than migrating)
- User DB unaffected (will need to map old node_ids to new content keys in code)

## Navigation

**For Implementers:**
1. Read this overview
2. Pick a phase from the roadmap
3. Read the corresponding detailed spec document
4. Implement according to "Implementation Steps" section in that doc
5. Submit PR with reference to the spec document

**Next Steps:**
- [01-content-schema-v2-purist.md](01-content-schema-v2-purist.md) - Start here for content.db v2 implementation

---

**Last Updated:** 2025-11-17
**Next Review:** After Phase 1 implementation

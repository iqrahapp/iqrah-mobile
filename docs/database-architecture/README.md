# Database Architecture Documentation

**Last Updated:** 2025-11-17
**Status:** Comprehensive Audit + v2 Implementation Specs

## Overview

This folder contains two complementary sets of documentation:

1. **📊 Audit Documents** (this folder) - Comprehensive analysis of current state
2. **📋 v2 Implementation Specs** ([v2-implementation-specs/](v2-implementation-specs/)) - Focused implementation tasks

## Quick Navigation

**New to the project?**
→ Start with [00-executive-summary.md](00-executive-summary.md)

**Understanding gaps and why v2 is needed?**
→ Read [09-gaps-and-recommendations.md](09-gaps-and-recommendations.md)

**Ready to implement v2?**
→ Go to [v2-implementation-specs/](v2-implementation-specs/)

**Need to understand current Rust code?**
→ See [05-rust-implementation.md](05-rust-implementation.md)

## Folder Structure

```
docs/database-architecture/
│
├── README.md                           # This file
│
├── v2-implementation-specs/            # NEW: Implementation-ready specs
│   ├── README.md                       # Roadmap and implementation guide
│   ├── 00-overview.md                  # v2 Architecture overview
│   ├── 01-content-schema-v2-purist.md  # Authoritative schema spec (P0)
│   ├── 02-translations-normalization.md # Translator system spec (P1)
│   ├── 03-packages-plan.md             # Package system (P2-P4)
│   ├── 04-versioning-strategy.md       # Versioning & migration (P0)
│   └── 05-axis-integration.md          # Knowledge axis (P3)
│
├── 00-executive-summary.md             # Audit overview and key findings
├── 01-content-database.md              # Content DB analysis (v1)
├── 02-user-database.md                 # User DB analysis
├── 03-knowledge-graph.md               # Graph structure and CBOR import
├── 04-database-interactions.md         # How DBs interact
├── 05-rust-implementation.md           # Module responsibilities
├── 06-knowledge-axis-design.md         # Axis design gap analysis
├── 07-navigation-and-algorithms.md     # Navigation strategies
├── 08-flexible-content-import.md       # Package system analysis
└── 09-gaps-and-recommendations.md      # Gap summary with priorities
```

## Audit Documents (This Folder)

These documents provide **comprehensive analysis** of the current database architecture, identifying gaps between Python design and Rust implementation.

### Core Audit Documents

| Document | Purpose |
|----------|---------|
| [00-executive-summary.md](00-executive-summary.md) | Quick overview of findings, architecture, and gaps |
| [09-gaps-and-recommendations.md](09-gaps-and-recommendations.md) | **Start here** - Prioritized gap analysis and recommendations |

### Detailed Analysis

| Document | Topic |
|----------|-------|
| [01-content-database.md](01-content-database.md) | Content DB schema (v1), Python vs Rust comparison |
| [02-user-database.md](02-user-database.md) | User DB schema, FSRS + energy model, migrations |
| [03-knowledge-graph.md](03-knowledge-graph.md) | Graph structure, node IDs, CBOR import flow |
| [04-database-interactions.md](04-database-interactions.md) | How content.db and user.db interact, lookup paths |
| [05-rust-implementation.md](05-rust-implementation.md) | **Reference for code** - Module structure, repositories, services |
| [06-knowledge-axis-design.md](06-knowledge-axis-design.md) | Multi-dimensional learning model, implementation gaps |
| [07-navigation-and-algorithms.md](07-navigation-and-algorithms.md) | ID inference vs edge traversal, abstraction quality |
| [08-flexible-content-import.md](08-flexible-content-import.md) | Package management design vs implementation |

### Key Findings from Audit

**✅ Strengths:**
- Clean separation of content.db and user.db
- CBOR import working correctly
- Energy propagation functional
- FSRS integration solid
- Repository pattern well implemented

**❌ Critical Gaps:**
1. **Content DB couples to graph** - Uses generic `nodes` table with graph-specific node_id
2. **Knowledge axis unused** - Python generates axis nodes, Rust filters them out
3. **No graph migration strategy** - User progress at risk with node ID changes
4. **String-based translators** - Performance issues, no normalization
5. **Package system missing** - Designed in Python, not implemented in Rust

## v2 Implementation Specs

**Location:** [v2-implementation-specs/](v2-implementation-specs/)

These are **small, focused, implementation-ready documents** designed to be assigned to AI agents or developers. Each spec:
- Addresses specific gaps from the audit
- Provides step-by-step implementation instructions
- Includes validation checklists
- Estimates effort required

### Implementation Roadmap

**Phase 1: Core Schema (P0 - Before Production)**
- Purist content.db schema (natural keys, no node_id)
- Normalized translators (integer PKs)
- Version tracking and validation
- **Effort:** 1-2 weeks

**Phase 2: Multi-Translation (P1 - MVP)**
- Ship with 3-5 translations
- Translator selection UI
- **Effort:** 3-5 days

**Phase 3: Package System (P2-P4 - Post-MVP)**
- Downloadable packages
- Audio recitations
- **Effort:** 2-3 weeks

**Phase 4: Knowledge Axis (P3 - Advanced)**
- Axis-specific exercises
- Cross-axis synergies
- **Effort:** 2 weeks

**Full details:** [v2-implementation-specs/README.md](v2-implementation-specs/README.md)

## How to Use This Documentation

### Understanding Current State

1. Read [00-executive-summary.md](00-executive-summary.md) for overview
2. Read [09-gaps-and-recommendations.md](09-gaps-and-recommendations.md) for detailed gap analysis
3. Read specific audit docs (01-08) for deep dives on particular areas
4. Use [05-rust-implementation.md](05-rust-implementation.md) as reference for code locations

### Implementing v2 Redesign

1. Read [v2-implementation-specs/00-overview.md](v2-implementation-specs/00-overview.md) for architecture decisions
2. Pick a spec from [v2-implementation-specs/README.md](v2-implementation-specs/README.md) roadmap
3. Implement according to the spec's implementation steps
4. Check validation checklist before submitting PR
5. Reference the spec in your PR description

### For AI Agents

**Audit docs:** Read-only reference for understanding context
**Implementation specs:** Pick one, implement it, submit PR

Each implementation spec is designed as: **one agent → one spec → one PR**

## Design Decisions (Confirmed for v2)

These decisions are implemented in the v2 specs:

1. **✅ Purist Content DB** - No node_id, use natural keys (chapter_number, verse_key, word_id)
2. **✅ Normalized Translators** - Integer PKs, languages + translators tables
3. **✅ Fixed CHECK + NULL** - Correct `OR x IS NULL` pattern throughout
4. **✅ XOR + Partial Indexes** - Enforced for text_variants
5. **✅ Explicit CASCADE** - Documented for all FKs
6. **✅ Graph ID Stability** - Immutable node IDs with validation

**Full details:** [v2-implementation-specs/00-overview.md#confirmed-design-decisions](v2-implementation-specs/00-overview.md#confirmed-design-decisions)

## Questions?

| Question | Answer |
|----------|--------|
| What's wrong with current schema? | See [09-gaps-and-recommendations.md](09-gaps-and-recommendations.md) |
| How does current code work? | See [05-rust-implementation.md](05-rust-implementation.md) |
| What should v2 look like? | See [v2-implementation-specs/01-content-schema-v2-purist.md](v2-implementation-specs/01-content-schema-v2-purist.md) |
| How to implement v2? | See [v2-implementation-specs/README.md](v2-implementation-specs/README.md) |
| Where is X implemented? | See [05-rust-implementation.md](05-rust-implementation.md) for file locations |
| Why normalize translators? | See [v2-implementation-specs/02-translations-and-translators-normalization.md](v2-implementation-specs/02-translations-and-translators-normalization.md) |

## Contributing

**To audit documents:**
- These are reference/historical
- Update only if current state analysis changes
- Keep as comprehensive analysis

**To implementation specs:**
- Keep focused and atomic
- Include validation checklists
- Provide effort estimates
- Cross-reference audit docs for context

---

**Last Updated:** 2025-11-17
**Status:** Audit complete, v2 specs ready for implementation

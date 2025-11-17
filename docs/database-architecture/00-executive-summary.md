# Executive Summary: Iqrah Database Architecture

**Last Updated:** 2025-11-16
**Status:** Architecture Review

## Overview

The Iqrah Quranic learning application implements a **three-database architecture** that separates concerns between read-only Quranic content, mutable user learning progress, and a sophisticated knowledge graph structure.

## Architecture at a Glance

```
┌─────────────────────────────────────────────────────────────┐
│                    Application Layer                        │
│             (Rust: iqrah-core services)                     │
└────────────┬────────────────────────────────┬───────────────┘
             │                                │
             ▼                                ▼
┌────────────────────────┐      ┌──────────────────────────┐
│  ContentRepository     │      │  UserRepository          │
│  (iqrah-storage)       │      │  (iqrah-storage)         │
└────────────┬───────────┘      └───────────┬──────────────┘
             │                              │
             ▼                              ▼
┌────────────────────────┐      ┌──────────────────────────┐
│   CONTENT.DB           │      │   USER.DB                │
│   (Shipped, Read-Only) │      │   (Device, Read-Write)   │
│                        │      │                          │
│   • nodes              │      │   • user_memory_states   │
│   • edges              │      │   • propagation_events   │
│   • quran_text         │      │   • session_state        │
│   • translations       │      │   • user_stats           │
└────────────────────────┘      └──────────────────────────┘
```

## Three Database Components

### 1. Content Database (content.db)
- **Purpose:** Read-only Quranic knowledge graph and text
- **Storage:** Shipped with application
- **Schema:** nodes, edges, quran_text, translations
- **Location:** [migrations_content/20241116000001_content_schema.sql](../../rust/crates/iqrah-storage/migrations_content/20241116000001_content_schema.sql)

### 2. User Database (user.db)
- **Purpose:** User learning progress and state
- **Storage:** Created on user's device
- **Schema:** user_memory_states (FSRS + Energy), propagation_events, session_state
- **Location:** [migrations_user/20241116000001_user_schema.sql](../../rust/crates/iqrah-storage/migrations_user/20241116000001_user_schema.sql)

### 3. Knowledge Graph (Embedded in Content DB)
- **Purpose:** Structural and semantic relationships between Quranic nodes
- **Format:** CBOR (Compact Binary Object Representation)
- **Import:** Graph generated in Python, imported to Content DB via CBOR
- **Design:** [research_and_dev/iqrah-knowledge-graph2/](../../research_and_dev/iqrah-knowledge-graph2/)

## Key Design Principles

1. **Separation of Concerns:** Content vs. User data completely isolated
2. **Two Files:** content.db (read-only) + user.db (read-write)
3. **Repository Pattern:** Clean abstraction via ContentRepository and UserRepository traits
4. **FSRS Integration:** Spaced repetition with custom energy propagation
5. **Knowledge Axis Design:** Multi-dimensional learning (memorization, translation, tajweed, etc.)

## Critical Findings

### Strengths
- Clean database separation allows content updates without touching user data
- CBOR import working correctly for graph structure
- Energy propagation through graph edges functional
- FSRS integration solid
- Repository abstraction provides good testing and modularity

### Major Gaps

| Gap | Severity | Impact |
|-----|----------|--------|
| **Knowledge Axis Implementation** | CRITICAL | Python generates axis nodes (memorization, translation), Rust ignores them. No exercise targeting by axis. |
| **Flexible Content Packages** | HIGH | Comprehensive design for translations/audio packages exists in Python but zero implementation in Rust. |
| **Graph Migration Strategy** | HIGH | No handling of node ID changes across versions; user progress would be lost. |
| **Schema Mismatch** | MEDIUM | Python schema.py has rich morphology tables; Rust only has basic nodes/edges/text. |

## Quick Reference: Your Questions Answered

1. **Content DB Migrations (Flexible vs Inflexible)?** → No version table in Rust, no distinction. Python schema designs it but not implemented.
2. **Knowledge Graph ↔ Content DB Connection?** → String-based join via node IDs (e.g., "WORD_INSTANCE:1:1:3")
3. **Knowledge Graph Migrations & User Progression?** → User progress stored separately, but NO migration logic for node ID changes.
4. **User DB Migrations?** → Standard SQLx migrations, clean and functional.
5. **One Database or Multiple?** → TWO separate files (content.db + user.db)
6. **Flexible Data Import?** → Designed in Python, NOT implemented in Rust.
7. **Graph Navigation (ID inference vs edges)?** → ID inference for structure, edges for semantics. Well abstracted.
8. **Knowledge Axis Design Understanding?** → Python: excellent. Rust: significant gap.

## Document Index

- [01-content-database.md](01-content-database.md) - Content DB schema and design
- [02-user-database.md](02-user-database.md) - User DB schema and FSRS integration
- [03-knowledge-graph.md](03-knowledge-graph.md) - Graph structure and CBOR import
- [04-database-interactions.md](04-database-interactions.md) - How databases interact
- [05-rust-implementation.md](05-rust-implementation.md) - Module responsibilities
- [06-knowledge-axis-design.md](06-knowledge-axis-design.md) - Multi-axis learning design
- [07-navigation-and-algorithms.md](07-navigation-and-algorithms.md) - Graph traversal patterns
- [08-flexible-content-import.md](08-flexible-content-import.md) - Package management design
- [09-gaps-and-recommendations.md](09-gaps-and-recommendations.md) - Detailed gap analysis

## Recommended Next Steps

1. **Immediate:** Review knowledge axis gap - decide if it's needed for current scope
2. **Short-term:** Design graph migration strategy before first production release
3. **Medium-term:** Implement flexible content package system or simplify Python schema to match Rust
4. **Long-term:** Consider schema versioning strategy for both content and user DBs

---

**Navigation:** [Next: Content Database →](01-content-database.md)

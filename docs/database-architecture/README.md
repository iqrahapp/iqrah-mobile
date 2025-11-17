# Iqrah Database Architecture Documentation

**Last Updated:** 2025-11-16
**Status:** Comprehensive Architecture Review

## Overview

This folder contains a complete analysis of the Iqrah Quranic learning application's database architecture, covering the current design, implementation status, and gaps between the Python design and Rust implementation.

## Quick Start

**New to the project?** Start with:
1. [00-executive-summary.md](00-executive-summary.md) - High-level overview and key findings
2. [09-gaps-and-recommendations.md](09-gaps-and-recommendations.md) - What needs to be done

**Looking for specific information?** See the Quick Reference Guide below.

## Document Index

### Core Architecture

| Document | Description | Key Questions |
|----------|-------------|---------------|
| [00-executive-summary.md](00-executive-summary.md) | High-level overview, key findings, navigation guide | All questions (quick answers) |
| [01-content-database.md](01-content-database.md) | Content DB schema, Python vs Rust comparison | Q1: Migration strategy |
| [02-user-database.md](02-user-database.md) | User DB schema, FSRS + Energy model | Q4: User DB migrations |
| [03-knowledge-graph.md](03-knowledge-graph.md) | Graph structure, CBOR import, node IDs | Q3: Graph migrations |
| [04-database-interactions.md](04-database-interactions.md) | How DBs interact, lookup paths | Q2: Graph-DB connection, Q5: One or multiple DBs |
| [05-rust-implementation.md](05-rust-implementation.md) | Module responsibilities, class structure | General implementation reference |

### Advanced Topics

| Document | Description | Key Questions |
|----------|-------------|---------------|
| [06-knowledge-axis-design.md](06-knowledge-axis-design.md) | Multi-dimensional learning model, axis concept | Q8: Axis design understanding |
| [07-navigation-and-algorithms.md](07-navigation-and-algorithms.md) | ID inference vs edge traversal | Q7: Navigation strategy |
| [08-flexible-content-import.md](08-flexible-content-import.md) | Package system for translations/audio | Q6: Flexible content import |
| [09-gaps-and-recommendations.md](09-gaps-and-recommendations.md) | Gap analysis, prioritized action plan | All gaps with recommendations |

## Quick Reference Guide

### Your Original Questions Answered

1. **Q1: Migration strategy for flexible vs inflexible content?**
   - **Answer:** No distinction in current Rust implementation. Python designs it but not implemented.
   - **See:** [01-content-database.md](01-content-database.md#q1-migration-strategy)

2. **Q2: Knowledge graph ↔ Content DB connection?**
   - **Answer:** String-based join using node IDs (e.g., "WORD_INSTANCE:1:1:3")
   - **See:** [04-database-interactions.md](04-database-interactions.md#q2-knowledge-graph--content-db-connection)

3. **Q3: Graph migrations and user progression preservation?**
   - **Answer:** User progress stored separately, but NO migration logic for node ID changes
   - **See:** [03-knowledge-graph.md](03-knowledge-graph.md#q3-knowledge-graph-migrations--user-progression)

4. **Q4: User DB migration handling?**
   - **Answer:** Standard SQLx migrations, clean and functional
   - **See:** [02-user-database.md](02-user-database.md#q4-user-db-migration-strategy)

5. **Q5: One database or multiple files?**
   - **Answer:** TWO files (content.db + user.db)
   - **See:** [04-database-interactions.md](04-database-interactions.md#q5-database-file-architecture)

6. **Q6: Flexible content import (translations, scripts)?**
   - **Answer:** Designed in Python, NOT implemented in Rust
   - **See:** [08-flexible-content-import.md](08-flexible-content-import.md)

7. **Q7: ID inference vs edge traversal? Logic exposed or abstracted?**
   - **Answer:** ID inference for structure, edges for semantics. Well abstracted.
   - **See:** [07-navigation-and-algorithms.md](07-navigation-and-algorithms.md#q7-id-inference-vs-edge-traversal)

8. **Q8: Knowledge axis design understanding?**
   - **Answer:** Python: excellent understanding. Rust: significant gap.
   - **See:** [06-knowledge-axis-design.md](06-knowledge-axis-design.md)

### Critical Findings

**🔴 Critical Gaps:**
1. **Knowledge axis NOT implemented** - Nodes generated but filtered out of sessions
2. **No graph migration strategy** - User progress at risk with node ID changes
3. **Flexible content system missing** - Cannot add translations without full DB replacement

**🟡 Important Gaps:**
4. Schema mismatch between Python and Rust
5. No content version tracking
6. Package management not implemented

**✅ Working Well:**
- Database separation (content + user)
- CBOR import
- Energy propagation
- FSRS integration
- Repository abstraction
- SQLx migrations

### Key Architectural Decisions

**Two Databases:**
- `content.db` - Read-only, shipped with app
- `user.db` - Read-write, created on device

**Navigation Strategy:**
- ID inference for sequential navigation (fast, O(1))
- Edge traversal for semantic relationships (flexible, graph-based)

**Learning Model:**
- FSRS for spaced repetition (stability, difficulty)
- Custom energy for graph-based propagation

**Graph Structure:**
- Nodes: Chapter → Verse → Word → Morphology
- Edges: Dependency (structural) + Knowledge (semantic)
- Distributions: Const, Normal, Beta (probabilistic energy)

## File Locations Reference

### Rust Implementation

**Storage Layer:**
- Content DB: [rust/crates/iqrah-storage/src/content/](../../rust/crates/iqrah-storage/src/content/)
- User DB: [rust/crates/iqrah-storage/src/user/](../../rust/crates/iqrah-storage/src/user/)
- Migrations: [rust/crates/iqrah-storage/migrations_*/](../../rust/crates/iqrah-storage/)

**Core Layer:**
- Domain models: [rust/crates/iqrah-core/src/domain/](../../rust/crates/iqrah-core/src/domain/)
- Repository traits: [rust/crates/iqrah-core/src/ports/](../../rust/crates/iqrah-core/src/ports/)
- Services: [rust/crates/iqrah-core/src/services/](../../rust/crates/iqrah-core/src/services/)
- CBOR import: [rust/crates/iqrah-core/src/cbor_import.rs](../../rust/crates/iqrah-core/src/cbor_import.rs)

### Python Design

**Knowledge Graph:**
- Graph builder: [research_and_dev/iqrah-knowledge-graph2/src/iqrah/graph/](../../research_and_dev/iqrah-knowledge-graph2/src/iqrah/graph/)
- Schema design: [research_and_dev/iqrah-knowledge-graph2/src/iqrah/content/schema.py](../../research_and_dev/iqrah-knowledge-graph2/src/iqrah/content/schema.py)
- Axis definitions: [research_and_dev/iqrah-knowledge-graph2/src/iqrah/graph/knowledge.py](../../research_and_dev/iqrah-knowledge-graph2/src/iqrah/graph/knowledge.py)

## Action Items

**Before Production Release (P0):**
- [ ] Implement graph migration strategy (ID stability guarantee)
- [ ] Add content version tracking

**Decision Required (P1):**
- [ ] Knowledge axis support: Implement now or defer?

**Short-term (P2-P3):**
- [ ] Resolve schema mismatch (choose authoritative schema)
- [ ] Add multi-translation support (optional)

**Long-term (P4):**
- [ ] Full package system for flexible content
- [ ] Audio recitation support

See [09-gaps-and-recommendations.md](09-gaps-and-recommendations.md) for detailed action plan.

## Navigation

**Linear Reading (Recommended for First Time):**
1. Start: [00-executive-summary.md](00-executive-summary.md)
2. Follow the "Next →" links at the bottom of each document
3. End: [09-gaps-and-recommendations.md](09-gaps-and-recommendations.md)

**Topic-Based Reading:**
- **"How does it work now?"** → [05-rust-implementation.md](05-rust-implementation.md)
- **"What's missing?"** → [09-gaps-and-recommendations.md](09-gaps-and-recommendations.md)
- **"How should it work?"** → [06-knowledge-axis-design.md](06-knowledge-axis-design.md), [08-flexible-content-import.md](08-flexible-content-import.md)
- **"Database design?"** → [01-content-database.md](01-content-database.md), [02-user-database.md](02-user-database.md)

## Contributing

When updating this documentation:

1. **Maintain cross-references** - Use relative links to other docs
2. **Include file locations** - Always reference source code with paths and line numbers
3. **Update last updated date** - At top of each modified document
4. **Keep executive summary current** - It's the entry point

## Questions or Feedback

If you find:
- Outdated information
- Missing details
- Confusing explanations

Please update the relevant document and note the change in the executive summary.

---

**Last Updated:** 2025-11-16
**Documents:** 11 files (including this README)
**Total Analysis:** Comprehensive review of 3-database architecture with 8 detailed questions answered

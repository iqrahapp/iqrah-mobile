# Data Architecture v2: Production Design

**Version:** 2.0.0
**Last Updated:** 2024-11-24
**Status:** Authoritative

## Executive Summary

Iqrah's data architecture employs a strict separation of concerns between immutable Quranic content (shipped with the app) and mutable user learning progress (stored on the device). This design relies on a two-database system (`content.db` and `user.db`) coordinated by the application layer.

This document defines the authoritative contracts for Node IDs, which serve as the primary keys linking content, the knowledge graph, and user progress. **Stability of these IDs is critical**: once a Node ID is released in production, it must never change, as user learning history is keyed to it.

The architecture also introduces a "knowledge axis" concept, allowing users to track progress in multiple dimensions (memorization, translation, tajweed) for the same underlying content.

## Database Design

### Architecture Decision: 2 Databases

We use two separate SQLite database files:

1.  **Content Database (`content.db`)**:
    *   **Nature:** Read-only, immutable (except for package installations).
    *   **Distribution:** Shipped with the application bundle.
    *   **Contents:**
        *   Quranic text (Uthmani, etc.) and metadata (Chapters, Verses, Words).
        *   The Knowledge Graph (Nodes, Edges) defining learning dependencies.
        *   Linguistic data (Lemmas, Roots, Morphology).
        *   Available content packages (Translations, Audio).
    *   **Updates:** Updated by replacing the entire file during app updates (for core data) or by inserting rows (for downloadable packages).

2.  **User Database (`user.db`)**:
    *   **Nature:** Read-write, local to the user's device.
    *   **Distribution:** Created on first launch.
    *   **Contents:**
        *   User learning state (FSRS memory stability/difficulty, energy levels).
        *   Session history and exercise logs.
        *   User preferences and settings.
    *   **Updates:** Managed via standard SQL migrations.

**Rationale:**
*   **Separation of Concerns:** We can update the Quranic data and Knowledge Graph (e.g., adding new connections or fixing typos) without risking corruption of user data.
*   **Performance:** `content.db` can be optimized for read-heavy workloads (indexes, vacuumed), while `user.db` handles high-frequency writes.
*   **Simplification:** Avoids complex migration scripts for content data; we simply replace the reference data.

### Graph Update Strategy

The Knowledge Graph (nodes and edges) lives within `content.db`.

*   **Frequency:** Monthly (approximate cadence).
*   **Method:** **Erase & Replace**.
    *   The build pipeline regenerates the entire graph from source Python definitions.
    *   The new graph is imported into `content.db`.
    *   **Critical Constraint:** The new graph MUST preserve all existing Node IDs. New nodes can be added, but existing IDs cannot be changed or removed to ensure `user.db` references remain valid.

## Node ID Specification

Node IDs are the "foreign keys" that link the Content DB, Knowledge Graph, and User DB. They are string-based identifiers.

**⚠️ IMPORTANT:** All node IDs MUST use the **prefixed format**. Unprefixed IDs (e.g., "1:1") are strictly forbidden in the graph and user data.

### Content Node Formats

These nodes represent the actual Quranic entities.

| Entity | Format | Example | Description |
| :--- | :--- | :--- | :--- |
| **Chapter** | `CHAPTER:{number}` | `CHAPTER:1` | Surah number (1-114). |
| **Verse** | `VERSE:{chapter}:{verse}` | `VERSE:1:1` | Verse reference. |
| **Word** | `WORD:{word_id}` | `WORD:123` | Unique word ID from DB (autoincrement). *Note: Python tooling currently uses `WORD:{text}`, but production runtime expects integer-based IDs.* |
| **Word Instance** | `WORD_INSTANCE:{c}:{v}:{p}` | `WORD_INSTANCE:1:1:1` | Specific occurrence of a word at `position` (1-indexed) in a verse. |
| **Root** | `ROOT:{root_text}` | `ROOT:ktb` | Arabic root text. |
| **Lemma** | `LEMMA:{lemma_text}` | `LEMMA:kataba` | Dictionary form of the word. |

### Knowledge Node Formats

Knowledge nodes represent a specific *aspect* of learning a content node. They are "virtual" nodes in the graph that wrap a content node.

*   **Format:** `{base_node_id}:{axis}`
*   **Example:** `VERSE:1:1:memorization`

**Valid Axes:**

| Axis | Description | Applicable Nodes |
| :--- | :--- | :--- |
| `memorization` | Rote recall of the item. | Chapter, Verse, Word Instance |
| `translation` | Understanding the meaning. | Chapter, Verse, Word Instance, Word, Lemma |
| `tafsir` | Deep understanding/exegesis. | Chapter, Verse |
| `tajweed` | Correct pronunciation rules. | Verse, Word Instance |
| `contextual_memorization` | Recall within the sequence. | Word Instance |
| `meaning` | Definition of a root. | Root |

### Parsing and Validation Rules

1.  **Prefix Validation:** IDs must start with a valid prefix from the `NodeType` enum.
2.  **Delimiter:** Components are separated by `:`.
3.  **Numeric Validation:** Numeric parts (chapter, verse, position, word_id) must be parsed as integers and fall within valid ranges (e.g., Chapter 1-114).
4.  **Axis Validation:** If parsing a knowledge node, the suffix must match a valid `KnowledgeAxis`.
5.  **Strictness:** Parsers should reject malformed IDs immediately rather than attempting fuzzy matching.

## Node ID Stability Policy

### Guarantees

We promise users that their learning progress will never be lost due to technical updates. To fulfill this:

> **Once a Node ID is released in a production build, it is IMMUTABLE.**

*   It cannot be renamed.
*   It cannot be deleted.
*   It cannot change its meaning (e.g., `WORD:123` cannot point to a different word later).

### Rationale

The `user_memory_states` table in `user.db` is keyed by `node_id`. If we change `VERSE:1:1` to `V:1:1`, the user's memory stability history for Al-Fatihah is orphaned, effectively resetting their progress.

### Enforcement

*   **Task 1.5 (Stability Validation):** A build-time check will compare the generated graph against a "golden record" of released IDs. Any missing or changed IDs will fail the build.

### Migration Process

If a structural change is absolutely unavoidable (e.g., correcting a massive error in the Quran text dataset):

1.  **Deprecation:** The old node remains in the graph but is marked hidden/deprecated.
2.  **Migration Script:** A SQL migration for `user.db` must be written to move user progress from the old ID to the new ID.
3.  **Avoidance:** We prioritize adding *new* nodes over changing existing ones.

## Schema Versioning

We use **Semantic Versioning** (`major.minor.patch`) for the `content.db` schema, tracked in the `schema_version` table.

*   **Major (X.0.0):** Breaking changes to the schema structure (tables/columns removed) or Node ID format changes (requires user.db migration).
*   **Minor (0.X.0):** Graph updates (new edges, new nodes, weight adjustments) or new content packages. No breaking changes.
*   **Patch (0.0.X):** Data fixes (typos in translation, score corrections) that do not alter the graph structure significantly.

## Graph Update Process

The monthly graph update procedure:

1.  **Generate:** Python pipeline generates the new graph (CBOR format).
2.  **Validate:** Check for ID stability against the golden record.
3.  **Build Content DB:**
    *   Begin Transaction.
    *   Clear `edges`, `node_metadata`, `goals`, `node_goals` tables.
    *   Insert new graph data.
    *   Commit.
4.  **Ship:** Release updated `content.db` in the app bundle.
5.  **User Update:** On app launch, if the shipped `content.db` version > existing version, the app uses the new file. `user.db` remains untouched.

## Related Documentation

*   `docs/content-db-schema.md` - Detailed SQL schema reference.
*   `docs/database-architecture/00-executive-summary.md` - Architecture overview.
*   `docs/database-architecture/06-knowledge-axis-design.md` - Deep dive into Knowledge Axes.

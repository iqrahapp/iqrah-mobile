# Data Architecture v2: Production Design

**Version:** 2.0.0
**Last Updated:** 2024-11-24
**Status:** Authoritative

## Executive Summary
This document defines the production data architecture for the Iqrah mobile application. It establishes the **2-Database Design** (static content + dynamic user/graph data) and enforces **Strict Node ID Contracts** to ensure data integrity across the Python R&D pipeline and the Rust mobile core. It also defines the **Stability Policy** required to preserve user learning progress across updates.

## Database Design

### Architecture Decision: 2 Databases

We utilize a split-database architecture to separate immutable content from mutable user data and dynamic graph structures.

| Database | Type | Responsibilities | Update Strategy |
|----------|------|------------------|-----------------|
| **`content.db`** | Read-Only | • Quranic text (Uthmani, Simple)<br>• Translations & Transliterations<br>• Morphology (Roots, Lemmas)<br>• Audio timing data | **App Updates Only**<br>Shipped with APK/IPA or downloaded as a single artifact. |
| **`user.db`** | Read-Write | • User learning progress (FSRS states)<br>• Knowledge Graph structure (Edges)<br>• Node scores (PageRank)<br>• Learning goals<br>• Session history | **Hybrid**<br>• User data: Never deleted<br>• Graph data: Monthly erase/replace via migration |

**Rationale:**
1.  **Performance:** `content.db` can be optimized for read-heavy text queries (FTS5, specific page sizes).
2.  **Updates:** Translations and text corrections can be shipped without touching user progress.
3.  **Graph Evolution:** The knowledge graph (edges/scores) evolves faster than the text. Storing graph structure in `user.db` (or a dedicated attached DB) allows us to update the "brain" of the app without redownloading the heavy text content.

### Graph Update Strategy
The Knowledge Graph is updated monthly. Since the graph structure (edges) is derived from R&D, we use an **Erase & Replace** strategy for the graph tables within the writable database, while **strictly preserving** the `user_memory_states` table.

```sql
-- Example Graph Update Transaction
BEGIN TRANSACTION;
-- 1. Clear old graph structure
DELETE FROM edges;
DELETE FROM node_metadata;
DELETE FROM node_goals;

-- 2. Insert new graph structure (from CBOR/SQL import)
-- ... inserts ...

-- 3. Verify integrity (Task 1.5)
-- Ensure all node_ids in user_memory_states exist in new graph
COMMIT;
```

## Node ID Specification

Node IDs are the primary keys linking `content.db`, `user.db`, and the Knowledge Graph. They **MUST** be stable strings.

### Content Node Formats
These nodes represent physical/textual entities in the Quran.

| Type | Format | Example | Description |
|------|--------|---------|-------------|
| **Chapter** | `CHAPTER:{number}` | `CHAPTER:1` | Surah 1 (Al-Fatihah) |
| **Verse** | `VERSE:{chapter}:{verse}` | `VERSE:1:1` | Surah 1, Ayah 1 |
| **Word** | `WORD:{id}` | `WORD:123` | Unique word ID (from corpus) |
| **Word Instance** | `WORD_INSTANCE:{c}:{v}:{p}` | `WORD_INSTANCE:1:1:1` | Word at position 1 in 1:1 |
| **Root** | `ROOT:{text}` | `ROOT:كتب` | Arabic root (ktb) |
| **Lemma** | `LEMMA:{text}` | `LEMMA:كتب` | Dictionary form |

### Knowledge Node Formats
These nodes represent abstract learning concepts associated with a content node. They are formed by appending an **Axis** to a Content Node ID.

**Format:** `{ContentNodeID}:{Axis}`

| Axis | Description | Example |
|------|-------------|---------|
| `memorization` | Rote memorization of the item | `VERSE:1:1:memorization` |
| `translation` | Understanding the meaning | `WORD:123:translation` |
| `tafsir` | Deep exegesis/explanation | `VERSE:1:1:tafsir` |
| `tajweed` | Pronunciation rules | `WORD_INSTANCE:1:1:1:tajweed` |
| `contextual` | Flow/connection to neighbors | `WORD_INSTANCE:1:1:1:contextual` |

### Parsing and Validation Rules

1.  **Delimiter:** Always use colon (`:`) as the separator.
2.  **Prefix:** The first segment MUST be a valid `NodeType` (uppercase preferred in ID, handled case-insensitively in logic if needed, but strict uppercase is recommended for storage).
3.  **Validation:**
    *   `CHAPTER`: 1-114
    *   `VERSE`: Valid chapter, valid verse number for that chapter.
    *   `POSITION`: 1-indexed integer.

**Rust Parsing Logic (Target):**
```rust
// pseudo-code
let parts: Vec<&str> = id.split(':').collect();
match parts[0] {
    "VERSE" => { /* parse parts[1] as chapter, parts[2] as verse */ },
    "CHAPTER" => { /* parse parts[1] as number */ },
    // ...
    _ => return Err(InvalidNodeId)
}
```

## Node ID Stability Policy

### Guarantees
> **CRITICAL:** Once a Node ID is released in a production build, it **MUST NOT** change or be removed without a migration path.

**Why?**
The `user_memory_states` table keys user progress by `node_id`. If `VERSE:1:1` becomes `AYAH:1:1`, the user loses their memorization progress for that verse.

### Enforcement
1.  **Validation Pipeline (Task 1.5):** The Python build script will compare the new graph against the previous production graph.
2.  **Breaking Change Detection:**
    *   If a Node ID exists in `prev_graph` but is missing in `new_graph` -> **ERROR** (unless explicitly marked deprecated).
    *   If a Node ID changes format -> **ERROR**.

### Migration Process
If a breaking change is unavoidable (e.g., fixing a typo in a Root ID):
1.  Create a SQL migration file.
2.  `UPDATE user_memory_states SET node_id = 'NEW_ID' WHERE node_id = 'OLD_ID';`
3.  Ship migration with the app update.

## Schema Versioning

We use Semantic Versioning (`major.minor.patch`) for the data schema, tracked in the `schema_version` table in both DBs.

*   **Major (1.0.0 -> 2.0.0):** Breaking schema changes (table structure, column removals) or Node ID format changes. Requires app code update.
*   **Minor (1.1.0 -> 1.2.0):** New tables, new Node ID types (additive), or new graph content. Backward compatible.
*   **Patch (1.1.1 -> 1.1.2):** Data fixes, score adjustments, typo corrections.

## Related Documentation
*   [Database Architecture Audit](../database-architecture/README.md)
*   [Content DB Schema](../content-db-schema.md)
*   [Scheduler V2 Design](../todo/scheduler-v2-knowledge-graph.md)

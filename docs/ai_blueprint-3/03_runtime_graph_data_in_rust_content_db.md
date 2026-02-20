# 03 - Runtime Graph Data In `rust/content.db`

This section describes what the app actually loads and schedules from.

## 1) Main Runtime Source

App startup (`lib/main.dart`) copies bundled `rust/content.db` and uses it as primary content graph source.

CBOR import in Rust (`rust/crates/iqrah-core/src/cbor_import.rs`) currently parses CBOR but does not persist nodes/edges (`insert_*` calls are TODO/commented).

Implication: in normal runs, graph behavior is dictated by the prebuilt SQLite DB, not by live CBOR import.

## 2) How `content.db` Is Produced In Rust Tooling

Generator crate:
- `rust/crates/iqrah-gen/src/main.rs`
- `rust/crates/iqrah-gen/src/content.rs`
- `rust/crates/iqrah-gen/src/graph.rs`
- `rust/crates/iqrah-gen/src/knowledge.rs`

Observed characteristics:
- Builds dependency graph in Rust and writes directly to DB tables.
- Builds knowledge edges with fixed/simple weights (mostly 1.0, some 0.5 cross-axis).
- Computes simple metadata scores (type heuristic + in-degree log transform), not Python PageRank pipeline.

## 3) Actual DB Counts (from `rust/content.db`)

Observed via local sqlite query during audit:
- `nodes`: 276,367
- `edges`: 468,737
- `node_metadata`: 558,970
- `goals`: 0
- `node_goals`: 0

Node type bits from encoded IDs (`id >> 56`):
- type 1: 114
- type 2: 6,236
- type 4: 83,668
- type 5: 179,922
- type 6: 1,651
- type 7: 4,776

Notably absent: type 3 (`WORD`) nodes.

## 4) Data Shape Drift Found

### A) No dedicated `WORD` node type in shipped DB
- ID type 3 count is zero.
- Word-like behavior is mostly through `WORD_INSTANCE` and knowledge nodes.

### B) `nodes.node_type` column drift for lemmas
- Lemma IDs decode to type bits 7 (`LEMMA:*`).
- But `nodes.node_type` for these rows is `5` (same as knowledge rows).
- This can confuse any SQL path that relies on `node_type` column rather than decoding ID bits.

### C) Verse ukey style mismatch
- Base verse rows in `nodes` use unprefixed keys like `1:1`.
- Runtime canonical formatting often expects `VERSE:1:1` from node-id utility functions.

### D) Metadata keys present
- `foundational_score` and `influence_score` for all nodes
- `quran_order` for verse nodes only (count 6,236)
- no `difficulty_score` present

### E) Goal tables empty
- `goals` and `node_goals` are empty in current bundled DB.
- Advanced goal-based scheduler candidate query cannot operate meaningfully without seeding these tables.

## 5) Edge Structure Snapshot

- edge_type 0 (dependency): 199,139
- edge_type 1 (knowledge): 269,598

Dominant relation patterns:
- `4 -> 5` (word_instance to lemma/knowledge)
- `4 -> 2` (word_instance to verse)
- `2 -> 1` (verse to chapter)
- `5 -> 5` (knowledge-to-knowledge)
- `5 -> 6` (lemma to root)

## 6) Practical Implication

The runtime graph is coherent enough to run the app, but it is a simplified and somewhat inconsistent representation relative to the Python R&D pipeline.

This is a major source of "backend feels Ferrari, frontend/scheduling feels underpowered" because the shipped runtime graph and scheduler wiring are not matching the full modeling ambition.
